# cShot — AI One-Shot Pack Generator

Generate, refine, rank, and export producer-ready one-shot packs from natural language prompts. No cloud, no API keys.

**Status:** Python CLI is the working backend. Rust/Tauri UI is experimental.

## Quick Start

```bash
# 1. Install
pip install -r requirements.txt

# 2. Generate your first one-shot
./cshot prompt "dark soft piano stab" --out outputs/my_first.wav

# 3. Hear what adjectives do
./cshot prompt "bright hard piano stab" --out outputs/bright_hard.wav
./cshot prompt "dark soft piano stab"  --out outputs/dark_soft.wav

# 4. Batch from a prompt
./cshot prompt "warm punchy kick" --count 10 --out outputs/kicks/
```

## Install

**Requirements:** Python 3.10+, pip.

```bash
git clone <repo-url> && cd cShot
pip install -r requirements.txt
```

That's it. No accounts, no API keys, no Docker. The `cshot` wrapper at repo root calls `python3 gen.py`.

## 10 Example Prompts

All generate single WAV files for immediate listening:

| # | Command | What You Get |
|---|---------|-------------|
| 1 | `./cshot prompt "dark soft piano stab" --out outputs/01.wav` | Mellow acoustic piano, soft attack |
| 2 | `./cshot prompt "bright hard piano stab" --out outputs/02.wav` | Bright piano with hard transient |
| 3 | `./cshot prompt "punchy kick 808" --out outputs/03.wav` | Kick drum with 808 sub |
| 4 | `./cshot prompt "warm synth pluck" --out outputs/04.wav` | Soft filtered synth pluck |
| 5 | `./cshot prompt "aggressive distorted bass" --out outputs/05.wav` | Distorted growly bass |
| 6 | `./cshot prompt "clean nylon guitar" --out outputs/06.wav` | Clean acoustic-guitar pluck |
| 7 | `./cshot prompt "cinematic impact fx" --out outputs/07.wav` | Big impact with long tail |
| 8 | `./cshot prompt "mellow synth pad" --out outputs/08.wav` | Soft evolving pad |
| 9 | `./cshot prompt "bright synth chord" --out outputs/09.wav` | Major chord stab |
| 10 | `./cshot prompt "lo-fi glitch texture" --out outputs/10.wav` | Noisy glitch with lo-fi character |

Try replacing adjectives: `bright`/`dark`, `soft`/`punchy`, `clean`/`distorted`, `dry`/`wet`, `narrow`/`wide`.

## Supported Sound Families

| Family | Nouns | Profiles |
|--------|-------|----------|
| Piano | `piano`, `keys` | acoustic, felt, dark, bright, lo-fi |
| Synth | `stab`, `pluck`, `pad`, `lead`, `chord`, `synth` | stab, pluck, pad, chord, lead, bass |
| Bass | `bass`, `808`, `reese`, `sub` | 808, reese, distorted, pluck, fm, hybrid |
| Guitar | `guitar`, `nylon`, `acoustic` | nylon, muted, bright, dark, processed, reversed |
| FX | `impact`, `fx`, `riser`, `glitch`, `noise`, `vinyl`, `texture` | impact, downlifter, riser, glitch, noise_hit, vinyl, air |
| Drums | `kick`, `snare`, `clap`, `hat`, `hihat`, `open_hat` | Basic synthesized drums |

## Adjectives

| Category | Words |
|----------|-------|
| Brightness | bright, dark, warm, mellow, edgy, crisp |
| Attack | soft, hard, punchy, gentle, aggressive |
| Character | distorted, clean, lo-fi, glitchy, metallic |
| Space | dry, wet, narrow, wide, intimate, huge |
| Style | vintage, modern, analog, digital, dusty, glossy |

`./cshot prompt "dark bright piano"` warns about conflicting descriptors.

## All Commands

```bash
./cshot scan                              # Scan reference folders
./cshot profiles                          # Build class profiles
./cshot oneshot <class> --out file.wav    # Single one-shot
./cshot batch --class kick --count 20     # Batch generation
./cshot prompt "dark soft piano"          # Natural language prompt
./cshot prompt "punchy kick" --count 10   # Batch from prompt
./cshot qa                                # QA audit
./cshot all                               # Full pipeline
./cshot piano-gen acoustic --count 10     # Piano generation
./cshot synth-gen stab --count 10         # Synth generation
./cshot bass-gen 808 --count 10           # Bass generation
./cshot guitar-gen nylon --count 10       # Guitar generation
./cshot fx-gen impact --count 10          # FX generation
./cshot mvp-audit                         # Full 100-file audit
```

## Output Format

- 44100 Hz, 16-bit mono WAV
- Normalized to ~0.9 peak amplitude
- DAW-friendly filenames

## Known Working Path (Python CLI)

The Python synthesis engine (`gen/`) is the **official working backend**. It produces reference-grounded outputs across all families and supports natural language prompting with 60+ adjectives.

To verify your setup:
```bash
./cshot prompt "dark soft piano stab" --out outputs/test.wav
# → Writes a ~1.5s WAV to outputs/test.wav
# → Outputs feature summary (centroid, attack, RMS, etc.)
```

## Known Broken / Experimental (Rust/Tauri)

The Rust DSP engine and Tauri desktop app were early prototypes. They contain ambitious but unverified code:

- **Tauri desktop app** (`npm run tauri dev`) — UI shell works, generation may lag behind Python
- **Rust CLI** (`cargo run --bin cshot-cli`) — mirrors some Python commands, not in sync
- **Rust synthesis engine** (`src-tauri/src/audio/`) — 5-layer resynthesis framework, needs parity with Python
- **Plugin prototype** (`cargo run --bin cshot-plugin`) — standalone VST prototype, not production-ready

Development priority: Python CLI first. Rust/Tauri will be revisited once generation quality stabilizes.

## Troubleshooting

### `pip install` fails

```bash
# Install system deps for soundfile
# Ubuntu/Debian:
sudo apt-get install libsndfile1
# macOS:
brew install libsndfile
```

### `ModuleNotFoundError: No module named 'gen'`

Run commands from repo root (where `cshot` and `gen/` live):
```bash
cd /path/to/cShot
./cshot prompt "dark soft piano"
```

### Generated WAV is silent / very short

Some prompt + family combinations produce thin output. Try adding adjectives:
```bash
./cshot prompt "warm punchy kick" --out outputs/test.wav    # Better
# vs
./cshot prompt "kick" --out outputs/test.wav                # May be thin
```

### `outputs/` not created

The script creates the directory automatically. Check permissions or create it:
```bash
mkdir -p outputs
```

### "Unknown command" error

```bash
./cshot --help   # List all commands
./cshot prompt "dark soft piano"   # "prompt" is one word
```

### WAV won't play

Ensure the file is valid:
```bash
file outputs/test.wav
# Should show: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 44100 Hz
```

### Pitch sounds wrong

Tonal families (piano, keys) are locked to specific pitches by default. Use `--count 10` to get variation.

### I want more sounds

```
./cshot mvp-audit   # Generates 100 files across 20+ categories
```

## Engine Status

| Engine | Status | Notes |
|--------|--------|-------|
| **Python (`gen/`)** | **Official** | Stable, reference-informed, natural language prompting |
| **Rust/Tauri** | **Experimental** | Needs parity; not the current development focus |

## License

MIT

Generated sounds from the local engine are safe for commercial use. See [docs/COPYRIGHT-SAFETY.md](docs/COPYRIGHT-SAFETY.md).
