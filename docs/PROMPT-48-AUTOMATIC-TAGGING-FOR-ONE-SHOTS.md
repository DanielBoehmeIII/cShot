# Prompt 48 — Automatic Tagging for One-Shots

Infer sound characteristics from audio analysis + AI classification to auto-tag every generated sound.

---

## 1. Tag Categories

| Category | Tags | Inference Method | Confidence |
|----------|------|-----------------|------------|
| Sound type | kick, snare, hihat, clap, perc, bass, fx, vocal, other | Spectral classifier | High |
| Genre fit | trap, house, techno, lo-fi, rock, cinematic, 808, ambient | Embedding similarity | Medium |
| Mood | aggressive, dark, bright, warm, soft, eerie, epic, neutral | Spectral + embedding | Medium |
| Brightness | dark, warm, neutral, bright, piercing | Spectral centroid | High |
| Punch | soft, moderate, punchy, aggressive | Transient analysis | High |
| Length | short (<200ms), medium (200-1000ms), long (>1000ms) | Duration | High |
| Texture | clean, distorted, noisy, lo-fi, metallic, wooden | Spectral shape + ZCR | Medium |
| Loudness | quiet, moderate, loud, saturated | RMS + crest factor | High |
| Dynamics | compressed, natural, dynamic | Crest factor ratio | Medium |
| Stereo width | mono, narrow, wide, stereo | Channel correlation | N/A (mono first) |
| Transient | soft attack, moderate, sharp attack, click | Onset strength | High |
| Harmonic content | tonal, noisy, mixed | Spectral flatness | High |
| Pitch | high, mid, low, sub | Spectral centroid + pitch detection | Medium |
| Energy | low energy, moderate, high energy, intense | RMS envelope | Medium |

---

## 2. Signal-Processing Features

### Extracted Features (computed in Rust)

```rust
pub struct SignalFeatures {
    // Temporal
    pub duration_ms: f32,
    pub rms: f32,                    // Root mean square amplitude
    pub peak: f32,                   // Peak amplitude
    pub crest_factor: f32,           // Peak / RMS ratio
    pub zero_crossing_rate: f32,     // Per-sample zero crossings
    pub dc_offset: f32,
    
    // Spectral
    pub spectral_centroid_hz: f32,   // "Brightness" center of mass
    pub spectral_rolloff_hz: f32,    // Frequency below which 85% energy
    pub spectral_flatness: f32,      // Tonal vs. noise-like (0=tonal, 1=noise)
    pub spectral_bandwidth_hz: f32,  // Width of spectrum
    pub spectral_contrast: Vec<f32>, // Energy in each octave band
    pub mel_spectrum: Vec<f32>,      // 128-band mel energies
    
    // Temporal envelope
    pub attack_time_ms: f32,         // Time from 10% to 90% of max envelope
    pub decay_time_ms: f32,          // Time from 90% to 50%
    pub release_time_ms: f32,        // Time from 50% to 10%
    pub envelope_shape: EnvelopeType, // short_attack, long_release, etc.
    
    // Onset/transient
    pub onset_strength: f32,         // Peak onset detection function value
    pub num_onsets: u32,             // Number of detected onsets
    pub transient_ratio: f32,        // Transient energy / total energy
    
    // Pitch
    pub estimated_pitch_hz: Option<f32>,
    pub pitch_confidence: f32,
    pub harmonic_ratio: f32,         // Energy in harmonic partials vs. noise
    
    // Energy bands
    pub energy_sub_low: f32,         // 20-60 Hz
    pub energy_low: f32,             // 60-250 Hz
    pub energy_low_mid: f32,         // 250-500 Hz
    pub energy_mid: f32,             // 500-2000 Hz
    pub energy_high_mid: f32,        // 2000-4000 Hz
    pub energy_high: f32,            // 4000-8000 Hz
    pub energy_very_high: f32,       // 8000-20000 Hz
}
```

### Feature Extraction Functions

```rust
impl SignalFeatures {
    pub fn extract(audio: &[f32], sample_rate: u32) -> Self;
    pub fn extract_spectral(audio: &[f32], sample_rate: u32) -> SpectralFeatures;
    pub fn extract_envelope(audio: &[f32], sample_rate: u32) -> EnvelopeFeatures;
    pub fn extract_onsets(audio: &[f32], sample_rate: u32) -> OnsetFeatures;
    pub fn extract_pitch(audio: &[f32], sample_rate: u32) -> PitchFeatures;
    pub fn extract_energy_bands(audio: &[f32], sample_rate: u32) -> EnergyBands;
}
```

---

## 3. Rule-Based Tagging (Prototype Phase)

Tag rules derived directly from signal features. No ML needed:

```rust
pub fn rule_based_tags(features: &SignalFeatures, prompt: &str) -> Vec<Tag> {
    let mut tags = Vec::new();
    
    // Sound type
    tags.push(infer_sound_type(features));
    
    // Duration
    if features.duration_ms < 200.0 { tags.push(tag("short", 1.0)); }
    else if features.duration_ms < 1000.0 { tags.push(tag("medium", 1.0)); }
    else { tags.push(tag("long", 1.0)); }
    
    // Brightness
    if features.spectral_centroid_hz < 500.0 { tags.push(tag("dark", 0.9)); }
    else if features.spectral_centroid_hz < 2000.0 { tags.push(tag("warm", 0.7)); }
    else if features.spectral_centroid_hz < 4000.0 { tags.push(tag("bright", 0.8)); }
    else { tags.push(tag("piercing", 0.8)); }
    
    // Punch (crest factor + attack time)
    if features.crest_factor > 15.0 && features.attack_time_ms < 5.0 {
        tags.push(tag("punchy", 0.9));
    } else if features.crest_factor > 10.0 {
        tags.push(tag("moderate", 0.6));
    } else {
        tags.push(tag("soft", 0.7));
    }
    
    // Texture
    if features.spectral_flatness > 0.8 {
        tags.push(tag("noisy", 0.8));
    } else if features.spectral_flatness < 0.2 {
        tags.push(tag("clean", 0.7));
    } else if features.zero_crossing_rate > 0.3 {
        tags.push(tag("bright", 0.6));
    }
    
    // Energy
    if features.rms > 0.3 { tags.push(tag("loud", 0.9)); }
    else if features.rms < 0.05 { tags.push(tag("quiet", 0.8)); }
    
    // Dynamics
    if features.crest_factor > 12.0 {
        tags.push(tag("dynamic", 0.7));
    } else if features.crest_factor < 8.0 {
        tags.push(tag("compressed", 0.7));
    }
    
    // Pitch region (from spectral centroid)
    if features.estimated_pitch_hz.is_some() {
        let pitch = features.estimated_pitch_hz.unwrap();
        if pitch < 100.0 { tags.push(tag("sub", 0.8)); }
        else if pitch < 250.0 { tags.push(tag("low", 0.7)); }
        else if pitch < 500.0 { tags.push(tag("mid", 0.6)); }
        else { tags.push(tag("high", 0.7)); }
    }
    
    // Prompt keyword extraction
    let prompt_lower = prompt.to_lowercase();
    if prompt_lower.contains("trap") { tags.push(tag("trap", 0.8)); }
    if prompt_lower.contains("house") { tags.push(tag("house", 0.8)); }
    if prompt_lower.contains("techno") { tags.push(tag("techno", 0.8)); }
    if prompt_lower.contains("808") { tags.push(tag("808", 0.9)); }
    if prompt_lower.contains("lo-fi") || prompt_lower.contains("lofi") {
        tags.push(tag("lo-fi", 0.8));
    }
    if prompt_lower.contains("cinematic") { tags.push(tag("cinematic", 0.8)); }
    if prompt_lower.contains("ambient") { tags.push(tag("ambient", 0.8)); }
    
    tags
}

pub fn infer_sound_type(features: &SignalFeatures) -> Tag {
    // Decision tree based on spectral + temporal features
    let centroid = features.spectral_centroid_hz;
    let zcr = features.zero_crossing_rate;
    let crest = features.crest_factor;
    let attack = features.attack_time_ms;
    let sub = features.energy_sub_low;
    let high = features.energy_high + features.energy_very_high;
    
    let type_name = if crest > 15.0 && centroid < 2000.0 && attack < 5.0 {
        if sub > 0.4 { "kick" } else { "snare" }
    } else if zcr > 0.2 && high > 0.3 && features.duration_ms < 500.0 {
        "hihat"
    } else if crest > 10.0 && centroid > 3000.0 && attack < 10.0 {
        "clap"
    } else if features.duration_ms > 2000.0 && centroid < 500.0 {
        "bass"
    } else if features.num_onsets > 3 {
        "percussion"
    } else if features.spectral_flatness > 0.7 && zcr > 0.25 {
        "fx"
    } else {
        "other"
    };
    
    tag(type_name, 0.7) // Lower confidence for rule-based
}
```

---

## 4. AI Classification Methods (MVP+ Phase)

### Method 1: k-NN Embedding Similarity

```rust
pub struct EmbeddingClassifier {
    // Pre-computed embeddings for known one-shot classes
    reference_embeddings: HashMap<String, Vec<Vec<f32>>>,
    // Pre-computed genre embeddings
    genre_embeddings: HashMap<String, Vec<f32>>,
}

impl EmbeddingClassifier {
    pub fn new(reference_db: &[TaggedSound]) -> Self;
    
    pub fn classify(&self, features: &SignalFeatures) -> Vec<Tag> {
        // Compute embedding from signal features
        let embedding = self.compute_embedding(features);
        
        let mut tags = Vec::new();
        
        // Find nearest sound type
        for (type_name, type_embeddings) in &self.reference_embeddings {
            let similarities: Vec<f32> = type_embeddings.iter()
                .map(|ref_emb| cosine_similarity(&embedding, ref_emb))
                .collect();
            let max_sim = similarities.iter().cloned().fold(0.0f32, f32::max);
            
            if max_sim > 0.7 {
                tags.push(tag(type_name, max_sim));
            }
        }
        
        // Find nearest genre
        for (genre, genre_emb) in &self.genre_embeddings {
            let sim = cosine_similarity(&embedding, genre_emb);
            if sim > 0.6 {
                tags.push(tag(genre, sim));
            }
        }
        
        tags
    }
    
    fn compute_embedding(&self, features: &SignalFeatures) -> Vec<f32> {
        // Concatenate key features into a fixed-size vector
        let mut emb = Vec::new();
        emb.push(features.spectral_centroid_hz / 10000.0);
        emb.push(features.spectral_flatness);
        emb.push(features.zero_crossing_rate);
        emb.push(features.crest_factor / 20.0);
        emb.push(features.attack_time_ms / 100.0);
        emb.push(features.decay_time_ms / 500.0);
        emb.push(features.rms);
        emb.push(features.harmonic_ratio);
        emb.extend_from_slice(&features.energy_bands_normalized());
        emb.extend_from_slice(&features.spectral_contrast_normalized());
        emb
    }
}
```

### Method 2: Lightweight Neural Classifier

```rust
// Small ONNX model for sound type + attribute classification
// Input: 128×128 mel-spectrogram
// Output: 8 sound types + 16 attribute probabilities
// Model size: ~5MB
// Inference: <10ms on CPU

pub struct NeuralClassifier {
    session: ort::Session,
}

impl NeuralClassifier {
    pub fn new(model_path: &Path) -> Result<Self>;
    
    pub fn predict(&self, mel_spectrogram: &[f32]) -> Result<ClassificationResult>;
    
    pub fn predict_from_audio(&self, audio: &[f32], sample_rate: u32) -> Result<ClassificationResult>;
}

pub struct ClassificationResult {
    pub sound_type: String,
    pub sound_type_confidence: f32,
    pub attributes: Vec<AttributePrediction>,
    pub all_probs: Vec<f32>,  // Raw output probabilities
}

pub struct AttributePrediction {
    pub name: String,
    pub probability: f32,
}
```

### Method 3: CLAP Embedding (Text-Audio Alignment)

```rust
// Use CLAP model to compute text-audio alignment
// Compare generated audio embedding against known tag embeddings

pub struct ClapTagger {
    model: ClapModel,
    tag_embeddings: HashMap<String, Vec<f32>>, // Pre-computed tag text embeddings
}

impl ClapTagger {
    pub fn new() -> Self;
    
    pub fn tag_sound(&self, audio: &[f32], sample_rate: u32) -> Vec<Tag> {
        // Compute audio embedding
        let audio_emb = self.model.encode_audio(audio, sample_rate);
        
        // Compare against all tag embeddings
        let mut results: Vec<(String, f32)> = self.tag_embeddings.iter()
            .map(|(tag, tag_emb)| (tag.clone(), cosine_similarity(&audio_emb, tag_emb)))
            .collect();
        
        // Sort by similarity, keep above threshold
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.into_iter()
            .filter(|(_, score)| *score > 0.25)
            .take(10)
            .map(|(tag, score)| Tag { name: tag, source: "model", confidence: score })
            .collect()
    }
}
```

---

## 5. Tag Schema (Database)

```sql
CREATE TABLE tags (
    sound_id    TEXT NOT NULL REFERENCES sounds(id) ON DELETE CASCADE,
    tag         TEXT NOT NULL,
    source      TEXT NOT NULL DEFAULT 'auto',  -- 'auto' (rule), 'model' (AI), 'user' (manual)
    confidence  REAL DEFAULT 1.0,              -- 0.0-1.0
    created_at  TEXT NOT NULL,
    PRIMARY KEY (sound_id, tag, source)
);

-- Tag metadata
CREATE TABLE tag_vocabulary (
    tag         TEXT PRIMARY KEY,
    category    TEXT NOT NULL,     -- type, genre, mood, texture, dynamics, etc.
    description TEXT,
    color       TEXT,              -- For UI tag display
    hidden      INTEGER DEFAULT 0  -- Hide from auto-suggest
);

-- Popular tags for auto-complete
CREATE VIEW popular_tags AS
SELECT tag, COUNT(*) as count, category
FROM tags
JOIN tag_vocabulary ON tags.tag = tag_vocabulary.tag
GROUP BY tag
ORDER BY count DESC;
```

---

## 6. Confidence Scores

| Score Range | Meaning | UI Treatment |
|-------------|---------|-------------|
| 0.9-1.0 | Certain (e.g., duration <200ms → "short") | Solid color, no question |
| 0.7-0.9 | High confidence (e.g., spectral centroid → "bright") | Normal display |
| 0.5-0.7 | Medium (e.g., genre by embedding) | Slightly dimmed |
| 0.3-0.5 | Low (model guess) | Dimmed, dotted border |
| <0.3 | Below threshold, not shown | Hidden unless user expands |

---

## 7. User Tag Editing

### UI Behavior

```
Tags on a sound slot:
  ┌──────────────────────────────────┐
  │ kick  bright  punchy  short      │ ← auto-tags (solid)
  │ trap  ★                           │ ← user-tags (starred)
  │ [+ Add tag...]                    │ ← inline input
  └──────────────────────────────────┘

Click × on auto-tag: Keep tag but mark as user-rejected (don't auto-apply again)
Click × on user-tag: Remove tag
Click +: Type new tag, Enter to add
Double-click tag: Edit in-place

Auto-suggest dropdown:
  - Shows popular tags matching typed text
  - Shows previous user tags first
  - Shows auto-tags from tag_vocabulary second
```

### Tag Merge Logic

```rust
pub fn merge_tags(sound_id: &str, auto_tags: &[Tag], existing_user_tags: &[Tag]) -> Vec<Tag> {
    // 1. Start with all auto-tags (confidence > threshold)
    let mut merged: Vec<Tag> = auto_tags.iter()
        .filter(|t| t.confidence > CONFIDENCE_THRESHOLD)
        .cloned()
        .collect();
    
    // 2. Add user tags (always included)
    for user_tag in existing_user_tags {
        // Replace auto-tag with user version if same name
        merged.retain(|t| t.tag != user_tag.tag || t.source == "user");
        merged.push(user_tag.clone());
    }
    
    // 3. Remove user-rejected tags
    merged.retain(|t| !is_user_rejected(sound_id, &t.tag));
    
    // 4. Deduplicate and sort
    merged.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    merged.dedup_by(|a, b| a.tag == b.tag);
    
    merged
}
```

---

## 8. Implementation Order

```
Phase 1 — Prototype (rule-based only):
  1. extract SignalFeatures from audio
  2. rule_based_tags() → type, duration, brightness, punch
  3. Prompt keyword extraction → genre tags
  4. Save tags to favorites.json

Phase 2 — MVP:
  5. k-NN embedding classifier with reference database
  6. confidence scores on all tags
  7. Tag vocabulary table in SQLite
  8. User tag CRUD (add/edit/remove/reject)
  9. Tag search/filter in library
  10. Auto-suggest from vocabulary + history

Phase 3 — Post-MVP:
  11. Neural classifier (ONNX, mel-spectrogram input)
  12. CLAP embedding-based tagging
  13. Mood prediction (requires labeled dataset)
  14. User feedback loop: accepted/rejected tags improve model
  15. Batch re-tagging (re-compute tags for entire library)
```

---

## 9. Tag Pipeline Flow

```
Audio (f32 buffer, sample_rate)
    │
    ▼
┌─────────────────────┐
│ SignalFeatures       │  ← Rust, real-time, <10ms
│ .extract()           │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Rule-Based Tags      │  ← Rust, instant
│ (type, duration,     │
│  brightness, punch,  │
│  texture, energy)    │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Keyword Tags         │  ← Rust, instant
│ (from prompt text)   │
│ genre, style, mood   │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Embedding Tags       │  ← Rust/Python (MVP+)
│ (k-NN on features)   │  ~5ms
│ genre, mood, texture │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Neural Tags          │  ← ONNX (post-MVP)
│ (mel-spectrogram     │  ~10ms
│  → classifier)       │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ Merge + Dedup        │  ← Rust
│ Remove rejected      │
│ Apply confidence     │
│ Sort by confidence   │
└────────┬────────────┘
         │
         ▼
    Final tags (saved to DB)
```

---

## 10. Summary

The auto-tagging system starts simple (rule-based from signal features) and grows into AI classification. Even the rule-based version provides useful tags: sound type, duration, brightness, and punch. These come at zero ML cost and <1ms computation time. AI methods layer on top for genre, mood, and texture — adding confidence scores so users know when to trust them.

The tag system is designed for iteration: every user edit (accept, reject, add, remove) is data for improving the model.
