pub mod analyze;
pub mod dsp;
pub mod io;
pub mod mock;
pub mod process;
pub mod synthesize;
pub mod validate;
pub mod waveform;

#[allow(unused_imports)]
pub use analyze::*;
#[allow(unused_imports)]
pub use dsp::*;
#[allow(unused_imports)]
pub use io::*;
#[allow(unused_imports)]
pub use mock::*;
#[allow(unused_imports)]
pub use process::*;
#[allow(unused_imports)]
pub use synthesize::*;
#[allow(unused_imports)]
pub use validate::*;
#[allow(unused_imports)]
pub use waveform::*;

pub const SAMPLE_RATE: u32 = 44100;

#[derive(Clone, Copy, PartialEq, Debug)]
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
