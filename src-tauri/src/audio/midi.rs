use super::SoundType;
use super::resynthesize::{self, ResynthesisParams};

// ─── MIDI Note to Sound Mapping ─────────────────────────

/// Standard drum MIDI note mapping (General MIDI subset)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrumNote {
    Kick = 36,
    Snare = 38,
    ClosedHat = 42,
    OpenHat = 46,
    Clap = 39,
    TomLow = 41,
    TomMid = 45,
    TomHigh = 48,
    Perc = 70,
    Crash = 49,
    Ride = 51,
}

impl DrumNote {
    pub fn from_midi_note(note: u8) -> Option<(SoundType, f32)> {
        match note {
            36 | 35 => Some((SoundType::Kick, 60.0)),
            37 | 38 | 40 => Some((SoundType::Snare, 200.0)),
            39 => Some((SoundType::Clap, 180.0)),
            42 | 44 => Some((SoundType::ClosedHat, 400.0)),
            46 => Some((SoundType::OpenHat, 300.0)),
            41 => Some((SoundType::Tom, 80.0)),
            43 => Some((SoundType::Tom, 100.0)),
            45 => Some((SoundType::Tom, 120.0)),
            47 => Some((SoundType::Tom, 140.0)),
            48 | 50 => Some((SoundType::Tom, 180.0)),
            49 | 57 => Some((SoundType::Fx, 200.0)),
            51 | 53 => Some((SoundType::Fx, 300.0)),
            54..=72 => Some((SoundType::Perc, note as f32 * 8.0)),
            _ => {
                if note < 36 {
                    Some((SoundType::Bass, 40.0 + note as f32 * 2.0))
                } else if note > 72 {
                    Some((SoundType::Perc, note as f32 * 10.0))
                } else {
                    None
                }
            }
        }
    }
}

// ─── Velocity Mapping ────────────────────────────────────

pub fn velocity_to_gain(velocity: u8) -> f32 {
    (velocity as f32 / 127.0).clamp(0.0, 1.0)
}

pub fn velocity_to_punch(velocity: u8) -> f32 {
    let norm = velocity as f32 / 127.0;
    0.5 + norm * 0.5
}

pub fn velocity_to_click(velocity: u8) -> f32 {
    let norm = velocity as f32 / 127.0;
    norm * norm
}

pub fn velocity_to_decay(velocity: u8) -> f32 {
    let norm = velocity as f32 / 127.0;
    0.6 + norm * 0.4
}

pub fn params_for_velocity(params: &ResynthesisParams, velocity: u8) -> ResynthesisParams {
    let mut p = params.clone();
    let gain = velocity_to_gain(velocity);
    p.body_gain = p.body_gain * (0.3 + gain * 0.7);
    p.click_amount = p.click_amount * velocity_to_click(velocity);
    p.decay_ms = p.decay_ms * velocity_to_decay(velocity);
    p
}

// ─── Rapid Randomization ─────────────────────────────────

pub fn randomize_params_fast(params: &ResynthesisParams, amount: f32) -> ResynthesisParams {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    params.clone().with_seed(nanos).randomize(amount)
}

pub fn generate_rapid_variants(params: &ResynthesisParams, count: usize, amount: f32) -> Vec<Vec<f32>> {
    let mut variants = Vec::with_capacity(count);
    for i in 0..count {
        let seed = i as u64 ^ 0xABCD;
        let p = params.clone().with_seed(seed).randomize(amount);
        let s = resynthesize::resynthesize(&p);
        if !s.is_empty() {
            variants.push(s);
        }
    }
    variants
}

// ─── Preset Morphing ─────────────────────────────────────

pub fn morph_params(a: &ResynthesisParams, b: &ResynthesisParams, t: f32) -> ResynthesisParams {
    let t = t.clamp(0.0, 1.0);
    ResynthesisParams {
        sound_type: if t < 0.5 { a.sound_type } else { b.sound_type },
        duration_ms: a.duration_ms + (b.duration_ms - a.duration_ms) * t,
        pitch_hz: a.pitch_hz + (b.pitch_hz - a.pitch_hz) * t,
        pitch_drop_ratio: a.pitch_drop_ratio + (b.pitch_drop_ratio - a.pitch_drop_ratio) * t,
        attack_ms: a.attack_ms + (b.attack_ms - a.attack_ms) * t,
        decay_ms: a.decay_ms + (b.decay_ms - a.decay_ms) * t,
        tail_ms: a.tail_ms + (b.tail_ms - a.tail_ms) * t,
        noise_amount: a.noise_amount + (b.noise_amount - a.noise_amount) * t,
        noise_hp_hz: a.noise_hp_hz + (b.noise_hp_hz - a.noise_hp_hz) * t,
        click_amount: a.click_amount + (b.click_amount - a.click_amount) * t,
        body_gain: a.body_gain + (b.body_gain - a.body_gain) * t,
        sub_gain: a.sub_gain + (b.sub_gain - a.sub_gain) * t,
        saturation_drive: a.saturation_drive + (b.saturation_drive - a.saturation_drive) * t,
        brightness: a.brightness + (b.brightness - a.brightness) * t,
        layer_mix: merge_layer_mix(&a.layer_mix, &b.layer_mix, t),
        seed: if t < 0.5 { a.seed } else { b.seed },
        stereo_width: a.stereo_width + (b.stereo_width - a.stereo_width) * t,
        filter_sweep: a.filter_sweep + (b.filter_sweep - a.filter_sweep) * t,
        metallic_amount: a.metallic_amount + (b.metallic_amount - a.metallic_amount) * t,
    }
}

fn merge_layer_mix(a: &[f32], b: &[f32], t: f32) -> Vec<f32> {
    let len = a.len().max(b.len());
    (0..len).map(|i| {
        let av = a.get(i).copied().unwrap_or(0.0);
        let bv = b.get(i).copied().unwrap_or(0.0);
        av + (bv - av) * t
    }).collect()
}

// ─── Parameter Automation ─────────────────────────────────

#[derive(Clone, Debug)]
pub struct AutomationPoint {
    pub time_ms: f32,
    pub value: f32,
}

#[derive(Clone, Debug)]
pub struct AutomationLane {
    pub param_name: String,
    pub points: Vec<AutomationPoint>,
}

impl AutomationLane {
    pub fn new(param_name: &str) -> Self {
        Self {
            param_name: param_name.to_string(),
            points: Vec::new(),
        }
    }

    pub fn add_point(&mut self, time_ms: f32, value: f32) {
        self.points.push(AutomationPoint { time_ms, value });
        self.points.sort_by(|a, b| a.time_ms.partial_cmp(&b.time_ms).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn value_at(&self, time_ms: f32) -> f32 {
        if self.points.is_empty() { return 0.5; }
        if self.points.len() == 1 { return self.points[0].value; }

        if time_ms <= self.points[0].time_ms {
            return self.points[0].value;
        }
        if time_ms >= self.points.last().unwrap().time_ms {
            return self.points.last().unwrap().value;
        }

        for i in 0..self.points.len() - 1 {
            let a = &self.points[i];
            let b = &self.points[i + 1];
            if time_ms >= a.time_ms && time_ms <= b.time_ms {
                let t = (time_ms - a.time_ms) / (b.time_ms - a.time_ms).max(0.001);
                return a.value + (b.value - a.value) * t;
            }
        }
        0.5
    }
}

// ─── Live Preview ────────────────────────────────────────

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PreviewConfig {
    pub sound_type: String,
    pub pitch_hz: f32,
    pub duration_ms: f32,
    pub character: f32,
    pub weight: f32,
    pub length_pct: f32,
    pub punch: f32,
    pub velocity: u8,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            sound_type: "kick".to_string(),
            pitch_hz: 60.0,
            duration_ms: 300.0,
            character: 0.0,
            weight: 0.5,
            length_pct: 0.5,
            punch: 0.5,
            velocity: 100,
        }
    }
}

impl PreviewConfig {
    pub fn to_params(&self) -> ResynthesisParams {
        let st = SoundType::from_str(&self.sound_type);
        let mut base = resynthesize::params_for_sound_type(st, self.pitch_hz, self.duration_ms);

        base.seed = 42;

        // Apply simple mode controls
        base.brightness = (base.brightness + self.character * 0.3).clamp(0.0, 1.0);
        base.saturation_drive = (base.saturation_drive + self.character.max(0.0) * 0.5).max(1.0);
        base.body_gain = (base.body_gain * (0.5 + self.weight * 0.5)).clamp(0.1, 1.0);
        base.sub_gain = (base.sub_gain + self.weight * 0.3).clamp(0.0, 1.0);
        base.duration_ms = base.duration_ms * (0.4 + self.length_pct * 0.8);
        base.click_amount = base.click_amount * (0.2 + self.punch * 0.8);
        base.attack_ms = base.attack_ms * (1.0 + (1.0 - self.punch) * 0.5);

        // Apply velocity
        let gain = self.velocity as f32 / 127.0;
        base.body_gain *= 0.3 + gain * 0.7;
        base.click_amount *= velocity_to_click(self.velocity);
        base.decay_ms *= velocity_to_decay(self.velocity);

        base
    }

    pub fn generate(&self) -> Vec<f32> {
        let params = self.to_params();
        resynthesize::resynthesize(&params)
    }
}

// ─── Sound Type from MIDI Program Change ─────────────────

pub fn sound_type_from_program(program: u8) -> SoundType {
    match program {
        0..=10 => SoundType::Kick,
        11..=20 => SoundType::Snare,
        21..=30 => SoundType::ClosedHat,
        31..=40 => SoundType::OpenHat,
        41..=45 => SoundType::Clap,
        46..=55 => SoundType::Tom,
        56..=70 => SoundType::Perc,
        71..=80 => SoundType::Bass,
        81..=100 => SoundType::Fx,
        _ => SoundType::Other,
    }
}
