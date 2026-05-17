"""Genre profiles: genre-specific parameter defaults for generation."""
import json
import sys
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.prompt import parse_prompt, generate_from_prompt, generate_single_from_prompt, _resolve_generator, _generate_variation, _seed_from_prompt, _write_metadata
from gen.io import write_wav
import numpy as np
import random


GENRE_PROFILES = {
    "trap": {
        "label": "Trap",
        "default_adjectives": ["punchy", "aggressive", "dark"],
        "family_biases": {"drums": 0.4, "bass": 0.3, "synth": 0.2, "fx": 0.1},
        "overrides": {"high_shelf_db": -2.0, "saturation": 0.3, "compression": 0.5},
    },
    "drill": {
        "label": "Drill",
        "default_adjectives": ["dark", "aggressive", "edgy"],
        "family_biases": {"drums": 0.3, "bass": 0.3, "synth": 0.3, "fx": 0.1},
        "overrides": {"high_shelf_db": -3.0, "brightness": 0.5, "saturation": 0.4},
    },
    "rage": {
        "label": "Rage",
        "default_adjectives": ["aggressive", "distorted", "loud"],
        "family_biases": {"synth": 0.4, "bass": 0.3, "drums": 0.2, "fx": 0.1},
        "overrides": {"saturation": 0.7, "distortion": 0.5, "brightness": 1.3},
    },
    "ambient": {
        "label": "Ambient",
        "default_adjectives": ["soft", "airy", "warm", "sustained"],
        "family_biases": {"keys": 0.4, "synth": 0.3, "fx": 0.2, "guitar": 0.1},
        "overrides": {"attack_ms": 40.0, "sustain_level": 0.7, "release_ms": 600, "noise_mix": 0.2},
    },
    "house": {
        "label": "House",
        "default_adjectives": ["bright", "punchy", "clean"],
        "family_biases": {"drums": 0.4, "keys": 0.25, "synth": 0.25, "bass": 0.1},
        "overrides": {"brightness": 1.2, "high_shelf_db": 3.0, "saturation": 0.1},
    },
    "techno": {
        "label": "Techno",
        "default_adjectives": ["edgy", "metallic", "dry", "punchy"],
        "family_biases": {"drums": 0.4, "synth": 0.3, "fx": 0.2, "bass": 0.1},
        "overrides": {"brightness": 1.3, "saturation": 0.3, "reverb_mix": 0.0},
    },
    "hyperpop": {
        "label": "Hyperpop",
        "default_adjectives": ["bright", "distorted", "glossy", "wide"],
        "family_biases": {"synth": 0.4, "drums": 0.2, "keys": 0.2, "bass": 0.1, "fx": 0.1},
        "overrides": {"brightness": 1.6, "stereo_width": 0.8, "saturation": 0.5, "distortion": 0.3},
    },
    "rnb": {
        "label": "R&B",
        "default_adjectives": ["warm", "smooth", "soft"],
        "family_biases": {"keys": 0.3, "bass": 0.25, "drums": 0.2, "guitar": 0.15, "synth": 0.1},
        "overrides": {"brightness": 0.75, "saturation": 0.1, "attack_ms": 10.0},
    },
    "cinematic": {
        "label": "Cinematic",
        "default_adjectives": ["big", "huge", "airy", "sustained"],
        "family_biases": {"fx": 0.3, "keys": 0.25, "synth": 0.25, "guitar": 0.1, "drums": 0.1},
        "overrides": {"stereo_width": 0.9, "release_ms": 700, "brightness": 1.1},
    },
    "lo_fi": {
        "label": "Lo-fi",
        "default_adjectives": ["lo_fi", "warm", "dusty", "mellow"],
        "family_biases": {"keys": 0.35, "drums": 0.25, "guitar": 0.2, "bass": 0.1, "synth": 0.1},
        "overrides": {"lo_fi": 0.4, "noise_floor": 0.01, "bit_depth": 8, "brightness": 0.65},
    },
}

GENRE_FAMILY_NOUNS = {
    "drums": "kick",
    "bass": "bass",
    "keys": "piano",
    "synth": "synth",
    "guitar": "guitar",
    "fx": "impact",
}


def cmd_genre(args):
    """Generate sounds for a specific genre."""
    genre_name = args.genre.lower().replace("-", "_").replace(" ", "_")
    if genre_name not in GENRE_PROFILES:
        print(f"Error: unknown genre '{args.genre}'", file=sys.stderr)
        print(f"Available: {', '.join(sorted(GENRE_PROFILES.keys()))}")
        sys.exit(1)

    genre = GENRE_PROFILES[genre_name]
    count = args.count
    out_dir = Path(args.out) if args.out else Path(f"outputs/{genre_name}")
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"Genre: {genre['label']}")
    print(f"  Adjectives: {', '.join(genre['default_adjectives'])}")
    print(f"  Families: {', '.join(genre['family_biases'].keys())}")
    print(f"  Overrides: {json.dumps(genre['overrides'])}")
    print(f"\nGenerating {count} files...")

    paths = []
    for i in range(count):
        # Pick a family based on genre bias
        import random as rnd
        families = list(genre["family_biases"].keys())
        weights = list(genre["family_biases"].values())
        family = rnd.choices(families, weights=weights, k=1)[0]
        noun = GENRE_FAMILY_NOUNS.get(family, "synth")

        # Build prompt with genre adjectives + family noun
        adj = rnd.choice(genre["default_adjectives"])
        prompt = f"{adj} {noun}"
        parsed = parse_prompt(prompt)

        # Merge genre overrides on top of adjective overrides
        for k, v in genre["overrides"].items():
            if k in parsed["overrides"]:
                if isinstance(v, (int, float)) and isinstance(parsed["overrides"][k], (int, float)):
                    parsed["overrides"][k] += v
            else:
                parsed["overrides"][k] = v

        seed = _seed_from_prompt(f"{genre_name}_{prompt}_{i}", i)
        np.random.seed(seed % 2**32)

        gen_fn, default_dur, default_pitch, gen_family, profile_name, overrides = _resolve_generator(parsed)
        dur = default_dur
        pitch = default_pitch
        samples, actual_dur, actual_pitch = _generate_variation(dur, pitch, gen_family, gen_fn)

        out_path = out_dir / f"{genre_name}_{adj}_{noun}_{i+1:03d}.wav"
        write_wav(out_path, samples)
        _write_metadata(out_path, parsed, seed, actual_dur, actual_pitch)
        paths.append(out_path)

    print(f"\nGenerated {len(paths)} files → {out_dir}")
    for p in paths:
        print(f"  {p.name}")
