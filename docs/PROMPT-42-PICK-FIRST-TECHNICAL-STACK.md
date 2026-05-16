# Prompt 42 — Pick the First Technical Stack

Choose the fastest serious stack for cShot's core loop: upload audio, prompt, generate, preview, export, save.

---

## 1. Stack Comparison

### Next.js

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Audio upload | 5/10 | Works via browser API but serverless functions have timeout limits (10s on Vercel Hobby, 60s on Pro). Generation can't run in a serverless function |
| Waveform preview | 7/10 | Web Audio API in browser. Fine for display, but no native audio processing |
| Prompt-based gen | 3/10 | No local model execution. Must call external API. Vercel edge functions add latency |
| Async jobs | 4/10 | No native job queue. Need external service (Inngest, Trigger.dev) |
| WAV export | 7/10 | Works via Blob download. File save dialog is browser-controlled |
| Saved favorites | 8/10 | Database via Prisma + SQLite/Postgres. Well-supported |
| DAW/plugin expansion | 1/10 | Web app can't become a VST3/AU plugin. Would need to rebuild for native |
| **Overall** | **Web-only, good for SaaS, dead end for desktop/plugin** |

### Vite/React (Browser Only)

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Audio upload | 6/10 | File API works. No backend to process on |
| Waveform preview | 8/10 | Web Audio API + canvas. Actually good |
| Prompt-based gen | 2/10 | Must call external API. CORS issues. No local model |
| Async jobs | 3/10 | No backend. Polling or WebSocket to external service |
| WAV export | 7/10 | Blob download. Works but limited |
| Saved favorites | 6/10 | localStorage or IndexedDB. Not portable |
| DAW/plugin expansion | 1/10 | Same as Next.js — dead end for plugin |
| **Overall** | **Fast to prototype, but can't become the real product** |

### Electron/Tauri

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Audio upload | 9/10 | Native file system access. Drag-and-drop from OS |
| Waveform preview | 9/10 | Web Audio API + canvas. Same quality as browser, no limitations |
| Prompt-based gen | 8/10 | Can run local models (ONNX, llama.cpp). Can call APIs. Best of both |
| Async jobs | 8/10 | Native threads. Rust backend (Tauri) handles blocking work off the UI thread |
| WAV export | 10/10 | Native save dialog. Write directly to filesystem. No browser restrictions |
| Saved favorites | 9/10 | SQLite or JSON on local filesystem. Full control |
| DAW/plugin expansion | 7/10 | Tauri: Rust backend can be partially reused in a VST3 (via NIH-plug). Electron: dead end |
| **Overall** | **Best balance of prototype speed + future expansion** |

### Tauri vs. Electron Sub-Comparison

| Dimension | Tauri v2 | Electron |
|-----------|----------|----------|
| Install size | ~5MB + webview | ~150MB + Chromium |
| Memory usage | ~80MB idle | ~200MB idle |
| Backend language | Rust | Node.js |
| Audio processing | Native performance (hound, symphonia crates) | CPU-bound (Node.js audio libs are wrappers) |
| Plugin potential | VST3 via NIH-plug (Rust) | No native plugin path |
| Build speed | Fast (Rust compiles) | Fast (JS bundling) |
| Learning curve | Medium (need Rust) | Low (JS/TS everywhere) |
| Ecosystem maturity | Growing (Tauri v2 stable) | Mature (everything exists) |
| **Verdict** | **Better for cShot** | Viable but heavier |

### Python/FastAPI + React Frontend

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Audio upload | 8/10 | Python handles file uploads well. FastAPI + Uvicorn |
| Waveform preview | 7/10 | Compute on backend, render on frontend. Extra round-trip |
| Prompt-based gen | 9/10 | Best ML ecosystem. Hugging Face, diffusers, torchaudio. This is Python's superpower |
| Async jobs | 8/10 | FastAPI + Celery/ARQ. Well-established patterns |
| WAV export | 7/10 | Backend writes WAV, sends as download. Works but extra hop |
| Saved favorites | 8/10 | FastAPI + SQLite/Postgres. Standard web pattern |
| DAW/plugin expansion | 3/10 | Python can't become a VST3. Would need Rust/C++ bridge |
| **Overall** | **Best for ML experimentation, worst for desktop/plugin delivery** |

### Node Backend (Express/Fastify) + React Frontend

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Audio upload | 7/10 | Standard multer/multipart. Works |
| Waveform preview | 6/10 | Backend can compute waveform data. Extra round-trip |
| Prompt-based gen | 5/10 | Node.js bindings for ONNX exist but immature. Usually calls Python subprocess |
| Async jobs | 8/10 | Bull/BullMQ for Redis-backed queues. Battle-tested |
| WAV export | 7/10 | Same as FastAPI — backend writes, frontend downloads |
| Saved favorites | 8/10 | Express + SQLite/Postgres. Standard |
| DAW/plugin expansion | 2/10 | Node.js can't become a VST3 |
| **Overall** | **Worst of both worlds — weaker ML than Python, weaker desktop than Tauri** |

### Local Audio Processing (Native Rust/C++)

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Audio upload | 10/10 | Native file handling. Zero overhead |
| Waveform preview | 8/10 | Compute natively, send to UI. Fast but needs IPC |
| Prompt-based gen | 7/10 | Rust ML ecosystem growing (ort, candle, burn). Not as mature as Python |
| Async jobs | 10/10 | Native threading. Perfect for audio workloads |
| WAV export | 10/10 | Direct filesystem write. No overhead |
| Saved favorites | 7/10 | SQLite via rusqlite. Well-supported |
| DAW/plugin expansion | 10/10 | VST3 in Rust (NIH-plug) is the most direct path |
| **Overall** | **Endgame stack, but slowest to prototype in** |

### Cloud Generation APIs

| Dimension | Rating | Notes |
|-----------|--------|-------|
| Audio upload | 7/10 | Send file to API, get result back |
| Waveform preview | 6/10 | Need to download result first, then display |
| Prompt-based gen | 9/10 | Best quality, no local GPU needed. Pay per generation |
| Async jobs | 6/10 | API-dependent. Some are sync, some async |
| WAV export | 7/10 | Download from API, save locally |
| Saved favorites | 8/10 | Same as web stack |
| DAW/plugin expansion | 5/10 | Can work, but requires internet. Latency adds up |
| **Overall** | **Best quality, fastest to start, has ongoing cost** |

---

## 2. Recommended Stack: Tauri v2 + React + Rust + Cloud API (Hybrid)

### The Hybrid Strategy

```
┌──────────────────────────────────────────────┐
│                  Tauri App                     │
│  ┌──────────────────────────────────────────┐ │
│  │         Frontend (React + Vite)          │ │
│  │  - Prompt bar + upload zone              │ │
│  │  - Waveform + playback (Web Audio API)   │ │
│  │  - Export trigger + favorites toggle     │ │
│  └──────────────────┬───────────────────────┘ │
│                     │ IPC (invoke)             │
│  ┌──────────────────▼───────────────────────┐ │
│  │          Rust Backend (Tauri)             │ │
│  │  - File I/O, WAV encoding                │ │
│  │  - Audio processing (trim, normalize)    │ │
│  │  - Local DSP (envelopes, EQ, pitch)      │ │
│  │  - Favorites JSON persistence            │ │
│  │  - Cloud API orchestration               │ │
│  └──────────────────┬───────────────────────┘ │
│                     │ HTTP                     │
│  ┌──────────────────▼───────────────────────┐ │
│  │      Cloud Gen API (REST/WebSocket)      │ │
│  │  - Stable Audio / AudioCraft / ElevenLabs │ │
│  │  - Prompt → one-shot generation          │ │
│  │  - Async job with polling                │ │
│  └──────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
```

### Why Hybrid

| Concern | Local (Rust) | Cloud (API) | Hybrid Choice |
|---------|-------------|-------------|---------------|
| Upload handling | Excellent | Good | **Local** — files stay on user's machine until generation |
| DSP/processing | Instant | Latency + cost | **Local** — trim, normalize, fade are free and instant |
| Waveform rendering | Fast | Extra round-trip | **Local** — compute from local file |
| AI generation | Slow (CPU) or GPU-dependent | Fast + high quality | **Cloud** — best quality, no GPU requirement |
| WAV export | Direct filesystem | Need to download | **Local** — write directly |
| Favorites | Local persistence | DB round-trip | **Local** — JSON file, zero latency |
| Future VST3 | Rust NIH-plug | Would need client | **Local path preserved** — Rust backend can evolve into plugin |

### Why Not Pure Local Inference

- ONNX Runtime + diffusion model = 2-5s on RTX 3060 but 30-60s on CPU
- Most producers use laptops (M-series Macs or mid-range Windows)
- A cloud API gives 2-3s generation on any hardware
- Cost at prototype scale: ~$0.10/generation → $10 for first 100 tests
- Local DSP handles all non-AI audio work instantly

---

## 3. Stack Components

### Layer 1: Desktop Shell (Tauri v2)

```
Framework: Tauri v2
Rust edition: 2024
Window: 900x700, resizable, dark theme
Build target: Developer's OS first (macOS for M1 testing)
Distribution: Direct download (no auto-update for prototype)
```

Key config:
```toml
# Cargo.toml
[dependencies]
tauri = { version = "2", features = ["dialog"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
hound = "3"
uuid = { version = "1", features = ["v4"] }
base64 = "0.22"
```

### Layer 2: Frontend (React + Vite)

```
Framework: React 18 + TypeScript + Vite
Styling: Tailwind CSS (minimal config)
State: useState + useEffect (no state library for prototype)
Audio: Web Audio API
Build: Vite dev server → Tauri embeds production build
```

### Layer 3: Audio Processing (Rust)

```
Modules:
  - audio/process.rs: trim silence, normalize peak, apply fades
  - audio/dsp.rs: EQ shelves, envelope shaping, pitch shifting
  - audio/analyze.rs: spectral centroid, RMS, zero-crossing rate, duration
  - audio/waveform.rs: compute 80-point waveform thumbnail
  - audio/export.rs: write WAV at 44.1kHz/24-bit/mono
```

### Layer 4: Generation API Interface (Rust)

```rust
// Cloud API client
pub struct GenApiClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl GenApiClient {
    // Submit generation job
    pub async fn generate(&self, prompt: &str, reference: Option<&[f32]>) -> Result<GenJob>;
    
    // Poll for completion
    pub async fn poll(&self, job_id: &str) -> Result<GenStatus>;
    
    // Download result
    pub async fn download(&self, job_id: &str) -> Result<Vec<f32>>;
}
```

### Layer 5: Persistence (Flat JSON)

```rust
// favorites.rs
pub struct FavoritesStore {
    path: PathBuf,
    favorites: HashSet<String>,
    metadata: HashMap<String, SoundMetadata>,
}

impl FavoritesStore {
    pub fn load(path: &Path) -> Self;
    pub fn toggle(&mut self, sound_id: &str, meta: SoundMetadata) -> bool;
    pub fn is_favorited(&self, sound_id: &str) -> bool;
    pub fn save(&self) -> Result<()>;
    pub fn list(&self) -> Vec<SoundMetadata>;
}
```

---

## 4. Async Job Architecture

Since the prototype uses a cloud API, generation is inherently async:

```
1. Frontend invokes: generate_sound(prompt, reference_path)
2. Rust backend:
   a. If reference: load WAV, analyze, send to cloud API
   b. Submit prompt to cloud API → get job_id
   c. Return job_id immediately
3. Frontend starts polling: get_generation_status(job_id)
4. Rust backend:
   a. Poll cloud API every 500ms
   b. On complete: download audio, run DSP (trim, normalize, fades)
   c. Compute waveform thumbnail
   d. Save WAV to ~/cShot/audio/{uuid}.wav
   e. Return SoundResult
5. Frontend: show waveform, enable play/export/fav
```

For the cut prototype (Prompt 41), this can be simplified to synchronous with a fake spinner since we're using pre-generated WAVs + local DSP. The async architecture becomes real when connecting to a real API.

---

## 5. Future Expansion Path

```
Prototype (Weeks 21-25)
├── Tauri + React + Rust
├── Local DSP (trim, normalize, fades)
├── Pre-generated WAVs + keyword matching
└── Cloud API (ready but not connected)
        │
        ▼
MVP (Prompt 39 + 40)
├── Same stack
├── Real cloud API integration
├── 6-slot grid
├── SQLite database
├── Favorites + tags
└── Library browser
        │
        ▼
Post-MVP
├── Local model inference (ONNX Runtime)
├── VST3/AU plugin (NIH-plug, reuse Rust DSP)
├── GPU-accelerated generation
├── Variation tree
└── Mix-readiness engine
```

The key insight: **Tauri + Rust + React is the only stack that preserves your investment at every stage.** The DSP code you write for the prototype goes directly into the MVP. The MVP's Rust backend becomes the core of the VST3 plugin. The React frontend adapts to the plugin UI. Nothing is thrown away.

---

## 6. Summary Recommendation

| Decision | Choice | Why |
|----------|--------|-----|
| Desktop framework | Tauri v2 | Lightest, fastest, preserves plugin path |
| Frontend | React + TypeScript + Vite | Familiar, fast iteration, Web Audio API |
| State management | useState + useEffect | Prototype doesn't need a store library |
| Styling | Tailwind CSS | Minimal overhead, easy to theme |
| Audio processing | Rust (hound + custom DSP) | Native performance, no dependencies |
| AI generation | Cloud API (not local model) | Best quality on any hardware, fast to integrate |
| Persistence | Flat JSON | Zero setup, easy to migrate to SQLite later |
| Async pattern | Frontend polling via IPC | Simple, no job queue needed |
| Build target | Developer's OS only | Ship fast, validate before expanding |

**The fastest serious stack for cShot is Tauri v2 + React/Vite + Rust + Cloud Gen API.** Build the prototype in 3 days. Connect the real API in 3 more. Then decide what to optimize.
