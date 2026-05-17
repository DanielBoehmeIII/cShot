"""Taste profile: learn from ratings to influence generation preferences."""
import json
import sys
from pathlib import Path
from collections import Counter

from gen import REPO_ROOT
from gen.rating import _load_ratings

TASTE_PROFILE_PATH = REPO_ROOT / "taste_profile.json"


def build_taste_profile() -> dict:
    """Build a taste profile from rating history."""
    ratings = _load_ratings()
    if not ratings:
        return {"families": {}, "adjectives": {}, "total_ratings": 0}

    family_counts = Counter()
    family_favs = Counter()
    adjective_counts = Counter()
    adjective_favs = Counter()

    for r in ratings:
        rating = r.get("rating", "")
        file = r.get("file", "")
        prompt = r.get("_prompt", "")
        family = r.get("_family", "")

        # Infer family from file path if not stored
        if not family:
            folder_map = {
                "piano": "piano-gen", "keys": "piano-gen", "bell": "piano-gen", "rhodes": "piano-gen",
                "synth": "synth-gen", "pluck": "synth-gen", "stab": "synth-gen", "pad": "synth-gen", "lead": "synth-gen",
                "bass": "bass-gen", "808": "bass-gen", "reese": "bass-gen", "sub": "bass-gen",
                "guitar": "guitar-gen", "nylon": "guitar-gen",
                "fx": "fx-gen", "impact": "fx-gen", "riser": "fx-gen", "glitch": "fx-gen",
                "drums": "drums", "kick": "drums", "snare": "drums", "clap": "drums", "hat": "drums",
            }
            for keyword, fam in folder_map.items():
                if keyword in file.lower():
                    family = fam
                    break

        if family:
            family_counts[family] += 1
            if rating in ("favorite", "good"):
                family_favs[family] += 1

    profile = {
        "total_ratings": len(ratings),
        "favorites": sum(1 for r in ratings if r.get("rating") == "favorite"),
        "good": sum(1 for r in ratings if r.get("rating") == "good"),
        "bad": sum(1 for r in ratings if r.get("rating") == "bad"),
        "trash": sum(1 for r in ratings if r.get("rating") == "trash"),
        "families": {
            fam: {
                "total": family_counts[fam],
                "favorites": family_favs[fam],
                "favorability": round(family_favs[fam] / max(family_counts[fam], 1) * 100, 1),
            }
            for fam in sorted(family_counts.keys())
        },
    }

    # Save
    TASTE_PROFILE_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(TASTE_PROFILE_PATH, "w") as f:
        json.dump(profile, f, indent=2)

    return profile


def cmd_taste_profile(args):
    """Show and build the taste profile from rating history."""
    if not TASTE_PROFILE_PATH.exists():
        print("Building taste profile from ratings...")
        profile = build_taste_profile()
    else:
        with open(TASTE_PROFILE_PATH) as f:
            profile = json.load(f)

    print(f"Taste Profile")
    print(f"{'='*50}")
    print(f"Total ratings: {profile.get('total_ratings', 0)}")
    print(f"  Favorites: {profile.get('favorites', 0)}")
    print(f"  Good:      {profile.get('good', 0)}")
    print(f"  Bad:       {profile.get('bad', 0)}")
    print(f"  Trash:     {profile.get('trash', 0)}")
    print()

    families = profile.get("families", {})
    if families:
        print("Family Preferences:")
        for fam, data in sorted(families.items(), key=lambda x: x[1]["favorability"], reverse=True):
            bar = "█" * int(data["favorability"] / 10)
            print(f"  {fam:12s} {data['favorability']:5.1f}% favorable  {bar}")
    print(f"\nProfile saved: {TASTE_PROFILE_PATH}")


def cmd_prompt_history(args):
    """Show prompt generation history from ratings and metadata."""
    ratings = _load_ratings()
    if not ratings:
        print("No rating history yet.")
        return

    prompt_ratings = Counter()
    prompt_favs = Counter()

    for r in ratings:
        file = r.get("file", "")
        # Try to extract prompt from folder structure
        parts = Path(file).parts
        prompt = " ".join(parts[:-1]) if len(parts) > 1 else file
        prompt_ratings[prompt] += 1
        if r.get("rating") in ("favorite", "good"):
            prompt_favs[prompt] += 1

    print("Prompt History (by rating count):")
    print(f"{'='*60}")
    for prompt, count in prompt_ratings.most_common(20):
        favs = prompt_favs.get(prompt, 0)
        fav_pct = favs / max(count, 1) * 100
        bar = "█" * max(1, int(fav_pct / 10))
        print(f"  {count:>3} ratings, {favs:>2} favorites ({fav_pct:4.0f}%)  {prompt[:50]}")
