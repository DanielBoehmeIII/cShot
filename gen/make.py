"""One-command producer mode: scan, generate, rank, export, report."""
import json
import sys
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.pack import cmd_pack
from gen.rank import score_file
from gen.rating import _load_ratings
from gen.io import read_wav, write_wav
from gen.polish import polish_file


def cmd_make(args):
    """One-command producer mode: generate a polished, ranked pack."""
    prompt = " ".join(args.prompt) if isinstance(args.prompt, list) else args.prompt
    out_dir = Path(args.out) if args.out else Path(f"Packs/{prompt.replace(' ', '_')}")
    count = args.count

    print(f"cShot Make — One-Command Producer Mode")
    print(f"{'='*60}")
    print(f"Prompt: '{prompt}'")
    print(f"Target: {count} files → {out_dir}")
    print()

    # Step 1: Generate pack
    print("[1/4] Generating pack...")
    pack_args = type("Args", (), {
        "prompt": args.prompt,
        "count": count,
        "out": str(out_dir),
    })()
    cmd_pack(pack_args)

    # Step 2: Polish all files
    print("\n[2/4] Polishing files...")
    wav_files = sorted(out_dir.rglob("*.wav"))
    for w in wav_files:
        validation = polish_file(w, target_db=-1.0, in_place=True)

    # Step 3: Rank
    print("\n[3/4] Ranking files...")
    ratings = _load_ratings()
    scored = []
    for w in wav_files:
        s = score_file(w, ratings)
        scored.append(s)

    scored.sort(key=lambda x: x["score"], reverse=True)

    rank_path = out_dir / "rank_report.json"
    with open(rank_path, "w") as f:
        json.dump({"total_files": len(scored), "rankings": scored}, f, indent=2)

    # Step 4: Export top results
    print("\n[4/4] Exporting top results...")
    top_dir = out_dir / "_top"
    top_dir.mkdir(exist_ok=True)

    top_n = min(20, len(scored))
    top_results = []
    for i in range(top_n):
        s = scored[i]
        src = Path(s["path"])
        dest = top_dir / f"{i+1:03d}_{src.name}"
        import shutil
        shutil.copy2(src, dest)
        top_results.append({
            "rank": i + 1,
            "file": dest.name,
            "score": s["score"],
            "category": src.parent.name,
        })

    # Write summary report
    summary = {
        "prompt": prompt,
        "total_generated": len(wav_files),
        "top_n": top_n,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "pass_rate": f"{sum(1 for s in scored if s['pass'])}/{len(scored)}",
        "top_results": top_results,
        "rank_report": str(rank_path),
    }

    summary_path = out_dir / "summary.json"
    with open(summary_path, "w") as f:
        json.dump(summary, f, indent=2)

    print(f"\n{'='*60}")
    print(f"Done: {len(wav_files)} files in {out_dir}")
    print(f"  Polish:  in-place")
    print(f"  Ranked:  {len(scored)} files")
    print(f"  Top {top_n}: {top_dir}")
    print(f"  Summary: {summary_path}")
    print(f"  Rank report: {rank_path}")
