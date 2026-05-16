# Prompt 64 — Personal Sonic Identity

Design the concept of a user's personal sonic identity — a rich preference model that captures not just what sounds they like, but their unique sonic fingerprint across multiple dimensions.

---

## 1. What Is a Personal Sonic Identity?

### Definition

A personal sonic identity is a multi-dimensional model of a user's sonic preferences, creative tendencies, and aesthetic instincts. It's more than "what sounds they like" — it captures *how they think about sound*.

```
A producer's sonic identity includes:
  - What sounds they reach for (favorite textures, types)
  - How they shape those sounds (brightness, punch, dynamics)
  - What emotional palette they work in (dark/bright, aggressive/gentle)
  - What genres they gravitate toward
  - How experimental vs. commercial their taste is
  - Their signature moves as a sound designer
  
Analogous to:
  - Spotify's taste profile but for sound creation, not listening
  - A chef's palate profile (spicy tolerance, umami preference, etc.)
  - A photographer's visual style (contrast preference, color grading)
```

### Why Identity vs. Memory

| Concept | Taste Memory | Sonic Identity |
|---------|-------------|----------------|
| Scope | What sounds you like | How you think about sound |
| Update frequency | Per action | Per session |
| Dimensionality | 768-d embedding | Multi-dimensional profile |
| Use | Adapt generation parameters | Guide all creative decisions |
| Output | "This user likes punchy kicks" | "This user is a trap producer who likes dark, punchy kicks with sub-bass, tends toward simple arrangements, and prefers clean over lo-fi" |

---

## 2. Sonic Identity Dimensions

### Learned Dimensions

The identity model learns 10 distinct dimensions:

```
Dimension 1: Favorite Textures
  Scale: [Warm] ◄───────────────────────────► [Bright]
         Dark, round, smooth           Harsh, edgy, glassy
  How learned: Average spectral centroid of exported sounds
  Signal: 200-4000Hz centroid position

Dimension 2: Rhythmic Tendencies  
  Scale: [Groove] ◄─────────────────────────► [Grid]
         Loose, swingy, behind beat    Tight, quantized, ahead
  How learned: BPM consistency, swing preference in prompts
  Signal: "tight" vs "loose" keywords, BPM variance across sessions

Dimension 3: Emotional Palette
  Scale: [Dark] ◄────────────────────────────► [Bright]
         Minor, moody, atmospheric     Major, energetic, uplifting
  How learned: Spectral content + prompt mood keywords
  Signal: "dark", "moody", "atmospheric" vs "bright", "happy", "uplifting"

Dimension 4: Genre Taste
  Scale: Genre distribution across exports
  How learned: Genre classifier on exported sounds
  Signal: Genre matches from generation metadata

Dimension 5: Brightness/Darkness
  Scale: [Dark] ◄───────────────────────────► [Bright]
         Sub-heavy, rolled-off highs   Presence-heavy, airy
  How learned: High-frequency energy ratio in exports
  Signal: Spectral centroid + high-band energy (>8kHz)

Dimension 6: Punch/Softness
  Scale: [Soft] ◄───────────────────────────► [Punchy]
         Gentle attacks, round         Sharp transients, aggressive
  How learned: Crest factor + onset strength of favorite sounds
  Signal: Average crest factor of exported sounds

Dimension 7: Clean/Dirty
  Scale: [Clean] ◄──────────────────────────► [Dirty]
         Pristine, polished, sterile   Saturated, gritty, noisy
  How learned: Noise floor, distortion content in exports
  Signal: THD estimation, spectral flatness of kept sounds

Dimension 8: Simple/Complex
  Scale: [Simple] ◄───────────────────────► [Complex]
         Pure tones, single layer      Layered, evolving, dense
  How learned: Spectral variance + multiplicity in exports
  Signal: Number of spectral peaks, temporal variance

Dimension 9: Experimental/Commercial
  Scale: [Experimental] ◄───────────────► [Commercial]
         Unusual, niche, risky         Conventional, proven, safe
  How learned: Prompt uniqueness + sound feature entropy
  Signal: Rarity of generated sounds vs. training distribution

Dimension 10: Sparse/Dense
  Scale: [Sparse] ◄──────────────────────────► [Dense]
         Minimal, room, space          Full, layered, wall of sound
  How learned: Preferred tail length, reverb content
  Signal: Average decay time, spectral flux
```

### Identity Profile Structure

```rust
pub struct SonicIdentity {
    pub user_id: String,
    pub version: u32,
    
    // 10 dimensions, each 0.0 to 1.0
    pub dimensions: IdentityDimensions,
    
    // Rich metadata
    pub metadata: IdentityMetadata,
    
    // Computation tracking
    pub confidence: f64,          // 0.0 (guessing) to 1.0 (confident)
    pub signal_count: u64,        // Total signals used
    pub last_updated: DateTime<Utc>,
}

pub struct IdentityDimensions {
    pub texture_warmth: f64,           // 0=warm, 1=bright
    pub rhythmic_grid: f64,            // 0=groove, 1=grid
    pub emotional_brightness: f64,     // 0=dark, 1=bright
    pub brightness: f64,               // 0=dark, 1=bright
    pub punch: f64,                    // 0=soft, 1=punchy
    pub cleanliness: f64,              // 0=clean, 1=dirty
    pub complexity: f64,               // 0=simple, 1=complex
    pub experimentalism: f64,          // 0=commercial, 1=experimental
    pub density: f64,                  // 0=sparse, 1=dense
    pub genre_affinity: Vec<(String, f64)>,  // [(trap, 0.8), ...]
}

pub struct IdentityMetadata {
    pub top_textures: Vec<String>,
    pub top_genres: Vec<String>,
    pub signature_move: Option<String>,  // e.g., "punchy kick with long sub tail"
    pub mood_keywords: Vec<(String, f64)>,
    pub frequently_used_prompt_patterns: Vec<String>,
    pub session_consistency: f64,       // How much taste varies between sessions
}
```

---

## 3. Learning the Sonic Identity

### Update Algorithm

```rust
pub struct SonicIdentityLearner {
    db: Pool<SqliteConnection>,
    decay_factor: f64,  // 0.95 (older signals weigh less)
}

impl SonicIdentityLearner {
    /// Update identity after a user session
    pub async fn update_identity(&self, user_id: &str) -> Result<SonicIdentity> {
        // 1. Load current identity (or create default)
        let mut identity = self.load_or_create(user_id).await?;
        
        // 2. Collect session signals
        let signals = self.get_recent_signals(user_id, since_last_update).await?;
        if signals.is_empty() {
            return Ok(identity);
        }
        
        // 3. Compute each dimension from signals
        identity.dimensions.texture_warmth = self.compute_texture(&signals);
        identity.dimensions.rhythmic_grid = self.compute_rhythm(&signals);
        identity.dimensions.emotional_brightness = self.compute_emotion(&signals);
        identity.dimensions.brightness = self.compute_brightness(&signals);
        identity.dimensions.punch = self.compute_punch(&signals);
        identity.dimensions.cleanliness = self.compute_cleanliness(&signals);
        identity.dimensions.complexity = self.compute_complexity(&signals);
        identity.dimensions.experimentalism = self.compute_experimentalism(&signals);
        identity.dimensions.density = self.compute_density(&signals);
        identity.dimensions.genre_affinity = self.compute_genres(&signals);
        
        // 4. Update metadata
        identity.metadata = self.compute_metadata(&signals, &identity);
        
        // 5. Update confidence (diminishing returns)
        identity.signal_count += signals.len() as u64;
        identity.confidence = 1.0 - (-0.1 * identity.signal_count as f64).exp();
        
        // 6. Save
        identity.last_updated = Utc::now();
        identity.version += 1;
        self.save(user_id, &identity).await?;
        
        Ok(identity)
    }
    
    /// Compute brightness from spectral content of exported sounds
    fn compute_brightness(&self, signals: &[TasteSignal]) -> f64 {
        let exports: Vec<_> = signals.iter()
            .filter(|s| s.action_type == "export")
            .collect();
        
        if exports.is_empty() {
            return 0.5; // Neutral default
        }
        
        let avg_centroid: f64 = exports.iter()
            .filter_map(|s| s.features.spectral_centroid)
            .average();
        
        // Map centroid (500-5000Hz range) to 0-1 brightness
        // 500Hz → 0.0 (dark), 5000Hz → 1.0 (bright), 2000Hz → ~0.33
        ((avg_centroid - 500.0) / 4500.0).clamp(0.0, 1.0)
    }
    
    /// Compute experimental vs commercial
    fn compute_experimentalism(&self, signals: &[TasteSignal]) -> f64 {
        // Measure: how unique are the user's prompts vs. common prompts?
        let prompts: Vec<&str> = signals.iter()
            .filter_map(|s| s.prompt.as_deref())
            .collect();
        
        let uniqueness = self.prompt_uniqueness_score(&prompts);
        
        // Measure: how diverse are their sound choices?
        let type_diversity = self.sound_type_entropy(signals);
        
        // Measure: do they generate unusual combination prompts?
        let combo_rate = self.unusual_combo_rate(&prompts);
        
        // Weighted combination
        uniqueness * 0.4 + type_diversity * 0.3 + combo_rate * 0.3
    }
}
```

### Signal Feature Extraction

Each user action produces a feature vector that feeds into the identity model.

```rust
pub struct SignalFeatures {
    // Spectral
    pub spectral_centroid: Option<f64>,
    pub spectral_flatness: Option<f64>,
    pub spectral_flux: Option<f64>,
    pub high_freq_energy_ratio: Option<f64>, // Energy >8kHz / total
    
    // Temporal
    pub duration_ms: Option<f64>,
    pub attack_time_ms: Option<f64>,
    pub decay_time_ms: Option<f64>,
    pub crest_factor: Option<f64>,
    
    // Dynamic
    pub rms: Option<f64>,
    pub peak: Option<f64>,
    pub dynamic_range: Option<f64>,
    
    // Content
    pub zero_crossing_rate: Option<f64>,
    pub harmonic_ratio: Option<f64>,
    pub estimated_thd: Option<f64>,  // Total harmonic distortion
    
    // Semantic
    pub sound_type: Option<SoundType>,
    pub bpm: Option<u32>,
    pub genre_matches: Vec<String>,
    pub mood_keywords: Vec<String>,
}
```

---

## 4. How Identity Improves Generation

### Direct Generation Adaptation

```
Without Identity:
  Prompt: "kick"
  → Model generates a generic kick (average of all kicks in training data)
  → Default post-processing (flat EQ, moderate transient)
  → Random seed, no consistent character

With Identity:
  Prompt: "kick"
  
  Identity: { punch: 0.8, brightness: 0.2, sub_weight: 0.9, genre: "trap" }
  
  → Prompt augmented: "punchy trap kick heavy sub, tight tail"
  → Generation params: cfg_scale=8.5, temperature=0.9
  → Post-processing: transient +4dB, sub +3dB, high shelf -2dB
  → Seed: hash(user_id) for consistent character
  → Result: A kick that sounds like "this user's kick" — consistent across sessions
```

### Generation Scoring

```rust
pub struct IdentityAwareGenerator {
    identity: SonicIdentity,
    gateway: ModelGateway,
}

impl IdentityAwareGenerator {
    /// Generate a sound adapted to the user's identity
    pub async fn generate_for_user(
        &self,
        prompt: &str,
        config: GenerationConfig,
    ) -> Result<Vec<GenerationResult>> {
        let base_prompt = prompt.to_string();
        
        // 1. Generate 3 candidate sounds with different seeds
        let mut candidates = Vec::new();
        for _ in 0..3 {
            let adapted = self.adapt_generation(&base_prompt, &config);
            let result = self.gateway.generate(adapted).await?;
            candidates.push(result);
        }
        
        // 2. Score each candidate against user identity
        for candidate in &mut candidates {
            let identity_score = self.score_identity_match(&candidate.audio);
            candidate.identity_score = identity_score;
            candidate.sound_score = (candidate.sound_score * 0.6) 
                                  + (identity_score * 100.0 * 0.4);
        }
        
        // 3. Sort by score (highest first)
        candidates.sort_by(|a, b| b.sound_score.partial_cmp(&a.sound_score));
        
        // 4. Return top 3 (user sees highest-scored first)
        Ok(candidates.into_iter().take(3).collect())
    }
    
    /// Score how well a generated sound matches the user's identity
    pub fn score_identity_match(&self, audio: &[f32]) -> f64 {
        let features = extract_features(audio, 44100);
        let mut score = 0.0_f64;
        
        // Brightness match (10 points)
        let brightness = features.spectral_centroid.map(|c| {
            ((c - 500.0) / 4500.0).clamp(0.0, 1.0)
        }).unwrap_or(0.5);
        score += 10.0 * (1.0 - (brightness - self.identity.dimensions.brightness).abs());
        
        // Punch match (10 points) 
        let punch = features.crest_factor.map(|c| {
            ((c - 4.0) / 16.0).clamp(0.0, 1.0)
        }).unwrap_or(0.5);
        score += 10.0 * (1.0 - (punch - self.identity.dimensions.punch).abs());
        
        // Cleanliness match (10 points)
        let cleanliness = features.estimated_thd.map(|thd| {
            (1.0 - (thd / 0.1).min(1.0)) // Lower THD = cleaner
        }).unwrap_or(0.5);
        score += 10.0 * (1.0 - (cleanliness - self.identity.dimensions.cleanliness).abs());
        
        // Complexity match (10 points)
        if let Some(flux) = features.spectral_flux {
            let complexity = (flux / 0.5).min(1.0);
            score += 10.0 * (1.0 - (complexity - self.identity.dimensions.complexity).abs());
        }
        
        // Duration match (5 points)
        if let Some(dur) = features.duration_ms {
            // Normalized to 0-1 (assume 3s max)
            let dur_norm = (dur / 3000.0).min(1.0);
            if let Some(pref_dur) = self.identity.preferred_duration() {
                score += 5.0 * (1.0 - (dur_norm - pref_dur).abs());
            }
        }
        
        // Genre match (5 points, bonus)
        if let Some(genre) = features.genre() {
            if self.identity.dimensions.genre_affinity.iter()
                .any(|(g, _)| g == &genre) 
            {
                score += 5.0;
            }
        }
        
        score
    }
    
    /// Adapt generation parameters based on identity
    fn adapt_generation(&self, prompt: &str, config: &GenerationConfig) -> GenerationRequest {
        let identity = &self.identity.dimensions;
        
        let mut adapted = GenerationRequest {
            prompt: prompt.to_string(),
            config: config.clone(),
        };
        
        // Augment prompt with identity-aware descriptors
        let mut augmentations = Vec::new();
        if identity.brightness < 0.3 { augmentations.push("dark".to_string()); }
        if identity.brightness > 0.7 { augmentations.push("bright".to_string()); }
        if identity.punch > 0.7 { augmentations.push("punchy".to_string()); }
        if identity.cleanliness > 0.7 { augmentations.push("clean".to_string()); }
        if identity.cleanliness < 0.3 { augmentations.push("dirty".to_string()); }
        if identity.density > 0.7 { augmentations.push("layered".to_string()); }
        
        if !augmentations.is_empty() && !prompt.contains(&augmentations[0]) {
            adapted.prompt = format!("{} {}", prompt, augmentations.join(" "));
        }
        
        // Adapt post-processing params
        adapted.config.post_process.punch_boost = (identity.punch * 6.0) as f32;
        adapted.config.post_process.brightness = (identity.brightness * 3.0 - 1.5) as f32;
        adapted.config.post_process.warmth = ((1.0 - identity.brightness) * 2.0) as f32;
        
        // Use identity-consistent seed
        adapted.config.seed = self.identity_consistent_seed();
        
        adapted
    }
    
    /// Derive a deterministic seed from the user's identity for consistency
    fn identity_consistent_seed(&self) -> u32 {
        // Hash the identity embedding to produce a stable seed
        // This means the user gets a "consistent character" across sessions
        let hash = blake3::hash(&self.identity.user_id.as_bytes());
        let seed_bytes: [u8; 4] = [hash.as_bytes()[0], hash.as_bytes()[1], 
                                   hash.as_bytes()[2], hash.as_bytes()[3]];
        u32::from_le_bytes(seed_bytes)
    }
}
```

---

## 5. How Identity Improves Search

### Identity-Weighted Search

When searching generated sounds or future sample libraries:

```rust
pub fn identity_weighted_search(
    query: &str,
    results: Vec<SoundMetadata>,
    identity: &SonicIdentity,
) -> Vec<SoundMetadata> {
    let mut scored = results.into_iter().map(|sound| {
        let text_score = text_search_score(query, &sound);
        let identity_score = identity_match_score(&sound, identity);
        let combined = text_score * 0.6 + identity_score * 0.4;
        (sound, combined)
    }).collect::<Vec<_>>();
    
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.into_iter().map(|(s, _)| s).collect()
}

fn identity_match_score(sound: &SoundMetadata, identity: &SonicIdentity) -> f64 {
    // How well does this sound match the user's identity?
    // This boosts sounds that are "in the user's wheelhouse"
    // without filtering out genuinely different results
    
    let mut score = 0.5; // Neutral baseline
    
    // Type match: +0.2 if user frequently uses this type
    if identity.metadata.top_sound_types.contains(&sound.sound_type) {
        score += 0.2;
    }
    
    // Genre match: +0.15 if genre is familiar
    if let Some(ref genre) = sound.genre {
        if identity.dimensions.genre_affinity.iter().any(|(g, _)| g == genre) {
            score += 0.15;
        }
    }
    
    // Spectral match: ±0.15 based on brightness alignment
    if let Some(centroid) = sound.spectral_centroid {
        let sound_brightness = ((centroid - 500.0) / 4500.0).clamp(0.0, 1.0);
        let diff = (sound_brightness - identity.dimensions.brightness).abs();
        score += (1.0 - diff) * 0.15 - 0.075; // -0.075 to +0.075
    }
    
    score.clamp(0.0, 1.0)
}
```

---

## 6. How Identity Improves Packs

### Identity-Aware Pack Generation

When generating packs (Phase 2), the sonic identity shapes the entire pack's character:

```rust
pub fn generate_pack_for_user(
    user_id: &str,
    prompt: &str,
    identity: &SonicIdentity,
) -> Vec<SoundGeneration> {
    // 1. Base sounds on user identity
    let base_params = adapt_generation_params(prompt, identity);
    
    // 2. Generate variations across identity-relevant dimensions
    let mut pack = Vec::new();
    
    // Main sound (closest to identity center)
    pack.push(generate_sound(&base_params));
    
    // Variation 1: Brighter version (exploration)
    let mut bright_params = base_params.clone();
    bright_params.post_process.brightness += 1.0;
    pack.push(generate_sound(&bright_params));
    
    // Variation 2: Punchier version
    let mut punchy_params = base_params.clone();
    punchy_params.post_process.punch_boost += 2.0;
    pack.push(generate_sound(&punchy_params));
    
    // Variation 3: Darker, moodier (emotional range)
    let mut dark_params = base_params.clone();
    dark_params.post_process.brightness -= 1.5;
    dark_params.post_process.warmth += 1.0;
    pack.push(generate_sound(&dark_params));
    
    // Variation 4: Outside comfort zone (discovery)
    if identity.dimensions.experimentalism > 0.5 {
        let mut wild_params = base_params.clone();
        wild_params.temperature *= 1.5;
        pack.push(generate_sound(&wild_params));
    }
    
    pack
}
```

---

## 7. How Identity Improves DAW Workflows

### DAW Context Awareness (Phase 3)

When cShot becomes a DAW plugin, the sonic identity can inform:

```rust
/// DAW context + identity → suggested sounds
pub fn suggest_for_current_context(
    daw_context: &DawContext,
    identity: &SonicIdentity,
) -> Vec<GenerationSuggestion> {
    let mut suggestions = Vec::new();
    
    // 1. What's missing in the arrangement?
    //    (based on track analysis, frequency spectrum, arrangement)
    if daw_context.missing_low_end() {
        suggestions.push(GenerationSuggestion {
            prompt: "sub kick".to_string(),
            identity_adapted: true,
            reason: "Your track needs low-end weight",
        });
    }
    
    if daw_context.missing_mid_range() {
        suggestions.push(GenerationSuggestion {
            prompt: "mid-range percussion".to_string(),
            identity_adapted: true,
            reason: "Fill the mid-range with your signature texture",
        });
    }
    
    // 2. What's the user's signature sound?
    if let Some(ref sig) = identity.metadata.signature_move {
        suggestions.push(GenerationSuggestion {
            prompt: sig.clone(),
            identity_adapted: true,
            reason: "Your signature sound — it always works",
        });
    }
    
    // 3. What worked in similar sessions?
    //    (from generation history)
    suggestions
}
```

---

## 8. User Facing Identity

### Identity Card

Users should see their sonic identity as a badge of pride.

```
Your Sonic Identity Card:

┌─────────────────────────────────────────────────────────┐
│                    YOUR SONIC ID                         │
│                                                          │
│                    ╱ PUNCHY ╲                            │
│                  ╱            ╲                          │
│                ╱    TRAP       ╲                        │
│              ╱     PRODUCER     ╲                       │
│            ╱                    ╲                       │
│          ╱  ʀ ᴇ ᴘ s  ◇  ᴅ ᴀ ᴘ  ╲                      │
│        ╱                        ╲                       │
│      ╱      350 signals · 92%     ╲                     │
│    ╱         confidence            ╲                    │
│  ╱                                  ╲                    │
│ ╱  dark · punchy · subby · clean     ╲                  │
│─────────────────────────────────────────                  │
│                                                          │
│  Dimensions:                                             │
│  Texture:  Warm ──●──── Bright      (20% bright)        │
│  Rhythm:   Groove ──────●─ Grid     (85% grid)          │
│  Emotion:  Dark ─●────── Bright     (15% bright)        │
│  Punch:    Soft ────────● Punchy   (90% punchy)        │
│  Clean:    Clean ─●────── Dirty    (15% dirty)          │
│  Complex:  Simple ───●──── Complex   (40% complex)       │
│  Exp:      Exp ─●─────── Commercial (20% exp)           │
│  Density:  Sparse ─────●─ Dense     (70% dense)          │
│                                                          │
│  Signature: "Punchy trap kick with dark sub tail"       │
│  Evolved: Over 3 months · Last updated: 2 hours ago     │
│                                                          │
│  [Share Identity Card] [View Evolution] [Reset]         │
└─────────────────────────────────────────────────────────┘
```

### Identity Evolution Timeline

```
Your Sonic Journey:

Month 1:   ████████░░░░░░░░  "Learning the basics — kicks mostly"
Month 2:   ██████████░░░░░░  "Getting punchier, BPM creeping up"
Month 3:   ████████████░░░░  "Found your sound: dark, punchy, trap"
Month 4:   ██████████████░░  "Exploring more snares + textures"
Now:       ████████████████  "Solidified signature"

Key milestones:
  • Session 12: First "punchy" preference detected
  • Session 25: Genre narrowed to trap/hip-hop
  • Session 40: Brightness preference stabilized (dark)
  • Session 60: Signature sound detected: "punchy kick with sub"
```

---

## 9. Identity Cold Start

### Bootstrap Sequence

```rust
pub struct IdentityColdStart {
    stage: u8, // 0-5
}

impl IdentityColdStart {
    pub async fn bootstrap(mut self, user_id: &str) -> SonicIdentity {
        let mut identity = SonicIdentity::default();
        
        match self.stage {
            0 => {
                // First generation: no data, all defaults (0.5 neutral)
                identity.dimensions = IdentityDimensions::neutral();
                identity.confidence = 0.0;
            }
            1 => {
                // After 1st export: set sound type preference
                // All other dimensions remain neutral
                identity.dimensions.texture_warmth = 0.5;
                identity.dimensions.brightness = 0.5;
                identity.confidence = 0.1;
            }
            2 => {
                // After 5 exports: rough brightness + punch estimate
                identity.confidence = 0.25;
            }
            3 => {
                // After 15 exports: all dimensions have weak signal
                identity.confidence = 0.5;
            }
            4 => {
                // After 30 exports: solid identity
                identity.confidence = 0.7;
            }
            5 => {
                // After 50+ exports: stable identity
                identity.confidence = 0.85;
            }
        }
        
        identity.signal_count = self.stage as u64;
        identity.last_updated = Utc::now();
        identity
    }
}
```

### Cold-Start Prompt Inference

When there's no identity data yet, infer from the user's first prompts:

```rust
pub fn infer_from_prompt(prompt: &str) -> PartialIdentity {
    let mut identity = PartialIdentity::default();
    let lower = prompt.to_lowercase();
    
    // Brightness
    if lower.contains("dark") || lower.contains("deep") || lower.contains("sub") {
        identity.brightness = Some(0.2);
    }
    if lower.contains("bright") || lower.contains("crisp") || lower.contains("shiny") {
        identity.brightness = Some(0.8);
    }
    
    // Punch
    if lower.contains("punchy") || lower.contains("crack") || lower.contains("sharp") {
        identity.punch = Some(0.8);
    }
    if lower.contains("soft") || lower.contains("gentle") || lower.contains("round") {
        identity.punch = Some(0.2);
    }
    
    // Cleanliness
    if lower.contains("clean") || lower.contains("pristine") || lower.contains("pure") {
        identity.cleanliness = Some(0.8);
    }
    if lower.contains("dirty") || lower.contains("gritty") || lower.contains("lo-fi") {
        identity.cleanliness = Some(0.2);
    }
    
    // Genre
    let genres = ["trap", "hip-hop", "house", "techno", "lo-fi", "rock", "cinematic", "pop"];
    for genre in &genres {
        if lower.contains(genre) {
            identity.genre = Some(genre.to_string());
            break;
        }
    }
    
    identity
}
```

---

## 10. Identity Use Cases Summary

| Use Case | What Identity Provides | Value |
|----------|----------------------|-------|
| Generation | Adapted prompt, params, post-processing | Sound matches user's taste from day 1 |
| Search | Identity-weighted result ranking | "Sounds like me" results first |
| Packs | Identity-aware variation selection | Pack feels personal, not generic |
| DAW plugin | Context-aware suggestions | "Fill this gap with your sound" |
| Export naming | Identity-based organization | "punchy_kick_my_style.wav" |
| History | Identity-annotated timeline | See how your taste evolved |
| Share | Identity card as badge | "This is my sound" |
| Onboarding | Cold-start from prompt analysis | Zero-friction personalization |
| Retention | Identity gets better with use | Switching cost: "cShot knows my sound" |

---

## 11. Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                 Sonic Identity System                          │
│                                                               │
│  ┌────────────┐    ┌──────────────┐    ┌───────────────────┐  │
│  │ Signal      │───▶│ Identity     │───▶│ Generation        │  │
│  │ Collector   │    │ Learner      │    │ Adapter           │  │
│  │             │    │              │    │                   │  │
│  │ Captures    │    │ Computes 10  │    │ Modifies prompts, │  │
│  │ every user  │    │ dimensions   │    │ params, post-proc │  │
│  │ action      │    │ from signals │    │ based on identity │  │
│  └────────────┘    └──────┬───────┘    └───────────────────┘  │
│                           │                                    │
│                           ▼                                    │
│               ┌───────────────────────┐                       │
│               │ Identity Store        │                       │
│               │ - Current identity    │                       │
│               │ - History/snapshots   │                       │
│               │ - Cold-start presets  │                       │
│               └───────────────────────┘                       │
│                                                               │
│  ┌────────────┐    ┌──────────────┐                           │
│  │ Search     │    │ Pack         │                           │
│  │ Adapter    │    │ Adapter      │                           │
│  └────────────┘    └──────────────┘                           │
└──────────────────────────────────────────────────────────────┘
```
