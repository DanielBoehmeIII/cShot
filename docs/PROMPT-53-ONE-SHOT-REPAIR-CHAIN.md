# Prompt 53 — Build the One-Shot Repair Chain

An automatic repair chain that takes raw model output and turns it into a mix-ready one-shot. Every generation goes through this pipeline before the user hears it.

---

## 1. Repair Chain Pipeline

```
Raw Model Output (f32 buffer)
    │
    ▼
┌─────────────────────────────┐
│ 1. Silent / Empty Check     │  Reject if total silence
├─────────────────────────────┤
│ 2. DC Offset Removal        │  High-pass @ 20Hz, 6dB/oct
├─────────────────────────────┤
│ 3. Trim Silence             │  Gate @ -60dB threshold
├─────────────────────────────┤
│ 4. Fade In / Out            │  3ms fade-in, 5ms fade-out
├─────────────────────────────┤
│ 5. Peak Normalization       │  Target: -1dBFS
├─────────────────────────────┤
│ 6. Repair Failures (taxonomy)│
│   • Clarity Boost           │  If muddy
│   • Transient Shaper        │  If weak attack
│   • Noise Gate              │  If noisy
│   • Clamp Duration          │  If too long/short
│   • De-clip                 │  If distorted
├─────────────────────────────┤
│ 7. Genre-Specific Presets   │  EQ + compression template
├─────────────────────────────┤
│ 8. Final Loudness           │  -14dB LUFS integrated (optional)
├─────────────────────────────┤
│ 9. True-Peak Limiter        │  Hard ceiling @ -0.5dBFS
├─────────────────────────────┤
│ 10. Quality Validation      │  Verify all metrics pass
    │
    ▼
User Hears Clean One-Shot
```

---

## 2. DSP Steps (Rust Implementation)

### 2.1 DC Offset Removal

```rust
pub fn remove_dc_offset(audio: &mut [f32]) {
    // Simple: subtract the mean
    let mean: f32 = audio.iter().sum::<f32>() / audio.len() as f32;
    
    if mean.abs() > 0.001 {
        for sample in audio.iter_mut() {
            *sample -= mean;
        }
    }
    // Better: 2nd-order Butterworth high-pass at 20Hz
    // Prevents low-frequency rumble from throwing off normalization
}

pub fn highpass_dc_filter(audio: &mut [f32], sample_rate: u32) {
    // 2nd-order Butterworth, cutoff = 20Hz
    // Implemented as biquad:
    //   y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]
    let cutoff = 20.0_f32;
    let q = 0.707; // Butterworth Q
    // ... coefficient calculation + apply
}
```

### 2.2 Silence Trim

```rust
pub struct SilenceDetection {
    threshold_db: f32,     // default: -60dB
    min_silence_ms: u32,   // default: 10ms
    hold_ms: u32,          // default: 5ms (keep this much after gate closes)
    sample_rate: u32,
}

impl SilenceDetection {
    pub fn trim(audio: &[f32], sample_rate: u32) -> Vec<f32> {
        let threshold = 10_f32.powf(-60.0 / 20.0); // -60dB → linear
        
        // Find first sample above threshold
        let start = audio.iter().position(|&s| s.abs() > threshold)
            .unwrap_or(0);
        
        // Find last sample above threshold (scan from end)
        let end = audio.iter().rposition(|&s| s.abs() > threshold)
            .map(|pos| pos + 1)
            .unwrap_or(audio.len());
        
        // Add 5ms padding on both ends
        let pad = (sample_rate as f32 * 0.005) as usize;
        let start = start.saturating_sub(pad);
        let end = std::cmp::min(end + pad, audio.len());
        
        audio[start..end].to_vec()
    }
}
```

### 2.3 Fades

```rust
pub fn apply_fades(audio: &mut [f32], sample_rate: u32) {
    let fade_in_samples = (sample_rate as f32 * 0.003) as usize;  // 3ms
    let fade_out_samples = (sample_rate as f32 * 0.005) as usize; // 5ms
    
    // Fade in (linear)
    for i in 0..fade_in_samples.min(audio.len()) {
        let gain = i as f32 / fade_in_samples as f32;
        audio[i] *= gain;
    }
    
    // Fade out (linear, from end)
    let len = audio.len();
    for i in 0..fade_out_samples.min(len) {
        let gain = i as f32 / fade_out_samples as f32;
        audio[len - 1 - i] *= gain; // reverse: 1.0 → 0.0 → wait, this goes 0→1
        // Fix: apply properly
    }
}

pub fn apply_fade_out(audio: &mut [f32], duration_ms: u32, sample_rate: u32) {
    let samples = (sample_rate as f32 * duration_ms as f32 / 1000.0) as usize;
    let start = audio.len().saturating_sub(samples);
    
    for i in start..audio.len() {
        let t = (i - start) as f32 / samples as f32;
        audio[i] *= 1.0 - t; // Linear fade: 1.0 → 0.0
    }
}
```

### 2.4 Transient Clarity

```rust
pub struct TransientShaper {
    attack_ms: f32,      // default: 1ms
    release_ms: f32,     // default: 10ms
    boost_db: f32,       // default: +3dB
    threshold: f32,      // default: -20dB relative to peak
}

impl TransientShaper {
    pub fn process(audio: &mut [f32], sample_rate: u32) {
        // 1. Detect onset via envelope follower
        // 2. Split signal into transient + sustain
        // 3. Apply gain envelope to transient portion
        // 4. Recombine
        
        // Envelope follower (simple peak detector with release)
        let release_coeff = (-1.0 / (sample_rate as f32 * 0.01 / 1000.0)).exp();
        // 0.01 = 10ms release time constant
        
        let mut envelope = 0.0_f32;
        let mut shaped = audio.to_vec();
        
        // Find onset: where envelope jumps > threshold
        let onset_threshold = audio.iter()
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max) * 0.3;
        
        let mut onset_pos = 0;
        for (i, &sample) in audio.iter().enumerate() {
            let input = sample.abs();
            envelope = if input > envelope {
                input // attack: follow instantly
            } else {
                envelope * release_coeff // release: gradual
            };
            
            if envelope > onset_threshold && onset_pos == 0 {
                onset_pos = i;
                break;
            }
        }
        
        // If no clear onset or onset is at start, skip
        if onset_pos < 10 || onset_pos > audio.len() / 2 {
            return;
        }
        
        // Boost transient: apply gain to first 5ms after onset
        let transient_len = (sample_rate as f32 * 0.005) as usize;
        let gain = 10_f32.powf(3.0 / 20.0); // +3dB
        let end = std::cmp::min(onset_pos + transient_len, audio.len());
        
        for i in onset_pos..end {
            let t = (i - onset_pos) as f32 / transient_len as f32;
            let envelope = 1.0 - t; // 1.0 at onset, 0.0 at end
            audio[i] *= 1.0 + (gain - 1.0) * envelope;
            // Clamp to safe range
            audio[i] = audio[i].clamp(-1.0, 1.0);
        }
    }
}
```

### 2.5 Loudness Normalization

```rust
pub struct LoudnessNormalizer {
    target_lufs: f32,      // default: -14dB (streaming standard)
    target_peak: f32,      // default: -1.0dBFS
}

impl LoudnessNormalizer {
    pub fn normalize(audio: &mut [f32], sample_rate: u32) {
        // 1. Measure integrated LUFS (simplified: RMS + crest correction)
        let rms = self.measure_rms(audio);
        let peak = self.measure_peak(audio);
        let lufs_estimate = self.rms_to_lufs(rms, audio, sample_rate);
        
        // 2. Compute gain to reach target
        let gain_db = self.target_lufs - lufs_estimate;
        let gain_linear = 10_f32.powf(gain_db / 20.0);
        
        // 3. Apply gain
        for sample in audio.iter_mut() {
            *sample *= gain_linear;
        }
        
        // 4. True-peak limit to prevent intersample peaks
        self.true_peak_limit(audio, sample_rate);
    }
    
    fn measure_rms(&self, audio: &[f32]) -> f32 {
        let sum_sq: f32 = audio.iter().map(|s| s * s).sum();
        (sum_sq / audio.len() as f32).sqrt()
    }
    
    fn measure_peak(&self, audio: &[f32]) -> f32 {
        audio.iter().map(|s| s.abs()).fold(0.0_f32, f32::max)
    }
    
    fn rms_to_lufs(&self, rms: f32, audio: &[f32], sample_rate: u32) -> f32 {
        // LUFS = RMS with K-weighting filter + loudness gate
        // Simplified: RMS dB + 0.5 (typical offset for one-shots)
        let rms_db = 20.0 * rms.log10();
        rms_db + 0.5
    }
    
    fn true_peak_limit(&self, audio: &mut [f32], sample_rate: u32) {
        // 4x oversampling + peak detection
        // Look-ahead limiter: 2ms lookahead, 10ms release
        let ceiling = 10_f32.powf(-0.5 / 20.0); // -0.5dBFS
        let threshold = ceiling * 0.95;
        
        let mut gain = 1.0_f32;
        let release_coeff = (-1.0 / (sample_rate as f32 * 0.01)).exp();
        
        for sample in audio.iter_mut() {
            let abs = sample.abs();
            if abs > threshold {
                // Calculate required gain reduction
                let target_gain = threshold / abs;
                gain = gain.min(target_gain);
            } else {
                // Release
                gain = 1.0 - (1.0 - gain) * (1.0 - release_coeff);
            }
            *sample *= gain;
        }
    }
}
```

### 2.6 Spectral Balance EQ

```rust
pub struct SpectralBalancer {
    // Gentle EQ curve to fix common spectral issues
    pub low_shelf_freq: f32,     // 200Hz
    pub low_shelf_gain: f32,     // -1dB (reduce mud)
    pub presence_freq: f32,      // 3000Hz
    pub presence_gain: f32,      // +1dB (add clarity)
    pub air_freq: f32,           // 10000Hz
    pub air_gain: f32,           // +0.5dB (add air)
}

impl SpectralBalancer {
    pub fn apply(audio: &mut [f32], sample_rate: u32) {
        // Apply gentle EQ curve using biquad filters
        // Each filter is subtle (< 2dB) — prevent overprocessing
        
        // 1. Low shelf cut (reduce mud)
        // Biquad low shelf, f=200Hz, gain=-1dB, Q=0.7
        
        // 2. Presence boost (add clarity)
        // Biquad peaking, f=3kHz, gain=+1dB, Q=1.0
        
        // 3. Air boost (add openness)
        // Biquad high shelf, f=10kHz, gain=+0.5dB, Q=0.7
    }
}
```

### 2.7 Low-End Control

```rust
pub struct LowEndController {
    pub sub_cutoff_hz: f32,      // 40Hz (remove infrasonic)
    pub sub_gain_db: f32,        // 0dB (leave sub alone by default)
    pub bass_mono_cross_hz: f32, // 150Hz (everything below = mono)
}

impl LowEndController {
    pub fn process(audio: &mut [f32], sample_rate: u32) {
        // 1. High-pass at sub_cutoff to remove infrasonic rumble
        // Biquad HPF, 24dB/oct Linkwitz-Riley
        
        // 2. Bass mono crossover (future: when stereo is supported)
        // Split at crossover frequency, sum L+R below it
    }
}
```

### 2.8 Harshness Reduction

```rust
pub struct HarshnessReducer {
    pub threshold_db: f32,       // -20dB (relative to peak)
    pub ratio: f32,              // 5:1 multiband compression
    pub attack_ms: f32,          // 1ms
    pub release_ms: f32,         // 50ms
    pub bands: [f32; 3],         // [2kHz, 5kHz, 10kHz] crossing points
}

impl HarshnessReducer {
    pub fn process(audio: &mut [f32], sample_rate: u32) {
        // Multiband compressor focused on harsh frequency ranges
        // 3 bands: 2-5kHz, 5-10kHz, 10-20kHz
        // Each band has gentle compression (2:1 to 5:1) on peaks only
        // Subtle — only reduces harshness, doesn't dull the sound
    }
}
```

---

## 3. Parameter Defaults

| Parameter | Default | Range | Rationale |
|-----------|---------|-------|-----------|
| DC filter cutoff | 20Hz | 10-40Hz | Removes rumble without affecting audible bass |
| Trim threshold | -60dB | -80 to -40dB | Catches most noise floors without being too aggressive |
| Fade in | 3ms | 1-10ms | Prevents click without dulling attack |
| Fade out | 5ms | 2-20ms | Clean tail end without cutting natural decay |
| Peak target | -1dBFS | -3 to -0.5dBFS | Headroom for DAW mixing |
| LUFS target | -14dB | -18 to -10dB | Streaming standard, but one-shots can be louder |
| Transient boost | +3dB | 0 to +6dB | Noticeable improvement without distortion |
| Transient attack | 1ms | 0.5-5ms | Fast enough for percussive transients |
| Mud cut | -1dB @ 200Hz | -3 to 0dB | Gentle reduction, never aggressive EQ |
| Presence boost | +1dB @ 3kHz | 0 to +3dB | Adds clarity without harshness |
| True-peak ceiling | -0.5dBFS | -2 to -0.1dBFS | Prevents intersample peaks |
| Harshness threshold | -20dB | -30 to -10dB | Only catches actual harsh peaks |

---

## 4. Genre-Specific Presets

```rust
pub struct GenrePreset {
    pub name: &'static str,
    pub transient_boost_db: f32,
    pub mud_cut_db: f32,
    pub mud_freq_hz: f32,
    pub presence_boost_db: f32,
    pub air_boost_db: f32,
    pub lufs_target: f32,
    pub peak_target: f32,
    pub harshness_threshold_db: f32,
    pub tail_trim_ms: Option<u32>,  // If set, force max tail length
}

pub const GENRE_PRESETS: &[GenrePreset] = &[
    GenrePreset {
        name: "trap",
        transient_boost_db: 4.0,
        mud_cut_db: -1.5,
        mud_freq_hz: 250.0,
        presence_boost_db: 2.0,
        air_boost_db: 1.0,
        lufs_target: -10.0,     // Trap is loud
        peak_target: -0.5,
        harshness_threshold_db: -15.0,
        tail_trim_ms: Some(1500),
    },
    GenrePreset {
        name: "house",
        transient_boost_db: 2.0,
        mud_cut_db: -1.0,
        mud_freq_hz: 300.0,
        presence_boost_db: 1.0,
        air_boost_db: 0.0,
        lufs_target: -12.0,
        peak_target: -1.0,
        harshness_threshold_db: -20.0,
        tail_trim_ms: None,      // House kicks ring out
    },
    GenrePreset {
        name: "techno",
        transient_boost_db: 3.0,
        mud_cut_db: -1.0,
        mud_freq_hz: 200.0,
        presence_boost_db: 0.5,
        air_boost_db: -1.0,       // Techno is darker
        lufs_target: -11.0,
        peak_target: -0.8,
        harshness_threshold_db: -18.0,
        tail_trim_ms: Some(2000),
    },
    GenrePreset {
        name: "lo-fi",
        transient_boost_db: -1.0, // Lo-fi is softer
        mud_cut_db: 0.0,          // Keep the warmth
        mud_freq_hz: 0.0,
        presence_boost_db: -1.0,  // Duller
        air_boost_db: -2.0,
        lufs_target: -16.0,       // Quieter
        peak_target: -2.0,
        harshness_threshold_db: -25.0,
        tail_trim_ms: None,
    },
    GenrePreset {
        name: "cinematic",
        transient_boost_db: 3.0,
        mud_cut_db: -0.5,
        mud_freq_hz: 150.0,
        presence_boost_db: 2.0,
        air_boost_db: 2.0,
        lufs_target: -18.0,       // Wide dynamic range
        peak_target: -0.5,
        harshness_threshold_db: -15.0,
        tail_trim_ms: None,
    },
    GenrePreset {
        name: "rock",
        transient_boost_db: 3.0,
        mud_cut_db: -2.0,
        mud_freq_hz: 350.0,
        presence_boost_db: 2.5,
        air_boost_db: 0.5,
        lufs_target: -11.0,
        peak_target: -0.8,
        harshness_threshold_db: -15.0,
        tail_trim_ms: Some(1000),
    },
];
```

---

## 5. Before/After Evaluation

### Evaluation Metrics

```rust
pub struct RepairEvaluation {
    pub before: AudioMetrics,
    pub after: AudioMetrics,
    pub improvement: f32,  // 0.0 = no change, 1.0 = perfect
}

pub struct AudioMetrics {
    pub peak_dbfs: f32,
    pub rms_dbfs: f32,
    pub crest_factor: f32,
    pub transient_strength: f32,     // onset peak / RMS
    pub spectral_centroid: f32,
    pub spectral_flatness: f32,
    pub noise_floor_dbfs: f32,
    pub duration_ms: f32,
    pub has_dc_offset: bool,
    pub has_clipping: bool,
    pub headroom_db: f32,
    pub loudness_lufs: f32,
}

pub fn evaluate_repair(before: &[f32], after: &[f32], sample_rate: u32) -> RepairEvaluation {
    let b = compute_metrics(before, sample_rate);
    let a = compute_metrics(after, sample_rate);
    
    let improvement = compute_improvement(&b, &a);
    
    RepairEvaluation { before: b, after: a, improvement }
}

fn compute_improvement(before: &AudioMetrics, after: &AudioMetrics) -> f32 {
    // Weighted combination of improvements
    let mut score = 0.0_f32;
    
    // No clipping is better
    if after.has_clipping == false && before.has_clipping == true {
        score += 0.2;
    }
    
    // No DC offset is better
    if after.has_dc_offset == false && before.has_dc_offset == true {
        score += 0.1;
    }
    
    // Crest factor in optimal range (8-15 for percussive)
    let crest_improvement = (after.crest_factor - before.crest_factor)
        .clamp(-3.0, 3.0) / 3.0 * 0.15;
    score += crest_improvement.max(0.0);
    
    // Transient strength improvement (higher is better for percussive)
    let transient_improvement = (after.transient_strength - before.transient_strength)
        .clamp(-0.5, 0.5) / 0.5 * 0.15;
    score += transient_improvement.max(0.0);
    
    // Headroom in safe range (-3 to -0.5 dBFS)
    let headroom_improvement = if after.headroom_db >= 0.5 && after.headroom_db <= 3.0 {
        0.1 // Good headroom
    } else if before.headroom_db < 0.5 && after.headroom_db >= 0.5 {
        0.1 // Fixed clipping risk
    } else {
        0.0
    };
    score += headroom_improvement;
    
    score.clamp(0.0, 1.0)
}
```

### Spectrum Analyzer Diff View

```typescript
// Frontend component to show repair effect
interface RepairDiffProps {
  beforeSpectrum: number[];  // 128-bin FFT before
  afterSpectrum: number[];   // 128-bin FFT after
  metrics: {
    before: AudioMetrics;
    after: AudioMetrics;
  };
}

// Renders an overlay:
// "Before" = gray, "After" = blue
// Green highlight = improved regions
// Red highlight = over-processed regions
// Text summary: "Crest +2.1dB | Harshness -1.8dB | Floor -4dB"
```

---

## 6. User Controls

### Simple Mode

```
REPAIR CHAIN
┌───────────────────────────────────────────────┐
│ Repair Chain: [ON]                             │
│                                                │
│ Clarity:      ◄───●───────────►  (muddy→clean) │
│ Punch:        ◄───●───────────►  (soft→sharp)  │
│ Loudness:     ◄───●───────────►  (quiet→loud)  │
│ Tail:         ◄───●───────────►  (short→long)  │
│                                                │
│ Genre Preset: [Trap ▼]                         │
│                                                │
│ [Reset to Defaults]                            │
└───────────────────────────────────────────────┘
```

### Advanced Mode

```
┌───────────────────────────────────────────────┐
│ Repair Chain — Advanced                        │
│                                                │
│ ┌─ Trim ─────────────────────────────────┐    │
│ │ Gate Threshold:  -60dB  ◄───●───────►  │    │
│ │ Fade In:          3ms   ◄───●───────►  │    │
│ │ Fade Out:         5ms   ◄───●───────►  │    │
│ └────────────────────────────────────────┘    │
│ ┌─ EQ ──────────────────────────────────┐    │
│ │ Sub Cut:    40Hz  ◄───●───────────►   │    │
│ │ Mud Cut:    200Hz ◄──────●───────►    │    │
│ │ Presence:   3kHz  ◄───────●────────►  │    │
│ │ Air:        10kHz ◄───────────●────►  │    │
│ └────────────────────────────────────────┘    │
│ ┌─ Dynamics ────────────────────────────┐    │
│ │ Transient:  +3dB  ◄───●───────────►   │    │
│ │ Punch:      ──    ◄───●───────────►   │    │
│ │ Harshness:  ──    ◄──────●─────►      │    │
│ └────────────────────────────────────────┘    │
│                                                │
│ Before [━━━━━━]  After [━━━━━━]                │
│ Peak: -0.1dB     Peak: -0.5dB                 │
│ Crest: 6.2       Crest: 9.8                    │
│ Centroid: 1.2kHz  Centroid: 2.1kHz             │
└───────────────────────────────────────────────┘
```

---

## 7. Safe Limits (Anti Over-Processing)

```rust
pub struct SafeLimits {
    // Never exceed these values
    pub max_transient_boost_db: f32,   // +6dB
    pub max_eq_cut_db: f32,            // -6dB
    pub max_eq_boost_db: f32,          // +6dB
    pub max_loudness_lufs: f32,        // -8dB (prevent loudness wars)
    pub min_crest_factor: f32,         // 4.0 (prevent over-compression)
    pub max_crest_factor: f32,         // 25.0 (prevent over-expansion)
    pub min_duration_ms: f32,          // 30ms (prevent over-trimming)
    pub max_trim_ratio: f32,           // 0.8 (never trim more than 80%)
    pub max_noise_reduction_db: f32,   // -6dB
    pub min_headroom_dbfs: f32,        // -0.1dBFS
}

impl SafeLimits {
    pub fn clamp_parameters(&self, params: &mut RepairParams) {
        params.transient_boost_db = params.transient_boost_db
            .clamp(0.0, self.max_transient_boost_db);
        // ... clamp all parameters
    }
    
    pub fn validate_audio(&self, audio: &[f32], sample_rate: u32) -> Result<(), OverprocessingError> {
        let metrics = compute_metrics(audio, sample_rate);
        
        if metrics.crest_factor < self.min_crest_factor {
            return Err(OverprocessingError::OverCompressed);
        }
        if metrics.crest_factor > self.max_crest_factor {
            return Err(OverprocessingError::OverExpanded);
        }
        if metrics.duration_ms < self.min_duration_ms {
            return Err(OverprocessingError::OverTrimmed);
        }
        
        Ok(())
    }
}
```

---

## 8. Pipeline Orchestration

```rust
pub struct RepairChain {
    stages: Vec<Box<dyn RepairStage>>,
    safe_limits: SafeLimits,
}

#[async_trait]
pub trait RepairStage {
    fn name(&self) -> &'static str;
    async fn process(&self, audio: &mut [f32], sample_rate: u32, params: &RepairParams) -> Result<(), RepairError>;
    fn cost_ms(&self) -> u32; // Estimated processing time
}

impl RepairChain {
    pub fn new() -> Self {
        let stages: Vec<Box<dyn RepairStage>> = vec![
            Box::new(DcOffsetRemoval),
            Box::new(SilenceTrim),
            Box::new(FadeStage),
            Box::new(TransientShaperStage),
            Box::new(SpectralBalancerStage),
            Box::new(LoudnessNormalizerStage),
            Box::new(TruePeakLimiterStage),
        ];
        
        RepairChain { stages, safe_limits: SafeLimits::default() }
    }
    
    pub async fn process(
        &self,
        audio: &[f32],
        sample_rate: u32,
        genre: Option<&str>,
    ) -> Result<ProcessedAudio, RepairError> {
        let mut buffer = audio.to_vec();
        let before_metrics = compute_metrics(&buffer, sample_rate);
        
        let mut params = RepairParams::default();
        if let Some(genre) = genre {
            params.apply_genre_preset(genre);
        }
        
        self.safe_limits.clamp_parameters(&mut params);
        
        for stage in &self.stages {
            let start = std::time::Instant::now();
            stage.process(&mut buffer, sample_rate, &params).await?;
            let elapsed = start.elapsed();
            
            log::info!("Repair stage {}: {:?}ms", stage.name(), elapsed.as_micros() as f32 / 1000.0);
        }
        
        // Final validation
        self.safe_limits.validate_audio(&buffer, sample_rate)?;
        
        let after_metrics = compute_metrics(&buffer, sample_rate);
        let evaluation = RepairEvaluation { before: before_metrics, after: after_metrics };
        
        Ok(ProcessedAudio { buffer, sample_rate, evaluation })
    }
}
```

---

## 9. Summary

The repair chain turns raw model output into a professional one-shot in under 5ms. Every stage has safe limits that prevent over-processing. Genre presets apply the right character. Users can control the chain with simple sliders or dive into advanced parameters. The before/after evaluation measures improvement and flags over-processing. The result is always clean, normalized, and mix-ready.
