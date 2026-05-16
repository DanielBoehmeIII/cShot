use std::path::Path;

pub const MAX_UPLOAD_DURATION_MS: f32 = 60_000.0;
pub const MAX_ONE_SHOT_DURATION_MS: f32 = 5_000.0;
pub const MIN_DURATION_MS: f32 = 10.0;
pub const SUPPORTED_FORMATS: &[&str] = &["wav"];

pub enum FileFormat {
    Wav,
    Unsupported(String),
}

pub fn detect_format(path: &Path) -> FileFormat {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "wav" => FileFormat::Wav,
        other => FileFormat::Unsupported(other.to_string()),
    }
}

pub struct UploadValidation {
    pub is_valid: bool,
    pub format_ok: bool,
    pub size_ok: bool,
    pub duration_ok: bool,
    pub error: Option<String>,
}

pub fn validate_upload(path: &Path, file_size: u64) -> UploadValidation {
    if !path.exists() {
        return UploadValidation {
            is_valid: false,
            format_ok: false,
            size_ok: false,
            duration_ok: false,
            error: Some("File not found".to_string()),
        };
    }

    if file_size == 0 {
        return UploadValidation {
            is_valid: false,
            format_ok: true,
            size_ok: false,
            duration_ok: false,
            error: Some("File is empty".to_string()),
        };
    }

    let format = detect_format(path);
    let format_ok = matches!(format, FileFormat::Wav);
    if !format_ok {
        let ext = match format {
            FileFormat::Unsupported(ref e) => e.clone(),
            _ => "unknown".to_string(),
        };
        return UploadValidation {
            is_valid: false,
            format_ok: false,
            size_ok: true,
            duration_ok: false,
            error: Some(format!(
                "Unsupported format '.{}'. Only WAV files are supported.",
                ext
            )),
        };
    }

    match hound::WavReader::open(path) {
        Ok(reader) => {
            let spec = reader.spec();
            let num_samples = reader.duration() as u64;
            let duration_ms =
                num_samples as f32 / spec.sample_rate as f32 * 1000.0;

            let mut issues = Vec::new();

            if num_samples == 0 {
                issues.push("File contains no audio data".to_string());
            }

            if duration_ms > MAX_UPLOAD_DURATION_MS {
                issues.push(format!(
                    "File is too long ({:.0}s). Maximum upload duration is {}s.",
                    duration_ms / 1000.0,
                    MAX_UPLOAD_DURATION_MS as u32 / 1000
                ));
            }

            if duration_ms < MIN_DURATION_MS {
                issues.push(format!(
                    "File is too short ({:.0}ms). Minimum duration is {}ms.",
                    duration_ms, MIN_DURATION_MS as u32
                ));
            }

            if issues.is_empty() {
                UploadValidation {
                    is_valid: true,
                    format_ok: true,
                    size_ok: true,
                    duration_ok: true,
                    error: None,
                }
            } else {
                UploadValidation {
                    is_valid: false,
                    format_ok: true,
                    size_ok: true,
                    duration_ok: false,
                    error: Some(issues.join(" ")),
                }
            }
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("no valid WAV") || msg.contains("header") {
                UploadValidation {
                    is_valid: false,
                    format_ok: false,
                    size_ok: true,
                    duration_ok: false,
                    error: Some("File is not a valid WAV file. The header could not be read.".to_string()),
                }
            } else {
                UploadValidation {
                    is_valid: false,
                    format_ok: true,
                    size_ok: true,
                    duration_ok: false,
                    error: Some(format!("Could not read audio file: {}", msg)),
                }
            }
        }
    }
}

pub fn validate_one_shot_duration(duration_ms: f32) -> Result<(), String> {
    if duration_ms > MAX_ONE_SHOT_DURATION_MS {
        return Err(format!(
            "Generated sound is too long ({:.0}ms). Maximum one-shot duration is {}s.",
            duration_ms,
            MAX_ONE_SHOT_DURATION_MS as u32 / 1000
        ));
    }
    if duration_ms < MIN_DURATION_MS {
        return Err(format!(
            "Generated sound is too short ({:.0}ms). Minimum duration is {}ms.",
            duration_ms, MIN_DURATION_MS as u32
        ));
    }
    Ok(())
}
