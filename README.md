# cShot — Custom One-Shot Kits for Producers

Create producer-ready one-shot kits from songs, samples, genres, and vibes. No cloud, no API keys.

```
cshot make "dark rnb one shot kit"     # Generate from a vibe
cshot from-song song.wav               # Generate from a song
cshot from-sample clap.wav             # Generate from a sample
```

## Quick Start

```bash
pip install -r requirements.txt
./cshot make "dark rnb one shot kit" --count 50
```

## 3 Core Commands

| Command | Description |
|---------|-------------|
| `cshot make "dark rnb one shot kit"` | Generate a complete kit from a description |
| `cshot from-song track.wav` | Analyze a song, generate matching kit |
| `cshot from-sample kick.wav` | Build a mini kit around one sample |

## Producer Tools

| Command | Description |
|---------|-------------|
| `cshot listen <dir>` | Interactive listening session |
| `cshot rate <file> --rating favorite\|trash` | Rate individual sounds |
| `cshot favorites` | List your favorited sounds |
| `cshot rank <dir>` | Rank sounds by quality |
| `cshot taste` | Show your learned taste profile |

## Advanced (Lab)

All research and development commands are under `cshot lab`:

```
cshot lab prompt "dark soft piano"     # Single-shot generation
cshot lab piano-gen acoustic           # Family-specific generation
cshot lab scan                         # Reference library tools
cshot lab pack-census                  # Pack analysis
cshot lab --help                       # Full list of lab commands
```

## Examples

```bash
# 1. Generate a 50-sound dark RnB kit
./cshot make "dark rnb one shot kit" --count 50

# 2. Generate from a song you're working on
./cshot from-song my_beat.wav --count 40

# 3. Build sounds around a sample you like
./cshot from-sample my_clap.wav --count 20

# 4. Curate your results
./cshot listen outputs/kits/my_kit
./cshot rate outputs/kits/my_kit/kick_001.wav --rating favorite
./cshot favorites
```

## Install

```bash
pip install -r requirements.txt
./cshot make "dark rnb one shot kit"
```

Or install to PATH:
```bash
pip install -e .
cshot make "dark rnb one shot kit"
```

## Output

- 44100 Hz, 16-bit mono WAV, normalized to -1dB peak
- Organized by category (kicks, snares, hats, etc.)
- Metadata sidecar per file (prompt, seed, family)
- Manifest + README per kit
- Rating history in `ratings.jsonl`

## Status

Python CLI is the working backend. Rust/Tauri UI is experimental.

For questions and feedback: https://github.com/anomalyco/cShot/issues
