use super::resynthesize::ResynthesisParams;
use super::params::{ExposedParams, ControlMode};
use super::humanize::HumanizeParams;
use super::SoundType;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InstrumentPreset {
    pub name: String,
    pub category: String,
    pub sound_type: String,
    pub params: ExposedParams,
    pub humanize: HumanizeParams,
    pub macro_mappings: Vec<MacroMapping>,
    pub description: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MacroMapping {
    pub macro_index: usize,
    pub name: String,
    pub targets: Vec<MacroTarget>,
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MacroTarget {
    pub param: String,
    pub scale: f32,
    pub offset: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MorphMacroState {
    pub morph_position: f32,
    pub macro_values: [f32; 4],
    pub random_lock: Vec<String>,
}

impl Default for MorphMacroState {
    fn default() -> Self {
        Self {
            morph_position: 0.5,
            macro_values: [0.5, 0.5, 0.5, 0.5],
            random_lock: Vec::new(),
        }
    }
}

impl InstrumentPreset {
    pub fn builtin_presets() -> Vec<Self> {
        vec![
            Self::trap_kit(),
            Self::house_kit(),
            Self::cinematic_kit(),
            Self::lo_fi_kit(),
        ]
    }

    fn trap_kit() -> Self {
        Self {
            name: "Trap Foundation".to_string(),
            category: "Kit".to_string(),
            sound_type: "kick".to_string(),
            params: ExposedParams {
                mode: ControlMode::SoundDesigner,
                character: 0.3, weight: 0.8, length: 0.4, punch: 0.8, complexity: 0.3,
                transient: 0.7, body: 0.6, tail: 0.2, saturation: 0.4, noise: 0.1,
                sub: 0.8, brightness: 0.3, decay: 0.4,
                analog_feel: 0.1, humanize: 0.0,
                ..Default::default()
            },
            humanize: HumanizeParams::default(),
            macro_mappings: vec![
                MacroMapping {
                    macro_index: 0, name: "Punch".to_string(),
                    targets: vec![
                        MacroTarget { param: "punch".to_string(), scale: 1.0, offset: 0.0 },
                        MacroTarget { param: "transient".to_string(), scale: 0.5, offset: 0.0 },
                    ],
                    min: 0.0, max: 1.0,
                },
                MacroMapping {
                    macro_index: 1, name: "Sub Weight".to_string(),
                    targets: vec![
                        MacroTarget { param: "sub".to_string(), scale: 1.0, offset: 0.0 },
                        MacroTarget { param: "weight".to_string(), scale: 0.7, offset: 0.0 },
                    ],
                    min: 0.0, max: 1.0,
                },
                MacroMapping {
                    macro_index: 2, name: "Tone".to_string(),
                    targets: vec![
                        MacroTarget { param: "character".to_string(), scale: 1.0, offset: 0.0 },
                        MacroTarget { param: "brightness".to_string(), scale: 0.5, offset: 0.0 },
                    ],
                    min: -1.0, max: 1.0,
                },
                MacroMapping {
                    macro_index: 3, name: "Decay".to_string(),
                    targets: vec![
                        MacroTarget { param: "length".to_string(), scale: 1.0, offset: 0.0 },
                        MacroTarget { param: "decay".to_string(), scale: 0.5, offset: 0.0 },
                    ],
                    min: 0.0, max: 1.0,
                },
            ],
            description: "Hard-hitting trap kick with deep sub, tight decay, and aggressive punch.".to_string(),
        }
    }

    fn house_kit() -> Self {
        Self {
            name: "House Four-on-Floor".to_string(),
            category: "Kit".to_string(),
            sound_type: "kick".to_string(),
            params: ExposedParams {
                mode: ControlMode::SoundDesigner,
                character: 0.4, weight: 0.6, length: 0.5, punch: 0.7, complexity: 0.2,
                transient: 0.6, body: 0.7, tail: 0.4, saturation: 0.3, noise: 0.0,
                sub: 0.5, brightness: 0.5, decay: 0.5,
                analog_feel: 0.2, humanize: 0.1,
                ..Default::default()
            },
            humanize: HumanizeParams {
                analog_drift: 0.05, transient_randomness: 0.05, ..Default::default()
            },
            macro_mappings: vec![
                MacroMapping {
                    macro_index: 0, name: "Weight".to_string(),
                    targets: vec![MacroTarget { param: "weight".to_string(), scale: 1.0, offset: 0.0 }],
                    min: 0.0, max: 1.0,
                },
                MacroMapping {
                    macro_index: 1, name: "Brightness".to_string(),
                    targets: vec![MacroTarget { param: "brightness".to_string(), scale: 1.0, offset: 0.0 }],
                    min: 0.0, max: 1.0,
                },
            ],
            description: "Round, warm house kick with moderate sub and clean attack.".to_string(),
        }
    }

    fn cinematic_kit() -> Self {
        Self {
            name: "Cinematic Boom".to_string(),
            category: "FX".to_string(),
            sound_type: "fx".to_string(),
            params: ExposedParams {
                mode: ControlMode::SoundDesigner,
                character: 0.2, weight: 0.9, length: 0.9, punch: 0.6, complexity: 0.5,
                transient: 0.5, body: 0.8, tail: 0.8, saturation: 0.4, noise: 0.2,
                sub: 0.9, brightness: 0.3, decay: 0.8,
                analog_feel: 0.3, humanize: 0.1,
                stereo_width: 0.6,
                ..Default::default()
            },
            humanize: HumanizeParams {
                non_static_layers: 0.1, ..Default::default()
            },
            macro_mappings: vec![
                MacroMapping {
                    macro_index: 0, name: "Impact".to_string(),
                    targets: vec![MacroTarget { param: "punch".to_string(), scale: 1.0, offset: 0.0 }],
                    min: 0.0, max: 1.0,
                },
                MacroMapping {
                    macro_index: 1, name: "Tail".to_string(),
                    targets: vec![MacroTarget { param: "length".to_string(), scale: 1.0, offset: 0.0 }],
                    min: 0.0, max: 1.0,
                },
            ],
            description: "Epic cinematic boom with massive sub, long tail, and stereo width.".to_string(),
        }
    }

    fn lo_fi_kit() -> Self {
        Self {
            name: "Lo-Fi Warmth".to_string(),
            category: "Character".to_string(),
            sound_type: "snare".to_string(),
            params: ExposedParams {
                mode: ControlMode::SoundDesigner,
                character: -0.3, weight: 0.4, length: 0.4, punch: 0.3, complexity: 0.5,
                transient: 0.3, body: 0.4, tail: 0.3, saturation: 0.6, noise: 0.4,
                sub: 0.1, brightness: 0.3, decay: 0.4,
                analog_feel: 0.7, humanize: 0.6,
                ..Default::default()
            },
            humanize: HumanizeParams::lo_fi(),
            macro_mappings: vec![
                MacroMapping {
                    macro_index: 0, name: "Warmth".to_string(),
                    targets: vec![MacroTarget { param: "character".to_string(), scale: 1.0, offset: 0.0 }],
                    min: -1.0, max: 1.0,
                },
                MacroMapping {
                    macro_index: 1, name: "Texture".to_string(),
                    targets: vec![MacroTarget { param: "complexity".to_string(), scale: 1.0, offset: 0.0 }],
                    min: 0.0, max: 1.0,
                },
            ],
            description: "Warm lo-fi snare with tape saturation, noise, and natural variation.".to_string(),
        }
    }

    pub fn apply_macro(&self, state: &MorphMacroState) -> ExposedParams {
        let mut params = self.params.clone();
        for mapping in &self.macro_mappings {
            if mapping.macro_index >= 4 { continue; }
            let val = state.macro_values[mapping.macro_index];
            for target in &mapping.targets {
                let scaled = val * target.scale + target.offset;
                match target.param.as_str() {
                    "character" => params.character = scaled.clamp(-1.0, 1.0),
                    "weight" => params.weight = scaled.clamp(0.0, 1.0),
                    "length" => params.length = scaled.clamp(0.0, 1.0),
                    "punch" => params.punch = scaled.clamp(0.0, 1.0),
                    "complexity" => params.complexity = scaled.clamp(0.0, 1.0),
                    "transient" => params.transient = scaled.clamp(0.0, 1.0),
                    "body" => params.body = scaled.clamp(0.0, 1.0),
                    "tail" => params.tail = scaled.clamp(0.0, 1.0),
                    "saturation" => params.saturation = scaled.clamp(0.0, 1.0),
                    "noise" => params.noise = scaled.clamp(0.0, 1.0),
                    "sub" => params.sub = scaled.clamp(0.0, 1.0),
                    "brightness" => params.brightness = scaled.clamp(0.0, 1.0),
                    "decay" => params.decay = scaled.clamp(0.0, 1.0),
                    _ => {}
                }
            }
        }
        params
    }
}
