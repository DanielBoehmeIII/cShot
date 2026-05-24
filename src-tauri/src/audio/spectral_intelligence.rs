use std::f32::consts::PI;
use super::SAMPLE_RATE;
use super::analyze;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpectralFingerprint {
    pub bands: Vec<f32>,
    pub centroid: f32,
    pub rolloff: f32,
    pub brightness: f32,
    pub sub_energy: f32,
    pub low_mid_energy: f32,
    pub high_mid_energy: f32,
    pub presence_energy: f32,
    pub air_energy: f32,
    pub tonal_ratio: f32,
    pub noise_ratio: f32,
    pub resonance_peaks: Vec<(f32, f32)>,
    pub harshness: f32,
    pub muddiness: f32,
    pub spectral_flatness: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SpectralMatch {
    pub overall: f32,
    pub band_similarity: f32,
    pub centroid_similarity: f32,
    pub brightness_similarity: f32,
    pub tonal_similarity: f32,
    pub resonance_similarity: f32,
}

fn compute_spectrum(samples: &[f32], fft_size: usize) -> Vec<f32> {
    let mut spectrum = vec![0.0f32; fft_size / 2];
    if samples.len() < fft_size { return spectrum; }
    let half = fft_size / 2;
    for k in 0..half {
        let mut re = 0.0f32;
        let mut im = 0.0f32;
        for n in 0..fft_size {
            let angle = -2.0 * PI * k as f32 * n as f32 / fft_size as f32;
            re += samples[n] * angle.cos();
            im += samples[n] * angle.sin();
        }
        spectrum[k] = (re * re + im * im).sqrt();
    }
    spectrum
}

fn average_spectrum(samples: &[f32], fft_size: usize) -> Vec<f32> {
    let half = fft_size / 2;
    let mut avg = vec![0.0f32; half];
    let mut count = 0;
    let step = fft_size / 2;
    let mut offset = 0;
    while offset + fft_size <= samples.len() {
        let frame = &samples[offset..offset + fft_size];
        let spec = compute_spectrum(frame, fft_size);
        for i in 0..half {
            avg[i] += spec[i];
        }
        count += 1;
        offset += step;
    }
    if count > 0 {
        for v in avg.iter_mut() { *v /= count as f32; }
    }
    avg
}

fn freq_to_bin(freq: f32, fft_size: usize) -> usize {
    ((freq / SAMPLE_RATE as f32) * fft_size as f32) as usize
}

fn bin_to_freq(bin: usize, fft_size: usize) -> f32 {
    bin as f32 * SAMPLE_RATE as f32 / fft_size as f32
}

pub fn extract_fingerprint(samples: &[f32]) -> SpectralFingerprint {
    let fft_size = 2048;
    let half = fft_size / 2;
    let avg_spec = average_spectrum(samples, fft_size);
    if avg_spec.is_empty() || avg_spec.iter().all(|&v| v == 0.0) {
        return SpectralFingerprint {
            bands: vec![0.0f32; 8],
            centroid: 0.0, rolloff: 0.0, brightness: 0.0,
            sub_energy: 0.0, low_mid_energy: 0.0, high_mid_energy: 0.0,
            presence_energy: 0.0, air_energy: 0.0,
            tonal_ratio: 0.5, noise_ratio: 0.5,
            resonance_peaks: vec![],
            harshness: 0.0, muddiness: 0.0,
            spectral_flatness: 0.5,
        };
    }

    let total_energy: f32 = avg_spec.iter().sum();

    let sub_bins = freq_to_bin(150.0, fft_size);
    let low_mid_bins = freq_to_bin(500.0, fft_size);
    let high_mid_bins = freq_to_bin(2000.0, fft_size);
    let presence_bins = freq_to_bin(6000.0, fft_size);
    let air_bins = freq_to_bin(12000.0, fft_size);

    let sub_energy: f32 = avg_spec[..sub_bins.min(half)].iter().sum();
    let low_mid_energy: f32 = avg_spec[sub_bins.min(half)..low_mid_bins.min(half)].iter().sum();
    let high_mid_energy: f32 = avg_spec[low_mid_bins.min(half)..high_mid_bins.min(half)].iter().sum();
    let presence_energy: f32 = avg_spec[high_mid_bins.min(half)..presence_bins.min(half)].iter().sum();
    let air_energy: f32 = avg_spec[presence_bins.min(half)..air_bins.min(half)].iter().sum();

    let norm = |e: f32| if total_energy > 0.0 { e / total_energy } else { 0.0 };

    // 8-band fingerprint
    let band_edges = [
        freq_to_bin(60.0, fft_size),
        freq_to_bin(150.0, fft_size),
        freq_to_bin(400.0, fft_size),
        freq_to_bin(1000.0, fft_size),
        freq_to_bin(3000.0, fft_size),
        freq_to_bin(6000.0, fft_size),
        freq_to_bin(10000.0, fft_size),
        half,
    ];
    let mut bands = Vec::with_capacity(8);
    let mut prev = 0;
    for &edge in &band_edges {
        let edge = edge.min(half);
        let band_energy: f32 = avg_spec[prev..edge].iter().sum();
        bands.push(norm(band_energy));
        prev = edge;
    }

    let centroid = analyze::compute_spectral_centroid(samples);
    let rolloff = analyze::compute_spectral_rolloff(samples, 0.85);
    let brightness = analyze::compute_brightness(samples);
    let spectral_flatness = analyze::compute_spectral_flatness(samples);
    // Tonal vs noise ratio
    let tonal_ratio = (1.0 - spectral_flatness).max(0.0);
    let noise_ratio = spectral_flatness;

    // Find resonance peaks
    let mut peaks = Vec::new();
    for i in 3..half.saturating_sub(3) {
        if avg_spec[i] > avg_spec[i - 1] && avg_spec[i] > avg_spec[i - 2]
            && avg_spec[i] > avg_spec[i + 1] && avg_spec[i] > avg_spec[i + 2]
        {
            let prominence = if avg_spec[i] > 0.001 {
                let local_min = avg_spec[i - 3..=i + 3].iter().cloned().fold(f32::MAX, f32::min);
                avg_spec[i] / local_min.min(avg_spec[i]).max(0.001)
            } else {
                0.0
            };
            if prominence > 2.0 {
                peaks.push((bin_to_freq(i, fft_size), prominence));
            }
        }
    }
    peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    peaks.truncate(10);

    // Harshness detection
    let harshness = {
        let harsh_band_start = freq_to_bin(2000.0, fft_size);
        let harsh_band_end = freq_to_bin(8000.0, fft_size);
        if harsh_band_end > harsh_band_start && harsh_band_end <= half {
            let harsh_energy: f32 = avg_spec[harsh_band_start..harsh_band_end].iter().sum();
            let rest_energy = total_energy - harsh_energy;
            if rest_energy > 0.0 {
                (harsh_energy / rest_energy * 2.0).min(1.0)
            } else { 0.0 }
        } else { 0.0 }
    };

    // Muddiness detection
    let muddiness = {
        let mud_band_start = freq_to_bin(150.0, fft_size);
        let mud_band_end = freq_to_bin(500.0, fft_size);
        if mud_band_end > mud_band_start && mud_band_end <= half {
            let mud_energy: f32 = avg_spec[mud_band_start..mud_band_end].iter().sum();
            let high_energy = total_energy - sub_energy - mud_energy;
            let mud_ratio = if high_energy > 0.0 { mud_energy / high_energy } else { 0.0 };
            if mud_ratio > 0.5 { ((mud_ratio - 0.5) * 2.0).min(1.0) } else { 0.0 }
        } else { 0.0 }
    };

    SpectralFingerprint {
        bands,
        centroid,
        rolloff,
        brightness,
        sub_energy: norm(sub_energy),
        low_mid_energy: norm(low_mid_energy),
        high_mid_energy: norm(high_mid_energy),
        presence_energy: norm(presence_energy),
        air_energy: norm(air_energy),
        tonal_ratio,
        noise_ratio,
        resonance_peaks: peaks,
        harshness,
        muddiness,
        spectral_flatness,
    }
}

pub fn compare_fingerprints(a: &SpectralFingerprint, b: &SpectralFingerprint) -> SpectralMatch {
    let band_sim = if a.bands.len() == b.bands.len() && !a.bands.is_empty() {
        let diff: f32 = a.bands.iter().zip(b.bands.iter())
            .map(|(x, y)| (x - y).abs())
            .sum::<f32>() / a.bands.len() as f32;
        (1.0 - diff).max(0.0)
    } else {
        0.5
    };

    let centroid_sim = if a.centroid > 0.0 && b.centroid > 0.0 {
        let ratio = if a.centroid > b.centroid {
            b.centroid / a.centroid
        } else {
            a.centroid / b.centroid
        };
        ratio.max(0.0)
    } else {
        0.5
    };

    let brightness_sim = 1.0 - (a.brightness - b.brightness).abs();
    let tonal_sim = 1.0 - (a.tonal_ratio - b.tonal_ratio).abs();

    let resonance_sim = if a.resonance_peaks.is_empty() && b.resonance_peaks.is_empty() {
        1.0
    } else if a.resonance_peaks.is_empty() || b.resonance_peaks.is_empty() {
        0.3
    } else {
        let mut sim = 0.0f32;
        let max_peaks = a.resonance_peaks.len().min(b.resonance_peaks.len()).min(5);
        for i in 0..max_peaks {
            let freq_diff = (a.resonance_peaks[i].0 - b.resonance_peaks[i].0).abs();
            if freq_diff < 100.0 {
                sim += 1.0 - freq_diff / 100.0;
            }
        }
        sim / max_peaks.max(1) as f32
    };

    let overall = band_sim * 0.3 + centroid_sim * 0.2 + brightness_sim * 0.15
        + tonal_sim * 0.2 + resonance_sim * 0.15;

    SpectralMatch {
        overall,
        band_similarity: band_sim,
        centroid_similarity: centroid_sim,
        brightness_similarity: brightness_sim,
        tonal_similarity: tonal_sim,
        resonance_similarity: resonance_sim,
    }
}

pub fn detect_harshness(samples: &[f32]) -> f32 {
    let fp = extract_fingerprint(samples);
    fp.harshness
}

pub fn detect_muddiness(samples: &[f32]) -> f32 {
    let fp = extract_fingerprint(samples);
    fp.muddiness
}

pub fn detect_resonance(samples: &[f32]) -> Vec<(f32, f32)> {
    let fp = extract_fingerprint(samples);
    fp.resonance_peaks.clone()
}

pub fn spectral_anomaly_score(samples: &[f32]) -> f32 {
    let fp = extract_fingerprint(samples);
    let harsh_penalty = fp.harshness * 0.3;
    let mud_penalty = fp.muddiness * 0.3;
    let flatness_penalty = (1.0 - fp.spectral_flatness).abs() * 0.2;
    let balance = 1.0 - (fp.sub_energy - 0.2).abs().min(0.5) * 0.2;
    (balance - harsh_penalty - mud_penalty - flatness_penalty * 0.2).clamp(0.0, 1.0)
}
