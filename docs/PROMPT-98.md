# Prompt 98 — The cShot Technical Principles

## Guiding Principles for Every Engineering Decision

These principles are not aspirational — they are binding. Every PR, every architecture decision, every dependency choice must be evaluated against them. If a decision violates a principle, it must be justified in writing.

---

### Principle 1: Audio Quality Is Non-Negotiable

**Statement:** *cShot outputs must be indistinguishable from (or better than) professionally recorded samples.*

**Why this matters:** If the first kick a producer generates sounds bad, they never generate a second one. Audio quality is the table stakes for entering a producer's workflow.

**Engineering implications:**
- Generation post-processing always applies: trim silence, normalize peak to -1.0dB, fade in/out
- Never output audio with DC offset, clipping, or truncated transients
- Sample rate floor: 44.1kHz (anything lower is not production-grade)
- Bit depth minimum: 16-bit (24-bit default for export)
- Monitor generation quality with automated QC: detect distortion, noise floor, clipped samples
- If quality check fails, regenerate silently (don't show the user a bad sample)
- SoundScore below 50 should not reach the user interface

**Violation example:** Skipping quality checks to reduce latency is not allowed.

---

### Principle 2: Latency Is a UX Metric

**Statement:** *Every millisecond of latency is friction. Generation should feel like thought, not waiting.*

**Why this matters:** The core value prop is "stop browsing, start making." If "making" takes 15 seconds, the producer has already lost focus. Alpha testing showed P95 latency of 14.7s was a critical issue.

**Engineering implications:**
- Generation target: <5 seconds P50, <10 seconds P95 for cloud generation
- Local inference target: <15 seconds (acceptable for offline, but optimize aggressively)
- Waveform preview must appear instantly (not after generation completes)
- Export must complete in <1 second
- App cold start: <2 seconds
- SoundGrid population must be progressive (show partial results as they arrive)
- Use optimistic UI: show loading state immediately, fill content as available
- Measure and instrument every latency source: model inference, network, post-processing, rendering

**Violation example:** Adding a processing step that adds 2 seconds without measurable quality improvement.

---

### Principle 3: Controllability Over Automation

**Statement:** *The producer always decides. cShot suggests, never decides. Every output is inspectable and adjustable.*

**Why this matters:** cShot's thesis is that AI should accelerate taste, not replace it. A black-box generator that hides parameters from the user betrays this thesis. The producer must understand why they got a specific sound and how to change it.

**Engineering implications:**
- Every generation includes provenance: model, seed, prompt, timestamp, parameters
- No "auto-magic" modes that hide parameters without consent
- All sound analysis (SoundScore, metrics) is displayed, not hidden
- Prompt history must be accessible for replay and iteration
- Reference-based generation must be transparent about how the reference was used
- No automatic export, upload, or sharing of user-generated sounds
- User must explicitly confirm destructive actions (overwriting, deletion)

**Violation example:** An "auto-enhance" button that applies processing without showing what it did.

---

### Principle 4: User Privacy by Default

**Statement:** *No user data leaves the device without explicit consent. Local-first means local-first.*

**Why this matters:** Producers generate unique, personal sounds. These are creative assets, potentially commercially valuable. cShot must never be a vector for data leakage, training data harvesting, or surveillance.

**Engineering implications:**
- All audio processing runs locally (trim, normalize, analyze, encode)
- Model inference can use cloud APIs — but audio data is never stored on cloud servers
- Prompts are sent to cloud models for inference only — no logging of prompt-audio pairs without consent
- User consent required for: cloud sync, usage analytics, model improvement data collection
- Local mode must be fully functional (no features gated behind online requirement)
- Opt-in for: sharing generation data for model improvement, public packs, community features
- Export to DAW is local file system — no cloud intermediary
- Content-addressed storage ensures data integrity without external dependencies

**Violation example:** Sending usage analytics or prompt data to a third-party analytics service without explicit opt-in.

---

### Principle 5: Copyright Safety by Design

**Statement:** *Every cShot-generated sound is legally safe to use in commercial music. Provenance is traceable.*

**Why this matters:** Producers need to release music. Labels require clearance. Competitors who ignore copyright will be locked out of commercial use. Copyright safety is a moat.

**Engineering implications:**
- Training data must be: (a) licensed, (b) public domain, or (c) generated by cShot itself
- No training on copyrighted sample packs without explicit license
- Memorization detection: periodically sample generations and check nearest-neighbors in training set
- Audio watermarking (inaudible, robust) embedded in every generation for provenance
- Provenance record: model hash + seed + prompt + timestamp = verifiable origin
- If a generated sound is a near-copy of a training sample (>0.95 embedding similarity), it must be rejected
- Clear license display per generation: "Generated by cShot vX using Model Y — Commercial use allowed"
- Legal audit trail: every generation is logged with deterministically reproducible parameters

**Violation example:** Using a model trained on unlicensed Splice samples, even if the model is open-source.

---

### Principle 6: Local-First Architecture

**Statement:** *cShot works fully offline. The cloud enhances but never gates core functionality.*

**Why this matters:** Producers work in studios, on planes, in coffee shops without internet. A cloud-dependent tool fails in the exact moments of inspiration. Local-first also means no latency, no privacy risk, and no subscription lock-in for core generation.

**Engineering implications:**
- Core generation loop must function without internet (text → local ONNX model → DSP → preview → export)
- Cloud generation is the default (better quality), but local is always available as fallback
- Library is stored locally (SQLite + content-addressed files) — cloud sync is supplementary
- Settings and preferences are local files — no account required to use the app
- If cloud generation fails (network error, API down), automatically fall back to local (don't show error)
- Local-first sync model: local is authoritative; cloud is a replica
- App must launch and be usable without any network call
- All UI assets bundled in the app — no CDN-dependent assets

**Violation example:** Requiring a login or internet connection to generate a sound, even on first launch.

---

### Principle 7: Model Abstraction

**Statement:** *No model-specific code in the UI. Models are pluggable, swappable, and comparable.*

**Why this matters:** The model landscape is evolving rapidly. Today's best model (ElevenLabs SFX) will be surpassed. The architecture must allow swapping models without rewriting UI or backend logic.

**Engineering implications:**
- Gateway pattern: UI sends a `GenerationRequest { prompt, reference, params }`, receives a `GenerationResponse { audio, metadata }`
- No model-specific parameters exposed to the UI without abstraction
- All models conform to a `AudioGenerator` trait (Rust) or interface
- Models can be remote (HTTP API) or local (ONNX runtime) — same interface
- Generation response includes the model name, but business logic doesn't depend on it
- New models can be added by implementing the trait — no other code changes
- Model versioning: track which model version generated each sound for reproducibility

**Violation example:** Writing UI code that checks `if model === "elevenlabs"` to handle parameters differently.

---

### Principle 8: DSP Reliability

**Statement:** *All audio processing must be deterministic, glitch-free, and verifiable.*

**Why this matters:** Producers cannot tolerate audio glitches. A single truncated transient, click artifact, or DC offset erodes trust. DSP code must be the most tested, most reliable code in the system.

**Engineering implications:**
- All DSP functions must be pure (input → output, no side effects, no state)
- Deterministic: same input always produces identical output (down to the bit)
- Saturating arithmetic to prevent wraparound on integer audio data
- Floating-point audio must be bounded to [-1.0, 1.0] — no NaN, no Inf
- All DSP code is tested with known reference outputs (golden WAV files in test fixtures)
- Property-based testing: verify invariants (no clipping, no DC, no silence-only outputs)
- Audio processing errors are never silent — log + recover gracefully
- Input validation on all audio data entering the pipeline (sample rate, bit depth, channel count)

**Violation example:** An edge case in the trim function that truncates a transient by 2ms because of an off-by-one error.

---

### Principle 9: Metadata Integrity

**Statement:** *Every sound is fully described by its metadata. No orphan files. No missing provenance.*

**Why this matters:** As a producer's library grows, metadata becomes the primary navigation interface. If metadata is incomplete, inconsistent, or lost, the library becomes unusable. Metadata is also the foundation for personalization, search, and provenance.

**Engineering implications:**
- Every `sounds` table row must have non-null: id, hash, prompt, model, seed, created_at, sample_rate, bit_depth, duration
- Metadata is written before audio files (write-ahead: if metadata write fails, audio write is rolled back)
- Metadata is immutable after creation (edits create new rows with parent_id reference)
- Audio files without metadata entries are garbage-collected on startup
- Content-addressed storage means the hash is the link between metadata and file — must be verified on read
- Export includes a sidecar `.cshot-meta.json` file with full provenance metadata
- If metadata and audio file are inconsistent (hash mismatch), flag for repair — don't silently serve

**Violation example:** Saving audio to disk without immediately writing the corresponding metadata row.

---

### Principle 10: DAW Compatibility

**Statement:** *cShot outputs must work in every major DAW without additional processing.*

**Why this matters:** The last mile is critical. If a producer exports from cShot and the WAV doesn't import correctly into Ableton/FL Studio/Logic/Pro Tools/Reaper, the entire workflow is broken.

**Engineering implications:**
- Default export: WAV, 24-bit, 44.1kHz, mono — the most universally compatible format in music production
- WAV files must use standard headers (no proprietary chunks that might confuse DAWs)
- Output files must have proper sample-accurate length (no padding, no truncation)
- File naming: semantic, sanitized (alphanumeric + underscores), no special characters
- Export path is user-configurable but defaults to DAW-friendly location (e.g., `~/Music/cShot/`)
- SMPTE-compatible timing: generation duration is sample-accurate, specified in metadata
- Normalized peak at -1.0dB (headroom for DAW mixing — industry standard)
- No metadata-only files (like `.cshot-meta.json`) in the DAW import path unless requested

**Violation example:** Exporting a WAV with a non-standard sample rate (e.g., 22,050 Hz) that some DAWs handle poorly.

---

### Principle 11: Research Extensibility

**Statement:** *The architecture must support experiments, model swaps, and data collection for research without destabilizing the product.*

**Why this matters:** cShot is both a product and a research lab. Engineers and researchers must be able to iterate on models, metrics, and features without shipping unstable code to users. The architecture must support a fast research cycle alongside a stable product.

**Engineering implications:**
- Feature flags for all experimental features: roll out to beta users first, then GA
- Model gateway supports A/B testing: route a percentage of users to experimental model
- All generation parameters are logged for offline analysis (prompt, seed, model, latency, outcome)
- User interactions (generation, export, delete, rate, tag) are logged as anonymized events
- Research tracks have isolated code paths (e.g., `research/soundscore/v2/`) that don't affect production code
- ONNX model files are versioned and backward-compatible (old generations remain playable with new models)
- Dataset export: users can opt-in to share generation data for research (separate consent from product analytics)
- Publication code is open-sourced (models, metrics, datasets) — product code remains proprietary

**Violation example:** A research experiment that modifies the generation pipeline for all users without a feature flag or A/B test.

---

## Principle Conflict Resolution

When principles conflict, use this priority order:

1. **Audio quality** — never sacrifice sound quality for any other principle
2. **Copyright safety** — never risk legal safety for speed or convenience
3. **User privacy** — never violate privacy for data collection or model improvement
4. **Controllability** — never take agency away from the producer
5. **Latency** — speed matters, but not at the cost of 1-4
6. **DSP reliability** — deterministic, verified processing
7. **Metadata integrity** — data trustworthiness
8. **Local-first** — offline capability
9. **Model abstraction** — future-proof model integration
10. **DAW compatibility** — last-mile reliability
11. **Research extensibility** — enabling innovation

**Example conflict:** A researcher wants to collect detailed user interaction data to improve SoundScore. This conflicts with Principle 4 (privacy). Resolution: Privacy wins. Collect only anonymized, aggregated data on an opt-in basis, with clear disclosure.

**Example conflict:** A product manager wants to reduce latency by skipping quality checks. This conflicts with Principle 1 (audio quality). Resolution: Quality wins. Find latency improvements elsewhere (model optimization, parallel processing) instead.
