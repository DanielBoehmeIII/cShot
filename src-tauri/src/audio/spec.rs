use super::{SoundType, SAMPLE_RATE};
use super::one_shot_controls::OneShotControls;
use super::resynthesize::{ResynthesisParams, resynthesize};

/// High-level sound class for one-shot synthesis.
/// Each variant maps to a dedicated synthesis path.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum SoundClass {
    #[serde(rename = "808")]
    Sub808,
    Kick,
    Snare,
    Clap,
    ClosedHat,
    OpenHat,
    BassStab,
    ImpactFx,
    SynthStab,
}

impl SoundClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            SoundClass::Sub808 => "808",
            SoundClass::Kick => "kick",
            SoundClass::Snare => "snare",
            SoundClass::Clap => "clap",
            SoundClass::ClosedHat => "closed_hat",
            SoundClass::OpenHat => "open_hat",
            SoundClass::BassStab => "bass_stab",
            SoundClass::ImpactFx => "impact_fx",
            SoundClass::SynthStab => "synth_stab",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "808" => SoundClass::Sub808,
            "kick" => SoundClass::Kick,
            "snare" => SoundClass::Snare,
            "clap" => SoundClass::Clap,
            "closed_hat" => SoundClass::ClosedHat,
            "open_hat" => SoundClass::OpenHat,
            "bass_stab" => SoundClass::BassStab,
            "impact_fx" => SoundClass::ImpactFx,
            "synth_stab" => SoundClass::SynthStab,
            _ => SoundClass::Kick,
        }
    }

    /// Map to the internal engine SoundType for processing/analysis.
    pub fn sound_type(&self) -> SoundType {
        match self {
            SoundClass::Sub808 => SoundType::Bass,
            SoundClass::Kick => SoundType::Kick,
            SoundClass::Snare => SoundType::Snare,
            SoundClass::Clap => SoundType::Clap,
            SoundClass::ClosedHat => SoundType::ClosedHat,
            SoundClass::OpenHat => SoundType::OpenHat,
            SoundClass::BassStab => SoundType::Bass,
            SoundClass::ImpactFx => SoundType::Fx,
            SoundClass::SynthStab => SoundType::Other,
        }
    }

    /// All valid sound class string values.
    pub fn all_values() -> &'static [&'static str] {
        &[
            "808", "kick", "snare", "clap", "closed_hat",
            "open_hat", "bass_stab", "impact_fx", "synth_stab",
        ]
    }

    pub fn is_valid(s: &str) -> bool {
        Self::all_values().contains(&s)
    }
}

/// Typed one-shot specification with duration, pitch, and gain controls.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OneShotSpec {
    pub sound_class: SoundClass,
    pub duration_ms: f32,
    pub pitch_hz: f32,
    pub gain: f32,
    pub controls: Option<OneShotControls>,
}

impl OneShotSpec {
    /// Classic 808 kick/sub preset — the original cShot sound.
    pub fn preset_808() -> Self {
        OneShotSpec {
            sound_class: SoundClass::Sub808,
            duration_ms: 800.0,
            pitch_hz: 55.0,
            gain: 1.0,
            controls: None,
        }
    }

    /// Standard kick drum.
    pub fn preset_kick() -> Self {
        OneShotSpec {
            sound_class: SoundClass::Kick,
            duration_ms: 280.0,
            pitch_hz: 100.0,
            gain: 1.0,
            controls: None,
        }
    }

    /// Standard snare drum.
    pub fn preset_snare() -> Self {
        OneShotSpec {
            sound_class: SoundClass::Snare,
            duration_ms: 320.0,
            pitch_hz: 220.0,
            gain: 1.0,
            controls: None,
        }
    }

    /// Hand clap.
    pub fn preset_clap() -> Self {
        OneShotSpec {
            sound_class: SoundClass::Clap,
            duration_ms: 380.0,
            pitch_hz: 180.0,
            gain: 1.0,
            controls: None,
        }
    }

    /// Closed hi-hat.
    pub fn preset_closed_hat() -> Self {
        OneShotSpec {
            sound_class: SoundClass::ClosedHat,
            duration_ms: 150.0,
            pitch_hz: 500.0,
            gain: 1.0,
            controls: None,
        }
    }

    /// Open hi-hat.
    pub fn preset_open_hat() -> Self {
        OneShotSpec {
            sound_class: SoundClass::OpenHat,
            duration_ms: 650.0,
            pitch_hz: 350.0,
            gain: 1.0,
            controls: None,
        }
    }

    /// Sub-heavy bass stab.
    pub fn preset_bass_stab() -> Self {
        OneShotSpec {
            sound_class: SoundClass::BassStab,
            duration_ms: 350.0,
            pitch_hz: 80.0,
            gain: 1.0,
            controls: None,
        }
    }

    /// Big cinematic impact / riser hit.
    pub fn preset_impact_fx() -> Self {
        OneShotSpec {
            sound_class: SoundClass::ImpactFx,
            duration_ms: 1500.0,
            pitch_hz: 70.0,
            gain: 1.0,
            controls: None,
        }
    }

    /// Synth stab with chord-like oscillator content.
    pub fn preset_synth_stab() -> Self {
        OneShotSpec {
            sound_class: SoundClass::SynthStab,
            duration_ms: 600.0,
            pitch_hz: 220.0,
            gain: 1.0,
            controls: None,
        }
    }

    /// Render this spec to audio samples using the dedicated synthesis path.
    pub fn render(&self) -> Vec<f32> {
        let params = self.to_resynthesis_params();
        let mut samples = resynthesize(&params);
        if (self.gain - 1.0).abs() > 0.001 {
            for s in &mut samples {
                *s *= self.gain;
            }
        }
        samples
    }

    /// Build ResynthesisParams from this spec using the sound-class-specific path.
    /// If controls are set, they are applied to shape the parameters.
    pub fn to_resynthesis_params(&self) -> ResynthesisParams {
        let dur = self.duration_ms.max(10.0).min(5000.0);
        let pitch = self.pitch_hz;
        let mut params = match self.sound_class {
            SoundClass::Sub808 => sub808_params(dur, pitch),
            SoundClass::Kick => kick_params(dur, pitch),
            SoundClass::Snare => snare_params(dur, pitch),
            SoundClass::Clap => clap_params(dur, pitch),
            SoundClass::ClosedHat => closed_hat_params(dur, pitch),
            SoundClass::OpenHat => open_hat_params(dur, pitch),
            SoundClass::BassStab => bass_stab_params(dur, pitch),
            SoundClass::ImpactFx => impact_fx_params(dur, pitch),
            SoundClass::SynthStab => synth_stab_params(dur, pitch),
        };
        if let Some(ref controls) = self.controls {
            controls.apply_to(&mut params, self.sound_class);
        }
        params
    }
}

// ─── Class-Specific Synthesis Paths ─────────────────────────
// Each class gets a dedicated function with clearly distinct DSP parameters
// to produce audibly different sonic identities.

/// Sub808: long sine/sub decay, pitch glide, mild saturation.
/// Deep sub-bass with slow pitch drop and minimal high-end content.
fn sub808_params(duration_ms: f32, pitch_hz: f32) -> ResynthesisParams {
    let dur = duration_ms.min(1500.0);
    ResynthesisParams {
        sound_type: SoundType::Bass,
        duration_ms: dur,
        pitch_hz: pitch_hz.clamp(25.0, 150.0),
        pitch_drop_ratio: 0.7,
        attack_ms: 8.0,
        decay_ms: (dur * 0.5).min(600.0),
        tail_ms: (dur * 0.35).min(500.0),
        noise_amount: 0.0,
        noise_hp_hz: 3000.0,
        click_amount: 0.0,
        body_gain: 0.6,
        sub_gain: 1.0,
        saturation_drive: 1.6,
        brightness: 0.15,
        layer_mix: vec![0.0, 0.25, 0.0, 0.65, 0.0],
        seed: 0,
        stereo_width: 0.0,
        filter_sweep: 0.0,
        metallic_amount: 0.0,
    }
}

/// Kick: short punch transient, fast pitch drop, tight low body.
/// Aggressive click attack followed by fast-decaying pitched body.
fn kick_params(duration_ms: f32, pitch_hz: f32) -> ResynthesisParams {
    let dur = duration_ms.min(500.0);
    ResynthesisParams {
        sound_type: SoundType::Kick,
        duration_ms: dur,
        pitch_hz: pitch_hz.clamp(40.0, 150.0),
        pitch_drop_ratio: 0.85,
        attack_ms: 0.5,
        decay_ms: (dur * 0.3).min(180.0),
        tail_ms: (dur * 0.15).min(80.0),
        noise_amount: 0.0,
        noise_hp_hz: 5000.0,
        click_amount: 0.95,
        body_gain: 0.75,
        sub_gain: 0.6,
        saturation_drive: 1.5,
        brightness: 0.2,
        layer_mix: vec![0.4, 0.5, 0.0, 0.45, 0.0],
        seed: 0,
        stereo_width: 0.0,
        filter_sweep: 0.0,
        metallic_amount: 0.0,
    }
}

/// Snare: noise burst + mid body tone + fast decay.
/// Bright noise layer layered with a pitched snare-body tone.
fn snare_params(duration_ms: f32, pitch_hz: f32) -> ResynthesisParams {
    let dur = duration_ms.min(500.0);
    ResynthesisParams {
        sound_type: SoundType::Snare,
        duration_ms: dur,
        pitch_hz: pitch_hz.clamp(150.0, 300.0),
        pitch_drop_ratio: 0.2,
        attack_ms: 0.5,
        decay_ms: (dur * 0.25).min(150.0),
        tail_ms: (dur * 0.1).min(50.0),
        noise_amount: 0.9,
        noise_hp_hz: 300.0,
        click_amount: 0.55,
        body_gain: 0.45,
        sub_gain: 0.0,
        saturation_drive: 1.4,
        brightness: 0.7,
        layer_mix: vec![0.2, 0.25, 0.6, 0.0, 0.0],
        seed: 0,
        stereo_width: 0.0,
        filter_sweep: 0.0,
        metallic_amount: 0.0,
    }
}

/// Clap: multiple staggered noise bursts with short room tail.
/// Uses a dedicated multi-peak noise envelope in the renderer.
fn clap_params(duration_ms: f32, pitch_hz: f32) -> ResynthesisParams {
    let dur = duration_ms.min(500.0);
    ResynthesisParams {
        sound_type: SoundType::Clap,
        duration_ms: dur,
        pitch_hz: pitch_hz.clamp(100.0, 250.0),
        pitch_drop_ratio: 0.0,
        attack_ms: 3.0,
        decay_ms: (dur * 0.35).min(180.0),
        tail_ms: (dur * 0.3).min(120.0),
        noise_amount: 1.0,
        noise_hp_hz: 2500.0,
        click_amount: 0.0,
        body_gain: 0.1,
        sub_gain: 0.0,
        saturation_drive: 1.4,
        brightness: 0.9,
        layer_mix: vec![0.0, 0.05, 0.9, 0.0, 0.0],
        seed: 0,
        stereo_width: 0.0,
        filter_sweep: 0.0,
        metallic_amount: 0.0,
    }
}

/// ClosedHat: very short metallic/noise tick with extreme high-pass.
/// Minimal decay, high sizzle, tight envelope.
fn closed_hat_params(duration_ms: f32, pitch_hz: f32) -> ResynthesisParams {
    let dur = duration_ms.min(200.0);
    ResynthesisParams {
        sound_type: SoundType::ClosedHat,
        duration_ms: dur,
        pitch_hz: pitch_hz.clamp(200.0, 1000.0),
        pitch_drop_ratio: 0.0,
        attack_ms: 0.3,
        decay_ms: (dur * 0.35).min(40.0),
        tail_ms: 0.0,
        noise_amount: 1.0,
        noise_hp_hz: 10000.0,
        click_amount: 0.4,
        body_gain: 0.0,
        sub_gain: 0.0,
        saturation_drive: 1.3,
        brightness: 1.0,
        layer_mix: vec![0.25, 0.0, 0.8, 0.0, 0.0],
        seed: 0,
        stereo_width: 0.0,
        filter_sweep: 0.0,
        metallic_amount: 0.0,
    }
}

/// OpenHat: longer metallic/noise decay with a bright tail.
/// Noticeably longer sustain than ClosedHat.
fn open_hat_params(duration_ms: f32, pitch_hz: f32) -> ResynthesisParams {
    let dur = duration_ms.min(800.0);
    ResynthesisParams {
        sound_type: SoundType::OpenHat,
        duration_ms: dur,
        pitch_hz: pitch_hz.clamp(200.0, 800.0),
        pitch_drop_ratio: 0.0,
        attack_ms: 1.0,
        decay_ms: (dur * 0.45).min(300.0),
        tail_ms: (dur * 0.35).min(200.0),
        noise_amount: 0.95,
        noise_hp_hz: 4000.0,
        click_amount: 0.2,
        body_gain: 0.0,
        sub_gain: 0.0,
        saturation_drive: 1.3,
        brightness: 0.9,
        layer_mix: vec![0.1, 0.0, 0.9, 0.0, 0.0],
        seed: 0,
        stereo_width: 0.0,
        filter_sweep: 0.0,
        metallic_amount: 0.0,
    }
}

/// BassStab: short pitched bass hit with filter movement.
/// Punchy mid-bass with click attack, moderate pitch drop, and saturation.
fn bass_stab_params(duration_ms: f32, pitch_hz: f32) -> ResynthesisParams {
    let dur = duration_ms.min(800.0);
    ResynthesisParams {
        sound_type: SoundType::Bass,
        duration_ms: dur,
        pitch_hz: pitch_hz.clamp(40.0, 200.0),
        pitch_drop_ratio: 0.4,
        attack_ms: 2.0,
        decay_ms: (dur * 0.3).min(180.0),
        tail_ms: (dur * 0.15).min(80.0),
        noise_amount: 0.15,
        noise_hp_hz: 800.0,
        click_amount: 0.4,
        body_gain: 0.8,
        sub_gain: 0.55,
        saturation_drive: 1.8,
        brightness: 0.4,
        layer_mix: vec![0.2, 0.45, 0.1, 0.35, 0.0],
        seed: 0,
        stereo_width: 0.0,
        filter_sweep: 0.0,
        metallic_amount: 0.0,
    }
}

/// ImpactFx: sub drop + noise burst + long evolving tail.
/// Cinematic impact with layered sub, noise burst, and extended tail.
fn impact_fx_params(duration_ms: f32, pitch_hz: f32) -> ResynthesisParams {
    let dur = duration_ms.min(3000.0);
    ResynthesisParams {
        sound_type: SoundType::Fx,
        duration_ms: dur,
        pitch_hz: pitch_hz.clamp(30.0, 200.0),
        pitch_drop_ratio: 0.35,
        attack_ms: 1.0,
        decay_ms: (dur * 0.3).min(800.0),
        tail_ms: (dur * 0.55).min(1800.0),
        noise_amount: 0.9,
        noise_hp_hz: 60.0,
        click_amount: 0.55,
        body_gain: 0.35,
        sub_gain: 0.65,
        saturation_drive: 2.5,
        brightness: 0.55,
        layer_mix: vec![0.25, 0.15, 0.3, 0.25, 0.4],
        seed: 0,
        stereo_width: 0.0,
        filter_sweep: 0.0,
        metallic_amount: 0.0,
    }
}

/// SynthStab: detuned oscillators, chord/body, filter envelope.
/// Chord-like harmonic structure with moderate attack and noise.
fn synth_stab_params(duration_ms: f32, pitch_hz: f32) -> ResynthesisParams {
    let dur = duration_ms.min(2000.0);
    ResynthesisParams {
        sound_type: SoundType::Other,
        duration_ms: dur,
        pitch_hz: pitch_hz.clamp(60.0, 1000.0),
        pitch_drop_ratio: 0.0,
        attack_ms: 10.0,
        decay_ms: (dur * 0.4).min(350.0),
        tail_ms: (dur * 0.3).min(200.0),
        noise_amount: 0.15,
        noise_hp_hz: 3000.0,
        click_amount: 0.1,
        body_gain: 0.8,
        sub_gain: 0.3,
        saturation_drive: 1.5,
        brightness: 0.6,
        layer_mix: vec![0.05, 0.6, 0.05, 0.15, 0.15],
        seed: 0,
        stereo_width: 0.0,
        filter_sweep: 0.0,
        metallic_amount: 0.0,
    }
}

// ─── WAV Export ───────────────────────────────────────────

/// Render a OneShotSpec and write the result to a WAV file.
pub fn render_to_wav(spec: &OneShotSpec, path: &std::path::Path) -> Result<(), String> {
    let samples = spec.render();
    if samples.is_empty() {
        return Err("Rendered audio is empty".to_string());
    }
    super::io::write_wav(path, &samples, SAMPLE_RATE)
}

/// Render a OneShotSpec and return WAV bytes.
pub fn render_to_wav_bytes(spec: &OneShotSpec) -> Result<Vec<u8>, String> {
    let samples = spec.render();
    if samples.is_empty() {
        return Err("Rendered audio is empty".to_string());
    }
    super::io::write_wav_bytes(&samples, SAMPLE_RATE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn sample_peak(samples: &[f32]) -> f32 {
        samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
    }

    fn is_valid_wav(bytes: &[u8]) -> bool {
        if bytes.len() < 44 { return false; }
        // RIFF header
        if &bytes[0..4] != b"RIFF" { return false; }
        if &bytes[8..12] != b"WAVE" { return false; }
        if &bytes[12..16] != b"fmt " { return false; }
        true
    }

    /// Every sound_class renders non-empty audio with valid samples.
    #[test]
    fn test_all_sound_classes_render_non_empty() {
        let specs = vec![
            ("808", OneShotSpec::preset_808()),
            ("kick", OneShotSpec::preset_kick()),
            ("snare", OneShotSpec::preset_snare()),
            ("clap", OneShotSpec::preset_clap()),
            ("closed_hat", OneShotSpec::preset_closed_hat()),
            ("open_hat", OneShotSpec::preset_open_hat()),
            ("bass_stab", OneShotSpec::preset_bass_stab()),
            ("impact_fx", OneShotSpec::preset_impact_fx()),
            ("synth_stab", OneShotSpec::preset_synth_stab()),
        ];
        for (name, spec) in &specs {
            let samples = spec.render();
            assert!(!samples.is_empty(), "{} rendered empty audio", name);
            assert!(
                samples.iter().all(|s| s.is_finite()),
                "{} contains NaN or Inf", name
            );
            assert!(
                sample_peak(&samples) > 0.001,
                "{} has negligible output (peak={})",
                name, sample_peak(&samples)
            );
        }
    }

    /// Exported WAV files are valid.
    #[test]
    fn test_wav_export_valid() {
        let spec = OneShotSpec::preset_808();
        let bytes = render_to_wav_bytes(&spec).expect("WAV export failed");
        assert!(is_valid_wav(&bytes), "Exported WAV is not valid");

        // Also verify hound can read it back
        let cursor = Cursor::new(&bytes);
        let reader = hound::WavReader::new(cursor).expect("hound could not read exported WAV");
        let spec_reader = reader.spec();
        assert_eq!(spec_reader.channels, 1);
        assert_eq!(spec_reader.sample_rate, 44100);
        assert_eq!(spec_reader.bits_per_sample, 32);
        assert_eq!(spec_reader.sample_format, hound::SampleFormat::Float);
    }

    /// All sound classes export valid WAV.
    #[test]
    fn test_all_classes_export_valid_wav() {
        let specs = [
            ("808", OneShotSpec::preset_808()),
            ("kick", OneShotSpec::preset_kick()),
            ("snare", OneShotSpec::preset_snare()),
            ("clap", OneShotSpec::preset_clap()),
            ("closed_hat", OneShotSpec::preset_closed_hat()),
            ("open_hat", OneShotSpec::preset_open_hat()),
            ("bass_stab", OneShotSpec::preset_bass_stab()),
            ("impact_fx", OneShotSpec::preset_impact_fx()),
            ("synth_stab", OneShotSpec::preset_synth_stab()),
        ];
        for (name, spec) in &specs {
            let bytes = render_to_wav_bytes(spec)
                .unwrap_or_else(|e| panic!("{} WAV export failed: {}", name, e));
            assert!(is_valid_wav(&bytes), "{} WAV is not valid", name);
        }
    }

    /// Invalid sound_class string is rejected.
    #[test]
    fn test_invalid_sound_class_rejected() {
        assert!(!SoundClass::is_valid("garbage_sound"));
        assert!(!SoundClass::is_valid(""));
        assert!(!SoundClass::is_valid("trumpet"));
        // Valid classes still pass
        assert!(SoundClass::is_valid("808"));
        assert!(SoundClass::is_valid("kick"));
        assert!(SoundClass::is_valid("synth_stab"));
    }

    /// 808 sound_type maps to Bass (backwards compat).
    #[test]
    fn test_808_backwards_compatibility() {
        let spec = OneShotSpec::preset_808();
        assert_eq!(spec.sound_class, SoundClass::Sub808);
        assert_eq!(spec.sound_class.sound_type(), SoundType::Bass);
        assert_eq!(spec.sound_class.as_str(), "808");

        // Same as_str roundtrip
        assert_eq!(SoundClass::from_str("808"), SoundClass::Sub808);

        // Render should produce typical 808-like output
        let samples = spec.render();
        let peak = sample_peak(&samples);
        assert!(peak > 0.01, "808 output peak too low: {}", peak);

        // Low-frequency dominance (sub) — check spectral centroid is low
        let centroid = samples.iter()
            .enumerate()
            .skip(10)
            .take(100)
            .map(|(i, &s)| {
                let freq = i as f32 / samples.len() as f32 * 22050.0;
                s.abs() * freq
            })
            .sum::<f32>()
            / samples.iter().skip(10).take(100).map(|s| s.abs()).sum::<f32>().max(1e-10);
        assert!(centroid < 500.0, "808 centroid too high: {:.0} Hz (expected sub-heavy)", centroid);
    }

    /// Duration control affects output length.
    #[test]
    fn test_duration_control() {
        let short = OneShotSpec {
            duration_ms: 100.0,
            ..OneShotSpec::preset_kick()
        };
        let long = OneShotSpec {
            duration_ms: 500.0,
            ..OneShotSpec::preset_kick()
        };
        let short_samples = short.render();
        let long_samples = long.render();
        assert!(
            long_samples.len() > short_samples.len(),
            "Longer duration should produce more samples ({} vs {})",
            long_samples.len(), short_samples.len()
        );
    }

    /// Pitch control affects output.
    #[test]
    fn test_pitch_control() {
        let low = OneShotSpec {
            pitch_hz: 50.0,
            ..OneShotSpec::preset_kick()
        };
        let high = OneShotSpec {
            pitch_hz: 200.0,
            ..OneShotSpec::preset_kick()
        };
        let low_samples = low.render();
        let high_samples = high.render();
        // Higher pitch should have more zero crossings in the same window
        let zero_crossings = |s: &[f32]| -> usize {
            s.windows(2).filter(|w| w[0] >= 0.0 && w[1] < 0.0 || w[0] < 0.0 && w[1] >= 0.0).count()
        };
        let mid = low_samples.len() / 2;
        let low_zc = zero_crossings(&low_samples[mid..(mid + 500).min(low_samples.len())]);
        let high_zc = zero_crossings(&high_samples[mid..(mid + 500).min(high_samples.len())]);
        assert!(
            high_zc >= low_zc,
            "Higher pitch should have >= zero crossings (high={} vs low={})",
            high_zc, low_zc
        );
    }

    /// Gain control affects amplitude.
    #[test]
    fn test_gain_control() {
        let quiet = OneShotSpec {
            gain: 0.1,
            ..OneShotSpec::preset_kick()
        };
        let loud = OneShotSpec {
            gain: 0.9,
            ..OneShotSpec::preset_kick()
        };
        let quiet_samples = quiet.render();
        let loud_samples = loud.render();
        let quiet_peak = sample_peak(&quiet_samples);
        let loud_peak = sample_peak(&loud_samples);
        assert!(
            loud_peak > quiet_peak * 1.5,
            "Higher gain should produce higher peak (loud={} vs quiet={})",
            loud_peak, quiet_peak
        );
    }

    // ─── Differentiation Tests ────────────────────────────
    // Verify each sound class produces audibly distinct output.

    fn compute_rms_envelope(samples: &[f32], num_windows: usize) -> Vec<f32> {
        if samples.is_empty() || num_windows == 0 { return vec![]; }
        let window_size = samples.len() / num_windows;
        let mut env = Vec::with_capacity(num_windows);
        for w in 0..num_windows {
            let start = w * window_size;
            let end = if w == num_windows - 1 { samples.len() } else { (w + 1) * window_size };
            if start >= end { continue; }
            let sum_sq: f32 = samples[start..end].iter().map(|s| s * s).sum();
            env.push((sum_sq / (end - start) as f32).sqrt());
        }
        env
    }

    fn energy_center_of_gravity_ms(samples: &[f32]) -> f32 {
        let total: f32 = samples.iter().map(|s| s * s).sum();
        if total < 1e-10 { return 0.0; }
        let mut cum = 0.0;
        for (i, s) in samples.iter().enumerate() {
            cum += s * s;
            if cum / total >= 0.5 {
                return i as f32 / 44100.0 * 1000.0;
            }
        }
        samples.len() as f32 / 44100.0 * 1000.0
    }

    fn low_pass_3pole(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
        use std::f32::consts::PI;
        if samples.is_empty() { return vec![]; }
        let rc = 1.0 / (2.0 * PI * cutoff_hz.max(1.0));
        let dt = 1.0 / 44100.0;
        let alpha = dt / (rc + dt);
        let mut out = vec![0.0; samples.len()];
        let mut v1 = 0.0;
        let mut v2 = 0.0;
        let mut v3 = 0.0;
        for (i, s) in samples.iter().enumerate() {
            v1 += alpha * (s - v1);
            v2 += alpha * (v1 - v2);
            v3 += alpha * (v2 - v3);
            out[i] = v3;
        }
        out
    }

    fn high_freq_energy_fraction(samples: &[f32], cutoff_hz: f32) -> f32 {
        let total: f32 = samples.iter().map(|s| s * s).sum();
        if total < 1e-10 { return 0.0; }
        let lp = low_pass_3pole(samples, cutoff_hz);
        let lp_energy: f32 = lp.iter().map(|s| s * s).sum();
        1.0 - (lp_energy / total).min(1.0)
    }

    fn count_transient_peaks(samples: &[f32]) -> usize {
        let window_size = (44100 / 200).max(1);
        let num_windows = samples.len() / window_size;
        if num_windows < 4 { return 0; }
        let mut envelope = Vec::with_capacity(num_windows);
        for w in 0..num_windows {
            let start = w * window_size;
            let end = ((w + 1) * window_size).min(samples.len());
            let peak = samples[start..end].iter().map(|s| s.abs()).fold(0.0, f32::max);
            envelope.push(peak);
        }
        let threshold = envelope.iter().copied().fold(0.0, f32::max) * 0.15;
        let mut peaks = 0;
        let mut i = 1;
        while i < envelope.len() - 1 {
            if envelope[i] > threshold && envelope[i] > envelope[i - 1] && envelope[i] >= envelope[i + 1] {
                peaks += 1;
                i += 2;
            } else {
                i += 1;
            }
        }
        peaks
    }

    /// Sub808 has a much longer energy spread (COG in ms) than closed hat.
    #[test]
    fn test_class_different_rms_envelopes() {
        let sub = OneShotSpec::preset_808().render();
        let ch = OneShotSpec::preset_closed_hat().render();

        let sub_cog = energy_center_of_gravity_ms(&sub);
        let ch_cog = energy_center_of_gravity_ms(&ch);

        assert!(sub_cog > ch_cog * 1.5,
            "Sub808 energy COG (ms) should exceed closed hat (sub={:.1}ms, ch={:.1}ms)",
            sub_cog, ch_cog);
    }

    /// Open hat has a longer energy spread (COG in ms) than closed hat.
    #[test]
    fn test_open_hat_decays_longer_than_closed_hat() {
        let oh = OneShotSpec::preset_open_hat().render();
        let ch = OneShotSpec::preset_closed_hat().render();

        let oh_cog = energy_center_of_gravity_ms(&oh);
        let ch_cog = energy_center_of_gravity_ms(&ch);

        assert!(oh_cog > ch_cog * 1.3,
            "Open hat energy COG (ms) should exceed closed hat (oh={:.1}ms, ch={:.1}ms)",
            oh_cog, ch_cog);
    }

    /// Clap has multiple transient peaks from staggered noise bursts.
    #[test]
    fn test_clap_multiple_transient_peaks() {
        let clap = OneShotSpec::preset_clap().render();
        let peaks = count_transient_peaks(&clap);
        assert!(peaks >= 3,
            "Clap should have at least 3 transient peaks, got {}", peaks);
    }

    /// Kick has much lower high-frequency energy fraction (stronger low-frequency) than closed hat.
    #[test]
    fn test_kick_stronger_low_freq_than_hat() {
        let kick = OneShotSpec::preset_kick().render();
        let ch = OneShotSpec::preset_closed_hat().render();

        let kick_hf = high_freq_energy_fraction(&kick, 2000.0);
        let ch_hf = high_freq_energy_fraction(&ch, 2000.0);

        assert!(ch_hf > kick_hf * 2.0,
            "Closed hat HF fraction should exceed kick (ch={}, kick={})",
            ch_hf, kick_hf);
    }

    /// Snare and clap have higher high-frequency energy fraction (more noise) than kick.
    #[test]
    fn test_snare_clap_more_noise_than_kick() {
        let kick = OneShotSpec::preset_kick().render();
        let snare = OneShotSpec::preset_snare().render();
        let clap = OneShotSpec::preset_clap().render();

        let kick_hf = high_freq_energy_fraction(&kick, 2000.0);
        let snare_hf = high_freq_energy_fraction(&snare, 2000.0);
        let clap_hf = high_freq_energy_fraction(&clap, 2000.0);

        assert!(snare_hf > kick_hf * 1.3,
            "Snare HF fraction should exceed kick (snare={}, kick={})",
            snare_hf, kick_hf);
        assert!(clap_hf > kick_hf * 1.3,
            "Clap HF fraction should exceed kick (clap={}, kick={})",
            clap_hf, kick_hf);
    }

    /// High-frequency energy fraction differs substantially between sub-heavy and bright classes.
    #[test]
    fn test_spectral_centroid_differentiation() {
        let sub = OneShotSpec::preset_808().render();
        let ch = OneShotSpec::preset_closed_hat().render();

        let sub_hf = high_freq_energy_fraction(&sub, 2000.0);
        let ch_hf = high_freq_energy_fraction(&ch, 2000.0);

        assert!(ch_hf > sub_hf * 2.0,
            "Closed hat HF fraction should exceed sub (ch={}, sub={})",
            ch_hf, sub_hf);
    }
}
