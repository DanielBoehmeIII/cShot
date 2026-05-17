
use crate::db::SoundEntry;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SimilarSound {
    pub entry: SoundEntry,
    pub similarity_score: f32,
    pub match_reasons: Vec<String>,
}

pub fn metadata_similarity(a: &SoundEntry, b: &SoundEntry) -> f32 {
    let mut score = 0.0f32;
    let mut total_weight = 0.0f32;

    let type_weight = 3.0;
    if a.sound_type == b.sound_type {
        score += type_weight;
    }
    total_weight += type_weight;

    let duration_diff = (a.duration_ms - b.duration_ms).abs();
    let duration_sim = 1.0 - (duration_diff / 2000.0).clamp(0.0, 1.0);
    score += duration_sim * 2.0;
    total_weight += 2.0;

    if a.rms > 0.0 && b.rms > 0.0 {
        let rms_diff = (a.rms - b.rms).abs();
        let rms_sim = 1.0 - (rms_diff / 0.5).clamp(0.0, 1.0);
        score += rms_sim * 1.5;
        total_weight += 1.5;
    }

    if a.spectral_centroid > 0.0 && b.spectral_centroid > 0.0 {
        let centroid_diff = (a.spectral_centroid - b.spectral_centroid).abs();
        let centroid_sim = 1.0 - (centroid_diff / 5000.0).clamp(0.0, 1.0);
        score += centroid_sim * 2.0;
        total_weight += 2.0;
    }

    if a.peak > 0.0 && b.peak > 0.0 {
        let peak_diff = (a.peak - b.peak).abs();
        let peak_sim = 1.0 - (peak_diff / 0.5).clamp(0.0, 1.0);
        score += peak_sim * 0.5;
        total_weight += 0.5;
    }

    if total_weight > 0.0 {
        score / total_weight
    } else {
        0.0
    }
}

pub fn find_similar_sounds(
    target: &SoundEntry,
    candidates: &[SoundEntry],
    max_results: usize,
) -> Vec<SimilarSound> {
    let mut scored: Vec<SimilarSound> = candidates
        .iter()
        .filter(|c| c.id != target.id)
        .map(|candidate| {
            let score = metadata_similarity(target, candidate);

            let mut reasons = Vec::new();
            if candidate.sound_type == target.sound_type {
                reasons.push(format!("same type: {}", candidate.sound_type));
            }
            let duration_diff = (candidate.duration_ms - target.duration_ms).abs();
            if duration_diff < 200.0 {
                reasons.push("similar duration".to_string());
            }
            if candidate.rms > 0.0 && target.rms > 0.0 {
                let rms_diff = (candidate.rms - target.rms).abs();
                if rms_diff < 0.1 {
                    reasons.push("similar loudness".to_string());
                }
            }
            if candidate.spectral_centroid > 0.0 && target.spectral_centroid > 0.0 {
                let c_diff = (candidate.spectral_centroid - target.spectral_centroid).abs();
                if c_diff < 500.0 {
                    reasons.push("similar brightness".to_string());
                }
            }

            SimilarSound {
                entry: candidate.clone(),
                similarity_score: score,
                match_reasons: reasons,
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    scored.truncate(max_results);
    scored
}

pub fn extract_tags_list(tags_json: &str) -> Vec<String> {
    serde_json::from_str(tags_json).unwrap_or_else(|_| {
        tags_json.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
    })
}

pub fn match_descriptor(entry: &SoundEntry, descriptor: &str) -> bool {
    let tags = extract_tags_list(&entry.tags);
    tags.contains(&descriptor.to_string())
}

pub fn filter_by_descriptors(entries: &[SoundEntry], include: &[String]) -> Vec<SoundEntry> {
    if include.is_empty() {
        return entries.to_vec();
    }
    entries
        .iter()
        .filter(|e| {
            let tags = extract_tags_list(&e.tags);
            include.iter().all(|d| tags.contains(d))
        })
        .cloned()
        .collect()
}

pub fn available_descriptors(entries: &[SoundEntry]) -> Vec<(String, usize)> {
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for entry in entries {
        let tags = extract_tags_list(&entry.tags);
        for tag in tags {
            *counts.entry(tag).or_insert(0) += 1;
        }
    }
    let mut result: Vec<(String, usize)> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

// ─── Enhanced Semantic Search ───────────────────────────

pub fn parse_natural_language_query(query: &str) -> SemanticQuery {
    let lower = query.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    let mut target_type: Option<String> = None;
    let mut target_descriptors: Vec<String> = Vec::new();
    let mut target_genre: Option<String> = None;
    let mut bpm: Option<f32> = None;
    let mut duration_range: Option<(f32, f32)> = None;

    let type_keywords = [
        ("kick", "kick"), ("snare", "snare"), ("hat", "closed_hat"), ("hi-hat", "closed_hat"),
        ("hihat", "closed_hat"), ("open hat", "open_hat"), ("clap", "clap"), ("tom", "tom"),
        ("perc", "perc"), ("bass", "bass"), ("fx", "fx"), ("crash", "fx"), ("rim", "perc"),
        ("impact", "fx"), ("sweep", "fx"), ("whoosh", "fx"), ("riser", "fx"),
    ];

    let genre_keywords = [
        "trap", "drill", "house", "techno", "lo-fi", "lofi", "cinematic", "ambient",
        "dubstep", "phonk", "hyperpop", "garage", "industrial", "synthwave",
    ];

    for w in &words {
        if let Some(bpm_str) = w.strip_suffix("bpm").or_else(|| w.strip_suffix("BPM")) {
            if let Ok(v) = bpm_str.parse::<f32>() {
                if (60.0..=200.0).contains(&v) { bpm = Some(v); }
            }
        }
        for &(kw, st) in &type_keywords {
            if *w == kw { target_type = Some(st.to_string()); }
        }
        if genre_keywords.contains(w) {
            target_genre = Some(w.to_string());
        }
    }

    let descriptor_terms = [
        "punchy", "punch", "hard", "soft", "gentle", "aggressive", "crack", "snap", "click",
        "bright", "crisp", "dark", "warm", "dull", "muffled", "airy", "metallic", "thin",
        "distorted", "crunchy", "gritty", "clean", "saturated", "noisy", "noise",
        "short", "long", "tight", "ring", "deep", "sub", "subby", "boomy", "low",
        "fat", "thick", "heavy", "dense", "full", "sparse",
        "cinematic", "epic", "massive", "wide", "mono", "spacious",
        "analog", "vintage", "digital", "tape", "raw", "smooth",
        "glossy", "harsh", "sizzle", "round", "woody", "hollow", "tinny",
    ];

    for w in &words {
        if descriptor_terms.contains(w) {
            if !target_descriptors.contains(&w.to_string()) {
                target_descriptors.push(w.to_string());
            }
        }
    }

    if lower.contains("short") {
        duration_range = Some((0.0, 200.0));
    } else if lower.contains("long") {
        duration_range = Some((500.0, 10000.0));
    }

    SemanticQuery {
        target_type,
        target_descriptors,
        target_genre,
        bpm,
        duration_range,
        raw_query: query.to_string(),
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SemanticQuery {
    pub target_type: Option<String>,
    pub target_descriptors: Vec<String>,
    pub target_genre: Option<String>,
    pub bpm: Option<f32>,
    pub duration_range: Option<(f32, f32)>,
    pub raw_query: String,
}

pub fn search_by_semantic_query(
    entries: &[SoundEntry],
    query: &SemanticQuery,
    max_results: usize,
) -> Vec<SimilarSound> {
    let mut scored: Vec<SimilarSound> = entries
        .iter()
        .map(|entry| {
            let mut score = 0.0f32;
            let mut reasons = Vec::new();

            if let Some(ref target_type) = query.target_type {
                if entry.sound_type == *target_type || entry.tags.contains(target_type) {
                    score += 3.0;
                    reasons.push(format!("type: {}", target_type));
                }
            }

            if let Some(ref genre) = query.target_genre {
                let tags = extract_tags_list(&entry.tags);
                if tags.contains(genre) || entry.prompt.to_lowercase().contains(genre) {
                    score += 2.0;
                    reasons.push(format!("genre: {}", genre));
                }
            }

            let tags = extract_tags_list(&entry.tags);
            let prompt_lower = entry.prompt.to_lowercase();
            for desc in &query.target_descriptors {
                if tags.contains(desc) || prompt_lower.contains(desc) {
                    score += 1.0;
                    if !reasons.iter().any(|r| r.contains(desc)) {
                        reasons.push(format!("matches: {}", desc));
                    }
                }
            }

            if let Some((min_dur, max_dur)) = query.duration_range {
                if entry.duration_ms >= min_dur && entry.duration_ms <= max_dur {
                    score += 1.0;
                    reasons.push("duration match".to_string());
                }
            }

            SimilarSound {
                entry: entry.clone(),
                similarity_score: score,
                match_reasons: reasons,
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    scored.truncate(max_results);
    scored
}
