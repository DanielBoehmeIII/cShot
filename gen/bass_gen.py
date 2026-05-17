"""Bass generation: sub, Reese, distorted, pluck, FM bass with full controls."""

import math
import random
import sys
import time
from pathlib import Path

import numpy as np
from scipy import signal as sp_signal

from gen import SAMPLE_RATE
from gen.dsp import (
    noise_like, tape_saturation, soft_clip,
    biquad_low_shelf, biquad_high_shelf, biquad_peaking,
)
from gen.io import write_wav


# ─── Oscillator helpers ─────────────────────────────────

def _osc_saw(t: float, freq: float, detune: float = 0.0) -> float:
    return ((t * (freq * (1.0 + detune))) % 1.0) * 2.0 - 1.0


def _osc_square(t: float, freq: float, pw: float = 0.5, detune: float = 0.0) -> float:
    phase = (t * (freq * (1.0 + detune))) % 1.0
    return 1.0 if phase < pw else -1.0


def _osc_sine(t: float, freq: float, phase_offset: float = 0.0) -> float:
    return math.sin(2.0 * math.pi * freq * t + phase_offset)


# ─── Bass synthesis ─────────────────────────────────────

def synthesize_bass(duration_ms: float = 800.0, pitch_hz: float = 55.0,
                    profile_name: str = "808", overrides: dict = None) -> np.ndarray:
    """Generate a bass one-shot from the named profile."""
    pf = dict(BASS_PROFILES.get(profile_name, BASS_PROFILES["808"]))
    if overrides:
        pf.update(overrides)

    dur_s = duration_ms / 1000.0
    num_samples = max(100, int(SAMPLE_RATE * dur_s))
    samples = np.zeros(num_samples)

    profile_type = pf["type"]
    drive = pf.get("drive", 0.0)
    growl = pf.get("growl", 0.0)
    glide = pf.get("glide", 0.0)
    sub_body_balance = pf.get("sub_body_balance", 0.5)

    # Pitch envelope (glide)
    def _pitch_env(t: float) -> float:
        if glide <= 0:
            return pitch_hz
        frac = min(t / max(dur_s, 0.001), 1.0)
        return pitch_hz * (1.0 + glide * (1.0 - frac))

    # ADSR
    attack_s = pf["attack_ms"] / 1000.0
    decay_s = pf["decay_ms"] / 1000.0
    sustain_level = pf["sustain_level"]
    release_s = pf["release_ms"] / 1000.0

    def _adsr(t: float) -> float:
        if t < attack_s:
            return t / max(attack_s, 0.001)
        elif t < attack_s + decay_s:
            dt = (t - attack_s) / max(decay_s, 0.001)
            return 1.0 - (1.0 - sustain_level) * dt
        elif t < dur_s - release_s:
            return sustain_level
        else:
            rt = (t - (dur_s - release_s)) / max(release_s, 0.001)
            return sustain_level * (1.0 - rt)

    # Sub oscillator state
    sub_phase = 0.0
    filter_hist = 0.0
    growl_hist = 0.0

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        freq = _pitch_env(t)
        env = _adsr(t)

        if profile_type == "sub":
            val = _osc_sine(t, freq)
            val = val * env

        elif profile_type == "reese":
            n_detune = pf.get("detune_layers", 7)
            val = 0.0
            for layer in range(n_detune):
                d = (layer - (n_detune - 1) / 2.0) * pf.get("detune_amount", 3.0) / 100.0
                val += _osc_saw(t, freq, d)
            val = val / max(n_detune, 1) * env

        elif profile_type == "distorted":
            val = _osc_saw(t, freq, 0)
            val += _osc_square(t, freq, 0.3) * 0.5
            val = val * env * 1.5
            # Distortion
            val = tape_saturation(val, 1.0 + drive * 4.0)

        elif profile_type == "pluck":
            val = _osc_saw(t, freq, 0.02)
            val += _osc_sine(t, freq * 0.5) * 0.3
            val = val * env
            # Fast decay shaping
            val *= math.exp(-8.0 * t)

        elif profile_type == "fm":
            mod_freq = freq * pf.get("fm_ratio", 2.0)
            mod_depth = pf.get("fm_depth", 0.5) * (1.0 + drive * 2.0)
            modulator = mod_depth * math.sin(2.0 * math.pi * mod_freq * t)
            val = math.sin(2.0 * math.pi * freq * t + modulator)
            val = val * env

        else:
            val = _osc_sine(t, freq) * env

        # Sub/body balance: mix sub (sine at pitch) with body (oscillator)
        sub = _osc_sine(t, freq) * 0.8 * env
        samples[i] = val * (1.0 - sub_body_balance) + sub * sub_body_balance

        # Growl: resonant low-pass with feedback
        if growl > 0:
            g_freq = 80.0 + growl * 200.0
            g_alpha = min(1.0, 2.0 * math.pi * g_freq / SAMPLE_RATE)
            growl_hist = growl_hist + g_alpha * (samples[i] - growl_hist)
            samples[i] = samples[i] * (1.0 - growl * 0.5) + growl_hist * (growl * 0.7)

    # Post-processing
    if pf.get("low_shelf_db", 0) != 0:
        samples = biquad_low_shelf(samples, 100.0, pf["low_shelf_db"], 0.7)
    if pf.get("high_shelf_db", 0) != 0:
        samples = biquad_high_shelf(samples, 2000.0, pf["high_shelf_db"], 0.7)

    # Clipping safety
    peak = np.max(np.abs(samples))
    if peak > 0:
        if peak > 0.99:
            for i in range(len(samples)):
                samples[i] = soft_clip(samples[i], 0.9)
        samples = samples / max(np.max(np.abs(samples)), 1e-10) * 0.9

    return samples.astype(np.float32)


# ─── Bass profiles ──────────────────────────────────────

BASS_PROFILES = {
    "808": {
        "label": "808 Sub",
        "default_duration_ms": 800.0,
        "default_pitch_hz": 55.0,
        "type": "sub",
        "attack_ms": 5.0,
        "decay_ms": 300.0,
        "sustain_level": 0.3,
        "release_ms": 200.0,
        "drive": 0.0,
        "growl": 0.0,
        "glide": -0.3,
        "sub_body_balance": 0.9,
        "detune_layers": 1,
        "detune_amount": 0.0,
        "fm_ratio": 2.0,
        "fm_depth": 0.0,
        "low_shelf_db": 4.0,
        "high_shelf_db": -6.0,
    },
    "reese": {
        "label": "Reese Bass",
        "default_duration_ms": 600.0,
        "default_pitch_hz": 80.0,
        "type": "reese",
        "attack_ms": 10.0,
        "decay_ms": 200.0,
        "sustain_level": 0.4,
        "release_ms": 150.0,
        "drive": 0.2,
        "growl": 0.3,
        "glide": 0.0,
        "sub_body_balance": 0.4,
        "detune_layers": 7,
        "detune_amount": 5.0,
        "fm_ratio": 2.0,
        "fm_depth": 0.0,
        "low_shelf_db": 2.0,
        "high_shelf_db": 2.0,
    },
    "distorted": {
        "label": "Distorted Bass",
        "default_duration_ms": 500.0,
        "default_pitch_hz": 70.0,
        "type": "distorted",
        "attack_ms": 3.0,
        "decay_ms": 150.0,
        "sustain_level": 0.2,
        "release_ms": 100.0,
        "drive": 0.8,
        "growl": 0.5,
        "glide": -0.1,
        "sub_body_balance": 0.3,
        "detune_layers": 1,
        "detune_amount": 0.0,
        "fm_ratio": 2.0,
        "fm_depth": 0.0,
        "low_shelf_db": 0.0,
        "high_shelf_db": 0.0,
    },
    "pluck": {
        "label": "Pluck Bass",
        "default_duration_ms": 350.0,
        "default_pitch_hz": 100.0,
        "type": "pluck",
        "attack_ms": 2.0,
        "decay_ms": 80.0,
        "sustain_level": 0.0,
        "release_ms": 50.0,
        "drive": 0.1,
        "growl": 0.0,
        "glide": 0.0,
        "sub_body_balance": 0.5,
        "detune_layers": 1,
        "detune_amount": 0.0,
        "fm_ratio": 2.0,
        "fm_depth": 0.0,
        "low_shelf_db": 3.0,
        "high_shelf_db": 0.0,
    },
    "fm": {
        "label": "FM Bass",
        "default_duration_ms": 600.0,
        "default_pitch_hz": 80.0,
        "type": "fm",
        "attack_ms": 5.0,
        "decay_ms": 200.0,
        "sustain_level": 0.3,
        "release_ms": 150.0,
        "drive": 0.3,
        "growl": 0.2,
        "glide": -0.2,
        "sub_body_balance": 0.5,
        "detune_layers": 1,
        "detune_amount": 0.0,
        "fm_ratio": 3.0,
        "fm_depth": 0.7,
        "low_shelf_db": 2.0,
        "high_shelf_db": -1.0,
    },
}


# ─── Hybrid bass from cluster ───────────────────────────

def synthesize_hybrid_bass(duration_ms: float, pitch_hz: float,
                           cluster_profile: dict = None) -> np.ndarray:
    """Generate hybrid bass using reference cluster profile to blend profiles."""
    if not cluster_profile:
        return synthesize_bass(duration_ms, pitch_hz, "808")

    # Extract cluster characteristics
    centroid = cluster_profile.get("spectral_centroid", {}).get("mean", 500)
    low_band = cluster_profile.get("low_band_energy", {}).get("mean", 0.8)
    high_band = cluster_profile.get("high_band_energy", {}).get("mean", 0.05)
    hpr_val = cluster_profile.get("hpr", {}).get("mean", 0.5)
    decay = cluster_profile.get("decay_length_ms", {}).get("mean", 200)

    # Select profile based on cluster centroid
    if centroid > 1500 and high_band > 0.1:
        base = "distorted"
    elif centroid > 800:
        base = "reese"
    elif low_band > 0.7 and hpr_val > 0.5:
        base = "808"
    elif hpr_val < 0.3:
        base = "pluck"
    else:
        base = "fm"

    overrides = {}
    if decay > 200:
        overrides["decay_ms"] = min(decay, 800)
        overrides["sustain_level"] = 0.3
    elif decay < 50:
        overrides["decay_ms"] = 80
        overrides["sustain_level"] = 0.0

    low_target = 0.3 if low_band < 0.3 else 0.7
    overrides["sub_body_balance"] = low_target
    overrides["drive"] = min(max(centroid / 3000, 0.0), 1.0)

    return synthesize_bass(duration_ms, pitch_hz, base, overrides)


# ─── CLI commands ───────────────────────────────────────

def cmd_bass_gen(args):
    """Generate bass samples from named profiles with optional control overrides."""
    profile_name = args.profile
    if profile_name not in BASS_PROFILES and profile_name not in ("all", "hybrid"):
        print(f"Unknown profile: {profile_name}")
        print(f"Valid: {', '.join(sorted(BASS_PROFILES.keys()))}, all, hybrid")
        sys.exit(1)

    count = args.count
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    overrides = {}
    if args.drive is not None:
        overrides["drive"] = args.drive
    if args.growl is not None:
        overrides["growl"] = args.growl
    if args.glide is not None:
        overrides["glide"] = args.glide
    if args.sub_balance is not None:
        overrides["sub_body_balance"] = args.sub_balance

    seed_offset = int(time.time() * 1000) % 1000000

    if profile_name == "hybrid":
        print(f"Generating {count} Hybrid Bass (cluster-informed) → {out_dir}...")
        from gen.scanning import load_profiles
        import json
        clusters_path = Path(args.clusters) if args.clusters else Path("reference_clusters.json")
        cluster_data = None
        if clusters_path.exists():
            with open(clusters_path) as f:
                cluster_data = json.load(f)

        for i in range(count):
            seed = (seed_offset + i) * 314159265
            random.seed(seed)
            np.random.seed(seed % 2**32)
            pitch_var = 55.0 * (1.0 + (random.random() - 0.5) * 0.3)
            dur_var = 600.0 * (1.0 + (random.random() - 0.5) * 0.2)

            cluster_profile = None
            if cluster_data:
                assignments = cluster_data.get("assignments", [])
                all_clusters = sorted(set(a["cluster"] for a in assignments))
                if all_clusters:
                    from gen.cluster_gen import compute_cluster_profile
                    with open("reference_analysis.json") as f:
                        analysis = json.load(f)
                    cid = random.choice(all_clusters)
                    cluster_profile = compute_cluster_profile(assignments, analysis, cid)

            samples = synthesize_hybrid_bass(dur_var, pitch_var, cluster_profile)
            out_path = out_dir / f"bass_hybrid_{i+1:03d}.wav"
            write_wav(out_path, samples)
            if i == 0 or (i + 1) % 5 == 0:
                print(f"  [{i+1}/{count}] {out_path.name}")
        print(f"  Done: {count} Hybrid Bass → {out_dir}")
        return

    profiles_to_gen = list(BASS_PROFILES.keys()) if profile_name == "all" else [profile_name]
    for pn in profiles_to_gen:
        pf = BASS_PROFILES[pn]
        out_subdir = out_dir / pn if profile_name == "all" else out_dir
        out_subdir.mkdir(parents=True, exist_ok=True)
        print(f"Generating {count} {pf['label']} → {out_subdir}...")

        for i in range(count):
            seed = (seed_offset + i) * 314159265 + hash(pn) % 1000000
            random.seed(seed)
            np.random.seed(seed % 2**32)
            dur_var = pf["default_duration_ms"] * (1.0 + (random.random() - 0.5) * 0.2)
            pitch_var = pf["default_pitch_hz"] * (1.0 + (random.random() - 0.5) * 0.15)
            samples = synthesize_bass(dur_var, pitch_var, pn, overrides if overrides else None)

            out_path = out_subdir / f"bass_{pn}_{i+1:03d}.wav"
            write_wav(out_path, samples)
            if i == 0 or (i + 1) % 5 == 0:
                print(f"  [{i+1}/{count}] {out_path.name}")
        print(f"  Done: {count} {pf['label']} → {out_subdir}")


def cmd_bass_qa(args):
    """Bass QA: low-end strength, harmonic richness, pitch stability, clipping safety."""
    from gen.io import read_wav
    from gen.features import compute_features, compute_hpr, detect_pitch_full

    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = sorted(in_dir.glob("*.wav"))
    if not wav_files:
        print(f"Error: no .wav files in {in_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Bass QA: {len(wav_files)} files\n")

    print(f"  {'File':<40} {'Low-End':>8} {'Richness':>9} {'Pitch':>8} {'Clip':>6} {'→':>4}")
    print(f"  {'─'*40} {'─'*8} {'─'*9} {'─'*8} {'─'*6} {'─'*4}")

    results = []
    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            continue
        samples, sr = result
        feats = compute_features(samples, sr)
        pitch_info = detect_pitch_full(samples, sr)

        low_end = feats.get("low_band_energy", 0)
        centroid = feats.get("spectral_centroid", 0)
        peak = feats.get("peak", 0)
        is_clipped = peak > 0.99
        hpr = compute_hpr(samples, sr)
        has_good_low_end = low_end > 0.3
        has_harmonic_richness = 100 < centroid < 3000
        has_pitch = pitch_info["confidence"] > 0.3
        has_fundamental = pitch_info["pitch_hz"] < 200 if pitch_info["pitch_hz"] > 0 else False

        if has_good_low_end and has_harmonic_richness and not is_clipped:
            verdict = "✓"
        elif not has_good_low_end and centroid > 3000:
            verdict = "thin"
        elif is_clipped:
            verdict = "clip"
        else:
            verdict = "△"

        results.append({
            "file": wav_path.name,
            "low_band_energy": round(low_end, 4),
            "centroid_hz": round(centroid, 1),
            "pitch_hz": round(pitch_info["pitch_hz"], 1),
            "pitch_conf": round(pitch_info["confidence"], 3),
            "peak": round(peak, 4),
            "is_clipped": is_clipped,
            "has_good_low_end": has_good_low_end,
            "has_harmonic_richness": has_harmonic_richness,
            "has_pitch": has_pitch,
            "hpr": round(hpr, 4),
            "verdict": verdict,
        })

        print(f"  {verdict:>4s} {wav_path.name:<36s} {low_end:>7.2f}  {centroid:>6.0f}Hz "
              f"{pitch_info['pitch_hz']:>6.1f}Hz {peak:>.3f} {hpr:.2f}")

    n = len(results)
    good = sum(1 for r in results if r["verdict"] == "✓")
    thin = sum(1 for r in results if r["verdict"] == "thin")
    clip = sum(1 for r in results if r["is_clipped"])
    low_ok = sum(1 for r in results if r["has_good_low_end"])
    rich = sum(1 for r in results if r["has_harmonic_richness"])
    pitch_ok = sum(1 for r in results if r["has_pitch"])

    print(f"\n  {'='*55}")
    print(f"  BASS QA SUMMARY")
    print(f"  {'='*55}")
    print(f"  Files tested:     {n}")
    print(f"  Good:             {good}/{n}")
    print(f"  Thin/no low-end:  {thin}")
    print(f"  Clipping:         {clip}")
    print(f"  Has low-end:      {low_ok}/{n}")
    print(f"  Has rich body:    {rich}/{n}")
    print(f"  Has pitch:        {pitch_ok}/{n}")

    if args.output:
        import json, time
        out = {
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "files_tested": n,
            "good": good,
            "results": results,
        }
        Path(args.output).write_text(json.dumps(out, indent=2))
        print(f"\n  Results: {args.output}")
