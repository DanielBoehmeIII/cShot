"""Reference Database — full pack ingestion with comprehensive feature extraction.

Usage:
  python -m gen.reference_db scan
  python -m gen.reference_db health
"""

import json
import math
import sqlite3
import subprocess
import struct
import sys
import tempfile
import time
from pathlib import Path
from typing import Optional

import numpy as np
from tqdm import tqdm

from gen import REPO_ROOT, PACKS_DIR, SAMPLE_RATE

AUDIO_EXTS = {'.wav', '.wave', '.aif', '.aiff', '.aifc', '.flac', '.mp3', '.wv'}
FFMPEG_EXTS = {'.mp3', '.wv'}


def find_all_audio_files(root_dir: Path) -> list[Path]:
    files = []
    for ext in AUDIO_EXTS:
        files.extend(root_dir.rglob(f"*{ext}"))
        files.extend(root_dir.rglob(f"*{ext.upper()}"))
    files = sorted(set(f for f in files if f.is_file()))
    ignored_dirs = {'__MACOSX'}
    files = [f for f in files if not any(part in ignored_dirs for part in f.parts)]
    return files


def read_audio_file(path: Path) -> Optional[np.ndarray]:
    ext = path.suffix.lower()
    try:
        if ext in FFMPEG_EXTS:
            return _read_via_ffmpeg(path)
        import soundfile as sf
        data, sr = sf.read(str(path))
        if data.ndim > 1:
            data = data.mean(axis=1)
        if sr != SAMPLE_RATE:
            from scipy import signal
            new_len = int(len(data) * SAMPLE_RATE / sr)
            data = signal.resample(data, new_len)
        peak = np.max(np.abs(data))
        if peak > 0:
            data = data / peak * 0.95
        return data.astype(np.float32)
    except Exception:
        return _read_via_ffmpeg(path)


def _read_corrupted_wav(path: Path) -> Optional[np.ndarray]:
    """Try to read a corrupted WAV file by manually parsing its RIFF structure."""
    try:
        raw = path.read_bytes()
        if raw[:4] != b'RIFF' or raw[8:12] != b'WAVE':
            return None
        pos = 12
        channels = 2
        sample_rate = SAMPLE_RATE
        bits_per_sample = 16
        data_found = None
        while pos < len(raw) - 8:
            chunk_id = raw[pos:pos+4]
            chunk_size = struct.unpack('<I', raw[pos+4:pos+8])[0]
            if chunk_id == b'fmt ':
                fmt_data = raw[pos+8:pos+8+min(chunk_size, 16)]
                if len(fmt_data) >= 16:
                    channels = struct.unpack('<H', fmt_data[2:4])[0] or 2
                    sample_rate = struct.unpack('<I', fmt_data[4:8])[0] or SAMPLE_RATE
                    bits_per_sample = struct.unpack('<H', fmt_data[14:16])[0] or 16
            elif chunk_id == b'data':
                data_found = raw[pos+8:pos+8+chunk_size]
                break
            pos += 8 + chunk_size
            if pos % 2:
                pos += 1
        if data_found is None or len(data_found) < 10:
            return None
        bytes_per_sample = max(bits_per_sample // 8, 1) * max(channels, 1)
        n_samples = len(data_found) // bytes_per_sample
        if n_samples < 10:
            return None
        usable = n_samples * bytes_per_sample
        data_found = data_found[:usable]
        if bits_per_sample == 16:
            samples = np.frombuffer(data_found, dtype=np.int16).astype(np.float32)
        elif bits_per_sample == 8:
            samples = np.frombuffer(data_found, dtype=np.uint8).astype(np.float32) - 128.0
        elif bits_per_sample == 32:
            samples = np.frombuffer(data_found, dtype=np.int32).astype(np.float32) / 2**31
        elif bits_per_sample == 24:
            arr = np.frombuffer(data_found, dtype=np.uint8).reshape(-1, 3)
            samples = np.zeros(len(arr), dtype=np.float32)
            for i in range(len(arr)):
                val = int.from_bytes(arr[i].tobytes(), 'little', signed=True)
                samples[i] = val / 2**23 if val != -2**23 else -1.0
        else:
            samples = np.frombuffer(data_found, dtype=np.int16).astype(np.float32)
        if channels and channels > 1:
            try:
                samples = samples.reshape(-1, channels).mean(axis=1)
            except Exception:
                samples = samples.reshape(-1, 2).mean(axis=1) if len(samples) % 2 == 0 else samples
        max_val = np.iinfo(np.int16).max if bits_per_sample < 24 else 2**31
        samples = np.clip(samples / max_val, -1.0, 1.0)
        if sample_rate != SAMPLE_RATE:
            from scipy import signal
            new_len = int(len(samples) * SAMPLE_RATE / sample_rate)
            samples = signal.resample(samples, new_len)
        peak = np.max(np.abs(samples))
        if peak > 0.001:
            samples = samples / peak * 0.95
        return samples.astype(np.float32)
    except Exception:
        return None


def _read_via_ffmpeg(path: Path) -> Optional[np.ndarray]:
    try:
        import soundfile as sf
        with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as tmp:
            tmp_path = Path(tmp.name)
        result = subprocess.run(
            ['ffmpeg', '-y', '-i', str(path), '-ac', '1', '-ar', str(SAMPLE_RATE),
             '-f', 'wav', '-sample_fmt', 's16', str(tmp_path)],
            capture_output=True, timeout=120
        )
        if result.returncode != 0:
            tmp_path.unlink(missing_ok=True)
            return _read_corrupted_wav(path)
        data, sr = sf.read(str(tmp_path))
        tmp_path.unlink(missing_ok=True)
        peak = np.max(np.abs(data))
        if peak > 0:
            data = data / peak * 0.95
        return data.astype(np.float32)
    except Exception:
        return _read_corrupted_wav(path)


def compute_envelope_contour(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    n = len(samples)
    if n < 100:
        return {"attack_ms": 0, "decay_ms": 0, "sustain_ms": 0, "release_ms": 0,
                "peak_time_ms": 0, "envelope_shape": "unknown",
                "attack_slope": 0, "decay_slope": 0}
    env = np.abs(samples)
    peak = np.max(env)
    if peak < 0.001:
        return {"attack_ms": 0, "decay_ms": 0, "sustain_ms": 0, "release_ms": 0,
                "peak_time_ms": 0, "envelope_shape": "silent",
                "attack_slope": 0, "decay_slope": 0}
    env_norm = env / peak
    peak_idx = int(np.argmax(env_norm))
    peak_time_ms = peak_idx / sr * 1000

    t10 = int(np.argmax(env_norm >= 0.1))
    t90 = int(np.argmax(env_norm >= 0.9))
    attack_ms = (t90 - t10) / sr * 1000 if t90 > t10 else peak_idx / sr * 1000

    decay_start = peak_idx
    decay_target = 0.4
    sustain_cross = None
    for i in range(decay_start, len(env_norm)):
        if env_norm[i] <= decay_target:
            sustain_cross = i
            break
    decay_ms = ((sustain_cross or peak_idx) - decay_start) / sr * 1000

    release_target = 0.1
    release_cross = None
    for i in range(decay_start, len(env_norm)):
        if env_norm[i] <= release_target:
            release_cross = i
            break
    release_ms = ((release_cross or len(env_norm)) - decay_start) / sr * 1000

    sustain_ms = max(0, release_ms - decay_ms)

    attack_slope = float(env_norm[t90] - env_norm[t10]) / max(attack_ms, 1) * 1000 if attack_ms > 0 else 0
    decay_slope = float(decay_target - 1.0) / max(decay_ms, 1) * 1000 if decay_ms > 0 else 0

    if attack_ms < 5:
        shape = "sharp"
    elif attack_ms < 30:
        shape = "fast"
    elif attack_ms < 80:
        shape = "moderate"
    else:
        shape = "slow"

    if release_ms > 500:
        shape += "_long_release"
    elif release_ms < 50:
        shape += "_short"

    return {
        "attack_ms": round(attack_ms, 2),
        "decay_ms": round(decay_ms, 2),
        "sustain_ms": round(sustain_ms, 2),
        "release_ms": round(release_ms, 2),
        "peak_time_ms": round(peak_time_ms, 2),
        "envelope_shape": shape,
        "attack_slope": round(attack_slope, 4),
        "decay_slope": round(decay_slope, 4),
    }


def compute_mfcc_librosa(samples: np.ndarray, sr: int = SAMPLE_RATE) -> list[float]:
    try:
        import librosa
        mfcc = librosa.feature.mfcc(y=samples.astype(np.float32), sr=sr, n_mfcc=13,
                                     n_fft=2048, hop_length=512)
        return [float(v) for v in np.mean(mfcc, axis=1)]
    except Exception:
        from gen.features import compute_mfccs
        return compute_mfccs(samples, sr)


def compute_chroma_librosa(samples: np.ndarray, sr: int = SAMPLE_RATE) -> list[float]:
    try:
        import librosa
        chroma = librosa.feature.chroma_stft(y=samples.astype(np.float32), sr=sr,
                                              n_fft=4096, hop_length=2048)
        return [float(v) for v in np.mean(chroma, axis=1)]
    except Exception:
        from gen.features import compute_chroma
        return compute_chroma(samples, sr)


def compute_loudness(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    n = len(samples)
    if n < 10:
        return {"rms": 0.0, "peak": 0.0, "peak_dbfs": -96.0, "rms_dbfs": -96.0}
    rms = float(np.sqrt(np.mean(samples ** 2)))
    peak = float(np.max(np.abs(samples)))
    eps = 1e-10
    rms_db = 20 * math.log10(max(rms, eps))
    peak_db = 20 * math.log10(max(peak, eps))
    return {
        "rms": round(rms, 6),
        "peak": round(peak, 6),
        "rms_dbfs": round(rms_db, 2),
        "peak_dbfs": round(peak_db, 2),
    }


def compute_lufs(samples: np.ndarray, sr: int = SAMPLE_RATE) -> float:
    n = len(samples)
    if n < sr * 0.4:
        return -96.0
    try:
        from scipy import signal
        pre_filter = signal.butter(2, [20, 20000], 'band', fs=sr, output='sos')
        filtered = signal.sosfilt(pre_filter, samples)
        frame_len = int(0.4 * sr)
        hop = int(0.1 * sr)
        powers = []
        for start in range(0, n - frame_len + 1, hop):
            frame = filtered[start:start + frame_len]
            mean_sq = float(np.mean(frame ** 2))
            if mean_sq > 1e-12:
                powers.append(mean_sq)
        if not powers:
            return -96.0
        powers.sort()
        top_idx = int(len(powers) * 0.7)
        top_powers = powers[:top_idx] if top_idx > 0 else powers
        integrated = float(np.mean(top_powers))
        if integrated < 1e-12:
            return -96.0
        return round(-0.691 + 10 * math.log10(integrated), 2)
    except Exception:
        return -96.0


def _safe_librosa_stft(samples: np.ndarray, n_fft: int = 2048, hop_length: int = 512):
    """Safe STFT with adaptive n_fft for short signals."""
    import librosa
    effective_n_fft = min(n_fft, max(32, len(samples)))
    if effective_n_fft < 32:
        return None
    if effective_n_fft < n_fft:
        hop_length = max(1, effective_n_fft // 4)
    try:
        return np.abs(librosa.stft(samples.astype(np.float32), n_fft=effective_n_fft, hop_length=hop_length))
    except Exception:
        return None


def compute_spectral_features_deep(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    try:
        import librosa
        spec = _safe_librosa_stft(samples, n_fft=2048, hop_length=512)
        if spec is None or spec.shape[1] < 1:
            raise ValueError("STFT too small")
        cent = float(np.mean(librosa.feature.spectral_centroid(S=spec, sr=sr)))
        rolloff = float(np.mean(librosa.feature.spectral_rolloff(S=spec, sr=sr)))
        bandwidth = float(np.mean(librosa.feature.spectral_bandwidth(S=spec, sr=sr)))
        flatness = float(np.mean(librosa.feature.spectral_flatness(S=spec)))
        contrast = librosa.feature.spectral_contrast(S=spec, sr=sr)
        contrast_mean = [float(v) for v in np.mean(contrast, axis=1).tolist()]
        return {
            "spectral_centroid": round(cent, 2),
            "spectral_rolloff": round(rolloff, 2),
            "spectral_bandwidth": round(bandwidth, 2),
            "spectral_flatness": round(flatness, 6),
            "spectral_contrast": contrast_mean,
        }
    except Exception:
        from gen.features import (
            compute_spectral_centroid, compute_spectral_rolloff,
            compute_spectral_bandwidth
        )
        return {
            "spectral_centroid": round(compute_spectral_centroid(samples, sr), 2),
            "spectral_rolloff": round(compute_spectral_rolloff(samples, sr), 2),
            "spectral_bandwidth": round(compute_spectral_bandwidth(samples, sr), 2),
            "spectral_flatness": 0.0,
            "spectral_contrast": [],
        }


def compute_zcr_librosa(samples: np.ndarray) -> float:
    try:
        import librosa
        return float(np.mean(librosa.feature.zero_crossing_rate(samples.astype(np.float32))))
    except Exception:
        from gen.features import compute_zcr
        return compute_zcr(samples)


def compute_transient_pattern(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    try:
        import librosa
        onset_env = librosa.onset.onset_strength(y=samples.astype(np.float32), sr=sr)
        onset_frames = librosa.onset.onset_detect(onset_envelope=onset_env, sr=sr, backtrack=True)
        onset_times = librosa.frames_to_time(onset_frames, sr=sr).tolist()
        num_onsets = len(onset_times)
        onset_density = num_onsets / max(len(samples) / sr, 0.01)
        onset_strength_mean = float(np.mean(onset_env)) if len(onset_env) > 0 else 0.0
        onset_strength_std = float(np.std(onset_env)) if len(onset_env) > 0 else 0.0
        return {
            "onset_count": num_onsets,
            "onset_density": round(onset_density, 4),
            "onset_times_ms": [round(t * 1000, 1) for t in onset_times],
            "onset_strength_mean": round(onset_strength_mean, 6),
            "onset_strength_std": round(onset_strength_std, 6),
        }
    except Exception:
        from gen.features import detect_transients
        strength, count, onsets = detect_transients(samples, sr)
        return {
            "onset_count": count,
            "onset_density": count / max(len(samples) / sr, 0.01),
            "onset_times_ms": [round(t, 1) for t in onsets],
            "onset_strength_mean": strength,
            "onset_strength_std": 0.0,
        }


def compute_hpr_librosa(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    try:
        import librosa
        D = librosa.stft(samples.astype(np.float32))
        D_harm, D_perc = librosa.decompose.hpss(D, kernel_size=31)
        harm_energy = float(np.sum(np.abs(D_harm) ** 2))
        perc_energy = float(np.sum(np.abs(D_perc) ** 2))
        total = harm_energy + perc_energy
        hpr = harm_energy / total if total > 0 else 0.5
        return {"hpr": round(hpr, 4), "harmonic_energy": round(harm_energy, 2), "percussive_energy": round(perc_energy, 2)}
    except Exception:
        from gen.features import compute_hpr
        hpr = compute_hpr(samples, sr)
        return {"hpr": round(hpr, 4), "harmonic_energy": 0.0, "percussive_energy": 0.0}


def estimate_key_from_mfcc_chroma(mfcc: list[float], chroma: list[float]) -> tuple[str, float]:
    """Estimate key from chroma directly (fast, no pyin needed)."""
    from gen.features import estimate_key, NOTE_NAMES
    import math
    chroma_arr = np.array(chroma)
    if np.sum(chroma_arr) < 0.001:
        return "unknown", 0.0
    major_profile = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88]
    minor_profile = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17]
    best_key = None
    best_corr = -999.0
    for tonic in range(12):
        rotated_major = major_profile[tonic:] + major_profile[:tonic]
        rotated_minor = minor_profile[tonic:] + minor_profile[:tonic]
        try:
            corr_major = float(np.corrcoef(chroma_arr, rotated_major)[0, 1])
            corr_minor = float(np.corrcoef(chroma_arr, rotated_minor)[0, 1])
        except Exception:
            corr_major = -999
            corr_minor = -999
        if corr_major > best_corr:
            best_corr = corr_major
            best_key = f"{NOTE_NAMES[tonic]} major"
        if corr_minor > best_corr:
            best_corr = corr_minor
            best_key = f"{NOTE_NAMES[tonic]} minor"
    confidence = float(np.clip((best_corr + 1.0) / 2.0, 0.0, 1.0))
    return best_key or "unknown", confidence


def compute_pitch_key(samples: np.ndarray, sr: int = SAMPLE_RATE,
                      chroma: list[float] | None = None) -> dict:
    """Fast pitch/key detection using autocorrelation + chroma key finding."""
    from gen.features import detect_pitch_full, estimate_key
    pitch_info = detect_pitch_full(samples, sr)
    freq = pitch_info["pitch_hz"]
    key_name, key_conf = "unknown", 0.0
    if chroma and sum(chroma) > 0.001:
        key_name, key_conf = estimate_key_from_mfcc_chroma([], chroma)
    return {
        "pitch_hz": round(freq, 2),
        "midi_note": pitch_info["midi_note"],
        "note_name": pitch_info["note_name"],
        "pitch_confidence": round(pitch_info["confidence"], 4),
        "key": key_name,
        "key_confidence": round(key_conf, 4),
    }


def compute_stereo_width(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    return {"width": 0.0, "correlation": 1.0, "is_stereo": False}


def compute_texture_fingerprint(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    try:
        import librosa
        spec = _safe_librosa_stft(samples, n_fft=1024, hop_length=256)
        if spec is None or spec.shape[1] < 1:
            raise ValueError("STFT too small")
        rms_feat = librosa.feature.rms(S=spec)
        rms_std = float(np.std(rms_feat))
        rms_mean = float(np.mean(rms_feat))
        zcr = compute_zcr_librosa(samples)
        flatness = 0.0
        try:
            flatness = float(np.mean(librosa.feature.spectral_flatness(S=spec)))
        except Exception:
            pass
        sub_energy = 0.0
        freqs = librosa.fft_frequencies(sr=sr, n_fft=1024)
        spec_mean = np.mean(spec, axis=1)
        sub_mask = freqs < 100
        if np.any(sub_mask):
            sub_energy = float(np.sum(spec_mean[sub_mask]) / max(np.sum(spec_mean), 1e-10))
        return {
            "rms_variation": round(rms_std / max(rms_mean, 1e-10), 4),
            "zcr_mean": round(zcr, 6),
            "spectral_flatness": round(flatness, 6),
            "sub_energy_ratio": round(sub_energy, 4),
            "uniformity": round(1.0 - min(rms_std / max(rms_mean, 1e-10) / 5, 1.0), 4),
        }
    except Exception:
        return {"rms_variation": 0.0, "zcr_mean": 0.0, "spectral_flatness": 0.0, "sub_energy_ratio": 0.0, "uniformity": 0.0}


def compute_semantic_tags(path: Path) -> list[str]:
    tags = set()
    parent = path.parent.name.lower()
    grandparent = path.parent.parent.name.lower() if path.parent.parent else ""
    name = path.stem.lower()
    stem_clean = ''.join(c for c in name if c.isalnum() or c in ' _-').strip()

    tag_map = {
        'kick': ['kick', 'drum', 'low_end', 'percussion', 'transient'],
        'snare': ['snare', 'drum', 'percussion', 'crack', 'sharp'],
        'clap': ['clap', 'snap', 'percussion', 'bright', 'sharp'],
        'hat': ['hihat', 'hat', 'cymbal', 'bright', 'short'],
        'hi-hat': ['hihat', 'hat', 'cymbal', 'bright', 'short'],
        'hihat': ['hihat', 'hat', 'cymbal', 'bright', 'short'],
        'open': ['open', 'bright', 'wash', 'cymbal', 'long'],
        '808': ['808', 'bass', 'sub', 'low_end', 'trap'],
        'bass': ['bass', 'low_end', 'sub', 'tonal'],
        'fx': ['fx', 'impact', 'texture', 'atmospheric'],
        'impact': ['impact', 'fx', 'hit', 'transient', 'loud'],
        'synth': ['synth', 'tonal', 'electronic', 'melodic'],
        'stab': ['stab', 'tonal', 'short', 'bright', 'electronic'],
        'guitar': ['guitar', 'string', 'tonal', 'pluck', 'acoustic'],
        'piano': ['piano', 'keyboard', 'tonal', 'acoustic', 'melodic'],
        'vocal': ['vocal', 'voice', 'tonal', 'melodic'],
        'vox': ['vocal', 'voice', 'tonal'],
        'crash': ['crash', 'cymbal', 'bright', 'loud', 'wash'],
        'ride': ['ride', 'cymbal', 'bright', 'wash'],
        'tom': ['tom', 'drum', 'percussion', 'tonal'],
        'rim': ['rim', 'drum', 'percussion', 'sharp', 'short'],
        'perc': ['percussion', 'perc', 'hit', 'short'],
        'shake': ['shaker', 'shake', 'percussion', 'rhythm'],
        'loop': ['loop', 'rhythm', 'pattern', 'long'],
        'riser': ['riser', 'build_up', 'fx', 'texture'],
        'noise': ['noise', 'texture', 'atmospheric', 'fx'],
        'pad': ['pad', 'atmospheric', 'tonal', 'ambient'],
        'chord': ['chord', 'tonal', 'harmonic', 'melodic'],
        'arpeggio': ['arpeggio', 'melodic', 'tonal', 'pattern'],
    }

    for keyword, tag_list in tag_map.items():
        if keyword in name or keyword in parent or keyword in grandparent:
            tags.update(tag_list)

    # Pack-level tags
    if 'monteray' in parent or 'monteray' in grandparent:
        tags.update(['monteray', 'trap', 'drum_kit'])
    if 'bay area' in parent or 'bay area' in grandparent:
        tags.update(['bay_area', 'west_coast', 'hyphy', 'hip_hop'])
    if 'destiny' in parent or 'raf3ox' in parent or 'raf3ox' in grandparent:
        tags.update(['destiny', 'trap', 'drum_kit'])
    if 'legacy' in parent or 'legacy' in grandparent:
        tags.add('legacy')
    if 'modeaudio' in parent or 'modeaudio' in grandparent:
        tags.update(['modeaudio', 'professional'])
    if 'shapes' in parent or 'shapes' in grandparent:
        tags.add('shapes')

    if not tags:
        tags.add('unknown')

    return sorted(tags)


def _compute_all_features_from_stft(spec: np.ndarray, sr: int, samples: np.ndarray, partial: bool = False) -> dict:
    """Compute spectral features from pre-computed STFT magnitude spectrogram."""
    import librosa
    feats = {}
    if spec is None or spec.shape[1] < 2:
        return {}
    try:
        cent = float(np.mean(librosa.feature.spectral_centroid(S=spec, sr=sr)))
        feats["spectral_centroid"] = round(cent, 2)
    except Exception:
        feats["spectral_centroid"] = 0.0
    try:
        rolloff = float(np.mean(librosa.feature.spectral_rolloff(S=spec, sr=sr)))
        feats["spectral_rolloff"] = round(rolloff, 2)
    except Exception:
        feats["spectral_rolloff"] = 0.0
    try:
        bandwidth = float(np.mean(librosa.feature.spectral_bandwidth(S=spec, sr=sr)))
        feats["spectral_bandwidth"] = round(bandwidth, 2)
    except Exception:
        feats["spectral_bandwidth"] = 0.0
    try:
        flatness = float(np.mean(librosa.feature.spectral_flatness(S=spec)))
        feats["spectral_flatness"] = round(flatness, 6)
    except Exception:
        feats["spectral_flatness"] = 0.0
    try:
        contrast = librosa.feature.spectral_contrast(S=spec, sr=sr)
        feats["spectral_contrast"] = [float(v) for v in np.mean(contrast, axis=1).tolist()]
    except Exception:
        feats["spectral_contrast"] = []

    if not partial:
        try:
            mfcc = librosa.feature.mfcc(S=librosa.power_to_db(spec ** 2), sr=sr, n_mfcc=13)
            for i, v in enumerate(np.mean(mfcc, axis=1)):
                feats[f"mfcc_{i+1}"] = round(float(v), 6)
        except Exception:
            pass

        try:
            chroma = librosa.feature.chroma_stft(S=spec, sr=sr)
            for i, v in enumerate(np.mean(chroma, axis=1)):
                feats[f"chroma_{i}"] = round(float(v), 6)
        except Exception:
            pass

        try:
            rms_feat = librosa.feature.rms(S=spec)
            rms_arr = rms_feat.flatten()
            rms_std = float(np.std(rms_arr)) if len(rms_arr) > 0 else 0.0
            rms_mean = float(np.mean(rms_arr)) if len(rms_arr) > 0 else 1.0
            freqs = librosa.fft_frequencies(sr=sr, n_fft=(spec.shape[0] - 1) * 2)
            spec_mean = np.mean(spec, axis=1)
            sub_mask = freqs < 100
            sub_energy = 0.0
            if np.any(sub_mask) and np.sum(spec_mean) > 0:
                sub_energy = float(np.sum(spec_mean[sub_mask]) / max(np.sum(spec_mean), 1e-10))
            feats["rms_variation"] = round(rms_std / max(rms_mean, 1e-10), 4)
            feats["sub_energy_ratio"] = round(sub_energy, 4)
            feats["uniformity"] = round(1.0 - min(rms_std / max(rms_mean, 1e-10) / 5, 1.0), 4)
        except Exception:
            pass
    return feats


def compute_all_features(samples: np.ndarray, sr: int, path: Path) -> dict:
    duration_ms = len(samples) / sr * 1000.0
    loudness = compute_loudness(samples, sr)
    lufs = compute_lufs(samples, sr)

    analysis_max_s = 2.0
    n_analysis = min(len(samples), int(analysis_max_s * sr))
    analysis_segment = samples[:n_analysis]

    # Single STFT pass for all spectral features
    import librosa
    spec = _safe_librosa_stft(analysis_segment, n_fft=2048, hop_length=512)
    spectral = _compute_all_features_from_stft(spec, sr, analysis_segment, partial=False) if spec is not None else {}

    zcr = compute_zcr_librosa(analysis_segment)

    envelope = compute_envelope_contour(samples, sr)

    # Transients from onset envelope (fast)
    transients = {"onset_count": 0, "onset_density": 0, "onset_times_ms": [], "onset_strength_mean": 0, "onset_strength_std": 0}
    if spec is not None and spec.shape[1] >= 3:
        try:
            onset_env = librosa.onset.onset_strength(S=spec, sr=sr)
            onset_frames = librosa.onset.onset_detect(onset_envelope=onset_env, sr=sr, backtrack=True)
            onset_times = librosa.frames_to_time(onset_frames, sr=sr).tolist()
            transients = {
                "onset_count": len(onset_times),
                "onset_density": len(onset_times) / max(n_analysis / sr, 0.01),
                "onset_times_ms": [round(t * 1000, 1) for t in onset_times],
                "onset_strength_mean": float(np.mean(onset_env)) if len(onset_env) > 0 else 0.0,
                "onset_strength_std": float(np.std(onset_env)) if len(onset_env) > 0 else 0.0,
            }
        except Exception:
            pass

    # HPR on first 0.5s only for speed
    hpr_segment = samples[:min(len(samples), int(0.5 * sr))]
    hpr = {"hpr": 0.5, "harmonic_energy": 0.0, "percussive_energy": 0.0}
    if len(hpr_segment) > sr // 8:
        try:
            spec_small = _safe_librosa_stft(hpr_segment, n_fft=1024, hop_length=256)
            if spec_small is not None and spec_small.shape[1] >= 3:
                D_harm, D_perc = librosa.decompose.hpss(spec_small, kernel_size=31)
                harm_energy = float(np.sum(np.abs(D_harm) ** 2))
                perc_energy = float(np.sum(np.abs(D_perc) ** 2))
                total = harm_energy + perc_energy
                hpr_val = harm_energy / total if total > 0 else 0.5
                hpr = {"hpr": round(hpr_val, 4), "harmonic_energy": round(harm_energy, 2), "percussive_energy": round(perc_energy, 2)}
        except Exception:
            pass

    chroma_vals = [spectral.get(f"chroma_{i}", 0) for i in range(12)]
    pitch_key = compute_pitch_key(samples, sr, chroma_vals)
    tags = compute_semantic_tags(path)

    try:
        rel_path = str(path.relative_to(REPO_ROOT))
    except ValueError:
        rel_path = str(path)
    pack = path.parent.parent.name if path.parent.parent else path.parent.name
    category = path.parent.name
    feats = {
        "file_path": rel_path,
        "file_name": path.name,
        "pack": pack,
        "category": category,
        "extension": path.suffix.lower(),
        "duration_ms": round(duration_ms, 2),
        "num_samples": len(samples),
        **loudness,
        "lufs_integrated": lufs,
        "zcr": zcr,
        **spectral,
        **envelope,
        **transients,
        **hpr,
        **pitch_key,
        "rms_variation": spectral.get("rms_variation", 0),
        "sub_energy_ratio": spectral.get("sub_energy_ratio", 0),
        "uniformity": spectral.get("uniformity", 0),
        "tags": tags,
        "semantic_tags": ",".join(tags),
    }
    return feats


def build_database(force: bool = False):
    out_dir = REPO_ROOT / "reference_db"
    out_dir.mkdir(exist_ok=True)
    db_path = out_dir / "reference_db.sqlite"
    manifest_path = out_dir / "reference_manifest.json"

    if db_path.exists() and not force:
        print(f"Database exists at {db_path}. Use --force to rebuild.")
        return

    print("=" * 60)
    print("cShot Reference Database — Full Pack Ingestion")
    print("=" * 60)
    t0 = time.time()

    audio_files = find_all_audio_files(PACKS_DIR)
    print(f"\nFound {len(audio_files)} audio files across Packs/")

    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA synchronous=OFF")
    c = conn.cursor()

    c.execute("""CREATE TABLE IF NOT EXISTS reference_sounds (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        file_path TEXT UNIQUE,
        file_name TEXT,
        pack TEXT,
        category TEXT,
        extension TEXT,
        duration_ms REAL,
        num_samples INTEGER,
        rms REAL,
        peak REAL,
        rms_dbfs REAL,
        peak_dbfs REAL,
        lufs_integrated REAL,
        zcr REAL,
        spectral_centroid REAL,
        spectral_rolloff REAL,
        spectral_bandwidth REAL,
        spectral_flatness REAL,
        spectral_contrast TEXT,
        mfcc TEXT,
        chroma TEXT,
        attack_ms REAL,
        decay_ms REAL,
        sustain_ms REAL,
        release_ms REAL,
        peak_time_ms REAL,
        envelope_shape TEXT,
        attack_slope REAL,
        decay_slope REAL,
        onset_count INTEGER,
        onset_density REAL,
        onset_times TEXT,
        onset_strength_mean REAL,
        onset_strength_std REAL,
        hpr REAL,
        harmonic_energy REAL,
        percussive_energy REAL,
        pitch_hz REAL,
        midi_note REAL,
        note_name TEXT,
        pitch_confidence REAL,
        key TEXT,
        key_confidence REAL,
        rms_variation REAL,
        spectral_flatness_texture REAL,
        sub_energy_ratio REAL,
        uniformity REAL,
        semantic_tags TEXT
    )""")

    conn.commit()
    manifest_entries = []
    errors = 0
    skipped_short = 0

    for idx, fpath in enumerate(tqdm(audio_files, desc="Ingesting", unit="files")):
        rel = str(fpath.relative_to(REPO_ROOT))
        samples = read_audio_file(fpath)
        if samples is None:
            errors += 1
            continue
        if len(samples) < 100:
            skipped_short += 1
            continue

        feats = compute_all_features(samples, SAMPLE_RATE, fpath)

        c.execute("""INSERT OR REPLACE INTO reference_sounds (
            file_path, file_name, pack, category, extension,
            duration_ms, num_samples,
            rms, peak, rms_dbfs, peak_dbfs, lufs_integrated,
            zcr, spectral_centroid, spectral_rolloff, spectral_bandwidth,
            spectral_flatness, spectral_contrast,
            mfcc, chroma,
            attack_ms, decay_ms, sustain_ms, release_ms, peak_time_ms,
            envelope_shape, attack_slope, decay_slope,
            onset_count, onset_density, onset_times,
            onset_strength_mean, onset_strength_std,
            hpr, harmonic_energy, percussive_energy,
            pitch_hz, midi_note, note_name, pitch_confidence,
            key, key_confidence,
            rms_variation, spectral_flatness_texture, sub_energy_ratio, uniformity,
            semantic_tags
        ) VALUES (?,?,?,?,?, ?,?,?,?,?,?,?, ?,?,?,?,?,?, ?,?, ?,?,?,?,?,?, ?,?,?, ?,?,?, ?,?, ?,?,?, ?,?,?, ?,?, ?,?,?,?, ?)""", (
            feats["file_path"], feats["file_name"], feats["pack"], feats["category"], feats["extension"],
            feats["duration_ms"], feats["num_samples"],
            feats["rms"], feats["peak"], feats["rms_dbfs"], feats["peak_dbfs"], feats["lufs_integrated"],
            feats["zcr"], feats["spectral_centroid"], feats["spectral_rolloff"], feats["spectral_bandwidth"],
            feats["spectral_flatness"], json.dumps(feats.get("spectral_contrast", [])),
            json.dumps([feats.get(f"mfcc_{i+1}", 0) for i in range(13)]),
            json.dumps([feats.get(f"chroma_{i}", 0) for i in range(12)]),
            feats["attack_ms"], feats["decay_ms"], feats["sustain_ms"], feats["release_ms"], feats["peak_time_ms"],
            feats["envelope_shape"], feats["attack_slope"], feats["decay_slope"],
            feats["onset_count"], feats["onset_density"], json.dumps(feats.get("onset_times_ms", [])),
            feats["onset_strength_mean"], feats["onset_strength_std"],
            feats["hpr"], feats["harmonic_energy"], feats["percussive_energy"],
            feats["pitch_hz"], feats["midi_note"], feats["note_name"], feats["pitch_confidence"],
            feats["key"], feats["key_confidence"],
            feats["rms_variation"], feats["spectral_flatness"], feats["sub_energy_ratio"], feats["uniformity"],
            feats["semantic_tags"],
        ))

        manifest_entry = {k: v for k, v in feats.items() if k != "onset_times_ms"}
        manifest_entries.append(manifest_entry)

        if (idx + 1) % 200 == 0:
            conn.commit()

    conn.commit()
    conn.close()
    elapsed = time.time() - t0

    manifest = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "engine": "cshot-reference-db-v2",
        "pack_root": str(PACKS_DIR),
        "total_files": len(audio_files),
        "ingested": len(manifest_entries),
        "skipped_short": skipped_short,
        "errors": errors,
        "elapsed_seconds": round(elapsed, 1),
        "features": ["mfcc_13", "chroma_12", "spectral_centroid", "spectral_rolloff",
                     "spectral_bandwidth", "spectral_flatness", "spectral_contrast",
                     "envelope_adsr", "onset_pattern", "hpr", "pitch_key",
                     "loudness_lufs", "texture_fingerprint", "semantic_tags"],
        "entries": manifest_entries,
    }

    manifest_path.write_text(json.dumps(manifest, indent=2))
    print(f"\n{'=' * 60}")
    print(f"Database:    {db_path} ({db_path.stat().st_size / 1024 / 1024:.1f} MB)")
    print(f"Manifest:    {manifest_path}")
    print(f"Ingested:    {len(manifest_entries)} / {len(audio_files)}")
    print(f"Skipped (short): {skipped_short}")
    print(f"Errors:      {errors}")
    print(f"Time:        {elapsed:.1f}s")
    print(f"{'=' * 60}")

    return manifest


def compute_embeddings(force: bool = False):
    """Compute numpy embeddings array from the reference database."""
    out_dir = REPO_ROOT / "reference_db"
    out_dir.mkdir(exist_ok=True)
    embed_path = out_dir / "reference_embeddings.npy"
    embed_index = out_dir / "reference_embeddings_index.json"

    if embed_path.exists() and not force:
        print(f"Embeddings exist at {embed_path}. Use --force to rebuild.")
        return

    db_path = out_dir / "reference_db.sqlite"
    if not db_path.exists():
        print("No database found. Run 'scan' first.")
        return

    conn = sqlite3.connect(str(db_path))
    c = conn.cursor()
    c.execute("SELECT file_path, mfcc, chroma, spectral_centroid, spectral_bandwidth, "
              "spectral_rolloff, spectral_flatness, zcr, hpr, attack_ms, decay_ms, "
              "onset_count, onset_density, pitch_hz, pitch_confidence, rms, "
              "rms_variation, sub_energy_ratio, uniformity "
              "FROM reference_sounds ORDER BY id")
    rows = c.fetchall()
    conn.close()

    print(f"Computing embeddings for {len(rows)} reference sounds...")

    embeddings_list = []
    index = []
    for row in rows:
        fp = row[0]
        mfcc = json.loads(row[1]) if row[1] else [0]*13
        chroma = json.loads(row[2]) if row[2] else [0]*12
        spectral = [row[3] or 0, row[4] or 0, row[5] or 0, row[6] or 0]
        zcr = [row[7] or 0]
        hpr = [row[8] or 0]
        envelope = [row[9] or 0, row[10] or 0]
        onsets = [row[11] or 0, row[12] or 0]
        pitch = [row[13] or 0, row[14] or 0]
        loudness = [row[15] or 0]
        texture = [row[16] or 0, row[17] or 0, row[18] or 0]

        vec = np.array(mfcc + chroma + spectral + zcr + hpr + envelope + onsets + pitch + loudness + texture,
                       dtype=np.float32)
        embeddings_list.append(vec)
        index.append(fp)

    embeddings = np.stack(embeddings_list) if embeddings_list else np.zeros((0, 1), dtype=np.float32)

    np.save(str(embed_path), embeddings)
    embed_index.write_text(json.dumps({"files": index, "dim": embeddings.shape[1]}, indent=2))
    print(f"Embeddings: {embed_path} ({embeddings.shape})")
    print(f"Index:      {embed_index} ({len(index)} files)")


def generate_health_report():
    out_dir = REPO_ROOT / "reference_db"
    db_path = out_dir / "reference_db.sqlite"
    if not db_path.exists():
        print("No database found. Run 'scan' first.")
        return

    conn = sqlite3.connect(str(db_path))
    c = conn.cursor()

    c.execute("SELECT COUNT(*) FROM reference_sounds")
    total = c.fetchone()[0]

    c.execute("SELECT pack, COUNT(*) FROM reference_sounds GROUP BY pack ORDER BY COUNT(*) DESC")
    pack_counts = c.fetchall()

    c.execute("SELECT category, COUNT(*) FROM reference_sounds GROUP BY category ORDER BY COUNT(*) DESC")
    category_counts = c.fetchall()

    c.execute("SELECT extension, COUNT(*) FROM reference_sounds GROUP BY extension ORDER BY COUNT(*) DESC")
    ext_counts = c.fetchall()

    for agg_col, agg_name in [
        ("duration_ms", "Duration (ms)"),
        ("spectral_centroid", "Spectral Centroid (Hz)"),
        ("spectral_bandwidth", "Spectral Bandwidth (Hz)"),
        ("spectral_flatness", "Spectral Flatness"),
        ("pitch_hz", "Pitch (Hz)"),
        ("pitch_confidence", "Pitch Confidence"),
        ("rms", "RMS Amplitude"),
        ("rms_dbfs", "RMS (dBFS)"),
        ("lufs_integrated", "LUFS Integrated"),
        ("attack_ms", "Attack (ms)"),
        ("decay_ms", "Decay (ms)"),
        ("release_ms", "Release (ms)"),
        ("onset_count", "Onset Count"),
        ("onset_density", "Onset Density"),
        ("hpr", "Harmonic/Percussive Ratio"),
        ("key_confidence", "Key Confidence"),
    ]:
        c.execute(f"SELECT AVG({agg_col}), MIN({agg_col}), MAX({agg_col}) "
                  f"FROM reference_sounds WHERE {agg_col} IS NOT NULL")

    c.execute("SELECT COUNT(*) FROM reference_sounds WHERE duration_ms < 50")
    very_short = c.fetchone()[0]
    c.execute("SELECT COUNT(*) FROM reference_sounds WHERE duration_ms > 10000")
    very_long = c.fetchone()[0]
    c.execute("SELECT COUNT(*) FROM reference_sounds WHERE spectral_flatness < 0.01 OR spectral_flatness IS NULL")
    flatness_fail = c.fetchone()[0]
    c.execute("SELECT COUNT(*) FROM reference_sounds WHERE peak < 0.01")
    too_quiet = c.fetchone()[0]
    c.execute("SELECT COUNT(*) FROM reference_sounds WHERE pitch_hz < 1 AND duration_ms > 100")
    no_pitch = c.fetchone()[0]

    conn.close()

    report = f"""# Reference Database Health Report

Generated: {time.strftime("%Y-%m-%d %H:%M:%S UTC", time.gmtime())}

## Overview

| Metric | Value |
|--------|-------|
| Total Reference Sounds | {total} |
| Unique Packs | {len(pack_counts)} |
| Unique Categories | {len(category_counts)} |

## Pack Distribution

| Pack | Count |
|------|-------|
"""
    for pack, count in pack_counts:
        report += f"| {pack} | {count} |\n"

    report += f"""
## Category Distribution (Top 20)

| Category | Count |
|----------|-------|
"""
    for cat, count in category_counts[:20]:
        report += f"| {cat} | {count} |\n"

    report += f"""
## Format Distribution

| Format | Count |
|--------|-------|
"""
    for ext, count in ext_counts:
        report += f"| {ext} | {count} |\n"

    report += f"""
## Quality Checks

| Check | Value |
|-------|-------|
| Very short (<50ms) | {very_short} |
| Very long (>10s) | {very_long} |
| Flat/zero spectral flatness | {flatness_fail} |
| Too quiet (peak < 0.01) | {too_quiet} |
| No pitch detected | {no_pitch} |

## Coverage

- {total} reference sounds across {len(pack_counts)} packs
- {len(category_counts)} distinct categories
- Suitable for retrieval-augmented generation
- Suitable for reference-conditioned synthesis
"""
    report_path = out_dir / "reference_health_report.md"
    report_path.write_text(report)
    print(f"Health report: {report_path}")


def cmd_scan(args):
    build_database(force=getattr(args, 'force', False))
    compute_embeddings(force=getattr(args, 'force', False))
    generate_health_report()


def cmd_health(args):
    generate_health_report()


if __name__ == "__main__":
    import argparse
    p = argparse.ArgumentParser(description="cShot Reference Database")
    sp = p.add_subparsers(dest="cmd")
    sp.add_parser("scan", help="Full scan: ingest all packs, compute features, build DB")
    sp.add_parser("health", help="Generate health report")
    args = p.parse_args()
    if args.cmd == "scan":
        build_database(force=True)
        compute_embeddings(force=True)
        generate_health_report()
    elif args.cmd == "health":
        generate_health_report()
    else:
        p.print_help()
