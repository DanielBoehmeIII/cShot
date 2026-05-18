"""Quality Gate v1: producer-usability checks for generated sounds.

Checks:
  - weak transients (inaudible/unusable attack)
  - ugly clipping (hard clipping or distortion)
  - too-long tails (excessive decay for the sound class)
  - near-duplicates (two files that sound nearly identical)
  - muddy low end (excessive low-frequency energy without definition)

Usage:
  cshot gate <dir>         Run all gates on a directory of WAV files
  cshot gate <dir> --fix   Auto-remove files that fail gates
"""
import json
import shutil
import sys
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

from gen.io import read_wav, write_wav
from gen.features import compute_features
from gen.polish import polish_file


def check_weak_transient(feats: dict) -> tuple[bool, str]:
    """Check if a sound has a usable attack transient."""
    attack = feats.get("attack_ms", 0)
    trans_count = feats.get("transient_count", 0)
    early_rms = feats.get("early_rms", 0)
    rms = feats.get("rms", 0)

    if attack > 100:
        return False, f"attack too slow ({attack:.0f}ms)"
    if trans_count == 0 and early_rms < 0.1:
        return False, f"no transient detected (early_rms={early_rms:.3f})"
    if rms < 0.01:
        return False, f"too quiet (rms={rms:.4f})"
    return True, ""


def check_clipping(feats: dict) -> tuple[bool, str]:
    """Check for ugly hard clipping."""
    peak = feats.get("peak", 0)
    # Peak of exactly 1.0 usually means hard clipping
    if peak >= 0.999:
        return False, f"hard clipped (peak={peak:.4f})"

    zcr = feats.get("zero_crossing_rate", 0)
    if zcr > 0.4:
        cent = feats.get("spectral_centroid", 0)
        if cent > 15000:
            return False, f"harsh高频 content (zcr={zcr:.3f}, cent={cent:.0f}Hz)"

    return True, ""


def check_tail_length(feats: dict, sound_class: str = "") -> tuple[bool, str]:
    """Check if tail is too long for the sound class."""
    decay = feats.get("decay_length_ms", 0)
    duration = feats.get("duration_ms", 0)

    class_max_decay = {
        "kick": 500, "kicks": 500,
        "snare": 400, "snares": 400,
        "clap": 400, "claps": 400,
        "hat": 200, "hats": 200, "closed_hat": 200, "open_hat": 600,
        "perc": 600, "percs": 600,
        "808": 1500, "basses_808": 1500, "bass": 1500, "basses_sub": 1500,
        "synth": 2000, "synths": 2000,
        "keys": 3000, "piano": 3000,
        "guitar": 1500, "guitars": 1500,
        "impact": 2000, "impacts": 2000,
        "texture": 5000, "textures": 5000,
        "atmosphere": 8000, "atmospheres": 8000,
    }

    for key, max_decay in class_max_decay.items():
        if key in sound_class.lower():
            if decay > max_decay * 1.5:
                return False, f"tail too long ({decay:.0f}ms, max={max_decay}ms for {key})"
            return True, ""

    if duration > 10000:
        return False, f"too long ({duration:.0f}ms)"
    return True, ""


def check_near_duplicates(feats_list: list[dict], threshold: float = 0.95) -> list[tuple[int, int, float]]:
    """Detect near-duplicate pairs among a list of feature dicts."""
    pairs = []
    for i in range(len(feats_list)):
        for j in range(i + 1, len(feats_list)):
            fi = feats_list[i]
            fj = feats_list[j]

            vec_i = np.array([
                fi.get("spectral_centroid", 0) / 10000,
                fi.get("hpr", 0.5),
                fi.get("rms", 0) * 10,
                fi.get("transient_count", 0) / 20,
                fi.get("duration_ms", 0) / 5000,
                fi.get("low_band_energy", 0),
                fi.get("high_band_energy", 0),
            ])
            vec_j = np.array([
                fj.get("spectral_centroid", 0) / 10000,
                fj.get("hpr", 0.5),
                fj.get("rms", 0) * 10,
                fj.get("transient_count", 0) / 20,
                fj.get("duration_ms", 0) / 5000,
                fj.get("low_band_energy", 0),
                fj.get("high_band_energy", 0),
            ])

            dist = float(np.sqrt(np.sum((vec_i - vec_j) ** 2)))
            sim = max(0, 1.0 - dist / 2.0)

            if sim >= threshold:
                pairs.append((i, j, sim))

    return pairs


def check_muddy_low_end(feats: dict, sound_class: str = "") -> tuple[bool, str]:
    """Check for muddy/unclear low end.
    Less strict for bass/808 categories where low end is expected.
    """
    low_band = feats.get("low_band_energy", 0)
    mid_band = feats.get("mid_band_energy", 0)
    high_band = feats.get("high_band_energy", 0)
    hpr = feats.get("hpr", 0.5)
    centroid = feats.get("spectral_centroid", 0)

    is_bass = any(k in sound_class.lower() for k in ["bass", "808", "sub"])

    if not is_bass:
        if low_band > 0.95 and hpr > 0.95 and centroid < 200:
            return False, f"muddy low end (low={low_band:.3f}, hpr={hpr:.3f}, cent={centroid:.0f}Hz)"

        if low_band > 0.99 and mid_band < 0.005:
            return False, f"almost no mid/high content (low={low_band:.3f}, mid={mid_band:.4f})"

    peak = feats.get("peak", 0)
    rms = feats.get("rms", 0)
    if rms > 0.3 and peak >= 0.99 and low_band > 0.8:
        return False, f"loud muddy (rms={rms:.3f}, low={low_band:.3f})"

    return True, ""


def run_all_gates(file_paths: list[Path], dup_threshold: float = 0.98) -> dict:
    """Run all quality gates on a list of files.
    dedup_threshold: similarity threshold for near-duplicate detection (0.0-1.0).
    Returns per-file results and aggregate stats.
    """
    results = {}
    all_feats = []

    skip_dirs = {"_top", "_favorites", "_export"}
    filtered = [w for w in file_paths if w.parent.name not in skip_dirs]

    for w in filtered:
        result = read_wav(w)
        if result is None:
            results[str(w)] = {"pass": False, "reasons": ["could not read file"]}
            continue

        samples, sr = result
        if samples.ndim == 2:
            samples = samples.mean(axis=1)
        feats = compute_features(samples, sr)
        feats["_file"] = str(w)
        all_feats.append(feats)

        sound_class = w.parent.name if w.parent != w.parent.parent else ""

        checks = []
        reasons = []

        ok, reason = check_weak_transient(feats)
        checks.append(("weak_transient", ok))
        if not ok:
            reasons.append(reason)

        ok, reason = check_clipping(feats)
        checks.append(("clipping", ok))
        if not ok:
            reasons.append(reason)

        ok, reason = check_tail_length(feats, sound_class)
        checks.append(("tail_length", ok))
        if not ok:
            reasons.append(reason)

        ok, reason = check_muddy_low_end(feats, sound_class)
        checks.append(("muddy_low_end", ok))
        if not ok:
            reasons.append(reason)

        overall_pass = all(ok for _, ok in checks)
        results[str(w)] = {
            "pass": overall_pass,
            "reasons": reasons,
            "checks": {name: ok for name, ok in checks},
            "file": w.name,
        }

    duplicate_pairs = check_near_duplicates(all_feats, threshold=dup_threshold)
    dup_set = set()
    for i, j, sim in duplicate_pairs:
        f_i = all_feats[i]["_file"]
        f_j = all_feats[j]["_file"]
        results[f_i].setdefault("duplicates", []).append({"other": Path(f_j).name, "similarity": round(sim, 3)})
        results[f_j].setdefault("duplicates", []).append({"other": Path(f_i).name, "similarity": round(sim, 3)})
        dup_set.add(f_i)
        dup_set.add(f_j)

    for fpath_str in results:
        if fpath_str in dup_set:
            results[fpath_str]["pass"] = False
            results[fpath_str]["reasons"].append("near-duplicate detected")

    stats = {
        "total": len(filtered),
        "passed": sum(1 for r in results.values() if r["pass"]),
        "failed": sum(1 for r in results.values() if not r["pass"]),
        "pass_rate": 0.0,
    }
    if stats["total"] > 0:
        stats["pass_rate"] = round(stats["passed"] / stats["total"], 3)

    return {"results": results, "stats": stats, "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}


def cmd_gate(args):
    """Run quality gates on a directory of WAV files."""
    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found")
        sys.exit(1)

    wav_files = sorted(in_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No WAV files found in {in_dir}")
        return

    print(f"Quality Gate — {in_dir}")
    print(f"{'='*60}")
    print(f"Files: {len(wav_files)}")
    print(f"Gates: weak_transient, clipping, tail_length, muddy_low_end, near_duplicates")
    print()

    report = run_all_gates(wav_files, dup_threshold=0.98)

    for fpath_str, result in report["results"].items():
        rel = Path(fpath_str).relative_to(in_dir) if Path(fpath_str).is_absolute() else fpath_str
        status = "PASS" if result["pass"] else "FAIL"
        reasons = "; ".join(result["reasons"])
        print(f"  [{status}] {rel}" + (f"  — {reasons}" if reasons else ""))

    s = report["stats"]
    print(f"\n{'='*60}")
    print(f"Results: {s['passed']}/{s['total']} passed ({s['pass_rate']*100:.0f}%)")
    print(f"Failed: {s['failed']}")
    print()

    if args.fix and s["failed"] > 0:
        print("Removing failed files...")
        removed = 0
        for fpath_str, result in report["results"].items():
            if not result["pass"]:
                p = Path(fpath_str)
                if p.exists():
                    p.unlink()
                    removed += 1
        print(f"Removed {removed} files.")

    gate_path = in_dir / "quality_gate.json"
    with open(gate_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"Report: {gate_path}")
