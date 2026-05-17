"""Piano one-shot analysis: attack, hammer noise, decay, brightness, resonance."""

import json
import math
import sys
import time
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.features import (
    compute_features, compute_rms, compute_peak, compute_zcr,
    compute_spectral_centroid, compute_hpr, compute_attack_time, compute_decay_length,
    detect_pitch_full, hz_to_note,
)
from gen.io import read_wav


def analyze_piano_attack(samples: np.ndarray, sr: int) -> dict:
    """Analyze piano attack characteristics."""
    if len(samples) < 100:
        return {"attack_ms": 0, "attack_sharpness": 0, "hammer_noise_ratio": 0, "pre_noise_floor": 0}

    attack_ms = compute_attack_time(samples, sr)
    peak_idx = int(np.argmax(np.abs(samples)))
    attack_end = min(peak_idx + int(0.05 * sr), len(samples))

    if attack_end <= 10:
        return {"attack_ms": attack_ms, "attack_sharpness": 0, "hammer_noise_ratio": 0, "pre_noise_floor": 0}

    attack_region = samples[max(0, peak_idx - int(0.003 * sr)):attack_end]
    attack_energy = np.sum(attack_region ** 2)

    pre_noise_samples = samples[:max(1, int(0.010 * sr))]
    pre_noise_energy = np.sum(pre_noise_samples ** 2)

    total_energy = np.sum(samples ** 2)
    pre_noise_floor = pre_noise_energy / max(total_energy, 1e-10)

    # Attack sharpness: spectral centroid slope in first 10ms
    attack_start = samples[max(0, peak_idx - int(0.005 * sr)):max(10, peak_idx + int(0.010 * sr))]
    if len(attack_start) >= 32:
        attack_zcr = compute_zcr(attack_start)
        attack_centroid = compute_spectral_centroid(attack_start, sr)
    else:
        attack_zcr = 0
        attack_centroid = 0

    # Hammer noise: noise-like energy in the attack transient
    body_start = samples[attack_end:min(attack_end + int(0.100 * sr), len(samples))]
    body_energy = np.sum(body_start ** 2) if len(body_start) > 0 else 1e-10
    hammer_noise_ratio = attack_energy / max(body_energy, 1e-10)

    return {
        "attack_ms": round(attack_ms, 2),
        "attack_sharpness": round(attack_zcr, 4),
        "attack_centroid_hz": round(attack_centroid, 1),
        "hammer_noise_ratio": round(hammer_noise_ratio, 3),
        "pre_noise_floor": round(pre_noise_floor, 6),
    }


def analyze_piano_decay(samples: np.ndarray, sr: int) -> dict:
    """Analyze piano multi-stage decay: fast initial decay followed by slower sustain."""
    if len(samples) < sr // 10:
        return {"decay_ms": 0, "sustain_ratio": 0, "decay_shape": "unknown"}

    env = np.abs(samples)
    peak = np.max(env)
    if peak < 0.001:
        return {"decay_ms": 0, "sustain_ratio": 0, "decay_shape": "unknown"}

    decay_ms = compute_decay_length(samples, sr)
    peak_idx = np.argmax(env)

    # Characterize decay as multi-stage
    # Piano has: fast initial drop → slower sustain → release
    # Measure energy in three stages after peak
    total_len = len(samples)
    if peak_idx + 100 >= total_len:
        return {"decay_ms": decay_ms, "sustain_ratio": 0, "decay_shape": "unknown"}

    stage1_end = min(peak_idx + int(0.050 * sr), total_len)
    stage2_end = min(peak_idx + int(0.300 * sr), total_len)
    tail_end = max(stage2_end + 1, total_len)

    stage1 = samples[peak_idx:stage1_end]
    stage2 = samples[stage1_end:stage2_end]
    stage3 = samples[stage2_end:]

    e1 = np.sum(stage1 ** 2) / max(len(stage1), 1)
    e2 = np.sum(stage2 ** 2) / max(len(stage2), 1)
    e3 = np.sum(stage3 ** 2) / max(len(stage3), 1)

    # sustain_ratio: how much energy remains in the tail vs attack decay
    sustain_ratio = e3 / max(e1, 1e-10)

    if sustain_ratio > 0.3 and decay_ms > 200:
        decay_shape = "piano_like"
    elif sustain_ratio > 0.1 and decay_ms > 100:
        decay_shape = "sustained"
    elif decay_ms < 30:
        decay_shape = "stab"
    else:
        decay_shape = "percussive"

    return {
        "decay_ms": round(decay_ms, 2),
        "sustain_ratio": round(sustain_ratio, 4),
        "decay_shape": decay_shape,
        "stage1_energy": round(float(e1), 6),
        "stage2_energy": round(float(e2), 6),
        "stage3_energy": round(float(e3), 6),
    }


def analyze_piano_brightness(samples: np.ndarray, sr: int, pitch_hz: float) -> dict:
    """Analyze brightness relative to pitch. Piano has brighter tone = more high harmonics."""
    centroid = compute_spectral_centroid(samples, sr)
    feats = compute_features(samples, sr)
    high_band = feats.get("high_band_energy", 0)
    mid_band = feats.get("mid_band_energy", 0)

    # Brightness ratio: centroid vs fundamental pitch
    if pitch_hz > 0:
        brightness_ratio = centroid / max(pitch_hz, 1)
    else:
        brightness_ratio = 0

    # Classification
    if brightness_ratio > 8:
        brightness = "very_bright"
    elif brightness_ratio > 4:
        brightness = "bright"
    elif brightness_ratio > 2:
        brightness = "moderate"
    else:
        brightness = "dark"

    return {
        "centroid_hz": round(centroid, 1),
        "brightness_ratio": round(brightness_ratio, 2),
        "brightness": brightness,
        "high_band_energy": round(high_band, 4),
        "mid_band_energy": round(mid_band, 4),
    }


def analyze_piano_resonance(samples: np.ndarray, sr: int, pitch_hz: float) -> dict:
    """Analyze resonance: sympathetic vibrations and tail quality."""
    if len(samples) < sr // 5:
        return {"resonance_confidence": 0, "hpr": 0, "pitch_stability": 0, "has_resonance": False}

    hpr = compute_hpr(samples, sr)

    # Pitch stability in the tail
    peak_idx = np.argmax(np.abs(samples))
    tail_start = min(peak_idx + int(0.200 * sr), len(samples))
    if tail_start + 1000 >= len(samples):
        return {"resonance_confidence": 0, "hpr": hpr, "pitch_stability": 0, "has_resonance": False}

    # Measure pitch stability: split tail into 2 halves, compare pitch
    tail = samples[tail_start:]
    half = len(tail) // 2
    if half < 100:
        return {"resonance_confidence": 0, "hpr": hpr, "pitch_stability": 0, "has_resonance": False}

    tail_info1 = detect_pitch_full(tail[:half])
    tail_info2 = detect_pitch_full(tail[half:])
    tail_pitch1 = tail_info1["pitch_hz"]
    tail_pitch2 = tail_info2["pitch_hz"]
    conf1 = tail_info1["confidence"]
    conf2 = tail_info2["confidence"]

    pitch_stability = 0
    if tail_pitch1 > 0 and tail_pitch2 > 0 and conf1 > 0.3 and conf2 > 0.3:
        pitch_diff = abs(tail_pitch1 - tail_pitch2) / max(tail_pitch1, 0.001)
        pitch_stability = max(0, 1.0 - min(pitch_diff, 1.0))

    # Resonance confidence: combination of HPR, pitch stability, and long decay
    feats = compute_features(samples, sr)
    decay_ms = feats.get("decay_length_ms", 0)
    duration_ms = feats.get("duration_ms", 0)

    resonance_conf = 0.0
    if hpr > 0.5:
        resonance_conf += 0.3
    if pitch_stability > 0.8:
        resonance_conf += 0.3
    if decay_ms > 300:
        resonance_conf += 0.2
    if duration_ms > 500:
        resonance_conf += 0.2
    if pitch_hz > 80:
        resonance_conf += 0.1
    resonance_conf = min(resonance_conf, 1.0)

    return {
        "resonance_confidence": round(resonance_conf, 3),
        "hpr": round(hpr, 4),
        "pitch_stability": round(pitch_stability, 4),
        "tail_pitch1_hz": round(tail_pitch1, 1) if tail_pitch1 > 0 else 0,
        "tail_pitch2_hz": round(tail_pitch2, 1) if tail_pitch2 > 0 else 0,
        "has_resonance": resonance_conf > 0.5,
    }


def analyze_piano_full(samples: np.ndarray, sr: int) -> dict:
    """Complete piano analysis combining all metrics."""
    pitch_info = detect_pitch_full(samples, sr)
    pitch_hz = pitch_info["pitch_hz"]

    attack = analyze_piano_attack(samples, sr)
    decay = analyze_piano_decay(samples, sr)
    brightness = analyze_piano_brightness(samples, sr, pitch_hz)
    resonance = analyze_piano_resonance(samples, sr, pitch_hz)
    feats = compute_features(samples, sr)

    # Overall piano-likeness score (0-1)
    score = 0.0
    if pitch_info["confidence"] > 0.6:
        score += 0.15
    if decay.get("decay_shape") == "piano_like":
        score += 0.30
    elif decay.get("decay_shape") == "sustained":
        score += 0.20
    if resonance.get("has_resonance"):
        score += 0.25
    if 1 < attack.get("attack_ms", 0) < 100:
        score += 0.15
    if 2 < brightness.get("brightness_ratio", 0) < 12:
        score += 0.10
    if pitch_hz > 60:
        score += 0.05
    # Penalize very short attack (sub-millisecond = percussive, not piano)
    if attack.get("attack_ms", 0) < 0.5:
        score -= 0.15
    # Penalize very high pitch clarity that suggests synthetic tone
    if pitch_info["confidence"] > 0.95 and attack.get("attack_ms", 0) < 2:
        score -= 0.10
    score = max(0, min(score, 1.0))

    piano_likely = score >= 0.45

    return {
        "pitch_hz": round(pitch_hz, 1),
        "note": hz_to_note(pitch_hz),
        "pitch_confidence": round(pitch_info["confidence"], 3),
        "piano_likeness_score": round(score, 3),
        "is_piano_like": piano_likely,
        "duration_ms": round(feats["duration_ms"], 1),
        "attack": attack,
        "decay": decay,
        "brightness": brightness,
        "resonance": resonance,
    }


def cmd_analyze_piano(args):
    """Analyze piano one-shot characteristics for audio files."""
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

    print(f"Analyzing {len(wav_files)} file(s) for piano characteristics...\n")

    results = []
    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            continue
        samples, sr = result
        analysis = analyze_piano_full(samples, sr)
        analysis["file"] = wav_path.name
        results.append(analysis)

        piano_icon = "🎹" if analysis["is_piano_like"] else "  "
        dur = analysis["duration_ms"]
        note = analysis["note"]
        pitch = analysis["pitch_hz"]
        att = analysis["attack"]["attack_ms"]
        dec = analysis["decay"]["decay_ms"]
        dec_shape = analysis["decay"]["decay_shape"]
        bright = analysis["brightness"]["brightness"]
        res_conf = analysis["resonance"]["resonance_confidence"]
        score = analysis["piano_likeness_score"]

        print(f"  {piano_icon} {wav_path.name:40s} {note:>6} {pitch:>6.1f}Hz "
              f"att={att:>5.1f}ms dec={dec:>6.1f}ms {dec_shape:12s} "
              f"{bright:12s} res={res_conf:.2f} score={score:.2f}")

    if len(results) > 1:
        piano_count = sum(1 for r in results if r["is_piano_like"])
        avg_score = np.mean([r["piano_likeness_score"] for r in results])

        print(f"\n  {'='*55}")
        print(f"  PIANO ANALYSIS SUMMARY")
        print(f"  {'='*55}")
        print(f"  Total files:       {len(results)}")
        print(f"  Piano-like:         {piano_count}")
        print(f"  Avg piano score:    {avg_score:.3f}")

        piano_files = [r for r in results if r["is_piano_like"]]
        if piano_files:
            print(f"\n  Piano-like files:")
            for r in piano_files:
                print(f"    🎹 {r['file']:40s} note={r['note']:>6} pitch={r['pitch_hz']:>6.1f}Hz "
                      f"score={r['piano_likeness_score']:.2f}")

    if args.output:
        output = {
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "files_analyzed": len(results),
            "results": results,
        }
        out_path = Path(args.output)
        out_path.write_text(json.dumps(output, indent=2))
        print(f"\n  Results written to {out_path}")
