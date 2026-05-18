"""
Phase 5 — Tonal Recreation (Weeks 19-22)
Enhanced pack-aware tonal synthesis: piano, synth, guitar, bell/mallet.
"""

import math
import random
import numpy as np
from scipy import signal as sp_signal
from gen import SAMPLE_RATE
from gen.dsp import noise_like, tape_saturation, biquad_low_shelf, biquad_high_shelf


def _apply_style(style_profile: dict = None) -> dict:
    if not style_profile:
        return {}
    p = {}
    sp = style_profile
    if "brightness" in sp:
        p["brightness"] = 0.2 + sp["brightness"] * 0.8
    if "warmth" in sp:
        p["warmth"] = 0.3 + sp["warmth"] * 0.7
    if "saturation" in sp:
        p["saturation"] = 0.1 + sp["saturation"] * 0.9
    if "tonality" in sp:
        p["tonal_purity"] = 0.3 + sp["tonality"] * 0.7
    if "air" in sp:
        p["air"] = sp["air"]
    if "width" in sp:
        p["width"] = sp["width"]
    return p


# ─── Week 19: Piano Recreation ─────────────────────────

def synthesize_piano_enhanced(
    duration_ms: float = 1500.0,
    pitch_hz: float = 261.63,
    hammer_hardness: float = 0.5,
    resonance: float = 0.7,
    damping: float = 0.3,
    velocity: float = 0.7,
    style_profile: dict = None,
) -> np.ndarray:
    sp = _apply_style(style_profile)
    brightness = sp.get("brightness", 0.5 + hammer_hardness * 0.5)
    warmth = sp.get("warmth", 0.5)
    sat = sp.get("saturation", 0.1)

    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    n_partials = 12
    partial_amps = []
    for h in range(1, n_partials + 1):
        rolloff = math.exp(-0.4 * h * (1.0 + (1.0 - brightness) * 1.5))
        amp = rolloff * velocity * (0.8 + 0.2 * random.random())
        if h % 2 == 0:
            amp *= 0.7 + damping * 0.3
        partial_amps.append(amp)

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for h_idx, amp in enumerate(partial_amps):
            h = h_idx + 1
            inharmonicity = 1.0 + h ** 2 * 0.0002
            freq = pitch_hz * h * inharmonicity
            h_decay = max(0.01, math.exp(-max(0.5, (2.0 + h * 0.5 - damping * 0.5)) * t))
            val += math.sin(2 * math.pi * freq * t) * amp * h_decay
        samples[i] = val

    hammer_samples = min(int(0.006 * SAMPLE_RATE), num_samples)
    for i in range(hammer_samples):
        t = i / SAMPLE_RATE
        hammer_env = math.exp(-400.0 * t)
        hammer_noise = noise_like(i * 0.8) * hammer_env * 0.08 * hammer_hardness * velocity
        hammer_tone = math.sin(2 * math.pi * 2500.0 * t) * hammer_env * 0.06 * hammer_hardness * velocity
        samples[i] += hammer_noise + hammer_tone

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        body_res = math.sin(2 * math.pi * 120.0 * t) * math.exp(-2.0 * t) * 0.03 * resonance
        samples[i] += body_res

    if sat > 0.1:
        for i in range(len(samples)):
            al = 0.1 + sat * 0.3
            samples[i] = tape_saturation(samples[i], 1.0 + al)

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


# ─── Week 20: Synth Recreation ─────────────────────────

def synthesize_synth_enhanced(
    duration_ms: float = 600.0,
    pitch_hz: float = 220.0,
    osc_type: str = "saw",
    filter_cutoff: float = 0.6,
    filter_resonance: float = 0.3,
    detune: float = 0.3,
    attack_ms: float = 10.0,
    decay_ms: float = 200.0,
    sustain: float = 0.0,
    style_profile: dict = None,
) -> np.ndarray:
    sp = _apply_style(style_profile)
    brightness = sp.get("brightness", 0.3 + filter_cutoff * 0.7)
    width = sp.get("width", detune * 0.3)
    sat = sp.get("saturation", 0.2)

    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    if osc_type in ("saw", "lead"):
        n_osc = 5 + max(0, int(detune * 10))
        detunes = [1.0 + random.uniform(-detune * 0.02, detune * 0.02) for _ in range(n_osc)]
        amps = [1.0 / n_osc] * n_osc
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            val = 0.0
            for d, a in zip(detunes, amps):
                freq = pitch_hz * d
                phase = (freq * t) % 1.0
                saw = 2.0 * phase - 1.0
                if width > 0.1 and _idx % 2 == 0:
                    saw = 2.0 * ((freq * t * 1.003) % 1.0) - 1.0
                val += saw * a
            env = math.exp(-3.0 * t)
            samples[i] = val * env * 0.3

    elif osc_type in ("square", "pluck"):
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            val = 0.0
            for h in range(1, 9, 2):
                freq = pitch_hz * h
                h_amp = 1.0 / h
                env = math.exp(-2.5 * t * h ** 0.3)
                val += math.sin(2 * math.pi * freq * t) * h_amp * env
            samples[i] = val * 0.4

    elif osc_type in ("sine", "pad"):
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            val = math.sin(2 * math.pi * pitch_hz * t) * 0.5
            val += math.sin(2 * math.pi * pitch_hz * 1.5 * t) * 0.15
            val += math.sin(2 * math.pi * pitch_hz * 2.0 * t) * 0.08
            env = math.exp(-1.5 * t)
            samples[i] = val * env * 0.3

    elif osc_type == "bass":
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            freq_drop = pitch_hz * (1.0 - 0.3 * t / (duration_ms / 1000.0))
            val = math.sin(2 * math.pi * freq_drop * t) * 0.5
            val += math.sin(2 * math.pi * freq_drop * 2.0 * t) * 0.2
            val += math.sin(2 * math.pi * freq_drop * 3.0 * t) * 0.1
            env = math.exp(-3.0 * t)
            samples[i] = val * env * 0.5

    env_adsr = np.ones(num_samples)
    a_len = min(int(attack_ms * SAMPLE_RATE / 1000.0), num_samples)
    d_len = min(int(decay_ms * SAMPLE_RATE / 1000.0), num_samples - a_len)
    s_start = a_len + d_len
    if a_len > 0:
        env_adsr[:a_len] = np.linspace(0, 1, a_len)
    if d_len > 0:
        env_adsr[a_len:s_start] = np.linspace(1, sustain, d_len)
    env_adsr[s_start:] = sustain
    samples = samples * env_adsr

    sos_lp = sp_signal.butter(2, 200 + brightness * 8000, 'lowpass', fs=SAMPLE_RATE, output='sos')
    samples = sp_signal.sosfilt(sos_lp, samples)

    if filter_resonance > 0:
        sos_peak = sp_signal.butter(2, 500 + brightness * 4000, 'bandpass', fs=SAMPLE_RATE, output='sos')
        resonance = sp_signal.sosfilt(sos_peak, samples)
        samples = samples * (1.0 - filter_resonance * 0.5) + resonance * filter_resonance * 0.5

    if sat > 0.1:
        for i in range(len(samples)):
            samples[i] = tape_saturation(samples[i], 1.0 + sat * 0.5)

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


# ─── Week 21: Guitar Recreation ────────────────────────

def synthesize_guitar_enhanced(
    duration_ms: float = 500.0,
    pitch_hz: float = 220.0,
    pick_hardness: float = 0.5,
    body_resonance: float = 0.6,
    fret_noise: float = 0.3,
    style_profile: dict = None,
) -> np.ndarray:
    sp = _apply_style(style_profile)
    brightness = sp.get("brightness", 0.3 + pick_hardness * 0.4)
    tone = sp.get("tonal_purity", 0.7)

    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    harmonics = [(1.0, 1.0), (2.0, 0.55), (3.0, 0.4), (4.0, 0.25),
                 (5.0, 0.15), (6.0, 0.1), (7.0, 0.06), (8.0, 0.04)]

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for h, amp in harmonics:
            freq = pitch_hz * h
            h_decay_rate = 1.8 * h ** 0.6
            h_decay = max(0.01, math.exp(-h_decay_rate * t))
            inharm = 1.0 + h ** 2 * 0.0001
            val += math.sin(2 * math.pi * freq * inharm * t) * amp * h_decay
        norm = sum(amp for _, amp in harmonics)
        samples[i] = val / norm * 0.55 * tone

    pick_samples = min(int(0.008 * SAMPLE_RATE), num_samples)
    for i in range(pick_samples):
        t = i / SAMPLE_RATE
        pick_env = math.exp(-500.0 * t)
        pick_noise = noise_like(i * 0.85) * pick_env * 0.12 * pick_hardness
        samples[i] += pick_noise

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        body_env = math.exp(-3.5 * t)
        body = math.sin(2 * math.pi * 120.0 * t) * body_env * 0.06 * body_resonance
        body += math.sin(2 * math.pi * 180.0 * t) * body_env * 0.03 * body_resonance
        samples[i] += body

    if fret_noise > 0 and pick_hardness > 0.3:
        for i in range(min(pick_samples, num_samples)):
            t = i / SAMPLE_RATE
            fret = noise_like(i * 0.3) * math.exp(-300.0 * t) * 0.04 * fret_noise
            samples[i] += fret

    if brightness > 0.6:
        sos_hp = sp_signal.butter(2, 2000, 'highpass', fs=SAMPLE_RATE, output='sos')
        bright = sp_signal.sosfilt(sos_hp, samples) * (brightness - 0.5) * 0.3
        samples = samples + bright

    for i in range(len(samples)):
        samples[i] = tape_saturation(samples[i], 1.15)

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


# ─── Week 22: Bell/Mallet Recreation ───────────────────

def synthesize_bell_enhanced(
    duration_ms: float = 2000.0,
    pitch_hz: float = 440.0,
    hardness: float = 0.6,
    decay_rate: float = 0.5,
    inharmonicity: float = 0.5,
    style_profile: dict = None,
) -> np.ndarray:
    sp = _apply_style(style_profile)
    brightness = sp.get("brightness", 0.3 + hardness * 0.4)
    air_pct = sp.get("air", 0.2)

    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    partials = []
    for h in range(1, 9):
        inharm = h ** 2 * inharmonicity * 0.001
        freq = pitch_hz * (h * (1.0 + inharm))
        amp = (1.0 / h ** 0.6) * (0.5 + 0.5 * random.random())
        decay = max(1.0, (3.0 + h * 0.5) / (0.5 + decay_rate))
        partials.append((freq, amp, decay))

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for freq, amp, decay in partials:
            p_decay = math.exp(-decay * t)
            val += math.sin(2 * math.pi * freq * t) * amp * p_decay
        if hardness > 0.5 and t < 0.003:
            strike = noise_like(i * 0.9) * math.exp(-600.0 * t) * 0.1 * hardness
            val += strike
        samples[i] = val * brightness

    if air_pct > 0.1:
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            air_noise = noise_like(i * 0.1) * math.exp(-2.0 * t) * air_pct * 0.03
            samples[i] += air_noise

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.8
    return samples.astype(np.float32)


TONAL_REGISTRY = {
    "piano": synthesize_piano_enhanced,
    "synth": synthesize_synth_enhanced,
    "guitar": synthesize_guitar_enhanced,
    "bell": synthesize_bell_enhanced,
}
