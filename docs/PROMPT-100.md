# Prompt 100 — cShot v1 Engineering Spec

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        cShot Desktop App                             │
│                                                                     │
│  ┌───────────────────────┐          ┌─────────────────────────────┐ │
│  │   Frontend (WebView)   │  IPC     │    Rust Backend (Tauri)     │ │
│  │                       │◄────────►│                             │ │
│  │  React 18 + TypeScript │  invoke  │  - Audio Pipeline           │ │
│  │  Vite + Tailwind CSS   │  events  │  - Library Manager          │ │
│  │  Zustand (state)       │          │  - Model Gateway Client     │ │
│  │  Web Audio API (play)  │          │  - DSP Engine               │ │
│  │  Canvas (waveforms)    │          │  - Content-Addressed Store   │ │
│  └───────────────────────┘          │  - FAISS Vector Search       │ │
│                                      │  - Export Service            │ │
│                                      └──────────┬──────────────────┘ │
└─────────────────────────────────────────────────┼───────────────────┘
                                                  │
                    ┌─────────────────────────────┼─────────────────────┐
                    │        Cloud Services        │                     │
                    │                             │                     │
                    │  ┌──────────────────────┐   │                     │
                    │  │   FastAPI Gateway     │   │                     │
                    │  │   (Python)            │   │                     │
                    │  │   - Auth verification │   │                     │
                    │  │   - Model routing     │   │                     │
                    │  │   - Rate limiting     │   │                     │
                    │  └──────────┬───────────┘   │                     │
                    │             │                │                     │
                    │  ┌──────────┴───────────┐   │                     │
                    │  │   Model APIs          │   │                     │
                    │  │   - ElevenLabs SFX    │   │                     │
                    │  │   - Stable Audio Open │   │                     │
                    │  │   - Replicate (backup)│   │                     │
                    │  └──────────────────────┘   │                     │
                    │                             │                     │
                    │  ┌──────────────────────┐   │                     │
                    │  │   Supabase            │   │                     │
                    │  │   - Auth              │   │                     │
                    │  │   - User management   │   │                     │
                    │  └──────────────────────┘   │                     │
                    └─────────────────────────────┘─────────────────────┘
```

---

## Frontend Modules

### Directory Structure

```
src/
├── main.tsx                          # Entry point
├── App.tsx                           # Root component, routing
├── components/
│   ├── layout/
│   │   ├── TopBar.tsx                # Logo, title, settings, account
│   │   ├── StatusBar.tsx             # Status, generation progress
│   │   └── AppShell.tsx              # Main layout wrapper
│   ├── prompt/
│   │   ├── PromptBar.tsx             # Text input + generate button
│   │   ├── PromptHistory.tsx         # Recent prompts dropdown
│   │   ├── ReferenceDropZone.tsx     # Drag-and-drop reference upload
│   │   └── SemanticSuggestions.tsx   # Autocomplete overlay
│   ├── grid/
│   │   ├── SoundGrid.tsx             # 2×3 grid container
│   │   ├── SoundSlot.tsx             # Individual sound card
│   │   └── WaveformThumbnail.tsx     # SVG waveform renderer
│   ├── detail/
│   │   ├── DetailPanel.tsx           # Full detail overlay
│   │   ├── WaveformViewer.tsx        # Zoomable waveform + spectrogram
│   │   ├── SoundScoreDisplay.tsx     # Quality metric visualization
│   │   ├── MetadataCard.tsx          # Generation metadata
│   │   └── ProvenanceCard.tsx        # Model, seed, prompt info
│   ├── library/
│   │   ├── LibraryView.tsx           # Library grid
│   │   ├── LibrarySearch.tsx         # Search + filter bar
│   │   ├── PackList.tsx              # Pack sidebar
│   │   └── PackDetail.tsx            # Individual pack view
│   ├── export/
│   │   ├── ExportDialog.tsx          # Format selection modal
│   │   ├── ExportProgress.tsx        # Export progress indicator
│   │   └── ExportHistory.tsx         # Recent exports list
│   └── shared/
│       ├── Button.tsx
│       ├── Slider.tsx
│       ├── Modal.tsx
│       ├── Toast.tsx
│       ├── Spinner.tsx
│       └── Badge.tsx
├── stores/
│   ├── useGenerationStore.ts         # Generation state, results, prompt history
│   ├── useLibraryStore.ts            # Library state, search, packs
│   ├── useSettingsStore.ts           # User preferences, model selection
│   ├── useAudioStore.ts              # Playback state, current playing sound
│   └── useExportStore.ts             # Export queue, history
├── hooks/
│   ├── useAudioPlayback.ts           # Web Audio API wrapper
│   ├── useGeneration.ts              # Generation IPC calls
│   ├── useExport.ts                  # Export IPC calls
│   ├── useKeyboard.ts                # Keyboard shortcuts
│   ├── useLibrary.ts                 # Library IPC calls
│   └── useWaveform.ts               # Waveform data processing
├── lib/
│   ├── api.ts                        # Tauri IPC wrapper functions
│   ├── audio.ts                      # Audio buffer utilities
│   ├── format.ts                     # File size, duration formatters
│   └── constants.ts                  # App-wide constants
├── types/
│   ├── generation.ts                 # Generation request/response types
│   ├── library.ts                    # Sound, Pack types
│   ├── audio.ts                      # Audio analysis types
│   └── api.ts                        # IPC command types
└── styles/
    ├── globals.css                   # Tailwind base + custom CSS
    └── tokens.ts                     # Design tokens (colors, spacing)
```

### Key State Shapes

```typescript
// useGenerationStore.ts
interface GenerationState {
  prompt: string;
  isGenerating: boolean;
  results: SoundSlot[];
  history: GenerationEntry[];
  selectedSlotIndex: number | null;
  referenceAudio: AudioBuffer | null;
  
  setPrompt: (prompt: string) => void;
  generate: () => Promise<void>;
  regenerate: (index: number) => Promise<void>;
  selectSlot: (index: number) => void;
  clearResults: () => void;
}

interface SoundSlot {
  id: string;
  hash: string;
  audioData: Float32Array;          // raw float32 samples
  waveformPath: string;             // SVG path data for thumbnail
  metadata: SoundMetadata;
  isPlaying: boolean;
}

// useLibraryStore.ts
interface LibraryState {
  sounds: SoundEntry[];
  packs: Pack[];
  searchQuery: string;
  filters: LibraryFilters;
  viewMode: 'grid' | 'list';
  
  loadLibrary: () => Promise<void>;
  search: (query: string) => void;
  addToPack: (soundId: string, packId: string) => void;
  deleteSound: (soundId: string) => void;
  createPack: (name: string) => void;
}

interface SoundEntry {
  id: string;
  hash: string;
  prompt: string;
  model: string;
  seed: number;
  createdAt: string;
  durationMs: number;
  sampleRate: number;
  bitDepth: number;
  rms: number;
  peak: number;
  crestFactor: number;
  soundscoreOverall: number;
  rating: number | null;
  tags: string[];
  packId: string | null;
  isExported: boolean;
}
```

### IPC Commands (Tauri invoke)

```typescript
// Frontend calls Rust backend via invoke()

// Generation
invoke('generate', { prompt: string, referencePath?: string, model?: string })
// → { sounds: SoundSlot[], latencyMs: number }

invoke('regenerate', { soundId: string, prompt?: string })
// → { sound: SoundSlot }

// Library
invoke('get_library', { filters?: LibraryFilters })
// → { sounds: SoundEntry[], total: number }

invoke('search_library', { query: string, filters?: LibraryFilters })
// → { sounds: SoundEntry[] }

invoke('delete_sound', { soundId: string })
// → void

invoke('update_sound_metadata', { soundId: string, tags?: string[], rating?: number })
// → void

// Packs
invoke('create_pack', { name: string })
// → { pack: Pack }

invoke('add_to_pack', { soundId: string, packId: string })
// → void

invoke('get_packs')
// → { packs: Pack[] }

// Export
invoke('export_sound', { 
  soundId: string, 
  format: 'wav' | 'aiff' | 'flac' | 'mp3',
  bitDepth: 16 | 24 | 32,
  sampleRate: 44100 | 48000 | 96000,
  normalize: boolean,
  fadeInMs: number,
  fadeOutMs: number,
  outputPath: string,
  filename: string
})
// → { path: string, sizeBytes: number, durationMs: number }

// Reference
invoke('analyze_reference', { path: string })
// → { sampleRate: number, durationMs: number, spectralProfile: number[], keyEstimate?: string }

// Settings
invoke('get_settings')
// → { model: string, outputPath: string, ... }

invoke('update_settings', { settings: Partial<Settings> })
// → void
```

---

## Backend Services (Rust)

### Module Structure

```
src-tauri/
├── src/
│   ├── main.rs                       # Tauri entry point, command registration
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── generation.rs             # generate, regenerate commands
│   │   ├── library.rs                # Library CRUD commands
│   │   ├── export.rs                 # Export command
│   │   ├── reference.rs              # Reference analysis command
│   │   └── settings.rs               # Settings commands
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── pipeline.rs               # Orchesrates DSP chain
│   │   ├── dsp/
│   │   │   ├── mod.rs
│   │   │   ├── trim.rs               # Silence trimming
│   │   │   ├── normalize.rs          # Peak normalization
│   │   │   ├── fade.rs               # Fade in/out
│   │   │   └── analyze.rs            # RMS, crest factor, centroid, onset
│   │   ├── codec/
│   │   │   ├── mod.rs
│   │   │   ├── wav.rs                # WAV read/write (hound)
│   │   │   ├── aiff.rs               # AIFF read/write
│   │   │   ├── flac.rs               # FLAC encode
│   │   │   └── mp3.rs                # MP3 encode (lame)
│   │   └── waveform.rs               # SVG path generation from samples
│   ├── generation/
│   │   ├── mod.rs
│   │   ├── gateway.rs                # Model gateway router
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── elevenlabs.rs         # ElevenLabs SFX API client
│   │   │   ├── stableaudio.rs        # Stable Audio API client
│   │   │   └── local.rs              # Local ONNX inference
│   │   └── prompt_encoder.rs         # Local CLAP-style text encoder (ONNX)
│   ├── library/
│   │   ├── mod.rs
│   │   ├── database.rs               # SQLite via rusqlite
│   │   ├── storage.rs                # Content-addressed file store
│   │   ├── search.rs                 # Full-text search (prompts, tags)
│   │   └── vector_search.rs          # FAISS similarity search
│   ├── export/
│   │   ├── mod.rs
│   │   ├── export_service.rs         # Format selection, file naming
│   │   └── naming.rs                 # Semantic filename generation
│   ├── analysis/
│   │   ├── mod.rs
│   │   ├── soundscore.rs             # ONNX quality model inference
│   │   └── classifier.rs             # Sound type classification (CNN)
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs               # User config management
│   └── error.rs                      # Unified error types
```

### Core Traits

```rust
// Audio Generator trait — all models implement this
#[async_trait]
pub trait AudioGenerator {
    /// Generate audio from a text prompt
    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse>;
    
    /// Name of the model (for display and logging)
    fn model_name(&self) -> &str;
    
    /// Whether this model supports reference audio
    fn supports_reference(&self) -> bool;
    
    /// Maximum generation duration in seconds
    fn max_duration_seconds(&self) -> f32;
}

pub struct GenerationRequest {
    pub prompt: String,
    pub reference_audio: Option<Vec<f32>>,
    pub reference_sample_rate: Option<u32>,
    pub seed: Option<u64>,
    pub duration_seconds: Option<f32>,
}

pub struct GenerationResponse {
    pub audio: Vec<f32>,          // mono float32, 44.1kHz
    pub sample_rate: u32,         // usually 44100
    pub latency_ms: u64,
    pub model_name: String,
    pub model_version: String,
    pub seed: u64,
}
```

### Key Backend Functions

```rust
// Generation pipeline orchestrator
impl GenerationPipeline {
    pub async fn generate_sounds(
        &self,
        prompt: String,
        reference_path: Option<PathBuf>,
        count: usize,        // number of variations (default 6)
    ) -> Result<Vec<SoundSlot>> {
        // 1. Encode prompt to text embedding (local ONNX, <100ms)
        let embedding = self.prompt_encoder.encode(&prompt)?;
        
        // 2. Load reference audio if provided
        let reference = if let Some(path) = reference_path {
            Some(self.load_reference(path).await?)
        } else {
            None
        };
        
        // 3. Generate each variation
        let mut slots = Vec::with_capacity(count);
        for i in 0..count {
            let seed = if reference.is_some() { i as u64 } else { rand::random() };
            
            // 4. Route to model gateway
            let response = self.gateway.generate(GenerationRequest {
                prompt: prompt.clone(),
                reference_audio: reference.as_ref().map(|r| r.samples.clone()),
                reference_sample_rate: reference.as_ref().map(|r| r.sample_rate),
                seed: Some(seed),
                duration_seconds: Some(1.0), // one-shot duration
            }).await?;
            
            // 5. Apply DSP post-processing
            let processed = self.dsp_pipeline.process(response.audio, response.sample_rate)?;
            
            // 6. Compute SoundScore
            let soundscore = self.soundscore_model.score(&processed.samples, processed.sample_rate)?;
            
            // 7. Content-addressed storage
            let hash = self.storage.store(&processed.samples, processed.sample_rate, processed.bit_depth)?;
            
            // 8. Write metadata to database
            let sound_id = self.database.insert_sound(SoundEntry {
                hash: hash.clone(),
                prompt: prompt.clone(),
                model: response.model_name,
                seed: response.seed,
                duration_ms: processed.duration_ms,
                sample_rate: processed.sample_rate,
                bit_depth: processed.bit_depth,
                rms: processed.rms,
                peak: processed.peak,
                crest_factor: processed.crest_factor,
                spectral_centroid: processed.spectral_centroid,
                transient_time_ms: processed.transient_time_ms,
                soundscore_overall: soundscore.overall,
                soundscore_punch: soundscore.punch,
                soundscore_body: soundscore.body,
                soundscore_clarity: soundscore.clarity,
                soundscore_uniqueness: soundscore.uniqueness,
            })?;
            
            slots.push(SoundSlot {
                id: sound_id,
                hash,
                audio_data: processed.samples,
                metadata: processed.metadata,
            });
        }
        
        Ok(slots)
    }
}
```

---

## Database Schema (SQLite)

```sql
-- Core tables
CREATE TABLE sounds (
    id TEXT PRIMARY KEY,                -- UUID v4
    hash TEXT NOT NULL UNIQUE,           -- SHA-256 of audio data
    prompt TEXT NOT NULL,                -- original generation prompt
    text_embedding BLOB,                 -- CLAP embedding (768 float32s = 3072 bytes)
    model TEXT NOT NULL,                 -- model identifier
    model_version TEXT NOT NULL,         -- model version string
    seed INTEGER NOT NULL,               -- random seed
    duration_ms INTEGER NOT NULL,
    sample_rate INTEGER NOT NULL,
    bit_depth INTEGER NOT NULL,
    channels INTEGER NOT NULL DEFAULT 1,
    file_size_bytes INTEGER NOT NULL,
    
    -- Acoustic analysis
    rms REAL NOT NULL,
    peak REAL NOT NULL,
    crest_factor REAL NOT NULL,
    spectral_centroid REAL,
    spectral_rolloff REAL,
    transient_time_ms INTEGER,
    zero_crossing_rate REAL,
    
    -- SoundScore
    soundscore_punch REAL,
    soundscore_body REAL,
    soundscore_clarity REAL,
    soundscore_uniqueness REAL,
    soundscore_overall REAL,
    
    -- User metadata
    user_rating INTEGER CHECK(user_rating >= 1 AND user_rating <= 5),
    tags TEXT,                           -- JSON array of strings
    is_exported INTEGER DEFAULT 0,
    export_count INTEGER DEFAULT 0,
    last_exported_at TEXT,
    
    -- Provenance
    reference_hash TEXT REFERENCES sounds(hash),
    parent_id TEXT REFERENCES sounds(id),
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_sounds_hash ON sounds(hash);
CREATE INDEX idx_sounds_created ON sounds(created_at DESC);
CREATE INDEX idx_sounds_score ON sounds(soundscore_overall DESC);
CREATE INDEX idx_sounds_prompt ON sounds(prompt);
CREATE INDEX idx_sounds_model ON sounds(model);

CREATE VIRTUAL TABLE sounds_fts USING fts5(
    prompt, tags,
    content='sounds',
    content_rowid='rowid'
);

CREATE TABLE packs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_public INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pack_sounds (
    pack_id TEXT NOT NULL REFERENCES packs(id) ON DELETE CASCADE,
    sound_id TEXT NOT NULL REFERENCES sounds(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (pack_id, sound_id)
);

CREATE TABLE generation_log (
    id TEXT PRIMARY KEY,
    prompt TEXT NOT NULL,
    prompt_length INTEGER NOT NULL,
    reference_used INTEGER DEFAULT 0,
    model TEXT NOT NULL,
    model_version TEXT,
    seed INTEGER,
    count_requested INTEGER NOT NULL,
    count_generated INTEGER NOT NULL,
    latency_ms INTEGER NOT NULL,
    success INTEGER NOT NULL,
    error TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_genlog_created ON generation_log(created_at DESC);
CREATE INDEX idx_genlog_success ON generation_log(success);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE user_profile (
    id INTEGER PRIMARY KEY DEFAULT 1,
    total_generations INTEGER DEFAULT 0,
    total_exports INTEGER DEFAULT 0,
    favorite_model TEXT,
    sound_preferences TEXT,   -- JSON: preferred soundscore dimensions
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Triggers for denormalized counts
CREATE TRIGGER after_sound_insert AFTER INSERT ON sounds
BEGIN
    UPDATE user_profile SET total_generations = total_generations + 1 WHERE id = 1;
END;
```

---

## Storage Layout

```
~/.cshot/
├── audio/
│   ├── ab/
│   │   ├── c3d4e5f678901234567890abcdef0123456789abcdef0123456789abcdef0123.wav
│   │   └── ...
│   ├── cd/
│   └── ...                    # 2-level SHA-256 prefix directories
├── metadata.db                # SQLite database
├── faiss_index.bin            # FAISS vector index
├── models/
│   ├── text_encoder.onnx      # CLAP-style text encoder
│   ├── soundscore.onnx        # SoundScore quality model
│   └── classifier.onnx        # Sound type classifier (optional)
├── config.json                # User settings
├── cache/
│   ├── waveforms/             # Cached SVG waveform paths
│   └── thumbnails/            # Cached waveform thumbnail images
└── export/                    # Default export directory
    └── (user-named files)
```

---

## Audio Processing Modules

### DSP Pipeline

```rust
pub struct DspPipeline;

impl DspPipeline {
    pub fn process(&self, input: Vec<f32>, sample_rate: u32) -> Result<ProcessedAudio> {
        let mut samples = input;
        
        // 1. Trim silence
        samples = trim_silence(&samples, sample_rate, -60.0, 100);
        
        // 2. Normalize peak to -1.0dB
        samples = normalize_peak(&samples, -1.0);
        
        // 3. Apply fades
        samples = fade_in(&samples, sample_rate, 5);
        samples = fade_out(&samples, sample_rate, 10);
        
        // 4. Analyze
        let analysis = analyze(&samples, sample_rate);
        
        Ok(ProcessedAudio {
            samples,
            sample_rate,
            bit_depth: 24,           // default for storage
            duration_ms: (samples.len() as f64 / sample_rate as f64 * 1000.0) as u64,
            rms: analysis.rms,
            peak: analysis.peak,
            crest_factor: analysis.crest_factor,
            spectral_centroid: analysis.spectral_centroid,
            transient_time_ms: analysis.transient_time_ms,
            metadata: analysis,
        })
    }
}

fn trim_silence(samples: &[f32], sample_rate: u32, threshold_db: f32, hold_ms: u32) -> Vec<f32> {
    let threshold = 10.0_f32.powf(threshold_db / 20.0);
    let hold_samples = (hold_ms as f64 * sample_rate as f64 / 1000.0) as usize;
    
    // Find start: first sample above threshold with hold duration after
    let start = samples.windows(hold_samples)
        .position(|window| window.iter().any(|&s| s.abs() > threshold))
        .unwrap_or(0);
    
    // Find end: last sample above threshold with hold duration before
    let end = samples.windows(hold_samples)
        .rposition(|window| window.iter().any(|&s| s.abs() > threshold))
        .map(|pos| pos + hold_samples)
        .unwrap_or(samples.len());
    
    samples[start..end].to_vec()
}

fn normalize_peak(samples: &[f32], target_db: f32) -> Vec<f32> {
    let target = 10.0_f32.powf(target_db / 20.0);
    let current_peak = samples.iter().fold(0.0f32, |max, &s| max.max(s.abs()));
    
    if current_peak < 1e-10 { return samples.to_vec(); } // avoid divide-by-zero
    
    let gain = target / current_peak;
    samples.iter().map(|&s| (s * gain).clamp(-1.0, 1.0)).collect()
}

fn fade_in(samples: &[f32], sample_rate: u32, ms: u32) -> Vec<f32> {
    let fade_len = (ms as f64 * sample_rate as f64 / 1000.0) as usize;
    let fade_len = fade_len.min(samples.len());
    
    let mut result = samples.to_vec();
    for i in 0..fade_len {
        let gain = i as f32 / fade_len as f32;
        result[i] *= gain;
    }
    result
}

fn fade_out(samples: &[f32], sample_rate: u32, ms: u32) -> Vec<f32> {
    let fade_len = (ms as f64 * sample_rate as f64 / 1000.0) as usize;
    let fade_len = fade_len.min(samples.len());
    let start = samples.len() - fade_len;
    
    let mut result = samples.to_vec();
    for i in 0..fade_len {
        let gain = (fade_len - i) as f32 / fade_len as f32;
        result[start + i] *= gain;
    }
    result
}

fn analyze(samples: &[f32], sample_rate: u32) -> AudioAnalysis {
    let rms = (samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    let peak = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    let crest_factor = if rms > 1e-10 { 20.0 * (peak / rms).log10() } else { 0.0 };
    
    // Spectral centroid via simplified FFT
    let spectral_centroid = compute_spectral_centroid(samples, sample_rate);
    
    // Transient onset detection via energy envelope
    let transient_time_ms = detect_transient(samples, sample_rate);
    
    AudioAnalysis {
        rms: 20.0 * rms.log10(),           // convert to dB
        peak: 20.0 * peak.log10(),
        crest_factor,
        spectral_centroid,
        transient_time_ms,
    }
}
```

---

## Model Integration Layer

### Model Gateway

```rust
pub struct ModelGateway {
    elevenlabs: ElevenLabsClient,
    stable_audio: StableAudioClient,
    local: Option<LocalOnnxModel>,
    config: GatewayConfig,
}

impl ModelGateway {
    pub async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        // Try models in priority order
        let errors = Vec::new();
        
        // 1. Try local model first (zero latency, always available)
        if let Some(ref local) = self.local {
            if local.is_available() && self.config.prefer_local {
                match local.generate(&request).await {
                    Ok(response) => return Ok(response),
                    Err(e) => errors.push(("local", e)),
                }
            }
        }
        
        // 2. Try ElevenLabs (best quality)
        if self.elevenlabs.is_available() {
            match self.elevenlabs.generate(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => errors.push(("elevenlabs", e)),
            }
        }
        
        // 3. Fall back to Stable Audio
        if self.stable_audio.is_available() {
            match self.stable_audio.generate(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => errors.push(("stableaudio", e)),
            }
        }
        
        // 4. All models failed
        Err(GenerationError::AllModelsFailed(errors))
    }
}
```

### ElevenLabs SFX Client

```rust
pub struct ElevenLabsClient {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl ElevenLabsClient {
    pub async fn generate(&self, request: &GenerationRequest) -> Result<GenerationResponse> {
        // Build ElevenLabs API request
        let payload = serde_json::json!({
            "text": request.prompt,
            "duration_seconds": request.duration_seconds.unwrap_or(1.0),
            "seed": request.seed.unwrap_or_else(rand::random),
        });
        
        let start = Instant::now();
        let response = self.client
            .post(format!("{}/v1/sound-generation", self.base_url))
            .header("xi-api-key", &self.api_key)
            .json(&payload)
            .send()
            .await?;
            
        let latency = start.elapsed().as_millis() as u64;
        let bytes = response.bytes().await?;
        
        // Decode MP3/WAV response to float32 samples
        let audio = decode_audio_bytes(&bytes)?;
        
        Ok(GenerationResponse {
            audio,
            sample_rate: 44100,
            latency_ms: latency,
            model_name: "elevenlabs_sfx".into(),
            model_version: "v2".into(),
            seed: request.seed.unwrap_or(0),
        })
    }
}
```

---

## Job Queue

### Phase 1: Direct Async (No Queue)

In v1, generation is synchronous from the user's perspective: press generate, wait for results. No background job queue is needed.

```rust
// Tauri command handler
#[tauri::command]
async fn generate(
    state: State<'_, AppState>,
    prompt: String,
    reference_path: Option<String>,
    count: Option<usize>,
) -> Result<GenerationResult, String> {
    let pipeline = state.pipeline.lock().await;
    let reference = reference_path.map(PathBuf::from);
    
    let start = Instant::now();
    let sounds = pipeline.generate_sounds(prompt.clone(), reference, count.unwrap_or(6))
        .await
        .map_err(|e| e.to_string())?;
    let latency = start.elapsed().as_millis() as u64;
    
    Ok(GenerationResult { sounds, latency_ms: latency })
}
```

### Phase 2 Upgrade Path

Add in-process job queue when batch generation arrives:

```rust
// Background job queue (Rust tokio::sync::mpsc)
pub struct JobQueue {
    sender: mpsc::Sender<GenerationJob>,
    receiver: mpsc::Receiver<GenerationJob>,
}

pub struct GenerationJob {
    pub id: String,
    pub request: GenerationRequest,
    pub response_tx: oneshot::Sender<Result<GenerationResponse>>,
}

// Worker processes jobs sequentially or in parallel
impl JobQueue {
    pub async fn enqueue(&self, job: GenerationJob) {
        self.sender.send(job).await.unwrap();
    }
    
    pub async fn worker(mut self) {
        while let Some(job) = self.receiver.recv().await {
            let result = self.pipeline.generate(job.request).await;
            let _ = job.response_tx.send(result);
        }
    }
}
```

---

## API Routes (Cloud Gateway)

### FastAPI Gateway

```python
# FastAPI cloud gateway service
from fastapi import FastAPI, HTTPException, Depends
from pydantic import BaseModel

app = FastAPI(title="cShot Gateway")

class GenerationRequest(BaseModel):
    prompt: str
    model: str = "elevenlabs"      # elevenlabs, stableaudio, replicate
    seed: int | None = None
    duration_seconds: float = 1.0
    reference_audio: bytes | None = None

class GenerationResponse(BaseModel):
    audio_base64: str
    sample_rate: int
    model_used: str
    latency_ms: int
    seed: int

@app.post("/v1/generate")
async def generate(request: GenerationRequest, user=Depends(verify_auth)):
    # Check rate limit
    await check_rate_limit(user.id, request.model)
    
    # Route to model
    model = get_model(request.model)
    result = await model.generate(
        text=request.prompt,
        seed=request.seed,
        duration=request.duration_seconds,
        reference=request.reference_audio,
    )
    
    # Log generation
    await log_generation(user.id, request, result)
    
    return GenerationResponse(
        audio_base64=base64_encode(result.audio),
        sample_rate=result.sample_rate,
        model_used=request.model,
        latency_ms=result.latency_ms,
        seed=result.seed,
    )
```

---

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum CShotError {
    #[error("Generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("All models failed: {errors:?}")]
    AllModelsFailed { errors: Vec<(&'static str, Box<dyn std::error::Error>)> },
    
    #[error("Model {model} is not available")]
    ModelNotAvailable { model: String },
    
    #[error("Audio processing error: {0}")]
    AudioProcessing(String),
    
    #[error("Invalid audio format: {0}")]
    InvalidAudioFormat(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

// Tauri error serialization
impl serde::Serialize for CShotError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}
```

**Error handling strategy:**
- Recoverable errors (model timeout, network failure): log + retry with fallback model
- Unrecoverable errors (corrupt audio, database corruption): show user-friendly message + log details
- Silent errors (one bad generation in 6): drop bad result, show 5 instead of 6
- Edge cases: empty prompt (refuse with "Please describe the sound"), very long prompt (truncate), special characters (sanitize)

---

## Testing Strategy

| Layer | Tool | What to Test |
|---|---|---|
| DSP (Rust) | `cargo test` + golden files | Every DSP function against known reference outputs |
| Database (Rust) | `cargo test` with in-memory SQLite | Schema creation, CRUD, queries, FTS, migration |
| API clients (Rust) | Mock HTTP server (`wiremock`) | Request formation, response parsing, error handling, timeouts |
| Generation pipeline | Integration test with mock gateway | End-to-end flow: prompt → encoding → generation → DSP → storage |
| Frontend components | Vitest + React Testing Library | Component rendering, user interactions, state changes |
| Frontend integration | Playwright | Full IPC command flow, UI state machine, error states |
| Export | `cargo test` with temp dirs | Format correctness (WAV header, bit depth, sample rate) |
| Storage | `cargo test` with temp dirs | Content-addressed read/write, dedup, hash verification |
| FAISS search | `cargo test` | Index building, query, accuracy |
| SoundScore | Python test suite (research) | Model accuracy against human ratings, regression checks |

**CI pipeline:**
```
cargo check → cargo test (Rust) → vitest (frontend) → 
playwright (integration) → cargo build (release) → 
smoke test (generate + export)
```

---

## Deployment Plan

### Desktop App Distribution

| Platform | Distribution Method | Update Mechanism |
|---|---|---|
| macOS | DMG download + Notarized | Tauri updater (sparkle-like) |
| Windows | MSI installer + Code Signed | Tauri updater |
| Linux | AppImage | Tauri updater |

### Cloud Services Deployment

| Service | Hosting | Notes |
|---|---|---|
| FastAPI gateway | Railway / Fly.io / Render | Auto-scaling, global regions |
| Supabase | Supabase managed | Auth + user management |
| Model APIs | ElevenLabs / Stability AI managed | No self-hosting for v1 |
| Monitoring | Sentry (error tracking) + Grafana (metrics) | |

### Release Cadence

- Beta: weekly releases
- v1 stable: bi-weekly releases
- Hotfix: as needed (CI → build → notarize → publish in <2 hours)

---

## Observability

```rust
// Structured logging
use tracing::{info, warn, error, instrument};

#[instrument(skip(pipeline))]
pub async fn generate_command(
    prompt: String,
    reference_path: Option<String>,
) -> Result<GenerationResult> {
    info!(prompt_length = prompt.len(), "Generation started");
    
    let start = Instant::now();
    let result = pipeline.generate_sounds(&prompt, reference_path.map(PathBuf::from), 6).await;
    let latency = start.elapsed();
    
    match &result {
        Ok(sounds) => {
            info!(
                count = sounds.len(),
                latency_ms = latency.as_millis(),
                "Generation completed"
            );
        }
        Err(e) => {
            warn!(
                error = %e,
                latency_ms = latency.as_millis(),
                "Generation failed"
            );
        }
    }
    
    result.map_err(|e| e.to_string())
}
```

### Key Metrics (collected anonymously, opt-in)

| Metric | Source | Why |
|---|---|---|
| Generation latency | Rust tracing spans | Track speed promise |
| Model success rate | Gateway router | Model reliability |
| Export rate | Database queries | Core business metric |
| Regeneration rate | Database queries | Quality signal (inverse proxy) |
| Error frequency | Sentry | Stability monitoring |
| Active users | Supabase analytics | Growth tracking |
| SoundScore distribution | Database queries | Generation quality (model health) |
| Storage usage | Filesystem | Capacity planning |
