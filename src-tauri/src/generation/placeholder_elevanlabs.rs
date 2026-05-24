use std::time::Instant;
use super::provider::{
    AudioProvider, GenerationError, GenerationRequest, GenerationResponse, ProviderCapabilities,
};

const ELEVENLABS_API_URL: &str = "https://api.elevenlabs.io/v1/sound-effects/convert";

pub struct ElevenLabsProvider {
    api_key: Option<String>,
}

impl Default for ElevenLabsProvider {
    fn default() -> Self { Self::new() }
}

impl ElevenLabsProvider {
    pub fn new() -> Self {
        let key = std::env::var("CSHOT_ELEVENLABS_API_KEY").ok();
        Self {
            api_key: key.filter(|k| !k.is_empty()),
        }
    }
}

#[async_trait::async_trait]
impl AudioProvider for ElevenLabsProvider {
    fn name(&self) -> &str {
        "elevenlabs"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            name: "ElevenLabs Text-to-Sound-Effects",
            supports_reference_audio: false,
            supports_prompt: true,
            max_duration_seconds: 2.0,
            supported_input_types: vec!["text"],
            supported_output_types: vec!["wav_44k_f32"],
            estimated_latency_ms: 4000,
            estimated_cost_per_generation_cents: 10.0,
            requires_api_key: true,
            requires_network: true,
        }
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn reason_unavailable(&self) -> Option<String> {
        if self.api_key.is_none() {
            Some("CSHOT_ELEVENLABS_API_KEY not set. Set it in .env or your environment.".to_string())
        } else {
            None
        }
    }

    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse, GenerationError> {
        let key = self.api_key.as_ref()
            .ok_or_else(|| GenerationError::ApiKeyMissing("elevenlabs".to_string()))?;

        let start = Instant::now();

        let duration_seconds = request.duration_ms
            .map(|ms| (ms / 1000.0).max(0.5).min(2.0))
            .unwrap_or(1.0);

        let body = serde_json::json!({
            "text": request.prompt,
            "duration_seconds": duration_seconds,
            "prompt_influence": 0.3,
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| GenerationError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .post(ELEVENLABS_API_URL)
            .header("xi-api-key", key.as_str())
            .header("Accept", "audio/wav, audio/mpeg")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    GenerationError::Timeout("ElevenLabs API request timed out".to_string())
                } else if e.is_connect() {
                    GenerationError::NetworkError(format!("Cannot connect to ElevenLabs API: {}", e))
                } else {
                    GenerationError::NetworkError(format!("ElevenLabs API request failed: {}", e))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 | 403 => Err(GenerationError::ApiKeyMissing(
                    format!("ElevenLabs returned {}: {}. Check your API key.", status, error_body)
                )),
                 429 => Err(GenerationError::RateLimited(
                    "ElevenLabs rate limit exceeded. Try again later.".to_string()
                )),
                500..=599 => Err(GenerationError::ProviderUnavailable(
                    format!("ElevenLabs server error ({}): {}", status, error_body)
                )),
                _ => Err(GenerationError::Internal(
                    format!("ElevenLabs returned unexpected status {}: {}", status, error_body)
                )),
            };
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_lowercase();

        let audio_bytes = response
            .bytes()
            .await
            .map_err(|e| GenerationError::CorruptedOutput(
                format!("Failed to read ElevenLabs response body: {}", e)
            ))?;

        if audio_bytes.is_empty() {
            return Err(GenerationError::SilentOutput(
                "ElevenLabs returned empty audio".to_string()
            ));
        }

        if !content_type.contains("wav") && !content_type.contains("mpeg") && !content_type.contains("mp3") {
            return Err(GenerationError::UnsupportedFormat(
                format!("ElevenLabs returned unsupported content type: {}", content_type)
            ));
        }

        let samples = if content_type.contains("wav") {
            decode_wav_bytes(&audio_bytes)?
        } else {
            decode_mp3_bytes(&audio_bytes)?
        };

        if samples.is_empty() {
            return Err(GenerationError::SilentOutput(
                "Decoded ElevenLabs audio is empty".to_string()
            ));
        }

        let has_non_zero = samples.iter().any(|&s| s.abs() > 0.001);
        if !has_non_zero {
            return Err(GenerationError::SilentOutput(
                "ElevenLabs returned silent audio".to_string()
            ));
        }

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(GenerationResponse {
            samples,
            sample_rate: 44100,
            provider: "elevenlabs".to_string(),
            latency_ms,
        })
    }
}

fn decode_wav_bytes(bytes: &[u8]) -> Result<Vec<f32>, GenerationError> {
    let mut cursor = std::io::Cursor::new(bytes);
    let reader = hound::WavReader::new(&mut cursor)
        .map_err(|e| GenerationError::CorruptedOutput(
            format!("Failed to parse WAV from ElevenLabs: {}", e)
        ))?;

    let spec = reader.spec();
    if spec.sample_rate != 44100 && spec.sample_rate != 48000 && spec.sample_rate != 22050 {
        eprintln!("[cshot] ElevenLabs returned unexpected sample rate: {} Hz", spec.sample_rate);
    }

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

fn decode_mp3_bytes(bytes: &[u8]) -> Result<Vec<f32>, GenerationError> {
    let mut decoder = minimp3::Decoder::new(bytes);
    let mut all_samples = Vec::new();

    loop {
        match decoder.next_frame() {
            Ok(frame) => {
                let rate_ratio = 44100.0 / frame.sample_rate as f32;
                for sample in frame.data.iter() {
                    let normalized = *sample as f32 / i16::MAX as f32;
                    all_samples.push(normalized);
                }
                if rate_ratio != 1.0 {
                    let new_len = (all_samples.len() as f32 * rate_ratio) as usize;
                    let mut resampled = Vec::with_capacity(new_len);
                    for i in 0..new_len {
                        let src_idx = (i as f32 / rate_ratio) as usize;
                        let s = all_samples.get(src_idx).copied().unwrap_or(0.0);
                        resampled.push(s);
                    }
                    all_samples = resampled;
                }
            }
            Err(minimp3::Error::Eof) => break,
            Err(minimp3::Error::SkippedData) => continue,
            Err(e) => {
                if all_samples.is_empty() {
                    return Err(GenerationError::CorruptedOutput(
                        format!("Failed to decode MP3 from ElevenLabs: {}", e)
                    ));
                }
                break;
            }
        }
    }

    Ok(all_samples)
}
