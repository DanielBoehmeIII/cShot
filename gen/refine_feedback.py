"""Natural feedback refinement: map phrases like 'less harsh, more warm' to DSP changes."""
import json
import shutil
import sys
import time
from pathlib import Path

import numpy as np

from gen import SAMPLE_RATE
from gen.io import read_wav, write_wav
from gen.prompt import parse_prompt, generate_single_from_prompt, _seed_from_prompt
from gen.polish import polish_file


FEEDBACK_MAP = {
    "less harsh": {"brightness": -0.3, "high_shelf_db": -4.0},
    "more harsh": {"brightness": 0.3, "high_shelf_db": 4.0},
    "less bright": {"brightness": -0.3, "high_shelf_db": -4.0},
    "more bright": {"brightness": 0.3, "high_shelf_db": 4.0},
    "darker": {"brightness": -0.3, "high_shelf_db": -4.0, "filter_cutoff_end": -0.15},
    "brighter": {"brightness": 0.3, "high_shelf_db": 4.0, "filter_cutoff_end": 0.15},
    "more warm": {"brightness": -0.2, "low_shelf_db": 3.0, "high_shelf_db": -2.0},
    "less warm": {"brightness": 0.2, "low_shelf_db": -3.0, "high_shelf_db": 2.0},
    "softer": {"attack_ms": 10.0, "velocity": -0.1, "saturation": -0.1},
    "harder": {"attack_ms": -2.0, "velocity": 0.1, "saturation": 0.1},
    "more punchy": {"attack_ms": -2.0, "saturation": 0.2, "compression": 0.2, "transient_boost": 0.3},
    "less punchy": {"attack_ms": 5.0, "saturation": -0.2, "compression": -0.2},
    "shorter": {"duration_scale": 0.6, "decay_rate": 1.3},
    "longer": {"duration_scale": 1.5, "decay_rate": 0.7},
    "shorter tail": {"duration_scale": 0.6, "decay_rate": 1.5},
    "longer tail": {"duration_scale": 1.4, "decay_rate": 0.6},
    "cleaner": {"saturation": -0.3, "noise_floor": -0.01, "distortion": -0.2, "lo_fi": -0.2},
    "dirtier": {"saturation": 0.3, "noise_floor": 0.01, "distortion": 0.2},
    "more air": {"brightness": 0.2, "high_shelf_db": 3.0},
    "less air": {"brightness": -0.2, "high_shelf_db": -3.0},
    "more body": {"low_shelf_db": 4.0, "brightness": -0.1},
    "less body": {"low_shelf_db": -4.0, "brightness": 0.1},
    "more sub": {"low_shelf_db": 6.0, "sub_boost": 0.3},
    "less sub": {"low_shelf_db": -6.0, "sub_boost": -0.3},
    "more distortion": {"saturation": 0.3, "distortion": 0.3, "drive": 0.3},
    "less distortion": {"saturation": -0.3, "distortion": -0.3, "drive": -0.3},
    "narrower": {"stereo_width": -0.3},
    "wider": {"stereo_width": 0.3},
    "more analog": {"saturation": 0.1, "noise_floor": 0.005, "lo_fi": 0.1},
    "more digital": {"saturation": -0.1, "noise_floor": -0.005, "lo_fi": -0.1},
    "more texture": {"noise_floor": 0.01, "saturation": 0.15},
    "smoother": {"attack_ms": 5.0, "saturation": -0.1, "distortion": -0.1},
}


def _parse_feedback(feedback: str) -> dict:
    """Parse a feedback string like 'less harsh, more warm, shorter tail' into overrides."""
    feedback_lower = feedback.lower()
    overrides = {}
    found = []
    for phrase, mapping in FEEDBACK_MAP.items():
        if phrase in feedback_lower:
            overrides.update(mapping)
            found.append(phrase)
    return overrides, found


def cmd_refine_feedback(args):
    """Refine a generated file using natural language feedback."""
    file_path = Path(args.file)
    if not file_path.exists():
        print(f"Error: {file_path} not found", file=sys.stderr)
        sys.exit(1)

    feedback = args.feedback
    out_path = Path(args.out) if args.out else file_path.parent / f"{file_path.stem}_refined.wav"

    # Look for metadata sidecar
    meta_path = file_path.with_suffix(".json")
    if meta_path.exists():
        with open(meta_path) as f:
            meta = json.load(f)
        prompt = meta.get("prompt", "")
        seed = meta.get("seed")
        print(f"Found metadata: prompt='{prompt}', seed={seed}")
    else:
        prompt = file_path.stem.replace("_", " ")
        seed = None
        print(f"No metadata found, using filename as prompt: '{prompt}'")

    parsed = parse_prompt(prompt)
    overrides, found = _parse_feedback(feedback)

    if not found:
        print(f"Warning: no feedback phrases recognized. Available:")
        for phrase in sorted(FEEDBACK_MAP.keys()):
            print(f"  - {phrase}")
        sys.exit(1)

    print(f"Feedback: '{feedback}'")
    print(f"  → Recognized: {found}")
    print(f"  → Overrides: {json.dumps(overrides, indent=2)}")

    # Merge feedback overrides into parsed overrides
    for k, v in overrides.items():
        existing = parsed["overrides"].get(k, 0)
        if isinstance(v, (int, float)) and isinstance(existing, (int, float)):
            if k in ["duration_scale", "decay_rate"]:
                parsed["overrides"][k] = v
            else:
                parsed["overrides"][k] = existing + v
        else:
            parsed["overrides"][k] = v

    # Generate refined version
    use_seed = seed if seed else _seed_from_prompt(prompt)
    np.random.seed(use_seed % 2**32)

    samples = generate_single_from_prompt(parsed, seed=use_seed)
    write_wav(out_path, samples)

    # Save refinement metadata
    refinement_meta = {
        "source_file": str(file_path),
        "feedback": feedback,
        "recognized_phrases": found,
        "applied_overrides": overrides,
        "output": str(out_path),
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    ref_meta_path = out_path.with_suffix(".refinement.json")
    with open(ref_meta_path, "w") as f:
        json.dump(refinement_meta, f, indent=2)

    print(f"\nRefined → {out_path}")
    print(f"Refinement metadata → {ref_meta_path}")

    # Compare before/after features
    result_before = read_wav(file_path)
    result_after = read_wav(out_path)
    if result_before and result_after:
        from gen.features import compute_features
        before_feats = compute_features(result_before[0], result_before[1])
        after_feats = compute_features(result_after[0], result_after[1])
        print(f"\nBefore vs After:")
        for key in ["spectral_centroid", "attack_ms", "rms", "peak", "hpr"]:
            b = before_feats.get(key, 0)
            a = after_feats.get(key, 0)
            delta = a - b
            arrow = "↑" if delta > 0 else "↓" if delta < 0 else "="
            print(f"  {key:20s}: {b:>10.4f} → {a:>10.4f} {arrow}")
