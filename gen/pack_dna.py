"""
Week 4 — Pack DNA Analysis
Compute aggregate "style fingerprints" per pack.
"""

import json
import math
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

from gen import REPO_ROOT
from gen.pack_census import SafeEncoder


DNA_KEYS = [
    "loudness_profile", "transient_aggressiveness", "saturation_density",
    "stereo_width_profile", "spectral_balance", "tonal_noise_ratio",
]


def compute_pack_dna(files: dict) -> dict:
    pack_files = defaultdict(list)
    for path, entry in files.items():
        if "error" in entry:
            continue
        pack = entry.get("pack", "unknown")
        pack_files[pack].append(entry)

    dna = {}
    for pack, entries in pack_files.items():
        if len(entries) < 3:
            continue

        spectral_centroids = [e.get("spectral_centroid", 0) for e in entries]
        low_band = [e.get("low_band_energy", 0) for e in entries]
        mid_band = [e.get("mid_band_energy", 0) for e in entries]
        high_band = [e.get("high_band_energy", 0) for e in entries]
        trans_counts = [e.get("transient_count", 0) for e in entries]
        trans_strengths = [e.get("transient_strength", 0) for e in entries]
        hprs = [e.get("hpr", 0.5) for e in entries]
        crests = [e.get("crest_factor", 0) for e in entries]
        lufs_vals = [e.get("lufs_integrated", -100) for e in entries]
        widths = [e.get("stereo_width", 0) for e in entries]
        sats = [e.get("saturation_density", 0) for e in entries]
        attacks = [e.get("attack_ms", 0) for e in entries]
        decays = [e.get("decay_length_ms", 0) for e in entries]
        lras = [e.get("loudness_range", 0) for e in entries]

        loudness_profile = {
            "mean_lufs": float(np.mean(lufs_vals)),
            "std_lufs": float(np.std(lufs_vals)),
            "mean_crest": float(np.mean(crests)),
            "std_crest": float(np.std(crests)),
            "mean_loudness_range": float(np.mean(lras)),
            "dynamic_range": float(np.max(lufs_vals) - np.min(lufs_vals)) if lufs_vals else 0,
        }

        transient_aggressiveness = {
            "mean_transient_count": float(np.mean(trans_counts)),
            "mean_transient_strength": float(np.mean(trans_strengths)),
            "punch_score": float(np.mean([a for a in attacks if a > 0]) / max(np.mean(decays), 1) * 100) if attacks else 0,
            "percussive_ratio": sum(1 for t in trans_counts if t >= 1) / max(len(trans_counts), 1),
        }

        sat_density = {
            "mean_saturation": float(np.mean(sats)),
            "std_saturation": float(np.std(sats)),
            "high_saturation_ratio": sum(1 for s in sats if s > 0.02) / max(len(sats), 1),
        }

        valid_widths = [w for w in widths if w > 0.001]
        stereo_profile = {
            "mean_width": float(np.mean(valid_widths)) if valid_widths else 0,
            "std_width": float(np.std(valid_widths)) if valid_widths else 0,
            "mono_ratio": sum(1 for w in widths if w < 0.01) / max(len(widths), 1),
            "stereo_sample_max": widths[-1] if widths else 0,
        }

        spectral_balance = {
            "mean_centroid": float(np.mean(spectral_centroids)),
            "std_centroid": float(np.std(spectral_centroids)),
            "low_ratio": float(np.mean(low_band)),
            "mid_ratio": float(np.mean(mid_band)),
            "high_ratio": float(np.mean(high_band)),
            "brightness_score": float(np.mean(high_band) / max(np.mean(low_band), 0.001)),
        }

        tonal_noise_ratio = {
            "mean_hpr": float(np.mean(hprs)),
            "std_hpr": float(np.std(hprs)),
            "tonal_ratio": sum(1 for h in hprs if h > 0.6) / max(len(hprs), 1),
            "noisy_ratio": sum(1 for h in hprs if h < 0.3) / max(len(hprs), 1),
        }

        mean_attack = float(np.mean(attacks)) if attacks else 0
        mean_decay = float(np.mean(decays)) if decays else 0

        dna[pack] = {
            "num_files": len(entries),
            "packs_included": [pack],
            "loudness_profile": loudness_profile,
            "transient_aggressiveness": transient_aggressiveness,
            "saturation_density": sat_density,
            "stereo_width_profile": stereo_profile,
            "spectral_balance": spectral_balance,
            "tonal_noise_ratio": tonal_noise_ratio,
            "envelope_profile": {
                "mean_attack_ms": mean_attack,
                "mean_decay_ms": mean_decay,
                "attack_decay_ratio": mean_attack / max(mean_decay, 1),
            },
        }

    return dna


def compute_pack_similarity(dna: dict) -> tuple:
    packs = list(dna.keys())
    n = len(packs)
    if n < 2:
        return {}, []

    feature_pairs = [
        ("loudness_profile", "mean_lufs"),
        ("loudness_profile", "mean_crest"),
        ("transient_aggressiveness", "mean_transient_count"),
        ("transient_aggressiveness", "mean_transient_strength"),
        ("saturation_density", "mean_saturation"),
        ("stereo_width_profile", "mean_width"),
        ("spectral_balance", "mean_centroid"),
        ("spectral_balance", "low_ratio"),
        ("spectral_balance", "high_ratio"),
        ("tonal_noise_ratio", "mean_hpr"),
        ("envelope_profile", "mean_attack_ms"),
        ("envelope_profile", "mean_decay_ms"),
    ]

    matrix = np.zeros((n, len(feature_pairs)))
    for i, pack in enumerate(packs):
        pdna = dna[pack]
        for j, (section, key) in enumerate(feature_pairs):
            val = pdna.get(section, {}).get(key, 0)
            matrix[i, j] = float(val)

    means = np.mean(matrix, axis=0)
    stds = np.std(matrix, axis=0)
    stds[stds < 1e-10] = 1.0
    matrix_norm = (matrix - means) / stds

    similarity_matrix = {}
    pairs_list = []
    for i in range(n):
        for j in range(i + 1, n):
            dist = float(np.sqrt(np.sum((matrix_norm[i] - matrix_norm[j]) ** 2)))
            sim = float(1.0 / (1.0 + dist))
            similarity_matrix[f"{packs[i]}__{packs[j]}"] = {
                "pack_a": packs[i],
                "pack_b": packs[j],
                "distance": round(dist, 4),
                "similarity": round(sim, 4),
            }
            pairs_list.append((sim, packs[i], packs[j]))

    pairs_list.sort(reverse=True)

    fingerprints = {}
    for pack in packs:
        pdna = dna[pack]
        fp = []
        for section, key in feature_pairs:
            val = pdna.get(section, {}).get(key, 0)
            fp.append(round(float(val), 4))
        fingerprints[pack] = {
            "fingerprint": fp,
            "fingerprint_labels": [f"{s}/{k}" for s, k in feature_pairs],
        }

    return similarity_matrix, pairs_list, fingerprints


def cmd_pack_dna(args):
    """WEEK 4: Compute pack DNA fingerprints from pack_index.json."""
    census_dir = REPO_ROOT / "gen" / "census"
    index_path = census_dir / "pack_index.json"

    if not index_path.exists():
        print("Error: run 'pack-census' first")
        return

    with open(index_path) as f:
        census = json.load(f)

    files = census.get("files", {})
    print(f"Computing DNA for {census['total_packs']} packs")

    dna = compute_pack_dna(files)
    print(f"  {len(dna)} packs with sufficient data")

    sim_matrix, similar_pairs, fingerprints = compute_pack_similarity(dna)
    similar_pairs = [(s, a, b) for s, a, b in similar_pairs if s > 0.3]

    result = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_packs_analyzed": len(dna),
        "pack_dna": dna,
        "pack_similarity": sim_matrix,
        "most_similar_pairs": [
            {"pack_a": a, "pack_b": b, "similarity": round(s, 4)}
            for s, a, b in similar_pairs[:20]
        ],
        "fingerprints": fingerprints,
    }

    output_path = census_dir / "pack_dna.json"
    with open(output_path, "w") as f:
        json.dump(result, f, indent=2, cls=SafeEncoder)
    print(f"Wrote {output_path}")

    print(f"\nPack DNA Summary:")
    print(f"{'Pack':<45} {'Files':>6} {'LUFS':>8} {'Centroid':>10} {'Width':>8} {'HPR':>6} {'Transients':>10}")
    print("-" * 95)
    sorted_packs = sorted(dna.keys(), key=lambda p: dna[p].get("spectral_balance", {}).get("mean_centroid", 0))
    for pack in sorted_packs:
        pd = dna[pack]
        nf = pd["num_files"]
        luf = pd["loudness_profile"]["mean_lufs"]
        cent = pd["spectral_balance"]["mean_centroid"]
        w = pd["stereo_width_profile"]["mean_width"]
        hpr = pd["tonal_noise_ratio"]["mean_hpr"]
        tc = pd["transient_aggressiveness"]["mean_transient_count"]
        luf_str = f"{luf:.1f}" if luf > -90 else "silent"
        print(f"  {pack:<43} {nf:>6} {luf_str:>8} {cent:>8.0f}Hz {w:>7.3f} {hpr:>5.2f} {tc:>8.1f}")

    if similar_pairs:
        print(f"\nMost Similar Pack Pairs:")
        for s, a, b in similar_pairs[:10]:
            print(f"  {a:<40} ↔ {b:<40} (sim={s:.3f})")

    return result


def cmd_pack_dna_report(args):
    """Print a detailed pack DNA report."""
    census_dir = REPO_ROOT / "gen" / "census"
    path = census_dir / "pack_dna.json"
    if not path.exists():
        print("Run 'pack-dna' first")
        return

    with open(path) as f:
        data = json.load(f)

    dna = data.get("pack_dna", {})
    print(f"# Pack DNA Report — {len(dna)} packs\n")

    for pack_name in sorted(dna.keys()):
        pd = dna[pack_name]
        print(f"## {pack_name} ({pd['num_files']} files)")
        print()

        lp = pd["loudness_profile"]
        print(f"- **Loudness:** LUFS={lp['mean_lufs']:.1f} (±{lp['std_lufs']:.1f}), "
              f"Crest={lp['mean_crest']:.1f}, Range={lp['dynamic_range']:.1f}")

        ta = pd["transient_aggressiveness"]
        print(f"- **Transients:** {ta['mean_transient_count']:.1f} avg, "
              f"Strength={ta['mean_transient_strength']:.1f}, "
              f"Percussive={ta['percussive_ratio']*100:.0f}%")

        sd = pd["saturation_density"]
        print(f"- **Saturation:** {sd['mean_saturation']:.4f} avg, "
              f"{sd['high_saturation_ratio']*100:.0f}% highly saturated")

        sw = pd["stereo_width_profile"]
        print(f"- **Stereo:** Width={sw['mean_width']:.3f}, "
              f"Mono={sw['mono_ratio']*100:.0f}%")

        sb = pd["spectral_balance"]
        print(f"- **Spectrum:** Centroid={sb['mean_centroid']:.0f}Hz, "
              f"L/M/H={sb['low_ratio']:.2f}/{sb['mid_ratio']:.2f}/{sb['high_ratio']:.2f}")

        tn = pd["tonal_noise_ratio"]
        print(f"- **Tonal/Noise:** HPR={tn['mean_hpr']:.2f}, "
              f"Tonal={tn['tonal_ratio']*100:.0f}%, Noisy={tn['noisy_ratio']*100:.0f}%")

        ep = pd["envelope_profile"]
        print(f"- **Envelope:** Attack={ep['mean_attack_ms']:.1f}ms, "
              f"Decay={ep['mean_decay_ms']:.0f}ms")
        print()

    similar = data.get("most_similar_pairs", [])
    print("## Most Similar Pack Pairs")
    for pair in similar[:15]:
        print(f"- {pair['pack_a']} ↔ {pair['pack_b']} (sim={pair['similarity']:.3f})")
