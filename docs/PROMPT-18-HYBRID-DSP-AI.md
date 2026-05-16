# Prompt 18 — Build a Hybrid DSP + AI Architecture

cShot uses deterministic DSP where possible, AI only where necessary.

---

## 1. Design Philosophy

### 1.1 Decomposition Principle

Every one-shot can be decomposed into elements that are **best handled by DSP** vs **best handled by AI**:

| Sound Element | DSP | AI | Rationale |
|--------------|-----|----|-----------|
| Oscillator waveform | ✅ | | Deterministic — just math |
| Filter response | ✅ | | Physical process, exact |
| Envelope shape | ✅ | | Simple curve, predictable |
| FM/AM modulation | ✅ | | Exact frequency relationships |
| Reverb tail | ✅ | | Convolution/physical model |
| Distortion/saturation | ✅ | | Well-understood nonlinearity |
| Transient shape | ✅ (good) | ✅ (better) | Complex, learned patterns |
| Spectral texture | | ✅ | Stochastic, learned |
| Harmonic inharmonicity | ✅ (rough) | ✅ (exact) | Instrument-specific |
| Noise characteristics | ✅ (simple) | ✅ (detailed) | Colored noise vs learned |
| Genre conventions | | ✅ | Cultural, learned |
| Emotional quality | | ✅ | Subjective, learned |
| Production polish | | ✅ | Complex processing chains |
| Style transfer | | ✅ | High-level mapping |
| Creative exploration | | ✅ | Beyond human-coded |

### 1.2 Where to Use AI (and Only There)

AI is used for:
1. **Mapping intent to parameters** — text/semantics → DSP parameters
2. **Filling in what DSP cannot** — residual detail, texture, organic variation
3. **Correcting DSP artifacts** — "neural polish"
4. **Creative generation** — novel sounds without explicit rules
5. **Perceptual optimization** — making sound "feel right" (Prompt 11-12)

DSP is used for everything else.

---

## 2. Signal Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        cShot Hybrid Pipeline                              │
│                                                                          │
│  ┌─────────┐    ┌──────────┐    ┌───────────────┐    ┌──────────────┐  │
│  │ Neural   │───▶│  DSP     │───▶│  Neural        │───▶│  Perceptual  │  │
│  │ Frontend │    │  Engine  │    │  Refinement    │    │  Validation  │  │
│  │ (5M par) │    │ (CPU)    │    │ (10M par)      │    │  (Prompt 11) │  │
│  └─────────┘    └──────────┘    └───────────────┘    └──────────────┘  │
│       │              │                 │                     │          │
│       │              │                 │                     │          │
│       ▼              ▼                 ▼                     ▼          │
│  Text/Semantic   Raw Audio w/     Polished Audio       Quality         │
│  Conditioning    DSP Artifacts                          Feedback        │
└─────────────────────────────────────────────────────────────────────────┘
```

### Stage 1: Neural Frontend

```
Input: Text prompt, semantic embedding, latent code, perceptual targets
Size: ~5M parameters (tiny transformer or MLP)
Output: DSP synthesis parameters (128-dim vector)

Architecture:
- Input: text_embedding (768) + perceptual_targets (12) + genre_embedding (32)
- 2x transformer blocks (d_model=256, nhead=4)
- MLP projection to 128 DSP params
- Parameter constraints (sigmoid/tanh for bounded params)
```

### Stage 2: DSP Engine

```
Input: 128 synthesis parameters
Output: Raw audio buffer (2-5 seconds)

Components (all CPU, real-time, deterministic):
  1. Oscillator Bank (4 parallel oscillators)
     - Osc 1: Wavetable (morphing between 16 wavetables)
     - Osc 2: FM (carrier/modulator, ratio control)
     - Osc 3: Noise (colored, with spectral shaping)
     - Osc 4: Sub (sine only, pitch tracking)
  
  2. Mixer
     - Level, pan per oscillator
     - Bus compression
  
  3. Filter Section
     - Multi-mode: LP, HP, BP, notch, formant
     - Key-track, envelope modulation
     - Resonance control with self-oscillation
  
  4. Envelope Sections
     - Amp envelope (6-stage: delay, attack, hold, decay, sustain, release)
     - Filter envelope (same)
     - Pitch envelope (for transient click design)
     - 2x auxiliary envelopes (for modulation)
  
  5. Modulation Matrix
     - 6 sources × 16 destinations
     - Sources: envelopes, LFOs (×4), velocity, key-track
     - Destinations: osc pitch, filter freq, pan, level, etc.
  
  6. Effects Chain
     - Distortion (5 types: soft-clip, hard-clip, tube, tape, wavefold)
     - EQ (3-band parametric)
     - Reverb (convolution + algorithmic hybrid)
     - Delay (stereo, ping-pong, filtered)
     - Compressor
     - Width enhancer (mid-side processing)
```

### Stage 3: Neural Refinement

```
Input: DSP output audio (44.1kHz mono/stereo)
Size: ~10M parameters
Output: Refined audio (same length, improved quality)

Purpose:
  - Remove DSP artifacts (zipper noise, aliasing, phase discontinuities)
  - Add organic micro-variations (human imperfection)
  - Enhance texture and detail
  - Spectral shaping for naturalness

Architecture:
  - Small convolutional U-Net (4 down, 4 up, 16 channels)
  - Works on mel spectrogram (128 bands)
  - Output: complex spectrogram mask (gain per TF bin)
  - Apply: output = DSP_spectrogram * learned_mask
  - ISTFT back to waveform

Training:
  - Paired: DSP output → real recording
  - Loss: multi-scale spectral + perceptual + adversarial (tiny discriminator)
  - 100K steps on 10K paired examples
```

### Stage 4: Perceptual Validation (from Prompt 11)

```
After refinement, validate:
  - Transient quality (punch metric)
  - Spectral balance (centroid, rolloff)
  - Noise floor (cleanliness)
  - Dynamic range
  - Perceptual embedding distance to target
  - Emotional alignment

If validation fails, return to Stage 1 with corrected parameters.
(Closed-loop: max 3 iterations, typically 0-1)
```

---

## 3. Model Orchestration

### 3.1 Generation Modes

| Mode | Frontend | DSP | Refinement | Latency | Quality |
|------|----------|-----|------------|---------|---------|
| Draft | Random params | ✅ | ❌ | <1ms | 5/10 |
| Quick | Small NN (1M) | ✅ | ❌ | 2ms | 7/10 |
| Standard | Full NN (5M) | ✅ | ✅ | 20ms | 9/10 |
| Premium | Full NN + diffusion | ✅ | ✅ | 200ms | 10/10 |
| Creative | Latent exploration | ✅ | ✅ | 100ms | 9/10 |
| Batch | Full NN | ✅ | ✅ | 5ms/sound | 9/10 |

### 3.2 Adaptive Model Selection

```python
def select_generation_mode(user_request, hardware, latency_target):
    """Automatically select optimal generation mode."""
    
    if hardware.is_mobile or hardware.is_web:
        return 'quick'  # CPU-only, minimal model
    
    if latency_target == 'realtime':
        if hardware.has_gpu:
            return 'standard'
        return 'quick'
    
    if 'high_quality' in user_request or 'premium' in user_request:
        if hardware.has_gpu and hardware.vram_gb >= 8:
            return 'premium'
        return 'standard'
    
    if user_request.get('batch_size', 1) > 10:
        return 'batch'
    
    return 'standard'
```

### 3.3 Cascading Refinement

```python
def generate_with_cascade(prompt, quality_level='standard'):
    """Generate with optional cascading refinement stages."""
    
    # Stage 1: Frontend predicts DSP params
    dsp_params = neural_frontend(prompt)
    
    # Stage 2: DSP renders base audio
    base_audio = dsp_engine(dsp_params)
    
    if quality_level == 'draft':
        return base_audio
    
    # Stage 3: Neural refinement (artifact removal)
    refined_audio = neural_refinement(base_audio)
    
    if quality_level == 'quick':
        return refined_audio
    
    # Stage 4: Perceptual validation and correction
    perceptual_score = perceptual_validator(refined_audio, prompt)
    if perceptual_score < threshold:
        # Adjust and re-render
        dsp_params = correct_params(dsp_params, perceptual_score)
        base_audio = dsp_engine(dsp_params)
        refined_audio = neural_refinement(base_audio)
    
    if quality_level == 'standard':
        return refined_audio
    
    # Stage 5: Diffusion refinement (premium only)
    premium_audio = small_diffusion_refine(refined_audio, steps=10)
    
    return premium_audio
```

---

## 4. Optimization Systems

### 4.1 Inference Cost Reduction

| Technique | Savings | Complexity |
|-----------|---------|------------|
| DSP on CPU (while NN runs on NPU) | 2x parallel | Low |
| INT8 quantization of all NNs | 4x memory, 2x speed | Medium |
| Pruning (50% sparsity) | 2x speed | Medium |
| Distillation (large → small) | 5x speed | High |
| ONNX runtime deployment | 1.5-3x speed | Low |
| Perceptual early exit | Skip refinement for simple sounds | Medium |
| Cached DSP presets | 10x speed for common sounds | Low |
| Batch generation | Parallel DSP rendering | Low |

### 4.2 Target: Real-Time on Consumer Hardware

**Target spec for local inference:**
- CPU: Apple M1 / Intel i7 (any modern laptop)
- RAM: 8GB
- GPU: optional (integrated GPU okay)
- Latency: < 50ms per one-shot

**How:**
- Neural frontend: 5M params, ONNX, INT8 → 5ms on CPU
- DSP engine: C++ SIMD, real-time → 1-5ms
- Neural refinement: 10M params, ONNX, INT8 → 15ms on CPU (25ms on GPU)
- Total: ~25ms (well under 50ms target)

### 4.3 Memory Optimization

```python
# Model sizes (INT8 quantized):
models = {
    'neural_frontend': '5MB',   # 5M params × 1 byte
    'neural_refinement': '10MB', # 10M params × 1 byte
    'perceptual_validator': '2MB',
    'wavetables': '1MB',         # 16 wavetables × 64KB
    'impulse_responses': '5MB',  # 50 IRs for convolution reverb
    'total': '23MB'
}
# Entire generation system fits in 23MB. Runs in <100MB RAM.
```

---

## 5. Training Pipeline

### 5.1 Paired Data Generation

```
1. Collect 10,000 real one-shot recordings (kicks, snares, hats, etc.)
2. For each recording:
   a. Analyze with DSP parameter estimator (separate model)
   b. Extract: oscillator settings, filter settings, envelope settings, FX settings
   c. Store: (dsp_params, real_audio) pair
3. For each pair:
   a. Render DSP with predicted params → dsp_audio
   b. Store: (dsp_audio, real_audio) for refinement training
```

### 5.2 Training Objectives

| Model | Loss | Data |
|-------|------|------|
| Neural Frontend | L1 on DSP params + perceptual loss on output | (text/latent, dsp_params) |
| Neural Refinement | Multi-resolution STFT loss + Mel loss + adversarial | (dsp_audio, real_audio) |
| Perceptual Validator | Human preference ranking loss | (sounds, human scores) |

### 5.3 End-to-End Fine-Tuning

After individual training, fine-tune the full pipeline end-to-end with:
- Differentiable DSP (approximate gradients through DSP operations)
- Perceptual loss (Prompt 11) as main objective
- Joint optimization of frontend + refinement

---

## 6. Comparison: Pure AI vs Hybrid

| Criterion | Pure AI (Diffusion) | Hybrid (DSP+AI) | Advantage |
|-----------|--------------------|-----------------|-----------|
| Model size | 1-5B params | 15M params | Hybrid (100x smaller) |
| Training cost | 1000+ GPU hours | 100 GPU hours | Hybrid (10x cheaper) |
| Inference cost | 10-60s GPU | 5-50ms CPU | Hybrid (1000x faster) |
| Controllability | Text only | Text + every DSP param | Hybrid |
| Editability | Implicit (inpainting) | Explicit (tweak params) | Hybrid |
| Quality | State-of-the-art | Comparable with ref. | Pure AI (slightly) |
| Artifact type | AI artifacts | DSP artifacts (predictable) | Hybrid (fixable) |
| Determinism | Stochastic | Deterministic base | Hybrid |
| Novelty | Can invent new sounds | Extends synth sounds | Pure AI |
| Local hardware | GPU required | CPU possible | Hybrid |

---

## 7. DSP Engine Specification (Prototype)

```
cShotDSP v1.0 — C++ with Python bindings (pybind11)
Sample rate: 44.1kHz
Buffer size: 2048 samples (~46ms)

Oscillators:
  - Wavetable: 16 tables, 2048 samples each, 16-bit
  - FM: 2 operators, ratio 0.25-16, feedback 0-100%
  - Noise: white/pink/brown, with HP/LP filter
  - Sub: sine, pitch tracking from osc1

Filter:
  - State-variable filter (SVF)
  - Modes: LP12, LP24, HP12, HP24, BP, notch, peak, formant
  - Freq: 20-20kHz
  - Q: 0.1-20
  - Envelope modulation depth 0-100%

Envelopes:
  - AHDSR (delay 0-5s, attack 0.1ms-2s, hold 0-5s, decay 0.1ms-5s, sustain 0-100%, release 0.1ms-10s)
  - Exponential or linear segments
  - Velocity sensitivity 0-100%

Effects:
  - Distortion: 5 algorithms with drive 0-100%
  - EQ: 3-band, +/-18dB, Q 0.1-10
  - Reverb: Schroeder-Moorer (algorithmic), 4 parallel comb filters + 3 allpass
  - Delay: stereo, 0-2000ms, feedback 0-99%, LP filter
  - Compressor: threshold -60-0dB, ratio 1-20, attack 0.1-50ms, release 10-1000ms
  - Width: mid-side, width 0-200%

Performance:
  - < 1ms for full chain at 44.1kHz/2048 buffer on M1
  - SIMD optimized (ARM NEON / x86 SSE)
  - No dynamic allocation after init
```
