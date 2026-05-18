"""
Phase 8 — Dataset Intelligence (Weeks 31-34)
Duplicate detection, quality ranking, producer fingerprinting, pack imitation.
"""

import json
import math
import random
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.pack_census import SafeEncoder, compute_crest_factor
from gen.features import compute_features
from gen.io import read_audio_safe, write_wav
from gen.style_embed import (
    compute_style_fingerprint, embed_distance, STYLE_DIMENSIONS,
    CENSUS_DIR,
)
from gen.recreate import analyze_source, find_nearest_neighbors, infer_generator, build_target_profile, _call_generator

CENSUS_DIR = REPO_ROOT / "gen" / "census"


# Week 31: Duplicate/Derivative Detection

def compute_feature_hash(entry: dict, precision: int = 2) -> str:
    keys = ["spectral_centroid", "spectral_bandwidth", "hpr", "crest_factor",
            "low_band_energy", "mid_band_energy", "high_band_energy",
            "transient_count", "attack_ms", "decay_length_ms"]
    vals = []
    for k in keys:
        v = entry.get(k, 0)
        if isinstance(v, float):
            vals.append(round(v, precision))
        else:
            vals.append(v)
    return str(vals)


def cmd_find_duplicates(args):
    index_path = CENSUS_DIR / "pack_index.json"
    with open(index_path) as f:
        index = json.load(f)
    files = index.get("files", {})
    hashes = defaultdict(list)
    for path, entry in files.items():
        if "error" in entry:
            continue
        h = compute_feature_hash(entry)
        hashes[h].append(path)
    duplicates = {h: paths for h, paths in hashes.items() if len(paths) > 1}
    similarity_threshold = getattr(args, 'threshold', 0.95)
    graph = defaultdict(list)
    for h, paths in duplicates.items():
        for p in paths:
            for p2 in paths:
                if p != p2:
                    graph[p].append(p2)
    result = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_duplicate_groups": len(duplicates),
        "total_files_involved": sum(len(v) for v in duplicates.values()),
        "duplicate_graph": graph,
        "duplicate_groups": [{"hash": h, "files": paths} for h, paths in duplicates.items()],
    }
    output_path = CENSUS_DIR / "duplicates.json"
    with open(output_path, "w") as f:
        json.dump(result, f, indent=2, cls=SafeEncoder)
    print(f"Found {len(duplicates)} duplicate groups ({sum(len(v) for v in duplicates.values())} files)")
    return result


# Week 32: Quality Ranking

QUALITY_METRICS = ["punch", "clarity", "width", "originality", "production_quality"]


def score_quality(entry: dict) -> dict:
    punch = min(1.0, entry.get("crest_factor", 1) / 12.0) * 0.3
    punch += min(1.0, entry.get("early_rms", 0.5) * 2.0) * 0.3
    punch += min(1.0, (1.0 - min(entry.get("attack_ms", 10) / 50.0, 1.0))) * 0.4
    punch = min(1.0, punch)

    clarity = min(1.0, entry.get("spectral_centroid", 1000) / 6000.0) * 0.5
    clarity += min(1.0, 1.0 - abs(entry.get("hpr", 0.5) - 0.5) * 2.0) * 0.3
    clarity += min(1.0, entry.get("spectral_flux_mean", 0) * 0.5) * 0.2
    clarity = min(1.0, clarity)

    width = min(1.0, entry.get("stereo_width", 0) * 3.0)

    hpr = entry.get("hpr", 0.5)
    originality = min(1.0, abs(hpr - 0.5) * 2.0) * 0.5
    originality += min(1.0, entry.get("spectral_bandwidth", 0) / 5000.0) * 0.5
    originality = min(1.0, originality)

    production_quality = (punch + clarity + width) / 3.0

    overall = (punch * 0.3 + clarity * 0.25 + width * 0.15 + originality * 0.15 + production_quality * 0.15)
    return {
        "overall": round(overall, 4),
        "punch": round(punch, 4),
        "clarity": round(clarity, 4),
        "width": round(width, 4),
        "originality": round(originality, 4),
        "production_quality": round(production_quality, 4),
    }


def cmd_quality_rank(args):
    index_path = CENSUS_DIR / "pack_index.json"
    with open(index_path) as f:
        index = json.load(f)
    files = index.get("files", {})
    rankings = []
    for path, entry in files.items():
        if "error" in entry:
            continue
        scores = score_quality(entry)
        s = entry.get("style_embedding", {})
        rankings.append({
            "file": path,
            "pack": entry.get("pack", "?"),
            "category": entry.get("category", "?"),
            "scores": scores,
            "style": s,
        })
    rankings.sort(key=lambda x: x["scores"]["overall"], reverse=True)
    result = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_ranked": len(rankings),
        "top_100": rankings[:100],
        "rankings": rankings,
    }
    output_path = CENSUS_DIR / "quality_rankings.json"
    with open(output_path, "w") as f:
        json.dump(result, f, indent=2, cls=SafeEncoder)
    print(f"Ranked {len(rankings)} files")
    print(f"Top 5:")
    for r in rankings[:5]:
        print(f"  {r['file'][:60]:<62} overall={r['scores']['overall']:.4f}")
    return result


# Week 33: Producer Fingerprinting

def cmd_producer_fingerprint(args):
    index_path = CENSUS_DIR / "pack_index.json"
    with open(index_path) as f:
        idx = json.load(f)
    files = idx.get("files", {})
    pack_style = defaultdict(list)
    for path, entry in files.items():
        if "error" in entry or "style_embedding" not in entry:
            continue
        pack = entry.get("pack", "unknown")
        pack_style[pack].append(entry["style_embedding"])
    fingerprints = {}
    for pack, embeddings in pack_style.items():
        if len(embeddings) < 5:
            continue
        centroid = {}
        spread = {}
        for dim in STYLE_DIMENSIONS:
            vals = [e[dim] for e in embeddings]
            centroid[dim] = round(float(np.mean(vals)), 4)
            spread[dim] = round(float(np.std(vals)), 4)
        fingerprints[pack] = {
            "num_samples": len(embeddings),
            "centroid": centroid,
            "spread": spread,
            "signature": [centroid[d] for d in STYLE_DIMENSIONS],
        }
    result = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_producers": len(fingerprints),
        "dimensions": STYLE_DIMENSIONS,
        "fingerprints": fingerprints,
    }
    output_path = CENSUS_DIR / "producer_fingerprints.json"
    with open(output_path, "w") as f:
        json.dump(result, f, indent=2, cls=SafeEncoder)
    print(f"Found {len(fingerprints)} producer fingerprints")
    for pack, fp in sorted(fingerprints.items(), key=lambda x: -x[1]["num_samples"])[:10]:
        print(f"  {pack:<45} {fp['num_samples']:>4} files  sig={fp['signature'][:4]}")
    return result


# Week 34: "Could Belong In This Pack" — Imitation

def cmd_imitate(args):
    target_pack = args.pack_name
    count = getattr(args, 'count', 10)
    out_dir = Path(getattr(args, 'out', 'outputs/imitations'))
    out_dir.mkdir(parents=True, exist_ok=True)

    index_path = CENSUS_DIR / "pack_index.json"
    style_path = CENSUS_DIR / "pack_style_space.json"

    with open(index_path) as f:
        idx = json.load(f)
    with open(style_path) as f:
        style_data = json.load(f)

    if target_pack not in style_data.get("packs", {}):
        print(f"Error: pack '{target_pack}' not found")
        return

    pack_style = style_data["packs"][target_pack]["centroid"]
    pack_files = [p for p, e in idx.get("files", {}).items()
                  if e.get("pack") == target_pack and "error" not in e]

    if not pack_files:
        print(f"No valid files in pack '{target_pack}'")
        return

    template_path = REPO_ROOT / pack_files[random.randint(0, len(pack_files) - 1)]
    analysis = analyze_source(template_path)
    neighbors = find_nearest_neighbors(analysis, 8)
    route = infer_generator(analysis, neighbors)

    print(f"Imitating pack: {target_pack}")
    print(f"  Template: {template_path.name}")
    print(f"  Generator: {route['generator_family']}/{route['generator_profile']}")
    print(f"  Pack centroid: bright={pack_style['brightness']:.3f}, "
          f"aggr={pack_style['aggression']:.3f}, tonal={pack_style['tonality']:.3f}")

    generated = []
    for i in range(count):
        target = build_target_profile(analysis, route)
        target["style_profile"] = pack_style
        for dim in STYLE_DIMENSIONS:
            if dim in ("saturation", "punch", "air", "brightness", "width"):
                pack_val = pack_style.get(dim, 0.5)
                noise = random.gauss(0, 0.05)
                dim_val = max(0, min(1, pack_val + noise))
                if dim == "saturation":
                    target["drive"] = dim_val
                elif dim == "punch":
                    target["attack_strength"] = dim_val
                elif dim == "air":
                    target["air"] = dim_val
                elif dim == "brightness":
                    target["brightness"] = dim_val
                elif dim == "width":
                    target["stereo_width"] = dim_val

        samples, error = _call_generator(
            route["generator_family"], route["generator_profile"],
            target, i, out_dir,
        )
        if samples is None:
            continue

        out_name = f"imitate_{target_pack.replace('/','_').replace(' ','_')}_{i+1:03d}.wav"
        out_path = out_dir / out_name
        write_wav(out_path, samples)
        generated.append(str(out_path))
        print(f"  [{i+1}/{count}] {out_name}")

    print(f"Generated {len(generated)} imitations in {out_dir}")
    return generated
