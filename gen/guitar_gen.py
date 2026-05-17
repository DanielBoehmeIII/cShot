"""Guitar / plucked instrument generation with Karplus-Strong synthesis."""

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


# ─── Karplus-Strong plucked string ──────────────────────

def _ks_pluck(pitch_hz: float, duration_s: float, damping: float = 0.99,
              pick_position: float = 0.1, brightness: float = 0.5) -> np.ndarray:
    """Karplus-Strong plucked string synthesis.
    
    Args:
        pitch_hz: Fundamental frequency
        duration_s: Duration in seconds
        damping: String damping (0.9-0.999). Lower = faster decay
        pick_position: Where the string is plucked (0-1). Lower = brighter
        brightness: Lowpass filter coefficient on the string (0-1). Higher = brighter
    """
    num_samples = max(100, int(SAMPLE_RATE * duration_s))
    delay = max(1, int(SAMPLE_RATE / max(pitch_hz, 1)))

    # Initialize the delay line with noise
    buf = np.random.randn(delay).astype(np.float32) * 0.5

    # Pick position comb filter (simulates pluck position)
    pick_delay = max(1, int(delay * pick_position))
    comb = np.zeros(pick_delay + 1)
    comb[0] = 1.0
    comb[-1] = -1.0

    samples = np.zeros(num_samples)
    lp_state = 0.0
    lp_coeff = 1.0 - brightness * 0.9

    for i in range(num_samples):
        idx = i % delay
        val = buf[idx]

        # Lowpass filter (string stiffness)
        lp_state = lp_state + lp_coeff * (val - lp_state)

        # Pick-position comb
        pick_val = 0.0
        for j in range(len(comb)):
            src = (idx - j) % delay
            pick_val += comb[j] * buf[src]

        # Writeback with damping
        write_val = lp_state * damping
        buf[idx] = write_val

        samples[i] = pick_val

    return samples


def synthesize_guitar_stab(duration_ms: float = 500.0, pitch_hz: float = 220.0,
                           profile_name: str = "nylon", profiles: dict = None) -> np.ndarray:
    """Generate a guitar/plucked one-shot stab from the named profile."""
    pf = GUITAR_PROFILES.get(profile_name, GUITAR_PROFILES["nylon"])
    dur_s = duration_ms / 1000.0

    # Karplus-Strong pluck
    samples = _ks_pluck(
        pitch_hz=pitch_hz,
        duration_s=dur_s,
        damping=pf["damping"],
        pick_position=pf["pick_position"],
        brightness=pf["brightness"],
    )

    num_samples = len(samples)

    # ADSR body envelope
    attack_s = max(1, int(pf["attack_ms"] * SAMPLE_RATE / 1000.0))
    decay_s = max(attack_s + 1, int(pf["decay_ms"] * SAMPLE_RATE / 1000.0))
    release_s = max(decay_s + 1, int(num_samples - pf["release_ms"] * SAMPLE_RATE / 1000.0))
    sustain_level = pf["sustain_level"]

    for i in range(num_samples):
        if i < attack_s:
            p = i / attack_s
            samples[i] *= p
        elif i < decay_s:
            t = (i - attack_s) / max(decay_s - attack_s, 1)
            samples[i] *= 1.0 - (1.0 - sustain_level) * t
        elif i < release_s:
            samples[i] *= sustain_level
        else:
            t = (i - release_s) / max(num_samples - release_s, 1)
            samples[i] *= sustain_level * (1.0 - t) ** 2

    # Body resonance (soundboard simulation)
    if pf["body_resonance_hz"] > 0:
        res_len = min(num_samples, int(0.5 * SAMPLE_RATE))
        res = np.zeros(res_len)
        for i in range(res_len):
            t = i / SAMPLE_RATE
            res[i] = math.sin(2 * math.pi * pf["body_resonance_hz"] * t) * math.exp(-4.0 * t)
        res = res * pf["body_resonance_amp"]
        for i in range(res_len):
            samples[i] += res[i]

    # Pluck noise (finger/nail sound)
    if pf["pluck_noise_amp"] > 0:
        noise_len = min(num_samples, int(0.010 * SAMPLE_RATE))
        for i in range(noise_len):
            t = i / SAMPLE_RATE
            n = np.random.randn() * pf["pluck_noise_amp"] * math.exp(-200.0 * t)
            samples[i] += n

    # Global lowpass (tames KS brightness)
    sos = sp_signal.butter(2, 6000, 'lowpass', fs=SAMPLE_RATE, output='sos')
    samples = sp_signal.sosfilt(sos, samples)

    # Processing
    if pf["low_shelf_db"] != 0:
        samples = biquad_low_shelf(samples, 200.0, pf["low_shelf_db"], 0.7)
    if pf["high_shelf_db"] != 0:
        samples = biquad_high_shelf(samples, 3000.0, pf["high_shelf_db"], 0.7)
    if pf["saturation"] > 0:
        drive = 1.0 + pf["saturation"] * 4.0
        for i in range(len(samples)):
            samples[i] = tape_saturation(samples[i], drive)

    # Reverse mode
    if pf.get("reversed", False):
        samples = samples[::-1]

    # Chopped mode (short gate)
    if pf.get("chopped", False):
        chop_len = min(num_samples, int(0.100 * SAMPLE_RATE))
        if chop_len < num_samples:
            samples[chop_len:] = 0.0

    # Normalize
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    return samples.astype(np.float32)


# ─── Guitar profiles ────────────────────────────────────

GUITAR_PROFILES = {
    "nylon": {
        "label": "Nylon Guitar",
        "default_duration_ms": 800.0,
        "default_pitch_hz": 220.0,
        "damping": 0.997,
        "pick_position": 0.3,
        "brightness": 0.15,
        "attack_ms": 5.0,
        "decay_ms": 120.0,
        "sustain_level": 0.10,
        "release_ms": 250.0,
        "body_resonance_hz": 100.0,
        "body_resonance_amp": 0.10,
        "pluck_noise_amp": 0.03,
        "low_shelf_db": 3.0,
        "high_shelf_db": -3.0,
        "saturation": 0.0,
        "reversed": False,
        "chopped": False,
    },
    "muted": {
        "label": "Muted Guitar",
        "default_duration_ms": 300.0,
        "default_pitch_hz": 220.0,
        "damping": 0.965,
        "pick_position": 0.35,
        "brightness": 0.3,
        "attack_ms": 1.0,
        "decay_ms": 30.0,
        "sustain_level": 0.0,
        "release_ms": 50.0,
        "body_resonance_hz": 0.0,
        "body_resonance_amp": 0.0,
        "pluck_noise_amp": 0.08,
        "low_shelf_db": -2.0,
        "high_shelf_db": -2.0,
        "saturation": 0.1,
        "reversed": False,
        "chopped": False,
    },
    "bright": {
        "label": "Bright Guitar",
        "default_duration_ms": 700.0,
        "default_pitch_hz": 330.0,
        "damping": 0.995,
        "pick_position": 0.15,
        "brightness": 0.6,
        "attack_ms": 2.0,
        "decay_ms": 80.0,
        "sustain_level": 0.12,
        "release_ms": 200.0,
        "body_resonance_hz": 200.0,
        "body_resonance_amp": 0.05,
        "pluck_noise_amp": 0.05,
        "low_shelf_db": 0.0,
        "high_shelf_db": 3.0,
        "saturation": 0.0,
        "reversed": False,
        "chopped": False,
    },
    "dark": {
        "label": "Dark Guitar",
        "default_duration_ms": 1000.0,
        "default_pitch_hz": 165.0,
        "damping": 0.998,
        "pick_position": 0.4,
        "brightness": 0.1,
        "attack_ms": 8.0,
        "decay_ms": 200.0,
        "sustain_level": 0.08,
        "release_ms": 350.0,
        "body_resonance_hz": 80.0,
        "body_resonance_amp": 0.12,
        "pluck_noise_amp": 0.02,
        "low_shelf_db": 4.0,
        "high_shelf_db": -4.0,
        "saturation": 0.05,
        "reversed": False,
        "chopped": False,
    },
    "processed": {
        "label": "Processed Guitar",
        "default_duration_ms": 800.0,
        "default_pitch_hz": 220.0,
        "damping": 0.996,
        "pick_position": 0.25,
        "brightness": 0.4,
        "attack_ms": 4.0,
        "decay_ms": 120.0,
        "sustain_level": 0.08,
        "release_ms": 250.0,
        "body_resonance_hz": 100.0,
        "body_resonance_amp": 0.08,
        "pluck_noise_amp": 0.04,
        "low_shelf_db": 1.0,
        "high_shelf_db": 1.0,
        "saturation": 0.5,
        "reversed": False,
        "chopped": False,
    },
    "reversed": {
        "label": "Reversed Guitar",
        "default_duration_ms": 800.0,
        "default_pitch_hz": 220.0,
        "damping": 0.997,
        "pick_position": 0.3,
        "brightness": 0.2,
        "attack_ms": 3.0,
        "decay_ms": 100.0,
        "sustain_level": 0.10,
        "release_ms": 200.0,
        "body_resonance_hz": 120.0,
        "body_resonance_amp": 0.10,
        "pluck_noise_amp": 0.03,
        "low_shelf_db": 2.0,
        "high_shelf_db": -1.0,
        "saturation": 0.2,
        "reversed": True,
        "chopped": False,
    },
    "chopped": {
        "label": "Chopped Guitar",
        "default_duration_ms": 800.0,
        "default_pitch_hz": 220.0,
        "damping": 0.995,
        "pick_position": 0.3,
        "brightness": 0.3,
        "attack_ms": 2.0,
        "decay_ms": 80.0,
        "sustain_level": 0.10,
        "release_ms": 150.0,
        "body_resonance_hz": 120.0,
        "body_resonance_amp": 0.08,
        "pluck_noise_amp": 0.04,
        "low_shelf_db": 1.0,
        "high_shelf_db": 0.0,
        "saturation": 0.1,
        "reversed": False,
        "chopped": True,
    },
}


# ─── CLI commands ───────────────────────────────────────

def cmd_guitar_gen(args):
    """Generate guitar/plucked samples from named profiles."""
    profile_name = args.profile
    if profile_name not in GUITAR_PROFILES and profile_name != "all":
        print(f"Unknown profile: {profile_name}")
        print(f"Valid: {', '.join(sorted(GUITAR_PROFILES.keys()))}, all")
        sys.exit(1)

    count = args.count
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    profiles_to_gen = list(GUITAR_PROFILES.keys()) if profile_name == "all" else [profile_name]
    seed_offset = int(time.time() * 1000) % 1000000

    for pn in profiles_to_gen:
        pf = GUITAR_PROFILES[pn]
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
            samples = synthesize_guitar_stab(dur_var, pitch_var, pn)

            out_path = out_subdir / f"guitar_{pn}_{i+1:03d}.wav"
            write_wav(out_path, samples)

            if i == 0 or (i + 1) % 5 == 0:
                print(f"  [{i+1}/{count}] {out_path.name}")

        print(f"  Done: {count} {label} → {out_subdir}")


def cmd_guitar_qa(args):
    """Guitar QA: verify generated guitar stabs match pitch, transient, decay, body range."""
    from gen.io import read_wav
    from gen.features import compute_features, compute_attack_time, compute_decay_length, detect_pitch_full, hz_to_note
    from gen.refinement import feature_distance

    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = sorted(in_dir.glob("*.wav"))
    if not wav_files:
        print(f"Error: no .wav files in {in_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"Guitar QA: {len(wav_files)} files\n")

    print(f"  {'File':<42} {'Note':>8} {'Pitch':>8} {'Attack':>8} {'Decay':>8} {'Centroid':>10} {'Pluck':>6}")
    print(f"  {'─'*42} {'─'*8} {'─'*8} {'─'*8} {'─'*8} {'─'*10} {'─'*6}")

    results = []
    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            continue
        samples, sr = result
        feats = compute_features(samples, sr)
        pitch_info = detect_pitch_full(samples, sr)
        attack_ms = compute_attack_time(samples, sr)
        decay_ms = compute_decay_length(samples, sr)

        has_pitch = pitch_info["confidence"] > 0.3
        has_transient = feats["transient_count"] >= 1
        has_decay = decay_ms >= 0.05
        has_body = 500 < feats["spectral_centroid"] < 8000

        is_good = has_pitch and has_transient and has_decay and has_body

        results.append({
            "file": wav_path.name,
            "pitch_hz": pitch_info["pitch_hz"],
            "note": pitch_info["note_name"],
            "attack_ms": attack_ms,
            "decay_ms": decay_ms,
            "centroid": feats["spectral_centroid"],
            "transient_count": feats["transient_count"],
            "has_pitch": has_pitch,
            "has_transient": has_transient,
            "has_decay": has_decay,
            "has_body": has_body,
            "is_good": is_good,
        })

        pk = pitch_info["pitch_hz"]
        nt = pitch_info["note_name"]
        a = attack_ms
        d = decay_ms
        c = feats["spectral_centroid"]
        tr = feats["transient_count"]
        icon = "✓" if is_good else "△"
        print(f"  {icon} {wav_path.name:<40s} {nt:>8s} {pk:>7.1f}Hz "
              f"{a:>6.2f}ms {d:>6.2f}ms {c:>8.0f}Hz {tr:>4.0f}")

    good = sum(1 for r in results if r["is_good"])
    print(f"\n  {'='*55}")
    print(f"  GUITAR QA SUMMARY")
    print(f"  {'='*55}")
    print(f"  Files tested: {len(results)}")
    print(f"  Good:         {good}/{len(results)}")
    has_p = sum(1 for r in results if r["has_pitch"])
    has_t = sum(1 for r in results if r["has_transient"])
    has_d = sum(1 for r in results if r["has_decay"])
    has_b = sum(1 for r in results if r["has_body"])
    print(f"  Has pitch:    {has_p}/{len(results)}")
    print(f"  Has transient:{has_t}/{len(results)}")
    print(f"  Has decay:    {has_d}/{len(results)}")
    print(f"  Has body:     {has_b}/{len(results)}")

    if args.output:
        import json, time
        out = {
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "files_tested": len(results),
            "good": good,
            "results": results,
        }
        Path(args.output).write_text(json.dumps(out, indent=2))
        print(f"\n  Results: {args.output}")
