use std::collections::{HashMap, HashSet};
use crate::audio::SoundType;
use crate::audio::analyze::AudioAnalysis;
use crate::quality::QualityMetadata;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct SoundScore {
    pub overall: u32,
    pub no_clipping: bool,
    pub reasonable_duration: bool,
    pub non_silent: bool,
    pub clean_save: bool,
    pub user_signal: i32,
    pub failure_labels: Vec<String>,
    // Enhanced scoring fields
    pub novelty_score: f32,
    pub similarity_score: f32,
    pub recipe_history_score: f32,
    pub export_history_score: f32,
    pub quality_score: f32,
    pub taste_score: f32,
    pub ranking_factors: RankingFactors,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct RankingFactors {
    pub spectral_balance_weight: f32,
    pub transient_quality_weight: f32,
    pub tonal_clarity_weight: f32,
    pub novelty_weight: f32,
    pub user_affinity_weight: f32,
    pub quality_weight: f32,
}

impl Default for RankingFactors {
    fn default() -> Self {
        Self {
            spectral_balance_weight: 0.15,
            transient_quality_weight: 0.15,
            tonal_clarity_weight: 0.1,
            novelty_weight: 0.15,
            user_affinity_weight: 0.25,
            quality_weight: 0.2,
        }
    }
}

pub fn compute_score(
    quality: &QualityMetadata,
    _sound_type: SoundType,
    has_user_feedback: Option<bool>,
    usable: Option<bool>,
) -> SoundScore {
    let mut score = 50.0f32;
    let mut user_signal = 0i32;

    let no_clipping = !quality.clipping_detected;
    if no_clipping { score += 15.0; } else { score -= 15.0; }

    let reasonable_duration = quality.duration_appropriate;
    if reasonable_duration { score += 10.0; } else { score -= 10.0; }

    let non_silent = !quality.is_silent;
    if non_silent { score += 10.0; } else { score -= 20.0; }

    if quality.is_too_quiet { score -= 5.0; } else { score += 5.0; }

    score += 5.0;

    if let Some(thumbs_up) = has_user_feedback {
        if thumbs_up { score += 10.0; user_signal += 10; }
        else { score -= 10.0; user_signal -= 10; }
    }

    if let Some(is_usable) = usable {
        if is_usable { score += 5.0; user_signal += 5; }
        else { score -= 10.0; user_signal -= 10; }
    }

    let failure_labels = compute_labels(quality);

    // Enhanced quality composite score
    let quality_score = (
        quality.spectral_quality * 0.2 +
        quality.transient_quality * 0.25 +
        quality.tonal_clarity * 0.1 +
        quality.noise_floor_quality * 0.1 +
        quality.dynamic_range * 0.15 +
        quality.spectral_balance * 0.1 +
        quality.punch_quality * 0.1
    ).clamp(0.0, 1.0);

    score += quality_score * 10.0;

    let overall = score.clamp(0.0, 100.0) as u32;

    SoundScore {
        overall,
        no_clipping,
        reasonable_duration,
        non_silent,
        clean_save: true,
        user_signal,
        failure_labels,
        novelty_score: 0.5,
        similarity_score: 0.5,
        recipe_history_score: 0.5,
        export_history_score: 0.5,
        quality_score,
        taste_score: 0.5,
        ranking_factors: RankingFactors::default(),
    }
}

pub fn compute_enhanced_score(
    quality: &QualityMetadata,
    sound_type: SoundType,
    has_user_feedback: Option<bool>,
    usable: Option<bool>,
    analysis: Option<&AudioAnalysis>,
    library_sounds: &[SoundTypeStats],
    taste_profile: Option<&TasteInput>,
) -> SoundScore {
    let mut base = compute_score(quality, sound_type, has_user_feedback, usable);

    // Novelty scoring: how different is this from existing library sounds
    let novelty = compute_novelty(analysis, library_sounds);
    base.novelty_score = novelty;

    // Taste scoring: how well does this match user preferences
    if let Some(taste) = taste_profile {
        let taste_s = compute_taste_match(taste, analysis, &quality, &sound_type);
        base.taste_score = taste_s;
    }

    // Adjust overall score with novelty and taste
    let novelty_bonus = (novelty - 0.5) * 0.1;
    let taste_bonus = (base.taste_score - 0.5) * 0.15;
    let quality_bonus = (base.quality_score - 0.5) * 0.15;

    let adjusted = base.overall as f32 + novelty_bonus * 100.0 + taste_bonus * 100.0 + quality_bonus * 100.0;
    base.overall = adjusted.clamp(0.0, 100.0) as u32;

    base
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SoundTypeStats {
    pub sound_type: String,
    pub count: usize,
    pub avg_brightness: f32,
    pub avg_energy: f32,
    pub avg_duration_ms: f32,
}

pub struct TasteInput {
    pub preferred_type_scores: HashMap<String, f32>,
    pub preferred_genre_scores: HashMap<String, f32>,
    pub preferred_brightness: f32,
    pub preferred_energy: f32,
    pub avoided_qualities: Vec<String>,
    pub top_terms: Vec<(String, f32)>,
    pub total_actions: u64,
}

fn compute_novelty(analysis: Option<&AudioAnalysis>, library: &[SoundTypeStats]) -> f32 {
    let analysis = match analysis {
        Some(a) => a,
        None => return 0.5,
    };

    if library.is_empty() { return 0.8; }

    let mut novelty = 0.5f32;

    let same_type: Vec<&SoundTypeStats> = library.iter()
        .filter(|s| s.sound_type == analysis.sound_type_hint)
        .collect();

    if !same_type.is_empty() {
        let avg_brightness: f32 = same_type.iter().map(|s| s.avg_brightness).sum::<f32>() / same_type.len() as f32;
        let bright_diff = (analysis.brightness - avg_brightness).abs();
        novelty += bright_diff * 0.2;

        let avg_duration: f32 = same_type.iter().map(|s| s.avg_duration_ms).sum::<f32>() / same_type.len() as f32;
        let dur_diff = ((analysis.duration_ms - avg_duration).abs() / avg_duration.max(1.0)).min(1.0);
        novelty += dur_diff * 0.1;
    }

    let total_count: usize = library.iter().map(|s| s.count).sum();
    let rare_types: Vec<&SoundTypeStats> = library.iter()
        .filter(|s| s.count < 3 && s.count > 0)
        .collect();

    if same_type.len() >= total_count.saturating_sub(rare_types.len().max(1)) {
        novelty -= 0.05;
    }

    // Adjust for undersampled types
    if same_type.is_empty() {
        novelty += 0.2;
    }

    novelty.clamp(0.0, 1.0)
}

fn compute_taste_match(
    taste: &TasteInput,
    analysis: Option<&AudioAnalysis>,
    quality: &QualityMetadata,
    sound_type: &SoundType,
) -> f32 {
    let type_str = sound_type.as_str();
    let type_score = taste.preferred_type_scores.get(type_str).copied().unwrap_or(0.5);

    let mut score = type_score * 0.35;

    if let Some(analysis) = analysis {
        let bright_diff = 1.0 - (analysis.brightness - taste.preferred_brightness).abs();
        score += bright_diff * 0.2;

        let energy_match = {
            let crest = if quality.rms > 0.0 { quality.peak / quality.rms } else { 1.0 };
            let energy = (crest / 15.0).min(1.0);
            1.0 - (energy - taste.preferred_energy).abs()
        };
        score += energy_match * 0.15;
    }

    let quality_match = (
        quality.spectral_quality * 0.1 +
        quality.transient_quality * 0.1 +
        quality.noise_floor_quality * 0.05 +
        (1.0 - quality.clipping_percent / 100.0) * 0.05
    );
    score += quality_match * 0.15;

    if taste.avoided_qualities.contains(&type_str.to_string()) {
        score *= 0.6;
    }

    score.clamp(0.0, 1.0)
}

fn compute_labels(quality: &QualityMetadata) -> Vec<String> {
    let mut labels = Vec::new();
    if quality.clipping_detected { labels.push("clipped".to_string()); }
    if !quality.duration_appropriate && quality.duration_ms > 0.0 {
        if quality.duration_ms > 2000.0 { labels.push("too long".to_string()); }
        else { labels.push("duration".to_string()); }
    }
    if quality.is_too_quiet { labels.push("too quiet".to_string()); }
    if quality.is_silent { labels.push("silent".to_string()); }
    if quality.spectral_quality < 0.3 { labels.push("unbalanced".to_string()); }
    if quality.transient_quality < 0.3 { labels.push("muddy-transient".to_string()); }
    if quality.noise_floor_quality < 0.3 { labels.push("noisy".to_string()); }
    labels
}

pub fn rank_variants(
    variants: Vec<crate::generator::VariantResult>,
    quality_map: HashMap<String, QualityMetadata>,
    taste: Option<&TasteInput>,
) -> Vec<crate::generator::VariantResult> {
    let mut scored: Vec<(f32, crate::generator::VariantResult)> = variants
        .into_iter()
        .map(|v| {
            let q = quality_map.get(&v.id);
            let quality_score = q.map(|q| (
                q.spectral_quality * 0.2 +
                q.transient_quality * 0.25 +
                q.noise_floor_quality * 0.1 +
                q.dynamic_range * 0.15 +
                (1.0 - q.clipping_percent / 100.0) * 0.15
            )).unwrap_or(0.5);

            let taste_score = if let Some(t) = taste {
                let type_s = t.preferred_type_scores.get(&v.sound_type).copied().unwrap_or(0.5);
                type_s * 0.5 + quality_score * 0.5
            } else {
                quality_score
            };

            let final_score = v.score as f32 * 0.4 + taste_score * 100.0 * 0.4 + quality_score * 100.0 * 0.2;
            (final_score, v)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().map(|(_, v)| v).collect()
}
