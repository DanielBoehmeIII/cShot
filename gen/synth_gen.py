"""Synth generation: stab, pluck, pad, chord, lead, bass-stab with full controls."""

import math
import random
import sys
import time
from pathlib import Path

import numpy as np
from scipy import signal as sp_signal

from gen import SAMPLE_RATE
from gen.dsp import (
    noise_like, tape_saturation, soft_clip,
    biquad_low_shelf, biquad_high_shelf, biquad_peaking,
)
from gen.io import write_wav


# ─── Oscillator types ───────────────────────────────────

def osc_saw(t: float, freq: float, detune: float = 0.0) -> float:
    """Sawtooth oscillator with optional detune."""
    f = freq * (1.0 + detune)
    phase = (t * f) % 1.0
    return 2.0 * phase - 1.0


def osc_square(t: float, freq: float, pulse_width: float = 0.5, detune: float = 0.0) -> float:
    """Square/pulse oscillator with variable pulse width."""
    f = freq * (1.0 + detune)
    phase = (t * f) % 1.0
    return 1.0 if phase < pulse_width else -1.0


def osc_sine(t: float, freq: float, detune: float = 0.0) -> float:
    """Sine oscillator."""
    f = freq * (1.0 + detune)
    return math.sin(2.0 * math.pi * f * t)


def osc_noise(t: float, seed_offset: float = 0.0) -> float:
    """White noise oscillator (sample-based, deterministic)."""
    return noise_like(t * SAMPLE_RATE + seed_offset)


OSCILLATORS = {
    "saw": osc_saw,
    "square": osc_square,
    "sine": osc_sine,
    "noise": lambda t, f, d=0.0: osc_noise(t, d * 100.0),
}


# ─── Synth profiles ─────────────────────────────────────

SYNTH_PROFILES = {
    "stab": {
        "label": "Synth Stab",
        "default_duration_ms": 600.0,
        "default_pitch_hz": 220.0,
        "osc_type": "saw",
        "osc_mix": {"saw": 0.7, "square": 0.0, "sine": 0.2, "noise": 0.1},
        "detune_amount": 1.0,         # cents between osc layers
        "detune_layers": 3,
        "filter_envelope": "opening",
        "filter_cutoff_start": 0.3,
        "filter_cutoff_end": 1.0,
        "filter_resonance": 0.3,
        "attack_ms": 5,
        "decay_ms": 100,
        "sustain_level": 0.2,
        "release_ms": 200,
        "stereo_width": 0.0,
        "saturation": 0.3,
        "chord_density": 1,           # 1=single note, 3=triad, 4=7th chord
        "pulse_width": 0.5,
    },
    "pluck": {
        "label": "Synth Pluck",
        "default_duration_ms": 400.0,
        "default_pitch_hz": 440.0,
        "osc_type": "saw",
        "osc_mix": {"saw": 0.4, "square": 0.3, "sine": 0.2, "noise": 0.1},
        "detune_amount": 5.0,
        "detune_layers": 2,
        "filter_envelope": "closing",
        "filter_cutoff_start": 1.0,
        "filter_cutoff_end": 0.1,
        "filter_resonance": 0.5,
        "attack_ms": 2,
        "decay_ms": 60,
        "sustain_level": 0.0,
        "release_ms": 100,
        "stereo_width": 0.0,
        "saturation": 0.2,
        "chord_density": 1,
        "pulse_width": 0.4,
    },
    "pad": {
        "label": "Synth Pad Hit",
        "default_duration_ms": 2000.0,
        "default_pitch_hz": 220.0,
        "osc_type": "saw",
        "osc_mix": {"saw": 0.5, "square": 0.2, "sine": 0.2, "noise": 0.1},
        "detune_amount": 3.0,
        "detune_layers": 7,
        "filter_envelope": "opening",
        "filter_cutoff_start": 0.1,
        "filter_cutoff_end": 1.0,
        "filter_resonance": 0.2,
        "attack_ms": 40,
        "decay_ms": 200,
        "sustain_level": 0.6,
        "release_ms": 500,
        "stereo_width": 0.7,
        "saturation": 0.2,
        "chord_density": 3,
        "pulse_width": 0.5,
    },
    "chord": {
        "label": "Synth Chord Hit",
        "default_duration_ms": 1500.0,
        "default_pitch_hz": 261.63,
        "osc_type": "square",
        "osc_mix": {"saw": 0.2, "square": 0.6, "sine": 0.1, "noise": 0.1},
        "detune_amount": 2.0,
        "detune_layers": 5,
        "filter_envelope": "slightly_opening",
        "filter_cutoff_start": 0.4,
        "filter_cutoff_end": 0.8,
        "filter_resonance": 0.3,
        "attack_ms": 10,
        "decay_ms": 300,
        "sustain_level": 0.3,
        "release_ms": 400,
        "stereo_width": 0.5,
        "saturation": 0.4,
        "chord_density": 3,
        "pulse_width": 0.3,
    },
    "lead": {
        "label": "Lead Hit",
        "default_duration_ms": 800.0,
        "default_pitch_hz": 440.0,
        "osc_type": "saw",
        "osc_mix": {"saw": 0.6, "square": 0.3, "sine": 0.0, "noise": 0.1},
        "detune_amount": 8.0,
        "detune_layers": 3,
        "filter_envelope": "opening",
        "filter_cutoff_start": 0.2,
        "filter_cutoff_end": 1.0,
        "filter_resonance": 0.6,
        "attack_ms": 3,
        "decay_ms": 120,
        "sustain_level": 0.3,
        "release_ms": 300,
        "stereo_width": 0.3,
        "saturation": 0.6,
        "chord_density": 1,
        "pulse_width": 0.5,
    },
    "bass": {
        "label": "Synth Bass Stab",
        "default_duration_ms": 500.0,
        "default_pitch_hz": 80.0,
        "osc_type": "saw",
        "osc_mix": {"saw": 0.5, "square": 0.3, "sine": 0.2, "noise": 0.0},
        "detune_amount": 2.0,
        "detune_layers": 2,
        "filter_envelope": "closing",
        "filter_cutoff_start": 0.6,
        "filter_cutoff_end": 0.1,
        "filter_resonance": 0.4,
        "attack_ms": 8,
        "decay_ms": 150,
        "sustain_level": 0.1,
        "release_ms": 200,
        "stereo_width": 0.0,
        "saturation": 0.5,
        "chord_density": 1,
        "pulse_width": 0.5,
    },
}


# ─── Note helpers for chords ────────────────────────────

# Intervals for chord types (semitones from root)
CHORD_INTERVALS = {
    1: [0],                    # single note
    2: [0, 7],                 # power chord
    3: [0, 4, 7],              # major triad
    4: [0, 4, 7, 11],          # major 7th
}


def _semitone_to_ratio(semitones: float) -> float:
    """Convert semitone interval to frequency ratio."""
    return 2.0 ** (semitones / 12.0)


def _chord_frequencies(root_hz: float, density: int) -> list[float]:
    """Generate chord frequencies from root. density=1..4 maps to chord types."""
    intervals = CHORD_INTERVALS.get(min(density, 4), [0])
    return [root_hz * _semitone_to_ratio(s) for s in intervals]


# ─── Oscillator mix ─────────────────────────────────────

def _oscillator_mix(t: float, freqs: list[float], profile: dict, sample_idx: int) -> tuple[float, float]:
    """Compute L and R values from the oscillator mix. Returns (L, R)."""
    osc_mix = profile["osc_mix"]
    pw = profile.get("pulse_width", 0.5)
    d = profile.get("detune_amount", 0.0) / 100.0  # cents → ratio

    left = 0.0
    right = 0.0
    stereo = profile.get("stereo_width", 0.0)
    seed = profile["default_pitch_hz"]

    for freq in freqs:
        # Detune layers
        n_layers = profile.get("detune_layers", 1)
        for layer in range(n_layers):
            detune_offset = (layer - (n_layers - 1) / 2.0) * d
            val = 0.0
            for osc_type, mix in osc_mix.items():
                if mix <= 0:
                    continue
                if osc_type == "saw":
                    val += osc_saw(t, freq, detune_offset) * mix
                elif osc_type == "square":
                    val += osc_square(t, freq, pw, detune_offset) * mix
                elif osc_type == "sine":
                    val += osc_sine(t, freq, detune_offset) * mix
                elif osc_type == "noise":
                    val += osc_noise(t, freq + seed + layer) * mix

            # Per-layer panning for stereo width
            if stereo > 0 and n_layers > 1:
                pan = (layer / max(n_layers - 1, 1)) * 2.0 - 1.0  # -1..1
                left += val * (1.0 - stereo * max(pan, 0.0))
                right += val * (1.0 - stereo * max(-pan, 0.0))
            else:
                left += val
                right += val

    return left, right


# ─── Synthesis function ─────────────────────────────────

ADSR_NOTES = ["off", "attack", "decay", "sustain", "release"]


def _adsr_value(t: float, profile: dict, num_samples: int) -> float:
    """Compute ADSR envelope at time t."""
    sr = SAMPLE_RATE
    a = profile["attack_ms"] / 1000.0
    d = profile["decay_ms"] / 1000.0
    s = profile["sustain_level"]
    r = profile["release_ms"] / 1000.0
    total = num_samples / sr

    if t < a:
        return t / max(a, 0.001)  # linear attack
    elif t < a + d:
        dt = (t - a) / max(d, 0.001)
        return 1.0 - (1.0 - s) * dt  # decay to sustain
    elif t < total - r:
        return s  # sustain
    else:
        rt = (t - (total - r)) / max(r, 0.001)
        return s * (1.0 - rt)  # release


def _filter_cutoff(t: float, profile: dict) -> float:
    """Compute time-varying filter cutoff (0-1 normalized)."""
    total_ms = profile["default_duration_ms"]
    env = profile["filter_envelope"]
    start = profile["filter_cutoff_start"]
    end = profile["filter_cutoff_end"]
    dur = total_ms / 1000.0

    frac = min(t / max(dur, 0.001), 1.0)

    if env == "opening":
        return start + (end - start) * frac
    elif env == "closing":
        return start + (end - start) * (1.0 - frac)
    elif env == "slightly_opening":
        return start + (end - start) * (frac ** 0.5)
    else:
        return start + (end - start) * frac


def synthesize_synth(duration_ms: float = 600.0, pitch_hz: float = 220.0,
                     profile_name: str = "stab", profiles: dict = None,
                     overrides: dict = None) -> np.ndarray:
    """Generate a synth one-shot with the given profile and optional overrides."""
    profile = dict(SYNTH_PROFILES.get(profile_name, SYNTH_PROFILES["stab"]))
    if overrides:
        profile.update(overrides)

    dur = duration_ms
    if "default_duration_ms" in profile and overrides is None:
        dur = duration_ms

    num_samples = max(100, int(SAMPLE_RATE * dur / 1000.0))

    # Chord mode
    freqs = _chord_frequencies(pitch_hz, profile["chord_density"])

    # Filter parameters
    cutoff_min = 40.0
    cutoff_max = SAMPLE_RATE / 2.0
    resonance = profile["filter_resonance"]

    # Precompute noise oscillator IDs
    noise_id = random.randint(0, 1000)

    samples = np.zeros(num_samples)
    filter_hist = 0.0

    for i in range(num_samples):
        t = i / SAMPLE_RATE

        left, right = _oscillator_mix(t, freqs, profile, i)
        val = (left + right) / (max(len(freqs), 1) * max(profile.get("detune_layers", 1), 1))

        env = _adsr_value(t, profile, num_samples)
        val *= env

        # Brightness override → shift filter cutoff
        brightness_override = profile.get("brightness", 1.0)
        cutoff_norm = _filter_cutoff(t, profile)
        if brightness_override != 1.0:
            if brightness_override > 1.0:
                cutoff_norm = min(1.0, cutoff_norm * brightness_override)
            else:
                cutoff_norm = cutoff_norm * brightness_override
        cutoff_hz = cutoff_min + (cutoff_max - cutoff_min) * cutoff_norm
        alpha = min(1.0, 2.0 * math.pi * cutoff_hz / SAMPLE_RATE)
        alpha = min(alpha, 0.99)
        filter_hist = filter_hist + alpha * (val - filter_hist)
        val = filter_hist + resonance * (val - filter_hist) * 0.5

        sat = profile.get("saturation", 0.0)
        drive_amt = profile.get("drive", 0.0)
        total_sat = max(sat, drive_amt)
        if total_sat > 0.01:
            drive = 1.0 + total_sat * 4.0
            val = tape_saturation(val, drive)

        samples[i] = val

    # Post-processing EQ
    if profile.get("high_shelf_db", 0) != 0:
        samples = biquad_high_shelf(samples, 4000.0, profile["high_shelf_db"], 0.6)
    if profile.get("low_shelf_db", 0) != 0:
        samples = biquad_low_shelf(samples, 200.0, profile["low_shelf_db"], 0.7)

    # Bandwidth reduction (lo-fi, dark)
    bw = profile.get("bandwidth_reduce", 0.0)
    if bw > 0.01:
        cutoff = 10000.0 * (1.0 - bw * 0.88)
        cutoff = max(cutoff, 1000.0)
        sos = sp_signal.butter(4, cutoff, 'lowpass', fs=SAMPLE_RATE, output='sos')
        samples = sp_signal.sosfilt(sos, samples)

    # Stereo widening (before final noise to keep stereo image clean)
    stereo = profile.get("stereo_width", 0.0)
    if stereo > 0.01:
        stereo_out = np.zeros((num_samples, 2), dtype=np.float32)
        delay = max(1, int(0.001 * stereo * SAMPLE_RATE))
        for i in range(num_samples):
            L = samples[i] * (1.0 - stereo * 0.2)
            R = samples[max(0, i - delay)] * (1.0 - stereo * 0.2) * (1.0 - stereo * 0.15)
            stereo_out[i, 0] = L
            stereo_out[i, 1] = R
        for ch in range(2):
            pk = np.max(np.abs(stereo_out[:, ch]))
            if pk > 0:
                stereo_out[:, ch] /= pk * 1.1

        # Post-normalization lo-fi / noise for stereo
        nf = profile.get("noise_floor", 0.0)
        if nf > 0.001:
            for ch in range(2):
                stereo_out[:, ch] += np.random.randn(num_samples).astype(np.float32) * nf * 0.03
        lo_fi = profile.get("lo_fi", 0.0)
        if lo_fi > 0.1:
            bits = max(4, int(16 - lo_fi * 12))
            levels = 2 ** bits
            stereo_out = np.round(stereo_out * levels) / levels
        return stereo_out.astype(np.float32)

    # Mono normalize
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9

    # Post-normalization lo-fi / noise for mono
    nf = profile.get("noise_floor", 0.0)
    if nf > 0.001:
        samples += np.random.randn(len(samples)).astype(np.float32) * nf * 0.03
    lo_fi = profile.get("lo_fi", 0.0)
    if lo_fi > 0.1:
        bits = max(4, int(16 - lo_fi * 12))
        levels = 2 ** bits
        samples = np.round(samples * levels) / levels

    return samples.astype(np.float32)


# ─── CLI commands ───────────────────────────────────────

def cmd_synth_gen(args):
    """Generate synth samples from named profiles with optional control overrides."""
    profile_name = args.profile
    if profile_name not in SYNTH_PROFILES and profile_name != "all":
        print(f"Unknown profile: {profile_name}")
        print(f"Valid: {', '.join(sorted(SYNTH_PROFILES.keys()))}, all")
        sys.exit(1)

    count = args.count
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    # Parse control overrides
    overrides = {}
    if args.detune is not None:
        overrides["detune_amount"] = args.detune
    if args.filter_env:
        overrides["filter_envelope"] = args.filter_env
    if args.attack_ms is not None:
        overrides["attack_ms"] = args.attack_ms
    if args.decay_ms is not None:
        overrides["decay_ms"] = args.decay_ms
    if args.sustain is not None:
        overrides["sustain_level"] = args.sustain
    if args.stereo is not None:
        overrides["stereo_width"] = args.stereo
    if args.saturation is not None:
        overrides["saturation"] = args.saturation
    if args.chord is not None:
        overrides["chord_density"] = args.chord
    if args.osc:
        parts = args.osc.split(",")
        osc_mix = {}
        for p in parts:
            k, v = p.split("=")
            osc_mix[k.strip()] = float(v.strip())
        if osc_mix:
            overrides["osc_mix"] = osc_mix

    profiles_to_gen = list(SYNTH_PROFILES.keys()) if profile_name == "all" else [profile_name]
    seed_offset = int(time.time() * 1000) % 1000000

    for pn in profiles_to_gen:
        pf = SYNTH_PROFILES[pn]
        label = pf["label"]
        default_dur = pf["default_duration_ms"]
        default_pitch = pf["default_pitch_hz"]
        out_subdir = out_dir / pn if profile_name == "all" else out_dir
        out_subdir.mkdir(parents=True, exist_ok=True)

        print(f"Generating {count} {label} → {out_subdir}...")

        for i in range(count):
            seed = (seed_offset + i) * 314159265 + hash(pn) % 1000000
            random.seed(seed)
            np.random.seed(seed % 2**32)

            dur_var = default_dur * (1.0 + (random.random() - 0.5) * 0.2)
            pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * 0.15)
            samples = synthesize_synth(dur_var, pitch_var, pn, overrides=overrides if overrides else None)

            out_path = out_subdir / f"synth_{pn}_{i+1:03d}.wav"
            write_wav(out_path, samples)

            if i == 0 or (i + 1) % 5 == 0:
                print(f"  [{i+1}/{count}] {out_path.name}")

        print(f"  Done: {count} {label} → {out_subdir}")


def cmd_synth_refine(args):
    """Synth refinement: diagnose issues and suggest parameter adjustments."""
    from gen.io import read_wav
    from gen.features import compute_features, compute_spectral_centroid, compute_hpr
    from gen.features import compute_attack_time, compute_decay_length
    from gen.features import detect_pitch_full

    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = sorted(in_dir.glob("*.wav"))
    if not wav_files:
        print(f"Error: no .wav files in {in_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Synth refinement: analyzing {len(wav_files)} files...\n")

    # Aggregate features
    all_feats = []
    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            continue
        samples, sr = result
        feats = compute_features(samples, sr)
        feats["file"] = wav_path.name
        pitch_info = detect_pitch_full(samples, sr)
        feats["pitch_hz"] = pitch_info["pitch_hz"]
        feats["pitch_conf"] = pitch_info["confidence"]
        all_feats.append(feats)

    if not all_feats:
        print("No files could be analyzed")
        return

    # Compute averages
    avg_feats = {}
    for k in ["spectral_centroid", "low_band_energy", "high_band_energy",
               "transient_count", "decay_length_ms", "attack_ms", "rms",
               "zero_crossing_rate", "pitch_hz", "pitch_conf"]:
        vals = [f.get(k, 0) for f in all_feats]
        avg_feats[k] = float(np.mean(vals))

    print("  Aggregate diagnosis (averaged across files):\n")

    diagnoses = []
    suggestions = []

    # Brightness
    if avg_feats["spectral_centroid"] < 1500:
        diagnoses.append(f"too dark (centroid={avg_feats['spectral_centroid']:.0f}Hz < 1500Hz)")
        suggestions.append("  → Increase filter cutoff end, reduce low-pass, or boost high shelf +3dB")

    if avg_feats["spectral_centroid"] > 6000:
        diagnoses.append(f"too bright/edgy (centroid={avg_feats['spectral_centroid']:.0f}Hz > 6000Hz)")
        suggestions.append("  → Reduce filter cutoff, increase low-pass, or cut high shelf -3dB")

    # Noise content
    zcr = avg_feats["zero_crossing_rate"]
    if zcr > 0.3:
        diagnoses.append(f"too noisy (ZCR={zcr:.3f} > 0.3)")
        suggestions.append("  → Reduce noise oscillator mix, increase sine/square proportion")

    # Thinness
    low_band = avg_feats["low_band_energy"]
    if low_band < 0.05 and avg_feats["spectral_centroid"] > 3000:
        diagnoses.append(f"too thin (low_band={low_band:.3f} < 0.05)")
        suggestions.append("  → Boost low shelf +3dB, add sub oscillator, spread chord wider")

    # Transients
    if avg_feats["transient_count"] < 0.5:
        diagnoses.append(f"too few transients (count={avg_feats['transient_count']:.1f})")
        suggestions.append("  → Shorten attack time, add noise attack layer, increase filter snap")

    if avg_feats["transient_count"] > 10:
        diagnoses.append(f"too many transients (count={avg_feats['transient_count']:.1f})")
        suggestions.append("  → Lengthen attack, smooth filter envelope, reduce noise content")

    # Decay
    if avg_feats["decay_length_ms"] < 10:
        diagnoses.append(f"decay too short ({avg_feats['decay_length_ms']:.1f}ms)")
        suggestions.append("  → Increase release time, raise sustain level, slow filter decay")

    if avg_feats["decay_length_ms"] > 2000:
        diagnoses.append(f"decay too long ({avg_feats['decay_length_ms']:.0f}ms)")
        suggestions.append("  → Shorten release, lower sustain, gate the tail")

    # Stability (pitch)
    if avg_feats.get("pitch_conf", 0) < 0.4 and avg_feats.get("pitch_hz", 0) > 50:
        diagnoses.append(f"pitch too unstable (confidence={avg_feats['pitch_conf']:.2f})")
        suggestions.append("  → Reduce detuning, narrow stereo width, reduce filter modulation")

    # Variation
    variations = {}
    for k in ["spectral_centroid", "rms", "transient_count"]:
        vals = [f.get(k, 0) for f in all_feats]
        if max(vals) > 0:
            cv = np.std(vals) / max(np.mean(vals), 1e-10)
            variations[k] = cv
    low_var = [k for k, v in variations.items() if v < 0.05]
    if low_var:
        det_str = ", ".join(f"{k} (CV={variations[k]:.3f})" for k in low_var)
        diagnoses.append(f"not enough variation ({det_str})")
        suggestions.append("  → Widen pitch randomization, add more detune layers, vary filter cutoff")

    # Print
    if not diagnoses:
        print("  ✓ No issues detected. Synth sounds well-balanced.")
    else:
        for d in diagnoses:
            print(f"  ✗ {d}")
        print()
        for s in suggestions:
            print(f"  {s}")

    # Generate refined samples if --out specified
    if args.out:
        out_dir = Path(args.out)
        count = args.count
        print(f"\n  Generating {count} refined samples → {out_dir}...")
        out_dir.mkdir(parents=True, exist_ok=True)

        # Map diagnoses to overrides
        overrides = {}
        for d in diagnoses:
            if "too dark" in d:
                overrides["filter_cutoff_end"] = 1.0
                overrides["filter_envelope"] = "opening"
            elif "too bright" in d:
                overrides["filter_cutoff_end"] = 0.5
            elif "too noisy" in d:
                if "osc_mix" in overrides:
                    overrides["osc_mix"]["noise"] = 0.0
                else:
                    overrides["osc_mix"] = {"saw": 0.5, "square": 0.3, "sine": 0.2, "noise": 0.0}
            elif "too thin" in d:
                overrides["chord_density"] = 2
            elif "too few transients" in d:
                overrides["attack_ms"] = 2
            elif "too many transients" in d:
                overrides["attack_ms"] = 20
            elif "decay too short" in d:
                overrides["release_ms"] = 600
                overrides["sustain_level"] = 0.3
            elif "decay too long" in d:
                overrides["release_ms"] = 100
                overrides["sustain_level"] = 0.0
            elif "too unstable" in d:
                overrides["detune_amount"] = 1.0
                overrides["stereo_width"] = 0.0
            elif "not enough variation" in d:
                pass

        profile_name = args.target or "stab"
        seed_offset = int(time.time() * 1000) % 1000000
        pf = SYNTH_PROFILES.get(profile_name, SYNTH_PROFILES["stab"])
        for i in range(count):
            seed = (seed_offset + i) * 314159265 + hash(profile_name) % 1000000
            random.seed(seed)
            np.random.seed(seed % 2**32)
            dur_var = pf["default_duration_ms"] * (1.0 + (random.random() - 0.5) * 0.2)
            pitch_var = pf["default_pitch_hz"] * (1.0 + (random.random() - 0.5) * 0.15)
            samples = synthesize_synth(dur_var, pitch_var, profile_name, overrides=overrides)

            out_path = out_dir / f"synth_refined_{profile_name}_{i+1:03d}.wav"
            write_wav(out_path, samples)
            if i == 0 or (i + 1) % 5 == 0:
                print(f"    [{i+1}/{count}] {out_path.name}")
        print(f"  Done: {count} refined → {out_dir}")
