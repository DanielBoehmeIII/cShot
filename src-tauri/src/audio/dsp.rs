use std::f32::consts::PI;
use super::{DspParams, SAMPLE_RATE};

pub fn high_pass(samples: &mut [f32], cutoff_hz: f32) {
    if samples.len() < 4 { return; }
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    let dt = 1.0 / SAMPLE_RATE as f32;
    let alpha = rc / (rc + dt);
    let mut prev = samples[0];
    for sample in samples.iter_mut() {
        let input = *sample;
        *sample = alpha * (prev + *sample - prev);
        prev = input;
    }
}

pub fn low_pass(samples: &mut [f32], cutoff_hz: f32) {
    if samples.len() < 4 { return; }
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    let dt = 1.0 / SAMPLE_RATE as f32;
    let alpha = dt / (rc + dt);
    let mut prev = 0.0;
    for sample in samples.iter_mut() {
        prev += alpha * (*sample - prev);
        *sample = prev;
    }
}

pub fn biquad_low_shelf(samples: &mut [f32], freq: f32, gain_db: f32, q: f32) {
    if gain_db.abs() < 0.5 || samples.len() < 4 { return; }
    let a = 10.0_f32.powf(gain_db / 40.0);
    let w0 = 2.0 * PI * freq / SAMPLE_RATE as f32;
    let sin_w0 = w0.sin();
    let cos_w0 = w0.cos();
    let alpha = sin_w0 / (2.0 * q);
    let beta = a.sqrt() * alpha;
    let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + beta);
    let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
    let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - beta);
    let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + beta;
    let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
    let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - beta;
    apply_biquad(samples, b0, b1, b2, a0, a1, a2);
}

pub fn biquad_high_shelf(samples: &mut [f32], freq: f32, gain_db: f32, q: f32) {
    if gain_db.abs() < 0.5 || samples.len() < 4 { return; }
    let a = 10.0_f32.powf(gain_db / 40.0);
    let w0 = 2.0 * PI * freq / SAMPLE_RATE as f32;
    let sin_w0 = w0.sin();
    let cos_w0 = w0.cos();
    let alpha = sin_w0 / (2.0 * q);
    let beta = a.sqrt() * alpha;
    let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + beta);
    let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
    let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - beta);
    let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + beta;
    let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
    let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - beta;
    apply_biquad(samples, b0, b1, b2, a0, a1, a2);
}

pub fn biquad_peaking(samples: &mut [f32], freq: f32, gain_db: f32, q: f32) {
    if gain_db.abs() < 0.3 || samples.len() < 4 { return; }
    let a = 10.0_f32.powf(gain_db / 40.0);
    let w0 = 2.0 * PI * freq / SAMPLE_RATE as f32;
    let alpha = w0.sin() / (2.0 * q);
    let b0 = 1.0 + alpha * a;
    let b1 = -2.0 * w0.cos();
    let b2 = 1.0 - alpha * a;
    let a0 = 1.0 + alpha / a;
    let a1 = -2.0 * w0.cos();
    let a2 = 1.0 - alpha / a;
    apply_biquad(samples, b0, b1, b2, a0, a1, a2);
}

fn apply_biquad(samples: &mut [f32], b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
    let mut x1 = 0.0;
    let mut x2 = 0.0;
    let mut y1 = 0.0;
    let mut y2 = 0.0;
    for sample in samples.iter_mut() {
        let x = *sample;
        let y = (b0 / a0) * x + (b1 / a0) * x1 + (b2 / a0) * x2 - (a1 / a0) * y1 - (a2 / a0) * y2;
        x2 = x1;
        x1 = x;
        y2 = y1;
        y1 = y;
        *sample = y;
    }
}

pub fn apply_eq(samples: &mut [f32], params: &DspParams) {
    if params.low_pass || params.dark {
        low_pass(samples, 800.0);
    }
    if params.high_pass || params.bright {
        if !params.low_pass && !params.dark {
            high_pass(samples, 2000.0);
        }
    }
}

pub fn apply_envelope(samples: &mut [f32], attack_s: f32, decay_s: f32) {
    let attack_samples = (attack_s * SAMPLE_RATE as f32) as usize;
    let decay_samples = (decay_s * SAMPLE_RATE as f32) as usize;
    for i in 0..attack_samples.min(samples.len()) {
        samples[i] *= i as f32 / attack_samples as f32;
    }
    for i in 0..decay_samples.min(samples.len()) {
        let idx = samples.len() - 1 - i;
        samples[idx] *= i as f32 / decay_samples as f32;
    }
}

pub fn pitch_shift(samples: &[f32], ratio: f32) -> Vec<f32> {
    let new_len = (samples.len() as f32 / ratio) as usize;
    let mut out = vec![0.0f32; new_len];
    for i in 0..new_len {
        let src = i as f32 * ratio;
        let src_i = src as usize;
        let frac = src - src_i as f32;
        if src_i + 1 < samples.len() {
            out[i] = samples[src_i] * (1.0 - frac) + samples[src_i + 1] * frac;
        } else if src_i < samples.len() {
            out[i] = samples[src_i];
        }
    }
    out
}

pub fn apply_punch(samples: &mut [f32]) {
    let attack_ms = 3;
    let attack_samples = (attack_ms as f32 * SAMPLE_RATE as f32 / 1000.0) as usize;
    for i in 0..attack_samples.min(samples.len()) {
        let boost = 1.0 + 0.5 * (1.0 - i as f32 / attack_samples as f32);
        samples[i] *= boost;
    }
}

pub fn transient_enhance(samples: &mut [f32], boost_db: f32) {
    if samples.len() < 64 || boost_db <= 0.0 { return; }
    let onset_len = (SAMPLE_RATE as f32 * 0.008) as usize;
    let gain = 10.0_f32.powf(boost_db / 20.0);
    let threshold = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.25;
    let mut onset_pos = 0;
    for i in 10..samples.len().min(SAMPLE_RATE as usize / 2) {
        if samples[i].abs() > threshold {
            onset_pos = i;
            break;
        }
    }
    if onset_pos < 10 || onset_pos > samples.len() / 2 { return; }
    let end = std::cmp::min(onset_pos + onset_len, samples.len());
    for i in onset_pos..end {
        let t = (i - onset_pos) as f32 / onset_len as f32;
        let envelope = 1.0 - t;
        samples[i] *= 1.0 + (gain - 1.0) * envelope;
    }
}

pub fn noise_gate_tail(samples: &mut [f32], threshold_db: f32) {
    if samples.len() < 256 { return; }
    let threshold = 10.0_f32.powf(threshold_db / 20.0);
    let hold_samples = (SAMPLE_RATE as f32 * 0.01) as usize;
    let mut last_above = 0;
    for i in 0..samples.len() {
        if samples[i].abs() > threshold {
            last_above = i;
        }
    }
    let fade_start = std::cmp::min(last_above + hold_samples, samples.len());
    if fade_start >= samples.len() - 4 { return; }
    let fade_len = (SAMPLE_RATE as f32 * 0.005) as usize;
    let fade_end = std::cmp::min(fade_start + fade_len, samples.len());
    for i in fade_start..fade_end {
        let t = (i - fade_start) as f32 / fade_len as f32;
        samples[i] *= 1.0 - t;
    }
    for sample in samples.iter_mut().skip(fade_end) {
        *sample = 0.0;
    }
}

pub fn true_peak_limiter(samples: &mut [f32], ceiling_db: f32) {
    let ceiling = 10.0_f32.powf(ceiling_db / 20.0);
    let threshold = ceiling * 0.92;
    let mut gain: f32 = 1.0;
    let release_coeff = (-1.0 / (SAMPLE_RATE as f32 * 0.01)).exp();
    for sample in samples.iter_mut() {
        let abs = sample.abs();
        if abs > threshold {
            let target_gain = ceiling / (abs + 1e-10);
            gain = gain.min(target_gain);
        } else {
            gain = 1.0 - (1.0 - gain) * (1.0 - release_coeff);
        }
        *sample *= gain;
    }
}

pub fn spectral_balance(samples: &mut [f32], sound_type_str: &str) {
    let (mud_freq, mud_gain, presence_freq, presence_gain, air_freq, air_gain): (f32, f32, f32, f32, f32, f32) = match sound_type_str {
        "kick" => (200.0, -1.5, 3000.0, 1.5, 0.0, 0.0),
        "snare" => (250.0, -1.0, 3500.0, 2.0, 10000.0, 0.5),
        "closed_hat" => (0.0, 0.0, 3000.0, 1.0, 10000.0, 1.0),
        "open_hat" => (0.0, 0.0, 3000.0, 1.0, 10000.0, 0.5),
        "clap" => (300.0, -1.0, 3000.0, 1.5, 10000.0, 0.5),
        "tom" => (200.0, -1.0, 2500.0, 1.0, 0.0, 0.0),
        "perc" => (200.0, -0.5, 3000.0, 1.0, 10000.0, 0.5),
        "bass" => (150.0, -1.5, 2000.0, 0.5, 0.0, 0.0),
        "fx" => (0.0, 0.0, 2000.0, 0.5, 10000.0, 0.5),
        _ => (200.0, -0.5, 3000.0, 0.5, 10000.0, 0.3),
    };
    if mud_gain.abs() > 0.3 {
        biquad_low_shelf(samples, mud_freq, mud_gain, 0.7);
    }
    if presence_gain.abs() > 0.3 {
        biquad_peaking(samples, presence_freq, presence_gain, 1.0);
    }
    if air_gain.abs() > 0.3 {
        biquad_high_shelf(samples, air_freq, air_gain, 0.7);
    }
}
