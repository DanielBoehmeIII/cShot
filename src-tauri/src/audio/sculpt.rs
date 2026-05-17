use super::{SAMPLE_RATE, dsp, process};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SculptControls {
    pub transient_intensity: f32,
    pub tail_length: f32,
    pub brightness: f32,
    pub distortion: f32,
    pub density: f32,
    pub tonal_noise_balance: f32,
    pub sub_amount: f32,
    pub body_thickness: f32,
    pub attack_sharpness: f32,
    pub stereo_width: f32,
}

impl Default for SculptControls {
    fn default() -> Self {
        Self {
            transient_intensity: 0.5,
            tail_length: 0.5,
            brightness: 0.5,
            distortion: 0.0,
            density: 0.5,
            tonal_noise_balance: 0.5,
            sub_amount: 0.3,
            body_thickness: 0.5,
            attack_sharpness: 0.5,
            stereo_width: 0.0,
        }
    }
}

pub fn apply_sculpt(samples: &mut Vec<f32>, controls: &SculptControls) {
    if samples.is_empty() { return; }

    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return; }

    let ti = controls.transient_intensity;
    if (ti - 0.5).abs() > 0.05 {
        let boost_db = (ti - 0.5) * 12.0;
        dsp::transient_enhance(samples, boost_db);
    }

    let tl = controls.tail_length;
    if (tl - 0.5).abs() > 0.05 {
        let target_len = if tl < 0.5 {
            (samples.len() as f32 * (0.3 + tl * 1.4)).min(samples.len() as f32) as usize
        } else {
            let extra = (tl - 0.5) * 2.0 * SAMPLE_RATE as f32 * 0.3;
            (samples.len() as f32 + extra) as usize
        };

        if target_len < samples.len() {
            let fade_start = target_len;
            for i in fade_start..samples.len() {
                let t = (i - fade_start) as f32 / (samples.len() - fade_start).max(1) as f32;
                samples[i] *= (1.0 - t * t).max(0.0);
            }
            samples.truncate(target_len);
        } else if target_len > samples.len() {
            let extra = target_len - samples.len();
            samples.resize(target_len, 0.0);
            let fade_len = extra.min((SAMPLE_RATE as f32 * 0.05) as usize);
            for i in samples.len().saturating_sub(extra)..samples.len() {
                let t = (i - (samples.len() - extra)) as f32 / extra as f32;
                samples[i] = samples[samples.len() - extra - 1].abs() * 0.01 * t * (1.0 - t);
            }
        }
    }

    let br = controls.brightness;
    if (br - 0.5).abs() > 0.05 {
        if br > 0.5 {
            let gain_db = (br - 0.5) * 8.0;
            dsp::biquad_high_shelf(samples, 3000.0, gain_db, 0.7);
        } else {
            let gain_db = (0.5 - br) * 6.0;
            dsp::biquad_low_shelf(samples, 300.0, gain_db, 0.7);
        }
    }

    let dist = controls.distortion;
    if dist > 0.05 {
        let drive = 1.0 + dist * 4.0;
        dsp::apply_saturation(samples, drive);
    }

    let sub = controls.sub_amount;
    if sub > 0.05 {
        let len = samples.len();
        for i in 0..len {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = (-3.0 * t).exp();
            let sine = (2.0 * std::f32::consts::PI * 55.0 * t).sin();
            samples[i] = (samples[i] + sine * env * sub * 0.3).clamp(-1.0, 1.0);
        }
    }

    let body = controls.body_thickness;
    if (body - 0.5).abs() > 0.05 {
        let gain = if body > 0.5 {
            1.0 + (body - 0.5) * 0.6
        } else {
            1.0 - (0.5 - body) * 0.4
        };
        for s in samples.iter_mut() {
            *s = (*s * gain).clamp(-1.0, 1.0);
        }
    }

    let tnb = controls.tonal_noise_balance;
    if (tnb - 0.5).abs() > 0.05 {
        if tnb > 0.5 {
            let noise_amt = (tnb - 0.5) * 0.3;
            for i in 0..samples.len() {
                let t = i as f32 / SAMPLE_RATE as f32;
                let n = (i as f32 * 127.1).sin().fract() * 2.0 - 1.0;
                samples[i] = (samples[i] * (1.0 - noise_amt * 0.3) + n * noise_amt * 0.2).clamp(-1.0, 1.0);
            }
        }
    }

    let sw = controls.stereo_width;
    if sw > 0.05 && samples.len() > 1 {
        dsp::stereo_widen(samples, sw);
    }

    process::normalize_peak(samples, -1.0);
}

pub fn generate_sculpt_preview(samples: &[f32], controls: &SculptControls) -> Vec<f32> {
    let step = (samples.len() / (SAMPLE_RATE as usize * 2).min(samples.len())).max(1);
    let preview: Vec<f32> = samples.iter().step_by(step).copied().collect();
    let mut result = preview;
    apply_sculpt(&mut result, controls);
    result
}
