# Prompt 102 — Claude Code Prompt Pack for Building cShot v1

## How to Use

Each prompt below is designed to be handed directly to Claude Code (or any LLM coding agent) as a complete specification. Each prompt includes:
- **Goal**: what to build
- **File structure**: where to put things
- **Acceptance criteria**: how to know it's done
- **Dependencies**: what must exist first

Run prompts in order within each phase. Each prompt builds on the previous one.

---

## Phase 1: Foundation (Prompts 1-8)

### Prompt 1: Initialize Tauri v2 + React Project

```
Goal: Create a working Tauri v2 desktop app with React + TypeScript + Vite + Tailwind CSS.

Steps:
1. Run `npm create tauri-app@latest cshot -- --template react-ts`
2. Install dependencies: `npm install zustand tailwindcss @tailwindcss/vite lucide-react`
3. Configure Vite with Tailwind plugin
4. Set up Tailwind CSS with a dark theme (bg-gray-950, text-gray-100, accent-purple-500)
5. Create basic AppShell layout: TopBar (logo + title), main content area, StatusBar
6. Create `src/components/layout/AppShell.tsx`, `TopBar.tsx`, `StatusBar.tsx`
7. Verify the app runs with `npm run tauri dev`

Acceptance criteria:
- `npm run tauri dev` opens a native window with a dark background
- TopBar shows "cShot" logo/title
- StatusBar shows at the bottom
- The app window is resizable and has a minimum size of 900x600

Tech stack:
- Tauri v2 with Rust backend
- React 18 + TypeScript
- Vite 6
- Tailwind CSS v4
- Zustand for state management
```

### Prompt 2: Build the Rust Audio DSP Library

```
Goal: Create the core audio DSP pipeline in Rust for one-shot processing.

File location: `src-tauri/src/audio/dsp/`

Modules to create:
- `mod.rs` - re-exports
- `trim.rs` - Silence trimming (trim leading/trailing below -60dB threshold with 100ms hold)
- `normalize.rs` - Peak normalization to -1.0dB true peak
- `fade.rs` - Fade in (5ms linear) and fade out (10ms linear)
- `analyze.rs` - Compute RMS (dB), peak (dB), crest factor, spectral centroid, transient onset time

Technical requirements:
- All functions take `&[f32]` samples and `u32` sample_rate
- All functions return `Vec<f32>` or analysis struct
- Use `f32` for all audio processing
- No dependencies beyond std (no FFT crate for spectral centroid - use simplified approximation)
- Write unit tests using golden WAV files. Create test fixtures in `src-tauri/tests/fixtures/`
- Test: trim, normalize, fade, analyze (500+ test cases total)
- Test edge cases: silence, near-silence, clipped audio, very short audio (<10ms), DC offset

Also create `src-tauri/src/audio/pipeline.rs` that orchestrates the full DSP chain:
1. Trim silence
2. Normalize peak to -1.0dB
3. Fade in (5ms)
4. Fade out (10ms)
5. Analyze

Acceptance criteria:
- `cargo test` passes all DSP tests
- Pipeline produces correct output given known input (verify with test golden files)
- Pipeline handles edge cases without panicking
```

### Prompt 3: Implement Content-Addressed Storage

```
Goal: Create a content-addressed file storage system for audio files.

File location: `src-tauri/src/library/storage.rs`

Requirements:
- Audio files stored at `~/.cshot/audio/{2-char-prefix}/{full-hash}.wav`
- SHA-256 hash of audio data used as filename
- Store function: accepts Vec<f32> samples, sample_rate, bit_depth → returns SHA-256 hash string
- Read function: accepts hash → returns audio data + metadata
- Dedup: if same audio already stored, return existing hash without writing duplicate
- Verify hash on read: if file hash doesn't match stored hash, return error
- Write audio as WAV (using `hound` crate: 24-bit WAV for storage, 44.1kHz)
- Delete function: removes file (with verification that hash matches)

Dependencies:
- `hound` crate for WAV read/write
- `sha2` crate for SHA-256
- `anyhow` for error handling

Also create `src-tauri/src/library/mod.rs` that exports the storage module.

Acceptance criteria:
- Store audio → get hash → read by hash → get identical audio
- Storing same audio twice returns same hash (dedup)
- Hash on read is verified; corrupt file returns error
- Files are stored in correct 2-level prefix directory structure
```

### Prompt 4: Initialize SQLite Database

```
Goal: Set up SQLite database for sound metadata using rusqlite.

File location: `src-tauri/src/library/database.rs`

Schema to create in `fn init_database(path: &Path) -> Result<Connection>`:

```sql
CREATE TABLE IF NOT EXISTS sounds (
    id TEXT PRIMARY KEY,
    hash TEXT NOT NULL UNIQUE,
    prompt TEXT NOT NULL,
    text_embedding BLOB,
    model TEXT NOT NULL,
    model_version TEXT NOT NULL DEFAULT 'v1',
    seed INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    sample_rate INTEGER NOT NULL DEFAULT 44100,
    bit_depth INTEGER NOT NULL DEFAULT 24,
    channels INTEGER NOT NULL DEFAULT 1,
    file_size_bytes INTEGER NOT NULL,
    rms REAL,
    peak REAL,
    crest_factor REAL,
    spectral_centroid REAL,
    transient_time_ms INTEGER,
    soundscore_overall REAL,
    user_rating INTEGER,
    tags TEXT DEFAULT '[]',
    is_exported INTEGER DEFAULT 0,
    reference_hash TEXT,
    parent_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_sounds_hash ON sounds(hash);
CREATE INDEX IF NOT EXISTS idx_sounds_created ON sounds(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sounds_score ON sounds(soundscore_overall DESC);
```

Implement CRUD operations:
- `insert_sound(conn, entry: SoundEntry) -> Result<String>` (returns id)
- `get_sound(conn, id: &str) -> Result<SoundEntry>`
- `get_sound_by_hash(conn, hash: &str) -> Result<SoundEntry>`
- `list_sounds(conn, limit, offset) -> Result<Vec<SoundEntry>>`
- `update_sound(conn, id: &str, updates: SoundUpdate) -> Result<()>`
- `delete_sound(conn, id: &str) -> Result<()>`
- `search_sounds(conn, query: &str) -> Result<Vec<SoundEntry>>` (LIKE search on prompt)

Define types in `src-tauri/src/types/mod.rs`.

Dependencies:
- `rusqlite` crate with `bundled` feature

Acceptance criteria:
- `cargo test` passes all CRUD tests
- Database file is created at specified path
- Insert and retrieve return identical data
- Search finds sounds by prompt text
- Schema matches the spec exactly
```

### Prompt 5: Implement ElevenLabs SFX API Client

```
Goal: Create a model gateway with ElevenLabs SFX API client as the primary generation model.

File location: `src-tauri/src/generation/`

Files to create:
- `mod.rs` - re-exports
- `gateway.rs` - ModelGateway that routes to available models
- `models/mod.rs` - AudioGenerator trait
- `models/elevenlabs.rs` - ElevenLabs SFX API client

AudioGenerator trait in `models/mod.rs`:
```rust
#[async_trait]
pub trait AudioGenerator: Send + Sync {
    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse>;
    fn model_name(&self) -> &str;
    fn supports_reference(&self) -> bool;
}
```

ElevenLabsClient in `models/elevenlabs.rs`:
- API endpoint: POST `https://api.elevenlabs.io/v1/sound-generation`
- Header: `xi-api-key: {api_key}`
- Request body: `{ "text": prompt, "duration_seconds": 1.0, "seed": seed }`
- Response: binary audio (MP3 or WAV)
- Decode response to `Vec<f32>` samples at 44100 Hz
- Support `reqwest` for HTTP with timeout (15 seconds)
- Handle: 4xx errors (auth failure, rate limit), 5xx errors (server error), network errors
- Log all API calls with duration

Gateway in `gateway.rs`: (Phase 1 is simple — just ElevenLabs)
```rust
pub struct ModelGateway {
    elevenlabs: ElevenLabsClient,
}

impl ModelGateway {
    pub async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        self.elevenlabs.generate(&request).await
    }
}
```

Dependencies:
- `reqwest` with `json` feature
- `async-trait` crate
- `thiserror` for error types
- `serde` + `serde_json` for JSON

Also create `src-tauri/src/generation/prompt_encoder.rs`:
- Placeholder text encoder that returns a fixed 768-dim embedding
- (ML-optimized version will come in a later sprint)

Acceptance criteria:
- Mock HTTP test verifies correct request formation to ElevenLabs
- Error handling covers: timeout, 401, 429, 500, network error
- Generated audio is decoded to valid float32 samples
- Gateway trait allows easy addition of new models
```

### Prompt 6: Wire Generation End-to-End

```
Goal: Connect the full generation pipeline: Tauri IPC command → model gateway → DSP → storage → database → UI.

File location: `src-tauri/src/commands/generation.rs`

Tauri command:
```rust
#[tauri::command]
async fn generate(
    state: State<'_, AppState>,
    prompt: String,
    reference_path: Option<String>,
    count: Option<usize>,
) -> Result<GenerationResult, String>
```

This command:
1. Creates `GenerationRequest` from prompt
2. Calls `ModelGateway::generate()` 
3. Passes raw audio through `DspPipeline::process()`
4. Stores processed audio via `ContentAddressedStore::store()`
5. Inserts metadata into database
6. Returns `GenerationResult` with sound data and metadata

Create `AppState` in `src-tauri/src/main.rs`:
```rust
pub struct AppState {
    pub pipeline: Arc<Mutex<GenerationPipeline>>,
    pub gateway: Arc<ModelGateway>,
    pub storage: Arc<ContentAddressedStore>,
    pub db: Arc<Mutex<Connection>>,
}
```

Register the command in Tauri builder.

On the frontend, create `src/lib/api.ts` with:
```typescript
export async function generate(prompt: string, count?: number) {
    return await invoke('generate', { prompt, count });
}
```

Create `src/stores/useGenerationStore.ts`:
```typescript
interface GenerationState {
    prompt: string;
    isGenerating: boolean;
    results: SoundSlot[];
    setPrompt: (prompt: string) => void;
    generate: () => Promise<void>;
    clearResults: () => void;
}
```

Acceptance criteria:
- Select "Generate" from Tauri UI triggers backend pipeline
- Sound data returns to frontend and is accessible via Zustand store
- Generated audio is stored to disk and recorded in database
- `cargo build` compiles without errors
- `npm run tauri dev` runs and frontend can call `invoke('generate')`
```

### Prompt 7: Build SoundGrid UI

```
Goal: Create the main generation interface: prompt bar, sound grid, playback.

Component file locations:
- `src/components/prompt/PromptBar.tsx` - text input + Generate button
- `src/components/grid/SoundGrid.tsx` - 2×3 responsive grid
- `src/components/grid/SoundSlot.tsx` - individual sound card
- `src/components/grid/WaveformThumbnail.tsx` - SVG waveform renderer
- `src/hooks/useAudioPlayback.ts` - Web Audio API playback hook
- `src/hooks/useGeneration.ts` - generation hook wrapping IPC
- `src/stores/useAudioStore.ts` - playback state

PromptBar:
- Input field with placeholder: "Describe the sound you want... (e.g. 'punchy 808 kick, sub-bass tail')"
- Generate button (disabled when input is empty or isGenerating)
- Submit on Enter
- Loading state during generation
- Focus input on app load

SoundGrid:
- 2×3 grid of SoundSlot components
- Fills progressively as each generation completes
- Loading skeleton for empty/unfilled slots

SoundSlot:
- WaveformThumbnail (SVG path rendered as thin line, light gray on dark bg)
- SoundScore badge (colored: red <60, yellow 60-80, green >80) - placeholder if no score yet
- Sound type label (auto-detected: kick, snare, etc.) - placeholder if unknown
- Play/stop on click
- Selected state (purple border glow)
- Hover effect (slight scale + brighter)

WaveformThumbnail:
- Accepts Float32Array audio data
- Generates SVG path: downsample to ~100 points, draw polyline
- Animates during playback (playhead sweeps left to right)

useAudioPlayback:
- Web Audio API AudioContext
- decodeAudioData from raw float32 (convert to WAV buffer first)
- play(), stop(), isPlaying state
- Playback position tracking for waveform animation

useGeneration:
- Wraps invoke('generate')
- Handles loading state
- Handles errors (toast on failure)

Acceptance criteria:
- Type prompt → Enter → grid fills with 6 slots progressively
- Click any slot → sound plays → waveform animates → click again → stops
- Selected slot has visual highlight
- Empty state shows helpful placeholder text
- Loading state shows skeleton slots
```

### Prompt 8: Build Library Persistence & Detail View

```
Goal: Create library view with search, detail panel, and full-text search.

Component file locations:
- `src/components/library/LibraryView.tsx` - library grid view
- `src/components/library/LibrarySearch.tsx` - search bar + filter chips
- `src/components/detail/DetailPanel.tsx` - full detail overlay
- `src/components/detail/WaveformViewer.tsx` - zoomable waveform
- `src/components/detail/SoundScoreDisplay.tsx` - score breakdown
- `src/components/detail/MetadataCard.tsx` - sound metadata
- `src/stores/useLibraryStore.ts` - library state

Add Rust backend commands:
```rust
#[tauri::command]
async fn get_library(state: State<'_, AppState>) -> Result<Vec<SoundEntry>, String>

#[tauri::command]
async fn search_library(state: State<'_, AppState>, query: String) -> Result<Vec<SoundEntry>, String>

#[tauri::command]
async fn get_sound_detail(state: State<'_, AppState>, sound_id: String) -> Result<SoundDetail, String>
```

Implement `search_sounds` in database.rs using SQL LIKE on prompt field.
For FTS5, create virtual table:
```sql
CREATE VIRTUAL TABLE IF NOT EXISTS sounds_fts USING fts5(prompt, tags, content='sounds', content_rowid='rowid');
CREATE TRIGGER IF NOT EXISTS sounds_ai AFTER INSERT ON sounds BEGIN
    INSERT INTO sounds_fts(rowid, prompt, tags) VALUES (new.rowid, new.prompt, new.tags);
END;
```

Detail panel shows:
- Large waveform with zoom controls
- Spectral display placeholder (simplified frequency bars)
- SoundScore breakdown (4 horizontal bars: Punch, Body, Clarity, Uniqueness)
- Metadata: duration, sample rate, bit depth, RMS, peak, crest factor
- Prompt text (full, read-only)
- Model name, seed, created date
- Tags (display + edit)
- Rating (1-5 stars, clickable)
- Export button, Delete button, "Add to Pack" button

Acceptance criteria:
- Library shows all generated sounds in a grid
- Search filters in real-time as user types
- Click a sound → detail panel opens with full info
- Tag editing works (add/remove tags)
- Rating works (click star → persisted)
- Library persists across app restarts (generate → quit → reopen → find in library)
```

---

## Phase 2: Power Features (Prompts 9-14)

### Prompt 9: Implement Reference Upload & Analysis

```
Goal: Add reference audio upload with analysis and reference-based generation.

Backend: `src-tauri/src/commands/reference.rs`
```rust
#[tauri::command]
async fn analyze_reference(path: String) -> Result<ReferenceAnalysis, String>

#[tauri::command]
async fn generate_with_reference(
    state: State<'_, AppState>, 
    prompt: String,
    reference_path: String,
    count: Option<usize>,
) -> Result<GenerationResult, String>
```

References:
- Accepts: WAV, MP3, FLAC, AIFF (use `symphonia` crate for decoding)
- Converts to mono float32 at 44100 Hz
- Analyzes: waveform, RMS, peak, spectral centroid, duration
- Passes reference audio to ElevenLabs API (if supported) or uses it to condition generation

Frontend: `src/components/prompt/ReferenceDropZone.tsx`
- Drag-and-drop zone (accepts .wav, .mp3, .flac, .aiff)
- Click to open file picker
- Shows file name, duration, sample rate after drop
- Waveform preview of reference
- "Remove" button to clear
- Visual states: empty, dragging (highlight), loaded (show waveform + metadata)

Update PromptBar to show ReferenceDropZone when a toggle is clicked.

Dependencies:
- `symphonia` crate for audio decoding
- `rfd` crate for native file dialog (optional, use Tauri dialog)

Acceptance criteria:
- Drop a WAV file → waveform + metadata displayed
- Reference is decoded to float32 correctly
- Generate with reference → variations are clearly related to but different from reference
- Multiple audio formats supported (WAV, MP3, FLAC, AIFF)
- Error shown for unsupported formats or corrupt files
```

### Prompt 10: Build Multi-Format Export System

```
Goal: Create export system supporting WAV, AIFF, FLAC, and MP3.

Backend: `src-tauri/src/export/export_service.rs`

Export command:
```rust
#[tauri::command]
async fn export_sound(
    state: State<'_, AppState>,
    sound_id: String,
    format: String,        // "wav", "aiff", "flac", "mp3"
    bit_depth: u16,        // 16, 24, 32
    sample_rate: u32,      // 44100, 48000, 96000
    normalize: bool,        // true
    fade_in_ms: u32,        // 5
    fade_out_ms: u32,       // 10
    output_path: String,    // user-chosen directory
    filename: String,       // auto-generated or custom
) -> Result<ExportResult, String>
```

Use `hound` for WAV and AIFF, `claxon` for FLAC, and `lame` via Command or `minimp3-rs` crate for MP3.

Semantic filename generation:
- Parse prompt for key words: "kick", "snare", "808", "hi-hat", "clap", "percussion"
- Extract key if mentioned: "tuned to G" → "_G"
- Extract descriptors: "punchy", "tight", "round" → up to 2
- Format: `cShot_{type}_{descriptor1}_{descriptor2}_{key}.{ext}`
- Sanitize: lowercase, replace spaces/special chars with underscores, max 64 chars

Frontend: `src/components/export/ExportDialog.tsx`
- Modal dialog
- Format selection: WAV, AIFF, FLAC, MP3 (radio buttons)
- Bit depth: 16, 24, 32-float (dropdown, hide disabled options per format)
- Sample rate: 44.1k, 48k, 96k
- Normalize toggle (default on)
- Fade in/out toggles (default on)
- Output path with Browse button (Tauri dialog)
- Generated filename (editable)
- File size estimate
- Export button with progress
- Success state: path, file size, "Open in Finder" button

Update `src/hooks/useExport.ts`:
```typescript
export function useExport() {
    // open dialog, configure options, call invoke('export_sound')
    // track progress, return result
}
```

Acceptance criteria:
- Export WAV 24-bit 44.1kHz → opens correctly in Ableton, FL Studio, Logic Pro
- All formats produce valid files (verify header correctness)
- Filename is semantic and sanitized
- Export dialog remembers last-used settings
- Progress indicator shows during export
- Success notification with file path and "Reveal in Finder/Explorer"
```

### Prompt 11: Build Pack System

```
Goal: Create pack management system for organizing sounds into kits.

Backend - Add to `database.rs`:

New tables:
```sql
CREATE TABLE IF NOT EXISTS packs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS pack_sounds (
    pack_id TEXT NOT NULL REFERENCES packs(id) ON DELETE CASCADE,
    sound_id TEXT NOT NULL REFERENCES sounds(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (pack_id, sound_id)
);
```

New commands:
```rust
#[tauri::command]
async fn create_pack(state: State<'_, AppState>, name: String) -> Result<Pack, String>

#[tauri::command]
async fn get_packs(state: State<'_, AppState>) -> Result<Vec<Pack>, String>

#[tauri::command]
async fn add_sound_to_pack(state: State<'_, AppState>, pack_id: String, sound_id: String) -> Result<(), String>

#[tauri::command]
async fn remove_sound_from_pack(state: State<'_, AppState>, pack_id: String, sound_id: String) -> Result<(), String>

#[tauri::command]
async fn delete_pack(state: State<'_, AppState>, pack_id: String) -> Result<(), String>

#[tauri::command]
async fn export_pack(state: State<'_, AppState>, pack_id: String, format: ExportOptions) -> Result<Vec<ExportResult>, String>
```

Frontend components:
- `src/components/library/PackList.tsx` - sidebar showing packs with sound count
- `src/components/library/PackDetail.tsx` - sounds in pack with play/reorder/remove
- Add "Add to Pack" button in SoundSlot context menu and DetailPanel

Acceptance criteria:
- Create pack → appears in sidebar
- Add sound to pack → appears in pack detail view
- Pack shows all sounds with waveform thumbnails
- Remove sound from pack → sound still exists in library
- Delete pack → sounds remain in library (cascade)
- Export pack → all sounds exported to a folder
```

### Prompt 12: Implement SoundScore Model

```
Goal: Create lightweight ONNX model for quality scoring and integrate into pipeline.

This prompt requires Python for model training, then exports to ONNX for Rust inference.

Step 1: Train SoundScore model (Python script, saved to `research/soundscore/train.py`):
- Use lightweight CNN on mel-spectrogram features
- Input: 128×128 mel-spectrogram (1 second at 44.1kHz)
- Output: 5 scores (punch, body, clarity, uniqueness, overall) each 0-100
- Train on 2000 human-rated one-shots
- Loss: MSE on each dimension
- Model: 3 conv layers → global avg pooling → 2 dense layers → 5 outputs
- Export to ONNX with opset 18, optimize for CPU inference

Step 2: ONNX inference in Rust:
Create `src-tauri/src/analysis/soundscore.rs`:
```rust
pub struct SoundScoreModel {
    session: ort::Session,
}

impl SoundScoreModel {
    pub fn new(model_path: &Path) -> Result<Self>;
    pub fn score(&self, samples: &[f32], sample_rate: u32) -> Result<SoundScore>;
}
```

SoundScore struct:
```rust
pub struct SoundScore {
    pub punch: f32,       // transient impact, low-end punch
    pub body: f32,        // tonal body, fullness
    pub clarity: f32,     // definition, transient cleanliness
    pub uniqueness: f32,  // how different from typical sounds
    pub overall: f32,     // weighted combination
}
```

Integration in DSP pipeline: after processing, pass to SoundScore model.

Auto-regeneration: if overall score < 50, silently regenerate with different seed. Log the failure.

Dependencies:
- `ort` crate for ONNX Runtime inference
- `ndarray` for tensor handling

Acceptance criteria:
- SoundScore model runs inference in <10ms on CPU
- Scores are computed and persisted alongside sound metadata
- Low-scoring sounds (<50) are auto-regenerated
- Scores are returned to frontend for display
- `cargo test` verifies model loads and produces deterministic outputs
```

### Prompt 13: Build Settings & Keyboard Shortcuts

```
Goal: Add settings configuration and keyboard shortcuts.

Settings file: `src-tauri/src/config/settings.rs`
- Store in `~/.cshot/config.json`
- Default settings:
  - model: "elevenlabs"
  - output_path: "~/Music/cShot"
  - default_format: "wav"
  - default_bit_depth: 24
  - default_sample_rate: 44100
  - auto_normalize: true
  - theme: "dark"
  - key_bindings: { "generate": "Cmd+Enter", "export": "Cmd+E", "regenerate": "Cmd+R" }

Settings command:
```rust
#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String>
#[tauri::command]
async fn update_settings(state: State<'_, AppState>, settings: Settings) -> Result<(), String>
```

Frontend: `src/components/settings/SettingsView.tsx`
- General section: model selection (dropdown), output path (with Browse button)
- Export section: default format, bit depth, sample rate toggles
- Keyboard shortcuts reference
- Reset settings button

Keyboard shortcuts (global):
- `Cmd+Enter` or `Ctrl+Enter`: Generate (if prompt bar focused)
- `Cmd+E` or `Ctrl+E`: Export selected sound
- `Cmd+R` or `Ctrl+R`: Regenerate selected
- `Cmd+,` or `Ctrl+,`: Open settings
- `Cmd+L` or `Ctrl+L`: Focus library search
- `Cmd+N` or `Ctrl+N`: New generation
- `Cmd+1-6`: Select slot 1-6
- `Space`: Play/stop selected slot
- `Delete` or `Backspace`: Delete selected sound (with confirmation)

Acceptance criteria:
- Settings persist across restarts
- Changing model setting takes effect on next generation
- All keyboard shortcuts work
- Settings dialog shows correct current values
- Output path can be changed
```

### Prompt 14: Performance & Polish

```
Goal: Optimize performance and polish UI.

Performance tasks:
1. Parallel generation: generate all 6 slots concurrently (tokio::join! or futures::future::join_all)
2. Waveform caching: cache SVG paths to disk so regeneration doesn't recompute
3. Pre-warm model connections: keep HTTP keep-alive to ElevenLabs API
4. Lazy-load library: use pagination (20 items per page, load more on scroll)
5. Memory: release audio buffers when slot is deselected (store only hash, load on demand)
6. Cold start: defer non-critical initialization (library indexing, model loading) to background
7. Audio buffer management: keep last 3 played sounds in memory, drop older ones
8. Reduce WAV decode latency: store raw float32 alongside WAV for instant playback

Polish tasks:
1. Add micro-animations: slot entrance (scale from 0.95→1.0), waveform draw (left-to-right), score counter (count up)
2. Loading skeleton: shimmer placeholder for empty/loading slots
3. Empty states: helpful illustration + text for each view (no generations yet, no library results, no packs)
4. Toast notifications: generation complete, export complete, error, auto-regeneration
5. Tooltips: SoundScore meaning, format differences, keyboard shortcuts
6. SoundScore color coding: animated gradient on score badge
7. Scrollbar styling: thin, dark, matches theme
8. Window title: shows "cShot" + current state ("Generating...", "Exporting...")

Acceptance criteria:
- 6 concurrent generations complete in same time as 1 sequential
- Waveforms appear instantly (cached)
- App cold start < 2 seconds
- Library loads < 500ms for 1000 items (paginated)
- Memory usage stays under 500MB during extended use
- UI feels fluid at 60fps
- All empty/loading/error states are visually designed, not console.log
```

---

## Phase 3: Release (Prompts 15-18)

### Prompt 15: Error Handling & Edge Cases

```
Goal: Comprehensive error handling for every failure mode.

Create `src-tauri/src/error.rs`:
```rust
#[derive(Debug, thiserror::Error)]
pub enum CShotError {
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    #[error("Model {model} is not available")]
    ModelNotAvailable { model: String },
    #[error("Audio processing error: {0}")]
    AudioProcessing(String),
    #[error("Invalid audio format: {0}")]
    InvalidAudioFormat(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Rate limited. Try again in {seconds}s")]
    RateLimited { seconds: u64 },
}
```

Implement error handling for:
- Network errors during generation (timeout: retry once, then fallback)
- API authentication failure (show "Invalid API key" settings notification)
- Rate limiting (show "Too many generations, wait X seconds")
- Disk full during export (show error with space requirements)
- Corrupt database (auto-restore from backup, show notification)
- Missing model files (download prompt on first use)
- Empty prompt (refuse with "Please describe the sound")
- Generation produces silence (auto-regenerate, log)
- Generation produces clipping (auto-normalize, log)

Frontend: `src/components/shared/Toast.tsx`
- Success (green): generation complete, export complete
- Error (red): all error states mapped to user-friendly messages
- Warning (yellow): auto-fallback to local model, auto-regeneration
- Info (blue): generation queued, model switching

Error states for every component:
- Empty state: first use, no results
- Loading state: skeleton/spinner
- Error state: message + retry button
- Edge case: very long prompt (>200 chars) → truncate warning

Acceptance criteria:
- Every backend error returns a user-friendly message
- No unhandled panics reach the user
- Network failure → auto-fallback + notification
- Empty prompt → inline validation error
- Success/error/warning toasts display correctly
- Rate limiting shows countdown
```

### Prompt 16: First-Run Onboarding

```
Goal: Create a guided first-run experience that gets users to their first generation in <60 seconds.

Component: `src/components/onboarding/OnboardingFlow.tsx`

Flow (3 steps, 15 seconds each):
1. Welcome: "Stop browsing. Start making."
   - Illustration: waveform morphing into a sparkle
   - Text: "cShot generates unique, mix-ready one-shot samples from text prompts"
   - Button: "Let's make a sound →"

2. The Prompt: "Describe what you hear in your head"
   - Animated prompt bar with sample text typing: "Punchy 808 kick, round sub-bass tail, tuned to G"
   - Text: "Use natural language. Be specific. cShot understands producer vocabulary."
   - Button: "Try it →"

3. Your First Sound:
   - Pre-filled prompt: "Warm kick drum, soft attack, round body, mix-ready"
   - Generate button is highlighted and pulsing
   - Text: "Press Enter or click Generate to hear your first cShot sound"
   - Button: "Generate my first sound!" (triggers generation)

After generation completes:
- Show the 6 sound slots with a celebration animation
- "🎉 You just generated 6 unique kicks in 5 seconds."
- "That would have taken 45 minutes of browsing."
- Prompt to export or play next

Show onboarding only on first launch (check `config.json` for `has_onboarded: false`).

Acceptance criteria:
- Onboarding appears on first launch only
- Completing onboarding sets `has_onboarded: true`
- User can skip onboarding at any step
- First generation completes successfully
- Post-generation celebration shows
```

### Prompt 17: Beta Distribution Build

```
Goal: Configure builds for macOS and Windows beta distribution.

CI/CD: Create `.github/workflows/release.yml`

Jobs:
1. Build macOS (x86_64 + arm64 universal binary)
   - `npm run tauri build -- --target universal-apple-darwin`
   - Code signing: `APPLE_SIGNING_IDENTITY` env var
   - Notarization: `APPLE_NOTARY_KEY` env var
   - Output: `cShot_x.x.x_x64_x64.dmg`

2. Build Windows (x64)
   - `npm run tauri build -- --target x86_64-pc-windows-msvc`
   - Code signing: `WINDOWS_SIGNING_CERT` env var (or skip for beta)
   - Output: `cShot_x.x.x_x64-setup.exe` (or .msi)

3. Create GitHub release
   - Tag: `v{version}-beta.{n}`
   - Upload .dmg, .exe/.msi, and .tar.gz (Linux if available)
   - Generate release notes from commit log

Tauri updater configuration in `tauri.conf.json`:
```json
{
  "updater": {
    "active": true,
    "endpoints": ["https://releases.cshot.ai/updates/{{target}}-{{arch}}/{{current_version}}"],
    "dialog": true,
    "pubkey": "..."
  }
}
```

Error tracking: Integrate Sentry
- `src-tauri/src/main.rs`: `sentry::init("...")`
- Frontend: `@sentry/react` for React error boundary
- Capture: panics, unhandled errors, IPC failures

Analytics (opt-in):
- Backend: log to local SQLite `analytics` table
- Frontend: prompt user on first launch "Help us improve cShot?" with opt-in
- Events: generation, export, model used, latency, errors
- No personal data, no audio data, no prompt text

Acceptance criteria:
- `npm run tauri build` produces working DMG/EXE
- Beta build installs and runs on clean macOS/Windows
- Auto-updater checks for new versions
- Sentry captures if app crashes
- Analytics opt-in prompt works
- Install size < 15MB (compressed)
```

### Prompt 18: User Feedback System

```
Goal: Build in-app feedback collection.

Backend:
```rust
#[tauri::command]
async fn submit_feedback(
    state: State<'_, AppState>,
    rating: u8,              // 1-5
    feedback_text: String,   // optional, max 1000 chars
    category: String,        // "bug", "feature", "general"
) -> Result<(), String>
```

Frontend: `src/components/feedback/FeedbackWidget.tsx`
- Small floating button in status bar: "Feedback" 
- Opens modal with:
  - Rating selector (1-5 stars)
  - Category selector (Bug Report / Feature Request / General)
  - Text area (optional, max 1000 chars)
  - Submit button
  - Success confirmation

Also add feedback trigger points:
- After 3rd generation: "How's your first experience?" (small prompt)
- After first export: "Sound good? Rate this sound."
- After 10th generation: "Loving cShot? Share your feedback."
- After error: "Something went wrong. Help us fix it."

Acceptance criteria:
- Feedback form submits successfully
- Categories route correctly
- After-submit confirmation shown
- Trigger points appear at right intervals
- No duplicate triggers (don't ask twice in one session)
```

---

## Phase 4: Testing & Hardening (Prompts 19-22)

### Prompt 19: Rust Backend Tests

```
Goal: Comprehensive test coverage for all Rust backend code.

Write tests for:
1. DSP pipeline (`src-tauri/src/audio/dsp/`)
   - Each function with golden WAV files
   - Integration: full pipeline produces correctly formatted output
   - Edge cases: silence, DC offset, clipped, very short, very long, NaN

2. Database (`src-tauri/src/library/database.rs`)
   - CRUD for all tables
   - FTS search returns correct results
   - Index constraints (UNIQUE, NOT NULL)
   - Foreign key cascades
   - Concurrent read/write safety

3. Storage (`src-tauri/src/library/storage.rs`)
   - Write, read, hash verification
   - Dedup: same data returns same hash
   - Corrupt file detection
   - Directory creation

4. Export (`src-tauri/src/export/`)
   - WAV header correctness
   - All formats: open in test, verify properties
   - Bit depth conversion accuracy
   - Sample rate conversion
   - Filename sanitization

5. Model gateway (`src-tauri/src/generation/`)
   - Mock HTTP tests for ElevenLabs
   - Timeout handling
   - Error response parsing

Target: >85% code coverage

Use: `cargo test`, `cargo tarpaulin` for coverage, golden files in `tests/fixtures/`
```

### Prompt 20: Frontend Tests

```
Goal: Comprehensive test coverage for all React components and state.

Tools: Vitest + React Testing Library + Playwright

Test files mirror source structure:
```
src/__tests__/
├── components/
│   ├── prompt/PromptBar.test.tsx
│   ├── grid/SoundGrid.test.tsx
│   ├── grid/SoundSlot.test.tsx
│   ├── detail/DetailPanel.test.tsx
│   ├── library/LibraryView.test.tsx
│   ├── export/ExportDialog.test.tsx
│   └── shared/Toast.test.tsx
├── stores/
│   ├── useGenerationStore.test.ts
│   ├── useLibraryStore.test.ts
│   └── useAudioStore.test.ts
├── hooks/
│   ├── useAudioPlayback.test.ts
│   └── useGeneration.test.ts
└── integration/
    ├── generation-flow.test.ts
    └── export-flow.test.ts
```

Write tests for:
1. Component rendering: each component renders without crashing
2. Component interactions: button clicks, form inputs, drag-and-drop
3. State management: generation flow, library CRUD, playback state
4. Error states: empty state, loading state, error state display
5. Integration: full generation flow, export flow

Playwright e2e tests:
- Generation: type prompt → click generate → 6 slots appear → play a sound
- Library: generate → navigate to library → find sound → view details
- Export: generate → export → verify file on filesystem
- Reference: drop WAV → analysis shown → generate with reference

Target: >80% coverage on critical components, 100% on stores

Run: `npx vitest run` for unit, `npx playwright test` for e2e
```

### Prompt 21: Audio Quality & Regression Tests

```
Goal: Automated audio quality verification to prevent regressions.

Create `src-tauri/tests/audio_quality.rs`:

1. Golden sample tests:
   - Pre-generate 10 reference WAV files (known good kicks, snares, etc.)
   - Run DSP pipeline on each → compare output to golden reference
   - Difference must be below -80dB (i.e., deterministic processing)

2. Quality gate tests:
   - Generate 100 sounds through full pipeline
   - Each must pass: SoundScore > 30, no clipping, no silence, duration 100-1500ms
   - Track pass/fail ratio over time — alert on regression

3. Format compliance tests:
   - Export each format → open in reference decoder → verify:
     - WAV: correct RIFF header, fmt chunk, data chunk, no extra chunks
     - AIFF: correct FORM header, COMM chunk, SSND chunk
     - FLAC: correct STREAMINFO, valid compression
     - MP3: valid frames, correct bitrate

4. Latency regression tests:
   - Track generation latency per model per build
   - Alert if P50 latency increases by >20%

Run as separate: `cargo test --test audio_quality`
```

### Prompt 22: Security & Privacy Audit

```
Goal: Security review and privacy verification.

Create security checklist:

1. Data handling:
   - [ ] No audio data sent to analytics
   - [ ] No prompt text sent to third parties (only to model API)
   - [ ] API keys stored in OS keychain (not plaintext config file)
   - [ ] Local mode fully functional with no network calls
   - [ ] Opt-in consent for all data collection

2. File system:
   - [ ] cShot files contained within `~/.cshot/`
   - [ ] No writes outside `~/.cshot/` except user-chosen export path
   - [ ] Export path validated (no path traversal)
   - [ ] Temporary files cleaned up on exit

3. Network:
   - [ ] All API calls over HTTPS only
   - [ ] Certificate validation enabled
   - [ ] Timeouts on all HTTP requests (15s default)
   - [ ] No background network activity without user action

4. Model safety:
   - [ ] Generated audio checked for memorization (nearest neighbor in training set)
   - [ ] Output watermark embedded for provenance
   - [ ] Training data license audit documented

5. Supply chain:
   - [ ] All Rust crate dependencies vetted (no suspicious crates)
   - [ ] All npm dependencies vetted
   - [ ] `npm audit` passes with 0 critical vulnerabilities
   - [ ] `cargo audit` passes with 0 vulnerabilities

Run: `npm audit`, `cargo audit`, manual security review
```

## End-to-End Build Sequence

```
Phase 1 (Weeks 1-4): Foundation
  Prompt 1: Tauri + React scaffold
  Prompt 2: Rust DSP library
  Prompt 3: Content-addressed storage
  Prompt 4: SQLite database
  Prompt 5: ElevenLabs API client
  Prompt 6: Wire generation (first E2E flow)
  Prompt 7: SoundGrid UI + playback
  Prompt 8: Library + detail panel

Phase 2 (Weeks 5-8): Power Features
  Prompt 9: Reference upload + analysis
  Prompt 10: Multi-format export
  Prompt 11: Pack system
  Prompt 12: SoundScore model
  Prompt 13: Settings + keyboard shortcuts
  Prompt 14: Performance + polish

Phase 3 (Weeks 9-10): Release Readiness
  Prompt 15: Error handling + edge cases
  Prompt 16: First-run onboarding
  Prompt 17: Beta distribution build
  Prompt 18: User feedback system

Phase 4 (Weeks 11-12): Hardening
  Prompt 19: Rust backend tests
  Prompt 20: Frontend tests
  Prompt 21: Audio quality tests
  Prompt 22: Security + privacy audit
```
