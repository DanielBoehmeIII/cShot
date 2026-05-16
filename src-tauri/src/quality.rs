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
