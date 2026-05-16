# Prompt 57 — Invent High-Level Sound Controls

High-level controls for cShot that feel musical, not technical. A beginner should shape advanced sound design without knowing engineering terms.

---

## 1. Control Palette

```
┌──────────────────────────────────────────────────────────┐
│  SOUND SHAPE                                              │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ Punch         │  │ Body         │  │ Gloss         │   │
│  │ ○────●────○  │  │ ○──●────────○│  │ ○─────●─────○ │   │
│  │ soft  hard   │  │ thin  fat   │  │ matte shine  │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ Air           │  │ Weight       │  │ Snap          │   │
│  │ ○────●────○  │  │ ○─●─────────○│  │ ○─────●─────○ │   │
│  │ dry  airy   │  │ light heavy │  │ soft  crack  │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ Warmth        │  │ Width        │  │ Grit          │   │
│  │ ○────●────○  │  │ ○────●────○ │  │ ○───●───────○ │   │
│  │ cold warm   │  │ mono wide   │  │ clean dirty  │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ Darkness      │  │ Chaos        │  │ Cinematic     │   │
│  │ ○─●─────────○│  │ ○───●───────○│  │ ○─────●─────○ │   │
│  │ bright dark │  │ clean wild  │  │ small epic   │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                           │
│  [Reset All]  [Randomize]  [Copy to Prompt]              │
└──────────────────────────────────────────────────────────┘
```

---

## 2. Control Definitions

### 2.1 Punch — "How hard does it hit?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The impact force of the transient. A high-punch sound feels like it physically hits you. A low-punch sound feels soft or distant. |
| **Audio features** | Onset strength, crest factor, attack time, transient-to-sustain ratio |
| **DSP mapping** | Transient shaper gain (+0 to +6dB), attack time (1-10ms), transient envelope width |
| **Model parameters** | Conditioning on "punchy" vs "soft" latent direction, CFG scale for transient preservation |
| **User experience** | Moving right: the attack gets sharper and harder. The sound cuts through more. Moving left: the attack softens, the sound sits further back. |
| **Safe range** | 0-100 (default: 50). At 100: +6dB transient boost, 0.5ms attack. At 0: no boost, 10ms attack. |

```rust
pub fn apply_punch(audio: &mut [f32], sample_rate: u32, value: f32) {
    // value: 0.0 (soft) to 1.0 (maximum punch)
    let boost_db = value * 6.0; // 0 to +6dB
    let attack_ms = 10.0 - value * 9.5; // 10ms to 0.5ms
    let gain = 10_f32.powf(boost_db / 20.0);
    
    // Detect transient onset
    let onset = detect_onset(audio, sample_rate);
    let transient_len = (sample_rate as f32 * attack_ms / 1000.0) as usize;
    
    // Apply shaped gain to transient portion
    for i in onset..std::cmp::min(onset + transient_len, audio.len()) {
        let envelope = 1.0 - (i - onset) as f32 / transient_len as f32;
        audio[i] *= 1.0 + (gain - 1.0) * envelope;
        audio[i] = audio[i].clamp(-1.0, 1.0);
    }
}
```

### 2.2 Body — "How thick is it?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The fullness of the sound. A fat body fills the frequency spectrum. A thin body is lean and focused. |
| **Audio features** | Low-mid energy (200-800Hz), spectral centroid, bandwidth |
| **DSP mapping** | Low-mid shelf gain (-3 to +4dB) at 400Hz, bandwidth expansion via multiband saturation |
| **Model parameters** | Latent direction for "body/thickness", saturation amount |
| **User experience** | Moving right: the sound gets fuller, the low-mids thicken. Moving left: the sound gets leaner, more focused. |
| **Safe range** | 0-100 (default: 50). At 100: +4dB @ 400Hz + gentle saturation. At 0: -3dB @ 400Hz. |

### 2.3 Gloss — "How polished is the surface?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The sheen on the high end. Glossy sounds have a bright, expensive feel. Matte sounds are dull and organic. |
| **Audio features** | High-frequency energy (8-20kHz), spectral centroid, spectral rolloff |
| **DSP mapping** | High shelf gain (-3 to +6dB) at 8kHz, gentle excitation at 10kHz |
| **Model parameters** | High-frequency emphasis in latent space |
| **User experience** | Moving right: the sound gets shinier, more polished. Moving left: gets duller, more vintage. |
| **Safe range** | 0-100 (default: 50). At 100: +6dB @ 8kHz air shelf. At 0: -3dB @ 8kHz + gentle low-pass. |

### 2.4 Air — "How much space around it?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The sense of space and openness. Airy sounds feel like they're in a room. Dry sounds feel close and direct. |
| **Audio features** | High-frequency energy above 10kHz, reverb tail, stereo width (future) |
| **DSP mapping** | High shelf above 10kHz (-2 to +4dB), very short ambient reverb (0 to 200ms) |
| **Model parameters** | Room/reverb conditioning |
| **User experience** | Moving right: the sound opens up, breathes more. Moving left: gets dryer, more in-your-face. |
| **Safe range** | 0-100 (default: 30). At 100: +4dB air + 200ms room reverb. At 0: -2dB air, no reverb. |

### 2.5 Weight — "How much low end?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The gravitational pull of the low frequencies. Heavy sounds shake the room. Light sounds float above. |
| **Audio features** | Sub-low (20-60Hz) and low (60-250Hz) energy, spectral centroid |
| **DSP mapping** | Low shelf gain (-4 to +6dB) at 100Hz, sub-harmonic synthesis at extreme settings |
| **Model parameters** | Bass intensity conditioning |
| **User experience** | Moving right: the low end gets heavier, more physical. Moving left: the low end gets lighter, tighter. |
| **Safe range** | 0-100 (default: 50). At 100: +6dB @ 100Hz, gentle sub boost. At 0: -4dB @ 100Hz + high-pass. |

### 2.6 Snap — "How fast does it start?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The speed of the initial attack. Snappy sounds start instantly. Loose sounds have a slow, gradual attack. |
| **Audio features** | Attack time (0.5-20ms), onset sharpness, transient rise rate |
| **DSP mapping** | Attack envelope (0.5-20ms), transient onset emphasis, pre-transient click (very short <1ms) |
| **Model parameters** | Attack time conditioning |
| **User experience** | Moving right: the initial hit gets faster and more immediate. Moving left: the onset gets slower and more gradual. |
| **Safe range** | 0-100 (default: 50). At 100: 0.5ms attack + click emphasis. At 0: 20ms attack + softened onset. |

```rust
pub fn apply_snap(audio: &mut [f32], sample_rate: u32, value: f32) {
    // value: 0.0 (loose) to 1.0 (maximum snap)
    let target_attack_ms = 20.0 - value * 19.5; // 20ms → 0.5ms
    
    // Re-shape the attack envelope
    let onset = detect_onset(audio, sample_rate);
    let current_attack_samples = measure_attack_samples(audio, onset, sample_rate);
    let target_attack_samples = (sample_rate as f32 * target_attack_ms / 1000.0) as usize;
    
    if target_attack_samples < current_attack_samples {
        // Compress the attack to be faster
        let ratio = current_attack_samples as f32 / target_attack_samples as f32;
        // Time-compress the attack portion
        let compressed = time_compress(&audio[onset..onset + current_attack_samples], ratio);
        // Replace original attack with compressed version
        let end = std::cmp::min(onset + compressed.len(), audio.len());
        audio[onset..end].copy_from_slice(&compressed[..end - onset]);
    }
}
```

### 2.7 Warmth — "How analog does it feel?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The vintage, tube-like quality. Warm sounds have gentle saturation and rounded highs. Cold sounds are clean and digital. |
| **Audio features** | Even-order harmonics (2nd, 4th), spectral rolloff, noise floor character |
| **DSP mapping** | Gentle tape saturation (0.1-3% THD), low-pass shelf at 10kHz (-2 to 0dB), very subtle wow/flutter |
| **Model parameters** | "Analog warmth" latent direction, training on analog gear datasets |
| **User experience** | Moving right: the sound gets warmer, more vintage, more musical. Moving left: gets cleaner, more digital, more sterile. |
| **Safe range** | 0-100 (default: 30). At 100: 3% tape saturation + 10kHz rolloff. At 0: completely clean. |

### 2.8 Width — "How wide is it?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The stereo spread. Mono sounds come from a single point. Wide sounds fill the stereo field. |
| **Audio features** | Inter-channel correlation (future: stereo), mid-side ratio |
| **DSP mapping** | Haas effect (0-20ms delay), stereo widening via mid-side processing, chorus at extreme settings |
| **Model parameters** | Stereo width conditioning |
| **User experience** | Moving right: the sound spreads wider across the speakers. Moving left: narrows to mono. |
| **Safe range** | 0-100 (default: 0 for mono). At 100: wide stereo, Haas + mid-side. At 0: pure mono. |
| **Note** | Width control only appears when stereo generation is available |

### 2.9 Grit — "How dirty is it?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The amount of distortion, noise, and edge. Gritty sounds are aggressive and raw. Clean sounds are pristine. |
| **Audio features** | THD, spectral flatness, noise floor, high-frequency energy |
| **DSP mapping** | Waveshaping distortion (0-20% drive), bit crushing (16 to 4 bit), noise injection at extreme settings |
| **Model parameters** | Saturation/distortion conditioning, "aggressive" latent direction |
| **User experience** | Moving right: the sound gets hairier, more aggressive, more textured. Moving left: gets cleaner, smoother. |
| **Safe range** | 0-100 (default: 10). At 100: 20% drive + 8-bit crush + noise layer. At 0: pristine clean. |

```rust
pub fn apply_grit(audio: &mut [f32], sample_rate: u32, value: f32) {
    // value: 0.0 (clean) to 1.0 (maximum grit)
    
    // 1. Soft clip / waveshaping
    let drive = value * 10.0; // 0-10x gain before clipping
    for sample in audio.iter_mut() {
        let amplified = *sample * drive;
        *sample = amplified.tanh(); // Soft clipping
    }
    
    // 2. Bit crush at high values
    if value > 0.6 {
        let bits = 16.0 - (value - 0.6) / 0.4 * 12.0; // 16 bit → 4 bit
        let quantize = 2_f32.powi(bits as i32);
        for sample in audio.iter_mut() {
            *sample = (*sample * quantize).round() / quantize;
        }
    }
    
    // 3. Normalize back to prevent perceived loudness jumps
    let peak = audio.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
    if peak > 0.0 {
        let gain = 0.95 / peak;
        for sample in audio.iter_mut() {
            *sample *= gain;
        }
    }
}
```

### 2.10 Darkness — "How dark is the tone?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The overall brightness of the sound. Dark sounds have rolled-off highs and emphasis on low-mids. Bright sounds have extended highs and presence. |
| **Audio features** | Spectral centroid (300Hz-8kHz), high-frequency energy ratio |
| **DSP mapping** | Low-pass filter (3kHz-20kHz cutoff), high-shelf cut/boost |
| **Model parameters** | Spectral centroid latent conditioning |
| **User experience** | Moving right: the sound gets darker, warmer, less present. Moving left: gets brighter, more present, more detailed. |
| **Safe range** | 0-100 (default: 30). At 100: 3kHz low-pass. At 0: 20kHz flat (no rolloff). |

### 2.11 Chaos — "How unpredictable is it?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The amount of randomness, instability, and movement. Chaotic sounds evolve in unexpected ways. Stable sounds are predictable and consistent. |
| **Audio features** | Spectral flux, amplitude variance, pitch instability |
| **DSP mapping** | Random filter modulation, granularization at extreme settings, randomized envelope parameters |
| **Model parameters** | Temperature scaling, latent noise injection, diversity sampling |
| **User experience** | Moving right: the sound becomes more alive, unpredictable, evolving. Moving left: becomes more static, stable, consistent. |
| **Safe range** | 0-100 (default: 10). At 100: significant random modulation + granular texture. At 0: completely deterministic. |

### 2.12 Realism — "How organic does it sound?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The naturalness of the sound. Realistic sounds could be from a physical source. Plastic sounds are clearly synthetic. |
| **Audio features** | Spectral irregularity, inharmonicity, noise component, envelope naturalness |
| **DSP mapping** | Adding physical modeling artifacts, natural variation in envelope, subtle timing jitter |
| **Model parameters** | "Real vs synthetic" latent axis, training on organic vs electronic datasets |
| **User experience** | Moving right: sounds more organic, like a real acoustic source. Moving left: sounds more synthetic, electronic, designed. |
| **Safe range** | 0-100 (default: 50). At 100: maximum physical realism cues. At 0: pure synthetic character. |

### 2.13 Plasticity — "How malleable does it feel?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | How much the sound feels like it can be shaped. Plastic sounds are flexible, moldable. Rigid sounds are fixed, brittle. |
| **Audio features** | Spectral flexibility, formant structure, harmonic richness |
| **DSP mapping** | Formant shifting, spectral morphing, dynamic EQ response |
| **Model parameters** | Latent space controllability, disentangled representation |
| **User experience** | Moving right: the sound feels more shapeable, responsive to further processing. Moving left: feels more fixed, resistant to change. |
| **Safe range** | 0-100 (default: 50). Subtle — more of a meta-control about latent representation quality. |

### 2.14 Cinematic Scale — "How epic does it feel?"

| Aspect | Detail |
|--------|--------|
| **Perceptual meaning** | The grandeur and size. Cinematic sounds are larger than life. Small sounds are intimate and close. |
| **Audio features** | Reverb time (0.5-5s), dynamic range, spectral spread, low-end weight |
| **DSP mapping** | Reverb (room → hall → cathedral), delay (slap → long), stereo widening, multi-band compression for "bigness" |
| **Model parameters** | Spatial conditioning, reverb prediction |
| **User experience** | Moving right: the sound gets bigger, more epic, more cinematic. Moving left: gets smaller, more intimate, closer. |
| **Safe range** | 0-100 (default: 20). At 100: cathedral reverb + 50ms pre-delay + 100% wet. At 0: completely dry. |

```rust
pub fn apply_cinematic_scale(audio: &mut [f32], sample_rate: u32, value: f32) {
    if value < 0.05 { return; }
    
    // Convolution reverb with IR blending
    let reverb_time = 0.5 + value * 4.5; // 0.5s to 5s
    let pre_delay_ms = value * 50.0; // 0 to 50ms
    let wet_dry = value; // 0.0 to 1.0
    
    // Use pre-computed IRs at different sizes:
    // 0.0-0.3: Room IR
    // 0.3-0.6: Hall IR
    // 0.6-0.9: Cathedral IR
    // 0.9-1.0: Custom "epic" IR (hall + early reflections)
    
    let ir_index = (value * 3.0).min(2.0) as usize;
    let ir = &REVERB_IRS[ir_index];
    
    // Convolve with IR
    let mut wet = convolve(audio, ir, sample_rate);
    
    // Apply pre-delay
    let delay_samples = (sample_rate as f32 * pre_delay_ms / 1000.0) as usize;
    wet.rotate_right(delay_samples.min(wet.len()));
    
    // Mix dry/wet
    for i in 0..audio.len() {
        audio[i] = audio[i] * (1.0 - wet_dry) + wet[i] * wet_dry;
    }
}
```

---

## 3. Control-to-DSP Mapping Table

| Control | DSP Effect | Parameters | Safe Range | Cost |
|---------|-----------|------------|------------|------|
| Punch | Transient shaper | boost: 0-6dB, attack: 0.5-10ms | +6dB max | <1ms |
| Body | Low-mid shelf + saturation | gain: -3 to +4dB @ 400Hz | +4dB max | <1ms |
| Gloss | High shelf + exciter | gain: -3 to +6dB @ 8kHz | +6dB max | <1ms |
| Air | Air band + short reverb | gain: -2 to +4dB @ 10kHz, reverb: 0-200ms | +4dB | <2ms |
| Weight | Low shelf + sub synth | gain: -4 to +6dB @ 100Hz | +6dB | <1ms |
| Snap | Attack envelope shaping | attack: 0.5-20ms | 0.5ms min | <1ms |
| Warmth | Tape sat + gentle LP | THD: 0.1-3%, rolloff @ 10kHz | 3% THD | <2ms |
| Width | Haas + mid-side | delay: 0-20ms | 20ms max | <1ms |
| Grit | Wavefolder + bitcrush | drive: 0-20x, bits: 16→4 | 20x | <1ms |
| Darkness | Low-pass filter | cutoff: 3k-20kHz | 3kHz min | <1ms |
| Chaos | Random modulation | mod depth: 0-100% | 100% | <3ms |
| Realism | Physical modeling | various | n/a | <5ms |
| Plasticity | Formant morph | formant shift: 0.5-2x | 0.5-2x | <2ms |
| Cinematic | Convolution reverb | IR: room→cathedral, wet: 0-100% | 100% wet | <10ms |

---

## 4. Control-to-Model Mapping

```rust
/// How high-level controls map to model conditioning parameters
pub fn controls_to_model_conditioning(controls: &SoundControls) -> ModelConditioning {
    let mut conditioning = ModelConditioning::default();
    
    // Punch → transient emphasis + crest factor target
    conditioning.transient_emphasis = controls.punch * 2.0; // 0.0-2.0x
    conditioning.target_crest_factor = 6.0 + controls.punch * 10.0; // 6-16
    
    // Body → low-mid energy target
    conditioning.low_mid_bias = (controls.body - 0.5) * 0.3; // -0.15 to +0.15
    
    // Gloss → high-frequency bias
    conditioning.high_freq_bias = (controls.gloss - 0.5) * 0.4; // -0.2 to +0.2
    
    // Air → air band + reverb
    conditioning.air_band_gain = controls.air * 4.0 - 2.0; // -2 to +2 dB
    conditioning.reverb_amount = controls.air * 0.3; // 0-30% reverb
    
    // Weight → sub bias
    conditioning.sub_bias = (controls.weight - 0.5) * 0.5; // -0.25 to +0.25
    
    // Snap → attack time target
    conditioning.target_attack_ms = 20.0 - controls.snap * 19.5; // 20→0.5ms
    
    // Warmth → harmonic distortion target
    conditioning.target_thd = controls.warmth * 0.03; // 0-3%
    
    // Width → stereo spread
    conditioning.stereo_spread = controls.width; // 0.0-1.0
    
    // Grit → noise floor + distortion
    conditioning.noise_floor_target = -80.0 + controls.grit * 40.0; // -80 to -40dB
    conditioning.distortion_amount = controls.grit * 0.2; // 0-20%
    
    // Darkness → spectral centroid target
    let centroid_min = 3000.0;
    let centroid_max = 20000.0;
    conditioning.target_centroid_hz = centroid_max - controls.darkness * (centroid_max - centroid_min);
    // 20kHz → 3kHz
    
    // Chaos → temperature + latent noise
    conditioning.temperature = 0.5 + controls.chaos * 1.5; // 0.5-2.0
    conditioning.latent_noise = controls.chaos * 0.1; // 0-10%
    
    // Realism → synthetic vs organic
    conditioning.realism_bias = (controls.realism - 0.5) * 2.0; // -1.0 to +1.0
    
    conditioning
}
```

---

## 5. User Experience Design

### Control Interaction

```
Touch/Mouse interaction:
  - Drag slider: real-time preview updates
  - Double-click: reset to default (50)
  - Alt+drag: fine control (10x slower)
  - Right-click: "Copy to prompt" → adds "punchy, bright" etc. to text
  - Hover: shows the perceptual description + current value

Control grouping:
  TRANSIENT: Punch, Snap (how it starts)
  TONE: Body, Darkness, Warmth (how it sounds)
  SPACE: Air, Width, Cinematic (where it lives)
  TEXTURE: Gloss, Grit, Chaos, Realism (what it's made of)
  BASS: Weight (how deep it goes)
```

### Presets (Character Cards)

```
Character cards are named combinations of controls:

"Punchy Trap Kick"
  Punch: 85  |  Snap: 90  |  Body: 40
  Weight: 70 |  Gloss: 60 |  Darkness: 40
  Grit: 20   |  Warmth: 30

"Lo-Fi Warm Rim"
  Punch: 30  |  Snap: 40  |  Body: 70
  Weight: 30 |  Gloss: 20 |  Darkness: 70
  Grit: 50   |  Warmth: 85

"Cinematic Impact"
  Punch: 95  |  Snap: 80  |  Body: 80
  Weight: 90 |  Gloss: 70 |  Darkness: 60
  Grit: 40   |  Cinematic: 90

"Soft Ambient Texture"
  Punch: 10  |  Snap: 10  |  Body: 40
  Weight: 20 |  Gloss: 80 |  Air: 90
  Chaos: 70  |  Cinematic: 70

"Clean Digital Perc"
  Punch: 60  |  Snap: 70  |  Body: 30
  Weight: 30 |  Gloss: 90 |  Warmth: 10
  Grit: 5    |  Realism: 10
```

### Real-Time Preview

```typescript
// Controls update in real-time as user drags
// Debounce DSP processing (50ms) for smooth interaction

export function useSoundControls(audioBuffer: AudioBuffer) {
  const [controls, setControls] = useState<SoundControls>(defaultControls);
  const [processedBuffer, setProcessedBuffer] = useState<AudioBuffer | null>(null);
  const processingRef = useRef<AbortController | null>(null);
  
  const updateControl = useCallback(async (name: string, value: number) => {
    // Cancel any in-flight processing
    processingRef.current?.abort();
    const controller = new AbortController();
    processingRef.current = controller;
    
    // Update controls immediately (UI feels responsive)
    setControls(prev => ({ ...prev, [name]: value }));
    
    // Debounce heavy processing
    const timeout = setTimeout(async () => {
      const result = await invoke('process_controls', {
        audio: audioBuffer.getChannelData(0),
        sampleRate: audioBuffer.sampleRate,
        controls: { ...controls, [name]: value },
      });
      
      if (!controller.signal.aborted) {
        setProcessedBuffer(result);
      }
    }, 50);
    
    return () => clearTimeout(timeout);
  }, [audioBuffer, controls]);
  
  return { controls, updateControl, processedBuffer };
}
```

---

## 6. Safe Ranges & Warnings

```rust
pub struct ControlSafety {
    pub name: &'static str,
    pub safe_min: f32,
    pub safe_max: f32,
    pub warning_at_extreme: Option<&'static str>,
}

pub const CONTROL_SAFETY: &[ControlSafety] = &[
    ControlSafety { name: "punch", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: Some("May cause distortion on already-hot sounds") },
    ControlSafety { name: "body", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: Some("Too much body may cause muddiness") },
    ControlSafety { name: "gloss", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: Some("Maximum gloss may sound harsh on bright sources") },
    ControlSafety { name: "air", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: None },
    ControlSafety { name: "weight", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: Some("Maximum weight may cause sub-bass overload") },
    ControlSafety { name: "snap", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: None },
    ControlSafety { name: "warmth", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: None },
    ControlSafety { name: "width", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: Some("Maximum width may cause phase issues in mono") },
    ControlSafety { name: "grit", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: Some("Maximum grit may sound harsh on melodic content") },
    ControlSafety { name: "darkness", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: Some("Maximum darkness may remove too much high-end") },
    ControlSafety { name: "chaos", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: Some("Maximum chaos may produce unpredictable results") },
    ControlSafety { name: "realism", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: None },
    ControlSafety { name: "plasticity", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: None },
    ControlSafety { name: "cinematic", safe_min: 0.0, safe_max: 100.0, warning_at_extreme: Some("Maximum scale may wash out transient detail") },
];

pub fn validate_controls(controls: &SoundControls) -> Vec<ControlWarning> {
    CONTROL_SAFETY.iter().filter_map(|safety| {
        let value = controls.get(safety.name);
        let at_extreme = value >= safety.safe_max || value <= safety.safe_min;
        
        if at_extreme {
            safety.warning_at_extreme.map(|msg| ControlWarning {
                control: safety.name,
                message: msg,
                level: if value >= 95.0 || value <= 5.0 { WarningLevel::Caution }
                       else { WarningLevel::Info },
            })
        } else {
            None
        }
    }).collect()
}
```

---

## 7. Summary

Fourteen high-level controls map perceptual descriptors to DSP parameters. A beginner can shape a sound without knowing what a transient shaper or low-pass filter is. Each control has a clear perceptual definition, audio feature mapping, DSP implementation, model conditioning path, safe range, and user experience description. The controls group into transient, tone, space, texture, and bass categories. Character presets give users starting points. Real-time preview updates as controls change.
