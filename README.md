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

- **Generate** one-shot sounds from text prompts
- **Variants** — 8 DSP transforms per sound
- **Reference upload** — generate sounds that match a reference WAV
- **Library** — search, filter, find similar by metadata
- **Packs** — organize sounds into packs, export ZIP with metadata
- **Batch generation** — recipe-based multi-sound generation
- **Prompt editing** — edit existing sounds via text ("make it darker")
- **Provider bakeoff** — compare generation providers side-by-side
- **Export** — DAW-friendly filenames, open export folder

## Architecture

```
Frontend:  React 18 + TypeScript + Vite + Tailwind
Backend:   Rust (Tauri v2) + SQLite + DSP engine
Models:    ElevenLabs / Stable Audio / AudioLDM 2 / Mock DSP
Storage:   Content-addressed WAV files + SQLite metadata
```

## Project Status

Alpha — working prototype with local DSP generation. Real provider integration requires API keys.

## Build

```bash
npm run tauri build
```

## Commercial Use

Generated sounds from cShot's default DSP pipeline are safe for commercial
use. If you configure a third-party AI provider, review their terms regarding
commercial use of generated content.

See [docs/COPYRIGHT-SAFETY.md](docs/COPYRIGHT-SAFETY.md) for details.

## License

MIT
