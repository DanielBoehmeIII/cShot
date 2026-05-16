# Prompt 47 — Design the First Sound Library

Store, organize, search, and export generated one-shots.

---

## 1. Data Schema

### SQLite Tables

```sql
-- Sounds: Core table for every generated or imported sound
CREATE TABLE sounds (
    id              TEXT PRIMARY KEY,           -- UUID v4
    audio_hash      TEXT NOT NULL UNIQUE,       -- SHA-256 of audio content
    file_path       TEXT NOT NULL,              -- Relative to library root
    
    -- Origin
    source          TEXT NOT NULL DEFAULT 'generated',  -- 'generated', 'imported', 'reference'
    prompt          TEXT,                       -- Original generation prompt (NULL for imports)
    reference_id    TEXT REFERENCES sounds(id), -- Source sound if this is a variant
    model_name      TEXT,                       -- Model used for generation
    model_version   TEXT,                       -- Model version
    
    -- Generation parameters (JSON blob)
    generation_params TEXT,                     -- Seed, CFG scale, steps, etc.
    seed            INTEGER,
    
    -- Audio metadata
    duration_ms     REAL NOT NULL,
    sample_rate     INTEGER NOT NULL DEFAULT 44100,
    channels        INTEGER NOT NULL DEFAULT 1,
    peak_db         REAL,
    rms_db          REAL,
    loudness_lufs   REAL,
    
    -- Analysis (computed on generation/import)
    sound_type      TEXT,                       -- kick, snare, hat, clap, perc, bass, fx, other
    spectral_centroid_hz REAL,
    tempo_bpm       INTEGER,                    -- Estimated tempo (if applicable)
    estimated_key   TEXT,                       -- Estimated musical key (if applicable)
    
    -- User state
    is_favorite     INTEGER NOT NULL DEFAULT 0,
    rating          INTEGER,                    -- 1-5, NULL = unrated
    notes           TEXT,                       -- User notes
    
    -- Timestamps
    created_at      TEXT NOT NULL,              -- ISO 8601
    updated_at      TEXT NOT NULL,              -- ISO 8601
    last_played_at  TEXT,
    exported_at     TEXT,
    
    -- Usage tracking
    play_count      INTEGER NOT NULL DEFAULT 0,
    export_count    INTEGER NOT NULL DEFAULT 0
);

-- Tags: Many-to-many tags per sound
CREATE TABLE tags (
    sound_id    TEXT NOT NULL REFERENCES sounds(id) ON DELETE CASCADE,
    tag         TEXT NOT NULL,
    source      TEXT NOT NULL DEFAULT 'auto',   -- 'auto', 'user', 'model'
    confidence  REAL DEFAULT 1.0,               -- 0.0-1.0 for auto-tags
    created_at  TEXT NOT NULL,
    PRIMARY KEY (sound_id, tag, source)
);

-- Exports: Audit log of exports
CREATE TABLE exports (
    id              TEXT PRIMARY KEY,
    sound_id        TEXT NOT NULL REFERENCES sounds(id),
    file_path       TEXT NOT NULL,
    filename        TEXT NOT NULL,
    format          TEXT NOT NULL DEFAULT 'wav',
    sample_rate     INTEGER NOT NULL DEFAULT 44100,
    bit_depth       INTEGER NOT NULL DEFAULT 24,
    channels        INTEGER NOT NULL DEFAULT 1,
    file_size_bytes INTEGER NOT NULL,
    exported_at     TEXT NOT NULL
);

-- Generation log: History of generation requests
CREATE TABLE generation_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    prompt          TEXT NOT NULL,
    seed            INTEGER,
    model_name      TEXT,
    model_version   TEXT,
    duration_ms     REAL,                       -- Generation time in ms
    success         INTEGER NOT NULL DEFAULT 1,
    error_message   TEXT,
    sounds_created  INTEGER DEFAULT 0,          -- How many sounds from this gen
    created_at      TEXT NOT NULL
);

-- Library metadata
CREATE TABLE library_meta (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

-- Indexes
CREATE INDEX idx_sounds_type ON sounds(sound_type);
CREATE INDEX idx_sounds_favorite ON sounds(is_favorite);
CREATE INDEX idx_sounds_created ON sounds(created_at);
CREATE INDEX idx_sounds_source ON sounds(source);
CREATE INDEX idx_sounds_model ON sounds(model_name);
CREATE INDEX idx_tags_sound ON tags(sound_id);
CREATE INDEX idx_tags_tag ON tags(tag);
CREATE INDEX idx_exports_sound ON exports(sound_id);
CREATE INDEX idx_generation_log_created ON generation_log(created_at);

-- Full-text search
CREATE VIRTUAL TABLE sounds_fts USING fts5(
    prompt,
    sound_type,
    notes,
    content='sounds',
    content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER sounds_ai AFTER INSERT ON sounds BEGIN
    INSERT INTO sounds_fts(rowid, prompt, sound_type, notes)
    VALUES (new.rowid, new.prompt, new.sound_type, new.notes);
END;

CREATE TRIGGER sounds_ad AFTER DELETE ON sounds BEGIN
    INSERT INTO sounds_fts(sounds_fts, rowid, prompt, sound_type, notes)
    VALUES ('delete', old.rowid, old.prompt, old.sound_type, old.notes);
END;

CREATE TRIGGER sounds_au AFTER UPDATE ON sounds BEGIN
    INSERT INTO sounds_fts(sounds_fts, rowid, prompt, sound_type, notes)
    VALUES ('delete', old.rowid, old.prompt, old.sound_type, old.notes);
    INSERT INTO sounds_fts(rowid, prompt, sound_type, notes)
    VALUES (new.rowid, new.prompt, new.sound_type, new.notes);
END;
```

---

## 2. File Structure

```
~/cShot/
├── library.db                  # SQLite database (all metadata + search)
├── audio/                      # Content-addressed WAV files
│   └── {hash_prefix}/
│       └── {full_hash}.wav     # e.g., audio/a1/b2/a1b2c3d4....wav
├── exports/                    # User export directory
│   └── {date}/
│       └── {type}_{bpm}_{key}_{seed}.wav
├── cache/
│   ├── waveforms/              # Pre-computed waveform thumbnails
│   └── spectrograms/           # Pre-computed spectrogram images (post-MVP)
└── config.toml                 # User settings
```

### Content-Addressed Storage

```rust
pub struct ContentAddressedStore {
    base_path: PathBuf,
}

impl ContentAddressedStore {
    pub fn store(&self, audio: &[f32], sample_rate: u32) -> Result<(String, PathBuf)>;
    pub fn load(&self, hash: &str) -> Result<Vec<f32>>;
    pub fn delete(&self, hash: &str) -> Result<()>;
    pub fn path_for_hash(&self, hash: &str) -> PathBuf;
}
```

---

## 3. Metadata Format (for JSON export/import)

```json
{
  "version": 1,
  "exported_at": "2025-01-15T10:30:00Z",
  "sounds": [
    {
      "id": "a1b2c3d4-...",
      "audio_hash": "e3b0c442...",
      "prompt": "punchy trap kick 140bpm",
      "source": "generated",
      "model_name": "elevenlabs-sfx",
      "model_version": "1.0",
      "seed": 42,
      "generation_params": {
        "duration_seconds": 2.0,
        "prompt_influence": 0.5
      },
      "duration_ms": 423.5,
      "sample_rate": 44100,
      "channels": 1,
      "peak_db": -1.0,
      "rms_db": -8.2,
      "sound_type": "kick",
      "spectral_centroid_hz": 1850,
      "tempo_bpm": null,
      "estimated_key": null,
      "is_favorite": true,
      "rating": 4,
      "notes": "Great for trap verse drops",
      "tags": [
        {"tag": "punchy", "source": "auto", "confidence": 0.92},
        {"tag": "short", "source": "auto", "confidence": 1.0},
        {"tag": "trap", "source": "user", "confidence": 1.0}
      ],
      "created_at": "2025-01-15T10:25:00Z",
      "play_count": 12,
      "export_count": 3
    }
  ]
}
```

---

## 4. Rust Module Structure

```
src-tauri/src/
├── db/
│   ├── mod.rs              # Database initialization, connection pool
│   ├── schema.rs           # Table creation, migrations
│   ├── sounds.rs           # CRUD for sounds table
│   ├── tags.rs             # Tag operations
│   ├── exports.rs          # Export log operations
│   ├── search.rs           # Full-text search + filters
│   └── migrations/         # SQL migration files
│       └── 001_initial.sql
│
├── storage/
│   ├── mod.rs
│   └── content_addressed.rs  # File storage with SHA-256 dedup
│
├── commands/
│   ├── library.rs          # Tauri commands: CRUD, search, export
│   └── ...
```

### Key Tauri Commands

```typescript
// Library CRUD
invoke('get_sound', { sound_id: string }) => SoundData
invoke('get_sounds', { filters: SoundFilters, pagination: Pagination }) => PaginatedResult<SoundData>
invoke('delete_sound', { sound_id: string }) => void
invoke('update_sound_notes', { sound_id: string, notes: string }) => void
invoke('update_sound_rating', { sound_id: string, rating: number }) => void

// Favorites
invoke('get_favorites') => SoundData[]
invoke('toggle_favorite', { sound_id: string }) => boolean
invoke('is_favorited', { sound_id: string }) => boolean

// Tags
invoke('add_tag', { sound_id: string, tag: string, source?: string }) => void
invoke('remove_tag', { sound_id: string, tag: string }) => void
invoke('get_tags_for_sound', { sound_id: string }) => Tag[]
invoke('get_all_tags') => { tag: string, count: number }[]

// Search
invoke('search_sounds', { query: string, filters: SearchFilters, sort: SortOption }) => SoundData[]

// Export log
invoke('get_export_history', { limit?: number }) => ExportRecord[]
invoke('clear_export_history') => void

// Library stats
invoke('get_library_stats') => LibraryStats
// { total_sounds: 234, favorites: 45, total_duration: 98342, top_tags: [...], by_type: {...} }

// Import/Export library
invoke('export_library_json', { path: string }) => void
invoke('import_library_json', { path: string }) => ImportResult
```

---

## 5. Search/Filter UI

### Layout

```
┌─────────────────────────────────────────────────────┐
│ 🔍 Search library...                       [Sort ▼] │
│                                                    │
│ Filters: [All Types ▼] [Tag ▼] [Date ▼] [Model ▼]  │
│                                                    │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐│
│ │ Waveform  │ │ Waveform  │ │ Waveform  │ │ Waveform  ││
│ │ Kick      │ │ Snare    │ │ Hat      │ │ Kick     ││
│ │ 0.4s      │ │ 0.3s     │ │ 0.2s     │ │ 0.5s     ││
│ │ ★        │ │ ☆        │ │ ★        │ │ ★        ││
│ └──────────┘ └──────────┘ └──────────┘ └──────────┘│
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐│
│ │ ...      │ │ ...      │ │ ...      │ │ ...      ││
│ └──────────┘ └──────────┘ └──────────┘ └──────────┘│
│                                                    │
│ ← Prev [Page 1 of 10] Next →  Showing 40 of 394    │
└─────────────────────────────────────────────────────┘
```

### Filter Options

```
Sound Type:
  All | Kick | Snare | Hi-Hat | Clap | Percussion | Bass | FX | Other

Tags:
  Auto-generated tag cloud (top 20 by count)
  Size = frequency, color = category

Date:
  Today | This Week | This Month | This Year | Custom range

Model:
  All | ElevenLabs SFX | Stable Audio | AudioLDM 2 | Imported

Sort:
  Newest First | Oldest First | Most Played | Most Exported | Highest Rated | Duration

Favorites Toggle:
  Show All | Favorites Only
```

### Search Implementation

```rust
pub struct SearchQuery {
    pub text: Option<String>,           // FTS5 on prompt, notes, type
    pub sound_type: Option<SoundType>,
    pub tags: Option<Vec<String>>,      // AND logic
    pub tags_mode: TagMode,             // Any, All, None
    pub is_favorite: Option<bool>,
    pub model_name: Option<String>,
    pub source: Option<String>,
    pub min_duration_ms: Option<f32>,
    pub max_duration_ms: Option<f32>,
    pub date_from: Option<String>,      // ISO 8601
    pub date_to: Option<String>,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub page: u32,
    pub per_page: u32,                  // default: 40
}

pub struct SearchResult {
    pub sounds: Vec<SoundData>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub facets: SearchFacets,           // Aggregate counts for filters
}

pub struct SearchFacets {
    pub sound_types: Vec<(String, u32)>,
    pub tags: Vec<(String, u32)>,
    pub models: Vec<(String, u32)>,
    pub date_ranges: Vec<(String, u32)>,
}
```

---

## 6. Backup/Export Strategy

### Manual Export

```
User Action: Library → Export All → Save JSON
Exports: ~/cShot/exports/library_backup_2025-01-15.json

Format: Standard JSON (§3)
Contents: All metadata, no audio files
Use: Transfer library between machines, version control
```

### Full Backup

```
User Action: Library → Backup → Choose directory
Creates: ~/cShot/backups/
  ├── library.db                  # SQLite snapshot
  ├── audio/                      # All WAV files
  │   └── {hash_prefix}/
  │       └── {full_hash}.wav
  └── backup_manifest.json        # Metadata about the backup

Restore: Library → Restore → Choose backup directory
Behavior: Replaces current library (with confirmation)
```

### Auto-Backup

```rust
pub struct BackupConfig {
    pub auto_backup_enabled: bool,      // default: true
    pub interval_days: u32,             // default: 7
    pub max_backups: u32,              // default: 10
    pub backup_path: Option<PathBuf>,  // default: ~/cShot/backups/
}

// On app start, check if backup is due
// On first launch of the day, check last backup date
// Auto-backup runs in background thread
// Only backs up metadata + new audio (incremental)
```

### Export for DAW Use

```
User Action: Select sounds → Export → "Copy to Ableton"
Or: Select → Export → "Export as Drum Rack"

Future:
  - Export as Ableton Drum Rack (adg file)
  - Export as Logic Drum Machine Designer preset
  - Export as FL Studio FPC kit
  - Export as DecentSampler / SFZ instrument
  - Direct drag to DAW window
```

---

## 7. Stats View (For User Dashboard)

```typescript
interface LibraryStats {
  total_sounds: number;
  total_duration_minutes: number;
  favorites: number;
  exports: number;
  generations: number;
  
  by_type: {
    kick: number;
    snare: number;
    hihat: number;
    clap: number;
    percussion: number;
    bass: number;
    fx: number;
    other: number;
  };
  
  top_tags: { tag: string; count: number }[];
  top_models: { model: string; count: number }[];
  
  recent_activity: {
    generated_today: number;
    exported_today: number;
    favorited_today: number;
  };
  
  storage_used_mb: number;
  database_size_mb: number;
}
```

---

## 8. Implementation Order

```
Phase 1 — Prototype (flat JSON, no SQLite):
  1. favorites.json with flat metadata
  2. WAV files in flat ~/cShot/audio/ directory
  3. Basic "show favorites" toggle in UI

Phase 2 — MVP:
  4. SQLite database with schema migrations
  5. Auto-save every generated/imported sound to DB
  6. Tag CRUD (auto-tags + user tags)
  7. Search with FTS5
  8. Filter by type, tag, date, model
  9. Favorites persisted in DB
  10. Export history log

Phase 3 — Post-MVP:
  11. Content-addressed storage with dedup
  12. Library stats dashboard
  13. JSON metadata import/export
  14. Full backup/restore
  15. DAW export presets
```
