"""cshot onboard — interactive producer onboarding flow."""
import json
import shutil
import sys
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.kit_spec import KitSpec, CategoryCounts, DNAProfile
from gen.kit_engine import generate_kit, compute_kit_coherence, setup_kit_export
from gen.rank import score_file
from gen.rating import _load_ratings
from gen.kit_engine import cmd_kit_from_song, cmd_kit_from_sample


GENRES = {
    "1": {"name": "trap", "label": "Trap / Drill", "centroid": 1800, "transient": 0.7, "sat": 0.5, "width": 0.4},
    "2": {"name": "rnb", "label": "R&B / Soul", "centroid": 2200, "transient": 0.4, "sat": 0.3, "width": 0.5},
    "3": {"name": "house", "label": "House / Techno", "centroid": 3500, "transient": 0.6, "sat": 0.3, "width": 0.6},
    "4": {"name": "ambient", "label": "Ambient / Cinematic", "centroid": 4000, "transient": 0.2, "sat": 0.2, "width": 0.7},
    "5": {"name": "hyperpop", "label": "Hyperpop / Rage", "centroid": 5000, "transient": 0.7, "sat": 0.6, "width": 0.7},
    "6": {"name": "lo_fi", "label": "Lo-Fi / Chill", "centroid": 2000, "transient": 0.3, "sat": 0.2, "width": 0.3},
    "7": {"name": "cinematic", "label": "Cinematic / Epic", "centroid": 3500, "transient": 0.5, "sat": 0.3, "width": 0.6},
    "8": {"name": "custom", "label": "Custom (type your own vibe)", "centroid": 3000, "transient": 0.5, "sat": 0.3, "width": 0.4},
}

SIZES = {
    "1": {"label": "Small (20 sounds)", "count": 20},
    "2": {"label": "Medium (40 sounds)", "count": 40},
    "3": {"label": "Large (80 sounds)", "count": 80},
}

INPUT_TYPES = {
    "1": {"label": "Describe a vibe (e.g. 'dark rnb one shots')", "key": "description"},
    "2": {"label": "Upload a song file", "key": "song"},
    "3": {"label": "Upload a sample file", "key": "sample"},
}


def _prompt(options: dict, prompt_text: str) -> str:
    """Show numbered options and return the selected key."""
    print(prompt_text)
    for key, opt in sorted(options.items()):
        print(f"  [{key}] {opt['label']}")
    while True:
        choice = input("> ").strip()
        if choice in options:
            return choice
        print(f"Invalid choice. Enter a number 1-{len(options)}.")


def cmd_onboard(args):
    """Interactive onboarding: choose genre, input, size, generate, listen, export."""
    print()
    print("  ╔══════════════════════════════════════════╗")
    print("  ║        Welcome to cShot v3!              ║")
    print("  ║  Custom one-shot kits for producers      ║")
    print("  ╚══════════════════════════════════════════╝")
    print()
    print("Let's create your first kit. I'll guide you through it.")
    print()

    genre_key = _prompt(GENRES, "Step 1: Pick a genre or vibe:")
    genre = GENRES[genre_key]

    vibe_prompt = ""
    if genre_key == "8":
        vibe_prompt = input("Describe your vibe: ").strip()
        if not vibe_prompt:
            vibe_prompt = "custom one shot kit"

    input_key = _prompt(INPUT_TYPES, "\nStep 2: How do you want to create your kit?")
    input_type = INPUT_TYPES[input_key]

    song_path = None
    sample_path = None
    if input_type["key"] == "song":
        song_path = input("Enter path to song file: ").strip()
        while song_path and not Path(song_path).exists():
            song_path = input("File not found. Enter path: ").strip()
        if not song_path:
            input_type = INPUT_TYPES["1"]
            print("No file given. Using vibe description instead.")
    elif input_type["key"] == "sample":
        sample_path = input("Enter path to sample file: ").strip()
        while sample_path and not Path(sample_path).exists():
            sample_path = input("File not found. Enter path: ").strip()
        if not sample_path:
            input_type = INPUT_TYPES["1"]
            print("No file given. Using vibe description instead.")

    size_key = _prompt(SIZES, "\nStep 3: Choose your kit size:")
    count = SIZES[size_key]["count"]

    print()
    print("  ──────────────────────────────────────────")
    print(f"  Genre:  {genre['label']}")
    print(f"  Input:  {input_type['label']}")
    print(f"  Size:   {count} sounds")
    print("  ──────────────────────────────────────────")
    print()

    input("Press Enter to generate your kit...")
    print()

    if input_type["key"] == "song":
        song_args = type("Args", (), {"song": song_path, "count": count, "out": None})()
        cmd_kit_from_song(song_args)
        out_dir = None
        return
    elif input_type["key"] == "sample":
        sample_args = type("Args", (), {"sample": sample_path, "count": count, "out": None})()
        cmd_kit_from_sample(sample_args)
        out_dir = None
        return

    prompt = vibe_prompt if vibe_prompt else f"{genre['name']} one shot kit"
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "kits" / f"onboard_{genre['name']}_{time.strftime('%Y%m%d_%H%M%S')}"

    cats = CategoryCounts()
    for field in vars(cats):
        setattr(cats, field, 0)
    distribution = [
        ("kicks", 0.12), ("snares", 0.10), ("claps", 0.08), ("hats", 0.14),
        ("percs", 0.08), ("basses_808", 0.10), ("keys", 0.10), ("synths", 0.10),
        ("guitars", 0.05), ("impacts", 0.05), ("textures", 0.05), ("atmospheres", 0.03),
    ]
    for field, pct in distribution:
        setattr(cats, field, max(1, int(round(count * pct))))

    spec = KitSpec(
        name=f"onboard_{genre['name']}_kit",
        prompt=prompt,
        genre=genre["name"],
        target_dna=DNAProfile(
            spectral_centroid_target=genre["centroid"],
            transient_aggression=genre["transient"],
            saturation_density=genre["sat"],
            stereo_width=genre["width"],
            tonal_noise_ratio=0.6,
            loudness_lufs=-14,
            brightness=min(genre["centroid"] / 8000.0, 1.0),
            darkness=min(1.0 - genre["centroid"] / 8000.0, 1.0),
            grit=genre["sat"],
            dryness=0.6,
            punch=genre["transient"],
            softness=1.0 - genre["transient"],
            analog_warmth=0.2,
            width_span=genre["width"],
        ),
        categories=cats,
        total_target=count,
    )

    t0 = time.time()
    generated = generate_kit(spec, out_dir, polish=True)
    t1 = time.time()

    print(f"\n{'='*60}")
    print(f"Generated {generated}/{count} sounds in {t1-t0:.1f}s")
    print(f"Output: {out_dir}")
    print()

    coherence = compute_kit_coherence(out_dir)
    if "error" not in coherence:
        print(f"Coherence: {coherence.get('overall_coherence', 0):.3f}")
    print()

    wav_files = sorted(out_dir.rglob("*.wav"))
    ratings = _load_ratings()
    scored = []
    for w in wav_files:
        s = score_file(w, ratings)
        s["path"] = str(w.relative_to(out_dir))
        scored.append(s)
    scored.sort(key=lambda x: x.get("score", 0), reverse=True)

    top_dir = out_dir / "_top"
    top_dir.mkdir(exist_ok=True)
    top_n = min(10, len(scored))
    for i in range(top_n):
        src = out_dir / scored[i]["path"]
        dest = top_dir / f"{i+1:02d}_{Path(scored[i]['path']).name}"
        shutil.copy2(src, dest)

    setup_kit_export(out_dir, spec, coherence)

    print("  ── ONBOARDING COMPLETE ──")
    print(f"  Your kit is at: {out_dir}")
    print(f"  {generated} sounds organized in categories")
    print(f"  Manifest: {out_dir / 'manifest.json'}")
    print(f"  README:   {out_dir / 'README.md'}")
    print()

    answer = input("Listen to your kit now? (y/n): ").strip().lower()
    if answer == "y" or answer == "yes":
        from gen.listen import cmd_listen
        listen_args = type("Args", (), {"input_dir": str(out_dir), "notes": None})()
        cmd_listen(listen_args)

    answer = input("Export your favorites? (y/n): ").strip().lower()
    if answer == "y" or answer == "yes":
        export_dir = out_dir / "_favorites"
        export_dir.mkdir(exist_ok=True)
        fav_count = 0
        for r in ratings:
            if r["rating"] == "favorite":
                src = REPO_ROOT / r["file"]
                if src.exists():
                    shutil.copy2(src, export_dir / src.name)
                    fav_count += 1
        print(f"Exported {fav_count} favorites to {export_dir}")
