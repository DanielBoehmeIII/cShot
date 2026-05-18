"""Week 25 — Demo Kit Library: 5 flagship curated one-shot kits."""
import json
import shutil
import sys
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.kit_spec import KitSpec, CategoryCounts, DNAProfile, infer_spec_from_prompt
from gen.kit_engine import generate_kit, compute_kit_coherence, setup_kit_export
from gen.quality_gate import run_all_gates
from gen.rank import score_file
from gen.rating import _load_ratings
from gen.polish import polish_file
from gen.features import clear_feature_cache

DEMO_KITS = [
    {
        "name": "Dark_RnB_OneShots",
        "prompt": "dark R&B one shot kit",
        "genre": "rnb",
        "dna": DNAProfile(spectral_centroid_target=2200, transient_aggression=0.4, saturation_density=0.35, stereo_width=0.5, tonal_noise_ratio=0.7, loudness_lufs=-14, brightness=0.35, darkness=0.6, grit=0.3, dryness=0.6, punch=0.4, softness=0.6, analog_warmth=0.3, width_span=0.5),
    },
    {
        "name": "Alien_Trap_Drums",
        "prompt": "alien trap drum kit",
        "genre": "trap",
        "dna": DNAProfile(spectral_centroid_target=3000, transient_aggression=0.8, saturation_density=0.5, stereo_width=0.6, tonal_noise_ratio=0.4, loudness_lufs=-10, brightness=0.6, darkness=0.4, grit=0.6, dryness=0.8, punch=0.8, softness=0.2, analog_warmth=0.1, width_span=0.6),
    },
    {
        "name": "PluggnB_Keys",
        "prompt": "soft pluggnb keys and synths",
        "genre": "rnb",
        "dna": DNAProfile(spectral_centroid_target=4500, transient_aggression=0.2, saturation_density=0.25, stereo_width=0.6, tonal_noise_ratio=0.8, loudness_lufs=-16, brightness=0.6, darkness=0.3, grit=0.15, dryness=0.5, punch=0.2, softness=0.8, analog_warmth=0.4, width_span=0.6),
    },
    {
        "name": "Cinematic_Impacts",
        "prompt": "cinematic impacts and textures",
        "genre": "cinematic",
        "dna": DNAProfile(spectral_centroid_target=5000, transient_aggression=0.6, saturation_density=0.4, stereo_width=0.8, tonal_noise_ratio=0.5, loudness_lufs=-12, brightness=0.7, darkness=0.3, grit=0.4, dryness=0.4, punch=0.6, softness=0.4, analog_warmth=0.1, width_span=0.8),
    },
    {
        "name": "Experimental_Textures",
        "prompt": "experimental textures and fx",
        "genre": "ambient",
        "dna": DNAProfile(spectral_centroid_target=7000, transient_aggression=0.3, saturation_density=0.3, stereo_width=0.8, tonal_noise_ratio=0.4, loudness_lufs=-18, brightness=0.8, darkness=0.2, grit=0.3, dryness=0.3, punch=0.3, softness=0.7, analog_warmth=0.0, width_span=0.8),
    },
]

DEMO_COUNT = 30


def cmd_build_demo_kits(args):
    """Generate all 5 flagship demo kits."""
    out_base = Path(args.out) if args.out else REPO_ROOT / "outputs" / "demo_kits"
    out_base.mkdir(parents=True, exist_ok=True)
    count = args.count

    print(f"Building {len(DEMO_KITS)} Demo Kits")
    print(f"{'='*60}")
    print()

    for kit_def in DEMO_KITS:
        kit_dir = out_base / kit_def["name"]
        if kit_dir.exists():
            shutil.rmtree(kit_dir)

        print(f"[{kit_def['name']}]")
        print(f"  Prompt: '{kit_def['prompt']}'")
        print(f"  Target: {count} sounds")

        cats = CategoryCounts()
        for field in vars(cats):
            setattr(cats, field, 0)
        if kit_def["name"] == "Dark_RnB_OneShots":
            cats.kicks = 3; cats.snares = 2; cats.claps = 2; cats.hats = 4
            cats.percs = 2; cats.basses_808 = 3; cats.basses_sub = 2
            cats.keys = 3; cats.synths = 3; cats.guitars = 2
            cats.impacts = 2; cats.textures = 2
        elif kit_def["name"] == "Alien_Trap_Drums":
            cats.kicks = 4; cats.snares = 3; cats.claps = 3; cats.hats = 5
            cats.percs = 3; cats.basses_808 = 4; cats.synths = 3
            cats.impacts = 2; cats.glitches = 2; cats.textures = 1
        elif kit_def["name"] == "PluggnB_Keys":
            cats.keys = 6; cats.synths = 6; cats.guitars = 3
            cats.basses_808 = 3; cats.basses_sub = 2; cats.textures = 2
            cats.impacts = 2; cats.atmospheres = 2; cats.percs = 4
        elif kit_def["name"] == "Cinematic_Impacts":
            cats.impacts = 5; cats.risers = 3; cats.glitches = 3
            cats.textures = 4; cats.atmospheres = 4; cats.fx_noise = 3
            cats.synths = 3; cats.keys = 2; cats.percs = 3
        elif kit_def["name"] == "Experimental_Textures":
            cats.textures = 5; cats.atmospheres = 4; cats.fx_noise = 4
            cats.glitches = 4; cats.risers = 3; cats.impacts = 3
            cats.synths = 4; cats.percs = 3

        spec = KitSpec(
            name=kit_def["name"],
            prompt=kit_def["prompt"],
            genre=kit_def["genre"],
            target_dna=kit_def["dna"],
            categories=cats,
            total_target=count,
        )

        t0 = time.time()
        generated = generate_kit(spec, kit_dir, polish=True)
        t1 = time.time()
        print(f"  Generated: {generated} files in {t1-t0:.1f}s")

        wav_files = sorted(kit_dir.rglob("*.wav"))
        gate_report = run_all_gates(wav_files)
        removed = 0
        for p, r in gate_report["results"].items():
            if not r["pass"]:
                Path(p).unlink(missing_ok=True)
                removed += 1
        remaining = sorted(kit_dir.rglob("*.wav"))
        print(f"  After gate: {len(remaining)} files (removed {removed})")

        clear_feature_cache()
        for w in remaining:
            polish_file(w, target_db=-1.0, in_place=True)

        coherence = compute_kit_coherence(kit_dir)
        setup_kit_export(kit_dir, spec, coherence)

        top_dir = kit_dir / "_top"
        top_dir.mkdir(exist_ok=True)

        ratings = _load_ratings()
        scored = []
        for w in remaining:
            s = score_file(w, ratings)
            s["path"] = str(w.relative_to(kit_dir))
            scored.append(s)
        scored.sort(key=lambda x: x.get("score", 0), reverse=True)

        for i, s in enumerate(scored[:10]):
            src = kit_dir / s["path"]
            dest = top_dir / f"{i+1:02d}_{src.name}"
            shutil.copy2(src, dest)

        t2 = time.time()
        print(f"  Curated: {len(remaining)} best sounds")
        print(f"  Coherence: {coherence.get('overall_coherence', 0):.3f}")
        print(f"  Total time: {t2-t0:.1f}s")
        print()

    print(f"{'='*60}")
    print(f"Demo Kit Library Complete: {out_base}")
    print()
    for kit_def in DEMO_KITS:
        kit_dir = out_base / kit_def["name"]
        remaining = sorted(kit_dir.rglob("*.wav"))
        print(f"  {kit_def['name']:<25s} {len(remaining)} sounds")
