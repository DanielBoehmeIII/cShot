"""Dataset health checks: bad files, clipping, silence, duplicates, mislabeled folders."""

import json
import sys
import time
from pathlib import Path
from typing import Optional

import numpy as np

from gen import REPO_ROOT, SUPPORTED_EXTS
from gen.io import read_audio_safe
from gen.features import compute_features, compute_rms, compute_peak
from gen.scanning import find_reference_folders, classify_reference_file

CLIP_THRESHOLD = 0.99
SILENCE_RMS_THRESHOLD = 0.01
SILENCE_PEAK_THRESHOLD = 0.001
SILENCE_DURATION_MS_MIN = 50.0
DUPLICATE_DISTANCE_THRESHOLD = 0.05


def _check_for_clipping(samples: np.ndarray, sr: int) -> dict:
    """Check if file has excessive clipping."""
    peak = float(np.max(np.abs(samples)))
    clipped_count = int(np.sum(np.abs(samples) >= CLIP_THRESHOLD))
    total = len(samples)
    clipped_ratio = clipped_count / max(total, 1)
    return {
        "peak": peak,
        "clipped_samples": clipped_count,
        "clipped_ratio": clipped_ratio,
        "is_clipped": clipped_ratio > 0.01 or peak >= 1.0,
    }


def _check_for_silence(samples: np.ndarray, sr: int) -> dict:
    """Check if file is silence or near-silence."""
    rms = compute_rms(samples)
    peak = compute_peak(samples)
    duration_ms = len(samples) / sr * 1000.0
    return {
        "rms": rms,
        "peak": peak,
        "duration_ms": duration_ms,
        "is_silence": rms < SILENCE_RMS_THRESHOLD or peak < SILENCE_PEAK_THRESHOLD,
        "is_too_short": duration_ms < SILENCE_DURATION_MS_MIN,
    }


def _expected_class_from_path(path: Path) -> Optional[str]:
    """Infer the expected class from directory structure rather than filename."""
    parent = path.parent.name.lower()
    grandparent = path.parent.parent.name.lower() if path.parent.parent else ""
    name = path.stem.lower()

    # Check specific keywords in paths
    if "kick" in parent or "kick" in grandparent:
        return "kick"
    if "snare" in parent or "snare" in grandparent:
        return "snare"
    if "clap" in parent or "clap" in grandparent:
        return "clap"
    if "hi-hat" in parent or "hi-hat" in grandparent:
        if "open" in parent or "open" in grandparent:
            return "open_hat"
        return "closed_hat"
    if "hat" in parent or "hat" in grandparent:
        if "open" in parent or "open" in grandparent:
            return "open_hat"
        return "closed_hat"
    if "808" in parent or "sub" in parent:
        return "808"
    if "bass" in parent:
        return "bass_stab"
    if "fx" in parent or "impact" in parent:
        return "impact_fx"
    if "synth" in parent or "keys" in parent or "keyboard" in parent:
        return "synth_stab"
    if "guitar" in parent or "guitar" in grandparent:
        return "guitar_stab"
    if "perc" in parent or "rim" in parent or "tom" in parent:
        return "impact_fx"
    if "cymbal" in parent or "crash" in parent:
        return "open_hat"
    if "piano" in parent or "rhodes" in parent or "ep" in parent or "wurly" in parent:
        return "synth_stab"

    return None


def _check_mislabeled(analysis: dict) -> list[dict]:
    """Find files where classified class doesn't match path-inferred class."""
    mislabeled = []
    for rel_path, feats in analysis.get("files", {}).items():
        file_path = REPO_ROOT / rel_path
        classified = feats.get("class", "unknown")
        expected = _expected_class_from_path(file_path)
        if expected and classified != "unknown" and classified != expected:
            # Only flag if the directory clearly indicates a DIFFERENT class
            # Ignore "unknown" classification
            if classified == "unknown":
                continue
            # Check if the parent directory has the classified class name — if so, it's correctly placed
            parent_lower = str(file_path.parent).lower()
            if classified.replace("_", " ") in parent_lower or classified.replace("_", "-") in parent_lower:
                continue
            mislabeled.append({
                "file": rel_path,
                "classified_as": classified,
                "expected_from_path": expected,
            })

    return mislabeled


def _find_near_duplicates(analysis: dict) -> list[dict]:
    """Find files with nearly identical feature vectors within the same class."""
    files_by_class: dict[str, list[tuple[str, dict]]] = {}
    for rel_path, feats in analysis.get("files", {}).items():
        cls = feats.get("class", "unknown")
        if cls not in files_by_class:
            files_by_class[cls] = []
        files_by_class[cls].append((rel_path, feats))

    compare_keys = ["duration_ms", "rms", "spectral_centroid", "zero_crossing_rate",
                    "low_band_energy", "high_band_energy", "transient_count",
                    "decay_length_ms"]
    duplicates = []
    seen_pairs = set()

    for cls, items in files_by_class.items():
        n = len(items)
        for i in range(n):
            for j in range(i + 1, n):
                path_a, feats_a = items[i]
                path_b, feats_b = items[j]
                pair_key = tuple(sorted([path_a, path_b]))
                if pair_key in seen_pairs:
                    continue
                seen_pairs.add(pair_key)

                dist = 0.0
                count = 0
                for k in compare_keys:
                    if k in feats_a and k in feats_b:
                        va = feats_a[k]
                        vb = feats_b[k]
                        denom = max(abs(va), abs(vb), 1e-6)
                        dist += ((va - vb) / denom) ** 2
                        count += 1
                if count > 0:
                    dist = np.sqrt(dist / count)
                else:
                    dist = 999.0

                if dist < DUPLICATE_DISTANCE_THRESHOLD:
                    duplicates.append({
                        "class": cls,
                        "file_a": path_a,
                        "file_b": path_b,
                        "distance": float(dist),
                    })

    return duplicates


def run_health_checks(analysis: dict, all_audio_paths: Optional[list[Path]] = None) -> dict:
    """Run all dataset health checks on reference analysis data."""

    report = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "summary": {},
        "bad_files": [],
        "clipping": [],
        "silence": [],
        "duplicates": [],
        "mislabeled": [],
        "class_health": {},
    }

    # 1. Bad files — files that errored during read
    errors = analysis.get("errors", 0)
    total = analysis.get("total_files", 0)
    analyzed = analysis.get("analyzed", 0)
    skipped = analysis.get("skipped_format", 0)

    bad_files = []
    if all_audio_paths:
        for path in all_audio_paths:
            if path.suffix.lower() not in SUPPORTED_EXTS:
                continue
            result = read_audio_safe(path)
            if result is None:
                rel = str(path.relative_to(REPO_ROOT))
                bad_files.append({
                    "file": rel,
                    "reason": "read_error",
                    "class": classify_reference_file(path) or "unknown",
                })
    # Fall back to tracking from the analysis metadata
    bad_file_entries = bad_files if bad_files else []

    # 2. Clipping & silence — re-check analyzed files
    clipping_issues = []
    silence_issues = []
    class_stats = {}

    for rel_path, feats in analysis.get("files", {}).items():
        cls = feats.get("class", "unknown")
        file_path = REPO_ROOT / rel_path
        result = read_audio_safe(file_path)
        if result is None:
            continue
        samples, sr = result

        clip = _check_for_clipping(samples, sr)
        if clip["is_clipped"]:
            clipping_issues.append({
                "file": rel_path,
                "class": cls,
                "peak": clip["peak"],
                "clipped_ratio": clip["clipped_ratio"],
                "clipped_samples": clip["clipped_samples"],
            })

        silent = _check_for_silence(samples, sr)
        if silent["is_silence"]:
            silence_issues.append({
                "file": rel_path,
                "class": cls,
                "rms": silent["rms"],
                "peak": silent["peak"],
                "duration_ms": silent["duration_ms"],
            })
        elif silent["is_too_short"]:
            silence_issues.append({
                "file": rel_path,
                "class": cls,
                "reason": "too_short",
                "duration_ms": silent["duration_ms"],
            })

        # Per-class stats
        if cls not in class_stats:
            class_stats[cls] = {"count": 0, "clipped": 0, "silent": 0, "too_short": 0}
        class_stats[cls]["count"] += 1
        if clip["is_clipped"]:
            class_stats[cls]["clipped"] += 1
        if silent["is_silence"]:
            class_stats[cls]["silent"] += 1
        if silent["is_too_short"]:
            class_stats[cls]["too_short"] += 1

    # 3. Duplicates
    duplicates = _find_near_duplicates(analysis)

    # 4. Mislabeled
    mislabeled = _check_mislabeled(analysis)

    # Assemble report
    report["bad_files"] = bad_file_entries
    report["clipping"] = clipping_issues
    report["silence"] = silence_issues
    report["duplicates"] = duplicates
    report["mislabeled"] = mislabeled
    report["class_health"] = class_stats

    report["summary"] = {
        "total_files": total,
        "analyzed": analyzed,
        "errors": errors,
        "skipped_format": skipped,
        "bad_files_count": len(bad_file_entries) or errors,
        "clipping_count": len(clipping_issues),
        "silence_count": len(silence_issues),
        "duplicate_groups_count": len(duplicates),
        "mislabeled_count": len(mislabeled),
        "total_issues": (errors + len(clipping_issues) + len(silence_issues)
                         + len(duplicates) + len(mislabeled)),
    }

    return report


def print_health_report(report: dict):
    """Print human-readable health report to stdout."""
    s = report["summary"]

    print("=" * 60)
    print("  DATASET HEALTH REPORT")
    print("=" * 60)
    print(f"\n  Files: {s['total_files']} total, {s['analyzed']} analyzed")
    print(f"  Errors: {s['errors']} ({s['skipped_format']} format-skipped)")

    print(f"\n  {'ISSUE':<30} {'COUNT':>8}")
    print(f"  {'-'*30} {'-'*8}")
    print(f"  {'Bad files (read errors)':<30} {s['bad_files_count']:>8}")
    print(f"  {'Clipping':<30} {s['clipping_count']:>8}")
    print(f"  {'Silence / too short':<30} {s['silence_count']:>8}")
    print(f"  {'Near-duplicates':<30} {s['duplicate_groups_count']:>8}")
    print(f"  {'Mislabeled':<30} {s['mislabeled_count']:>8}")
    print(f"  {'─'*30} {'─'*8}")
    print(f"  {'TOTAL ISSUES':<30} {s['total_issues']:>8}")

    if report["clipping"]:
        print(f"\n  Clipping details (top 10):")
        for c in report["clipping"][:10]:
            print(f"    {c['file'][:70]:70s} peak={c['peak']:.3f} ratio={c['clipped_ratio']:.4f}")

    if report["silence"]:
        print(f"\n  Silence details (top 10):")
        for s_ in report["silence"][:10]:
            if "rms" in s_:
                print(f"    {s_['file'][:70]:70s} rms={s_['rms']:.4f} peak={s_['peak']:.4f}")
            else:
                print(f"    {s_['file'][:70]:70s} too_short dur={s_['duration_ms']:.0f}ms")

    if report["duplicates"]:
        print(f"\n  Near-duplicates (top 10):")
        for d in report["duplicates"][:10]:
            print(f"    [{d['class']:12s}] dist={d['distance']:.4f}")
            print(f"      A: {d['file_a'][:70]}")
            print(f"      B: {d['file_b'][:70]}")

    if report["mislabeled"]:
        print(f"\n  Mislabeled files (top 10):")
        for m in report["mislabeled"][:10]:
            print(f"    Classified as {m['classified_as']:12s} but path suggests {m['expected_from_path']:12s}")
            print(f"      {m['file'][:70]}")

    if report["class_health"]:
        print(f"\n  Per-class health:")
        print(f"  {'Class':<16} {'Count':>6} {'Clip':>6} {'Silent':>6} {'Short':>6}")
        print(f"  {'-'*16} {'-'*6} {'-'*6} {'-'*6} {'-'*6}")
        for cls in sorted(report["class_health"].keys()):
            ch = report["class_health"][cls]
            clip_pct = ch["clipped"] / max(ch["count"], 1) * 100
            silent_pct = ch["silent"] / max(ch["count"], 1) * 100
            short_pct = ch["too_short"] / max(ch["count"], 1) * 100
            print(f"  {cls:<16} {ch['count']:>6} {ch['clipped']:>6}({clip_pct:>4.0f}%) {ch['silent']:>6} {ch['too_short']:>6}")

    print(f"\n  Overall health: ",
          end="")
    if s["total_issues"] == 0:
        print("CLEAN")
    elif s["total_issues"] < 10:
        print("GOOD (minor issues)")
    elif s["total_issues"] < 100:
        print("FAIR (issues exist)")
    elif s["total_issues"] < 1000:
        print("POOR (significant cleanup needed)")
    else:
        print("CRITICAL (majority of files have issues)")

    print()


def build_health_markdown(report: dict) -> list[str]:
    """Build Markdown report for dataset health."""
    lines = []
    lines.append("# Dataset Health Report")
    lines.append("")
    lines.append(f"**Generated:** {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}")
    lines.append("")

    s = report["summary"]
    lines.append("## Summary")
    lines.append("")
    lines.append(f"| Metric | Value |")
    lines.append(f"|--------|-------|")
    lines.append(f"| Total files | {s['total_files']} |")
    lines.append(f"| Analyzed | {s['analyzed']} |")
    lines.append(f"| Errors | {s['errors']} |")
    lines.append(f"| Bad files | {s['bad_files_count']} |")
    lines.append(f"| Clipping | {s['clipping_count']} |")
    lines.append(f"| Silence | {s['silence_count']} |")
    lines.append(f"| Near-duplicates | {s['duplicate_groups_count']} |")
    lines.append(f"| Mislabeled | {s['mislabeled_count']} |")
    lines.append(f"| **Total issues** | **{s['total_issues']}** |")
    lines.append("")

    if report["clipping"]:
        lines.append("## Clipping Issues")
        lines.append("")
        lines.append("| File | Class | Peak | Clipped Ratio |")
        lines.append("|------|-------|------|--------------|")
        for c in report["clipping"][:20]:
            lines.append(f"| {c['file']} | {c['class']} | {c['peak']:.3f} | {c['clipped_ratio']:.4f} |")
        if len(report["clipping"]) > 20:
            lines.append(f"| ... and {len(report['clipping']) - 20} more |")
        lines.append("")

    if report["silence"]:
        lines.append("## Silence / Too-Short Files")
        lines.append("")
        lines.append("| File | Class | RMS | Peak | Duration |")
        lines.append("|------|-------|-----|------|----------|")
        for s_ in report["silence"][:20]:
            rms = s_.get("rms", "-")
            peak = s_.get("peak", "-")
            dur = s_.get("duration_ms", 0)
            lines.append(f"| {s_['file']} | {s_['class']} | {rms} | {peak} | {dur:.0f}ms |")
        if len(report["silence"]) > 20:
            lines.append(f"| ... and {len(report['silence']) - 20} more |")
        lines.append("")

    if report["duplicates"]:
        lines.append("## Near-Duplicates")
        lines.append("")
        lines.append("| Class | Distance | File A | File B |")
        lines.append("|-------|----------|--------|--------|")
        for d in report["duplicates"][:20]:
            lines.append(f"| {d['class']} | {d['distance']:.4f} | {d['file_a']} | {d['file_b']} |")
        if len(report["duplicates"]) > 20:
            lines.append(f"| ... and {len(report['duplicates']) - 20} more |")
        lines.append("")

    if report["mislabeled"]:
        lines.append("## Mislabeled Files")
        lines.append("")
        lines.append("| File | Classified As | Expected From Path |")
        lines.append("|------|---------------|-------------------|")
        for m in report["mislabeled"][:20]:
            lines.append(f"| {m['file']} | {m['classified_as']} | {m['expected_from_path']} |")
        if len(report["mislabeled"]) > 20:
            lines.append(f"| ... and {len(report['mislabeled']) - 20} more |")
        lines.append("")

    if report["class_health"]:
        lines.append("## Per-Class Health")
        lines.append("")
        lines.append("| Class | Count | Clipped | Silent | Too Short |")
        lines.append("|-------|-------|---------|--------|-----------|")
        for cls in sorted(report["class_health"].keys()):
            ch = report["class_health"][cls]
            lines.append(f"| {cls} | {ch['count']} | {ch['clipped']} | {ch['silent']} | {ch['too_short']} |")
        lines.append("")

    lines.append("---")
    lines.append("*Generated by cShot Dataset Health*")
    return lines


def cmd_dataset_health(args):
    """Run dataset health checks on reference folders."""

    use_existing = getattr(args, 'analysis', None) or getattr(args, 'input_dir', None)

    if use_existing and Path(use_existing).exists():
        with open(Path(use_existing)) as f:
            analysis = json.load(f)
        print(f"Loaded existing analysis from {use_existing}")
        all_paths = None
    else:
        # Run scan first
        from gen.scanning import cmd_scan
        scan_args = type('Args', (), {"output": None})()
        cmd_scan(scan_args)
        analysis_path = REPO_ROOT / "reference_analysis.json"
        with open(analysis_path) as f:
            analysis = json.load(f)
        # Collect paths
        ref_folders = find_reference_folders()
        all_paths = []
        for name, folder in ref_folders:
            for ext in SUPPORTED_EXTS:
                all_paths.extend(folder.rglob(f"*{ext}"))
                all_paths.extend(folder.rglob(f"*{ext.upper()}"))
        all_paths = sorted(set(all_paths))

    print("Running dataset health checks...")
    report = run_health_checks(analysis, all_paths)
    print_health_report(report)

    fmt = args.format
    out_path = REPO_ROOT / "dataset_health.json"
    if args.output:
        out_path = Path(args.output)
    elif fmt == "markdown":
        out_path = REPO_ROOT / "dataset_health.md"

    if fmt == "markdown":
        lines = build_health_markdown(report)
        out_path.write_text("\n".join(lines))
    else:
        out_path.write_text(json.dumps(report, indent=2))
    print(f"Health report written to {out_path}")

    return report
