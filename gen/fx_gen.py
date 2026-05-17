"""FX/Impact/Texture generation: impacts, risers, downlifters, glitches, textures."""

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


# ─── FX Synthesis ───────────────────────────────────────

def synthesize_fx(duration_ms: float = 1000.0, pitch_hz: float = 80.0,
                  profile_name: str = "impact", overrides: dict = None) -> np.ndarray:
    """Generate an FX one-shot from the named profile."""
    pf = dict(FX_PROFILES.get(profile_name, FX_PROFILES["impact"]))
    if overrides:
        pf.update(overrides)

    dur_s = duration_ms / 1000.0
    num_samples = max(100, int(SAMPLE_RATE * dur_s))
    samples = np.zeros(num_samples)
    ftype = pf["type"]

    # ── Impact: sub burst + noise attack + tail ──
    if ftype == "impact":
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            frac = i / num_samples

            # Sub burst
            freq = pitch_hz - pitch_hz * pf["pitch_drop"] * frac
            freq = max(freq, 20.0)
            sub = math.sin(2 * math.pi * freq * t)
            sub_env = math.exp(-pf["sub_decay"] * t)
            sub *= sub_env

            # Noise body
            n1 = noise_like(i * 0.5) * 0.5 + noise_like(i * 0.15) * 0.3
            noise_env = math.exp(-pf["noise_decay"] * t)
            noise_val = n1 * noise_env

            # Tail (reverb-like)
            tail = 0.0
            if t > pf["tail_start"]:
                tt = t - pf["tail_start"]
                tail = noise_like(i * 0.06) * math.exp(-pf["tail_decay"] * tt) * pf["tail_amp"]

            samples[i] = sub * pf["sub_mix"] + noise_val * pf["noise_mix"] + tail

        # Sub boost
        samples = biquad_low_shelf(samples, 80.0, 6.0, 0.7)

    # ── Riser: pitch + volume ramp up ──
    elif ftype == "riser":
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            frac = i / num_samples

            freq = pitch_hz * (1.0 + pf["pitch_rise"] * frac)
            env = frac ** pf["rise_curve"]

            s = math.sin(2 * math.pi * freq * t)
            n = noise_like(i * 0.3) * 0.3 + noise_like(i * 0.1) * 0.2
            samples[i] = (s * 0.5 + n * 0.5) * env

        # High shelf boost for air
        samples = biquad_high_shelf(samples, 3000.0, 4.0, 0.7)

    # ── Downlifter: pitch + volume ramp down ──
    elif ftype == "downlifter":
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            frac = i / num_samples

            freq = pitch_hz * (1.0 + pf["pitch_rise"] * (1.0 - frac))
            env = (1.0 - frac) ** pf["fall_curve"]

            s = math.sin(2 * math.pi * freq * t)
            n = noise_like(i * 0.3) * 0.4
            sub = math.sin(2 * math.pi * pitch_hz * 0.5 * t) * 0.6
            samples[i] = (s * 0.3 + n * 0.3 + sub * 0.4) * env

        samples = biquad_low_shelf(samples, 100.0, 5.0, 0.7)

    # ── Glitch: stuttering, bit-crushed noise bursts ──
    elif ftype == "glitch":
        grain_size = max(1, int(pf["grain_ms"] * SAMPLE_RATE / 1000.0))
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            grain_idx = (i // grain_size) % 2
            repeat_offset = i % grain_size

            src_idx = (grain_idx * grain_size + repeat_offset) % num_samples
            src_t = src_idx / SAMPLE_RATE

            val = noise_like(src_idx * 0.5) * 0.5 + math.sin(2 * math.pi * pitch_hz * src_t) * 0.3

            # Bit crushing
            bits = pf["bit_depth"]
            levels = 2 ** bits
            val = round(val * levels) / levels

            env = 1.0 - (i / max(num_samples, 1)) * pf["decay_rate"]
            samples[i] = val * max(env, 0)

    # ── Noise hit: shaped noise burst ──
    elif ftype == "noise_hit":
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            frac = i / num_samples

            n = noise_like(i * 0.7) * 0.6 + noise_like(i * 0.15) * 0.4
            env_shape = (1.0 - frac) ** pf["env_exponent"]
            if pf.get("hp_hz", 0) > 0:
                # Simple DC block via high-pass modeling
                n = n - noise_like(i * 0.71) * 0.5

            ton = math.sin(2 * math.pi * pitch_hz * t) * pf["tone_mix"]
            samples[i] = (n * (1.0 - pf["tone_mix"]) + ton) * env_shape

        if pf.get("hp_hz", 0) > 0:
            sos = sp_signal.butter(2, pf["hp_hz"], 'highpass', fs=SAMPLE_RATE, output='sos')
            samples = sp_signal.sosfilt(sos, samples)

    # ── Vinyl hit: crackle + thump ──
    elif ftype == "vinyl":
        for i in range(num_samples):
            t = i / SAMPLE_RATE

            # Initial thump
            thump = math.sin(2 * math.pi * 40.0 * t) * math.exp(-40.0 * t) * pf["thump_amp"]

            # Crackle
            crackle = 0.0
            if random.random() < pf["crackle_density"] * (1.0 / SAMPLE_RATE):
                crackle = (random.random() - 0.5) * pf["crackle_amp"]

            # Noise floor
            floor = noise_like(i * 0.5) * pf["noise_floor"]

            # Tone
            tone = math.sin(2 * math.pi * pitch_hz * t) * math.exp(-2.0 * t) * pf["tone_mix"]

            samples[i] = thump + crackle + floor + tone

    # ── Texture: air ──
    elif ftype == "air":
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            frac = i / num_samples
            n = noise_like(i * 0.99) * 0.5 + noise_like(i * 0.03) * 0.5
            env = math.exp(-pf["decay_rate"] * t) if pf["decay_rate"] > 0 else 1.0 - frac * 0.5
            samples[i] = n * env
        sos = sp_signal.butter(2, 2000, 'highpass', fs=SAMPLE_RATE, output='sos')
        samples = sp_signal.sosfilt(sos, samples)

    # ── Texture: crackle ──
    elif ftype == "crackle":
        crackle_rate = pf.get("crackle_density", 0.05)
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            val = 0.0
            if random.random() < crackle_rate:
                val = (random.random() - 0.5) * 0.5 * math.exp(-random.random() * 200 * t)
            val += noise_like(i * 0.5) * 0.01
            samples[i] = val

    # ── Texture: digital ──
    elif ftype == "digital":
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            frac = i / num_samples
            n = math.sin(2 * math.pi * pitch_hz * t) * 0.4
            n += math.sin(2 * math.pi * pitch_hz * 1.5 * t) * 0.2
            n += noise_like(i * 0.3) * 0.1
            bits = pf.get("bit_depth", 8)
            levels = 2 ** bits
            n = round(n * levels) / levels
            env = (1.0 - frac) ** 0.5
            samples[i] = n * env

    # ── Texture: analog ──
    elif ftype == "analog":
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            frac = i / num_samples
            n = noise_like(i * 0.5) * 0.3
            n += math.sin(2 * math.pi * pitch_hz * t) * 0.3
            n += noise_like(i * 0.03) * 0.2  # low frequency rumble
            n = tape_saturation(n, 1.5)
            env = math.exp(-pf["decay_rate"] * t)
            samples[i] = n * env

    # ── Texture: metallic ──
    elif ftype == "metallic":
        partials = [1.0, 2.7, 4.3, 5.9, 8.2, 11.1]
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            val = 0.0
            for h, ratio in enumerate(partials):
                decay = math.exp(-pf["decay_rate"] * t * (1.0 + h * 0.3))
                val += math.sin(2 * math.pi * pitch_hz * ratio * t) * (1.0 / (h + 1)) * decay
            samples[i] = val
        samples = biquad_high_shelf(samples, 3000.0, 6.0, 0.7)

    # ── Texture: organic ──
    elif ftype == "organic":
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            n = noise_like(i * 0.5) * 0.2
            n += math.sin(2 * math.pi * pitch_hz * t) * 0.15
            n += math.sin(2 * math.pi * pitch_hz * 2.01 * t) * 0.08
            n += noise_like(i * 0.01) * 0.1  # slow modulation
            env = math.exp(-pf["decay_rate"] * t) if pf["decay_rate"] > 0 else 1.0
            samples[i] = n * env

    # Normalize
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * pf.get("output_gain", 0.9)
    return samples.astype(np.float32)


# ─── FX Profiles ────────────────────────────────────────

FX_PROFILES = {
    # ── Impact family ──
    "impact": {
        "label": "Impact",
        "default_duration_ms": 1500.0, "default_pitch_hz": 70.0,
        "type": "impact", "pitch_drop": 0.6, "sub_decay": 2.5, "noise_decay": 12.0,
        "tail_start": 0.2, "tail_decay": 1.5, "tail_amp": 0.15,
        "sub_mix": 0.6, "noise_mix": 0.4, "output_gain": 0.9,
    },
    "sub_hit": {
        "label": "Sub Hit",
        "default_duration_ms": 1000.0, "default_pitch_hz": 50.0,
        "type": "impact", "pitch_drop": 0.4, "sub_decay": 1.5, "noise_decay": 30.0,
        "tail_start": 0.3, "tail_decay": 1.0, "tail_amp": 0.05,
        "sub_mix": 0.9, "noise_mix": 0.1, "output_gain": 0.9,
    },
    "noise_impact": {
        "label": "Noise Impact",
        "default_duration_ms": 800.0, "default_pitch_hz": 100.0,
        "type": "impact", "pitch_drop": 0.3, "sub_decay": 5.0, "noise_decay": 8.0,
        "tail_start": 0.3, "tail_decay": 2.0, "tail_amp": 0.2,
        "sub_mix": 0.2, "noise_mix": 0.8, "output_gain": 0.85,
    },
    # ── Riser / Downlifter ──
    "riser": {
        "label": "Riser",
        "default_duration_ms": 2000.0, "default_pitch_hz": 120.0,
        "type": "riser", "pitch_rise": 2.0, "rise_curve": 1.5, "output_gain": 0.85,
    },
    "downlifter": {
        "label": "Downlifter",
        "default_duration_ms": 2000.0, "default_pitch_hz": 200.0,
        "type": "downlifter", "pitch_rise": 0.8, "fall_curve": 0.7, "output_gain": 0.85,
    },
    # ── Glitch ──
    "glitch": {
        "label": "Glitch Hit",
        "default_duration_ms": 500.0, "default_pitch_hz": 200.0,
        "type": "glitch", "grain_ms": 20.0, "bit_depth": 8, "decay_rate": 0.5, "output_gain": 0.8,
    },
    "digital_glitch": {
        "label": "Digital Glitch",
        "default_duration_ms": 300.0, "default_pitch_hz": 400.0,
        "type": "glitch", "grain_ms": 8.0, "bit_depth": 4, "decay_rate": 0.7, "output_gain": 0.7,
    },
    # ── Noise hits ──
    "noise_hit": {
        "label": "Noise Hit",
        "default_duration_ms": 400.0, "default_pitch_hz": 1000.0,
        "type": "noise_hit", "env_exponent": 2.0, "tone_mix": 0.1, "hp_hz": 200, "output_gain": 0.8,
    },
    "tonal_hit": {
        "label": "Tonal Hit",
        "default_duration_ms": 600.0, "default_pitch_hz": 300.0,
        "type": "noise_hit", "env_exponent": 1.5, "tone_mix": 0.6, "hp_hz": 0, "output_gain": 0.85,
    },
    # ── Vinyl ──
    "vinyl": {
        "label": "Vinyl Hit",
        "default_duration_ms": 800.0, "default_pitch_hz": 200.0,
        "type": "vinyl", "thump_amp": 0.4, "crackle_density": 0.005, "crackle_amp": 0.3,
        "noise_floor": 0.02, "tone_mix": 0.15, "output_gain": 0.8,
    },
    # ── Textures ──
    "air": {
        "label": "Air Texture",
        "default_duration_ms": 2000.0, "default_pitch_hz": 100.0,
        "type": "air", "decay_rate": 0.5, "output_gain": 0.5,
    },
    "crackle": {
        "label": "Crackle",
        "default_duration_ms": 1000.0, "default_pitch_hz": 100.0,
        "type": "crackle", "crackle_density": 0.02, "output_gain": 0.6,
    },
    "digital": {
        "label": "Digital Texture",
        "default_duration_ms": 500.0, "default_pitch_hz": 300.0,
        "type": "digital", "bit_depth": 6, "output_gain": 0.7,
    },
    "analog": {
        "label": "Analog Texture",
        "default_duration_ms": 1000.0, "default_pitch_hz": 80.0,
        "type": "analog", "decay_rate": 3.0, "output_gain": 0.7,
    },
    "metallic": {
        "label": "Metallic Hit",
        "default_duration_ms": 800.0, "default_pitch_hz": 400.0,
        "type": "metallic", "decay_rate": 5.0, "output_gain": 0.8,
    },
    "organic": {
        "label": "Organic Texture",
        "default_duration_ms": 1200.0, "default_pitch_hz": 150.0,
        "type": "organic", "decay_rate": 2.0, "output_gain": 0.6,
    },
}


# ─── CLI ────────────────────────────────────────────────

def cmd_fx_gen(args):
    """Generate FX/impact/texture samples from named profiles."""
    profile_name = args.profile
    if profile_name not in FX_PROFILES and profile_name != "all":
        print(f"Unknown profile: {profile_name}")
        print(f"Valid: {', '.join(sorted(FX_PROFILES.keys()))}, all")
        sys.exit(1)

    count = args.count
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    seed_offset = int(time.time() * 1000) % 1000000

    profiles_to_gen = list(FX_PROFILES.keys()) if profile_name == "all" else [profile_name]
    for pn in profiles_to_gen:
        pf = FX_PROFILES[pn]
        out_subdir = out_dir / pn if profile_name == "all" else out_dir
        out_subdir.mkdir(parents=True, exist_ok=True)
        print(f"Generating {count} {pf['label']} → {out_subdir}...")

        for i in range(count):
            seed = (seed_offset + i) * 314159265 + hash(pn) % 1000000
            random.seed(seed)
            np.random.seed(seed % 2**32)
            dur_var = pf["default_duration_ms"] * (1.0 + (random.random() - 0.5) * 0.2)
            pitch_var = pf["default_pitch_hz"] * (1.0 + (random.random() - 0.5) * 0.15)
            samples = synthesize_fx(dur_var, pitch_var, pn)

            out_path = out_subdir / f"fx_{pn}_{i+1:03d}.wav"
            write_wav(out_path, samples)
            if i == 0 or (i + 1) % 5 == 0:
                print(f"  [{i+1}/{count}] {out_path.name}")
        print(f"  Done: {count} {pf['label']} → {out_subdir}")


def cmd_fx_qa(args):
    """FX QA: energy curve, decay shape, spectral motion, perceived size."""
    from gen.io import read_wav
    from gen.features import compute_features, compute_spectral_centroid, compute_hpr

    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = sorted(in_dir.glob("*.wav"))
    if not wav_files:
        print(f"Error: no .wav files in {in_dir}", file=sys.stderr)
        sys.exit(1)

    print(f"FX QA: {len(wav_files)} files\n")

    print(f"  {'File':<40} {'Duration':>10} {'Centroid':>10} {'HPR':>8} {'Energy':>10} {'Size':>8}")
    print(f"  {'─'*40} {'─'*10} {'─'*10} {'─'*8} {'─'*10} {'─'*8}")

    results = []
    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            continue
        samples, sr = result
        feats = compute_features(samples, sr)

        dur = feats["duration_ms"]
        centroid = feats["spectral_centroid"]
        hpr = compute_hpr(samples, sr)
        rms = feats["rms"]
        trans = feats["transient_count"]
        peak = feats["peak"]

        # Energy curve shape
        half = len(samples) // 2
        front_energy = float(np.sum(samples[:half] ** 2)) / max(half, 1)
        back_energy = float(np.sum(samples[half:] ** 2)) / max(len(samples) - half, 1)
        energy_ratio = front_energy / max(back_energy, 1e-10)

        if energy_ratio > 10:
            curve = "attack"
        elif energy_ratio > 3:
            curve = "decay"
        elif energy_ratio > 0.5:
            curve = "sustain"
        else:
            curve = "build"

        # Perceived size (combination of duration, low end, reverb-like tail)
        low_band = feats["low_band_energy"]
        size_score = min(1.0, (dur / 3000) * 0.4 + low_band * 0.3 + hpr * 0.3)
        if size_score > 0.7:
            size = "large"
        elif size_score > 0.4:
            size = "medium"
        else:
            size = "small"

        has_energy = rms > 0.01
        has_spectral_content = centroid > 50
        has_transient = trans >= 1
        has_no_clip = peak <= 0.99
        is_good = has_energy and has_spectral_content and has_transient and has_no_clip

        results.append({
            "file": wav_path.name,
            "duration_ms": round(dur, 1),
            "centroid_hz": round(centroid, 1),
            "hpr": round(hpr, 4),
            "rms": round(rms, 4),
            "energy_curve": curve,
            "size": size,
            "has_energy": has_energy,
            "has_spectral_content": has_spectral_content,
            "has_transient": has_transient,
            "is_good": is_good,
        })

        icon = "✓" if is_good else "△"
        print(f"  {icon} {wav_path.name:<38s} {dur:>7.0f}ms {centroid:>7.0f}Hz "
              f"{hpr:>6.2f} {curve:>8s} {size:>8s}")

    n = len(results)
    good = sum(1 for r in results if r["is_good"])
    print(f"\n  {'='*55}")
    print(f"  FX QA SUMMARY")
    print(f"  {'='*55}")
    print(f"  Files tested:   {n}")
    print(f"  Good:           {good}/{n}")

    from collections import Counter
    curves = Counter(r["energy_curve"] for r in results)
    sizes = Counter(r["size"] for r in results)
    print(f"  Energy curves:  {dict(curves)}")
    print(f"  Perceived size: {dict(sizes)}")

    if args.output:
        import json, time
        out = {
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "files_tested": n,
            "good": good,
            "results": results,
        }
        Path(args.output).write_text(json.dumps(out, indent=2))
        print(f"\n  Results: {args.output}")
