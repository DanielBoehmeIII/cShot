"""Tonal QA: verify generated piano/synth preserve pitch, decay, and harmonic structure."""

import json
import math
import sys
import time
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.features import (
    compute_features, compute_spectral_centroid, compute_hpr,
    compute_decay_length, compute_attack_time, detect_pitch_full,
    hz_to_note,
)
from gen.io import read_wav
from gen.scanning import load_profiles


def check_pitch(generated_pitch: float, ref_pitch_mean: float, ref_pitch_std: float) -> dict:
    """Check if generated pitch is within acceptable range of reference."""
    if ref_pitch_std <= 0:
        ref_pitch_std = ref_pitch_mean * 0.1
    z = (generated_pitch - ref_pitch_mean) / max(ref_pitch_std, 1)
    if abs(z) <= 1.0:
        status = "pass", "✓"
    elif abs(z) <= 2.0:
        status = "warn", "△"
    else:
        status = "fail", "✗"
    return {"z_score": round(z, 2), "status": status[0], "icon": status[1]}


def check_decay(generated_decay: float, ref_decay_mean: float, ref_decay_std: float) -> dict:
    """Check if generated decay is within acceptable range of reference."""
    if ref_decay_std <= 0:
        ref_decay_std = ref_decay_mean * 0.2 if ref_decay_mean > 0 else 1.0
    z = (generated_decay - ref_decay_mean) / max(ref_decay_std, 1)
    if abs(z) <= 1.0:
        status = "pass", "✓"
    elif abs(z) <= 2.0:
        status = "warn", "△"
    else:
        status = "fail", "✗"
    return {"z_score": round(z, 2), "status": status[0], "icon": status[1]}


def check_centroid(generated_centroid: float, ref_centroid_mean: float, ref_centroid_std: float) -> dict:
    """Check if generated spectral centroid matches reference."""
    if ref_centroid_std <= 0:
        ref_centroid_std = ref_centroid_mean * 0.15 if ref_centroid_mean > 0 else 100.0
    z = (generated_centroid - ref_centroid_mean) / max(ref_centroid_std, 1)
    if abs(z) <= 1.0:
        status = "pass", "✓"
    elif abs(z) <= 2.0:
        status = "warn", "△"
    else:
        status = "fail", "✗"
    return {"z_score": round(z, 2), "status": status[0], "icon": status[1]}


def check_hpr(generated_hpr: float, ref_hpr_mean: float, ref_hpr_std: float) -> dict:
    """Check if generated HPR matches reference."""
    if ref_hpr_std <= 0:
        ref_hpr_std = 0.1
    z = (generated_hpr - ref_hpr_mean) / max(ref_hpr_std, 0.01)
    if abs(z) <= 1.0:
        status = "pass", "✓"
    elif abs(z) <= 2.0:
        status = "warn", "△"
    else:
        status = "fail", "✗"
    return {"z_score": round(z, 2), "status": status[0], "icon": status[1]}


def cmd_tonal_qa(args):
    """QA: verify generated tonal samples preserve pitch, decay, and harmonic structure."""
    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    target_class = args.target
    if not target_class:
        print("Error: --target is required (e.g., synth_stab, piano_stab, clap)", file=sys.stderr)
        sys.exit(1)

    profiles = load_profiles()
    if target_class not in profiles:
        print(f"Error: class '{target_class}' not found in profiles", file=sys.stderr)
        sys.exit(1)

    profile = profiles[target_class]

    wav_files = sorted(in_dir.glob("*.wav"))
    if not wav_files:
        print(f"Error: no .wav files in {in_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Tonal QA: {len(wav_files)} generated files vs '{target_class}' reference")
    print()

    headers = ["File", "Note", "Pitch", "Decay", "Centroid", "HPR"]
    print(f"  {'File':<40} {'Note':>8} {'Pitch':>10} {'Decay':>10} {'Centroid':>10} {'HPR':>8}")
    print(f"  {'─'*40} {'─'*8} {'─'*10} {'─'*10} {'─'*10} {'─'*8}")

    ref_pitch = profile.get("spectral_centroid", {}).get("mean", 1000)
    ref_pitch_std = profile.get("spectral_centroid", {}).get("std", 500)
    ref_decay = profile.get("decay_length_ms", {}).get("mean", 100)
    ref_decay_std = profile.get("decay_length_ms", {}).get("std", 50)
    ref_centroid = profile.get("spectral_centroid", {}).get("mean", 2000)
    ref_centroid_std = profile.get("spectral_centroid", {}).get("std", 500)
    has_hpr = "hpr" in profile
    ref_hpr = profile.get("hpr", {}).get("mean", 0.5) if has_hpr else None
    ref_hpr_std = profile.get("hpr", {}).get("std", 0.2) if has_hpr else None

    # For pitch-aware targets, use hz_to_note for note display
    results = []
    pitch_results = []
    decay_results = []
    centroid_results = []
    hpr_results = []

    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            continue
        samples, sr = result

        pitch_info = detect_pitch_full(samples, sr)
        gen_pitch = pitch_info["pitch_hz"]
        gen_note = pitch_info["note_name"]

        feats = compute_features(samples, sr)
        gen_decay = feats.get("decay_length_ms", 0)
        gen_centroid = feats.get("spectral_centroid", 0)
        gen_hpr = compute_hpr(samples, sr)

        p_check = check_pitch(gen_pitch, ref_pitch, ref_pitch_std)
        d_check = check_decay(gen_decay, ref_decay, ref_decay_std)
        c_check = check_centroid(gen_centroid, ref_centroid, ref_centroid_std)
        if has_hpr and ref_hpr is not None:
            h_check = check_hpr(gen_hpr, ref_hpr, ref_hpr_std)
        else:
            h_check = {"z_score": 0, "status": "skip", "icon": "−"}

        pitch_results.append(p_check["status"])
        decay_results.append(d_check["status"])
        centroid_results.append(c_check["status"])
        hpr_results.append(h_check["status"])

        results.append({
            "file": wav_path.name,
            "note": gen_note,
            "pitch_hz": round(gen_pitch, 1),
            "decay_ms": round(gen_decay, 2),
            "centroid_hz": round(gen_centroid, 1),
            "hpr": round(gen_hpr, 4),
            "pitch_check": p_check,
            "decay_check": d_check,
            "centroid_check": c_check,
            "hpr_check": h_check,
            "all_pass": all(c["status"] in ("pass", "skip") for c in [p_check, d_check, c_check, h_check]),
        })

        print(f"  {p_check['icon']} {wav_path.name:<38s} {gen_note:>8s} "
              f"{p_check['status'][0].upper():>1s}{gen_pitch:>6.1f}Hz "
              f"{d_check['icon']}{gen_decay:>6.2f}ms "
              f"{c_check['icon']}{gen_centroid:>7.1f}Hz "
              f"{h_check['icon']}{gen_hpr:.3f}")

    # Summary
    n = len(results)
    if n == 0:
        print("\n  No files could be analyzed.")
        return

    p_pass = pitch_results.count("pass")
    p_warn = pitch_results.count("warn")
    p_fail = pitch_results.count("fail")
    d_pass = decay_results.count("pass")
    c_pass = centroid_results.count("pass")
    h_pass = hpr_results.count("pass")
    h_skip = hpr_results.count("skip")
    all_pass = sum(1 for r in results if r["all_pass"])

    print(f"\n  {'='*55}")
    print(f"  TONAL QA SUMMARY")
    print(f"  {'='*55}")
    print(f"  Target class:      {target_class}")
    print(f"  Files tested:      {n}")
    print(f"  All checks pass:   {all_pass}/{n}")
    print(f"  ───────────────────────────────────────────────")
    print(f"  Pitch (centroid):   ✓{p_pass} △{p_warn} ✗{p_fail}")
    print(f"  Decay length:       ✓{d_pass} △{n - d_pass - (n - d_pass)} ✗{n - d_pass}")
    print(f"  Spectral centroid:  ✓{c_pass} △{n - c_pass - (n - c_pass)} ✗{n - c_pass}")
    h_skip_str = f" −{h_skip}" if h_skip else ""
    print(f"  HPR (tonalness):    ✓{h_pass} △0 ✗{n - h_pass - h_skip}{h_skip_str}")

    overall_pass_rate = all_pass / max(n, 1) * 100
    status = "PASS" if overall_pass_rate >= 80 else ("WARN" if overall_pass_rate >= 50 else "FAIL")
    print(f"  Overall:            {overall_pass_rate:.0f}% → {status}")

    # Save results
    output = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "target_class": target_class,
        "files_tested": n,
        "all_pass_count": all_pass,
        "pass_rate": round(overall_pass_rate, 1),
        "status": status,
        "reference_profile": {
            "pitch_mean": ref_pitch,
            "pitch_std": ref_pitch_std,
            "decay_mean": ref_decay,
            "decay_std": ref_decay_std,
            "centroid_mean": ref_centroid,
            "centroid_std": ref_centroid_std,
            "hpr_mean": ref_hpr,
            "hpr_std": ref_hpr_std,
            "has_hpr": has_hpr,
        },
        "results": results,
    }

    out_path = Path(args.output) if args.output else in_dir / "tonal_qa_results.json"
    out_path.write_text(json.dumps(output, indent=2))
    print(f"\n  Results: {out_path}")
