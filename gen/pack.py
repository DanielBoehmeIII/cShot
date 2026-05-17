"""Batch pack generation: build themed packs from prompts."""
import json
import sys
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.prompt import parse_prompt, generate_from_prompt
from gen.polish import polish_file


PACK_CATEGORIES = {
    "drums": ["kick", "snare", "clap", "hat", "hihat", "open_hat"],
    "bass": ["808", "bass", "reese", "sub"],
    "keys": ["piano", "keys"],
    "synth": ["synth", "stab", "pluck", "pad", "lead", "chord"],
    "guitar": ["guitar", "nylon"],
    "fx": ["impact", "fx", "riser", "glitch", "noise", "vinyl", "texture"],
}


def _infer_category(prompt: str) -> str:
    prompt_lower = prompt.lower()
    for cat, keywords in PACK_CATEGORIES.items():
        for kw in keywords:
            if kw in prompt_lower:
                return cat
    return "other"


def cmd_pack(args):
    """Generate a themed pack from a prompt."""
    prompt = " ".join(args.prompt) if isinstance(args.prompt, list) else args.prompt
    count = args.count
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"cShot Pack Generator")
    print(f"{'='*60}")
    print(f"Theme: {prompt}")
    print(f"Target: {count} files")
    print(f"Output: {out_dir}")
    print()

    # Generate category prompts and counts
    category_prompts = {
        "drums": f"{prompt} drums",
        "bass": f"{prompt} bass",
        "keys": f"{prompt} keys",
        "synth": f"{prompt} synth",
        "guitar": f"{prompt} guitar",
        "fx": f"{prompt} fx",
    }

    per_category = max(1, count // len(category_prompts))
    total_planned = per_category * len(category_prompts)

    print(f"Plan: {per_category} files per {len(category_prompts)} categories = {total_planned} total")
    print()

    pack_manifest = {
        "theme": prompt,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_files": 0,
        "categories": {},
    }

    total_generated = 0
    for cat_name, cat_prompt in category_prompts.items():
        cat_dir = out_dir / cat_name
        cat_dir.mkdir(parents=True, exist_ok=True)

        parsed = parse_prompt(cat_prompt)
        paths = generate_from_prompt(
            parsed, per_category, cat_dir,
            name_template="{family}_{profile}_{n}"
        )

        # Polish each generated file
        polished_paths = []
        polish_results = []
        for w in paths:
            validation = polish_file(w, target_db=-1.0, in_place=True)
            polished_paths.append(w)
            polish_results.append({
                "file": w.name,
                "pass": validation["pass"],
                "peak": validation["peak"],
                "rms": validation["rms"],
                "duration_s": validation["duration_s"],
            })

        passed = sum(1 for r in polish_results if r["pass"])
        failed = len(polish_results) - passed
        cat_manifest = {
            "category": cat_name,
            "prompt": cat_prompt,
            "files": len(polished_paths),
            "passed": passed,
            "failed": failed,
            "polish_results": polish_results,
        }
        pack_manifest["categories"][cat_name] = cat_manifest
        total_generated += len(polished_paths)

        print(f"  [{cat_name:8s}] {len(polished_paths):>3} files → {cat_dir}  "
              f"({passed} pass, {failed} fail)")

    pack_manifest["total_files"] = total_generated

    # Write manifest
    manifest_path = out_dir / "pack_manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(pack_manifest, f, indent=2)
    print(f"\nManifest: {manifest_path}")

    # Write pack report
    report_path = out_dir / "pack_report.md"
    with open(report_path, "w") as f:
        f.write(f"# Pack Report: {prompt}\n\n")
        f.write(f"- Generated: {pack_manifest['generated_at']}\n")
        f.write(f"- Total files: {total_generated}\n")
        f.write(f"- Categories: {len(category_prompts)}\n\n")
        f.write("## Category Breakdown\n\n")
        f.write("| Category | Files | Pass | Fail | Prompt |\n")
        f.write("|----------|-------|------|------|--------|\n")
        for cat_name, cm in pack_manifest["categories"].items():
            f.write(f"| {cat_name} | {cm['files']} | {cm['passed']} | {cm['failed']} | `{cm['prompt']}` |\n")

        f.write("\n## Polish Details\n\n")
        for cat_name, cm in pack_manifest["categories"].items():
            f.write(f"### {cat_name}\n\n")
            f.write("| File | Pass | Peak | RMS | Duration |\n")
            f.write("|------|------|------|-----|----------|\n")
            for r in cm["polish_results"]:
                status = "PASS" if r["pass"] else "FAIL"
                f.write(f"| {r['file']} | {status} | {r['peak']:.4f} | {r['rms']:.4f} | {r['duration_s']:.3f}s |\n")
            f.write("\n")

    print(f"Report: {report_path}")
    print(f"\nDone. {total_generated} files in {out_dir}")
