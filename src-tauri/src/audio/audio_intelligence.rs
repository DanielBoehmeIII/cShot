use super::analyze::AudioAnalysis;
use super::spectral_intelligence::{extract_fingerprint, SpectralFingerprint};
use super::SoundType;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioIntelligenceReport {
    pub genre_role: GenreRole,
    pub transient_classification: TransientClassification,
    pub tonal_noise_decomposition: TonalNoiseDecomp,
    pub impact_prediction: ImpactPrediction,
    pub mix_role: MixRole,
    pub spectral_complexity: SpectralComplexity,
    pub confidence: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum GenreRole {
    KickDrum,
    SnareDrum,
    ClosedHat,
    OpenHat,
    Clap,
    Tom,
    Percussion,
    BassHit,
    FxSound,
    Vocal,
    Synth,
    Texture,
    Hybrid,
    Unknown,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TransientClassification {
    pub transient_count: usize,
    pub primary_type: TransientType,
    pub attack_sharpness: f32,
    pub decay_character: f32,
    pub has_multiple_hits: bool,
    pub onset_density: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TransientType {
    Sharp,
    Blunt,
    Layered,
    NoiseBurst,
    Tonal,
    MultiHit,
    None,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TonalNoiseDecomp {
    pub tonal_ratio: f32,
    pub noise_ratio: f32,
    pub harmonic_strength: f32,
    pub inharmonic_strength: f32,
    pub residual_noise: f32,
    pub tonal_peaks: Vec<(f32, f32)>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImpactPrediction {
    pub impact_score: f32,
    pub punch_factor: f32,
    pub weight: f32,
    pub transient_to_body_ratio: f32,
    pub perceived_loudness: f32,
    pub sub_energy: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum MixRole {
    Foundation,
    Backbone,
    Groove,
    Accent,
    Fill,
    Texture,
    Effect,
    TonalElement,
    Unknown,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpectralComplexity {
    pub spectral_flatness: f32,
    pub harmonic_noise_ratio: f32,
    pub band_energy_entropy: f32,
    pub spectral_spread: f32,
    pub complexity_score: f32,
    pub uniformity: f32,
}

pub fn analyze_intelligence(samples: &[f32], analysis: &AudioAnalysis) -> AudioIntelligenceReport {
    let fp = if samples.len() > 256 {
        extract_fingerprint(samples)
    } else {
        SpectralFingerprint {
            bands: vec![0.0; 8], centroid: 0.0, rolloff: 0.0, brightness: 0.0,
            sub_energy: 0.0, low_mid_energy: 0.0, high_mid_energy: 0.0,
            presence_energy: 0.0, air_energy: 0.0,
            tonal_ratio: 0.5, noise_ratio: 0.5,
            resonance_peaks: vec![], harshness: 0.0, muddiness: 0.0,
            spectral_flatness: 0.5,
        }
    };

    let genre_role = detect_genre_role(analysis, &fp);
    let transient_classification = classify_transients(analysis, samples);
    let tonal_noise_decomposition = decompose_tonal_noise(&fp, analysis);
    let impact_prediction = predict_impact(analysis, &fp);
    let mix_role = estimate_mix_role(&genre_role, analysis);
    let spectral_complexity = compute_spectral_complexity(&fp, analysis);

    let confidence = compute_confidence(&genre_role, &transient_classification, &fp);

    AudioIntelligenceReport {
        genre_role,
        transient_classification,
        tonal_noise_decomposition,
        impact_prediction,
        mix_role,
        spectral_complexity,
        confidence,
    }
}

fn detect_genre_role(analysis: &AudioAnalysis, fp: &SpectralFingerprint) -> GenreRole {
    let st = SoundType::from_str(&analysis.sound_type_hint);
    match st {
        SoundType::Kick => GenreRole::KickDrum,
        SoundType::Snare => GenreRole::SnareDrum,
        SoundType::ClosedHat => GenreRole::ClosedHat,
        SoundType::OpenHat => GenreRole::OpenHat,
        SoundType::Clap => GenreRole::Clap,
        SoundType::Tom => GenreRole::Tom,
        SoundType::Perc => GenreRole::Percussion,
        SoundType::Bass => {
            if fp.tonal_ratio > 0.6 && fp.resonance_peaks.len() > 2 {
                GenreRole::Synth
            } else {
                GenreRole::BassHit
            }
        }
        SoundType::Fx => {
            if fp.noise_ratio > 0.7 {
                GenreRole::Texture
            } else {
                GenreRole::FxSound
            }
        }
        SoundType::Other => {
            if analysis.has_pitch && analysis.pitch_estimate.unwrap_or(200.0) > 80.0
                && fp.tonal_ratio > 0.4 {
                if analysis.duration_ms > 1000.0 { GenreRole::Vocal }
                else { GenreRole::Synth }
            } else if fp.noise_ratio > 0.6 && analysis.duration_ms > 500.0 {
                GenreRole::Texture
            } else {
                GenreRole::Unknown
            }
        }
    }
}

fn classify_transients(analysis: &AudioAnalysis, _samples: &[f32]) -> TransientClassification {
    let count = analysis.transient_count;
    let strength = analysis.transient_strength;
    let crest = if analysis.rms > 0.0 { analysis.peak / analysis.rms } else { 1.0 };
    let has_onsets = !analysis.onset_times_ms.is_empty();

    let onset_density = if analysis.duration_ms > 0.0 {
        (count as f32 / (analysis.duration_ms / 1000.0).max(0.01)).min(50.0)
    } else {
        0.0
    };

    let primary_type = if count > 4 && onset_density > 8.0 {
        TransientType::MultiHit
    } else if count >= 3 && count <= 10 && analysis.noise_estimate > 0.5 {
        TransientType::Layered
    } else if analysis.noise_estimate > 0.6 && strength > 0.5 {
        TransientType::NoiseBurst
    } else if analysis.has_pitch && crest > 5.0 {
        TransientType::Tonal
    } else if crest > 8.0 {
        TransientType::Sharp
    } else if strength > 0.3 {
        TransientType::Blunt
    } else if count == 0 && !has_onsets {
        TransientType::None
    } else {
        TransientType::Blunt
    };

    TransientClassification {
        transient_count: count,
        primary_type,
        attack_sharpness: (crest / 15.0).min(1.0),
        decay_character: (analysis.decay_ms / 300.0).min(1.0),
        has_multiple_hits: has_onsets && count > 1,
        onset_density,
    }
}

fn decompose_tonal_noise(fp: &SpectralFingerprint, analysis: &AudioAnalysis) -> TonalNoiseDecomp {
    TonalNoiseDecomp {
        tonal_ratio: fp.tonal_ratio,
        noise_ratio: fp.noise_ratio,
        harmonic_strength: if analysis.has_pitch { fp.tonal_ratio * 0.8 } else { fp.tonal_ratio * 0.2 },
        inharmonic_strength: if analysis.has_pitch { fp.noise_ratio * 0.3 } else { fp.noise_ratio * 0.6 },
        residual_noise: fp.spectral_flatness * 0.5,
        tonal_peaks: fp.resonance_peaks.clone(),
    }
}

fn predict_impact(analysis: &AudioAnalysis, fp: &SpectralFingerprint) -> ImpactPrediction {
    let crest = if analysis.rms > 0.0 { analysis.peak / analysis.rms } else { 1.0 };
    let punch = (crest / 12.0).min(1.0) * 0.5 + analysis.transient_strength * 0.3 + fp.sub_energy * 0.2;
    let weight = (analysis.rms * 5.0).min(1.0) * 0.4 + fp.sub_energy * 0.3 + fp.low_mid_energy * 0.3;
    let tbr = if analysis.rms > 0.0 && crest > 3.0 {
        (crest / 20.0).min(1.0)
    } else {
        0.1
    };
    let perceived = (analysis.loudness_lufs.abs() / 30.0).min(1.0).max(0.0) * 0.5 + analysis.peak * 0.3 + analysis.rms * 0.2;
    let impact = (punch * 0.4 + weight * 0.3 + tbr * 0.2 + fp.sub_energy * 0.1).min(1.0);

    ImpactPrediction {
        impact_score: impact,
        punch_factor: punch.min(1.0),
        weight: weight.min(1.0),
        transient_to_body_ratio: tbr,
        perceived_loudness: perceived.min(1.0),
        sub_energy: fp.sub_energy,
    }
}

fn estimate_mix_role(role: &GenreRole, analysis: &AudioAnalysis) -> MixRole {
    match role {
        GenreRole::KickDrum => {
            if analysis.decay_ms > 200.0 { MixRole::Foundation }
            else { MixRole::Backbone }
        }
        GenreRole::SnareDrum => MixRole::Backbone,
        GenreRole::ClosedHat | GenreRole::OpenHat => MixRole::Groove,
        GenreRole::Clap => MixRole::Accent,
        GenreRole::Tom => MixRole::Fill,
        GenreRole::Percussion => {
            if analysis.duration_ms > 200.0 { MixRole::Groove }
            else { MixRole::Accent }
        }
        GenreRole::BassHit | GenreRole::Synth => MixRole::TonalElement,
        GenreRole::FxSound => MixRole::Effect,
        GenreRole::Texture => MixRole::Texture,
        GenreRole::Vocal => MixRole::TonalElement,
        GenreRole::Hybrid => MixRole::Unknown,
        GenreRole::Unknown => MixRole::Unknown,
    }
}

fn compute_spectral_complexity(fp: &SpectralFingerprint, _analysis: &AudioAnalysis) -> SpectralComplexity {
    let flatness = fp.spectral_flatness;
    let hnr = fp.tonal_ratio / fp.noise_ratio.max(0.01);
    let band_entropy = if fp.bands.len() >= 4 {
        let total: f32 = fp.bands.iter().sum();
        if total > 0.0 {
            -fp.bands.iter().filter(|&&v| v > 0.0)
                .map(|&v| { let p = v / total; p * p.log2() })
                .sum::<f32>() / fp.bands.len() as f32
        } else { 0.0 }
    } else { 0.0 };

    let spread = (fp.centroid / 5000.0).min(1.0);
    let complexity = (flatness * 0.2 + (1.0 - (hnr - 1.0).abs().min(1.0)) * 0.3
        + band_entropy * 0.2 + spread * 0.15 + fp.harshness * 0.15).min(1.0);
    let uniformity = 1.0 - band_entropy;

    SpectralComplexity {
        spectral_flatness: flatness,
        harmonic_noise_ratio: hnr.min(10.0),
        band_energy_entropy: band_entropy,
        spectral_spread: spread,
        complexity_score: complexity,
        uniformity,
    }
}

fn compute_confidence(role: &GenreRole, tc: &TransientClassification, _fp: &SpectralFingerprint) -> f32 {
    let role_conf = match role {
        GenreRole::Unknown => 0.3,
        GenreRole::Hybrid => 0.5,
        _ => 0.8,
    };
    let transient_conf = match tc.primary_type {
        TransientType::None => 0.4,
        _ => 0.8,
    };
    (role_conf * 0.5f32 + transient_conf * 0.5f32).min(1.0f32)
}

pub fn score_for_ranking(analysis: &AudioAnalysis, samples: &[f32]) -> f32 {
    let report = analyze_intelligence(samples, analysis);
    let impact = report.impact_prediction.impact_score;
    let clarity = 1.0 - report.tonal_noise_decomposition.residual_noise;
    let role_confidence = match report.genre_role {
        GenreRole::Unknown => 0.3,
        _ => 0.8,
    };
    let complexity_bonus = if report.spectral_complexity.complexity_score > 0.3 { 0.1 } else { 0.0 };
    let mix_value = match report.mix_role {
        MixRole::Foundation | MixRole::Backbone => 0.15,
        MixRole::Groove | MixRole::TonalElement => 0.1,
        _ => 0.05,
    };
    (impact * 0.3 + clarity * 0.2 + role_confidence * 0.2 + complexity_bonus + mix_value).min(1.0)
}

pub fn recommendation_score(analysis: &AudioAnalysis, samples: &[f32]) -> f32 {
    let report = analyze_intelligence(samples, analysis);
    let quality = score_for_ranking(analysis, samples);
    let transient = report.transient_classification.attack_sharpness;
    let tbr = report.impact_prediction.transient_to_body_ratio;
    let has_usable_transient = transient > 0.4 && tbr > 0.15;
    let good_duration = analysis.duration_ms > 50.0 && analysis.duration_ms < 5000.0;
    let no_clipping = !analysis.has_clipping;
    let not_silent = !analysis.is_silent;

    let usability = if not_silent && no_clipping && good_duration { 0.3 } else { 0.0 };
    let transient_bonus = if has_usable_transient { 0.2 } else { 0.0 };

    (quality * 0.5 + usability + transient_bonus).min(1.0)
}

// ─── Weeks 225-228: Advanced Sound Inference ─────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SoundInference {
    pub aggressiveness: f32,
    pub brightness: f32,
    pub envelope_type: EnvelopeType,
    pub transient_style: TransientStyle,
    pub tonal_noise_balance: f32,
    pub genre_affinity: Vec<(String, f32)>,
    pub mix_role: MixRole,
    pub suggested_genres: Vec<String>,
    pub sound_character: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum EnvelopeType {
    FastDecay,
    SlowDecay,
    LongRelease,
    Sustained,
    Percussive,
    Ambient,
    Pulsing,
    Complex,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TransientStyle {
    SharpClick,
    SoftTap,
    NoiseBlast,
    Layered,
    RimShot,
    SubPunch,
    TonalHit,
    MultiHit,
    None,
}

pub fn infer_sound_character(samples: &[f32], analysis: &AudioAnalysis) -> SoundInference {
    let fp = if samples.len() > 256 {
        extract_fingerprint(samples)
    } else {
        return SoundInference {
            aggressiveness: 0.5, brightness: analysis.brightness,
            envelope_type: EnvelopeType::FastDecay,
            transient_style: TransientStyle::None,
            tonal_noise_balance: 0.5,
            genre_affinity: Vec::new(),
            mix_role: MixRole::Unknown,
            suggested_genres: Vec::new(),
            sound_character: Vec::new(),
        };
    };

    let crest = if analysis.rms > 0.0 { analysis.peak / analysis.rms } else { 1.0 };
    let aggressiveness = ((crest / 15.0).min(1.0) * 0.4 + analysis.transient_strength.min(10.0) / 10.0 * 0.3
        + fp.harshness * 0.2 + (analysis.zero_crossing_rate / 0.3).min(1.0) * 0.1).min(1.0);

    let envelope_type = if analysis.attack_ms < 2.0 && analysis.decay_ms < 80.0 && analysis.tail_ms < 50.0 {
        EnvelopeType::Percussive
    } else if analysis.attack_ms < 3.0 && analysis.decay_ms < 200.0 && analysis.tail_ms < 100.0 {
        EnvelopeType::FastDecay
    } else if analysis.decay_ms > 300.0 && analysis.tail_ms < 200.0 {
        EnvelopeType::SlowDecay
    } else if analysis.tail_ms > 200.0 && analysis.duration_ms > 500.0 {
        EnvelopeType::LongRelease
    } else if analysis.duration_ms > 1000.0 && analysis.transient_count <= 1 {
        EnvelopeType::Sustained
    } else if analysis.transient_count > 3 && analysis.duration_ms > 300.0 {
        EnvelopeType::Complex
    } else if analysis.transient_count > 1 && analysis.duration_ms < 500.0 {
        EnvelopeType::Pulsing
    } else {
        EnvelopeType::Ambient
    };

    let transient_style = {
        let count = analysis.transient_count;
        let atk = analysis.attack_ms;
        let _zcr = analysis.zero_crossing_rate;
        if count > 4 { TransientStyle::MultiHit }
        else if count >= 3 && analysis.noise_estimate > 0.4 { TransientStyle::Layered }
        else if crest > 10.0 && atk < 2.0 && fp.centroid > 3000.0 { TransientStyle::SharpClick }
        else if crest > 8.0 && atk < 3.0 && analysis.noise_estimate > 0.5 { TransientStyle::NoiseBlast }
        else if crest > 8.0 && atk < 5.0 && fp.sub_energy > 0.3 { TransientStyle::SubPunch }
        else if crest > 6.0 && atk < 5.0 && analysis.has_pitch { TransientStyle::TonalHit }
        else if crest > 5.0 && atk < 8.0 { TransientStyle::RimShot }
        else if crest > 3.0 { TransientStyle::SoftTap }
        else { TransientStyle::None }
    };

    let tonal_noise_balance = analysis.noise_estimate;

    let mut character_tags: Vec<String> = Vec::new();
    if aggressiveness > 0.6 { character_tags.push("aggressive".to_string()); }
    else if aggressiveness < 0.3 { character_tags.push("gentle".to_string()); }
    if analysis.brightness > 0.6 { character_tags.push("bright".to_string()); }
    else if analysis.brightness < 0.3 { character_tags.push("dark".to_string()); }
    if crest > 10.0 { character_tags.push("punchy".to_string()); }
    if fp.sub_energy > 0.3 { character_tags.push("sub-heavy".to_string()); }
    if fp.muddiness > 0.5 { character_tags.push("muddy".to_string()); }
    if fp.harshness > 0.5 { character_tags.push("harsh".to_string()); }
    if analysis.noise_estimate > 0.6 { character_tags.push("noisy".to_string()); }
    else if analysis.noise_estimate < 0.2 { character_tags.push("clean".to_string()); }
    if fp.spectral_flatness > 0.6 { character_tags.push("textured".to_string()); }
    if analysis.duration_ms > 1000.0 { character_tags.push("long".to_string()); }
    else if analysis.duration_ms < 150.0 { character_tags.push("short".to_string()); }

    let mut genre_affinity: Vec<(String, f32)> = Vec::new();
    if analysis.sub_energy_ratio > 0.3 || fp.sub_energy > 0.3 {
        genre_affinity.push(("trap".to_string(), 0.7));
        genre_affinity.push(("drill".to_string(), 0.6));
        genre_affinity.push(("dubstep".to_string(), 0.5));
    }
    if analysis.brightness > 0.6 && analysis.noise_estimate > 0.5 {
        genre_affinity.push(("techno".to_string(), 0.6));
        genre_affinity.push(("house".to_string(), 0.5));
    }
    if analysis.brightness < 0.3 && analysis.noise_estimate > 0.3 {
        genre_affinity.push(("lo-fi".to_string(), 0.7));
        genre_affinity.push(("ambient".to_string(), 0.5));
    }
    if analysis.transient_count > 3 && analysis.duration_ms < 300.0 {
        genre_affinity.push(("dnb".to_string(), 0.5));
        genre_affinity.push(("breakbeat".to_string(), 0.5));
    }
    if analysis.duration_ms > 1500.0 && analysis.sub_energy_ratio > 0.2 {
        genre_affinity.push(("cinematic".to_string(), 0.7));
        genre_affinity.push(("orchestral".to_string(), 0.4));
    }
    if aggressiveness > 0.7 && fp.harshness > 0.3 {
        genre_affinity.push(("phonk".to_string(), 0.5));
        genre_affinity.push(("metal".to_string(), 0.4));
    }
    genre_affinity.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let suggested_genres: Vec<String> = genre_affinity.iter().take(3).map(|(g, _)| g.clone()).collect();

    let mix_role = estimate_mix_role_from_inference(&envelope_type, analysis, &fp);

    SoundInference {
        aggressiveness,
        brightness: analysis.brightness,
        envelope_type,
        transient_style,
        tonal_noise_balance,
        genre_affinity,
        mix_role,
        suggested_genres,
        sound_character: character_tags,
    }
}

fn estimate_mix_role_from_inference(env: &EnvelopeType, analysis: &AudioAnalysis, fp: &SpectralFingerprint) -> MixRole {
    match env {
        EnvelopeType::Percussive => {
            if fp.sub_energy > 0.3 { MixRole::Foundation }
            else { MixRole::Accent }
        }
        EnvelopeType::FastDecay => {
            if analysis.transient_strength > 3.0 { MixRole::Backbone }
            else { MixRole::Groove }
        }
        EnvelopeType::SlowDecay => {
            if analysis.transient_count > 1 { MixRole::Fill }
            else { MixRole::TonalElement }
        }
        EnvelopeType::LongRelease => {
            if analysis.noise_estimate > 0.5 { MixRole::Texture }
            else { MixRole::Effect }
        }
        EnvelopeType::Sustained => MixRole::TonalElement,
        EnvelopeType::Ambient => MixRole::Texture,
        EnvelopeType::Pulsing => MixRole::Groove,
        EnvelopeType::Complex => MixRole::Fill,
    }
}

// ─── Genre Intelligence ──────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GenreIntelligence {
    pub primary_genre: String,
    pub genre_confidence: f32,
    pub genre_candidates: Vec<(String, f32)>,
    pub genre_specific: GenreSpecificProfile,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GenreSpecificProfile {
    pub expected_transient_behavior: String,
    pub expected_tail_style: String,
    pub expected_saturation: String,
    pub expected_envelope: String,
    pub expected_variation_style: String,
}

pub fn analyze_genre_intelligence(analysis: &AudioAnalysis) -> GenreIntelligence {
    let candidates = compute_genre_candidates(analysis);
    let primary = candidates.first().map(|(g, _)| g.clone()).unwrap_or_else(|| "other".to_string());
    let confidence = candidates.first().map(|(_, c)| *c).unwrap_or(0.0);
    let profile = compute_genre_profile(&primary);
    GenreIntelligence {
        primary_genre: primary,
        genre_confidence: confidence,
        genre_candidates: candidates,
        genre_specific: profile,
    }
}

fn compute_genre_candidates(analysis: &AudioAnalysis) -> Vec<(String, f32)> {
    let mut candidates = Vec::new();
    let _sc = analysis.spectral_centroid;
    let noise = analysis.noise_estimate;
    let sub = analysis.sub_energy_ratio;
    let attack = analysis.attack_ms;
    let crest = analysis.crest_factor;
    let brightness = analysis.brightness;
    let _transient = analysis.transient_strength;
    let zcr = analysis.zero_crossing_rate;
    let duration = analysis.duration_ms;
    let trans_count = analysis.transient_count;

    match analysis.sound_type_hint.as_str() {
        "kick" | "bass" => {
            if sub > 0.4 && attack < 3.0 && crest > 10.0 {
                candidates.push(("trap".to_string(), 0.8));
                candidates.push(("drill".to_string(), 0.7));
            }
            if sub > 0.3 && attack < 5.0 && brightness < 0.4 {
                candidates.push(("house".to_string(), 0.75));
            }
            if sub > 0.5 && duration > 500.0 && crest < 8.0 {
                candidates.push(("cinematic".to_string(), 0.7));
            }
            if attack < 2.0 && sub > 0.2 && brightness > 0.3 {
                candidates.push(("techno".to_string(), 0.7));
            }
            if sub > 0.35 && attack < 4.0 && crest > 9.0 {
                candidates.push(("jersey".to_string(), 0.6));
            }
            if brightness > 0.5 && attack < 3.0 && noise < 0.3 {
                candidates.push(("pop".to_string(), 0.65));
            }
            if sub > 0.3 && crest > 12.0 && attack < 2.0 {
                candidates.push(("rage".to_string(), 0.6));
            }
            if noise > 0.3 && crest > 8.0 {
                candidates.push(("industrial".to_string(), 0.55));
            }
        }
        "snare" | "clap" => {
            if attack < 2.0 && crest > 12.0 && noise > 0.3 {
                candidates.push(("trap".to_string(), 0.75));
                candidates.push(("drill".to_string(), 0.7));
            }
            if noise > 0.5 && brightness > 0.6 && trans_count > 2 {
                candidates.push(("house".to_string(), 0.65));
            }
            if brightness > 0.7 && attack < 3.0 && noise > 0.4 {
                candidates.push(("pop".to_string(), 0.6));
            }
            if noise > 0.6 && trans_count > 3 {
                candidates.push(("hyperpop".to_string(), 0.55));
            }
            if attack < 3.0 && crest > 10.0 {
                candidates.push(("jersey".to_string(), 0.6));
            }
            if noise > 0.5 && brightness > 0.5 && attack < 4.0 {
                candidates.push(("techno".to_string(), 0.55));
            }
            if noise > 0.6 && crest > 10.0 {
                candidates.push(("rage".to_string(), 0.5));
            }
        }
        "closed_hat" | "open_hat" => {
            if brightness > 0.7 && zcr > 0.2 && duration < 300.0 {
                candidates.push(("trap".to_string(), 0.7));
                candidates.push(("drill".to_string(), 0.65));
            }
            if zcr > 0.25 && brightness > 0.8 && noise > 0.6 {
                candidates.push(("techno".to_string(), 0.8));
            }
            if brightness > 0.6 && zcr > 0.15 && noise > 0.4 {
                candidates.push(("house".to_string(), 0.7));
            }
            if zcr > 0.3 && brightness > 0.8 {
                candidates.push(("hyperpop".to_string(), 0.5));
            }
            if noise > 0.5 && zcr > 0.2 {
                candidates.push(("jersey".to_string(), 0.5));
            }
        }
        "fx" => {
            if duration > 1000.0 && noise > 0.3 {
                candidates.push(("cinematic".to_string(), 0.8));
            }
            if noise > 0.5 && brightness > 0.5 {
                candidates.push(("ambient".to_string(), 0.6));
            }
            if analysis.transient_strength > 5.0 {
                candidates.push(("industrial".to_string(), 0.5));
            }
        }
        _ => {
            candidates.push(("pop".to_string(), 0.4));
            candidates.push(("other".to_string(), 0.3));
        }
    }

    // Normalize scores
    let total: f32 = candidates.iter().map(|(_, s)| s).sum();
    if total > 0.0 {
        for (_, s) in candidates.iter_mut() {
            *s /= total;
        }
    }
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates
}

fn compute_genre_profile(genre: &str) -> GenreSpecificProfile {
    match genre {
        "trap" => GenreSpecificProfile {
            expected_transient_behavior: "sharp, fast attack, high crest".to_string(),
            expected_tail_style: "short, tight, minimal decay".to_string(),
            expected_saturation: "warm tape, moderate saturation".to_string(),
            expected_envelope: "fast attack, quick decay, no sustain".to_string(),
            expected_variation_style: "pitch variation, sub-heavy variants".to_string(),
        },
        "drill" => GenreSpecificProfile {
            expected_transient_behavior: "very sharp, aggressive attack, high crest".to_string(),
            expected_tail_style: "very short, tight, gated".to_string(),
            expected_saturation: "heavy saturation, distorted character".to_string(),
            expected_envelope: "extremely fast attack, rapid decay".to_string(),
            expected_variation_style: "pitch down, sub-enhanced variants".to_string(),
        },
        "hyperpop" => GenreSpecificProfile {
            expected_transient_behavior: "exaggerated sharpness, bright attack".to_string(),
            expected_tail_style: "extremely short, often gated".to_string(),
            expected_saturation: "heavy clipping, bright distortion".to_string(),
            expected_envelope: "fast attack, extremely short".to_string(),
            expected_variation_style: "extreme pitch shifts, bright variants".to_string(),
        },
        "house" => GenreSpecificProfile {
            expected_transient_behavior: "clean, punchy, consistent attack".to_string(),
            expected_tail_style: "medium decay, tight tail".to_string(),
            expected_saturation: "light warmth, clean saturation".to_string(),
            expected_envelope: "consistent, reliable envelope".to_string(),
            expected_variation_style: "rhythmic variation, groove-oriented".to_string(),
        },
        "techno" => GenreSpecificProfile {
            expected_transient_behavior: "sharp, industrial, driving attack".to_string(),
            expected_tail_style: "short, dry, tight".to_string(),
            expected_saturation: "heavy distortion, aggressive clipping".to_string(),
            expected_envelope: "fast attack, minimal decay".to_string(),
            expected_variation_style: "saturation variation, filter sweeps".to_string(),
        },
        "cinematic" => GenreSpecificProfile {
            expected_transient_behavior: "broad, impactful, dramatic attack".to_string(),
            expected_tail_style: "long, evolving, textured decay".to_string(),
            expected_saturation: "warm compression, gentle saturation".to_string(),
            expected_envelope: "long attack, slow decay, extended tail".to_string(),
            expected_variation_style: "reverberant, sub-heavy, expansive variants".to_string(),
        },
        "pop" => GenreSpecificProfile {
            expected_transient_behavior: "clean, polished, bright attack".to_string(),
            expected_tail_style: "controlled decay, clean tail".to_string(),
            expected_saturation: "light, clean, transparent".to_string(),
            expected_envelope: "balanced, polished, radio-ready".to_string(),
            expected_variation_style: "brightness variation, clean variants".to_string(),
        },
        "jersey" => GenreSpecificProfile {
            expected_transient_behavior: "sharp, pitched-down attack".to_string(),
            expected_tail_style: "short, tight, gated".to_string(),
            expected_saturation: "moderate saturation, warm".to_string(),
            expected_envelope: "fast attack, quick decay".to_string(),
            expected_variation_style: "pitch-down, sub-enhanced".to_string(),
        },
        "rage" => GenreSpecificProfile {
            expected_transient_behavior: "aggressive, loud, saturated attack".to_string(),
            expected_tail_style: "short, punchy, minimal".to_string(),
            expected_saturation: "heavy, distorted, limiting".to_string(),
            expected_envelope: "fast attack, loud throughout".to_string(),
            expected_variation_style: "distortion-heavy, aggressive variants".to_string(),
        },
        "industrial" => GenreSpecificProfile {
            expected_transient_behavior: "harsh, metallic, noisy attack".to_string(),
            expected_tail_style: "abrasive, textured, unpredictable".to_string(),
            expected_saturation: "extreme distortion, harsh clipping".to_string(),
            expected_envelope: "unconventional, asymmetric".to_string(),
            expected_variation_style: "noise-heavy, distortion-focused variants".to_string(),
        },
        "ambient" | "ambient_perc" => GenreSpecificProfile {
            expected_transient_behavior: "soft, rounded, subtle attack".to_string(),
            expected_tail_style: "long, evolving, ethereal decay".to_string(),
            expected_saturation: "minimal, gentle warmth".to_string(),
            expected_envelope: "slow attack, very long release".to_string(),
            expected_variation_style: "texture-focused, spatial variants".to_string(),
        },
        "ui" | "ui_game" => GenreSpecificProfile {
            expected_transient_behavior: "crisp, immediate, clean attack".to_string(),
            expected_tail_style: "extremely short, no tail".to_string(),
            expected_saturation: "none or minimal".to_string(),
            expected_envelope: "instant attack, instant release".to_string(),
            expected_variation_style: "pitch variation, click-focused".to_string(),
        },
        _ => GenreSpecificProfile {
            expected_transient_behavior: "balanced, adaptive".to_string(),
            expected_tail_style: "adaptive to source".to_string(),
            expected_saturation: "adaptive".to_string(),
            expected_envelope: "adaptive".to_string(),
            expected_variation_style: "balanced variation".to_string(),
        },
    }
}
