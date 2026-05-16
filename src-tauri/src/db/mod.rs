use rusqlite::{Connection, Result as SqlResult, params};
use std::path::Path;

const SCHEMA_VERSION: i32 = 6;

pub fn init_database(db_path: &Path) -> SqlResult<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    create_schema(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> SqlResult<()> {
    let current_version: i32 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |r| r.get(0))
        .unwrap_or(0);

    if current_version >= SCHEMA_VERSION {
        return Ok(());
    }

    if current_version < 2 {
        let _ = conn.execute_batch("ALTER TABLE sounds ADD COLUMN file_hash TEXT DEFAULT ''");
        conn.execute("INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (2, datetime('now'))", [])?;
    }
    if current_version < 3 {
        let _ = conn.execute_batch("ALTER TABLE sounds ADD COLUMN parent_id TEXT DEFAULT ''");
        conn.execute("INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (3, datetime('now'))", [])?;
    }
    if current_version < 4 {
        let _ = conn.execute_batch("ALTER TABLE sounds ADD COLUMN provider_version TEXT DEFAULT ''");
        conn.execute("INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (4, datetime('now'))", [])?;
    }
    if current_version < 5 {
        let _ = conn.execute_batch("ALTER TABLE exports ADD COLUMN filename TEXT DEFAULT ''");
        conn.execute("INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (5, datetime('now'))", [])?;
    }
    if current_version < 6 {
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS recipes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                prompt_template TEXT NOT NULL DEFAULT '',
                description TEXT DEFAULT '',
                category TEXT DEFAULT '',
                tags TEXT DEFAULT '[]',
                transform_defaults TEXT DEFAULT '{}',
                is_builtin INTEGER DEFAULT 0,
                usage_count INTEGER DEFAULT 0,
                is_favorite INTEGER DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS imported_samples (
                id TEXT PRIMARY KEY,
                original_path TEXT NOT NULL,
                filename TEXT NOT NULL,
                format TEXT DEFAULT 'wav',
                duration_ms REAL,
                sample_rate INTEGER,
                channels INTEGER,
                file_size_bytes INTEGER,
                rms REAL,
                peak REAL,
                tags TEXT DEFAULT '[]',
                notes TEXT DEFAULT '',
                is_favorite INTEGER DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ALTER TABLE sounds ADD COLUMN notes TEXT DEFAULT '';
            ALTER TABLE packs ADD COLUMN notes TEXT DEFAULT '';"
        ).ok();
        let _ = conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_sounds_source ON sounds(source)");
        let _ = conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_sounds_model ON sounds(model)");
        let _ = conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_packs_category ON packs(tags)");
        conn.execute("INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (6, datetime('now'))", [])?;
    }

    Ok(())
}

fn create_schema(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sounds (
            id TEXT PRIMARY KEY,
            prompt TEXT NOT NULL,
            sound_type TEXT,
            duration_ms REAL NOT NULL,
            sample_rate INTEGER NOT NULL DEFAULT 44100,
            rms REAL,
            peak REAL,
            spectral_centroid REAL,
            tags TEXT DEFAULT '[]',
            is_favorite INTEGER DEFAULT 0,
            source TEXT DEFAULT 'generated',
            reference_path TEXT,
            variant_name TEXT,
            model TEXT DEFAULT 'mock-dsp',
            seed INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS exports (
            id TEXT PRIMARY KEY,
            sound_id TEXT NOT NULL REFERENCES sounds(id) ON DELETE CASCADE,
            file_path TEXT NOT NULL,
            format TEXT NOT NULL DEFAULT 'wav',
            sample_rate INTEGER NOT NULL DEFAULT 44100,
            bit_depth INTEGER NOT NULL DEFAULT 24,
            file_size_bytes INTEGER NOT NULL DEFAULT 0,
            exported_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS packs (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT DEFAULT '',
            tags TEXT DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS pack_sounds (
            pack_id TEXT NOT NULL REFERENCES packs(id) ON DELETE CASCADE,
            sound_id TEXT NOT NULL REFERENCES sounds(id) ON DELETE CASCADE,
            sort_order INTEGER NOT NULL DEFAULT 0,
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (pack_id, sound_id)
        );

        CREATE TABLE IF NOT EXISTS embeddings (
            sound_id TEXT PRIMARY KEY REFERENCES sounds(id) ON DELETE CASCADE,
            vector BLOB NOT NULL,
            provider TEXT NOT NULL DEFAULT 'mock-embedding',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_sounds_created ON sounds(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_sounds_favorite ON sounds(is_favorite);
        CREATE INDEX IF NOT EXISTS idx_exports_sound ON exports(sound_id);
        "
    )?;

    let has_model: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('sounds') WHERE name='model'")
        .and_then(|mut s| s.query_row([], |r| r.get::<_, i32>(0)))
        .map(|c| c > 0)
        .unwrap_or(false);
    if !has_model {
        conn.execute_batch("ALTER TABLE sounds ADD COLUMN model TEXT DEFAULT 'mock-dsp';").ok();
    }
    let has_seed: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('sounds') WHERE name='seed'")
        .and_then(|mut s| s.query_row([], |r| r.get::<_, i32>(0)))
        .map(|c| c > 0)
        .unwrap_or(false);
    if !has_seed {
        conn.execute_batch("ALTER TABLE sounds ADD COLUMN seed INTEGER DEFAULT 0;").ok();
    }

    Ok(())
}

pub fn insert_sound(conn: &Connection, entry: &SoundEntry) -> SqlResult<String> {
    conn.execute(
        "INSERT INTO sounds (id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, source, variant_name, model, seed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            entry.id,
            entry.prompt,
            entry.sound_type,
            entry.duration_ms,
            entry.sample_rate,
            entry.rms,
            entry.peak,
            entry.spectral_centroid,
            entry.tags,
            entry.source,
            entry.variant_name,
            entry.model,
            entry.seed,
        ],
    )?;
    Ok(entry.id.clone())
}

pub fn get_sound(conn: &Connection, sound_id: &str) -> SqlResult<Option<SoundEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, is_favorite, source, variant_name, model, seed, created_at
         FROM sounds WHERE id = ?1"
    )?;
    let mut rows = stmt.query(params![sound_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_recent_sounds(conn: &Connection, limit: usize) -> SqlResult<Vec<SoundEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, is_favorite, source, variant_name, model, seed, created_at
         FROM sounds ORDER BY created_at DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn search_sounds(conn: &Connection, query: &str) -> SqlResult<Vec<SoundEntry>> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, is_favorite, source, variant_name, model, seed, created_at
         FROM sounds WHERE prompt LIKE ?1 OR sound_type LIKE ?1 OR tags LIKE ?1 OR source LIKE ?1 OR model LIKE ?1 OR variant_name LIKE ?1 OR COALESCE(notes,'') LIKE ?1
         ORDER BY created_at DESC LIMIT 50"
    )?;
    let rows = stmt.query_map(params![pattern], |row| {
        Ok(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn toggle_favorite(conn: &Connection, sound_id: &str) -> SqlResult<bool> {
    let current: bool = conn.query_row(
        "SELECT is_favorite FROM sounds WHERE id = ?1",
        params![sound_id],
        |row| row.get::<_, i32>(0).map(|v| v != 0),
    ).unwrap_or(false);

    let new_val = !current;
    conn.execute(
        "UPDATE sounds SET is_favorite = ?1 WHERE id = ?2",
        params![new_val as i32, sound_id],
    )?;
    Ok(new_val)
}

pub fn get_favorites(conn: &Connection) -> SqlResult<Vec<SoundEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, is_favorite, source, variant_name, model, seed, created_at
         FROM sounds WHERE is_favorite = 1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn delete_sound(conn: &Connection, sound_id: &str) -> SqlResult<bool> {
    let affected = conn.execute("DELETE FROM sounds WHERE id = ?1", params![sound_id])?;
    Ok(affected > 0)
}

pub fn list_sounds_paginated(conn: &Connection, limit: usize, offset: usize) -> SqlResult<Vec<SoundEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, is_favorite, source, variant_name, model, seed, created_at
         FROM sounds ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
    )?;
    let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
        Ok(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn count_sounds(conn: &Connection) -> SqlResult<i64> {
    conn.query_row("SELECT COUNT(*) FROM sounds", [], |row| row.get(0))
}

pub fn count_favorites(conn: &Connection) -> SqlResult<i64> {
    conn.query_row("SELECT COUNT(*) FROM sounds WHERE is_favorite = 1", [], |row| row.get(0))
}

pub fn list_favorites_paginated(conn: &Connection, limit: usize, offset: usize) -> SqlResult<Vec<SoundEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, is_favorite, source, variant_name, model, seed, created_at
         FROM sounds WHERE is_favorite = 1 ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
    )?;
    let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
        Ok(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn list_all_sounds(conn: &Connection) -> SqlResult<Vec<SoundEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, is_favorite, source, variant_name, model, seed, created_at
         FROM sounds ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn upsert_embedding(conn: &Connection, sound_id: &str, vector: &[f32], provider: &str) -> SqlResult<()> {
    let blob: Vec<u8> = vector.iter().flat_map(|f| f.to_le_bytes()).collect();
    conn.execute(
        "INSERT OR REPLACE INTO embeddings (sound_id, vector, provider) VALUES (?1, ?2, ?3)",
        params![sound_id, blob, provider],
    )?;
    Ok(())
}

pub fn get_embedding(conn: &Connection, sound_id: &str) -> SqlResult<Option<(Vec<f32>, String)>> {
    let mut stmt = conn.prepare("SELECT vector, provider FROM embeddings WHERE sound_id = ?1")?;
    let mut rows = stmt.query(params![sound_id])?;
    if let Some(row) = rows.next()? {
        let blob: Vec<u8> = row.get(0)?;
        let provider: String = row.get(1)?;
        let vector: Vec<f32> = blob.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
        Ok(Some((vector, provider)))
    } else {
        Ok(None)
    }
}

pub fn list_all_embeddings(conn: &Connection) -> SqlResult<Vec<(String, Vec<f32>, String)>> {
    let mut stmt = conn.prepare("SELECT sound_id, vector, provider FROM embeddings")?;
    let rows = stmt.query_map([], |row| {
        let sound_id: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        let provider: String = row.get(2)?;
        let vector: Vec<f32> = blob.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
        Ok((sound_id, vector, provider))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn create_pack(conn: &Connection, id: &str, title: &str, description: &str, tags: &[String]) -> SqlResult<()> {
    let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO packs (id, title, description, tags) VALUES (?1, ?2, ?3, ?4)",
        params![id, title, description, tags_json],
    )?;
    Ok(())
}

pub fn delete_pack(conn: &Connection, pack_id: &str) -> SqlResult<bool> {
    let affected = conn.execute("DELETE FROM packs WHERE id = ?1", params![pack_id])?;
    Ok(affected > 0)
}

pub fn list_packs(conn: &Connection) -> SqlResult<Vec<PackEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, tags, created_at, updated_at,
         (SELECT COUNT(*) FROM pack_sounds WHERE pack_id = packs.id) as sound_count
         FROM packs ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PackEntry {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            tags: row.get(3)?,
            sound_count: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn add_sound_to_pack(conn: &Connection, pack_id: &str, sound_id: &str, sort_order: i32) -> SqlResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO pack_sounds (pack_id, sound_id, sort_order) VALUES (?1, ?2, ?3)",
        params![pack_id, sound_id, sort_order],
    )?;
    conn.execute("UPDATE packs SET updated_at = datetime('now') WHERE id = ?1", params![pack_id])?;
    Ok(())
}

pub fn remove_sound_from_pack(conn: &Connection, pack_id: &str, sound_id: &str) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM pack_sounds WHERE pack_id = ?1 AND sound_id = ?2",
        params![pack_id, sound_id],
    )?;
    conn.execute("UPDATE packs SET updated_at = datetime('now') WHERE id = ?1", params![pack_id])?;
    Ok(())
}

pub fn get_pack_sounds(conn: &Connection, pack_id: &str) -> SqlResult<Vec<SoundEntry>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.prompt, s.sound_type, s.duration_ms, s.sample_rate, s.rms, s.peak, s.spectral_centroid, s.tags, s.is_favorite, s.source, s.variant_name, s.model, s.seed, s.created_at
         FROM sounds s
         JOIN pack_sounds ps ON s.id = ps.sound_id
         WHERE ps.pack_id = ?1
         ORDER BY ps.sort_order ASC"
    )?;
    let rows = stmt.query_map(params![pack_id], |row| {
        Ok(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn reorder_pack_sounds(conn: &Connection, pack_id: &str, sound_ids: &[String]) -> SqlResult<()> {
    for (i, sound_id) in sound_ids.iter().enumerate() {
        conn.execute(
            "UPDATE pack_sounds SET sort_order = ?1 WHERE pack_id = ?2 AND sound_id = ?3",
            params![i as i32, pack_id, sound_id],
        )?;
    }
    Ok(())
}

pub fn update_pack_metadata(conn: &Connection, pack_id: &str, title: &str, description: &str, tags: &[String]) -> SqlResult<()> {
    let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "UPDATE packs SET title = ?1, description = ?2, tags = ?3, updated_at = datetime('now') WHERE id = ?4",
        params![title, description, tags_json, pack_id],
    )?;
    Ok(())
}

pub fn log_export(
    conn: &Connection,
    id: &str,
    sound_id: &str,
    file_path: &str,
    file_size_bytes: i64,
) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO exports (id, sound_id, file_path, file_size_bytes) VALUES (?1, ?2, ?3, ?4)",
        params![id, sound_id, file_path, file_size_bytes],
    )?;
    Ok(())
}

// ─── Recipe Functions ────────────────────────────────

pub fn create_recipe(conn: &Connection, entry: &RecipeEntry) -> SqlResult<String> {
    let tags_json = serde_json::to_string(&entry.tags).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO recipes (id, title, prompt_template, description, category, tags, transform_defaults, is_builtin)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![entry.id, entry.title, entry.prompt_template, entry.description, entry.category, tags_json, entry.transform_defaults, entry.is_builtin],
    )?;
    Ok(entry.id.clone())
}

pub fn update_recipe(conn: &Connection, entry: &RecipeEntry) -> SqlResult<()> {
    let tags_json = serde_json::to_string(&entry.tags).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "UPDATE recipes SET title=?1, prompt_template=?2, description=?3, category=?4, tags=?5, transform_defaults=?6, updated_at=datetime('now') WHERE id=?7",
        params![entry.title, entry.prompt_template, entry.description, entry.category, tags_json, entry.transform_defaults, entry.id],
    )?;
    Ok(())
}

pub fn delete_recipe(conn: &Connection, recipe_id: &str) -> SqlResult<bool> {
    let affected = conn.execute("DELETE FROM recipes WHERE id=?1 AND is_builtin=0", params![recipe_id])?;
    Ok(affected > 0)
}

pub fn list_recipes(conn: &Connection) -> SqlResult<Vec<RecipeEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, prompt_template, description, category, tags, transform_defaults, is_builtin, usage_count, is_favorite, created_at, updated_at
         FROM recipes ORDER BY is_favorite DESC, usage_count DESC, updated_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        let tags_str: String = row.get(5)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        Ok(RecipeEntry {
            id: row.get(0)?,
            title: row.get(1)?,
            prompt_template: row.get(2)?,
            description: row.get(3)?,
            category: row.get(4)?,
            tags,
            transform_defaults: row.get(6)?,
            is_builtin: row.get::<_, i32>(7)? != 0,
            usage_count: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows { results.push(row?); }
    Ok(results)
}

pub fn search_recipes(conn: &Connection, query: &str) -> SqlResult<Vec<RecipeEntry>> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, title, prompt_template, description, category, tags, transform_defaults, is_builtin, usage_count, is_favorite, created_at, updated_at
         FROM recipes WHERE title LIKE ?1 OR description LIKE ?1 OR category LIKE ?1 OR tags LIKE ?1
         ORDER BY is_favorite DESC, usage_count DESC LIMIT 50"
    )?;
    let rows = stmt.query_map(params![pattern], |row| {
        let tags_str: String = row.get(5)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        Ok(RecipeEntry {
            id: row.get(0)?,
            title: row.get(1)?,
            prompt_template: row.get(2)?,
            description: row.get(3)?,
            category: row.get(4)?,
            tags,
            transform_defaults: row.get(6)?,
            is_builtin: row.get::<_, i32>(7)? != 0,
            usage_count: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows { results.push(row?); }
    Ok(results)
}

pub fn toggle_recipe_favorite(conn: &Connection, recipe_id: &str) -> SqlResult<bool> {
    let current: bool = conn.query_row(
        "SELECT is_favorite FROM recipes WHERE id = ?1",
        params![recipe_id],
        |row| row.get::<_, i32>(0).map(|v| v != 0),
    ).unwrap_or(false);
    let new_val = !current;
    conn.execute("UPDATE recipes SET is_favorite=?1, updated_at=datetime('now') WHERE id=?2", params![new_val as i32, recipe_id])?;
    Ok(new_val)
}

pub fn increment_recipe_usage(conn: &Connection, recipe_id: &str) -> SqlResult<()> {
    conn.execute("UPDATE recipes SET usage_count = usage_count + 1, updated_at = datetime('now') WHERE id=?1", params![recipe_id])?;
    Ok(())
}

// ─── Imported Samples Functions ───────────────────────

pub fn insert_imported_sample(conn: &Connection, entry: &ImportedSampleEntry) -> SqlResult<String> {
    let tags_json = serde_json::to_string(&entry.tags).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO imported_samples (id, original_path, filename, format, duration_ms, sample_rate, channels, file_size_bytes, rms, peak, tags, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![entry.id, entry.original_path, entry.filename, entry.format, entry.duration_ms, entry.sample_rate, entry.channels, entry.file_size_bytes, entry.rms, entry.peak, tags_json, entry.notes],
    )?;
    Ok(entry.id.clone())
}

pub fn list_imported_samples(conn: &Connection) -> SqlResult<Vec<ImportedSampleEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, original_path, filename, format, duration_ms, sample_rate, channels, file_size_bytes, rms, peak, tags, notes, is_favorite, created_at
         FROM imported_samples ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        let tags_str: String = row.get(10)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        Ok(ImportedSampleEntry {
            id: row.get(0)?,
            original_path: row.get(1)?,
            filename: row.get(2)?,
            format: row.get(3)?,
            duration_ms: row.get(4)?,
            sample_rate: row.get(5)?,
            channels: row.get(6)?,
            file_size_bytes: row.get(7)?,
            rms: row.get(8)?,
            peak: row.get(9)?,
            tags,
            notes: row.get(11)?,
            is_favorite: row.get::<_, i32>(12)? != 0,
            created_at: row.get(13)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows { results.push(row?); }
    Ok(results)
}

pub fn delete_imported_sample(conn: &Connection, sample_id: &str) -> SqlResult<bool> {
    let affected = conn.execute("DELETE FROM imported_samples WHERE id=?1", params![sample_id])?;
    Ok(affected > 0)
}

// ─── Duplicate Detection ──────────────────────────────

pub fn find_sounds_by_hash(conn: &Connection, hash: &str) -> SqlResult<Vec<SoundEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, is_favorite, source, variant_name, model, seed, created_at
         FROM sounds WHERE file_hash = ?1"
    )?;
    let rows = stmt.query_map(params![hash], |row| {
        Ok(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows { results.push(row?); }
    Ok(results)
}

pub fn get_sound_children(conn: &Connection, parent_id: &str) -> SqlResult<Vec<SoundEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, prompt, sound_type, duration_ms, sample_rate, rms, peak, spectral_centroid, tags, is_favorite, source, variant_name, model, seed, created_at
         FROM sounds WHERE parent_id = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![parent_id], |row| {
        Ok(SoundEntry {
            id: row.get(0)?,
            prompt: row.get(1)?,
            sound_type: row.get(2)?,
            duration_ms: row.get(3)?,
            sample_rate: row.get(4)?,
            rms: row.get(5)?,
            peak: row.get(6)?,
            spectral_centroid: row.get(7)?,
            tags: row.get(8)?,
            is_favorite: row.get::<_, i32>(9)? != 0,
            source: row.get(10)?,
            variant_name: row.get(11)?,
            model: row.get(12)?,
            seed: row.get(13)?,
            created_at: row.get(14)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows { results.push(row?); }
    Ok(results)
}

// ─── Library Stats ────────────────────────────────────

pub fn get_library_stats(conn: &Connection) -> SqlResult<LibraryStats> {
    let total_sounds: i64 = conn.query_row("SELECT COUNT(*) FROM sounds", [], |r| r.get(0)).unwrap_or(0);
    let total_favorites: i64 = conn.query_row("SELECT COUNT(*) FROM sounds WHERE is_favorite=1", [], |r| r.get(0)).unwrap_or(0);
    let total_exports: i64 = conn.query_row("SELECT COUNT(*) FROM exports", [], |r| r.get(0)).unwrap_or(0);
    let total_packs: i64 = conn.query_row("SELECT COUNT(*) FROM packs", [], |r| r.get(0)).unwrap_or(0);
    let total_imported: i64 = conn.query_row("SELECT COUNT(*) FROM imported_samples", [], |r| r.get(0)).unwrap_or(0);
    let disk_bytes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(file_size_bytes), 0) FROM (
            SELECT file_size_bytes FROM exports
            UNION ALL
            SELECT f.file_size_bytes FROM imported_samples f
        )", [], |r| r.get(0)
    ).unwrap_or(0);
    Ok(LibraryStats {
        total_sounds: total_sounds as u64,
        total_favorites: total_favorites as u64,
        total_exports: total_exports as u64,
        total_packs: total_packs as u64,
        total_imported: total_imported as u64,
        disk_bytes: disk_bytes as u64,
    })
}

// ─── Structs ──────────────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecipeEntry {
    pub id: String,
    pub title: String,
    pub prompt_template: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub transform_defaults: String,
    pub is_builtin: bool,
    pub usage_count: i64,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImportedSampleEntry {
    pub id: String,
    pub original_path: String,
    pub filename: String,
    pub format: String,
    pub duration_ms: Option<f32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub file_size_bytes: Option<i64>,
    pub rms: Option<f32>,
    pub peak: Option<f32>,
    pub tags: Vec<String>,
    pub notes: String,
    pub is_favorite: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LibraryStats {
    pub total_sounds: u64,
    pub total_favorites: u64,
    pub total_exports: u64,
    pub total_packs: u64,
    pub total_imported: u64,
    pub disk_bytes: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PackEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tags: String,
    pub sound_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SoundEntry {
    pub id: String,
    pub prompt: String,
    pub sound_type: String,
    pub duration_ms: f32,
    pub sample_rate: u32,
    pub rms: f32,
    pub peak: f32,
    pub spectral_centroid: f32,
    pub tags: String,
    pub is_favorite: bool,
    pub source: String,
    pub variant_name: Option<String>,
    pub model: String,
    pub seed: i64,
    pub created_at: String,
}
