# Prompt 56 — Design Sonic Cohesion Metrics

How should cShot know whether a group of sounds belongs together? Metrics that measure sonic cohesion across 10 dimensions.

---

## 1. Cohesion Dimensions

| Dimension | Definition | Why It Matters for Packs |
|-----------|-----------|-------------------------|
| Timbral cohesion | Similarity of tonal character across sounds | The pack sounds like one palette |
| Genre cohesion | All sounds fit the same genre | Credibility — users buy packs for a genre |
| Emotional cohesion | Consistent mood across sounds | Producers pick packs for a vibe |
| Loudness consistency | Same perceived volume across sounds | No gain-staging surprises in DAW |
| Spectral balance | Consistent frequency profile | The pack doesn't have one piercing sound |
| Transient compatibility | Similar attack character | Drummers/beatmakers feel consistency |
| Stereo consistency | Same stereo width approach | Mix coherence |
| Production style similarity | Same level of processing/polish | Professional presentation |
| Harmonic compatibility | Works together in the same key | Musical cohesion |
| Temporal consistency | Similar duration/length patterns | Workflow predictability |

---

## 2. Metric Definitions

### 2.1 Timbral Cohesion

```rust
pub fn timbral_cohesion(sounds: &[GeneratedSound]) -> CohesionScore {
    // Compare spectral profiles across sounds
    // High scores = similar timbre palette
    
    let embeddings: Vec<Vec<f32>> = sounds.iter()
        .map(|s| timbre_embedding(&s.features))
        .collect();
    
    // Pairwise cosine similarity
    let mut similarities = Vec::new();
    for i in 0..embeddings.len() {
        for j in (i+1)..embeddings.len() {
            similarities.push(cosine_similarity(&embeddings[i], &embeddings[j]));
        }
    }
    
    let mean_sim = mean(&similarities);
    let std_sim = std_dev(&similarities);
    
    // Ideal: high mean similarity (0.5-0.8) with low variance
    // Too high (>0.9): sounds are clones — boring pack
    // Too low (<0.3): sounds don't belong together — random pack
    // High variance: some sounds fit, some don't
    
    let mean_score = if mean_sim > 0.5 && mean_sim < 0.85 { 1.0 }
                    else if mean_sim > 0.3 { 0.5 + (mean_sim - 0.3) / 0.2 * 0.5 }
                    else { mean_sim / 0.3 * 0.3 };
    
    let variance_penalty = (std_sim * 2.0).min(0.3); // Penalize inconsistency
    
    CohesionScore {
        dimension: "timbral",
        score: (mean_score - variance_penalty).clamp(0.0, 1.0),
        raw_mean: mean_sim,
        raw_std: std_sim,
        interpretation: if mean_sim > 0.85 { "Sounds are too similar — add more variety" }
                        else if mean_sim < 0.3 { "Sounds don't share a timbre palette" }
                        else { "Good timbral cohesion" },
    }
}

fn timbre_embedding(features: &SignalFeatures) -> Vec<f32> {
    // Focus on spectral shape — the core of timbre
    vec![
        features.spectral_centroid_hz / 10000.0,
        features.spectral_flatness,
        features.spectral_bandwidth_hz / 10000.0,
        features.zero_crossing_rate,
        features.harmonic_ratio,
        // Mel spectrum summary (PCA-reduced to 6 dims)
        features.mel_spectrum_pca[0],
        features.mel_spectrum_pca[1],
        features.mel_spectrum_pca[2],
        features.mel_spectrum_pca[3],
        features.mel_spectrum_pca[4],
        features.mel_spectrum_pca[5],
    ]
}
```

### 2.2 Genre Cohesion

```rust
pub fn genre_cohesion(sounds: &[GeneratedSound]) -> CohesionScore {
    // Classify genre for each sound, measure agreement
    let genres: Vec<Option<String>> = sounds.iter()
        .map(|s| classify_genre(&s.features, &s.prompt))
        .collect();
    
    let genre_names: Vec<&str> = genres.iter()
        .filter_map(|g| g.as_deref())
        .collect();
    
    if genre_names.len() < 2 {
        return CohesionScore {
            dimension: "genre",
            score: 0.5,
            raw_mean: 0.0,
            raw_std: 0.0,
            interpretation: "Not enough genre data",
        };
    }
    
    // Measure majority genre proportion
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for g in &genre_names {
        *counts.entry(g).or_default() += 1;
    }
    let majority = counts.values().max().copied().unwrap_or(0);
    let majority_ratio = majority as f32 / genre_names.len() as f32;
    
    // Also check genre similarity (e.g., trap and hip-hop are close)
    let genre_similarity = genre_pairwise_similarity(&genre_names);
    
    let combined = majority_ratio * 0.6 + genre_similarity * 0.4;
    
    CohesionScore {
        dimension: "genre",
        score: combined,
        raw_mean: majority_ratio,
        raw_std: 1.0 - genre_similarity, // std-like: disagreement
        interpretation: if majority_ratio > 0.8 { "Strong genre cohesion" }
                        else if majority_ratio > 0.5 { "Moderate genre agreement" }
                        else { "Genre is mixed across the pack" },
    }
}

fn genre_pairwise_similarity(genres: &[&str]) -> f32 {
    // Genre distance matrix (0.0 = unrelated, 1.0 = same genre)
    let related_pairs: &[(&str, &str, f32)] = &[
        ("trap", "hip-hop", 0.8),
        ("trap", "rnb", 0.6),
        ("house", "techno", 0.6),
        ("house", "electronic", 0.7),
        ("techno", "electronic", 0.7),
        ("lo-fi", "hip-hop", 0.5),
        ("lo-fi", "jazz", 0.4),
        ("cinematic", "ambient", 0.6),
        ("cinematic", "orchestral", 0.8),
        ("dnb", "drum and bass", 1.0),
        ("dnb", "electronic", 0.6),
    ];
    
    let mut total_sim = 0.0_f32;
    let mut pairs = 0;
    
    for i in 0..genres.len() {
        for j in (i+1)..genres.len() {
            if genres[i] == genres[j] {
                total_sim += 1.0;
            } else {
                let sim = related_pairs.iter()
                    .find(|(a, b, _)| {
                        (a == &genres[i] && b == &genres[j]) ||
                        (a == &genres[j] && b == &genres[i])
                    })
                    .map(|(_, _, s)| *s)
                    .unwrap_or(0.1); // Unrelated genres
                total_sim += sim;
            }
            pairs += 1;
        }
    }
    
    if pairs == 0 { 1.0 } else { total_sim / pairs as f32 }
}
```

### 2.3 Emotional Cohesion

```rust
pub fn emotional_cohesion(sounds: &[GeneratedSound]) -> CohesionScore {
    // Map each sound to valence-arousal space
    // Measure clustering in this 2D space
    
    let emotional_profiles: Vec<[f32; 2]> = sounds.iter()
        .map(|s| emotional_profile(&s.features))
        .collect();
    
    // Centroid of all points
    let centroid: [f32; 2] = [
        emotional_profiles.iter().map(|p| p[0]).sum::<f32>() / emotional_profiles.len() as f32,
        emotional_profiles.iter().map(|p| p[1]).sum::<f32>() / emotional_profiles.len() as f32,
    ];
    
    // Average distance from centroid
    let avg_distance: f32 = emotional_profiles.iter()
        .map(|p| ((p[0] - centroid[0]).powi(2) + (p[1] - centroid[1]).powi(2)).sqrt())
        .sum::<f32>() / emotional_profiles.len() as f32;
    
    // Max possible distance in [0,1]² is ~1.414
    // Lower distance = more emotional cohesion
    let score = (1.0 - (avg_distance / 1.414)).max(0.0);
    
    CohesionScore {
        dimension: "emotional",
        score,
        raw_mean: avg_distance,
        raw_std: std_dev(&emotional_profiles.iter().map(|p| p[0]).collect()),
        interpretation: if score > 0.8 { "Strong emotional consistency" }
                        else if score > 0.5 { "Moderate emotional range" }
                        else { "Wide emotional variation" },
    }
}

fn emotional_profile(features: &SignalFeatures) -> [f32; 2] {
    // Valence (positive ↔ negative): spectral centroid + harmonic ratio
    let valence = (
        (features.spectral_centroid_hz / 10000.0) * 0.4  // Brighter = more positive
        + features.harmonic_ratio * 0.3                     // More tonal = more positive
        + (1.0 - features.spectral_flatness) * 0.3          // Less noisy = more positive
    ).clamp(0.0, 1.0);
    
    // Arousal (calm ↔ intense): RMS + crest factor + onset strength
    let arousal = (
        features.rms * 0.3                                  // Louder = more intense
        + (features.crest_factor / 20.0) * 0.3              // More dynamic = more intense
        + features.onset_strength * 0.2                     // Sharper attack = more intense
        + (features.energy_high + features.energy_very_high) * 0.2 // More high freq = more intense
    ).clamp(0.0, 1.0);
    
    [valence, arousal]
}
```

### 2.4 Loudness Consistency

```rust
pub fn loudness_consistency(sounds: &[GeneratedSound]) -> CohesionScore {
    let loudness_values: Vec<f32> = sounds.iter()
        .map(|s| s.features.rms) // Or LUFS if computed
        .collect();
    
    let mean_loudness = mean(&loudness_values);
    let variance = variance(&loudness_values, mean_loudness);
    let std = variance.sqrt();
    
    // In dB terms: how much gain-staging variance
    let std_db = 20.0 * std.log10();
    
    // Ideal: all sounds within ±1.5dB of each other
    let score = if std_db < 1.0 { 1.0 }
                else if std_db < 3.0 { 1.0 - (std_db - 1.0) / 2.0 * 0.4 }
                else if std_db < 6.0 { 0.6 - (std_db - 3.0) / 3.0 * 0.4 }
                else { 0.2 };
    
    CohesionScore {
        dimension: "loudness_consistency",
        score,
        raw_mean: mean_loudness,
        raw_std: std,
        interpretation: if std_db < 1.5 { "Excellent loudness consistency" }
                        else if std_db < 3.0 { "Acceptable variation" }
                        else { "Loudness varies significantly" },
    }
}
```

### 2.5 Spectral Balance

```rust
pub fn spectral_balance_consistency(sounds: &[GeneratedSound]) -> CohesionScore {
    // Get spectral centroid for each sound, check variation
    let centroids: Vec<f32> = sounds.iter()
        .map(|s| s.features.spectral_centroid_hz)
        .collect();
    
    let mean_centroid = mean(&centroids);
    let cv = std_dev(&centroids) / mean_centroid; // Coefficient of variation
    
    // Check energy band distribution consistency
    let band_profiles: Vec<[f32; 6]> = sounds.iter()
        .map(|s| energy_band_profile(&s.features))
        .collect();
    
    // Pairwise correlation of band profiles
    let mut correlations = Vec::new();
    for i in 0..band_profiles.len() {
        for j in (i+1)..band_profiles.len() {
            correlations.push(correlation(&band_profiles[i], &band_profiles[j]));
        }
    }
    let mean_corr = mean(&correlations);
    
    // Combined score: low centroid CV + high band correlation
    let cv_score = if cv < 0.3 { 1.0 }
                   else if cv < 0.5 { 0.7 }
                   else if cv < 0.8 { 0.4 }
                   else { 0.1 };
    
    let corr_score = mean_corr.max(0.0);
    
    CohesionScore {
        dimension: "spectral_balance",
        score: cv_score * 0.4 + corr_score * 0.6,
        raw_mean: mean_centroid,
        raw_std: std_dev(&centroids),
        interpretation: if cv < 0.3 { "Consistent spectral balance" }
                        else if cv < 0.5 { "Moderate spectral variation" }
                        else { "Wide spectral range across pack" },
    }
}

fn energy_band_profile(features: &SignalFeatures) -> [f32; 6] {
    let total = features.energy_sub_low + features.energy_low
              + features.energy_low_mid + features.energy_mid
              + features.energy_high_mid + features.energy_high;
    if total == 0.0 { return [0.0; 6]; }
    [
        features.energy_sub_low / total,
        features.energy_low / total,
        features.energy_low_mid / total,
        features.energy_mid / total,
        features.energy_high_mid / total,
        features.energy_high / total,
    ]
}
```

### 2.6 Transient Compatibility

```rust
pub fn transient_compatibility(sounds: &[GeneratedSound]) -> CohesionScore {
    let attack_times: Vec<f32> = sounds.iter()
        .map(|s| s.features.attack_time_ms)
        .collect();
    
    let crest_factors: Vec<f32> = sounds.iter()
        .map(|s| s.features.crest_factor)
        .collect();
    
    // Within-type consistency (kicks should have similar attack to each other)
    let by_type = group_by_type(sounds);
    let within_type_scores: Vec<f32> = by_type.iter().map(|(_, group)| {
        let attacks: Vec<f32> = group.iter().map(|s| s.features.attack_time_ms).collect();
        let mean_a = mean(&attacks);
        let cv_a = std_dev(&attacks) / mean_a.max(1.0);
        if cv_a < 0.3 { 1.0 } else if cv_a < 0.5 { 0.7 } else { 0.3 }
    }).collect();
    
    let within_type_score = mean(&within_type_scores);
    
    // Across-type complementarity (kicks attack vs snares attack should differ appropriately)
    // Different types should have distinct but compatible transient characters
    
    CohesionScore {
        dimension: "transient",
        score: within_type_score,
        raw_mean: mean(&attack_times),
        raw_std: std_dev(&attack_times),
        interpretation: if within_type_score > 0.7 { "Consistent transient character" }
                        else { "Transients vary within types" },
    }
}
```

### 2.7 Stereo Consistency

```rust
pub fn stereo_consistency(sounds: &[GeneratedSound]) -> CohesionScore {
    // For mono generation, this is trivially 1.0
    // For future stereo: check correlation between L/R across pack
    
    // Placeholder for now — mono is the default
    CohesionScore {
        dimension: "stereo_consistency",
        score: 1.0,
        raw_mean: 1.0,
        raw_std: 0.0,
        interpretation: "All sounds are mono (consistent)",
    }
}
```

### 2.8 Production Style Similarity

```rust
pub fn production_style_similarity(sounds: &[GeneratedSound]) -> CohesionScore {
    // Measure how processed/polished each sound is
    // Features: dynamic range, noise floor, spectral balance
    
    let style_signals: Vec<[f32; 4]> = sounds.iter()
        .map(|s| {
            [
                s.features.crest_factor,                                  // Dynamic range
                s.features.rms,                                            // Perceived loudness
                1.0 - s.features.spectral_flatness,                       // Cleanness
                (1.0 - (s.features.noise_floor_dbfs / -80.0).abs()).max(0.0), // Polish
            ]
        })
        .collect();
    
    let mean_style: [f32; 4] = [
        style_signals.iter().map(|s| s[0]).sum::<f32>() / style_signals.len() as f32,
        style_signals.iter().map(|s| s[1]).sum::<f32>() / style_signals.len() as f32,
        style_signals.iter().map(|s| s[2]).sum::<f32>() / style_signals.len() as f32,
        style_signals.iter().map(|s| s[3]).sum::<f32>() / style_signals.len() as f32,
    ];
    
    let avg_deviation: f32 = style_signals.iter()
        .map(|s| {
            (s[0] - mean_style[0]).abs() / mean_style[0].max(0.01) +
            (s[1] - mean_style[1]).abs() / mean_style[1].max(0.01) +
            (s[2] - mean_style[2]).abs() / mean_style[2].max(0.01) +
            (s[3] - mean_style[3]).abs() / mean_style[3].max(0.01)
        })
        .sum::<f32>() / style_signals.len() as f32;
    
    let score = (1.0 - (avg_deviation / 4.0).min(1.0)).max(0.0);
    
    CohesionScore {
        dimension: "production_style",
        score,
        raw_mean: avg_deviation,
        raw_std: 0.0,
        interpretation: if score > 0.7 { "Consistent production style" }
                        else { "Production quality varies across pack" },
    }
}
```

---

## 3. Combined Cohesion Score

```rust
pub struct PackCohesion {
    pub dimensions: Vec<CohesionScore>,
    pub overall: f32,
    pub weakest_dimension: String,
    pub strongest_dimension: String,
}

impl PackCohesion {
    pub fn compute(sounds: &[GeneratedSound]) -> Self {
        let dimensions = vec![
            timbral_cohesion(sounds),
            genre_cohesion(sounds),
            emotional_cohesion(sounds),
            loudness_consistency(sounds),
            spectral_balance_consistency(sounds),
            transient_compatibility(sounds),
            stereo_consistency(sounds),
            production_style_similarity(sounds),
        ];
        
        let overall = dimensions.iter().map(|d| d.score).sum::<f32>() / dimensions.len() as f32;
        
        let weakest = dimensions.iter()
            .min_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            .map(|d| d.dimension.to_string())
            .unwrap_or_default();
        
        let strongest = dimensions.iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            .map(|d| d.dimension.to_string())
            .unwrap_or_default();
        
        PackCohesion { dimensions, overall, weakest_dimension: weakest, strongest_dimension: strongest }
    }
}

pub struct CohesionScore {
    pub dimension: &'static str,
    pub score: f32,
    pub raw_mean: f32,
    pub raw_std: f32,
    pub interpretation: &'static str,
}
```

---

## 4. Embedding-Based Pack Clustering

```rust
pub struct PackEmbeddingClusterer {
    pub algorithm: ClusteringAlgorithm,
    pub distance_metric: DistanceMetric,
    pub min_cohesion: f32, // 0.0-1.0, minimum cohesion to accept clustering
}

pub enum ClusteringAlgorithm {
    KMeans { k: usize },
    Agglomerative { n_clusters: usize, linkage: Linkage },
    DBSCAN { eps: f32, min_samples: usize },
    HDBSCAN { min_cluster_size: usize },
}

pub enum DistanceMetric {
    Cosine,
    Euclidean,
    Manhattan,
    Correlation,
}

impl PackEmbeddingClusterer {
    pub fn cluster_library(&self, sounds: &[GeneratedSound]) -> Vec<PackCluster> {
        // 1. Compute embeddings for all sounds
        let embeddings: Vec<Vec<f32>> = sounds.iter()
            .map(|s| full_embedding(&s.features))
            .collect();
        
        // 2. Apply clustering algorithm
        let labels = match self.algorithm {
            ClusteringAlgorithm::KMeans { k } => kmeans(&embeddings, k, &self.distance_metric),
            ClusteringAlgorithm::Agglomerative { n_clusters, linkage } => {
                agglomerative(&embeddings, n_clusters, linkage, &self.distance_metric)
            },
            ClusteringAlgorithm::DBSCAN { eps, min_samples } => {
                dbscan(&embeddings, eps, min_samples, &self.distance_metric)
            },
            ClusteringAlgorithm::HDBSCAN { min_cluster_size } => {
                hdbscan(&embeddings, min_cluster_size, &self.distance_metric)
            },
        };
        
        // 3. Group sounds by cluster label
        let mut clusters: HashMap<isize, Vec<GeneratedSound>> = HashMap::new();
        for (label, sound) in labels.into_iter().zip(sounds.iter()) {
            if label >= 0 { // -1 = noise point in DBSCAN
                clusters.entry(label).or_default().push(sound.clone());
            }
        }
        
        // 4. Calculate cohesion for each cluster
        let mut pack_clusters = Vec::new();
        for (label, cluster_sounds) in clusters {
            let cohesion = PackCohesion::compute(&cluster_sounds);
            
            if cohesion.overall >= self.min_cohesion {
                pack_clusters.push(PackCluster {
                    label,
                    sounds: cluster_sounds,
                    cohesion,
                });
            }
        }
        
        // 5. Sort by cohesion (best packs first)
        pack_clusters.sort_by(|a, b| b.cohesion.overall.partial_cmp(&a.cohesion.overall).unwrap());
        
        pack_clusters
    }
}

fn full_embedding(features: &SignalFeatures) -> Vec<f32> {
    // Comprehensive embedding for clustering
    let mut emb = Vec::new();
    
    // Timbre (11 dims)
    emb.push(features.spectral_centroid_hz / 10000.0);
    emb.push(features.spectral_flatness);
    emb.push(features.spectral_bandwidth_hz / 10000.0);
    emb.push(features.zero_crossing_rate);
    emb.push(features.harmonic_ratio);
    emb.extend_from_slice(&features.mel_spectrum_pca);
    
    // Dynamics (4 dims)
    emb.push(features.crest_factor / 20.0);
    emb.push(features.rms);
    emb.push(features.attack_time_ms / 100.0);
    emb.push(features.transient_ratio);
    
    // Spectral shape (6 dims)
    emb.push(features.energy_sub_low);
    emb.push(features.energy_low);
    emb.push(features.energy_low_mid);
    emb.push(features.energy_mid);
    emb.push(features.energy_high_mid);
    emb.push(features.energy_high);
    
    // Temporal (3 dims)
    emb.push(features.duration_ms / 5000.0);
    emb.push(features.decay_time_ms / 500.0);
    emb.push(features.release_time_ms / 1000.0);
    
    emb
}
```

---

## 5. Outlier Detection

```rust
pub fn detect_outliers(sounds: &[GeneratedSound], threshold: f32) -> Vec<usize> {
    // Find sounds that don't fit with the rest of the pack
    // Uses isolation forest or simple distance-based approach
    
    let embeddings: Vec<Vec<f32>> = sounds.iter()
        .map(|s| full_embedding(&s.features))
        .collect();
    
    // 1. Compute average embedding (centroid)
    let centroid = mean_embedding(&embeddings);
    
    // 2. Compute distance of each sound from centroid
    let distances: Vec<f32> = embeddings.iter()
        .map(|emb| cosine_distance(emb, &centroid))
        .collect();
    
    // 3. Find sounds beyond threshold (mean + 2*std)
    let mean_dist = mean(&distances);
    let std_dist = std_dev(&distances);
    let cutoff = mean_dist + threshold * std_dist;
    
    distances.iter()
        .enumerate()
        .filter(|(_, &d)| d > cutoff)
        .map(|(i, _)| i)
        .collect()
}

pub fn suggest_repair_for_outliers(sounds: &mut [GeneratedSound], outlier_indices: &[usize]) {
    for &idx in outlier_indices {
        let sound = &sounds[idx];
        let cohesion = PackCohesion::compute(&[sound.clone()]);
        
        println!("Outlier #{}: {}", idx, cohesion.weakest_dimension);
        println!("  Suggestion: Regenerate with prompt adjusted for [{}]", 
                 cohesion.weakest_dimension);
    }
}
```

---

## 6. Diversity Controls

```rust
pub struct DiversityController {
    pub min_diversity: f32,      // 0.0-1.0, minimum diversity within type
    pub max_diversity: f32,      // 0.0-1.0, maximum diversity within type
    pub target_per_type: usize,  // desired number per category
}

impl DiversityController {
    pub fn enforce(&self, sounds: &[GeneratedSound]) -> Vec<GeneratedSound> {
        let by_type = group_by_type(sounds);
        let mut result = Vec::new();
        
        for (type_name, group) in by_type {
            if group.len() <= 1 {
                result.extend(group);
                continue;
            }
            
            let embeddings: Vec<Vec<f32>> = group.iter()
                .map(|s| full_embedding(&s.features))
                .collect();
            
            // Measure current diversity
            let mut pair_dists = Vec::new();
            for i in 0..embeddings.len() {
                for j in (i+1)..embeddings.len() {
                    pair_dists.push(cosine_distance(&embeddings[i], &embeddings[j]));
                }
            }
            let mean_dist = mean(&pair_dists);
            
            // Normalize distance to 0-1 diversity score
            let diversity = (mean_dist * 2.0).min(1.0);
            
            if diversity < self.min_diversity {
                // Too similar — regenerate some with more variation
                println!("{} diversity too low ({:.2}), need > {}", 
                         type_name, diversity, self.min_diversity);
                // Mark for regeneration with more diverse prompts
            } else if diversity > self.max_diversity {
                // Too varied — sounds might not belong together
                println!("{} diversity too high ({:.2}), need < {}", 
                         type_name, diversity, self.max_diversity);
                // Could split into subgroups
            }
            
            result.extend(group);
        }
        
        result
    }
}
```

---

## 7. Pack Completeness Scoring

```rust
pub struct PackCompleteness {
    pub has_essential_types: bool,       // Kick, snare, hat for drum packs
    pub type_distribution: f32,          // Are types balanced?
    pub size_appropriateness: f32,       // Right size for the genre?
    pub variation_depth: f32,            // Enough variation per type?
    pub overall: f32,
}

pub fn score_completeness(sounds: &[GeneratedSound], template: &PackTemplate) -> PackCompleteness {
    let by_type = group_by_type(sounds);
    let types_present: Vec<&str> = by_type.keys().map(|s| s.as_str()).collect();
    
    // Essential types present?
    let essential = ["kick", "snare", "hihat"];
    let has_essential = essential.iter()
        .all(|e| types_present.contains(e));
    
    // Type distribution (evenness)
    let counts: Vec<f32> = by_type.values().map(|g| g.len() as f32).collect();
    let max_count = counts.iter().cloned().fold(0.0_f32, f32::max);
    let min_count = counts.iter().cloned().fold(f32::MAX, f32::min);
    let type_distribution = if max_count > 0.0 {
        1.0 - (max_count - min_count) / max_count
    } else {
        0.0
    };
    
    // Size appropriateness
    let total = sounds.len();
    let size_appropriateness = if total >= template.min_per_type * template.sound_types.len() {
        1.0
    } else {
        total as f32 / (template.min_per_type * template.sound_types.len()) as f32
    };
    
    // Variation depth (at least 3 per type for drum packs)
    let variation_scores: Vec<f32> = by_type.iter().map(|(_, group)| {
        let depth = group.len() as f32;
        if depth >= 4.0 { 1.0 }
        else if depth >= 3.0 { 0.8 }
        else if depth >= 2.0 { 0.5 }
        else { 0.2 }
    }).collect();
    let variation_depth = mean(&variation_scores);
    
    let overall = (if has_essential { 0.3 } else { 0.0 })
                + type_distribution * 0.25
                + size_appropriateness * 0.25
                + variation_depth * 0.20;
    
    PackCompleteness {
        has_essential_types: has_essential,
        type_distribution,
        size_appropriateness,
        variation_depth,
        overall: overall.clamp(0.0, 1.0),
    }
}
```

---

## 8. Visualization

### Cohesion Radar Chart

```
        Timbral
          │
          0.82
          │
Genre─────┼─────Emotional
  0.75    │      0.68
          │
          │
Loudness──┼─────Spectral
  0.91    │      0.77
          │
          │
Transient─┼─────Production
  0.71    │      0.83
          │
       Stereo
        1.00

Overall Cohesion: 0.81 ⭐ Excellent
Weakest: Emotional (0.68) — consider mood adjustment
```

### Pack Comparison View

```
PACK A: Trap Dark Kit 001
  Cohesion: 0.81
  Types: 5   Sounds: 14
  Avg Score: 82%
  Best for: Trap, Hip-hop producers

PACK B: Lo-Fi Warm Drums
  Cohesion: 0.73
  Types: 4   Sounds: 10
  Avg Score: 76%
  Best for: Lo-fi, Chill producers
  ⚠ Lower timbral cohesion — wider sonic range

PACK C: Cinematic Impacts
  Cohesion: 0.88
  Types: 3   Sounds: 8
  Avg Score: 79%
  Best for: Game audio, film
  ✓ Strong emotional consistency
```

---

## 9. Summary

Eight cohesion metrics measure whether sounds belong together. Timbral, genre, emotional, loudness, spectral, transient, stereo, and production style each contribute to an overall cohesion score. Embedding-based clustering groups library sounds into pack proposals. Outlier detection flags sounds that don't fit. Diversity controls ensure enough variation within each type. Pack completeness scoring tells cShot when a pack is ready to export. The result: cShot knows when a group of sounds forms a professional sample pack versus a random collection.
