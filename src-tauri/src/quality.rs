use crate::audio::{self, SoundType, SAMPLE_RATE};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct QualityMetadata {
    pub duration_ms: f32,
    pub peak: f32,
    pub rms: f32,
    pub clipping_detected: bool,
    pub clipping_percent: f32,
    pub silence_trimmed: bool,
    pub transform_type: String,
    pub is_silent: bool,
    pub is_too_quiet: bool,
    pub duration_appropriate: bool,
    // Enhanced quality metrics
    pub spectral_quality: f32,      // 0.0-1.0 (balanced spectrum)
    pub transient_quality: f32,     // 0.0-1.0 (clean transient definition)
    pub tonal_clarity: f32,         // 0.0-1.0 (clear pitch content)
    pub noise_floor_quality: f32,   // 0.0-1.0 (low noise floor)
    pub dynamic_range: f32,         // 0.0-1.0 (good crest factor)
    pub spectral_balance: f32,      // 0.0-1.0 (no extreme imbalances)
    pub punch_quality: f32,         // 0.0-1.0 (transient-to-body ratio)
}

pub fn compute_quality(
    samples: &[f32],
    sound_type: SoundType,
    transform: &str,
    was_trimmed: bool,
) -> QualityMetadata {
    let duration_ms = samples.len() as f32 / SAMPLE_RATE as f32 * 1000.0;
    let rms = audio::compute_rms(samples);
    let peak = audio::compute_peak(samples);

    let clipped_count = samples.iter().filter(|&&s| s.abs() >= 1.0).count();
    let clipping_percent = if samples.is_empty() {
        0.0
    } else {
        clipped_count as f32 / samples.len() as f32 * 100.0
    };
    let clipping_detected = peak >= 0.99 || clipping_percent > 0.05;

    let is_silent = rms < 0.001;
    let is_too_quiet = rms < 0.03 && !is_silent;

    let duration_appropriate = match sound_type {
        SoundType::Kick => duration_ms >= 80.0 && duration_ms <= 1500.0,
        SoundType::Snare => duration_ms >= 60.0 && duration_ms <= 1200.0,
        SoundType::ClosedHat => duration_ms >= 20.0 && duration_ms <= 500.0,
        SoundType::OpenHat => duration_ms >= 50.0 && duration_ms <= 1200.0,
        SoundType::Clap => duration_ms >= 60.0 && duration_ms <= 1000.0,
        SoundType::Tom => duration_ms >= 80.0 && duration_ms <= 1500.0,
        SoundType::Perc => duration_ms >= 30.0 && duration_ms <= 1500.0,
        SoundType::Bass => duration_ms >= 100.0 && duration_ms <= 3000.0,
        SoundType::Fx => duration_ms >= 50.0 && duration_ms <= 5000.0,
        SoundType::Other => duration_ms >= 20.0 && duration_ms <= 5000.0,
    };

    // Enhanced quality computation
    let crest = if rms > 0.0 { peak / rms } else { 1.0 };
    let spectral_centroid = audio::compute_spectral_centroid(samples);
    let zcr = audio::compute_zero_crossing_rate(samples);
    let sub_energy = audio::compute_energy_sub_low(samples);
    let brightness = audio::compute_brightness(samples);
    let attack_ms = audio::compute_attack_time(samples);

    let expected_range: (f32, f32) = match sound_type {
        SoundType::Kick => (200.0, 4000.0),
        SoundType::Snare => (500.0, 6000.0),
        SoundType::ClosedHat => (4000.0, 10000.0),
        SoundType::OpenHat => (2000.0, 8000.0),
        SoundType::Clap => (1000.0, 5000.0),
        SoundType::Bass => (80.0, 2000.0),
        SoundType::Tom => (300.0, 3000.0),
        SoundType::Perc => (500.0, 5000.0),
        SoundType::Fx => (200.0, 8000.0),
        SoundType::Other => (200.0, 6000.0),
    };

    let has_content = peak > 0.01;
    
    let spectral_quality = if !has_content {
        0.0
    } else {
        let mut score = 0.5f32;
        if spectral_centroid >= expected_range.0 && spectral_centroid <= expected_range.1 {
            score += 0.3;
        }
        if brightness > 0.05 && brightness < 0.95 {
            score += 0.2;
        }
        score.clamp(0.0, 1.0)
    };

    let transient_quality = {
        let mut score = 0.5f32;
        let expected_crest: f32 = match sound_type {
            SoundType::Kick => 8.0,
            SoundType::Snare => 6.0,
            SoundType::ClosedHat => 4.0,
            SoundType::Clap => 5.0,
            _ => 4.0,
        };
        if crest >= expected_crest { score += 0.3; }
        else if crest >= expected_crest * 0.5 { score += 0.1; }
        if clipping_detected { score -= 0.3; }
        if attack_ms > 0.0 && attack_ms < expected_range.1 / 1000.0 * 5.0 { score += 0.2; }
        score.clamp(0.0, 1.0)
    };

    let tonal_clarity = {
        let pitch = audio::estimate_pitch(samples, SAMPLE_RATE);
        if let Some(pitch_val) = pitch {
            let in_range = pitch_val > 40.0 && pitch_val < 2000.0;
            let low_noise = zcr < 0.08;
            let clean_harmonics = spectral_centroid / pitch_val.max(1.0) < 30.0;
            let mut score: f32 = 0.4;
            if in_range { score += 0.2; }
            if low_noise { score += 0.2; }
            if clean_harmonics { score += 0.2; }
            score
        } else {
            (1.0f32 - zcr * 1.5).clamp(0.1, 0.6)
        }
    };

    let noise_floor_quality = if peak > 0.001 {
        let tail_third_start = (samples.len() * 2 / 3).max(1);
        let tail_third = &samples[tail_third_start..];
        let tail_rms = audio::compute_rms(tail_third);
        let noise_ratio = if rms > 0.0 { (tail_rms / rms).min(1.0) } else { 1.0 };
        (1.0 - noise_ratio * 0.8).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let dynamic_range = {
        let range: f32 = (crest / 18.0).min(1.0);
        if crest < 2.0 { range * 0.3 }
        else if crest > 16.0 { 1.0 - (crest - 16.0) / 10.0 }
        else { range }
    };

    let spectral_balance = {
        let has_sub = sub_energy > 0.05;
        let has_low_mid = spectral_centroid > 200.0 && spectral_centroid < 800.0;
        let has_highs = spectral_centroid > 3000.0;
        let sub_heavy = sub_energy > 0.6;
        let bright_heavy = brightness > 0.85;
        let mut score: f32 = 0.3;
        if has_sub && has_highs { score += 0.4; }
        else if has_sub || has_highs { score += 0.2; }
        if has_low_mid { score += 0.1; }
        if sub_heavy || bright_heavy { score -= 0.1; }
        score.clamp(0.0, 1.0)
    };

    let punch_quality = {
        let expected_crest: f32 = match sound_type {
            SoundType::Kick => 10.0,
            SoundType::Snare => 8.0,
            SoundType::Clap => 6.0,
            SoundType::ClosedHat => 5.0,
            _ => 5.0,
        };
        let ratio = (crest / expected_crest).min(2.0);
        if ratio >= 0.8 && ratio <= 1.5 { 0.8 + (ratio - 1.0).abs() * 0.1 }
        else if ratio < 0.8 { (ratio / 0.8) * 0.6 }
        else { 0.7 }
    };

    QualityMetadata {
        duration_ms,
        peak,
        rms,
        clipping_detected,
        clipping_percent,
        silence_trimmed: was_trimmed,
        transform_type: transform.to_string(),
        is_silent,
        is_too_quiet,
        duration_appropriate,
        spectral_quality,
        transient_quality,
        tonal_clarity,
        noise_floor_quality,
        dynamic_range,
        spectral_balance,
        punch_quality,
    }
}

#[allow(dead_code)]
pub fn compute_failure_labels(quality: &QualityMetadata) -> Vec<String> {
    let mut labels = Vec::new();
    if quality.clipping_detected {
        labels.push("clipped".to_string());
    }
    if !quality.duration_appropriate && quality.duration_ms > 0.0 {
        labels.push("too long".to_string());
    }
    if quality.is_too_quiet {
        labels.push("too quiet".to_string());
    }
    if quality.is_silent {
        labels.push("silent".to_string());
    }
    labels
}

#[allow(dead_code)]
pub fn missing_file_label() -> Vec<String> {
    vec!["missing file".to_string()]
}

#[allow(dead_code)]
pub fn processing_failed_label(error: &str) -> Vec<String> {
    vec![format!("processing failed: {}", error)]
}
