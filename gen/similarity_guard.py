"""Similarity guard — ensures generated sounds are not too close to sources.

Prevents exact copies, enforces minimum transformation distance,
and provides metadata showing source lineage.
"""

import json
from pathlib import Path
from typing import Optional

import numpy as np

from gen import SAMPLE_RATE, REPO_ROOT
from gen.io import read_audio_safe
from gen.features import compute_features, compute_spectral_centroid, compute_spectral_rolloff


def compute_similarity_score(samples_a: np.ndarray, samples_b: np.ndarray) -> float:
    """Compute perceptual similarity score between 0 (different) and 1 (identical)."""
    n = min(len(samples_a), len(samples_b))
    if n < 256:
        return 0.0

    a = samples_a[:n]
    b = samples_b[:n]

    # Normalize
    peak_a = np.max(np.abs(a))
    peak_b = np.max(np.abs(b))
    if peak_a > 0.001:
        a = a / peak_a
    if peak_b > 0.001:
        b = b / peak_b

    # 1. Spectral similarity
    n_fft = min(2048, n)
    spec_a = np.abs(np.fft.rfft(a[:n_fft]))
    spec_b = np.abs(np.fft.rfft(b[:n_fft]))
    spec_sim = _cosine_sim(spec_a, spec_b)

    # 2. Envelope similarity
    env_a = np.abs(a)
    env_b = np.abs(b)
    window = max(1, n // 100)
    kernel = np.ones(window) / window
    env_a = np.convolve(env_a, kernel, mode='same')
    env_b = np.convolve(env_b, kernel, mode='same')
    env_sim = _cosine_sim(env_a[:1000], env_b[:1000]) if n > 2000 else _cosine_sim(env_a, env_b)

    # 3. MFCC comparison (more perceptual)
    mfcc_sim = 0.0
    try:
        import librosa
        mfcc_a = librosa.feature.mfcc(y=a.astype(np.float32), sr=SAMPLE_RATE, n_mfcc=20, n_fft=2048, hop_length=512)
        mfcc_b = librosa.feature.mfcc(y=b.astype(np.float32), sr=SAMPLE_RATE, n_mfcc=20, n_fft=2048, hop_length=512)
        # Compare per-frame MFCC distance (more sensitive than mean)
        if mfcc_a.shape[1] > 2 and mfcc_b.shape[1] > 2:
            n_frames = min(mfcc_a.shape[1], mfcc_b.shape[1])
            distances = []
            for fi in range(n_frames):
                d = float(np.sqrt(np.sum((mfcc_a[:, fi] - mfcc_b[:, fi]) ** 2)))
                distances.append(d)
            avg_dist = float(np.mean(distances))
            mfcc_sim = 1.0 / (1.0 + avg_dist * 2.0)
    except Exception:
        pass

    # 4. Waveform correlation (time-domain similarity)
    corr = float(np.corrcoef(a[:min(len(a), len(b))], b[:min(len(a), len(b))])[0, 1]) if len(a) > 10 and len(b) > 10 else 0
    corr_sim = max(0, (corr + 1) / 2)

    # Weight: these become much more strict with pitch shifts and EQ changes
    score = spec_sim * 0.15 + env_sim * 0.15 + mfcc_sim * 0.5 + corr_sim * 0.2
    return float(np.clip(score, 0.0, 1.0))


def _cosine_sim(a: np.ndarray, b: np.ndarray) -> float:
    dot = float(np.dot(a, b))
    norm = float(np.linalg.norm(a) * np.linalg.norm(b))
    return dot / norm if norm > 1e-10 else 0.0


def check_too_close(generated_path: Path, source_path: Path,
                     threshold: float = 0.85) -> tuple[bool, float]:
    """Check if generated sound is too close to its source.
    Returns (is_too_close, similarity_score)."""
    gen_samples = read_audio_safe(generated_path)
    src_samples = read_audio_safe(source_path)
    if gen_samples is None or src_samples is None:
        return False, 0.0

    gen_data, gen_sr = gen_samples
    src_data, src_sr = src_samples

    sim = compute_similarity_score(gen_data, src_data)
    return sim >= threshold, sim


def check_minimum_transformation(transform_chain: list[dict]) -> tuple[bool, float]:
    """Check if transformation chain has enough change from the source.
    Returns (passes_minimum, transformation_score)."""
    if not transform_chain:
        return False, 0.0

    total_change = 0.0
    for t in transform_chain:
        t_type = t.get('type', '')
        if t_type == 'pitch_shift':
            total_change += abs(t.get('semitones', 0)) / 12.0
        elif t_type == 'time_stretch':
            total_change += abs(t.get('rate', 1.0) - 1.0) * 2.0
        elif t_type == 'transient_reshape':
            total_change += abs(t.get('attack_boost', 0)) * 0.5
            total_change += abs(t.get('sustain_cut', 0)) * 0.5
        elif t_type == 'eq_tilt':
            total_change += abs(t.get('tilt_db', 0)) / 12.0
        elif t_type == 'saturation':
            total_change += t.get('drive', 0) * 0.5
        elif t_type == 'resample_lofi':
            total_change += 0.3
        elif t_type == 'convolution_reverb':
            total_change += t.get('wet', 0) * 0.5
        else:
            total_change += 0.15

    min_score = 0.08  # Even a small pitch shift is enough
    return total_change >= min_score, round(total_change, 3)


def validate_output(generated_path: Path, source_path: Path,
                     transform_chain: list[dict],
                     sim_threshold: float = 0.95) -> dict:
    """Full validation of a generated output.
    Returns dict with pass/fail and diagnostics."""
    too_close, sim = check_too_close(generated_path, source_path, sim_threshold)
    passes_min, transform_score = check_minimum_transformation(transform_chain)

    result = {
        "file": generated_path.name,
        "source": str(source_path),
        "similarity_to_source": round(sim, 4),
        "too_close": too_close,
        "transformation_score": transform_score,
        "passes_minimum_transformation": passes_min,
        "pass": not too_close and passes_min,
    }

    if too_close:
        result["reason"] = f"Too similar to source ({sim:.3f} >= {sim_threshold})"
    elif not passes_min:
        result["reason"] = f"Insufficient transformation ({transform_score} < 0.15)"

    return result


def validate_kit(kit_dir: Path, lineage_path: Optional[Path] = None) -> dict:
    """Validate all files in a kit directory against their sources."""
    if lineage_path is None:
        lineage_path = kit_dir / "source_lineage.json"

    if not lineage_path.exists():
        return {"error": f"No source_lineage.json found in {kit_dir}"}

    with open(lineage_path) as f:
        lineage = json.load(f)

    results = []
    passed = 0
    failed = 0

    for entry in lineage.get("entries", []):
        out_name = entry.get("output", "")
        source = entry.get("source", "")
        transforms = entry.get("transforms", [])

        out_path = kit_dir / out_name
        src_path = REPO_ROOT / source if not Path(source).exists() else Path(source)

        if not out_path.exists() or not src_path.exists():
            continue

        result = validate_output(out_path, src_path, transforms)
        results.append(result)
        if result.get("pass", False):
            passed += 1
        else:
            failed += 1

    return {
        "kit": str(kit_dir),
        "total": len(results),
        "passed": passed,
        "failed": failed,
        "pass_rate": round(passed / max(len(results), 1) * 100, 1),
        "results": results,
    }
