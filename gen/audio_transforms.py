"""High-quality audio transformations for reference-conditioned generation.

All transforms operate on numpy float32 arrays at 44100 Hz.
Designed to make real reference samples sound like new, unique productions.
"""

import math
import random
from typing import Optional

import numpy as np
from scipy import signal as sp_signal

from gen import SAMPLE_RATE


def pitch_shift(samples: np.ndarray, semitones: float) -> np.ndarray:
    """Pitch shift using phase vocoder with overlap-add."""
    if abs(semitones) < 0.5:
        return samples.copy()
    n = len(samples)
    if n < 256:
        return samples.copy()

    import librosa
    shifted = librosa.effects.pitch_shift(
        y=samples.astype(np.float32), sr=SAMPLE_RATE,
        n_steps=semitones, bins_per_octave=12
    )
    if len(shifted) > n:
        shifted = shifted[:n]
    elif len(shifted) < n:
        shifted = np.pad(shifted, (0, n - len(shifted)))
    return shifted.astype(np.float32)


def time_stretch(samples: np.ndarray, rate: float) -> np.ndarray:
    """Time stretch. rate < 1 = faster/shorter, rate > 1 = slower/longer."""
    if abs(rate - 1.0) < 0.01:
        return samples.copy()
    n = len(samples)
    if n < 256:
        return samples.copy()

    import librosa
    stretched = librosa.effects.time_stretch(
        y=samples.astype(np.float32), rate=rate
    )
    new_n = int(n / rate)
    if len(stretched) > new_n:
        stretched = stretched[:new_n]
    elif len(stretched) < new_n:
        stretched = np.pad(stretched, (0, new_n - len(stretched)))
    return stretched.astype(np.float32)


def transient_reshape(samples: np.ndarray, gain: float = 1.0,
                       attack_boost: float = 0.0, sustain_cut: float = 0.0) -> np.ndarray:
    """Reshape transients and sustain independently using multi-band envelope."""
    n = len(samples)
    if n < 256:
        return samples.copy()

    import librosa
    onset_env = librosa.onset.onset_strength(y=samples.astype(np.float32), sr=SAMPLE_RATE)
    onset_frames = librosa.onset.onset_detect(onset_envelope=onset_env, sr=SAMPLE_RATE, backtrack=True)

    result = samples.copy()
    hop_length = 512
    max_onset_samples = int(0.05 * SAMPLE_RATE)

    for frame in onset_frames:
        center = int(frame * hop_length)
        start = max(0, center - int(0.005 * SAMPLE_RATE))
        end = min(n, center + max_onset_samples)

        if end - start < 10:
            continue

        segment = result[start:end].copy()
        if attack_boost != 0:
            boost_db = attack_boost * 6.0
            env = np.linspace(1.0, 0.0, len(segment)) ** 2
            if boost_db > 0:
                segment = segment * (1.0 + env * (10 ** (boost_db / 20) - 1.0))
            else:
                segment = segment * (1.0 - env * (1.0 - 10 ** (boost_db / 20)))

        if sustain_cut != 0:
            gate_db = sustain_cut * 12.0
            gate_ratio = 10 ** (gate_db / 20)
            segment[int(len(segment) * 0.3):] *= gate_ratio

        result[start:end] = segment

    if gain != 1.0:
        result *= gain

    return result.astype(np.float32)


def eq_tilt(samples: np.ndarray, tilt_db: float = 0.0,
            shelf_freq: float = 1000.0) -> np.ndarray:
    """Spectral tilt using low/high shelf filtering."""
    if abs(tilt_db) < 0.5:
        return samples.copy()

    if tilt_db > 0:
        sos = sp_signal.butter(2, shelf_freq / (SAMPLE_RATE / 2), 'highpass', output='sos')
    else:
        sos = sp_signal.butter(2, shelf_freq / (SAMPLE_RATE / 2), 'lowpass', output='sos')

    filtered = sp_signal.sosfilt(sos, samples)

    dry = abs(tilt_db) / 12.0
    wet = min(1.0, abs(tilt_db) / 6.0)
    result = samples * (1.0 - wet) + filtered * wet
    return result.astype(np.float32)


def parametric_eq(samples: np.ndarray, freq: float, gain_db: float,
                  q: float = 1.0) -> np.ndarray:
    """Parametric peaking EQ filter."""
    if abs(gain_db) < 0.5:
        return samples.copy()

    w0 = 2 * math.pi * freq / SAMPLE_RATE
    alpha = math.sin(w0) / (2 * q)
    A = 10 ** (gain_db / 40.0)

    b0 = 1 + alpha * A
    b1 = -2 * math.cos(w0)
    b2 = 1 - alpha * A
    a0 = 1 + alpha / A
    a1 = -2 * math.cos(w0)
    a2 = 1 - alpha / A

    b = np.array([b0, b1, b2]) / a0
    a = np.array([1.0, a1 / a0, a2 / a0])
    return sp_signal.filtfilt(b, a, samples).astype(np.float32)


def saturation(samples: np.ndarray, drive: float = 0.0,
               mode: str = 'tape') -> np.ndarray:
    """Apply saturation/distortion to add harmonics and perceived loudness."""
    if drive < 0.01:
        return samples.copy()

    result = samples.copy() * (1.0 + drive * 2.0)

    if mode == 'tape':
        result = np.tanh(result * 1.5) / np.tanh(1.5)
    elif mode == 'tube':
        result = np.sign(result) * (1.0 - np.exp(-np.abs(result * 3.0)))
        result = result / max(np.max(np.abs(result)), 0.001)
    elif mode == 'soft':
        result = np.clip(result * 0.8, -1.0, 1.0)
        result = result - (result ** 3) / 3.0
        result = result / max(np.max(np.abs(result)), 0.001)
    elif mode == 'hard':
        result = np.clip(result, -1.0, 1.0)
    elif mode == 'diode':
        result = np.where(result > 0, result / (1 + result), result)
        result = result / max(np.max(np.abs(result)), 0.001)

    result = result * 0.9
    return result.astype(np.float32)


def convolution_reverb(samples: np.ndarray, ir_path: Optional[str] = None,
                        wet: float = 0.3, ir_duration: float = 0.5) -> np.ndarray:
    """Apply convolution reverb using a synthetic room impulse response."""
    if wet < 0.01:
        return samples.copy()
    n = len(samples)
    if n < 100:
        return samples.copy()

    ir_len = int(ir_duration * SAMPLE_RATE)
    ir = _generate_room_ir(ir_len)
    convolved = sp_signal.fftconvolve(samples, ir, mode='full')[:n]
    result = samples * (1.0 - wet) + convolved * wet

    peak = np.max(np.abs(result))
    if peak > 0:
        result = result / peak * 0.95
    return result.astype(np.float32)


def _generate_room_ir(length: int, decay: float = 0.3) -> np.ndarray:
    """Generate a synthetic room impulse response."""
    noise = np.random.randn(length).astype(np.float32)
    env = np.exp(-np.arange(length) / (length * decay))
    ir = noise * env
    sos_lp = sp_signal.butter(6, 8000 / (SAMPLE_RATE / 2), 'lowpass', output='sos')
    ir = sp_signal.sosfilt(sos_lp, ir)
    ir = ir / max(np.max(np.abs(ir)), 0.001) * 0.5
    return ir


def spectral_morph(samples_a: np.ndarray, samples_b: np.ndarray,
                    morph_amount: float = 0.5) -> np.ndarray:
    """Cross-synthesis: morph spectral envelope of A toward B."""
    n = min(len(samples_a), len(samples_b))
    if n < 512:
        return samples_a.copy()

    import librosa
    a = samples_a[:n].astype(np.float32)
    b = samples_b[:n].astype(np.float32)

    n_fft = 2048
    hop = 512

    stft_a = librosa.stft(a, n_fft=n_fft, hop_length=hop)
    stft_b = librosa.stft(b, n_fft=n_fft, hop_length=hop)

    mag_a, phase_a = np.abs(stft_a), np.angle(stft_a)
    mag_b, _ = np.abs(stft_b), np.angle(stft_b)

    mag_morph = mag_a * (1.0 - morph_amount) + mag_b * morph_amount

    stft_morph = mag_morph * np.exp(1j * phase_a)
    result = librosa.istft(stft_morph, hop_length=hop, length=n)
    peak = np.max(np.abs(result))
    if peak > 0:
        result = result / peak * 0.95
    return result.astype(np.float32)


def envelope_morph(samples: np.ndarray, target_env: np.ndarray,
                    amount: float = 1.0) -> np.ndarray:
    """Transfer amplitude envelope from target to samples."""
    n = min(len(samples), len(target_env))
    if n < 100:
        return samples.copy()

    source = samples[:n].copy()
    target = target_env[:n]

    src_env = np.abs(source)
    src_peak = np.max(src_env)
    if src_peak < 0.001:
        return samples.copy()

    tgt_env = np.abs(target)
    tgt_peak = np.max(tgt_env)
    if tgt_peak < 0.001:
        return samples.copy()

    src_env_norm = src_env / src_peak
    tgt_env_norm = tgt_env / tgt_peak

    # Smooth the envelope
    window = int(0.005 * SAMPLE_RATE)
    if window > 1:
        kernel = np.ones(window) / window
        src_env_norm = np.convolve(src_env_norm, kernel, mode='same')
        tgt_env_norm = np.convolve(tgt_env_norm, kernel, mode='same')

    combo = src_env_norm * (1.0 - amount) + tgt_env_norm * amount
    eps = 1e-8
    result = source / (src_env_norm + eps) * (combo + eps)
    peak = np.max(np.abs(result))
    if peak > 0:
        result = result / peak * 0.95
    return result.astype(np.float32)


def hpss_split(samples: np.ndarray, kernel_size: int = 31) -> tuple[np.ndarray, np.ndarray]:
    """Split into harmonic and percussive components."""
    n = len(samples)
    if n < 512:
        return samples.copy(), np.zeros_like(samples)

    import librosa
    D = librosa.stft(samples.astype(np.float32))
    D_harm, D_perc = librosa.decompose.hpss(D, kernel_size=kernel_size)
    harm = librosa.istft(D_harm, length=n)
    perc = librosa.istft(D_perc, length=n)
    return harm.astype(np.float32), perc.astype(np.float32)


def noise_body_tail_recombine(samples: np.ndarray, noise_gain: float = 0.0,
                               body_gain: float = 0.0, tail_gain: float = 0.0) -> np.ndarray:
    """Split sound into noise/body/tail and recombine with different gains."""
    n = len(samples)
    if n < 256:
        return samples.copy()

    env = np.abs(samples)
    peak_idx = np.argmax(env)
    peak_val = env[peak_idx]

    # Find noise floor: first few samples before onset
    noise_floor_start = max(0, peak_idx - int(0.1 * SAMPLE_RATE))
    noise_floor = float(np.percentile(env[:noise_floor_start], 10)) if noise_floor_start > 100 else peak_val * 0.01

    # Body: around the peak
    body_start = max(0, peak_idx - int(0.02 * SAMPLE_RATE))
    body_end = min(n, peak_idx + int(0.2 * SAMPLE_RATE))

    # Tail: after body
    tail_start = body_end
    tail_end = n

    result = samples.copy()

    if noise_gain != 0:
        noise = np.random.randn(n).astype(np.float32) * noise_floor * noise_gain
        result += noise

    if body_gain != 0:
        result[body_start:body_end] *= (1.0 + body_gain)

    if tail_gain != 0:
        tail_db = tail_gain * 12.0
        tail_ratio = 10 ** (tail_db / 20)
        result[tail_start:tail_end] *= tail_ratio

    peak = np.max(np.abs(result))
    if peak > 0:
        result = result / peak * 0.95
    return result.astype(np.float32)


def stereo_width(samples: np.ndarray, width: float = 1.0) -> np.ndarray:
    """Adjust stereo width using mid-side processing.
    width=0: mono, width=1: original, width>1: wider.
    """
    if samples.ndim != 2 or samples.shape[1] != 2:
        return samples.copy() if samples.ndim == 1 else samples.mean(axis=1)
    if abs(width - 1.0) < 0.01:
        return samples.copy()

    mid = (samples[:, 0] + samples[:, 1]) / 2.0
    side = (samples[:, 0] - samples[:, 1]) / 2.0

    side *= width
    left = mid + side
    right = mid - side

    result = np.column_stack([left, right])
    peak = np.max(np.abs(result))
    if peak > 0:
        result = result / peak * 0.95
    return result.astype(np.float32)


def layer_sounds(samples_list: list[np.ndarray]) -> np.ndarray:
    """Layer multiple sounds together with intelligent gain staging."""
    if not samples_list:
        return np.zeros(0, dtype=np.float32)

    max_len = max(len(s) for s in samples_list)
    result = np.zeros(max_len, dtype=np.float32)

    for s in samples_list:
        s_norm = s / max(np.max(np.abs(s)), 0.001) * 0.5
        if len(s_norm) < max_len:
            s_norm = np.pad(s_norm, (0, max_len - len(s_norm)))
        result += s_norm

    peak = np.max(np.abs(result))
    if peak > 0:
        result = result / peak * 0.95
    return result.astype(np.float32)


def resample_lofi(samples: np.ndarray, sample_rate_reduction: float = 1.0,
                   bit_depth: int = 16) -> np.ndarray:
    """Lo-fi effect by reducing sample rate and/or bit depth."""
    if sample_rate_reduction >= 1.0 and bit_depth >= 16:
        return samples.copy()

    result = samples.copy()

    if sample_rate_reduction < 1.0:
        orig_len = len(result)
        new_sr = int(SAMPLE_RATE * sample_rate_reduction)
        new_len = int(orig_len * sample_rate_reduction)
        if new_len > 10:
            downsampled = sp_signal.resample(result, new_len)
            result = sp_signal.resample(downsampled, orig_len)

    if bit_depth < 16:
        steps = 2 ** bit_depth
        max_val = np.max(np.abs(result))
        if max_val > 0.001:
            result = np.round(result / max_val * (steps / 2)) / (steps / 2) * max_val

    return result.astype(np.float32)


def apply_transform_chain(samples: np.ndarray,
                           transforms: list[dict]) -> np.ndarray:
    """Apply a chain of transforms to samples.
    Each transform dict: {'type': str, 'params': {...}}
    """
    result = samples.copy()

    for t in transforms:
        t_type = t.get('type', '')
        params = {k: v for k, v in t.items() if k != 'type'}

        try:
            if t_type == 'pitch_shift':
                result = pitch_shift(result, **params)
            elif t_type == 'time_stretch':
                result = time_stretch(result, **params)
            elif t_type == 'transient_reshape':
                result = transient_reshape(result, **params)
            elif t_type == 'eq_tilt':
                result = eq_tilt(result, **params)
            elif t_type == 'parametric_eq':
                result = parametric_eq(result, **params)
            elif t_type == 'saturation':
                result = saturation(result, **params)
            elif t_type == 'convolution_reverb':
                result = convolution_reverb(result, **params)
            elif t_type == 'envelope_morph':
                pass  # needs target env
            elif t_type == 'noise_body_tail':
                result = noise_body_tail_recombine(result, **params)
            elif t_type == 'stereo_width':
                result = stereo_width(result, **params)
            elif t_type == 'resample_lofi':
                result = resample_lofi(result, **params)
        except Exception:
            pass

    peak = np.max(np.abs(result))
    if peak > 0.001:
        result = result / peak * 0.95
    return result.astype(np.float32)


def generate_random_transform_chain(ref_category: str = '') -> list[dict]:
    """Generate a random transform chain appropriate for a sound category."""
    chain = []
    rng = random.Random()

    # 80% chance of pitch shift (±1-12 semitones)
    semitones = rng.uniform(-12, 12)
    if abs(semitones) > 1:
        chain.append({'type': 'pitch_shift', 'semitones': round(semitones, 1)})

    # 40% chance of time stretch
    if rng.random() < 0.4:
        rate = rng.uniform(0.5, 2.0)
        if abs(rate - 1.0) > 0.05:
            chain.append({'type': 'time_stretch', 'rate': round(rate, 2)})

    # 60% chance of transient reshape
    if rng.random() < 0.6:
        attack = rng.uniform(-1.0, 1.0)
        sustain = rng.uniform(-1.0, 0.5)
        if abs(attack) > 0.1 or abs(sustain) > 0.1:
            chain.append({
                'type': 'transient_reshape',
                'attack_boost': round(attack, 2),
                'sustain_cut': round(sustain, 2),
            })

    # 50% chance of EQ tilt
    if rng.random() < 0.5:
        tilt = rng.uniform(-6, 6)
        if abs(tilt) > 0.5:
            chain.append({'type': 'eq_tilt', 'tilt_db': round(tilt, 1),
                         'shelf_freq': rng.choice([200, 500, 1000, 2000, 4000])})

    # 50% chance of saturation
    if rng.random() < 0.5:
        drive = rng.uniform(0.1, 0.8)
        mode = rng.choice(['tape', 'tube', 'soft', 'hard'])
        chain.append({'type': 'saturation', 'drive': round(drive, 2), 'mode': mode})

    # 30% chance of convolution reverb
    if rng.random() < 0.3:
        chain.append({
            'type': 'convolution_reverb',
            'wet': round(rng.uniform(0.05, 0.4), 2),
            'ir_duration': round(rng.uniform(0.1, 0.8), 2),
        })

    # 30% chance of noise/body/tail manipulation
    if rng.random() < 0.3:
        chain.append({
            'type': 'noise_body_tail',
            'noise_gain': round(rng.uniform(-0.5, 0.5), 2),
            'body_gain': round(rng.uniform(-0.3, 0.3), 2),
            'tail_gain': round(rng.uniform(-0.5, 0.3), 2),
        })

    # 20% chance of resample/lo-fi
    if rng.random() < 0.2:
        chain.append({
            'type': 'resample_lofi',
            'sample_rate_reduction': round(rng.uniform(0.25, 0.9), 2),
            'bit_depth': rng.choice([8, 12]),
        })

    return chain
