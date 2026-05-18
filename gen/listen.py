"""Listening UX v2: fast keyboard-driven audition with play/next/favorite/trash, A/B compare, export favorites."""
import json
import shutil
import subprocess
import sys
import time
from collections import defaultdict
from pathlib import Path
from typing import Optional

from gen import REPO_ROOT
from gen.io import read_wav
from gen.features import compute_features
from gen.rating import _save_rating


def _find_audio_player() -> Optional[str]:
    for player in ["aplay", "paplay", "ffplay", "gst-play-1.0", "play"]:
        if shutil.which(player):
            return player
    return None


def _play_audio(path: Path, player: Optional[str] = None):
    if not player:
        player = _find_audio_player()
    if not player:
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
    except Exception:
        pass


def _load_session(path: Path) -> dict:
    if path.exists():
        with open(path) as f:
            return json.load(f)
    return {"ratings": {}, "a_bracket": [], "b_bracket": [], "session_count": 0}


def _save_session(path: Path, data: dict):
    data["last_updated"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    path.write_text(json.dumps(data, indent=2))


def _list_files(in_dir: Path) -> list[Path]:
    wavs = sorted(in_dir.rglob("*.wav"))
    return wavs


def _format_duration(ms: float) -> str:
    if ms < 1000:
        return f"{ms:.0f}ms"
    return f"{ms/1000:.1f}s"


def cmd_listen(args):
    """Fast keyboard-driven listening session."""
    in_dir = Path(args.input_dir)
    if not in_dir.exists():
        print(f"Error: {in_dir} not found", file=sys.stderr)
        sys.exit(1)

    wav_files = _list_files(in_dir)
    if not wav_files:
        print(f"Error: no .wav files found in {in_dir}", file=sys.stderr)
        sys.exit(1)

    session_path = args.notes if args.notes else in_dir / "listen_session.json"
    session = _load_session(session_path)
    session["session_count"] = session.get("session_count", 0) + 1
    ratings = session.setdefault("ratings", {})
    a_slot = session.get("a_slot")
    b_slot = session.get("b_slot")

    player = _find_audio_player()
    has_audio = player is not None
    if not has_audio:
        print("No audio player found — install aplay, paplay, or ffplay")

    total = len(wav_files)
    idx = session.get("last_index", 0)
    if idx >= total:
        idx = 0

    rated_count = sum(1 for v in ratings.values() if v != "skip")
    skipped_count = sum(1 for v in ratings.values() if v == "skip")

    print(f"\ncShot Listen — {total} files")
    print(f"{'='*60}")
    print()
    print("Keys:")
    print("  ENTER/SPACE  play")
    print("  f            favorite")
    print("  t            trash")
    print("  g            good")
    print("  b            bad")
    print("  s            skip / next")
    print("  a            set A bracket (compare)")
    print("  B            set B bracket (compare)")
    print("  A/B          play A/B slot")
    print("  e            export favorites")
    print("  q            quit")
    print()

    while 0 <= idx < total:
        wav = wav_files[idx]
        rel = str(wav.relative_to(in_dir))
        current_rating = ratings.get(str(wav), "")

        cat = wav.parent.name if wav.parent != in_dir else ""
        cat_str = f" [{cat}]" if cat else ""

        result = read_wav(wav)
        feats = {}
        if result:
            samples, sr = result
            feats = compute_features(samples, sr)
            dur = _format_duration(feats.get("duration_ms", 0))
            cent = feats.get("spectral_centroid", 0)
            rms = feats.get("rms", 0)
            meta = f"{dur} | cent={cent:.0f}Hz | rms={rms:.3f}"
        else:
            meta = ""

        a_mark = " [A]" if str(wav) == a_slot else ""
        b_mark = " [B]" if str(wav) == b_slot else ""
        rating_dot = "★" if current_rating == "favorite" else "✗" if current_rating == "trash" else "✓" if current_rating == "good" else " " if current_rating == "bad" else " "
        prog = f"[{idx+1}/{total}]"

        print(f"  {prog}{cat_str} {rating_dot} {rel}")
        if meta:
            print(f"         {meta}")
        if current_rating:
            print(f"         [{current_rating}]")
        print()

        _play_audio(wav, player)

        while True:
            try:
                inp = input(f"  [{idx+1}/{total}] > ").strip().lower()
            except (EOFError, KeyboardInterrupt):
                inp = "q"

            if inp in ("", " "):
                _play_audio(wav, player)
                continue

            if inp == "s":
                ratings[str(wav)] = "skip"
                session["last_index"] = idx + 1
                _save_session(session_path, session)
                idx += 1
                break

            if inp == "n":
                session["last_index"] = idx + 1
                _save_session(session_path, session)
                idx += 1
                break

            if inp == "f":
                ratings[str(wav)] = "favorite"
                _save_rating({"file": rel, "rating": "favorite", "notes": "", "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())})
                _save_session(session_path, session)
                print(f"  ★ Favorited")
                idx += 1
                break

            if inp == "t":
                ratings[str(wav)] = "trash"
                _save_rating({"file": rel, "rating": "trash", "notes": "", "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())})
                _save_session(session_path, session)
                print(f"  ✗ Trashed")
                idx += 1
                break

            if inp == "g":
                ratings[str(wav)] = "good"
                _save_rating({"file": rel, "rating": "good", "notes": "", "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())})
                _save_session(session_path, session)
                print(f"  ✓ Good")
                idx += 1
                break

            if inp == "b":
                ratings[str(wav)] = "bad"
                _save_rating({"file": rel, "rating": "bad", "notes": "", "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())})
                _save_session(session_path, session)
                print(f"  ✗ Bad")
                idx += 1
                break

            if inp == "a":
                a_slot = str(wav)
                session["a_slot"] = a_slot
                _save_session(session_path, session)
                print(f"  Set A bracket: {rel}")
                continue

            if inp == "b":
                b_slot = str(wav)
                session["b_slot"] = b_slot
                _save_session(session_path, session)
                print(f"  Set B bracket: {rel}")
                continue

            if inp == "A":
                if a_slot:
                    _play_audio(Path(a_slot), player)
                continue

            if inp == "B":
                if b_slot:
                    _play_audio(Path(b_slot), player)
                continue

            if inp == "ab":
                if a_slot:
                    print(f"  A: {Path(a_slot).name}")
                    _play_audio(Path(a_slot), player)
                if b_slot:
                    print(f"  B: {Path(b_slot).name}")
                    _play_audio(Path(b_slot), player)
                continue

            if inp == "e":
                export_dir = in_dir / "_favorites"
                export_dir.mkdir(exist_ok=True)
                count = 0
                for fpath_str, rating in ratings.items():
                    if rating == "favorite":
                        src = Path(fpath_str) if Path(fpath_str).exists() else in_dir / fpath_str
                        if src.exists():
                            shutil.copy2(src, export_dir / src.name)
                            count += 1
                other_favs = []
                from gen.rating import _load_ratings
                for r in _load_ratings():
                    if r["rating"] == "favorite":
                        fp = REPO_ROOT / r["file"]
                        if fp.exists() and fp.suffix == ".wav":
                            shutil.copy2(fp, export_dir / fp.name)
                            count += 1
                print(f"  Exported {count} favorites → {export_dir}")
                continue

            if inp == "q":
                session["last_index"] = idx
                _save_session(session_path, session)
                rated = sum(1 for v in ratings.values() if v in ("favorite", "good", "bad", "trash"))
                skipped = sum(1 for v in ratings.values() if v == "skip")
                print(f"\nSession saved. Rated: {rated}  Skipped: {skipped}")
                print(f"Favorites: {export_dir}/" if 'export_dir' in dir() and export_dir.exists() else "")
                return

    _save_session(session_path, session)
    rated = sum(1 for v in ratings.values() if v in ("favorite", "good", "bad", "trash"))
    skipped = sum(1 for v in ratings.values() if v == "skip")
    favs = sum(1 for v in ratings.values() if v == "favorite")
    print(f"\n{'='*60}")
    print(f"Complete! All {total} files reviewed.")
    print(f"Rated: {rated}  Skipped: {skipped}  Favorites: {favs}")
    print(f"Session: {session_path}")

    if favs > 0:
        answer = input("\nExport favorites? (y/n): ").strip().lower()
        if answer == "y" or answer == "yes":
            export_dir = in_dir / "_favorites"
            export_dir.mkdir(exist_ok=True)
            count = 0
            for fpath_str, rating in ratings.items():
                if rating == "favorite":
                    src = Path(fpath_str) if Path(fpath_str).exists() else in_dir / fpath_str
                    if src.exists():
                        shutil.copy2(src, export_dir / src.name)
                        count += 1
            from gen.rating import _load_ratings
            for r in _load_ratings():
                if r["rating"] == "favorite":
                    fp = REPO_ROOT / r["file"]
                    if fp.exists() and fp.suffix == ".wav":
                        shutil.copy2(fp, export_dir / fp.name)
                        count += 1
            print(f"Exported {count} favorites → {export_dir}")


def cmd_listening_report(args):
    """Show summary of listening session with ratings."""
    in_dir = Path(args.input_dir)
    session_path = Path(args.notes) if args.notes else in_dir / "listen_session.json"

    if not session_path.exists():
        print(f"Error: {session_path} not found.")
        sys.exit(1)

    with open(session_path) as f:
        session = json.load(f)

    ratings = session.get("ratings", {})
    if not ratings:
        print("No ratings found in session.")
        return

    stats = defaultdict(int)
    for fpath, rating in ratings.items():
        stats[rating] += 1

    favorites = [f for f, r in ratings.items() if r == "favorite"]
    good = [f for f, r in ratings.items() if r == "good"]
    bad = [f for f, r in ratings.items() if r == "bad"]
    trash = [f for f, r in ratings.items() if r == "trash"]

    total = len(ratings)
    print(f"Listening Report — {in_dir}")
    print(f"{'='*60}")
    print(f"Total files reviewed: {total}")
    print(f"  Favorite: {stats.get('favorite', 0)}")
    print(f"  Good:     {stats.get('good', 0)}")
    print(f"  Bad:      {stats.get('bad', 0)}")
    print(f"  Trash:    {stats.get('trash', 0)}")
    print(f"  Skip:     {stats.get('skip', 0)}")
    print()

    if favorites:
        print(f"Favorites ({len(favorites)}):")
        for f in favorites[:20]:
            p = Path(f)
            print(f"  ★ {p.name}")
    if good:
        print(f"\nGood ({len(good)}):")
        for f in good[:10]:
            p = Path(f)
            print(f"  ✓ {p.name}")

    print(f"\nSession count: {session.get('session_count', 0)}")
    print(f"Last updated: {session.get('last_updated', 'never')}")
