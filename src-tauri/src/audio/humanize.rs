use std::f32::consts::PI;
use super::SAMPLE_RATE;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HumanizeParams {
    pub analog_drift: f32,
    pub instability: f32,
    pub transient_randomness: f32,
    pub envelope_variation: f32,
    pub saturation_randomness: f32,
    pub non_static_layers: f32,
    pub phase_variation: f32,
    pub humanize_transients: f32,
}

impl Default for HumanizeParams {
    fn default() -> Self {
        Self {
            analog_drift: 0.0,
            instability: 0.0,
            transient_randomness: 0.0,
            envelope_variation: 0.0,
            saturation_randomness: 0.0,
            non_static_layers: 0.0,
            phase_variation: 0.0,
            humanize_transients: 0.0,
        }
    }
}

impl HumanizeParams {
    pub fn warm() -> Self {
        Self {
            analog_drift: 0.15,
            instability: 0.05,
            transient_randomness: 0.1,
            envelope_variation: 0.08,
            saturation_randomness: 0.1,
            non_static_layers: 0.08,
            phase_variation: 0.1,
            humanize_transients: 0.1,
        }
    }

    pub fn vintage() -> Self {
        Self {
            analog_drift: 0.25,
            instability: 0.1,
            transient_randomness: 0.15,
            envelope_variation: 0.12,
            saturation_randomness: 0.2,
            non_static_layers: 0.12,
            phase_variation: 0.15,
            humanize_transients: 0.15,
        }
    }

    pub fn lo_fi() -> Self {
        Self {
            analog_drift: 0.4,
            instability: 0.15,
            transient_randomness: 0.2,
            envelope_variation: 0.15,
            saturation_randomness: 0.25,
            non_static_layers: 0.15,
            phase_variation: 0.2,
            humanize_transients: 0.2,
        }
    }

    pub fn scaled(&self, amount: f32) -> Self {
        Self {
            analog_drift: self.analog_drift * amount,
            instability: self.instability * amount,
            transient_randomness: self.transient_randomness * amount,
            envelope_variation: self.envelope_variation * amount,
            saturation_randomness: self.saturation_randomness * amount,
            non_static_layers: self.non_static_layers * amount,
            phase_variation: self.phase_variation * amount,
            humanize_transients: self.humanize_transients * amount,
        }
    }
}

fn noise(phase: f32) -> f32 {
    ((phase * 127.1).sin() * 43758.5453).fract() * 2.0 - 1.0
}

fn frac(x: f32) -> f32 {
    x - x.floor()
}

pub fn apply_analog_drift(samples: &mut [f32], amount: f32, sample_rate: u32, seed: u64) {
    if amount <= 0.001 || samples.is_empty() { return; }
    let s = seed as f32;
    let lfo_rate_1 = 0.3 + frac(s * 1.7) * 2.0;
    let lfo_rate_2 = 1.0 + frac(s * 3.1) * 4.0;
    let lfo_rate_3 = 0.05 + frac(s * 5.3) * 0.3;
    let drift_depth = amount * 0.15 * 0.002;
    let mut random_walk = 0.0f32;
    let walk_decay = 0.995;
    let walk_noise_floor = 0.00005;

    let tmp = samples.to_vec();
    for i in 0..samples.len() {
        let t = i as f32 / sample_rate as f32;
        let lfo = (2.0 * PI * lfo_rate_1 * t).sin()
            + 0.5 * (2.0 * PI * lfo_rate_2 * t).sin()
            + 0.25 * (2.0 * PI * lfo_rate_3 * t).sin();
        let wn = noise((i as f32 * s * 0.01).fract()) * 0.0001;
        random_walk = random_walk * walk_decay + wn;
        random_walk = random_walk.clamp(-walk_noise_floor, walk_noise_floor);
        let drift = (lfo / 1.75 + random_walk * 100.0) * drift_depth;
        let src_idx = (i as f32 * (1.0 + drift)).clamp(0.0, (samples.len() - 1) as f32) as usize;
        samples[i] = tmp.get(src_idx).copied().unwrap_or(tmp.last().copied().unwrap_or(0.0));
    }
}

pub fn apply_instability(samples: &mut [f32], amount: f32, seed: u64) {
    if amount <= 0.001 || samples.is_empty() { return; }
    let s = seed as f32;
    for i in 0..samples.len() {
        let jitter = noise((i as f32 * 0.07 + s).fract()) * amount * 0.002;
        let phase_jitter = noise((i as f32 * 0.13 + s * 0.7).fract()) * amount * 0.0003;
        let smoothed = samples[i] * (1.0 + jitter);
        samples[i] = smoothed + phase_jitter;
    }
}

pub fn apply_transient_randomness(samples: &mut [f32], amount: f32, seed: u64) {
    if amount <= 0.001 || samples.len() < 64 { return; }
    let s = seed as f32;
    let onset_len = (SAMPLE_RATE as f32 * 0.008) as usize;
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return; }
    let threshold = peak * 0.2;
    let onset_pos = samples.iter().position(|s| s.abs() > threshold).unwrap_or(0);
    if onset_pos < 2 || onset_pos > samples.len() / 2 { return; }
    let onset_end = (onset_pos + onset_len).min(samples.len());
    let pre_onset = onset_pos.saturating_sub(4);
    let timing_jitter = ((frac(s * 7.1) - 0.5) * 2.0 * amount * 4.0) as isize;
    let samples_copy = samples.to_vec();
    for i in pre_onset..onset_end.min(samples.len()) {
        let jittered_idx = (i as isize + timing_jitter).clamp(0, samples.len() as isize - 1) as usize;
        if jittered_idx < samples.len() {
            let var = 1.0 + (frac(s * 11.3 + i as f32 * 0.03) - 0.5) * amount * 0.3;
            samples[i] = samples_copy[jittered_idx] * var;
        }
    }
}

pub fn apply_envelope_variation(samples: &mut [f32], amount: f32, seed: u64) {
    if amount <= 0.001 || samples.len() < 128 { return; }
    let s = seed as f32;
    let window = (SAMPLE_RATE as f32 * 0.003) as usize;
    let mut rng_state = s;
    for chunk_start in (0..samples.len()).step_by(window) {
        let chunk_end = (chunk_start + window).min(samples.len());
        let var = 1.0 + (frac(rng_state * 13.7) - 0.5) * 2.0 * amount * 0.015;
        rng_state = frac(rng_state * 17.3 + 0.1);
        for i in chunk_start..chunk_end {
            samples[i] *= var;
        }
    }
}

pub fn apply_saturation_randomness(samples: &mut [f32], amount: f32, seed: u64) {
    if amount <= 0.001 || samples.is_empty() { return; }
    let s = seed as f32;
    use super::dsp::{SaturationType, soft_clip, tape_saturation, tube_saturation, exponential_saturation};
    let sat_types = [
        SaturationType::Tape,
        SaturationType::Tube,
        SaturationType::SoftClip,
        SaturationType::Exponential,
    ];
    let segment_len = (SAMPLE_RATE as f32 * (0.02 + amount * 0.08)) as usize;
    let max_segments = (samples.len() / segment_len.max(1)).max(1);
    let mut rng_state = s;
    for seg in 0..max_segments {
        let start = seg * segment_len;
        let end = (start + segment_len).min(samples.len());
        if start >= end { break; }
        let sat_idx = ((rng_state * 3.99) as usize).min(3);
        rng_state = frac(rng_state * 19.7 + 0.3);
        let drive_var = 1.0 + (frac(rng_state * 23.1) - 0.5) * amount * 0.3;
        rng_state = frac(rng_state * 29.3 + 0.7);
        let drive = 1.0 + amount * 0.5 * drive_var;
        let sat_fn: fn(f32, f32) -> f32 = match sat_types[sat_idx] {
            SaturationType::Tape => tape_saturation,
            SaturationType::Tube => tube_saturation,
            SaturationType::SoftClip => soft_clip,
            SaturationType::Exponential => exponential_saturation,
            _ => soft_clip,
        };
        for i in start..end {
            samples[i] = sat_fn(samples[i], drive);
        }
    }
}

pub fn apply_layer_breathing(samples: &mut [f32], amount: f32, seed: u64, sample_rate: u32) {
    if amount <= 0.001 || samples.is_empty() { return; }
    let s = seed as f32;
    let rate_1 = 0.5 + frac(s * 2.3) * 2.0;
    let rate_2 = 2.0 + frac(s * 5.7) * 3.0;
    let depth = amount * 0.04;
    for i in 0..samples.len() {
        let t = i as f32 / sample_rate as f32;
        let mod_1 = (2.0 * PI * rate_1 * t).sin();
        let mod_2 = (2.0 * PI * rate_2 * t).sin() * 0.3;
        let modulation = 1.0 + (mod_1 + mod_2) * depth;
        samples[i] *= modulation;
    }
}

pub fn apply_phase_variation(samples: &mut [f32], amount: f32, seed: u64, sample_rate: u32) {
    if amount <= 0.001 || samples.len() < 128 { return; }
    let s = seed as f32;
    let block_len = (sample_rate as f32 * (0.005 + amount * 0.015)) as usize;
    let max_block = (samples.len() / block_len.max(1)).max(1);
    let mut rng_state = s;
    for block in 0..max_block {
        let start = block * block_len;
        let end = (start + block_len).min(samples.len());
        if start >= end { break; }
        let phase_shift = (rng_state - 0.5) * 2.0 * amount * 0.3;
        rng_state = frac(rng_state * 31.7 + 0.5);
        for i in start..end {
            let rel = (i - start) as f32 / (end - start).max(1) as f32;
            let t = i as f32 / sample_rate as f32;
            let morph = rel * phase_shift;
            let pm = (2.0 * PI * 0.5 * t + morph).sin() * 0.5 + 0.5;
            samples[i] *= 1.0 + (pm - 0.5) * amount * 0.02;
        }
    }
}

pub fn apply_humanized_transients(samples: &mut [f32], amount: f32, seed: u64) {
    if amount <= 0.001 || samples.len() < 128 { return; }
    let s = seed as f32;
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return; }
    let threshold = peak * 0.15;
    let onset_len = (SAMPLE_RATE as f32 * 0.006) as usize;
    let mut rng_state = s;
    let mut i = 0;
    while i < samples.len().min(SAMPLE_RATE as usize) {
        if samples[i].abs() > threshold {
            let onset_end = (i + onset_len).min(samples.len());
            let onset_peak = samples[i..onset_end].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            if onset_peak > 0.001 {
                let pre_rand = frac(rng_state * 37.1);
                rng_state = frac(rng_state * 41.3 + 0.1);
                for j in i..onset_end {
                    let t = (j - i) as f32 / (onset_end - i).max(1) as f32;
                    let pre_ring = (2.0 * PI * 2000.0 * (pre_rand * 0.5 + 0.75) * t).sin()
                        * (1.0 - t) * amount * 0.03;
                    let scatter = frac(rng_state * 43.7 + j as f32 * 0.01) * amount * 0.02;
                    samples[j] += pre_ring + scatter * onset_peak;
                }
                let skip = (pre_rand * amount * 3.0 * onset_len as f32) as usize;
                i = onset_end + skip;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
}

pub fn humanize(samples: &mut [f32], params: &HumanizeParams, seed: u64) {
    if samples.is_empty() { return; }
    humanize_internal(samples, params, seed, SAMPLE_RATE);
}

fn humanize_internal(samples: &mut [f32], params: &HumanizeParams, seed: u64, sample_rate: u32) {
    if params.analog_drift > 0.001 {
        let drift_seed = seed.wrapping_mul(2);
        apply_analog_drift(samples, params.analog_drift, sample_rate, drift_seed);
    }
    if params.humanize_transients > 0.001 {
        let ht_seed = seed.wrapping_mul(3);
        apply_humanized_transients(samples, params.humanize_transients, ht_seed);
    }
    if params.envelope_variation > 0.001 {
        let ev_seed = seed.wrapping_mul(5);
        apply_envelope_variation(samples, params.envelope_variation, ev_seed);
    }
    if params.instability > 0.001 {
        let ins_seed = seed.wrapping_mul(7);
        apply_instability(samples, params.instability, ins_seed);
    }
    if params.transient_randomness > 0.001 {
        let tr_seed = seed.wrapping_mul(11);
        apply_transient_randomness(samples, params.transient_randomness, tr_seed);
    }
    if params.non_static_layers > 0.001 {
        let ns_seed = seed.wrapping_mul(13);
        apply_layer_breathing(samples, params.non_static_layers, ns_seed, sample_rate);
    }
    if params.saturation_randomness > 0.001 {
        let sr_seed = seed.wrapping_mul(17);
        apply_saturation_randomness(samples, params.saturation_randomness, sr_seed);
    }
    if params.phase_variation > 0.001 {
        let pv_seed = seed.wrapping_mul(19);
        apply_phase_variation(samples, params.phase_variation, pv_seed, sample_rate);
    }
}

pub fn compute_humanize_from_prompt(prompt: &str) -> HumanizeParams {
    let lower = prompt.to_lowercase();
    let mut hp = HumanizeParams::default();

    if lower.contains("analog") {
        hp = HumanizeParams::warm().scaled(0.8);
    }
    if lower.contains("vintage") || lower.contains("classic") {
        hp = HumanizeParams::vintage().scaled(0.9);
    }
    if lower.contains("lo-fi") || lower.contains("lofi") || lower.contains("lo fi") {
        hp = HumanizeParams::lo_fi().scaled(1.0);
    }
    if lower.contains("human") || lower.contains("natural") || lower.contains("organic") {
        hp.analog_drift = hp.analog_drift.max(0.2);
        hp.envelope_variation = hp.envelope_variation.max(0.15);
        hp.humanize_transients = hp.humanize_transients.max(0.2);
        hp.transient_randomness = hp.transient_randomness.max(0.15);
    }
    if lower.contains("warm") {
        hp.analog_drift = hp.analog_drift.max(0.1);
        hp.saturation_randomness = hp.saturation_randomness.max(0.1);
        hp.instability = hp.instability.max(0.03);
    }
    if lower.contains("tape") {
        hp.analog_drift = hp.analog_drift.max(0.3);
        hp.instability = hp.instability.max(0.08);
        hp.non_static_layers = hp.non_static_layers.max(0.1);
    }
    if lower.contains("dust") || lower.contains("crackle") {
        hp.instability = hp.instability.max(0.2);
    }
    if lower.contains("alive") || lower.contains("breathing") || lower.contains("live") {
        hp.non_static_layers = hp.non_static_layers.max(0.2);
        hp.envelope_variation = hp.envelope_variation.max(0.12);
    }
    if lower.contains("imperfect") || lower.contains("raw") || lower.contains("rough") {
        hp.transient_randomness = hp.transient_randomness.max(0.2);
        hp.humanize_transients = hp.humanize_transients.max(0.2);
        hp.instability = hp.instability.max(0.1);
    }
    if lower.contains("sterile") || lower.contains("digital") || lower.contains("clean") {
        hp = HumanizeParams::default();
    }

    hp
}
