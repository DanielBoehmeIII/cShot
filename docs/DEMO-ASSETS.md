# cShot Demo Assets

No copyrighted audio. No large binary files. These are text-only assets: prompts,
reference descriptions, recipe presets, and metadata fixtures.

---

## 1. Demo Prompts

### Kick Drums

| Prompt | Character | Duration | Notes |
|--------|-----------|----------|-------|
| `punchy kick 140` | Fast attack, short subby tail, aggressive | ~400ms | Hero prompt. Works every time. |
| `deep 808 sub kick, tuned to E` | Long sub tail, warm, round | ~800ms | Good for trap/hip-hop demo |
| `tight electronic kick, clicky attack` | Very short, metallic transient | ~200ms | Shows transient precision |
| `rock kick, natural beater attack` | Mid-forward, ring, organic | ~500ms | Contrast with electronic |
| `house kick, round, warm, four-on-the-floor` | Soft attack, long body | ~600ms | Genre-specific demo |
| `techno kick, distorted, driving, industrial` | Gritty, long decay, overdriven | ~800ms | Shows saturation character |

### Snares & Claps

| Prompt | Character | Duration | Notes |
|--------|-----------|----------|-------|
| `crack snare, tight, bright` | Sharp attack, bright body, short | ~300ms | Clean, versatile |
| `trap snare with clap layer` | Layered, resonant, crisp | ~350ms | Shows multi-layer concept |
| `rimshot, wooden, dry` | High-pitched, no body, natural | ~150ms | Minimalist demo |
| `analog clap, crunchy, warm` | Noise burst, saturated, vintage | ~400ms | Shows warmth character |
| `deep clap with reverb, wide` | Room tail, stereo spread | ~800ms | Shows spatial processing |

### Hi-Hats

| Prompt | Character | Duration | Notes |
|--------|-----------|----------|-------|
| `closed hi-hat, tight, bright` | Very short, crisp | ~100ms | Essential every session |
| `open hi-hat, wash` | Longer decay, sizzle | ~400ms | Good contrast with closed |
| `shaker, organic, high` | Loose, multiple hits | ~300ms | Natural texture demo |
| `ride cymbal bell, cutting` | Metallic sustain, high-pitched | ~500ms | Percussive variety |

### Bass / 808

| Prompt | Character | Duration | Notes |
|--------|-----------|----------|-------|
| `warm sub bass, long sustain` | Deep fundamental, smooth | ~1000ms | Shows tonal generation |
| `808 slide, from E to A, distorted` | Pitch glide, gritty | ~1200ms | Advanced technique demo |
| `sine wave sub, clean, 50Hz` | Pure tone, no harmonics | ~800ms | Shows fundamental control |

### FX & Impacts

| Prompt | Character | Duration | Notes |
|--------|-----------|----------|-------|
| `cinematic impact, epic, orchestral` | Wide, layered, massive | ~1500ms | Hero demo moment |
| `riser, tension, electronic` | Pitch sweep up, noise | ~2000ms | Tension-builder |
| `reverse cymbal, swell` | Slow fade-in, shimmer | ~1500ms | Classic transition |
| `sub hit, deep, long release` | Pure low-end thump | ~1000ms | Simple but effective |

### Ambient / Pads

| Prompt | Character | Duration | Notes |
|--------|-----------|----------|-------|
| `dark ambient pad, evolving` | Slow, cold, atmospheric | ~3000ms | Texture generation demo |
| `warm drone, soft, low` | Simple, warm, sustaining | ~3000ms | Contrast with dark pad |

---

## 2. Recommended Reference Audio Descriptions

These describe the ideal reference audio for testing reference-based generation.
Users can find equivalent audio from public-domain sources (freesound.org, etc.).

### Reference: Punchy Kick

```
Format: WAV, mono, 44.1kHz, 16-bit
Duration: 200-500ms
Character:
  - Fast transient attack (<3ms)
  - Strong 60-80Hz fundamental
  - Minimal body, mostly sub + click
  - Clean cutoff, no tail reverb
  - Peak: -6 to -3 dBFS
Source suggestion: Search freesound.org for "acoustic kick drum close"
```

### Reference: Warm Snare

```
Format: WAV, mono, 44.1kHz, 16-bit
Duration: 150-400ms
Character:
  - Moderate attack (3-8ms)
  - Body resonance around 200-400Hz
  - Bright crack at 5-10kHz
  - Natural ring, short decay
  - No reverb preferred (reference purity)
Source suggestion: Search freesound.org for "snare drum dry"
```

### Reference: 808 Bass Hit

```
Format: WAV, mono, 44.1kHz, 16-bit
Duration: 400-1200ms
Character:
  - Deep sub fundamental (35-60Hz)
  - Clean sine character
  - Long decay with pitch stability
  - Minimal harmonics for clean reference
Source suggestion: Search freesound.org for "sine wave sub low"
```

### Reference: Hi-Hat Closed

```
Format: WAV, mono, 44.1kHz, 16-bit
Duration: 50-150ms
Character:
  - Very fast attack (<1ms)
  - Broad frequency content (2-15kHz)
  - Short decay, no sustain
  - Clean, bright, metallic
Source suggestion: Search freesound.org for "hi hat closed"
```

### Reference: Room Tone

```
Format: WAV, mono or stereo, 44.1kHz, 16-bit
Duration: 1000-3000ms
Character:
  - Natural room ambience
  - Flat frequency response
  - No distinct sounds or transients
  - Low noise floor
Source suggestion: Record your own room tone (silent room, 3 seconds)
```

---

## 3. Demo Recipe Presets

These are simplified recipe presets usable with the mock DSP pipeline.

```json
{
  "presets": [
    {
      "id": "punchy_trap_kick",
      "name": "Punchy Trap Kick",
      "prompt": "punchy kick 140",
      "dsp": {
        "low_pass": false,
        "high_pass": false,
        "punch": true,
        "bright": false,
        "dark": false,
        "gain": 1.2,
        "noise_amt": 0.0,
        "decay_factor": 0.8
      }
    },
    {
      "id": "deep_808_sub",
      "name": "Deep 808 Sub",
      "prompt": "deep 808 sub kick, tuned to E",
      "dsp": {
        "low_pass": true,
        "high_pass": false,
        "punch": false,
        "bright": false,
        "dark": true,
        "gain": 1.3,
        "noise_amt": 0.0,
        "decay_factor": 1.5
      }
    },
    {
      "id": "bright_snare_crack",
      "name": "Bright Snare Crack",
      "prompt": "crack snare, tight, bright",
      "dsp": {
        "low_pass": false,
        "high_pass": true,
        "punch": true,
        "bright": true,
        "dark": false,
        "gain": 1.1,
        "noise_amt": 0.2,
        "decay_factor": 0.6
      }
    },
    {
      "id": "warm_house_kick",
      "name": "Warm House Kick",
      "prompt": "house kick, round, warm, four-on-the-floor",
      "dsp": {
        "low_pass": false,
        "high_pass": false,
        "punch": false,
        "bright": false,
        "dark": true,
        "gain": 1.0,
        "noise_amt": 0.0,
        "decay_factor": 1.0
      }
    },
    {
      "id": "cinematic_impact",
      "name": "Cinematic Impact",
      "prompt": "cinematic impact, epic, orchestral",
      "dsp": {
        "low_pass": false,
        "high_pass": false,
        "punch": true,
        "bright": true,
        "dark": false,
        "gain": 1.5,
        "noise_amt": 0.3,
        "decay_factor": 2.0
      }
    },
    {
      "id": "distorted_techno_kick",
      "name": "Distorted Techno Kick",
      "prompt": "techno kick, distorted, driving, industrial",
      "dsp": {
        "low_pass": false,
        "high_pass": false,
        "punch": true,
        "bright": true,
        "dark": false,
        "gain": 1.8,
        "noise_amt": 0.1,
        "decay_factor": 1.2
      }
    },
    {
      "id": "ambient_pad_texture",
      "name": "Ambient Pad Texture",
      "prompt": "dark ambient pad, evolving",
      "dsp": {
        "low_pass": true,
        "high_pass": false,
        "punch": false,
        "bright": false,
        "dark": true,
        "gain": 0.8,
        "noise_amt": 0.4,
        "decay_factor": 3.0
      }
    }
  ]
}
```

---

## 4. Sample Metadata Fixtures

These fixture records represent what a generated sound's metadata looks like
in the database. Useful for testing library display without real generation.

```json
{
  "sounds": [
    {
      "id": "demo-kick-001",
      "prompt": "punchy kick 140",
      "sound_type": "kick",
      "duration_ms": 423.0,
      "sample_rate": 44100,
      "rms": 0.245,
      "peak": 0.891,
      "spectral_centroid": 1850.0,
      "tags": ["kick", "punchy", "short"],
      "is_favorite": true,
      "source": "demo",
      "variant_name": null,
      "model": "mock-dsp",
      "seed": 847291,
      "score": 87,
      "failure_labels": []
    },
    {
      "id": "demo-snare-002",
      "prompt": "crack snare, tight, bright",
      "sound_type": "snare",
      "duration_ms": 312.0,
      "sample_rate": 44100,
      "rms": 0.198,
      "peak": 0.875,
      "spectral_centroid": 4200.0,
      "tags": ["snare", "bright", "crack"],
      "is_favorite": false,
      "source": "demo",
      "variant_name": null,
      "model": "mock-dsp",
      "seed": 847292,
      "score": 82,
      "failure_labels": []
    },
    {
      "id": "demo-hat-003",
      "prompt": "closed hi-hat, tight, bright",
      "sound_type": "closed_hat",
      "duration_ms": 98.0,
      "sample_rate": 44100,
      "rms": 0.120,
      "peak": 0.812,
      "spectral_centroid": 8800.0,
      "tags": ["closed_hat", "bright", "short"],
      "is_favorite": true,
      "source": "demo",
      "variant_name": null,
      "model": "mock-dsp",
      "seed": 847293,
      "score": 90,
      "failure_labels": []
    },
    {
      "id": "demo-808-004",
      "prompt": "deep 808 sub kick, tuned to E",
      "sound_type": "bass",
      "duration_ms": 890.0,
      "sample_rate": 44100,
      "rms": 0.320,
      "peak": 0.920,
      "spectral_centroid": 320.0,
      "tags": ["bass", "sub", "deep"],
      "is_favorite": false,
      "source": "demo",
      "variant_name": null,
      "model": "mock-dsp",
      "seed": 847294,
      "score": 78,
      "failure_labels": []
    },
    {
      "id": "demo-impact-005",
      "prompt": "cinematic impact, epic, orchestral",
      "sound_type": "fx",
      "duration_ms": 1520.0,
      "sample_rate": 44100,
      "rms": 0.280,
      "peak": 0.950,
      "spectral_centroid": 2100.0,
      "tags": ["fx", "cinematic", "long"],
      "is_favorite": true,
      "source": "demo",
      "variant_name": null,
      "model": "mock-dsp",
      "seed": 847295,
      "score": 85,
      "failure_labels": []
    },
    {
      "id": "demo-clap-006",
      "prompt": "analog clap, crunchy, warm",
      "sound_type": "clap",
      "duration_ms": 350.0,
      "sample_rate": 44100,
      "rms": 0.210,
      "peak": 0.860,
      "spectral_centroid": 3800.0,
      "tags": ["clap", "warm", "crunchy"],
      "is_favorite": false,
      "source": "demo",
      "variant_name": null,
      "model": "mock-dsp",
      "seed": 847296,
      "score": 76,
      "failure_labels": []
    }
  ]
}
```
