use super::humanize::HumanizeParams;
use super::resynthesize::ResynthesisParams;
use super::params::{ExposedParams, ControlMode};
use super::transform::TransformParams;
use super::SoundType;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CreativeIntentProfile {
    pub energy: f32,
    pub aggression: f32,
    pub polish: f32,
    pub realism: f32,
    pub experimentalism: f32,
    pub analog_feel: f32,
    pub cinematic_scale: f32,
    pub density: f32,
    pub impact: f32,
}

impl Default for CreativeIntentProfile {
    fn default() -> Self {
        Self {
            energy: 0.5,
            aggression: 0.3,
            polish: 0.5,
            realism: 0.3,
            experimentalism: 0.2,
            analog_feel: 0.2,
            cinematic_scale: 0.2,
            density: 0.5,
            impact: 0.5,
        }
    }
}

impl CreativeIntentProfile {
    pub fn preset(name: &str) -> Self {
        match name {
            "punchy_drum" => Self {
                energy: 0.7, aggression: 0.6, polish: 0.4,
                realism: 0.3, experimentalism: 0.1, analog_feel: 0.2,
                cinematic_scale: 0.1, density: 0.5, impact: 0.9,
            },
            "cinematic_boom" => Self {
                energy: 0.8, aggression: 0.4, polish: 0.8,
                realism: 0.6, experimentalism: 0.3, analog_feel: 0.4,
                cinematic_scale: 0.9, density: 0.7, impact: 0.8,
            },
            "lo_fi_warm" => Self {
                energy: 0.3, aggression: 0.1, polish: 0.2,
                realism: 0.8, experimentalism: 0.2, analog_feel: 0.8,
                cinematic_scale: 0.1, density: 0.4, impact: 0.3,
            },
            "aggressive_dubstep" => Self {
                energy: 0.9, aggression: 0.9, polish: 0.3,
                realism: 0.1, experimentalism: 0.7, analog_feel: 0.3,
                cinematic_scale: 0.5, density: 0.8, impact: 0.9,
            },
            "clean_precision" => Self {
                energy: 0.5, aggression: 0.2, polish: 0.9,
                realism: 0.3, experimentalism: 0.1, analog_feel: 0.1,
                cinematic_scale: 0.1, density: 0.5, impact: 0.6,
            },
            "ambient_texture" => Self {
                energy: 0.2, aggression: 0.0, polish: 0.6,
                realism: 0.7, experimentalism: 0.5, analog_feel: 0.5,
                cinematic_scale: 0.7, density: 0.3, impact: 0.1,
            },
            "hard_trap" => Self {
                energy: 0.8, aggression: 0.7, polish: 0.5,
                realism: 0.3, experimentalism: 0.2, analog_feel: 0.2,
                cinematic_scale: 0.3, density: 0.7, impact: 0.9,
            },
            "experimental_glitch" => Self {
                energy: 0.6, aggression: 0.5, polish: 0.1,
                realism: 0.1, experimentalism: 0.9, analog_feel: 0.3,
                cinematic_scale: 0.2, density: 0.6, impact: 0.5,
            },
            "bass_massive" => Self {
                energy: 0.8, aggression: 0.5, polish: 0.6,
                realism: 0.2, experimentalism: 0.1, analog_feel: 0.3,
                cinematic_scale: 0.4, density: 0.9, impact: 0.7,
            },
            "folk_acoustic" => Self {
                energy: 0.3, aggression: 0.0, polish: 0.3,
                realism: 0.9, experimentalism: 0.1, analog_feel: 0.7,
                cinematic_scale: 0.1, density: 0.3, impact: 0.2,
            },
            "cyberpunk" => Self {
                energy: 0.9, aggression: 0.8, polish: 0.5,
                realism: 0.1, experimentalism: 0.8, analog_feel: 0.1,
                cinematic_scale: 0.6, density: 0.8, impact: 0.8,
            },
            "orchestral_epic" => Self {
                energy: 0.7, aggression: 0.3, polish: 0.9,
                realism: 0.7, experimentalism: 0.0, analog_feel: 0.5,
                cinematic_scale: 1.0, density: 0.8, impact: 0.7,
            },
            "minimal_techno" => Self {
                energy: 0.6, aggression: 0.3, polish: 0.8,
                realism: 0.4, experimentalism: 0.2, analog_feel: 0.5,
                cinematic_scale: 0.2, density: 0.4, impact: 0.6,
            },
            "retro_video_game" => Self {
                energy: 0.5, aggression: 0.2, polish: 0.3,
                realism: 0.1, experimentalism: 0.5, analog_feel: 0.6,
                cinematic_scale: 0.1, density: 0.4, impact: 0.4,
            },
            "jazz_brush" => Self {
                energy: 0.2, aggression: 0.0, polish: 0.4,
                realism: 0.9, experimentalism: 0.1, analog_feel: 0.9,
                cinematic_scale: 0.1, density: 0.2, impact: 0.1,
            },
            _ => Self::default(),
        }
    }

    pub fn to_resynthesis_params(&self, base: &ResynthesisParams) -> ResynthesisParams {
        let mut p = base.clone();

        let energy_mod = self.energy * 2.0 - 1.0;
        let aggression_mod = self.aggression;
        let polish_mod = self.polish;
        let density_mod = self.density * 2.0 - 1.0;
        let impact_mod = self.impact;
        let cinematic_mod = self.cinematic_scale;

        let brightness_from_energy = energy_mod * 0.3;
        let brightness_from_polish = (polish_mod - 0.5) * 0.3;
        let brightness_from_aggression = aggression_mod * 0.2;
        let brightness_tgt = 0.5 + brightness_from_energy + brightness_from_polish + brightness_from_aggression;
        p.brightness = brightness_tgt.clamp(0.0, 1.0);

        let sat_from_energy = energy_mod.max(0.0) * 0.5;
        let sat_from_aggression = aggression_mod * 0.8;
        let sat_from_polish = (polish_mod - 0.5) * (-0.3);
        let sat_from_density = density_mod.max(0.0) * 0.3;
        p.saturation_drive = (base.saturation_drive + sat_from_energy + sat_from_aggression + sat_from_polish + sat_from_density).max(1.0);

        let click_from_impact = impact_mod * 0.5;
        let click_from_aggression = aggression_mod * 0.2;
        let click_from_energy = energy_mod.max(0.0) * 0.15;
        p.click_amount = (base.click_amount + click_from_impact + click_from_aggression + click_from_energy).clamp(0.0, 1.0);

        let att_from_impact = (1.0 - impact_mod) * 3.0;
        let att_from_aggression = (1.0 - aggression_mod) * 2.0;
        let att_from_polish = (1.0 - polish_mod) * 1.0;
        p.attack_ms = (base.attack_ms + att_from_impact + att_from_aggression + att_from_polish).max(0.3);

        let body_from_density = density_mod * 0.3;
        let body_from_impact = impact_mod * 0.15;
        let body_from_energy = energy_mod.max(0.0) * 0.2;
        p.body_gain = (base.body_gain + body_from_density + body_from_impact + body_from_energy).clamp(0.0, 1.0);

        let sub_from_cinematic = cinematic_mod * 0.4;
        let sub_from_density = density_mod.max(0.0) * 0.2;
        let sub_from_impact = impact_mod * 0.2;
        p.sub_gain = (base.sub_gain + sub_from_cinematic + sub_from_density + sub_from_impact).clamp(0.0, 1.0);

        let noise_from_experimental = self.experimentalism * 0.3;
        let noise_from_realism = (1.0 - self.realism) * 0.15;
        let noise_from_aggression = aggression_mod * 0.15;
        let noise_clean = (1.0 - polish_mod) * 0.15;
        p.noise_amount = (base.noise_amount + noise_from_experimental + noise_from_realism + noise_from_aggression + noise_clean).clamp(0.0, 1.0);

        let dur_from_cinematic = cinematic_mod * 0.6;
        let dur_from_energy = energy_mod * 0.2;
        let dur_from_density = density_mod * 0.2;
        let dur_scale = 1.0 + dur_from_cinematic + dur_from_energy.max(0.0) * 0.15 + dur_from_density.max(0.0) * 0.1;
        p.duration_ms = base.duration_ms * dur_scale;

        let tail_from_cinematic = cinematic_mod * 0.6;
        let tail_from_experimental = self.experimentalism * 0.3;
        let tail_clean = (1.0 - polish_mod) * 0.3;
        p.tail_ms = base.tail_ms * (1.0 + tail_from_cinematic + tail_from_experimental + tail_clean);

        p
    }

    pub fn to_humanize_params(&self) -> HumanizeParams {
        let af = self.analog_feel;
        let realism = self.realism;
        let experiment = self.experimentalism;
        let polish = self.polish;

        HumanizeParams {
            analog_drift: af * 0.3 + realism * 0.1,
            instability: af * 0.15 + experiment * 0.1,
            transient_randomness: realism * 0.15 + experiment * 0.2 - polish * 0.1,
            envelope_variation: realism * 0.12 + experiment * 0.1 - polish * 0.08,
            saturation_randomness: af * 0.2 + experiment * 0.1 - polish * 0.1,
            non_static_layers: (af + realism + experiment) * 0.33 * 0.12,
            phase_variation: af * 0.12 + experiment * 0.1,
            humanize_transients: realism * 0.2 + experiment * 0.1 - polish * 0.1,
        }
    }

    pub fn to_transform_params(&self) -> TransformParams {
        let mut t = TransformParams::default();

        let tilt = (self.energy * 2.0 - 1.0) * 0.3
            + self.aggression * 0.15
            - (1.0 - self.polish) * 0.1;
        if tilt.abs() > 0.05 {
            t.brightness_tilt = Some(tilt.clamp(-1.0, 1.0));
        }

        let sat = self.aggression * 0.6 + self.impact * 0.3 + self.energy * 0.2;
        if sat > 0.1 {
            t.saturation_drive = Some(1.0 + sat * 2.0);
        }

        let transient = self.impact * 0.6 + self.aggression * 0.3 + self.energy * 0.15;
        if transient > 0.1 {
            t.transient_boost_db = Some(transient * 6.0);
        }

        let noise = self.experimentalism * 0.2 + (1.0 - self.polish) * 0.1;
        if noise > 0.05 {
            t.noise_add = Some(noise * 0.3);
        }

        let sub = self.cinematic_scale * 0.3 + self.density * 0.15;
        if sub > 0.05 {
            t.sub_add = Some(sub * 0.3);
        }

        t
    }

    pub fn to_exposed_params(&self) -> ExposedParams {
        ExposedParams {
            mode: ControlMode::Simple,
            character: ((self.energy * 2.0 - 1.0) * 0.5 + self.aggression * 0.3 - self.analog_feel * 0.2).clamp(-1.0, 1.0),
            weight: (self.density * 0.5 + self.impact * 0.3 + self.cinematic_scale * 0.2).clamp(0.0, 1.0),
            length: (self.cinematic_scale * 0.5 + 0.5).clamp(0.0, 1.0),
            punch: (self.impact * 0.6 + self.aggression * 0.3 + self.energy * 0.1).clamp(0.0, 1.0),
            complexity: (self.experimentalism * 0.5 + (1.0 - self.polish) * 0.3 + self.density * 0.2).clamp(0.0, 1.0),
            analog_feel: self.analog_feel,
            humanize: self.realism,
            ..Default::default()
        }
    }

    pub fn from_descriptors(descriptors: &[String]) -> Self {
        let mut profile = Self::default();
        let mut count: f32 = 0.0;

        for d in descriptors {
            let lower = d.to_lowercase();
            match lower.as_str() {
                "energetic" | "high_energy" | "lively" => {
                    profile.energy = (profile.energy + 0.3).min(1.0);
                    count += 1.0;
                }
                "low_energy" | "calm" | "subdued" => {
                    profile.energy = (profile.energy - 0.3).max(0.0);
                    count += 1.0;
                }
                "aggressive" | "harsh" | "intense" => {
                    profile.aggression = (profile.aggression + 0.35).min(1.0);
                    count += 1.0;
                }
                "gentle" | "soft" | "smooth" => {
                    profile.aggression = (profile.aggression - 0.3).max(0.0);
                    count += 1.0;
                }
                "polished" | "clean" | "refined" => {
                    profile.polish = (profile.polish + 0.3).min(1.0);
                    count += 1.0;
                }
                "raw" | "rough" | "unpolished" => {
                    profile.polish = (profile.polish - 0.3).max(0.0);
                    count += 1.0;
                }
                "realistic" | "natural" | "organic" => {
                    profile.realism = (profile.realism + 0.35).min(1.0);
                    count += 1.0;
                }
                "synthetic" | "digital" | "artificial" => {
                    profile.realism = (profile.realism - 0.3).max(0.0);
                    count += 1.0;
                }
                "experimental" | "weird" | "glitch" => {
                    profile.experimentalism = (profile.experimentalism + 0.4).min(1.0);
                    count += 1.0;
                }
                "conventional" | "standard" | "normal" => {
                    profile.experimentalism = (profile.experimentalism - 0.3).max(0.0);
                    count += 1.0;
                }
                "analog" | "vintage" | "warm" => {
                    profile.analog_feel = (profile.analog_feel + 0.3).min(1.0);
                    count += 1.0;
                }
                "modern" | "sterile" => {
                    profile.analog_feel = (profile.analog_feel - 0.25).max(0.0);
                    count += 1.0;
                }
                "cinematic" | "epic" | "grand" => {
                    profile.cinematic_scale = (profile.cinematic_scale + 0.35).min(1.0);
                    count += 1.0;
                }
                "intimate" | "small" | "contained" => {
                    profile.cinematic_scale = (profile.cinematic_scale - 0.3).max(0.0);
                    count += 1.0;
                }
                "dense" | "full" | "thick" => {
                    profile.density = (profile.density + 0.3).min(1.0);
                    count += 1.0;
                }
                "sparse" | "thin" | "minimal" => {
                    profile.density = (profile.density - 0.3).max(0.0);
                    count += 1.0;
                }
                "impactful" | "punchy" | "heavy" => {
                    profile.impact = (profile.impact + 0.3).min(1.0);
                    count += 1.0;
                }
                "light" | "delicate" | "subtle" => {
                    profile.impact = (profile.impact - 0.3).max(0.0);
                    count += 1.0;
                }
                _ => {}
            }
        }

        if count > 0.0 {
            let _avg = count.max(1.0);
        }

        profile
    }

    pub fn blend(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            energy: a.energy * (1.0 - t) + b.energy * t,
            aggression: a.aggression * (1.0 - t) + b.aggression * t,
            polish: a.polish * (1.0 - t) + b.polish * t,
            realism: a.realism * (1.0 - t) + b.realism * t,
            experimentalism: a.experimentalism * (1.0 - t) + b.experimentalism * t,
            analog_feel: a.analog_feel * (1.0 - t) + b.analog_feel * t,
            cinematic_scale: a.cinematic_scale * (1.0 - t) + b.cinematic_scale * t,
            density: a.density * (1.0 - t) + b.density * t,
            impact: a.impact * (1.0 - t) + b.impact * t,
        }
    }

    pub fn to_coordinated_params(
        &self,
        sound_type: SoundType,
        pitch_hz: f32,
        duration_ms: f32,
    ) -> CoordinatedParams {
        let base_resynth = super::resynthesize::params_for_sound_type(sound_type, pitch_hz, duration_ms);
        let resynth = self.to_resynthesis_params(&base_resynth);
        let humanize = self.to_humanize_params();
        let transform = self.to_transform_params();
        let exposed = self.to_exposed_params();

        CoordinatedParams {
            resynthesis: resynth,
            humanize,
            transform,
            exposed,
            profile: self.clone(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CoordinatedParams {
    pub resynthesis: ResynthesisParams,
    pub humanize: HumanizeParams,
    pub transform: TransformParams,
    pub exposed: ExposedParams,
    pub profile: CreativeIntentProfile,
}

pub fn generate_with_intent(
    sound_type: SoundType,
    pitch_hz: f32,
    duration_ms: f32,
    profile: &CreativeIntentProfile,
) -> CoordinatedParams {
    profile.to_coordinated_params(sound_type, pitch_hz, duration_ms)
}

pub fn intent_label(value: f32) -> &'static str {
    if value < 0.2 { "very low" }
    else if value < 0.4 { "low" }
    else if value < 0.6 { "moderate" }
    else if value < 0.8 { "high" }
    else { "very high" }
}
