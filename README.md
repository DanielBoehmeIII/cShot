# cShot — AI One-Shot Pack Generator

Generate, refine, rank, and export producer-ready one-shot packs from natural language prompts. No cloud, no API keys.

**Status:** Python CLI is the working backend. Rust/Tauri UI is experimental.

## Quick Start

```bash
# 1. Install
pip install -r requirements.txt

# 2. Generate your first one-shot
./cshot prompt "dark soft piano stab" --out outputs/my_first.wav

# 3. One-command producer mode
./cshot make "dark rnb one shot pack" --count 100

# 4. UI (optional: pip install gradio)
python3 app.py
```

## Install

```bash
# Option A: Manual
pip install -r requirements.txt
./cshot prompt "dark soft piano" --out outputs/test.wav

# Option B: Package (adds `cshot` to PATH)
pip install -e .
cshot prompt "dark soft piano" --out outputs/test.wav

# Option C: Install script (auto-detects libsndfile)
./install.sh
```

## 10 Example Prompts

| # | Command |
|---|---------|
| 1 | `./cshot prompt "dark soft piano stab" --out outputs/01.wav` |
| 2 | `./cshot prompt "bright hard piano stab" --out outputs/02.wav` |
| 3 | `./cshot prompt "punchy kick 808" --out outputs/03.wav` |
| 4 | `./cshot prompt "warm synth pluck" --out outputs/04.wav` |
| 5 | `./cshot prompt "aggressive distorted bass" --out outputs/05.wav` |
| 6 | `./cshot prompt "clean nylon guitar" --out outputs/06.wav` |
| 7 | `./cshot prompt "cinematic impact fx" --out outputs/07.wav` |
| 8 | `./cshot prompt "mellow synth pad" --out outputs/08.wav` |
| 9 | `./cshot prompt "bright synth chord" --out outputs/09.wav` |
| 10 | `./cshot prompt "lo-fi glitch texture" --out outputs/10.wav` |

## Commands

### Generation
| Command | Description |
|---------|-------------|
| `cshot prompt <text> [--seed N] [--name-template T] [--out path]` | Generate from natural language |
| `cshot oneshot <class> --out file.wav` | Generate a single class |
| `cshot batch --class kick --count 20` | Batch generation |
| `cshot <family>-gen <profile> --count N` | Family-specific: piano-gen, synth-gen, bass-gen, guitar-gen, fx-gen |
| `cshot genre <name> --count N` | Genre-aware generation (trap, drill, rage, ambient, house, techno, hyperpop, rnb, cinematic, lo-fi) |
| `cshot similar <ref.wav> --count N` | Generate variations near a reference sample |
| `cshot variations <ref.wav> --spread medium --count N` | Variation cloud around a reference |
| `cshot blend a.wav b.wav --mode mix|envelope` | Blend two samples |
| `cshot regenerate --metadata file.json` | Reproduce from metadata sidecar |
| `cshot make "<pack theme>" --count N` | One-command: generate + polish + rank + export |

### Presets & Themes
| Command | Description |
|---------|-------------|
| `cshot save-preset <name> --from metadata.json` | Save generation config as preset |
| `cshot preset list` | List saved presets by family |
| `cshot preset generate <name> [--count N]` | Generate from a saved preset |
| `cshot theme "Noir Piano Kit"` | Generate a themed pack (5 built-in themes) |
| `cshot pack "<theme>" --count N --out dir/` | Batch pack generation (6 categories) |

### Quality & Rating
| Command | Description |
|---------|-------------|
| `cshot rate <file> --rating good|bad|favorite|trash` | Rate a generated file |
| `cshot ratings summary` | Show rating summary |
| `cshot favorites` | List all favorited files |
| `cshot rank <dir>` | Auto-rank files by quality + ratings |
| `cshot top <dir> --n 10` | Show top-ranked files |
| `cshot polish <file|dir> [--target-db -1]` | Trim, fade, normalize, validate |
| `cshot refine-feedback <file> "less harsh, more warm"` | Natural language refinement |
| `cshot pack-audit <dir>` | Pack-level quality audit |
| `cshot mvp-audit` | Full 100-file QA audit |

### Analysis & Search
| Command | Description |
|---------|-------------|
| `cshot taste` | Show learned preference profile |
| `cshot prompt-history` | Show prompt rating history |
| `cshot search-ref <query>` | Search reference library |
| `cshot dataset-health` | Check reference library health |
| `cshot scan` | Scan reference folders |
| `cshot detect-pitch <file>` | Detect pitch and MIDI note |
| `cshot compare-prompt dirA dirB` | Compare two generated sets |

### Utilities
| Command | Description |
|---------|-------------|
| `cshot --help` | All commands |
| `cshot <cmd> --help` | Per-command help |
| `python3 app.py` | Launch Gradio UI |

## Supported Sound Families

| Family | Nouns | Profiles |
|--------|-------|----------|
| **Piano** | piano, keys | acoustic, bright, dark, soft, felt, lo-fi, compressed, bell, rhodes |
| **Synth** | stab, pluck, pad, chord, lead, synth | stab, pluck, pad, chord, lead, bass, fm, wavetable |
| **Bass** | bass, 808, reese, sub | 808, reese, distorted, pluck, fm |
| **Guitar** | guitar, nylon | nylon, muted, bright, dark, processed, reversed, chopped |
| **FX** | impact, fx, riser, glitch, noise, vinyl, texture | impact, downlifter, riser, glitch, noise_hit, vinyl, air, sub_hit, tonal_hit |
| **Drums** | kick, snare, clap, hat | Basic synthesized drums |

## Adjectives (50+)

`bright` `dark` `warm` `mellow` `edgy` `crisp` `expensive` `dusty` `vintage` `glossy` `crunchy`
`soft` `hard` `punchy` `gentle` `aggressive`
`distorted` `clean` `lo-fi` `glitchy` `metallic` `analog` `digital`
`wide` `narrow` `big` `small` `intimate` `huge` `tiny`
`dry` `wet`
`short` `long` `sustained` `staccato`
`airy` `noisy` `smooth` `rough`
`high` `low` `mid`

Conflicting pairs (e.g. `dark bright`) are detected and reported.

## Feature Summary

- **44100 Hz, 16-bit mono WAV**, normalized to ~0.95 peak
- **Metadata sidecar** (`.json`) stores prompt, seed, family, profile, overrides
- **Deterministic mode**: `--seed 42` produces bit-identical output
- **Ratings**: stored in `ratings.jsonl`, used by ranking
- **Presets**: saved to `presets/<family>/<name>.json`
- **Taste profile**: `taste_profile.json` tracks family preferences

## Known Working Path (Python CLI)

The Python synthesis engine (`gen/`) is the **official working backend**.

To verify:
```bash
./cshot prompt "dark soft piano stab" --out outputs/test.wav
```

## Known Broken / Experimental (Rust/Tauri)

- Tauri desktop app: UI shell works, generation not in sync with Python
- Rust CLI: mirrors some Python commands, not maintained
- Plugin prototype: standalone VST prototype, not production-ready

Development priority: Python CLI first.

## Troubleshooting

```bash
# libsndfile not found
sudo apt-get install libsndfile1   # Ubuntu/Debian
brew install libsndfile            # macOS

# ModuleNotFoundError: No module named 'gen'
# Run from repo root
cd /path/to/cShot

# WAV won't play
file outputs/test.wav
# Should show: RIFF ... WAVE audio, Microsoft PCM, 16 bit, mono 44100 Hz

# More sounds
./cshot mvp-audit   # 100 files across 20 categories
```

## License

MIT. Generated sounds are safe for commercial use.
