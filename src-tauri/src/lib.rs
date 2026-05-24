// Crate-level clippy allows for intentional style choices & DSP code patterns
#![allow(
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::needless_range_loop,
    clippy::excessive_precision,
    clippy::too_many_arguments,
    clippy::manual_range_contains,
    clippy::assign_op_pattern,
    clippy::type_complexity,
    clippy::manual_clamp,
    clippy::blocks_in_conditions,
    clippy::needless_borrow,
    clippy::derivable_impls,
    clippy::vec_init_then_push,
)]

use std::sync::Mutex;

pub struct AppState {
    pub favorites: Mutex<crate::favorites::FavoritesStore>,
    pub db: Mutex<rusqlite::Connection>,
    pub provider_registry: Mutex<crate::generation::registry::ProviderRegistry>,
    pub history: Mutex<crate::history::ActionHistory>,
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
            history: Mutex::new(crate::history::ActionHistory::new(50)),
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::generate_sound,
            crate::commands::generate_variants,
            crate::commands::generate_resynthesis_variants,
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
            crate::commands::analyze_prompt,
            crate::commands::apply_repair,
            crate::commands::transform_sound,
            crate::commands::resynthesize_sound,
            crate::commands::recreate_sound,
            crate::commands::hybrid_reconstruct,
            crate::commands::get_audio_analysis,
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
            crate::commands::undo_last_action,
            crate::commands::redo_last_action,
            crate::commands::can_undo_action,
            crate::commands::can_redo_action,
            crate::commands::get_last_generation,
            crate::commands::record_generation_action,
            crate::commands::get_sound_duration,
            crate::commands::get_control_modes,
            crate::commands::params_from_exposed,
            crate::commands::generate_with_params,
            crate::commands::get_exposed_params_defaults,
            crate::commands::generate_variant_from_params,
            crate::commands::trigger_midi_note,
            crate::commands::rapid_randomize,
            crate::commands::morph_presets,
            crate::commands::morph_sounds_command,
            crate::commands::explore_stream,
            crate::commands::branch_from_sound,
            crate::commands::recipe_roulette,
            crate::commands::quick_compare,
            crate::commands::live_preview_generate,
            crate::commands::apply_spectral_edit,
            crate::commands::isolate_region,
            crate::commands::generate_intelligent_pack,
            crate::commands::analyze_pack_cohesion_command,
            crate::commands::analyze_kit_command,
            crate::commands::design_workflow,
            crate::commands::record_taste_action,
            crate::commands::get_taste_profile,
            crate::commands::get_taste_suggestions,
            crate::commands::get_preferred_defaults,
            crate::commands::score_variant_by_taste,
            crate::commands::mutate_sound_command,
            crate::commands::hybridize_sounds_command,
            crate::commands::get_available_mutations,
            crate::commands::get_session_state,
            crate::commands::set_active_sound_session,
            crate::commands::set_last_prompt_session,
            crate::commands::set_view_state_session,
            crate::commands::list_presets,
            crate::commands::save_preset_command,
            crate::commands::delete_preset_command,
            crate::commands::verify_sound_integrity,
            crate::commands::export_sound_safe,
            crate::commands::get_app_identity,
            crate::commands::get_default_presets,
            crate::commands::get_quick_start_workflows,
            crate::commands::get_capability_summary,
            crate::commands::parse_natural_language_query,
            crate::commands::semantic_search,
            crate::commands::analyze_audio_intelligence,
            crate::commands::score_for_ranking_command,
            crate::commands::recommendation_score_command,
            crate::commands::recreate_advanced_command,
            crate::commands::sculpt_sound,
            crate::commands::sculpt_preview,
            crate::commands::auto_tag_sound,
            crate::commands::suggest_pack_command,
            crate::commands::suggest_filename_command,
            crate::commands::get_builtin_presets,
            crate::commands::apply_preset_macro,
            crate::commands::run_stress_test,
            crate::commands::validate_export_command,
            crate::commands::batch_validate_exports,
            crate::commands::get_library_optimization_stats,
            crate::commands::get_library_cache,
            crate::commands::set_library_cache,
            crate::commands::generate_with_intent,
            crate::commands::get_intent_presets,
            crate::commands::blend_intent_profiles,
            crate::commands::evolve_sound_command,
            crate::commands::save_evolution_member,
            crate::commands::quick_branch,
            crate::commands::batch_favorite,
            crate::commands::batch_export,
            crate::commands::get_recent_prompts,
            crate::commands::list_sound_classes,
            crate::commands::get_oneshot_defaults,
            crate::commands::render_oneshot,
            crate::commands::export_oneshot_wav,
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
pub mod history;
pub mod generation;
pub mod integrity;
pub mod generator;
pub mod prompt;
pub mod prompt_dsp;
pub mod quality;
pub mod score;
pub mod semantic_library;
pub mod storage;

fn dirs_or_home() -> String {
    std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".to_string())
}
