import sys
from pathlib import Path
from typing import Optional
import numpy as np
from gen import SAMPLE_RATE


def read_wav(path: Path, mono: bool = True) -> tuple[np.ndarray, int]:
    """Read WAV file, return (samples_normed, sample_rate).
    If mono=True (default), stereo is collapsed to mono.
    If mono=False, stereo is preserved as (N, 2)."""
    import soundfile as sf
    data, sr = sf.read(str(path))
    if mono and data.ndim > 1:
        data = data.mean(axis=1)
    if sr != SAMPLE_RATE:
        from scipy import signal
        ratio = SAMPLE_RATE / sr
        if data.ndim == 1:
            new_len = int(len(data) * ratio)
            data = signal.resample(data, new_len)
        else:
            new_len = int(data.shape[0] * ratio)
            new_data = np.zeros((new_len, data.shape[1]))
            for ch in range(data.shape[1]):
                new_data[:, ch] = signal.resample(data[:, ch], new_len)
            data = new_data
        sr = SAMPLE_RATE
    if data.ndim == 1:
        peak = np.max(np.abs(data))
        if peak > 0:
            data = data / peak * 0.95
    else:
        for ch in range(data.shape[1]):
            peak = np.max(np.abs(data[:, ch]))
            if peak > 0:
                data[:, ch] = data[:, ch] / peak * 0.95
    return data.astype(np.float32), sr


def read_aiff(path: Path) -> tuple[np.ndarray, int]:
    """Read AIFF file by converting through soundfile."""
    import soundfile as sf
    data, sr = sf.read(str(path))
    if data.ndim > 1:
        data = data.mean(axis=1)
    if sr != SAMPLE_RATE:
        from scipy import signal
        ratio = SAMPLE_RATE / sr
        new_len = int(len(data) * ratio)
        data = signal.resample(data, new_len)
        sr = SAMPLE_RATE
    peak = np.max(np.abs(data))
    if peak > 0:
        data = data / peak * 0.95
    return data.astype(np.float32), sr


def write_wav(path: Path, samples: np.ndarray, sr: int = SAMPLE_RATE):
    """Write WAV file using soundfile."""
    import soundfile as sf
    sf.write(str(path), samples, sr, subtype='PCM_16')


def _read_with_ffmpeg(path: Path, mono: bool = True) -> Optional[tuple[np.ndarray, int]]:
    """Read audio via ffmpeg conversion for unsupported formats (e.g. .wv, malformed .wav)."""
    import subprocess
    import tempfile
    import soundfile as sf
    try:
        with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as tmp:
            tmp_path = Path(tmp.name)
        ac = '1' if mono else '2'
        result = subprocess.run(
            ['ffmpeg', '-y', '-i', str(path), '-ac', ac, '-ar', str(SAMPLE_RATE),
             '-f', 'wav', str(tmp_path)],
            capture_output=True, timeout=30
        )
        if result.returncode != 0:
            tmp_path.unlink(missing_ok=True)
            return None
        data, sr = sf.read(str(tmp_path))
        tmp_path.unlink(missing_ok=True)
        if mono and data.ndim > 1:
            data = data.mean(axis=1)
        peak = np.max(np.abs(data))
        if peak > 0:
            data = data / peak * 0.95
        return data.astype(np.float32), sr
    except Exception:
        return None


def read_audio_safe(path: Path, mono: bool = True) -> Optional[tuple[np.ndarray, int]]:
    """Read any supported audio file safely. Returns mono by default.
    Falls back to ffmpeg for unsupported/corrupted files."""
    import soundfile as sf
    ext = path.suffix.lower()
    try:
        if ext in ('.mp3',):
            return None
        data, sr = sf.read(str(path))
        if mono and data.ndim > 1:
            data = data.mean(axis=1)
        if sr != SAMPLE_RATE:
            from scipy import signal
            ratio = SAMPLE_RATE / sr
            if data.ndim == 1:
                new_len = int(len(data) * ratio)
                data = signal.resample(data, new_len)
            else:
                new_len = int(data.shape[0] * ratio)
                new_data = np.zeros((new_len, data.shape[1]))
                for ch in range(data.shape[1]):
                    new_data[:, ch] = signal.resample(data[:, ch], new_len)
                data = new_data
            sr = SAMPLE_RATE
        peak = np.max(np.abs(data))
        if peak > 0:
            data = data / peak * 0.95
        return data.astype(np.float32), sr
    except Exception:
        return _read_with_ffmpeg(path, mono=mono)
