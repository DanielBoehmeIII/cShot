"""
Week 1 — Song Analysis Module

Extracts comprehensive sonic DNA from full songs:
  - BPM + confidence
  - key + confidence
  - section segmentation
  - transient density
  - spectral mood
  - brightness curve
  - low-end profile
  - stereo movement
  - harmonic complexity

Exports: song_dna.json
"""

import json
import math
import sys
import time
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.io import read_audio_safe
from gen.features import (
    compute_features, detect_tempo, detect_pitch_full, estimate_key,
    compute_spectral_centroid, compute_hpr,
)

NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]


def _frame(samples: np.ndarray, frame_size: int, hop_size: int) -> np.ndarray:
    n_frames = max(1, (len(samples) - frame_size) // hop_size + 1)
    frames = np.zeros((n_frames, frame_size))
    for i in range(n_frames):
        start = i * hop_size
        end = start + frame_size
        frames[i, :min(frame_size, len(samples) - start)] = samples[start:end]
    return frames


def compute_brightness_curve(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Spectral centroid over sliding windows across the full song."""
    frame_size = 4096
    hop_size = 2048
    frames = _frame(samples, frame_size, hop_size)
    centroids = []
    for i in range(frames.shape[0]):
        c = compute_spectral_centroid(frames[i], sr)
        centroids.append(c)
    arr = np.array(centroids)
    if len(arr) < 2:
        return {"mean": 3000.0, "std": 0.0, "min": 3000.0, "max": 3000.0, "slope": 0.0, "curve": [3000.0]}
    centroid_mean = float(np.mean(arr))
    centroid_std = float(np.std(arr))
    centroid_min = float(np.min(arr))
    centroid_max = float(np.max(arr))
    times = np.arange(len(arr)) * hop_size / sr
    if len(arr) > 1:
        slope = float(np.polyfit(times, arr, 1)[0])
    else:
        slope = 0.0
    sampled = arr[::max(1, len(arr) // 50)].tolist()
    return {
        "mean": round(centroid_mean, 1),
        "std": round(centroid_std, 1),
        "min": round(centroid_min, 1),
        "max": round(centroid_max, 1),
        "slope": round(slope, 3),
        "curve": sampled,
    }


def compute_low_end_profile(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Detailed low-end analysis: sub (20-60Hz), bass (60-250Hz), low-mid (250-500Hz)."""
    frame_size = 8192
    frames = _frame(samples, frame_size, frame_size // 2)
    n_freq = frame_size // 2 + 1
    sub_energies = []
    bass_energies = []
    low_mid_energies = []
    total_energies = []
    for i in range(frames.shape[0]):
        spec = np.abs(np.fft.rfft(frames[i])) ** 2
        freqs = np.fft.rfftfreq(frame_size, 1.0 / sr)
        if len(spec) < len(freqs):
            freqs = freqs[:len(spec)]
        total = np.sum(spec) + 1e-10
        sub = np.sum(spec[(freqs >= 20) & (freqs < 60)]) / total
        bass = np.sum(spec[(freqs >= 60) & (freqs < 250)]) / total
        low_mid = np.sum(spec[(freqs >= 250) & (freqs < 500)]) / total
        sub_energies.append(float(sub))
        bass_energies.append(float(bass))
        low_mid_energies.append(float(low_mid))
        total_energies.append(float(total))
    if not sub_energies:
        return {"sub_ratio": 0.0, "bass_ratio": 0.0, "low_mid_ratio": 0.0, "sub_bass_ratio": 0.0}
    sub_mean = float(np.mean(sub_energies))
    bass_mean = float(np.mean(bass_energies))
    low_mid_mean = float(np.mean(low_mid_energies))
    sub_bass = sub_mean + bass_mean
    return {
        "sub_ratio": round(sub_mean, 4),
        "bass_ratio": round(bass_mean, 4),
        "low_mid_ratio": round(low_mid_mean, 4),
        "sub_bass_ratio": round(sub_bass, 4),
        "bass_dominance": "sub_bass" if sub_bass > 0.3 else "balanced" if sub_bass > 0.1 else "bright",
    }


def compute_stereo_movement(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Stereo width/correlation variation over time."""
    if samples.ndim != 2 or samples.shape[1] < 2:
        return {"mean_correlation": 1.0, "std_correlation": 0.0, "movement": "mono"}
    frame_size = 4096
    hop_size = 2048
    corrs = []
    for i in range(0, len(samples) - frame_size, hop_size):
        L = samples[i:i+frame_size, 0]
        R = samples[i:i+frame_size, 1]
        if np.std(L) < 1e-10 or np.std(R) < 1e-10:
            continue
        c = float(np.corrcoef(L, R)[0, 1])
        corrs.append(c)
    if len(corrs) < 2:
        return {"mean_correlation": 1.0, "std_correlation": 0.0, "movement": "mono"}
    mean_c = float(np.mean(corrs))
    std_c = float(np.std(corrs))
    if mean_c < 0.6:
        movement = "wide"
    elif mean_c < 0.85:
        movement = "moderate"
    elif std_c > 0.15:
        movement = "dynamic"
    else:
        movement = "narrow"
    return {
        "mean_correlation": round(mean_c, 4),
        "std_correlation": round(std_c, 4),
        "movement": movement,
    }


def compute_harmonic_complexity(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Harmonic complexity: partial count, inharmonicity, spectral evenness."""
    frame_size = 8192
    n = min(len(samples), frame_size)
    window = np.hanning(n)
    spec = np.abs(np.fft.rfft(samples[:n] * window))
    freqs = np.fft.rfftfreq(n, 1.0 / sr)
    total = np.sum(spec) + 1e-10
    spec_norm = spec / total
    peaks = []
    for i in range(1, len(spec) - 1):
        if spec[i] > spec[i-1] and spec[i] > spec[i+1] and spec[i] > total * 0.005:
            peaks.append((freqs[i], spec[i]))
    n_partials = len(peaks)
    if n_partials > 1:
        ratios = [p[0] / peaks[0][0] for p in peaks[1:]]
        ideal_ratios = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        in_harmonic = 0
        for r in ratios[:14]:
            nearest = min(ideal_ratios, key=lambda x: abs(x - r))
            if abs(r - nearest) > 0.05:
                in_harmonic += 1
        inharmonicity = in_harmonic / max(len(ratios[:14]), 1)
    else:
        inharmonicity = 0.0
    shannon = -np.sum(spec_norm * np.log2(spec_norm + 1e-10)) / np.log2(len(spec_norm) + 1)
    return {
        "partial_count": n_partials,
        "inharmonicity": round(inharmonicity, 4),
        "spectral_evenness": round(float(shannon), 4),
    }


def extract_section_signature(segment: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Extract full sonic signature for a single section."""
    if len(segment) < sr // 4:
        return {"mood": "unknown", "transient_density": "unknown", "texture": "unknown",
                "hpr": 0.5, "centroid": 3000, "stereo_width": 0.5, "low_end_ratio": 0.0}
    from gen.features import compute_hpr, compute_spectral_centroid, compute_stereo_correlation
    feats = compute_features(segment, sr)
    centroid = feats.get("spectral_centroid", 3000)
    hpr = compute_hpr(segment, sr)
    trans = feats.get("transient_count", 5)
    if centroid > 5000:
        mood = "bright"
    elif centroid > 3000:
        mood = "balanced"
    else:
        mood = "dark"
    if trans > 10:
        trans_density = "high"
    elif trans > 4:
        trans_density = "medium"
    else:
        trans_density = "low"
    if hpr > 0.7:
        texture = "tonal"
    elif hpr < 0.3:
        texture = "noisy"
    else:
        texture = "mixed"
    rms = feats.get("rms", 0.01)
    energy_level = "loud" if rms > 0.3 else "medium" if rms > 0.05 else "quiet"
    low_band = feats.get("low_band_energy", 0.0)
    return {
        "mood": mood,
        "centroid_hz": round(centroid, 0),
        "transient_density": trans_density,
        "transient_count": int(trans),
        "texture": texture,
        "hpr": round(hpr, 3),
        "energy_level": energy_level,
        "rms": round(rms, 4),
        "low_end_ratio": round(low_band, 4),
    }


def detect_sections(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Section segmentation with per-section sonic signatures."""
    frame_size = 8192
    hop_size = 4096
    if len(samples) < sr * 4:
        sec_sig = extract_section_signature(samples, sr)
        return {"count": 1, "sections": [{"label": "full", "start_s": 0.0,
                "end_s": len(samples)/sr, "duration_s": len(samples)/sr,
                "energy": 1.0, "sonic_signature": sec_sig}]}
    frames = _frame(samples, frame_size, hop_size)
    energies = []
    centroids = []
    prev_spec = None
    fluxs = []
    for i in range(frames.shape[0]):
        seg = frames[i]
        e = float(np.sqrt(np.mean(seg ** 2)))
        energies.append(e)
        c = compute_spectral_centroid(seg, sr)
        centroids.append(c)
        spec = np.abs(np.fft.rfft(seg))
        if prev_spec is not None:
            half = min(len(spec), len(prev_spec))
            f = float(np.sum(np.maximum(0, spec[:half] - prev_spec[:half])))
            fluxs.append(f)
        else:
            fluxs.append(0.0)
        prev_spec = spec[:]
    energies = np.array(energies)
    centroids = np.array(centroids)
    fluxs = np.array(fluxs)
    fluxs_norm = fluxs / max(np.max(fluxs), 1e-10)
    threshold = np.mean(fluxs_norm) + np.std(fluxs_norm) * 2.0
    boundaries = [0]
    for i in range(1, len(fluxs_norm)):
        if fluxs_norm[i] > threshold and i - boundaries[-1] > 4:
            boundaries.append(i)
    if boundaries[-1] != len(fluxs_norm) - 1:
        boundaries.append(len(fluxs_norm) - 1)
    time_per_frame = hop_size / sr
    section_labels = ["intro", "verse", "chorus", "bridge", "drop", "breakdown", "build", "outro"]
    sections = []
    for idx in range(len(boundaries) - 1):
        start_frame = boundaries[idx]
        end_frame = boundaries[idx + 1]
        start_s = round(start_frame * time_per_frame, 1)
        end_s = round(end_frame * time_per_frame, 1)
        seg_energies = energies[start_frame:end_frame]
        seg_centroids = centroids[start_frame:end_frame]
        avg_energy = float(np.mean(seg_energies)) if len(seg_energies) > 0 else 0
        avg_centroid = float(np.mean(seg_centroids)) if len(seg_centroids) > 0 else 3000
        label = section_labels[idx] if idx < len(section_labels) else f"section_{idx}"
        if idx == len(boundaries) - 2:
            label = "outro"
        elif avg_energy < np.percentile(energies, 25):
            if label != "outro":
                label = "intro" if idx == 0 else "breakdown"
        elif avg_energy > np.percentile(energies, 75):
            label = "chorus" if "chorus" not in [s["label"] for s in sections] else "drop"
        elif avg_centroid > np.percentile(centroids, 70):
            label = "bridge" if label != "intro" else "intro"
        seg_samples = samples[int(start_s * sr):int(end_s * sr)]
        sonic_sig = extract_section_signature(seg_samples, sr)
        sections.append({
            "label": label,
            "start_s": start_s,
            "end_s": end_s,
            "duration_s": round(end_s - start_s, 1),
            "energy": round(avg_energy, 4),
            "centroid": round(avg_centroid, 1),
            "sonic_signature": sonic_sig,
        })
    reduced = []
    merged_label = None
    for s in sections:
        if merged_label and s["label"] == merged_label:
            reduced[-1]["end_s"] = s["end_s"]
            reduced[-1]["duration_s"] = round(reduced[-1]["end_s"] - reduced[-1]["start_s"], 1)
            reduced[-1]["energy"] = round((reduced[-1]["energy"] + s["energy"]) / 2, 4)
        else:
            reduced.append(dict(s))
            merged_label = s["label"]
    return {
        "count": len(reduced),
        "sections": reduced,
    }


def compute_dynamic_range(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Crest factor, dynamic range, and loudness distribution."""
    if len(samples) < sr:
        return {"crest_factor": 0.0, "dynamic_range_db": 0.0}
    frame_size = 2048
    hop_size = 1024
    rms_vals = []
    for i in range(0, len(samples) - frame_size, hop_size):
        r = float(np.sqrt(np.mean(samples[i:i+frame_size] ** 2)))
        rms_vals.append(max(r, 1e-10))
    rms_arr = np.array(rms_vals)
    peak = float(np.max(np.abs(samples))) + 1e-10
    avg_rms = float(np.mean(rms_arr))
    crest = peak / avg_rms
    rms_clipped = np.clip(rms_arr, 1e-5, 1.0)
    rms_db = 20 * np.log10(rms_clipped)
    dynamic_range = float(np.max(rms_db) - np.min(rms_db))
    return {
        "crest_factor": round(crest, 2),
        "dynamic_range_db": round(dynamic_range, 1),
        "peak_rms_db": round(float(np.max(rms_db)), 1),
        "avg_rms_db": round(float(np.mean(rms_db)), 1),
        "min_rms_db": round(float(np.min(rms_db)), 1),
    }


def analyze_song(song_path: Path) -> dict:
    """Comprehensive song analysis with Week 1 feature extraction."""
    if not song_path.exists():
        return {"error": f"file not found: {song_path}"}

    result = read_audio_safe(song_path, mono=False)
    if result is None:
        return {"error": "could not read file"}

    samples, sr = result
    duration_s = len(samples) / sr

    if samples.ndim == 2:
        mono = samples.mean(axis=1)
        stereo = samples
    else:
        mono = samples
        stereo = None

    total_seconds = int(duration_s)
    analysis_windows = []
    for start in range(0, max(total_seconds - 2, 1), max(total_seconds // 8, 1)):
        end = min(start + 4, total_seconds)
        win = mono[start * sr:end * sr]
        if len(win) < sr // 2:
            continue
        feats = compute_features(win, sr)
        analysis_windows.append(feats)

    if not analysis_windows:
        return {"error": "song too short for analysis"}

    all_tempos = []
    all_tempo_confs = []
    for w_start in range(0, max(total_seconds - 5, 1), max(total_seconds // 6, 1)):
        w_end = min(w_start + 10, total_seconds)
        segment = mono[w_start * sr:w_end * sr]
        if len(segment) >= sr:
            bpm, conf = detect_tempo(segment, sr)
            all_tempos.append(bpm)
            all_tempo_confs.append(conf)

    if all_tempos:
        tempo_weights = np.array(all_tempo_confs) + 0.01
        tempo_bpm = float(np.average(all_tempos, weights=tempo_weights))
        tempo_conf = float(np.mean(all_tempo_confs))
        tempo_conf = min(tempo_conf, 0.95)
        tempo_consistency = 1.0 - min(float(np.std(all_tempos)) / max(tempo_bpm, 1), 1.0)
        effective_conf = tempo_conf * (0.5 + 0.5 * tempo_consistency)
        effective_conf = min(effective_conf, 0.95)
    else:
        tempo_bpm = 120.0
        effective_conf = 0.0

    all_pitches = []
    pitch_confidences = []
    for w_start in range(0, max(total_seconds - 3, 1), max(total_seconds // 12, 1)):
        w_end = min(w_start + 6, total_seconds)
        segment = mono[w_start * sr:w_end * sr]
        if len(segment) < sr // 2:
            continue
        p = detect_pitch_full(segment, sr)
        if p["confidence"] > 0.25:
            all_pitches.append(p["pitch_hz"])
            pitch_confidences.append(p["confidence"])

    if all_pitches:
        w = np.array(pitch_confidences) + 0.01
        avg_pitch = float(np.average(all_pitches, weights=w))
        key_name, key_conf = estimate_key(all_pitches)
        key_conf = min(key_conf, 0.95)
    else:
        avg_pitch = 0.0
        key_name = "unknown"
        key_conf = 0.0

    mid_window = mono[max(0, len(mono)//2 - sr*15):len(mono)//2 + sr*15]
    if len(mid_window) < sr:
        mid_window = mono
    mid_feats = compute_features(mid_window, sr)

    centroid = mid_feats.get("spectral_centroid", 3000)
    if centroid > 5000:
        spectral_mood = "bright"
    elif centroid > 3000:
        spectral_mood = "balanced"
    else:
        spectral_mood = "dark"

    trans_count = mid_feats.get("transient_count", 5)
    if trans_count > 10:
        transient_density = "high"
    elif trans_count > 4:
        transient_density = "medium"
    else:
        transient_density = "low"

    hpr = mid_feats.get("hpr", 0.5)
    if hpr > 0.7:
        texture = "tonal"
    elif hpr < 0.3:
        texture = "noisy"
    else:
        texture = "mixed"

    brightness_curve = compute_brightness_curve(mono, sr)
    low_end = compute_low_end_profile(mono, sr)
    stereo_movement = compute_stereo_movement(samples, sr) if stereo is not None else {"movement": "mono", "mean_correlation": 1.0, "std_correlation": 0.0}
    harmonic = compute_harmonic_complexity(mono, sr)
    dynamic = compute_dynamic_range(mono, sr)
    sections = detect_sections(mono, sr)

    if centroid < 2000 and trans_count > 6:
        dominant_style = "trap"
    elif centroid < 2500 and trans_count > 5:
        dominant_style = "drill"
    elif centroid < 3000 and trans_count < 4 and hpr > 0.7:
        dominant_style = "rnb"
    elif centroid > 4000 and stereo_movement["movement"] in ("wide", "dynamic"):
        dominant_style = "hyperpop" if trans_count > 6 else "electronic"
    elif centroid > 3000 and stereo_movement["mean_correlation"] > 0.8:
        dominant_style = "ambient"
    elif centroid < 2500 and hpr > 0.7:
        dominant_style = "lo-fi"
    elif centroid > 3000 and trans_count > 6 and stereo_movement["movement"] in ("wide", "dynamic"):
        dominant_style = "cinematic"
    elif centroid > 3000 and trans_count > 5:
        dominant_style = "house"
    elif centroid > 3500 and trans_count > 7:
        dominant_style = "techno"
    elif centroid > 3000 and trans_count > 6:
        dominant_style = "rage"
    elif centroid < 3500:
        dominant_style = "hip-hop"
    else:
        dominant_style = "electronic"

    section_moods = []
    for s in sections.get("sections", []):
        if s.get("centroid", 3000) > 4500:
            section_moods.append("bright")
        elif s.get("centroid", 3000) < 2000:
            section_moods.append("dark")
        else:
            section_moods.append("balanced")

    analysis = {
        "file": song_path.name,
        "format": "wav",
        "duration_s": round(duration_s, 1),
        "duration_minutes": round(duration_s / 60, 1),
        "sample_rate": sr,
        "channels": samples.shape[1] if samples.ndim == 2 else 1,
        "tempo_bpm": round(tempo_bpm, 1),
        "tempo_confidence": round(effective_conf, 3),
        "tempo_consistency": round(tempo_consistency, 3) if all_tempos else 0,
        "key": key_name,
        "key_confidence": round(key_conf, 3),
        "pitch_hz": round(avg_pitch, 1),
        "spectral": {
            "centroid_hz": round(centroid, 0),
            "mood": spectral_mood,
            "brightness_curve": brightness_curve,
            "centroid_profile": {
                "low": round(mid_feats.get("low_band_energy", 0) * 100, 1),
                "mid": round(mid_feats.get("mid_band_energy", 0) * 100, 1),
                "high": round(mid_feats.get("high_band_energy", 0) * 100, 1),
            },
            "rolloff_hz": round(mid_feats.get("spectral_rolloff", 0), 0),
        },
        "low_end": low_end,
        "transients": {
            "count": int(trans_count),
            "density": transient_density,
            "strength": round(mid_feats.get("transient_strength", 0), 2),
        },
        "texture": {
            "hpr": round(hpr, 3),
            "type": texture,
            "harmonic_complexity": harmonic,
        },
        "stereo": stereo_movement,
        "dynamics": dynamic,
        "sections": sections,
        "section_moods": section_moods,
        "dominant_style": dominant_style,
        "rms": round(mid_feats.get("rms", 0), 4),
        "spectral_flux_mean": round(mid_feats.get("spectral_flux_mean", 0), 3),
        "spectral_flux_std": round(mid_feats.get("spectral_flux_std", 0), 3),
    }

    return analysis


def analyze_songs_batch(song_paths: list[Path]) -> list[dict]:
    """Analyze multiple songs and return a list of results."""
    results = []
    for sp in song_paths:
        print(f"  Analyzing: {sp.name}")
        analysis = analyze_song(sp)
        if "error" in analysis:
            print(f"    Error: {analysis['error']}")
        else:
            results.append(analysis)
    return results


def cmd_song_dna(args):
    """Analyze one or more songs and export song_dna.json."""
    inputs = args.inputs
    out = args.output
    paths = []
    for inp in inputs:
        p = Path(inp)
        if p.exists():
            if p.is_dir():
                for f in sorted(p.rglob("*.wav")):
                    if f.stat().st_size > 100000:
                        paths.append(f)
            else:
                paths.append(p)
        else:
            print(f"Warning: {inp} not found")

    if not paths:
        print("No valid audio files found.")
        sys.exit(1)

    print(f"cShot Song DNA Analysis")
    print(f"{'='*60}")
    print(f"Files to analyze: {len(paths)}")
    print()

    results = analyze_songs_batch(paths)
    if not results:
        print("No successful analyses.")
        sys.exit(1)

    export = {
        "cshot_song_dna_v1": True,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "analysis_count": len(results),
        "results": results,
    }

    out_path = Path(out) if out else REPO_ROOT / "song_dna.json"
    with open(out_path, "w") as f:
        json.dump(export, f, indent=2)

    print(f"Exported {len(results)} song analyses → {out_path}")
    print()

    for r in results[:5]:
        print(f"  {r['file']}:")
        print(f"    Duration: {r['duration_minutes']}m  Tempo: {r['tempo_bpm']} BPM (conf={r['tempo_confidence']:.2f})")
        print(f"    Key: {r['key']} (conf={r['key_confidence']:.2f})")
        print(f"    Mood: {r['spectral']['mood']}  Style: {r['dominant_style']}")
        print(f"    Sections: {r['sections']['count']}  Stereo: {r['stereo']['movement']}")
        print(f"    Texture: {r['texture']['type']}  Transients: {r['transients']['density']}")
        print(f"    Low-end: {r['low_end']['bass_dominance']}  Harmonics: {r['texture']['harmonic_complexity']['partial_count']} partials")
        print()


def extract_palette(analysis: dict) -> dict:
    """Infer sonic palette from a full song analysis.
    Returns drum texture, bass character, synth brightness, atmosphere density, FX style.
    """
    tempo = analysis.get("tempo_bpm", 120)
    centroid = analysis.get("spectral", {}).get("centroid_hz", 3000)
    mood = analysis.get("spectral", {}).get("mood", "balanced")
    trans_count = analysis.get("transients", {}).get("count", 5)
    trans_density = analysis.get("transients", {}).get("density", "medium")
    hpr = analysis.get("texture", {}).get("hpr", 0.5)
    texture_type = analysis.get("texture", {}).get("type", "mixed")
    low_end = analysis.get("low_end", {})
    stereo = analysis.get("stereo", {})
    style = analysis.get("dominant_style", "electronic")
    sections = analysis.get("sections", {}).get("sections", [])
    brightness_curve = analysis.get("spectral", {}).get("brightness_curve", {})
    harmonic = analysis.get("texture", {}).get("harmonic_complexity", {})

    bc_slope = brightness_curve.get("slope", 0)
    bc_std = brightness_curve.get("std", 0)
    n_partials = harmonic.get("partial_count", 5)
    inharm = harmonic.get("inharmonicity", 0)

    # ── Drum Texture Palette ──
    drum_qualities = []
    if trans_count > 8:
        drum_qualities.append("busy")
    elif trans_count > 4:
        drum_qualities.append("moderate")
    else:
        drum_qualities.append("sparse")

    if trans_density == "high":
        drum_qualities.append("dense")
    elif trans_density == "medium":
        drum_qualities.append("balanced")
    else:
        drum_qualities.append("sparse")

    if centroid > 4000:
        drum_qualities.append("bright")
    elif centroid < 2000:
        drum_qualities.append("dark")
    else:
        drum_qualities.append("warm")

    stereo_m = stereo.get("movement", "narrow")
    if stereo_m in ("wide", "dynamic"):
        drum_qualities.append("wide")
    else:
        drum_qualities.append("focused")

    if style in ("trap", "drill", "rage", "hip-hop"):
        drum_qualities.append("808-heavy")
    elif style in ("house", "techno"):
        drum_qualities.append("electronic")
    elif style in ("ambient", "cinematic"):
        drum_qualities.append("textural")
    elif style in ("lo-fi",):
        drum_qualities.append("vintage")

    drum_palette = {
        "primary_qualities": drum_qualities[:4],
        "transient_profile": trans_density,
        "stereo_placement": stereo_m,
        "suggested_drum_types": _suggest_drum_types(style, trans_density, centroid),
    }

    # ── Bass Character ──
    sub_ratio = low_end.get("sub_ratio", 0)
    bass_ratio = low_end.get("bass_ratio", 0)
    bass_dom = low_end.get("bass_dominance", "balanced")
    sub_bass_total = sub_ratio + bass_ratio
    bass_qualities = []
    if sub_bass_total > 0.4:
        bass_qualities.append("sub-heavy")
    elif sub_bass_total > 0.2:
        bass_qualities.append("present")
    else:
        bass_qualities.append("lean")
    if centroid < 2000 and trans_count > 5:
        bass_qualities.append("punchy")
    elif centroid < 3000:
        bass_qualities.append("warm")
    else:
        bass_qualities.append("bright")
    if texture_type == "noisy" or hpr < 0.4:
        bass_qualities.append("distorted")
    elif hpr > 0.7:
        bass_qualities.append("clean")
    else:
        bass_qualities.append("textured")
    if n_partials > 10:
        bass_qualities.append("harmonic-rich")
    elif n_partials > 5:
        bass_qualities.append("focused")
    else:
        bass_qualities.append("pure")

    bass_character = {
        "primary_qualities": bass_qualities[:4],
        "profile": bass_dom,
        "sub_bass_ratio": round(sub_bass_total, 3),
        "suggested_bass_types": _suggest_bass_types(style, bass_dom, sub_bass_total),
    }

    # ── Synth Brightness Profile ──
    synth_qualities = []
    if centroid > 4500:
        synth_qualities.append("bright")
    elif centroid > 3000:
        synth_qualities.append("balanced")
    else:
        synth_qualities.append("dark")
    if bc_slope > 3:
        synth_qualities.append("evolving")
    elif bc_slope < -2:
        synth_qualities.append("darkening")
    else:
        synth_qualities.append("stable")
    if bc_std > 800:
        synth_qualities.append("dynamic")
    else:
        synth_qualities.append("consistent")
    if hpr > 0.7:
        synth_qualities.append("harmonic")
    elif hpr < 0.3:
        synth_qualities.append("noisy")
    else:
        synth_qualities.append("mixed")
    if n_partials > 12:
        synth_qualities.append("complex")
    elif n_partials > 6:
        synth_qualities.append("moderate")
    else:
        synth_qualities.append("simple")

    synth_palette = {
        "primary_qualities": synth_qualities[:4],
        "brightness_centroid": round(centroid, 0),
        "brightness_curve_slope": bc_slope,
        "suggested_synth_types": _suggest_synth_types(style, centroid, hpr),
    }

    # ── Atmosphere Density ──
    atmos_qualities = []
    if trans_density == "high" and harmonic.get("spectral_evenness", 0) > 0.6:
        atmos_qualities.append("dense")
    elif trans_density == "sparse" and hpr > 0.7:
        atmos_qualities.append("airy")
    elif trans_density == "medium":
        atmos_qualities.append("moderate")
    else:
        atmos_qualities.append("sparse")

    if stereo_m == "wide":
        atmos_qualities.append("expansive")
    elif stereo_m == "dynamic":
        atmos_qualities.append("moving")
    elif stereo_m == "moderate":
        atmos_qualities.append("balanced")
    else:
        atmos_qualities.append("intimate")

    if centroid > 4000:
        atmos_qualities.append("bright")
    elif centroid < 2000:
        atmos_qualities.append("moody")
    else:
        atmos_qualities.append("neutral")

    dynamics_range = analysis.get("dynamics", {}).get("dynamic_range_db", 40)
    if dynamics_range > 70:
        atmos_qualities.append("dynamic")
    elif dynamics_range > 40:
        atmos_qualities.append("controlled")
    else:
        atmos_qualities.append("compressed")

    atmosphere = {
        "primary_qualities": atmos_qualities[:4],
        "density": "dense" if len(sections) > 8 else "moderate" if len(sections) > 4 else "sparse",
        "stereo_field": stereo_m,
        "suggested_atmosphere_types": _suggest_atmos_types(style, centroid, stereo_m),
    }

    # ── FX Style ──
    fx_qualities = []
    bc_std_val = brightness_curve.get("std", 0)
    if isinstance(bc_std_val, (int, float)) and bc_std_val > 600:
        fx_qualities.append("sweeping")
    else:
        fx_qualities.append("steady")

    if trans_density == "high" and inharm > 0.7:
        fx_qualities.append("glitchy")
    elif trans_density == "high" and trans_count > 10:
        fx_qualities.append("impactful")
    elif trans_density == "low" and texture_type == "tonal":
        fx_qualities.append("smooth")
    else:
        fx_qualities.append("textural")

    if stereo_m in ("wide",):
        fx_qualities.append("spatial")
    elif stereo_m == "dynamic":
        fx_qualities.append("evolving")
    else:
        fx_qualities.append("direct")

    if inharm > 0.7:
        fx_qualities.append("dissonant")
    elif n_partials > 12 and style in ("cinematic", "hyperpop", "ambient"):
        fx_qualities.append("complex")
    elif style in ("hip-hop", "trap", "drill"):
        fx_qualities.append("rhythmic")
    else:
        fx_qualities.append("clean")

    profile = "cinematic"
    if "impactful" in fx_qualities or "rhythmic" in fx_qualities:
        profile = "rhythmic"
    elif "smooth" in fx_qualities or "clean" in fx_qualities:
        profile = "ambient"

    fx_style = {
        "primary_qualities": fx_qualities[:4],
        "profile": profile,
    }

    palette = {
        "drum_texture": drum_palette,
        "bass_character": bass_character,
        "synth_brightness": synth_palette,
        "atmosphere_density": atmosphere,
        "fx_style": fx_style,
        "production_era": _infer_era(analysis),
    }

    return palette


def _suggest_drum_types(style: str, density: str, centroid: float) -> list[str]:
    types = []
    if style in ("trap", "drill", "hip-hop", "rage"):
        types = ["808_kick", "trap_snare", "layered_clap", "hihat_roll", "perc_flam"]
    elif style in ("house", "techno"):
        types = ["punchy_kick", "tight_snare", "closed_hat", "open_hat", "shaker"]
    elif style in ("ambient", "cinematic"):
        types = ["soft_kick", "ambient_snare", "textural_perc", "noise_hit", "impact"]
    elif style in ("hyperpop", "electronic"):
        types = ["processed_kick", "glitch_snare", "fx_clap", "distorted_hat", "bubble_perc"]
    else:
        types = ["kick", "snare", "clap", "hat", "perc"]
    if density == "sparse":
        types = types[:3]
    return types


def _suggest_bass_types(style: str, profile: str, sub_ratio: float) -> list[str]:
    types = []
    if profile == "sub_bass" or sub_ratio > 0.3:
        types = ["808_sub", "sine_sub", "sub_bass"]
    else:
        types = ["mid_bass", "reese_bass", "pluck_bass"]
    if style in ("trap", "drill", "hip-hop", "rage"):
        types = ["808_slide", "808_pluck", "distorted_808"]
    elif style in ("house", "techno"):
        types = ["punchy_bass", "reese", "fm_bass"]
    elif style in ("ambient", "cinematic"):
        types = ["sub_drone", "evolving_bass", "soft_sub"]
    return types


def _suggest_synth_types(style: str, centroid: float, hpr: float) -> list[str]:
    types = []
    if centroid > 4000:
        types = ["bright_lead", "arpeggio", "bell"]
    elif centroid > 2500:
        types = ["warm_pad", "chord_stab", "pluck"]
    else:
        types = ["dark_pad", "sub_synth", "low_lead"]
    if hpr > 0.7:
        types += ["harmonic_pad"]
    else:
        types += ["textural_pad"]
    if style in ("cinematic", "ambient"):
        types = ["evolving_pad", "atmos_pad", "string_pad"]
    elif style in ("hyperpop", "electronic"):
        types = ["glitch_synth", "fx_lead", "wobble"]
    return types[:4]


def _suggest_atmos_types(style: str, centroid: float, stereo: str) -> list[str]:
    types = []
    if centroid > 4000:
        types = ["bright_air", "shimmer", "high_noise"]
    else:
        types = ["dark_ambience", "sub_drone", "low_rumble"]
    if stereo in ("wide", "dynamic"):
        types += ["wide_pad", "spatial_texture"]
    else:
        types += ["tight_texture", "focused_ambience"]
    if style in ("cinematic", "ambient"):
        types = ["evolving_drone", "field_recording", "granular_cloud"]
    return types[:4]


def _suggest_fx_types(style: str, density: str, centroid: float) -> list[str]:
    types = ["impact", "riser", "glitch", "transition"]
    if style in ("cinematic", "ambient"):
        types = ["cinematic_impact", "whoosh", "reverse_cymbal", "sub_hit"]
    elif style in ("trap", "hip-hop"):
        types = ["808_impact", "adlib_echo", "vinyl_crackle", "noise_sweep"]
    elif style in ("hyperpop", "electronic"):
        types = ["glitch_impact", "bitcrush_transition", "stutter_fx"]
    return types[:4]


def _infer_era(analysis: dict) -> str:
    """Infer production era from sonic characteristics."""
    centroid = analysis.get("spectral", {}).get("centroid_hz", 3000)
    hpr = analysis.get("texture", {}).get("hpr", 0.5)
    low_end = analysis.get("low_end", {}).get("sub_bass_ratio", 0)
    crest = analysis.get("dynamics", {}).get("crest_factor", 10)
    style = analysis.get("dominant_style", "electronic")

    if style in ("trap", "drill", "rage", "hyperpop") and low_end > 0.3 and hpr < 0.6:
        return "modern (2015+)"
    if crest > 15 and centroid < 3000:
        return "classic (1990s-2000s)"
    if hpr > 0.7 and centroid < 3000 and crest < 12:
        return "vintage (1970s-1980s)"
    if centroid > 4000 and crest < 10:
        return "modern (2015+)"
    return "contemporary (2000s-present)"


def cmd_song_palette(args):
    """Extract sonic palette from songs and export palette_embedding.json."""
    inputs = args.inputs
    out = args.output
    paths = []
    for inp in inputs:
        p = Path(inp)
        if p.exists():
            if p.is_dir():
                for f in sorted(p.rglob("*.wav")):
                    if f.stat().st_size > 100000:
                        paths.append(f)
            else:
                paths.append(p)

    if not paths:
        print("No valid audio files found.")
        sys.exit(1)

    print(f"cShot Song Palette Extraction")
    print(f"{'='*60}")
    print(f"Files to analyze: {len(paths)}")
    print()

    palettes = []
    for sp in paths[:10]:
        print(f"  Analyzing: {sp.name}")
        analysis = analyze_song(sp)
        if "error" in analysis:
            print(f"    Error: {analysis['error']}")
            continue
        palette = extract_palette(analysis)
        palette_entry = {
            "file": sp.name,
            "path": str(sp),
            "song_dna": {
                "tempo": analysis.get("tempo_bpm"),
                "key": analysis.get("key"),
                "style": analysis.get("dominant_style"),
                "mood": analysis.get("spectral", {}).get("mood"),
            },
            "palette": palette,
        }
        palettes.append(palette_entry)
        print(f"    Drums: {', '.join(palette['drum_texture']['primary_qualities'][:3])}")
        print(f"    Bass: {', '.join(palette['bass_character']['primary_qualities'][:3])}")
        print(f"    Synth: {', '.join(palette['synth_brightness']['primary_qualities'][:3])}")
        print(f"    Atmos: {', '.join(palette['atmosphere_density']['primary_qualities'][:3])}")
        print(f"    FX: {', '.join(palette['fx_style']['primary_qualities'][:3])}")
        print()

    export = {
        "cshot_palette_embedding_v1": True,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "palette_count": len(palettes),
        "palettes": palettes,
    }

    out_path = Path(out) if out else REPO_ROOT / "palette_embedding.json"
    with open(out_path, "w") as f:
        json.dump(export, f, indent=2)

    print(f"Exported {len(palettes)} palettes → {out_path}")


def cmd_section_dna(args):
    """Extract section-level DNA: detect sections and print per-section signatures."""
    inputs = args.inputs
    paths = []
    for inp in inputs:
        p = Path(inp)
        if p.exists():
            if p.is_dir():
                for f in sorted(p.rglob("*.wav")):
                    if f.stat().st_size > 100000:
                        paths.append(f)
            else:
                paths.append(p)

    if not paths:
        print("No valid audio files found.")
        sys.exit(1)

    print(f"cShot Section DNA Analysis")
    print(f"{'='*60}")
    print()

    for song_path in paths[:3]:
        print(f"Song: {song_path.name}")
        print(f"{'-'*60}")
        analysis = analyze_song(song_path)
        if "error" in analysis:
            print(f"  Error: {analysis['error']}\n")
            continue

        print(f"  Tempo: {analysis['tempo_bpm']} BPM  Key: {analysis['key']}")
        print(f"  Overall: {analysis['spectral']['mood']}, {analysis['dominant_style']}, {analysis['transients']['density']} transients")
        print()

        for sec in analysis['sections']['sections']:
            sig = sec.get('sonic_signature', {})
            print(f"  [{sec['label']:<10s}] {sec['start_s']:>6.1f}s-{sec['end_s']:>6.1f}s "
                  f"({sec['duration_s']:>5.1f}s)  "
                  f"mood={sig.get('mood', '?'):<9s}  "
                  f"trans={sig.get('transient_density', '?'):<7s}  "
                  f"tex={sig.get('texture', '?'):<7s}  "
                  f"hpr={sig.get('hpr', 0):.2f}  "
                  f"cent={sig.get('centroid_hz', 0):.0f}Hz  "
                  f"energy={sig.get('energy_level', '?'):<7s}")
        print()
