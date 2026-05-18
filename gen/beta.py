"""Beta test infrastructure: generate kits for producers, package feedback, collect results."""
import csv
import json
import shutil
import sys
import time
from pathlib import Path
from zipfile import ZipFile

from gen import REPO_ROOT
from gen.kit_spec import infer_spec_from_prompt
from gen.kit_engine import generate_kit, setup_kit_export, compute_kit_coherence
from gen.quality_gate import run_all_gates
from gen.feedback import _generate_feedback_form, _generate_rating_sheet


BETA_PRODUCERS = [
    {"name": "producer_1", "genre": "trap", "prompt": "dark trap one shot kit"},
    {"name": "producer_2", "genre": "rnb", "prompt": "smooth rnb one shot kit"},
    {"name": "producer_3", "genre": "house", "prompt": "deep house one shot kit"},
    {"name": "producer_4", "genre": "cinematic", "prompt": "cinematic impact kit"},
    {"name": "producer_5", "genre": "experimental", "prompt": "experimental texture kit"},
]


def cmd_beta_round(args):
    """Generate beta test kits for a round of producers."""
    round_num = args.round or 1
    count = args.count
    out_base = Path(args.out) if args.out else REPO_ROOT / "outputs" / "beta" / f"round_{round_num}"
    out_base.mkdir(parents=True, exist_ok=True)

    producers = BETA_PRODUCERS
    if args.producers:
        indices = [int(i.strip()) for i in args.producers.split(",") if i.strip().isdigit()]
        producers = [BETA_PRODUCERS[i-1] for i in indices if 1 <= i <= len(BETA_PRODUCERS)]

    print(f"Beta Round {round_num} — {len(producers)} producers")
    print(f"{'='*60}")
    print()

    all_packages = []

    for p in producers:
        print(f"  Producer: {p['name']} ({p['genre']})")
        kit_dir = out_base / p["name"] / "kit"
        kit_dir.mkdir(parents=True, exist_ok=True)

        spec = infer_spec_from_prompt(p["prompt"])
        total = sum(v for k, v in vars(spec.categories).items() if isinstance(v, int))
        if total > 0 and total != count:
            ratio = count / total
            for field in vars(spec.categories):
                current = getattr(spec.categories, field)
                if isinstance(current, int):
                    setattr(spec.categories, field, max(1, int(round(current * ratio))))
        spec.total_target = count

        generated = generate_kit(spec, kit_dir, polish=True)
        print(f"    Generated: {generated} files")

        wav_files = sorted(kit_dir.rglob("*.wav"))
        gate_report = run_all_gates(wav_files, dup_threshold=0.98)
        for fp, r in gate_report["results"].items():
            if not r["pass"]:
                Path(fp).unlink(missing_ok=True)
        gate_stats = gate_report["stats"]
        print(f"    After gate: {gate_stats['passed']}/{gate_stats['total']} passed")

        remaining = sorted(kit_dir.rglob("*.wav"))
        coherence = compute_kit_coherence(kit_dir)
        setup_kit_export(kit_dir, spec, coherence)

        csv_content = _generate_rating_sheet(kit_dir)
        csv_path = kit_dir.parent / f"{p['name']}_rating_sheet.csv"
        csv_path.write_text(csv_content)

        form_content = _generate_feedback_form(f"{p['genre']} Kit for {p['name']}")
        form_path = kit_dir.parent / f"{p['name']}_feedback_form.md"
        form_path.write_text(form_content)

        zip_path = kit_dir.parent / f"{p['name']}_beta_pack.zip"
        with ZipFile(zip_path, "w") as zf:
            for f in kit_dir.parent.rglob("*"):
                if f.is_file() and f.suffix != ".zip":
                    zf.write(f, f.relative_to(kit_dir.parent))
        print(f"    Packaged: {zip_path}")

        all_packages.append({
            "producer": p["name"],
            "genre": p["genre"],
            "prompt": p["prompt"],
            "files": len(remaining),
            "zip": str(zip_path),
            "coherence": coherence.get("overall_coherence", 0),
        })
        print()

    summary = {
        "round": round_num,
        "producers": all_packages,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    summary_path = out_base / "beta_round_summary.json"
    with open(summary_path, "w") as f:
        json.dump(summary, f, indent=2)

    print(f"{'='*60}")
    print(f"Beta Round {round_num} Complete")
    print(f"  Summary: {summary_path}")
    print(f"  Packages ready for distribution")
