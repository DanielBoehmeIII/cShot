use crate::audio::{DspParams, SoundType};

pub struct ParsedPrompt {
    pub sound_type: SoundType,
    pub dsp: DspParams,
}

pub fn parse_prompt(text: &str) -> ParsedPrompt {
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let sound_type = classify_type(&words);
    let dsp = build_dsp(&words, sound_type);
    ParsedPrompt { sound_type, dsp }
}

fn classify_type(words: &[&str]) -> SoundType {
    let mut scores: Vec<(SoundType, i32)> = vec![
        (SoundType::Kick, 0),
        (SoundType::Snare, 0),
        (SoundType::ClosedHat, 0),
        (SoundType::OpenHat, 0),
        (SoundType::Clap, 0),
        (SoundType::Tom, 0),
        (SoundType::Perc, 0),
        (SoundType::Bass, 0),
        (SoundType::Fx, 0),
    ];

    let keyword_map: Vec<(&str, &[(SoundType, i32)])> = vec![
        ("kick", &[(SoundType::Kick, 10), (SoundType::Bass, 1)][..]),
        ("kicker", &[(SoundType::Kick, 8)][..]),
        ("808", &[(SoundType::Kick, 5), (SoundType::Bass, 3)][..]),
        ("snare", &[(SoundType::Snare, 10)][..]),
        ("clap", &[(SoundType::Clap, 10)][..]),
        ("hat", &[(SoundType::ClosedHat, 5), (SoundType::OpenHat, 5)][..]),
        ("hi-hat", &[(SoundType::ClosedHat, 5), (SoundType::OpenHat, 5)][..]),
        ("closed", &[(SoundType::ClosedHat, 8)][..]),
        ("open", &[(SoundType::OpenHat, 8)][..]),
        ("hihat", &[(SoundType::ClosedHat, 5), (SoundType::OpenHat, 5)][..]),
        ("cymbal", &[(SoundType::OpenHat, 4)][..]),
        ("ride", &[(SoundType::OpenHat, 3)][..]),
        ("crash", &[(SoundType::Fx, 5), (SoundType::OpenHat, 3)][..]),
        ("tom", &[(SoundType::Tom, 10)][..]),
        ("perc", &[(SoundType::Perc, 8)][..]),
        ("percussion", &[(SoundType::Perc, 6)][..]),
        ("bass", &[(SoundType::Bass, 8), (SoundType::Kick, 2)][..]),
        ("sub", &[(SoundType::Bass, 6)][..]),
        ("fx", &[(SoundType::Fx, 8)][..]),
        ("effect", &[(SoundType::Fx, 6)][..]),
        ("riser", &[(SoundType::Fx, 8)][..]),
        ("impact", &[(SoundType::Fx, 7), (SoundType::Kick, 3)][..]),
        ("noise", &[(SoundType::Fx, 4), (SoundType::Perc, 3)][..]),
        ("rim", &[(SoundType::Perc, 6)][..]),
        ("rimshot", &[(SoundType::Perc, 8)][..]),
        ("click", &[(SoundType::Perc, 5)][..]),
        ("stamp", &[(SoundType::Perc, 4)][..]),
        ("shaker", &[(SoundType::Perc, 5)][..]),
        ("whoosh", &[(SoundType::Fx, 9)][..]),
        ("sweep", &[(SoundType::Fx, 7)][..]),
        ("swish", &[(SoundType::Fx, 6)][..]),
        ("sword", &[(SoundType::Fx, 6)][..]),
        ("footstep", &[(SoundType::Perc, 5), (SoundType::Kick, 2)][..]),
        ("ui", &[(SoundType::Perc, 4)][..]),
        ("pad", &[(SoundType::Fx, 5)][..]),
        ("atmosphere", &[(SoundType::Fx, 6)][..]),
        ("boom", &[(SoundType::Kick, 4), (SoundType::Bass, 3)][..]),
        ("hit", &[(SoundType::Perc, 4), (SoundType::Kick, 2)][..]),
        ("thud", &[(SoundType::Kick, 5)][..]),
        ("wooden", &[(SoundType::Perc, 5)][..]),
        ("metallic", &[(SoundType::Perc, 4)][..]),
    ];

    for word in words {
        if let Some(mappings) = keyword_map.iter().find(|(k, _)| *k == *word) {
            for (sound_type, score) in mappings.1 {
                if let Some(entry) = scores.iter_mut().find(|(st, _)| st == sound_type) {
                    entry.1 += score;
                }
            }
        }
    }

    scores
        .into_iter()
        .max_by_key(|(_, score)| *score)
        .map(|(st, _)| st)
        .unwrap_or(SoundType::Kick)
}

fn build_dsp(words: &[&str], _sound_type: SoundType) -> DspParams {
    let mut dsp = DspParams::default();

    for word in words {
        match *word {
            "punchy" | "punch" | "crack" | "snap" => {
                dsp.punch = true;
                dsp.high_pass = true;
            }
            "bright" | "crisp" | "shiny" | "glossy" | "shimmering" => dsp.bright = true,
            "dark" | "dull" | "muffled" | "warm" => dsp.dark = true,
            "soft" | "gentle" => {
                dsp.gain = 0.6;
                dsp.high_pass = true;
            }
            "hard" | "aggressive" | "heavy" => dsp.gain = 1.5,
            "distorted" | "crunchy" | "gritty" | "industrial" | "overdriven" => {
                dsp.gain = 2.0;
                dsp.noise_amt = 0.2;
            }
            "short" => dsp.decay_factor = 0.5,
            "long" | "ring" | "epic" | "massive" => dsp.decay_factor = 2.0,
            "low" | "deep" | "subby" => {
                dsp.low_pass = true;
            }
            "high" | "thin" | "airy" => {
                dsp.high_pass = true;
            }
            "fast" => dsp.decay_factor = 0.3,
            "layered" => dsp.gain = 1.3,
            "resonant" => dsp.decay_factor = 1.5,
            "loud" => dsp.gain = 1.8,
            _ => {}
        }

        if let Ok(bpm) = word.parse::<f32>() {
            if bpm >= 60.0 && bpm <= 200.0 {
                dsp.bpm = Some(bpm);
            }
        }
    }

    dsp
}
