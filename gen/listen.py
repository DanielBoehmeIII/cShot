"""Listening workflow: interactive rating with playback, rank outputs, save notes."""

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional

from gen import REPO_ROOT, SAMPLE_RATE
from gen.io import read_wav, write_wav
from gen.features import compute_features


def _find_audio_player() -> Optional[str]:
    """Find an available system audio player for WAV playback."""
    for player in ["aplay", "paplay", "ffplay", "gst-play-1.0", "play"]:
        if shutil.which(player):
            return player
    return None


def _play_audio(path: Path, player: Optional[str] = None):
    """Play a WAV file using the system audio player."""
    if not player:
        player = _find_audio_player()
    if not player:
        print("  [no audio player found — install aplay, paplay, or ffplay]")
        return

    try:
        if player == "ffplay":
            subprocess.run([player, "-nodisp", "-autoexit", "-loglevel", "quiet", str(path)],
                          timeout=10, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)
        elif player == "gst-play-1.0":
            subprocess.run([player, "--quiet", str(path)],
                          timeout=10, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)
        else:
            subprocess.run([player, str(path)],
                          timeout=10, stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)
    except subprocess.TimeoutExpired:
        pass
    except Exception as e:
        print(f"  [playback error: {e}]")


def _load_notes(notes_path: Path) -> dict:
    """Load existing listening notes from JSON file."""
    if notes_path.exists():
        with open(notes_path) as f:
            return json.load(f)
    return {
        "generated_at": None,
        "last_updated": None,
        "session_count": 0,
        "notes": {},
    }


def _save_notes(notes_path: Path, notes_data: dict):
    """Save listening notes to JSON file."""
    notes_data["last_updated"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    notes_path.write_text(json.dumps(notes_data, indent=2))


def _format_duration(ms: float) -> str:
    if ms < 1000:
        return f"{ms:.0f}ms"
    return f"{ms / 1000:.1f}s"


def _feature_summary_line(feats: dict) -> str:
    cent = feats.get("spectral_centroid", 0)
    low = feats.get("low_band_energy", 0)
    high = feats.get("high_band_energy", 0)
    trans = feats.get("transient_count", 0)
    decay = feats.get("decay_length_ms", 0)
    hpr = feats.get("hpr", -1)
    pitch = feats.get("pitch_hz", 0)
    parts = [f"cent={cent:.0f}Hz", f"low={low:.2f}", f"high={high:.2f}",
             f"trans={trans}", f"decay={decay:.0f}ms"]
    if hpr >= 0:
        parts.append(f"hpr={hpr:.2f}")
    if pitch > 0:
        parts.append(f"pitch={pitch:.0f}Hz")
    return " | ".join(parts)


def cmd_listen(args):
    """Interactive listening session: rate files, mark good/bad, add notes."""
    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = sorted(in_dir.glob("*.wav"))
    if not wav_files:
        print(f"Error: no .wav files found in {in_dir}", file=sys.stderr)
        sys.exit(1)

    notes_path = Path(args.notes) if args.notes else in_dir / "listening_notes.json"
    notes_data = _load_notes(notes_path)

    if notes_data["generated_at"] is None:
        notes_data["generated_at"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    notes_data["session_count"] = notes_data.get("session_count", 0) + 1

    player = _find_audio_player()
    if not player:
        print("Warning: no audio player found (install aplay/paplay/ffplay)")
    else:
        print(f"Audio player: {player}")

    print(f"Directory: {in_dir}")
    print(f"Files: {len(wav_files)}")
    print(f"Notes: {notes_path}")
    print()
    print("Controls:")
    print("  1-5  = rate (1=bad, 5=excellent)")
    print("  g     = mark as GOOD")
    print("  b     = mark as BAD")
    print("  +g    = good + rate (e.g. +4 = good + rate 4)")
    print("  n     = add note")
    print("  p     = play again")
    print("  f     = show features")
    print("  s     = skip")
    print("  q     = quit session")
    print()

    session_ratings = {}
    skipped = 0
    rated_count = 0

    for idx, wav_path in enumerate(wav_files):
        rel_path = str(wav_path.resolve().relative_to(REPO_ROOT))
        existing = notes_data.get("notes", {}).get(str(wav_path), {})

        print(f"\n{'='*60}")
        print(f"  [{idx + 1}/{len(wav_files)}] {wav_path.name}")
        print(f"  {rel_path}")

        # Show existing rating if any
        if existing:
            old_rating = existing.get("rating")
            old_flag = existing.get("flag", "")
            old_note = existing.get("note", "")
            flag_str = f" [{old_flag.upper()}]" if old_flag else ""
            rating_str = f" ★={old_rating}" if old_rating else ""
            note_str = f" \"{old_note}\"" if old_note else ""
            print(f"  Previous:{rating_str}{flag_str}{note_str}")

        # Compute and show features
        result = read_wav(wav_path)
        if result:
            samples, sr = result
            feats = compute_features(samples, sr)
            dur = _format_duration(feats["duration_ms"])
            rms = feats["rms"]
            print(f"  {dur} | RMS={rms:.3f} | {_feature_summary_line(feats)}")

        # Play the file
        print("  Playing...", end=" ", flush=True)
        _play_audio(wav_path, player)
        print()

        # Interactive rating loop
        while True:
            try:
                inp = input(f"  [{idx + 1}/{len(wav_files)}] Rate (1-5/g/b/note/p/feat/s/q): ").strip().lower()
            except (EOFError, KeyboardInterrupt):
                print("\nQuitting session.")
                _save_notes(notes_path, notes_data)
                return

            if inp == "q":
                _save_notes(notes_path, notes_data)
                print(f"\nSession saved. Rated: {rated_count}, Skipped: {skipped}")
                return

            if inp == "s":
                skipped += 1
                break

            if inp == "p":
                _play_audio(wav_path, player)
                continue

            if inp == "f":
                if result:
                    print(f"  Features: {json.dumps({k: round(v, 4) if isinstance(v, float) else v for k, v in feats.items() if isinstance(v, (int, float))}, indent=2)}")
                continue

            if inp == "n":
                try:
                    note = input("  Note: ").strip()
                except (EOFError, KeyboardInterrupt):
                    continue
                entry = notes_data.setdefault("notes", {}).setdefault(str(wav_path), {})
                entry["note"] = note
                entry["file"] = wav_path.name
                entry["rel_path"] = rel_path
                _save_notes(notes_path, notes_data)
                print(f"  Note saved.")
                continue

            if inp in ("g", "good"):
                entry = notes_data.setdefault("notes", {}).setdefault(str(wav_path), {})
                entry["flag"] = "good"
                entry["file"] = wav_path.name
                entry["rel_path"] = rel_path
                _save_notes(notes_path, notes_data)
                print(f"  Marked GOOD.")
                rated_count += 1
                break

            if inp in ("b", "bad"):
                entry = notes_data.setdefault("notes", {}).setdefault(str(wav_path), {})
                entry["flag"] = "bad"
                entry["file"] = wav_path.name
                entry["rel_path"] = rel_path
                _save_notes(notes_path, notes_data)
                print(f"  Marked BAD.")
                rated_count += 1
                break

            # Check for "+g" or "+4" style input
            if inp.startswith("+") and len(inp) > 1:
                rest = inp[1:]
                if rest.isdigit():
                    rating = int(rest)
                    if 1 <= rating <= 5:
                        entry = notes_data.setdefault("notes", {}).setdefault(str(wav_path), {})
                        entry["rating"] = rating
                        entry["flag"] = "good"
                        entry["file"] = wav_path.name
                        entry["rel_path"] = rel_path
                        _save_notes(notes_path, notes_data)
                        print(f"  Rated {rating}/5 + GOOD.")
                        rated_count += 1
                        break
                continue

            if inp.isdigit():
                rating = int(inp)
                if 1 <= rating <= 5:
                    entry = notes_data.setdefault("notes", {}).setdefault(str(wav_path), {})
                    entry["rating"] = rating
                    entry["file"] = wav_path.name
                    entry["rel_path"] = rel_path
                    if rating >= 4:
                        entry["flag"] = "good"
                    elif rating <= 2:
                        entry["flag"] = "bad"
                    _save_notes(notes_path, notes_data)
                    print(f"  Rated {rating}/5.")
                    rated_count += 1
                    break

            print("  Invalid input. Use 1-5, g, b, n, p, f, s, or q.")

    _save_notes(notes_path, notes_data)

    # Session summary
    ratings = []
    flags = {"good": 0, "bad": 0}
    for path_str, entry in notes_data.get("notes", {}).items():
        r = entry.get("rating")
        if r:
            ratings.append(r)
        fl = entry.get("flag", "")
        if fl in flags:
            flags[fl] += 1

    print(f"\n{'='*60}")
    print(f"  SESSION COMPLETE")
    print(f"{'='*60}")
    if ratings:
        avg = sum(ratings) / len(ratings)
        print(f"  Avg rating: {avg:.2f}/5 ({len(ratings)} rated)")
    print(f"  Good: {flags['good']}, Bad: {flags['bad']}")
    print(f"  Skipped: {skipped}")
    print(f"  Notes: {notes_path}")


def cmd_listening_report(args):
    """Show summary of listening notes with top/bottom rankings."""
    in_dir = Path(args.input_dir)
    notes_path = Path(args.notes) if args.notes else in_dir / "listening_notes.json"

    if not notes_path.exists():
        print(f"Error: {notes_path} not found.", file=sys.stderr)
        sys.exit(1)

    with open(notes_path) as f:
        notes_data = json.load(f)

    entries = list(notes_data.get("notes", {}).items())

    print("=" * 60)
    print("  LISTENING REPORT")
    print("=" * 60)
    print(f"\n  Source: {in_dir}")
    print(f"  Sessions: {notes_data.get('session_count', 0)}")
    print(f"  Total entries: {len(entries)}")
    print(f"  Last updated: {notes_data.get('last_updated', 'never')}")

    # Separate rated and flagged
    rated = []
    good = []
    bad = []
    noted = []

    for path_str, entry in entries:
        r = entry.get("rating")
        if r:
            rated.append((r, path_str, entry))
        fl = entry.get("flag", "")
        if fl == "good":
            good.append((path_str, entry))
        elif fl == "bad":
            bad.append((path_str, entry))
        if entry.get("note"):
            noted.append((path_str, entry))

    if rated:
        rated.sort(key=lambda x: x[0], reverse=True)
        avg = sum(r for r, _, _ in rated) / len(rated)
        print(f"\n  ├─ Average rating: {avg:.2f}/5")
        print(f"  ├─ Rated files: {len(rated)}")

        print(f"\n  Top 5:")
        for r, path_str, entry in rated[:5]:
            flag = f" [{entry.get('flag', '').upper()}]" if entry.get('flag') else ""
            note = f" — {entry['note']}" if entry.get('note') else ""
            print(f"    ★ {r}/5{flag}  {path_str.split('/')[-1]}{note}")

        print(f"\n  Bottom 5:")
        for r, path_str, entry in rated[-5:]:
            flag = f" [{entry.get('flag', '').upper()}]" if entry.get('flag') else ""
            note = f" — {entry['note']}" if entry.get('note') else ""
            print(f"    ★ {r}/5{flag}  {path_str.split('/')[-1]}{note}")

    if good:
        print(f"\n  Flagged GOOD: {len(good)} files")
        for path_str, entry in good[:5]:
            note = f" — {entry['note']}" if entry.get('note') else ""
            print(f"    ✓ {path_str.split('/')[-1]}{note}")

    if bad:
        print(f"\n  Flagged BAD: {len(bad)} files")
        for path_str, entry in bad[:5]:
            note = f" — {entry['note']}" if entry.get('note') else ""
            print(f"    ✗ {path_str.split('/')[-1]}{note}")

    if noted:
        print(f"\n  All notes ({len(noted)}):")
        for path_str, entry in noted:
            r = entry.get("rating", "-")
            fl = entry.get("flag", "")
            flag_str = f" [{fl.upper()}]" if fl else ""
            print(f"    [{r}]{flag_str} {path_str.split('/')[-1]}: {entry['note']}")

    if not rated and not good and not bad and not noted:
        print("\n  No ratings or notes found.")

    # Per-class breakdown if directory has class subdirs
    if rated:
        from collections import defaultdict
        class_ratings = defaultdict(list)
        for r, path_str, entry in rated:
            cls = path_str.split("/")[-2] if "/" in path_str else "?"
            class_ratings[cls].append(r)

        if len(class_ratings) > 1:
            print(f"\n  Per-class averages:")
            for cls in sorted(class_ratings.keys()):
                vals = class_ratings[cls]
                avg_c = sum(vals) / len(vals)
                print(f"    {cls:15s}: {avg_c:.2f}/5 ({len(vals)} files)")

    print()
