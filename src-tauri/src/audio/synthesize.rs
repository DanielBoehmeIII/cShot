use std::f32::consts::PI;
use super::{SoundType, noise, SAMPLE_RATE, DspParams};
use super::dsp;

pub fn generate_base(sound_type: SoundType, duration_ms: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_ms / 1000.0) as usize;
    let mut samples = vec![0.0f32; num_samples];
    match sound_type {
        SoundType::Kick => synthesize_kick_layered(&mut samples, duration_ms),
        SoundType::Snare => synthesize_snare_advanced(&mut samples),
        SoundType::ClosedHat => synthesize_metallic_hat(&mut samples, true),
        SoundType::OpenHat => synthesize_metallic_hat(&mut samples, false),
        SoundType::Clap => synthesize_clap_advanced(&mut samples),
        SoundType::Tom => synthesize_tom(&mut samples),
        SoundType::Perc => synthesize_fm_percussion(&mut samples, duration_ms),
        SoundType::Bass => synthesize_bass_hit(&mut samples, duration_ms),
        SoundType::Fx => synthesize_cinematic_boom(&mut samples, duration_ms),
        SoundType::Other => synthesize_other(&mut samples),
    }
    samples
}

pub fn generate_with_dsp(sound_type: SoundType, duration_ms: f32, dsp_params: &DspParams) -> Vec<f32> {
    let mut samples = generate_base(sound_type, duration_ms);

    if dsp_params.punch {
        let attack_len = (SAMPLE_RATE as f32 * 0.003) as usize;
        for i in 0..attack_len.min(samples.len()) {
            let t = i as f32 / attack_len as f32;
            let boost = 1.0 + 0.6 * (1.0 - t * t);
            samples[i] *= boost;
        }
    }

    let safe_gain = dsp_params.gain.clamp(0.3, 3.0);
    if (safe_gain - 1.0).abs() > 0.01 {
        for s in samples.iter_mut() { *s *= safe_gain; }
    }

    if dsp_params.noise_amt > 0.0 {
        for i in 0..samples.len() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let n = noise((i as f32 * 0.3).fract());
            let env = (-20.0 * t).exp();
            samples[i] += n * dsp_params.noise_amt * env * 0.3;
        }
    }

    samples
}

fn adsr_envelope(i: usize, num_samples: usize, attack_pct: f32, decay_pct: f32, sustain_level: f32, release_pct: f32) -> f32 {
    let total = num_samples.max(1) as f32;
    let attack_s = (attack_pct * total) as usize;
    let decay_s = attack_s + (decay_pct * total) as usize;
    let release_s = num_samples.saturating_sub((release_pct * total) as usize);

    if i < attack_s {
        let p = i as f32 / attack_s.max(1) as f32;
        p * (2.0 - p)
    } else if i < decay_s {
        let t = (i - attack_s) as f32 / (decay_s - attack_s).max(1) as f32;
        let decay = 1.0 - (1.0 - sustain_level) * (t * t);
        decay.max(0.0)
    } else if i < release_s {
        sustain_level
    } else {
        let t = (i - release_s) as f32 / (num_samples - release_s).max(1) as f32;
        sustain_level * (1.0 - t) * (1.0 - t)
    }
}

fn synthesize_kick_layered(samples: &mut [f32], _duration_ms: f32) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = i as f32 / len.max(1) as f32;

        let click_env = (-150.0 * t).exp();
        let click_freq = 5000.0 - 3000.0 * frac.min(1.0);
        let n_click = noise((i as f32 * 0.9).fract());
        let click = (2.0 * PI * click_freq * t).sin() * click_env * 0.25
            + n_click * click_env * 0.12;

        let impact_freq = 2000.0 - 1200.0 * (t / 0.01).min(1.0);
        let impact_env = (-120.0 * t).exp();
        let impact = (2.0 * PI * impact_freq * t).sin() * impact_env * 0.35;

        let body_pitch_drop = 150.0 - 105.0 * (t / 0.25).min(1.0);
        let body_env = (-7.0 * t).exp();
        let body = (2.0 * PI * body_pitch_drop * t).sin() * body_env * 0.7;
        let body_saturated = dsp::tape_saturation(body * 1.4, 1.5);

        let sub_freq = 60.0 - 15.0 * (t / 0.4).min(1.0);
        let sub_env = (-3.5 * t).exp();
        let sub = (2.0 * PI * sub_freq * t).sin() * sub_env * 0.55;

        let dist_env = (-10.0 * t).exp();
        let dist = dsp::tape_saturation(body_saturated * 2.0, 2.0) * dist_env * 0.12;

        samples[i] = click + impact + body_saturated * 0.8 + sub + dist;
    }
}

fn synthesize_snare_advanced(samples: &mut [f32]) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = i as f32 / len.max(1) as f32;

        let tone_freq = 220.0 - 40.0 * frac.min(1.0);
        let tone_env = (-15.0 * t).exp();
        let tone = (2.0 * PI * tone_freq * t).sin() * tone_env * 0.3;

        let n_low = noise((i as f32 * 0.12).fract());
        let n_high = noise((i as f32 * 0.5).fract());
        let n_mid = noise((i as f32 * 0.25).fract());
        let noise_env = (-20.0 * t).exp();
        let noise_body = (n_low * 0.4 + n_mid * 0.35 + n_high * 0.25) * noise_env * 0.5;

        let rattle_freq = 6000.0 - 2000.0 * frac.min(1.0);
        let rattle_phase = (2.0 * PI * rattle_freq * t).sin();
        let rattle_noise = noise((i as f32 * 0.7).fract());
        let rattle_env = (-45.0 * t).exp();
        let rattle = rattle_phase * rattle_noise * rattle_env * 0.12;

        let crack_env = (-250.0 * t).exp();
        let crack = noise((i as f32 * 0.8).fract()) * crack_env * 0.4;

        samples[i] = tone + noise_body + rattle + crack;
    }
}

fn synthesize_metallic_hat(samples: &mut [f32], is_closed: bool) {
    let len = samples.len();
    let decay_rate = if is_closed { 35.0 } else { 7.0 };
    let hp_freq = if is_closed { 7000.0 } else { 4000.0 };

    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;

        let car_freq = if is_closed { 7000.0 } else { 5000.0 };
        let mod_freq = 1500.0;
        let mod_index = 3.0 * (-15.0 * t).exp();
        let modulation = mod_index * (2.0 * PI * mod_freq * t).sin();
        let fm_metal = (2.0 * PI * car_freq * t + modulation).sin() * 0.35;

        let n1 = noise((i as f32 * 0.6).fract());
        let n2 = noise((i as f32 * 0.9).fract());
        let noise_sig = n1 * 0.5 + n2 * 0.5;

        let env = (-decay_rate * t).exp();
        let click_env = (-250.0 * t).exp();

        samples[i] = (fm_metal * 0.5 + noise_sig * 0.5) * env + noise_sig * click_env * 0.2;
    }

    let rc = 1.0 / (2.0 * PI * hp_freq);
    let dt = 1.0 / SAMPLE_RATE as f32;
    let alpha = (rc / (rc + dt)).clamp(0.0, 1.0);
    let mut prev1 = 0.0;
    let mut prev2 = 0.0;
    for sample in samples.iter_mut() {
        let input = *sample;
        let tmp = alpha * (prev1 + *sample - prev1);
        *sample = alpha * (prev2 + tmp - prev2);
        prev1 = input;
        prev2 = tmp;
    }
}

fn synthesize_clap_advanced(samples: &mut [f32]) {
    let len = samples.len();
    let hit_times: [f32; 8] = [0.0, 0.006, 0.012, 0.018, 0.028, 0.040, 0.055, 0.072];

    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let n1 = noise((i as f32 * 0.25).fract());
        let n2 = noise((i as f32 * 0.15).fract());
        let n3 = noise((i as f32 * 0.4).fract());

        let mut val = 0.0;
        for &hit_t in &hit_times {
            let dt = (t - hit_t).max(0.0);
            let hit_env = (-55.0 * dt).exp();
            val += (n1 * 0.5 + n2 * 0.3 + n3 * 0.2) * hit_env;
        }

        let body_env = (-12.0 * t).exp();
        let body = (2.0 * PI * 200.0 * t).sin() * body_env * 0.12;

        let sub_env = (-6.0 * t).exp();
        let sub = (2.0 * PI * 80.0 * t).sin() * sub_env * 0.08;

        let saturated = dsp::tape_saturation(val * 0.3 + body + sub, 1.3);
        samples[i] = saturated.clamp(-1.0, 1.0);
    }
}

fn synthesize_fm_percussion(samples: &mut [f32], duration_ms: f32) {
    let len = samples.len();
    let dur = duration_ms / 1000.0;
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (t / dur).min(1.0);

        let car_freq = 400.0 - 200.0 * frac;
        let mod_freq = car_freq * 2.0;
        let mod_index = 2.0 * (1.0 - frac);
        let modulator = mod_index * (2.0 * PI * mod_freq * t).sin();
        let carrier = (2.0 * PI * car_freq * t + modulator).sin();

        let n = noise((i as f32 * 0.5).fract());
        let noise_env = (-35.0 * t).exp();

        let env = adsr_envelope(i, len, 0.005, 0.05, 0.0, 0.5);
        samples[i] = (carrier * 0.5 + n * noise_env * 0.2) * env;
    }
}

fn synthesize_tonal_perc(samples: &mut [f32], _duration_ms: f32, pitch_hz: f32) {
    let len = samples.len();
    let harmonics: [(f32, f32); 5] = [
        (1.0, 1.0), (2.0, 0.5), (3.0, 0.25), (4.0, 0.12), (5.0, 0.06),
    ];
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = adsr_envelope(i, len, 0.005, 0.15, 0.0, 0.5);
        let mut val = 0.0;
        for &(harmonic, amp) in &harmonics {
            let freq = pitch_hz * harmonic;
            val += (2.0 * PI * freq * t).sin() * amp;
        }
        let norm: f32 = harmonics.iter().map(|(_, a)| a).sum();
        samples[i] = (val / norm) * env * 0.7;
    }
}

fn synthesize_cinematic_boom(samples: &mut [f32], duration_ms: f32) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (t / (duration_ms / 1000.0).max(0.001)).min(1.0);

        let sub_freq = 40.0 - 10.0 * frac;
        let sub_env = (-1.5 * t).exp();
        let sub = (2.0 * PI * sub_freq * t).sin() * sub_env * 0.5;

        let sweep_freq = 80.0 + 2000.0 * frac;
        let sweep = (2.0 * PI * sweep_freq * t).sin() * 0.15;

        let n = noise((i as f32 * 0.06).fract());
        let noise_env = (-0.8 * t).exp();
        let n_val = n * noise_env * 0.12;

        let impact_env = (-100.0 * t).exp();
        let impact = noise((i as f32 * 0.7).fract()) * impact_env * 0.3;

        samples[i] = sub + sweep + n_val + impact;
    }
}

fn synthesize_ui_click(samples: &mut [f32]) {
    let click_len = (SAMPLE_RATE as f32 * 0.03) as usize;
    let len = click_len.min(samples.len());
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (-250.0 * t).exp();
        let click = (2.0 * PI * 4000.0 * t).sin() * env * 0.6
            + noise((i as f32 * 0.8).fract()) * env * 0.3;
        samples[i] = click;
    }
}

fn synthesize_bass_hit(samples: &mut [f32], duration_ms: f32) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (t / (duration_ms / 1000.0)).min(1.0);

        let freq1 = 55.0 + 30.0 * (1.0 - frac);
        let freq2 = 55.3 + 30.0 * (1.0 - frac);
        let env = (-2.5 * t).exp();

        let osc1 = (2.0 * PI * freq1 * t).sin();
        let osc2 = (2.0 * PI * freq2 * t).sin();

        let sub = (2.0 * PI * 30.0 * t).sin() * env * 0.35;

        let mix = (osc1 * 0.5 + osc2 * 0.5) * env;
        let saturated = dsp::tape_saturation(mix * 1.5, 1.6);

        let click_env = (-180.0 * t).exp();
        let click = (2.0 * PI * 2500.0 * t).sin() * click_env * 0.15;

        samples[i] = saturated * 0.7 + sub + click;
    }
}

fn synthesize_tom(samples: &mut [f32]) {
    let len = samples.len();
    let harmonics: [(f32, f32); 3] = [(1.0, 1.0), (2.0, 0.35), (3.0, 0.15)];
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (t / 0.3).min(1.0);
        let freq = 120.0 - 40.0 * frac;
        let env = (-5.0 * t).exp();

        let mut val = 0.0;
        for &(harmonic, amp) in &harmonics {
            val += (2.0 * PI * freq * harmonic * t).sin() * amp;
        }
        let norm: f32 = harmonics.iter().map(|(_, a)| a).sum();
        let tone = (val / norm) * env * 0.7;

        let n = noise((i as f32 * 0.25).fract());
        let noise_env = (-14.0 * t).exp();

        let saturated = dsp::tape_saturation(tone + n * noise_env * 0.15, 1.2);
        samples[i] = saturated;
    }
}

fn synthesize_other(samples: &mut [f32]) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let n1 = noise((i as f32 * 0.3).fract());
        let n2 = noise((i as f32 * 0.15).fract());
        let env = (-6.0 * t).exp();
        samples[i] = (n1 * 0.6 + n2 * 0.4) * env;
    }
}

pub fn generate_resonant_impact(duration_ms: f32, freq_hz: f32, resonance: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_ms / 1000.0) as usize;
    let mut samples = vec![0.0f32; num_samples];
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let n = noise((i as f32 * 0.3).fract());
        let resonant_sig = (2.0 * PI * freq_hz * t).sin() * n * resonance;
        let env = adsr_envelope(i, len, 0.01, 0.15, 0.0, 0.8);
        samples[i] = resonant_sig * env * 0.6;
    }
    samples
}

pub fn generate_ui_click() -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * 0.03) as usize;
    let mut samples = vec![0.0f32; num_samples];
    synthesize_ui_click(&mut samples);
    samples
}

pub fn generate_tonal_perc(duration_ms: f32, pitch_hz: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_ms / 1000.0) as usize;
    let mut samples = vec![0.0f32; num_samples];
    synthesize_tonal_perc(&mut samples, duration_ms, pitch_hz);
    samples
}

pub fn generate_cinematic_boom(duration_ms: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_ms / 1000.0) as usize;
    let mut samples = vec![0.0f32; num_samples];
    synthesize_cinematic_boom(&mut samples, duration_ms);
    samples
}

pub fn generate_bass_hit(duration_ms: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_ms / 1000.0) as usize;
    let mut samples = vec![0.0f32; num_samples];
    synthesize_bass_hit(&mut samples, duration_ms);
    samples
}
