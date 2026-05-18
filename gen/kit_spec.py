"""
WEEK 1 — KitSpec: Formal one-shot kit blueprint schema.

A KitSpec describes everything needed to generate a producer-ready one-shot kit:
name, prompt, source references, genre/style, category plan, targets, export format.

Usage:
  cshot kit-spec "dark rnb one shots" --out kit_spec.json
"""

import json
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

import numpy as np

from gen import REPO_ROOT
from gen.genre import GENRE_PROFILES


@dataclass
class CategoryCounts:
    kicks: int = 6
    snares: int = 4
    claps: int = 4
    hats: int = 8
    open_hats: int = 2
    percs: int = 4
    basses_808: int = 8
    basses_sub: int = 2
    keys: int = 5
    synths: int = 5
    guitars: int = 3
    impacts: int = 3
    risers: int = 2
    glitches: int = 2
    textures: int = 2
    atmospheres: int = 2
    fx_noise: int = 2


@dataclass
class DNAProfile:
    spectral_centroid_target: float = 2500.0
    transient_aggression: float = 0.5
    saturation_density: float = 0.3
    stereo_width: float = 0.3
    tonal_noise_ratio: float = 0.7
    loudness_lufs: float = -14.0
    brightness: float = 0.5
    darkness: float = 0.0
    grit: float = 0.2
    dryness: float = 0.7
    punch: float = 0.5
    softness: float = 0.0
    analog_warmth: float = 0.0
    width_span: float = 0.3


@dataclass
class CohesionTargets:
    spectral_cohesion: float = 0.7
    loudness_consistency: float = 0.8
    transient_consistency: float = 0.6
    stereo_consistency: float = 0.7
    category_diversity: float = 0.5
    duplicate_risk_max: float = 0.15
    source_similarity_max: float = 0.3


@dataclass
class ExportConfig:
    format: str = "wav"
    sample_rate: int = 44100
    bit_depth: int = 24
    normalize: bool = True
    normalize_target_db: float = -1.0
    trim_silence: bool = True
    trim_threshold_db: float = -60.0
    fade_in_ms: float = 3.0
    fade_out_ms: float = 5.0
    mono_bass: bool = True
    bass_split_hz: float = 120.0
    metadata_sidecar: bool = True
    manifest: bool = True
    readme: bool = True


@dataclass
class KitSpec:
    name: str = ""
    prompt: str = ""
    source_refs: list[str] = field(default_factory=list)
    genre: str = ""
    style: str = ""
    target_dna: DNAProfile = field(default_factory=DNAProfile)
    categories: CategoryCounts = field(default_factory=CategoryCounts)
    cohesion: CohesionTargets = field(default_factory=CohesionTargets)
    export: ExportConfig = field(default_factory=ExportConfig)
    total_target: int = 60
    created_at: str = ""
    version: str = "0.2.0"


def kit_spec_to_dict(spec: KitSpec) -> dict:
    d = asdict(spec)
    d["total_calculated"] = sum(v for k, v in asdict(spec.categories).items())
    return d


def kit_spec_from_dict(d: dict) -> KitSpec:
    cat = CategoryCounts(**d.get("categories", {}))
    dna = DNAProfile(**d.get("target_dna", {}))
    coh = CohesionTargets(**d.get("cohesion", {}))
    exp = ExportConfig(**d.get("export", {}))
    return KitSpec(
        name=d.get("name", ""),
        prompt=d.get("prompt", ""),
        source_refs=d.get("source_refs", []),
        genre=d.get("genre", ""),
        style=d.get("style", ""),
        target_dna=dna,
        categories=cat,
        cohesion=coh,
        export=exp,
        total_target=d.get("total_target", 60),
        created_at=d.get("created_at", ""),
        version=d.get("version", "0.2.0"),
    )


# ─── Style inference ─────────────────────────────────────

GENRE_DNA_BIAS = {
    "trap": DNAProfile(
        spectral_centroid_target=2000.0,
        transient_aggression=0.8,
        saturation_density=0.5,
        stereo_width=0.2,
        tonal_noise_ratio=0.6,
        loudness_lufs=-10.0,
        brightness=0.3,
        darkness=0.7,
        grit=0.6,
        dryness=0.8,
        punch=0.9,
        softness=0.0,
        analog_warmth=0.1,
        width_span=0.2,
    ),
    "drill": DNAProfile(
        spectral_centroid_target=1800.0,
        transient_aggression=0.7,
        saturation_density=0.5,
        stereo_width=0.3,
        tonal_noise_ratio=0.5,
        loudness_lufs=-9.0,
        brightness=0.3,
        darkness=0.8,
        grit=0.7,
        dryness=0.7,
        punch=0.8,
        softness=0.0,
        analog_warmth=0.05,
        width_span=0.25,
    ),
    "rage": DNAProfile(
        spectral_centroid_target=3500.0,
        transient_aggression=0.9,
        saturation_density=0.8,
        stereo_width=0.6,
        tonal_noise_ratio=0.4,
        loudness_lufs=-8.0,
        brightness=0.7,
        darkness=0.3,
        grit=0.9,
        dryness=0.6,
        punch=0.9,
        softness=0.0,
        analog_warmth=0.0,
        width_span=0.5,
    ),
    "rnb": DNAProfile(
        spectral_centroid_target=2200.0,
        transient_aggression=0.3,
        saturation_density=0.2,
        stereo_width=0.4,
        tonal_noise_ratio=0.8,
        loudness_lufs=-14.0,
        brightness=0.4,
        darkness=0.4,
        grit=0.1,
        dryness=0.5,
        punch=0.3,
        softness=0.6,
        analog_warmth=0.5,
        width_span=0.4,
    ),
    "ambient": DNAProfile(
        spectral_centroid_target=3000.0,
        transient_aggression=0.1,
        saturation_density=0.1,
        stereo_width=0.8,
        tonal_noise_ratio=0.5,
        loudness_lufs=-20.0,
        brightness=0.5,
        darkness=0.3,
        grit=0.0,
        dryness=0.1,
        punch=0.1,
        softness=0.9,
        analog_warmth=0.3,
        width_span=0.8,
    ),
    "house": DNAProfile(
        spectral_centroid_target=3500.0,
        transient_aggression=0.7,
        saturation_density=0.2,
        stereo_width=0.4,
        tonal_noise_ratio=0.7,
        loudness_lufs=-12.0,
        brightness=0.7,
        darkness=0.1,
        grit=0.1,
        dryness=0.8,
        punch=0.7,
        softness=0.1,
        analog_warmth=0.2,
        width_span=0.4,
    ),
    "techno": DNAProfile(
        spectral_centroid_target=4000.0,
        transient_aggression=0.8,
        saturation_density=0.4,
        stereo_width=0.3,
        tonal_noise_ratio=0.5,
        loudness_lufs=-11.0,
        brightness=0.6,
        darkness=0.3,
        grit=0.5,
        dryness=0.9,
        punch=0.8,
        softness=0.0,
        analog_warmth=0.1,
        width_span=0.3,
    ),
    "hyperpop": DNAProfile(
        spectral_centroid_target=5000.0,
        transient_aggression=0.6,
        saturation_density=0.6,
        stereo_width=0.9,
        tonal_noise_ratio=0.5,
        loudness_lufs=-10.0,
        brightness=0.9,
        darkness=0.0,
        grit=0.6,
        dryness=0.4,
        punch=0.6,
        softness=0.1,
        analog_warmth=0.0,
        width_span=0.9,
    ),
    "lo_fi": DNAProfile(
        spectral_centroid_target=2000.0,
        transient_aggression=0.2,
        saturation_density=0.1,
        stereo_width=0.2,
        tonal_noise_ratio=0.7,
        loudness_lufs=-16.0,
        brightness=0.3,
        darkness=0.4,
        grit=0.1,
        dryness=0.3,
        punch=0.2,
        softness=0.7,
        analog_warmth=0.8,
        width_span=0.2,
    ),
    "cinematic": DNAProfile(
        spectral_centroid_target=3000.0,
        transient_aggression=0.5,
        saturation_density=0.2,
        stereo_width=0.9,
        tonal_noise_ratio=0.6,
        loudness_lufs=-16.0,
        brightness=0.6,
        darkness=0.3,
        grit=0.2,
        dryness=0.2,
        punch=0.5,
        softness=0.3,
        analog_warmth=0.1,
        width_span=0.8,
    ),
}

GENRE_CATEGORY_BIAS = {
    "trap": CategoryCounts(
        kicks=8, snares=6, claps=4, hats=10, open_hats=2, percs=4,
        basses_808=10, basses_sub=2, keys=2, synths=4, guitars=0,
        impacts=2, risers=2, glitches=2, textures=2, atmospheres=0, fx_noise=2,
    ),
    "drill": CategoryCounts(
        kicks=6, snares=4, claps=4, hats=10, open_hats=2, percs=4,
        basses_808=10, basses_sub=2, keys=3, synths=6, guitars=0,
        impacts=2, risers=2, glitches=2, textures=1, atmospheres=0, fx_noise=2,
    ),
    "rage": CategoryCounts(
        kicks=6, snares=4, claps=2, hats=6, open_hats=2, percs=2,
        basses_808=6, basses_sub=2, keys=2, synths=10, guitars=0,
        impacts=4, risers=4, glitches=4, textures=2, atmospheres=2, fx_noise=2,
    ),
    "rnb": CategoryCounts(
        kicks=4, snares=2, claps=4, hats=6, open_hats=2, percs=4,
        basses_808=6, basses_sub=4, keys=8, synths=6, guitars=4,
        impacts=2, risers=2, glitches=1, textures=2, atmospheres=2, fx_noise=1,
    ),
    "ambient": CategoryCounts(
        kicks=2, snares=0, claps=0, hats=2, open_hats=1, percs=2,
        basses_808=2, basses_sub=4, keys=6, synths=10, guitars=4,
        impacts=2, risers=4, glitches=2, textures=6, atmospheres=8, fx_noise=4,
    ),
    "house": CategoryCounts(
        kicks=8, snares=4, claps=6, hats=10, open_hats=3, percs=6,
        basses_808=4, basses_sub=4, keys=6, synths=6, guitars=0,
        impacts=2, risers=2, glitches=1, textures=2, atmospheres=0, fx_noise=1,
    ),
    "techno": CategoryCounts(
        kicks=8, snares=6, claps=4, hats=8, open_hats=2, percs=6,
        basses_808=4, basses_sub=4, keys=2, synths=8, guitars=0,
        impacts=4, risers=4, glitches=4, textures=2, atmospheres=2, fx_noise=2,
    ),
    "hyperpop": CategoryCounts(
        kicks=4, snares=2, claps=2, hats=6, open_hats=2, percs=2,
        basses_808=4, basses_sub=2, keys=6, synths=12, guitars=2,
        impacts=4, risers=4, glitches=4, textures=4, atmospheres=2, fx_noise=4,
    ),
    "lo_fi": CategoryCounts(
        kicks=4, snares=2, claps=2, hats=4, open_hats=1, percs=3,
        basses_808=4, basses_sub=3, keys=8, synths=4, guitars=4,
        impacts=1, risers=1, glitches=1, textures=4, atmospheres=2, fx_noise=1,
    ),
    "cinematic": CategoryCounts(
        kicks=4, snares=2, claps=2, hats=4, open_hats=1, percs=2,
        basses_808=4, basses_sub=4, keys=6, synths=8, guitars=2,
        impacts=6, risers=6, glitches=4, textures=6, atmospheres=6, fx_noise=2,
    ),
}


def infer_spec_from_prompt(prompt: str) -> KitSpec:
    prompt_lower = prompt.lower()

    genre = _detect_genre(prompt_lower)
    dna = GENRE_DNA_BIAS.get(genre, DNAProfile())

    if genre in GENRE_CATEGORY_BIAS:
        cats = GENRE_CATEGORY_BIAS[genre]
    else:
        cats = _infer_categories_from_prompt(prompt_lower, dna)

    total = sum(v for k, v in asdict(cats).items())

    name = _make_spec_name(prompt, genre)
    style = _infer_style(prompt_lower)

    return KitSpec(
        name=name,
        prompt=prompt,
        genre=genre,
        style=style,
        target_dna=dna,
        categories=cats,
        total_target=total,
        created_at=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    )


def _detect_genre(prompt_lower: str) -> str:
    for gn in ["trap", "drill", "rage", "hyperpop", "house", "techno",
               "ambient", "cinematic", "lo_fi", "lofi"]:
        if gn in prompt_lower:
            if gn == "lofi":
                return "lo_fi"
            return gn
    if "rnb" in prompt_lower or "r&b" in prompt_lower or "randb" in prompt_lower:
        return "rnb"
    return "custom"


def _infer_style(prompt_lower: str) -> str:
    style_tags = []
    style_keywords = {
        "dark": "dark",
        "bright": "bright",
        "glossy": "glossy",
        "futuristic": "futuristic",
        "vintage": "vintage",
        "lo-fi": "lo-fi",
        "lofi": "lo-fi",
        "experimental": "experimental",
        "minimal": "minimal",
        "aggressive": "aggressive",
        "soft": "soft",
        "warm": "warm",
        "dusty": "dusty",
        "clean": "clean",
        "distorted": "distorted",
        "cinematic": "cinematic",
        "ethereal": "ethereal",
    }
    for kw, tag in style_keywords.items():
        if kw in prompt_lower:
            style_tags.append(tag)
    return ", ".join(style_tags) if style_tags else ""


def _infer_categories_from_prompt(prompt_lower: str, dna: DNAProfile) -> CategoryCounts:
    cats = CategoryCounts()

    has_drums = any(w in prompt_lower for w in ["drum", "perc", "hit", "beat"])
    has_bass = any(w in prompt_lower for w in ["bass", "808", "low", "sub"])
    has_keys = any(w in prompt_lower for w in ["keys", "piano", "melody", "chord"])
    has_synth = any(w in prompt_lower for w in ["synth", "stab", "pluck", "lead"])
    has_guitar = any(w in prompt_lower for w in ["guitar", "pluck", "string", "nylon"])
    has_fx = any(w in prompt_lower for w in ["fx", "impact", "riser", "glitch", "texture"])
    has_atmo = any(w in prompt_lower for w in ["atmo", "ambient", "pad", "texture"])

    is_bass_heavy = dna.darkness > 0.5 and dna.punch > 0.6
    is_bright = dna.brightness > 0.6
    is_wide = dna.stereo_width > 0.6

    if has_drums or is_bass_heavy:
        cats.kicks = 8
        cats.snares = 6
        cats.claps = 4
        cats.hats = 10
        cats.percs = 4
    if has_bass or is_bass_heavy:
        cats.basses_808 = 8
        cats.basses_sub = 2
    if has_keys:
        cats.keys = 8
    if has_synth or is_bright:
        cats.synths = 8
    if has_guitar:
        cats.guitars = 4
    if has_fx or is_wide:
        cats.impacts = 4
        cats.risers = 3
        cats.glitches = 3
    if has_atmo:
        cats.atmospheres = 4
        cats.textures = 4

    if not any([has_drums, has_bass, has_keys, has_synth, has_guitar, has_fx, has_atmo]):
        cats.kicks = 6
        cats.snares = 4
        cats.claps = 4
        cats.hats = 8
        cats.basses_808 = 8
        cats.keys = 5
        cats.synths = 5
        cats.impacts = 3
        cats.textures = 2

    return cats


def _make_spec_name(prompt: str, genre: str) -> str:
    from gen.prompt import _safe_filename
    if genre == "custom":
        return _safe_filename(prompt)[:40] + "_kit"
    genre_label = GENRE_PROFILES.get(genre, {}).get("label", genre)
    return f"{genre_label.lower().replace(' ', '_')}_one_shot_kit"


# ─── KitSpec templates ────────────────────────────────────

KIT_SPEC_TEMPLATES = {
    "dark_rnb": KitSpec(
        name="dark_rnb_one_shot_kit",
        prompt="dark rnb one shots",
        genre="rnb",
        style="dark, warm, intimate",
        target_dna=DNAProfile(
            spectral_centroid_target=2000.0,
            transient_aggression=0.3,
            saturation_density=0.15,
            stereo_width=0.35,
            tonal_noise_ratio=0.85,
            loudness_lufs=-14.0,
            brightness=0.35,
            darkness=0.65,
            grit=0.15,
            dryness=0.5,
            punch=0.35,
            softness=0.6,
            analog_warmth=0.6,
            width_span=0.35,
        ),
        categories=CategoryCounts(
            kicks=4, snares=2, claps=4, hats=6, open_hats=2, percs=4,
            basses_808=6, basses_sub=4, keys=8, synths=6, guitars=4,
            impacts=2, risers=2, glitches=1, textures=2, atmospheres=2, fx_noise=1,
        ),
        total_target=60,
    ),
    "trap_god": KitSpec(
        name="trap_god_one_shot_kit",
        prompt="trap god one shot kit",
        genre="trap",
        style="dark, aggressive, punchy",
        target_dna=DNAProfile(
            spectral_centroid_target=2000.0,
            transient_aggression=0.85,
            saturation_density=0.5,
            stereo_width=0.2,
            tonal_noise_ratio=0.55,
            loudness_lufs=-10.0,
            brightness=0.3,
            darkness=0.7,
            grit=0.65,
            dryness=0.8,
            punch=0.9,
            softness=0.0,
            analog_warmth=0.1,
            width_span=0.2,
        ),
        categories=CategoryCounts(
            kicks=8, snares=6, claps=4, hats=10, open_hats=2, percs=4,
            basses_808=10, basses_sub=2, keys=2, synths=4, guitars=0,
            impacts=2, risers=2, glitches=2, textures=2, atmospheres=0, fx_noise=2,
        ),
        total_target=60,
    ),
    "cinematic_impacts": KitSpec(
        name="cinematic_impacts_kit",
        prompt="cinematic impacts and textures",
        genre="cinematic",
        style="big, huge, wide, sustained",
        target_dna=DNAProfile(
            spectral_centroid_target=3000.0,
            transient_aggression=0.5,
            saturation_density=0.2,
            stereo_width=0.9,
            tonal_noise_ratio=0.55,
            loudness_lufs=-16.0,
            brightness=0.6,
            darkness=0.35,
            grit=0.2,
            dryness=0.15,
            punch=0.5,
            softness=0.3,
            analog_warmth=0.1,
            width_span=0.8,
        ),
        categories=CategoryCounts(
            kicks=4, snares=2, claps=2, hats=4, open_hats=1, percs=2,
            basses_808=4, basses_sub=4, keys=6, synths=8, guitars=2,
            impacts=6, risers=6, glitches=4, textures=6, atmospheres=6, fx_noise=2,
        ),
        total_target=60,
    ),
    "hyperpop_synth": KitSpec(
        name="hyperpop_synth_kit",
        prompt="hyperpop synth one shots",
        genre="hyperpop",
        style="bright, glossy, wide, distorted",
        target_dna=DNAProfile(
            spectral_centroid_target=5000.0,
            transient_aggression=0.6,
            saturation_density=0.65,
            stereo_width=0.9,
            tonal_noise_ratio=0.45,
            loudness_lufs=-10.0,
            brightness=0.9,
            darkness=0.05,
            grit=0.6,
            dryness=0.35,
            punch=0.6,
            softness=0.1,
            analog_warmth=0.0,
            width_span=0.9,
        ),
        categories=CategoryCounts(
            kicks=4, snares=2, claps=2, hats=6, open_hats=2, percs=2,
            basses_808=4, basses_sub=2, keys=6, synths=12, guitars=2,
            impacts=4, risers=4, glitches=4, textures=4, atmospheres=2, fx_noise=4,
        ),
        total_target=60,
    ),
    "ambient_textures": KitSpec(
        name="ambient_textures_kit",
        prompt="ambient textures and atmospheres",
        genre="ambient",
        style="soft, airy, warm, sustained",
        target_dna=DNAProfile(
            spectral_centroid_target=3000.0,
            transient_aggression=0.1,
            saturation_density=0.1,
            stereo_width=0.8,
            tonal_noise_ratio=0.5,
            loudness_lufs=-20.0,
            brightness=0.5,
            darkness=0.3,
            grit=0.0,
            dryness=0.1,
            punch=0.1,
            softness=0.9,
            analog_warmth=0.3,
            width_span=0.8,
        ),
        categories=CategoryCounts(
            kicks=2, snares=0, claps=0, hats=2, open_hats=1, percs=2,
            basses_808=2, basses_sub=4, keys=6, synths=10, guitars=4,
            impacts=2, risers=4, glitches=2, textures=6, atmospheres=8, fx_noise=4,
        ),
        total_target=60,
    ),
}


def cmd_kit_spec(args):
    """Generate a KitSpec JSON from a natural language prompt."""
    prompt = " ".join(args.prompt) if isinstance(args.prompt, list) else args.prompt
    out_path = Path(args.out) if args.out else None

    spec = infer_spec_from_prompt(prompt)

    print(f"cShot KitSpec Generator")
    print(f"{'='*60}")
    print(f"Prompt: {prompt}")
    print()

    print(f"Kit: {spec.name}")
    print(f"  Genre:      {spec.genre}")
    print(f"  Style:      {spec.style or '(inferred from genre)'}")
    print(f"  Total:      {spec.total_target} files")
    print()

    print("  Category Plan:")
    cat = spec.categories
    total = 0
    for field_name, label in [
        ("kicks", "Kicks"), ("snares", "Snares"), ("claps", "Claps"),
        ("hats", "Hats"), ("open_hats", "Open Hats"), ("percs", "Percs"),
        ("basses_808", "808s"), ("basses_sub", "Sub Bass"),
        ("keys", "Keys"), ("synths", "Synths"), ("guitars", "Guitars"),
        ("impacts", "Impacts"), ("risers", "Risers"), ("glitches", "Glitches"),
        ("textures", "Textures"), ("atmospheres", "Atmospheres"), ("fx_noise", "FX Noise"),
    ]:
        count = getattr(cat, field_name, 0)
        if count > 0:
            print(f"    {label:<15s} {count}")
            total += count
    print(f"    {'─'*22}")
    print(f"    {'Total':<15s} {total}")
    print()

    dna = spec.target_dna
    print("  DNA Targets:")
    print(f"    Spectral centroid:  {dna.spectral_centroid_target:.0f} Hz")
    print(f"    Transient aggr:     {dna.transient_aggression:.2f}")
    print(f"    Saturation:         {dna.saturation_density:.2f}")
    print(f"    Stereo width:       {dna.stereo_width:.2f}")
    print(f"    Tonal/noise ratio:  {dna.tonal_noise_ratio:.2f}")
    print(f"    Loudness:           {dna.loudness_lufs:.1f} LUFS")
    print(f"    Brightness:         {dna.brightness:.2f} / Darkness: {dna.darkness:.2f}")
    print(f"    Grit:               {dna.grit:.2f} / Punch: {dna.punch:.2f}")
    print()

    if out_path:
        out_path.parent.mkdir(parents=True, exist_ok=True)
        spec_dict = kit_spec_to_dict(spec)
        with open(out_path, "w") as f:
            json.dump(spec_dict, f, indent=2)
        print(f"Wrote: {out_path}")
    else:
        spec_dict = kit_spec_to_dict(spec)
        print(json.dumps(spec_dict, indent=2))
