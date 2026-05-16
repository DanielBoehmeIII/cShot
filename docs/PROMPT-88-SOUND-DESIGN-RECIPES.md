# Prompt 88 — Sound Design Recipes

A recipe system for cShot. Each recipe is a repeatable, parameterized blueprint for creating a specific type of one-shot.

---

## 1. Recipe Format

### Recipe Structure

```yaml
recipe:
  id: "cinematic_impact"
  name: "Cinematic Impact"
  description: "A massive, epic impact sound for trailers and drops"
  level: 3                    # difficulty 1-5
  estimated_time: "2-5 min"   # from generation to finished sound

  # What the user types to get this
  default_prompt: "epic cinematic impact, massive, orchestral, sub drop, whoosh"

  # Audio characteristics (for quality check)
  target_characteristics:
    duration_ms: 1500-3000
    loudness_lufs: -14 to -10
    spectral_centroid_hz: 500-2000
    stereo_width: 0.5-1.0
    has_sub: true
    has_noise_component: true

  # How to generate the raw material
  source_ingredients:
    - type: "generation"
      prompt: "deep sub boom, 30Hz, long decay, 2 seconds"
      role: "low_end"
      model: "cshot-generator-v1"
    - type: "generation"
      prompt: "white noise swell, rising, 2 seconds"
      role: "noise_sweep"
      model: "cshot-generator-v1"
    - type: "generation"
      prompt: "metallic crash, bright, short reverb tail"
      role: "top_end_crash"
      model: "cshot-generator-v1"

  # What DSP to apply, in order
  processing_chain:
    - processor: "time_stretch"
      params:
        source: "low_end"
        stretch_ratio: 1.5      # Make it longer
    - processor: "equalizer"
      params:
        source: "noise_sweep"
        high_pass: 200
        low_pass: 8000
        bell_gain_2k: 3.0       # Boost presence
    - processor: "compressor"
      params:
        source: "all"
        ratio: 4.0
        threshold: -18
        attack: 5ms
        release: 200ms
        makeup: 3.0
    - processor: "reverb"
      params:
        source: "all"
        room_size: 0.8
        decay: 2.5s
        mix: 0.3               # 30% wet
    - processor: "saturator"
      params:
        source: "all"
        drive: 3.0
        type: "tape"
        mix: 0.5
    - processor: "limiter"
      params:
        ceiling: -0.5
        release: 100ms

  # Key parameters (for recipe browser / parameter presets)
  key_parameters:
    intensity: 0.7            # 0-1: more = bigger, more compressed
    decay_length: 0.6         # 0-1: shorter vs longer tail
    brightness: 0.5           # 0-1: dark vs bright
    stereo_spread: 0.7        # 0-1: narrow vs wide

  # How to create variations
  variation_strategies:
    - change: "pitch"
      range: [-5, 5]          # semitones
      effect: "Different key feel"
    - change: "decay_length"
      range: [0.3, 0.9]
      effect: "Tighter vs more epic"
    - change: "reverb_mix"
      range: [0.1, 0.7]
      effect: "Dry vs ambient"
    - change: "intensity"
      range: [0.3, 1.0]
      effect: "Subtle vs over-the-top"
    - swap: "top_end_crash"
      alternatives: ["orchestral hit", "gong hit", "glass break"]
      effect: "Different impact character"

  # Signs the recipe failed
  failure_signs:
    - "No sub bass content (< 40Hz)"
    - "Duration too short (< 1s) or long (> 5s)"
    - "Clipping (peak > -0.1 dBFS)"
    - "Too quiet (LUFS < -20)"
    - "No noise component (spectral_crest < 0.3)"
    - "Mono-compatibility issues (phase correlation < 0.3)"

  # Export settings
  export_defaults:
    format: "wav"
    sample_rate: 48000
    bit_depth: 24
    channels: 2               # stereo (impacts should be wide)
    normalize: true
    normalize_target: -14
```

---

## 2. Recipe Library

### Genre Recipes

```yaml
recipe:
  id: "glossy_hyperpop_snare"
  name: "Glossy Hyperpop Snare"
  level: 2
  default_prompt: "hyperpop snare, bright, glossy, pitchy, short, crisp"

  source_ingredients:
    - type: "generation"
      prompt: "crisp snare, 200Hz body, bright top, 300ms"
      role: "snare_body"
    - type: "generation"
      prompt: "white noise burst, high-passed at 5kHz, sharp attack"
      role: "noise_top"

  processing_chain:
    - processor: "pitch_shift"
      params:
        source: "snare_body"
        shift: 2               # +2 semitones (brighter)
    - processor: "transient_shaper"
      params:
        source: "snare_body"
        attack_gain: 4.0
        attack_time: 3ms
    - processor: "equalizer"
      params:
        source: "all"
        low_cut: 150
        high_shelf_10k: 4.0   # Boost air
        bell_200: -3.0        # Reduce boxiness
        bell_5k: 3.0          # Boost snap
    - processor: "compressor"
      params:
        ratio: 6.0
        attack: 1ms
        release: 80ms
        makeup: 2.0
    - processor: "saturator"
      params:
        drive: 2.0
        type: "tube"
        mix: 0.4
    - processor: "limiter"
      params:
        ceiling: -0.3
        release: 50ms

  key_parameters:
    pitch: 2                  # semitones from original
    snap: 0.7                 # 0-1: transient emphasis
    air: 0.8                  # 0-1: high-frequency sheen
    body: 0.4                 # 0-1: low-mid presence

  variation_strategies:
    - change: "pitch"
      range: [-3, 5]
      effect: "Darker to brighter snare"
    - change: "snap"
      range: [0.3, 1.0]
      effect: "Soft to aggressive attack"
    - swap: "noise_top"
      alternatives: ["808 clap top", "layered paper tear", "vinyl crackle"]
      effect: "Different texture character"

  export_defaults:
    format: "wav"
    sample_rate: 44100
    bit_depth: 24
    channels: 1
```

```yaml
recipe:
  id: "dark_drill_808_hit"
  name: "Dark Drill 808 Hit"
  level: 2
  default_prompt: "drill 808 hit, dark, heavy sub, punchy, distorted"

  source_ingredients:
    - type: "generation"
      prompt: "808 sub kick, 35Hz fundamental, long decay, 800ms"
      role: "sub"
    - type: "generation"
      prompt: "punchy kick attack, click, 5kHz transient, 50ms"
      role: "attack"

  processing_chain:
    - processor: "pitch_shift"
      params:
        source: "sub"
        shift: -3              # -3 semitones (darker)
    - processor: "equalizer"
      params:
        source: "sub"
        low_shelf_40: 4.0     # Boost sub
        high_cut: 120         # Cut everything above 120Hz
    - processor: "equalizer"
      params:
        source: "attack"
        low_cut: 2000
        high_shelf_5k: 6.0   # Boost click
    - processor: "compressor"
      params:
        source: "attack"
        ratio: 8.0
        attack: 0.5ms
        release: 30ms
        makeup: 6.0
    - processor: "saturator"
      params:
        source: "sub"
        drive: 5.0
        type: "tube"
        mix: 0.6             # 60% wet — heavy saturation
    - processor: "limiter"
      params:
        source: "all"
        ceiling: -0.5
        release: 100ms

  key_parameters:
    sub_frequency: 35          # Hz
    sub_distortion: 0.7        # 0-1
    click_attack: 0.8          # 0-1
    body_length: 0.6           # 0-1

  variation_strategies:
    - change: "pitch"
      range: [-5, 0]
      effect: "Darker sub (lower = more felt than heard)"
    - change: "sub_distortion"
      range: [0.3, 1.0]
      effect: "Clean sub to gritty distorted 808"
    - swap: "attack"
      alternatives: ["rimshot click", "metallic ping", "vinyl pop"]
      effect: "Different attack character"

  failure_signs:
    - "Sub frequency above 50Hz (too high for drill)"
    - "No sub content below 40Hz"
    - "Attack too soft (transient_energy < 0.3)"
    - "Duration too long (> 1.5s, will clash with beat)"

  export_defaults:
    format: "wav"
    sample_rate: 44100
    bit_depth: 24
    channels: 1
```

```yaml
recipe:
  id: "crunchy_analog_clap"
  name: "Crunchy Analog Clap"
  level: 2
  default_prompt: "analog clap, crunchy, warm, vintage, drum machine"

  source_ingredients:
    - type: "generation"
      prompt: "drum machine clap, noise burst, 200ms, crisp"
      role: "clap_core"
    - type: "generation"
      prompt: "room ambience, short reverb tail, 400ms"
      role: "ambience"

  processing_chain:
    - processor: "transient_shaper"
      params:
        source: "clap_core"
        attack_gain: 2.0
        sustain_cut: -4.0     # Tighten the clap
    - processor: "saturator"
      params:
        source: "clap_core"
        drive: 6.0
        type: "tape"
        mix: 0.7             # Heavy tape saturation
    - processor: "equalizer"
      params:
        source: "clap_core"
        low_cut: 300
        high_shelf_8k: -2.0  # Roll off highs (vintage vibe)
        bell_1k: 3.0         # Boost body
    - processor: "compressor"
      params:
        source: "all"
        ratio: 3.0
        attack: 10ms          # Let transient through
        release: 150ms
        makeup: 2.0
    - processor: "bitcrusher"
      params:
        source: "clap_core"
        bit_depth: 12         # Reduce bit depth for crunch
        sample_rate_reduction: 0.0
        mix: 0.3

  key_parameters:
    crunch: 0.8               # 0-1: saturation + bitcrush amount
    body: 0.5                 # 0-1: low-mid emphasis
    room: 0.3                 # 0-1: ambience mix
    vintage: 0.6              # 0-1: high-frequency rolloff + noise

  variation_strategies:
    - change: "crunch"
      range: [0.2, 1.0]
      effect: "Clean to gritty"
    - change: "room"
      range: [0.0, 0.7]
      effect: "Dry to washy"
    - change: "vintage"
      range: [0.0, 1.0]
      effect: "Modern clean to lo-fi vintage"
```

```yaml
recipe:
  id: "icy_ambient_texture"
  name: "Icy Ambient Texture"
  level: 3
  default_prompt: "icy ambient texture, cold, metallic, shimmering, evolving"

  source_ingredients:
    - type: "generation"
      prompt: "glass harmonica tone, high pitch, 2 second sustain"
      role: "tonal_layer"
    - type: "generation"
      prompt: "wind noise, cold, thin, 3 seconds"
      role: "noise_layer"
    - type: "generation"
      prompt: "metallic ringing, high frequency, 1 second decay"
      role: "ring_layer"

  processing_chain:
    - processor: "pitch_shift"
      params:
        source: "tonal_layer"
        shift: 12              # +1 octave
    - processor: "equalizer"
      params:
        source: "tonal_layer"
        low_cut: 1000
        high_shelf_4k: 6.0
        bell_2k: 4.0
    - processor: "time_stretch"
      params:
        source: "tonal_layer"
        stretch_ratio: 3.0    # Stretch to 6 seconds
    - processor: "pitch_shift"
      params:
        source: "ring_layer"
        shift: 24              # +2 octaves
    - processor: "reverb"
      params:
        source: "all"
        room_size: 0.95
        decay: 5.0
        damping: 0.1          # Minimal damping (bright reverb)
        pre_delay: 50ms
        mix: 0.8
    - processor: "chorus"
      params:
        source: "all"
        rate: 0.2
        depth: 0.7
        mix: 0.5
    - processor: "compressor"
      params:
        ratio: 2.0
        attack: 50ms
        release: 500ms
        makeup: 0.0

  key_parameters:
    coldness: 0.8              # 0-1: high-frequency emphasis
    shimmer: 0.6               # 0-1: chorus + modulation
    decay: 0.7                 # 0-1: reverb decay length
    movement: 0.5              # 0-1: LFO modulation speed

  variation_strategies:
    - change: "pitch"
      range: [0, 24]
      effect: "Lower ice to higher crystal"
    - change: "decay"
      range: [0.3, 1.0]
      effect: "Shorter chime to long evolving pad"
    - swap: "tonal_layer"
      alternatives: ["sine wave pad", "glass bowl rub", "crystal glass"]
      effect: "Different tonal character"

  export_defaults:
    format: "wav"
    sample_rate: 48000
    bit_depth: 24
    channels: 2
    normalize: true
```

```yaml
recipe:
  id: "jersey_club_bed_squeak"
  name: "Jersey Club Bed Squeak"
  level: 1
  default_prompt: "Jersey club bed squeak, high pitched, springy, bouncy"

  source_ingredients:
    - type: "generation"
      prompt: "short metallic ping, high pitch, 100ms"
      role: "ping"

  processing_chain:
    - processor: "pitch_shift"
      params:
        source: "ping"
        shift: 24              # +2 octaves — very high
    - processor: "equalizer"
      params:
        source: "ping"
        low_cut: 2000
        high_shelf_8k: 8.0    # Boost the squeak
        bell_3k: 6.0
    - processor: "transient_shaper"
      params:
        source: "ping"
        attack_gain: 6.0
        attack_time: 1ms
    - processor: "compressor"
      params:
        ratio: 10.0
        attack: 1ms
        release: 50ms
        makeup: 0.0
    - processor: "limiter"
      params:
        ceiling: -1.0
        release: 10ms

  key_parameters:
    pitch: 24                 # semitones up
    squeak: 0.8               # 0-1: high-frequency emphasis
    bounce: 0.6               # 0-1: compression character

  variation_strategies:
    - change: "pitch"
      range: [12, 36]
      effect: "Lower squeak to mouse squeak"
    - change: "duration"
      range: [30, 200]
      effect: "Short blip to longer squeak"

  export_defaults:
    format: "wav"
    sample_rate: 44100
    bit_depth: 16
    channels: 1
```

```yaml
recipe:
  id: "expensive_pop_perc"
  name: "Expensive Pop Percussion"
  level: 2
  default_prompt: "pop percussion hit, expensive, clean, polished, punchy"

  source_ingredients:
    - type: "generation"
      prompt: "clean percussion hit, wooden tone, bright attack, 500ms"
      role: "perc_core"
    - type: "generation"
      prompt: "subtle shaker, high frequencies, 200ms"
      role: "texture_top"

  processing_chain:
    - processor: "equalizer"
      params:
        source: "perc_core"
        low_cut: 80
        high_shelf_10k: 3.0   # Polished high end
        bell_400: -2.0        # Clean out mud
        bell_2k: 2.0          # Presence bump
    - processor: "transient_shaper"
      params:
        source: "perc_core"
        attack_gain: 2.0
        attack_time: 5ms
    - processor: "compressor"
      params:
        source: "all"
        ratio: 3.0
        attack: 10ms
        release: 100ms
        makeup: 1.0
    - processor: "saturator"
      params:
        source: "all"
        drive: 1.5
        type: "tape"
        mix: 0.2             # Very subtle — just warmth
    - processor: "reverb"
      params:
        source: "all"
        room_size: 0.2        # Small room
        decay: 0.3s
        damping: 0.7
        mix: 0.15
    - processor: "limiter"
      params:
        ceiling: -0.5
        release: 50ms

  key_parameters:
    polish: 0.7               # 0-1: cleanliness and sheen
    punch: 0.6                # 0-1: transient emphasis
    space: 0.3                # 0-1: room ambience
    texture: 0.4              # 0-1: top-end texture layer

  variation_strategies:
    - change: "pitch"
      range: [-7, 7]
      effect: "Different tuning for different keys"
    - change: "space"
      range: [0.0, 0.6]
      effect: "Dry to roomy"
    - swap: "texture_top"
      alternatives: ["finger snap", "tongue click", "coin drop"]
      effect: "Different top texture"

  failure_signs:
    - "Too noisy or distorted (noise_floor > -40dB)"
    - "Duration too long (> 1s, should be tight)"
    - "Not enough high-end (spectral_centroid < 2000Hz)"
    - "Sounds cheap or harsh"

  export_defaults:
    format: "wav"
    sample_rate: 48000
    bit_depth: 24
    channels: 1
```

---

## 3. Recipe Engine

### Recipe Execution

```python
class RecipeEngine:
    """Executes recipes: generates ingredients, applies DSP, checks quality."""

    def __init__(self, generator: ModelClient, dsp_engine: DSPEngine):
        self.generator = generator
        self.dsp = dsp_engine

    async def execute(self, recipe: Recipe, params: Dict[str, float]) -> AudioBuffer:
        # Phase 1: Generate all source ingredients
        ingredients = {}
        for ingredient in recipe.source_ingredients:
            audio = await self.generator.generate(
                prompt=ingredient.prompt,
                model=ingredient.get("model", "default"),
            )
            ingredients[ingredient.role] = audio

        # Phase 2: Apply processing chain
        current = ingredients  # Dict[str, AudioBuffer]
        final = AudioBuffer.silence()

        for step in recipe.processing_chain:
            processor = self.dsp.get_processor(step.processor)

            # Apply to specified source or all
            source_key = step.params.pop("source", "all")
            if source_key == "all":
                # Mix all ingredients, process together
                if not final:
                    final = self.mix_ingredients(ingredients)
                final = processor.process(final, step.params)
            else:
                # Process individual ingredient
                audio = ingredients[source_key]
                ingredients[source_key] = processor.process(audio, step.params)

        if not final:
            final = self.mix_ingredients(ingredients)

        # Phase 3: Apply parameter overrides
        if params:
            final = self.apply_parameters(final, recipe, params)

        # Phase 4: Quality check
        quality = self.check_quality(final, recipe)
        if not quality.passed:
            return self.repair(final, recipe, quality)

        return final

    def apply_parameters(self, audio: AudioBuffer, recipe: Recipe,
                         params: Dict[str, float]) -> AudioBuffer:
        """Apply user-facing key parameters to the recipe."""
        for param_name, value in params.items():
            if param_name in recipe.key_parameters:
                # Each parameter maps to specific DSP changes
                mapping = self.get_parameter_mapping(recipe.id, param_name)
                audio = self.dsp.process(audio, mapping.processor, {
                    mapping.param: self.map_value(value, mapping.range, mapping.dsp_range)
                })
        return audio

    def check_quality(self, audio: AudioBuffer, recipe: Recipe) -> QualityResult:
        """Check generated sound against recipe targets."""
        analysis = analyze(audio)
        failures = []

        for key, target in recipe.target_characteristics.items():
            actual = getattr(analysis, key, None)
            if actual is None:
                continue
            if isinstance(target, tuple):
                if actual < target[0] or actual > target[1]:
                    failures.append(Failure(key, actual, target))
            elif isinstance(target, bool):
                if target and not actual:
                    failures.append(Failure(key, "missing", "present"))

        return QualityResult(passed=len(failures) == 0, failures=failures)
```

### Recipe Browser

```
┌─────────────────────────────────────────────────────────┐
│  RECIPE BROWSER                                         │
│                                                         │
│  [Search recipes...]                    [By genre ▼]    │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │  ⚡ Cinematic Impact          Level 3 • 2-5 min  │   │
│  │  Epic impact for trailers and drops              │   │
│  │  [intensity: ████████░░] [decay: ██████░░░░]    │   │
│  │  [Generate] [Preview] [Show Steps]               │   │
│  ├─────────────────────────────────────────────────┤   │
│  │  🥁 Glossy Hyperpop Snare    Level 2 • 1-3 min  │   │
│  │  Bright, pitchy snare for pop productions        │   │
│  │  [snap: ████████░░] [air: ███████░░░]           │   │
│  │  [Generate] [Preview] [Show Steps]               │   │
│  ├─────────────────────────────────────────────────┤   │
│  │  🎸 Dark Drill 808 Hit        Level 2 • 1-3 min  │   │
│  │  Heavy sub kick with distorted character         │   │
│  │  [sub: ████████░░] [crunch: ██████░░░░]         │   │
│  │  [Generate] [Preview] [Show Steps]               │   │
│  ├─────────────────────────────────────────────────┤   │
│  │  👏 Crunchy Analog Clap       Level 2 • 1-3 min  │   │
│  │  Warm, vintage drum machine clap                 │   │
│  │  [crunch: ████████░░] [vintage: ██████░░░░]     │   │
│  │  [Generate] [Preview] [Show Steps]               │   │
│  ├─────────────────────────────────────────────────┤   │
│  │  ❄️ Icy Ambient Texture       Level 3 • 3-5 min  │   │
│  │  Cold, shimmering, evolving ambient texture      │   │
│  │  [coldness: ██████░░░░] [decay: ████████░░]     │   │
│  │  [Generate] [Preview] [Show Steps]               │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  [New Recipe...] [My Recipes] [Trending]                │
└─────────────────────────────────────────────────────────┘
```

### Recipe Storage

```yaml
# Recipes stored as YAML in ~/.cshot/recipes/
# Built-in recipes: in app bundle
# User recipes: in home directory
# Community recipes: download from optional cloud

~/.cshot/recipes/
├── builtin/
│   ├── glossy_hyperpop_snare.yaml
│   ├── dark_drill_808_hit.yaml
│   ├── cinematic_impact.yaml
│   ├── crunchy_analog_clap.yaml
│   ├── icy_ambient_texture.yaml
│   ├── jersey_club_bed_squeak.yaml
│   ├── expensive_pop_perc.yaml
│   └── ... (20+ built-in recipes)
│
├── user/
│   ├── my_custom_kick.yaml      # User-created recipe
│   └── my_edit_of_snare.yaml    # Modified built-in recipe
│
└── community/                    # Optional, downloaded
    ├── trap_snare_101.yaml
    └── lofi_kick_kit.yaml
```

---

## 4. Recipe Creation UI

```
┌─────────────────────────────────────────────────────────┐
│  CREATE RECIPE                                          │
│                                                         │
│  Name: ____________________________                     │
│  Description: ____________________  Level: [1-5 ▼]      │
│                                                         │
│  ── SOURCE INGREDIENTS ──                               │
│  [Add Ingredient]                                       │
│                                                         │
│  Ingredient 1:                                          │
│    Role: low_end     Prompt: "deep sub boom, 30Hz..."   │
│    Model: [cshot-generator-v1 ▼]  [► Preview]           │
│                                                         │
│  Ingredient 2:                                          │
│    Role: noise_sweep  Prompt: "white noise swell..."    │
│    Model: [cshot-generator-v1 ▼]  [► Preview]           │
│                                                         │
│  ── PROCESSING CHAIN ──                                  │
│  [Add Processor]                                        │
│                                                         │
│  1. [Time Stretch ▼]                                    │
│     source: [low_end ▼]  stretch_ratio: [1.5]          │
│                                                         │
│  2. [Equalizer ▼]                                       │
│     source: [noise_sweep ▼]  hp: [200]  lp: [8000]    │
│                                                         │
│  3. [Compressor ▼]                                      │
│     source: [all ▼]  ratio: [4.0]  threshold: [-18]   │
│                                                         │
│  ── KEY PARAMETERS ──                                    │
│  [Add Parameter]                                        │
│                                                         │
│  intensity: 0.7  →  maps to compressor ratio + limiter  │
│  decay: 0.6     →  maps to reverb decay + time stretch  │
│                                                         │
│  ── TARGET CHARACTERISTICS ──                            │
│  [Add Criterion]                                        │
│                                                         │
│  duration_ms: 1500-3000  │  loudness_lufs: -14 to -10  │
│  has_sub: true           │  stereo_width: 0.5-1.0      │
│                                                         │
│  [Save Recipe]  [Test Generate]  [Cancel]               │
└─────────────────────────────────────────────────────────┘
```

---

## Summary

1. **Recipe format**: Structured YAML blueprint — ingredients (generation prompts), processing chain (DSP steps), key parameters (user-facing controls), variation strategies, failure signs, export defaults
2. **Recipe library**: 7+ built-in recipes covering hyperpop, drill, cinematic, analog, ambient, Jersey club, and pop — more added over time
3. **Recipe engine**: Executes recipes end-to-end — generates ingredients, applies DSP chain, checks quality, auto-repairs failures, applies user parameter overrides
4. **User-accessible control**: Sliders for key parameters (not 50 knobs — 3-5 high-level controls per recipe)
5. **Variation system**: Built-in strategies for pitch, timing, texture swaps — each recipe designed to produce family of related sounds
6. **Quality gates**: Each recipe has measurable target characteristics and failure signs — auto-check after generation, auto-repair or retry on failure
