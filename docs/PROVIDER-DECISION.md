# Provider Decision — cShot

## Updated Direction (May 2026)

**cShot is now local-first. There is no cloud dependency.**

The "mock-dsp" provider has been renamed to **cShot Engine** and is always
the default. Cloud providers (ElevenLabs, Stable Audio, AudioLDM 2) are
registered but never auto-selected and never required.

## Current State

cShot uses `cshot-engine` as the default and only active provider — a local
DSP synthesis engine (sine waves, noise, envelopes, filters) with no AI
model dependency and no API key required.

Placeholder providers exist for ElevenLabs, Stable Audio, and AudioLDM 2
but are hidden from the main generation flow. They appear only in the
Provider Selector for users who explicitly configure them.

## Evaluation

| Factor | cShot Engine (Local) | Cloud Provider |
|--------|----------------------|----------------|
| Quality | Improving (synthesis + analysis-driven) | Higher but inconsistent |
| Latency | ~500ms | 2-10s (network) |
| Cost | $0 | Per-generation API cost |
| Offline | Yes | No |
| Setup | None | API key, env config |
| Reliability | Deterministic, always available | Variable (network, rate limits) |
| Copyright | Safe (algorithmic) | Provider-dependent |
| Complexity | Zero | API integration, error handling |

## Principle

No paid-per-generation dependency. Local engine must always work.
No API key required for core functionality.
Do not call cloud models by default.
