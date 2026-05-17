import math
import random
import numpy as np
from scipy import signal as sp_signal
from gen import SAMPLE_RATE
from gen.dsp import noise_like, tape_saturation, biquad_low_shelf, biquad_high_shelf


def synthesize_kick(duration_ms: float = 280.0, pitch_hz: float = 100.0,
                    profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    if profiles and "kick" in profiles:
        k = profiles["kick"]
        ref_centroid = k["spectral_centroid"]["mean"]
        target_pitch = max(40.0, ref_centroid * 0.05)
        pitch_hz = np.clip(pitch_hz * 0.7 + target_pitch * 0.3, 40, 150)

    click_samples = min(int(0.008 * SAMPLE_RATE), num_samples)
    for i in range(click_samples):
        t = i / SAMPLE_RATE
        env = math.exp(-120.0 * t)
        click = math.sin(2 * math.pi * 5000.0 * t) * env * 0.4
        click += noise_like(i * 0.7) * env * 0.15
        samples[i] += click

    impact_samples = min(int(0.03 * SAMPLE_RATE), num_samples)
    for i in range(impact_samples):
        t = i / SAMPLE_RATE
        impact_freq = 2000.0 - 1200.0 * (t / 0.015)
        env = math.exp(-80.0 * t)
        impact = math.sin(2 * math.pi * impact_freq * t) * env * 0.3
        samples[i] += impact

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        body_freq = pitch_hz * 2.0 - pitch_hz * 1.5 * (t / 0.3)
        body_freq = max(body_freq, pitch_hz * 0.6)
        body_env = math.exp(-8.0 * t)
        body = math.sin(2 * math.pi * body_freq * t) * body_env * 0.5
        body += math.sin(2 * math.pi * body_freq * 1.5 * t) * body_env * 0.2
        body += math.sin(2 * math.pi * body_freq * 2.0 * t) * body_env * 0.1
        samples[i] += body * 0.7

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        sub_freq = pitch_hz * 0.6 - 10.0 * (t / 0.3)
        sub_freq = max(sub_freq, 30.0)
        sub_env = math.exp(-3.5 * t)
        sub = math.sin(2 * math.pi * sub_freq * t) * sub_env * 0.4
        sub += math.sin(2 * math.pi * sub_freq * 0.5 * t) * sub_env * 0.15
        samples[i] += sub

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        dist_env = math.exp(-10.0 * t)
        saturated = tape_saturation(samples[i] * 1.5, 1.6)
        samples[i] = samples[i] * 0.7 + saturated * dist_env * 0.3

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    return samples.astype(np.float32)


def synthesize_snare(duration_ms: float = 320.0, pitch_hz: float = 220.0,
                     profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        frac = i / num_samples
        tone_freq = pitch_hz - 15.0 * frac
        tone_env = math.exp(-10.0 * t)
        tone = math.sin(2 * math.pi * tone_freq * t) * tone_env * 0.35
        tone += math.sin(2 * math.pi * tone_freq * 2.0 * t) * tone_env * 0.10
        samples[i] += tone

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        n1 = noise_like(i * 0.02)
        n2 = noise_like(i * 0.06)
        n3 = noise_like(i * 0.12)
        noise_body = n1 * 0.35 + n2 * 0.40 + n3 * 0.25
        noise_env = math.exp(-12.0 * t)
        samples[i] += noise_body * noise_env * 0.40

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        if t < 0.02:
            crack_env = math.exp(-140.0 * t)
            crack = noise_like(i * 0.15) * crack_env * 0.20
            samples[i] += crack

    sos = sp_signal.butter(3, 6000, 'lowpass', fs=SAMPLE_RATE, output='sos')
    samples = sp_signal.sosfilt(sos, samples)

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


def synthesize_clap(duration_ms: float = 380.0, pitch_hz: float = 180.0,
                    profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    hit_times = [0.0, 0.010, 0.024, 0.042, 0.065, 0.095]
    burst_amps = [0.20, 0.38, 0.30, 0.22, 0.16, 0.10]

    noise = np.random.randn(num_samples + 4096).astype(np.float32)
    sos_hp = sp_signal.butter(4, 500, 'highpass', fs=SAMPLE_RATE, output='sos')
    noise = sp_signal.sosfilt(sos_hp, noise)

    sos_body = sp_signal.butter(4, [800, 3800], 'bandpass', fs=SAMPLE_RATE, output='sos')
    noise_body = sp_signal.sosfilt(sos_body, noise)
    sos_dark = sp_signal.butter(4, 2000, 'lowpass', fs=SAMPLE_RATE, output='sos')
    noise_dark = sp_signal.sosfilt(sos_dark, noise)

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for j, (hit_t, amp) in enumerate(zip(hit_times, burst_amps)):
            dt = t - hit_t
            if dt < 0:
                continue
            bright = max(0.0, 1.0 - j * 0.15)
            hit_env = math.exp(-80.0 * dt)
            burst_body = noise_body[i] * bright + noise_dark[i] * (1.0 - bright * 0.5)
            val += burst_body * hit_env * amp
        samples[i] = val * 0.40

    for i in range(num_samples):
        samples[i] = tape_saturation(samples[i], 1.2)

    delay_samples = int(0.030 * SAMPLE_RATE)
    for i in range(delay_samples, num_samples):
        samples[i] += samples[i - delay_samples] * 0.04

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


def synthesize_closed_hat(duration_ms: float = 100.0, pitch_hz: float = 1000.0,
                          profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    metallic_partials = [
        (1.0, 0.3), (2.68, 0.5), (4.0, 0.7), (5.33, 0.6),
        (7.6, 0.4), (10.8, 0.25), (15.0, 0.15),
    ]

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for ratio, amp in metallic_partials:
            freq = pitch_hz * ratio
            decay = math.exp(-50.0 * t * (1.0 + ratio * 0.25))
            val += math.sin(2 * math.pi * freq * t) * amp * decay
        if t < 0.004:
            noise_tick = np.random.randn() * math.exp(-600.0 * t) * 0.25
            val += noise_tick
        samples[i] = val

    sos_hp = sp_signal.butter(4, 2500, 'highpass', fs=SAMPLE_RATE, output='sos')
    samples = sp_signal.sosfilt(sos_hp, samples)
    samples = biquad_high_shelf(samples, 5000.0, 5.0, 0.6)

    gate_samples = min(int(0.055 * SAMPLE_RATE), num_samples)
    for i in range(gate_samples, num_samples):
        samples[i] *= 0.0

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.7
    return samples.astype(np.float32)


def synthesize_open_hat(duration_ms: float = 650.0, pitch_hz: float = 350.0,
                        profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    for i in range(num_samples):
        t = i / SAMPLE_RATE

        tone_body = math.sin(2 * math.pi * 400.0 * t) * 0.06
        tone_wash = math.sin(2 * math.pi * 2000.0 * t) * 0.10
        tone_high = math.sin(2 * math.pi * 3800.0 * t) * 0.08
        tone = tone_body + tone_wash + tone_high

        n1 = noise_like(i * 0.02)
        n2 = noise_like(i * 0.06)
        n3 = noise_like(i * 0.12)
        noise_val = n1 * 0.20 + n2 * 0.35 + n3 * 0.45

        env = math.exp(-3.5 * t)
        click_env = math.exp(-60.0 * t)

        samples[i] = (tone * 0.15 + noise_val * 0.85) * env + noise_val * click_env * 0.03

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.8
    return samples.astype(np.float32)


def synthesize_808(duration_ms: float = 800.0, pitch_hz: float = 55.0,
                   profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        frac = i / num_samples

        freq = pitch_hz - pitch_hz * 0.4 * frac
        freq = max(freq, 25.0)

        sub = math.sin(2 * math.pi * freq * t) * 0.5
        sub += math.sin(2 * math.pi * freq * 0.5 * t) * 0.15
        sub += math.sin(2 * math.pi * freq * 2.0 * t) * 0.08

        env = math.exp(-2.5 * t)

        val = sub * env
        val = tape_saturation(val * 1.3, 1.4)

        samples[i] = val

    samples = biquad_low_shelf(samples, 80.0, 4.0, 0.7)

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    return samples.astype(np.float32)


def synthesize_bass_stab(duration_ms: float = 350.0, pitch_hz: float = 80.0,
                         profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    click_samples = min(int(0.005 * SAMPLE_RATE), num_samples)
    for i in range(click_samples):
        t = i / SAMPLE_RATE
        env = math.exp(-300.0 * t)
        click = math.sin(2 * math.pi * 3000.0 * t) * env * 0.2
        click += noise_like(i * 0.9) * env * 0.1
        samples[i] += click

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        frac = i / num_samples

        freq = pitch_hz - pitch_hz * 0.3 * frac
        body_env = math.exp(-6.0 * t)

        body = math.sin(2 * math.pi * freq * t) * body_env * 0.5
        body += math.sin(2 * math.pi * freq * 2.0 * t) * body_env * 0.2
        body += math.sin(2 * math.pi * freq * 3.0 * t) * body_env * 0.1

        sub = math.sin(2 * math.pi * pitch_hz * 0.5 * t) * math.exp(-4.0 * t) * 0.35

        samples[i] += body + sub

    for i in range(len(samples)):
        samples[i] = tape_saturation(samples[i], 1.5)
    samples = biquad_low_shelf(samples, 100.0, 3.0, 0.7)

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    return samples.astype(np.float32)


def synthesize_impact_fx(duration_ms: float = 1500.0, pitch_hz: float = 70.0,
                         profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    impact_samples = min(int(0.05 * SAMPLE_RATE), num_samples)
    for i in range(impact_samples):
        t = i / SAMPLE_RATE
        env = math.exp(-80.0 * t)
        impact = noise_like(i * 0.5) * env * 0.5
        impact += math.sin(2 * math.pi * pitch_hz * 4.0 * t) * env * 0.3
        samples[i] += impact

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        frac = i / num_samples
        sub_freq = pitch_hz - pitch_hz * 0.5 * frac
        sub_freq = max(sub_freq, 25.0)
        sub_env = math.exp(-1.5 * t)
        sub = math.sin(2 * math.pi * sub_freq * t) * sub_env * 0.4
        samples[i] += sub

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        if t > 0.02:
            noise_env = math.exp(-0.8 * t)
            n = noise_like(i * 0.06) * 0.4 + noise_like(i * 0.15) * 0.3 + noise_like(i * 0.3) * 0.3
            samples[i] += n * noise_env * 0.15

    for i in range(1, num_samples):
        samples[i] += samples[i-1] * 0.15

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    return samples.astype(np.float32)


def synthesize_synth_stab(duration_ms: float = 600.0, pitch_hz: float = 220.0,
                          profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    detunes = [1.0, 1.01, 0.99, 1.02, 0.98, 1.005, 0.995]
    amps = [0.3, 0.2, 0.2, 0.1, 0.1, 0.05, 0.05]

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for detune, amp in zip(detunes, amps):
            freq = pitch_hz * detune
            val += math.sin(2 * math.pi * freq * t) * amp
        val += math.sin(2 * math.pi * pitch_hz * 1.5 * t) * 0.15
        env = math.exp(-3.0 * t)
        samples[i] = val * env * 0.5

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        frac = i / num_samples
        filter_cutoff = 2000.0 + 4000.0 * math.sin(math.pi * frac)
        dt = 1.0 / SAMPLE_RATE
        rc = 1.0 / (2 * math.pi * max(filter_cutoff, 20.0))
        alpha = dt / (rc + dt)
        if i > 0:
            samples[i] = samples[i-1] + alpha * (samples[i] - samples[i-1])

    for i in range(len(samples)):
        samples[i] = tape_saturation(samples[i], 1.3)

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.85
    return samples.astype(np.float32)


def synthesize_guitar_stab(duration_ms: float = 500.0, pitch_hz: float = 220.0,
                           profiles: dict = None) -> np.ndarray:
    num_samples = int(SAMPLE_RATE * duration_ms / 1000.0)
    samples = np.zeros(num_samples)

    harmonics = [(1.0, 1.0), (2.0, 0.6), (3.0, 0.45), (4.0, 0.3),
                 (5.0, 0.2), (6.0, 0.12), (7.0, 0.08), (8.0, 0.05)]

    for i in range(num_samples):
        t = i / SAMPLE_RATE
        val = 0.0
        for h, amp in harmonics:
            freq = pitch_hz * h
            h_decay = math.exp(-2.0 * t * h ** 0.5)
            val += math.sin(2 * math.pi * freq * t) * amp * h_decay
        if t < 0.005:
            pluck = noise_like(i * 0.9) * math.exp(-600.0 * t) * 0.15
            val += pluck
        body_res = math.sin(2 * math.pi * 120.0 * t) * math.exp(-4.0 * t) * 0.08
        val += body_res
        norm = sum(amp for _, amp in harmonics)
        samples[i] = val / norm * 0.6

    for i in range(len(samples)):
        samples[i] = tape_saturation(samples[i], 1.2)

    peak = np.max(np.abs(samples))
    if peak > 0:
        samples = samples / peak * 0.9
    return samples.astype(np.float32)


SYNTHESIS_CLASSES = {
    "kick": ("Kick", synthesize_kick, 280.0, 100.0),
    "snare": ("Snare", synthesize_snare, 320.0, 220.0),
    "clap": ("Clap", synthesize_clap, 380.0, 180.0),
    "closed_hat": ("Closed Hat", synthesize_closed_hat, 100.0, 1000.0),
    "open_hat": ("Open Hat", synthesize_open_hat, 650.0, 350.0),
    "808": ("808 Sub", synthesize_808, 800.0, 55.0),
    "bass_stab": ("Bass Stab", synthesize_bass_stab, 350.0, 80.0),
    "impact_fx": ("Impact FX", synthesize_impact_fx, 1500.0, 70.0),
    "synth_stab": ("Synth Stab", synthesize_synth_stab, 600.0, 220.0),
    "guitar_stab": ("Guitar Stab", synthesize_guitar_stab, 500.0, 220.0),
}
