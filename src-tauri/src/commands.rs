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
pub async fn get_active_provider(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let registry = state.provider_registry.lock().map_err(|e| e.to_string())?;
    Ok(registry.active_provider_name()
        .unwrap_or_else(|| "mock-dsp".to_string()))
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

/// Check if any non-mock provider has its API key configured.
/// This gives a clear error early instead of failing during fallback.
fn check_provider_keys() -> Result<(), String> {
    let registry = crate::generation::build_default_registry();
    let has_real_provider = registry.available_providers()
        .iter()
        .any(|p| p.name() != "mock-dsp" && p.is_available());

    if !has_real_provider {
        let reasons: Vec<String> = registry.available_providers()
            .iter()
            .filter(|p| p.name() != "mock-dsp")
            .filter_map(|p| p.reason_unavailable())
            .collect();

        if !reasons.is_empty() {
            return Err(format!(
                "No generation providers configured. {}. Set up a provider in .env or use mock-dsp.",
                reasons.join("; ")
            ));
        }
    }
    Ok(())
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
        return Err(format!(
            "The sound file could not be found on disk. It may have been deleted or moved. Try generating the sound again."
        ));
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