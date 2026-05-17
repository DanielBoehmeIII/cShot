pub mod analyze;
pub mod analysis_cache;
pub mod audio_intelligence;
pub mod creative_intent;
pub mod dsp;
pub mod evolution;
pub mod humanize;
pub mod hybrid;
pub mod instrument;
pub mod identity;
pub mod io;
pub mod midi;
pub mod mock;
pub mod morph;
pub mod mutation;

pub mod packs;
pub mod params;
pub mod process;
pub mod recreate;
pub mod resynthesize;
pub mod sculpt;
pub mod session;
pub mod sound_designer;
pub mod spectral_edit;
pub mod spectral_intelligence;
pub mod stress_test;
pub mod synthesize;
pub mod taste;
pub mod transform;
pub mod validate;
pub mod waveform;
pub mod workflow;

#[allow(unused_imports)]
pub use analyze::*;
#[allow(unused_imports)]
pub use analysis_cache::*;
#[allow(unused_imports)]
pub use humanize::*;
#[allow(unused_imports)]
pub use morph::*;
#[allow(unused_imports)]
pub use transform::*;
#[allow(unused_imports)]
pub use dsp::*;
#[allow(unused_imports)]
pub use io::*;
#[allow(unused_imports)]
pub use mock::*;
#[allow(unused_imports)]
pub use process::*;
#[allow(unused_imports)]
pub use recreate::*;
#[allow(unused_imports)]
pub use resynthesize::*;
#[allow(unused_imports)]
pub use spectral_intelligence::*;
#[allow(unused_imports)]
pub use synthesize::*;
#[allow(unused_imports)]
pub use validate::*;
#[allow(unused_imports)]
pub use waveform::*;

pub const SAMPLE_RATE: u32 = 44100;

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum SoundType {
    Kick,
    Snare,
    ClosedHat,
    OpenHat,
    Clap,
    Tom,
    Perc,
    Bass,
    Fx,
    Other,
}

impl SoundType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SoundType::Kick => "kick",
            SoundType::Snare => "snare",
            SoundType::ClosedHat => "closed_hat",
            SoundType::OpenHat => "open_hat",
            SoundType::Clap => "clap",
            SoundType::Tom => "tom",
            SoundType::Perc => "perc",
            SoundType::Bass => "bass",
            SoundType::Fx => "fx",
            SoundType::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "kick" => SoundType::Kick,
            "snare" => SoundType::Snare,
            "closed_hat" => SoundType::ClosedHat,
            "open_hat" => SoundType::OpenHat,
            "clap" => SoundType::Clap,
            "tom" => SoundType::Tom,
            "perc" => SoundType::Perc,
            "bass" => SoundType::Bass,
            "fx" => SoundType::Fx,
            _ => SoundType::Other,
        }
    }
}

fn noise(phase: f32) -> f32 {
    ((phase * 127.1).sin() * 43758.5453).fract() * 2.0 - 1.0
}

#[derive(Clone)]
pub struct DspParams {
    pub low_pass: bool,
    pub high_pass: bool,
    pub punch: bool,
    pub bright: bool,
    pub dark: bool,
    pub bpm: Option<f32>,
    pub pitch_shift: Option<f32>,
    pub gain: f32,
    pub noise_amt: f32,
    pub decay_factor: f32,
}

impl Default for DspParams {
    fn default() -> Self {
        Self {
            low_pass: false,
            high_pass: false,
            punch: false,
            bright: false,
            dark: false,
            bpm: None,
            pitch_shift: None,
            gain: 1.0,
            noise_amt: 0.0,
            decay_factor: 1.0,
        }
    }
}
