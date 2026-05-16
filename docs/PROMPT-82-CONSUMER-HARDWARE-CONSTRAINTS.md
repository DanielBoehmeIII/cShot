# Prompt 82 — Consumer Hardware Constraints

Design cShot for normal consumer laptops. Not a server rack. Not a gaming PC. A 2022 MacBook Air or a Dell XPS 13.

---

## 1. Hardware Baseline

### Target Spec

```
Tier 1 (Minimum — must work, may be slow)
  CPU: Intel i5-1135G7 / AMD Ryzen 5 5500U (4 cores, 8 threads)
  RAM: 8 GB (6 GB available after OS)
  GPU: Integrated (Intel Iris Xe / AMD Vega 8)
  VRAM: 0 dedicated (shared with system)
  Storage: 256 GB SSD (150 GB free)
  Battery: 45 Wh
  Screen: 1920×1080, 60 Hz
  Audio: Onboard Realtek + headphones

Tier 2 (Target — feels fast)
  CPU: Apple M2 / Intel i7-12700H / AMD Ryzen 7 6800H
  RAM: 16 GB (12 GB available)
  GPU: Apple M2 GPU / NVIDIA RTX 3050 / AMD Radeon 6600M
  VRAM: 0 (M2 unified) / 4 GB (RTX 3050)
  Storage: 512 GB SSD (300 GB free)
  Battery: 70 Wh
  Screen: 2560×1600, 120 Hz

Tier 3 (Premium — best experience)
  CPU: Apple M3 Max / Intel i9-13900H / AMD Ryzen 9 7945HX
  RAM: 32+ GB
  GPU: M3 Max 40-core / NVIDIA RTX 4070+
  VRAM: Unified / 8+ GB
  Storage: 1 TB SSD
```

### Key Constraint: Tier 1 is the floor

cShot must launch, load a sound, play it, search the library, and generate a basic sound on Tier 1 hardware within acceptable time:
- **Launch to ready**: < 2 seconds
- **Sound playback**: < 100ms latency
- **Library search**: < 500ms for 10,000 sounds
- **Local generation**: < 10 seconds (basic), < 30 seconds (quality)

---

## 2. CPU Limits

### Reality Check

| Operation | CPU Time (Tier 1) | Acceptable |
|-----------|-------------------|------------|
| Decode 10s WAV | 2ms | Instant |
| Waveform render (10s) | 5ms | Instant |
| Spectrogram render (10s) | 50ms | Instant |
| LUFS analysis (10s) | 30ms | Instant |
| ONNX inference (1.5s audio) | 2-8s | Slow |
| FFT convolution (1s IR) | 15ms | Instant |
| Full export (10s WAV → 44.1k 24-bit) | 50ms | Instant |

**Critical insight**: Traditional audio DSP is trivially fast on modern CPUs. Model inference is the bottleneck — by 2-3 orders of magnitude.

### CPU Strategy

```
1. Keep model inference OFF the main thread
   - Rust async task pool for generation
   - UI stays responsive even during 15s generation
   - Show progress: waveform fills in as generation proceeds

2. Pin inference threads to performance cores
   - Use thread affinity where available
   - Don't compete with UI thread
   - Reserve 1 core for UI + audio playback

3. Batch audio analysis at idle
   - New imports → hash → write file → return immediately
   - Analysis (waveform, loudness, spectral) runs as background job
   - Results stream to UI as they complete
   - User can play/export before analysis finishes

4. Use SIMD for DSP hot paths
   - x86: SSE4.2/AVX2
   - ARM: NEON
   - Rust: `wide` crate, `simdeez`, or manual intrinsics
   - DSP operations: 4-8x speedup over scalar
```

### CPU Budget Allocation

```
┌─────────────────────────────────────────┐
│  Total CPU Budget (8 threads)            │
├────────────┬────────────┬───────────────┤
│  UI Thread  │  Audio     │  Worker Pool  │
│  (1 core)  │  Playback  │  (6 cores)    │
│            │  (1 core)  │               │
├────────────┼────────────┼───────────────┤
│ ~5%        │ ~2%        │ ~93%          │
│            │            │               │
│ Rendering  │ Buffer     │ Generation    │
│ Events     │ fill       │ Analysis      │
│ IPC        │ Dispatching│ Export        │
│            │ Low-latency│ Import        │
│            │            │ Cache build   │
└────────────┴────────────┴───────────────┘
```

---

## 3. GPU / VRAM Limits

### The Hard Truth

```
Tier 1 (integrated GPU):
  - Shared memory: 0.5-2 GB (stolen from system RAM)
  - No dedicated VRAM
  - Already rendering: OS compositor, browser tabs, DAW
  - Available for inference: ~200-500 MB
  - Cannot load models > ~300 MB comfortably

Tier 2 (discrete GPU, 4 GB):
  - 4 GB VRAM, but OS + game/daw may use 1-2 GB
  - Available for inference: ~2 GB
  - Can load models up to ~1.5 GB (with activation memory)

Tier 3 (8+ GB):
  - Available for inference: 4-6 GB
  - Can load most open-source audio models
```

### GPU Strategy

```
1. Default to CPU inference
   - ONNX Runtime CPU provider is the baseline
   - Works everywhere, no GPU required
   - Acceptable for generation (5-15s on modern CPU)

2. GPU as optional accelerator
   - Check available VRAM at startup
   - Only use GPU if model fits comfortably (<70% VRAM)
   - Never exceed VRAM — OOM kills the app
   - Graceful fallback to CPU if GPU fails

3. Aggressive quantization for GPU
   - INT8 model fits in 500 MB → usable on integrated GPU
   - FP16 model fits in 1 GB → usable on 4 GB GPU
   - INT4 model fits in 200 MB → usable everywhere

4. Apple Silicon advantage
   - M1/M2/M3 unified memory: model can use 2-4 GB without copying
   - ANE (Neural Engine) is very efficient: ~15W for sustained inference
   - CoreML path for Apple hardware is a priority
   - Metal Performance Shaders for GPU compute

5. GPU memory management
   - Load model at startup (if fits) or on-demand
   - Unload model after generation if memory pressure
   - Monitor memory usage, warn user if critical
   - Prioritize playback/preview stability over inference speed
```

### GPU Model Fit Matrix

```
Model Size     | Integrated | 4 GB GPU | 8 GB GPU | M-series
               | (Tier 1)   | (Tier 2) | (Tier 3) | (unified)
───────────────┼────────────┼──────────┼──────────┼──────────
50 MB (INT4)   │ ✓ Perfect  │ ✓ Perfect│ ✓ Perfect│ ✓ Perfect
200 MB (INT8)  │ ✓ Fits     │ ✓ Perfect│ ✓ Perfect│ ✓ Perfect
500 MB (FP16)  │ ✗ Too big  │ ✓ Fits   │ ✓ Perfect│ ✓ Fits
1 GB (FP16)    │ ✗          │ ✓ Tight  │ ✓ Fits   │ ✓ Fits
2 GB (FP32)    │ ✗          │ ✗ OOM    │ ✓ Tight  │ ✓ Tight
4 GB (FP32)    │ ✗          │ ✗        │ ✗ OOM    │ ✓ Tight
```

---

## 4. RAM Limits

### Memory Budget (Tier 1: 8 GB, ~6 GB available)

```
┌──────────────────────────────────────────┐
│  Memory Allocation (8 GB total)           │
├──────────────────────┬───────────────────┤
│ OS + Background      │ 2 GB              │
│ App: UI + Framework  │ 200 MB            │
│ App: Sound Data      │ 200 MB (20 sounds)│
│ App: Waveform Cache  │ 100 MB            │
│ App: Model (INT8)    │ 200 MB            │
│ App: DSP Buffers     │ 50 MB             │
│ Reserved for DAW     │ 500 MB            │
│ Free (allocatable)   │ ~2.75 GB          │
└──────────────────────┴───────────────────┘
```

### RAM Strategy

```
1. Memory-mapped audio files
   - Never load entire WAV files into RAM
   - Use mmap for instant access to any position
   - OS handles page-in/page-out based on access patterns
   - Result: 100 MB RAM can handle 10,000 sounds

2. Streaming playback from disk
   - Playback reads from mmap in small buffers (4096 samples)
   - Never keep entire sound in memory for playback
   - Only warm up the first 100ms for instant start

3. Model loading discipline
   - Load model lazily (on first generation request)
   - Unload model after N minutes of inactivity
   - Never keep multiple models in memory
   - Quantized model is always the default

4. Waveform cache limits
   - Keep last 50 waveforms in RAM (~100 MB for 10s sounds)
   - LRU eviction for older waveforms
   - Regenerate from audio file on cache miss (<5ms)

5. Garbage collection awareness
   - Rust has no GC — explicit memory management
   - No GC pauses (critical for audio playback)
   - Arena allocator for DSP scratch buffers
   - Pool allocator for frequently-allocated types (AudioBuffer, FFT plans)
```

### Memory Budget by Tier

```
                    Tier 1 (8GB)  Tier 2 (16GB)  Tier 3 (32GB)
                    ────────────  ─────────────  ─────────────
UI + Framework       200 MB        300 MB          500 MB
Sound Data (mmap)    50 MB         100 MB          200 MB
Waveform Cache       100 MB        200 MB          500 MB
Spectrogram Cache    100 MB        200 MB          500 MB
Model                 200 MB        1 GB           2 GB
DSP Buffers           50 MB         100 MB          200 MB
Embedding Cache       50 MB         100 MB          500 MB
OS Reserve           2 GB          3 GB           4 GB
DAW Reserve          500 MB        1 GB           2 GB
Free                  ~3.75 GB     ~10 GB          ~22 GB
```

---

## 5. Disk Usage

### Storage Budget

```
cShot Installation (~150 MB)
  ├── App binary: 50 MB
  ├── Qt/Web runtime: 70 MB (Tauri bundles minimal)
  └── Assets: 30 MB (icons, default presets)

Initial Library (0 sounds)
  └── 0 MB

Active Library (1000 sounds, ~3s each)
  ├── Sound files: 1.5 GB (44.1k/16/ mono, ~50 KB/s = 150 KB/sound)
  ├── Database: 5 MB
  ├── Waveform cache: 100 MB (SVG + JSON peak data)
  └── Embedding cache: 50 MB

Power Library (10,000 sounds)
  ├── Sound files: 15 GB
  ├── Database: 50 MB
  ├── Waveform cache: 500 MB
  └── Embedding cache: 200 MB

Models (optional, per model)
  ├── INT4 quantized: 50 MB
  ├── INT8 quantized: 200 MB
  ├── FP16: 500 MB
  └── FP32: 2 GB
```

### Disk Strategy

```
1. Content-addressed deduplication
   - Same sound imported twice → stored once
   - Generated variation identical to existing → no new file
   - Hash comparison is instant (check before write)

2. Aggressive compression for waveform caches
   - SVG waveform: ~50 KB raw → ~5 KB brotli
   - Store compressed on disk, decompress on access
   - Peak data (float array) stored as delta-encoded int16 → 4x smaller

3. Automatic cache eviction
   - Waveforms: keep last N sounds, evict oldest
   - Spectrograms: keep last 50 sounds only
   - Embeddings: keep all (small, expensive to recompute)
   - Trash: auto-delete after 30 days

4. Lazy analysis
   - On import: hash + db insert + copy file only
   - Generate waveform: first time user views it
   - Generate spectrogram: if user switches to spectrogram view
   - Generate embedding: on first semantic search
   - Spread analysis across idle CPU time

5. Storage health monitoring
   - Warn at 80% library capacity of available disk
   - Block generation at 95%
   - Show storage breakdown: sounds vs cache vs models
   - One-click "clear unused caches"
```

---

## 6. Inference Speed

### Speed Targets

```
                    Target        Acceptable    Unacceptable
                    ─────────     ──────────    ────────────
Simple gen (kick)    1-2 s        <5 s           >10 s
Medium gen (snare)   2-4 s        <8 s           >15 s
Complex gen (fx)     4-8 s        <15 s          >30 s
Batch gen (10)       10-20 s      <45 s          >60 s
Embedding compute    0.5-1 s      <2 s           >5 s
```

### Inference Speed by Hardware

```
Model: INT8 quantized 1.5s-duration generator

            CPU (FP32)    CPU (INT8)    GPU (FP16)   Apple ANE
            ─────────    ──────────    ──────────   ─────────
Tier 1       12 s          5 s           4 s          N/A
Tier 2        6 s          2.5 s         1.5 s        1.5 s
Tier 3        3 s          1.2 s         0.6 s        0.4 s
```

### Speed Strategy

```
1. Progressive generation (the key insight)
   ┌──────────────────────────────────────────────────────────┐
   │  Step 1: Tiny model (0.2s) → rough draft plays          │
   │         User hears approximate envelope + timbre         │
   │                                                          │
   │  Step 2: Medium model (1.5s) → usable version plays     │
   │         Good enough to decide keep/toss                 │
   │                                                          │
   │  Step 3: Full model (5s) → final quality replaces       │
   │         Seamless upgrade while playing                   │
   │                                                          │
   │  UX: User hears sound at 0.2s, decides at 1.5s,         │
   │      never waits for final quality to make decision      │
   └──────────────────────────────────────────────────────────┘

2. Prompt caching
   - Same prompt = cached result (hash of normalized prompt)
   - Similar prompt = warm-start from nearest cached generation
   - Cache hit: <100ms return
   - Cache miss: full generation

3. Seed-controlled determinism
   - Same prompt + same seed = same output
   - Allows pre-generation of common sounds at idle
   - "Generate 10" just runs 10 seeds through cached model

4. Batch generation optimization
   - Generate multiple sounds in single inference pass
   - GPU utilization much higher with batch > 1
   - Batch of 8: only 2x cost of single generation

5. Latency hiding
   - Start generating while user is still typing prompt
   - Pre-generate default variations before user asks
   - Keep last model warm (loaded in memory) for instant use
```

---

## 7. Battery Impact

### Power Budget

```
Component           Power Draw    Time to 10% battery (50 Wh)
                    ──────────    ───────────────────────────
App idle            ~0.5 W        100 hours
UI interaction      ~2 W          25 hours
Audio playback      ~3 W          16 hours
CPU inference       ~15 W         3.3 hours (12% battery per gen)
GPU inference       ~25 W         2 hours (20% battery per gen)
All max             ~40 W         1.25 hours
```

### Battery Strategy

```
1. Inference power management
   - Detect laptop battery mode
   - On battery: force CPU inference only (no GPU)
   - On battery: limit to INT8 quantized model
   - On battery: disable batch generation (sequential only)
   - On battery: extend timeout tolerance (accept slower)

2. Power-aware model selection
   ┌────────────────┬──────────┬──────────┬──────────┐
   │ Mode           │ Quality  │ Speed    │ Battery  │
   ├────────────────┼──────────┼──────────┼──────────┤
   │ Plugged in     │ Best     │ Fast     │ N/A      │
   │ Battery (high) │ Good     │ Medium   │ -15%/hr  │
   │ Battery (med)  │ Fair     │ Slow     │ -5%/hr   │
   │ Battery (low)  │ Basic    │ Slow     │ -2%/hr   │
   │ Battery saver  │ Offline  │ N/A      │ -0.5%/hr │
   └────────────────┴──────────┴──────────┴──────────┘

3. Idle power
   - Model unloads after 30s of inactivity = saves ~10W
   - UI goes to minimal rendering at 10s idle
   - Audio device releases after 5s silence
   - Background analysis pauses when on battery
   - Save/restore model state to resume quickly

4. Thermal awareness
   - Monitor CPU temperature
   - Throttle inference if > 80°C (fan noise = bad UX)
   - Show thermal indicator in status bar
   - Pause batch operations during thermal throttle
```

---

## 8. Model Quantization

### Quantization Strategy for Consumer Hardware

```
Full precision (FP32)
  Quality: 100%
  Size: 2 GB
  Speed: 1x (baseline)
  RAM: 2 GB + activation memory
  Verdict: NOT for consumer laptops

Half precision (FP16)
  Quality: ~99.5%
  Size: 1 GB
  Speed: 1.5-2x (GPU), 1x (CPU)
  RAM: 1 GB + activation memory
  Verdict: Tier 3 only, GPU recommended

INT8 (dynamic quantization)
  Quality: ~96%
  Size: 500 MB
  Speed: 2-3x (CPU), via VNNI instructions
  RAM: 500 MB + activation memory
  Verdict: DEFAULT for all tiers — best balance

INT8 (static quantization)
  Quality: ~94%
  Size: 500 MB
  Speed: 3-4x (CPU) — calibration dataset needed
  RAM: 500 MB + activation memory
  Verdict: Good for production deploy, needs calibration data

INT4 (GPTQ / AWQ)
  Quality: ~88-92%
  Size: 250 MB
  Speed: 2-3x (CPU), limited GPU kernel support
  RAM: 250 MB + activation memory
  Verdict: Tier 1 only, acceptable quality for draft generation
```

### Implementation Path in Rust/ONNX

```rust
// ONNX Runtime quantization configuration
struct QuantizationConfig {
    // Which quantization to use
    quant_type: QuantType,      // Int4, Int8, Float16, Float32

    // Calibration dataset (for static quantization)
    calibration_data: Option<Vec<AudioExample>>,

    // Which ops to quantize (leave some in FP32 for quality)
    op_exclusions: Vec<OpType>,  // e.g., ["Softmax", "LayerNorm"]

    // Per-channel or per-tensor quantization
    per_channel: bool,

    // Symmetric or asymmetric
    symmetric: bool,
}

// Load quantized model
fn load_quantized_model(path: &Path, config: QuantizationConfig) -> Result<InferenceSession> {
    // 1. Load ONNX model proto
    // 2. Apply quantization (or load pre-quantized)
    // 3. Create ONNX Runtime session with appropriate provider
    // 4. Warm up with dummy input
    // 5. Return ready session
}
```

### Quantized Model Download Strategy

```
First launch:
  - No local model
  - All generation via cloud (if enabled) or DSP presets
  - Prompt: "Download local model for offline generation? (200 MB)"

On download:
  - Download INT8 quantized model as default
  - Show progress bar
  - Verify checksum
  - Store in ~/.cshot/models/quantized/

On update:
  - Check remote manifest for newer model version
  - Download in background
  - Swap atomically (download to temp, rename when complete)
  - Keep old model until new one is confirmed working

Model variants offered:
  - Tiny (50 MB, INT4, fast draft quality)
  - Standard (200 MB, INT8, default — recommended)
  - Quality (500 MB, FP16, for GPU users)
```

---

## 9. Caching

### Multi-Level Cache Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    L1: In-Memory                          │
│  ┌───────────────────────────────────────────────────┐  │
│  │  • Current sound data (mmap window)               │  │
│  │  • Active waveform SVGs (last 20 viewed)           │  │
│  │  • Search results (last 5 queries)                 │  │
│  │  • Currently loaded model weights                  │  │
│  │  Size: ~300 MB typical                             │  │
│  └───────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                    L2: Fast Disk (SSD)                    │
│  ┌───────────────────────────────────────────────────┐  │
│  │  • Waveform SVGs (brotli compressed)              │  │
│  │  • Spectrogram images (WebP)                      │  │
│  │  • Embeddings (binary format)                     │  │
│  │  • Recent generations (last 100)                  │  │
│  │  • Prompt → hash index                            │  │
│  │  Size: ~500 MB default, configurable              │  │
│  └───────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                    L3: Cold Store (Library)               │
│  ┌───────────────────────────────────────────────────┐  │
│  │  • All sound files (content-addressed)            │  │
│  │  • Database (SQLite)                              │  │
│  │  • Original imports                               │  │
│  │  • Full resolution spectrograms (regenerated)     │  │
│  │  Size: 10s of GB                                  │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Cache Invalidation

```rust
enum CacheInvalidation {
    Never,          // Embeddings (immutable, only computed once)
    OnAccess,       // Waveforms (regenerated if missing)
    OnModification, // Spectrograms (regenerated after DSP edit)
    OnVersion,      // Model output (regenerated if model updates)
    TTL(Duration),  // Search results (expire after 30s)
}
```

### Predictive Caching

```
1. Pre-cache on startup
   - Load last 10 accessed sounds into mmap
   - Render last 5 viewed waveforms
   - Warm up last used model (if enough RAM)

2. Pre-cache during idle
   - Generate "sound similar to this" for currently viewed sound
   - Render spectrograms for sounds without them
   - Compute embeddings for un-embedded sounds

3. Pre-cache on prompt
   - User types "kick" → load kick waveforms into memory
   - User types "808" → warm up 808-related model if multiple models
   - User selects collection → background cache all sounds in collection
```

---

## 10. Progressive Generation

### The Core UX Innovation

```
Traditional approach:
  Prompt → [wait 5 seconds] → Hear sound → [maybe] → Regenerate

cShot progressive approach:
  Prompt → Hear draft at 0.2s → Refined at 1.5s → Final at 5s
           ↓                          ↓              ↓
       "Not what I want"     "Close, but harder"   "Perfect!"
           ↓                          ↓              ↓
        New prompt             Refine params      Export
```

### Architecture

```rust
struct ProgressiveGenerator {
    // Three models of increasing quality/size
    draft_model:   Box<dyn LocalGenerator>,   // INT4, 50 MB, 200ms
    preview_model: Box<dyn LocalGenerator>,    // INT8, 200 MB, 1.5s
    final_model:   Box<dyn LocalGenerator>,    // FP16, 500 MB, 5s

    // Streaming state
    current_draft: Option<GeneratedAudio>,
    current_preview: Option<GeneratedAudio>,
    current_final: Option<GeneratedAudio>,
}

impl ProgressiveGenerator {
    async fn generate(&mut self, prompt: &str, seed: u64)
        -> Receiver<ProgressiveUpdate>
    {
        let (tx, rx) = channel(16);

        // Spawn each stage
        let tx1 = tx.clone();
        spawn(async move {
            let draft = self.draft_model.generate(prompt, seed).await;
            tx1.send(ProgressiveUpdate::Draft(draft)).await;
        });

        let tx2 = tx.clone();
        spawn(async move {
            sleep(Duration::from_millis(500)).await; // offset start
            let preview = self.preview_model.generate(prompt, seed).await;
            tx2.send(ProgressiveUpdate::Preview(preview)).await;
        });

        let tx3 = tx.clone();
        spawn(async move {
            sleep(Duration::from_millis(2000)).await; // offset start
            let final_ = self.final_model.generate(prompt, seed).await;
            tx3.send(ProgressiveUpdate::Final(final_)).await;
        });

        rx // Receiver streams updates to UI
    }
}

// UI receives these as they complete
enum ProgressiveUpdate {
    Draft(GeneratedAudio),      // First preview at 200ms
    Preview(GeneratedAudio),    // Better quality at ~2s
    Final(GeneratedAudio),      // Best quality at ~5-8s
    Error(GenerationError),
}
```

### Audio Seamless Transition

```
Draft phase:
  - Generates at 8 kHz, mono, heavily quantized
  - Only captures: envelope shape, basic timbre, rough pitch
  - Enough to know "this is a kick" vs "this is wrong"

Transition to preview:
  - Crossfade from draft to preview (5ms linear crossfade)
  - Preview fills in: spectral detail, transient sharpness, texture
  - User can loop preview while final generates

Transition to final:
  - Seamless replacement
  - Final has full bandwidth, stereo width, dynamic range
  - If user already decided to keep/regenerate, skip final entirely
```

### Early Termination

```
User presses regenerate → cancel all ongoing stages → start new generation
User presses play on different sound → cancel all ongoing stages → free resources
User closes app → cancel all ongoing stages → save partial state? No, discard.
User exports draft → export whatever stage is currently playing
User changes parameter → restart from prompt with new params, keep seed
```

---

## 11. Local Indexing

### Indexing Pipeline

```
Import event
    │
    ▼
┌──────────────────┐
│ Hash + Dedup     │ ← Instant, always
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Copy to content  │ ← Instant, always
│ store            │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ DB insert        │ ← Instant, always
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Queue background │ ← Returns immediately
│ analysis jobs    │
└────────┬─────────┘
         │
    ┌────┴────┬──────────┬──────────┬──────────┐
    ▼         ▼          ▼          ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│ Wave   │ │ LUFS   │ │ Pitch  │ │ Temp   │ │ Embed  │
│ form   │ │ RMS    │ │ Detect │ │ Detect │ │ ing    │
└────────┘ └────────┘ └────────┘ └────────┘ └────────┘
    │         │          │          │          │
    ▼         ▼          ▼          ▼          ▼
┌──────────────────────────────────────────────────┐
│              DB updates (analysis results)         │
└──────────────────────────────────────────────────┘
```

### Priority Queue

```rust
#[derive(Eq, PartialEq)]
enum AnalysisPriority {
    Critical,    // Waveform (needed for UI rendering)
    High,        // Loudness (needed for normalized playback)
    Medium,      // Pitch/tempo (needed for search)
    Low,         // Embedding (needed for semantic search)
    Background,  // Spectrogram, genre classification, etc.
}

struct AnalysisJob {
    sound_id: String,
    priority: AnalysisPriority,
    work: Box<dyn FnOnce(&mut SoundRecord) -> Result<()> + Send>,
}

struct AnalysisQueue {
    jobs: BinaryHeap<Job>, // Priority queue
    active_count: AtomicUsize,
    max_concurrent: usize, // = num_cpus::get() - 1
}
```

### Indexing Performance Targets

```
Operation                Single Sound    1000 Sounds (batch)
                        ─────────────    ───────────────────
Hash + DB insert         <1 ms           500 ms
Waveform extraction      3 ms            3 s
LUFS analysis            20 ms           20 s
Pitch detection          15 ms           15 s
Embedding (CPU INT8)     500 ms          8 min (background, ~1/s)
Spectrogram              30 ms           30 s
Genre classification     100 ms          100 s
All background jobs      700 ms          ~10 min (background)
```

---

## 12. Practical Architecture That Feels Fast

### The Golden Rule

> **Perceived performance > actual performance.**
> cShot must _feel_ instant even when it's not.

### UX Patterns for Speed

```
1. Optimistic UI
   - User presses "Generate" → show waveform placeholder immediately
   - Sound appears to start playing at 0s (really starts at 200ms draft)
   - Result: generation feels instant even when it takes 5s

2. Skeleton loading
   - Waveform loads as gray rectangle → fills in as data arrives
   - Spectrogram loads as gradient → sharpens as frequency bins compute
   - Search shows count immediately → results stream in

3. Infinite scroll with virtualization
   - Library renders only visible items (20-40 sounds)
   - Sound cards are 72px tall → thousands scroll smoothly
   - Waveforms render on demand when card enters viewport

4. Background everything
   - Analysis never blocks UI
   - Export runs in background with progress
   - Model loading shows warm-up animation

5. Thumbnail-first rendering
   - Show tiny waveform thumbnail (200 bytes) in 5ms
   - Show medium waveform (2 KB) at 50ms
   - Show full waveform (20 KB) at 200ms
   - User never waits for full render to see the shape
```

### Resource Budget Summary

```
                  Tier 1 (8GB)      Tier 2 (16GB)     Tier 3 (32GB)
                  ────────────      ─────────────     ─────────────
RAM (app total)   800 MB            2 GB              4 GB
RAM (model)       200 MB (INT8)     500 MB (INT8)     2 GB (FP16/FP32)
Disk (app)        150 MB            150 MB            150 MB
Disk (library)    2 GB / 1000 snd   10 GB / 5000 snd  50 GB / 25000 snd
CPU threads       2-4               4-6               6-8
GPU VRAM          0                 2 GB              6 GB
Gen time (basic)  3-5 s             1.5-3 s           0.5-1 s
Gen time (quality) 8-15 s           4-8 s             1-3 s
Startup time      <2 s              <1.5 s            <1 s
Search latency    <200 ms           <100 ms           <50 ms
```

### Decision Tree (per user action)

```
User action
    │
    ▼
┌──────────────────────────────────────────────┐
│ Can we show something in <100ms?              │
├──────────────────────────────────────────────┤
│ YES → Show skeleton / placeholder / thumbnail │
│ NO  → Show progress indicator + ETA          │
└────────────────────┬─────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────┐
│ Can we show final result in <2s?              │
├──────────────────────────────────────────────┤
│ YES → Compute fully, show result             │
│ NO  → Show progressive result (draft → final)│
│      or return cached result                  │
└──────────────────────────────────────────────┘
```

### What "Feels Fast" Means for Each Feature

```
Feature        | Feels fast if...
───────────────┼─────────────────────────────────────────────
App startup    | Ready to use in <1s (cold), <0.3s (warm)
Import sound   | Visible in library instantly, analysis fills in
Search         | Results appear as you type (debounced 150ms)
Play sound     | Starts within 50ms of click
Generate sound | Audible result within 0.5s, refines over 5s
Export         | File appears in <1s, dialog confirms instantly
Browse library | Scrolls at 60fps, waveforms render on visibility
Edit params    | Parameter change updates sound in <100ms
Batch ops      | Progress bar moves smoothly, cancel responsive
Model load     | <2s to first generation, shows progress
```

---

## Summary

cShot on consumer hardware is a set of deliberate tradeoffs:

1. **CPU inference as default** — GPU is bonus, not requirement
2. **INT8 quantization as standard** — best quality/speed/size intersection
3. **Progressive generation is the UX moat** — draft at 200ms, refine to quality
4. **Memory-mapped everything** — 10,000 sounds without loading them all
5. **Background analysis queue** — UI never blocks on audio processing
6. **Power-aware throttling** — battery mode changes model selection, concurrency
7. **Multi-level cache with predictive warmup** — anticipate what user needs
8. **Perceived performance over actual** — skeletons, placeholders, progressive enhancement

The result: cShot feels instant on a MacBook Air, runs on a 5-year-old Dell, and scales up to a desktop workstation without redesign.
