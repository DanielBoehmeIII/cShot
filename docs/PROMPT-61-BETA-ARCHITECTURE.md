# Prompt 61 — Design the Beta Architecture

The beta architecture for cShot after alpha validation. What to rebuild, what to keep, and how every system connects.

---

## 0. Context: Where Alpha Left Off

Alpha proved:
- **Kicks and bass are magical** (4.2★, 42.7% export rate) — cShot is 2x better at these than anything else
- **Users treat cShot as a faucet, not a library** — generate → export → done, no browsing
- **Reference upload is a superpower** — 2x satisfaction, 3x export rate when conditioned on a reference
- **Latency variance is the #1 UX killer** — P95 at 14.7s caused abandonment
- **7.2% failure rate** — silent failures, clipping, wrong duration

Stack after alpha: Tauri v2 + React + Rust + ElevenLabs SFX API + local DSP (trim, normalize, fade)

---

## 1. Beta Architecture Overview

```
┌────────────────────────────────────────────────────────────────────┐
│                        Tauri Desktop App                            │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │  React UI    │  │  Audio Engine│  │  User Library (SQLite)   │  │
│  │  (Vite + TS) │◄─┤  (Rust DSP)  │  │  - Favorites            │  │
│  │              │  │              │  │  - History              │  │
│  │  - Prompt    │  │  - Trim      │  │  - Tags                │  │
│  │  - Waveform  │  │  - Normalize │  │  - Taste embeddings    │  │
│  │  - Controls  │  │  - EQ       │  │  - Feedback ratings    │  │
│  │  - Grid      │  │  - Analyze  │  └──────────┬───────────────┘  │
│  └──────┬───────┘  └──────┬──────┘             │                  │
│         │ IPC (invoke)    │ IPC                 │ IPC               │
│         ▼                 ▼                     ▼                  │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                    Rust Orchestrator                          │  │
│  │  - Generation pipeline orchestration                        │  │
│  │  - Job queue (async, event-driven)                          │  │
│  │  - SoundScore computation                                    │  │
│  │  - Repair chain dispatching                                  │  │
│  │  - Model gateway client                                      │  │
│  │  - Export pipeline                                           │  │
│  │  - Cost tracking                                             │  │
│  └────────────────────────┬────────────────────────────────────┘  │
│                           │                                       │
└───────────────────────────┼───────────────────────────────────────┘
                            │ HTTPS
                            ▼
┌────────────────────────────────────────────────────────────────────┐
│                     Cloud Services Layer                            │
│                                                                    │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Model Gateway    │  │ Audio Proc   │  │ Auth & User Service  │  │
│  │                  │  │ Service      │  │                      │  │
│  │ - ElevenLabs SFX │  │ - Reference  │  │ - JWT auth           │  │
│  │ - Model fallback │  │   analysis   │  │ - User profiles      │  │
│  │ - RipX DAW       │  │ - Stem sep   │  │ - Subscription mgmt  │  │
│  │ - Custom model   │  │   (future)   │  │ - Rate limiting      │  │
│  │ - A/B routing    │  │ - BPM/key    │  └──────────────────────┘  │
│  └────────┬─────────┘  └──────┬───────┘                           │
│           │                   │                                    │
│  ┌────────▼───────────────────▼───────┐  ┌──────────────────────┐  │
│  │      Job Queue (Redis + Bull)      │  │   File Storage       │  │
│  │      - Generation jobs             │  │   (S3 / R2)          │  │
│  │      - Repair chain jobs           │  │   - Generated WAVs   │  │
│  │      - Batch pack jobs             │  │   - Reference uploads│  │
│  │      - Export jobs                 │  │   - Pack archives    │  │
│  │      - Feedback processing         │  │   - Temp cache       │  │
│  └────────────────────────────────────┘  └──────────────────────┘  │
│                                                                    │
│  ┌──────────────────────┐  ┌───────────────────────────────────┐  │
│  │   Metadata DB        │  │   Monitoring & Cost Control       │  │
│  │   (Postgres)         │  │                                   │  │
│  │   - Users            │  │   - Generation latency (P50/P95)  │  │
│  │   - Generations log  │  │   - API cost per generation       │  │
│  │   - Exports log      │  │   - Error rate tracking           │  │
│  │   - Feedback ratings │  │   - Monthly budget enforcement    │  │
│  │   - Sound metadata   │  │   - Anomaly detection             │  │
│  │   - Taste profiles   │  │   - Daily cost reports            │  │
│  └──────────────────────┘  └───────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

---

## 2. Frontend Architecture (Beta)

### What Stays from Alpha

| Component | Status | Reason |
|-----------|--------|--------|
| Single-screen generation UI | **Keep** | Validated — users love it. No app redesign needed. |
| Prompt input + generate button | **Keep** | Core loop works. Small UX tweaks (auto-focus, shortcuts). |
| Waveform rendering | **Keep** | Web Audio API + canvas. Reliable, fast. |
| Sound grid (horizontal scroll) | **Keep** | Users generate 1-2 sounds, not 6. Keep it lean. |
| Export flow (one-click WAV) | **Keep** | Rated "perfect." Don't touch it. |
| Favorites (heart toggle) | **Keep** | 30% used it. Worth keeping for future taste learning. |
| Dark theme | **Keep** | Producers expect dark UIs. No redesign needed. |
| Toast notifications | **Keep** | Works. Expand for new events. |

### What Gets Rebuilt

| Component | Change | Why |
|-----------|--------|-----|
| Prompt suggestion chips | **Rebuild** | Alpha showed users need guidance. Chips for BPM, genre, mood, descriptors. |
| Reference upload zone | **Rebuild** | Make it first-class. Dedicated drop zone, always visible. Show analysis results. |
| Sound detail panel | **New** | Expandable panel shows SoundScore, tags, waveform, controls, repair info. |
| High-level controls | **New** | Punch, body, weight, snap, air knobs (Prompt 57). Post-generation sound shaping. |
| Generation history | **New** | Scrollable sidebar showing recent generations with prompt, score, type. |
| Keyboard shortcuts | **Rebuild** | Space=play/pause, Enter=generate, Tab=focus, Escape=clear. DAW-like. |
| Feedback widget | **Rebuild** | End-of-session 2-question survey instead of intrusive in-flow prompts. |
| SoundScore badge | **New** | Prominent quality score on each sound. Color-coded (red/yellow/green). |

### Key New Components

```typescript
// New frontend components for beta
interface ReferenceZone {
  file: File | null;
  analysis: {
    bpm: number | null;
    key: string | null;
    spectral_profile: number[];
    genre_hints: string[];
  } | null;
  status: 'empty' | 'analyzing' | 'ready' | 'error';
}

interface HighLevelControls {
  soundId: string;
  controls: {
    punch: number;    // 0-100, transient emphasis
    body: number;     // 0-100, low-mid presence
    weight: number;   // 0-100, sub-bass content
    snap: number;     // 0-100, attack sharpness
    air: number;      // 0-100, high-frequency shimmer
  };
  onControlChange: (control: string, value: number) => void;
}

interface SoundScoreBadge {
  score: number;      // 0-100
  breakdown: {
    mix_readiness: number;
    punch: number;
    clarity: number;
    uniqueness: number;
  };
  color: 'red' | 'yellow' | 'green';
}
```

---

## 3. Backend Architecture (Rust/Tauri)

### What Stays from Alpha

| Module | Status | Reason |
|--------|--------|--------|
| `audio/dsp.rs` (trim, normalize, fade) | **Keep** | Works. Extend for EQ, envelopes. |
| `audio/analyze.rs` (duration, RMS, peak) | **Keep** | Works. Extend for spectral analysis. |
| `audio/export.rs` (WAV writer) | **Keep** | Rated "perfect." Don't touch. |
| `commands/export.rs` | **Keep** | Works. |
| `storage/files.rs` (content-addressed) | **Keep** | Good architecture. Extend for cloud sync. |

### What Gets Rebuilt

| Module | Change | Why |
|--------|--------|-----|
| `commands/generation.rs` | **Rebuild** | Needs async job queue, multiple model support, SoundScore integration. |
| `audio/generate.rs` | **Rebuild** | Separate orchestration from model calls. Add retry, fallback validation. |
| `model/gateway.rs` | **New** | Multi-model abstraction. Route by sound type, check health, fallback chain. |
| `audio/repair.rs` | **New** | Repair chain (Prompt 53): detect issues, apply fixes, validate output. |
| `audio/quality.rs` | **New** | SoundScore (Prompt 54): compute mix-readiness, punch, clarity, uniqueness. |
| `audio/controls.rs` | **New** | High-level controls (Prompt 57): apply punch, body, weight, snap, air. |
| `commands/batch.rs` | **New** | Batch generation for pack building (Phase 2). |
| `db/library.rs` | **Rebuild** | Upgrade from flat JSON to SQLite. Add taste embeddings table. |
| `db/feedback.rs` | **New** | Feedback ratings tracking. Aggregate stats per user. |
| `jobs/mod.rs` | **New** | Async job queue with progress events. |
| `cost/mod.rs` | **New** | Track API cost per generation, enforce monthly budgets. |
| `monitoring/mod.rs` | **New** | Telemetry collection, latency tracking, error logging. |

### Rust Module Structure (Beta)

```
src-tauri/
├── src/
│   ├── main.rs                    # Tauri entry, plugin registration
│   ├── lib.rs
│   │
│   ├── commands/                  # Tauri IPC handlers
│   │   ├── mod.rs
│   │   ├── generation.rs          # generate, regenerate, retry
│   │   ├── preview.rs             # get_audio_data, waveform thumbnail
│   │   ├── export.rs              # export_wav, export_batch
│   │   ├── controls.rs            # apply high-level controls
│   │   ├── reference.rs           # upload, analyze reference
│   │   ├── library.rs             # favorites, history, search
│   │   ├── feedback.rs            # submit rating, get stats
│   │   └── settings.rs            # user preferences, cost controls
│   │
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── dsp.rs                 # trim, normalize, fade, EQ, envelope
│   │   ├── analyze.rs             # spectral centroid, RMS, BPM, key, type classification
│   │   ├── quality.rs             # SoundScore computation
│   │   ├── repair.rs              # repair chain (detect → fix → validate)
│   │   ├── controls.rs            # high-level control application
│   │   ├── morph.rs               # sound morphing (Phase 2)
│   │   └── export.rs              # WAV encoding
│   │
│   ├── model/
│   │   ├── mod.rs
│   │   ├── gateway.rs             # multi-model abstraction layer
│   │   ├── elevenlabs.rs          # ElevenLabs SFX API client
│   │   ├── fallback.rs            # fallback chain logic
│   │   └── types.rs               # generation request/response types
│   │
│   ├── jobs/
│   │   ├── mod.rs
│   │   ├── queue.rs               # async job queue
│   │   ├── generation_job.rs      # generation work
│   │   └── repair_job.rs          # repair chain work
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── library.rs             # sounds, favorites, history
│   │   ├── feedback.rs            # ratings, feedback events
│   │   ├── taste.rs               # taste embeddings, preference data
│   │   └── migrations.rs          # schema management
│   │
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── local.rs               # content-addressed local storage
│   │   ├── cache.rs               # LRU audio cache
│   │   └── cloud.rs               # S3/R2 sync (future)
│   │
│   ├── cost/
│   │   ├── mod.rs
│   │   ├── tracker.rs             # per-generation cost tracking
│   │   ├── budget.rs              # monthly budget enforcement
│   │   └── limits.rs              # rate limiting, concurrent generation caps
│   │
│   └── monitoring/
│       ├── mod.rs
│       ├── telemetry.rs           # event collection
│       ├── metrics.rs             # latency, error rate, generation stats
│       └── logging.rs             # structured logging
```

---

## 4. Audio Processing Service

A lightweight Rust service that handles all audio processing outside the Tauri process. Decoupled so it can be reused in the future VST3 plugin.

```
┌─────────────────────────────┐
│   Audio Processing Service  │
│                             │
│  ┌───────────────────────┐  │
│  │   DSP Pipeline        │  │
│  │                       │  │
│  │  Input → Trim → EQ →  │  │
│  │  Normalize → Envelope │  │
│  │  → Fade → Output      │  │
│  └───────────────────────┘  │
│                             │
│  ┌───────────────────────┐  │
│  │   Repair Chain        │  │
│  │                       │  │
│  │  Detect → Classify →  │  │
│  │  Fix → Validate       │  │
│  └───────────────────────┘  │
│                             │
│  ┌───────────────────────┐  │
│  │   Analysis Engine     │  │
│  │                       │  │
│  │  Sound type classify  │  │
│  │  Spectral analysis    │  │
│  │  BPM/key detection    │  │
│  │  Quality scoring      │  │
│  └───────────────────────┘  │
│                             │
│  ┌───────────────────────┐  │
│  │   High-Level Controls │  │
│  │                       │  │
│  │  Punch → transient    │  │
│  │  Body → low-mid shelf │  │
│  │  Weight → sub-bass    │  │
│  │  Snap → attack shape  │  │
│  │  Air → high shelf     │  │
│  └───────────────────────┘  │
└─────────────────────────────┘
```

**Key design decisions:**
- All DSP is synchronous, sub-millisecond. Never blocks generation.
- Accepts raw Float32 audio, returns processed Float32 audio.
- Stateless pipeline — each call is independent.
- Exposed via gRPC or shared library for plugin reuse.

---

## 5. Model Gateway

Abstraction layer over all generation models. The beta needs to handle multiple providers transparently.

```
┌──────────────────────────────────────────────┐
│              Model Gateway                     │
│                                                │
│  Request: { prompt, reference, sound_type }    │
│                                                │
│  1. Route by sound_type:                       │
│     - Kicks/bass → ElevenLabs SFX (primary)    │
│     - Snares → Custom snare-finetuned model     │
│     - Hats → ElevenLabs + DSP repair            │
│     - Unknown → Round-robin test both           │
│                                                │
│  2. Health check:                              │
│     - Is provider reachable?                   │
│     - Latency OK? (<10s target)                │
│     - Quota remaining?                         │
│                                                │
│  3. Execute with fallback:                     │
│     Primary → On failure → Fallback →          │
│     On failure → DSP template generation       │
│                                                │
│  4. Validate output:                           │
│     - Duration in expected range              │
│     - No clipping (>0dBFS)                     │
│     - No silence (RMS > threshold)             │
│     - SoundScore > minimum                     │
│     - If fail: retry or mark for repair        │
└────────────────────────────────────────────────┘
```

### Provider Routing Table

| Sound Type | Primary | Fallback 1 | Fallback 2 | Notes |
|-----------|---------|-----------|-----------|-------|
| Kick | ElevenLabs SFX | Stable Audio | DSP synth | ElevenLabs is 4.2★ for kicks |
| Snare | Custom snare model | ElevenLabs SFX | DSP + noise | Alpha snares were 1.5★ — needs dedicated fix |
| Hi-hat | ElevenLabs SFX | Stable Audio | DSP synth | Needs closed-hat fix (too long) |
| Bass/808 | ElevenLabs SFX | Stable Audio | DSP sine + FX | 42.7% export rate — keep primary |
| Clap | ElevenLabs SFX | Stable Audio | DSP layered | Acceptable quality |
| Perc | Stable Audio | ElevenLabs SFX | DSP synthesis | Lower volume category |
| FX | Stable Audio | ElevenLabs SFX | DSP granular | Needs improvement |
| Texture | Stable Audio | — | DSP granular | 2.3★ — may never be good with current models |

### Gateway API

```rust
pub struct GenerationRequest {
    pub prompt: String,
    pub sound_type: Option<SoundType>,
    pub reference: Option<Vec<f32>>,
    pub reference_analysis: Option<ReferenceAnalysis>,
    pub seed: Option<u32>,
    pub duration_ms: Option<u32>,
    pub bpm: Option<u32>,
    pub key: Option<String>,
    pub quality_tier: QualityTier, // Fast | Balanced | Quality
}

pub struct GenerationResponse {
    pub audio: Vec<f32>,
    pub sample_rate: u32,
    pub provider: String,
    pub model_version: String,
    pub latency_ms: u64,
    pub cost_cents: f64,
    pub sound_score: SoundScore,
    pub issues: Vec<AudioIssue>,
    pub analysis: AudioAnalysis,
}

pub enum QualityTier {
    Fast,      // ~2s, lower quality, lower cost
    Balanced,  // ~5s, default
    Quality,   // ~10s, best quality, higher cost
}
```

---

## 6. File Storage

### Local Storage (Primary)

```
~/cShot/
├── audio/{hash_prefix}/{hash}.wav    # Content-addressed, dedup
├── exports/{date}/{type}_{bpm}_{key}_{seed}.wav
├── references/{uuid}.wav              # User-uploaded reference tracks
├── packs/{pack_id}/{sound}.wav        # Generated packs (Phase 2)
├── cache/waveforms/{hash}.json        # Pre-computed waveform thumbnails
├── cache/analysis/{hash}.json         # Cached analysis results
├── library.db                         # SQLite
├── config.toml                        # User preferences
└── telemetry.db                       # Local telemetry buffer
```

### Cloud Storage (for sync, Phase 3+)

Cloud storage is not required for beta. Local-first is the right model. But the storage layer should be designed so cloud sync is a future addition.

```
Storage trait:
  store(audio, metadata) → hash
  load(hash) → audio
  list(user_id) → metadata[]
  delete(hash)
  
Implementations:
  LocalStorage  → filesystem
  CloudStorage  → S3/R2 (future)
```

---

## 7. Metadata Database (Postgres)

Replaces local-only SQLite with a cloud Postgres instance for user accounts, cross-session taste learning, and monitoring. The local SQLite remains for offline cache.

### Schema

```sql
-- Users & auth
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    subscription_tier TEXT NOT NULL DEFAULT 'free',
    subscription_ends_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ
);

CREATE TABLE api_keys (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    key_hash TEXT NOT NULL,
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

-- Generation tracking
CREATE TABLE generations (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    prompt TEXT NOT NULL,
    sound_type TEXT,
    provider TEXT NOT NULL,
    model_version TEXT,
    latency_ms INTEGER NOT NULL,
    cost_cents REAL NOT NULL,
    success BOOLEAN NOT NULL DEFAULT TRUE,
    error_type TEXT,
    sound_score REAL,
    seed INTEGER,
    duration_ms REAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User actions for taste learning
CREATE TABLE user_actions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    generation_id UUID REFERENCES generations(id),
    action_type TEXT NOT NULL, -- 'export', 'favorite', 'delete', 'skip', 'rating'
    rating INTEGER,             -- 1-5 for rating actions
    sound_score REAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_actions_user ON user_actions(user_id, created_at DESC);
CREATE INDEX idx_generations_user ON generations(user_id, created_at DESC);

-- Feedback
CREATE TABLE feedback (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    session_id UUID,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    rating INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cost tracking
CREATE TABLE cost_entries (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    generation_id UUID REFERENCES generations(id),
    provider TEXT NOT NULL,
    amount_cents REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    category TEXT NOT NULL, -- 'generation', 'repair', 'analysis'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Taste Embeddings (For Phase 4 prep)

```sql
CREATE TABLE taste_embeddings (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    embedding VECTOR(768),        -- pgvector
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    generation_count INTEGER DEFAULT 0,
    export_count INTEGER DEFAULT 0,
    favorite_count INTEGER DEFAULT 0,
    
    -- Preference aggregates
    pref_spectral_brightness REAL,    -- 0-1, average spectral centroid
    pref_punch REAL,                   -- 0-1
    pref_sub_weight REAL,              -- 0-1
    pref_temporal_density REAL,        -- 0-1, busy vs sparse
    pref_duration_ms REAL,             -- average preferred duration
    pref_genres TEXT[],                -- most common genre prompts
    pref_sound_types TEXT[],           -- most generated/exported types
    
    reset_token TEXT,                  -- for privacy reset
    reset_requested_at TIMESTAMPTZ
);
```

---

## 8. Job Queue

### Architecture

```
                    ┌─────────────────────┐
                    │   Frontend submits   │
                    │   generation request │
                    └──────────┬──────────┘
                               │
                    ┌──────────▼──────────┐
                    │   Rust Job Queue     │
                    │                     │
                    │   Queue: VecDeque   │
                    │   Max depth: 10     │
                    │   FIFO priority     │
                    └──────────┬──────────┘
                               │
                    ┌──────────▼──────────┐
                    │   Worker Thread     │
                    │                     │
                    │  1. Dequeue job     │
                    │  2. Model gateway   │
                    │  3. Audio process   │
                    │  4. Quality check   │
                    │  5. Emit event      │
                    └─────────────────────┘
```

### Job Types

```rust
pub enum JobType {
    Generation(GenerationJob),
    Regeneration(RegenerationJob),   // "generate more like this"
    Repair(RepairJob),               // repair chain on existing sound
    BatchGeneration(BatchJob),       // for packs (Phase 2)
    ReferenceAnalysis(RefJob),       // analyze uploaded reference
}

pub struct GenerationJob {
    pub id: String,
    pub prompt: String,
    pub config: GenerationConfig,
    pub reference: Option<Vec<f32>>,
    pub submitted_at: Instant,
    pub retry_count: u8,
    pub max_retries: u8,
}
```

### Progress Events

```typescript
// Emitted from Rust to frontend via Tauri events
interface GenerationProgress {
  jobId: string;
  stage: 'queued' | 'generating' | 'processing' | 'analyzing' | 'complete' | 'failed';
  progress: number;       // 0-100
  message: string;        // User-facing status text
}

// Example flow:
// Stage: queued → "Waiting in line..."
// Stage: generating → "Generating sound..." (0-60%)
// Stage: processing → "Applying DSP..." (60-80%)
// Stage: analyzing → "Checking quality..." (80-95%)
// Stage: complete → "Sound ready!" (100%)
// Stage: failed → "Generation failed. Retrying..." (error)
```

---

## 9. User Library (Local SQLite)

Alpha proved users don't want a library browser. But the beta needs a hidden library for:
- Generation history (scrollable, searchable by prompt)
- Favorites (for export and taste learning)
- Recently exported sounds
- Feedback ratings tied to specific sounds

### Key Decision: Library is a data store, not a UI destination

The library should not have its own page. It is accessed through:
- "History" sidebar in the generation view
- "Favorites" filter in the generation view
- Implicit: the app remembers what you liked

### Library Schema (SQLite, local)

```sql
CREATE TABLE sounds (
    id TEXT PRIMARY KEY,
    audio_hash TEXT NOT NULL UNIQUE,
    prompt TEXT NOT NULL,
    seed INTEGER NOT NULL,
    provider TEXT NOT NULL,
    model_version TEXT NOT NULL,
    
    -- Audio metadata
    duration_ms REAL NOT NULL,
    sample_rate INTEGER NOT NULL,
    channels INTEGER NOT NULL DEFAULT 1,
    rms REAL,
    peak REAL,
    
    -- Analysis
    sound_type TEXT,
    spectral_centroid REAL,
    sound_score REAL,
    tags TEXT,                     -- JSON array
    
    -- User state
    is_favorite INTEGER DEFAULT 0,
    user_rating INTEGER,           -- 1-5, NULL if unrated
    export_count INTEGER DEFAULT 0,
    
    -- Timestamps
    created_at TEXT NOT NULL,
    exported_at TEXT,
    last_played_at TEXT,
    
    -- Cost
    cost_cents REAL
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    generation_count INTEGER DEFAULT 0,
    export_count INTEGER DEFAULT 0,
    favorite_count INTEGER DEFAULT 0
);
```

---

## 10. Feedback Tracking

### What Alpha Taught

| Feedback Method | Engagement | Quality | Verdict |
|----------------|-----------|---------|---------|
| In-flow rating prompts | 40% answered | Low | Remove from flow |
| End-of-session survey | 25% opened | Medium | Keep, simplify |
| Direct calls/interviews | 5 users | Very high | Invest here |
| Telemetry (opt-in) | 100% | High | Essential |

### Beta Feedback Architecture

```
┌─────────────────────────────────────────────┐
│          Feedback Collection                 │
│                                             │
│  Implicit (always on):                      │
│  ┌───────────────────────────────────────┐  │
│  │  - Export → positive signal          │  │
│  │  - Favorite → strong positive        │  │
│  │  - Delete → negative signal          │  │
│  │  - Skip/regenerate → weak negative   │  │
│  │  - Time spent previewing → interest   │  │
│  │  - Prompt modification → iteration    │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  Explicit (sparse):                        │
│  ┌───────────────────────────────────────┐  │
│  │  - SoundScore displays automatically │  │
│  │  - "Was this sound useful?" (1 tap)  │  │
│  │  - End-of-session: "What should we   │  │
│  │    improve?" (free text, optional)   │  │
│  │  - Monthly NPS survey (email)        │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  Automated:                                 │
│  ┌───────────────────────────────────────┐  │
│  │  - SoundScore computed for every     │  │
│  │    generation                         │  │
│  │  - Repair chain logs every fix       │  │
│  │  - Generation latency tracked        │  │
│  │  - Error rate aggregated             │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### Feedback Data Model

```rust
pub struct FeedbackEvent {
    pub user_id: String,
    pub generation_id: String,
    pub event_type: FeedbackType,
    pub value: Option<f64>,         // rating 1-5, or score
    pub context: Option<String>,    // free text
    pub created_at: DateTime<Utc>,
}

pub enum FeedbackType {
    // Implicit
    Export,
    Favorite,
    Unfavorite,
    DeleteGenerated,
    Regenerate,
    PreviewLonger,        // previewed >10s = high interest
    PreviewShorter,       // previewed <1s = low interest
    
    // Explicit
    RatingStar,           // 1-5 star rating
    UsefulYes,
    UsefulNo,
    FreeText,
    
    // Automated
    SoundScore,           // computed quality score
    RepairApplied,
    IssueDetected,
}
```

---

## 11. Export System

### What Stays

- One-click WAV export (native save dialog) — keep exactly as is
- Drag-and-drop from app to Finder/Explorer — keep
- Default naming: `{type}_{bpm}_{key}_{seed}.wav` — keep

### What Gets Added

| Feature | Description | Priority |
|---------|-------------|----------|
| Batch export | Select multiple sounds, export as zip | Medium |
| Metadata embedding | Write prompt, tags, SoundScore into WAV metadata | Medium |
| Export all from session | "Export all sounds from this session" button | Low |
| Preset naming | Customizable filename templates | Low |
| Resampled export | Export at 48kHz, 96kHz for film/game | Low |

### WAV Metadata Embedding

```rust
// Embed structured metadata in WAV files using RIFF chunks
pub struct ExportMetadata {
    pub prompt: String,
    pub sound_type: String,
    pub sound_score: f64,
    pub bpm: Option<u32>,
    pub key: Option<String>,
    pub tags: Vec<String>,
    pub model_version: String,
    pub generation_id: String,
    pub provider: String,
}

// Written as RIFF INFO chunk + custom cShot chunk
// WAV file with metadata is still a valid WAV — plays everywhere
// cShot can read its own chunks for provenance tracking
```

---

## 12. Monitoring System

### What to Monitor

```
Generation Metrics:
  - P50/P90/P95 latency per provider     ← Critical (alpha P95 was 14.7s)
  - Generation success rate               ← Critical (alpha was 92.8%)
  - SoundScore distribution               ← Quality health
  - Error rate by error type              ← Issue detection
  - Retry rate                            ← Model reliability

Cost Metrics:
  - Cost per generation (mean, P90)       ← Budget tracking
  - Cost per user per session             ← User economics
  - Monthly total by provider             ← Provider spend
  - Daily budget burn rate                ← Budget alerts

User Metrics:
  - Generations per session               ← Engagement
  - Exports per session                   ← Value delivery
  - Session duration                       ← Stickiness
  - Return rate (D1, D7, D30)            ← Retention
  - Favorite rate                          ← Satisfaction
  - Rerate/regenerate rate                ← Quality issues

System Metrics:
  - App cold start time                   ← Desktop perf
  - Memory usage (idle, generating)       ← Desktop perf
  - CPU usage during DSP                  ← Desktop perf
  - Crash rate                             ← Stability
  - IPC latency                            ← UI responsiveness
```

### Monitoring Implementation

```rust
// Local first, synced to cloud periodically
pub struct TelemetryBuffer {
    events: Vec<TelemetryEvent>,
    max_buffer_size: usize,  // 1000 events before flush
    sync_interval: Duration, // 5 minutes
}

pub struct TelemetryEvent {
    pub event_type: MetricType,
    pub value: f64,
    pub tags: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>, // anonymized, opt-in
}

pub enum MetricType {
    GenerationLatency,
    SoundScore,
    ErrorCount,
    ExportCount,
    FavoriteCount,
    RepairCount,
    CostCents,
    SessionDuration,
    MemoryUsage,
    Crash,
}
```

### Alert Thresholds

| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| P95 latency | >8s | >12s | Auto-fallback to faster model |
| Success rate | <95% | <90% | Pause affected provider, alert dev |
| SoundScore mean | <50 | <35 | Flag quality regression |
| Error rate | >5% | >10% | Investigate root cause |
| Cost/day | >$50 | >$100 | Reduce generation limits |
| Memory usage | >1GB | >2GB | Force GC, warn user |
| Crash rate | >2% | >5% | Rollback if possible |

---

## 13. Cost Controls

### Cost Tracking

```rust
pub struct CostTracker {
    // Per-session budget
    session_budget_cents: u32,      // Default: 100 ($1.00)
    session_spent_cents: u32,
    
    // Monthly budget
    monthly_budget_cents: u32,      // Default: 1000 ($10.00)
    monthly_spent_cents: u32,
    
    // Per-generation costs by provider
    provider_costs: HashMap<String, f64>, // { "elevenlabs": 0.10, "stable_audio": 0.05 }
    
    // Rate limiting
    max_generations_per_minute: u32,  // Default: 10
    generation_timestamps: VecDeque<Instant>,
}
```

### Budget Enforcement

```
Per-generation:
  - Before each generation, estimate cost based on provider
  - Check: session budget still available?
  - Check: monthly budget not exceeded?
  - Check: rate limit not hit?
  - If any check fails: show user-friendly message
    "You've used your generation budget this session (~$0.50).
     Upgrade or wait until next session."

Per-user tier:
  Free tier: $5/month budget, max 10 gen/session
  Pro tier: $20/month budget, max 50 gen/session
  Unlimited tier: $50/month, no per-session cap

Transparency:
  - Show estimated cost before generation (subtle, non-intrusive)
  - Show "This generation cost $0.08" after generation
  - Show monthly usage in settings
  - Never surprise user with costs
```

### Cost Optimization Strategies

| Strategy | Savings | Complexity | Status |
|----------|---------|------------|--------|
| Cache identical prompts | 5-10% | Low | Implement now |
| Use fast model for first pass | 30-50% | Low | Implement now |
| Batch similar prompts | 15-25% | Medium | Phase 2 |
| Local inference for DSP | Eliminates cloud cost | Done | Already done |
| Cache common sounds (kicks) | 10-15% | Low | Implement now |
| Throttle retries | 2-5% | Low | Implement now |
| Provider arbitrage (cheapest route) | 10-20% | Medium | Beta |
| Self-host model | 60-80% (if own GPU) | Very high | Phase 4 |

---

## 14. Generation Pipeline (Full Beta Flow)

```
User types prompt → "punchy trap kick 140bpm"
         │
         ▼
┌─────────────────────┐
│ 1. Validate Prompt   │
│    - Not empty        │
│    - Not excessive    │
│    - Extract BPM/key  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ 2. Route to Provider │
│    - Sound type: KICK│
│    → ElevenLabs SFX  │
│    → Quality: Balance│
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ 3. Generate          │
│    - Submit to API   │
│    - Poll for result │
│    - Timeout at 10s  │
└──────────┬──────────┘
           │
     ┌─────┴─────┐
     │           │
     ▼           ▼
  Success     Failure
     │           │
     ▼           ▼
┌────────┐  ┌────────────┐
│ 4.     │  │ Retry with │
│ Audio  │  │ fallback   │
│ Process│  │ provider   │
└────┬───┘  └──────┬─────┘
     │              │
     ▼              ▼
┌────────┐      ┌────────┐
│ 5.     │      │ 4b.    │
│ Quality│      │ Audio  │
│ Check  │      │ Process│
└────┬───┘      └────┬───┘
     │                │
     ▼                ▼
┌────────┐      ┌────────┐
│ 6.     │      │ 5b.    │
│ Analyze│      │ Quality│
│ Sound  │      │ Check  │
│ Score  │      │        │
└────┬───┘      └────┬───┘
     │                │
     ▼                ▼
┌────────┐      If still failing:
│ 7.     │      → Mark as low quality
│ Save & │      → Apply repair chain
│ Return │      → If unfixable: return error
└────────┘
```

---

## 15. Phased Beta Rollout

### Beta Phase 1 (Weeks 1-4): Reliability & Quality

```
Focus: Fix the alpha issues, make generation reliable

Ships:
  ✓ Model gateway with fallback chain
  ✓ SoundScore implementation
  ✓ Repair chain for common issues (clipping, silence, wrong duration)
  ✓ Async job queue with progress events
  ✓ Prompt suggestion chips
  ✓ Keyboard shortcuts (Space, Enter, Tab, Escape)
  ✓ Reference analysis results display
  ✓ Sound quality badge
  ✓ End-of-session feedback widget
  
Metrics targets:
  - Generation success rate: >98%
  - P95 latency: <8s
  - SoundScore mean: >55
  - User rating: >3.5★
```

### Beta Phase 2 (Weeks 5-8): Personalization Foundation

```
Focus: Start learning user preferences, improve sounds

Ships:
  ✓ User accounts (optional)
  ✓ Taste embedding collection (implicit signals)
  ✓ High-level sound controls (punch, body, weight, snap, air)
  ✓ Generation history sidebar
  ✓ Batch export
  ✓ Cost tracking display
  ✓ Telemetry dashboard (dev only)

Metrics targets:
  - 30% of users create accounts
  - 2+ exports per session
  - 30% 7-day return rate
```

### Beta Phase 3 (Weeks 9-12): Expansion

```
Focus: Packs, performance, polish

Ships:
  ✓ Pack builder (kicks + bass packs)
  ✓ Cohesion metrics for packs
  ✓ WAV metadata embedding
  ✓ Performance optimization (cache, pre-warm)
  ✓ Plugin prototype investigation
  ✓ Migration path to DAW plugin
  ✓ Full monitoring dashboard

Metrics targets:
  - 10% conversion to paid
  - 20% of sessions use pack builder
  - <500MB idle memory
```

---

## 16. What to Rebuild vs. What to Keep (Summary)

### Keep from Alpha (Don't Touch)

| Component | Lines | Status |
|-----------|-------|--------|
| WAV export (native dialog + hound) | ~150 | Perfect |
| Audio DSP trim/normalize/fade | ~80 | Works |
| Content-addressed storage | ~100 | Good architecture |
| Single-screen generation UI concept | — | Validated |
| Web Audio API playback | ~100 | Works |
| Toast notification system | ~50 | Works |
| Dark theme design tokens | — | Works |

### Refactor (Same Purpose, Better Implementation)

| Component | Why | Effort |
|-----------|-----|--------|
| Prompt input component | Add chips, auto-focus, shortcuts | 1 day |
| Sound grid | Add SoundScore badge, type label emphasis | 1 day |
| Favorites store | Migrate from JSON to SQLite | 1 day |
| Generation orchestration | Extract from commands into pipeline | 2 days |
| Reference upload | Make first-class, add analysis display | 2 days |

### Rebuild from Scratch

| Component | Why | Effort |
|-----------|-----|--------|
| Model gateway | Alpha had none (single hardcoded provider) | 3 days |
| Async job queue | Alpha was sync, blocking UI | 2 days |
| SoundScore engine | Didn't exist in alpha | 3 days |
| Repair chain | Didn't exist in alpha | 4 days |
| High-level controls | Didn't exist in alpha | 3 days |
| Feedback tracking | Alpha had broken in-flow prompts | 2 days |
| Cost tracking | Didn't exist in alpha | 2 days |
| Telemetry/monitoring | Alpha had none | 3 days |
| SQLite library DB | Alpha used flat JSON | 2 days |
| Taste embeddings (schema) | Didn't exist in alpha | 1 day |

### New for Beta

| Component | Effort | Phase |
|-----------|--------|-------|
| Prompt suggestion chips | 1 day | Phase 1 |
| Sound quality badge | 1 day | Phase 1 |
| High-level controls | 3 days | Phase 2 |
| Generation history sidebar | 2 days | Phase 2 |
| User accounts | 3 days | Phase 2 |
| Cost display in UI | 1 day | Phase 2 |
| Batch export | 2 days | Phase 2 |
| Pack builder | 5 days | Phase 3 |
| WAV metadata | 1 day | Phase 3 |
| Plugin prototype | — | Phase 3 (investigate only) |

---

## 17. Total Build Estimate

| Phase | Duration | Focus | Key Deliverables |
|-------|----------|-------|-----------------|
| Beta Phase 1 | 4 weeks | Reliability | Model gateway, job queue, SoundScore, repair chain, prompt chips, shortcuts |
| Beta Phase 2 | 4 weeks | Personalization | User accounts, taste embeddings, controls, history, cost tracking |
| Beta Phase 3 | 4 weeks | Expansion | Pack builder, performance, WAV metadata, plugin research |
| **Total** | **12 weeks** | **Ship beta to 100 users** | |

### Team Requirements

| Role | Phase 1 | Phase 2 | Phase 3 |
|------|---------|---------|---------|
| Rust/Tauri developer | 1 (full) | 1 (full) | 1 (full) |
| React/TypeScript developer | 1 (full) | 1 (full) | 1 (full) |
| Audio DSP engineer | 0.5 | 1 (full) | 1 (full) |
| ML engineer | 0 (API-only) | 0.5 | 0.5 |
| Backend/infra | 0 (local-first) | 0.5 | 0.5 |

---

## 18. Risk Register (Beta-Specific)

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| ElevenLabs API quality regression | Medium | High | Fallback chain to other providers |
| User accounts reduce conversion | Medium | Medium | Make accounts optional for Phase 1 |
| Desync between local DB and cloud | Low | Medium | Hybrid sync: local primary, cloud secondary |
| High cloud costs at scale | High | Critical | Cost controls, budgets, local model path |
| Beta testers churn before Phase 1 validation | Medium | High | Weekly check-in calls, fast iteration on feedback |
| Plugin build proves too complex | Medium | Medium | Defer, keep standalone as primary product |
| Audio quality doesn't improve enough | Medium | High | Invest in SoundScore-driven generation decisions |
