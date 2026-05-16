# Prompt 71 — Build a Universal Embedding for One-Shots

Design a universal embedding system for one-shots that captures timbre, transient behavior, spectral balance, emotional perception, genre affinity, mix placement, production style, texture, realism, and energy. It must power search, recommendation, clustering, morphing, generation conditioning, similarity detection, pack creation, and taste modeling.

---

## 1. Why a Universal Embedding Changes Everything

### The Current Fragmentation

```
Today's audio embeddings are fragmented:

  CLAP (LAION):        Text-audio alignment. Good for search, bad for fine-grained similarity.
  TRILL (Google):      Speech representation. Not designed for percussion.
  VGGish:              Generic audio events. Too coarse for one-shot detail.
  OpenL3:              Self-supervised. Decent for timbre, misses temporal structure.
  wav2vec 2.0:         Speech-optimized. Transient information is compressed out.
  MuLan (Google):      Music-text. Designed for songs (30s segments), not 400ms hits.
  Jukebox (OpenAI):    Raw audio VQ-VAE. Powerful but enormous — impractical for real-time.
  Contrastive (CL):    Good for discriminative tasks, weak for generation conditioning.

No single embedding captures ALL the dimensions that matter for one-shots:
  - Timbre        — What the sound is made of
  - Transient     — How the sound attacks
  - Spectral      — Where the energy lives
  - Emotional     — How the sound feels
  - Genre         — Where the sound belongs
  - Mix placement — Where the sound sits
  - Production    — How the sound was made
  - Texture       — The surface quality
  - Realism       — How "real" it sounds
  - Energy        — How aggressive it is
```

### The Unified Hypothesis

> **A single embedding space that jointly models all perceptual and production-relevant dimensions of a one-shot enables every downstream system — search, recommendation, morphing, generation, clustering, taste modeling — to share the same semantic foundation. The embedding isn't a feature extractor. It's the universal language of the platform.**

```
Without universal embedding:
  Search uses CLAP          → misses transient similarity
  Recommendation uses CL    → misses genre alignment
  Clustering uses VGGish    → misses production style
  Morphing uses raw latent  → misses perceptual axes
  Generation uses ASD latent → misses user taste

  Result: 5 separate models, 5 embedding spaces, no shared understanding.

With universal embedding:
  ONE embedding captures all dimensions.
  Search, recommend, cluster, morph, generate — all use the same space.
  Every system benefits from every other system's data.
  Taste feedback improves search. Similarity data improves generation.

  Result: 1 embedding space, shared representation, compounding intelligence.
```

---

## 2. The Dimensions of Sonic Variation

### 10 Perceptual Axes

```
Axis 1 — TIMBRE
  What the sound is made of.
  Continuum: Synthetic ← → Acoustic ← → Hybrid
  Sub-dimensions: brightness, warmth, body, nasal, metallic, woody, airy, buzzy
  Why it matters: Primary categorization axis for any sound.

Axis 2 — TRANSIENT BEHAVIOR  
  How the sound attacks and decays.
  Continuum: Soft ← → Punchy ← → Aggressive
  Sub-dimensions: attack time, peak sharpness, transient-to-tail ratio, decay shape, sustain
  Why it matters: Defines the "feel" of a hit. Critical for drum sounds.

Axis 3 — SPECTRAL BALANCE
  Where the energy lives across frequency.
  Continuum: Dark ← → Balanced ← → Bright
  Sub-dimensions: sub energy, low-mid presence, high-mid cut, air band, spectral centroid
  Why it matters: Determines how a sound sits in a mix.

Axis 4 — EMOTIONAL PERCEPTION
  How the sound makes the listener feel.
  Continuum: Cold ← → Neutral ← → Warm
  Sub-dimensions: aggression, melancholy, tension, excitement, darkness, nostalgia
  Why it matters: Bridges technical and creative intent.

Axis 5 — GENRE AFFINITY
  Which genre conventions the sound fits.
  Continuum: Specific ← → Multipurpose ← → Experimental
  Sub-dimensions: trap, house, techno, pop, cinematic, lo-fi, metal, experimental
  Why it matters: Enables genre-aware generation and recommendation.

Axis 6 — MIX PLACEMENT
  Where the sound naturally sits in a mix.
  Continuum: Back ← → Center ← → Forward
  Sub-dimensions: headroom, spectral carve, stereo width, dynamic range, perceived loudness
  Why it matters: Determines if a sound is "mix-ready" without processing.

Axis 7 — PRODUCTION STYLE
  How the sound was produced/manipulated.
  Continuum: Raw ← → Processed ← → Designed
  Sub-dimensions: saturation, compression, reverb, distortion, modulation, layering
  Why it matters: Describes the production fingerprint.

Axis 8 — TEXTURE
  The surface quality of the sound.
  Continuum: Smooth ← → Gritty ← → Noisy
  Sub-dimensions: graininess, harmonic richness, noise floor, roughness, bite
  Why it matters: Describes the "feel" of the surface, distinct from timbre.

Axis 9 — REALISM
  How "real" vs. "synthetic" the sound feels.
  Continuum: Synthetic ← → Hybrid ← → Organic
  Sub-dimensions: natural variation, harmonic complexity, noise authenticity, artifact presence
  Why it matters: Critical for film/game applications. Also captures generation quality.

Axis 10 — ENERGY
  How aggressive or intense the sound is.
  Continuum: Gentle ← → Moderate ← → Intense
  Sub-dimensions: RMS power, peak factor, spectral flux, transient density, perceived loudness
  Why it matters: Primary filter for mood-based selection.
```

### The Embedding Structure

```
Universal One-Shot Embedding (UShOt-v1)
  ───────────────────────────────────

  Total dimensionality: D = 1024

  ┌────────────────────────────────────────────────────────────┐
  │  Shared backbone (256d)                                    │
  │  • Learned via contrastive pretraining on 10M+ one-shots   │
  │  • Captures general audio structure                        │
  │  • Not interpretable — but foundation for all axes         │
  ├────────────────────────────────────────────────────────────┤
  │  Axis-specific heads (10 × 64d = 640d)                    │
  │  • Each axis has a dedicated 64d subspace                  │
  │  • Learned via supervised fine-tuning on labeled data      │
  │  • Partially interpretable through probe analysis          │
  ├────────────────────────────────────────────────────────────┤
  │  Cross-axis interactions (128d)                            │
  │  • Mixture-of-experts layer combining axis subspaces       │
  │  • Captures correlations: "bright + punchy = aggressive"   │
  │  • Enables semantic vector arithmetic                      │
  └────────────────────────────────────────────────────────────┘

  Why 1024d:
    - Large enough to capture 10+ independent perceptual axes
    - Small enough for real-time similarity search (FAISS IVF-PQ)
    - Compatible with CLAP projection (512d → 1024d)
    - Enables 50M-vector index in ~4GB RAM with product quantization
```

---

## 3. Research Foundations

### Contrastive Learning

```
Why: The backbone of the universal embedding.

Core idea: Learn representations by pulling similar sounds together
and pushing dissimilar sounds apart in embedding space.

Contrastive objective (NT-Xent):
  L = -log( exp(sim(z_i, z_j)/τ) / Σ_k exp(sim(z_i, z_k)/τ) )

Where:
  - z_i, z_j: positive pair (same sound, different augmentations)
  - z_i, z_k: negative pair (different sounds)
  - sim(): cosine similarity
  - τ: temperature (lower = harder separation)

Key insight for one-shots:
  Positive pairs should be:
    - Same sound with different mix context (dry vs wet)
    - Same sound at different pitch (C1 vs C2 kick)
    - Same type (kick-1 vs kick-2) → softer positives
    - Same sound different render (slightly different generation)
  
  Negative pairs should be:
    - Different sound types (kick vs snare)
    - Same type but very different character
    - Random batch negatives
            
  The quality of the embedding depends ENTIRELY on the quality
  of the positive/negative pair definition.
```

### Multimodal Embeddings (CLAP)

```
CLAP (Contrastive Language-Audio Pretraining):

  Parallel encoders:
    Audio → HTSAT transformer → 512d audio embedding
    Text  → RoBERTa/DistilBERT → 512d text embedding
  
  Contrastive loss aligns audio-text pairs in shared space.

  What CLAP gives cShot:
    - Zero-shot search: "punchy kick" finds relevant audio
    - Text conditioning for generation
    - Genre/emotion labels via text projection
  
  What CLAP misses for one-shots:
    - Trained on 15-second clips, not 400ms hits
    - Temporal resolution too coarse for transient detail
    - Doesn't capture production style or mix placement
    - Emotion axis is text-dependent, not perceptually grounded

  Our approach: CLAP as initialization, NOT as final embedding.
    - Initialize audio encoder with CLAP weights
    - Fine-tune on one-shot dataset with axis-specific heads
    - Augment with temporal features (transient encoder)
    - Keep text projection for zero-shot capabilities
```

### Triplet Loss

```
Triplet loss for fine-grained similarity:

  L = max(0, d(a, p) - d(a, n) + margin)

  Where:
    a: anchor sound
    p: positive (similar sound, same axis target)
    n: negative (dissimilar sound)

  Why triplet loss matters for one-shots:
    - Can target specific axes: "same timbre, different energy"
    - Handles fine-grained: "more punchy but equally bright"
    - Natural fit for semantic vector arithmetic training
  
  Mining strategy:
    Easy triplets: different type (kick vs snare)
    Hard triplets: same type, different transient (kick vs kick)
    Semi-hard: for production style, same genre
  
  Adaptive margin per axis:
    Timbre: margin=0.3
    Transient: margin=0.5
    Energy: margin=0.4
    (Wider margin = more separation needed)
```

### Metric Learning

```
Metric learning refines the embedding space for specific tasks.

  Proxy-based: 
    Learn class proxies for sound types (kick, snare, hat, etc.)
    Teaches the embedding to cluster by type.
    Good for: automatic tagging, organization.
  
  ArcFace (Additive Angular Margin):
    Pushes embeddings of same class onto hypersphere.
    Creates angular separation between types.
    Good for: type classification, few-shot adaptation.
  
  N-pair loss:
    Generalizes triplet to N negatives per anchor.
    More stable training, better gradient signal.
    Good for: backbone pretraining.

  Hierarchical metric learning:
    Level 1: Type (kick vs snare vs hat)
    Level 2: Character (punchy kick vs deep kick)
    Level 3: Instance (this specific kick vs that specific kick)
    
    The hierarchy is critical: a kick should be closer to another kick
    than to a snare, even if the snare has similar timbre.
```

### Latent Disentanglement

```
Disentanglement separates the embedding into independent factors.

  β-VAE approach:
    L = reconstruction_loss + β * KL_divergence
    Higher β (β=4-10) forces latent dimensions to be independent.
    
  FactorVAE:
    Adds a discriminator that predicts which factor varies.
    Penalizes encoding multiple factors in same dimension.
    
  Disentanglement for one-shots:
    Each perceptual axis (timbre, transient, etc.) should be
    independently manipulable in the latent space.
    
    "Make it punchier but keep the same timbre"
      → Modify transient axis only
      → All other axes stay fixed
    
  Challenges:
    Complete disentanglement is theoretically impossible.
    But partial disentanglement (grouped factors) is achievable.
    
    Our approach: Weakly-supervised disentanglement.
      - Label pairs by which axis differs
      - "These two sounds have the same timbre but different energy"
      - Train encoder to separate axes
      - Not perfectly disentangled, but practically useful
```

---

## 4. Architecture Design

### UShOt-v1: Universal One-Shot Embedding

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│  Audio Input (mono 44.1kHz, 441ms = 16384 samples)                     │
│                                                                         │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │  Frontend: Multi-resolution spectrogram                        │    │
│  │                                                                │    │
│  │  • STFT (1024 FFT, 512 hop)       → 32 × 513 bins              │    │
│  │  • MEL (128 bands, 2048 FFT)      → 32 × 128 mel               │    │
│  │  • CQT (48 bins/oct, 7 octaves)   → 32 × 336 bins              │    │
│  │  • Transient envelope (time domain) → 16384 samples            │    │
│  │                                                                │    │
│  │  Each representation captures different information:            │    │
│  │    STFT: full spectral detail, harmonic structure               │    │
│  │    MEL: perceptual frequency, timbre content                      │    │
│  │    CQT: pitch-invariant representation, transient detail         │    │
│  │    Envelope: attack shape, decay profile, transient sharpness   │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                                   │                                     │
│  ┌────────────────────────────────▼────────────────────────────────┐    │
│  │  Encoder: HTSAT-Transformer + Temporal CNN                     │    │
│  │                                                                │    │
│  │  • HTSAT backbone (initialized from CLAP)                      │    │
│  │    - Spectral attention over mel-spectrogram                   │    │
│  │    - Output: 512d frame-level embeddings                       │    │
│  │    - Frame pooling: learnable attention across 32 frames        │    │
│  │                                                                │    │
│  │  • Temporal CNN branch                                         │    │
│  │    - 1D convolutions over raw transient envelope               │    │
│  │    - 3 layers: 64→128→256 channels, kernel=3                  │    │
│  │    - Global max pooling → 256d transient vector                │    │
│  │    - Captures attack shape that spectrogram blurs              │    │
│  │                                                                │    │
│  │  • Fusion: Cross-attention between spectral and temporal       │    │
│  │    - Spectral attends to temporal: "where is the transient?"   │    │
│  │    - Temporal attends to spectral: "what frequency is the hit?"│    │
│  │    - Output: 768d fused representation                         │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                                   │                                     │
│  ┌────────────────────────────────▼────────────────────────────────┐    │
│  │  Projection: Axis-Specific Heads + Cross-Axis Mixer            │    │
│  │                                                                │    │
│  │  Input: 768d fused representation                              │    │
│  │                                                                │    │
│  │  ┌────────────────────────────────────────────────────┐        │    │
│  │  │  Shared backbone → 256d                             │        │    │
│  │  │  • LayerNorm + Linear(768→512) + GELU               │        │    │
│  │  │  • LayerNorm + Linear(512→256)                      │        │    │
│  │  │  • Skip connection from fused rep                   │        │    │
│  │  └────────────────────────────────────────────────────┘        │    │
│  │                                                                │    │
│  │  ┌────────┬────────┬────────┬────────┬────────┬────────┬┐      │    │
│  │  │Timbre  │Transnt │Spectrl │Emotion │Genre   │Mix Pl. ││ ...  │    │
│  │  │64d     │64d      │64d      │64d      │64d      │64d      ││      │    │
│  │  └────────┴────────┴────────┴────────┴────────┴────────┴┘      │    │
│  │                                                                │    │
│  │  Each head:                                                     │    │
│  │    • Linear(256→128) + GELU + LayerNorm                        │    │
│  │    • Linear(128→64) + L2 normalize                             │    │
│  │    • Axis-specific temperature scaling                         │    │
│  │                                                                │    │
│  │  ┌────────────────────────────────────────────────────┐        │    │
│  │  │  Cross-Axis Mixture-of-Experts (128d)               │        │    │
│  │  │  • 8 experts, top-2 routing                         │        │    │
│  │  │  • Combines axis subspaces into interaction space    │        │    │
│  │  │  • Enables vector arithmetic across axes             │        │    │
│  │  └────────────────────────────────────────────────────┘        │    │
│  └────────────────────────────────────────────────────────────────┘    │
│                                   │                                     │
│                                   ▼                                     │
│                       1024d Universal Embedding                        │
│                   (256 shared + 640 axis + 128 cross)                  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Downstream Projectors

```
Each downstream task gets a lightweight projector on top of the embedding:

  Search projector:
    Linear(1024→512) — aligns with text CLAP embedding
    Trained with: contrastive audio-text pairs
    Inference: O(1) dot product with query embedding

  Recommendation projector:
   MLP(1024→256→64) → taste embedding
    Trained with: implicit feedback (export/fav/skip)
    Inference: nearest-neighbor in taste space

  Clustering projector:
    Linear(1024→32) + softmax → cluster assignment
    Trained with: self-supervised (DeepCluster)
    Inference: O(1) cluster lookup

  Generation conditioning projector:
    MLP(1024→512→512) → diffusion conditioning vector
    Trained with: embedding → reconstruction loss
    Inference: injected into UNet via cross-attention

  Morphing projector:
    Identity(1024) — operates directly in embedding space
    Morph: (1-t)*embed_a + t*embed_b → decode
    No training needed — linear interpolation works

  Similarity detector:
    Dot product(1024) — cosine similarity
    No training needed
    Inference: FAISS index query

  Pack cohesion projector:
    MLP(1024→256)→ cohesion_score
    Trained with: pack membership labels
    Inference: average embedding + cohesion check

  Taste model projector:
    MLP(1024→512→256) → personal preference score
    Trained with: user interaction history
    Inference: re-ranks search results by taste
```

---

## 5. Training Pipeline

### Phase 1: Contrastive Pretraining

```
Objective: Learn shared backbone (256d) from large unlabeled dataset.

Data: 10M+ one-shots (see Dataset Structure below)

Loss: NT-Xent (infoNCE) with mixed positive strategy

  Positive pairs (in-batch):
    - Same sound, different augment (p=0.4)
    - Same type, adjacent character (p=0.3)
    - Same genre, different type (p=0.2)
    - Same prompt, different seed (p=0.1)

  Augmentations:
    Pitch shift: ±3 semitones
    Time stretch: 0.9x-1.1x
    EQ carve: random HP/LP filter
    Gain: ±6dB
    Noise addition: -40dB floor
    Impulse response: small room convolution (10%)

Batch size: 2048 (large batch critical for contrastive learning)
Temperature: τ = 0.07 (initial), cosine warmup to 0.1
Optimizer: AdamW, lr=3e-4, weight decay=0.01
Schedule: Cosine decay, 500K steps
Hardware: 8× A100 (80GB), ~3 days

Validation metrics:
  - Recall@1 on type classification
  - NDCG@10 on similarity judgments
  - Alignment with CLAP text embeddings
```

### Phase 2: Axis-Supervised Fine-Tuning

```
Objective: Train 10 axis-specific heads (64d each) using labeled data.

Data: 500K labeled one-shots with per-axis annotations

Loss: Combination of:

  For each axis head:
    Regression axis (Timbre, Spectral, Energy):
      MSE loss between predicted and annotated axis values
    
    Classification axis (Genre, Type):
      Cross-entropy with label smoothing
    
    Ordinal axis (Realism, Emotion):
      Ordinal regression loss (CORAL)
    
    Contrastive axis (Transient, Texture, Production, Mix):
      Triplet loss with axis-specific margin

  Total loss:
    L = Σ(λ_i * L_i) for all 10 axes
    + 0.1 * contrastive_loss (backbone regularization)
    + 0.01 * L2 weight decay

  Per-axis weights (λ_i):
    Timbre:    1.0  (primary)
    Transient: 1.0  (primary)
    Spectral:  0.8  (important)
    Energy:    0.8  (important)
    Genre:     0.6  (important)
    Texture:   0.5  (medium)
    Production: 0.5  (medium)
    Mix:       0.4  (medium)
    Emotion:   0.4  (medium)
    Realism:   0.3  (lower — subjective)

Training:
  Heads trained sequentially (freeze backbone → train head → unfreeze backbone + fine-tune)
  After all heads converge: joint fine-tuning of backbone + all heads
  Total steps: 200K
  Hardware: 4× A100, ~2 days

Validation:
  Per-axis held-out test set
  Human evaluation: correlation with expert ratings
  Consistency: same sound, different labelers → variance < 0.2
```

### Phase 3: Cross-Axis Training

```
Objective: Train cross-axis MoE layer (128d).

Data: 100K sounds with multi-axis manipulation pairs.

  Pair types:
    "Same timbre, different transient"
    "Same energy, different emotion"
    "Same genre, different mix placement"
    
Loss: 
  Axis-matching loss:
    For each pair, predict which axis differs.
    "timbre_match && transient_diff && energy_match..."
    → Multi-label classification over 10 axes.
  
  Interaction prediction loss:
    Predict combined effect: "punchy + bright = aggressive"
    → Regression over cross-axis interactions.

Training: MoE with 8 experts, top-2 routing, load balancing loss
Steps: 100K
Hardware: 2× A100, ~1 day
```

### Phase 4: Downstream Adaptation

```
Each downstream task fine-tunes its projector:

  Search projector (50K steps):
    Data: 2M query-sound relevance pairs
    Loss: ListNet ranking loss
    Metric: NDCG@100

  Recommendation projector (100K steps):
    Data: 500K user interaction sequences
    Loss: Bayesian Personalized Ranking
    Metric: HitRate@10, AUC

  Clustering projector (30K steps):
    Data: unsupervised (self-training)
    Loss: KL divergence + consistency
    Metric: NMI, ARI

  Generation conditioning (200K steps):
    Data: 500K prompt-sound pairs
    Loss: L2 in latent space (embedding → reconstruction)
    Metric: FAD, CLAP score

  Total pipeline: ~7 days on 8× A100 for initial training
  Incremental updates: ~1 day per month with new data
```

---

## 6. Dataset Structure

### Core Dataset: One-Shot Collection

```
Dataset: cShot-OneShot-10M
Size: 10,000,000 one-shot sounds

Sources:
  ┌──────────────────────────┬────────────┬──────────────────────┐
  │ Source                   │ Count      │ Quality              │
  ├──────────────────────────┼────────────┼──────────────────────┤
  │ cShot existing data      │ 500K       │ ★★★★★ (user-rated)  │
  │ Licensed sample packs    │ 1M         │ ★★★★☆ (professional)│
  │ Synthetic (cShot gen)    │ 5M         │ ★★★☆☆ (clean but limited)│
  │ Freesound/CC0 sources    │ 2M         │ ★★★☆☆ (variable)    │
  │ AudioSet filtered (one-shots)│ 500K    │ ★★☆☆☆ (noisy)      │
  │ Data augmentation        │ 1M         │ ★★★☆☆ (controlled)  │
  └──────────────────────────┴────────────┴──────────────────────┘

Structure:
  ┌──────────────────────────────────────────────────────────────────┐
  │ sound_id: uuid                                                  │
  │ audio_path: path/to/sound.wav                                   │
  │ duration_ms: 412                                                │
  │ sample_rate: 44100                                              │
  │ channels: 1 (mono)                                              │
  │                                                                  │
  │ metadata:                                                       │
  │   source: "cshot_generated" / "licensed" / "freesound"          │
  │   type: "kick" / "snare" / "hi-hat" / "clap" / "808" / "perc"  │
  │   subtype: "punchy_kick" / "deep_kick" / "trap_hat"             │
  │   genre_affinity: ["trap", "hip-hop"]                            │
  │   bpm: 140                                                      │
  │   key: "F#m" (optional)                                          │
  │   license: "cc0" / "cshot_commercial" / "restricted"            │
  │   prompt: "punchy trap kick 140bpm" (if generated)              │
  │   model_id: "elevenlabs-sfx-2.0" (if generated)                 │
  │                                                                  │
  │ axis_labels: (10 axes, each 0.0-1.0)                           │
  │   timbre: 0.85      # synthetic vs acoustic                     │
  │   transient: 0.72   # soft vs aggressive                        │
  │   spectral_balance: 0.43  # dark vs bright                       │
  │   emotion: 0.61     # cold vs warm                              │
  │   genre_affinity: 0.33  # trap affinity                         │
  │   mix_placement: 0.52   # back vs forward                        │
  │   production_style: 0.78  # raw vs designed                      │
  │   texture: 0.39     # smooth vs gritty                          │
  │   realism: 0.88     # synthetic vs organic                      │
  │   energy: 0.65      # gentle vs intense                         │
  │                                                                  │
  │ pairwise_data: (optional, for contrastive training)             │
  │   similar_to: [uuid, uuid, ...]                                 │
  │   dissimilar_to: [uuid, uuid, ...]                              │
  │   same_axis_different: {"transient": uuid, "timbre": uuid}     │
  │   human_rating: 4.2                                             │
  └──────────────────────────────────────────────────────────────────┘
```

### Labeling Strategy

```
Automated labeling (70% of data):
  ┌──────────────────────────────────────────────┬──────────────────┐
  │ Method                                       │ Axes              │
  ├──────────────────────────────────────────────┼──────────────────┤
  │ Signal processing heuristics                 │ Energy, Transient│
  │ Spectral analysis + model                    │ Spectral, Timbre  │
  │ Genre classifier (pretrained)                │ Genre              │
  │ Production style classifier                  │ Production, Mix   │
  │ CLAP text projection (zero-shot)            │ Emotion, Texture  │
  │ SoundScore + metrics                         │ Realism, Quality  │
  └──────────────────────────────────────────────┴──────────────────┘

Human labeling (30% of data — gold standard):
  ┌──────────────────────────────────────────────┬──────────────────┐
  │ Method                                       │ Axes              │
  ├──────────────────────────────────────────────┼──────────────────┤
  │ Expert producers (100 labelers, paid)        │ All 10 axes       │
  │ Pairwise comparison ("which is punchier?")   │ Transient, Energy │
  │ Category selection ("choose genre")          │ Genre              │
  │ Free text description                        │ All (text proxy)  │
  │ Likert scale (1-7 per axis)                  │ All 10 axes       │
  └──────────────────────────────────────────────┴──────────────────┘

  Human labeling cost:
    500K sounds × 10 axes × 3 labelers ÷ 100 labelers
    = 42.5 hours of labeling per axis
    = 425 hours total
    = ~$8,500 at $20/hr

Quality control:
  - Inter-rater reliability: Cohen's κ > 0.7 per axis
  - Gold standard: 10K sounds with 10 labelers each (majority vote)
  - Active learning: label most uncertain sounds first
  - Rater calibration: periodic anchor sound checks
```

---

## 7. Embedding Dimensionality Analysis

### Why 1024?

```
Tradeoff analysis:

  D=128  ("tiny"):
    Pros: Fast indexing, low memory (50M vectors = 24GB)
    Cons: Cannot capture 10 independent axes (12.8d/axis)
    Limitation: Collapses perceptual distinctions
    Use: Mobile/edge deployment only
    Verdict: ✓ Acceptable for on-device — ✗ Insufficient for platform

  D=256  ("small"):
    Pros: Good compression, reasonable recall
    Cons: 25.6d/axis — moderate separation
    Limitation: Cross-axis interactions compressed
    Use: Lightweight search index
    Verdict: ✓ Acceptable for fast search — ✗ Weak for generation conditioning

  D=512  ("medium"):
    Pros: 51.2d/axis — good separation
    Cons: Cross-axis interactions crowded
    Limitation: MoE layer needs more space
    Use: Good general-purpose embedding
    Verdict: ✓ Good baseline — ~ Acceptable compromise

  D=1024  ("recommended"):
    Pros: 102.4d/axis — excellent separation
         256d shared backbone
         128d cross-axis interactions
    Cons: 50M vectors = 200GB (mitigated by PQ to 16GB)
    Use: Full platform embedding
    Verdict: ✓ Optimal for quality — ✓ Manageable with quantization

  D=2048  ("large"):
    Pros: Very high capacity
    Cons: Overfitting risk, slow indexing, dim curse
    Use: Not needed — 1024 captures enough with fine-tuned heads
    Verdict: ✗ Diminishing returns — ~ Not worth the cost

D=1024 is the sweet spot: enough capacity for 10 disentangled axes
plus cross-axis interactions, without excessive storage cost.
```

### Storage & Query Estimates

```
Index type: FAISS IVF-PQ (Inverted File with Product Quantization)

At 50M vectors (cShot scale year 1):
  ┌────────────────────────────┬──────────┬─────────────┐
  │ Metric                     │ D=1024   │ D=1024+PQ   │
  ├────────────────────────────┼──────────┼─────────────┤
  │ Raw storage                │ 200 GB   │ 200 GB      │
  │ With PQ (M=128, nbits=8)   │ —        │ 12.5 GB     │
  │ IVF index (nlist=100K)     │ 50 GB    │ 10 GB       │
  │ Total index memory         │ 250 GB   │ 22.5 GB     │
  │ Query latency (recall=0.9) │ 2ms      │ 5ms         │
  │ Query throughput (1 GPU)   │ 50K QPS  │ 100K QPS    │
  │ Index build time           │ 4 hrs    │ 2 hrs       │
  └────────────────────────────┴──────────┴─────────────┘

  Recommendation: Use IVF-PQ with M=128 products.
    Recall @ 100: 0.92 (vs brute force)
    Latency: 5ms per query
    Memory: 22.5GB for 50M vectors — fits on 1 A100

  Multi-index strategy:
    ┌──────────────────────────────────────────────────────────────┐
    │  Hot index (10M recent/popular):  IVF-PQ, 4.5GB, <1ms      │
    │  Cold index (40M archived): IVF-PQ, 18GB, 5ms              │
    │  Search: query hot + cold, merge results                    │
    │  Write: append to hot, background merge to cold             │
    └──────────────────────────────────────────────────────────────┘
```

---

## 8. Nearest-Neighbor Systems

### FAISS-Based Search Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Query Types:                                                       │
│                                                                     │
│  1. Text-to-Sound    : "punchy trap kick" → text_embed(text)       │
│                           → nearest neighbor in embedding space    │
│                                                                     │
│  2. Sound-to-Sound   : "like this kick" → embed(reference)          │
│                           → nearest neighbor (exclude self)        │
│                                                                     │
│  3. Hybrid (text + ref): weighted avg of text_embed + sound_embed  │
│                           → nearest neighbor                       │
│                                                                     │
│  4. Semantic vector   : embed( kick_A ) - embed( kick_B )          │
│     arithmetic         + embed( punchy_clap )                       │
│                           → "make this punchier" → nearest neighbor│
│                                                                     │
│  5. Multi-axis query  : timbre=0.8, transient=0.3, energy=0.9      │
│                           → compose axis embeddings → nearest      │
│                                                                     │
│  6. Taste-filtered    : user_embed(user) + query_embed              │
│                           → nearest neighbor with taste bias        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

Index hierarchy:
  ┌──────────────┐
  │  Query       │
  └──────┬───────┘
         │
  ┌──────▼───────┐
  │  Router      │  ← determines query type
  └──────┬───────┘
         │
  ┌──────▼──────────────────────────────────────────────────────┐
  │  FAISS Index (IVF-PQ, D=1024)                               │
  │                                                              │
  │  ┌──────────────────────────────────┐  ┌──────────────────┐  │
  │  │  Hot Index (10M, GPU)            │  │  Cold Index      │  │
  │  │  • IVF with 20K centroids       │  │  (40M, CPU)      │  │
  │  │  • PQ M=64 for 4.5GB total       │  │  • IVF-PQ M=128  │  │
  │  │  • <1ms latency                  │  │  • 5ms latency   │  │
  │  └──────────────────────────────────┘  └──────────────────┘  │
  │                                                              │
  │  ┌──────────────────────────────────────────────────────┐    │
  │  │  Merge Results: rank by cosine similarity            │    │
  │  │  Re-rank (top 100): user taste bias + freshness      │    │
  │  └──────────────────────────────────────────────────────┘    │
  └──────────────────────────────────────────────────────────────┘
         │
  ┌──────▼───────────────┐
  │  Results (top 100)   │
  └──────────────────────┘
```

### Multi-Index Query Router

```
def route_query(query_type, text=None, sound=None, user=None):
    if query_type == "text":
        q = text_encoder(text)  # 1024d
    elif query_type == "sound":
        q = audio_encoder(sound)  # 1024d
    elif query_type == "hybrid":
        q = 0.7 * text_encoder(text) + 0.3 * audio_encoder(sound)
    elif query_type == "multiaxes":
        q = compose_axis_embeddings(axes_dict)
    elif query_type == "taste":
        q = 0.8 * audio_encoder(sound) + 0.2 * user_embed(user)
    
    # Optional: reweight by axis
    if axis_weights:
        q = apply_axis_weights(q, axis_weights)  
        # "more metallic" = increase timbre_metallic dimension
    
    # Query hot index
    hot_results = hot_index.search(q, k=100)
    
    # Query cold index if needed
    if hot_results.confidence < threshold or need_more_results:
        cold_results = cold_index.search(q, k=50)
        all_results = merge(hot_results, cold_results)
    else:
        all_results = hot_results
    
    # Re-rank with taste
    if user:
        all_results = taste_ranker.rerank(all_results, user)
    
    return all_results[:50]
```

---

## 9. Semantic Vector Arithmetic

### How It Works

```
The embedding space supports vector arithmetic — a property of
contrastive learning where directions encode meaningful semantic axes.

In embedding space:
  vec("punchy kick") - vec("deep kick") ≈ direction("punchier")
  vec("aggressive snare") - vec("soft snare") ≈ direction("aggressive")
  
  So:  vec("current sound") + direction("punchier")
       ≈ vec("current sound but punchier")

  Example:
    current = embed(user_sound)
    punchiness = embed("punchy kick") - embed("soft kick")
    result = current + 0.3 * punchiness
    
    nearest_neighbor(result) → sound like current but punchier
    decode_from_embedding(result) → generated sound like current but punchier

Axis-specific arithmetic:
  Modify TIMBRE only:
    metallic = timbre_head("metallic sound") - timbre_head("warm sound")
    result = embed(sound)
    result[timbre_slice] += 0.5 * metallic
    
  Modify TRANSIENT only:
    punchier = transient_head("punchy") - transient_head("soft")
    result = embed(sound)
    result[transient_slice] += 0.3 * punchier
    
  Combined:
    result = embed(sound)
    result[transient_slice] += 0.3 * punchier
    result[spectral_slice] -= 0.2 * brightness  # "make darker"
    result[energy_slice] += 0.1 * intensity
```

### Transform Library (Precomputed Directions)

```
Precomputed semantic directions (computed from labeled data):

  ┌───────────────────────┬────────────────────────────────┐
  │ Direction             │ Source vectors                  │
  ├───────────────────────┼────────────────────────────────┤
  │ punchier              │ avg(punchy_kicks) - avg(kicks) │
  │ softer                │ avg(soft_kicks) - avg(kicks)    │
  │ brighter              │ avg(bright) - avg(dark)        │
  │ darker                │ avg(dark) - avg(bright)        │
  │ warmer                │ avg(warm) - avg(cold)          │
  │ more metallic         │ avg(metallic) - avg(wooden)    │
  │ more expensive        │ avg(produced) - avg(raw)       │
  │ more aggressive       │ avg(aggressive) - avg(gentle)  │
  │ more cinematic        │ avg(cinematic) - avg(neutral)  │
  │ more vintage          │ avg(vintage) - avg(modern)     │
  │ more realistic        │ avg(organic) - avg(synthetic)  │
  │ more textured         │ avg(gritty) - avg(smooth)      │
  │ more forward (mix)    │ avg(forward) - avg(back)       │
  │ trap-oriented         │ avg(trap_genre) - avg(all)     │
  │ house-oriented        │ avg(house_genre) - avg(all)    │
  └───────────────────────┴────────────────────────────────┘

  Each direction is a 1024d unit vector.
  Users can compose: result = current + 0.5*punchier + 0.3*darker - 0.2*metallic
  
  Composability is the killer feature:
    "I want this sound but punchier, darker, and more expensive sounding."
    → literal vector arithmetic → 5ms → new sound suggestion
```

---

## 10. Evaluation Methods

### Automated Evaluation

```
1. Type Classification Accuracy
   Task: classify into 10 types (kick, snare, hat, clap, 808, tom, perc, fx, impact, texture)
   Metric: Top-1 accuracy, macro F1
   Baseline: CLAP: 0.72, VGGish: 0.68, UShOt-v1 target: 0.92
   
2. Similarity Retrieval (Recall@K)
   Task: given a query sound, retrieve similar sounds from held-out set
   Metric: Recall@1, Recall@10, NDCG@100
   Baseline (human-labeled similar pairs): CLAP R@1=0.31, UShOt target R@1=0.65
   
3. Semantic Consistency
   Task: verify that vector arithmetic produces consistent results
   Metric: 
     - "punchy kick" - "soft kick" + "soft snare" should ≈ "punchy snare"
     - Measured by cosine similarity to expected result
   Baseline: random = 0.0, target: 0.6+ consistency score

4. Axis Disentanglement
   Task: modify one axis, verify other axes remain unchanged
   Metric: 
     - Mutual Information Gap (MIG): higher = more disentangled
     - DCI Disentanglement score: higher = better
   Target: MIG > 0.4, DCI > 0.6 (state of art for audio)

5. Cross-Modal Alignment
   Task: text-to-audio retrieval
   Metric: R@1, R@5 for text queries
   Baseline: CLAP R@1=0.45, UShOt target R@1=0.55

6. Generation Conditioning Quality
   Task: use embedding as conditioning for diffusion model
   Metric: FAD (Fréchet Audio Distance), CLAP score
   Target: FAD < 2.0, CLAP score > 0.35
```

### Human Evaluation

```
1. Perceptual Axis Alignment
   Method: Users rate 100 sounds on each axis (Likert 1-7)
   Metric: Spearman correlation between model prediction and human rating
   Target: ρ > 0.8 for all axes

2. Similarity Judgment
   Method: A/B test: "Which sound is more similar to the reference?"
   Metric: Agreement between model ranking and human ranking
   Target: 85%+ agreement

3. Semantic Navigation
   Method: "Make this sound punchier" — rate 5 results
   Metric: % of results that are genuinely punchier (per human)
   Target: 80%+ success rate

4. Pack Cohesion
   Method: Generate 20-sound pack. Humans rate cohesion.
   Metric: Average cohesion score (1-10)
   Target: 7.5+ (professional pack quality)

5. Taste Alignment
   Method: Users train model on 50 favorites. Rate 20 recommendations.
   Metric: % "I would use this sound" (like/export rate)
   Target: 40%+ engagement (baseline random: 5-10%)
```

---

## 11. Embedding Serving Infrastructure

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│  ┌──────────────────────────────────────────────────┐       │
│  │  Embedding Service                                 │       │
│  │                                                    │       │
│  │  ┌────────────────┐  ┌─────────────────────────┐   │       │
│  │  │  Inference GPU  │  │  FAISS Index (GPU)      │   │       │
│  │  │  (T4/A10)      │  │  • IVF-PQ 50M vectors    │   │       │
│  │  │  • HTSAT encoder │  │  • 5ms query latency    │   │       │
│  │  │  • Axis heads   │  │  • 22.5GB memory        │   │       │
│  │  │  • 10ms latency │  │                          │   │       │
│  │  └────────────────┘  └─────────────────────────┘   │       │
│  │                                                    │       │
│  │  ┌────────────────┐  ┌─────────────────────────┐   │       │
│  │  │  Text Encoder  │  │  Cache (Redis)          │   │       │
│  │  │  (CPU)         │  │  • LRU 1M embeddings    │   │       │
│  │  │  • DistilBERT  │  │  • TTL: 24h             │   │       │
│  │  │  • 5ms latency │  │  • 8GB RAM              │   │       │
│  │  └────────────────┘  └─────────────────────────┘   │       │
│  └──────────────────────────────────────────────────┘       │
│                                                             │
│  ┌──────────────────────────────────────────────────┐       │
│  │  Write Path                                        │       │
│  │                                                    │       │
│  │  Every new sound → embed → append to:             │       │
│  │    • Hot index (immediate, GPU)                   │       │
│  │    • Write-ahead log (durability)                 │       │
│  │    • Background: merge into cold index            │       │
│  │    • Cache for 24h (same sound rechurned)         │       │
│  └──────────────────────────────────────────────────┘       │
│                                                             │
│  ┌──────────────────────────────────────────────────┐       │
│  │  Update Path                                       │       │
│  │                                                    │       │
│  │  When model improves:                              │       │
│  │    • Re-embed all sounds (background job, 2 days) │       │
│  │    • Rebuild FAISS index (4 hours)                │       │
│  │    • Switch index atomically (blue-green)         │       │
│  │    • Old index kept for 7 days (rollback)         │       │
│  └──────────────────────────────────────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 12. Implementation Roadmap

```
Phase 1 — Research (2 months):
  ✓ Build labeling pipeline + label 500K sounds
  ✓ Train contrastive backbone on 10M sounds (Phase 1)
  ✓ Fine-tune 10 axis heads (Phase 2)
  ✓ Evaluate on all metrics
  ✓ Validation: ρ > 0.8 correlation with human ratings

Phase 2 — Infrastructure (1 month):
  ✓ Build embedding service (GPU inference)
  ✓ Deploy FAISS index (hot + cold)
  ✓ Build write path (embed new sounds on generation)
  ✓ Build update path (re-embed on model update)
  ✓ Latency: <10ms for embed, <5ms for search

Phase 3 — Integration (1 month):
  ✓ Replace CLAP with UShOt-v1 in search
  ✓ Replace CLAP with UShOt-v1 in generation conditioning
  ✓ Build semantic vector arithmetic API
  ✓ Build taste projector
  ✓ Build pack cohesion projector
  ✓ A/B test: UShOt-v1 vs CLAP for all downstream tasks

Phase 4 — Iteration (ongoing):
  ✓ Monthly embedding model update with new data
  ✓ Quarterly human evaluation
  ✓ Continuous active learning for labeling
  ✓ Axis refinement based on user feedback
  ✓ Dimensionality reduction for edge deployment (256d variant)

Total timeline: ~4 months to full deployment
```

---

## 13. Summary

```
UShOt-v1: Universal One-Shot Embedding

  Dimensions:    1024 (256 shared + 10×64 axis + 128 cross)
  Model:         HTSAT transformer + temporal CNN + axis heads + MoE
  Training:      4-phase pipeline (contrastive → supervised → cross-axis → downstream)
  Data:          10M one-shots with 500K gold-labeled by axes
  Index:         FAISS IVF-PQ, 50M vectors, 5ms query, 22.5GB memory
  Search:        Text, sound, hybrid, semantic arithmetic, taste-filtered
  Downstreams:   Search, recommendation, clustering, morphing, generation,
                 similarity, pack creation, taste modeling

  Key insight:
    A SINGLE embedding space that captures all perceptual + production
    dimensions enables every downstream system to share understanding.
    Taste feedback improves search. Similarity data improves generation.
    Every system benefits from every other system's data.

  The embedding isn't a feature — it's the shared language of the platform.
```

