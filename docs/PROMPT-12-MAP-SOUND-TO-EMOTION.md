# Prompt 12 — Map Sound to Emotion

cShot emotional audio architecture: from acoustic features to perceived feeling.

---

## 1. Foundations

### 1.1 Dimensional Models of Emotion

The dominant framework for mapping sound to emotion in computational systems builds on two-dimensional valence-arousal space (Russell's circumplex model), extended for sound design:

```
                AROUSAL (high)
                    |
         Angry —— Tense —— Excited
            |               |
     NEGATIVE —— NEUTRAL —— POSITIVE  VALENCE
            |               |
        Sad —— Calm —— Peaceful
                    |
                AROUSAL (low)
```

cShot adds a third dimension — **Power/Dominance** — which maps to perceptual axes like weight, punch, and width. This 3D space (Valence, Arousal, Power) forms the core emotional coordinate system.

### 1.2 Semantic Differential Scales

Sound designers describe sounds using bipolar adjective pairs (Osgood's semantic differential). cShot will use these as the primary annotation layer:

| Scale | Low (-1) | High (+1) | Primary Acoustic Correlates |
|-------|----------|-----------|-----------------------------|
| Warm/Cold | Warm | Cold | Spectral centroid, harmonic richness, low-freq energy |
| Hard/Soft | Hard | Soft | Attack time, transient peak, HF content |
| Bright/Dark | Bright | Dark | Spectral centroid, HF energy ratio |
| Rough/Smooth | Rough | Smooth | Spectral flatness, AM depth, noise content |
| Full/Thin | Full | Thin | Bandwidth, LF energy, harmonic density |
| Natural/Artificial | Natural | Artificial | Spectral irregularity, phase coherence, inharmonicity |
| Dry/Wet | Dry | Wet | Reverb ratio, early-to-late energy |
| Clean/Dirty | Clean | Dirty | Noise floor, distortion, spectral roughness |
| Narrow/Wide | Narrow | Wide | Stereo width, IACC, MS ratio |
| Static/Dynamic | Static | Dynamic | Temporal variance, spectral flux |

---

## 2. Audio-to-Emotion Mapping Architecture

### 2.1 Perceptual Front-End

```
Input Audio (mono/stereo 44.1kHz)
    ↓
Multi-resolution STFT (3 windows: 512, 1024, 4096)
    ↓
Mel spectrogram (128 bands, 3 time resolutions)
    ↓
Perceptual weighting (Fletcher-Munson equal loudness contours)
    ↓
Temporal decomposition (attack/sustain/release segmentation)
    ↓
Feature extraction ~ 356 features total
```

### 2.2 Feature Groups

**Group A — Temporal Features** (64)
- Attack time (10-90%), decay time, release time
- Envelope shape moments (skew, kurtosis)
- Transient density, transient regularity
- Amplitude modulation depth & rate
- Onset asynchrony across frequency bands
- RMS contour, crest factor

**Group B — Spectral Features** (128)
- Spectral centroid, spread, skewness, kurtosis
- Spectral rolloff (85%, 95%)
- Spectral flatness per critical band (24 Bark bands)
- Spectral crest factor per band
- Mel-frequency cepstral coefficients (MFCC 1-20)
- Spectral contrast (peaks/valleys per subband)
- Chroma features (12-bin pitch class profile)
- Tonal power ratio, noise power ratio
- Inharmonicity coefficient

**Group C — Perceptual Features** (56)
- Specific loudness (24 Bark bands, Zwicker model)
- Sharpness (Zwicker's sharpness model)
- Roughness (AM depth in critical bands)
- Fluctuation strength (modulations 1-20Hz)
- Tonality (tonal/noise ratio at psychoacoustic level)
- Sensory dissonance (sethares dissonance curve)
- Pitch strength, pitch salience

**Group D — Spatial/Stereo Features** (28)
- Inter-aural level difference (ILD) per band
- Inter-aural time difference (ITD) per band
- Inter-aural cross-correlation (IACC) — early, late, full
- Mid-side energy ratio per band
- Phase difference distribution
- Stereo centroid, stereo width

**Group E — Production Features** (80)
- Dynamic range (crest factor, short-term vs long-term)
- Compression amount (envelope correlation with amplitude)
- Distortion metrics (harmonic distortion, intermodulation)
- Noise floor profile (shape, stationarity)
- Reverb characteristics (decay time per band, early reflections)
- Spectral balance (spectral tilt, smile curve deviation)
- Transient shaping metrics (attack boost/cut)
- Saturation metrics (even/odd harmonic generation)

### 2.3 Emotion Embedding Network

```python
class EmotionEmbedding(nn.Module):
    """Map acoustic features to 3D VAP (valence/arousal/power) + semantic embedding."""
    
    def __init__(self):
        self.feature_encoder = nn.Sequential(
            nn.Linear(356, 512),
            nn.LayerNorm(512),
            nn.GELU(),
            nn.Dropout(0.2),
            nn.Linear(512, 256),
        )
        self.vap_head = nn.Linear(256, 3)  # valence, arousal, power
        self.semantic_head = nn.Linear(256, 64)  # semantic embedding (64-D)
        self.emotion_head = nn.Linear(256, 20)  # multi-label emotion classifier
        
    def forward(self, features):
        x = self.feature_encoder(features)
        vap = torch.tanh(self.vap_head(x))  # [-1, 1]
        semantic = F.normalize(self.semantic_head(x), dim=-1)
        emotions = torch.sigmoid(self.emotion_head(x))  # multi-label
        return vap, semantic, emotions
```

### 2.4 Training Strategy

| Stage | Data | Loss | Purpose |
|-------|------|------|---------|
| 1. Pretrain (unsupervised) | 1M+ unlabeled one-shots | Contrastive (SimCLR-style) | Learn general audio representations |
| 2. VAP alignment | 50k labeled (V,A,P) | MSE on VAP | Learn emotional coordinate space |
| 3. Semantic alignment | 20k labeled (semantic diffs) | Cosine embedding loss | Learn semantic descriptor space |
| 4. Emotion classification | 10k labeled (emotion tags) | Binary cross-entropy | Learn multi-label emotion prediction |
| 5. Human feedback (RLHF) | Pairwise preferences | Bradley-Terry preference loss | Align with human sound designers |

---

## 3. Latent Emotional Spaces

### 3.1 Emotional Coordinate System (VAP)

cShot stores each one-shot at a coordinate in 3D emotional space:

```
One-shot A:  V=+0.7, A=+0.6, P=+0.5   → "Euphoric, powerful"
One-shot B:  V=-0.6, A=+0.8, P=+0.9   → "Aggressive, intense"
One-shot C:  V=+0.3, A=-0.5, P=-0.2   → "Calm, gentle"
```

### 3.2 Semantic Descriptor Space (64-D)

Learned continuous space where:
- `cosine_distance("dark", "cold")` is small
- `cosine_distance("dark", "bright")` is large
- New descriptors can be projected in via text embedding alignment

### 3.3 Emotion Label Space (20-D multi-label)

Emotion labels cShot will support natively:

Aggressive / Anger / Anticipation / Cinematic / Cold / Dark / Dirty / Dreamy / Euphoric / Explosive / Futuristic / Glossy / Happy / Intimate / Joy / Melancholic / Nostalgic / Powerful / Sad / Tense

Training uses multi-label binary cross-entropy — a sound can be both "dark" and "powerful" simultaneously.

---

## 4. Semantic Audio Navigation

### 4.1 Query Types

**Direct retrieval:**
```
"dark booming kick" → encode text → nearest neighbors in embedding space
"similar to sample X but more aggressive" → encode X → offset along Arousal
```

**Mood board retrieval:**
```
[euphoric +0.3, dark -0.5, cinematic +0.8] → decode position in VAP space
    → retrieve sounds at that coordinate
```

**Interpolation navigation:**
```
Sound A (dark, calm) → interpolate latent → Sound B (bright, aggressive)
    → generate intermediate sounds along path
```

### 4.2 Emotion Interpolation

```python
def interpolate_emotional(audio_a, audio_b, steps=10):
    emb_a = emotion_model(audio_a)   # (vap, sem, emo)
    emb_b = emotion_model(audio_b)
    
    results = []
    for alpha in linspace(0, 1, steps):
        vap = lerp(emb_a.vap, emb_b.vap, alpha)
        sem = slerp(emb_a.sem, emb_b.sem, alpha)  # spherical interp
        # Decode VAP+sem back to generation parameters
        params = emotion_to_params(vap, sem)
        generated = sound_generator(params)
        results.append(generated)
    
    return results  # gradual emotional morph
```

### 4.3 Controllable Emotional Generation

Control knobs for the cShot interface:

| Control | Effect on Sound |
|---------|-----------------|
| Valence knob (-1 to +1) | Spectral centroid, major/minor harmonic shift |
| Arousal knob (-1 to +1) | Tempo, transient density, dynamic range |
| Power knob (-1 to +1) | LF extension, transient punch, stereo width |
| Warm/Cold slider | Spectral tilt, harmonic density |
| Clean/Dirty slider | Noise floor, distortion amount |
| Natural/Artificial slider | Inharmonicity, phase coherence, spectral irregularity |
| Emotional text prompt | Free-form: "make it sound like a hopeful sunrise" |

---

## 5. Perception-Driven Generation

The ultimate goal: cShot generates sounds that feel the way the user wants, not just sound the way the training data does.

### 5.1 Generation Control Flow

```
User intent (text/sliders/coordinates)
    ↓
Emotion parameter decoder (learned mapping)
    ↓
Sound generation parameters (latent code + conditioning)
    ↓
Audio generation (diffusion/GAN/hybrid)
    ↓
Perceptual analysis (psychoacoustic feature extractor)
    ↓
Emotion prediction (emotion model)  
    ↓
Compare with user intent → delta
    ↓
Return to generation with corrected parameters
```

### 5.2 Closed-Loop Correction

```python
def generate_with_emotional_feedback(prompt, max_iter=5):
    params = emotion_to_params(prompt)
    for i in range(max_iter):
        audio = generate(params)
        predicted = predict_emotion(audio)
        error = prompt - predicted  # VAP vector difference
        if norm(error) < threshold:
            break
        params = params + lr * error_gradient(params, error)
    return audio
```

---

## 6. Human Validation Protocol

| Test | Method | Metric |
|------|--------|--------|
| VAP accuracy | Human raters on 1,000 sounds | R² between predicted and human VAP |
| Semantic alignment | Rank retrieval hits | Precision@k for semantic queries |
| Interpolation quality | ABX preference | % preference over baseline interpolation |
| Control precision | User adjusts sliders to match target | Mean distance from target in VAP space |
| Emotional realism | Blind listening test | % of sounds correctly labeled by emotion |
