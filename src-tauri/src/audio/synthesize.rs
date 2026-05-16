use std::f32::consts::PI;
use super::{SoundType, noise, SAMPLE_RATE};

pub fn generate_base(sound_type: SoundType, duration_ms: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_ms / 1000.0) as usize;
    let mut samples = vec![0.0f32; num_samples];
    match sound_type {
        SoundType::Kick => {
            let start_freq = 150.0;
            let end_freq = 40.0;
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let frac = (i as f32) / (num_samples as f32);
                let freq = start_freq - (start_freq - end_freq) * frac.min(1.0);
                let env = (-8.0 * t).exp();
                samples[i] = (2.0 * PI * freq * t).sin() * env;
            }
        }
        SoundType::Snare => {
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let n = (i as f32 * 0.1).fract();
                let noise_val = noise(n);
                let tone = (2.0 * PI * 200.0 * t).sin();
                let noise_env = (-18.0 * t).exp();
                let tone_env = (-14.0 * t).exp();
                samples[i] = noise_val * noise_env * 0.6 + tone * tone_env * 0.4;
            }
        }
        SoundType::ClosedHat => {
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let n = (i as f32 * 0.3).fract();
                let noise_val = noise(n);
                let env = (-25.0 * t).exp();
                samples[i] = noise_val * env;
            }
            super::dsp::high_pass(&mut samples, 5000.0);
        }
        SoundType::OpenHat => {
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let n = (i as f32 * 0.3).fract();
                let noise_val = noise(n);
                let env = (-5.0 * t).exp();
                samples[i] = noise_val * env;
            }
            super::dsp::high_pass(&mut samples, 4000.0);
        }
        SoundType::Clap => {
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let n = (i as f32 * 0.15).fract();
                let noise_val = noise(n);
                let mut env = (-15.0 * t).exp();
                if t < 0.01 {
                    env *= 0.3;
                }
                samples[i] = noise_val * env;
            }
        }
        SoundType::Tom => {
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let freq = 120.0 - 40.0 * (t / 0.3).min(1.0);
                let env = (-4.0 * t).exp();
                samples[i] = (2.0 * PI * freq * t).sin() * env;
            }
        }
        SoundType::Perc => {
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let n = (i as f32 * 0.5).fract();
                let noise_val = noise(n);
                let tone = (2.0 * PI * 600.0 * t).sin();
                let env = (-20.0 * t).exp();
                samples[i] = (noise_val * 0.5 + tone * 0.5) * env;
            }
        }
        SoundType::Bass => {
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let env = (-2.0 * t).exp();
                let freq = 55.0 + 20.0 * (1.0 - (t / 0.5).min(1.0));
                samples[i] = (2.0 * PI * freq * t).sin() * env;
            }
        }
        SoundType::Fx => {
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let n = (i as f32 * 0.05).fract();
                let noise_val = noise(n);
                let freq = 200.0 + 2000.0 * (t / 0.5).min(1.0);
                let tone = (2.0 * PI * freq * t).sin();
                let env = (-1.5 * t).exp();
                samples[i] = (tone * 0.4 + noise_val * 0.6) * env;
            }
        }
        SoundType::Other => {
            for i in 0..num_samples {
                let t = i as f32 / SAMPLE_RATE as f32;
                let n = (i as f32 * 0.2).fract();
                let noise_val = noise(n);
                let env = (-6.0 * t).exp();
                samples[i] = noise_val * env;
            }
        }
    }
    samples
}
