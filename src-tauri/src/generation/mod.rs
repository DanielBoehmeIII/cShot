pub mod bakeoff;
pub mod mock;
pub mod placeholder_elevanlabs;
pub mod placeholder_stableaudio;
pub mod placeholder_audioldm;
pub mod provider;
pub mod registry;
pub mod validator;

#[allow(unused_imports)]
pub use provider::*;
pub use registry::ProviderRegistry;

/// Build the standard provider registry with all known providers.
/// Mock is always available; real providers require env keys.
pub fn build_default_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new().with_defaults();

    registry.register(Box::new(placeholder_elevanlabs::ElevenLabsProvider::new()));
    registry.register(Box::new(placeholder_stableaudio::StableAudioProvider::new()));
    registry.register(Box::new(placeholder_audioldm::AudioLdmProvider::new()));

    if registry.healthy_providers().len() > 1 {
        registry.set_active("elevenlabs");
    }

    registry
}
