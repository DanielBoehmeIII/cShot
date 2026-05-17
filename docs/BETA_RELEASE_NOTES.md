# cShot Beta Release Notes — v2.0

## What cShot Does

cShot is a local-first, competitive one-shot sound design tool. Upload a reference → it understands the sound → recreate it → edit with prompts → export-ready instantly. Faster and more controllable than browsing sample packs or generic AI generation.

### Core Workflow

```
reference sound → locally recreated intelligently → editable with prompts → export-ready instantly
```

### Key Features (v2)

- **Reference Sound Recreation** — Upload any one-shot. cShot analyzes its envelope, transients, body, tail, sub, noise, and spectrum. Then recreates it locally with high-fidelity matching across 6+ perceptual dimensions.
- **Prompt-to-Sound Editing** — 60+ semantic descriptors (punchy, dark, bright, warm, aggressive, clean, distorted, cinematic, analog, tight, huge, metallic, soft, modern, etc.) with intensity scaling, conflict resolution, and genre-aware interpretation.
- **Reference + Prompt Fusion** — Combine uploaded sounds with prompt-based edits. "Make this snare darker", "preserve transient but shorten tail", "keep the vibe but make it cleaner". The combined workflow is cShot's killer feature.
- **5-Layer Resynthesis Engine** — Transient, body, noise, sub, tail layers with per-layer modulation, analog saturation, multi-band compression, and adaptive dynamics for mix-ready output.
- **Constrained Variation Engine** — 18 controlled variations with quality-aware filtering. Modes: safer, balanced, experimental, more aggressive, more polished, closer to original. Most variants feel intentionally designed.
- **Advanced Audio Intelligence** — Infers sound category, transient style, envelope type, aggressiveness, brightness, genre role, mix role. Uses this for better recreation, prompt interpretation, and variation generation.
- **Sound Quality Pipeline** — Multi-stage saturation (tape → tube → soft-clip), lookahead limiter, adaptive compressor, de-esser, DC removal, subsonic filter, spectral balance, transient shaping, anti-click processing.
- **Smart Recreation Ranking** — Rank results by closest, cleanest, most aggressive, most experimental, best quality, most novel, or balanced. Combines spectral similarity, envelope match, quality score, user affinity, and novelty.
- **Workflow Speed** — Quick branch from any sound, batch favorite/export, recent prompts store, keyboard shortcuts, undo/redo, fast compare.
- **DAW Automation Ready** — Plugin prototype exposes 8 automation parameters + MIDI note trigger
- **Batch Render** — CLI supports batch rendering multiple variants
- **CLI Recreate** — `cshot-cli recreate` generates approximations with similarity ranking
- **Sound Design Recipes** — Pre-built and custom recipe templates for quick generation
- **Sample Import** — Import existing WAV/MP3 samples, analyze and auto-tag them
- **Folder Import** — Bulk import folders with safety limits and duplicate detection
- **Pack Builder** — Group sounds into packs with cohesion analysis
- **Export to Desktop** — One-click WAV export with DAW-friendly semantic filenames
- **Integrity Tools** — Scan for missing/orphan files, repair metadata
- **Local-First** — All data stored locally. No cloud accounts required

## What Users Should Test (v2)

### Critical: Reference Recreation
1. **Upload a kick/snare/hat** → click Recreate → listen to the 5+ approximations → rate how close they feel
2. **Upload a complex sound** (layered, with tail, effect) → check if the spectral + envelope match holds
3. **Try "closest" vs "cleaner" vs "more experimental" ranking** → does the right variant rise to the top?

### Critical: Prompt Editing
4. **"make this kick darker"** → upload a kick → add the prompt → hear the fusion result
5. **"preserve transient but shorten tail"** → upload any sound with a long tail
6. **"keep the vibe but make it cleaner"** → test on a noisy/distorted sound
7. **"turn this into techno"** → genre shift preserves character but changes vibe

### Smart Variants
8. **Mode testing**: safer → balanced → experimental → aggressive → polished → closer
9. **New variants**: modern, aggressive, polished — do they feel intentionally designed?

### Prompt Predictability
10. **Intensity modifiers**: "very punchy" vs "slightly punchy" vs "extremely bright"
11. **Conflict resolution**: "bright but warm", "aggressive but clean" — does the net effect make sense?
12. **Compound prompts**: "punchy snare with a long tail but not harsh"

### All Sound Types
13. **Kick, snare, closed/open hat, clap, tom, perc, bass, FX** — each should feel competitive

### Workflow Speed
14. **Quick branch** from any sound → immediate variation
15. **Batch favorite/export** → select multiple sounds
16. **Recent prompts** → pick from history
17. **Keyboard shortcuts**: ⌘Z undo, ⌘⇧Z redo, R regenerate, space play/stop, ⌘1 generator, ⌘2 library

## Architecture Overview

```
Frontend:  React 18 + TypeScript + Vite + Tailwind
Backend:   Rust (Tauri v2) + SQLite + DSP engine
Engine:    cShot Engine (local synthesis, analysis, transformation)
           ├── synthesize.rs     — 10 advanced drum synthesis modules
           ├── analyze.rs        — Full audio analysis pipeline + spectral profiling
           ├── resynthesize.rs   — 5-layer resynthesis + 16 variants + harmonics
           ├── hybrid.rs         — Hybrid sample+synthesis engine
           ├── recreate.rs       — Multi-band recreation + transient profiling
           ├── transform.rs      — Prompt-based + DSP transformation
           ├── prompt_dsp.rs     — 60+ descriptors + semantic graph + genre scaling
           ├── embeddings.rs     — 64-dim Sound DNA embedding
           ├── dsp.rs            — Biquad filters, limiters, transient shaping
           └── process.rs        — Repair chain + validation
Storage:   Content-addressed WAV files + SQLite metadata
CLI:       cshot-cli (generate, analyze, transform, recreate, render, benchmark)
Plugin:    cshot-plugin (standalone prototype, 8 params, MIDI, presets)
```

## Known Limitations

| Limitation | Impact | Status |
|------------|--------|--------|
| Mono audio only | No stereo/spatial sounds | Planned post-beta |
| Local synthesis quality | Improving but not matching high-end hardware synths yet | Continuous improvement |
| No VST3/AU plugin | Must export and import to DAW | Post-beta — prototype exists |
| Basic auto-tagging | Tags may be imprecise for some sounds | Improvement planned |
| No multi-language prompts | English only | Post-beta |
| Mock embeddings only | Embedding-based similarity uses analysis, not ML models | Ready for real model swap |

## How to Install and Run

```bash
npm install
npm run tauri dev     # Development
npm run tauri build   # Production build
```

### CLI

```bash
cargo run --bin cshot-cli -- generate "punchy kick 140bpm" --output kick.wav
cargo run --bin cshot-cli -- analyze input.wav
cargo run --bin cshot-cli -- transform input.wav "darker shorter" --output transformed.wav
cargo run --bin cshot-cli -- recreate input.wav --count 4 --output best.wav
cargo run --bin cshot-cli -- render "warm sub bass" --count 10 --dir ./my_pack
cargo run --bin cshot-cli -- list-recipes
cargo run --bin cshot-cli -- benchmark
```

### Plugin Prototype

```bash
cargo run --bin cshot-plugin -- output.wav kick 60 0.5 0.6
cargo run --bin cshot-plugin -- output.wav snare 200 0.7 0.8 0.3 0.1 0.0 my_snare_preset
```

## What's Coming Next

- **VST3/AU Plugin** — Generate sounds directly in your DAW
- **Real-Time Generation** — Sub-second generation for immediate preview
- **Stereo Support** — Stereo field control and spatial sounds
- **Improved Auto-Tagging** — ML-based tag recommendations
- **Cloud Sync** — Optional sync of recipes and favorites across devices
- **Marketplace** — Share and sell sound design recipes
- **Fine-Tuning** — Personalize the generation model to your taste
- **Sound Graph** — Visual discovery through sonic similarity
