use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SoundMetadata {
    pub id: String,
    pub prompt: String,
    pub sound_type: String,
    pub duration_ms: f32,
    pub created_at: String,
    pub source: String,
    pub model: String,
    pub seed: i64,
    pub variant_name: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Store {
    favorites: HashSet<String>,
    sounds: HashMap<String, SoundMetadata>,
}

pub struct FavoritesStore {
    path: PathBuf,
    store: Store,
}

impl FavoritesStore {
    pub fn load() -> Self {
        let path = crate::storage::favorites_path();
        let store = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or(Store {
                    favorites: HashSet::new(),
                    sounds: HashMap::new(),
                })
        } else {
            Store {
                favorites: HashSet::new(),
                sounds: HashMap::new(),
            }
        };
        Self { path, store }
    }

    pub fn toggle(&mut self, id: &str, meta: SoundMetadata) -> bool {
        if self.store.favorites.contains(id) {
            self.store.favorites.remove(id);
            self.store.sounds.remove(id);
            let _ = self.save();
            false
        } else {
            self.store.favorites.insert(id.to_string());
            self.store
                .sounds
                .insert(id.to_string(), meta);
            let _ = self.save();
            true
        }
    }

    pub fn is_favorited(&self, id: &str) -> bool {
        self.store.favorites.contains(id)
    }

    pub fn list(&self) -> Vec<SoundMetadata> {
        self.store
            .favorites
            .iter()
            .filter_map(|id| self.store.sounds.get(id))
            .cloned()
            .collect()
    }

    fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(&self.store).map_err(|e| e.to_string())?;
        fs::write(&self.path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn app_dir() -> PathBuf {
    crate::storage::app_root()
}
