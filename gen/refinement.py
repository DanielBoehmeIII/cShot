import json
import math
import random
import sys
import time
from pathlib import Path
from typing import Optional

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.features import FEATURE_KEYS_FULL, compute_features
from gen.io import read_wav, write_wav
from gen.dsp import biquad_high_shelf, biquad_low_shelf
from gen.scanning import load_profiles
from gen.synthesis import SYNTHESIS_CLASSES
from gen.reports import build_markdown_report, build_html_report

COMPARE_KEYS_FULL = [
    ("spectral_centroid", "Hz", "{:.0f}"),
    ("low_band_energy", "", "{:.4f}"),
    ("mid_band_energy", "", "{:.4f}"),
    ("high_band_energy", "", "{:.4f}"),
    ("zero_crossing_rate", "", "{:.4f}"),
    ("transient_count", "", "{:.1f}"),
    ("amplitude_peaks", "", "{:.1f}"),
    ("decay_length_ms", "ms", "{:.1f}"),
    ("attack_ms", "ms", "{:.1f}"),
    ("rms", "", "{:.4f}"),
    ("duration_ms", "ms", "{:.1f}"),
]


def feature_distance(f1: dict, f2: dict) -> float:
    """Compute weighted Euclidean distance between feature vectors."""
    weights = {
        "spectral_centroid": 0.20,
        "low_band_energy": 0.12,
        "high_band_energy": 0.12,
        "zero_crossing_rate": 0.12,
        "decay_length_ms": 0.10,
        "transient_count": 0.02,
        "amplitude_peaks": 0.06,
        "rms": 0.08,
        "duration_ms": 0.08,
        "mid_band_energy": 0.10,
    }
    dist = 0.0
    for k, w in weights.items():
        if k in f1 and k in f2:
            v1 = f1[k]
            v2 = f2[k]
            denom = max(abs(v1), abs(v2), 1e-6)
            dist += w * ((v1 - v2) / denom) ** 2
    return math.sqrt(dist)


def _compute_z_scores(gen_stats: dict, profile: dict) -> dict:
    scores = {}
    for k in FEATURE_KEYS_FULL:
        if k in gen_stats and k in profile:
            gv = gen_stats[k]["mean"]
            rm = profile[k]["mean"]
            rs = profile[k]["std"]
            z = (gv - rm) / max(rs, 1e-10)
            scores[k] = {"generated": gv, "ref_mean": rm, "ref_std": rs, "z_score": z}
    return scores


def cmd_analyze_output(args):
    """Compute features for generated output directories."""
    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = sorted(in_dir.glob("*.wav"))
    if not wav_files:
        print(f"Error: no .wav files found in {in_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Analyzing {len(wav_files)} files in {in_dir}...")

    features_list = []
    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            continue
        samples, sr = result
        feats = compute_features(samples, sr)
        feats["file"] = wav_path.name
        features_list.append(feats)

    if not features_list:
        print("Error: no files could be analyzed", file=sys.stderr)
        sys.exit(1)

    stats = {}
    for k in FEATURE_KEYS_FULL:
        vals = [f[k] for f in features_list if k in f]
        if vals:
            stats[k] = {
                "mean": float(np.mean(vals)),
                "std": float(np.std(vals)),
                "min": float(np.min(vals)),
                "max": float(np.max(vals)),
                "median": float(np.median(vals)),
            }

    output = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "source_dir": str(in_dir),
        "num_files": len(features_list),
        "stats": stats,
        "files": features_list,
    }

    out_path = in_dir / "generated_analysis.json"
    out_path.write_text(json.dumps(output, indent=2))

    print(f"\nAnalysis written to {out_path}")
    print(f"  Files: {len(features_list)}")
    if "spectral_centroid" in stats:
        print(f"  centroid: {stats['spectral_centroid']['mean']:.0f}Hz ({stats['spectral_centroid']['std']:.0f})")
    if "low_band_energy" in stats:
        print(f"  low_band: {stats['low_band_energy']['mean']:.3f} ({stats['low_band_energy']['std']:.3f})")
    if "high_band_energy" in stats:
        print(f"  high_band: {stats['high_band_energy']['mean']:.3f} ({stats['high_band_energy']['std']:.3f})")
    if "transient_count" in stats:
        print(f"  transients: {stats['transient_count']['mean']:.1f} ({stats['transient_count']['std']:.1f})")
    if "decay_length_ms" in stats:
        print(f"  decay: {stats['decay_length_ms']['mean']:.1f}ms ({stats['decay_length_ms']['std']:.1f})")
    if "amplitude_peaks" in stats:
        print(f"  amp_peaks: {stats['amplitude_peaks']['mean']:.1f} ({stats['amplitude_peaks']['std']:.1f})")

    return output


def cmd_compare_to_references(args):
    """Compare generated outputs against reference class profiles."""
    in_dir = Path(args.input_dir)
    analysis_path = in_dir / "generated_analysis.json"
    if not analysis_path.exists():
        print(f"Error: {analysis_path} not found. Run 'analyze-output' first.", file=sys.stderr)
        sys.exit(1)

    with open(analysis_path) as f:
        gen_data = json.load(f)
    gen_stats = gen_data.get("stats", {})

    profiles = load_profiles()
    target_cls = args.target

    gen_feats = {}
    for k, _, _ in COMPARE_KEYS_FULL:
        if k in gen_stats:
            gen_feats[k] = gen_stats[k]["mean"]

    ref_classes = sorted(profiles.keys())

    print("=" * 70)
    print(f"  COMPARE GENERATED vs REFERENCE PROFILES")
    print("=" * 70)
    print(f"\n  Source: {in_dir}")
    if target_cls:
        print(f"  Target: {target_cls}")
    print()

    header = f"  {'Reference Class':<20} {'Distance':>10} {'Centroid Δ':>10} {'Low Δ':>10} {'High Δ':>10} {'Trans Δ':>8} {'Decay Δ':>8}"
    sep = f"  {'-'*20} {'-'*10} {'-'*10} {'-'*10} {'-'*10} {'-'*8} {'-'*8}"
    print(header)
    print(sep)

    distances = []
    for ref_cls in ref_classes:
        profile = profiles[ref_cls]
        ref_feats = {}
        for k in ["spectral_centroid", "low_band_energy", "mid_band_energy",
                   "high_band_energy", "zero_crossing_rate", "transient_count",
                   "amplitude_peaks", "decay_length_ms", "rms", "duration_ms"]:
            if k in profile:
                ref_feats[k] = profile[k]["mean"]

        dist = feature_distance(gen_feats, ref_feats)

        def _delta(key):
            return gen_feats.get(key, 0) - ref_feats.get(key, 0)

        d_cent = _delta("spectral_centroid")
        d_low = _delta("low_band_energy")
        d_high = _delta("high_band_energy")
        d_trans = _delta("transient_count")
        d_decay = _delta("decay_length_ms")

        marker = "  ← TARGET" if ref_cls == target_cls else ""
        print(f"  {ref_cls:<20} {dist:>10.4f} {d_cent:>+10.0f} {d_low:>+10.4f} {d_high:>+10.4f} {d_trans:>+8.1f} {d_decay:>+8.1f}{marker}")
        distances.append((dist, ref_cls))

    distances.sort()
    nearest = distances[0][1]

    target_dist = None
    if target_cls:
        for d, rc in distances:
            if rc == target_cls:
                target_dist = d
                break

    print(f"\n  Nearest: {nearest} (d={distances[0][0]:.4f})")
    if target_cls:
        print(f"  Target '{target_cls}' distance: {target_dist:.4f}")
        if nearest == target_cls:
            print(f"  Status: maps to target ✓")
        else:
            print(f"  Status: maps to '{nearest}' instead ✗")

    compare_cls = target_cls if target_cls else nearest
    if compare_cls in profiles:
        profile = profiles[compare_cls]
        print()
        print(f"  Feature-by-feature vs '{compare_cls}':")
        print(f"  {'Feature':<22} {'Generated':>10} {'Ref Mean':>10} {'Ref Std':>10} {'Z-Score':>10} {'Status':>12}")
        print(f"  {'-'*22} {'-'*10} {'-'*10} {'-'*10} {'-'*10} {'-'*12}")

        for k, unit, fmt in COMPARE_KEYS_FULL:
            if k not in gen_stats or k not in profile:
                continue
            gv = gen_stats[k]["mean"]
            rm = profile[k]["mean"]
            rs = profile[k]["std"]
            z = (gv - rm) / max(rs, 1e-10)

            if abs(z) < 1.0:
                status = "✓ good"
            elif abs(z) < 2.0:
                status = "△ mild"
            elif abs(z) < 3.0:
                status = "▲ deviant"
            else:
                status = "✗ bad"

            label = k.replace("_", " ")
            print(f"  {label:<22} {fmt.format(gv):>10} {fmt.format(rm):>10} {fmt.format(rs):>10} {z:>+10.2f} {status:>12}")

    result = {
        "generated_dir": str(in_dir),
        "target_class": target_cls,
        "distances": {rc: d for d, rc in distances},
        "nearest_class": nearest,
        "target_distance": target_dist,
    }
    result_path = in_dir / "comparison_result.json"
    result_path.write_text(json.dumps(result, indent=2))
    print(f"\n  Written: {result_path}")

    return result


def cmd_diagnose(args):
    """Explain why a generated class sounds wrong using measurable differences."""
    in_dir = Path(args.input_dir)
    analysis_path = in_dir / "generated_analysis.json"
    comparison_path = in_dir / "comparison_result.json"

    if not analysis_path.exists():
        print(f"Error: {analysis_path} not found. Run 'analyze-output' first.", file=sys.stderr)
        sys.exit(1)

    with open(analysis_path) as f:
        gen_data = json.load(f)
    gen_stats = gen_data.get("stats", {})
    gen_files = gen_data.get("files", [])

    profiles = load_profiles()
    target_cls = args.target

    if not target_cls and comparison_path.exists():
        with open(comparison_path) as f:
            comp = json.load(f)
        target_cls = comp.get("nearest_class")
        if target_cls:
            print(f"  Auto-detected target class from comparison: {target_cls}")

    if not target_cls:
        print("Error: specify --target or run 'compare-to-references' first", file=sys.stderr)
        sys.exit(1)

    if target_cls not in profiles:
        print(f"Error: class '{target_cls}' not found in profiles", file=sys.stderr)
        sys.exit(1)

    profile = profiles[target_cls]
    z_scores = _compute_z_scores(gen_stats, profile)

    diagnoses = []

    if "spectral_centroid" in z_scores:
        z = z_scores["spectral_centroid"]["z_score"]
        if z > 2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("too bright", f"centroid {z_scores['spectral_centroid']['generated']:.0f}Hz is +{z:.1f}σ above ref {z_scores['spectral_centroid']['ref_mean']:.0f}Hz", sev, "spectral_centroid"))
        elif z < -2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("too dark", f"centroid {z_scores['spectral_centroid']['generated']:.0f}Hz is {z:.1f}σ below ref {z_scores['spectral_centroid']['ref_mean']:.0f}Hz", sev, "spectral_centroid"))

    if "spectral_centroid" in z_scores and "high_band_energy" in z_scores:
        zc = z_scores["spectral_centroid"]["z_score"]
        zh = z_scores["high_band_energy"]["z_score"]
        if zc > 1.5 and zh > 1.5:
            sev = min((abs(zc) + abs(zh)) / 8, 1.0)
            diagnoses.append(("too metallic", f"centroid +{zc:.1f}σ AND high-band +{zh:.1f}σ — excessive highs suggesting metallic ringing", sev, "high_band_energy"))

    if "low_band_energy" in z_scores:
        z = z_scores["low_band_energy"]["z_score"]
        if z > 2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("too much low end", f"low-band {z_scores['low_band_energy']['generated']:.4f} is +{z:.1f}σ above ref {z_scores['low_band_energy']['ref_mean']:.4f}", sev, "low_band_energy"))
        elif z < -2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("too thin", f"low-band {z_scores['low_band_energy']['generated']:.4f} is {z:.1f}σ below ref {z_scores['low_band_energy']['ref_mean']:.4f}", sev, "low_band_energy"))

    if "transient_count" in z_scores:
        z = z_scores["transient_count"]["z_score"]
        gv = z_scores["transient_count"]["generated"]
        rv = z_scores["transient_count"]["ref_mean"]
        if z > 2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("too many transients", f"transient count {gv:.0f} is +{z:.1f}σ above ref {rv:.0f}", sev, "transient_count"))
        elif z < -2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("too few transients", f"transient count {gv:.0f} is {z:.1f}σ below ref {rv:.0f}", sev, "transient_count"))

    if "amplitude_peaks" in z_scores and target_cls == "clap":
        gp = z_scores["amplitude_peaks"]["generated"]
        if gp < 4:
            diagnoses.append(("too few burst peaks", f"amplitude_peaks={gp:.0f}, expected >= 5 for realistic clap multi-burst", 0.8, "amplitude_peaks"))

    if "decay_length_ms" in z_scores:
        z = z_scores["decay_length_ms"]["z_score"]
        gd = z_scores["decay_length_ms"]["generated"]
        rd = z_scores["decay_length_ms"]["ref_mean"]
        if z > 2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("decay too long", f"decay {gd:.1f}ms is +{z:.1f}σ above ref {rd:.1f}ms", sev, "decay_length_ms"))
        elif z < -2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("decay too short", f"decay {gd:.1f}ms is {z:.1f}σ below ref {rd:.1f}ms", sev, "decay_length_ms"))

    if "attack_ms" in z_scores:
        z = z_scores["attack_ms"]["z_score"]
        if z > 2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("attack too slow", f"attack {z_scores['attack_ms']['generated']:.1f}ms is +{z:.1f}σ above ref {z_scores['attack_ms']['ref_mean']:.1f}ms", sev, "attack_ms"))
        elif z < -2.0:
            sev = min(abs(z) / 5, 1.0)
            diagnoses.append(("attack too fast", f"attack {z_scores['attack_ms']['generated']:.1f}ms is {z:.1f}σ below ref {z_scores['attack_ms']['ref_mean']:.1f}ms", sev, "attack_ms"))

    if comparison_path.exists():
        with open(comparison_path) as f:
            comp = json.load(f)
        nearest = comp.get("nearest_class")
        target_dist_val = comp.get("target_distance")
        if nearest and nearest != target_cls:
            diagnoses.append((f"too close to '{nearest}'", f"nearest ref class is '{nearest}' instead of '{target_cls}' — classifier confusion", 0.9, "nearest_class"))
        if target_dist_val is not None and target_dist_val > 0.5:
            sev = min(target_dist_val, 1.0)
            diagnoses.append(("not close enough to target", f"distance to '{target_cls}' is {target_dist_val:.4f}, should be < 0.5", sev, "target_distance"))

    if len(gen_files) >= 3:
        low_var = []
        for k in ["spectral_centroid", "low_band_energy", "transient_count"]:
            if k in gen_stats:
                mv = gen_stats[k]["mean"]
                sv = gen_stats[k]["std"]
                if mv > 0 and sv / mv < 0.05:
                    low_var.append((k, sv / mv))
        if low_var:
            details = ", ".join(f"{k} (CV={cv:.3f})" for k, cv in low_var)
            diagnoses.append(("not enough variation", f"coefficient of variation < 0.05 for: {details}", 0.5, "variation"))

    print("=" * 70)
    print(f"  DIAGNOSIS: Generated '{target_cls}' vs Reference")
    print("=" * 70)

    if not diagnoses:
        print(f"\n  ✓ All features within ±2σ of '{target_cls}' reference. No issues detected.")
    else:
        diagnoses.sort(key=lambda x: x[2], reverse=True)
        print(f"\n  {len(diagnoses)} issue(s) found:\n")
        for i, (issue, detail, sev, feat) in enumerate(diagnoses, 1):
            bar = "█" * int(sev * 10) + "░" * (10 - int(sev * 10))
            print(f"  {i}. {issue.upper()}")
            print(f"     {detail}")
            print(f"     Severity: {bar} ({sev:.0%})")
            print()

    result = {
        "generated_dir": str(in_dir),
        "target_class": target_cls,
        "z_scores": z_scores,
        "diagnoses": [{"issue": d[0], "detail": d[1], "severity": d[2], "feature": d[3]} for d in diagnoses],
        "diagnosis_count": len(diagnoses),
        "has_issues": len(diagnoses) > 0,
    }
    result_path = in_dir / "diagnosis_result.json"
    result_path.write_text(json.dumps(result, indent=2))
    print(f"  Written: {result_path}")

    return result


def cmd_refine(args):
    """Suggest and apply parameter/DSP changes based on diagnosis."""
    from_dir = Path(args.from_dir)
    diagnosis_path = from_dir / "diagnosis_result.json"
    analysis_path = from_dir / "generated_analysis.json"

    if not diagnosis_path.exists() or not analysis_path.exists():
        print(f"Error: diagnosis/analysis not found in {from_dir}. Run 'diagnose' first.", file=sys.stderr)
        sys.exit(1)

    with open(diagnosis_path) as f:
        diagnosis = json.load(f)
    with open(analysis_path) as f:
        gen_data = json.load(f)

    target_cls = args.target or diagnosis.get("target_class")
    if not target_cls:
        print("Error: specify --target or run 'diagnose' first", file=sys.stderr)
        sys.exit(1)

    issues = diagnosis.get("diagnoses", [])
    z_scores = diagnosis.get("z_scores", {})

    profiles = load_profiles()

    if target_cls not in SYNTHESIS_CLASSES:
        print(f"Error: '{target_cls}' is not a synthesis class", file=sys.stderr)
        sys.exit(1)

    label, synth_fn, default_dur, default_pitch = SYNTHESIS_CLASSES[target_cls]

    adjustments = {}
    adjustments["variation_boost"] = 0.0

    for issue in issues:
        feat = issue.get("feature", "")
        sev = issue.get("severity", 0.5)

        if feat == "spectral_centroid":
            z = z_scores.get("spectral_centroid", {}).get("z_score", 0)
            if z > 2:
                adjustments["centroid_cut"] = min(abs(z) * 0.05, 0.3)
            elif z < -2:
                adjustments["centroid_boost"] = min(abs(z) * 0.05, 0.3)

        elif feat == "high_band_energy":
            z = z_scores.get("high_band_energy", {}).get("z_score", 0)
            if z > 2:
                adjustments["high_shelf_cut_db"] = min(abs(z) * 1.5, 6.0)
            elif z < -2:
                adjustments["high_shelf_boost_db"] = min(abs(z) * 1.5, 6.0)

        elif feat == "low_band_energy":
            z = z_scores.get("low_band_energy", {}).get("z_score", 0)
            if z > 2:
                adjustments["low_shelf_cut_db"] = min(abs(z) * 1.5, 6.0)
            elif z < -2:
                adjustments["low_shelf_boost_db"] = min(abs(z) * 1.5, 6.0)

        elif feat == "transient_count":
            z = z_scores.get("transient_count", {}).get("z_score", 0)
            if z > 2:
                adjustments["transient_threshold"] = 1.0
            elif z < -2:
                adjustments["transient_boost"] = min(abs(z) * 0.15, 0.5)

        elif feat == "decay_length_ms":
            z = z_scores.get("decay_length_ms", {}).get("z_score", 0)
            if z > 2:
                adjustments["decay_reduction"] = min(abs(z) * 0.05, 0.3)
            elif z < -2:
                adjustments["decay_extension"] = min(abs(z) * 0.05, 0.3)

        elif feat == "amplitude_peaks" and target_cls == "clap":
            gp = z_scores.get("amplitude_peaks", {}).get("generated", 0)
            if gp < 5:
                adjustments["clap_burst_layers"] = int(max(5, 8 - gp))

        elif feat == "attack_ms":
            z = z_scores.get("attack_ms", {}).get("z_score", 0)
            if z > 2:
                adjustments["attack_shorten"] = min(abs(z) * 0.1, 0.5)
            elif z < -2:
                adjustments["attack_lengthen"] = min(abs(z) * 0.1, 0.5)

        elif feat == "variation":
            adjustments["variation_boost"] = 0.3

    has_confusion = any(d.get("feature") in ("nearest_class", "target_distance") for d in issues)
    if has_confusion and target_cls in profiles:
        ref_centroid = profiles[target_cls]["spectral_centroid"]["mean"]
        gen_centroid = gen_data.get("stats", {}).get("spectral_centroid", {}).get("mean", 0)
        diff = ref_centroid - gen_centroid
        if abs(diff) > 200:
            if diff > 0:
                adjustments["centroid_boost"] = max(adjustments.get("centroid_boost", 0), min(abs(diff) / 5000, 0.3))
            else:
                adjustments["centroid_cut"] = max(adjustments.get("centroid_cut", 0), min(abs(diff) / 5000, 0.3))

    print("=" * 70)
    print(f"  REFINEMENT PLAN")
    print("=" * 70)
    print(f"  Target: {target_cls}")
    print(f"  From:   {from_dir}")
    print(f"  To:     {args.out}")
    print(f"  Issues: {len(issues)}")

    if issues:
        print(f"\n  Adjustments:")
        if not adjustments:
            print("    (none)")
        for key, val in sorted(adjustments.items()):
            print(f"    {key}: {val}")

    suggestions = []
    for issue in issues:
        iname = issue.get("issue", "")
        if "too bright" in iname or "too metallic" in iname:
            suggestions.append("High-shelf cut (-3 to -6 dB at 4-8 kHz) to reduce excessive highs")
        if "too dark" in iname:
            suggestions.append("High-shelf boost (+3 to +6 dB at 4-8 kHz) to add air/brightness")
        if "too much low end" in iname:
            suggestions.append("Low-shelf cut (-3 to -6 dB at 100-200 Hz) to reduce mud")
        if "too thin" in iname:
            suggestions.append("Low-shelf boost (+3 to +6 dB at 100-200 Hz) to add body")
        if "too few transients" in iname:
            suggestions.append("Shorten attack, increase noise burst count, or sharpen envelopes")
        if "too many transients" in iname:
            suggestions.append("Lengthen attack, smooth envelopes, or reduce noise layers")
        if "decay too long" in iname:
            suggestions.append("Reduce duration by 10-30%, tighten release phase")
        if "decay too short" in iname:
            suggestions.append("Increase duration by 10-30%, extend release phase")
        if "too few burst peaks" in iname:
            suggestions.append("Increase clap burst layers from 6 to 8+ staggered noise hits")
        if "attack too slow" in iname:
            suggestions.append("Shorten attack envelope, increase initial transient amplitude")
        if "attack too fast" in iname:
            suggestions.append("Slightly soften attack, reduce click layer amplitude")
        if "too close" in iname:
            suggestions.append("Shift spectral centroid toward target class centroid")
        if "not close enough" in iname:
            suggestions.append("Move feature vector toward target reference means")
        if "not enough variation" in iname:
            suggestions.append("Widen pitch/duration randomization range (increase variance)")

    print(f"\n  Suggestions:")
    for s in suggestions:
        print(f"    → {s}")

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    count = args.count
    seed_offset = int(time.time() * 1000) % 1000000

    print(f"\n  Generating {count} refined samples → {out_dir}...")
    params_log = []

    for i in range(count):
        seed = (seed_offset + i) * 314159265 + hash(target_cls) % 1000000
        random.seed(seed)
        np.random.seed(seed % 2**32)

        dur_var = default_dur * (1.0 + (random.random() - 0.5) * 0.3)
        pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * 0.2)

        cb = adjustments.get("centroid_boost", 0)
        cc = adjustments.get("centroid_cut", 0)
        if cb > 0:
            pitch_var *= (1.0 + cb)
        if cc > 0:
            pitch_var *= (1.0 - cc)

        de = adjustments.get("decay_extension", 0)
        dr = adjustments.get("decay_reduction", 0)
        if de > 0:
            dur_var *= (1.0 + de)
        if dr > 0:
            dur_var *= (1.0 - dr)

        vb = adjustments.get("variation_boost", 0)
        if vb > 0:
            dur_var = default_dur * (1.0 + (random.random() - 0.5) * 0.5)
            pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * 0.4)

        samples = synth_fn(dur_var, pitch_var, profiles)

        hs_cut = adjustments.get("high_shelf_cut_db", 0)
        hs_boost = adjustments.get("high_shelf_boost_db", 0)
        if hs_cut > 0:
            samples = biquad_high_shelf(samples, 4000.0, -hs_cut, 0.7)
        if hs_boost > 0:
            samples = biquad_high_shelf(samples, 4000.0, hs_boost, 0.7)

        ls_cut = adjustments.get("low_shelf_cut_db", 0)
        ls_boost = adjustments.get("low_shelf_boost_db", 0)
        if ls_cut > 0:
            samples = biquad_low_shelf(samples, 150.0, -ls_cut, 0.7)
        if ls_boost > 0:
            samples = biquad_low_shelf(samples, 150.0, ls_boost, 0.7)

        peak = np.max(np.abs(samples))
        if peak > 0:
            samples = samples / peak * 0.9
        gain = 0.85 + random.random() * 0.15
        samples = samples * gain

        out_path = out_dir / f"{target_cls}_refined_{i+1:03d}.wav"
        write_wav(out_path, samples)
        if i == 0 or (i + 1) % 5 == 0:
            print(f"    [{i+1}/{count}] {out_path.name}")

        params_log.append({
            "file": out_path.name, "seed": seed,
            "duration_ms": dur_var, "pitch_hz": pitch_var,
            "high_shelf_cut_db": hs_cut, "high_shelf_boost_db": hs_boost,
            "low_shelf_cut_db": ls_cut, "low_shelf_boost_db": ls_boost,
            "decay_extension": de, "decay_reduction": dr,
            "centroid_boost": cb, "centroid_cut": cc,
            "variation_boost": vb,
        })

    log = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "target_class": target_cls,
        "from_dir": str(from_dir),
        "out_dir": str(out_dir),
        "issues": issues,
        "adjustments": adjustments,
        "suggestions": suggestions,
        "num_generated": count,
        "params_log": params_log,
    }
    log_path = out_dir / "refinement_log.json"
    log_path.write_text(json.dumps(log, indent=2))
    print(f"\n  Log: {log_path}")
    print(f"  Done: {count} refined {label} → {out_dir}")

    return log


def cmd_audit_report(args):
    """Write an HTML or Markdown report with feature comparison tables."""
    in_dir = Path(args.input_dir)
    analysis_path = in_dir / "generated_analysis.json"
    comparison_path = in_dir / "comparison_result.json"
    diagnosis_path = in_dir / "diagnosis_result.json"

    analysis = comparison = diagnosis = None
    if analysis_path.exists():
        with open(analysis_path) as f:
            analysis = json.load(f)
    if comparison_path.exists():
        with open(comparison_path) as f:
            comparison = json.load(f)
    if diagnosis_path.exists():
        with open(diagnosis_path) as f:
            diagnosis = json.load(f)

    target_cls = args.target
    if not target_cls:
        target_cls = (diagnosis or comparison or {}).get("target_class")

    fmt = args.format
    if fmt == "markdown":
        lines = build_markdown_report(analysis, comparison, diagnosis, in_dir, target_cls)
    else:
        lines = build_html_report(analysis, comparison, diagnosis, in_dir, target_cls)

    out_path = in_dir / f"audit_report.{fmt}"
    out_path.write_text("\n".join(lines))
    print(f"Audit report written to {out_path}")
