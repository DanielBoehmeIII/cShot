"""
Week 9-12 — Style Embeddings, Pack Style Space, Style Transfer, Variation Radius
"""

import json
import math
import random
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.features import (
    compute_features, compute_spectral_centroid, compute_spectral_bandwidth,
    compute_hpr, compute_rms, compute_peak, compute_mfccs,
)
from gen.io import read_audio_safe, write_wav
from gen.pack_census import (
    compute_crest_factor, compute_lufs, compute_stereo_width,
    compute_saturation_density, SafeEncoder,
)
from gen.recreate import (
    analyze_source, find_nearest_neighbors, infer_generator,
    build_target_profile, _call_generator, envelope_similarity,
    harmonic_similarity, noise_similarity, compute_overall_score,
)

CENSUS_DIR = REPO_ROOT / "gen" / "census"

STYLE_DIMENSIONS = [
    "brightness", "aggression", "width", "warmth",
    "saturation", "punch", "air", "tonality",
    "complexity", "dynamics",
]

STYLE_DIM_DESCRIPTIONS = {
    "brightness": "Spectral centroid weighted — higher = brighter",
    "aggression": "Transient count + crest factor + saturation — higher = more aggressive",
    "width": "Stereo side/mid ratio — higher = wider",
    "warmth": "Low/mid band ratio — higher = warmer, less bright",
    "saturation": "Harmonic distortion density — higher = more saturated",
    "punch": "Early RMS / attack ratio — higher = punchier transient",
    "air": "High-frequency noise energy — higher = airier",
    "tonality": "Harmonic-to-noise ratio — higher = more tonal, less noisy",
    "complexity": "Number of partials / peaks — higher = more complex timbre",
    "dynamics": "Loudness range — higher = more dynamic",
}


def compute_style_fingerprint(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    features = compute_features(samples, sr)
    crest = compute_crest_factor(samples)
    width = compute_stereo_width(samples)
    sat = compute_saturation_density(samples)
    centroid = features.get("spectral_centroid", 0)
    low = features.get("low_band_energy", 0)
    mid = features.get("mid_band_energy", 0)
    high = features.get("high_band_energy", 0)
    hpr = features.get("hpr", 0.5)
    trans = features.get("transient_count", 0)
    attack = features.get("attack_ms", 10)
    decay = features.get("decay_length_ms", 100)
    early = features.get("early_rms", 1.0)
    zcr = features.get("zero_crossing_rate", 0)
    flux = features.get("spectral_flux_mean", 0)
    lra = features.get("loudness_range", 0)
    peaks = features.get("amplitude_peaks", 1)

    brightness = float(np.clip(centroid / 8000.0, 0, 1))
    aggression = float(np.clip((crest / 10.0) * 0.3 + (trans / 20.0) * 0.3 + sat * 2.0 * 0.4, 0, 1))
    warm = float(np.clip(low / max(mid + high + low, 0.01) * 2.0, 0, 1))
    width_norm = float(np.clip(width, 0, 1))
    sat_norm = float(np.clip(sat * 15.0, 0, 1))
    punch = float(np.clip(early / max(attack + 1, 1) * 50.0, 0, 1))
    air = float(np.clip(high / max(mid + low + high, 0.01) * 3.0, 0, 1))
    tonality = float(np.clip(hpr, 0, 1))
    complexity = float(np.clip((zcr * 0.3 + flux * 2.0 + peaks / 10.0 * 0.3) / 1.5, 0, 1))
    dynamics_val = float(np.clip(lra / 40.0, 0, 1))

    vector = {
        "brightness": round(brightness, 4),
        "aggression": round(aggression, 4),
        "width": round(width_norm, 4),
        "warmth": round(warm, 4),
        "saturation": round(sat_norm, 4),
        "punch": round(punch, 4),
        "air": round(air, 4),
        "tonality": round(tonality, 4),
        "complexity": round(complexity, 4),
        "dynamics": round(dynamics_val, 4),
    }

    return vector


def embed_distance(a: dict, b: dict) -> float:
    d = 0.0
    for dim in STYLE_DIMENSIONS:
        va = a.get(dim, 0)
        vb = b.get(dim, 0)
        d += (va - vb) ** 2
    return math.sqrt(d / len(STYLE_DIMENSIONS))


def compute_pack_centroid(pack_name: str, pack_index: dict) -> dict:
    files = pack_index.get("files", {})
    vectors = []
    for path, entry in files.items():
        if "error" in entry or entry.get("pack", "") != pack_name:
            continue
        v = entry.get("style_embedding")
        if v:
            vectors.append(v)
    if not vectors:
        return None
    centroid = {}
    for dim in STYLE_DIMENSIONS:
        vals = [v[dim] for v in vectors]
        centroid[dim] = round(float(np.mean(vals)), 4)
    return centroid


def compute_style_embeddings_from_index():
    index_path = CENSUS_DIR / "pack_index.json"
    if not index_path.exists():
        print("Error: run pack-census first")
        return
    with open(index_path) as f:
        index = json.load(f)
    files = index.get("files", {})
    print(f"Computing style embeddings for {len(files)} files...")
    count = 0
    for path, entry in files.items():
        if "error" in entry:
            continue
        samples_data = entry.get("duration_ms")
        if not samples_data:
            continue
        wav_path = REPO_ROOT / path
        if not wav_path.exists():
            continue
        result = read_audio_safe(wav_path, mono=True)
        if result is None:
            continue
        samples, sr = result
        fp = compute_style_fingerprint(samples, sr)
        entry["style_embedding"] = fp
        count += 1
        if count % 500 == 0:
            print(f"  {count} embedded...")
    output_path = CENSUS_DIR / "pack_index.json"
    with open(output_path, "w") as f:
        json.dump(index, f, indent=2, cls=SafeEncoder)
    print(f"Updated {output_path} with style embeddings ({count} files)")
    return count


def cmd_embed(args):
    wav_path = Path(args.input)
    if not wav_path.exists():
        print(f"Error: {wav_path} not found")
        return
    result = read_audio_safe(wav_path, mono=True)
    if result is None:
        print("Error: cannot read audio")
        return
    samples, sr = result
    fp = compute_style_fingerprint(samples, sr)
    print(f"Style Embedding: {wav_path.name}")
    print(f"{'Dimension':<15} {'Value':>8}  {'Description'}")
    print("-" * 65)
    for dim in STYLE_DIMENSIONS:
        val = fp[dim]
        bar = "█" * int(val * 20) + "░" * (20 - int(val * 20))
        desc = STYLE_DIM_DESCRIPTIONS.get(dim, "")
        print(f"  {dim:<13} {val:>7.3f}  {bar}  {desc}")
    return fp


def cmd_embed_folder(args):
    folder = Path(args.folder)
    if not folder.exists():
        print(f"Error: {folder} not found")
        return
    wavs = sorted(folder.glob("*.wav"))
    print(f"Embedding {len(wavs)} files from {folder}")
    results = {}
    for wav_path in wavs:
        result = read_audio_safe(wav_path, mono=True)
        if result is None:
            continue
        samples, sr = result
        fp = compute_style_fingerprint(samples, sr)
        results[wav_path.name] = fp
    avg = {}
    for dim in STYLE_DIMENSIONS:
        vals = [r[dim] for r in results.values()]
        avg[dim] = round(float(np.mean(vals)), 4) if vals else 0
    print(f"\nAverage Style Fingerprint ({len(results)} files):")
    for dim in STYLE_DIMENSIONS:
        print(f"  {dim:<15} {avg[dim]:.4f}")
    return {"files": results, "average": avg}


def cmd_pack_style(args):
    index_path = CENSUS_DIR / "pack_index.json"
    if not index_path.exists():
        print("Error: run pack-census first")
        return
    with open(index_path) as f:
        index = json.load(f)
    files = index.get("files", {})
    pack_vectors = defaultdict(list)
    for path, entry in files.items():
        if "error" in entry:
            continue
        v = entry.get("style_embedding")
        if v:
            pack = entry.get("pack", "unknown")
            pack_vectors[pack].append(v)
    centroids = {}
    for pack, vectors in pack_vectors.items():
        if len(vectors) < 3:
            continue
        centroid = {}
        for dim in STYLE_DIMENSIONS:
            centroid[dim] = round(float(np.mean([v[dim] for v in vectors])), 4)
        centroids[pack] = {
            "num_files": len(vectors),
            "centroid": centroid,
            "std": {dim: round(float(np.std([v[dim] for v in vectors])), 4) for dim in STYLE_DIMENSIONS},
        }
    output = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_packs": len(centroids),
        "style_dimensions": STYLE_DIMENSIONS,
        "packs": centroids,
    }
    output_path = CENSUS_DIR / "pack_style_space.json"
    with open(output_path, "w") as f:
        json.dump(output, f, indent=2, cls=SafeEncoder)
    print(f"Pack Style Space: {len(centroids)} packs")
    print(f"\n{'Pack':<45} {'Bright':>7} {'Aggr':>6} {'Width':>6} {'Warm':>6} {'Sat':>6} {'Punch':>6} {'Air':>6} {'Tonal':>6}")
    print("-" * 95)
    for pack in sorted(centroids.keys()):
        c = centroids[pack]["centroid"]
        print(f"  {pack:<43} {c['brightness']:>7.3f} {c['aggression']:>6.3f} {c['width']:>6.3f} {c['warmth']:>6.3f} {c['saturation']:>6.3f} {c['punch']:>6.3f} {c['air']:>6.3f} {c['tonality']:>6.3f}")
    return output


def cmd_style_transfer(args):
    source_path = Path(args.input)
    target_pack = args.style_pack
    count = getattr(args, 'count', 5)
    out_dir = Path(getattr(args, 'out', 'outputs/style_transfer'))
    variation_radius = getattr(args, 'variation_radius', 0.3)
    out_dir.mkdir(parents=True, exist_ok=True)

    if not source_path.exists():
        print(f"Error: {source_path} not found")
        return

    index_path = CENSUS_DIR / "pack_index.json"
    style_path = CENSUS_DIR / "pack_style_space.json"
    if not index_path.exists() or not style_path.exists():
        print("Error: run pack-census and pack-style first")
        return

    with open(index_path) as f:
        index = json.load(f)
    with open(style_path) as f:
        style_data = json.load(f)

    if target_pack not in style_data.get("packs", {}):
        print(f"Error: pack '{target_pack}' not found in style space")
        return

    pack_centroid = style_data["packs"][target_pack]["centroid"]
    pack_std = style_data["packs"][target_pack]["std"]

    print(f"Analyzing source: {source_path.name}")
    analysis = analyze_source(source_path)
    if analysis is None:
        return

    source_fp = compute_style_fingerprint(
        read_audio_safe(source_path, mono=True)[0],
        SAMPLE_RATE,
    )
    print(f"  Source fingerprint: bright={source_fp['brightness']:.3f}, aggression={source_fp['aggression']:.3f}, tonality={source_fp['tonality']:.3f}")
    print(f"  Target pack '{target_pack}' centroid: bright={pack_centroid['brightness']:.3f}, aggression={pack_centroid['aggression']:.3f}, tonality={pack_centroid['tonality']:.3f}")

    neighbors = find_nearest_neighbors(analysis, 8)
    gen_route = infer_generator(analysis, neighbors)

    print(f"  Source family: {analysis['sonic_family']} → generator: {gen_route['generator_family']}/{gen_route['generator_profile']}")

    generated = []
    for i in range(count):
        target = build_target_profile(analysis, gen_route)
        blend = np.clip(variation_radius, 0.1, 1.0)
        for dim in STYLE_DIMENSIONS:
            if dim in ("width", "saturation", "punch", "air", "dynamics"):
                src_val = source_fp.get(dim, 0.5)
                pack_val = pack_centroid.get(dim, 0.5)
                spread = pack_std.get(dim, 0.1)
                noise = random.gauss(0, spread * variation_radius)
                shifted = src_val * (1 - blend) + pack_val * blend + noise
                shifted = float(np.clip(shifted, 0, 1))
                mapped_key = _dim_to_target_param(dim)
                if mapped_key and mapped_key in target:
                    target[mapped_key] = max(0.0, min(1.0, target.get(mapped_key, 0.5) * (0.5 + shifted * 0.5)))

        result, error = _call_generator(
            gen_route["generator_family"],
            gen_route["generator_profile"],
            target, i, out_dir,
        )
        if result is None:
            print(f"  [{i+1}/{count}] Failed: {error}")
            continue

        env_sim = envelope_similarity(analysis["envelope"], result, SAMPLE_RATE)
        harm_sim = harmonic_similarity(analysis["harmonic"], result, SAMPLE_RATE)
        noise_sim = noise_similarity(analysis["noise"], result, SAMPLE_RATE)
        overall = compute_overall_score(env_sim["score"], harm_sim["score"], noise_sim["score"])
        gen_fp = compute_style_fingerprint(result, SAMPLE_RATE)
        style_dist = embed_distance(source_fp, gen_fp)

        out_name = f"transfer_{source_path.stem}_to_{target_pack.replace('/','_').replace(' ','_')}_{i+1:03d}.wav"
        out_path = out_dir / out_name
        write_wav(out_path, result)

        meta = {
            "source": str(source_path),
            "target_pack": target_pack,
            "variation": i + 1,
            "variation_radius": variation_radius,
            "gen_family": gen_route["generator_family"],
            "gen_profile": gen_route["generator_profile"],
            "source_embedding": source_fp,
            "target_centroid": pack_centroid,
            "result_embedding": gen_fp,
            "style_distance_from_source": round(style_dist, 4),
            "envelope_similarity": env_sim,
            "harmonic_similarity": harm_sim,
            "noise_similarity": noise_sim,
            "overall_score": overall,
            "generated": str(out_path),
        }
        meta_path = out_dir / f"{out_name}.json"
        with open(meta_path, "w") as f:
            json.dump(meta, f, indent=2)
        generated.append(meta)
        print(f"  [{i+1}/{count}] {out_name}  style_dist={style_dist:.3f}  overall={overall['overall']:.3f} ({overall['rating']})")

    print(f"\n  Generated {len(generated)}/{count} style transfers to {out_dir}")
    return generated


def _dim_to_target_param(dim: str):
    mapping = {
        "brightness": "centroid_target",
        "saturation": "drive",
        "width": "stereo_width",
        "punch": "attack_ms",
    }
    return mapping.get(dim)


def cmd_variation_chain(args):
    source_path = Path(args.source)
    count = getattr(args, 'count', 5)
    radius_start = getattr(args, 'radius_start', 0.1)
    radius_end = getattr(args, 'radius_end', 1.0)
    out_dir = Path(getattr(args, 'out', 'outputs/variations'))
    out_dir.mkdir(parents=True, exist_ok=True)

    if not source_path.exists():
        print(f"Error: {source_path} not found")
        return

    analysis = analyze_source(source_path)
    if analysis is None:
        return

    source_fp = compute_style_fingerprint(
        read_audio_safe(source_path, mono=True)[0],
        SAMPLE_RATE,
    )
    neighbors = find_nearest_neighbors(analysis, 8)
    gen_route = infer_generator(analysis, neighbors)

    radii = np.linspace(radius_start, radius_end, count)
    generated = []
    for i, r in enumerate(radii):
        target = build_target_profile(analysis, gen_route)
        target_shifted = _apply_variation(target, source_fp, r)
        result, error = _call_generator(
            gen_route["generator_family"],
            gen_route["generator_profile"],
            target_shifted, i, out_dir,
        )
        if result is None:
            continue
        gen_fp = compute_style_fingerprint(result, SAMPLE_RATE)
        env_sim = envelope_similarity(analysis["envelope"], result, SAMPLE_RATE)
        harm_sim = harmonic_similarity(analysis["harmonic"], result, SAMPLE_RATE)
        noise_sim = noise_similarity(analysis["noise"], result, SAMPLE_RATE)
        overall = compute_overall_score(env_sim["score"], harm_sim["score"], noise_sim["score"])
        style_dist = embed_distance(source_fp, gen_fp)

        out_name = f"variation_{source_path.stem}_radius{r:.2f}_{i+1:03d}.wav"
        out_path = out_dir / out_name
        write_wav(out_path, result)
        generated.append({
            "variation": i + 1,
            "radius": round(r, 3),
            "file": str(out_path),
            "style_distance": round(style_dist, 4),
            "overall": overall,
        })
        print(f"  [{i+1}/{count}] r={r:.2f}  style_dist={style_dist:.3f}  overall={overall['overall']:.3f} ({overall['rating']})")

    print(f"\n  Variation chain: {len(generated)} files from radius {radius_start} → {radius_end}")
    return generated


def _apply_variation(target: dict, source_fp: dict, radius: float) -> dict:
    varied = dict(target)
    for key in ["duration_ms", "pitch_hz", "attack_ms", "decay_ms", "stereo_width", "saturation", "drive"]:
        if key in varied:
            spread = radius * 0.5
            varied[key] = varied[key] * (1.0 + random.uniform(-spread, spread))
    return varied


def cmd_pack_style_viz(args):
    style_path = CENSUS_DIR / "pack_style_space.json"
    if not style_path.exists():
        print("Error: run pack-style first")
        return
    with open(style_path) as f:
        data = json.load(f)
    packs = data.get("packs", {})
    lines = []
    lines.append("<!DOCTYPE html><html lang='en'><head>")
    lines.append("<meta charset='UTF-8'><title>Pack Style Space</title>")
    lines.append("<style>*{margin:0;padding:0;box-sizing:border-box}")
    lines.append("body{font-family:-apple-system,sans-serif;background:#0d1117;color:#c9d1d9;padding:20px}")
    lines.append("h1{border-bottom:1px solid #30363d;padding-bottom:8px}")
    lines.append("svg{background:#161b22;border:1px solid #30363d;border-radius:8px}")
    lines.append(".legend{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0}")
    lines.append(".legend-item{padding:4px 10px;background:#161b22;border-radius:4px;font-size:12px}")
    lines.append("table{border-collapse:collapse;margin:12px 0;width:100%}")
    lines.append("th,td{border:1px solid #30363d;padding:6px 10px;font-size:13px;text-align:left}")
    lines.append("th{background:#161b22}")
    lines.append("</style></head><body>")
    lines.append(f"<h1>Pack Style Space — {len(packs)} packs</h1>")
    lines.append("<p>Each pack's centroid in 10-dimensional style space (shown as PCA projection onto first 2 dims)</p>")
    pack_names = sorted(packs.keys())
    if len(pack_names) >= 2:
        matrix = []
        for pn in pack_names:
            c = packs[pn]["centroid"]
            row = [c[dim] for dim in STYLE_DIMENSIONS]
            matrix.append(row)
        X = np.array(matrix, dtype=np.float64)
        X_centered = X - np.mean(X, axis=0)
        U, s, Vt = np.linalg.svd(X_centered, full_matrices=False)
        proj = U[:, :2] * s[:2]
        pca_labels = [f"PC1 ({s[0]**2/np.sum(s**2)*100:.0f}%)", f"PC2 ({s[1]**2/np.sum(s**2)*100:.0f}%)"]
        min_x, max_x = np.min(proj[:, 0]), np.max(proj[:, 0])
        min_y, max_y = np.min(proj[:, 1]), np.max(proj[:, 1])
        x_range = max(max_x - min_x, 0.1)
        y_range = max(max_y - min_y, 0.1)
        pad_x, pad_y = x_range * 0.15, y_range * 0.15
        colors = ["#ff6b6b", "#ffa726", "#66bb6a", "#42a5f5", "#ab47bc", "#26c6da",
                   "#ef5350", "#ffca28", "#8d6e63", "#78909c", "#ec407a", "#ff8a65",
                   "#9ccc65", "#5c6bc0", "#7e57c2"]
        lines.append(f"<svg width='100%' height='500' viewBox='0 0 900 500'>")
        lines.append(f"<text x='20' y='30' fill='#8b949e' font-size='13'>PC Projection</text>")
        lines.append(f"<line x1='80' y1='450' x2='880' y2='450' stroke='#30363d'/>")
        lines.append(f"<line x1='80' y1='50' x2='80' y2='450' stroke='#30363d'/>")
        lines.append(f"<text x='450' y='480' fill='#8b949e' text-anchor='middle' font-size='13'>{pca_labels[0]}</text>")
        lines.append(f"<text x='30' y='250' fill='#8b949e' text-anchor='middle' font-size='13' transform='rotate(-90,30,250)'>{pca_labels[1]}</text>")
        for i, (pn, (px, py)) in enumerate(zip(pack_names, proj)):
            x = 80 + (px - (min_x - pad_x)) / (x_range + 2 * pad_x) * 800
            y = 450 - (py - (min_y - pad_y)) / (y_range + 2 * pad_y) * 400
            x = max(85, min(875, x))
            y = max(55, min(445, y))
            color = colors[i % len(colors)]
            sz = 10 + min(30, packs[pn]["num_files"] / 50)
            lines.append(f"<circle cx='{x:.1f}' cy='{y:.1f}' r='{sz:.1f}' fill='{color}' opacity='0.7'/>")
            lines.append(f"<text x='{x:.1f}' y='{y-10:.1f}' fill='{color}' font-size='10' text-anchor='middle'>{pn[:30]}</text>")
        lines.append("</svg>")
    lines.append("<h2>Pack Style Profiles</h2>")
    lines.append("<table><thead><tr><th>Pack</th><th>Files</th>")
    for dim in STYLE_DIMENSIONS:
        lines.append(f"<th>{dim[:6]}</th>")
    lines.append("</tr></thead><tbody>")
    for pn in sorted(packs.keys()):
        p = packs[pn]
        lines.append(f"<tr><td>{pn}</td><td>{p['num_files']}</td>")
        c = p["centroid"]
        for dim in STYLE_DIMENSIONS:
            lines.append(f"<td>{c[dim]:.3f}</td>")
        lines.append("</tr>")
    lines.append("</tbody></table>")
    lines.append("</body></html>")
    viz_path = CENSUS_DIR / "pack_style_space.html"
    with open(viz_path, "w") as f:
        f.write("\n".join(lines))
    print(f"Written {viz_path}")
