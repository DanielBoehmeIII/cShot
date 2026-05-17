"""Pitch detection CLI: detect pitch, note name, and key for audio files."""

import json
import sys
import time
from pathlib import Path

import numpy as np

from gen import REPO_ROOT
from gen.io import read_wav
from gen.features import detect_pitch_full, hz_to_note, NOTE_NAMES


def cmd_detect_pitch(args):
    """Detect pitch, note, and key for audio files in a directory."""
    in_path = Path(args.input)
    if not in_path.exists():
        print(f"Error: {in_path} not found", file=sys.stderr)
        sys.exit(1)

    # Collect files
    if in_path.is_dir():
        wav_files = sorted(in_path.rglob("*.wav"))
        if not wav_files:
            print(f"Error: no .wav files found in {in_path}", file=sys.stderr)
            sys.exit(1)
    else:
        wav_files = [in_path]

    print(f"Analyzing pitch for {len(wav_files)} file(s)...")
    print()

    results = []
    all_pitches = []

    for wav_path in wav_files:
        result = read_wav(wav_path)
        if result is None:
            print(f"  ✗ {wav_path.name:40s} could not read")
            continue
        samples, sr = result
        info = detect_pitch_full(samples, sr)
        info["file"] = wav_path.name
        results.append(info)
        if info["confidence"] > 0.3:
            all_pitches.append(info["pitch_hz"])

        # Build display
        note_str = f"{info['note_name']:>6}" if info["note_name"] != "---" else "  ---"
        conf_str = f"conf={info['confidence']:.2f}"
        midi_str = f"MIDI={info['midi_note']:.1f}" if info["midi_note"] > 0 else ""
        pitch_str = f"{info['pitch_hz']:>7.1f}Hz" if info["pitch_hz"] > 0 else "  no pitch"

        print(f"  {wav_path.name:40s} {pitch_str} {note_str}  {midi_str:12s} {conf_str}")

    # Summary
    if len(results) > 1:
        print(f"\n  {'='*55}")
        print(f"  PITCH SUMMARY")
        print(f"  {'='*55}")

        pitches = [r["pitch_hz"] for r in results if r["pitch_hz"] > 0]
        confidences = [r["confidence"] for r in results if r["confidence"] > 0]

        if pitches:
            avg_pitch = np.mean(pitches)
            avg_note = hz_to_note(avg_pitch)
            avg_conf = np.mean(confidences)
            print(f"  Average pitch:    {avg_pitch:.1f}Hz ({avg_note})")
            print(f"  Average confidence: {avg_conf:.2f}")
            print(f"  Pitch range:      {min(pitches):.1f}Hz - {max(pitches):.1f}Hz")

        # Key estimation
        from gen.features import estimate_key
        key_name, key_conf = estimate_key(all_pitches)
        if key_conf > 0.3:
            print(f"  Estimated key:    {key_name} (confidence={key_conf:.2f})")
        else:
            print(f"  Key estimation:   inconclusive (low pitch content)")

        # Confidence distribution
        high_conf = sum(1 for r in results if r["confidence"] >= 0.8)
        mid_conf = sum(1 for r in results if 0.3 <= r["confidence"] < 0.8)
        low_conf = sum(1 for r in results if r["confidence"] < 0.3)
        print(f"  Pitch detected:   {high_conf} high, {mid_conf} medium, {low_conf} low/no")

    # Save results
    if args.output:
        output = {
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "files_analyzed": len(results),
            "results": results,
        }
        out_path = Path(args.output)
        out_path.write_text(json.dumps(output, indent=2))
        print(f"\n  Results written to {out_path}")
