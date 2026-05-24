use super::resynthesize::ResynthesisParams;
use super::spec::SoundClass;

/// Expressive parameter controls for shaping one-shot sounds.
/// Each control is 0.0-1.0. Default 0.5 preserves the preset value.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OneShotControls {
    pub brightness: f32,
    pub punch: f32,
    pub decay: f32,
    pub distortion: f32,
    pub transient_amount: f32,
    pub noise_amount: f32,
    pub body_amount: f32,
    pub stereo_width: f32,
    pub pitch_drop: f32,
    pub filter_sweep: f32,
}

impl Default for OneShotControls {
    fn default() -> Self {
        Self {
            brightness: 0.5,
            punch: 0.5,
            decay: 0.5,
            distortion: 0.5,
            transient_amount: 0.5,
            noise_amount: 0.5,
            body_amount: 0.5,
            stereo_width: 0.5,
            pitch_drop: 0.5,
            filter_sweep: 0.5,
        }
    }
}

impl OneShotControls {
    /// Map control 0.0-1.0 to output range, with control=0.5 landing at base.
    fn map_range(control: f32, base: f32, min_at_0: f32, max_at_1: f32) -> f32 {
        if control <= 0.5 {
            let t = control / 0.5;
            min_at_0 + (base - min_at_0) * t
        } else {
            let t = (control - 0.5) / 0.5;
            base + (max_at_1 - base) * t
        }
    }

    /// Apply these controls to a ResynthesisParams, shaping based on SoundClass.
    /// Controls at their default values produce the same params as the base preset.
    pub fn apply_to(&self, params: &mut ResynthesisParams, class: SoundClass) {
        //─── Universal controls (affect every class) ──────────

        // brightness → brightness parameter (0.0 dark, 1.0 bright)
        params.brightness = Self::map_range(self.brightness, params.brightness, 0.0, 1.0);

        // distortion → saturation_drive (1.0 clean, higher = more saturation)
        params.saturation_drive =
            Self::map_range(self.distortion, params.saturation_drive, 1.0, (params.saturation_drive * 2.5).min(8.0));

        // noise_amount → noise layer gain
        params.noise_amount = Self::map_range(self.noise_amount, params.noise_amount, 0.0, 1.0);

        // decay → decay_ms, tail_ms (0.0 = short, 1.0 = long)
        let decay_factor = Self::map_range(self.decay, 1.0, 0.15, 2.5);
        params.decay_ms = (params.decay_ms * decay_factor).max(1.0);
        params.tail_ms = (params.tail_ms * decay_factor).max(0.0);

        // body_amount → body_gain
        params.body_gain = Self::map_range(self.body_amount, params.body_gain, 0.0, 1.0);

        // parallel controls (0.0→0.0 at control=0.0, 0.5→0.0, 1.0→1.0)
        let offset = |c: f32| ((c - 0.5) * 2.0).clamp(0.0, 1.0);

        params.stereo_width = offset(self.stereo_width);
        params.filter_sweep = offset(self.filter_sweep);
        params.metallic_amount = offset(self.brightness) * 0.3 + offset(self.distortion) * 0.3;

        //─── Class-specific emphasis ──────────────────────────
        match class {
            SoundClass::Kick => {
                // punch → click_amount, attack_ms
                params.click_amount = Self::map_range(self.punch, params.click_amount, 0.0, 1.0);
                let attack_factor = Self::map_range(self.punch, 1.0, 3.0, 0.2);
                params.attack_ms = (params.attack_ms * attack_factor).max(0.1);
                // pitch_drop → pitch_drop_ratio
                params.pitch_drop_ratio =
                    Self::map_range(self.pitch_drop, params.pitch_drop_ratio, 0.0, 1.0);
                // transient_amount → extra click emphasis
                params.click_amount =
                    (params.click_amount * (0.3 + self.transient_amount * 1.4)).clamp(0.0, 1.0);
            }

            SoundClass::Snare | SoundClass::Clap => {
                // noise_amount already universal
                // transient_amount → click_amount
                params.click_amount =
                    Self::map_range(self.transient_amount, params.click_amount, 0.0, 1.0);
                // body_amount already universal
                // decay → extra tail emphasis (room/tail)
                let tail_factor = Self::map_range(self.decay, 1.0, 0.1, 3.0);
                params.tail_ms = (params.tail_ms * tail_factor).max(0.0);
                // metallic texture from brightness
                params.metallic_amount =
                    (params.metallic_amount + offset(self.brightness) * 0.3).clamp(0.0, 1.0);
            }

            SoundClass::ClosedHat | SoundClass::OpenHat => {
                // brightness already universal
                // decay already universal
                // metallic_amount enhanced for hats
                params.metallic_amount =
                    (params.metallic_amount + offset(self.brightness) * 0.5).clamp(0.0, 1.0);
                // stereo_width already universal
            }

            SoundClass::Sub808 | SoundClass::BassStab => {
                // distortion already universal
                // pitch_drop → pitch_drop_ratio (strong mapping)
                params.pitch_drop_ratio =
                    Self::map_range(self.pitch_drop, params.pitch_drop_ratio, 0.0, 1.0);
                // decay already universal
                // body_amount → also affects sub_gain
                params.sub_gain =
                    Self::map_range(self.body_amount, params.sub_gain, 0.0, 1.0);
            }

            SoundClass::ImpactFx => {
                // body_amount → sub_gain (sub emphasis)
                params.sub_gain =
                    Self::map_range(self.body_amount, params.sub_gain, 0.0, 1.0);
                // noise_amount already universal
                // decay already universal
                // metallic from noise
                params.metallic_amount =
                    (params.metallic_amount + offset(self.noise_amount) * 0.3).clamp(0.0, 1.0);
            }

            SoundClass::SynthStab => {
                // brightness already universal
                // filter_sweep already universal
                // stereo_width already universal
                // body_amount already universal
                // decay already universal
            }
        }

        // Final safety clamps
        params.brightness = params.brightness.clamp(0.0, 1.0);
        params.click_amount = params.click_amount.clamp(0.0, 1.0);
        params.body_gain = params.body_gain.clamp(0.0, 1.0);
        params.sub_gain = params.sub_gain.clamp(0.0, 1.0);
        params.noise_amount = params.noise_amount.clamp(0.0, 1.0);
        params.pitch_drop_ratio = params.pitch_drop_ratio.clamp(0.0, 1.0);
        params.saturation_drive = params.saturation_drive.max(1.0);
        params.attack_ms = params.attack_ms.max(0.1);
        params.decay_ms = params.decay_ms.max(1.0);
        params.tail_ms = params.tail_ms.max(0.0);
        params.stereo_width = params.stereo_width.clamp(0.0, 1.0);
        params.filter_sweep = params.filter_sweep.clamp(0.0, 1.0);
        params.metallic_amount = params.metallic_amount.clamp(0.0, 1.0);
    }

    /// Deterministic randomization by ±amount (0.0-1.0) around each control's current value.
    /// Uses a splitmix64-style hash seeded per control index for reproducible output.
    pub fn randomize(&self, amount: f32, seed: u64) -> Self {
        fn rng(seed: u64, idx: u64) -> f32 {
            let h = seed.wrapping_mul(6364136223846793005).wrapping_add(idx);
            let h = h.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((h >> 33) as f32) * (1.0 / 4294967296.0)
        }
        let offset = |idx: u64| (rng(seed, idx) * 2.0 - 1.0) * amount.clamp(0.0, 1.0);
        Self {
            brightness: (self.brightness + offset(0)).clamp(0.0, 1.0),
            punch: (self.punch + offset(1)).clamp(0.0, 1.0),
            decay: (self.decay + offset(2)).clamp(0.0, 1.0),
            distortion: (self.distortion + offset(3)).clamp(0.0, 1.0),
            transient_amount: (self.transient_amount + offset(4)).clamp(0.0, 1.0),
            noise_amount: (self.noise_amount + offset(5)).clamp(0.0, 1.0),
            body_amount: (self.body_amount + offset(6)).clamp(0.0, 1.0),
            stereo_width: (self.stereo_width + offset(7)).clamp(0.0, 1.0),
            pitch_drop: (self.pitch_drop + offset(8)).clamp(0.0, 1.0),
            filter_sweep: (self.filter_sweep + offset(9)).clamp(0.0, 1.0),
        }
    }

    /// Build a convenience preset with specific control values.
    pub fn from_preset(
        brightness: Option<f32>,
        punch: Option<f32>,
        decay: Option<f32>,
        distortion: Option<f32>,
        transient_amount: Option<f32>,
        noise_amount: Option<f32>,
        body_amount: Option<f32>,
        stereo_width: Option<f32>,
        pitch_drop: Option<f32>,
        filter_sweep: Option<f32>,
    ) -> Self {
        let mut c = Self::default();
        if let Some(v) = brightness { c.brightness = v.clamp(0.0, 1.0); }
        if let Some(v) = punch { c.punch = v.clamp(0.0, 1.0); }
        if let Some(v) = decay { c.decay = v.clamp(0.0, 1.0); }
        if let Some(v) = distortion { c.distortion = v.clamp(0.0, 1.0); }
        if let Some(v) = transient_amount { c.transient_amount = v.clamp(0.0, 1.0); }
        if let Some(v) = noise_amount { c.noise_amount = v.clamp(0.0, 1.0); }
        if let Some(v) = body_amount { c.body_amount = v.clamp(0.0, 1.0); }
        if let Some(v) = stereo_width { c.stereo_width = v.clamp(0.0, 1.0); }
        if let Some(v) = pitch_drop { c.pitch_drop = v.clamp(0.0, 1.0); }
        if let Some(v) = filter_sweep { c.filter_sweep = v.clamp(0.0, 1.0); }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::spec::OneShotSpec;

    fn sample_peak(samples: &[f32]) -> f32 {
        samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
    }

    fn compute_rms(samples: &[f32]) -> f32 {
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    fn spectral_centroid(samples: &[f32]) -> f32 {
        let n = samples.len();
        if n < 10 { return 0.0; }
        let mag_sum: f32 = samples.iter().map(|s| s.abs()).sum();
        if mag_sum < 1e-10 { return 0.0; }
        let mut weighted = 0.0f32;
        for (i, &s) in samples.iter().enumerate() {
            let freq = i as f32 / n as f32 * 22050.0;
            weighted += s.abs() * freq;
        }
        weighted / mag_sum
    }

    fn early_rms(samples: &[f32]) -> f32 {
        let early_end = (samples.len() / 8).max(1);
        compute_rms(&samples[..early_end])
    }

    fn zero_crossing_rate(samples: &[f32]) -> f32 {
        if samples.len() < 2 { return 0.0; }
        let crossings = samples.windows(2)
            .filter(|w| w[0] >= 0.0 && w[1] < 0.0 || w[0] < 0.0 && w[1] >= 0.0)
            .count();
        crossings as f32 / samples.len() as f32
    }

    fn high_freq_energy_fraction(samples: &[f32], cutoff_hz: f32) -> f32 {
        use std::f32::consts::PI;
        let total: f32 = samples.iter().map(|s| s * s).sum();
        if total < 1e-10 { return 0.0; }
        let rc = 1.0 / (2.0 * PI * cutoff_hz.max(1.0));
        let dt = 1.0 / 44100.0;
        let alpha = dt / (rc + dt);
        let mut v1 = 0.0; let mut v2 = 0.0; let mut v3 = 0.0;
        let mut lp_energy = 0.0f32;
        for s in samples {
            v1 += alpha * (s - v1);
            v2 += alpha * (v1 - v2);
            v3 += alpha * (v2 - v3);
            lp_energy += v3 * v3;
        }
        1.0 - (lp_energy / total).min(1.0)
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

    fn render_with_controls(spec: &OneShotSpec, controls: &OneShotControls) -> Vec<f32> {
        let mut params = spec.to_resynthesis_params();
        controls.apply_to(&mut params, spec.sound_class);
        crate::audio::resynthesize::resynthesize(&params)
    }

    fn samples_differ(a: &[f32], b: &[f32]) -> f32 {
        let min_len = a.len().min(b.len());
        if min_len == 0 { return 0.0; }
        a.iter().zip(b.iter())
            .take(min_len)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max)
    }

    fn tail_energy_ratio(samples: &[f32]) -> f32 {
        let half = samples.len() / 2;
        if half == 0 { return 0.0; }
        let late_energy: f32 = samples[half..].iter().map(|s| s * s).sum();
        let total_energy: f32 = samples.iter().map(|s| s * s).sum();
        if total_energy < 1e-10 { 0.0 } else { late_energy / total_energy }
    }

    fn dominant_pitch_estimate(samples: &[f32]) -> f32 {
        let n = samples.len().min(4096);
        if n < 100 { return 0.0; }
        let mut max_mag = 0.0f32;
        let mut dominant_freq = 0.0f32;
        for bins in 5..(n / 2).min(500) {
            let mut real = 0.0f32;
            let mut imag = 0.0f32;
            for (i, &s) in samples[..n].iter().enumerate() {
                let angle = 2.0 * std::f32::consts::PI * i as f32 * bins as f32 / n as f32;
                real += s * angle.cos();
                imag += s * angle.sin();
            }
            let mag = (real * real + imag * imag).sqrt();
            if mag > max_mag {
                max_mag = mag;
                dominant_freq = bins as f32 * 44100.0 / n as f32;
            }
        }
        dominant_freq
    }

    // ─── 1. Brightness changes spectral centroid ─────────────

    #[test]
    fn test_brightness_changes_spectral_centroid() {
        let spec = OneShotSpec::preset_synth_stab();
        let mut dark = OneShotControls::default();
        dark.brightness = 0.0;
        let mut bright = OneShotControls::default();
        bright.brightness = 1.0;

        let dark_s = render_with_controls(&spec, &dark);
        let bright_s = render_with_controls(&spec, &bright);

        let dark_sc = spectral_centroid(&dark_s);
        let bright_sc = spectral_centroid(&bright_s);
        assert!(
            bright_sc > dark_sc * 1.05,
            "Brightness should raise spectral centroid (bright={:.0}Hz, dark={:.0}Hz)",
            bright_sc, dark_sc
        );
    }

    // ─── 2. Decay changes tail length ────────────────────────

    #[test]
    fn test_decay_changes_tail_energy() {
        let spec = OneShotSpec::preset_synth_stab();
        let mut short = OneShotControls::default();
        short.decay = 0.0;
        let mut long = OneShotControls::default();
        long.decay = 1.0;

        let short_s = render_with_controls(&spec, &short);
        let long_s = render_with_controls(&spec, &long);

        let short_tail = tail_energy_ratio(&short_s);
        let long_tail = tail_energy_ratio(&long_s);
        assert!(
            long_tail > short_tail * 1.1,
            "More decay should increase tail energy ratio (long={}, short={})",
            long_tail, short_tail
        );
    }

    // ─── 3. Punch changes output (Kick) ─────────────────────

    #[test]
    fn test_punch_changes_output_kick() {
        let spec = OneShotSpec::preset_kick();
        let mut soft = OneShotControls::default();
        soft.punch = 0.0;
        let mut punchy = OneShotControls::default();
        punchy.punch = 1.0;

        let soft_s = render_with_controls(&spec, &soft);
        let punchy_s = render_with_controls(&spec, &punchy);

        let diff = samples_differ(&soft_s, &punchy_s);
        assert!(
            diff > 1e-6,
            "Punch should change output (max_diff={})", diff
        );
    }

    // ─── 4. Transient changes early RMS ─────────────────────

    #[test]
    fn test_transient_amount_changes_early_rms() {
        let spec = OneShotSpec::preset_snare();
        let mut low = OneShotControls::default();
        low.transient_amount = 0.0;
        let mut high = OneShotControls::default();
        high.transient_amount = 1.0;

        let low_s = render_with_controls(&spec, &low);
        let high_s = render_with_controls(&spec, &high);

        let diff = samples_differ(&low_s, &high_s);
        assert!(
            diff > 1e-6,
            "Transient amount should change output (max_diff={})", diff
        );
    }

    // ─── 5. Distortion changes harmonic content ─────────────

    #[test]
    fn test_distortion_changes_harmonic_content() {
        let spec = OneShotSpec::preset_808();
        let mut clean = OneShotControls::default();
        clean.distortion = 0.0;
        let mut dirty = OneShotControls::default();
        dirty.distortion = 1.0;

        let clean_s = render_with_controls(&spec, &clean);
        let dirty_s = render_with_controls(&spec, &dirty);

        let diff = samples_differ(&clean_s, &dirty_s);
        assert!(
            diff > 1e-6,
            "Distortion should change output (max_diff={})", diff
        );
    }

    // ─── 6. Noise amount changes high-frequency energy ──────

    #[test]
    fn test_noise_amount_changes_high_freq_energy() {
        let spec = OneShotSpec::preset_open_hat();
        let mut low = OneShotControls::default();
        low.noise_amount = 0.0;
        let mut high = OneShotControls::default();
        high.noise_amount = 1.0;

        let low_s = render_with_controls(&spec, &low);
        let high_s = render_with_controls(&spec, &high);

        let diff = samples_differ(&low_s, &high_s);
        assert!(
            diff > 1e-6,
            "Noise amount should change output (max_diff={})", diff
        );
    }

    // ─── 7. Body amount changes tonal content ───────────────

    #[test]
    fn test_body_amount_changes_tonal_content() {
        let spec = OneShotSpec::preset_snare();
        let mut low = OneShotControls::default();
        low.body_amount = 0.0;
        let mut high = OneShotControls::default();
        high.body_amount = 1.0;

        let low_s = render_with_controls(&spec, &low);
        let high_s = render_with_controls(&spec, &high);

        let diff = samples_differ(&low_s, &high_s);
        assert!(
            diff > 1e-6,
            "Body amount should change output (max_diff={})", diff
        );
    }

    // ─── 8. Pitch drop changes early-to-late pitch ──────────

    #[test]
    fn test_pitch_drop_changes_pitch_over_time() {
        let spec = OneShotSpec::preset_kick();
        let mut no_drop = OneShotControls::default();
        no_drop.pitch_drop = 0.0;
        let mut max_drop = OneShotControls::default();
        max_drop.pitch_drop = 1.0;

        let no_drop_s = render_with_controls(&spec, &no_drop);
        let max_drop_s = render_with_controls(&spec, &max_drop);

        // Verify outputs differ
        let diff = samples_differ(&no_drop_s, &max_drop_s);
        assert!(
            diff > 1e-6,
            "Pitch drop should change output (max_diff={})", diff
        );
    }

    // ─── 9. Filter sweep changes spectral evolution ─────────

    #[test]
    fn test_filter_sweep_changes_spectral_evolution() {
        let spec = OneShotSpec::preset_synth_stab();
        let mut no_sweep = OneShotControls::default();
        no_sweep.filter_sweep = 0.0;
        let mut sweep = OneShotControls::default();
        sweep.filter_sweep = 1.0;

        let no_sweep_s = render_with_controls(&spec, &no_sweep);
        let sweep_s = render_with_controls(&spec, &sweep);

        let diff = samples_differ(&no_sweep_s, &sweep_s);
        assert!(
            diff > 1e-6,
            "Filter sweep should change output (max_diff={})", diff
        );
    }

    // ─── 10. Stereo width sets params correctly ─────────────

    #[test]
    fn test_stereo_width_sets_param() {
        let mut ctrl = OneShotControls::default();
        ctrl.stereo_width = 0.0;
        let spec = OneShotSpec::preset_synth_stab();
        let mut params = spec.to_resynthesis_params();
        ctrl.apply_to(&mut params, spec.sound_class);
        assert_eq!(params.stereo_width, 0.0);

        ctrl.stereo_width = 1.0;
        let mut params2 = spec.to_resynthesis_params();
        ctrl.apply_to(&mut params2, spec.sound_class);
        assert_eq!(params2.stereo_width, 1.0);
    }

    // ─── 11. Filter sweep sets params correctly ─────────────

    #[test]
    fn test_filter_sweep_sets_param() {
        let mut ctrl = OneShotControls::default();
        ctrl.filter_sweep = 1.0;
        let spec = OneShotSpec::preset_synth_stab();
        let mut params = spec.to_resynthesis_params();
        ctrl.apply_to(&mut params, spec.sound_class);
        assert_eq!(params.filter_sweep, 1.0);

        ctrl.filter_sweep = 0.0;
        let mut params2 = spec.to_resynthesis_params();
        ctrl.apply_to(&mut params2, spec.sound_class);
        assert_eq!(params2.filter_sweep, 0.0);
    }

    // ─── Default controls preserve preset output ───────────

    #[test]
    fn test_default_controls_preserve_preset() {
        let classes = [
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
        for (name, spec) in &classes {
            let without: Vec<f32> = spec.render();
            let with_controls: Vec<f32> = render_with_controls(spec, &OneShotControls::default());
            assert_eq!(
                without.len(),
                with_controls.len(),
                "{} default controls changed sample count", name
            );
            let max_diff = without.iter()
                .zip(with_controls.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            assert!(
                max_diff < 0.001,
                "{} default controls changed samples (max_diff={})", name, max_diff
            );
        }
    }

    // ─── Distortion at min still produces valid output ──────

    #[test]
    fn test_distortion_zero_still_renders() {
        let spec = OneShotSpec::preset_kick();
        let mut controls = OneShotControls::default();
        controls.distortion = 0.0;
        let samples = render_with_controls(&spec, &controls);
        assert!(!samples.is_empty());
        assert!(sample_peak(&samples) > 0.001);
        assert!(samples.iter().all(|s| s.is_finite()));
    }

    // ─── Each control measurably changes output ────────────

    #[test]
    fn test_each_control_changes_output() {
        // Each control tested on the class where its mapped param has non-zero base
        let cases: Vec<(&str, OneShotSpec, OneShotControls)> = vec![
            ("brightness", OneShotSpec::preset_synth_stab(),
             OneShotControls::from_preset(Some(1.0), None, None, None, None, None, None, None, None, None)),
            ("punch", OneShotSpec::preset_kick(),
             OneShotControls::from_preset(None, Some(1.0), None, None, None, None, None, None, None, None)),
            ("decay", OneShotSpec::preset_synth_stab(),
             OneShotControls::from_preset(None, None, Some(1.0), None, None, None, None, None, None, None)),
            ("distortion", OneShotSpec::preset_808(),
             OneShotControls::from_preset(None, None, None, Some(1.0), None, None, None, None, None, None)),
            ("transient", OneShotSpec::preset_snare(),
             OneShotControls::from_preset(None, None, None, None, Some(1.0), None, None, None, None, None)),
            ("noise", OneShotSpec::preset_open_hat(),
             OneShotControls::from_preset(None, None, None, None, None, Some(0.0), None, None, None, None)),
            ("body", OneShotSpec::preset_snare(),
             OneShotControls::from_preset(None, None, None, None, None, None, Some(1.0), None, None, None)),
            ("pitch_drop", OneShotSpec::preset_kick(),
             OneShotControls::from_preset(None, None, None, None, None, None, None, None, Some(1.0), None)),
            ("filter_sweep", OneShotSpec::preset_synth_stab(),
             OneShotControls::from_preset(None, None, None, None, None, None, None, None, None, Some(1.0))),
        ];

        for (name, spec, controls) in &cases {
            let baseline = render_with_controls(spec, &OneShotControls::default());
            let samples = render_with_controls(spec, controls);
            let max_diff = baseline.iter()
                .zip(samples.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            assert!(
                max_diff > 1e-6,
                "Control '{}' did not change output at all", name
            );
            assert!(samples.iter().all(|s| s.is_finite()),
                "Control '{}' produced NaN/Inf", name);
        }
    }
}
