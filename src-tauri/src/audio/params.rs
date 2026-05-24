use super::humanize::HumanizeParams;
use super::resynthesize::ResynthesisParams;
use super::SoundType;

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ControlMode {
    Simple,
    Advanced,
    SoundDesigner,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExposedParams {
    pub mode: ControlMode,

    // Simple mode (5 controls)
    pub character: f32,       // -1.0 (dark/warm) to 1.0 (bright/aggressive)
    pub weight: f32,          // 0.0 (thin) to 1.0 (heavy)
    pub length: f32,          // 0.0 (short) to 1.0 (long)
    pub punch: f32,           // 0.0 (soft) to 1.0 (punchy)
    pub complexity: f32,      // 0.0 (clean) to 1.0 (complex/noisy)

    // Advanced mode (additional controls)
    pub transient: f32,       // 0.0-1.0
    pub body: f32,            // 0.0-1.0
    pub tail: f32,            // 0.0-1.0
    pub saturation: f32,      // 0.0-1.0
    pub noise: f32,           // 0.0-1.0
    pub sub: f32,             // 0.0-1.0
    pub brightness: f32,      // 0.0-1.0
    pub decay: f32,           // 0.0-1.0

    // Sound Designer mode (full controls)
    pub stereo_width: f32,    // 0.0-1.0
    pub texture: f32,         // 0.0-1.0 (tonal vs noise blend)
    pub pitch_offset: f32,    // -12.0 to 12.0 semitones
    pub attack: f32,          // 0.0-1.0
    pub resonance: f32,       // 0.0-1.0
    pub spectral_tilt: f32,   // -1.0 to 1.0
    pub distortion_type: f32, // 0.0=tape, 0.33=tube, 0.66=soft, 1.0=hard
    pub envelope_curve: f32,  // 0.0=linear, 0.5=exponential, 1.0=logarithmic
    // Humanization controls (analog feel)
    pub analog_feel: f32,     // 0.0-1.0 (amount of analog drift + instability)
    pub humanize: f32,        // 0.0-1.0 (humanized transients + envelope variation)
}

impl Default for ExposedParams {
    fn default() -> Self {
        Self {
            mode: ControlMode::Simple,
            character: 0.0,
            weight: 0.5,
            length: 0.5,
            punch: 0.5,
            complexity: 0.3,
            transient: 0.5,
            body: 0.5,
            tail: 0.3,
            saturation: 0.3,
            noise: 0.3,
            sub: 0.3,
            brightness: 0.5,
            decay: 0.5,
            stereo_width: 0.0,
            texture: 0.5,
            pitch_offset: 0.0,
            attack: 0.5,
            resonance: 0.3,
            spectral_tilt: 0.0,
            distortion_type: 0.0,
            envelope_curve: 0.5,
            analog_feel: 0.0,
            humanize: 0.0,
        }
    }
}

impl ExposedParams {
    pub fn simple(character: f32, weight: f32, length: f32, punch: f32, complexity: f32) -> Self {
        let mut p = Self {
            mode: ControlMode::Simple,
            ..Self::default()
        };
        p.character = character.clamp(-1.0, 1.0);
        p.weight = weight.clamp(0.0, 1.0);
        p.length = length.clamp(0.0, 1.0);
        p.punch = punch.clamp(0.0, 1.0);
        p.complexity = complexity.clamp(0.0, 1.0);
        p
    }

    pub fn to_resynthesis_params(&self, sound_type: SoundType, pitch_hz: f32, duration_ms: f32) -> ResynthesisParams {
        let base = super::resynthesize::params_for_sound_type(sound_type, pitch_hz, duration_ms);
        self.apply_to_base(&base)
    }

    fn apply_to_base(&self, base: &ResynthesisParams) -> ResynthesisParams {
        let mut p = base.clone();

        // Map character: -1=dark/warm, 0=neutral, 1=bright/aggressive
        let char_idx = self.character * 0.5 + 0.5; // 0.0-1.0
        p.brightness = (base.brightness + (char_idx - 0.5) * 0.6).clamp(0.0, 1.0);
        p.saturation_drive = (base.saturation_drive + (char_idx - 0.5) * 0.8).max(1.0);

        // Map weight: 0=thin, 1=heavy
        p.body_gain = (base.body_gain * (0.5 + self.weight * 0.5)).clamp(0.1, 1.0);
        p.sub_gain = (base.sub_gain + self.weight * 0.3).clamp(0.0, 1.0);

        // Map length: 0=short, 1=long
        p.duration_ms = base.duration_ms * (0.4 + self.length * 0.8);
        p.tail_ms = base.tail_ms * (0.2 + self.length * 1.2);
        p.decay_ms = base.decay_ms * (0.5 + self.length * 0.8);

        // Map punch: 0=soft, 1=punchy
        p.attack_ms = base.attack_ms * (1.0 + (1.0 - self.punch) * 0.5);
        p.click_amount = base.click_amount * (0.2 + self.punch * 0.8);

        // Map complexity: 0=clean, 1=complex/noisy
        p.noise_amount = (base.noise_amount + self.complexity * 0.3).clamp(0.0, 1.0);
        if self.complexity > 0.5 {
            p.saturation_drive = (p.saturation_drive + (self.complexity - 0.5) * 0.4).max(1.0);
        }

        // Advanced: transient
        if self.transient != 0.5 {
            p.click_amount = (p.click_amount * (self.transient * 2.0)).clamp(0.0, 1.0);
            if self.transient < 0.5 {
                p.attack_ms = (p.attack_ms * (1.0 + (0.5 - self.transient))).min(30.0);
            }
        }

        // Advanced: body
        if self.body != 0.5 {
            p.body_gain = (p.body_gain * (self.body * 2.0)).clamp(0.0, 1.0);
        }

        // Advanced: tail
        if self.tail != 0.3 {
            p.tail_ms = base.tail_ms * (0.2 + self.tail * 1.5);
        }

        // Advanced: saturation
        if self.saturation > 0.0 {
            p.saturation_drive = (base.saturation_drive + self.saturation * 1.5).max(1.0);
        }

        // Advanced: noise
        if self.noise > 0.0 {
            p.noise_amount = (p.noise_amount + self.noise * 0.3).clamp(0.0, 1.0);
        }

        // Advanced: sub
        if self.sub != 0.3 {
            p.sub_gain = (base.sub_gain + self.sub * 0.4).clamp(0.0, 1.0);
        }

        // Advanced: brightness (overrides character)
        if self.brightness != 0.5 {
            p.brightness = self.brightness;
        }

        // Advanced: decay
        if self.decay != 0.5 {
            let factor = 0.3 + self.decay * 1.4;
            p.decay_ms = base.decay_ms * factor;
        }

        // Sound Designer: pitch offset
        if self.pitch_offset.abs() > 0.5 {
            let ratio = 2.0_f32.powf(self.pitch_offset / 12.0);
            p.pitch_hz = base.pitch_hz * ratio;
        }

        // Sound Designer: attack (overrides)
        if self.attack != 0.5 {
            p.attack_ms = base.attack_ms * (0.2 + self.attack * 1.8);
        }

        // Sound Designer: spectral tilt (character override)
        if self.spectral_tilt.abs() > 0.05 {
            let tilt_brightness = 0.5 + self.spectral_tilt * 0.5;
            p.brightness = tilt_brightness.clamp(0.0, 1.0);
        }

        // Sound Designer: texture (tonal vs noise blend)
        if self.texture != 0.5 {
            let noise_scale = self.texture * 0.8;
            let tonal_scale = (1.0 - self.texture) * 0.8;
            p.noise_amount = (p.noise_amount * (0.2 + noise_scale)).clamp(0.0, 1.0);
            p.body_gain = (p.body_gain * (0.2 + tonal_scale)).clamp(0.0, 1.0);
        }

        p
    }

    pub fn to_humanize_params(&self) -> HumanizeParams {
        let af = self.analog_feel.clamp(0.0, 1.0);
        let hu = self.humanize.clamp(0.0, 1.0);
        HumanizeParams {
            analog_drift: af * 0.3,
            instability: af * 0.1,
            transient_randomness: hu * 0.15,
            envelope_variation: hu * 0.1,
            saturation_randomness: af * 0.15,
            non_static_layers: (af + hu) * 0.5 * 0.1,
            phase_variation: af * 0.1,
            humanize_transients: hu * 0.2,
        }
    }
}

// ─── Mode Presets ───────────────────────────────────────

pub fn simple_mode_preset(character: f32, weight: f32, length: f32, punch: f32, complexity: f32) -> ExposedParams {
    ExposedParams::simple(character, weight, length, punch, complexity)
}

pub fn advanced_mode_preset(
    character: f32, weight: f32, length: f32, punch: f32, complexity: f32,
    transient: f32, body: f32, tail: f32, saturation: f32,
    noise: f32, sub: f32, brightness: f32, decay: f32,
) -> ExposedParams {
    ExposedParams {
        mode: ControlMode::Advanced,
        character, weight, length, punch, complexity,
        transient, body, tail, saturation, noise, sub, brightness, decay,
        ..Default::default()
    }
}

pub fn sound_designer_mode_preset(
    character: f32, weight: f32, length: f32, punch: f32, complexity: f32,
    transient: f32, body: f32, tail: f32, saturation: f32,
    noise: f32, sub: f32, brightness: f32, decay: f32,
    stereo_width: f32, texture: f32, pitch_offset: f32,
    attack: f32, resonance: f32, spectral_tilt: f32,
    distortion_type: f32, envelope_curve: f32,
    analog_feel: f32, humanize: f32,
) -> ExposedParams {
    ExposedParams {
        mode: ControlMode::SoundDesigner,
        character, weight, length, punch, complexity,
        transient, body, tail, saturation, noise, sub, brightness, decay,
        stereo_width, texture, pitch_offset, attack, resonance,
        spectral_tilt, distortion_type, envelope_curve,
        analog_feel, humanize,
    }
}
