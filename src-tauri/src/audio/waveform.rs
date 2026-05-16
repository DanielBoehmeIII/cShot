pub fn compute_waveform(samples: &[f32], num_points: usize) -> Vec<f32> {
    if samples.is_empty() || num_points == 0 {
        return vec![0.0; num_points];
    }
    let mut waveform = Vec::with_capacity(num_points);
    let chunk_size = (samples.len() / num_points).max(1);
    for i in 0..num_points {
        let start = i * chunk_size;
        let end = ((i + 1) * chunk_size).min(samples.len());
        if start >= samples.len() {
            waveform.push(0.0);
        } else {
            let peak = samples[start..end]
                .iter()
                .map(|&s| s.abs())
                .fold(0.0f32, f32::max);
            waveform.push(peak);
        }
    }
    waveform
}
