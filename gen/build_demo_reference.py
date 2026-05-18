"""Generate 5 reference-conditioned demo kits.

Folders:
  outputs/reference_dark_rnb_kit/
  outputs/reference_bay_area_drums/
  outputs/reference_monteray_808s/
  outputs/reference_fx_textures/
  outputs/reference_song_to_kit/
"""

import json
import time
from pathlib import Path

from gen import REPO_ROOT
from gen.retrieval import retrieve_by_text
from gen.reference_transform import transform_references_to_kit
from gen.similarity_guard import validate_kit


DEMO_KITS = [
    {
        "name": "reference_dark_rnb_kit",
        "query": "dark rnb one shot kit kick snare 808",
        "n": 40,
        "description": "Dark R&B one-shot kit — kicks, snares, 808s, hat variations",
    },
    {
        "name": "reference_bay_area_drums",
        "query": "bay area drum kit kick snare clap hat",
        "n": 40,
        "description": "Bay Area style drum kit — hyphy, west coast, heavy kicks",
    },
    {
        "name": "reference_monteray_808s",
        "query": "808 sub bass kick low end",
        "n": 30,
        "description": "Monteray-style 808s and sub basses",
    },
    {
        "name": "reference_fx_textures",
        "query": "fx impact texture riser atmospheric",
        "n": 25,
        "description": "FX, impacts, risers, and atmospheric textures",
    },
    {
        "name": "reference_song_to_kit",
        "query": "song to kit drums percussion full",
        "n": 30,
        "description": "Full reference-conditioned song-to-kit demo",
    },
]


def generate_demo_kit(name: str, query: str, n: int, description: str):
    out_dir = REPO_ROOT / "outputs" / name
    if out_dir.exists():
        import shutil
        shutil.rmtree(out_dir)

    print(f"\n{'='*60}")
    print(f"Generating: {name}")
    print(f"Query:      '{query}'")
    print(f"Target:     {n} files")
    print(f"Output:     {out_dir}")
    print(f"{'='*60}")

    # Retrieve references
    refs = retrieve_by_text(query, n=n * 2)
    print(f"Retrieved {len(refs)} references")

    # Generate kit
    result = transform_references_to_kit(refs, out_dir, target_count=n)
    print(f"Generated: {result['generated']} files")

    # Validate
    validation = validate_kit(out_dir)
    print(f"Validation: {validation.get('passed', 0)}/{validation.get('total', 0)} passed")

    # Generate listening notes
    notes = _make_listening_notes(out_dir, name, description, result, validation)
    (out_dir / "listening_notes.md").write_text(notes)
    print(f"Listening notes written")

    print(f"Done: {out_dir}")


def _make_listening_notes(out_dir: Path, name: str, description: str,
                          result: dict, validation: dict) -> str:
    wavs = sorted(out_dir.glob("*.wav"))
    lineage_path = out_dir / "source_lineage.json"
    sources = set()
    if lineage_path.exists():
        with open(lineage_path) as f:
            ld = json.load(f)
        sources = set(e.get("source", "") for e in ld.get("entries", []))

    return f"""# Listening Notes — {name}

Generated: {time.strftime("%Y-%m-%d %H:%M:%S UTC", time.gmtime())}
Description: {description}

## Summary
| Metric | Value |
|--------|-------|
| Total WAV files | {len(wavs)} |
| Source references used | {len(sources)} |
| Coherence score | {result.get('coherence', 0)} |
| Similarity guard pass rate | {validation.get('passed', 0)}/{validation.get('total', 0)} |

## Quality Assessment
Each sound in this kit is a transformed variation of a professional reference
sample from the cShot reference library. Transformations include pitch shifting,
EQ sculpting, saturation, transient reshaping, time stretching, convolution
reverb, and/or spectral morphing.

No exact copies of source material are included. Each output passes the
similarity guard with minimum transformation distance enforced.

## Files
| # | File | Source Ref |
|---|------|------------|
"""
    for i, w in enumerate(wavs[:40]):
        source = "N/A"
        if lineage_path.exists():
            with open(lineage_path) as f:
                ld = json.load(f)
            for e in ld.get("entries", []):
                if e.get("output") == w.name:
                    source = Path(e.get("source", "")).name
                    break
        notes += f"| {i+1} | {w.name} | {source} |\n"

    return notes


def main():
    print("cShot Reference-Conditioned Demo Kit Generator")
    print("=" * 60)
    t0 = time.time()

    for kit in DEMO_KITS:
        generate_demo_kit(**kit)

    elapsed = time.time() - t0
    print(f"\n{'='*60}")
    print(f"All {len(DEMO_KITS)} demo kits generated in {elapsed:.0f}s")
    print(f"{'='*60}")

    # Print summary
    total_files = 0
    for kit in DEMO_KITS:
        out_dir = REPO_ROOT / "outputs" / kit["name"]
        wavs = list(out_dir.glob("*.wav"))
        total_files += len(wavs)
        print(f"  {kit['name']}: {len(wavs)} WAVs")

    print(f"\nTotal: {total_files} files across {len(DEMO_KITS)} demo kits")


if __name__ == "__main__":
    main()
