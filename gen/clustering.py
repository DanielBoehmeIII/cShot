"""Reference clustering: PCA, K-Means, cluster-to-class mapping."""

import json
import math
import sys
import time
from pathlib import Path
from typing import Optional

import numpy as np
from scipy.cluster.vq import kmeans, vq

from gen import REPO_ROOT
from gen.features import FEATURE_KEYS_FULL

CLUSTER_FEATURES = [
    "spectral_centroid", "zero_crossing_rate",
    "low_band_energy", "mid_band_energy", "high_band_energy",
    "transient_count", "decay_length_ms", "attack_ms",
    "hpr", "pitch_hz", "pitch_confidence",
    "spectral_flux_mean", "spectral_flux_std",
    "mfcc_1", "mfcc_2", "mfcc_3", "mfcc_4", "mfcc_5",
]


def extract_feature_matrix(analysis: dict, feature_keys: list[str] = None) -> tuple[np.ndarray, list[str], list[str]]:
    """Extract feature matrix from reference analysis.
    Returns (matrix, file_paths, classes).
    Matrix shape: (n_files, n_features).
    """
    if feature_keys is None:
        feature_keys = CLUSTER_FEATURES

    files = analysis.get("files", {})
    matrix_rows = []
    file_paths = []
    classes = []

    for rel_path, feats in files.items():
        row = []
        valid = True
        for k in feature_keys:
            v = feats.get(k)
            if v is None or (isinstance(v, float) and (math.isnan(v) or math.isinf(v))):
                v = 0.0
            row.append(float(v))
        matrix_rows.append(row)
        file_paths.append(rel_path)
        classes.append(feats.get("class", "unknown"))

    return np.array(matrix_rows, dtype=np.float64), file_paths, classes


def normalize_matrix(X: np.ndarray) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Z-score normalize columns. Returns (normalized, mean, std)."""
    mean = np.mean(X, axis=0)
    std = np.std(X, axis=0)
    std[std < 1e-10] = 1.0
    X_norm = (X - mean) / std
    return X_norm, mean, std


def compute_pca(X: np.ndarray, n_components: int = 10) -> tuple[np.ndarray, np.ndarray, float]:
    """PCA via SVD. Returns (projected, components, explained_variance_ratio)."""
    n, m = X.shape
    n_components = min(n_components, n, m)
    X_centered = X - np.mean(X, axis=0)
    U, s, Vt = np.linalg.svd(X_centered, full_matrices=False)
    projected = U[:, :n_components] * s[:n_components]
    total_var = np.sum(s ** 2)
    explained = np.sum(s[:n_components] ** 2) / max(total_var, 1e-10)
    return projected, Vt[:n_components], explained


def cluster_kmeans(X: np.ndarray, n_clusters: int = 12, n_iter: int = 50, seed: int = 42) -> tuple[np.ndarray, np.ndarray]:
    """K-Means clustering using scipy. Returns (labels, centroids)."""
    np.random.seed(seed)
    centroids, distortion = kmeans(X, n_clusters, iter=n_iter)
    labels, _ = vq(X, centroids)
    return labels, centroids


def map_clusters_to_classes(labels: np.ndarray, classes: list[str]) -> dict:
    """Map each cluster to its majority class. Returns {cluster_id: majority_class}."""
    cluster_classes: dict[int, dict[str, int]] = {}
    for label, cls in zip(labels, classes):
        if label not in cluster_classes:
            cluster_classes[label] = {}
        cluster_classes[label][cls] = cluster_classes[label].get(cls, 0) + 1

    mapping = {}
    cluster_metrics = {}
    for cid, cls_counts in cluster_classes.items():
        total = sum(cls_counts.values())
        majority_cls = max(cls_counts, key=cls_counts.get)
        majority_pct = cls_counts[majority_cls] / max(total, 1)
        mapping[int(cid)] = majority_cls
        cluster_metrics[int(cid)] = {
            "majority_class": majority_cls,
            "purity": round(majority_pct, 4),
            "size": total,
            "class_distribution": {k: v for k, v in sorted(cls_counts.items(), key=lambda x: -x[1])},
        }
    return mapping, cluster_metrics


def compute_cluster_confusion(cluster_metrics: dict, mapping: dict) -> list[dict]:
    """Find clusters with high class impurity (confusing classes)."""
    confusion = []
    for cid, metrics in cluster_metrics.items():
        if metrics["purity"] < 0.7 and metrics["size"] >= 5:
            classes_str = ", ".join(
                f"{k}({v})" for k, v in metrics["class_distribution"].items()
            )
            confusion.append({
                "cluster": cid,
                "majority_class": metrics["majority_class"],
                "purity": metrics["purity"],
                "size": metrics["size"],
                "class_distribution": metrics["class_distribution"],
            })
    return sorted(confusion, key=lambda x: x["purity"])


def cmd_cluster_references(args):
    """Run automatic reference clustering without relying on filenames."""

    analysis_path = Path(args.analysis) if args.analysis else REPO_ROOT / "reference_analysis.json"
    if not analysis_path.exists():
        print(f"Error: {analysis_path} not found. Run 'scan' first.", file=sys.stderr)
        sys.exit(1)

    with open(analysis_path) as f:
        analysis = json.load(f)

    n_clusters = args.n_clusters if hasattr(args, 'n_clusters') and args.n_clusters else 12
    n_pca = args.pca_dims if hasattr(args, 'pca_dims') and args.pca_dims else min(10, n_clusters)

    feature_keys = CLUSTER_FEATURES
    print(f"Using {len(feature_keys)} features for clustering")
    print(f"PCA dimensions: {n_pca}")
    print(f"K-Means clusters: {n_clusters}")

    # 1. Extract feature matrix
    X, file_paths, classes = extract_feature_matrix(analysis, feature_keys)
    print(f"Feature matrix: {X.shape[0]} files × {X.shape[1]} features")

    # Filter out files with zero features (all zeros)
    valid_mask = np.any(np.abs(X) > 1e-10, axis=1)
    if np.sum(valid_mask) < X.shape[0]:
        print(f"  Filtered out {X.shape[0] - np.sum(valid_mask)} zero-feature files")
        X = X[valid_mask]
        file_paths = [p for i, p in enumerate(file_paths) if valid_mask[i]]
        classes = [c for i, c in enumerate(classes) if valid_mask[i]]

    if X.shape[0] < n_clusters:
        print(f"Error: too few valid files ({X.shape[0]}) for {n_clusters} clusters", file=sys.stderr)
        sys.exit(1)

    # 2. Normalize
    X_norm, feat_mean, feat_std = normalize_matrix(X)

    # 3. PCA
    X_pca, components, explained_var = compute_pca(X_norm, n_pca)
    print(f"PCA explained variance: {explained_var:.1%}")

    # 4. K-Means
    labels, centroids = cluster_kmeans(X_pca, n_clusters)
    n_unique = len(set(labels))
    print(f"K-Means: {n_unique} non-empty clusters formed")

    # 5. Map clusters to classes
    mapping, cluster_metrics = map_clusters_to_classes(labels, classes)

    # 6. Compute confusion
    confusion = compute_cluster_confusion(cluster_metrics, mapping)

    # Print report
    print(f"\n{'='*60}")
    print(f"  CLUSTERING REPORT")
    print(f"{'='*60}")
    print(f"\n  Cluster → Class Mapping:")
    print(f"  {'Cluster':>8} {'Size':>6} {'Purity':>8} {'Majority Class':>20}")
    print(f"  {'─'*8} {'─'*6} {'─'*8} {'─'*20}")
    for cid in sorted(cluster_metrics.keys()):
        m = cluster_metrics[cid]
        purity_str = f"{m['purity']:.0%}"
        print(f"  {cid:>8} {m['size']:>6} {purity_str:>8} {m['majority_class']:>20}")

    if confusion:
        print(f"\n  Confusing clusters (purity < 70%, size >= 5):")
        for c in confusion:
            print(f"\n    Cluster {c['cluster']}: {c['majority_class']} (purity={c['purity']:.0%}, size={c['size']})")
            for cls_name, count in c["class_distribution"].items():
                pct = count / c["size"] * 100
                bar = "█" * int(pct / 5) + "░" * (20 - int(pct / 5))
                print(f"      {cls_name:15s} {count:>4} ({pct:>5.1f}%) {bar}")

    # Class-level cross-tabulation
    print(f"\n  Class distribution across clusters:")
    unique_classes = sorted(set(classes))
    print(f"  {'Class':<18}", end="")
    for cid in sorted(cluster_metrics.keys()):
        print(f" {'C'+str(cid):>6}", end="")
    print(f" {'Total':>6}")
    print(f"  {'─'*18}", end="")
    for cid in sorted(cluster_metrics.keys()):
        print(f" {'─'*6}", end="")
    print(f" {'─'*6}")

    class_cluster = {}
    for cls in unique_classes:
        class_cluster[cls] = {cid: 0 for cid in cluster_metrics}
    for label, cls in zip(labels, classes):
        if cls in class_cluster and label in class_cluster[cls]:
            class_cluster[cls][label] += 1

    for cls in sorted(class_cluster.keys()):
        total = sum(class_cluster[cls].values())
        if total == 0:
            continue
        print(f"  {cls:<18}", end="")
        for cid in sorted(cluster_metrics.keys()):
            v = class_cluster[cls][cid]
            pct = v / max(total, 1) * 100
            if pct >= 10:
                print(f" {v:>4}  ", end="")
            else:
                print(f"  {'.':>4}  ", end="")
        print(f" {total:>6}")

    # Build output
    result = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "config": {
            "n_clusters": n_clusters,
            "n_pca": n_pca,
            "n_features": len(feature_keys),
            "features_used": feature_keys,
        },
        "pca_explained_variance": round(explained_var, 4),
        "cluster_metrics": cluster_metrics,
        "cluster_mapping": mapping,
        "confusing_clusters": confusion,
        "assignments": [
            {"file": p, "class": c, "cluster": int(l), "mapped_class": mapping.get(int(l), "?")}
            for p, c, l in zip(file_paths, classes, labels)
        ],
    }

    out_path = Path(args.output) if args.output else REPO_ROOT / "reference_clusters.json"
    out_path.write_text(json.dumps(result, indent=2))
    print(f"\nCluster assignments written to {out_path}")

    return result
