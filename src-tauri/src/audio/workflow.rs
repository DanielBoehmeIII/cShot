use super::analyze::AudioAnalysis;
use super::audio_intelligence::{self};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AutoTags {
    pub sound_type: String,
    pub descriptors: Vec<String>,
    pub genre_hints: Vec<String>,
    pub mix_role: String,
    pub energy_level: String,
    pub duration_label: String,
}

pub fn infer_tags(samples: &[f32], analysis: &AudioAnalysis) -> AutoTags {
    let intelligence = audio_intelligence::analyze_intelligence(samples, analysis);
    let mut descriptors = Vec::new();

    if analysis.transient_strength > 0.5 { descriptors.push("punchy".to_string()); }
    if analysis.transient_count > 3 { descriptors.push("layered".to_string()); }
    if analysis.brightness > 0.6 { descriptors.push("bright".to_string()); }
    if analysis.brightness < 0.3 { descriptors.push("dark".to_string()); }
    if analysis.noise_estimate > 0.6 { descriptors.push("noisy".to_string()); }
    if analysis.noise_estimate < 0.2 { descriptors.push("clean".to_string()); }
    if analysis.sub_energy_ratio > 0.3 { descriptors.push("subby".to_string()); }
    if analysis.crest_factor > 8.0 { descriptors.push("impactful".to_string()); }
    if analysis.spectral_centroid > 4000.0 { descriptors.push("bright".to_string()); }
    if analysis.spectral_centroid < 500.0 { descriptors.push("dark".to_string()); }
    if analysis.rms > 0.3 { descriptors.push("loud".to_string()); }
    if analysis.rms < 0.05 && analysis.peak > 0.1 { descriptors.push("dynamic".to_string()); }
    if analysis.duration_ms < 100.0 { descriptors.push("short".to_string()); }
    if analysis.duration_ms > 1000.0 { descriptors.push("long".to_string()); }
    if analysis.has_clipping { descriptors.push("clipped".to_string()); }

    let energy_level = if analysis.rms > 0.3 || analysis.peak > 0.9 {
        "high".to_string()
    } else if analysis.rms > 0.1 {
        "medium".to_string()
    } else {
        "low".to_string()
    };

    let duration_label = if analysis.duration_ms < 100.0 {
        "short".to_string()
    } else if analysis.duration_ms < 500.0 {
        "medium".to_string()
    } else {
        "long".to_string()
    };

    let mix_role = format!("{:?}", intelligence.mix_role);
    let genre_hints = Vec::new();

    AutoTags {
        sound_type: analysis.sound_type_hint.clone(),
        descriptors,
        genre_hints,
        mix_role,
        energy_level,
        duration_label,
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PackSuggestion {
    pub title: String,
    pub sound_count: usize,
    pub has_kick: bool,
    pub has_snare: bool,
    pub has_hat: bool,
    pub has_clap: bool,
    pub has_perc: bool,
    pub has_bass: bool,
    pub has_fx: bool,
    pub missing_roles: Vec<String>,
    pub cohesion_score: f32,
}

pub fn suggest_pack(sounds: &[AudioAnalysis]) -> PackSuggestion {
    let mut has_kick = false;
    let mut has_snare = false;
    let mut has_hat = false;
    let mut has_clap = false;
    let mut has_perc = false;
    let mut has_bass = false;
    let mut has_fx = false;

    for a in sounds {
        match a.sound_type_hint.as_str() {
            "kick" => has_kick = true,
            "snare" => has_snare = true,
            "closed_hat" | "open_hat" => has_hat = true,
            "clap" => has_clap = true,
            "perc" => has_perc = true,
            "bass" => has_bass = true,
            "fx" => has_fx = true,
            _ => {}
        }
    }

    let mut missing = Vec::new();
    if !has_kick { missing.push("kick".to_string()); }
    if !has_snare { missing.push("snare".to_string()); }
    if !has_hat { missing.push("hi-hat".to_string()); }
    if !has_perc && !has_clap { missing.push("perc/clap".to_string()); }
    if !has_bass { missing.push("bass/sub".to_string()); }

    let total_roles = [has_kick, has_snare, has_hat, has_clap, has_perc, has_bass, has_fx]
        .iter().filter(|&&b| b).count();
    let cohesion = if sounds.len() > 1 {
        let avg_duration: f32 = sounds.iter().map(|a| a.duration_ms).sum::<f32>() / sounds.len() as f32;
        let dur_variance: f32 = sounds.iter().map(|a| (a.duration_ms - avg_duration).abs()).sum::<f32>() / sounds.len() as f32;
        let duration_cohesion = 1.0 - (dur_variance / avg_duration.max(1.0)).min(1.0) * 0.3;
        let type_variety = (total_roles as f32 / 7.0).min(1.0) * 0.4;
        let count_bonus = (sounds.len() as f32 / 16.0).min(1.0) * 0.3;
        (duration_cohesion + type_variety + count_bonus).min(1.0)
    } else {
        0.5
    };

    let title = format!("{} sounds pack", sounds.len());

    PackSuggestion {
        title,
        sound_count: sounds.len(),
        has_kick,
        has_snare,
        has_hat,
        has_clap,
        has_perc,
        has_bass,
        has_fx,
        missing_roles: missing,
        cohesion_score: cohesion,
    }
}

pub fn suggest_filename(analysis: &AudioAnalysis, tags: &AutoTags) -> String {
    let st = &tags.sound_type;
    let desc = tags.descriptors.first().map(|d| d.as_str()).unwrap_or("sound");
    let energy = &tags.energy_level;
    format!("{}_{}_{}_{}ms", st, desc, energy, analysis.duration_ms as u32)
}
