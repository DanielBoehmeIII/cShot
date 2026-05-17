use std::io::Cursor;
use std::path::Path;

pub fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) -> Result<(), String> {
    if samples.is_empty() {
        return Err("Cannot write empty audio buffer".to_string());
    }
    if samples.iter().any(|s| s.is_nan() || s.is_infinite()) {
        return Err("Audio buffer contains NaN or Inf values".to_string());
    }

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(path, spec).map_err(|e| {
        format!("Could not create WAV file at {}: {}", path.display(), e)
    })?;

    let mut written = 0usize;
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        writer.write_sample(clamped).map_err(|e| {
            format!("Failed to write sample {}: {}", written, e)
        })?;
        written += 1;
    }

    writer.finalize().map_err(|e| format!("Failed to finalize WAV: {}", e))?;

    // Verify write succeeded
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if file_size < 44 {
        return Err("Written WAV file is too small to be valid".to_string());
    }

    Ok(())
}

pub fn write_wav_bytes(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    if samples.is_empty() {
        return Err("Cannot write empty audio buffer".to_string());
    }
    if samples.iter().any(|s| s.is_nan() || s.is_infinite()) {
        return Err("Audio buffer contains NaN or Inf values".to_string());
    }

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut cursor = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut cursor, spec).map_err(|e| e.to_string())?;

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        writer.write_sample(clamped).map_err(|e| e.to_string())?;
    }

    writer.finalize().map_err(|e| e.to_string())?;
    let bytes = cursor.into_inner();
    if bytes.len() < 44 {
        return Err("Generated WAV bytes too small".to_string());
    }
    Ok(bytes)
}

pub fn read_wav(path: &Path) -> Result<Vec<f32>, String> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    // WAV header is 44 bytes minimum, 8 bytes minimum for RF64
    if file_size < 44 {
        return Err(format!("File too small for valid WAV ({} bytes)", file_size));
    }

    let mut reader = hound::WavReader::open(path).map_err(|e| {
        format!("Could not read WAV file {}: {}", path.display(), e)
    })?;

    let spec = reader.spec();
    if spec.channels == 0 {
        return Err("WAV file has zero channels".to_string());
    }
    if spec.sample_rate == 0 {
        return Err("WAV file has zero sample rate".to_string());
    }
    if spec.bits_per_sample == 0 {
        return Err("WAV file has zero bit depth".to_string());
    }

    let total_samples = reader.duration() as usize * spec.channels as usize;
    if total_samples == 0 {
        return Err("WAV file contains no audio data".to_string());
    }

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .filter_map(|s| s.ok())
            .take(total_samples)
            .collect(),
        hound::SampleFormat::Int => {
            let max_val = (2i64.pow(spec.bits_per_sample as u32) / 2 - 1) as f64;
            reader
                .samples::<i32>()
                .filter_map(|s| s.ok())
                .take(total_samples)
                .map(|s| (s as f64 / max_val) as f32)
                .collect()
        }
    };

    if samples.is_empty() {
        return Err("No audio samples could be read from WAV file".to_string());
    }

    // Ensure no NaN/Inf from parsing
    if samples.iter().any(|s| s.is_nan() || s.is_infinite()) {
        return Err("WAV file contains invalid sample values".to_string());
    }

    // Downmix to mono if necessary
    if spec.channels > 1 {
        let mono: Vec<f32> = samples
            .chunks(spec.channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / spec.channels as f32)
            .collect();
        return Ok(mono);
    }

    Ok(samples)
}

/// Robust WAV reader with multiple retry strategies
pub fn read_wav_safe(path: &Path) -> Result<Vec<f32>, String> {
    let result = read_wav(path);
    if result.is_ok() {
        return result;
    }

    // Retry once after a short pause (for file system races)
    std::thread::sleep(std::time::Duration::from_millis(10));
    read_wav(path)
}
