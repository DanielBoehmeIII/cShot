use std::sync::Mutex;

pub struct AppState {
    pub favorites: Mutex<crate::favorites::FavoritesStore>,
    pub db: Mutex<rusqlite::Connection>,
    pub provider_registry: Mutex<crate::generation::registry::ProviderRegistry>,
}

pub fn run() {
    // Load .env file from project root or home directory
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_filename(std::path::Path::new(&dirs_or_home()).join(".cshot.env"));

    let db_path = crate::storage::database_path();
    std::fs::create_dir_all(crate::storage::app_root()).ok();
    let db = crate::db::init_database(&db_path).expect("Failed to init database");

    let registry = crate::generation::build_default_registry();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            favorites: std::sync::Mutex::new(crate::favorites::FavoritesStore::load()),
            db: Mutex::new(db),
            provider_registry: Mutex::new(registry),
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::generate_sound,
            crate::commands::generate_variants,
            crate::commands::get_audio_data,
            crate::commands::export_wav,
            crate::commands::toggle_favorite,
            crate::commands::get_favorites,
            crate::commands::get_sound_history,
            crate::commands::search_sounds,
            crate::commands::get_sound_detail,
            crate::commands::copy_reference,
            crate::commands::analyze_reference,
            crate::commands::read_audio_file,
            crate::commands::delete_sound,
            crate::commands::list_sounds,
            crate::commands::count_library_sounds,
            crate::commands::count_favorite_sounds,
            crate::commands::export_all_favorites,
            crate::commands::submit_feedback,
            crate::commands::open_export_folder,
            crate::commands::get_recent_exports,
            crate::commands::cleanup_clear_generated,
            crate::commands::cleanup_clear_failed,
            crate::commands::cleanup_clear_orphans,
            crate::commands::cleanup_reset_database,
            crate::commands::cleanup_rebuild_metadata,
            crate::commands::cleanup_clear_everything,
            crate::commands::scan_integrity,
            crate::commands::update_missing_hashes,
            crate::commands::get_sound_provenance,
            crate::commands::apply_repair,
            crate::commands::get_sound_children,
            crate::commands::create_recipe,
            crate::commands::update_recipe,
            crate::commands::delete_recipe,
            crate::commands::list_recipes,
            crate::commands::search_recipes,
            crate::commands::toggle_recipe_favorite,
            crate::commands::increment_recipe_usage,
            crate::commands::generate_from_recipe,
            crate::commands::import_sample,
            crate::commands::scan_folder_import,
            crate::commands::find_duplicates,
            crate::commands::get_library_stats,
            crate::commands::update_pack_notes,
            crate::commands::update_pack_metadata,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub mod audio;
pub mod cleanup;
pub mod commands;
pub mod db;
pub mod embeddings;
pub mod favorites;
pub mod feedback;
pub mod generation;
pub mod integrity;
pub mod generator;
pub mod prompt;
pub mod quality;
pub mod score;
pub mod semantic_library;
pub mod storage;

fn dirs_or_home() -> String {
    std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".to_string())
}
