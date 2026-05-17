"""Synth stab analysis: oscillator body, filter envelope, detune, stereo width."""

import json
import math
import sys
import time
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.features import (
    compute_features, compute_spectral_centroid, compute_hpr,
    compute_attack_time, compute_decay_length, detect_pitch_full,
    hz_to_note, hz_to_midi, NOTE_NAMES,
)
from gen.io import read_wav


def analyze_harmonic_profile(samples: np.ndarray, sr: int, pitch_hz: float) -> dict:
    """Analyze oscillator body: harmonic structure and waveform type hints."""
    if pitch_hz <= 0 or len(samples) < 1024:
        return {"harmonic_type": "unknown", "odd_even_ratio": 0, "harmonic_richness": 0}

    n = min(len(samples), 8192)
    spectrum = np.abs(np.fft.rfft(samples[:n]))
    freqs = np.fft.rfftfreq(n, 1.0 / sr)

    # Find harmonic peaks
    harmonics = {}
    for h_idx in range(1, 20):
        target = pitch_hz * h_idx
        if target > sr / 2:
            break
        search_start = max(0, int((target - pitch_hz * 0.25) * n / sr))
        search_end = min(len(spectrum), int((target + pitch_hz * 0.25) * n / sr))
        if search_end <= search_start:
            continue
        peak_idx = search_start + np.argmax(spectrum[search_start:search_end])
        peak_val = float(spectrum[peak_idx])
        peak_freq = float(freqs[peak_idx])
        harmonics[h_idx] = {"amplitude": peak_val, "freq": peak_freq}

    if not harmonics:
        return {"harmonic_type": "noisy", "odd_even_ratio": 0, "harmonic_richness": 0}

    fund = harmonics.get(1, {}).get("amplitude", 1e-10)

    # Odd/even ratio: synths like square waves have strong odd harmonics
    odd_sum = sum(h["amplitude"] for hi, h in harmonics.items() if hi % 2 == 1 and hi > 1)
    even_sum = sum(h["amplitude"] for hi, h in harmonics.items() if hi % 2 == 0)
    odd_even_ratio = odd_sum / max(even_sum, 1e-10)

    # Harmonic richness: how many harmonics are above -40dB from fundamental
    fund_db = 20 * math.log10(max(fund, 1e-10))
    rich_count = 0
    for hi, h in harmonics.items():
        amp_db = 20 * math.log10(max(h["amplitude"], 1e-10))
        if amp_db > fund_db - 40:
            rich_count += 1

    # Categorize waveform type
    if odd_even_ratio > 5 and rich_count >= 5:
        waveform_hint = "square_like"
    elif odd_even_ratio > 2 and rich_count >= 4:
        waveform_hint = "pulse_like"
    elif rich_count >= 8:
        waveform_hint = "sawtooth_like"
    elif rich_count >= 3:
        waveform_hint = "triangle_like"
    elif rich_count >= 1:
        waveform_hint = "sine_like"
    else:
        waveform_hint = "noisy"

    return {
        "harmonic_type": waveform_hint,
        "odd_even_ratio": round(odd_even_ratio, 2),
        "harmonic_richness": rich_count,
        "num_harmonics_detected": len(harmonics),
        "fundamental_db": round(fund_db, 1),
    }


def analyze_filter_envelope(samples: np.ndarray, sr: int) -> dict:
    """Detect filter envelope by measuring spectral centroid over time."""
    if len(samples) < sr // 20:
        return {"filter_motion": "none", "centroid_start": 0, "centroid_end": 0, "centroid_ratio": 0}

    frame_size = min(len(samples) // 4, int(0.050 * sr))
    if frame_size < 64:
        frame_size = min(1024, len(samples))

    n_frames = max(2, len(samples) // frame_size)
    centroids = []
    times = []

    for i in range(n_frames):
        start = i * frame_size
        end = min(start + frame_size, len(samples))
        if end - start < 64:
            break
        frame = samples[start:end]
        c = compute_spectral_centroid(frame, sr)
        centroids.append(c)
        times.append(start / sr * 1000)

    if len(centroids) < 2:
        return {"filter_motion": "none", "centroid_start": 0, "centroid_end": 0, "centroid_ratio": 0}

    c_start = centroids[0]
    c_mid = centroids[len(centroids) // 2]
    c_end = centroids[-1]
    c_min = min(centroids)
    c_max = max(centroids)
    c_range = c_max - c_min
    c_avg = np.mean(centroids)

    # Classify filter motion
    if c_range < c_avg * 0.05:
        filter_motion = "static"
    elif c_end > c_start * 1.3 and c_mid > c_start:
        filter_motion = "opening"
    elif c_end < c_start * 0.7 and c_mid < c_start:
        filter_motion = "closing"
    elif c_end > c_start * 1.1:
        filter_motion = "opening_slightly"
    elif c_end < c_start * 0.9:
        filter_motion = "closing_slightly"
    else:
        filter_motion = "dynamic"

    return {
        "filter_motion": filter_motion,
        "centroid_start_hz": round(c_start, 1),
        "centroid_mid_hz": round(c_mid, 1),
        "centroid_end_hz": round(c_end, 1),
        "centroid_range_hz": round(c_range, 1),
        "centroid_ratio": round(c_end / max(c_start, 1), 3),
    }


def analyze_detune(samples: np.ndarray, sr: int, pitch_hz: float) -> dict:
    """Measure detune by analyzing amplitude modulation and harmonic spreading."""
    if pitch_hz <= 0 or len(samples) < sr // 10:
        return {"detune_amount": 0, "has_detune": False, "modulation_depth": 0}

    # Amplitude envelope analysis: look for beating
    env = np.abs(samples)
    peak = np.max(env)
    if peak < 0.001:
        return {"detune_amount": 0, "has_detune": False, "modulation_depth": 0}

    env = env / peak
    dur_s = len(samples) / sr

    # Bandpass around the fundamental to isolate beating
    n = min(len(samples), 8192)
    spectrum = np.abs(np.fft.rfft(samples[:n]))
    freqs = np.fft.rfftfreq(n, 1.0 / sr)

    # Look for sidebands around harmonics (indicating detune)
    sideband_energy = 0
    total_harmonic_energy = 0
    for h_idx in range(1, 8):
        target = pitch_hz * h_idx
        if target > sr / 2:
            break
        bw = pitch_hz * 0.5
        center_idx = int(target * n / sr)
        bw_bins = int(bw * n / sr)
        if center_idx - bw_bins < 0 or center_idx + bw_bins >= len(spectrum):
            continue
        # Center peak
        center_peak = np.max(spectrum[max(0, center_idx - 2):min(len(spectrum), center_idx + 2)])
        total_harmonic_energy += center_peak
        # Sideband energy (just outside the main peak)
        side_l = np.sum(spectrum[max(0, center_idx - bw_bins):max(0, center_idx - 2)])
        side_r = np.sum(spectrum[min(len(spectrum), center_idx + 2):min(len(spectrum), center_idx + bw_bins)])
        sideband_energy += side_l + side_r

    # Modulation depth from envelope
    env_smooth = np.convolve(env, np.ones(100) / 100, mode='same')
    env_diff = np.abs(env - env_smooth)
    modulation_depth = float(np.mean(env_diff)) if len(env_diff) > 0 else 0

    detune_score = 0
    if sideband_energy > total_harmonic_energy * 0.05:
        detune_score += 0.5
    if modulation_depth > 0.02:
        detune_score += 0.3
    if total_harmonic_energy > 0 and len(harmonics := list(range(1, 8))) > 3:
        detune_score += 0.2

    if detune_score > 0.5:
        detune_amount = "heavy"
        has_detune = True
    elif detune_score > 0.2:
        detune_amount = "light"
        has_detune = True
    else:
        detune_amount = "none"
        has_detune = False

    return {
        "detune_amount": detune_amount,
        "has_detune": has_detune,
        "detune_score": round(detune_score, 3),
        "sideband_ratio": round(sideband_energy / max(total_harmonic_energy, 1e-10), 4),
        "modulation_depth": round(modulation_depth, 5),
    }


def analyze_stereo_width(samples: np.ndarray, sr: int) -> dict:
    """Estimate stereo width from L/R correlation (only if stereo)."""
    if samples.ndim < 2 or samples.shape[1] < 2:
        return {"is_stereo": False, "width": 0, "correlation": 1.0, "width_label": "mono"}

    left = samples[:, 0]
    right = samples[:, 1]

    # Correlation coefficient
    if np.std(left) < 1e-6 or np.std(right) < 1e-6:
        corr = 1.0
    else:
        corr = float(np.corrcoef(left, right)[0, 1])

    # Width: 1 - |correlation| (0=mono, 1=wide)
    width = 1.0 - abs(corr)

    if width < 0.1:
        label = "mono"
    elif width < 0.3:
        label = "narrow"
    elif width < 0.6:
        label = "moderate"
    else:
        label = "wide"

    return {
        "is_stereo": True,
        "width": round(width, 3),
        "correlation": round(corr, 4),
        "width_label": label,
    }


def analyze_synth_full(samples: np.ndarray, sr: int) -> dict:
    """Complete synth analysis combining all metrics."""
    pitch_info = detect_pitch_full(samples, sr)
    pitch_hz = pitch_info["pitch_hz"]

    harmonic = analyze_harmonic_profile(samples, sr, pitch_hz)
    filter_env = analyze_filter_envelope(samples, sr)
    detune = analyze_detune(samples, sr, pitch_hz)
    stereo = analyze_stereo_width(samples, sr)
    feats = compute_features(samples, sr)
    attack_ms = compute_attack_time(samples, sr)

    return {
        "pitch_hz": round(pitch_hz, 1),
        "note": hz_to_note(pitch_hz),
        "pitch_confidence": round(pitch_info["confidence"], 3),
        "duration_ms": round(feats["duration_ms"], 1),
        "attack_ms": round(attack_ms, 2),
        "centroid_hz": round(feats.get("spectral_centroid", 0), 1),
        "hpr": round(compute_hpr(samples, sr), 4),
        "harmonic": harmonic,
        "filter_envelope": filter_env,
        "detune": detune,
        "stereo": stereo,
    }


def cmd_analyze_synth(args):
    """Analyze synth stab characteristics for audio files."""
    in_path = Path(args.input)
    if not in_path.exists():
        print(f"Error: {in_path} not found", file=sys.stderr)
        sys.exit(1)

    if in_path.is_dir():
        wav_files = sorted(in_path.rglob("*.wav"))
    else:
        wav_files = [in_path]

    if not wav_files:
        print(f"Error: no .wav files found", file=sys.stderr)
        sys.exit(1)

    print(f"Analyzing {len(wav_files)} file(s) for synth characteristics...\n")

    results = []
    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            print(f"  ✗ {wav_path.name}: could not read")
            continue
        samples, sr = result
        analysis = analyze_synth_full(samples, sr)
        analysis["file"] = wav_path.name
        results.append(analysis)

        wf = wav_path.name[:38]
        note = analysis["note"]
        pitch = analysis["pitch_hz"]
        htype = analysis["harmonic"]["harmonic_type"]
        fmotion = analysis["filter_envelope"]["filter_motion"]
        det = analysis["detune"]["detune_amount"]
        stereo_label = analysis["stereo"]["width_label"] if analysis["stereo"]["is_stereo"] else "mono"

        print(f"  {wf:40s} {note:>6} {pitch:>6.1f}Hz "
              f"{htype:15s} {fmotion:18s} detune={det:6s} {stereo_label:8s}")

    if len(results) > 1:
        print(f"\n  {'='*55}")
        print(f"  SYNTH ANALYSIS SUMMARY")
        print(f"  {'='*55}")

        types = {}
        motions = {}
        detunes = {}
        for r in results:
            t = r["harmonic"]["harmonic_type"]
            types[t] = types.get(t, 0) + 1
            m = r["filter_envelope"]["filter_motion"]
            motions[m] = motions.get(m, 0) + 1
            d = r["detune"]["detune_amount"]
            detunes[d] = detunes.get(d, 0) + 1

        print(f"  Harmonic types: {types}")
        print(f"  Filter motions: {motions}")
        print(f"  Detune:         {detunes}")
        stereo_count = sum(1 for r in results if r["stereo"]["is_stereo"])
        print(f"  Stereo files:   {stereo_count}/{len(results)}")

    if args.output:
        output = {
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "files_analyzed": len(results),
            "results": results,
        }
        out_path = Path(args.output)
        out_path.write_text(json.dumps(output, indent=2))
        print(f"\n  Results written to {out_path}")
