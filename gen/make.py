"""One-command producer mode: generate kit, quality gate, polish, rank, export."""
import json
import shutil
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.kit_spec import KitSpec, CategoryCounts, DNAProfile, infer_spec_from_prompt
from gen.kit_engine import generate_kit, compute_kit_coherence, setup_kit_export
from gen.quality_gate import run_all_gates
from gen.rank import score_file
from gen.rating import _load_ratings
from gen.polish import polish_file


def cmd_make(args):
    """One-command producer mode: generate, gate, polish, rank, export."""
    prompt = " ".join(args.prompt) if isinstance(args.prompt, list) else args.prompt
    out_dir = Path(args.out) if args.out else REPO_ROOT / "Packs" / prompt.replace(" ", "_").lower()
    count = args.count
    strategy = getattr(args, 'strategy', 'reference')

    print(f"cShot Make — One-Command Producer Mode")
    print(f"{'='*60}")
    print(f"Prompt: '{prompt}'")
    print(f"Strategy: {strategy}")
    print(f"Target: {count} files → {out_dir}")
    print()

    t0 = time.time()

    if strategy == 'reference':
        from gen.retrieval import retrieve_by_text
        from gen.reference_transform import transform_references_to_kit
        from gen.similarity_guard import validate_kit

        refs = retrieve_by_text(prompt, n=count * 2)
        print(f"[1/4] Retrieved {len(refs)} references for '{prompt}'")
        print(f"[2/4] Transforming references into kit...")
        result = transform_references_to_kit(refs, out_dir, target_count=count)
        t1 = time.time()
        print(f"  → {result['generated']}/{count} files in {t1-t0:.1f}s")
        print()

        print(f"[3/4] Running similarity guard...")
        validation = validate_kit(out_dir)
        if validation.get("total", 0) > 0:
            print(f"  → {validation['passed']}/{validation['total']} passed, {validation['failed']} failed")
            if validation['failed'] > 0:
                notes_path = out_dir / "listening_notes.md"
                notes = _generate_listening_notes(out_dir)
                notes_path.write_text(notes)
                print(f"  → Listening notes: {notes_path}")
        print()

        print(f"[4/4] Export complete")
        t2 = time.time()
        print(f"{'='*60}")
        print(f"Kit:      {out_dir}")
        print(f"Files:    {result['generated']}")
        print(f"Time:     {t2-t0:.1f}s")
        print(f"Coherence:{result.get('coherence', 0):.3f}")
        print(f"Manifest: {out_dir / 'manifest.json'}")
        print(f"Lineage:  {out_dir / 'source_lineage.json'}")
        return

    spec = infer_spec_from_prompt(prompt)
    total = sum(v for k, v in vars(spec.categories).items() if isinstance(v, int))
    if total > 0 and total != count:
        ratio = count / total
        for field in vars(spec.categories):
            current = getattr(spec.categories, field)
            if isinstance(current, int):
                setattr(spec.categories, field, max(1, int(round(current * ratio))))
    spec.total_target = count

    print(f"[1/5] Generating kit...")
    generated = generate_kit(spec, out_dir, polish=True)
    t1 = time.time()

    wav_files = sorted(out_dir.rglob("*.wav"))
    print(f"  → {generated}/{count} files in {t1-t0:.1f}s")
    print()

    print(f"[2/5] Running quality gates...")
    gate_report = run_all_gates(wav_files)
    failed_files = [Path(p) for p, r in gate_report["results"].items() if not r["pass"]]
    for w in failed_files:
        if w.exists():
            w.unlink()
    gate_stats = gate_report["stats"]
    print(f"  → {gate_stats['passed']}/{gate_stats['total']} passed, {gate_stats['failed']} removed")
    print()

    print(f"[3/5] Polishing...")
    remaining = sorted(out_dir.rglob("*.wav"))
    for w in remaining:
        polish_file(w, target_db=-1.0, in_place=True)
    print(f"  → {len(remaining)} files polished")
    print()

    print(f"[4/5] Ranking...")
    ratings = _load_ratings()
    scored = []
    for w in remaining:
        s = score_file(w, ratings)
        s["path"] = str(w.relative_to(out_dir))
        scored.append(s)
    scored.sort(key=lambda x: x.get("score", 0), reverse=True)

    rank_path = out_dir / "rank_report.json"
    with open(rank_path, "w") as f:
        json.dump({"total_files": len(scored), "rankings": scored}, f, indent=2)

    top_dir = out_dir / "_top"
    top_dir.mkdir(exist_ok=True)
    top_n = min(20, len(scored), count)
    top_results = []
    for i in range(top_n):
        s = scored[i]
        src = out_dir / s["path"]
        dest = top_dir / f"{i+1:03d}_{src.name}"
        shutil.copy2(src, dest)
        top_results.append({
            "rank": i + 1,
            "file": dest.name,
            "score": s.get("score", 0),
            "category": src.parent.name,
        })
    print(f"  → Top {top_n} exported to {top_dir}")
    print()

    coherence = compute_kit_coherence(out_dir)
    setup_kit_export(out_dir, spec, coherence)

    t2 = time.time()

    print(f"[5/5] Export complete")
    print()
    print(f"{'='*60}")
    print(f"Kit:      {out_dir}")
    print(f"Files:    {len(remaining)} (removed {gate_stats['failed']} low-quality)")
    print(f"Time:     {t2-t0:.1f}s")
    print(f"Coherence:{coherence.get('overall_coherence', 0):.3f}")
    print(f"Top 20:   {top_dir}")
    print(f"Manifest: {out_dir / 'manifest.json'}")
    print(f"README:   {out_dir / 'README.md'}")
    print()
    print(f"Listen:   cshot listen {out_dir}")
    print(f"Gate:     cshot gate {out_dir}")


def _generate_listening_notes(kit_dir: Path) -> str:
    """Generate listening notes for a reference-conditioned kit."""
    import time
    lineage = kit_dir / "source_lineage.json"
    manifest = kit_dir / "manifest.json"
    if lineage.exists():
        with open(lineage) as f:
            ldata = json.load(f)
    else:
        ldata = {"entries": []}

    wavs = sorted(kit_dir.glob("*.wav"))
    return f"""# Listening Notes

Generated: {time.strftime("%Y-%m-%d %H:%M:%S UTC", time.gmtime())}
Kit: {kit_dir.name}

## Summary
- Total WAV files: {len(wavs)}
- Source files used: {len(set(e.get("source", "") for e in ldata.get("entries", [])))}

## Quality Assessment
Each sound is a transformed variant of a professional reference sample.
Transformations include pitch shifting, EQ, saturation, transient reshaping,
time stretching, convolution reverb, and spectral morphing.

## Files
"""

