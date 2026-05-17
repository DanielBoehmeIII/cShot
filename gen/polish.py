"""Export quality: trim, fade, normalize, validate audio files."""
import json
import sys
from pathlib import Path

import numpy as np

from gen import SAMPLE_RATE
from gen.io import read_wav, write_wav


def trim_silence(samples: np.ndarray, threshold_db: float = -60.0,
                 min_trim_ms: float = 1.0) -> np.ndarray:
    """Trim leading and trailing silence below threshold_db.
    threshold_db: dB below peak to consider as silence.
    """
    threshold = 10 ** (threshold_db / 20.0)
    if np.max(np.abs(samples)) == 0:
        return samples
    mask = np.abs(samples) > threshold
    if not np.any(mask):
        return samples
    start = int(np.argmax(mask))
    end = int(len(mask) - np.argmax(mask[::-1]))
    min_trim_samples = int(SAMPLE_RATE * min_trim_ms / 1000)
    if end - start < min_trim_samples:
        return samples
    return samples[start:end]


def apply_fade(samples: np.ndarray, fade_in_ms: float = 3.0,
               fade_out_ms: float = 5.0) -> np.ndarray:
    """Apply short fade-in and fade-out to prevent clicks."""
    result = samples.copy()
    n_in = min(int(SAMPLE_RATE * fade_in_ms / 1000), len(result))
    n_out = min(int(SAMPLE_RATE * fade_out_ms / 1000), len(result))
    if n_in > 0:
        result[:n_in] *= np.linspace(0, 1, n_in)
    if n_out > 0:
        result[-n_out:] *= np.linspace(1, 0, n_out)
    return result


def normalize_peak(samples: np.ndarray, target_db: float = -1.0) -> np.ndarray:
    """Normalize peak amplitude to target_db (e.g., -1.0 dB, -6.0 dB, -3.0 dB).
    0 dB = full scale. -1 dB = ~0.891, -3 dB = ~0.708, -6 dB = ~0.501.
    """
    target_amplitude = 10 ** (target_db / 20.0)
    peak = np.max(np.abs(samples))
    if peak <= 0:
        return samples
    gain = target_amplitude / peak
    return samples * gain


def validate_audio(samples: np.ndarray) -> dict:
    """Check for issues: clipping, silence, NaN, infinite values.
    Returns dict with pass/fail + metrics.
    """
    issues = []
    peak = float(np.max(np.abs(samples)))
    rms = float(np.sqrt(np.mean(samples ** 2)))

    if np.any(np.isnan(samples)):
        issues.append("contains NaN values")
    if np.any(np.isinf(samples)):
        issues.append("contains infinite values")
    if peak > 0.999:
        issues.append(f"clipping (peak={peak:.4f})")
    if rms < 0.0001:
        issues.append(f"silence/near-silence (RMS={rms:.6f})")
    if len(samples) < 100:
        issues.append(f"too short ({len(samples)} samples)")

    return {
        "pass": len(issues) == 0,
        "issues": issues,
        "peak": round(peak, 6),
        "rms": round(rms, 6),
        "duration_s": round(len(samples) / SAMPLE_RATE, 3),
    }


def polish_file(wav_path: Path, target_db: float = -1.0,
                trim_db: float = -60.0, fade_ms: tuple = (3.0, 5.0),
                in_place: bool = True) -> dict:
    """Polish a single WAV file: validate → trim → fade → normalize.
    Returns validation result dict.
    """
    result = read_wav(wav_path)
    if result is None:
        return {"pass": False, "issues": ["could not read file"]}
    samples, sr = result

    original = samples.copy()
    samples = trim_silence(samples, trim_db)
    samples = apply_fade(samples, fade_ms[0], fade_ms[1])
    samples = normalize_peak(samples, target_db)
    validation = validate_audio(samples)

    if not in_place:
        stem = wav_path.stem
        out_path = wav_path.parent / f"{stem}_polished.wav"
    else:
        out_path = wav_path

    write_wav(out_path, samples)
    validation["output"] = str(out_path)
    return validation


def cmd_polish(args):
    """Polish a directory of WAV files or a single file."""
    input_path = Path(args.input)
    if not input_path.exists():
        print(f"Error: {input_path} not found", file=sys.stderr)
        sys.exit(1)

    target_db = args.target_db
    trim_db = args.trim_db
    fade_in = args.fade_in_ms
    fade_out = args.fade_out_ms

    if input_path.is_file():
        files = [input_path]
    else:
        files = sorted(input_path.rglob("*.wav"))

    if not files:
        print(f"No .wav files found in {input_path}")
        return

    results = []
    print(f"Polishing {len(files)} file(s) (target={target_db}dB, trim={trim_db}dB, fade={fade_in}/{fade_out}ms)")
    print(f"{'='*60}")
    passed = 0
    failed = 0
    for w in files:
        validation = polish_file(w, target_db=target_db, trim_db=trim_db,
                                 fade_ms=(fade_in, fade_out))
        status = "PASS" if validation["pass"] else "FAIL"
        if validation["pass"]:
            passed += 1
        else:
            failed += 1
        issues = "; ".join(validation["issues"]) if validation["issues"] else "ok"
        print(f"  {status:4s} {w.name:35s} peak={validation['peak']:.4f} "
              f"rms={validation['rms']:.4f} dur={validation['duration_s']:.3f}s  [{issues}]")
        results.append({
            "file": w.name,
            "pass": validation["pass"],
            "issues": validation["issues"],
            "peak": validation["peak"],
            "rms": validation["rms"],
            "duration_s": validation["duration_s"],
        })

    report = {
        "total": len(files),
        "passed": passed,
        "failed": failed,
        "target_db": target_db,
        "trim_db": trim_db,
        "results": results,
    }
    report_path = input_path / "polish_report.json" if input_path.is_dir() else input_path.parent / "polish_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"\nReport: {report_path}")
    print(f"  {passed}/{len(files)} passed, {failed} failed")
