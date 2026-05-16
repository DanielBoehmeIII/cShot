# Prompt 55 — Generate Cohesive Sample Packs

A system that turns generated one-shots into cohesive, professional sample packs. One prompt, one aesthetic, 10+ usable sounds.

---

## 1. What Makes a Great Sample Pack

| Quality | Definition | Why It Matters |
|---------|-----------|----------------|
| Cohesive aesthetic | All sounds feel like they belong to the same palette | Producers grab the whole pack, not just one sound |
| Useful variation | Different enough to be distinct, similar enough to fit together | Covers more use cases without feeling random |
| Clear categories | Kicks here, snares there, hats grouped | Fast workflow — producer knows where to look |
| Normalized loudness | Every sound sits at the same perceived volume | No surprise gain-staging when loading into DAW |
| Consistent naming | `TK_Punchy_01.wav`, `TK_Punchy_02.wav` | Professional presentation, easy to organize |
| Metadata | Key, BPM, type, description embedded | DAW integration, Splice-style browsing |
| Cover concept | Visual identity for the pack | Marketability, brand recognition |
| Export structure | `PackName/Type/Sound.wav` | Drag-and-drop ready for any DAW |

---

## 2. Pack Generation Workflow

```
User Types Prompt:
  "trap drum kit, punchy and dark"
  
  OR selects:
  Pack Genre: [Trap ▼]
  Pack Mood: [Dark ▼]  
  Pack Size: [12 sounds ▼]
  Include: [✓ Kicks] [✓ Snares] [✓ Hats] [☐ Perc] [☐ FX]
    │
    ▼
┌─────────────────────────────────────────────┐
│ 1. Parse Pack Intent                         │
│    • Extract genre, mood, sound types        │
│    • Generate base embeddings for cohesion    │
├─────────────────────────────────────────────┤
│ 2. Generate Sound Blueprints                  │
│    • For each sound type in pack:             │
│      - Define variation axes (pitch, length)  │
│      - Generate diverse prompt variations     │
├─────────────────────────────────────────────┤
│ 3. Batch Generate (N parallel requests)       │
│    • Each sound has unique seed + variation   │
│    • All use the same base embedding          │
├─────────────────────────────────────────────┤
│ 4. Repair Chain (batch)                       │
│    • Each sound through repair pipeline       │
│    • Type-specific presets applied            │
├─────────────────────────────────────────────┤
│ 5. Quality Filter                             │
│    • Remove failures, near-duplicates         │
│    • Reject below SoundScore threshold        │
├─────────────────────────────────────────────┤
│ 6. Cluster & Categorize                       │
│    • Group by sound type                      │
│    • Order by similarity within type          │
├─────────────────────────────────────────────┤
│ 7. Normalize Pack                             │
│    • Match loudness across all sounds         │
│    • Match spectral balance across types      │
├─────────────────────────────────────────────┤
│ 8. Name & Export                              │
│    • Generate pack name + sound names         │
│    • Write WAV files + metadata + cover       │
│    • Create folder structure                  │
└─────────────────────────────────────────────┘
    │
    ▼
Exported Pack:
  ~/Desktop/Trap_Dark_Kit_001/
  ├── 00_Pack_Info.txt
  ├── Cover.png (optional)
  ├── Kicks/
  │   ├── TK_Punchy_Kick_01.wav
  │   ├── TK_Punchy_Kick_02.wav
  │   └── TK_Punchy_Kick_03.wav
  ├── Snares/
  │   ├── TK_Tight_Snare_01.wav
  │   ├── TK_Tight_Snare_02.wav
  │   └── TK_Tight_Snare_03.wav
  ├── Hats/
  │   ├── TK_Dark_Hat_01.wav
  │   ├── TK_Dark_Hat_02.wav
  │   └── TK_Dark_Hat_03.wav
  └── FX/
      ├── TK_Riser_01.wav
      └── TK_Impact_01.wav
```

---

## 3. Clustering Method

### Embedding-Based Clustering

```rust
pub struct PackClusterer {
    pub n_clusters: usize,
    pub min_cluster_size: usize,
    pub max_cluster_size: usize,
}

impl PackClusterer {
    pub fn cluster_sounds(
        &self,
        sounds: &[GeneratedSound],
    ) -> Result<PackCluster, PackError> {
        // 1. Compute embeddings for each sound
        let embeddings: Vec<Vec<f32>> = sounds.iter()
            .map(|s| compute_sound_embedding(&s.features))
            .collect();
        
        // 2. Hierarchical clustering (agglomerative)
        //    Linkage: Ward's method (minimizes within-cluster variance)
        //    Distance: cosine distance on embeddings
        let clusters = hierarchical_clustering(&embeddings, self.n_clusters);
        
        // 3. Assign cluster labels to sounds
        let mut categorized: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, cluster_id) in clusters.iter().enumerate() {
            let type_name = sounds[i].features.sound_type.clone();
            categorized.entry(type_name).or_default().push(i);
        }
        
        // 4. Within each type cluster, order by similarity
        for indices in categorized.values_mut() {
            indices.sort_by(|&a, &b| {
                let sim_a = centroid_similarity(&embeddings[a], &embeddings);
                let sim_b = centroid_similarity(&embeddings[b], &embeddings);
                sim_b.partial_cmp(&sim_a).unwrap() // Most representative first
            });
        }
        
        Ok(PackCluster { clusters: categorized })
    }
}

fn compute_sound_embedding(features: &SignalFeatures) -> Vec<f32> {
    // Embedding captures: timbre, dynamics, spectral shape
    vec![
        features.spectral_centroid_hz / 10000.0,
        features.spectral_flatness,
        features.spectral_bandwidth_hz / 10000.0,
        features.zero_crossing_rate,
        features.crest_factor / 20.0,
        features.attack_time_ms / 100.0,
        features.decay_time_ms / 500.0,
        features.rms,
        features.harmonic_ratio,
        features.transient_ratio,
        // Mel spectrum (reduced via PCA to 8 dims)
        // Already normalized in SignalFeatures
    ]
}
```

### Diversity Control

```rust
pub fn ensure_diversity(
    selected: &[usize],
    all_embeddings: &[Vec<f32>],
    min_distance: f32,
) -> Vec<usize> {
    // Greedy diversity selection:
    // 1. Pick the most central sound first (best representative)
    // 2. For each subsequent pick, choose the sound farthest
    //    from all previously picked sounds
    // 3. Stop when max_cluster_size reached
    
    let mut diverse = Vec::new();
    let mut available: Vec<usize> = (0..all_embeddings.len()).collect();
    
    while diverse.len() < selected.len() {
        if diverse.is_empty() {
            // Pick centroid
            diverse.push(centroid_index(all_embeddings));
        } else {
            // Pick farthest from selected
            let next = available.iter()
                .max_by(|&&a, &&b| {
                    let min_dist_a = diverse.iter()
                        .map(|&d| cosine_distance(&all_embeddings[a], &all_embeddings[d]))
                        .fold(f32::MAX, f32::min);
                    let min_dist_b = diverse.iter()
                        .map(|&d| cosine_distance(&all_embeddings[b], &all_embeddings[d]))
                        .fold(f32::MAX, f32::min);
                    min_dist_a.partial_cmp(&min_dist_b).unwrap()
                })
                .copied().unwrap();
            diverse.push(next);
        }
        
        available.retain(|&i| i != *diverse.last().unwrap());
        
        // Stop if remaining sounds are too similar
        if available.iter().all(|&i| {
            diverse.iter()
                .all(|&d| cosine_distance(&all_embeddings[i], &all_embeddings[d]) < min_distance)
        }) {
            break;
        }
    }
    
    diverse
}
```

---

## 4. Genre/Theme Controls

```rust
pub struct PackTemplate {
    pub name: &'static str,
    pub genres: &'static [&'static str],
    pub sound_types: &'static [&'static str],
    pub min_per_type: usize,
    pub max_per_type: usize,
    pub default_size: usize,
    pub mood_presets: &'static [&'static str],
}

pub const PACK_TEMPLATES: &[PackTemplate] = &[
    PackTemplate {
        name: "Drum Kit",
        genres: &["trap", "hip-hop", "rnb", "pop"],
        sound_types: &["kick", "snare", "hihat", "clap", "perc"],
        min_per_type: 3,
        max_per_type: 6,
        default_size: 16,
        mood_presets: &["punchy", "dark", "bright", "warm", "aggressive"],
    },
    PackTemplate {
        name: "Electronic Percussion",
        genres: &["house", "techno", "electronic", "dnb"],
        sound_types: &["kick", "clap", "hihat", "perc", "fx"],
        min_per_type: 2,
        max_per_type: 5,
        default_size: 12,
        mood_presets: &["driving", "deep", "sharp", "layered"],
    },
    PackTemplate {
        name: "Cinematic FX",
        genres: &["cinematic", "ambient", "orchestral"],
        sound_types: &["fx", "perc", "bass"],
        min_per_type: 2,
        max_per_type: 8,
        default_size: 10,
        mood_presets: &["epic", "dark", "ethereal", "tense", "dramatic"],
    },
    PackTemplate {
        name: "Bass Pack",
        genres: &["trap", "dnb", "dubstep", "techno"],
        sound_types: &["bass", "kick", "fx"],
        min_per_type: 3,
        max_per_type: 8,
        default_size: 12,
        mood_presets: &["deep", "aggressive", "subby", "distorted", "clean"],
    },
    PackTemplate {
        name: "Organic Percussion",
        genres: &["lo-fi", "jazz", "folk", "indie"],
        sound_types: &["kick", "snare", "hihat", "perc", "clap"],
        min_per_type: 2,
        max_per_type: 4,
        default_size: 12,
        mood_presets: &["warm", "soft", "vintage", "natural", "roomy"],
    },
];

/// Prompt templates per sound type, ensuring diversity
pub fn generate_sound_prompts(
    base_prompt: &str,
    sound_type: &str,
    variation_count: usize,
) -> Vec<String> {
    // Variation axes:
    // 1. Intensity: soft ↔ aggressive
    // 2. Pitch: low ↔ high (within type range)
    // 3. Length: short ↔ long (within type range)
    // 4. Texture: clean ↔ processed
    
    let modifiers = match sound_type {
        "kick" => vec![
            "punchy", "deep", "tight", "subby", "clicky",
            "boomy", "dry", "layered", "processed", "natural",
        ],
        "snare" => vec![
            "crack", "tight", "fat", "snappy", "dry",
            "reverberant", "layered", "bright", "dark", "wooden",
        ],
        "hihat" => vec![
            "tight", "sizzly", "dark", "bright", "crisp",
            "washy", "short", "metallic", "closed", "open",
        ],
        "clap" => vec![
            "tight", "wide", "deep", "snappy", "layered",
            "dry", "reverberant", "bright", "soft", "processed",
        ],
        "perc" => vec![
            "wooden", "metallic", "clicky", "deep", "ringing",
            "short", "tonal", "noisy", "shaker", "tambourine",
        ],
        "bass" => vec![
            "subby", "deep", "distorted", "clean", "punchy",
            "long", "short", "modulated", "sine", "sawtooth",
        ],
        "fx" => vec![
            "riser", "impact", "sweep", "glitch", "reverse",
            "ambient", "granular", "stutter", "sub hit", "noise",
        ],
        _ => vec!["default"],
    };
    
    // Generate variations by cycling through modifiers
    let mut prompts = Vec::new();
    for i in 0..variation_count {
        let mod_idx = i % modifiers.len();
        let prompt = format!(
            "{} {} {}, {} kit",
            base_prompt, modifiers[mod_idx], sound_type, base_prompt
        );
        prompts.push(prompt);
    }
    
    // Ensure each has a different random seed
    prompts
}
```

---

## 5. Automatic Naming System

```rust
pub fn generate_pack_name(genre: &str, mood: &str, seed: u64) -> String {
    let prefixes = [
        "Project", "Black", "Silver", "Dark", "Neon", "Shadow",
        "Phantom", "Cyber", "Analog", "Digital", "Vapor", "Crystal",
        "Noir", "Crimson", "Abyss", "Solar", "Lunar", "Echo",
        "Pulse", "Vertex", "Drift", "Flux", "Core", "Prism",
    ];
    
    let suffixes = [
        "Kit", "Pack", "Collection", "Drums", "Essentials", "Elements",
        "Series", "Volume", "Tones", "Sounds", "Lab", "Studio",
        "Beats", "Percussion", "Reserve", "Archive",
    ];
    
    let prefix = prefixes[seed as usize % prefixes.len()];
    let suffix = suffixes[(seed / prefixes.len() as u64) as usize % suffixes.len()];
    
    format!("{} {} {}", prefix, capitalize(mood), suffix)
}

pub fn generate_sound_filename(
    pack_prefix: &str,
    sound_type: &str,
    index: usize,
    features: &SignalFeatures,
) -> String {
    let type_abbr = match sound_type {
        "kick" => "KK",
        "snare" => "SN",
        "hihat" => "HH",
        "clap" => "CP",
        "perc" => "PC",
        "bass" => "BS",
        "fx" => "FX",
        _ => "OT",
    };
    
    let descriptor = if features.crest_factor > 12.0 { "Punchy" }
    else if features.crest_factor < 7.0 { "Soft" }
    else { "Tight" };
    
    let brightness = if features.spectral_centroid_hz > 4000.0 { "Bright" }
    else if features.spectral_centroid_hz < 1500.0 { "Dark" }
    else { "Warm" };
    
    format!(
        "{}_{}_{}_{}_{:02}.wav",
        pack_prefix, type_abbr, descriptor, brightness, index
    )
}
```

### Naming Conventions

```
Standard naming:  {PackPrefix}_{Type}_{Descriptor}_{Number}.wav
  → TK_KK_Punchy_01.wav
  → TK_SN_Crack_02.wav
  → TK_HH_Dark_03.wav

Preview naming:   {PackPrefix}_{Type}_{Number}.wav (same descriptor omitted)
  → TK_KK_01.wav, TK_KK_02.wav

Splice-style:     {PackPrefix} {Descriptor} {Type} {Number}.wav
  → "TK Punchy Kick 01.wav"
  → "TK Crack Snare 02.wav"
```

---

## 6. Quality Filter

```rust
pub struct PackQualityFilter {
    pub min_sound_score: f32,          // 0.5 — reject poor sounds
    pub max_similarity_within_pack: f32, // 0.85 — reject near-duplicates
    pub max_similarity_to_existing: f32, // 0.9 — reject copies of user's library
    pub min_duration_ms: f32,          // 30ms
    pub max_duration_ms: f32,          // 5000ms
}

impl PackQualityFilter {
    pub fn filter(
        &self,
        sounds: &mut Vec<GeneratedSound>,
        existing_library: &[Vec<f32>],
    ) -> Vec<GeneratedSound> {
        // 1. Remove below minimum score
        sounds.retain(|s| s.score >= self.min_sound_score);
        
        // 2. Remove near-duplicates within pack
        let mut keep = Vec::new();
        let mut kept_embeddings: Vec<Vec<f32>> = Vec::new();
        
        for sound in sounds.iter() {
            let emb = compute_sound_embedding(&sound.features);
            let is_duplicate = kept_embeddings.iter()
                .any(|k| cosine_similarity(&emb, k) > self.max_similarity_within_pack);
            
            if !is_duplicate {
                keep.push(sound.clone());
                kept_embeddings.push(emb);
            }
        }
        
        // 3. Remove duplicates of user's existing library
        keep.retain(|s| {
            let emb = compute_sound_embedding(&s.features);
            !existing_library.iter()
                .any(|lib_emb| cosine_similarity(&emb, lib_emb) > self.max_similarity_to_existing)
        });
        
        // 4. Remove out-of-range durations
        keep.retain(|s| {
            s.features.duration_ms >= self.min_duration_ms
            && s.features.duration_ms <= self.max_duration_ms
        });
        
        keep
    }
}
```

---

## 7. Export Format

### Folder Structure

```
{Trap_Dark_Kit_001}/
├── 00_Pack_Info.txt
├── 00_License.txt
├── Cover.png (optional, 1024×1024)
│
├── Kicks/
│   ├── TK_KK_Punchy_01.wav
│   ├── TK_KK_Punchy_02.wav
│   ├── TK_KK_Deep_03.wav
│   └── TK_KK_Deep_04.wav
│
├── Snares/
│   ├── TK_SN_Crack_01.wav
│   ├── TK_SN_Crack_02.wav
│   ├── TK_SN_Fat_03.wav
│   └── TK_SN_Fat_04.wav
│
├── Hats/
│   ├── TK_HH_Tight_01.wav
│   ├── TK_HH_Tight_02.wav
│   ├── TK_HH_Open_03.wav
│   └── TK_HH_Open_04.wav
│
├── Perc/
│   ├── TK_PC_Clack_01.wav
│   └── TK_PC_Clack_02.wav
│
└── FX/
    ├── TK_FX_Riser_01.wav
    └── TK_FX_Impact_02.wav
```

### Pack_Info.txt

```
cShot Sample Pack
─────────────────
Title:      Trap Dark Kit 001
Genre:      Trap
Mood:       Dark
Created:    2025-02-15
Sounds:     14
Size:       28.4 MB

Contents:
  Kicks:  4  (punchy, deep, subby)
  Snares: 4  (crack, fat, tight)
  Hats:   4  (tight closed, open wash)
  Perc:   2  (clack, shaker)
  FX:     2  (riser, impact)

Generated with cShot AI v0.2.0
Model: cShot-base-v1

License: Free for commercial use.
         These sounds may be used in any commercial or non-commercial project.
         These sounds may not be re-sold or redistributed as a sample pack.
         These sounds may not be used to train AI models.

Sound List:
  TK_KK_Punchy_01.wav     Kick       0.42s   -1.0dBFS  C
  TK_KK_Punchy_02.wav     Kick       0.38s   -0.8dBFS  C#
  TK_KK_Deep_03.wav       Kick       0.55s   -1.2dBFS  G
  TK_KK_Deep_04.wav       Kick       0.61s   -1.0dBFS  F
  TK_SN_Crack_01.wav      Snare      0.28s   -0.5dBFS  ---
  TK_SN_Crack_02.wav      Snare      0.31s   -0.7dBFS  ---
  TK_SN_Fat_03.wav        Snare      0.35s   -1.0dBFS  ---
  TK_SN_Fat_04.wav        Snare      0.29s   -0.9dBFS  ---
  TK_HH_Tight_01.wav      Hi-hat     0.12s   -1.5dBFS  ---
  TK_HH_Tight_02.wav      Hi-hat     0.15s   -1.3dBFS  ---
  TK_HH_Open_03.wav       Open Hat   0.42s   -0.8dBFS  ---
  TK_HH_Open_04.wav       Open Hat   0.38s   -1.0dBFS  ---
  TK_PC_Clack_01.wav      Perc       0.18s   -2.0dBFS  ---
  TK_FX_Riser_01.wav      FX         2.10s   -0.5dBFS  D
  TK_FX_Impact_02.wav     FX         1.80s   -0.3dBFS  F#
```

### WAV Metadata (iXML chunk)

```rust
pub fn write_pack_wav(
    path: &Path,
    audio: &[f32],
    sample_rate: u32,
    metadata: &SoundMetadata,
) -> Result<(), PackError> {
    // 1. Write WAV file with standard header (44 bytes)
    // 2. Append iXML chunk with metadata:
    //    <BPM>140</BPM>
    //    <KEY>Cm</KEY>
    //    <TYPE>Kick</TYPE>
    //    <DESCRIPTION>Punchy trap kick, dark, subby</DESCRIPTION>
    //    <PACK>Trap Dark Kit 001</PACK>
    //    <GENERATOR>cShot v0.2.0</GENERATOR>
    // 3. Append acidization chunk (for Ableton/FL Studio compatibility)
    //    - OneShot=1
    //    - Root note (if pitched)
    //    - Beats (if tempo-locked)
}
```

---

## 8. Pack Generation UX

### Main Interface

```
┌──────────────────────────────────────────────────────┐
│  PACK BUILDER                                        │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │  Describe your pack idea...                   │   │
│  │  e.g., "trap drum kit, dark and punchy"       │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  Genre: [Trap ▼]  Mood: [Dark ▼]  Size: [16 ▼]     │
│                                                      │
│  Sound Types:                                        │
│  [✓] Kicks (4)  [✓] Snares (4)  [✓] Hats (4)       │
│  [☐] Perc (2)   [☐] Claps (2)   [☐] FX (2)         │
│                                                      │
│  [⚡ Generate Pack]                                   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │  Progress: ████████████░░░░░░ 75%              │   │
│  │  "Generating kicks 3/4..."                     │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  Preview Generated Sounds:                           │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐        │
│  │KK01│ │KK02│ │KK03│ │SN01│ │SN02│ │HH01│ ...     │
│  │▶   │ │▶   │ │▶   │ │▶   │ │▶   │ │▶   │        │
│  └────┘ └────┘ └────┘ └────┘ └────┘ └────┘        │
│                                                      │
│  [Regenerate Selected]  [Preview All]                │
│  [Export Pack ⬇]                                     │
└──────────────────────────────────────────────────────┘
```

### Regeneration Controls

```
Individual sound regeneration:
  Click a sound → detail panel opens:
    ┌──────────────────────────────┐
    │ TK_KK_Punchy_01.wav          │
    │ ━━━━━━━━━━━━━━━━━━━━         │
    │ 0.42s · Kick · C             │
    │                              │
    │ SoundScore: 84% ★★★★        │
    │                              │
    │ [▶ Play] [↻ Regenerate]     │
    │ [✕ Remove from Pack]        │
    │                              │
    │ Prompt: "trap kick punchy,   │
    │          dark, subby"        │
    │ [Edit Prompt ↴]              │
    └──────────────────────────────┘

Pack-wide controls:
  [↻ Regenerate All Kicks] — Replace all kicks with new variations
  [↻ Regenerate Weak Sounds] — Only sounds below 60% score
  [🔀 Shuffle Variations] — Reorder within each category
  [+] Add More Sounds — "Add 2 more snares"
```

---

## 9. Pack Scoring

```rust
pub struct PackScore {
    pub overall: f32,           // 0.0-1.0
    pub cohesion: f32,          // How well sounds fit together
    pub diversity: f32,         // Variation within types
    pub coverage: f32,          // Completeness of sound types
    pub quality: f32,           // Average SoundScore
    pub loudness_consistency: f32,
}

pub fn score_pack(sounds: &[GeneratedSound]) -> PackScore {
    let embeddings: Vec<Vec<f32>> = sounds.iter()
        .map(|s| compute_sound_embedding(&s.features))
        .collect();
    
    // Cohesion: average pairwise similarity
    // Target: 0.5-0.7 (not too samey, not too random)
    let mut similarities = Vec::new();
    for i in 0..embeddings.len() {
        for j in (i+1)..embeddings.len() {
            similarities.push(cosine_similarity(&embeddings[i], &embeddings[j]));
        }
    }
    let mean_sim = similarities.iter().sum::<f32>() / similarities.len() as f32;
    let cohesion = if mean_sim > 0.4 && mean_sim < 0.8 { 1.0 }
                  else if mean_sim > 0.2 { 0.6 + (mean_sim - 0.2) / 0.2 * 0.4 }
                  else { mean_sim / 0.2 * 0.5 };
    
    // Diversity: within-type variation
    let mut diversity_scores = Vec::new();
    for (_type_name, group) in group_by_type(sounds) {
        let group_embs: Vec<Vec<f32>> = group.iter()
            .map(|s| compute_sound_embedding(&s.features))
            .collect();
        let mut group_dists = Vec::new();
        for i in 0..group_embs.len() {
            for j in (i+1)..group_embs.len() {
                group_dists.push(cosine_distance(&group_embs[i], &group_embs[j]));
            }
        }
        let mean_dist = group_dists.iter().sum::<f32>() / group_dists.len() as f32;
        // Ideal distance: 0.2-0.5 (different but related)
        diversity_scores.push(if mean_dist > 0.15 && mean_dist < 0.6 { 1.0 }
                             else { 0.5 });
    }
    let diversity = diversity_scores.iter().sum::<f32>() / diversity_scores.len() as f32;
    
    // Coverage: does the pack have the expected sound types?
    let coverage = coverage_score(sounds);
    
    // Quality: average SoundScore across all sounds
    let quality = sounds.iter().map(|s| s.score).sum::<f32>() / sounds.len() as f32;
    
    // Loudness consistency: variance of RMS across sounds
    let rms_values: Vec<f32> = sounds.iter().map(|s| s.features.rms).collect();
    let mean_rms = rms_values.iter().sum::<f32>() / rms_values.len() as f32;
    let variance = rms_values.iter()
        .map(|r| (r - mean_rms).powi(2))
        .sum::<f32>() / rms_values.len() as f32;
    let loudness_consistency = (1.0 - (variance * 10.0).min(1.0)).max(0.0);
    
    let overall = cohesion * 0.25 + diversity * 0.20 + coverage * 0.20
                + quality * 0.20 + loudness_consistency * 0.15;
    
    PackScore {
        overall: overall.clamp(0.0, 1.0),
        cohesion,
        diversity,
        coverage,
        quality,
        loudness_consistency,
    }
}
```

---

## 10. Summary

The pack builder turns one-shot generation into a batch workflow. One prompt produces 10-20 cohesive sounds organized by type, normalized in loudness, and professionally named and exported. The clustering algorithm ensures variation within each category. The quality filter removes failures and duplicates. The export structure is ready for DAW import. Users can regenerate individual sounds, adjust the pack composition, and export with full metadata.
