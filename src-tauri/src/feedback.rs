use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FeedbackEntry {
    pub thumbs_up: bool,
    pub thumbs_down: bool,
    pub usable: Option<bool>,
    pub note: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FeedbackStore {
    pub entries: HashMap<String, FeedbackEntry>,
    #[serde(skip)]
    path: PathBuf,
}

impl FeedbackStore {
    pub fn load() -> Self {
        let path = app_dir().join("feedback.json");
        if path.exists() {
            let data = std::fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
            let entries: HashMap<String, FeedbackEntry> =
                serde_json::from_str(&data).unwrap_or_default();
            Self { entries, path }
        } else {
            Self {
                entries: HashMap::new(),
                path,
            }
        }
    }

    pub fn save(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.entries) {
            let _ = std::fs::write(&self.path, data);
        }
    }

    pub fn get(&self, sound_id: &str) -> Option<&FeedbackEntry> {
        self.entries.get(sound_id)
    }

    pub fn set_thumbs(&mut self, sound_id: &str, up: bool, down: bool) {
        let entry = self.entry_mut(sound_id);
        entry.thumbs_up = up;
        entry.thumbs_down = down;
        entry.updated_at = now_iso();
        self.save();
    }

    pub fn set_note(&mut self, sound_id: &str, note: Option<String>) {
        let entry = self.entry_mut(sound_id);
        entry.note = note;
        entry.updated_at = now_iso();
        self.save();
    }

    pub fn set_usable(&mut self, sound_id: &str, usable: bool) {
        let entry = self.entry_mut(sound_id);
        entry.usable = Some(usable);
        entry.updated_at = now_iso();
        self.save();
    }

    fn entry_mut(&mut self, sound_id: &str) -> &mut FeedbackEntry {
        self.entries.entry(sound_id.to_string()).or_insert_with(|| FeedbackEntry {
            thumbs_up: false,
            thumbs_down: false,
            usable: None,
            note: None,
            updated_at: String::new(),
        })
    }
}

fn app_dir() -> PathBuf {
    crate::storage::app_root()
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let time = secs % 86400;
    let hours = time / 3600;
    let mins = (time % 3600) / 60;
    let sec = time % 60;
    format!(
        "2025-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        (days / 30) % 12 + 1,
        days % 30 + 1,
        hours,
        mins,
        sec
    )
}
