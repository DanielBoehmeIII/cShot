"""Prompt-to-sound: parse natural language, map to profiles, generate, diagnose, refine."""

import json
import math
import random
import re
import sys
import time
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.io import write_wav, read_wav
from gen.features import (
    compute_features,
    compute_spectral_centroid,
    compute_hpr,
    compute_spectral_bandwidth,
    compute_early_rms,
    compute_stereo_correlation,
    compute_noise_floor_estimate,
)

# ─── Adjective → control mappings ───────────────────────
# Each adjective maps to parameter overrides that generators actually use.
# The overrides must be compatible with the target generator's profile parameters.

ADJECTIVE_MAP = {
    # Brightness / spectral tilt
    "bright": {"brightness": 1.6, "high_shelf_db": 8.0, "filter_cutoff_end": 1.0},
    "dark": {"brightness": 0.3, "high_shelf_db": -8.0, "filter_cutoff_end": 0.25},
    "warm": {"brightness": 0.65, "low_shelf_db": 5.0, "high_shelf_db": -3.0},
    "mellow": {"brightness": 0.55, "saturation": 0.05, "high_shelf_db": -3.0},
    "edgy": {"brightness": 1.5, "saturation": 0.5, "high_shelf_db": 6.0},
    "crisp": {"brightness": 1.4, "attack_ms": 2.0, "high_shelf_db": 5.0},

    # Attack / velocity
    "soft": {"attack_ms": 30.0, "velocity": 0.15, "hammer_noise_amp": 0.02, "saturation": 0.0},
    "hard": {"attack_ms": 1.0, "velocity": 0.95, "hammer_noise_amp": 0.25, "saturation": 0.5},
    "punchy": {"attack_ms": 2.0, "saturation": 0.5, "compression": 0.6, "transient_boost": 1.0},
    "gentle": {"attack_ms": 30.0, "saturation": 0.0, "velocity": 0.2},
    "aggressive": {"attack_ms": 1.0, "drive": 1.0, "saturation": 0.8, "velocity": 1.0},

    # Character / timbre
    "distorted": {"saturation": 0.95, "drive": 1.2, "distortion": 0.8},
    "clean": {"saturation": 0.0, "noise_floor": 0.0, "distortion": 0.0, "drive": 0.0, "lo_fi": 0.0},
    "lo_fi": {"saturation": 0.3, "bit_depth": 8, "noise_floor": 0.03, "bandwidth_reduce": 0.45, "lo_fi": 0.5},
    "glitchy": {"grain_ms": 10.0, "bit_depth": 6, "decay_rate": 0.7},
    "metallic": {"brightness": 1.5, "high_shelf_db": 8.0, "saturation": 0.3, "resonance_boost": 0.5},

    # Space / width
    "wide": {"stereo_width": 0.85, "stereo_detune": 3.0},
    "narrow": {"stereo_width": 0.0, "stereo_detune": 0.0},
    "big": {"stereo_width": 0.7, "sustain_level": 0.5, "release_ms": 600},
    "small": {"stereo_width": 0.0, "sustain_level": 0.0, "release_ms": 30},
    "intimate": {"stereo_width": 0.0, "sustain_level": 0.1, "release_ms": 80},

    # Duration
    "short": {"duration_scale": 0.4, "sustain_level": 0.0},
    "long": {"duration_scale": 1.8, "sustain_level": 0.5, "release_ms": 800},
    "sustained": {"sustain_level": 0.7, "release_ms": 800},
    "staccato": {"sustain_level": 0.0, "release_ms": 15},

    # Texture
    "airy": {"noise_mix": 0.4, "high_shelf_db": 8.0, "brightness": 1.3},
    "noisy": {"noise_mix": 0.6, "saturation": 0.4, "noise_floor": 0.04},
    "smooth": {"saturation": 0.05, "attack_ms": 18.0, "brightness": 0.75},
    "rough": {"saturation": 0.6, "attack_ms": 1.0, "brightness": 1.4},

    # Pitch
    "high": {"pitch_scale": 2.0},
    "low": {"pitch_scale": 0.5},
    "mid": {"pitch_scale": 1.0},
}

# Primary instrument nouns (higher priority than style modifiers)
PRIMARY_NOUNS = {
    "piano", "keys", "synth", "bass", "808", "reese", "sub",
    "guitar", "nylon", "acoustic",
    "kick", "snare", "clap", "hat", "hihat", "open_hat",
    "impact", "fx", "riser", "glitch", "noise", "vinyl", "texture",
}

NOUN_MAP = {
    "piano": ("piano-gen", "acoustic"),
    "keys": ("piano-gen", "acoustic"),
    "stab": ("synth-gen", "stab"),
    "synth": ("synth-gen", "stab"),
    "pluck": ("synth-gen", "pluck"),
    "pad": ("synth-gen", "pad"),
    "lead": ("synth-gen", "lead"),
    "chord": ("synth-gen", "chord"),
    "bass": ("bass-gen", "808"),
    "808": ("bass-gen", "808"),
    "reese": ("bass-gen", "reese"),
    "sub": ("bass-gen", "808"),
    "guitar": ("guitar-gen", "nylon"),
    "nylon": ("guitar-gen", "nylon"),
    "acoustic": ("guitar-gen", "nylon"),
    "impact": ("fx-gen", "impact"),
    "fx": ("fx-gen", "impact"),
    "riser": ("fx-gen", "riser"),
    "glitch": ("fx-gen", "glitch"),
    "noise": ("fx-gen", "noise_hit"),
    "vinyl": ("fx-gen", "vinyl"),
    "texture": ("fx-gen", "air"),
    "kick": ("batch", "kick"),
    "snare": ("batch", "snare"),
    "clap": ("batch", "clap"),
    "hat": ("batch", "closed_hat"),
    "hihat": ("batch", "closed_hat"),
    "open_hat": ("batch", "open_hat"),
}

FAMILY_GENERATORS = {
    "piano-gen": ("gen.piano_gen", "synthesize_piano_stab", "PIANO_PROFILES"),
    "synth-gen": ("gen.synth_gen", "synthesize_synth", "SYNTH_PROFILES"),
    "bass-gen": ("gen.bass_gen", "synthesize_bass", "BASS_PROFILES"),
    "guitar-gen": ("gen.guitar_gen", "synthesize_guitar_stab", "GUITAR_PROFILES"),
    "fx-gen": ("gen.fx_gen", "synthesize_fx", "FX_PROFILES"),
    "batch": ("gen.synthesis", None, "SYNTHESIS_CLASSES"),
}


def parse_prompt(prompt: str) -> dict:
    """Parse a natural language prompt into (family, profile, overrides).
    
    Priority: primary instrument nouns > style modifiers.
    "piano stab" → piano-gen acoustic (not synth-gen stab).
    """
    words = prompt.lower().split()
    adjectives = []
    noun = None
    style_modifier = None

    for w in words:
        w_clean = w.strip(".,!?:;")
        if w_clean in ADJECTIVE_MAP:
            adjectives.append(w_clean)
        elif w_clean in NOUN_MAP:
            if w_clean in PRIMARY_NOUNS:
                noun = w_clean
            elif noun is None:
                noun = w_clean
            elif noun in PRIMARY_NOUNS:
                style_modifier = w_clean
            else:
                noun = w_clean

    if not noun:
        for word in words:
            for noun_key in NOUN_MAP:
                if noun_key in word:
                    noun = noun_key
                    break
            if noun:
                break

    if not noun:
        noun = "stab"

    family, default_profile = NOUN_MAP[noun]
    overrides = {}

    for adj in adjectives:
        mapping = ADJECTIVE_MAP[adj]
        overrides.update(mapping)

    # Style modifier can adjust profile within same family
    profile = default_profile
    if style_modifier and style_modifier in NOUN_MAP and family == NOUN_MAP[style_modifier][0]:
        profile = NOUN_MAP[style_modifier][1]

    return {
        "family": family,
        "default_profile": profile,
        "adjectives": adjectives,
        "noun": noun,
        "overrides": overrides,
        "raw_prompt": prompt,
    }


def generate_from_prompt(parsed: dict, count: int = 1, out_dir: Path = None) -> list[Path]:
    """Generate audio from a parsed prompt."""
    family = parsed["family"]
    profile_name = parsed["default_profile"]
    overrides = parsed["overrides"]

    if family == "piano-gen":
        from gen.piano_gen import synthesize_piano_stab, PIANO_PROFILES
        pf = PIANO_PROFILES.get(profile_name, PIANO_PROFILES["acoustic"])
        default_dur = pf["default_duration_ms"]
        default_pitch = pf["default_pitch_hz"]
        gen_fn = lambda dur, pitch: synthesize_piano_stab(dur, pitch, profile_name, overrides=overrides)
    elif family == "synth-gen":
        from gen.synth_gen import synthesize_synth, SYNTH_PROFILES
        pf = SYNTH_PROFILES.get(profile_name, SYNTH_PROFILES["stab"])
        default_dur = pf["default_duration_ms"]
        default_pitch = pf["default_pitch_hz"]
        gen_fn = lambda dur, pitch: synthesize_synth(dur, pitch, profile_name, overrides=overrides)
    elif family == "bass-gen":
        from gen.bass_gen import synthesize_bass, BASS_PROFILES
        pf = BASS_PROFILES.get(profile_name, BASS_PROFILES["808"])
        default_dur = pf["default_duration_ms"]
        default_pitch = pf["default_pitch_hz"]
        gen_fn = lambda dur, pitch: synthesize_bass(dur, pitch, profile_name, overrides)
    elif family == "guitar-gen":
        from gen.guitar_gen import synthesize_guitar_stab, GUITAR_PROFILES
        pf = GUITAR_PROFILES.get(profile_name, GUITAR_PROFILES["nylon"])
        default_dur = pf["default_duration_ms"]
        default_pitch = pf["default_pitch_hz"]
        gen_fn = lambda dur, pitch: synthesize_guitar_stab(dur, pitch, profile_name)
    elif family == "fx-gen":
        from gen.fx_gen import synthesize_fx, FX_PROFILES
        pf = FX_PROFILES.get(profile_name, FX_PROFILES["impact"])
        default_dur = pf["default_duration_ms"]
        default_pitch = pf["default_pitch_hz"]
        gen_fn = lambda dur, pitch: synthesize_fx(dur, pitch, profile_name, overrides)
    elif family == "batch":
        from gen.synthesis import SYNTHESIS_CLASSES
        if profile_name not in SYNTHESIS_CLASSES:
            return []
        _, synth_fn, default_dur, default_pitch = SYNTHESIS_CLASSES[profile_name]
        gen_fn = lambda dur, pitch: synth_fn(dur, pitch)
    else:
        return []

    duration_scale = overrides.pop("duration_scale", 1.0)
    pitch_scale = overrides.pop("pitch_scale", 1.0)
    dur = default_dur * duration_scale
    pitch = default_pitch * pitch_scale

    seed_offset = int(time.time() * 1000) % 1000000
    out_dir = out_dir or Path("outputs/prompt")
    out_dir.mkdir(parents=True, exist_ok=True)

    paths = []
    for i in range(count):
        seed = (seed_offset + i) * 314159265 + hash(parsed["raw_prompt"]) % 1000000
        random.seed(seed)
        np.random.seed(seed % 2**32)

        dur_var = dur * (1.0 + (random.random() - 0.5) * 0.08)
        # Piano: pitch-locked (minimal variation)
        if family == "piano-gen":
            pitch_var = pitch * (1.0 + (random.random() - 0.5) * 0.02)
        else:
            pitch_var = pitch * (1.0 + (random.random() - 0.5) * 0.08)
        samples = gen_fn(dur_var, pitch_var)

        safe_name = re.sub(r'[^a-z0-9]+', '_', parsed["raw_prompt"].lower())[:35].strip("_")
        out_path = out_dir / f"prompt_{safe_name}_{i+1:03d}.wav"
        write_wav(out_path, samples)
        paths.append(out_path)

    return paths


def compute_aggregate_features(wav_paths: list[Path]) -> dict:
    """Compute mean/variance of key features across a set of wav files."""
    all_feats = []
    for w in wav_paths:
        result = read_wav(w, mono=False)
        if result is None:
            continue
        samples, sr = result
        if samples.ndim == 2:
            stereo_corr = compute_stereo_correlation(samples)
            mono_samples = samples.mean(axis=1)
        else:
            stereo_corr = 1.0
            mono_samples = samples
        feats = compute_features(mono_samples, sr)
        feats["stereo_correlation"] = stereo_corr
        all_feats.append(feats)

    if not all_feats:
        return {}

    agg = {}
    keys = [
        "spectral_centroid", "spectral_bandwidth", "attack_ms",
        "early_rms", "rms", "peak", "zero_crossing_rate",
        "hpr", "low_band_energy", "high_band_energy",
        "noise_floor", "stereo_correlation",
    ]
    for k in keys:
        vals = [f.get(k, 0) for f in all_feats]
        agg[f"{k}_mean"] = float(np.mean(vals))
        agg[f"{k}_std"] = float(np.std(vals)) if len(vals) > 1 else 0.0

    agg["count"] = len(all_feats)
    return agg


# ─── Pairwise contrast tests ────────────────────────────

CONTRAST_TESTS = {
    "bright_vs_dark": {
        "label": "bright vs dark → centroid difference",
        "pair": ("bright", "dark"),
        "feature": "spectral_centroid_mean",
        "check": lambda b, d: b > d * 1.3,
        "min_diff_pct": 30,
    },
    "soft_vs_punchy": {
        "label": "soft vs punchy → attack time difference",
        "pair": ("soft", "punchy"),
        "feature": "attack_ms_mean",
        "check": lambda s, p: s > p * 2.0,
        "min_diff_pct": 100,
    },
    "soft_vs_punchy_rms": {
        "label": "soft vs punchy → early RMS difference",
        "pair": ("soft", "punchy"),
        "feature": "early_rms_mean",
        "check": lambda s, p: p > s * 1.08,
        "min_diff_pct": 8,
    },
    "clean_vs_distorted": {
        "label": "clean vs distorted → centroid difference",
        "pair": ("clean", "distorted"),
        "feature": "spectral_centroid_mean",
        "check": lambda c, d: d > c * 1.1,
        "min_diff_pct": 10,
    },
    "narrow_vs_wide": {
        "label": "narrow vs wide → stereo correlation difference",
        "pair": ("narrow", "wide"),
        "feature": "stereo_correlation_mean",
        "check": lambda n, w: n > w * 1.05,
        "min_diff_pct": 5,
    },
    "lo_fi_vs_clean": {
        "label": "lo-fi vs clean → bandwidth difference",
        "pair": ("lo_fi", "clean"),
        "feature": "spectral_bandwidth_mean",
        "check": lambda l, c: c > l * 1.2,
        "min_diff_pct": 20,
    },
}

DESCRIPTOR_PAIRS_COMMON = [
    ("bright", "dark"),
    ("soft", "punchy"),
    ("clean", "distorted"),
    ("narrow", "wide"),
    ("lo_fi", "clean"),
]


def run_contrast_test(adj_a: str, adj_b: str, family: str = "synth-gen",
                      profile: str = "pluck", count: int = 5,
                      work_dir: Path = None) -> dict:
    """Generate samples with two contrasting adjectives and compare features."""
    from gen.io import write_wav

    work_dir = work_dir or Path("/tmp/cshot_contrast")
    a_dir = work_dir / f"{adj_a}_vs_{adj_b}" / adj_a
    b_dir = work_dir / f"{adj_a}_vs_{adj_b}" / adj_b
    a_dir.mkdir(parents=True, exist_ok=True)
    b_dir.mkdir(parents=True, exist_ok=True)

    prompt_a = f"{adj_a} {profile}"
    prompt_b = f"{adj_b} {profile}"

    # Force family/profile if needed
    if family == "piano-gen":
        prompt_a = f"{adj_a} piano"
        prompt_b = f"{adj_b} piano"
    elif family == "bass-gen":
        prompt_a = f"{adj_a} bass"
        prompt_b = f"{adj_b} bass"

    parsed_a = parse_prompt(prompt_a)
    parsed_b = parse_prompt(prompt_b)

    paths_a = generate_from_prompt(parsed_a, count, a_dir)
    paths_b = generate_from_prompt(parsed_b, count, b_dir)

    feats_a = compute_aggregate_features(paths_a)
    feats_b = compute_aggregate_features(paths_b)

    return {
        "adj_a": adj_a,
        "adj_b": adj_b,
        "prompt_a": prompt_a,
        "prompt_b": prompt_b,
        "family": family,
        "profile": profile,
        "count": count,
        "features_a": feats_a,
        "features_b": feats_b,
    }


def check_contrast(result: dict) -> list[dict]:
    """Check if contrast test results pass the threshold checks.
    Each test's check function receives (val_for_adj_a, val_for_adj_b).
    The check should return True when the contrast is meaningful.
    """
    fa = result["features_a"]
    fb = result["features_b"]
    checks = []

    for test_name, test in CONTRAST_TESTS.items():
        pair = test["pair"]
        if result["adj_a"] != pair[0] or result["adj_b"] != pair[1]:
            continue
        feat = test["feature"]
        val_a = fa.get(feat, 0)
        val_b = fb.get(feat, 0)
        if val_a == 0 and val_b == 0:
            checks.append({
                "test": test_name,
                "label": test["label"],
                "passed": False,
                "reason": f"feature {feat} both zero",
                "val_a": val_a,
                "val_b": val_b,
            })
        else:
            passed = test["check"](val_a, val_b)
            denom = max(min(val_a, val_b), 1e-10)
            pct_diff = abs(val_a - val_b) / denom * 100 if denom > 0 else 0
            checks.append({
                "test": test_name,
                "label": test["label"],
                "passed": passed,
                "val_a": round(val_a, 4),
                "val_b": round(val_b, 4),
                "diff_pct": round(pct_diff, 1),
                "min_diff_pct": test["min_diff_pct"],
            })

    return checks


# ─── CLI commands ───────────────────────────────────────

def cmd_prompt(args):
    """Generate from a natural language prompt."""
    prompt = " ".join(args.prompt) if isinstance(args.prompt, list) else args.prompt
    parsed = parse_prompt(prompt)

    print(f"Prompt: {parsed['raw_prompt']}")
    print(f"  → Family: {parsed['family']}")
    print(f"  → Profile: {parsed['default_profile']}")
    print(f"  → Nouns: {[parsed['noun']]}")
    print(f"  → Adjectives: {parsed['adjectives']}")
    if parsed['overrides']:
        print(f"  → Overrides: {json.dumps(parsed['overrides'], indent=4)}")

    out_dir = Path(args.out) if args.out else Path("outputs/prompt")
    count = args.count
    paths = generate_from_prompt(parsed, count, out_dir)

    print(f"\n  Generated {len(paths)} file(s) → {out_dir}")
    for p in paths:
        print(f"    {p.name}")

    # Print feature summary
    if paths:
        feats = compute_aggregate_features(paths)
        print(f"\n  Feature summary:")
        for k in ["spectral_centroid_mean", "attack_ms_mean", "early_rms_mean",
                   "hpr_mean", "spectral_bandwidth_mean", "noise_floor_mean",
                   "stereo_correlation_mean"]:
            if k in feats:
                print(f"    {k}: {feats[k]:.4f}")


def cmd_prompt_refine(args):
    """Refine a prompt-based generation: diagnose + regenerate."""
    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = sorted(in_dir.glob("*.wav"))
    if not wav_files:
        print(f"Error: no .wav files", file=sys.stderr)
        sys.exit(1)

    prompt = args.prompt or in_dir.name

    print(f"Prompt: {prompt}")
    print(f"Files: {len(wav_files)}")
    print()

    all_issues = set()
    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            continue
        samples, sr = result
        feats = compute_features(samples, sr)
        centroid = feats["spectral_centroid"]

        prompt_lower = prompt.lower()

        if "bright" in prompt_lower and centroid < 3000:
            all_issues.add("not_bright_enough")
        if "dark" in prompt_lower and centroid > 5000:
            all_issues.add("not_dark_enough")
        if "soft" in prompt_lower and feats.get("attack_ms", 0) < 3:
            all_issues.add("too_hard")
        if "hard" in prompt_lower and feats.get("attack_ms", 0) > 10:
            all_issues.add("too_soft")
        if "bass" in prompt_lower and feats["low_band_energy"] < 0.3:
            all_issues.add("lacks_low_end")
        if "metallic" in prompt_lower and centroid < 4000:
            all_issues.add("not_metallic_enough")

    if all_issues:
        print("Diagnosed issues:")
        for issue in all_issues:
            print(f"  ✗ {issue}")
        print()
    else:
        print("  ✓ No issues detected from prompt analysis.\n")

    fixes = {
        "not_bright_enough": ("Increase brightness", {"brightness": 1.6, "high_shelf_db": 6.0, "filter_cutoff_end": 1.0}),
        "not_dark_enough": ("Reduce brightness", {"brightness": 0.25, "high_shelf_db": -6.0, "filter_cutoff_end": 0.2}),
        "too_hard": ("Soften attack", {"attack_ms": 25.0, "velocity": 0.2, "saturation": 0.0}),
        "too_soft": ("Harden attack", {"attack_ms": 1.0, "velocity": 0.9, "saturation": 0.4}),
        "lacks_low_end": ("Boost low end", {"low_shelf_db": 5.0, "sub_body_balance": 0.8}),
        "not_metallic_enough": ("Add metallic character", {"brightness": 1.5, "high_shelf_db": 8.0}),
    }

    if all_issues and args.out:
        out_dir = Path(args.out)
        out_dir.mkdir(parents=True, exist_ok=True)

        merged_overrides = {}
        for issue in all_issues:
            if issue in fixes:
                _, fix_overrides = fixes[issue]
                merged_overrides.update(fix_overrides)

        parsed = parse_prompt(prompt)
        parsed["overrides"].update(merged_overrides)

        print(f"Generating {args.count} refined samples → {out_dir}...")
        new_paths = generate_from_prompt(parsed, args.count, out_dir)
        for p in new_paths:
            print(f"  {p.name}")
        print(f"Done.")
    elif all_issues:
        for issue in all_issues:
            if issue in fixes:
                desc, overrides = fixes[issue]
                print(f"  → {desc}: {overrides}")
        print()
        print(f"  Run with --out <dir> to regenerate with these fixes.")


def cmd_compare_prompt(args):
    """Compare feature reports between two generated directories."""
    dir_a = Path(args.dir_a)
    dir_b = Path(args.dir_b)

    wavs_a = sorted(dir_a.glob("*.wav"))
    wavs_b = sorted(dir_b.glob("*.wav"))

    if not wavs_a or not wavs_b:
        print("Error: one or both directories have no .wav files", file=sys.stderr)
        sys.exit(1)

    print(f"\n  Comparing:")
    print(f"    A: {dir_a} ({len(wavs_a)} files)")
    print(f"    B: {dir_b} ({len(wavs_b)} files)")
    print()

    feats_a = compute_aggregate_features(wavs_a)
    feats_b = compute_aggregate_features(wavs_b)

    compare_keys = [
        ("spectral_centroid_mean", "Spectral Centroid (Hz)", "higher = brighter"),
        ("spectral_bandwidth_mean", "Spectral Bandwidth (Hz)", "higher = wider spectrum"),
        ("attack_ms_mean", "Attack Time (ms)", "lower = faster attack"),
        ("early_rms_mean", "Early RMS Ratio", "higher = punchier attack"),
        ("hpr_mean", "Harmonic/Noise Ratio", "higher = more harmonic, less noisy"),
        ("rms_mean", "RMS Level", "higher = louder"),
        ("noise_floor_mean", "Noise Floor", "higher = noisier"),
        ("stereo_correlation_mean", "Stereo Correlation", "lower = wider"),
        ("zero_crossing_rate_mean", "Zero Crossing Rate", "higher = more high freq"),
        ("high_band_energy_mean", "High Band Energy", "higher = brighter"),
        ("low_band_energy_mean", "Low Band Energy", "higher = bassier"),
    ]

    print(f"  {'Feature':<35s} {'A':>12s} {'B':>12s} {'Diff':>10s} {'Direction':>20s}")
    print(f"  {'─'*35} {'─'*12} {'─'*12} {'─'*10} {'─'*20}")

    diffs = {}
    for key, label, direction in compare_keys:
        va = feats_a.get(key, 0)
        vb = feats_b.get(key, 0)
        if va == 0 and vb == 0:
            continue
        denom = max(min(va, vb), 1e-10)
        pct = abs(va - vb) / denom * 100 if denom > 0 else 0
        arrow = "↑" if va > vb else "↓"
        print(f"  {label:<35s} {va:>10.4f}  {vb:>10.4f}  {arrow}{pct:>7.1f}%  {direction:>20s}")
        diffs[key] = {"a": round(va, 4), "b": round(vb, 4), "diff_pct": round(pct, 1)}

    print()
    notable = {k: v for k, v in diffs.items() if v["diff_pct"] > 15}
    if notable:
        print(f"  Notable differences (>15%):")
        for k, v in notable.items():
            print(f"    {k:<35s} A={v['a']:>10.4f}  B={v['b']:>10.4f}  ({v['diff_pct']:.1f}%)")
    else:
        print(f"  No notable differences (>15%) detected — adjectives may not be working.")


def cmd_contrast_test(args):
    """Run pairwise contrast tests to verify adjectives make audible differences."""
    family = args.family or "synth-gen"
    profile = args.profile or "pluck"
    count = args.count or 5
    work_dir = Path(args.out) if args.out else Path("/tmp/cshot_contrast")

    pairs_to_test = args.pairs if args.pairs else DESCRIPTOR_PAIRS_COMMON

    print("=" * 65)
    print("  PROMPT CONTRAST TEST: Do adjectives actually change the sound?")
    print("=" * 65)
    print(f"  Generator: {family} / {profile}")
    print(f"  Samples per adjective: {count}")
    print(f"  Pairs to test: {len(pairs_to_test)}")
    print()

    all_results = {}
    all_checks = {}
    total_passed = 0
    total_checks = 0

    for adj_a, adj_b in pairs_to_test:
        adj_a = adj_a.lower()
        adj_b = adj_b.lower()

        print(f"  Testing: [{adj_a}] vs [{adj_b}]")
        print(f"  {'─'*55}")

        result = run_contrast_test(adj_a, adj_b, family, profile, count, work_dir)
        checks = check_contrast(result)

        key = f"{adj_a}_vs_{adj_b}"
        all_results[key] = result
        all_checks[key] = checks

        fa = result["features_a"]
        fb = result["features_b"]

        # Print key feature comparisons
        for k in ["spectral_centroid_mean", "attack_ms_mean", "early_rms_mean",
                   "hpr_mean", "spectral_bandwidth_mean", "noise_floor_mean",
                   "stereo_correlation_mean"]:
            va = fa.get(k, 0)
            vb = fb.get(k, 0)
            if va or vb:
                arrow = "↑" if va > vb else "↓" if vb > va else "="
                print(f"    {k:<30s} {adj_a}: {va:>10.4f}  {adj_b}: {vb:>10.4f}  {arrow}")

        print()
        passed = 0
        for c in checks:
            icon = "✓" if c["passed"] else "✗"
            if c["passed"]:
                passed += 1
            print(f"    {icon} {c['label']:<50s} "
                  f"a={c.get('val_a', '?'):>8}  b={c.get('val_b', '?'):>8}  "
                  f"diff={c.get('diff_pct', 0):.0f}%")

        total_checks += len(checks)
        total_passed += passed

        pair_status = "PASS" if passed == len(checks) else f"{passed}/{len(checks)} PASS"
        print(f"  → {pair_status}")
        print()

    # Summary
    print(f"  {'='*55}")
    print(f"  CONTRAST TEST SUMMARY")
    print(f"  {'='*55}")
    pct = total_passed / max(total_checks, 1) * 100
    print(f"  {total_passed}/{total_checks} checks passed ({pct:.0f}%)")
    if total_passed == total_checks:
        print(f"  STATUS: ALL CONTRASTS PASS ✓ — adjectives make measurable differences")
    elif pct >= 60:
        print(f"  STATUS: MOST CONTRASTS PASS △ — some adjectives need tuning")
    else:
        print(f"  STATUS: TOO MANY CONTRASTS FAIL ✗ — adjectives not making measurable differences")

    # Save results
    report = {
        "family": family,
        "profile": profile,
        "samples_per_adj": count,
        "pairs_tested": pairs_to_test,
        "total_checks": total_checks,
        "passed": total_passed,
        "pass_rate_pct": round(pct, 1),
        "results": {
            k: {
                "prompt_a": v["prompt_a"],
                "prompt_b": v["prompt_b"],
                "features_a": v["features_a"],
                "features_b": v["features_b"],
                "checks": all_checks[k],
            }
            for k, v in all_results.items()
        },
    }

    report_path = work_dir / "contrast_test_report.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2))
    print(f"\n  Full report: {report_path}")


def cmd_mvp_audit(args):
    """Final MVP audit: generate across all categories with stricter descriptor and piano checks."""
    print("=" * 60)
    print("  cShot MVP AUDIT (STRICT)")
    print("=" * 60)
    print()

    out_dir = Path(args.out) if args.out else Path("outputs/mvp_audit")
    out_dir.mkdir(parents=True, exist_ok=True)

    categories = [
        ("kick", "batch kick", 5),
        ("snare", "batch snare", 5),
        ("clap", "batch clap", 5),
        ("hat", "batch closed_hat", 5),
        ("808", "bass-gen 808", 5),
        ("bass_stab", "bass-gen reese", 5),
        ("piano", "piano-gen acoustic", 10),
        ("synth", "synth-gen stab", 10),
        ("guitar", "guitar-gen nylon", 10),
        ("fx", "fx-gen impact", 10),
        ("pluck", "synth-gen pluck", 5),
        ("pad", "synth-gen pad", 5),
        ("lead", "synth-gen lead", 5),
        ("bell", "piano-gen bell", 5),
        ("rhodes", "piano-gen rhodes", 5),
        ("distorted_bass", "bass-gen distorted", 5),
        ("fm_bass", "bass-gen fm", 5),
        ("bright_guitar", "guitar-gen bright", 5),
        ("riser", "fx-gen riser", 5),
        ("glitch", "fx-gen glitch", 5),
    ]

    total_target = sum(c[2] for c in categories)
    print(f"Target: {total_target} files across {len(categories)} categories\n")

    results = {}
    total_generated = 0
    total_errors = 0

    for cat_name, prompt_template, count in categories:
        cat_dir = out_dir / cat_name
        cat_dir.mkdir(parents=True, exist_ok=True)

        try:
            if prompt_template.startswith("batch"):
                parts = prompt_template.split()
                class_name = parts[1]
                from gen.synthesis import SYNTHESIS_CLASSES
                if class_name not in SYNTHESIS_CLASSES:
                    continue
                _, synth_fn, default_dur, default_pitch = SYNTHESIS_CLASSES[class_name]
                seed_offset = int(time.time() * 1000) % 1000000
                for i in range(count):
                    seed = (seed_offset + i) * 314159265
                    random.seed(seed)
                    np.random.seed(seed % 2**32)
                    dur_var = default_dur * (1.0 + (random.random() - 0.5) * 0.3)
                    pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * 0.2)
                    samples = synth_fn(dur_var, pitch_var)
                    out_path = cat_dir / f"{cat_name}_{i+1:03d}.wav"
                    write_wav(out_path, samples)
                total_generated += count
            else:
                parsed = parse_prompt(prompt_template)
                paths = generate_from_prompt(parsed, count, cat_dir)
                total_generated += len(paths)
                total_errors += count - len(paths)

            results[cat_name] = {"generated": count, "errors": 0}
            print(f"  ✓ {cat_name:20s} {count} files → {cat_dir}")

        except Exception as e:
            results[cat_name] = {"generated": 0, "errors": count, "error": str(e)}
            print(f"  ✗ {cat_name:20s} FAILED: {e}")
            total_errors += count

    # ── Strict QA ──
    print(f"\n  {'='*55}")
    print(f"  STRICT QA: Basic integrity + Piano realism + Descriptor contrast")
    print(f"  {'='*55}")
    print(f"  Total generated: {total_generated}")
    print(f"  Total errors:    {total_errors}")
    print()

    qa_results = {}
    for cat_name, prompt_template, count in categories:
        cat_dir = out_dir / cat_name
        wavs = sorted(cat_dir.glob("*.wav"))
        if not wavs:
            continue

        basic_passes = 0
        for w in wavs:
            result = read_wav(w)
            if result is None:
                continue
            samples, sr = result
            feats = compute_features(samples, sr)
            if (feats["peak"] <= 0.99 and feats["rms"] > 0.001
                    and not np.any(np.isnan(samples))):
                basic_passes += 1

        basic_rate = basic_passes / max(len(wavs), 1) * 100

        # Piano realism check
        piano_passes = None
        if cat_name == "piano":
            from gen.piano import analyze_piano_full
            piano_scores = []
            piano_like = 0
            for w in wavs:
                result = read_wav(w)
                if result is None:
                    continue
                samples, sr = result
                pa = analyze_piano_full(samples, sr)
                piano_scores.append(pa["piano_likeness_score"])
                if pa["is_piano_like"]:
                    piano_like += 1
            avg_piano_score = np.mean(piano_scores) if piano_scores else 0
            piano_passes = piano_like
            piano_rate = piano_like / max(len(wavs), 1) * 100
            piano_status = "✓" if piano_rate >= 50 else "✗"
            print(f"  {piano_status} {cat_name:20s} piano_score={avg_piano_score:.3f}  "
                  f"piano_like={piano_like}/{len(wavs)} ({piano_rate:.0f}%)")

        # Category passes basic but with piano restriction
        if cat_name == "piano":
            passes = piano_passes if piano_passes is not None else basic_passes
            rate = passes / max(len(wavs), 1) * 100
            cat_pass = rate >= 50
        else:
            rate = basic_rate
            cat_pass = rate >= 80

        qa_results[cat_name] = {
            "files": len(wavs),
            "basic_pass": basic_passes,
            "pass": passes if cat_name == "piano" else basic_passes,
            "rate": f"{rate:.0f}%",
            "cat_pass": cat_pass,
        }
        if cat_name != "piano":
            icon = "✓" if rate >= 80 else ("△" if rate >= 50 else "✗")
            print(f"  {icon} {cat_name:20s} {basic_passes:>3}/{len(wavs):>3} pass ({rate:>5.0f}%)")

    # ── Descriptor contrast tests ──
    print(f"\n  {'─'*55}")
    print(f"  DESCRIPTOR CONTRAST TESTS")
    print(f"  {'─'*55}")

    contrast_dir = out_dir / "_contrast_tests"
    synth_pairs = [
        ("bright", "dark"),
        ("soft", "punchy"),
        ("clean", "distorted"),
        ("narrow", "wide"),
        ("lo_fi", "clean"),
    ]

    contrast_results = {}
    contrast_passes = 0
    contrast_total = 0

    for adj_a, adj_b in synth_pairs:
        result = run_contrast_test(adj_a, adj_b, "synth-gen", "pluck", 5, contrast_dir)
        checks = check_contrast(result)
        key = f"{adj_a}_vs_{adj_b}"
        contrast_results[key] = checks
        for c in checks:
            contrast_total += 1
            icon = "✓" if c["passed"] else "✗"
            if c["passed"]:
                contrast_passes += 1
            print(f"  {icon} {c['label']:<55s} "
                  f"a={c.get('val_a', '?'):>8}  b={c.get('val_b', '?'):>8}  "
                  f"diff={c.get('diff_pct', 0):.0f}%")

    contrast_rate = contrast_passes / max(contrast_total, 1) * 100
    contrast_status = "PASS" if contrast_rate >= 80 else ("WARN" if contrast_rate >= 50 else "FAIL")
    print(f"\n  Contrast tests: {contrast_passes}/{contrast_total} pass ({contrast_rate:.0f}%) — {contrast_status}")

    # ── Overall ──
    qa_pass_cats = sum(1 for r in qa_results.values() if r["cat_pass"])
    total_cats = len(qa_results)
    total_pass = sum(r["pass"] for r in qa_results.values())
    total_files = sum(r["files"] for r in qa_results.values())
    overall_rate = total_pass / max(total_files, 1) * 100

    print(f"\n  {'='*55}")
    print(f"  OVERALL RESULTS")
    print(f"  {'='*55}")
    print(f"  Category pass rate:   {qa_pass_cats}/{total_cats} categories pass")
    print(f"  File pass rate:       {total_pass}/{total_files} ({overall_rate:.0f}%)")
    print(f"  Contrast pass rate:   {contrast_passes}/{contrast_total} ({contrast_rate:.0f}%)")

    # Stricter status: require basic QA + piano realism + contrast
    basic_ok = total_files > 0 and overall_rate >= 80
    piano_ok = all(
        r["pass"] / max(r["files"], 1) >= 0.5
        for name, r in qa_results.items()
        if name == "piano"
    )
    contrast_ok = contrast_rate >= 70

    if basic_ok and piano_ok and contrast_ok:
        status = "MVP PASS (STRICT)"
    elif basic_ok and piano_ok:
        status = "MVP PASS (BASIC) — contrast tests need work"
    elif basic_ok:
        status = "MVP WARN — piano realism or contrast tests failing"
    else:
        status = "MVP FAIL — basic generation quality below threshold"

    print(f"  STATUS: {status}")

    # Save audit
    audit = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_generated": total_generated,
        "total_errors": total_errors,
        "overall_pass_rate": round(overall_rate, 1),
        "category_pass_rate": f"{qa_pass_cats}/{total_cats}",
        "contrast_pass_rate": round(contrast_rate, 1),
        "status": status,
        "categories": categories,
        "qa_results": qa_results,
        "contrast_results": contrast_results,
    }
    audit_path = out_dir / "mvp_audit.json"
    audit_path.write_text(json.dumps(audit, indent=2))
    print(f"\n  Full audit: {audit_path}")
