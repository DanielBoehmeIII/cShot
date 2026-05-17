"""Rating system: rate files, store in ratings.jsonl, query favorites/summary."""
import json
import sys
import time
from pathlib import Path
from typing import Optional
from collections import Counter

from gen import REPO_ROOT


RATINGS_FILE = REPO_ROOT / "ratings.jsonl"


def _load_ratings() -> list[dict]:
    if not RATINGS_FILE.exists():
        return []
    ratings = []
    with open(RATINGS_FILE) as f:
        for line in f:
            line = line.strip()
            if line:
                try:
                    ratings.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
    return ratings


def _save_rating(entry: dict):
    RATINGS_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(RATINGS_FILE, "a") as f:
        f.write(json.dumps(entry) + "\n")


def cmd_rate(args):
    file_path = Path(args.file)
    if not file_path.exists():
        print(f"Error: {file_path} not found", file=sys.stderr)
        sys.exit(1)
    rating = args.rating
    notes = args.notes or ""

    resolved = str(file_path.resolve())
    repo_rel = resolved.replace(str(REPO_ROOT.resolve()) + "/", "")

    import hashlib
    file_hash = hashlib.md5(resolved.encode()).hexdigest()

    entry = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "file": repo_rel,
        "file_hash": file_hash,
        "rating": rating,
        "notes": notes,
    }
    _save_rating(entry)
    print(f"Rated {repo_rel} as {rating}" + (f" — {notes}" if notes else ""))


def cmd_ratings_summary(args):
    ratings = _load_ratings()
    if not ratings:
        print("No ratings found.")
        return

    total = len(ratings)
    rating_counts = Counter(r["rating"] for r in ratings)
    favorited = [r for r in ratings if r["rating"] == "favorite"]
    trash = [r for r in ratings if r["rating"] == "trash"]
    good = [r for r in ratings if r["rating"] == "good"]
    bad = [r for r in ratings if r["rating"] == "bad"]
    unique_files = len(set(r["file"] for r in ratings))

    print(f"Rating Summary ({total} total, {unique_files} unique files)")
    print(f"{'='*50}")
    print(f"  favorite: {len(favorited)}")
    print(f"  good:     {len(good)}")
    print(f"  bad:      {len(bad)}")
    print(f"  trash:    {len(trash)}")
    print()

    if favorited:
        print("Favorites:")
        for r in favorited[-10:]:
            note = f" — {r['notes']}" if r.get("notes") else ""
            print(f"  {r['file']}{note}")
    if trash:
        print("\nTrash:")
        for r in trash[-10:]:
            note = f" — {r['notes']}" if r.get("notes") else ""
            print(f"  {r['file']}{note}")


def cmd_favorites(args):
    ratings = _load_ratings()
    favorited = [r for r in ratings if r["rating"] == "favorite"]
    if not favorited:
        print("No favorites yet.")
        return

    print(f"Favorite files ({len(favorited)} total):")
    print(f"{'='*50}")
    for r in favorited:
        note = f" — {r['notes']}" if r.get("notes") else ""
        print(f"  {r['file']}{note}")
