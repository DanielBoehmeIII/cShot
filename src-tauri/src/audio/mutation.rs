use super::analyze::AudioAnalysis;
use super::resynthesize::{self};
use super::recreate::{self, compute_similarity, SimilarityReport, params_from_analysis};

// ─── Mutation Types ─────────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum MutationType {
    Recreate,
    Mutate(f32),           // amount 0.0-1.0
    Hybridize,             // blend with another sound
    Evolve(f32),           // amount 0.0-1.0
    Branch(String),        // variant name
    CleanUp,
    Exaggerate(f32),       // amount 0.0-1.0
    Modernize,
    GenreShift(String),    // target genre
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MutationResult {
    pub id: String,
    pub samples: Vec<f32>,
    pub similarity: SimilarityReport,
    pub mutation: MutationType,
    pub description: String,
    pub params: String,
}

// ─── Core Mutation Engine ────────────────────────────────

pub fn mutate_sound(
    samples: &[f32],
    analysis: &AudioAnalysis,
    amount: f32,
) -> (Vec<f32>, SimilarityReport) {
    let base_params = params_from_analysis(analysis, samples);
    let seed = (amount * 1000.0) as u64;
    let mutated = base_params.clone().with_seed(seed).randomize(amount);
    let recreated = resynthesize::resynthesize(&mutated);
    let similarity = compute_similarity(samples, &recreated, analysis);
    (recreated, similarity)
}

pub fn evolve_sound(
    samples: &[f32],
    analysis: &AudioAnalysis,
    amount: f32,
    direction: &str,
) -> (Vec<f32>, SimilarityReport) {
    let base_params = params_from_analysis(analysis, samples);
    let seed = (amount * 2000.0) as u64 + 100;
    let mut params = base_params.clone().with_seed(seed);

    match direction {
        "harder" => {
            params.saturation_drive = (base_params.saturation_drive + amount).min(5.0);
            params.click_amount = (base_params.click_amount + amount * 0.3).min(1.0);
        }
        "cleaner" => {
            params.saturation_drive = (base_params.saturation_drive - amount * 0.5).max(1.0);
            params.noise_amount = (base_params.noise_amount - amount * 0.3).max(0.0);
        }
        "warmer" => {
            params.brightness = (base_params.brightness - amount * 0.3).max(0.0);
            params.saturation_drive = (base_params.saturation_drive + amount * 0.3).min(3.0);
        }
        "brighter" => {
            params.brightness = (base_params.brightness + amount * 0.3).min(1.0);
            params.saturation_drive = (base_params.saturation_drive + amount * 0.2).min(3.0);
        }
        "heavier" => {
            params.body_gain = (base_params.body_gain + amount * 0.2).min(1.0);
            params.sub_gain = (base_params.sub_gain + amount * 0.3).min(1.0);
        }
        "lighter" => {
            params.body_gain = (base_params.body_gain - amount * 0.3).max(0.1);
            params.sub_gain = (base_params.sub_gain - amount * 0.4).max(0.0);
            params.duration_ms *= 1.0 - amount * 0.3;
        }
        "longer" => {
            params.duration_ms = base_params.duration_ms * (1.0 + amount * 0.5);
            params.tail_ms = base_params.tail_ms + amount * 200.0;
        }
        "shorter" => {
            params.duration_ms = base_params.duration_ms * (1.0 - amount * 0.4);
            params.tail_ms = (base_params.tail_ms - amount * 100.0).max(0.0);
        }
        _ => {
            params = params.randomize(amount * 0.5);
        }
    }

    let recreated = resynthesize::resynthesize(&params);
    let similarity = compute_similarity(samples, &recreated, analysis);
    (recreated, similarity)
}

pub fn hybridize_sounds(
    samples_a: &[f32],
    analysis_a: &AudioAnalysis,
    samples_b: &[f32],
    analysis_b: &AudioAnalysis,
    blend: f32,
) -> (Vec<f32>, SimilarityReport) {
    let params_a = params_from_analysis(analysis_a, samples_a);
    let params_b = params_from_analysis(analysis_b, samples_b);
    let hybrid = super::midi::morph_params(&params_a, &params_b, blend);
    let recreated = resynthesize::resynthesize(&hybrid);
    let similarity = compute_similarity(samples_a, &recreated, analysis_a);
    (recreated, similarity)
}

pub fn clean_up_sound(
    samples: &[f32],
    analysis: &AudioAnalysis,
) -> (Vec<f32>, SimilarityReport) {
    let base_params = params_from_analysis(analysis, samples);
    let mut params = base_params.clone();
    params.noise_amount = params.noise_amount * 0.3;
    params.saturation_drive = 1.0;
    let recreated = resynthesize::resynthesize(&params);
    let similarity = compute_similarity(samples, &recreated, analysis);
    (recreated, similarity)
}

pub fn exaggerate_sound(
    samples: &[f32],
    analysis: &AudioAnalysis,
    amount: f32,
    quality: &str,
) -> (Vec<f32>, SimilarityReport) {
    let base_params = params_from_analysis(analysis, samples);
    let mut params = base_params.clone();

    match quality {
        "punch" | "punchy" => {
            params.click_amount = (params.click_amount + amount * 0.4).min(1.0);
            params.attack_ms = (params.attack_ms * (1.0 - amount * 0.3)).max(0.5);
            params.saturation_drive = (params.saturation_drive + amount * 0.3).min(4.0);
        }
        "sub" | "subby" => {
            params.sub_gain = (params.sub_gain + amount * 0.5).min(1.0);
        }
        "bright" | "brightness" => {
            params.brightness = (params.brightness + amount * 0.5).min(1.0);
        }
        "dark" | "darkness" => {
            params.brightness = (params.brightness - amount * 0.5).max(0.0);
        }
        "noise" | "noisy" => {
            params.noise_amount = (params.noise_amount + amount * 0.5).min(1.0);
        }
        "distortion" | "distorted" => {
            params.saturation_drive = (params.saturation_drive + amount * 1.5).min(6.0);
        }
        "long" | "length" => {
            params.duration_ms = params.duration_ms * (1.0 + amount * 0.8);
            params.tail_ms = params.tail_ms + amount * 300.0;
        }
        "short" => {
            params.duration_ms = params.duration_ms * (1.0 - amount * 0.5);
            params.tail_ms = (params.tail_ms - amount * 100.0).max(0.0);
        }
        "tight" | "tightness" => {
            params.decay_ms *= 1.0 - amount * 0.5;
            params.tail_ms = 0.0;
            params.attack_ms = (params.attack_ms * 0.7).max(0.5);
        }
        _ => {
            params = params.randomize(amount * 0.4);
        }
    }

    let recreated = resynthesize::resynthesize(&params);
    let similarity = compute_similarity(samples, &recreated, analysis);
    (recreated, similarity)
}

pub fn modernize_sound(
    samples: &[f32],
    analysis: &AudioAnalysis,
) -> (Vec<f32>, SimilarityReport) {
    let base_params = params_from_analysis(analysis, samples);
    let mut params = base_params.clone();
    // Modern production: tighter, punchier, cleaner low end
    params.decay_ms *= 0.7;
    params.click_amount = (params.click_amount + 0.15).min(1.0);
    params.sub_gain = (params.sub_gain + 0.1).min(1.0);
    params.brightness = (params.brightness + 0.1).min(1.0);
    params.saturation_drive = (params.saturation_drive + 0.2).min(3.0);
    params.noise_amount = (params.noise_amount * 0.5).max(0.0);
    params.attack_ms = (params.attack_ms * 0.8).max(0.5);
    let recreated = resynthesize::resynthesize(&params);
    let similarity = compute_similarity(samples, &recreated, analysis);
    (recreated, similarity)
}

pub fn genre_shift_sound(
    samples: &[f32],
    analysis: &AudioAnalysis,
    target_genre: &str,
) -> (Vec<f32>, SimilarityReport) {
    recreate::genre_shift(samples, &analysis.sound_type_hint, target_genre)
}

// ─── Batch Mutation ──────────────────────────────────────

pub fn apply_mutation_preset(
    samples: &[f32],
    analysis: &AudioAnalysis,
    mutation: &str,
    intensity: f32,
) -> MutationResult {
    let (mutated, similarity) = match mutation {
        "recreate" => {
            let (s, _a, sim) = recreate::recreate_single(samples, intensity);
            (s, sim)
        }
        "mutate" => mutate_sound(samples, analysis, intensity),
        "clean" | "clean-up" | "cleanup" => clean_up_sound(samples, analysis),
        "exaggerate" | "exaggerate-punch" => exaggerate_sound(samples, analysis, intensity, "punch"),
        "exaggerate-sub" => exaggerate_sound(samples, analysis, intensity, "sub"),
        "exaggerate-bright" => exaggerate_sound(samples, analysis, intensity, "bright"),
        "exaggerate-dark" => exaggerate_sound(samples, analysis, intensity, "dark"),
        "exaggerate-distortion" => exaggerate_sound(samples, analysis, intensity, "distortion"),
        "exaggerate-short" => exaggerate_sound(samples, analysis, intensity, "short"),
        "exaggerate-long" => exaggerate_sound(samples, analysis, intensity, "long"),
        "modernize" => modernize_sound(samples, analysis),
        "evolve-harder" => evolve_sound(samples, analysis, intensity, "harder"),
        "evolve-cleaner" => evolve_sound(samples, analysis, intensity, "cleaner"),
        "evolve-warmer" => evolve_sound(samples, analysis, intensity, "warmer"),
        "evolve-brighter" => evolve_sound(samples, analysis, intensity, "brighter"),
        "evolve-heavier" => evolve_sound(samples, analysis, intensity, "heavier"),
        "evolve-lighter" => evolve_sound(samples, analysis, intensity, "lighter"),
        "evolve-longer" => evolve_sound(samples, analysis, intensity, "longer"),
        "evolve-shorter" => evolve_sound(samples, analysis, intensity, "shorter"),
        "genre-trap" => genre_shift_sound(samples, analysis, "trap"),
        "genre-techno" => genre_shift_sound(samples, analysis, "techno"),
        "genre-cinematic" => genre_shift_sound(samples, analysis, "cinematic"),
        "genre-lo-fi" | "genre-lofi" => genre_shift_sound(samples, analysis, "lo-fi"),
        "genre-drill" => genre_shift_sound(samples, analysis, "drill"),
        "genre-house" => genre_shift_sound(samples, analysis, "house"),
        "genre-dubstep" => genre_shift_sound(samples, analysis, "dubstep"),
        _ => {
            let (s, _a, sim) = recreate::recreate_single(samples, 0.5);
            (s, sim)
        }
    };

    MutationResult {
        id: uuid::Uuid::new_v4().to_string(),
        samples: mutated,
        similarity,
        mutation: match mutation {
            "recreate" => MutationType::Recreate,
            "mutate" => MutationType::Mutate(intensity),
            "clean" | "clean-up" | "cleanup" => MutationType::CleanUp,
            m if m.starts_with("exaggerate") => MutationType::Exaggerate(intensity),
            "modernize" => MutationType::Modernize,
            m if m.starts_with("evolve") => MutationType::Evolve(intensity),
            m if m.starts_with("genre") => MutationType::GenreShift(m.trim_start_matches("genre-").to_string()),
            _ => MutationType::Mutate(intensity),
        },
        description: format!("{} (intensity: {:.0}%)", mutation, intensity * 100.0),
        params: String::new(),
    }
}
