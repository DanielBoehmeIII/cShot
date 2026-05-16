# Prompt 81 — Make cShot Work Locally

A local-first architecture for cShot. Every feature must function offline. Cloud is optional enhancement, never a dependency.

---

## 1. Local Database

### Engine Choice: SQLite via `rusqlite` (Rust) + `sql.js` (Web fallback)

SQLite is the correct choice for a local-first desktop audio tool. It requires no server process, has zero configuration, supports concurrent reads, and has battle-tested durability.

### Schema

```sql
-- Core entity: every sound generated or imported
CREATE TABLE sounds (
  id          TEXT PRIMARY KEY,          -- UUIDv7 (time-sortable)
  hash        TEXT NOT NULL UNIQUE,      -- SHA-256 of raw audio data
  title       TEXT NOT NULL,
  description TEXT,                      -- user notes
  duration_ms INTEGER NOT NULL,
  sample_rate INTEGER NOT NULL DEFAULT 44100,
  channels    INTEGER NOT NULL DEFAULT 1,
  bit_depth   INTEGER NOT NULL DEFAULT 16,
  file_size   INTEGER NOT NULL,          -- bytes
  format      TEXT NOT NULL DEFAULT 'wav',
  created_at  TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
  source      TEXT NOT NULL,             -- 'generation' | 'import' | 'recording'
  deleted     INTEGER NOT NULL DEFAULT 0 -- soft delete
);

-- Generation metadata (for generated sounds)
CREATE TABLE generations (
  sound_id       TEXT PRIMARY KEY REFERENCES sounds(id),
  prompt         TEXT NOT NULL,
  model_id       TEXT NOT NULL,
  model_version  TEXT,
  seed           INTEGER,
  duration_target INTEGER,              -- requested duration in ms
  params_json    TEXT,                   -- JSON blob of generation parameters
  inference_ms   INTEGER,               -- generation time
  cloud          INTEGER NOT NULL DEFAULT 0  -- 0=local, 1=cloud
);

-- Import metadata (for imported samples)
CREATE TABLE imports (
  sound_id      TEXT PRIMARY KEY REFERENCES sounds(id),
  original_path TEXT,
  original_name TEXT,
  import_batch  TEXT,                    -- batch ID for grouped imports
  tags_auto     TEXT,                    -- JSON: auto-detected tags
  tags_user     TEXT                     -- JSON: user-assigned tags
);

-- Tags (normalized)
CREATE TABLE tags (
  id    INTEGER PRIMARY KEY AUTOINCREMENT,
  name  TEXT NOT NULL UNIQUE COLLATE NOCASE
);

CREATE TABLE sound_tags (
  sound_id TEXT NOT NULL REFERENCES sounds(id),
  tag_id   INTEGER NOT NULL REFERENCES tags(id),
  source   TEXT NOT NULL DEFAULT 'user', -- 'user' | 'auto' | 'model'
  PRIMARY KEY (sound_id, tag_id)
);

-- Collections (packs, folders, playlists)
CREATE TABLE collections (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  description TEXT,
  cover_sound_id TEXT REFERENCES sounds(id),
  created_at  TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE collection_sounds (
  collection_id TEXT NOT NULL REFERENCES collections(id),
  sound_id      TEXT NOT NULL REFERENCES sounds(id),
  position      INTEGER NOT NULL DEFAULT 0,
  notes         TEXT,
  PRIMARY KEY (collection_id, sound_id)
);

-- User taste/preference data (privacy-preserving)
CREATE TABLE taste_events (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  sound_id    TEXT NOT NULL REFERENCES sounds(id),
  event_type  TEXT NOT NULL,             -- 'save' | 'export' | 'delete' | 'replay' | 'regenerate' | 'rate'
  rating      INTEGER,                   -- 1-5, NULL for non-rating events
  context     TEXT,                      -- JSON: what project/context at time of event
  created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Usage analytics (local-only, never phones home)
CREATE TABLE usage_log (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  action      TEXT NOT NULL,
  detail      TEXT,                      -- JSON
  duration_ms INTEGER,
  created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sync state (for future cloud sync)
CREATE TABLE sync_state (
  id           TEXT PRIMARY KEY,
  entity_type  TEXT NOT NULL,            -- 'sound' | 'collection' | 'setting'
  entity_id    TEXT NOT NULL,
  local_version INTEGER NOT NULL DEFAULT 1,
  sync_version  INTEGER DEFAULT NULL,
  sync_status   TEXT NOT NULL DEFAULT 'pending', -- 'pending' | 'synced' | 'conflict'
  last_sync    TEXT,
  checksum     TEXT
);
```

### Indexing Strategy

```sql
-- Primary lookups
CREATE INDEX idx_sounds_hash ON sounds(hash);
CREATE INDEX idx_sounds_source ON sounds(source);
CREATE INDEX idx_sounds_created ON sounds(created_at);
CREATE INDEX idx_sounds_deleted ON sounds(deleted) WHERE deleted = 0;

-- Search optimization
CREATE INDEX idx_generations_prompt ON generations(prompt);
CREATE INDEX idx_taste_events_sound ON taste_events(sound_id);
CREATE INDEX idx_taste_events_type ON taste_events(event_type);

-- Collection ordering
CREATE INDEX idx_collection_sounds_pos ON collection_sounds(collection_id, position);
```

### Key Design Decisions

- **UUIDv7**: time-sortable primary keys — natural ordering, no auto-increment issues
- **Content-addressed dedup**: SHA-256 hash ensures same sound never stored twice
- **Soft delete**: trash/recovery without data loss
- **JSON blobs**: for extensible metadata without schema migrations
- **Event sourcing for taste**: full history preservation, enables recomputation of taste embeddings

---

## 2. File Storage Layout

### Root: `~/.cshot/` (Linux/macOS) or `%APPDATA%/cShot/` (Windows)

```
~/.cshot/
├── cshot.db                    # SQLite database (all metadata)
├── cshot.db-wal                # WAL journal
├── cshot.db-shm                # Shared memory (WAL)
│
├── library/                    # All sound files
│   ├── content/                # Content-addressed storage
│   │   ├── ab/                 # First two hex chars of hash
│   │   │   ├── ab3f...91.wav   # File named by hash
│   │   │   └── ...
│   │   ├── cd/
│   │   └── ...
│   ├── imports/                # Original imported files (symlinks or copies)
│   │   └── {batch-id}/
│   │       ├── original_name_1.wav
│   │       └── ...
│   └── exports/                # User exports (organized by date)
│       └── 2026-05-15/
│           ├── My Kick_01.wav
│           └── ...
│
├── cache/
│   ├── waveforms/              # Pre-rendered waveform PNGs/SVGs
│   │   ├── ab/ab3f...91.svg
│   │   └── ...
│   ├── spectrograms/           # Pre-rendered spectrogram images
│   │   └── ...
│   ├── embeddings/             # Cached audio embeddings (ONNX output)
│   │   └── ...
│   └── thumbnails/             # Smaller waveform preview thumbnails
│       └── ...
│
├── models/                     # Local model files
│   ├── quantized/              # Quantized ONNX models
│   │   ├── generator_v1.onnx
│   │   └── embedding_v1.onnx
│   ├── lora/                   # LoRA adapter weights
│   │   └── ...
│   └── manifests/              # Model metadata / version info
│       └── registry.json
│
├── config/                     # User configuration
│   ├── settings.json           # App settings
│   ├── keybindings.json        # Custom keyboard shortcuts
│   ├── recipes/                # User-created sound recipes
│   │   └── ...
│   └── themes/                 # UI themes
│       └── ...
│
├── exports/                    # Export presets
│   └── presets.json
│
├── temp/                       # Temporary files (cleared on exit)
│   ├── render/                 # In-progress render output
│   └── download/               # Cloud generation downloads
│
├── sync/                       # Sync state (when cloud is enabled)
│   ├── manifest.json           # Local sync manifest
│   └── journal/                # Sync operation journal
│       └── ...
│
└── logs/
    ├── app.log                 # Application log
    ├── generation.log          # Generation attempts + results
    └── sync.log                # Sync operations log
```

### Content-Addressed Storage Design

```
Hash: SHA-256(48 bytes) → 64 hex chars
Storage path: ~/.cshot/library/content/{first 2 chars}/{full hash}.wav

Example:
  Hash:    ab3f7c...e91
  Path:    ~/.cshot/library/content/ab/ab3f7c...e91.wav
```

Benefits:
- Automatic deduplication at filesystem level
- Path-based integrity verification (filename IS the checksum)
- Directory sharding prevents millions of files in single directory
- No filename collisions — ever

### Waveform Cache

- Rendered at multiple zoom levels: full, 1s, 100ms
- SVG format for crisp rendering at any scale
- Progressive loading: render thumbnail first, then full resolution
- Invalidated on sound modification, re-rendered on next access
- Compressed with brotli for storage efficiency

---

## 3. Desktop App Architecture

### Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    UI LAYER (React + Vite)               │
│                                                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │  Prompt  │ │  Grid/   │ │  Waveform│ │  Settings│  │
│  │  Bar     │ │  List    │ │  Detail  │ │  Panel   │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│       │            │            │            │          │
│  ┌────┴────────────┴────────────┴────────────┴─────┐   │
│  │           Zustand Stores                         │   │
│  │  (generation | library | player | settings)      │   │
│  └──────────────────────┬──────────────────────────┘   │
│                         │ IPC (invoke)                  │
├─────────────────────────┼───────────────────────────────┤
│                    RUST LAYER (Tauri Commands)          │
│                         │                               │
│  ┌──────────────────────┴──────────────────────────┐   │
│  │              Command Handlers                     │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐          │   │
│  │  │ generate │ │ library  │ │ export   │          │   │
│  │  │_sound    │ │_search   │ │_sound    │          │   │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘          │   │
│  │  ┌────┴─────┐ ┌────┴─────┐ ┌────┴─────┐          │   │
│  │  │ analysis │ │  db/     │ │ audio/   │          │   │
│  │  │_pipeline │ │ storage  │ │_export   │          │   │
│  │  └──────────┘ └──────────┘ └──────────┘          │   │
│  └───────────────────────────────────────────────────┘   │
│                         │                               │
├─────────────────────────┼───────────────────────────────┤
│                  SERVICE LAYER (Rust Modules)            │
│                         │                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │  Audio   │ │  Model   │ │   DB     │ │  Sync    │  │
│  │  Engine  │ │  Runtime │ │  Layer   │ │  Engine  │  │
│  │ (DSP +   │ │ (ONNX    │ │(SQLite   │ │ (CRDT    │  │
│  │  Playback)│ │ 推理)   │ │ migrations)│ │  sync)   │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │         Filesystem Abstraction Layer              │   │
│  │  (content-addressed store, cache manager, temp)   │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Module Map (Rust)

```
cshot-core/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API, re-exports
│   │
│   ├── db/
│   │   ├── mod.rs                # Database trait + factory
│   │   ├── migrations.rs         # SQLite schema migrations
│   │   ├── models.rs             # Rust structs mirroring schema
│   │   ├── sounds.rs             # Sound CRUD operations
│   │   ├── tags.rs               # Tag management
│   │   ├── collections.rs        # Collection (pack) management
│   │   ├── search.rs             # Full-text + tag search
│   │   ├── taste.rs              # Taste event recording/querying
│   │   └── sync.rs               # Sync state tracking
│   │
│   ├── storage/
│   │   ├── mod.rs                # Storage trait
│   │   ├── content_store.rs      # Content-addressed file store
│   │   ├── cache.rs              # Waveform/embedding/spectrogram cache
│   │   ├── imports.rs            # Import handling (copy, hash, organize)
│   │   ├── exports.rs            # Export with format conversion
│   │   └── temp.rs               # Temp file lifecycle management
│   │
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── decode.rs             # WAV/MP3/FLAC/Ogg decoding (Symphonia)
│   │   ├── encode.rs             # WAV export encoding (hound)
│   │   ├── analyze.rs            # RMS, peak, LUFS, spectral centroid, zero-crossing
│   │   ├── waveform.rs           # Waveform data extraction for UI
│   │   ├── spectrogram.rs        # STFT spectrogram generation
│   │   ├── dsp/
│   │   │   ├── mod.rs
│   │   │   ├── normalize.rs      # Peak + loudness normalization (EBU R128)
│   │   │   ├── trim.rs           # Silence trimming
│   │   │   ├── fade.rs           # Fade in/out
│   │   │   ├── resample.rs       # Sample rate conversion
│   │   │   └── analyze_loudness.rs # LUFS measurement
│   │   └── playback.rs           # Real-time playback via cpal
│   │
│   ├── model/
│   │   ├── mod.rs                # Model trait (local vs remote)
│   │   ├── local.rs              # ONNX Runtime inference
│   │   ├── remote.rs             # Cloud API client
│   │   ├── embedding.rs          # Audio embedding extraction
│   │   ├── quantized.rs          # Quantized model loader
│   │   └── types.rs              # Model input/output types
│   │
│   ├── processing/
│   │   ├── mod.rs
│   │   ├── pipeline.rs           # Audio processing pipeline orchestrator
│   │   ├── chain.rs              # DSP chain (apply effects in sequence)
│   │   └── jobs.rs               # Background job queue (CPU-intensive work)
│   │
│   ├── sync/
│   │   ├── mod.rs                # Sync trait
│   │   ├── engine.rs             # CRDT-based sync engine
│   │   ├── manifest.rs           # Local/remote manifest comparison
│   │   ├── transfer.rs           # File transfer + resume
│   │   └── conflict.rs           # Conflict resolution strategies
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   ├── settings.rs           # Settings loading/saving
│   │   └── defaults.rs           # Default configuration values
│   │
│   └── util/
│       ├── hash.rs               # SHA-256 hashing
│       ├── id.rs                 # UUIDv7 generation
│       └── time.rs               # Timestamp utilities
```

### State Management (Zustand)

```typescript
// Store structure
interface GenerationStore {
  currentPrompt: string;
  history: GenerationRecord[];
  isGenerating: boolean;
  progress: number;          // 0-100
  localOnly: boolean;        // force local-only mode

  setPrompt: (p: string) => void;
  generate: () => Promise<Sound>;
  regenerate: (soundId: string, modifications: Partial<GenParams>) => Promise<Sound>;
  cancelGeneration: () => void;
}

interface LibraryStore {
  sounds: Sound[];
  collections: Collection[];
  searchQuery: string;
  activeFilters: Filter[];
  sortOrder: SortOrder;
  viewMode: 'grid' | 'list';

  search: (query: string) => void;
  filterByTags: (tags: string[]) => void;
  addToCollection: (soundId: string, collectionId: string) => void;
  deleteSound: (soundId: string) => void;
}

interface PlayerStore {
  currentSound: Sound | null;
  isPlaying: boolean;
  position: number;
  volume: number;
  loop: boolean;
  playbackMode: 'once' | 'loop' | 'section';

  play: (sound: Sound) => void;
  stop: () => void;
  seek: (position: number) => void;
  setVolume: (v: number) => void;
}

interface SettingsStore {
  databasePath: string;
  libraryPath: string;
  audioDevice: string;
  sampleRate: number;
  localInference: boolean;
  cloudEnabled: boolean;
  cloudEndpoint: string;
  syncEnabled: boolean;
  modelPreferences: ModelConfig[];

  updateSettings: (partial: Partial<Settings>) => void;
  resetToDefaults: () => void;
}
```

---

## 4. Sync Model

### CRDT-Based Sync (when cloud is enabled)

cShot uses a **merge-after-write** model with Conflict-Free Replicated Data Types (CRDTs) for metadata, and content-addressed deduplication for audio files.

```
Sync Architecture:

  ┌─────────────────────┐        ┌─────────────────────┐
  │  Device A (laptop)  │        │  Device B (desktop) │
  │                     │        │                     │
  │  ┌───────────────┐  │        │  ┌───────────────┐  │
  │  │ Local DB      │  │        │  │ Local DB      │  │
  │  │ (SQLite)      │  │        │  │ (SQLite)      │  │
  │  └───────┬───────┘  │        │  └───────┬───────┘  │
  │          │          │        │          │          │
  │  ┌───────▼───────┐  │        │  ┌───────▼───────┐  │
  │  │ Sync Journal  │  │        │  │ Sync Journal  │  │
  │  │ (local ops)   │  │        │  │ (local ops)   │  │
  │  └───────┬───────┘  │        │  └───────┬───────┘  │
  └──────────┼──────────┘        └──────────┼──────────┘
             │                              │
             │          ┌─────────┐          │
             ├──────────►  Cloud  ◄──────────┤
             │          │ Server  │          │
             │          └─────────┘          │
             │           (optional)          │
             │                              │
             │  ┌──────────────────────┐    │
             └──┤ Peer-to-Peer (LAN)   ├────┘
                └──────────────────────┘
```

### Sync Protocol

1. **Operation Log**: Each mutation creates an entry in `sync_state` with a Lamport timestamp (device_id + counter)
2. **Manifest Exchange**: Devices exchange manifests listing their latest version for each entity
3. **Pull Missing**: Each device requests entities where remote version > local version
4. **Merge**: CRDT merge rules resolve conflicts automatically (last-writer-wins for scalar fields, set-union for tags)
5. **Content Transfer**: Audio files transferred via content hash — if remote already has hash, skip transfer
6. **Acknowledgement**: Both sides update sync state, log completion

### Conflict Resolution Rules

| Entity | Conflict Type | Strategy |
|--------|--------------|----------|
| Sound metadata | Field-level | Last-writer-wins per field |
| Tags | Set membership | Union (both tags survive) |
| Collections | Item ordering | Last-writer-wins for order |
| Deletion | Tombstone | Deletion wins (logical delete) |
| Generation params | Full replace | Last-writer-wins |
| Taste events | Append-only | All events preserved, no conflict |
| Settings | Full replace | Manual merge dialog |

### Offline-First Guarantees

- All operations succeed without network
- Sync is eventual — no consistency requirements during offline operation
- No blocking operations waiting for sync
- Conflicts are rare and automatically resolved
- User is notified of sync status but never blocked by it

---

## 5. Cloud/Local Boundary

### Strict Boundary Definition

```
┌─────────────────────────────────────────────────────────┐
│                    LOCAL (Always Available)               │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ✓ Local database (SQLite) — all metadata                │
│  ✓ Content-addressed file storage — all audio files      │
│  ✓ Audio analysis — waveform, spectrogram, loudness      │
│  ✓ Local search — full-text, tags, metadata              │
│  ✓ Waveform preview — cached SVG rendering               │
│  ✓ DSP processing — normalize, trim, fade, resample      │
│  ✓ Export — WAV, FLAC, MP3, Ogg                          │
│  ✓ Playback — all local sounds                           │
│  ✓ Collection management — create, edit, organize        │
│  ✓ Taste tracking — all preference events                │
│  ✓ Settings — all app configuration                      │
│  ✓ Undo/redo — full history locally                      │
│  ✓ Batch operations — tag, move, export                  │
│  ✓ Backup / restore — full local snapshot                │
│                                                          │
├─────────────────────────────────────────────────────────┤
│                 CLOUD (Optional Enhancement)              │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ~ Cloud generation (high-quality models)                │
│  ~ Cloud embedding (larger, more accurate models)        │
│  ~ Cross-device sync (library + settings)                │
│  ~ Cloud backup (off-site redundancy)                    │
│  ~ Community packs (shared collections)                  │
│  ~ Collaborative editing (multi-user collections)        │
│  ~ Advanced search (semantic search via cloud embedding) │
│  ~ Model download (large model files on first use)       │
│                                                          │
├─────────────────────────────────────────────────────────┤
│                   FUTURE LOCAL (Goal)                     │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  → Local generation (quantized ONNX models)              │
│  → Local embedding (small efficient models)              │
│  → Local preference model (on-device taste learning)     │
│  → Local semantic search (via local embeddings)          │
│  → Local genre classification (lightweight classifier)   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Architectural Principle: Cloud as Accelerator, Not Gate

```
User Action → Check local capability → Can do locally? → YES → Execute locally
                                                          ↓ NO
                                              Is cloud enabled? → NO → Show "not available offline"
                                                                  ↓ YES
                                                        Execute on cloud → Return result
```

### Cloud API Design (Optional Module)

```rust
// Cloud service trait — implemented by remote module
#[async_trait]
trait CloudService {
    // Generation
    async fn generate_one_shot(&self, params: GenParams) -> Result<Sound>;
    async fn generate_batch(&self, params: Vec<GenParams>) -> Result<Vec<Sound>>;

    // Embedding
    async fn compute_embedding(&self, audio_data: &[u8]) -> Result<Vec<f32>>;
    async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;

    // Sync
    async fn push_manifest(&self, manifest: SyncManifest) -> Result<()>;
    async fn pull_changes(&self, since_version: u64) -> Result<SyncDelta>;

    // Models
    async fn available_models(&self) -> Result<Vec<ModelInfo>>;
    async fn download_model(&self, model_id: &str) -> Result<PathBuf>; // to local cache
}
```

### Offline Degradation Path

| Feature | Online | Offline |
|---------|--------|---------|
| Generate sound | Cloud or local model | Local model only (or DSP presets) |
| Search library | Semantic + tag + text | Tag + text only |
| Waveform preview | Full quality | Cached or computed locally |
| Embedding similarity | Full embedding model | Lightweight local embedding |
| Sync | All devices synced | Local only, sync when online |
| Model download | Available | Not available |
| Community packs | Browse and download | Local collections only |

---

## 6. Migration Path Toward Local Generation

### Phase 0: Cloud-Only Generation (MVP)
```
- All generation via cloud API (Stable Audio / AudioCraft)
- Local everything else (DB, storage, analysis, search, playback)
- No offline generation — generation button disabled without internet
- But: all existing sounds playable and searchable offline
```

### Phase 1: DSP Presets + Simple Local Generation
```
- Rule-based local generation for basic sounds:
  - Sine/sub kicks via oscillator + envelope
  - White noise hats via filtered noise
  - Basic claps via layered noise bursts
- These are low quality but functional offline
- Users can trigger them without internet
- Cloud still used for complex/specific sounds
```

### Phase 2: Quantized ONNX Model
```
- Download quantized model (first-time setup, ~200MB)
- ONNX Runtime inference on CPU
- Supports: full model → ONNX → INT8 quantization → local
- Limitation: slower (5-15s), lower quality, specific model
- Stored at: ~/.cshot/models/quantized/generator_v1.onnx
- Fallback: if model not downloaded, show download prompt
```

### Phase 3: Multi-Model Local Runtime
```
- Support for multiple local models:
  - Tiny model (fast, ~50MB, <2s generation)
  - Standard model (balanced, ~200MB, <5s generation)
  - Quality model (slow, ~500MB, <15s generation)
- Automatic model selection based on:
  - Available RAM
  - CPU cores/performance
  - Battery status (skip quality model on battery)
  - User preference (speed vs quality slider)
- Model warmup cache for common prompts
```

### Phase 4: Progressive Generation
```
- Start generating with tiny model (200ms → rough result)
- Refine with medium model while tiny result previews (2s)
- Final pass with quality model (8s → final)
- User hears progressively better versions
- Can stop early if tiny result is "good enough"
- This is the "feels fast" architecture
```

### Phase 5: Local First, Cloud Optional
```
- Default: all generation is local
- Cloud used only for:
  - Higher quality than local models can achieve
  - Models not available locally
  - Cross-device sync
- Local models improve via:
  - ONNX optimization improvements
  - Better quantization techniques
  - Hardware acceleration (Apple M-series ANE, NVIDIA CUDA)
- cShot becomes truly local-first
```

### Quantization Roadmap

```
FP32 model (baseline)     → 100% quality, 100% size
  ↓
FP16 model                → 99% quality, 50% size
  ↓
INT8 (dynamic)            → 95% quality, 25% size
  ↓
INT8 (static)             → 93% quality, 25% size (faster inference)
  ↓
INT4 + GPTQ/AWQ           → 88% quality, 12% size
  ↓
Distilled student model   → 85% quality, 5% size (trained from scratch for small size)
```

### Hardware Acceleration Targets

| Platform | Acceleration | Library |
|----------|-------------|---------|
| Apple Silicon (M1+) | ANE + Metal | CoreML / ANE via `coreml-rs` |
| NVIDIA GPU | CUDA | ONNX Runtime CUDA provider |
| AMD GPU | ROCm | ONNX Runtime ROCm provider |
| Intel GPU / CPU | OpenVINO | ONNX Runtime OpenVINO provider |
| CPU (fallback) | x86-64 SIMD | ONNX Runtime CPU provider (mklml/dnnl) |

### Local Generation API (Rust)

```rust
// Trait for local model inference
#[async_trait]
trait LocalGenerator {
    /// Check if model is loaded and ready
    fn is_ready(&self) -> bool;

    /// Load model into memory (called once at startup or on demand)
    async fn load(&mut self, model_path: &Path, config: ModelConfig) -> Result<()>;

    /// Generate a one-shot from prompt
    async fn generate(&self, params: GenParams) -> Result<GeneratedAudio>;

    /// Generate with progressive quality levels
    async fn generate_progressive(
        &self,
        params: GenParams,
        on_intermediate: Box<dyn Fn(GeneratedAudio, QualityLevel)>
    ) -> Result<GeneratedAudio>;

    /// Unload model to free RAM
    fn unload(&mut self);

    /// Memory usage in bytes
    fn memory_usage(&self) -> u64;
}

/// Quality levels for progressive generation
enum QualityLevel {
    Draft,      // Tiny model, ~200ms, 8kHz mono, rough
    Preview,    // Medium model, ~2s, 22kHz mono, usable
    Standard,   // Main model, ~5s, 44.1kHz mono, good
    High,       // Quality model, ~15s, 48kHz stereo, best
    Export,     // Full quality, ~30s, 48kHz stereo 32-bit, master
}
```

---

## Summary

cShot's local-first architecture is built on:
1. **SQLite** for all metadata — zero-config, offline-capable, fast
2. **Content-addressed storage** — automatic dedup, integrity, organization
3. **Three-layer Rust architecture** — UI (React) → Commands (Tauri) → Services (Rust)
4. **CRDT-based sync** — offline-first, conflict-free, cloud-optional
5. **Strict cloud/local boundary** — all core features work offline
6. **Phased migration toward local generation** — DSP presets → quantized ONNX → multi-model → progressive

Every design decision prioritizes offline capability. Cloud is always optional, never required.
