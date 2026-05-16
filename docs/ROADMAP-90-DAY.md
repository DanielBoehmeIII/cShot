# cShot 90-Day Roadmap

## Current Status (Week 0)

- **Frontend**: React 18 + TS + Vite + Tailwind. 5 views: Generate, Bakeoff, Library, Packs, Batch.
- **Backend**: Tauri v2 + Rust. Provider registry, DSP pipeline, SQLite, semantic library, embeddings.
- **Generation**: Mock DSP (always available), ElevenLabs (real API, needs key), Stable Audio (stub), AudioLDM 2 (stub).
- **Storage**: Local WAV files + SQLite. ~50 tests passing.

## Biggest Risks

1. **Quality ceiling** — Mock DSP sounds are basic. Real providers need paid API keys.
2. **No users** — No alpha testers signed up yet. Risk of building in a vacuum.
3. **Commands.rs is monolithic** — 1081 lines will become unmaintainable.
4. **No error recovery** in batch gen or multi-step workflows.

## Strongest Wedge

**Speed of generation** — 5 seconds from prompt to playable sound. This is the core value prop. Everything serves this loop.

## What to Cut

- Marketplace (too early)
- Mobile app (wrong platform)
- Plugin development (Phase 3)
- Social / community features
- Multi-language support
- Enterprise auth / teams

## What to Improve

1. **Provider quality** — Fix Stable Audio API, add auto-retry with backoff
2. **UI polish** — Loading states, error recovery, keyboard shortcuts
3. **Alpha readiness** — Auto-update, crash reporting, feedback collection
4. **Architecture** — Split commands.rs, extract view routing

## Technical Roadmap

**Weeks 1-4: Reliability**
- Stable Audio API implementation
- Generation timeout + retry UI
- Error state recovery in all views
- Split commands.rs into modules

**Weeks 5-8: Alpha Launch**
- Auto-update (Tauri updater)
- In-app feedback trigger
- Crash reporting
- Onboarding flow (suggested prompts, tutorial)

**Weeks 9-12: Quality**
- SoundScore display in generation UI
- Quality badges (clipping, duration, loudness)
- Reference analysis improvements
- Prompt suggestion engine

## UX Roadmap

- Loading spinners with ETA
- Error toasts with retry actions
- Keyboard shortcut reference
- Empty states with helpful guidance
- Sound comparison (A/B between variants)

## Model Roadmap

- Week 1-2: Fix Stable Audio API provider
- Week 3-4: Add generation timeout + retry
- Week 5-6: Provider health checks
- Week 7-8: Model quality A/B comparison
- Week 9-10: Custom prompt templates

## Alpha Testing Roadmap

- Week 1-2: Alpha test script + feedback form
- Week 3-4: Recruit 5-10 testers
- Week 5-6: Run test sessions
- Week 7-8: Collect + analyze feedback
- Week 9-10: Ship fixes from top issues
- Week 11-12: Public demo page

## Weekly Plan

| Week | Focus |
|------|-------|
| 1 | Stable Audio API implementation |
| 2 | Generation timeout + retry UI |
| 3 | Split commands.rs into modules |
| 4 | Error state recovery in all views |
| 5 | Tauri auto-update + in-app feedback |
| 6 | Alpha tester recruitment |
| 7 | Onboarding flow + tutorial |
| 8 | Alpha testing sessions |
| 9 | SoundScore display + quality badges |
| 10 | Prompt suggestion engine |
| 11 | Fix top alpha issues |
| 12 | Public demo page + publish |
