import json
import math
import sys
import time
from pathlib import Path
from typing import Optional

import numpy as np

from gen import REPO_ROOT, PACKS_DIR, SPANISH_GUITAR_DIR, SUPPORTED_EXTS
from gen.io import read_audio_safe
from gen.features import compute_features


def load_profiles(path: Optional[Path] = None) -> dict:
    p = path or REPO_ROOT / "class_profiles.json"
    if not p.exists():
        print(f"Error: {p} not found. Run 'profiles' first.", file=sys.stderr)
        sys.exit(1)
    with open(p) as f:
        return json.load(f).get("profiles", {})


def find_reference_folders():
    """Auto-discover reference folders under the repo root."""
    folders = []
    if PACKS_DIR.exists():
        folders.append(("Packs", PACKS_DIR))
    if SPANISH_GUITAR_DIR.exists():
        folders.append(("Spanish Guitar", SPANISH_GUITAR_DIR))
    if PACKS_DIR.exists():
        for sub in PACKS_DIR.iterdir():
            if sub.is_dir() and not sub.name.startswith(".") and not sub.name.endswith(".zip"):
                wav_files = list(sub.rglob("*.wav")) + list(sub.rglob("*.WAV")) + list(sub.rglob("*.wv"))
                if len(wav_files) > 0:
                    folders.append((f"Packs/{sub.name}", sub))
    return folders


def classify_reference_file(path: Path) -> Optional[str]:
    """Classify a reference audio file into a sound class based on path and filename."""
    name = path.stem.lower()
    parent = str(path.parent).lower()
    grandparent = str(path.parent.parent).lower() if path.parent.parent else ""

    if "kick" in name and "snare" not in name:
        return "kick"
    if "snare" in name:
        return "snare"
    if "clap" in name:
        return "clap"
    if ("ch" in name and "hat" not in name) or "closed" in name or "ch_" in name.lower():
        if "open" not in name:
            return "closed_hat"
    if ("oh" in name and "hat" not in name) or "open" in name:
        return "open_hat"
    if "hat" in name or "hihat" in name or "hi-hat" in name:
        if "open" in name:
            return "open_hat"
        return "closed_hat"
    if "808" in name or "sub" in name:
        return "808"
    if "bass" in name:
        return "bass_stab"
    if "fx" in name or "impact" in name or "boom" in name or "crash" in name:
        return "impact_fx"
    if "synth" in name or "stab" in name:
        return "synth_stab"
    if "guitar" in name or "spanish" in name or "chord" in name:
        return "guitar_stab"

    if "kick" in parent or "kick" in grandparent:
        return "kick"
    if "snare" in parent or "snare" in grandparent:
        return "snare"
    if "clap" in parent or "clap" in grandparent:
        return "clap"
    if "hat" in parent or "hat" in grandparent or "hi-hat" in parent or "hi-hat" in grandparent:
        if "open" in parent or "open" in grandparent:
            return "open_hat"
        return "closed_hat"
    if "808" in parent or "sub" in parent:
        return "808"
    if "bass" in parent:
        return "bass_stab"
    if "fx" in parent or "impact" in parent:
        return "impact_fx"
    if "synth" in parent:
        return "synth_stab"
    if "guitar" in parent or "guitar" in grandparent:
        return "guitar_stab"
    if "cymbal" in parent or "crash" in parent:
        return "impact_fx"
    if "perc" in parent or "rim" in parent or "tom" in parent:
        return "impact_fx"

    return None


def cmd_scan(args):
    """Scan reference folders and compute per-file features."""
    output_path = Path(args.output) if args.output else REPO_ROOT / "reference_analysis.json"

    ref_folders = find_reference_folders()
    print(f"Scanning {len(ref_folders)} reference folders for audio files...")

    all_files = []
    for name, folder in ref_folders:
        for ext in SUPPORTED_EXTS:
            all_files.extend(folder.rglob(f"*{ext}"))
            all_files.extend(folder.rglob(f"*{ext.upper()}"))

    all_files = sorted(set(all_files))
    print(f"Found {len(all_files)} audio files")

    supported = [f for f in all_files if f.suffix.lower() in SUPPORTED_EXTS]
    skipped = [f for f in all_files if f.suffix.lower() not in SUPPORTED_EXTS]

    if skipped:
        print(f"Skipped {len(skipped)} files with unsupported formats (e.g. MP3)")
        for f in skipped[:10]:
            print(f"  Skipped: {f.relative_to(REPO_ROOT)}")
        if len(skipped) > 10:
            print(f"  ... and {len(skipped) - 10} more")

    results = {}
    analyzed = 0
    errors = 0

    for path in supported:
        rel_path = str(path.relative_to(REPO_ROOT))
        cls = classify_reference_file(path)
        if cls is None:
            parent_dir = path.parent.name.lower()
            if "clap" in parent_dir:
                cls = "clap"
            elif "kick" in parent_dir:
                cls = "kick"
            elif "snare" in parent_dir:
                cls = "snare"
            elif "hi-hat" in parent_dir or "hat" in parent_dir:
                cls = "closed_hat"
            elif "cymbal" in parent_dir:
                cls = "open_hat"
            else:
                cls = "unknown"

        result = read_audio_safe(path)
        if result is None:
            errors += 1
            continue

        samples, sr = result
        features = compute_features(samples, sr)
        features["file_path"] = rel_path
        features["class"] = cls

        results[rel_path] = features
        analyzed += 1

        if analyzed % 50 == 0:
            print(f"  Analyzed {analyzed}/{len(supported)}...")

    output = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_files": len(supported),
        "analyzed": analyzed,
        "skipped_format": len(skipped),
        "errors": errors,
        "sample_rate": 44100,
        "files": results,
    }

    output_path.write_text(json.dumps(output, indent=2))
    print(f"\nWrote reference analysis to {output_path}")
    print(f"  Total files: {len(supported)}")
    print(f"  Analyzed: {analyzed}")
    print(f"  Skipped: {len(skipped)}")
    print(f"  Errors: {errors}")

    class_counts = {}
    for f, feats in results.items():
        c = feats["class"]
        class_counts[c] = class_counts.get(c, 0) + 1
    print(f"\nClass distribution:")
    for c in sorted(class_counts.keys()):
        print(f"  {c}: {class_counts[c]}")


def cmd_profiles(args):
    """Build class profiles from reference analysis."""
    analysis_path = Path(args.analysis) if args.analysis else REPO_ROOT / "reference_analysis.json"
    if not analysis_path.exists():
        print(f"Error: {analysis_path} not found. Run 'scan' first.", file=sys.stderr)
        sys.exit(1)

    with open(analysis_path) as f:
        analysis = json.load(f)

    files = analysis.get("files", {})

    class_features = {}
    for rel_path, feats in files.items():
        cls = feats["class"]
        if cls == "unknown":
            continue
        if cls not in class_features:
            class_features[cls] = []
        class_features[cls].append(feats)

    profiles = {}
    for cls, feats_list in class_features.items():
        if len(feats_list) < 2:
            print(f"  Warning: class '{cls}' only has {len(feats_list)} samples, skipping profile")
            continue

        keys = ["duration_ms", "rms", "peak", "zero_crossing_rate",
                "spectral_centroid", "spectral_rolloff",
                "low_band_energy", "mid_band_energy", "high_band_energy",
                "transient_count", "decay_length_ms", "attack_ms"]

        profile = {"num_references": len(feats_list)}
        for k in keys:
            vals = [f[k] for f in feats_list if k in f]
            if vals:
                profile[k] = {
                    "mean": float(np.mean(vals)),
                    "std": float(np.std(vals)),
                    "min": float(np.min(vals)),
                    "max": float(np.max(vals)),
                    "median": float(np.median(vals)),
                }
            else:
                profile[k] = {"mean": 0.0, "std": 0.0, "min": 0.0, "max": 0.0, "median": 0.0}

        def distance_to_mean(feats, profile, keys):
            dist = 0.0
            for k in keys:
                if k in feats and k in profile:
                    mean = profile[k]["mean"]
                    std = profile[k]["std"]
                    if std > 0:
                        dist += ((feats[k] - mean) / std) ** 2
            return math.sqrt(dist)

        scored = [(distance_to_mean(f, profile, keys), f["file_path"]) for f in feats_list]
        scored.sort(key=lambda x: x[0])
        profile["representative_samples"] = [s[1] for s in scored[:5]]

        profiles[cls] = profile

    output = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "num_classes": len(profiles),
        "profiles": profiles,
    }

    output_path = Path(args.output) if args.output else REPO_ROOT / "class_profiles.json"
    output_path.write_text(json.dumps(output, indent=2))
    print(f"Wrote class profiles to {output_path}")
    print(f"Classes: {', '.join(sorted(profiles.keys()))}")
    for cls in sorted(profiles.keys()):
        p = profiles[cls]
        print(f"  {cls}: {p['num_references']} refs, centroid={p['spectral_centroid']['mean']:.0f}Hz, "
              f"low={p['low_band_energy']['mean']:.2f}, high={p['high_band_energy']['mean']:.2f}")

    return profiles
