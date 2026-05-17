"""Batch pack generation and audit."""
import json
import sys
import time
from collections import Counter
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.prompt import parse_prompt, generate_from_prompt
from gen.polish import polish_file, validate_audio
from gen.io import read_wav
from gen.features import compute_features


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


def cmd_pack_audit(args):
    """Audit an existing pack directory for quality issues."""
    pack_dir = Path(args.pack_dir)
    if not pack_dir.exists():
        print(f"Error: {pack_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = sorted(pack_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No .wav files found in {pack_dir}")
        return

    print(f"Pack Audit: {pack_dir}")
    print(f"{'='*60}")
    print(f"Files: {len(wav_files)}")
    print()

    # Check category subdirectories
    categories = {}
    for w in wav_files:
        cat = w.parent.name
        categories.setdefault(cat, []).append(w)

    print(f"Categories: {len(categories)}")
    for cat, files in sorted(categories.items()):
        print(f"  {cat:12s}: {len(files)} files")

    print(f"\n--- Quality Checks ---\n")

    results = []
    issues_found = 0
    for w in wav_files:
        cat = w.parent.name
        result = read_wav(w)
        if result is None:
            results.append({"file": str(w), "pass": False, "issues": ["unreadable"]})
            issues_found += 1
            continue
        samples, sr = result
        validation = validate_audio(samples)
        feats = compute_features(samples, sr)

        entry = {
            "file": str(w.relative_to(pack_dir)),
            "category": cat,
            "pass": validation["pass"],
            "issues": validation["issues"],
            "peak": validation["peak"],
            "rms": validation["rms"],
            "duration_s": validation["duration_s"],
            "spectral_centroid": round(float(feats.get("spectral_centroid", 0)), 1),
            "attack_ms": round(float(feats.get("attack_ms", 0)), 2),
        }
        results.append(entry)
        if not validation["pass"]:
            issues_found += 1
            print(f"  ! {entry['file']}: {'; '.join(validation['issues'])}")

    if issues_found == 0:
        print(f"  All {len(wav_files)} files pass basic QA")

    # Variation check per category
    print(f"\n--- Variation Check ---\n")
    weak_categories = []
    for cat, files in sorted(categories.items()):
        if len(files) < 2:
            continue
        centroids = []
        attacks = []
        durations = []
        for w in files:
            result = read_wav(w)
            if result is None:
                continue
            samples, sr = result
            feats = compute_features(samples, sr)
            centroids.append(feats.get("spectral_centroid", 0))
            attacks.append(feats.get("attack_ms", 0))
            durations.append(len(samples) / sr)

        if len(centroids) > 1:
            centroid_range = max(centroids) - min(centroids)
            attack_range = max(attacks) - min(attacks)
            duration_range = max(durations) - min(durations)
            weak = centroid_range < 50 or (attack_range < 1 and len(files) > 2)
            if weak:
                weak_categories.append(cat)
                print(f"  △ {cat}: low variation (centroid range={centroid_range:.0f}, "
                      f"attack range={attack_range:.2f}ms)")
            else:
                print(f"  ✓ {cat}: centroid range={centroid_range:.0f}, "
                      f"attack range={attack_range:.2f}ms, dur range={duration_range:.3f}s")

    # Summary
    print(f"\n--- Audit Summary ---")
    print(f"  Total files:    {len(wav_files)}")
    print(f"  Issues:         {issues_found}")
    print(f"  Weak variation: {len(weak_categories)} category(s)")
    overall_pass = issues_found == 0 and len(weak_categories) == 0
    print(f"  Overall:        {'PASS' if overall_pass else 'ISSUES FOUND'}")

    audit = {
        "pack_dir": str(pack_dir),
        "total_files": len(wav_files),
        "categories": {cat: len(files) for cat, files in sorted(categories.items())},
        "issues_found": issues_found,
        "weak_categories": weak_categories,
        "overall_pass": overall_pass,
        "results": results,
    }
    audit_path = pack_dir / "pack_audit.json"
    with open(audit_path, "w") as f:
        json.dump(audit, f, indent=2)
    print(f"\nAudit saved: {audit_path}")
