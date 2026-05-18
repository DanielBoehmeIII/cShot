"""Quality passes for focused sound family generation and curation."""
import json
import shutil
import sys
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.kit_spec import KitSpec, CategoryCounts, DNAProfile
from gen.kit_engine import generate_kit, compute_kit_coherence, setup_kit_export
from gen.features import clear_feature_cache
from gen.polish import polish_file
from gen.quality_gate import run_all_gates
from gen.rank import score_file
from gen.rating import _load_ratings

FAMILY_CONFIGS = {
    "drums": {
        "label": "Drums (kicks, snares, claps, hats, percs)",
        "dna": DNAProfile(spectral_centroid_target=3500, transient_aggression=0.7, saturation_density=0.4, stereo_width=0.3, tonal_noise_ratio=0.4, loudness_lufs=-10, brightness=0.5, darkness=0.3, grit=0.4, dryness=0.7, punch=0.7, softness=0.3, analog_warmth=0.1, width_span=0.3),
        "cats": {"kicks": 0.2, "snares": 0.15, "claps": 0.15, "hats": 0.25, "open_hats": 0.1, "percs": 0.15},
    },
    "bass": {
        "label": "Bass (808s, sub, deep)",
        "dna": DNAProfile(spectral_centroid_target=2000, transient_aggression=0.4, saturation_density=0.5, stereo_width=0.4, tonal_noise_ratio=0.8, loudness_lufs=-8, brightness=0.3, darkness=0.6, grit=0.5, dryness=0.5, punch=0.4, softness=0.6, analog_warmth=0.3, width_span=0.4),
        "cats": {"basses_808": 0.5, "basses_sub": 0.5},
    },
    "tonal": {
        "label": "Tonal (keys, synths, guitars)",
        "dna": DNAProfile(spectral_centroid_target=5000, transient_aggression=0.3, saturation_density=0.3, stereo_width=0.5, tonal_noise_ratio=0.7, loudness_lufs=-14, brightness=0.6, darkness=0.3, grit=0.2, dryness=0.6, punch=0.3, softness=0.7, analog_warmth=0.2, width_span=0.5),
        "cats": {"keys": 0.3, "synths": 0.4, "guitars": 0.3},
    },
    "fx": {
        "label": "FX (impacts, risers, downlifters, glitches, textures)",
        "dna": DNAProfile(spectral_centroid_target=6000, transient_aggression=0.5, saturation_density=0.4, stereo_width=0.7, tonal_noise_ratio=0.5, loudness_lufs=-12, brightness=0.7, darkness=0.2, grit=0.4, dryness=0.4, punch=0.5, softness=0.5, analog_warmth=0.1, width_span=0.7),
        "cats": {"impacts": 0.2, "risers": 0.15, "glitches": 0.15, "textures": 0.2, "atmospheres": 0.15, "fx_noise": 0.15},
    },
}


def cmd_quality_pass(args):
    """Generate and rank sounds for a specific family."""
    family = args.family
    count = args.count
    keep = args.keep
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "quality" / f"{family}_pass_{time.strftime('%Y%m%d_%H%M%S')}"

    config = FAMILY_CONFIGS[family]

    print(f"Quality Pass — {config['label']}")
    print(f"{'='*60}")
    print(f"Target: {count} sounds (curate to {keep})")
    print(f"Output: {out_dir}")
    print()

    cats = CategoryCounts()
    for field in vars(cats):
        setattr(cats, field, 0)
    for cat_name, pct in config["cats"].items():
        if hasattr(cats, cat_name):
            setattr(cats, cat_name, max(1, int(round(count * pct))))

    spec = KitSpec(
        name=f"{family}_quality_pass",
        prompt=f"{family} one shot collection",
        genre=family,
        target_dna=config["dna"],
        categories=cats,
        total_target=count,
    )

    t0 = time.time()
    generated = generate_kit(spec, out_dir, polish=True)
    t1 = time.time()
    print(f"  → {generated} sounds in {t1-t0:.1f}s")
    print()

    print(f"[2/4] Running quality gates...")
    wav_files = sorted(out_dir.rglob("*.wav"))
    dup_thresh = 0.96 if family in ("drums", "bass") else 0.97
    gate_report = run_all_gates(wav_files, dup_threshold=dup_thresh)
    for p, r in gate_report["results"].items():
        if not r["pass"]:
            Path(p).unlink(missing_ok=True)
    gate_stats = gate_report["stats"]
    print(f"  → {gate_stats['passed']}/{gate_stats['total']} passed")
    print()

    print(f"[3/4] Polishing...")
    remaining = sorted(out_dir.rglob("*.wav"))
    for w in remaining:
        polish_file(w, target_db=-1.0, in_place=True)
    print()

    print(f"[4/4] Ranking and selecting top {keep}...")
    clear_feature_cache()
    ratings = _load_ratings()
    scored = []
    for w in remaining:
        s = score_file(w, ratings)
        s["path"] = str(w.relative_to(out_dir))
        scored.append(s)
    scored.sort(key=lambda x: x.get("score", 0), reverse=True)

    keep_actual = min(keep, len(scored))
    curated = set()
    for i in range(keep_actual):
        s = scored[i]
        src = out_dir / s["path"]
        curated.add(src)

    for w in remaining:
        if w not in curated:
            w.unlink()

    final_files = sorted(out_dir.rglob("*.wav"))
    t2 = time.time()
    print(f"  → {len(final_files)} curated sounds kept")
    print()

    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"Coherence: {coherence.get('overall_coherence', 0):.3f}")

    report = {
        "family": family,
        "generated": generated,
        "after_gate": gate_stats["passed"],
        "final": len(final_files),
        "time_s": round(t2 - t0, 1),
    }
    report_path = out_dir / "quality_pass_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)

    print(f"\n{'='*60}")
    print(f"Quality pass: {family}")
    print(f"  Generated: {generated} → Curated: {len(final_files)}")
    print(f"  Time: {t2-t0:.1f}s")
    print(f"  Report: {report_path}")
    print(f" Listen: cshot listen {out_dir}")
    print(f" Review: cshot gate {out_dir}")
