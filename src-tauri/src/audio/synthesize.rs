use std::f32::consts::PI;
use super::{SoundType, noise, SAMPLE_RATE, DspParams};

pub fn generate_base(sound_type: SoundType, duration_ms: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_ms / 1000.0) as usize;
    let mut samples = vec![0.0f32; num_samples];
    match sound_type {
        SoundType::Kick => synthesize_kick(&mut samples),
        SoundType::Snare => synthesize_snare(&mut samples),
        SoundType::ClosedHat => synthesize_hat(&mut samples, true),
        SoundType::OpenHat => synthesize_hat(&mut samples, false),
        SoundType::Clap => synthesize_clap(&mut samples),
        SoundType::Tom => synthesize_tom(&mut samples),
        SoundType::Perc => synthesize_perc(&mut samples),
        SoundType::Bass => synthesize_bass(&mut samples),
        SoundType::Fx => synthesize_fx(&mut samples),
        SoundType::Other => synthesize_other(&mut samples),
    }
    samples
}

pub fn generate_with_dsp(sound_type: SoundType, duration_ms: f32, dsp: &DspParams) -> Vec<f32> {
    let mut samples = generate_base(sound_type, duration_ms);

    if dsp.punch {
        let attack_len = (SAMPLE_RATE as f32 * 0.003) as usize;
        for i in 0..attack_len.min(samples.len()) {
            let boost = 1.0 + 0.6 * (1.0 - i as f32 / attack_len as f32);
            samples[i] *= boost;
        }
    }

    let safe_gain = dsp.gain.clamp(0.3, 3.0);
    if (safe_gain - 1.0).abs() > 0.01 {
        for s in samples.iter_mut() { *s *= safe_gain; }
    }

    if dsp.noise_amt > 0.0 {
        for i in 0..samples.len() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let n = noise((i as f32 * 0.3).fract());
            let env = (-20.0 * t).exp();
            samples[i] += n * dsp.noise_amt * env * 0.3;
        }
    }

    samples
}

fn synthesize_kick(samples: &mut [f32]) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = i as f32 / len.max(1) as f32;

        // Three-layer kick
        // 1. Click/transient (1000-5000Hz, very short)
        let click_freq = 3000.0 - 2000.0 * frac.min(1.0);
        let click_env = (-80.0 * t).exp();
        let click = (2.0 * PI * click_freq * t).sin() * click_env * 0.25;

        // 2. Body (pitch drop from 150Hz to 45Hz)
        let body_freq = 150.0 - (150.0 - 45.0) * (t / 0.3).min(1.0);
        let body_env = (-6.0 * t).exp();
        let body = (2.0 * PI * body_freq * t).sin() * body_env * 0.8;

        // 3. Sub (very low, sustained)
        let sub_freq = 60.0 - 20.0 * (t / 0.4).min(1.0);
        let sub_env = (-4.0 * t).exp() * 0.5;
        let sub = (2.0 * PI * sub_freq * t).sin() * sub_env;

        // 4. Distortion layer (adds punch)
        let dist = (body * 1.5).tanh() * 0.15;

        samples[i] = click + body + sub + dist;
    }

    // Apply a transient boost envelope
    let attack_samples = (SAMPLE_RATE as f32 * 0.002) as usize;
    for i in 0..attack_samples.min(len) {
        samples[i] *= 1.0 + 0.3 * (1.0 - i as f32 / attack_samples as f32);
    }
}

fn synthesize_snare(samples: &mut [f32]) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = i as f32 / len.max(1) as f32;

        // Tone (200Hz)
        let tone_freq = 200.0 - 30.0 * frac.min(1.0);
        let tone_env = (-12.0 * t).exp();
        let tone = (2.0 * PI * tone_freq * t).sin() * tone_env * 0.35;

        // Dual noise layers
        let n1 = noise((i as f32 * 0.15).fract());
        let n2 = noise((i as f32 * 0.08).fract());
        let noise_env = (-18.0 * t).exp();
        let noise_val = (n1 * 0.7 + n2 * 0.3) * noise_env * 0.55;

        // Snare wire rattle (high-frequency buzz)
        let rattle_freq = 5000.0 - 2000.0 * frac.min(1.0);
        let rattle = (2.0 * PI * rattle_freq * t).sin();
        let rattle_noise_val = noise((i as f32 * 0.5).fract());
        let rattle_env = (-30.0 * t).exp();
        let wire = rattle * rattle_noise_val * rattle_env * 0.1;

        samples[i] = tone + noise_val + wire;
    }

    // Crack transient
    let crack_len = (SAMPLE_RATE as f32 * 0.003) as usize;
    for i in 0..crack_len.min(len) {
        let frac = i as f32 / crack_len as f32;
        samples[i] *= 1.0 + 0.5 * (1.0 - frac);
    }
}

fn synthesize_hat(samples: &mut [f32], is_closed: bool) {
    let len = samples.len();
    let decay = if is_closed { 30.0 } else { 6.0 };
    let hp_cutoff = if is_closed { 6000.0 } else { 4000.0 };

    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let n = noise((i as f32 * 0.5).fract());
        let n2 = noise((i as f32 * 0.7).fract());
        let env = (-decay * t).exp();
        samples[i] = (n * 0.6 + n2 * 0.4) * env;
    }

    // High-pass filter to remove low-end rumble
    let rc = 1.0 / (2.0 * PI * hp_cutoff);
    let dt = 1.0 / SAMPLE_RATE as f32;
    let alpha = rc / (rc + dt);
    let mut prev = samples[0];
    for sample in samples.iter_mut() {
        let input = *sample;
        *sample = alpha * (prev + *sample - prev);
        prev = input;
    }
}

fn synthesize_clap(samples: &mut [f32]) {
    let len = samples.len();
    // Multi-hit clap: several noise bursts close together
    let hit_times: [f32; 6] = [0.0, 0.008, 0.016, 0.024, 0.035, 0.050];
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let n1 = noise((i as f32 * 0.2).fract());
        let n2 = noise((i as f32 * 0.12).fract());

        let mut val = 0.0;
        for &hit_t in &hit_times {
            let dt = (t - hit_t).max(0.0);
            let hit_env = (-35.0 * dt).exp();
            val += (n1 * 0.6 + n2 * 0.4) * hit_env;
        }

        // Body
        let body_env = (-10.0 * t).exp();
        let body = (2.0 * PI * 180.0 * t).sin() * body_env * 0.15;

        samples[i] = (val * 0.35 + body).clamp(-1.0, 1.0);
    }
}

fn synthesize_tom(samples: &mut [f32]) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (t / 0.3).min(1.0);
        let freq = 120.0 - 40.0 * frac;
        let env = (-4.0 * t).exp();
        let tone = (2.0 * PI * freq * t).sin() * env * 0.7;

        let n = noise((i as f32 * 0.25).fract());
        let noise_env = (-12.0 * t).exp();
        samples[i] = tone + n * noise_env * 0.2;
    }
}

fn synthesize_perc(samples: &mut [f32]) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let n1 = noise((i as f32 * 0.5).fract());
        let n2 = noise((i as f32 * 0.35).fract());

        let freq = 400.0 + 400.0 * (1.0 - (t / 0.2).min(1.0));
        let tone = (2.0 * PI * freq * t).sin() * 0.4;
        let noise_val = n1 * 0.5 + n2 * 0.5;
        let env = (-18.0 * t).exp();

        samples[i] = (tone + noise_val * 0.3) * env;
    }
}

fn synthesize_bass(samples: &mut [f32]) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (t / 0.6).min(1.0);

        // Two oscillators detuned for thickness
        let freq1 = 55.0 + 25.0 * (1.0 - frac);
        let freq2 = 55.5 + 25.0 * (1.0 - frac);
        let env = (-2.0 * t).exp();

        let osc1 = (2.0 * PI * freq1 * t).sin();
        let osc2 = (2.0 * PI * freq2 * t).sin();

        // Saturation for warmth
        let mix = (osc1 * 0.5 + osc2 * 0.5) * env;
        let saturated = (mix * 1.3).tanh();

        samples[i] = saturated;
    }
}

fn synthesize_fx(samples: &mut [f32]) {
    let len = samples.len();
    for i in 0..len {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (t / 0.8).min(1.0);

        // Rising tone sweep
        let sweep_freq = 100.0 + 3000.0 * frac;
        let sweep = (2.0 * PI * sweep_freq * t).sin() * 0.3;

        // Noise bed
        let n = noise((i as f32 * 0.08).fract());
        let noise_env = (-1.2 * t).exp();

        // Sub rumble
        let sub = (2.0 * PI * 40.0 * t).sin() * 0.4;

        let env = (-1.5 * t).exp();
        samples[i] = (sweep + n * 0.5 * noise_env + sub) * env;
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
