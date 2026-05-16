use super::provider::{AudioProvider, GenerationError, GenerationRequest, GenerationResponse, ProviderCapabilities, ValidationResult};
use super::mock::MockProvider;
use super::validator;

pub struct ProviderRegistry {
    providers: Vec<Box<dyn AudioProvider>>,
    active_provider_name: Option<String>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            active_provider_name: None,
        }
    }

    pub fn with_defaults(mut self) -> Self {
        self.register(Box::new(MockProvider));
        self.set_active("mock-dsp");
        self
    }

    pub fn register(&mut self, provider: Box<dyn AudioProvider>) {
        let name = provider.name().to_string();
        if !self.providers.iter().any(|p| p.name() == name) {
            self.providers.push(provider);
        }
    }

    pub fn set_active(&mut self, name: &str) {
        if self.providers.iter().any(|p| p.name() == name) {
            self.active_provider_name = Some(name.to_string());
        }
    }

    pub fn active_provider_name(&self) -> Option<String> {
        self.active_provider_name.clone()
    }

    pub fn active_provider(&self) -> Option<&Box<dyn AudioProvider>> {
        self.active_provider_name.as_ref().and_then(|name| {
            self.providers.iter().find(|p| p.name() == name)
        })
    }

    pub fn available_providers(&self) -> Vec<&dyn AudioProvider> {
        self.providers.iter().map(|p| p.as_ref()).collect()
    }

    pub fn healthy_providers(&self) -> Vec<&dyn AudioProvider> {
        self.providers.iter()
            .filter(|p| p.is_available())
            .map(|p| p.as_ref())
            .collect()
    }

    pub fn all_provider_metadata(&self) -> Vec<ProviderCapabilities> {
        self.providers.iter().map(|p| p.capabilities()).collect()
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    pub fn has_available(&self) -> bool {
        self.providers.iter().any(|p| p.is_available())
    }

    /// Generate using the active provider with automatic fallback.
    /// 1. Try active provider (with timeout)
    /// 2. If it fails, try any other available provider
    /// 3. Final fallback: mock-dsp (always available)
    pub async fn generate_with_fallback(
        &self,
        request: GenerationRequest,
    ) -> Result<(GenerationResponse, Option<ValidationResult>, Vec<String>), String> {
        let mut errors: Vec<String> = Vec::new();

        let pre_check = validator::validate_pre_generation(&request);
        if let Err(e) = pre_check {
            return Err(e);
        }

        let attempts: Vec<&str> = {
            let mut order = Vec::new();

            if let Some(ref active) = self.active_provider_name {
                if active != "mock-dsp" {
                    order.push(active.as_str());
                }
            }

            for p in &self.providers {
                let name = p.name();
                if name != "mock-dsp" && self.active_provider_name.as_deref() != Some(name) {
                    if p.is_available() {
                        order.push(name);
                    }
                }
            }

            order.push("mock-dsp");
            order
        };

        for &name in &attempts {
            if let Some(provider) = self.providers.iter().find(|p| p.name() == name) {
                if !provider.is_available() {
                    let reason = provider.reason_unavailable()
                        .unwrap_or_else(|| format!("Provider '{}' is not available", name));
                    errors.push(reason);
                    continue;
                }

                let timeout_ms = if name == "mock-dsp" { 5000 } else { 15000 };
                let result = tokio::time::timeout(
                    std::time::Duration::from_millis(timeout_ms),
                    provider.generate(request.clone()),
                )
                .await;

                let response = match result {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => {
                        let msg = format!("Provider '{}' failed: {}", name, e);
                        errors.push(msg);
                        match &e {
                            GenerationError::ApiKeyMissing(_) | GenerationError::InvalidRequest(_) => {
                                break; // Non-retryable — stop trying other providers
                            }
                            _ => continue, // Retryable — try next provider
                        }
                    }
                    Err(_) => {
                        errors.push(format!("Provider '{}' timed out after {}ms", name, timeout_ms));
                        continue;
                    }
                };

                let validation = validator::validate_generated_sound(&response.samples);
                if !validation.passed && validation.has_silence {
                    errors.push(format!("Provider '{}' generated silent audio", name));
                    continue; // Retry with fallback
                }
                if !validation.passed {
                    errors.push(format!(
                        "Provider '{}' returned audio with issues: {}",
                        name,
                        validation.issues.join(", ")
                    ));
                }

                return Ok((response, Some(validation), errors));
            }
        }

        Err(format!(
            "All providers failed. Errors: {}",
            errors.join(" | ")
        ))
    }
}
