use super::SAMPLE_RATE;
use super::analyze::{analyze_audio, AudioAnalysis};
use super::dsp;
use super::resynthesize;
use super::recreate::{params_from_analysis, compute_similarity, SimilarityReport};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MorphControls {
    pub amount: f32,
    pub preserve_source_identity: f32,
    pub exaggerate: f32,
    pub preserve_transient: f32,
    pub preserve_body: f32,
    pub preserve_tail: f32,
    pub transient_transfer: f32,
    pub tail_transfer: f32,
    pub tonal_blend: f32,
    pub texture_blend: f32,
}

impl Default for MorphControls {
    fn default() -> Self {
        Self {
            amount: 0.5,
            preserve_source_identity: 0.5,
            exaggerate: 0.0,
            preserve_transient: 1.0,
            preserve_body: 0.5,
            preserve_tail: 0.5,
            transient_transfer: 0.5,
            tail_transfer: 0.5,
            tonal_blend: 0.5,
            texture_blend: 0.5,
        }
    }
}

fn linear_crossfade(a: &[f32], b: &[f32], t: f32) -> Vec<f32> {
    let max_len = a.len().max(b.len());
    let mut out = Vec::with_capacity(max_len);
    for i in 0..max_len {
        let av = a.get(i).copied().unwrap_or(0.0);
        let bv = b.get(i).copied().unwrap_or(0.0);
        out.push(av * (1.0 - t) + bv * t);
    }
    out
}

fn power_crossfade(a: &[f32], b: &[f32], t: f32) -> Vec<f32> {
    let max_len = a.len().max(b.len());
    let mut out = Vec::with_capacity(max_len);
    let gain_a = (1.0 - t).sqrt();
    let gain_b = t.sqrt();
    for i in 0..max_len {
        let av = a.get(i).copied().unwrap_or(0.0);
        let bv = b.get(i).copied().unwrap_or(0.0);
        out.push(av * gain_a + bv * gain_b);
    }
    out
}

fn align_lengths(samples: &[f32], target_len: usize) -> Vec<f32> {
    if samples.len() == target_len {
        return samples.to_vec();
    }
    if samples.len() < target_len {
        let mut padded = samples.to_vec();
        padded.resize(target_len, 0.0);
        return padded;
    }
    let ratio = target_len as f32 / samples.len() as f32;
    let mut out = Vec::with_capacity(target_len);
    let mut pos = 0.0;
    for _ in 0..target_len {
        let idx = pos as usize;
        out.push(samples.get(idx).copied().unwrap_or(0.0));
        pos += 1.0 / ratio;
    }
    out
}

fn find_transient_onset(samples: &[f32]) -> usize {
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return 0; }
    let threshold = peak * 0.2;
    for i in 10..samples.len().min(SAMPLE_RATE as usize / 2) {
        if samples[i].abs() > threshold {
            return i;
        }
    }
    0
}

fn find_tail_start(samples: &[f32], onset: usize) -> usize {
    if samples.len() < 256 { return samples.len() / 2; }
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return samples.len() / 2; }
    let decay_target = peak * 0.1;
    for i in onset..samples.len() {
        if samples[i].abs() <= decay_target {
            return i;
        }
    }
    samples.len() * 3 / 4
}

fn extract_transient(samples: &[f32], onset: usize) -> Vec<f32> {
    let onset_len = (SAMPLE_RATE as f32 * 0.015) as usize;
    let mut isolated = vec![0.0f32; samples.len()];
    let end = (onset + onset_len).min(samples.len());
    for i in onset..end {
        let t = (i - onset) as f32 / (end - onset).max(1) as f32;
        let env = ((1.0 - t * t) * (1.0 - t * 0.5)).max(0.0);
        isolated[i] = samples[i] * env;
    }
    isolated
}

fn extract_body(samples: &[f32], onset: usize, tail_start: usize) -> Vec<f32> {
    let mut isolated = vec![0.0f32; samples.len()];
    let body_start = (onset + (SAMPLE_RATE as f32 * 0.015) as usize).min(samples.len());
    let body_end = tail_start.min(samples.len());
    if body_start >= body_end { return isolated; }
    for i in body_start..body_end {
        isolated[i] = samples[i];
    }
    isolated
}

fn extract_tail(samples: &[f32], tail_start: usize) -> Vec<f32> {
    let mut isolated = vec![0.0f32; samples.len()];
    if tail_start >= samples.len() { return isolated; }
    for i in tail_start..samples.len() {
        isolated[i] = samples[i];
    }
    isolated
}

fn extract_noise_portion(samples: &[f32]) -> Vec<f32> {
    let mut noise_est = vec![0.0f32; samples.len()];
    let zcr = super::analyze::compute_zero_crossing_rate(samples);
    if zcr < 0.05 { return noise_est; }
    for i in 0..samples.len() {
        let smoothed = if i > 0 { samples[i] - samples[i - 1] } else { samples[i] };
        noise_est[i] = smoothed.abs() * 0.5;
    }
    noise_est
}

fn extract_tonal_portion(samples: &[f32], _onset: usize) -> Vec<f32> {
    let mut tonal = samples.to_vec();
    let zcr = super::analyze::compute_zero_crossing_rate(samples);
    if zcr < 0.03 { return tonal; }
    let cutoff = 2000.0;
    dsp::low_pass(&mut tonal, cutoff);
    tonal
}

pub fn morph_samples(
    source: &[f32],
    target: &[f32],
    controls: &MorphControls,
) -> (Vec<f32>, SimilarityReport) {
    let amount = controls.amount.clamp(0.0, 1.0);
    if source.is_empty() { return (target.to_vec(), SimilarityReport::default()); }
    if target.is_empty() { return (source.to_vec(), SimilarityReport::default()); }
    if amount <= 0.0 { return (source.to_vec(), SimilarityReport::default()); }
    if amount >= 1.0 { return (target.to_vec(), SimilarityReport::default()); }

    let max_len = source.len().max(target.len());
    let src = align_lengths(source, max_len);
    let tgt = align_lengths(target, max_len);

    let source_onset = find_transient_onset(&src);
    let target_onset = find_transient_onset(&tgt);
    let source_tail_start = find_tail_start(&src, source_onset);
    let target_tail_start = find_tail_start(&tgt, target_onset);

    let morph_amt = amount * (1.0 + controls.exaggerate * 0.5).min(2.0);
    let morph_amt = morph_amt.clamp(0.0, 1.0);

    // Base crossfade
    let mut result = linear_crossfade(&src, &tgt, morph_amt);

    // Transient transfer: take target's transient and blend with source's body
    if controls.transient_transfer > 0.01 {
        let src_transient = extract_transient(&src, source_onset);
        let tgt_transient = extract_transient(&tgt, target_onset);
        let tt = controls.transient_transfer * morph_amt;

        let preserve_t = controls.preserve_transient.clamp(0.0, 1.0);
        for i in 0..max_len {
            let src_t = src_transient[i];
            let tgt_t = tgt_transient[i];
            let transient_blend = src_t * (1.0 - tt) * preserve_t + tgt_t * tt;
            let body_mask = if i < source_tail_start.max(target_tail_start) { 1.0 } else { 0.0 };
            result[i] = result[i] * (1.0 - body_mask * tt * 0.3) + transient_blend * body_mask * tt * 0.7;
        }
    }

    // Tail transfer
    if controls.tail_transfer > 0.01 {
        let src_tail = extract_tail(&src, source_tail_start);
        let tgt_tail = extract_tail(&tgt, target_tail_start);
        let tl = controls.tail_transfer * morph_amt;
        let preserve_tl = controls.preserve_tail.clamp(0.0, 1.0);

        for i in 0..max_len {
            let tail_mask = if i >= source_tail_start.min(target_tail_start) { 1.0 } else { 0.0 };
            let src_tl = src_tail[i];
            let tgt_tl = tgt_tail[i];
            if tail_mask > 0.5 {
                let tail_blend = src_tl * (1.0 - tl) * preserve_tl + tgt_tl * tl;
                result[i] = result[i] * (1.0 - tl * 0.5) + tail_blend * tl * 0.5;
            }
        }
    }

    // Tonal blend
    if controls.tonal_blend > 0.01 {
        let src_tonal = extract_tonal_portion(&src, source_onset);
        let tgt_tonal = extract_tonal_portion(&tgt, target_onset);
        let tb = controls.tonal_blend * morph_amt;
        for i in 0..max_len {
            let tonal_blend = src_tonal[i] * (1.0 - tb) + tgt_tonal[i] * tb;
            result[i] = result[i] * 0.6 + tonal_blend * 0.4;
        }
    }

    // Texture blend (noise content)
    if controls.texture_blend > 0.01 {
        let src_noise = extract_noise_portion(&src);
        let tgt_noise = extract_noise_portion(&tgt);
        let tx = controls.texture_blend * morph_amt;
        for i in 0..max_len {
            let noise_blend = src_noise[i] * (1.0 - tx) + tgt_noise[i] * tx;
            result[i] += noise_blend * tx * 0.1;
        }
    }

    // Preserve source identity: pull result back toward source
    let identity = controls.preserve_source_identity.clamp(0.0, 1.0);
    if identity > 0.01 {
        let pull = identity * 0.3;
        for i in 0..max_len {
            let src_v = src.get(i).copied().unwrap_or(0.0);
            result[i] = result[i] * (1.0 - pull) + src_v * pull;
        }
    }

    // Body preservation
    if controls.preserve_body > 0.01 {
        let src_body = extract_body(&src, source_onset, source_tail_start);
        let pb = controls.preserve_body * morph_amt * 0.3;
        for i in 0..max_len {
            if src_body[i].abs() > 0.001 {
                result[i] = result[i] * (1.0 - pb) + src_body[i] * pb;
            }
        }
    }

    // Normalize
    let peak = result.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak > 0.99 {
        let gain = 0.95 / peak;
        for s in result.iter_mut() { *s *= gain; }
    }

    let analysis = analyze_audio(source, SAMPLE_RATE, 1);
    let report = compute_similarity(source, &result, &analysis);
    (result, report)
}

pub fn vector_morph(
    samples_a: &[f32],
    analysis_a: &AudioAnalysis,
    samples_b: &[f32],
    analysis_b: &AudioAnalysis,
    controls: &MorphControls,
) -> (Vec<f32>, SimilarityReport) {
    let params_a = params_from_analysis(analysis_a, samples_a);
    let params_b = params_from_analysis(analysis_b, samples_b);
    let amt = controls.amount.clamp(0.0, 1.0);

    let mut morph_params = super::midi::morph_params(&params_a, &params_b, amt);

    let preserve_b = controls.preserve_body.clamp(0.0, 1.0);
    let preserve_t = controls.preserve_transient.clamp(0.0, 1.0);
    let preserve_tl = controls.preserve_tail.clamp(0.0, 1.0);

    if preserve_b > 0.5 {
        morph_params.body_gain = params_a.body_gain * (1.0 - amt * (1.0 - preserve_b))
            + morph_params.body_gain * amt * (1.0 - preserve_b);
    }
    if preserve_t > 0.5 {
        morph_params.click_amount = params_a.click_amount * (1.0 - amt * (1.0 - preserve_t))
            + morph_params.click_amount * amt * (1.0 - preserve_t);
        morph_params.attack_ms = params_a.attack_ms * (1.0 - amt * 0.3 * (1.0 - preserve_t))
            + morph_params.attack_ms * amt * 0.3 * (1.0 - preserve_t);
    }
    if preserve_tl > 0.5 {
        morph_params.tail_ms = params_a.tail_ms * (1.0 - amt * (1.0 - preserve_tl))
            + morph_params.tail_ms * amt * (1.0 - preserve_tl);
    }

    if controls.exaggerate > 0.01 {
        let ex = controls.exaggerate.clamp(0.0, 1.0);
        morph_params.saturation_drive = (morph_params.saturation_drive + ex * 0.5).max(1.0);
        morph_params.brightness = (morph_params.brightness + (params_b.brightness - params_a.brightness) * ex * 0.5).clamp(0.0, 1.0);
    }

    let recreated = resynthesize::resynthesize(&morph_params);
    let aligned = align_lengths(&recreated, samples_a.len().max(samples_b.len()));
    let report = compute_similarity(samples_a, &aligned, analysis_a);
    (aligned, report)
}

pub fn morph(
    source: &[f32],
    target: &[f32],
    controls: &MorphControls,
) -> (Vec<f32>, SimilarityReport) {
    morph_samples(source, target, controls)
}
