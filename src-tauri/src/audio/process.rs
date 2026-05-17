use super::{DspParams, SoundType};
use super::dsp;

pub fn trim_silence(samples: &mut Vec<f32>, threshold: f32) {
    if samples.is_empty() { return; }
    let start = samples.iter().position(|&s| s.abs() > threshold).unwrap_or(0);
    let end = samples
        .iter()
        .rposition(|&s| s.abs() > threshold)
        .map(|pos| pos + 1)
        .unwrap_or(samples.len());
    if start < end && end <= samples.len() {
        *samples = samples[start..end].to_vec();
    }
}

pub fn normalize_peak(samples: &mut [f32], target_db: f32) {
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak > 0.0 {
        let target_peak = 10.0f32.powf(target_db / 20.0);
        let gain = target_peak / peak;
        if gain > 0.0 && gain < 100.0 {
            for sample in samples.iter_mut() {
                *sample *= gain;
            }
        }
    }
}

pub fn apply_fade(samples: &mut [f32], fade_in_s: f32, fade_out_s: f32) {
    dsp::apply_smooth_fade(samples, fade_in_s, fade_out_s);
}

pub fn shorten_duration(samples: &mut Vec<f32>, _sample_rate: u32, fraction: f32) {
    if samples.is_empty() || fraction <= 0.0 { return; }
    let new_len = (samples.len() as f32 * fraction.clamp(0.1, 0.95)) as usize;
    if new_len < samples.len() && new_len > 0 {
        samples.truncate(new_len);
    }
}

pub fn brighten(samples: &mut [f32]) {
    dsp::biquad_high_shelf(samples, 3000.0, 3.0, 0.7);
    for sample in samples.iter_mut() {
        let high = *sample * 1.2;
        *sample = (*sample + high.clamp(-1.0, 1.0)) * 0.5;
    }
}

pub fn darken(samples: &mut [f32], _sample_rate: u32) {
    dsp::biquad_low_shelf(samples, 300.0, -3.0, 0.7);
}

pub fn has_nan_or_inf(samples: &[f32]) -> bool {
    samples.iter().any(|s| s.is_nan() || s.is_infinite())
}

pub fn validate_audio_integrity(samples: &[f32]) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Audio buffer is empty".to_string());
    }
    if has_nan_or_inf(samples) {
        return Err("Audio contains NaN or Inf values".to_string());
    }
    Ok(())
}

pub fn process_sound(
    samples: &mut Vec<f32>,
    dsp_params: &DspParams,
    sound_type: SoundType,
) -> Vec<String> {
    validate_audio_integrity(samples).ok();

    // 1. Remove DC offset + subsonic filter
    dsp::remove_dc_offset(samples);
    dsp::subsonic_filter(samples);

    // 2. Trim silence (generous threshold)
    trim_silence(samples, 0.001);

    // 3. Apply smooth fades (anti-click)
    dsp::apply_smooth_fade(samples, 0.002, 0.005);

    // 4. Spectral balance EQ (mud cut, presence boost, air)
    if samples.len() > 64 {
        dsp::spectral_balance(samples, sound_type.as_str());
    }

    // 5. Apply prompt-based EQ
    dsp::apply_eq(samples, dsp_params);

    // 6. Transient enhancement (improved shaping)
    let transient_boost = match sound_type {
        SoundType::Kick | SoundType::Snare | SoundType::Clap => {
            if dsp_params.punch { 4.0 } else { 2.5 }
        }
        SoundType::ClosedHat | SoundType::OpenHat | SoundType::Tom | SoundType::Perc => {
            if dsp_params.punch { 3.0 } else { 1.5 }
        }
        _ => 0.0,
    };
    if transient_boost > 0.0 {
        dsp::multiband_transient_shape(samples, transient_boost);
    }
    if dsp_params.punch {
        dsp::apply_punch(samples);
    }

    // 7. Apply gain with limiting
    let safe_gain = dsp_params.gain.min(3.0);
    if (safe_gain - 1.0).abs() > 0.01 {
        for sample in samples.iter_mut() {
            *sample *= safe_gain;
        }
    }

    // 8. Adaptive compression for punch
    dsp::adaptive_compressor(samples, -12.0, 3.0, 1.0, 50.0);

    // 9. Noise gate tail
    dsp::noise_gate_tail(samples, -55.0);

    // 10. Anti-click de-click
    dsp::de_click(samples, 0.5);

    // 11. Normalize peak with headroom
    normalize_peak(samples, -1.0);

    // 12. Lookahead limiter for clean ceiling
    dsp::lookahead_limiter(samples, -0.5, 1.0);

    // 13. Final smooth fade
    dsp::apply_smooth_fade(samples, 0.001, 0.002);

    // 14. Final cleanup
    trim_silence(samples, 0.0001);

    if has_nan_or_inf(samples) {
        samples.fill(0.0);
    }

    let tags = super::analyze::apply_autotags(samples, &sound_type, None, None);
    tags
}

// ─── High-Quality Recreation Processing Chain ────────────

pub fn process_recreation(samples: &mut Vec<f32>, sound_type: &SoundType) {
    validate_audio_integrity(samples).ok();

    dsp::remove_dc_offset(samples);
    dsp::subsonic_filter(samples);

    trim_silence(samples, 0.001);
    dsp::apply_smooth_fade(samples, 0.001, 0.003);

    if samples.len() > 64 {
        dsp::spectral_balance(samples, sound_type.as_str());
    }

    let boost = match sound_type {
        SoundType::Kick => 3.0,
        SoundType::Snare => 2.0,
        SoundType::Clap => 2.0,
        SoundType::ClosedHat | SoundType::OpenHat => 1.0,
        _ => 0.0,
    };
    if boost > 0.0 {
        dsp::multiband_transient_shape(samples, boost);
    }

    dsp::adaptive_compressor(samples, -15.0, 2.5, 1.0, 50.0);
    dsp::noise_gate_tail(samples, -55.0);
    dsp::de_click(samples, 0.3);

    normalize_peak(samples, -1.0);
    dsp::lookahead_limiter(samples, -0.5, 1.0);
    dsp::apply_smooth_fade(samples, 0.001, 0.002);
    trim_silence(samples, 0.0001);

    if has_nan_or_inf(samples) {
        samples.fill(0.0);
    }
}

// ─── Perfection Workflow Utilities ────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MicroAdjustment {
    pub param: String,
    pub delta: f32,
    pub description: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ABComparison {
    pub version_a_label: String,
    pub version_b_label: String,
    pub version_a_waveform: Vec<f32>,
    pub version_b_waveform: Vec<f32>,
    pub version_a_analysis: String,
    pub version_b_analysis: String,
    pub version_a_rms: f32,
    pub version_b_rms: f32,
    pub version_a_peak: f32,
    pub version_b_peak: f32,
    pub version_a_crest: f32,
    pub version_b_crest: f32,
    pub differences: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExportAssessment {
    pub confidence: f32,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
    pub gain_staging: GainStageInfo,
    pub export_ready: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GainStageInfo {
    pub peak_db: f32,
    pub rms_db: f32,
    pub crest_db: f32,
    pub headroom_db: f32,
    pub recommended_gain_db: f32,
    pub has_clipping: bool,
    pub is_too_quiet: bool,
    pub dynamic_range: f32,
}

pub fn apply_micro_adjustments(samples: &mut Vec<f32>, adjustments: &[MicroAdjustment]) {
    for adj in adjustments {
        match adj.param.as_str() {
            "gain" => {
                let gain_db = adj.delta;
                let gain_linear = 10.0_f32.powf(gain_db / 20.0);
                for s in samples.iter_mut() { *s *= gain_linear; }
            }
            "brightness" => {
                let tilt = adj.delta.clamp(-1.0, 1.0);
                if tilt > 0.0 {
                    dsp::biquad_high_shelf(samples, 3000.0, tilt * 4.0, 0.7);
                } else {
                    dsp::biquad_low_shelf(samples, 300.0, tilt.abs() * 3.0, 0.7);
                }
            }
            "punch" => {
                let boost = adj.delta.clamp(0.0, 6.0);
                if boost > 0.1 {
                    dsp::transient_enhance(samples, boost);
                }
            }
            "saturation" => {
                let drive = 1.0 + adj.delta.clamp(0.0, 4.0);
                if drive > 1.01 {
                    for s in samples.iter_mut() { *s = dsp::tape_saturation(*s, drive); }
                }
            }
            "sub" => {
                let amount = adj.delta.clamp(0.0, 1.0);
                if amount > 0.01 {
                    let len = samples.len();
                    for i in 0..len {
                        let t = i as f32 / super::SAMPLE_RATE as f32;
                        let env = (-2.5 * t).exp();
                        let sub = (2.0 * std::f32::consts::PI * 55.0 * t).sin() * env * amount;
                        samples[i] = (samples[i] + sub * 0.3).clamp(-1.0, 1.0);
                    }
                }
            }
            "duration" => {
                let scale = adj.delta.clamp(0.3, 3.0);
                let new_len = (samples.len() as f32 * scale) as usize;
                if new_len > 0 && new_len != samples.len() {
                    if new_len < samples.len() {
                        samples.truncate(new_len);
                    } else {
                        let extra = new_len - samples.len();
                        samples.resize(new_len, 0.0);
                    }
                }
            }
            "noise_gate" => {
                let threshold_db = adj.delta.clamp(-80.0, -20.0);
                dsp::noise_gate_tail(samples, threshold_db);
            }
            "attack" => {
                let attack_ms = adj.delta.clamp(0.1, 50.0);
                let attack_samples = (attack_ms / 1000.0 * super::SAMPLE_RATE as f32) as usize;
                let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                if peak > 0.001 {
                    let threshold = peak * 0.1;
                    let onset = samples.iter().position(|s| s.abs() > threshold).unwrap_or(0);
                    let end = (onset + attack_samples).min(samples.len());
                    for i in onset..end {
                        let t = (i - onset) as f32 / (end - onset).max(1) as f32;
                        samples[i] *= (t * (2.0 - t)).min(1.0);
                    }
                }
            }
            _ => {}
        }
    }
    normalize_peak(samples, -1.0);
    dsp::lookahead_limiter(samples, -0.5, 1.0);
}

pub fn compute_ab_comparison(
    version_a: &[f32],
    version_b: &[f32],
    label_a: &str,
    label_b: &str,
    num_waveform_points: usize,
) -> ABComparison {
    let step_a = (version_a.len() / num_waveform_points.max(1)).max(1);
    let step_b = (version_b.len() / num_waveform_points.max(1)).max(1);
    let wf_a: Vec<f32> = version_a.iter().step_by(step_a).copied().collect();
    let wf_b: Vec<f32> = version_b.iter().step_by(step_b).copied().collect();

    let rms_a = (version_a.iter().map(|s| s * s).sum::<f32>() / version_a.len().max(1) as f32).sqrt();
    let rms_b = (version_b.iter().map(|s| s * s).sum::<f32>() / version_b.len().max(1) as f32).sqrt();
    let peak_a = version_a.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let peak_b = version_b.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let crest_a = if rms_a > 0.0 { peak_a / rms_a } else { 1.0 };
    let crest_b = if rms_b > 0.0 { peak_b / rms_b } else { 1.0 };

    let analysis_a = super::analyze::analyze_audio(version_a, super::SAMPLE_RATE, 1);
    let analysis_b = super::analyze::analyze_audio(version_b, super::SAMPLE_RATE, 1);

    let mut diffs = Vec::new();
    if (analysis_a.rms - analysis_b.rms).abs() > 0.02 {
        diffs.push(format!("RMS: {:.2} vs {:.2}", analysis_a.rms, analysis_b.rms));
    }
    if (analysis_a.crest_factor - analysis_b.crest_factor).abs() > 1.0 {
        diffs.push(format!("Crest: {:.1} vs {:.1}", analysis_a.crest_factor, analysis_b.crest_factor));
    }
    if (analysis_a.brightness - analysis_b.brightness).abs() > 0.05 {
        diffs.push(format!("Brightness: {:.1}% vs {:.1}%", analysis_a.brightness * 100.0, analysis_b.brightness * 100.0));
    }
    if (analysis_a.attack_ms - analysis_b.attack_ms).abs() > 1.0 {
        diffs.push(format!("Attack: {:.1}ms vs {:.1}ms", analysis_a.attack_ms, analysis_b.attack_ms));
    }
    if (analysis_a.transient_strength - analysis_b.transient_strength).abs() > 0.5 {
        diffs.push(format!("Transient: {:.1} vs {:.1}", analysis_a.transient_strength, analysis_b.transient_strength));
    }
    if (analysis_a.sub_energy_ratio - analysis_b.sub_energy_ratio).abs() > 0.05 {
        diffs.push(format!("Sub energy: {:.1}% vs {:.1}%", analysis_a.sub_energy_ratio * 100.0, analysis_b.sub_energy_ratio * 100.0));
    }
    if (analysis_a.noise_estimate - analysis_b.noise_estimate).abs() > 0.05 {
        diffs.push(format!("Noise: {:.1}% vs {:.1}%", analysis_a.noise_estimate * 100.0, analysis_b.noise_estimate * 100.0));
    }

    ABComparison {
        version_a_label: label_a.to_string(),
        version_b_label: label_b.to_string(),
        version_a_waveform: wf_a,
        version_b_waveform: wf_b,
        version_a_analysis: serde_json::to_string(&analysis_a).unwrap_or_default(),
        version_b_analysis: serde_json::to_string(&analysis_b).unwrap_or_default(),
        version_a_rms: analysis_a.rms,
        version_b_rms: analysis_b.rms,
        version_a_peak: analysis_a.peak,
        version_b_peak: analysis_b.peak,
        version_a_crest: analysis_a.crest_factor,
        version_b_crest: analysis_b.crest_factor,
        differences: diffs,
    }
}

pub fn assess_export_readiness(samples: &[f32], target_headroom_db: f32) -> ExportAssessment {
    let analysis = super::analyze::analyze_audio(samples, super::SAMPLE_RATE, 1);
    let peak_db = if analysis.peak > 0.0 { 20.0 * analysis.peak.log10() } else { -90.0 };
    let rms_db = if analysis.rms > 0.0 { 20.0 * analysis.rms.log10() } else { -90.0 };
    let crest_db = peak_db - rms_db;
    let headroom = target_headroom_db - peak_db.max(target_headroom_db - 12.0);
    let has_clip = analysis.has_clipping;
    let is_quiet = analysis.rms < 0.05;

    let recommended_gain = if is_quiet {
        (-12.0 - rms_db).clamp(0.0, 24.0)
    } else if peak_db > target_headroom_db {
        (target_headroom_db - peak_db).clamp(-12.0, 0.0)
    } else {
        0.0
    };

    let dynamic_range = crest_db;

    let mut warnings = Vec::new();
    if has_clip { warnings.push("Signal has clipping".to_string()); }
    if is_quiet { warnings.push("Signal is very quiet".to_string()); }
    if analysis.is_silent { warnings.push("Signal appears silent".to_string()); }
    if analysis.duration_ms < 20.0 { warnings.push("Very short sound".to_string()); }

    let mut recommendations = Vec::new();
    if has_clip { recommendations.push("Reduce gain or apply limiting".to_string()); }
    if is_quiet { recommendations.push(format!("Boost gain by {:.0} dB", recommended_gain)); }
    if analysis.crest_factor < 4.0 { recommendations.push("Sound may lack dynamic punch".to_string()); }
    if analysis.crest_factor > 20.0 { recommendations.push("Very dynamic - consider compression".to_string()); }
    if analysis.noise_floor_db > -40.0 { recommendations.push("High noise floor detected".to_string()); }

    let confidence = if has_clip || analysis.is_silent {
        0.2
    } else if is_quiet || analysis.duration_ms < 20.0 {
        0.5
    } else {
        0.9 - (analysis.noise_estimate * 0.2).min(0.3)
    }.clamp(0.0, 1.0);

    ExportAssessment {
        confidence,
        warnings,
        recommendations,
        gain_staging: GainStageInfo {
            peak_db,
            rms_db,
            crest_db,
            headroom_db: headroom,
            recommended_gain_db: recommended_gain,
            has_clipping: has_clip,
            is_too_quiet: is_quiet,
            dynamic_range,
        },
        export_ready: !has_clip && !analysis.is_silent && !is_quiet,
    }
}

pub fn selective_regenerate(
    samples: &mut Vec<f32>,
    region: &str,
    params: &crate::audio::resynthesize::ResynthesisParams,
) {
    let num_samples = samples.len();
    let (start, end) = match region {
        "attack" => (0, (num_samples as f32 * 0.1) as usize),
        "body" => ((num_samples as f32 * 0.1) as usize, (num_samples as f32 * 0.6) as usize),
        "tail" => ((num_samples as f32 * 0.6) as usize, num_samples),
        _ => return,
    };
    if start >= end || end > num_samples { return; }

    let full = crate::audio::resynthesize::resynthesize(params);
    let len = full.len().min(num_samples);
    for i in start..end.min(len) {
        samples[i] = full[i];
    }
    normalize_peak(samples, -1.0);
    dsp::lookahead_limiter(samples, -0.5, 1.0);
}
