use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub prompt: String,
    pub reference_audio: Option<Vec<f32>>,
    pub reference_sample_rate: Option<u32>,
    pub duration_ms: Option<f32>,
    pub seed: Option<u64>,
}

impl GenerationRequest {
    pub fn from_prompt(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            reference_audio: None,
            reference_sample_rate: None,
            duration_ms: None,
            seed: None,
        }
    }

    pub fn with_reference(mut self, samples: Vec<f32>, sample_rate: u32) -> Self {
        self.reference_audio = Some(samples);
        self.reference_sample_rate = Some(sample_rate);
        self
    }

    pub fn with_duration(mut self, ms: f32) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub provider: String,
    pub latency_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub name: &'static str,
    pub supports_reference_audio: bool,
    pub supports_prompt: bool,
    pub max_duration_seconds: f32,
    pub supported_input_types: Vec<&'static str>,
    pub supported_output_types: Vec<&'static str>,
    pub estimated_latency_ms: u32,
    pub estimated_cost_per_generation_cents: f32,
    pub requires_api_key: bool,
    pub requires_network: bool,
}

impl fmt::Display for ProviderCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (latency:~{}ms, cost:~{}¢, network:{}, api_key:{})",
            self.name, self.estimated_latency_ms, self.estimated_cost_per_generation_cents,
            if self.requires_network { "yes" } else { "no" },
            if self.requires_api_key { "yes" } else { "no" },
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub issues: Vec<String>,
    pub rms: f32,
    pub peak: f32,
    pub duration_ms: f32,
    pub has_silence: bool,
    pub has_clipping: bool,
    pub has_nan: bool,
}

#[derive(Clone, Debug)]
pub enum GenerationError {
    ProviderUnavailable(String),
    ApiKeyMissing(String),
    NetworkError(String),
    InvalidRequest(String),
    Timeout(String),
    CorruptedOutput(String),
    SilentOutput(String),
    RateLimited(String),
    UnsupportedFormat(String),
    Internal(String),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerationError::ProviderUnavailable(msg) => write!(f, "Provider unavailable: {}", msg),
            GenerationError::ApiKeyMissing(provider) => write!(f, "API key missing for provider: {}", provider),
            GenerationError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            GenerationError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            GenerationError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            GenerationError::CorruptedOutput(msg) => write!(f, "Corrupted output: {}", msg),
            GenerationError::SilentOutput(msg) => write!(f, "Silent output: {}", msg),
            GenerationError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
            GenerationError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            GenerationError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for GenerationError {}

#[async_trait::async_trait]
pub trait AudioProvider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;
    fn is_available(&self) -> bool;
    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse, GenerationError>;
    fn reason_unavailable(&self) -> Option<String> {
        None
    }
}
