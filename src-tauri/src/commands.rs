use std::fs;
use std::io::Write;
use std::path::PathBuf;
use crate::audio;
use crate::db::{self, SoundEntry};
use crate::favorites::SoundMetadata as FavMetadata;
use crate::feedback;
use crate::generator;
use crate::generation::provider::GenerationRequest;
use crate::quality::{self, QualityMetadata};
use crate::score::{self, SoundScore};
use crate::storage;
use crate::AppState;

use tauri::State;
use rusqlite::params;
use zip::write::FileOptions;

#[derive(Clone, serde::Serialize)]
pub struct ReferenceAnalysis {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub duration_ms: f32,
    pub sample_rate: u32,
    pub channels: u16,
    pub file_type: String,
    pub waveform: Vec<f32>,
    pub rms: f32,
    pub peak: f32,
    pub validation_message: Option<String>,
}

#[tauri::command]
pub async fn analyze_reference(path: String) -> Result<ReferenceAnalysis, String> {
    let src = PathBuf::from(&path);
    if !src.exists() {
        return Err("Reference file not found".to_string());
    }

    let file_size = fs::metadata(&src).map(|m| m.len()).unwrap_or(0);
    let validation = audio::validate::validate_upload(&src, file_size);
    if !validation.is_valid {
        return Err(validation.error.unwrap_or_else(|| "Invalid file".to_string()));
    }

    let ext = src.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "wav".to_string());
    let filename = src.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let samples: Vec<f32> = audio::read_wav(&src)?;
    let duration_ms = samples.len() as f32 / 44100.0 * 1000.0;

    let waveform = audio::compute_waveform(&samples, 80);
    let rms = audio::compute_rms(&samples);
    let peak = audio::compute_peak(&samples);

    let id = uuid::Uuid::new_v4().to_string();

    let validation_message = if duration_ms > audio::validate::MAX_UPLOAD_DURATION_MS {
        Some(format!(
            "File is {:.0}s long, which is long for a reference. Consider using a shorter clip.",
            duration_ms / 1000.0
        ))
    } else {
        None
    };

    Ok(ReferenceAnalysis {
        id,
        path,
        filename,
        duration_ms,
        sample_rate: 44100,
        channels: 1,
        file_type: ext,
        waveform,
        rms,
        peak,
        validation_message,
    })
}

#[derive(Clone, serde::Serialize)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub is_available: bool,
    pub reason_unavailable: Option<String>,
    pub supports_reference_audio: bool,
    pub max_duration_seconds: f32,
    pub estimated_latency_ms: u32,
    pub estimated_cost_cents: f32,
    pub requires_api_key: bool,
    pub requires_network: bool,
}

#[tauri::command]
pub async fn get_generation_providers() -> Vec<ProviderInfo> {
    let registry = crate::generation::build_default_registry();
    registry.available_providers().iter().map(|p| {
        let caps = p.capabilities();
        ProviderInfo {
            name: p.name().to_string(),
            display_name: caps.name.to_string(),
            is_available: p.is_available(),
            reason_unavailable: p.reason_unavailable(),
            supports_reference_audio: caps.supports_reference_audio,
            max_duration_seconds: caps.max_duration_seconds,
            estimated_latency_ms: caps.estimated_latency_ms,
            estimated_cost_cents: caps.estimated_cost_per_generation_cents,
            requires_api_key: caps.requires_api_key,
            requires_network: caps.requires_network,
        }
    }).collect()
}

#[tauri::command]
pub async fn set_active_provider(
    state: State<'_, AppState>,
    provider_name: String,
) -> Result<String, String> {
    let mut registry = state.provider_registry.lock().map_err(|e| e.to_string())?;
    registry.set_active(&provider_name);
    Ok(provider_name)
}

#[tauri::command]
pub async fn analyze_prompt(prompt: String) -> crate::prompt_dsp::PromptDspControls {
    crate::prompt_dsp::parse_prompt_rich(&prompt)
}

#[tauri::command]
pub async fn get_active_provider(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let registry = state.provider_registry.lock().map_err(|e| e.to_string())?;
    Ok(registry.active_provider_name()
        .unwrap_or_else(|| "cshot-engine".to_string()))
}

#[tauri::command]
pub async fn generate_sound(
    prompt: String,
    reference_path: Option<String>,
    provider_name: Option<String>,
) -> Result<generator::SoundResult, String> {
    let trimmed = prompt.trim().to_string();
    if trimmed.is_empty() {
        return Err("Please describe the sound you want to generate.".to_string());
    }
    if trimmed.len() > 500 {
        return Err("Prompt is too long. Keep it under 500 characters.".to_string());
    }

    let mut request = GenerationRequest::from_prompt(&trimmed);

    if let Some(ref path) = reference_path {
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            return Err("Reference file not found. Please check the file path.".to_string());
        }
        let file_size = std::fs::metadata(&path_buf).map(|m| m.len()).unwrap_or(0);
        let validation = audio::validate::validate_upload(&path_buf, file_size);
        if !validation.is_valid {
            return Err(validation.error.unwrap_or_else(|| "Invalid reference file".to_string()));
        }
        let samples = audio::read_wav(&path_buf)?;
        request = request.with_reference(samples, 44100u32);
    }

    let provider = provider_name.as_deref();
    let result = generate_with_retry(request, &trimmed, provider, 2).await?;
    Ok(result)
}

/// Check that the local engine is available.
/// This never blocks — the local engine is always available.
/// Cloud provider key status is only relevant in Settings, never in the generation flow.
fn check_provider_keys() -> Result<(), String> {
    let registry = crate::generation::build_default_registry();
    let has_local_engine = registry.available_providers()
        .iter()
        .any(|p| p.name() == "cshot-engine");

    if has_local_engine {
        return Ok(());
    }

    Err("cShot Engine is not available. This should not happen.".to_string())
}

async fn generate_with_retry(
    request: GenerationRequest,
    prompt_text: &str,
    preferred_provider: Option<&str>,
    max_retries: u32,
) -> Result<generator::SoundResult, String> {
    check_provider_keys()?;

    let mut attempts = 0u32;

    loop {
        let registry = crate::generation::build_default_registry();
        if let Some(name) = preferred_provider {
            if registry.available_providers().iter().any(|p| p.name() == name) {
                // registry.set_active is not directly accessible since build_default_registry returns owned
            }
        }

        let result = registry.generate_with_fallback(request.clone()).await;
        match result {
            Ok((response, _validation, _errors)) => {
                let result = save_and_return_from_response(&response, prompt_text)?;
                if result.score >= 30 || attempts >= max_retries {
                    return Ok(result);
                }
                attempts += 1;
            }
            Err(e) => {
                if attempts >= max_retries {
                    return Err(format!(
                        "Couldn't generate the sound. {}",
                        user_friendly_error(&e)
                    ));
                }
                attempts += 1;
            }
        }
    }
}

fn user_friendly_error(error: &str) -> String {
    let lower = error.to_lowercase();
    if lower.contains("silent") || lower.contains("empty") {
        "The generated sound was silent. Try a more specific prompt.".to_string()
    } else if lower.contains("network") || lower.contains("timeout") || lower.contains("connection") {
        "Network issue. Check your connection and try again.".to_string()
    } else if lower.contains("api key") || lower.contains("auth") || lower.contains("unauthorized") {
        "Authentication issue. Check your API configuration.".to_string()
    } else if lower.contains("rate limit") || lower.contains("too many") {
        "Too many requests. Please wait a moment and try again.".to_string()
    } else if lower.contains("clip") || lower.contains("corrupt") || lower.contains("nan") {
        "The generated sound had quality issues. Try a different prompt.".to_string()
    } else if lower.contains("not found") || lower.contains("invalid") {
        error.to_string()
    } else {
        format!("Something went wrong: {}", error)
    }
}

fn save_and_return_from_response(
    response: &crate::generation::provider::GenerationResponse,
    prompt_text: &str,
) -> Result<generator::SoundResult, String> {
    let sound_dir = storage::audio_dir();
    fs::create_dir_all(&sound_dir).map_err(|e| e.to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let wav_path = sound_dir.join(format!("{}.wav", id));
    audio::write_wav(&wav_path, &response.samples, response.sample_rate)?;

    let waveform = audio::compute_waveform(&response.samples, 80);
    let duration_ms = response.samples.len() as f32 / response.sample_rate as f32 * 1000.0;

    let sound_type = crate::prompt::parse_prompt(prompt_text).sound_type;
    let sound_type_str = sound_type.as_str().to_string();
    let tags = audio::apply_autotags(&response.samples, &sound_type, None, Some(prompt_text));

    let rms = audio::compute_rms(&response.samples);
    let peak = audio::compute_peak(&response.samples);
    let spectral_centroid = audio::compute_spectral_centroid(&response.samples);

    let q = quality::compute_quality(&response.samples, sound_type, "original", true);

    let fb_store = feedback::FeedbackStore::load();
    let fb = fb_store.get(&id);
    let user_feedback = fb.map(|f| f.thumbs_up).or(Some(false));
    let usable = fb.and_then(|f| f.usable);

    let s = score::compute_score(&q, sound_type, user_feedback, usable);

    if let Ok(db_path) = crate::generator::db_path() {
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            let _ = db::insert_sound(&conn, &db::SoundEntry {
                id: id.clone(),
                prompt: prompt_text.to_string(),
                sound_type: sound_type_str.clone(),
                duration_ms,
                sample_rate: response.sample_rate,
                rms,
                peak,
                spectral_centroid,
                tags: tags_json,
                is_favorite: false,
                source: "generated".to_string(),
                variant_name: None,
                model: response.provider.clone(),
                seed: 0,
                created_at: String::new(),
            });
        }
    }

    crate::integrity::store_hash_for_new_sound(&id, &response.samples);

    Ok(generator::SoundResult {
        id,
        waveform,
        sound_type: sound_type_str,
        tags,
        duration_ms,
        prompt: prompt_text.to_string(),
        variant_name: None,
        source: "generated".to_string(),
        model: response.provider.clone(),
        seed: 0,
        rms,
        peak,
        spectral_centroid,
        score: s.overall,
        failure_labels: s.failure_labels,
    })
}

#[tauri::command]
pub async fn generate_resynthesis_variants(
    prompt: String,
    count: usize,
) -> Result<Vec<generator::VariantResult>, String> {
    let trimmed = prompt.trim().to_string();
    if trimmed.is_empty() {
        return Err("Please describe the sound you want.".to_string());
    }
    let ctrl = crate::prompt_dsp::parse_prompt_rich(&trimmed);
    crate::generator::generate_resynthesis_variants(&trimmed, &ctrl.sound_type, count)
}

#[tauri::command]
pub async fn generate_variants(
    prompt: String,
    source_id: String,
    count: usize,
) -> Result<Vec<generator::VariantResult>, String> {
    let src_path = storage::sound_path(&source_id);
    if !src_path.exists() {
        return Err(format!("Source sound {} not found on disk", source_id));
    }
    let samples = audio::read_wav(&src_path)?;

    let sound_type = "other".to_string();
    generator::generate_variants(&prompt, &samples, &sound_type, count)
}

#[tauri::command]
pub async fn resynthesize_sound(
    sound_id: String,
) -> Result<generator::SoundResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let resynth = audio::resynthesize::resynthesize_from_analysis(&analysis);

    if resynth.is_empty() {
        return Err("Resynthesis produced empty audio".to_string());
    }

    let prompt = format!("resynthesis of {} (analysis-driven)", analysis.sound_type_hint);
    generator::save_and_return(&resynth, &prompt, &analysis.sound_type_hint, Some("resynthesis"), 0)
}

#[derive(Clone, serde::Serialize)]
pub struct RecreateResult {
    pub approximations: Vec<RecreateApproximation>,
    pub original_analysis: crate::audio::analyze::AudioAnalysis,
}

#[derive(Clone, serde::Serialize)]
pub struct RecreateApproximation {
    pub id: String,
    pub sound_result: generator::SoundResult,
    pub similarity: crate::audio::recreate::SimilarityReport,
}

#[tauri::command]
pub async fn recreate_sound(
    sound_id: String,
    count: Option<usize>,
    fidelity: Option<f32>,
    preserve_transient: Option<bool>,
    preserve_body: Option<bool>,
    preserve_tail: Option<bool>,
) -> Result<RecreateResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);

    let approx_count = count.unwrap_or(4).clamp(1, 10);
    let fid = fidelity.unwrap_or(0.5).clamp(0.0, 1.0);
    let pres_t = preserve_transient.unwrap_or(true);
    let pres_b = preserve_body.unwrap_or(true);
    let pres_tail = preserve_tail.unwrap_or(true);

    let approximations = audio::recreate::generate_approximations(
        &samples, &analysis, approx_count, fid, pres_t, pres_b, pres_tail,
    );

    let mut results = Vec::new();
    for approx in &approximations {
        let prompt = format!("recreation of {} (fidelity:{:.0}%, similarity:{:.0}%)",
            analysis.sound_type_hint, fid * 100.0, approx.similarity.overall * 100.0);
        let sound_result = generator::save_and_return(
            &approx.samples,
            &prompt,
            &analysis.sound_type_hint,
            Some(&format!("recreate-v{}", approx.seed)),
            approx.seed as i64,
        )?;
        results.push(RecreateApproximation {
            id: approx.id.clone(),
            sound_result,
            similarity: approx.similarity.clone(),
        });
    }

    Ok(RecreateResult {
        approximations: results,
        original_analysis: analysis,
    })
}

#[tauri::command]
pub async fn transform_sound(
    sound_id: String,
    prompt: String,
) -> Result<generator::SoundResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let ctrl = crate::prompt_dsp::parse_prompt_rich(&prompt);
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let st = audio::SoundType::from_str(&analysis.sound_type_hint);
    let pitch = analysis.pitch_estimate.unwrap_or(200.0);
    let base = audio::resynthesize::params_for_sound_type(st, pitch, analysis.duration_ms);
    let params = ctrl.to_resynthesis_params(&base);
    let transformed = audio::transform::transform_with_params(&samples, &params);

    let ctrl = crate::prompt_dsp::parse_prompt_rich(&prompt);
    let st = if ctrl.sound_type_score > 0.3 { ctrl.sound_type } else {
        let a = audio::analyze::analyze_audio(&samples, 44100, 1);
        a.sound_type_hint
    };

    let sound_result = generator::save_and_return(
        &transformed, &prompt, &st,
        Some("transformed"), 0,
    )?;

    // Link to parent in DB
    if let Some(db_path) = crate::storage::database_path().to_str().map(|s| s.to_string()) {
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let _ = conn.execute(
                "UPDATE sounds SET parent_id = ?1 WHERE id = ?2",
                rusqlite::params![sound_id, sound_result.id],
            );
        }
    }

    Ok(sound_result)
}

#[tauri::command]
pub async fn get_audio_data(sound_id: String) -> Result<Vec<f32>, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err(format!("Sound file not found on disk: {}", sound_id));
    }
    audio::read_wav(&path)
}

fn pick_descriptor(prompt: &str, sound_type: &str) -> String {
    let style_words = [
        "punchy", "dark", "bright", "warm", "soft", "hard",
        "aggressive", "distorted", "clean", "noisy", "deep", "sub",
        "crisp", "shiny", "round", "tight", "fat", "dry",
        "wet", "metallic", "wooden", "organic", "crack", "snappy",
    ];
    let lower = prompt.to_lowercase();
    for &word in &style_words {
        if lower.contains(word) {
            return word.to_string();
        }
    }
    sound_type.to_string()
}

fn semantic_filename(sound_id: &str, prompt: &str, sound_type: &str, variant_name: Option<&str>, daw_friendly: bool) -> String {
    let id_short = if sound_id.len() >= 8 { &sound_id[..8] } else { sound_id };
    let base = if let Some(v) = variant_name {
        format!("cshot_{}_{}_{}", sound_type, v, id_short)
    } else {
        let descriptor = pick_descriptor(prompt, sound_type);
        if daw_friendly {
            format!("{} {}.{}", sound_type, descriptor, id_short)
        } else {
            format!("cshot_{}_{}_{}", sound_type, descriptor, id_short)
        }
    };

    let sanitized: String = if daw_friendly {
        base.chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '.' || c == '-' { c } else { '_' })
            .collect()
    } else {
        base.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect()
    };
    let sanitized = sanitized.trim_matches('_').to_string();
    let max_len = 60;
    if sanitized.len() > max_len {
        sanitized[..max_len].to_string()
    } else if sanitized.is_empty() {
        format!("cshot_{}_{}", sound_type, id_short)
    } else {
        sanitized
    }
}

#[derive(Clone, serde::Serialize)]
pub struct ExportResult {
    pub path: String,
    pub filename: String,
    pub file_size_bytes: u64,
}

#[tauri::command]
pub async fn export_wav(
    state: State<'_, AppState>,
    sound_id: String,
    daw_friendly: Option<bool>,
) -> Result<ExportResult, String> {
    let src = storage::sound_path(&sound_id);

    if !src.exists() {
          return Err(
            "The sound file could not be found on disk. It may have been deleted or moved. Try generating the sound again.".to_string()
        );
    }

    let file_size = fs::metadata(&src).map(|m| m.len()).unwrap_or(0);
    if file_size < 44 {
        return Err("The sound file appears to be empty or corrupted. Try generating again.".to_string());
    }

    let readable = std::fs::read(&src).ok();
    match readable {
        Some(ref data) if data.len() >= 44 && data[0..4] == [0x52, 0x49, 0x46, 0x46] => {},
        _ => return Err("The sound file is corrupted and cannot be exported. Try generating a new sound.".to_string()),
    }

    let (prompt, sound_type, variant_name) = if let Ok(conn) = state.db.lock() {
        if let Ok(Some(sound)) = db::get_sound(&conn, &sound_id) {
            (sound.prompt, sound.sound_type, sound.variant_name)
        } else {
            (String::new(), "sound".to_string(), None)
        }
    } else {
        (String::new(), "sound".to_string(), None)
    };

    let friendly = daw_friendly.unwrap_or(true);
    let filename = semantic_filename(&sound_id, &prompt, &sound_type, variant_name.as_deref(), friendly);
    let dest_filename = format!("{}.wav", filename);

    let desktop = storage::export_dir();
    let mut dest = desktop.join(&dest_filename);

    if dest.exists() {
        let mut counter = 1;
        loop {
            let alt = desktop.join(format!("{}_{}.wav", filename, counter));
            if !alt.exists() {
                dest = alt;
                break;
            }
            counter += 1;
            if counter > 100 {
                return Err("Too many files with similar name on Desktop".to_string());
            }
        }
    }

    fs::copy(&src, &dest).map_err(|e| {
        format!("Failed to export file: {}. Check that Desktop is writable.", e)
    })?;

    let exported_size = fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);

    if let Ok(conn) = state.db.lock() {
        let export_id = uuid::Uuid::new_v4().to_string();
        let _ = db::log_export(&conn, &export_id, &sound_id, &dest.to_string_lossy(), exported_size as i64);
    }

    Ok(ExportResult {
        path: dest.to_string_lossy().to_string(),
        filename: dest_filename,
        file_size_bytes: exported_size,
    })
}

#[derive(Clone, serde::Serialize)]
pub struct OpenFolderResult {
    pub path: String,
    pub opened: bool,
}

#[tauri::command]
pub async fn open_export_folder() -> Result<OpenFolderResult, String> {
    let path = storage::export_dir();
    let path_str = path.to_string_lossy().to_string();
    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg(&path_str).spawn().ok(); }
    #[cfg(target_os = "linux")]
    { std::process::Command::new("xdg-open").arg(&path_str).spawn().ok(); }
    #[cfg(target_os = "windows")]
    { std::process::Command::new("explorer").arg(&path_str).spawn().ok(); }
    Ok(OpenFolderResult { path: path_str, opened: true })
}

#[tauri::command]
pub async fn get_recent_exports(state: State<'_, AppState>) -> Result<Vec<ExportResult>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut rows = conn.prepare(
        "SELECT file_path, sound_id, file_size_bytes FROM exports ORDER BY exported_at DESC LIMIT 10"
    ).map_err(|e| e.to_string())?;
    let results = rows.query_map([], |row| {
        let path: String = row.get(0)?;
        let sound_id: String = row.get(1)?;
        let size: i64 = row.get(2)?;
        let filename = std::path::Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| sound_id);
        Ok(ExportResult { path, filename, file_size_bytes: size as u64 })
    }).map_err(|e| e.to_string())?;
    let mut exports = Vec::new();
    for r in results { exports.push(r.map_err(|e| e.to_string())?); }
    Ok(exports)
}

#[tauri::command]
pub async fn read_audio_file(path: String) -> Result<Vec<f32>, String> {
    let src = PathBuf::from(&path);
    if !src.exists() {
        return Err("File not found".to_string());
    }
    audio::read_wav(&src)
}

#[tauri::command]
pub async fn toggle_favorite(
    state: State<'_, AppState>,
    sound_id: String,
    prompt: Option<String>,
    sound_type: Option<String>,
    duration_ms: Option<f32>,
) -> Result<bool, String> {
    let mut favs = state.favorites.lock().map_err(|e| e.to_string())?;
    let meta = FavMetadata {
        id: sound_id.clone(),
        prompt: prompt.unwrap_or_default(),
        sound_type: sound_type.unwrap_or_default(),
        duration_ms: duration_ms.unwrap_or(0.0),
        created_at: chrono_now(),
        source: String::new(),
        model: String::new(),
        seed: 0,
        variant_name: None,
    };
    let is_fav = favs.toggle(&sound_id, meta);
    if let Ok(conn) = state.db.lock() {
        let _ = db::toggle_favorite(&conn, &sound_id);
    }
    Ok(is_fav)
}

#[tauri::command]
pub async fn get_favorites(state: State<'_, AppState>) -> Result<Vec<SoundEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_favorites(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sound_history(state: State<'_, AppState>) -> Result<Vec<SoundEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_recent_sounds(&conn, 50).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_sounds(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SoundEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::search_sounds(&conn, &query).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sound_detail(
    state: State<'_, AppState>,
    sound_id: String,
) -> Result<SoundEntry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_sound(&conn, &sound_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Sound not found".to_string())
}

#[tauri::command]
pub async fn copy_reference(source_id: String) -> Result<Vec<f32>, String> {
    let path = storage::sound_path(&source_id);
    if !path.exists() {
        return Err(format!("Reference file not found: {}", source_id));
    }
    audio::read_wav(&path)
}

#[tauri::command]
pub async fn delete_sound(
    state: State<'_, AppState>,
    sound_id: String,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_sound(&conn, &sound_id).map_err(|e| e.to_string())?;
    let audio_path = storage::sound_path(&sound_id);
    if audio_path.exists() {
        fs::remove_file(&audio_path).ok();
    }
    Ok(true)
}

#[tauri::command]
pub async fn list_sounds(
    state: State<'_, AppState>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<SoundEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    db::list_sounds_paginated(&conn, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn count_library_sounds(state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::count_sounds(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn count_favorite_sounds(state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::count_favorites(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_all_favorites(
    state: State<'_, AppState>,
) -> Result<ExportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let favorites = db::get_favorites(&conn).map_err(|e| e.to_string())?;
    if favorites.is_empty() {
        return Err("No favorites to export".to_string());
    }
    let desktop = storage::export_dir();
    let timestamp = chrono_now();
    let zip_name = format!("cshot_favorites_{}.zip", timestamp);
    let zip_path = desktop.join(&zip_name);
    let file = fs::File::create(&zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    for sound in &favorites {
        let audio_path = storage::sound_path(&sound.id);
        if audio_path.exists() {
            let short_id = if sound.id.len() >= 8 { &sound.id[..8] } else { &sound.id };
            let filename = format!("{}_{}.wav", sound.sound_type, short_id);
            let data = fs::read(&audio_path).map_err(|e| e.to_string())?;
            zip.start_file(&filename, FileOptions::<'_, ()>::default())
                .map_err(|e| e.to_string())?;
            zip.write_all(&data).map_err(|e| e.to_string())?;
        }
    }
    zip.finish().map_err(|e| e.to_string())?;
    let file_size = fs::metadata(&zip_path).map(|m| m.len()).unwrap_or(0);
    Ok(ExportResult {
        path: zip_path.to_string_lossy().to_string(),
        filename: zip_name,
        file_size_bytes: file_size,
    })
}

#[tauri::command]
pub async fn submit_feedback(
    sound_id: String,
    thumbs_up: bool,
    thumbs_down: bool,
    usable: Option<bool>,
) -> Result<(), String> {
    let mut store = feedback::FeedbackStore::load();
    store.set_thumbs(&sound_id, thumbs_up, thumbs_down);
    if let Some(u) = usable {
        store.set_usable(&sound_id, u);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_feedback(
    sound_id: String,
) -> Result<Option<feedback::FeedbackEntry>, String> {
    let store = feedback::FeedbackStore::load();
    Ok(store.get(&sound_id).cloned())
}

#[tauri::command]
pub async fn set_feedback_note(
    sound_id: String,
    note: Option<String>,
) -> Result<(), String> {
    let mut store = feedback::FeedbackStore::load();
    store.set_note(&sound_id, note);
    Ok(())
}

#[tauri::command]
pub async fn get_audio_analysis(
    sound_id: String,
) -> Result<crate::audio::analyze::AudioAnalysis, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound file not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = crate::audio::analyze::analyze_audio(&samples, 44100, 1);
    Ok(analysis)
}

#[tauri::command]
pub async fn get_sound_quality(
    state: State<'_, AppState>,
    sound_id: String,
) -> Result<QualityMetadata, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound file not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let sound_type = state
        .db
        .lock()
        .map_err(|e| e.to_string())?
        .query_row(
            "SELECT sound_type FROM sounds WHERE id = ?1",
            rusqlite::params![sound_id],
            |row| row.get::<_, String>(0),
        )
        .map(|t| audio::SoundType::from_str(&t))
        .unwrap_or(audio::SoundType::Other);
    Ok(quality::compute_quality(&samples, sound_type, "original", true))
}

#[tauri::command]
pub async fn get_sound_score(
    state: State<'_, AppState>,
    sound_id: String,
) -> Result<SoundScore, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound file not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let sound_type = state
        .db
        .lock()
        .map_err(|e| e.to_string())?
        .query_row(
            "SELECT sound_type FROM sounds WHERE id = ?1",
            rusqlite::params![sound_id],
            |row| row.get::<_, String>(0),
        )
        .map(|t| audio::SoundType::from_str(&t))
        .unwrap_or(audio::SoundType::Other);
    let q = quality::compute_quality(&samples, sound_type, "original", true);
    let fb_store = feedback::FeedbackStore::load();
    let fb = fb_store.get(&sound_id);
    let user_feedback = fb.map(|f| f.thumbs_up);
    let usable = fb.and_then(|f| f.usable);
    Ok(score::compute_score(&q, sound_type, user_feedback, usable))
}

#[tauri::command]
pub async fn get_bakeoff_data() -> Result<crate::generation::bakeoff::BakeoffSummary, String> {
    crate::generation::bakeoff::get_bakeoff_data().await
}

#[tauri::command]
pub async fn run_mini_bakeoff(
    prompt: String,
) -> Result<crate::generation::bakeoff::BakeoffEntry, String> {
    crate::generation::bakeoff::run_mini_bakeoff(prompt).await
}

#[tauri::command]
pub async fn find_similar_sounds(
    state: State<'_, AppState>,
    sound_id: String,
    max_results: Option<usize>,
) -> Result<Vec<crate::semantic_library::SimilarSound>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let target = db::get_sound(&conn, &sound_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Sound not found".to_string())?;
    let all = db::list_all_sounds(&conn).map_err(|e| e.to_string())?;
    let results = crate::semantic_library::find_similar_sounds(
        &target,
        &all,
        max_results.unwrap_or(10),
    );
    Ok(results)
}

#[tauri::command]
pub async fn filter_by_descriptors(
    state: State<'_, AppState>,
    include: Vec<String>,
) -> Result<Vec<SoundEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let all = db::list_all_sounds(&conn).map_err(|e| e.to_string())?;
    Ok(crate::semantic_library::filter_by_descriptors(&all, &include))
}

#[tauri::command]
pub async fn get_available_descriptors(
    state: State<'_, AppState>,
) -> Result<Vec<(String, usize)>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let all = db::list_all_sounds(&conn).map_err(|e| e.to_string())?;
    Ok(crate::semantic_library::available_descriptors(&all))
}

/// Apply a repair to an existing sound, creating a new version.
#[derive(Debug, serde::Deserialize)]
pub enum RepairAction {
    Normalize,
    TrimSilence,
    Fade,
    Shorten,
    Brighten,
    Darken,
    Punch,
    AddSub,
    Saturation,
    Soften,
    Sharpen,
    Compress,
}

#[tauri::command]
pub async fn apply_repair(
    sound_id: String,
    action: RepairAction,
) -> Result<generator::SoundResult, String> {
    let src_path = storage::sound_path(&sound_id);
    if !src_path.exists() {
        return Err(format!("Sound file not found: {}", sound_id));
    }

    let mut samples = audio::read_wav(&src_path)?;
    let prompt = {
        let db_path = storage::database_path();
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let result: Result<String, _> = conn.query_row(
                "SELECT prompt FROM sounds WHERE id = ?1",
                rusqlite::params![sound_id],
                |row| row.get(0),
            );
            result.unwrap_or_else(|_| "repaired sound".to_string())
        } else {
            "repaired sound".to_string()
        }
    };
    let sound_type = audio::SoundType::Other;

    match action {
        RepairAction::Normalize => {
            audio::process::normalize_peak(&mut samples, -1.0);
        }
        RepairAction::TrimSilence => {
            audio::process::trim_silence(&mut samples, 0.001);
        }
        RepairAction::Fade => {
            audio::process::apply_fade(&mut samples, 0.005, 0.010);
        }
        RepairAction::Shorten => {
            audio::process::shorten_duration(&mut samples, 44100, 0.7);
        }
        RepairAction::Brighten => {
            audio::process::brighten(&mut samples);
        }
        RepairAction::Darken => {
            audio::process::darken(&mut samples, 44100);
        }
        RepairAction::Punch => {
            audio::dsp::apply_punch(&mut samples);
        }
        RepairAction::AddSub => {
            let len = samples.len();
            for i in 0..len {
                let t = i as f32 / 44100.0;
                let env = (-3.0 * t).exp();
                let sub = (2.0 * std::f32::consts::PI * 55.0 * t).sin() * env * 0.3;
                samples[i] = (samples[i] + sub).clamp(-1.0, 1.0);
            }
        }
        RepairAction::Saturation => {
            for s in samples.iter_mut() {
                *s = (*s * 1.5).tanh();
            }
        }
        RepairAction::Soften => {
            let rc = 1.0 / (2.0 * std::f32::consts::PI * 3000.0);
            let dt = 1.0 / 44100.0;
            let alpha = dt / (rc + dt);
            let mut prev = 0.0;
            for sample in samples.iter_mut() {
                prev += alpha * (*sample - prev);
                *sample = prev;
            }
            for s in samples.iter_mut() { *s *= 0.7; }
        }
        RepairAction::Sharpen => {
            let onset_len = (44100.0 * 0.005) as usize;
            let threshold = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.3;
            for i in 10..samples.len().min(44100 / 2) {
                if samples[i].abs() > threshold {
                    let end = (i + onset_len).min(samples.len());
                    for j in i..end {
                        let t = (j - i) as f32 / onset_len as f32;
                        samples[j] *= 1.0 + 0.5 * (1.0 - t);
                    }
                    break;
                }
            }
        }
        RepairAction::Compress => {
            let threshold = 0.3;
            let ratio = 3.0;
            for s in samples.iter_mut() {
                let abs = s.abs();
                if abs > threshold {
                    let reduction = (abs - threshold) * (1.0 - 1.0 / ratio);
                    *s = s.signum() * (abs - reduction);
                }
            }
        }
    }

    audio::process::normalize_peak(&mut samples, -1.0);
    audio::process::validate_audio_integrity(&samples)?;

    let action_name = format!("{:?}", action);
    let repair_prompt = format!("{} (repaired: {})", prompt, action_name);
    generator::save_and_return(&samples, &repair_prompt, sound_type.as_str(), Some(&action_name), 0)
}

#[derive(Clone, serde::Serialize)]
pub struct PackCohesion {
    pub tag_overlap: f32,
    pub duration_consistency: f32,
    pub loudness_consistency: f32,
    pub category_balance: f32,
    pub overall_cohesion: f32,
}

#[tauri::command]
pub async fn get_pack_cohesion(
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<PackCohesion, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sounds = db::get_pack_sounds(&conn, &pack_id).map_err(|e| e.to_string())?;
    if sounds.is_empty() {
        return Ok(PackCohesion {
            tag_overlap: 0.0,
            duration_consistency: 0.0,
            loudness_consistency: 0.0,
            category_balance: 0.0,
            overall_cohesion: 0.0,
        });
    }
    let n = sounds.len() as f32;
    let mut tag_pairs = 0usize;
    let mut tag_matches = 0usize;
    let mut durations: Vec<f32> = sounds.iter().map(|s| s.duration_ms).collect();
    let mut loudnesses: Vec<f32> = sounds.iter().map(|s| s.rms).collect();
    let mut categories: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for s in &sounds {
        *categories.entry(s.sound_type.clone()).or_insert(0) += 1;
        let tags: Vec<String> =
            serde_json::from_str(&s.tags).unwrap_or_else(|_| vec![]);
        for other in &sounds {
            if s.id >= other.id {
                continue;
            }
            let other_tags: Vec<String> =
                serde_json::from_str(&other.tags).unwrap_or_else(|_| vec![]);
            tag_pairs += 1;
            let common = tags.iter().filter(|t| other_tags.contains(t)).count();
            if common > 0 {
                tag_matches += 1;
            }
        }
    }
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    loudnesses.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let dur_range = if n > 1.0 {
        (durations.last().unwrap_or(&0.0) - durations.first().unwrap_or(&0.0))
            / durations.iter().copied().fold(0.0f32, f32::max).max(1.0)
    } else {
        0.0
    };
    let loud_range = if n > 1.0 {
        (loudnesses.last().unwrap_or(&0.0) - loudnesses.first().unwrap_or(&0.0))
            .abs()
            .min(1.0)
    } else {
        0.0
    };
    let tag_overlap = if tag_pairs > 0 {
        tag_matches as f32 / tag_pairs as f32
    } else {
        0.0
    };
    let duration_consistency = 1.0 - dur_range;
    let loudness_consistency = 1.0 - loud_range;
    let max_cat = categories.values().copied().fold(0usize, usize::max).max(1);
    let category_balance = categories.len() as f32 / max_cat as f32 * 0.5;
    let overall_cohesion = (tag_overlap * 0.3
        + duration_consistency * 0.2
        + loudness_consistency * 0.2
        + category_balance * 0.3)
        .clamp(0.0, 1.0);
    Ok(PackCohesion {
        tag_overlap,
        duration_consistency,
        loudness_consistency,
        category_balance,
        overall_cohesion,
    })
}

#[tauri::command]
pub async fn create_pack(
    state: State<'_, AppState>,
    title: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::create_pack(
        &conn,
        &id,
        &title,
        &description.unwrap_or_default(),
        &tags.unwrap_or_default(),
    )
    .map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn list_packs(
    state: State<'_, AppState>,
) -> Result<Vec<db::PackEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_packs(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_pack(
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_pack(&conn, &pack_id).map_err(|e| e.to_string())
}

#[derive(Clone, serde::Serialize)]
pub struct PackDetail {
    pub pack: db::PackEntry,
    pub sounds: Vec<SoundEntry>,
}

#[tauri::command]
pub async fn get_pack_detail(
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<PackDetail, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let packs = db::list_packs(&conn).map_err(|e| e.to_string())?;
    let pack = packs
        .into_iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| "Pack not found".to_string())?;
    let sounds = db::get_pack_sounds(&conn, &pack_id).map_err(|e| e.to_string())?;
    Ok(PackDetail { pack, sounds })
}

#[tauri::command]
pub async fn add_to_pack(
    state: State<'_, AppState>,
    pack_id: String,
    sound_id: String,
    sort_order: Option<i32>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::add_sound_to_pack(&conn, &pack_id, &sound_id, sort_order.unwrap_or(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_from_pack(
    state: State<'_, AppState>,
    pack_id: String,
    sound_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::remove_sound_from_pack(&conn, &pack_id, &sound_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_pack(
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<ExportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let packs = db::list_packs(&conn).map_err(|e| e.to_string())?;
    let pack = packs
        .into_iter()
        .find(|p| p.id == pack_id)
        .ok_or_else(|| "Pack not found".to_string())?;
    let sounds = db::get_pack_sounds(&conn, &pack_id).map_err(|e| e.to_string())?;
    if sounds.is_empty() {
        return Err("Pack is empty".to_string());
    }
    let desktop = storage::export_dir();
    let safe_title: String = pack
        .title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let zip_name = format!("cshot_pack_{}.zip", safe_title);
    let zip_path = desktop.join(&zip_name);
    let file = fs::File::create(&zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    for sound in &sounds {
        let audio_path = storage::sound_path(&sound.id);
        if audio_path.exists() {
            let short_id = if sound.id.len() >= 8 { &sound.id[..8] } else { &sound.id };
            let filename =
                format!("{}_{}_{}.wav", sound.sound_type, short_id, safe_title);
            let data = fs::read(&audio_path).map_err(|e| e.to_string())?;
            zip.start_file(&filename, FileOptions::<'_, ()>::default())
                .map_err(|e| e.to_string())?;
            zip.write_all(&data).map_err(|e| e.to_string())?;
        }
    }
    zip.finish().map_err(|e| e.to_string())?;
    let file_size = fs::metadata(&zip_path).map(|m| m.len()).unwrap_or(0);
    Ok(ExportResult {
        path: zip_path.to_string_lossy().to_string(),
        filename: zip_name,
        file_size_bytes: file_size,
    })
}

#[tauri::command]
pub async fn update_sound_tags(
    state: State<'_, AppState>,
    sound_id: String,
    tags: Vec<String>,
) -> Result<(), String> {
    let tags_json = serde_json::to_string(&tags).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE sounds SET tags = ?1 WHERE id = ?2",
        rusqlite::params![tags_json, sound_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_sound_embedding(
    state: State<'_, AppState>,
    sound_id: String,
) -> Result<Vec<f32>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_embedding(&conn, &sound_id)
        .map_err(|e| e.to_string())?
        .map(|(vec, _)| vec)
        .ok_or_else(|| "No embedding found for this sound".to_string())
}

#[tauri::command]
pub async fn recompute_embeddings(
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let all = db::list_all_sounds(&conn).map_err(|e| e.to_string())?;
    let mut count = 0usize;
    for sound in &all {
        let audio_path = storage::sound_path(&sound.id);
        let samples = if audio_path.exists() {
            audio::read_wav(&audio_path).unwrap_or_default()
        } else {
            vec![]
        };
        let vec = crate::embeddings::compute_mock_embedding(sound, &samples);
        db::upsert_embedding(&conn, &sound.id, &vec, "mock-embedding")
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

#[derive(Clone, serde::Serialize)]
pub struct HybridSimilarResult {
    pub entry: SoundEntry,
    pub similarity_score: f32,
    pub match_reasons: Vec<String>,
}

#[tauri::command]
pub async fn hybrid_find_similar(
    state: State<'_, AppState>,
    sound_id: String,
    max_results: Option<usize>,
) -> Result<Vec<HybridSimilarResult>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let target = db::get_sound(&conn, &sound_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Sound not found".to_string())?;
    let all = db::list_all_sounds(&conn).map_err(|e| e.to_string())?;
    let all_embeddings = db::list_all_embeddings(&conn).map_err(|e| e.to_string())?;
    let embeddings: Vec<crate::embeddings::Embedding> = all_embeddings
        .into_iter()
        .map(|(sound_id, vector, provider)| crate::embeddings::Embedding {
            sound_id,
            vector,
            provider,
            created_at: String::new(),
        })
        .collect();
    let results =
        crate::embeddings::hybrid_similarity(&target, &all, &embeddings, max_results.unwrap_or(10));
    Ok(results
        .into_iter()
        .map(|(entry, score, reasons)| HybridSimilarResult {
            entry,
            similarity_score: score,
            match_reasons: reasons,
        })
        .collect())
}

#[derive(Clone, serde::Serialize)]
pub struct ProvenanceInfo {
    pub id: String,
    pub prompt: String,
    pub sound_type: String,
    pub duration_ms: f32,
    pub model: String,
    pub seed: i64,
    pub file_hash: String,
    pub parent_id: String,
    pub created_at: String,
    pub source: String,
    pub export_count: i64,
    pub last_exported_at: Option<String>,
}

#[tauri::command]
pub async fn get_sound_provenance(
    state: State<'_, AppState>,
    sound_id: String,
) -> Result<ProvenanceInfo, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let sound = db::get_sound(&conn, &sound_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Sound not found".to_string())?;

    let file_hash: String = conn
        .query_row(
            "SELECT COALESCE(file_hash, '') FROM sounds WHERE id = ?1",
            rusqlite::params![sound_id],
            |row| row.get(0),
        )
        .unwrap_or_default();

    let parent_id: String = conn
        .query_row(
            "SELECT COALESCE(parent_id, '') FROM sounds WHERE id = ?1",
            rusqlite::params![sound_id],
            |row| row.get(0),
        )
        .unwrap_or_default();

    let export_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM exports WHERE sound_id = ?1",
            rusqlite::params![sound_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let last_exported_at: Option<String> = conn
        .query_row(
            "SELECT MAX(exported_at) FROM exports WHERE sound_id = ?1",
            rusqlite::params![sound_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    Ok(ProvenanceInfo {
        id: sound.id,
        prompt: sound.prompt,
        sound_type: sound.sound_type,
        duration_ms: sound.duration_ms,
        model: sound.model,
        seed: sound.seed,
        file_hash,
        parent_id,
        created_at: sound.created_at,
        source: sound.source,
        export_count,
        last_exported_at,
    })
}

#[tauri::command]
pub async fn scan_integrity() -> Result<crate::integrity::IntegrityReport, String> {
    crate::integrity::scan_integrity()
}

#[tauri::command]
pub async fn update_missing_hashes() -> Result<usize, String> {
    crate::integrity::update_missing_hashes()
}

#[derive(Clone, serde::Serialize)]
pub struct CleanupResult {
    pub deleted_sounds: usize,
    pub deleted_orphans: usize,
    pub deleted_failed: usize,
    pub errors: Vec<String>,
    pub favorites_protected: usize,
}

#[tauri::command]
pub async fn cleanup_clear_generated(
    include_favorites: bool,
) -> Result<CleanupResult, String> {
    let result = crate::cleanup::clear_generated_sounds(include_favorites)?;
    Ok(CleanupResult {
        deleted_sounds: result.deleted_sounds,
        deleted_orphans: result.deleted_orphans,
        deleted_failed: result.deleted_failed,
        errors: result.errors,
        favorites_protected: result.favorites_protected,
    })
}

#[tauri::command]
pub async fn cleanup_clear_failed() -> Result<CleanupResult, String> {
    let result = crate::cleanup::clear_failed_jobs()?;
    Ok(CleanupResult {
        deleted_sounds: result.deleted_sounds,
        deleted_orphans: result.deleted_orphans,
        deleted_failed: result.deleted_failed,
        errors: result.errors,
        favorites_protected: result.favorites_protected,
    })
}

#[tauri::command]
pub async fn cleanup_clear_orphans() -> Result<CleanupResult, String> {
    let result = crate::cleanup::clear_orphaned_files()?;
    Ok(CleanupResult {
        deleted_sounds: result.deleted_sounds,
        deleted_orphans: result.deleted_orphans,
        deleted_failed: result.deleted_failed,
        errors: result.errors,
        favorites_protected: result.favorites_protected,
    })
}

#[tauri::command]
pub async fn cleanup_reset_database(dry_run: bool) -> Result<String, String> {
    crate::cleanup::reset_database(dry_run)
}

#[tauri::command]
pub async fn cleanup_rebuild_metadata() -> Result<CleanupResult, String> {
    let result = crate::cleanup::rebuild_metadata_from_storage()?;
    Ok(CleanupResult {
        deleted_sounds: result.deleted_sounds,
        deleted_orphans: result.deleted_orphans,
        deleted_failed: result.deleted_failed,
        errors: result.errors,
        favorites_protected: result.favorites_protected,
    })
}

#[tauri::command]
pub async fn cleanup_clear_everything() -> Result<CleanupResult, String> {
    #[cfg(not(debug_assertions))]
    {
        return Err("This command is only available in development builds.".to_string());
    }
    #[cfg(debug_assertions)]
    {
        let result = crate::cleanup::clear_everything()?;
        Ok(CleanupResult {
            deleted_sounds: result.deleted_sounds,
            deleted_orphans: result.deleted_orphans,
            deleted_failed: result.deleted_failed,
            errors: result.errors,
            favorites_protected: result.favorites_protected,
        })
    }
}

// ─── Version Tree / Children ─────────────────────────

#[tauri::command]
pub async fn get_sound_children(
    state: State<'_, AppState>,
    sound_id: String,
) -> Result<Vec<SoundEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_sound_children(&conn, &sound_id).map_err(|e| e.to_string())
}

// ─── Recipe Commands ─────────────────────────────────

#[tauri::command]
pub async fn create_recipe(
    state: State<'_, AppState>,
    title: String,
    prompt_template: String,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    transform_defaults: Option<String>,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::create_recipe(&conn, &db::RecipeEntry {
        id: id.clone(),
        title,
        prompt_template,
        description: description.unwrap_or_default(),
        category: category.unwrap_or_default(),
        tags: tags.unwrap_or_default(),
        transform_defaults: transform_defaults.unwrap_or_else(|| "{}".to_string()),
        is_builtin: false,
        usage_count: 0,
        is_favorite: false,
        created_at: String::new(),
        updated_at: String::new(),
    }).map_err(|e| e.to_string())?;
    Ok(id)
}

#[tauri::command]
pub async fn update_recipe(
    state: State<'_, AppState>,
    id: String,
    title: String,
    prompt_template: String,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    transform_defaults: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_recipe(&conn, &db::RecipeEntry {
        id,
        title,
        prompt_template,
        description: description.unwrap_or_default(),
        category: category.unwrap_or_default(),
        tags: tags.unwrap_or_default(),
        transform_defaults: transform_defaults.unwrap_or_else(|| "{}".to_string()),
        is_builtin: false,
        usage_count: 0,
        is_favorite: false,
        created_at: String::new(),
        updated_at: String::new(),
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_recipe(
    state: State<'_, AppState>,
    recipe_id: String,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_recipe(&conn, &recipe_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_recipes(
    state: State<'_, AppState>,
) -> Result<Vec<db::RecipeEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_recipes(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_recipes(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<db::RecipeEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::search_recipes(&conn, &query).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_recipe_favorite(
    state: State<'_, AppState>,
    recipe_id: String,
) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::toggle_recipe_favorite(&conn, &recipe_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn increment_recipe_usage(
    state: State<'_, AppState>,
    recipe_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::increment_recipe_usage(&conn, &recipe_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_from_recipe(
    state: State<'_, AppState>,
    recipe_id: String,
    prompt_override: Option<String>,
) -> Result<crate::generator::SoundResult, String> {
    let prompt = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let recipes = db::list_recipes(&conn).map_err(|e| e.to_string())?;
        let recipe = recipes.into_iter().find(|r| r.id == recipe_id)
            .ok_or_else(|| "Recipe not found".to_string())?;
        let prompt = prompt_override.unwrap_or(recipe.prompt_template);
        if prompt.trim().is_empty() {
            return Err("Recipe prompt template is empty".to_string());
        }
        let _ = db::increment_recipe_usage(&conn, &recipe_id);
        prompt
    };
    let result = crate::commands::generate_sound(prompt, None, None).await?;
    Ok(result)
}

// ─── Import Sample Command ────────────────────────────

#[derive(Clone, serde::Serialize)]
pub struct ImportResult {
    pub id: String,
    pub filename: String,
    pub duration_ms: f32,
    pub sample_rate: u32,
    pub channels: u16,
    pub file_size_bytes: u64,
    pub rms: f32,
    pub peak: f32,
    pub tags: Vec<String>,
    pub waveform: Vec<f32>,
}

#[tauri::command]
pub async fn import_sample(
    state: State<'_, AppState>,
    path: String,
) -> Result<ImportResult, String> {
    let src = std::path::PathBuf::from(&path);
    if !src.exists() {
        return Err("File not found".to_string());
    }
    let file_size = std::fs::metadata(&src).map(|m| m.len()).unwrap_or(0);
    let ext = src.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_else(|| "wav".to_string());
    if ext != "wav" && ext != "mp3" {
        return Err("Only WAV and MP3 files are supported for import".to_string());
    }
    let filename = src.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

    let (samples, sample_rate, channels) = if ext == "mp3" {
        let data = std::fs::read(&src).map_err(|e| e.to_string())?;
        let mut decoder = minimp3::Decoder::new(&data[..]);
        let mut all_samples = Vec::new();
        let mut actual_sample_rate = 44100u32;
        let mut actual_channels = 1u16;
        let mut first = true;
        while let Ok(frame) = decoder.next_frame() {
            if first {
                actual_sample_rate = frame.sample_rate as u32;
                actual_channels = frame.channels as u16;
                first = false;
            }
            for sample in frame.data {
                all_samples.push(sample as f32 / 32768.0);
            }
        }
        if all_samples.is_empty() {
            return Err("Could not decode MP3 file".to_string());
        }
        (all_samples, actual_sample_rate, actual_channels)
    } else {
        let samples = audio::read_wav(&src)?;
        (samples, 44100u32, 1u16)
    };

    let duration_ms = samples.len() as f32 / sample_rate as f32 * 1000.0;
    let waveform = audio::compute_waveform(&samples, 80);
    let rms = audio::compute_rms(&samples);
    let peak = audio::compute_peak(&samples);
    let tags = audio::apply_autotags(&samples, &audio::SoundType::Other, None, Some(&filename));
    let id = uuid::Uuid::new_v4().to_string();

    let dest = crate::storage::audio_dir().join(format!("{}.wav", id));
    std::fs::create_dir_all(crate::storage::audio_dir()).map_err(|e| e.to_string())?;
    audio::write_wav(&dest, &samples, 44100)?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tags_str = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
    let _ = conn.execute(
        "INSERT OR IGNORE INTO imported_samples (id, original_path, filename, format, duration_ms, sample_rate, channels, file_size_bytes, rms, peak, tags)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![id, path, filename, ext, duration_ms, sample_rate, channels, file_size as i64, rms, peak, tags_str],
    );
    drop(conn);

    Ok(ImportResult {
        id,
        filename,
        duration_ms,
        sample_rate,
        channels,
        file_size_bytes: file_size,
        rms,
        peak,
        tags,
        waveform,
    })
}

// ─── Folder Import (Dry-run) ─────────────────────────

#[derive(Clone, serde::Serialize)]
pub struct FolderScanResult {
    pub total_files: usize,
    pub valid_files: usize,
    pub oversized_files: usize,
    pub unsupported_files: usize,
    pub duplicates: Vec<String>,
    pub samples: Vec<FolderSampleInfo>,
}

#[derive(Clone, serde::Serialize)]
pub struct FolderSampleInfo {
    pub path: String,
    pub filename: String,
    pub duration_ms: f32,
    pub file_size_bytes: u64,
    pub format: String,
    pub is_duplicate: bool,
}

#[tauri::command]
pub async fn scan_folder_import(
    path: String,
) -> Result<FolderScanResult, String> {
    let dir = std::path::PathBuf::from(&path);
    if !dir.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    let max_file_size: u64 = 50 * 1024 * 1024; // 50MB
    let supported = ["wav", "mp3"];
    let mut total = 0usize;
    let mut valid = 0usize;
    let mut oversized = 0usize;
    let mut unsupported = 0usize;
    let mut samples = Vec::new();

    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_file() { continue; }
        total += 1;
        if total > 200 { break; }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !supported.contains(&ext.as_str()) {
            unsupported += 1;
            continue;
        }
        let file_size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        if file_size > max_file_size {
            oversized += 1;
            continue;
        }
        let filename = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let duration_ms = if ext == "wav" {
            if let Ok(s) = audio::read_wav(&p) {
                s.len() as f32 / 44100.0 * 1000.0
            } else { 0.0 }
        } else {
            let data = std::fs::read(&p).unwrap_or_default();
            let mut decoder = minimp3::Decoder::new(&data[..]);
            let mut total_samples = 0usize;
            while let Ok(frame) = decoder.next_frame() {
                total_samples += frame.data.len();
            }
            total_samples as f32 / 44100.0 * 1000.0
        };
        samples.push(FolderSampleInfo {
            path: p.to_string_lossy().to_string(),
            filename,
            duration_ms,
            file_size_bytes: file_size,
            format: ext,
            is_duplicate: false,
        });
        valid += 1;
    }
    Ok(FolderScanResult {
        total_files: total,
        valid_files: valid,
        oversized_files: oversized,
        unsupported_files: unsupported,
        duplicates: vec![],
        samples,
    })
}

// ─── Duplicate Detection ──────────────────────────────

#[derive(Clone, serde::Serialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub sounds: Vec<SoundEntry>,
}

#[tauri::command]
pub async fn find_duplicates(
    state: State<'_, AppState>,
) -> Result<Vec<DuplicateGroup>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let all = db::list_all_sounds(&conn).map_err(|e| e.to_string())?;
    let mut by_hash: std::collections::HashMap<String, Vec<SoundEntry>> = std::collections::HashMap::new();
    for sound in &all {
        let hash: String = conn.query_row(
            "SELECT COALESCE(file_hash, '') FROM sounds WHERE id = ?1",
            rusqlite::params![sound.id],
            |row| row.get(0),
        ).unwrap_or_default();
        if !hash.is_empty() {
            by_hash.entry(hash.clone()).or_default().push(sound.clone());
        }
    }
    let groups: Vec<DuplicateGroup> = by_hash.into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(hash, sounds)| DuplicateGroup { hash, sounds })
        .collect();
    Ok(groups)
}

// ─── Library Stats ────────────────────────────────────

#[tauri::command]
pub async fn get_library_stats(
    state: State<'_, AppState>,
) -> Result<db::LibraryStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_library_stats(&conn).map_err(|e| e.to_string())
}

// ─── Update Pack Notes ────────────────────────────────

#[tauri::command]
pub async fn update_pack_notes(
    state: State<'_, AppState>,
    pack_id: String,
    notes: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE packs SET notes = ?1, updated_at = datetime('now') WHERE id = ?2",
        rusqlite::params![notes, pack_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_pack_metadata(
    state: State<'_, AppState>,
    pack_id: String,
    title: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let current = db::list_packs(&conn).map_err(|e| e.to_string())?;
    let pack = current.into_iter().find(|p| p.id == pack_id).ok_or_else(|| "Pack not found".to_string())?;
    let new_title = if title.is_empty() { pack.title } else { title };
    let new_desc = description.unwrap_or(pack.description);
    let new_tags = tags.unwrap_or_else(|| serde_json::from_str(&pack.tags).unwrap_or_default());
    let _ = db::update_pack_metadata(&conn, &pack_id, &new_title, &new_desc, &new_tags);
    if let Some(n) = notes {
        let _ = conn.execute("UPDATE packs SET notes = ?1 WHERE id = ?2", rusqlite::params![n, pack_id]);
    }
    Ok(())
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

// ─── Undo / Redo ───────────────────────────────────────

#[tauri::command]
pub async fn undo_last_action(state: State<'_, AppState>) -> Result<Option<crate::history::ActionEntry>, String> {
    let mut history = state.history.lock().map_err(|e| e.to_string())?;
    Ok(history.undo())
}

#[tauri::command]
pub async fn redo_last_action(state: State<'_, AppState>) -> Result<Option<crate::history::ActionEntry>, String> {
    let mut history = state.history.lock().map_err(|e| e.to_string())?;
    Ok(history.redo())
}

#[tauri::command]
pub async fn can_undo_action(state: State<'_, AppState>) -> Result<bool, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    Ok(history.can_undo())
}

#[tauri::command]
pub async fn can_redo_action(state: State<'_, AppState>) -> Result<bool, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    Ok(history.can_redo())
}

// ─── Hybrid Reconstruction ─────────────────────────────

#[derive(Clone, serde::Serialize)]
pub struct HybridResult {
    pub sound_result: generator::SoundResult,
    pub analysis: crate::audio::analyze::AudioAnalysis,
}

#[tauri::command]
pub async fn hybrid_reconstruct(
    sound_id: String,
    synth_blend: Option<f32>,
    replace_transient: Option<bool>,
    replace_body: Option<bool>,
    replace_tail: Option<bool>,
    regenerate_tail: Option<bool>,
    sub_reinforce: Option<f32>,
    preserve_transient: Option<bool>,
    preserve_tail: Option<bool>,
    preserve_pitch: Option<bool>,
) -> Result<HybridResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);

    let params = audio::hybrid::HybridParams {
        synth_blend: synth_blend.unwrap_or(0.5),
        replace_transient: replace_transient.unwrap_or(false),
        replace_body: replace_body.unwrap_or(false),
        replace_tail: replace_tail.unwrap_or(false),
        regenerate_tail: regenerate_tail.unwrap_or(false),
        sub_reinforce: sub_reinforce.unwrap_or(0.0),
        transient_amount: if replace_transient.unwrap_or(false) { 1.0 } else { 0.0 },
        body_amount: if replace_body.unwrap_or(false) { 1.0 } else { 0.0 },
        spectral_blend: 0.0,
        preserve_transient: preserve_transient.unwrap_or(true),
        preserve_tail: preserve_tail.unwrap_or(true),
        preserve_pitch: preserve_pitch.unwrap_or(true),
        preserve_rhythm: true,
        preserve_texture: true,
    };

    let hybridized = audio::hybrid::hybrid_reconstruct(&samples, &analysis, &params);

    let prompt = format!("hybrid reconstruction of {} (blend:{:.0}%)",
        analysis.sound_type_hint, params.synth_blend * 100.0);
    let sound_result = generator::save_and_return(
        &hybridized, &prompt, &analysis.sound_type_hint,
        Some("hybrid"), 0,
    )?;

    Ok(HybridResult { sound_result, analysis })
}

// ─── Quick Compare ─────────────────────────────────────

#[derive(Clone, serde::Serialize)]
pub struct CompareResult {
    pub current_id: String,
    pub previous_id: Option<String>,
    pub has_previous: bool,
}

#[tauri::command]
pub async fn get_last_generation(state: State<'_, AppState>) -> Result<Option<crate::history::ActionEntry>, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    Ok(history.undo_stack.back().cloned())
}

#[tauri::command]
pub async fn record_generation_action(
    state: State<'_, AppState>,
    sound_id: String,
    prompt: String,
) -> Result<(), String> {
    let mut history = state.history.lock().map_err(|e| e.to_string())?;
    history.push_action(crate::history::ActionEntry {
        action_type: "generate".to_string(),
        sound_id,
        prompt,
        timestamp: chrono_now(),
    });
    Ok(())
}

// ─── Intelligent Pack Generation ─────────────────────

#[tauri::command]
pub async fn generate_intelligent_pack(
    genre: String,
    sound_count: usize,
) -> Result<audio::packs::GeneratedPack, String> {
    let profile = audio::packs::PackProfile::for_genre(&genre);
    let pack = audio::packs::generate_intelligent_pack(&profile, sound_count.clamp(4, 32));
    Ok(pack)
}

#[tauri::command]
pub async fn analyze_pack_cohesion_command(
    sound_ids: Vec<String>,
) -> Result<audio::packs::PackCohesion, String> {
    let mut analyses = Vec::new();
    let mut roles = Vec::new();
    for id in &sound_ids {
        let path = storage::sound_path(id);
        if !path.exists() { continue; }
        if let Ok(samples) = audio::read_wav(&path) {
            let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
            let st = audio::SoundType::from_str(&analysis.sound_type_hint);
            analyses.push(analysis);
            roles.push(audio::packs::SoundRole::from_sound_type(st));
        }
    }
    if analyses.is_empty() {
        return Err("No valid sounds found".to_string());
    }
    Ok(audio::packs::analyze_pack_cohesion(&analyses, &roles))
}

// ─── Spectral Editing ────────────────────────────────

#[tauri::command]
pub async fn apply_spectral_edit(
    sound_id: String,
    edit_params: audio::spectral_edit::SpectralEditParams,
) -> Result<generator::SoundResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let mut edited = samples;
    audio::spectral_edit::apply_spectral_edits(&mut edited, &edit_params);
    if edited.is_empty() {
        return Err("Spectral edit produced empty audio".to_string());
    }
    let prompt = format!("spectral edit of {}", analysis.sound_type_hint);
    generator::save_and_return(&edited, &prompt, &analysis.sound_type_hint, Some("spectral-edited"), 0)
}

#[tauri::command]
pub async fn isolate_region(
    sound_id: String,
    region: String,
) -> Result<generator::SoundResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let isolated = match region.as_str() {
        "transient" => audio::spectral_edit::extract_transient_region(&samples),
        "body" => audio::spectral_edit::extract_body_region(&samples),
        "tail" => audio::spectral_edit::extract_tail_region(&samples),
        _ => return Err("Region must be one of: transient, body, tail".to_string()),
    };
    if isolated.is_empty() {
        return Err("Isolation produced empty audio".to_string());
    }
    let mut normalized = isolated;
    audio::process::normalize_peak(&mut normalized, -1.0);
    let prompt = format!("isolated {} from {}", region, analysis.sound_type_hint);
    generator::save_and_return(&normalized, &prompt, &analysis.sound_type_hint, Some(&format!("isolated-{}", region)), 0)
}

// ─── Smart Recreation + Mutation ─────────────────────

#[tauri::command]
pub async fn mutate_sound_command(
    sound_id: String,
    mutation: String,
    intensity: f32,
) -> Result<generator::SoundResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let result = audio::mutation::apply_mutation_preset(&samples, &analysis, &mutation, intensity.clamp(0.0, 1.0));
    if result.samples.is_empty() {
        return Err("Mutation produced empty audio".to_string());
    }
    let prompt = format!("mutation:{} intensity:{:.0}%", mutation, intensity * 100.0);
    generator::save_and_return(&result.samples, &prompt, &analysis.sound_type_hint, Some(&format!("mutation-{}", mutation)), (intensity * 100.0) as i64)
}

#[tauri::command]
pub async fn hybridize_sounds_command(
    sound_id_a: String,
    sound_id_b: String,
    blend: f32,
) -> Result<generator::SoundResult, String> {
    let path_a = storage::sound_path(&sound_id_a);
    let path_b = storage::sound_path(&sound_id_b);
    if !path_a.exists() || !path_b.exists() {
        return Err("One or both source sounds not found".to_string());
    }
    let samples_a = audio::read_wav(&path_a)?;
    let samples_b = audio::read_wav(&path_b)?;
    let analysis_a = audio::analyze::analyze_audio(&samples_a, 44100, 1);
    let analysis_b = audio::analyze::analyze_audio(&samples_b, 44100, 1);
    let (hybrid, _) = audio::mutation::hybridize_sounds(&samples_a, &analysis_a, &samples_b, &analysis_b, blend.clamp(0.0, 1.0));
    if hybrid.is_empty() {
        return Err("Hybridization produced empty audio".to_string());
    }
    let prompt = format!("hybrid {:.0}% {} + {:.0}% {}",
        (1.0 - blend) * 100.0, analysis_a.sound_type_hint,
        blend * 100.0, analysis_b.sound_type_hint);
    generator::save_and_return(&hybrid, &prompt, &analysis_a.sound_type_hint, Some("hybrid"), (blend * 100.0) as i64)
}

#[tauri::command]
pub async fn get_available_mutations() -> Vec<String> {
    vec![
        "recreate".to_string(),
        "mutate".to_string(),
        "clean-up".to_string(),
        "modernize".to_string(),
        "exaggerate-punch".to_string(),
        "exaggerate-sub".to_string(),
        "exaggerate-bright".to_string(),
        "exaggerate-dark".to_string(),
        "exaggerate-distortion".to_string(),
        "exaggerate-short".to_string(),
        "exaggerate-long".to_string(),
        "evolve-harder".to_string(),
        "evolve-cleaner".to_string(),
        "evolve-warmer".to_string(),
        "evolve-brighter".to_string(),
        "evolve-heavier".to_string(),
        "evolve-lighter".to_string(),
        "evolve-longer".to_string(),
        "evolve-shorter".to_string(),
        "genre-trap".to_string(),
        "genre-techno".to_string(),
        "genre-cinematic".to_string(),
        "genre-lo-fi".to_string(),
        "genre-drill".to_string(),
        "genre-house".to_string(),
        "genre-dubstep".to_string(),
    ]
}

// ─── Personal Taste Model ────────────────────────────

#[tauri::command]
pub async fn record_taste_action(
    action: String,
    sound_id: String,
) -> Result<(), String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let (prompt, genre_hints) = if let Ok(db_path) = crate::generator::db_path() {
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            if let Ok(Some(entry)) = crate::db::get_sound(&conn, &sound_id) {
                let tags: Vec<String> = serde_json::from_str(&entry.tags).unwrap_or_default();
                let genres: Vec<String> = tags.iter().filter(|t| {
                    matches!(t.as_str(), "trap" | "techno" | "house" | "lo-fi" | "cinematic" | "drill" | "dubstep" | "ambient" | "electronic")
                }).cloned().collect();
                (entry.prompt, genres)
            } else {
                (String::new(), vec![])
            }
        } else {
            (String::new(), vec![])
        }
    } else {
        (String::new(), vec![])
    };

    let action_enum = match action.as_str() {
        "favorite" | "fav" => audio::taste::UserAction::Favorited,
        "export" => audio::taste::UserAction::Exported,
        "delete" => audio::taste::UserAction::Deleted,
        "regenerate" => audio::taste::UserAction::Regenerated,
        "thumbs_up" => audio::taste::UserAction::ThumbsUp,
        "thumbs_down" => audio::taste::UserAction::ThumbsDown,
        "preview" => audio::taste::UserAction::Previewed,
        "play" => audio::taste::UserAction::Played,
        _ => return Err(format!("Unknown action: {}", action)),
    };

    let record = audio::taste::ActionRecord {
        action: action_enum,
        sound_type: analysis.sound_type_hint.clone(),
        prompt,
        brightness: analysis.brightness,
        energy: analysis.rms,
        duration_ms: analysis.duration_ms,
        genre_hints,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    let mut model = audio::taste::TasteModel::load();
    model.record_action(record);
    model.save();
    Ok(())
}

#[tauri::command]
pub async fn get_taste_profile() -> audio::taste::TasteProfile {
    let model = audio::taste::TasteModel::load();
    model.profile
}

#[tauri::command]
pub async fn get_taste_suggestions(_sound_type: String) -> Vec<String> {
    let model = audio::taste::TasteModel::load();
    model.top_preferred_terms(10).into_iter().map(|(t, _)| t).collect()
}

#[tauri::command]
pub async fn get_preferred_defaults(sound_type: String) -> audio::params::ExposedParams {
    let model = audio::taste::TasteModel::load();
    model.suggested_defaults(&sound_type)
}

#[tauri::command]
pub async fn score_variant_by_taste(
    sound_type: String,
    brightness: f32,
    energy: f32,
) -> f32 {
    let model = audio::taste::TasteModel::load();
    model.score_variant(&sound_type, brightness, energy)
}

// ─── Session Persistence ────────────────────────────────

#[tauri::command]
pub async fn get_session_state() -> audio::session::SessionState {
    let manager = audio::session::SessionManager::load();
    manager.state().clone()
}

#[tauri::command]
pub async fn set_active_sound_session(sound_id: String) -> Result<(), String> {
    let mut manager = audio::session::SessionManager::load();
    manager.set_active_sound(sound_id);
    Ok(())
}

#[tauri::command]
pub async fn set_last_prompt_session(prompt: String, sound_type: String) -> Result<(), String> {
    let mut manager = audio::session::SessionManager::load();
    manager.set_last_prompt(prompt, sound_type);
    Ok(())
}

#[tauri::command]
pub async fn set_view_state_session(view: String) -> Result<(), String> {
    let mut manager = audio::session::SessionManager::load();
    manager.set_view_state(view);
    Ok(())
}

#[tauri::command]
pub async fn list_presets() -> Vec<audio::session::PresetEntry> {
    audio::session::load_presets()
}

#[tauri::command]
pub async fn save_preset_command(
    name: String,
    sound_type: String,
    params_json: String,
    tags: Vec<String>,
) -> Result<(), String> {
    let preset = audio::session::PresetEntry {
        name,
        sound_type,
        params_json,
        tags,
        created_at: chrono_now(),
    };
    audio::session::save_preset(&preset);
    Ok(())
}

#[tauri::command]
pub async fn delete_preset_command(name: String) -> Result<(), String> {
    audio::session::delete_preset(&name);
    Ok(())
}

// ─── Crash Safety / Integrity ───────────────────────────

#[tauri::command]
pub async fn verify_sound_integrity(sound_id: String) -> Result<bool, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Ok(false);
    }
    match audio::read_wav(&path) {
        Ok(samples) => {
            if samples.is_empty() { return Ok(false); }
            if samples.iter().any(|s| s.is_nan() || s.is_infinite()) { return Ok(false); }
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn export_sound_safe(sound_id: String, output_path: String) -> Result<(), String> {
    let src = storage::sound_path(&sound_id);
    if !src.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav_safe(&src)?;
    audio::validate::validate_output_samples(&samples)?;
    let dest = std::path::PathBuf::from(&output_path);

    // Atomic write: write to temp then rename
    let temp_path = dest.with_extension("tmp");
    audio::write_wav(&temp_path, &samples, 44100)?;
    std::fs::rename(&temp_path, &dest).map_err(|e| format!("Failed to write output: {}", e))?;

    Ok(())
}

// ─── Identity & Onboarding ────────────────────────────

#[tauri::command]
pub async fn get_app_identity() -> String {
    audio::identity::identity_statement()
}

#[tauri::command]
pub async fn get_default_presets() -> Vec<audio::identity::DefaultPreset> {
    audio::identity::default_presets()
}

#[tauri::command]
pub async fn get_quick_start_workflows() -> Vec<audio::identity::Workflow> {
    audio::identity::quick_start_workflows()
}

#[tauri::command]
pub async fn get_capability_summary() -> Vec<Vec<String>> {
    audio::identity::capability_summary().into_iter()
        .map(|(k, v)| vec![k.to_string(), v.to_string()])
        .collect()
}

// ─── Rapid-fire Preview ────────────────────────────────

#[tauri::command]
pub async fn get_sound_duration(sound_id: String) -> Result<f32, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    Ok(samples.len() as f32 / 44100.0 * 1000.0)
}

// ─── Parameter Exposure System ─────────────────────────

#[tauri::command]
pub async fn get_control_modes() -> Vec<String> {
    vec!["simple".to_string(), "advanced".to_string(), "sound_designer".to_string()]
}

#[tauri::command]
pub async fn params_from_exposed(
    exposed: audio::params::ExposedParams,
    sound_type: String,
    pitch_hz: f32,
    duration_ms: f32,
) -> String {
    let st = audio::SoundType::from_str(&sound_type);
    let params = exposed.to_resynthesis_params(st, pitch_hz, duration_ms);
    format!("{:?}", params)
}

#[tauri::command]
pub async fn generate_with_params(
    exposed: audio::params::ExposedParams,
    sound_type: String,
    pitch_hz: Option<f32>,
    duration_ms: Option<f32>,
) -> Result<generator::SoundResult, String> {
    let st = audio::SoundType::from_str(&sound_type);
      let pitch = pitch_hz.unwrap_or(match st {
        audio::SoundType::Kick | audio::SoundType::Bass => 60.0,
        audio::SoundType::Snare => 200.0,
        audio::SoundType::ClosedHat => 400.0,
        audio::SoundType::OpenHat => 300.0,
        audio::SoundType::Clap => 180.0,
        audio::SoundType::Tom => 120.0,
        audio::SoundType::Perc => 300.0,
        _ => 200.0,
    });
    let dur = duration_ms.unwrap_or(300.0);
    let params = exposed.to_resynthesis_params(st, pitch, dur);
    let samples = audio::resynthesize::resynthesize(&params);
    if samples.is_empty() {
        return Err("Generation produced empty audio".to_string());
    }
    let prompt = format!("params mode:{:?} type:{}", exposed.mode, sound_type);
    generator::save_and_return(&samples, &prompt, &sound_type, Some("params-generated"), 0)
}

#[tauri::command]
pub async fn get_exposed_params_defaults(mode: String) -> audio::params::ExposedParams {
    match mode.as_str() {
        "advanced" => audio::params::ExposedParams { mode: audio::params::ControlMode::Advanced, ..Default::default() },
        "sound_designer" => audio::params::ExposedParams { mode: audio::params::ControlMode::SoundDesigner, ..Default::default() },
        _ => audio::params::ExposedParams::default(),
    }
}

// ─── Real Instrument Behavior ─────────────────────────

#[tauri::command]
pub async fn trigger_midi_note(
    midi_note: u8,
    velocity: u8,
    character: Option<f32>,
    weight: Option<f32>,
) -> Result<generator::SoundResult, String> {
    let (st, pitch) = audio::midi::DrumNote::from_midi_note(midi_note)
        .unwrap_or((audio::SoundType::Perc, midi_note as f32 * 8.0));
    let dur = match st {
        audio::SoundType::Kick => 300.0, audio::SoundType::Snare => 350.0,
        audio::SoundType::ClosedHat => 150.0, audio::SoundType::OpenHat => 500.0,
        audio::SoundType::Clap => 300.0, audio::SoundType::Bass => 600.0,
        audio::SoundType::Perc => 200.0, audio::SoundType::Fx => 800.0,
        audio::SoundType::Tom => 400.0, audio::SoundType::Other => 300.0,
    };
    let mut params = audio::resynthesize::params_for_sound_type(st, pitch, dur);
    let v = audio::midi::params_for_velocity(&params, velocity);
    params = v;
    if let Some(c) = character {
        params.brightness = (params.brightness + c * 0.3).clamp(0.0, 1.0);
    }
    if let Some(w) = weight {
        params.body_gain = (params.body_gain * (0.5 + w * 0.5)).clamp(0.1, 1.0);
        params.sub_gain = (params.sub_gain + w * 0.3).clamp(0.0, 1.0);
    }
    let samples = audio::resynthesize::resynthesize(&params);
    if samples.is_empty() {
        return Err("MIDI trigger produced empty audio".to_string());
    }
    let prompt = format!("midi note:{} vel:{} type:{}", midi_note, velocity, st.as_str());
    generator::save_and_return(&samples, &prompt, st.as_str(), Some("midi-trigger"), velocity as i64)
}

#[tauri::command]
pub async fn rapid_randomize(
    sound_id: String,
    amount: f32,
    count: usize,
) -> Result<Vec<generator::SoundResult>, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let st = audio::SoundType::from_str(&analysis.sound_type_hint);
    let pitch = analysis.pitch_estimate.unwrap_or(200.0);
    let base_params = audio::resynthesize::params_for_sound_type(st, pitch, analysis.duration_ms);
    let variants = audio::midi::generate_rapid_variants(&base_params, count.min(16), amount.clamp(0.1, 1.0));
    let mut results = Vec::new();
    for (i, v) in variants.iter().enumerate() {
        let prompt = format!("rapid variant {} of {} amount:{:.1}", i + 1, count, amount);
        match generator::save_and_return(v, &prompt, st.as_str(), Some("rapid-variant"), i as i64) {
            Ok(r) => results.push(r),
            Err(_) => continue,
        }
    }
    Ok(results)
}

#[tauri::command]
pub async fn morph_presets(
    preset_a_id: String,
    preset_b_id: String,
    morph_amount: f32,
) -> Result<generator::SoundResult, String> {
    let path_a = storage::sound_path(&preset_a_id);
    let path_b = storage::sound_path(&preset_b_id);
    if !path_a.exists() || !path_b.exists() {
        return Err("One or both source sounds not found".to_string());
    }
    let samples_a = audio::read_wav(&path_a)?;
    let samples_b = audio::read_wav(&path_b)?;
    let analysis_a = audio::analyze::analyze_audio(&samples_a, 44100, 1);
    let analysis_b = audio::analyze::analyze_audio(&samples_b, 44100, 1);
    let st_a = audio::SoundType::from_str(&analysis_a.sound_type_hint);
    let st_b = audio::SoundType::from_str(&analysis_b.sound_type_hint);
    let pitch_a = analysis_a.pitch_estimate.unwrap_or(200.0);
    let pitch_b = analysis_b.pitch_estimate.unwrap_or(200.0);
    let params_a = audio::resynthesize::params_for_sound_type(st_a, pitch_a, analysis_a.duration_ms);
    let params_b = audio::resynthesize::params_for_sound_type(st_b, pitch_b, analysis_b.duration_ms);
    let morphed = audio::midi::morph_params(&params_a, &params_b, morph_amount);
    let samples = audio::resynthesize::resynthesize(&morphed);
    if samples.is_empty() {
        return Err("Morph produced empty audio".to_string());
    }
    let target_type = if morph_amount < 0.5 { st_a.as_str() } else { st_b.as_str() };
    let prompt = format!("morph {:.0}% {} to {}", morph_amount * 100.0, st_a.as_str(), st_b.as_str());
    generator::save_and_return(&samples, &prompt, target_type, Some("morphed"), (morph_amount * 100.0) as i64)
}

#[tauri::command]
pub async fn morph_sounds_command(
    sound_a_id: String,
    sound_b_id: String,
    amount: f32,
    preserve_source_identity: Option<f32>,
    exaggerate: Option<f32>,
    preserve_transient: Option<f32>,
    preserve_body: Option<f32>,
    preserve_tail: Option<f32>,
    transient_transfer: Option<f32>,
    tail_transfer: Option<f32>,
    tonal_blend: Option<f32>,
    texture_blend: Option<f32>,
) -> Result<(generator::SoundResult, audio::recreate::SimilarityReport), String> {
    let path_a = storage::sound_path(&sound_a_id);
    let path_b = storage::sound_path(&sound_b_id);
    if !path_a.exists() || !path_b.exists() {
        return Err("One or both source sounds not found".to_string());
    }
    let samples_a = audio::read_wav(&path_a)?;
    let samples_b = audio::read_wav(&path_b)?;

    let controls = audio::morph::MorphControls {
        amount: amount.clamp(0.0, 1.0),
        preserve_source_identity: preserve_source_identity.unwrap_or(0.5),
        exaggerate: exaggerate.unwrap_or(0.0),
        preserve_transient: preserve_transient.unwrap_or(1.0),
        preserve_body: preserve_body.unwrap_or(0.5),
        preserve_tail: preserve_tail.unwrap_or(0.5),
        transient_transfer: transient_transfer.unwrap_or(0.5),
        tail_transfer: tail_transfer.unwrap_or(0.5),
        tonal_blend: tonal_blend.unwrap_or(0.5),
        texture_blend: texture_blend.unwrap_or(0.5),
    };

    let (morphed, report) = audio::morph::morph(&samples_a, &samples_b, &controls);
    if morphed.is_empty() {
        return Err("Morph produced empty audio".to_string());
    }

    let analysis_a = audio::analyze::analyze_audio(&samples_a, 44100, 1);
    let analysis_b = audio::analyze::analyze_audio(&samples_b, 44100, 1);
    let target_type = if amount < 0.5 { analysis_a.sound_type_hint.clone() } else { analysis_b.sound_type_hint.clone() };
    let prompt = format!("morph {:.0}% to {:.0}%", (1.0 - amount) * 100.0, amount * 100.0);

    let result = generator::save_and_return(
        &morphed, &prompt, &target_type, Some("morphed"),
        (amount * 10000.0) as i64,
    )?;

    Ok((result, report))
}

#[tauri::command]
pub async fn live_preview_generate(config: audio::midi::PreviewConfig) -> Result<generator::SoundResult, String> {
    let samples = config.generate();
    if samples.is_empty() {
        return Err("Preview produced empty audio".to_string());
    }
    let prompt = format!("preview {}", config.sound_type);
    generator::save_and_return(&samples, &prompt, &config.sound_type, Some("preview"), config.velocity as i64)
}

#[tauri::command]
pub async fn generate_variant_from_params(
    sound_id: String,
    exposed: audio::params::ExposedParams,
) -> Result<generator::SoundResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let st = audio::SoundType::from_str(&analysis.sound_type_hint);
    let pitch = analysis.pitch_estimate.unwrap_or(200.0);
    let params = exposed.to_resynthesis_params(st, pitch, analysis.duration_ms);
    let recreated = audio::resynthesize::resynthesize(&params);
    if recreated.is_empty() {
        return Err("Recreation produced empty audio".to_string());
    }
    let prompt = format!("variant from params mode:{:?}", exposed.mode);
    generator::save_and_return(&recreated, &prompt, &analysis.sound_type_hint, Some("params-variant"), 0)
}

#[tauri::command]
pub async fn analyze_kit_command(sound_ids: Vec<String>) -> Result<audio::packs::KitAdvice, String> {
    if sound_ids.is_empty() {
        return Err("No sounds provided".to_string());
    }

    let mut sounds = Vec::new();
    for id in &sound_ids {
        let path = storage::sound_path(id);
        if !path.exists() {
            return Err(format!("Sound not found: {}", id));
        }
        let samples = audio::read_wav(&path)?;
        let st = audio::SoundType::from_str(&"other");
        let role = audio::packs::SoundRole::from_sound_type(st);
        sounds.push(audio::packs::GeneratedPackSound {
            role,
            name: id.clone(),
            samples,
            energy: 0.5,
            params: String::new(),
        });
    }

    let advice = audio::packs::analyze_kit_composition(&sounds);
    Ok(advice)
}

#[tauri::command]
pub async fn explore_stream(
    prompt: String,
    count: usize,
    variation: f32,
) -> Result<Vec<generator::VariantResult>, String> {
    if count == 0 || count > 20 {
        return Err("Count must be between 1 and 20".to_string());
    }
    let variation = variation.clamp(0.1, 1.0);

    let ctrl = crate::prompt_dsp::parse_prompt_rich(&prompt);
    let st = audio::SoundType::from_str(&ctrl.sound_type);
    let base_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64;

    let pitch = ctrl.pitch_hz.unwrap_or(200.0);
    let dur = ctrl.duration_ms.unwrap_or(300.0);
    let base_params = audio::resynthesize::params_for_sound_type(st, pitch, dur);

    let mut results = Vec::new();
    for i in 0..count {
        let seed = base_seed.wrapping_add((i as i64 * 7919) ^ 0xABCD);
        let randomized = base_params.clone().with_seed(seed as u64).randomize(variation);
        let mut samples = audio::resynthesize::resynthesize(&randomized);

        let parsed = crate::prompt::parse_prompt(&prompt);
        audio::process::process_sound(&mut samples, &parsed.dsp, st);
        if samples.is_empty() { continue; }

        let result = generator::save_and_return(
            &samples, &prompt, st.as_str(), Some("explore"), seed,
        )?;

        let tags = audio::apply_autotags(&samples, &st, Some("explore"), Some(&prompt));
        results.push(generator::VariantResult {
            id: result.id,
            waveform: result.waveform,
            sound_type: result.sound_type,
            tags,
            duration_ms: result.duration_ms,
            prompt: result.prompt,
            variant_name: format!("explore-{}", i),
            source: "explore".to_string(),
            model: "cshot-engine".to_string(),
            seed,
            score: result.score,
            failure_labels: result.failure_labels,
        });
    }
    Ok(results)
}

#[tauri::command]
pub async fn branch_from_sound(
    sound_id: String,
    mutation_name: String,
    intensity: f32,
) -> Result<generator::SoundResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);

    let mutation = if mutation_name.is_empty() { "mutate" } else { &mutation_name };
    let intensity = intensity.clamp(0.1, 1.0);
    let result = audio::mutation::apply_mutation_preset(&samples, &analysis, mutation, intensity);

    if result.samples.is_empty() {
        return Err("Branch produced empty audio".to_string());
    }

    let prompt = format!("branch: {} from ({:.0}%)", mutation, intensity * 100.0);
    generator::save_and_return(
        &result.samples, &prompt, &analysis.sound_type_hint,
        Some(&format!("branch-{}", mutation)), 0,
    )
}

#[tauri::command]
pub async fn recipe_roulette() -> Result<generator::SoundResult, String> {
    let recipes = vec![
        "punchy kick with sub",
        "crisp snare with crack",
        "bright closed hat tight",
        "warm open hat wash",
        "layered clap with reverb",
        "deep tom with body",
        "metallic perc with snap",
        "sub bass with distortion",
        "cinematic impact with tail",
        "airy perc with sizzle",
        "dark kick with boom",
        "tight snare with click",
        "noisy clap with texture",
        "warm bass with sub",
        "aggressive snare with crunch",
        "smooth open hat with air",
        "vintage kick with analog feel",
        "raw perc with organic texture",
    ];

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as usize;

    let idx = seed % recipes.len();
    let prompt = recipes[idx].to_string();
    generator::generate(&prompt, None)
}

#[tauri::command]
pub async fn quick_compare(
    prompt: String,
    count: usize,
) -> Result<Vec<generator::VariantResult>, String> {
    let count = count.clamp(2, 8);
    let ctrl = crate::prompt_dsp::parse_prompt_rich(&prompt);
    let st = audio::SoundType::from_str(&ctrl.sound_type);
    let base_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64;

    let pitch = ctrl.pitch_hz.unwrap_or(200.0);
    let dur = ctrl.duration_ms.unwrap_or(300.0);
    let base_params = audio::resynthesize::params_for_sound_type(st, pitch, dur);

    let variant_styles = ["brighter", "darker", "punchier", "softer", "cleaner", "warmer", "distorted", "subbier"];
    let mut results = Vec::new();

    for i in 0..count.min(variant_styles.len()) {
        let seed = base_seed.wrapping_add((i as i64 * 1337) ^ 0xCAFE);
        let randomized = base_params.clone().with_seed(seed as u64).randomize(0.25);
        let params = randomized.to_variant(variant_styles[i]);
        let mut samples = audio::resynthesize::resynthesize(&params);
        let parsed = crate::prompt::parse_prompt(&prompt);
        audio::process::process_sound(&mut samples, &parsed.dsp, st);
        if samples.is_empty() { continue; }

        let result = generator::save_and_return(
            &samples, &prompt, st.as_str(), Some(variant_styles[i]), seed,
        )?;

        let tags = audio::apply_autotags(&samples, &st, Some(variant_styles[i]), Some(&prompt));
        results.push(generator::VariantResult {
            id: result.id,
            waveform: result.waveform,
            sound_type: result.sound_type,
            tags,
            duration_ms: result.duration_ms,
            prompt: result.prompt,
            variant_name: variant_styles[i].to_string(),
            source: "compare".to_string(),
            model: "cshot-engine".to_string(),
            seed,
            score: result.score,
            failure_labels: result.failure_labels,
        });
    }
    Ok(results)
}

#[derive(Clone, serde::Serialize)]
pub struct DesignWorkflowResult {
    pub recreation: Option<generator::SoundResult>,
    pub mutation: Option<generator::SoundResult>,
    pub morphed: Option<generator::SoundResult>,
    pub branched: Vec<generator::VariantResult>,
    pub recreation_similarity: Option<audio::recreate::SimilarityReport>,
    pub workflow_steps: Vec<String>,
    pub total_time_ms: f64,
}

#[tauri::command]
pub async fn design_workflow(
    source_sound_id: String,
    reference_sound_id: Option<String>,
    prompt: Option<String>,
    do_recreate: bool,
    do_mutate: bool,
    do_morph: bool,
    do_branch: bool,
    recreate_fidelity: Option<f32>,
    mutation_name: Option<String>,
    mutation_intensity: Option<f32>,
    morph_amount: Option<f32>,
    branch_count: Option<usize>,
) -> Result<DesignWorkflowResult, String> {
    let start = std::time::Instant::now();

    let path = storage::sound_path(&source_sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let source_samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&source_samples, 44100, 1);
    let st = audio::SoundType::from_str(&analysis.sound_type_hint);
    let prompt_text = prompt.unwrap_or_else(|| format!("design workflow from {}", analysis.sound_type_hint));

    let mut result = DesignWorkflowResult {
        recreation: None,
        mutation: None,
        morphed: None,
        branched: Vec::new(),
        recreation_similarity: None,
        workflow_steps: Vec::new(),
        total_time_ms: 0.0,
    };

    // Step 1: Recreate
    if do_recreate {
        let fidelity = recreate_fidelity.unwrap_or(0.7).clamp(0.1, 1.0);
        let (recreated, _analysis, sim) = audio::recreate::recreate_single(&source_samples, fidelity);
        if !recreated.is_empty() {
            let saved = generator::save_and_return(
                &recreated, &prompt_text, st.as_str(), Some("recreated"), 0,
            )?;
            result.recreation = Some(saved);
            result.recreation_similarity = Some(sim);
            result.workflow_steps.push("recreation".to_string());
        }
    }

    // Step 2: Mutate the recreation
    if do_mutate {
        let samples = result.recreation.as_ref()
            .and_then(|r| {
                let p = storage::sound_path(&r.id);
                audio::read_wav(&p).ok()
            })
            .unwrap_or_else(|| source_samples.clone());

        let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
        let mutation = mutation_name.as_deref().unwrap_or("mutate");
        let intensity = mutation_intensity.unwrap_or(0.5).clamp(0.1, 1.0);
        let mutated = audio::mutation::apply_mutation_preset(&samples, &analysis, mutation, intensity);

        if !mutated.samples.is_empty() {
            let saved = generator::save_and_return(
                &mutated.samples, &format!("{} mutated", prompt_text), st.as_str(),
                Some(&format!("mutated-{}", mutation)), 0,
            )?;
            result.mutation = Some(saved);
            result.workflow_steps.push(format!("mutation: {}", mutation));
        }
    }

    // Step 3: Morph with reference
    if do_morph {
        if let Some(ref_id) = &reference_sound_id {
            let ref_path = storage::sound_path(ref_id);
            if ref_path.exists() {
                if let Ok(ref_samples) = audio::read_wav(&ref_path) {
                    let amt = morph_amount.unwrap_or(0.5).clamp(0.0, 1.0);
                    let controls = audio::morph::MorphControls {
                        amount: amt,
                        ..Default::default()
                    };
                    let (morphed, _report) = audio::morph::morph(&source_samples, &ref_samples, &controls);
                    if !morphed.is_empty() {
                        let saved = generator::save_and_return(
                            &morphed, &format!("morphed {:.0}%", amt * 100.0), st.as_str(),
                            Some("morphed"), (amt * 100.0) as i64,
                        )?;
                        result.morphed = Some(saved);
                        result.workflow_steps.push(format!("morph: {:.0}%", amt * 100.0));
                    }
                }
            }
        }
    }

    // Step 4: Branch
    if do_branch {
        let count = branch_count.unwrap_or(4).clamp(1, 8);
        let variants = generator::generate_smart_variants(
            &prompt_text, st.as_str(), count, 1, 0.5,
        )?;
        result.branched = variants;
        result.workflow_steps.push(format!("branch: {} variants", count));
    }

    result.total_time_ms = start.elapsed().as_secs_f64() * 1000.0;
    Ok(result)
}

// ─── Creative Intent Commands ───────────────────────────

#[derive(Clone, serde::Serialize)]
pub struct IntentGenerateResult {
    pub sound: generator::SoundResult,
    pub profile: audio::creative_intent::CreativeIntentProfile,
    pub params_summary: String,
}

#[tauri::command]
pub async fn generate_with_intent(
    prompt: String,
    intent_profile: audio::creative_intent::CreativeIntentProfile,
    reference_path: Option<String>,
) -> Result<IntentGenerateResult, String> {
    let reference_samples = if let Some(ref_path) = &reference_path {
        let path = std::path::PathBuf::from(ref_path);
        if path.exists() {
            Some(audio::read_wav(&path)?)
        } else {
            None
        }
    } else {
        None
    };

    let analysis = reference_samples.as_ref().map(|s| audio::analyze::analyze_audio(s, 44100, 1));

    let (sound_type, pitch_hz, duration_ms) = if let Some(a) = &analysis {
        let st = audio::SoundType::from_str(&a.sound_type_hint);
        let pitch = a.pitch_estimate.unwrap_or(200.0);
        (st, pitch, a.duration_ms)
    } else {
        let parsed = crate::prompt_dsp::parse_prompt_rich(&prompt);
        let st = audio::SoundType::from_str(&parsed.sound_type);
        let pitch = 200.0;
        let duration = 300.0;
        (st, pitch, duration)
    };

    let coordinated = audio::creative_intent::generate_with_intent(sound_type, pitch_hz, duration_ms, &intent_profile);
    let full_prompt = format!("{} [intent: energy={:.1} aggression={:.1} polish={:.1} realism={:.1} experimental={:.1} analog={:.1} cinematic={:.1} density={:.1} impact={:.1}]",
        prompt,
        intent_profile.energy, intent_profile.aggression, intent_profile.polish,
        intent_profile.realism, intent_profile.experimentalism, intent_profile.analog_feel,
        intent_profile.cinematic_scale, intent_profile.density, intent_profile.impact,
    );

    let mut samples = audio::resynthesize::resynthesize(&coordinated.resynthesis);

    let humanize_params = &coordinated.humanize;
    if humanize_params.analog_drift > 0.001 || humanize_params.instability > 0.001 {
        audio::humanize::humanize(&mut samples, humanize_params, 42);
    }

    let mut transform_params = coordinated.transform;
    if let Some(sat) = transform_params.saturation_drive {
        if sat > 1.01 {
            audio::dsp::apply_saturation_multi_stage(&mut samples, sat);
        }
        transform_params.saturation_drive = None;
    }
    audio::transform::apply_dsp_transforms(&mut samples, &transform_params);
    audio::process::normalize_peak(&mut samples, -1.0);

    let sound = generator::save_and_return(
        &samples, &full_prompt, sound_type.as_str(),
        Some("intent-generated"), (intent_profile.energy * 100.0) as i64,
    )?;

    let params_summary = format!(
        "brightness:{:.2} sat:{:.2} click:{:.2} attack:{:.1}ms body:{:.2} sub:{:.2} noise:{:.2} dur:{:.0}ms tail:{:.0}ms",
        coordinated.resynthesis.brightness,
        coordinated.resynthesis.saturation_drive,
        coordinated.resynthesis.click_amount,
        coordinated.resynthesis.attack_ms,
        coordinated.resynthesis.body_gain,
        coordinated.resynthesis.sub_gain,
        coordinated.resynthesis.noise_amount,
        coordinated.resynthesis.duration_ms,
        coordinated.resynthesis.tail_ms,
    );

    Ok(IntentGenerateResult {
        sound,
        profile: intent_profile,
        params_summary,
    })
}

#[tauri::command]
pub async fn get_intent_presets() -> Vec<(&'static str, audio::creative_intent::CreativeIntentProfile)> {
    vec![
        ("neutral", audio::creative_intent::CreativeIntentProfile::default()),
        ("punchy_drum", audio::creative_intent::CreativeIntentProfile::preset("punchy_drum")),
        ("cinematic_boom", audio::creative_intent::CreativeIntentProfile::preset("cinematic_boom")),
        ("lo_fi_warm", audio::creative_intent::CreativeIntentProfile::preset("lo_fi_warm")),
        ("aggressive_dubstep", audio::creative_intent::CreativeIntentProfile::preset("aggressive_dubstep")),
        ("clean_precision", audio::creative_intent::CreativeIntentProfile::preset("clean_precision")),
        ("ambient_texture", audio::creative_intent::CreativeIntentProfile::preset("ambient_texture")),
        ("hard_trap", audio::creative_intent::CreativeIntentProfile::preset("hard_trap")),
        ("experimental_glitch", audio::creative_intent::CreativeIntentProfile::preset("experimental_glitch")),
        ("bass_massive", audio::creative_intent::CreativeIntentProfile::preset("bass_massive")),
        ("folk_acoustic", audio::creative_intent::CreativeIntentProfile::preset("folk_acoustic")),
        ("cyberpunk", audio::creative_intent::CreativeIntentProfile::preset("cyberpunk")),
        ("orchestral_epic", audio::creative_intent::CreativeIntentProfile::preset("orchestral_epic")),
        ("minimal_techno", audio::creative_intent::CreativeIntentProfile::preset("minimal_techno")),
        ("retro_video_game", audio::creative_intent::CreativeIntentProfile::preset("retro_video_game")),
        ("jazz_brush", audio::creative_intent::CreativeIntentProfile::preset("jazz_brush")),
    ]
}

#[tauri::command]
pub async fn blend_intent_profiles(
    profile_a: audio::creative_intent::CreativeIntentProfile,
    profile_b: audio::creative_intent::CreativeIntentProfile,
    blend: f32,
) -> audio::creative_intent::CreativeIntentProfile {
    audio::creative_intent::CreativeIntentProfile::blend(&profile_a, &profile_b, blend)
}

// ─── Advanced Recreation Commands ────────────────────────

#[tauri::command]
pub async fn recreate_advanced_command(
    sound_id: String,
    config: audio::recreate::AdvancedRecreationConfig,
) -> Result<(generator::SoundResult, audio::recreate::SimilarityReport), String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let (recreated, sim) = audio::recreate::recreate_advanced(&samples, &config);
    if recreated.is_empty() {
        return Err("Recreation produced empty audio".to_string());
    }

    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let prompt = format!("advanced recreation: {} (fidelity:{:.1})", config.mode.label(), config.fidelity);
    let sound = generator::save_and_return(
        &recreated, &prompt, &analysis.sound_type_hint,
        Some(&format!("recreate-{}", config.mode.label())), 0,
    )?;
    Ok((sound, sim))
}

// ─── Semantic Query Command ─────────────────────────────

#[tauri::command]
pub async fn parse_natural_language_query(query: String) -> crate::semantic_library::SemanticQuery {
    crate::semantic_library::parse_natural_language_query(&query)
}

#[tauri::command]
pub async fn semantic_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<crate::semantic_library::SimilarSound>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let all = crate::db::list_all_sounds(&conn).map_err(|e| e.to_string())?;
    let parsed = crate::semantic_library::parse_natural_language_query(&query);
    Ok(crate::semantic_library::search_by_semantic_query(&all, &parsed, 20))
}

// ─── Audio Intelligence Commands ─────────────────────────

#[tauri::command]
pub async fn analyze_audio_intelligence(sound_id: String) -> Result<audio::audio_intelligence::AudioIntelligenceReport, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    Ok(audio::audio_intelligence::analyze_intelligence(&samples, &analysis))
}

#[tauri::command]
pub async fn score_for_ranking_command(sound_id: String) -> Result<f32, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    Ok(audio::audio_intelligence::score_for_ranking(&analysis, &samples))
}

#[tauri::command]
pub async fn recommendation_score_command(sound_id: String) -> Result<f32, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    Ok(audio::audio_intelligence::recommendation_score(&analysis, &samples))
}

// ─── Library Optimization Commands ──────────────────────

#[derive(Clone, serde::Serialize)]
pub struct LibraryOptimizationStats {
    pub total_count: i64,
    pub favorite_count: i64,
    pub avg_duration_ms: f64,
    pub unique_types: Vec<(String, i64)>,
    pub total_size_bytes: i64,
    pub estimated_import_count: i64,
}

#[tauri::command]
pub async fn get_library_optimization_stats(state: State<'_, AppState>) -> Result<LibraryOptimizationStats, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let total_count: i64 = db.query_row("SELECT COUNT(*) FROM sounds", [], |r| r.get(0)).unwrap_or(0);
    let favorite_count: i64 = db.query_row("SELECT COUNT(*) FROM sounds WHERE is_favorite=1", [], |r| r.get(0)).unwrap_or(0);
    let avg_duration_ms: f64 = db.query_row("SELECT COALESCE(AVG(duration_ms), 0) FROM sounds", [], |r| r.get(0)).unwrap_or(0.0);
    let total_size: i64 = db.query_row("SELECT COALESCE(SUM(rms * 1000), 0) FROM sounds", [], |r| r.get(0)).unwrap_or(0);
    let estimated_import: i64 = db.query_row("SELECT COUNT(*) FROM imported_samples", [], |r| r.get(0)).unwrap_or(0);

    let mut stmt = db.prepare("SELECT sound_type, COUNT(*) as c FROM sounds WHERE sound_type IS NOT NULL AND sound_type != '' GROUP BY sound_type ORDER BY c DESC").map_err(|e| e.to_string())?;
    let unique_types: Vec<(String, i64)> = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(LibraryOptimizationStats {
        total_count,
        favorite_count,
        avg_duration_ms,
        unique_types,
        total_size_bytes: total_size,
        estimated_import_count: estimated_import,
    })
}

#[tauri::command]
pub async fn get_library_cache(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.query_row(
        "SELECT value FROM library_cache WHERE key = ?1", params![key],
        |r| r.get::<_, String>(0),
    ).ok();
    Ok(result)
}

#[tauri::command]
pub async fn set_library_cache(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO library_cache (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
        params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Workflow Automation Commands ───────────────────────

#[tauri::command]
pub async fn auto_tag_sound(sound_id: String) -> Result<audio::workflow::AutoTags, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    Ok(audio::workflow::infer_tags(&samples, &analysis))
}

#[tauri::command]
pub async fn suggest_pack_command(sound_ids: Vec<String>) -> Result<audio::workflow::PackSuggestion, String> {
    if sound_ids.is_empty() {
        return Err("No sounds provided".to_string());
    }
    let mut analyses = Vec::new();
    for sid in &sound_ids {
        let path = storage::sound_path(sid);
        if path.exists() {
            if let Ok(samples) = audio::read_wav(&path) {
                analyses.push(audio::analyze::analyze_audio(&samples, 44100, 1));
            }
        }
    }
    if analyses.is_empty() {
        return Err("No valid sounds found".to_string());
    }
    Ok(audio::workflow::suggest_pack(&analyses))
}

#[tauri::command]
pub async fn suggest_filename_command(sound_id: String) -> Result<String, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let tags = audio::workflow::infer_tags(&samples, &analysis);
    Ok(audio::workflow::suggest_filename(&analysis, &tags))
}

// ─── Stress Test & Validation Commands ──────────────────

#[tauri::command]
pub async fn run_stress_test(iterations: usize) -> audio::stress_test::StressTestResult {
    audio::stress_test::run_stress_test(iterations)
}

#[tauri::command]
pub async fn validate_export_command(sound_id: String) -> Result<audio::stress_test::ExportValidation, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    Ok(audio::stress_test::validate_export(&samples, 44100))
}

#[tauri::command]
pub async fn batch_validate_exports(sound_ids: Vec<String>) -> Result<Vec<(String, audio::stress_test::ExportValidation)>, String> {
    let mut results = Vec::new();
    for sid in sound_ids {
        let path = storage::sound_path(&sid);
        if path.exists() {
            if let Ok(samples) = audio::read_wav(&path) {
                let validation = audio::stress_test::validate_export(&samples, 44100);
                results.push((sid, validation));
            }
        }
    }
    Ok(results)
}

// ─── Workflow Speed: Quick Branch ───────────────────────

#[tauri::command]
pub async fn quick_branch(
    sound_id: String,
    prompt_modification: Option<String>,
) -> Result<generator::SoundResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);

    let (result_samples, branch_label) = if let Some(mod_prompt) = &prompt_modification {
        let ctrl = crate::prompt_dsp::parse_prompt_rich(mod_prompt);
        let fusion = audio::recreate::recreate_with_prompt(&samples, &ctrl);
        (fusion.fusion_sound, format!("branch:{}", mod_prompt))
    } else {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        let mut p = audio::recreate::params_from_analysis(&analysis, &samples);
        p = p.with_seed(seed as u64).randomize(0.1);
        let branched = audio::resynthesize::resynthesize(&p);
        (branched, "quick-branch".to_string())
    };

    if result_samples.is_empty() {
        return Err("Branch produced empty audio".to_string());
    }

    let prompt = format!("branch of {} - {}", analysis.sound_type_hint, branch_label);
    generator::save_and_return(
        &result_samples, &prompt, &analysis.sound_type_hint,
        Some(&branch_label), 0,
    )
}

#[tauri::command]
pub async fn batch_favorite(
    sound_ids: Vec<String>,
    favorite: bool,
) -> Result<usize, String> {
    use crate::favorites::SoundMetadata;
    let mut store = crate::favorites::FavoritesStore::load();
    let mut count = 0usize;
    for id in &sound_ids {
        if favorite {
            let meta = SoundMetadata {
                id: id.clone(),
                prompt: String::new(),
                sound_type: String::new(),
                duration_ms: 0.0,
                created_at: String::new(),
                source: String::new(),
                model: String::new(),
                seed: 0,
                variant_name: None,
            };
            store.toggle(id, meta);
        } else {
            if store.is_favorited(id) {
                let meta = SoundMetadata {
                    id: id.clone(),
                    prompt: String::new(),
                    sound_type: String::new(),
                    duration_ms: 0.0,
                    created_at: String::new(),
                    source: String::new(),
                    model: String::new(),
                    seed: 0,
                    variant_name: None,
                };
                store.toggle(id, meta);
            }
        }
        count += 1;
    }
    Ok(count)
}

#[tauri::command]
pub async fn batch_export(
    state: State<'_, AppState>,
    sound_ids: Vec<String>,
    export_dir: Option<String>,
) -> Result<Vec<String>, String> {
    let dir = if let Some(d) = export_dir {
        std::path::PathBuf::from(&d)
    } else {
        crate::storage::export_dir()
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut exported = Vec::new();

    for id in &sound_ids {
        let path = storage::sound_path(id);
        if !path.exists() { continue; }
        let samples = audio::read_wav(&path)?;
        let meta = crate::db::get_sound(&db, id)
            .map_err(|e| e.to_string())?
            .unwrap_or(db::SoundEntry {
                id: id.clone(), prompt: String::new(), sound_type: "other".to_string(),
                duration_ms: 0.0, sample_rate: 44100, rms: 0.0, peak: 0.0,
                spectral_centroid: 0.0, tags: "[]".to_string(), is_favorite: false,
                source: String::new(), variant_name: None, created_at: String::new(),
                model: "cshot-engine".to_string(), seed: 0,
            });
        let name = format!("{}-{}.wav", meta.sound_type, &id[..8]);
        let out_path = dir.join(&name);
        audio::write_wav(&out_path, &samples, 44100)?;
        exported.push(out_path.to_string_lossy().to_string());
    }

    Ok(exported)
}

#[tauri::command]
pub async fn get_recent_prompts(limit: Option<usize>) -> Vec<crate::audio::session::RecentEntry> {
    let store = crate::audio::session::RecentsStore::load();
    store.recent_prompts(limit.unwrap_or(20)).into_iter().cloned().collect()
}

// ─── Instrument Preset Commands ─────────────────────────

#[tauri::command]
pub async fn get_builtin_presets() -> Vec<audio::instrument::InstrumentPreset> {
    audio::instrument::InstrumentPreset::builtin_presets()
}

#[tauri::command]
pub async fn apply_preset_macro(
    preset: audio::instrument::InstrumentPreset,
    macro_state: audio::instrument::MorphMacroState,
) -> audio::instrument::InstrumentPreset {
    let params = preset.apply_macro(&macro_state);
    let mut updated = preset;
    updated.params = params;
    updated
}

// ─── Sound Sculpting Commands ───────────────────────────

#[tauri::command]
pub async fn sculpt_sound(
    sound_id: String,
    controls: audio::sculpt::SculptControls,
) -> Result<generator::SoundResult, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let mut samples = audio::read_wav(&path)?;
    audio::sculpt::apply_sculpt(&mut samples, &controls);
    if samples.is_empty() {
        return Err("Sculpting produced empty audio".to_string());
    }
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);
    let prompt = format!("sculpted transient:{:.1} tail:{:.1} brightness:{:.1} distortion:{:.1} density:{:.1} tonal_noise:{:.1}",
        controls.transient_intensity, controls.tail_length, controls.brightness,
        controls.distortion, controls.density, controls.tonal_noise_balance);
    generator::save_and_return(&samples, &prompt, &analysis.sound_type_hint, Some("sculpted"), 0)
}

#[tauri::command]
pub async fn sculpt_preview(
    sound_id: String,
    controls: audio::sculpt::SculptControls,
) -> Result<Vec<f32>, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    Ok(audio::sculpt::generate_sculpt_preview(&samples, &controls))
}

// ─── Evolution Commands ─────────────────────────────────

#[tauri::command]
pub async fn evolve_sound_command(
    state: State<'_, AppState>,
    sound_id: String,
    config: audio::evolution::EvolutionConfig,
    target_direction: Option<String>,
    direction_intensity: Option<f32>,
) -> Result<audio::evolution::EvolutionState, String> {
    let path = storage::sound_path(&sound_id);
    if !path.exists() {
        return Err("Source sound not found".to_string());
    }
    let samples = audio::read_wav(&path)?;
    let analysis = audio::analyze::analyze_audio(&samples, 44100, 1);

    let prompt = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        crate::db::get_sound(&db, &sound_id)
            .map_err(|e| e.to_string())?
            .map(|s| s.prompt)
            .unwrap_or_else(|| "evolved sound".to_string())
    };

    let request = audio::evolution::EvolveStepRequest {
        parent_samples: samples,
        parent_analysis: analysis,
        parent_prompt: prompt,
        parent_id: sound_id,
        config,
        target_direction,
        direction_intensity: direction_intensity.unwrap_or(0.5),
    };

    let state = audio::evolution::run_evolution(&request);
    Ok(state)
}

#[tauri::command]
pub async fn save_evolution_member(
    member: audio::evolution::EvolutionMember,
    sound_type: String,
    prompt: String,
) -> Result<generator::SoundResult, String> {
    if member.samples.is_empty() {
        return Err("Evolution member has no audio data".to_string());
    }
    let label = if member.is_elite { "evolution-elite" } else { "evolution-member" };
    generator::save_and_return(
        &member.samples, &prompt, &sound_type,
        Some(label), member.generation as i64,
    )
}

// ─── OneShot API ───────────────────────────────────────────────────

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SoundClassInfo {
    pub value: String,
    pub label: String,
    pub description: String,
}

#[derive(Clone, serde::Serialize)]
pub struct OneshotPreviewResult {
    pub samples: Vec<f32>,
    pub duration_ms: f32,
    pub peak: f32,
    pub rms: f32,
    pub sample_rate: u32,
}

#[derive(Clone, serde::Serialize)]
pub struct OneshotExportResult {
    pub path: String,
    pub filename: String,
    pub file_size_bytes: u64,
}

#[tauri::command]
pub async fn list_sound_classes() -> Vec<SoundClassInfo> {
    vec![
        SoundClassInfo { value: "808".into(), label: "808".into(), description: "Classic 808 kick/sub — deep sub-bass with slow pitch drop".into() },
        SoundClassInfo { value: "kick".into(), label: "Kick".into(), description: "Standard kick drum — punchy attack, fast decay, low body".into() },
        SoundClassInfo { value: "snare".into(), label: "Snare".into(), description: "Snare drum — noise burst + mid body tone".into() },
        SoundClassInfo { value: "clap".into(), label: "Clap".into(), description: "Hand clap — staggered noise bursts, short room tail".into() },
        SoundClassInfo { value: "closed_hat".into(), label: "Closed Hat".into(), description: "Closed hi-hat — short metallic tick, extreme high-pass".into() },
        SoundClassInfo { value: "open_hat".into(), label: "Open Hat".into(), description: "Open hi-hat — longer metallic decay, bright tail".into() },
        SoundClassInfo { value: "bass_stab".into(), label: "Bass Stab".into(), description: "Bass stab — short pitched bass hit, filter movement".into() },
        SoundClassInfo { value: "impact_fx".into(), label: "Impact FX".into(), description: "Cinematic impact — sub drop + noise burst + evolving tail".into() },
        SoundClassInfo { value: "synth_stab".into(), label: "Synth Stab".into(), description: "Synth stab — detuned oscillators, chord-like body".into() },
    ]
}

#[tauri::command]
pub async fn get_oneshot_defaults(sound_class: String) -> Result<crate::audio::spec::OneShotSpec, String> {
    let class = crate::audio::spec::SoundClass::from_str(&sound_class);
    Ok(match class {
        crate::audio::spec::SoundClass::Sub808 => crate::audio::spec::OneShotSpec::preset_808(),
        crate::audio::spec::SoundClass::Kick => crate::audio::spec::OneShotSpec::preset_kick(),
        crate::audio::spec::SoundClass::Snare => crate::audio::spec::OneShotSpec::preset_snare(),
        crate::audio::spec::SoundClass::Clap => crate::audio::spec::OneShotSpec::preset_clap(),
        crate::audio::spec::SoundClass::ClosedHat => crate::audio::spec::OneShotSpec::preset_closed_hat(),
        crate::audio::spec::SoundClass::OpenHat => crate::audio::spec::OneShotSpec::preset_open_hat(),
        crate::audio::spec::SoundClass::BassStab => crate::audio::spec::OneShotSpec::preset_bass_stab(),
        crate::audio::spec::SoundClass::ImpactFx => crate::audio::spec::OneShotSpec::preset_impact_fx(),
        crate::audio::spec::SoundClass::SynthStab => crate::audio::spec::OneShotSpec::preset_synth_stab(),
    })
}

#[tauri::command]
pub async fn render_oneshot(
    sound_class: String,
    duration_ms: f32,
    pitch_hz: f32,
    gain: f32,
    controls: Option<crate::audio::one_shot_controls::OneShotControls>,
) -> Result<OneshotPreviewResult, String> {
    let class = crate::audio::spec::SoundClass::from_str(&sound_class);
    let spec = crate::audio::spec::OneShotSpec {
        sound_class: class,
        duration_ms,
        pitch_hz,
        gain,
        controls,
    };
    let samples = spec.render();
    if samples.is_empty() {
        return Err("Rendered audio is empty".to_string());
    }
    if samples.iter().any(|s| s.is_nan() || s.is_infinite()) {
        return Err("Rendered audio contains NaN/Inf".to_string());
    }
    let duration_ms_actual = samples.len() as f32 / crate::audio::SAMPLE_RATE as f32 * 1000.0;
    let peak = crate::audio::compute_peak(&samples);
    let rms = crate::audio::compute_rms(&samples);
    Ok(OneshotPreviewResult {
        samples,
        duration_ms: duration_ms_actual,
        peak,
        rms,
        sample_rate: crate::audio::SAMPLE_RATE,
    })
}

#[tauri::command]
pub async fn export_oneshot_wav(
    sound_class: String,
    duration_ms: f32,
    pitch_hz: f32,
    gain: f32,
    controls: Option<crate::audio::one_shot_controls::OneShotControls>,
    output_path: String,
) -> Result<OneshotExportResult, String> {
    let class = crate::audio::spec::SoundClass::from_str(&sound_class);
    let spec = crate::audio::spec::OneShotSpec {
        sound_class: class,
        duration_ms,
        pitch_hz,
        gain,
        controls,
    };
    let path = std::path::PathBuf::from(&output_path);
    crate::audio::spec::render_to_wav(&spec, &path)?;
    let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let filename = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(OneshotExportResult {
        path: path.to_string_lossy().to_string(),
        filename,
        file_size_bytes: file_size,
    })
}