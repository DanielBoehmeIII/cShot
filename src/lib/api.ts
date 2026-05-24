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

// ─── Prompt Analysis API ────────────────────────────────

export interface DetectedDescriptor {
  word: string;
  category: string;
  confidence: number;
  description: string;
}

export interface PromptDspControls {
  sound_type: string;
  sound_type_score: number;
  attack_ms: number | null;
  decay_ms: number | null;
  tail_ms: number | null;
  duration_ms: number | null;
  pitch_hz: number | null;
  pitch_drop_ratio: number | null;
  noise_amount: number | null;
  saturation_drive: number | null;
  brightness: number | null;
  sub_gain: number | null;
  click_amount: number | null;
  transient_boost: number | null;
  body_gain: number | null;
  density: number | null;
  aggressiveness: number | null;
  warmth: number | null;
  crunch: number | null;
  texture: number | null;
  stereo_width: number | null;
  tonal_noise_balance: number | null;
  descriptors: DetectedDescriptor[];
  genre_hints: string[];
  bpm: number | null;
  compound_parts: CompoundEditPart[];
}

export interface CompoundEditPart {
  text: string;
  descriptors: string[];
  is_exclusion: boolean;
}

export async function analyzePrompt(prompt: string): Promise<PromptDspControls> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("analyze_prompt", { prompt });
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

export async function generateResynthesisVariants(
  prompt: string,
  count: number,
): Promise<VariantResult[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("generate_resynthesis_variants", { prompt, count });
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

// ─── Morph API ─────────────────────────────────────────────

export interface MorphControls {
  amount: number;
  preserve_source_identity: number;
  exaggerate: number;
  preserve_transient: number;
  preserve_body: number;
  preserve_tail: number;
  transient_transfer: number;
  tail_transfer: number;
  tonal_blend: number;
  texture_blend: number;
}

export interface MorphResult {
  sound_result: SoundResult;
  similarity: SimilarityReport;
}

export async function morphSounds(
  soundAId: string,
  soundBId: string,
  controls: MorphControls,
): Promise<MorphResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("morph_sounds_command", {
    soundAId,
    soundBId,
    amount: controls.amount,
    preserveSourceIdentity: controls.preserve_source_identity,
    exaggerate: controls.exaggerate,
    preserveTransient: controls.preserve_transient,
    preserveBody: controls.preserve_body,
    preserveTail: controls.preserve_tail,
    transientTransfer: controls.transient_transfer,
    tailTransfer: controls.tail_transfer,
    tonalBlend: controls.tonal_blend,
    textureBlend: controls.texture_blend,
  });
}

// ─── Exploration API ──────────────────────────────────────

export async function exploreStream(
  prompt: string,
  count: number,
  variation: number,
): Promise<VariantResult[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("explore_stream", { prompt, count, variation });
}

export async function branchFromSound(
  soundId: string,
  mutationName: string,
  intensity: number,
): Promise<SoundResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("branch_from_sound", { soundId, mutationName, intensity });
}

export async function recipeRoulette(): Promise<SoundResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("recipe_roulette");
}

export async function quickCompare(
  prompt: string,
  count: number,
): Promise<VariantResult[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("quick_compare", { prompt, count });
}

// ─── Kit Analysis API ─────────────────────────────────────

export interface KitAdvice {
  missing_roles: string[];
  duplicate_roles: [string, number][];
  tonal_issues: string[];
  transient_issues: string[];
  balance_rating: number;
  recommendations: string[];
}

export async function analyzeKit(soundIds: string[]): Promise<KitAdvice> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("analyze_kit_command", { soundIds });
}

// ─── Design Workflow API ──────────────────────────────────

export interface DesignWorkflowResult {
  recreation: SoundResult | null;
  mutation: SoundResult | null;
  morphed: SoundResult | null;
  branched: VariantResult[];
  recreation_similarity: SimilarityReport | null;
  workflow_steps: string[];
  total_time_ms: number;
}

export async function designWorkflow(
  sourceSoundId: string,
  options: {
    referenceSoundId?: string;
    prompt?: string;
    doRecreate?: boolean;
    doMutate?: boolean;
    doMorph?: boolean;
    doBranch?: boolean;
    recreateFidelity?: number;
    mutationName?: string;
    mutationIntensity?: number;
    morphAmount?: number;
    branchCount?: number;
  },
): Promise<DesignWorkflowResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("design_workflow", {
    sourceSoundId,
    referenceSoundId: options.referenceSoundId || null,
    prompt: options.prompt || null,
    doRecreate: options.doRecreate ?? true,
    doMutate: options.doMutate ?? false,
    doMorph: options.doMorph ?? false,
    doBranch: options.doBranch ?? false,
    recreateFidelity: options.recreateFidelity ?? null,
    mutationName: options.mutationName || null,
    mutationIntensity: options.mutationIntensity ?? null,
    morphAmount: options.morphAmount ?? null,
    branchCount: options.branchCount ?? null,
  });
}

// ─── Audio Analysis API ────────────────────────────────────

export interface AudioAnalysis {
  duration_ms: number;
  sample_rate: number;
  channels: number;
  peak: number;
  rms: number;
  crest_factor: number;
  loudness_lufs: number;
  noise_floor_db: number;
  attack_ms: number;
  decay_ms: number;
  tail_ms: number;
  envelope: number[];
  has_leading_silence: boolean;
  has_trailing_silence: boolean;
  leading_silence_ms: number;
  trailing_silence_ms: number;
  spectral_centroid: number;
  spectral_rolloff: number;
  brightness: number;
  zero_crossing_rate: number;
  sub_energy_ratio: number;
  noise_estimate: number;
  transient_strength: number;
  transient_count: number;
  onset_times_ms: number[];
  pitch_estimate: number | null;
  has_pitch: boolean;
  has_clipping: boolean;
  clipping_count: number;
  is_silent: boolean;
  spectral_profile: number[];
  sound_type_hint?: string;
}

export async function getAudioAnalysis(soundId: string): Promise<AudioAnalysis> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_audio_analysis", { soundId });
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

// ─── Transform API ─────────────────────────────────────

export async function transformSound(
  soundId: string,
  prompt: string,
): Promise<SoundResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("transform_sound", { soundId, prompt });
}

// ─── Hybrid Reconstruction API ─────────────────────────

export interface HybridResult {
  sound_result: SoundResult;
  analysis: AudioAnalysis;
}

export async function hybridReconstruct(
  soundId: string,
  synthBlend?: number,
  replaceTransient?: boolean,
  replaceBody?: boolean,
  replaceTail?: boolean,
  subReinforce?: number,
): Promise<HybridResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("hybrid_reconstruct", {
    soundId,
    synthBlend: synthBlend ?? null,
    replaceTransient: replaceTransient ?? null,
    replaceBody: replaceBody ?? null,
    replaceTail: replaceTail ?? null,
    regenerateTail: null,
    subReinforce: subReinforce ?? null,
    preserveTransient: null,
    preserveTail: null,
    preservePitch: null,
  });
}

// ─── Recreation API ─────────────────────────────────────

export interface SimilarityReport {
  overall: number;
  envelope_match: number;
  spectral_match: number;
  rms_match: number;
  transient_match: number;
  duration_match: number;
}

export interface RecreateApproximation {
  id: string;
  sound_result: SoundResult;
  similarity: SimilarityReport;
}

export interface RecreateResult {
  approximations: RecreateApproximation[];
  original_analysis: AudioAnalysis;
}

export async function recreateSound(
  soundId: string,
  count?: number,
  fidelity?: number,
  preserveTransient?: boolean,
  preserveBody?: boolean,
  preserveTail?: boolean,
): Promise<RecreateResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("recreate_sound", {
    soundId,
    count: count ?? null,
    fidelity: fidelity ?? null,
    preserveTransient: preserveTransient ?? null,
    preserveBody: preserveBody ?? null,
    preserveTail: preserveTail ?? null,
  });
}

// ─── Resynthesis API ─────────────────────────────────────

export async function resynthesizeSound(soundId: string): Promise<SoundResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("resynthesize_sound", { soundId });
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

export type RepairAction = "Normalize" | "TrimSilence" | "Fade" | "Shorten" | "Brighten" | "Darken" | "Punch" | "AddSub" | "Saturation" | "Soften" | "Sharpen" | "Compress";

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

// ─── Undo / Redo / Iteration API ────────────────────────

export async function recordGenerationAction(soundId: string, prompt: string): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("record_generation_action", { soundId, prompt });
}

export async function undoLastAction(): Promise<{ action_type: string; sound_id: string; prompt: string; timestamp: string } | null> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("undo_last_action");
}

export async function redoLastAction(): Promise<{ action_type: string; sound_id: string; prompt: string; timestamp: string } | null> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("redo_last_action");
}

export async function canUndoAction(): Promise<boolean> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("can_undo_action");
}

export async function canRedoAction(): Promise<boolean> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("can_redo_action");
}

export async function getSoundDuration(soundId: string): Promise<number> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_sound_duration", { soundId });
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

// ─── Creative Intent API ─────────────────────────────────

export interface CreativeIntentProfile {
  energy: number;
  aggression: number;
  polish: number;
  realism: number;
  experimentalism: number;
  analog_feel: number;
  cinematic_scale: number;
  density: number;
  impact: number;
}

export interface IntentGenerateResult {
  sound: SoundResult;
  profile: CreativeIntentProfile;
  params_summary: string;
}

export async function generateWithIntent(
  prompt: string,
  intentProfile: CreativeIntentProfile,
  referencePath?: string,
): Promise<IntentGenerateResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("generate_with_intent", {
    prompt,
    intentProfile,
    referencePath: referencePath || null,
  });
}

export async function getIntentPresets(): Promise<[string, CreativeIntentProfile][]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_intent_presets");
}

export async function blendIntentProfiles(
  profileA: CreativeIntentProfile,
  profileB: CreativeIntentProfile,
  blend: number,
): Promise<CreativeIntentProfile> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("blend_intent_profiles", { profileA, profileB, blend });
}

// ─── Evolution API ──────────────────────────────────────

export interface RegionLock {
  lock_transient: boolean;
  lock_body: boolean;
  lock_tail: boolean;
  lock_sub: boolean;
  lock_noise: boolean;
}

export interface TraitPreference {
  preferred: string[];
  disliked: string[];
}

export interface EvolutionConfig {
  generations: number;
  population_size: number;
  mutation_rate: number;
  crossover_rate: number;
  quality_bias: number;
  novelty_bias: number;
  preserve_best: boolean;
  elite_count: number;
  region_lock: RegionLock;
  trait_preference: TraitPreference;
}

export interface EvolutionMember {
  id: string;
  generation: number;
  parent_ids: string[];
  quality_score: number;
  novelty_score: number;
  score: number;
  is_elite: boolean;
}

export interface EvolutionSnapshot {
  generation: number;
  member_count: number;
  best_score: number;
  avg_score: number;
  best_id: string;
  best_novelty: number;
}

export interface EvolutionState {
  generation: number;
  parent_id: string;
  parent_prompt: string;
  population: EvolutionMember[];
  history: EvolutionSnapshot[];
  config: EvolutionConfig;
  sound_type: string;
  quality_score: number;
}

export async function evolveSoundCommand(
  soundId: string,
  config: EvolutionConfig,
  targetDirection?: string,
  directionIntensity?: number,
): Promise<EvolutionState> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("evolve_sound_command", {
    soundId,
    config,
    targetDirection: targetDirection || null,
    directionIntensity: directionIntensity || null,
  });
}

export async function saveEvolutionMember(
  member: EvolutionMember,
  soundType: string,
  prompt: string,
): Promise<SoundResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("save_evolution_member", { member, soundType, prompt });
}

// ─── Library Optimization API ───────────────────────────

export interface LibraryOptimizationStats {
  total_count: number;
  favorite_count: number;
  avg_duration_ms: number;
  unique_types: [string, number][];
  total_size_bytes: number;
  estimated_import_count: number;
}

export async function getLibraryOptimizationStats(): Promise<LibraryOptimizationStats> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_library_optimization_stats");
}

export async function getLibraryCache(key: string): Promise<string | null> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_library_cache", { key });
}

export async function setLibraryCache(key: string, value: string): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("set_library_cache", { key, value });
}

// ─── Semantic Search API ────────────────────────────────

export interface SemanticQuery {
  target_type: string | null;
  target_descriptors: string[];
  target_genre: string | null;
  bpm: number | null;
  duration_range: [number, number] | null;
  raw_query: string;
}

export async function parseNaturalLanguageQuery(query: string): Promise<SemanticQuery> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("parse_natural_language_query", { query });
}

export interface SemanticSearchResult {
  entry: SoundEntry;
  similarity_score: number;
  match_reasons: string[];
}

export async function semanticSearch(query: string): Promise<SemanticSearchResult[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("semantic_search", { query });
}

// ─── Workflow Automation API ────────────────────────────

export interface AutoTags {
  sound_type: string;
  descriptors: string[];
  genre_hints: string[];
  mix_role: string;
  energy_level: string;
  duration_label: string;
}

export interface PackSuggestion {
  title: string;
  sound_count: number;
  has_kick: boolean;
  has_snare: boolean;
  has_hat: boolean;
  has_clap: boolean;
  has_perc: boolean;
  has_bass: boolean;
  has_fx: boolean;
  missing_roles: string[];
  cohesion_score: number;
}

export async function autoTagSound(soundId: string): Promise<AutoTags> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("auto_tag_sound", { soundId });
}

export async function suggestPack(soundIds: string[]): Promise<PackSuggestion> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("suggest_pack_command", { soundIds });
}

export async function suggestFilename(soundId: string): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("suggest_filename_command", { soundId });
}

// ─── Advanced Recreation API ────────────────────────────

export interface AdvancedRecreationConfig {
  mode: string;
  fidelity: number;
  transient_preservation: number;
  body_preservation: number;
  tail_preservation: number;
  spectral_matching: number;
  sub_reconstruction: number;
  transient_timing_align: boolean;
  harmonic_profile_match: boolean;
  tail_texture_match: boolean;
}

export async function recreateAdvanced(
  soundId: string,
  config: AdvancedRecreationConfig,
): Promise<[SoundResult, any]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("recreate_advanced_command", { soundId, config });
}

// ─── Sculpting API ──────────────────────────────────────

export interface SculptControls {
  transient_intensity: number;
  tail_length: number;
  brightness: number;
  distortion: number;
  density: number;
  tonal_noise_balance: number;
  sub_amount: number;
  body_thickness: number;
  attack_sharpness: number;
  stereo_width: number;
}

export async function sculptSound(
  soundId: string,
  controls: SculptControls,
): Promise<SoundResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("sculpt_sound", { soundId, controls });
}

export async function sculptPreview(
  soundId: string,
  controls: SculptControls,
): Promise<number[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("sculpt_preview", { soundId, controls });
}

// ─── Instrument Presets API ─────────────────────────────

export interface MacroTarget {
  param: string;
  scale: number;
  offset: number;
}

export interface MacroMapping {
  macro_index: number;
  name: string;
  targets: MacroTarget[];
  min: number;
  max: number;
}

export interface InstrumentPreset {
  name: string;
  category: string;
  sound_type: string;
  macro_mappings: MacroMapping[];
  description: string;
}

export async function getBuiltinPresets(): Promise<InstrumentPreset[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_builtin_presets");
}

export async function applyPresetMacro(
  preset: InstrumentPreset,
  macroState: { morph_position: number; macro_values: number[]; random_lock: string[] },
): Promise<InstrumentPreset> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("apply_preset_macro", { preset, macroState });
}

// ─── Stress Test & Validation API ───────────────────────

export interface StressTestResult {
  total_tests: number;
  passed: number;
  failed: number;
  avg_generation_time_ms: number;
  max_generation_time_ms: number;
  min_generation_time_ms: number;
  silent_outputs: number;
  clipped_outputs: number;
  errors: string[];
}

export interface ExportValidation {
  is_valid: boolean;
  duration_ms: number;
  sample_rate: number;
  peak: number;
  rms: number;
  has_clipping: boolean;
  is_silent: boolean;
  dc_offset: number;
  warnings: string[];
}

export async function runStressTest(iterations: number): Promise<StressTestResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("run_stress_test", { iterations });
}

export async function validateExport(soundId: string): Promise<ExportValidation> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("validate_export_command", { soundId });
}

export async function batchValidateExports(soundIds: string[]): Promise<[string, ExportValidation][]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("batch_validate_exports", { soundIds });
}

// ─── Audio Intelligence API ─────────────────────────────

export async function analyzeAudioIntelligence(soundId: string): Promise<any> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("analyze_audio_intelligence", { soundId });
}

// ─── OneShot API ───────────────────────────────────────────────

export interface OneShotControls {
  brightness: number;
  punch: number;
  decay: number;
  distortion: number;
  transient_amount: number;
  noise_amount: number;
  body_amount: number;
  stereo_width: number;
  pitch_drop: number;
  filter_sweep: number;
}

export interface OneShotSpec {
  sound_class: string;
  duration_ms: number;
  pitch_hz: number;
  gain: number;
  controls: OneShotControls | null;
}

export interface SoundClassInfo {
  value: string;
  label: string;
  description: string;
}

export interface OneshotPreviewResult {
  samples: number[];
  duration_ms: number;
  peak: number;
  rms: number;
  sample_rate: number;
}

export interface OneshotExportResult {
  path: string;
  filename: string;
  file_size_bytes: number;
}

export const DEFAULT_ONESHOT_CONTROLS: OneShotControls = {
  brightness: 0.5,
  punch: 0.5,
  decay: 0.5,
  distortion: 0.5,
  transient_amount: 0.5,
  noise_amount: 0.5,
  body_amount: 0.5,
  stereo_width: 0.5,
  pitch_drop: 0.5,
  filter_sweep: 0.5,
};

export async function listSoundClasses(): Promise<SoundClassInfo[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("list_sound_classes");
}

export async function getOneshotDefaults(soundClass: string): Promise<OneShotSpec> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_oneshot_defaults", { soundClass });
}

export async function renderOneshot(
  soundClass: string,
  durationMs: number,
  pitchHz: number,
  gain: number,
  controls: OneShotControls | null,
): Promise<OneshotPreviewResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("render_oneshot", {
    soundClass,
    durationMs,
    pitchHz,
    gain,
    controls,
  });
}

export async function exportOneshotWav(
  soundClass: string,
  durationMs: number,
  pitchHz: number,
  gain: number,
  controls: OneShotControls | null,
  outputPath: string,
): Promise<OneshotExportResult> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("export_oneshot_wav", {
    soundClass,
    durationMs,
    pitchHz,
    gain,
    controls,
    outputPath,
  });
}
