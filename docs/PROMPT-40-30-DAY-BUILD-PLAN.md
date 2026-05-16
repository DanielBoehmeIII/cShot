# Prompt 40 — Design the First 30-Day Build Plan

A day-by-day blueprint for a solo developer to build a working cShot prototype in 30 days.

---

## Overview

### The Rules

```
1. One developer, 30 days
2. Work in public (GitHub commits daily)
3. Ship something usable every week
4. Cut scope ruthlessly — every day you work on the thing that 
   creates the most user-visible progress
5. No premature optimization. No building for scale. No "we'll need this later."
6. If something takes longer than a day, it gets simplified
```

### Weekly Milestones

```
Week 1 (Days 1-7):   "Hello, Sound" — Can generate one sound from one prompt
Week 2 (Days 8-14):  "Six Slots" — Full grid, preview, export
Week 3 (Days 15-21): "Don't Lose It" — Library, favorites, tags
Week 4 (Days 22-30): "Ship It" — Polish, packaging, beta release
```

### MVP Features

```
✓ Upload audio snippet (drag-drop reference file)
✓ Prompt desired one-shot (text input)
✓ Generate or transform sound (model inference)
✓ Preview multiple results (6-slot grid + playback)
✓ Tag/save favorites (basic library)
✓ Export WAV files (download to disk)
```

---

## Week 1: "Hello, Sound" — Days 1-7

```
Goal: Generate a single sound from a hardcoded prompt.
      Hear it. Save it to a WAV file.
      This is the most important week — prove the core works.
```

### Day 1 — Project Skeleton

```
Goal: Tauri + React app compiles and shows a window.

Exact tasks:
  - Install Tauri v2 CLI: cargo install tauri-cli
  - Create project: npm create tauri-app@latest
  - Set up React + TypeScript + Vite inside the Tauri shell
  - Create minimal Tailwind config
  - Build and run: cargo tauri dev
  - Verify: window opens, shows "cShot" in a styled H1

Files to create:
  - src-tauri/Cargo.toml
  - src-tauri/src/main.rs (minimal)
  - src-tauri/tauri.conf.json
  - src/App.tsx (basic shell)
  - src/main.tsx
  - package.json with React, Vite, Tailwind deps

Technical decisions:
  - Use Tauri v2 (latest stable)
  - Yarn or npm (no preference)
  - Tailwind for styling

Testing method:
  - cargo tauri dev — app window appears
  - "cShot" text is visible and styled

Success criteria: Blank app window with title "cShot"
```

### Day 2 — Rust Audio Backend

```
Goal: Rust code that can process audio buffers.
      Read a WAV, modify it, write it back. Proves the audio chain works.

Exact tasks:
  - Add hound crate to Cargo.toml
  - Create src-tauri/src/audio/mod.rs with:
    - load_wav(path) → Vec<f32>
    - save_wav(path, samples, sample_rate)
    - normalize(samples) → apply peak normalization
    - trim_silence(samples) → remove leading/trailing silence
  - Write a test that: loads a test WAV → trims → normalizes → saves → verifies
  - If no test WAV exists, generate a sine tone as test fixture

Files to create/modify:
  - src-tauri/Cargo.toml (add hound, uuid, sha2)
  - src-tauri/src/audio/mod.rs
  - src-tauri/src/audio/process.rs
  - src-tauri/tests/audio_test.rs

Technical decisions:
  - Pure Rust audio processing (no external DSP libs)
  - f32 samples internally, i16 for WAV I/O
  - All audio is mono, 44.1kHz for MVP

Testing method:
  - cargo test — all audio tests pass
  - Manually verify test output WAV plays correctly

Success criteria: Code can load, modify, and save WAV files
```

### Day 3 — Model Integration (Onnx Runtime)

```
Goal: Load an ONNX model and run inference.
      Even if the output is garbage, the pipeline works.

Exact tasks:
  - Add ort crate to Cargo.toml
  - Create src-tauri/src/model/mod.rs:
    - ModelLoader struct (downloads/loads model from local path)
    - fn load(path) → Session
    - Create src-tauri/src/model/inference.rs:
    - fn generate(session, prompt_embedding) → Vec<f32>
  - Download a test model (AudioLDM 2 distilled variant)
  - Run inference with a hardcoded embedding
  - Convert model output (spectrogram → audio via vocoder if needed)

Files to create/modify:
  - src-tauri/Cargo.toml (add ort)
  - src-tauri/src/model/mod.rs
  - src-tauri/src/model/inference.rs
  - src-tauri/src/model/loader.rs

Technical decisions:
  - ONNX Runtime with CUDA (GPU) + CPU fallback
  - Model stored in ~/cShot/models/ at runtime
  - Model downloaded separately (not bundled) to keep binary small
  - For MVP: model expects text embedding input, outputs raw audio

Testing method:
  - cargo test -- --nocapture — inference runs without crash
  - Inspect output: non-silent, reasonable length

Success criteria: Model loads, inference runs, audio comes out
```

### Day 4 — Inference Cache & IPC

```
Goal: Frontend can trigger inference via Tauri IPC.
      First actual user flow: click button → sound comes back.

Exact tasks:
  - Create Tauri command: generate_sound(prompt, config)
  - Implement prompt → embedding conversion (simple lookup table for MVP)
  - Wire model inference to command handler
  - Return sound_id and metadata to frontend
  - Create src-tauri/src/commands/generation.rs

Frontend:
  - Add invoke() call from @tauri-apps/api
  - Add "Generate" button that calls the command
  - Show response in console log

Files to create/modify:
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/commands/generation.rs
  - src-tauri/src/lib.rs (register command)
  - src/App.tsx (add generate button + console log)

Technical decisions:
  - IPC via Tauri's invoke() system
  - Return JSON with sound_id, duration, sample rate
  - Audio written to disk (content-addressed), path returned

Testing method:
  - Click "Generate" in UI
  - Check console: sound_id returned
  - Check filesystem: WAV file created

Success criteria: Frontend button triggers backend generation
```

### Day 5 — Post-Processing Pipeline

```
Goal: Generated audio is clean, trimmed, normalized, and playable.

Exact tasks:
  - Implement full post-processing chain:
    1. Remove DC offset
    2. Trim leading/trailing silence (> -60dB threshold)
    3. Apply fade in (2ms) and fade out (10ms)
    4. Peak normalize to -1dBFS
    5. Zero-pad to minimum 100ms
  - Handle edge cases: silent output, very short output, clipping
  - Wrap in GenerationPipeline struct
  - Test on 50 random outputs, verify all pass quality checks

Files to create/modify:
  - src-tauri/src/audio/pipeline.rs (new)
  - src-tauri/src/audio/mod.rs (add pipeline module)
  - src-tauri/tests/pipeline_test.rs

Technical decisions:
  - Chain-of-responsibility pattern (each stage is modular)
  - Stages can be skipped via config flags
  - All parameters configurable (defaults for MVP)

Testing method:
  - cargo test — pipeline tests pass
  - Generate 10 sounds, verify: no DC offset, proper fades, peak ≤ -0.5dBFS

Success criteria: Post-processing makes every output clean and playable
```

### Day 6 — Sound Type Classification

```
Goal: Auto-detect whether generated sound is kick, snare, hat, etc.

Exact tasks:
  - Implement lightweight classifier in Rust:
    - Extract features: spectral centroid, crest factor, onset strength,
      zero-crossing rate, energy bands (sub, low, mid, high)
    - Simple decision tree or k-NN classifier
    - Training: hardcoded thresholds derived from known samples
  - Create SoundType enum and classify() function
  - Integrate into generation pipeline (classify after post-process)
  - Return type in generation response

Files to create/modify:
  - src-tauri/src/audio/classify.rs (new)
  - src-tauri/src/audio/mod.rs (add classify module)
  - src-tauri/src/audio/analyze.rs (feature extraction helpers)
  - Modify pipeline to call classify

Technical decisions:
  - Rule-based classifier for MVP (no ML dependency)
  - Types: kick, snare, hihat_closed, hihat_open, clap, percussion, bass, fx
  - Thresholds calibrated from analyzing 200 labeled one-shots

Testing method:
  - Generate 20 sounds of each type, check classifier accuracy > 60%
  - Manually verify edge cases (ambiguous sounds)

Success criteria: Generated sounds get a correct type label >60% of the time
```

### Day 7 — Week 1 Integration

```
Goal: End-to-end: press button → generate → classify → save → play.

Exact tasks:
  - Wire all components: model → post-process → classify → store
  - Add basic frontend waveform display (canvas-based, 80 points)
  - Show sound type label on generated sound
  - Show duration + status in UI
  - Delete old prototype code, clean up

Demo scenario:
  1. Open app
  2. Click "Generate" (hardcoded prompt for now)
  3. Wait 2-5 seconds
  4. See waveform thumbnail
  5. See "Kick · 0.4s" label
  6. Click play → hear the sound

Files to create/modify:
  - Frontend SoundSlot component with waveform
  - Backend: full pipeline integration
  - Clean up unused scaffolding

Testing method:
  - Demo flow: 5 generations, all succeed, all playable
  - Memory check: no leaks (watch Activity Monitor / Task Manager)

Success criteria: Complete generation pipeline works end-to-end
```

---

## Week 2: "Six Slots" — Days 8-14

```
Goal: Full 6-slot sound grid with prompt input, preview, and export.
      This is the first version that feels like a real app.
```

### Day 8 — Prompt Input

```
Goal: User can type a prompt and press Enter to generate.

Exact tasks:
  - Build PromptBar component:
    - Text input (auto-focus on app start)
    - Placeholder: "Describe the sound you want..."
    - Enter key to generate (or ⌘+Enter)
    - Send prompt text to backend command
  - Implement prompt → embedding:
    - For MVP: simple bag-of-words feature vector from keyword mapping
    - Keywords: kick, snare, hat, 808, dark, punchy, bright, short, long
    - Future: replace with CLAP text encoder
  - Show loading state during generation
  - Disable input during generation (prevent double-submit)

Files to create/modify:
  - src/components/prompt/PromptBar.tsx
  - src/stores/useGenerationStore.ts
  - Backend: accept prompt string, convert to embedding

Technical decisions:
  - Prompt embedding is the weakest link in MVP — accept this
  - Simple keyword → latent vector mapping (50 keywords x 768-d table)
  - Document: "embedding quality will improve post-MVP"

Testing method:
  - Type "kick" → Enter → generates a kick-like sound
  - Type "snare" → Enter → generates snare-like sound
  - Empty prompt → show validation error

Success criteria: Typing a prompt and pressing Enter generates a sound
```

### Day 9 — Sound Grid

```
Goal: 6-slot grid that displays generated sounds with waveforms.

Exact tasks:
  - Build SoundGrid component (CSS grid, 3x2 or 6x1 layout)
  - Build SoundSlot component:
    - Waveform thumbnail (SVG or canvas, 80 data points)
    - Sound type label (badge)
    - Duration text
    - Hover: show quick actions (play, favorite, export)
  - Handle states: empty, loading, complete, error
  - Handle grid fill order: slot 1 → slot 2 → ... → slot 6
  - Overflow: oldest sound replaced after 6 (or user can clear)

Files to create/modify:
  - src/components/grid/SoundGrid.tsx
  - src/components/grid/SoundSlot.tsx
  - src/components/grid/WaveformThumbnail.tsx
  - src/stores/useGenerationStore.ts

Technical decisions:
  - Grid is always 6 slots (no pagination in MVP)
  - Waveform data: 80 Float32 values, pre-computed on backend
  - CSS transition on new sound appearance (fade in)

Testing method:
  - Generate 6 sounds → grid fills left to right
  - Generate 7th → replaces first
  - Each slot shows correct waveform, type, duration

Success criteria: 6-slot grid displays generated sounds
```

### Day 10 — Audio Playback

```
Goal: Click a sound slot → hear the sound. Instant.

Exact tasks:
  - Implement useAudioPlayback hook:
    - Create AudioContext on first user interaction
    - Load audio data via invoke('get_audio_data')
    - Decode to AudioBuffer
    - Play/stop/toggle
    - Crossfade between sounds (10ms)
  - Wire play button on SoundSlot:
    - Click → play (if stopped)
    - Click → stop (if playing)
    - Different slot → crossfade
  - Show playback state (playing indicator on slot)
  - Cache AudioBuffers in memory (last 20 sounds)

Files to create/modify:
  - src/hooks/useAudioPlayback.ts
  - src/components/grid/SlotControls.tsx
  - Backend: get_audio_data IPC command

Technical decisions:
  - Web Audio API for playback (not native)
  - Audio data transferred as Float32 array via IPC
  - Cache limit: 20 sounds ≈ ~35MB (acceptable for desktop)

Testing method:
  - Click slot → sound plays
  - Click again → sound stops
  - Click different slot → new sound plays (old stops)

Success criteria: Click any sound, hear it immediately
```

### Day 11 — WAV Export

```
Goal: Export a sound as a WAV file to disk.

Exact tasks:
  - Build export command in Rust:
    - Takes sound_id + output directory
    - Writes 44.1kHz / 24-bit / mono WAV
    - Auto-names: {type}_{timestamp}.wav
  - Build export dialog (or use Tauri's built-in save dialog):
    - File → Export (or Cmd+E)
    - Tauri save dialog: ~/Desktop or custom
  - Build Export button on SoundSlot:
    - Click → save dialog → export → success toast
  - Track export history (in memory for MVP)

Files to create/modify:
  - src-tauri/src/commands/export.rs
  - src/components/export/ExportDialog.tsx
  - src/hooks/useExport.ts
  - SoundSlot: add export button

Technical decisions:
  - Use Tauri's dialog plugin for native save dialog
  - 24-bit WAV as default (professional standard)
  - Filename includes type + date for organization

Testing method:
  - Generate sound → click export → save dialog → save
  - Open in DAW or Audacity → plays correctly
  - Check: 44.1kHz, 24-bit, mono, correct duration

Success criteria: Export a generated sound as a valid WAV file
```

### Day 12 — "More Like This" Variants

```
Goal: Click a sound → generate 6 variants of it.

Exact tasks:
  - Implement generate_variants backend command:
    - Takes source_id + count
    - Generates count new sounds with same prompt but different seeds
    - Returns array of sound metadata
  - Add "↻ Variants" button to SoundSlot
  - On click: generate 5 more sounds, fill remaining grid slots
  - Keep original sound, add 5 variants around it
  - Show variant relationship in UI (small parent link text)

Files to create/modify:
  - src-tauri/src/commands/generation.rs (add generate_variants)
  - src/components/grid/SlotControls.tsx (add variant button)
  - src/stores/useGenerationStore.ts (handle variant flow)

Technical decisions:
  - Variants = same prompt + same config + different seed
  - Variants fill empty slots first, then replace oldest
  - Original sound stays in place (doesn't get replaced)

Testing method:
  - Generate → click variants → 5 new sounds appear
  - All sounds are different (listen/verify)
  - Original sound remains

Success criteria: One click generates 5 variants of a sound
```

### Day 13 — Keyboard Shortcuts

```
Goal: Full keyboard control for generation workflow.

Exact tasks:
  - Implement useKeyboard hook:
    - Space: play/stop selected sound
    - Enter: generate from prompt
    - Cmd+1-6: select sound slots
    - Cmd+E: export selected sound
    - Esc: clear selection / close dialogs
    - ← → ↑ ↓: navigate grid
    - F: favorite/unfavorite
    - ?: show shortcuts help overlay
  - Add shortcut hints to UI elements (hover tooltips)
  - Build shortcuts reference panel (Cmd+/ or ?)

Files to create/modify:
  - src/hooks/useKeyboard.ts
  - src/components/shared/ShortcutsHelp.tsx
  - Modify SoundGrid, PromptBar to respond to keyboard events

Technical decisions:
  - All shortcuts use Cmd (macOS) / Ctrl (Windows/Linux)
  - Tauri detect platform, adjust display
  - Shortcuts work in all app states

Testing method:
  - Navigate grid with arrow keys
  - Space plays/pauses
  - Cmd+E exports
  - All shortcuts work without mouse

Success criteria: Full generation workflow works from keyboard alone
```

### Day 14 — Week 2 Integration

```
Goal: Polish the Week 2 flow. Full generation, preview, export, variants.

Exact tasks:
  - Error handling for all flows:
    - Generation fails → show error toast, retry button
    - Export fails → error toast, log details
    - Playback fails → silent failure, log
  - Loading states: skeleton placeholder on grid during generation
  - Toast notifications: success/error feedback
  - Basic styling pass: consistent spacing, colors, typography
  - Clean up temporary code, unused files

Demo scenario:
  1. Type "dark trap kick" → Enter
  2. 6 sounds appear over 5-10 seconds
  3. Click each → preview
  4. Click variants on best one → 5 more appear
  5. Favorite the best one (Cmd+F)
  6. Export it (Cmd+E) → save to desktop
  7. Open in DAW → it plays, it's a usable kick

Files to create/modify:
  - src/components/shared/Toast.tsx
  - Error boundaries
  - Styling pass on all components

Testing method:
  - Full scenario: 10 reps, all succeed
  - No crashes, no silent failures
  - Export: drag into Ableton → works

Success criteria: Complete generation-to-export flow in <30 seconds
```

---

## Week 3: "Don't Lose It" — Days 15-21

```
Goal: Library, favorites, tags, persistence across sessions.
      Every sound you make is saved and searchable.
```

### Day 15 — Database Setup

```
Goal: SQLite database initialized on app start, ready for use.

Exact tasks:
  - Add rusqlite crate to Cargo.toml
  - Create src-tauri/src/db/mod.rs:
    - Database struct with connection pool (single connection for MVP)
    - init() → create tables if not exist
    - Schema: sounds, tags, exports tables
  - Create migration system (versioned SQL scripts)
  - Run migrations on every app start
  - Create database at ~/cShot/library.db

Files to create/modify:
  - src-tauri/Cargo.toml (add rusqlite)
  - src-tauri/src/db/mod.rs
  - src-tauri/src/db/migrations.rs
  - src-tauri/src/db/migrations/001_initial.sql

Technical decisions:
  - rusqlite with bundled SQLite (no system dependency)
  - Single connection (thread-safe via Mutex for MVP)
  - Migration files in src-tauri/src/db/migrations/

Testing method:
  - cargo test — db tests pass
  - Delete library.db → restart → recreated with schema
  - Query tables: they exist with correct columns

Success criteria: SQLite database initializes with correct schema
```

### Day 16 — Save Generated Sounds

```
Goal: Every generated sound is automatically saved to the database.

Exact tasks:
  - Implement save_sound() in database module:
    - Insert sound metadata: id, hash, prompt, seed, model version, params
    - Insert analysis data: type, duration, rms, peak
  - Integrate into generation pipeline:
    - After post-process + classify → save to DB
  - Create sound_id (UUID) as primary key
  - Verify saved sounds persist across app restarts

Files to create/modify:
  - src-tauri/src/db/sounds.rs
  - Modify generation pipeline to call db.save_sound()

Technical decisions:
  - Auto-save: no user action needed
  - Sound metadata saved immediately after generation
  - Actual audio file saved to content-addressed storage (already working)

Testing method:
  - Generate sound → close app → reopen → query DB → sound exists
  - Generate 10 sounds → all in DB → query returns 10

Success criteria: All generated sounds survive app restarts
```

### Day 17 — Favorites System

```
Goal: Mark sounds as favorites, view them in a separate list.

Exact tasks:
  - Add is_favorite column to sounds table
  - Implement toggle_favorite backend command
  - Add ★ favorite button to SoundSlot (toggle state)
  - Add favorites list view:
    - Toggle: show all sounds / show favorites only
    - List shows waveform thumbnails, type, prompt
  - Implement get_favorites backend query
  - Sync favorite state across sessions

Files to create/modify:
  - src-tauri/src/commands/library.rs
  - src/components/grid/SlotControls.tsx (favorite button)
  - src/components/library/FavoritesList.tsx
  - src/stores/useLibraryStore.ts

Technical decisions:
  - Favorite = boolean toggle (no star ratings in MVP)
  - Favorites view = filtered grid (same SoundSlot component)
  - Favorite state synced eagerly (no delay)

Testing method:
  - ★ a sound → star fills → close app → reopen → star still filled
  - Toggle to favorites-only view → only favorited sounds shown
  - Unstar → removed from favorites

Success criteria: Favorites persist across sessions and are filterable
```

### Day 18 — Tag System

```
Goal: Auto-tag generated sounds, let users add/remove tags.

Exact tasks:
  - Implement tags table in DB (sound_id, tag, source, confidence)
  - Auto-tag on generation:
    - Sound type → tag (e.g., "kick", "snare")
    - Analysis-based tags: "short", "loud", "bright", "dark"
    - Prompt keyword → tag: if prompt contains "808" → tag "808"
  - Tag editor UI on detail panel:
    - Show auto-tags (source=auto, dimmer)
    - Show user-tags (source=user, normal)
    - Click + to add tag (text input → Enter)
    - Click × to remove tag
  - Implement add_tag / remove_tag backend commands

Files to create/modify:
  - src-tauri/src/db/tags.rs
  - src-tauri/src/commands/library.rs (add tag CRUD)
  - src/components/detail/TagEditor.tsx
  - Tag integration into generation pipeline

Technical decisions:
  - Auto-tags marked as source='auto' (can be promoted to user)
  - Tags are simple strings (no hierarchy, no categories in MVP)
  - Max 20 tags per sound (prevent bloat)

Testing method:
  - Generate → auto-tags appear
  - Add user tag: "my-favorite-kick" → persists
  - Remove auto-tag → tag gone
  - Close/reopen → tags still there

Success criteria: Sounds are auto-tagged, users can add/remove tags
```

### Day 19 — Library Browser

```
Goal: Browse all saved sounds in a searchable list.

Exact tasks:
  - Build LibraryPage component:
    - Grid view of all saved sounds (paginated: 12 per page)
    - Each item: waveform thumbnail, type badge, tags, prompt
    - Sort by: date (newest first), type, favorites
    - Filter by: type, tag, date range
  - Implement search_library backend command:
    - Full-text search on prompt text
    - Filter by sound_type
    - Filter by tag
    - Sort + pagination
  - Add link in TopBar to switch between Generator / Library

Files to create/modify:
  - src/pages/LibraryPage.tsx
  - src/components/library/LibraryGrid.tsx
  - src-tauri/src/commands/library.rs (add search)
  - src-tauri/src/db/search.rs

Technical decisions:
  - SQLite FTS5 for full-text search
  - Simple pagination (page-based, not infinite scroll in MVP)
  - Library is read-only from this view (actions: play, export, tag, favorite)

Testing method:
  - Generate 20 sounds → all appear in Library
  - Search "kick" → only kick-type sounds shown
  - Filter by tag "dark" → dark-tagged sounds shown
  - Sort by date → oldest/newest first

Success criteria: Library shows all saved sounds with search and filters
```

### Day 20 — Reference Audio Import

```
Goal: User can drag audio files into cShot for analysis + generation-near.

Exact tasks:
  - Build ReferenceDropZone component:
    - Drag-and-drop zone in prompt area
    - Accepts: WAV, AIFF, FLAC, MP3
    - Click to open file picker as fallback
  - Implement import_audio backend command:
    - Decode audio file (use symphonia for broad format support)
    - Convert to mono 44.1kHz f32
    - Run analysis: BPM, key, spectral content
    - Save to library (marked source='imported')
  - Show import results: "128 BPM, F# minor, punchy"
  - Pre-fill prompt with detected characteristics
  - Add "Generate Similar" button (uses reference as generation seed)

Files to create/modify:
  - src/components/prompt/ReferenceDropZone.tsx
  - src-tauri/src/commands/import.rs
  - src-tauri/src/audio/import.rs (decode + analyze)
  - src-tauri/Cargo.toml (add symphonia for audio decoding)

Technical decisions:
  - symphonia for audio decoding (supports WAV, AIFF, FLAC, MP3, OGG)
  - Imported files copied to content-addressed storage
  - BPM detection: simple onset-based algorithm
  - Key detection: chromagram + template matching

Testing method:
  - Drag WAV file → imported, analyzed, shows in library
  - Check: BPM and key are at least close to correct
  - "Generate Similar" produces sound in same ballpark

Success criteria: Drag audio in, cShot analyzes it, can generate-near-it
```

### Day 21 — Week 3 Integration

```
Goal: All Week 3 features working together. Full library flow.

Exact tasks:
  - Polish library page layout and performance
  - Handle empty states (no library, no favorites, no tags)
  - Handle edge cases: import same file twice → dedup
  - Fix: favorites sync across generator/library views
  - Fix: tags editable from generator detail panel
  - Refine layout, fix spacing/alignment issues
  - Write a comprehensive test script for the full flow

Demo scenario:
  1. Import a drum loop from user's desktop
  2. cShot analyzes: 140 BPM, C# minor
  3. Prompt: "kick that fits" → generates 6 kicks
  4. Favorite best 2, tag them "trap-kick"
  5. Switch to Library → filter by tag "trap-kick" → see 2 kicks
  6. Click one → preview → export → done

Files to create/modify:
  - Polish pass on all Week 3 components
  - Error states for library/import

Testing method:
  - Full scenario works without bugs
  - Library loads <1s even with 100+ sounds
  - Import 5 different formats (wav, aiff, flac, mp3, ogg) → all work

Success criteria: Complete library workflow — import, generate, tag, browse, export
```

---

## Week 4: "Ship It" — Days 22-30

```
Goal: Polish, packaging, documentation, and beta release.
      Make it something you'd actually send to a friend.
```

### Day 22 — Error Handling & Robustness

```
Goal: App doesn't crash on any normal error. Graceful degradation.

Exact tasks:
  - Wrap every IPC command in Result<T, E>
  - Map backend errors to user-friendly messages:
    - "Generation failed: model not loaded. Try restarting."
    - "Export failed: disk full or write permission denied."
    - "Import failed: unsupported audio format."
  - Frontend error boundaries:
    - React ErrorBoundary per major section
    - Fallback UI: "Something went wrong" + retry button
    - Specific: generation error → show error on affected slot
  - Handle: model loading failure, GPU OOM, disk full, file not found

Files to create/modify:
  - src/components/shared/ErrorBoundary.tsx
  - src-tauri/src/error.rs (error types + user messages)
  - Update all commands with proper error handling

Testing method:
  - Delete model file → generation fails gracefully
  - Fill disk → export shows "disk full" message
  - Remove permissions → clear error message
  - No crashes in any error scenario

Success criteria: App handles all errors gracefully, no crashes
```

### Day 23 — Performance Optimization

```
Goal: Meet MVP performance targets.

Exact tasks:
  - Profile backend:
    - Where is time spent? (model loading, inference, post-process, save)
    - Optimize: batch inference? Model quantization (FP16)?
    - Optimize: audio cache (don't re-read from disk on preview)
    - Optimize: waveform thumbnail computation (pre-compute on gen)
  - Profile frontend:
    - React DevTools: unnecessary re-renders?
    - Audio playback: optimize buffer management
    - Grid rendering: virtualization if needed (not for 6 slots)
  - Memory check: generate 50 sounds, check memory usage

Targets:
  - Model load: <2 seconds (cold start)
  - Generation: <5 seconds (GPU), <20 seconds (M1), <60 seconds (CPU)
  - Preview: <5ms from click to audio
  - Export: <50ms
  - Memory: <500MB idle, <2GB during generation

Files to modify:
  - src-tauri/src/audio/cache.rs
  - src-tauri/src/model/inference.rs (FP16 quantization)
  - Frontend: memoize components, useCallback/useMemo as needed

Testing method:
  - Measure all targets with console.time / profiling tools
  - Generate 50 sounds → measure average time
  - Check memory stable (no leaks)

Success criteria: All performance targets met
```

### Day 24 — Settings Page

```
Goal: User can configure basic preferences.

Exact tasks:
  - Build SettingsPage:
    - Audio: output directory, default format (WAV/44.1k/24-bit)
    - Generation: default duration, model preference (fast/quality)
    - Behavior: auto-preview on/off, dark/light theme
    - Storage: library location, cache size
    - About: version, model versions, credits
  - Implement settings persistence (config.toml)
  - Implement config read/write backend commands

Files to create/modify:
  - src/pages/SettingsPage.tsx
  - src-tauri/src/config/mod.rs
  - src-tauri/src/config/settings.rs
  - TopBar: add Settings link

Technical decisions:
  - config.toml in ~/cShot/config.toml
  - Settings auto-saved on change (no "save" button)
  - Defaults: output=~/Desktop, format=WAV/44.1k/24-bit, auto-preview=on

Testing method:
  - Change output directory → export goes to new location
  - Disable auto-preview → generate doesn't auto-play
  - Change model to "fast" → generation is faster, lower quality
  - Restart app → settings persist

Success criteria: Settings page works and persists preferences
```

### Day 25 — Visual Polish

```
Goal: App looks good enough to show someone.

Exact tasks:
  - Consistent design system pass:
    - Colors: dark theme with accent purple (#6C5CE7)
    - Typography: JetBrains Mono for UI, Inter for prose
    - Spacing: 4px grid, consistent padding
    - Border radius: 8px cards, 4px buttons
    - Shadows: subtle depth on hovered/active elements
  - Animations: generation progress, sound appearance, transitions
  - Responsive: window resize works (min 800x600)
  - Dark/light theme toggle (from settings)
  - Favicon / app icon

Files to modify:
  - src/styles/globals.css (design tokens)
  - All components: consistent styling pass
  - src-tauri/tauri.conf.json (window title, icon)

Technical decisions:
  - CSS custom properties for theming
  - Tailwind with custom config for design system
  - No animation library (CSS transitions + keyframes)

Testing method:
  - Visual inspection: all pages look consistent
  - Resize: layout adjusts gracefully
  - Theme toggle: light/dark both work

Success criteria: App looks polished and professional
```

### Day 26 — Onboarding & Help

```
Goal: First-time user understands what to do.

Exact tasks:
  - Build onboarding overlay (first launch only):
    - Step 1: "Type what you want to hear" (highlights prompt)
    - Step 2: "Click a sound to preview" (highlights grid)
    - Step 3: "Export to your DAW" (highlights export)
    - Dismiss: "Got it" button + "Don't show again" checkbox
  - Build quick-start guide (accessible from help menu):
    - "How to generate sounds"
    - "How to use variants"
    - "How to organize your library"
    - "Keyboard shortcuts"
  - Add tooltips on all UI elements (hover delay: 500ms)
  - Add status bar tips ("Tip: Press Space to preview")

Files to create/modify:
  - src/components/onboarding/OnboardingOverlay.tsx
  - src/pages/HelpPage.tsx
  - Add tooltips to all interactive elements
  - StatusBar: rotating tips

Technical decisions:
  - Onboarding shown once (stored in settings)
  - Can be re-triggered from Help menu
  - Tooltips: simple CSS-based, no library needed

Testing method:
  - Delete config → restart → onboarding appears
  - Complete onboarding → dismissed permanently
  - Access help → quick-start guide readable

Success criteria: New user can understand the app without instructions
```

### Day 27 — Packaging & Distribution

```
Goal: App can be installed on any supported platform.

Exact tasks:
  - Tauri build configuration:
    - macOS: .dmg with code signing (Apple Developer ID)
    - Windows: .msi or .exe with signing
    - Linux: .AppImage (universal) + .deb
  - Configure: app name, version, icon, publisher
  - Configure auto-update (Tauri updater + GitHub Releases)
  - Test clean install on:
    - macOS (Intel + Apple Silicon)
    - Windows 10/11
    - Ubuntu 22.04 / Fedora
  - Handle first-launch model download:
    - Check ~/cShot/models/ on start
    - If missing, download from CDN with progress bar
    - Resume on interruption

Files to create/modify:
  - src-tauri/tauri.conf.json (build config)
  - src-tauri/updater.json (Tauri update config)
  - src-tauri/src/model/loader.rs (download on first launch)
  - .github/workflows/release.yml (CI build)

Technical decisions:
  - Models served from CDN (Cloudflare R2 or S3)
  - App binary: ~10MB, models: ~500MB-2GB (separate download)
  - Auto-update: Tauri built-in updater (checks GitHub Releases)

Testing method:
  - Build on macOS: creates .dmg
  - Clean install: download → open → works
  - First launch: model downloads with visible progress
  - Auto-update: old version → opens → new version available → updates

Success criteria: App installs and runs on all three platforms
```

### Day 28 — Testing & Bug Fixes

```
Goal: App is stable enough for beta testers.

Exact tasks:
  - Run comprehensive test script (30 scenarios):
    1. Generate from prompt (10 prompts)
    2. Generate variants
    3. Export sound
    4. Batch export
    5. Favorite/unfavorite
    6. Add/remove tags
    7. Search library
    8. Import audio file
    9. Generate from imported reference
    10. Settings changes
    11. Keyboard shortcuts
    12. App restart (persistence check)
    13. Window resize
    14. Theme toggle
    15. Empty states
    16. Error states
    17. Performance (time generation)
    18. Memory usage
    19. File > 100MB import
    20. Multi-format import
  - Fix all bugs found
  - Log remaining known issues in RELEASE_NOTES.md

Files to create/modify:
  - MANUAL_TESTING.md (test script)
  - RELEASE_NOTES.md (known issues, limitations)
  - Bug fixes throughout

Testing method:
  - Execute test script on macOS, Windows, Linux
  - Each scenario: pass/fail
  - All P0 bugs fixed before release

Success criteria: Test script passes at 95%+ on all platforms
```

### Day 29 — Documentation & Demo

```
Goal: Beta testers can understand and use the app.

Exact tasks:
  - Write QUICKSTART.md:
    - Installation
    - First launch
    - Basic usage (3 screenshots)
    - Export to DAW
    - Tips & tricks
  - Record demo video (60 seconds):
    - Open app → type prompt → generate → preview → favorite → export
    - Screen recording with voiceover (optional)
  - Write RELEASE_NOTES.md:
    - What is cShot
    - Features (MVP scope)
    - Known limitations
    - System requirements
    - How to give feedback
  - Create GitHub Issues template for bug reports

Files to create/modify:
  - QUICKSTART.md
  - RELEASE_NOTES.md
  - .github/ISSUE_TEMPLATE/bug_report.md
  - .github/ISSUE_TEMPLATE/feature_request.md

Testing method:
  - Ask one friend to follow QUICKSTART without help
  - Time to first exported sound: target <2 minutes

Success criteria: A non-technical user can install and use the app from docs alone
```

### Day 30 — Release Day

```
Goal: Ship v0.1.0-beta to first 50 testers.

Exact tasks:
  - Build final release binaries:
    - cShot-0.1.0-beta-x86_64.dmg (macOS Intel)
    - cShot-0.1.0-beta-aarch64.dmg (macOS Apple Silicon)
    - cShot-0.1.0-beta-x86_64.msi (Windows)
    - cShot-0.1.0-beta-x86_64.AppImage (Linux)
  - Upload to GitHub Release (tag v0.1.0-beta)
  - Send to first 50 beta testers (email / Discord / Twitter)
  - Set up feedback channel (Discord server or GitHub Discussions)
  - Celebrate. This is the first working version of cShot.

Files to create:
  - GitHub Release v0.1.0-beta
  - CHANGELOG.md (first entry)

Testing method:
  - Each tester downloads and runs
  - Track: install success rate, crash rate, first export time
  - Collect: "what did you think?" survey

Success criteria: 50 people download and run cShot
```

---

## Build Plan Summary

### File/Module Creation Timeline

```
Day  1: src-tauri/Cargo.toml, src-tauri/src/main.rs, src/App.tsx, package.json
Day  2: src-tauri/src/audio/mod.rs, src-tauri/src/audio/process.rs
Day  3: src-tauri/src/model/mod.rs, src-tauri/src/model/inference.rs
Day  4: src-tauri/src/commands/generation.rs
Day  5: src-tauri/src/audio/pipeline.rs
Day  6: src-tauri/src/audio/classify.rs, src-tauri/src/audio/analyze.rs
Day  7: src/components/grid/SoundSlot.tsx (waveform)
Day  8: src/components/prompt/PromptBar.tsx, src/stores/useGenerationStore.ts
Day  9: src/components/grid/SoundGrid.tsx, WaveformThumbnail.tsx
Day 10: src/hooks/useAudioPlayback.ts
Day 11: src-tauri/src/commands/export.rs, src/components/export/ExportDialog.tsx
Day 12: generate_variants command (modify generation.rs)
Day 13: src/hooks/useKeyboard.ts, ShortcutsHelp.tsx
Day 14: src/components/shared/Toast.tsx (polish)
Day 15: src-tauri/src/db/mod.rs, src-tauri/src/db/migrations.rs
Day 16: src-tauri/src/db/sounds.rs
Day 17: src-tauri/src/commands/library.rs, src/components/library/FavoritesList.tsx
Day 18: src-tauri/src/db/tags.rs, src/components/detail/TagEditor.tsx
Day 19: src/pages/LibraryPage.tsx, src-tauri/src/db/search.rs
Day 20: src/components/prompt/ReferenceDropZone.tsx, src-tauri/src/commands/import.rs
Day 21: Polish pass on library components
Day 22: src/components/shared/ErrorBoundary.tsx, src-tauri/src/error.rs
Day 23: src-tauri/src/audio/cache.rs, optimization pass
Day 24: src/pages/SettingsPage.tsx, src-tauri/src/config/mod.rs
Day 25: src/styles/globals.css, styling pass
Day 26: src/components/onboarding/OnboardingOverlay.tsx, src/pages/HelpPage.tsx
Day 27: src-tauri/tauri.conf.json, tauri/updater.json, CI workflow
Day 28: MANUAL_TESTING.md, bug fixes
Day 29: QUICKSTART.md, RELEASE_NOTES.md, demo video
Day 30: GitHub Release, distribution
```

### Technical Decisions Log (Week 4)

| Decision | Choice | Why |
|----------|--------|-----|
| Distribution | GitHub Releases + direct download | Simple, free, no store fees |
| Auto-update | Tauri updater | Built-in, minimal setup |
| Model delivery | CDN download on first launch | Keep binary small, separate concerns |
| Beta channel | Direct email / Discord | Controlled rollout, immediate feedback |
| Bug tracking | GitHub Issues | Integrated with repo |
| Feedback | Discord server | Real-time, community building |
| Documentation | Markdown files in repo | Version-controlled, simple |

### Risk Register (Week 4)

| Risk | Mitigation |
|------|-----------|
| Model download too large (2GB+) | Compress model, show progress, resume support |
| Code signing certificates not obtained | Start process early (Apple takes 1-2 weeks) |
| Cross-platform build issues | Test on actual hardware, not just CI |
| Beta testers don't show up | Have 50+ signups before release day |
| Critical bug found on release day | 24-hour hotfix window, communicate clearly |

---

## Post-MVP Roadmap

```
Day 31+: Based on beta feedback, prioritize:

Week 5:  Quality improvements (model fine-tuning, better prompts)
Week 6:  VST3/AU plugin (standalone → DAW integration)
Week 7:  Variation tree (graph visualization)
Week 8:  Mix-readiness engine (EQ, compression, transient shaping)
Week 9:  Genre-aware processing
Week 10: Copyright safety (similarity detection, provenance)
```

---

## Final Note

```
The MVP is not the vision. It's the smallest thing that proves the core idea:

  "Type what you want, get a unique, usable sound."

Everything else — the DAW plugins, the variation tree, the latent explorer,
the mix engine, the collaborative platform — is built on top of this.

If the MVP doesn't make you smile the first time you hear a generated sound,
nothing else matters. Get that feeling right in 30 days.
```
