import math
import numpy as np
from scipy import signal as sp_signal
from gen import SAMPLE_RATE

FEATURE_KEYS_FULL = [
    "duration_ms", "rms", "peak", "zero_crossing_rate",
    "spectral_centroid", "spectral_bandwidth", "spectral_rolloff",
    "low_band_energy", "mid_band_energy", "high_band_energy",
    "early_rms", "noise_floor", "stereo_correlation",
    "transient_count", "amplitude_peaks", "transient_strength",
    "decay_length_ms", "attack_ms",
    "spectral_flux_mean", "spectral_flux_std",
    "hpr", "pitch_hz", "pitch_confidence",
    "mfcc_1", "mfcc_2", "mfcc_3", "mfcc_4", "mfcc_5",
    "mfcc_6", "mfcc_7", "mfcc_8", "mfcc_9", "mfcc_10",
    "mfcc_11", "mfcc_12", "mfcc_13",
    "chroma_0", "chroma_1", "chroma_2", "chroma_3",
    "chroma_4", "chroma_5", "chroma_6", "chroma_7",
    "chroma_8", "chroma_9", "chroma_10", "chroma_11",
]



def compute_rms(samples: np.ndarray) -> float:
    return float(np.sqrt(np.mean(samples ** 2)))


def compute_peak(samples: np.ndarray) -> float:
    return float(np.max(np.abs(samples)))


def compute_zcr(samples: np.ndarray) -> float:
    if len(samples) < 2:
        return 0.0
    zero_crossings = np.sum(np.diff(np.signbit(samples)))
    return float(zero_crossings) / len(samples)


def compute_spectral_centroid(samples: np.ndarray, sr: int = SAMPLE_RATE) -> float:
    n = min(len(samples), 2048)
    if n < 4:
        return 0.0
    freqs = np.fft.rfftfreq(n, 1/sr)
    spectrum = np.abs(np.fft.rfft(samples[:n]))
    if np.sum(spectrum) == 0:
        return 0.0
    return float(np.sum(freqs * spectrum) / np.sum(spectrum))


def compute_spectral_rolloff(samples: np.ndarray, percentile: float = 0.85, sr: int = SAMPLE_RATE) -> float:
    n = min(len(samples), 4096)
    if n < 4:
        return 0.0
    spectrum = np.abs(np.fft.rfft(samples[:n])) ** 2
    total = np.sum(spectrum)
    if total <= 0:
        return 0.0
    cum = np.cumsum(spectrum)
    idx = np.searchsorted(cum, total * percentile)
    return float(idx * sr / n)


def compute_band_energies(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    n = min(len(samples), 4096)
    if n < 4:
        return {"low": 0.0, "mid": 0.0, "high": 0.0}
    spectrum = np.abs(np.fft.rfft(samples[:n])) ** 2
    freqs = np.fft.rfftfreq(n, 1/sr)
    total = np.sum(spectrum)
    if total <= 0:
        return {"low": 0.0, "mid": 0.0, "high": 0.0}
    low_mask = freqs < 200
    mid_mask = (freqs >= 200) & (freqs < 4000)
    high_mask = freqs >= 4000
    low = np.sum(spectrum[low_mask]) / total
    mid = np.sum(spectrum[mid_mask]) / total
    high = np.sum(spectrum[high_mask]) / total
    return {"low": float(low), "mid": float(mid), "high": float(high)}


def detect_transients(samples: np.ndarray, sr: int = SAMPLE_RATE) -> tuple[float, int, list[float]]:
    """Detect transients using spectral flux. Returns (strength, count, onset_times_ms)."""
    if len(samples) < 512:
        return (0.0, 0, [])
    frame_size = 512
    hop_size = 256
    prev_spec = np.zeros(frame_size // 2)
    fluxes = []
    for i in range(0, len(samples) - frame_size, hop_size):
        frame = samples[i:i+frame_size]
        spectrum = np.abs(np.fft.rfft(frame))
        flux = np.sum(np.maximum(0, spectrum[:len(prev_spec)] - prev_spec))
        fluxes.append(flux)
        prev_spec = spectrum[:len(prev_spec)]
    fluxes = np.array(fluxes)
    if len(fluxes) < 2:
        return (0.0, 0, [])
    mean_flux = np.mean(fluxes)
    max_flux = np.max(fluxes)
    strength = float(max_flux / mean_flux) if mean_flux > 0 else 1.0
    threshold = mean_flux * 3.0
    onsets = []
    in_onset = False
    for j, f in enumerate(fluxes):
        if f > threshold and not in_onset:
            onsets.append(float(j * hop_size / sr * 1000.0))
            in_onset = True
        elif f <= threshold * 0.5:
            in_onset = False
    return (strength, len(onsets), onsets)


def compute_decay_length(samples: np.ndarray, sr: int = SAMPLE_RATE) -> float:
    """Time (ms) for envelope to decay to 10% of peak."""
    if len(samples) < 100:
        return 0.0
    env = np.abs(samples)
    peak = np.max(env)
    if peak < 0.001:
        return 0.0
    threshold = peak * 0.1
    peak_idx = np.argmax(env)
    for i in range(peak_idx, len(env)):
        if env[i] <= threshold:
            return float((i - peak_idx) / sr * 1000.0)
    return float((len(env) - peak_idx) / sr * 1000.0)


def compute_attack_time(samples: np.ndarray, sr: int = SAMPLE_RATE) -> float:
    """Time from 10% to 90% of peak amplitude."""
    if len(samples) < 100:
        return 0.0
    env = np.abs(samples)
    peak = np.max(env)
    if peak < 0.001:
        return 0.0
    t10_val = peak * 0.1
    t90_val = peak * 0.9
    t10 = np.argmax(env >= t10_val)
    t90 = np.argmax(env >= t90_val)
    if t90 > t10:
        return float((t90 - t10) / sr * 1000.0)
    return 0.0


def count_amplitude_peaks(samples: np.ndarray, sr: int = SAMPLE_RATE,
                           min_gap_ms: float = 3.0, threshold_pct: float = 0.15) -> int:
    """Count burst peaks in amplitude envelope (designed for clap multi-burst detection)."""
    if len(samples) < 100:
        return 0
    env = np.abs(samples)
    peak_val = np.max(env)
    if peak_val < 0.001:
        return 0
    threshold = peak_val * threshold_pct
    min_gap = max(int(min_gap_ms * sr / 1000.0), 2)
    peaks = []
    i = 1
    while i < len(samples) - 1:
        if env[i] > env[i-1] and env[i] >= env[i+1] and env[i] >= threshold:
            peaks.append(i)
            i += min_gap
        else:
            i += 1
    return len(peaks)


def _dct(x: np.ndarray, norm: str = 'ortho') -> np.ndarray:
    """Type-II DCT (matching scipy.fftpack.dct behavior)."""
    n = len(x)
    X = np.zeros(n)
    for k in range(n):
        X[k] = np.sum(x * np.cos(math.pi * k * (2.0 * np.arange(n) + 1.0) / (2.0 * n)))
    if norm == 'ortho':
        X[0] *= 1.0 / math.sqrt(n)
        X[1:] *= math.sqrt(2.0 / n)
    return X


def _mel_filterbank(n_fft: int, sr: int, n_mels: int = 26, fmin: float = 0.0, fmax: float = None) -> np.ndarray:
    """Build a Mel filterbank matrix. Returns (n_mels, n_fft//2+1)."""
    if fmax is None:
        fmax = sr / 2.0
    mel_min = 2595.0 * math.log10(1.0 + fmin / 700.0)
    mel_max = 2595.0 * math.log10(1.0 + fmax / 700.0)
    mel_points = np.linspace(mel_min, mel_max, n_mels + 2)
    hz_points = 700.0 * (10.0 ** (mel_points / 2595.0) - 1.0)
    bins = np.floor((n_fft + 1) * hz_points / sr).astype(int)
    fbank = np.zeros((n_mels, n_fft // 2 + 1))
    for m in range(1, n_mels + 1):
        left = bins[m - 1]
        center = bins[m]
        right = bins[m + 1]
        for k in range(left, center):
            if k < fbank.shape[1]:
                fbank[m - 1, k] = (k - left) / max(center - left, 1)
        for k in range(center, right):
            if k < fbank.shape[1]:
                fbank[m - 1, k] = (right - k) / max(right - center, 1)
    return fbank


def _frame(samples: np.ndarray, frame_size: int, hop_size: int) -> np.ndarray:
    """Split signal into overlapping frames. Returns (n_frames, frame_size)."""
    n_frames = max(1, (len(samples) - frame_size) // hop_size + 1)
    frames = np.zeros((n_frames, frame_size))
    for i in range(n_frames):
        start = i * hop_size
        end = start + frame_size
        frames[i, :min(frame_size, len(samples) - start)] = samples[start:end]
    return frames


def compute_spectral_flux(samples: np.ndarray, sr: int = SAMPLE_RATE,
                          frame_size: int = 1024, hop_size: int = 512) -> tuple[float, float, float]:
    """Compute spectral flux features. Returns (mean_flux, std_flux, max_flux)."""
    if len(samples) < frame_size:
        return 0.0, 0.0, 0.0
    frames = _frame(samples, frame_size, hop_size)
    n_frames = frames.shape[0]
    prev_spec = np.zeros(frame_size // 2)
    fluxes = []
    for f_idx in range(n_frames):
        spectrum = np.abs(np.fft.rfft(frames[f_idx]))
        half_len = min(len(spectrum), len(prev_spec))
        flux = float(np.sum(np.maximum(0, spectrum[:half_len] - prev_spec[:half_len])))
        fluxes.append(flux)
        prev_spec = spectrum[:half_len]
    if len(fluxes) < 2:
        return 0.0, 0.0, 0.0
    arr = np.array(fluxes)
    return float(np.mean(arr)), float(np.std(arr)), float(np.max(arr))


def compute_mfccs(samples: np.ndarray, sr: int = SAMPLE_RATE,
                  n_mfcc: int = 13, frame_size: int = 1024, hop_size: int = 512,
                  n_mels: int = 26) -> list[float]:
    """Compute MFCCs averaged over all frames. Returns n_mfcc coefficients."""
    if len(samples) < frame_size:
        return [0.0] * n_mfcc
    frames = _frame(samples, frame_size, hop_size)
    window = np.hanning(frame_size)
    fbank = _mel_filterbank(frame_size, sr, n_mels)
    mfccs_sum = np.zeros(n_mfcc)
    count = 0
    for f_idx in range(frames.shape[0]):
        seg = frames[f_idx] * window
        power_spec = np.abs(np.fft.rfft(seg)) ** 2
        power_spec = np.maximum(power_spec, 1e-10)
        mel_energy = np.dot(fbank, power_spec)
        mel_energy = np.maximum(mel_energy, 1e-10)
        log_mel = np.log(mel_energy)
        mfcc = _dct(log_mel, norm='ortho')[:n_mfcc]
        mfccs_sum += mfcc
        count += 1
    if count == 0:
        return [0.0] * n_mfcc
    avg = (mfccs_sum / count).tolist()
    return avg


def compute_chroma(samples: np.ndarray, sr: int = SAMPLE_RATE,
                   frame_size: int = 4096, hop_size: int = 2048) -> list[float]:
    """Compute 12-bin chromagram averaged over all frames. Returns 12 normalized bins (C, C#, D, ..., B)."""
    if len(samples) < frame_size:
        return [0.0] * 12
    frames = _frame(samples, frame_size, hop_size)
    window = np.hanning(frame_size)
    chroma_bins = np.zeros(12)
    count = 0
    for f_idx in range(frames.shape[0]):
        seg = frames[f_idx] * window
        spec = np.abs(np.fft.rfft(seg))
        freqs = np.fft.rfftfreq(frame_size, 1.0 / sr)
        for k in range(len(spec)):
            if freqs[k] < 40.0 or freqs[k] > sr / 2:
                continue
            midi = 12.0 * math.log2(freqs[k] / 440.0) + 69.0
            if midi < 0:
                continue
            bin_idx = int(round(midi)) % 12
            chroma_bins[bin_idx] += spec[k]
        count += 1
    if count == 0 or np.sum(chroma_bins) == 0:
        return [0.0] * 12
    chroma_norm = chroma_bins / np.sum(chroma_bins)
    return chroma_norm.tolist()


def compute_hpr(samples: np.ndarray, sr: int = SAMPLE_RATE,
                frame_size: int = 1024, hop_size: int = 512,
                kernel_size: int = 11) -> float:
    """Harmonic/Percussive ratio. Returns harmonic_energy / total_energy."""
    if len(samples) < frame_size:
        return 0.5
    frames = _frame(samples, frame_size, hop_size)
    window = np.hanning(frame_size)
    n_freq = frame_size // 2 + 1
    n_frames = frames.shape[0]
    specgram = np.zeros((n_freq, n_frames))
    for f_idx in range(n_frames):
        seg = frames[f_idx] * window
        specgram[:, f_idx] = np.abs(np.fft.rfft(seg))
    specgram = np.maximum(specgram, 1e-10)
    harm = sp_signal.medfilt2d(specgram, kernel_size=(1, kernel_size))
    perc = sp_signal.medfilt2d(specgram, kernel_size=(kernel_size, 1))
    harm_energy = np.sum(harm ** 2)
    total_energy = np.sum(perc ** 2) + harm_energy
    if total_energy < 1e-10:
        return 0.5
    return float(harm_energy / total_energy)


NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]


def hz_to_midi(freq: float) -> float:
    """Convert frequency in Hz to MIDI note number (69 = A4 = 440Hz)."""
    if freq <= 0:
        return 0.0
    return 12.0 * math.log2(freq / 440.0) + 69.0


def midi_to_note(midi: float) -> str:
    """Convert MIDI note number to note name + octave (e.g., 69.0 -> 'A4')."""
    note_idx = int(round(midi)) % 12
    octave = int(round(midi)) // 12 - 1
    cents = int((midi - round(midi)) * 100)
    base = f"{NOTE_NAMES[note_idx]}{octave}"
    if abs(cents) >= 5:
        return f"{base} ({cents:+d}¢)"
    return base


def hz_to_note(freq: float) -> str:
    """Convert frequency in Hz to nearest note name + octave (e.g., 440.0 -> 'A4')."""
    if freq <= 0:
        return "---"
    return midi_to_note(hz_to_midi(freq))


def estimate_key(pitches_hz: list[float], confidence_threshold: float = 0.3) -> tuple[str, float]:
    """Estimate the most likely musical key from a list of pitch (Hz) readings.
    Returns (key_name, confidence) where key_name is like 'C major' or 'A minor'.
    Uses Krumhansl-Schmuckler key-finding algorithm (simplified).
    """
    if not pitches_hz:
        return "unknown", 0.0

    # Convert to MIDI pitch classes (0-11)
    pitch_classes = []
    for p in pitches_hz:
        if p <= 0:
            continue
        midi = hz_to_midi(p)
        pc = int(round(midi)) % 12
        pitch_classes.append(pc)

    if not pitch_classes:
        return "unknown", 0.0

    # Profile vectors for major and minor keys (Krumhansl-Schmuckler)
    major_profile = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88]
    minor_profile = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17]

    # Count occurrences of each pitch class
    chroma = np.zeros(12)
    for pc in pitch_classes:
        chroma[pc] += 1

    best_key = None
    best_corr = -999.0

    for tonic in range(12):
        # Rotate profiles to this tonic
        rotated_major = major_profile[tonic:] + major_profile[:tonic]
        rotated_minor = minor_profile[tonic:] + minor_profile[:tonic]

        corr_major = float(np.corrcoef(chroma, rotated_major)[0, 1])
        corr_minor = float(np.corrcoef(chroma, rotated_minor)[0, 1])

        if corr_major > best_corr:
            best_corr = corr_major
            best_key = f"{NOTE_NAMES[tonic]} major"
        if corr_minor > best_corr:
            best_corr = corr_minor
            best_key = f"{NOTE_NAMES[tonic]} minor"

    confidence = float(np.clip((best_corr + 1.0) / 2.0, 0.0, 1.0))
    return best_key or "unknown", confidence


def detect_pitch_full(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Full pitch detection: Hz, MIDI note, note name, confidence, key estimate."""
    pitch_hz, confidence = compute_pitch_confidence(samples, sr)
    midi = hz_to_midi(pitch_hz) if pitch_hz > 0 else 0.0
    note = midi_to_note(midi) if midi > 0 else "---"
    return {
        "pitch_hz": pitch_hz,
        "midi_note": round(midi, 1),
        "note_name": note,
        "confidence": confidence,
    }


def compute_pitch_confidence(samples: np.ndarray, sr: int = SAMPLE_RATE,
                             fmin: float = 30.0, fmax: float = 2000.0) -> tuple[float, float]:
    """Estimate pitch and confidence using normalized autocorrelation.
    Returns (pitch_hz, confidence) where confidence is 0-1 normalized peak correlation."""
    if len(samples) < 100:
        return 0.0, 0.0
    env = np.abs(samples)
    peak = np.max(env)
    if peak < 0.001:
        return 0.0, 0.0
    n = len(samples)
    lag_min = max(int(sr / fmax), 2)
    lag_max = min(int(sr / fmin), n // 2)
    if lag_max <= lag_min:
        return 0.0, 0.0
    samples = samples / peak
    n_fft = 2 ** int(math.ceil(math.log2(n)))
    spectrum = np.fft.rfft(samples, n=n_fft)
    power = np.abs(spectrum) ** 2
    autocorr = np.fft.irfft(power, n=n_fft)[:n]
    autocorr[:lag_min] = 0
    autocorr[lag_max:] = 0
    autocorr[0] = 0
    max_val = np.max(autocorr[lag_min:lag_max])
    if max_val < 1e-6:
        return 0.0, 0.0
    peak_lag = np.argmax(autocorr[lag_min:lag_max]) + lag_min
    pitch = sr / peak_lag
    energy = np.sum(samples ** 2)
    normalized = max_val / max(energy, 1e-10)
    confidence = float(np.clip(np.sqrt(normalized), 0.0, 1.0))
    return float(pitch), confidence


def compute_spectral_bandwidth(samples: np.ndarray, sr: int = SAMPLE_RATE) -> float:
    """Spectral bandwidth: weighted standard deviation around centroid."""
    n = min(len(samples), 2048)
    if n < 4:
        return 0.0
    freqs = np.fft.rfftfreq(n, 1/sr)
    spectrum = np.abs(np.fft.rfft(samples[:n]))
    total = np.sum(spectrum)
    if total == 0:
        return 0.0
    centroid = np.sum(freqs * spectrum) / total
    bandwidth = np.sqrt(np.sum(((freqs - centroid) ** 2) * spectrum) / total)
    return float(bandwidth)


def compute_early_rms(samples: np.ndarray, sr: int = SAMPLE_RATE, window_ms: float = 50.0) -> float:
    """Early RMS: RMS in first window_ms as ratio of total RMS. Higher = punchier attack."""
    if len(samples) < 100:
        return 0.0
    window_n = min(int(window_ms * sr / 1000.0), len(samples))
    early = np.sqrt(np.mean(samples[:window_n] ** 2))
    total = np.sqrt(np.mean(samples ** 2))
    if total < 1e-10:
        return 0.0
    return float(early / total)


def compute_stereo_correlation(samples: np.ndarray) -> float:
    """Stereo correlation between channels. 1.0 = identical, 0.0 = uncorrelated, -1.0 = opposite."""
    if samples.ndim != 2 or samples.shape[1] < 2:
        return 1.0
    L = samples[:, 0]
    R = samples[:, 1]
    if np.std(L) < 1e-10 or np.std(R) < 1e-10:
        return 1.0
    corr = np.corrcoef(L, R)[0, 1]
    return float(corr)


def compute_noise_floor_estimate(samples: np.ndarray, percentile: float = 10.0) -> float:
    """Estimate noise floor as a percentile of the amplitude envelope."""
    if len(samples) < 100:
        return 0.0
    env = np.abs(samples)
    return float(np.percentile(env, percentile))


def compute_features(samples: np.ndarray, sr: int = SAMPLE_RATE) -> dict:
    """Compute comprehensive feature set for a sample."""
    duration_ms = len(samples) / sr * 1000.0
    rms = compute_rms(samples)
    peak = compute_peak(samples)
    zcr = compute_zcr(samples)
    centroid = compute_spectral_centroid(samples, sr)
    rolloff = compute_spectral_rolloff(samples, 0.85, sr)
    bands = compute_band_energies(samples, sr)
    trans_strength, trans_count, onsets = detect_transients(samples, sr)
    decay_len = compute_decay_length(samples, sr)
    attack_ms = compute_attack_time(samples, sr)
    amp_peaks = count_amplitude_peaks(samples, sr)

    # New features
    flux_mean, flux_std, flux_max = compute_spectral_flux(samples, sr)
    hpr = compute_hpr(samples, sr)
    pitch_hz, pitch_conf = compute_pitch_confidence(samples, sr)
    mfccs = compute_mfccs(samples, sr)
    chroma = compute_chroma(samples, sr)

    bandwidth = compute_spectral_bandwidth(samples, sr)
    early_rms = compute_early_rms(samples, sr)
    noise_floor = compute_noise_floor_estimate(samples)
    stereo_corr = compute_stereo_correlation(samples) if samples.ndim == 2 else 1.0

    feats = {
        "duration_ms": duration_ms,
        "rms": rms,
        "peak": peak,
        "zero_crossing_rate": zcr,
        "spectral_centroid": centroid,
        "spectral_bandwidth": bandwidth,
        "spectral_rolloff": rolloff,
        "low_band_energy": bands["low"],
        "mid_band_energy": bands["mid"],
        "high_band_energy": bands["high"],
        "early_rms": early_rms,
        "noise_floor": noise_floor,
        "stereo_correlation": stereo_corr,
        "transient_count": trans_count,
        "amplitude_peaks": amp_peaks,
        "transient_strength": trans_strength,
        "decay_length_ms": decay_len,
        "attack_ms": attack_ms,
        "num_samples": len(samples),
    }

    feats["spectral_flux_mean"] = flux_mean
    feats["spectral_flux_std"] = flux_std
    feats["hpr"] = hpr
    feats["pitch_hz"] = pitch_hz
    feats["pitch_confidence"] = pitch_conf
    for i, val in enumerate(mfccs):
        feats[f"mfcc_{i+1}"] = val
    for i, val in enumerate(chroma):
        feats[f"chroma_{i}"] = val

    return feats
