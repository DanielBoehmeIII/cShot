"""Auto-ranking: score and rank generated outputs by quality, variation, and preference."""
import json
import sys
from pathlib import Path
from collections import Counter

import numpy as np

from gen import SAMPLE_RATE
from gen.io import read_wav
from gen.features import compute_features
from gen.rating import _load_ratings
from gen.polish import validate_audio


def score_file(wav_path: Path, ratings: list[dict] = None) -> dict:
    """Score a single WAV file: quality + rating history + feature desirability."""
    result = read_wav(wav_path)
    if result is None:
        return {"file": wav_path.name, "score": 0.0, "pass": False, "issues": ["unreadable"]}

    samples, sr = result
    feats = compute_features(samples, sr)
    validation = validate_audio(samples)

    score = 50.0  # Base score

    # Quality
    if validation["pass"]:
        score += 20.0
    if 0.05 < feats.get("rms", 0) < 0.5:
        score += 10.0
    if 2 < feats.get("attack_ms", 10) < 80:
        score += 10.0
    if feats.get("noise_floor", 0.1) < 0.01:
        score += 5.0
    if feats.get("peak", 1) > 0.99:
        score -= 20.0

    # Rating history
    if ratings:
        resolved = str(wav_path.resolve())
        file_ratings = [r for r in ratings if r.get("file") in resolved or r.get("file") == wav_path.name]
        for r in file_ratings:
            if r["rating"] == "favorite":
                score += 30.0
            elif r["rating"] == "good":
                score += 15.0
            elif r["rating"] == "bad":
                score -= 20.0
            elif r["rating"] == "trash":
                score -= 40.0

    # Feature desirability
    centroid = feats.get("spectral_centroid", 1000)
    if 500 < centroid < 8000:
        score += 5.0

    return {
        "file": wav_path.name,
        "path": str(wav_path),
        "score": round(max(0, min(100, score)), 1),
        "pass": validation["pass"],
        "issues": validation["issues"],
        "features": {
            "rms": round(float(feats.get("rms", 0)), 4),
            "peak": round(float(feats.get("peak", 0)), 4),
            "spectral_centroid": round(float(feats.get("spectral_centroid", 0)), 1),
            "attack_ms": round(float(feats.get("attack_ms", 0)), 2),
            "duration_s": round(len(samples) / sr, 3),
        },
    }


def cmd_rank(args):
    """Rank all WAV files in a directory by quality score."""
    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = sorted(in_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No .wav files found in {in_dir}")
        return

    ratings = _load_ratings()

    print(f"Ranking {len(wav_files)} files in {in_dir}...")
    scored = []
    for w in wav_files:
        s = score_file(w, ratings)
        scored.append(s)
        if not s["pass"]:
            print(f"  ✗ {s['file']}: {'; '.join(s['issues'])}")

    scored.sort(key=lambda x: x["score"], reverse=True)

    print(f"\n--- Rankings ---")
    for i, s in enumerate(scored, 1):
        flag = " ★" if any(r.get("rating") == "favorite" for r in ratings
                          if r.get("file") in s["path"]) else ""
        print(f"  {i:>3}. {s['score']:5.1f}  {s['file']}{flag}")

    report = {
        "total_files": len(scored),
        "rankings": scored,
    }
    report_path = in_dir / "rank_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"\nReport: {report_path}")


def cmd_top(args):
    """Show top N ranked files in a directory."""
    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    n = args.n
    wav_files = sorted(in_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No .wav files found in {in_dir}")
        return

    ratings = _load_ratings()
    scored = [score_file(w, ratings) for w in wav_files if read_wav(w) is not None]
    scored.sort(key=lambda x: x["score"], reverse=True)
    top = scored[:n]

    print(f"Top {n} of {len(scored)} files in {in_dir}:")
    print(f"{'='*60}")
    for i, s in enumerate(top, 1):
        print(f"  {i:>3}. {s['score']:5.1f}  {s['file']}")
        print(f"       RMS={s['features']['rms']:.4f}  "
              f"Centroid={s['features']['spectral_centroid']:.0f}Hz  "
              f"Attack={s['features']['attack_ms']:.1f}ms")
        if s["issues"]:
            print(f"       Issues: {'; '.join(s['issues'])}")
