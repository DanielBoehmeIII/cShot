# cShot v2 Architecture Review

## What Is Clean
- Provider trait + registry with automatic fallback chain
- DSP pipeline (process → validate → save)
- SQLite schema with migrations
- Content-addressed local storage
- Semantic library with metadata similarity
- Embedding readiness layer (trait + mock + hybrid similarity)

## What Is Fragile
- All generation providers except mock are stubs for the `generate` method (ElevenLabs is fully implemented) — Stable Audio and AudioLDM need real implementations
- No timeout management in the generation pipeline (registry doesn't cancel slow providers)
- No user authentication/accounts (all local)
- No error recovery in batch generation (one failure stops the batch)

## What Should Be Refactored
- `commands.rs` is too large (1081 lines) — should be split into modules (generation, library, packs, feedback)
- `App.tsx` is 600+ lines — should extract views into route-based structure
- Config loading (`.env`) is done at startup but not reloadable at runtime
- No centralized error type — functions return `Result<_, String>` everywhere

## What Should Stay Simple
- Single-screen generation UI — don't add routing framework
- SQLite — don't migrate to Postgres yet
- Metadata similarity — don't add vector DB until library > 10K sounds
- Local file storage — don't add cloud sync yet

## What Blocks Real Model Integration
- Stable Audio provider returns "not implemented" (no API call)
- AudioLDM 2 requires local model deployment (not trivial)
- No model A/B testing infrastructure
- No usage quota/budget tracking for API costs

## What Blocks Alpha Testing
- No auto-update mechanism (testers must build from source)
- No crash reporting
- No in-app feedback collection (feedback modal exists but isn't triggered)
- No telemetry to understand usage patterns

## What Blocks Desktop/Plugin Future
- No VST3 plugin (Tauri app only — requires NIH-plug work)
- No DAW context awareness (BPM, key extraction)
- No drag-from-app-to-DAW (requires OS-level drag support)
- No MIDI integration
