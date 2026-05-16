# Prompt 62 — Make Generation Reliable

A reliability system for cShot generation. Handle every failure mode, recover automatically when possible, and never show the user a broken sound.

---

## 1. Failure Mode Coverage

Every failure mode is classified, detected, and has a prescribed response. The system handles 12 failure modes organized into 4 categories:

### Model Failures (The API returned something bad)

| Failure | Detection | Action | User Sees |
|---------|-----------|--------|-----------|
| Failed model call | HTTP error / timeout | Retry with fallback model | "Generation taking longer than usual..." |
| Slow generation | Latency > P90 threshold | Cancel, retry with faster model | Brief spinner, no visible failure |
| Bad outputs | SoundScore < threshold | Auto-regenerate (up to 3 tries) | Sound appears normally after retry |
| Corrupted files | WAV parsing error, NaN samples | Discard, regenerate | "Something went wrong. Regenerating..." |

### Audio Issues (The sound itself is broken)

| Failure | Detection | Action | User Sees |
|---------|-----------|--------|-----------|
| Clipping | Peak > 0dBFS, >0.1% samples at ±1.0 | Repair: soft-clip reconstruction or reduce gain | Subtle quality badge adjustment |
| Silence | RMS < -60dB threshold | Auto-regenerate | Brief spinner, no visible failure |
| Wrong duration | Outside type-expected range | Repair: trim or extend tail | Duration badge updates |
| DC offset | Mean > 0.001 | Remove with HPF @ 20Hz | Invisible (always applied) |

### Quality Issues (Technically OK but not good enough)

| Failure | Detection | Action | User Sees |
|---------|-----------|--------|-----------|
| Muddy | Spectral flatness > 0.7, centroid < 800Hz | EQ repair: low-mid cut + presence boost | "Clarity Boost applied" badge |
| Weak transient | Crest factor < 8, onset strength < 0.3 | Transient shaper: +3dB boost | "Punch Enhanced" badge |
| Too noisy | Noise floor > -50dB, spectral flatness > 0.7 | Spectral noise gate | "Noise Reduced" badge |
| Boring/generic | Nearest neighbor distance < 0.1 | Add saturation + filter movement | "Character added" badge |

### Prompt Issues (The model didn't understand the user)

| Failure | Detection | Action | User Sees |
|---------|-----------|--------|-----------|
| Not prompt-aligned | CLAP score < 0.3 | Flag for user action, provide suggestion | "Doesn't match your prompt — try adding [suggestion]" |
| Wrong type | Type classifier disagrees with prompt | Flag, suggest type correction | Type badge shows detected (not expected) type |
| Ambiguous prompt | No clear sound type detected | Suggest types as chips | "What kind of sound? [Kick] [Snare] [FX]" |

---

## 2. Retry System

### Retry Strategy

```
Generation Request
    │
    ▼
┌─────────────────┐
│ Attempt 1        │  Primary provider, balanced quality
│ (default config) │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
  Success   Failure
    │         │
    │         ▼
    │   ┌─────────────────┐
    │   │ Attempt 2        │  Fallback provider, fast quality
    │   │ (different model)│
    │   └────────┬────────┘
    │            │
    │       ┌────┴────┐
    │       ▼         ▼
    │     Success   Failure
    │       │         │
    │       │         ▼
    │       │   ┌─────────────────┐
    │       │   │ Attempt 3        │  DSP template synthesis
    │       │   │ (last resort)    │
    │       │   └────────┬────────┘
    │       │            │
    │       │       ┌────┴────┐
    │       │       ▼         ▼
    │       │     Success   Failure
    │       │       │         │
    │       │       │         ▼
    │       │       │   Return error to user
    │       │       │   "Couldn't generate this sound.
    │       │       │    Try a different prompt."
    │       │       │
    ▼       ▼       ▼
   Return result to user
```

### Retry Configuration

```rust
pub struct RetryConfig {
    pub max_attempts: u8,                    // 3
    pub timeout_ms: u32,                     // 15000 (15s total before giving up)
    pub per_attempt_timeout_ms: u32,         // 8000 (8s per attempt)
    pub backoff_ms: u32,                     // 500
    pub exponential_backoff: bool,           // true (500, 1000, 2000)
    
    // Which errors trigger a retry
    pub retryable_errors: Vec<ErrorType>,
    pub non_retryable_errors: Vec<ErrorType>,
}

// Retryable: network timeout, 500 error, rate limit, model timeout
// Non-retryable: invalid prompt, auth failure, unsupported sound type

pub enum ErrorType {
    NetworkTimeout,
    Http5xx,
    Http4xx,
    RateLimited,
    ModelTimeout,
    InvalidPrompt,
    AuthFailure,
    CorruptedOutput,
    SilentOutput,
    QualityBelowThreshold,
}
```

### Retry Counters and Metrics

| Metric | Target | Alert At |
|--------|--------|----------|
| First-attempt success rate | >95% | <90% |
| Retry success rate | >50% | <30% |
| Total failure rate (after all retries) | <2% | >5% |
| Average retries per generation | <0.1 | >0.3 |
| Mean time to resolution | <8s | >12s |

---

## 3. Validation Checks

### Pre-Generation Validation

Run before submitting to the model API. Catches issues early.

```rust
pub fn validate_generation_request(req: &GenerationRequest) -> Result<(), ValidationError> {
    // 1. Prompt validation
    if req.prompt.trim().is_empty() {
        return Err(ValidationError::EmptyPrompt);
    }
    if req.prompt.len() > 200 {
        return Err(ValidationError::PromptTooLong);
    }
    
    // 2. BPM validation
    if let Some(bpm) = req.bpm {
        if bpm < 40 || bpm > 300 {
            return Err(ValidationError::BpmOutOfRange);
        }
    }
    
    // 3. Duration validation
    if let Some(ms) = req.duration_ms {
        if ms < 50 || ms > 10000 {
            return Err(ValidationError::DurationOutOfRange);
        }
    }
    
    // 4. Reference validation
    if let Some(ref audio) = req.reference {
        if audio.is_empty() {
            return Err(ValidationError::EmptyReference);
        }
        if audio.len() as f32 / req.sample_rate as f32 > 60.0 {
            return Err(ValidationError::ReferenceTooLong);
        }
    }
    
    // 5. Sound type validation (if explicitly specified)
    if let Some(ref st) = req.sound_type {
        if !VALID_SOUND_TYPES.contains(st) {
            return Err(ValidationError::UnknownSoundType);
        }
    }
    
    Ok(())
}
```

### Post-Generation Validation

Run on every generation before the user hears it. This is the quality gate.

```rust
pub struct ValidationResult {
    pub passed: bool,
    pub score: f64,
    pub issues: Vec<AudioIssue>,
    pub severity: Severity,  // None, Minor, Major, Critical
}

pub fn validate_generated_sound(
    audio: &[f32],
    sample_rate: u32,
    expected_type: Option<SoundType>,
) -> ValidationResult {
    let mut issues = Vec::new();
    let mut all_passed = true;
    
    // 1. Silent check (fast, <1ms)
    let rms = compute_rms(audio);
    if rms < 0.001 {
        issues.push(AudioIssue::Silence);
        all_passed = false;
    }
    
    // 2. Clipping check
    let peak = compute_peak(audio);
    let clipped_samples = audio.iter().filter(|&&s| s.abs() >= 1.0).count();
    if peak >= 1.0 || clipped_samples as f64 / audio.len() as f64 > 0.001 {
        issues.push(AudioIssue::Clipping { 
            peak_dbfs: 20.0 * peak.log10(), 
            clipped_percent: clipped_samples as f64 / audio.len() as f64 * 100.0 
        });
        // Not a "failed" check — clipping is repairable
    }
    
    // 3. Duration check
    let duration_ms = audio.len() as f64 / sample_rate as f64 * 1000.0;
    if let Some(ref expected) = expected_type {
        let (min_ms, max_ms) = expected.duration_range();
        if duration_ms < min_ms as f64 {
            issues.push(AudioIssue::TooShort { actual_ms: duration_ms, expected_min_ms: min_ms });
        }
        if duration_ms > max_ms as f64 {
            issues.push(AudioIssue::TooLong { actual_ms: duration_ms, expected_max_ms: max_ms });
        }
    }
    
    // 4. DC offset check
    let mean = audio.iter().sum::<f32>() / audio.len() as f32;
    if mean.abs() > 0.001 {
        issues.push(AudioIssue::DcOffset { dc_value: mean });
        // Always repairable
    }
    
    // 5. NaN/Inf check
    if audio.iter().any(|s| s.is_nan() || s.is_infinite()) {
        issues.push(AudioIssue::CorruptedOutput);
        all_passed = false;
    }
    
    // 6. SoundScore check (if computed)
    // SoundScore < 30 is critical, 30-50 is minor, >50 is fine
    
    let severity = classify_severity(&issues);
    
    ValidationResult {
        passed: all_passed && severity != Severity::Critical,
        score: compute_validation_score(&issues, rms, peak),
        issues,
        severity,
    }
}
```

### Validation Thresholds by Sound Type

| Sound Type | Min Duration | Max Duration | Min RMS | Max Peak | Min Crest |
|-----------|-------------|-------------|---------|----------|-----------|
| Kick | 100ms | 2000ms | 0.05 | 0.95 | 8.0 |
| Snare | 80ms | 1500ms | 0.05 | 0.95 | 7.0 |
| Hi-hat (closed) | 30ms | 500ms | 0.03 | 0.90 | 6.0 |
| Hi-hat (open) | 100ms | 1000ms | 0.03 | 0.90 | 6.0 |
| Clap | 80ms | 1200ms | 0.05 | 0.95 | 7.0 |
| Bass/808 | 200ms | 4000ms | 0.08 | 0.95 | 5.0 |
| Percussion | 50ms | 2000ms | 0.04 | 0.95 | 6.0 |
| FX | 100ms | 5000ms | 0.03 | 0.95 | 4.0 |
| Texture | 500ms | 10000ms | 0.02 | 0.90 | 3.0 |

---

## 4. Fallback Models

### Fallback Chain

```
Primary Provider                     Fallback 1                     Fallback 2
─────────────────                    ──────────                     ──────────
ElevenLabs SFX API  ──fail──►    Stable Audio API  ──fail──►   DSP Template Synth
                                                                  │
                                                                  ▼
                                                              Simple sine + noise
                                                              generation (last resort)
```

### DSP Template Synthesis (Last Resort)

When all API models fail, cShot can generate basic sounds locally using DSP. This ensures users always get something, even if it's simpler.

```rust
pub fn synthesize_fallback(prompt: &str, sound_type: SoundType) -> Vec<f32> {
    match sound_type {
        SoundType::Kick => {
            // Basic kick: sine sweep + noise burst
            let sample_rate = 44100;
            let duration = 0.5; // seconds
            let mut audio = vec![0.0_f32; (sample_rate as f32 * duration) as usize];
            
            for (i, sample) in audio.iter_mut().enumerate() {
                let t = i as f32 / sample_rate as f32;
                let freq = 150.0 * (-t * 15.0).exp(); // pitch sweep: 150Hz → 0
                let amp = (-t * 8.0).exp();            // amplitude envelope
                *sample = (2.0 * std::f32::consts::PI * freq * t).sin() * amp;
            }
            
            // Apply repair chain (trim, normalize, fade)
            audio
        },
        SoundType::Snare => {
            // Basic snare: noise burst + tone
            // Similar approach with noise generator
            vec![]
        },
        // ... other types
        _ => vec![] // empty, will be caught by silent check
    }
}
```

### Provider Health Check

```rust
pub struct ProviderHealth {
    pub provider: String,
    pub is_reachable: bool,
    pub last_latency_ms: u64,
    pub rolling_success_rate: f64,   // Last 100 requests
    pub current_quota_remaining: u32,
    pub rate_limited_until: Option<DateTime<Utc>>,
    pub degradation_started_at: Option<DateTime<Utc>>,
}

pub struct HealthChecker {
    providers: Vec<Box<dyn ModelProvider>>,
    check_interval: Duration,   // 30 seconds
}

impl HealthChecker {
    pub async fn check_all(&self) -> Vec<ProviderHealth> {
        let mut results = Vec::new();
        for provider in &self.providers {
            let health = provider.check_health().await;
            results.push(health);
        }
        results
    }
    
    /// Pick the best available provider for a given sound type
    pub fn select_provider(&self, sound_type: SoundType) -> Option<&Box<dyn ModelProvider>> {
        let healthy: Vec<_> = self.providers.iter()
            .filter(|p| p.is_healthy())
            .collect();
        
        // Prefer specialized providers, fall back to general
        for provider in &healthy {
            if provider.specialization() == Some(sound_type) {
                return Some(provider);
            }
        }
        
        healthy.first() // Any healthy provider
    }
}
```

---

## 5. Repair Chain Integration

### Repair Chain Flow (Prompt 53 integration)

Every generation passes through the repair chain before the user hears it:

```
Model Output (raw f32 buffer)
    │
    ▼
┌──────────────────────────────┐
│ 1. Validate Raw Output        │  ← Silent? Corrupted? Clipped?
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ 2. Apply DSP Pipeline        │
│    • Remove DC offset        │  ← Always applied
│    • Trim silence            │  ← Always applied
│    • Fade in/out (3ms/5ms)  │  ← Always applied
│    • Peak normalization      │  ← Always applied (-1dBFS)
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ 3. Detect Audio Issues       │  ← Muddy? Weak transient? Noisy?
└──────────┬───────────────────┘
           │
     ┌─────┴─────┐
     │           │
  No Issues   Issues Found
     │           │
     │           ▼
     │    ┌────────────────────┐
     │    │ 4. Apply Repairs    │
     │    │    • EQ clarity     │
     │    │    • Transient      │
     │    │    • Noise gate     │
     │    │    • Duration fix   │
     │    └────────┬───────────┘
     │             │
     │             ▼
     │    ┌────────────────────┐
     │    │ 5. Re-validate     │
     │    │    • Pass? → Done  │
     │    │    • Fail? → Flag  │
     │    └────────────────────┘
     │
     ▼
┌──────────────────────────────┐
│ 6. Final Validation          │
│    • SoundScore > threshold  │
│    • No remaining issues     │
│    • Duration in range       │
│    • Peak < 0dBFS            │
└──────────┬───────────────────┘
           │
           ▼
    User Hears Clean Sound
```

### Repair Logging

```rust
pub struct RepairLog {
    pub generation_id: String,
    pub issues_detected: Vec<DetectedIssue>,
    pub repairs_applied: Vec<AppliedRepair>,
    pub final_validation: ValidationResult,
    pub total_repair_time_ms: f32,
}

pub struct DetectedIssue {
    pub issue_type: AudioIssue,    // Muddy, WeakTransient, etc.
    pub severity: f64,             // 0.0 (barely) to 1.0 (severe)
    pub detection_method: String,  // "rule_low_centroid", "ml_classifier", etc.
    pub detected_at_ms: f32,       // Position in audio
}

pub struct AppliedRepair {
    pub repair_type: String,       // "eq_clarity_boost", "transient_shaper"
    pub parameters: HashMap<String, f64>,
    pub improvement_score: f64,    // 0.0 (no change) to 1.0 (perfect)
    pub duration_ms: f32,
}
```

---

## 6. Timeout Recovery

### Generation Timeout Strategy

```
Elapsed Time    Action
─────────────   ──────
      0s        Submit to primary provider
      3s        No response yet → continue waiting
      5s        No response yet → log as "slow generation" 
      8s        ⚠ Timeout threshold → Cancel primary
                → Submit to fallback provider (faster model)
      13s       Fallback responding → continue
      16s       Fallback responding → continue
      18s       Fallback done → return result
                Total time: 18s (worst case with fallback)
```

### Timeout Configuration

```rust
pub struct TimeoutConfig {
    // Per-provider timeouts
    pub primary_provider_timeout_ms: u32,    // 8000
    pub fallback_provider_timeout_ms: u32,   // 10000
    
    // Total generation timeout (from user click)
    pub total_timeout_ms: u32,               // 20000
    
    // Recovery actions
    pub cancel_previous_on_fallback: bool,   // true (cancel primary after timeout)
    pub show_progress_after_ms: u32,         // 3000 (show spinner after 3s)
    
    // User communication
    pub warning_message_at_ms: u32,          // 5000
    // "This is taking longer than usual..."
    pub fallback_message: &'static str,      // "Switching to a faster model..."
    pub error_message: &'static str,         // "Couldn't generate this sound."
}
```

### Timeout Edge Cases

| Scenario | Behavior |
|----------|----------|
| Primary returns after timeout but before fallback | Use whichever completes first with higher quality |
| Both timeout | Return error, user-facing message |
| Primary returns corrupted, fallback still running | Wait for fallback, discard primary |
| User navigates away during generation | Cancel all pending requests (invisible to user) |
| Multiple rapid generations | Queue with max 3 concurrent, cancel oldest if queue full |

---

## 7. Duplicate Output Detection

### Problem

Users who generate the same prompt twice sometimes get identical (or near-identical) outputs due to the same seed or model determinism. This erodes trust — if the sound isn't unique, why use cShot?

### Detection

```rust
pub struct DuplicateDetector {
    recent_hashes: LruCache<String, Vec<u8>>,  // SHA-256 of recent generations
    max_history: usize,                          // 100 most recent
    similarity_threshold: f64,                   // 0.95 (cross-correlation)
}

impl DuplicateDetector {
    pub fn check_duplicate(&self, audio: &[f32], prompt: &str) -> Option<DuplicateInfo> {
        // 1. Fast check: compare SHA-256 hash
        let hash = sha256(audio);
        if self.recent_hashes.values().any(|h| *h == hash) {
            return Some(DuplicateInfo {
                confidence: 1.0,
                method: "exact_hash",
                original_prompt: None, // exact match detected
            });
        }
        
        // 2. Slower check: cross-correlation for near-duplicates
        for (original_prompt, original_hash) in self.recent_hashes.iter() {
            let original = load_audio_by_hash(original_hash);
            if let Some(original) = original {
                let similarity = cross_correlate(audio, &original);
                if similarity > self.similarity_threshold {
                    return Some(DuplicateInfo {
                        confidence: similarity,
                        method: "cross_correlation",
                        original_prompt: Some(original_prompt.clone()),
                    });
                }
            }
        }
        
        None
    }
    
    pub fn record_generation(&mut self, audio: &[f32], prompt: String) {
        let hash = sha256(audio);
        self.recent_hashes.put(prompt, hash);
    }
}
```

### Response to Duplicate

```rust
if let Some(dup) = detector.check_duplicate(&audio, &prompt) {
    match dup.confidence {
        c if c > 0.99 => {
            // Exact duplicate: discard and retry with forced new seed
            return regenerate_with_new_seed();
        },
        c if c > 0.90 => {
            // Very similar: warn user, offer "make more different"
            return GenerationResult {
                audio,
                duplicate_warning: Some("This sounds very similar to a recent generation. Click 'Make Different' for more variety."),
            };
        },
        _ => {
            // Slightly similar: note internally, don't warn user
            metrics.record_near_duplicate();
        },
    }
}
```

---

## 8. User-Facing Error States

### Error State Matrix

| Situation | User Message | Action | Severity |
|-----------|-------------|--------|----------|
| Generation taking >5s | Progress bar + "Generating your sound..." | Wait (auto-resolve) | Info |
| Generation taking >8s | "This is taking longer than usual..." | Wait (auto-resolve) | Warning |
| Generation failed (retrying) | "Something went wrong. Retrying with a different model..." | Wait (auto-resolve) | Info |
| Generation failed (all retries exhausted) | "Couldn't generate this sound. Try a different prompt or check your connection." | [Try Again] [Change Prompt] | Error |
| Network offline | "You're offline. cShot needs an internet connection to generate sounds." | [Retry] | Error |
| API rate limited | "Too many generations. Wait a moment or upgrade your plan." | [Wait] [Upgrade] | Warning |
| Invalid prompt | "Try a more specific prompt, like 'punchy trap kick 140bpm'" | [Edit Prompt] | Info |
| SoundScore very low | "This sound is lower quality than usual. Try regenerating or tweaking your prompt." | [Regenerate] [Keep Anyway] | Warning |
| Budget exhausted | "You've used your generation budget for this month. Upgrade for unlimited generations." | [Upgrade] [Wait] | Info |

### UI Components

```typescript
// Status bar always shows generation state
type GenerationStatus = 
  | { state: 'idle' }
  | { state: 'generating'; progress: number; message: string }     // 0-100%
  | { state: 'retrying'; attempt: number; message: string }        // "Retry 2/3"
  | { state: 'fallback'; message: string }                         // "Using faster model"
  | { state: 'repairing'; repairs: string[] }                      // "Applying clarity boost..."
  | { state: 'complete'; soundId: string; score: number }
  | { state: 'error'; code: string; message: string; retryable: boolean }

// Error toast with actions
interface ErrorToast {
  type: 'warning' | 'error';
  title: string;
  message: string;
  actions: ErrorAction[];
  autoDismissMs: number | null;  // null = requires user action
  id: string;
}

interface ErrorAction {
  label: string;
  action: 'retry' | 'change_prompt' | 'upgrade' | 'dismiss' | 'keep_anyway';
  primary: boolean;
}
```

---

## 9. Logging and Debugging

### Structured Logging

Every generation produces a structured log entry for debugging and analysis.

```rust
pub struct GenerationLogEntry {
    // Identification
    pub generation_id: String,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    
    // Request
    pub prompt: String,
    pub sound_type: Option<String>,
    pub bpm: Option<u32>,
    pub key: Option<String>,
    pub reference_present: bool,
    pub quality_tier: String,
    
    // Execution
    pub provider_attempts: Vec<ProviderAttempt>,
    pub total_latency_ms: u64,
    pub repair_time_ms: f32,
    pub validation_result: ValidationResult,
    
    // Result
    pub success: bool,
    pub sound_score: Option<f64>,
    pub duration_ms: f64,
    pub peak_dbfs: f64,
    pub rms_dbfs: f64,
    
    // Cost
    pub cost_cents: f64,
    pub provider_used: String,
    pub model_version: String,
}

pub struct ProviderAttempt {
    pub provider: String,
    pub model_version: String,
    pub latency_ms: u64,
    pub success: bool,
    pub error: Option<String>,
    pub cost_cents: f64,
}
```

### Debug Mode (Developer Tool)

Activated with `--debug` flag or environment variable `CSHOT_DEBUG=1`. Shows detailed information about each generation.

```
Debug Panel (toggle with Ctrl+D):
┌─────────────────────────────────────────────────────┐
│ Generation #42 — "punchy trap kick 140bpm"          │
│──────────────────────────────────────────────────────│
│                                                      │
│ Pipeline Timeline:                                   │
│  [──── Generation ────] 4.2s (ElevenLabs SFX v2)   │
│    ├─ Submit: 0.03s                                  │
│    ├─ Process: 3.8s                                  │
│    └─ Download: 0.37s                                │
│  [─── Validation ───] 0.004s                         │
│    ├─ Silent check: 0.3ms ✓                          │
│    ├─ Clipping check: 0.5ms ✓                        │
│    ├─ Duration check: 0.2ms ✓ (412ms, kick range)   │
│    └─ SoundScore: 0.002ms → 72                      │
│  [─── Repair ───────] 0.8ms                          │
│    ├─ DC offset: 0.1ms ✓                             │
│    ├─ Trim silence: 0.2ms (trimmed 12ms front)      │
│    ├─ Fade: 0.1ms ✓                                  │
│    ├─ Normalize: 0.1ms (peak: -0.3→-1.0dBFS)        │
│    └─ Transient: 0.3ms (+2.1dB, crest: 7→9)        │
│                                                      │
│ Validation: PASSED (score: 0.92)                    │
│ Cost: $0.08 (ElevenLabs SFX)                        │
│ SoundScore: 72 (Mix-ready: 85, Punch: 68,           │
│              Clarity: 70, Uniqueness: 65)           │
│                                                      │
│ [Copy Log] [Save Report] [View Raw Audio]            │
└─────────────────────────────────────────────────────┘
```

### Debug Console Logging

```rust
// Log levels for development
pub enum LogLevel {
    Error,      // Generation failures, API errors
    Warn,       // Retries, slow generation, low quality
    Info,       // Successful generations, user actions
    Debug,      // Detailed pipeline timings (development only)
    Trace,      // Sample-level audio data (rarely used)
}

// Every log entry is structured JSON for machine parsing
// and human-readable text for console viewing
```

### Production Monitoring Dashboard

```
┌─────────────────────────────────────────────────────────┐
│ Generation Reliability Dashboard                         │
│                                                          │
│ Last 24 hours    ┌──────────────────────────────────────┐│
│ Success Rate:    │ 98.2% ||||||||||||||||||||||||||||  ││
│ P50 Latency:     │ 3.1s  ||||||||||||||||              ││
│ P95 Latency:     │ 7.8s  ||||||||||||||||||||||||||    ││
│ Avg SoundScore:  │ 58    ||||||||||||||||||||          ││
│ Repair Rate:     │ 12%   |||||||                       ││
│ Retry Rate:      │ 4%    ||                            ││
│ Cost/gen:        │ $0.07 |||||||                       ││
└─────────────────────────────────────────────────────────┘

┌─ Failure Breakdown ─────────────────────────────────────┐
│                                                     │  %  │
│ Silent output            ████                       │ 2.1 │
│ Clipping                 ██                          │ 1.2 │
│ Too long                 █                           │ 0.8 │
│ Too short                █                           │ 0.5 │
│ Corrupted output         █                           │ 0.4 │
│ Network timeout          █                           │ 0.3 │
│ API error                █                           │ 0.2 │
└──────────────────────────────────────────────────────────┘
```

---

## 10. Reliability Guarantees

### What Users Can Expect

```
cShot Reliability Promise (Beta):

  1. Every sound you hear has passed automated quality checks.
     If it's silent, clipped, or corrupt, you'll never hear it.

  2. Generation succeeds >98% of the time on the first attempt.
     If it fails, cShot silently retries with a different model.

  3. Most sounds generate in <5 seconds.
     If it takes longer, cShot shows you progress and switches
     to a faster model automatically.

  4. Every sound goes through the repair chain (trim, normalize,
     EQ, transient shaping) before you hear it.

  5. If a generation truly fails after all retries, cShot tells
     you exactly what happened and what to do next.
```

### Metrics Targets

| Metric | Alpha | Beta Target | Stretch |
|--------|-------|-------------|---------|
| Generation success rate | 92.8% | >98% | >99% |
| P50 latency | 3.1s | <3s | <2s |
| P95 latency | 14.7s | <8s | <5s |
| User satisfaction | 3.5★ | >4.0★ | >4.5★ |
| SoundScore mean | N/A | >55 | >65 |
| Repair rate | 0% (none) | <15% | <10% |
| Retry rate | 0% (none) | <5% | <3% |
| User-visible errors | 7.2% | <2% | <1% |
| Crash rate | Unknown | <0.5% | <0.1% |

---

## 11. Reliability Architecture Diagram

```
                    ┌──────────────────────┐
                    │    User Types Prompt  │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │  Pre-Gen Validation  │  ← Catches bad input early
                    └──────────┬───────────┘
                               │
                               ▼
              ┌───────────────────────────────────┐
              │         Model Gateway              │
              │  ┌─────────────────────────────┐  │
              │  │  Provider Selector           │  │  ← Routes by sound type
              │  │  - Health check             │  │  ← Picks healthy provider
              │  │  - Cost check               │  │  ← Budget-aware
              │  │  - Route decision           │  │
              │  └──────────┬──────────────────┘  │
              │             │                      │
              │  ┌──────────▼──────────────────┐  │
              │  │  Retry Handler              │  │  ← Up to 3 attempts
              │  │  - Exponential backoff      │  │
              │  │  - Fallback chain           │  │
              │  │  - Timeout management       │  │
              │  └──────────┬──────────────────┘  │
              └─────────────┼─────────────────────┘
                            │
                            ▼
              ┌───────────────────────────────────┐
              │      Raw Model Output              │
              │      (f32 audio buffer)            │
              └──────────────┬────────────────────┘
                             │
                             ▼
              ┌───────────────────────────────────┐
              │      Post-Gen Validation          │  ← Silent? Clipped? Corrupt?
              └──────────────┬────────────────────┘
                             │
                    ┌────────┴────────┐
                    ▼                 ▼
              ┌────────────┐   ┌──────────────┐
              │  Valid     │   │  Invalid     │
              │  Continue  │   │  Apply       │
              └──────┬─────┘   │  Repair      │
                     │         │  Chain       │
                     │         └──────┬───────┘
                     │                │
                     │         ┌──────┴───────┐
                     │         │  Re-validate  │
                     │         └──────┬───────┘
                     │          ┌─────┴─────┐
                     │          ▼           ▼
                     │     ┌────────┐  ┌──────────┐
                     │     │ Pass   │  │ Fail     │
                     │     │ Done   │  │ Retry    │
                     │     └────────┘  │ or flag  │
                     │                 └──────────┘
                     ▼
              ┌───────────────────────────────────┐
              │      Duplicate Check              │  ← Compare with recent gens
              └──────────────┬────────────────────┘
                             │
                    ┌────────┴────────┐
                    ▼                 ▼
              ┌────────────┐   ┌──────────────┐
              │  Unique    │   │  Duplicate   │
              │  Continue  │   │  Discard     │
              └──────┬─────┘   │  or warn     │
                     │         └──────────────┘
                     │
                     ▼
              ┌───────────────────────────────────┐
              │      Duplicate Check              │  ← Compare with recent gens
              └──────────────┬────────────────────┘
                     │
                     ▼
              ┌───────────────────────────────────┐
              │      Final Quality Score          │  ← SoundScore + validation
              └──────────────┬────────────────────┘
                             │
                             ▼
              ┌───────────────────────────────────┐
              │      Deliver to User              │
              │      - Waveform display           │
              │      - SoundScore badge           │
              │      - Repair notes (if any)      │
              │      - Ready to preview/export    │
              └───────────────────────────────────┘
```

---

## 12. Summary: The Reliability Pyramid

```
                    ┌─────────┐
                    │  User   │
                    │  Trust  │
                    ├─────────┤
                    │Repair   │  ← Fix minor issues (muddy, weak, noisy)
                    │Chain    │
                   ├──────────┤
                   │Post-Gen  │  ← Catch issues before user hears them
                   │Validate  │
                  ├───────────┤
                  │Retry &    │  ← Handle API failures transparently
                  │Fallback   │
                 ├────────────┤
                 │Timeout     │  ← Never let a generation hang
                 │Recovery    │
                ├─────────────┤
                │Duplicate    │  ← Every generation is unique
                │Detection    │
               ├──────────────┤
               │Health       │  ← Monitor providers, pre-empt failures
               │Monitoring   │
              ├───────────────┤
              │Pre-Gen       │  ← Catch bad input before API call
              │Validation    │
             ├────────────────┤
             │Quality        │  ← SoundScore gates every generation
             │Gate           │
            └────────────────┘
```

Every layer catches failures that the layer above misses. The user never sees a failure because a lower layer catches and fixes it first. If a failure reaches the user, it means all 8 layers failed — that should be <1% of generations.
