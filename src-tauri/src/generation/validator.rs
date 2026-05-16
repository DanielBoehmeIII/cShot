use crate::audio::SAMPLE_RATE;
use super::provider::{ValidationResult, GenerationRequest};

pub fn validate_pre_generation(request: &GenerationRequest) -> Result<(), String> {
    let trimmed = request.prompt.trim();
    if trimmed.is_empty() {
        return Err("Prompt cannot be empty. Describe the sound you want.".to_string());
    }
    if trimmed.len() > 500 {
        return Err(format!(
            "Prompt is too long ({} chars). Maximum is 500 characters.",
            trimmed.len()
        ));
    }
    if let Some(ref ref_audio) = request.reference_audio {
        if ref_audio.is_empty() {
            return Err("Reference audio is empty.".to_string());
        }
        if ref_audio.len() as f32 / SAMPLE_RATE as f32 > 60.0 {
            return Err("Reference audio is too long (max 60 seconds).".to_string());
        }
    }
    Ok(())
}

pub fn validate_generated_sound(samples: &[f32]) -> ValidationResult {
    let mut issues = Vec::new();

    if samples.is_empty() {
        return ValidationResult {
            passed: false,
            issues: vec!["Audio buffer is empty".to_string()],
            rms: 0.0,
            peak: 0.0,
            duration_ms: 0.0,
            has_silence: true,
            has_clipping: false,
            has_nan: false,
        };
    }

    let has_nan = samples.iter().any(|s| s.is_nan() || s.is_infinite());
    if has_nan {
        issues.push("Audio contains NaN or Inf values".to_string());
    }

    let rms = crate::audio::compute_rms(samples);
    let peak = crate::audio::compute_peak(samples);
    let duration_ms = samples.len() as f32 / SAMPLE_RATE as f32 * 1000.0;

    let has_silence = rms < 0.001;
    if has_silence {
        issues.push("Generated sound is silent (RMS below threshold)".to_string());
    }

    let clipped_count = samples.iter().filter(|&&s| s.abs() >= 1.0).count();
    let has_clipping = peak >= 0.99 || (!samples.is_empty() && clipped_count as f32 / samples.len() as f32 > 0.001);
    if has_clipping {
        issues.push(format!("Clipping detected ({} samples at max)", clipped_count));
    }

    let passed = !has_nan && !has_silence;

    ValidationResult {
        passed,
        issues,
        rms,
        peak,
        duration_ms,
        has_silence,
        has_clipping,
        has_nan,
    }
}
