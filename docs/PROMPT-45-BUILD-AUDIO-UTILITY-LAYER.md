# Prompt 45 — Build the Audio Utility Layer

The foundational audio processing layer for cShot — file handling, analysis, transformation, and export.

---

## 1. Capabilities Overview

| Capability | Description | Priority | Phase |
|------------|-------------|----------|-------|
| File upload | Accept WAV/MP3/FLAC from user's filesystem | P0 | Prototype |
| WAV/MP3 conversion | Convert uploaded files to standard format | P0 | Prototype |
| Trimming | Remove silence from start/end, crop to duration | P0 | Prototype |
| Silence detection | Find silent regions in audio | P1 | MVP |
| Loudness normalization | Normalize to target loudness (LUFS or peak) | P0 | Prototype |
| Clipping detection | Detect and optionally repair clipped samples | P1 | MVP |
| Waveform rendering | Generate thumbnail waveform data for UI | P0 | Prototype |
| Spectrogram rendering | Generate spectrogram image data | P2 | Post-MVP |
| Duration validation | Check that audio is within acceptable range | P0 | Prototype |
| Export formatting | Write WAV at configurable sample rate/bit depth | P0 | Prototype |

---

## 2. Library Decisions

| Need | Library | Language | Why |
|------|---------|----------|-----|
| WAV read/write | `hound` (Rust) | Rust | Pure Rust, simple API, no system deps. Already in stack |
| Audio decoding (MP3, FLAC, etc.) | `symphonia` (Rust) | Rust | Supports WAV, MP3, FLAC, OGG, AIFF. Pure Rust |
| Audio processing | Custom Rust | Rust | Trim, normalize, fade. Simple enough to implement directly |
| Waveform rendering | Custom Rust → SVG/JSON | Rust | 80-point downsampled amplitude data to frontend |
| Signal analysis | Custom Rust | Rust | Spectral centroid, RMS, peak detection. No dependency needed |
| Resampling | Custom or `sample` crate | Rust | `sample` crate for high-quality resampling if needed |
| Spectrogram | Custom Rust (FFT via `rustfft`) | Rust | FFT-based spectrogram computation |

### Why Not Python Libraries

```
librosa and pedalboard are excellent but:
- They add a Python dependency to the Rust/Tauri stack
- Python subprocess calls add 200-500ms overhead per operation
- Rust can do everything needed for MVP without external processes
- When Python is needed (model inference), it's a separate process

Strategy: Pure Rust for utility layer. Python only for model inference.
```

---

## 3. Module Structure

```
src-tauri/src/audio/
├── mod.rs                    # Re-exports, public API
├── process.rs                # Core processing pipeline
├── analyze.rs                # Feature extraction, analysis
├── waveform.rs               # Waveform thumbnail generation
├── spectrogram.rs            # Spectrogram computation (P2)
├── export.rs                 # WAV export with formatting
├── import.rs                 # File upload + decoding
├── normalize.rs              # Loudness/peak normalization
├── trim.rs                   # Silence trimming + duration crop
├── detect.rs                 # Silence detection, clipping detection
└── resample.rs               # Sample rate conversion
```

### Module Dependency Graph

```
import.rs ──→ process.rs ──→ export.rs
                 │
                 ├── analyze.rs
                 ├── waveform.rs
                 ├── normalize.rs
                 ├── trim.rs
                 ├── detect.rs
                 └── resample.rs
```

---

## 4. Function Signatures

### `process.rs` — Pipeline Orchestration

```rust
pub struct ProcessingPipeline {
    pub trim_silence: bool,
    pub normalize_peak: bool,
    pub normalize_loudness: bool,
    pub target_peak_db: f32,       // default: -1.0
    pub target_loudness_lufs: f32, // default: -14.0
    pub fade_in_ms: f32,           // default: 2.0
    pub fade_out_ms: f32,          // default: 10.0
    pub min_duration_ms: f32,      // default: 50.0
    pub max_duration_ms: f32,      // default: 5000.0
}

impl ProcessingPipeline {
    pub fn new() -> Self;
    pub fn process(&self, audio: &[f32], sample_rate: u32) -> Result<ProcessedAudio>;
    pub fn process_with_config(audio: &[f32], sample_rate: u32, config: &ProcessConfig) -> Result<ProcessedAudio>;
}

pub struct ProcessedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: f32,
    pub peak_db: f32,
    pub rms_db: f32,
    pub dc_offset_removed: bool,
    pub silence_trimmed: bool,
    pub was_normalized: bool,
}

pub struct ProcessConfig {
    pub trim_threshold_db: f32,     // default: -60
    pub normalize_target_db: f32,   // default: -1
    pub fade_in_ms: f32,
    pub fade_out_ms: f32,
}
```

### `import.rs` — File Upload and Decoding

```rust
pub fn import_audio(path: &Path) -> Result<AudioFile>;
pub fn decode_to_f32(path: &Path) -> Result<(Vec<f32>, u32, u16)>;
pub fn validate_format(path: &Path) -> Result<FileFormat>;
pub fn convert_to_mono(audio: &[f32], channels: u16) -> Vec<f32>;
pub fn estimate_duration(file_size: u64, format: FileFormat) -> f32;

pub enum FileFormat {
    Wav, Mp3, Flac, Ogg, Aiff, Unknown,
}

pub struct AudioFile {
    pub path: PathBuf,
    pub format: FileFormat,
    pub duration_ms: f32,
    pub sample_rate: u32,
    pub channels: u16,
    pub file_size_bytes: u64,
}
```

### `trim.rs` — Silence Trimming and Cropping

```rust
pub fn trim_silence(audio: &[f32], sample_rate: u32, threshold_db: f32) -> Vec<f32>;
pub fn crop(audio: &[f32], start_ms: f32, duration_ms: f32, sample_rate: u32) -> Vec<f32>;
pub fn apply_fade(audio: &mut [f32], sample_rate: u32, fade_in_ms: f32, fade_out_ms: f32);
pub fn pad_to_minimum(audio: &[f32], min_duration_ms: f32, sample_rate: u32) -> Vec<f32>;
```

### `normalize.rs` — Loudness and Peak

```rust
pub fn peak_normalize(audio: &mut [f32], target_db: f32) -> f32;
pub fn loudness_normalize(audio: &mut [f32], sample_rate: u32, target_lufs: f32) -> Result<f32>;
pub fn compute_peak_db(audio: &[f32]) -> f32;
pub fn compute_rms_db(audio: &[f32]) -> f32;
pub fn compute_integrated_loudness(audio: &[f32], sample_rate: u32) -> Result<f32>;
```

### `analyze.rs` — Feature Extraction

```rust
pub struct AudioAnalysis {
    pub duration_ms: f32,
    pub sample_rate: u32,
    pub channels: u16,
    pub peak_db: f32,
    pub rms_db: f32,
    pub dynamic_range_db: f32,
    pub spectral_centroid_hz: f32,
    pub spectral_rolloff_hz: f32,
    pub zero_crossing_rate: f32,
    pub crest_factor: f32,
    pub estimated_pitch_hz: Option<f32>,
    pub has_clipping: bool,
    pub clipping_percent: f32,
    pub dc_offset: f32,
    pub is_silent: bool,
}

pub fn analyze(audio: &[f32], sample_rate: u32) -> AudioAnalysis;
pub fn compute_spectral_centroid(audio: &[f32], sample_rate: u32) -> f32;
pub fn compute_zero_crossing_rate(audio: &[f32]) -> f32;
pub fn compute_crest_factor(audio: &[f32]) -> f32;
pub fn compute_dynamic_range(audio: &[f32]) -> f32;
pub fn detect_clipping(audio: &[f32]) -> (bool, f32);
pub fn detect_silence(audio: &[f32], threshold_db: f32) -> Vec<SilenceRegion>;

pub struct SilenceRegion {
    pub start_ms: f32,
    pub end_ms: f32,
    pub duration_ms: f32,
}
```

### `waveform.rs` — UI Display Data

```rust
pub fn generate_waveform(audio: &[f32], num_points: usize) -> Vec<f32>;
pub fn generate_waveform_min_max(audio: &[f32], num_points: usize) -> Vec<(f32, f32)>;
pub fn normalize_waveform(points: &[f32]) -> Vec<f32>;

// Default: 80 points for SoundSlot thumbnails
// Returns values in range [-1.0, 1.0]
```

### `export.rs` — WAV Output

```rust
pub struct ExportConfig {
    pub sample_rate: u32,       // default: 44100
    pub bit_depth: u16,         // default: 24
    pub channels: u16,          // default: 1 (mono)
    pub include_metadata: bool, // default: true
    pub filename_template: String, // default: "{type}_{bpm}_{key}_{seed}"
}

pub struct ExportResult {
    pub path: PathBuf,
    pub filename: String,
    pub duration_ms: f32,
    pub sample_rate: u32,
    pub bit_depth: u16,
    pub file_size_bytes: u64,
    pub format: String,
}

pub fn export_wav(audio: &[f32], output_path: &Path, config: &ExportConfig) -> Result<ExportResult>;
pub fn generate_filename(analysis: &AudioAnalysis, template: &str) -> String;
pub fn validate_output_path(path: &Path) -> Result<()>;
```

### `detect.rs` — Quality Checks

```rust
pub fn check_audio_quality(audio: &[f32], sample_rate: u32) -> QualityReport;

pub struct QualityReport {
    pub is_usable: bool,
    pub issues: Vec<QualityIssue>,
    pub score: f32,  // 0.0 - 1.0
}

pub enum QualityIssue {
    TooShort { duration_ms: f32, min_ms: f32 },
    TooLong { duration_ms: f32, max_ms: f32 },
    Clipping { percent: f32 },
    TooQuiet { rms_db: f32 },
    TooNoisy { snr_db: f32 },
    DcOffset { offset: f32 },
    Silent,
}
```

---

## 5. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Failed to decode audio: {0}")]
    DecodeError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(u32),
    
    #[error("Audio too short: {duration_ms}ms (minimum {min_ms}ms)")]
    TooShort { duration_ms: f32, min_ms: f32 },
    
    #[error("Audio too long: {duration_ms}ms (maximum {max_ms}ms)")]
    TooLong { duration_ms: f32, max_ms: f32 },
    
    #[error("Silent audio")]
    SilentAudio,
    
    #[error("Export failed: {0}")]
    ExportError(String),
    
    #[error("Normalization failed: {0}")]
    NormalizationError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
```

---

## 6. Test Files

```
src-tauri/tests/
├── audio_process_test.rs      # trim, normalize, fade pipeline
├── audio_analyze_test.rs      # Feature extraction accuracy
├── audio_import_test.rs       # File format handling
├── audio_export_test.rs       # WAV output correctness
├── audio_waveform_test.rs     # Waveform generation
├── audio_detect_test.rs       # Quality checks
└── fixtures/                  # Test audio files
    ├── sine_440.wav           # 440Hz sine, 1 second
    ├── sine_440_clipped.wav   # Clipped version for detection
    ├── silence.wav            # Silent file
    ├── short.wav              # 10ms file (below minimum)
    ├── long.wav               # 10s file (above typical max)
    ├── stereo.wav             # Stereo file for mono conversion
    ├── kick_reference.wav     # Professional kick
    └── noise_floor.wav        # Low SNR for quality check
```

### Critical Test Cases

```rust
#[test]
fn test_trim_silence_removes_leading_zeros() {
    let mut audio = vec![0.0; 44100]; // 1 second of silence
    audio.extend_from_slice(&generate_sine(440, 1.0, 44100)); // 1 second of tone
    audio.extend_from_slice(&vec![0.0; 44100]); // trailing silence
    
    let trimmed = trim_silence(&audio, 44100, -60.0);
    assert!((trimmed.len() as f32 / 44100.0 - 1.0).abs() < 0.01);
}

#[test]
fn test_normalize_does_not_clip() {
    let audio = generate_sine(440, 0.5, 44100); // half amplitude
    let mut normalized = audio.clone();
    peak_normalize(&mut normalized, -1.0);
    
    let peak = normalized.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak <= 1.0);
    assert!((peak - 10f32.powf(-1.0/20.0)).abs() < 0.01);
}

#[test]
fn test_detect_clipping_finds_clipped_samples() {
    let mut audio = generate_sine(440, 1.0, 44100);
    for sample in audio.iter_mut() {
        if sample.abs() > 0.99 { *sample = 1.0; } // clip
    }
    let (has_clipping, percent) = detect_clipping(&audio);
    assert!(has_clipping);
    assert!(percent > 0.0);
}

#[test]
fn test_import_wav_roundtrip() {
    let audio = generate_sine(440, 1.0, 44100);
    let temp = tempfile::NamedTempFile::new().unwrap();
    export_wav(&audio, temp.path(), &ExportConfig::default()).unwrap();
    let (imported, sr, ch) = decode_to_f32(temp.path()).unwrap();
    assert_eq!(sr, 44100);
    assert_eq!(ch, 1);
    assert!((imported.len() as f32 / sr as f32 - 1.0).abs() < 0.01);
}

#[test]
fn test_reject_silent_audio() {
    let audio = vec![0.0; 44100];
    let report = check_audio_quality(&audio, 44100);
    assert!(!report.is_usable);
    assert!(report.issues.iter().any(|i| matches!(i, QualityIssue::Silent)));
}

#[test]
fn test_convert_stereo_to_mono() {
    let stereo = vec![0.5, -0.5, 0.3, -0.3]; // interleaved
    let mono = convert_to_mono(&stereo, 2);
    assert_eq!(mono.len(), 2); // averaged
    assert!((mono[0] - 0.0).abs() < 0.01); // (0.5 + -0.5) / 2 = 0.0
}
```

---

## 7. CLI Utilities (Development Only)

```rust
// Binary: cshot-audio
// Usage during development for testing audio module independently

cargo run --bin cshot-audio analyze path/to/sound.wav
  → Duration: 1.23s | SR: 44100 | Peak: -0.5dB | RMS: -12.3dB
  → Spectral centroid: 2450Hz | ZCR: 0.12 | Clipping: 0%

cargo run --bin cshot-audio trim path/to/sound.wav --threshold -60
  → Trimmed: 412ms removed from start, 23ms from end

cargo run --bin cshot-audio normalize path/to/sound.wav --target -1
  → Normalized: peak moved from -6.2dB to -1.0dB (gain: +5.2dB)

cargo run --bin cshot-audio convert input.mp3 output.wav --sr 44100 --bits 24
  → Converted: MP3 → WAV (44.1kHz/24-bit/mono)

cargo run --bin cshot-audio waveform path/to/sound.wav --points 80
  → [0.12, 0.45, 0.78, 0.92, ...]  // 80 JSON float values

cargo run --bin cshot-audio check path/to/sound.wav
  → Quality: 7.8/10
  → Issues: none
```

---

## 8. Processing Pipeline Flow

```
Audio Input
    │
    ▼
┌──────────────────────────────────────┐
│ import_audio()                        │
│  - Validate format                    │
│  - Decode to f32                      │
│  - Convert to mono if stereo          │
│  - Return (samples, sr, channels)     │
└──────────────────┬───────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│ check_audio_quality()                 │
│  - Reject silent audio               │
│  - Reject too short/long             │
│  - Warn on clipping                  │
│  - Return QualityReport              │
└──────────────────┬───────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│ trim_silence()                        │
│  - Find first/last above threshold   │
│  - Crop to active region             │
│  - Return trimmed buffer              │
└──────────────────┬───────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│ apply_fade()                          │
│  - Apply 2ms fade-in                 │
│  - Apply 10ms fade-out               │
│  - Modify buffer in-place            │
└──────────────────┬───────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│ peak_normalize()                      │
│  - Find peak amplitude               │
│  - Calculate gain to target -1dBFS    │
│  - Apply gain to all samples          │
│  - Return applied gain                │
└──────────────────┬───────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│ analyze()                             │
│  - Compute all features              │
│  - Return AudioAnalysis               │
└──────────────────┬───────────────────┘
                   │
                   ▼
┌──────────────────────────────────────┐
│ generate_waveform()                   │
│  - Downsample to 80 points           │
│  - Return Vec<f32> for UI            │
└──────────────────┬───────────────────┘
                   │
                   ▼
          Ready for export or preview
```

---

## 9. Performance Targets

| Operation | Target Time | Notes |
|-----------|-------------|-------|
| WAV decode (10s file) | <10ms | hound is fast |
| MP3 decode (10s file) | <50ms | symphonia performs well |
| Trim silence (10s file) | <5ms | Simple scan |
| Peak normalize (10s file) | <5ms | Single pass |
| Full analysis (10s file) | <20ms | Spectral centroid requires FFT |
| Waveform thumbnail | <2ms | Downsampling only |
| WAV export (10s file) | <10ms | hound write |
| Full pipeline | <100ms | All operations combined |

---

## 10. Implementation Order

```
Phase 1 — Prototype (build first):
  1. import.rs — decode WAV files (hound only)
  2. process.rs — trim + normalize + fade pipeline
  3. waveform.rs — 80-point thumbnail
  4. analyze.rs — basic analysis (duration, RMS, peak)
  5. export.rs — WAV write at 44.1kHz/24-bit/mono
  6. detect.rs — basic quality checks (silent, clipping, too short)

Phase 2 — MVP (add second):
  7. import.rs — add symphonia for MP3/FLAC/AIFF support
  8. normalize.rs — loudness normalization (EBU R128)
  9. trim.rs — crop to duration, smart silence detection
  10. analyze.rs — spectral centroid, ZCR, crest factor
  11. detect.rs — comprehensive quality report

Phase 3 — Post-MVP (add last):
  12. resample.rs — sample rate conversion
  13. spectrogram.rs — FFT-based spectrogram rendering
  14. analyze.rs — pitch detection, onset detection
```
