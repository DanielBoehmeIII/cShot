use super::provider::{AudioProvider, GenerationRequest, GenerationResponse, ProviderCapabilities, ValidationResult};
use super::mock::MockProvider;
use super::validator;

pub struct ProviderRegistry {
    providers: Vec<Box<dyn AudioProvider>>,
    active_provider_name: Option<String>,
}

impl Default for ProviderRegistry {
    fn default() -> Self { Self::new() }
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
        self.set_active("cshot-engine");
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

    pub fn active_provider(&self) -> Option<&dyn AudioProvider> {
        self.active_provider_name.as_ref().and_then(|name| {
            self.providers.iter().find(|p| p.name() == name).map(|p| p.as_ref())
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
    /// Local Engine (cshot-engine) is always tried first.
    /// Cloud providers are only tried if explicitly configured and selected.
    pub async fn generate_with_fallback(
        &self,
        request: GenerationRequest,
    ) -> Result<(GenerationResponse, Option<ValidationResult>, Vec<String>), String> {
        let mut errors: Vec<String> = Vec::new();

        let pre_check = validator::validate_pre_generation(&request);
        pre_check?;

        // Build attempt order: cshot-engine first, then active cloud provider if any
        let attempts: Vec<&str> = {
            let mut order = Vec::new();
            order.push("cshot-engine");

            if let Some(ref active) = self.active_provider_name {
                if active != "cshot-engine" {
                    if self.providers.iter().any(|p| p.name() == active && p.is_available()) {
                        order.push(active.as_str());
                    }
                }
            }

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

                let timeout_ms = 5000;
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
                        continue;
                    }
                    Err(_) => {
                        errors.push(format!("Provider '{}' timed out after {}ms", name, timeout_ms));
                        continue;
                    }
                };

                let validation = validator::validate_generated_sound(&response.samples);
                if !validation.passed && validation.has_silence {
                    errors.push(format!("Provider '{}' generated silent audio", name));
                    continue;
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
