# Prompt 58 — Design Sound Morphing

A sound morphing system for cShot that lets users blend between two one-shots, interpolate timbres, and generate intermediate variants.

---

## 1. Morphing Concept

```
Sound A (Source)                    Sound B (Target)
  ┌──────────┐                       ┌──────────┐
  │ kick     │                       │ snare    │
  │ punchy   │                       │ crack    │
  │ 140bpm   │                       │ dark     │
  │ sub-heavy│                       │ tight    │
  └────┬─────┘                       └────┬─────┘
       │                                  │
       └──────────┬──────────┬────────────┘
                  │          │
                  ▼          ▼
            ┌──────────┐ ┌──────────┐
            │ Morph 25% │ │ Morph 50% │
            │ more kick │ │ balanced  │
            │ less snap │ │ hybrid    │
            └──────────┘ └──────────┘
                            
            ┌──────────┐ ┌──────────┐
            │ Morph 75% │ │ Extreme  │
            │ more snare│ │ morph     │
            │ less sub  │ │ new sound │
            └──────────┘ └──────────┘
```

---

## 2. Prototype Approach (Immediately Buildable)

### 2.1 Crossfade + EQ Blend

Simplest implementation using DSP-only techniques.

```rust
pub fn crossfade_morph(
    audio_a: &[f32],
    audio_b: &[f32],
    sample_rate: u32,
    morph_amount: f32, // 0.0 = A, 1.0 = B
) -> Vec<f32> {
    // 1. Align both sounds to same length (pad or trim)
    let max_len = std::cmp::max(audio_a.len(), audio_b.len());
    let mut aligned_a = audio_a.to_vec();
    let mut aligned_b = audio_b.to_vec();
    aligned_a.resize(max_len, 0.0);
    aligned_b.resize(max_len, 0.0);
    
    // 2. Align onsets (detect transient in both, shift to match)
    let onset_a = detect_onset(&aligned_a, sample_rate);
    let onset_b = detect_onset(&aligned_b, sample_rate);
    let shift = onset_a as isize - onset_b as isize;
    if shift > 0 {
        aligned_b.rotate_right(shift as usize);
    } else {
        aligned_a.rotate_right((-shift) as usize);
    }
    
    // 3. Apply EQ morph (spectral crossfade)
    // Low frequencies interpolate slowly, highs interpolate fast
    // This sounds more natural than flat crossfade
    
    // 4. Simple amplitude crossfade
    let gain_a = 1.0 - morph_amount;
    let gain_b = morph_amount;
    
    aligned_a.iter()
        .zip(aligned_b.iter())
        .map(|(a, b)| a * gain_a + b * gain_b)
        .collect()
}
```

### 2.2 Spectral Cross-Synthesis

```rust
pub fn spectral_morph(
    audio_a: &[f32],
    audio_b: &[f32],
    sample_rate: u32,
    morph_amount: f32, // 0.0 = A, 1.0 = B
) -> Vec<f32> {
    // 1. STFT both signals
    let window_size = 2048;
    let hop_size = 512;
    
    let stft_a = stft(audio_a, window_size, hop_size);
    let stft_b = stft(b audio_b, window_size, hop_size);
    
    // 2. Align frames (match number of frames)
    let n_frames = std::cmp::min(stft_a.len(), stft_b.len());
    
    // 3. For each frame, interpolate magnitude and phase
    let mut morphed = Vec::with_capacity(n_frames);
    
    for frame in 0..n_frames {
        let mag_a = stft_a[frame].magnitude();
        let mag_b = stft_b[frame].magnitude();
        let phase_a = stft_a[frame].phase();
        let phase_b = stft_b[frame].phase();
        
        // Interpolate magnitude (linear)
        let mag_morph = mag_a * (1.0 - morph_amount) + mag_b * morph_amount;
        
        // Interpolate phase (with unwrapping)
        let phase_diff = phase_b - phase_a;
        let phase_unwrapped = phase_diff - (2.0 * std::f32::consts::PI) *
            (phase_diff / (2.0 * std::f32::consts::PI)).round();
        let phase_morph = phase_a + phase_unwrapped * morph_amount;
        
        morphed.push(Complex::from_polar(mag_morph, phase_morph));
    }
    
    // 4. ISTFT back to time domain
    istft(&morphed, window_size, hop_size, audio_a.len())
}
```

### 2.3 Envelope Alignment

```rust
pub struct EnvelopeAligner {
    pub attack_ratio: f32,  // 0.0 = A's attack, 1.0 = B's attack
    pub sustain_ratio: f32,
    pub release_ratio: f32,
}

impl EnvelopeAligner {
    pub fn align_and_morph(
        &self,
        audio_a: &[f32],
        audio_b: &[f32],
        sample_rate: u32,
    ) -> Vec<f32> {
        // 1. Extract amplitude envelopes
        let env_a = extract_envelope(audio_a, sample_rate);
        let env_b = extract_envelope(audio_b, sample_rate);
        
        // 2. Segment each envelope into attack/sustain/release
        let seg_a = segment_envelope(&env_a, sample_rate);
        let seg_b = segment_envelope(&env_b, sample_rate);
        
        // 3. Time-warp each segment to a common length
        let attack_len = lerp(seg_a.attack_len, seg_b.attack_len, self.attack_ratio);
        let sustain_len = lerp(seg_a.sustain_len, seg_b.sustain_len, self.sustain_ratio);
        let release_len = lerp(seg_a.release_len, seg_b.release_len, self.release_ratio);
        
        // 4. Resample each segment to target length
        let morphed_env = Envelope {
            attack: resample(&seg_a.attack_samples, attack_len),
            sustain: resample(&seg_a.sustain_samples, sustain_len),
            release: resample(&seg_a.release_samples, release_len),
        };
        
        // 5. Apply combined envelope to spectral morphed signal
        let spectral = spectral_morph(audio_a, audio_b, sample_rate, 0.5);
        apply_envelope(&spectral, &morphed_env);
        
        spectral
    }
}

fn segment_envelope(env: &[f32], sample_rate: u32) -> EnvelopeSegments {
    // Find attack end (first time envelope goes from risi
    // Find attack end (first time envelope goes from rising to flat/falling)
    // Find sustain end (where envelope drops below 50% of peak)
    // Rest is release
    
    let peak_idx = env.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);
    
    // Attack: 0 to peak_idx (or first significant plateau)
    let attack_end = env[..peak_idx].windows(3)
        .position(|w| w[1] < w[0] && w[2] < w[1])
        .unwrap_or(peak_idx);
    
    // Release: where envelope crosses 50% of peak on the way down
    let peak_val = env[peak_idx];
    let release_start = env[peak_idx..].iter()
        .position(|&s| s < peak_val * 0.5)
        .map(|i| i + peak_idx)
        .unwrap_or(env.len() - 1);
    
    EnvelopeSegments {
        attack_len: attack_end,
        attack_samples: env[..attack_end].to_vec(),
        sustain_len: release_start - attack_end,
        sustain_samples: env[attack_end..release_start].to_vec(),
        release_len: env.len() - release_start,
        release_samples: env[release_start..].to_vec(),
    }
}
```

---

## 3. Research-Grade Approach (Future)

### 3.1 Latent Interpolation

```rust
/// Interpolate in the model's latent space for the most natural morphs.
/// Requires access to the model's encoder/decoder.
pub struct LatentMorpher {
    encoder: ModelEncoder,
    decoder: ModelDecoder,
    interpolator: LatentInterpolator,
}

impl LatentMorpher {
    pub fn morph(
        &self,
        audio_a: &[f32],
        audio_b: &[f32],
        morph_amount: f32,     // 0.0 = A, 1.0 = B
        num_intermediates: usize, // How many intermediate steps
        interpolation: InterpolationType,
    ) -> Vec<Vec<f32>> {
        // 1. Encode both sounds to latent vectors
        let latent_a = self.encoder.encode(audio_a);
        let latent_b = self.encoder.encode(audio_b);
        
        // 2. Interpolate in latent space
        let latents = match interpolation {
            InterpolationType::Linear => {
                // Simple linear: z = (1-t)*z_a + t*z_b
                (0..=num_intermediates).map(|i| {
                    let t = i as f32 / num_intermediates as f32;
                    latent_a * (1.0 - t) + latent_b * t
                }).collect()
            },
            InterpolationType::Spherical => {
                // Spherical linear interpolation (slerp)
                // More natural for high-dimensional spaces
                // Prevents "hole" artifacts where linear path leaves the manifold
                slerp(&latent_a, &latent_b, num_intermediates)
            },
            InterpolationType::Disentangled => {
                // Interpolate different latent dimensions at different rates
                // E.g., timbre changes fast, pitch changes slow
                // Requires disentangled latent space
                disentangled_interp(&latent_a, &latent_b, num_intermediates)
            },
        };
        
        // 3. Decode each latent back to audio
        latents.iter()
            .map(|z| self.decoder.decode(z))
            .collect()
    }
}
```

### 3.2 Spectral Morphing (Advanced)

```rust
/// Phase-coherent spectral morphing with transient preservation.
pub struct AdvancedSpectralMorpher {
    pub fft_size: usize,          // 2048
    pub hop_size: usize,          // 256 (high overlap for quality)
    pub transient_window: usize,  // 128 (separate window for transients)
}

impl AdvancedSpectralMorpher {
    pub fn morph(
        &self,
        audio_a: &[f32],
        audio_b: &[f32],
        morph_amount: f32,
    ) -> Vec<f32> {
        // 1. Separate transient and tonal components
        let (transient_a, tonal_a) = self.separate_transient_tonal(audio_a);
        let (transient_b, tonal_b) = self.separate_transient_tonal(audio_b);
        
        // 2. Morph tonal components using STFT interpolation
        let tonal_morph = self.morph_tonal(&tonal_a, &tonal_b, morph_amount);
        
        // 3. Crossfade transients (transients don't interpolate well spectrally)
        let transient_morph = self.morph_transients(&transient_a, &transient_b, morph_amount);
        
        // 4. Recombine
        let mut output = tonal_morph;
        for i in 0..output.len() {
            output[i] += transient_morph[i];
        }
        
        output
    }
    
    fn separate_transient_tonal(&self, audio: &[f32]) -> (Vec<f32>, Vec<f32>) {
        // Median filtering in spectrogram domain
        // Transients = horizontal median (short windows)
        // Tonal = vertical median (long windows across frequency)
        // Classic "HPSS" (Harmonic-Percussive Sound Separation)
        
        let stft = stft(audio, self.fft_size, self.hop_size);
        
        let n_bins = stft[0].len();
        let n_frames = stft.len();
        
        // Horizontal median filter (percussive = transients)
        let percussive: Vec<Vec<f32>> = (0..n_frames).map(|f| {
            let mut mags: Vec<f32> = stft.iter().map(|frame| frame[f].magnitude()).collect();
            // ... window around frame f, median per bin
            mags
        }).collect();
        
        // Vertical median filter (harmonic = tonal)
        let harmonic: Vec<Vec<f32>> = (0..n_frames).map(|f| {
            // median across neighboring bins within same frame
            stft[f].iter().map(|bin| {
                // ... median over neighboring bins
                bin.magnitude()
            }).collect()
        }).collect();
        
        // ISTFT back
        let transient = istft(&percussive, self.fft_size, self.hop_size, audio.len());
        let tonal = istft(&harmonic, self.fft_size, self.hop_size, audio.len());
        
        (transient, tonal)
    }
    
    fn morph_transients(&self, a: &[f32], b: &[f32], amount: f32) -> Vec<f32> {
        // Transients crossfade with onset alignment
        let onset_a = detect_onset(a, 44100);
        let onset_b = detect_onset(b, 44100);
        
        // Extract transient portion (first ~20ms after onset)
        let transient_len = (44100.0 * 0.02) as usize;
        let trans_a: Vec<f32> = a[onset_a..std::cmp::min(onset_a + transient_len, a.len())].to_vec();
        let trans_b: Vec<f32> = b[onset_b..std::cmp::min(onset_b + transient_len, b.len())].to_vec();
        
        // Crossfade with envelope alignment
        morph_envelopes(&trans_a, &trans_b, amount)
    }
}
```

### 3.3 Neural Codec Interpolation

```rust
/// Using a neural audio codec (EnCodec, DAC, etc.) for latent morphing.
/// The codec's compressed latent space is inherently smooth and invertible.
pub struct NeuralCodecMorpher {
    codec_encoder: NeuralCodec,
    codec_decoder: NeuralCodec,
}

impl NeuralCodecMorpher {
    pub fn morph(
        &self,
        audio_a: &[f32],
        audio_b: &[f32],
        morph_amount: f32,
        sample_rate: u32,
    ) -> Vec<f32> {
        // 1. Encode both to codec latent frames
        let codes_a = self.codec_encoder.encode(audio_a, sample_rate);
        let codes_b = self.codec_encoder.encode(audio_b, sample_rate);
        
        // 2. Align frame counts (pad shorter one)
        let max_frames = std::cmp::max(codes_a.len(), codes_b.len());
        let mut codes_a_aligned = codes_a;
        let mut codes_b_aligned = codes_b;
        
        // Pad with zeros (silence in codec space)
        codes_a_aligned.resize(max_frames, vec![0.0; codes_a[0].len()]);
        codes_b_aligned.resize(max_frames, vec![0.0; codes_b[0].len()]);
        
        // 3. Interpolate each frame's codebook entries
        let morphed_codes: Vec<Vec<f32>> = codes_a_aligned.iter()
            .zip(codes_b_aligned.iter())
            .map(|(frame_a, frame_b)| {
                frame_a.iter()
                    .zip(frame_b.iter())
                    .map(|(a, b)| a * (1.0 - morph_amount) + b * morph_amount)
                    .collect()
            })
            .collect();
        
        // 4. Decode back to audio
        self.codec_decoder.decode(&morphed_codes, sample_rate)
    }
}
```

---

## 4. UX Model

### Main Morph Interface

```
┌──────────────────────────────────────────────────────────┐
│  SOUND MORPHER                                           │
│                                                          │
│  Sound A                    Sound B                      │
│  ┌──────────────────┐      ┌──────────────────┐         │
│  │ punchy_kick.wav  │      │ crack_snare.wav  │         │
│  │ ━━━━━━━━━━━━━━   │      │ ━━━━━━━━━━━━━    │         │
│  │ Kick · 0.42s     │      │ Snare · 0.31s    │         │
│  │ ★ Favorited      │      │ ★ Favorited      │         │
│  └──────────────────┘      └──────────────────┘         │
│                                                          │
│  [Swap A↔B]                                              │
│                                                          │
│  Morph Amount:                                            │
│  More Kick ◄─────●─────────────► More Snare              │
│                50%                                        │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │ Morphed Sound (50% kick, 50% snare)               │   │
│  │ ━━━━━━━━━━━━━━━━━━━━━━                           │   │
│  │ ▶ Preview · Save · Regenerate                    │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  Intermediate Steps:                                      │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐                     │
│  │ 0% │ │ 25%│ │ 50%│ │ 75%│ │100%│                     │
│  │▶   │ │▶   │ │▶   │ │▶   │ │▶   │                     │
│  └────┘ └────┘ └────┘ └────┘ └────┘                     │
│                                                          │
│  [Export All Morphs]  [Export as Pack]  [Clear]         │
└──────────────────────────────────────────────────────────┘
```

### Morph Control Detail

```
Advanced Morph Controls:

  ┌─ TIMBRE ──────────────────────────────────────┐
  │ Follow Sound A ◄─────●─────────────► Sound B  │
  │ (spectral character)                          │
  └───────────────────────────────────────────────┘
  
  ┌─ ENVELOPE ────────────────────────────────────┐
  │ Attack:   ◄──●─────────────►  (A's attack → B's)│
  │ Sustain:  ◄─────●───────────►                   │
  │ Release:  ◄────────●───────►                   │
  └───────────────────────────────────────────────┘
  
  ┌─ TRANSIENT ───────────────────────────────────┐
  │ Preserve: [✓] Keep both transients            │
  │ Blend:    ◄──────●─────────►                  │
  │            A's transient     B's transient    │
  └───────────────────────────────────────────────┘
  
  ┌─ EMOTION ───────────────────────────────────┐
  │ Punch: +2                                   │
  │ Warmth: +1                                  │
  │ (morph blends emotions, then these nudge)   │
  └──────────────────────────────────────────────┘
```

### Crossbreeding (Multi-Parent)

```
  Advanced: Crossbreed Textures

  Sound A (kick, punchy)
  Sound B (snare, crack)

  Blend in additional texture:
    ┌──────────────────────────────────────┐
    │ Texture layer: [white noise ▼]        │
    │ Amount: ◄──────●──────────────────►   │
    │         none              prominent   │
    └──────────────────────────────────────┘

  Or add a third sound as "character" layer:
    Sound C (shaker, organic) at 20%
    → Result: punchy kick-snare hybrid with organic texture
```

---

## 5. Quality Risks & Mitigations

| Risk | Cause | Mitigation |
|------|-------|------------|
| Phase cancellation | Simple crossfade causes destructive interference | Use spectral morphing instead, or randomize phase alignment |
| Transient smearing | Spectral interpolation blurs the attack | HPSS separation: morph transients separately with time-domain crossfade |
| Metallic artifacts | STFT phase interpolation errors | Use phase gradient integration, or magnitude-only morph + Griffin-Lim |
| Latent space "holes" | Linear interpolation leaves the data manifold | Use spherical interpolation (slerp) or Riemannian interpolation |
| Duration mismatch | Two sounds of different lengths | Envelope-based time warping before morphing |
| Loudness jumps | Different perceived levels | Pre-normalize both sounds to same LUFS before morphing |
| Boring intermediates | All morphs sound like 50/50 averages | Use disentangled interpolation (timbre morphs at different rate than dynamics) |
| Artifacts at extremes | 0% and 100% should sound identical to originals | At 0%: output = audio_a (exact copy). At 100%: output = audio_b (exact copy). Verify. |

```rust
pub fn validate_morph(
    original_a: &[f32],
    original_b: &[f32],
    morph_0: &[f32],
    morph_100: &[f32],
) -> MorphValidation {
    // Verify endpoints match originals
    let match_a = correlation(original_a, morph_0) > 0.999;
    let match_b = correlation(original_b, morph_100) > 0.999;
    
    // Verify no clipping
    let peak = morph_0.iter()
        .chain(morph_100.iter())
        .map(|s| s.abs())
        .fold(0.0_f32, f32::max);
    let no_clipping = peak <= 1.0;
    
    // Verify no silence
    let rms_0 = compute_rms(morph_0);
    let rms_100 = compute_rms(morph_100);
    let has_audio = rms_0 > 0.001 && rms_100 > 0.001;
    
    MorphValidation {
        endpoints_match: match_a && match_b,
        no_clipping,
        has_audio,
        passed: match_a && match_b && no_clipping && has_audio,
    }
}
```

---

## 6. Implementation Path

```
Phase 1 — Crossfade Morph (1-2 days)
  - Onset alignment
  - Equal duration
  - Simple crossfade
  - EQ blend (low frequencies crossfade slow, highs fast)
  - "Good enough" for alpha, simple to understand

Phase 2 — Spectral Morph (1 week)
  - STFT-based magnitude/phase interpolation
  - HPSS transient separation
  - Envelope alignment
  - Phase gradient integration for clean phase
  - Sounds significantly better, few artifacts

Phase 3 — Model-Based Morph (2-4 weeks, requires model access)
  - Latent encoding + interpolation
  - Neural codec morphing
  - Disentangled control (morph timbre independently of dynamics)
  - Highest quality, most natural results
  - Requires encoder/decoder model architecture

Phase 4 — Crossbreeding (1-2 weeks after Phase 3)
  - Multi-parent morphing
  - Texture layering
  - Emotional interpolation
  - Intermediate variant generation in batch
```

---

## 7. Summary

Sound morphing lets users blend two one-shots into hybrids. The prototype approach (crossfade + spectral morph) is immediately buildable with DSP. The research-grade approach requires model-level access for latent interpolation and neural codec morphing. HPSS separation preserves transients through the morph. The UX lets users control timbre, envelope, and transient blending independently. Crossbreeding enables multi-sound textures. Quality risks include phase cancellation, transient smearing, and latent space holes — each with a clear mitigation.
