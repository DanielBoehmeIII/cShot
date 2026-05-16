# Prompt 83 — Fine-Tune for One-Shots

A fine-tuning strategy specifically for one-shot audio generation. Every decision optimized for short, percussive, mix-ready sounds.

---

## 1. Dataset Requirements

### One-Shot Dataset Specification

```
Dataset size:      Minimum 10,000 one-shots, target 100,000+
Duration range:    0.1s – 10s (99th percentile: <5s)
Sample rate:       44,100 Hz (target), 22,050 Hz (acceptable)
Channels:          Mono (preferred), stereo (downmixed for training)
Bit depth:         16-bit or 24-bit
Format:            WAV (uncompressed preferred)

Genre distribution:
  Kick drums:      15%        (sub, 808, acoustic, electronic, layered)
  Snares:          12%        (acoustic, trap, rimshot, clap/snare hybrid)
  Hi-hats:         10%        (closed, open, pedal, ride bell)
  Percussion:      10%        (shakers, tambourines, congas, bongos, claves)
  Claps:           8%         (single, ensemble, processed, layered)
  Cymbals:         8%         (crash, ride, splash, china)
  Toms:            5%         (high, mid, floor, electronic)
  FX / Impacts:    10%        (risers, downlifters, impacts, sweeps)
  Foleys:          5%         (rustles, clicks, pops, breaths)
  Synth one-shots: 10%        (plucks, stabs, blips, hits)
  Textures:        5%         (pads, noise beds, atmospheres — short)
  Processed:       2%         (reversed, glitched, heavily processed)
```

### Quality Requirements

```
Critical (must pass for inclusion):
  ✓ No clipping (peak < -0.1 dBFS)
  ✓ No silence >1s at start (trimmed)
  ✓ No silence >2s at end (fade out)
  ✓ No DC offset (>0.01)
  ✓ 16-bit or higher bit depth
  ✓ Full bandwidth (no LPF below 16 kHz unless intentional)
  ✓ Sample rate >= 22,050 Hz

Important (strongly preferred):
  ✓ Consistent loudness (-18 LUFS integrated target)
  ✓ Clean transient start (no pre-ringing)
  ✓ Named with semantic description
  ✓ Genre-labeled
  ✓ Key-labeled (for tonal sounds)

Nice to have:
  ✓ Recording environment noted
  ✓ Processing chain documented
  ✓ Original source noted (synthesizer model, microphone, etc.)
  ✓ Multiple velocity layers
```

### Dataset Sources

```
1. Licensed sample packs (primary)
   - Splice, Loopmasters, Producer Loops, Noiiz
   - Focus: packs that include raw/unprocessed one-shots
   - License: verified royalty-free for training

2. Public domain / Creative Commons
   - Freesound.org (CC0 / CC-BY)
   - Philharmonia Orchestra samples
   - University of Iowa Musical Instrument Samples
   - VSCO-2 CE

3. Synthetic generation
   - Synthetically generated kicks (sine + envelope models)
   - Algorithmically generated percussion (Gabor noise, FM)
   - Physical modeling (modal synthesis, waveguide)
   - These are free of copyright concerns

4. Partnership / direct licensing
   - Direct deals with sample pack creators
   - Revenue sharing for packs used in training
   - Transparent attribution system
```

---

## 2. Labeling Strategy

### Label Taxonomy

```
ONE-SHOT LABELS (required field: type)

type: [kick, snare, hihat, clap, cymbal, tom, perc, fx, foley, synth, texture]
  ├── kick
  │   ├── sub_kick
  │   ├── acoustic_kick
  │   ├── electronic_kick
  │   ├── 808_kick
  │   ├── layered_kick
  │   └── processed_kick
  │
  ├── snare
  │   ├── acoustic_snare
  │   ├── trap_snare
  │   ├── rimshot
  │   ├── clap_snare
  │   └── processed_snare
  │
  ├── hihat
  │   ├── closed_hihat
  │   ├── open_hihat
  │   ├── pedal_hihat
  │   └── processed_hihat
  │
  ├── perc
  │   ├── shaker
  │   ├── tambourine
  │   ├── conga
  │   ├── bongo
  │   ├── clave
  │   ├── cowbell
  │   ├── triangle
  │   └── maraca
  │
  ├── fx
  │   ├── impact
  │   ├── riser
  │   ├── downlifter
  │   ├── sweep
  │   └── whoosh
  │
  ├── synth
  │   ├── pluck
  │   ├── stab
  │   ├── blip
  │   └── hit
  │
  └── texture
      ├── noise_bed
      ├── rustle
      └── atmosphere


PRODUCTION LABELS (recommended)

genre: [hiphop, trap, edm, house, techno, dubstep, pop, rock, lo-fi, cinematic, jazz, world, experimental, drum_and_bass, footwork, ambient]

mood: [dark, bright, warm, cold, aggressive, soft, punchy, airy, thick, thin, clean, dirty, vintage, modern]

character: [dry, wet, tight, loose, resonant, muted, ringy, thuddy, clicky, boomy, crisp, dull, sharp, round, metallic, wooden, plastic, glass]


TECHNICAL LABELS (computed/auto)

  envelope: [short_decay, long_decay, sustained, percussive, swelling]
  dynamics: [quiet, moderate, loud, very_loud]
  spectral_centroid: [low, mid, high] (Hz ranges)
  spectral_rolloff: [low, mid, high]
  zero_crossing_rate: [low, medium, high]
  tempo (BPM): integer or null
  key: [C, C#, D, ..., B] or null
  loudness: LUFS integrated value
  peak: dBFS value
  duration: milliseconds
  transient_energy_ratio: float (0-1)
  noise_tonal_ratio: float (0-1)
```

### Labeling Pipeline

```
                        RAW AUDIO FILES
                              │
                              ▼
┌─────────────────────────────────────────────┐
│               AUTO-LABELING                  │
│                                              │
│  Step 1: Audio Analysis Engine               │
│    • Duration, sample rate, channels         │
│    • RMS, peak, LUFS loudness                │
│    • Spectral centroid, rolloff, flux        │
│    • Zero-crossing rate, autocorrelation     │
│    • Envelope detection (ADSR parameters)    │
│    • Transient detection, onset strength     │
│    • Noise/tonal decomposition               │
│    • Pitch detection (if tonal)              │
│                                              │
│  Step 2: Classifier Ensemble                 │
│    • Type classifier (kick/snare/hat/etc.)   │
│    • Subtype classifier (if type known)      │
│    • Genre classifier                        │
│    • Mood classifier                         │
│    • Instrument classifier (if applicable)   │
│    • Quality score (production readiness)    │
│                                              │
│  Step 3: Initial label set                   │
│    ~30 technical + 5-10 semantic labels      │
│    ~85% accuracy from auto-labeling          │
└─────────────────────┬───────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│            HUMAN LABELING (Crowd/Expert)      │
│                                              │
│  Step 4: Verification                        │
│    • Expert review of auto labels            │
│    • Fix misclassifications                  │
│    • Add missing semantic labels             │
│    • Quality check (remove bad samples)      │
│                                              │
│  Step 5: Prompt Writing                     │
│    • For each one-shot: write 3-5 prompts    │
│    • Descriptive: "a punchy kick with        │
│      sub-bass and a quick attack"            │
│    • Comparative: "like a 909 kick but       │
│      with more low-end body"                 │
│    • Production: "trap kick, hard transient, │
│      40Hz sub, short decay"                  │
│                                              │
│  Step 6: Quality gate                        │
│    • Two expert raters per sample            │
│    • Label confidence score (0-1)            │
│    • Reject if <0.8 agreement                │
└─────────────────────┬───────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│              FINAL LABEL SET                 │
│                                              │
│  ~50 labels per sound total:                 │
│    • 10 auto-technical                       │
│    • 5 auto-semantic                         │
│    • 10 verified-semantic                    │
│    • 5 verified-technical                    │
│    • 5 human-written prompts                 │
│    • 15 computed features (cache)            │
└─────────────────────────────────────────────┘
```

---

## 3. Conditioning Signals

### Multi-Conditioning Architecture

```
Input prompt ───────────┐
                        ▼
              ┌──────────────────┐
              │  Text Encoder    │
              │  (CLAP / T5 /    │
              │   FLAN-T5)       │
              └────────┬─────────┘
                       │ text_embedding (512-1024d)
                       │
Target genre ─────┐    │
Target mood ──────┤    │
Target BPM ───────┤    ├─────► Cross-attention to UNet / DiT
Target key ───────┤    │
Target duration ──┘    │
                       │
Target type ─────────┐ │
                     │ │
              ┌──────▼─▼──────────┐
              │  Condition Mixer  │
              │  (learned blend)  │
              └────────┬─────────┘
                       │ conditioning_vector
                       ▼
              ┌──────────────────┐
              │  Generator       │
              │  (Diffusion /    │
              │   VAE / GAN)     │
              └────────┬─────────┘
                       │ audio_output
                       ▼
              ┌──────────────────┐
              │  Post-Processor  │
              │  (trim, normalize│
              │   fade, LUFS)    │
              └──────────────────┘
```

### Text Conditioning

```python
# Text encoder: CLAP (Contrastive Language-Audio Pretraining)
# CLAP projects text and audio into shared embedding space
# Fine-tune the text encoder alongside the generator

text_encoder = CLAPTextEncoder()
# Freeze early layers, fine-tune last 2 transformer blocks

# Prompt template augmentation during training:
prompts = [
    # Direct description
    "kick drum, punchy, 40Hz sub, fast attack",
    "a punchy kick drum with a 40Hz sub and fast attack",

    # Comparative
    "909 kick, but with more sub presence",
    "like an 808 kick but tighter",

    # Production context
    "trap kick for a hard-hitting beat",
    "lo-fi kick, warm and round",

    # Abstract
    "dark kick that feels heavy",
    "bright kick with click attack",
]

# Multiple prompts per sound → improves generalization
# Up to 10x augmentation of text-side data
```

### Genre Conditioning

```python
# Genre as one-hot or learned embedding
genres = [
    "hiphop", "trap", "edm", "house", "techno",
    "dubstep", "pop", "rock", "lo-fi", "cinematic",
    "jazz", "drum_and_bass", "ambient", "experimental"
]

genre_embedding = nn.Embedding(num_genres, 64)
# Each genre learns a 64-dim conditioning vector
# Mixed with text embedding via learned gating:
conditioning = text_embedding * 0.7 + genre_embed * 0.3
```

### Parametric Conditioning

```python
# Explicit parameter conditioning for controllable generation
parameters = {
    "duration": float,          # 0.1 – 10.0 seconds
    "pitch": float,             # MIDI note number (0-127)
    "velocity": float,          # 0.0 – 1.0
    "low_cut": float,           # Hz
    "high_cut": float,          # Hz
    "decay": float,             # 0.0 – 1.0 (envelope decay time)
    "body": float,              # 0.0 – 1.0 (low-mid content)
    "attack": float,            # 0.0 – 1.0 (transient sharpness)
    "noise_content": float,     # 0.0 – 1.0
    "stereo_width": float,      # 0.0 – 1.0
}

# Encode as learned Fourier features (NeRF-style positional encoding)
# → allows fine-grained continuous control
```

### Conditioning Dropout (Classifier-Free Guidance)

```python
# During training: randomly drop conditioning signals
# → enables classifier-free guidance at inference

def apply_conditioning_dropout(conditioning_dict, drop_rate=0.1):
    for key in conditioning_dict:
        if random.random() < drop_rate:
            conditioning_dict[key] = NULL_EMBEDDING  # learned null token
    return conditioning_dict

# At inference: guide toward condition via:
# guidance_scale = 3.0
# output = unconditional_output + guidance * (conditional_output - unconditional_output)
```

---

## 4. Augmentation

### Audio Augmentation (On-the-Fly)

```python
class OneShotAugmentation:
    """
    Augmentation pipeline for one-shot training.
    All augmentations are applied in the waveform domain
    and must preserve the essential character of the sound.
    """

    def __call__(self, audio: torch.Tensor, sample_rate: int = 44100) -> torch.Tensor:
        # 1. Time-domain augmentations (applied always)
        audio = self.time_shift(audio, max_shift_ms=50)        # small timing variance
        audio = self.trim_silence(audio, threshold_db=-40)     # consistent start
        audio = self.random_gain(audio, gain_db=(-3, 3))       # level variance

        # 2. Spectral augmentations (applied probabilistically)
        if random.random() < 0.3:
            audio = self.random_eq(audio, sr=sample_rate)       # gentle EQ carve
        if random.random() < 0.2:
            audio = self.random_filter(audio, sr=sample_rate)   # random HP/LP
        if random.random() < 0.1:
            audio = self.add_harmonics(audio, sr=sample_rate)   # subtle saturation

        # 3. Perceptual augmentations (maintains character)
        if random.random() < 0.15:
            audio = self.pitch_shift(audio, sr=sample_rate,     # ±2 semitones
                                      n_steps=random.uniform(-2, 2))
        if random.random() < 0.1:
            audio = self.time_stretch(audio, sr=sample_rate,    # ±10% tempo
                                       rate=random.uniform(0.9, 1.1))

        # 4. Noise augmentations (teaches robustness)
        if random.random() < 0.05:
            audio = self.add_noise(audio, snr_db=random.uniform(30, 50))
        if random.random() < 0.02:
            audio = self.add_reverb(audio, sr=sample_rate,
                                     room_size=random.uniform(0.1, 0.3))

        return audio
```

### Text-Side Augmentation

```python
def augment_prompt(prompt: str, label_dict: dict) -> str:
    """Generate alternative phrasings of the same sound description."""

    templates = [
        "{type}, {adjective}, {params}",
        "a {adjective} {type} with {params}",
        "{type} sound, {adjective}, {params}",
        "{adjective} {type} for {genre}",
        "like a {type} but {params}",
    ]

    template = random.choice(templates)

    return template.format(
        type=label_dict["type"],
        adjective=random.choice(label_dict["adjectives"]),
        params=random.choice(label_dict["param_descriptions"]),
        genre=label_dict.get("genre", ""),
    )

# Example:
# Input: ("punchy kick, 40Hz, fast attack", {type: "kick", adjectives: ["punchy", "tight"], ...})
# Output: "a tight kick with fast attack and 40Hz sub"
# Output: "punchy kick sound for trap"
# Output: "like a kick but with fast attack"
```

### Latent Augmentation

```python
# Interpolation between latent codes of same-class sounds
# → teaches the model that "kick" class has a continuous manifold

batch_a = encode(audio_1)  # kick A
batch_b = encode(audio_2)  # kick B
alpha = random.uniform(0.0, 1.0)
interpolated = alpha * batch_a + (1 - alpha) * batch_b
loss = reconstruction_loss(decoder(interpolated), audio_1)
# → teaches smooth interpolation within sound class
```

---

## 5. Loss Functions

### Primary Losses

```python
# Total loss = weighted combination
total_loss = (
    λ_diffusion * diffusion_loss +
    λ_reconstruction * reconstruction_loss +
    λ_adversarial * adversarial_loss +
    λ_feature * feature_matching_loss +
    λ_clap * clap_similarity_loss +
    λ_consistency * consistency_loss
)
```

### Diffusion Loss (for diffusion-based generators)

```python
# Standard noise-prediction loss
def diffusion_loss(model, x_0, condition, noise_schedule):
    """
    x_0: clean audio
    condition: conditioning vector (text + params)
    noise_schedule: β_t values
    """
    t = sample_timestep()                          # random timestep
    noise = torch.randn_like(x_0)                  # sample noise
    x_t = add_noise(x_0, noise, t, noise_schedule) # noisy version

    noise_pred = model(x_t, t, condition)          # predict noise

    # Simple L2 loss on noise prediction
    loss = F.mse_loss(noise_pred, noise)

    # Optional: frequency-weighted MSE (emphasize perceptually important bands)
    if use_freq_weighted:
        loss = frequency_weighted_mse(noise_pred, noise)

    return loss
```

### CLAP Similarity Loss

```python
def clap_similarity_loss(generated_audio, prompt_text):
    """
    Encourage generated audio to match prompt semantics.
    Uses frozen CLAP model as perceptual judge.
    """
    audio_embed = clap_audio_encoder(generated_audio)  # [B, D]
    text_embed = clap_text_encoder(prompt_text)         # [B, D]

    # Normalize embeddings
    audio_embed = F.normalize(audio_embed, dim=-1)
    text_embed = F.normalize(text_embed, dim=-1)

    # Cosine similarity (higher is better)
    similarity = (audio_embed * text_embed).sum(dim=-1)

    # Loss = 1 - similarity (lower is better)
    return (1 - similarity).mean()
```

### Feature Matching Loss

```python
def feature_matching_loss(real_audio, generated_audio, discriminator):
    """
    Match features from discriminator's intermediate layers.
    Prevents mode collapse, improves perceptual quality.
    """
    real_feats = discriminator.get_features(real_audio)
    fake_feats = discriminator.get_features(generated_audio)

    loss = 0
    for real_f, fake_f in zip(real_feats, fake_feats):
        loss += F.l1_loss(fake_f, real_f.detach())

    return loss
```

### Consistency Loss

```python
def consistency_loss(audio, aug_audio, condition):
    """
    Augmented version should produce same embedding as original.
    Teaches invariance to small perturbations.
    """
    embed = encoder(audio)
    aug_embed = encoder(aug_audio)
    return F.mse_loss(embed, aug_embed.detach())
```

### Loss Weight Schedule

```
Training Phase 1 (200k steps): Focus on reconstruction
  diffusion_loss:          λ=1.0
  reconstruction_loss:    λ=0.5
  adversarial_loss:       λ=0.0
  feature_matching_loss:  λ=0.0
  clap_similarity_loss:   λ=0.1
  consistency_loss:       λ=0.0

Training Phase 2 (100k steps): Add adversarial
  diffusion_loss:          λ=1.0
  reconstruction_loss:    λ=0.2
  adversarial_loss:       λ=0.2
  feature_matching_loss:  λ=0.5
  clap_similarity_loss:   λ=0.3
  consistency_loss:       λ=0.1

Training Phase 3 (50k steps): Semantic alignment
  diffusion_loss:          λ=1.0
  reconstruction_loss:    λ=0.1
  adversarial_loss:       λ=0.1
  feature_matching_loss:  λ=0.2
  clap_similarity_loss:   λ=1.0  ← emphasize prompt adherence
  consistency_loss:       λ=0.2

Fine-tuning phase (20k steps): Production polish
  diffusion_loss:          λ=0.5
  reconstruction_loss:    λ=0.0
  adversarial_loss:       λ=0.1
  feature_matching_loss:  λ=0.1
  clap_similarity_loss:   λ=2.0  ← strong semantic alignment
  consistency_loss:       λ=0.5
```

---

## 6. Evaluation

### Objective Metrics

```python
class OneShotEvaluator:
    """Comprehensive evaluation suite for one-shot generation."""

    # Audio quality metrics
    @staticmethod
    def si_snr(clean, generated) -> float:
        """Scale-invariant signal-to-noise ratio. Higher = better."""
        ...

    @staticmethod
    def pesq(clean, generated, sr=16000) -> float:
        """Perceptual evaluation of speech quality. -0.5 to 4.5."""
        ...

    @staticmethod
    def stoi(clean, generated, sr=16000) -> float:
        """Short-term objective intelligibility. 0 to 1."""
        ...

    @staticmethod
    def frechet_audio_distance(feat_real, feat_fake) -> float:
        """FAD: distribution distance in embedding space. Lower = better."""
        ...

    @staticmethod
    def inception_score(audio, classifier) -> float:
        """IS: diversity × quality. Higher = better."""
        ...

    # One-shot specific metrics
    @staticmethod
    def transient_preservation(clean, gen) -> float:
        """Correlation of transient envelopes. 0-1. Higher = better."""
        ...

    @staticmethod
    def spectral_similarity(clean, gen) -> float:
        """Cosine similarity of averaged spectrograms. 0-1."""
        ...

    @staticmethod
    def loudness_match(clean, gen) -> float:
        """|LUFS_clean - LUFS_gen|. Lower = better."""
        ...

    @staticmethod
    def prompt_adherence(prompt, audio, clap_model) -> float:
        """CLAP similarity between prompt and audio."""
        text_emb = clap_model.encode_text(prompt)
        audio_emb = clap_model.encode_audio(audio)
        return F.cosine_similarity(text_emb, audio_emb).item()
```

### Subjective Evaluation (MUSHRA-Style)

```
Test design:
  - 30 participants (mix of producers, sound designers, casual listeners)
  - Each trial: reference + 5 test sounds (randomized, anonymous)
  - Rate each on 0-100 scale:
    a) Overall quality
    b) How well it matches prompt
    c) Production readiness (could use in a track today?)
    d) Uniqueness (doesn't sound like generic samples)

Analysis:
  - Mean opinion score (MOS) per model
  - 95% confidence intervals
  - Pairwise significance (Wilcoxon signed-rank)
  - Inter-rater agreement (Krippendorff's alpha)
```

### Evaluation Dataset

```
Held-out test set: 2,000 one-shots (never seen during training)
  - 200 per major category (kick, snare, hat, clap, perc, fx, synth, etc.)
  - 500 from each distribution analog genre
  - Diverse sources (not all from same pack)

Test prompts: 500 prompts written specifically for evaluation
  - 200 specific: "808 kick, 40Hz sub, papery attack, short decay"
  - 150 creative: "a kick that sounds like a thunderclap"
  - 100 comparative: "like a 909 but punchier"
  - 50 abstract: "make the percussion equivalent of neon light"
```

### Continuous Evaluation During Training

```python
# Every 5,000 training steps:
eval_metrics = {}

# Generate from 50 fixed test prompts (seeded for reproducibility)
for prompt in TEST_PROMPTS:
    audio = generate(prompt, seed=42)

# Compute metrics
fad = compute_fad(audio, REFERENCE_EMBEDDINGS)
eval_metrics["FAD"] = fad

clap_score = mean([prompt_adherence(p, a) for p, a in zip(TEST_PROMPTS, outputs)])
eval_metrics["CLAP_score"] = clap_score

# Early stopping criteria:
# Stop if FAD hasn't improved for 20,000 steps
# Stop if CLAP score > 0.35 (suggesting excellent prompt adherence)
```

---

## 7. Overfitting Risks

### Risk Assessment

```
Risk                          | Likelihood | Impact  | Mitigation
──────────────────────────────┼────────────┼─────────┼──────────────────────────
Memorization of training      │ High       │ Severe  | Dedup, augmentation
samples (copyright risk)      │            │         | fidelity check
                              │            │         |
Training distribution too     │ Medium     │ High    | Oversample rare types
narrow (only certain kicks)   │            │         | synthetic augmentation
                              │            │         |
Prompt overfitting (ignores   │ Medium     │ High    | Conditioning dropout
prompt, uses type only)       │            │         | CLAP similarity loss
                              │            │         |
Over-smoothing (all kicks     │ Medium     │ Medium  | Adversarial training
sound similar)                │            │         | diversity loss
                              │            │         |
Catastrophic forgetting       │ Low        │ High    | Replay buffer
of rare types during          │            │         | elastic weight
fine-tuning                   │            │         | consolidation
                              │            │         |
Evaluation overfitting        │ Medium     │ Medium  | Fresh test set each eval
(test set leaking)            │            │         | never tune on test set
```

### Detection Methods

```python
def check_memorization(generated_audio, training_set, threshold_si_snr=30):
    """
    Check if generated audio is too similar to any training sample.
    Si-SNR > 30dB → likely memorization.
    """
    for train_sample in training_set:
        snr = si_snr(generated_audio, train_sample)
        if snr > threshold_si_snr:
            return True, train_sample  # memorized!
    return False, None


def check_diversity(generated_batch, threshold_fad=0.5):
    """
    Check if a batch of generations are too similar.
    Low FAD within batch → mode collapse.
    """
    intra_fad = compute_intra_batch_fad(generated_batch)
    if intra_fad < threshold_fad:
        return True  # mode collapse detected
    return False


def check_prompt_ignoring(generated_audio, prompt, clap_model, threshold=0.15):
    """
    Check if model actually followed the prompt.
    Very low CLAP similarity → prompt ignored.
    """
    similarity = prompt_adherence(prompt, generated_audio, clap_model)
    if similarity < threshold:
        return True  # prompt not followed
    return False
```

### Regularization Strategy

```python
# 1. Strong augmentation (prevents memorization)
augmentation = OneShotAugmentation(
    time_shift_ms=50,
    random_eq_strength=0.3,
    noise_snr_range=(30, 50),
)

# 2. Conditioning dropout (prevents prompt ignoring)
conditioning_dropout = 0.15

# 3. Weight decay
optimizer = torch.optim.AdamW(model.parameters(), weight_decay=0.01)

# 4. Gradient clipping
torch.nn.utils.clip_grad_norm_(model.parameters(), max_norm=1.0)

# 5. Early stopping with patience
patience = 20000  # steps without FAD improvement

# 6. Label smoothing for classifier
label_smoothing = 0.1
```

---

## 8. Copyright Safety

### Dataset Auditing

```python
class CopyrightAuditor:
    """Ensures training data is safe."""

    def audit_dataset(self, dataset_path):
        # 1. Check all source licenses
        violations = []
        for file in dataset_path.iterdir():
            license = self.read_license(file)
            if license not in ALLOWED_LICENSES:
                violations.append((file, license))

        # 2. Check for watermarked content
        for audio_file in dataset_path.glob("*.wav"):
            if self.detect_watermark(audio_file):
                violations.append((audio_file, "watermark detected"))

        # 3. Check against opt-out list
        for audio_file in dataset_path.glob("*.wav"):
            hash = sha256(audio_file.read_bytes())
            if hash in OPTOUT_REGISTRY:
                violations.append((audio_file, "creator opted out"))

        return violations


# Allowed licenses for training:
ALLOWED_LICENSES = {
    "CC0 1.0",           # Public domain
    "CC BY 4.0",         # Attribution required
    "CC BY-SA 4.0",      # Share-alike
    "Royalty-Free",      # Verified commercial license
    "Custom License",    # Reviewed and approved
}

# Blocked licenses:
BLOCKED_LICENSES = {
    "All Rights Reserved",
    "CC BY-NC 4.0",      # Non-commercial
    "CC BY-ND 4.0",      # No derivatives
    "Unlicensed",
    "Copyrighted (unknown)",
}
```

### Generation Guardrails

```python
# At inference time:
def generate_safe(prompt: str, seed: int) -> GeneratedAudio:
    # 1. Check prompt against blocked content
    blocked_keywords = ["ripoff", "copy of", "exact same as", ...]
    if any(kw in prompt.lower() for kw in blocked_keywords):
        return SafetyError("Prompt references copyrighted content")

    # 2. Generate
    audio = model.generate(prompt, seed)

    # 3. Check output against known copyrighted samples
    for ref_hash in COPYRIGHTED_REGISTRY:
        if sha256(audio.wav) == ref_hash:
            return SafetyError("Output matches copyrighted sample")

    # 4. Check output similarity to training data
    memorized, match = check_memorization(audio, training_set)
    if memorized:
        # Regenerate with different seed
        return generate_safe(prompt, seed + 1)

    # 5. Add invisible watermark (for traceability)
    audio = add_watermark(audio)

    return audio
```

---

## 9. Genre Balancing

### Balanced Sampling Strategy

```python
class BalancedSampler:
    """
    Ensures all genres and types are well-represented during training.
    Prevents the model from specializing on over-represented types.
    """

    def __init__(self, dataset):
        # Count samples per type
        type_counts = defaultdict(int)
        for sample in dataset:
            type_counts[sample.label['type']].add(sample.id)
        self.type_counts = dict(type_counts)

        # Compute sampling weights (inverse frequency)
        max_count = max(self.type_counts.values())
        for type_name, count in self.type_counts.items():
            self.weights[type_name] = max_count / count  # rare types get higher weight

    def sample(self) -> DatasetItem:
        # 1. Sample type with temperature (prevent over/under sampling)
        type_weights = torch.tensor(list(self.weights.values()))
        type_weights = type_weights ** 0.7  # temperature: 1.0 = exact inverse, <1 = smoother
        chosen_type = random.choices(
            list(self.weights.keys()),
            weights=type_weights
        )[0]

        # 2. Sample within type
        candidates = self.type_to_ids[chosen_type]
        return random.choice(candidates)

    def get_batch(self, batch_size: int) -> List[DatasetItem]:
        return [self.sample() for _ in range(batch_size)]
```

### Target Distribution

```
Type distribution (target for training batches):
  kick:        15%  (oversampled if underrepresented)
  snare:       12%
  hihat:       10%
  percussion:  10%
  clap:         8%
  cymbal:       8%
  fx:          10%
  tom:          5%
  foley:        5%
  synth:       10%
  texture:      5%
  processed:    2%

Genre distribution (secondary sampling):
  Popular (hip-hop, trap, EDM): represent as 50% of batch
  Niche (experimental, world): represent as 10%
  Genre-agnostic: remainder balanced
```

---

## 10. Prompt Adherence

### Measuring Prompt Adherence

```python
def compute_prompt_adherence(generated: Audio, prompt: str, model: CLAPModel) -> float:
    """Primary metric: how well does the audio match the prompt?"""
    audio_emb = model.encode_audio(generated)
    text_emb = model.encode_text(prompt)
    return F.cosine_similarity(audio_emb, text_emb).item()  # [-1, 1]


def compute_prompt_bank_score(generated: Audio, bank: List[Tuple[str, float]]) -> float:
    """
    Prompt bank: list of (expected_characteristic, importance_weight).
    Score for multi-attribute prompts.
    """
    score = 0.0
    total_weight = 0.0
    for attr, weight in bank:
        detected = detect_attribute(generated, attr)  # e.g., "punchy" → punch detection
        score += weight * detected
        total_weight += weight
    return score / total_weight
```

### Improving Prompt Adherence

```python
# 1. During training: CLAP similarity loss
clap_loss = 1 - F.cosine_similarity(
    clap.encode_audio(generated),
    clap.encode_text(prompt)
)

# 2. During inference: classifier-free guidance scale
# Higher guidance = stronger prompt following (but less diversity)
guidance_scale = 3.0  # typical range: 1.5-7.5

# 3. During inference: prompt rewriting
# If prompt is vague, expand it with auto-completion
def expand_prompt(short_prompt: str) -> str:
    return PROMPT_TEMPLATES.get(short_prompt, short_prompt)
    # "kick" → "a punchy kick drum, 40Hz sub, fast attack, short decay"

# 4. Post-generation: prompt adherence filter
def filter_by_adherence(candidates: List[Audio], prompt: str, threshold: float) -> Audio:
    scored = [(compute_prompt_adherence(a, prompt), a) for a in candidates]
    best = max(scored, key=lambda x: x[0])
    if best[0] < threshold:
        return None  # no candidate met threshold, regenerate
    return best[1]
```

---

## 11. Comparison: Fine-Tuning Approaches

### Full Fine-Tuning

```
PROS:
  ✓ Maximum quality potential (all weights can adapt)
  ✓ Best prompt adherence (full model learns prompt structure)
  ✓ Can learn entirely new sound classes
  ✓ No architectural constraints

CONS:
  ✗ Very expensive (full model training, 8× A100s for days)
  ✗ One model per task (can't have many fine-tuned variants)
  ✗ Large model file to distribute (2-10 GB)
  ✗ Full re-training for any update
  ✗ Highest overfitting risk
  ✗ Copyright liability (model can memorize training data)

Best for: Final production model after architecture is stable.
Not suitable for: Iterative experimentation, per-user personalization.
```

### LoRA / Adapters

```
PROS:
  ✓ ~10,000× fewer parameters than full fine-tune
  ✓ Train in hours on single GPU (vs days for full)
  ✓ Tiny file size (2-50 MB per adapter)
  ✓ Can swap adapters at runtime (genre adapter, era adapter)
  ✓ Composable adapters (trap + 909 + punchy = combined)
  ✓ Less overfitting (low-rank constraint acts as regularizer)
  ✓ Easy to distribute (download genre pack as 5 MB file)
  ✓ Can train without full model (adapter only)

CONS:
  ✗ Slightly lower quality than full fine-tune (2-5% FAD)
  ✗ Limited capacity to learn entirely new patterns
  ✗ Adapter interference (combining too many degrades quality)
  ✗ Need base model to be good first

Best for: Genre-specific tuning, user personalization, iteration.
Recommended as DEFAULT approach for cShot.

Implementation:
  rank = 32        # LoRA rank (16-64 for audio)
  alpha = 64       # scaling factor
  target_modules = ["cross_attn.q", "cross_attn.v", "ffn.up", "ffn.down"]
```

### Retrieval Augmentation

```
PROS:
  ✓ Zero training required (works with existing generator)
  ✓ Can add new sounds instantly (just index them)
  ✓ Perfect for rare/explicit requests (show me X like Y)
  ✓ Improves consistency (retrieved reference anchors generation)
  ✓ No overfitting risk
  ✓ Copyright-safe (retrieved samples are reference, not memorized)
  ✓ Easy to remove specific references (delete from index)

CONS:
  ✗ Requires retrieval index at inference time
  ✗ Slower (retrieval + generation adds latency)
  ✗ Quality depends on retrieval quality
  ✗ Might overly constrain creativity (too similar to reference)
  ✗ Requires embedding model + similarity search

Best for: "Make more like this" feature, reference-based generation.
Used alongside LoRA, not instead of.

Architecture:
  prompt → embed → retrieve top-3 similar from library → condition generator
  Reference audio conditions the generation via cross-attention
```

### Preference Tuning (RLHF / DPO)

```
PROS:
  ✓ Aligns with human preferences (what sounds GOOD)
  ✓ Can optimize for subjective quality (not just reconstruction)
  ✓ Naturally learns taste (users save sounds they like)
  ✓ Can be personalized per user

CONS:
  ✗ Requires high-quality preference data (expensive to collect)
  ✗ Reward model can be gamed (reward hacking)
  ✗ May reduce diversity (converges to "safe" sounds)
  ✗ Complex training pipeline
  ✗ Per-user personalization requires per-user data

Best for: Polishing output quality, aligning with user taste.
Use after base model is trained.

Implementation:
  Step 1: Collect preference pairs (sound A > sound B)
  Step 2: Train reward model on preferences
  Step 3: Fine-tune generator with DPO (Direct Preference Optimization)
  No explicit RL needed — DPO works from preference pairs directly.
```

### Reward Models

```
PROS:
  ✓ Can optimize multiple objectives (quality + adherence + diversity)
  ✓ Continuous signal (not just binary better/worse)
  ✓ Can use automated metrics as rewards (FAD, CLAP)
  ✓ Reweighable at inference (tune quality vs creativity)

CONS:
  ✗ Reward model is another trained model (maintenance)
  ✗ Reward model inherits its own biases
  ✗ Need to avoid reward hacking
  ✗ Inference-time guidance adds complexity

Best for: Production deployment where multiple objectives matter.
```

### Decision Matrix

```
                 Full FT      LoRA        Retrieval    Preference   Reward
                 ───────      ────        ────────    ──────────    ─────
Quality          ★★★★★       ★★★★☆      ★★★☆☆       ★★★★★        ★★★★☆
Training cost    ✗✗✗✗✗       ★★★★☆       ★★★★★       ★★★☆☆        ★★★☆☆
Inference speed  ★★★★★       ★★★★★       ★★★☆☆       ★★★★★        ★★★★☆
Personalization  ✗✗✗✗✗       ★★★★★       ★★★★☆       ★★★★★        ★★★★☆
File size        ✗✗✗✗✗       ★★★★★       N/A          ★★★★☆        ★★★★☆
Distribution     ✗✗✗✗✗       ★★★★★       ★★★★★       ★★★★☆        ★★★★☆
Copyright safety ✗✗✗✗✗       ★★★☆☆       ★★★★★       ★★★☆☆        ★★★★☆
Overfitting risk ✗✗✗✗✗       ★★★☆☆       ★★★★★       ★★★☆☆        ★★★☆☆
Iteration speed  ✗✗✗✗✗       ★★★★★       ★★★★★       ★★★☆☆        ★★★★☆
Novel patterns   ★★★★★       ★★★☆☆       ★★★☆☆       ★★★★☆        ★★★☆☆

RECOMMENDATION:
  Primary:     LoRA (adapters for genres, styles, user preferences)
  Secondary:   Retrieval augmentation (reference-based generation)
  Tertiary:    Preference tuning (DPO for aligning with user taste)
  Reserve:     Full fine-tune (final model release, once)
```

---

## Summary

1. **Dataset**: 100k+ diverse one-shots across 12 categories, multi-label with technical + semantic + prompt annotations
2. **Labeling**: Auto-labeling pipeline + expert verification + prompt writing
3. **Conditioning**: Text (CLAP) + genre + parametric (duration, pitch, envelope) with conditioning dropout
4. **Augmentation**: On-the-fly waveform transforms + text-side prompt augmentation + latent interpolation
5. **Loss**: Diffusion loss + CLAP similarity + feature matching + consistency — schedule changes across training phases
6. **Evaluation**: FAD + CLAP adherence + MUSHRA subjective test + per-category metrics
7. **Overfitting**: Augmentation, dedup, conditioning dropout, memorization detection
8. **Copyright**: License auditing, similarity checking, invisible watermarking
9. **Genre balancing**: Inverse-frequency sampling with temperature smoothing
10. **Approach**: LoRA adapters as primary method, retrieval augmentation as secondary, preference tuning for polish
