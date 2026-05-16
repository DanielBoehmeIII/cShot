# Prompt 97 — Lock the Year-One Architecture

## Year-One Architecture Decisions

### Decision Matrix

Every decision below follows three rules:
1. **Choose the simplest serious option** — it must work for production, but no over-engineering
2. **Explain what NOT to build yet** — avoid premature optimization and scope creep
3. **Define future upgrade path** — know how to evolve when the time comes

---

### 1. Frontend

**Decision:** React 18 + TypeScript + Vite + Tailwind CSS

**Why this:**
- Most mature web UI ecosystem — fast development, large talent pool
- Vite: instant HMR, tiny bundles, TypeScript-native
- Tailwind: zero runtime, easy theming, fast iteration
- Runs inside Tauri WebView for desktop app

**What NOT to build yet:**
- No Next.js/SSR (this is a desktop app, not a website)
- No state machine library (Zustand is sufficient until stores exceed 20)
- No i18n (English-only for year one)
- No component library (custom design system is small enough to hand-roll)
- No React Flow / graph UI (variation tree is Phase 2)
- No PWA / mobile (desktop-only for year one)

**Future upgrade path:**
- Add Zustand slices as store complexity grows → Zustand with immer + persist
- If UI becomes complex enough for storybook → add Storybook in Phase 2
- If design system grows large → extract to internal `@cshot/ui` package
- If graph features needed → add React Flow in Phase 2

---

### 2. Backend

**Decision:** Rust (Tauri's native backend) + Python (FastAPI for cloud services)

**Why this:**
- Rust in Tauri handles: file I/O, audio processing, IPC, database access, plugin bridge
- Python FastAPI handles: model inference gateway, async job queue, authentication
- Rust is the right tool for DSP + desktop integration
- Python is the right tool for ML inference + API services
- Separation keeps concerns clean: desktop logic in Rust, cloud logic in Python

**What NOT to build yet:**
- No GraphQL (REST + JSON is simpler, fewer dependencies)
- No gRPC (HTTP/1.1 is fine for year-one traffic volume)
- No WebSocket streaming (generation is request-response, not real-time)
- No microservices (monorepo with two services: Rust desktop + Python cloud)
- No Kubernetes (single Docker Compose deployment or bare-metal)
- No CDN (file downloads are small — WAV files are 200-500KB)

**Future upgrade path:**
- If traffic exceeds 100K requests/day → add nginx + load balancing
- If real-time features needed → add WebSocket support
- If model inference scales → separate model serving to dedicated GPU instances
- If team grows → split into well-defined service boundaries

---

### 3. Desktop/Web Direction

**Decision:** Tauri v2 desktop app (primary) + minimal web presence (marketing only)

**Why this:**
- Tauri: ~5MB binary vs. Electron's ~150MB — critical for download conversion
- Rust backend enables native audio processing, VST3 plugin bridge, low-level audio access
- Producers expect a desktop app — they distrust web-only audio tools
- Offline-capable: local inference, local storage, no cloud dependency
- Tauri v2 supports mobile (future path), but year one is macOS + Windows

**What NOT to build yet:**
- No web app (cshot.app is marketing + waitlist, not the product)
- No Linux build (year one is macOS + Windows; Linux in Phase 2)
- No auto-update (manual download for MVP; Tauri updater in Phase 2)
- No mobile app (Phase 3 at earliest)
- No PWA (producers don't use audio PWAs)

**Future upgrade path:**
- Add Tauri updater for auto-updates
- Add Linux support (Tauri supports Linux natively)
- If cloud-first features justify it → consider web app for library browsing + sharing
- If mobile sampling workflows emerge → Tauri mobile build

---

### 4. Model Gateway

**Decision:** Cloud API gateway (ElevenLabs SFX + Stable Audio Open) with local ONNX fallback

**Why this:**
- Cloud APIs give best quality immediately without GPU hardware requirements
- ElevenLabs SFX: best text-to-SFX quality, low latency (2-4s), commercial license
- Stable Audio Open: open weights, high-quality, can self-host
- Local ONNX fallback enables offline mode (fine-tuned AudioLDM 2, INT8 quantized)
- Gateway pattern abstracts model selection from the UI — swap models without UI changes

**Architecture:**
```
User Prompt → Text Encoder (local ONNX, CLAP-style, 768-d)
           → Gateway Router (chooses model based on: availability, latency, quality, user tier)
           → Model Adapter (normalizes API-specific parameters to cShot internal schema)
           → Generation API (ElevenLabs / Stable Audio / Local ONNX)
           → Raw Audio → Rust DSP Pipeline → QC Checks → User
```

**What NOT to build yet:**
- No custom diffusion model training (year one uses fine-tuned open models)
- No own inference serving infra (use cloud APIs + Replicate/Banana for self-hosted)
- No model ensemble (route to best model, don't combine)
- No A/B testing framework (manual comparison in dev)
- No streaming generation (generate full audio, then deliver)

**Future upgrade path:**
- Train custom one-shot diffusion model (Phase 2, after dataset grows)
- Self-host inference on dedicated GPU when query volume > 10K/day
- Add model ensemble for quality improvement (Phase 3)
- Fine-tune per-user models (Phase 2, requires taste embedding data)

---

### 5. DSP Engine

**Decision:** Rust native DSP library (custom, using `hound` + `symphonia` + NIH-plug)

**Why this:**
- Rust: zero-cost abstractions, no GC pauses, safe concurrency, easy FFI for VST3
- `hound`: WAV read/write — simple, well-maintained, pure Rust
- `symphonia`: audio decoding for reference uploads (MP3, FLAC, AAC, WAV)
- NIH-plug: VST3/AU plugin framework when we build the plugin
- Custom DSP routines: trim silence, normalize peak, fade in/out, spectral analysis

**DSP pipeline (per generation):**
1. Receive raw float32 buffer from model gateway
2. Auto-trim leading/trailing silence (< -60dB threshold)
3. Normalize peak to -1.0dB (headroom for mixing)
4. Fade in (5ms) and fade out (10ms tail) — click-free
5. Compute analytics: RMS, crest factor, spectral centroid, transient time
6. Compute SoundScore (ONNX model, ~5MB, runs locally)
7. Encode to target format (WAV 16/24-bit, AIFF, FLAC)
8. Content-address store (SHA-256 of audio data)
9. Return to UI for preview

**What NOT to build yet:**
- No real-time audio processing (DSP runs on generation, not on live audio)
- No VST3 plugin hosting (loading VSTs is Phase 3)
- No multi-band processing (single-band DSP is sufficient for one-shots)
- No convolution reverb / FX (Phase 2 for "add reverb" feature)
- No pitch detection → pitch shifting (Phase 2)
- No time-stretching (not needed for one-shots)

**Future upgrade path:**
- Add real-time preview streaming while generation completes
- Add VST3 hosting: chain generated sound through a VST3 effect
- Add multi-band compressor for mix-readiness enhancement
- Add pitch detection + pitch shifting for reference-based generation
- Add spectral morphing between two one-shots

---

### 6. Audio Storage

**Decision:** Content-addressed file system (SHA-256 hash as filename)

**Why this:**
- Deduplication: same generation produces the same file once
- Integrity verification: hash mismatch = corrupted file = regenerate
- Simple: no database needed for file retrieval, just hash lookups
- Portable: directory structure is self-describing
- No lock-in: standard filesystem, easily migrated, backed up, synced

**Storage layout:**
```
~/.cshot/
├── audio/
│   ├── ab/
│   │   ├── c3d4e5... (SHA-256 prefix directory)
│   │   └── ... (file: f1a2b3...wav)
│   └── ... (flat structure, 2-level hash prefix dirs)
├── metadata/        (SQLite database)
├── models/          (ONNX models, cached)
├── config.json      (user settings)
├── export/          (default export location)
└── cache/           (waveform thumbnails, temp files)
```

**What NOT to build yet:**
- No cloud storage sync (Phase 2 feature)
- No S3/object storage (filesystem works for year-one scale)
- No compression for storage (WAV is fine; 500KB per sound × 10,000 sounds = 5GB)
- No encrypted-at-rest (OS-level encryption is sufficient)
- No distributed storage (single-user desktop app)

**Future upgrade path:**
- Add S3 sync for cloud library (Phase 2)
- Add encrypted vault option (Phase 3 for commercial users)
- Add storage quotas per user tier
- Add compression + decompression on sync for bandwidth savings

---

### 7. Metadata Database

**Decision:** SQLite via `rusqlite` (Rust, synchronous, embedded)

**Why this:**
- Zero configuration: database file lives alongside audio storage
- Embedded: no separate server process, no connection management
- Reliable: SQLite is the most deployed database engine in the world
- Performant: easily handles 100K+ records for single-user desktop app
- `rusqlite`: mature Rust bindings, type-safe, well-documented
- Easy migration path to Postgres when cloud sync is needed

**Schema (MVP):**
```sql
CREATE TABLE sounds (
    id TEXT PRIMARY KEY,            -- UUID v4
    hash TEXT NOT NULL UNIQUE,       -- SHA-256 of audio data
    prompt TEXT NOT NULL,            -- generation prompt
    model TEXT NOT NULL,             -- model used (elevenlabs, stableaudio, local)
    seed INTEGER NOT NULL,           -- random seed for reproducibility
    created_at TEXT NOT NULL,        -- ISO 8601 timestamp
    
    -- analytics
    duration_ms INTEGER NOT NULL,
    sample_rate INTEGER NOT NULL,
    bit_depth INTEGER NOT NULL,
    channels INTEGER NOT NULL DEFAULT 1,
    rms REAL NOT NULL,
    peak REAL NOT NULL,
    crest_factor REAL NOT NULL,
    spectral_centroid REAL,
    transient_time_ms INTEGER,
    
    -- quality
    soundscore_punch REAL,
    soundscore_body REAL,
    soundscore_clarity REAL,
    soundscore_uniqueness REAL,
    soundscore_overall REAL,
    
    -- user metadata
    rating INTEGER,                  -- 1-5 stars (nullable)
    tags TEXT,                       -- JSON array of user tags
    pack_id TEXT REFERENCES packs(id),
    is_exported INTEGER DEFAULT 0,
    exported_at TEXT,
    
    -- provenance
    reference_hash TEXT,             -- SHA-256 of reference audio if used
    parent_id TEXT REFERENCES sounds(id)
);

CREATE TABLE packs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE generation_log (
    id TEXT PRIMARY KEY,
    prompt TEXT NOT NULL,
    model TEXT NOT NULL,
    latency_ms INTEGER NOT NULL,
    success INTEGER NOT NULL,
    error TEXT,
    created_at TEXT NOT NULL
);
```

**What NOT to build yet:**
- No Postgres (overkill for single-user desktop + adds DevOps burden)
- No migrations framework (version the schema, `ALTER TABLE` on update)
- No full-text search (SQLite FTS5 is a future addition; basic LIKE is fine for MVP)
- No replication / backup service (`cp ~/.cshot/metadata.db` is the backup)
- No connection pooling (single connection, single writer)
- No ORM (raw SQL via `rusqlite` is cleaner for this schema size)

**Future upgrade path:**
- Add Postgres for cloud sync (Phase 2) — by then schema is stable
- Add FTS5 for full-text search across prompts and tags
- Add migration scripts for schema evolution
- Add backup-to-cloud feature

---

### 8. Vector Search

**Decision:** FAISS (local, flat index, cosine similarity) + pgvector (cloud, Phase 2)

**Why this:**
- FAISS is the industry standard for vector search, has Rust bindings (`faiss-rs`)
- Local: no network latency, works offline, no privacy concerns
- Flat index: brute-force search is fast enough for <100K vectors (sub-millisecond)
- Search use case: "find sounds similar to this one" via UShOt embedding
- Phase 1 only needs similarity search within the user's own library

**What NOT to build yet:**
- No pgvector (cloud dependency, not needed until cloud sync arrives)
- No approximate search (IVF, HNSW — flat index is fast enough for MVP scale)
- No sharding (single index, local, <100K vectors)
- No embedding server (compute embeddings in the DSP pipeline, index on write)
- No multi-modal search (text → audio search is Phase 2; Phase 1 is audio → audio)

**Future upgrade path:**
- Add pgvector when cloud library sync arrives (Phase 2)
- Switch from flat to IVF or HNSW when library exceeds 100K sounds
- Add text-to-sound semantic search using CLAP embeddings (Phase 2)
- Add cross-user similarity search for "sounds like this one" in community

---

### 9. Job Queue

**Decision:** None in v1 (direct async generation, no queue needed)

**Why this:**
- Generation is request-response: user clicks "Generate," waits for result, sees it
- No batch processing needed in v1 (single-sound generation only)
- No background jobs that need queuing (exports are instant, analytics are inline)
- Adding a queue adds infrastructure complexity without user-visible benefit

**What NOT to build yet:**
- No Redis/Bull (queue infrastructure is overkill for v1)
- No Celery/task queue (Python async + HTTP request to model API is sufficient)
- No progress tracking beyond a spinner (generation takes 4-8 seconds; a spinner is fine)
- No job prioritization (single user, single request at a time)

**Future upgrade path:**
- Add in-process job queue (Rust `tokio::sync::mpsc` channel) when batch generation arrives
- Add Redis-backed queue (BullMQ) when cloud services need background processing
- Add progress tracking for batch jobs

---

### 10. User Accounts

**Decision:** Email-based authentication via Supabase (free tier for up to 10K users)

**Why this:**
- Supabase: free tier includes auth, user management, 50K MAU, built-in UI components
- Email + magic link: simplest auth flow, no password management
- Supabase handles: password hashing, session management, email verification
- Can self-host Supabase later if needed
- Separates auth from backend complexity

**What NOT to build yet:**
- No OAuth/SSO (Google login is Phase 2)
- No team accounts (Phase 3 enterprise feature)
- No API keys for programmatic access (Phase 3)
- No SAML/enterprise auth (Phase 4)
- No custom auth server (Supabase eliminates the need)

**Future upgrade path:**
- Add OAuth providers (Google, Apple — Phase 2)
- Self-host Supabase when usage exceeds free tier
- Add organization accounts for teams
- Add API key management for integrations

---

### 11. Export System

**Decision:** Rust-native WAV/AIFF/FLAC/MP3 export via `hound` + `lame` + custom FLAC encoder

**Why this:**
- `hound`: WAV and AIFF in pure Rust — no system dependencies
- FLAC: Rust `claxon` for encoding, or shell out to system `flac`
- MP3: `lame` via system library or `minimp3-rs` for encoding
- Export is synchronous and fast (<500ms for a one-shot)
- Direct write to user-chosen directory

**Export options (MVP):**
- WAV: 16-bit, 24-bit, 32-bit float @ 44.1kHz, 48kHz, 96kHz
- AIFF: 16-bit, 24-bit @ 44.1kHz
- FLAC: lossless compression level 5
- MP3: 320kbps CBR
- Default: WAV 24-bit 44.1kHz (industry standard for music production)

**What NOT to build yet:**
- No OGG/Opus (not widely used in music production)
- No M4A/AAC (Apple ecosystem, non-standard for producer exchange)
- No multi-format bulk export (Phase 2 pack feature)
- No export presets (hard-code sensible defaults)
- No export history beyond basic logging in SQLite

**Future upgrade path:**
- Add batch export for packs
- Add export presets (save your preferred format combination)
- Add drag-and-drop export to DAW browser folder
- Add export to Splice / Loopcloud compatibility

---

### 12. DAW/Plugin Roadmap

**Decision:** No plugin in v1. Desktop app only. Plugin roadmap starts Month 7.

**Why this:**
- Building a VST3/AU plugin is significant engineering: different audio buffer model, host communication, UI constraints
- The desktop app must be proven valuable before adding plugin complexity
- Desktop app serves as the "authoring environment"; plugin is the "consumer"
- NIH-plug is the chosen framework when we build it, but not yet

**Phased plugin approach:**
- Month 7: Research — plugin architecture probe, NIH-plug evaluation, UI constraints
- Month 8: Prototype — simple VST3 that receives generated audio from cShot process
- Month 9: Beta — basic plugin with generate button, waveform display, drag-to-DAW
- Month 12: V1 — full plugin with prompt input, variation grid, auto-populate Drum Rack

**What NOT to build yet (in v1):**
- No VST3 plugin (starts Month 7)
- No AU plugin (starts Month 7)
- No AAX plugin (Pro Tools — Phase 3, if at all)
- No CLAP plugin (Phase 3, emerging standard)
- No standalone plugin without desktop app (plugin is an extension, not the product)

**Future upgrade path:**
- VST3 → AU → CLAP → AAX in that order of priority
- Plugin communicates with desktop app via named pipe / local socket
- Eventually plugin becomes standalone (self-contained, no desktop app needed)

---

## Architecture Diagram (Year One)

```
┌──────────────────────────────────────────────────────────┐
│                 cShot Desktop App (Tauri v2)             │
│  ┌───────────────────────┐  ┌──────────────────────────┐ │
│  │   React UI (Vite+TS)  │  │     Rust Backend          │ │
│  │                       │  │                           │ │
│  │  ┌─────────────────┐  │  │  ┌───────────────────┐   │ │
│  │  │ PromptBar       │  │  │  │ Audio Pipeline    │   │ │
│  │  │ SoundGrid       │  │  │  │ - Post-processing │   │ │
│  │  │ DetailPanel     │  │  │  │ - Analyze         │   │ │
│  │  │ ExportDialog    │  │  │  │ - Encode/Export   │   │ │
│  │  │ LibraryView     │◄─┼──┼──┤ - Onset detect    │   │ │
│  │  │ Settings        │  │  │  └───────────────────┘   │ │
│  │  └─────────────────┘  │  │                           │ │
│  │                       │  │  ┌───────────────────┐   │ │
│  │  ┌─────────────────┐  │  │  │ Library Manager   │   │ │
│  │  │ Zustand Stores  │  │  │  │ - SQLite (rusqlite)│   │ │
│  │  │ - generation    │◄─┼──┼──┤ - FAISS index     │   │ │
│  │  │ - library       │  │  │  │ - Content-addressed│   │ │
│  │  │ - settings      │  │  │  └───────────────────┘   │ │
│  │  │ - audio         │  │  │                           │ │
│  │  └─────────────────┘  │  │  ┌───────────────────┐   │ │
│  │                       │  │  │ Model Gateway      │   │ │
│  └───────────────────────┘  │  │ - HTTP client      │   │ │
│                             │  │ - Local ONNX       │   │ │
│                             │  │ - Failover logic   │   │ │
│                             │  └───────────────────┘   │ │
│                             │                           │ │
│                             │  ┌───────────────────┐   │ │
│                             │  │ IPC Bridge         │   │ │
│                             │  └───────────────────┘   │ │
│                             └──────────┬────────────────┘ │
└────────────────────────────────────────┼──────────────────┘
                                         │
                              ┌──────────┴──────────┐
                              │     HTTPS / WSS      │
                              └──────────┬──────────┘
                                         │
┌────────────────────────────────────────┼──────────────────┐
│           Cloud Services                │                  │
│  ┌─────────────────┐  ┌────────────────┐                  │
│  │ FastAPI Gateway  │  │ Supabase Auth │                  │
│  │ - Auth check     │  │ - Email login │                  │
│  │ - Model routing  │  │ - Session mgmt│                  │
│  │ - Rate limiting  │  └────────────────┘                  │
│  └────────┬────────┘                                      │
│           │                                                │
│  ┌────────┴────────┐  ┌────────────────┐                  │
│  │ Model API Client│  │ Job Queue      │                  │
│  │ - ElevenLabs    │  │ (Redis, Phase 2)│                  │
│  │ - Stable Audio  │  └────────────────┘                  │
│  │ - Replicate     │                                      │
│  └─────────────────┘                                      │
└────────────────────────────────────────────────────────────┘
```

## Summary: What Ships in Year One

| Component | Ships | Technology |
|---|---|---|
| Desktop app | ✅ | Tauri v2 + Rust + React |
| Prompt-based generation | ✅ | Text → CLAP embedding → Diffusion |
| 6-slot variation grid | ✅ | React + Web Audio API |
| Reference upload | ✅ | Drag-and-drop WAV + spectral analysis |
| Waveform preview | ✅ | Canvas-based SVG waveform |
| SoundScore | ✅ | ONNX quality model (~5MB) |
| SQLite library | ✅ | rusqlite, content-addressed storage |
| FAISS similarity search | ✅ | Local vector search (audio → audio) |
| WAV export | ✅ | hound (Rust) |
| AIFF/FLAC/MP3 export | ✅ | hound + claxon + lame |
| Pack creation | ✅ | Group sounds → named pack → export all |
| Offline mode | ✅ | Local ONNX inference (Phase 2 month 5) |
| User accounts | ✅ | Supabase email auth |
| Windows support | ✅ | Tauri cross-compile |
| VST3/AU plugin | ❌ | Month 7-12 |
| Cloud sync | ❌ | Phase 2 |
| Taste personalization | ❌ | Phase 2 |
| VST3 hosting | ❌ | Phase 3 |
| Community marketplace | ❌ | Phase 3 |

## Non-Goals for Year One

- Do NOT build a cloud storage platform
- Do NOT build a community marketplace
- Do NOT build mobile apps
- Do NOT build a web version
- Do NOT build enterprise features
- Do NOT build DAW plugin hosting
- Do NOT build real-time collaboration
- Do NOT build a public API
- Do NOT build any social/feed features
- Do NOT build advanced audio editing (EQ, compression, reverb)
