"""Blend mode: mix two audio samples together with feature blending."""
import json
import sys
from pathlib import Path

import numpy as np

from gen import SAMPLE_RATE
from gen.io import read_wav, write_wav
from gen.prompt import _write_metadata, parse_prompt


def _blend_samples(a: np.ndarray, b: np.ndarray, blend: float = 0.5) -> np.ndarray:
    """Blend two samples. blend=0 → only a, blend=1 → only b."""
    max_len = max(len(a), len(b))
    result = np.zeros(max_len)
    result[:len(a)] += a * (1.0 - blend)
    result[:len(b)] += b * blend
    peak = np.max(np.abs(result))
    if peak > 0.95:
        result = result * (0.95 / peak)
    return result.astype(np.float32)


def _envelope_blend(a: np.ndarray, b: np.ndarray, blend: float = 0.5) -> np.ndarray:
    """Blend envelopes: take attack from a, body/tail from b."""
    max_len = max(len(a), len(b))
    result = np.zeros(max_len)
    split = min(len(a), len(b)) // 4
    result[:split] = a[:split]  # Attack from a
    result[split:] = b[split:]  # Body/tail from b
    # Crossfade at split point
    fade_len = min(100, split)
    for i in range(fade_len):
        t = i / fade_len
        idx = split - fade_len + i
        if 0 <= idx < max_len:
            val_a = a[idx] if idx < len(a) else 0
            val_b = b[idx] if idx < len(b) else 0
            result[idx] = val_a * (1 - t) + val_b * t
    peak = np.max(np.abs(result))
    if peak > 0.95:
        result = result * (0.95 / peak)
    return result.astype(np.float32)


def cmd_blend(args):
    """Blend two audio samples together."""
    path_a = Path(args.sample_a)
    path_b = Path(args.sample_b)

    if not path_a.exists():
        print(f"Error: {path_a} not found", file=sys.stderr)
        sys.exit(1)
    if not path_b.exists():
        print(f"Error: {path_b} not found", file=sys.stderr)
        sys.exit(1)

    result_a = read_wav(path_a)
    result_b = read_wav(path_b)
    if result_a is None or result_b is None:
        print("Error: could not read one or both files", file=sys.stderr)
        sys.exit(1)

    samples_a, _ = result_a
    samples_b, _ = result_b
    blend = args.blend
    mode = args.mode
    out_path = Path(args.out) if args.out else Path("outputs/blend") / f"blend_{path_a.stem}_{path_b.stem}.wav"
    out_path.parent.mkdir(parents=True, exist_ok=True)

    import time
    import random

    if mode == "envelope":
        result = _envelope_blend(samples_a, samples_b, blend)
    else:
        result = _blend_samples(samples_a, samples_b, blend)

    write_wav(out_path, result)

    print(f"Blend: {path_a.name} + {path_b.name}")
    print(f"  Mode: {mode}")
    print(f"  Blend: {blend:.2f} (0=only A, 1=only B)")
    print(f"  Duration A: {len(samples_a)/SAMPLE_RATE:.3f}s")
    print(f"  Duration B: {len(samples_b)/SAMPLE_RATE:.3f}s")
    print(f"  Result:    {len(result)/SAMPLE_RATE:.3f}s")
    print(f"  Output: {out_path}")
