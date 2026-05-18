"""
Weeks 5-8 — Recreation Engine
Nearest-neighbor reconstruction, envelope/harmonic/noise layer matching.
"""

import json
import math
import random
import sys
import time
import traceback
from collections import defaultdict
from pathlib import Path

import numpy as np

from gen import REPO_ROOT, SAMPLE_RATE, TAURI_DIR
from gen.features import (
    compute_features, compute_spectral_centroid, compute_spectral_bandwidth,
    compute_hpr, compute_attack_time, compute_decay_length,
    compute_rms, compute_peak, detect_transients, compute_spectral_flux,
    count_amplitude_peaks, compute_pitch_confidence, compute_mfccs,
    compute_early_rms, compute_noise_floor_estimate,
    compute_stereo_correlation, hz_to_midi, midi_to_note,
    detect_pitch_full, FEATURE_KEYS_FULL,
)
from gen.pack_census import compute_crest_factor
from gen.io import read_audio_safe, write_wav, read_wav
from gen.pack_census import SafeEncoder
from gen.semantics import extract_semantic_tags, guess_sonic_family

CENSUS_DIR = REPO_ROOT / "gen" / "census"


def _load_json(path: Path):
    if path.exists():
        with open(path) as f:
            return json.load(f)
    return None


def _euc_dist(a: dict, b: dict, keys: list[str]) -> float:
    d = 0.0
    n = 0
    for k in keys:
        va = a.get(k)
        vb = b.get(k)
        if va is not None and vb is not None and isinstance(va, (int, float)) and isinstance(vb, (int, float)):
            d += (float(va) - float(vb)) ** 2
            n += 1
    return math.sqrt(d / max(n, 1))


COMPARE_KEYS = [
    "spectral_centroid", "spectral_bandwidth", "zero_crossing_rate",
    "low_band_energy", "mid_band_energy", "high_band_energy",
    "transient_count", "decay_length_ms", "attack_ms",
    "hpr", "pitch_hz", "crest_factor",
    "mfcc_1", "mfcc_2", "mfcc_3", "mfcc_4", "mfcc_5",
]


# ─── Phase 1: Source Analysis ──────────────────────────────

def analyze_envelope(samples: np.ndarray, sr: int) -> dict:
    env = np.abs(samples)
    peak = np.max(env)
    if peak < 0.001:
        return {"attack_ms": 0, "decay_ms": 0, "sustain_ms": 0, "release_ms": 0,
                "peak_idx": 0, "burst_count": 0, "total_duration_ms": len(samples) / sr * 1000}

    total_ms = len(samples) / sr * 1000
    t_10 = peak * 0.1
    t_90 = peak * 0.9
    t_30 = peak * 0.3

    onset = np.argmax(env >= t_10) if np.any(env >= t_10) else 0
    attack_end = np.argmax(env >= t_90) if np.any(env >= t_90) else 0
    attack_ms = (attack_end - onset) / sr * 1000 if attack_end > onset else 1.0

    peak_idx = np.argmax(env)
    decay_point = None
    for i in range(peak_idx, len(env)):
        if env[i] <= t_30:
            decay_point = i
            break
    if decay_point is None:
        decay_point = len(env) - 1
    decay_ms = (decay_point - peak_idx) / sr * 1000

    sustain_point = None
    for i in range(decay_point, len(env)):
        if env[i] <= t_10:
            sustain_point = i
            break
    if sustain_point is None:
        sustain_point = len(env) - 1
    sustain_ms = (sustain_point - decay_point) / sr * 1000 if sustain_point > decay_point else 0
    release_ms = (len(env) - sustain_point) / sr * 1000 if sustain_point < len(env) else 0

    burst_count = count_amplitude_peaks(samples, sr)

    return {
        "attack_ms": round(attack_ms, 2),
        "decay_ms": round(decay_ms, 2),
        "sustain_ms": round(sustain_ms, 2),
        "release_ms": round(release_ms, 2),
        "peak_idx": int(peak_idx),
        "burst_count": burst_count,
        "total_duration_ms": round(total_ms, 1),
        "attack_shape": "sharp" if attack_ms < 5 else "moderate" if attack_ms < 30 else "slow",
        "decay_shape": "short" if decay_ms < 50 else "medium" if decay_ms < 300 else "long",
    }


def analyze_harmonic(samples: np.ndarray, sr: int) -> dict:
    pitch_info = detect_pitch_full(samples, sr)
    hpr = compute_hpr(samples, sr)
    centroid = compute_spectral_centroid(samples, sr)
    bw = compute_spectral_bandwidth(samples, sr)
    early = compute_early_rms(samples, sr)

    n = min(len(samples), 8192)
    if n < 256:
        return {"pitch_hz": 0, "hpr": 0.5, "family": "noise", "centroid": 0}

    spectrum = np.abs(np.fft.rfft(samples[:n]))
    freqs = np.fft.rfftfreq(n, 1/sr)
    total = np.sum(spectrum)
    if total < 1e-10:
        return {"pitch_hz": 0, "hpr": 0.5, "family": "noise", "centroid": 0}

    sub_energy = float(np.sum(spectrum[freqs < 80]) / max(total, 1e-10))
    low_energy = float(np.sum(spectrum[(freqs >= 80) & (freqs < 300)]) / max(total, 1e-10))
    mid_energy = float(np.sum(spectrum[(freqs >= 300) & (freqs < 3000)]) / max(total, 1e-10))
    high_energy = float(np.sum(spectrum[freqs >= 3000]) / max(total, 1e-10))

    harmonic_peaks = []
    pk = np.max(spectrum)
    if pk > 0:
        threshold = pk * 0.05
        for freq_idx in range(1, len(spectrum) - 1):
            if spectrum[freq_idx] > spectrum[freq_idx - 1] and spectrum[freq_idx] >= spectrum[freq_idx + 1] and spectrum[freq_idx] >= threshold:
                harmonic_peaks.append(float(freqs[freq_idx]))

    h = pitch_info["pitch_hz"]
    family = "noise"
    if hpr > 0.55 and h > 20:
        if h < 120 and sub_energy > 0.3:
            family = "sub_bass"
        elif h < 300:
            family = "tonal_bass"
        elif centroid < 3000:
            family = "tonal_mid"
        else:
            family = "tonal_high"
    elif hpr > 0.35:
        family = "mixed"
    else:
        if centroid > 4000 and high_energy > 0.4:
            family = "noise_high"
        elif sub_energy > 0.3:
            family = "noise_sub"
        else:
            family = "noise"

    if early > 2.0 and hpr < 0.4:
        family = "percussive_noise"

    return {
        "pitch_hz": pitch_info["pitch_hz"],
        "midi_note": pitch_info["midi_note"],
        "note_name": pitch_info["note_name"],
        "pitch_confidence": pitch_info["confidence"],
        "hpr": round(hpr, 3),
        "spectral_centroid": round(centroid, 1),
        "spectral_bandwidth": round(bw, 1),
        "sub_energy": round(sub_energy, 3),
        "low_energy": round(low_energy, 3),
        "mid_energy": round(mid_energy, 3),
        "high_energy": round(high_energy, 3),
        "harmonic_peaks": harmonic_peaks[:20],
        "family": family,
        "num_harmonic_peaks": len(harmonic_peaks),
    }


def analyze_noise(samples: np.ndarray, sr: int) -> dict:
    n = len(samples)
    if n < 1024:
        return {"noise_color": "white", "air_pct": 0, "texture_pct": 0, "tail_noise_pct": 0}

    attack = compute_attack_time(samples, sr)
    decay = compute_decay_length(samples, sr)
    peak_idx = np.argmax(np.abs(samples))

    tail_start = min(peak_idx + int(decay / 1000 * sr * 0.5), n)
    tail_samples = samples[tail_start:] if tail_start < n else np.zeros(100)

    if len(tail_samples) < 100:
        tail_noise_pct = 0
    else:
        tail_rms = compute_rms(tail_samples)
        total_rms = compute_rms(samples)
        tail_noise_pct = float(tail_rms / max(total_rms, 1e-10))

    n_fft = min(n, 4096)
    spectrum = np.abs(np.fft.rfft(samples[:n_fft]))
    freqs = np.fft.rfftfreq(n_fft, 1/sr)
    total = np.sum(spectrum)

    if total < 1e-10:
        return {"noise_color": "white", "air_pct": 0, "texture_pct": 0, "tail_noise_pct": 0}

    low_sum = np.sum(spectrum[freqs < 500])
    mid_sum = np.sum(spectrum[(freqs >= 500) & (freqs < 5000)])
    high_sum = np.sum(spectrum[freqs >= 5000])

    noise_color = "white"
    if high_sum > mid_sum * 2 and high_sum > low_sum * 2:
        noise_color = "blue"
    elif high_sum > mid_sum * 1.5:
        noise_color = "pinkish"
    elif low_sum > mid_sum * 2 and low_sum > high_sum * 2:
        noise_color = "brown"
    elif low_sum > mid_sum * 1.5:
        noise_color = "brownish"
    elif mid_sum > high_sum and mid_sum > low_sum:
        noise_color = "white"

    air_pct = float(high_sum / max(total, 1e-10))
    texture_pct = float(np.sum(spectrum[1::2]) / max(total, 1e-10))

    zcr = float(np.mean(np.abs(np.diff(np.signbit(samples[:min(n, 4410)]))))) if n > 10 else 0
    roughness = zcr * 10

    return {
        "noise_color": noise_color,
        "air_pct": round(air_pct, 4),
        "texture_pct": round(texture_pct, 4),
        "tail_noise_pct": round(tail_noise_pct, 4),
        "roughness": round(roughness, 3),
        "low_spectral_pct": round(float(low_sum / max(total, 1e-10)), 4),
        "mid_spectral_pct": round(float(mid_sum / max(total, 1e-10)), 4),
        "high_spectral_pct": round(float(high_sum / max(total, 1e-10)), 4),
    }


def analyze_source(wav_path: Path) -> dict:
    result = read_audio_safe(wav_path, mono=True)
    if result is None:
        print(f"  Error: cannot read {wav_path}")
        return None
    samples, sr = result

    base = compute_features(samples, sr)
    base["crest_factor"] = float(compute_peak(samples) / max(compute_rms(samples), 1e-10))

    envelope = analyze_envelope(samples, sr)
    harmonic = analyze_harmonic(samples, sr)
    noise = analyze_noise(samples, sr)

    family_path = ""
    for p in wav_path.parents:
        if p.name.lower() in ("packs",):
            break
        if family_path:
            family_path = p.name + "/" + family_path
        else:
            family_path = p.name

    tags = extract_semantic_tags(wav_path.stem, wav_path.parent.name, family_path)
    sonic_family = guess_sonic_family(tags, base.get("category", "other"), family_path)

    name_tokens = wav_path.stem.lower().split()
    is_drum = any(t in name_tokens for t in ["kick", "snare", "clap", "hat", "hihat", "hh", "oh", "rim", "tom", "perc"])
    is_bass = any(t in name_tokens for t in ["bass", "808", "sub", "reese"])
    is_synth = any(t in name_tokens for t in ["synth", "stab", "lead", "pluck", "pad", "chord"])
    is_piano = any(t in name_tokens for t in ["piano", "keys", "rhodes", "wurly", "ep"])
    is_guitar = any(t in name_tokens for t in ["guitar", "nylon", "acoustic"])
    is_fx = any(t in name_tokens for t in ["fx", "impact", "riser", "glitch", "noise"])

    return {
        "file_path": str(wav_path),
        "filename": wav_path.name,
        "duration_ms": base["duration_ms"],
        "features": base,
        "envelope": envelope,
        "harmonic": harmonic,
        "noise": noise,
        "semantic_tags": tags,
        "sonic_family": sonic_family,
        "file_hints": {
            "is_drum": is_drum,
            "is_bass": is_bass,
            "is_synth": is_synth,
            "is_piano": is_piano,
            "is_guitar": is_guitar,
            "is_fx": is_fx,
        },
    }


# ─── Phase 2: Neighbor Search ────────────────────────────

def find_nearest_neighbors(analysis: dict, n: int = 8) -> list[dict]:
    pack_index = _load_json(CENSUS_DIR / "pack_index.json")
    if not pack_index:
        return []

    source_feats = analysis["features"]
    files = pack_index.get("files", {})
    scored = []
    for path, entry in files.items():
        if "error" in entry:
            continue
        dist = _euc_dist(source_feats, entry, COMPARE_KEYS)
        scored.append((dist, path, entry))

    scored.sort(key=lambda x: x[0])
    nearest = []
    for dist, path, entry in scored[:n]:
        nearest.append({
            "file_path": path,
            "distance": round(dist, 4),
            "category": entry.get("category", "?"),
            "pack": entry.get("pack", "?"),
            "family": entry.get("family", "?"),
            "sonic_family": entry.get("sonic_family", "?"),
        })

    return nearest


# ─── Phase 3: Generator Routing ──────────────────────────

def infer_generator(analysis: dict, neighbors: list[dict]) -> dict:
    hints = analysis.get("file_hints", {})
    harmonic = analysis["harmonic"]
    envelope = analysis["envelope"]
    tags = analysis.get("semantic_tags", {})
    sonic_family = analysis.get("sonic_family", "other")
    features = analysis["features"]

    neighbor_cats = [n["category"] for n in neighbors[:5]]
    neighbor_fams = [n["sonic_family"] for n in neighbors[:5]]

    from collections import Counter
    cat_votes = Counter(neighbor_cats)
    fam_votes = Counter(neighbor_fams)
    top_cat = cat_votes.most_common(1)[0][0] if cat_votes else "other"
    top_fam = fam_votes.most_common(1)[0][0] if fam_votes else "other"

    source_sonic = sonic_family

    gen_family = None
    profile = None
    confidence = 0.0

    if source_sonic.startswith("bass") or source_sonic in ("other_808",):
        gen_family = "bass"
        if "808" in source_sonic or "sub" in source_sonic:
            profile = "808"
        elif "reese" in source_sonic:
            profile = "reese"
        elif "distorted" in source_sonic:
            profile = "distorted"
        else:
            profile = "808"
        confidence = 0.9

    elif source_sonic.startswith("tonal_piano"):
        gen_family = "piano"
        if harmonic["hpr"] > 0.7:
            profile = "acoustic" if harmonic["pitch_confidence"] > 0.5 else "bell"
        elif harmonic["spectral_centroid"] > 3000:
            profile = "bright"
        else:
            profile = "soft"
        confidence = 0.9

    elif source_sonic.startswith("tonal_guitar"):
        gen_family = "guitar"
        env_shape = envelope["attack_shape"]
        profile = "nylon" if env_shape == "sharp" else "bright" if env_shape == "moderate" else "dark"
        confidence = 0.9

    elif source_sonic.startswith("tonal_synth") or source_sonic.startswith("other_synth"):
        gen_family = "synth"
        env_shape = envelope["attack_shape"]
        if env_shape == "sharp":
            profile = "stab" if features.get("transient_count", 0) < 2 else "pluck"
        elif env_shape == "slow":
            profile = "pad"
        else:
            profile = "lead"
        if harmonic["pitch_hz"] < 100:
            profile = "bass"
        confidence = 0.9

    elif source_sonic.startswith("fx"):
        gen_family = "fx"
        if envelope["decay_ms"] > 500:
            profile = "impact" if envelope["attack_ms"] < 20 else "riser"
        elif envelope["burst_count"] > 3:
            profile = "glitch"
        else:
            profile = "impact"
        confidence = 0.85

    elif hints["is_drum"] or top_cat in ("kick", "snare", "clap", "closed_hat", "open_hat", "percussion"):
        gen_family = "drum"
        subtype_map = {"kick": "kick", "snare": "snare", "clap": "clap", "closed_hat": "closed_hat", "open_hat": "open_hat", "percussion": "impact_fx"}
        profile = subtype_map.get(top_cat, "kick")
        confidence = 0.8

    elif hints["is_bass"] or top_fam.startswith("bass") or harmonic["family"] in ("sub_bass", "tonal_bass"):
        gen_family = "bass"
        if harmonic["pitch_hz"] < 60 and harmonic["hpr"] > 0.6:
            profile = "808"
        elif harmonic["hpr"] > 0.4 and harmonic["spectral_centroid"] < 2000:
            profile = "reese"
        elif features.get("saturation_density", 0) > 0.02:
            profile = "distorted"
        else:
            profile = "pluck"
        confidence = 0.85

    elif hints["is_synth"] or top_fam.startswith("tonal_synth"):
        gen_family = "synth"
        env = envelope["attack_shape"]
        if env == "sharp":
            profile = "stab" if features.get("transient_count", 0) < 2 else "pluck"
        elif env == "slow":
            profile = "pad"
        else:
            profile = "lead"
        if harmonic["pitch_hz"] < 100:
            profile = "bass"
        confidence = 0.8

    elif hints["is_piano"] or top_fam.startswith("tonal_piano"):
        gen_family = "piano"
        if harmonic["hpr"] > 0.7:
            profile = "acoustic" if harmonic["pitch_confidence"] > 0.5 else "bell"
        elif harmonic["spectral_centroid"] > 3000:
            profile = "bright"
        else:
            profile = "soft"
        confidence = 0.85

    elif hints["is_guitar"] or top_fam.startswith("tonal_guitar"):
        gen_family = "guitar"
        env = envelope["attack_shape"]
        profile = "nylon" if env == "sharp" else "bright" if env == "moderate" else "dark"
        confidence = 0.8

    elif hints["is_fx"] or top_fam.startswith("fx") or top_cat == "fx":
        gen_family = "fx"
        if envelope["decay_ms"] > 500:
            profile = "impact" if envelope["attack_ms"] < 20 else "riser"
        elif envelope["burst_count"] > 3:
            profile = "glitch"
        else:
            profile = "impact"
        confidence = 0.75

    else:
        hf = harmonic["family"]
        if hf.startswith("tonal") or harmonic["hpr"] > 0.6:
            gen_family = "synth" if harmonic["spectral_centroid"] > 2000 else "bass"
            profile = "lead" if gen_family == "synth" else "808"
        elif hf.startswith("noise") or harmonic["hpr"] < 0.35:
            gen_family = "fx"
            profile = "impact"
        else:
            gen_family = "synth"
            profile = "pluck"
        confidence = 0.5

    return {
        "generator_family": gen_family,
        "generator_profile": profile,
        "confidence": round(confidence, 2),
        "top_category": top_cat,
        "top_sonic_family": top_fam,
        "neighbor_category_votes": dict(cat_votes.most_common(5)),
        "neighbor_family_votes": dict(fam_votes.most_common(5)),
    }


# ─── Phase 4: Target Profile ──────────────────────────────

def build_target_profile(analysis: dict, gen_route: dict) -> dict:
    feats = analysis["features"]
    env = analysis["envelope"]
    harm = analysis["harmonic"]
    noise = analysis["noise"]

    profile = {
        "duration_ms": max(100, feats.get("duration_ms", 500) * random.uniform(0.7, 1.3)),
        "pitch_hz": max(20, harm["pitch_hz"] * random.uniform(0.8, 1.2)) if harm["pitch_hz"] > 0 else 220.0,
        "attack_ms": max(0.5, env["attack_ms"] * random.uniform(0.6, 1.5)),
        "decay_ms": max(5, env["decay_ms"] * random.uniform(0.7, 1.4)),
        "stereo_width": min(1.0, max(0.0, feats.get("stereo_width", 0) * random.uniform(0.5, 1.5))),
        "saturation": min(1.0, max(0.0, feats.get("saturation_density", 0) * random.uniform(0.5, 2.0) * 2)),
        "hpr_target": harm["hpr"],
        "centroid_target": harm["spectral_centroid"],
        "brightness": min(2.0, max(0.2, harm["spectral_centroid"] / 4000)),
        "drive": min(1.0, max(0.0, feats.get("saturation_density", 0) * 5)),
        "noise_color": noise["noise_color"],
        "air": noise["air_pct"],
        "style_profile": None,
    }

    if gen_route["generator_family"] == "drum":
        profile["duration_ms"] = max(50, min(1500, env["total_duration_ms"] * random.uniform(0.8, 1.3)))
        profile["pitch_hz"] = max(30, min(2000, harm["pitch_hz"] * random.uniform(0.7, 1.5))) if harm["pitch_hz"] > 0 else 200.0

    elif gen_route["generator_family"] == "bass":
        profile["pitch_hz"] = max(30, min(200, harm["pitch_hz"] * random.uniform(0.7, 1.3))) if harm["pitch_hz"] > 20 else 55.0
        profile["glide"] = max(-0.5, min(0.5, (env["attack_ms"] / 100) * 0.3 - 0.1))

    elif gen_route["generator_family"] == "piano":
        profile["pitch_hz"] = max(60, min(2000, harm["pitch_hz"] * random.uniform(0.8, 1.2))) if harm["pitch_hz"] > 20 else 261.63

    elif gen_route["generator_family"] == "guitar":
        profile["pitch_hz"] = max(80, min(1200, harm["pitch_hz"] * random.uniform(0.8, 1.2))) if harm["pitch_hz"] > 20 else 220.0

    return profile


# ─── Phase 5: Generation ──────────────────────────────────

def _call_generator(gen_family: str, profile_name: str, target: dict,
                    variation_idx: int, out_dir: Path) -> tuple:
    from gen.synth_gen import synthesize_synth, SYNTH_PROFILES
    from gen.piano_gen import synthesize_piano_stab, PIANO_PROFILES
    from gen.guitar_gen import synthesize_guitar_stab, GUITAR_PROFILES
    from gen.bass_gen import synthesize_bass, BASS_PROFILES
    from gen.fx_gen import synthesize_fx, FX_PROFILES
    from gen.synthesis import (
        synthesize_kick, synthesize_snare, synthesize_clap,
        synthesize_closed_hat, synthesize_open_hat, synthesize_808,
        synthesize_bass_stab, synthesize_impact_fx, synthesize_synth_stab,
    )
    from gen.drum_gen import ENHANCED_DRUM_REGISTRY
    from gen.tonal_gen import TONAL_REGISTRY

    duration_ms = target.get("duration_ms", 500)
    pitch_hz = target.get("pitch_hz", 220.0)

    seed = int(time.time() * 1000000 + variation_idx) % (2 ** 31)
    random.seed(seed)
    np.random.seed(seed % 2 ** 32)

    OVERRIDE_MAP = {
        "bass": BASS_PROFILES,
        "synth": SYNTH_PROFILES,
        "piano": PIANO_PROFILES,
        "guitar": GUITAR_PROFILES,
        "fx": FX_PROFILES,
    }

    samples = None
    metadata_overrides = {}

    if gen_family == "drum":
        style_profile = target.get("style_profile")
        drum_map = {
            "kick": (synthesize_kick, {"pitch_hz": min(200, pitch_hz)}),
            "snare": (synthesize_snare, {"pitch_hz": min(400, pitch_hz)}),
            "clap": (synthesize_clap, {}),
            "closed_hat": (synthesize_closed_hat, {"pitch_hz": min(2000, max(500, pitch_hz))}),
            "open_hat": (synthesize_open_hat, {"pitch_hz": min(1000, max(200, pitch_hz))}),
            "808": (synthesize_808, {"pitch_hz": min(120, pitch_hz)}),
            "impact_fx": (synthesize_impact_fx, {}),
        }
        enhanced = ENHANCED_DRUM_REGISTRY
        if profile_name in enhanced and style_profile:
            fn = enhanced[profile_name]
            dur = max(50, min(1500, duration_ms))
            try:
                samples = fn(duration_ms=dur, pitch_hz=min(2000, max(20, pitch_hz)),
                             style_profile=style_profile)
            except Exception as e:
                return None, str(e)
        elif profile_name in drum_map:
            fn, overrides = drum_map[profile_name]
            dur = max(50, min(1500, duration_ms))
            overrides.update(metadata_overrides)
            try:
                samples = fn(duration_ms=dur, **overrides)
            except Exception as e:
                return None, str(e)
        else:
            samples = synthesize_impact_fx(duration_ms=max(50, min(1500, duration_ms)))

    elif gen_family == "bass":
        profile_map = {"808": "808", "reese": "reese", "distorted": "distorted", "pluck": "pluck", "fm": "fm"}
        pn = profile_map.get(profile_name, "808")
        overrides = {
            "drive": target.get("drive", 0.3),
            "glide": target.get("glide", 0.0),
            "sub_body_balance": target.get("hpr_target", 0.5),
        }
        try:
            samples = synthesize_bass(duration_ms, pitch_hz, pn, overrides=overrides)
        except Exception as e:
            samples = synthesize_bass(duration_ms, pitch_hz, "808")

    elif gen_family == "synth":
        style_profile = target.get("style_profile")
        if style_profile:
            try:
                enhanced = TONAL_REGISTRY.get("synth")
                osc_types = {"stab": "saw", "pluck": "square", "pad": "sine", "lead": "saw", "bass": "bass", "chord": "saw"}
                samples = enhanced(
                    duration_ms=duration_ms, pitch_hz=pitch_hz,
                    osc_type=osc_types.get(profile_name, "saw"),
                    filter_cutoff=target.get("brightness", 0.6),
                    attack_ms=target.get("attack_ms", 10),
                    decay_ms=target.get("decay_ms", 200),
                    style_profile=style_profile,
                )
            except Exception:
                samples = None
        if samples is None:
            profile_map = {"stab": "stab", "pluck": "pluck", "pad": "pad", "lead": "lead", "bass": "bass", "chord": "chord"}
            pn = profile_map.get(profile_name, "stab")
            overrides = {
                "attack_ms": target.get("attack_ms"),
                "decay_ms": target.get("decay_ms"),
                "saturation": target.get("saturation", 0.2),
                "stereo_width": target.get("stereo_width", 0.3),
            }
            try:
                samples = synthesize_synth(duration_ms, pitch_hz, pn, overrides=overrides)
            except Exception as e:
                samples = synthesize_synth(duration_ms, pitch_hz, "stab")

    elif gen_family == "piano":
        style_profile = target.get("style_profile")
        if style_profile:
            try:
                enhanced = TONAL_REGISTRY.get("piano")
                samples = enhanced(
                    duration_ms=duration_ms, pitch_hz=pitch_hz,
                    hammer_hardness=target.get("attack_strength", 0.5),
                    resonance=0.7, damping=0.3, velocity=0.7,
                    style_profile=style_profile,
                )
            except Exception:
                samples = None
        if samples is None:
            profile_map = {"acoustic": "acoustic", "bright": "bright", "dark": "dark", "soft": "soft", "bell": "bell"}
            pn = profile_map.get(profile_name, "acoustic")
            overrides = {"brightness": target.get("brightness", 1.0)}
            try:
                samples = synthesize_piano_stab(duration_ms, pitch_hz, pn, overrides=overrides)
            except Exception:
                samples = synthesize_piano_stab(duration_ms, pitch_hz, "acoustic")

    elif gen_family == "guitar":
        style_profile = target.get("style_profile")
        if style_profile:
            try:
                enhanced = TONAL_REGISTRY.get("guitar")
                samples = enhanced(
                    duration_ms=duration_ms, pitch_hz=pitch_hz,
                    pick_hardness=0.5 + (target.get("attack_strength", 0.5) * 0.5),
                    body_resonance=0.6, fret_noise=0.3,
                    style_profile=style_profile,
                )
            except Exception:
                samples = None
        if samples is None:
            profile_map = {"nylon": "nylon", "bright": "bright", "dark": "dark", "muted": "muted", "processed": "processed"}
            pn = profile_map.get(profile_name, "nylon")
            try:
                samples = synthesize_guitar_stab(duration_ms, pitch_hz, pn)
            except Exception:
                samples = synthesize_guitar_stab(duration_ms, pitch_hz, "nylon")

    elif gen_family == "fx":
        profile_map = {"impact": "impact", "riser": "riser", "glitch": "glitch", "noise_hit": "noise_hit"}
        pn = profile_map.get(profile_name, "impact")
        try:
            samples = synthesize_fx(duration_ms, pitch_hz, pn)
        except Exception as e:
            samples = synthesize_fx(duration_ms, pitch_hz, "impact")

    else:
        samples = synthesize_synth(duration_ms, pitch_hz, "stab")

    if samples is None or len(samples) < 100:
        return None, "generation_failed"

    return samples, None


# ─── Phase 6: Similarity Scoring ──────────────────────────

def envelope_similarity(src_env: dict, gen_samples: np.ndarray, sr: int) -> dict:
    gen_env = analyze_envelope(gen_samples, sr)

    a_diff = abs(src_env["attack_ms"] - gen_env["attack_ms"]) / max(src_env["attack_ms"], 1)
    d_diff = abs(src_env["decay_ms"] - gen_env["decay_ms"]) / max(src_env["decay_ms"], 1)
    t_diff = abs(src_env["total_duration_ms"] - gen_env["total_duration_ms"]) / max(src_env["total_duration_ms"], 1)
    b_diff = abs(src_env["burst_count"] - gen_env["burst_count"]) / max(src_env["burst_count"], 1)

    score = 1.0 - min(1.0, (a_diff * 0.3 + d_diff * 0.3 + t_diff * 0.2 + b_diff * 0.2))
    score = max(0.0, min(1.0, score))

    return {
        "score": round(score, 4),
        "source_attack_ms": src_env["attack_ms"],
        "gen_attack_ms": gen_env["attack_ms"],
        "source_decay_ms": src_env["decay_ms"],
        "gen_decay_ms": gen_env["decay_ms"],
        "source_burst_count": src_env["burst_count"],
        "gen_burst_count": gen_env["burst_count"],
    }


def harmonic_similarity(src_harm: dict, gen_samples: np.ndarray, sr: int) -> dict:
    gen_harm = analyze_harmonic(gen_samples, sr)

    p_diff = abs(src_harm["pitch_hz"] - gen_harm["pitch_hz"]) / max(src_harm["pitch_hz"], 20)
    hpr_diff = abs(src_harm["hpr"] - gen_harm["hpr"])
    cent_diff = abs(src_harm["spectral_centroid"] - gen_harm["spectral_centroid"]) / max(src_harm["spectral_centroid"], 100)
    sub_diff = abs(src_harm["sub_energy"] - gen_harm["sub_energy"])
    high_diff = abs(src_harm["high_energy"] - gen_harm["high_energy"])

    score = 1.0 - min(1.0, (p_diff * 0.25 + hpr_diff * 0.25 + cent_diff * 0.2 + sub_diff * 0.15 + high_diff * 0.15))
    score = max(0.0, min(1.0, score))

    return {
        "score": round(score, 4),
        "source_hpr": src_harm["hpr"],
        "gen_hpr": gen_harm["hpr"],
        "source_centroid": src_harm["spectral_centroid"],
        "gen_centroid": gen_harm["spectral_centroid"],
        "source_family": src_harm["family"],
        "gen_family": gen_harm["family"],
    }


def noise_similarity(src_noise: dict, gen_samples: np.ndarray, sr: int) -> dict:
    gen_noise = analyze_noise(gen_samples, sr)

    air_diff = abs(src_noise["air_pct"] - gen_noise["air_pct"])
    tex_diff = abs(src_noise["texture_pct"] - gen_noise["texture_pct"])
    tail_diff = abs(src_noise["tail_noise_pct"] - gen_noise["tail_noise_pct"])
    low_diff = abs(src_noise["low_spectral_pct"] - gen_noise["low_spectral_pct"])
    high_diff = abs(src_noise["high_spectral_pct"] - gen_noise["high_spectral_pct"])
    color_match = 1.0 if src_noise["noise_color"] == gen_noise["noise_color"] else 0.5

    score = 1.0 - min(1.0, (air_diff * 0.15 + tex_diff * 0.15 + tail_diff * 0.15 +
                             low_diff * 0.2 + high_diff * 0.2 + (1 - color_match) * 0.15))
    score = max(0.0, min(1.0, score))

    return {
        "score": round(score, 4),
        "source_color": src_noise["noise_color"],
        "gen_color": gen_noise["noise_color"],
        "source_air": src_noise["air_pct"],
        "gen_air": gen_noise["air_pct"],
    }


def compute_overall_score(env_score: float, harm_score: float, noise_score: float) -> dict:
    overall = env_score * 0.35 + harm_score * 0.35 + noise_score * 0.30
    return {
        "overall": round(overall, 4),
        "envelope_weight": 0.35,
        "harmonic_weight": 0.35,
        "noise_weight": 0.30,
        "rating": "excellent" if overall > 0.75 else "good" if overall > 0.55 else "fair" if overall > 0.35 else "poor",
    }


# ─── Main Recreate Command ────────────────────────────────

def cmd_recreate(args):
    wav_path = Path(args.input)
    if not wav_path.exists():
        print(f"Error: {wav_path} not found")
        return

    count = getattr(args, 'count', 5)
    out_dir = Path(getattr(args, 'out', 'outputs/recreate'))
    out_dir.mkdir(parents=True, exist_ok=True)

    source_name = wav_path.stem.replace(" ", "_")
    timestamp = time.strftime("%Y%m%d_%H%M%S")

    print(f"Analyzing: {wav_path.name}")

    analysis = analyze_source(wav_path)
    if analysis is None:
        return

    print(f"  Duration: {analysis['duration_ms']:.0f}ms")
    print(f"  Pitch: {analysis['harmonic']['note_name']} ({analysis['harmonic']['pitch_hz']:.1f}Hz)")
    print(f"  HPR: {analysis['harmonic']['hpr']:.3f}  Family: {analysis['harmonic']['family']}")
    print(f"  Sonic Family: {analysis['sonic_family']}")
    print(f"  Envelope: attack={analysis['envelope']['attack_ms']:.1f}ms, "
          f"decay={analysis['envelope']['decay_ms']:.0f}ms, "
          f"bursts={analysis['envelope']['burst_count']}")

    neighbors = find_nearest_neighbors(analysis, 8)
    if neighbors:
        print(f"\n  Nearest neighbors:")
        for n in neighbors[:5]:
            print(f"    [{n['distance']:.4f}] {Path(n['file_path']).name} ({n['category']}, {n['pack']})")
    else:
        print(f"\n  No pack index found — generating from analysis only")

    gen_route = infer_generator(analysis, neighbors)
    print(f"\n  Generator route: {gen_route['generator_family']}/{gen_route['generator_profile']} "
          f"(confidence={gen_route['confidence']})")

    style_profile = None
    if neighbors:
        source_pack = neighbors[0].get("pack", "")
        style_data = _load_json(CENSUS_DIR / "pack_style_space.json")
        if style_data and source_pack in style_data.get("packs", {}):
            style_profile = style_data["packs"][source_pack].get("centroid")

    generated = []
    for i in range(count):
        target = build_target_profile(analysis, gen_route)
        target["duration_ms"] *= random.uniform(0.8, 1.3)
        target["pitch_hz"] *= random.uniform(0.85, 1.2)
        if style_profile:
            target["style_profile"] = style_profile

        result, error = _call_generator(
            gen_route["generator_family"],
            gen_route["generator_profile"],
            target, i, out_dir
        )
        if result is None:
            print(f"  [{i+1}/{count}] Failed: {error}")
            continue

        env_sim = envelope_similarity(analysis["envelope"], result, SAMPLE_RATE)
        harm_sim = harmonic_similarity(analysis["harmonic"], result, SAMPLE_RATE)
        noise_sim = noise_similarity(analysis["noise"], result, SAMPLE_RATE)
        overall = compute_overall_score(env_sim["score"], harm_sim["score"], noise_sim["score"])

        out_name = f"recreate_{source_name}_{timestamp}_{i+1:03d}.wav"
        out_path = out_dir / out_name
        write_wav(out_path, result)

        meta = {
            "source": str(wav_path),
            "generated": str(out_path),
            "variation": i + 1,
            "generator_family": gen_route["generator_family"],
            "generator_profile": gen_route["generator_profile"],
            "target_pitch_hz": round(target["pitch_hz"], 1),
            "target_duration_ms": round(target["duration_ms"], 1),
            "envelope_similarity": env_sim,
            "harmonic_similarity": harm_sim,
            "noise_similarity": noise_sim,
            "overall_score": overall,
            "nearest_neighbors": neighbors[:5],
            "inferred_sonic_family": analysis["sonic_family"],
            "timestamp": timestamp,
        }

        meta_path = out_dir / f"{out_name}.json"
        with open(meta_path, "w") as f:
            json.dump(meta, f, indent=2, cls=SafeEncoder)

        generated.append(meta)
        print(f"  [{i+1}/{count}] {out_name}  "
              f"env={env_sim['score']:.3f}  "
              f"harm={harm_sim['score']:.3f}  "
              f"noise={noise_sim['score']:.3f}  "
              f"overall={overall['overall']:.3f} ({overall['rating']})")

    report = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "source": str(wav_path),
        "analysis": analysis,
        "generator_route": gen_route,
        "nearest_neighbors": neighbors,
        "results": generated,
        "summary": {
            "total_attempted": count,
            "total_generated": len(generated),
            "avg_envelope_sim": round(float(np.mean([g["envelope_similarity"]["score"] for g in generated])), 4) if generated else 0,
            "avg_harmonic_sim": round(float(np.mean([g["harmonic_similarity"]["score"] for g in generated])), 4) if generated else 0,
            "avg_noise_sim": round(float(np.mean([g["noise_similarity"]["score"] for g in generated])), 4) if generated else 0,
            "avg_overall": round(float(np.mean([g["overall_score"]["overall"] for g in generated])), 4) if generated else 0,
        },
    }

    report_path = out_dir / f"recreate_report_{source_name}_{timestamp}.json"
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2, cls=SafeEncoder)
    print(f"\n  Report: {report_path}")
    print(f"  Generated {len(generated)}/{count} variations in {out_dir}")


def cmd_recreate_folder(args):
    folder = Path(args.folder)
    if not folder.exists():
        print(f"Error: {folder} not found")
        return

    count = getattr(args, 'count', 5)
    out_dir = Path(getattr(args, 'out', 'outputs/recreate_folder'))
    out_dir.mkdir(parents=True, exist_ok=True)

    wavs = sorted(folder.glob("*.wav"))
    if not wavs:
        wavs = sorted(folder.glob("*.WAV"))
    if not wavs:
        wavs = sorted(folder.glob("*.flac"))
    if not wavs:
        wavs = sorted(folder.glob("*.wv"))

    print(f"Recreating {len(wavs)} files from {folder}")
    all_results = []
    for wav_path in wavs[:getattr(args, 'max_files', 20)]:
        print(f"\n{'='*60}")
        args_copy = lambda: None
        args_copy.input = str(wav_path)
        args_copy.count = count
        args_copy.out = str(out_dir)
        cmd_recreate(args_copy)
        all_results.append({"source": str(wav_path), "out": str(out_dir)})

    print(f"\n{'='*60}")
    print(f"Completed: {len(all_results)} sources processed into {out_dir}")


def cmd_recreate_audit(args):
    audit_dir = Path(getattr(args, 'input_dir', 'outputs/recreate_test'))
    if not audit_dir.exists():
        print(f"Error: {audit_dir} not found. Run 'recreate' first.")
        return

    meta_files = sorted(audit_dir.glob("*.json"))
    if not meta_files:
        meta_files = sorted(audit_dir.glob("recreate_report_*.json"))

    if not meta_files:
        print(f"No metadata found in {audit_dir}")
        return

    print(f"Auditing {len(meta_files)} metadata files in {audit_dir}")

    all_scores = {"envelope": [], "harmonic": [], "noise": [], "overall": []}
    results = []

    for mf in meta_files:
        with open(mf) as f:
            data = json.load(f)

        if "results" in data:
            entries = data["results"]
        else:
            entries = [data]

        for entry in entries:
            env = entry.get("envelope_similarity", {}).get("score", 0)
            harm = entry.get("harmonic_similarity", {}).get("score", 0)
            noi = entry.get("noise_similarity", {}).get("score", 0)
            ovr = entry.get("overall_score", {}).get("overall", 0)
            all_scores["envelope"].append(env)
            all_scores["harmonic"].append(harm)
            all_scores["noise"].append(noi)
            all_scores["overall"].append(ovr)
            results.append(entry)

    import statistics
    def stats(vals):
        return {
            "mean": round(statistics.mean(vals), 4) if vals else 0,
            "median": round(statistics.median(vals), 4) if vals else 0,
            "min": round(min(vals), 4) if vals else 0,
            "max": round(max(vals), 4) if vals else 0,
            "std": round(statistics.stdev(vals), 4) if len(vals) > 1 else 0,
        }

    audit = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "audit_dir": str(audit_dir),
        "total_metadata_files": len(meta_files),
        "total_generations": len(results),
        "scores": {
            "envelope_similarity": stats(all_scores["envelope"]),
            "harmonic_similarity": stats(all_scores["harmonic"]),
            "noise_similarity": stats(all_scores["noise"]),
            "overall_score": stats(all_scores["overall"]),
        },
        "results": results,
    }

    audit_path = Path(getattr(args, 'output', None) or (CENSUS_DIR / "recreation_audit.json"))
    with open(audit_path, "w") as f:
        json.dump(audit, f, indent=2, cls=SafeEncoder)

    md_lines = []
    md_lines.append("# Recreation Engine — Audit Report")
    md_lines.append("")
    md_lines.append(f"**Generated:** {audit['generated_at']}")
    md_lines.append(f"**Audit directory:** `{audit_dir}`")
    md_lines.append(f"**Generations analyzed:** {len(results)}")
    md_lines.append("")
    md_lines.append("## Score Summary")
    md_lines.append("")
    md_lines.append("| Metric | Mean | Median | Min | Max | Std |")
    md_lines.append("|--------|------|--------|-----|-----|-----|")
    for key, label in [("envelope_similarity", "Envelope"), ("harmonic_similarity", "Harmonic"),
                        ("noise_similarity", "Noise"), ("overall_score", "Overall")]:
        s = audit["scores"][key]
        md_lines.append(f"| {label} | {s['mean']:.4f} | {s['median']:.4f} | {s['min']:.4f} | {s['max']:.4f} | {s['std']:.4f} |")
    md_lines.append("")
    md_lines.append("## Per-Generation Breakdown")
    md_lines.append("")
    md_lines.append("| Source | Generator | Env | Harm | Noise | Overall |")
    md_lines.append("|--------|-----------|-----|------|-------|---------|")
    for r in results:
        src = Path(r.get("source", "?")).name
        gen = f"{r.get('generator_family','?')}/{r.get('generator_profile','?')}"
        env = r.get("envelope_similarity", {}).get("score", 0)
        harm = r.get("harmonic_similarity", {}).get("score", 0)
        noi = r.get("noise_similarity", {}).get("score", 0)
        ovr = r.get("overall_score", {}).get("overall", 0)
        md_lines.append(f"| {src} | {gen} | {env:.3f} | {harm:.3f} | {noi:.3f} | {ovr:.3f} |")
    md_lines.append("")
    md_lines.append("## Score Distribution")
    md_lines.append("")
    all_ovr = all_scores["overall"]
    if all_ovr:
        bins = [(0, 0.25, "Poor"), (0.25, 0.5, "Fair"), (0.5, 0.75, "Good"), (0.75, 1.0, "Excellent")]
        for lo, hi, label in bins:
            cnt = sum(1 for s in all_ovr if lo <= s < hi)
            bar = "█" * max(1, cnt * 40 // max(len(all_ovr), 1))
            md_lines.append(f"| {label:<10} | {bar:<40} | {cnt:>4} ({cnt/max(len(all_ovr),1)*100:.0f}%) |")
            md_lines.append(f"| {'-'*10} | {'-'*40} | {'-'*10} |")
    md_lines.append("")
    md_lines.append("---")
    md_lines.append("*Generated by cShot Recreation Engine (Weeks 5-8)*")

    md_path = CENSUS_DIR / "recreation_audit.md"
    with open(md_path, "w") as f:
        f.write("\n".join(md_lines))
    print(f"Written audit to {md_path}")

    print(f"\nRecreation Audit Summary:")
    for key, label in [("envelope_similarity", "Envelope"), ("harmonic_similarity", "Harmonic"),
                        ("noise_similarity", "Noise"), ("overall_score", "Overall")]:
        s = audit["scores"][key]
        print(f"  {label}: mean={s['mean']:.4f}, median={s['median']:.4f}, range=[{s['min']:.4f}, {s['max']:.4f}]")

    return audit
