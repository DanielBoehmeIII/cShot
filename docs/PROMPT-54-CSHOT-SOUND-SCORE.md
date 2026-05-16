# Prompt 54 — Create the cShot Sound Score

A single usefulness score for every generated one-shot. Combines technical quality, prompt alignment, and user behavior into one number cShot uses to learn what sounds are worth keeping.

---

## 1. Score Formula

```
SoundScore = (Technical × 0.25) + (Alignment × 0.20) + (Uniqueness × 0.15) 
           + (MixReadiness × 0.15) + (GenreFit × 0.10) + (UserSignal × 0.15)

Range: 0.0 (unusable) to 1.0 (perfect)
```

### Sub-Score Breakdown

| Component | Weight | Inputs | Update Frequency |
|-----------|--------|--------|-----------------|
| Technical | 0.25 | Signal features, DSP metrics | On generation |
| Alignment | 0.20 | CLAP score, keyword match | On generation |
| Uniqueness | 0.15 | Embedding similarity to library | On generation + weekly |
| Mix Readiness | 0.15 | Crest factor, headroom, noise floor | On generation |
| Genre Fit | 0.10 | Genre classifier confidence | On generation |
| User Signal | 0.15 | Favorites, exports, ratings, replays | On user interaction |

---

## 2. Sub-Score Definitions

### 2.1 Technical Quality (0.0 – 1.0)

```rust
pub struct TechnicalQuality {
    // Each sub-metric is 0.0–1.0
    pub transient_clarity: f32,
    pub spectral_balance: f32,
    pub noise_floor_quality: f32,
    pub dynamic_range: f32,
    pub duration_appropriateness: f32,
}

pub fn compute_technical(features: &SignalFeatures) -> f32 {
    let transient = transient_score(features);
    let spectral = spectral_balance_score(features);
    let noise = noise_floor_score(features);
    let dynamics = dynamic_range_score(features);
    let duration = duration_score(features);
    
    // Weighted average
    let score = transient * 0.30 + spectral * 0.25 + noise * 0.20 + dynamics * 0.15 + duration * 0.10;
    
    // Penalty for clipping or DC offset
    let penalty = if features.peak >= 1.0 { -0.2 } else { 0.0 }
                + if features.dc_offset.abs() > 0.001 { -0.1 } else { 0.0 };
    
    (score + penalty).clamp(0.0, 1.0)
}

fn transient_score(f: &SignalFeatures) -> f32 {
    // Ideal crest factor for one-shots: 10-15
    let cf = f.crest_factor;
    if cf < 4.0 { 0.1 }          // Over-compressed
    else if cf < 8.0 { 0.3 + (cf - 4.0) / 8.0 * 0.3 } // Low but usable
    else if cf < 15.0 { 0.6 + (cf - 8.0) / 7.0 * 0.4 } // Sweet spot
    else if cf < 20.0 { 1.0 - (cf - 15.0) / 5.0 * 0.2 } // A bit high
    else { 0.5 }                  // Over-expanded
}

fn spectral_balance_score(f: &SignalFeatures) -> f32 {
    // Check energy distribution across bands
    // Good: one or two dominant bands, no band > 50% of total
    // Bad: energy evenly spread (washy) OR one band dominates (unbalanced)
    let bands = [
        f.energy_sub_low, f.energy_low, f.energy_low_mid,
        f.energy_mid, f.energy_high_mid, f.energy_high, f.energy_very_high,
    ];
    let total: f32 = bands.iter().sum();
    if total == 0.0 { return 0.0; }
    
    let fractions: Vec<f32> = bands.iter().map(|b| b / total).collect();
    let max_band = fractions.iter().cloned().fold(0.0_f32, f32::max);
    let band_count = fractions.iter().filter(|&&f| f > 0.1).count();
    
    if max_band > 0.6 { 0.4 }          // One band dominates
    else if band_count <= 1 { 0.3 }    // Way too narrow
    else if band_count >= 6 { 0.4 }    // Way too spread
    else if band_count >= 3 { 1.0 }    // 3-5 active bands ideal
    else if band_count == 2 { 0.7 }    // Two bands — often fine
    else { 0.5 }
}

fn noise_floor_score(f: &SignalFeatures) -> f32 {
    // Estimate noise floor from quietest 10% of samples
    // Score: lower noise floor = higher score
    let noise_floor_db = f.rms_to_noise_floor_db();
    if noise_floor_db < -60.0 { 1.0 }
    else if noise_floor_db < -50.0 { 0.8 + (noise_floor_db + 60.0) / 10.0 * 0.2 }
    else if noise_floor_db < -40.0 { 0.5 + (noise_floor_db + 50.0) / 10.0 * 0.3 }
    else if noise_floor_db < -30.0 { 0.2 + (noise_floor_db + 40.0) / 10.0 * 0.3 }
    else { 0.1 }
}

fn dynamic_range_score(f: &SignalFeatures) -> f32 {
    // Crest factor indicates dynamic range
    let cf = f.crest_factor;
    if cf >= 6.0 && cf <= 18.0 { 1.0 }
    else if cf >= 4.0 { 0.6 }
    else { 0.3 }
}

fn duration_score(f: &SignalFeatures) -> f32 {
    // Duration appropriateness depends on sound type
    // Simplified: score based on type-expected duration
    let (ideal_min, ideal_max) = match f.sound_type.as_str() {
        "kick" => (0.15, 1.0),   // 150ms-1s
        "snare" => (0.1, 0.8),
        "hihat" => (0.05, 0.4),
        "clap" => (0.15, 0.8),
        "bass" => (0.3, 3.0),
        "perc" => (0.1, 1.5),
        "fx" => (0.5, 5.0),
        _ => (0.05, 5.0),
    };
    let dur_s = f.duration_ms / 1000.0;
    if dur_s >= ideal_min && dur_s <= ideal_max { 1.0 }
    else if dur_s < ideal_min { 0.3 + (dur_s / ideal_min) * 0.7 }
    else { 0.3 + (ideal_max / dur_s) * 0.7 }
}
```

### 2.2 Prompt Alignment (0.0 – 1.0)

```rust
pub fn compute_alignment(prompt: &str, features: &SignalFeatures, clap_score: Option<f32>) -> f32 {
    // 1. CLAP embedding similarity (if available)
    let clap = clap_score.unwrap_or(0.5);
    
    // 2. Keyword match: does the generated type match prompt keywords?
    let keyword_score = keyword_alignment(prompt, features);
    
    // 3. Attribute match: does brightness/pitch match descriptive words?
    let attribute_score = attribute_alignment(prompt, features);
    
    clap * 0.5 + keyword_score * 0.3 + attribute_score * 0.2
}

fn keyword_alignment(prompt: &str, features: &SignalFeatures) -> f32 {
    let prompt_lower = prompt.to_lowercase();
    
    let type_keywords = [
        (["kick", "kicker", "kickdrum"], "kick"),
        (["snare", "snares", "rimshot", "rim"], "snare"),
        (["hat", "hihat", "hi-hat", "cymbal", "ride", "crash"], "hihat"),
        (["clap", "claps", "handclap"], "clap"),
        (["bass", "sub", "808", "lowend"], "bass"),
        (["perc", "percussion", "shaker", "tambourine"], "perc"),
        (["fx", "effect", "riser", "impact", "sweep"], "fx"),
    ];
    
    for (keywords, expected_type) in &type_keywords {
        if keywords.iter().any(|k| prompt_lower.contains(k)) {
            if features.sound_type == *expected_type {
                return 1.0;  // Type matches
            } else {
                return 0.3;  // Type mismatch — generated wrong thing
            }
        }
    }
    
    0.5 // No type keywords found in prompt
}

fn attribute_alignment(prompt: &str, features: &SignalFeatures) -> f32 {
    let prompt_lower = prompt.to_lowercase();
    let centroid = features.spectral_centroid_hz;
    
    let mut matches = 0.0;
    let mut total = 0.0;
    
    // Brightness
    if prompt_lower.contains("bright") || prompt_lower.contains("shine") {
        total += 1.0;
        if centroid > 3000.0 { matches += 1.0; }
    }
    if prompt_lower.contains("dark") || prompt_lower.contains("warm") {
        total += 1.0;
        if centroid < 2000.0 { matches += 1.0; }
    }
    
    // Punch
    if prompt_lower.contains("punchy") || prompt_lower.contains("crack") || prompt_lower.contains("sharp") {
        total += 1.0;
        if features.crest_factor > 10.0 { matches += 1.0; }
    }
    if prompt_lower.contains("soft") || prompt_lower.contains("gentle") {
        total += 1.0;
        if features.crest_factor < 8.0 { matches += 1.0; }
    }
    
    // Duration
    if prompt_lower.contains("short") || prompt_lower.contains("tight") {
        total += 1.0;
        if features.duration_ms < 300.0 { matches += 1.0; }
    }
    if prompt_lower.contains("long") || prompt_lower.contains("ring") {
        total += 1.0;
        if features.duration_ms > 800.0 { matches += 1.0; }
    }
    
    if total == 0.0 { 0.5 } else { matches / total }
}
```

### 2.3 Uniqueness (0.0 – 1.0)

```rust
pub fn compute_uniqueness(
    features: &SignalFeatures,
    library_embeddings: &[Vec<f32>],
) -> f32 {
    if library_embeddings.is_empty() {
        return 0.8; // No library to compare against — assume unique
    }
    
    let embedding = compute_embedding(features);
    
    // Find nearest neighbor distance
    let min_distance: f32 = library_embeddings.iter()
        .map(|lib_emb| cosine_distance(&embedding, lib_emb))
        .fold(f32::MAX, f32::min);
    
    // Convert distance to uniqueness score
    // distance 0.0 (identical) → score 0.0
    // distance 0.3 → score 0.7 (sweet spot: related but different)
    // distance 0.8+ → score 1.0 (completely unique)
    if min_distance < 0.05 { 0.1 }       // Near-copy
    else if min_distance < 0.1 { 0.3 }    // Very similar
    else if min_distance < 0.2 { 0.6 }    // Moderately unique
    else if min_distance < 0.4 { 0.8 }    // Unique
    else { 1.0 }                           // Very unique
}

fn compute_embedding(features: &SignalFeatures) -> Vec<f32> {
    vec![
        features.spectral_centroid_hz / 10000.0,
        features.spectral_flatness,
        features.zero_crossing_rate,
        features.crest_factor / 20.0,
        features.attack_time_ms / 100.0,
        features.rms,
        features.harmonic_ratio,
        features.duration_ms / 5000.0,
    ]
}

fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    1.0 - dot / (norm_a * norm_b + 1e-10)
}
```

### 2.4 Mix Readiness (0.0 – 1.0)

```rust
pub fn compute_mix_readiness(features: &SignalFeatures, repaired: bool) -> f32 {
    let mut score = 0.0;
    
    // Headroom: enough space without clipping
    let headroom = 20.0 * (1.0 - features.peak).max(1e-10).log10(); // dB below 0
    if headroom >= 1.0 { score += 0.25 }
    else if headroom >= 0.5 { score += 0.15 }
    else { score += 0.05 }
    
    // No clipping
    if features.peak < 0.99 { score += 0.20 }
    else { score += 0.0 }
    
    // Crest factor: not too compressed, not too dynamic
    let cf = features.crest_factor;
    if cf >= 6.0 && cf <= 18.0 { score += 0.20 }
    else { score += 0.10 }
    
    // Noise floor: clean enough to layer
    let noise_db = features.rms_to_noise_floor_db();
    if noise_db < -55.0 { score += 0.15 }
    else if noise_db < -45.0 { score += 0.10 }
    else { score += 0.05 }
    
    // DC offset
    if features.dc_offset.abs() < 0.001 { score += 0.10 }
    else { score += 0.0 }
    
    // Zero-length check
    if features.duration_ms > 30.0 { score += 0.10 }
    else { score += 0.0 }
    
    // Repaired bonus
    if repaired { score += 0.05 }
    
    score.clamp(0.0, 1.0)
}
```

### 2.5 Genre Fit (0.0 – 1.0)

```rust
pub fn compute_genre_fit(
    prompt: &str,
    features: &SignalFeatures,
    genre_classifier: Option<&GenreClassifier>,
) -> f32 {
    // 1. If prompt specifies a genre, check agreement
    let prompt_genre = extract_genre_from_prompt(prompt);
    
    // 2. Classify genre from audio features
    let audio_genre = genre_classifier
        .and_then(|c| c.predict(features))
        .unwrap_or_default();
    
    match (prompt_genre, audio_genre) {
        (Some(pg), Some(ag)) if pg == ag => 1.0,  // Agreement
        (Some(pg), Some(ag)) => 0.3,               // Disagreement
        (Some(_), None) => 0.5,                     // Prompt specified, can't verify
        (None, Some(_)) => 0.7,                     // Audio has genre, user didn't ask
        (None, None) => 0.5,                        // Nothing to compare
    }
}

fn extract_genre_from_prompt(prompt: &str) -> Option<String> {
    let genres = [
        "trap", "house", "techno", "lo-fi", "lofi", "rock", "metal",
        "pop", "rnb", "r&b", "jazz", "blues", "country", "folk",
        "ambient", "drone", "cinematic", "orchestral", "electronic",
        "dubstep", "dnb", "drum and bass", "garage", "uk garage",
        "soul", "funk", "disco", "latin", "reggae", "dub",
    ];
    
    let lower = prompt.to_lowercase();
    for genre in &genres {
        if lower.contains(genre) {
            return Some(genre.to_string());
        }
    }
    None
}
```

### 2.6 User Signal (Dynamic, Starts Neutral)

```rust
pub fn compute_user_signal(sound: &Sound) -> f32 {
    let mut score = 0.5; // Neutral start
    
    // Each positive signal increases score
    // Each negative signal decreases score
    
    // Favorited
    if sound.is_favorited {
        score += 0.15;
    }
    
    // Exported (strongest signal — user actually used it)
    if sound.export_count > 0 {
        score += 0.25;
    }
    if sound.export_count > 3 {
        score += 0.10; // Repeated export = very useful
    }
    
    // Replayed (listened more than once)
    if sound.play_count > 1 {
        score += 0.05;
    }
    if sound.play_count > 5 {
        score += 0.05;
    }
    
    // User rating
    if let Some(rating) = sound.user_rating {
        let rating_score = (rating as f32 - 1.0) / 3.0; // Normalize 1-4 to 0-1
        score += rating_score * 0.20;
    }
    
    // Negative signals
    if sound.is_unfavorited {
        score -= 0.10;
    }
    if sound.preview_abandoned { // Played <30% of duration
        score -= 0.05;
    }
    
    score.clamp(0.0, 1.0)
}
```

---

## 3. Confidence Levels

```rust
pub struct SoundScore {
    pub overall: f32,           // 0.0-1.0
    pub technical: f32,
    pub alignment: f32,
    pub uniqueness: f32,
    pub mix_readiness: f32,
    pub genre_fit: f32,
    pub user_signal: f32,
    pub confidence: Confidence, // How reliable is this score?
    pub computed_at: String,
}

pub enum Confidence {
    Initial,    // Just generated — no user feedback yet
    Low,        // <3 interactions
    Medium,     // 3-10 interactions
    High,       // >10 interactions or exported
    Finalizing, // User has explicitly rated
}

impl SoundScore {
    pub fn confidence_level(&self) -> &str {
        match self.confidence {
            Confidence::Initial => "initial",
            Confidence::Low => "low",
            Confidence::Medium => "medium",
            Confidence::High => "high",
            Confidence::Finalizing => "final",
        }
    }
    
    pub fn display_score(&self) -> String {
        let icon = if self.confidence as u8 <= Confidence::Low as u8 {
            "~" // Tilde = approximate
        } else {
            ""  // Stable
        };
        
        let stars = match self.overall {
            s if s >= 0.9 => "★★★★★",
            s if s >= 0.7 => "★★★★",
            s if s >= 0.5 => "★★★",
            s if s >= 0.3 => "★★",
            _ => "★",
        };
        
        format!("{}{:.0}% {}", icon, self.overall * 100.0, stars)
    }
}
```

---

## 4. Score Computation Pipeline

```rust
pub struct ScoreEngine {
    library_embeddings: Vec<Vec<f32>>,
    genre_classifier: Option<GenreClassifier>,
}

impl ScoreEngine {
    pub async fn compute(
        &self,
        prompt: &str,
        features: &SignalFeatures,
        sound: &Sound,
        clap_score: Option<f32>,
    ) -> SoundScore {
        // Run all sub-scores (most are O(1) or O(n) in feature vector)
        let technical = compute_technical(features);
        let alignment = compute_alignment(prompt, features, clap_score);
        let uniqueness = compute_uniqueness(features, &self.library_embeddings);
        let mix_readiness = compute_mix_readiness(features, sound.was_repaired);
        let genre_fit = compute_genre_fit(prompt, features, self.genre_classifier.as_ref());
        let user_signal = compute_user_signal(sound);
        
        let overall = technical * 0.25
                    + alignment * 0.20
                    + uniqueness * 0.15
                    + mix_readiness * 0.15
                    + genre_fit * 0.10
                    + user_signal * 0.15;
        
        SoundScore {
            overall: overall.clamp(0.0, 1.0),
            technical,
            alignment,
            uniqueness,
            mix_readiness,
            genre_fit,
            user_signal,
            confidence: score_confidence(sound),
            computed_at: Utc::now().to_rfc3339(),
        }
    }
}

fn score_confidence(sound: &Sound) -> Confidence {
    let interaction_count = sound.play_count + sound.export_count + sound.rating_count;
    
    if sound.export_count > 0 { Confidence::High }
    else if interaction_count > 10 { Confidence::Medium }
    else if interaction_count >= 3 { Confidence::Low }
    else { Confidence::Initial }
}
```

---

## 5. Database Storage

```sql
-- Sound scores, updated on generation and on user interaction
CREATE TABLE sound_scores (
    sound_id        TEXT PRIMARY KEY,
    overall         REAL NOT NULL,     -- 0.0-1.0
    technical       REAL NOT NULL,
    alignment       REAL NOT NULL,
    uniqueness      REAL NOT NULL,
    mix_readiness   REAL NOT NULL,
    genre_fit       REAL NOT NULL,
    user_signal     REAL NOT NULL,
    confidence      TEXT NOT NULL,     -- initial, low, medium, high, final
    computed_at     TEXT NOT NULL,
    version         INTEGER DEFAULT 1, -- for tracking score formula changes
    
    -- Sub-metrics for debugging
    crest_factor    REAL,
    spectral_centroid REAL,
    clap_score      REAL,
    nearest_neighbor_distance REAL,
    interaction_count INTEGER DEFAULT 0,
    
    FOREIGN KEY (sound_id) REFERENCES sounds(id)
);

-- Score history (for seeing how scores change over time)
CREATE TABLE score_history (
    id              TEXT PRIMARY KEY,
    sound_id        TEXT NOT NULL,
    overall         REAL NOT NULL,
    confidence      TEXT NOT NULL,
    trigger_event   TEXT NOT NULL,      -- 'generation', 'favorite', 'export', 'rating', 'replay'
    computed_at     TEXT NOT NULL,
    FOREIGN KEY (sound_id) REFERENCES sounds(id)
);

-- Track which sounds score well for different prompt categories
CREATE VIEW high_scoring_prompts AS
SELECT
    g.prompt,
    AVG(ss.overall) AS avg_score,
    COUNT(*) AS generation_count,
    SUM(CASE WHEN i.interaction_type = 'export' THEN 1 ELSE 0 END) AS export_count
FROM generations g
JOIN sounds s ON s.generation_id = g.id
JOIN sound_scores ss ON ss.sound_id = s.id
LEFT JOIN interactions i ON i.sound_id = s.id
GROUP BY g.prompt
HAVING generation_count >= 3
ORDER BY avg_score DESC;
```

---

## 6. Dashboard Visualization

### Sound Score Card

```
┌──────────────────────────────────────────────┐
│  Sound Score: 78% ★★★★                      │
│                                              │
│  Technical    ████████████▉   89%  ▲ +4%     │
│  Alignment    █████████     75%  ▲ +2%     │
│  Uniqueness   ████████▉     72%  ▼ -1%     │
│  Mix Ready    ████████████   82%  — same    │
│  Genre Fit    █████████▋     78%  — same    │
│  User Signal  ██████▉        55%  ▲ +15%   │
│                                              │
│  Confidence: High (12 interactions)          │
│  Last updated: 2 min ago                     │
└──────────────────────────────────────────────┘
```

### Library Overview

```
┌──────────────────────────────────────────────┐
│  LIBRARY BY SCORE                            │
│                                              │
│  Top Sound Types:                            │
│  Kick      ████████████████▉  82% avg        │
│  Snare     ██████████████    70% avg         │
│  Hat       ████████████████  78% avg         │
│  Bass      ███████████████   74% avg         │
│  Perc      ███████████▉      60% avg         │
│                                              │
│  Best Prompt: "punchy kick 140bpm" — 91%     │
│  Worst Prompt: "ambient pad" — 42%           │
│                                              │
│  Score Distribution:                         │
│  90-100%: ■■■■■■■  (12 sounds)              │
│  70-89%:  ■■■■■■■■■■■  (28 sounds)          │
│  50-69%:  ■■■■■■  (15 sounds)               │
│  30-49%:  ■■  (5 sounds)                     │
│  0-29%:   ■  (2 sounds)                      │
└──────────────────────────────────────────────┘
```

### Score Trends

```
┌──────────────────────────────────────────────┐
│  SCORE TREND — Last 30 Days                  │
│                                              │
│  Avg Score ░░░░░░░▒▒▒▒▓▓▓▓▓▓▓▓██████        │
│  ════════════════════════════════════════►   │
│  Week 1    Week 2    Week 3    Week 4        │
│                                              │
│  ▲ Score improving (model tuning working)    │
│  ▲ User signal now converged with technical  │
│     (users agree with the algorithm)          │
│  ▼ Uniqueness dropping slightly              │
│     (model generating more similar sounds)   │
└──────────────────────────────────────────────┘
```

---

## 7. Learning Loop

```
User generates sound
    │
    ▼
Technical Score computed (instant)
Alignment Score computed (instant)
Uniqueness Score computed (instant)
Mix Readiness Score computed (instant)
Genre Fit Score computed (instant)
    │
    ▼
Overall SoundScore (all auto, confidence: initial)
    │
    ▼
User interacts with sound:
    ├── Preview → replay counter +1
    ├── Favorite → user_signal +0.15
    ├── Export → user_signal +0.25, confidence → high
    ├── Rating → user_signal weighted by rating
    └── Unfavorite → user_signal -0.10
    │
    ▼
Recompute score → confidence increases
    │
    ▼
Weekly batch:
    ├── Recompute uniqueness against full library
    ├── Recalibrate weights based on export prediction accuracy
    │   (if score predicts exports, but low-score sounds get exported,
    │    the weights are wrong — adjust)
    └── Update genre classifier with new labeled data
```

### Weight Optimization

```rust
/// Every week, check if the score weights predict export behavior.
/// If low-scoring sounds are being exported, the formula is wrong.
pub fn optimize_weights(sounds: &[Sound]) -> WeightAdjustment {
    let mut miscalibrations = Vec::new();
    
    for sound in sounds {
        let predicted_score = sound.score.overall;
        let actual_exported = sound.export_count > 0;
        
        // A sound scoring <0.4 that gets exported means the formula
        // undervalues something about this sound
        if predicted_score < 0.4 && actual_exported {
            miscalibrations.push(Miscalibration {
                sound_id: sound.id.clone(),
                predicted: predicted_score,
                exported: true,
                // Which sub-scores were low?
                low_scores: [
                    ("technical", sound.score.technical < 0.4),
                    ("alignment", sound.score.alignment < 0.4),
                    ("uniqueness", sound.score.uniqueness < 0.4),
                    ("mix_readiness", sound.score.mix_readiness < 0.4),
                    ("genre_fit", sound.score.genre_fit < 0.4),
                    ("user_signal", sound.score.user_signal < 0.4),
                ].iter().filter(|(_, low)| *low).map(|(n, _)| *n).collect(),
            });
        }
    }
    
    // If exports are consistently happening despite low scores in
    // a specific dimension, that dimension's weight is too high
    let mut weight_adjustments = HashMap::new();
    for m in &miscalibrations {
        for dim in &m.low_scores {
            *weight_adjustments.entry(*dim).or_insert(0) += 1;
        }
    }
    
    WeightAdjustment {
        adjustments: weight_adjustments,
        sample_size: miscalibrations.len(),
    }
}
```

---

## 8. Summary

The SoundScore combines 6 weighted dimensions into one number. Technical quality, prompt alignment, and mix readiness are computed instantly from audio features. Uniqueness compares against the user's library. User signal starts neutral and improves with favorites, exports, and ratings. Confidence tracks how reliable the score is. The score updates on every interaction and recalibrates weekly. cShot uses it to surface the best sounds, identify weak prompts, and learn what makes a one-shot actually useful.
