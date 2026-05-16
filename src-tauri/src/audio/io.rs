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
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).map_err(|e| e.to_string())?;
    let amplitude = i32::MAX as f64;
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0) as f64;
        let int_val = (clamped * amplitude) as i32;
        writer
            .write_sample(int_val)
            .map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;

    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if file_size < 44 {
        return Err("Written WAV file is too small to be valid".to_string());
    }

    Ok(())
}

#[allow(dead_code)]
pub fn write_wav_bytes(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    if samples.is_empty() {
        return Err("Cannot write empty audio buffer".to_string());
    }

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut cursor, spec).map_err(|e| e.to_string())?;
    let amplitude = i32::MAX as f64;
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0) as f64;
        let int_val = (clamped * amplitude) as i32;
        writer
            .write_sample(int_val)
            .map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(cursor.into_inner())
}

pub fn read_wav(path: &Path) -> Result<Vec<f32>, String> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if file_size < 44 {
        return Err("File is too small to be a valid WAV (minimum 44 bytes)".to_string());
    }

    let mut reader = hound::WavReader::open(path).map_err(|e| {
        format!("Could not read WAV file: {}", e)
    })?;
    let spec = reader.spec();

    if spec.channels == 0 || spec.sample_rate == 0 {
        return Err("Invalid WAV spec: zero channels or sample rate".to_string());
    }

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .filter_map(|s| s.ok())
            .collect(),
        hound::SampleFormat::Int => {
            let max = (2i32.pow(spec.bits_per_sample as u32 - 1) - 1) as f64;
            reader
                .samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f64 / max as f64)
                .map(|s| s as f32)
                .collect()
        }
    };

    if samples.is_empty() {
        return Err("WAV file contains no audio data".to_string());
    }

    if spec.channels > 1 {
        let mono: Vec<f32> = samples
            .chunks(spec.channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / spec.channels as f32)
            .collect();
        return Ok(mono);
    }

    Ok(samples)
}
