"""
WEEK 2+ — Kit generation engine: pack DNA targeting, category planning,
coherence metrics, and full kit generation pipeline.
"""

import json
import math
import random
import sys
import time
from collections import Counter, defaultdict
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE
from gen.io import read_wav, write_wav
from gen.features import compute_features, detect_tempo, detect_pitch_full, estimate_key
from gen.polish import polish_file, trim_silence, apply_fade, normalize_peak, validate_audio
from gen.prompt import (
    parse_prompt, _resolve_generator, _generate_variation,
    _seed_from_prompt, _write_metadata, _safe_filename,
)
from gen.song import analyze_song as analyze_song_enhanced, extract_palette
from gen.kit_spec import (
    KitSpec, CategoryCounts, DNAProfile, CohesionTargets, ExportConfig,
    kit_spec_to_dict, infer_spec_from_prompt, GENRE_DNA_BIAS, GENRE_CATEGORY_BIAS,
)


# ─── DNA → Parameter Mapping ─────────────────────────────

def dna_to_overrides(dna: dict) -> dict:
    """Convert a pack_dna.json DNA entry to generator parameter overrides."""
    overrides = {}

    sb = dna.get("spectral_balance", {})
    centroid = sb.get("mean_centroid", 2500)
    if centroid > 5000:
        overrides["brightness"] = 1.4
        overrides["high_shelf_db"] = 6.0
    elif centroid > 3500:
        overrides["brightness"] = 1.1
        overrides["high_shelf_db"] = 3.0
    elif centroid < 1500:
        overrides["brightness"] = 0.3
        overrides["high_shelf_db"] = -6.0
    else:
        overrides["brightness"] = 0.6
        overrides["high_shelf_db"] = 0.0

    ta = dna.get("transient_aggressiveness", {})
    trans_count = ta.get("mean_transient_count", 5)
    if trans_count > 8:
        overrides["attack_ms"] = 1.0
        overrides["saturation"] = 0.6
        overrides["compression"] = 0.7
    elif trans_count > 4:
        overrides["attack_ms"] = 3.0
        overrides["saturation"] = 0.3
        overrides["compression"] = 0.4
    elif trans_count < 2:
        overrides["attack_ms"] = 30.0
        overrides["saturation"] = 0.05
        overrides["compression"] = 0.1
    else:
        overrides["attack_ms"] = 10.0
        overrides["saturation"] = 0.15
        overrides["compression"] = 0.2

    sd = dna.get("saturation_density", {})
    sat = sd.get("mean_saturation", 0.3)
    overrides["saturation"] = overrides.get("saturation", 0.3) + sat * 0.5

    sw = dna.get("stereo_width_profile", {})
    width = sw.get("mean_width", 0.3)
    overrides["stereo_width"] = min(width * 2.0, 1.0)

    tn = dna.get("tonal_noise_ratio", {})
    hpr = tn.get("mean_hpr", 0.6)
    overrides["noise_mix"] = max(0.0, 1.0 - hpr * 1.2)

    ep = dna.get("envelope_profile", {})
    env_atk = ep.get("mean_attack_ms", 5)
    env_dec = ep.get("mean_decay_ms", 200)
    if env_dec > 300:
        overrides["sustain_level"] = 0.4
        overrides["release_ms"] = min(env_dec * 1.5, 1000)
    else:
        overrides["sustain_level"] = 0.1
        overrides["release_ms"] = min(env_dec, 500)

    return overrides


def dna_to_genre_hint(dna: dict) -> str:
    """Map a pack DNA to the nearest genre."""
    sb = dna.get("spectral_balance", {})
    ta = dna.get("transient_aggressiveness", {})
    sd = dna.get("saturation_density", {})
    sw = dna.get("stereo_width_profile", {})
    tn = dna.get("tonal_noise_ratio", {})
    lp = dna.get("loudness_profile", {})

    centroid = sb.get("mean_centroid", 2500)
    trans = ta.get("mean_transient_count", 5)
    sat = sd.get("mean_saturation", 0.3)
    width = sw.get("mean_width", 0.3)
    hpr = tn.get("mean_hpr", 0.6)
    lufs = lp.get("mean_lufs", -14)

    if centroid < 2500 and trans > 6 and sat > 0.3:
        return "trap"
    if centroid < 2000 and trans > 5 and sat > 0.3:
        return "drill"
    if centroid < 2500 and trans < 4 and sat < 0.3 and hpr > 0.7:
        return "rnb"
    if centroid > 4000 and width > 0.6 and sat > 0.4:
        return "hyperpop"
    if centroid > 3000 and width > 0.6 and lufs < -15:
        return "ambient"
    if centroid < 3000 and sat < 0.2 and hpr > 0.6 and lufs < -14:
        return "lo_fi"
    if centroid > 3000 and trans > 5 and width > 0.5:
        return "cinematic"
    if centroid > 3000 and trans > 6 and sat < 0.3:
        return "house"
    if centroid > 3500 and trans > 6 and sat > 0.3:
        return "techno"
    if centroid > 3000 and trans > 5 and sat > 0.5 and width > 0.5:
        return "rage"
    return "custom"


def dna_to_kit_spec(pack_name: str, dna: dict) -> KitSpec:
    """Build a KitSpec from a pack DNA entry."""
    genre_hint = dna_to_genre_hint(dna)
    
    if genre_hint in GENRE_CATEGORY_BIAS:
        cats = GENRE_CATEGORY_BIAS[genre_hint]
    else:
        cats = CategoryCounts()

    sb = dna.get("spectral_balance", {})
    ta = dna.get("transient_aggressiveness", {})
    sd = dna.get("saturation_density", {})
    st = dna.get("stereo_width_profile", {})
    tn = dna.get("tonal_noise_ratio", {})
    lp = dna.get("loudness_profile", {})

    centroid = sb.get("mean_centroid", 2500)
    trans = ta.get("mean_transient_count", 5)
    sat = sd.get("mean_saturation", 0.3)
    width = st.get("mean_width", 0.3)
    hpr = tn.get("mean_hpr", 0.6)
    lufs = lp.get("mean_lufs", -14)

    target_dna = DNAProfile(
        spectral_centroid_target=centroid,
        transient_aggression=min(trans / 10.0, 1.0),
        saturation_density=min(sat * 2.0, 1.0),
        stereo_width=min(width * 2.0, 1.0),
        tonal_noise_ratio=hpr,
        loudness_lufs=lufs if lufs > -90 else -14,
        brightness=min(centroid / 8000.0, 1.0),
        darkness=min(1.0 - centroid / 8000.0, 1.0),
        grit=min(sat * 2.0, 1.0),
        dryness=0.5 + hpr * 0.3,
        punch=min(trans / 10.0, 1.0),
        softness=min(1.0 - trans / 10.0, 1.0),
        analog_warmth=0.2 if hpr > 0.6 else 0.0,
        width_span=min(width * 2.0, 1.0),
    )

    total = sum(v for k, v in vars(cats).items() if isinstance(v, int))
    clean_name = pack_name.replace("Packs/", "").replace("src-tauri/Packs/", "")
    safe_name = _safe_filename(clean_name)[:30] + "_dna_kit"

    return KitSpec(
        name=safe_name,
        prompt=f"kit from {clean_name} DNA",
        genre=genre_hint,
        style=f"dna-derived from {clean_name}",
        target_dna=target_dna,
        categories=cats,
        total_target=total,
        created_at=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    )


def load_pack_dna() -> dict:
    """Load the pack_dna.json file."""
    dna_path = REPO_ROOT / "gen" / "census" / "pack_dna.json"
    if not dna_path.exists():
        print("Error: pack_dna.json not found. Run 'pack-dna' first.")
        sys.exit(1)
    with open(dna_path) as f:
        return json.load(f)


def find_pack_in_dna(pack_name: str, dna_data: dict) -> tuple:
    """Find a pack by name (partial match) in the DNA data.
    Returns (full_key, dna_entry) or (None, None).
    """
    pack_dna = dna_data.get("pack_dna", {})
    name_lower = pack_name.lower()

    for key in pack_dna:
        key_lower = key.lower().replace("\\", "/")
        if name_lower in key_lower:
            return key, pack_dna[key]

    for key in pack_dna:
        key_lower = key.lower().replace("\\", "/")
        pack_short = key_lower.split("/")[-1]
        if name_lower in pack_short:
            return key, pack_dna[key]

    return None, None


def list_pack_dna(dna_data: dict):
    """Print available packs with DNA info."""
    pack_dna = dna_data.get("pack_dna", {})
    print(f"Available packs ({len(pack_dna)}):")
    print(f"{'Pack':<50} {'Files':>6} {'LUFS':>8} {'Centroid':>10} {'Width':>8} {'Trans':>8}")
    print("-" * 92)
    sorted_packs = sorted(
        pack_dna.keys(),
        key=lambda k: pack_dna[k].get("spectral_balance", {}).get("mean_centroid", 0)
    )
    for key in sorted_packs:
        pd = pack_dna[key]
        nf = pd.get("num_files", 0)
        lp = pd.get("loudness_profile", {})
        sb = pd.get("spectral_balance", {})
        st = pd.get("stereo_width_profile", {})
        ta = pd.get("transient_aggressiveness", {})
        luf = lp.get("mean_lufs", 0)
        cent = sb.get("mean_centroid", 0)
        w = st.get("mean_width", 0)
        tc = ta.get("mean_transient_count", 0)
        short_key = key.replace("src-tauri/Packs/", "").replace("Packs/", "")
        luf_str = f"{luf:.1f}" if luf > -90 else "n/a"
        print(f"  {short_key:<48} {nf:>6} {luf_str:>8} {cent:>8.0f}Hz {w:>7.3f} {tc:>6.1f}")


# ─── Kit Generation Engine ────────────────────────────────

CATEGORY_GENERATOR_MAP = {
    "kicks": ("batch", "kick", 0.4),
    "snares": ("batch", "snare", 0.3),
    "claps": ("batch", "clap", 0.2),
    "hats": ("batch", "closed_hat", 0.2),
    "open_hats": ("batch", "open_hat", 0.2),
    "percs": ("synth-gen", "perc", 0.3),
    "basses_808": ("bass-gen", "808", 0.5),
    "basses_sub": ("bass-gen", "808", 0.3),
    "keys": ("piano-gen", "acoustic", 0.3),
    "synths": ("synth-gen", "stab", 0.4),
    "guitars": ("guitar-gen", "nylon", 0.3),
    "impacts": ("fx-gen", "impact", 0.5),
    "risers": ("fx-gen", "riser", 0.4),
    "glitches": ("fx-gen", "glitch", 0.5),
    "textures": ("fx-gen", "air", 0.5),
    "atmospheres": ("fx-gen", "noise_hit", 0.5),
    "fx_noise": ("fx-gen", "vinyl", 0.3),
}

FAMILY_PITCH_MAP = {
    "kick": (150, 60),
    "snare": (300, 200),
    "clap": (400, 300),
    "closed_hat": (8000, 6000),
    "open_hat": (6000, 4000),
    "808": (80, 40),
    "piano": (440, 220),
    "synth": (440, 220),
    "guitar": (440, 300),
    "impact": (200, 100),
    "riser": (300, 150),
    "glitch": (400, 200),
    "air": (1000, 500),
    "noise_hit": (500, 200),
    "vinyl": (2000, 1000),
}


def generate_category_file(cat_name: str, index: int, overrides: dict,
                           out_dir: Path, kit_name: str) -> tuple:
    """Generate a single file for a category.
    Returns (out_path, samples) or (None, None) on error.
    """
    if cat_name not in CATEGORY_GENERATOR_MAP:
        return None, None

    family, profile, var_amount = CATEGORY_GENERATOR_MAP[cat_name]

    # Pick a descriptive adjective based on overrides
    adj_pool = ["bright", "dark", "warm", "punchy", "soft", "clean",
                "distorted", "mellow", "crisp", "airy", "dry", "wet",
                "wide", "narrow", "smooth", "rough"]
    
    if overrides.get("brightness", 0.5) > 0.8:
        adj_pool = ["bright", "crisp", "glossy", "expensive"]
    elif overrides.get("brightness", 0.5) < 0.4:
        adj_pool = ["dark", "warm", "mellow", "dusty"]
    if overrides.get("punch", 0.5) > 0.7:
        adj_pool += ["punchy", "hard", "aggressive"]
    elif overrides.get("punch", 0.5) < 0.3:
        adj_pool += ["soft", "gentle", "smooth"]

    adj = random.choice(adj_pool)

    # Build prompt
    if cat_name == "kicks":
        prompt = f"{adj} kick"
    elif cat_name == "snares":
        prompt = f"{adj} snare"
    elif cat_name == "claps":
        prompt = f"{adj} clap"
    elif cat_name == "hats":
        prompt = f"{adj} hihat"
    elif cat_name == "open_hats":
        prompt = f"{adj} open hat"
    elif cat_name == "percs":
        prompt = f"{adj} percussion"
    elif cat_name == "basses_808":
        prompt = f"{adj} 808"
    elif cat_name == "basses_sub":
        prompt = f"{adj} sub bass"
    elif cat_name == "keys":
        prompt = f"{adj} piano"
    elif cat_name == "synths":
        prompt = f"{adj} synth"
    elif cat_name == "guitars":
        prompt = f"{adj} guitar"
    elif cat_name == "impacts":
        prompt = f"{adj} impact"
    elif cat_name == "risers":
        prompt = f"{adj} riser"
    elif cat_name == "glitches":
        prompt = f"{adj} glitch"
    elif cat_name == "textures":
        prompt = f"{adj} texture"
    elif cat_name == "atmospheres":
        prompt = f"{adj} atmosphere"
    elif cat_name == "fx_noise":
        prompt = f"{adj} noise"
    else:
        prompt = f"{adj} synth"

    parsed = parse_prompt(prompt)

    og_overrides = dict(parsed.get("overrides", {}))
    for k, v in overrides.items():
        if k in og_overrides and isinstance(v, (int, float)) and isinstance(og_overrides[k], (int, float)):
            parsed["overrides"][k] = og_overrides[k] * 0.5 + v * 0.5
        else:
            parsed["overrides"][k] = v

    try:
        gen_fn, default_dur, default_pitch, gen_family, profile_name, _ = _resolve_generator(parsed)
    except (ValueError, KeyError):
        return None, None

    seed = _seed_from_prompt(f"{kit_name}_{cat_name}_{index}", index)
    np.random.seed(seed % 2**32)
    dur_var = default_dur * (1.0 + (random.random() - 0.5) * var_amount)
    pitch_var = default_pitch * (1.0 + (random.random() - 0.5) * var_amount * 0.5)

    try:
        samples, actual_dur, actual_pitch = _generate_variation(dur_var, pitch_var, gen_family, gen_fn)
    except Exception:
        return None, None

    out_path = out_dir / f"{kit_name}_{cat_name}_{adj}_{index+1:03d}.wav"
    write_wav(out_path, samples)
    _write_metadata(out_path, parsed, seed, actual_dur, actual_pitch)

    return out_path, samples


def generate_kit(spec: KitSpec, out_dir: Path, polish: bool = True) -> int:
    """Generate a complete kit from a KitSpec. Returns file count."""
    out_dir.mkdir(parents=True, exist_ok=True)

    dna = spec.target_dna
    overrides = {
        "brightness": dna.brightness,
        "high_shelf_db": (dna.brightness - 0.5) * 12,
        "attack_ms": max(1.0, (1.0 - dna.punch) * 30),
        "saturation": dna.grit * 0.8 + dna.saturation_density * 0.3,
        "stereo_width": dna.stereo_width,
        "noise_mix": 1.0 - dna.tonal_noise_ratio * 0.8,
        "sustain_level": 0.5 if dna.dryness < 0.5 else 0.1,
    }

    cats = spec.categories
    category_counts = {k: v for k, v in vars(cats).items() if isinstance(v, int) and v > 0}

    total_planned = sum(category_counts.values())
    total_generated = 0

    print(f"Generating kit: {spec.name}")
    print(f"  Prompt:        {spec.prompt}")
    print(f"  Genre:         {spec.genre}")
    print(f"  Total target:  {total_planned} files")
    print(f"  Output:        {out_dir}")
    print()

    kit_name = _safe_filename(spec.name)[:25]

    for cat_name, count in sorted(category_counts.items()):
        cat_dir = out_dir / cat_name
        cat_dir.mkdir(parents=True, exist_ok=True)

        generated = 0
        for i in range(count):
            out_path, samples = generate_category_file(cat_name, i, overrides,
                                                        cat_dir, kit_name)
            if out_path is not None:
                if polish:
                    polish_file(out_path, target_db=-1.0, in_place=True)
                generated += 1

        total_generated += generated
        print(f"  [{cat_name:<15s}] {generated}/{count} generated")

    return total_generated


# ─── Kit Audit / Coherence Metrics ───────────────────────

def compute_kit_features(kit_dir: Path) -> dict:
    """Compute aggregate features for all WAV files in a kit directory."""
    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        return {}

    all_feats = []
    for w in wav_files:
        result = read_wav(w)
        if result is None:
            continue
        samples, sr = result
        if samples.ndim == 2:
            samples_mono = samples.mean(axis=1)
        else:
            samples_mono = samples
        feats = compute_features(samples_mono, sr)
        feats["file"] = str(w.relative_to(kit_dir))
        feats["category"] = w.parent.name
        all_feats.append(feats)

    return {
        "total_files": len(all_feats),
        "files": all_feats,
    }


def compute_kit_coherence(kit_dir: Path) -> dict:
    """Compute kit-level coherence metrics."""
    kit_data = compute_kit_features(kit_dir)
    if not kit_data or not kit_data["files"]:
        return {"error": "no valid files found"}

    files = kit_data["files"]

    centroids = [f.get("spectral_centroid", 0) for f in files]
    rms_vals = [f.get("rms", 0) for f in files]
    trans_counts = [f.get("transient_count", 0) for f in files]
    attacks = [f.get("attack_ms", 0) for f in files]
    hprs = [f.get("hpr", 0.5) for f in files]
    low_bands = [f.get("low_band_energy", 0) for f in files]
    high_bands = [f.get("high_band_energy", 0) for f in files]

    non_zero_centroids = [c for c in centroids if c > 0]
    centroid_cv = np.std(non_zero_centroids) / max(np.mean(non_zero_centroids), 1) if non_zero_centroids else 1

    non_zero_rms = [r for r in rms_vals if r > 0]
    rms_cv = np.std(non_zero_rms) / max(np.mean(non_zero_rms), 1e-6) if non_zero_rms else 1

    non_zero_trans = [t for t in trans_counts if t > 0]
    trans_cv = np.std(non_zero_trans) / max(np.mean(non_zero_trans), 1) if non_zero_trans else 1

    non_zero_hpr = [h for h in hprs if h > 0]
    hpr_cv = np.std(non_zero_hpr) / max(np.mean(non_zero_hpr), 0.01) if non_zero_hpr else 1

    spectral_cohesion = max(0, 1.0 - centroid_cv * 0.5)
    loudness_consistency = max(0, 1.0 - rms_cv * 0.3)
    transient_consistency = max(0, 1.0 - trans_cv * 0.3)

    stereo_vals = []
    for f in files:
        result = read_wav(Path(kit_dir) / f["file"])
        if result is not None:
            samples, sr = result
            if samples.ndim == 2:
                from gen.features import compute_stereo_correlation
                stereo_vals.append(compute_stereo_correlation(samples))
    if stereo_vals:
        stereo_cv = np.std(stereo_vals) / max(np.mean(stereo_vals), 0.01)
    else:
        stereo_cv = 0.5
    stereo_consistency = max(0, 1.0 - stereo_cv * 0.3)

    categories = defaultdict(list)
    for f in files:
        categories[f.get("category", "unknown")].append(f)

    category_diversity = len(categories) / max(1, kit_data["total_files"]) * 5
    category_diversity = min(category_diversity, 1.0)

    filenames = [f["file"] for f in files]
    name_similarity = _compute_filename_duplicate_risk(filenames)
    duplicate_risk = name_similarity

    spectral_cohesion = max(0, min(spectral_cohesion, 1.0))
    loudness_consistency = max(0, min(loudness_consistency, 1.0))
    transient_consistency = max(0, min(transient_consistency, 1.0))
    stereo_consistency = max(0, min(stereo_consistency, 1.0))

    overall = (spectral_cohesion + loudness_consistency + transient_consistency
               + stereo_consistency + category_diversity) / 5.0

    return {
        "total_files": kit_data["total_files"],
        "categories": dict(categories),
        "spectral_cohesion": round(spectral_cohesion, 3),
        "loudness_consistency": round(loudness_consistency, 3),
        "transient_consistency": round(transient_consistency, 3),
        "stereo_consistency": round(stereo_consistency, 3),
        "category_diversity": round(category_diversity, 3),
        "duplicate_risk": round(duplicate_risk, 3),
        "overall_coherence": round(overall, 3),
        "centroid_mean": float(np.mean(non_zero_centroids)) if non_zero_centroids else 0,
        "centroid_std": float(np.std(non_zero_centroids)) if non_zero_centroids else 0,
        "rms_mean": float(np.mean(non_zero_rms)) if non_zero_rms else 0,
        "rms_std": float(np.std(non_zero_rms)) if non_zero_rms else 0,
        "transient_mean": float(np.mean(non_zero_trans)) if non_zero_trans else 0,
        "hpr_mean": float(np.mean(non_zero_hpr)) if non_zero_hpr else 0,
    }


def _compute_filename_duplicate_risk(filenames: list[str]) -> float:
    """Estimate duplicate risk from filename patterns."""
    if len(filenames) < 2:
        return 0.0
    prefixes = [f.rsplit("_", 1)[0] if "_" in f else f.split(".")[0] for f in filenames]
    counter = Counter(prefixes)
    repeated = sum(c - 1 for c in counter.values())
    return min(repeated / max(len(filenames) - 1, 1), 1.0)


def compute_dna_distance(generated_feats: dict, target_dna: dict) -> dict:
    """Compute distance between generated kit features and target pack DNA."""
    gf = generated_feats
    td = target_dna

    centroid_gen = gf.get("centroid_mean", 0)
    centroid_target = td.get("spectral_balance", {}).get("mean_centroid", 2500)
    centroid_dist = abs(centroid_gen - centroid_target) / max(centroid_target, 1)

    rms_gen = gf.get("rms_mean", 0)
    lufs_target = td.get("loudness_profile", {}).get("mean_lufs", -14)
    lufs_est = -20 + 20 * math.log10(max(rms_gen, 1e-10))
    loudness_dist = abs(lufs_est - lufs_target) / max(abs(lufs_target), 1)

    trans_gen = gf.get("transient_mean", 0)
    trans_target = td.get("transient_aggressiveness", {}).get("mean_transient_count", 5)
    trans_dist = abs(trans_gen - trans_target) / max(trans_target, 1)

    hpr_gen = gf.get("hpr_mean", 0.5)
    hpr_target = td.get("tonal_noise_ratio", {}).get("mean_hpr", 0.6)
    hpr_dist = abs(hpr_gen - hpr_target) / max(hpr_target, 0.1)

    overall_dist = (centroid_dist + loudness_dist + trans_dist + hpr_dist) / 4.0
    overall_sim = max(0, 1.0 - overall_dist)

    return {
        "centroid_distance": round(centroid_dist, 3),
        "loudness_distance": round(loudness_dist, 3),
        "transient_distance": round(trans_dist, 3),
        "tonal_noise_distance": round(hpr_dist, 3),
        "overall_distance": round(overall_dist, 3),
        "overall_similarity": round(overall_sim, 3),
        "centroid_generated": round(centroid_gen, 1),
        "centroid_target": round(centroid_target, 1),
        "transient_generated": round(trans_gen, 1),
        "transient_target": round(trans_target, 1),
        "hpr_generated": round(hpr_gen, 3),
        "hpr_target": round(hpr_target, 3),
    }


# ─── Song Analysis (Weeks 7-8) ────────────────────────────

PRODUCTION_STYLES = [
    "trap", "drill", "rage", "pop", "rnb", "house", "techno",
    "ambient", "lo-fi", "cinematic", "rock", "metal", "jazz",
    "classical", "folk", "electronic", "hip-hop", "dubstep",
    "garage", "footwork", "hyperpop",
]

TEXTURE_TYPES = [
    "dense", "sparse", "layered", "minimal",
    "airy", "heavy", "bright", "dark",
    "warm", "cold", "clean", "distorted",
    "acoustic", "electronic", "organic", "synthetic",
]


def analyze_song(song_path: Path) -> dict:
    """Analyze a full song file for kit generation parameters."""
    result = read_wav(song_path, mono=False)
    if result is None:
        return {"error": "could not read file"}

    samples, sr = result
    duration = len(samples) / sr

    if samples.ndim == 2:
        mono = samples.mean(axis=1)
    else:
        mono = samples

    if len(mono) > sr * 30:
        mid_section = mono[len(mono)//2 - sr*15:len(mono)//2 + sr*15]
    else:
        mid_section = mono

    feats = compute_features(mid_section, sr)

    bpm, bpm_conf = detect_tempo(mid_section, sr)

    pitch_info = detect_pitch_full(mid_section, sr)
    key_name = "unknown"
    key_conf = 0.0

    all_pitches = []
    for i in range(0, len(mid_section), sr):
        segment = mid_section[i:i+sr]
        if len(segment) > sr // 4:
            p = detect_pitch_full(segment, sr)
            if p["confidence"] > 0.3:
                all_pitches.append(p["pitch_hz"])
    if all_pitches:
        key_name, key_conf = estimate_key(all_pitches)

    centroid = feats.get("spectral_centroid", 3000)
    if centroid > 5000:
        spectral_mood = "bright"
    elif centroid > 3000:
        spectral_mood = "balanced"
    else:
        spectral_mood = "dark"

    trans_count = feats.get("transient_count", 5)
    if trans_count > 8:
        transient_density = "high"
    elif trans_count > 4:
        transient_density = "medium"
    else:
        transient_density = "low"

    loudness = feats.get("rms", 0.1)
    if loudness > 0.3:
        loudness_level = "loud"
    elif loudness > 0.1:
        loudness_level = "moderate"
    else:
        loudness_level = "quiet"

    hpr = feats.get("hpr", 0.5)
    if hpr > 0.7:
        texture = "tonal"
    elif hpr < 0.3:
        texture = "noisy"
    else:
        texture = "mixed"

    sections_count = max(1, int(duration / 8))
    dominant_sections = ["intro", "verse", "chorus", "bridge", "outro"]
    detected_sections = min(sections_count, len(dominant_sections))

    dominant_style = "electronic"
    if centroid < 3000 and loudness < 0.15:
        dominant_style = "ambient"
    elif centroid < 2500 and trans_count > 5:
        dominant_style = "trap"
    elif centroid > 4000 and bpm > 120:
        dominant_style = "electronic"

    dominant_texture = "mixed"
    if hpr > 0.7 and centroid < 3000:
        dominant_texture = "warm"
    elif hpr < 0.3 and centroid > 4000:
        dominant_texture = "bright"
    elif trans_count > 8:
        dominant_texture = "dense"

    analysis = {
        "file": song_path.name,
        "duration_s": round(duration, 1),
        "tempo_bpm": round(bpm, 1),
        "tempo_confidence": round(bpm_conf, 3),
        "key": key_name,
        "key_confidence": round(key_conf, 3),
        "pitch": round(pitch_info["pitch_hz"], 1),
        "spectral_centroid": round(centroid, 0),
        "spectral_mood": spectral_mood,
        "transient_density": transient_density,
        "transient_count": trans_count,
        "loudness_level": loudness_level,
        "loudness_rms": round(loudness, 4),
        "hpr": round(hpr, 3),
        "texture": texture,
        "detected_sections": detected_sections,
        "dominant_style": dominant_style,
        "dominant_texture": dominant_texture,
        "duration_minutes": round(duration / 60, 1),
    }

    return analysis


def song_analysis_to_dna(analysis: dict) -> DNAProfile:
    """Convert song analysis to a DNAProfile for kit generation."""
    centroid = analysis.get("spectral_centroid", 3000)
    bpm = analysis.get("tempo_bpm", 120)
    trans = analysis.get("transient_count", 5)
    hpr = analysis.get("hpr", 0.5)

    punch = min(trans / 10.0, 1.0)
    if trans < 3:
        punch = 0.2

    brightness = min(centroid / 8000.0, 1.0)
    darkness = min(1.0 - brightness, 1.0)

    width = 0.4
    if bpm > 130:
        width = 0.5
    elif bpm < 90:
        width = 0.3

    sat = 0.3
    if "loud" in str(analysis.get("loudness_level", "")):
        sat = 0.5

    tonal = hpr

    return DNAProfile(
        spectral_centroid_target=centroid,
        transient_aggression=punch,
        saturation_density=sat,
        stereo_width=width,
        tonal_noise_ratio=tonal,
        loudness_lufs=-10 if analysis.get("loudness_level") == "loud" else -14,
        brightness=brightness,
        darkness=darkness,
        grit=sat,
        dryness=0.5 + tonal * 0.3,
        punch=punch,
        softness=1.0 - punch,
        analog_warmth=0.3 if tonal > 0.6 else 0.0,
        width_span=width,
    )


def cmd_kit_from_song(args):
    """Analyze a song and generate a kit matching its sonic profile."""
    strategy = getattr(args, 'strategy', 'reference')
    if strategy == 'reference':
        from gen.retrieval import cmd_kit_from_song_retrieval
        cmd_kit_from_song_retrieval(args)
        return
    song_path = Path(args.song)
    if not song_path.exists():
        print(f"Error: {song_path} not found")
        sys.exit(1)

    count = args.count
    style = getattr(args, "style", "inspired")
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / f"from_{song_path.stem}"
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"cShot Kit from Song Analysis")
    print(f"{'='*60}")
    print(f"Song:  {song_path}")
    print(f"Style: {style}")
    print()

    analysis = analyze_song(song_path)
    if "error" in analysis:
        print(f"Error: {analysis['error']}")
        sys.exit(1)

    print(f"Song Analysis:")
    print(f"  Duration:        {analysis['duration_minutes']}m ({analysis['duration_s']}s)")
    print(f"  Tempo:           {analysis['tempo_bpm']} BPM (conf={analysis['tempo_confidence']:.2f})")
    print(f"  Key:             {analysis['key']} (conf={analysis['key_confidence']:.2f})")
    print(f"  Spectral mood:   {analysis['spectral_mood']} ({analysis['spectral_centroid']:.0f} Hz)")
    print(f"  Transient dens:  {analysis['transient_density']} ({analysis['transient_count']})")
    print(f"  Loudness:        {analysis['loudness_level']}")
    print(f"  Texture:         {analysis['texture']} (HPR={analysis['hpr']:.2f})")
    print(f"  Dominant style:  {analysis['dominant_style']}")
    print(f"  Dominant txtr:   {analysis['dominant_texture']}")
    print(f"  Sections:        {analysis['detected_sections']}")
    print()

    dna = song_analysis_to_dna(analysis)

    key_lower = analysis.get("key", "C major").lower()
    if "minor" in key_lower:
        mood_adjectives = ["dark", "emotional", "deep"]
    else:
        mood_adjectives = ["bright", "uplifting", "warm"]

    bpm = analysis.get("tempo_bpm", 120)
    if bpm > 130:
        energy_adj = "energetic"
    elif bpm > 100:
        energy_adj = "groovy"
    else:
        energy_adj = "chill"

    dominant_genre = analysis.get("dominant_style", "electronic")
    prompt = f"{mood_adjectives[0]} {energy_adj} {dominant_genre} kit"

    key_note = analysis.get("key", "C").split()[0]
    bpm_tag = int(round(analysis.get("tempo_bpm", 120)))

    genre_for_cats = dominant_genre if dominant_genre in GENRE_CATEGORY_BIAS else "custom"
    if genre_for_cats == "custom":
        cats = CategoryCounts()
        cats.kicks = max(3, int(count * 0.1))
        cats.snares = max(2, int(count * 0.08))
        cats.claps = max(2, int(count * 0.08))
        cats.hats = max(4, int(count * 0.12))
        cats.percs = max(2, int(count * 0.06))
        cats.basses_808 = max(2, int(count * 0.1))
        cats.keys = max(3, int(count * 0.1))
        cats.synths = max(3, int(count * 0.1))
        cats.impacts = max(2, int(count * 0.06))
        remaining = count - sum(v for k, v in vars(cats).items() if isinstance(v, int))
        cats.textures = max(1, int(remaining * 0.5))
        cats.atmospheres = max(1, int(remaining * 0.5))
    else:
        cats = GENRE_CATEGORY_BIAS[genre_for_cats]
        total = sum(v for k, v in vars(cats).items() if isinstance(v, int))
        if total > 0 and total != count:
            ratio = count / total
            for field in vars(cats):
                current = getattr(cats, field)
                if isinstance(current, int):
                    setattr(cats, field, max(1, int(round(current * ratio))))

    kit_name = f"{song_path.stem[:20]}_bpm{bpm_tag}_{key_note}".replace(" ", "_")
    kit_name = _safe_filename(kit_name)

    style_factor = {"strict": 0.3, "inspired": 0.5, "wild": 0.8}.get(style, 0.5)

    if style == "wild":
        import random
        dna.brightness = min(1.0, dna.brightness * random.uniform(0.7, 1.3))
        dna.darkness = min(1.0, dna.darkness * random.uniform(0.7, 1.3))
        dna.punch = min(1.0, dna.punch * random.uniform(0.6, 1.4))
        dna.spectral_centroid_target *= random.uniform(0.7, 1.3)
        dna.stereo_width = min(1.0, dna.stereo_width * random.uniform(0.8, 1.5))
    elif style == "strict":
        dna.brightness = min(1.0, dna.brightness * 0.95)
        dna.darkness = min(1.0, dna.darkness * 0.95)

    spec = KitSpec(
        name=kit_name,
        prompt=prompt,
        source_refs=[str(song_path)],
        genre=dominant_genre,
        style=f"{style} derivation from {song_path.name}, tempo={bpm_tag}bpm, key={analysis['key']}",
        target_dna=dna,
        categories=cats,
        total_target=count,
        created_at=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    )

    print(f"Kit Plan:")
    print(f"  Name:            {spec.name}")
    print(f"  Derivation:      {style} (factor={style_factor})")
    print(f"  Mood:            {mood_adjectives[0]}, {energy_adj}")
    print(f"  Key:             {analysis['key']}")
    print(f"  Tempo:           {bpm_tag} BPM")
    print(f"  Target:          {count} files")
    print()

    generated = generate_kit(spec, out_dir, polish=True)
    print(f"\nGenerated {generated}/{count} files → {out_dir}")

    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"\nCoherence: {coherence.get('overall_coherence', 0):.3f}")
        print(f"Centroid:  {coherence.get('centroid_mean', 0):.0f} Hz")

    report = {
        "song_analysis": analysis,
        "kit_spec": kit_spec_to_dict(spec),
        "generated": generated,
        "coherence": coherence,
        "target_dna": vars(dna),
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    report_path = out_dir / "kit_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"\nBPM/key/mood metadata in: {report_path}")
    print(f"Report: {report_path}")


# ─── CLI Commands ─────────────────────────────────────────

# ─── Mini Kit from Song (Week 4) ─────────────────────────

def cmd_mini_kit_from_song(args):
    """Generate a focused 13-file mini kit from a song analysis.
    5 drums + 3 basses + 3 tonal stabs + 2 FX, coherent with song mood.
    """
    song_path = Path(args.song)
    if not song_path.exists():
        print(f"Error: {song_path} not found")
        sys.exit(1)

    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "mini_kits" / f"from_{song_path.stem}"
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"cShot Mini Kit from Song (Week 4)")
    print(f"{'='*60}")
    print(f"Song: {song_path}")
    print()

    analysis = analyze_song_enhanced(song_path)
    if "error" in analysis:
        print(f"Error: {analysis['error']}")
        sys.exit(1)

    palette = extract_palette(analysis)

    print(f"Song Analysis:")
    print(f"  Duration:  {analysis['duration_minutes']}m")
    print(f"  Tempo:     {analysis['tempo_bpm']} BPM (conf={analysis['tempo_confidence']:.2f})")
    print(f"  Key:       {analysis['key']} (conf={analysis['key_confidence']:.2f})")
    print(f"  Style:     {analysis['dominant_style']}")
    print(f"  Mood:      {analysis['spectral']['mood']}")
    print()
    print(f"Palette:")
    print(f"  Drums:     {', '.join(palette['drum_texture']['primary_qualities'][:3])}")
    print(f"  Bass:      {', '.join(palette['bass_character']['primary_qualities'][:3])}")
    print(f"  Synth:     {', '.join(palette['synth_brightness']['primary_qualities'][:3])}")
    print(f"  Atmos:     {', '.join(palette['atmosphere_density']['primary_qualities'][:3])}")
    print(f"  FX:        {', '.join(palette['fx_style']['primary_qualities'][:3])}")
    print()

    dna = song_analysis_to_dna(analysis)

    cats = CategoryCounts()
    for field in vars(cats):
        setattr(cats, field, 0)
    cats.kicks = 2
    cats.snares = 1
    cats.claps = 1
    cats.hats = 1
    cats.basses_808 = 2
    cats.basses_sub = 1
    cats.keys = 1
    cats.synths = 1
    cats.guitars = 1
    cats.impacts = 1
    cats.textures = 1

    key_lower = analysis.get("key", "C major").lower()
    bpm = analysis.get("tempo_bpm", 120)
    style = analysis.get("dominant_style", "electronic")
    mood_adj = "dark" if "minor" in key_lower else "bright"
    energy_adj = "energetic" if bpm > 130 else "groovy" if bpm > 100 else "chill"
    key_note = analysis.get("key", "C").split()[0]
    bpm_tag = int(round(bpm))

    kit_name = f"mini_{song_path.stem[:15]}_{bpm_tag}_{key_note}_{mood_adj}"
    kit_name = _safe_filename(kit_name)

    spec = KitSpec(
        name=kit_name,
        prompt=f"{mood_adj} {energy_adj} {style} mini kit",
        source_refs=[str(song_path)],
        genre=style,
        style=f"mini-kit from {song_path.name}, {bpm_tag}bpm, {analysis['key']}",
        target_dna=dna,
        categories=cats,
        total_target=13,
        created_at=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    )

    print(f"Mini Kit Plan:")
    print(f"  Name:       {spec.name}")
    print(f"  Style:      {style}")
    print(f"  Mood:       {mood_adj}, {energy_adj}")
    print(f"  Key:        {analysis['key']}  BPM: {bpm_tag}")
    print(f"  Target:     13 files (5 drums, 3 bass, 3 tonal, 2 FX)")
    print()

    generated = generate_kit(spec, out_dir, polish=True)
    print(f"\nGenerated {generated}/13 files → {out_dir}")

    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"Coherence: {coherence.get('overall_coherence', 0):.3f}")
        print(f"Centroid:  {coherence.get('centroid_mean', 0):.0f} Hz")

    report = {
        "song_analysis": analysis,
        "palette": palette,
        "kit_spec": kit_spec_to_dict(spec),
        "generated": generated,
        "coherence": coherence,
        "target_dna": vars(dna),
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    report_path = out_dir / "mini_kit_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"Report: {report_path}")


# ─── Genre DNA (Week 9) ──────────────────────────────────

def cmd_genre_dna(args):
    """Generate a kit from a genre profile."""
    genre_name = args.genre.lower().replace("-", "_").replace(" ", "_")
    count = args.count
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / f"{genre_name}_kit"
    out_dir.mkdir(parents=True, exist_ok=True)

    from gen.kit_spec import GENRE_DNA_BIAS, GENRE_CATEGORY_BIAS

    if genre_name not in GENRE_DNA_BIAS:
        print(f"Unknown genre '{genre_name}'. Available genres:")
        for g in sorted(GENRE_DNA_BIAS.keys()):
            print(f"  {g}")
        sys.exit(1)

    dna = GENRE_DNA_BIAS[genre_name]
    cats = GENRE_CATEGORY_BIAS.get(genre_name, CategoryCounts())

    from gen.genre import GENRE_PROFILES
    genre_label = GENRE_PROFILES.get(genre_name, {}).get("label", genre_name)

    total = sum(v for k, v in vars(cats).items() if isinstance(v, int))
    if total > 0 and total != count:
        ratio = count / total
        for field in vars(cats):
            current = getattr(cats, field)
            if isinstance(current, int):
                setattr(cats, field, max(1, int(round(current * ratio))))

    spec = KitSpec(
        name=f"{genre_name}_one_shot_kit",
        prompt=f"{genre_label.lower()} one shot kit",
        genre=genre_name,
        target_dna=dna,
        categories=cats,
        total_target=count,
        created_at=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    )

    print(f"cShot Genre DNA — {genre_label}")
    print(f"{'='*60}")
    print(f"Genre:      {genre_label}")
    print(f"Target:     {count} files")
    print(f"Centroid:   {dna.spectral_centroid_target:.0f} Hz")
    print(f"Width:      {dna.stereo_width:.2f}")
    print(f"Transient:  {dna.transient_aggression:.2f}")
    print(f"Saturation: {dna.saturation_density:.2f}")
    print(f"Brightness: {dna.brightness:.2f} / Darkness: {dna.darkness:.2f}")
    print()

    generated = generate_kit(spec, out_dir, polish=True)
    print(f"\nGenerated {generated}/{count} files → {out_dir}")

    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"Coherence: {coherence.get('overall_coherence', 0):.3f}")

    report = {
        "genre": genre_name,
        "genre_label": genre_label,
        "kit_spec": kit_spec_to_dict(spec),
        "generated": generated,
        "coherence": coherence,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    report_path = out_dir / "kit_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"Report: {report_path}")


def cmd_list_genre_dna(args):
    """List all available genre DNA profiles."""
    from gen.kit_spec import GENRE_DNA_BIAS
    from gen.genre import GENRE_PROFILES

    print(f"Available Genre DNA Profiles ({len(GENRE_DNA_BIAS)}):")
    print(f"{'Genre':<15s} {'Centroid':>10s} {'Transient':>10s} {'Width':>8s} {'Bright':>8s} {'Dark':>8s} {'Punch':>8s}")
    print("-" * 67)
    for gn in sorted(GENRE_DNA_BIAS.keys()):
        dna = GENRE_DNA_BIAS[gn]
        label = GENRE_PROFILES.get(gn, {}).get("label", gn)
        print(f"  {gn:<13s} {dna.spectral_centroid_target:>8.0f}Hz {dna.transient_aggression:>8.2f} {dna.stereo_width:>6.2f} {dna.brightness:>6.2f} {dna.darkness:>6.2f} {dna.punch:>6.2f}")


# ─── Kit Ranking (Week 14) ───────────────────────────────

def cmd_kit_rank(args):
    """Rank files inside a kit from best to worst."""
    kit_dir = Path(args.kit_folder)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No .wav files found")
        return

    print(f"Kit Ranking: {kit_dir}")
    print(f"Files: {len(wav_files)}")
    print()

    from gen.rank import score_file
    from gen.rating import _load_ratings

    ratings = _load_ratings()

    scored = []
    for w in wav_files:
        s = score_file(w, ratings)
        rel = w.relative_to(kit_dir)
        s["path"] = str(rel)
        scored.append(s)

    scored.sort(key=lambda x: x.get("score", 0), reverse=True)

    n = len(scored)
    categories = {"best": [], "good": [], "experimental": [], "reject": []}
    for i, s in enumerate(scored):
        pct = 1.0 - (i / max(n - 1, 1))
        if pct >= 0.75:
            categories["best"].append(s)
        elif pct >= 0.50:
            categories["good"].append(s)
        elif pct >= 0.25:
            categories["experimental"].append(s)
        else:
            categories["reject"].append(s)

    print(f"Ranking (best → worst):")
    print(f"  Best ({len(categories['best'])}):")
    for s in categories["best"][:10]:
        print(f"    ✓ {s['path']:<50s} score={s.get('score', 0):.3f}")
    print(f"  Good ({len(categories['good'])}):")
    for s in categories["good"][:5]:
        print(f"    △ {s['path']:<50s} score={s.get('score', 0):.3f}")
    print(f"  Experimental ({len(categories['experimental'])}):")
    for s in categories["experimental"][:3]:
        print(f"    ◇ {s['path']:<50s} score={s.get('score', 0):.3f}")
    print(f"  Reject ({len(categories['reject'])}):")
    for s in categories["reject"][:3]:
        print(f"    ✗ {s['path']:<50s} score={s.get('score', 0):.3f}")

    result = {
        "kit_dir": str(kit_dir),
        "total_files": len(scored),
        "ranking": {
            "best": [s["path"] for s in categories["best"]],
            "good": [s["path"] for s in categories["good"]],
            "experimental": [s["path"] for s in categories["experimental"]],
            "reject": [s["path"] for s in categories["reject"]],
        },
        "scores": scored,
    }
    rank_path = kit_dir / "rank_report.json"
    with open(rank_path, "w") as f:
        json.dump(result, f, indent=2)
    print(f"\nRank report: {rank_path}")


# ─── Kit Repair (Week 15) ────────────────────────────────

def cmd_kit_repair(args):
    """Auto-regenerate weak/low-ranked files in a kit."""
    kit_dir = Path(args.kit_folder)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No .wav files found")
        return

    print(f"Kit Repair: {kit_dir}")
    print(f"Files: {len(wav_files)}")
    print()

    coherence = compute_kit_coherence(kit_dir)
    if "error" in coherence:
        print(f"Error: {coherence['error']}")
        sys.exit(1)

    files = coherence.get("files", [])
    weak_files = []
    for f in files:
        centroid = f.get("spectral_centroid", 0)
        rms = f.get("rms", 0)
        score = 0.5
        if rms < 0.01:
            score -= 0.3
        if centroid < 100 or centroid > 10000:
            score -= 0.2
        if f.get("duration_ms", 0) < 20:
            score -= 0.3
        if score < 0.4:
            weak_files.append(f)

    if not weak_files:
        print("No weak files found. All files pass basic quality.")
        return

    print(f"Weak files to regenerate: {len(weak_files)}")
    print()

    overrides = dna_to_overrides({
        "spectral_balance": {"mean_centroid": coherence.get("centroid_mean", 2500)},
        "transient_aggressiveness": {"mean_transient_count": coherence.get("transient_mean", 5)},
        "saturation_density": {"mean_saturation": 0.3},
        "stereo_width_profile": {"mean_width": 0.3},
        "tonal_noise_ratio": {"mean_hpr": coherence.get("hpr_mean", 0.6)},
        "envelope_profile": {"mean_attack_ms": 5, "mean_decay_ms": 200},
    })

    regenerated = 0
    for weak in weak_files:
        rel_path = Path(weak["file"])
        cat = rel_path.parent.name
        filename = rel_path.name

        old_path = kit_dir / rel_path
        if not old_path.exists():
            continue

        old_path.unlink()

        cat_overrides = dict(overrides)
        if cat in ("kicks", "snares", "claps", "percs"):
            cat_overrides["attack_ms"] = max(1, cat_overrides.get("attack_ms", 3))
            cat_overrides["saturation"] = min(1.0, cat_overrides.get("saturation", 0.3) + 0.2)
        elif cat in ("basses_808", "basses_sub"):
            cat_overrides["brightness"] = max(0.2, cat_overrides.get("brightness", 0.5) - 0.3)
            cat_overrides["saturation"] = min(1.0, cat_overrides.get("saturation", 0.3) + 0.1)
        elif cat in ("keys", "synths", "guitars"):
            cat_overrides["attack_ms"] = cat_overrides.get("attack_ms", 8)
            cat_overrides["sustain_level"] = 0.3
        elif cat in ("impacts", "risers", "glitches", "textures", "atmospheres"):
            cat_overrides["stereo_width"] = min(1.0, cat_overrides.get("stereo_width", 0.5) + 0.2)
            cat_overrides["sustain_level"] = 0.4

        out_dir = kit_dir / cat
        out_dir.mkdir(parents=True, exist_ok=True)

        index = int(time.time() * 1000) % 10000
        result = generate_category_file(cat, index, cat_overrides, out_dir, f"repaired_{kit_dir.name}")

        if result[0] is not None:
            out_path, _ = result
            polish_file(out_path, target_db=-1.0, in_place=True)
            regenerated += 1
            print(f"  ✓ Regenerated: {filename}")
        else:
            print(f"  ✗ Failed: {filename}")

    if regenerated:
        new_coherence = compute_kit_coherence(kit_dir)
        if "error" not in new_coherence:
            print(f"\nCoherence improved: {coherence.get('overall_coherence', 0):.3f} → {new_coherence.get('overall_coherence', 0):.3f}")

    print(f"\nRegenerated {regenerated}/{len(weak_files)} files")


# ─── Kit Variations (Week 16) ────────────────────────────

VARIATION_MODES = {
    "tight": {"var_amount": 0.15, "adj_pool": ["same"]},
    "medium": {"var_amount": 0.30, "adj_pool": ["brighter", "darker", "punchier", "softer"]},
    "wild": {"var_amount": 0.50, "adj_pool": ["bright", "dark", "distorted", "wide", "narrow", "glitchy"]},
}


def cmd_kit_variations(args):
    """Generate variations of an existing kit."""
    kit_dir = Path(args.kit_folder)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    mode = args.mode
    if mode not in VARIATION_MODES:
        print(f"Unknown mode '{mode}'. Use: tight, medium, wild")
        sys.exit(1)

    out_dir = Path(args.out) if args.out else kit_dir.parent / f"{kit_dir.name}_{mode}"
    out_dir.mkdir(parents=True, exist_ok=True)

    mode_config = VARIATION_MODES[mode]

    print(f"Kit Variations: {kit_dir} → {out_dir}")
    print(f"Mode: {mode} (variation={mode_config['var_amount']})")
    print()

    coherence = compute_kit_coherence(kit_dir)
    if "error" in coherence:
        print(f"Error: {coherence['error']}")
        sys.exit(1)

    centroid = coherence.get("centroid_mean", 2500)
    hpr = coherence.get("hpr_mean", 0.6)
    trans = coherence.get("transient_mean", 5)
    rms = coherence.get("rms_mean", 0.1)

    overrides = {
        "brightness": min(centroid / 8000.0, 1.0),
        "stereo_width": min(0.3 + mode_config["var_amount"], 1.0),
        "saturation": 0.3 + mode_config["var_amount"] * 0.3,
        "noise_mix": max(0, 1.0 - hpr * 0.8),
        "attack_ms": max(1, (1.0 - min(trans / 10.0, 1.0)) * 20),
    }

    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No .wav files found")
        return

    kit_name = _safe_filename(kit_dir.name)[:20]
    total_generated = 0

    for w in wav_files:
        rel = w.relative_to(kit_dir)
        cat = rel.parent.name
        stem = w.stem

        if cat == "_top" or not cat:
            continue

        cat_dir = out_dir / cat
        cat_dir.mkdir(parents=True, exist_ok=True)

        cat_overrides = dict(overrides)
        if mode == "tight":
            cat_overrides["brightness"] = overrides.get("brightness", 0.5)
        elif mode == "wild":
            if cat in ("hats", "open_hats"):
                cat_overrides["stereo_width"] = min(1.0, overrides.get("stereo_width", 0.5) + 0.3)
            elif cat in ("basses_808", "basses_sub"):
                cat_overrides["saturation"] = min(1.0, overrides.get("saturation", 0.3) + 0.2)
            elif cat in ("keys", "synths"):
                cat_overrides["brightness"] = min(1.0, overrides.get("brightness", 0.5) + 0.3)

        index = int(time.time() * 1000 + total_generated) % 100000
        result = generate_category_file(cat, index, cat_overrides, cat_dir, f"{kit_name}_{mode}")

        if result[0] is not None:
            out_path, _ = result
            polish_file(out_path, target_db=-1.0, in_place=True)
            total_generated += 1

    print(f"\nGenerated {total_generated} {mode} variation files → {out_dir}")


# ─── Naming System (Week 17) ─────────────────────────────

MOOD_NAMES = {
    "bright": ["shine", "glow", "ray", "dawn", "sun", "light", "crystal", "ice", "star", "nova"],
    "dark": ["shadow", "moon", "void", "abyss", "night", "dark", "obsidian", "onyx", "raven", "storm"],
    "warm": ["ember", "copper", "amber", "sienna", "gold", "warmth", "honey", "maple", "rust", "clay"],
    "cold": ["frost", "ice", "glacier", "arctic", "snow", "crystal", "steel", "silver", "tin", "chill"],
    "aggressive": ["fury", "rage", "thunder", "blitz", "strike", "breaker", "crash", "volt", "spike", "blade"],
    "soft": ["velvet", "silk", "feather", "cloud", "mist", "dew", "petal", "moss", "linen", "drift"],
    "ambient": ["ether", "drift", "haze", "vapor", "aura", "orb", "shimmer", "ripple", "wave", "tide"],
    "punchy": ["pulse", "beat", "thump", "bang", "smack", "pop", "snap", "crack", "boom", "pow"],
}


def generate_kit_name(category: str, mood: str, key: str = "", index: int = 0) -> str:
    """Generate a producer-ready filename for a kit file."""
    mood_lower = mood.lower()
    mood_words = []
    for m, words in MOOD_NAMES.items():
        if m in mood_lower or mood_lower in m:
            mood_words = words
            break
    if not mood_words:
        mood_words = ["element", "wave", "pulse", "drift", "core"]

    import hashlib
    seed_hash = int(hashlib.md5(f"{category}_{mood}_{index}".encode()).hexdigest()[:8], 16)
    rng = random.Random(seed_hash)

    word = rng.choice(mood_words)
    if key:
        key_clean = key.replace(" ", "").replace("#", "sharp").replace("b", "flat")
        return f"{category}_{word}_{key_clean}_{index+1:03d}.wav"
    return f"{category}_{word}_{index+1:03d}.wav"


def rename_kit_with_naming(kit_dir: Path, style_mood: str = "", key: str = ""):
    """Rename all files in a kit with producer-friendly names."""
    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        return

    for i, w in enumerate(wav_files):
        cat = w.parent.name
        new_name = generate_kit_name(cat, style_mood or cat, key, i)
        new_path = w.parent / new_name
        if not new_path.exists():
            w.rename(new_path)


def cmd_kit_naming(args):
    """Apply producer-ready naming to a kit."""
    kit_dir = Path(args.kit_folder)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    mood = args.mood or "dark"
    key = args.key or ""

    rename_kit_with_naming(kit_dir, mood, key)
    print(f"Renamed files in {kit_dir} with mood='{mood}', key='{key}'")
    for w in sorted(kit_dir.rglob("*.wav"))[:5]:
        rel = w.relative_to(kit_dir)
        print(f"  {rel}")


# ─── Kit from Description (Week 10 alias) ────────────────

def cmd_kit_from_description(args):
    """Generate a kit from a natural language description."""
    prompt = " ".join(args.prompt) if isinstance(args.prompt, list) else args.prompt
    count = args.count
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / _safe_filename(prompt)[:30]
    out_dir.mkdir(parents=True, exist_ok=True)

    spec = infer_spec_from_prompt(prompt)

    total = sum(v for k, v in vars(spec.categories).items() if isinstance(v, int))
    if total > 0 and total != count:
        ratio = count / total
        for field in vars(spec.categories):
            current = getattr(spec.categories, field)
            if isinstance(current, int):
                setattr(spec.categories, field, max(1, int(round(current * ratio))))
    spec.total_target = count

    print(f"cShot Kit from Description")
    print(f"{'='*60}")
    print(f"Description: {prompt}")
    print(f"Genre:       {spec.genre}")
    print(f"Style:       {spec.style or '(inferred)'}")
    print(f"Target:      {count} files")
    print()

    generated = generate_kit(spec, out_dir, polish=True)
    print(f"\nGenerated {generated}/{count} files → {out_dir}")

    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"Coherence: {coherence.get('overall_coherence', 0):.3f}")

    report = {
        "description": prompt,
        "kit_spec": kit_spec_to_dict(spec),
        "generated": generated,
        "coherence": coherence,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    report_path = out_dir / "kit_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"Report: {report_path}")


def cmd_kit_from_pack_dna(args):
    """Generate a kit targeting a pack's DNA fingerprint."""
    pack_name = args.pack_name
    count = args.count
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / _safe_filename(pack_name)
    out_dir.mkdir(parents=True, exist_ok=True)

    dna_data = load_pack_dna()
    key, entry = find_pack_in_dna(pack_name, dna_data)

    if key is None:
        print(f"Pack '{pack_name}' not found in pack DNA.")
        list_pack_dna(dna_data)
        sys.exit(1)

    # Build KitSpec from DNA
    kit_spec = dna_to_kit_spec(key, entry)

    print(f"cShot Kit from Pack DNA")
    print(f"{'='*60}")
    print(f"Source pack: {key}")
    print(f"  Files:     {entry.get('num_files', 0)}")
    print(f"  Genre:     {kit_spec.genre}")
    print(f"  Target:    {kit_spec.total_target} files (will generate {count})")
    print()

    # Adjust counts proportionally
    cats = kit_spec.categories
    total = sum(v for k, v in vars(cats).items() if isinstance(v, int))
    if total > 0:
        ratio = count / total
        for field in vars(cats):
            current = getattr(cats, field)
            if isinstance(current, int):
                setattr(cats, field, max(1, int(round(current * ratio))))
    kit_spec.total_target = count

    kit_spec.source_refs = [key]

    # Generate the kit
    generated = generate_kit(kit_spec, out_dir, polish=True)
    print(f"\nGenerated {generated}/{count} files → {out_dir}")

    # Compute generated kit features
    print(f"\n{'='*60}")
    print(f"Kit audit & DNA distance report")
    print(f"{'='*60}")
    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"\nCoherence Metrics:")
        print(f"  Spectral cohesion:    {coherence['spectral_cohesion']:.3f}")
        print(f"  Loudness consistency: {coherence['loudness_consistency']:.3f}")
        print(f"  Transient consistency: {coherence['transient_consistency']:.3f}")
        print(f"  Stereo consistency:   {coherence['stereo_consistency']:.3f}")
        print(f"  Category diversity:   {coherence['category_diversity']:.3f}")
        print(f"  Duplicate risk:       {coherence['duplicate_risk']:.3f}")
        print(f"  Overall coherence:    {coherence['overall_coherence']:.3f}")

        distance = compute_dna_distance(coherence, entry)
        print(f"\nDNA Distance to Source Pack:")
        print(f"  Spectral centroid:    {distance['centroid_generated']:.0f} Hz vs {distance['centroid_target']:.0f} Hz (Δ={distance['centroid_distance']:.3f})")
        print(f"  Transient density:    {distance['transient_generated']:.1f} vs {distance['transient_target']:.1f} (Δ={distance['transient_distance']:.3f})")
        print(f"  Tonal/noise (HPR):    {distance['hpr_generated']:.3f} vs {distance['hpr_target']:.3f} (Δ={distance['tonal_noise_distance']:.3f})")
        print(f"  Overall similarity:   {distance['overall_similarity']:.3f}")
        print(f"  Overall distance:     {distance['overall_distance']:.3f}")

        report = {
            "kit_spec": kit_spec_to_dict(kit_spec),
            "source_pack": key,
            "generated_files": generated,
            "coherence": coherence,
            "dna_distance": distance,
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        }
        report_path = out_dir / "kit_report.json"
        with open(report_path, "w") as f:
            json.dump(report, f, indent=2)
        print(f"\nReport: {report_path}")
    else:
        print(f"  Warning: {coherence.get('error', 'unknown')}")


def cmd_kit_audit(args):
    """Audit a kit folder for coherence and quality."""
    kit_dir = Path(args.kit_folder)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    wav_files = sorted(kit_dir.rglob("*.wav"))
    print(f"Kit Audit: {kit_dir}")
    print(f"{'='*60}")
    print(f"Files: {len(wav_files)}")
    print()

    categories = Counter()
    for w in wav_files:
        categories[w.parent.name] += 1
    print(f"Categories ({len(categories)}):")
    for cat, count in sorted(categories.items()):
        print(f"  {cat:<20s} {count}")
    print()

    coherence = compute_kit_coherence(kit_dir)
    if "error" not in coherence:
        print("Coherence Metrics:")
        for k in ["spectral_cohesion", "loudness_consistency",
                   "transient_consistency", "stereo_consistency",
                   "category_diversity", "duplicate_risk", "overall_coherence"]:
            print(f"  {k:<25s} {coherence.get(k, 'N/A')}")
        print()
        print(f"  Centroid: {coherence.get('centroid_mean', 0):.0f} Hz ± {coherence.get('centroid_std', 0):.0f}")
        print(f"  RMS:      {coherence.get('rms_mean', 0):.4f} ± {coherence.get('rms_std', 0):.4f}")
        print(f"  HPR:      {coherence.get('hpr_mean', 0):.3f}")
    else:
        print(f"Error: {coherence.get('error')}")

    result = coherence
    result["categories"] = dict(categories)
    result["total_files"] = len(wav_files)
    audit_path = kit_dir / "kit_audit.json"
    with open(audit_path, "w") as f:
        json.dump(result, f, indent=2)
    print(f"\nAudit saved: {audit_path}")


def cmd_kit_from_folder(args):
    """Generate a kit from a reference folder, inferring DNA and categories."""
    folder = Path(args.folder)
    if not folder.exists():
        print(f"Error: folder {folder} not found")
        sys.exit(1)

    count = args.count
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / f"from_{folder.name}"
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"cShot Kit from Folder")
    print(f"{'='*60}")
    print(f"Source: {folder}")
    print()

    wav_files = sorted(folder.rglob("*.wav"))[:200]
    if not wav_files:
        print("No .wav files found in folder")
        sys.exit(1)

    print(f"Found {len(wav_files)} reference files")
    print(f"Analyzing sonic fingerprint...")

    all_feats = []
    for w in wav_files[:50]:
        result = read_wav(w)
        if result is None:
            continue
        samples, sr = result
        if samples.ndim == 2:
            samples = samples.mean(axis=1)
        feats = compute_features(samples, sr)
        all_feats.append(feats)

    if not all_feats:
        print("Could not analyze any reference files")
        sys.exit(1)

    # Compute average features
    avg_feats = {}
    for k in ["spectral_centroid", "transient_count", "hpr", "rms",
              "attack_ms", "decay_length_ms", "low_band_energy", "high_band_energy"]:
        vals = [f.get(k, 0) for f in all_feats]
        avg_feats[k] = float(np.mean(vals)) if vals else 0

    # Infer genre from features
    centroid = avg_feats.get("spectral_centroid", 2500)
    trans = avg_feats.get("transient_count", 5)
    hpr = avg_feats.get("hpr", 0.6)
    attack = avg_feats.get("attack_ms", 10)

    if attack < 3 and trans > 6:
        genre = "trap"
    elif hpr > 0.7 and centroid < 3000:
        genre = "rnb"
    elif centroid > 4000 and trans < 3:
        genre = "ambient"
    elif centroid > 5000 and trans > 4:
        genre = "hyperpop"
    elif hpr < 0.4 and trans > 5:
        genre = "rage"
    elif centroid > 3000 and trans < 4:
        genre = "cinematic"
    elif centroid < 3000 and trans < 3:
        genre = "lo_fi"
    else:
        genre = "custom"

    # Build KitSpec
    spec = infer_spec_from_prompt(f"{genre} one shots")
    spec.name = _safe_filename(folder.name)[:30] + "_kit"
    spec.source_refs = [str(folder)]

    total = sum(v for k, v in vars(spec.categories).items() if isinstance(v, int))
    if total > 0:
        ratio = count / total
        for field in vars(spec.categories):
            current = getattr(spec.categories, field)
            if isinstance(current, int):
                setattr(spec.categories, field, max(1, int(round(current * ratio))))
    spec.total_target = count

    generated = generate_kit(spec, out_dir, polish=True)
    print(f"\nGenerated {generated}/{count} files → {out_dir}")

    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"\nCoherence: {coherence.get('overall_coherence', 0):.3f}")

    report = {
        "source_folder": str(folder),
        "kit_spec": kit_spec_to_dict(spec),
        "generated": generated,
        "coherence": coherence,
        "inferred_genre": genre,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    report_path = out_dir / "kit_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"Report: {report_path}")


def cmd_kit_from_sample(args):
    """Generate a mini kit from a single reference sample."""
    strategy = getattr(args, 'strategy', 'reference')
    if strategy == 'reference':
        from gen.retrieval import retrieve_by_audio
        from gen.reference_transform import transform_references_to_kit
        sample_path = Path(args.sample)
        if not sample_path.exists():
            print(f"Error: {sample_path} not found")
            sys.exit(1)
        count = args.count
        out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / f"from_{sample_path.stem}"
        out_dir.mkdir(parents=True, exist_ok=True)
        refs = retrieve_by_audio(str(sample_path), n=count * 2)
        print(f"Retrieved {len(refs)} references similar to sample")
        transform_references_to_kit(refs, out_dir, target_count=count)
        print(f"Kit generated: {out_dir}")
        return

    sample_path = Path(args.sample)
    if not sample_path.exists():
        print(f"Error: {sample_path} not found")
        sys.exit(1)

    count = args.count
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / f"from_{sample_path.stem}"
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"cShot Kit from Sample")
    print(f"{'='*60}")
    print(f"Source: {sample_path.name}")
    print()

    result = read_wav(sample_path)
    if result is None:
        print("Could not read sample")
        sys.exit(1)
    samples, sr = result
    if samples.ndim == 2:
        samples = samples.mean(axis=1)
    feats = compute_features(samples, sr)

    centroid = feats.get("spectral_centroid", 2000)
    attack = feats.get("attack_ms", 10)
    hpr = feats.get("hpr", 0.5)
    trans = feats.get("transient_count", 3)

    print(f"Sample Analysis:")
    print(f"  Spectral centroid: {centroid:.0f} Hz")
    print(f"  Attack:            {attack:.1f} ms")
    print(f"  HPR:               {hpr:.3f}")
    print(f"  Transients:        {trans}")
    print(f"  Duration:          {feats.get('duration_ms', 0):.0f} ms")
    print()

    # Infer style
    overrides = {}
    if centroid > 5000:
        overrides["brightness"] = 1.4
        overrides["high_shelf_db"] = 6.0
    elif centroid < 2000:
        overrides["brightness"] = 0.3
        overrides["high_shelf_db"] = -6.0
    else:
        overrides["brightness"] = 0.6

    if attack < 3:
        overrides["attack_ms"] = 2.0
        overrides["saturation"] = 0.5
    elif attack > 15:
        overrides["attack_ms"] = 25.0
        overrides["saturation"] = 0.05
    else:
        overrides["attack_ms"] = 8.0

    if hpr < 0.4:
        overrides["noise_mix"] = 0.3

    # Determine dominant family
    if centroid < 2000 and trans < 4:
        dominant_cats = ["basses_808", "keys", "basses_sub"]
    elif centroid > 6000 and attack < 5:
        dominant_cats = ["synths", "hats", "fx_noise"]
    elif 2000 < centroid < 5000 and attack < 8:
        dominant_cats = ["synths", "keys", "guitars"]
    elif attack < 3 and trans > 4:
        dominant_cats = ["kicks", "claps", "snares", "percs"]
    else:
        dominant_cats = ["keys", "synths", "basses_808"]

    all_cats = dominant_cats + ["kicks", "claps", "hats", "basses_808",
                                 "synths", "impacts", "textures"]
    cats = CategoryCounts()
    total = 0
    for cat_name in all_cats:
        if total >= count:
            break
        this_count = max(1, int(count / len(all_cats)))
        if cat_name == "kicks" and hasattr(cats, "kicks"):
            cats.kicks = this_count
        elif cat_name == "claps" and hasattr(cats, "claps"):
            cats.claps = this_count
        elif cat_name == "snares" and hasattr(cats, "snares"):
            cats.snares = this_count
        elif cat_name == "hats" and hasattr(cats, "hats"):
            cats.hats = this_count
        elif cat_name == "basses_808" and hasattr(cats, "basses_808"):
            cats.basses_808 = this_count
        elif cat_name == "basses_sub" and hasattr(cats, "basses_sub"):
            cats.basses_sub = this_count
        elif cat_name == "keys" and hasattr(cats, "keys"):
            cats.keys = this_count
        elif cat_name == "synths" and hasattr(cats, "synths"):
            cats.synths = this_count
        elif cat_name == "guitars" and hasattr(cats, "guitars"):
            cats.guitars = this_count
        elif cat_name == "impacts" and hasattr(cats, "impacts"):
            cats.impacts = this_count
        elif cat_name == "textures" and hasattr(cats, "textures"):
            cats.textures = this_count
        elif cat_name == "percs" and hasattr(cats, "percs"):
            cats.percs = this_count
        elif cat_name == "fx_noise" and hasattr(cats, "fx_noise"):
            cats.fx_noise = this_count
        total += this_count

    spec = KitSpec(
        name=_safe_filename(sample_path.stem)[:25] + "_mini_kit",
        prompt=f"mini kit from {sample_path.stem}",
        source_refs=[str(sample_path)],
        target_dna=DNAProfile(
            spectral_centroid_target=centroid,
            transient_aggression=min(trans / 10.0, 1.0),
            saturation_density=feats.get("saturation_density", 0.3) if "saturation_density" in feats else 0.3,
            stereo_width=0.3,
            tonal_noise_ratio=hpr,
            loudness_lufs=-14,
            brightness=min(centroid / 8000.0, 1.0),
            darkness=min(1.0 - centroid / 8000.0, 1.0),
            grit=0.3,
            dryness=0.6,
            punch=min(trans / 10.0, 1.0),
            softness=min(1.0 - trans / 10.0, 1.0),
            analog_warmth=0.2 if hpr > 0.6 else 0.0,
            width_span=0.3,
        ),
        categories=cats,
        total_target=count,
    )

    generated = generate_kit(spec, out_dir, polish=True)
    print(f"\nGenerated {generated}/{count} files → {out_dir}")

    # Show features
    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"Coherence: {coherence.get('overall_coherence', 0):.3f}")
        print(f"Centroid:  {coherence.get('centroid_mean', 0):.0f} Hz (source: {centroid:.0f} Hz)")

    report = {
        "source": str(sample_path),
        "kit_spec": kit_spec_to_dict(spec),
        "generated": generated,
        "coherence": coherence,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    report_path = out_dir / "kit_report.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"Report: {report_path}")


def cmd_plan_kit(args):
    """Show a kit category plan without generating."""
    prompt = " ".join(args.prompt) if isinstance(args.prompt, list) else args.prompt
    spec = infer_spec_from_prompt(prompt)

    print(f"cShot Kit Planner")
    print(f"{'='*60}")
    print(f"Prompt:       {prompt}")
    print(f"Kit name:     {spec.name}")
    print(f"Genre:        {spec.genre}")
    print(f"Style:        {spec.style or '(inferred)'}")
    print(f"Total files:  {spec.total_target}")
    print()
    print("Category Plan:")
    cats = spec.categories
    total = 0
    for field_name, label in [
        ("kicks", "Kicks"), ("snares", "Snares"), ("claps", "Claps"),
        ("hats", "Hats"), ("open_hats", "Open Hats"), ("percs", "Percs"),
        ("basses_808", "808s"), ("basses_sub", "Sub Bass"),
        ("keys", "Keys"), ("synths", "Synths"), ("guitars", "Guitars"),
        ("impacts", "Impacts"), ("risers", "Risers"), ("glitches", "Glitches"),
        ("textures", "Textures"), ("atmospheres", "Atmospheres"), ("fx_noise", "FX Noise"),
    ]:
        count = getattr(cats, field_name, 0)
        if count > 0:
            print(f"  {label:<15s} {count}")
            total += count
    print(f"  {'─'*22}")
    print(f"  {'Total':<15s} {total}")
    print()
    print("DNA Targets:")
    dna = spec.target_dna
    print(f"  Centroid:     {dna.spectral_centroid_target:.0f} Hz")
    print(f"  Transients:   {dna.transient_aggression:.2f}")
    print(f"  Saturation:   {dna.saturation_density:.2f}")
    print(f"  Stereo width: {dna.stereo_width:.2f}")
    print(f"  HPR:          {dna.tonal_noise_ratio:.2f}")
    print(f"  Brightness:   {dna.brightness:.2f}")
    print(f"  Darkness:     {dna.darkness:.2f}")
    print(f"  Punch:        {dna.punch:.2f}")


# ─── Kit Export (Week 18-19) ─────────────────────────────

def setup_kit_export(kit_dir: Path, spec: KitSpec = None, coherence: dict = None):
    """Setup export package for a kit: README, manifest, polish."""
    if coherence is None:
        coherence = compute_kit_coherence(kit_dir)

    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        return None

    categories = {}
    for w in wav_files:
        cat = w.parent.name
        categories.setdefault(cat, []).append(w.name)

    manifest = {
        "kit_name": kit_dir.name,
        "total_files": len(wav_files),
        "categories": {cat: len(files) for cat, files in categories.items()},
        "coherence": coherence,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "sample_rate": 44100,
        "bit_depth": 24,
        "format": "wav",
    }
    if spec:
        manifest["kit_spec"] = kit_spec_to_dict(spec)

    manifest_path = kit_dir / "manifest.json"
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)

    readme_lines = [
        f"# {kit_dir.name.replace('_', ' ').title()}",
        "",
        f"Generated by cShot v0.2.0",
        f"Total files: {len(wav_files)}",
        "",
    ]
    if coherence and "error" not in coherence:
        readme_lines.append("## Coherence Metrics")
        readme_lines.append(f"- Spectral cohesion: {coherence.get('spectral_cohesion', 0):.3f}")
        readme_lines.append(f"- Loudness consistency: {coherence.get('loudness_consistency', 0):.3f}")
        readme_lines.append(f"- Transient consistency: {coherence.get('transient_consistency', 0):.3f}")
        readme_lines.append(f"- Stereo consistency: {coherence.get('stereo_consistency', 0):.3f}")
        readme_lines.append(f"- Category diversity: {coherence.get('category_diversity', 0):.3f}")
        readme_lines.append(f"- Overall coherence: {coherence.get('overall_coherence', 0):.3f}")
        readme_lines.append("")

    readme_lines.append("## Categories")
    for cat, count in sorted(categories.items()):
        readme_lines.append(f"- {cat}: {count} files")
    readme_lines.append("")
    readme_lines.append("## Usage")
    readme_lines.append("Drag and drop into your DAW of choice. All files are normalized, trimmed, and DAW-ready.")
    readme_lines.append("")

    readme_path = kit_dir / "README.md"
    with open(readme_path, "w") as f:
        f.write("\n".join(readme_lines))

    pack_notes = [
        f"# Pack Notes: {kit_dir.name}",
        "",
        "## Description",
        f"{spec.prompt if spec else 'One-shot kit generated by cShot'}",
        "",
        "## Production Tips",
        "- Layer kicks with 808s for thickness",
        "- Use room mics on claps",
        "- Process hats with auto-pan for movement",
        "- Re-pitch 808s to match your key",
        "",
        "## Processing",
        f"- Normalized to -1dB peak",
        "- Trimmed silence",
        "- Fade in/out applied",
        "- Stereo preserved on wide sounds",
        "- Mono below 120Hz",
        "",
    ]
    notes_path = kit_dir / "pack_notes.md"
    with open(notes_path, "w") as f:
        f.write("\n".join(pack_notes))

    cover_prompt = _generate_cover_prompt(kit_dir.name, spec)
    cover_path = kit_dir / "cover_art_prompt.txt"
    with open(cover_path, "w") as f:
        f.write(cover_prompt + "\n")

    return manifest


def _generate_cover_prompt(kit_name: str, spec: KitSpec = None) -> str:
    words = kit_name.replace("_", " ").title()
    genre = spec.genre if spec else "electronic"
    style = spec.style if spec else "modern"
    return (
        f"Album art for a sample pack called '{words}'. "
        f"{genre.title()} music style, {style} aesthetic. "
        f"Dark moody scene, cinematic lighting, "
        f"abstract sound waves, geometric particles, "
        f"deep gradient background, premium feel, "
        f"textured, 3000x3000, no text, no logo."
    )


def cmd_kit_export(args):
    kit_dir = Path(args.kit_folder)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)
    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        print("No .wav files found")
        return
    print(f"Kit Export: {kit_dir}")
    print()
    print(f"Polishing {len(wav_files)} files...")
    passed = failed = 0
    for w in wav_files:
        result = polish_file(w, target_db=-1.0, in_place=True)
        if result["pass"]:
            passed += 1
        else:
            failed += 1
    print(f"  {passed} passed, {failed} failed")
    print()
    coherence = compute_kit_coherence(kit_dir)
    if "error" not in coherence:
        setup_kit_export(kit_dir, coherence=coherence)
    print(f"Export complete:")
    print(f"  {kit_dir}/README.md")
    print(f"  {kit_dir}/manifest.json")
    print(f"  {kit_dir}/pack_notes.md")
    print(f"  {kit_dir}/cover_art_prompt.txt")


# ─── Kit Similarity Guard (Week 20) ──────────────────────

def cmd_kit_similarity(args):
    kit_dir = Path(args.kit_folder)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    refs = args.refs or []
    ref_dirs = [Path(r) for r in refs]
    if not ref_dirs:
        print("No reference directories specified. Use --refs <dir>")
        return

    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        print("No .wav files found")
        return

    print(f"Similarity Risk Guard: {kit_dir}")
    print(f"Comparing {len(wav_files)} gen files against {len(ref_dirs)} sources...")
    print()

    all_gen = []
    for w in wav_files:
        result = read_wav(w)
        if result is None:
            continue
        samples, sr = result
        if samples.ndim == 2:
            samples = samples.mean(axis=1)
        feats = compute_features(samples, sr)
        feats["_file"] = str(w.relative_to(kit_dir))
        all_gen.append(feats)

    all_ref = []
    for rd in ref_dirs:
        for w in sorted(rd.rglob("*.wav"))[:100]:
            result = read_wav(w)
            if result is None:
                continue
            samples, sr = result
            if samples.ndim == 2:
                samples = samples.mean(axis=1)
            feats = compute_features(samples, sr)
            feats["_file"] = str(w.relative_to(rd) if rd != w.parent else w.name)
            all_ref.append(feats)

    if not all_ref:
        print("No valid reference features")
        return

    high_risk = []
    for gf in all_gen:
        gv = np.array([gf.get("spectral_centroid", 0) / 10000, gf.get("hpr", 0.5), gf.get("rms", 0) * 10, gf.get("transient_count", 0) / 20])
        min_dist = float("inf")
        nearest = ""
        for rf in all_ref:
            rv = np.array([rf.get("spectral_centroid", 0) / 10000, rf.get("hpr", 0.5), rf.get("rms", 0) * 10, rf.get("transient_count", 0) / 20])
            d = float(np.sqrt(np.sum((gv - rv) ** 2)))
            if d < min_dist:
                min_dist = d
                nearest = rf.get("_file", "?")
        risk = max(0, 1.0 - min_dist)
        if risk > args.threshold:
            high_risk.append({"file": gf["_file"], "risk": round(risk, 3), "nearest": nearest})

    if high_risk:
        print(f"  High risk files ({len(high_risk)}):")
        for hr in sorted(high_risk, key=lambda x: x["risk"], reverse=True):
            print(f"    {hr['file']:<50s} risk={hr['risk']:.3f} nearest={hr['nearest']}")
    else:
        print(f"  No files exceed threshold ({args.threshold})")

    result = {"total": len(all_gen), "refs": len(all_ref), "threshold": args.threshold, "high_risk": high_risk}
    with open(kit_dir / "similarity_risk.json", "w") as f:
        json.dump(result, f, indent=2)


# ─── More Like Kit (Week 27) ─────────────────────────────

def cmd_more_like_kit(args):
    kit_dir = Path(args.kit_folder)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)
    count = args.count
    out_dir = Path(args.out) if args.out else kit_dir.parent / f"{kit_dir.name}_sibling"
    out_dir.mkdir(parents=True, exist_ok=True)

    coherence = compute_kit_coherence(kit_dir)
    if "error" in coherence:
        print(f"Error: {coherence['error']}")
        sys.exit(1)

    c = coherence.get("centroid_mean", 2500)
    h = coherence.get("hpr_mean", 0.6)
    t = coherence.get("transient_mean", 5)

    dna = DNAProfile(spectral_centroid_target=c, transient_aggression=min(t / 10, 1), stereo_width=0.3, tonal_noise_ratio=h)
    cats = CategoryCounts()
    actual = coherence.get("categories", {})
    for cat_field in sorted(vars(cats).keys()):
        for ck in actual:
            if cat_field in ck:
                setattr(cats, cat_field, len(actual[ck]))
                break

    total = sum(v for k, v in vars(cats).items() if isinstance(v, int))
    if total > 0 and total != count:
        ratio = count / total
        for field in vars(cats):
            v = getattr(cats, field)
            if isinstance(v, int):
                setattr(cats, field, max(1, int(round(v * ratio))))

    spec = KitSpec(name=f"{kit_dir.name}_sibling", prompt=f"sibling of {kit_dir.name}", target_dna=dna, categories=cats, total_target=count)
    spec.source_refs = [str(kit_dir)]
    generated = generate_kit(spec, out_dir, polish=True)
    print(f"Generated {generated}/{count} sibling files -> {out_dir}")

    nc = compute_kit_coherence(out_dir)
    if "error" not in nc:
        print(f"Sibling coherence: {nc.get('overall_coherence', 0):.3f} (source: {coherence.get('overall_coherence', 0):.3f})")

    report = {"source": str(kit_dir), "spec": kit_spec_to_dict(spec), "generated": generated, "coherence": nc, "source_coherence": coherence}
    with open(out_dir / "kit_report.json", "w") as f:
        json.dump(report, f, indent=2)


# ─── Merge Kits (Week 28) ───────────────────────────────

def cmd_merge_kits(args):
    ka, kb = Path(args.kit_a), Path(args.kit_b)
    if not ka.exists() or not kb.exists():
        print("One or both kit dirs not found")
        sys.exit(1)
    out = Path(args.out) if args.out else ka.parent / f"{ka.name}_x_{kb.name}"
    out.mkdir(parents=True, exist_ok=True)

    ca = compute_kit_coherence(ka)
    cb = compute_kit_coherence(kb)
    if "error" in ca or "error" in cb:
        print("Could not analyze one or both kits")
        sys.exit(1)

    c_avg = (ca.get("centroid_mean", 2500) + cb.get("centroid_mean", 2500)) / 2
    h_avg = (ca.get("hpr_mean", 0.6) + cb.get("hpr_mean", 0.6)) / 2
    t_avg = (ca.get("transient_mean", 5) + cb.get("transient_mean", 5)) / 2

    dna = DNAProfile(spectral_centroid_target=c_avg, transient_aggression=min(t_avg / 10, 1), stereo_width=0.4, tonal_noise_ratio=h_avg)
    cats = CategoryCounts()
    all_keys = set(list(ca.get("categories", {}).keys()) + list(cb.get("categories", {}).keys()))
    for ck in all_keys:
        na = len(ca.get("categories", {}).get(ck, []))
        nb = len(cb.get("categories", {}).get(ck, []))
        avg = max(1, int(round((na + nb) / 2)))
        for field in vars(cats):
            if field in ck or ck in field:
                setattr(cats, field, avg)

    spec = KitSpec(name=f"{ka.name}_x_{kb.name}", prompt=f"merge of {ka.name} and {kb.name}", target_dna=dna, categories=cats, total_target=sum(v for k, v in vars(cats).items() if isinstance(v, int)))
    spec.source_refs = [str(ka), str(kb)]
    generated = generate_kit(spec, out, polish=True)
    nc = compute_kit_coherence(out)
    print(f"Generated {generated} merged files -> {out}")
    if "error" not in nc:
        print(f"Merged coherence: {nc.get('overall_coherence', 0):.3f}")

    report = {"kit_a": str(ka), "kit_b": str(kb), "spec": kit_spec_to_dict(spec), "generated": generated, "coherence": nc}
    with open(out / "kit_report.json", "w") as f:
        json.dump(report, f, indent=2)


# ─── Kit Presets (Week 29) ──────────────────────────────

def cmd_kit_preset(args):
    presets_dir = REPO_ROOT / "presets" / "kit_presets"
    presets_dir.mkdir(parents=True, exist_ok=True)

    if args.kit_preset_action == "save":
        src = Path(args.source_dir)
        if not src.exists():
            print(f"Error: {src} not found")
            sys.exit(1)
        coherence = compute_kit_coherence(src)
        preset = {"name": args.name or src.name, "source": str(src), "coherence": coherence, "saved_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
        p = presets_dir / f"{_safe_filename(args.name or src.name)}.json"
        with open(p, "w") as f:
            json.dump(preset, f, indent=2)
        print(f"Saved: {p}")

    elif args.kit_preset_action == "list":
        files = sorted(presets_dir.glob("*.json"))
        if not files:
            print("No presets.")
            return
        for pf in files:
            d = json.load(open(pf))
            print(f"  {pf.stem:<30s} source={d.get('source', '?')[:40]}")

    elif args.kit_preset_action == "generate":
        name = args.from_name
        p = presets_dir / f"{_safe_filename(name)}.json"
        if not p.exists():
            print(f"Preset '{name}' not found")
            return
        preset = json.load(open(p))
        count = args.count or 60
        out = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / f"preset_{_safe_filename(name)}"
        cats = CategoryCounts()
        for ck, files in preset.get("coherence", {}).get("categories", {}).items():
            for field in vars(cats):
                if field in ck or ck in field:
                    setattr(cats, field, len(files))
        c = preset.get("coherence", {}).get("centroid_mean", 2500)
        h = preset.get("coherence", {}).get("hpr_mean", 0.6)
        t = preset.get("coherence", {}).get("transient_mean", 5)
        dna = DNAProfile(spectral_centroid_target=c, transient_aggression=min(t / 10, 1), stereo_width=0.3, tonal_noise_ratio=h)
        spec = KitSpec(name=f"preset_{_safe_filename(name)}", prompt=f"from preset {name}", target_dna=dna, categories=cats, total_target=count)
        total = sum(v for k, v in vars(cats).items() if isinstance(v, int))
        if total > 0:
            ratio = count / total
            for field in vars(cats):
                v = getattr(cats, field)
                if isinstance(v, int):
                    setattr(cats, field, max(1, int(round(v * ratio))))
        g = generate_kit(spec, out, polish=True)
        print(f"Generated {g} files -> {out}")

    else:
        print("Usage: cshot kit-preset save|list|generate")


# ─── Batch Kit Factory (Week 30) ─────────────────────────

def cmd_kit_factory(args):
    """Generate multiple kits from a prompts file."""
    prompts_file = Path(args.prompts_file)
    if not prompts_file.exists():
        print(f"Error: {prompts_file} not found")
        sys.exit(1)

    count_per = args.count_per_kit
    out_base = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / "factory"
    out_base.mkdir(parents=True, exist_ok=True)

    with open(prompts_file) as f:
        prompts = [line.strip() for line in f if line.strip()]

    print(f"Batch Kit Factory ({len(prompts)} prompts)")
    print(f"{'='*60}")
    print()

    all_reports = []
    for i, prompt in enumerate(prompts):
        kit_name = _safe_filename(prompt)[:25]
        kit_dir = out_base / f"{i+1:02d}_{kit_name}"
        kit_dir.mkdir(parents=True, exist_ok=True)

        print(f"[{i+1}/{len(prompts)}] {prompt}")
        print(f"{'-'*40}")

        spec = infer_spec_from_prompt(prompt)
        total = sum(v for k, v in vars(spec.categories).items() if isinstance(v, int))
        if total > 0 and total != count_per:
            ratio = count_per / total
            for field in vars(spec.categories):
                current = getattr(spec.categories, field)
                if isinstance(current, int):
                    setattr(spec.categories, field, max(1, int(round(current * ratio))))
        spec.total_target = count_per

        generated = generate_kit(spec, kit_dir, polish=True)
        print(f"  => {generated}/{count_per} files -> {kit_dir}")
        print()

        coherence = compute_kit_coherence(kit_dir)
        if "error" in coherence:
            coherence = {"overall_coherence": 0, "error": coherence.get("error", "")}

        setup_kit_export(kit_dir, spec, coherence)

        report = {"prompt": prompt, "dir": str(kit_dir), "generated": generated, "coherence": coherence}
        all_reports.append(report)

    summary_path = out_base / "factory_report.json"
    summary = {"total_kits": len(all_reports), "kits": all_reports, "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
    with open(summary_path, "w") as f:
        json.dump(summary, f, indent=2)

    print(f"\n{'='*60}")
    print(f"Factory complete: {len(all_reports)} kits")
    avg_coherence = sum(r["coherence"].get("overall_coherence", 0) for r in all_reports) / max(len(all_reports), 1)
    print(f"Average coherence: {avg_coherence:.3f}")
    print(f"Summary: {summary_path}")


# ─── Export Kit alias (Week 40) ──────────────────────────

def cmd_export_kit(args):
    """Alias for kit-export with additional metadata sidecars."""
    kit_dir = Path(args.kit_folder)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        print("No .wav files found")
        return

    print(f"Export Kit: {kit_dir}")
    print(f"  Files: {len(wav_files)}")
    print()

    coherence = compute_kit_coherence(kit_dir)
    if "error" not in coherence:
        setup_kit_export(kit_dir, coherence=coherence)

    for w in wav_files:
        result = read_wav(w)
        if result is None:
            continue
        samples, sr = result
        samples = trim_silence(samples, -60)
        samples = apply_fade(samples, 3, 5)
        samples = normalize_peak(samples, -1.0)
        write_wav(w, samples)

    print(f"All files polished and exported in {kit_dir}")
    print(f"  README.md, manifest.json, pack_notes.md, cover_art_prompt.txt")
