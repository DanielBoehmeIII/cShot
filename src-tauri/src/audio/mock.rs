use super::process;
use super::SAMPLE_RATE;

pub fn reverse(samples: &[f32]) -> Vec<f32> {
    let mut out = samples.to_vec();
    out.reverse();
    out
}

pub fn saturate(samples: &[f32], drive: f32) -> Vec<f32> {
    let mut out = Vec::with_capacity(samples.len());
    for &s in samples {
        let s = s * drive;
        let s = s.tanh();
        out.push(s);
    }
    out
}

pub fn shorten(samples: &[f32], factor: f32) -> Vec<f32> {
    let new_len = (samples.len() as f32 * factor) as usize;
    let new_len = new_len.max(64);
    let mut out = Vec::with_capacity(new_len);
    let step = samples.len() as f32 / new_len as f32;
    let mut pos = 0.0;
    for _ in 0..new_len {
        let idx = pos as usize;
        if idx < samples.len() {
            out.push(samples[idx]);
        }
        pos += step;
    }
    out
}

pub fn layer(samples_a: &[f32], samples_b: &[f32]) -> Vec<f32> {
    let max_len = samples_a.len().max(samples_b.len());
    let mut out = Vec::with_capacity(max_len);
    for i in 0..max_len {
        let a = samples_a.get(i).copied().unwrap_or(0.0) * 0.7;
        let b = samples_b.get(i).copied().unwrap_or(0.0) * 0.7;
        out.push((a + b).clamp(-1.0, 1.0));
    }
    out
}

pub fn transient_shape(samples: &mut [f32], amount: f32) {
    if amount <= 0.0 || samples.len() < 64 {
        return;
    }
    let onset_len = (SAMPLE_RATE as f32 * 0.005) as usize;
    let envelope: Vec<f32> = (0..onset_len)
        .map(|i| 1.0 + (amount - 1.0) * (1.0 - i as f32 / onset_len as f32))
        .collect();

    let threshold = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.3;
    let mut in_onset = false;
    let mut envelope_idx = 0;

    for i in 0..samples.len() {
        if !in_onset && samples[i].abs() > threshold && i > 10 {
            in_onset = true;
            envelope_idx = 0;
        }
        if in_onset {
            if envelope_idx < envelope.len() {
                samples[i] *= envelope[envelope_idx];
                envelope_idx += 1;
            } else {
                in_onset = false;
            }
        }
    }
}

pub enum MockVariant {
    Trimmed,
    Repitched,
    Reversed,
    Saturated,
    Shortened,
    Layered,
    TransientShaped,
    Randomized,
    BrightVariant,
    DarkVariant,
    PunchyVariant,
    AiryVariant,
    GrittyVariant,
    SubbyVariant,
    TightVariant,
}

pub fn apply_variant(samples: &[f32], variant: &MockVariant, seed: u64) -> Vec<f32> {
    match variant {
        MockVariant::Trimmed => {
            let mut s = samples.to_vec();
            process::trim_silence(&mut s, 0.001);
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::Repitched => {
            let ratios = [0.7, 0.85, 1.0, 1.15, 1.3, 1.5];
            let idx = (seed as usize) % ratios.len();
            let ratio = ratios[idx];
            let s = super::dsp::pitch_shift(samples, ratio);
            let mut s = s;
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::Reversed => {
            reverse(samples)
        }
        MockVariant::Saturated => {
            let drives = [1.5, 2.0, 3.0, 5.0, 8.0];
            let idx = (seed as usize) % drives.len();
            let s = saturate(samples, drives[idx]);
            let mut s = s;
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::Shortened => {
            let factors = [0.3, 0.5, 0.65, 0.8];
            let idx = (seed as usize) % factors.len();
            shorten(samples, factors[idx])
        }
        MockVariant::Layered => {
            let shifted = super::dsp::pitch_shift(samples, 1.2);
            layer(samples, &shifted)
        }
        MockVariant::TransientShaped => {
            let amounts = [1.5, 2.0, 3.0, 4.0];
            let idx = (seed as usize) % amounts.len();
            let mut s = samples.to_vec();
            transient_shape(&mut s, amounts[idx]);
            let mut s = s;
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::Randomized => {
            let rng_seed = seed as f32 * 0.618;
            let drive = 1.0 + (rng_seed * 3.0).fract() * 4.0;
            let pitch = 0.7 + (rng_seed * 7.0).fract() * 0.8;
            let shorten_factor = 0.4 + (rng_seed * 13.0).fract() * 0.5;

            let mut s = saturate(samples, drive);
            s = super::dsp::pitch_shift(&s, pitch);
            if (rng_seed * 11.0).fract() > 0.5 {
                s = shorten(&s, shorten_factor);
            }
            let mut s = s;
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::BrightVariant => {
            let mut s = samples.to_vec();
            super::dsp::biquad_high_shelf(&mut s, 4000.0, 3.0, 0.7);
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::DarkVariant => {
            let mut s = samples.to_vec();
            super::dsp::low_pass(&mut s, 3000.0);
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::PunchyVariant => {
            let mut s = samples.to_vec();
            super::dsp::transient_enhance(&mut s, 5.0);
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::AiryVariant => {
            let mut s = samples.to_vec();
            super::dsp::biquad_high_shelf(&mut s, 8000.0, 4.0, 0.7);
            super::dsp::noise_gate_tail(&mut s, -55.0);
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::GrittyVariant => {
            let s = saturate(samples, 2.5);
            let mut s = s;
            super::dsp::biquad_low_shelf(&mut s, 200.0, -1.0, 0.7);
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::SubbyVariant => {
            let mut s = samples.to_vec();
            super::dsp::biquad_low_shelf(&mut s, 100.0, 4.0, 0.7);
            super::dsp::high_pass(&mut s, 30.0);
            process::normalize_peak(&mut s, -1.0);
            s
        }
        MockVariant::TightVariant => {
            let mut s = shorten(samples, 0.6);
            super::dsp::transient_enhance(&mut s, 3.0);
            process::normalize_peak(&mut s, -1.0);
            s
        }
    }
}

pub fn generate_variant_name(variant: &MockVariant) -> &'static str {
    match variant {
        MockVariant::Trimmed => "trimmed",
        MockVariant::Repitched => "repitched",
        MockVariant::Reversed => "reversed",
        MockVariant::Saturated => "saturated",
        MockVariant::Shortened => "shortened",
        MockVariant::Layered => "layered",
        MockVariant::TransientShaped => "shaped",
        MockVariant::Randomized => "randomized",
        MockVariant::BrightVariant => "bright",
        MockVariant::DarkVariant => "dark",
        MockVariant::PunchyVariant => "punchy",
        MockVariant::AiryVariant => "airy",
        MockVariant::GrittyVariant => "gritty",
        MockVariant::SubbyVariant => "subby",
        MockVariant::TightVariant => "tight",
    }
}
