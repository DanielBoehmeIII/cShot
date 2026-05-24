use super::analyze::analyze_audio;
use super::resynthesize;
use super::recreate;
use super::mutation;
use super::process;
use super::SAMPLE_RATE;
use super::SoundType;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StressTestResult {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub avg_generation_time_ms: f64,
    pub max_generation_time_ms: f64,
    pub min_generation_time_ms: f64,
    pub silent_outputs: usize,
    pub clipped_outputs: usize,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExportValidation {
    pub is_valid: bool,
    pub duration_ms: f32,
    pub sample_rate: u32,
    pub peak: f32,
    pub rms: f32,
    pub has_clipping: bool,
    pub is_silent: bool,
    pub dc_offset: f32,
    pub warnings: Vec<String>,
}

pub fn run_stress_test(iterations: usize) -> StressTestResult {
    let mut errors = Vec::new();
    let mut total_time = 0.0f64;
    let mut max_time = 0.0f64;
    let mut min_time = f64::MAX;
    let mut silent = 0;
    let mut clipped = 0;
    let mut passed = 0;
    let mut failed = 0;

    let sound_types = [
        SoundType::Kick, SoundType::Snare, SoundType::ClosedHat,
        SoundType::OpenHat, SoundType::Clap, SoundType::Tom,
        SoundType::Perc, SoundType::Bass, SoundType::Fx, SoundType::Other,
    ];

    for i in 0..iterations {
        let st = sound_types[i % sound_types.len()];
        let dur = 100.0 + (i as f32 * 37.0) % 2000.0;
        let pitch = 40.0 + (i as f32 * 13.0) % 800.0;

        let start = std::time::Instant::now();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut params = super::resynthesize::params_for_sound_type(st, pitch, dur);
            params = params.with_seed(i as u64).randomize(0.5);

            let samples = resynthesize::resynthesize(&params);
            if samples.is_empty() {
                return Err("Empty synthesis result".to_string());
            }
            let mut s = samples;

            let analysis = analyze_audio(&s, SAMPLE_RATE, 1);
            let (recreated, _a, _sim) = recreate::recreate_single(&s, 0.7);
            if recreated.is_empty() {
                return Err("Empty recreation".to_string());
            }

            let (mutated, _sim2) = mutation::mutate_sound(&s, &analysis, 0.5);
            if mutated.is_empty() {
                return Err("Empty mutation".to_string());
            }

            process::normalize_peak(&mut s, -1.0);
            if s.iter().any(|&v| v.is_nan() || v.is_infinite()) {
                return Err("NaN/Inf in samples".to_string());
            }

            Ok((s, recreated, mutated))
        }));

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        total_time += elapsed;
        max_time = max_time.max(elapsed);
        min_time = min_time.min(elapsed);

        match result {
            Ok(Ok((s, _, _))) => {
                passed += 1;
                let peak = s.iter().map(|&v| v.abs()).fold(0.0f32, f32::max);
                let _rms = s.iter().map(|&v| v * v).sum::<f32>().sqrt() / (s.len() as f32).sqrt();
                if peak < 0.001 { silent += 1; }
                if peak >= 1.0 { clipped += 1; }
            }
            Ok(Err(e)) => {
                failed += 1;
                errors.push(format!("Iteration {}: {}", i, e));
            }
            Err(_) => {
                failed += 1;
                errors.push(format!("Iteration {}: Panic", i));
            }
        }
    }

    let avg = if iterations > 0 { total_time / iterations as f64 } else { 0.0 };

    StressTestResult {
        total_tests: iterations,
        passed,
        failed,
        avg_generation_time_ms: avg,
        max_generation_time_ms: max_time,
        min_generation_time_ms: if min_time == f64::MAX { 0.0 } else { min_time },
        silent_outputs: silent,
        clipped_outputs: clipped,
        errors,
    }
}

pub fn validate_export(samples: &[f32], sample_rate: u32) -> ExportValidation {
    let mut warnings = Vec::new();
    let duration_ms = samples.len() as f32 / sample_rate as f32 * 1000.0;
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    let rms = samples.iter().map(|&s| s * s).sum::<f32>().sqrt() / (samples.len() as f32).sqrt();
    let has_clipping = samples.iter().any(|&s| s.abs() >= 1.0);
    let is_silent = peak < 0.001;

    let dc_offset = samples.iter().sum::<f32>() / samples.len() as f32;

    if duration_ms < 10.0 {
        warnings.push("Very short audio (<10ms)".to_string());
    }
    if duration_ms > 10000.0 {
        warnings.push("Very long audio (>10s)".to_string());
    }
    if has_clipping {
        warnings.push("Clipping detected".to_string());
    }
    if is_silent {
        warnings.push("Silent audio".to_string());
    }
    if dc_offset.abs() > 0.01 {
        warnings.push(format!("DC offset: {:.4}", dc_offset));
    }
    if rms < 0.01 && peak > 0.1 {
        warnings.push("Low RMS with high peak (possible spike)".to_string());
    }

    ExportValidation {
        is_valid: !is_silent && !has_clipping && duration_ms >= 10.0,
        duration_ms,
        sample_rate,
        peak,
        rms,
        has_clipping,
        is_silent,
        dc_offset,
        warnings,
    }
}
