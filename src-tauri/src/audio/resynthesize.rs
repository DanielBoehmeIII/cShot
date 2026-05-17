use std::f32::consts::PI;
use super::{SoundType, SAMPLE_RATE};
use super::dsp::{self, ClickCharacter, TransientConfig};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ResynthesisParams {
    pub sound_type: SoundType,
    pub duration_ms: f32,
    pub pitch_hz: f32,
    pub pitch_drop_ratio: f32,
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub tail_ms: f32,
    pub noise_amount: f32,
    pub noise_hp_hz: f32,
    pub click_amount: f32,
    pub body_gain: f32,
    pub sub_gain: f32,
    pub saturation_drive: f32,
    pub brightness: f32,
    pub layer_mix: Vec<f32>,
    pub seed: u64,
}

impl Default for ResynthesisParams {
    fn default() -> Self {
        Self {
            sound_type: SoundType::Other,
            duration_ms: 300.0,
            pitch_hz: 200.0,
            pitch_drop_ratio: 0.0,
            attack_ms: 2.0,
            decay_ms: 100.0,
            tail_ms: 50.0,
            noise_amount: 0.0,
            noise_hp_hz: 2000.0,
            click_amount: 0.0,
            body_gain: 1.0,
            sub_gain: 0.0,
            saturation_drive: 1.0,
            brightness: 0.5,
            layer_mix: vec![1.0, 0.0, 0.0, 0.0, 0.0],
            seed: 0,
        }
    }
}

fn frac(x: f32) -> f32 {
    x - x.floor()
}

impl ResynthesisParams {
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn randomize(&self, variation_amount: f32) -> Self {
        let s = self.seed as f32;
        let mut p = self.clone();
        let r1 = frac(s * 1.618);
        let r2 = frac(s * 3.141);
        let r3 = frac(s * 5.789);
        let r4 = frac(s * 7.331);
        let r5 = frac(s * 11.237);
        let r6 = frac(s * 13.971);

        p.duration_ms *= 1.0 + (r1 - 0.5) * variation_amount;
        p.pitch_hz *= 1.0 + (r2 - 0.5) * variation_amount * 0.3;
        p.pitch_drop_ratio = (p.pitch_drop_ratio + (r3 - 0.5) * variation_amount * 0.2).clamp(0.0, 1.0);
        p.attack_ms = (p.attack_ms + (r4 - 0.5) * variation_amount * 5.0).max(0.1);
        p.decay_ms *= 1.0 + (r5 - 0.5) * variation_amount * 0.4;
        p.tail_ms *= 1.0 + (r6 - 0.5) * variation_amount * 0.5;
        p.noise_amount = (p.noise_amount + (r1 - 0.5) * variation_amount * 0.2).clamp(0.0, 1.0);
        p.saturation_drive = (p.saturation_drive + (r2 - 0.5) * variation_amount * 0.3).max(1.0);
        p.brightness = (p.brightness + (r3 - 0.5) * variation_amount * 0.2).clamp(0.0, 1.0);
        p.sub_gain = (p.sub_gain + (r4 - 0.5) * variation_amount * 0.15).clamp(0.0, 1.0);
        p.click_amount = (p.click_amount + (r5 - 0.5) * variation_amount * 0.2).clamp(0.0, 1.0);
        p
    }

    pub fn to_variant(&self, variant_name: &str) -> Self {
        let mut p = self.clone();
        match variant_name {
            "brighter" => p.brightness = (p.brightness + 0.25).min(1.0),
            "darker" => p.brightness = (p.brightness - 0.25).max(0.0),
            "punchier" => { p.click_amount = (p.click_amount + 0.2).min(1.0); p.saturation_drive = (p.saturation_drive + 0.15).min(3.0); }
            "softer" => { p.attack_ms = (p.attack_ms + 5.0).min(30.0); p.click_amount = (p.click_amount * 0.3).max(0.0); }
            "shorter" => { p.duration_ms *= 0.5; p.decay_ms *= 0.5; p.tail_ms *= 0.3; }
            "longer" => { p.duration_ms *= 1.8; p.tail_ms = (p.tail_ms + 100.0).min(2000.0); }
            "distorted" => p.saturation_drive = (p.saturation_drive + 0.6).min(5.0),
            "cleaner" => { p.saturation_drive = 1.0; p.noise_amount = 0.0; }
            "subbier" => p.sub_gain = (p.sub_gain + 0.25).min(1.0),
            "airier" => { p.brightness = (p.brightness + 0.15).min(1.0); p.noise_amount = (p.noise_amount + 0.1).min(0.5); }
            "noisier" => p.noise_amount = (p.noise_amount + 0.2).min(1.0),
            "fattier" => { p.body_gain = (p.body_gain + 0.15).min(1.0); p.sub_gain = (p.sub_gain + 0.1).min(1.0); }
            "tighter" => { p.decay_ms *= 0.5; p.tail_ms = 0.0; }
            "metallic" => { p.brightness = (p.brightness + 0.15).min(1.0); p.pitch_hz *= 1.3; p.saturation_drive = (p.saturation_drive + 0.15).min(3.0); }
            "thinner" => { p.body_gain *= 0.5; p.sub_gain *= 0.3; }
            "warmer" => { p.brightness = (p.brightness - 0.15).max(0.0); p.saturation_drive = (p.saturation_drive + 0.1).min(3.0); }
            _ => {}
        }
        p
    }
}

fn noise(phase: f32) -> f32 {
    ((phase * 127.1).sin() * 43758.5453).fract() * 2.0 - 1.0
}

pub struct ResynthesisLayers {
    pub transient: Vec<f32>,
    pub body: Vec<f32>,
    pub noise: Vec<f32>,
    pub sub: Vec<f32>,
    pub tail: Vec<f32>,
}

pub fn resynthesize_from_analysis(
    analysis: &super::analyze::AudioAnalysis,
) -> Vec<f32> {
    let st = SoundType::from_str(&analysis.sound_type_hint);
    let pitch = analysis.pitch_estimate.unwrap_or(200.0);
    let params = params_for_sound_type(st, pitch, analysis.duration_ms);
    render_layers(&params)
}

pub fn resynthesize(params: &ResynthesisParams) -> Vec<f32> {
    render_layers(params)
}

pub fn params_for_sound_type(sound_type: SoundType, pitch_hz: f32, duration_ms: f32) -> ResynthesisParams {
    let dur = if duration_ms > 0.0 { duration_ms } else { 300.0 };
    let base = ResynthesisParams { sound_type, seed: 0, ..Default::default() };
    match sound_type {
        SoundType::Kick => ResynthesisParams {
            duration_ms: dur,
            pitch_hz: pitch_hz.clamp(40.0, 150.0),
            pitch_drop_ratio: 0.7,
            attack_ms: 1.0,
            decay_ms: (dur * 0.4).min(300.0),
            tail_ms: (dur * 0.3).min(200.0),
            noise_amount: 0.0,
            noise_hp_hz: 5000.0,
            click_amount: 0.6,
            body_gain: 0.8,
            sub_gain: 0.5,
            saturation_drive: 1.3,
            brightness: 0.3,
            layer_mix: vec![0.3, 0.6, 0.0, 0.4, 0.0],
            ..base
        },
        SoundType::Snare => ResynthesisParams {
            duration_ms: dur.min(400.0),
            pitch_hz: pitch_hz.clamp(150.0, 300.0),
            pitch_drop_ratio: 0.15,
            attack_ms: 1.0,
            decay_ms: (dur * 0.3).min(200.0),
            tail_ms: (dur * 0.3).min(150.0),
            noise_amount: 0.7,
            noise_hp_hz: 200.0,
            click_amount: 0.4,
            body_gain: 0.4,
            sub_gain: 0.0,
            saturation_drive: 1.2,
            brightness: 0.6,
            layer_mix: vec![0.2, 0.3, 0.5, 0.0, 0.0],
            ..base
        },
        SoundType::ClosedHat => ResynthesisParams {
            duration_ms: dur.min(200.0),
            pitch_hz: pitch_hz.clamp(200.0, 800.0),
            pitch_drop_ratio: 0.0,
            attack_ms: 0.5,
            decay_ms: (dur * 0.5).min(100.0),
            tail_ms: 0.0,
            noise_amount: 1.0,
            noise_hp_hz: 6000.0,
            click_amount: 0.3,
            body_gain: 0.0,
            sub_gain: 0.0,
            saturation_drive: 1.2,
            brightness: 0.9,
            layer_mix: vec![0.2, 0.0, 0.8, 0.0, 0.0],
            ..base
        },
        SoundType::OpenHat => ResynthesisParams {
            duration_ms: dur.min(800.0),
            pitch_hz: pitch_hz.clamp(200.0, 600.0),
            pitch_drop_ratio: 0.0,
            attack_ms: 1.0,
            decay_ms: (dur * 0.4).min(300.0),
            tail_ms: (dur * 0.3).min(200.0),
            noise_amount: 1.0,
            noise_hp_hz: 4000.0,
            click_amount: 0.2,
            body_gain: 0.0,
            sub_gain: 0.0,
            saturation_drive: 1.2,
            brightness: 0.8,
            layer_mix: vec![0.1, 0.0, 0.9, 0.0, 0.0],
            ..base
        },
        SoundType::Clap => ResynthesisParams {
            duration_ms: dur.min(400.0),
            pitch_hz: pitch_hz.clamp(100.0, 250.0),
            pitch_drop_ratio: 0.0,
            attack_ms: 2.0,
            decay_ms: (dur * 0.3).min(200.0),
            tail_ms: (dur * 0.3).min(150.0),
            noise_amount: 0.9,
            noise_hp_hz: 500.0,
            click_amount: 0.0,
            body_gain: 0.2,
            sub_gain: 0.0,
            saturation_drive: 1.3,
            brightness: 0.7,
            layer_mix: vec![0.0, 0.15, 0.7, 0.0, 0.0],
            ..base
        },
        SoundType::Bass => ResynthesisParams {
            duration_ms: dur.min(1000.0),
            pitch_hz: pitch_hz.clamp(30.0, 120.0),
            pitch_drop_ratio: 0.3,
            attack_ms: 5.0,
            decay_ms: (dur * 0.4).min(400.0),
            tail_ms: (dur * 0.3).min(300.0),
            noise_amount: 0.0,
            noise_hp_hz: 2000.0,
            click_amount: 0.0,
            body_gain: 0.9,
            sub_gain: 0.6,
            saturation_drive: 1.5,
            brightness: 0.2,
            layer_mix: vec![0.0, 0.5, 0.0, 0.4, 0.0],
            ..base
        },
        SoundType::Perc => ResynthesisParams {
            duration_ms: dur.min(300.0),
            pitch_hz: pitch_hz.clamp(200.0, 800.0),
            pitch_drop_ratio: 0.0,
            attack_ms: 1.0,
            decay_ms: (dur * 0.4).min(150.0),
            tail_ms: 0.0,
            noise_amount: 0.5,
            noise_hp_hz: 500.0,
            click_amount: 0.4,
            body_gain: 0.5,
            sub_gain: 0.0,
            saturation_drive: 1.2,
            brightness: 0.5,
            layer_mix: vec![0.2, 0.3, 0.4, 0.0, 0.0],
            ..base
        },
        SoundType::Fx => ResynthesisParams {
            duration_ms: dur.min(2000.0),
            pitch_hz: pitch_hz.clamp(50.0, 500.0),
            pitch_drop_ratio: 0.0,
            attack_ms: 20.0,
            decay_ms: (dur * 0.3).min(600.0),
            tail_ms: (dur * 0.5).min(1000.0),
            noise_amount: 0.6,
            noise_hp_hz: 100.0,
            click_amount: 0.0,
            body_gain: 0.3,
            sub_gain: 0.3,
            saturation_drive: 1.2,
            brightness: 0.5,
            layer_mix: vec![0.0, 0.2, 0.3, 0.2, 0.3],
            ..base
        },
        SoundType::Tom => ResynthesisParams {
            duration_ms: dur.min(500.0),
            pitch_hz: pitch_hz.clamp(80.0, 200.0),
            pitch_drop_ratio: 0.3,
            attack_ms: 2.0,
            decay_ms: (dur * 0.4).min(250.0),
            tail_ms: (dur * 0.2).min(100.0),
            noise_amount: 0.3,
            noise_hp_hz: 500.0,
            click_amount: 0.3,
            body_gain: 0.7,
            sub_gain: 0.2,
            saturation_drive: 1.2,
            brightness: 0.4,
            layer_mix: vec![0.15, 0.5, 0.2, 0.15, 0.0],
            ..base
        },
        SoundType::Other => ResynthesisParams {
            duration_ms: dur.min(500.0),
            pitch_hz,
            pitch_drop_ratio: 0.0,
            attack_ms: 5.0,
            decay_ms: (dur * 0.3).min(200.0),
            tail_ms: 0.0,
            noise_amount: 0.5,
            noise_hp_hz: 200.0,
            click_amount: 0.0,
            body_gain: 0.4,
            sub_gain: 0.0,
            saturation_drive: 1.2,
            brightness: 0.5,
            layer_mix: vec![0.0, 0.3, 0.5, 0.0, 0.0],
            ..base
        },
    }
}

fn render_layers(params: &ResynthesisParams) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * params.duration_ms / 1000.0) as usize;
    if num_samples == 0 { return vec![]; }

    let layers = generate_layers(params, num_samples);
    let mut output = vec![0.0f32; num_samples];

    for i in 0..num_samples {
        let mut val = 0.0f32;
        val += layers.transient.get(i).copied().unwrap_or(0.0) * params.layer_mix[0];
        val += layers.body.get(i).copied().unwrap_or(0.0) * params.layer_mix[1];
        val += layers.noise.get(i).copied().unwrap_or(0.0) * params.layer_mix[2];
        val += layers.sub.get(i).copied().unwrap_or(0.0) * params.layer_mix[3];
        val += layers.tail.get(i).copied().unwrap_or(0.0) * params.layer_mix[4];
        output[i] = val;
    }

    // Multi-stage saturation for analog character
    if params.saturation_drive > 1.01 {
        for s in output.iter_mut() {
            *s = super::dsp::tape_saturation(*s, params.saturation_drive);
        }
        if params.saturation_drive > 2.5 {
            for s in output.iter_mut() {
                *s = super::dsp::tube_saturation(*s, params.saturation_drive * 0.4);
            }
        }
        if params.saturation_drive > 4.0 {
            for s in output.iter_mut() {
                *s = super::dsp::soft_clip(*s, 1.0);
            }
        }
    }

    // Brightness filter
    if params.brightness < 0.35 {
        super::dsp::biquad_low_shelf(&mut output, 3000.0, (params.brightness - 0.5) * 6.0, 0.7);
    } else if params.brightness > 0.65 {
        super::dsp::biquad_high_shelf(&mut output, 3000.0, (params.brightness - 0.5) * 6.0, 0.7);
    }

    // Adaptive compression for punch
    super::dsp::adaptive_compressor(&mut output, -18.0, 2.0, 1.0, 60.0);

    // Subtle analog drift during rendering (built-in character)
    if params.saturation_drive > 1.3 || params.noise_amount > 0.15 {
        let analog_amt = ((params.saturation_drive - 1.0) * 0.03 + params.noise_amount * 0.02).min(0.12);
        if analog_amt > 0.01 {
            super::humanize::apply_analog_drift(&mut output, analog_amt, SAMPLE_RATE, params.seed.wrapping_mul(97));
        }
    }

    // Phase variation across layers (natural phase alignment)
    if params.brightness > 0.3 || params.body_gain > 0.2 {
        let pv_amt = (1.0 - params.brightness).abs() * 0.02 + params.body_gain * 0.015;
        if pv_amt > 0.01 {
            super::humanize::apply_phase_variation(&mut output, pv_amt, params.seed.wrapping_mul(101), SAMPLE_RATE);
        }
    }

    // Layer breathing for non-static feel
    if params.noise_amount > 0.2 || params.body_gain > 0.3 {
        let breath_amt = params.noise_amount * 0.015 + params.body_gain * 0.01;
        if breath_amt > 0.01 {
            super::humanize::apply_layer_breathing(&mut output, breath_amt.min(0.04), params.seed.wrapping_mul(103), SAMPLE_RATE);
        }
    }

    // Multi-band dynamics for punch and clarity
    super::dsp::multiband_compressor(&mut output, 0.4, 0.3, 3.0);

    // De-essing / harshness reduction
    if params.brightness > 0.5 || params.saturation_drive > 2.0 {
        let harshness = ((params.brightness - 0.5) * 0.3 + (params.saturation_drive - 1.0) * 0.05).min(0.3);
        if harshness > 0.01 && params.sound_type != SoundType::ClosedHat && params.sound_type != SoundType::OpenHat {
            super::dsp::biquad_high_shelf(&mut output, 8000.0, -harshness * 4.0, 0.7);
        }
    }

    // Normalize with headroom
    let peak = output.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak > 0.85 {
        let gain = 0.85 / peak;
        for s in output.iter_mut() { *s *= gain; }
    }

    // Final limiting with gentle ceiling
    super::dsp::lookahead_limiter(&mut output, -0.5, 2.0);

    output
}

pub fn generate_layers(params: &ResynthesisParams, num_samples: usize) -> ResynthesisLayers {
    ResynthesisLayers {
        transient: generate_transient_layer(params, num_samples),
        body: generate_body_layer(params, num_samples),
        noise: generate_noise_layer(params, num_samples),
        sub: generate_sub_layer(params, num_samples),
        tail: generate_tail_layer(params, num_samples),
    }
}

fn generate_transient_layer(params: &ResynthesisParams, num_samples: usize) -> Vec<f32> {
    if params.click_amount <= 0.0 { return vec![0.0f32; num_samples]; }

    let click_character = ClickCharacter::for_sound_type(params.sound_type.as_str());
    let click_freq_base = match params.sound_type {
        SoundType::Kick => 3500.0,
        SoundType::Snare | SoundType::Clap => 5500.0,
        SoundType::ClosedHat => 8000.0,
        SoundType::OpenHat => 6000.0,
        SoundType::Perc => 5000.0,
        SoundType::Bass => 2500.0,
        SoundType::Tom => 3000.0,
        _ => 4000.0,
    };

    let sharpness = params.click_amount * (0.5 + 0.5 * (1.0 - params.attack_ms / 50.0).clamp(0.0, 1.0));

    let mut tc = TransientConfig {
        click_character,
        sharpness,
        density: params.click_amount * 0.5,
        pitch_click_hz: click_freq_base,
        click_bandwidth: 0.6,
        ring_decay: 0.08 + params.tail_ms / 2000.0 * 0.12,
        transient_duration_ms: (1.0 + params.attack_ms * 2.0).min(12.0),
        pre_attack_ms: 0.0,
        attack_curve: 2.0,
        transient_saturation: 1.0 + (params.saturation_drive - 1.0) * 0.3,
        multiband_boost: [1.0, 1.0, 1.0, 1.0],
    };

    if params.sound_type == SoundType::Kick {
        tc.multiband_boost = [1.3, 1.0, 0.5, 0.3];
    } else if params.sound_type == SoundType::Snare {
        tc.multiband_boost = [0.5, 0.8, 1.0, 1.2];
    } else if params.sound_type == SoundType::ClosedHat {
        tc.multiband_boost = [0.0, 0.2, 0.8, 1.3];
    }

    let mut layer = super::dsp::generate_click(&tc, num_samples);

    for s in layer.iter_mut() {
        *s *= params.click_amount;
    }

    layer
}

fn generate_body_layer(params: &ResynthesisParams, num_samples: usize) -> Vec<f32> {
    if params.body_gain <= 0.0 { return vec![0.0f32; num_samples]; }
    let mut layer = vec![0.0f32; num_samples];

    let attack_samples = (SAMPLE_RATE as f32 * params.attack_ms / 1000.0) as usize;
    let decay_end = ((attack_samples as f32) + (SAMPLE_RATE as f32 * params.decay_ms / 1000.0)) as usize;
    let decay_end = decay_end.min(num_samples);

    let harmonics: &[(f32, f32)] = match params.sound_type {
        SoundType::Kick => &[(1.0, 1.0), (1.5, 0.4), (2.0, 0.25), (2.5, 0.1), (3.0, 0.05)],
        SoundType::Snare => &[(1.0, 1.0), (1.5, 0.5), (2.0, 0.4), (3.0, 0.2), (4.0, 0.08)],
        SoundType::Bass => &[(1.0, 1.0), (2.0, 0.5), (3.0, 0.3), (4.0, 0.15), (5.0, 0.08), (6.0, 0.04)],
        SoundType::Tom => &[(1.0, 1.0), (2.0, 0.5), (3.0, 0.25), (4.0, 0.1)],
        SoundType::Perc => &[(1.0, 1.0), (2.0, 0.5), (3.0, 0.15)],
        SoundType::Clap => &[(1.0, 0.7), (1.5, 0.5), (2.0, 0.25)],
        _ => &[(1.0, 1.0), (2.0, 0.4), (3.0, 0.15)],
    };
    let norm = harmonics.iter().map(|(_, a)| a).sum::<f32>();

    let body_phase: f32 = 0.0;

    // Pre-warp phase for specific types to simulate analog imperfections
    let phase_warp = match params.sound_type {
        SoundType::Kick => 0.02,
        SoundType::Snare => 0.01,
        SoundType::Bass => 0.03,
        SoundType::Tom => 0.015,
        _ => 0.0,
    };

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (i as f32 / decay_end.max(1) as f32).min(1.0);
        let freq = params.pitch_hz * (1.0 - params.pitch_drop_ratio * frac);
        let pw_mod = 1.0 + phase_warp * (2.0 * PI * 0.5 * t).sin();

        let env = if i < attack_samples {
            let p = i as f32 / attack_samples.max(1) as f32;
            p * (2.0 - p)
        } else {
            let decay_t = t - params.attack_ms / 1000.0;
            if params.decay_ms > 1.0 {
                let decay_rate = 3.5 / (params.decay_ms / 1000.0).max(0.001);
                (-decay_rate * decay_t).exp()
            } else {
                (-12.0 * decay_t).exp()
            }
        };

        let mut val = 0.0;
        for &(harmonic, amp) in harmonics {
            val += (2.0 * PI * freq * harmonic * t * pw_mod + body_phase * harmonic).sin() * amp;
        }

        val = (val / norm.max(1.0)) * env * params.body_gain;

        layer[i] = val;
    }

    // Post-filter body layer for snare to add metallic resonance
    if params.sound_type == SoundType::Snare || params.sound_type == SoundType::Clap {
        let resonance = match params.sound_type {
            SoundType::Snare => 0.15,
            SoundType::Clap => 0.08,
            _ => 0.0,
        };
        if resonance > 0.0 {
            for i in 2..layer.len() {
                layer[i] += layer[i - 1] * resonance * 0.5 - layer[i - 2] * resonance * 0.25;
            }
        }
    }

    layer
}

fn generate_noise_layer(params: &ResynthesisParams, num_samples: usize) -> Vec<f32> {
    if params.noise_amount <= 0.0 { return vec![0.0f32; num_samples]; }
    let mut layer = vec![0.0f32; num_samples];
    let decay_samples = (SAMPLE_RATE as f32 * params.decay_ms / 1000.0) as usize;
    let tail_samples = (SAMPLE_RATE as f32 * params.tail_ms / 1000.0) as usize;

    let s1 = params.seed as f32;

    // Multi-oscillator noise with different densities
    let noise_rates: [(f32, f32, f32); 6] = [
        (0.08, 0.02, 0.25),   // sub rumble
        (0.25, 0.07, 0.20),   // low texture
        (0.50, 0.15, 0.18),   // mid body
        (0.80, 0.30, 0.15),   // upper mid
        (1.20, 0.60, 0.12),   // high sizzle
        (2.00, 1.00, 0.10),   // air
    ];

    // Moving HP filter - sweeps from lower to higher over the sound's life
    let hp_sweep_start = (params.noise_hp_hz * 0.5).max(50.0);
    let hp_sweep_end = (params.noise_hp_hz * 1.5).min(18000.0);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (i as f32 / num_samples.max(1) as f32).min(1.0);

        let mut n = 0.0f32;
        for (rate, spread, amp) in &noise_rates {
            let p1 = (i as f32 * rate + s1 * amp).fract();
            let p2 = (i as f32 * (rate + spread) + s1 * amp * 0.7).fract();
            let p3 = (i as f32 * (rate * 1.5) + s1 * amp * 0.3).fract();
            let noise_val = noise(p1) * 0.5 + noise(p2) * 0.3 + noise(p3) * 0.2;
            n += noise_val * amp;
        }
        n = n.clamp(-1.0, 1.0);

        // Moving HP filter
        let hp_now = hp_sweep_start + (hp_sweep_end - hp_sweep_start) * frac;
        let rc = 1.0 / (2.0 * PI * hp_now.max(20.0));
        let dt = 1.0 / SAMPLE_RATE as f32;
        let alpha = (rc / (rc + dt)).clamp(0.0, 1.0);
        let filtered = if i > 0 {
            n + alpha * (layer[i-1] - n)
        } else {
            n
        };

        // Dual-stage envelope: fast decay then slower tail
        let env = if i < decay_samples {
            let decay_t = t;
            let fast_decay = (-6.0 * decay_t).exp();
            let texture_mod = 1.0 + 0.08 * (2.0 * PI * 3.0 * t).sin();
            fast_decay * texture_mod
        } else {
            let tail_t = t - params.decay_ms / 1000.0;
            let tail_factor = (-2.5 * tail_t).exp();
            let decay_factor = (-6.0 * params.decay_ms / 1000.0).exp();
            let noise_mod = 1.0 + 0.05 * (2.0 * PI * 0.7 * t).sin();
            tail_factor * decay_factor * noise_mod
        };

        layer[i] = filtered * env * params.noise_amount;
    }

    // Multi-stage high-pass for natural sizzle (cleanup after moving filter)
    if params.noise_hp_hz > 20.0 {
        let rc = 1.0 / (2.0 * PI * params.noise_hp_hz);
        let dt = 1.0 / SAMPLE_RATE as f32;
        let alpha = (rc / (rc + dt)).clamp(0.0, 1.0);
        let mut prev1 = 0.0;
        let mut prev2 = 0.0;
        let mut prev3 = 0.0;
        for sample in layer.iter_mut() {
            let input = *sample;
            let tmp1 = alpha * (prev1 + *sample - prev1);
            let tmp2 = alpha * (prev2 + tmp1 - prev2);
            *sample = alpha * (prev3 + tmp2 - prev3);
            prev1 = input;
            prev2 = tmp1;
            prev3 = tmp2;
        }
    }

    // Noise shaping for perceived brightness
    if params.brightness > 0.5 {
        let shaping_amt = (params.brightness - 0.5) * 0.1;
        for i in 0..layer.len() {
            let noise_fb = ((i as f32 * 0.13).fract() * 127.1).sin() * 43758.5453;
            let fb = (noise_fb.fract() * 2.0 - 1.0) * shaping_amt;
            layer[i] = (layer[i] * (1.0 - shaping_amt * 0.5) + fb * shaping_amt).clamp(-1.0, 1.0);
        }
    }

    layer
}

fn generate_sub_layer(params: &ResynthesisParams, num_samples: usize) -> Vec<f32> {
    if params.sub_gain <= 0.0 { return vec![0.0f32; num_samples]; }
    let mut layer = vec![0.0f32; num_samples];
    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let frac = (i as f32 / num_samples as f32).min(1.0);
        let sub_freq = (params.pitch_hz * 0.5).max(25.0) * (1.0 - params.pitch_drop_ratio * 0.5 * frac);
        let sub_freq2 = sub_freq * 0.5;

        let sub_env = (-2.5 * t).exp();
        let env_mod = 1.0 + 0.05 * (2.0 * PI * 0.5 * t).sin();
        let env = sub_env * env_mod;

        let osc1 = (2.0 * PI * sub_freq * t).sin() * 0.6;
        let osc2 = (2.0 * PI * sub_freq2 * t + 0.3).sin() * 0.25;
        let osc3 = (2.0 * PI * sub_freq * 1.5 * t + 0.7).sin() * 0.15;

        let mix = (osc1 + osc2 + osc3) * env * params.sub_gain;
        layer[i] = (mix * 1.5).tanh() * 0.85;
    }
    layer
}

fn generate_tail_layer(params: &ResynthesisParams, num_samples: usize) -> Vec<f32> {
    if params.tail_ms <= 0.0 || params.tail_ms > params.duration_ms * 0.8 {
        return vec![0.0f32; num_samples];
    }
    let mut layer = vec![0.0f32; num_samples];
    let tail_start = ((params.duration_ms - params.tail_ms) / 1000.0 * SAMPLE_RATE as f32) as usize;
    let tail_len = num_samples - tail_start;
    if tail_len <= 0 { return layer; }

    let sub_freq = (params.pitch_hz * 0.4).max(25.0);

    let (tone_amp, noise_amp, decay_rate, texture_density, resonance_q) = match params.sound_type {
        SoundType::Kick => (0.5, 0.04, 1.8, 0.08, 0.6),
        SoundType::Snare => (0.15, 0.3, 2.5, 0.4, 0.3),
        SoundType::OpenHat => (0.03, 0.4, 3.5, 0.5, 0.1),
        SoundType::Bass => (0.65, 0.01, 1.0, 0.02, 0.7),
        SoundType::Fx => (0.25, 0.25, 1.2, 0.3, 0.5),
        SoundType::Clap => (0.1, 0.35, 3.0, 0.45, 0.2),
        SoundType::ClosedHat => (0.02, 0.2, 6.0, 0.3, 0.1),
        SoundType::Perc => (0.2, 0.15, 3.5, 0.2, 0.4),
        SoundType::Tom => (0.4, 0.08, 2.0, 0.12, 0.5),
        _ => (0.35, 0.18, 2.0, 0.2, 0.3),
    };

    let s1 = params.seed as f32;

    // Non-static decay movement
    let mod_rate = 0.2 + params.tail_ms / 2000.0 * 0.5;
    let mod_depth = (0.05 + params.noise_amount * 0.1).min(0.2);

    for i in tail_start..num_samples {
        let local_t = (i - tail_start) as f32 / SAMPLE_RATE as f32;
        let global_t = i as f32 / SAMPLE_RATE as f32;

        // Non-static decay with LFO
        let decay_lfo = 1.0 + mod_depth * (2.0 * PI * mod_rate * global_t).sin();
        let env = (-decay_rate * local_t * decay_lfo).exp() * 0.35;

        // Textured noise with movement
        let n1 = noise((i as f32 * 0.15 + s1 * 0.1).fract());
        let n2 = noise((i as f32 * 0.07 + s1 * 0.07).fract());
        let n3 = noise((i as f32 * 0.03 + s1 * 0.13).fract());
        let n4 = noise((i as f32 * 0.22 + s1 * 0.17).fract());

        // Moving filter on noise (moves from brighter to darker over tail)
        let hp_frac = (local_t / (params.tail_ms / 1000.0).max(0.01)).min(1.0);
        let noise_hp = 500.0 + 3000.0 * (1.0 - hp_frac);
        let rc = 1.0 / (2.0 * PI * noise_hp.max(20.0));
        let dt = 1.0 / SAMPLE_RATE as f32;
        let alpha = (rc / (rc + dt)).clamp(0.0, 1.0);
        let noise_raw = n1 * 0.35 + n2 * 0.25 + n3 * 0.2 + n4 * 0.2;
        let noise_filtered = noise_raw * (1.0 - alpha * 0.7);

        // Texture density variation
        let texture_variation = 0.5 + 0.5 * (2.0 * PI * 0.8 * global_t).sin();
        let noise_val = noise_filtered * noise_amp * (1.0 + texture_density * texture_variation * 0.3);

        // Tonal with resonant behavior
        let tone_phase = 2.0 * PI * sub_freq * local_t;
        let resonance_mod = 1.0 + resonance_q * 0.3 * (2.0 * PI * sub_freq * 1.5 * local_t).sin() * (-3.0 * local_t).exp();
        let tone = (tone_phase.sin() * tone_amp
            + (tone_phase * 1.5).sin() * tone_amp * 0.2
            + (tone_phase * 2.0).sin() * tone_amp * 0.05) * resonance_mod;

        layer[i] = (tone + noise_val) * env;
    }

    // Apply analog instability
    super::dsp::analog_tail_instability(&mut layer[tail_start..], 0.03, s1);

    // Apply resonant filter for cinematic tails
    if resonance_q > 0.3 {
        let res_freq = match params.sound_type {
            SoundType::Kick => sub_freq * 3.0,
            SoundType::Bass => sub_freq * 2.0,
            SoundType::Fx => 100.0,
            _ => sub_freq * 4.0,
        };
        let config = super::dsp::TailTextureConfig {
            resonant_q: resonance_q * 0.5,
            resonant_freq: res_freq,
            resonant_modulation: 0.3,
            ..Default::default()
        };
        super::dsp::resonant_tail_filter(&mut layer[tail_start..], res_freq, resonance_q * 0.5, 0.3);
    }

    layer
}
