"""Preset saving and loading for reusable generation configurations."""
import json
import sys
import time
from pathlib import Path

from gen import REPO_ROOT

PRESETS_DIR = REPO_ROOT / "presets"


def _family_dir(family: str) -> Path:
    d = PRESETS_DIR / family
    d.mkdir(parents=True, exist_ok=True)
    return d


def _list_presets() -> dict[str, list[dict]]:
    """Return {family: [{name, path, prompt, ...}]}."""
    if not PRESETS_DIR.exists():
        return {}
    result = {}
    for family_dir in sorted(PRESETS_DIR.iterdir()):
        if not family_dir.is_dir():
            continue
        family = family_dir.name
        result[family] = []
        for preset_file in sorted(family_dir.glob("*.json")):
            with open(preset_file) as f:
                data = json.load(f)
            data["path"] = str(preset_file)
            result[family].append(data)
    return result


def cmd_save_preset(args):
    name = args.name
    meta_path = Path(args.from_meta)
    if not meta_path.exists():
        print(f"Error: {meta_path} not found", file=sys.stderr)
        sys.exit(1)

    with open(meta_path) as f:
        meta = json.load(f)

    family = meta.get("family", "unknown")
    family_subdir = _family_dir(family)
    preset_path = family_subdir / f"{name}.json"

    preset = {
        "name": name,
        "family": family,
        "profile": meta.get("profile", ""),
        "prompt": meta.get("prompt", ""),
        "adjectives": meta.get("adjectives", []),
        "overrides": meta.get("overrides", {}),
        "seed": meta.get("seed"),
        "source_file": meta.get("file", ""),
        "saved_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }

    with open(preset_path, "w") as f:
        json.dump(preset, f, indent=2)

    print(f"Saved preset '{name}' ({family}) → {preset_path}")


def cmd_preset_list(args):
    presets = _list_presets()
    if not presets:
        print("No presets saved yet.")
        return

    total = sum(len(v) for v in presets.values())
    print(f"Presets ({total} total):")
    print(f"{'='*50}")
    for family, family_presets in sorted(presets.items()):
        print(f"\n[{family}]")
        for p in family_presets:
            seed_str = f" (seed={p['seed']})" if p.get("seed") else ""
            print(f"  {p['name']:25s} → {p.get('prompt', '?')}{seed_str}")


def cmd_preset_generate(args):
    name = args.name
    presets = _list_presets()
    found = None
    for family, family_presets in presets.items():
        for p in family_presets:
            if p["name"] == name:
                found = p
                break
        if found:
            break

    if not found:
        print(f"Error: preset '{name}' not found", file=sys.stderr)
        print("Use 'cshot preset list' to see available presets.")
        sys.exit(1)

    out_dir = Path(args.out) if args.out else PRESETS_DIR / "generated" / found["family"]
    out_dir.mkdir(parents=True, exist_ok=True)

    from gen.prompt import parse_prompt, generate_from_prompt, generate_single_from_prompt
    from gen.io import write_wav
    import random, numpy as np

    prompt = found["prompt"]
    parsed = parse_prompt(prompt)

    count = args.count
    base_seed = found.get("seed")

    if count == 1:
        out_path = out_dir / f"preset_{name}.wav"
        out_path.parent.mkdir(parents=True, exist_ok=True)
        from gen.prompt import _resolve_generator, _generate_variation, _seed_from_prompt, _write_metadata
        seed = base_seed if base_seed is not None else _seed_from_prompt(prompt)
        random.seed(seed)
        np.random.seed(seed % 2**32)
        gen_fn, default_dur, default_pitch, family, _, overrides = _resolve_generator(parsed)
        duration_scale = overrides.pop("duration_scale", 1.0)
        pitch_scale = overrides.pop("pitch_scale", 1.0)
        dur = default_dur * duration_scale
        pitch = default_pitch * pitch_scale
        samples, actual_dur, actual_pitch = _generate_variation(dur, pitch, family, gen_fn)
        write_wav(out_path, samples)
        _write_metadata(out_path, parsed, seed, actual_dur, actual_pitch)
        print(f"Generated '{name}' → {out_path} (seed={seed})")
    else:
        paths = generate_from_prompt(parsed, count, out_dir, base_seed=base_seed)
        print(f"Generated {len(paths)} file(s) from preset '{name}' → {out_dir}")
        for p in paths:
            print(f"  {p.name}")
