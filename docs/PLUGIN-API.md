# cShot Plugin API Surface

## Overview

cShot's DSP engine is designed to be extracted into a standalone Rust crate
that can be used in VST3/AU/CLAP plugin contexts. This document defines the
API surface that separates app-level concerns from the audio engine.

## Core Types (plugin-safe)

These types have no dependency on Tauri, SQLite, or any app infrastructure:

```rust
// In cshot-dsp or crate::audio::*

// --- Synthesis ---
pub struct ResynthesisParams { ... }
impl ResynthesisParams {
    pub fn params_for_sound_type(sound_type: SoundType, pitch: f32, duration_ms: f32) -> Self;
    pub fn resynthesize(&self) -> Vec<f32>;
    pub fn with_seed(self, seed: u64) -> Self;
    pub fn randomize(&self, variation: f32) -> Self;
    pub fn to_variant(&self, name: &str) -> Self;
}

// --- Advanced Synthesis (v2) ---
pub fn generate_resonant_impact(duration_ms: f32, freq_hz: f32, resonance: f32) -> Vec<f32>;
pub fn generate_ui_click() -> Vec<f32>;
pub fn generate_tonal_perc(duration_ms: f32, pitch_hz: f32) -> Vec<f32>;
pub fn generate_cinematic_boom(duration_ms: f32) -> Vec<f32>;
pub fn generate_bass_hit(duration_ms: f32) -> Vec<f32>;

// --- Analysis ---
pub struct AudioAnalysis { ... }
pub fn analyze_audio(samples: &[f32], sample_rate: u32, channels: u16) -> AudioAnalysis;

// --- DSP ---
pub fn low_pass(samples: &mut [f32], cutoff_hz: f32);
pub fn high_pass(samples: &mut [f32], cutoff_hz: f32);
pub fn pitch_shift(samples: &[f32], ratio: f32) -> Vec<f32>;
pub fn transient_enhance(samples: &mut [f32], boost_db: f32);
pub fn normalize_peak(samples: &mut [f32], target_db: f32);
pub fn biquad_low_shelf(samples: &mut [f32], freq: f32, gain_db: f32, q: f32);
pub fn biquad_high_shelf(samples: &mut [f32], freq: f32, gain_db: f32, q: f32);
pub fn biquad_peaking(samples: &mut [f32], freq: f32, gain_db: f32, q: f32);
pub fn spectral_balance(samples: &mut [f32], sound_type: &str);
pub fn true_peak_limiter(samples: &mut [f32], ceiling_db: f32);
pub fn noise_gate_tail(samples: &mut [f32], threshold_db: f32);
pub fn apply_envelope(samples: &mut [f32], attack_s: f32, decay_s: f32);

// --- Hybrid Engine (v2) ---
pub struct HybridParams { ... }
pub fn hybrid_reconstruct(original: &[f32], analysis: &AudioAnalysis, params: &HybridParams) -> Vec<f32>;
pub fn layer_transient(original: &[f32], synth: &mut [f32], blend: f32);
pub fn spectral_blend(original: &[f32], synth: &mut [f32], amount: f32);

// --- Transform ---
pub struct TransformParams { ... }
pub fn apply_dsp_transforms(samples: &mut Vec<f32>, params: &TransformParams);
pub fn transform_with_params(source: &[f32], params: &ResynthesisParams) -> Vec<f32>;

// --- Recreate ---
pub fn compute_similarity(original: &[f32], recreated: &[f32], analysis: &AudioAnalysis) -> SimilarityReport;
pub fn compute_multiband_similarity(original: &[f32], recreated: &[f32]) -> f32;
pub fn compute_multiband_spectral_match(original: &[f32], recreated: &[f32]) -> f32;
pub fn params_from_analysis(analysis: &AudioAnalysis, samples: &[f32]) -> ResynthesisParams;
pub fn generate_approximations(...) -> Vec<ApproximationResult>;

// --- Prompt DSP Mapping ---
pub struct PromptDspControls { ... }
pub fn parse_prompt_rich(text: &str) -> PromptDspControls;
impl PromptDspControls {
    pub fn to_resynthesis_params(&self, base: &ResynthesisParams) -> ResynthesisParams;
    pub fn to_dsp_params(&self) -> DspParams;
}
```

## App-Level Types (not plugin-safe)

These types depend on Tauri, SQLite, file I/O, or the app state:

```rust
// In crate::commands, crate::generator, crate::db, crate::storage

pub struct AppState { ... }       // Tauri state
pub struct SoundResult { ... }     // DB-backed sound
pub struct SoundEntry { ... }      // DB row
pub fn save_and_return(...) -> SoundResult;  // writes to disk + DB
```

## Plugin Strategy

### Phase 1: Extract cshot-dsp crate (now)

The audio engine (`crate::audio`) is already mostly independent with:
- Pure functions with no side effects
- No SQLite, no Tauri, no file I/O
- Advanced synthesis modules (FM, resonant, hybrid, cinematic)
- Multi-band similarity engine for recreation

### Phase 2: Plugin Frameworks (evaluate)

| Framework | Language | UI | CLAP | VST3 | Notes |
|-----------|----------|----|------|------|-------|
| nih-plug | Rust | egui/baseview | Yes | Yes | Most mature Rust plugin framework |
| vst3-sys | Rust | Custom | No | Yes | Low-level bindings, more work |
| baseview | Rust | Custom | Via nih-plug | Via nih-plug | Window system abstraction |
| CLAP-first | C/Rust | Any | Native | Via wrapper | Newer standard, cleaner API |

**Recommendation**: Start with `nih-plug` for CLAP support, add VST3 wrapping later.

### Phase 3: DAW Automation & MIDI

The plugin binary now exposes:
- **8 DAW automation parameters**: Type, Pitch, Decay, Brightness, Distortion, Noise, Sub, Click
- **MIDI Note On**: Triggers one-shot generation on rising edge
- **MIDI Velocity**: Maps to output gain (0.0-1.0)
- **MIDI Pitch Bend**: +/- 12 semitones pitch modulation
- **Latency**: 0 samples (offline generation, streamed playback)

### Phase 4: Parameter Serialization

Plugin presets are JSON-based:
```json
{
  "name": "my_kick",
  "sound_type": "kick",
  "pitch_hz": 60.0,
  "decay_ms": 200.0,
  "brightness": 0.5,
  "distortion": 0.3,
  "noise_amount": 0.0,
  "sub_gain": 0.4
}
```

## CLI Harness

`cargo run --bin cshot-cli` provides headless access:

```bash
cshot-cli generate "punchy kick 140bpm" --output kick.wav
cshot-cli analyze input.wav
cshot-cli transform input.wav "darker shorter" --output transformed.wav
cshot-cli recreate input.wav --count 4 --output best.wav
cshot-cli render "warm sub bass" --count 10 --dir ./my_pack
cshot-cli list-recipes
cshot-cli benchmark
```

## Portability

All DSP functions:
- Use only `std` (no platform-specific deps)
- Operate on `Vec<f32>` or `&[f32]`
- No heap allocation in hot paths (pre-allocated buffers)
- Deterministic for same seed + params
- No threading (caller manages threads)
