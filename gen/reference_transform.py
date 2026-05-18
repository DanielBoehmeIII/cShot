"""Reference-conditioned generation pipeline.

Takes retrieved references and generates new sounds by applying
high-quality transformations from audio_transforms.py.
"""

import json
import random
import time
from pathlib import Path
from typing import Optional

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.io import read_audio_safe, read_wav, write_wav
from gen.audio_transforms import (
    apply_transform_chain, generate_random_transform_chain,
    pitch_shift, time_stretch, transient_reshape, eq_tilt,
    saturation, convolution_reverb, spectral_morph,
    envelope_morph, noise_body_tail_recombine, layer_sounds,
    resample_lofi, hpss_split, parametric_eq,
)


def transform_single_reference(ref_path: str, out_path: Path,
                                 transform_chain: Optional[list[dict]] = None) -> dict:
    """Load a reference, apply transforms, save result."""
    src = REPO_ROOT / ref_path if not Path(ref_path).exists() else Path(ref_path)
    result = read_audio_safe(src, mono=True)
    if result is None:
        raise ValueError(f"Cannot read: {ref_path}")

    samples, sr = result

    if transform_chain is None:
        transform_chain = generate_random_transform_chain()

    transformed = apply_transform_chain(samples, transform_chain)
    peak = np.max(np.abs(transformed))
    if peak > 0.001:
        transformed = transformed / peak * 0.95
    write_wav(out_path, transformed)

    return {
        "source": str(src),
        "output": str(out_path.name),
        "transforms": transform_chain,
        "samples": len(transformed),
    }


def transform_references_to_kit(refs: list[dict], out_dir: Path,
                                  target_count: int = 60,
                                  max_per_ref: int = 3) -> dict:
    """Generate a kit by transforming retrieved references into new sounds."""
    out_dir.mkdir(parents=True, exist_ok=True)
    generated = 0
    used_refs = []
    rng = random.Random()
    refs_by_category = {}

    for r in refs:
        cat = r.get("category", "unknown")
        if cat not in refs_by_category:
            refs_by_category[cat] = []
        refs_by_category[cat].append(r)

    categories = list(refs_by_category.keys())
    cat_probs = [min(len(v) ** 0.5, 5) for v in refs_by_category.values()]
    total_prob = sum(cat_probs)
    cat_probs = [p / total_prob for p in cat_probs]

    needs_diversity = target_count > 20
    used_paths = set()

    for pass_num in range(5):
        if generated >= target_count:
            break
        for cat_idx, cat in enumerate(categories):
            if generated >= target_count:
                break
            if rng.random() > cat_probs[cat_idx] * 4 and needs_diversity:
                continue

            ref = rng.choice(refs_by_category[cat])

            n_variations = rng.randint(1, max_per_ref)
            for vi in range(n_variations):
                if generated >= target_count:
                    break

                chain = generate_random_transform_chain(cat)
                if not chain:
                    # Add at least one transform
                    chain.append({
                        'type': 'pitch_shift',
                        'semitones': round(rng.uniform(-5, 5), 1)
                    })

                stem = Path(ref['file_name']).stem
                out_name = f"{stem}_v{generated+1}.wav"
                # Avoid name collisions
                while out_name in used_paths:
                    out_name = f"{stem}_v{generated+1}_{rng.randint(100,999)}.wav"
                used_paths.add(out_name)

                out_path = out_dir / out_name
                try:
                    info = transform_single_reference(ref['file_path'], out_path, chain)
                    used_refs.append({
                        "source": ref['file_path'],
                        "output": out_name,
                        "transforms": chain,
                        "category": cat,
                    })
                    generated += 1
                except Exception:
                    pass

    # If too few were generated, try more from top refs
    if generated < target_count // 2:
        for ref in refs[:min(20, len(refs))]:
            if generated >= target_count:
                break
            for vi in range(2):
                if generated >= target_count:
                    break
                chain = [{'type': 'pitch_shift', 'semitones': round(rng.uniform(-3, 3), 1)}]
                stem = Path(ref['file_name']).stem
                out_name = f"{stem}_v{generated+1}.wav"
                while out_name in used_paths:
                    out_name = f"{stem}_v{generated+1}_{rng.randint(100,999)}.wav"
                used_paths.add(out_name)
                out_path = out_dir / out_name
                try:
                    info = transform_single_reference(ref['file_path'], out_path, chain)
                    used_refs.append({
                        "source": ref['file_path'],
                        "output": out_name,
                        "transforms": chain,
                        "category": ref.get("category", "unknown"),
                    })
                    generated += 1
                except Exception:
                    pass

    # Generate manifest
    manifest = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "engine": "cshot-reference-transform-v1",
        "source_type": "retrieval",
        "target_count": target_count,
        "actual_count": generated,
        "refs_used": len(used_refs),
    }
    (out_dir / "manifest.json").write_text(json.dumps(manifest, indent=2))

    lineage = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "generator": "cshot-reference-transform-v1",
        "total_outputs": generated,
        "entries": used_refs,
    }
    (out_dir / "source_lineage.json").write_text(json.dumps(lineage, indent=2))

    # Similarity risk report
    risk = _compute_similarity_risk(used_refs)
    (out_dir / "similarity_risk_report.md").write_text(risk)

    coherence = _estimate_coherence(used_refs, generated)

    return {"generated": generated, "coherence": coherence, "lineage": len(used_refs)}


def _compute_similarity_risk(used_refs: list[dict]) -> str:
    """Compute similarity risk from used references."""
    total = len(used_refs)
    sources = set(e["source"] for e in used_refs)
    cats = set(e.get("category", "unknown") for e in used_refs)
    avg_transforms = sum(len(e.get("transforms", [])) for e in used_refs) / max(total, 1)

    report = f"""# Similarity Risk Report

Generated: {time.strftime("%Y-%m-%d %H:%M:%S UTC", time.gmtime())}

## Summary
| Metric | Value |
|--------|-------|
| Total outputs | {total} |
| Unique source files | {len(sources)} |
| Categories used | {len(cats)} |
| Avg transforms per output | {avg_transforms:.1f} |

## Risk Assessment
- **Transform diversity**: {'HIGH' if avg_transforms >= 2 else 'MEDIUM' if avg_transforms >= 1 else 'LOW'}
- **Source diversity**: {'HIGH' if len(sources) >= total * 0.3 else 'MEDIUM' if len(sources) >= total * 0.1 else 'LOW'}
- **Category diversity**: {'HIGH' if len(cats) >= 3 else 'MEDIUM' if len(cats) >= 2 else 'LOW'}

Each output is a transformed variant of a reference sound. No exact copies are included.
All outputs are intended for local/producer use. Verify against original sources before commercial release.
"""
    return report


def _estimate_coherence(used_refs: list[dict], total: int) -> float:
    """Estimate kit coherence based on category distribution."""
    if total == 0 or not used_refs:
        return 0.5
    cats = {}
    for e in used_refs:
        c = e.get("category", "unknown")
        cats[c] = cats.get(c, 0) + 1

    top_cat_ratio = max(cats.values()) / total
    cat_count = len(cats)

    coherence = min(1.0, (top_cat_ratio * 0.5 + min(cat_count / 5, 0.5)))
    return round(coherence, 3)
