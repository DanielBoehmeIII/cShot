use std::fs;
use std::path::Path;

use crate::audio;
use crate::audio::humanize::{humanize, compute_humanize_from_prompt, HumanizeParams};
use crate::db;
use crate::feedback;
use crate::prompt;
use crate::prompt_dsp;
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
/// Uses the new layer-based resynthesis engine with prompt-to-DSP control.
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

    let ctrl = prompt_dsp::parse_prompt_rich(prompt_text);
    let st = audio::SoundType::from_str(&ctrl.sound_type);
    let parsed = prompt::parse_prompt(prompt_text);

    audio::process::process_sound(&mut ref_samples, &parsed.dsp, st);

    let humanize_params = compute_humanize_from_prompt(prompt_text);
    if humanize_params.analog_drift > 0.001
        || humanize_params.instability > 0.001
        || humanize_params.humanize_transients > 0.001
    {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        humanize(&mut ref_samples, &humanize_params, seed);
    }

    audio::process::validate_audio_integrity(&ref_samples)?;

    let actual_duration_ms = ref_samples.len() as f32 / 44100.0 * 1000.0;
    audio::validate::validate_one_shot_duration(actual_duration_ms).ok();

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64;

    save_and_return(&ref_samples, prompt_text, st.as_str(), None, seed)
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

    let ctrl = prompt_dsp::parse_prompt_rich(prompt_text);
    let st = audio::SoundType::from_str(&ctrl.sound_type);
    let st_str = st.as_str().to_string();

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
        let parsed = prompt::parse_prompt(prompt_text);
        audio::process::process_sound(&mut ref_samples, &parsed.dsp, st);
        ref_samples
    } else {
        // Use the new resynthesis engine with prompt-to-DSP control
        let pitch = ctrl.pitch_hz.unwrap_or_else(|| match st {
            audio::SoundType::Kick | audio::SoundType::Bass => 60.0,
            audio::SoundType::Snare => 200.0,
            audio::SoundType::ClosedHat | audio::SoundType::OpenHat => 400.0,
            audio::SoundType::Clap => 180.0,
            audio::SoundType::Tom => 120.0,
            audio::SoundType::Perc => 300.0,
            audio::SoundType::Fx => 100.0,
            audio::SoundType::Other => 200.0,
        });
        let dur = ctrl.duration_ms.unwrap_or_else(|| match st {
            audio::SoundType::Kick => 300.0,
            audio::SoundType::Snare => 350.0,
            audio::SoundType::ClosedHat => 200.0,
            audio::SoundType::OpenHat => 600.0,
            audio::SoundType::Clap => 350.0,
            audio::SoundType::Tom => 500.0,
            audio::SoundType::Perc => 250.0,
            audio::SoundType::Bass => 600.0,
            audio::SoundType::Fx => 1000.0,
            audio::SoundType::Other => 400.0,
        });

        let mut base_params = audio::resynthesize::params_for_sound_type(st, pitch, dur);
        base_params.seed = seed as u64;
        let params = ctrl.to_resynthesis_params(&base_params);

        let mut samples = audio::resynthesize::resynthesize(&params);

        let parsed = prompt::parse_prompt(prompt_text);
        audio::process::process_sound(&mut samples, &parsed.dsp, st);

        // Apply humanization (analog drift, instability, etc.)
        let humanize_params = compute_humanize_from_prompt(prompt_text);
        if humanize_params.analog_drift > 0.001
            || humanize_params.instability > 0.001
            || humanize_params.humanize_transients > 0.001
            || humanize_params.envelope_variation > 0.001
            || humanize_params.non_static_layers > 0.001
        {
            humanize(&mut samples, &humanize_params, seed as u64);
        }

        if samples.is_empty() {
            return Err("Generation produced empty audio. Please try a different prompt.".to_string());
        }

        if let Some(bpm) = ctrl.bpm {
            let beat_dur = 60.0 / bpm;
            let beat_samples = (beat_dur * 44100.0) as usize;
            if beat_samples < samples.len() {
                samples.truncate(beat_samples);
            }
        }

        samples
    };

    audio::process::validate_audio_integrity(&samples)?;

    let actual_duration_ms = samples.len() as f32 / 44100.0 * 1000.0;
    audio::validate::validate_one_shot_duration(actual_duration_ms).ok();

    save_and_return(&samples, prompt_text, &st_str, None, seed)
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
            model: "cshot-engine".to_string(),
            seed,
            score: result.score,
            failure_labels: result.failure_labels,
        });
    }

    Ok(results)
}

/// Smart variant generation with constraints, novelty scoring, and genre awareness.
/// mode: 0=safer, 1=balanced, 2=more experimental, 3=more aggressive, 4=more polished, 5=closer to original
/// quality_threshold: minimum score to include a variant
pub fn generate_smart_variants(
    prompt_text: &str,
    sound_type_str: &str,
    count: usize,
    mode: u8,
    similarity_target: f32,
) -> Result<Vec<VariantResult>, String> {
    let ctrl = prompt_dsp::parse_prompt_rich(prompt_text);
    let st = audio::SoundType::from_str(&ctrl.sound_type);
    let base_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64;

    let pitch = ctrl.pitch_hz.unwrap_or(200.0);
    let dur = ctrl.duration_ms.unwrap_or(300.0);
    let base_params = audio::resynthesize::params_for_sound_type(st, pitch, dur);

    // Variation intensity based on mode
    let (variation_amount, similarity_weight, quality_weight) = match mode {
        0 => (0.1, 0.5, 0.4),    // safer: keep close, high quality threshold
        1 => (0.3, 0.3, 0.3),    // balanced
        2 => (0.6, 0.1, 0.2),    // experimental: more variation, less similarity
        3 => (0.4, 0.2, 0.25),   // more aggressive: moderate variation, lean aggressive
        4 => (0.2, 0.4, 0.45),   // more polished: cleaner, closer to original
        5 => (0.12, 0.6, 0.3),   // closer to original: minimal variation
        _ => (0.3, 0.3, 0.3),
    };

    let variant_intents: &[(&str, fn(&mut audio::resynthesize::ResynthesisParams))] = &[
        ("brighter", |p| { p.brightness = (p.brightness + 0.2).min(1.0); }),
        ("darker", |p| { p.brightness = (p.brightness - 0.2).max(0.0); p.saturation_drive = (p.saturation_drive + 0.1).max(1.0); }),
        ("punchier", |p| { p.click_amount = (p.click_amount + 0.2).min(1.0); p.attack_ms = (p.attack_ms * 0.7).max(0.3); p.saturation_drive = (p.saturation_drive + 0.15).max(1.0); }),
        ("softer", |p| { p.click_amount = (p.click_amount * 0.3).max(0.0); p.attack_ms = (p.attack_ms + 3.0).min(30.0); p.saturation_drive = (p.saturation_drive - 0.2).max(1.0); }),
        ("shorter", |p| { p.duration_ms *= 0.5; p.decay_ms *= 0.5; }),
        ("longer", |p| { p.duration_ms *= 1.5; p.tail_ms = (p.tail_ms + 50.0).min(2000.0); }),
        ("distorted", |p| { p.saturation_drive = (p.saturation_drive + 0.6).min(5.0); }),
        ("cleaner", |p| { p.saturation_drive = 1.0; p.noise_amount = 0.0; p.click_amount = (p.click_amount * 0.5).max(0.0); }),
        ("subbier", |p| { p.sub_gain = (p.sub_gain + 0.2).min(1.0); }),
        ("airier", |p| { p.brightness = (p.brightness + 0.15).min(1.0); p.noise_amount = (p.noise_amount + 0.1).min(0.5); }),
        ("noisier", |p| { p.noise_amount = (p.noise_amount + 0.2).min(1.0); }),
        ("tighter", |p| { p.decay_ms *= 0.5; p.tail_ms = (p.tail_ms * 0.3).max(0.0); }),
        ("fattier", |p| { p.body_gain = (p.body_gain + 0.15).min(1.0); p.sub_gain = (p.sub_gain + 0.1).min(1.0); }),
        ("metallic", |p| { p.brightness = (p.brightness + 0.15).min(1.0); p.pitch_hz *= 1.3; p.saturation_drive = (p.saturation_drive + 0.15).min(3.0); }),
        ("thinner", |p| { p.body_gain *= 0.5; p.sub_gain *= 0.3; }),
        ("warmer", |p| { p.brightness = (p.brightness - 0.15).max(0.0); p.saturation_drive = (p.saturation_drive + 0.1).min(3.0); }),
        ("modern", |p| { p.decay_ms *= 0.6; p.tail_ms = (p.tail_ms * 0.4).max(0.0); p.click_amount = (p.click_amount + 0.2).min(1.0); p.sub_gain = (p.sub_gain + 0.15).min(1.0); p.brightness = (p.brightness + 0.1).min(1.0); p.noise_amount = (p.noise_amount * 0.5).max(0.0); }),
        ("aggressive", |p| { p.click_amount = (p.click_amount + 0.3).min(1.0); p.attack_ms = (p.attack_ms * 0.5).max(0.3); p.saturation_drive = (p.saturation_drive + 0.4).min(5.0); p.body_gain = (p.body_gain + 0.1).min(1.0); }),
        ("polished", |p| { p.saturation_drive = 1.2; p.noise_amount = (p.noise_amount * 0.3).max(0.0); p.brightness = (p.brightness + 0.1).min(1.0); p.click_amount = (p.click_amount * 0.8).min(1.0); }),
    ];

    let mut results = Vec::new();
    let actual_count = count.min(variant_intents.len());

    for i in 0..actual_count {
        let seed = base_seed.wrapping_add(i as i64);
        let (name, apply_fn) = &variant_intents[i];
        let mut variant_param = base_params.clone().with_seed(seed as u64).randomize(variation_amount);
        
        if mode == 3 {
            apply_fn(&mut variant_param);
            variant_param.saturation_drive = (variant_param.saturation_drive + 0.15).min(5.0);
            variant_param.click_amount = (variant_param.click_amount + 0.1).min(1.0);
        }
        
        apply_fn(&mut variant_param);
        
        let mut samples = audio::resynthesize::resynthesize(&variant_param);

        if samples.is_empty() {
            continue;
        }

        let variant_humanize = match *name {
            "warmer" | "fattier" => HumanizeParams::warm().scaled(0.3),
            "noisier" | "airier" => HumanizeParams { instability: 0.04, non_static_layers: 0.05, ..Default::default() },
            "distorted" | "aggressive" => HumanizeParams { saturation_randomness: 0.12, ..Default::default() },
            "metallic" => HumanizeParams { instability: 0.03, ..Default::default() },
            "polished" | "modern" => HumanizeParams::default(),
            _ => HumanizeParams::default(),
        };
        if variant_humanize.analog_drift > 0.001 || variant_humanize.instability > 0.001 {
            humanize(&mut samples, &variant_humanize, seed as u64);
        }

        let quality = crate::quality::compute_quality(&samples, st, name, true);
        if quality.is_silent || quality.clipping_percent > 50.0 {
            continue;
        }

        let q_score = quality.spectral_quality * 0.2 + quality.transient_quality * 0.25
            + quality.dynamic_range * 0.15 + quality.punch_quality * 0.2
            + (1.0 - quality.clipping_percent / 100.0) * 0.2;
            
        if mode == 0 && q_score < 0.4 { continue; }
        if mode == 4 && q_score < 0.5 { continue; }

        let result = save_and_return(
            &samples,
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
            model: "cshot-engine".to_string(),
            seed,
            score: result.score,
            failure_labels: result.failure_labels,
        });
    }

    Ok(results)
}

/// Legacy wrapper for backwards compatibility
pub fn generate_resynthesis_variants(
    prompt_text: &str,
    sound_type_str: &str,
    count: usize,
) -> Result<Vec<VariantResult>, String> {
    generate_smart_variants(prompt_text, sound_type_str, count, 1, 0.5)
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
                model: "cshot-engine".to_string(),
                seed: seed as i64,
            });
        }
    }

    crate::integrity::store_hash_for_new_sound(&id, samples);

    // Record in recents store for fast workflow
    let mut recents = crate::audio::session::RecentsStore::load();
    recents.record_sound(&id);
    if !prompt_text.is_empty() && variant_name.is_none() {
        recents.record_prompt(prompt_text, sound_type_str);
    }

    Ok(SoundResult {
        id,
        waveform,
        sound_type: sound_type_str.to_string(),
        tags,
        duration_ms,
        prompt: prompt_text.to_string(),
        variant_name: variant_name.map(|s| s.to_string()),
        source: "generated".to_string(),
        model: "cshot-engine".to_string(),
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
