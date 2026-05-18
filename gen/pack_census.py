"""
Week 1 — Full Pack Census
Scan all packs, compute comprehensive features, produce pack_index.json, pack_stats.md, family_distribution.json
"""

import json
import math
import sys
import time
from collections import defaultdict
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, PACKS_DIR, SUPPORTED_EXTS, SAMPLE_RATE
from gen.io import read_audio_safe
from gen.features import (
    compute_features,
    compute_spectral_centroid,
    compute_spectral_bandwidth,
    compute_hpr,
    compute_spectral_flux,
    compute_rms,
    compute_peak,
)


def compute_crest_factor(samples: np.ndarray) -> float:
    peak = compute_peak(samples)
    rms = compute_rms(samples)
    if rms < 1e-10:
        return 0.0
    return float(peak / rms)


def safe_float(v, default=0.0):
    if v is None or (isinstance(v, float) and (math.isinf(v) or math.isnan(v))):
        return default
    return float(v)


class SafeEncoder(json.JSONEncoder):
    def default(self, obj):
        return str(obj)
    def encode(self, o):
        return super().encode(_sanitize(o))
    def iterencode(self, o, _one_shot=False):
        return super().iterencode(_sanitize(o), _one_shot)


def _sanitize(obj):
    if isinstance(obj, dict):
        return {k: _sanitize(v) for k, v in obj.items()}
    elif isinstance(obj, list):
        return [_sanitize(v) for v in obj]
    elif isinstance(obj, float):
        if math.isinf(obj) or math.isnan(obj):
            return 0.0
        return obj
    return obj


def compute_lufs(samples: np.ndarray, sr: int = SAMPLE_RATE) -> float:
    """Simplified LUFS (ITU-R BS.1770-4) measurement.
    Uses pre-filter + RLB weighting + mean power integration.
    Returns integrated LUFS (dB)."""
    if len(samples) < sr:
        return -100.0

    if samples.ndim == 1:
        channels = [samples]
    else:
        channels = [samples[:, c] for c in range(samples.shape[1])]

    g = [1.0, 1.0, 1.0, 1.0, 1.0]
    g = g[:len(channels)]

    from scipy import signal as sp_signal

    b, a = _pre_filter_coeffs(sr)
    total_power = 0.0
    total_g = 0.0
    for ch_idx, ch in enumerate(channels):
        filtered = sp_signal.lfilter(b, a, ch)
        mean_sq = np.mean(filtered ** 2)
        total_power += g[ch_idx] * mean_sq
        total_g += g[ch_idx]

    if total_g <= 0 or total_power <= 1e-12:
        return -100.0

    lufs = -0.691 + 10.0 * math.log10(total_power / total_g)
    return float(lufs)


def _pre_filter_coeffs(sr: int):
    """ITU-R BS.1770-4 pre-filter + RLB weighting as a single IIR."""
    from scipy import signal as sp_signal

    f0 = 38.0
    Q = 0.5
    w0 = 2.0 * math.pi * f0 / sr
    alpha = math.sin(w0) / (2.0 * Q)

    b0 = 1.0
    b1 = -2.0 * math.cos(w0)
    b2 = 1.0
    a0 = 1.0 + alpha
    a1 = -2.0 * math.cos(w0)
    a2 = 1.0 - alpha

    b_pre = [b0 / a0, b1 / a0, b2 / a0]
    a_pre = [1.0, a1 / a0, a2 / a0]

    f0_rlb = 168.34
    Q_rlb = 0.691
    w0_rlb = 2.0 * math.pi * f0_rlb / sr
    alpha_rlb = math.sin(w0_rlb) / (2.0 * Q_rlb)

    b_rlb = [1.0, 0.0, -1.0]
    a_rlb = [1.0 + alpha_rlb, -2.0 * math.cos(w0_rlb), 1.0 - alpha_rlb]

    b_combined = np.convolve(b_pre, b_rlb)
    a_combined = np.convolve(a_pre, a_rlb)

    return b_combined, a_combined


def compute_stereo_width(samples: np.ndarray) -> float:
    """Stereo width as RMS(mid-side ratio)."""
    if samples.ndim != 2 or samples.shape[1] < 2:
        return 0.0
    L = samples[:, 0]
    R = samples[:, 1]
    mid = (L + R) / 2.0
    side = (L - R) / 2.0
    mid_rms = np.sqrt(np.mean(mid ** 2))
    side_rms = np.sqrt(np.mean(side ** 2))
    if mid_rms < 1e-10:
        return 0.0
    return float(side_rms / mid_rms)


def compute_loudness_range(samples: np.ndarray, sr: int = SAMPLE_RATE) -> float:
    """Simplified loudness range: ratio of max short-term loudness to noise floor."""
    if len(samples) < sr // 2:
        return 0.0
    frame_len = int(0.1 * sr)
    hop = frame_len // 2
    levels = []
    for i in range(0, len(samples) - frame_len, hop):
        frame = samples[i:i + frame_len]
        rms = np.sqrt(np.mean(frame ** 2))
        if rms > 1e-10:
            levels.append(20.0 * math.log10(rms))
    if len(levels) < 3:
        return 0.0
    levels.sort()
    top = np.mean(levels[-int(len(levels) * 0.1):])
    bottom = np.mean(levels[:int(len(levels) * 0.1)])
    return float(top - bottom)


def compute_saturation_density(samples: np.ndarray) -> float:
    """Estimate saturation density from high-frequency harmonic content."""
    if len(samples) < 1024:
        return 0.0
    spectrum = np.abs(np.fft.rfft(samples[:4096]))
    total = np.sum(spectrum)
    if total < 1e-10:
        return 0.0
    harm = np.sum(spectrum[2::2])
    return float(harm / total)


def discover_pack_folders() -> list[tuple[str, Path]]:
    """Discover all pack directories under both Packs roots."""
    packs_root = REPO_ROOT / "Packs"
    tauri_packs = PACKS_DIR
    folders = []

    roots_to_check = []
    if packs_root.exists():
        roots_to_check.append(("Packs", packs_root))
    if tauri_packs.exists():
        roots_to_check.append(("src-tauri/Packs", tauri_packs))

    def has_audio(subdir: Path) -> bool:
        for ext in SUPPORTED_EXTS:
            if list(subdir.rglob(f"*{ext}")) or list(subdir.rglob(f"*{ext.upper()}")):
                return True
        return False

    for prefix, root in roots_to_check:
        if root == tauri_packs:
            for sub in sorted(root.iterdir()):
                if sub.is_dir() and not sub.name.startswith(".") and not sub.name.endswith(".zip") and not sub.name == "__MACOSX":
                    if has_audio(sub):
                        folders.append((f"src-tauri/Packs/{sub.name}", sub))
        else:
            folders.append((prefix, root))
            for sub in sorted(root.iterdir()):
                if sub.is_dir() and not sub.name.startswith("."):
                    wavs = list(sub.rglob("*.wav"))
                    if wavs:
                        folders.append((f"Packs/{sub.name}", sub))

    return folders


def collect_audio_files(pack_folder: Path) -> list[Path]:
    files = []
    for ext in SUPPORTED_EXTS:
        for f in pack_folder.rglob(f"*{ext}"):
            if "__MACOSX" not in f.parts:
                files.append(f)
        for f in pack_folder.rglob(f"*{ext.upper()}"):
            if "__MACOSX" not in f.parts:
                files.append(f)
    return sorted(set(files))


def compute_pack_census_features(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    width = compute_stereo_width(samples)

    if samples.ndim == 2:
        samples = samples.mean(axis=1)

    base = compute_features(samples, sr)

    crest = compute_crest_factor(samples)
    lufs = compute_lufs(samples, sr)
    lra = compute_loudness_range(samples, sr)
    sat = compute_saturation_density(samples)

    base["crest_factor"] = crest
    base["lufs_integrated"] = lufs
    base["stereo_width"] = width
    base["loudness_range"] = lra
    base["saturation_density"] = sat

    return base


def infer_family(path: Path, pack_root: Path) -> str:
    """Infer the family from subdirectory structure."""
    try:
        rel = path.parent.relative_to(pack_root)
        parts = rel.parts
        if parts:
            return parts[0]
    except ValueError:
        pass
    return "root"


def classify_category(family: str, name: str) -> str:
    """Map family/filename to a broader sound category."""
    fl = family.lower()
    nl = name.lower()

    if "kick" in nl and "snare" not in nl:
        return "kick"
    if "snare" in nl:
        return "snare"
    if "clap" in nl:
        return "clap"
    if "hat" in nl or "hihat" in nl or "hi-hat" in nl or "hh" in nl or "oh" in nl:
        if "open" in nl:
            return "open_hat"
        return "closed_hat"
    if "808" in nl or "sub" in nl:
        return "808"
    if "bass" in nl or "bass" in fl:
        return "bass"
    if "guitar" in nl or "guitar" in fl:
        return "guitar"
    if "piano" in nl or "keys" in fl or "keyboard" in fl:
        return "piano"
    if "synth" in nl or "synth" in fl:
        return "synth"
    if "fx" in nl or "fx" in fl or "impact" in nl or "sfx" in fl:
        return "fx"
    if "crash" in nl or "cymbal" in nl or "crash" in fl or "cymbal" in fl:
        return "cymbal"
    if "perc" in nl or "perc" in fl or "rim" in nl or "tom" in nl:
        return "percussion"
    if "vox" in nl or "vocal" in nl or "vocal" in fl:
        return "vocal"
    if "loop" in fl or "loop" in nl:
        return "loop"
    if "riser" in nl or "rise" in fl or "riser" in fl:
        return "riser"
    if "flac" in fl or "instrument" in fl:
        return "instrument"
    return "other"


def cmd_pack_census(args):
    """WEEK 1: Full Pack Census — scan all packs, build index, stats, family distribution."""
    output_dir = Path(args.output) if args.output else REPO_ROOT / "gen" / "census"
    output_dir.mkdir(parents=True, exist_ok=True)

    pack_folders = discover_pack_folders()
    print(f"Found {len(pack_folders)} pack folders")

    all_files = []
    for pack_name, pack_root in pack_folders:
        files = collect_audio_files(pack_root)
        for f in files:
            all_files.append((pack_name, pack_root, f))

    print(f"Total audio files found: {len(all_files)}")

    index = {}
    errors = []
    family_counts = defaultdict(lambda: defaultdict(int))
    pack_file_lists = defaultdict(list)

    analyzed = 0
    total = len(all_files)
    start = time.time()

    for pack_name, pack_root, fpath in all_files:
        rel_path = fpath.relative_to(REPO_ROOT)
        family = infer_family(fpath, pack_root)
        category = classify_category(family, fpath.stem)
        ext = fpath.suffix.lower()
        name = fpath.stem
        size_bytes = fpath.stat().st_size

        entry = {
            "file_path": str(rel_path),
            "pack": pack_name,
            "family": family,
            "category": category,
            "filename": name,
            "extension": ext,
            "size_bytes": size_bytes,
            "sample_rate": SAMPLE_RATE,
        }

        result = read_audio_safe(fpath, mono=False)
        if result is None:
            entry["error"] = "could_not_read"
            entry["duration_ms"] = 0.0
            entry["rms"] = 0.0
            entry["peak"] = 0.0
            entry["crest_factor"] = 0.0
            entry["lufs_integrated"] = -float("inf")
            entry["spectral_centroid"] = 0.0
            entry["spectral_bandwidth"] = 0.0
            entry["transient_count"] = 0
            entry["hpr"] = 0.5
            entry["spectral_flux_mean"] = 0.0
            entry["spectral_flux_std"] = 0.0
            entry["stereo_width"] = 0.0
            entry["zero_crossing_rate"] = 0.0
            entry["attack_ms"] = 0.0
            entry["decay_length_ms"] = 0.0
            entry["pitch_hz"] = 0.0
            entry["pitch_confidence"] = 0.0
            entry["loudness_range"] = 0.0
            entry["saturation_density"] = 0.0
            errors.append(str(rel_path))
        else:
            samples, sr = result
            feats = compute_pack_census_features(samples, sr)
            entry.update(feats)

        index[str(rel_path)] = entry
        family_counts[pack_name][family] += 1
        pack_file_lists[pack_name].append(entry)
        analyzed += 1

        if analyzed % 200 == 0:
            elapsed = time.time() - start
            rate = analyzed / elapsed if elapsed > 0 else 0
            print(f"  {analyzed}/{total} files ({rate:.0f} files/sec)...")

    elapsed = time.time() - start
    print(f"Analyzed {analyzed}/{total} files in {elapsed:.1f}s")

    pack_index_path = output_dir / "pack_index.json"
    pack_stats_path = output_dir / "pack_stats.md"
    family_dist_path = output_dir / "family_distribution.json"

    pack_names = sorted(set(pack_name for pack_name, _, _ in all_files))
    total_families = {}
    for pack_name in pack_names:
        total_families[pack_name] = dict(family_counts[pack_name])

    pack_stats = {}
    for pack_name, entries in pack_file_lists.items():
        valid = [e for e in entries if "error" not in e]
        if not valid:
            continue
        keys = ["duration_ms", "rms", "peak", "crest_factor", "lufs_integrated",
                "spectral_centroid", "spectral_bandwidth", "stereo_width",
                "transient_count", "hpr", "spectral_flux_mean",
                "zero_crossing_rate", "attack_ms", "decay_length_ms",
                "pitch_hz", "loudness_range", "saturation_density"]
        stats = {}
        for k in keys:
            vals = [e.get(k, 0.0) for e in valid if isinstance(e.get(k), (int, float))]
            if vals:
                stats[k] = {
                    "mean": float(np.mean(vals)),
                    "std": float(np.std(vals)),
                    "min": float(np.min(vals)),
                    "max": float(np.max(vals)),
                    "median": float(np.median(vals)),
                }
            else:
                stats[k] = {"mean": 0.0, "std": 0.0, "min": 0.0, "max": 0.0, "median": 0.0}
        pack_stats[pack_name] = {
            "total_files": len(entries),
            "valid_files": len(valid),
            "error_files": len(entries) - len(valid),
            "features": stats,
        }

    category_dist = {}
    for entry in index.values():
        cat = entry.get("category", "other")
        pack = entry.get("pack", "unknown")
        if cat not in category_dist:
            category_dist[cat] = {}
        if pack not in category_dist[cat]:
            category_dist[cat][pack] = 0
        category_dist[cat][pack] += 1

    output = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_packs": len(pack_names),
        "total_folders": len(pack_folders),
        "total_files_found": total,
        "total_files_analyzed": analyzed,
        "total_errors": len(errors),
        "sample_rate": SAMPLE_RATE,
        "packs": pack_names,
        "pack_families": total_families,
        "pack_stats": pack_stats,
        "category_distribution": category_dist,
        "error_files": errors,
        "files": index,
    }

    pack_index_path.write_text(json.dumps(output, indent=2, cls=SafeEncoder))
    print(f"Wrote {pack_index_path} ({len(index)} files)")

    md_lines = []
    md_lines.append("# cShot Pack Census Report")
    md_lines.append("")
    md_lines.append(f"**Generated:** {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}")
    md_lines.append(f"**Total packs:** {len(pack_names)}")
    md_lines.append(f"**Total folders scanned:** {len(pack_folders)}")
    md_lines.append(f"**Total audio files found:** {total}")
    md_lines.append(f"**Total files analyzed:** {analyzed}")
    md_lines.append(f"**Errors (corrupted/unreadable):** {len(errors)}")
    md_lines.append("")

    md_lines.append("## Pack Summary")
    md_lines.append("")
    md_lines.append("| Pack | Files | Families | Duration (avg) | Centroid (avg) | LUFS (avg) | HPR (avg) |")
    md_lines.append("|------|-------|----------|----------------|----------------|------------|-----------|")
    for pn in sorted(pack_names):
        ps = pack_stats.get(pn, {})
        fs = ps.get("features", {})
        nf = ps.get("valid_files", 0)
        dur = fs.get("duration_ms", {}).get("mean", 0)
        cent = fs.get("spectral_centroid", {}).get("mean", 0)
        luf = fs.get("lufs_integrated", {}).get("mean", 0)
        hpr = fs.get("hpr", {}).get("mean", 0)
        fam_count = len(total_families.get(pn, {}))
        luf_str = f"{luf:.1f}" if luf > -90 else "silent"
        md_lines.append(f"| {pn} | {nf} | {fam_count} | {dur:.0f}ms | {cent:.0f}Hz | {luf_str} | {hpr:.2f} |")
    md_lines.append("")

    md_lines.append("## Category Distribution")
    md_lines.append("")
    md_lines.append("| Category | Total | Packs |")
    md_lines.append("|----------|-------|-------|")
    for cat in sorted(category_dist.keys()):
        total_cat = sum(category_dist[cat].values())
        pack_cat = len(category_dist[cat])
        md_lines.append(f"| {cat} | {total_cat} | {pack_cat} |")
    md_lines.append("")

    md_lines.append("## Per-Pack Family Distribution")
    md_lines.append("")
    for pn in sorted(pack_names):
        md_lines.append(f"### {pn}")
        md_lines.append("")
        md_lines.append("| Family | Count |")
        md_lines.append("|--------|-------|")
        for fam in sorted(total_families.get(pn, {}).keys()):
            cnt = total_families[pn][fam]
            md_lines.append(f"| {fam} | {cnt} |")
        md_lines.append("")

    md_lines.append("## Error Files")
    md_lines.append("")
    if errors:
        for err in errors:
            md_lines.append(f"- `{err}`")
    else:
        md_lines.append("None — all files read successfully.")
    md_lines.append("")

    md_lines.append("## Pack Feature Profiles")
    md_lines.append("")
    md_lines.append("| Pack | Duration (ms) | Centroid (Hz) | Bandwidth (Hz) | Crest | LUFS | Width |")
    md_lines.append("|------|--------------|---------------|----------------|-------|------|-------|")
    for pn in sorted(pack_names):
        ps = pack_stats.get(pn, {})
        fs = ps.get("features", {})
        dur = fs.get("duration_ms", {}).get("mean", 0)
        cent = fs.get("spectral_centroid", {}).get("mean", 0)
        bw = fs.get("spectral_bandwidth", {}).get("mean", 0)
        crest = fs.get("crest_factor", {}).get("mean", 0)
        luf = fs.get("lufs_integrated", {}).get("mean", 0)
        width = fs.get("stereo_width", {}).get("mean", 0)
        luf_str = f"{luf:.1f}" if luf > -90 else "silent"
        md_lines.append(f"| {pn} | {dur:.0f} | {cent:.0f} | {bw:.0f} | {crest:.1f} | {luf_str} | {width:.3f} |")

    md_lines.append("---")
    md_lines.append("*Generated by cShot Pack Census (Week 1)*")

    pack_stats_path.write_text("\n".join(md_lines))
    print(f"Wrote {pack_stats_path}")

    family_dist = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_families": {p: len(f) for p, f in total_families.items()},
        "families": total_families,
        "category_distribution": category_dist,
    }
    family_dist_path.write_text(json.dumps(family_dist, indent=2, cls=SafeEncoder))
    print(f"Wrote {family_dist_path}")

    print(f"\nPack Census complete:")
    print(f"  Packs: {len(pack_names)}")
    print(f"  Files: {analyzed}")
    print(f"  Errors: {len(errors)}")
    print(f"  Families: {sum(len(v) for v in total_families.values())}")

    return output


def cmd_pack_census_quick(args):
    """Quick census — just list what packs and file counts exist, no feature extraction."""
    pack_folders = discover_pack_folders()
    print(f"Found {len(pack_folders)} pack folders\n")

    total_files = 0
    for pack_name, pack_root in pack_folders:
        files = collect_audio_files(pack_root)
        families = defaultdict(int)
        for f in files:
            family = infer_family(f, pack_root)
            families[family] += 1
        total_files += len(files)
        print(f"  {pack_name}: {len(files)} files")
        for fam in sorted(families.keys()):
            print(f"    {fam}: {families[fam]}")
        print()

    print(f"Total: {total_files} audio files across {len(pack_folders)} folders")
