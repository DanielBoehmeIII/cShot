use std::collections::HashSet;
use std::fs;

use crate::audio;
use crate::db;
use crate::storage;

pub struct CleanupResult {
    pub deleted_sounds: usize,
    pub deleted_orphans: usize,
    pub deleted_failed: usize,
    pub errors: Vec<String>,
    pub favorites_protected: usize,
}

/// Clear all generated sounds (non-favorite, non-exported).
/// Requires explicit `include_favorites: false` to protect favorites.
pub fn clear_generated_sounds(include_favorites: bool) -> Result<CleanupResult, String> {
    let app_root = storage::app_root();
    let db_path = storage::database_path();
    let audio_dir = storage::audio_dir();

    if !app_root.exists() {
        return Err("cShot data directory not found. Nothing to clean.".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    let all = db::list_all_sounds(&conn).map_err(|e| format!("Could not read library: {}", e))?;

    let mut result = CleanupResult {
        deleted_sounds: 0,
        deleted_orphans: 0,
        deleted_failed: 0,
        errors: Vec::new(),
        favorites_protected: 0,
    };

    for sound in &all {
        if sound.is_favorite && !include_favorites {
            result.favorites_protected += 1;
            continue;
        }

        let audio_path = audio_dir.join(format!("{}.wav", sound.id));
        if audio_path.exists() {
            if let Err(e) = fs::remove_file(&audio_path) {
                result.errors.push(format!("Could not delete {}: {}", sound.id, e));
            }
        }

        if let Err(e) = db::delete_sound(&conn, &sound.id) {
            result.errors.push(format!("Could not remove {} from DB: {}", sound.id, e));
        }

        result.deleted_sounds += 1;
    }

    if let Err(e) = conn.execute("VACUUM", []) {
        result.errors.push(format!("Vacuum failed: {}", e));
    }

    Ok(result)
}

/// Delete sounds that have failure labels (failed generations).
pub fn clear_failed_jobs() -> Result<CleanupResult, String> {
    let db_path = storage::database_path();
    let audio_dir = storage::audio_dir();

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    let all = db::list_all_sounds(&conn).map_err(|e| format!("Could not read library: {}", e))?;

    let mut result = CleanupResult {
        deleted_sounds: 0,
        deleted_orphans: 0,
        deleted_failed: 0,
        errors: Vec::new(),
        favorites_protected: 0,
    };

    for sound in &all {
        let tags: Vec<String> =
            serde_json::from_str(&sound.tags).unwrap_or_default();
        let has_failure = tags.iter().any(|t| {
            matches!(
                t.as_str(),
                "silent" | "clipped" | "too quiet" | "too long" | "duration"
            )
        });

        if !has_failure {
            continue;
        }

        if sound.is_favorite {
            result.favorites_protected += 1;
            continue;
        }

        let audio_path = audio_dir.join(format!("{}.wav", sound.id));
        if audio_path.exists() {
            fs::remove_file(&audio_path).ok();
        }

        if db::delete_sound(&conn, &sound.id).is_ok() {
            result.deleted_failed += 1;
            result.deleted_sounds += 1;
        }
    }

    Ok(result)
}

/// Find and delete WAV files in the audio directory that have no database entry.
pub fn clear_orphaned_files() -> Result<CleanupResult, String> {
    let db_path = storage::database_path();
    let audio_dir = storage::audio_dir();

    if !audio_dir.exists() {
        return Ok(CleanupResult {
            deleted_sounds: 0,
            deleted_orphans: 0,
            deleted_failed: 0,
            errors: Vec::new(),
            favorites_protected: 0,
        });
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    let all = db::list_all_sounds(&conn).map_err(|e| format!("Could not read library: {}", e))?;
    let known_ids: HashSet<String> = all.iter().map(|s| format!("{}.wav", s.id)).collect();

    let entries = fs::read_dir(&audio_dir).map_err(|e| format!("Could not read audio dir: {}", e))?;

    let mut result = CleanupResult {
        deleted_sounds: 0,
        deleted_orphans: 0,
        deleted_failed: 0,
        errors: Vec::new(),
        favorites_protected: 0,
    };

    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("wav") {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if !known_ids.contains(&filename) {
            if let Err(e) = fs::remove_file(&path) {
                result
                    .errors
                    .push(format!("Could not delete orphan {}: {}", filename, e));
            } else {
                result.deleted_orphans += 1;
            }
        }
    }

    Ok(result)
}

/// Reset the database: backup existing, then re-create schema.
pub fn reset_database(dry_run: bool) -> Result<String, String> {
    let db_path = storage::database_path();

    if !db_path.exists() {
        return Err("Database not found. Nothing to reset.".to_string());
    }

    let backup_path = db_path.with_extension("db.backup");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let backup_path = backup_path.with_extension(format!("db.backup.{}", timestamp));

    if dry_run {
        return Ok(format!(
            "Would backup to: {}\nWould re-create database schema.",
            backup_path.display()
        ));
    }

    fs::copy(&db_path, &backup_path)
        .map_err(|e| format!("Backup failed: {}. Reset aborted.", e))?;

    fs::remove_file(&db_path).ok();
    fs::remove_file(db_path.with_extension("db-wal")).ok();
    fs::remove_file(db_path.with_extension("db-shm")).ok();

    let conn = crate::db::init_database(&db_path)
        .map_err(|e| format!("Could not re-create database: {}", e))?;

    let count = conn
        .query_row("SELECT COUNT(*) FROM sounds", [], |row| row.get::<_, i64>(0))
        .unwrap_or(0);

    Ok(format!(
        "Database reset complete.\nBackup saved to: {}\nNew database initialized ({} sounds).",
        backup_path.display(),
        count
    ))
}

/// Rebuild metadata by scanning audio directory for WAV files
/// and matching them to database entries.
pub fn rebuild_metadata_from_storage() -> Result<CleanupResult, String> {
    let db_path = storage::database_path();
    let audio_dir = storage::audio_dir();

    if !audio_dir.exists() {
        return Err("Audio storage directory not found.".to_string());
    }

    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Could not open database: {}", e))?;

    let all = db::list_all_sounds(&conn).map_err(|e| format!("Could not read library: {}", e))?;
    let known_ids: HashSet<String> = all.iter().map(|s| s.id.clone()).collect();

    let entries = fs::read_dir(&audio_dir).map_err(|e| format!("Could not read audio dir: {}", e))?;

    let mut result = CleanupResult {
        deleted_sounds: 0,
        deleted_orphans: 0,
        deleted_failed: 0,
        errors: Vec::new(),
        favorites_protected: 0,
    };

    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read entry: {}", e))?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("wav") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if known_ids.contains(&stem) {
            continue;
        }

        let samples = match audio::read_wav(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let rms = audio::compute_rms(&samples);
        let peak = audio::compute_peak(&samples);
        let spectral_centroid = audio::compute_spectral_centroid(&samples);
        let duration_ms = samples.len() as f32 / 44100.0 * 1000.0;

        let entry = db::SoundEntry {
            id: stem,
            prompt: String::new(),
            sound_type: "other".to_string(),
            duration_ms,
            sample_rate: 44100,
            rms,
            peak,
            spectral_centroid,
            tags: "[]".to_string(),
            is_favorite: false,
            source: "recovered".to_string(),
            variant_name: None,
            model: "unknown".to_string(),
            seed: 0,
            created_at: String::new(),
        };

        if db::insert_sound(&conn, &entry).is_ok() {
            result.deleted_sounds += 1;
        }
    }

    Ok(result)
}

/// Development-only: delete everything including favorites.
#[cfg(debug_assertions)]
pub fn clear_everything() -> Result<CleanupResult, String> {
    let audio_dir = storage::audio_dir();

    let conn = rusqlite::Connection::open(storage::database_path())
        .map_err(|e| format!("Could not open database: {}", e))?;

    conn.execute("DELETE FROM sounds", [])
        .map_err(|e| format!("Could not clear database: {}", e))?;

    if audio_dir.exists() {
        for entry in fs::read_dir(&audio_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            fs::remove_file(entry.path()).ok();
        }
    }

    conn.execute("VACUUM", []).ok();

    Ok(CleanupResult {
        deleted_sounds: 0,
        deleted_orphans: 0,
        deleted_failed: 0,
        errors: Vec::new(),
        favorites_protected: 0,
    })
}
