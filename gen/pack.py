"""Batch pack generation, themes, and audit."""
import json
import random
import sys
import time
from collections import Counter
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.prompt import parse_prompt, generate_from_prompt, _seed_from_prompt, _resolve_generator, _generate_variation, _write_metadata
from gen.polish import polish_file, validate_audio
from gen.io import read_wav, write_wav
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


# Theme-based category + prompt plans for sample packs
THEME_PLANS = {
    "noir piano kit": {
        "label": "Noir Piano Kit",
        "adjectives": ["dark", "soft", "warm", "mellow", "dusty", "vintage", "intimate"],
        "categories": {
            "keys": {"profile": "piano", "count": 15, "variations": ["acoustic", "felt", "dark", "lo-fi"]},
            "fx": {"profile": "fx", "count": 10, "variations": ["impact", "riser", "air", "vinyl"]},
            "bass": {"profile": "bass", "count": 10, "variations": ["808", "sub", "reese"]},
            "drums": {"profile": "drums", "count": 15, "variations": ["kick", "snare", "clap", "hat"]},
        },
    },
    "trap god kit": {
        "label": "Trap God Kit",
        "adjectives": ["aggressive", "punchy", "dark", "distorted", "edgy", "hard"],
        "categories": {
            "drums": {"profile": "drums", "count": 20, "variations": ["kick", "snare", "clap", "hat"]},
            "bass": {"profile": "bass", "count": 15, "variations": ["808", "reese", "distorted", "fm"]},
            "synth": {"profile": "synth", "count": 10, "variations": ["stab", "pluck", "lead"]},
            "fx": {"profile": "fx", "count": 5, "variations": ["impact", "riser", "glitch"]},
        },
    },
    "cinematic impacts": {
        "label": "Cinematic Impacts",
        "adjectives": ["big", "huge", "airy", "sustained", "bright", "wide", "aggressive"],
        "categories": {
            "fx": {"profile": "fx", "count": 25, "variations": ["impact", "riser", "downlifter", "air", "noise_hit"]},
            "keys": {"profile": "keys", "count": 10, "variations": ["acoustic", "bell", "rhodes"]},
            "synth": {"profile": "synth", "count": 10, "variations": ["pad", "lead", "chord"]},
            "drums": {"profile": "drums", "count": 5, "variations": ["kick", "snare"]},
        },
    },
    "hyperpop synth pack": {
        "label": "Hyperpop Synth Pack",
        "adjectives": ["bright", "glossy", "distorted", "wide", "crisp", "edgy", "digital", "aggressive"],
        "categories": {
            "synth": {"profile": "synth", "count": 25, "variations": ["stab", "pluck", "lead", "pad", "chord"]},
            "bass": {"profile": "bass", "count": 10, "variations": ["808", "distorted", "fm"]},
            "drums": {"profile": "drums", "count": 10, "variations": ["kick", "snare", "clap", "hat"]},
            "fx": {"profile": "fx", "count": 5, "variations": ["glitch", "noise_hit", "impact"]},
        },
    },
    "lo-fi keys pack": {
        "label": "Lo-fi Keys Pack",
        "adjectives": ["lo_fi", "warm", "dusty", "mellow", "soft", "vintage", "intimate"],
        "categories": {
            "keys": {"profile": "keys", "count": 25, "variations": ["acoustic", "felt", "bell", "rhodes", "dark"]},
            "drums": {"profile": "drums", "count": 10, "variations": ["kick", "snare", "clap", "hat"]},
            "bass": {"profile": "bass", "count": 10, "variations": ["808", "sub", "reese"]},
            "fx": {"profile": "fx", "count": 5, "variations": ["vinyl", "air", "noise_hit"]},
        },
    },
}

FAMILY_PROFILE_MAP = {
    "keys": ("piano-gen", "piano"),
    "drums": ("batch", "kick"),
    "bass": ("bass-gen", "bass"),
    "synth": ("synth-gen", "synth"),
    "fx": ("fx-gen", "fx"),
}


def cmd_theme(args):
    """Generate a themed pack with coherent naming and category planning."""
    theme_name = " ".join(args.theme).lower().strip()
    if theme_name not in THEME_PLANS:
        print(f"Unknown theme '{theme_name}'. Available themes:")
        for t in sorted(THEME_PLANS.keys()):
            print(f"  {t}")
        sys.exit(1)

    theme = THEME_PLANS[theme_name]
    out_dir = Path(args.out) if args.out else Path(f"Packs/{theme_name.replace(' ', '_')}")
    out_dir.mkdir(parents=True, exist_ok=True)

    total_planned = sum(c["count"] for c in theme["categories"].values())

    print(f"Theme: {theme['label']}")
    print(f"{'='*60}")
    for cat_name, cat_info in sorted(theme["categories"].items()):
        print(f"  {cat_name:10s}: {cat_info['count']:>3} files ({', '.join(cat_info['variations'])})")
    print(f"\n  Total: {total_planned} files")
    print()

    theme_plan = {
        "theme": theme_name,
        "label": theme["label"],
        "adjectives": theme["adjectives"],
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_planned": total_planned,
        "categories": {},
    }

    total_generated = 0
    for cat_name, cat_info in sorted(theme["categories"].items()):
        cat_dir = out_dir / cat_name
        cat_dir.mkdir(parents=True, exist_ok=True)

        count = cat_info["count"]
        variations = cat_info["variations"]
        per_var = max(1, count // len(variations))

        gen_family, profile_noun = FAMILY_PROFILE_MAP.get(cat_name, ("synth-gen", "synth"))
        cat_paths = []

        for var_name in variations:
            adj = random.choice(theme["adjectives"])
            prompt = f"{adj} {var_name}"
            parsed = parse_prompt(prompt)

            for i in range(per_var):
                seed = _seed_from_prompt(f"{theme_name}_{cat_name}_{var_name}_{i}", i)
                np.random.seed(seed % 2**32)

                try:
                    gen_fn, default_dur, default_pitch, gen_family_name, profile_name, overrides = _resolve_generator(parsed)
                    dur = default_dur
                    pitch = default_pitch
                    samples, actual_dur, actual_pitch = _generate_variation(dur, pitch, gen_family_name, gen_fn)
                    safe_theme = theme_name.replace(" ", "_").lower()[:20]
                    out_path = cat_dir / f"{safe_theme}_{var_name}_{adj}_{i+1:03d}.wav"
                    write_wav(out_path, samples)
                    _write_metadata(out_path, parsed, seed, actual_dur, actual_pitch)
                    cat_paths.append(out_path)
                except Exception as e:
                    print(f"  ✗ {cat_name}/{var_name}: {e}")

        total_generated += len(cat_paths)
        theme_plan["categories"][cat_name] = {
            "count": len(cat_paths),
            "variations": variations,
            "files": [p.name for p in cat_paths],
        }
        print(f"  ✓ {cat_name:10s}: {len(cat_paths)} files → {cat_dir}")

    theme_plan["total_generated"] = total_generated

    plan_path = out_dir / "theme_plan.json"
    with open(plan_path, "w") as f:
        json.dump(theme_plan, f, indent=2)
    print(f"\nTheme plan: {plan_path}")
    print(f"Generated {total_generated}/{total_planned} files in {out_dir}")
