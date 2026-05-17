use std::f32::consts::PI;
use super::{SoundType, SAMPLE_RATE};
use super::analyze::AudioAnalysis;
use super::resynthesize::{self};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HybridParams {
    /// Blend ratio between original and synthesis (0=original, 1=synth)
    pub synth_blend: f32,
    /// Replace transient entirely
    pub replace_transient: bool,
    /// Replace body entirely
    pub replace_body: bool,
    /// Replace tail entirely
    pub replace_tail: bool,
    /// Regenerate tail using synthesis
    pub regenerate_tail: bool,
    /// Add sub reinforcement
    pub sub_reinforce: f32,
    /// Transient replacement amount (0-1)
    pub transient_amount: f32,
    /// Body replacement amount (0-1)
    pub body_amount: f32,
    /// Spectral blend amount (0=original spec, 1=synth spec)
    pub spectral_blend: f32,
    /// Preserve original transient
    pub preserve_transient: bool,
    /// Preserve original tail
    pub preserve_tail: bool,
    /// Preserve original pitch
    pub preserve_pitch: bool,
    /// Preserve original rhythm
    pub preserve_rhythm: bool,
    /// Preserve original texture
    pub preserve_texture: bool,
}

impl Default for HybridParams {
    fn default() -> Self {
        Self {
            synth_blend: 0.5,
            replace_transient: false,
            replace_body: false,
            replace_tail: false,
            regenerate_tail: false,
            sub_reinforce: 0.0,
            transient_amount: 0.0,
            body_amount: 0.0,
            spectral_blend: 0.0,
            preserve_transient: true,
            preserve_tail: true,
            preserve_pitch: true,
            preserve_rhythm: true,
            preserve_texture: true,
        }
    }
}

/// Hybrid reconstruction: blend original audio with synthesis layers.
/// Returns the hybridized audio buffer.
pub fn hybrid_reconstruct(
    original: &[f32],
    analysis: &AudioAnalysis,
    params: &HybridParams,
) -> Vec<f32> {
    let st = SoundType::from_str(&analysis.sound_type_hint);
    let pitch = analysis.pitch_estimate.unwrap_or_else(|| match st {
        SoundType::Kick | SoundType::Bass => 60.0,
        SoundType::Snare => 200.0,
        SoundType::ClosedHat => 400.0,
        SoundType::OpenHat => 300.0,
        SoundType::Clap => 180.0,
        _ => 200.0,
    });

    let base_params = resynthesize::params_for_sound_type(st, pitch, analysis.duration_ms);
    let synth = resynthesize::resynthesize(&base_params);

    if synth.is_empty() {
        return original.to_vec();
    }

    let max_len = original.len().max(synth.len());
    let mut output = vec![0.0f32; max_len];

    // Detect transient region
    let transient_end = find_transient_end(original);
    let _body_start = transient_end;
    let tail_start = find_tail_start(original, analysis);

    for i in 0..max_len {
        let orig = original.get(i).copied().unwrap_or(0.0);
        let syn = synth.get(i).copied().unwrap_or(0.0);

        let val = if i < transient_end {
            // Transient region
            if params.replace_transient {
                if params.preserve_transient {
                    orig * (1.0 - params.transient_amount) + syn * params.transient_amount
                } else {
                    syn
                }
            } else {
                orig * (1.0 - params.synth_blend) + syn * params.synth_blend
            }
        } else if i < tail_start {
            // Body region
            if params.replace_body {
                if params.preserve_texture {
                    orig * (1.0 - params.body_amount) + syn * params.body_amount
                } else {
                    syn
                }
            } else {
                orig * (1.0 - params.synth_blend) + syn * params.synth_blend
            }
        } else {
            // Tail region
            if params.replace_tail || params.regenerate_tail {
                if params.preserve_tail {
                    orig * (1.0 - params.synth_blend) + syn * params.synth_blend
                } else {
                    syn
                }
            } else {
                orig
            }
        };

        output[i] = val;
    }

    // Sub reinforcement
    if params.sub_reinforce > 0.0 {
        for i in 0..output.len() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = (-3.0 * t).exp();
            let sub = (2.0 * PI * 55.0 * t).sin() * env * params.sub_reinforce;
            output[i] = (output[i] + sub).clamp(-1.0, 1.0);
        }
    }

    // Spectral blending
    if params.spectral_blend > 0.01 {
        let dry = original.to_vec();
        let blend_len = dry.len().min(output.len());
        for i in 0..blend_len {
            let dry_signal = &dry[i..];
            let wet_signal = &output[i..];
            let dry_energy: f32 = dry_signal.iter().take(256).map(|&s| s * s).sum::<f32>() / 256.0;
            let wet_energy: f32 = wet_signal.iter().take(256).map(|&s| s * s).sum::<f32>() / 256.0;
            if wet_energy > 0.001 && dry_energy > 0.001 {
                let ratio = (dry_energy / wet_energy).sqrt().min(2.0);
                output[i] *= 1.0 + (ratio - 1.0) * params.spectral_blend;
            }
        }
    }

    // Normalize
    let peak = output.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak > 1.0 {
        let gain = 0.95 / peak;
        for s in output.iter_mut() { *s *= gain; }
    }

    output
}

/// Find the end of the transient region (first few milliseconds).
fn find_transient_end(samples: &[f32]) -> usize {
    let max_transient_len = (SAMPLE_RATE as f32 * 0.015) as usize;
    let peak_idx = samples.iter()
        .enumerate()
        .map(|(i, &s)| (i, s.abs()))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let end = peak_idx + max_transient_len;
    end.min(samples.len())
}

/// Find where the tail begins (toward the end of the sound).
fn find_tail_start(samples: &[f32], analysis: &AudioAnalysis) -> usize {
    if analysis.duration_ms < 200.0 {
        return samples.len();
    }
    let tail_fraction = 0.3;
    let tail_pos = (samples.len() as f32 * (1.0 - tail_fraction)) as usize;
    tail_pos.min(samples.len())
}

/// Extract transient from original and layer onto synthesis.
pub fn layer_transient(original: &[f32], synth: &mut [f32], blend: f32) {
    let trans_len = (SAMPLE_RATE as f32 * 0.01) as usize;
    let len = trans_len.min(original.len()).min(synth.len());
    for i in 0..len {
        synth[i] = synth[i] * (1.0 - blend) + original[i] * blend;
    }
}

/// Blend the spectral profile of original into synthesis.
pub fn spectral_blend(original: &[f32], synth: &mut [f32], amount: f32) {
    if amount <= 0.0 { return; }
    let len = original.len().min(synth.len());
    let block = 256;
    let mut i = 0;
    while i + block <= len {
        let orig_rms: f32 = original[i..i+block].iter().map(|&s| s * s).sum::<f32>() / block as f32;
        let synth_rms: f32 = synth[i..i+block].iter().map(|&s| s * s).sum::<f32>() / block as f32;
        if synth_rms > 0.001 && orig_rms > 0.001 {
            let gain = (orig_rms / synth_rms).sqrt().min(3.0);
            for j in i..i+block {
                synth[j] *= 1.0 + (gain - 1.0) * amount;
            }
        }
        i += block;
    }
}
