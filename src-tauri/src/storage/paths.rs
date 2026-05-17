use std::path::PathBuf;

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn app_root() -> PathBuf {
    home_dir().join("cShot")
}

pub fn audio_dir() -> PathBuf {
    app_root().join("audio")
}

pub fn database_path() -> PathBuf {
    app_root().join("library.db")
}

pub fn sound_path(sound_id: &str) -> PathBuf {
    audio_dir().join(format!("{}.wav", sound_id))
}

pub fn favorites_path() -> PathBuf {
    app_root().join("favorites.json")
}

pub fn feedback_path() -> PathBuf {
    app_root().join("feedback.json")
}

pub fn export_dir() -> PathBuf {
    home_dir().join("Desktop")
}

pub fn export_dir_organized() -> PathBuf {
    let now = chrono_now();
    app_root().join("exports").join(now)
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
