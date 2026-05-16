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
