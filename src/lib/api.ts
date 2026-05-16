// ─── Sound Types ─────────────────────────────────────────────

export interface SoundResult {
  id: string;
  waveform: number[];
  sound_type: string;
  tags: string[];
  duration_ms: number;
  prompt: string;
  variant_name?: string;
  source: string;
  model: string;
  seed: number;
  rms: number;
  peak: number;
  spectral_centroid: number;
  score: number;
  failure_labels: string[];
}

export interface VariantResult {
  id: string;
  waveform: number[];
  sound_type: string;
  tags: string[];
  duration_ms: number;
  prompt: string;
  variant_name: string;
  source: string;
  model: string;
  seed: number;
  score: number;
  failure_labels: string[];
}

export interface SoundMetadata {
  id: string;
  prompt: string;
  sound_type: string;
  duration_ms: number;
  created_at: string;
  source: string;
  model: string;
  seed: number;
  variant_name?: string;
}

export interface SoundEntry {
  id: string;
  prompt: string;
  sound_type: string;
  duration_ms: number;
  sample_rate: number;
  rms: number;
  peak: number;
  spectral_centroid: number;
  tags: string;
  is_favorite: boolean;
  source: string;
  variant_name: string | null;
  model: string;
  seed: number;
  created_at: string;
}

export interface ExportResult {
  path: string;
  filename: string;
  file_size_bytes: number;
}

export interface SoundDetail {
  entry: SoundEntry;
  waveform: number[];
  file_size_bytes: number;
  file_exists: boolean;
}

export interface ReferenceAnalysis {
  id: string;
  path: string;
  filename: string;
  duration_ms: number;
  sample_rate: number;
  channels: number;
  file_type: string;
  waveform: number[];
  rms: number;
  peak: number;
  validation_message?: string;
}

// ─── Generation API ─────────────────────────────────────────

export async function generateSound(
  prompt: string,
  referencePath?: string,
  providerName?: string,
): Promise<SoundResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("generate_sound", {
    prompt,
    referencePath: referencePath || null,
    providerName: providerName || null,
  });
}

export async function generateVariants(
  prompt: string,
  sourceId: string,
  count: number,
): Promise<VariantResult[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("generate_variants", {
    prompt,
    sourceId,
    count,
  });
}

export async function regenerateSound(
  prompt: string,
  sourceId: string,
): Promise<SoundResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("regenerate_sound", { prompt, sourceId });
}

// ─── Playback API ───────────────────────────────────────────

export async function getAudioData(soundId: string): Promise<number[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_audio_data", { soundId });
}

// ─── Export API ─────────────────────────────────────────────

export async function exportWav(soundId: string): Promise<ExportResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("export_wav", { soundId });
}

export async function exportAllFavorites(): Promise<ExportResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("export_all_favorites");
}

export async function openExportFolder(): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  const result: { path: string; opened: boolean } = await invoke("open_export_folder");
  return result.path;
}

export async function getRecentExports(): Promise<ExportResult[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_recent_exports");
}

// ─── Favorites API ──────────────────────────────────────────

export async function toggleFavorite(
  soundId: string,
  prompt: string,
  soundType: string,
  durationMs: number,
  seed?: number,
  variantName?: string,
): Promise<boolean> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("toggle_favorite", {
    soundId,
    prompt,
    soundType,
    durationMs,
    seed: seed ?? 0,
    variantName: variantName ?? null,
  });
}

export async function getFavorites(): Promise<SoundMetadata[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_favorites");
}

export async function countFavoriteSounds(): Promise<number> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("count_favorite_sounds");
}

// ─── Library API ────────────────────────────────────────────

export async function listSounds(
  limit: number,
  offset: number,
  favoritesOnly: boolean,
): Promise<SoundEntry[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("list_sounds", { limit, offset, favoritesOnly });
}

export async function getSoundDetail(soundId: string): Promise<SoundDetail> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_sound_detail", { soundId });
}

export async function getSoundHistory(): Promise<SoundEntry[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_sound_history");
}

export async function deleteSound(soundId: string): Promise<boolean> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("delete_sound", { soundId });
}

export async function countLibrarySounds(): Promise<number> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("count_library_sounds");
}

// ─── Search API ─────────────────────────────────────────────

export async function searchSounds(query: string): Promise<SoundEntry[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("search_sounds", { query });
}

export async function findSimilarSounds(
  soundId: string,
  maxResults?: number,
): Promise<SoundEntry[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("find_similar_sounds", { soundId, maxResults: maxResults ?? null });
}

// ─── Reference API ──────────────────────────────────────────

export async function analyzeReference(path: string): Promise<ReferenceAnalysis> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("analyze_reference", { path });
}

export async function openReferenceDialog(): Promise<string | null> {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const file = await open({
      filters: [{ name: "WAV Audio", extensions: ["wav"] }],
      multiple: false,
    });
    if (!file) return null;
    return copyReference(file as string);
  } catch {
    return null;
  }
}

async function copyReference(path: string): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("copy_reference", { sourceId: path });
}

// ─── Pack API ───────────────────────────────────────────────

export async function createPack(
  title: string,
  description?: string,
  tags?: string[],
): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("create_pack", { title, description: description ?? null, tags: tags ?? null });
}

export async function listPacks(): Promise<SoundEntry[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("list_packs");
}

export async function deletePack(packId: string): Promise<boolean> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("delete_pack", { packId });
}

export async function addToPack(
  packId: string,
  soundId: string,
  sortOrder?: number,
): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("add_to_pack", { packId, soundId, sortOrder: sortOrder ?? null });
}

export async function removeFromPack(packId: string, soundId: string): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("remove_from_pack", { packId, soundId });
}

export async function exportPack(packId: string): Promise<ExportResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("export_pack", { packId });
}

// ─── Quality & Score API ────────────────────────────────────

export async function getSoundQuality(soundId: string): Promise<SoundEntry> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_sound_quality", { soundId });
}

export async function getSoundScore(soundId: string): Promise<SoundEntry> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_sound_score", { soundId });
}

// ─── Feedback API ───────────────────────────────────────────

export async function submitFeedback(
  soundId: string,
  thumbsUp: boolean,
  thumbsDown: boolean,
  usable?: boolean,
): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("submit_feedback", { soundId, thumbsUp, thumbsDown, usable: usable ?? null });
}

// ─── Repair API ────────────────────────────────────────────

export interface RepairResult {
  id: string;
  waveform: number[];
  prompt: string;
  sound_type: string;
  tags: string[];
  duration_ms: number;
  source: string;
  model: string;
  seed: number;
  score: number;
  failure_labels: string[];
  rms: number;
  peak: number;
  spectral_centroid: number;
  variant_name?: string;
}

export type RepairAction = "Normalize" | "TrimSilence" | "Fade" | "Shorten" | "Brighten" | "Darken" | "Punch";

export async function applyRepair(
  soundId: string,
  action: RepairAction,
): Promise<RepairResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("apply_repair", { soundId, action });
}

// ─── Integrity API ──────────────────────────────────────────

export interface IntegrityReport {
  total_db_sounds: number;
  total_disk_files: number;
  missing_files: string[];
  orphan_files: string[];
  hash_mismatches: string[];
  db_entries_without_hash: number;
}

export async function scanIntegrity(): Promise<IntegrityReport> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("scan_integrity");
}

export async function updateMissingHashes(): Promise<number> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("update_missing_hashes");
}

// ─── Cleanup API ────────────────────────────────────────────

export interface CleanupResult {
  deleted_sounds: number;
  deleted_orphans: number;
  deleted_failed: number;
  errors: string[];
  favorites_protected: number;
}

export async function cleanupClearGenerated(includeFavorites: boolean): Promise<CleanupResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("cleanup_clear_generated", { includeFavorites });
}

export async function cleanupClearFailed(): Promise<CleanupResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("cleanup_clear_failed");
}

export async function cleanupClearOrphans(): Promise<CleanupResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("cleanup_clear_orphans");
}

export async function cleanupResetDatabase(dryRun: boolean): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("cleanup_reset_database", { dryRun });
}

export async function cleanupRebuildMetadata(): Promise<CleanupResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("cleanup_rebuild_metadata");
}

export async function cleanupClearEverything(): Promise<CleanupResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("cleanup_clear_everything");
}

// ─── Version Tree API ──────────────────────────────────

export interface SoundEntry {
  id: string;
  prompt: string;
  sound_type: string;
  duration_ms: number;
  sample_rate: number;
  rms: number;
  peak: number;
  spectral_centroid: number;
  tags: string;
  is_favorite: boolean;
  source: string;
  variant_name: string | null;
  model: string;
  seed: number;
  created_at: string;
}

export interface ProvenanceInfo {
  id: string;
  prompt: string;
  sound_type: string;
  duration_ms: number;
  model: string;
  seed: number;
  file_hash: string;
  parent_id: string;
  created_at: string;
  source: string;
  export_count: number;
  last_exported_at: string | null;
}

export async function getSoundChildren(soundId: string): Promise<SoundEntry[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_sound_children", { soundId });
}

export async function getSoundProvenance(soundId: string): Promise<ProvenanceInfo> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_sound_provenance", { soundId });
}

// ─── Recipe API ────────────────────────────────────────

export interface RecipeEntry {
  id: string;
  title: string;
  prompt_template: string;
  description: string;
  category: string;
  tags: string[];
  transform_defaults: string;
  is_builtin: boolean;
  usage_count: number;
  is_favorite: boolean;
  created_at: string;
  updated_at: string;
}

export async function createRecipe(
  title: string,
  promptTemplate: string,
  description?: string,
  category?: string,
  tags?: string[],
  transformDefaults?: string,
): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("create_recipe", {
    title,
    promptTemplate,
    description: description || null,
    category: category || null,
    tags: tags || null,
    transformDefaults: transformDefaults || null,
  });
}

export async function updateRecipe(
  id: string,
  title: string,
  promptTemplate: string,
  description?: string,
  category?: string,
  tags?: string[],
  transformDefaults?: string,
): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("update_recipe", {
    id,
    title,
    promptTemplate,
    description: description || null,
    category: category || null,
    tags: tags || null,
    transformDefaults: transformDefaults || null,
  });
}

export async function deleteRecipe(recipeId: string): Promise<boolean> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("delete_recipe", { recipeId });
}

export async function listRecipes(): Promise<RecipeEntry[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("list_recipes");
}

export async function searchRecipes(query: string): Promise<RecipeEntry[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("search_recipes", { query });
}

export async function toggleRecipeFavorite(recipeId: string): Promise<boolean> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("toggle_recipe_favorite", { recipeId });
}

export async function incrementRecipeUsage(recipeId: string): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("increment_recipe_usage", { recipeId });
}

export async function generateFromRecipe(recipeId: string, promptOverride?: string): Promise<SoundResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("generate_from_recipe", { recipeId, promptOverride: promptOverride || null });
}

// ─── Import API ────────────────────────────────────────

export interface ImportResult {
  id: string;
  filename: string;
  duration_ms: number;
  sample_rate: number;
  channels: number;
  file_size_bytes: number;
  rms: number;
  peak: number;
  tags: string[];
  waveform: number[];
}

export async function importSample(path: string): Promise<ImportResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("import_sample", { path });
}

// ─── Folder Import API ────────────────────────────────

export interface FolderSampleInfo {
  path: string;
  filename: string;
  duration_ms: number;
  file_size_bytes: number;
  format: string;
  is_duplicate: boolean;
}

export interface FolderScanResult {
  total_files: number;
  valid_files: number;
  oversized_files: number;
  unsupported_files: number;
  duplicates: string[];
  samples: FolderSampleInfo[];
}

export async function scanFolderImport(path: string): Promise<FolderScanResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("scan_folder_import", { path });
}

// ─── Duplicate Detection API ──────────────────────────

export interface DuplicateGroup {
  hash: string;
  sounds: SoundEntry[];
}

export async function findDuplicates(): Promise<DuplicateGroup[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("find_duplicates");
}

// ─── Library Stats API ────────────────────────────────

export interface LibraryStats {
  total_sounds: number;
  total_favorites: number;
  total_exports: number;
  total_packs: number;
  total_imported: number;
  disk_bytes: number;
}

export async function getLibraryStats(): Promise<LibraryStats> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_library_stats");
}

// ─── Pack Notes API ────────────────────────────────────

export async function updatePackNotes(packId: string, notes: string): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("update_pack_notes", { packId, notes });
}

export async function updatePackMetadata(
  packId: string,
  title: string,
  description?: string,
  tags?: string[],
  notes?: string,
): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("update_pack_metadata", {
    packId,
    title,
    description: description || null,
    tags: tags || null,
    notes: notes || null,
  });
}
