"""Feedback collection: export feedback forms, rating sheets, and producer test packages."""
import csv
import json
import shutil
import sys
import time
from pathlib import Path
from zipfile import ZipFile

from gen import REPO_ROOT
from gen.rating import _load_ratings


def _generate_rating_sheet(kit_dir: Path) -> str:
    """Generate a CSV rating sheet for all WAV files in a kit."""
    wav_files = sorted(kit_dir.rglob("*.wav"))
    lines = ["filename,category,duration_ms,rating,would_use,notes"]
    for w in wav_files:
        cat = w.parent.name if w.parent != kit_dir else ""
        lines.append(f"{w.name},{cat},,,,")
    return "\n".join(lines)


def _generate_feedback_form(kit_name: str) -> str:
    """Generate a markdown feedback form for producers."""
    return f"""# Producer Feedback: {kit_name}

Thank you for testing cShot! Your feedback helps us make better kits.

## Overall Impressions

1. What genre would you categorize this kit as?
   _________________________________________________

2. Rate the overall quality of this kit (1-5): ___

3. Would you use sounds from this kit in a release?
   [ ] Yes, as-is
   [ ] Yes, with processing
   [ ] Maybe after some edits
   [ ] No

4. What would you pay for a kit like this?
   [ ] $5-10
   [ ] $10-20
   [ ] $20-30
   [ ] Nothing

## Sound Quality

5. How are the drums? (kicks, snares, claps, hats)
   _________________________________________________
   _________________________________________________

6. How are the melodic sounds? (keys, synths, basses)
   _________________________________________________
   _________________________________________________

7. How are the FX/textures?
   _________________________________________________
   _________________________________________________

## Per-Sound Ratings

For each sound file, please rate:
  U = Would use
  M = Maybe with edits
  N = Would not use
  ? = Haven't listened

| # | File | Rating | Notes |
|---|------|--------|-------|
|   |      |        |       |
|   |      |        |       |
|   |      |        |       |

## Additional Feedback

8. What's missing from this kit?
   _________________________________________________

9. What would make this kit better?
   _________________________________________________

10. Any other thoughts?
    _________________________________________________
    _________________________________________________

---

Thank you! Please return this form to [your email/discord].
"""


def cmd_feedback_pack(args):
    """Package a kit with feedback forms for producer testing."""
    kit_dir = Path(args.kit_dir)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No WAV files found in {kit_dir}")
        return

    kit_name = kit_dir.name
    producer = args.name or "producer"
    out_dir = Path(args.out) if args.out else REPO_ROOT / "outputs" / "feedback" / f"{kit_name}_for_{producer}"
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"Feedback Pack — {kit_name}")
    print(f"{'='*60}")
    print(f"Producer: {producer}")
    print(f"Output:   {out_dir}")
    print()

    csv_content = _generate_rating_sheet(kit_dir)
    csv_path = out_dir / f"{kit_name}_rating_sheet.csv"
    csv_path.write_text(csv_content)
    print(f"  ✓ Rating sheet: {csv_path.name} ({len(wav_files)} sounds)")

    form_content = _generate_feedback_form(kit_name)
    form_path = out_dir / f"{kit_name}_feedback_form.md"
    form_path.write_text(form_content)
    print(f"  ✓ Feedback form: {form_path.name}")

    existing_ratings = _load_ratings()
    if existing_ratings:
        kit_ratings = [r for r in existing_ratings if kit_name in r.get("file", "")]
        if kit_ratings:
            ratings_path = out_dir / f"{kit_name}_existing_ratings.json"
            with open(ratings_path, "w") as f:
                json.dump(kit_ratings, f, indent=2)
            print(f"  ✓ Existing ratings: {ratings_path.name} ({len(kit_ratings)} ratings)")

    kit_copy = out_dir / "kit"
    kit_copy.mkdir(exist_ok=True)
    for w in wav_files:
        rel = w.relative_to(kit_dir)
        dest = kit_copy / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(w, dest)

    print(f"  ✓ Kit copied: {kit_copy} ({len(wav_files)} files)")

    zip_path = out_dir.parent / f"{kit_name}_for_{producer}.zip"
    with ZipFile(zip_path, "w") as zf:
        for f in out_dir.rglob("*"):
            if f.is_file():
                zf.write(f, f.relative_to(out_dir))
    print(f"  ✓ ZIP package: {zip_path}")

    summary = {
        "kit": kit_name,
        "producer": producer,
        "files": len(wav_files),
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "package": str(zip_path),
        "contents": {
            "rating_sheet": str(csv_path),
            "feedback_form": str(form_path),
            "kit_copy": str(kit_copy),
        },
    }
    summary_path = out_dir / "feedback_summary.json"
    with open(summary_path, "w") as f:
        json.dump(summary, f, indent=2)

    print(f"\n{'='*60}")
    print(f"Feedback pack ready!")
    print(f"  ZIP: {zip_path}")
    print(f"  Send to producer with instructions to:")
    print(f"    1. Listen to the kit")
    print(f"    2. Fill out the rating sheet CSV")
    print(f"    3. Fill out the feedback form")
    print(f"    4. Return both")
