# Prompt 11 — Model Human Perception of Punch

Research-grade psychoacoustic framework for one-shot perception in cShot.

---

## 1. Perceptual Feature Space for One-Shots

Rather than describing one-shots via waveform similarity (RMSE, cross-correlation), cShot will describe them via a **perceptual feature vector** — dimensions that map directly to human auditory experience.

### Primary Perceptual Axes

| Axis | Description | Psychoacoustic Basis | Measurement |
|------|-------------|---------------------|-------------|
| **Punch** | Perceived impact/force of transient onset | Temporal masking curve slope, peak envelope rise time | Transient peak-to-RMS ratio within first 5ms, attack time (10-90%) |
| **Weight** | Perceived low-frequency power and body | Spectral centroid below 250Hz, Fletcher-Munson low-frequency sensitivity | LF energy ratio (20-250Hz / total), spectral slope below 500Hz |
| **Warmth** | Perceived richness in low-mid harmonics | Spectral rolloff, harmonic density in 200-2000Hz range | Spectral centroid deviation from flat, odd/even harmonic ratio |
| **Brightness** | Perceived high-frequency presence and air | Spectral centroid, high-frequency energy above 5kHz | Spectral centroid (Hz), HF energy ratio (>5kHz / total) |
| **Harshness** | Perceived unpleasant high-frequency energy | Spectral flatness in 2-8kHz, roughness (amplitude modulation in critical bands) | Peak-to-average ratio in 2-8kHz, dissonance measure (sensory roughness) |
| **Depth** | Perceived front-to-back spatial placement | Early-to-late reverberation ratio, direct-to-reverberant energy | Dry/wet ratio, envelope decay slope |
| **Width** | Perceived stereo spaciousness | Inter-aural cross-correlation (IACC), stereo pan spread | Mid-side energy ratio, phase coherence across channels |
| **Texture** | Perceived surface detail/grain of sound | Noise-to-tonal ratio, spectral irregularity, sub-band temporal variation | Spectral crest factor per ERB band, noise floor modulation |
| **Gloss** | Perceived polish/smoothness of tone | Spectral envelope smoothness, high-frequency rolloff consistency | Spectral envelope kurtosis, MFCC variance |
| **Air** | Perceived high-frequency openness | Energy above 10kHz, noise floor above 12kHz | Ultrasonic energy ratio (>12kHz), HF noise correlation |
| **Presence** | Perceived forwardness/immediacy | Midrange (1-4kHz) energy relative to lows and highs | Presence band energy ratio (1-4kHz / total), onset density |
| **Realism** | Perceived naturalness / lack of artifact | Spectral deviation from natural instrument templates, phase coherence | Distance from manifold of natural sounds, phase linearity |

---

## 2. Measurable Psychoacoustic Metrics

### 2.1 Transient Metrics (Punch Axis)

```
Attack Sharpness = max(d/dt(amplitude_envelope)) in first 20ms
Transient Peak Ratio = peak_amplitude / RMS(first 50ms)
Impact Density = integral of squared envelope derivative (0-10ms)
Transient Spectral Spread = spectral centroid at attack peak vs sustain
```

### 2.2 Temporal Metrics

```
Forward Masking Slope = rate of threshold recovery after transient offset
Decay Character = inflection points in log-amplitude decay curve
Release Flatness = spectral centroid drift during release phase
Onset Asynchrony = timing spread across frequency bands at attack
```

### 2.3 Spectral Metrics

```
Spectral Centroid = sum(f_k * A_k) / sum(A_k)  (weighted mean frequency)
Spectral Rolloff = frequency below which X% of energy lies (85%, 95%)
Spectral Flatness = geometric_mean(A_k) / arithmetic_mean(A_k) (per critical band)
Spectral Crest = peak / RMS within each critical band
Spectral Irregularity = sum(|A_k - (A_{k-1} + A_k + A_{k+1})/3|)

Brightness Index = energy_ratio(5kHz-20kHz, 20Hz-20kHz)
Warmth Index = energy_ratio(200Hz-2kHz, total)
Body Index = energy_ratio(20Hz-250Hz, total)
Presence Index = energy_ratio(1kHz-4kHz, total)
```

### 2.4 Masking Metrics

```
Simultaneous Masking Threshold = excitation pattern across ERB scale
Temporal Masking Curve = forward masking decay (dB/ms) post-transient
Masking Margin = difference between signal level and masking threshold
Perceptual Entropy = bits required to encode above masking threshold
```

### 2.5 Loudness Metrics (ISO 532)

```
Specific Loudness = loudness per critical band (sones/Bark)
Instantaneous Loudness = time-varying total loudness
Short-term Loudness = integrated over ~100ms windows
Loudness Dynamics = variance of short-term loudness over sound duration
Peak-to-Loudness Ratio = true peak / integrated loudness
```

### 2.6 Spatial Metrics

```
IACC (Inter-Aural Cross-Correlation) = measure of spatial diffuseness
Apparent Source Width = derived from IACC and frequency
Listener Envelopment = late-field energy / early-field energy
Phase Coherence = inter-channel phase consistency per band
Stereo centroid = perceptual center of stereo image
```

---

## 3. Emotional Sound Descriptor System

cShot will implement a **layered emotional descriptor** system for each one-shot:

### Layer 1: Low-Level Perceptual Primatives
The 12 axes from Section 1, computed directly from audio.

### Layer 2: Mid-Level Sound-Quality Descriptors
```
Aggression = f(punch, harshness, brightness, instability)
Softness = f(-harshness, warmth, slow attack, smooth decay)
Richness = f(harmonic density, warmth, texture complexity)
Thinness = f(-weight, -warmth, narrow bandwidth)
Clarity = f(presence, low masking, transient precision)
Muddiness = f(-clarity, excess low-mid energy, slow decay)
Punchiness = f(attack sharpness, transient ratio, impact density)
Smoothness = f(-spectral irregularity, -harshness, gloss)
```

### Layer 3: High-Level Emotional Labels
These are learned embeddings mapped from Layers 1-2 via a trained projection:

| Emotional Label | Typical Perceptual Signature |
|----------------|------------------------------|
| Dark | Low centroid, low brightness, high weight, slow attack |
| Euphoric | Wide stereo, high brightness, rich harmonics, long release |
| Cinematic | High depth, wide, complex texture, dynamic range |
| Metallic | High spectral centroid, high harmonic content, long ringing decay |
| Nostalgic | Warm, soft attack, moderate compression, slight lo-fi |
| Glossy | Very low spectral irregularity, smooth envelope, high presence |
| Cold | Low warmth, high centroid, minimal reverb, fast decay |
| Organic | Irregular spectral envelope, slight inharmonicity, natural variation |
| Futuristic | Complex inharmonic spectra, evolving texture, wide stereo |
| Dirty | High noise floor, spectral roughness, distortion artifacts, saturation |
| Expensive | Clean transients, full bandwidth, precise stereo, no noise |
| Crunchy | High mid-range distortion, grittiness, clipped transients |
| Intimate | Close micing, minimal reverb, narrow stereo, detailed texture |
| Explosive | Massive transient, long impact, wide frequency spread, high dynamic range |

### Mapping Layer 2 → Layer 3

```
emotional_scores = softmax(W * perceptual_features + b)
```

Where `W` is a learned projection matrix trained on human-labeled datasets (e.g., sound designers rating one-shots along semantic differential scales).

---

## 4. Perceptual Embedding Architecture

### Overview

```
Input Audio → STFT/Mel-Spectrogram → Perceptual Feature Extraction
                                         ↓
                                Perceptual Fingerprint (12-D)
                                         ↓
                                Learned Projection (256-D)
                                         ↓
                            Perceptual Embedding Space
```

### 4.1 Feature Extraction Stage

```
Audio (44.1kHz)
  → STFT (1024 window, 512 hop)
  → Multi-resolution Mel (128 bands, 3 resolutions)
  → Envelope extraction (3 time constants: 1ms, 10ms, 100ms)
  → Instantaneous frequency estimation
```

### 4.2 Perceptual Frontend

```
Mel bands → Perceptual weighting (Fletcher-Munson inverse)
         → ERB scale warping
         → Temporal masking estimation
         → Spectral masking estimation
         → Specific loudness (Zwicker model)
```

### 4.3 Embedding Network Architecture

```python
class PerceptualEmbedding(nn.Module):
    def __init__(self):
        self.conv1 = Conv2D(128, 64, 3, 2)   # spectral compression
        self.conv2 = Conv2D(64, 128, 3, 2)   # temporal compression
        self.conv3 = Conv2D(128, 256, 3, 2)
        self.perceptual_pool = PerceptualWeightedPooling()  # learnable
        self.proj = Linear(256, 256)
        
    def forward(self, audio):
        mel = mel_spectrogram(audio, 128)
        per_weighted = fletcher_munson_weight(mel)
        mask = compute_masking(per_weighted)
        x = per_weighted * mask  # apply psychoacoustic mask
        x = self.conv1(x)
        x = self.conv2(x)
        x = self.conv3(x)
        x = self.perceptual_pool(x)
        # Normalize to hyper-sphere
        emb = F.normalize(self.proj(x), dim=-1)
        return emb
```

### 4.4 Training Objectives

| Loss | Type | Purpose |
|------|------|---------|
| Perceptual Triplet Loss | Metric learning | Same-perception sounds cluster |
| Emotional Alignment Loss | Cross-entropy | Embedding predicts emotional labels |
| Ranking Loss | Ordinal regression | "More punchy than" ordering preserved |
| Contrastive Loss | Self-supervised | Augmented views of same sound close |
| Perceptual Metric Loss | Regression | Match known psychoacoustic metrics |

### 4.5 Perceptual Augmentations

Used for self-supervised contrastive pretraining:
- **Temporal stretch** (time-stretch preserving pitch)
- **Spectral tilt** (EQ shelving)
- **Dynamic range compression**
- **Reverb addition** (varying room size)
- **Noise injection** at various spectral shapes
- **Band-limiting**
- **Transient shaping** (attack/sustain modification)

---

## 5. Visualization & Navigation

cShot will provide a **2D/3D perceptual map** of a user's library:

```
UMAP / t-SNE projection of perceptual embeddings
  - Axes labeled by dominant perceptual dimensions
  - Color-coded by emotional descriptor
  - Size proportional to "punch" or "weight"
  - Click to hear, drag to generate interpolated sounds
```

**Navigation modes:**
- Perceptual similarity search ("find sounds that punch like this")
- Emotional mood board ("make this sound more euphoric")
- Perceptual blending ("interpolate between dark/cinematic and bright/glossy")

---

## 6. Key Design Goals

1. **Perceptual > Waveform**: Two waveforms with low RMSE but different perceived quality should be far apart in this space; two waveforms with high RMSE but identical perceived quality should be close.

2. **Interpretable Axes**: Each dimension of the embedding should correlate (at least roughly) with a named perceptual attribute.

3. **Controllable**: Moving along a perceptual axis in the embedding space should produce a predictable change in the generated audio.

4. **Cognitively Validated**: The space should align with human perceptual similarity judgments (tested via ABX listening tests and multidimensional scaling experiments).

5. **Invariances**: The embedding should be invariant to loudness normalization, sample rate (above minimum), and irrelevant acoustic variations.
