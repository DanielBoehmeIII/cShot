use std::collections::HashMap;
use std::path::PathBuf;

// ─── User Action Types ──────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum UserAction {
    Favorited,
    Exported,
    Deleted,
    Regenerated,
    UsedRecipe,
    ThumbsUp,
    ThumbsDown,
    Previewed,
    Played,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionRecord {
    pub action: UserAction,
    pub sound_type: String,
    pub prompt: String,
    pub brightness: f32,
    pub energy: f32,
    pub duration_ms: f32,
    pub genre_hints: Vec<String>,
    pub timestamp: u64,
}

// ─── Taste Profile ─────────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TasteProfile {
    pub preferred_sound_types: HashMap<String, f32>,     // sound_type -> score (positive = liked)
    pub preferred_brightness: f32,                        // 0.0-1.0 average
    pub preferred_energy: f32,                            // 0.0-1.0 average
    pub preferred_duration_ms: f32,                       // average duration liked
    pub preferred_genres: HashMap<String, f32>,           // genre -> score
    pub avoided_qualities: Vec<String>,                   // things user consistently downvoted
    pub common_prompt_terms: HashMap<String, f32>,        // term -> frequency
    pub total_actions: u64,
    pub version: u32,
}

impl Default for TasteProfile {
    fn default() -> Self {
        Self {
            preferred_sound_types: HashMap::new(),
            preferred_brightness: 0.5,
            preferred_energy: 0.5,
            preferred_duration_ms: 300.0,
            preferred_genres: HashMap::new(),
            avoided_qualities: Vec::new(),
            common_prompt_terms: HashMap::new(),
            total_actions: 0,
            version: 1,
        }
    }
}

// ─── Taste Model ───────────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TasteModel {
    pub profile: TasteProfile,
    pub action_history: Vec<ActionRecord>,
    #[serde(skip)]
    path: PathBuf,
    max_history: usize,
    pub is_dirty: bool,
}

impl TasteModel {
    pub fn load() -> Self {
        let path = Self::taste_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(model) = serde_json::from_str::<Self>(&data) {
                    return Self { path, ..model };
                }
            }
        }
        Self {
            profile: TasteProfile::default(),
            action_history: Vec::new(),
            path,
            max_history: 500,
            is_dirty: false,
        }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Some(parent) = self.path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&self.path, &json);
        }
    }

    fn taste_path() -> PathBuf {
        crate::storage::app_root().join("taste_model.json")
    }

    pub fn record_action(&mut self, record: ActionRecord) {
        let st = &record.sound_type;

        match record.action {
            UserAction::Favorited | UserAction::ThumbsUp | UserAction::Exported => {
                let entry = self.profile.preferred_sound_types.entry(st.clone()).or_insert(0.0);
                *entry += 0.1;
                if record.brightness > 0.0 {
                    self.profile.preferred_brightness = self.profile.preferred_brightness * 0.9 + record.brightness * 0.1;
                }
                self.profile.preferred_energy = self.profile.preferred_energy * 0.9 + record.energy * 0.1;
                self.profile.preferred_duration_ms = self.profile.preferred_duration_ms * 0.9 + record.duration_ms * 0.1;

                for genre in &record.genre_hints {
                    let g_entry = self.profile.preferred_genres.entry(genre.clone()).or_insert(0.0);
                    *g_entry += 0.15;
                }

                for word in record.prompt.split_whitespace() {
                    let clean = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                    if clean.len() > 2 {
                        let term = self.profile.common_prompt_terms.entry(clean).or_insert(0.0);
                        *term += 1.0;
                    }
                }
            }
            UserAction::Deleted | UserAction::ThumbsDown => {
                let entry = self.profile.preferred_sound_types.entry(st.clone()).or_insert(0.0);
                *entry = (*entry - 0.05).max(0.0);
                if !self.profile.avoided_qualities.contains(st) {
                    self.profile.avoided_qualities.push(st.clone());
                }
            }
            UserAction::Regenerated | UserAction::UsedRecipe => {
                // Slightly reinforce the type
                let entry = self.profile.preferred_sound_types.entry(st.clone()).or_insert(0.0);
                *entry += 0.02;
                for genre in &record.genre_hints {
                    let g_entry = self.profile.preferred_genres.entry(genre.clone()).or_insert(0.0);
                    *g_entry += 0.05;
                }
            }
            _ => {}
        }

        self.profile.total_actions += 1;
        self.action_history.push(record);
        if self.action_history.len() > self.max_history {
            self.action_history.remove(0);
        }
        self.is_dirty = true;

        // Auto-save every 10 actions
        if self.profile.total_actions.is_multiple_of(10) {
            self.save();
        }
    }

    // ─── Query Methods ─────────────────────────────────

    pub fn preferred_sound_type_score(&self, sound_type: &str) -> f32 {
        self.profile.preferred_sound_types.get(sound_type).copied().unwrap_or(0.5).clamp(0.0, 1.0)
    }

    pub fn preferred_genre_score(&self, genre: &str) -> f32 {
        self.profile.preferred_genres.get(genre).copied().unwrap_or(0.5).clamp(0.0, 1.0)
    }

    pub fn term_frequency(&self, term: &str) -> f32 {
        self.profile.common_prompt_terms.get(term).copied().unwrap_or(0.0)
    }

    pub fn top_preferred_terms(&self, n: usize) -> Vec<(String, f32)> {
        let mut terms: Vec<(String, f32)> = self.profile.common_prompt_terms.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        terms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        terms.into_iter().take(n).collect()
    }

    pub fn top_preferred_types(&self, n: usize) -> Vec<(String, f32)> {
        let mut types: Vec<(String, f32)> = self.profile.preferred_sound_types.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        types.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        types.into_iter().take(n).collect()
    }

    pub fn preferred_params_defaults(&self, _sound_type: &str) -> (f32, f32, f32) {
        (self.profile.preferred_brightness, self.profile.preferred_energy, self.profile.preferred_duration_ms)
    }

    pub fn suggested_defaults(&self, _sound_type: &str) -> super::params::ExposedParams {
        super::params::ExposedParams {
            brightness: self.profile.preferred_brightness,
            weight: self.profile.preferred_energy,
            length: (self.profile.preferred_duration_ms / 500.0).clamp(0.0, 1.0),
            character: (self.profile.preferred_brightness - 0.5) * 2.0,
            ..super::params::ExposedParams::default()
        }
    }

    pub fn score_variant(&self, sound_type: &str, brightness: f32, energy: f32) -> f32 {
        let type_score = self.preferred_sound_type_score(sound_type);
        let bright_diff = 1.0 - (brightness - self.profile.preferred_brightness).abs();
        let energy_diff = 1.0 - (energy - self.profile.preferred_energy).abs();
        type_score * 0.4 + bright_diff * 0.3 + energy_diff * 0.3
    }
}
