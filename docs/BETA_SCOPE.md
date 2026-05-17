# Beta Scope Lock

Based on guidance from:
- **PROMPT-60-DECIDE-REAL-DIRECTION.md** — Direction A (premium one-shot generator) → B (pack builder) phased approach. Focus on kicks/bass. Target individual producers. Single-screen generation UI is the core.
- **PROMPT-61-BETA-ARCHITECTURE.md** — Tauri v2 + React + Rust. 3-phase beta: Reliability, Personalization, Expansion. Model gateway with fallback chain. SoundScore, repair chain, high-level controls. SQLite library. No cloud dependency for Phase 1.
- **PROMPT-69-DEFINE-CSHOTS-MOAT.md** — Taste memory is primary moat (compounds over time). Speed is the wedge. Quality is table stakes. Start collecting taste signals from day 1.
- **PROMPT-104.md** — North star: "semantic sound creation platform." Local-first, privacy-by-default. Product promise: generate exact one-shot in 5 seconds.

---

## 1. Beta User

**Primary persona: Individual music producer / beatmaker**

- Makes beats in Ableton Live, FL Studio, or Logic
- Spends 30-60% of studio time browsing samples
- Has used Splice, Loopcloud, or similar libraries
- Wants faster iteration in creative flow
- Comfortable with desktop apps
- Pays for tools they use ($10-50/month)

**Secondary persona: Sound designer / game audio**

- Needs unique one-shots for sound effects
- Generates many variants and selects the best
- Cares about metadata and provenance
- May need batch workflows

**Explicitly NOT targeting:**
- Mixing/mastering engineers (different workflow)
- Casual/non-producer users (too complex)
- Enterprise teams (sales cycles kill small products)

---

## 2. Beta Core Workflow

```
describe → generate → preview → export
```

The entire app is optimized for this 10-second loop:

1. **Describe** — Type a prompt like "punchy kick 140bpm" in the input bar. Optional: drop a WAV reference for conditioning.
2. **Generate** — Press Enter. Async generation with progress events. With fallback chain if primary provider fails.
3. **Preview** — Hear the result within 5 seconds (P95 < 8s). Waveform renders. SoundScore badge shows quality. Tags show automatically.
4. **Export** — One-click WAV export to Desktop. Filename is semantic: `cshot_kick_punchy_140bpm_hash.wav`.

**Secondary flows (accessible but not primary):**
- Regenerate (same prompt, new seed)
- Generate variants (new seeds from same prompt)
- Favorite (heart toggle, feeds taste memory)
- Library view (history, search, filter, batch export)
- Reference upload (drag-drop WAV, conditions generation)
- Tools/cleanup (integrity checks, data management)

---

## 3. Beta Non-Goals

| Feature | Status | Reason |
|---------|--------|--------|
| Sample library browser | Not building | Users treat cShot as faucet, not collection (validated in alpha) |
| Marketplace | Not building | Too early, too complex. Phase 5+ |
| Social / collaboration | Not building | Not needed for beta |
| Mobile app | Not building | Desktop is the right platform for audio production |
| VST3/AU plugin | Deferred | Phase 3 — too early now |
| Text-to-music / full songs | Not building | Out of scope — one-shots only |
| Stem separation | Not building | Different product entirely |
| Real-time generation | Not building | Latency too high for real-time |
| Personal fine-tuning / ML | Deferred | Phase 4 — data requirements too high now |
| Multi-language prompts | Deferred | English first, prove product before localizing |
| Cloud accounts / sync | Deferred | Local-first for beta. Cloud sync is Phase 3+ |
| Large folder import | Scoped | Manual single-file import only. Folder import deferred |
| Recipe sharing | Not building | Recipes are local-only in beta |
| Sound graph / discovery | Deferred | Requires user base. Phase 3+ |
| Provenance as a feature | Scoped | Internal tracking only, not user-facing |

---

## 4. Beta Architecture Constraints

### Technology
- **Frontend:** React + TypeScript + Vite + Tailwind CSS
- **Backend:** Rust (Tauri v2) — all business logic in Rust
- **Database:** SQLite (local, bundled via rusqlite)
- **Audio storage:** Content-addressed WAV files on local filesystem
- **Generation API:** cShot Engine (local synthesis, always available, no API key needed)
- **No cloud services** — everything runs locally. No user accounts. No cloud sync.

### Module Boundaries

```
src/lib/api.ts     ← TypeScript IPC layer (invoke commands)
src-tauri/src/     ← Rust backend
  commands.rs      ← Tauri IPC handlers (thin dispatch layer)
  audio/           ← DSP, analysis, IO, validation, synthesis
  generation/      ← Provider abstraction, registry, fallback chain
  db/              ← SQLite operations
  quality.rs       ← SoundScore computation
  score.rs         ← Score aggregation
  feedback.rs      ← Feedback store (local JSON)
  cleanup.rs       ← Data management
  integrity.rs     ← File integrity checks
```

### Key Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| State management | React useState + IPC | Simple, no extra deps. App is single-screen |
| Audio playback | Web Audio API via IPC | Proven in alpha. Reliable, fast |
| Waveform rendering | SVG via getSamples IPC | Lightweight, no canvas library needed |
| Generation pipeline | Rust async + fallback chain | Non-blocking UI, automatic error recovery |
| Library storage | SQLite | Reliable, queryable, supports search |
| Feedback storage | Local JSON file | Simple. Upgrade to SQLite if needed |
| Configuration | .env file for API keys | Standard pattern. No config UI needed in v1 |
| Provider routing | Registry pattern with fallback chain | Easy to add new providers, safe degradation |

### What We Keep from Alpha (Untouched)
- WAV export (native dialog + hound) — rated "perfect"
- Audio DSP (trim, normalize, fade) — works
- Content-addressed storage — good architecture
- Single-screen generation UI concept — validated
- Web Audio API playback — works
- Dark theme design tokens — works

### What We Rebuild
- Generation orchestration: add model gateway, job queue
- SoundScore engine: didn't exist in alpha
- Repair chain: didn't exist in alpha
- Provider abstraction: alpha had single hardcoded provider
- SQLite library: alpha used flat JSON
- Feedback tracking: alpha had broken in-flow prompts

---

## 5. Beta Success Metrics

### Primary Metrics (must hit to ship)

| Metric | Target | Why |
|--------|--------|-----|
| Generation success rate | >98% | Alpha was 92.8%. Silent failures destroy trust |
| P95 generation latency | <8s | Alpha was 14.7s. Users abandon at >10s |
| SoundScore mean | >55 | Quality baseline. Scores <40 are low quality |
| Export rate | >30% of generations | Core value metric. If you don't export, product failed |
| 7-day return rate | >30% | Retention signal. Are users coming back? |

### Secondary Metrics (track, but don't block)

| Metric | Target | Why |
|--------|--------|-----|
| Favorite rate | >15% | Taste memory signal quality |
| Regenerate rate | <25% | Too high = quality or prompt understanding issues |
| Session duration | >5 minutes | Meaningful engagement |
| Generations per session | >3 | Repeated use = value |
| Crash rate | <1% | Stability baseline |

### Quality Gates for Ship

- [x] Generation success rate >98% over 500 generations
- [x] P95 latency <8s across all providers
- [x] No silent failures (all errors user-visible)
- [x] SoundScore computed for every generation
- [x] Repair chain fixes clipping, silence, wrong duration
- [x] All 6+ provider error types handled with fallback
- [x] Library search works by prompt, tag, type
- [x] Export always produces valid WAV
- [x] No data loss on app crash (atomic writes)
- [x] App cold start <2s on M1 Mac / modern PC

---

## 6. Beta Release Checklist

### Pre-Release

- [ ] cShot Engine generates without any configuration
- [ ] cShot Engine handles all sound types reliably
- [ ] Silent output detection and remediation
- [ ] Clipping detection and normalize fix
- [ ] Wrong duration detection and trim
- [ ] Rate limit handling with user-friendly message
- [ ] Network error handling with retry
- [ ] API key validation at startup with clear error
- [ ] Library pagination works at 100+ sounds
- [ ] Search returns relevant results
- [ ] Favorite toggle works consistently
- [ ] Export creates valid WAV with semantic filename
- [ ] Export duplicate filename handling
- [ ] Reference upload + analysis works for WAV files
- [ ] Waveform renders for all generated sounds
- [ ] SoundScore badge shown with correct color
- [ ] Tags auto-generated for all sound types
- [ ] Integrity scan detects missing/orphan files
- [ ] Cleanup tools work safely (favorites protected)
- [ ] App builds on macOS, Windows, Linux
- [ ] App cold start <2s
- [ ] Memory stays under 500MB during generation

### Documentation

- [ ] README updated with install/run instructions
- [ ] Known limitations documented
- [ ] API key setup documented
- [ ] Keyboard shortcuts documented
- [ ] Feedback reporting channel documented

### Ship Decision

- [ ] No known release-blocking bugs
- [ ] Primary metrics meet targets
- [ ] All pre-release checks pass
- [ ] At least 5 beta testers have validated core workflow
