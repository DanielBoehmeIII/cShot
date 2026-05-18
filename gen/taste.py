"""Taste Learning v2 — per-family, prompt, and pack-level preference learning + Personal DNA."""
import json
import math
import shutil
import sys
import time
from collections import Counter, defaultdict
from pathlib import Path
from zipfile import ZipFile

import numpy as np

from gen import REPO_ROOT
from gen.rating import _load_ratings
from gen.features import compute_features, compute_features_cached, clear_feature_cache
from gen.io import read_wav
from gen.kit_spec import DNAProfile, CategoryCounts
from gen.kit_engine import generate_kit, compute_kit_coherence, setup_kit_export
from gen.quality_gate import run_all_gates
from gen.rank import score_file
from gen.polish import polish_file

TASTE_PROFILE_PATH = REPO_ROOT / "taste_profile.json"


def build_taste_profile() -> dict:
    """Build a comprehensive taste profile from rating history."""
    ratings = _load_ratings()
    if not ratings:
        base = {"total_ratings": 0, "families": {}, "prompts": {}, "preferences": {}}
        _save_profile(base)
        return base

    family_counts = Counter()
    family_favs = Counter()
    family_centroids = defaultdict(list)
    family_transients = defaultdict(list)
    prompt_counts = Counter()
    prompt_favs = Counter()

    for r in ratings:
        rating = r.get("rating", "")
        file = r.get("file", "")
        family = r.get("_family", "")

        if not family:
            folder_map = {
                "piano": "piano-gen", "keys": "piano-gen", "bell": "piano-gen",
                "synth": "synth-gen", "pluck": "synth-gen", "stab": "synth-gen",
                "bass": "bass-gen", "808": "bass-gen", "reese": "bass-gen",
                "guitar": "guitar-gen",
                "fx": "fx-gen", "impact": "fx-gen", "riser": "fx-gen", "glitch": "fx-gen",
                "kick": "drums", "snare": "drums", "clap": "drums", "hat": "drums",
            }
            for kw, fam in folder_map.items():
                if kw in file.lower():
                    family = fam
                    break

        if family:
            family_counts[family] += 1
            if rating in ("favorite", "good"):
                family_favs[family] += 1

        path = REPO_ROOT / file
        if path.exists():
            try:
                feats = compute_features_cached(str(path))
                if feats:
                    family_centroids[family or "unknown"].append(feats.get("spectral_centroid", 0))
                    family_transients[family or "unknown"].append(feats.get("transient_count", 0))
            except Exception:
                pass

    prompt_texts = list(prompt_counts.keys())

    families = {}
    for fam in sorted(family_counts.keys()):
        cents = family_centroids.get(fam, [])
        trans = family_transients.get(fam, [])
        families[fam] = {
            "total": family_counts[fam],
            "favorites": family_favs[fam],
            "favorability": round(family_favs[fam] / max(family_counts[fam], 1) * 100, 1),
            "avg_centroid": round(float(np.mean(cents)), 0) if cents else None,
            "avg_transient": round(float(np.mean(trans)), 1) if trans else None,
        }

    profile = {
        "total_ratings": len(ratings),
        "favorites": sum(1 for r in ratings if r.get("rating") == "favorite"),
        "good": sum(1 for r in ratings if r.get("rating") == "good"),
        "bad": sum(1 for r in ratings if r.get("rating") == "bad"),
        "trash": sum(1 for r in ratings if r.get("rating") == "trash"),
        "families": families,
        "preferences": {
            "fav_darkness": "dark" if any("dark" in r.get("file", "").lower() for r in ratings if r.get("rating") == "favorite") else "neutral",
            "fav_punch": "high" if any("punch" in r.get("file", "").lower() for r in ratings if r.get("rating") == "favorite") else "neutral",
        },
    }

    _save_profile(profile)
    return profile


def _save_profile(profile: dict):
    TASTE_PROFILE_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(TASTE_PROFILE_PATH, "w") as f:
        json.dump(profile, f, indent=2)


def cmd_taste_profile(args):
    """Show and refresh the taste profile."""
    rebuild = getattr(args, "rebuild", False)
    if rebuild or not TASTE_PROFILE_PATH.exists():
        print("Building taste profile from ratings...")
        profile = build_taste_profile()
    else:
        with open(TASTE_PROFILE_PATH) as f:
            profile = json.load(f)

    print(f"Taste Profile")
    print(f"{'='*50}")
    print(f"Total ratings: {profile.get('total_ratings', 0)}")
    print(f"  ★ Favorites: {profile.get('favorites', 0)}")
    print(f"  ✓ Good:      {profile.get('good', 0)}")
    print(f"  ✗ Bad:       {profile.get('bad', 0)}")
    print(f"  ✗ Trash:     {profile.get('trash', 0)}")
    print()

    families = profile.get("families", {})
    if families:
        print("Family Preferences (by favorability):")
        for fam, data in sorted(families.items(), key=lambda x: x[1]["favorability"], reverse=True):
            bar = "█" * int(data["favorability"] / 10)
            cent = f" cent={data['avg_centroid']:.0f}Hz" if data.get("avg_centroid") else ""
            print(f"  {fam:12s} {data['favorability']:5.1f}% favorable {bar}{cent}")

    prefs = profile.get("preferences", {})
    if prefs:
        print(f"\nLearned preferences:")
        for k, v in prefs.items():
            print(f"  {k}: {v}")

    print(f"\nSaved: {TASTE_PROFILE_PATH}")
    print(f"Run with --rebuild to refresh from latest ratings")


# ── Week 23: Personal Producer DNA ──

def learn_dna_from_folder(folder: Path) -> DNAProfile:
    """Analyze a folder of favorite WAV files and build a DNA profile."""
    wavs = sorted(folder.rglob("*.wav"))
    if not wavs:
        print(f"No WAV files in {folder}")
        sys.exit(1)

    centroids, transients, hprs, rms_vals, attacks, decays, low_bands, high_bands = [], [], [], [], [], [], [], []
    for w in wavs:
        feats = compute_features_cached(str(w))
        if not feats:
            continue
        centroids.append(feats.get("spectral_centroid", 2500))
        transients.append(feats.get("transient_count", 5))
        hprs.append(feats.get("hpr", 0.6))
        rms_vals.append(feats.get("rms", 0.1))
        attacks.append(feats.get("attack_ms", 10))
        decays.append(feats.get("decay_length_ms", 100))
        low_bands.append(feats.get("low_band_energy", 0.5))
        high_bands.append(feats.get("high_band_energy", 0.5))

    if not centroids:
        print("Could not analyze any files")
        sys.exit(1)

    avg_centroid = float(np.mean(centroids))
    avg_transient = float(np.mean(transients))
    avg_hpr = float(np.mean(hprs))
    avg_rms = float(np.mean(rms_vals))

    punch = min(avg_transient / 10.0, 1.0)
    brightness = min(avg_centroid / 8000.0, 1.0)
    darkness = min(1.0 - brightness, 1.0)
    width = 0.3 + (float(np.std(hprs)) if len(hprs) > 1 else 0.0)

    return DNAProfile(
        spectral_centroid_target=avg_centroid,
        transient_aggression=punch,
        saturation_density=min(1.0, 0.2 + (1.0 - avg_hpr) * 0.5),
        stereo_width=min(width, 1.0),
        tonal_noise_ratio=avg_hpr,
        loudness_lufs=-10 + 20 * math.log10(max(avg_rms, 1e-10)) if avg_rms > 0 else -14,
        brightness=brightness,
        darkness=darkness,
        grit=min(1.0, 0.2 + (1.0 - avg_hpr) * 0.3),
        dryness=0.5 + avg_hpr * 0.3,
        punch=punch,
        softness=1.0 - punch,
        analog_warmth=0.2 if avg_hpr > 0.6 else 0.0,
        width_span=min(width, 1.0),
    )


def cmd_learn_from_folder(args):
    """Learn a DNA profile from a folder of favorite WAV files."""
    folder = Path(args.folder)
    if not folder.exists():
        print(f"Error: {folder} not found")
        sys.exit(1)

    out_path = Path(args.out) if args.out else REPO_ROOT / "producer_dna.json"

    print(f"Learning DNA from {folder}...")
    print()

    dna = learn_dna_from_folder(folder)

    dna_dict = {
        "spectral_centroid_target": dna.spectral_centroid_target,
        "transient_aggression": dna.transient_aggression,
        "saturation_density": dna.saturation_density,
        "stereo_width": dna.stereo_width,
        "tonal_noise_ratio": dna.tonal_noise_ratio,
        "brightness": dna.brightness,
        "darkness": dna.darkness,
        "grit": dna.grit,
        "dryness": dna.dryness,
        "punch": dna.punch,
        "softness": dna.softness,
        "analog_warmth": dna.analog_warmth,
        "width_span": dna.width_span,
    }

    with open(out_path, "w") as f:
        json.dump(dna_dict, f, indent=2)

    print(f"Producer DNA Profile")
    print(f"{'='*50}")
    print(f"  Centroid:          {dna.spectral_centroid_target:.0f} Hz")
    print(f"  Transient punch:   {dna.transient_aggression:.2f}")
    print(f"  Brightness:        {dna.brightness:.2f}")
    print(f"  Darkness:          {dna.darkness:.2f}")
    print(f"  Saturation/grit:   {dna.grit:.2f}")
    print(f"  Stereo width:      {dna.stereo_width:.2f}")
    print(f"  Dryness:           {dna.dryness:.2f}")
    print(f"  Punch:             {dna.punch:.2f}")
    print()
    print(f"Saved to: {out_path}")
    print(f"Use with: cshot make --dna {out_path}")


def cmd_taste_prompt_history(args):
    """Show prompt-level rating history."""
    ratings = _load_ratings()
    if not ratings:
        print("No rating history yet.")
        return

    prompt_stats = defaultdict(lambda: {"total": 0, "fav": 0, "good": 0, "bad": 0, "trash": 0})
    for r in ratings:
        file = r.get("file", "")
        parts = Path(file).parts
        prompt = " / ".join(parts[:-1]) if len(parts) > 1 else file
        prompt_stats[prompt][r.get("rating", "unknown")] += 1
        prompt_stats[prompt]["total"] += 1

    print("Prompt History (by total ratings):")
    print(f"{'='*60}")
    sorted_prompts = sorted(prompt_stats.items(), key=lambda x: x[1]["total"], reverse=True)
    for prompt, stats in sorted_prompts[:20]:
        fav = stats["fav"]
        total = stats["total"]
        pct = fav / max(total, 1) * 100
        bar = "█" * max(1, int(pct / 10))
        print(f"  {total:>3} ratings, {fav:>2} fav ({pct:3.0f}%) {bar}  {prompt[:50]}")


# ── Week 24: .cshotpack format ──

def cmd_export_cshotpack(args):
    """Export a kit to .cshotpack format."""
    kit_dir = Path(args.kit_dir)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    out_path = Path(args.out) if args.out else kit_dir.parent / f"{kit_dir.name}.cshotpack"

    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No WAV files found")
        return

    metadata = {
        "format": "cshotpack",
        "version": 1,
        "name": kit_dir.name,
        "created_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_files": len(wav_files),
        "categories": {},
    }

    has_dna = False
    dna_data = {}
    dna_path = kit_dir / "producer_dna.json"
    if dna_path.exists():
        dna_data = json.load(open(dna_path))
        has_dna = True
        metadata["dna"] = dna_data

    coherence_path = kit_dir / "kit_audit.json"
    if coherence_path.exists():
        metadata["coherence"] = json.load(open(coherence_path))

    manifest_path = kit_dir / "manifest.json"
    if manifest_path.exists():
        manifest = json.load(open(manifest_path))
        metadata["manifest"] = manifest

    for w in wav_files:
        cat = w.parent.name if w.parent != kit_dir else "root"
        metadata["categories"].setdefault(cat, 0)
        metadata["categories"][cat] += 1

    with ZipFile(out_path, "w") as zf:
        zf.writestr("cshotpack.json", json.dumps(metadata, indent=2))
        for w in wav_files:
            arcname = f"sounds/{w.parent.name}/{w.name}" if w.parent != kit_dir else f"sounds/{w.name}"
            zf.write(w, arcname)

    print(f"Exported {kit_dir.name} → {out_path}")
    print(f"  {len(wav_files)} sounds")
    print(f"  DNA: {'yes' if has_dna else 'no'}")
    print(f"  Categories: {len(metadata['categories'])}")


def cmd_import_cshotpack(args):
    """Import a .cshotpack file."""
    pack_path = Path(args.pack_file)
    if not pack_path.exists():
        print(f"Error: {pack_path} not found")
        sys.exit(1)

    out_dir = Path(args.out) if args.out else REPO_ROOT / "Packs" / pack_path.stem
    out_dir.mkdir(parents=True, exist_ok=True)

    with ZipFile(pack_path, "r") as zf:
        if "cshotpack.json" not in zf.namelist():
            print("Invalid .cshotpack (missing manifest)")
            sys.exit(1)

        metadata = json.loads(zf.read("cshotpack.json"))
        zf.extractall(out_dir)

    # Move sounds out of the sounds/ subdirectory
    sounds_dir = out_dir / "sounds"
    if sounds_dir.exists():
        for w in sorted(sounds_dir.rglob("*.wav")):
            rel = w.relative_to(sounds_dir)
            dest = out_dir / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.move(str(w), str(dest))
        shutil.rmtree(sounds_dir)

    dna_path = out_dir / "producer_dna.json"
    has_dna = dna_path.exists()

    print(f"Imported {pack_path.stem} → {out_dir}")
    print(f"  Sounds: {metadata.get('total_files', '?')}")
    print(f"  DNA: {'yes' if has_dna else 'no'}")
    print(f"  Name: {metadata.get('name', pack_path.stem)}")
