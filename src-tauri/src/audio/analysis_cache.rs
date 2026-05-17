use std::collections::HashMap;
use std::path::PathBuf;
use super::analyze::{AudioAnalysis, analyze_audio};

pub struct AnalysisCache {
    cache: HashMap<String, AudioAnalysis>,
    path: PathBuf,
    dirty: bool,
}

impl AnalysisCache {
    /// Load cache from a specific directory path.
    /// No app-level dependencies - accepts path directly.
    pub fn load(cache_dir: &std::path::Path) -> Self {
        let path = cache_dir.join("analysis_cache.json");
        let cache = std::fs::read_to_string(&path).ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default();
        Self { cache, path, dirty: false }
    }

    pub fn save(&mut self) {
        if !self.dirty { return; }
        if let Ok(data) = serde_json::to_string(&self.cache) {
            let _ = std::fs::write(&self.path, data);
        }
        self.dirty = false;
    }

    pub fn get(&self, id: &str) -> Option<&AudioAnalysis> {
        self.cache.get(id)
    }

    pub fn analyze_and_cache(&mut self, id: &str, samples: &[f32], sample_rate: u32, channels: u16) -> AudioAnalysis {
        if let Some(cached) = self.cache.get(id) {
            return cached.clone();
        }
        let analysis = analyze_audio(samples, sample_rate, channels);
        self.cache.insert(id.to_string(), analysis.clone());
        self.dirty = true;
        analysis
    }

    pub fn invalidate(&mut self, id: &str) {
        self.cache.remove(id);
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.dirty = true;
    }
}
