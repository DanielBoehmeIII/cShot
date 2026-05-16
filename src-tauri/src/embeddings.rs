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

pub fn compute_mock_embedding(sound: &SoundEntry, samples: &[f32]) -> Vec<f32> {
        let mut vec = Vec::with_capacity(EMBEDDING_DIM);
        let rms = sound.rms;
        let centroid = sound.spectral_centroid;
        let duration = sound.duration_ms;
        let peak = sound.peak;

        for i in 0..EMBEDDING_DIM {
            let idx = i as f32;
            let val = (rms * (idx * 0.5).sin()
                + (centroid / 1000.0) * (idx * 0.3).cos()
                + (duration / 100.0) * (idx * 0.7).sin()
                + peak * (idx * 0.2).cos())
                * 0.25;
            vec.push(val);
        }

        if !samples.is_empty() {
            let sample_val = samples.iter().take(64).map(|&s| s.abs()).fold(0.0f32, f32::max);
            vec[0] += sample_val * 0.1;
            vec[EMBEDDING_DIM - 1] += (samples.len() as f32 / 44100.0 * 1000.0) / 10000.0;
        }

        normalize_in_place(&mut vec);
        vec
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
                    if vec_sim > 0.5 {
                        reasons.push(format!("embedding-similarity: {:.2}", vec_sim));
                        sim = sim * 0.4 + vec_sim * 0.6;
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
