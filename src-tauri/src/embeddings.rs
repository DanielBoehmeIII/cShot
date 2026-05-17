use crate::audio;
use crate::audio::{SAMPLE_RATE, SoundType};
use crate::db::SoundEntry;

const EMBEDDING_DIM: usize = 64;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Embedding {
    pub sound_id: String,
    pub vector: Vec<f32>,
    pub provider: String,
    pub created_at: String,
}

pub trait EmbeddingProvider: Send + Sync {
    fn name(&self) -> &str;
    fn compute(&self, sound: &SoundEntry, samples: &[f32]) -> Vec<f32>;
}

/// Compute a rich Sound DNA embedding from audio analysis.
/// Vector structure (64-dim):
///   [0..7]   Transient profile (attack sharpness, transient strength, clickiness)
///   [8..15]  Envelope profile (attack time, decay, tail, envelope shape)
///   [16..23] Spectral profile (centroid, rolloff, brightness, sub energy)
///   [24..31] Perceptual features (loudness, density, noisiness, tonalness)
///   [32..39] Temporal features (duration, transient count, onset regularity)
///   [40..47] Sound type one-hot encoding (10 types)
///   [48..55] Genre/style descriptor encoding
///   [56..63] Reserved / metadata
pub fn compute_mock_embedding(sound: &SoundEntry, samples: &[f32]) -> Vec<f32> {
    let mut vec = vec![0.0f32; EMBEDDING_DIM];

    if samples.len() < 256 {
        return fill_from_metadata(sound, &mut vec);
    }

    let analysis = audio::analyze::analyze_audio(samples, SAMPLE_RATE, 1);

    // [0..7] Transient profile
    vec[0] = analysis.transient_strength.clamp(0.0, 1.0);
    vec[1] = (analysis.transient_count as f32 / 20.0).min(1.0);
    vec[2] = if analysis.attack_ms > 0.0 { (1.0 / analysis.attack_ms * 10.0).min(1.0) } else { 0.0 };
    vec[3] = analysis.crest_factor / 20.0;
    vec[4] = analysis.zero_crossing_rate * 2.0;

    // [8..15] Envelope profile
    vec[8] = (analysis.attack_ms / 50.0).min(1.0);
    vec[9] = (analysis.decay_ms / 500.0).min(1.0);
    vec[10] = (analysis.tail_ms / 500.0).min(1.0);
    vec[11] = (analysis.duration_ms / 2000.0).min(1.0);

    // Envelope shape via spectral profile moments
    if !analysis.envelope.is_empty() {
        let env = &analysis.envelope;
        let peak_idx = env.iter()
            .enumerate()
            .map(|(i, &v)| (i, v))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        vec[12] = peak_idx as f32 / env.len().max(1) as f32;
        let tail_energy: f32 = env.iter().skip(env.len() * 3 / 4).sum::<f32>();
        let total_energy: f32 = env.iter().sum();
        vec[13] = if total_energy > 0.0 { tail_energy / total_energy } else { 0.0 };
    }

    // [16..23] Spectral profile
    vec[16] = (analysis.spectral_centroid / 8000.0).min(1.0);
    vec[17] = (analysis.spectral_rolloff / 12000.0).min(1.0);
    vec[18] = analysis.brightness;
    vec[19] = analysis.sub_energy_ratio;
    vec[20] = analysis.noise_estimate;

    // Spectral profile shape (balance across bands)
    if analysis.spectral_profile.len() >= 8 {
        let bands = 8;
        let bin_size = analysis.spectral_profile.len() / bands;
        for b in 0..bands.min(8) {
            if b * bin_size < analysis.spectral_profile.len() {
                let end = ((b + 1) * bin_size).min(analysis.spectral_profile.len());
                let val: f32 = analysis.spectral_profile[b * bin_size..end].iter().sum::<f32>()
                    / (end - b * bin_size).max(1) as f32;
                if b < 4 {
                    vec[21] += val * (1.0 - b as f32 * 0.25);
                }
            }
        }
    }

    // [24..31] Perceptual features
    vec[24] = analysis.rms * 5.0;
    vec[25] = analysis.loudness_lufs / -10.0 + 1.0;
    vec[26] = analysis.noise_estimate;
    vec[27] = if analysis.has_pitch { 0.5 + analysis.pitch_estimate.unwrap_or(0.0) / 8000.0 * 0.5 } else { 0.0 };
    vec[28] = analysis.zero_crossing_rate * 2.0;
    vec[29] = analysis.crest_factor / 20.0;
    vec[30] = if analysis.has_clipping { 0.8 } else { 0.0 };

    // [32..39] Temporal features
    vec[32] = (analysis.duration_ms / 5000.0).min(1.0);
    vec[33] = (analysis.transient_count as f32 / 15.0).min(1.0);
    vec[34] = if analysis.transient_count > 1 && analysis.duration_ms > 0.0 {
        let avg_gap = analysis.duration_ms / analysis.transient_count as f32;
        (avg_gap / 500.0).min(1.0)
    } else { 0.0 };

    // [40..47] Sound type encoding
    let st = SoundType::from_str(&analysis.sound_type_hint);
    let type_idx = match st {
        SoundType::Kick => 0, SoundType::Snare => 1, SoundType::ClosedHat => 2,
        SoundType::OpenHat => 3, SoundType::Clap => 4, SoundType::Tom => 5,
        SoundType::Perc => 6, SoundType::Bass => 7, SoundType::Fx => 8,
        SoundType::Other => 9,
    };
    if type_idx < 8 {
        vec[40 + type_idx] = 1.0;
    } else {
        vec[48] = 1.0; // other
    }

    // [56..63] Metadata / reserved
    vec[56] = if !samples.is_empty() { (samples.len() as f32 / SAMPLE_RATE as f32 * 1000.0) / 10000.0 } else { 0.0 };
    vec[57] = analysis.peak;
    vec[58] = analysis.sound_type_hint.chars().fold(0.0f32, |acc, c| acc + c as u8 as f32 * 0.01) % 1.0;

    normalize_in_place(&mut vec);
    vec
}

fn fill_from_metadata(sound: &SoundEntry, vec: &mut [f32]) -> Vec<f32> {
    vec[0] = sound.rms * 3.0;
    vec[8] = (sound.spectral_centroid / 8000.0).min(1.0);
    vec[16] = (sound.duration_ms / 5000.0).min(1.0);
    vec[24] = sound.peak;
    let st = SoundType::from_str(&sound.sound_type);
    let type_idx = match st {
        SoundType::Kick => 0, SoundType::Snare => 1, SoundType::ClosedHat => 2,
        SoundType::OpenHat => 3, SoundType::Clap => 4, SoundType::Tom => 5,
        SoundType::Perc => 6, SoundType::Bass => 7, _ => 8,
    };
    if type_idx < 8 {
        vec[40 + type_idx] = 1.0;
    }
    normalize_in_place(vec);
    vec.to_vec()
}

fn normalize_in_place(v: &mut [f32]) {
    let mag: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag > 0.0 {
        for x in v.iter_mut() {
            *x /= mag;
        }
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
    }
}

pub fn find_similar_by_embedding(
    target: &Embedding,
    all_embeddings: &[Embedding],
    max_results: usize,
) -> Vec<(String, f32)> {
    let mut scored: Vec<(String, f32)> = all_embeddings
        .iter()
        .filter(|e| e.sound_id != target.sound_id)
        .map(|e| {
            let sim = cosine_similarity(&target.vector, &e.vector);
            (e.sound_id.clone(), sim)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(max_results);
    scored
}

pub fn hybrid_similarity(
    target: &SoundEntry,
    candidates: &[SoundEntry],
    embeddings: &[Embedding],
    max_results: usize,
) -> Vec<(SoundEntry, f32, Vec<String>)> {
    let target_embedding = embeddings.iter().find(|e| e.sound_id == target.id);

    let mut scored: Vec<(SoundEntry, f32, Vec<String>)> = candidates
        .iter()
        .filter(|c| c.id != target.id)
        .map(|candidate| {
            let mut sim = crate::semantic_library::metadata_similarity(target, candidate);
            let mut reasons = vec![];

            if let Some(ref te) = target_embedding {
                if let Some(ce) = embeddings.iter().find(|e| e.sound_id == candidate.id) {
                    let vec_sim = cosine_similarity(&te.vector, &ce.vector);
                    if vec_sim > 0.4 {
                        reasons.push(format!("dna-similarity: {:.2}", vec_sim));
                        sim = sim * 0.3 + vec_sim * 0.7;
                    }
                }
            }

            if candidate.sound_type == target.sound_type {
                reasons.push(format!("same-type: {}", candidate.sound_type));
            }
            let dur_diff = (candidate.duration_ms - target.duration_ms).abs();
            if dur_diff < 200.0 {
                reasons.push("similar-duration".to_string());
            }

            (candidate.clone(), sim, reasons)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(max_results);
    scored
}
