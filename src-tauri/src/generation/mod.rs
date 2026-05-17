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

/// Build the standard provider registry.
/// The local engine (cshot-engine) is always the default and always available.
/// Cloud providers are registered only for explicit opt-in via Settings.
pub fn build_default_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new().with_defaults();

    // Cloud providers are registered but never auto-selected.
    // They appear in Settings for users who explicitly configure them.
    registry.register(Box::new(placeholder_elevanlabs::ElevenLabsProvider::new()));
    registry.register(Box::new(placeholder_stableaudio::StableAudioProvider::new()));
    registry.register(Box::new(placeholder_audioldm::AudioLdmProvider::new()));

    // Local engine is always the default. No cloud auto-selection.
    registry.set_active("cshot-engine");

    registry
}
