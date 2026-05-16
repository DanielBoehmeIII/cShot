# cShot — Sound Laboratory

A desktop app for generating unique, production-ready one-shot audio samples from natural language prompts.

Built with Tauri v2 + React + Rust.

## Quick Start

```bash
npm install
npm run tauri dev
```

For real AI generation, copy `.env.example` to `.env` and add an ElevenLabs API key.

## Features

- **Generate** one-shot sounds from text prompts — kicks, snares, hats, claps, percussion, bass, FX, and more
- **Variants** — 4-12 DSP transforms per sound (pitch shift, saturation, transient shape, bright/dark/sub variants)
- **Reference upload** — generate sounds conditioned on a reference WAV
- **Library** — browse, search, filter, favorite, delete all your sounds
- **Sound score** — every sound gets an automated quality score (0-100)
- **Export** — one-click WAV export to Desktop with semantic filenames
- **Provider selection** — hot-swap between Mock DSP (free, local) and ElevenLabs (API key required)
- **Keyboard shortcuts** — ⌘1 Generator, ⌘2 Library, Space play, R regenerate, ⌘E export, ? shortcuts help
- **Import** — import WAV/MP3 files with auto-analysis and tagging
- **Repair chain** — normalize, trim silence, fade, brighten, darken, punch
- **Recipe presets** — 18 built-in professional sound recipes (Trap Kick, Drill 808, House Kick, etc.)
- **Privacy** — local-first, no accounts, clear session memory

## Beta Status

cShot is in **Beta** — the core workflow is stable and usable for real production. The focus is on:
- Generation quality (improved DSP synthesis)
- Producer workflow speed (keyboard shortcuts, library, search)
- Reliability provider chain (Mock DSP always works, ElevenLabs for real AI)

### What's NOT in Beta

- VST3/AU plugin (standalone only)
- Marketplace/social features
- Cloud sync
- Stereo support
- Semantic search / embeddings

## Architecture

```
Frontend:  React 18 + TypeScript + Vite + Tailwind
Backend:   Rust (Tauri v2) + SQLite + DSP engine
Models:    ElevenLabs / Mock DSP
Storage:   Content-addressed WAV files + SQLite metadata
```

## Build

```bash
npm run tauri build
```

## Configuration

Copy `.env.example` to `.env` and optionally add your ElevenLabs API key:
```
CSHOT_ELEVENLABS_API_KEY=your_key_here
```

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

## Project Structure

```
src/              — React frontend
├── App.tsx       — Main app (generator + library views)
├── components/   — UI components (PromptBar, SoundCard, VariantCard, etc.)
├── hooks/        — React hooks (useAudioPlayback, useToast)
└── lib/          — API bindings, session memory, utilities

src-tauri/        — Rust backend (Tauri)
└── src/
    ├── commands.rs    — IPC handlers (generation, export, library, etc.)
    ├── audio/         — DSP, synthesis, analysis, I/O, validation
    ├── generation/    — Provider abstraction, registry, fallback chain
    ├── db/            — SQLite operations
    ├── generator.rs   — Sound generation orchestration
    ├── quality.rs     — Sound quality computation
    ├── score.rs       — Sound scoring
    └── storage/       — File I/O paths
```

## Commercial Use

Generated sounds from cShot's default DSP pipeline are safe for commercial
use. If you configure a third-party AI provider, review their terms regarding
commercial use of generated content.

See [docs/COPYRIGHT-SAFETY.md](docs/COPYRIGHT-SAFETY.md) for details.

## License

MIT
