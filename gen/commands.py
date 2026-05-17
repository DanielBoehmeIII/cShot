import argparse
import json
import math
import random
import sys
import time
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.features import compute_features, compute_rms, compute_peak
from gen.io import write_wav
from gen.scanning import load_profiles, classify_reference_file
from gen.synthesis import SYNTHESIS_CLASSES
from gen.refinement import feature_distance


def cmd_oneshot(args):
    """Generate one-shots for a class."""
    class_name = args.class_name

    if class_name not in SYNTHESIS_CLASSES and class_name != "all":
        print(f"Unknown class: {class_name}")
        print(f"Valid classes: {', '.join(sorted(SYNTHESIS_CLASSES.keys()))}, all")
        sys.exit(1)

    if args.out:
        single_out = Path(args.out)
        single_out.parent.mkdir(parents=True, exist_ok=True)
        cls = class_name

        if cls not in SYNTHESIS_CLASSES:
            print(f"Error: --out mode requires a single class, not 'all'", file=sys.stderr)
            sys.exit(1)

        label, synth_fn, default_dur, default_pitch = SYNTHESIS_CLASSES[cls]

        profiles = load_profiles() if Path(args.profiles or REPO_ROOT / "class_profiles.json").exists() else None

        seed = int(time.time() * 1000) % 1000000
        random.seed(seed)
        np.random.seed(seed % 2**32)
        dur_var = default_dur * (1.0 + (random.random() - 0.5) * 0.3)
        pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * 0.2)
        samples = synth_fn(dur_var, pitch_var, profiles)
        gain = 0.85 + random.random() * 0.15
        samples = samples * gain
        write_wav(single_out, samples)
        print(f"Generated {cls} → {single_out}")
        return

    count = args.count
    profiles = load_profiles() if Path(args.profiles or REPO_ROOT / "class_profiles.json").exists() else None

    output_dir = Path(args.output_dir) if args.output_dir else REPO_ROOT / "generated_audit"
    output_dir.mkdir(parents=True, exist_ok=True)

    seed_offset = int(time.time() * 1000) % 1000000

    classes_to_generate = list(SYNTHESIS_CLASSES.keys()) if class_name == "all" else [class_name]

    for cls in classes_to_generate:
        class_output_dir = output_dir / cls
        class_output_dir.mkdir(parents=True, exist_ok=True)

        label, synth_fn, default_dur, default_pitch = SYNTHESIS_CLASSES[cls]

        print(f"Generating {count} {label}...")

        for i in range(count):
            seed = (seed_offset + i) * 314159265 + hash(cls) % 1000000
            random.seed(seed)
            np.random.seed(seed % 2**32)

            dur_var = default_dur * (1.0 + (random.random() - 0.5) * 0.3)
            pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * 0.2)

            samples = synth_fn(dur_var, pitch_var, profiles)

            gain = 0.85 + random.random() * 0.15
            samples = samples * gain

            out_path = class_output_dir / f"{cls}_{i+1:03d}.wav"
            write_wav(out_path, samples)

            if i == 0 or (i + 1) % 5 == 0:
                print(f"  [{i+1}/{count}] {out_path.name}")

        print(f"  Done: {count} {label} files → {class_output_dir}\n")


def cmd_batch(args):
    """Batch generate many samples of a class into a directory."""
    cls = args.class_name
    count = args.count
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    if cls not in SYNTHESIS_CLASSES:
        print(f"Unknown class: {cls}")
        print(f"Valid classes: {', '.join(sorted(SYNTHESIS_CLASSES.keys()))}")
        sys.exit(1)

    profiles = load_profiles() if Path(args.profiles or REPO_ROOT / "class_profiles.json").exists() else None

    label, synth_fn, default_dur, default_pitch = SYNTHESIS_CLASSES[cls]
    seed_offset = int(time.time() * 1000) % 1000000

    print(f"Generating {count} {label} → {out_dir}...")

    for i in range(count):
        seed = (seed_offset + i) * 314159265 + hash(cls) % 1000000
        random.seed(seed)
        np.random.seed(seed % 2**32)

        dur_var = default_dur * (1.0 + (random.random() - 0.5) * 0.3)
        pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * 0.2)

        samples = synth_fn(dur_var, pitch_var, profiles)
        gain = 0.85 + random.random() * 0.15
        samples = samples * gain

        out_path = out_dir / f"{cls}_{i+1:03d}.wav"
        write_wav(out_path, samples)

        if i == 0 or (i + 1) % 5 == 0:
            print(f"  [{i+1}/{count}] {out_path.name}")

    print(f"  Done: {count} {label} → {out_dir}")


def cmd_qa(args):
    """Generate QA audit: all classes x 10, manifest + feature report."""
    output_dir = Path(args.output_dir) if args.output_dir else REPO_ROOT / "generated_audit"
    output_dir.mkdir(parents=True, exist_ok=True)

    samples_per_class = args.samples if args.samples else 10

    profiles = load_profiles() if Path(args.profiles or REPO_ROOT / "class_profiles.json").exists() else None

    print(f"cShot QA Audit")
    print(f"==============")
    print(f"Output: {output_dir}")
    print(f"Samples per class: {samples_per_class}\n")

    manifest_entries = []
    feature_report = {}
    total_generated = 0
    total_passed = 0
    total_failed = 0

    seed_offset = int(time.time() * 1000) % 1000000

    for cls in sorted(SYNTHESIS_CLASSES.keys()):
        label, synth_fn, default_dur, default_pitch = SYNTHESIS_CLASSES[cls]
        class_dir = output_dir / cls
        class_dir.mkdir(parents=True, exist_ok=True)

        class_passed = 0
        class_failed = 0

        for i in range(samples_per_class):
            seed = (seed_offset + i) * 314159265 + hash(cls) % 1000000
            random.seed(seed)
            np.random.seed(seed % 2**32)

            dur_var = default_dur * (1.0 + (random.random() - 0.5) * 0.3)
            pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * 0.2)

            samples = synth_fn(dur_var, pitch_var, profiles)

            gain = 0.85 + random.random() * 0.15
            samples = samples * gain

            total_generated += 1
            issues = []

            if len(samples) == 0:
                issues.append("silent")
            peak = compute_peak(samples)
            rms = compute_rms(samples)
            if peak > 0.99:
                issues.append("clipped")
            if peak < 0.001:
                issues.append("too_quiet")
            if np.any(np.isnan(samples)) or np.any(np.isinf(samples)):
                issues.append("nan")

            status = "pass" if len(issues) == 0 else ("warn" if len(issues) <= 1 else "fail")

            if status != "fail":
                out_path = class_dir / f"{cls}_{i+1:03d}.wav"
                write_wav(out_path, samples)
                class_passed += 1
            else:
                class_failed += 1

            feats = compute_features(samples)

            entry = {
                "class": cls,
                "index": i + 1,
                "seed": seed,
                "status": status,
                "issues": issues,
                "path": str(f"{cls}/{cls}_{i+1:03d}.wav"),
                "duration_ms": feats["duration_ms"],
                "peak": feats["peak"],
                "rms": feats["rms"],
                "spectral_centroid": feats["spectral_centroid"],
                "zero_crossing_rate": feats["zero_crossing_rate"],
                "low_band_energy": feats["low_band_energy"],
                "mid_band_energy": feats["mid_band_energy"],
                "high_band_energy": feats["high_band_energy"],
                "transient_count": feats["transient_count"],
                "amplitude_peaks": feats["amplitude_peaks"],
                "decay_length_ms": feats["decay_length_ms"],
            }
            manifest_entries.append(entry)

            if cls not in feature_report:
                feature_report[cls] = []
            feature_report[cls].append(feats)

        pass_rate = (class_passed / max(class_passed + class_failed, 1)) * 100
        print(f"  {label:12} {class_passed:>3}/{class_passed + class_failed:>3} passed ({pass_rate:>5.1f}%)")
        total_passed += class_passed
        total_failed += class_failed

    manifest = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "cshot_version": "1.0.0-py",
        "engine": "cshot-recovery-synthesis",
        "total_sounds": total_generated,
        "summary": {"total": total_generated, "passed": total_passed, "failed": total_failed},
        "entries": manifest_entries,
    }
    manifest_path = output_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2))

    feature_summary = {}
    for cls, feats_list in feature_report.items():
        keys = ["duration_ms", "rms", "peak", "spectral_centroid", "zero_crossing_rate",
                "low_band_energy", "mid_band_energy", "high_band_energy",
                "transient_count", "amplitude_peaks", "decay_length_ms"]
        summary = {}
        for k in keys:
            vals = [f[k] for f in feats_list]
            if vals:
                summary[k] = {
                    "mean": float(np.mean(vals)),
                    "std": float(np.std(vals)),
                    "min": float(np.min(vals)),
                    "max": float(np.max(vals)),
                }
            else:
                summary[k] = {"mean": 0.0, "std": 0.0, "min": 0.0, "max": 0.0}
        feature_summary[cls] = summary

    feature_report_out = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "classes": feature_summary,
        "class_means": {cls: {k: summary[k]["mean"] for k in summary} for cls, summary in feature_summary.items()},
    }
    feat_report_path = output_dir / "feature_report.json"
    feat_report_path.write_text(json.dumps(feature_report_out, indent=2))

    print(f"\nQA Summary")
    print(f"----------")
    print(f"  Total: {total_generated}")
    print(f"  Passed: {total_passed}")
    print(f"  Failed: {total_failed}")
    pass_pct = (total_passed / max(total_generated, 1)) * 100
    print(f"  Pass rate: {pass_pct:.1f}%")
    print(f"\nManifest: {manifest_path}")
    print(f"Feature report: {feat_report_path}")

    return feature_report_out, manifest


def cmd_compare(args):
    """Compare generated vs reference features."""
    feat_report_path = Path(args.generated) if args.generated else REPO_ROOT / "generated_audit" / "feature_report.json"
    if not feat_report_path.exists():
        print(f"Error: {feat_report_path} not found. Run 'qa' first.", file=sys.stderr)
        sys.exit(1)

    with open(feat_report_path) as f:
        gen_data = json.load(f)
    gen_means = gen_data.get("class_means", {})

    analysis_path = Path(args.reference) if args.reference else REPO_ROOT / "reference_analysis.json"
    if not analysis_path.exists():
        print(f"Error: {analysis_path} not found. Run 'scan' first.", file=sys.stderr)
        sys.exit(1)

    with open(analysis_path) as f:
        ref_data = json.load(f)

    ref_by_class = {}
    for rel_path, feats in ref_data.get("files", {}).items():
        cls = feats.get("class", "unknown")
        if cls not in ref_by_class:
            ref_by_class[cls] = []
        ref_by_class[cls].append(feats)

    ref_means = {}
    for cls, feats_list in ref_by_class.items():
        keys = ["spectral_centroid", "low_band_energy", "mid_band_energy", "high_band_energy",
                "zero_crossing_rate", "rms", "duration_ms", "transient_count", "amplitude_peaks", "decay_length_ms"]
        means = {}
        for k in keys:
            vals = [f.get(k, 0) for f in feats_list]
            means[k] = float(np.mean(vals)) if vals else 0.0
        ref_means[cls] = means

    print("Audio QA Comparison")
    print("===================\n")

    results = {}

    for gen_cls in sorted(gen_means.keys()):
        print(f"\n{'='*60}")
        print(f"  {gen_cls.upper()} - Generated vs References")
        print(f"{'='*60}")

        g = gen_means[gen_cls]

        print(f"  {'Feature':<25} {'Generated':>10} {'Reference':>10} {'Match':>8}")
        print(f"  {'-'*25} {'-'*10} {'-'*10} {'-'*8}")

        for k in ["spectral_centroid", "low_band_energy", "high_band_energy",
                   "zero_crossing_rate", "transient_count", "amplitude_peaks", "decay_length_ms"]:
            gen_v = g.get(k, 0)
            if gen_cls in ref_means and k in ref_means[gen_cls]:
                ref_v = ref_means[gen_cls][k]
                ratio = gen_v / max(ref_v, 1e-6)
                match = "✓" if 0.25 <= ratio <= 4.0 else ("△" if 0.1 <= ratio <= 10 else "✗")
            else:
                ref_v = 0
                match = "?"
            print(f"  {k:<25} {gen_v:>10.3f} {ref_v:>10.3f} {match:>8}")

        print(f"\n  Distances to reference classes:")
        ref_classes = sorted([c for c in ref_means.keys() if c != "unknown"])

        gen_feats = gen_means[gen_cls]

        distances = []
        for ref_cls in ref_classes:
            ref_feats = ref_means[ref_cls]
            dist = feature_distance(gen_feats, ref_feats)
            distances.append((dist, ref_cls))

        distances.sort()

        print(f"  {'Reference Class':<20} {'Distance':>10}")
        print(f"  {'-'*20} {'-'*10}")
        for d, rc in distances:
            marker = "← SELF" if rc == gen_cls else ""
            print(f"  {rc:<20} {d:>10.4f}  {marker}")

        results[gen_cls] = {
            "feature_comparison": {k: {"generated": g.get(k, 0),
                                       "reference": ref_means.get(gen_cls, {}).get(k, 0)}
                                   for k in ["spectral_centroid", "low_band_energy", "high_band_energy",
                                             "zero_crossing_rate", "transient_count", "amplitude_peaks", "decay_length_ms"]},
            "distances": {rc: d for d, rc in distances},
            "nearest_reference": distances[0][1] if distances else None,
            "distance_to_self": next((d for d, rc in distances if rc == gen_cls), None),
        }

    print(f"\n{'='*60}")
    print(f"  AUDIO QA ASSERTIONS")
    print(f"{'='*60}")

    assertions = []

    if "clap" in results:
        clap_nearest = results["clap"].get("nearest_reference", "")
        passed = clap_nearest == "clap"
        assertions.append(("Clap nearest reference is clap", passed, f"nearest={clap_nearest}"))

    if "closed_hat" in results:
        ch_nearest = results["closed_hat"].get("nearest_reference", "")
        passed = ch_nearest in ("closed_hat", "open_hat")
        assertions.append(("Closed_hat nearest ref is closed_hat or open_hat", passed, f"nearest={ch_nearest}"))

    if "closed_hat" in gen_means and "clap" in gen_means:
        hat_cent = gen_means["closed_hat"].get("spectral_centroid", 0)
        clap_cent = gen_means["clap"].get("spectral_centroid", 0)
        passed = hat_cent > clap_cent * 1.3
        assertions.append(("Hat centroid > clap centroid * 1.3", passed, f"hat={hat_cent:.0f}, clap={clap_cent:.0f}"))

    if "clap" in gen_means:
        clap_peaks = gen_means["clap"].get("amplitude_peaks", 0)
        passed = clap_peaks >= 5
        assertions.append(("Clap has >= 5 amplitude peaks", passed, f"amplitude_peaks={clap_peaks}"))

    if "closed_hat" in gen_means:
        ch_trans = gen_means["closed_hat"].get("transient_count", 0)
        passed = ch_trans <= 2
        assertions.append(("Closed_hat has <= 2 transient peaks", passed, f"transient_count={ch_trans}"))

    if "clap" in gen_means and "closed_hat" in gen_means:
        clap_decay = gen_means["clap"].get("decay_length_ms", 0)
        ch_decay = gen_means["closed_hat"].get("decay_length_ms", 0)
        passed = ch_decay < clap_decay * 0.8
        assertions.append(("Closed_hat decay < clap decay * 0.8", passed,
                          f"hat_decay={ch_decay:.3f}ms, clap_decay={clap_decay:.3f}ms"))

    if "kick" in gen_means and "closed_hat" in gen_means:
        kick_low = gen_means["kick"].get("low_band_energy", 0)
        hat_low = gen_means["closed_hat"].get("low_band_energy", 0)
        passed = kick_low > hat_low * 1.5
        assertions.append(("Kick low-band > hat low-band * 1.5", passed, f"kick_low={kick_low:.4f}, hat_low={hat_low:.4f}"))

    if "kick" in gen_means and "clap" in gen_means:
        kick_low = gen_means["kick"].get("low_band_energy", 0)
        clap_low = gen_means["clap"].get("low_band_energy", 0)
        passed = kick_low > clap_low * 1.5
        assertions.append(("Kick low-band > clap low-band * 1.5", passed, f"kick_low={kick_low:.4f}, clap_low={clap_low:.4f}"))

    if "closed_hat" in gen_means and "snare" in gen_means:
        hat_cent = gen_means["closed_hat"].get("spectral_centroid", 0)
        snare_cent = gen_means["snare"].get("spectral_centroid", 0)
        passed = hat_cent > snare_cent * 1.2
        assertions.append(("Hat centroid > snare centroid * 1.2", passed, f"hat={hat_cent:.0f}, snare={snare_cent:.0f}"))

    all_passed = True
    for desc, passed, detail in assertions:
        icon = "✅" if passed else "❌"
        if not passed:
            all_passed = False
        print(f"  {icon} {desc}")
        print(f"     {detail}")

    print(f"\n  Overall: {'ALL PASSED' if all_passed else 'SOME FAILED'}")

    nearest_matches = {}
    for gen_cls, res in results.items():
        nearest_matches[gen_cls] = {
            "nearest_reference_class": res["nearest_reference"],
            "distance_to_self": res["distance_to_self"],
            "all_distances": res["distances"],
        }

    match_path = Path(args.output) if args.output else REPO_ROOT / "generated_audit" / "nearest_reference_matches.json"
    match_path.write_text(json.dumps(nearest_matches, indent=2))
    print(f"\nNearest reference matches: {match_path}")

    return results, assertions, all_passed


def cmd_all(args):
    """Run full pipeline: scan → profiles → qa → compare."""
    from gen.scanning import cmd_scan, cmd_profiles
    scan_args = argparse.Namespace(output=None)
    cmd_scan(scan_args)

    profiles_args = argparse.Namespace(analysis=None, output=None)
    cmd_profiles(profiles_args)

    qa_args = argparse.Namespace(output_dir=None, samples=10, profiles=None)
    cmd_qa(qa_args)

    compare_args = argparse.Namespace(generated=None, reference=None, output=None)
    cmd_compare(compare_args)

    print("\nFull pipeline complete!")
