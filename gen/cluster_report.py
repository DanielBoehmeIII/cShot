"""Cluster visualization reports: class maps, nearest neighbors, confusing classes."""

import json
import math
import time
from pathlib import Path
from collections import defaultdict

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.features import FEATURE_KEYS_FULL

COMPARE_KEYS = [
    "spectral_centroid", "low_band_energy", "mid_band_energy",
    "high_band_energy", "zero_crossing_rate", "transient_count",
    "decay_length_ms", "amplitude_peaks", "rms",
]


def _z_dist(a: dict, b: dict, keys: list[str]) -> float:
    """Euclidean distance between two feature dicts (z-score normalized per key)."""
    dist = 0.0
    n = 0
    for k in keys:
        if k in a and k in b:
            v1 = a[k]
            v2 = b[k]
            denom = max(abs(v1), abs(v2), 1e-6)
            dist += ((v1 - v2) / denom) ** 2
            n += 1
    return math.sqrt(dist / max(n, 1))


def _load_json(path: Path) -> dict:
    with open(path) as f:
        return json.load(f)


def cmd_cluster_report(args):
    """Generate cluster visualization report: class map, nearest neighbors, confusing classes."""

    # Load data
    profiles_path = Path(args.profiles) if args.profiles else REPO_ROOT / "class_profiles.json"
    analysis_path = Path(args.analysis) if args.analysis else REPO_ROOT / "reference_analysis.json"
    clusters_path = Path(args.clusters) if args.clusters else REPO_ROOT / "reference_clusters.json"

    if not profiles_path.exists():
        print(f"Error: {profiles_path} not found. Run 'profiles' first.", file=sys.stderr)
        sys.exit(1)

    profiles_data = _load_json(profiles_path)
    profiles = profiles_data.get("profiles", {})

    analysis = {}
    if analysis_path.exists():
        analysis = _load_json(analysis_path)

    cluster_data = None
    cluster_assignments = None
    if clusters_path.exists():
        cluster_data = _load_json(clusters_path)
        cluster_assignments = cluster_data.get("assignments", [])

    feats = {}
    for cls_name in profiles:
        p = profiles[cls_name]
        feats[cls_name] = {k: p[k]["mean"] for k in COMPARE_KEYS if k in p}
        feats[cls_name]["spectral_rolloff"] = p.get("spectral_rolloff", {}).get("mean", 0)

    # 1. Between-class distance matrix
    class_names = sorted(feats.keys())
    n = len(class_names)
    dist_matrix = np.zeros((n, n))

    print("=" * 60)
    print("  CLUSTER VISUALIZATION REPORT")
    print("=" * 60)
    print()
    print("  Class Similarity Matrix")
    print("  (distance — lower = more similar)")
    print()

    header = f"  {'':<16}" + "".join(f"{c:<12}" for c in class_names[:8])
    print(header)
    for i, c1 in enumerate(class_names):
        row = f"  {c1:<16}"
        for j, c2 in enumerate(class_names):
            d = _z_dist(feats[c1], feats[c2], COMPARE_KEYS)
            dist_matrix[i][j] = d
            if j < 8:
                row += f"{d:<12.4f}"
        print(row)

    if len(class_names) > 8:
        print(f"  ... {len(class_names) - 8} more classes truncated")

    # 2. Nearest neighbors for each class
    print()
    print("  Nearest Neighbors per Class")
    print(f"  {'Class':<16} {'1st':<20} {'2nd':<20} {'3rd':<20}")
    print(f"  {'─'*16} {'─'*20} {'─'*20} {'─'*20}")

    all_neighbors = {}
    for i, c1 in enumerate(class_names):
        dists = [(dist_matrix[i][j], class_names[j]) for j in range(n) if j != i]
        dists.sort()
        neighbors = [(d, c) for d, c in dists[:5]]
        all_neighbors[c1] = neighbors
        row = f"  {c1:<16}"
        for d, c in neighbors[:3]:
            row += f" {c}:{d:<.3f}  "
        print(row)

    # 3. Confusing class pairs (lowest cross-class distances)
    print()
    print("  Most Confusable Class Pairs")
    print(f"  {'Class A':<16} {'Class B':<16} {'Distance':>10}")
    print(f"  {'─'*16} {'─'*16} {'─'*10}")

    pairs = []
    for i in range(n):
        for j in range(i + 1, n):
            pairs.append((dist_matrix[i][j], class_names[i], class_names[j]))
    pairs.sort()

    for d, c1, c2 in pairs[:15]:
        marker = " ← MOST SIMILAR" if pairs.index((d, c1, c2)) == 0 else ""
        print(f"  {c1:<16} {c2:<16} {d:>10.4f}{marker}")

    # 4. Within-class representative files
    print()
    print("  Representative Files per Class")
    print(f"  (closest to class centroid)")
    print()

    for cls_name in class_names:
        profile = profiles.get(cls_name, {})
        reps = profile.get("representative_samples", [])
        centroid_val = profile.get("spectral_centroid", {}).get("mean", 0)
        print(f"  {cls_name} (centroid={centroid_val:.0f}Hz, {profile.get('num_references', 0)} refs):")
        for r in reps[:3]:
            short = r.split("/")[-1] if "/" in r else r
            print(f"    • {short}")
        if reps:
            print()

    # 5. Per-class feature ranges
    print()
    print("  Per-Class Feature Ranges")
    print(f"  {'Class':<16} {'Centroid':>10} {'Low Band':>10} {'High Band':>10} {'Transients':>12} {'Decay ms':>10}")
    print(f"  {'─'*16} {'─'*10} {'─'*10} {'─'*10} {'─'*12} {'─'*10}")

    for cls_name in class_names:
        p = profiles.get(cls_name, {})
        cent = p.get("spectral_centroid", {}).get("mean", 0)
        low = p.get("low_band_energy", {}).get("mean", 0)
        high = p.get("high_band_energy", {}).get("mean", 0)
        trans = p.get("transient_count", {}).get("mean", 0)
        decay = p.get("decay_length_ms", {}).get("mean", 0)
        print(f"  {cls_name:<16} {cent:>8.0f}Hz {low:>9.3f} {high:>9.3f} {trans:>8.1f}  {decay:>7.1f}")

    # 6. Per-class PCA position (if clusters available)
    if cluster_assignments:
        print()
        print("  Cluster Assignment Summary")
        print(f"  {'Class':<16} {'Primary Clusters':>30}")
        print(f"  {'─'*16} {'─'*30}")

        class_clusters = defaultdict(lambda: defaultdict(int))
        for entry in cluster_assignments:
            cls = entry.get("class", "unknown")
            cid = entry.get("cluster", -1)
            class_clusters[cls][cid] += 1

        for cls_name in class_names:
            clusters_dict = class_clusters.get(cls_name, {})
            total = sum(clusters_dict.values())
            sorted_clusters = sorted(clusters_dict.items(), key=lambda x: -x[1])
            cluster_str = ", ".join(f"C{c}({v})" for c, v in sorted_clusters[:4])
            print(f"  {cls_name:<16} {cluster_str:>30}")

    # 7. Report generation
    fmt = args.format
    if fmt == "html":
        _build_html_report(profiles, feats, class_names, dist_matrix, all_neighbors, pairs, args.output)
    elif fmt == "markdown":
        _build_markdown_report(profiles, class_names, dist_matrix, all_neighbors, pairs, args.output)


def _build_html_report(profiles, feats, class_names, dist_matrix, all_neighbors, pairs, output_path):
    """Generate HTML report with tables."""
    lines = []
    lines.append("<!DOCTYPE html><html lang='en'><head><meta charset='UTF-8'>")
    lines.append("<title>cShot Cluster Visualization Report</title>")
    lines.append("<style>")
    lines.append("body{font-family:-apple-system,system-ui,sans-serif;max-width:960px;margin:2em auto;padding:0 1em;background:#0d1117;color:#c9d1d9}")
    lines.append("h1{border-bottom:1px solid #30363d;padding-bottom:.3em}")
    lines.append("h2{margin-top:1.5em;border-bottom:1px solid #21262d;padding-bottom:.2em}")
    lines.append("table{border-collapse:collapse;width:100%;margin:1em 0}")
    lines.append("th,td{border:1px solid #30363d;padding:6px 12px;text-align:left}")
    lines.append("th{background:#161b22}")
    lines.append("tr:nth-child(even){background:#0d1117}")
    lines.append("tr:nth-child(odd){background:#161b22}")
    lines.append(".hot{background:#3b1a1a}.warm{background:#3b2e1a}.cool{background:#1a2e3b}.cold{background:#1a1a3b}")
    lines.append("</style></head><body>")
    lines.append("<h1>Cluster Visualization Report</h1>")
    lines.append(f"<p>Generated: {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}</p>")

    # Feature range table
    lines.append("<h2>Per-Class Feature Profile</h2>")
    lines.append("<table><thead><tr><th>Class</th><th>Refs</th><th>Centroid</th><th>Low Band</th><th>Mid Band</th><th>High Band</th><th>Transients</th><th>Decay</th><th>HPR</th></tr></thead><tbody>")
    for cls_name in class_names:
        p = profiles.get(cls_name, {})
        lines.append(f"<tr><td>{cls_name}</td><td>{p.get('num_references',0)}</td>"
                     f"<td>{p.get('spectral_centroid',{}).get('mean',0):.0f}Hz</td>"
                     f"<td>{p.get('low_band_energy',{}).get('mean',0):.3f}</td>"
                     f"<td>{p.get('mid_band_energy',{}).get('mean',0):.3f}</td>"
                     f"<td>{p.get('high_band_energy',{}).get('mean',0):.3f}</td>"
                     f"<td>{p.get('transient_count',{}).get('mean',0):.1f}</td>"
                     f"<td>{p.get('decay_length_ms',{}).get('mean',0):.1f}ms</td>"
                     f"<td>{p.get('attack_ms',{}).get('mean',0):.1f}ms</td></tr>")
    lines.append("</tbody></table>")

    # Nearest neighbor table
    lines.append("<h2>Nearest Neighbors (Top 3 per Class)</h2>")
    lines.append("<table><thead><tr><th>Class</th><th>#1</th><th>#2</th><th>#3</th></tr></thead><tbody>")
    for cls_name in class_names:
        neighbors = all_neighbors.get(cls_name, [])
        row = f"<tr><td><strong>{cls_name}</strong></td>"
        for d, c in neighbors[:3]:
            row += f"<td>{c} ({d:.4f})</td>"
        row += "</tr>"
        lines.append(row)
    lines.append("</tbody></table>")

    # Similarity matrix as heatmap
    lines.append("<h2>Class Similarity Heatmap</h2>")
    lines.append("<p>Lower values (darker) = more similar</p>")
    lines.append("<table><thead><tr><th></th>")
    for c in class_names:
        lines.append(f"<th>{c}</th>")
    lines.append("</tr></thead><tbody>")
    for i, c1 in enumerate(class_names):
        lines.append(f"<tr><td><strong>{c1}</strong></td>")
        for j, c2 in enumerate(class_names):
            d = dist_matrix[i][j]
            if i == j:
                lines.append(f"<td style='background:#1a3b1a'>0</td>")
            elif d < 0.15:
                lines.append(f"<td style='background:#3b1a1a'>{d:.3f}</td>")
            elif d < 0.3:
                lines.append(f"<td style='background:#3b2e1a'>{d:.3f}</td>")
            elif d < 0.5:
                lines.append(f"<td style='background:#1a2e3b'>{d:.3f}</td>")
            else:
                lines.append(f"<td style='background:#1a1a3b'>{d:.3f}</td>")
        lines.append("</tr>")
    lines.append("</tbody></table>")

    # Confusing pairs
    lines.append("<h2>Most Confusable Pairs</h2>")
    lines.append("<table><thead><tr><th>Rank</th><th>Class A</th><th>Class B</th><th>Distance</th></tr></thead><tbody>")
    for rank, (d, c1, c2) in enumerate(pairs[:20], 1):
        sev = "high" if d < 0.15 else ("medium" if d < 0.25 else "low")
        lines.append(f"<tr><td>{rank}</td><td>{c1}</td><td>{c2}</td><td>{d:.4f} ({sev})</td></tr>")
    lines.append("</tbody></table>")

    lines.append("<hr><p><em>Generated by cShot Cluster Visualization</em></p>")
    lines.append("</body></html>")

    text = "\n".join(lines)
    if output_path:
        Path(output_path).write_text(text)
    else:
        Path(REPO_ROOT / "cluster_report.html").write_text(text)
    print(f"HTML report written to {output_path or REPO_ROOT / 'cluster_report.html'}")


def _build_markdown_report(profiles, class_names, dist_matrix, all_neighbors, pairs, output_path):
    """Generate Markdown report."""
    lines = []
    lines.append("# Cluster Visualization Report")
    lines.append("")
    lines.append(f"**Generated:** {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}")
    lines.append("")

    # Feature range table
    lines.append("## Per-Class Feature Profile")
    lines.append("")
    lines.append("| Class | Refs | Centroid | Low Band | Mid Band | High Band | Transients | Decay |")
    lines.append("|-------|------|----------|----------|----------|-----------|------------|-------|")
    for cls_name in class_names:
        p = profiles.get(cls_name, {})
        lines.append(f"| {cls_name} | {p.get('num_references',0)} | "
                     f"{p.get('spectral_centroid',{}).get('mean',0):.0f}Hz | "
                     f"{p.get('low_band_energy',{}).get('mean',0):.3f} | "
                     f"{p.get('mid_band_energy',{}).get('mean',0):.3f} | "
                     f"{p.get('high_band_energy',{}).get('mean',0):.3f} | "
                     f"{p.get('transient_count',{}).get('mean',0):.1f} | "
                     f"{p.get('decay_length_ms',{}).get('mean',0):.1f}ms |")
    lines.append("")

    # Nearest neighbors
    lines.append("## Nearest Neighbors")
    lines.append("")
    lines.append("| Class | 1st | 2nd | 3rd |")
    lines.append("|-------|-----|-----|-----|")
    for cls_name in class_names:
        neighbors = all_neighbors.get(cls_name, [])
        row = f"| {cls_name} |"
        for d, c in neighbors[:3]:
            row += f" {c} ({d:.4f}) |"
        lines.append(row)
    lines.append("")

    # Confusing pairs
    lines.append("## Most Confusable Pairs")
    lines.append("")
    lines.append("| Rank | Class A | Class B | Distance |")
    lines.append("|------|---------|---------|----------|")
    for rank, (d, c1, c2) in enumerate(pairs[:20], 1):
        lines.append(f"| {rank} | {c1} | {c2} | {d:.4f} |")
    lines.append("")

    text = "\n".join(lines)
    if output_path:
        Path(output_path).write_text(text)
    else:
        Path(REPO_ROOT / "cluster_report.md").write_text(text)
    print(f"Markdown report written to {output_path or REPO_ROOT / 'cluster_report.md'}")
