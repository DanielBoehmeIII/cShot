use std::f32::consts::PI;
use super::{SoundType, SAMPLE_RATE};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioAnalysis {
    pub duration_ms: f32,
    pub sample_rate: u32,
    pub channels: u16,
    pub peak: f32,
    pub rms: f32,
    pub crest_factor: f32,
    pub loudness_lufs: f32,
    pub noise_floor_db: f32,
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub tail_ms: f32,
    pub envelope: Vec<f32>,
    pub has_leading_silence: bool,
    pub has_trailing_silence: bool,
    pub leading_silence_ms: f32,
    pub trailing_silence_ms: f32,
    pub spectral_centroid: f32,
    pub spectral_rolloff: f32,
    pub brightness: f32,
    pub zero_crossing_rate: f32,
    pub sub_energy_ratio: f32,
    pub noise_estimate: f32,
    pub transient_strength: f32,
    pub transient_count: usize,
    pub onset_times_ms: Vec<f32>,
    pub pitch_estimate: Option<f32>,
    pub has_pitch: bool,
    pub has_clipping: bool,
    pub clipping_count: usize,
    pub is_silent: bool,
    pub spectral_profile: Vec<f32>,
    pub sound_type_hint: String,
    // New transient analysis fields
    pub transient_sharpness: f32,
    pub transient_spectral_centroid: f32,
    pub transient_density: f32,
    pub attack_sharpness_perception: f32,
    pub perceived_impact: f32,
    pub transient_spectral_distribution: Vec<f32>,
    pub click_character_hint: String,
}

fn compute_sound_type_hint(
    is_silent: bool,
    pitch_estimate: Option<f32>,
    attack_ms: f32,
    decay_ms: f32,
    duration_ms: f32,
    brightness: f32,
    noise_estimate: f32,
    zero_crossing_rate: f32,
    spectral_centroid: f32,
    transient_count: usize,
) -> String {
    if is_silent { return "silence".to_string(); }
    if pitch_estimate.is_some() && pitch_estimate.unwrap() < 200.0
        && attack_ms < 5.0 && decay_ms < 300.0 && brightness < 0.3 {
        return "kick".to_string();
    }
    if transient_count > 3 && transient_count < 10
        && noise_estimate > 0.6 && attack_ms < 10.0 {
        return "clap".to_string();
    }
    if noise_estimate > 0.7 && zero_crossing_rate > 0.15
        && spectral_centroid > 4000.0 && duration_ms < 500.0 {
        if duration_ms < 300.0 { return "closed_hat".to_string(); }
        return "open_hat".to_string();
    }
    if noise_estimate > 0.4 && zero_crossing_rate > 0.08
        && attack_ms < 8.0 && duration_ms < 600.0 {
        return "snare".to_string();
    }
    if pitch_estimate.is_some() && pitch_estimate.unwrap() < 150.0
        && duration_ms > 300.0 && brightness < 0.4 {
        return "bass".to_string();
    }
    if duration_ms > 1500.0 && transient_count <= 1
        && spectral_centroid > 500.0 {
        return "fx".to_string();
    }
    if attack_ms < 5.0 && duration_ms < 400.0 {
        return "perc".to_string();
    }
    "other".to_string()
}

impl AudioAnalysis {
    pub fn sound_type_hint(&self) -> &str {
        &self.sound_type_hint
    }
}

pub fn analyze_audio(samples: &[f32], sample_rate: u32, channels: u16) -> AudioAnalysis {
    let duration_ms = if sample_rate > 0 {
        samples.len() as f32 / sample_rate as f32 * 1000.0
    } else { 0.0 };

    let peak = compute_peak(samples);
    let rms = compute_rms(samples);
    let crest_factor = if rms > 0.0 { peak / rms } else { 1.0 };
    let zero_crossing_rate = compute_zero_crossing_rate(samples);
    let spectral_centroid = compute_spectral_centroid(samples);
    let sub_energy_ratio = compute_energy_sub_low(samples);
    let attack_ms = compute_attack_time(samples);
    let has_clipping = samples.iter().any(|&s| s.abs() >= 1.0);
    let clipping_count = samples.iter().filter(|&&s| s.abs() >= 1.0).count();
    let is_silent = peak < 0.001 || rms < 0.0001;
    let loudness_lufs = compute_integrated_loudness(samples);
    let noise_floor_db = estimate_noise_floor(samples);
    let noise_estimate = estimate_noise_ratio(samples);
    let brightness = compute_brightness(samples);
    let spectral_rolloff = compute_spectral_rolloff(samples, 0.85);
    let envelope = extract_envelope(samples, 256);
    let (decay_ms, tail_ms) = compute_decay_and_tail(samples, sample_rate, attack_ms);
    let (transient_strength, transient_count, onset_times_ms) = detect_transients(samples, sample_rate);
    let pitch_estimate = estimate_pitch(samples, sample_rate);
    let has_pitch = pitch_estimate.is_some_and(|p| p > 20.0 && p < 8000.0);
    let spectral_profile = compute_spectral_profile(samples, 64);
    let (leading_silence_ms, trailing_silence_ms) = detect_silence_regions(samples, sample_rate);
    let has_leading_silence = leading_silence_ms > 1.0;
    let has_trailing_silence = trailing_silence_ms > 1.0;

    let transient_sharpness = compute_transient_sharpness_local(samples);
    let (transient_spectral_centroid, transient_spectral_distribution) = compute_transient_spectral_profile(samples);
    let transient_density = compute_transient_density(samples);
    let attack_sharpness_perception = compute_attack_sharpness_perception(samples, attack_ms);
    let perceived_impact = compute_perceived_impact(crest_factor, transient_strength, attack_ms, rms);
    let click_character_hint = classify_click_character(samples, spectral_centroid, transient_sharpness, attack_ms, zero_crossing_rate);

    AudioAnalysis {
        duration_ms,
        sample_rate,
        channels,
        peak,
        rms,
        crest_factor,
        loudness_lufs,
        noise_floor_db,
        attack_ms,
        decay_ms,
        tail_ms,
        envelope,
        has_leading_silence,
        has_trailing_silence,
        leading_silence_ms,
        trailing_silence_ms,
        spectral_centroid,
        spectral_rolloff,
        brightness,
        zero_crossing_rate,
        sub_energy_ratio,
        noise_estimate,
        transient_strength,
        transient_count,
        onset_times_ms,
        pitch_estimate,
        has_pitch,
        has_clipping,
        clipping_count,
        is_silent,
        spectral_profile,
        sound_type_hint: compute_sound_type_hint(
            is_silent, pitch_estimate, attack_ms, decay_ms,
            duration_ms, brightness, noise_estimate,
            zero_crossing_rate, spectral_centroid, transient_count,
        ),
        transient_sharpness,
        transient_spectral_centroid,
        transient_density,
        attack_sharpness_perception,
        perceived_impact,
        transient_spectral_distribution,
        click_character_hint,
    }
}

pub fn compute_rms(samples: &[f32]) -> f32 {
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

pub fn compute_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max)
}

pub fn compute_crest_factor(samples: &[f32]) -> f32 {
    let peak = compute_peak(samples);
    let rms = compute_rms(samples);
    if rms > 0.0 { peak / rms } else { 1.0 }
}

pub fn compute_integrated_loudness(samples: &[f32]) -> f32 {
    let n = samples.len();
    if n < 4800 { return -60.0; }
    let block_size = 4800;
    let mut block_powers = Vec::new();
    let mut i = 0;
    while i + block_size <= n {
        let block: f32 = samples[i..i + block_size].iter().map(|&s| s * s).sum::<f32>() / block_size as f32;
        block_powers.push(block);
        i += block_size;
    }
    if block_powers.is_empty() { return -60.0; }
    let weighted_sum: f32 = block_powers.iter().sum();
    let mean_power = weighted_sum / block_powers.len() as f32;
    if mean_power <= 0.0 { return -60.0; }
    let lufs = -0.691 + 10.0 * mean_power.log10();
    lufs.max(-60.0)
}

pub fn estimate_noise_floor(samples: &[f32]) -> f32 {
    if samples.len() < 256 { return -90.0; }
    let mut sorted: Vec<f32> = samples.iter().map(|&s| s.abs()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let percentile_idx = (sorted.len() as f32 * 0.05) as usize;
    let floor = sorted.get(percentile_idx).copied().unwrap_or(0.0);
    if floor <= 0.0 { return -90.0; }
    20.0 * floor.log10()
}

fn estimate_noise_ratio(samples: &[f32]) -> f32 {
    if samples.len() < 256 { return 0.5; }
    let centroid = compute_spectral_centroid(samples);
    let zcr = compute_zero_crossing_rate(samples);
    let zcr_noise = (zcr / 0.5).min(1.0);
    let centroid_noise = (centroid / 8000.0).min(1.0);
    let crest = compute_crest_factor(samples);
    let crest_noise = (1.0 - (crest / 20.0).min(1.0)).max(0.0);
    let spectral_flatness = compute_spectral_flatness(samples);
    zcr_noise * 0.3 + centroid_noise * 0.3 + crest_noise * 0.2 + spectral_flatness * 0.2
}

pub fn compute_spectral_flatness(samples: &[f32]) -> f32 {
    let n = samples.len().min(2048);
    if n < 4 { return 0.5; }
    let mut geometric_sum = 0.0f32;
    let mut arithmetic_sum = 0.0f32;
    let mut count = 0;
    for i in 1..n {
        let mag = samples[i].abs().max(1e-10);
        geometric_sum += mag.ln();
        arithmetic_sum += mag;
        count += 1;
    }
    if count == 0 || arithmetic_sum <= 0.0 { return 0.5; }
    let geom_mean = (geometric_sum / count as f32).exp();
    let arith_mean = arithmetic_sum / count as f32;
    (geom_mean / arith_mean).min(1.0)
}

pub fn compute_spectral_centroid(samples: &[f32]) -> f32 {
    let n = samples.len();
    if n < 2 { return 0.0; }
    let mut magnitude_sum = 0.0f32;
    let mut weighted_sum = 0.0f32;
    for i in 0..n.min(2048) {
        let freq = i as f32 * SAMPLE_RATE as f32 / n as f32;
        let mag = samples[i].abs();
        magnitude_sum += mag;
        weighted_sum += freq * mag;
    }
    if magnitude_sum > 0.0 { weighted_sum / magnitude_sum } else { 0.0 }
}

pub fn compute_spectral_rolloff(samples: &[f32], percentile: f32) -> f32 {
    let n = samples.len().min(4096);
    if n < 4 { return 0.0; }
    let total_energy: f32 = samples.iter().take(n).map(|&s| s * s).sum();
    if total_energy <= 0.0 { return 0.0; }
    let threshold = total_energy * percentile;
    let mut cumulative = 0.0f32;
    for i in 0..n {
        cumulative += samples[i] * samples[i];
        if cumulative >= threshold {
            return i as f32 * SAMPLE_RATE as f32 / n as f32;
        }
    }
    SAMPLE_RATE as f32 / 2.0
}

pub fn compute_brightness(samples: &[f32]) -> f32 {
    let n = samples.len().min(4096);
    if n < 4 { return 0.0; }
    let total_energy: f32 = samples.iter().take(n).map(|&s| s * s).sum();
    if total_energy <= 0.0 { return 0.0; }
    let cutoff_bin = (2000.0 * n as f32 / SAMPLE_RATE as f32) as usize;
    if cutoff_bin >= n { return 0.0; }
    let high_energy: f32 = samples[cutoff_bin..n].iter().map(|&s| s * s).sum();
    high_energy / total_energy
}

pub fn compute_energy_sub_low(samples: &[f32]) -> f32 {
    let total: f32 = samples.iter().map(|&s| s * s).sum();
    if total == 0.0 { return 0.0; }
    let mut sub = 0.0f32;
    let n = samples.len().min(2048);
    for i in 0..n {
        let freq = i as f32 * SAMPLE_RATE as f32 / n as f32;
        if freq < 100.0 {
            sub += samples[i] * samples[i];
        }
    }
    sub / total
}

pub fn compute_zero_crossing_rate(samples: &[f32]) -> f32 {
    if samples.len() < 2 { return 0.0; }
    let mut crossings = 0;
    for i in 1..samples.len() {
        if (samples[i] >= 0.0 && samples[i - 1] < 0.0) || (samples[i] < 0.0 && samples[i - 1] >= 0.0) { crossings += 1; }
    }
    crossings as f32 / samples.len() as f32
}

pub fn compute_attack_time(samples: &[f32]) -> f32 {
    let n = samples.len();
    if n < 100 { return 0.0; }
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return 0.0; }
    let threshold_10 = peak * 0.1;
    let threshold_90 = peak * 0.9;
    let mut t10 = 0;
    let mut t90 = 0;
    for i in 0..n {
        if samples[i].abs() >= threshold_10 && t10 == 0 { t10 = i; }
        if samples[i].abs() >= threshold_90 { t90 = i; break; }
    }
    if t90 > t10 && t10 > 0 {
        (t90 - t10) as f32 / SAMPLE_RATE as f32 * 1000.0
    } else {
        let threshold_50 = peak * 0.5;
        for i in 0..n {
            if samples[i].abs() >= threshold_50 {
                return i as f32 / SAMPLE_RATE as f32 * 1000.0;
            }
        }
        0.0
    }
}

pub fn compute_decay_and_tail(samples: &[f32], sample_rate: u32, _attack_ms: f32) -> (f32, f32) {
    if samples.len() < 100 || sample_rate == 0 {
        return (0.0, 0.0);
    }
    let peak_idx = samples.iter()
        .enumerate()
        .map(|(i, &s)| (i, s.abs()))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let peak_val = samples[peak_idx].abs();
    if peak_val < 0.001 { return (0.0, 0.0); }
    
    let env = extract_envelope(samples, 256);
    let env_peak = env.iter().copied().fold(0.0f32, f32::max);
    if env_peak < 0.001 { return (0.0, 0.0); }
    
    let decay_env_target = env_peak * 0.1;
    let tail_env_target = env_peak * 0.01;
    
    let env_peak_idx = env.iter()
        .position(|&v| v >= env_peak * 0.95)
        .unwrap_or(0);
    
    let mut decay_env_end = env_peak_idx;
    for i in env_peak_idx..env.len() {
        if env[i] <= decay_env_target {
            decay_env_end = i;
            break;
        }
        decay_env_end = i;
    }
    
    let mut tail_env_end = decay_env_end;
    for i in decay_env_end..env.len() {
        if env[i] <= tail_env_target {
            tail_env_end = i;
            break;
        }
        tail_env_end = i;
    }
    
    let total_ms = samples.len() as f32 / sample_rate as f32 * 1000.0;
    let decay_ms = (decay_env_end as f32 / env.len() as f32) * total_ms;
    let tail_ms = ((tail_env_end - decay_env_end) as f32 / env.len() as f32) * total_ms;
    
    (decay_ms, tail_ms)
}

pub fn extract_envelope(samples: &[f32], num_points: usize) -> Vec<f32> {
    if samples.is_empty() || num_points == 0 { return vec![]; }
    let points = num_points.min(samples.len());
    let mut envelope = Vec::with_capacity(points);
    let step = (samples.len() / points).max(1);
    for i in 0..points {
        let start = i * step;
        let end = ((i + 1) * step).min(samples.len());
        let max_amp = samples[start..end].iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
        envelope.push(max_amp);
    }
    envelope
}

pub fn detect_transients(samples: &[f32], sample_rate: u32) -> (f32, usize, Vec<f32>) {
    if samples.len() < 512 || sample_rate == 0 {
        return (0.0, 0, vec![]);
    }
    let frame_size = 512;
    let hop_size = 256;
    let mut spectral_flux = Vec::new();
    let mut prev_spectrum = vec![0.0f32; frame_size / 2];
    let mut i = 0;
    while i + frame_size <= samples.len() {
        let frame = &samples[i..i + frame_size];
        let mut spectrum = vec![0.0f32; frame_size / 2];
        for k in 0..frame_size / 2 {
            let mut re = 0.0f32;
            let mut im = 0.0f32;
            for n in 0..frame_size {
                let angle = -2.0 * PI * k as f32 * n as f32 / frame_size as f32;
                re += frame[n] * angle.cos();
                im += frame[n] * angle.sin();
            }
            spectrum[k] = (re * re + im * im).sqrt();
        }
        let mut flux = 0.0f32;
        for k in 0..frame_size / 2 {
            let diff = spectrum[k] - prev_spectrum[k];
            if diff > 0.0 { flux += diff; }
        }
        spectral_flux.push(flux);
        prev_spectrum = spectrum;
        i += hop_size;
    }
    if spectral_flux.is_empty() { return (0.0, 0, vec![]); }
    let max_flux = spectral_flux.iter().copied().fold(0.0f32, f32::max);
    let mean_flux = spectral_flux.iter().sum::<f32>() / spectral_flux.len() as f32;
    let transient_strength = if mean_flux > 0.0 { max_flux / mean_flux } else { 1.0 };
    let threshold = mean_flux * 3.0;
    let mut onset_times = Vec::new();
    let mut in_onset = false;
    for (j, &flux) in spectral_flux.iter().enumerate() {
        if flux > threshold && !in_onset {
            onset_times.push(j as f32 * hop_size as f32 / sample_rate as f32 * 1000.0);
            in_onset = true;
        } else if flux <= threshold * 0.5 {
            in_onset = false;
        }
    }
    let transient_count = onset_times.len();
    (transient_strength, transient_count, onset_times)
}

pub fn estimate_pitch(samples: &[f32], sample_rate: u32) -> Option<f32> {
    if samples.len() < 200 || sample_rate == 0 { return None; }
    let mut prev = 0.0;
    let mut filtered = Vec::with_capacity(samples.len());
    for &s in samples {
        let f = s - prev + 0.9997 * prev;
        prev = s;
        filtered.push(f);
    }
    let min_lag = (sample_rate as f32 / 2000.0) as usize;
    let max_lag = (sample_rate as f32 / 40.0) as usize;
    if max_lag >= filtered.len() || min_lag >= max_lag { return None; }
    let mut best_lag = min_lag;
    let mut best_corr = 0.0f32;
    for lag in min_lag..=max_lag.min(filtered.len() - 1) {
        let n = filtered.len() - lag;
        let mut corr = 0.0f32;
        let mut energy = 0.0f32;
        for i in 0..n {
            corr += filtered[i] * filtered[i + lag];
            energy += filtered[i] * filtered[i] + filtered[i + lag] * filtered[i + lag];
        }
        if energy > 0.0 {
            let norm_corr = corr / energy.sqrt();
            if norm_corr > best_corr && norm_corr > 0.1 {
                best_corr = norm_corr;
                best_lag = lag;
            }
        }
    }
    if best_corr < 0.2 { return None; }
    let pitch = sample_rate as f32 / best_lag as f32;
    if pitch < 20.0 || pitch > 8000.0 { return None; }
    Some(pitch)
}

pub fn compute_spectral_profile(samples: &[f32], num_bins: usize) -> Vec<f32> {
    if samples.is_empty() || num_bins == 0 { return vec![]; }
    let profile = vec![0.0f32; num_bins];
    let fft_size = 2048;
    if samples.len() < fft_size { return profile; }
    let mut avg_spectrum = vec![0.0f32; fft_size / 2];
    let mut num_frames = 0;
    let mut offset = 0;
    while offset + fft_size <= samples.len() {
        let frame = &samples[offset..offset + fft_size];
        for k in 0..fft_size / 2 {
            let mut re = 0.0f32;
            let mut im = 0.0f32;
            for n in 0..fft_size {
                let angle = -2.0 * PI * k as f32 * n as f32 / fft_size as f32;
                re += frame[n] * angle.cos();
                im += frame[n] * angle.sin();
            }
            avg_spectrum[k] += (re * re + im * im).sqrt();
        }
        num_frames += 1;
        offset += fft_size / 2;
    }
    if num_frames == 0 { return profile; }
    for v in avg_spectrum.iter_mut() { *v /= num_frames as f32; }
    let bin_size = avg_spectrum.len() / num_bins;
    let mut result = Vec::with_capacity(num_bins);
    for b in 0..num_bins {
        let start = b * bin_size;
        let end = ((b + 1) * bin_size).min(avg_spectrum.len());
        if start < end {
            let mean: f32 = avg_spectrum[start..end].iter().sum::<f32>() / (end - start) as f32;
            result.push(mean);
        } else {
            result.push(0.0);
        }
    }
    let max_val = result.iter().copied().fold(0.0f32, f32::max);
    if max_val > 0.0 {
        for v in result.iter_mut() { *v /= max_val; }
    }
    result
}

pub fn detect_silence_regions(samples: &[f32], sample_rate: u32) -> (f32, f32) {
    if samples.is_empty() || sample_rate == 0 { return (0.0, 0.0); }
    let threshold = 0.001;
    let leading = samples.iter()
        .position(|&s| s.abs() > threshold)
        .map(|p| p as f32 / sample_rate as f32 * 1000.0)
        .unwrap_or(0.0);
    let trailing = samples.iter()
        .rposition(|&s| s.abs() > threshold)
        .map(|p| (samples.len() - p - 1) as f32 / sample_rate as f32 * 1000.0)
        .unwrap_or(0.0);
    (leading, trailing)
}

// ─── Transient Analysis Extensions ────────────────────────

fn compute_transient_sharpness_local(samples: &[f32]) -> f32 {
    let n = samples.len().min(SAMPLE_RATE as usize / 2);
    if n < 64 { return 0.5; }
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return 0.0; }
    let threshold = peak * 0.05;
    let onset = samples.iter().position(|&s| s.abs() > threshold).unwrap_or(0);
    let window = (SAMPLE_RATE as f32 * 0.008) as usize;
    let end = (onset + window).min(samples.len());
    if end <= onset + 2 { return 0.5; }
    let region: Vec<f32> = samples[onset..end].iter().map(|&s| s.abs()).collect();
    let peak_r = region.iter().copied().fold(0.0f32, f32::max);
    if peak_r < 0.001 { return 0.0; }
    let mut rise_time = 0;
    let rise_thresh = peak_r * 0.3;
    for (j, &v) in region.iter().enumerate() {
        if v >= rise_thresh { rise_time = j; break; }
    }
    if rise_time == 0 { return 1.0; }
    let sharpness = (1.0 - rise_time as f32 / region.len() as f32).clamp(0.0, 1.0);
    let area: f32 = region.iter().sum::<f32>() / region.len() as f32;
    let peak_to_area = if area > 0.001 { (peak_r / area).min(5.0) / 5.0 } else { 0.5 };
    sharpness * 0.6 + peak_to_area * 0.4
}

fn compute_transient_spectral_profile(samples: &[f32]) -> (f32, Vec<f32>) {
    let n = samples.len().min(SAMPLE_RATE as usize / 2);
    if n < 256 { return (0.0, vec![0.0; 8]); }
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return (0.0, vec![0.0; 8]); }
    let threshold = peak * 0.08;
    let onset = samples.iter().position(|&s| s.abs() > threshold).unwrap_or(0);
    let transient_window = (SAMPLE_RATE as f32 * 0.02) as usize;
    let end = (onset + transient_window).min(samples.len());
    if end <= onset + 32 { return (0.0, vec![0.0; 8]); }
    let transient_region = &samples[onset..end];
    let fft_size = 256;
    if transient_region.len() < fft_size { return (0.0, vec![0.0; 8]); }
    let num_bins = 8;
    let mut spectrum = vec![0.0f32; fft_size / 2];
    for k in 0..fft_size / 2 {
        let mut re = 0.0f32;
        let mut im = 0.0f32;
        for n in 0..fft_size.min(transient_region.len()) {
            let angle = -2.0 * PI * k as f32 * n as f32 / fft_size as f32;
            re += transient_region[n] * angle.cos();
            im += transient_region[n] * angle.sin();
        }
        spectrum[k] = (re * re + im * im).sqrt();
    }
    let max_spec = spectrum.iter().copied().fold(0.0f32, f32::max);
    if max_spec > 0.0 {
        for v in spectrum.iter_mut() { *v /= max_spec; }
    }
    let mut weighted_sum = 0.0f32;
    let mut mag_sum = 0.0f32;
    for (i, &v) in spectrum.iter().enumerate() {
        let freq = i as f32 * SAMPLE_RATE as f32 / fft_size as f32;
        weighted_sum += freq * v;
        mag_sum += v;
    }
    let centroid = if mag_sum > 0.0 { weighted_sum / mag_sum } else { 0.0 };
    let bin_size = spectrum.len() / num_bins;
    let mut profile = Vec::with_capacity(num_bins);
    for b in 0..num_bins {
        let start = b * bin_size;
        let end = ((b + 1) * bin_size).min(spectrum.len());
        if start < end {
            let mean: f32 = spectrum[start..end].iter().sum::<f32>() / (end - start) as f32;
            profile.push(mean);
        } else {
            profile.push(0.0);
        }
    }
    (centroid, profile)
}

fn compute_transient_density(samples: &[f32]) -> f32 {
    let n = samples.len().min(SAMPLE_RATE as usize);
    if n < 512 { return 0.0; }
    let frame_size = 512;
    let hop_size = 256;
    let mut energy_vals = Vec::new();
    let mut i = 0;
    while i + frame_size <= n {
        let frame = &samples[i..i + frame_size];
        let energy: f32 = frame.iter().map(|&s| s * s).sum::<f32>() / frame_size as f32;
        energy_vals.push(energy);
        i += hop_size;
    }
    if energy_vals.len() < 4 { return 0.0; }
    let mean_energy: f32 = energy_vals.iter().sum::<f32>() / energy_vals.len() as f32;
    if mean_energy < 1e-10 { return 0.0; }
    let mut above_threshold = 0;
    let threshold = mean_energy * 2.5;
    for &e in &energy_vals {
        if e > threshold { above_threshold += 1; }
    }
    (above_threshold as f32 / energy_vals.len() as f32).clamp(0.0, 1.0)
}

fn compute_attack_sharpness_perception(samples: &[f32], attack_ms: f32) -> f32 {
    if samples.len() < 64 { return 0.5; }
    if attack_ms <= 0.0 { return 1.0; }
    let attack_samples = (attack_ms / 1000.0 * SAMPLE_RATE as f32) as usize;
    let attack_samples = attack_samples.max(2);
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return 0.0; }
    let threshold = peak * 0.1;
    let onset = samples.iter().position(|&s| s.abs() > threshold).unwrap_or(0);
    let end = (onset + attack_samples * 2).min(samples.len());
    if end <= onset + 1 { return 0.5; }
    let attack_region: Vec<f32> = samples[onset..end].iter().map(|&s| s.abs()).collect();
    let peak_a = attack_region.iter().copied().fold(0.0f32, f32::max);
    if peak_a < 0.001 { return 0.0; }
    let mut rise_samples_10_to_90 = 0;
    let mut _rise_samples_50_to_peak = 0;
    let t10 = peak_a * 0.1;
    let t50 = peak_a * 0.5;
    let t90 = peak_a * 0.9;
    let mut found_10 = false;
    let mut found_50 = false;
    for (j, &v) in attack_region.iter().enumerate() {
        if !found_10 && v >= t10 { found_10 = true; }
        if !found_50 && v >= t50 { _rise_samples_50_to_peak = j; found_50 = true; }
        if v >= t90 { rise_samples_10_to_90 = j; break; }
    }
    if rise_samples_10_to_90 < 1 { return 1.0; }
    let attack_slope = peak_a / rise_samples_10_to_90.max(1) as f32;
    let norm_slope = (attack_slope / 0.5).min(1.0);
    let speed = 1.0 - (rise_samples_10_to_90 as f32 / 100.0).min(1.0);
    norm_slope * 0.5 + speed * 0.5
}

fn compute_perceived_impact(crest_factor: f32, transient_strength: f32, attack_ms: f32, rms: f32) -> f32 {
    let crest_norm = (crest_factor / 20.0).min(1.0);
    let transient_norm = (transient_strength / 10.0).min(1.0);
    let attack_norm = if attack_ms > 0.0 {
        (1.0 - (attack_ms / 50.0).min(1.0)).max(0.0)
    } else {
        0.0
    };
    let rms_norm = (rms * 5.0).min(1.0);
    crest_norm * 0.3 + transient_norm * 0.3 + attack_norm * 0.25 + rms_norm * 0.15
}

fn classify_click_character(samples: &[f32], spectral_centroid: f32, sharpness: f32, attack_ms: f32, zcr: f32) -> String {
    if samples.len() < 64 { return "unknown".to_string(); }
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return "none".to_string(); }
    if sharpness > 0.75 && spectral_centroid > 5000.0 && attack_ms < 3.0 {
        if zcr > 0.2 { return "sharp_noise".to_string(); }
        return "sharp_pitched".to_string();
    }
    if sharpness < 0.3 && spectral_centroid < 1000.0 {
        return "round_thud".to_string();
    }
    if spectral_centroid > 6000.0 && zcr > 0.2 && attack_ms < 5.0 {
        return "metallic".to_string();
    }
    if spectral_centroid < 500.0 && attack_ms > 5.0 {
        return "sub_boom".to_string();
    }
    if sharpness > 0.6 && spectral_centroid > 2000.0 && attack_ms < 8.0 {
        if zcr > 0.15 { return "crack".to_string(); }
        return "snap".to_string();
    }
    if sharpness > 0.4 && spectral_centroid > 1000.0 {
        return "hybrid".to_string();
    }
    "general".to_string()
}

fn extract_prompt_keywords(prompt: &str) -> Vec<String> {
    let lower = prompt.to_lowercase();
    let mut keywords = Vec::new();
    let genre_map: Vec<&str> = vec![
        "trap", "house", "techno", "lo-fi", "lofi", "cinematic",
        "ambient", "dubstep", "dnb", "drum and bass", "rock",
        "metal", "jazz", "hip-hop", "hip hop", "rnb", "808",
        "foley", "orchestral", "synth", "electronic", "dance",
        "garage", "grime", "boom bap", "afro", "latin", "reggaeton",
        "drill", "phonk", "footwork", "jersey", "uk-garage",
        "industrial", "synthwave", "vaporwave", "breakbeat",
        "future-bass", "hyperpop", "emo", "trap-metal",
    ];
    for &genre in &genre_map {
        if lower.contains(genre) {
            let clean = genre.replace(" ", "-");
            if !keywords.contains(&clean) {
                keywords.push(clean);
            }
        }
    }
    let style_map = vec![
        "punchy", "dark", "bright", "warm", "soft", "hard",
        "aggressive", "distorted", "clean", "noisy", "deep", "sub",
        "crisp", "shiny", "gentle", "round", "tight", "fat", "dry",
        "wet", "layered", "metallic", "wooden", "organic", "digital",
        "airy", "hollow", "boomy", "click", "crack", "snap",
        "thick", "thin", "smooth", "gritty", "glitchy", "sweep",
    ];
    for &style in &style_map {
        if lower.contains(style) && !keywords.contains(&style.to_string()) {
            keywords.push(style.to_string());
        }
    }
    let mood_map = vec![
        "epic", "dark", "ominous", "happy", "sad", "melodic",
        "aggressive", "calm", "tense", "dreamy", "haunting",
        "uplifting", "mysterious", "heavy", "light", "powerful",
    ];
    for &mood in &mood_map {
        if lower.contains(mood) && !keywords.contains(&mood.to_string()) {
            keywords.push(mood.to_string());
        }
    }
    let production_map = vec![
        "compressed", "saturated", "limited", "reverb", "delayed",
        "phaser", "flanger", "chorus", "sidechain", "filtered",
        "lofi", "glue", "busy", "minimal", "layered", "textured",
    ];
    for &prod in &production_map {
        if lower.contains(prod) && !keywords.contains(&prod.to_string()) {
            keywords.push(prod.to_string());
        }
    }
    keywords
}

pub fn apply_autotags(samples: &[f32], sound_type: &SoundType, variant_name: Option<&str>, prompt: Option<&str>) -> Vec<String> {
    let mut tags = Vec::new();
    let rms = compute_rms(samples);
    let centroid = compute_spectral_centroid(samples);
    let crest = compute_crest_factor(samples);
    let zcr = compute_zero_crossing_rate(samples);
    let sub_energy = compute_energy_sub_low(samples);
    let duration_ms = samples.len() as f32 / SAMPLE_RATE as f32 * 1000.0;
    let attack_ms = compute_attack_time(samples);

    tags.push(sound_type.as_str().to_string());

    if centroid > 4000.0 { tags.push("bright".to_string()); }
    else if centroid > 2500.0 { tags.push("warm".to_string()); }
    if centroid < 800.0 { tags.push("dark".to_string()); }
    if centroid > 5000.0 && zcr > 0.15 { tags.push("metallic".to_string()); }
    if centroid < 500.0 && sub_energy > 0.4 { tags.push("boomy".to_string()); }
    if rms > 0.25 { tags.push("loud".to_string()); }
    if rms < 0.08 { tags.push("quiet".to_string()); }
    if duration_ms < 150.0 { tags.push("short".to_string()); }
    else if duration_ms < 500.0 { tags.push("medium".to_string()); }
    if duration_ms > 1500.0 { tags.push("long".to_string()); }
    if crest > 12.0 && rms > 0.08 { tags.push("punchy".to_string()); }
    if crest < 6.0 { tags.push("compressed".to_string()); }
    if crest >= 6.0 && crest <= 12.0 { tags.push("dynamic".to_string()); }
    if attack_ms > 0.0 && attack_ms < 3.0 { tags.push("sharp-attack".to_string()); }
    else if attack_ms >= 3.0 && attack_ms < 10.0 { tags.push("moderate-attack".to_string()); }
    else if attack_ms >= 10.0 { tags.push("soft-attack".to_string()); }
    if sub_energy > 0.3 { tags.push("sub".to_string()); }
    if sub_energy < 0.05 && centroid > 2000.0 { tags.push("thin".to_string()); }
    if zcr > 0.25 { tags.push("noisy".to_string()); }
    if zcr < 0.02 { tags.push("clean".to_string()); }

    if let Some(vname) = variant_name {
        match vname {
            "reversed" => tags.push("reversed".to_string()),
            "saturated" | "distorted" => { tags.push("distorted".to_string()); tags.push("saturated".to_string()); }
            "shortened" | "trimmed" => tags.push("short".to_string()),
            "repitched" => tags.push("repitched".to_string()),
            "shaped" => tags.push("shaped".to_string()),
            "layered" => tags.push("layered".to_string()),
            "randomized" => tags.push("randomized".to_string()),
            _ => {}
        }
    }

    if let Some(p) = prompt {
        let prompt_tags = extract_prompt_keywords(p);
        for pt in prompt_tags {
            if !tags.contains(&pt) {
                tags.push(pt);
            }
        }
        let recipe_tags = infer_recipe_tags(p, sound_type);
        for rt in recipe_tags {
            if !tags.contains(&rt) {
                tags.push(rt);
            }
        }
    }

    if centroid > 0.0 && centroid < 2000.0 { tags.push("warm-range".to_string()); }
    if centroid > 3000.0 { tags.push("bright-range".to_string()); }
    let has_clipping = samples.iter().any(|&s| s.abs() >= 1.0);
    if has_clipping { tags.push("clipped".to_string()); }
    if crest > 15.0 { tags.push("high-crest".to_string()); }

    let transient_sharpness = compute_transient_sharpness_local(samples);
    let perceived_impact = compute_perceived_impact(crest, compute_transient_density(samples), attack_ms, rms);
    let click_hint = classify_click_character(samples, centroid, transient_sharpness, attack_ms, zcr);

    if transient_sharpness > 0.7 { tags.push("sharp-attack".to_string()); }
    if transient_sharpness < 0.3 { tags.push("soft-attack".to_string()); }
    if perceived_impact > 0.7 { tags.push("high-impact".to_string()); }
    if perceived_impact < 0.3 { tags.push("low-impact".to_string()); }

    match click_hint.as_str() {
        "sharp_noise" => tags.push("click-sharp-noise".to_string()),
        "sharp_pitched" => tags.push("click-sharp-pitched".to_string()),
        "metallic" => tags.push("click-metallic".to_string()),
        "crack" => tags.push("click-crack".to_string()),
        "snap" => tags.push("click-snap".to_string()),
        "round_thud" => tags.push("click-thud".to_string()),
        "sub_boom" => tags.push("click-sub".to_string()),
        "hybrid" => tags.push("click-hybrid".to_string()),
        _ => {}
    }

    tags
}

fn infer_recipe_tags(prompt: &str, sound_type: &SoundType) -> Vec<String> {
    let lower = prompt.to_lowercase();
    let mut tags = Vec::new();

    match sound_type {
        SoundType::Kick => {
            if lower.contains("808") || lower.contains("boomy") || lower.contains("deep") {
                tags.push("808-influence".to_string());
            }
            if lower.contains("click") || lower.contains("tight") || lower.contains("electronic") {
                tags.push("clicky-attack".to_string());
            }
            if lower.contains("distorted") || lower.contains("lo-fi") || lower.contains("gritty") {
                tags.push("processed".to_string());
            }
            if lower.contains("sub") || lower.contains("deep") || lower.contains("heavy") {
                tags.push("sub-heavy".to_string());
            }
            if lower.contains("acoustic") || lower.contains("natural") || lower.contains("rock") || lower.contains("real") {
                tags.push("acoustic-style".to_string());
            }
        }
        SoundType::Snare => {
            if lower.contains("crack") || lower.contains("bright") || lower.contains("metal") {
                tags.push("bright-crack".to_string());
            }
            if lower.contains("trap") || lower.contains("layered") || lower.contains("clap") {
                tags.push("layered-snare".to_string());
            }
            if lower.contains("rim") || lower.contains("rimshot") || lower.contains("wood") {
                tags.push("rim-style".to_string());
            }
            if lower.contains("military") || lower.contains("marching") || lower.contains("march") {
                tags.push("march-style".to_string());
            }
        }
        SoundType::ClosedHat | SoundType::OpenHat => {
            if lower.contains("tight") || lower.contains("short") || lower.contains("closed") {
                tags.push("tight-hat".to_string());
            }
            if lower.contains("wash") || lower.contains("open") || lower.contains("sizzle") {
                tags.push("washy-hat".to_string());
            }
            if lower.contains("acoustic") || lower.contains("jazz") || lower.contains("real") {
                tags.push("acoustic-hat".to_string());
            }
        }
        SoundType::Bass => {
            if lower.contains("sub") || lower.contains("808") || lower.contains("deep") {
                tags.push("sub-bass".to_string());
            }
            if lower.contains("warm") || lower.contains("round") || lower.contains("sine") {
                tags.push("warm-bass".to_string());
            }
            if lower.contains("distorted") || lower.contains("gritty") || lower.contains("saturated") {
                tags.push("distorted-bass".to_string());
            }
        }
        SoundType::Fx => {
            if lower.contains("riser") || lower.contains("build") || lower.contains("sweep") || lower.contains("rise") {
                tags.push("riser".to_string());
            }
            if lower.contains("impact") || lower.contains("hit") || lower.contains("boom") || lower.contains("cinematic") {
                tags.push("impact".to_string());
            }
            if lower.contains("reverse") || lower.contains("swell") || lower.contains("wash") {
                tags.push("swell".to_string());
            }
            if lower.contains("glitch") || lower.contains("stutter") || lower.contains("digital") {
                tags.push("glitch".to_string());
            }
            if lower.contains("orchestral") || lower.contains("cinematic") || lower.contains("dramatic") {
                tags.push("cinematic-fx".to_string());
            }
        }
        SoundType::Perc | SoundType::Tom | SoundType::Clap | SoundType::Other => {
            if lower.contains("shaker") || lower.contains("tambourine") || lower.contains("cowbell") {
                tags.push("hand-perc".to_string());
            }
            if lower.contains("electronic") || lower.contains("synth") || lower.contains("digital") {
                tags.push("electronic-perc".to_string());
            }
            if lower.contains("acoustic") || lower.contains("organic") || lower.contains("natural") {
                tags.push("acoustic-perc".to_string());
            }
        }
    }

    if lower.contains("reverb") || lower.contains("ambient") || lower.contains("space") || lower.contains("wet") {
        if !tags.contains(&"wet".to_string()) { tags.push("wet".to_string()); }
    }
    if lower.contains("dry") && !tags.contains(&"dry".to_string()) {
        tags.push("dry".to_string());
    }
    if lower.contains("layer") || lower.contains("stack") || lower.contains("mult") {
        tags.push("layered".to_string());
    }
    if lower.contains("bpm") || lower.contains("tempo") || lower.matches(|c: char| c.is_ascii_digit()).count() >= 3 {
        tags.push("tempo-specified".to_string());
    }
    tags
}
