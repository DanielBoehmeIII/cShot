# Prompt 41 — Cut the MVP Until It Ships

Take cShot's existing MVP spec (Prompts 39-40) and aggressively reduce it to the absolute minimum that still proves the core loop.

---

## 1. The Core Loop (Everything Exists to Serve This)

```
reference audio → prompt → generated one-shot → preview → save/export
```

If a feature doesn't directly touch this loop, it gets cut. If it touches it indirectly, it gets cut. If it's nice-to-have, it gets cut.

---

## 2. What Stays

| Item | Why It Stays | Minimum Viable Form |
|------|-------------|---------------------|
| Prompt input | Entry point of the loop | Single text input, Enter to submit, no history, no autocomplete |
| Generation trigger | The entire point | One button. Click → wait → get sound. No config, no options |
| One result slot | Proves generation works | Single slot, not 6. Generate replaces it. No grid |
| Waveform thumbnail | Shows something happened | 80-point SVG rendered from float array. Pre-computed on backend |
| Instant preview | Core feedback mechanism | Click waveform → play. Web Audio API. Single sound in memory |
| WAV export | Gets sound into DAW | One button → saves to ~/Desktop/{name}.wav. No dialog, no options |
| Auto-tagging (basic) | Makes sound feel real | Sound type (kick/snare/hat/other) + 2-3 analysis tags |
| Favorites (basic) | Gives feeling of ownership | Heart toggle. Persisted in flat JSON file. No library view |
| Reference upload | Differentiator from text-only | Drag a WAV onto the prompt bar. One file. No analysis shown |

---

## 3. What Gets Cut

| Feature | Existing Reference | Why Cut |
|---------|-------------------|---------|
| 6-slot sound grid | Prompt 39 §3, Day 9 | Single slot is enough to prove generation. Grid is nice but adds layout state complexity |
| "More like this" variants | Prompt 39 §3, Day 12 | Core loop doesn't need iteration on iteration. Generate → judge → export is enough |
| Library browser | Prompt 39, Day 19 | Library of one session isn't meaningful. Favorites live in a JSON file |
| Search/filter | Prompt 39, Day 19 | Nothing to search. Cut entirely |
| Settings page | Prompt 39, Day 24 | Defaults are fine. No settings needed |
| Onboarding overlay | Prompt 39, Day 26 | The app has one input and one button. It's self-evident |
| Keyboard shortcuts | Prompt 39, Day 13 | Cut. Unnecessary complexity. Mouse-only is fine for prototype |
| Error boundaries | Prompt 39, Day 22 | Basic error handling only. No ErrorBoundary components |
| Status bar | Prompt 39, Day 14 | Cut. Unnecessary chrome |
| Detail panel | Prompt 39, Day 14 | Cut. Tags and metadata on the single slot card |
| Tag editor | Prompt 39, Day 18 | Auto-tags only. No user tag editing |
| Export history | Prompt 39, Day 11 | Cut. Export happens, you get a file. That's it |
| Dark/light theme | Prompt 39, Day 25 | Dark only. One theme |
| Database (SQLite) | Prompt 39, Day 15 | Replace with flat JSON file. Favorites and tags persist, but don't need a DB |
| Content-addressed storage | Prompt 39, §8 | Cut. Save to a flat directory with UUID filenames |
| Audio cache | Prompt 39, §8 | Cut. Single sound in memory. No need for LRU |
| Generation queue | Prompt 39, §9 | Cut. Single generation, blocking UI (with spinner) |
| Progress events | Prompt 39, §9 | Cut. Single spinner. Don't need stage-by-stage progress |
| Sound type classifier (ML) | Prompt 39, Day 6 | Rule-based. Spectral centroid + zero-crossing rate + duration thresholds |
| BPM/key detection | Prompt 39, Day 20 | Cut. Not needed for prototype |
| Batch export | Prompt 39, §5 | Cut. Single export only |
| Drag-to-export | Prompt 39, §11 | Cut. Button-based export only |
| Auto-update | Prompt 39, Day 27 | Cut. Manual download for prototype |
| Code signing | Prompt 39, Day 27 | Cut. Not shipping to general public |
| Multi-platform build | Prompt 39, Day 27 | Ship on one platform (developer's primary OS) |
| Help page | Prompt 39, Day 26 | Cut. No help needed for 3-step app |
| Demo video | Prompt 39, Day 29 | Cut. Show a friend in person |

---

## 4. What Can Be Faked

| Fake | What It Simulates | How to Fake It |
|------|-------------------|----------------|
| Model inference | Real AI generation | Pre-generate 20 WAV files from different prompts. Pick the closest match based on keyword overlap. Run real DSP (pitch shift, filter, time-stretch) to make it feel unique per prompt |
| Text embedding | Understanding the prompt | Keyword-to-config mapping: "kick" → low shelf boost + short decay envelope, "snare" → mid-band noise burst, "bright" → high shelf, "dark" → low-pass filter |
| Sound type classification | ML classifier | Derive from the keyword mapping. If prompt says "kick", type = kick. If ambiguous, measure spectral centroid |
| Generation progress | Real-time progress bar | Fake it. After click, animate a progress ring for 2-3 seconds, then reveal the sound |
| "AI is generating" | Model is running | Show spinning indicator. The DSP pipeline actually runs during this time |
| Waveform rendering | Live analysis | Pre-compute waveform data for pre-generated files. For DSP output, compute on the fly (fast enough without GPU) |

---

## 5. What Must Be Real

| Real Component | Why It Must Be Real | Minimum Viable Implementation |
|----------------|---------------------|------------------------------|
| Audio file upload | User provides their own audio | Tauri file picker (wav only). Copy file to app directory. Decode to f32 buffer |
| DSP transformation | Proves cShot can process audio | Trim silence, normalize peak, apply envelopes. Rust with hound crate |
| WAV export | User gets a usable file | hound crate writes 44.1kHz/24-bit/mono WAV to user-specified location |
| Instant preview | User hears the result immediately | Web Audio API playback. Float32 array sent from Rust via IPC |
| Prompt parsing | Connects text to sound generation | Simple keyword → parameters mapping (50 keywords, lookup table) |
| Favorites persistence | User keeps their work | Flat JSON file (~/cShot/favorites.json). Read on load, write on toggle |

---

## 6. What Validates the Idea

| Validation Question | How the Prototype Answers It | Success Signal |
|---------------------|------------------------------|----------------|
| "Can I get a usable sound from a text prompt?" | User types → gets a sound → plays it → exports it | Sound is recognizable as the requested type. User exports it and drags into DAW |
| "Is this faster than searching Splice?" | Time from prompt to preview is measured | Under 10 seconds from hitting Enter to hearing audio |
| "Does the sound fit in a mix?" | User drops exported WAV into their DAW alongside their track | Sound sits in the mix without sounding obviously synthetic or broken |
| "Would I use this in a real project?" | User's own reaction | User favs the sound, exports it, continues working with it later |
| "Is the reference audio making it better?" | User uploads a reference clip, generates a related sound | Generated sound is clearly related to the reference (same ballpark timbre, pitch, energy) |
| "Do I want to do this again?" | User generates more than once | Return rate: user generates 5+ times in a session |

---

## 7. The Cut-Down Technical Spec

### Frontend

```
Stack: React + TypeScript + Vite + Tauri v2
Pages: 1 (single page, no routing)
Components:
  - PromptBar (text input + upload button + generate button)
  - SoundCard (waveform thumbnail + type badge + play button + fav button + export button)
  - LoadingSpinner (while generating)
State: React useState + useEffect (no Zustand, no store library)
Audio: Web Audio API, single AudioContext, one buffer at a time
```

### Backend (Rust via Tauri)

```
Commands:
  - generate_sound(prompt: String, reference_path: Option<String>) → SoundResult
  - get_audio_data(sound_id: String) → Vec<f32>
  - export_wav(sound_id: String, path: String) → ExportResult
  - toggle_favorite(sound_id: String) → bool
  - get_favorites() → Vec<SoundMetadata>
  - upload_reference() → String (returns file path)
```

### Audio Processing (Rust)

```
Pipeline:
  1. Parse prompt → keyword map (simple lookup)
  2. If reference provided: analyze reference (duration, RMS, spectral centroid)
  3. Load nearest pre-generated WAV (or apply DSP to reference)
  4. Apply prompt-based DSP: EQ shelves, envelope shaping, pitch shift if needed
  5. Trim silence, normalize peak to -1dBFS
  6. Save to ~/cShot/audio/{uuid}.wav
  7. Return metadata: id, type, duration, waveform_data
```

### Persistence

```
File: ~/cShot/favorites.json
Format:
{
  "favorites": ["uuid1", "uuid2"],
  "sounds": {
    "uuid1": {
      "prompt": "punchy kick",
      "type": "kick",
      "duration_ms": 423,
      "created_at": "2025-01-15T10:30:00Z"
    }
  }
}
```

### File Structure

```
~/cShot/
├── audio/          # WAV files, flat directory
│   ├── {uuid}.wav
│   └── ...
├── favorites.json  # Persisted favorites
└── references/     # Uploaded reference audio
    └── {uuid}.wav
```

---

## 8. Comparison: Before vs. After

| Dimension | Prompt 39 MVP | This Cut |
|-----------|---------------|----------|
| Components | 25+ | 3 |
| Rust modules | 15+ | 4 |
| Frontend pages | 4 | 1 |
| Database | SQLite with migrations | Single JSON file |
| State management | Zustand stores (4) | React useState |
| Generation slots | 6 | 1 |
| Model inference | ONNX Runtime, CUDA, GPU | Pre-generated WAVs + DSP |
| Settings | Full settings page | Zero settings |
| Onboarding | Overlay + tips + help | Nothing |
| Export options | Dialog with format choices | Save to ~/Desktop |
| Build targets | macOS + Windows + Linux | Developer's OS only |
| **Estimated build time** | 30 days | **3 days** |

---

## 9. Summary

The aggressively cut MVP is a single-screen desktop app where you:

1. Type "punchy kick 140" or drag in a reference kick
2. Wait 2-3 seconds (fake progress spinner)
3. Hear a sound, see a waveform
4. Click heart to save, click export to get a WAV
5. Open it in your DAW
6. Decide: "Is this useful?"

Everything else — the grid, the library, the settings, the model — is validation-dependent. Build the 3-day prototype. Put it in front of one producer. If they smile, keep going. If they don't, the idea needs to change before you invest more.

The prototype is not the product. It's a question: *"Would you use this?"*
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
# Prompt 39 — Write the cShot MVP Technical Spec

A serious technical specification for a solo developer building toward a long-term vision.

---

## 1. Product Thesis

### One Sentence

```
cShot is a desktop application that uses AI to generate unique, mix-ready 
one-shot samples from text prompts — faster than searching a sample library.
```

### Core Insight

```
Producers spend 30-60% of their production time searching for samples.
Most of that time is wasted on:
  - Scrolling through irrelevant results
  - Preview fatigue (ears stop working after 15 sounds)
  - Settling for "good enough" because you're tired of searching
  - Processing samples to fit the mix

cShot eliminates the search entirely: type what you want, get it instantly,
uniquely, and mix-ready.
```

### MVP Scope

```
The MVP is a standalone desktop app with:
  1. Text-to-one-shot generation
  2. Instant preview
  3. Favorites and tagging
  4. WAV export
  5. Basic variation system

Non-goals for MVP:
  - DAW plugins (VST3/AU/AAX) — standalone only
  - Cloud sync / collaboration
  - Advanced sample library management
  - Latent space navigation
  - Variation tree (just basic "generate more like this")
  - Mix-readiness engine (basic normalization only)
  - Copyright safety checks (user responsibility for MVP)
  - Genre-aware processing
```

---

## 2. Target Users

### Primary Persona

```
Name: Alex
Role: Music producer (beatmaker, electronic, hip-hop)
Age: 18-35
Skill: Intermediate — knows what sounds they want, knows how DAWs work
Pain: "I spend hours looking for the right kick instead of making music"
Stack: Ableton Live, FL Studio, or Logic on a modern laptop
Need: Fast, unique, mix-ready one-shots without leaving creative flow

MVP priority: Alex wants to get a good kick/snare/hat in <10 seconds.
```

### Secondary Persona

```
Name: Jordan
Role: Sound designer (game audio, post-production)
Age: 25-45
Skill: Expert — deep understanding of audio, specific requirements
Pain: "I need 50 variations of an impact sound and I'm going to lose my mind"
Stack: Pro Tools, Reaper, FMOD/Wwise
Need: Batch generation, precise parameter control, variation systems
MVP priority: Jordan needs the variation system and precise control.
```

### Tertiary Persona

```
Name: Sam
Role: Hobbyist musician
Age: 14-25
Skill: Beginner — learning production, intimidated by synthesis
Pain: "I don't know how to design sounds, I just want them to sound good"
Stack: GarageBand, BandLab, or phone
Need: Simple interface, good defaults, educational
MVP priority: Sam needs the simplest possible experience.
```

---

## 3. Core User Flow

```
1. LAUNCH
   → App opens to prompt screen
   → "Describe the sound you want..."
   → Empty sound grid (6 slots, all showing "generate your first sound")

2. PROMPT
   → User types: "punchy trap kick 140bpm"
   → [Enter] to generate
   → Or clicks ⚡ Generate button

3. GENERATE
   → Loading state on first grid slot (progress ring, 2-5 seconds)
   → Sound appears: waveform, auto-tags, type label
   → Auto-preview plays (optional, can be toggled)
   → Remaining 5 slots fill with variants
   → Total: 6 generated sounds from one prompt

4. PREVIEW
   → Click any slot → instant playback
   → Waveform animates with playback position
   → Click different slot → crossfade to new sound
   → Space bar to play/stop

5. ITERATE
   → Click ♥ to favorite a sound
   → Click "↻ More like this" → generates 6 variants of selected sound
   → Click "↻ Cheaper" → less unique, faster generation
   → Click "↻ Wilder" → more experimental variants

6. EXPORT
   → Click ↓ on any sound
   → Choose: WAV 44.1kHz/24-bit (default)
   → File appears in ~/cShot/exports/ or chosen location
   → Or drag sound to Finder desktop
```

---

## 4. Frontend Architecture

### Tech Stack

```
Framework: Tauri v2 (Rust backend + WebView frontend)
Frontend: React 18 + TypeScript + Vite
State: Zustand (lightweight, TypeScript-first)
Audio Player: Web Audio API (via Tauri's WebView)
Routing: React Router (settings, library, about pages)
Styling: Tailwind CSS + custom design system
Icons: Lucide React (open-source icon set)
Graphs: React Flow (for variation tree in post-MVP)
Testing: Vitest + React Testing Library
Build: Vite → Tauri build (macOS .dmg, Windows .exe, Linux .AppImage)
```

### Why Tauri

```
Compared to Electron:
  - 10x smaller bundle (~5MB vs ~150MB)
  - 2x lower memory usage (~80MB vs ~200MB)
  - Rust backend enables audio processing without Node.js overhead
  - Direct system audio access via Rust crates
  - Native file system access without permissions layer
  
Compared to native (Swift/Electron):
  - Single codebase for all platforms
  - Web-based UI is faster to iterate (the solo developer advantage)
  - Rust provides native performance for audio processing
  - Can ship VST3 plugin later (via Rust VST3 crate, e.g., NIH-plug)
```

### Frontend Structure

```
src/
├── App.tsx                    # Root with router
├── main.tsx                   # Entry point
├── vite-env.d.ts
│
├── components/
│   ├── layout/
│   │   ├── TopBar.tsx         # Logo, settings, mode toggle
│   │   └── StatusBar.tsx      # Generation status, model info, audio meter
│   │
│   ├── prompt/
│   │   ├── PromptBar.tsx      # Text input + generate button
│   │   ├── PromptHistory.tsx  # Recent prompts dropdown
│   │   └── ReferenceDropZone.tsx  # Drag audio reference
│   │
│   ├── grid/
│   │   ├── SoundGrid.tsx      # 6-slot grid container
│   │   ├── SoundSlot.tsx      # Individual sound card
│   │   ├── WaveformThumbnail.tsx  # Mini waveform SVG
│   │   └── SlotControls.tsx   # Play, fav, export, variant buttons
│   │
│   ├── detail/
│   │   ├── DetailPanel.tsx    # Full sound detail sidebar
│   │   ├── WaveformViewer.tsx # Interactive waveform + spectrogram
│   │   ├── TagEditor.tsx      # View/edit tags
│   │   └── ProvenanceCard.tsx # Simple provenance summary
│   │
│   ├── export/
│   │   ├── ExportDialog.tsx   # Export settings
│   │   └── ExportHistory.tsx  # Recent exports
│   │
│   └── shared/
│       ├── Button.tsx
│       ├── Slider.tsx
│       ├── Modal.tsx
│       ├── Toast.tsx
│       └── Spinner.tsx
│
├── stores/
│   ├── useGenerationStore.ts  # Generation state, current sounds
│   ├── useLibraryStore.ts     # Favorites, tags, library
│   ├── useSettingsStore.ts    # User preferences
│   └── useAudioStore.ts       # Playback state
│
├── hooks/
│   ├── useAudioPlayback.ts    # Play/stop/preview sounds
│   ├── useGeneration.ts       # Call generation API
│   ├── useExport.ts           # Export to file
│   └── useKeyboard.ts         # Keyboard shortcuts
│
├── lib/
│   ├── api.ts                 # Tauri invoke() bindings
│   ├── audio.ts               # Web Audio utilities
│   ├── format.ts              # File naming, formatting
│   └── constants.ts           # App constants
│
└── styles/
    ├── globals.css            # Base styles, variables
    └── design-system.ts       # Design tokens
```

### Key Components Detail

#### SoundSlot.tsx

```typescript
interface SoundSlotProps {
  id: string | null;           // null = empty slot
  index: number;
  isPlaying: boolean;
  isFavorited: boolean;
  waveform: number[];          // Pre-computed waveform (80 samples)
  type: SoundType;
  tags: string[];
  onPlay: (id: string) => void;
  onFavorite: (id: string) => void;
  onVariant: (id: string) => void;
  onExport: (id: string) => void;
  onSelect: (id: string) => void;
  isSelected: boolean;
  generationProgress: number | null;  // 0-100 or null if done
}
```

#### SoundGrid.tsx

```typescript
interface SoundGridState {
  slots: (SoundData | null)[];  // 6 slots
  selectedId: string | null;
  playingId: string | null;
}

interface SoundData {
  id: string;
  waveform: number[];
  type: SoundType;
  tags: string[];
  duration: number;
  isFavorited: boolean;
  audioPath: string;           // Path to cached audio file
  createdAt: string;
  prompt: string;
  seed: number;
}
```

### Frontend State Management

```typescript
// generationStore.ts
interface GenerationState {
  prompt: string;
  isGenerating: boolean;
  progress: number | null;       // 0-100
  currentBatch: SoundData[] | null;
  selectedSoundId: string | null;
  playingSoundId: string | null;
  
  // Actions
  setPrompt: (prompt: string) => void;
  generate: () => Promise<void>;
  generateVariants: (soundId: string) => Promise<void>;
  selectSound: (id: string | null) => void;
  clearGrid: () => void;
}

// libraryStore.ts
interface LibraryState {
  favorites: SoundData[];
  recentExports: ExportRecord[];
  tags: string[];
  
  // Actions
  toggleFavorite: (sound: SoundData) => void;
  addTag: (soundId: string, tag: string) => void;
  removeTag: (soundId: string, tag: string) => void;
}
```

---

## 5. Backend Architecture (Rust/Tauri)

### Rust Backend Structure

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── src/
│   ├── main.rs                  # Tauri entry point
│   ├── lib.rs                   # Plugin registration
│   │
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── generation.rs        # generate_sound, generate_variants
│   │   ├── playback.rs          # get_audio_data
│   │   ├── export.rs            # export_wav
│   │   └── library.rs           # favorites, tags, search
│   │
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── generate.rs          # Model inference orchestration
│   │   ├── dsp.rs               # Post-processing (trim, normalize, fade)
│   │   ├── analyze.rs           # Sound type classification, BPM detection
│   │   └── export.rs            # WAV file writing
│   │
│   ├── model/
│   │   ├── mod.rs
│   │   ├── loader.rs            # Model loading and management
│   │   ├── inference.rs         # ONNX Runtime inference
│   │   └── types.rs             # Model output types
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── library.rs           # SQLite operations
│   │   └── migrations.rs        # Schema management
│   │
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── files.rs             # Content-addressed file storage
│   │   └── cache.rs             # LRU cache for generated audio
│   │
│   └── config/
│       ├── mod.rs
│       └── settings.rs          # User settings management
```

### Key Rust Dependencies

```toml
[dependencies]
tauri = { version = "2", features = ["dialog", "fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }

# Audio
hound = "3"                      # WAV read/write
symphonia = "0.5"                # Audio decoding (for imports)

# Inference
ort = "2"                        # ONNX Runtime bindings

# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# Utilities
uuid = { version = "1", features = ["v4"] }
sha2 = "0.10"
chrono = { version = "0.4", features = ["serde"] }
log = "0.4"
thiserror = "1"
```

### Tauri Commands (IPC)

```typescript
// Commands exposed to frontend via @tauri-apps/api

// Generation
invoke('generate_sound', { 
  prompt: string,
  config: { 
    seed?: number,
    temperature?: number,
    type?: SoundType,
    bpm?: number,
    key?: string,
    duration_ms?: number 
  }
}) => Promise<SoundResult>

invoke('generate_variants', {
  source_id: string,
  count: number  // how many variants to generate
}) => Promise<SoundResult[]>

// Audio
invoke('get_audio_data', {
  sound_id: string,
  format: 'wav' | 'float32'
}) => Promise<number[]>  // Returns audio samples for waveform display

invoke('get_waveform_thumbnail', {
  sound_id: string,
  width: number,
  height: number
}) => Promise<number[]>  // Returns downsampled waveform points

// Export
invoke('export_wav', {
  sound_id: string,
  path: string,           // Directory to export to
  filename?: string,       // Custom filename (optional)
  sample_rate?: number,    // Default: 44100
  bit_depth?: number       // Default: 24
}) => Promise<ExportResult>

invoke('export_batch', {
  sound_ids: string[],
  directory: string
}) => Promise<ExportResult[]>

// Library
invoke('get_favorites') => Promise<SoundMetadata[]>
invoke('toggle_favorite', { sound_id: string }) => Promise<void>
invoke('add_tag', { sound_id: string, tag: string }) => Promise<void>
invoke('remove_tag', { sound_id: string, tag: string }) => Promise<void>
invoke('search_library', { query: string }) => Promise<SoundMetadata[]>

// Settings
invoke('get_settings') => Promise<AppSettings>
invoke('update_settings', { settings: AppSettings }) => Promise<void>
```

---

## 6. Audio Processing Pipeline

### Generation Pipeline

```
┌────────────┐
│  Prompt    │  "punchy trap kick 140bpm"
└─────┬──────┘
      │
      ▼
┌──────────────┐
│  Text →      │  Encode prompt to embedding (768-d)
│  Embedding   │  Using CLAP-style text encoder (ONNX)
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Diffusion   │  Generate audio from embedding
│  Model       │  50 steps, DDIM sampler
│              │  Output: mono Float32 @ 44100Hz
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Post-       │  Trim silence
│  Process     │  Normalize peak to -1dB
│              │  Apply fade in/out
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Analyze     │  Detect: type (kick/snare/hat/...)
│              │         duration, RMS, peak
│              │         auto-tags (dark, punchy, etc.)
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Save        │  Write to content-addressed storage
│              │  Create DB record
│              │  Return metadata to frontend
└──────────────┘
```

### Generation Configuration

| Parameter | Default | Range | Description |
|-----------|---------|-------|-------------|
| inference_steps | 50 | 10-100 | More steps = higher quality, slower |
| sampler | DDIM | DDIM, PNDM, DPM++ | Sampler algorithm |
| cfg_scale | 7.5 | 1.0-15.0 | How closely to follow prompt |
| temperature | 1.0 | 0.5-2.0 | Randomness in generation |
| seed | random | any u32 | Reproducibility |
| duration_ms | 1000 | 100-5000 | Target sound duration |
| sample_rate | 44100 | 22050-96000 | Output sample rate |

### Post-Processing

```rust
pub fn post_process(audio: &mut [f32], sample_rate: u32) -> ProcessingResult {
    // 1. Remove DC offset
    let dc_offset = audio.iter().sum::<f32>() / audio.len() as f32;
    for sample in audio.iter_mut() {
        *sample -= dc_offset;
    }
    
    // 2. Trim leading silence (< -60dB)
    let threshold = 0.001;  // -60dB
    let start = audio.iter().position(|&s| s.abs() > threshold).unwrap_or(0);
    let end = audio.iter().rposition(|&s| s.abs() > threshold).unwrap_or(audio.len() - 1);
    let trimmed = &mut audio[start..=end];
    
    // 3. Apply fade in/out (2ms in, 10ms out)
    let fade_in_len = (0.002 * sample_rate as f32) as usize;
    let fade_out_len = (0.010 * sample_rate as f32) as usize;
    
    for i in 0..fade_in_len.min(trimmed.len()) {
        trimmed[i] *= (i as f32 / fade_in_len as f32);
    }
    for i in 0..fade_out_len.min(trimmed.len()) {
        let idx = trimmed.len() - 1 - i;
        trimmed[idx] *= (i as f32 / fade_out_len as f32);
    }
    
    // 4. Normalize peak to -1dB
    let peak = trimmed.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    let target_peak = 10f32.powf(-1.0 / 20.0);  // -1dB
    let gain = target_peak / peak;
    for sample in trimmed.iter_mut() {
        *sample *= gain;
    }
    
    ProcessingResult {
        dc_offset_removed: dc_offset.abs() > 0.001,
        silence_trimmed: start > 0 || end < audio.len() - 1,
        peak_normalized_to: -1.0,
        final_duration_ms: trimmed.len() as f32 / sample_rate as f32 * 1000.0,
    }
}
```

### Audio Analysis

```rust
pub struct AudioAnalysis {
    pub sound_type: SoundType,     // kick, snare, hat, clap, perc, bass, fx, other
    pub duration_ms: f32,
    pub rms: f32,                  // Root mean square amplitude
    pub peak: f32,                 // Peak amplitude
    pub spectral_centroid: f32,    // Brightness measure
    pub estimated_pitch: Option<f32>,  // Estimated fundamental frequency
    pub tags: Vec<String>,         // Auto-generated tags
    pub embedding: Vec<f32>,       // 768-d Sound DNA embedding
}

pub enum SoundType {
    Kick, Snare, HiHat, Clap, Percussion, Bass, Fx, Other
}

pub fn analyze(audio: &[f32], sample_rate: u32) -> AudioAnalysis {
    // Sound type classification using lightweight CNN
    let sound_type = classify_sound_type(audio, sample_rate);
    
    // Basic metrics
    let rms = compute_rms(audio);
    let peak = compute_peak(audio);
    let spectral_centroid = compute_spectral_centroid(audio, sample_rate);
    
    // Duration
    let duration_ms = audio.len() as f32 / sample_rate as f32 * 1000.0;
    
    // Tags from analysis
    let mut tags = Vec::new();
    if spectral_centroid > 4000.0 { tags.push("bright".to_string()); }
    if spectral_centroid < 1000.0 { tags.push("dark".to_string()); }
    if rms > 0.3 { tags.push("loud".to_string()); }
    if rms < 0.1 { tags.push("quiet".to_string()); }
    if duration_ms < 200.0 { tags.push("short".to_string()); }
    if duration_ms > 2000.0 { tags.push("long".to_string()); }
    
    // Embedding
    let embedding = compute_embedding(audio, sample_rate);
    
    AudioAnalysis {
        sound_type,
        duration_ms,
        rms,
        peak,
        spectral_centroid,
        estimated_pitch: None,  // Requires pitch detection model
        tags,
        embedding,
    }
}
```

---

## 7. Model / API Choices

### Generation Model

```
Primary Approach: Fine-tuned open-source diffusion model

Model: AudioLDM 2 (or Stable Audio Open, whichever is more capable at MVP time)
  - Generate raw audio from text prompts
  - ~800M parameters
  - Output: mono 44.1kHz audio
  
Fine-tuning:
  - Train on CC0 / public domain one-shot dataset
  - Focus on short sounds (0.1-5 seconds)
  - Add sound type classifier as auxiliary output
  
Edge Cases:
  - Very short prompts (<3 words): prepend genre/sound type
  - Very long prompts (>50 words): truncate to key tokens
  - Non-sound descriptions: fallback to generic defaults
  
Inference:
  - ONNX Runtime with CUDA provider
  - CPU fallback with x86 optimizations
  - Target: 2-5 seconds generation on RTX 3060
  - Target: 10-20 seconds on M1 Mac
  - Target: 30-60 seconds on CPU-only (acceptable for MVP but slow)
```

### Model Variants (MVP)

```
For MVP, use a smaller distilled model for speed:

  Model A: "Fast" — 4-step LCM/LDM distilled variant
    - Quality: 7/10
    - Speed: 0.5-1 second on GPU
    - Use: Initial generation, "cheap" mode
  
  Model B: "Quality" — Full 50-step diffusion
    - Quality: 9/10
    - Speed: 2-5 seconds on GPU
    - Use: "I like that, make it better" refinement

Future: Both models can evolve independently
```

### Sound Type Classifier

```
Architecture: Lightweight CNN (MobileNet-style for audio)
  - Input: Mel-spectrogram (64x64)
  - Output: SoundType classification + tag predictions
  - Size: ~5MB
  - Speed: <10ms inference
  - Training: Labeled one-shot dataset
  
Labels:
  - Primary: kick, snare, hi-hat (closed), hi-hat (open), clap,
             rimshot, tom, percussion, bass hit, FX, vocal,
             synth stab, impact, foley, other
  - Attributes: punchy, soft, bright, dark, short, long,
                metallic, wooden, noisy, clean, distorted
```

### Database Schema

```sql
-- Core tables
CREATE TABLE sounds (
    id TEXT PRIMARY KEY,              -- UUID v4
    audio_hash TEXT NOT NULL UNIQUE,  -- SHA-256 of audio
    prompt TEXT NOT NULL,             -- Original prompt
    seed INTEGER NOT NULL,
    model_version TEXT NOT NULL,
    params_json TEXT NOT NULL,        -- Generation parameters
    
    -- Audio metadata
    duration_ms REAL NOT NULL,
    sample_rate INTEGER NOT NULL DEFAULT 44100,
    channels INTEGER NOT NULL DEFAULT 1,
    rms REAL,
    peak REAL,
    
    -- Analysis
    sound_type TEXT,                  -- kick, snare, hat, etc.
    spectral_centroid REAL,
    
    -- User state
    is_favorite INTEGER DEFAULT 0,
    notes TEXT,
    
    -- Timestamps
    created_at TEXT NOT NULL,         -- ISO 8601
    exported_at TEXT,                 -- Last export time
    last_played_at TEXT,
    
    -- File path
    relative_path TEXT NOT NULL       -- Relative to library root
);

CREATE TABLE tags (
    sound_id TEXT NOT NULL REFERENCES sounds(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'auto',  -- 'auto', 'user'
    confidence REAL DEFAULT 1.0,
    PRIMARY KEY (sound_id, tag)
);

CREATE TABLE exports (
    id TEXT PRIMARY KEY,
    sound_id TEXT NOT NULL REFERENCES sounds(id),
    file_path TEXT NOT NULL,
    format TEXT NOT NULL DEFAULT 'wav',
    sample_rate INTEGER,
    bit_depth INTEGER,
    exported_at TEXT NOT NULL
);

CREATE TABLE generation_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    prompt TEXT NOT NULL,
    seed INTEGER,
    model_version TEXT,
    duration_ms REAL,              -- Generation time
    success INTEGER DEFAULT 1,
    error_message TEXT,
    created_at TEXT NOT NULL
);

-- Indexes
CREATE INDEX idx_sounds_type ON sounds(sound_type);
CREATE INDEX idx_sounds_favorite ON sounds(is_favorite);
CREATE INDEX idx_sounds_created ON sounds(created_at);
CREATE INDEX idx_tags_sound ON tags(sound_id);
CREATE INDEX idx_tags_tag ON tags(tag);
```

---

## 8. File Storage

### Directory Structure

```
~/cShot/
├── library.db                   # SQLite database
├── audio/                       # Content-addressed audio files
│   └── {hash_prefix}/
│       └── {full_hash}.wav      # e.g., /audio/a1/b2/a1b2c3d4....wav
├── exports/                     # User export directory
│   └── {date}/
│       └── {type}_{bpm}_{key}_{seed}.wav
├── models/                      # Downloaded model files
│   ├── fast/
│   │   └── model.onnx
│   ├── quality/
│   │   └── model.onnx
│   └── classifier.onnx
├── cache/                       # Temporary files
│   └── waveforms/               # Pre-computed thumbnail waveforms
└── config.toml                  # User preferences
```

### Content-Addressed Storage

```rust
pub struct ContentAddressedStorage {
    base_path: PathBuf,
}

impl ContentAddressedStorage {
    pub fn store(&self, audio: &[f32], sample_rate: u32) -> Result<String> {
        // Compute SHA-256
        let hash = sha256(audio);
        let hash_hex = hex::encode(hash);
        
        // Create path: /audio/{first 2}/{next 2}/{full hash}.wav
        let dir = self.base_path.join("audio")
            .join(&hash_hex[0..2])
            .join(&hash_hex[2..4]);
        std::fs::create_dir_all(&dir)?;
        
        let path = dir.join(format!("{}.wav", hash_hex));
        
        // Only write if doesn't exist (dedup)
        if !path.exists() {
            self.write_wav(&path, audio, sample_rate)?;
        }
        
        Ok(hash_hex)
    }
    
    pub fn load(&self, hash: &str) -> Result<Vec<f32>> {
        let path = self.base_path.join("audio")
            .join(&hash[0..2])
            .join(&hash[2..4])
            .join(format!("{}.wav", hash));
        
        self.read_wav(&path)
    }
}
```

### Cache Strategy

```rust
pub struct AudioCache {
    /// LRU cache of generated audio buffers
    cache: LruCache<String, Vec<f32>>,
    max_memory_mb: usize,
}

impl AudioCache {
    pub fn new(max_memory_mb: usize) -> Self {
        // Estimate: 10 seconds of 44.1kHz f32 mono = ~1.7MB
        let max_entries = max_memory_mb * 1024 * 1024 / (10 * 44100 * 4);
        Self {
            cache: LruCache::new(max_entries),
            max_memory_mb,
        }
    }
    
    pub fn get(&mut self, sound_id: &str) -> Option<&Vec<f32>> {
        self.cache.get(sound_id)
    }
    
    pub fn insert(&mut self, sound_id: String, audio: Vec<f32>) {
        self.cache.put(sound_id, audio);
    }
    
    /// Pre-warm cache with recently played/favorited sounds
    pub fn warm(&mut self, db: &Library, favorites: &[String]) {
        for id in favorites {
            if !self.cache.contains(id) {
                if let Ok(audio) = load_sound_from_disk(id) {
                    self.cache.put(id.clone(), audio);
                }
            }
        }
    }
}
```

---

## 9. Job Queue

### Architecture

```
Generation requests are handled asynchronously to keep UI responsive.

Frontend              Backend                        Model
   │                     │                             │
   │  invoke('gen...')   │                             │
   │────────────────────→│                             │
   │                     │  Queue job                  │
   │                     │  Return job_id              │
   │←────────────────────│                             │
   │                     │                             │
   │  listen('progress') │  Process job                │
   │←───── progress ─────│────────────────────────────→│
   │  (events)           │  Inference...               │
   │                     │  Post-process...            │
   │                     │  Save...                    │
   │                     │                             │
   │  listen('complete') │  Job done                   │
   │←───── result ───────│                             │
```

### Backend Implementation

```rust
pub struct GenerationQueue {
    queue: Arc<Mutex<VecDeque<GenerationJob>>>,
    worker_handle: JoinHandle<()>,
    event_emitter: EventEmitter,
}

struct GenerationJob {
    id: String,
    prompt: String,
    config: GenerationConfig,
    created_at: Instant,
}

impl GenerationQueue {
    pub fn submit(&self, prompt: String, config: GenerationConfig) -> String {
        let job_id = Uuid::new_v4().to_string();
        
        let job = GenerationJob {
            id: job_id.clone(),
            prompt,
            config,
            created_at: Instant::now(),
        };
        
        self.queue.lock().unwrap().push_back(job);
        job_id
    }
    
    async fn worker_loop(&self) {
        loop {
            let job = self.queue.lock().unwrap().pop_front();
            
            if let Some(job) = job {
                // Emit progress events
                self.event_emitter.emit("generation_progress", &ProgressEvent {
                    job_id: job.id.clone(),
                    progress: 0,
                    stage: "starting",
                });
                
                // Run inference
                let result = generate_audio(&job.prompt, &job.config).await;
                
                match result {
                    Ok(audio) => {
                        // Post-process
                        let processed = post_process(&mut audio, 44100);
                        
                        // Save
                        let hash = storage.store(&audio, 44100);
                        
                        // Analyze
                        let analysis = analyze(&audio, 44100);
                        
                        // Create DB record
                        let sound_id = db.insert_sound(SoundRecord {
                            audio_hash: hash,
                            prompt: job.prompt,
                            seed: job.config.seed,
                            // ...
                        });
                        
                        // Emit complete
                        self.event_emitter.emit("generation_complete", &CompleteEvent {
                            job_id: job.id,
                            sound_id,
                            analysis,
                        });
                    }
                    Err(e) => {
                        self.event_emitter.emit("generation_error", &ErrorEvent {
                            job_id: job.id,
                            error: e.to_string(),
                        });
                    }
                }
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
```

### Concurrency

```
Single generation queue (MVP):
  - One job at a time (GPU memory limitation)
  - Queue: FIFO
  - Max queue depth: 10 (frontend prevents more)
  - Timeout: 60 seconds per job
  
Future:
  - Parallel CPU/GPU scheduling
  - Batch generation (multiple sounds from one prompt)
  - Priority queue (preview > export > batch)
```

---

## 10. Preview System

### Architecture

```
Frontend needs low-latency audio playback without loading entire files.

Strategy:
  1. On generation complete: audio file written to disk
  2. Frontend requests audio data as base64 Float32 array
  3. Frontend caches in memory (Web Audio API AudioBuffer)
  4. Playback via Web Audio API (sub-5ms latency)
  
Audio transfer:
  - invoke('get_audio_data', { sound_id }) → number[]
  - ~4-40KB for waveform display data (downsampled)
  - Full audio only loaded on play
  
Caching:
  - LRU cache in frontend (keep last 20 played sounds in memory)
  - ~1.7MB per 10-second mono sound at 44.1kHz
  - Max: ~35MB cache (reasonable for desktop app)
```

### Frontend Playback

```typescript
// useAudioPlayback.ts
export function useAudioPlayback() {
  const audioContext = useRef<AudioContext | null>(null);
  const buffers = useRef<Map<string, AudioBuffer>>(new Map());
  const source = useRef<AudioBufferSourceNode | null>(null);
  
  const getContext = () => {
    if (!audioContext.current) {
      audioContext.current = new AudioContext();
    }
    return audioContext.current;
  };
  
  const loadAudio = async (soundId: string): Promise<AudioBuffer> => {
    const cached = buffers.current.get(soundId);
    if (cached) return cached;
    
    // Request audio data from backend
    const audioData = await invoke('get_audio_data', { sound_id: soundId });
    
    // Decode to AudioBuffer
    const ctx = getContext();
    const float32 = new Float32Array(audioData);
    const buffer = ctx.createBuffer(1, float32.length, 44100);
    buffer.getChannelData(0).set(float32);
    
    // Cache
    buffers.current.set(soundId, buffer);
    
    // Evict oldest if cache too large
    if (buffers.current.size > 20) {
      const oldest = buffers.current.keys().next().value;
      buffers.current.delete(oldest);
    }
    
    return buffer;
  };
  
  const play = async (soundId: string) => {
    stop();
    
    const ctx = getContext();
    const buffer = await loadAudio(soundId);
    
    const src = ctx.createBufferSource();
    src.buffer = buffer;
    src.connect(ctx.destination);
    src.start(0);
    
    source.current = src;
  };
  
  const stop = () => {
    source.current?.stop();
    source.current = null;
  };
  
  return { play, stop, loadAudio };
}
```

---

## 11. Export System

### WAV Export

```rust
pub fn export_wav(
    sound_id: &str,
    output_path: &Path,
    sample_rate: u32,
    bit_depth: u16,
) -> Result<ExportMetadata> {
    // Load audio from content-addressed storage
    let audio = storage.load(hash_from_db(sound_id))?;
    
    // Resample if needed
    let audio = if sample_rate != 44100 {
        resample(&audio, 44100, sample_rate)
    } else {
        audio
    };
    
    // Convert bit depth
    let samples: Vec<i32> = match bit_depth {
        16 => audio.iter().map(|&s| (s * i16::MAX as f32) as i32).collect(),
        24 => audio.iter().map(|&s| (s * 8388607.0) as i32).collect(),
        32 => audio.iter().map(|&s| (s * i32::MAX as f32) as i32).collect(),
        _ => return Err(ExportError::UnsupportedBitDepth),
    };
    
    // Write WAV
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: bit_depth,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create(output_path, spec)?;
    for &sample in &samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    
    // Generate filename from metadata
    let analysis = db.get_analysis(sound_id)?;
    let filename = format!(
        "{}_{}_{}bpm_{}_{}.wav",
        analysis.sound_type,
        analysis.tags.first().unwrap_or(&"sound".to_string()),
        analysis.bpm.unwrap_or(0),
        analysis.key.unwrap_or("Unknown"),
        &sound_id[..8]
    );
    
    Ok(ExportMetadata {
        file_path: output_path.to_path_buf(),
        filename,
        duration_ms: (audio.len() as f64 / sample_rate as f64 * 1000.0) as u64,
        sample_rate,
        bit_depth,
        file_size_bytes: samples.len() * (bit_depth as usize / 8),
    })
}
```

### Drag-to-Export (Desktop)

```
When user drags a sound from the app to Finder/Desktop:

  1. Frontend detects HTML5 drag event on SoundSlot
  2. Sets drag data as the sound_id (string)
  3. On drop, frontend calls invoke('export_temp', { sound_id })
  4. Backend exports to temp file at drop location
  5. Returns { path, filename }
  6. Frontend shows success toast
  
Drag formats:
  - WAV: Standard drag of file URL
  - macOS: Promise file provider (file appears after drop)
```

---

## 12. Evaluation Metrics

### MVP Success Metrics

```
User-facing:
  - Time from prompt to first preview: <5 seconds
  - Time from prompt to export: <10 seconds
  - Sound quality: 7/10 average user rating
  - Generation success rate: >95% (no crashes, no silent output)
  - App cold start: <3 seconds
  - Memory usage: <500MB idle, <2GB generating

Technical:
  - Model inference: <5s on RTX 3060, <20s on M1, <60s on CPU
  - Pipeline latency: <100ms post-processing
  - Export: <50ms WAV write (including resample)
  - DB queries: <10ms
  - Crash rate: <1% of sessions

Business:
  - Favorites per session: >3
  - Exports per session: >2
  - Return rate: >40% day-7 retention
  - Time spent in app: >10 min/session
```

### Quality Metrics (for model evaluation)

```
Objective:
  - FAD (Fréchet Audio Distance) vs. professional one-shot dataset
  - CLAP score (text-audio alignment)
  - SNR (signal-to-noise ratio)
  - Peak clipping rate: <1% of generated sounds

Subjective:
  - User preference in A/B test vs. Splice samples (target: 50%+)
  - Perceived quality rating (1-5 scale, target: 3.5+)
  - "Would you use this in a track?" (target: >70% yes)
```

### Testing Method

```
For each daily build:
  1. Generate 100 sounds from 10 prompt templates
  2. Check: 0 crashes, 0 silent outputs, <10 outliers (>30s generation)
  3. Check: no clipping, all have reasonable length (50ms-5s)
  4. Spot-check 10 sounds for quality

For weekly release:
  1. Generate 1000 sounds across 50 prompts
  2. FAD calculation against reference dataset
  3. 5 users blind-test 20 pairs (cShot vs. Splice)
  4. Survey: "Would you use this?"
```

---

## 13. Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Model generates poor quality audio | Medium | Critical | Use distilled model for reliability, fallback to DSP templates |
| Generation too slow (<30fps feel) | Medium | High | Progressive display, show placeholder waveforms |
| GPU memory exhaustion | Low | High | Graceful fallback, CPU fallback path, memory budget |
| ONNX Runtime compatibility | Medium | Medium | Pin versions, test on Windows/macOS/Linux |
| Model overfitting to training data | Medium | High | Dedup training data, similarity check on output |
| User confusion with controls | Medium | Medium | Beginner mode, tooltips, limited initial surface area |
| Large download size | Low | Medium | Model download on first launch, progress UI |
| Silent/short/truncated outputs | Low | High | Post-processing detection, auto-retry |
| Cross-platform WAV compatibility | Low | Low | Use hound crate (battle-tested) |
| Solo developer burnout | High | Critical | Aggressive scoping, focus on working prototype not perfection |

---

## 14. Phased Implementation Plan

### Phase 0: Foundation (Days 1-7)

```
Goal: Working app skeleton with Tauri + generation from hardcoded prompts

Day 1-2: Project setup
  - Initialize Tauri + React project
  - Set up build pipeline (Vite, Tauri config)
  - Implement basic window, menu, and file dialogs
  - Set up Rust crate structure

Day 3-4: Model integration
  - Wire up ONNX Runtime in Rust
  - Implement model loading from local file
  - Implement basic inference (hardcoded prompt)
  - Test: generate single sound, save to WAV

Day 5-7: Audio pipeline
  - Implement post-processing (trim, normalize, fade)
  - Implement WAV export
  - Implement basic analysis (duration, RMS, peak)
  - Test: end-to-end generation → save → analyze
```

### Phase 1: Frontend MVP (Days 8-14)

```
Goal: Working UI with prompt input, sound grid, preview, export

Day 8-9: UI shell
  - Implement TopBar, StatusBar, PromptBar components
  - Implement SoundGrid + SoundSlot components
  - Wire up Tauri IPC commands from frontend
  - Test: UI renders, buttons fire events

Day 10-11: Generation flow
  - Implement generate_sound IPC command
  - Frontend generation state (loading → complete)
  - Progress indicator on grid slots
  - Test: type prompt → click generate → see waveform

Day 12-14: Preview + Export
  - Implement audio playback (Web Audio API)
  - Implement get_audio_data IPC command
  - Wire up play/stop per sound slot
  - Implement export flow
  - Test: generate → preview → export → play in DAW
```

### Phase 2: Library & Iteration (Days 15-21)

```
Goal: Favorites, tags, "more like this" variants

Day 15-16: Database
  - Implement SQLite in Rust
  - Create schema, migrations
  - Implement CRUD operations
  - Test: save/load sounds from DB

Day 17-18: Favorites + Tags
  - Implement favorite toggle UI
  - Implement tag add/remove UI
  - Build favorites view
  - Test: favorite → restart → favorites persist

Day 19-21: Variants
  - Implement generate_variants (same prompt, different seeds)
  - Variant button on sound slot
  - Generate 6 variants, fill grid
  - Test: generate → variants → each sound is different
```

### Phase 3: Polish & Release (Days 22-30)

```
Goal: Beta-quality release

Day 22-23: Quality improvements
  - Error handling (generation failures, model loading issues)
  - Graceful degradation (CPU fallback)
  - Progress events for generation
  - Test: all error paths

Day 24-25: Performance
  - Optimize model inference (batch processing, GPU memory)
  - Implement audio cache
  - Profile and fix UI jank
  - Target: <5s generation on target GPU

Day 26-27: UX Polish
  - Implement keyboard shortcuts
  - Add loading states and transitions
  - Dark theme consistency
  - About/settings page
  - Test: full user flow without mouse

Day 28-29: Packaging
  - Tauri build configuration for macOS/Windows/Linux
  - Code signing (macOS), installer (Windows)
  - Auto-update mechanism
  - Test: clean install on all platforms

Day 30: Release
  - Generate test prompts and verify quality
  - Write quick-start guide
  - Create demo video
  - Ship v0.1.0-beta
```

---

## 15. Technology Decision Log

| Decision | Choice | Alternatives Considered | Rationale |
|----------|--------|------------------------|-----------|
| Desktop framework | Tauri v2 | Electron, Flutter, SwiftUI | Lightest weight, Rust backend, cross-platform |
| Frontend | React + TypeScript | Vue, Svelte, Solid | Developer familiarity, ecosystem, type safety |
| State management | Zustand | Redux, Jotai, MobX | Minimal boilerplate, TS-first |
| Model runtime | ONNX Runtime | TensorFlow Lite, llama.cpp, custom CUDA | Broad hardware support, stable, well-documented |
| Audio I/O | Web Audio API | CPAL (Rust), JUCE | Zero additional overhead (built into WebView) |
| WAV encoding | hound (Rust) | symphonia, rodio | Simple, pure Rust, no system deps |
| Database | SQLite (rusqlite) | Sled, RocksDB, JSON files | Reliable, zero-config, queryable |
| Content addressing | SHA-256 | Custom hashing, UUID-based | Standard, dedup, integrity verification |
| IPC | Tauri invoke + events | gRPC, custom socket | Built-in, typed, async |
| Prompt embedding | CLAP-style (ONNX) | Custom embedding model | Pre-trained, available, effective |

---

## Summary

| Dimension | MVP Decision |
|-----------|-------------|
| Scope | Standalone desktop app, text-to-one-shot, preview, export |
| Stack | Tauri + React + Rust + ONNX Runtime |
| Model | Fine-tuned AudioLDM 2 (fast + quality variants) |
| Storage | Content-addressed + SQLite |
| UI | 6-slot grid, prompt bar, playback, library |
| Timeline | 30 days to beta |
| Distribution | Direct download (.dmg/.exe/.AppImage) |
| Monetization | MVP is free (data collection for model improvement) |

The MVP is not the vision. The MVP is the smallest thing that proves the core insight: generating a unique, usable sound from a text prompt is faster and better than searching a sample library.
