"""cshot demo — fast demo mode: generate a 20-sound kit in under 2 minutes."""
import json
import shutil
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.kit_spec import KitSpec, CategoryCounts, DNAProfile
from gen.kit_engine import generate_kit, compute_kit_coherence, setup_kit_export
from gen.rank import score_file
from gen.rating import _load_ratings
from gen.listen import cmd_listen


def cmd_demo(args):
    """Generate a 20-sound demo kit in under 2 minutes."""
    prompt = " ".join(args.prompt) if isinstance(args.prompt, list) else args.prompt
    count = args.count
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "demo" / f"demo_{time.strftime('%Y%m%d_%H%M%S')}"

    print(f"cShot Demo Mode")
    print(f"{'='*60}")
    print(f"Prompt: '{prompt}'")
    print(f"Target: {count} sounds")
    print(f"Output: {out_dir}")
    print()

    t0 = time.time()

    cats = CategoryCounts()
    for field in vars(cats):
        setattr(cats, field, 0)
    distribution = {
        "kicks": 0.15, "snares": 0.10, "claps": 0.10, "hats": 0.15,
        "percs": 0.10, "basses_808": 0.10, "keys": 0.10, "synths": 0.10,
        "impacts": 0.05, "textures": 0.05,
    }
    total_assigned = 0
    for field, pct in distribution.items():
        n = max(1, int(round(count * pct)))
        setattr(cats, field, n)
        total_assigned += n

    spec = KitSpec(
        name=f"demo_{prompt.replace(' ', '_')[:20]}",
        prompt=prompt,
        genre="custom",
        target_dna=DNAProfile(
            spectral_centroid_target=3000,
            transient_aggression=0.5,
            saturation_density=0.3,
            stereo_width=0.4,
            tonal_noise_ratio=0.6,
            loudness_lufs=-14,
            brightness=0.5,
            darkness=0.5,
            grit=0.3,
            dryness=0.6,
            punch=0.5,
            softness=0.5,
            analog_warmth=0.2,
            width_span=0.4,
        ),
        categories=cats,
        total_target=count,
    )

    generated = generate_kit(spec, out_dir, polish=True)
    t1 = time.time()

    print(f"\n{'='*60}")
    print(f"Generated {generated}/{count} sounds in {t1-t0:.1f}s")
    print()

    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"Coherence:  {coherence.get('overall_coherence', 0):.3f}")
        print(f"Centroid:   {coherence.get('centroid_mean', 0):.0f} Hz")
        print(f"Diversity:  {coherence.get('category_diversity', 0):.3f}")
        print()

    wav_files = sorted(out_dir.rglob("*.wav"))
    ratings = _load_ratings()
    scored = []
    for w in wav_files:
        s = score_file(w, ratings)
        s["path"] = str(w.relative_to(out_dir))
        scored.append(s)
    scored.sort(key=lambda x: x.get("score", 0), reverse=True)

    top_dir = out_dir / "_top"
    top_dir.mkdir(exist_ok=True)
    top_n = min(10, len(scored))
    for i in range(top_n):
        s = scored[i]
        src = Path(s["path"])
        dest = top_dir / f"{i+1:02d}_{src.name}"
        shutil.copy2(out_dir / src, dest)

    setup_kit_export(out_dir, spec, coherence)

    manifest = {
        "demo": True,
        "prompt": prompt,
        "count": generated,
        "generation_time_s": round(t1 - t0, 1),
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "coherence": coherence,
        "top_sounds": [{"rank": i+1, "file": s["path"], "score": round(s.get("score", 0), 3)} for i, s in enumerate(scored[:10])],
    }
    manifest_path = out_dir / "demo_manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)

    print(f"{'='*60}")
    print(f"Demo kit ready!")
    print(f"  Location: {out_dir}")
    print(f"  Files:    {generated}")
    print(f"  Time:     {t1-t0:.1f}s")
    print(f"  Manifest: {manifest_path}")
    print(f"  Top 10:   {top_dir}")
    print()
    print(f"To listen:")
    print(f"  cshot listen {out_dir}")
    print()
    print(f"To rate:")
    print(f"  cshot rate <file> --rating favorite|trash")
