"""Make Similar mode: analyze a sample and generate variations near its feature space."""
import json
import random
import sys
import time
from pathlib import Path

import numpy as np

from gen import SAMPLE_RATE
from gen.io import read_wav, write_wav
from gen.features import compute_features
from gen.prompt import parse_prompt, _resolve_generator, _generate_variation, _seed_from_prompt, _write_metadata


def _infer_family_from_features(feats: dict) -> str:
    """Heuristic: map feature vector to most likely family."""
    centroid = feats.get("spectral_centroid", 1000)
    attack = feats.get("attack_ms", 10)
    hpr = feats.get("hpr", 0.5)
    duration = feats.get("duration_ms", 500)

    if centroid < 3000 and hpr < 0.3 and duration > 400:
        return "bass-gen"
    if centroid > 7000 and attack < 5 and hpr < 0.5:
        return "fx-gen"
    if 2000 < centroid < 6000 and 5 < attack < 20:
        return "piano-gen"
    if 3000 < centroid < 8000 and attack < 10:
        return "synth-gen"
    if 2000 < centroid < 5000 and 3 < attack < 15:
        return "guitar-gen"
    return "synth-gen"


def cmd_similar(args):
    """Generate variations similar to a reference sample."""
    ref_path = Path(args.reference)
    if not ref_path.exists():
        print(f"Error: {ref_path} not found", file=sys.stderr)
        sys.exit(1)

    count = args.count
    out_dir = Path(args.out) if args.out else Path("outputs/similar")
    out_dir.mkdir(parents=True, exist_ok=True)

    # Analyze reference
    result = read_wav(ref_path)
    if result is None:
        print(f"Error: could not read {ref_path}", file=sys.stderr)
        sys.exit(1)
    samples, sr = result
    feats = compute_features(samples, sr)
    family = _infer_family_from_features(feats)

    ref_centroid = feats.get("spectral_centroid", 2000)
    ref_attack = feats.get("attack_ms", 10)
    ref_rms = feats.get("rms", 0.1)
    ref_duration = feats.get("duration_ms", 500)

    print(f"Reference: {ref_path}")
    print(f"  → Detected family: {family}")
    print(f"  → Centroid: {ref_centroid:.0f} Hz")
    print(f"  → Attack: {ref_attack:.1f} ms")
    print(f"  → RMS: {ref_rms:.4f}")
    print(f"  → Duration: {ref_duration:.0f} ms")
    print(f"\nGenerating {count} similar variations...")

    # Build a prompt that targets the feature space
    prompt_parts = []
    if ref_centroid > 6000:
        prompt_parts.append("bright")
    elif ref_centroid < 2000:
        prompt_parts.append("dark")
    else:
        prompt_parts.append("warm")

    if ref_attack < 3:
        prompt_parts.append("punchy")
    elif ref_attack > 20:
        prompt_parts.append("soft")
    else:
        prompt_parts.append("mellow")

    family_to_noun = {
        "bass-gen": "bass",
        "piano-gen": "piano",
        "synth-gen": "synth",
        "guitar-gen": "guitar",
        "fx-gen": "impact",
    }
    noun = family_to_noun.get(family, "synth")
    prompt = " ".join(prompt_parts + [noun])
    parsed = parse_prompt(prompt)

    # Generate with feature-preserving overrides
    gen_fn, default_dur, default_pitch, gen_family, profile_name, overrides = _resolve_generator(parsed)
    dur = default_dur * (ref_duration / max(default_dur, 1))
    pitch = default_pitch

    for i in range(count):
        seed = _seed_from_prompt(f"{ref_path.stem}_similar_{i}", i)
        np.random.seed(seed % 2**32)

        samples, actual_dur, actual_pitch = _generate_variation(dur, pitch, gen_family, gen_fn)

        out_path = out_dir / f"similar_{ref_path.stem}_{i+1:03d}.wav"
        write_wav(out_path, samples)
        _write_metadata(out_path, parsed, seed, actual_dur, actual_pitch)

        if i == 0 or (i + 1) % 10 == 0:
            print(f"  [{i+1}/{count}] {out_path.name}")

    print(f"\nDone. {count} files in {out_dir}")


SPREAD_MAP = {
    "low": 0.03,
    "medium": 0.10,
    "high": 0.30,
}


def cmd_variations(args):
    """Generate a cloud of diverse variations around a reference sample."""
    ref_path = Path(args.reference)
    if not ref_path.exists():
        print(f"Error: {ref_path} not found", file=sys.stderr)
        sys.exit(1)

    count = args.count
    spread = SPREAD_MAP.get(args.spread, 0.10)
    out_dir = Path(args.out) if args.out else Path("outputs/variations")
    out_dir.mkdir(parents=True, exist_ok=True)

    result = read_wav(ref_path)
    if result is None:
        print(f"Error: could not read {ref_path}", file=sys.stderr)
        sys.exit(1)
    samples, sr = result
    feats = compute_features(samples, sr)
    family = _infer_family_from_features(feats)

    ref_centroid = feats.get("spectral_centroid", 2000)
    ref_attack = feats.get("attack_ms", 10)

    print(f"Reference: {ref_path}")
    print(f"  → Detected family: {family}")
    print(f"  → Centroid: {ref_centroid:.0f} Hz, Attack: {ref_attack:.1f} ms")
    print(f"  → Spread: {args.spread} ({spread * 100:.0f}% variation)")
    print(f"\nGenerating {count} variations...")

    # Generate a diverse cloud
    family_to_noun = {
        "bass-gen": "bass",
        "piano-gen": "piano",
        "synth-gen": "synth",
        "guitar-gen": "guitar",
        "fx-gen": "impact",
    }
    noun = family_to_noun.get(family, "synth")

    # Use varying adjectives to get diversity
    adjective_pool = ["bright", "dark", "warm", "punchy", "soft", "clean",
                      "distorted", "mellow", "crisp", "airy"]
    all_paths = []

    for i in range(count):
        adj = adjective_pool[i % len(adjective_pool)]
        prompt = f"{adj} {noun}"
        parsed = parse_prompt(prompt)

        gen_fn, default_dur, default_pitch, gen_family, profile_name, overrides = _resolve_generator(parsed)
        dur = default_dur * (1.0 + (random.random() - 0.5) * spread)
        pitch = default_pitch * (1.0 + (random.random() - 0.5) * spread)

        seed = _seed_from_prompt(f"{ref_path.stem}_var_{adj}_{i}", i)
        np.random.seed(seed % 2**32)

        samples, actual_dur, actual_pitch = _generate_variation(dur, pitch, gen_family, gen_fn)

        out_path = out_dir / f"var_{adj}_{i+1:03d}.wav"
        write_wav(out_path, samples)
        _write_metadata(out_path, parsed, seed, actual_dur, actual_pitch)
        all_paths.append(out_path)

    # Auto-rank top 5 by highest RMS (proxy for impact)
    ranked = []
    for p in all_paths:
        result = read_wav(p)
        if result is None:
            continue
        s, _ = result
        f = compute_features(s, sr)
        ranked.append((p, f.get("rms", 0)))
    ranked.sort(key=lambda x: x[1], reverse=True)
    top5 = ranked[:5]

    top_dir = out_dir / "top5"
    top_dir.mkdir(exist_ok=True)
    print(f"\nTop 5 (by RMS):")
    for i, (p, rms) in enumerate(top5, 1):
        dest = top_dir / p.name
        import shutil
        shutil.copy2(p, dest)
        print(f"  {i}. {p.name} (RMS={rms:.4f})")

    print(f"\nDone. {count} files in {out_dir}")
    print(f"Top 5 exported to {top_dir}")
