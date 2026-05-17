use super::resynthesize::{self, ResynthesisParams};
use super::SAMPLE_RATE;
use super::dsp;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TransformParams {
    pub duration_scale: Option<f32>,
    pub pitch_shift_semitones: Option<f32>,
    pub filter_low_hz: Option<f32>,
    pub filter_high_hz: Option<f32>,
    pub saturation_drive: Option<f32>,
    pub transient_boost_db: Option<f32>,
    pub noise_add: Option<f32>,
    pub sub_add: Option<f32>,
    pub click_add: Option<f32>,
    pub brightness_tilt: Option<f32>,
    pub reverse: bool,
}

impl Default for TransformParams {
    fn default() -> Self {
        Self {
            duration_scale: None,
            pitch_shift_semitones: None,
            filter_low_hz: None,
            filter_high_hz: None,
            saturation_drive: None,
            transient_boost_db: None,
            noise_add: None,
            sub_add: None,
            click_add: None,
            brightness_tilt: None,
            reverse: false,
        }
    }
}

pub fn transform_with_params(
    source_samples: &[f32],
    params: &ResynthesisParams,
) -> Vec<f32> {
    let synthesized = resynthesize::resynthesize(params);
    if synthesized.is_empty() {
        return source_samples.to_vec();
    }

    let mut result = Vec::with_capacity(source_samples.len().max(synthesized.len()));
    let max_len = source_samples.len().max(synthesized.len());
    for i in 0..max_len {
        let src = source_samples.get(i).copied().unwrap_or(0.0);
        let synth = synthesized.get(i).copied().unwrap_or(0.0);
        result.push((src * 0.4 + synth * 0.6).clamp(-1.0, 1.0));
    }
    super::process::normalize_peak(&mut result, -1.0);
    result
}

pub fn apply_dsp_transforms(
    samples: &mut Vec<f32>,
    params: &TransformParams,
) {
    if params.reverse {
        samples.reverse();
    }
    if let Some(drive) = params.saturation_drive {
        if drive > 1.01 {
            for s in samples.iter_mut() {
                *s = dsp::tape_saturation(*s, drive);
            }
        }
    }
    if let Some(hz) = params.filter_low_hz {
        if hz > 20.0 {
            dsp::low_pass(samples, hz);
        }
    }
    if let Some(hz) = params.filter_high_hz {
        if hz > 20.0 {
            dsp::high_pass(samples, hz);
        }
    }
    if let Some(db) = params.transient_boost_db {
        if db > 0.0 {
            dsp::transient_enhance(samples, db);
        }
    }
    if let Some(amt) = params.noise_add {
        if amt > 0.0 {
            let len = samples.len();
            for i in 0..len {
                let t = i as f32 / SAMPLE_RATE as f32;
                let n = super::noise((i as f32 * 0.3).fract());
                let env = (-20.0 * t).exp();
                samples[i] += n * amt * env * 0.3;
            }
        }
    }
    if let Some(amt) = params.sub_add {
        if amt > 0.0 {
            let len = samples.len();
            for i in 0..len {
                let t = i as f32 / SAMPLE_RATE as f32;
                let env = (-3.0 * t).exp();
                let sub = (2.0 * std::f32::consts::PI * 55.0 * t).sin() * env * amt;
                samples[i] = (samples[i] + sub).clamp(-1.0, 1.0);
            }
        }
    }
    if let Some(amt) = params.click_add {
        if amt > 0.0 {
            let click_len = (SAMPLE_RATE as f32 * 0.003) as usize;
            let click_end = click_len.min(samples.len());
            for i in 0..click_end {
                let t = i as f32 / SAMPLE_RATE as f32;
                let click_env = (-80.0 * t).exp();
                let click = (2.0 * std::f32::consts::PI * 3000.0 * t).sin() * click_env * amt * 0.3;
                samples[i] = (samples[i] + click).clamp(-1.0, 1.0);
            }
        }
    }
    if let Some(tilt) = params.brightness_tilt {
        if tilt > 0.0 {
            dsp::biquad_high_shelf(samples, 3000.0, tilt * 6.0, 0.7);
        } else if tilt < 0.0 {
            dsp::biquad_low_shelf(samples, 300.0, tilt.abs() * 4.0, 0.7);
        }
    }
    if let Some(scale) = params.duration_scale {
        if scale > 0.0 && (scale - 1.0).abs() > 0.01 {
            let new_len = (samples.len() as f32 * scale) as usize;
            if new_len > 0 && new_len != samples.len() {
                let mut resampled = Vec::with_capacity(new_len);
                let step = samples.len() as f32 / new_len as f32;
                let mut pos = 0.0;
                for _ in 0..new_len {
                    let idx = pos as usize;
                    if idx < samples.len() {
                        resampled.push(samples[idx]);
                    }
                    pos += step;
                }
                *samples = resampled;
            }
        }
    }
    if let Some(semitones) = params.pitch_shift_semitones {
        if semitones.abs() > 0.5 {
            let ratio = 2.0_f32.powf(semitones / 12.0);
            let shifted = dsp::pitch_shift(samples, ratio);
            *samples = shifted;
        }
    }
    super::process::normalize_peak(samples, -1.0);
}
