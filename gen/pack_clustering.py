"""
Week 3 — Automatic Sonic Clustering
Cluster sounds by actual audio similarity (ignoring filenames).
Discover natural families: distorted sub, airy clap, metallic perc, synth stab, etc.
"""

import json
import math
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

from gen import REPO_ROOT
from gen.pack_census import SafeEncoder


CLUSTER_FEATURES = [
    "spectral_centroid", "spectral_bandwidth", "zero_crossing_rate",
    "low_band_energy", "mid_band_energy", "high_band_energy",
    "transient_count", "decay_length_ms", "attack_ms",
    "hpr", "pitch_hz", "pitch_confidence",
    "spectral_flux_mean", "spectral_flux_std",
    "crest_factor", "loudness_range", "saturation_density",
    "stereo_width",
    "mfcc_1", "mfcc_2", "mfcc_3", "mfcc_4", "mfcc_5",
    "mfcc_6", "mfcc_7", "mfcc_8", "mfcc_9", "mfcc_10",
]

PCA_COMPONENTS = 15
N_CLUSTERS = 25


def extract_feature_matrix(files: dict, feature_keys: list[str] = None) -> tuple:
    if feature_keys is None:
        feature_keys = CLUSTER_FEATURES

    matrix_rows = []
    file_paths = []
    categories = []
    packs = []

    for path, entry in files.items():
        if "error" in entry:
            continue
        row = []
        valid = True
        for k in feature_keys:
            v = entry.get(k)
            if v is None or (isinstance(v, float) and (math.isnan(v) or math.isinf(v))):
                v = 0.0
            row.append(float(v))
        matrix_rows.append(row)
        file_paths.append(path)
        categories.append(entry.get("category", "other"))
        packs.append(entry.get("pack", "unknown"))

    X = np.array(matrix_rows, dtype=np.float64)
    return X, file_paths, categories, packs


def normalize_matrix(X: np.ndarray) -> tuple:
    mean = np.mean(X, axis=0)
    std = np.std(X, axis=0)
    std[std < 1e-10] = 1.0
    X_norm = (X - mean) / std
    return X_norm, mean, std


def compute_pca(X: np.ndarray, n_components: int = PCA_COMPONENTS) -> tuple:
    n, m = X.shape
    n_components = min(n_components, n, m)
    X_centered = X - np.mean(X, axis=0)
    U, s, Vt = np.linalg.svd(X_centered, full_matrices=False)
    projected = U[:, :n_components] * s[:n_components]
    total_var = np.sum(s ** 2)
    explained = float(np.sum(s[:n_components] ** 2) / max(total_var, 1e-10))
    return projected, Vt[:n_components], explained


def cluster_kmeans(X: np.ndarray, n_clusters: int = N_CLUSTERS, seed: int = 42) -> tuple:
    from scipy.cluster.vq import kmeans, vq
    np.random.seed(seed)
    centroids, distortion = kmeans(X, n_clusters, iter=50)
    labels, _ = vq(X, centroids)
    return labels, centroids


def generate_cluster_descriptions(labels: np.ndarray, categories: list[str],
                                   feature_means: dict, cluster_centroids: np.ndarray,
                                   cluster_idx: int, files_in_cluster: list) -> dict:
    cat_counts = defaultdict(int)
    for cat in categories:
        cat_counts[cat] += 1
    total = len(categories)
    top_cats = sorted(cat_counts.items(), key=lambda x: -x[1])[:5]

    centroid = cluster_centroids[cluster_idx]

    char_tags = []
    if centroid[0] > 10000:
        char_tags.append("bright")
    elif centroid[0] < 500:
        char_tags.append("dark")
    elif centroid[0] < 2000:
        char_tags.append("warm")

    bw_idx = CLUSTER_FEATURES.index("spectral_bandwidth")
    if centroid[bw_idx] > 5000:
        char_tags.append("wide")
    elif centroid[bw_idx] < 500:
        char_tags.append("narrow")

    trans_idx = CLUSTER_FEATURES.index("transient_count")
    if centroid[trans_idx] > 3:
        char_tags.append("multi-hit")
    elif centroid[trans_idx] < 1:
        char_tags.append("sustained")

    hpr_idx = CLUSTER_FEATURES.index("hpr")
    if centroid[hpr_idx] > 0.7:
        char_tags.append("tonal")
    elif centroid[hpr_idx] < 0.3:
        char_tags.append("noisy")

    crest_idx = CLUSTER_FEATURES.index("crest_factor")
    if centroid[crest_idx] > 6:
        char_tags.append("peaky")
    elif centroid[crest_idx] < 3:
        char_tags.append("compressed")

    sat_idx = CLUSTER_FEATURES.index("saturation_density")
    if centroid[sat_idx] > 0.05:
        char_tags.append("saturated")

    attack_idx = CLUSTER_FEATURES.index("attack_ms")
    if centroid[attack_idx] < 5:
        char_tags.append("fast_attack")
    elif centroid[attack_idx] > 50:
        char_tags.append("slow_attack")

    loud_idx = CLUSTER_FEATURES.index("loudness_range")
    if centroid[loud_idx] > 30:
        char_tags.append("dynamic")
    elif centroid[loud_idx] < 15:
        char_tags.append("flat")

    label = " + ".join([top_cats[0][0]] + char_tags[:3])
    label = label.replace("_", " ").title()

    name = f"Cluster {cluster_idx}: {label}"

    return {
        "cluster_id": int(cluster_idx),
        "name": name,
        "size": total,
        "characteristics": char_tags,
        "top_categories": [{"category": c, "count": v, "pct": v / total} for c, v in top_cats],
        "avg_spectral_centroid": float(centroid[0]),
        "avg_transient_count": float(centroid[trans_idx]),
        "avg_hpr": float(centroid[hpr_idx]),
        "avg_crest_factor": float(centroid[crest_idx]),
        "avg_attack_ms": float(centroid[attack_idx]),
        "avg_saturation": float(centroid[sat_idx]),
        "avg_loudness_range": float(centroid[loud_idx]),
    }


def cmd_pack_cluster(args):
    """WEEK 3: Cluster pack sounds by audio similarity. Generates cluster_report.json + visualization."""
    census_dir = REPO_ROOT / "gen" / "census"
    index_path = census_dir / "pack_index.json"

    if not index_path.exists():
        print("Error: run 'pack-census' first")
        return

    with open(index_path) as f:
        census = json.load(f)

    files = census.get("files", {})
    n_clusters = getattr(args, 'n_clusters', None) or N_CLUSTERS
    n_pca = getattr(args, 'pca_dims', None) or PCA_COMPONENTS

    print(f"Clustering {len(files)} files from {census['total_packs']} packs...")

    X, file_paths, categories, packs = extract_feature_matrix(files)
    print(f"Feature matrix: {X.shape[0]} valid files x {X.shape[1]} features")

    if X.shape[0] < n_clusters:
        print(f"Not enough valid files ({X.shape[0]}) for {n_clusters} clusters")
        return

    X_norm, feat_mean, feat_std = normalize_matrix(X)
    X_pca, components, explained_var = compute_pca(X_norm, n_pca)
    print(f"PCA explained variance: {explained_var:.1%}")

    labels, centroids = cluster_kmeans(X_pca, n_clusters)
    n_unique = len(set(labels))
    print(f"K-Means: {n_unique} non-empty clusters")

    cluster_files = defaultdict(list)
    cluster_categories = defaultdict(list)
    cluster_packs = defaultdict(list)
    for i, (label, cat, pack) in enumerate(zip(labels, categories, packs)):
        cluster_files[int(label)].append(file_paths[i])
        cluster_categories[int(label)].append(cat)
        cluster_packs[int(label)].append(pack)

    feature_means = {}
    for i, k in enumerate(CLUSTER_FEATURES):
        feature_means[k] = float(feat_mean[i])

    cluster_centroids_full = np.zeros((n_clusters, len(CLUSTER_FEATURES)))
    for cid in range(n_clusters):
        mask = labels == cid
        if np.sum(mask) > 0:
            cluster_centroids_full[cid] = np.mean(X_norm[mask], axis=0)

    cluster_descriptions = {}
    for cid in sorted(cluster_files.keys()):
        files_in_cluster = cluster_files[cid]
        cats_for_cluster = cluster_categories[cid]
        desc = generate_cluster_descriptions(
            labels, cats_for_cluster, feature_means,
            cluster_centroids_full, cid, files_in_cluster
        )
        cluster_descriptions[cid] = desc

    cross_pack = defaultdict(lambda: defaultdict(int))
    for cid, pk_list in cluster_packs.items():
        for pk in pk_list:
            cross_pack[cid][pk] += 1

    pack_diversity = {}
    for cid, pk_counts in cross_pack.items():
        total = sum(pk_counts.values())
        pack_diversity[int(cid)] = {
            "num_packs": len(pk_counts),
            "packs": dict(sorted(pk_counts.items(), key=lambda x: -x[1])[:10]),
            "entropy": -sum((c / total) * math.log(c / total) for c in pk_counts.values()) / max(math.log(len(pk_counts) + 1), 0.01) if total > 0 else 0,
        }

    pca_coords = X_pca[:, :2].tolist()
    cluster_assignments = []
    for i, (path, label) in enumerate(zip(file_paths, labels)):
        entry = files.get(path, {})
        cluster_assignments.append({
            "file": path,
            "category": categories[i],
            "pack": packs[i],
            "cluster": int(label),
            "cluster_name": cluster_descriptions.get(int(label), {}).get("name", f"Cluster {label}"),
            "pca_x": pca_coords[i][0],
            "pca_y": pca_coords[i][1],
            "semantic_tags": entry.get("semantic_tags", {}),
            "sonic_family": entry.get("sonic_family", "unknown"),
        })

    result = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "config": {
            "n_clusters": n_clusters,
            "n_pca": n_pca,
            "n_features": len(CLUSTER_FEATURES),
            "features_used": CLUSTER_FEATURES,
            "total_files_scanned": len(files),
            "valid_files_clustered": X.shape[0],
        },
        "pca_explained_variance": round(explained_var, 4),
        "cluster_descriptions": cluster_descriptions,
        "pack_diversity": pack_diversity,
        "assignments": cluster_assignments,
    }

    report_path = census_dir / "cluster_report.json"
    with open(report_path, "w") as f:
        json.dump(result, f, indent=2, cls=SafeEncoder)
    print(f"Wrote {report_path}")

    viz_path = census_dir / "cluster_visualization.html"
    _build_visualization(result, viz_path)
    print(f"Wrote {viz_path}")

    print(f"\nCluster Summary:")
    for cid in sorted(cluster_descriptions.keys()):
        desc = cluster_descriptions[cid]
        print(f"  {desc['name']:<45} size={desc['size']:>4}  cats={', '.join(c['category'] for c in desc['top_categories'][:3])}")

    return result


def _build_visualization(data: dict, output_path: Path):
    CLUSTER_COLORS = [
        "#ff6b6b", "#ffa726", "#66bb6a", "#42a5f5", "#ab47bc",
        "#26c6da", "#ef5350", "#ffca28", "#8d6e63", "#78909c",
        "#ec407a", "#ff8a65", "#9ccc65", "#5c6bc0", "#7e57c2",
        "#26a69a", "#e57373", "#ffb74d", "#81c784", "#64b5f6",
        "#ce93d8", "#4dd0e1", "#f06292", "#a1887f", "#90a4ae",
    ]

    lines = []
    lines.append("<!DOCTYPE html>")
    lines.append("<html lang='en'>")
    lines.append("<head>")
    lines.append("<meta charset='UTF-8'>")
    lines.append("<title>cShot Pack Cluster Visualization</title>")
    lines.append("<style>")
    lines.append("*{margin:0;padding:0;box-sizing:border-box}")
    lines.append("body{font-family:-apple-system,system-ui,sans-serif;background:#0d1117;color:#c9d1d9;padding:20px}")
    lines.append("h1,h2,h3{border-bottom:1px solid #30363d;padding-bottom:8px;margin:20px 0 12px}")
    lines.append(".plot{position:relative;width:100%;height:700px;background:#161b22;border:1px solid #30363d;border-radius:8px;overflow:hidden}")
    lines.append(".dot{position:absolute;width:8px;height:8px;border-radius:50%;cursor:pointer;transition:all .15s}")
    lines.append(".dot:hover{width:14px;height:14px;z-index:100;box-shadow:0 0 12px rgba(255,255,255,.4)}")
    lines.append(".legend{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0}")
    lines.append(".legend-item{display:flex;align-items:center;gap:6px;padding:4px 10px;background:#161b22;border-radius:4px;font-size:12px;cursor:pointer}")
    lines.append(".legend-item .swatch{width:12px;height:12px;border-radius:3px}")
    lines.append(".tooltip{position:fixed;display:none;background:#1a1a2e;border:1px solid #30363d;border-radius:6px;padding:8px 12px;font-size:12px;max-width:350px;z-index:999;pointer-events:none}")
    lines.append("table{border-collapse:collapse;width:100%;margin:12px 0}")
    lines.append("th,td{border:1px solid #30363d;padding:6px 10px;text-align:left;font-size:13px}")
    lines.append("th{background:#161b22}")
    lines.append(".stats{display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:12px;margin:12px 0}")
    lines.append(".stat-card{background:#161b22;border:1px solid #30363d;border-radius:6px;padding:12px}")
    lines.append(".stat-card .val{font-size:24px;font-weight:700;color:#58a6ff}")
    lines.append(".stat-card .lbl{font-size:11px;color:#8b949e;margin-top:4px}")
    lines.append(".cluster-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(320px,1fr));gap:12px;margin:12px 0}")
    lines.append(".cluster-card{background:#161b22;border:1px solid #30363d;border-radius:6px;padding:12px}")
    lines.append(".cluster-card .cid{font-size:18px;font-weight:700}")
    lines.append(".cluster-card .cname{color:#8b949e;font-size:13px;margin:4px 0 8px}")
    lines.append(".cluster-card .bar{height:6px;background:#30363d;border-radius:3px;margin:4px 0;overflow:hidden}")
    lines.append(".cluster-card .bar-fill{height:100%;border-radius:3px}")
    lines.append(".filter-bar{display:flex;gap:8px;margin:12px 0;flex-wrap:wrap}")
    lines.append(".filter-bar select,.filter-bar input{padding:6px 10px;background:#161b22;border:1px solid #30363d;border-radius:4px;color:#c9d1d9;font-size:13px}")
    lines.append("</style>")
    lines.append("</head>")
    lines.append("<body>")

    config = data.get("config", {})
    descs = data.get("cluster_descriptions", {})
    assigns = data.get("assignments", [])

    lines.append(f"<h1>Pack Cluster Visualization</h1>")
    lines.append(f"<p>Generated: {data.get('generated_at', '')}</p>")

    lines.append("<div class='stats'>")
    lines.append(f"<div class='stat-card'><div class='val'>{config.get('valid_files_clustered', 0):,}</div><div class='lbl'>Files Clustered</div></div>")
    lines.append(f"<div class='stat-card'><div class='val'>{len(descs)}</div><div class='lbl'>Clusters</div></div>")
    lines.append(f"<div class='stat-card'><div class='val'>{config.get('n_features', 0)}</div><div class='lbl'>Features</div></div>")
    lines.append(f"<div class='stat-card'><div class='val'>{data.get('pca_explained_variance', 0)*100:.1f}%</div><div class='lbl'>PCA Variance Explained</div></div>")
    lines.append("</div>")

    min_x = min((a.get("pca_x", 0) for a in assigns), default=-3)
    max_x = max((a.get("pca_x", 0) for a in assigns), default=3)
    min_y = min((a.get("pca_y", 0) for a in assigns), default=-3)
    max_y = max((a.get("pca_y", 0) for a in assigns), default=3)
    x_range = max(abs(max_x - min_x), 1)
    y_range = max(abs(max_y - min_y), 1)
    pad_x = x_range * 0.1
    pad_y = y_range * 0.1

    lines.append("<div class='legend' id='legend'>")
    for cid in sorted(descs.keys()):
        d = descs[cid]
        color = CLUSTER_COLORS[cid % len(CLUSTER_COLORS)]
        lines.append(f"<div class='legend-item' onclick='toggleCluster({cid})' id='legend-{cid}'>")
        lines.append(f"<div class='swatch' style='background:{color}'></div>")
        lines.append(f"<span>Cluster {cid}: {d['name'][:30]}</span>")
        lines.append(f"<span style='color:#8b949e'>({d['size']})</span>")
        lines.append("</div>")
    lines.append("</div>")

    lines.append("<div class='filter-bar'>")
    lines.append("<select id='categoryFilter' onchange='applyFilter()'>")
    lines.append("<option value='all'>All Categories</option>")
    cats = sorted(set(a.get("category", "other") for a in assigns))
    for cat in cats:
        lines.append(f"<option value='{cat}'>{cat}</option>")
    lines.append("</select>")
    lines.append("<select id='clusterFilter' onchange='applyFilter()'>")
    lines.append("<option value='all'>All Clusters</option>")
    for cid in sorted(descs.keys()):
        lines.append(f"<option value='{cid}'>Cluster {cid}</option>")
    lines.append("</select>")
    lines.append("<input type='text' id='searchBox' placeholder='Search files...' oninput='applyFilter()'>")
    lines.append("</div>")

    lines.append("<div class='plot' id='plot' onmousemove='moveTooltip(event)'>")
    lines.append("<svg width='100%' height='100%' style='position:absolute;top:0;left:0;pointer-events:none'>")
    lines.append("<defs>")
    lines.append("<filter id='glow'><feGaussianBlur stdDeviation='2' result='blur'/><feMerge><feMergeNode in='blur'/><feMergeNode in='SourceGraphic'/></feMerge></filter>")
    lines.append("</defs>")
    lines.append("</svg>")
    for a in assigns:
        cid = a.get("cluster", 0)
        color = CLUSTER_COLORS[cid % len(CLUSTER_COLORS)]
        x_pct = (a.get("pca_x", 0) - (min_x - pad_x)) / (x_range + 2 * pad_x) * 100
        y_pct = (a.get("pca_y", 0) - (min_y - pad_y)) / (y_range + 2 * pad_y) * 100
        x_pct = max(0.5, min(99.5, x_pct))
        y_pct = max(0.5, min(99.5, y_pct))
        name = Path(a.get("file", "")).name
        cat = a.get("category", "")
        pack = a.get("pack", "")
        tags = a.get("semantic_tags", {})
        char = ", ".join(tags.get("sound_character", [])[:5])
        family = a.get("sonic_family", "")

        lines.append(f"<div class='dot' style='left:{x_pct:.1f}%;top:{y_pct:.1f}%;background:{color}' "
                     f"onmouseenter='showTooltip(event,`{name}`,`Cluster {cid}`,`{cat}`,`{pack}`,`{char}`,`{family}`)' "
                     f"onmouseleave='hideTooltip()' data-cluster='{cid}' data-category='{cat}' "
                     f"data-name='{name.lower()}'></div>")
    lines.append("</div>")

    lines.append("<div class='tooltip' id='tooltip'>")
    lines.append("<div id='tt-name' style='font-weight:700;margin-bottom:4px'></div>")
    lines.append("<div id='tt-cluster'></div>")
    lines.append("<div id='tt-category'></div>")
    lines.append("<div id='tt-pack'></div>")
    lines.append("<div id='tt-character'></div>")
    lines.append("<div id='tt-family'></div>")
    lines.append("</div>")

    lines.append("<h2>Cluster Profiles</h2>")
    lines.append("<div class='cluster-grid'>")
    for cid in sorted(descs.keys()):
        d = descs[cid]
        color = CLUSTER_COLORS[cid % len(CLUSTER_COLORS)]
        lines.append("<div class='cluster-card'>")
        lines.append(f"<div class='cid' style='color:{color}'>Cluster {cid}</div>")
        lines.append(f"<div class='cname'>{d.get('name', '')}</div>")
        lines.append(f"<div>Size: <strong>{d['size']}</strong></div>")
        lines.append(f"<div>Characteristics: <em>{', '.join(d.get('characteristics', []))}</em></div>")
        lines.append("<div style='margin-top:8px'>")
        for tc in d.get("top_categories", [])[:6]:
            pct = tc["pct"]
            bar_w = int(pct * 100)
            lines.append(f"<div style='display:flex;justify-content:space-between;font-size:11px;margin:2px 0'>")
            lines.append(f"<span>{tc['category']}</span><span>{tc['count']} ({pct*100:.0f}%)</span>")
            lines.append(f"</div>")
            lines.append(f"<div class='bar'><div class='bar-fill' style='width:{bar_w}%;background:{color}'></div></div>")
        lines.append("</div>")
        lines.append(f"<div style='margin-top:8px;font-size:11px;color:#8b949e'>")
        lines.append(f"Centroid: {d.get('avg_spectral_centroid',0):.0f}Hz | HPR: {d.get('avg_hpr',0):.2f} | ")
        lines.append(f"Crest: {d.get('avg_crest_factor',0):.1f} | Attack: {d.get('avg_attack_ms',0):.0f}ms")
        lines.append("</div>")
        lines.append("</div>")
    lines.append("</div>")

    lines.append("<h2>Cluster Assignments (sample)</h2>")
    lines.append("<table><thead><tr><th>File</th><th>Cluster</th><th>Category</th><th>Family</th><th>Pack</th></tr></thead><tbody>")
    for a in assigns[:200]:
        name = Path(a.get("file", "")).name
        lines.append(f"<tr><td>{name}</td><td>{a.get('cluster_name','')}</td><td>{a.get('category','')}</td><td>{a.get('sonic_family','')}</td><td>{a.get('pack','')}</td></tr>")
    if len(assigns) > 200:
        lines.append(f"<tr><td colspan='5'>... and {len(assigns) - 200} more</td></tr>")
    lines.append("</tbody></table>")

    lines.append("<script>")
    lines.append("const tooltip=document.getElementById('tooltip');")
    lines.append("function showTooltip(e, name, cluster, cat, pack, char, family){")
    lines.append("document.getElementById('tt-name').textContent=name;")
    lines.append("document.getElementById('tt-cluster').textContent='Cluster: '+cluster;")
    lines.append("document.getElementById('tt-category').textContent='Category: '+cat;")
    lines.append("document.getElementById('tt-pack').textContent='Pack: '+pack;")
    lines.append("document.getElementById('tt-character').textContent='Character: '+char;")
    lines.append("document.getElementById('tt-family').textContent='Family: '+family;")
    lines.append("tooltip.style.display='block';}")
    lines.append("function hideTooltip(){tooltip.style.display='none';}")
    lines.append("function moveTooltip(e){tooltip.style.left=(e.clientX+15)+'px';tooltip.style.top=(e.clientY-10)+'px';}")
    lines.append("function toggleCluster(cid){")
    lines.append("const dots=document.querySelectorAll(`.dot[data-cluster='${cid}']`);")
    lines.append("const leg=document.getElementById(`legend-${cid}`);")
    lines.append("const hidden=dots.length>0&&dots[0].style.opacity==='0.15';")
    lines.append("dots.forEach(d=>{d.style.opacity=hidden?'1':'0.15';});")
    lines.append("if(leg)leg.style.opacity=hidden?'1':'0.5';}")
    lines.append("function applyFilter(){")
    lines.append("const catF=document.getElementById('categoryFilter').value;")
    lines.append("const cluF=document.getElementById('clusterFilter').value;")
    lines.append("const search=document.getElementById('searchBox').value.toLowerCase();")
    lines.append("document.querySelectorAll('.dot').forEach(d=>{")
    lines.append("let show=(catF==='all'||d.dataset.category===catF)")
    lines.append("&&(cluF==='all'||d.dataset.cluster===cluF)")
    lines.append("&&(!search||d.dataset.name.includes(search));")
    lines.append("d.style.display=show?'block':'none';")
    lines.append("});}")
    lines.append("</script>")

    lines.append("</body></html>")

    output_path.write_text("\n".join(lines))
