import math
import numpy as np
from scipy import signal as sp_signal
from gen import SAMPLE_RATE


def noise_like(phase: float) -> float:
    """Deterministic noise function (matching Rust impl)."""
    return ((phase * 127.1 + 33.3) % 1.0 * 43758.5453) % 1.0 * 2.0 - 1.0


def envelope_adsr(i: int, num_samples: int, attack_pct: float = 0.01,
                  decay_pct: float = 0.1, sustain_level: float = 0.0,
                  release_pct: float = 0.5) -> float:
    total = max(num_samples, 1)
    attack_s = int(attack_pct * total)
    decay_s = attack_s + int(decay_pct * total)
    release_s = max(int(num_samples - release_pct * total), 0)

    if i < attack_s:
        p = i / max(attack_s, 1)
        return p * (2.0 - p)
    elif i < decay_s:
        t = (i - attack_s) / max(decay_s - attack_s, 1)
        decay = 1.0 - (1.0 - sustain_level) * (t * t)
        return max(decay, 0.0)
    elif i < release_s:
        return sustain_level
    else:
        t = (i - release_s) / max(num_samples - release_s, 1)
        return sustain_level * (1.0 - t) ** 2


def tape_saturation(x: float, drive: float) -> float:
    x = x * drive
    if x > 1.0:
        return (x + 1.0) / 2.0
    elif x < -1.0:
        return (x - 1.0) / 2.0
    return x * (1.0 + x * x * 0.333)


def soft_clip(x: float, threshold: float = 1.0) -> float:
    if x > threshold:
        return threshold + (x - threshold) / (1.0 + (x - threshold) ** 2)
    elif x < -threshold:
        return -threshold + (x + threshold) / (1.0 + (x + threshold) ** 2)
    return x


def biquad_low_shelf(samples: np.ndarray, freq: float, gain_db: float, q: float = 0.7):
    """Apply a low shelf filter."""
    sr = SAMPLE_RATE
    w0 = 2 * math.pi * freq / sr
    A = 10 ** (gain_db / 40.0)
    alpha = math.sin(w0) / (2 * q)
    cos_w0 = math.cos(w0)

    b0 = A * ((A + 1) - (A - 1) * cos_w0 + 2 * math.sqrt(A) * alpha)
    b1 = 2 * A * ((A - 1) - (A + 1) * cos_w0)
    b2 = A * ((A + 1) - (A - 1) * cos_w0 - 2 * math.sqrt(A) * alpha)
    a0 = (A + 1) + (A - 1) * cos_w0 + 2 * math.sqrt(A) * alpha
    a1 = -2 * ((A - 1) + (A + 1) * cos_w0)
    a2 = (A + 1) + (A - 1) * cos_w0 - 2 * math.sqrt(A) * alpha

    b = np.array([b0, b1, b2]) / a0
    a = np.array([1.0, a1 / a0, a2 / a0])
    return sp_signal.filtfilt(b, a, samples)


def biquad_high_shelf(samples: np.ndarray, freq: float, gain_db: float, q: float = 0.7):
    """Apply a high shelf filter."""
    sr = SAMPLE_RATE
    w0 = 2 * math.pi * freq / sr
    A = 10 ** (gain_db / 40.0)
    alpha = math.sin(w0) / (2 * q)
    cos_w0 = math.cos(w0)

    b0 = A * ((A + 1) + (A - 1) * cos_w0 + 2 * math.sqrt(A) * alpha)
    b1 = -2 * A * ((A - 1) + (A + 1) * cos_w0)
    b2 = A * ((A + 1) + (A - 1) * cos_w0 - 2 * math.sqrt(A) * alpha)
    a0 = (A + 1) - (A - 1) * cos_w0 + 2 * math.sqrt(A) * alpha
    a1 = 2 * ((A - 1) - (A + 1) * cos_w0)
    a2 = (A + 1) - (A - 1) * cos_w0 - 2 * math.sqrt(A) * alpha

    b = np.array([b0, b1, b2]) / a0
    a = np.array([1.0, a1 / a0, a2 / a0])
    return sp_signal.filtfilt(b, a, samples)


def biquad_peaking(samples: np.ndarray, freq: float, gain_db: float, q: float = 2.0):
    """Apply a peaking EQ filter."""
    sr = SAMPLE_RATE
    w0 = 2 * math.pi * freq / sr
    A = 10 ** (gain_db / 40.0)
    alpha = math.sin(w0) / (2 * q)
    cos_w0 = math.cos(w0)

    b0 = 1 + alpha * A
    b1 = -2 * cos_w0
    b2 = 1 - alpha * A
    a0 = 1 + alpha / A
    a1 = -2 * cos_w0
    a2 = 1 - alpha / A

    b = np.array([b0, b1, b2]) / a0
    a = np.array([1.0, a1 / a0, a2 / a0])
    return sp_signal.filtfilt(b, a, samples)
