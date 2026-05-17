use super::SAMPLE_RATE;
use super::dsp;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpectralEditParams {
    pub isolate_transient: f32,     // 0.0-1.0
    pub isolate_body: f32,          // 0.0-1.0
    pub isolate_tail: f32,          // 0.0-1.0
    pub brighten_region: f32,       // -1.0 (dark) to 1.0 (bright)
    pub mud_remove: f32,            // 0.0-1.0
    pub harshness_soften: f32,      // 0.0-1.0
    pub spectral_tilt: f32,         // -1.0 to 1.0
    pub noise_reduction: f32,       // 0.0-1.0
    pub transient_emphasis: f32,    // 0.0-1.0
    pub low_cut_hz: f32,            // 0.0 = off
    pub high_cut_hz: f32,           // 0.0 = off
    pub notch_hz: f32,              // center frequency
    pub notch_q: f32,               // quality factor
}

impl Default for SpectralEditParams {
    fn default() -> Self {
        Self {
            isolate_transient: 1.0,
            isolate_body: 1.0,
            isolate_tail: 1.0,
            brighten_region: 0.0,
            mud_remove: 0.0,
            harshness_soften: 0.0,
            spectral_tilt: 0.0,
            noise_reduction: 0.0,
            transient_emphasis: 0.0,
            low_cut_hz: 0.0,
            high_cut_hz: 0.0,
            notch_hz: 0.0,
            notch_q: 10.0,
        }
    }
}

// ─── Transient/Body/Tail Isolation ──────────────────────

pub fn extract_transient_region(samples: &[f32]) -> Vec<f32> {
    let mut isolated = vec![0.0f32; samples.len()];
    let onset_len = (SAMPLE_RATE as f32 * 0.015) as usize;
    let threshold = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.2;
    for i in 10..samples.len().min(SAMPLE_RATE as usize) {
        if samples[i].abs() > threshold {
            let end = (i + onset_len).min(samples.len());
            for j in i..end {
                let t = (j - i) as f32 / (end - i).max(1) as f32;
                let env = (1.0 - t * t).max(0.0);
                isolated[j] = samples[j] * env;
            }
            break;
        }
    }
    isolated
}

pub fn extract_body_region(samples: &[f32]) -> Vec<f32> {
    let mut isolated = samples.to_vec();
    let onset_len = (SAMPLE_RATE as f32 * 0.02) as usize;
    let threshold = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * 0.15;
    for i in 10..samples.len().min(SAMPLE_RATE as usize) {
        if samples[i].abs() > threshold {
            let end = (i + onset_len).min(samples.len());
            for j in i..end {
                let t = (j - i) as f32 / (end - i).max(1) as f32;
                isolated[j] *= (t * t).min(1.0);
            }
            break;
        }
    }
    // Fade tail
    let fade_start = (samples.len() as f32 * 0.6) as usize;
    for i in fade_start..samples.len() {
        let t = (i - fade_start) as f32 / (samples.len() - fade_start).max(1) as f32;
        isolated[i] *= 1.0 - t * t;
    }
    isolated
}

pub fn extract_tail_region(samples: &[f32]) -> Vec<f32> {
    let mut isolated = samples.to_vec();
    let tail_start = (samples.len() as f32 * 0.4) as usize;
    for i in 0..tail_start.min(samples.len()) {
        isolated[i] *= 0.0;
    }
    // Fade in the tail
    let fade_in = (SAMPLE_RATE as f32 * 0.02) as usize;
    for i in tail_start..(tail_start + fade_in).min(samples.len()) {
        let t = (i - tail_start) as f32 / fade_in as f32;
        isolated[i] *= t;
    }
    isolated
}

pub fn isolate_regions(samples: &[f32], transient: f32, body: f32, tail: f32) -> Vec<f32> {
    if transient >= 1.0 && body >= 1.0 && tail >= 1.0 {
        return samples.to_vec();
    }

    let trans = extract_transient_region(samples);
    let body_sig = extract_body_region(samples);
    let tail_sig = extract_tail_region(samples);

    let mut output = vec![0.0f32; samples.len()];
    for i in 0..samples.len() {
        let mut val = 0.0;
        val += trans.get(i).copied().unwrap_or(0.0) * transient;
        val += body_sig.get(i).copied().unwrap_or(0.0) * body;
        val += tail_sig.get(i).copied().unwrap_or(0.0) * tail;
        output[i] = val;
    }
    output
}

// ─── Spectral Brightening/Darkening ─────────────────────

pub fn brighten_spectral_region(samples: &mut [f32], amount: f32, freq_hz: f32) {
    if amount.abs() < 0.05 { return; }
    if amount > 0.0 {
        dsp::biquad_high_shelf(samples, freq_hz, amount * 8.0, 0.7);
    } else {
        dsp::biquad_low_shelf(samples, freq_hz, amount * 6.0, 0.7);
    }
}

// ─── Mud Removal ────────────────────────────────────────

pub fn remove_mud(samples: &mut [f32], amount: f32) {
    if amount <= 0.0 { return; }
    // Cut in the 150-300 Hz muddy range
    dsp::biquad_low_shelf(samples, 200.0, -amount * 6.0, 0.7);
    // Slight high shelf boost to compensate
    if amount > 0.5 {
        dsp::biquad_high_shelf(samples, 3000.0, amount * 1.5, 0.7);
    }
}

// ─── Harshness Softening ────────────────────────────────

pub fn soften_harshness(samples: &mut [f32], amount: f32) {
    if amount <= 0.0 { return; }
    // Target 2-6 kHz region where harshness lives
    dsp::biquad_peaking(samples, 3500.0, -amount * 4.0, 2.0);
    dsp::biquad_peaking(samples, 6000.0, -amount * 2.0, 1.5);
    // Gentle low-pass to tame extreme highs
    if amount > 0.7 {
        dsp::low_pass(samples, 12000.0 - (amount - 0.7) * 4000.0);
    }
}

// ─── Spectral Tilt ──────────────────────────────────────

pub fn apply_tilt(samples: &mut [f32], tilt: f32) {
    dsp::tilt_spectrum(samples, tilt);
}

// ─── Noise Reduction (simple spectral gate) ─────────────

pub fn reduce_noise(samples: &mut [f32], amount: f32) {
    if amount <= 0.0 { return; }
    // Simple noise gate by reducing low-level signal
    let threshold = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max) * (0.02 + (1.0 - amount) * 0.08);
    for sample in samples.iter_mut() {
        if sample.abs() < threshold {
            *sample *= amount * 0.5;
        }
    }
    // Hiss reduction: gentle low-pass on noise floor
    dsp::low_pass(samples, 14000.0 - amount * 4000.0);
}

// ─── Transient Emphasis ─────────────────────────────────

pub fn emphasize_transients(samples: &mut [f32], amount: f32) {
    if amount <= 0.0 { return; }
    dsp::transient_enhance(samples, amount * 6.0);
    if amount > 0.5 {
        dsp::biquad_high_shelf(samples, 3000.0, (amount - 0.5) * 3.0, 0.7);
    }
}

// ─── Filtering ──────────────────────────────────────────

pub fn apply_cutoff(samples: &mut [f32], low_hz: f32, high_hz: f32) {
    if low_hz > 20.0 {
        dsp::high_pass(samples, low_hz);
    }
    if high_hz > 20.0 && high_hz < SAMPLE_RATE as f32 / 2.0 {
        dsp::low_pass(samples, high_hz);
    }
}

pub fn apply_notch(samples: &mut [f32], freq_hz: f32, q: f32) {
    if freq_hz <= 0.0 || freq_hz >= SAMPLE_RATE as f32 / 2.0 { return; }
    dsp::biquad_peaking(samples, freq_hz, -12.0, q);
}

// ─── Master Spectral Processing ─────────────────────────

pub fn apply_spectral_edits(samples: &mut Vec<f32>, params: &SpectralEditParams) {
    if samples.is_empty() { return; }

    // 1. Isolate regions (rewrite samples)
    if params.isolate_transient < 1.0 || params.isolate_body < 1.0 || params.isolate_tail < 1.0 {
        let edited = isolate_regions(samples, params.isolate_transient, params.isolate_body, params.isolate_tail);
        *samples = edited;
    }

    // 2. Apply low/high cut
    apply_cutoff(samples, params.low_cut_hz, params.high_cut_hz);

    // 3. Notch
    if params.notch_hz > 0.0 {
        apply_notch(samples, params.notch_hz, params.notch_q);
    }

    // 4. Mud removal
    if params.mud_remove > 0.0 {
        remove_mud(samples, params.mud_remove);
    }

    // 5. Harshness softening
    if params.harshness_soften > 0.0 {
        soften_harshness(samples, params.harshness_soften);
    }

    // 6. Spectral tilt
    if params.spectral_tilt.abs() > 0.05 {
        apply_tilt(samples, params.spectral_tilt);
    }

    // 7. Brighten region
    if params.brighten_region.abs() > 0.05 {
        brighten_spectral_region(samples, params.brighten_region, 3000.0);
    }

    // 8. Noise reduction
    if params.noise_reduction > 0.0 {
        reduce_noise(samples, params.noise_reduction);
    }

    // 9. Transient emphasis
    if params.transient_emphasis > 0.0 {
        emphasize_transients(samples, params.transient_emphasis);
    }

    // Normalize after edits
    super::process::normalize_peak(samples, -1.0);
}
