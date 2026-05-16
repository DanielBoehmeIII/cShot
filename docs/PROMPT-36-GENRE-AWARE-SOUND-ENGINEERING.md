# Prompt 36 — Genre-Aware Sound Engineering

cShot should know what a "trap kick" sounds like vs a "house kick" — and generate accordingly.

---

## 1. Genre Family Definitions

### Genre Map

```
Electronic Dance:
  ┌─ House (deep, tech, progressive, electro, future)
  ├─ Techno (minimal, industrial, acid, hard, melodic)
  ├─ Trance (uplifting, progressive, psy)
  ├─ Dubstep (brostep, deep, riddim, melodic)
  ├─ Drum & Bass (liquid, neurofunk, jump-up, jungle)
  ├─ Garage (UKG, future garage, 2-step)
  ├─ Breakbeat (nu-skool, big beat)
  └─ Hardcore (hardstyle, gabber, frenchcore)

Hip-Hop & R&B:
  ┌─ Trap (atlanta, melodic, drill, plugg, rage)
  ├─ Boom Bap (classic, lo-fi, jazz rap)
  ├─ R&B (contemporary, alternative, neo-soul)
  ├─ Pop Rap (melodic, mainstream)
  └─ Experimental (abstract, glitch, cloud)

Pop:
  ┌─ Mainstream Pop (dance-pop, synth-pop, electropop)
  ├─ Indie Pop (bedroom pop, dream pop, art pop)
  ├─ Hyperpop (digicore, glitchcore, scenecore)
  └─ K-Pop (industrial pop structure)

Ambient & Experimental:
  ┌─ Ambient (dark ambient, drone, space)
  ├─ IDM (braindance, glitch, microsound)
  ├─ Sound Collage (plunderphonics, found sound)
  └─ Noise (power electronics, harsh noise, wall)

Cinematic & Game Audio:
  ┌─ Orchestral (epic, hybrid, chamber)
  ├─ Game Audio (adaptive, retro, chiptune)
  ├─ Sound Design (foley, UI, transitions)
  └─ Trailer Music (epic, suspense, action)

Global:
  ┌─ Afrobeat (afrobeats, amapiano, gqom)
  ├─ Latin (reggaeton, dancehall, baile funk)
  ├─ Asian (J-pop, C-pop, Bollywood)
  └─ Folk / Traditional (varies by region)
```

---

## 2. Genre Sound Profiles

### Trap

```
BPM Range: 130-180 (typically 140-160)
Common Sounds: 808 kick, crisp snare, layered hi-hats, claps, percs

KICK:
  ┌─────────────────────────────────────────────┐
  │ Character: Punchy, short decay, subby       │
  │ Transient: Fast attack (<3ms), clicky       │
  │                                              │
  │ Best Practice:                               │
  │  - Fundamental: 50-80Hz                     │
  │  - Click: 2-5kHz spike                      │
  │  - Decay: 100-300ms (short, tight)          │
  │  - Body: Minimal, mostly sub + click        │
  │  - Typical Tail: Clean cutoff, no reverb    │
  │                                              │
  │ Production Notes:                            │
  │  - Often layered (sub + click + noise)       │
  │  - Sidechained to 808 in arrangement        │
  │  - Mono (always)                            │
  │  - Loudness: ~-6 to -10dBFS peak            │
  └─────────────────────────────────────────────┘

SNARE:
  ┌─────────────────────────────────────────────┐
  │ Character: Crisp, resonant, mid-forward     │
  │ Transient: Fast attack, bright              │
  │                                              │
  │ Best Practice:                               │
  │  - Body: 200-400Hz resonance                │
  │  - Crack: 5-10kHz                           │
  │  - Decay: 150-300ms                         │
  │  - Tuning: Often tuned to track key         │
  │  - Reverb: Short room or none               │
  │                                              │
  │ Production Notes:                            │
  │  - Often layered with clap                  │
  │  - Can be pitched up/down for variation     │
  │  - Mono (center-panned)                     │
  └─────────────────────────────────────────────┘

HI-HAT:
  ┌─────────────────────────────────────────────┐
  │ Character: Bright, fast, rhythmic           │
  │                                              │
  │ Patterns:                                    │
  │  - Trap: rapid 1/16 or 1/32 notes           │
  │  - Often with velocity variation            │
  │  - Rolls and triplet feels common           │
  │                                              │
  │ Best Practice:                               │
  │  - Attack: <1ms                             │
  │  - Decay: 50-100ms                          │
  │  - Brightness: 8-15kHz presence             │
  │  - Panning: alternating L/R per hit         │
  └─────────────────────────────────────────────┘

808/BASS:
  ┌─────────────────────────────────────────────┐
  │ Character: Deep sub, long decay, pitch      │
  │              glide                          │
  │                                              │
  │ Best Practice:                               │
  │  - Fundamental: 30-80Hz (tuned to key)      │
  │  - Distortion: subtle (for harmonic         │
  │    presence on small speakers)              │
  │  - Decay: 400-800ms or whole bar            │
  │  - Portamento: 50-200ms glide               │
  │  - Mono (essential)                         │
  └─────────────────────────────────────────────┘
```

### Drill

```
BPM Range: 140-175 (typically 150-165)

KICK:
  ┌─────────────────────────────────────────────┐
  │ Character: Short, punchy, aggressive        │
  │  - Fundamental: 60-80Hz                     │
  │  - Click: 3-5kHz (emphasized)              │
  │  - Decay: Very short, 50-150ms             │
  │  - Often uses 808 kick pattern              │
  │  - Typically sparser than trap              │
  └─────────────────────────────────────────────┘

SNARE/CLAP:
  ┌─────────────────────────────────────────────┐
  │ Character: Layered, aggressive crack        │
  │  - Multiple layers: snare + clap + noise    │
  │  - Heavy reverb (short gated)               │
  │  - 200Hz body, 5-10kHz snap                 │
  │  - Often pitched up for tension             │
  └─────────────────────────────────────────────┘

Pattern Characteristics:
  - Syncopated, swung feel
  - Kicks on 1 and 3 (or half-time feel)
  - Snares on 3 (half-time) or 2 and 4
  - Hi-hats: 1/16 or 1/32 with heavy variation
  - Slides and syncopation in 808
```

### Jersey Club

```
BPM Range: 130-160 (typically 140-150)

KICK:
  ┌─────────────────────────────────────────────┐
  │ Character: Heavy, punchy, distorted         │
  │  - Fundamental: 60-100Hz                    │
  │  - Heavy saturation                         │
  │  - Short decay: 100-200ms                   │
  │  - Often uses 808 samples                   │
  │  - Kick patterns are the main rhythmic       │
  │    driver                                   │
  └─────────────────────────────────────────────┘

Unique Characteristics:
  - Sample-heavy, vocal chops
  - "Jersey club beat" — triplet-based kick patterns
  - Bouncy, danceable feel
  - Layered percussion from multiple sources
  - Heavy use of sidechain compression
  - Kicks often played in rapid succession (16th notes)
```

### House

```
BPM Range: 115-135 (typically 120-130)

KICK:
  ┌─────────────────────────────────────────────┐
  │ Character: Round, warm, four-on-the-floor   │
  │ Transient: Moderate attack (5-10ms)         │
  │                                              │
  │ Best Practice:                               │
  │  - Fundamental: 40-60Hz (deep house),       │
  │                 60-80Hz (tech house)         │
  │  - Body: 150-300Hz warmth                   │
  │  - Click: minimal instead softened          │
  │  - Decay: 200-400ms                         │
  │  - Envelope: round, not aggressive          │
  │                                              │
  │ Production Notes:                            │
  │  - Always on every beat (4/4)               │
  │  - Gentle compression (2:1-4:1)             │
  │  - Subtle sidechain from bass               │
  │  - Mono (essential)                         │
  │  - Reverb: short plate or room (10-15% wet) │
  └─────────────────────────────────────────────┘

CLAP/SHAPER:
  ┌─────────────────────────────────────────────┐
  │ Character: Crisp, layered, on 2 and 4       │
  │                                              │
  │ Best Practice:                               │
  │  - SNR body: 150-500Hz                      │
  │  - Transient: crisp 3-8kHz                  │
  │  - Layered: clap + snare + noise            │
  │  - Reverb: plate or room (20-30%)           │
  │  - Decay: 200-500ms                         │
  │  - Panning: centered or slight variation    │
  └─────────────────────────────────────────────┘

HI-HAT:
  ┌─────────────────────────────────────────────┐
  │ Character: Shuffled, groovy, off-beat       │
  │                                              │
  │ Patterns:                                    │
  │  - Open hats on off-beats (8ths)            │
  │  - Closed hats for shuffle feel             │
  │  - Shuffle/swing: 55-65%                    │
  │                                              │
  │ Best Practice:                               │
  │  - Open hat: longer decay (200-400ms)       │
  │  - Closed hat: short (50-100ms)             │
  │  - Subtle saturation for warmth             │
  └─────────────────────────────────────────────┘
```

### Techno

```
BPM Range: 120-150 (typically 125-140)

KICK:
  ┌─────────────────────────────────────────────┐
  │ Character: Driving, relentless, industrial  │
  │ Transient: Sharp, aggressive                │
  │                                              │
  │ Best Practice:                               │
  │  - Fundamental: 45-65Hz                     │
  │  - Distortion: moderate-heavy               │
  │  - Decay: 200-600ms (longer than house)     │
  │  - Texture: gritty, lo-fi, overdriven       │
  │  - Envelope: slow attack for pumping feel   │
  │                                              │
  │ Production Notes:                            │
  │  - Always on every beat (4/4)               │
  │  - Heavy processing: saturation, EQ,        │
  │    sometimes reverb                         │
  │  - The kick is THE most important element   │
  │  - Often runs through external effects      │
  │  - Sidechain everything to kick             │
  └─────────────────────────────────────────────┘

HAT:
  ┌─────────────────────────────────────────────┐
  │ Character: Industrial, metallic             │
  │                                              │
  │  - Often distorted                          │
  │  - Off-beat pattern                         │
  │  - Longer decay for atmosphere              │
  │  - Heavy use of reverb on hats              │
  │  - Experimental panning and effects         │
  └─────────────────────────────────────────────┘

Unique Characteristics:
  - Minimal elements, maximum impact
  - Heavy use of reverb and delay as texture
  - Percussion often processed beyond recognition
  - Kicks frequently have long, distorted tails
  - Industrial, mechanical feel
```

### Hyperpop

```
BPM Range: 140-200 (often 160-180)

KICK:
  ┌─────────────────────────────────────────────┐
  │ Character: Extreme, compressed, distorted   │
  │  - Maximum loudness (-6 to -3 LUFS)        │
  │  - Heavy distortion/clipping                │
  │  - Often layered with 808-style sub         │
  │  - Short aggressive decay                  │
  │  - Sometimes pitched unnaturally            │
  └─────────────────────────────────────────────┘

Unique Characteristics:
  - Dense, chaotic production
  - Extreme compression (everything loud)
  - Wide stereo field
  - Heavy saturation and bitcrushing
  - Cartoonish or exaggerated samples
  - Glitch effects, stutters, pops
  - Genre-blending (pop + electronic + experimental)
  - Unconventional sound design prioritized
```

### Ambient

```
BPM Range: No fixed tempo (60-90 if present)

KICK (when present):
  ┌─────────────────────────────────────────────┐
  │ Character: Soft, distant, atmospheric       │
  │  - Low volume, more felt than heard         │
  │  - Long decay with reverb                   │
  │  - No click, no aggressive transient        │
  │  - Heavy processing (reverb, delay)         │
  │  - Often sidechained subtly                 │
  └─────────────────────────────────────────────┘

Unique Characteristics:
  - Texture over rhythm
  - Long, evolving sounds
  - Minimal percussion
  - Heavy reverb and delay
  - Field recordings and found sounds
  - No compression or minimal
  - Wide stereo field
  - Dynamic range preserved
```

### Cinematic

```
BPM Range: Variable (60-120, often rubato)

PERCUSSION:
  ┌─────────────────────────────────────────────┐
  │ Character: Epic, huge, impactful            │
  │                                              │
  │ Best Practice:                               │
  │  - Massive taiko drums, orchestral hits     │
  │  - Heavy reverb (cathedral, hall)           │
  │  - Wide stereo (often >100%)                │
  │  - Loudness: dynamic (-20 to -10 LUFS)      │
  │  - Hybrid electronic + orchestral           │
  │                                              │
  │ Sound Design Elements:                       │
  │  - Impacts, hits, whooshes, rises           │
  │  - Braams, orchestral stabs                │
  │  - Sub hits, trailer kicks                  │
  │  - Scoring percussion                       │
  └─────────────────────────────────────────────┘

Unique Characteristics:
  - Genre-specific: "epic," "suspense," "action"
  - Called "impacts" or "hits," not just kicks/snares
  - Massive dynamic range
  - Surround sound mixes (5.1, 7.1, Atmos)
  - Tempo-mapped but tempo-free sections
  - Every sound is a "moment"
```

### Game Audio

```
Interactive Audio Design:

PERCUSSION BY TYPE:
  ┌─────────────────────────────────────────────┐
  │ Character: Functional, mix-respecting        │
  │                                              │
  │ Best Practice:                               │
  │  - Moderate loudness (-18 LUFS target)      │
  │  - Headroom for runtime mixing              │
  │  - Mono-compatible (always)                 │
  │  - Multiple variations (3-5 per sound)      │
  │  - Loop points included in metadata         │
  │                                              │
  │ Sound Types:                                 │
  │  - UI/hover/click sounds                    │
  │  - Footsteps (multiple surfaces)            │
  │  - Impacts (light, medium, heavy)           │
  │  - Weapon sounds (guns, melee, magic)       │
  │  - Ambient one-shots                        │
  │  - Vocalizations (grunts, shouts)           │
  └─────────────────────────────────────────────┘

Unique Requirements:
  - Must not fatigue on repetition
  - Designed for procedural playback
  - Context-adaptive (reverb zone, material)
  - Streaming-ready (multiple quality LODs)
  - Game engine middleware compatibility (FMOD, Wwise)
```

### Pop

```
BPM Range: 70-160 (typically 90-130)

KICK:
  ┌─────────────────────────────────────────────┐
  │ Character: Clean, polished, precise         │
  │  - Well-defined transient (moderate click)  │
  │  - Warm body (150-250Hz)                    │
  │  - Sub presence (50-70Hz)                   │
  │  - Controlled decay (200-400ms)             │
  │  - Clean, minimal processing                │
  └─────────────────────────────────────────────┘

Unique Characteristics:
  - Polished, commercial sound
  - Consistent across playback systems
  - Tight, clean transient design
  - Structured arrangement hooks
  - Vocal-forward mixing (percussion sits under)
```

### R&B

```
BPM Range: 60-100 (typically 70-90)

KICK:
  ┌─────────────────────────────────────────────┐
  │ Character: Warm, soft, groove-focused       │
  │  - Rounded attack (no sharp click)          │
  │  - Subby but not aggressive                 │
  │  - Long decay for warmth                    │
  │  - Sits behind vocals                        │
  │  - Often uses 808 or live kick samples      │
  └─────────────────────────────────────────────┘

Unique Characteristics:
  - Laid-back, groovy feel
  - Pocket-based drumming
  - Soft transient design
  - Warm saturation
  - Live instrument feel
  - Vocals are priority, percussion supports
```

### Experimental

```
No fixed rules. But common approaches:

  ┌─────────────────────────────────────────────┐
  │ Character: Unconventional, boundary-pushing │
  │                                              │
  │ Common Techniques:                           │
  │  - Heavy processing beyond recognition      │
  │  - Found sounds and field recordings        │
  │  - Extreme stereo manipulation              │
  │  - Granular synthesis textures              │
  │  - Non-standard tunings                      │
  │  - Algorithmic generation                   │
  │  - Destroy and rebuild approach             │
  └─────────────────────────────────────────────┘
```

---

## 3. Transient Expectations by Genre

```
Genre          Attack    Body    Decay    Click    Sub     Character
─────────────────────────────────────────────────────────────────────
Trap           Fast      Minimal Short   Pronounced Yes    Aggressive
Drill          Fast      Minimal V. Short Pronounced Yes    Aggressive
Jersey Club    Fast      Medium  Short   Moderate   Yes    Bouncy
House          Moderate  Medium  Moderate Minimal    Yes    Warm
Techno         Sharp     Heavy   Long    Moderate   Yes    Driving
Hard Techno    Very Fast Heavy   Long    Aggressive Yes    Industrial
Hyperpop       Extreme   Varied  Short   Extreme    Yes    Chaotic
Ambient        Slow      None    Long    None       Maybe  Atmospheric
Cinematic      Varied    Massive Long    Varied     Yes    Epic
Game Audio     Moderate  Medium  Short   Minimal    Yes    Functional
Pop            Moderate  Medium  Moderate Moderate   Yes    Polished
R&B            Soft      Medium  Long    Minimal    Yes    Warm
Dubstep        Fast      Heavy   Short   Pronounced Yes    Aggressive
Drum & Bass    Fast      Medium  Short   Moderate   Yes    Energetic
Boom Bap       Moderate  Warm    Short   Minimal    Yes    Gritty
Lo-fi          Soft      Warm    Medium  None       Maybe  Vintage
```

---

## 4. Loudness Norms

```
Genre           Integrated LUFS    Peak dBFS    Dynamic Range
────────────────────────────────────────────────────────────────
Trap            -8 to -11         -0.5 to -3    3-6 dB
Drill           -8 to -11         -0.5 to -3    3-5 dB
Jersey Club     -9 to -12         -1 to -4      4-7 dB
House           -10 to -14        -1 to -4      5-8 dB
Techno          -10 to -14        -1 to -4      5-8 dB
Hyperpop        -6 to -10         -0.5 to -2    2-4 dB
Ambient         -18 to -24        -3 to -8      10-20 dB
Cinematic       -14 to -20        -2 to -6      8-16 dB
Game Audio      -18 to -24        -3 to -8      10-18 dB
Pop             -10 to -14        -1 to -3      4-7 dB
R&B             -10 to -14        -1 to -4      5-9 dB
Boom Bap        -10 to -14        -1 to -4      5-9 dB
Lo-fi           -12 to -16        -2 to -5      6-10 dB
Drum & Bass     -8 to -12         -1 to -3      3-6 dB
Dubstep         -7 to -10         -0.5 to -2    2-5 dB
```

---

## 5. Saturation Style

```
Genre           Saturation Type    Amount    Character
───────────────────────────────────────────────────────────────
Trap            Soft clip          Moderate  Clean aggression
Drill           Soft clip          Moderate  Aggressive
Jersey Club     Tape + clip        Heavy     Warm + distorted
House           Tape               Light     Warm glue
Techno          Overdrive          Heavy     Industrial grit
Hyperpop        Digital clip       Extreme   Destroyed
Ambient         None or tape       None-Lt   Clean, natural
Cinematic       Tape               Light     Warmth without grit
Game Audio      None               None      Clean, functional
Pop             Tape + tube        Light     Polished warmth
R&B             Tape               Light-Mod Warm, smooth
Boom Bap        Analog console     Moderate  Gritty character
Lo-fi           Tape + vinyl       Heavy     Vintage character
Dubstep         Hard clip          Heavy     Aggressive
```

---

## 6. Stereo Behavior

```
Genre           Width       Center    Side    Notes
───────────────────────────────────────────────────────────────
Trap            0-20%       Kick, Snare  Hats    Mostly mono
Drill           0-20%       Kick, Snare  Hats    Mostly mono
Jersey Club     20-40%      Kick, Snare  Perc    Bouncy width
House           30-50%      Kick, Snare  Hats, Perc Groovy width
Techno          40-70%      Kick         FX, Hats Industrial width
Hyperpop        60-100%     Kick         Everything Maximum width
Ambient         80-120%     Nothing      Everything Immersive
Cinematic       80-120%     Impacts      Layers   Epic width
Game Audio      0-40%       Everything   Ambience Context-dependent
Pop             30-50%      Kick, Snare  Hats     Balanced
R&B             20-40%      Kick, Snare  Hats     Warm center
Boom Bap        10-30%      Kick, Snare  Hats     Vintage center
Lo-fi           20-40%      Kick, Snare  Hats     Vintage width
```

---

## 7. Reverb/Delay Use

```
Genre           Reverb Type    Decay    Mix     Delay?
──────────────────────────────────────────────────────────────
Trap            Room           <0.5s    10-15%  No
Drill           Gated room     0.3-0.8s 15-25%  Sometimes
Jersey Club     Room/plate     0.5-1.0s 20-30%  Sometimes
House           Plate/room     0.5-1.5s 15-25%  Occasionally
Techno          Hall/industrial 1-4s    20-50%  Yes (ping-pong)
Hyperpop        Shimmer/hall   1-3s     30-60%  Yes (extreme)
Ambient         Cathedral/hall 3-10s   40-80%  Yes (long)
Cinematic       Cathedral      2-6s     30-60%  Yes (massive)
Game Audio      Context-adapt  0.5-3s   10-40%  Context-dependent
Pop             Plate/room     0.5-1.5s 15-25%  Occasionally
R&B             Plate/spring   0.5-2.0s 15-30%  Sometimes
Boom Bap        Room/spring    0.3-1.0s 10-20%  Occasionally
Lo-fi           Vinyl/spring   0.5-1.5s 20-40%  Often
```

---

## 8. Mix Placement

```
Genre           Kick    Snare   Hats    Clap    Perc    Bass
────────────────────────────────────────────────────────────────
Trap            Center  Center  L/R alt Center  L/R var Center
Drill           Center  Center  L/R alt Center  L/R var Center
Jersey Club     Center  Center  L/R     Center  L/R     Center
House           Center  Center  L/R     Center  L/R     Center
Techno          Center  Center  L/R     Center  Wide    Center
Hyperpop        Center  Center  Wide    Center  Wide    Center
Ambient         Center  N/A     N/A     N/A     Wide    Diffuse
Cinematic       Center  L/R     N/A     Center  Wide    Center
Game Audio      Center  Var     Var     Center  Var     Center
Pop             Center  Center  L/R     Center  L/R     Center
R&B             Center  Center  L/R     Center  L/R     Center
Boom Bap        Center  Center  L/R     Center  Center  Center
Lo-fi           Center  Center  L/R     Center  Center  Center
```

---

## 9. How cShot Adapts to Genre

### Genre Adaptation Engine

```python
class GenreAdaptationEngine:
    """
    Automatically adapts generated sounds to genre context.
    Called after generation, before mix-readiness pipeline.
    """
    
    def __init__(self, genre, bpm=None, key=None):
        self.genre = genre
        self.bpm = bpm
        self.key = key
        self.profile = GENRE_PROFILES.get(genre, GENRE_PROFILES['default'])
    
    def adapt(self, sound, sound_type):
        """
        Adapt a generated sound to genre expectations.
        sound: raw audio array
        sound_type: 'kick', 'snare', 'hat', 'clap', 'perc', 'bass'
        """
        pipeline = self.profile.get(sound_type, {})
        
        audio = sound.copy()
        
        # 1. Transient shaping
        if 'transient' in pipeline:
            audio = self.shape_transient(audio, pipeline['transient'])
        
        # 2. EQ
        if 'eq' in pipeline:
            audio = self.apply_eq(audio, pipeline['eq'])
        
        # 3. Saturation
        if 'saturation' in pipeline:
            audio = self.apply_saturation(audio, pipeline['saturation'])
        
        # 4. Decay adjustment (respect BPM)
        if 'decay' in pipeline:
            audio = self.adjust_decay(audio, pipeline['decay'])
        
        # 5. Stereo placement
        if 'stereo' in pipeline:
            audio = self.apply_stereo(audio, pipeline['stereo'])
        
        # 6. Reverb
        if 'reverb' in pipeline:
            audio = self.apply_reverb(audio, pipeline['reverb'])
        
        # 7. Loudness
        if 'loudness' in pipeline:
            target = pipeline['loudness']
            audio = self.match_loudness(audio, target)
        
        return audio
    
    def shape_transient(self, audio, config):
        """Shape transient to genre expectations."""
        attack_ms = config.get('attack_ms', 5)
        boost_db = config.get('boost_db', 0)
        shape = config.get('shape', 'moderate')
        
        # Convert to samples
        sr = 44100
        attack_samples = int(attack_ms * sr / 1000)
        
        # Adjust transient
        transient = audio[:attack_samples]
        gain = 10 ** (boost_db / 20)
        transient *= gain
        
        # Shape envelope
        if shape == 'aggressive':
            envelope = np.linspace(1.0, 0.3, attack_samples)
        elif shape == 'soft':
            envelope = np.linspace(0.7, 1.0, attack_samples)  # Slow rise
        else:  # moderate
            envelope = np.linspace(1.0, 0.6, attack_samples)
        
        audio[:attack_samples] = transient * envelope
        return audio
    
    def adjust_decay(self, audio, config):
        """Adjust decay length based on BPM and genre."""
        bpm = self.bpm or 120
        beat_ms = 60000 / bpm
        
        # Target decay in beats
        decay_beats = config.get('decay_beats', 0.5)
        target_decay_ms = decay_beats * beat_ms
        
        # Find current decay
        envelope = compute_envelope(np.abs(audio))
        current_decay = find_decay_time(envelope)  # in samples
        
        # If current decay > target, apply envelope
        if current_decay > target_decay_ms * 44.1:
            # Apply exponential decay from current point
            decay_start = int(target_decay_ms * 44.1)
            if decay_start < len(audio):
                remaining = audio[decay_start:]
                decay_curve = np.exp(-np.linspace(0, 4, len(remaining)))
                audio[decay_start:] = remaining * decay_curve
        
        return audio
    
    def apply_reverb(self, audio, config):
        """Apply genre-appropriate reverb."""
        reverb_type = config.get('type', 'room')
        decay = config.get('decay', 0.5)
        mix = config.get('mix', 0.15)
        
        # Simplified: use convolution with appropriate IR
        ir = get_ir_for_type(reverb_type, decay)
        wet = convolve(audio, ir)
        wet = wet[:len(audio)]  # Trim to match length
        
        # Mix dry/wet
        output = (1 - mix) * audio + mix * wet
        return output
    
    def apply_stereo(self, audio, config):
        """Apply genre-appropriate stereo placement."""
        width = config.get('width', 0.0)
        pan = config.get('pan', 'center')
        
        if audio.ndim < 2 or audio.shape[1] < 2:
            audio = np.column_stack([audio, audio])
        
        if width == 0:
            # Mono: average channels
            mono = (audio[:, 0] + audio[:, 1]) / 2
            audio[:, 0] = mono
            audio[:, 1] = mono
        else:
            # Apply width
            mid = (audio[:, 0] + audio[:, 1]) / 2
            side = (audio[:, 0] - audio[:, 1]) / 2
            side *= min(1.0, width)
            audio[:, 0] = mid + side
            audio[:, 1] = mid - side
        
        return audio
```

### Genre Profiles Data Structure

```python
GENRE_PROFILES = {
    'trap': {
        'kick': {
            'transient': {'attack_ms': 2, 'boost_db': 3, 'shape': 'aggressive'},
            'eq': [
                {'type': 'peak', 'freq': 60, 'gain': 2, 'q': 1.0},
                {'type': 'peak', 'freq': 3000, 'gain': 3, 'q': 2.0},
                {'type': 'high_pass', 'freq': 25, 'slope': 24},
            ],
            'saturation': {'type': 'soft_clip', 'amount': 1.5},
            'decay': {'decay_beats': 0.25},
            'stereo': {'width': 0.0, 'pan': 'center'},
            'reverb': None,
            'loudness': -10,
        },
        'snare': {
            'transient': {'attack_ms': 1, 'boost_db': 2, 'shape': 'aggressive'},
            'eq': [
                {'type': 'peak', 'freq': 200, 'gain': 2, 'q': 1.5},
                {'type': 'peak', 'freq': 7000, 'gain': 2, 'q': 2.0},
            ],
            'saturation': {'type': 'soft_clip', 'amount': 1.2},
            'decay': {'decay_beats': 0.5},
            'stereo': {'width': 0.0, 'pan': 'center'},
            'reverb': {'type': 'room', 'decay': 0.3, 'mix': 0.1},
            'loudness': -10,
        },
        'hat': {
            'transient': {'attack_ms': 0.5, 'boost_db': 1, 'shape': 'aggressive'},
            'eq': [
                {'type': 'high_pass', 'freq': 300, 'slope': 12},
                {'type': 'peak', 'freq': 10000, 'gain': 2, 'q': 2.0},
            ],
            'decay': {'decay_beats': 0.125},
            'stereo': {'width': 0.6, 'pan': 'alternating'},
            'loudness': -12,
        },
    },
    'house': {
        'kick': {
            'transient': {'attack_ms': 5, 'boost_db': 1, 'shape': 'moderate'},
            'eq': [
                {'type': 'peak', 'freq': 50, 'gain': 1.5, 'q': 1.0},
                {'type': 'peak', 'freq': 200, 'gain': 1, 'q': 1.5},
            ],
            'saturation': {'type': 'tape', 'amount': 0.5},
            'decay': {'decay_beats': 0.5},
            'stereo': {'width': 0.0, 'pan': 'center'},
            'reverb': {'type': 'plate', 'decay': 0.3, 'mix': 0.1},
            'loudness': -12,
        },
        # ... more sound types
    },
    # ... more genres
}
```

### Genre Auto-Detection

```python
def detect_genre(context):
    """
    Detect genre from available context.
    Falls back through multiple signals.
    """
    if context.get('genre'):
        return context['genre']
    
    if context.get('bpm'):
        # Guess from BPM range
        bpm = context['bpm']
        if 130 <= bpm <= 180:
            return 'trap'  # Conservative default for higher BPMs
        elif 115 <= bpm <= 135:
            return 'house'
        elif 60 <= bpm <= 100:
            return 'rnb'
        elif 140 <= bpm <= 200:
            return 'hyperpop'
    
    if context.get('track_name'):
        # Keyword matching
        name_lower = context['track_name'].lower()
        genre_keywords = {
            'trap': ['trap', '808', 'drill'],
            'house': ['house', 'deep', 'tech'],
            'techno': ['techno', 'industrial', 'minimal'],
            'ambient': ['ambient', 'drone', 'atmospheric'],
            'cinematic': ['cinematic', 'epic', 'orchestral', 'trailer'],
        }
        for genre, keywords in genre_keywords.items():
            if any(k in name_lower for k in keywords):
                return genre
    
    return 'default'
```

---

## Summary

| Genre | Kick Character | Decay | Click | Saturation | Width |
|-------|---------------|-------|-------|-----------|-------|
| Trap | Aggressive punch | Short | Strong | Clip | Mono |
| Drill | Sharp stab | V.Short | Strong | Clip | Mono |
| Jersey Club | Heavy bounce | Short | Mod. | Tape+Clip | Narrow |
| House | Round warmth | Moderate | Soft | Tape | Moderate |
| Techno | Driving grit | Long | Mod. | Overdrive | Wide |
| Hyperpop | Extreme | Short | Extreme | Digital | Max |
| Ambient | Soft thud | Long | None | None | Max |
| Cinematic | Massive hit | Long | Varied | Tape | Max |
| Pop | Clean polish | Moderate | Mod. | Tube | Mod. |
| R&B | Warm pocket | Long | Soft | Tape | Mod. |

cShot uses this knowledge to automatically shape every generated sound to fit its genre context.
