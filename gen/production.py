"""
Phase 6 — Production Aesthetics (Weeks 23-26)
Saturation modeling, stereo field recreation, FX chain inference, texture layers.
"""

import math
import numpy as np
from scipy import signal as sp_signal
from gen import SAMPLE_RATE
from gen.dsp import noise_like, tape_saturation, biquad_low_shelf, biquad_high_shelf


# ─── Week 23: Saturation Modeling ──────────────────────

def apply_saturation(samples: np.ndarray, style: str = "tape", drive: float = 0.5,
                     color: str = "neutral") -> np.ndarray:
    out = samples.copy()
    if drive <= 0:
        return out

    if style == "tape":
        gain = 1.0 + drive * 2.0
        for i in range(len(out)):
            out[i] = tape_saturation(out[i], gain)
            if color == "warm":
                out[i] = tape_saturation(out[i] * 1.2, gain * 0.8)

    elif style == "clip":
        threshold = 1.0 - drive * 0.5
        for i in range(len(out)):
            if abs(out[i]) > threshold:
                out[i] = math.copysign(threshold + (abs(out[i]) - threshold) * 0.3, out[i])

    elif style == "analog":
        for i in range(len(out)):
            even = tape_saturation(out[i] * 1.5, 1.0 + drive)
            odd = out[i] * 1.2
            out[i] = even * 0.6 + odd * 0.4
            if color == "warm":
                out[i] += out[i] ** 3 * 0.05
            elif color == "bright":
                out[i] += out[i] ** 2 * 0.03 * drive

    elif style == "digital":
        bits = max(4, int(16 - drive * 12))
        for i in range(len(out)):
            out[i] = round(out[i] * (2 ** (bits - 1))) / (2 ** (bits - 1))

    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


# ─── Week 24: Stereo Field Recreation ──────────────────

def apply_stereo_width(samples: np.ndarray, width: float = 0.3,
                        imaging: str = "mid_side") -> np.ndarray:
    if samples.ndim != 2 or samples.shape[1] < 2:
        if samples.ndim == 1:
            mid = samples
            side = np.zeros_like(samples)
        else:
            return samples
    else:
        L = samples[:, 0]
        R = samples[:, 1]
        mid = (L + R) / 2.0
        side = (L - R) / 2.0

    if imaging == "mid_side":
        mid_gain = max(0.1, 1.0 - width * 0.3)
        side_gain = width
        new_l = mid * mid_gain + side * side_gain
        new_r = mid * mid_gain - side * side_gain
    elif imaging == "haas":
        delay = max(1, int(width * 0.03 * SAMPLE_RATE))
        new_l = mid + side
        new_r = np.roll(mid - side, delay)
        if delay > 0:
            new_r[:delay] = 0
    else:
        new_l = mid + side * width
        new_r = mid - side * width

    stereo_out = np.column_stack([new_l, new_r])
    peak = np.max(np.abs(stereo_out))
    if peak > 0:
        stereo_out = stereo_out / peak * 0.95
    return stereo_out


def add_stereo_from_mono(samples: np.ndarray, width: float = 0.3,
                          detune_cents: float = 2.0) -> np.ndarray:
    if samples.ndim == 2 and samples.shape[1] >= 2:
        return apply_stereo_width(samples, width)
    delay = int(SAMPLE_RATE / 1000 * 0.5)
    L = samples.copy()
    R = np.roll(samples, max(1, int(delay * width)))
    n = len(samples)
    offset = int(detune_cents * n / (1200 * SAMPLE_RATE / pitch_estimate(samples)))
    if offset > 0 and offset < n:
        R = np.roll(samples, -offset) * 0.7 + samples * 0.3
    R = np.roll(R, max(1, int(delay * width * 0.5)))
    R[:max(1, int(delay * width * 0.5))] = 0
    stereo = np.column_stack([L, R])
    peak = np.max(np.abs(stereo))
    if peak > 0:
        stereo = stereo / peak * 0.95
    return stereo


def pitch_estimate(samples: np.ndarray) -> float:
    n = min(len(samples), 4096)
    if n < 256:
        return 440.0
    spec = np.abs(np.fft.rfft(samples[:n]))
    freqs = np.fft.rfftfreq(n, 1/SAMPLE_RATE)
    lo = max(1, np.searchsorted(freqs, 50))
    hi = np.searchsorted(freqs, 2000)
    if hi - lo < 2:
        return 440.0
    peak_idx = np.argmax(spec[lo:hi]) + lo
    return float(freqs[peak_idx])


# ─── Week 25: FX Chain Inference ───────────────────────

def apply_eq(samples: np.ndarray, low_gain: float = 0.0, mid_gain: float = 0.0,
             high_gain: float = 0.0) -> np.ndarray:
    out = samples.copy()
    if abs(low_gain) > 0.01:
        out = biquad_low_shelf(out, 250.0, 3.0, 1.0 + low_gain)
    if abs(high_gain) > 0.01:
        out = biquad_high_shelf(out, 4000.0, 3.0, 1.0 + high_gain)
    if abs(mid_gain) > 0.01:
        sos_peak = sp_signal.butter(2, [400, 3000], 'bandpass', fs=SAMPLE_RATE, output='sos')
        mid = sp_signal.sosfilt(sos_peak, out)
        out = out + mid * mid_gain * 0.5
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


def apply_compression(samples: np.ndarray, threshold: float = 0.5,
                       ratio: float = 3.0, attack_ms: float = 5.0,
                       release_ms: float = 50.0) -> np.ndarray:
    if threshold >= 1.0:
        return samples
    out = samples.copy()
    attack_s = attack_ms / 1000.0
    release_s = release_ms / 1000.0
    env = 0.0
    for i in range(len(out)):
        abs_val = abs(out[i])
        env = max(abs_val, env * (1.0 - 1.0 / (attack_s * SAMPLE_RATE)))
        if env > threshold:
            gain_reduction = threshold + (env - threshold) / ratio
            out[i] *= gain_reduction / max(env, 1e-10)
        env *= (1.0 - 1.0 / (release_s * SAMPLE_RATE))
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


def apply_reverb(samples: np.ndarray, decay: float = 0.3, size: str = "medium",
                 mix: float = 0.2) -> np.ndarray:
    if mix <= 0:
        return samples
    delay_ms = {"small": 15, "medium": 30, "large": 60, "hall": 100}
    d_ms = delay_ms.get(size, 30)
    delay_samples = int(d_ms * SAMPLE_RATE / 1000.0)
    wet = np.zeros(len(samples))
    for i in range(len(samples)):
        wet[i] = samples[i]
        if i >= delay_samples:
            wet[i] += wet[i - delay_samples] * decay
            if i >= delay_samples * 2:
                wet[i] += wet[i - delay_samples * 2] * decay * 0.5
    out = samples * (1.0 - mix) + wet * mix
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


def apply_delay(samples: np.ndarray, delay_ms: float = 120.0,
                feedback: float = 0.3, mix: float = 0.2) -> np.ndarray:
    if mix <= 0:
        return samples
    delay_samples = int(delay_ms * SAMPLE_RATE / 1000.0)
    if delay_samples < 1:
        return samples
    wet = np.zeros(len(samples) + delay_samples * 4)
    for i in range(len(samples)):
        wet[i] = samples[i]
        if i >= delay_samples:
            wet[i] += wet[i - delay_samples] * feedback
            if i >= delay_samples * 2:
                wet[i] += wet[i - delay_samples * 2] * feedback ** 2
    out = np.zeros(len(samples))
    for i in range(len(samples)):
        out[i] = samples[i] * (1.0 - mix) + wet[i] * mix
    return out


def apply_chorus(samples: np.ndarray, rate: float = 0.5, depth: float = 0.003,
                 mix: float = 0.3) -> np.ndarray:
    if mix <= 0:
        return samples
    n = len(samples)
    out = np.zeros(n)
    for i in range(n):
        t = i / SAMPLE_RATE
        mod = depth * math.sin(2 * math.pi * rate * t)
        delay = int(abs(mod) * SAMPLE_RATE)
        idx = max(0, i - delay)
        chorus = samples[idx] if idx < n else 0
        out[i] = samples[i] * (1.0 - mix) + chorus * mix
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


INFERRED_FX_CHAIN = {
    "eq": apply_eq,
    "comp": apply_compression,
    "reverb": apply_reverb,
    "delay": apply_delay,
    "chorus": apply_chorus,
    "saturation": apply_saturation,
}


def apply_fx_chain(samples: np.ndarray, chain: list[dict]) -> np.ndarray:
    out = samples.copy()
    for fx in chain:
        fn_name = fx.get("type", "")
        params = {k: v for k, v in fx.items() if k != "type"}
        fn = INFERRED_FX_CHAIN.get(fn_name)
        if fn:
            out = fn(out, **params)
    return out


# ─── Week 26: Texture Layer Modeling ───────────────────

def add_vinyl_noise(samples: np.ndarray, amount: float = 0.02,
                    crackle: float = 0.3) -> np.ndarray:
    n = len(samples)
    noise = np.random.randn(n).astype(np.float32)
    sos_lp = sp_signal.butter(2, 200, 'lowpass', fs=SAMPLE_RATE, output='sos')
    noise = sp_signal.sosfilt(sos_lp, noise)
    if crackle > 0:
        for i in range(n):
            if random.random() < crackle * 0.001:
                crackle_len = min(20, n - i)
                noise[i:i + crackle_len] += np.random.randn(crackle_len) * crackle * 0.5
    out = samples + noise * amount
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


def add_tape_hiss(samples: np.ndarray, amount: float = 0.01,
                  color: str = "brown") -> np.ndarray:
    n = len(samples)
    hiss = np.random.randn(n).astype(np.float32)
    if color == "brown":
        sos_lp = sp_signal.butter(2, 400, 'lowpass', fs=SAMPLE_RATE, output='sos')
        hiss = sp_signal.sosfilt(sos_lp, hiss)
    elif color == "pink":
        sos_lp = sp_signal.butter(2, 2000, 'lowpass', fs=SAMPLE_RATE, output='sos')
        hiss = sp_signal.sosfilt(sos_lp, hiss)
    out = samples + hiss * amount
    return out


def add_air_layer(samples: np.ndarray, amount: float = 0.02,
                  freq_min: float = 8000.0) -> np.ndarray:
    n = len(samples)
    air = np.random.randn(n).astype(np.float32)
    sos_hp = sp_signal.butter(4, freq_min, 'highpass', fs=SAMPLE_RATE, output='sos')
    air = sp_signal.sosfilt(sos_hp, air)
    env = np.ones(n)
    env_decay = int(n * 0.3)
    if env_decay < n:
        env[env_decay:] = np.exp(-np.arange(n - env_decay) / (n - env_decay) * 3.0)
    out = samples + air * amount * env
    return out


def add_granular_residue(samples: np.ndarray, amount: float = 0.01,
                          grain_size_ms: float = 20.0) -> np.ndarray:
    n = len(samples)
    grain_samples = int(grain_size_ms * SAMPLE_RATE / 1000.0)
    residue = np.zeros(n)
    pos = random.randint(0, grain_samples)
    while pos < n:
        end = min(pos + grain_samples, n)
        grain = samples[pos:end].copy() * random.uniform(0.1, 0.5)
        grain *= np.hanning(len(grain))
        residue[pos:end] += grain
        pos += random.randint(grain_samples // 4, grain_samples * 2)
    out = samples + residue * amount
    peak = np.max(np.abs(out))
    if peak > 0:
        out = out / peak * 0.95
    return out


TEXTURE_REGISTRY = {
    "vinyl": add_vinyl_noise,
    "tape_hiss": add_tape_hiss,
    "air": add_air_layer,
    "granular": add_granular_residue,
}


def apply_texture(samples: np.ndarray, texture: str, amount: float = 0.02,
                  **kwargs) -> np.ndarray:
    fn = TEXTURE_REGISTRY.get(texture)
    if fn:
        return fn(samples, amount=amount, **kwargs)
    return samples
