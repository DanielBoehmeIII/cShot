use super::resynthesize::{self, ResynthesisParams, ResynthesisLayers};
use super::SoundType;
use super::dsp;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LayerSolo {
    pub transient: bool,
    pub body: bool,
    pub noise: bool,
    pub sub: bool,
    pub tail: bool,
}

impl Default for LayerSolo {
    fn default() -> Self {
        Self {
            transient: true,
            body: true,
            noise: true,
            sub: true,
            tail: true,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SoundDesignerControls {
    pub layer_solo: LayerSolo,
    pub transient_layer: TransientLayerControls,
    pub body_layer: BodyLayerControls,
    pub noise_layer: NoiseLayerControls,
    pub sub_layer: SubLayerControls,
    pub tail_layer: TailLayerControls,
    pub global: GlobalControls,
}

impl Default for SoundDesignerControls {
    fn default() -> Self {
        Self {
            layer_solo: LayerSolo::default(),
            transient_layer: TransientLayerControls::default(),
            body_layer: BodyLayerControls::default(),
            noise_layer: NoiseLayerControls::default(),
            sub_layer: SubLayerControls::default(),
            tail_layer: TailLayerControls::default(),
            global: GlobalControls::default(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TransientLayerControls {
    pub enabled: bool,
    pub amount: f32,
    pub sharpness: f32,
    pub click_character: String,
    pub pitch_hz: f32,
    pub duration_ms: f32,
    pub saturation: f32,
    pub noise_blend: f32,
    pub high_boost_db: f32,
    pub low_boost_db: f32,
    pub replace_source: Option<Vec<f32>>,
}

impl Default for TransientLayerControls {
    fn default() -> Self {
        Self {
            enabled: true,
            amount: 0.5,
            sharpness: 0.7,
            click_character: "sharp".to_string(),
            pitch_hz: 4000.0,
            duration_ms: 8.0,
            saturation: 1.0,
            noise_blend: 0.2,
            high_boost_db: 0.0,
            low_boost_db: 0.0,
            replace_source: None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BodyLayerControls {
    pub enabled: bool,
    pub amount: f32,
    pub pitch_hz: f32,
    pub pitch_drop: f32,
    pub thickness: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub harmonic_richness: f32,
    pub resonance: f32,
    pub replace_source: Option<Vec<f32>>,
}

impl Default for BodyLayerControls {
    fn default() -> Self {
        Self {
            enabled: true,
            amount: 0.7,
            pitch_hz: 200.0,
            pitch_drop: 0.3,
            thickness: 0.5,
            brightness: 0.5,
            saturation: 1.2,
            harmonic_richness: 0.5,
            resonance: 0.0,
            replace_source: None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NoiseLayerControls {
    pub enabled: bool,
    pub amount: f32,
    pub high_pass_hz: f32,
    pub density: f32,
    pub movement: f32,
    pub brightness: f32,
    pub texture_layers: usize,
    pub replace_source: Option<Vec<f32>>,
}

impl Default for NoiseLayerControls {
    fn default() -> Self {
        Self {
            enabled: true,
            amount: 0.5,
            high_pass_hz: 2000.0,
            density: 0.5,
            movement: 0.3,
            brightness: 0.5,
            texture_layers: 2,
            replace_source: None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SubLayerControls {
    pub enabled: bool,
    pub amount: f32,
    pub pitch_hz: f32,
    pub pitch_drop: f32,
    pub saturation: f32,
    pub harmonics: bool,
    pub replace_source: Option<Vec<f32>>,
}

impl Default for SubLayerControls {
    fn default() -> Self {
        Self {
            enabled: true,
            amount: 0.4,
            pitch_hz: 55.0,
            pitch_drop: 0.3,
            saturation: 1.5,
            harmonics: true,
            replace_source: None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TailLayerControls {
    pub enabled: bool,
    pub amount: f32,
    pub length_ms: f32,
    pub tone_amp: f32,
    pub noise_amp: f32,
    pub decay_rate: f32,
    pub resonance: f32,
    pub texture_density: f32,
    pub analog_instability: f32,
    pub replace_source: Option<Vec<f32>>,
}

impl Default for TailLayerControls {
    fn default() -> Self {
        Self {
            enabled: true,
            amount: 0.5,
            length_ms: 200.0,
            tone_amp: 0.35,
            noise_amp: 0.18,
            decay_rate: 2.0,
            resonance: 0.3,
            texture_density: 0.2,
            analog_instability: 0.03,
            replace_source: None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GlobalControls {
    pub master_gain: f32,
    pub compression: f32,
    pub limiter: bool,
    pub normalize: bool,
}

impl Default for GlobalControls {
    fn default() -> Self {
        Self {
            master_gain: 1.0,
            compression: 0.5,
            limiter: true,
            normalize: true,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LayerVisualization {
    pub transient_waveform: Vec<f32>,
    pub body_waveform: Vec<f32>,
    pub noise_waveform: Vec<f32>,
    pub sub_waveform: Vec<f32>,
    pub tail_waveform: Vec<f32>,
    pub combined_waveform: Vec<f32>,
    pub layer_peaks: [f32; 5],
    pub layer_rms: [f32; 5],
}

pub fn render_sound_designer(
    params: &ResynthesisParams,
    controls: &SoundDesignerControls,
) -> Vec<f32> {
    let num_samples = (super::SAMPLE_RATE as f32 * params.duration_ms / 1000.0) as usize;
    if num_samples == 0 { return vec![]; }

    let layers = resynthesize::generate_layers(params, num_samples);
    let mut output = vec![0.0f32; num_samples];
    let solo = &controls.layer_solo;

    if controls.transient_layer.enabled && solo.transient {
        let mut tl = if let Some(ref replace) = controls.transient_layer.replace_source {
            replace.clone()
        } else {
            layers.transient.clone()
        };
        let s = controls.transient_layer.amount;
        let gain_db = controls.transient_layer.high_boost_db * 0.5;
        if gain_db.abs() > 0.5 {
            dsp::biquad_high_shelf(&mut tl, 5000.0, gain_db, 0.7);
        }
        let low_db = controls.transient_layer.low_boost_db * 0.5;
        if low_db.abs() > 0.5 {
            dsp::biquad_low_shelf(&mut tl, 200.0, low_db, 0.7);
        }
        if controls.transient_layer.saturation > 1.01 {
            for x in tl.iter_mut() { *x = dsp::tape_saturation(*x, controls.transient_layer.saturation); }
        }
        for i in 0..num_samples {
            output[i] += tl.get(i).copied().unwrap_or(0.0) * s;
        }
    }

    if controls.body_layer.enabled && solo.body {
        let mut bl = if let Some(ref replace) = controls.body_layer.replace_source {
            replace.clone()
        } else {
            layers.body.clone()
        };
        let bt = controls.body_layer.thickness;
        if (bt - 0.5).abs() > 0.05 {
            let gain = 1.0 + (bt - 0.5) * 0.5;
            for x in bl.iter_mut() { *x *= gain; }
        }
        if controls.body_layer.saturation > 1.01 {
            for x in bl.iter_mut() { *x = dsp::tape_saturation(*x, controls.body_layer.saturation); }
        }
        if controls.body_layer.resonance > 0.01 {
            for i in 2..bl.len() {
                bl[i] += bl[i - 1] * controls.body_layer.resonance * 0.3;
            }
        }
        for i in 0..num_samples {
            output[i] += bl.get(i).copied().unwrap_or(0.0) * controls.body_layer.amount;
        }
    }

    if controls.noise_layer.enabled && solo.noise {
        let mut nl = if let Some(ref replace) = controls.noise_layer.replace_source {
            replace.clone()
        } else {
            layers.noise.clone()
        };
        if controls.noise_layer.brightness > 0.5 {
            let shelf_db = (controls.noise_layer.brightness - 0.5) * 4.0;
            dsp::biquad_high_shelf(&mut nl, 4000.0, shelf_db, 0.7);
        }
        for i in 0..num_samples {
            output[i] += nl.get(i).copied().unwrap_or(0.0) * controls.noise_layer.amount;
        }
    }

    if controls.sub_layer.enabled && solo.sub {
        let mut sl = if let Some(ref replace) = controls.sub_layer.replace_source {
            replace.clone()
        } else {
            layers.sub.clone()
        };
        if controls.sub_layer.saturation > 1.01 {
            for x in sl.iter_mut() { *x = dsp::tape_saturation(*x, controls.sub_layer.saturation); }
        }
        for i in 0..num_samples {
            output[i] += sl.get(i).copied().unwrap_or(0.0) * controls.sub_layer.amount;
        }
    }

    if controls.tail_layer.enabled && solo.tail {
        let mut tll = if let Some(ref replace) = controls.tail_layer.replace_source {
            replace.clone()
        } else {
            layers.tail.clone()
        };
        if controls.tail_layer.analog_instability > 0.01 {
            dsp::analog_tail_instability(&mut tll, controls.tail_layer.analog_instability, 42.0);
        }
        if controls.tail_layer.resonance > 0.1 {
            dsp::resonant_tail_filter(&mut tll, params.pitch_hz * 0.4, controls.tail_layer.resonance * 0.5, 0.3);
        }
        for i in 0..num_samples {
            output[i] += tll.get(i).copied().unwrap_or(0.0) * controls.tail_layer.amount;
        }
    }

    if controls.global.compression > 0.01 {
        let ratio = 2.0 + controls.global.compression * 3.0;
        let threshold = -12.0 + (1.0 - controls.global.compression) * 12.0;
        dsp::adaptive_compressor(&mut output, threshold, ratio, 3.0, 40.0);
    }

    if controls.global.limiter {
        dsp::lookahead_limiter(&mut output, -0.5, 2.0);
    }

    if controls.global.normalize {
        let peak = output.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        if peak > 0.85 {
            let gain = 0.85 / peak;
            for s in output.iter_mut() { *s *= gain; }
        }
    }

    output
}

pub fn compute_layer_visualization(
    params: &ResynthesisParams,
    controls: &SoundDesignerControls,
    num_points: usize,
) -> LayerVisualization {
    let num_samples = (super::SAMPLE_RATE as f32 * params.duration_ms / 1000.0) as usize;
    if num_samples == 0 {
        return LayerVisualization {
            transient_waveform: vec![],
            body_waveform: vec![],
            noise_waveform: vec![],
            sub_waveform: vec![],
            tail_waveform: vec![],
            combined_waveform: vec![],
            layer_peaks: [0.0; 5],
            layer_rms: [0.0; 5],
        };
    }

    let layers = resynthesize::generate_layers(params, num_samples);
    let combined = render_sound_designer(params, controls);

    let step = (num_samples / num_points.max(1)).max(1);
    let down_sample = |v: &[f32]| -> Vec<f32> {
        v.iter().step_by(step).copied().collect()
    };

    let calc_peak = |v: &[f32]| -> f32 { v.iter().map(|s| s.abs()).fold(0.0f32, f32::max) };
    let calc_rms = |v: &[f32]| -> f32 {
        let sum: f32 = v.iter().map(|s| s * s).sum();
        (sum / v.len().max(1) as f32).sqrt()
    };

    LayerVisualization {
        transient_waveform: down_sample(&layers.transient),
        body_waveform: down_sample(&layers.body),
        noise_waveform: down_sample(&layers.noise),
        sub_waveform: down_sample(&layers.sub),
        tail_waveform: down_sample(&layers.tail),
        combined_waveform: down_sample(&combined),
        layer_peaks: [
            calc_peak(&layers.transient),
            calc_peak(&layers.body),
            calc_peak(&layers.noise),
            calc_peak(&layers.sub),
            calc_peak(&layers.tail),
        ],
        layer_rms: [
            calc_rms(&layers.transient),
            calc_rms(&layers.body),
            calc_rms(&layers.noise),
            calc_rms(&layers.sub),
            calc_rms(&layers.tail),
        ],
    }
}

pub fn swap_layer(source: &ResynthesisLayers, target: &ResynthesisLayers, layer_name: &str) -> ResynthesisLayers {
    let mut result = ResynthesisLayers {
        transient: source.transient.clone(),
        body: source.body.clone(),
        noise: source.noise.clone(),
        sub: source.sub.clone(),
        tail: source.tail.clone(),
    };
    match layer_name {
        "transient" => result.transient = target.transient.clone(),
        "body" => result.body = target.body.clone(),
        "noise" => result.noise = target.noise.clone(),
        "sub" => result.sub = target.sub.clone(),
        "tail" => result.tail = target.tail.clone(),
        _ => {}
    }
    result
}

pub fn render_swapped_layers(base: &ResynthesisParams, layers: &ResynthesisLayers, layer_mix: &[f32; 5]) -> Vec<f32> {
    let num_samples = (super::SAMPLE_RATE as f32 * base.duration_ms / 1000.0) as usize;
    if num_samples == 0 { return vec![]; }
    let mut output = vec![0.0f32; num_samples];
    for i in 0..num_samples {
        let mut val = 0.0;
        val += layers.transient.get(i).copied().unwrap_or(0.0) * layer_mix[0];
        val += layers.body.get(i).copied().unwrap_or(0.0) * layer_mix[1];
        val += layers.noise.get(i).copied().unwrap_or(0.0) * layer_mix[2];
        val += layers.sub.get(i).copied().unwrap_or(0.0) * layer_mix[3];
        val += layers.tail.get(i).copied().unwrap_or(0.0) * layer_mix[4];
        output[i] = val;
    }
    output
}

pub fn solo_layer(params: &ResynthesisParams, layer_name: &str) -> Vec<f32> {
    let n = (super::SAMPLE_RATE as f32 * params.duration_ms / 1000.0) as usize;
    if n == 0 { return vec![]; }
    let layers = resynthesize::generate_layers(params, n);
    match layer_name {
        "transient" => layers.transient,
        "body" => layers.body,
        "noise" => layers.noise,
        "sub" => layers.sub,
        "tail" => layers.tail,
        _ => vec![0.0f32; n],
    }
}
