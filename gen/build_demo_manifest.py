"""Build golden demo manifest: scan generated outputs, compute features, rank, select top 50."""
import json
import hashlib
import time
from pathlib import Path

import numpy as np

from gen import SAMPLE_RATE
from gen.io import read_wav
from gen.features import compute_features


def build_manifest(input_dir: Path, output_path: Path, top_n: int = 50):
    wav_files = sorted(input_dir.rglob("*.wav"))
    print(f"Scanning {len(wav_files)} files in {input_dir} ...")

    entries = []
    for w in wav_files:
        cat = w.parent.name
        result = read_wav(w)
        if result is None:
            continue
        samples, sr = result
        feats = compute_features(samples, sr)
        quality_score = _compute_quality_score(feats)

        prompt_str = _infer_prompt(cat, w)
        family = _infer_family(cat)

        entries.append({
            "file": str(w.relative_to(input_dir)),
            "category": cat,
            "prompt": prompt_str,
            "family": family,
            "path": str(w),
            "seed": _extract_seed(w),
            "sample_rate": sr,
            "duration_s": round(len(samples) / sr, 3),
            "features": {
                "rms": round(float(feats.get("rms", 0)), 6),
                "peak": round(float(feats.get("peak", 0)), 6),
                "spectral_centroid": round(float(feats.get("spectral_centroid", 0)), 2),
                "attack_ms": round(float(feats.get("attack_ms", 0)), 2),
                "hpr": round(float(feats.get("hpr", 0)), 4),
                "noise_floor": round(float(feats.get("noise_floor", 0)), 6),
                "duration_s": round(len(samples) / sr, 3),
            },
            "quality_score": round(quality_score, 3),
        })

    # Balanced selection: pick top per category, then fill with best remaining
    entries.sort(key=lambda e: e["quality_score"], reverse=True)

    categories = {}
    for e in entries:
        categories.setdefault(e["category"], []).append(e)

    # Pick top 2 from each category, then fill to top_n with best remaining
    selected = []
    seen_files = set()
    cat_count = max(2, top_n // len(categories) if len(categories) > 0 else 2)
    for cat in sorted(categories.keys()):
        for e in categories[cat][:cat_count]:
            if e["file"] not in seen_files and len(selected) < top_n:
                selected.append(e)
                seen_files.add(e["file"])
    # Fill remaining spots with best unselected
    for e in entries:
        if e["file"] not in seen_files and len(selected) < top_n:
            selected.append(e)
            seen_files.add(e["file"])

    manifest = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "total_candidates": len(entries),
        "top_n": len(selected),
        "categories": sorted(set(e["category"] for e in entries)),
        "selection_strategy": "top-2-per-category-then-best-remaining",
        "entries": entries,
        "top_entries": selected,
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"Manifest written to {output_path}")
    print(f"  Total: {len(entries)} candidates")
    print(f"  Selected: {len(selected)} (balanced across {len(categories)} categories)")

    summary_path = output_path.with_name("demo_top50_summary.txt")
    with open(summary_path, "w") as f:
        f.write(f"Golden Demo — Top {len(selected)} Balanced Selections\n")
        f.write(f"{'='*60}\n\n")
        cats_shown = {}
        for i, e in enumerate(selected, 1):
            cats_shown[e["category"]] = cats_shown.get(e["category"], 0) + 1
            f.write(f"{i:>3}. [{e['category']:20s}] {Path(e['file']).name}\n")
            f.write(f"     Family: {e['family']:12s} | Prompt: {e['prompt']}\n")
            f.write(f"     Score: {e['quality_score']:.3f} | "
                    f"RMS: {e['features']['rms']:.4f} | "
                    f"Peak: {e['features']['peak']:.4f} | "
                    f"Centroid: {e['features']['spectral_centroid']:.0f} | "
                    f"Attack: {e['features']['attack_ms']:.1f}ms | "
                    f"Dur: {e['features']['duration_s']:.2f}s\n")
        f.write(f"\nCategory distribution:\n")
        for cat in sorted(cats_shown):
            f.write(f"  {cat:20s}: {cats_shown[cat]} files\n")
    print(f"Summary written to {summary_path}")


def _compute_quality_score(feats: dict) -> float:
    score = 1.0
    peak = feats.get("peak", 0)
    rms = feats.get("rms", 0)
    noise_floor = feats.get("noise_floor", 0)

    if peak > 0.99:
        score *= 0.5
    if rms < 0.001:
        score *= 0.3
    if 0.05 < rms < 0.5:
        score *= 1.2
    elif rms < 0.01:
        score *= 0.7
    if noise_floor < 0.005:
        score *= 1.1
    elif noise_floor > 0.02:
        score *= 0.8
    attack = feats.get("attack_ms", 10)
    if 2 < attack < 80:
        score *= 1.1
    return score


def _infer_prompt(cat: str, path: Path) -> str:
    name = path.stem
    if name.startswith("prompt_"):
        parts = name.replace("prompt_", "", 1).rsplit("_", 1)[0]
        return parts.replace("_", " ")
    cat_lower = cat.replace("_", " ")
    return cat_lower


def _infer_family(cat: str) -> str:
    piano_keywords = {"piano", "bell", "rhodes"}
    synth_keywords = {"synth", "pluck", "pad", "lead", "bright", "dark", "distorted",
                      "clean", "narrow", "wide", "lo_fi", "soft", "punchy"}
    bass_keywords = {"808", "bass_stab", "distorted_bass", "fm_bass", "reese"}
    guitar_keywords = {"guitar", "bright_guitar", "nylon"}
    fx_keywords = {"fx", "riser", "glitch", "impact"}
    drum_keywords = {"kick", "snare", "clap", "hat", "closed_hat", "open_hat"}

    if cat in drum_keywords:
        return "drums"
    if cat in piano_keywords:
        return "piano"
    if cat in synth_keywords:
        return "synth"
    if cat in bass_keywords:
        return "bass"
    if cat in guitar_keywords:
        return "guitar"
    if cat in fx_keywords:
        return "fx"
    return "other"


def _extract_seed(path: Path) -> int:
    h = hashlib.md5(path.stem.encode()).hexdigest()
    return int(h[:8], 16)


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("input_dir")
    parser.add_argument("--output", "-o", default="outputs/demo_manifest.json")
    parser.add_argument("--top", "-n", type=int, default=50)
    args = parser.parse_args()
    build_manifest(Path(args.input_dir), Path(args.output), args.top)
