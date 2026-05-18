"""Week 34 — DAW Export Polish: organize kits for Ableton/FL Studio."""
import json
import shutil
import sys
import time
from pathlib import Path
from zipfile import ZipFile

from gen import REPO_ROOT
from gen.io import read_wav
from gen.features import compute_features
from gen.polish import polish_file, trim_silence, apply_fade, normalize_peak
from gen.kit_engine import compute_kit_coherence, setup_kit_export


def cmd_export_daw(args):
    """Export a kit with DAW-friendly organization: folders, naming, key/BPM tags."""
    kit_dir = Path(args.kit_dir)
    if not kit_dir.exists():
        print(f"Error: {kit_dir} not found")
        sys.exit(1)

    wav_files = sorted(kit_dir.rglob("*.wav"))
    if not wav_files:
        print(f"No WAV files found")
        return

    daw = args.daw.lower()
    out_dir = Path(args.out) if args.out else kit_dir.parent / f"{kit_dir.name}_{daw}"
    out_dir.mkdir(parents=True, exist_ok=True)

    print(f"DAW Export — {daw.upper()}")
    print(f"{'='*60}")
    print(f"Source: {kit_dir}")
    print(f"Target: {out_dir}")
    print(f"Files:  {len(wav_files)}")
    print()

    coherence = compute_kit_coherence(kit_dir)
    bpm = coherence.get("transient_mean", 120) * 15
    bpm = max(60, min(200, int(bpm)))

    category_map = {
        "kicks": "Drums/Kicks", "snares": "Drums/Snares", "claps": "Drums/Claps",
        "hats": "Drums/Hats", "open_hats": "Drums/Open Hats", "percs": "Drums/Percussion",
        "basses_808": "Bass/808", "basses_sub": "Bass/Sub",
        "keys": "Melodic/Keys", "synths": "Melodic/Synths", "guitars": "Melodic/Guitars",
        "impacts": "FX/Impacts", "risers": "FX/Risers", "glitches": "FX/Glitches",
        "textures": "FX/Textures", "atmospheres": "FX/Atmospheres", "fx_noise": "FX/Noise",
    }

    for w in wav_files:
        cat = w.parent.name
        daw_cat = category_map.get(cat, f"Other/{cat}")

        dest_dir = out_dir / daw_cat
        dest_dir.mkdir(parents=True, exist_ok=True)

        result = read_wav(w)
        if result is None:
            continue
        samples, sr = result
        if samples.ndim == 2:
            samples = samples.mean(axis=1)

        feats = compute_features(samples, sr)
        samples = trim_silence(samples, -60)
        samples = apply_fade(samples, 3, 5)
        samples = normalize_peak(samples, -1.0)

        pitch = feats.get("pitch_hz", 0)
        key_tag = f"_{int(pitch)}Hz" if pitch > 0 else ""

        name = w.stem
        new_name = f"{daw_cat.replace('/', '_')}_{name}{key_tag}.wav"
        new_path = dest_dir / new_name

        from gen.io import write_wav
        write_wav(new_path, samples)

    if args.zipped:
        zip_path = out_dir.parent / f"{kit_dir.name}_{daw}.zip"
        with ZipFile(zip_path, "w") as zf:
            for f in out_dir.rglob("*"):
                if f.is_file():
                    zf.write(f, f.relative_to(out_dir))
        print(f"Zipped: {zip_path}")

    print(f"\n{'='*60}")
    print(f"DAW Export Complete: {out_dir}")
    print(f"  Format: {daw.upper()} folder structure")
    print(f"  BPM tag: ~{bpm}")
    if pitch:
        print(f"  Pitch-tagged files")
    print()
    print(f"Drag the entire {out_dir.name} folder into your DAW.")
