use super::provider::{
    AudioProvider, GenerationError, GenerationRequest, GenerationResponse, ProviderCapabilities,
};

pub struct AudioLdmProvider;

impl AudioLdmProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AudioProvider for AudioLdmProvider {
    fn name(&self) -> &str {
        "audioldm2"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            name: "AudioLDM 2 (Self-Hosted)",
            supports_reference_audio: true,
            supports_prompt: true,
            max_duration_seconds: 10.0,
            supported_input_types: vec!["text", "reference_audio"],
            supported_output_types: vec!["wav_16k_f32", "wav_44k_f32"],
            estimated_latency_ms: 15000,
            estimated_cost_per_generation_cents: 0.0,
            requires_api_key: false,
            requires_network: false,
        }
    }

    fn is_available(&self) -> bool {
        false
    }

    fn reason_unavailable(&self) -> Option<String> {
        Some("AudioLDM 2 requires local model deployment. Not yet configured.".to_string())
    }

    async fn generate(&self, _request: GenerationRequest) -> Result<GenerationResponse, GenerationError> {
        Err(GenerationError::ProviderUnavailable(
            "AudioLDM 2 provider is not yet implemented.".to_string()
        ))
    }
}
