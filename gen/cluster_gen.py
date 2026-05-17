"""Cluster-targeted generation: generate toward cluster statistics instead of class rules."""

import json
import random
import math
import sys
import time
from pathlib import Path
from collections import defaultdict

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.features import compute_features, FEATURE_KEYS_FULL
from gen.io import read_wav, write_wav
from gen.synthesis import SYNTHESIS_CLASSES, synthesize_kick, synthesize_snare, synthesize_clap
from gen.synthesis import synthesize_closed_hat, synthesize_open_hat, synthesize_808
from gen.synthesis import synthesize_bass_stab, synthesize_impact_fx, synthesize_synth_stab, synthesize_guitar_stab
from gen.dsp import biquad_high_shelf, biquad_low_shelf, biquad_peaking
from gen.scanning import load_profiles
from gen.refinement import feature_distance

PROFILE_KEYS = [
    "spectral_centroid", "low_band_energy", "mid_band_energy", "high_band_energy",
    "zero_crossing_rate", "transient_count", "amplitude_peaks",
    "decay_length_ms", "attack_ms", "rms",
    "hpr", "pitch_hz", "spectral_flux_mean",
    "mfcc_1", "mfcc_2", "mfcc_3",
]


def compute_cluster_profile(cluster_assignments: list[dict], analysis: dict, cluster_id: int) -> dict:
    """Compute feature profile for a single cluster from its constituent files."""
    cluster_files = [
        a["file"] for a in cluster_assignments if a["cluster"] == cluster_id
    ]

    if not cluster_files:
        print(f"Error: cluster {cluster_id} has no files", file=sys.stderr)
        sys.exit(1)

    # Gather features for each file in the cluster
    files_meta = analysis.get("files", {})
    feat_lists: dict[str, list[float]] = defaultdict(list)

    for rel_path in cluster_files:
        feats = files_meta.get(rel_path)
        if feats is None:
            continue
        for k in PROFILE_KEYS:
            v = feats.get(k)
            if v is not None and not (isinstance(v, float) and (math.isnan(v) or math.isinf(v))):
                feat_lists[k].append(float(v))

    # Compute aggregate stats
    profile = {"num_files": len(cluster_files)}
    for k in PROFILE_KEYS:
        vals = feat_lists.get(k, [])
        if len(vals) >= 2:
            profile[k] = {
                "mean": float(np.mean(vals)),
                "std": float(np.std(vals)),
                "min": float(np.min(vals)),
                "max": float(np.max(vals)),
                "median": float(np.median(vals)),
            }
        elif len(vals) == 1:
            profile[k] = {"mean": vals[0], "std": 0.0, "min": vals[0], "max": vals[0], "median": vals[0]}
        else:
            profile[k] = {"mean": 0.0, "std": 0.0, "min": 0.0, "max": 0.0, "median": 0.0}

    return profile


def find_nearest_synthesis_class(cluster_profile: dict, class_profiles: dict) -> tuple[str, float]:
    """Find the synthesis class whose feature profile is closest to the cluster."""
    cluster_means = {k: cluster_profile[k]["mean"] for k in PROFILE_KEYS if k in cluster_profile}

    best_cls = None
    best_dist = float("inf")

    for cls_name, cp in class_profiles.items():
        if cls_name not in SYNTHESIS_CLASSES:
            continue
        ref_means = {k: cp[k]["mean"] for k in PROFILE_KEYS if k in cp and k in cluster_profile}
        if not ref_means:
            continue
        dist = feature_distance(cluster_means, ref_means)
        if dist < best_dist:
            best_dist = dist
            best_cls = cls_name

    return best_cls, best_dist


def apply_cluster_adjustments(samples: np.ndarray, cluster_profile: dict,
                               class_profile: dict) -> np.ndarray:
    """Apply DSP adjustments to push generated samples toward cluster profile."""
    adjustments_applied = []

    for feat_name in ["spectral_centroid", "high_band_energy", "low_band_energy"]:
        if feat_name not in cluster_profile or feat_name not in class_profile:
            continue
        cp = cluster_profile[feat_name]
        rp = class_profile[feat_name]
        cluster_mean = cp["mean"]
        class_mean = rp["mean"]
        class_std = max(rp.get("std", 0.001), 0.001)
        z = (cluster_mean - class_mean) / class_std

        if abs(z) < 0.5:
            continue

        if feat_name == "spectral_centroid":
            if z > 1.0:
                boost_db = min(z * 1.5, 6.0)
                samples = biquad_high_shelf(samples, 4000.0, boost_db, 0.7)
                adjustments_applied.append(f"high_shelf_boost:{boost_db:.1f}dB (centroid z={z:.2f})")
            elif z < -1.0:
                cut_db = min(abs(z) * 1.5, 6.0)
                samples = biquad_high_shelf(samples, 4000.0, -cut_db, 0.7)
                adjustments_applied.append(f"high_shelf_cut:{cut_db:.1f}dB (centroid z={z:.2f})")

        elif feat_name == "high_band_energy":
            if z > 1.0:
                boost_db = min(z * 2.0, 8.0)
                samples = biquad_high_shelf(samples, 6000.0, boost_db, 0.7)
                adjustments_applied.append(f"high_energy_boost:{boost_db:.1f}dB (z={z:.2f})")

        elif feat_name == "low_band_energy":
            if z > 1.0:
                boost_db = min(z * 2.0, 8.0)
                samples = biquad_low_shelf(samples, 200.0, boost_db, 0.7)
                adjustments_applied.append(f"low_energy_boost:{boost_db:.1f}dB (z={z:.2f})")
            elif z < -1.0:
                cut_db = min(abs(z) * 2.0, 8.0)
                samples = biquad_low_shelf(samples, 200.0, -cut_db, 0.7)
                adjustments_applied.append(f"low_energy_cut:{cut_db:.1f}dB (z={z:.2f})")

    return samples, adjustments_applied


def cmd_cluster_gen(args):
    """Generate samples targeting a specific reference cluster's statistics."""

    clusters_path = Path(args.clusters) if args.clusters else REPO_ROOT / "reference_clusters.json"
    analysis_path = Path(args.analysis) if args.analysis else REPO_ROOT / "reference_analysis.json"
    profiles_path = REPO_ROOT / "class_profiles.json"

    if not clusters_path.exists():
        print(f"Error: {clusters_path} not found. Run 'cluster-references' first.", file=sys.stderr)
        sys.exit(1)

    with open(clusters_path) as f:
        cluster_data = json.load(f)

    with open(analysis_path) as f:
        analysis = json.load(f)

    class_profiles = load_profiles(profiles_path)

    cluster_id = args.cluster_id
    cluster_assignments = cluster_data.get("assignments", [])
    cluster_metrics = cluster_data.get("cluster_metrics", {})

    # Validate cluster
    valid_clusters = [int(c) for c in cluster_metrics.keys()]
    if cluster_id not in valid_clusters:
        print(f"Error: cluster {cluster_id} not found. Valid: {sorted(valid_clusters)}", file=sys.stderr)
        sys.exit(1)

    # Compute cluster profile
    print(f"Computing profile for cluster {cluster_id}...")
    cluster_profile = compute_cluster_profile(cluster_assignments, analysis, cluster_id)
    majority_class = cluster_metrics.get(str(cluster_id), {}).get("majority_class", "unknown")
    print(f"  Files: {cluster_profile['num_files']}")
    print(f"  Majority class: {majority_class}")

    # Print feature ranges
    for k in ["spectral_centroid", "low_band_energy", "high_band_energy",
               "transient_count", "decay_length_ms", "hpr"]:
        if k in cluster_profile:
            v = cluster_profile[k]
            print(f"  {k}: mean={v['mean']:.3f} std={v['std']:.3f} [{v['min']:.3f}, {v['max']:.3f}]")

    # Find nearest synthesis class
    synth_class, dist_to_nearest = find_nearest_synthesis_class(cluster_profile, class_profiles)
    if synth_class is None:
        print("Error: no matching synthesis class found for this cluster", file=sys.stderr)
        sys.exit(1)

    print(f"\nNearest synthesis class: {synth_class} (distance={dist_to_nearest:.4f})")

    # Also find the majority class synth distance for comparison
    maj_dist = None
    if majority_class in SYNTHESIS_CLASSES and majority_class in class_profiles:
        maj_means = {k: class_profiles[majority_class][k]["mean"] for k in PROFILE_KEYS
                     if k in class_profiles[majority_class]}
        clus_means = {k: cluster_profile[k]["mean"] for k in PROFILE_KEYS if k in cluster_profile}
        maj_dist = feature_distance(clus_means, maj_means)
        print(f"  Majority class '{majority_class}' distance: {maj_dist:.4f}")

    if dist_to_nearest <= maj_dist if maj_dist else True:
        use_class = synth_class
        print(f"  Using synthesis class: {use_class}")
    else:
        use_class = majority_class
        print(f"  Using majority class instead: {use_class}")

    # Get synthesis function
    if use_class not in SYNTHESIS_CLASSES:
        print(f"Error: nearest class '{use_class}' is not synthetically generated", file=sys.stderr)
        sys.exit(1)

    label, synth_fn, default_dur, default_pitch = SYNTHESIS_CLASSES[use_class]

    # Generate
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    count = args.count
    seed_offset = int(time.time() * 1000) % 1000000

    print(f"\nGenerating {count} samples targeting cluster {cluster_id} → {out_dir}...")
    params_log = []

    for i in range(count):
        seed = (seed_offset + i) * 314159265 + hash(str(cluster_id)) % 1000000
        random.seed(seed)
        np.random.seed(seed % 2**32)

        # Adjust duration and pitch toward cluster statistics
        cluster_centroid = cluster_profile.get("spectral_centroid", {}).get("mean", default_pitch)
        cluster_decay = cluster_profile.get("decay_length_ms", {}).get("mean", default_dur)
        cluster_attack = cluster_profile.get("attack_ms", {}).get("mean", 0)

        dur_var = default_dur
        pitch_var = default_pitch

        # If cluster has meaningful decay info, adjust duration
        if cluster_decay > 0:
            # Map decay ratio to duration adjustment
            decay_ratio = cluster_decay / max(default_dur, 1)
            dur_var = default_dur * max(0.3, min(decay_ratio * 1.5, 3.0))
            dur_var += (random.random() - 0.5) * dur_var * 0.2

        # Adjust pitch toward cluster centroid if relevant
        if cluster_centroid > 200:
            cent_ratio = cluster_centroid / max(default_pitch, 1)
            pitch_var = default_pitch * max(0.5, min(cent_ratio * 0.8, 2.0))
            pitch_var += (random.random() - 0.5) * pitch_var * 0.15

        samples = synth_fn(dur_var, pitch_var, class_profiles if profiles_path.exists() else None)

        # Apply DSP adjustments toward cluster
        class_profile_for_adjust = class_profiles.get(use_class, {})
        samples, adjustments = apply_cluster_adjustments(samples, cluster_profile, class_profile_for_adjust)

        peak = np.max(np.abs(samples))
        if peak > 0:
            samples = samples / peak * 0.9
        gain = 0.85 + random.random() * 0.15
        samples = samples * gain

        out_path = out_dir / f"cluster{cluster_id}_{use_class}_{i+1:03d}.wav"
        write_wav(out_path, samples)
        if i == 0 or (i + 1) % 5 == 0:
            print(f"  [{i+1}/{count}] {out_path.name}")

        params_log.append({
            "file": out_path.name,
            "seed": seed,
            "duration_ms": dur_var,
            "pitch_hz": pitch_var,
            "adjustments": adjustments,
        })

    # Save log
    log = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "cluster_id": cluster_id,
        "cluster_size": cluster_profile["num_files"],
        "majority_class": majority_class,
        "synthesis_class": use_class,
        "distance_to_nearest_class": round(dist_to_nearest, 4),
        "distance_to_majority_class": round(maj_dist, 4) if maj_dist else None,
        "cluster_profile_summary": {
            k: {"mean": round(cluster_profile[k]["mean"], 4),
                "std": round(cluster_profile[k]["std"], 4)}
            for k in ["spectral_centroid", "low_band_energy", "high_band_energy",
                      "transient_count", "decay_length_ms", "hpr"]
            if k in cluster_profile
        },
        "params_log": params_log,
    }
    log_path = out_dir / "cluster_gen_log.json"
    log_path.write_text(json.dumps(log, indent=2))
    print(f"\nLog: {log_path}")
    print(f"Done: {count} cluster-{cluster_id} samples → {out_dir}")


def cmd_cluster_qa(args):
    """QA: verify generated outputs are closer to target cluster than rival clusters."""
    in_dir = Path(args.input_dir)
    clusters_path = Path(args.clusters) if args.clusters else REPO_ROOT / "reference_clusters.json"
    analysis_path = REPO_ROOT / "reference_analysis.json"

    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)
    if not clusters_path.exists():
        print(f"Error: {clusters_path} not found. Run 'cluster-references' first.", file=sys.stderr)
        sys.exit(1)
    if not analysis_path.exists():
        print(f"Error: {analysis_path} not found. Run 'scan' first.", file=sys.stderr)
        sys.exit(1)

    with open(clusters_path) as f:
        cluster_data = json.load(f)
    with open(analysis_path) as f:
        analysis = json.load(f)

    # If target cluster not specified, try to detect from cluster_gen_log.json
    target_cluster_id = args.cluster_id
    if target_cluster_id is None:
        log_path = in_dir / "cluster_gen_log.json"
        if log_path.exists():
            with open(log_path) as f:
                log_data = json.load(f)
            target_cluster_id = log_data.get("cluster_id")
            print(f"Auto-detected target cluster from log: {target_cluster_id}")

    if target_cluster_id is None:
        print("Error: specify --cluster-id or run from a cluster-gen output directory", file=sys.stderr)
        sys.exit(1)

    # Compute profiles for ALL clusters
    cluster_assignments = cluster_data.get("assignments", [])
    cluster_ids = sorted(set(a["cluster"] for a in cluster_assignments))

    print(f"Computing profiles for {len(cluster_ids)} clusters...")
    all_cluster_profiles = {}
    for cid in cluster_ids:
        all_cluster_profiles[cid] = compute_cluster_profile(cluster_assignments, analysis, cid)

    target_profile = all_cluster_profiles.get(target_cluster_id)
    if not target_profile:
        print(f"Error: cluster {target_cluster_id} not found", file=sys.stderr)
        sys.exit(1)

    # Collect generated WAVs
    wav_files = sorted(in_dir.glob("*.wav"))
    if not wav_files:
        print(f"Error: no .wav files in {in_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"\nCluster QA: {len(wav_files)} generated files vs {len(cluster_ids)} reference clusters")
    print(f"Target cluster: {target_cluster_id}")
    print()

    # Test each file
    results = []
    passed = 0
    failed = 0

    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            print(f"  ✗ {wav_path.name}: could not read")
            failed += 1
            continue
        samples, sr = result
        feats = compute_features(samples, sr)

        # Build feature dict matching PROFILE_KEYS
        gen_feats = {k: feats.get(k, 0.0) for k in PROFILE_KEYS}

        # Distance to every cluster
        cluster_dists = []
        for cid in cluster_ids:
            cp = all_cluster_profiles[cid]
            ref_means = {k: cp[k]["mean"] for k in PROFILE_KEYS if k in cp}
            dist = feature_distance(gen_feats, ref_means)
            cluster_dists.append((dist, cid))

        cluster_dists.sort()
        nearest_cid = cluster_dists[0][1]
        nearest_dist = cluster_dists[0][0]
        target_dist = next(d for d, c in cluster_dists if c == target_cluster_id)

        is_pass = nearest_cid == target_cluster_id
        if is_pass:
            passed += 1
            icon = "✓"
        else:
            failed += 1
            icon = "✗"

        # Show nearest rival info
        rival = None
        for d, c in cluster_dists:
            if c != target_cluster_id:
                rival = (c, d)
                break

        results.append({
            "file": wav_path.name,
            "target_cluster": target_cluster_id,
            "nearest_cluster": nearest_cid,
            "distance_to_target": round(target_dist, 4),
            "distance_to_nearest": round(nearest_dist, 4),
            "nearest_rival_cluster": rival[0] if rival else None,
            "distance_to_rival": round(rival[1], 4) if rival else None,
            "pass": is_pass,
        })

        rival_str = f" (rival=C{rival[0]}@{rival[1]:.3f})" if rival else ""
        print(f"  {icon} {wav_path.name:45s} target=C{target_cluster_id} dist={target_dist:.4f} "
              f"nearest=C{nearest_cid} dist={nearest_dist:.4f}{rival_str}")

    # Summary
    print(f"\n  {'='*55}")
    print(f"  CLUSTER QA SUMMARY")
    print(f"  {'='*55}")
    print(f"  Target cluster:    C{target_cluster_id}")
    print(f"  Files tested:      {len(wav_files)}")
    print(f"  Passed:             {passed}")
    print(f"  Failed:             {failed}")
    pass_rate = (passed / max(len(wav_files), 1)) * 100
    print(f"  Pass rate:          {pass_rate:.1f}%")
    status = "PASS" if pass_rate >= 80 else ("WARN" if pass_rate >= 50 else "FAIL")
    print(f"  Overall:            {status}")

    # Print target cluster features for context
    print(f"\n  Target cluster C{target_cluster_id} profile:")
    for k in ["spectral_centroid", "low_band_energy", "high_band_energy",
               "transient_count", "decay_length_ms", "hpr"]:
        if k in target_profile:
            v = target_profile[k]
            print(f"    {k}: mean={v['mean']:.3f} std={v['std']:.3f}")

    # Print generated vs target feature comparison
    print(f"\n  Generated vs cluster centroid (averaged):")
    gen_avgs = {}
    for k in PROFILE_KEYS:
        vals = []
        for r in results:
            result_dir = in_dir
        # Re-compute from the files
        for wav_path in wav_files:
            res = read_wav(wav_path)
            if res:
                s, _ = res
                f_ = compute_features(s, SAMPLE_RATE)
                if k in f_:
                    vals.append(f_[k])
        if vals:
            gen_avgs[k] = float(np.mean(vals))

    for k in ["spectral_centroid", "low_band_energy", "high_band_energy",
               "transient_count", "decay_length_ms"]:
        if k in gen_avgs and k in target_profile:
            gv = gen_avgs[k]
            tv = target_profile[k]["mean"]
            ts = max(target_profile[k]["std"], 0.001)
            z = (gv - tv) / ts
            print(f"    {k:25s}: gen={gv:.3f} target={tv:.3f} z={z:+.2f}")

    # Save results
    output = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "target_cluster": target_cluster_id,
        "files_tested": len(wav_files),
        "passed": passed,
        "failed": failed,
        "pass_rate": round(pass_rate, 1),
        "status": status,
        "results": results,
        "target_cluster_profile": {
            k: {"mean": round(target_profile[k]["mean"], 4),
                "std": round(target_profile[k]["std"], 4)}
            for k in ["spectral_centroid", "low_band_energy", "high_band_energy",
                      "transient_count", "decay_length_ms", "hpr"]
            if k in target_profile
        },
    }

    out_path = Path(args.output) if args.output else in_dir / "cluster_qa_results.json"
    out_path.write_text(json.dumps(output, indent=2))
    print(f"\n  Results: {out_path}")
