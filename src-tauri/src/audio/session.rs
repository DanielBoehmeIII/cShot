use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionState {
    pub active_sound_id: Option<String>,
    pub last_prompt: Option<String>,
    pub last_sound_type: Option<String>,
    pub recent_sound_ids: Vec<String>,
    pub view_state: String,
    pub active_pack_id: Option<String>,
    pub provider_name: Option<String>,
    pub version: u32,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            active_sound_id: None,
            last_prompt: None,
            last_sound_type: None,
            recent_sound_ids: Vec::new(),
            view_state: "generator".to_string(),
            active_pack_id: None,
            provider_name: Some("cshot-engine".to_string()),
            version: 1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionManager {
    state: SessionState,
    path: PathBuf,
    is_dirty: bool,
}

impl SessionManager {
    pub fn load() -> Self {
        let path = Self::session_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&data) {
                    return Self { state, path, is_dirty: false };
                }
            }
        }
        Self {
            state: SessionState::default(),
            path,
            is_dirty: false,
        }
    }

    fn session_path() -> PathBuf {
        crate::storage::app_root().join("session.json")
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.state) {
            let _ = std::fs::write(&self.path, &json);
        }
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub fn set_active_sound(&mut self, sound_id: String) {
        self.state.active_sound_id = Some(sound_id.clone());
        self.state.recent_sound_ids.retain(|id| id != &sound_id);
        self.state.recent_sound_ids.push(sound_id);
        if self.state.recent_sound_ids.len() > 50 {
            self.state.recent_sound_ids.remove(0);
        }
        self.is_dirty = true;
        self.save();
    }

    pub fn set_last_prompt(&mut self, prompt: String, sound_type: String) {
        self.state.last_prompt = Some(prompt);
        self.state.last_sound_type = Some(sound_type);
        self.is_dirty = true;
        self.save();
    }

    pub fn set_view_state(&mut self, view: String) {
        self.state.view_state = view;
        self.is_dirty = true;
        self.save();
    }

    pub fn set_active_pack(&mut self, pack_id: Option<String>) {
        self.state.active_pack_id = pack_id;
        self.is_dirty = true;
        self.save();
    }

    pub fn set_provider(&mut self, provider: String) {
        self.state.provider_name = Some(provider);
        self.is_dirty = true;
        self.save();
    }

    pub fn last_prompt(&self) -> Option<&str> {
        self.state.last_prompt.as_deref()
    }

    pub fn last_sound_type(&self) -> Option<&str> {
        self.state.last_sound_type.as_deref()
    }

    pub fn recent_sounds(&self) -> &[String] {
        &self.state.recent_sound_ids
    }

    pub fn active_sound(&self) -> Option<&str> {
        self.state.active_sound_id.as_deref()
    }
}

// ─── Predefined Presets ────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresetEntry {
    pub name: String,
    pub sound_type: String,
    pub params_json: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

pub fn load_presets() -> Vec<PresetEntry> {
    let path = crate::storage::app_root().join("presets.json");
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(presets) = serde_json::from_str::<Vec<PresetEntry>>(&data) {
                return presets;
            }
        }
    }
    Vec::new()
}

pub fn save_preset(preset: &PresetEntry) {
    let mut presets = load_presets();
    presets.retain(|p| p.name != preset.name);
    presets.push(preset.clone());
    let path = crate::storage::app_root().join("presets.json");
    if let Ok(json) = serde_json::to_string_pretty(&presets) {
        let _ = std::fs::write(&path, &json);
    }
}

pub fn delete_preset(name: &str) {
    let mut presets = load_presets();
    presets.retain(|p| p.name != name);
    let path = crate::storage::app_root().join("presets.json");
    if let Ok(json) = serde_json::to_string_pretty(&presets) {
        let _ = std::fs::write(&path, &json);
    }
}

// ─── Recent Prompts / Recipes Store ─────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecentEntry {
    pub text: String,
    pub sound_type: String,
    pub used_at: String,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecentsStore {
    pub prompts: Vec<RecentEntry>,
    pub sound_ids: Vec<String>,
}

impl RecentsStore {
    pub fn load() -> Self {
        let path = crate::storage::app_root().join("recents.json");
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(store) = serde_json::from_str(&data) {
                    return store;
                }
            }
        }
        Self { prompts: Vec::new(), sound_ids: Vec::new() }
    }

    pub fn save(&self) {
        let path = crate::storage::app_root().join("recents.json");
        if let Ok(json) = serde_json::to_string_pretty(&self) {
            let _ = std::fs::write(&path, &json);
        }
    }

    pub fn record_prompt(&mut self, prompt: &str, sound_type: &str) {
        self.prompts.retain(|e| e.text != prompt);
        self.prompts.push(RecentEntry {
            text: prompt.to_string(),
            sound_type: sound_type.to_string(),
            used_at: chrono::Utc::now().to_rfc3339(),
            count: 1,
        });
        if self.prompts.len() > 100 {
            self.prompts.remove(0);
        }
        self.save();
    }

    pub fn record_sound(&mut self, sound_id: &str) {
        self.sound_ids.retain(|id| id != sound_id);
        self.sound_ids.push(sound_id.to_string());
        if self.sound_ids.len() > 200 {
            self.sound_ids.remove(0);
        }
        self.save();
    }

    pub fn recent_prompts(&self, limit: usize) -> Vec<&RecentEntry> {
        self.prompts.iter().rev().take(limit).collect()
    }

    pub fn recent_sounds(&self, limit: usize) -> Vec<&str> {
        self.sound_ids.iter().rev().take(limit).map(|s| s.as_str()).collect()
    }
}
