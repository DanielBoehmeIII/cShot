# cShot 90-Day Roadmap (Updated — Local-First Direction)

## Current Status (Week 0)

- **Frontend**: React 18 + TS + Vite + Tailwind. Generate + Library views.
- **Backend**: Tauri v2 + Rust. Provider registry (local-first), DSP engine, SQLite, semantic library.
- **Generation**: cShot Engine (local synthesis, always available, no API key).
- **Storage**: Local WAV files + SQLite. 50+ tests passing.
- **Direction**: Local-first sound design laboratory. No cloud generation dependency.

## Biggest Risks

1. **Quality ceiling** — cShot Engine (DSP synthesis) sounds are functional but not yet production-grade.
2. **No users** — No alpha testers signed up yet. Risk of building in a vacuum.
3. **Commands.rs is monolithic** — Large file needs modularization.
4. **No audio analysis layer** — Can't "read" a sound to recreate it yet.

## Strongest Wedge

**Speed of generation** — 5 seconds from prompt to playable sound. Local-only means zero latency, zero API cost, zero setup.

## What to Cut

- Cloud generation APIs as primary workflow (local engine is default)
- Marketplace (too early)
- Mobile app (wrong platform)
- Social / community features
- Multi-language support
- Enterprise auth / teams

## What to Improve

1. **Local engine quality** — Multi-layer synthesis, analysis-driven generation
2. **Audio analysis** — Extract envelope, transient, spectral data from any sound
3. **UI polish** — Loading states, error recovery, keyboard shortcuts
4. **Architecture** — Split commands.rs, plugin-ready DSP core

## Technical Roadmap

**Weeks 1-4: Remove Cloud Dependency**
- Rename mock-dsp → cShot Engine
- Fix blocking provider errors when no API keys
- Make local generation always default
- Update README, docs, config, and UI

**Weeks 5-8: Local Audio Analysis**
- Audio feature extraction (envelope, transient, spectral)
- Sound classification heuristic
- Analysis cache and persistence
- Audio inspector UI

**Weeks 9-12: One-Shot Resynthesis**
- Layer-based synthesis engine (noise, tonal, transient, body, tail)
- Envelope-following resynthesis
- Category-specific generators (kick, snare, hat, clap, perc, bass, impact)

## UX Roadmap

- Loading spinners with ETA
- Error toasts with retry actions
- Keyboard shortcut reference
- Empty states with helpful guidance
- Sound comparison (A/B between variants)

## Engine Roadmap

- Week 1-4: Remove cloud dependency, rename engine, fix startup errors
- Week 5-8: Audio analysis pipeline (RMS, peak, spectral, envelope, transient)
- Week 9-12: Multi-layer synthesis per category
- Week 13-16: Prompt-to-DSP control (natural language → synthesis parameters)
- Week 17-20: Better local generation (multi-layer, genre presets, seed control)
- Week 21-24: Recreate sound from reference (analyze → resynthesize)
- Week 25-28: Local sample transformation (edit transients, body, tail)
- Week 29-32: Plugin-ready architecture (separate DSP core from app)
- Week 33-36: VST/CLAP prototype
- Week 37-40: Beta finish (stability, sound quality, export)

## Alpha Testing Roadmap

- Week 1-2: Alpha test script + feedback form
- Week 3-4: Recruit 5-10 testers
- Week 5-6: Run test sessions
- Week 7-8: Collect + analyze feedback
- Week 9-10: Ship fixes from top issues
- Week 11-12: Public demo page
