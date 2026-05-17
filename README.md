# cShot — Local Sound Design Laboratory

A desktop app for generating, recreating, editing, and transforming production-ready one-shot audio samples — entirely locally.

Built with Tauri v2 + React + Rust.

## Quick Start

```bash
npm install
npm run tauri dev
```

No API keys, no accounts, no cloud dependencies. cShot works immediately after install.

## Features

### Sound Generation
- **Text-to-one-shot** — Type a prompt like "punchy kick 140bpm" and get a WAV in under 500ms
- **Resynthesis engine** — 5-layer modular synthesis (transient, body, noise, sub, tail) per sound type
- **Advanced drum synthesis** — Layered kick, advanced snare, metallic hat (FM), multi-hit clap, FM percussion, resonant impact, tonal percussion, cinematic boom, UI click, bass hit
- **10 sound types** — Kick, snare, closed/open hat, clap, tom, perc, bass, FX, and more
- **Prompt-to-DSP** — 60+ natural language descriptors with semantic graph (punchy, airy, crunchy, warm, glossy, vintage, analog, digital, etc.)
- **Genre presets** — 25+ genre templates with genre-specific descriptor interpretation (trap, drill, house, techno, lo-fi, dubstep, etc.)
- **Recipe presets** — 18 built-in professional sound recipes
- **Smart variants** — 16 constrained variations with quality-aware filtering (brighter, darker, punchier, softer, shorter, longer, distorted, cleaner, subbier, airier, noisier, tighter, fattier, metallic, thinner, warmer)
- **Mode control** — Safer / balanced / experimental variation modes

### Audio Analysis
- **Full analysis pipeline** — Duration, RMS, peak, loudness LUFS, noise floor, spectral centroid/rolloff/brightness
- **Envelope extraction** — Attack/decay/tail detection with envelope curve
- **Multi-band spectral profile** — 64-bin spectral profile for detailed timbre analysis
- **Transient detection** — Onset strength, count, timing via spectral flux
- **Pitch estimation** — Autocorrelation-based fundamental frequency detection
- **Silence detection** — Leading/trailing silence measurement
- **Transient profiling** — Sharpness, spectral spread, onset shape

### Sound Recreation
- **Recreate any sound** — Analyze → extract sound DNA → choose synthesis strategy → generate approximations → rank by similarity
- **Multi-band similarity scoring** — 6-dimension comparison (envelope attack/body/tail, spectral low/mid/high, RMS, transient, duration)
- **Analysis-driven params** — Attack time, decay, tail, brightness, noise, sub, body gain all extracted from reference
- **Fidelity control** — 0-100% closeness to original with preservation constraints
- **Preserve toggles** — Keep transient/body/tail during recreation

### Hybrid Sample + Synthesis Engine
- **Layer synthesis onto source audio** — Blend original with synthesized layers
- **Transient replacement** — Replace or blend transient region
- **Body replacement** — Replace or blend tonal body
- **Tail regeneration** — Replace or regenerate tail with synthesis
- **Sub reinforcement** — Add synthesized sub bass to any sound
- **Spectral blending** — Match spectral profile between original and synthesis
- **Preservation controls** — Preserve original transient, tail, pitch, rhythm, texture

### Sound Transformation
- **Prompt-based editing** — "make this darker, shorter, and punchier" actually changes the sound
- **12 DSP operations** — Reverse, saturate, filter, pitch shift, transient shape, add sub/noise/click, brightness tilt, duration scale
- **Parent-child lineage** — Every transform links back to source

### Library & Export
- **Library** — Browse, search, filter, favorite, delete all your sounds
- **Export** — One-click WAV export to Desktop with semantic filenames (DAW-friendly)
- **Batch export** — Export all variants at once, export favorite packs as ZIP
- **Import** — Import WAV/MP3 files with auto-analysis and tagging
- **Repair chain** — Normalize, trim silence, fade, brighten/darken, punch, add sub, saturate, soften, sharpen, compress
- **Score** — Every sound gets an automated quality score (0-100)
- **Undo/Redo** — Full generation history with undo/redo support
- **Quick compare** — Compare current and previous generation

### Producer Workflow
- **Keyboard shortcuts** — Full keyboard-driven workflow (⌘1/2 views, Space play, R regenerate, ⌘E export, ⌘F favorite, ⌘Z undo)
- **Export folder** — Organized exports with DAW-friendly filenames
- **Rapid iteration** — Regenerate (R), undo (⌘Z), redo (⌘⇧Z) without touching mouse
- **Drag/drop ready** — Reference audio drag-drop support
- **Pack system** — Organize sounds into packs, export packs as ZIP

### Plugin Ready
- **DSP core separated** — No app-level dependencies in audio engine
- **CLI harness** — `cargo run --bin cshot-cli` for headless generation, analysis, transform, recreate, batch render
- **Plugin prototype** — Standalone binary with 8 DAW automation parameters, MIDI note trigger, preset save/load
- **VST3/CLAP compatible** — DSP engine ready for nih-plug wrapping
- **MIDI support** — Note On trigger, velocity gain, pitch bend modulation
- **Preset system** — JSON-based preset save/load for plugin parameters

### Sound DNA + Similarity
- **64-dim sound embedding** — Rich vector representation (transient profile, envelope, spectral, perceptual, temporal, type encoding)
- **Hybrid similarity** — Combines metadata similarity + embedding cosine similarity
- **Similarity search** — Find similar sounds in your library
- **Filter by descriptors** — Filter library by tags and descriptors

### Privacy
- Local-first, no accounts
- No data leaves your machine
- Clear session memory

## Beta Status

cShot is in **Beta** — the core workflow is stable and usable for real production.
All generation happens locally with zero cloud dependencies.

### What's in Beta

- ✓ Local DSP engine with 5-layer resynthesis and advanced drum synthesis
- ✓ Prompt-to-DSP mapping with 60+ descriptors and semantic graph
- ✓ Reference recreation with multi-band similarity scoring
- ✓ Desktop app with full generation, library, export workflow
- ✓ Undo/redo, keyboard shortcuts, quick iteration
- ✓ Hybrid sample + synthesis engine
- ✓ Smart constrained variation system
- ✓ Sound DNA embedding and similarity search
- ✓ CLI tool for headless generation, batch rendering, benchmark
- ✓ Plugin prototype with DAW automation and MIDI support

### What's NOT in Beta

- VST3/AU plugin (standalone only, plugin prototype exists)
- Marketplace/social features
- Cloud sync
- Stereo support (mono only, stereo planned)

## Architecture

```
Frontend:  React 18 + TypeScript + Vite + Tailwind
Backend:   Rust (Tauri v2) + SQLite + DSP engine
Engine:    cShot Engine (local synthesis, analysis, transformation)
           ├── synthesize.rs     — Layer-based resynthesis (5 layers)
           ├── analyze.rs        — Full audio analysis pipeline
           ├── resynthesize.rs   — Category-specific synthesis recipes
           ├── transform.rs      — Prompt-based + DSP transformation
           ├── recreate.rs       — Reference recreation + similarity scoring
           ├── prompt_dsp.rs     — NLP-to-synthesis-parameter mapping
           ├── dsp.rs            — Filters, EQ, transient shaping
           └── process.rs        — Repair chain + validation
Storage:   Content-addressed WAV files + SQLite metadata
CLI:       cshot-cli (headless generation, analysis, benchmark)
Plugin:    cshot-plugin (standalone plugin prototype)
```

## Build

```bash
npm run tauri build     # Desktop app
cargo run --bin cshot-cli  # CLI tool
cargo run --bin cshot-plugin  # Plugin prototype
```

## Configuration

cShot works out of the box with no configuration.

For optional cloud providers, see the Settings panel in the app.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| ⌘1 | Generator view |
| ⌘2 | Library view |
| Space | Play / Stop |
| Enter | Generate |
| R | Regenerate |
| ⌘E | Export |
| ⌘F | Favorite |
| ? | Shortcuts help |

## Commercial Use

Generated sounds from cShot's local engine are safe for commercial use.
See [docs/COPYRIGHT-SAFETY.md](docs/COPYRIGHT-SAFETY.md) for details.

## License

MIT
