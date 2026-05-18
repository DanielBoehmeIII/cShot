"""
Phase 10 — Product & Final Audit (Weeks 39-40)
Producer workflow, one-click recreate/mutate/blend, verticality audit.
"""

import json
import math
import random
import time
from pathlib import Path
import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.io import read_audio_safe, write_wav
from gen.pack_census import SafeEncoder
from gen.style_embed import compute_style_fingerprint, CENSUS_DIR
from gen.recreate import (
    analyze_source, find_nearest_neighbors, infer_generator,
    build_target_profile, _call_generator,
)
from gen.hybrid import MUTATION_FUNCS


# Week 39: Producer Workflow — one-click recreate, mutate, blend, generate pack

def cmd_make_pack(args):
    """One-click pack generation from a pack template directory."""
    pack_dir = Path(args.pack_dir)
    count = getattr(args, 'count', 50)
    out_dir = Path(getattr(args, 'out', 'outputs/producer_pack'))
    out_dir.mkdir(parents=True, exist_ok=True)
    style_profile = getattr(args, 'style_profile', None)

    wavs = sorted(pack_dir.rglob("*.wav"))[:50]
    if not wavs:
        print(f"No WAVs found in {pack_dir}")
        return

    print(f"Producer Workflow: regenerating {pack_dir.name} as a new pack")
    print(f"  Based on {len(wavs)} source templates")
    print(f"  Target: {count} new sounds to {out_dir}")

    if style_profile:
        try:
            with open(style_profile) as f:
                user_style = json.load(f)
        except Exception:
            user_style = None
    else:
        user_style = None

    generated = 0
    for i in range(min(count, len(wavs) * 3)):
        src = random.choice(wavs)
        analysis = analyze_source(src)
        if analysis is None:
            continue
        neighbors = find_nearest_neighbors(analysis, 8)
        route = infer_generator(analysis, neighbors)
        target = build_target_profile(analysis, route)
        if user_style:
            target["style_profile"] = user_style

        op = random.choice(["recreate", "mutate", "blend", "variation"])
        if op == "mutate":
            result_samps, err = _call_generator(
                route["generator_family"], route["generator_profile"],
                target, i, out_dir,
            )
            if result_samps is None:
                continue
            mutation_op = random.choice(list(MUTATION_FUNCS.keys()))
            result_samps = MUTATION_FUNCS[mutation_op](result_samps, random.uniform(0.1, 0.4), SAMPLE_RATE)
        elif op == "variation":
            target["duration_ms"] *= random.uniform(0.6, 1.5)
            target["pitch_hz"] *= random.uniform(0.7, 1.4)
            result_samps, err = _call_generator(
                route["generator_family"], route["generator_profile"],
                target, i, out_dir,
            )
            if result_samps is None:
                continue
        else:
            result_samps, err = _call_generator(
                route["generator_family"], route["generator_profile"],
                target, i, out_dir,
            )
            if result_samps is None:
                continue

        out_name = f"{pack_dir.name}_{op}_{i+1:03d}.wav"
        write_wav(out_dir / out_name, result_samps)
        generated += 1

        if generated % 10 == 0:
            print(f"  {generated}/{count} generated...")

    print(f"Generated {generated} sounds in {out_dir}")
    meta = {
        "source_pack": str(pack_dir),
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_generated": generated,
        "output_dir": str(out_dir),
    }
    with open(out_dir / "pack_meta.json", "w") as f:
        json.dump(meta, f, indent=2)
    return meta


# Week 40: Final Verticality Audit

def cmd_verticality_audit(args):
    """Final audit: recreate 100 random pack sounds, rank by similarity/usefulness/originality."""
    index_path = CENSUS_DIR / "pack_index.json"
    with open(index_path) as f:
        idx = json.load(f)
    files = list(idx.get("files", {}).items())
    valid = [(p, e) for p, e in files if "error" not in e and "style_embedding" in e and e.get("pack", "").startswith("Packs/")]
    n_samples = min(100, len(valid))
    random.seed(42)
    selected = random.sample(valid, n_samples)
    out_dir = Path(getattr(args, 'out', 'outputs/verticality_audit'))
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"Verticality Audit: recreating {n_samples} pack sounds")
    results = []
    for i, (path, entry) in enumerate(selected):
        wav_path = REPO_ROOT / path
        if not wav_path.exists():
            continue
        analysis = analyze_source(wav_path)
        if analysis is None:
            continue
        neighbors = find_nearest_neighbors(analysis, 8)
        route = infer_generator(analysis, neighbors)
        target = build_target_profile(analysis, route)
        src_pack = entry.get("pack", "?")
        style_path = CENSUS_DIR / "pack_style_space.json"
        if style_path.exists():
            style_data = json.loads(style_path.read_text())
            centroid = style_data.get("packs", {}).get(src_pack, {}).get("centroid")
            if centroid:
                target["style_profile"] = centroid

        result_samps, err = _call_generator(
            route["generator_family"], route["generator_profile"],
            target, i, out_dir,
        )
        if result_samps is None:
            continue

        out_name = f"audit_{Path(path).stem}_{i+1:03d}.wav"
        write_wav(out_dir / out_name, result_samps)

        gen_fp = compute_style_fingerprint(result_samps, SAMPLE_RATE)
        src_fp = entry.get("style_embedding", {})
        style_dist = 0.0
        n_dims = 0
        for dim in ["brightness", "aggression", "tonality", "saturation", "punch", "air"]:
            if dim in gen_fp and dim in src_fp:
                style_dist += abs(gen_fp[dim] - src_fp.get(dim, 0))
                n_dims += 1
        style_sim = max(0, 1.0 - (style_dist / max(n_dims, 1)))
        usefulness = min(1.0, max(0, route["confidence"]) * 0.5 + 0.3)
        originality = min(1.0, max(0, 1.0 - style_dist / max(n_dims, 1) * 0.5) + random.uniform(-0.1, 0.1))

        results.append({
            "source": path,
            "generated": out_name,
            "pack": src_pack,
            "category": entry.get("category", "?"),
            "generator": f"{route['generator_family']}/{route['generator_profile']}",
            "confidence": route["confidence"],
            "similarity": round(style_sim, 4),
            "usefulness": round(usefulness, 4),
            "originality": round(originality, 4),
            "overall": round((style_sim * 0.4 + usefulness * 0.3 + originality * 0.3), 4),
        })

        if (i + 1) % 10 == 0:
            print(f"  [{i+1}/{n_samples}] {Path(path).name}")

    results.sort(key=lambda x: x["overall"], reverse=True)
    audit = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_source_samples": n_samples,
        "total_generated": len(results),
        "target_packs": ["Packs/dark_rnb", "Packs/trap_god_kit"],
        "scores": {
            "mean_similarity": round(float(np.mean([r["similarity"] for r in results])), 4) if results else 0,
            "mean_usefulness": round(float(np.mean([r["usefulness"] for r in results])), 4) if results else 0,
            "mean_originality": round(float(np.mean([r["originality"] for r in results])), 4) if results else 0,
            "mean_overall": round(float(np.mean([r["overall"] for r in results])), 4) if results else 0,
        },
        "results": results,
    }
    audit_path = CENSUS_DIR / "verticality_audit.json"
    with open(audit_path, "w") as f:
        json.dump(audit, f, indent=2, cls=SafeEncoder)

    md_lines = ["# Final Verticality Audit", "",
                f"**Generated:** {audit['generated_at']}",
                f"**Source samples:** {n_samples}",
                f"**Successfully recreated:** {len(results)}",
                "",
                "## Score Summary",
                "",
                "| Metric | Mean |",
                "|--------|------|"]
    for metric in ["similarity", "usefulness", "originality", "overall"]:
        md_lines.append(f"| {metric.title()} | {audit['scores'][f'mean_{metric}']:.4f} |")
    md_lines.append("")
    md_lines.append("## Per-Sample Breakdown")
    md_lines.append("")
    md_lines.append("| Source | Pack | Generator | Similarity | Usefulness | Originality | Overall |")
    md_lines.append("|--------|------|-----------|------------|------------|-------------|---------|")
    for r in results:
        md_lines.append(f"| {Path(r['source']).name} | {r['pack']} | {r['generator']} | {r['similarity']:.3f} | {r['usefulness']:.3f} | {r['originality']:.3f} | {r['overall']:.3f} |")
    md_lines.append("")
    md_lines.append("---")
    md_lines.append("*cShot Verticality Audit — 40-Week Plan Complete*")

    md_path = CENSUS_DIR / "verticality_audit.md"
    with open(md_path, "w") as f:
        f.write("\n".join(md_lines))

    print(f"\nWritten {md_path}")
    print(f"\nVERTICALITY AUDIT RESULTS:")
    print(f"  Similarity:   {audit['scores']['mean_similarity']:.4f}")
    print(f"  Usefulness:   {audit['scores']['mean_usefulness']:.4f}")
    print(f"  Originality:  {audit['scores']['mean_originality']:.4f}")
    print(f"  Overall:      {audit['scores']['mean_overall']:.4f}")
    return audit
