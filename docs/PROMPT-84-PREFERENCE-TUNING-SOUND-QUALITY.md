# Prompt 84 — Preference Tuning for Sound Quality

A preference-tuning system for cShot that learns what sounds good from user behavior, without requiring explicit ratings.

---

## 1. Feedback Signals

### Signal Taxonomy

```
Every user interaction is a signal. The question is: what does it mean?

SIGNAL HIERARCHY:

Strong Positive (high confidence, low noise)
  ───────────────────────────────────────────
  SAVED     → "I want to keep this permanently"
  EXPORTED  → "I'm using this in a production"
  USED_IN_PACK → "This belongs in a curated collection"
  RATED_5   → "Explicit: I love this"

Weak Positive (some signal, noisy)
  ───────────────────────────────────────────
  PLAYED_COMPLETELY → "I listened to the whole thing"
  REPLAYED  → "I listened more than once"
  REGENERATED_FROM → "I wanted more like this"
  COPY_PARAMS → "I want to replicate these settings"

Weak Negative (some signal, noisy)
  ───────────────────────────────────────────
  PLAYED_PARTIALLY → "I stopped listening before end"
  SKIPPED   → "I moved on quickly"
  REGENERATED → "I asked for something else"

Strong Negative (high confidence)
  ───────────────────────────────────────────
  DELETED   → "I never want to hear this again"
  RATED_1   → "Explicit: I hate this"
  REPORTED  → "This is broken/wrong/offensive"

Context Signals (modifiers, not standalone)
  ───────────────────────────────────────────
  MANUALLY_EDITED → "The sound was close but needed changes"
  EXPORTED_AFTER_EDIT → "After fixing, it was usable"
  REGENERATION_PROMPT → "The specific change requested"
  PLAYBACK_CONTEXT → "What they were doing when they judged"
  COLLECTION_CONTEXT → "What else is in this pack?"
```

### Signal Weighting

```python
# Each signal has a confidence weight (0-1) and a polarity (+1/-1/0)
SIGNAL_WEIGHTS = {
    # Strong positive
    "saved":              SignalWeight(confidence=0.95, polarity=+1.0),
    "exported":           SignalWeight(confidence=0.90, polarity=+1.0),
    "used_in_pack":       SignalWeight(confidence=0.85, polarity=+1.0),
    "rated_5":            SignalWeight(confidence=1.00, polarity=+1.0),
    "rated_4":            SignalWeight(confidence=0.80, polarity=+0.6),

    # Weak positive
    "played_completely":  SignalWeight(confidence=0.40, polarity=+0.3),
    "replayed":           SignalWeight(confidence=0.55, polarity=+0.5),
    "regenerated_from":   SignalWeight(confidence=0.50, polarity=+0.4),
    "copy_params":        SignalWeight(confidence=0.30, polarity=+0.3),

    # Weak negative
    "played_partially":   SignalWeight(confidence=0.25, polarity=-0.2),
    "skipped":            SignalWeight(confidence=0.15, polarity=-0.2),
    "regenerated":        SignalWeight(confidence=0.35, polarity=-0.3),
    "rated_2":            SignalWeight(confidence=0.80, polarity=-0.6),

    # Strong negative
    "deleted":            SignalWeight(confidence=0.95, polarity=-1.0),
    "rated_1":            SignalWeight(confidence=1.00, polarity=-1.0),
    "reported":           SignalWeight(confidence=1.00, polarity=-1.0),

    # Context (modify other signals)
    "manually_edited":    SignalWeight(confidence=0.60, polarity= 0.0),  # modifier
    "exported_after_edit": SignalWeight(confidence=0.85, polarity=+0.5), # positive after fix
}
```

### Implicit Preference Extraction

```python
class PreferenceExtractor:
    """
    Extract pairwise preferences from implicit behavior.
    Turns user actions into (chosen, rejected) pairs.
    """

    def extract_pairs(self, session: UserSession) -> List[PreferencePair]:
        pairs = []

        # Pattern 1: Generation series
        # User generated 5 kicks, kept 1, deleted 3, ignored 1
        series = self.get_generation_series(session)
        for group in series:
            winners = [s for s in group if s.action in STRONG_POSITIVE]
            losers  = [s for s in group if s.action in STRONG_NEGATIVE]
            for w in winners:
                for l in losers:
                    pairs.append(PreferencePair(
                        chosen=w.sound_id,
                        rejected=l.sound_id,
                        confidence=w.confidence * l.confidence,
                    ))

        # Pattern 2: Regeneration parent
        # User said "generate more like this" → parent is good
        regenerations = self.get_regeneration_chains(session)
        for parent, children in regenerations:
            for child in children:
                if child.action in WEAK_POSITIVE:
                    pairs.append(PreferencePair(
                        chosen=parent.sound_id,
                        rejected=child.sound_id,
                        confidence=0.5 * child.confidence,
                    ))

        # Pattern 3: Export vs skip
        # In a session, exported sounds > not-exported sounds
        all_sounds = self.get_session_sounds(session)
        exported = [s for s in all_sounds if s.action == "exported"]
        skipped  = [s for s in all_sounds if s.action == "skipped"]
        for e in exported:
            for s in skipped:
                pairs.append(PreferencePair(
                    chosen=e.sound_id,
                    rejected=s.sound_id,
                    confidence=0.3,  # weaker signal, many confounds
                ))

        # Pattern 4: Edited and exported
        # User edited a sound then exported it = good after edit
        edits = self.get_edit_chains(session)
        for original, edited, action in edits:
            if action == "exported":
                pairs.append(PreferencePair(
                    chosen=edited.sound_id,
                    rejected=original.sound_id,
                    confidence=0.7,
                ))

        return pairs


class PreferencePair:
    sound_id_chosen: str
    sound_id_rejected: str
    confidence: float           # 0-1, how sure we are about this preference
    timestamp: datetime
    context: Dict               # What was user doing? Project? Genre?
    prompt: str                 # Original generation prompt
```

### Data Aggregation

```python
# User-level preference database
class TasteDatabase:
    """Stores and aggregates preference signals per user."""

    def __init__(self, db_path: str):
        self.db = sqlite3.connect(db_path)

    def record_event(self, user_id: str, sound_id: str, event: UserEvent):
        """Record a single user interaction event."""
        self.db.execute("""
            INSERT INTO taste_events
                (user_id, sound_id, event_type, context, confidence, polarity, created_at)
            VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
        """, (user_id, sound_id, event.event_type,
              json.dumps(event.context),
              SIGNAL_WEIGHTS[event.event_type].confidence,
              SIGNAL_WEIGHTS[event.event_type].polarity))

    def get_preference_score(self, user_id: str, sound_id: str) -> float:
        """Aggregate all signals for a user-sound pair into a single score."""
        rows = self.db.execute("""
            SELECT event_type, confidence, polarity
            FROM taste_events
            WHERE user_id = ? AND sound_id = ?
        """, (user_id, sound_id)).fetchall()

        if not rows:
            return 0.0

        # Weighted sum with decay (recent events matter more)
        score = 0.0
        total_weight = 0.0
        for event_type, confidence, polarity in rows:
            age_days = self.get_event_age(user_id, sound_id, event_type)
            time_decay = exp(-age_days / 30)  # 30-day half-life
            weight = confidence * time_decay
            score += weight * polarity
            total_weight += weight

        return score / total_weight if total_weight > 0 else 0.0

    def get_preference_pairs(self, user_id: str,
                             min_confidence: float = 0.5,
                             limit: int = 10000) -> List[PreferencePair]:
        """Extract high-confidence preference pairs for training."""
        extractor = PreferenceExtractor()
        session = self.get_user_session(user_id)
        pairs = extractor.extract_pairs(session)
        return [p for p in pairs if p.confidence >= min_confidence][:limit]
```

---

## 2. Reward Model

### Architecture

```
                    REWARD MODEL ARCHITECTURE

  Audio ──► Audio Encoder ──► Embedding ──┐
                                          │
  Prompt ──► Text Encoder ──► Embedding ──┼──► Fusion ──► MLP ──► Score (0-1)
                                          │
  Metadata ──► Feature Encoder ───────────┘
    (duration, genre, params, etc.)
```

```python
class AudioRewardModel(nn.Module):
    """
    Predicts how much a user will like a generated sound.
    Trained on implicit preference pairs.
    """

    def __init__(self, config: RewardModelConfig):
        super().__init__()
        # Audio encoder (pretrained, fine-tuned)
        self.audio_encoder = CLAPAudioEncoder(config.audio_encoder)
        self.audio_proj = nn.Linear(config.audio_dim, config.hidden_dim)

        # Text encoder (pretrained, frozen)
        self.text_encoder = CLAPTextEncoder(config.text_encoder)
        self.text_proj = nn.Linear(config.text_dim, config.hidden_dim)

        # Metadata encoder
        self.meta_encoder = nn.Sequential(
            nn.Linear(config.meta_dim, config.hidden_dim // 2),
            nn.ReLU(),
            nn.Linear(config.hidden_dim // 2, config.hidden_dim),
        )

        # Fusion
        self.fusion = nn.TransformerEncoderLayer(
            d_model=config.hidden_dim,
            nhead=8,
            dim_feedforward=config.hidden_dim * 4,
        )

        # Score head (0 = bad, 1 = perfect)
        self.score_head = nn.Sequential(
            nn.Linear(config.hidden_dim, config.hidden_dim // 2),
            nn.ReLU(),
            nn.Dropout(0.1),
            nn.Linear(config.hidden_dim // 2, config.hidden_dim // 4),
            nn.ReLU(),
            nn.Linear(config.hidden_dim // 4, 1),
            nn.Sigmoid(),  # output in [0, 1]
        )

    def forward(self, audio: Tensor, prompt: str, metadata: Dict) -> Tensor:
        # Encode modalities
        a_emb = self.audio_proj(self.audio_encoder(audio))
        t_emb = self.text_proj(self.text_encoder(prompt))
        m_emb = self.meta_encoder(encode_metadata(metadata))

        # Fusion
        combined = torch.stack([a_emb, t_emb, m_emb], dim=1)
        fused = self.fusion(combined).mean(dim=1)

        # Score
        return self.score_head(fused).squeeze(-1)
```

### Training on Pairwise Preferences

```python
class RewardTrainer:
    """
    Train reward model using Bradley-Terry preference model.
    P(chosen > rejected) = σ(score_chosen - score_rejected)
    """

    def train_step(self, batch: List[PreferencePair]) -> float:
        # Encode chosen and rejected
        chosen_scores = self.reward_model(
            batch.chosen_audio,
            batch.chosen_prompt,
            batch.chosen_metadata,
        )
        rejected_scores = self.reward_model(
            batch.rejected_audio,
            batch.rejected_prompt,
            batch.rejected_metadata,
        )

        # Bradley-Terry loss
        logits = chosen_scores - rejected_scores  # higher = chosen > rejected
        loss = -F.logsigmoid(logits).mean()

        # Confidence-weighted variant
        if use_confidence:
            weights = torch.tensor(batch.confidence)
            loss = -(weights * F.logsigmoid(logits)).mean()

        # Regularization (prevent score explosion)
        loss += 0.01 * (chosen_scores.pow(2).mean() + rejected_scores.pow(2).mean())

        loss.backward()
        return loss.item()


# Inference
def score_sound(reward_model, sound: Sound, prompt: str, user_id: str) -> float:
    """Predict how much this user will like this sound."""
    metadata = {
        "duration": sound.duration_ms,
        "genre": sound.genre,
        "type": sound.type,
        "loudness": sound.loudness_lufs,
        "user_history": get_user_similar_sounds(user_id, sound),
    }
    return reward_model(sound.audio_tensor, prompt, metadata).item()
```

### Reward Model Evaluation

```python
def evaluate_reward_model(reward_model, test_pairs: List[PreferencePair]) -> Dict:
    """
    Evaluate reward model on held-out preference pairs.
    Metrics:
      - Accuracy: % of pairs where chosen scored > rejected
      - AUC: area under ROC curve
      - Spearman rank correlation: ranking consistency
    """
    accuracies = []
    diffs = []
    for pair in test_pairs:
        score_chosen = reward_model(pair.chosen_audio, pair.chosen_prompt, pair.chosen_metadata)
        score_rejected = reward_model(pair.rejected_audio, pair.rejected_prompt, pair.rejected_metadata)
        diffs.append(score_chosen.item() - score_rejected.item())
        accuracies.append(1.0 if score_chosen > score_rejected else 0.0)

    return {
        "accuracy": mean(accuracies),
        "auc": compute_auc(diffs, [1]*len(diffs)),
        "mean_diff": mean(diffs),
        "calibration_error": compute_calibration_error(reward_model, test_pairs),
    }
```

---

## 3. Pairwise Ranking Data

### Data Collection Pipeline

```
                    USER BEHAVIOR
                         │
                         ▼
              ┌─────────────────────┐
              │  Event Logger        │
              │  (Tauri hook)        │
              │  Every interaction   │
              └──────────┬──────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  Preference Parser   │
              │  Extract pairs from  │
              │  sessions            │
              └──────────┬──────────┘
                         │
              ┌──────────┴──────────┐
              │                     │
              ▼                     ▼
    ┌──────────────────┐  ┌──────────────────┐
    │  Local DB         │  │  Optional Cloud  │
    │  (user-specific)  │  │  (aggregated)    │
    │  ~10k pairs/user  │  │  ~1M pairs total │
    └──────────────────┘  └──────────────────┘
```

### Pair Quality Filtering

```python
class PairQualityFilter:
    """Filter out low-quality or misleading preference pairs."""

    def filter(self, pairs: List[PreferencePair]) -> List[PreferencePair]:
        filtered = []

        for pair in pairs:
            # Remove if same sound (dedup)
            if pair.chosen == pair.rejected:
                continue

            # Remove if confidence too low
            if pair.confidence < 0.3:
                continue

            # Remove if too old (preferences drift)
            if pair.age_days > 90:
                continue

            # Remove if sounds are too similar (no information)
            similarity = audio_similarity(pair.chosen_audio, pair.rejected_audio)
            if similarity > 0.95:
                continue

            # Remove if context is too noisy
            if pair.context.get("distracted", False):
                continue

            filtered.append(pair)

        return filtered
```

### Synthetic Pair Generation

```python
class SyntheticPairGenerator:
    """
    Generate synthetic preference pairs for cold-start users.
    Uses known objective quality criteria:
      - No clipping > no clipping → prefer no clipping
      - Good transient > muddy → prefer clear transient
      - Consistent loudness > inconsistent → prefer consistent
    """

    def generate_synthetic_pairs(self, sounds: List[Sound]) -> List[PreferencePair]:
        pairs = []
        for a, b in itertools.combinations(sounds, 2):
            reasons = []

            # Quality heuristic 1: clipping
            if a.clips and not b.clips:
                pairs.append(PreferencePair(chosen=b, rejected=a,
                              confidence=0.3, source="synthetic_clipping"))
            elif b.clips and not a.clips:
                pairs.append(PreferencePair(chosen=a, rejected=b,
                              confidence=0.3, source="synthetic_clipping"))

            # Quality heuristic 2: DC offset
            if abs(a.dc_offset) < abs(b.dc_offset):
                pairs.append(PreferencePair(chosen=a, rejected=b,
                              confidence=0.2, source="synthetic_dc_offset"))

            # Quality heuristic 3: dynamic range
            if a.dynamic_range > b.dynamic_range:
                pairs.append(PreferencePair(chosen=a, rejected=b,
                              confidence=0.15, source="synthetic_dynamic_range"))

        return pairs
```

---

## 4. RLHF / RLAIF Equivalent for Audio

### Why Not Standard RLHF?

```
Standard RLHF (RL from Human Feedback):
  1. SFT (supervised fine-tuning) on demonstrations
  2. Train reward model on human preferences
  3. RL fine-tune (PPO) using reward model

Problem for audio generation:
  ✗ PPO is unstable and complex for diffusion models
  ✗ Audio is continuous (not discrete tokens) — harder to optimize
  ✗ Reward model needs to evaluate full audio, not just a token
  ✗ PPO requires on-policy generation (very expensive for diffusion)

Audio-adapted approach: DPO (Direct Preference Optimization)
  ✓ No RL needed — directly optimizes from preference pairs
  ✓ Works with any differentiable generator
  ✓ Stable training (no PPO instability)
  ✓ Uses the same preference data
  ✓ Actually simpler than PPO
```

### DPO for One-Shot Generation

```python
class DPOTrainer:
    """
    Direct Preference Optimization for audio diffusion models.
    From: "Direct Preference Optimization" (Rafailov et al., 2023)
    Adapted for continuous audio output.
    """

    def __init__(self, generator: DiffusionModel, reward_model: RewardModel):
        self.generator = generator
        self.reward_model = reward_model

    def train_step(self, batch: List[PreferencePair]) -> float:
        """
        DPO loss:
          L = -E[ log σ(β * (r(x_chosen) - r(x_rejected))) ]

        where r(x) = generation probability under policy
              β = temperature parameter
        """
        # Generate or use stored generations
        chosen_audio = batch.chosen_audio
        rejected_audio = batch.rejected_audio

        # Get log-probabilities from generator
        with torch.no_grad():
            logprob_chosen = self.generator.log_prob(chosen_audio, batch.chosen_prompt)
            logprob_rejected = self.generator.log_prob(rejected_audio, batch.rejected_prompt)

        # DPO loss
        beta = 0.1  # temperature, lower = more preference-focused
        logits = beta * (logprob_chosen - logprob_rejected)
        loss = -F.logsigmoid(logits).mean()

        loss.backward()
        return loss.item()


class RLAIFTrainer:
    """
    RL from AI Feedback.
    Uses the reward model (trained on human preferences) to
    generate synthetic preference pairs for further training.
    """

    def generate_ai_pairs(self, prompts: List[str]) -> List[PreferencePair]:
        pairs = []
        for prompt in prompts:
            # Generate multiple candidates
            candidates = [self.generator.generate(prompt, seed=s)
                         for s in range(10)]

            # Score with reward model
            scores = [self.reward_model(c, prompt, {}) for c in candidates]

            # Create pairs: best > worst
            sorted_idx = sorted(range(len(scores)), key=lambda i: scores[i])
            best = candidates[sorted_idx[-1]]
            worst = candidates[sorted_idx[0]]

            # Label as AI-generated preference
            pairs.append(PreferencePair(
                chosen=best.id,
                rejected=worst.id,
                confidence=0.6,  # lower than human feedback
                source="ai_feedback",
            ))

        return pairs
```

### Preference Learning Loop

```
                      ┌───────────────────────┐
                      │  User Actions          │
                      │  (saves, exports,      │
                      │   deletes, replays)    │
                      └───────────┬───────────┘
                                  │
                                  ▼
                      ┌───────────────────────┐
                      │  Preference Extractor  │
                      │  (pairs from behavior) │
                      └───────────┬───────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    │                           │
                    ▼                           ▼
          ┌──────────────────┐      ┌──────────────────┐
          │  Reward Model     │      │  RLAIF: Generate │
          │  Training         │      │  synthetic pairs │
          │  (human pairs)    │      │  (model pairs)   │
          └────────┬─────────┘      └────────┬─────────┘
                   │                         │
                   └──────────┬──────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  DPO Generator    │
                    │  Fine-tuning      │
                    │  (all pairs)      │
                    └────────┬─────────┘
                             │
                             ▼
                    ┌──────────────────┐
                    │  Better Sounds   │
                    │  → More Positive │
                    │     Feedback     │
                    └──────────────────┘
```

### RLHF Quality Impact

```
Expected improvement from DPO fine-tuning:

                Before DPO    After DPO     After 3 rounds
                ──────────    ─────────     ──────────────
FAD (lower=better)   2.1        1.4            1.1
CLAP adherence       0.28       0.35           0.38
User save rate       12%        22%            28%
User export rate     5%         11%            15%
Avg rating (1-5)     3.2        3.8            4.1
Re-generate rate     45%        32%            25%

Key insight: Most improvement comes from eliminating
"obviously bad" generations (clipping, muddy, wrong type).
Preference tuning can't create new capabilities — it polishes.
```

---

## 5. User-Personalized Reward Models

### Architecture: Global + Local

```
┌─────────────────────────────────────────────────────────┐
│                  GLOBAL REWARD MODEL                      │
│  Trained on ALL users' preferences                       │
│  Learns: what sounds "good" in general                   │
│  Size: ~50M parameters                                   │
│  Update frequency: weekly (cloud)                        │
│  Downloaded to local once per week                       │
│                                                          │
│  Predicts: "How good is this sound, objectively?"        │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                  LOCAL ADAPTER                            │
│  Lightweight LoRA adapts global model to user's taste    │
│  Learns: what THIS user specifically likes               │
│  Size: ~2M parameters (4 MB)                             │
│  Update: after every session (local training)            │
│  Stored: in user's local ~/.cshot/                       │
│                                                          │
│  Predicts: "How much would THIS user like this sound?"   │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                  FINAL SCORE                             │
│  Personal score = 0.7 * global + 0.3 * local_adapter    │
│  When user has < 100 events: 1.0 * global               │
│  When user has > 1000 events: 0.5 * global + 0.5 * local│
│  When user has > 10000 events: 0.3 * global + 0.7 * local│
└─────────────────────────────────────────────────────────┘
```

### Local Training

```python
class LocalRewardAdapter:
    """
    Lightweight personalization adapter that trains on-device.
    Uses LoRA-style adaptation of the global reward model.
    """

    def __init__(self, global_model: AudioRewardModel, user_id: str):
        self.global_model = global_model
        self.user_id = user_id

        # Freeze global model
        for param in self.global_model.parameters():
            param.requires_grad = False

        # Add LoRA adapters to all linear layers
        self.adapters = []
        for name, module in self.global_model.named_modules():
            if isinstance(module, nn.Linear):
                lora = LoRALayer(
                    in_dim=module.in_features,
                    out_dim=module.out_features,
                    rank=8,  # very small — 2M params total
                )
                self.adapters.append(lora)
                # Register hook to apply LoRA after original linear
                module.register_forward_hook(
                    lambda mod, inp, out: out + lora(inp[0])
                )

    def train_on_session(self, session: UserSession, epochs: int = 5):
        pairs = PreferenceExtractor().extract_pairs(session)
        pairs = PairQualityFilter().filter(pairs)

        if len(pairs) < 10:
            return  # Not enough data

        optimizer = torch.optim.AdamW(
            [p for adapter in self.adapters for p in adapter.parameters()],
            lr=1e-4,
        )

        for epoch in range(epochs):
            for batch in batch_pairs(pairs, batch_size=32):
                chosen_scores = self.forward(batch.chosen_audio, batch.chosen_prompt, {})
                rejected_scores = self.forward(batch.rejected_audio, batch.rejected_prompt, {})
                loss = -F.logsigmoid(chosen_scores - rejected_scores).mean()
                loss.backward()
                optimizer.step()
                optimizer.zero_grad()

    def forward(self, audio, prompt, metadata) -> Tensor:
        # Global score + local adjustment
        global_score = self.global_model(audio, prompt, metadata)
        local_adjustment = sum(a(audio) for a in self.adapters)  # simplified
        return global_score + 0.3 * local_adjustment  # dampened
```

### Personalization Features

```python
# Features that differentiate user taste
USER_TASTE_FEATURES = {
    "preferred_genres": ["trap", "drill", "hyperpop"],
    "preferred_types": {
        "kick": "sub_kick, 808, boomy",
        "snare": "trap_snare, rimshot, tight",
        "perc": "shaker, tambourine",
    },
    "avoided_characteristics": ["too_metallic", "too_long_decay"],
    "loudness_preference": "loud",          # quiet / moderate / loud
    "dry_wet_preference": 0.7,              # 0 = completely dry, 1 = very processed
    "vintage_modern": 0.3,                  # 0 = vintage, 1 = modern
    "analog_digital": 0.6,                  # 0 = analog, 1 = digital
    "preferred_producers": ["mike_dean", "finneas"],
    "typical_projects": ["album", "beat_tape", "soundtrack"],
    "favorite_artists": ["travis_scott", "ye", "100_gecs"],
}

# These are extracted from behavior, not explicitly asked
# Derived from: genres of exported sounds, saved packs,
# similar sounds in library, listened-to reference tracks.
```

---

## 6. Global Quality Model

### Purpose

```
The Global Quality Model answers:
  "Is this a well-produced one-shot, regardless of taste?"

It scores:
  - Technical quality (no clipping, clean transients, etc.)
  - Production readiness (would a pro engineer accept this?)
  - Genre appropriateness (does this kick fit trap?)
  - Diversity (is this just like every other kick?)

It does NOT score:
  - Whether the user will like it (that's the personal model)
  - Whether it matches the prompt (that's prompt adherence)
  - Whether it's creative (that's the creativity model)
```

### Architecture

```python
class GlobalQualityModel(nn.Module):
    """
    Predicts objective production quality of a one-shot.
    Think: "Would a Grammy-winning engineer approve this?"
    """

    def __init__(self):
        super().__init__()
        # Audio encoder
        self.encoder = AudioEncoder(pretrained=True)

        # Technical quality heads
        self.transient_quality = nn.Sequential(
            nn.Linear(512, 128), nn.ReLU(),
            nn.Linear(128, 1), nn.Sigmoid(),
        )
        self.spectral_balance = nn.Sequential(
            nn.Linear(512, 128), nn.ReLU(),
            nn.Linear(128, 1), nn.Sigmoid(),
        )
        self.noise_floor = nn.Sequential(
            nn.Linear(512, 128), nn.ReLU(),
            nn.Linear(128, 1), nn.Sigmoid(),
        )
        self.dynamic_range = nn.Sequential(
            nn.Linear(512, 128), nn.ReLU(),
            nn.Linear(128, 1), nn.Sigmoid(),
        )
        self.mix_readiness = nn.Sequential(
            nn.Linear(512, 128), nn.ReLU(),
            nn.Linear(128, 1), nn.Sigmoid(),
        )

        # Overall quality (learned from expert ratings)
        self.overall = nn.Sequential(
            nn.Linear(512, 256), nn.ReLU(),
            nn.Dropout(0.2),
            nn.Linear(256, 128), nn.ReLU(),
            nn.Linear(128, 1), nn.Sigmoid(),
        )

    def forward(self, audio: Tensor) -> QualityScores:
        features = self.encoder(audio)
        return QualityScores(
            transient_quality=self.transient_quality(features),
            spectral_balance=self.spectral_balance(features),
            noise_floor=self.noise_floor(features),
            dynamic_range=self.dynamic_range(features),
            mix_readiness=self.mix_readiness(features),
            overall=self.overall(features),
        )


@dataclass
class QualityScores:
    transient_quality: float    # 0-1: clean attack, no pre-ringing
    spectral_balance: float     # 0-1: good frequency distribution
    noise_floor: float          # 0-1: low noise, clean signal
    dynamic_range: float        # 0-1: appropriate range for type
    mix_readiness: float        # 0-1: would pro use as-is
    overall: float              # 0-1: global quality score
```

### Training Data

```python
# Global quality model is trained on expert-annotated data
TRAINING_SOURCES = {
    "expert_ratings": {
        "source": "Professional sound engineers rate 10,000 sounds",
        "rating_scale": "1-5 on quality, mix-readiness, technical flaws",
        "raters_per_sound": 3,
    },
    "production_data": {
        "source": "Sounds used in released tracks vs. unused",
        "label": "Was this sound in a commercial release?",
        "size": "50,000+ sounds from Splice/Loopmasters",
    },
    "synthetic_data": {
        "source": "Deliberately degraded sounds with known issues",
        "label": "Type of degradation (clipping, noise, etc.)",
        "size": "10,000 degraded variants of 1,000 clean sounds",
    },
}

# Known quality defects the model must detect:
QUALITY_DEFECTS = {
    "clipping": "Peak > -0.1 dBFS, visible hard limiting",
    "dc_offset": "Non-zero DC component",
    "pre_ringing": "Audible artifact before transient (from LPF/codec)",
    "noise_floor_audible": "Background noise > -60 dBFS",
    "muddy_transient": "Attack is smeared, not sharp",
    "spectral_hole": "Missing frequency content in important range",
    "excessive_sibilance": "Too much energy 4-8 kHz",
    "phase_issues": "Mono-incompatible, phase cancellation",
    "too_quiet": "LUFS < -24 (too quiet for production)",
    "too_loud": "LUFS > -8 (too compressed, no headroom)",
}
```

---

## 7. Safety Filters

### What to Filter

```python
SAFETY_CATEGORIES = {
    "copyright_infringement": {
        "description": "Output too similar to copyrighted work",
        "detection": "Audio fingerprint match + embedding similarity > threshold",
        "action": "Block generation, inform user",
        "severity": "critical",
    },
    "explicit_harmful": {
        "description": "Hate speech, violence, explicit content",
        "detection": "Prompt classifier + output classifier",
        "action": "Block generation, log for review",
        "severity": "critical",
    },
    "deceptive_content": {
        "description": "Sounds that mimic real-world dangerous sounds",
        "detection": "Onomatopoeia/word matching in prompt",
        "action": "Block plausible dangerous sounds (gunshots, etc.)",
        "severity": "high",
    },
    "low_quality": {
        "description": "Objectively broken or unusable output",
        "detection": "Global quality model score < 0.3",
        "action": "Auto-regenerate with different seed",
        "severity": "normal",
    },
    "promet_mismatch": {
        "description": "Generated wrong type entirely",
        "detection": "Type classifier vs. expected type from prompt",
        "action": "Auto-regenerate with stronger conditioning",
        "severity": "normal",
    },
    "not_a_one_shot": {
        "description": "Generated long-form audio instead of one-shot",
        "detection": "Duration > 15s or multiple onsets",
        "action": "Crop/trim or regenerate with shorter duration",
        "severity": "normal",
    },
}
```

### Filter Pipeline

```
Prompt ──► Safety Prompt Filter ──► Generation ──► Output Safety Filter ──► User
              │                                                    │
              ▼                                                    ▼
         Blocked?                                            Problems?
              │                                                    │
         Yes ─┴─► "Can't generate that"                    Yes ─┴─► Auto-fix or
              │                                                    │  regenerate
         No ───► Continue                                    No ───► Deliver to user
```

```python
class SafetyFilter:
    """Multi-stage safety pipeline for cShot generation."""

    def __init__(self, global_quality_model: GlobalQualityModel):
        self.quality_model = global_quality_model

    def check_prompt(self, prompt: str) -> SafetyResult:
        # Check against blocked patterns
        for category, patterns in BLOCKED_PROMPT_PATTERNS.items():
            for pattern in patterns:
                if re.search(pattern, prompt, re.IGNORECASE):
                    return SafetyResult(
                        safe=False,
                        category=category,
                        reason=f"Prompt matches blocked pattern: {pattern}",
                    )

        # Check prompt embedding against banned embedding clusters
        prompt_emb = self.prompt_encoder.encode(prompt)
        for banned_cluster in BANNED_PROMPT_CLUSTERS:
            similarity = cosine_similarity(prompt_emb, banned_cluster.centroid)
            if similarity > banned_cluster.threshold:
                return SafetyResult(
                    safe=False,
                    category="prompt_similar_to_known_harmful",
                    reason="Prompt too similar to known harmful patterns",
                )

        return SafetyResult(safe=True)

    def check_output(self, audio: Tensor, prompt: str) -> SafetyResult:
        # Check generation quality
        quality = self.quality_model(audio)

        if quality.overall < 0.2:
            return SafetyResult(
                safe=False,
                category="low_quality",
                reason=f"Low quality score: {quality.overall:.2f}",
                auto_fix="regenerate",
            )

        # Check audio fingerprint against copyrighted registry
        fp = compute_audio_fingerprint(audio)
        for ref_fp, ref_info in COPYRIGHTED_FINGERPRINTS.items():
            if fingerprint_distance(fp, ref_fp) < MATCH_THRESHOLD:
                return SafetyResult(
                    safe=False,
                    category="copyright_infringement",
                    reason=f"Matches copyrighted sample: {ref_info['title']}",
                    auto_fix="regenerate_with_variation",
                )

        # Check duration (must be one-shot)
        duration = compute_duration(audio, sample_rate=44100)
        if duration > 15.0:
            return SafetyResult(
                safe=False,
                category="not_a_one_shot",
                reason=f"Duration {duration:.1f}s exceeds 15s limit",
                auto_fix="trim_to_10s",
            )

        return SafetyResult(safe=True)

    def filter_and_fix(self, audio: Tensor, prompt: str,
                       max_retries: int = 3) -> Tuple[Tensor, SafetyReport]:
        report = SafetyReport()

        # Check prompt
        prompt_result = self.check_prompt(prompt)
        report.prompt_check = prompt_result
        if not prompt_result.safe:
            return None, report

        # Check and fix output
        for attempt in range(max_retries):
            output_result = self.check_output(audio, prompt)
            report.add_check(output_result, attempt)

            if output_result.safe:
                return audio, report

            if output_result.auto_fix == "regenerate":
                audio = regenerate_with_different_seed(prompt)
            elif output_result.auto_fix == "trim_to_10s":
                audio = trim_audio(audio, max_duration=10.0)
            elif output_result.auto_fix == "regenerate_with_variation":
                audio = regenerate_with_variation(prompt, variation_strength=0.3)
            else:
                # Can't auto-fix
                break

        return None, report  # Failed after max retries
```

---

## 8. Complete Preference System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       PREFERENCE SYSTEM ARCHITECTURE                     │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                         EVENT PIPELINE                           │   │
│  │                                                                  │   │
│  │  User Action → Event Logger → Signal Extractor → Pair Generator │   │
│  │       ↑                             │                │           │   │
│  │       │                             ▼                ▼           │   │
│  │       │                    ┌────────────────┐  ┌──────────┐     │   │
│  │       │                    │ Quality Filter  │  │ Local DB │     │   │
│  │       │                    └────────┬───────┘  └──────────┘     │   │
│  │       │                             │                           │   │
│  │       └─────────────────────────────┘                           │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                    │
│                                    ▼                                    │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                       TRAINING LOOP                              │   │
│  │                                                                  │   │
│  │  ┌──────────┐    ┌──────────────┐    ┌────────────────────────┐ │   │
│  │  │ Global   │◄───│ Preference   │◄───│ All Users' Pairs       │ │   │
│  │  │ Reward   │    │ Trainer (DPO)│    │ (cloud aggregated)     │ │   │
│  │  │ Model    │───►│              │    └────────────────────────┘ │   │
│  │  └──────────┘    └──────────────┘                               │   │
│  │       │                                                         │   │
│  │       ▼                                                         │   │
│  │  ┌──────────┐    ┌──────────────┐    ┌────────────────────────┐ │   │
│  │  │ Local    │◄───│ Local        │◄───│ This User's Pairs      │ │   │
│  │  │ Adapter  │    │ Fine-Tune    │    │ (local, private)       │ │   │
│  │  │ (LoRA)   │───►│              │    └────────────────────────┘ │   │
│  │  └──────────┘    └──────────────┘                               │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                    │
│                                    ▼                                    │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                      INFERENCE PIPELINE                         │   │
│  │                                                                  │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │   │
│  │  │ Generator│─►│ Quality  │─►│ Reward   │─►│ Safety Filter    │─►│   │
│  │  │ (LoRA FT)│  │ Check    │  │ Score    │  │ (final check)    │  │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Summary

1. **Feedback signals**: 15+ signals from implicit user behavior, weighted by confidence and polarity
2. **Reward model**: Audio + text + metadata fusion → score. Trained with Bradley-Terry pairwise loss
3. **Pairwise ranking data**: Extracted from generation series, regeneration chains, export/skip patterns
4. **RLAIF for audio**: DPO (Direct Preference Optimization) replaces PPO — simpler, stable, effective
5. **User-personalized reward**: Global model + local LoRA adapter trains on-device from user's behavior
6. **Global quality model**: Scores production readiness independent of taste — trained on expert ratings
7. **Safety filters**: Multi-stage (prompt + output + quality) with auto-fix and configurable thresholds
8. **System loop**: User actions → preference pairs → DPO training → better generations → more actions
