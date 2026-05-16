use std::fs;
use std::path::Path;

use crate::audio;
use crate::db;
use crate::feedback;
use crate::prompt;
use crate::quality;
use crate::score;

#[derive(Clone, serde::Serialize)]
pub struct SoundResult {
    pub id: String,
    pub waveform: Vec<f32>,
    pub sound_type: String,
    pub tags: Vec<String>,
    pub duration_ms: f32,
    pub prompt: String,
    pub variant_name: Option<String>,
    pub source: String,
    pub model: String,
    pub rms: f32,
    pub peak: f32,
    pub spectral_centroid: f32,
    pub seed: i64,
    pub score: u32,
    pub failure_labels: Vec<String>,
}

#[derive(Clone, serde::Serialize)]
pub struct VariantResult {
    pub id: String,
    pub waveform: Vec<f32>,
    pub sound_type: String,
    pub tags: Vec<String>,
    pub duration_ms: f32,
    pub prompt: String,
    pub variant_name: String,
    pub source: String,
    pub model: String,
    pub seed: i64,
    pub score: u32,
    pub failure_labels: Vec<String>,
}

/// Generate a sound from prompt text and optional reference audio samples (in-memory).
/// This is used by the provider abstraction layer.
pub fn generate_with_reference(
    prompt_text: &str,
    reference_samples: &[f32],
    reference_sample_rate: u32,
) -> Result<SoundResult, String> {
    let trimmed = prompt_text.trim();
    if trimmed.is_empty() {
        return Err("Prompt cannot be empty. Describe the sound you want.".to_string());
    }
    if trimmed.len() > 500 {
        return Err(format!(
            "Prompt is too long ({} chars). Maximum is 500 characters.",
            trimmed.len()
        ));
    }

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64;

    let parsed = prompt::parse_prompt(prompt_text);

    let mut ref_samples = reference_samples.to_vec();
    if reference_sample_rate != 44100 {
        let ratio = 44100.0 / reference_sample_rate as f32;
        let new_len = (ref_samples.len() as f32 * ratio) as usize;
        let mut resampled = Vec::with_capacity(new_len);
        for i in 0..new_len {
            let src_idx = (i as f32 / ratio) as usize;
            let s = ref_samples.get(src_idx).copied().unwrap_or(0.0);
            resampled.push(s);
        }
        ref_samples = resampled;
    }

    audio::process::process_sound(&mut ref_samples, &parsed.dsp, parsed.sound_type);
    audio::process::validate_audio_integrity(&ref_samples)?;

    let actual_duration_ms = ref_samples.len() as f32 / 44100.0 * 1000.0;
    audio::validate::validate_one_shot_duration(actual_duration_ms).ok();

    save_and_return(&ref_samples, prompt_text, parsed.sound_type.as_str(), None, seed)
}

pub fn generate(
    prompt_text: &str,
    reference_path: Option<&str>,
) -> Result<SoundResult, String> {
    let trimmed = prompt_text.trim();
    if trimmed.is_empty() {
        return Err("Prompt cannot be empty. Describe the sound you want.".to_string());
    }
    if trimmed.len() > 500 {
        return Err(format!(
            "Prompt is too long ({} chars). Maximum is 500 characters.",
            trimmed.len()
        ));
    }

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64;

    let parsed = prompt::parse_prompt(prompt_text);

    let samples = if let Some(ref_path) = reference_path {
        let path = Path::new(ref_path);
        if !path.exists() {
            return Err("Reference file not found. Please check the file path.".to_string());
        }
        let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let validation = audio::validate::validate_upload(path, file_size);
        if !validation.is_valid {
            return Err(validation.error.unwrap_or_else(|| "Invalid reference file".to_string()));
        }

        let mut ref_samples = audio::read_wav(path)?;
        audio::process::process_sound(&mut ref_samples, &parsed.dsp, parsed.sound_type);
        ref_samples
    } else {
        let duration_ms = match parsed.sound_type {
            audio::SoundType::Kick => audio::validate::MAX_ONE_SHOT_DURATION_MS.min(500.0),
            audio::SoundType::Snare => 350.0,
            audio::SoundType::ClosedHat => 200.0,
            audio::SoundType::OpenHat => 800.0,
            audio::SoundType::Clap => 400.0,
            audio::SoundType::Tom => 600.0,
            audio::SoundType::Perc => 300.0,
            audio::SoundType::Bass => audio::validate::MAX_ONE_SHOT_DURATION_MS.min(800.0),
            audio::SoundType::Fx => 1000.0,
            audio::SoundType::Other => 500.0,
        };

        let mut samples = audio::synthesize::generate_base(parsed.sound_type, duration_ms);

        if let Some(bpm) = parsed.dsp.bpm {
            let beat_dur = 60.0 / bpm;
            let beat_samples = (beat_dur * 44100.0) as usize;
            if beat_samples < samples.len() {
                samples.truncate(beat_samples);
            }
        }

        audio::process::process_sound(&mut samples, &parsed.dsp, parsed.sound_type);

        if samples.is_empty() {
            return Err("Generation produced empty audio. Please try a different prompt.".to_string());
        }

        samples
    };

    audio::process::validate_audio_integrity(&samples)?;

    let actual_duration_ms = samples.len() as f32 / 44100.0 * 1000.0;
    audio::validate::validate_one_shot_duration(actual_duration_ms).ok();

    save_and_return(&samples, prompt_text, parsed.sound_type.as_str(), None, seed)
}

pub fn generate_variants(
    prompt_text: &str,
    source_samples: &[f32],
    sound_type_str: &str,
    count: usize,
) -> Result<Vec<VariantResult>, String> {
    if source_samples.is_empty() {
        return Err("Source audio is empty, cannot generate variants".to_string());
    }

    let variant_types = [
        audio::mock::MockVariant::Trimmed,
        audio::mock::MockVariant::Repitched,
        audio::mock::MockVariant::Saturated,
        audio::mock::MockVariant::Shortened,
        audio::mock::MockVariant::TransientShaped,
        audio::mock::MockVariant::BrightVariant,
        audio::mock::MockVariant::DarkVariant,
        audio::mock::MockVariant::PunchyVariant,
        audio::mock::MockVariant::AiryVariant,
        audio::mock::MockVariant::GrittyVariant,
        audio::mock::MockVariant::SubbyVariant,
        audio::mock::MockVariant::TightVariant,
    ];

    let actual_count = count.min(variant_types.len());
    if actual_count == 0 {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    let base_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64;

    for i in 0..actual_count {
        let variant = &variant_types[i];
        let seed = base_seed.wrapping_add(i as i64);
        let variant_samples = audio::mock::apply_variant(source_samples, variant, seed as u64);

        if variant_samples.is_empty() {
            continue;
        }

        let name = audio::mock::generate_variant_name(variant);
        let result = save_and_return(
            &variant_samples,
            prompt_text,
            sound_type_str,
            Some(name),
            seed,
        )?;

        results.push(VariantResult {
            id: result.id,
            waveform: result.waveform,
            sound_type: result.sound_type,
            tags: result.tags,
            duration_ms: result.duration_ms,
            prompt: result.prompt,
            variant_name: name.to_string(),
            source: "variant".to_string(),
            model: "mock-dsp".to_string(),
            seed,
            score: result.score,
            failure_labels: result.failure_labels,
        });
    }

    Ok(results)
}

pub fn save_and_return(
    samples: &[f32],
    prompt_text: &str,
    sound_type_str: &str,
    variant_name: Option<&str>,
    seed: i64,
) -> Result<SoundResult, String> {
    let sound_dir = crate::storage::audio_dir();
    fs::create_dir_all(&sound_dir).map_err(|e| e.to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let wav_path = sound_dir.join(format!("{}.wav", id));
    audio::write_wav(&wav_path, samples, 44100)?;

    let waveform = audio::compute_waveform(samples, 80);
    let duration_ms = samples.len() as f32 / 44100.0 * 1000.0;

    let sound_type = audio::SoundType::from_str(sound_type_str);
    let tags = audio::apply_autotags(samples, &sound_type, variant_name, Some(prompt_text));

    let rms = audio::compute_rms(samples);
    let peak = audio::compute_peak(samples);
    let spectral_centroid = audio::compute_spectral_centroid(samples);

    let q = quality::compute_quality(samples, sound_type, variant_name.unwrap_or("original"), true);

    let fb_store = feedback::FeedbackStore::load();
    let fb = fb_store.get(&id);
    let user_feedback = fb.map(|f| f.thumbs_up).or(Some(false));
    let usable = fb.and_then(|f| f.usable);

    let s = score::compute_score(&q, sound_type, user_feedback, usable);

    if let Ok(db_path) = db_path() {
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            let source = if variant_name.is_some() { "variant" } else { "generated" };
            let _ = db::insert_sound(&conn, &db::SoundEntry {
                id: id.clone(),
                prompt: prompt_text.to_string(),
                sound_type: sound_type_str.to_string(),
                duration_ms,
                sample_rate: 44100,
                rms,
                peak,
                spectral_centroid,
                tags: tags_json,
                is_favorite: false,
                source: source.to_string(),
                variant_name: variant_name.map(|s| s.to_string()),
                created_at: String::new(),
                model: "mock-dsp".to_string(),
                seed: seed as i64,
            });
        }
    }

    crate::integrity::store_hash_for_new_sound(&id, samples);

    Ok(SoundResult {
        id,
        waveform,
        sound_type: sound_type_str.to_string(),
        tags,
        duration_ms,
        prompt: prompt_text.to_string(),
        variant_name: variant_name.map(|s| s.to_string()),
        source: "generated".to_string(),
        model: "mock-dsp".to_string(),
        seed,
        rms,
        peak,
        spectral_centroid,
        score: s.overall,
        failure_labels: s.failure_labels,
    })
}

pub fn db_path() -> Result<std::path::PathBuf, String> {
    Ok(crate::storage::database_path())
}
