# Prompt 63 — Build User Taste Memory

Design cShot's user taste memory system. Learn from every user action to deliver better sounds over time.

---

## 1. What Taste Memory Learns From

### Signal Sources

Every user interaction is a signal about their taste. The system collects 8 signal types:

```
Signal Type          | Example                     | Weight | Reliability
─────────────────────|─────────────────────────────|────────|───────────
★ Export             | User exports a sound        | +1.0  | Very high
♥ Favorite           | User hearts a sound         | +0.8  | Very high
✗ Delete             | User deletes a generation   | -0.7  | High
→ Skip               | User generates then ignores | -0.3  | Medium
★★★★★ Rating (explicit)| User rates 1-5 stars       | ±1.0  | Very high
✏ Prompt edit        | User changes prompt pattern | ±0.5  | Medium
📁 Genre selection    | User picks a genre chip     | +0.6  | High
🔊 Preview duration   | User previews >10s = interest| +0.4  | Low
```

### What Gets Learned

| Attribute | Data Source | Update Frequency |
|-----------|------------|-----------------|
| Preferred sound types | Export/favorite ratios by type | Per session |
| Preferred BPM range | BPM from prompts + reference analysis | Per session |
| Spectral brightness | Average centroid of exported sounds | Per session |
| Punch preference | Average crest factor of favorites | Per session |
| Sub-weight preference | Average low-end energy of exports | Per session |
| Genre affinity | Genre keyword frequency in prompts | Per session |
| Duration preference | Average exported duration | Per session |
| Prompt style | Prompt structure patterns (verbosity, specificity) | Rolling |
| Reference usage | Frequency + effectiveness of reference uploads | Rolling |
| Time-of-day patterns | When user generates what types | Rolling |

---

## 2. Taste Embedding Architecture

### Embedding Structure

```
User Taste Embedding (768-dimensional vector)

Each user has one taste embedding that summarizes their sonic preferences.
The embedding is updated after every session using implicit signals.

┌─────────────────────────────────────────────────────────────────┐
│ User Taste Embedding (768-d)                                     │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │ Audio Feature Preferences (256-d)                        │    │
│  │ - Preferred spectral centroid                             │    │
│  │ - Preferred transient strength                            │    │
│  │ - Preferred low-end energy                                │    │
│  │ - Preferred brightness                                    │    │
│  │ - Preferred dynamic range                                 │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │ Genre/Style Preferences (256-d)                          │    │
│  │ - Genre embedding (weighted by export rate)              │    │
│  │ - Style attributes (clean/dirty, experimental/commercial) │    │
│  │ - Texture preferences                                    │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │ Interaction Patterns (256-d)                             │    │
│  │ - Prompt style (verbose vs terse, specific vs abstract)  │    │
│  │ - Exploration rate (try new sounds vs stick to favorites)│    │
│  │ - Session patterns (quick gen vs deep exploration)       │    │
│  └──────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Embedding Update Algorithm

```rust
pub struct TasteEmbedding {
    embedding: Vec<f32>,           // 768-d vector
    metadata: TasteMetadata,
    last_updated: DateTime<Utc>,
    generation_count: u64,
}

pub struct TasteMetadata {
    // Preference aggregates
    pub pref_sound_types: Vec<(SoundType, f64)>,        // [(Kick, 0.7), (Snare, 0.2), ...]
    pub pref_bpm_range: (u32, u32),                     // (120, 150)
    pub pref_brightness: f64,                           // 0 (dark) to 1 (bright)
    pub pref_punch: f64,                                // 0 (soft) to 1 (punchy)
    pub pref_sub_weight: f64,                           // 0 (thin) to 1 (heavy sub)
    pub pref_genres: Vec<(String, f64)>,                // [("trap", 0.6), ...]
    pub pref_duration_ms: f64,                          // preferred sound length
    pub pref_cleanliness: f64,                          // 0 (lo-fi/dirty) to 1 (pristine)
    pub pref_complexity: f64,                           // 0 (simple) to 1 (complex/layered)
    pub exploration_rate: f64,                          // 0 (stubborn) to 1 (exploratory)
}

impl TasteEmbedding {
    /// Update embedding based on a user action
    pub fn update(&mut self, action: &UserAction, sound: &SoundFeatures) {
        let weight = action.signal_weight();
        
        // 1. Update feature preferences
        for (dim, value) in sound.feature_vector().iter().enumerate() {
            let delta = (value - self.embedding[dim]) * weight * 0.1;
            self.embedding[dim] += delta;
            // Clamp to [0, 1]
            self.embedding[dim] = self.embedding[dim].clamp(0.0, 1.0);
        }
        
        // 2. Update metadata aggregates
        self.update_metadata(action, sound);
        
        // 3. Increment counter
        self.generation_count += 1;
        self.last_updated = Utc::now();
    }
    
    /// Get diversity bonus (generate outside comfort zone)
    pub fn exploration_bonus(&self) -> f64 {
        // Users who always pick the same type get an exploration push
        let entropy = self.type_entropy();
        if entropy < 0.5 {
            0.3 // Generate more varied options
        } else {
            0.0
        }
    }
}
```

---

## 3. Recommendation Logic

### How Taste Memory Improves Generation

```
Without Taste Memory:
  User types "kick" → Model generates a generic kick
  Random seed, random quality, random character
  
With Taste Memory:
  User types "kick" → System knows:
    - User prefers: punchy kicks with sub-bass, BPM 140-160, trap genre
    - User exports: kicks with crest factor > 10, centroid ~3kHz
    - User avoids: long tails, bright attack
  
  → Prompt augmented: "punchy trap kick 140bpm with heavy sub, tight"
  → Model parameters: cfg_scale=8.0 (follow prompt closely)
  → Seed: derived from taste fingerprint (consistent character)
  → Post-processing: transient boost +3dB, sub boost +2dB, tail trim
```

### Recommendation Scoring

```rust
pub fn score_generation_for_user(
    generation: &SoundFeatures,
    taste: &TasteEmbedding,
) -> f64 {
    let mut score = 0.0_f64;
    
    // 1. Feature match score (0-50 points)
    let feature_distance = cosine_similarity(
        &generation.feature_vector(),
        &taste.embedding
    );
    score += feature_distance * 50.0;
    
    // 2. Type match score (0-20 points)
    if taste.pref_sound_types.iter()
        .any(|(t, _)| *t == generation.sound_type) 
    {
        let type_weight = taste.pref_sound_types.iter()
            .find(|(t, _)| *t == generation.sound_type)
            .map(|(_, w)| w)
            .unwrap_or(&0.0);
        score += type_weight * 20.0;
    }
    
    // 3. Genre match score (0-15 points)
    if let Some(genre) = &generation.detected_genre {
        if taste.pref_genres.iter().any(|(g, _)| g == genre) {
            score += 10.0;
        }
    }
    
    // 4. BPM match score (0-10 points)
    if let Some(bpm) = generation.bpm {
        let (min_bpm, max_bpm) = taste.pref_bpm_range;
        if bpm >= min_bpm && bpm <= max_bpm {
            score += 10.0;
        } else {
            // Partial credit for close BPMs
            let distance = (bpm as f64 - (min_bpm + max_bpm) as f64 / 2.0).abs();
            score += (10.0 - distance * 0.1).max(0.0);
        }
    }
    
    // 5. Exploration bonus (0-5 points)
    score += taste.exploration_bonus() * 5.0;
    
    score.clamp(0.0, 100.0)
}
```

### Adaptive Generation Parameters

```rust
pub fn adapt_generation_params(
    taste: &TasteEmbedding,
    base_params: GenerationParams,
) -> GenerationParams {
    let mut adapted = base_params.clone();
    
    // Adjust temperature based on exploration rate
    adapted.temperature *= (1.0 + taste.exploration_rate()).clamp(0.8, 1.5);
    
    // Adjust CFG scale based on prompt specificity preference
    if taste.prompt_verbosity() < 0.3 {
        adapted.cfg_scale *= 1.2; // Follow terse prompts more closely
    }
    
    // Adjust duration based on preference
    adapted.duration_ms = taste.pref_duration_ms as u32;
    
    // Adjust post-processing based on taste
    adapted.post_process.punch_boost = taste.pref_punch as f32 * 6.0;
    adapted.post_process.sub_boost = taste.pref_sub_weight as f32 * 4.0;
    adapted.post_process.brightness = taste.pref_brightness as f32 * 3.0;
    
    adapted
}
```

---

## 4. Privacy Model

### Data Collection Philosophy

```
Explicit Consent + Granular Control + Complete Transparency

cShot's privacy model:
  1. ALL taste learning is LOCAL by default (on-device).
  2. Cloud sync is OPT-IN and clearly explained.
  3. Users can view their complete taste profile at any time.
  4. Users can reset or delete their taste profile instantly.
  5. No audio data is ever uploaded for taste learning.
  6. Only aggregate preference vectors leave the device (if opted in).
```

### Privacy Tiers

```
Tier 1 — Local Only (Default)
  ┌─────────────────────────────────────────────┐
  │ Taste memory lives entirely on-device       │
  │ in SQLite. Never uploaded.                  │
  │                                             │
  │ Features:                                   │
  │ • Full taste learning                       │
  │ • Adaptive generation                       │
  │ • Personalized recommendations              │
  │ • Export/favorite-based learning            │
  │ • No account required                       │
  │                                             │
  │ Limitations:                                │
  │ • Lost if app data is cleared               │
  │ • No cross-device sync                      │
  │ • Can't contribute to collective models     │
  └─────────────────────────────────────────────┘

Tier 2 — Cloud Synced (Opt-in)
  ┌─────────────────────────────────────────────┐
  │ Taste embedding is synced to cloud (encrypted)│
  │ for cross-device access.                    │
  │                                             │
  │ Features (all of Tier 1 +):                 │
  │ • Cross-device sync                         │
  │ • Account-based persistence                 │
  │ • Improved cold-start on new devices        │
  │                                             │
  │ Data uploaded:                              │
  │ • Taste embedding vector (768 floats)       │
  │ • Preference metadata (no raw data)         │
  │ • NOT prompts, NOT audio, NOT actions       │
  └─────────────────────────────────────────────┘

Tier 3 — Collective Improvement (Opt-in bonus)
  ┌─────────────────────────────────────────────┐
  │ Anonymized taste data contributes to        │
  │ improving cShot for everyone.               │
  │                                             │
  │ Features (all of Tier 1/2 +):               │
  • Better cold-start for new users             │
  • Trend detection (what producers want now)   │
  • Improved genre/type weighting               │
  │                                             │
  │ Data uploaded:                              │
  │ • Anonymized, aggregated taste patterns     │
  │ • No personally identifiable information    │
  │ • Differential privacy (ε=2.0)              │
  └─────────────────────────────────────────────┘
```

### Data Storage (Local SQLite)

```sql
-- Taste memory table (local only)
CREATE TABLE taste_profile (
    id INTEGER PRIMARY KEY DEFAULT 1,   -- Singleton row
    embedding TEXT NOT NULL,             -- JSON array of 768 floats
    pref_sound_types TEXT,               -- JSON: [{"type":"kick","weight":0.7},...]
    pref_bpm_min INTEGER,
    pref_bpm_max INTEGER,
    pref_brightness REAL,               -- 0-1
    pref_punch REAL,                    -- 0-1
    pref_sub_weight REAL,               -- 0-1
    pref_cleanliness REAL,              -- 0-1
    pref_complexity REAL,               -- 0-1
    pref_genres TEXT,                   -- JSON: [{"genre":"trap","weight":0.6},...]
    pref_duration_ms REAL,
    exploration_rate REAL,              -- 0-1
    generation_count INTEGER DEFAULT 0,
    last_updated TEXT NOT NULL
);

-- Raw signal log (for recomputation, auto-pruned after 30 days)
CREATE TABLE taste_signals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    generation_id TEXT NOT NULL,
    action_type TEXT NOT NULL,           -- 'export', 'favorite', 'delete', etc.
    sound_type TEXT,
    feature_vector TEXT,                 -- JSON array of 256 feature values
    bpm INTEGER,
    genre TEXT,
    weight REAL,                         -- Signal weight (0.0 to 1.0)
    created_at TEXT NOT NULL
);

CREATE INDEX idx_taste_signals_created ON taste_signals(created_at);
```

### Privacy Controls UI

```typescript
interface PrivacySettings {
  tasteLearning: {
    enabled: boolean;             // Master switch (default: true)
    localOnly: boolean;           // Default: true (no cloud sync)
    cloudSync: boolean;           // Opt-in
    collectiveImprovement: boolean; // Opt-in, requires cloudSync
  };
  
  dataManagement: {
    viewProfile: () => TasteProfileSummary;
    resetTaste: () => void;       // Clears all learned data
    deleteAccount: () => void;    // Removes all cloud data
    downloadData: () => void;     // Exports taste profile as JSON
  };
  
  transparency: {
    lastUpdated: string;
    dataStored: string;           // "2.4 KB of preference data"
    signalsCollected: number;     // "142 signals since Feb 14"
    nextSyncDate: string | null;  // Only if cloud sync enabled
  };
}

// Privacy Settings Screen
┌────────────────────────────────────────────────┐
│ Privacy & Data                                  │
│                                                 │
│ Taste Learning                              [ON]│
│   ─────────────────────────────────────────      │
│   cShot learns your sonic preferences from      │
│   your exports, favorites, and listening.       │
│   All data stays on your device by default.     │
│                                                 │
│ Cloud Sync                              [OFF]  │
│   ─────────────────────────────────────────      │
│   Sync your taste profile across devices.       │
│   Only preference vectors, not raw data.        │
│                                                 │
│ Collective Improvement                   [OFF]  │
│   ─────────────────────────────────────────      │
│   Help improve cShot for everyone with          │
│   anonymized, aggregated preference data.       │
│                                                 │
│ ┌─────────────────────────────────────────┐     │
│ │ View Your Taste Profile                 │     │
│ └─────────────────────────────────────────┘     │
│ ┌─────────────────────────────────────────┐     │
│ │ Reset Taste Memory                      │     │
│ └─────────────────────────────────────────┘     │
│ ┌─────────────────────────────────────────┐     │
│ │ Download Your Data                      │     │
│ └─────────────────────────────────────────┘     │
│                                                 │
│ Profile size: 2.4 KB | Updated: 2 hours ago    │
│ Signals collected: 142 | Since: Feb 14, 2025   │
└────────────────────────────────────────────────┘
```

---

## 5. Reset/Forget Controls

### Reset Levels

```
Level 1 — Soft Reset (Clear Signals)
  What happens:
    • Removes raw signal log (taste_signals table)
    • Keeps current embedding
    • Stops new signals from accumulating
  When to use: "I want a fresh start but keep my current preferences for now"
  Effort: Instant

Level 2 — Full Taste Reset
  What happens:
    • Clears entire taste_profile table
    • Clears all taste_signals
    • Recomputes embedding from scratch (starts neutral)
    • Does NOT delete favorites or generation history
  When to use: "My taste has completely changed"
  Effort: Instant

Level 3 — Account Deletion
  What happens:
    • Deletes all cloud data
    • Leaves local taste data intact (user must reset locally too)
    • Cancels subscription if applicable
  When to use: "I want to leave cShot entirely"
  Effort: 24h processing window

Level 4 — Data Export (not a reset, but a companion)
  What happens:
    • Exports taste profile as human-readable JSON
    • Includes embedding, preferences, genre weights
    • Includes generation count and learning history
  When to use: "I want to see what cShot knows about me"
```

### Implementation

```rust
pub struct TastePrivacyManager {
    db: Pool<SqliteConnection>,
}

impl TastePrivacyManager {
    pub async fn soft_reset(&self, user_id: &str) -> Result<()> {
        // Clear raw signals, keep embedding
        sqlx::query("DELETE FROM taste_signals WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
    
    pub async fn full_reset(&self, user_id: &str) -> Result<()> {
        // Clear everything taste-related
        sqlx::query("DELETE FROM taste_signals WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.db)
            .await?;
        sqlx::query("DELETE FROM taste_profile WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.db)
            .await?;
        // Reinitialize with default embedding
        self.initialize_taste(user_id).await?;
        Ok(())
    }
    
    pub async fn export_profile(&self, user_id: &str) -> Result<TasteProfileExport> {
        let profile = sqlx::query_as::<_, TasteProfile>("SELECT * FROM taste_profile WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&self.db)
            .await?;
        
        match profile {
            Some(p) => Ok(TasteProfileExport {
                exported_at: Utc::now(),
                embedding: p.embedding,
                preferences: TastePreferences {
                    sound_types: p.pref_sound_types,
                    bpm_range: (p.pref_bpm_min, p.pref_bpm_max),
                    brightness: p.pref_brightness,
                    punch: p.pref_punch,
                    sub_weight: p.pref_sub_weight,
                    cleanliness: p.pref_cleanliness,
                    complexity: p.pref_complexity,
                    genres: p.pref_genres,
                    duration_ms: p.pref_duration_ms,
                },
                stats: TasteStats {
                    generation_count: p.generation_count,
                    exploration_rate: p.exploration_rate,
                    last_updated: p.last_updated,
                },
            }),
            None => Err(Error::NoTasteProfile),
        }
    }
}
```

---

## 6. Cold-Start Strategy

### Problem

New users have zero taste data. The first 10-20 generations have no personalization. Without a cold-start strategy, taste memory is invisible until it matters — but it needs to show value immediately.

### Cold-Start Approaches

```
Approach 1 — Genre Quiz (Optional, 1 tap)
  "What kind of music do you make?"
  [Trap/Hip-Hop] [EDM] [Lo-fi] [Rock] [Cinematic] [Experimental]
  
  Sets initial taste embedding from genre archetype.
  ~60% accuracy on day 1, improves with real signals.

Approach 2 — First Session Implicit Learning
  No quiz, no questions. The system learns aggressively from the first session:
    • After 3 generations → initial type preference (kicks vs snares)
    • After 1 export → strong signal (this is what they want)
    • After 5 exports → first taste profile solid enough to adapt generation
    • After 20 exports → full taste profile
    
  Pros: Zero friction, learns naturally
  Cons: First 5 generations are generic

Approach 3 — Collective Cold-Start
  Use anonymized data from similar users to bootstrap:
    • New user exports a kick → find users who also export kicks
    • Borrow aggregate preferences from similar cohort
    • As user generates more, personal signal replaces cohort signal
    
  Requires: Collective Improvement opt-in from existing users
  Pros: Best cold-start quality
  Cons: Privacy complexity, requires user base

Approach 4 — Prompt-Based Inference
  Analyze the user's first prompt to infer preferences:
    "punchy trap kick 140bpm" → BPM 140, genre trap, punchy
    "dark ambient texture" → experimental, dark, texture-focused
    "kick" → generic, unknown preferences
    
  Pros: Zero friction, instant
  Cons: Only works for specific prompts
```

### Recommended Cold-Start Strategy

```
Phase 1 (Beta launch): Approach 2 + Approach 4
  • No quiz (zero friction)
  • Implicit learning from first session
  • Prompt-based inference for immediate adaptation
  • After 5 generations, show "cShot is learning your taste" notification
  • After 10 exports, show first "Recommended for you" generation

Phase 2 (Post-beta): Add Approach 1 as optional
  • Show genre selection on first launch (dismissable)
  • "This helps cShot learn your sound faster"
  • Skip button available

Phase 3 (Scale): Add Approach 3 if user base is large enough
  • Only with explicit opt-in
  • Clear explanation of how data is used
  • Differential privacy guarantees
```

### Cold-Start UI

```
First Session Experience:

┌─────────────────────────────────────────────────────────┐
│  [prompt input]                                         │
│                                                          │
│  ┌─ Quick Start ──────────────────────────────────┐     │
│  │   👋 Welcome! Try typing a sound description.  │     │
│  │   cShot will learn your preferences as you go. │     │
│  │                                                  │     │
│  │   Try these:                                     │     │
│  │   [punchy trap kick] [tight snare] [deep 808]   │     │
│  │   [bright hat] [dark ambient texture]           │     │
│  └──────────────────────────────────────────────────┘     │
│                                                          │
│  [generation area — empty state]                         │
│                                                          │
│  "Generate your first sound to get started"              │
└─────────────────────────────────────────────────────────┘

After 5 Generations:

┌─────────────────────────────────────────────────────────┐
│  [prompt input with chips]                                │
│                                                          │
│  🔔 cShot is learning your style!                       │
│  Your preferences will improve with every generation.    │
│  [View Taste Profile] [Dismiss]                          │
│                                                          │
│  [generation grid with sounds]                           │
│                                                          │
│  Recent: Kick x3, Snare x1, Hi-hat x1                   │
│  Detected preference: Punchy kicks, BPM 130-150         │
└─────────────────────────────────────────────────────────┘

After 10 Exports:

┌─────────────────────────────────────────────────────────┐
│  [prompt input]                                          │
│                                                          │
│  🔔 cShot knows your taste!                             │
│  Future generations will be adapted to your style.       │
│  ● You prefer: Punchy kicks with heavy sub              │
│  ● Your BPM range: 130-150                              │
│  ● Your genre: Trap/Hip-Hop                             │
│  [View Full Profile] [Dismiss]                          │
│                                                          │
│  ⭐ Recommended for you:                                │
│  [punchy kick with sub] → will generate in your style   │
│                                                          │
│  [generation grid]                                       │
└─────────────────────────────────────────────────────────┘
```

---

## 7. Taste Profile Visualization (User-Facing)

Users should be able to see what cShot knows about them. Not just for transparency — it's also delightful.

```typescript
interface TasteProfileView {
  summary: {
    soundTypes: { label: string; percentage: number }[];
    bpmRange: { min: number; max: number };
    topGenres: { label: string; match: number }[];
    mood: string; // "Punchy & Bright" or "Dark & Textured"
  };
  
  radar: {
    axes: { label: string; value: number }[];
    // axes: Punch, Brightness, Sub-Weight, Cleanliness, Complexity
  };
  
  timeline: {
    // How taste has evolved over time
    snapshots: { date: string; dominantType: string; entropy: number }[];
  };
}
```

```
Taste Profile Screen:

┌─────────────────────────────────────────────────────────┐
│  Your Sonic Profile                                      │
│                                                          │
│  ┌─ Radar Chart ─────────────────────────────────┐      │
│  │                                                 │      │
│  │            Punch                               │      │
│  │          80%                                   │      │
│  │       ●                                        │      │
│  │  Comp ◄───●───► Brightness                     │      │
│  │  20%     ●   75%                               │      │
│  │       ●                                        │      │
│  │  Clean ◄───●───► Sub-Weight                     │      │
│  │  90%     ●   80%                               │      │
│  │           ●                                    │      │
│  │          Simp                                   │      │
│  │          35%                                   │      │
│  └────────────────────────────────────────────────┘      │
│                                                          │
│  Your Top Sounds:                                        │
│  Kick · 45% of exports  ◼◼◼◼◼◼◼◼◼◼                      │
│  Snare · 18%            ◼◼◼◼                             │
│  Bass · 15%             ◼◼◼                              │
│  Hi-hat · 12%           ◼◼                               │
│                                                          │
│  Your BPM Range: 130-150                                 │
│                                                          │
│  Your Genres: Trap (60%), Hip-Hop (25%), Lo-fi (15%)    │
│                                                          │
│  Taste evolved:                                          │
│  ─────────────────────────────────────────►              │
│  Feb  ████████ Kick-heavy                               │
│  Mar  ██████████ More snares                             │
│  Apr  ████████████ Balanced, higher BPM                  │
│                                                          │
│  Profile built from 142 signals over 3 months           │
│  [Reset Taste] [Download Data]                           │
└─────────────────────────────────────────────────────────┘
```

---

## 8. Taste Memory System Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     User Action                           │
│  (export / favorite / delete / skip / rate / prompt)     │
└────────────────────────┬─────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│               Signal Processor                            │
│                                                           │
│  1. Classify action type → weight                         │
│  2. Extract sound features (256-d vector)                 │
│  3. Compute signal delta                                  │
│  4. Store in taste_signals table                          │
└────────────────────────┬─────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│               Embedding Updater                           │
│                                                           │
│  Runs: After each session (batch update)                  │
│                                                           │
│  1. Load current embedding from taste_profile             │
│  2. Weighted average of session signals                  │
│  3. Update metadata aggregates                           │
│  4. Save to taste_profile                                │
│  5. If cloud sync: upload embedding (encrypted)          │
└────────────────────────┬─────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│               Generation Adapter                          │
│                                                           │
│  Called: Before each generation                           │
│                                                           │
│  1. Load current taste embedding                          │
│  2. Adapt generation parameters                           │
│     - Prompt augmentation                                 │
│     - Model params (cfg_scale, temperature)              │
│     - Post-processing params                             │
│  3. Score candidate sounds (if multiple)                 │
│  4. Return adapted GenerationRequest                     │
└──────────────────────────────────────────────────────────┘
```

---

## 9. Summary

```
Taste Memory System — Key Design Decisions:

  1. Local-first, cloud-optional: All learning happens on-device.
     Cloud sync is opt-in and limited to preference vectors.

  2. Implicit signals are primary: Export is the strongest signal.
     Ratings are secondary. We don't need users to fill out forms.

  3. Cold-start is frictionless: No quiz at first launch.
     The system learns from prompt patterns + first few generations.

  4. Full transparency: Users can view, export, and reset their
     taste profile at any time. No black box.

  5. Taste improves generation measurably: After 10+ exports,
     adapted generations should score 20% higher on SoundScore
     than non-adapted ones.

  6. Privacy is a feature: Taste memory that stays on-device is
     a selling point, not a limitation.
```
