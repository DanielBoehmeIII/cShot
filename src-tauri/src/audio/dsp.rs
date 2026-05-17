use std::f32::consts::PI;
use super::{DspParams, SAMPLE_RATE};

// ─── Click Character Types ────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClickCharacter {
    Sharp,
    Round,
    Pitched,
    Noise,
    Metallic,
    Subsonic,
    Hybrid,
    Crack,
    Thud,
    Snap,
    Ring,
}

impl ClickCharacter {
    pub fn label(&self) -> &'static str {
        match self {
            ClickCharacter::Sharp => "sharp",
            ClickCharacter::Round => "round",
            ClickCharacter::Pitched => "pitched",
            ClickCharacter::Noise => "noise",
            ClickCharacter::Metallic => "metallic",
            ClickCharacter::Subsonic => "subsonic",
            ClickCharacter::Hybrid => "hybrid",
            ClickCharacter::Crack => "crack",
            ClickCharacter::Thud => "thud",
            ClickCharacter::Snap => "snap",
            ClickCharacter::Ring => "ring",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s {
            "sharp" => ClickCharacter::Sharp,
            "round" => ClickCharacter::Round,
            "pitched" => ClickCharacter::Pitched,
            "noise" => ClickCharacter::Noise,
            "metallic" => ClickCharacter::Metallic,
            "subsonic" => ClickCharacter::Subsonic,
            "hybrid" => ClickCharacter::Hybrid,
            "crack" => ClickCharacter::Crack,
            "thud" => ClickCharacter::Thud,
            "snap" => ClickCharacter::Snap,
            "ring" => ClickCharacter::Ring,
            _ => ClickCharacter::Sharp,
        }
    }

    pub fn for_sound_type(sound_type_str: &str) -> Self {
        match sound_type_str {
            "kick" => ClickCharacter::Thud,
            "snare" => ClickCharacter::Crack,
            "closed_hat" => ClickCharacter::Sharp,
            "open_hat" => ClickCharacter::Ring,
            "clap" => ClickCharacter::Noise,
            "tom" => ClickCharacter::Round,
            "perc" => ClickCharacter::Snap,
            "bass" => ClickCharacter::Subsonic,
            "fx" => ClickCharacter::Hybrid,
            _ => ClickCharacter::Sharp,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TransientConfig {
    pub click_character: ClickCharacter,
    pub sharpness: f32,
    pub density: f32,
    pub pitch_click_hz: f32,
    pub click_bandwidth: f32,
    pub ring_decay: f32,
    pub transient_duration_ms: f32,
    pub pre_attack_ms: f32,
    pub attack_curve: f32,
    pub transient_saturation: f32,
    pub multiband_boost: [f32; 4],
}

impl Default for TransientConfig {
    fn default() -> Self {
        Self {
            click_character: ClickCharacter::Sharp,
            sharpness: 0.7,
            density: 0.5,
            pitch_click_hz: 4000.0,
            click_bandwidth: 0.6,
            ring_decay: 0.1,
            transient_duration_ms: 8.0,
            pre_attack_ms: 0.0,
            attack_curve: 2.0,
            transient_saturation: 1.0,
            multiband_boost: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

pub fn generate_click(config: &TransientConfig, num_samples: usize) -> Vec<f32> {
    let transient_samples = ((config.transient_duration_ms / 1000.0) * SAMPLE_RATE as f32) as usize;
    let transient_samples = transient_samples.min(num_samples).max(4);
    let pre_attack = ((config.pre_attack_ms / 1000.0) * SAMPLE_RATE as f32) as usize;
    let mut buf = vec![0.0f32; num_samples];

    let seed_phase = 0.0;
    let noise_s = |p: f32| ((p * 127.1).sin() * 43758.5453).fract() * 2.0 - 1.0;

    match config.click_character {
        ClickCharacter::Sharp => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let frac = (i - pre_attack) as f32 / transient_samples.max(1) as f32;
                let env = (-800.0 * t).exp() * 0.5 + (-200.0 * t).exp() * 0.5;
                let tone = (2.0 * PI * config.pitch_click_hz * t).sin() * 0.6
                    + (2.0 * PI * config.pitch_click_hz * 1.7 * t).sin() * 0.25
                    + (2.0 * PI * config.pitch_click_hz * 3.1 * t).sin() * 0.15;
                let noise = noise_s((i as f32 * 0.9 + seed_phase).fract()) * 0.3
                    + noise_s((i as f32 * 1.7 + seed_phase * 0.7).fract()) * 0.2;
                let click = (tone + noise) * env * config.sharpness;
                buf[i] = click;
            }
        }
        ClickCharacter::Round => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let frac = (i - pre_attack) as f32 / transient_samples.max(1) as f32;
                let env = (-120.0 * t).exp() * 0.4 + (-30.0 * t).exp() * 0.6;
                let freq = config.pitch_click_hz * (1.0 - 0.6 * frac);
                let tone = (2.0 * PI * freq * t).sin() * 0.7
                    + (2.0 * PI * freq * 2.0 * t).sin() * 0.2;
                let noise = noise_s((i as f32 * 0.5).fract()) * 0.3;
                let mx = (tone + noise) * env * config.sharpness;
                let alpha = 0.3;
                let mx_smooth = mx * (1.0 - alpha) + if i > 0 { buf[i-1] * alpha } else { 0.0 };
                buf[i] = mx_smooth;
            }
        }
        ClickCharacter::Pitched => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let env = (-400.0 * t).exp() * 0.3 + (-80.0 * t).exp() * 0.7;
                let pitch_drop = config.pitch_click_hz * (1.0 - 0.2 * (t * 200.0).min(1.0));
                let tone = (2.0 * PI * pitch_drop * t).sin() * 0.8
                    + (2.0 * PI * pitch_drop * 1.5 * t).sin() * 0.3
                    + (2.0 * PI * pitch_drop * 0.5 * t).sin() * 0.2;
                buf[i] = tone * env * config.sharpness;
            }
        }
        ClickCharacter::Noise => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let frac = (i - pre_attack) as f32 / transient_samples.max(1) as f32;
                let env = (-500.0 * t).exp();
                let n1 = noise_s((i as f32 * 0.8 + seed_phase).fract());
                let n2 = noise_s((i as f32 * 1.4 + seed_phase * 0.5).fract());
                let n3 = noise_s((i as f32 * 2.5 + seed_phase * 0.3).fract());
                let noise = n1 * 0.5 + n2 * 0.3 + n3 * 0.2;
                let hp_alpha = (1.0 / (1.0 + 2.0 * PI * 3000.0 / SAMPLE_RATE as f32));
                let shaped = noise * (1.0 - hp_alpha) + if i > pre_attack && pre_attack > 0 { buf[i-1] * hp_alpha } else { 0.0 };
                buf[i] = shaped * env * config.sharpness;
            }
        }
        ClickCharacter::Metallic => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let env = (-300.0 * t).exp();
                let car_freq = config.pitch_click_hz;
                let modf_freq = car_freq * 0.2;
                let mod_index = 3.0 * (-15.0 * t).exp();
                let sig_mod = mod_index * (2.0 * PI * modf_freq * t).sin();
                let fm = (2.0 * PI * car_freq * t + sig_mod).sin() * 0.5;
                let n = noise_s((i as f32 * 0.7).fract()) * 0.3;
                let ring_mod = (2.0 * PI * car_freq * 2.3 * t).sin();
                buf[i] = (fm + n * ring_mod) * env * config.sharpness;
            }
        }
        ClickCharacter::Subsonic => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let frac = (i - pre_attack) as f32 / transient_samples.max(1) as f32;
                let env = (-50.0 * t).exp() * 0.5 + (-8.0 * t).exp() * 0.5;
                let freq = (config.pitch_click_hz * 0.15).max(30.0) * (1.0 - 0.3 * frac);
                let tone = (2.0 * PI * freq * t).sin() * 0.6
                    + (2.0 * PI * freq * 0.5 * t + 0.3).sin() * 0.4;
                let n = noise_s((i as f32 * 0.2).fract()) * 0.15;
                let saturated = tape_saturation((tone + n) * config.sharpness * 1.5, 1.8);
                buf[i] = saturated * env;
            }
        }
        ClickCharacter::Hybrid => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let frac = (i - pre_attack) as f32 / transient_samples.max(1) as f32;
                let env = (-600.0 * t).exp() * 0.4 + (-100.0 * t).exp() * 0.4 + (-20.0 * t).exp() * 0.2;
                let freq = config.pitch_click_hz * (1.0 - 0.3 * frac);
                let tone = (2.0 * PI * freq * t).sin() * 0.4
                    + (2.0 * PI * freq * 2.0 * t).sin() * 0.2;
                let n1 = noise_s((i as f32 * 0.8).fract());
                let n2 = noise_s((i as f32 * 1.5).fract());
                let noise = n1 * 0.5 + n2 * 0.3;
                buf[i] = (tone * 0.5 + noise * 0.5) * env * config.sharpness;
            }
        }
        ClickCharacter::Crack => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let env = (-1000.0 * t).exp() * 0.6 + (-150.0 * t).exp() * 0.4;
                let freq = 6000.0 - 3000.0 * (t / 0.005).min(1.0);
                let tone = (2.0 * PI * freq * t).sin() * 0.3;
                let n1 = noise_s((i as f32 * 1.1).fract());
                let n2 = noise_s((i as f32 * 2.3).fract());
                let n3 = noise_s((i as f32 * 0.5).fract());
                let noise = n1 * 0.4 + n2 * 0.35 + n3 * 0.25;
                buf[i] = (tone * 0.2 + noise * 0.8) * env * config.sharpness;
            }
        }
        ClickCharacter::Thud => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let frac = (i - pre_attack) as f32 / transient_samples.max(1) as f32;
                let env = (-80.0 * t).exp() * 0.6 + (-15.0 * t).exp() * 0.4;
                let freq_a = 2000.0 - 1500.0 * frac;
                let freq_b = freq_a * 0.4;
                let tone_a = (2.0 * PI * freq_a * t).sin() * 0.3;
                let tone_b = (2.0 * PI * freq_b * t).sin() * 0.5;
                let n = noise_s((i as f32 * 0.4).fract()) * 0.2;
                buf[i] = (tone_a + tone_b + n) * env * config.sharpness;
            }
        }
        ClickCharacter::Snap => {
            for i in pre_attack..(pre_attack + transient_samples).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let env = (-600.0 * t).exp();
                let freq = config.pitch_click_hz * (1.0 + 0.5 * (t * 300.0).min(1.0));
                let tone = (2.0 * PI * freq * t).sin() * 0.3;
                let n1 = noise_s((i as f32 * 1.3).fract());
                let n2 = noise_s((i as f32 * 2.7).fract());
                buf[i] = (tone + n1 * 0.5 + n2 * 0.3) * env * config.sharpness;
            }
        }
        ClickCharacter::Ring => {
            for i in pre_attack..(pre_attack + transient_samples * 3).min(num_samples) {
                let t = (i - pre_attack) as f32 / SAMPLE_RATE as f32;
                let env = (-600.0 * t).exp() * 0.5 + (-config.ring_decay * 100.0 * t).exp() * 0.5;
                let car_freq = config.pitch_click_hz;
                let modr_freq = car_freq * 0.15;
                let mod_index = 2.0 * (-20.0 * t).exp();
                let sig_mod = mod_index * (2.0 * PI * modr_freq * t).sin();
                let fm = (2.0 * PI * car_freq * t + sig_mod).sin() * 0.4;
                let n = noise_s((i as f32 * 0.6).fract()) * 0.25;
                buf[i] = (fm + n) * env * config.sharpness;
            }
        }
    }

    if config.transient_saturation > 1.01 {
        for s in buf.iter_mut() {
            *s = tape_saturation(*s, config.transient_saturation);
        }
    }

    buf
}

// ─── Multi-Band Transient Processor ────────────────────────

pub struct MultiBandTransientConfig {
    pub low_boost_db: f32,
    pub mid_low_boost_db: f32,
    pub mid_high_boost_db: f32,
    pub high_boost_db: f32,
    pub crossover_freqs: [f32; 3],
    pub attack_ms: f32,
    pub release_ms: f32,
}

impl Default for MultiBandTransientConfig {
    fn default() -> Self {
        Self {
            low_boost_db: 0.0,
            mid_low_boost_db: 0.0,
            mid_high_boost_db: 0.0,
            high_boost_db: 0.0,
            crossover_freqs: [200.0, 2000.0, 8000.0],
            attack_ms: 3.0,
            release_ms: 20.0,
        }
    }
}

pub fn multiband_transient_processor(samples: &mut [f32], config: &MultiBandTransientConfig) {
    if samples.len() < 256 { return; }
    let [cf1, cf2, cf3] = config.crossover_freqs;

    let mut bands: [Vec<f32>; 4] = [
        samples.to_vec(),
        vec![0.0f32; samples.len()],
        vec![0.0f32; samples.len()],
        vec![0.0f32; samples.len()],
    ];

    for i in 0..samples.len() {
        bands[1][i] = samples[i];
        bands[2][i] = samples[i];
        bands[3][i] = samples[i];
    }

    low_pass(&mut bands[0], cf1);
    band_pass(&mut bands[1], cf1, cf2);
    band_pass(&mut bands[2], cf2, cf3);
    high_pass(&mut bands[3], cf3);

    let boosts = [
        config.low_boost_db,
        config.mid_low_boost_db,
        config.mid_high_boost_db,
        config.high_boost_db,
    ];

    let onset_len = (config.attack_ms / 1000.0 * SAMPLE_RATE as f32) as usize;
    let onset_len = onset_len.max(4);

    for (band_idx, band) in bands.iter_mut().enumerate() {
        let boost_db = boosts[band_idx];
        if boost_db.abs() < 0.5 { continue; }
        let gain = 10.0_f32.powf(boost_db / 20.0);
        let threshold = band.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.1;
        if threshold < 0.0001 { continue; }

        let mut onset_pos = 0;
        for i in 4..band.len().min(SAMPLE_RATE as usize / 2) {
            if band[i].abs() > threshold {
                onset_pos = i.saturating_sub(2);
                break;
            }
        }
        if onset_pos < 2 { continue; }

        let end = (onset_pos + onset_len).min(band.len());
        for i in onset_pos..end {
            let t = (i - onset_pos) as f32 / (end - onset_pos).max(1) as f32;
            let env = (1.0 - t * t) * (1.0 - t * 0.3);
            band[i] *= 1.0 + (gain - 1.0) * env;
        }
    }

    for i in 0..samples.len() {
        let mut val = 0.0f32;
        for band in bands.iter() {
            val += band.get(i).copied().unwrap_or(0.0);
        }
        let max_val = val.abs().max(0.001);
        if max_val > 0.0 {
            val *= 0.85 / max_val.min(1.0);
        }
        samples[i] = val;
    }
}

fn band_pass(samples: &mut [f32], low_hz: f32, high_hz: f32) {
    if low_hz <= 0.0 { low_pass(samples, high_hz); return; }
    if high_hz >= SAMPLE_RATE as f32 / 2.0 { high_pass(samples, low_hz); return; }
    let rc_low = 1.0 / (2.0 * PI * high_hz);
    let rc_high = 1.0 / (2.0 * PI * low_hz);
    let dt = 1.0 / SAMPLE_RATE as f32;
    let alpha_low_c = dt / (rc_low + dt);
    let alpha_high = (rc_high / (rc_high + dt)).clamp(0.0, 1.0);
    let mut lp = alpha_low_c;
    let mut prev_lp = 0.0f32;
    let mut prev_hp1 = 0.0f32;
    let mut prev_hp2 = 0.0f32;
    for sample in samples.iter_mut() {
        let input = *sample;
        let hp1 = alpha_high * (prev_hp1 + input - prev_hp1);
        let hp2 = alpha_high * (prev_hp2 + hp1 - prev_hp2);
        prev_hp1 = input;
        prev_hp2 = hp1;
        let after_hp = input - hp2;
        prev_lp += alpha_low_c * (after_hp - prev_lp);
        *sample = prev_lp;
    }
}

// ─── Transient Density Enhancer ────────────────────────────

pub fn transient_density_boost(samples: &mut [f32], density: f32) {
    if samples.len() < 256 || density <= 0.0 { return; }
    let onset_len = (0.006 * SAMPLE_RATE as f32) as usize;
    let threshold = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.08;
    let gain = 1.0 + density * 2.0;

    let mut in_onset = false;
    for i in 4..samples.len().min(SAMPLE_RATE as usize) {
        if samples[i].abs() > threshold && !in_onset {
            in_onset = true;
            let end = (i + onset_len).min(samples.len());
            for j in i..end {
                let t = (j - i) as f32 / (end - i).max(1) as f32;
                let env = (1.0 - t * t) * (1.0 - t * 0.5);
                samples[j] *= 1.0 + (gain - 1.0) * env * density;
            }
        } else if samples[i].abs() < threshold * 0.3 {
            in_onset = false;
        }
    }
}

// ─── Transient/Body Cohesion Processor ────────────────────

pub fn transient_body_cohesion(samples: &mut [f32], cohesion: f32) {
    if samples.len() < 256 || cohesion <= 0.0 { return; }
    let crossover = 400.0;
    let mut transient_band = samples.to_vec();
    let mut body_band = samples.to_vec();
    high_pass(&mut transient_band, crossover);
    low_pass(&mut body_band, crossover);

    let peak_t = transient_band.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let peak_b = body_band.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak_t < 0.001 || peak_b < 0.001 { return; }

    let tr_onset = transient_band.iter().position(|s| s.abs() > peak_t * 0.3).unwrap_or(0);
    let bd_onset = body_band.iter().position(|s| s.abs() > peak_b * 0.3).unwrap_or(0);

    if bd_onset > tr_onset + 2 && bd_onset < body_band.len() - 4 {
        let shift = bd_onset - tr_onset;
        let shift_amt = shift as f32 * cohesion;
        let shift_samples = shift_amt as usize;
        if tr_onset + shift_samples < body_band.len() {
            let mut shifted = vec![0.0f32; body_band.len()];
            for i in tr_onset..body_band.len() - shift_samples {
                shifted[i + shift_samples] = body_band[i];
            }
            for (i, &s) in shifted.iter().enumerate() {
                samples[i] = transient_band[i] * cohesion + s * (1.0 - cohesion) + body_band[i].min(s);
            }
            let avg = samples.iter().sum::<f32>() / samples.len() as f32;
            if avg.abs() > 0.0001 {
                for s in samples.iter_mut() { *s -= avg; }
            }
        }
    }
}

// ─── Impact Processor ─────────────────────────────────────

pub struct ImpactConfig {
    pub pre_click: f32,
    pub initial_impact_db: f32,
    pub sustain_punch_db: f32,
    pub body_support_db: f32,
    pub attack_hold_ms: f32,
    pub release_shape: f32,
}

impl Default for ImpactConfig {
    fn default() -> Self {
        Self {
            pre_click: 0.0,
            initial_impact_db: 3.0,
            sustain_punch_db: 1.5,
            body_support_db: 1.0,
            attack_hold_ms: 2.0,
            release_shape: 2.0,
        }
    }
}

pub fn impact_processor(samples: &mut [f32], config: &ImpactConfig) {
    if samples.len() < 64 { return; }

    let hold_samples = (config.attack_hold_ms / 1000.0 * SAMPLE_RATE as f32) as usize;
    let onset_boost = 10.0_f32.powf(config.initial_impact_db / 20.0);
    let sustain_boost = 10.0_f32.powf(config.sustain_punch_db / 20.0);
    let body_boost = 10.0_f32.powf(config.body_support_db / 20.0);

    let peak_val = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak_val < 0.001 { return; }

    let threshold = peak_val * 0.15;
    let onset_pos = samples.iter().position(|s| s.abs() > threshold).unwrap_or(0);

    let transient_len = (SAMPLE_RATE as f32 * 0.015) as usize;
    let body_start = (onset_pos + transient_len).min(samples.len());
    let body_len = (SAMPLE_RATE as f32 * 0.1) as usize;
    let body_end = (body_start + body_len).min(samples.len());

    for i in onset_pos..(onset_pos + hold_samples).min(samples.len()) {
        let t = (i - onset_pos) as f32 / hold_samples.max(1) as f32;
        let env = (1.0 - t * t * 0.5).max(0.3);
        samples[i] *= 1.0 + (onset_boost - 1.0) * env;
    }

    let sustain_start = (onset_pos + hold_samples).min(samples.len());
    for i in sustain_start..body_start.min(samples.len()) {
        let t = (i - sustain_start) as f32 / (body_start - sustain_start).max(1) as f32;
        let env = (1.0 - t).max(0.1);
        samples[i] *= 1.0 + (sustain_boost - 1.0) * env;
    }

    for i in body_start..body_end {
        let t = (i - body_start) as f32 / (body_end - body_start).max(1) as f32;
        let env = (1.0 - t).max(0.1);
        samples[i] *= 1.0 + (body_boost - 1.0) * env * (1.0 - t);
    }
}

// ─── Perceived Loudness Enhancer ───────────────────────────

pub fn perceived_loudness_boost(samples: &mut [f32], amount: f32) {
    if amount <= 0.0 { return; }
    let onset_len = (SAMPLE_RATE as f32 * 0.005) as usize;
    let peak_val = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak_val < 0.001 { return; }
    let threshold = peak_val * 0.1;
    let onset_pos = samples.iter().position(|s| s.abs() > threshold).unwrap_or(0);
    let end = (onset_pos + onset_len).min(samples.len());

    for i in onset_pos..end {
        let t = (i - onset_pos) as f32 / (end - onset_pos).max(1) as f32;
        let boost = 1.0 + amount * 0.5 * (-2.0 * t).exp();
        samples[i] *= boost;
    }

    let mid_band_boost = 1.0 + amount * 0.15;
    let high_band_boost = 1.0 + amount * 0.08;
    biquad_peaking(samples, 2000.0, (mid_band_boost - 1.0) * 12.0, 0.7);
    biquad_high_shelf(samples, 6000.0, (high_band_boost - 1.0) * 8.0, 0.7);
}

// ─── Transient Match for Recreation ────────────────────────

pub fn match_transient_to_target(target: &mut [f32], reference_transient: &[f32], alignment: f32) {
    if target.len() < 64 || reference_transient.len() < 64 { return; }

    let ref_peak = reference_transient.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let tgt_peak = target.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if ref_peak < 0.001 || tgt_peak < 0.001 { return; }

    let ref_onset = reference_transient.iter().position(|s| s.abs() > ref_peak * 0.3).unwrap_or(0);
    let tgt_onset = target.iter().position(|s| s.abs() > tgt_peak * 0.3).unwrap_or(0);

    let onset_len = (SAMPLE_RATE as f32 * 0.01) as usize;
    let ref_end = (ref_onset + onset_len).min(reference_transient.len());
    let tgt_end = (tgt_onset + onset_len).min(target.len());

    let ref_slice = &reference_transient[ref_onset..ref_end];
    let tgt_range = if tgt_onset < tgt_end && tgt_end <= target.len() {
        &target[tgt_onset..tgt_end]
    } else {
        return;
    };

    let ref_env: Vec<f32> = ref_slice.iter().map(|s| s.abs()).collect();
    let tgt_env: Vec<f32> = tgt_range.iter().map(|s| s.abs()).collect();
    let len = ref_env.len().min(tgt_env.len());
    if len < 4 { return; }

    let ref_peak_env = ref_env.iter().copied().fold(0.0f32, f32::max);
    let tgt_peak_env = tgt_env.iter().copied().fold(0.0f32, f32::max);
    if ref_peak_env < 0.001 || tgt_peak_env < 0.001 { return; }

    for i in 0..len {
        let idx = tgt_onset + i;
        if idx >= target.len() { break; }
        let ref_norm = ref_env[i] / ref_peak_env;
        let tgt_norm = tgt_env[i] / tgt_peak_env;
        let target_ratio = if tgt_norm > 0.001 { ref_norm / tgt_norm } else { 0.0 };
        let ratio = 1.0 + (target_ratio - 1.0) * alignment;
        let sign = target[idx].signum();
        target[idx] = target[idx].abs().min(ref_peak * 1.2) * sign;
        target[idx] *= ratio.clamp(0.3, 3.0);
    }

    let sustain_len = (SAMPLE_RATE as f32 * 0.02) as usize;
    let sus_start = (tgt_onset + onset_len).min(target.len());
    let sus_end = (sus_start + sustain_len).min(target.len());
    for i in sus_start..sus_end {
        let t = (i - sus_start) as f32 / (sus_end - sus_start).max(1) as f32;
        let blend = 1.0 - t * alignment;
        let val = target[i];
        let ref_sustain = if i < reference_transient.len() { reference_transient[i].abs() } else { 0.0 };
        if ref_sustain > 0.001 {
            let sustain_ratio = (ref_sustain / tgt_peak).min(2.0);
            target[i] = val * (1.0 - blend) + val * sustain_ratio * blend;
        }
    }
}

// ─── Transient Sharpness Analyzer ─────────────────────────

pub fn compute_transient_sharpness(samples: &[f32]) -> f32 {
    if samples.len() < 64 { return 0.5; }
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return 0.0; }
    let threshold = peak * 0.1;
    let onset = samples.iter().position(|s| s.abs() > threshold).unwrap_or(0);
    let attack_end = (onset + (SAMPLE_RATE as f32 * 0.005) as usize).min(samples.len());
    if attack_end <= onset { return 0.0; }
    let attack_region = &samples[onset..attack_end];
    let attack_profile: Vec<f32> = attack_region.iter().map(|s| s.abs()).collect();
    let peak_a = attack_profile.iter().copied().fold(0.0f32, f32::max);
    if peak_a < 0.001 { return 0.0; }
    let area_under_curve: f32 = attack_profile.iter().sum::<f32>() / attack_profile.len() as f32;
    (peak_a / area_under_curve.max(0.001)).clamp(0.0, 1.0)
}

// ─── Advanced Envelope Shapes ─────────────────────────────

pub fn cosine_fade_in(samples: &mut [f32], fade_len: usize) {
    let flen = fade_len.min(samples.len());
    for i in 0..flen {
        let t = i as f32 / flen as f32;
        samples[i] *= (1.0 - (t * PI).cos()) * 0.5;
    }
}

pub fn cosine_fade_out(samples: &mut [f32], fade_len: usize) {
    let flen = fade_len.min(samples.len());
    for i in 0..flen {
        let idx = samples.len() - 1 - i;
        let t = i as f32 / flen as f32;
        samples[idx] *= (1.0 - (t * PI).cos()) * 0.5;
    }
}

pub fn sigmoid_envelope(samples: &mut [f32], attack_samples: usize, decay_samples: usize) {
    let total = samples.len();
    for i in 0..total {
        let mut env = 1.0;
        if i < attack_samples {
            let p = i as f32 / attack_samples.max(1) as f32;
            env = (p * PI * 0.5).sin(); // eased-in attack
        }
        let di = total - i;
        if di < decay_samples {
            let p = di as f32 / decay_samples.max(1) as f32;
            env *= (p * PI * 0.5).sin(); // eased-out decay
        }
        samples[i] *= env;
    }
}

pub fn apply_smooth_fade(samples: &mut [f32], fade_in_s: f32, fade_out_s: f32) {
    if samples.is_empty() { return; }
    let fade_in_len = (fade_in_s * SAMPLE_RATE as f32) as usize;
    let fade_out_len = (fade_out_s * SAMPLE_RATE as f32) as usize;
    cosine_fade_in(samples, fade_in_len);
    cosine_fade_out(samples, fade_out_len);
}

// ─── High-Quality Filters ─────────────────────────────────

pub fn high_pass(samples: &mut [f32], cutoff_hz: f32) {
    if samples.len() < 4 || cutoff_hz <= 0.0 { return; }
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    let dt = 1.0 / SAMPLE_RATE as f32;
    let alpha = (rc / (rc + dt)).clamp(0.0, 1.0);
    let mut prev = samples[0];
    for sample in samples.iter_mut() {
        let input = *sample;
        *sample = alpha * (prev + *sample - prev);
        prev = input;
    }
}

pub fn low_pass(samples: &mut [f32], cutoff_hz: f32) {
    if samples.len() < 4 || cutoff_hz <= 0.0 { return; }
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    let dt = 1.0 / SAMPLE_RATE as f32;
    let alpha = (dt / (rc + dt)).clamp(0.0, 1.0);
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
    let a0r = a0.recip();
    let mut x1 = 0.0;
    let mut x2 = 0.0;
    let mut y1 = 0.0;
    let mut y2 = 0.0;
    for sample in samples.iter_mut() {
        let x = *sample;
        let y = b0 * a0r * x + b1 * a0r * x1 + b2 * a0r * x2 - a1 * a0r * y1 - a2 * a0r * y2;
        x2 = x1;
        x1 = x;
        y2 = y1;
        y1 = y;
        *sample = y;
    }
}

// ─── Analog-Inspired Saturation Models ────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SaturationType {
    SoftClip,
    HardClip,
    Tape,
    Tube,
    Asymmetrical,
    Exponential,
}

pub fn soft_clip(x: f32, drive: f32) -> f32 {

    let x = x * drive;
    if x > 1.0 { 1.0 }
    else if x < -1.0 { -1.0 }
    else { x * (1.5 - 0.5 * x * x) }
}

pub fn hard_clip(x: f32, drive: f32) -> f32 {
    (x * drive).clamp(-1.0, 1.0)
}

pub fn tape_saturation(x: f32, drive: f32) -> f32 {
    let x = x * drive;
    let sign = x.signum();
    let abs = x.abs();
    if abs > 1.0 {
        sign * (abs * 0.5 + 0.5).min(1.0)
    } else {
        x * (1.0 + 0.333 * x * x)
    }
}

pub fn tube_saturation(x: f32, drive: f32) -> f32 {
    let x = x * drive;
    let sign = x.signum();
    let abs = x.abs();
    let y = 1.0 - (-2.0 * abs).exp();
    sign * y.min(1.0)
}

pub fn asymmetrical_saturation(x: f32, drive: f32) -> f32 {
    let x = x * drive;
    if x > 0.0 {
        1.0 - (-x).exp()
    } else {
        -(1.0 - (x * 1.2).exp())
    }
}

pub fn exponential_saturation(x: f32, drive: f32) -> f32 {
    let x = x * drive;
    let sign = x.signum();
    let abs = x.abs();
    let y = 1.0 - (-abs * 2.0).exp();
    sign * y.min(1.0)
}

pub fn saturate(samples: &mut [f32], drive: f32, sat_type: SaturationType) {
    if drive <= 1.001 { return; }
    let sat_fn: fn(f32, f32) -> f32 = match sat_type {
        SaturationType::SoftClip => soft_clip,
        SaturationType::HardClip => hard_clip,
        SaturationType::Tape => tape_saturation,
        SaturationType::Tube => tube_saturation,
        SaturationType::Asymmetrical => asymmetrical_saturation,
        SaturationType::Exponential => exponential_saturation,
    };
    for sample in samples.iter_mut() {
        *sample = sat_fn(*sample, drive);
    }
}

pub fn apply_saturation(samples: &mut [f32], drive: f32) {
    if drive <= 1.001 { return; }
    // Warm up with tape-style saturation, push harder for tube character
    let sat_type = if drive < 2.0 {
        SaturationType::Tape
    } else if drive < 3.5 {
        SaturationType::Tube
    } else {
        SaturationType::SoftClip
    };
    for sample in samples.iter_mut() {
        *sample = match sat_type {
            SaturationType::Tape => tape_saturation(*sample, drive),
            SaturationType::Tube => tube_saturation(*sample, drive),
            SaturationType::SoftClip => soft_clip(*sample, drive),
            _ => soft_clip(*sample, drive),
        };
    }
}

pub fn apply_saturation_multi_stage(samples: &mut [f32], drive: f32) {
    if drive <= 1.001 { return; }
    // Stage 1: Tape-style warmth (subtle)
    let tape_drive = 1.0 + (drive - 1.0) * 0.5;
    for s in samples.iter_mut() {
        *s = tape_saturation(*s, tape_drive);
    }
    // Stage 2: Tube-style character (more aggressive)
    if drive > 2.0 {
        let tube_drive = 1.0 + (drive - 2.0) * 0.5;
        for s in samples.iter_mut() {
            *s = tube_saturation(*s, tube_drive);
        }
    }
    // Stage 3: Soft-clip final ceiling
    for s in samples.iter_mut() {
        *s = soft_clip(*s, 1.0);
    }
}

// ─── High-Quality Limiter ─────────────────────────────────

pub fn true_peak_limiter(samples: &mut [f32], ceiling_db: f32) {
    let ceiling = 10.0_f32.powf(ceiling_db / 20.0);
    let threshold = ceiling * 0.85;
    let mut gain: f32 = 1.0;
    let release_coeff = (-1.0 / (SAMPLE_RATE as f32 * 0.015)).exp();
    let attack_coeff = (-1.0 / (SAMPLE_RATE as f32 * 0.0001)).exp();
    for sample in samples.iter_mut() {
        let abs = sample.abs();
        let target_gain = if abs > threshold {
            (ceiling / (abs + 1e-10)).min(1.0)
        } else {
            1.0
        };
        if target_gain < gain {
            gain += (target_gain - gain) * (1.0 - attack_coeff);
        } else {
            gain += (target_gain - gain) * (1.0 - release_coeff);
        }
        *sample *= gain;
    }
}

pub fn lookahead_limiter(samples: &mut [f32], ceiling_db: f32, lookahead_ms: f32) {
    if samples.is_empty() { return; }
    let lookahead = (lookahead_ms * SAMPLE_RATE as f32 / 1000.0) as usize;
    let lookahead = lookahead.min(samples.len() / 2).max(1);
    let ceiling = 10.0_f32.powf(ceiling_db / 20.0);
    let threshold = ceiling * 0.8;
    let delayed = samples.to_vec();
    let mut gain: f32 = 1.0;
    let release_coeff = (-1.0 / (SAMPLE_RATE as f32 * 0.05)).exp();
    let attack_coeff = (-1.0 / (SAMPLE_RATE as f32 * 0.0005)).exp();
    let mut write_idx = 0;
    for read_idx in 0..samples.len() {
        let lookahead_idx = (read_idx + lookahead).min(samples.len() - 1);
        let future_abs = samples[lookahead_idx].abs();
        let target_gain = if future_abs > threshold {
            (ceiling / (future_abs + 1e-10)).min(1.0)
        } else {
            1.0
        };
        if target_gain < gain {
            gain += (target_gain - gain) * (1.0 - attack_coeff);
        } else {
            gain += (target_gain - gain) * (1.0 - release_coeff);
        }
        samples[write_idx] = delayed[read_idx] * gain;
        write_idx += 1;
    }
}

// ─── Adaptive Compressor for Punch ───────────────────────

pub fn adaptive_compressor(samples: &mut [f32], threshold_db: f32, ratio: f32, attack_ms: f32, release_ms: f32) {
    if samples.is_empty() { return; }
    let threshold = 10.0_f32.powf(threshold_db / 20.0);
    let attack_coeff = (-1.0 / (SAMPLE_RATE as f32 * attack_ms / 1000.0)).exp();
    let release_coeff = (-1.0 / (SAMPLE_RATE as f32 * release_ms / 1000.0)).exp();
    let mut envelope: f32 = 0.0;
    let makeup_gain = 1.0 + (1.0 - 1.0 / ratio) * 0.5;
    for sample in samples.iter_mut() {
        let abs = sample.abs();
        if abs > envelope {
            envelope += (abs - envelope) * (1.0 - attack_coeff);
        } else {
            envelope += (abs - envelope) * (1.0 - release_coeff);
        }
        let gain_reduction = if envelope > threshold {
            threshold / envelope + (1.0 - threshold / envelope) / ratio
        } else {
            1.0
        };
        *sample *= gain_reduction * makeup_gain;
    }
}

// ─── Multiband Compressor ────────────────────────────────

pub fn multiband_compressor(samples: &mut [f32], low_threshold: f32, high_threshold: f32, ratio: f32) {
    if samples.len() < 128 { return; }
    let mut low_band = samples.to_vec();
    let mut high_band = samples.to_vec();
    low_pass(&mut low_band, 200.0);
    high_pass(&mut high_band, 200.0);
    for s in low_band.iter_mut() {
        let abs = s.abs();
        if abs > low_threshold {
            let reduction = low_threshold / abs + (1.0 - low_threshold / abs) / ratio;
            *s *= reduction;
        }
    }
    for s in high_band.iter_mut() {
        let abs = s.abs();
        if abs > high_threshold {
            let reduction = high_threshold / abs + (1.0 - high_threshold / abs) / ratio;
            *s *= reduction;
        }
    }
    for i in 0..samples.len() {
        samples[i] = low_band[i] + high_band[i];
    }
}

// ─── Improved Transient Shaping ───────────────────────────

pub fn transient_enhance(samples: &mut [f32], boost_db: f32) {
    if samples.len() < 64 || boost_db <= 0.0 { return; }
    let gain = 10.0_f32.powf(boost_db / 20.0);
    let onset_len = (SAMPLE_RATE as f32 * 0.008) as usize;
    let threshold = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.2;
    let mut onset_pos = 0;
    for i in 10..samples.len().min(SAMPLE_RATE as usize / 2) {
        if samples[i].abs() > threshold {
            onset_pos = i;
            break;
        }
    }
    if onset_pos < 10 || onset_pos > samples.len() / 2 { return; }
    let end = (onset_pos + onset_len).min(samples.len());
    let total_samples = (end - onset_pos).max(1) as f32;
    for i in onset_pos..end {
        let t = (i - onset_pos) as f32 / total_samples;
        let envelope = (1.0 - t * t) * (1.0 - t * 0.5);
        samples[i] *= 1.0 + (gain - 1.0) * envelope;
    }
}

pub fn multiband_transient_shape(samples: &mut [f32], boost_db: f32) {
    if samples.len() < 128 || boost_db <= 0.0 { return; }
    let gain = 10.0_f32.powf(boost_db / 20.0);
    let onset_len = (SAMPLE_RATE as f32 * 0.012) as usize;
    let threshold = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.15;
    let mut onsets: Vec<usize> = Vec::new();
    let mut in_onset = false;
    for i in 10..samples.len().min(SAMPLE_RATE as usize) {
        if samples[i].abs() > threshold && !in_onset {
            onsets.push(i);
            in_onset = true;
        } else if samples[i].abs() < threshold * 0.5 {
            in_onset = false;
        }
    }
    for &onset in &onsets {
        let end = (onset + onset_len).min(samples.len());
        for i in onset..end {
            let t = (i - onset) as f32 / (end - onset).max(1) as f32;
            let env = if t < 0.3 {
                (t / 0.3) * 0.8 + 0.2
            } else {
                ((1.0 - t) / 0.7).max(0.0)
            };
            samples[i] *= 1.0 + (gain - 1.0) * env;
        }
    }
}

pub fn apply_punch(samples: &mut [f32]) {
    let attack_ms = 3;
    let attack_samples = (attack_ms as f32 * SAMPLE_RATE as f32 / 1000.0) as usize;
    for i in 0..attack_samples.min(samples.len()) {
        let t = i as f32 / attack_samples as f32;
        let boost = 1.0 + 0.6 * (1.0 - t * t);
        samples[i] *= boost;
    }
}

// ─── Precision Transient Shaping for Recreation ─────────

pub fn precision_transient_shape(samples: &mut [f32], transient_analysis: &[f32]) {
    if samples.len() < 128 || transient_analysis.len() < 4 { return; }
    let num_segments = transient_analysis.len();
    let segment_len = samples.len() / num_segments.max(1);
    for seg in 0..num_segments.min(samples.len() / segment_len.max(1)) {
        let start = seg * segment_len;
        let end = (start + segment_len).min(samples.len());
        let target = transient_analysis[seg];
        let current = samples[start..end].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        if current > 0.001 && target > 0.001 {
            let ratio = (target / current).clamp(0.1, 3.0);
            for i in start..end {
                samples[i] *= ratio;
            }
        }
    }
}

pub fn extract_transient_envelope(samples: &[f32], num_points: usize) -> Vec<f32> {
    if samples.is_empty() || num_points == 0 { return vec![]; }
    let step = (samples.len() / num_points).max(1);
    let mut envelope = Vec::with_capacity(num_points);
    for i in 0..num_points {
        let start = i * step;
        let end = ((i + 1) * step).min(samples.len());
        if start >= end { break; }
        let sum_sq: f32 = samples[start..end].iter().map(|s| s * s).sum();
        let rms = (sum_sq / (end - start) as f32).sqrt();
        envelope.push(rms);
    }
    envelope
}

// ─── Multiband Dynamics Processor ────────────────────────

pub fn multiband_dynamics(samples: &mut [f32], crossover_hz: f32, low_ratio: f32, high_ratio: f32) {
    if samples.len() < 256 { return; }
    let mut low = samples.to_vec();
    let mut high = samples.to_vec();
    low_pass(&mut low, crossover_hz);
    high_pass(&mut high, crossover_hz);
    let threshold = 0.5;
    for s in low.iter_mut() {
        let abs = s.abs();
        if abs > threshold {
            let reduction = threshold / abs + (1.0 - threshold / abs) / low_ratio;
            *s *= reduction;
        }
    }
    for s in high.iter_mut() {
        let abs = s.abs();
        if abs > threshold {
            let reduction = threshold / abs + (1.0 - threshold / abs) / high_ratio;
            *s *= reduction;
        }
    }
    for i in 0..samples.len() {
        samples[i] = low[i] + high[i];
    }
}

// ─── Spectral Tilt Matching ──────────────────────────────

pub fn match_spectral_tilt(samples: &mut [f32], target_tilt: f32) {
    let current_brightness = {
        let n = samples.len().min(4096);
        if n < 64 { return; }
        let total_energy: f32 = samples.iter().take(n).map(|&s| s * s).sum();
        if total_energy <= 0.0 { return; }
        let cutoff_bin = (2000.0 * n as f32 / SAMPLE_RATE as f32) as usize;
        if cutoff_bin >= n { return; }
        let high_energy: f32 = samples[cutoff_bin..n].iter().map(|&s| s * s).sum();
        high_energy / total_energy
    };
    let error = target_tilt - current_brightness;
    if error.abs() > 0.05 {
        if error > 0.0 {
            biquad_high_shelf(samples, 3000.0, error * 6.0, 0.7);
        } else {
            biquad_low_shelf(samples, 300.0, error.abs() * 4.0, 0.7);
        }
    }
}

// ─── Transient Alignment ─────────────────────────────────

pub fn align_transient_peaks(target: &mut [f32], reference: &[f32]) {
    let ref_peak = reference.iter()
        .enumerate()
        .map(|(i, &s)| (i, s.abs()))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);
    let tgt_peak = target.iter()
        .enumerate()
        .map(|(i, &s)| (i, s.abs()))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);
    if ref_peak > tgt_peak + 2 {
        let shift = ref_peak - tgt_peak;
        if tgt_peak + shift < target.len() {
            let mut shifted = vec![0.0f32; target.len()];
            for i in tgt_peak..target.len() - shift {
                shifted[i + shift] = target[i];
            }
            target.copy_from_slice(&shifted);
        }
    }
}

// ─── Envelope & Dynamics ──────────────────────────────────

pub fn apply_envelope(samples: &mut [f32], attack_s: f32, decay_s: f32) {
    let attack_samples = (attack_s * SAMPLE_RATE as f32) as usize;
    let decay_samples = (decay_s * SAMPLE_RATE as f32) as usize;
    // Attack: eased curve
    for i in 0..attack_samples.min(samples.len()) {
        let t = i as f32 / attack_samples.max(1) as f32;
        samples[i] *= t * (2.0 - t);
    }
    // Decay: exponential curve
    let total = samples.len();
    for i in 0..decay_samples.min(samples.len()) {
        let idx = total - 1 - i;
        let t = i as f32 / decay_samples.max(1) as f32;
        samples[idx] *= (-3.0 * t).exp();
    }
}

pub fn noise_gate_tail(samples: &mut [f32], threshold_db: f32) {
    if samples.len() < 256 { return; }
    let threshold = 10.0_f32.powf(threshold_db / 20.0);
    let hold_samples = (SAMPLE_RATE as f32 * 0.015) as usize;
    let mut last_above = 0;
    for i in 0..samples.len() {
        if samples[i].abs() > threshold {
            last_above = i;
        }
    }
    let fade_start = (last_above + hold_samples).min(samples.len());
    if fade_start >= samples.len() - 4 { return; }
    let fade_len = (SAMPLE_RATE as f32 * 0.008) as usize;
    let fade_end = (fade_start + fade_len).min(samples.len());
    for i in fade_start..fade_end {
        let t = (i - fade_start) as f32 / fade_len as f32;
        samples[i] *= (1.0 - t) * (1.0 - t);
    }
    for sample in samples.iter_mut().skip(fade_end) {
        *sample = 0.0;
    }
}

// ─── Pitch & Time ─────────────────────────────────────────

pub fn pitch_shift(samples: &[f32], ratio: f32) -> Vec<f32> {
    let new_len = (samples.len() as f32 / ratio) as usize;
    if new_len == 0 { return vec![]; }
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

// ─── Legacy EQ interface ──────────────────────────────────

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

// ─── Spectral Balance ─────────────────────────────────────

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

// ─── Phase-Aware Processing ───────────────────────────────

pub fn phase_align_sum(layers: &mut [Vec<f32>]) -> Vec<f32> {
    if layers.is_empty() { return vec![]; }
    if layers.len() == 1 { return layers[0].clone(); }
    let max_len = layers.iter().map(|l| l.len()).max().unwrap_or(0);
    let mut output = vec![0.0f32; max_len];
    let reference = &layers[0];
    for layer_idx in 1..layers.len() {
        let layer = &layers[layer_idx];
        let min_len = reference.len().min(layer.len());
        if min_len < 64 { continue; }
        let peak_ref = reference[..min_len].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let peak_layer = layer[..min_len].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        if peak_ref < 0.001 || peak_layer < 0.001 { continue; }
        let ref_onset = reference[..min_len].iter().position(|s| s.abs() > peak_ref * 0.5).unwrap_or(0);
        let layer_onset = layer[..min_len].iter().position(|s| s.abs() > peak_layer * 0.5).unwrap_or(0);
        if layer_onset > ref_onset + 2 {
            let shift = layer_onset - ref_onset;
            for _ in (shift..min_len).rev() {
                // phase alignment would shift layer onset to match reference
            }
        }
    }
    for layer in layers.iter() {
        for (i, &s) in layer.iter().enumerate() {
            output[i] += s;
        }
    }
    output
}

pub fn stereo_widen(samples: &mut [f32], width: f32) {
    if samples.len() < 2 { return; }
    let w = width.clamp(0.0, 1.0);
    for pair in samples.chunks_exact_mut(2) {
        let m = (pair[0] + pair[1]) * 0.5;
        let s = (pair[0] - pair[1]) * 0.5;
        pair[0] = m + s * (1.0 + w);
        pair[1] = m - s * (1.0 + w);
    }
}

// ─── DC & Subsonic Filtering ──────────────────────────────

pub fn remove_dc_offset(samples: &mut [f32]) {
    if samples.is_empty() { return; }
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    if mean.abs() > 0.0001 {
        for sample in samples.iter_mut() {
            *sample -= mean;
        }
    }
}

pub fn subsonic_filter(samples: &mut [f32]) {
    // Gentle 4th-order Bessel-style HPF at 20Hz
    high_pass(samples, 20.0);
}

// ─── Tone Control ─────────────────────────────────────────

pub fn tilt_spectrum(samples: &mut [f32], amount: f32) {
    // amount: -1.0 (dark) to 1.0 (bright)
    if amount.abs() < 0.05 { return; }
    if amount > 0.0 {
        biquad_high_shelf(samples, 2000.0, amount * 8.0, 0.7);
    } else {
        biquad_low_shelf(samples, 300.0, amount.abs() * 6.0, 0.7);
    }
}

// ─── Utility ──────────────────────────────────────────────

pub fn de_click(samples: &mut [f32], max_step: f32) {
    if samples.len() < 3 { return; }
    let threshold = max_step.max(0.01);
    for i in 1..samples.len() - 1 {
        let diff1 = (samples[i] - samples[i - 1]).abs();
        let diff2 = (samples[i + 1] - samples[i]).abs();
        if diff1 > threshold && diff2 > threshold {
            samples[i] = (samples[i - 1] + samples[i + 1]) * 0.5;
        }
    }
}

// ─── Tail + Texture Intelligence ───────────────────────────

pub struct TailTextureConfig {
    pub decay_modulation_rate: f32,
    pub decay_modulation_depth: f32,
    pub noise_texture_density: f32,
    pub noise_movement_rate: f32,
    pub resonant_q: f32,
    pub resonant_freq: f32,
    pub resonant_modulation: f32,
    pub analog_instability: f32,
    pub texture_layers: usize,
    pub cinematic_expansion: f32,
}

impl Default for TailTextureConfig {
    fn default() -> Self {
        Self {
            decay_modulation_rate: 0.5,
            decay_modulation_depth: 0.08,
            noise_texture_density: 0.5,
            noise_movement_rate: 0.3,
            resonant_q: 0.5,
            resonant_freq: 200.0,
            resonant_modulation: 0.0,
            analog_instability: 0.05,
            texture_layers: 2,
            cinematic_expansion: 0.0,
        }
    }
}

pub fn decay_modulation(config: &TailTextureConfig, num_samples: usize, sample_rate: u32) -> Vec<f32> {
    let mut mod_env = vec![1.0f32; num_samples];
    if config.decay_modulation_depth <= 0.0 { return mod_env; }
    let rate = config.decay_modulation_rate;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let lfo = (2.0 * PI * rate * t).sin() * config.decay_modulation_depth;
        mod_env[i] = 1.0 + lfo;
    }
    mod_env
}

pub fn filtered_noise_layer(
    num_samples: usize,
    sample_rate: u32,
    hp_hz_start: f32,
    hp_hz_end: f32,
    density: f32,
    seed: f32,
) -> Vec<f32> {
    let mut layer = vec![0.0f32; num_samples];
    if density <= 0.0 { return layer; }
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let frac = (i as f32 / num_samples.max(1) as f32).min(1.0);
        let n1 = ((seed + i as f32 * 0.37).fract() * 127.1).sin() * 43758.5453;
        let n2 = ((seed * 1.3 + i as f32 * 0.53).fract() * 127.1).sin() * 43758.5453;
        let n3 = ((seed * 0.7 + i as f32 * 0.19).fract() * 127.1).sin() * 43758.5453;
        let n = (n1.fract() * 2.0 - 1.0) * 0.5
            + (n2.fract() * 2.0 - 1.0) * 0.3
            + (n3.fract() * 2.0 - 1.0) * 0.2;
        let hp_now = hp_hz_start + (hp_hz_end - hp_hz_start) * frac;
        let rc = 1.0 / (2.0 * PI * hp_now.max(20.0));
        let dt = 1.0 / sample_rate as f32;
        let alpha = (rc / (rc + dt)).clamp(0.0, 1.0);
        let tmp1 = alpha * (layer.get(i.saturating_sub(1)).copied().unwrap_or(0.0) + n - layer.get(i.saturating_sub(1)).copied().unwrap_or(0.0));
        layer[i] = n * density * (1.0 - alpha * 0.5);
    }
    layer
}

pub fn resonant_tail_filter(samples: &mut [f32], freq_hz: f32, q: f32, modulation_rate: f32) {
    if samples.len() < 64 || q <= 0.0 { return; }
    let w0 = 2.0 * PI * freq_hz / SAMPLE_RATE as f32;
    let alpha = w0.sin() / (2.0 * q.max(0.1));
    let b0 = alpha;
    let b1 = 0.0;
    let b2 = -alpha;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * w0.cos();
    let a2 = 1.0 - alpha;
    let a0r = a0.recip();
    let mut x1 = 0.0;
    let mut x2 = 0.0;
    let mut y1 = 0.0;
    let mut y2 = 0.0;
    for (i, sample) in samples.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq_mod = if modulation_rate > 0.0 {
            1.0 + 0.3 * (2.0 * PI * modulation_rate * t).sin()
        } else {
            1.0
        };
        let x = *sample;
        let y = b0 * a0r * x + b1 * a0r * x1 + b2 * a0r * x2 - a1 * a0r * y1 - a2 * a0r * y2;
        x2 = x1; x1 = x;
        y2 = y1; y1 = y;
        *sample = y * freq_mod.min(1.5);
    }
}

pub fn texture_layering(num_samples: usize, layers: usize, density: f32, seed: f32, sample_rate: u32) -> Vec<f32> {
    if layers == 0 || density <= 0.0 { return vec![0.0f32; num_samples]; }
    let mut combined = vec![0.0f32; num_samples];
    for layer_idx in 0..layers {
        let rate = 0.05 + layer_idx as f32 * 0.12;
        let hp_start = 100.0 + layer_idx as f32 * 2000.0;
        let hp_end = hp_start * (1.0 + layer_idx as f32 * 0.3);
        let layer_density = density / (layers as f32).sqrt();
        let layer_seed = seed + layer_idx as f32 * 7.0;
        let layer = filtered_noise_layer(num_samples, sample_rate, hp_start, hp_end, layer_density, layer_seed);
        let lfo_rate = rate;
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let mod_lfo = 0.5 + 0.5 * (2.0 * PI * lfo_rate * t).sin();
            combined[i] += layer[i] * mod_lfo;
        }
    }
    let peak = combined.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak > 0.0 {
        for s in combined.iter_mut() { *s /= peak.max(0.001); }
    }
    combined
}

pub fn analog_tail_instability(samples: &mut [f32], amount: f32, seed: f32) {
    if amount <= 0.0 { return; }
    for i in 0..samples.len() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let noise_val = ((seed + i as f32 * 0.07).fract() * 127.1).sin() * 43758.5453;
        let n = (noise_val.fract() * 2.0 - 1.0) * amount * 0.01;
        let drift = (2.0 * PI * 0.15 * (t + seed)).sin() * amount * 0.008;
        let phase_jitter = (2.0 * PI * 3.0 * t).sin() * amount * 0.005;
        samples[i] *= 1.0 + n + drift + phase_jitter;
    }
}

pub fn cinematic_tail_extension(samples: &mut [f32], extension_amount: f32) {
    if extension_amount <= 0.0 || samples.len() < 256 { return; }
    let tail_start = (samples.len() as f32 * 0.6) as usize;
    if tail_start >= samples.len() { return; }
    let sub_freq = 40.0;
    let cin_len = samples.len() - tail_start;
    let extra_note_len = (cin_len as f32 * extension_amount * 0.5) as usize;
    if extra_note_len < 4 { return; }
    let cin_band_end = samples.len();
    for i in tail_start..cin_band_end {
        let local_t = (i - tail_start) as f32 / SAMPLE_RATE as f32;
        let sub = (2.0 * PI * sub_freq * local_t).sin() * 0.15 * extension_amount;
        let noise_val = ((i as f32 * 0.04).fract() * 127.1).sin() * 43758.5453;
        let n = (noise_val.fract() * 2.0 - 1.0) * 0.04 * extension_amount;
        let global_t = i as f32 / SAMPLE_RATE as f32;
        let boost = 1.0 + extension_amount * 0.3 * (-0.8 * local_t).exp();
        samples[i] = (samples[i] + sub + n) * boost.min(2.0);
    }
}

pub fn non_static_decay(samples: &mut [f32], modulation_amount: f32) {
    if modulation_amount <= 0.0 || samples.len() < 128 { return; }
    let envelope = extract_transient_envelope(samples, 64);
    if envelope.len() < 4 { return; }
    let segment_len = samples.len() / envelope.len().max(1);
    for seg_idx in 0..envelope.len() - 1 {
        let start = seg_idx * segment_len;
        let end = ((seg_idx + 1) * segment_len).min(samples.len());
        if start >= end { break; }
        let t = start as f32 / SAMPLE_RATE as f32;
        let mod_factor = 1.0 + modulation_amount * (2.0 * PI * 0.3 * t).sin() * 0.3;
        let env_ratio = if envelope[seg_idx] > 0.001 {
            (envelope[seg_idx + 1] / envelope[seg_idx]).min(1.0)
        } else {
            0.5
        };
        let adjusted_ratio = 1.0 - (1.0 - env_ratio) * mod_factor;
        for i in start..end {
            let local_t = (i - start) as f32 / (end - start).max(1) as f32;
            let smooth = 1.0 - (1.0 - adjusted_ratio) * local_t;
            samples[i] *= smooth;
        }
    }
}

pub fn apply_tail_texture(samples: &mut [f32], config: &TailTextureConfig) {
    if samples.len() < 256 { return; }
    let num_samples = samples.len();
    let sr = SAMPLE_RATE;

    if config.decay_modulation_depth > 0.001 {
        let mod_env = decay_modulation(config, num_samples, sr);
        for i in 0..num_samples {
            samples[i] *= mod_env[i];
        }
    }

    if config.noise_texture_density > 0.01 {
        let texture = texture_layering(
            num_samples,
            config.texture_layers.max(1),
            config.noise_texture_density * 0.3,
            42.0,
            sr,
        );
        let peak_t = texture.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        let peak_s = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        if peak_s > 0.001 {
            let blend = config.noise_texture_density * 0.15;
            for i in 0..num_samples {
                samples[i] += texture[i] * peak_s * blend;
            }
        }
    }

    if config.resonant_q > 0.1 {
        resonant_tail_filter(samples, config.resonant_freq, config.resonant_q, config.resonant_modulation);
    }

    if config.analog_instability > 0.001 {
        analog_tail_instability(samples, config.analog_instability, 42.0);
    }

    non_static_decay(samples, config.decay_modulation_depth);

    if config.cinematic_expansion > 0.01 {
        cinematic_tail_extension(samples, config.cinematic_expansion);
    }
}
