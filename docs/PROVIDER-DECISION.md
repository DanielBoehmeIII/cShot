# Provider Decision — cShot

## Current State

cShot uses `mock-dsp` as the default provider — algorithmic synthesis
(sine waves, noise, envelopes, filters) with no AI model dependency.
Placeholder providers exist for ElevenLabs, Stable Audio, and AudioLDM 2
but require API keys and are not tested.

## Evaluation

| Factor | Mock DSP | Real Provider |
|--------|----------|---------------|
| Quality | Basic (functional one-shots) | Higher (realistic, varied) |
| Latency | ~500ms | 2-10s (network) |
| Cost | $0 | Per-generation API cost |
| Offline | Yes | No (requires network) |
| Setup | None | API key, env config |
| Reliability | Deterministic | Variable (network, rate limits) |
| Copyright | Safe (algorithmic) | Provider-dependent |
| Complexity | Zero | API integration, error handling |

## Recommendation

**Do not integrate a real provider yet.**

Rationale:
1. Mock DSP quality is sufficient for alpha testing the core loop
2. Real provider integration adds cost, latency, and complexity without
   validating the product thesis
3. The product thesis is: "Can a producer get a usable sound from a text
   prompt faster than browsing samples?" — this is testable with mock DSP
4. Adding a real provider now would conflate "AI quality" with "product
   value" — we need to validate the product first
5. Mock DSP lets us iterate fast (no API costs, no network issues)

## When to Revisit

- After alpha testing confirms the core loop is valuable
- When user feedback specifically asks for higher sound quality
- When we need to demonstrate "real AI" for funding/investor purposes

## Provider Shortlist (Future)

1. **ElevenLabs SFX** — Best quality for drum sounds, paid API
2. **Stable Audio Open** — Open-source, can self-host, good for one-shots
3. **AudioCraft (Meta)** — Free, open-source, needs local inference setup

## Fallback Plan

Mock DSP remains the default fallback even when real providers are added.
The `generate_with_fallback` chain in the registry already supports this.
