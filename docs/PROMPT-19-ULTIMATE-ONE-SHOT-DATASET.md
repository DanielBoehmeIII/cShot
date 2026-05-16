# Prompt 19 — Design the Ultimate One-Shot Dataset

Research-grade dataset for one-shot generation and understanding.

---

## 1. Dataset Requirements

### 1.1 Scale

| Tier | Size | Purpose |
|------|------|---------|
| Core | 100,000 professionally curated one-shots | Training, validation, test |
| Extended | 1,000,000 curated + auto-labeled | Scaling training |
| Full | 10,000,000+ web-scale | Foundation model training |

### 1.2 Diversity Requirements

- **Sound types**: kicks, snares, hats, claps, toms, percussion, bass hits, synth stabs, FX, risers, impacts, atmospheres, foley, organic, found sound, hybrid
- **Genres**: house, techno, DnB, trap, hip-hop, lo-fi, dubstep, garage, ambient, pop, rock, orchestral, cinematic, experimental, glitch, IDM, jazz, world
- **Production eras**: 1970s-present, with decade labels
- **Quality levels**: professional releases → bedroom production → mobile recording
- **Processing states**: raw/unprocessed → lightly processed → heavily processed → mastered

---

## 2. Collection Methods

### 2.1 Primary Sources

| Source | Method | Expected Volume | Quality | Copyright |
|--------|--------|----------------|---------|-----------|
| Public sample packs (CC0) | Crawl Freesound, SampleSwap, etc. | 50,000 | Medium-high | CC0/Creative Commons |
| Licensed sample packs | Partner with producers/labels | 100,000 | High | Licensed for training |
| User contributions | Upload portal with review | 500,000 | Variable | User grants license |
| Synthetic generation | DSP engine (Prompt 18) | 5,000,000 | Controllable | Owned |
| YouTube/audio extraction | Extract percussion from music | 2,000,000 | Variable | Fair use research? |
| Field recordings | Crowd-sourced | 50,000 | Variable | CC0 |
| DAW project mining | Extract from open projects | 100,000 | High | Project-dependent |

### 2.2 Copyright-Safe Strategy

1. **CC0 first**: Prioritize public domain / CC0 samples
2. **Synthetic generation**: Create unlimited clean data from DSP engine
3. **Fair use research exemption**: Small-scale extraction for academic purposes
4. **Licensing partnerships**: Formal agreements with sample pack creators
5. **User-contribution license**: Platform TOS grants training rights
6. **Derivative detection**: Filter samples too similar to copyrighted works

---

## 3. Cleaning Pipeline

```
Raw Audio
  ↓
Deduplication (acoustic fingerprinting)
  ↓
Silence/truncation removal
  ↓
Clipping detection
  ↓
Noise floor normalization
  ↓
Sample rate standardization (44.1kHz)
  ↓
Bit depth standardization (24-bit)
  ↓
Loudness normalization (LUFS)
  ↓
Length normalization (pad/trim to 4 seconds)
  ↓
Quality scoring (perceptual model)
  ↓
Clean Dataset
```

### 3.1 Quality Filtering

```python
def quality_score(audio):
    """Score a one-shot on technical quality (0-1)."""
    scores = {}
    
    # Noise floor
    noise_db = estimate_noise_floor(audio)
    scores['noise_floor'] = sigmoid((noise_db + 60) / 10)  # -60dB or better = 0.5
    
    # Clipping
    peak = max(abs(audio))
    scores['clipping'] = 1.0 if peak < 0.99 else max(0, 1 - (peak - 0.99) * 10)
    
    # Bandwidth
    spectral_rolloff = compute_spectral_rolloff(audio)
    scores['bandwidth'] = min(1.0, spectral_rolloff / 18000)  # close to 18kHz = good
    
    # Dynamic range
    crest = peak / rms(audio)
    scores['dynamic_range'] = min(1.0, crest / 20)  # crest factor ~20 = good
    
    # Stereo integrity (for stereo files)
    if audio.shape[0] == 2:
        phase_coherence = abs(mean(audio[0] * audio[1]))
        scores['phase'] = min(1.0, phase_coherence * 5)  # some correlation is expected
    
    # Overall
    weights = {'noise_floor': 0.3, 'clipping': 0.3, 'bandwidth': 0.2, 
               'dynamic_range': 0.1, 'phase': 0.1}
    
    return sum(scores.get(k, 0.5) * w for k, w in weights.items())
```

### 3.2 Deduplication Pipeline

```python
def deduplicate(dataset, threshold=0.95):
    """Remove near-duplicate samples."""
    fingerprints = []
    unique = []
    
    for audio in dataset:
        fp = acoustic_fingerprint(audio)  # 64-bit hash
        is_duplicate = False
        for existing_fp in fingerprints:
            if hamming_distance(fp, existing_fp) < threshold * 64:
                is_duplicate = True
                break
        if not is_duplicate:
            fingerprints.append(fp)
            unique.append(audio)
    
    return unique
```

---

## 4. Annotation Systems

### 4.1 Label Taxonomy

#### Sound Type (hierarchical, 4 levels)
```
Percussion
  ├── Kick
  │   ├── Acoustic Kick
  │   ├── Electronic Kick
  │   ├── 808 Kick
  │   └── Layered Kick
  ├── Snare
  │   ├── Acoustic Snare
  │   ├── Electronic Snare
  │   ├── Rimshot
  │   └── Clap
  ├── Hi-hat
  │   ├── Closed Hat
  │   ├── Open Hat
  │   └── Pedal Hat
  └── ...
Tonal
  ├── Bass Hit
  ├── Synth Stab
  ├── Piano Hit
  └── ...
FX
  ├── Riser
  ├── Impact
  ├── Sweep
  └── ...
```

#### Genre Labels (multi-label)
Electronic genres → subgenres. Each sample can have multiple genre tags with confidence weights.

#### Production Labels
| Label | Values |
|-------|--------|
| Processing level | Raw, Light, Heavy, Mastered |
| Source | Synthesized, Sampled, Recorded, Hybrid |
| Reverb | None, Room, Hall, Plate, Spring, Convolution |
| Distortion | None, Light, Moderate, Heavy |
| Compression | None, Light, Moderate, Heavy |
| Stereo type | Mono, Stereo, Mid-Side |
| Bit depth | 16-bit, 24-bit, 32-bit |
| Sample rate | 44.1k, 48k, 96k |

#### Technical Descriptors (auto-extracted)
From Prompt 11: punch, warmth, brightness, harshness, depth, width, texture, gloss, air, presence, realism.

#### Emotional Labels (auto-predicted)
From Prompt 12: VAP coordinates + semantic descriptors + emotion categories.

### 4.2 Automated Annotation Pipeline

```python
class AutoAnnotator:
    """Fully automated annotation of one-shot dataset."""
    
    def annotate(self, audio):
        return {
            # Technical
            'sample_rate': audio.sample_rate,
            'bit_depth': audio.bit_depth,
            'channels': audio.channels,
            'duration': len(audio) / audio.sample_rate,
            'peak_level': max(abs(audio)),
            'rms_level': rms(audio),
            'crest_factor': max(abs(audio)) / rms(audio),
            'noise_floor': estimate_noise_floor(audio),
            
            # Sound type classification
            'sound_type': self.classify_sound_type(audio),
            'sound_type_confidence': ...,
            'subtype': self.classify_subtype(audio),
            
            # Genre prediction
            'genre_predictions': self.predict_genre(audio),  # list of (genre, confidence)
            
            # Perceptual descriptors (Prompt 11)
            'perceptual': perceptual_model(audio),
            
            # Emotional descriptors (Prompt 12)
            'emotional': emotion_model(audio),
            
            # Production descriptors
            'processing_level': estimate_processing(audio),
            'reverb_amount': estimate_reverb(audio),
            'compression_amount': estimate_compression(audio),
            'distortion_amount': estimate_distortion(audio),
            
            # Latent embedding (Prompt 13)
            'dna_embedding': dna_encoder(audio),
            
            # Cluster assignment
            'cluster_id': cluster_model.predict(dna_encoder(audio))
        }
```

### 4.3 Human Annotation Strategy

| Annotation Type | Method | Samples | Cost | Quality |
|----------------|--------|---------|------|---------|
| Sound type | Crowd (5 votes/sample) | 100,000 | Low | High (majority vote) |
| Genre | Expert (2 votes) | 50,000 | Medium | Very high |
| Emotional | Expert (3 votes) | 20,000 | High | Highest |
| Quality rating | Crowd (3 votes) | 100,000 | Low | Medium |
| Perceptual similarity | Paired comparison | 10,000 pairs | High | Very high |

### 4.4 Self-Supervised Labeling

Use contrastive learning to propagate labels:
1. Train embedding model on 10K expertly labeled samples
2. Embed all 1M+ unlabeled samples
3. Propagate labels via k-NN in embedding space
4. Confidence threshold → auto-labeled
5. Active learning → select uncertain samples for human labeling

---

## 5. Augmentation Systems

### 5.1 Perceptual Augmentations (preserve identity)

```python
def augment_preserving(audio, n_variants=5):
    """Apply augmentations that preserve the sound's identity."""
    variants = []
    for _ in range(n_variants):
        aug = audio.copy()
        
        # Mild EQ variation
        if random() < 0.5:
            aug = apply_shelving_eq(aug, gain=uniform(-2, 2))
        
        # Mild dynamics variation
        if random() < 0.3:
            aug = apply_compression(aug, ratio=uniform(1.5, 2.5))
        
        # Mild pitch shift (semitone ± 1)
        if random() < 0.3:
            aug = pitch_shift(aug, semitones=uniform(-1, 1))
        
        # Mild time stretch (5% max)
        if random() < 0.2:
            aug = time_stretch(aug, rate=uniform(0.95, 1.05))
        
        # Mild reverb
        if random() < 0.3:
            aug = apply_reverb(aug, size=uniform(0.1, 0.3))
        
        variants.append(aug)
    
    return variants
```

### 5.2 Transformative Augmentations (new sounds from old)

```python
def augment_transform(audio, n_variants=5):
    """Apply aggressive augmentations that create new sounds."""
    variants = []
    for _ in range(n_variants):
        aug = audio.copy()
        
        # Extreme EQ
        if random() < 0.5:
            aug = apply_extreme_eq(aug, profile=choice(['telephone', 'bass_only', 'highs_only']))
        
        # Extreme pitch
        if random() < 0.4:
            aug = pitch_shift(aug, semitones=uniform(-12, 12))
        
        # Extreme time
        if random() < 0.3:
            aug = time_stretch(aug, rate=uniform(0.5, 2.0))
        
        # Extreme distortion
        if random() < 0.3:
            aug = apply_bitcrush(aug, bits=choice([4, 8, 12]))
        
        # Granular processing
        if random() < 0.3:
            aug = granular_transform(aug, grain_size=uniform(10, 100), 
                                    density=uniform(0.5, 2.0))
        
        # Reverse
        if random() < 0.2:
            aug = reverse(aug)
        
        variants.append(aug)
    
    return variants
```

### 5.3 Synthetic Data Generation

Use the DSP engine from Prompt 18 to generate unlimited labeled training data:

```python
def generate_synthetic_batch(n=1000):
    """Generate fully labeled synthetic one-shots."""
    batch = []
    for _ in range(n):
        params = random_dsp_params()
        audio = dsp_engine.render(params)
        label = {
            'sound_type': params_to_type(params),
            'params': params,
            'perceptual': compute_perceptual(audio),
            'genre_prototype': params_to_genre(params),
        }
        batch.append((audio, label))
    return batch
```

---

## 6. Dataset Splits

| Split | Size | Purpose |
|-------|------|---------|
| Train | 80% (80K core + 800K extended) | Model training |
| Validation | 10% (10K core) | Hyperparameter tuning |
| Test (closed) | 5% (5K core, never seen) | Academic benchmark |
| Test (open) | 5% (5K core, public) | Reproducible comparison |
| Blind test | 10K external | Unseen domain generalization |
| Stress test | 1K pathological cases | Robustness (clipping, noise, etc.) |

---

## 7. Embedding Pipeline

```
Audio (all splits)
  ↓
DNA Encoder (Prompt 14) → 768-dim embedding
  ↓
PCA → 256-dim (whitened)
  ↓
Store in vector database (Prompt 20)
  ↓
Index for similarity search, clustering, label propagation
```

Pre-computed for all samples. Updated when encoder improves.

---

## 8. Dataset Format

```
cshot-dataset-v1/
├── metadata/              # JSON files
│   ├── samples.json       # All sample metadata
│   ├── taxonomy.json      # Label taxonomy definition
│   ├── splits.json        # Train/val/test splits
│   └── licenses.json      # Per-sample license info
├── audio/                 # Audio files
│   ├── train/             # 800,000 files
│   ├── val/               # 100,000 files
│   └── test/              # 100,000 files
├── features/              # Pre-computed features
│   ├── embeddings.h5      # DNA embeddings (768D, 1M samples)
│   ├── perceptual.h5      # Perceptual features (12D)
│   ├── mfcc.h5            # MFCCs (20D, per-frame)
│   └── mel.h5             # Mel spectrograms (128 bands)
├── annotations/           # Human annotations
│   ├── sound_types.csv
│   ├── genres.csv
│   ├── emotional.csv
│   └── quality.csv
└── index/                 # Vector search indexes
    ├── hnsw.bin           # HNSW index for embeddings
    └── metadata.idx       # Metadata index for filtering
```

---

## 9. Dataset Statistics & Quality Metrics

### Targets
```
Total samples:    1,000,000+
Total hours:      1,100 hours
Unique sound types: 50+
Genre coverage:   25+ genres
Source diversity: 5+ collection methods
Human annotations: 100,000+ samples
Human labels:     500,000+ individual labels
```

### Quality Gates
```
Before release:
  - Noise floor < -50dB for 95% of samples
  - No clipping for 99% of samples
  - Dedup threshold >95% fingerprint similarity
  - Sound type accuracy > 90% (human validated)
  - Genre accuracy > 80% (human validated)
  - Embedding coverage > 99% (computable)
```
