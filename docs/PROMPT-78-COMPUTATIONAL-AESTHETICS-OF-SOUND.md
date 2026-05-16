# Prompt 78 — Computational Aesthetics of Sound

Design a research direction around computational aesthetics for one-shots. Why certain drums feel expensive, why some sounds feel amateur, why textures feel nostalgic, why certain transients dominate mixes, why some timbres feel modern. Explore psychoacoustics, cultural conditioning, production trends, mastering aesthetics, spectral balance preferences, and temporal dynamics. Propose measurable aesthetic features, learned aesthetic embeddings, predictive quality models, and trend forecasting systems.

---

## 1. The Aesthetic Question

### Why Some Sounds Feel Expensive

```
Close your eyes. Hear a kick drum.

  "Expensive" kick:
    - Solid, weighty low-end — not boomy, not thin
    - Attack that cuts without harshness
    - Body that fills the frequency slot perfectly
    - Tail that decays naturally, no resonant ringing
    - Sounds like it was recorded in a million-dollar room
    - Takes up exactly the right amount of spectral space
    - Even at low volume, it sounds "big"

  "Cheap" kick:
    - Muddy or thin low-end
    - Attack that's either dull or harsh
    - Body that conflicts with other elements
    - Tail that rings unnaturally or cuts off too fast
    - Sounds like a bedroom recording
    - Spectral masking with other elements
    - Needs volume to sound "big"

  What makes the difference?
    - It's not just frequency response (EQ can fix some of it)
    - It's not just dynamics (compression can fix some of it)
    - It's COMPLEX INTERACTIONS of timbre, transient, texture,
      spectral balance, and cultural conditioning
    - "Expensive" is an EMERGENT PROPERTY, not a single parameter

  The hypothesis of computational aesthetics:
    "Expensive" and "cheap" are learnable patterns in the
    embedding space. They correlate with measurable acoustic
    features, but the pattern is more than the sum of features.
    
    If we can MODEL aesthetic perception, we can:
    - PREDICT which sounds will be perceived as high-quality
    - GENERATE sounds that are inherently "expensive"
    - OPTIMIZE sounds toward aesthetic targets
    - UNDERSTAND the acoustic basis of aesthetic judgment
```

### The Aesthetic Dimensions

```
Proposed aesthetic dimensions for one-shots:

  DIMENSION 1 — PRESTIGE ("Expensive" vs "Cheap")
    Correlates with: production polish, spectral refinement, naturalness
    Acoustic basis: transient precision, harmonic coherence, noise floor
    Cultural basis: association with professional studios, high-end gear

  DIMENSION 2 — MODERNITY ("Modern" vs "Vintage" vs "Timeless")
    Correlates with: current production trends, genre conventions
    Acoustic basis: spectral tilt toward highs (modern) or mids (vintage)
    Cultural basis: era-specific production aesthetics
  
  DIMENSION 3 — NOSTALGIA ("Nostalgic" vs "Contemporary")
    Correlates with: emotional association with past eras
    Acoustic basis: lo-fi characteristics, vinyl noise, tape saturation
    Cultural basis: personal and collective memory of past music

  DIMENSION 4 — POWER ("Dominating" vs "Subtle")
    Correlates with: transient aggression, spectral density, dynamics
    Acoustic basis: peak-to-RMS ratio, low-frequency energy, sharpness
    Cultural basis: genre — trap kicks dominate, lo-fi kicks sit back

  DIMENSION 5 — SOPHISTICATION ("Refined" vs "Raw" vs "Crude")
    Correlates with: production complexity, attention to detail
    Acoustic basis: spectral smoothness, transient cleanliness
    Cultural basis: association with "artistic" vs "commercial" production

  DIMENSION 6 — AUTHENTICITY ("Organic" vs "Synthetic")
    Correlates with: naturalness, human feel, imperfection
    Acoustic basis: micro-variation, non-uniform decay, imperfect timing
    Cultural basis: "real instruments" vs "AI/electronic" bias

  DIMENSION 7 — ATTENTION ("Demanding" vs "Background")
    Correlates with: how much the sound demands listener focus
    Acoustic basis: transient salience, spectral novelty, dynamic contrast
    Cultural basis: "drop" sounds in EDM vs pad textures in ambient
```

---

## 2. Psychoacoustic Foundations

### What Science Knows

```
Psychoacoustic principles relevant to one-shot aesthetics:

  1. SPECTRAL CENTROID AND BRIGHTNESS
     Higher centroid = perceived as brighter, more modern, sometimes cheaper
     Lower centroid = perceived as warmer, darker, sometimes more expensive
     Sweet spot for "expensive" kicks: centroid around 2-4kHz, not 1kHz or 8kHz
     → The "Goldilocks zone" of perceived quality

  2. ATTACK TRANSIENT SHARPNESS
     Attack time < 5ms = perceived as aggressive, punchy, sometimes harsh
     Attack time 5-15ms = perceived as natural, balanced, professional
     Attack time > 15ms = perceived as soft, dull, amateur
     "Expensive" kicks cluster in the 5-12ms attack range

  3. SPECTRAL IRREGULARITY
     Very smooth spectrum = synthetic, processed, sometimes "cheap"
     Moderately irregular = natural, organic, "expensive"
     Very irregular = resonant, uneven, amateur
     The ideal irregularity is like a natural acoustic instrument

  4. TEMPORAL NATURALNESS
     Exponential decay = natural, acoustic, "real"
     Linear decay = synthetic, electronic, "designed"
     Gated/truncated decay = deliberate effect, genre-dependent
     Natural decay is perceived as higher quality in most contexts

  5. NOISE FLOOR AND MICRO-DETAIL
     Completely clean (-96dB noise floor) = sterile, "digital"
     Moderate noise (-60dB, natural room tone) = warm, "alive"
     High noise (-40dB, tape hiss) = vintage, lo-fi
     Unexpected noise (clicks, pops) = amateur, "broken"
     The "ideal" noise floor depends on genre and intention

  6. DYNAMIC RANGE
     Wide dynamic range (15-20dB) = expressive, "live"
     Compressed (< 6dB) = aggressive, "radio-ready"
     The trend in one-shots: modern production favors compressed
     But "expensive" compression preserves transient detail

  7. HARMONIC STRUCTURE
     Even harmonics = warm, "tube-like", expensive association
     Odd harmonics = aggressive, "transistor", cheap association
     Real instruments have specific harmonic ratios
     AI-generated sounds sometimes have unnatural harmonic patterns
```

### The Psychoacoustic Gap

```
What psychoacoustics DOESN'T explain:

  Why does a 2018 trap kick sound "dated" in 2025?
    → Psychoacoustics: the acoustics haven't changed
    → Answer: CULTURAL CONDITIONING, not psychoacoustics

  Why does a $5000 microphone make a kick sound "better"?
    → Psychoacoustics: frequency response differences are subtle
    → Answer: ASSOCIATION (we're conditioned to value expensive gear)

  Why do some "imperfect" sounds feel MORE valuable?
    → Psychoacoustics: imperfection should be "worse"
    → Answer: AUTHENTICITY BIAS — imperfection signals human involvement

  The gap:
    Psychoacoustics explains the LOW-LEVEL perception.
    It does NOT explain HIGH-LEVEL aesthetic judgment.
    
    "Expensive" is not in the spectrum.
    "Expensive" is in the CULTURE.
    
    Computational aesthetics must model BOTH:
    - Psychoacoustic features (measurable from audio)
    - Cultural associations (learned from human judgment)
```

---

## 3. Cultural Conditioning of Sound

### How Production Trends Shape Aesthetics

```
What sounded "good" has changed dramatically over time:

  ┌────────────┬──────────────────────┬────────────────────────────┐
  │ Era        │ "Good" Kick Sound    │ Production Context          │
  ├────────────┼──────────────────────┼────────────────────────────┤
  │ 1980s      │ Gated, boomy,        │ Live drums, gated reverb,  │
  │            │ artificial           │ big snare sound            │
  ├────────────┼──────────────────────┼────────────────────────────┤
  │ 1990s      │ Tight, sample-based, │ MPC, Akai samplers,        │
  │            │ punchy               │ 12-bit grit               │
  ├────────────┼──────────────────────┼────────────────────────────┤
  │ 2000s      │ Overcompressed,      │ Loudness wars, brickwall   │
  │            │ hyper-aggressive      │ limiting                  │
  ├────────────┼──────────────────────┼────────────────────────────┤
  │ 2010s      │ 808 revival,         │ Trap, SoundCloud,          │
  │            │ distorted, sub-heavy │ heavy 808 sub-bass        │
  ├────────────┼──────────────────────┼────────────────────────────┤
  │ 2020s      │ Clean but punchy,    │ AI-assisted mixing,        │
  │            │ wide dynamic range   │ streaming normalization    │
  ├────────────┼──────────────────────┼────────────────────────────┤
  │ 2025+      │ ??                   │ AI-generated, hyper-      │
  │            │                      │ personalized, genre-fluid  │
  └────────────┴──────────────────────┴────────────────────────────┘

  Key insight: "Good" is NOT universal.
  "Good" is ERA-SPECIFIC, GENRE-SPECIFIC, and SCENE-SPECIFIC.

  A 1980s gated kick sounds "cheap" in a 2025 trap mix.
  A 2025 trap kick would have sounded "wrong" in a 1980s mix.

  But within an era and genre, there is CONSENSUS.
  Producers agree on what sounds "expensive" in their context.
  This consensus can be LEARNED and MODELED.
```

### Acoustic Markers of Aesthetic Categories

```
"EXPENSIVE" acoustic markers (cross-era, cross-genre):
  ┌────────────────────────────┬────────────────────────────────┐
  │ Marker                     │ Measurement                    │
  ├────────────────────────────┼────────────────────────────────┤
  │ Clean transient onset      │ Group delay flatness < 0.3    │
  │ Natural spectral rolloff   │ Spectral slope -6dB/octave    │
  │ Controlled low end         │ Sub/low-mid ratio in 0.3-0.7 │
  │ No resonant peaks          │ Max Q of spectral peaks < 10  │
  │ Balanced compression       │ Crest factor 8-12dB           │
  │ Appropriate noise floor    │ Noise at -70 to -50dB         │
  │ Coherent stereo image      │ Mid/side correlation > 0.7    │
  └────────────────────────────┴────────────────────────────────┘

"CHEAP" acoustic markers:
  ┌────────────────────────────┬────────────────────────────────┐
  │ Marker                     │ Measurement                    │
  ├────────────────────────────┼────────────────────────────────┤
  │ Blurry transient           │ Group delay > 2ms variation   │
  │ Harsh high end             │ Spectral centroid > 8kHz      │
  │ Muddy low end              │ Sub/low-mid ratio > 0.8       │
  │ Resonant peaks             │ Spectral peaks with Q > 20    │
  │ Overcompressed             │ Crest factor < 6dB            │
  │ Digital noise              │ Noise at -90dB (clicks, pops) │
  │ Phase issues               │ Mono compatibility < 0.5      │
  └────────────────────────────┴────────────────────────────────┘

"NOSTALGIC" acoustic markers:
  ┌────────────────────────────┬────────────────────────────────┐
  │ Marker                     │ Measurement                    │
  ├────────────────────────────┼────────────────────────────────┤
  │ Moderate noise floor       │ -50 to -40dB hiss/hum         │
  │ Saturated harmonics        │ Even/odd ratio 1.5-2.0        │
  │ Bandwidth limiting         │ Rolloff starting at 12-16kHz  │
  │ Subtle wow/flutter         │ Pitch variation ±0.5%         │
  │ Non-linear transient       │ Slight attack compression     │
  │ Warm distortion            │ THD 1-3%                      │
  └────────────────────────────┴────────────────────────────────┘

"MODERN" acoustic markers:
  ┌────────────────────────────┬────────────────────────────────┐
  │ Marker                     │ Measurement                    │
  ├────────────────────────────┼────────────────────────────────┤
  │ Clean transient            │ Group delay < 1ms variation   │
  │ Extended high end          │ Content up to 22kHz            │
  │ Tight low end              │ Sub/low-mid ratio 0.4-0.6     │
  │ Loud but dynamic           │ Crest factor 8-10dB, -8 LUFS  │
  │ Precise stereo             │ MS correlation 0.8-0.9        │
  │ No distortion artifacts    │ THD < 0.5%                    │
  │ Hyper-clean noise floor    │ Noise at -85dB or lower       │
  └────────────────────────────┴────────────────────────────────┘
```

---

## 4. Measurable Aesthetic Features

### Feature Taxonomy

```
LEVEL 1 — SPECTRAL FEATURES (psychoacoustic, directly measurable):

  Spectral centroid:       Weighted mean frequency of spectrum
  Spectral rolloff:        Frequency below which 85% of energy lies
  Spectral flatness:       How noise-like (1.0) vs tonal (0.0)
  Spectral slope:          Overall tilt of spectrum (dB/octave)
  Spectral flux:           Frame-to-frame spectral change
  Mel-frequency cepstrum:  MFCCs 1-20 (timbre features)
  Spectral contrast:       Peak-to-valley ratio in sub-bands
  Harmonic ratio:          Even/odd harmonic energy distribution
  Sub-bass ratio:          Energy below 60Hz / total energy
  Low-mid ratio:           Energy 100-250Hz / total energy
  Presence ratio:          Energy 2-4kHz / total energy
  Air ratio:               Energy 8-16kHz / total energy
  Spectral Q:              Sharpness of resonant peaks
  Tonalness:               Ratio of tonal vs noise-like energy

LEVEL 2 — TEMPORAL FEATURES (attack/decay structure):

  Attack time:             Time from onset to peak (ms)
  Attack sharpness:        Slope of attack (dB/ms)
  Decay time:              Time from peak to -20dB (ms)
  Decay curve shape:       Exponential vs linear fit error
  Transient-to-tail ratio: Energy in first 50ms vs rest
  Peak-to-RMS ratio:       Crest factor (dB)
  Temporal centroid:       Energy-weighted center of time
  Onset clarity:           RMS rise slope (dB/sec)
  Ring duration:           Time for resonant tail to decay

LEVEL 3 — TEXTURAL FEATURES (micro-structure):

  Noise floor:             Minimum RMS level in quietest 100ms
  Noise character:         White/pink/brown noise fit
  Micro-variation:         Sample-to-sample jitter in periodicity
  Granularity:             Grain size distribution (perceptual)
  Roughness:               Amplitude modulation 20-200Hz
  Fluctuation strength:    Amplitude modulation 0.5-20Hz

LEVEL 4 — PRODUCTION FEATURES (engineered, from signal processing):

  Compression amount:      Dynamic range reduction ratio
  Saturation amount:       THD+noise percentage
  Reverb time:             RT60 estimate
  Reverb pre-delay:        Time before reverb onset
  Stereo width:            Mid/side ratio
  Phase coherence:         Inter-channel correlation
  Loudness:                Integrated LUFS
  Loudness range:          LRA (Loudness Range)
  True peak:               Inter-sample peak level

LEVEL 5 — PERCEPTUAL FEATURES (learned, from human ratings):

  Punchiness model:        Regression from features → punch score
  Warmth model:            Regression → warmth score
  Expensive-ness model:    Regression → prestige score
  Modernity model:         Regression → year/style
  Emotional model:         Regression → valence, arousal
  Quality model:           SoundScore
  Genre alignment:         Classifier → genre probability
  Era alignment:           Classifier → decade probability
  Aesthetic embedding:     Learned embedding → aesthetic space
```

### Feature Computation Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Audio input (one-shot, 44.1kHz)                                   │
│         │                                                           │
│  ┌──────▼──────────────────────────────────────────────────────┐   │
│  │  Feature Extraction (50+ features, all sub-5ms)              │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Spectral features (20): centroid, rolloff, flatness,  │  │   │
│  │  │  slope, flux, MFCC 1-20, contrast, harmonic ratio     │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Temporal features (8): attack, decay, crest factor,   │  │   │
│  │  │  transient ratio, onset clarity, temporal centroid     │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Textural features (4): noise floor, roughness,        │  │   │
│  │  │  micro-variation, granularity                          │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Production features (10): compression, saturation,    │  │   │
│  │  │  reverb time, pre-delay, stereo width, phase, loudness │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼                                                           │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Aesthetic Model (trained on human ratings)                   │   │
│  │                                                               │   │
│  │  Input: 50+ acoustic features                                 │   │
│  │  Model: Gradient-boosted tree + neural net ensemble            │   │
│  │  Output:                                                      │   │
│  │    - Prestige score (0-1): "how expensive does this sound?"   │   │
│  │    - Modernity score (0-1): "how modern vs vintage?"          │   │
│  │    - Power score (0-1): "how dominating vs subtle?"           │   │
│  │    - Nostalgia trigger (0-1): "does this evoke nostalgia?"    │   │
│  │    - Authenticity score (0-1): "organic vs synthetic?"        │   │
│  │    - Aesthetic embedding (128d): placement in aesthetic space │   │
│  └──────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼                                                           │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Quality Prediction                                           │   │
│  │                                                               │   │
│  │  From features + aesthetic scores:                            │   │
│  │    - Predicted SoundScore (0-100)                            │   │
│  │    - Predicted export rate (%)                               │   │
│  │    - Predicted mix-readiness (0-1)                           │   │
│  │    - Predicted genre fit (per genre)                         │   │
│  │    - Predicted user preference (per taste profile)           │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 5. Learned Aesthetic Embeddings

### Aesthetic Space

```
Instead of hand-designing aesthetic features, LEARN the aesthetic
space from human judgments.

  Dataset:
    - 50,000 one-shots
    - Each rated by 5+ listeners on:
      - "How expensive does this sound?" (1-7)
      - "How modern?" (1-7)
      - "How powerful?" (1-7)
      - "How nostalgic?" (1-7)
      - "How authentic?" (1-7)
      - Free text: "What does this sound remind you of?"

  Training:
    - Audio encoder → 1024d UShOt embedding
    - Aesthetic projector → 128d aesthetic embedding
    - Aesthetic decoder → {prestige, modernity, power, nostalgia, authenticity}
    - Train: triplet loss + regression + cross-entropy on text associations

  Aesthetic embedding properties:
    - Similar sounds have similar aesthetic scores
    - Directions encode aesthetic dimensions
    - vec("expensive kick") - vec("cheap kick") = "prestige" direction
    - vec("modern") - vec("vintage") = "modernity" direction
    - Interpolation produces meaningful aesthetic morphs

  Aesthetic space visualization:
    ┌─────────────────────────────────────────────────────────────┐
    │                                                             │
    │    Modern                                                  │
    │      ↑                                                    │
    │      │     ● AI-gen kicks (modern + synthetic)            │
    │      │        ● 2025 trap kicks (modern + expensive)      │
    │      │                                                    │
    │      │    ● 2010 trap kicks (modern-ish + raw)            │
    │      │                                                    │
    │      │                  ● 1990s sampled kicks (vintage)   │
    │      │                                                    │
    │      │        ● 1980s gated kicks (vintage + expensive)   │
    │    ──┼───────────────────────────────→ Prestige           │
    │      │                                                    │
    │      │   ● Bedroom recordings (modern + cheap)            │
    │      │                                                    │
    │      │              ● Amateur demos (cheap + vintage)     │
    │      │                                                    │
    │  ←───┼─────────────── +  ────────────────→               │
    │      │    Organic                            Synthetic    │
    │      │                                                    │
    └─────────────────────────────────────────────────────────────┘
```

### Explaining Aesthetic Judgments

```
A model is not useful if it can't EXPLAIN its judgments.

  "Why does this sound sound expensive?"
  → Feature attribution: which features contributed most?
    1. Transient clarity (+0.32 to score): "clean, precise attack"
    2. Spectral balance (+0.28): "well-controlled low end"
    3. Noise profile (+0.15): "appropriate room tone"
    4. Compression (-0.05): "slightly overcompressed"

  "Why does this sound sound cheap?"
  → Feature attribution:
    1. Resonant peak at 200Hz (-0.41): "boxy, muddy low-mid"
    2. Blurry transient (-0.28): "attack lacks definition"
    3. High noise floor (-0.18): "audible hiss"
    4. Narrow stereo (-0.08): "mono, lacks width"

  Explanation delivery:
    "Your kick sounds cheap because it has a resonant peak at 200Hz
     making it sound boxy, and the attack transient lacks definition.
     Try EQ: reduce 200Hz by 3dB and add 5ms attack pre-delay."

    This transforms aesthetic judgment into ACTIONABLE ADVICE.
    The system doesn't just say "this is bad" — it says
    "here's EXACTLY what to change to make it better."
```

---

## 6. Predictive Quality Models

### Quality Prediction Architecture

```
Input → Feature Extraction → Quality Model → Quality Score + Explanation

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Quality Model — Multi-task architecture:                          │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Task 1 — Absolute Quality (SoundScore)                      │  │
│  │  Regression: 0-100 score                                     │  │
│  │  Training data: 100K sounds with expert quality ratings      │  │
│  │  Test accuracy: ±5 points (current best)                     │  │
│  │  Target: ±3 points                                           │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Task 2 — Export Prediction (will user export this?)         │  │
│  │  Binary classification: export (1) / skip (0)               │  │
│  │  Training data: 500K generation → action pairs              │  │
│  │  Test accuracy: 78% (current)                               │  │
│  │  Target: 85%                                                │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Task 3 — Mix-Readiness (will this sound fit a mix?)         │  │
│  │  Regression: 0-1 readiness score                             │  │
│  │  Training data: 20K sounds tested in mix context             │  │
│  │  Test accuracy: ±0.1                                         │  │
│  │  Target: ±0.05                                               │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Task 4 — Comparison (which of two sounds is better?)        │  │
│  │  Pairwise ranking                                           │  │
│  │  Training data: 50K pairs with human preference             │  │
│  │  Test accuracy: 82%                                         │  │
│  │  Target: 90%                                                │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  Model architecture: XGBoost + Transformer ensemble                │
│  Features: 50+ acoustic + 128 aesthetic embedding                  │
│  Training: 3 days on 8× A100                                       │
│  Inference: < 5ms per sound (CPU)                                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Quality Thresholds by Genre

```
Quality is genre-specific — what's good for trap is wrong for lo-fi.

  ┌──────────────────────┬──────────┬──────────┬──────────────┐
  │ Genre                │ Min Good │ Target   │ Max (clipping)│
  │                      │ SoundScore│ SoundScore│               │
  ├──────────────────────┼──────────┼──────────┼──────────────┤
  │ Trap                 │ 70       │ 80       │ 95            │
  │ Drill                │ 72       │ 82       │ 95            │
  │ House                │ 68       │ 78       │ 92            │
  │ Techno               │ 65       │ 75       │ 90            │
  │ Lo-fi                │ 55       │ 65       │ 85            │
  │ Cinematic            │ 72       │ 82       │ 95            │
  │ Pop                  │ 70       │ 80       │ 93            │
  │ Experimental         │ 50       │ 65       │ 90            │
  │ Ambient              │ 60       │ 70       │ 88            │
  │ Game audio           │ 68       │ 78       │ 92            │
  └──────────────────────┴──────────┴──────────┴──────────────┘

  Lo-fi has a lower floor because noise/distortion is PART OF the genre.
  But even in lo-fi: intentional noise ≠ bad recording.

  Quality model must be GENRE-AWARE.
  A sound that's "too clean" for lo-fi is "perfect" for pop.
  Training: genre label + genre-specific quality head.
```

---

## 7. Trend Forecasting

### Why Trend Forecasting Matters

```
Sample packs are fashion — they have seasons, trends, and saturation.

  Trend-aware pack creation:
    "Drill kits are peaking. Rage kits are rising. Lo-fi is stable."
    → Generate a rage kit NOW, not a drill kit.
    
  If cShot can FORECAST aesthetic trends, it can:
    - Guide creators toward trending styles
    - Anticipate market demand before it peaks
    - Avoid generating packs in saturated niches
    - Position itself as a trend leader, not follower
```

### Trend Data Sources

```
1. INTERNAL DATA
   - Export rates by genre/style over time
   - Search queries trending up/down
   - Successful pack characteristics
   - User taste evolution (what users shift toward)

2. EXTERNAL MARKET DATA
   - Splice/Loopmasters top charts
   - YouTube beat tutorial keywords (Trending)
   - TikTok audio trends (emerging sounds)
   - Producer forum discussions (Gearspace, Reddit)
   - Streaming charts (what hits use for drums)
   - Instagram/Reel audio usage

3. CULTURAL SIGNALS
   - New producer breakthrough sounds
   - Genre emergence (drill → rage → next?)
   - Technology shifts (new synths, new processing)
   - Nostalgia cycles (20-year retro cycle)
   - Social media sound virality

4. PREDICTIVE FEATURES
   - Derivative of trend curves (is it accelerating?)
   - Cross-correlation between genres
   - Novelty score (how different from existing packs)
   - Celebrity/tastemaker adoption rate
   - Production complexity trend (simplifying vs complexifying)
```

### Forecasting Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Data Ingestion                                               │  │
│  │  • Daily scrape: Splice charts, YouTube trends, TikTok audio │  │
│  │  • Continuous: internal export/search/favorite data           │  │
│  │  • Weekly: genre expert interviews (optional)                │  │
│  │  → Features: genre popularity, BPM trends, timbre trends     │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                    │                                                │
│                    ▼                                                │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Trend Model                                                  │  │
│  │                                                               │  │
│  │  Time-series model (LSTM + Transformer):                     │  │
│  │    Input: 24 months of trend data                            │  │
│  │    Output: 6-month forecast for each genre/style dimension    │  │
│  │                                                               │  │
│  │  Forecast dimensions:                                         │  │
│  │    - Genre popularity (next 6 months)                        │  │
│  │    - BPM trend (are beats getting faster/slower?)            │  │
│  │    - Timbre trend (darker or brighter? more grit or clean?)  │  │
│  │    - Production trend (more processed or more raw?)          │  │
│  │    - Transient trend (punchier or softer?)                   │  │
│  │    - 808 trend (more distorted or more clean?)               │  │
│  │    - Nostalgia offset (which era is back in style?)          │  │
│  │                                                               │  │
│  │  Forecast horizon: 3-month (tactical), 6-month (strategic)   │  │
│  │  Update frequency: weekly                                    │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                    │                                                │
│                    ▼                                                │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Output                                                       │  │
│  │                                                               │  │
│  │  "Spring 2026 Sound Trend Report":                           │  │
│  │    - Rage beats peaking (expect saturation by Q3)            │  │
│  │    - Dark cinematic drums rising (+40% YoY)                  │  │
│  │    - Lo-fi declining (-15% YoY)                              │  │
│  │    - House resurgence predicted (late 2026)                  │  │
│  │    - Nostalgia cycle: Y2K sounds returning                   │  │
│  │    - BPM trend: slowing (140→132 average)                   │  │
│  │    - Timbre trend: warmer, more analog                      │  │
│  │    - Transient trend: punchier, shorter attack               │  │
│  │                                                               │  │
│  │  "Recommended pack concepts":                                 │  │
│  │    1. Dark cinematic drum kit (high demand, low supply)      │  │
│  │    2. Warm house drum kit (predicted resurgence)             │  │
│  │    3. Y2K nostalgia pack (nostalgia cycle aligning)          │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 8. Research Experiments

### Experiment 1: Aesthetic Consensus

```
Hypothesis: There is measurable consensus among producers about
what sounds "expensive" vs "cheap" for one-shots.

Method:
  - 100 one-shots (kicks, snares, hats)
  - 50 producers rate each on "expensive-ness" (1-7)
  - Measure inter-rater reliability (ICC)

Expected result:
  - ICC > 0.7 for kicks (strong consensus)
  - ICC > 0.6 for snares (moderate consensus)
  - ICC > 0.5 for hi-hats (weaker — more style-dependent)

If consensus exists → aesthetic modeling IS possible.
If no consensus → aesthetics are purely subjective (unlikely).
```

### Experiment 2: Acoustic Basis of Prestige

```
Hypothesis: "Expensive" correlates with specific acoustic features.

Method:
  - From Experiment 1: divide sounds into high-prestige and low-prestige
  - Compare acoustic features between groups
  - Which features differ significantly?

Expected result:
  - High-prestige kicks: attack 5-12ms, centroid 2-4kHz, crest factor 8-12dB
  - Low-prestige kicks: attack < 3ms or > 20ms, centroid > 6kHz, crest < 6dB
  - Regression: these features explain 60%+ of prestige variance
```

### Experiment 3: Era Decoding

```
Hypothesis: Listeners can accurately identify the era of a one-shot.

Method:
  - One-shots from 1980-2025 (10 per year, 450 total)
  - 30 listeners guess: "What decade is this from?"
  - Measure: era classification accuracy

Expected result:
  - Above-chance for all decades (25% blind → 50%+ actual)
  - Best for extreme decades (80s, 2020s)
  - Worst for transitional periods (late 90s → early 2000s)

If eras are decodable → era-conditioned generation is possible.
"Generate a kick that sounds like 1995" — achievable via embedding.
```

### Experiment 4: Nostalgia Trigger Analysis

```
Hypothesis: Nostalgia in sound is triggered by specific acoustic patterns.

Method:
  - 200 one-shots with known nostalgic association
  - Extract acoustic features
  - Train model: predict nostalgia score from features
  - Interpret: which features drive nostalgia?

Expected result:
  - Top predictors: noise floor (-50dB), moderate THD (1-3%), bandwidth < 16kHz
  - Nostalgia is NOT about frequency content — it's about TEXTURE
  - "Vintage" = lo-fi texture, not specific spectral shape
```

---

## 9. Implementation Roadmap

```
Phase 1 — Aesthetic Measurement (2 months):
  ✓ Implement 50+ acoustic feature extractors
  ✓ Train aesthetic regression models (prestige, modernity, power)
  ✓ Build feature attribution (SHAP values for explanation)
  ✓ Validation: correlation with human ratings > 0.7

Phase 2 — Aesthetic Embedding (1 month):
  ✓ Collect human aesthetic ratings (50K sounds, 5+ raters each)
  ✓ Train aesthetic embedding (128d in UShOt space)
  ✓ Build aesthetic space visualization
  ✓ Find directions: "prestige", "modernity", "nostalgia"

Phase 3 — Quality Prediction (2 months):
  ✓ Train multi-task quality model
  ✓ Integrate with generation pipeline (SoundScore v2)
  ✓ Genre-aware quality thresholds
  ✓ Explainable quality assessment

Phase 4 — Trend Forecasting (2 months):
  ✓ Build trend data pipeline (internal + external data)
  ✓ Train trend forecasting model (6-month horizon)
  ✓ Weekly trend report generation
  ✓ Integration with pack builder (trend-aware recommendations)

Total timeline: ~7 months for full computational aesthetics system
```

---

## 10. Summary

```
Computational Aesthetics of Sound

  Core thesis:
    Aesthetic judgments about sound ("expensive", "cheap", "modern",
    "nostalgic") are NOT purely subjective. They correlate with
    measurable acoustic features and follow learnable cultural patterns.

  What we can measure:
    - 50+ acoustic features (spectral, temporal, textural, production)
    - 5 primary aesthetic dimensions (prestige, modernity, power,
      nostalgia, authenticity)
    - 128d aesthetic embedding in UShOt space
    - Genre-aware quality thresholds

  What we can predict:
    - How "expensive" a sound will be perceived
    - Which era a sound belongs to
    - Whether a user will export or skip
    - Whether a sound is mix-ready
    - What genre a sound best fits
    - How a sound's aesthetic will trend over time

  What we can generate:
    - Sounds optimized for specific aesthetic targets
    - Packs aligned with predicted trends
    - Sounds with explainable aesthetic improvement paths

  The deeper question:
    Aesthetics are not in the waveform. They're in the CULTURE.
    But culture leaves traces in acoustic patterns.
    Computational aesthetics is about finding those traces
    and learning to navigate them.

    "Expensive" is not a frequency. It's a pattern of patterns.
    Once modeled, it becomes a dimension we can optimize for.
```

