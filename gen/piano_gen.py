"""Piano / keys generation: acoustic piano, electric piano, Rhodes, bell profiles."""

import math
import random
import sys
import time
from pathlib import Path

import numpy as np
from scipy import signal as sp_signal

from gen import SAMPLE_RATE
from gen.dsp import (
    noise_like, envelope_adsr, tape_saturation, soft_clip,
    biquad_low_shelf, biquad_high_shelf, biquad_peaking,
)
from gen.io import write_wav


# ─── Piano Profiles ─────────────────────────────────────

PIANO_PROFILES = {
    "acoustic": {
        "label": "Acoustic Piano",
        "default_duration_ms": 1500.0,
        "default_pitch_hz": 261.63,
        # Harmonic amplitudes (harmonic_idx: amplitude)
        "harmonics": [
            (1, 1.00), (2, 0.60), (3, 0.45), (4, 0.30),
            (5, 0.18), (6, 0.10), (7, 0.06), (8, 0.03),
        ],
        # Decay rates per harmonic (higher harmonics decay faster)
        "harmonic_decay": [1.5, 2.0, 3.0, 4.0, 5.5, 7.5, 10.0, 13.0],
        "hammer_noise_amp": 0.12,
        "hammer_noise_duration_s": 0.003,
        "body_resonance_hz": 80.0,
        "body_resonance_amp": 0.06,
        "brightness": 1.0,
        "stereo_width": 0.0,
        "lo_fi": 0.0,
        "compression": 0.0,
    },
    "bright": {
        "label": "Bright Piano",
        "default_duration_ms": 1200.0,
        "default_pitch_hz": 261.63,
        "harmonics": [
            (1, 1.00), (2, 0.70), (3, 0.60), (4, 0.45),
            (5, 0.30), (6, 0.20), (7, 0.12), (8, 0.08),
        ],
        "harmonic_decay": [3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 14.0, 18.0],
        "hammer_noise_amp": 0.18,
        "hammer_noise_duration_s": 0.004,
        "body_resonance_hz": 120.0,
        "body_resonance_amp": 0.04,
        "brightness": 1.5,
        "stereo_width": 0.0,
        "lo_fi": 0.0,
        "compression": 0.0,
    },
    "dark": {
        "label": "Dark Piano",
        "default_duration_ms": 1800.0,
        "default_pitch_hz": 261.63,
        "harmonics": [
            (1, 1.00), (2, 0.50), (3, 0.30), (4, 0.15),
            (5, 0.08), (6, 0.04), (7, 0.02), (8, 0.01),
        ],
        "harmonic_decay": [1.5, 2.5, 3.5, 5.0, 7.0, 10.0, 14.0, 18.0],
        "hammer_noise_amp": 0.08,
        "hammer_noise_duration_s": 0.002,
        "body_resonance_hz": 60.0,
        "body_resonance_amp": 0.10,
        "brightness": 0.5,
        "stereo_width": 0.0,
        "lo_fi": 0.0,
        "compression": 0.0,
    },
    "soft": {
        "label": "Soft Piano",
        "default_duration_ms": 1500.0,
        "default_pitch_hz": 261.63,
        "harmonics": [
            (1, 1.00), (2, 0.55), (3, 0.35), (4, 0.20),
            (5, 0.10), (6, 0.05), (7, 0.03), (8, 0.01),
        ],
        "harmonic_decay": [4.0, 5.5, 7.0, 9.0, 12.0, 16.0, 20.0, 25.0],
        "hammer_noise_amp": 0.05,
        "hammer_noise_duration_s": 0.002,
        "body_resonance_hz": 80.0,
        "body_resonance_amp": 0.05,
        "brightness": 0.7,
        "stereo_width": 0.0,
        "lo_fi": 0.0,
        "compression": 0.2,
    },
    "lo_fi": {
        "label": "Lo-Fi Piano",
        "default_duration_ms": 1200.0,
        "default_pitch_hz": 261.63,
        "harmonics": [
            (1, 1.00), (2, 0.50), (3, 0.35), (4, 0.25),
            (5, 0.15), (6, 0.10), (7, 0.06), (8, 0.04),
        ],
        "harmonic_decay": [3.0, 4.5, 6.0, 7.5, 10.0, 13.0, 17.0, 22.0],
        "hammer_noise_amp": 0.15,
        "hammer_noise_duration_s": 0.003,
        "body_resonance_hz": 70.0,
        "body_resonance_amp": 0.06,
        "brightness": 0.8,
        "stereo_width": 0.0,
        "lo_fi": 0.4,
        "compression": 0.3,
    },
    "compressed": {
        "label": "Compressed Piano",
        "default_duration_ms": 1000.0,
        "default_pitch_hz": 261.63,
        "harmonics": [
            (1, 1.00), (2, 0.60), (3, 0.45), (4, 0.30),
            (5, 0.18), (6, 0.10), (7, 0.06), (8, 0.03),
        ],
        "harmonic_decay": [2.5, 3.5, 4.5, 6.0, 8.0, 10.0, 14.0, 18.0],
        "hammer_noise_amp": 0.15,
        "hammer_noise_duration_s": 0.004,
        "body_resonance_hz": 80.0,
        "body_resonance_amp": 0.08,
        "brightness": 1.2,
        "stereo_width": 0.0,
        "lo_fi": 0.0,
        "compression": 0.8,
    },
    "wide": {
        "label": "Wide Piano",
        "default_duration_ms": 1500.0,
        "default_pitch_hz": 261.63,
        "harmonics": [
            (1, 1.00), (2, 0.60), (3, 0.45), (4, 0.30),
            (5, 0.18), (6, 0.10), (7, 0.06), (8, 0.03),
        ],
        "harmonic_decay": [3.0, 4.0, 5.5, 7.0, 9.0, 12.0, 16.0, 20.0],
        "hammer_noise_amp": 0.12,
        "hammer_noise_duration_s": 0.003,
        "body_resonance_hz": 80.0,
        "body_resonance_amp": 0.06,
        "brightness": 1.0,
        "stereo_width": 0.6,
        "lo_fi": 0.0,
        "compression": 0.0,
    },
    "felt": {
        "label": "Felt Piano",
        "default_duration_ms": 1400.0,
        "default_pitch_hz": 261.63,
        "harmonics": [
            (1, 1.00), (2, 0.45), (3, 0.25), (4, 0.12),
            (5, 0.06), (6, 0.03), (7, 0.01), (8, 0.005),
        ],
        "harmonic_decay": [2.0, 3.0, 4.0, 5.5, 8.0, 11.0, 15.0, 20.0],
        "hammer_noise_amp": 0.03,
        "hammer_noise_duration_s": 0.002,
        "body_resonance_hz": 50.0,
        "body_resonance_amp": 0.08,
        "brightness": 0.4,
        "stereo_width": 0.0,
        "lo_fi": 0.0,
        "compression": 0.3,
    },
    # ── Electric Piano / Rhodes / Bell ──
    "rhodes": {
        "label": "Rhodes EP",
        "default_duration_ms": 2000.0,
        "default_pitch_hz": 261.63,
        "harmonics": [
            (1, 1.00), (2, 0.80), (3, 0.55), (4, 0.65),
            (5, 0.25), (6, 0.40), (7, 0.10), (8, 0.15),
        ],
        "harmonic_decay": [5.0, 4.0, 8.0, 5.0, 12.0, 7.0, 18.0, 10.0],
        "hammer_noise_amp": 0.06,
        "hammer_noise_duration_s": 0.002,
        "body_resonance_hz": 150.0,
        "body_resonance_amp": 0.04,
        "brightness": 0.9,
        "stereo_width": 0.3,
        "lo_fi": 0.0,
        "compression": 0.1,
    },
    "wurly": {
        "label": "Wurlitzer EP",
        "default_duration_ms": 1500.0,
        "default_pitch_hz": 261.63,
        "harmonics": [
            (1, 1.00), (2, 0.90), (3, 0.70), (4, 0.50),
            (5, 0.30), (6, 0.40), (7, 0.15), (8, 0.20),
        ],
        "harmonic_decay": [4.0, 3.5, 5.0, 6.0, 9.0, 8.0, 14.0, 12.0],
        "hammer_noise_amp": 0.10,
        "hammer_noise_duration_s": 0.003,
        "body_resonance_hz": 100.0,
        "body_resonance_amp": 0.05,
        "brightness": 1.1,
        "stereo_width": 0.2,
        "lo_fi": 0.1,
        "compression": 0.2,
    },
    "bell": {
        "label": "Bell / Vibraphone",
        "default_duration_ms": 2500.0,
        "default_pitch_hz": 440.0,
        "harmonics": [
            (1, 1.00), (2, 0.30), (3, 0.75), (4, 0.25),
            (5, 0.50), (6, 0.15), (7, 0.35), (8, 0.10),
        ],
        "harmonic_decay": [6.0, 4.0, 6.5, 4.5, 7.0, 5.0, 8.0, 5.5],
        "hammer_noise_amp": 0.03,
        "hammer_noise_duration_s": 0.001,
        "body_resonance_hz": 200.0,
        "body_resonance_amp": 0.02,
        "brightness": 1.3,
        "stereo_width": 0.0,
        "lo_fi": 0.0,
        "compression": 0.0,
    },
}


def synthesize_piano_stab(duration_ms: float = 1500.0, pitch_hz: float = 261.63,
                           profile_name: str = "acoustic", profiles: dict = None,
                           overrides: dict = None) -> np.ndarray:
    """Generate a piano/keys one-shot stab from the named profile with adjective overrides.
    
    Models piano as: hammer transient + resonant harmonic body + long decay.
    Velocity behavior: soft = rounder/slower/darker, hard = brighter/faster/louder.
    Pitch-locked harmonic stack (no random pitch shift).
    """
    pf = dict(PIANO_PROFILES.get(profile_name, PIANO_PROFILES["acoustic"]))
    if overrides:
        pf.update(overrides)

    num_samples = max(100, int(SAMPLE_RATE * duration_ms / 1000.0))
    samples = np.zeros(num_samples)

    # ── Velocity (0-1): maps adjectives into piano behavior ──
    velocity = pf.get("velocity", 0.6)
    # Extract control parameters from profile + overrides
    brightness = pf.get("brightness", 1.0)
    attack_ms_override = pf.get("attack_ms", None)
    hammer_noise_amp = pf.get("hammer_noise_amp", 0.12)
    sustain_level = pf.get("sustain_level", 0.15)
    release_ms = pf.get("release_ms", 300.0)

    # Velocity → attack time: soft (0) = 35ms, hard (1) = 2ms
    if attack_ms_override is not None:
        attack_s = max(1, int(attack_ms_override * SAMPLE_RATE / 1000.0))
    else:
        attack_time_ms = 35.0 - velocity * 33.0  # 35ms at soft, 2ms at hard
        attack_time_ms = max(1.0, min(attack_time_ms, 50.0))
        attack_s = max(1, int(attack_time_ms * SAMPLE_RATE / 1000.0))

    # Velocity → hammer noise: soft = quiet, hard = loud
    if "hammer_noise_amp" not in pf or pf["hammer_noise_amp"] == pf.get("hammer_noise_amp", 0.12):
        hammer_noise_amp = 0.02 + velocity * 0.23  # 0.02 at soft, 0.25 at hard

    # Velocity → brightness boost
    vel_brightness = 0.5 + velocity * 1.0  # 0.5 at soft, 1.5 at hard
    effective_brightness = brightness * vel_brightness

    # ── ADSR envelope ──
    decay_s = max(int(0.150 * SAMPLE_RATE), 1)
    release_s = max(int(num_samples - release_ms * SAMPLE_RATE / 1000.0), attack_s + decay_s + 1)
    adsr = np.ones(num_samples)
    for i in range(num_samples):
        if i < attack_s:
            p = i / max(attack_s, 1)
            adsr[i] = p * (2.0 - p)
        elif i < decay_s:
            t = (i - attack_s) / max(decay_s - attack_s, 1)
            adsr[i] = 1.0 - (1.0 - sustain_level) * (t * t)
        elif i < release_s:
            adsr[i] = sustain_level
        else:
            t = (i - release_s) / max(num_samples - release_s, 1)
            adsr[i] = sustain_level * (1.0 - t) ** 2

    # ── Pitch-locked harmonic oscillator stack ──
    harmonics = pf["harmonics"]
    harm_decay = pf["harmonic_decay"]

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for h_idx, (h, amp) in enumerate(harmonics):
            freq = pitch_hz * h  # pitch-locked: no random detune
            d_idx = min(h_idx, len(harm_decay) - 1)
            decay_rate = harm_decay[d_idx]
            h_env = math.exp(-decay_rate * t)
            bright_boost = 1.0 + (effective_brightness - 1.0) * h_idx * 0.25
            bright_boost = max(0.15, min(bright_boost, 4.0))
            val += math.sin(2 * math.pi * freq * t) * amp * h_env * bright_boost

        body_freq = pf["body_resonance_hz"]
        body_amp = pf["body_resonance_amp"]
        val += math.sin(2 * math.pi * body_freq * t) * math.exp(-2.5 * t) * body_amp

        samples[i] = val * adsr[i]

    # Normalize harmonic body (before transient addition)
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.5

    # ── Hammer transient (noise + click burst) ──
    noise_dur = pf["hammer_noise_duration_s"]
    noise_len = min(int(noise_dur * SAMPLE_RATE), num_samples)
    hammer_noise = np.random.randn(noise_len + 100).astype(np.float32) * hammer_noise_amp
    for i in range(noise_len):
        t = i / SAMPLE_RATE
        env = math.exp(-600.0 * t)
        samples[i] += hammer_noise[i] * env

    # ── Post-processing: EQ shelves ──
    if pf.get("high_shelf_db", 0) != 0:
        samples = biquad_high_shelf(samples, 3000.0, pf["high_shelf_db"], 0.6)
    if pf.get("low_shelf_db", 0) != 0:
        samples = biquad_low_shelf(samples, 250.0, pf["low_shelf_db"], 0.7)

    # ── Saturation / Distortion ──
    saturation = pf.get("saturation", 0.0)
    drive = pf.get("drive", 0.0)
    total_drive = max(saturation, drive)
    if total_drive > 0.01:
        drive_amount = 1.0 + total_drive * 4.0
        for i in range(len(samples)):
            samples[i] = tape_saturation(samples[i], drive_amount)

    # ── Lo-fi processing ──
    lo_fi = pf.get("lo_fi", 0.0)
    if lo_fi > 0.01:
        if lo_fi > 0.1:
            bits = max(4, int(16 - lo_fi * 12))
            levels = 2 ** bits
            samples = np.round(samples * levels) / levels
        noise_floor_amp = pf.get("noise_floor", 0.0)
        if noise_floor_amp > 0 or lo_fi > 0:
            nf = 0.005 * lo_fi + noise_floor_amp * 0.01
            samples += np.random.randn(num_samples).astype(np.float32) * nf

    # ── Bandwidth reduction (lo-fi / dark) ──
    bw_reduce = pf.get("bandwidth_reduce", 0.0)
    if bw_reduce > 0.01:
        cutoff = 8000.0 * (1.0 - bw_reduce * 0.85)
        cutoff = max(cutoff, 800.0)
        sos = sp_signal.butter(4, cutoff, 'lowpass', fs=SAMPLE_RATE, output='sos')
        samples = sp_signal.sosfilt(sos, samples)

    # ── Compression ──
    compression = pf.get("compression", 0.0)
    if compression > 0.01:
        window = int(0.010 * SAMPLE_RATE)
        rms_env = np.convolve(samples ** 2, np.ones(window) / window, mode='same')
        rms_env = np.sqrt(np.maximum(rms_env, 1e-10))
        threshold = 0.1
        ratio = 1.0 + compression * 3.0
        gain = np.ones_like(samples)
        mask = rms_env > threshold
        gain[mask] = (threshold + (rms_env[mask] - threshold) / ratio) / np.maximum(rms_env[mask], 1e-10)
        samples = samples * gain

    # ── Stereo width ──
    stereo = pf.get("stereo_width", 0.0)
    if stereo > 0.01:
        stereo_samples = np.zeros((num_samples, 2), dtype=np.float32)
        stereo_samples[:, 0] = samples * (1.0 - stereo * 0.25)
        detune_ratio = 1.0 + stereo * 0.004
        delay = int(0.0015 * stereo * SAMPLE_RATE)
        for i in range(num_samples):
            src_idx = max(0, i - delay)
            stereo_samples[i, 1] = samples[src_idx] * (1.0 - stereo * 0.25) * detune_ratio
        samples = stereo_samples.reshape(-1)

    # ── Final normalize ──
    if samples.ndim == 1:
        peak = np.max(np.abs(samples))
        if peak > 0:
            samples = samples / peak * 0.9
    else:
        for ch in range(samples.shape[1]):
            peak = np.max(np.abs(samples[:, ch]))
            if peak > 0:
                samples[:, ch] = samples[:, ch] / peak * 0.9

    return samples.astype(np.float32)


# ─── CLI command ────────────────────────────────────────


PIANO_CLASSES = {
    name: (info["label"], info["default_duration_ms"], info["default_pitch_hz"])
    for name, info in PIANO_PROFILES.items()
}


def generate_piano_batch(profile_name: str, count: int, out_dir: Path, profiles: dict = None):
    """Batch generate piano/keys samples."""
    pf = PIANO_PROFILES[profile_name]
    label = pf["label"]
    default_dur = pf["default_duration_ms"]
    default_pitch = pf["default_pitch_hz"]
    seed_offset = int(time.time() * 1000) % 1000000

    out_dir.mkdir(parents=True, exist_ok=True)
    print(f"Generating {count} {label} → {out_dir}...")

    for i in range(count):
        seed = (seed_offset + i) * 314159265 + hash(profile_name) % 1000000
        random.seed(seed)
        np.random.seed(seed % 2**32)

        dur_var = default_dur * (1.0 + (random.random() - 0.5) * 0.3)
        pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * 0.15)

        samples = synthesize_piano_stab(dur_var, pitch_var, profile_name, profiles)

        out_path = out_dir / f"piano_{profile_name}_{i+1:03d}.wav"
        write_wav(out_path, samples)

        if i == 0 or (i + 1) % 5 == 0:
            print(f"  [{i+1}/{count}] {out_path.name}")

    print(f"  Done: {count} {label} → {out_dir}")


def cmd_piano_gen(args):
    """Generate piano/keys samples from named profiles."""
    profile_name = args.profile
    if profile_name not in PIANO_PROFILES and profile_name != "all":
        print(f"Unknown profile: {profile_name}")
        print(f"Valid profiles: {', '.join(sorted(PIANO_PROFILES.keys()))}, all")
        sys.exit(1)

    count = args.count
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    profiles_to_generate = list(PIANO_PROFILES.keys()) if profile_name == "all" else [profile_name]

    for pn in profiles_to_generate:
        generate_piano_batch(pn, count, out_dir / pn if profile_name == "all" else out_dir)

    if profile_name == "all":
        print(f"\nAll piano profiles generated in {out_dir}")


def cmd_piano_audit(args):
    """Piano audit report: compare generated piano samples to reference clusters."""
    from gen.tonal_qa import cmd_tonal_qa as run_tonal_qa

    print("=" * 60)
    print("  PIANO AUDIT REPORT")
    print("=" * 60)
    print()

    in_dir = Path(args.input_dir)

    # Use the tonal QA system to evaluate against synth_stab reference
    audit_results = {"by_profile": {}}

    # Find all subdirectories (each profile = one variation)
    profile_dirs = [d for d in in_dir.iterdir() if d.is_dir()]

    if not profile_dirs:
        profile_dirs = [in_dir]
        # Try to detect profile from listening_notes.json or just run plain QA
        print(f"  Analyzing: {in_dir}")
    else:
        print(f"  Found {len(profile_dirs)} profile directories")

    for pdir in sorted(profile_dirs):
        name = pdir.name
        wavs = list(pdir.glob("*.wav"))
        if not wavs:
            continue
        print(f"\n  Profile: {name} ({len(wavs)} files)")
        # Run pitch analysis
        from gen.pitch import cmd_detect_pitch
        # Simple inline analysis
        pitches = []
        centroids = []
        decays = []
        for wav_path in wavs:
            from gen.io import read_wav
            s, sr_ = read_wav(wav_path)
            if s is None:
                continue
            from gen.features import detect_pitch_full, compute_features
            pitch_info = detect_pitch_full(s, sr_)
            feats = compute_features(s, sr_)
            pitches.append(pitch_info["pitch_hz"])
            centroids.append(feats.get("spectral_centroid", 0))
            decays.append(feats.get("decay_length_ms", 0))

        if pitches:
            import numpy as np
            avg_pitch = np.mean(pitches)
            avg_cent = np.mean(centroids)
            avg_decay = np.mean(decays)
            from gen.features import hz_to_note
            print(f"    Avg pitch: {avg_pitch:.1f}Hz ({hz_to_note(avg_pitch)})")
            print(f"    Avg centroid: {avg_cent:.0f}Hz")
            print(f"    Avg decay: {avg_decay:.1f}ms")

            # Piano-likeness check
            piano_scores = []
            for wav_path in wavs:
                s, sr_ = read_wav(wav_path)
                if s is None:
                    continue
                from gen.piano import analyze_piano_full
                pa = analyze_piano_full(s, sr_)
                piano_scores.append(pa["piano_likeness_score"])
            avg_score = np.mean(piano_scores)
            print(f"    Avg piano score: {avg_score:.2f}")
            print(f"    Piano-like: {sum(1 for s in piano_scores if s >= 0.45)}/{len(piano_scores)}")

            audit_results["by_profile"][name] = {
                "files": len(wavs),
                "avg_pitch_hz": round(float(avg_pitch), 1),
                "avg_centroid_hz": round(float(avg_cent), 1),
                "avg_decay_ms": round(float(avg_decay), 1),
                "avg_piano_score": round(float(avg_score), 3),
            }

    print(f"\n  {'='*55}")
    print(f"  AUDIT SUMMARY")
    print(f"  {'='*55}")
    for name, res in audit_results["by_profile"].items():
        print(f"  {name:15s}: {res['files']} files, pitch={res['avg_pitch_hz']}Hz, "
              f"centroid={res['avg_centroid_hz']}Hz, piano_score={res['avg_piano_score']:.2f}")

    if args.output:
        import json, time
        out_path = Path(args.output)
        audit_results["generated_at"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        out_path.write_text(json.dumps(audit_results, indent=2))
        print(f"\n  Report: {out_path}")
