use serde::{Serialize, Deserialize};
use crate::generation::build_default_registry;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BakeoffEntry {
    pub prompt: String,
    pub sound_type: String,
    pub provider_results: Vec<ProviderResult>,
    pub reference_file: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderResult {
    pub provider_name: String,
    pub provider_display: String,
    pub sound_id: Option<String>,
    pub latency_ms: u64,
    pub score: u32,
    pub failure_labels: Vec<String>,
    pub rms: f32,
    pub peak: f32,
    pub spectral_centroid: f32,
    pub duration_ms: f32,
    pub user_rating: Option<i32>,
    pub is_favorited: bool,
    pub is_exported: bool,
    pub warnings: Vec<String>,
    pub failed: bool,
    pub failure_reason: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BakeoffSummary {
    pub total_prompts: usize,
    pub providers_tested: Vec<String>,
    pub provider_summaries: Vec<ProviderSummary>,
    pub overall_failure_rate: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderSummary {
    pub name: String,
    pub display_name: String,
    pub is_available: bool,
    pub reason_unavailable: Option<String>,
    pub avg_score: f32,
    pub avg_latency_ms: f32,
    pub total_generations: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub failure_rate: f32,
    pub avg_user_rating: f32,
    pub favorite_rate: f32,
    pub export_rate: f32,
    pub supports_reference_audio: bool,
    pub max_duration_seconds: f32,
    pub requires_api_key: bool,
    pub requires_network: bool,
}

pub async fn get_bakeoff_data() -> Result<BakeoffSummary, String> {
    let registry = build_default_registry();
    let providers = registry.available_providers();

    let provider_summaries: Vec<ProviderSummary> = providers.iter().map(|p| {
        let caps = p.capabilities();
        ProviderSummary {
            name: p.name().to_string(),
            display_name: caps.name.to_string(),
            is_available: p.is_available(),
            reason_unavailable: p.reason_unavailable(),
            avg_score: 0.0,
            avg_latency_ms: caps.estimated_latency_ms as f32,
            total_generations: 0,
            success_count: 0,
            failure_count: 0,
            failure_rate: 0.0,
            avg_user_rating: 0.0,
            favorite_rate: 0.0,
            export_rate: 0.0,
            supports_reference_audio: caps.supports_reference_audio,
            max_duration_seconds: caps.max_duration_seconds,
            requires_api_key: caps.requires_api_key,
            requires_network: caps.requires_network,
        }
    }).collect();

    Ok(BakeoffSummary {
        total_prompts: 0,
        providers_tested: providers.iter().map(|p| p.name().to_string()).collect(),
        provider_summaries,
        overall_failure_rate: 0.0,
    })
}

pub async fn run_mini_bakeoff(prompt: String) -> Result<BakeoffEntry, String> {
    let registry = build_default_registry();
    let providers = registry.healthy_providers();

    if providers.is_empty() {
        return Err("No providers available for bakeoff".to_string());
    }

    let sound_type = crate::prompt::parse_prompt(&prompt).sound_type;
    let sound_type_str = sound_type.as_str().to_string();

    let mut provider_results = Vec::new();

    for provider in providers {
        let caps = provider.capabilities();
        let req = crate::generation::provider::GenerationRequest::from_prompt(&prompt);

        match provider.generate(req).await {
            Ok(response) => {
                let duration_ms = response.samples.len() as f32 / 44100.0 * 1000.0;
                let rms = crate::audio::compute_rms(&response.samples);
                let peak = crate::audio::compute_peak(&response.samples);
                let spectral_centroid = crate::audio::compute_spectral_centroid(&response.samples);

                let q = crate::quality::compute_quality(&response.samples, sound_type, "original", true);
                let s = crate::score::compute_score(&q, sound_type, Some(false), None);

                let mut warnings = Vec::new();
                if q.clipping_detected {
                    warnings.push("Clipping detected".to_string());
                }
                if q.is_too_quiet {
                    warnings.push("Below target loudness".to_string());
                }
                if !q.duration_appropriate {
                    warnings.push("Duration outside expected range".to_string());
                }

                let sound_id = uuid::Uuid::new_v4().to_string();
                let audio_dir = crate::storage::audio_dir();
                std::fs::create_dir_all(&audio_dir).ok();
                let wav_path = audio_dir.join(format!("{}.wav", sound_id));
                crate::audio::write_wav(&wav_path, &response.samples, 44100).ok();

                provider_results.push(ProviderResult {
                    provider_name: provider.name().to_string(),
                    provider_display: caps.name.to_string(),
                    sound_id: Some(sound_id),
                    latency_ms: response.latency_ms,
                    score: s.overall,
                    failure_labels: s.failure_labels,
                    rms,
                    peak,
                    spectral_centroid,
                    duration_ms,
                    user_rating: None,
                    is_favorited: false,
                    is_exported: false,
                    warnings,
                    failed: false,
                    failure_reason: None,
                });
            }
            Err(e) => {
                provider_results.push(ProviderResult {
                    provider_name: provider.name().to_string(),
                    provider_display: caps.name.to_string(),
                    sound_id: None,
                    latency_ms: 0,
                    score: 0,
                    failure_labels: vec![],
                    rms: 0.0,
                    peak: 0.0,
                    spectral_centroid: 0.0,
                    duration_ms: 0.0,
                    user_rating: None,
                    is_favorited: false,
                    is_exported: false,
                    warnings: vec![],
                    failed: true,
                    failure_reason: Some(e.to_string()),
                });
            }
        }
    }

    Ok(BakeoffEntry {
        prompt,
        sound_type: sound_type_str,
        provider_results,
        reference_file: None,
    })
}
