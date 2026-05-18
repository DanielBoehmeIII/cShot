"""
Phase 4 — Drum Domination (Weeks 13-18)
Enhanced pack-aware drum synthesis: kick, snare, clap, hat, 808, percussion.
"""

import math
import random
import numpy as np
from scipy import signal as sp_signal
from gen import SAMPLE_RATE
from gen.dsp import noise_like, tape_saturation, biquad_low_shelf, biquad_high_shelf


def _apply_style_profile(params: dict, style_profile: dict = None) -> dict:
    if not style_profile:
        return params
    p = dict(params)
    sp = style_profile

    if "punch" in sp and sp["punch"] > 0:
        p["attack_strength"] = 0.5 + sp["punch"] * 0.5
    if "brightness" in sp and sp["brightness"] > 0:
        p["brightness"] = 0.2 + sp["brightness"] * 0.8
    if "saturation" in sp and sp["saturation"] > 0:
        p["drive"] = 0.3 + sp["saturation"] * 0.7
    if "warmth" in sp and sp["warmth"] > 0:
        p["low_boost"] = 0.5 + sp["warmth"] * 0.5
    if "tonality" in sp and sp["tonality"] > 0:
        p["tonal_blend"] = 0.2 + sp["tonality"] * 0.6
    if "air" in sp and sp["air"] > 0:
        p["air"] = sp["air"]
    if "width" in sp and sp["width"] > 0:
        p["width"] = sp["width"]
    if "aggression" in sp and sp["aggression"] > 0:
        p["aggression"] = 0.3 + sp["aggression"] * 0.7
    if "dynamics" in sp and sp["dynamics"] > 0:
        p["dynamics"] = 0.3 + sp["dynamics"] * 0.7

    return p


def _compute_adsr_env(num_samples: int, attack_ms: float = 5.0,
                       decay_ms: float = 100.0, sustain_level: float = 0.3,
                       release_ms: float = 50.0) -> np.ndarray:
    env = np.ones(num_samples)
    a_len = min(int(attack_ms * SAMPLE_RATE / 1000.0), num_samples)
    d_len = min(int(decay_ms * SAMPLE_RATE / 1000.0), num_samples - a_len)
    r_len = min(int(release_ms * SAMPLE_RATE / 1000.0), num_samples)
    s_start = a_len + d_len
    r_start = num_samples - r_len

    if a_len > 0:
        env[:a_len] = np.linspace(0, 1, a_len)
    if d_len > 0:
        env[a_len:s_start] = np.linspace(1, sustain_level, d_len)
    if r_len > 0:
        env[r_start:] = np.linspace(env[r_start - 1] if r_start > 0 else sustain_level, 0, r_len)

    return env


# ─── Week 13: 808 Recreation ────────────────────────────

def synthesize_808_enhanced(
    duration_ms: float = 800.0,
    pitch_hz: float = 55.0,
    glide: float = 0.0,
    distortion: float = 0.5,
    saturation: float = 0.6,
    sub_body_balance: float = 0.7,
    texture_amount: float = 0.1,
    style_profile: dict = None,
) -> np.ndarray:
    params = _apply_style_profile({
        "attack_strength": 1.0,
        "drive": distortion,
        "low_boost": 0.7,
        "tonal_blend": 0.8,
        "air": 0.0,
        "aggression": 0.5,
    }, style_profile)

    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    glide_offset = glide * 30.0

    click_samples = min(int(0.008 * SAMPLE_RATE), num_samples)
    for i in range(click_samples):
        t = i / SAMPLE_RATE
        click_env = math.exp(-200.0 * t)
        click = math.sin(2 * math.pi * 3000.0 * t) * click_env * 0.15 * params["attack_strength"]
        click += noise_like(i * 0.5) * click_env * 0.08 * params["attack_strength"]
        samples[i] += click

    body_gain = params.get("tonal_blend", 0.8)
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        frac = i / num_samples

        freq = pitch_hz + glide_offset - (pitch_hz * 0.4 + glide_offset * 0.5) * frac
        freq = max(freq, 22.0)

        sub = math.sin(2 * math.pi * freq * t) * 0.5
        sub += math.sin(2 * math.pi * freq * 0.5 * t) * 0.12
        sub += math.sin(2 * math.pi * freq * 2.0 * t) * 0.10
        sub += math.sin(2 * math.pi * freq * 3.0 * t) * 0.05

        env = math.exp(-2.5 * t * (1.0 + (1.0 - saturation) * 0.5))
        sub *= env

        noise_amt = texture_amount * 0.05
        if noise_amt > 0:
            sub += noise_like(i * 0.2) * env * noise_amt

        sub = tape_saturation(sub, 1.0 + params["drive"])
        sub = biquad_low_shelf(sub, 60.0, 4.0, params.get("low_boost", 0.7))

        samples[i] += sub * body_gain * 0.7

    samples = biquad_low_shelf(samples, 80.0, 4.0, 0.7)
    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9

    return samples.astype(np.float32)


# ─── Week 14: Kick Recreation ─────────────────────────

def synthesize_kick_enhanced(
    duration_ms: float = 280.0,
    pitch_hz: float = 100.0,
    attack_punch: float = 0.7,
    sub_depth: float = 0.8,
    click_amount: float = 0.5,
    clip_amount: float = 0.0,
    style_profile: dict = None,
) -> np.ndarray:
    params = _apply_style_profile({
        "attack_strength": attack_punch,
        "drive": clip_amount,
        "low_boost": sub_depth,
        "brightness": 0.6,
    }, style_profile)

    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)
    click_samples = min(int(0.01 * SAMPLE_RATE), num_samples)

    for i in range(click_samples):
        t = i / SAMPLE_RATE
        click_env = math.exp(-160.0 * t)
        click_noise = noise_like(i * 0.3) * click_env * 0.12 * params["attack_strength"]
        click_tone = math.sin(2 * math.pi * (5000.0 - 3000.0 * t / 0.01) * t) * click_env * 0.2 * params["attack_strength"]
        samples[i] += click_noise + click_tone

    impact_samples = min(int(0.03 * SAMPLE_RATE), num_samples)
    for i in range(impact_samples):
        t = i / SAMPLE_RATE
        impact_freq = 2000.0 - 1200.0 * (t / 0.02)
        impact_env = math.exp(-90.0 * t)
        impact = math.sin(2 * math.pi * impact_freq * t) * impact_env * 0.25 * params["attack_strength"]
        samples[i] += impact

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        body_freq = pitch_hz * 2.0 - pitch_hz * 1.5 * (t / min(duration_ms / 1000.0, 0.3))
        body_freq = max(body_freq, pitch_hz * 0.5)

        body_env = math.exp(-6.0 * t * (1.0 + (1.0 - sub_depth) * 0.5))
        body = math.sin(2 * math.pi * body_freq * t) * body_env * 0.45
        body += math.sin(2 * math.pi * body_freq * 1.5 * t) * body_env * 0.15
        body += math.sin(2 * math.pi * body_freq * 2.0 * t) * body_env * 0.08
        samples[i] += body * 0.7

    sub_gain = params.get("low_boost", 0.8)
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        sub_freq = pitch_hz * 0.5 - 5.0 * (t / 0.3)
        sub_freq = max(sub_freq, 28.0)
        sub_env = math.exp(-3.0 * t)
        sub = math.sin(2 * math.pi * sub_freq * t) * sub_env * 0.35 * sub_gain
        samples[i] += sub

    if params.get("drive", 0) > 0.3:
        for i in range(len(samples)):
            samples[i] = tape_saturation(samples[i], 1.0 + params["drive"])

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    return samples.astype(np.float32)


# ─── Week 15: Snare Recreation ────────────────────────

def synthesize_snare_enhanced(
    duration_ms: float = 320.0,
    pitch_hz: float = 220.0,
    body_amount: float = 0.5,
    crack_amount: float = 0.7,
    noise_amount: float = 0.8,
    tail_length: float = 0.5,
    style_profile: dict = None,
) -> np.ndarray:
    params = _apply_style_profile({
        "tonal_blend": body_amount,
        "attack_strength": crack_amount,
        "aggression": noise_amount,
        "brightness": 0.6,
    }, style_profile)

    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    body_gain = params.get("tonal_blend", 0.5)
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        frac = i / num_samples
        tone_freq = pitch_hz - 20.0 * frac
        tone_freq = max(tone_freq, 80.0)
        tone_env = math.exp(-8.0 * t * (1.0 + (1.0 - body_amount) * 0.5))
        tone = math.sin(2 * math.pi * tone_freq * t) * tone_env * 0.3 * body_gain
        tone += math.sin(2 * math.pi * tone_freq * 2.0 * t) * tone_env * 0.08 * body_gain
        samples[i] += tone

    noise_amp = min(1.0, params.get("aggression", 0.7) * 0.4 + 0.4)
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        n1 = noise_like(i * 0.02)
        n2 = noise_like(i * 0.06)
        n3 = noise_like(i * 0.12)
        noise_body = n1 * 0.3 + n2 * 0.35 + n3 * 0.35
        tail_factor = tail_length * 0.5 + 1.0
        noise_env = math.exp(-10.0 * t / tail_factor)
        samples[i] += noise_body * noise_env * 0.35 * noise_amp

    crack_amp = params.get("attack_strength", 0.7)
    crack_len = min(int(0.025 * SAMPLE_RATE), num_samples)
    sos_hp = sp_signal.butter(4, 3000, 'highpass', fs=SAMPLE_RATE, output='sos')
    crack_noise = np.random.randn(crack_len + 64) * 0.5
    crack_noise = sp_signal.sosfilt(sos_hp, crack_noise)
    for i in range(min(crack_len, num_samples)):
        t = i / SAMPLE_RATE
        crack_env = math.exp(-140.0 * t)
        samples[i] += crack_noise[i] * crack_env * 0.25 * crack_amp

    sos_lp = sp_signal.butter(3, 5500, 'lowpass', fs=SAMPLE_RATE, output='sos')
    samples = sp_signal.sosfilt(sos_lp, samples)

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


# ─── Week 16: Clap Recreation ─────────────────────────

def synthesize_clap_enhanced(
    duration_ms: float = 380.0,
    pitch_hz: float = 180.0,
    burst_timing: float = 0.5,
    room_size: float = 0.3,
    body_noise: float = 0.6,
    style_profile: dict = None,
) -> np.ndarray:
    params = _apply_style_profile({
        "aggression": body_noise,
        "air": 0.3,
        "dynamics": 0.3,
    }, style_profile)

    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)

    timing_spread = 0.008 + burst_timing * 0.015
    base_times = [0.0, 0.010, 0.024, 0.042, 0.065, 0.095]
    hit_times = []
    for bt in base_times:
        jitter = random.uniform(-timing_spread * 0.3, timing_spread * 0.3)
        hit_times.append(max(0, bt + jitter))

    burst_amps = [0.15, 0.40, 0.35, 0.25, 0.18, 0.10]
    burst_amps = [a * (0.5 + body_noise * 0.5) for a in burst_amps]

    noise = np.random.randn(num_samples + 4096).astype(np.float32)
    sos_hp = sp_signal.butter(4, 400, 'highpass', fs=SAMPLE_RATE, output='sos')
    noise_hp = sp_signal.sosfilt(sos_hp, noise)
    sos_body = sp_signal.butter(4, [600, 4000], 'bandpass', fs=SAMPLE_RATE, output='sos')
    noise_bp = sp_signal.sosfilt(sos_body, noise)
    sos_dark = sp_signal.butter(4, 1800, 'lowpass', fs=SAMPLE_RATE, output='sos')
    noise_dark = sp_signal.sosfilt(sos_dark, noise)
    sos_air = sp_signal.butter(4, 6000, 'highpass', fs=SAMPLE_RATE, output='sos')
    noise_air = sp_signal.sosfilt(sos_air, noise)

    air_amount = params.get("air", 0.3)
    samples = np.zeros(num_samples)
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for j, (hit_t, amp) in enumerate(zip(hit_times, burst_amps)):
            dt = t - hit_t
            if dt < 0:
                continue
            bright = max(0.0, 1.0 - j * 0.12)
            hit_env = math.exp(-70.0 * dt)
            b = noise_bp[i] * bright + noise_dark[i] * (1.0 - bright * 0.5)
            b += noise_air[i] * air_amount * 0.15
            val += b * hit_env * amp
        samples[i] = val * 0.35

    for i in range(len(samples)):
        samples[i] = tape_saturation(samples[i], 1.1 + body_noise * 0.3)

    if room_size > 0.1:
        delay_len = int(room_size * 0.05 * SAMPLE_RATE)
        decay = 0.3 * room_size
        for i in range(delay_len, num_samples):
            samples[i] += samples[i - delay_len] * decay

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


# ─── Week 17: Hat Recreation ──────────────────────────

def synthesize_hat_enhanced(
    duration_ms: float = 100.0,
    pitch_hz: float = 1000.0,
    metallic: float = 0.7,
    decay_shimmer: float = 0.5,
    noise_color: str = "white",
    style_profile: dict = None,
) -> np.ndarray:
    params = _apply_style_profile({
        "brightness": metallic,
        "air": decay_shimmer if duration_ms < 200 else 0.3,
        "tonal_blend": metallic,
    }, style_profile)

    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    metallic_partials = [
        (1.0, 0.35), (2.68, 0.5), (4.0, 0.65), (5.33, 0.55),
        (7.6, 0.35), (10.8, 0.2), (15.0, 0.12),
        (1.5, 0.15), (3.2, 0.25), (6.0, 0.2),
    ]

    meta_amp = params.get("tonal_blend", 0.7)
    shimmer = decay_shimmer * 2.0 + 1.0
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for ratio, amp in metallic_partials:
            freq = pitch_hz * ratio
            decay_rate = 50.0 * (1.0 + ratio * 0.3) / shimmer
            decay = math.exp(-decay_rate * t)
            val += math.sin(2 * math.pi * freq * t) * amp * decay
        if t < 0.003:
            noise_tick = np.random.randn() * math.exp(-500.0 * t) * 0.2
            val += noise_tick
        samples[i] = val * meta_amp

    noise_air = params.get("air", 0.3)
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        n1 = noise_like(i * 0.03)
        n2 = noise_like(i * 0.07)
        n3 = noise_like(i * 0.15)
        ns = n1 * 0.3 + n2 * 0.4 + n3 * 0.3
        if noise_color == "blue":
            ns = ns * 0.3 + noise_like(i * 0.3) * 0.7
        elif noise_color == "brownish":
            ns = noise_like(i * 0.01) * 0.5 + n1 * 0.3 + n2 * 0.2
        ns_env = math.exp(-80.0 * t / shimmer) * noise_air
        samples[i] += ns * ns_env * 0.15

    sos_hp = sp_signal.butter(4, 2000 if duration_ms < 200 else 1500,
                              'highpass', fs=SAMPLE_RATE, output='sos')
    samples = sp_signal.sosfilt(sos_hp, samples)

    if duration_ms < 150:
        gate_samples = min(int(duration_ms * 0.55 * SAMPLE_RATE / 1000.0), num_samples)
        for i in range(gate_samples, num_samples):
            samples[i] *= 0.0

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.75
    return samples.astype(np.float32)


def synthesize_open_hat_enhanced(
    duration_ms: float = 650.0,
    pitch_hz: float = 350.0,
    metallic: float = 0.6,
    decay_shimmer: float = 0.7,
    style_profile: dict = None,
) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    shimmer_rate = 2.5 + decay_shimmer * 2.0
    partials = [(1.0, 0.2, 4.0), (2.5, 0.15, 5.0), (4.0, 0.12, 6.0),
                (6.0, 0.08, 8.0), (8.5, 0.05, 10.0)]

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        tone_val = 0.0
        for ratio, amp, decay_rate in partials:
            freq = pitch_hz * ratio
            decay = math.exp(-decay_rate * t / shimmer_rate)
            tone_val += math.sin(2 * math.pi * freq * t) * amp * decay
        n1 = noise_like(i * 0.02)
        n2 = noise_like(i * 0.06)
        n3 = noise_like(i * 0.12)
        noise_val = n1 * 0.2 + n2 * 0.35 + n3 * 0.45
        env = math.exp(-shimmer_rate * 0.3 * t)
        click_env = math.exp(-50.0 * t)
        val = (tone_val * 0.15 * metallic + noise_val * 0.85) * env + noise_val * click_env * 0.03
        samples[i] = val

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.8
    return samples.astype(np.float32)


# ─── Week 18: Percussion Recreation ───────────────────

def synthesize_percussion(
    duration_ms: float = 200.0,
    pitch_hz: float = 300.0,
    perc_type: str = "rim",
    brightness: float = 0.5,
    resonance: float = 0.5,
    style_profile: dict = None,
) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    if perc_type in ("rim", "click", "rimshot"):
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            env = math.exp(-20.0 * t)
            val = math.sin(2 * math.pi * pitch_hz * t) * env * 0.5
            val += noise_like(i * 0.5) * env * 0.3 * brightness
            if t < 0.002:
                val += noise_like(i * 0.9) * math.exp(-800.0 * t) * 0.2
            samples[i] = val

    elif perc_type in ("tom", "bongo", "congas"):
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            freq_drop = pitch_hz * (1.0 - 0.2 * t / (duration_ms / 1000.0))
            env = math.exp(-12.0 * t * (1.0 + (1.0 - resonance) * 0.5))
            val = math.sin(2 * math.pi * freq_drop * t) * env * 0.45
            val += math.sin(2 * math.pi * freq_drop * 2.0 * t) * env * 0.15
            val += noise_like(i * 0.3) * env * 0.1
            if t < 0.003:
                val += noise_like(i * 0.8) * math.exp(-600.0 * t) * 0.15
            samples[i] = val

    elif perc_type in ("shaker", "maraca"):
        sos_hp = sp_signal.butter(4, 3000, 'highpass', fs=SAMPLE_RATE, output='sos')
        noise = np.random.randn(num_samples + 64).astype(np.float32)
        noise = sp_signal.sosfilt(sos_hp, noise)
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            env = math.exp(-15.0 * t)
            samples[i] = noise[i] * env * 0.4

    elif perc_type in ("cowbell", "clave", "block"):
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            env = math.exp(-30.0 * t)
            val = math.sin(2 * math.pi * pitch_hz * t) * env * 0.5
            val += math.sin(2 * math.pi * pitch_hz * 1.5 * t) * env * 0.2
            val += math.sin(2 * math.pi * pitch_hz * 2.0 * t) * env * 0.1
            samples[i] = val

    elif perc_type in ("snap", "click"):
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            env = math.exp(-120.0 * t)
            val = noise_like(i * 0.7) * env * 0.4
            val += math.sin(2 * math.pi * pitch_hz * t) * env * 0.3
            samples[i] = val

    else:
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            env = math.exp(-15.0 * t)
            val = math.sin(2 * math.pi * pitch_hz * t) * env * 0.3
            val += noise_like(i * 0.3) * env * 0.3
            samples[i] = val

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


# ─── Registration ──────────────────────────────────────

ENHANCED_DRUM_REGISTRY = {
    "kick": synthesize_kick_enhanced,
    "snare": synthesize_snare_enhanced,
    "clap": synthesize_clap_enhanced,
    "closed_hat": synthesize_hat_enhanced,
    "open_hat": synthesize_open_hat_enhanced,
    "808": synthesize_808_enhanced,
}
