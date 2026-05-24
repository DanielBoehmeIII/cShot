use std::time::Instant;
use super::provider::{
    AudioProvider, GenerationError, GenerationRequest, GenerationResponse, ProviderCapabilities,
};

const STABLE_AUDIO_API_URL: &str = "https://api.stability.ai/v2beta/stable-audio/generate";

pub struct StableAudioProvider {
    api_key: Option<String>,
}

impl Default for StableAudioProvider {
    fn default() -> Self { Self::new() }
}

impl StableAudioProvider {
    pub fn new() -> Self {
        let key = std::env::var("CSHOT_STABLEAUDIO_API_KEY").ok();
        Self {
            api_key: key.filter(|k| !k.is_empty()),
        }
    }
}

#[async_trait::async_trait]
impl AudioProvider for StableAudioProvider {
    fn name(&self) -> &str {
        "stable-audio"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            name: "Stable Audio (API)",
            supports_reference_audio: false,
            supports_prompt: true,
            max_duration_seconds: 30.0,
            supported_input_types: vec!["text"],
            supported_output_types: vec!["wav_44k_stereo_f32"],
            estimated_latency_ms: 8000,
            estimated_cost_per_generation_cents: 8.0,
            requires_api_key: true,
            requires_network: true,
        }
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn reason_unavailable(&self) -> Option<String> {
        if self.api_key.is_none() {
            Some("CSHOT_STABLEAUDIO_API_KEY not set.".to_string())
        } else {
            None
        }
    }

    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse, GenerationError> {
        let key = self.api_key.as_ref()
            .ok_or_else(|| GenerationError::ApiKeyMissing("stable-audio".to_string()))?;

        let start = Instant::now();

        let duration_seconds = request.duration_ms
            .map(|ms| (ms / 1000.0).max(0.5).min(30.0))
            .unwrap_or(2.0);

        let body = serde_json::json!({
            "text_prompts": [
                {
                    "text": request.prompt,
                    "type": "text",
                    "weight": 1.0
                }
            ],
            "cfg_scale": 7,
            "steps": 50,
            "duration_seconds": duration_seconds,
            "seed": request.seed.unwrap_or(0),
            "output_format": "wav",
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| GenerationError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .post(STABLE_AUDIO_API_URL)
            .header("Authorization", format!("Bearer {}", key))
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    GenerationError::Timeout("Stable Audio API request timed out".to_string())
                } else if e.is_connect() {
                    GenerationError::NetworkError(format!("Cannot connect to Stable Audio API: {}", e))
                } else {
                    GenerationError::NetworkError(format!("Stable Audio API request failed: {}", e))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 | 403 => Err(GenerationError::ApiKeyMissing(
                    format!("Stable Audio returned {}: {}. Check your API key.", status, error_body)
                )),
                429 => Err(GenerationError::Internal(
                    "Stable Audio rate limit exceeded. Try again later.".to_string()
                )),
                500..=599 => Err(GenerationError::ProviderUnavailable(
                    format!("Stable Audio server error ({}): {}", status, error_body)
                )),
                _ => Err(GenerationError::Internal(
                    format!("Stable Audio returned unexpected status {}: {}", status, error_body)
                )),
            };
        }

        #[derive(serde::Deserialize)]
        struct Artifact {
            data: String,
            #[allow(dead_code)]
            #[serde(rename = "mimeType", default)]
            mime_type: String,
        }

        #[derive(serde::Deserialize)]
        struct GenerationResponseBody {
            artifacts: Vec<Artifact>,
        }

        let body_text = response.text().await
            .map_err(|e| GenerationError::CorruptedOutput(
                format!("Failed to read Stable Audio response: {}", e)
            ))?;

        let gen_response: GenerationResponseBody = serde_json::from_str(&body_text)
            .map_err(|e| GenerationError::CorruptedOutput(
                format!("Failed to parse Stable Audio response: {}", e)
            ))?;

        let artifact = gen_response.artifacts.into_iter().next()
            .ok_or_else(|| GenerationError::SilentOutput(
                "Stable Audio returned no audio artifacts".to_string()
            ))?;

        let audio_bytes = base64_decode(&artifact.data)
            .map_err(|e| GenerationError::CorruptedOutput(
                format!("Failed to decode Stable Audio base64: {}", e)
            ))?;

        if audio_bytes.is_empty() {
            return Err(GenerationError::SilentOutput(
                "Stable Audio returned empty audio".to_string()
            ));
        }

        let samples = decode_wav_bytes(&audio_bytes)?;

        if samples.is_empty() {
            return Err(GenerationError::SilentOutput(
                "Decoded Stable Audio audio is empty".to_string()
            ));
        }

        let has_non_zero = samples.iter().any(|&s| s.abs() > 0.001);
        if !has_non_zero {
            return Err(GenerationError::SilentOutput(
                "Stable Audio returned silent audio".to_string()
            ));
        }

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(GenerationResponse {
            samples,
            sample_rate: 44100,
            provider: "stable-audio".to_string(),
            latency_ms,
        })
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| format!("Base64 decode error: {}", e))
}

fn decode_wav_bytes(bytes: &[u8]) -> Result<Vec<f32>, GenerationError> {
    let mut cursor = std::io::Cursor::new(bytes);
    let reader = hound::WavReader::new(&mut cursor)
        .map_err(|e| GenerationError::CorruptedOutput(
            format!("Failed to parse WAV from Stable Audio: {}", e)
        ))?;

    let spec = reader.spec();

    match spec.sample_format {
        hound::SampleFormat::Float => {
            let samples: Vec<f32> = reader.into_samples::<f32>()
                .filter_map(|s| s.ok())
                .collect();
            Ok(samples)
        }
        hound::SampleFormat::Int => {
            let max_val = 2i32.pow(spec.bits_per_sample as u32 - 1) as f32;
            let samples: Vec<f32> = reader.into_samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max_val)
                .collect();
            Ok(samples)
        }
    }
}
