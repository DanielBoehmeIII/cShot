"""Auto-curation: over-generate, quality gate, rank, keep only best."""
import json
import shutil
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.kit_spec import infer_spec_from_prompt
from gen.kit_engine import generate_kit, compute_kit_coherence, setup_kit_export
from gen.quality_gate import run_all_gates
from gen.rank import score_file
from gen.rating import _load_ratings
from gen.polish import polish_file


def cmd_curate_pack(args):
    """Over-generate, gate, rank, keep only top sounds."""
    prompt = " ".join(args.prompt) if isinstance(args.prompt, list) else args.prompt
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "curated" / prompt.replace(" ", "_").lower()
    target = args.target
    use_gate = not getattr(args, 'no_gate', False)
    generate_count = int(target * args.overgenerate)

    print(f"cShot Curate Pack — Auto-Curation")
    print(f"{'='*60}")
    print(f"Prompt:          '{prompt}'")
    print(f"Generate:        {generate_count} sounds")
    print(f"Target keep:     {target} sounds")
    print(f"Overgenerate:    {args.overgenerate}x")
    print(f"Quality gate:    {use_gate}")
    print()

    t0 = time.time()
    spec = infer_spec_from_prompt(prompt)
    total = sum(v for k, v in vars(spec.categories).items() if isinstance(v, int))
    if total > 0 and total != generate_count:
        ratio = generate_count / total
        for field in vars(spec.categories):
            current = getattr(spec.categories, field)
            if isinstance(current, int):
                setattr(spec.categories, field, max(1, int(round(current * ratio))))
    spec.total_target = generate_count

    print(f"[1/4] Generating {generate_count} sounds...")
    generated = generate_kit(spec, out_dir, polish=True)
    print(f"  → {generated} files generated")
    print()

    wav_files = sorted(out_dir.rglob("*.wav"))

    if use_gate:
        print(f"[2/4] Running quality gates...")
        gate_report = run_all_gates(wav_files)
        failed_files = [Path(p) for p, r in gate_report["results"].items() if not r["pass"]]
        for w in failed_files:
            if w.exists():
                w.unlink()
        gate_stats = gate_report["stats"]
        print(f"  → {gate_stats['passed']}/{gate_stats['total']} passed, {gate_stats['failed']} removed")
        wav_files = sorted(out_dir.rglob("*.wav"))

    print(f"[3/4] Ranking {len(wav_files)} sounds...")
    ratings = _load_ratings()
    scored = []
    for w in wav_files:
        s = score_file(w, ratings)
        s["path"] = str(w.relative_to(out_dir))
        scored.append(s)
    scored.sort(key=lambda x: x.get("score", 0), reverse=True)
    print(f"  → Ranked")
    print()

    keep = min(target, len(scored))
    top_dir = out_dir / "_curated"
    top_dir.mkdir(exist_ok=True)

    kept_count = 0
    for i in range(keep):
        s = scored[i]
        src = out_dir / s["path"]
        dest = top_dir / src.name
        shutil.copy2(src, dest)
        kept_count += 1

    for w in wav_files:
        if w.parent == top_dir:
            continue
        w.unlink()

    for w in top_dir.iterdir():
        dest = out_dir / w.name
        shutil.move(str(w), str(dest))
    top_dir.rmdir()

    wav_files = sorted(out_dir.rglob("*.wav"))
    for w in wav_files:
        polish_file(w, target_db=-1.0, in_place=True)

    coherence = compute_kit_coherence(out_dir)
    setup_kit_export(out_dir, spec, coherence)

    t1 = time.time()

    print(f"[4/4] Curation complete")
    print()
    print(f"{'='*60}")
    print(f"Kit:      {out_dir}")
    print(f"Kept:     {kept_count}/{generated} sounds ({kept_count/generated*100:.0f}%)")
    print(f"Time:     {t1-t0:.1f}s")
    print(f"Coherence:{coherence.get('overall_coherence', 0):.3f}")
    print(f"Manifest: {out_dir/'manifest.json'}")
    print()
    print(f"Listen:   cshot listen {out_dir}")
