"""Reference search: find similar references by text or audio features."""
import json
import sys
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, PACKS_DIR, SPANISH_GUITAR_DIR, SUPPORTED_EXTS
from gen.io import read_audio_safe
from gen.features import compute_features


def _scan_references() -> list[dict]:
    """Scan reference directories and build feature index."""
    refs = []
    for root_dir in [PACKS_DIR, SPANISH_GUITAR_DIR]:
        if not root_dir.exists():
            continue
        for f in sorted(root_dir.rglob("*")):
            if f.suffix.lower() in SUPPORTED_EXTS:
                result = read_audio_safe(f)
                if result is None:
                    continue
                samples, sr = result
                feats = compute_features(samples, sr)
                refs.append({
                    "path": str(f.relative_to(REPO_ROOT)),
                    "category": f.parent.name,
                    "features": {
                        "spectral_centroid": float(feats.get("spectral_centroid", 0)),
                        "attack_ms": float(feats.get("attack_ms", 0)),
                        "rms": float(feats.get("rms", 0)),
                        "hpr": float(feats.get("hpr", 0)),
                        "duration_s": round(len(samples) / sr, 3),
                    },
                })
    return refs


def cmd_search_ref(args):
    """Search reference library by text query or similarity."""
    query = " ".join(args.query) if isinstance(args.query, list) else args.query

    refs = _scan_references()
    if not refs:
        print("No references found. Run 'cshot scan' first.")
        return

    print(f"Searching {len(refs)} references for '{query}'...\n")

    # Simple text matching on path/category
    query_lower = query.lower()
    matches = []
    for r in refs:
        score = 0
        path_lower = r["path"].lower()
        cat_lower = r["category"].lower()
        if query_lower in path_lower:
            score += 10
        if query_lower in cat_lower:
            score += 5
        for word in query_lower.split():
            if word in path_lower:
                score += 2
        if score > 0:
            matches.append((score, r))

    matches.sort(key=lambda x: x[0], reverse=True)
    top = matches[:20]

    if not top:
        print("No matches found. Try searching by sound type (kick, clap, bass, etc.)")
        return

    print(f"Top {len(top)} matches:")
    print(f"{'='*60}")
    for score, r in top:
        f = r["features"]
        print(f"  [{r['category']:20s}] {Path(r['path']).name}")
        print(f"      centroid={f['spectral_centroid']:.0f}Hz  "
              f"attack={f['attack_ms']:.1f}ms  "
              f"dur={f['duration_s']:.2f}s  "
              f"score={score}")
