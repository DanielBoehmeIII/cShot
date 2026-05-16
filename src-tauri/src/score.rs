use crate::audio::SoundType;
use crate::quality::QualityMetadata;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct SoundScore {
    pub overall: u32,
    pub no_clipping: bool,
    pub reasonable_duration: bool,
    pub non_silent: bool,
    pub clean_save: bool,
    pub user_signal: i32,
    pub failure_labels: Vec<String>,
}

pub fn compute_score(
    quality: &QualityMetadata,
    _sound_type: SoundType,
    has_user_feedback: Option<bool>,
    usable: Option<bool>,
) -> SoundScore {
    let mut score = 50.0f32;
    let mut user_signal = 0i32;

    let no_clipping = !quality.clipping_detected;
    if no_clipping {
        score += 15.0;
    } else {
        score -= 15.0;
    }

    let reasonable_duration = quality.duration_appropriate;
    if reasonable_duration {
        score += 10.0;
    } else {
        score -= 10.0;
    }

    let non_silent = !quality.is_silent;
    if non_silent {
        score += 10.0;
    } else {
        score -= 20.0;
    }

    if quality.is_too_quiet {
        score -= 5.0;
    } else {
        score += 5.0;
    }

    score += 5.0;

    if let Some(thumbs_up) = has_user_feedback {
        if thumbs_up {
            score += 10.0;
            user_signal += 10;
        } else {
            score -= 10.0;
            user_signal -= 10;
        }
    }

    if let Some(is_usable) = usable {
        if is_usable {
            score += 5.0;
            user_signal += 5;
        } else {
            score -= 10.0;
            user_signal -= 10;
        }
    }

    let failure_labels = compute_labels(quality);
    let overall = score.clamp(0.0, 100.0) as u32;

    SoundScore {
        overall,
        no_clipping,
        reasonable_duration,
        non_silent,
        clean_save: true,
        user_signal,
        failure_labels,
    }
}

fn compute_labels(quality: &QualityMetadata) -> Vec<String> {
    let mut labels = Vec::new();
    if quality.clipping_detected {
        labels.push("clipped".to_string());
    }
    if !quality.duration_appropriate && quality.duration_ms > 0.0 {
        if quality.duration_ms > 2000.0 {
            labels.push("too long".to_string());
        } else {
            labels.push("duration".to_string());
        }
    }
    if quality.is_too_quiet {
        labels.push("too quiet".to_string());
    }
    if quality.is_silent {
        labels.push("silent".to_string());
    }
    labels
}
