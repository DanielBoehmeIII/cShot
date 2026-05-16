use std::collections::HashSet;
use std::fs;

use crate::audio;
use crate::db;
use crate::storage;

#[derive(Clone, serde::Serialize)]
pub struct IntegrityReport {
    pub total_db_sounds: usize,
    pub total_disk_files: usize,
    pub missing_files: Vec<String>,
    pub orphan_files: Vec<String>,
    pub hash_mismatches: Vec<String>,
    pub db_entries_without_hash: usize,
}

pub fn compute_sha256(samples: &[f32]) -> String {
    use sha2::{Digest, Sha256};
    let bytes: Vec<u8> = samples
        .iter()
        .flat_map(|s| s.to_le_bytes())
        .collect();
    let hash = Sha256::digest(&bytes);
    format!("{:x}", hash)
}

pub fn scan_integrity() -> Result<IntegrityReport, String> {
    let db_path = storage::database_path();
    let audio_dir = storage::audio_dir();

    let conn =
        rusqlite::Connection::open(&db_path).map_err(|e| format!("Could not open DB: {}", e))?;

    let all = db::list_all_sounds(&conn).map_err(|e| format!("Could not read library: {}", e))?;

    let mut missing_files = Vec::new();
    let mut hash_mismatches = Vec::new();
    let mut db_entries_without_hash = 0;

    for sound in &all {
        let audio_path = audio_dir.join(format!("{}.wav", sound.id));

        if !audio_path.exists() {
            missing_files.push(sound.id.clone());
            continue;
        }

        match audio::read_wav(&audio_path) {
            Ok(samples) => {
                let hash = compute_sha256(&samples);
                let stored_hash = get_stored_hash(&conn, &sound.id).unwrap_or_default();

                if stored_hash.is_empty() {
                    db_entries_without_hash += 1;
                } else if stored_hash != hash {
                    hash_mismatches.push(sound.id.clone());
                }
            }
            Err(_) => {
                missing_files.push(format!("{} (corrupted)", sound.id));
            }
        }
    }

    let known_ids: HashSet<String> = all.iter().map(|s| s.id.clone()).collect();

    let mut orphan_files = Vec::new();
    if audio_dir.exists() {
        for entry in fs::read_dir(&audio_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("wav") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if !known_ids.contains(&stem) {
                orphan_files.push(stem);
            }
        }
    }

    let total_disk_files = if audio_dir.exists() {
        fs::read_dir(&audio_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            == Some("wav")
                    })
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };

    Ok(IntegrityReport {
        total_db_sounds: all.len(),
        total_disk_files,
        missing_files,
        orphan_files,
        hash_mismatches,
        db_entries_without_hash,
    })
}

/// Update/recompute checksums for all sounds in DB that lack one.
pub fn update_missing_hashes() -> Result<usize, String> {
    let db_path = storage::database_path();
    let audio_dir = storage::audio_dir();

    let conn =
        rusqlite::Connection::open(&db_path).map_err(|e| format!("Could not open DB: {}", e))?;

    let all = db::list_all_sounds(&conn).map_err(|e| format!("Could not read library: {}", e))?;
    let mut updated = 0;

    for sound in &all {
        let stored_hash = get_stored_hash(&conn, &sound.id).unwrap_or_default();
        if !stored_hash.is_empty() {
            continue;
        }

        let audio_path = audio_dir.join(format!("{}.wav", sound.id));
        if !audio_path.exists() {
            continue;
        }

        if let Ok(samples) = audio::read_wav(&audio_path) {
            let hash = compute_sha256(&samples);
            if set_stored_hash(&conn, &sound.id, &hash).is_ok() {
                updated += 1;
            }
        }
    }

    Ok(updated)
}

pub fn store_hash_for_new_sound(sound_id: &str, samples: &[f32]) {
    let db_path = storage::database_path();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let hash = compute_sha256(samples);
        let _ = conn.execute(
            "UPDATE sounds SET file_hash = ?1 WHERE id = ?2",
            rusqlite::params![hash, sound_id],
        );
    }
}

fn get_stored_hash(conn: &rusqlite::Connection, sound_id: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT file_hash FROM sounds WHERE id = ?1",
        rusqlite::params![sound_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .map_err(|e| e.to_string())
    .map(|opt| opt.unwrap_or_default())
}

fn set_stored_hash(
    conn: &rusqlite::Connection,
    sound_id: &str,
    hash: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE sounds SET file_hash = ?1 WHERE id = ?2",
        rusqlite::params![hash, sound_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
