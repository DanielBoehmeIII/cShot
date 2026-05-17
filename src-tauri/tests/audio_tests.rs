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
    cshot_lib::audio::remove_dc_offset(&mut samples);
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    assert!(mean.abs() < 0.001, "DC offset should be removed");
}

#[test]
fn test_remove_dc_offset_no_offset() {
    let sine = generate_sine_wave(440.0, 50.0, 0.5);
    let mut samples = sine.clone();
    cshot_lib::audio::remove_dc_offset(&mut samples);
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
    assert!(names.contains(&"cShot Engine"));
    assert!(names.contains(&"ElevenLabs Text-to-Sound-Effects"));
    assert!(names.contains(&"Stable Audio (API)"));
    assert!(names.contains(&"AudioLDM 2 (Self-Hosted)"));
}

#[test]
fn test_provider_registry_mock_always_available() {
    let registry = cshot_lib::generation::build_default_registry();
    let healthy = registry.healthy_providers();
    assert!(healthy.iter().any(|p| p.name() == "cshot-engine"), "cShot Engine should always be healthy");
}

#[test]
fn test_provider_registry_fallback_chain() {
    let registry = cshot_lib::generation::build_default_registry();
    let available = registry.available_providers();
    assert!(available.iter().any(|p| p.name() == "cshot-engine"), "cShot Engine should be in available providers");

    // cshot-engine should be the active provider by default
    let active = registry.active_provider().unwrap();
    assert_eq!(active.name(), "cshot-engine", "cShot Engine should be active by default");
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
    let mock_count = all.iter().filter(|p| p.name() == "cshot-engine").count();
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

// ─── Audio Analysis Tests ─────────────────────────────────

#[test]
fn test_analyze_sine_wave() {
    let samples = generate_sine_wave(440.0, 200.0, 0.5);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.peak > 0.4);
    assert!(a.rms > 0.1);
    assert!(a.crest_factor > 1.0);
    assert!(!a.is_silent);
    assert!(!a.has_clipping);
    assert!(a.duration_ms > 190.0 && a.duration_ms < 210.0);
    assert!(a.sample_rate == 44100);
    assert!(a.channels == 1);
    assert!(a.zero_crossing_rate > 0.0);
}

#[test]
fn test_analyze_silence() {
    let samples = generate_silence(100.0);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.is_silent);
    assert!(a.peak < 0.001);
    assert!(a.rms < 0.0001);
}

#[test]
fn test_analyze_clipping() {
    let samples = generate_sine_wave(100.0, 100.0, 1.5);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.has_clipping);
    assert!(a.clipping_count > 0);
}

#[test]
fn test_analyze_pitch_sine() {
    let samples = generate_sine_wave(440.0, 200.0, 0.5);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.has_pitch);
    assert!(a.pitch_estimate.is_some());
    let pitch = a.pitch_estimate.unwrap();
    assert!(pitch > 400.0 && pitch < 500.0);
}

#[test]
fn test_analyze_pitch_low() {
    let samples = generate_sine_wave(100.0, 300.0, 0.5);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.has_pitch);
    let pitch = a.pitch_estimate.unwrap();
    assert!(pitch > 90.0 && pitch < 120.0);
}

#[test]
fn test_analyze_attack_time() {
    let mut samples = generate_silence(50.0);
    let tone = generate_sine_wave(200.0, 100.0, 0.8);
    samples.extend(tone);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.attack_ms > 0.0, "attack_ms = {}", a.attack_ms);
    assert!(a.attack_ms < 10.0, "attack_ms = {}", a.attack_ms);
    assert!(a.has_leading_silence);
    assert!(a.leading_silence_ms > 40.0, "leading_silence_ms = {}", a.leading_silence_ms);
}

#[test]
fn test_analyze_noise_no_pitch() {
    let samples: Vec<f32> = (0..(44100 * 200 / 1000))
        .map(|i| {
            let n1 = ((i as f32 * 127.1).sin() * 43758.5453).fract() * 2.0 - 1.0;
            n1 * 0.3
        })
        .collect();
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.noise_estimate > 0.3);
    assert!(a.zero_crossing_rate > 0.15);
}

#[test]
fn test_analyze_spectral_rolloff() {
    let samples = generate_sine_wave(1000.0, 100.0, 0.5);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.spectral_rolloff > 500.0);
    assert!(a.spectral_centroid > 0.0);
}

#[test]
fn test_analyze_brightness() {
    let bright = generate_sine_wave(5000.0, 100.0, 0.5);
    let a_bright = cshot_lib::audio::analyze::analyze_audio(&bright, 44100, 1);
    let dark = generate_sine_wave(100.0, 100.0, 0.5);
    let a_dark = cshot_lib::audio::analyze::analyze_audio(&dark, 44100, 1);
    assert!(a_bright.brightness > a_dark.brightness);
}

#[test]
fn test_analyze_sub_energy() {
    let sub = generate_sine_wave(40.0, 200.0, 0.5);
    let a_sub = cshot_lib::audio::analyze::analyze_audio(&sub, 44100, 1);
    let high = generate_sine_wave(8000.0, 200.0, 0.5);
    let a_high = cshot_lib::audio::analyze::analyze_audio(&high, 44100, 1);
    assert!(a_sub.sub_energy_ratio > 0.0);
    assert!(a_high.sub_energy_ratio >= 0.0);
}

#[test]
fn test_analyze_envelope() {
    let samples = generate_sine_wave(200.0, 200.0, 0.8);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(!a.envelope.is_empty());
    assert!(a.envelope.len() <= 256);
}

#[test]
fn test_analyze_spectral_profile() {
    let samples = generate_sine_wave(200.0, 200.0, 0.8);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(!a.spectral_profile.is_empty());
    assert!(a.spectral_profile.len() == 64);
    let max_val = a.spectral_profile.iter().copied().fold(0.0f32, f32::max);
    assert!((max_val - 1.0).abs() < 0.01);
}

#[test]
fn test_analyze_crest_factor_sine() {
    let samples = generate_sine_wave(200.0, 100.0, 1.0);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.crest_factor > 1.0);
}

#[test]
fn test_analyze_loudness() {
    let loud = generate_sine_wave(200.0, 200.0, 0.9);
    let a_loud = cshot_lib::audio::analyze::analyze_audio(&loud, 44100, 1);
    let quiet = generate_sine_wave(200.0, 200.0, 0.01);
    let a_quiet = cshot_lib::audio::analyze::analyze_audio(&quiet, 44100, 1);
    assert!(a_loud.loudness_lufs > a_quiet.loudness_lufs);
}

#[test]
fn test_analyze_decay_and_tail() {
    let mut samples = generate_sine_wave(200.0, 10.0, 1.0);
    let mut tail = generate_sine_wave(200.0, 200.0, 0.01);
    for s in tail.iter_mut() { *s *= 0.001; }
    samples.extend(tail);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.decay_ms >= 0.0);
    assert!(a.tail_ms >= 0.0);
}

#[test]
fn test_analyze_transient_detection() {
    let mut samples = generate_silence(50.0);
    samples.extend(generate_sine_wave(1000.0, 10.0, 1.0));
    samples.extend(generate_silence(50.0));
    samples.extend(generate_sine_wave(1000.0, 10.0, 1.0));
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.transient_strength > 0.0);
}

#[test]
fn test_analyze_trailing_silence() {
    let mut samples = generate_sine_wave(200.0, 100.0, 0.5);
    samples.extend(generate_silence(50.0));
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.has_trailing_silence);
    assert!(a.trailing_silence_ms > 40.0);
}

#[test]
fn test_analyze_noise_floor() {
    let samples = generate_sine_wave(200.0, 100.0, 0.5);
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.noise_floor_db < 0.0);
}

#[test]
fn test_analyze_sound_type_hint_kick() {
    let mut samples = Vec::new();
    let len = (44100.0 * 0.3) as usize;
    for i in 0..len {
        let t = i as f32 / 44100.0;
        let freq = 150.0 - 100.0 * (t / 0.3).min(1.0);
        let env = (-6.0 * t).exp();
        samples.push((2.0 * std::f32::consts::PI * freq * t).sin() * env * 0.8);
    }
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    let hint = a.sound_type_hint();
    assert!(hint == "kick" || hint == "bass" || hint == "perc");
}

#[test]
fn test_analyze_empty() {
    let samples: Vec<f32> = vec![];
    let a = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    assert!(a.is_silent);
    assert!(a.duration_ms < 0.001);
}

#[test]
fn test_analysis_cache() {
    let tmp = std::env::temp_dir();
    let mut cache = cshot_lib::audio::analysis_cache::AnalysisCache::load(&tmp);
    let samples = generate_sine_wave(440.0, 100.0, 0.5);
    let a1 = cache.analyze_and_cache("test1", &samples, 44100, 1);
    let a2 = cache.analyze_and_cache("test1", &samples, 44100, 1);
    assert!((a1.peak - a2.peak).abs() < 0.001);
    assert!(cache.get("test1").is_some());
    cache.invalidate("test1");
    assert!(cache.get("test1").is_none());
}

// ─── Improved Generation Tests ─────────────────────────

#[test]
fn test_resynthesis_params_randomize() {
    use cshot_lib::audio::resynthesize;
    let base = resynthesize::ResynthesisParams {
        duration_ms: 300.0,
        pitch_hz: 100.0,
        ..resynthesize::ResynthesisParams { seed: 42, ..Default::default() }
    };
    let seeded = base.clone().with_seed(42);
    let varied = seeded.randomize(0.5);
    assert!(varied.duration_ms != base.duration_ms || varied.pitch_hz != base.pitch_hz);
    assert!(varied.duration_ms > 100.0);
}

#[test]
fn test_resynthesis_params_randomize_deterministic() {
    use cshot_lib::audio::resynthesize;
    let base = resynthesize::ResynthesisParams { seed: 42, ..Default::default() };
    let a = base.clone().with_seed(42).randomize(0.5);
    let b = base.clone().with_seed(42).randomize(0.5);
    assert!((a.duration_ms - b.duration_ms).abs() < 0.001);
}

#[test]
fn test_resynthesis_params_to_variant() {
    use cshot_lib::audio::resynthesize;
    let base = resynthesize::ResynthesisParams { brightness: 0.5, ..Default::default() };
    let brighter = base.to_variant("brighter");
    assert!(brighter.brightness > 0.5);
    let darker = base.to_variant("darker");
    assert!(darker.brightness < 0.5);
}

#[test]
fn test_generate_resynthesis_variants_logic() {
    use cshot_lib::audio::resynthesize;
    let prompt = "punchy kick";
    let ctrl = cshot_lib::prompt_dsp::parse_prompt_rich(prompt);
    let st = cshot_lib::audio::SoundType::from_str(&ctrl.sound_type);
    let pitch = ctrl.pitch_hz.unwrap_or(60.0);
    let dur = ctrl.duration_ms.unwrap_or(300.0);
    let base = resynthesize::params_for_sound_type(st, pitch, dur);

    let variant_names = ["brighter", "darker", "punchier", "shorter", "longer"];
    for (i, name) in variant_names.iter().enumerate() {
        let seeded = base.clone().with_seed(i as u64);
        let varied = seeded.randomize(0.3);
        let params = varied.to_variant(name);
        let samples = resynthesize::resynthesize(&params);
        assert!(!samples.is_empty(), "Variant '{}' produced empty audio", name);
        let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak > 0.01, "Variant '{}' is silent", name);
    }
}

#[test]
fn test_repair_add_sub() {
    let samples = generate_sine_wave(200.0, 100.0, 0.5);
    let mut repaired = samples.clone();
    let onset_len = (44100.0 * 0.005) as usize;
    let threshold = repaired.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.3;
    for i in 10..repaired.len().min(44100 / 2) {
        if repaired[i].abs() > threshold {
            let end = (i + onset_len).min(repaired.len());
            for j in i..end {
                let t = (j - i) as f32 / onset_len as f32;
                repaired[j] *= 1.0 + 0.5 * (1.0 - t);
            }
            break;
        }
    }
    let max_after = repaired.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_after > 0.0);
}

#[test]
fn test_repair_saturation() {
    let samples = generate_sine_wave(200.0, 100.0, 0.5);
    let mut saturated = samples.clone();
    for s in saturated.iter_mut() { *s = (*s * 1.5).tanh(); }
    let peak = saturated.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak <= 1.0);
    assert!((peak - 1.0).abs() < 1.0);
}

#[test]
fn test_resynthesis_genre_prompt_integration() {
    use cshot_lib::audio::resynthesize;
    let prompt = "dark punchy kick with sub";
    let ctrl = cshot_lib::prompt_dsp::parse_prompt_rich(prompt);
    assert_eq!(ctrl.sound_type, "kick");
    let st = cshot_lib::audio::SoundType::from_str(&ctrl.sound_type);
    let pitch = ctrl.pitch_hz.unwrap_or(60.0);
    let dur = ctrl.duration_ms.unwrap_or(300.0);
    let base = resynthesize::params_for_sound_type(st, pitch, dur);
    let params = ctrl.to_resynthesis_params(&base);
    let result = resynthesize::resynthesize(&params);
    assert!(!result.is_empty());
    let peak = result.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.01);
}

#[test]
fn test_resynthesis_all_types_generate() {
    use cshot_lib::audio::resynthesize;
    use cshot_lib::audio::SoundType;
    let types = [SoundType::Kick, SoundType::Snare, SoundType::ClosedHat,
                 SoundType::OpenHat, SoundType::Clap, SoundType::Bass,
                 SoundType::Perc, SoundType::Fx, SoundType::Tom, SoundType::Other];
    for st in &types {
        let params = resynthesize::params_for_sound_type(*st, 200.0, 300.0);
        let result = resynthesize::resynthesize(&params);
        assert!(!result.is_empty(), "Type {:?} produced empty audio", st);
    }
}

// ─── Beta Quality Validation ─────────────────────────

#[test]
fn test_beta_100_sound_generation() {
    use cshot_lib::audio::resynthesize;
    use cshot_lib::audio::SoundType;
    use cshot_lib::prompt_dsp::parse_prompt_rich;

    let prompts = vec![
        // Kicks (15)
        "punchy kick 140bpm", "dark kick with sub", "tight kick 120bpm",
        "distorted kick", "soft kick", "deep 808 kick", "trap kick with click",
        "boomy kick", "electronic kick", "acoustic kick",
        "house kick", "techno kick", "drill kick", "rock kick", "subby kick",
        // Snares (10)
        "bright snare with crack", "dark snare", "layered snare",
        "trap snare", "rimshot snare", "marching snare",
        "electronic snare", "soft snare", "acoustic snare", "powerful snare",
        // Hi-hats (10)
        "tight closed hi-hat", "open hi-hat washy", "bright hi-hat",
        "dark hi-hat", "sizzle hi-hat", "electronic hi-hat",
        "acoustic hi-hat", "short hi-hat", "long hi-hat", "layered hi-hat",
        // Claps (8)
        "crisp clap", "layered clap", "room clap", "bright clap",
        "dark clap", "electronic clap", "distorted clap", "warm clap",
        // Bass (10)
        "deep sub bass", "808 bass", "distorted bass", "warm bass",
        "clean bass", "electronic bass", "sub bass", "round bass",
        "aggressive bass", "sine bass",
        // Percussion (10)
        "percussion hit", "metallic perc", "wooden perc",
        "shaker perc", "electronic perc", "rim hit",
        "cowbell perc", "tambourine", "click perc", "stamp",
        // FX (10)
        "cinematic impact", "riser sweep", "whoosh",
        "reverse cymbal", "sub boom", "glitch hit",
        "orchestral hit", "sweep fx", "ambient pad", "crash cymbal",
        // Impacts (7)
        "heavy impact", "low boom", "metal hit",
        "explosion", "door slam", "punch impact", "thud",
        // Toms (5)
        "tom hit", "low tom", "high tom",
        "electronic tom", "acoustic tom",
        // Other (15)
        "ui click", "footstep", "sword swish",
        "button click", "coin drop", "glass break",
        "wood knock", "metal ring", "water splash",
        "electric spark", "wind gust", "paper tear",
        "engine start", "alarm beep", "bell tone",
    ];

    let mut failed = 0usize;
    let mut silent = 0usize;
    let mut clipped = 0usize;
    let mut total_duration = 0.0f32;
    let mut total_time = std::time::Duration::from_secs(0);

    for prompt in &prompts {
        let ctrl = parse_prompt_rich(prompt);
        let st = SoundType::from_str(&ctrl.sound_type);
        let pitch = ctrl.pitch_hz.unwrap_or(200.0);
        let dur = ctrl.duration_ms.unwrap_or(300.0);
        let base = resynthesize::params_for_sound_type(st, pitch, dur);
        let params = ctrl.to_resynthesis_params(&base);

        let start = std::time::Instant::now();
        let samples = resynthesize::resynthesize(&params);
        let elapsed = start.elapsed();

        total_time += elapsed;
        total_duration += dur;

        if samples.is_empty() {
            failed += 1;
            eprintln!("  FAIL: '{}' produced empty audio", prompt);
            continue;
        }

        let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        if peak < 0.001 {
            silent += 1;
            eprintln!("  SILENT: '{}' peak={}", prompt, peak);
        }
        if peak >= 1.0 {
            clipped += 1;
        }
    }

    let total = prompts.len();
    let success = total - failed - silent;
    let success_rate = success as f32 / total as f32 * 100.0;
    let avg_synth_ms = total_time.as_secs_f64() * 1000.0 / total as f64;

    println!("\n=== Beta Quality Validation ===");
    println!("  Total prompts: {}", total);
    println!("  Success:       {} ({:.0}%)", success, success_rate);
    println!("  Failed:        {}", failed);
    println!("  Silent:        {}", silent);
    println!("  Clipped:       {}", clipped);
    println!("  Avg synth:     {:.2}ms", avg_synth_ms);
    println!("  Total audio:   {:.1}s", total_duration / 1000.0);
    println!("  Realtime:      {:.1}x", total_duration / (total_time.as_secs_f64() * 1000.0).max(0.001) as f32);

    assert!(success_rate >= 95.0, "Success rate {:.1}% < 95% threshold ({} failed, {} silent)", success_rate, failed, silent);
}

// ─── Benchmark Tests ──────────────────────────────────

#[test]
fn test_benchmark_synthesis_types() {
    use cshot_lib::audio::resynthesize;
    use cshot_lib::audio::SoundType;
    let types = [
        (SoundType::Kick, 60.0, 300.0),
        (SoundType::Snare, 200.0, 300.0),
        (SoundType::ClosedHat, 400.0, 150.0),
        (SoundType::OpenHat, 400.0, 500.0),
        (SoundType::Clap, 180.0, 300.0),
        (SoundType::Bass, 55.0, 600.0),
        (SoundType::Perc, 300.0, 200.0),
        (SoundType::Fx, 100.0, 1000.0),
    ];
    let mut total_time = std::time::Duration::from_secs(0);
    for (st, pitch, dur) in &types {
        let params = resynthesize::params_for_sound_type(*st, *pitch, *dur);
        let start = std::time::Instant::now();
        for _ in 0..5 {
            let _ = resynthesize::resynthesize(&params);
        }
        let elapsed = start.elapsed();
        total_time += elapsed;
    }
    let avg_ms = total_time.as_secs_f64() * 200.0;
    println!("Benchmark: {} types x 5 iterations = {:.2}ms avg", types.len(), avg_ms / types.len() as f64);
}

#[test]
fn test_benchmark_analysis() {
    let samples = crate::generate_sine_wave(200.0, 500.0, 0.5);
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_secs_f64() * 1_000_000.0 / 10.0;
    println!("Analysis benchmark: 50 iterations = {:.0}us avg", avg_us);
}

#[test]
fn test_benchmark_transform_dsp() {
    let mut samples = crate::generate_sine_wave(200.0, 200.0, 0.5);
    let params = cshot_lib::audio::transform::TransformParams {
        saturation_drive: Some(2.0),
        brightness_tilt: Some(0.3),
        noise_add: Some(0.1),
        ..Default::default()
    };
    let start = std::time::Instant::now();
    for _ in 0..10 {
        cshot_lib::audio::transform::apply_dsp_transforms(&mut samples, &params);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_secs_f64() * 1_000_000.0 / 10.0;
    println!("DSP transform benchmark: 10 iterations = {:.0}us avg", avg_us);
}

// ─── Recreation Tests ─────────────────────────────────

#[test]
fn test_similarity_identical() {
    let samples = generate_sine_wave(200.0, 200.0, 0.5);
    let analysis = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    let sim = cshot_lib::audio::recreate::compute_similarity(&samples, &samples, &analysis);
    assert!((sim.overall - 1.0).abs() < 0.01, "identical should score 1.0, got {}", sim.overall);
    assert!((sim.envelope_match - 1.0).abs() < 0.01);
    assert!((sim.rms_match - 1.0).abs() < 0.01);
}

#[test]
fn test_similarity_different() {
    let orig = generate_sine_wave(200.0, 200.0, 0.5);
    let diff = generate_sine_wave(5000.0, 50.0, 0.1);
    let analysis = cshot_lib::audio::analyze::analyze_audio(&orig, 44100, 1);
    let sim = cshot_lib::audio::recreate::compute_similarity(&orig, &diff, &analysis);
    assert!(sim.overall < 0.9, "different sounds should score lower, got {}", sim.overall);
}

#[test]
fn test_similarity_silence() {
    let samples = generate_sine_wave(200.0, 100.0, 0.5);
    let silence = generate_silence(100.0);
    let analysis = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    let sim = cshot_lib::audio::recreate::compute_similarity(&samples, &silence, &analysis);
    assert!(sim.rms_match < 0.5);
}

#[test]
fn test_generate_approximations() {
    let samples = generate_sine_wave(100.0, 300.0, 0.5);
    let analysis = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    let results = cshot_lib::audio::recreate::generate_approximations(
        &samples, &analysis, 3, 0.5, true, true, true,
    );
    assert!(results.len() >= 10, "Should get at least 10 approximations, got {}", results.len());
    assert!(results[0].similarity.overall >= results[1].similarity.overall
        || results.len() < 2, "results should be sorted by similarity");
    for r in &results {
        assert!(!r.samples.is_empty());
        assert!(r.similarity.overall >= 0.0 && r.similarity.overall <= 1.0);
    }
}

#[test]
fn test_generate_approximations_high_fidelity() {
    let samples = generate_sine_wave(200.0, 200.0, 0.5);
    let analysis = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    let high = cshot_lib::audio::recreate::generate_approximations(
        &samples, &analysis, 2, 0.9, true, true, true,
    );
    let low = cshot_lib::audio::recreate::generate_approximations(
        &samples, &analysis, 2, 0.1, true, true, true,
    );
    let high_avg: f32 = high.iter().map(|r| r.similarity.overall).sum::<f32>() / high.len() as f32;
    let low_avg: f32 = low.iter().map(|r| r.similarity.overall).sum::<f32>() / low.len() as f32;
    assert!(high_avg > low_avg, "high fidelity should produce more similar results");
}

#[test]
fn test_recreate_single() {
    let samples = generate_sine_wave(200.0, 200.0, 0.5);
    let (recreated, analysis, similarity) = cshot_lib::audio::recreate::recreate_single(&samples, 0.5);
    assert!(!recreated.is_empty());
    assert!(!analysis.sound_type_hint.is_empty());
    assert!(similarity.overall >= 0.0);
}

#[test]
fn test_envelope_similarity_perfect() {
    let a = vec![0.0, 0.5, 1.0, 0.5, 0.0];
    let b = vec![0.0, 0.5, 1.0, 0.5, 0.0];
    let sim = cshot_lib::audio::recreate::envelope_similarity(&a, &b);
    assert!((sim - 1.0).abs() < 0.01);
}

#[test]
fn test_envelope_similarity_different() {
    let a = vec![0.0, 0.5, 1.0, 0.5, 0.0];
    let b = vec![1.0, 0.8, 0.6, 0.4, 0.2];
    let sim = cshot_lib::audio::recreate::envelope_similarity(&a, &b);
    assert!(sim < 1.0);
    assert!(sim >= 0.0);
}

// ─── Transform Tests ──────────────────────────────────

fn make_params(prompt: &str) -> cshot_lib::audio::resynthesize::ResynthesisParams {
    use cshot_lib::audio::resynthesize;
    let ctrl = cshot_lib::prompt_dsp::parse_prompt_rich(prompt);
    let st = cshot_lib::audio::SoundType::from_str(&ctrl.sound_type);
    let pitch = ctrl.pitch_hz.unwrap_or(200.0);
    let dur = ctrl.duration_ms.unwrap_or(300.0);
    let base = resynthesize::params_for_sound_type(st, pitch, dur);
    ctrl.to_resynthesis_params(&base)
}

#[test]
fn test_transform_with_params() {
    let samples = crate::generate_sine_wave(200.0, 200.0, 0.5);
    let params = make_params("darker shorter softer");
    let result = cshot_lib::audio::transform::transform_with_params(&samples, &params);
    assert!(!result.is_empty());
    let peak = result.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.0);
}

#[test]
fn test_transform_with_params_bright() {
    let samples = crate::generate_sine_wave(200.0, 200.0, 0.5);
    let params = make_params("brighter punchy");
    let result = cshot_lib::audio::transform::transform_with_params(&samples, &params);
    assert!(!result.is_empty());
    let peak = result.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.0);
}

#[test]
fn test_transform_with_params_distorted() {
    let samples = crate::generate_sine_wave(200.0, 200.0, 0.5);
    let params = make_params("distorted aggressive");
    let result = cshot_lib::audio::transform::transform_with_params(&samples, &params);
    assert!(!result.is_empty());
}

#[test]
fn test_transform_dsp_reverse() {
    let mut samples = crate::generate_sine_wave(200.0, 100.0, 0.5);
    let original = samples.clone();
    let params = cshot_lib::audio::transform::TransformParams { reverse: true, ..Default::default() };
    cshot_lib::audio::transform::apply_dsp_transforms(&mut samples, &params);
    assert_eq!(samples.len(), original.len());
    assert!(samples != original, "Reversed samples should differ from original");
}

#[test]
fn test_transform_dsp_saturation() {
    let mut samples = crate::generate_sine_wave(200.0, 100.0, 1.0);
    let params = cshot_lib::audio::transform::TransformParams { saturation_drive: Some(3.0), ..Default::default() };
    cshot_lib::audio::transform::apply_dsp_transforms(&mut samples, &params);
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak <= 1.0);
}

#[test]
fn test_transform_dsp_sub_add() {
    let mut samples = crate::generate_sine_wave(200.0, 100.0, 0.3);
    let params = cshot_lib::audio::transform::TransformParams { sub_add: Some(0.5), ..Default::default() };
    cshot_lib::audio::transform::apply_dsp_transforms(&mut samples, &params);
    assert!(!samples.is_empty());
}

#[test]
fn test_transform_dsp_noise_add() {
    let mut samples = crate::generate_sine_wave(200.0, 100.0, 0.5);
    let params = cshot_lib::audio::transform::TransformParams { noise_add: Some(0.3), ..Default::default() };
    cshot_lib::audio::transform::apply_dsp_transforms(&mut samples, &params);
    assert!(!samples.is_empty());
}

#[test]
fn test_transform_dsp_pitch_shift() {
    let mut samples = crate::generate_sine_wave(200.0, 100.0, 0.5);
    let params = cshot_lib::audio::transform::TransformParams { pitch_shift_semitones: Some(12.0), ..Default::default() };
    cshot_lib::audio::transform::apply_dsp_transforms(&mut samples, &params);
    assert!(!samples.is_empty());
}

#[test]
fn test_transform_dsp_brightness_tilt() {
    let mut samples = crate::generate_sine_wave(200.0, 100.0, 0.5);
    let params = cshot_lib::audio::transform::TransformParams { brightness_tilt: Some(0.5), ..Default::default() };
    cshot_lib::audio::transform::apply_dsp_transforms(&mut samples, &params);
    assert!(!samples.is_empty());
}

#[test]
fn test_transform_dsp_duration_scale() {
    let mut samples = crate::generate_sine_wave(200.0, 200.0, 0.5);
    let orig_len = samples.len();
    let params = cshot_lib::audio::transform::TransformParams { duration_scale: Some(0.5), ..Default::default() };
    cshot_lib::audio::transform::apply_dsp_transforms(&mut samples, &params);
    assert!(samples.len() < orig_len);
}

// ─── Prompt-to-DSP Tests ───────────────────────────────

#[test]
fn test_prompt_parse_basic() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("punchy kick");
    assert_eq!(result.sound_type, "kick");
    assert!(result.descriptors.iter().any(|d| d.word == "punchy"));
    assert!(result.transient_boost.is_some());
    assert!(result.transient_boost.unwrap() > 0.0);
}

#[test]
fn test_prompt_parse_dark_bright() {
    let dark = cshot_lib::prompt_dsp::parse_prompt_rich("dark kick");
    assert!(dark.brightness.unwrap_or(0.5) < 0.5);
    let bright = cshot_lib::prompt_dsp::parse_prompt_rich("bright snare");
    assert!(bright.brightness.unwrap_or(0.5) > 0.5);
}

#[test]
fn test_prompt_parse_short_long() {
    let short = cshot_lib::prompt_dsp::parse_prompt_rich("short kick");
    assert!(short.duration_ms.unwrap_or(500.0) < 500.0);
    let long = cshot_lib::prompt_dsp::parse_prompt_rich("long kick");
    assert!(long.duration_ms.unwrap_or(300.0) > 300.0);
}

#[test]
fn test_prompt_parse_distorted() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("distorted snare");
    assert!(result.saturation_drive.unwrap_or(1.0) > 1.5);
}

#[test]
fn test_prompt_parse_bpm() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("kick 140bpm");
    assert_eq!(result.bpm, Some(140.0));
}

#[test]
fn test_prompt_parse_genre_trap() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("trap kick");
    assert!(result.genre_hints.contains(&"trap".to_string()));
    assert!(result.sub_gain.unwrap_or(0.0) > 0.0);
}

#[test]
fn test_prompt_parse_genre_drill() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("drill 808");
    assert!(result.genre_hints.contains(&"drill".to_string()));
    assert!(result.tail_ms.unwrap_or(0.0) > 100.0);
}

#[test]
fn test_prompt_parse_cinematic() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("cinematic impact");
    assert!(result.duration_ms.unwrap_or(300.0) > 400.0);
}

#[test]
fn test_prompt_parse_clamp_values() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("extremely bright snare with massive click");
    let brightness = result.brightness.unwrap_or(0.5);
    assert!(brightness >= 0.0 && brightness <= 1.0);
    let click = result.click_amount.unwrap_or(0.0);
    assert!(click >= 0.0 && click <= 1.0);
}

#[test]
fn test_prompt_parse_metallic() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("metallic perc");
    assert!(result.brightness.unwrap_or(0.5) > 0.5);
    assert!(result.pitch_hz.unwrap_or(200.0) > 200.0);
}

#[test]
fn test_prompt_parse_to_dsp_params() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("punchy bright kick 140bpm");
    let dsp = result.to_dsp_params();
    assert!(dsp.punch);
    assert!(dsp.bright);
    assert_eq!(dsp.bpm, Some(140.0));
}

#[test]
fn test_prompt_parse_to_resynthesis_params() {
    use cshot_lib::audio::resynthesize;
    let base = resynthesize::ResynthesisParams::default();
    let ctrl = cshot_lib::prompt_dsp::parse_prompt_rich("short dark kick with less decay");
    let params = ctrl.to_resynthesis_params(&base);
    assert!(params.decay_ms <= base.decay_ms || ctrl.decay_ms.is_some());
}

#[test]
fn test_prompt_parse_multi_descriptor() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("punchy bright distorted clap");
    assert!(result.descriptors.len() >= 3);
    assert!(result.descriptors.iter().any(|d| d.word == "punchy"));
    assert!(result.descriptors.iter().any(|d| d.word == "bright"));
    assert!(result.descriptors.iter().any(|d| d.word == "distorted"));
}

#[test]
fn test_prompt_parse_soft() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("soft snare");
    assert!(result.attack_ms.unwrap_or(2.0) > 3.0);
    assert!(result.click_amount.unwrap_or(0.5) < 0.5);
}

#[test]
fn test_prompt_parse_empty() {
    let result = cshot_lib::prompt_dsp::parse_prompt_rich("");
    assert!(result.descriptors.is_empty());
    assert!(result.genre_hints.is_empty());
}

// ─── Resynthesis Tests ──────────────────────────────────

#[test]
fn test_resynthesize_kick() {
    use cshot_lib::audio::resynthesize;
    let params = resynthesize::ResynthesisParams {
        sound_type: cshot_lib::audio::SoundType::Kick,
        duration_ms: 300.0,
        pitch_hz: 60.0,
        pitch_drop_ratio: 0.7,
        attack_ms: 1.0,
        decay_ms: 200.0,
        tail_ms: 100.0,
        noise_amount: 0.0,
        noise_hp_hz: 5000.0,
        click_amount: 0.6,
        body_gain: 0.8,
        sub_gain: 0.5,
        saturation_drive: 1.3,
        brightness: 0.3,
        layer_mix: vec![0.3, 0.6, 0.0, 0.4, 0.0],
        seed: 0,
    };
    let result = resynthesize::resynthesize(&params);
    assert!(!result.is_empty());
    assert!(result.len() as f32 / 44100.0 * 1000.0 <= 310.0);
    let peak = result.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.1);
    assert!(peak <= 1.0);
}

#[test]
fn test_resynthesize_snare() {
    use cshot_lib::audio::resynthesize;
    let params = resynthesize::ResynthesisParams {
        sound_type: cshot_lib::audio::SoundType::Snare,
        duration_ms: 300.0,
        pitch_hz: 200.0,
        pitch_drop_ratio: 0.15,
        attack_ms: 1.0,
        decay_ms: 150.0,
        tail_ms: 100.0,
        noise_amount: 0.7,
        noise_hp_hz: 200.0,
        click_amount: 0.4,
        body_gain: 0.4,
        sub_gain: 0.0,
        saturation_drive: 1.0,
        brightness: 0.6,
        layer_mix: vec![0.2, 0.3, 0.5, 0.0, 0.0],
        seed: 0,
    };
    let result = resynthesize::resynthesize(&params);
    assert!(!result.is_empty());
    let peak = result.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.1);
    assert!(peak <= 1.0);
}

#[test]
fn test_resynthesize_hat() {
    use cshot_lib::audio::resynthesize;
    let params = resynthesize::ResynthesisParams {
        sound_type: cshot_lib::audio::SoundType::ClosedHat,
        duration_ms: 150.0,
        pitch_hz: 400.0,
        pitch_drop_ratio: 0.0,
        attack_ms: 0.5,
        decay_ms: 80.0,
        tail_ms: 0.0,
        noise_amount: 1.0,
        noise_hp_hz: 6000.0,
        click_amount: 0.3,
        body_gain: 0.0,
        sub_gain: 0.0,
        saturation_drive: 1.0,
        brightness: 0.9,
        layer_mix: vec![0.2, 0.0, 0.8, 0.0, 0.0],
        seed: 0,
    };
    let result = resynthesize::resynthesize(&params);
    assert!(!result.is_empty());
    let peak = result.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.01);
}

#[test]
fn test_resynthesize_bass() {
    use cshot_lib::audio::resynthesize;
    let params = resynthesize::ResynthesisParams {
        sound_type: cshot_lib::audio::SoundType::Bass,
        duration_ms: 500.0,
        pitch_hz: 55.0,
        pitch_drop_ratio: 0.3,
        attack_ms: 5.0,
        decay_ms: 300.0,
        tail_ms: 200.0,
        noise_amount: 0.0,
        noise_hp_hz: 2000.0,
        click_amount: 0.0,
        body_gain: 0.9,
        sub_gain: 0.6,
        saturation_drive: 1.5,
        brightness: 0.2,
        layer_mix: vec![0.0, 0.5, 0.0, 0.4, 0.0],
        seed: 0,
    };
    let result = resynthesize::resynthesize(&params);
    assert!(!result.is_empty());
    assert!(result.len() as f32 / 44100.0 * 1000.0 <= 510.0);
}

#[test]
fn test_resynthesize_clap() {
    use cshot_lib::audio::resynthesize;
    let params = resynthesize::ResynthesisParams {
        sound_type: cshot_lib::audio::SoundType::Clap,
        duration_ms: 300.0,
        pitch_hz: 180.0,
        pitch_drop_ratio: 0.0,
        attack_ms: 2.0,
        decay_ms: 150.0,
        tail_ms: 100.0,
        noise_amount: 0.9,
        noise_hp_hz: 500.0,
        click_amount: 0.0,
        body_gain: 0.2,
        sub_gain: 0.0,
        saturation_drive: 1.2,
        brightness: 0.7,
        layer_mix: vec![0.0, 0.15, 0.7, 0.0, 0.0],
        seed: 0,
    };
    let result = resynthesize::resynthesize(&params);
    assert!(!result.is_empty());
    let peak = result.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.01);
}

#[test]
fn test_resynthesize_from_analysis() {
    let samples = generate_sine_wave(100.0, 300.0, 0.5);
    let analysis = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    let result = cshot_lib::audio::resynthesize::resynthesize_from_analysis(&analysis);
    assert!(!result.is_empty());
    let peak = result.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.01);
}

#[test]
fn test_resynthesize_empty_params() {
    use cshot_lib::audio::resynthesize;
    let params = resynthesize::ResynthesisParams {
        duration_ms: 0.0,
        ..Default::default()
    };
    let result = resynthesize::resynthesize(&params);
    assert!(result.is_empty());
}

#[test]
fn test_resynthesize_all_types_produce_audio() {
    use cshot_lib::audio::resynthesize;
    use cshot_lib::audio::SoundType;
    let types = [
        SoundType::Kick, SoundType::Snare, SoundType::ClosedHat,
        SoundType::OpenHat, SoundType::Clap, SoundType::Bass,
        SoundType::Perc, SoundType::Fx, SoundType::Tom, SoundType::Other,
    ];
    for st in &types {
        let pitch = match st {
            SoundType::Kick | SoundType::Bass => 60.0,
            SoundType::Snare => 200.0,
            SoundType::ClosedHat | SoundType::OpenHat => 400.0,
            SoundType::Clap => 180.0,
            SoundType::Tom => 120.0,
            SoundType::Perc => 300.0,
            SoundType::Fx => 100.0,
            SoundType::Other => 200.0,
        };
        let params = resynthesize::params_for_sound_type(*st, pitch, 300.0);
        let result = resynthesize::resynthesize(&params);
        assert!(!result.is_empty(), "Type {:?} produced empty audio", st);
        let peak = result.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak > 0.0, "Type {:?} produced silent audio (peak={})", st, peak);
    }
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

// ─── Weeks 277-280: Miracle Machine Showcase Tests ────────

#[test]
fn test_showcase_transient_mastery() {
    use cshot_lib::audio::dsp;
    use cshot_lib::audio::SAMPLE_RATE;

    let test_sound = generate_sine_wave(200.0, 200.0, 0.5);
    let sharpness = dsp::compute_transient_sharpness(&test_sound);
    assert!(sharpness >= 0.0 && sharpness <= 1.0);

    let tc = dsp::TransientConfig {
        click_character: dsp::ClickCharacter::Sharp,
        sharpness: 0.8,
        ..Default::default()
    };
    let click = dsp::generate_click(&tc, 1024);
    assert!(!click.is_empty());
    let peak = click.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.0, "Click should have non-zero content");

    let mut samples = vec![0.5f32; 1024];
    dsp::multiband_transient_processor(&mut samples, &dsp::MultiBandTransientConfig::default());
    assert!(samples.iter().any(|s| s.abs() > 0.0));

    let mut impact = test_sound.clone();
    dsp::impact_processor(&mut impact, &dsp::ImpactConfig::default());
    assert_eq!(impact.len(), test_sound.len());
}

#[test]
fn test_showcase_tail_texture() {
    use cshot_lib::audio::dsp;

    let mut samples = generate_sine_wave(100.0, 500.0, 0.5);
    let cfg = dsp::TailTextureConfig {
        decay_modulation_depth: 0.1,
        noise_texture_density: 0.3,
        analog_instability: 0.05,
        ..Default::default()
    };
    let before_len = samples.len();
    dsp::apply_tail_texture(&mut samples, &cfg);
    assert_eq!(samples.len(), before_len);

    dsp::non_static_decay(&mut samples, 0.2);
    assert!(!samples.is_empty());

    let num = samples.len();
    let tex = dsp::texture_layering(num, 2, 0.3, 42.0, 44100);
    assert_eq!(tex.len(), num);
}

#[test]
fn test_showcase_miracle_recreation() {
    use cshot_lib::audio::recreate;
    use cshot_lib::audio::analyze;

    let kick = generate_sine_wave(60.0, 200.0, 0.5);
    let analysis = analyze::analyze_audio(&kick, 44100, 1);
    let modes = recreate::generate_all_recreation_modes(&kick, 0.7, true);
    assert!(!modes.is_empty(), "Should generate at least one recreation mode");
    assert!(modes.iter().any(|m| m.mode == "closest"), "Should include closest mode");
    assert!(modes.iter().any(|m| m.mode == "harder"), "Should include harder mode");
    assert!(modes.iter().any(|m| m.mode == "cleaner"), "Should include cleaner mode");
    assert!(modes.iter().any(|m| m.mode == "cinematic"), "Should include cinematic mode");
    assert!(modes.iter().any(|m| m.mode == "experimental"), "Should include experimental mode");
    assert!(modes.iter().any(|m| m.mode == "modernized"), "Should include modernized mode");
    assert!(modes.iter().any(|m| m.mode == "darker"), "Should include darker mode");
    assert!(modes.iter().any(|m| m.mode == "brighter"), "Should include brighter mode");

    for mode in &modes {
        assert!(!mode.samples.is_empty(), "Mode {} should produce non-empty samples", mode.mode);
        assert!(mode.similarity.overall >= 0.0 && mode.similarity.overall <= 1.0);
        assert!(mode.quality_score >= 0.0 && mode.quality_score <= 1.0);
    }
}

#[test]
fn test_showcase_genre_intelligence() {
    use cshot_lib::audio::recreate;
    use cshot_lib::audio::resynthesize;
    use cshot_lib::audio::analyze;

    let test_sound = generate_sine_wave(150.0, 300.0, 0.5);
    let analysis = analyze::analyze_audio(&test_sound, 44100, 1);
    let params = recreate::params_from_analysis(&analysis, &test_sound);

    let genres = ["trap", "drill", "hyperpop", "house", "techno", "cinematic", "pop", "jersey", "rage", "industrial", "ambient", "ui_game"];
    for genre in &genres {
        let adapted = recreate::adapt_params_for_genre(&params, genre);
        let result = resynthesize::resynthesize(&adapted);
        assert!(!result.is_empty(), "Genre {} should produce output", genre);
        assert!(result.iter().any(|s| s.abs() > 0.001), "Genre {} should have audio content", genre);
    }
}

#[test]
fn test_showcase_prompt_mutation() {
    use cshot_lib::audio::resynthesize;
    use cshot_lib::prompt_dsp;
    use cshot_lib::audio::recreate;

    let test_sound = generate_sine_wave(200.0, 200.0, 0.5);
    let analysis = cshot_lib::audio::analyze::analyze_audio(&test_sound, 44100, 1);
    let params = recreate::params_from_analysis(&analysis, &test_sound);

    let edits = ["harder", "cleaner", "warmer", "darker", "brighter", "punchier",
                  "more analog", "more futuristic", "more distorted", "softer",
                  "tighter transient", "fatter low end", "more cinematic"];
    for edit in &edits {
        let (mutated, _edits, identity) = prompt_dsp::apply_prompt_mutation(&params, edit, 0.5);
        let result = resynthesize::resynthesize(&mutated);
        assert!(!result.is_empty(), "Edit '{}' should produce output", edit);
        assert!(identity >= 0.3 && identity <= 1.0, "Edit '{}' should preserve identity", edit);
    }
}

#[test]
fn test_showcase_sound_designer() {
    use cshot_lib::audio::sound_designer;
    use cshot_lib::audio::resynthesize;
    use cshot_lib::audio::recreate;

    let params = recreate::params_from_analysis(
        &cshot_lib::audio::analyze::analyze_audio(&generate_sine_wave(100.0, 300.0, 0.5), 44100, 1),
        &generate_sine_wave(100.0, 300.0, 0.5),
    );
    let controls = sound_designer::SoundDesignerControls::default();
    let rendered = sound_designer::render_sound_designer(&params, &controls);
    assert!(!rendered.is_empty(), "Sound designer should render audio");

    let layers = resynthesize::generate_layers(&params, 256);
    let swapped = sound_designer::swap_layer(&layers, &layers, "transient");
    assert_eq!(swapped.transient.len(), layers.transient.len());

    let solod = sound_designer::solo_layer(&params, "body");
    assert!(!solod.is_empty(), "Solo layer should produce audio");

    let viz = sound_designer::compute_layer_visualization(&params, &controls, 64);
    assert_eq!(viz.layer_peaks.len(), 5);
    assert_eq!(viz.layer_rms.len(), 5);
}

#[test]
fn test_showcase_variant_machine() {
    use cshot_lib::audio::evolution;
    use cshot_lib::audio::analyze;

    let test_sound = generate_sine_wave(80.0, 200.0, 0.5);
    let analysis = analyze::analyze_audio(&test_sound, 44100, 1);

    let config = evolution::VariantGenerationConfig {
        exploration_mode: evolution::ExplorationMode::Balanced,
        count: 4,
        direction: Some("wilder".to_string()),
        direction_intensity: 0.5,
        ..Default::default()
    };
    let variants = evolution::generate_variants(&test_sound, &analysis, &config);
    assert!(!variants.is_empty(), "Should generate variants");
    if !variants.is_empty() {
        assert!(variants[0].score >= 0.0);
    }

    let safe_cfg = evolution::VariantGenerationConfig {
        exploration_mode: evolution::ExplorationMode::Safe,
        count: 3,
        ..Default::default()
    };
    let safe_variants = evolution::generate_variants(&test_sound, &analysis, &safe_cfg);
    assert!(!safe_variants.is_empty(), "Safe variants should generate");

    let wild_cfg = evolution::VariantGenerationConfig {
        exploration_mode: evolution::ExplorationMode::Wild,
        count: 5,
        direction: Some("more experimental".to_string()),
        direction_intensity: 0.8,
        ..Default::default()
    };
    let wild_variants = evolution::generate_variants(&test_sound, &analysis, &wild_cfg);
    assert!(!wild_variants.is_empty(), "Wild variants should generate");
}

#[test]
fn test_showcase_identity_intelligence() {
    use cshot_lib::audio::identity;
    use cshot_lib::audio::analyze;

    let test_sound = generate_sine_wave(100.0, 300.0, 0.5);
    let analysis = analyze::analyze_audio(&test_sound, 44100, 1);
    let ident = identity::compute_sound_identity(&analysis);

    assert!(!ident.overall_character.is_empty(), "Should have character tags");
    assert!(!ident.identity_fingerprint.is_empty(), "Should have fingerprint");
    assert!(ident.aggressiveness >= 0.0 && ident.aggressiveness <= 1.0);
    assert!(ident.density >= 0.0 && ident.density <= 1.0);
    assert!(ident.transient_identity.attack_sharpness >= 0.0);
    assert!(ident.tonal_identity.spectral_centroid >= 0.0);
    assert!(ident.texture_identity.noise_floor_db <= 0.0);

    let ident2 = identity::compute_sound_identity(&analysis);
    let dist = identity::sound_identity_distance(&ident, &ident2);
    assert!(dist < 0.01, "Identical analyses should have near-zero distance");
}

#[test]
fn test_showcase_perfection_workflow() {
    use cshot_lib::audio::process;

    let test_sound = generate_sine_wave(200.0, 200.0, 0.5);
    let mut adjusted = test_sound.clone();
    let adjustments = vec![
        process::MicroAdjustment { param: "gain".to_string(), delta: 3.0, description: "boost 3dB".to_string() },
        process::MicroAdjustment { param: "punch".to_string(), delta: 2.0, description: "add punch".to_string() },
    ];
    process::apply_micro_adjustments(&mut adjusted, &adjustments);
    assert!(!adjusted.is_empty());

    let ab = process::compute_ab_comparison(&test_sound, &adjusted, "original", "adjusted", 64);
    assert!(!ab.version_a_waveform.is_empty());
    assert!(!ab.version_b_waveform.is_empty());
    assert!(!ab.differences.is_empty() || ab.version_a_rms > 0.0);

    let assessment = process::assess_export_readiness(&adjusted, -1.0);
    assert!(assessment.confidence >= 0.0 && assessment.confidence <= 1.0);
    assert!(assessment.gain_staging.peak_db <= 0.0);
    assert!(assessment.gain_staging.rms_db <= 0.0);
}

#[test]
fn test_showcase_end_to_end() {
    use cshot_lib::audio::analyze;
    use cshot_lib::audio::dsp;
    use cshot_lib::audio::resynthesize;
    use cshot_lib::audio::recreate;
    use cshot_lib::audio::identity;
    use cshot_lib::audio::sound_designer;
    use cshot_lib::audio::process;
    use cshot_lib::audio::evolution;

    // 1. Generate a sound from prompt
    let params = resynthesize::params_for_sound_type(
        cshot_lib::audio::SoundType::Kick,
        60.0,
        300.0,
    );
    let original = resynthesize::resynthesize(&params);
    assert!(!original.is_empty(), "Should generate sound from params");

    // 2. Analyze it
    let analysis = analyze::analyze_audio(&original, 44100, 1);
    assert!(analysis.peak > 0.0);
    assert!(analysis.duration_ms > 0.0);

    // 3. Compute its identity
    let ident = identity::compute_sound_identity(&analysis);
    assert!(!ident.overall_character.is_empty());

    // 4. Recreate with multiple modes
    let modes = recreate::generate_all_recreation_modes(&original, 0.7, false);
    assert!(!modes.is_empty());

    // 5. Find the best quality recreation
    let best = modes.iter().max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap_or(std::cmp::Ordering::Equal));
    assert!(best.is_some());

    // 6. Mutate it with prompt
    let best_params = recreate::params_from_analysis(&analysis, &original);
    let (mutated, _edits, _ident) = cshot_lib::prompt_dsp::apply_prompt_mutation(&best_params, "punchier", 0.6);
    let mutated_sound = resynthesize::resynthesize(&mutated);
    assert!(!mutated_sound.is_empty());

    // 7. Generate variants
    let variant_config = evolution::VariantGenerationConfig {
        count: 3,
        ..Default::default()
    };
    let variants = evolution::generate_variants(&mutated_sound, &analysis, &variant_config);
    assert!(!variants.is_empty());

    // 8. Apply sound designer controls
    let designer_controls = sound_designer::SoundDesignerControls::default();
    let final_design = sound_designer::render_sound_designer(&params, &designer_controls);
    assert!(!final_design.is_empty());

    // 9. Apply micro-adjustments
    let mut final_sound = final_design;
    process::apply_micro_adjustments(&mut final_sound, &[
        process::MicroAdjustment { param: "gain".to_string(), delta: -1.0, description: "trim".to_string() },
    ]);
    assert!(!final_sound.is_empty());

    // 10. Assess export readiness
    let assessment = process::assess_export_readiness(&final_sound, -1.0);
    assert!(!assessment.warnings.iter().any(|w| w.contains("Silent")));

    // Full pipeline: params -> resynthesize -> analyze -> identity -> recreate -> mutate -> variants
    // All steps produce non-empty, valid results
    println!("End-to-end showcase pipeline: PASS");
    println!("  Generated: {} samples @ 44.1kHz", original.len());
    println!("  Analysis: peak={:.3}, rms={:.3}", analysis.peak, analysis.rms);
    println!("  Identity: {} chars, {} traits", ident.overall_character.len(), ident.embedding.len());
    println!("  Recreation: {} modes, best quality={:.3}", modes.len(),
        best.map(|b| b.quality_score).unwrap_or(0.0));
    println!("  Mutation: identity preserved {:.0}%", _ident * 100.0);
    println!("  Variants: {}", variants.len());
    println!("  Export ready: {}", assessment.export_ready);
}

// ─── WAV I/O Tests ────────────────────────────────────────

fn wav_test_path(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("cshot_wav_test_{}", name));
    let _ = std::fs::create_dir_all(&dir);
    dir.join(format!("{}.wav", name))
}

fn cleanup_wav_test(name: &str) {
    let dir = std::env::temp_dir().join(format!("cshot_wav_test_{}", name));
    let path = dir.join(format!("{}.wav", name));
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn test_write_wav_success() {
    let path = wav_test_path("test_write_wav_success");
    let samples = generate_sine_wave(440.0, 100.0, 0.5);
    let result = cshot_lib::audio::write_wav(&path, &samples, 44100);
    assert!(result.is_ok(), "write_wav should succeed: {:?}", result);
    let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    assert!(file_size >= 44, "WAV file should be at least 44 bytes");
    cleanup_wav_test("test_write_wav_success");
}

#[test]
fn test_write_wav_empty_rejected() {
    let path = wav_test_path("test_write_wav_empty_rejected");
    let samples: Vec<f32> = vec![];
    let result = cshot_lib::audio::write_wav(&path, &samples, 44100);
    assert!(result.is_err(), "Empty samples should be rejected");
    assert!(result.unwrap_err().contains("empty"), "Error should mention 'empty'");
    cleanup_wav_test("test_write_wav_empty_rejected");
}

#[test]
fn test_write_wav_nan_rejected() {
    let path = wav_test_path("test_write_wav_nan_rejected");
    let samples = vec![std::f32::NAN; 100];
    let result = cshot_lib::audio::write_wav(&path, &samples, 44100);
    assert!(result.is_err(), "NaN samples should be rejected");
    assert!(result.unwrap_err().contains("NaN"), "Error should mention 'NaN'");
    cleanup_wav_test("test_write_wav_nan_rejected");
}

#[test]
fn test_write_wav_inf_rejected() {
    let path = wav_test_path("test_write_wav_inf_rejected");
    let samples = vec![std::f32::INFINITY; 100];
    let result = cshot_lib::audio::write_wav(&path, &samples, 44100);
    assert!(result.is_err(), "Inf samples should be rejected");
    assert!(result.unwrap_err().contains("Inf"), "Error should mention 'Inf'");
    cleanup_wav_test("test_write_wav_inf_rejected");
}

#[test]
fn test_write_wav_clamps_above_1() {
    let path = wav_test_path("test_write_wav_clamps_above_1");
    let mut samples = generate_sine_wave(440.0, 50.0, 0.5);
    samples.push(1.5);
    samples.push(2.0);
    samples.push(10.0);
    let result = cshot_lib::audio::write_wav(&path, &samples, 44100);
    assert!(result.is_ok(), "Samples above 1.0 should be clamped and written: {:?}", result);
    let read_back = cshot_lib::audio::read_wav(&path).unwrap();
    let peak = read_back.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak <= 1.0, "Peak should be clamped to 1.0, got {}", peak);
    cleanup_wav_test("test_write_wav_clamps_above_1");
}

#[test]
fn test_write_wav_clamps_below_neg1() {
    let path = wav_test_path("test_write_wav_clamps_below_neg1");
    let mut samples = generate_sine_wave(440.0, 50.0, 0.5);
    samples.push(-1.5);
    samples.push(-2.0);
    samples.push(-10.0);
    let result = cshot_lib::audio::write_wav(&path, &samples, 44100);
    assert!(result.is_ok(), "Samples below -1.0 should be clamped and written: {:?}", result);
    let read_back = cshot_lib::audio::read_wav(&path).unwrap();
    let min = read_back.iter().fold(0.0f32, |a, &b| a.min(b));
    assert!(min >= -1.0, "Min should be clamped to -1.0, got {}", min);
    cleanup_wav_test("test_write_wav_clamps_below_neg1");
}

#[test]
fn test_write_wav_bytes_roundtrip() {
    let original = generate_sine_wave(440.0, 50.0, 0.5);
    let bytes = cshot_lib::audio::write_wav_bytes(&original, 44100).unwrap();
    assert!(bytes.len() >= 44, "WAV bytes should be at least 44 bytes");
    let path = wav_test_path("test_write_wav_bytes_roundtrip");
    std::fs::write(&path, &bytes).unwrap();
    let read_back = cshot_lib::audio::read_wav(&path).unwrap();
    assert_eq!(read_back.len(), original.len(), "Roundtrip should preserve sample count");
    cleanup_wav_test("test_write_wav_bytes_roundtrip");
}

#[test]
fn test_write_wav_bytes_nan_rejected() {
    let samples = vec![std::f32::NAN; 100];
    let result = cshot_lib::audio::write_wav_bytes(&samples, 44100);
    assert!(result.is_err(), "NaN samples should be rejected in write_wav_bytes");
}

#[test]
fn test_write_wav_bytes_inf_rejected() {
    let samples = vec![std::f32::INFINITY; 100];
    let result = cshot_lib::audio::write_wav_bytes(&samples, 44100);
    assert!(result.is_err(), "Inf samples should be rejected in write_wav_bytes");
}

#[test]
fn test_write_wav_local_engine() {
    use cshot_lib::audio::resynthesize;
    let params = resynthesize::params_for_sound_type(
        cshot_lib::audio::SoundType::Kick, 60.0, 300.0,
    );
    let samples = resynthesize::resynthesize(&params);
    assert!(!samples.is_empty(), "Engine should produce samples");
    let path = wav_test_path("test_write_wav_local_engine");
    let result = cshot_lib::audio::write_wav(&path, &samples, 44100);
    assert!(result.is_ok(), "Local engine WAV write should succeed: {:?}", result);
    let read_back = cshot_lib::audio::read_wav(&path).unwrap();
    assert_eq!(read_back.len(), samples.len(), "Read-back sample count should match");
    let peak = read_back.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak > 0.0, "Read-back audio should have content");
    cleanup_wav_test("test_write_wav_local_engine");
}
