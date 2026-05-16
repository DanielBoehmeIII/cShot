use super::{DspParams, SoundType, SAMPLE_RATE};
use super::dsp;

pub fn trim_silence(samples: &mut Vec<f32>, threshold: f32) {
    if samples.is_empty() {
        return;
    }
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

pub fn remove_dc_offset(samples: &mut [f32]) {
    if samples.is_empty() {
        return;
    }
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    if mean.abs() > 0.001 {
        for sample in samples.iter_mut() {
            *sample -= mean;
        }
    }
}

pub fn normalize_peak(samples: &mut [f32], target_db: f32) {
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak > 0.0 {
        let target_peak = 10.0f32.powf(target_db / 20.0);
        let gain = target_peak / peak;
        if gain > 0.0 {
            for sample in samples.iter_mut() {
                *sample *= gain;
            }
        }
    }
}

pub fn apply_fade(samples: &mut [f32], fade_in_s: f32, fade_out_s: f32) {
    if samples.is_empty() {
        return;
    }
    let fade_in_len = (fade_in_s * SAMPLE_RATE as f32) as usize;
    let fade_out_len = (fade_out_s * SAMPLE_RATE as f32) as usize;

    let flen = fade_in_len.min(samples.len());
    for i in 0..flen {
        samples[i] *= i as f32 / flen.max(1) as f32;
    }
    let flen = fade_out_len.min(samples.len());
    for i in 0..flen {
        let idx = samples.len() - 1 - i;
        samples[idx] *= i as f32 / flen.max(1) as f32;
    }
}

pub fn shorten_duration(samples: &mut Vec<f32>, _sample_rate: u32, fraction: f32) {
    if samples.is_empty() || fraction <= 0.0 {
        return;
    }
    let new_len = (samples.len() as f32 * fraction.clamp(0.1, 0.95)) as usize;
    if new_len < samples.len() && new_len > 0 {
        samples.truncate(new_len);
    }
}

pub fn brighten(samples: &mut [f32]) {
    let gain = 1.4;
    for sample in samples.iter_mut() {
        let high = *sample * gain;
        *sample = (*sample + high.clamp(-1.0, 1.0)) * 0.5;
    }
}

pub fn darken(samples: &mut [f32], sample_rate: u32) {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * 800.0);
    let dt = 1.0 / sample_rate as f32;
    let alpha = dt / (rc + dt);
    let mut prev = 0.0;
    for sample in samples.iter_mut() {
        prev += alpha * (*sample - prev);
        *sample = prev;
    }
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

    // 1. Remove DC offset
    remove_dc_offset(samples);
    dsp::high_pass(samples, 20.0);

    // 2. Trim silence
    trim_silence(samples, 0.001);

    // 3. Apply fades
    apply_fade(samples, 0.003, 0.008);

    // 4. Spectral balance EQ (mud cut, presence boost, air)
    if samples.len() > 64 {
        dsp::spectral_balance(samples, sound_type.as_str());
    }

    // 5. Apply prompt-based EQ
    dsp::apply_eq(samples, dsp_params);

    // 6. Transient enhancement for percussive sounds
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
        dsp::transient_enhance(samples, transient_boost);
    }
    if dsp_params.punch {
        dsp::apply_punch(samples);
    }

    // 7. Apply gain
    let safe_gain = dsp_params.gain.min(3.0);
    if (safe_gain - 1.0).abs() > 0.01 {
        for sample in samples.iter_mut() {
            *sample *= safe_gain;
        }
    }

    // 8. Noise gate tail
    dsp::noise_gate_tail(samples, -50.0);

    // 9. Normalize peak with headroom
    normalize_peak(samples, -1.0);

    // 10. True-peak limiter
    dsp::true_peak_limiter(samples, -0.5);

    // Fade again after any potential clicks from limiting
    apply_fade(samples, 0.001, 0.003);

    // Final cleanup
    trim_silence(samples, 0.0001);

    if has_nan_or_inf(samples) {
        samples.fill(0.0);
    }

    let tags = super::analyze::apply_autotags(samples, &sound_type, None, None);
    tags
}
