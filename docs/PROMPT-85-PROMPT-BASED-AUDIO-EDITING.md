# Prompt 85 — Prompt-Based Audio Editing

Users edit generated one-shots with natural language. "Make it punchier." "Remove the harsh top end." The system maps intent to DSP or model-based edits, non-destructively.

---

## 1. Edit-Intent Parser

### Intent Taxonomy

```
EDIT INTENTS (verbs that describe WHAT to change)

Level 1 — Global character
  make it [characteristic]     → "make it punchier"
  make it less [characteristic] → "make it less boomy"
  add more [quality]            → "add more body"
  remove the [quality]          → "remove the harshness"

Level 2 — Specific parameter
  more/less [parameter]        → "more attack"
  increase/decrease [parameter] → "decrease the decay"
  shorter/longer [parameter]   → "shorter release"
  [parameter] up/down          → "pitch down"

Level 3 — Analog/metaphor
  make it sound like [reference] → "make it sound vinyl"
  give it more [character]       → "give it more crunch"
  make it [adjective]           → "make it darker, grittier"

Level 4 — Comparative
  more like [type]               → "more like a 909"
  less like [type]               → "less like a trap snare"
  somewhere between [a] and [b]  → "between a clap and a snare"

Level 5 — Fix/repair
  fix the [problem]              → "fix the ringing"
  remove the [artifact]          → "remove the click"
  clean up the [issue]           → "clean up the low end"
```

### Parser Architecture

```python
class EditIntentParser:
    """
    Parses natural language edit commands into structured edit operations.
    Two-stage: intent classifier + parameter extractor.
    """

    def __init__(self):
        self.intent_classifier = self.load_intent_classifier()
        self.parameter_extractor = self.load_parameter_extractor()
        self.synonym_map = self.load_synonyms()
        self.negation_map = self.load_negations()

    def parse(self, edit_command: str) -> EditIntent:
        # Stage 1: Classify the primary intent
        intent = self.intent_classifier(edit_command)

        # {'verb': 'increase', 'target': 'attack', 'domain': 'transient',
        #  'confidence': 0.92, 'comparative': False}

        # Stage 2: Extract parameters from the command
        params = self.parameter_extractor(edit_command, intent)

        # {'amount': 0.3, 'unit': 'relative', 'frequency': None, 'time': None}

        # Stage 3: Resolve synonyms and negations
        resolved_intent = self.resolve_semantics(intent, params)

        return EditIntent(
            action=resolved_intent.action,       # 'increase' | 'decrease' | 'set' | 'remove'
            target=resolved_intent.target,        # 'attack' | 'decay' | 'brightness' | ...
            domain=resolved_intent.domain,        # 'transient' | 'spectral' | 'temporal' | 'tonal'
            amount=resolved_intent.amount,        # -1.0 to 1.0 (normalized)
            unit=resolved_intent.unit,            # 'relative' | 'absolute' | 'target_value'
            confidence=resolved_intent.confidence,
            original_text=edit_command,
        )
```

### Intent Classifier Training

```python
# Training data for intent classifier
# Format: (edit_command, intent_label, parameters)

TRAINING_EXAMPLES = [
    ("make it punchier",
     {"action": "increase", "target": "punch", "domain": "transient",
      "amount": 0.5, "confidence_weight": 0.9}),

    ("remove the harsh top end",
     {"action": "decrease", "target": "high_frequency_content",
      "domain": "spectral", "amount": 0.7, "frequency_range": "8k-16k",
      "confidence_weight": 0.95}),

    ("add more body",
     {"action": "increase", "target": "low_mid_content",
      "domain": "spectral", "amount": 0.4, "frequency_range": "200-500",
      "confidence_weight": 0.85}),

    ("make the transient sharper",
     {"action": "increase", "target": "transient_sharpness",
      "domain": "transient", "amount": 0.6,
      "confidence_weight": 0.9}),

    ("make it less metallic",
     {"action": "decrease", "target": "metallic_character",
      "domain": "spectral", "amount": 0.5,
      "confidence_weight": 0.8}),

    ("make it cleaner",
     {"action": "decrease", "target": "noise_floor",
      "domain": "noise", "amount": 0.3,
      "confidence_weight": 0.7}),

    ("make it more analog",
     {"action": "increase", "target": "analog_warmth",
      "domain": "tonal", "amount": 0.5,
      "confidence_weight": 0.75}),

    ("make it hit harder",
     {"action": "increase", "target": "impact",
      "domain": "transient", "amount": 0.7,
      "confidence_weight": 0.9}),

    ("make it shorter",
     {"action": "decrease", "target": "duration",
      "domain": "temporal", "amount": 0.5,
      "confidence_weight": 0.95}),

    ("more sub presence",
     {"action": "increase", "target": "sub_bass_content",
      "domain": "spectral", "amount": 0.5, "frequency_range": "20-60",
      "confidence_weight": 0.9}),

    ("less ring",
     {"action": "decrease", "target": "ringing",
      "domain": "spectral", "amount": 0.5,
      "confidence_weight": 0.85}),

    ("tighten it up",
     {"action": "decrease", "target": "decay_time",
      "domain": "temporal", "amount": 0.4,
      "confidence_weight": 0.8}),

    ("make it fatter",
     {"action": "increase", "target": "body",
      "domain": "spectral", "amount": 0.5,
      "confidence_weight": 0.8}),

    ("soften the attack",
     {"action": "decrease", "target": "attack_level",
      "domain": "transient", "amount": 0.4,
      "confidence_weight": 0.9}),

    ("give it more crunch",
     {"action": "increase", "target": "saturation_amount",
      "domain": "tonal", "amount": 0.5,
      "confidence_weight": 0.85}),

    ("scoop the mids",
     {"action": "decrease", "target": "mid_frequency_content",
      "domain": "spectral", "amount": 0.5, "frequency_range": "400-800",
      "confidence_weight": 0.8}),

    ("boost the highs",
     {"action": "increase", "target": "high_frequency_content",
      "domain": "spectral", "amount": 0.4, "frequency_range": "5k-12k",
      "confidence_weight": 0.9}),
]
```

### Parameter Normalization

```python
@dataclass
class EditIntent:
    action: str            # 'increase' | 'decrease' | 'set' | 'remove' | 'replace'
    target: str            # normalized attribute name
    domain: str            # 'transient' | 'spectral' | 'temporal' | 'tonal' | 'noise'
    amount: float          # -1.0 to 1.0 (negative = decrease direction)
    unit: str              # 'relative' | 'absolute' | 'target_value'
    confidence: float      # 0-1 how confident the parser is

    # Optional specifics
    frequency_range: Optional[Tuple[float, float]] = None
    time_range: Optional[Tuple[float, float]] = None
    target_value: Optional[float] = None

    def to_dsp_parameters(self, audio_analysis: AudioAnalysis) -> DSPParams:
        """
        Convert normalized edit intent to concrete DSP parameters.
        Uses audio analysis to set context-appropriate values.
        """
        mapping = EDIT_TO_DSP_MAPPING[(self.domain, self.target)]

        # Base value from mapping
        base_value = mapping.default_value

        # Adjust based on audio context
        if mapping.context_sensitive:
            base_value = mapping.compute_base(audio_analysis)

        # Apply the edit amount
        if self.unit == 'relative':
            final_value = base_value * (1 + self.amount * mapping.range)
        elif self.unit == 'absolute':
            final_value = self.amount
        elif self.unit == 'target_value':
            final_value = self.target_value

        # Clamp to valid range
        final_value = max(mapping.min_value, min(mapping.max_value, final_value))

        return DSPParams(
            processor=mapping.processor_id,
            parameter=mapping.param_name,
            value=final_value,
            frequency_range=self.frequency_range,
        )
```

---

## 2. DSP / Action Mapping

### Edit-to-DSP Mapping Table

```
Domain: TRANSIENT

  Intent Target         │ DSP Processor       │ Parameters
  ──────────────────────┼─────────────────────┼────────────────────────────
  attack                │ TransientShaper      │ attack_time, attack_gain
  punch                 │ TransientShaper      │ attack_gain (+sustain_cut)
  transient_sharpness   │ TransientShaper      │ attack_time (shorter=sharper)
  impact                │ Clipper + Envelope   │ clip_threshold + attack_boost
  attack_level          │ Gain Envelope        │ attack_gain_reduction
  decay_time            │ Envelope Follower    │ decay_time_constant
  release               │ Envelope Follower    │ release_time_constant
  sustain               │ Gain Envelope        │ sustain_level

Domain: SPECTRAL

  Intent Target         │ DSP Processor       │ Parameters
  ──────────────────────┼─────────────────────┼────────────────────────────
  brightness            │ EQ Shelf             │ high_shelf_gain (8k-12k)
  warmth                │ EQ Low Shelf         │ low_shelf_gain (100-300)
  body                  │ EQ Bell              │ bell_gain @ 200-400 Hz
  low_mid_content       │ EQ Bell              │ bell_gain @ 200-500 Hz
  mid_frequency_content │ EQ Bell              │ bell_gain @ 400-800 Hz
  high_frequency_content│ EQ High Shelf        │ high_shelf_gain
  sub_bass_content      │ EQ Low Shelf         │ low_shelf_gain (20-60 Hz)
  metallic_character    │ Notch Filter         │ notch_depth @ 2k-8k
  harshness             │ Dynamic EQ           │ reduction @ problematic freq
  ringing               │ Notch Filter         │ narrow_notch @ ringing_freq
  air                   │ EQ High Shelf        │ high_shelf_gain (12k-20k)
  presence              │ EQ Bell              │ bell_gain @ 3k-5k
  boominess             │ EQ Low Shelf         │ low_shelf_cut @ 100-250
  muddiness             │ EQ High Pass         │ hp_cut @ 80-150 Hz
  nasality              │ EQ Bell              │ bell_cut @ 800-2k

Domain: TEMPORAL

  Intent Target         │ DSP Processor       │ Parameters
  ──────────────────────┼─────────────────────┼────────────────────────────
  duration              │ Time Stretch        │ stretch_ratio (0.1-10x)
  length                │ Trim + Fade         │ new_duration, fade_type
  start                 │ Trim (front)        │ trim_start_ms
  end                   │ Trim (tail)         │ trim_end_ms
  spacing               │ Time Stretch        │ stretch_ratio (for pattern)

Domain: TONAL

  Intent Target         │ DSP Processor       │ Parameters
  ──────────────────────┼─────────────────────┼────────────────────────────
  saturation_amount     │ Saturator           │ drive, saturation_type
  distortion_level      │ Waveshaper          │ drive, curve_type
  compression_amount    │ Compressor          │ ratio, threshold, makeup
  analog_warmth         │ Tape Saturator      │ drive, bias, noise_floor
  bit_crush             │ Bitcrusher          │ bit_depth, sample_rate_red
  tube_character        │ Tube Saturator      │ drive, harmonics_mix

Domain: NOISE

  Intent Target         │ DSP Processor       │ Parameters
  ──────────────────────┼─────────────────────┼────────────────────────────
  noise_floor           │ Noise Gate          │ threshold, reduction
  hiss                  │ Noise Reduction     │ reduction_db, noise_profile
  click                 │ De-click            │ threshold, max_click_dur
  pop                   │ De-click            │ threshold (low freq)
  background_hum        │ Notch Filter        │ notch @ 50/60 Hz + harmonics
```

### DSP Engine

```rust
/// Process an audio buffer through a chain of DSP processors.
/// Each processor is configured by the edit intent parser.

struct DSPChain {
    processors: Vec<Box<dyn AudioProcessor>>,
}

#[async_trait]
trait AudioProcessor: Send {
    fn id(&self) -> &str;
    fn process(&mut self, buffer: &mut AudioBuffer, params: DSPParams) -> Result<()>;
    fn latency_compensation(&self) -> usize;
    fn bypass(&mut self, should_bypass: bool);
}

// Concrete processors
struct TransientShaper {
    attack_time: f32,      // 0.1-50 ms
    attack_gain: f32,      // -20 to +20 dB
    sustain_cut: f32,      // -20 to 0 dB
    release_time: f32,     // 10-500 ms
}

struct Equalizer {
    bands: Vec<EQBand>,
    // LowShelf, HighShelf, Bell, Notch, HighPass, LowPass
}

struct Saturator {
    drive: f32,            // 0-24 dB
    curve: SaturationCurve, // Tape, Tube, Exponential, HardClip
    mix: f32,              // 0-1 dry/wet
}

struct Compressor {
    threshold: f32,        // -60 to 0 dB
    ratio: f32,            // 1:1 to 20:1
    attack: f32,           // 0.1-50 ms
    release: f32,          // 10-500 ms
    makeup: f32,           // 0-24 dB
}
```

---

## 3. Model-Based Editing

### When DSP Is Not Enough

```
DSP (explicit signal processing):
  ✓ Fast, deterministic, zero-latency
  ✓ Perfect for: EQ, compression, transient shaping, saturation
  ✓ Limitation: can't change fundamental timbre or sound class

Model-based (neural audio processing):
  ✓ Can change fundamental character
  ✓ Perfect for: cross-synthesis, timbre transfer, style transfer
  ✓ Limitation: slower, less predictable, may introduce artifacts

Hybrid approach:
  Use DSP for all well-defined edits (EQ, envelope, gain)
  Use model for edits requiring semantic understanding
  Fall back from model to DSP when confidence is low
```

### Model Editing Types

```
1. Timbre transfer
   "Make it sound like a 909 kick" → style transfer model
   Input: current kick + target timbre embedding → output
   Model: AudioStyleTransfer (autoencoder + AdaIN)

2. Cross-synthesis
   "Mix this kick with this clap" → layering model
   Input: kick + clap → blended output
   Model: AudioLDM-based cross-synthesis

3. Repair
   "Remove the ringing" → blind source separation
   Input: audio → separated ringing + clean
   Model: Demucs-like separation

4. Text-guided editing
   "Make it sound like it's in a cathedral" → text-guided reverb
   Input: audio + text prompt → edited audio
   Model: AudioLDM Edit (latent diffusion + editing)

5. Parameter inference
   "More aggressive" → analyze audio, suggest DSP params
   Input: audio + text → DSP parameter set
   Model: Small regression model
```

### Model Selection Logic

```python
def select_edit_method(intent: EditIntent, audio: AudioBuffer) -> EditMethod:
    """
    Decide whether to use DSP or model-based editing.
    Based on: edit type, available models, latency requirements.
    """
    # DSP-only edits (fast, always available)
    if intent.domain in ('temporal', 'noise'):
        return EditMethod.DSP
    if intent.target in ('attack', 'decay', 'sustain', 'release'):
        return EditMethod.DSP
    if intent.target in ('duration', 'length', 'start', 'end'):
        return EditMethod.DSP

    # Model preferred edits (semantic change)
    if intent.target in ('timbre', 'character', 'texture'):
        if model_manager.is_loaded('timbre_transfer'):
            return EditMethod.MODEL_TIMBRE_TRANSFER
        return EditMethod.DSP_APPROXIMATION  # best-effort via EQ

    if intent.target in ('style', 'genre'):
        if model_manager.is_loaded('style_transfer'):
            return EditMethod.MODEL_STYLE_TRANSFER
        return EditMethod.UNSUPPORTED  # tell user it's not possible offline

    # Hybrid
    if intent.target == 'brightness':
        return EditMethod.DSP  # EQ is better than model for this

    # Default: DSP
    return EditMethod.DSP
```

### Latent Editing

```python
class LatentEditor:
    """
    Edit audio in latent space using text-guided diffusion editing.
    Approach: encode → edit latent → decode
    Based on: SDEdit / DiffEdit for audio.
    """

    def edit(self, audio: AudioBuffer, prompt: str, strength: float = 0.5) -> AudioBuffer:
        # 1. Encode audio to latent
        latent = self.vae.encode(audio)

        # 2. Add noise to latent (controlled by strength)
        # strength=0: no change, strength=1: complete regeneration
        noise_level = strength * self.max_noise
        noisy_latent = self.add_noise(latent, noise_level)

        # 3. Denoise with text guidance
        # The model reconstructs from noise, guided by edit prompt
        edited_latent = self.diffusion.denoise(
            noisy_latent,
            condition=self.text_encoder(prompt),
            guidance_scale=3.0,
            steps=25,  # fewer steps for editing vs generation
        )

        # 4. Decode back to audio
        return self.vae.decode(edited_latent)

    def edit_with_mask(self, audio: AudioBuffer, prompt: str,
                       mask: TimeMask, strength: float = 0.5) -> AudioBuffer:
        """
        Edit only a specific time region.
        E.g., "make the attack punchier" → edit first 50ms only.
        """
        # Apply mask: only edit masked region
        original = audio.clone()
        edited_region = self.edit(audio.slice(mask.start, mask.end), prompt, strength)
        return original.replace_region(mask, edited_region)
```

---

## 4. Undo/Versioning

### Edit History

```python
class EditHistory:
    """
    Non-destructive edit history with branching support.
    Every edit is stored as a delta, not a new file.
    """

    def __init__(self, sound_id: str, original_audio: AudioBuffer):
        self.sound_id = sound_id
        self.original_hash = compute_hash(original_audio)
        self.entries: List[EditEntry] = []
        self.branches: Dict[str, List[EditEntry]] = {}
        self.current_position = -1  # -1 = original, 0 = first edit, etc.

    def apply_edit(self, intent: EditIntent, params: DSPParams,
                   before_audio: AudioBuffer, after_audio: AudioBuffer) -> EditEntry:
        entry = EditEntry(
            id=generate_uuid(),
            parent_id=self.entries[-1].id if self.entries else None,
            intent=intent,
            params=params,
            before_hash=compute_hash(before_audio),
            after_hash=compute_hash(after_audio),
            before_waveform=extract_waveform_peaks(before_audio),
            after_waveform=extract_waveform_peaks(after_audio),
            timestamp=datetime.now(),
            description=intent.original_text,
        )
        self.entries.append(entry)
        self.current_position = len(self.entries) - 1
        return entry

    def undo(self) -> Optional[EditEntry]:
        if self.current_position < 0:
            return None  # Already at original
        entry = self.entries[self.current_position]
        self.current_position -= 1
        return entry  # Return entry with before_hash to reconstruct

    def redo(self) -> Optional[EditEntry]:
        if self.current_position >= len(self.entries) - 1:
            return None  # Already at latest
        self.current_position += 1
        return self.entries[self.current_position]

    def branch(self, branch_name: str, from_position: Optional[int] = None):
        pos = from_position or self.current_position
        self.branches[branch_name] = self.entries[:pos + 1].copy()

    def get_current_state(self) -> AudioBuffer:
        """Reconstruct audio by replaying all edits up to current_position."""
        audio = self.load_original()
        for i in range(self.current_position + 1):
            entry = self.entries[i]
            audio = self.apply_dsp_from_entry(audio, entry)
        return audio


@dataclass
class EditEntry:
    id: str
    parent_id: Optional[str]
    intent: EditIntent
    params: DSPParams
    before_hash: str
    after_hash: str
    before_waveform: WaveformPeaks  # for visual diff
    after_waveform: WaveformPeaks   # for visual diff
    timestamp: datetime
    description: str
```

### Storage

```rust
/// Edit history is stored as a JSON document alongside the sound.
/// Not in SQLite — edit histories can grow large and are accessed
/// only when the user opens the edit panel.

#[derive(Serialize, Deserialize)]
struct EditHistoryDocument {
    sound_id: String,
    original_hash: String,
    entries: Vec<EditHistoryEntry>,

    // Branches: name -> index range
    branches: HashMap<String, (usize, usize)>,
}

#[derive(Serialize, Deserialize)]
struct EditHistoryEntry {
    id: String,
    parent_id: Option<String>,

    // The parsed intent
    action: String,
    target: String,
    amount: f32,

    // Parameters used
    processor: String,
    parameters: HashMap<String, f32>,

    // Audio before/after hashes for cache lookup
    before_hash: String,
    after_hash: String,

    // Timestamps
    created_at: String,
}
```

---

## 5. Before/After Comparison

### Comparison UI

```
┌─────────────────────────────────────────────────────┐
│  Edit: "Make it punchier"                           │
│                                                     │
│  ┌───────────────────┐  ┌───────────────────┐       │
│  │    BEFORE          │  │    AFTER           │       │
│  │                    │  │                    │       │
│  │  [Waveform]        │  │  [Waveform]        │       │
│  │                    │  │                    │       │
│  │  Duration: 1.2s    │  │  Duration: 1.2s    │       │
│  │  Peak: -3.2 dB     │  │  Peak: -1.1 dB     │       │
│  │  LUFS: -16.4       │  │  LUFS: -13.2       │       │
│  │                    │  │                    │       │
│  │  [► Play Before]   │  │  [► Play After]    │       │
│  └───────────────────┘  └───────────────────┘       │
│                                                     │
│  ┌──────────────────────────────────────────────┐   │
│  │  Diff waveform (after - before, exaggerated)  │   │
│  │  [//////////////////////////////////]         │   │
│  │  Red = added energy    Blue = removed         │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
│  ┌──────────────────────────────────────────────┐   │
│  │  Spectral difference                          │   │
│  │  [Spectrogram diff overlay]                   │   │
│  │  Warm colors = energy added                   │   │
│  │  Cool colors = energy removed                 │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
│  [Accept]  [Reject]  [Tweak Parameters...]          │
└─────────────────────────────────────────────────────┘
```

### Audio Difference Calculation

```rust
/// Compute perceptual difference between before and after.
/// Used for visualization and confidence estimation.

struct AudioDiff {
    // Time-domain difference
    waveform_diff: Vec<f32>,          // sample-by-sample difference

    // Spectral difference
    spectral_diff: Vec<Vec<f32>>,    // frequency bins × time frames

    // Perceptual metrics
    loudness_change: f32,             // LUFS difference
    spectral_centroid_shift: f32,     // Hz
    transient_change: f32,           // transient energy ratio difference
    harmonic_change: f32,            // harmonic content difference

    // Semantic change
    edit_intended_effect_achieved: bool,  // Did the edit do what was intended?
}

impl AudioDiff {
    fn compute(before: &AudioBuffer, after: &AudioBuffer) -> Self {
        // 1. Sample-by-sample waveform difference
        let waveform_diff: Vec<f32> = before.samples
            .iter()
            .zip(after.samples.iter())
            .map(|(b, a)| a - b)
            .collect();

        // 2. Spectrogram difference (STFT magnitude)
        let before_spec = stft(before);
        let after_spec = stft(after);
        let spectral_diff: Vec<Vec<f32>> = before_spec
            .iter()
            .zip(after_spec.iter())
            .map(|(b, a)| {
                b.iter().zip(a.iter())
                    .map(|(b, a)| a - b)
                    .collect()
            })
            .collect();

        // 3. Perceptual metrics
        let loudness_change = compute_loudness(after) - compute_loudness(before);
        let centroid_shift = spectral_centroid(after) - spectral_centroid(before);

        AudioDiff {
            waveform_diff,
            spectral_diff,
            loudness_change,
            spectral_centroid_shift: centroid_shift,
            transient_change: compute_transient_diff(before, after),
            harmonic_change: compute_harmonic_diff(before, after),
            edit_intended_effect_achieved: false, // set by intent verifier
        }
    }
}
```

---

## 6. Confidence Estimation

### Edit Confidence

```python
class EditConfidenceEstimator:
    """
    Estimates how likely the edit will sound good.
    Returns score + explanation.
    """

    def estimate(self, intent: EditIntent, audio: AudioBuffer) -> EditConfidence:
        factors = []

        # 1. Intent parsing confidence
        factors.append(ConfidenceFactor(
            name="parsing",
            score=intent.confidence,
            detail=f"Parsed as: {intent.action} {intent.target}",
        ))

        # 2. DSP capability confidence
        dsp_available = intent.target in EDIT_TO_DSP_MAPPING
        factors.append(ConfidenceFactor(
            name="dsp_availability",
            score=1.0 if dsp_available else 0.0,
            detail="Available via DSP" if dsp_available else "Requires model-based editing",
        ))

        # 3. Parameter range check
        params = intent.to_dsp_parameters(audio.analysis)
        in_range = params.value >= params.min_value and params.value <= params.max_value
        factors.append(ConfidenceFactor(
            name="parameter_range",
            score=1.0 if in_range else 0.3,
            detail=f"Parameter {params.parameter}={params.value:.2f} is {
                'in range' if in_range else 'clamped to range'}",
        ))

        # 4. Audio suitability
        suitability = self.check_audio_suitability(intent, audio)
        factors.append(ConfidenceFactor(
            name="audio_suitability",
            score=suitability.score,
            detail=suitability.explanation,
        ))

        # 5. Model availability (if needed)
        if not dsp_available:
            model_ready = self.model_manager.is_loaded_for(intent.target)
            factors.append(ConfidenceFactor(
                name="model_availability",
                score=1.0 if model_ready else 0.0,
                detail="Model loaded" if model_ready else "Model not available offline",
            ))

        # Aggregate
        weights = {
            "parsing": 0.25,
            "dsp_availability": 0.25,
            "parameter_range": 0.15,
            "audio_suitability": 0.25,
            "model_availability": 0.10,
        }

        total_score = sum(f.score * weights.get(f.name, 0) for f in factors)
        total_weight = sum(weights.get(f.name, 0) for f in factors)

        return EditConfidence(
            score=total_score / total_weight,
            factors=factors,
        )
```

### Post-Edit Verification

```python
class EditVerifier:
    """
    After applying the edit, verify it achieved the intended effect.
    If not, suggest alternatives or roll back.
    """

    def verify(self, intent: EditIntent, before: AudioBuffer,
               after: AudioBuffer) -> VerificationResult:
        # Analyze what changed
        diff = AudioDiff::compute(before, after)

        # Check if the intended change happened
        expected_changes = self.get_expected_changes(intent)
        actual_changes = self.measure_actual_changes(diff, intent)

        # How well did the edit match the intent?
        match_score = self.compute_match(expected_changes, actual_changes)

        if match_score > 0.7:
            return VerificationResult(
                success=True,
                match_score=match_score,
                message="Edit applied successfully",
            )
        elif match_score > 0.4:
            return VerificationResult(
                success=True,
                match_score=match_score,
                message="Edit partially applied. Consider adjusting: " + self.get_suggestion(intent, diff),
                suggestion=self.get_suggestion(intent, diff),
            )
        else:
            return VerificationResult(
                success=False,
                match_score=match_score,
                message="Edit did not achieve intended effect",
                suggestion=self.get_alternative(intent),
                auto_rollback=True,
            )

    def get_expected_changes(self, intent: EditIntent) -> List[ExpectedChange]:
        """What should change in the audio if the edit is successful?"""
        if intent.target == "punch":
            return [
                ExpectedChange("transient_energy", direction="increase", min_delta=0.1),
                ExpectedChange("attack_time", direction="decrease", min_delta=0.05),
                ExpectedChange("spectral_slope", direction="increase", min_delta=0.02),
            ]
        elif intent.target == "brightness":
            return [
                ExpectedChange("spectral_centroid", direction="increase", min_delta=200),
                ExpectedChange("high_freq_energy", direction="increase", min_delta=0.1),
            ]
        # ... etc for all edit targets
```

---

## 7. Complete Edit Pipeline

```
                       USER: "Make it punchier"
                              │
                              ▼
                    ┌──────────────────┐
                    │ Intent Parser    │
                    │ {action: inc,    │
                    │  target: punch,  │
                    │  confidence: 0.92│
                    │  amount: 0.5}    │
                    └────────┬─────────┘
                             │
                    ┌────────▼─────────┐
                    │ Confidence        │
                    │ Estimator         │
                    │ Score: 0.87       │
                    │ "High confidence" │
                    └────────┬─────────┘
                             │
              ┌──────────────┴──────────────┐
              │                              │
     ┌────────▼────────┐          ┌─────────▼────────┐
     │ DSP Mapping     │          │ Model Selection   │
     │ TransientShaper │          │ (if DSP not       │
     │ attack_gain +3dB│          │  sufficient)      │
     │ sustain_cut -2dB│          │                   │
     └────────┬────────┘          └───────────────────┘
              │
     ┌────────▼────────┐
     │ Apply DSP       │
     │ → new audio     │
     └────────┬────────┘
              │
     ┌────────▼────────┐
     │ Verify Effect   │
     │ "Transient      │
     │  energy +15% ✓  │
     │  Attack -3ms ✓" │
     └────────┬────────┘
              │
     ┌────────▼────────┐
     │ Record Edit     │
     │ in History      │
     │ (non-destructive)│
     └────────┬────────┘
              │
     ┌────────▼────────┐
     │ Show Before/    │
     │ After Comparison│
     │ [Accept/Reject] │
     └─────────────────┘
```

---

## Summary

1. **Edit-intent parser**: Classifies commands into structured intents (action + target + amount), handles synonyms and negation
2. **DSP/action mapping**: Comprehensive table mapping 50+ edit targets to specific DSP processors with context-aware parameter computation
3. **Model-based editing**: Latent-space diffusion editing for edits DSP cannot handle (timbre transfer, style change, cross-synthesis)
4. **Undo/versioning**: Full non-destructive edit history with branching — stored as deltas, reconstructed by replay
5. **Before/after comparison**: Waveform diff, spectral diff, perceptual metrics, side-by-side playback
6. **Confidence estimation**: Multi-factor confidence score — parsing quality, DSP availability, parameter range, audio suitability, model readiness — with post-edit verification
