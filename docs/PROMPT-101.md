# Prompt 101 — v1 Build Plan

## Build Overview

**Timeline:** 12 weeks to MVP launch, 24 weeks to v1 stable
**Team:** Founder/CEO (full-stack) + Rust engineer + Frontend engineer + ML engineer
**Paradigm:** Ship early, validate, iterate. Every sprint must produce a demoable artifact.

---

## Phase 1: Foundation (Weeks 1-4)

### Milestone: "Type Something, Hear Something"

The user can type a prompt, get back audio, and play it. That's it. Everything else is polish.

### Sprint 1: Project Skeleton & Audio Pipeline

**Engineering tasks:**
- Initialize Tauri v2 project with React + TypeScript + Vite + Tailwind
- Set up Rust backend with Tauri IPC scaffolding
- Implement WAV read/write with `hound` crate
- Build audio DSP pipeline: trim silence, normalize peak, fade in/out
- Implement content-addressed storage (SHA-256 hash, file write/read)
- Set up SQLite via `rusqlite`, create `sounds` table schema
- Write DSP unit tests with golden WAV files

**Design tasks:**
- Finalize visual design system (colors, typography, spacing, dark theme)
- Design main generation screen layout (prompt bar + sound grid)
- Design waveform thumbnail component
- Create design tokens file

**Research tasks:**
- None (infrastructure sprint)

**Testing tasks:**
- DSP pipeline tests: each function against known inputs/outputs
- Storage tests: write, read, verify hash, dedup
- Database tests: schema creation, CRUD

**Demo at end of sprint:**
- Command-line tool that takes a WAV file, processes it through DSP chain, outputs processed WAV
- Tauri app skeleton that loads and displays "Hello from cShot"

**What ships:** Tauri shell, DSP library, storage module, database schema
**What gets tested:** DSP functions, storage write/read, database CRUD
**What can be cut:** Fancy UI, reference upload, multi-format export
**What proves progress:** Running `cargo test` passes with DSP golden tests

---

### Sprint 2: Model Gateway & First Generation

**Engineering tasks:**
- Implement `AudioGenerator` trait and model abstraction layer
- Build ElevenLabs SFX API client (HTTP, JSON, response parsing)
- Build text prompt encoder (CLAP-style ONNX model, local inference)
- Wire generation pipeline: prompt → encode → model gateway → DSP → storage → UI
- Implement basic "generate" Tauri command
- Build error handling for model failures, timeouts, network errors
- Add generation logging to SQLite

**Design tasks:**
- Design generation loading state (waveforms filling in progressively)
- Design error states (model offline, generation failed, timeout)
- Design empty state (first launch, no generations yet)
- Design prompt input with placeholder text

**Research tasks:**
- Fine-tune CLAP text encoder on producer vocabulary dataset (30 prompts × 5 sound types)
- Evaluate ElevenLabs vs Stable Audio vs AudioLDM for one-shot quality
- Establish baseline SoundScore model (heuristic version, before ML version)

**Testing tasks:**
- ElevenLabs API client: mock HTTP server (`wiremock`), test request/response/error
- Generation pipeline integration test: end-to-end with mock gateway
- Error handling: test all failure modes

**Demo at end of sprint:**
- Type "punchy kick" → 4-8 seconds → hear a kick play in the app
- First generation works. The core loop is alive.

**What ships:** Model gateway, ElevenLabs client, generation pipeline, first working end-to-end generation
**What gets tested:** API client with mocks, generation pipeline integration
**What can be cut:** Multi-model support (ElevenLabs only), Stable Audio integration
**What proves progress:** Typing a prompt and hearing a generated sound in the app

---

### Sprint 3: Sound Grid & Playback

**Engineering tasks:**
- Build SoundGrid component (2×3 responsive grid of SoundSlots)
- Build SoundSlot component: waveform thumbnail, SoundScore badge, label
- Implement waveform thumbnail generation (SVG path from audio samples)
- Build audio playback via Web Audio API (decode audio data, play, stop)
- Wire slot click → play sound
- Implement prompt bar with submit-on-Enter
- Build generation state management (Zustand store: prompt, results, loading, selection)
- Implement progressive slot filling (show each slot as it completes)

**Design tasks:**
- Design SoundSlot: waveform, score badge (color-coded), sound type label
- Design waveform animation during playback (cursor sweeps across waveform)
- Design generation progress indicator
- Design hover/active/selected states for slots
- Design SoundScore badge colors: red (<60), yellow (60-80), green (>80)

**Testing tasks:**
- Component rendering tests for SoundGrid, SoundSlot, PromptBar
- Audio playback tests (mock Web Audio API)
- State management tests (generation flow states)

**Demo at end of sprint:**
- Full generation UI: type prompt → 6 slots fill → click to play each sound
- The core product experience exists for the first time

**What ships:** SoundGrid, SoundSlot, waveform thumbnails, audio playback, prompt bar
**What gets tested:** Component rendering, state management, playback
**What can be cut:** Fancy animations, SoundScore colors
**What proves progress:** User can generate 6 sounds and play them all from the UI

---

### Sprint 4: Library Persistence & Browsing

**Engineering tasks:**
- Build LibraryView component (grid of sound cards with metadata)
- Implement library search: full-text search on prompts (SQLite FTS5)
- Implement library filtering: by model, date range, SoundScore range
- Build sound detail panel: waveform, metadata, SoundScore display
- Implement delete sound (with confirmation)
- Implement sound tagging (add/remove tags, free-form)
- Wire library state management (Zustand)
- Add library count and storage usage to status bar

**Design tasks:**
- Design LibraryView layout (search bar, filter chips, grid)
- Design Detail panel (side panel or overlay)
- Design tag editing UI (add tag input, tag pill components)
- Design empty library state

**Testing tasks:**
- Library search tests (FTS5 query correctness)
- Library filter tests
- Delete with confirmation tests
- Detail panel rendering tests

**Demo at end of sprint:**
- Generate sounds → they persist across app restarts
- Open Library → see all past generations → search by prompt → view details
- Tag a sound → filter by tag → find it again

**What ships:** Library persistence, search, filtering, detail panel, tagging
**What gets tested:** Search, filter, CRUD operations
**What can be cut:** FAISS similarity search, pack system, advanced sorting
**What proves progress:** Generating sounds, closing the app, reopening, and finding them in the library

---

## Phase 2: Power User Features (Weeks 5-8)

### Milestone: "Reference, Export, Pack"

The user can upload reference audio, control the sound, organize into packs, and export to their DAW.

### Sprint 5: Reference Upload & Variation

**Engineering tasks:**
- Build ReferenceDropZone component (drag-and-drop + file picker)
- Implement reference audio decoding via `symphonia` (WAV, MP3, FLAC, AAC)
- Build reference analysis: waveform, spectral profile, key metrics
- Implement reference-based generation: pass reference audio to model with text modification
- Build reference comparison view (original vs. generated side-by-side)
- Implement "regenerate with different seed" on individual slots
- Update generation pipeline to accept reference audio

**Design tasks:**
- Design ReferenceDropZone: dashed border, "Drop WAV here" text, file type indicators
- Design reference analysis display (waveform + spectral info)
- Design comparison view: side-by-side waveforms
- Design "regenerate" button on each sound slot

**Testing tasks:**
- Drag-and-drop event handling tests
- Audio decoding tests (multiple formats)
- Reference-based generation integration tests
- Regenerate flow tests

**Demo at end of sprint:**
- Drop a reference WAV → see its analysis → type "snappier attack" → 6 new variations → compare them

**What ships:** Reference upload and analysis, reference-based generation, regeneration
**What gets tested:** Reference pipeline, multi-format decoding, regenerate
**What can be cut:** Spectral comparison visualization, auto-key detection
**What proves progress:** User uploads a reference, gets 6 variations that are clearly related but different

---

### Sprint 6: Export System

**Engineering tasks:**
- Build ExportDialog component (format selection, bit depth, sample rate, options)
- Implement WAV export: 16-bit, 24-bit, 32-bit float via `hound`
- Implement AIFF export: 16-bit, 24-bit via `hound`
- Implement FLAC export: via `claxon` or system FLAC
- Implement MP3 export: via `lame` or `minimp3-rs`
- Build semantic filename generation: `cShot_{type}_{key}_{descriptor}.wav`
- Implement export progress indicator
- Build export history log
- Add "Export to DAW folder" shortcut

**Design tasks:**
- Design ExportDialog layout: format selection, options, preview, file path
- Design export progress animation
- Design export success state (checkmark, file path, "Open in Finder")
- Design recent exports list

**Testing tasks:**
- WAV header correctness tests (chunk sizes, format tags, bit depth)
- All format: open exported file in test, verify sample rate, bit depth, channel count, duration
- Filename sanitization tests
- Export edge cases: zero-length audio, very long filenames, special characters

**Demo at end of sprint:**
- Generate a kick → click Export → choose WAV 24-bit 44.1kHz → file appears on desktop → drop into Ableton → it plays perfectly

**What ships:** Multi-format export, export dialog, progress, history
**What gets tested:** Every export format for spec compliance
**What can be cut:** Batch export, export presets, drag-to-DAW
**What proves progress:** Export a sound, open it in a DAW, it works

---

### Sprint 7: Pack System

**Engineering tasks:**
- Build PackList component (sidebar or dropdown of packs)
- Build PackDetail component (sounds in pack, play, reorder)
- Implement `packs` and `pack_sounds` database tables
- Implement create/delete/rename pack
- Implement add/remove sound from pack
- Implement pack-level export (export all sounds in a pack)
- Build "Add to Pack" button in sound slots and detail panel
- Build pack creation flow: name → auto-populate from current grid

**Design tasks:**
- Design PackList: collapsible sidebar, pack cards with sound count
- Design PackDetail: horizontal list of sounds, play, reorder
- Design "New Pack" dialog (name input)
- Design "Add to Pack" context menu

**Testing tasks:**
- Pack CRUD tests
- Pack export tests (export all in correct format)
- Edge case: add same sound to multiple packs
- Edge case: delete pack with sounds (cascade behavior)

**Demo at end of sprint:**
- Generate kick → create "Night Run Kit" pack → add kick → generate snare → add to pack → generate hi-hat → add to pack → "Export All" → 3 WAV files in one folder

**What ships:** Pack CRUD, pack library, pack export
**What gets tested:** Pack operations, cascade behavior
**What can be cut:** Pack reordering, pack descriptions, pack art
**What proves progress:** User can assemble a full drum kit as a pack and export it all at once

---

### Sprint 8: SoundScore & Quality Feedback

**Engineering tasks:**
- Train initial SoundScore ONNX model (heuristic features → CNN regression)
- Integrate SoundScore inference into DSP pipeline
- Display SoundScore in SoundSlot badge and DetailPanel
- Build SoundScore breakdown visualization (Punch, Body, Clarity, Uniqueness bars)
- Implement auto-regeneration on low SoundScore (<50): silently retry
- Build quality feedback: "This sound was auto-regenerated due to quality check"
- Log SoundScore distributions for model improvement

**Design tasks:**
- Design SoundScore badge: large number, color gradient (red→yellow→green)
- Design SoundScore breakdown: horizontal bars with labels
- Design auto-regeneration notification (subtle toast)
- Design score animation on generation (counter animates from 0 to final score)

**Research tasks:**
- Collect 1000+ human quality ratings via internal tool
- Train SoundScore v0.1 (heuristic + simple CNN)
- Validate against human ratings: target 0.8+ Spearman correlation
- Identify failure modes: what sounds get low scores and why

**Testing tasks:**
- SoundScore model inference tests
- Auto-regeneration trigger tests
- Score display rendering tests

**Demo at end of sprint:**
- Generate sounds → see SoundScore badges appear in real-time → one sound gets 92, another gets 47 (auto-regenerated) → user sees quality feedback that matches their perception

**What ships:** SoundScore display, auto-regeneration on low quality, quality feedback
**What gets tested:** Model inference, auto-regeneration logic
**What can be cut:** SoundScore sub-dimension display, quality history chart
**What proves progress:** SoundScore visibly correlates with human-perceived quality

---

## Phase 3: Desktop Quality (Weeks 9-12)

### Milestone: "Ship-Ready Desktop App"

The app is polished, stable, installable, and ready for beta users.

### Sprint 9: Settings & Configuration

**Engineering tasks:**
- Build SettingsView component
- Implement settings persistence (config.json + SQLite settings table)
- Settings: model selection (ElevenLabs/Stable Audio/Local), output path, audio device, theme
- Implement model selection and gateway configuration
- Implement custom output path with Finder/Explorer dialog
- Build keyboard shortcuts system
- Add first-run setup wizard

**Design tasks:**
- Design SettingsView: sections, toggles, dropdowns, file picker
- Design first-run setup: 3 screens, 15 seconds each, "Let's go" final CTA
- Design keyboard shortcuts reference overlay

**Testing tasks:**
- Settings persistence tests (write → restart → read)
- Settings migration tests (old schema → new schema)
- Keyboard shortcut collision tests

**Demo at end of sprint:**
- Open Settings → change model to Stable Audio → generate → hear difference
- Change output path → export → file appears in new location
- Press `Cmd+Enter` to generate → `Cmd+E` to export

**What ships:** Settings, model selection, keyboard shortcuts, first-run wizard
**What gets tested:** Settings persistence, model switching
**What can be cut:** Audio device selection (use system default), multiple themes
**What proves progress:** Users can customize and control the app their way

---

### Sprint 10: Polish & Performance

**Engineering tasks:**
- Add smooth animations: slot filling, waveform drawing, transitions
- Performance: lazy-load library, virtualized grid if >100 items
- Performance: preload waveform thumbnails, cache to disk
- Performance: generation pipeline parallelization (generate slots concurrently)
- Reduce cold start time (<2 seconds)
- Reduce generation latency: pre-warm model connections, connection pooling
- Memory management: release audio buffers when not playing
- Add loading skeletons instead of spinners

**Design tasks:**
- Design loading skeleton for SoundGrid
- Design micro-animations: slot appears with scale, waveform draws left-to-right
- Design transition between main view and detail panel
- Design responsive behavior for window resize

**Testing tasks:**
- Performance benchmarks: generation time, start time, export time
- Memory usage tests: generate 50 sounds, check memory
- UI responsiveness tests at various window sizes

**Demo at end of sprint:**
- The app feels fast. Generation completes in <5 seconds. UI is responsive. Animations are smooth. Cold start is instant.

**What ships:** Animations, performance optimizations, caching, parallel generation
**What gets tested:** Performance benchmarks, memory usage
**What can be cut:** Virtualized grid (not needed until >500 sounds), GPU acceleration
**What proves progress:** Side-by-side comparison with Sprint 1: generation is 2x faster, UI is fluid

---

### Sprint 11: Error Handling & Edge Cases

**Engineering tasks:**
- Comprehensive error handling: network errors, model timeouts, disk full, corrupt audio
- Build error notification system (toast, inline error, error screen)
- Implement graceful degradation: if cloud model fails, fall back to local
- Handle edge cases: zero-length generation, all-silence generation, clipping
- Handle app termination during generation (save state, resume)
- Handle corrupt database (auto-backup, repair mode)
- Handle missing models (download prompt on first use)
- Add telemetry (opt-in): generation stats, error rates, feature usage

**Testing tasks:**
- Error state tests: each failure mode with expected user-facing message
- Edge case: generate 100 sounds in succession (no memory leak)
- Edge case: fill disk during export (graceful error)
- Edge case: network dropped during generation (fallback + notification)
- Edge case: app force-quit during generation (state recovery)

**Demo at end of sprint:**
- Unplug network → generate → app falls back to local model seamlessly → toast notification: "Cloud model unavailable, using local model"
- Try to export with full disk → clear error message with instructions

**What ships:** Error handling, graceful degradation, edge cases, telemetry
**What gets tested:** Every error state, recovery paths, edge cases
**What can be cut:** Repair mode UI (show repair in Terminal instead), auto-backup (manual backup instruction)
**What proves progress:** App handles every failure mode without crashing or confusing the user

---

### Sprint 12: Beta Release & Distribution

**Engineering tasks:**
- Configure macOS DMG build (notarized, signed)
- Configure Windows MSI build (signed, installer)
- Build auto-updater (Tauri updater)
- Set up Sentry error tracking
- Set up analytics (opt-in, privacy-first)
- Write install instructions
- Create beta user onboarding flow
- Build in-app feedback widget

**Design tasks:**
- Design beta badge and "Beta" indicators
- Design in-app feedback form (star rating + text)
- Design changelog/release notes view
- Design "Share Feedback" button in status bar

**Testing tasks:**
- Full install test: download → install → launch → generate → export on macOS and Windows
- Uninstall test: clean removal of all files
- Auto-update test: old version → update → new version with preserved data
- Beta user flow: invite → onboard → first generation

**Launch tasks:**
- Create beta landing page (cshot.ai/beta)
- Prepare beta invite email
- Set up beta Discord server
- Prepare beta FAQ document
- Define beta feedback collection process

**Demo at end of sprint:**
- Download cShot.dmg → drag to Applications → launch → type prompt → generate → export → done.
- The full product experience, installed and running, in under 30 seconds.

**What ships:** Beta release (macOS + Windows), auto-updater, error tracking, analytics
**What gets tested:** Full install/uninstall on both platforms, auto-update
**What can be cut:** Linux build, code signing on Windows (can distribute unsigned for beta)
**What proves progress:** A new user can download, install, and successfully generate their first sound in <2 minutes

---

## Phase 4: Iteration (Weeks 13-16)

### Milestone: "Beta Feedback → v1 Stable"

Collect beta feedback, fix issues, ship improvements, prepare for public launch.

### Sprint 13-14: Beta Feedback Loop

- Collect and triage all beta feedback
- Fix top 10 bugs by frequency
- Address top 5 feature requests
- Improve SoundScore model with beta data
- Monitor generation latency and error rates
- Performance pass: address any reported slowness

### Sprint 15-16: v1 Hardening

- Finalize all UI: no rough edges, consistent spacing
- Add what's missing: tooltips, empty states, loading states, error states
- Performance tuning: profile and optimize hot paths
- Security audit: data handling, API keys, file permissions
- Documentation: user guide, FAQ, troubleshooting
- Prepare marketing assets: screenshots, demo video, website updates

**v1 launch:** Public release. cShot is ready for producers.

---

## Complete Sprint Overview

```
Week │ Sprint │ Ship                                      │ Test                            │ Cut                    │ Progress Proof
─────┼────────┼───────────────────────────────────────────┼─────────────────────────────────┼────────────────────────┼─────────────────────────
  1  │ S1     │ Tauri shell, DSP lib, storage, DB schema  │ DSP golden tests, DB CRUD       │ Fancy UI, packs        │ cargo test passes
  2  │ S2     │ Model gateway, ElevenLabs, gen pipeline   │ API mocks, integration test     │ Multi-model, local     │ Type prompt → hear sound
  3  │ S3     │ SoundGrid, slots, playback, prompt bar    │ Component rendering, state mgmt │ Animations             │ Generate 6, play all
  4  │ S4     │ Library persistence, search, detail panel │ Search, filter, CRUD            │ FAISS search, packs    │ Close app → reopen → find
  5  │ S5     │ Reference upload, analysis, variation     │ Drag-and-drop, decode           │ Spectral comparison    │ Drop ref → get variations
  6  │ S6     │ Multi-format export, progress, history    │ WAV/AIFF/FLAC/MP3 correctness   │ Batch export           │ Export → open in DAW
  7  │ S7     │ Pack system, pack export                 │ Pack CRUD, cascade              │ Pack art, reorder      │ Build kit → export all
  8  │ S8     │ SoundScore, auto-regeneration             │ Model inference, trigger        │ Sub-dimensions, charts │ Score matches perception
  9  │ S9     │ Settings, keyboard shortcuts, onboarding  │ Persistence, migration          │ Multi-theme, audio dev │ Configure → generate
 10  │ S10    │ Animations, performance, caching          │ Benchmarks, memory              │ Virtualized grid       │ 2x faster than S1
 11  │ S11    │ Error handling, edge cases, telemetry     │ Every error mode, recovery      │ Repair UI             │ No crash on any failure
 12  │ S12    │ Beta release, distribution, auto-update   │ Full install/uninstall          │ Linux, code signing    │ Download → use <2 min
 13-14│ S13-14│ Bug fixes, top feature requests           │ Regression tests                │ New features           │ Bug count trending down
 15-16│ S15-16│ v1 hardening, docs, marketing assets      │ Full regression, security audit │ Enterprise features    │ "Ship it"
```

## Team Allocation Per Sprint

```
Sprint │ Founder/CEO           │ Rust Engineer           │ Frontend Engineer     │ ML Engineer
───────┼───────────────────────┼─────────────────────────┼───────────────────────┼────────────────────
  S1   │ Project setup, DB    │ DSP pipeline, storage   │ React scaffold, UI kit│ (idle)
  S2   │ Architecture         │ Model gateway, API      │ Wire gen to UI        │ Prompt encoder, eval
  S3   │ Product review       │ Waveform gen, playback  │ SoundGrid, slots      │ Baseline SoundScore
  S4   │ Library UX design    │ SQLite FTS, library CRUD│ LibraryView, detail   │ (idle)
  S5   │ Reference workflow   │ Symphonia decode, ref   │ ReferenceDropZone     │ (idle)
  S6   │ Export design        │ Export codecs           │ ExportDialog          │ (idle)
  S7   │ Pack UX              │ Pack DB, export         │ Pack components       │ (idle)
  S8   │ Quality UX           │ SoundScore integration  │ Score visualization   │ Train SoundScore v0.1
  S9   │ Settings design      │ Settings persistence    │ SettingsView          │ (idle)
  S10  │ Performance review   │ Optimization, caching   │ Animations, polish    │ (idle)
  S11  │ Error UX             │ Error handling, telemetr│ Error UI, feedback    │ (idle)
  S12  │ Beta launch          │ Build, updater, Sentry  │ Onboarding, feedback  │ (idle)
```

## Risk Register

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| ElevenLabs API changes/deprecation | Low | High | Model abstraction makes swapping easy; fallback to Stable Audio |
| Local ONNX inference too slow | Medium | Medium | Cloud-first for v1; quantize models aggressively; target <15s local |
| Sound quality inadequate for one-shots | Medium | Very high | Alpha validated kicks/808s are good; iterate on weak categories (snares, hats) |
| Producer adoption slower than expected | Medium | Medium | Free tier reduces barrier; reference upload bridges old workflow |
| Hiring delays (Rust engineer) | High | Medium | Founder builds Rust backend; cut scope to match capacity |
| Copyright challenge on training data | Low | Very high | Train only on licensed/public domain data; watermark all outputs |
| Competitor ships similar feature | Low | Medium | Taste model + plugin are hard to replicate; speed advantage from specialization |
