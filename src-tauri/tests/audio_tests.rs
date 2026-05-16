use std::path::Path;
use cshot_lib::generation::AudioProvider;

const SAMPLE_RATE: u32 = 44100;

fn generate_sine_wave(freq: f32, duration_ms: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_ms / 1000.0) as usize;
    let mut samples = vec![0.0f32; num_samples];
    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        samples[i] = (2.0 * std::f32::consts::PI * freq * t).sin() * amplitude;
    }
    samples
}

fn generate_silence(duration_ms: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_ms / 1000.0) as usize;
    vec![0.0f32; num_samples]
}

#[test]
fn test_trim_silence_normal_case() {
    let mut samples = generate_sine_wave(440.0, 100.0, 0.5);
    let original_len = samples.len();
    cshot_lib::audio::process::trim_silence(&mut samples, 0.001);
    assert!(!samples.is_empty(), "Trimmed samples should not be empty");
    assert!(samples.len() <= original_len, "Trimmed length should be <= original");
}

#[test]
fn test_trim_silence_empty_input() {
    let mut samples: Vec<f32> = vec![];
    cshot_lib::audio::process::trim_silence(&mut samples, 0.001);
    assert!(samples.is_empty(), "Empty input should stay empty after trim");
}

#[test]
fn test_trim_silence_all_silence() {
    let mut samples = generate_silence(50.0);
    assert!(samples.len() > 100, "Should have many silent samples");
    cshot_lib::audio::process::trim_silence(&mut samples, 0.001);
    assert!(samples.is_empty() || samples.iter().all(|&s| s < 0.001),
        "All-silence input should be trimmed to empty or near-empty");
}

#[test]
fn test_normalize_peak_normal_case() {
    let mut samples = generate_sine_wave(440.0, 100.0, 0.5);
    cshot_lib::audio::process::normalize_peak(&mut samples, -1.0);
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    let target = 10.0f32.powf(-1.0 / 20.0);
    assert!((peak - target).abs() < 0.01, "Peak should be within 0.01 of -1dBFS target");
}

#[test]
fn test_normalize_peak_empty() {
    let mut samples: Vec<f32> = vec![];
    cshot_lib::audio::process::normalize_peak(&mut samples, -1.0);
    assert!(samples.is_empty(), "Empty input should stay empty after normalize");
}

#[test]
fn test_normalize_peak_silence() {
    let mut samples = vec![0.0f32; 100];
    cshot_lib::audio::process::normalize_peak(&mut samples, -1.0);
    assert!(samples.iter().all(|&s| s == 0.0), "Silence should stay silent after normalize");
}

#[test]
fn test_normalize_peak_already_normalized() {
    let target = 10.0f32.powf(-1.0 / 20.0);
    let mut samples = vec![target * 0.8; 100];
    cshot_lib::audio::process::normalize_peak(&mut samples, -1.0);
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!((peak - target).abs() < 0.01, "Peak should be raised to -1dBFS");
}

#[test]
fn test_remove_dc_offset_normal_case() {
    let mut samples = vec![0.5f32; 100];
    samples.extend(vec![-0.5f32; 100]);
    cshot_lib::audio::process::remove_dc_offset(&mut samples);
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    assert!(mean.abs() < 0.001, "DC offset should be removed");
}

#[test]
fn test_remove_dc_offset_no_offset() {
    let sine = generate_sine_wave(440.0, 50.0, 0.5);
    let mut samples = sine.clone();
    cshot_lib::audio::process::remove_dc_offset(&mut samples);
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    assert!(mean.abs() < 0.01, "No-offset signal should stay balanced");
}

#[test]
fn test_apply_fade_normal_case() {
    let mut samples = generate_sine_wave(440.0, 100.0, 1.0);
    let original = samples.clone();
    cshot_lib::audio::process::apply_fade(&mut samples, 0.005, 0.01);
    assert_eq!(samples.len(), original.len(), "Fade should not change length");
    let mid_fade = (0.005 * SAMPLE_RATE as f32 * 0.5) as usize;
    assert!(
        samples[mid_fade].abs() < original[mid_fade].abs(),
        "Mid-fade sample should be quieter after fade-in"
    );
    let last = samples.len() - 1;
    assert!(samples[last].abs() < original[last].abs(), "Last sample should be quieter after fade-out");
}

#[test]
fn test_apply_fade_empty() {
    let mut samples: Vec<f32> = vec![];
    cshot_lib::audio::process::apply_fade(&mut samples, 0.005, 0.01);
    assert!(samples.is_empty(), "Empty input should stay empty after fade");
}

#[test]
fn test_validate_audio_integrity_normal() {
    let samples = generate_sine_wave(440.0, 50.0, 0.5);
    assert!(cshot_lib::audio::process::validate_audio_integrity(&samples).is_ok());
}

#[test]
fn test_validate_audio_integrity_empty() {
    let samples: Vec<f32> = vec![];
    assert!(cshot_lib::audio::process::validate_audio_integrity(&samples).is_err());
}

#[test]
fn test_validate_audio_integrity_nan() {
    let samples = vec![std::f32::NAN; 100];
    assert!(cshot_lib::audio::process::validate_audio_integrity(&samples).is_err());
}

#[test]
fn test_validate_audio_integrity_inf() {
    let samples = vec![std::f32::INFINITY; 100];
    assert!(cshot_lib::audio::process::validate_audio_integrity(&samples).is_err());
}

#[test]
fn test_compute_rms_silence() {
    let samples = vec![0.0f32; 1000];
    let rms = cshot_lib::audio::compute_rms(&samples);
    assert!(rms < 0.001, "RMS of silence should be near zero");
}

#[test]
fn test_compute_rms_sine() {
    let samples = generate_sine_wave(440.0, 100.0, 1.0);
    let rms = cshot_lib::audio::compute_rms(&samples);
    let expected = 1.0 / (2.0f32).sqrt();
    assert!((rms - expected).abs() < 0.05, "RMS of unit sine should be ~0.707");
}

#[test]
fn test_compute_peak_sine() {
    let samples = generate_sine_wave(440.0, 100.0, 0.8);
    let peak = cshot_lib::audio::compute_peak(&samples);
    assert!((peak - 0.8).abs() < 0.01, "Peak of 0.8-amplitude sine should be ~0.8");
}

#[test]
fn test_compute_peak_silence() {
    let samples = vec![0.0f32; 100];
    let peak = cshot_lib::audio::compute_peak(&samples);
    assert!(peak < 0.001, "Peak of silence should be near zero");
}

#[test]
fn test_compute_spectral_centroid_sine() {
    let samples = generate_sine_wave(440.0, 100.0, 0.5);
    let centroid = cshot_lib::audio::compute_spectral_centroid(&samples);
    assert!(centroid > 0.0, "Spectral centroid of sine should be positive");
}

#[test]
fn test_compute_spectral_centroid_silence() {
    let samples = vec![0.0f32; 100];
    let centroid = cshot_lib::audio::compute_spectral_centroid(&samples);
    assert!(centroid < 0.001, "Spectral centroid of silence should be near zero");
}

#[test]
fn test_compute_crest_factor_sine() {
    let samples = generate_sine_wave(440.0, 100.0, 1.0);
    let crest = cshot_lib::audio::compute_crest_factor(&samples);
    assert!((crest - (2.0f32).sqrt()).abs() < 0.1, "Crest factor of sine should be ~sqrt(2)");
}

#[test]
fn test_compute_crest_factor_silence() {
    let samples = vec![0.0f32; 100];
    let crest = cshot_lib::audio::compute_crest_factor(&samples);
    assert!(crest >= 1.0, "Crest factor should be at least 1.0");
}

#[test]
fn test_compute_zero_crossing_rate_sine() {
    let samples = generate_sine_wave(440.0, 50.0, 0.5);
    let zcr = cshot_lib::audio::compute_zero_crossing_rate(&samples);
    assert!(zcr > 0.0, "Zero crossing rate of sine should be positive");
}

#[test]
fn test_compute_zero_crossing_rate_constant() {
    let samples = vec![0.5f32; 100];
    let zcr = cshot_lib::audio::compute_zero_crossing_rate(&samples);
    assert!(zcr < 0.001, "Zero crossing rate of constant signal should be near zero");
}

#[test]
fn test_waveform_computation() {
    let samples = generate_sine_wave(440.0, 100.0, 0.8);
    let waveform = cshot_lib::audio::compute_waveform(&samples, 80);
    assert_eq!(waveform.len(), 80, "Waveform should have exactly 80 points");
    assert!(waveform.iter().any(|&v| v > 0.0), "Waveform of sine should have non-zero values");
}

#[test]
fn test_waveform_empty() {
    let samples: Vec<f32> = vec![];
    let waveform = cshot_lib::audio::compute_waveform(&samples, 80);
    assert_eq!(waveform.len(), 80, "Empty input should still return 80 points of zeros");
    assert!(waveform.iter().all(|&v| v == 0.0), "Empty input waveform should be all zeros");
}

#[test]
fn test_high_pass_filter() {
    let mut samples = generate_sine_wave(100.0, 50.0, 0.5);
    let original_rms = cshot_lib::audio::compute_rms(&samples);
    cshot_lib::audio::dsp::high_pass(&mut samples, 500.0);
    let filtered_rms = cshot_lib::audio::compute_rms(&samples);
    assert!(filtered_rms < original_rms, "High-pass should reduce energy of low frequency");
}

#[test]
fn test_low_pass_filter() {
    let mut samples = generate_sine_wave(5000.0, 50.0, 0.5);
    let original_rms = cshot_lib::audio::compute_rms(&samples);
    cshot_lib::audio::dsp::low_pass(&mut samples, 1000.0);
    let filtered_rms = cshot_lib::audio::compute_rms(&samples);
    assert!(filtered_rms < original_rms, "Low-pass should reduce energy of high frequency");
}

#[test]
fn test_apply_punch_increases_attack() {
    let mut samples = generate_sine_wave(200.0, 50.0, 0.5);
    let attack_samples = (3 * SAMPLE_RATE as usize) / 1000;
    let check_idx = attack_samples / 2;
    let original_val = samples[check_idx];
    cshot_lib::audio::dsp::apply_punch(&mut samples);
    assert!(
        samples[check_idx] > original_val,
        "Punch should boost early samples"
    );
}

#[test]
fn test_pitch_shift_up() {
    let samples = generate_sine_wave(440.0, 50.0, 0.5);
    let shifted = cshot_lib::audio::dsp::pitch_shift(&samples, 2.0);
    assert!(shifted.len() < samples.len(), "Pitch shift up should shorten duration");
}

#[test]
fn test_pitch_shift_down() {
    let samples = generate_sine_wave(440.0, 50.0, 0.5);
    let shifted = cshot_lib::audio::dsp::pitch_shift(&samples, 0.5);
    assert!(shifted.len() > samples.len(), "Pitch shift down should lengthen duration");
}

#[test]
fn test_compute_energy_sub_low() {
    let sub_samples = generate_sine_wave(60.0, 100.0, 0.5);
    let sub_energy = cshot_lib::audio::compute_energy_sub_low(&sub_samples);
    assert!(
        sub_energy >= 0.0 && sub_energy <= 1.0,
        "Sub energy ratio should be between 0 and 1"
    );
}

#[test]
fn test_apply_envelope_attack() {
    let mut samples = generate_sine_wave(440.0, 100.0, 1.0);
    cshot_lib::audio::dsp::apply_envelope(&mut samples, 0.01, 0.01);
    let attack_samples = (0.01 * SAMPLE_RATE as f32) as usize;
    let mid_attack = attack_samples / 2;
    let original = (2.0 * std::f32::consts::PI * 440.0 * (mid_attack as f32 / SAMPLE_RATE as f32)).sin();
    assert!(
        samples[mid_attack].abs() < original.abs(),
        "Mid-attack sample should be quieter after envelope"
    );
}

#[test]
fn test_process_sound_normal() {
    let mut samples = generate_sine_wave(440.0, 100.0, 0.8);
    let dsp = cshot_lib::audio::DspParams::default();
    let tags = cshot_lib::audio::process::process_sound(
        &mut samples,
        &dsp,
        cshot_lib::audio::SoundType::Kick,
    );
    assert!(!samples.is_empty(), "Processed samples should not be empty");
    assert!(!tags.is_empty(), "Should have auto-tags");
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    let target = 10.0f32.powf(-1.0 / 20.0);
    assert!((peak - target).abs() < 0.01, "Processed peak should be normalized to -1dBFS");
}

#[test]
fn test_synthesize_kick() {
    let samples = cshot_lib::audio::synthesize::generate_base(
        cshot_lib::audio::SoundType::Kick, 500.0,
    );
    assert!(!samples.is_empty(), "Kick synthesis should produce samples");
    let rms = cshot_lib::audio::compute_rms(&samples);
    assert!(rms > 0.001, "Kick synthesis should produce non-silent audio");
}

#[test]
fn test_synthesize_snare() {
    let samples = cshot_lib::audio::synthesize::generate_base(
        cshot_lib::audio::SoundType::Snare, 300.0,
    );
    assert!(!samples.is_empty(), "Snare synthesis should produce samples");
    let rms = cshot_lib::audio::compute_rms(&samples);
    assert!(rms > 0.001, "Snare synthesis should produce non-silent audio");
}

#[test]
fn test_synthesize_all_types() {
    let types = [
        cshot_lib::audio::SoundType::Kick,
        cshot_lib::audio::SoundType::Snare,
        cshot_lib::audio::SoundType::ClosedHat,
        cshot_lib::audio::SoundType::OpenHat,
        cshot_lib::audio::SoundType::Clap,
        cshot_lib::audio::SoundType::Tom,
        cshot_lib::audio::SoundType::Perc,
        cshot_lib::audio::SoundType::Bass,
        cshot_lib::audio::SoundType::Fx,
        cshot_lib::audio::SoundType::Other,
    ];
    for sound_type in &types {
        let samples = cshot_lib::audio::synthesize::generate_base(*sound_type, 300.0);
        assert!(!samples.is_empty(), "{:?} synthesis should produce samples", sound_type);
        let rms = cshot_lib::audio::compute_rms(&samples);
        assert!(rms > 0.001, "{:?} synthesis should produce non-silent audio", sound_type);
    }
}

#[test]
fn test_validate_upload_missing_file() {
    let result = cshot_lib::audio::validate::validate_upload(
        Path::new("/nonexistent/file.wav"), 0,
    );
    assert!(!result.is_valid, "Missing file should fail validation");
    assert!(result.error.is_some(), "Should have error message");
}

#[test]
fn test_validate_upload_empty() {
    let result = cshot_lib::audio::validate::validate_upload(
        Path::new("test.wav"), 0,
    );
    assert!(!result.is_valid, "Empty file should fail validation");
}

#[test]
fn test_sound_type_as_str() {
    use cshot_lib::audio::SoundType;
    assert_eq!(SoundType::Kick.as_str(), "kick");
    assert_eq!(SoundType::Snare.as_str(), "snare");
    assert_eq!(SoundType::ClosedHat.as_str(), "closed_hat");
    assert_eq!(SoundType::OpenHat.as_str(), "open_hat");
    assert_eq!(SoundType::Clap.as_str(), "clap");
    assert_eq!(SoundType::Tom.as_str(), "tom");
    assert_eq!(SoundType::Perc.as_str(), "perc");
    assert_eq!(SoundType::Bass.as_str(), "bass");
    assert_eq!(SoundType::Fx.as_str(), "fx");
    assert_eq!(SoundType::Other.as_str(), "other");
}

#[test]
fn test_sound_type_from_str() {
    use cshot_lib::audio::SoundType;
    assert_eq!(SoundType::from_str("kick"), SoundType::Kick);
    assert_eq!(SoundType::from_str("snare"), SoundType::Snare);
    assert_eq!(SoundType::from_str("closed_hat"), SoundType::ClosedHat);
    assert_eq!(SoundType::from_str("open_hat"), SoundType::OpenHat);
    assert_eq!(SoundType::from_str("clap"), SoundType::Clap);
    assert_eq!(SoundType::from_str("tom"), SoundType::Tom);
    assert_eq!(SoundType::from_str("perc"), SoundType::Perc);
    assert_eq!(SoundType::from_str("bass"), SoundType::Bass);
    assert_eq!(SoundType::from_str("fx"), SoundType::Fx);
    assert_eq!(SoundType::from_str("unknown"), SoundType::Other);
}

#[test]
fn test_dsp_params_default() {
    let dsp = cshot_lib::audio::DspParams::default();
    assert!(!dsp.low_pass);
    assert!(!dsp.high_pass);
    assert!(!dsp.punch);
    assert!(!dsp.bright);
    assert!(!dsp.dark);
    assert_eq!(dsp.gain, 1.0);
    assert_eq!(dsp.noise_amt, 0.0);
    assert_eq!(dsp.decay_factor, 1.0);
}

// ─── Provider Tests ───────────────────────────────────────────

#[test]
fn test_elevenlabs_provider_creation() {
    let provider = cshot_lib::generation::placeholder_elevanlabs::ElevenLabsProvider::new();
    assert_eq!(provider.name(), "elevenlabs");
    assert!(!provider.is_available(), "Should not be available without API key in test env");
    let reason = provider.reason_unavailable();
    assert!(reason.is_some(), "Should provide reason when unavailable");
    assert!(reason.unwrap().contains("CSHOT_ELEVENLABS_API_KEY"), "Reason should mention the env var");
}

#[test]
fn test_elevenlabs_provider_capabilities() {
    let provider = cshot_lib::generation::placeholder_elevanlabs::ElevenLabsProvider::new();
    let caps = provider.capabilities();
    assert!(caps.requires_api_key, "ElevenLabs should require API key");
    assert!(caps.requires_network, "ElevenLabs should require network");
    assert_eq!(caps.estimated_latency_ms, 4000);
    assert!(caps.estimated_cost_per_generation_cents > 0.0);
}

#[test]
fn test_stableaudio_provider_creation() {
    let provider = cshot_lib::generation::placeholder_stableaudio::StableAudioProvider::new();
    assert_eq!(provider.name(), "stable-audio");
    assert!(!provider.is_available(), "Should not be available without API key in test env");
    let reason = provider.reason_unavailable();
    assert!(reason.is_some(), "Should provide reason when unavailable");
}

#[test]
fn test_stableaudio_provider_capabilities() {
    let provider = cshot_lib::generation::placeholder_stableaudio::StableAudioProvider::new();
    let caps = provider.capabilities();
    assert!(caps.requires_api_key, "Stable Audio should require API key");
    assert!(caps.requires_network, "Stable Audio should require network");
    assert!(!caps.supports_reference_audio, "Stable Audio API should not support reference audio");
}

#[test]
fn test_audioldm_provider_creation() {
    let provider = cshot_lib::generation::placeholder_audioldm::AudioLdmProvider::new();
    assert_eq!(provider.name(), "audioldm2");
    assert!(!provider.is_available(), "Should not be available by default");
    let reason = provider.reason_unavailable();
    assert!(reason.is_some(), "Should provide reason when unavailable");
}

#[test]
fn test_audioldm_provider_capabilities() {
    let provider = cshot_lib::generation::placeholder_audioldm::AudioLdmProvider::new();
    let caps = provider.capabilities();
    assert!(!caps.requires_api_key, "AudioLDM should not require API key");
    assert!(!caps.requires_network, "AudioLDM should not require network");
    assert!(caps.supports_reference_audio, "AudioLDM should support reference audio");
}

#[test]
fn test_provider_registry_default_build() {
    let registry = cshot_lib::generation::build_default_registry();
    assert_eq!(registry.provider_count(), 4, "Should have 4 providers: mock + elevenlabs + stableaudio + audioldm");

    let all_caps = registry.all_provider_metadata();
    assert_eq!(all_caps.len(), 4, "Should have metadata for all 4 providers");

    let names: Vec<&str> = all_caps.iter().map(|c| c.name).collect();
    assert!(names.contains(&"Mock DSP Synthesizer"));
    assert!(names.contains(&"ElevenLabs Text-to-Sound-Effects"));
    assert!(names.contains(&"Stable Audio (API)"));
    assert!(names.contains(&"AudioLDM 2 (Self-Hosted)"));
}

#[test]
fn test_provider_registry_mock_always_available() {
    let registry = cshot_lib::generation::build_default_registry();
    let healthy = registry.healthy_providers();
    assert!(healthy.iter().any(|p| p.name() == "mock-dsp"), "Mock provider should always be healthy");
}

#[test]
fn test_provider_registry_fallback_chain() {
    let registry = cshot_lib::generation::build_default_registry();
    let available = registry.available_providers();
    assert!(available.iter().any(|p| p.name() == "mock-dsp"), "Mock should be in available providers");

    // mock-dsp should be the active provider when no API keys are set
    let active = registry.active_provider();
    assert!(active.is_some(), "Should have an active provider");
    if let Some(active) = active {
        assert_eq!(active.name(), "mock-dsp", "Mock-dsp should be active by default when no API keys");
    }
}

#[test]
fn test_provider_registration_deduplicates() {
    use cshot_lib::generation::ProviderRegistry;
    let mut registry = ProviderRegistry::new();

    let mock1 = Box::new(cshot_lib::generation::mock::MockProvider);
    let mock2 = Box::new(cshot_lib::generation::mock::MockProvider);

    registry.register(mock1);
    registry.register(mock2);  // Duplicate name - should be ignored

    let all = registry.available_providers();
    let mock_count = all.iter().filter(|p| p.name() == "mock-dsp").count();
    assert_eq!(mock_count, 1, "Duplicate provider names should be deduplicated");
}

#[test]
fn test_generation_request_with_prompt() {
    let req = cshot_lib::generation::provider::GenerationRequest::from_prompt("punchy kick 140");

    assert_eq!(req.prompt, "punchy kick 140");
    assert!(req.reference_audio.is_none());
    assert!(req.reference_sample_rate.is_none());
    assert!(req.duration_ms.is_none());
    assert!(req.seed.is_none());
}

#[test]
fn test_generation_request_with_reference() {
    let req = cshot_lib::generation::provider::GenerationRequest::from_prompt("punchy kick")
        .with_reference(vec![0.0, 0.1, 0.2], 44100)
        .with_duration(500.0)
        .with_seed(42);

    assert_eq!(req.prompt, "punchy kick");
    assert!(req.reference_audio.is_some());
    assert_eq!(req.reference_sample_rate, Some(44100));
    assert_eq!(req.duration_ms, Some(500.0));
    assert_eq!(req.seed, Some(42));
}

#[test]
fn test_generation_error_display() {
    use cshot_lib::generation::provider::GenerationError;

    let err = GenerationError::ApiKeyMissing("test".to_string());
    assert!(err.to_string().contains("API key missing"));

    let err = GenerationError::NetworkError("timeout".to_string());
    assert!(err.to_string().contains("Network error"));

    let err = GenerationError::ProviderUnavailable("down".to_string());
    assert!(err.to_string().contains("Provider unavailable"));

    let err = GenerationError::SilentOutput("empty".to_string());
    assert!(err.to_string().contains("Silent output"));

    let err = GenerationError::Timeout("slow".to_string());
    assert!(err.to_string().contains("Timeout"));

    let err = GenerationError::CorruptedOutput("bad".to_string());
    assert!(err.to_string().contains("Corrupted output"));
}
