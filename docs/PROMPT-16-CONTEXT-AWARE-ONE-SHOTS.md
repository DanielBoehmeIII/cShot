# Prompt 16 — Context-Aware One-Shots

cShot generates sounds that fit INTO music, not just exist independently.

---

## 1. Context Dimensions

### 1.1 Musical Context

| Dimension | Description | Source |
|-----------|-------------|--------|
| BPM | Beats per minute (tempo) | Audio analysis or user input |
| Key | Harmonic key and mode | Audio analysis or user input |
| Scale | Active scale degrees | Derived from key |
| Chord progression | Harmonic movement | Audio analysis |
| Arrangement position | Intro, verse, chorus, bridge, outro | Track analysis |
| Energy level | 0-1 normalized energy curve | RMS + spectral analysis |
| Mix density | Sparse → dense | Number of active tracks/elements |
| Genre | Primary and secondary genres | Genre classification |
| Era | Production era/style | Style classification |
| Reference track | Target production sound | User input or analysis |

### 1.2 Production Context

| Dimension | Description |
|-----------|-------------|
| Headroom | Available dynamic range |
| Frequency mask | Occupied frequency ranges by other elements |
| Sidechain bus | Kicks being side-chained? Compression settings? |
| Reverb bus | Reverb return characteristics |
| Effect chain | Existing FX on the bus this sound feeds into |
| Stereo field | Current stereo image balance |
| Loudness target | Target LUFS for final mix |

### 1.3 User Context

| Dimension | Description |
|-----------|-------------|
| User's genre focus | What the user typically produces |
| User's production level | Beginner → professional |
| User's hardware | Monitors, headphones, subwoofer |
| Environment | Studio, bedroom, laptop speakers |
| Goal | Demo, release, sound design exploration |

---

## 2. Contextual Embeddings

### 2.1 Context Encoder

```python
class ContextEncoder(nn.Module):
    """Encode full musical/production context into a conditioning vector."""
    
    def __init__(self):
        # Numerical context
        self.numerical_encoder = nn.Sequential(
            nn.Linear(20, 64),  # BPM, energy, position, headroom, etc.
            nn.GELU(),
            nn.Linear(64, 64)
        )
        
        # Genre embedding
        self.genre_embed = nn.Embedding(100, 32)
        
        # Key embedding
        self.key_embed = nn.Embedding(24, 16)
        
        # Spectral context (analyze stems)
        self.spectral_encoder = nn.TransformerEncoder(
            nn.TransformerEncoderLayer(d_model=128, nhead=4),
            num_layers=3
        )
        
        # Fusion
        self.fusion = nn.Sequential(
            nn.Linear(64 + 32 + 16 + 128, 256),
            nn.GELU(),
            nn.Linear(256, 128)
        )
        
    def forward(self, context):
        num_feats = self.numerical_encoder(context.numerical)
        genre_feat = self.genre_embed(context.genre_id)
        key_feat = self.key_embed(context.key_id)
        
        # Encode spectral context from other tracks
        if context.stems is not None:
            stem_specs = [mel_spectrogram(stem) for stem in context.stems]
            spectral_feat = self.spectral_encoder(stem_specs)
        else:
            spectral_feat = torch.zeros(128)
        
        # Fuse all context
        combined = torch.cat([num_feats, genre_feat, key_feat, spectral_feat])
        return self.fusion(combined)
```

### 2.2 Context-Aware Generation

```python
class ContextAwareGenerator:
    """Generate one-shots conditioned on musical context."""
    
    def generate_kick(self, context):
        conditioning = context_encoder(context)
        
        # Key adaptation: tune kick fundamental to key
        fundamental_freq = get_key_root_freq(context.key)
        
        # BPM adaptation: adjust decay time to tempo
        target_decay = self.bpm_to_decay(context.bpm, context.arrangement_position)
        
        # Energy adaptation: adjust punch and weight
        target_punch = self.energy_to_punch(context.energy_level)
        target_weight = self.energy_to_weight(context.energy_level)
        
        # Mix adaptation: avoid frequency masking
        frequency_gap_plan = self.plan_frequency_allocation(context.spectral_context)
        
        # Generate with conditioning
        sound = self.generator(
            conditioning=conditioning,
            fundamental=fundamental_freq,
            decay=target_decay,
            punch=target_punch,
            weight=target_weight,
            frequency_gap_plan=frequency_gap_plan
        )
        
        return sound
    
    def bpm_to_decay(self, bpm, position):
        """Decay should end before next kick."""
        beat_duration = 60.0 / bpm  # seconds per beat
        
        # At half-note division (typical kick spacing)
        if position in ['drop', 'chorus']:
            max_decay = beat_duration * 2  # decay fits in 2 beats
        elif position in ['verse', 'bridge']:
            max_decay = beat_duration * 1.5
        else:
            max_decay = beat_duration * 3  # more space in intros/outros
        
        return min(0.8, max_decay * 0.9)  # 90% of max
    
    def energy_to_punch(self, energy):
        """Map track energy to kick punch."""
        if energy < 0.3:  # low energy
            return 0.3     # soft, pillowy
        elif energy < 0.6:  # medium
            return 0.6     # balanced
        else:              # high energy
            return 0.85    # aggressive punch
```

---

## 3. Arrangement-Aware Generation

### 3.1 Position-Specific Sound Design

```python
class ArrangementAwareDesigner:
    """Design sounds that work at specific arrangement positions."""
    
    def design_for_position(self, sound_type, position, context):
        positions = {
            'intro': {
                'kick': {'decay': 'long', 'punch': 'low', 'sub': 'prominent'},
                'snare': {'reverb': 'high', 'body': 'full'},
                'hat': {'closed': True, 'velocity': 'low'},
            },
            'verse': {
                'kick': {'decay': 'medium', 'punch': 'medium', 'sub': 'moderate'},
                'snare': {'reverb': 'medium', 'body': 'medium'},
                'hat': {'pattern': '8th_notes', 'velocity': 'medium'},
            },
            'buildup': {
                'kick': {'decay': 'shortening', 'punch': 'increasing', 'frequency': 'rising'},
                'snare': {'reverb': 'increasing', 'hits': 'rolls'},
                'hat': {'speed': 'accelerating', 'velocity': 'rising'},
            },
            'drop': {
                'kick': {'decay': 'tight', 'punch': 'maximum', 'sub': 'powerful'},
                'snare': {'reverb': 'tight', 'body': 'punchy', 'crack': 'maximum'},
                'hat': {'pattern': 'driving', 'velocity': 'high'},
            },
            'bridge': {
                'kick': {'decay': 'long', 'punch': 'low', 'sub': 'warm'},
                'snare': {'reverb': 'high', 'body': 'atmospheric'},
                'hat': {'closed': True, 'velocity': 'very_low'},
            },
            'outro': {
                'kick': {'decay': 'long', 'punch': 'decreasing', 'sub': 'fading'},
                'snare': {'reverb': 'high', 'body': 'dissolving'},
                'hat': {'speed': 'slowing', 'velocity': 'decreasing'},
            },
        }
        
        position_params = positions.get(position, positions['verse'])
        return self.apply_position_params(sound_type, position_params, context)
```

### 3.2 Energy Curve Matching

```python
def match_energy_curve(sound, energy_track, position):
    """Shape the one-shot's energy to fit the track's energy curve."""
    # Extract track energy at this position
    local_energy = energy_track[position.start:position.end]
    
    # Shape the sound's envelope to match
    if local_energy.is_increasing():
        # Sound should build
        sound = apply_rising_envelope(sound, rate=local_energy.slope)
    elif local_energy.is_decreasing():
        # Sound should decay
        sound = apply_dropping_envelope(sound, rate=local_energy.slope)
    elif local_energy.is_peak():
        # Sound should be maximally present
        sound = maximize_impact(sound)
    elif local_energy.is_valley():
        # Sound should be subtle
        sound = reduce_presence(sound)
    
    return sound
```

---

## 4. Mix-Aware Synthesis

### 4.1 Frequency Allocation Planning

```python
def plan_frequency_allocation(existing_stems):
    """Analyze frequency usage and plan gaps for new sound."""
    # Compute spectral masks for each stem
    masks = {}
    for stem in existing_stems:
        spec = compute_per_band_energy(stem, num_bands=32)
        masks[stem.name] = spec
    
    # Sum all masks
    occupied = sum(masks.values())
    
    # Find unoccupied frequency bands
    threshold = 0.3 * max(occupied)  # 30% of peak
    gaps = []
    for band_idx, energy in enumerate(occupied):
        if energy < threshold:
            gaps.append(band_idx)
    
    # Recommend frequency focus
    if gaps:
        primary_gap = find_wideest_contiguous_gap(gaps)
        return {
            'focus_band': primary_gap.center,
            'avoid_bands': [i for i, e in enumerate(occupied) if e > threshold * 2],
            'allocation_plan': {
                'fundamental': place_in_gap(low_gaps, 50-100Hz),
                'body': place_in_gap(mid_gaps, 200-800Hz),
                'presence': place_in_gap(high_gaps, 2-5kHz),
                'air': place_in_gap(top_gaps, 8-15kHz),
            }
        }
    
    # No gaps: sound will need to blend via dynamic EQ / sidechain
    return {'strategy': 'sidechain', 'sidechain_source': find_competing_stem(stems)}
```

### 4.2 Dynamic Range Matching

```python
def match_dynamic_range(sound, context):
    """Adjust sound's dynamics to fit mix density."""
    mix_density = context.mix_density  # 0-1
    
    if mix_density < 0.3:  # sparse
        # Sound can have wide dynamic range
        sound.dynamic_range = 20  # dB
    elif mix_density < 0.6:  # moderate
        # Moderate compression
        sound.dynamic_range = 12
    else:  # dense
        # Heavy compression to cut through
        sound.dynamic_range = 6
    
    # Match loudness to context
    target_loudness = context.loudness_target - 3  # dB, slightly below master
    sound = normalize_loudness(sound, target_loudness)
    
    return sound
```

### 4.3 Phase and Stereo Compatibility

```python
def ensure_mix_compatibility(sound, context):
    """Ensure sound doesn't cause mix issues."""
    # Check phase correlation with kick bus
    if context.sidechain_bus == 'kick':
        # Sound will be sidechained to kick:
        # Design to pump with kick
        sound = design_for_sidechain(sound, threshold=-20, attack=1, release=50)
    
    # Stereo field compatibility
    if context.stereo_field_is_wide:
        # Keep new sound more centered
        sound.stereo_width = min(0.5, sound.stereo_width)
    else:
        # Can use width
        pass
    
    # Check for phase cancellation with existing similar sounds
    for existing in context.get_similar_sounds(sound.type):
        phase_alignment = measure_phase_coherence(sound, existing)
        if phase_alignment < 0.3:
            sound = shift_phase(sound, optimal_phase_offset(existing))
    
    return sound
```

---

## 5. Automatic Genre Adaptation

### 5.1 Genre Signature Database

```python
genre_signatures = {
    'techno': {
        'kick': {'punch': 0.7, 'body_freq': 60, 'click_freq': 4000, 'decay': 'short'},
        'hihat': {'velocity': 0.5, 'pattern': '16th_note', 'open_ratio': 0.3},
        'clap': {'reverb_size': 0.3, 'texture': 'metallic'},
        'typical_bpm_range': (125, 135),
        'mix_characteristics': {'compression': 'heavy', 'reverb': 'minimal'}
    },
    'lo-fi': {
        'kick': {'punch': 0.4, 'body_freq': 100, 'decay': 'medium', 'noise_floor': -40},
        'hihat': {'velocity': 0.3, 'pattern': 'offbeat', 'open_ratio': 0.1},
        'snare': {'reverb_size': 0.6, 'texture': 'soft', 'crack_freq': 3000},
        'typical_bpm_range': (80, 95),
        'mix_characteristics': {'compression': 'light', 'saturation': 'tape'}
    },
    'dnb': {
        'kick': {'punch': 0.8, 'body_freq': 80, 'decay': 'tight'},
        'snare': {'punch': 0.9, 'body_freq': 200, 'crack_freq': 5000},
        'hihat': {'velocity': 0.8, 'pattern': 'syncopated'},
        'typical_bpm_range': (170, 180),
        'mix_characteristics': {'compression': 'heavy', 'sidechain': 'prominent'}
    },
    # ... 50+ genre signatures
}
```

### 5.2 Genre Adaptation Engine

```python
def adapt_to_genre(sound, genre, context):
    """Modify a sound to fit a specific genre."""
    signature = genre_signatures[genre]
    sound_type = classify_sound_type(sound)
    type_params = signature[sound_type]
    
    # Apply genre-specific characteristics
    if 'punch' in type_params:
        sound = set_perceptual(sound, 'punch', type_params['punch'])
    if 'body_freq' in type_params:
        sound = shift_fundamental(sound, type_params['body_freq'])
    if 'decay' in type_params:
        sound = set_decay(sound, type_params['decay'])
    if 'noise_floor' in type_params:
        sound = add_noise_floor(sound, type_params['noise_floor'])
    if 'crack_freq' in type_params:
        sound = boost_band(sound, type_params['crack_freq'], q=2, gain=3)
    if 'reverb_size' in type_params:
        sound = set_reverb(sound, size=type_params['reverb_size'])
    if 'texture' in type_params:
        sound = apply_texture(sound, type_params['texture'])
    
    return sound
```

---

## 6. Context Pipeline Architecture

```
                    ┌──────────────────────────────────┐
                    │        User Inputs                │
                    │  (genre, BPM, key, stems, goals)  │
                    └──────────┬───────────────────────┘
                               ↓
┌──────────────────────────────────────────────────────────┐
│                  Context Analyzer                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │  Tempo   │  │   Key    │  │  Energy  │  │  Spectral ││
│  │  Detect  │  │  Detect  │  │  Curve   │  │  Analysis ││
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘│
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │  Genre   │  │  Arrang. │  │  Mix     │  │  Stem    ││
│  │  Classify│  │  Pos.    │  │  Density │  │  Sep.    ││
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘│
└──────────────────────┬───────────────────────────────────┘
                       ↓
┌──────────────────────────────────────────────────────────┐
│              Context Encoder (128-D embedding)            │
└──────────────────────┬───────────────────────────────────┘
                       ↓
┌──────────────────────────────────────────────────────────┐
│              Context-Conditioned Generator                │
│                                                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │  Freq    │ │  Dynamic │ │  Stereo  │ │  Percep. │    │
│  │  Planning│ │  Adapt.  │ │  Adapt.  │ │  Adapt.  │    │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │
│                                                           │
│  ┌────────────────────────────────────────────────┐      │
│  │           Generation Core (DSP + AI)            │      │
│  └────────────────────────────────────────────────┘      │
└──────────────────────┬───────────────────────────────────┘
                       ↓
                    Output Audio
                       ↓
┌──────────────────────────────────────────────────────────┐
│               Mix Validation                              │
│  - Frequency masking check                                │
│  - Phase coherence check                                  │
│  - Dynamic range check                                    │
│  - Stereo compatibility check                             │
│  - Perceptual integration score                           │
└──────────────────────────────────────────────────────────┘
```

---

## 7. Stem Integration Workflow

When a user provides stems:
1. **Separate** stems using source separation (if needed)
2. **Analyze** each stem for frequency content, dynamics, stereo
3. **Identify gaps** — frequency, dynamic, and textural
4. **Design** sound to fill identified gaps
5. **Validate** — check mix compatibility
6. **Iterate** — if validation fails, adjust and regenerate

---

## 8. Contextual Interpolation

```python
def generate_for_context_transition(context_a, context_b, t):
    """Generate a sound that bridges two musical contexts."""
    # Interpolate all context dimensions
    interpolated_context = {
        'bpm': lerp(context_a.bpm, context_b.bpm, t),
        'energy': lerp(context_a.energy, context_b.energy, t),
        'key': interpolate_key(context_a.key, context_b.key, t),
        'genre': blend_genre_weights(context_a.genre_weights, context_b.genre_weights, t),
        'position': lerp(context_a.position_embed, context_b.position_embed, t),
    }
    
    return generate_with_context(interpolated_context)
```

This allows cShot to generate sounds that smoothly transition between sections of a track — e.g., a kick that morphs from verse-character to drop-character over the buildup.
