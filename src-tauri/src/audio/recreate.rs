use super::analyze::{analyze_audio, AudioAnalysis};
use super::resynthesize::{self, ResynthesisParams};
use super::{SoundType, SAMPLE_RATE};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ApproximationResult {
    pub id: String,
    pub samples: Vec<f32>,
    pub similarity: SimilarityReport,
    pub params: String,
    pub seed: u64,
    pub strategy: RecreationStrategy,
    pub confidence: f32,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SimilarityReport {
    pub overall: f32,
    pub envelope_match: f32,
    pub spectral_match: f32,
    pub rms_match: f32,
    pub transient_match: f32,
    pub duration_match: f32,
    pub noise_match: f32,
    pub sub_match: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum RecreationStrategy {
    Closest,
    Cleaner,
    Punchier,
    Darker,
    Brighter,
    GenreAdapted(String),
    Mutated,
    Hybridized,
}

impl RecreationStrategy {
    pub fn label(&self) -> &str {
        match self {
            RecreationStrategy::Closest => "closest",
            RecreationStrategy::Cleaner => "cleaner",
            RecreationStrategy::Punchier => "punchier",
            RecreationStrategy::Darker => "darker",
            RecreationStrategy::Brighter => "brighter",
            RecreationStrategy::GenreAdapted(g) => g,
            RecreationStrategy::Mutated => "mutated",
            RecreationStrategy::Hybridized => "hybridized",
        }
    }
}

// ─── Sound Structure Extraction ───────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SoundStructure {
    pub attack_ms: f32,
    pub body_sustain_ms: f32,
    pub decay_ms: f32,
    pub tail_ms: f32,
    pub transient_sharpness: f32,
    pub noise_content: f32,
    pub tonal_content: f32,
    pub sub_energy: f32,
    pub spectral_centroid: f32,
    pub has_pitch: bool,
    pub pitch_hz: Option<f32>,
    pub is_percussive: bool,
    pub is_tonal: bool,
    pub is_noise_dominated: bool,
}

pub fn extract_sound_structure(_samples: &[f32], analysis: &AudioAnalysis) -> SoundStructure {
    SoundStructure {
        attack_ms: analysis.attack_ms,
        body_sustain_ms: analysis.decay_ms.max(analysis.duration_ms * 0.3),
        decay_ms: analysis.decay_ms,
        tail_ms: analysis.tail_ms,
        transient_sharpness: analysis.transient_strength,
        noise_content: analysis.noise_estimate,
        tonal_content: 1.0 - analysis.noise_estimate,
        sub_energy: analysis.sub_energy_ratio,
        spectral_centroid: analysis.spectral_centroid,
        has_pitch: analysis.has_pitch,
        pitch_hz: analysis.pitch_estimate,
        is_percussive: analysis.transient_count > 0 && analysis.attack_ms < 20.0,
        is_tonal: analysis.has_pitch && analysis.noise_estimate < 0.5,
        is_noise_dominated: analysis.noise_estimate > 0.7,
    }
}

// ─── Synthesis Strategy Inference ─────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SynthesisStrategy {
    pub sound_type: SoundType,
    pub method: SynthesisMethod,
    pub layers: Vec<String>,
    pub pitch_hz: Option<f32>,
    pub key_features: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum SynthesisMethod {
    LayeredResynthesis,
    FmSynthesis,
    Subtractive,
    NoiseBased,
    Hybrid,
}

pub fn infer_synthesis_strategy(structure: &SoundStructure, analysis: &AudioAnalysis) -> SynthesisStrategy {
    let sound_type = SoundType::from_str(&analysis.sound_type_hint);
    let method = match sound_type {
        SoundType::Kick => if structure.sub_energy > 0.4 {
            SynthesisMethod::LayeredResynthesis
        } else if structure.is_noise_dominated {
            SynthesisMethod::Hybrid
        } else {
            SynthesisMethod::LayeredResynthesis
        },
        SoundType::Snare | SoundType::Clap => if analysis.transient_count > 2 {
            SynthesisMethod::Hybrid
        } else if structure.noise_content > 0.6 {
            SynthesisMethod::NoiseBased
        } else {
            SynthesisMethod::LayeredResynthesis
        },
        SoundType::ClosedHat | SoundType::OpenHat => SynthesisMethod::NoiseBased,
        SoundType::Bass => if structure.is_tonal && structure.sub_energy > 0.3 {
            SynthesisMethod::Subtractive
        } else {
            SynthesisMethod::LayeredResynthesis
        },
        SoundType::Tom => SynthesisMethod::LayeredResynthesis,
        SoundType::Perc => if structure.is_noise_dominated {
            SynthesisMethod::NoiseBased
        } else {
            SynthesisMethod::Hybrid
        },
        SoundType::Fx => SynthesisMethod::Hybrid,
        SoundType::Other => if structure.is_noise_dominated && !structure.is_tonal {
            if structure.transient_sharpness > 2.0 {
                SynthesisMethod::NoiseBased
            } else {
                SynthesisMethod::Subtractive
            }
        } else if analysis.transient_count > 3 {
            SynthesisMethod::Hybrid
        } else if structure.sub_energy > 0.3 {
            SynthesisMethod::LayeredResynthesis
        } else if structure.is_tonal && structure.transient_sharpness < 1.5 {
            SynthesisMethod::FmSynthesis
        } else {
            SynthesisMethod::LayeredResynthesis
        },
    };

    let mut layers = Vec::new();
    match sound_type {
        SoundType::Kick => {
            if structure.transient_sharpness > 0.5 { layers.push("transient".to_string()); }
            if structure.tonal_content > 0.1 { layers.push("body".to_string()); }
            if structure.sub_energy > 0.1 { layers.push("sub".to_string()); }
            if structure.tail_ms > 30.0 { layers.push("tail".to_string()); }
        }
        SoundType::Snare => {
            if structure.transient_sharpness > 0.5 { layers.push("transient".to_string()); }
            layers.push("body".to_string());
            layers.push("noise".to_string());
            if structure.tail_ms > 30.0 { layers.push("tail".to_string()); }
        }
        SoundType::ClosedHat | SoundType::OpenHat => {
            layers.push("transient".to_string());
            layers.push("noise".to_string());
        }
        SoundType::Clap => {
            layers.push("body".to_string());
            layers.push("noise".to_string());
            if structure.tail_ms > 30.0 { layers.push("tail".to_string()); }
        }
        SoundType::Bass => {
            layers.push("body".to_string());
            if structure.sub_energy > 0.1 { layers.push("sub".to_string()); }
        }
        _ => {
            if structure.transient_sharpness > 0.5 { layers.push("transient".to_string()); }
            if structure.tonal_content > 0.2 { layers.push("body".to_string()); }
            if structure.noise_content > 0.2 { layers.push("noise".to_string()); }
            if structure.sub_energy > 0.15 { layers.push("sub".to_string()); }
            if structure.tail_ms > 50.0 { layers.push("tail".to_string()); }
        }
    }

    let mut key_features = Vec::new();
    if structure.is_percussive { key_features.push("percussive".to_string()); }
    if structure.is_tonal { key_features.push("tonal".to_string()); }
    if structure.sub_energy > 0.3 { key_features.push("sub-heavy".to_string()); }
    if structure.spectral_centroid > 4000.0 { key_features.push("bright".to_string()); }
    if structure.spectral_centroid < 800.0 { key_features.push("dark".to_string()); }
    if analysis.crest_factor > 12.0 { key_features.push("punchy".to_string()); }
    if analysis.noise_estimate > 0.7 { key_features.push("noisy".to_string()); }
    if analysis.brightness > 0.75 { key_features.push("bright".to_string()); }
    if analysis.brightness < 0.25 { key_features.push("dark".to_string()); }

    SynthesisStrategy {
        sound_type,
        method,
        layers,
        pitch_hz: analysis.pitch_estimate,
        key_features,
    }
}

// ─── Confidence Scoring ───────────────────────────────────

pub fn compute_recreation_confidence(structure: &SoundStructure, analysis: &AudioAnalysis) -> f32 {
    let mut score: f32 = 0.8;

    if analysis.is_silent { score -= 0.5; }
    if analysis.duration_ms < 10.0 { score -= 0.2; }
    if analysis.duration_ms > 10000.0 { score -= 0.1; }
    if analysis.has_clipping { score -= 0.1; }

    if structure.spectral_centroid > 100.0 && structure.spectral_centroid < 12000.0 {
        score += 0.05;
    } else {
        score -= 0.1;
    }

    if structure.transient_sharpness > 0.5 && structure.transient_sharpness < 20.0 {
        score += 0.05;
    }

    if analysis.noise_estimate > 0.95 { score -= 0.15; }

    score.clamp(0.0, 1.0)
}

// ─── Perceptual Band Weights ─────────────────────────────

pub fn get_perceptual_band_weights(sound_type: &SoundType) -> (f32, f32, f32, f32, f32) {
    match sound_type {
        SoundType::Kick => (0.40, 0.15, 0.25, 0.20, 0.00),
        SoundType::Snare => (0.25, 0.20, 0.30, 0.15, 0.10),
        SoundType::ClosedHat => (0.15, 0.10, 0.20, 0.05, 0.50),
        SoundType::OpenHat => (0.10, 0.10, 0.25, 0.05, 0.50),
        SoundType::Clap => (0.15, 0.15, 0.40, 0.10, 0.20),
        SoundType::Bass => (0.30, 0.25, 0.15, 0.30, 0.00),
        SoundType::Perc => (0.30, 0.20, 0.25, 0.10, 0.15),
        SoundType::Tom => (0.25, 0.25, 0.20, 0.20, 0.10),
        SoundType::Fx => (0.10, 0.15, 0.20, 0.15, 0.40),
        SoundType::Other => (0.25, 0.20, 0.25, 0.15, 0.15),
    }
}

// ─── Multi-band Similarity ────────────────────────────────

pub fn compute_multiband_similarity(original: &[f32], recreated: &[f32]) -> f32 {
    let orig_env = super::analyze::extract_envelope(original, 128);
    let recre_env = super::analyze::extract_envelope(recreated, 128);
    let len = orig_env.len().min(recre_env.len());
    if len < 4 { return 0.5; }

    let attack_end = (len / 6).max(2);
    let body_end = (len * 2 / 3).max(attack_end + 1);
    let tail_start = body_end;

    let attack_a = &orig_env[..attack_end];
    let attack_b = &recre_env[..attack_end];
    let body_a = &orig_env[attack_end..body_end];
    let body_b = &recre_env[attack_end..body_end];
    let tail_a = &orig_env[tail_start..];
    let tail_b = &recre_env[tail_start..];

    let attack_corr = envelope_similarity(attack_a, attack_b);
    let body_corr = envelope_similarity(body_a, body_b);
    let tail_corr = if tail_a.len() > 1 && tail_b.len() > 1 {
        envelope_similarity(tail_a, tail_b)
    } else {
        0.5
    };

    attack_corr * 0.35 + body_corr * 0.35 + tail_corr * 0.30
}

pub fn compute_multiband_similarity_perceptual(original: &[f32], recreated: &[f32], sound_type: &SoundType) -> f32 {
    let (aw, bw, tw, nw, sw) = get_perceptual_band_weights(sound_type);
    let total = aw + bw + tw + nw + sw;
    let (aw, bw, tw, nw, sw) = (aw/total, bw/total, tw/total, nw/total, sw/total);
    
    let orig_env = super::analyze::extract_envelope(original, 128);
    let recre_env = super::analyze::extract_envelope(recreated, 128);
    let len = orig_env.len().min(recre_env.len());
    if len < 4 { return 0.5; }

    let attack_end = (len / 6).max(2);
    let body_end = (len * 2 / 3).max(attack_end + 1);
    let tail_start = body_end;

    let attack_a = &orig_env[..attack_end];
    let attack_b = &recre_env[..attack_end];
    let body_a = &orig_env[attack_end..body_end];
    let body_b = &recre_env[attack_end..body_end];
    let tail_a = &orig_env[tail_start..];
    let tail_b = &recre_env[tail_start..];

    let attack_corr = envelope_similarity(attack_a, attack_b);
    let body_corr = envelope_similarity(body_a, body_b);
    let tail_corr = if tail_a.len() > 1 && tail_b.len() > 1 {
        envelope_similarity(tail_a, tail_b)
    } else {
        0.5
    };

    attack_corr * aw + body_corr * bw + tail_corr * tw + tail_corr * nw + body_corr * sw
}

pub fn compute_multiband_spectral_match(original: &[f32], recreated: &[f32]) -> f32 {
    let orig_profile = super::analyze::compute_spectral_profile(original, 32);
    let recre_profile = super::analyze::compute_spectral_profile(recreated, 32);
    let len = orig_profile.len().min(recre_profile.len());
    if len < 3 { return 0.5; }

    let low_end = (len / 3).max(1);
    let mid_end = (len * 2 / 3).max(low_end + 1);

    let low_orig: f32 = orig_profile[..low_end].iter().sum();
    let low_recre: f32 = recre_profile[..low_end].iter().sum();
    let mid_orig: f32 = orig_profile[low_end..mid_end].iter().sum();
    let mid_recre: f32 = recre_profile[low_end..mid_end].iter().sum();
    let high_orig: f32 = orig_profile[mid_end..].iter().sum();
    let high_recre: f32 = recre_profile[mid_end..].iter().sum();

    let total_orig = low_orig + mid_orig + high_orig + 1e-10;
    let total_recre = low_recre + mid_recre + high_recre + 1e-10;

    let low_ratio = (low_orig / total_orig).min(low_recre / total_recre)
        / (low_orig / total_orig).max(low_recre / total_recre).max(1e-6);
    let mid_ratio = (mid_orig / total_orig).min(mid_recre / total_recre)
        / (mid_orig / total_orig).max(mid_recre / total_recre).max(1e-6);
    let high_ratio = (high_orig / total_orig).min(high_recre / total_recre)
        / (high_orig / total_orig).max(high_recre / total_recre).max(1e-6);

    let band_match = low_ratio * 0.4 + mid_ratio * 0.35 + high_ratio * 0.25;
    band_match.clamp(0.0, 1.0)
}

pub fn compute_similarity(original: &[f32], recreated: &[f32], original_analysis: &AudioAnalysis) -> SimilarityReport {
    let st = SoundType::from_str(&original_analysis.sound_type_hint);
    let env_match = compute_multiband_similarity(original, recreated);
    let env_perceptual = compute_multiband_similarity_perceptual(original, recreated, &st);
    let spectral_match = compute_multiband_spectral_match(original, recreated);
    let recre_analysis = analyze_audio(recreated, SAMPLE_RATE, 1);

    let rms_orig = original_analysis.rms.max(0.001);
    let rms_recre = recre_analysis.rms.max(0.001);
    let rms_min = rms_orig.min(rms_recre);
    let rms_max = rms_orig.max(rms_recre);
    let rms_match = if rms_max > 0.0 { rms_min / rms_max } else { 0.5 };

    let trans_orig = original_analysis.transient_strength;
    let trans_recre = recre_analysis.transient_strength;
    let trans_min = trans_orig.min(trans_recre);
    let trans_max = trans_orig.max(trans_recre);
    let transient_match = if trans_max > 0.1 { trans_min / trans_max } else { 1.0 - (trans_orig - trans_recre).abs().min(1.0) };

    let dur_orig = original_analysis.duration_ms.max(1.0);
    let dur_recre = recre_analysis.duration_ms.max(1.0);
    let dur_ratio = (dur_orig / dur_recre).min(dur_recre / dur_orig);
    let duration_match = dur_ratio.clamp(0.0, 1.0);

    let noise_match = 1.0 - (original_analysis.noise_estimate - recre_analysis.noise_estimate).abs().min(1.0);
    let sub_match = 1.0 - (original_analysis.sub_energy_ratio - recre_analysis.sub_energy_ratio).abs().min(1.0);

    let crest_orig = original_analysis.crest_factor;
    let crest_recre = recre_analysis.crest_factor;
    let crest_min = crest_orig.min(crest_recre);
    let crest_max = crest_orig.max(crest_recre);
    let punch_match = if crest_max > 0.1 { crest_min / crest_max } else { 0.5 };

    let overall = env_match * 0.10 + env_perceptual * 0.18 + spectral_match * 0.20 + rms_match * 0.10
        + transient_match * 0.15 + duration_match * 0.08 + noise_match * 0.06 + sub_match * 0.08 + punch_match * 0.05;

    SimilarityReport {
        overall: overall.clamp(0.0, 1.0),
        envelope_match: env_match.max(env_perceptual),
        spectral_match,
        rms_match,
        transient_match,
        duration_match,
        noise_match,
        sub_match,
    }
}

pub fn envelope_similarity(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    if len < 2 { return 0.5; }
    let mut corr = 0.0f32;
    let mut a_energy = 0.0f32;
    let mut b_energy = 0.0f32;
    for i in 0..len {
        corr += a[i] * b[i];
        a_energy += a[i] * a[i];
        b_energy += b[i] * b[i];
    }
    let denom = (a_energy * b_energy).sqrt().max(1e-10);
    (corr / denom).clamp(0.0, 1.0)
}

pub fn extract_transient_profile(samples: &[f32]) -> (f32, f32, f32) {
    let peak_idx = samples.iter()
        .enumerate()
        .map(|(i, &s)| (i, s.abs()))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let onset_len = (SAMPLE_RATE as f32 * 0.015) as usize;
    let onset_start = peak_idx.saturating_sub(onset_len / 2);
    let onset_end = (onset_start + onset_len).min(samples.len());

    if onset_end <= onset_start { return (0.0, 0.0, 0.0); }

    let onset_region = &samples[onset_start..onset_end];
    let peak_val = onset_region.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);

    if peak_val < 0.001 { return (0.0, 0.0, 0.0); }

    let mut attack_samples = 0;
    let threshold = peak_val * 0.1;
    for i in 0..onset_region.len() {
        if onset_region[i].abs() >= threshold {
            attack_samples = i;
            break;
        }
    }

    let spectral_spread = if onset_region.len() > 64 {
        let centroid = super::analyze::compute_spectral_centroid(onset_region);
        centroid / 8000.0
    } else {
        0.5
    };

    let sharpness = if attack_samples > 0 {
        (peak_val / attack_samples as f32).min(1.0)
    } else {
        0.5
    };

    (sharpness, spectral_spread, peak_val)
}

pub fn extract_target_envelope(samples: &[f32], num_points: usize) -> Vec<f32> {
    let env = super::analyze::extract_envelope(samples, num_points);
    if env.is_empty() { return env; }
    let peak = env.iter().copied().fold(0.0f32, f32::max);
    if peak > 0.0 {
        env.iter().map(|&v| v / peak).collect()
    } else {
        env
    }
}

pub fn params_from_analysis(analysis: &AudioAnalysis, samples: &[f32]) -> ResynthesisParams {
    let st = SoundType::from_str(&analysis.sound_type_hint);
    let pitch = analysis.pitch_estimate.unwrap_or(match st {
        SoundType::Kick | SoundType::Bass => 60.0,
        SoundType::Snare => 200.0,
        SoundType::ClosedHat => 400.0,
        SoundType::OpenHat => 300.0,
        SoundType::Clap => 180.0,
        SoundType::Tom => 120.0,
        SoundType::Perc => 300.0,
        _ => 200.0,
    });

    let base = resynthesize::params_for_sound_type(st, pitch, analysis.duration_ms);
    let mut p = base.clone();

    p.attack_ms = analysis.attack_ms.clamp(0.1, 50.0);
    p.decay_ms = analysis.decay_ms.clamp(5.0, 2000.0);
    p.tail_ms = analysis.tail_ms.clamp(0.0, 2000.0);
    p.brightness = analysis.brightness;
    p.noise_amount = analysis.noise_estimate.clamp(0.0, 1.0);
    p.sub_gain = (analysis.sub_energy_ratio * 2.0).clamp(0.0, 1.0);
    p.body_gain = (analysis.rms * 5.0).clamp(0.1, 1.0);

    let (sharpness, _spectral_spread, _peak_val) = extract_transient_profile(samples);
    p.click_amount = (sharpness * 0.8).clamp(0.0, 1.0);

    if analysis.transient_count > 1 {
        p.noise_amount = (p.noise_amount + 0.15).min(1.0);
    }
    if analysis.transient_strength > 3.0 {
        p.saturation_drive = (base.saturation_drive * 1.2).max(1.0);
    }

    let crest = analysis.crest_factor;
    if crest > 12.0 {
        p.saturation_drive = (p.saturation_drive * 1.15).max(1.0);
        p.click_amount = (p.click_amount * 1.2).min(1.0);
    } else if crest < 4.0 {
        p.saturation_drive = (p.saturation_drive * 0.85).max(1.0);
        p.click_amount = (p.click_amount * 0.8).min(1.0);
    }

    match st {
        SoundType::Kick => {
            p.pitch_hz = pitch.clamp(40.0, 150.0);
            if analysis.sub_energy_ratio > 0.3 { p.sub_gain = (p.sub_gain * 1.3).min(1.0); }
            if analysis.transient_strength < 2.0 { p.click_amount = (p.click_amount * 1.5).min(1.0); }
            p.pitch_drop_ratio = 0.7;
        }
        SoundType::Snare => {
            p.pitch_hz = pitch.clamp(150.0, 300.0);
            if analysis.noise_estimate > 0.6 { p.noise_amount = (p.noise_amount * 1.2).min(1.0); }
            p.pitch_drop_ratio = 0.15;
        }
        SoundType::ClosedHat => {
            p.pitch_hz = pitch.clamp(300.0, 1000.0);
            p.noise_hp_hz = analysis.spectral_centroid.max(4000.0);
            p.noise_amount = 1.0;
            p.brightness = analysis.brightness.max(0.6);
        }
        SoundType::OpenHat => {
            p.pitch_hz = pitch.clamp(200.0, 600.0);
            p.noise_hp_hz = (analysis.spectral_centroid * 0.8).max(2000.0);
            p.noise_amount = 1.0;
        }
        SoundType::Bass => {
            p.pitch_hz = pitch.clamp(30.0, 120.0);
            if analysis.sub_energy_ratio > 0.4 { p.sub_gain = (p.sub_gain * 1.4).min(1.0); }
            p.body_gain = p.body_gain.max(0.5);
            p.pitch_drop_ratio = 0.3;
        }
        SoundType::Clap => {
            p.pitch_hz = pitch.clamp(100.0, 250.0);
            p.attack_ms = (analysis.attack_ms * 0.5).max(1.0);
            if analysis.transient_count > 2 { p.noise_amount = (p.noise_amount * 1.3).min(1.0); }
        }
        _ => {}
    }

    p
}

// ─── Genre Adaptation ─────────────────────────────────────

pub fn adapt_params_for_genre(params: &ResynthesisParams, genre: &str) -> ResynthesisParams {
    let mut p = params.clone();
    match genre {
        "trap" => {
            p.sub_gain = (p.sub_gain + 0.3).min(1.0);
            p.click_amount = (p.click_amount + 0.2).min(1.0);
            p.saturation_drive = (p.saturation_drive + 0.3).min(3.0);
            p.decay_ms *= 0.7;
            p.pitch_hz *= 0.9;
            p.attack_ms = (p.attack_ms * 0.8).max(0.5);
            p.noise_amount = (p.noise_amount * 0.5).max(0.0);
        }
        "techno" => {
            p.saturation_drive = (p.saturation_drive + 0.4).min(3.0);
            p.click_amount = (p.click_amount + 0.15).min(1.0);
            p.decay_ms *= 0.6;
            p.tail_ms *= 0.5;
            p.brightness = (p.brightness + 0.1).min(1.0);
            p.body_gain = (p.body_gain + 0.1).min(1.0);
        }
        "cinematic" => {
            p.duration_ms = (p.duration_ms * 1.5).min(3000.0);
            p.tail_ms = (p.tail_ms + 200.0).min(2000.0);
            p.sub_gain = (p.sub_gain + 0.2).min(1.0);
            p.saturation_drive = (p.saturation_drive + 0.2).min(3.0);
            p.body_gain = (p.body_gain + 0.1).min(1.0);
            p.decay_ms *= 1.2;
            p.pitch_drop_ratio = (p.pitch_drop_ratio + 0.1).min(1.0);
        }
        "lo-fi" | "lofi" => {
            p.saturation_drive = (p.saturation_drive + 0.5).min(4.0);
            p.brightness = (p.brightness - 0.2).max(0.0);
            p.noise_amount = (p.noise_amount + 0.15).min(1.0);
            p.decay_ms *= 0.8;
            p.attack_ms = (p.attack_ms + 2.0).min(20.0);
            p.click_amount = (p.click_amount * 0.6).max(0.0);
        }
        "house" => {
            p.decay_ms *= 0.6;
            p.click_amount = (p.click_amount + 0.1).min(1.0);
            p.brightness = (p.brightness + 0.1).min(1.0);
            p.sub_gain = (p.sub_gain + 0.1).min(1.0);
            p.saturation_drive = (p.saturation_drive + 0.15).min(3.0);
            p.noise_amount = (p.noise_amount * 0.7).max(0.0);
        }
        "drill" => {
            p.pitch_hz *= 0.85;
            p.sub_gain = (p.sub_gain + 0.35).min(1.0);
            p.click_amount = (p.click_amount + 0.3).min(1.0);
            p.saturation_drive = (p.saturation_drive + 0.4).min(3.0);
            p.decay_ms *= 0.65;
            p.attack_ms = (p.attack_ms * 0.7).max(0.3);
        }
        "dubstep" => {
            p.saturation_drive = (p.saturation_drive + 0.8).min(5.0);
            p.brightness = (p.brightness + 0.15).min(1.0);
            p.noise_amount = (p.noise_amount + 0.1).min(1.0);
            p.body_gain = (p.body_gain + 0.15).min(1.0);
            p.decay_ms *= 0.8;
        }
        "hyperpop" => {
            p.saturation_drive = (p.saturation_drive + 0.6).min(5.0);
            p.brightness = (p.brightness + 0.3).min(1.0);
            p.noise_amount = (p.noise_amount * 0.3).max(0.0);
            p.click_amount = (p.click_amount + 0.25).min(1.0);
            p.decay_ms *= 0.5;
            p.tail_ms *= 0.3;
            p.pitch_hz *= 1.2;
            p.attack_ms = (p.attack_ms * 0.6).max(0.3);
            p.body_gain = (p.body_gain * 0.7).max(0.1);
        }
        "jersey" => {
            p.pitch_hz *= 0.88;
            p.sub_gain = (p.sub_gain + 0.3).min(1.0);
            p.click_amount = (p.click_amount + 0.2).min(1.0);
            p.saturation_drive = (p.saturation_drive + 0.3).min(3.5);
            p.decay_ms *= 0.7;
            p.brightness = (p.brightness + 0.05).min(1.0);
            p.noise_amount = (p.noise_amount * 0.6).max(0.0);
        }
        "rage" => {
            p.saturation_drive = (p.saturation_drive + 0.7).min(5.0);
            p.brightness = (p.brightness + 0.2).min(1.0);
            p.body_gain = (p.body_gain + 0.2).min(1.0);
            p.sub_gain = (p.sub_gain + 0.15).min(1.0);
            p.click_amount = (p.click_amount + 0.25).min(1.0);
            p.decay_ms *= 0.7;
            p.attack_ms = (p.attack_ms * 0.6).max(0.3);
            p.noise_amount = (p.noise_amount + 0.1).min(1.0);
        }
        "industrial" => {
            p.saturation_drive = (p.saturation_drive + 0.9).min(6.0);
            p.noise_amount = (p.noise_amount + 0.2).min(1.0);
            p.body_gain = (p.body_gain + 0.15).min(1.0);
            p.brightness = (p.brightness + 0.05).min(1.0);
            p.click_amount = (p.click_amount + 0.15).min(1.0);
            p.decay_ms *= 0.8;
            p.attack_ms = (p.attack_ms * 0.7).max(0.5);
            p.pitch_hz *= if params.sound_type == SoundType::Kick { 0.9 } else { 1.1 };
        }
        "ambient_perc" | "ambient" => {
            p.decay_ms *= 1.5;
            p.tail_ms = (p.tail_ms + 300.0).min(3000.0);
            p.noise_amount = (p.noise_amount + 0.15).min(0.6);
            p.saturation_drive = (p.saturation_drive + 0.1).min(2.0);
            p.brightness = (p.brightness - 0.15).max(0.0);
            p.body_gain = (p.body_gain * 0.7).max(0.1);
            p.click_amount = (p.click_amount * 0.5).max(0.0);
            p.sub_gain = (p.sub_gain * 0.5).max(0.0);
        }
        "ui_game" | "ui" => {
            p.decay_ms *= 0.3;
            p.tail_ms = 0.0;
            p.click_amount = (p.click_amount + 0.3).min(1.0);
            p.attack_ms = (p.attack_ms * 0.5).max(0.5);
            p.noise_amount = (p.noise_amount * 0.5).max(0.0);
            p.duration_ms = p.duration_ms.min(300.0);
        }
        "pop" => {
            p.saturation_drive = (p.saturation_drive + 0.2).min(2.5);
            p.brightness = (p.brightness + 0.15).min(1.0);
            p.decay_ms *= 0.8;
            p.click_amount = (p.click_amount + 0.1).min(1.0);
            p.noise_amount = (p.noise_amount * 0.6).max(0.0);
            p.body_gain = (p.body_gain * 0.9).max(0.1);
            p.sub_gain = (p.sub_gain + 0.05).min(1.0);
        }
        _ => {}
    }
    p
}

// ─── Advanced Recreation Generation ───────────────────────

pub fn generate_approximations(
    original_samples: &[f32],
    original_analysis: &AudioAnalysis,
    count: usize,
    fidelity: f32,
    preserve_transient: bool,
    preserve_body: bool,
    preserve_tail: bool,
) -> Vec<ApproximationResult> {
    let base_params = params_from_analysis(original_analysis, original_samples);

    let mut results: Vec<ApproximationResult> = Vec::new();

    // Strategy 1: Closest recreation (high fidelity)
    let closest_params = base_params.clone().with_seed(42);
    let closest_samples = resynthesize::resynthesize(&closest_params);
    if !closest_samples.is_empty() {
        let sim = compute_similarity(original_samples, &closest_samples, original_analysis);
        results.push(ApproximationResult {
            id: uuid::Uuid::new_v4().to_string(),
            samples: closest_samples,
            similarity: sim,
            params: format!("{:?}", closest_params),
            seed: 42,
            strategy: RecreationStrategy::Closest,
            confidence: 0.9,
        });
    }

    // Strategy 2: Cleaner recreation (reduce noise/saturation)
    if base_params.noise_amount > 0.2 || base_params.saturation_drive > 1.5 {
        let mut cleaner = base_params.clone().with_seed(43);
        cleaner.noise_amount = (base_params.noise_amount * 0.4).min(1.0);
        cleaner.saturation_drive = 1.0;
        let cleaner_samples = resynthesize::resynthesize(&cleaner);
        if !cleaner_samples.is_empty() {
            let sim = compute_similarity(original_samples, &cleaner_samples, original_analysis);
            results.push(ApproximationResult {
                id: uuid::Uuid::new_v4().to_string(),
                samples: cleaner_samples,
                similarity: sim,
                params: format!("{:?}", cleaner),
                seed: 43,
                strategy: RecreationStrategy::Cleaner,
                confidence: 0.7,
            });
        }
    }

    // Strategy 3: Punchier recreation
    {
        let mut punchier = base_params.clone().with_seed(44);
        punchier.click_amount = (base_params.click_amount + 0.3).min(1.0);
        punchier.saturation_drive = (base_params.saturation_drive + 0.3).min(3.0);
        punchier.attack_ms = (base_params.attack_ms * 0.6).max(0.5);
        let punchier_samples = resynthesize::resynthesize(&punchier);
        if !punchier_samples.is_empty() {
            let sim = compute_similarity(original_samples, &punchier_samples, original_analysis);
            results.push(ApproximationResult {
                id: uuid::Uuid::new_v4().to_string(),
                samples: punchier_samples,
                similarity: sim,
                params: format!("{:?}", punchier),
                seed: 44,
                strategy: RecreationStrategy::Punchier,
                confidence: 0.75,
            });
        }
    }

    // Strategy 4: Darker recreation
    {
        let mut darker = base_params.clone().with_seed(45);
        darker.brightness = (base_params.brightness - 0.3).max(0.0);
        darker.saturation_drive = (base_params.saturation_drive + 0.15).min(3.0);
        let darker_samples = resynthesize::resynthesize(&darker);
        if !darker_samples.is_empty() {
            let sim = compute_similarity(original_samples, &darker_samples, original_analysis);
            results.push(ApproximationResult {
                id: uuid::Uuid::new_v4().to_string(),
                samples: darker_samples,
                similarity: sim,
                params: format!("{:?}", darker),
                seed: 45,
                strategy: RecreationStrategy::Darker,
                confidence: 0.7,
            });
        }
    }

    // Strategy 5: Brighter recreation
    {
        let mut brighter = base_params.clone().with_seed(46);
        brighter.brightness = (base_params.brightness + 0.3).min(1.0);
        brighter.noise_amount = (base_params.noise_amount + 0.05).min(1.0);
        let brighter_samples = resynthesize::resynthesize(&brighter);
        if !brighter_samples.is_empty() {
            let sim = compute_similarity(original_samples, &brighter_samples, original_analysis);
            results.push(ApproximationResult {
                id: uuid::Uuid::new_v4().to_string(),
                samples: brighter_samples,
                similarity: sim,
                params: format!("{:?}", brighter),
                seed: 46,
                strategy: RecreationStrategy::Brighter,
                confidence: 0.7,
            });
        }
    }

    // Strategy 6: Genre-adapted recreations
    let genres = ["trap", "techno", "cinematic", "lo-fi", "drill"];
    for (i, genre) in genres.iter().enumerate() {
        let mut genre_params = base_params.clone().with_seed(50 + i as u64);
        genre_params = adapt_params_for_genre(&genre_params, genre);
        let genre_samples = resynthesize::resynthesize(&genre_params);
        if !genre_samples.is_empty() {
            let sim = compute_similarity(original_samples, &genre_samples, original_analysis);
            results.push(ApproximationResult {
                id: uuid::Uuid::new_v4().to_string(),
                samples: genre_samples,
                similarity: sim,
                params: format!("{:?}", genre_params),
                seed: 50 + i as u64,
                strategy: RecreationStrategy::GenreAdapted(genre.to_string()),
                confidence: 0.6,
            });
        }
    }

    // Strategy 7: Randomized variations (from original fidelity param)
    for i in 0..count.max(2) {
        let seed = i as u64 + 100;
        let mut params = base_params.clone().with_seed(seed);
        let variation = 1.0 - fidelity;
        params = params.randomize(variation * 0.5);

        if preserve_transient {
            params.attack_ms = original_analysis.attack_ms.clamp(0.1, 50.0);
            params.click_amount = base_params.click_amount;
        }
        if preserve_body {
            params.pitch_hz = base_params.pitch_hz;
            params.pitch_drop_ratio = base_params.pitch_drop_ratio;
            params.body_gain = base_params.body_gain;
            params.sub_gain = base_params.sub_gain;
        }
        if preserve_tail {
            params.tail_ms = original_analysis.tail_ms.max(base_params.tail_ms * 0.8);
            params.decay_ms = original_analysis.decay_ms.max(base_params.decay_ms * 0.8);
        }

        let samples = resynthesize::resynthesize(&params);
        if samples.is_empty() { continue; }

        let similarity = compute_similarity(original_samples, &samples, original_analysis);

        results.push(ApproximationResult {
            id: uuid::Uuid::new_v4().to_string(),
            samples,
            similarity,
            params: format!("{:?}", params),
            seed,
            strategy: RecreationStrategy::Mutated,
            confidence: (fidelity * 0.8).clamp(0.3, 0.8),
        });
    }

    results.sort_by(|a, b| b.similarity.overall.partial_cmp(&a.similarity.overall).unwrap_or(std::cmp::Ordering::Equal));
    results
}

// ─── Targeted Mutations ───────────────────────────────────

pub fn recreate_with_target(
    samples: &[f32],
    target: &str,
    intensity: f32,
) -> (Vec<f32>, SimilarityReport) {
    let analysis = analyze_audio(samples, SAMPLE_RATE, 1);
    let base_params = params_from_analysis(&analysis, samples);
    let mut params = base_params.clone().with_seed(42);

    match target {
        "cleaner" => {
            params.noise_amount = (base_params.noise_amount * (1.0 - intensity)).max(0.0);
            params.saturation_drive = 1.0 + (base_params.saturation_drive - 1.0) * (1.0 - intensity);
        }
        "punchier" => {
            params.click_amount = (params.click_amount + intensity * 0.4).min(1.0);
            params.attack_ms = (params.attack_ms * (1.0 - intensity * 0.5)).max(0.5);
        }
        "darker" => {
            params.brightness = (params.brightness - intensity * 0.4).max(0.0);
        }
        "brighter" => {
            params.brightness = (params.brightness + intensity * 0.4).min(1.0);
        }
        "warmer" => {
            params.brightness = (params.brightness - intensity * 0.2).max(0.0);
            params.saturation_drive = (params.saturation_drive + intensity * 0.3).min(3.0);
        }
        "subbier" => {
            params.sub_gain = (params.sub_gain + intensity * 0.4).min(1.0);
        }
        "noisier" => {
            params.noise_amount = (params.noise_amount + intensity * 0.3).min(1.0);
        }
        "shorter" => {
            params.duration_ms *= 1.0 - intensity * 0.5;
            params.decay_ms *= 1.0 - intensity * 0.4;
        }
        "longer" => {
            params.duration_ms *= 1.0 + intensity * 0.5;
            params.tail_ms = (params.tail_ms + intensity * 200.0).min(2000.0);
        }
        "harder" => {
            params.saturation_drive = (params.saturation_drive + intensity * 0.8).min(5.0);
            params.click_amount = (params.click_amount + intensity * 0.2).min(1.0);
        }
        "softer" => {
            params.click_amount = (params.click_amount * (1.0 - intensity * 0.5)).max(0.0);
            params.saturation_drive = (params.saturation_drive - intensity * 0.3).max(1.0);
            params.attack_ms = (params.attack_ms + intensity * 5.0).min(30.0);
        }
        _ => {}
    }

    let recreated = resynthesize::resynthesize(&params);
    let similarity = compute_similarity(samples, &recreated, &analysis);
    (recreated, similarity)
}

pub fn recreate_single(
    samples: &[f32],
    fidelity: f32,
) -> (Vec<f32>, AudioAnalysis, SimilarityReport) {
    let analysis = analyze_audio(samples, SAMPLE_RATE, 1);
    let base_params = params_from_analysis(&analysis, samples);
    let params = base_params.clone().with_seed(42).randomize(1.0 - fidelity);
    let recreated = resynthesize::resynthesize(&params);
    let similarity = compute_similarity(samples, &recreated, &analysis);
    (recreated, analysis, similarity)
}

// ─── Genre Shift ──────────────────────────────────────────

pub fn genre_shift(samples: &[f32], _from_genre: &str, to_genre: &str) -> (Vec<f32>, SimilarityReport) {
    let analysis = analyze_audio(samples, SAMPLE_RATE, 1);
    let base_params = params_from_analysis(&analysis, samples);
    let mut params = adapt_params_for_genre(&base_params, to_genre);
    // Keep the original structure but apply to-genre adaptation
    params.attack_ms = base_params.attack_ms;
    params.pitch_drop_ratio = base_params.pitch_drop_ratio;
    let recreated = resynthesize::resynthesize(&params);
    let similarity = compute_similarity(samples, &recreated, &analysis);
    (recreated, similarity)
}

// ─── Advanced Recreation Modes ───────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AdvancedRecreationConfig {
    pub mode: AdvancedRecreationMode,
    pub fidelity: f32,
    pub transient_preservation: f32,
    pub body_preservation: f32,
    pub tail_preservation: f32,
    pub spectral_matching: f32,
    pub sub_reconstruction: f32,
    pub transient_timing_align: bool,
    pub harmonic_profile_match: bool,
    pub tail_texture_match: bool,
}

impl Default for AdvancedRecreationConfig {
    fn default() -> Self {
        Self {
            mode: AdvancedRecreationMode::Closest,
            fidelity: 0.7,
            transient_preservation: 0.8,
            body_preservation: 0.6,
            tail_preservation: 0.5,
            spectral_matching: 0.5,
            sub_reconstruction: 0.5,
            transient_timing_align: true,
            harmonic_profile_match: true,
            tail_texture_match: false,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum AdvancedRecreationMode {
    Closest,
    Cleaner,
    Harder,
    MoreModern,
    MoreAnalog,
    Cinematic,
    Experimental,
    Modernized,
    Darker,
    Brighter,
}

impl AdvancedRecreationMode {
    pub fn label(&self) -> &str {
        match self {
            AdvancedRecreationMode::Closest => "closest",
            AdvancedRecreationMode::Cleaner => "cleaner",
            AdvancedRecreationMode::Harder => "harder",
            AdvancedRecreationMode::MoreModern => "more modern",
            AdvancedRecreationMode::MoreAnalog => "more analog",
            AdvancedRecreationMode::Cinematic => "cinematic",
            AdvancedRecreationMode::Experimental => "experimental",
            AdvancedRecreationMode::Modernized => "modernized",
            AdvancedRecreationMode::Darker => "darker",
            AdvancedRecreationMode::Brighter => "brighter",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s {
            "closest" | "closer" => AdvancedRecreationMode::Closest,
            "cleaner" => AdvancedRecreationMode::Cleaner,
            "harder" => AdvancedRecreationMode::Harder,
            "more modern" | "modern" | "modernized" => AdvancedRecreationMode::Modernized,
            "more analog" | "analog" => AdvancedRecreationMode::MoreAnalog,
            "cinematic" => AdvancedRecreationMode::Cinematic,
            "experimental" => AdvancedRecreationMode::Experimental,
            "darker" => AdvancedRecreationMode::Darker,
            "brighter" => AdvancedRecreationMode::Brighter,
            _ => AdvancedRecreationMode::Closest,
        }
    }

    pub fn all_modes() -> Vec<AdvancedRecreationMode> {
        vec![
            AdvancedRecreationMode::Closest,
            AdvancedRecreationMode::Cleaner,
            AdvancedRecreationMode::Harder,
            AdvancedRecreationMode::Modernized,
            AdvancedRecreationMode::Cinematic,
            AdvancedRecreationMode::Experimental,
            AdvancedRecreationMode::Darker,
            AdvancedRecreationMode::Brighter,
            AdvancedRecreationMode::MoreAnalog,
        ]
    }
}

pub fn recreate_advanced(
    samples: &[f32],
    config: &AdvancedRecreationConfig,
) -> (Vec<f32>, SimilarityReport) {
    let analysis = analyze_audio(samples, SAMPLE_RATE, 1);
    let base_params = params_from_analysis(&analysis, samples);
    let mut params = base_params.clone().with_seed(42);

    match config.mode {
        AdvancedRecreationMode::Closest => {
            let variation = 1.0 - config.fidelity;
            params = base_params.clone().with_seed(42).randomize(variation * 0.3);
            if config.transient_preservation > 0.5 {
                params.attack_ms = analysis.attack_ms.clamp(0.1, 50.0);
                params.click_amount = base_params.click_amount;
            }
            if config.body_preservation > 0.5 {
                params.pitch_hz = base_params.pitch_hz;
                params.body_gain = base_params.body_gain;
            }
            if config.tail_preservation > 0.5 {
                params.tail_ms = analysis.tail_ms.max(base_params.tail_ms * 0.8);
                params.decay_ms = analysis.decay_ms.max(base_params.decay_ms * 0.8);
            }
            if config.spectral_matching > 0.3 {
                params.brightness = base_params.brightness * (0.8 + config.spectral_matching * 0.2);
                params.saturation_drive = base_params.saturation_drive * (0.8 + config.spectral_matching * 0.2).max(1.0);
            }
            if config.sub_reconstruction > 0.3 {
                params.sub_gain = base_params.sub_gain * (0.7 + config.sub_reconstruction * 0.3);
            }
        }
        AdvancedRecreationMode::Cleaner => {
            params.noise_amount = (base_params.noise_amount * 0.3).max(0.0);
            params.saturation_drive = 1.0 + (base_params.saturation_drive - 1.0) * 0.3;
            params.brightness = (base_params.brightness + 0.1).min(1.0);
            params.click_amount = base_params.click_amount.max(0.1);
        }
        AdvancedRecreationMode::Harder => {
            params.saturation_drive = (base_params.saturation_drive + 0.6 * config.fidelity).min(5.0);
            params.click_amount = (base_params.click_amount + 0.25 * config.fidelity).min(1.0);
            params.attack_ms = (base_params.attack_ms * (1.0 - 0.3 * config.fidelity)).max(0.5);
            params.body_gain = (base_params.body_gain + 0.1 * config.fidelity).min(1.0);
            params.sub_gain = (base_params.sub_gain + 0.1 * config.fidelity).min(1.0);
        }
        AdvancedRecreationMode::MoreModern => {
            params.decay_ms = base_params.decay_ms * 0.7;
            params.tail_ms = base_params.tail_ms * 0.6;
            params.click_amount = (base_params.click_amount + 0.15).min(1.0);
            params.sub_gain = (base_params.sub_gain + 0.15).min(1.0);
            params.brightness = (base_params.brightness + 0.1).min(1.0);
            params.saturation_drive = (base_params.saturation_drive + 0.15).min(3.0);
            params.noise_amount = (base_params.noise_amount * 0.4).max(0.0);
            params.attack_ms = (base_params.attack_ms * 0.8).max(0.5);
        }
        AdvancedRecreationMode::MoreAnalog => {
            params.saturation_drive = (base_params.saturation_drive + 0.25).min(3.0);
            params.brightness = (base_params.brightness - 0.05).max(0.0);
            params.noise_amount = (base_params.noise_amount + 0.05).min(0.3);
            if config.tail_texture_match {
                params.tail_ms = base_params.tail_ms * 1.1;
            }
        }
        AdvancedRecreationMode::Cinematic => {
            params.duration_ms = (base_params.duration_ms * 1.4).min(3000.0);
            params.tail_ms = (base_params.tail_ms + 200.0).min(2000.0);
            params.sub_gain = (base_params.sub_gain + 0.3).min(1.0);
            params.body_gain = (base_params.body_gain + 0.15).min(1.0);
            params.saturation_drive = (base_params.saturation_drive + 0.4).min(4.0);
            params.brightness = (base_params.brightness + 0.1).min(1.0);
            params.decay_ms = base_params.decay_ms * 1.2;
            params.pitch_drop_ratio = (base_params.pitch_drop_ratio + 0.15).min(1.0);
        }
        AdvancedRecreationMode::Experimental => {
            // Unconventional parameter combinations
            let variation = 1.0 - config.fidelity;
            params = params.randomize(variation * 0.8);
            params.pitch_hz = base_params.pitch_hz * if config.fidelity > 0.5 { 0.7 } else { 1.4 };
            params.noise_amount = (1.0 - (1.0 - base_params.noise_amount) * 0.5).min(1.0);
            params.saturation_drive = (base_params.saturation_drive + variation * 2.0).min(6.0);
            params.body_gain = (base_params.body_gain * (0.5 + variation)).min(1.0);
            params.click_amount = (base_params.click_amount + variation * 0.5).min(1.0);
            if config.tail_texture_match {
                params.tail_ms = base_params.tail_ms * (0.5 + variation);
            }
        }
        AdvancedRecreationMode::Modernized => {
            params.decay_ms = base_params.decay_ms * 0.6;
            params.tail_ms = base_params.tail_ms * 0.5;
            params.click_amount = (base_params.click_amount + 0.2).min(1.0);
            params.sub_gain = (base_params.sub_gain + 0.2).min(1.0);
            params.brightness = (base_params.brightness + 0.15).min(1.0);
            params.saturation_drive = (base_params.saturation_drive + 0.3).min(3.5);
            params.noise_amount = (base_params.noise_amount * 0.35).max(0.0);
            params.attack_ms = (base_params.attack_ms * 0.7).max(0.3);
            params.pitch_drop_ratio = base_params.pitch_drop_ratio * 0.8;
        }
        AdvancedRecreationMode::Darker => {
            params.brightness = (base_params.brightness - 0.35).max(0.0);
            params.saturation_drive = (base_params.saturation_drive + 0.2).min(3.0);
            params.noise_amount = (base_params.noise_amount * 0.6).max(0.0);
            params.sub_gain = (base_params.sub_gain + 0.1).min(1.0);
            params.body_gain = (base_params.body_gain + 0.05).min(1.0);
        }
        AdvancedRecreationMode::Brighter => {
            params.brightness = (base_params.brightness + 0.35).min(1.0);
            params.noise_amount = (base_params.noise_amount + 0.1).min(1.0);
            params.click_amount = (base_params.click_amount + 0.1).min(1.0);
            params.saturation_drive = (base_params.saturation_drive + 0.1).min(3.0);
            if params.noise_hp_hz < 2000.0 {
                params.noise_hp_hz = 2000.0;
            }
        }
    }

    let recreated = resynthesize::resynthesize(&params);
    let humanize_amt = if config.mode == AdvancedRecreationMode::MoreAnalog { 0.3 } else { 0.0 };
    if humanize_amt > 0.0 {
        let hp = super::humanize::HumanizeParams {
            analog_drift: humanize_amt * 0.15,
            instability: humanize_amt * 0.05,
            saturation_randomness: humanize_amt * 0.1,
            ..Default::default()
        };
        let mut with_human = recreated.clone();
        super::humanize::humanize(&mut with_human, &hp, 42);
        let sim = compute_similarity(samples, &with_human, &analysis);
        return (with_human, sim);
    }

    let similarity = compute_similarity(samples, &recreated, &analysis);
    (recreated, similarity)
}

pub fn recreate_advanced_with_target(
    samples: &[f32],
    mode: &str,
    fidelity: f32,
) -> (Vec<f32>, SimilarityReport) {
    let am = AdvancedRecreationMode::from_label(mode);
    let config = AdvancedRecreationConfig {
        mode: am,
        fidelity,
        ..Default::default()
    };
    recreate_advanced(samples, &config)
}

// ─── Multi-Mode Recreation Generator ─────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecreationModeResult {
    pub mode: String,
    pub samples: Vec<f32>,
    pub similarity: SimilarityReport,
    pub seed: u64,
    pub confidence: f32,
    pub quality_score: f32,
}

pub fn generate_all_recreation_modes(
    samples: &[f32],
    fidelity: f32,
    include_genres: bool,
) -> Vec<RecreationModeResult> {
    let analysis = analyze_audio(samples, SAMPLE_RATE, 1);
    let mut results = Vec::new();
    let desired_modes = AdvancedRecreationMode::all_modes();

    for mode in desired_modes {
        let config = AdvancedRecreationConfig {
            mode: mode.clone(),
            fidelity,
            transient_preservation: fidelity,
            body_preservation: fidelity * 0.8,
            tail_preservation: fidelity * 0.7,
            spectral_matching: fidelity * 0.7,
            sub_reconstruction: fidelity * 0.6,
            transient_timing_align: true,
            harmonic_profile_match: true,
            tail_texture_match: mode == AdvancedRecreationMode::MoreAnalog,
        };
        let (mode_samples, similarity) = recreate_advanced(samples, &config);
        let q = crate::quality::compute_quality(
            &mode_samples,
            SoundType::from_str(&analysis.sound_type_hint),
            &mode.label(),
            false,
        );
        let quality_score = q.spectral_quality * 0.3 + q.transient_quality * 0.3
            + q.dynamic_range * 0.2 + q.punch_quality * 0.2;

        results.push(RecreationModeResult {
            mode: mode.label().to_string(),
            samples: mode_samples,
            similarity,
            seed: 42,
            confidence: fidelity * 0.7 + 0.2,
            quality_score,
        });
    }

    // Genre-adapted variants if requested
    if include_genres {
        let base_params = params_from_analysis(&analysis, samples);
        let genres = ["trap", "drill", "techno", "house", "lo-fi", "cinematic", "hyperpop", "industrial"];
        for (i, genre) in genres.iter().enumerate() {
            let seed = 100 + i as u64;
            let mut genre_params = adapt_params_for_genre(&base_params, genre);
            genre_params.seed = seed;
            let genre_samples = resynthesize::resynthesize(&genre_params);
            if genre_samples.is_empty() { continue; }
            let similarity = compute_similarity(samples, &genre_samples, &analysis);
            let q = crate::quality::compute_quality(
                &genre_samples,
                SoundType::from_str(&analysis.sound_type_hint),
                genre,
                false,
            );
            let quality_score = q.spectral_quality * 0.3 + q.transient_quality * 0.3
                + q.dynamic_range * 0.2 + q.punch_quality * 0.2;
            results.push(RecreationModeResult {
                mode: format!("genre:{}", genre),
                samples: genre_samples,
                similarity,
                seed,
                confidence: 0.5,
                quality_score,
            });
        }
    }

    results.sort_by(|a, b| b.similarity.overall.partial_cmp(&a.similarity.overall).unwrap_or(std::cmp::Ordering::Equal));
    results
}

// ─── Sound Type Classification ────────────────────────────

pub fn classify_sound_type(samples: &[f32]) -> (SoundType, f32) {
    let analysis = analyze_audio(samples, SAMPLE_RATE, 1);
    let st = SoundType::from_str(&analysis.sound_type_hint);
    let confidence = compute_recreation_confidence(&extract_sound_structure(samples, &analysis), &analysis);
    (st, confidence)
}

// ─── Feedback Integration ─────────────────────────────────

pub fn record_recreation_feedback(recreation_id: &str, thumbs_up: bool) {
    let mut store = crate::feedback::FeedbackStore::load();
    store.set_thumbs(recreation_id, thumbs_up, !thumbs_up);
}

// ─── Recreation Ranking + Selection ─────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RankedRecreation {
    pub approximation: ApproximationResult,
    pub rank_score: f32,
    pub quality_score: f32,
    pub novelty_score: f32,
    pub user_affinity: f32,
    pub rank_label: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum RankingStrategy {
    Closest,
    Cleanest,
    MostAggressive,
    MostExperimental,
    BestQuality,
    MostNovel,
    Balanced,
}

pub fn rank_recreations(
    approximations: Vec<ApproximationResult>,
    strategy: RankingStrategy,
    user_favorites: &[String],
    quality_scores: &[f32],
) -> Vec<RankedRecreation> {
    let mut ranked: Vec<RankedRecreation> = approximations
        .into_iter()
        .enumerate()
        .map(|(i, approx)| {
            let similarity = &approx.similarity;
            let novelty = 1.0 - similarity.overall;
            let user_affinity = if user_favorites.contains(&approx.id) { 0.8 } else { 0.3 };
            let quality = quality_scores.get(i).copied().unwrap_or(0.5);
            let aggressiveness = similarity.transient_match * 0.5 + (1.0 - similarity.noise_match) * 0.3 + similarity.sub_match * 0.2;
            let experimental = novelty * 0.5 + (1.0 - similarity.spectral_match) * 0.3 + (1.0 - similarity.envelope_match) * 0.2;

            let rank_score = match strategy {
                RankingStrategy::Closest => {
                    similarity.overall * 0.5 + similarity.envelope_match * 0.2 +
                    similarity.spectral_match * 0.2 + quality * 0.1
                }
                RankingStrategy::Cleanest => {
                    quality * 0.4 + (1.0 - similarity.noise_match) * 0.2 +
                    similarity.spectral_match * 0.2 + user_affinity * 0.2
                }
                RankingStrategy::MostAggressive => {
                    aggressiveness * 0.5 + similarity.transient_match * 0.2 +
                    similarity.sub_match * 0.2 + quality * 0.1
                }
                RankingStrategy::MostExperimental => {
                    experimental * 0.5 + novelty * 0.3 + quality * 0.2
                }
                RankingStrategy::BestQuality => {
                    quality * 0.5 + similarity.overall * 0.3 + user_affinity * 0.2
                }
                RankingStrategy::MostNovel => {
                    novelty * 0.6 + experimental * 0.2 + quality * 0.2
                }
                RankingStrategy::Balanced => {
                    similarity.overall * 0.25 + quality * 0.25 + novelty * 0.2 +
                    aggressiveness * 0.15 + user_affinity * 0.15
                }
            };

            let rank_label = match strategy {
                RankingStrategy::Closest => format!("{:.0}% match", similarity.overall * 100.0),
                RankingStrategy::Cleanest => format!("{:.0}% clean", quality * 100.0),
                RankingStrategy::MostAggressive => format!("{:.0}% aggressive", aggressiveness * 100.0),
                RankingStrategy::MostExperimental => format!("{:.0}% experimental", experimental * 100.0),
                RankingStrategy::BestQuality => format!("{:.0}% quality", quality * 100.0),
                RankingStrategy::MostNovel => format!("{:.0}% novel", novelty * 100.0),
                RankingStrategy::Balanced => format!("score {:.0}", rank_score * 100.0),
            };

            RankedRecreation {
                rank_score,
                quality_score: quality,
                novelty_score: novelty,
                user_affinity,
                rank_label,
                approximation: approx,
            }
        })
        .collect();

    ranked.sort_by(|a, b| b.rank_score.partial_cmp(&a.rank_score).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

pub fn generate_and_rank(
    samples: &[f32],
    analysis: &AudioAnalysis,
    strategy: RankingStrategy,
    user_favorites: &[String],
) -> Vec<RankedRecreation> {
    let approximations = generate_approximations(samples, analysis, 6, 0.7, true, true, true);
    let quality_scores: Vec<f32> = approximations.iter().map(|a| {
        let q = crate::quality::compute_quality(
            &a.samples,
            SoundType::from_str(&analysis.sound_type_hint),
            &a.strategy.label(),
            false,
        );
        q.spectral_quality * 0.3 + q.transient_quality * 0.3 + q.dynamic_range * 0.2 + q.punch_quality * 0.2
    }).collect();

    rank_recreations(approximations, strategy, user_favorites, &quality_scores)
}

// ─── Reference + Prompt Fusion ───────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FusionResult {
    pub recreation: ApproximationResult,
    pub fusion_sound: Vec<f32>,
    pub fusion_similarity: SimilarityReport,
    pub edit_applied: String,
    pub edit_intensity: f32,
}

pub fn recreate_with_prompt(
    samples: &[f32],
    prompt_controls: &crate::prompt_dsp::PromptDspControls,
) -> FusionResult {
    let analysis = analyze_audio(samples, SAMPLE_RATE, 1);
    let base_params = params_from_analysis(&analysis, samples);
    let recreation_params = base_params.clone().with_seed(42);
    let recreation_samples = resynthesize::resynthesize(&recreation_params);
    let base_similarity = compute_similarity(samples, &recreation_samples, &analysis);

    let recreation_result = ApproximationResult {
        id: uuid::Uuid::new_v4().to_string(),
        samples: recreation_samples.clone(),
        similarity: base_similarity,
        params: format!("{:?}", recreation_params),
        seed: 42,
        strategy: RecreationStrategy::Closest,
        confidence: 0.85,
    };

    let st = SoundType::from_str(&analysis.sound_type_hint);
    let pitch = analysis.pitch_estimate.unwrap_or(200.0);
    let base_for_prompt = resynthesize::params_for_sound_type(st, pitch, analysis.duration_ms);
    let fusion_params = prompt_controls.to_resynthesis_params(&base_for_prompt);

    let mut fusion_params = fusion_params.clone();
    fusion_params.attack_ms = if prompt_controls.attack_ms.is_some() {
        prompt_controls.attack_ms.unwrap()
    } else {
        base_params.attack_ms
    };
    fusion_params.decay_ms = prompt_controls.decay_ms.unwrap_or(base_params.decay_ms);
    fusion_params.tail_ms = prompt_controls.tail_ms.unwrap_or(base_params.tail_ms);
    fusion_params.brightness = prompt_controls.brightness.unwrap_or(base_params.brightness);
    fusion_params.noise_amount = prompt_controls.noise_amount.unwrap_or(base_params.noise_amount);
    fusion_params.saturation_drive = prompt_controls.saturation_drive.unwrap_or(base_params.saturation_drive);
    fusion_params.sub_gain = prompt_controls.sub_gain.unwrap_or(base_params.sub_gain);
    fusion_params.click_amount = prompt_controls.click_amount.unwrap_or(base_params.click_amount);
    fusion_params.body_gain = prompt_controls.body_gain.unwrap_or(base_params.body_gain);
    fusion_params.pitch_hz = prompt_controls.pitch_hz.unwrap_or(base_params.pitch_hz);
    fusion_params.seed = 99;

    let fusion_samples = resynthesize::resynthesize(&fusion_params);
    let fusion_similarity = compute_similarity(samples, &fusion_samples, &analysis);

    let edit_intensity = prompt_controls.descriptors.iter()
        .map(|d| d.confidence)
        .fold(0.0f32, f32::max);

    let edit_summary = if !prompt_controls.descriptors.is_empty() {
        prompt_controls.descriptors.iter()
            .map(|d| d.word.clone())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "none".to_string()
    };

    FusionResult {
        recreation: recreation_result,
        fusion_sound: fusion_samples,
        fusion_similarity,
        edit_applied: edit_summary,
        edit_intensity,
    }
}

// ─── Reference-Preserving Transforms ─────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PreservingTransformConfig {
    pub prompt: String,
    pub transient_preservation: f32,
    pub body_preservation: f32,
    pub spectral_preservation: f32,
    pub envelope_preservation: f32,
}

impl Default for PreservingTransformConfig {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            transient_preservation: 0.7,
            body_preservation: 0.5,
            spectral_preservation: 0.4,
            envelope_preservation: 0.6,
        }
    }
}

pub fn preserving_transform(
    samples: &[f32],
    config: &PreservingTransformConfig,
) -> (Vec<f32>, SimilarityReport) {
    let analysis = analyze_audio(samples, SAMPLE_RATE, 1);
    let base_params = params_from_analysis(&analysis, samples);
    let ctrl = crate::prompt_dsp::parse_prompt_rich(&config.prompt);
    
    let st = SoundType::from_str(&analysis.sound_type_hint);
    let pitch = analysis.pitch_estimate.unwrap_or(200.0);
    let prompt_base = resynthesize::params_for_sound_type(st, pitch, analysis.duration_ms);
    let prompt_params = ctrl.to_resynthesis_params(&prompt_base);

    let mut blended = base_params.clone().with_seed(42);

    // Blend preservation with prompt modifications
    if config.transient_preservation > 0.0 {
        blended.attack_ms = base_params.attack_ms * config.transient_preservation
            + prompt_params.attack_ms * (1.0 - config.transient_preservation);
        blended.click_amount = base_params.click_amount * config.transient_preservation
            + prompt_params.click_amount * (1.0 - config.transient_preservation);
    }
    if config.body_preservation > 0.0 {
        blended.pitch_hz = base_params.pitch_hz * config.body_preservation
            + prompt_params.pitch_hz * (1.0 - config.body_preservation);
        blended.body_gain = base_params.body_gain * config.body_preservation
            + prompt_params.body_gain * (1.0 - config.body_preservation);
        blended.sub_gain = base_params.sub_gain * config.body_preservation
            + prompt_params.sub_gain * (1.0 - config.body_preservation);
    }
    if config.spectral_preservation > 0.0 {
        blended.brightness = base_params.brightness * config.spectral_preservation
            + prompt_params.brightness * (1.0 - config.spectral_preservation);
        blended.saturation_drive = base_params.saturation_drive * config.spectral_preservation
            + prompt_params.saturation_drive * (1.0 - config.spectral_preservation);
    }
    if config.envelope_preservation > 0.0 {
        blended.decay_ms = base_params.decay_ms * config.envelope_preservation
            + prompt_params.decay_ms * (1.0 - config.envelope_preservation);
        blended.tail_ms = base_params.tail_ms * config.envelope_preservation
            + prompt_params.tail_ms * (1.0 - config.envelope_preservation);
        blended.duration_ms = base_params.duration_ms * config.envelope_preservation
            + prompt_params.duration_ms * (1.0 - config.envelope_preservation);
    }
    if let Some(v) = ctrl.noise_amount { blended.noise_amount = v; }
    if let Some(v) = ctrl.pitch_drop_ratio { blended.pitch_drop_ratio = v; }

    let transformed = resynthesize::resynthesize(&blended);
    let similarity = compute_similarity(samples, &transformed, &analysis);
    (transformed, similarity)
}
