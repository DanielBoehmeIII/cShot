use std::time::Instant;

use super::provider::{
    AudioProvider, GenerationError, GenerationRequest, GenerationResponse, ProviderCapabilities,
};

pub struct MockProvider;

#[async_trait::async_trait]
impl AudioProvider for MockProvider {
    fn name(&self) -> &str {
        "cshot-engine"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            name: "cShot Engine",
            supports_reference_audio: true,
            supports_prompt: true,
            max_duration_seconds: 5.0,
            supported_input_types: vec!["text", "reference_wav"],
            supported_output_types: vec!["wav_44k_f32"],
            estimated_latency_ms: 500,
            estimated_cost_per_generation_cents: 0.0,
            requires_api_key: false,
            requires_network: false,
        }
    }

    fn is_available(&self) -> bool {
        true
    }

    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse, GenerationError> {
        let start = Instant::now();

        let result = if let Some(ref ref_samples) = request.reference_audio {
            crate::generator::generate_with_reference(
                &request.prompt,
                ref_samples,
                request.reference_sample_rate.unwrap_or(44100),
            )
        } else {
            crate::generator::generate(&request.prompt, None)
        };

        let response = match result {
            Ok(sound_result) => {
                let audio_path = crate::storage::sound_path(&sound_result.id);
                let samples = crate::audio::read_wav(&audio_path)
                    .map_err(|e| GenerationError::Internal(format!("Failed to read generated audio: {}", e)))?;

                let latency_ms = start.elapsed().as_millis() as u64;

                GenerationResponse {
                    samples,
                    sample_rate: 44100,
                    provider: "cshot-engine".to_string(),
                    latency_ms,
                }
            }
            Err(e) => {
                return Err(GenerationError::Internal(e));
            }
        };

        Ok(response)
    }
}
