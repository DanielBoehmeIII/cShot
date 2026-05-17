use super::analyze::{analyze_audio, AudioAnalysis};
use super::resynthesize::{self, ResynthesisParams};
use super::recreate::{self, compute_similarity, SimilarityReport, params_from_analysis};
use super::mutation;
use super::SoundType;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegionLock {
    pub lock_transient: bool,
    pub lock_body: bool,
    pub lock_tail: bool,
    pub lock_sub: bool,
    pub lock_noise: bool,
}

impl Default for RegionLock {
    fn default() -> Self {
        Self {
            lock_transient: false,
            lock_body: false,
            lock_tail: false,
            lock_sub: false,
            lock_noise: false,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TraitPreference {
    pub preferred: Vec<String>,
    pub disliked: Vec<String>,
}

impl Default for TraitPreference {
    fn default() -> Self {
        Self {
            preferred: Vec::new(),
            disliked: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EvolutionConfig {
    pub generations: usize,
    pub population_size: usize,
    pub mutation_rate: f32,
    pub crossover_rate: f32,
    pub quality_bias: f32,
    pub novelty_bias: f32,
    pub preserve_best: bool,
    pub elite_count: usize,
    pub region_lock: RegionLock,
    pub trait_preference: TraitPreference,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            generations: 5,
            population_size: 8,
            mutation_rate: 0.3,
            crossover_rate: 0.2,
            quality_bias: 0.6,
            novelty_bias: 0.3,
            preserve_best: true,
            elite_count: 2,
            region_lock: RegionLock::default(),
            trait_preference: TraitPreference::default(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EvolutionState {
    pub generation: usize,
    pub parent_id: String,
    pub parent_prompt: String,
    pub population: Vec<EvolutionMember>,
    pub history: Vec<EvolutionSnapshot>,
    pub config: EvolutionConfig,
    pub sound_type: String,
    pub quality_score: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EvolutionMember {
    pub id: String,
    pub generation: usize,
    pub parent_ids: Vec<String>,
    pub samples: Vec<f32>,
    pub params: ResynthesisParams,
    pub similarity_to_parent: SimilarityReport,
    pub quality_score: f32,
    pub novelty_score: f32,
    pub score: f32,
    pub is_elite: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EvolutionSnapshot {
    pub generation: usize,
    pub member_count: usize,
    pub best_score: f32,
    pub avg_score: f32,
    pub best_id: String,
    pub best_novelty: f32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EvolveStepRequest {
    pub parent_samples: Vec<f32>,
    pub parent_analysis: AudioAnalysis,
    pub parent_prompt: String,
    pub parent_id: String,
    pub config: EvolutionConfig,
    pub target_direction: Option<String>,
    pub direction_intensity: f32,
}

pub fn run_evolution(request: &EvolveStepRequest) -> EvolutionState {
    let config = &request.config;
    let analysis = &request.parent_analysis;
    let st = SoundType::from_str(&analysis.sound_type_hint);

    let base_params = params_from_analysis(&analysis, &request.parent_samples);
    let parent_score = compute_quality_score(&request.parent_samples, analysis);

    let mut population: Vec<EvolutionMember> = Vec::new();
    let mut history: Vec<EvolutionSnapshot> = Vec::new();

    for g in 0..config.generations.max(1) {
        let mut new_population: Vec<EvolutionMember> = Vec::new();

        if g == 0 {
            for i in 0..config.population_size {
                let member = if let Some(ref dir) = request.target_direction {
                    if i == 0 {
                        create_member_from_params(
                            st, &base_params, analysis, &request.parent_samples,
                            g, vec![request.parent_id.clone()],
                        )
                    } else {
                        evolve_direction(
                            &request.parent_samples, analysis, dir,
                            request.direction_intensity * (0.5 + (i as f32 / config.population_size as f32) * 0.5),
                            g, &request.parent_id,
                        )
                    }
                } else {
                    let variance = 0.2 + (i as f32 / config.population_size as f32) * 0.6;
                    let mutated = mutate_params(&base_params, variance, &config.region_lock, &config.trait_preference);
                    let samples = resynthesize::resynthesize(&mutated);
                    let sim = compute_similarity(&request.parent_samples, &samples, analysis);
                    let quality = compute_quality_score(&samples, analysis);
                    let novelty = 1.0 - sim.overall;
                    let score = compute_evolution_score(quality, novelty, config);

                    EvolutionMember {
                        id: uuid::Uuid::new_v4().to_string(),
                        generation: g,
                        parent_ids: vec![request.parent_id.clone()],
                        samples,
                        params: mutated,
                        similarity_to_parent: sim,
                        quality_score: quality,
                        novelty_score: novelty,
                        score,
                        is_elite: false,
                    }
                };
                new_population.push(member);
            }
        } else {
            let elites = select_elites(&population, config.elite_count);
            let mut offspring: Vec<EvolutionMember> = Vec::new();

            for elite in &elites {
                if config.preserve_best {
                    let mut m = elite.clone();
                    m.generation = g;
                    m.is_elite = true;
                    offspring.push(m);
                }
            }

            let needed = config.population_size.saturating_sub(offspring.len());
            for i in 0..needed {
                let parent_a = tournament_select(&population);
                let parent_b = tournament_select(&population);

                let new_member = if fast_rng(i as u64) < config.crossover_rate {
                    let crossed = crossover_params(&parent_a.params, &parent_b.params);
                    let samples = resynthesize::resynthesize(&crossed);
                    let sim = compute_similarity(&request.parent_samples, &samples, analysis);
                    let quality = compute_quality_score(&samples, analysis);
                    let novelty = 1.0 - sim.overall;
                    let score = compute_evolution_score(quality, novelty, config);

                    EvolutionMember {
                        id: uuid::Uuid::new_v4().to_string(),
                        generation: g,
                        parent_ids: vec![parent_a.id.clone(), parent_b.id.clone()],
                        samples,
                        params: crossed,
                        similarity_to_parent: sim,
                        quality_score: quality,
                        novelty_score: novelty,
                        score,
                        is_elite: false,
                    }
                } else {
                    let mutated = mutate_params(&parent_a.params, config.mutation_rate, &config.region_lock, &config.trait_preference);
                    let samples = resynthesize::resynthesize(&mutated);
                    let sim = compute_similarity(&request.parent_samples, &samples, analysis);
                    let quality = compute_quality_score(&samples, analysis);
                    let novelty = 1.0 - sim.overall;
                    let score = compute_evolution_score(quality, novelty, config);

                    EvolutionMember {
                        id: uuid::Uuid::new_v4().to_string(),
                        generation: g,
                        parent_ids: vec![parent_a.id.clone()],
                        samples,
                        params: mutated,
                        similarity_to_parent: sim,
                        quality_score: quality,
                        novelty_score: novelty,
                        score,
                        is_elite: false,
                    }
                };
                offspring.push(new_member);
            }

            new_population = offspring;
        }

        population = new_population;
        population.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        if config.preserve_best {
            let elite_end = config.elite_count.min(population.len());
            for member in population.iter_mut().take(elite_end) {
                member.is_elite = true;
            }
        }

        let best = population.first().map(|m| m.score).unwrap_or(0.0);
        let avg = if !population.is_empty() {
            population.iter().map(|m| m.score).sum::<f32>() / population.len() as f32
        } else {
            0.0
        };
        let best_novelty = population.first().map(|m| m.novelty_score).unwrap_or(0.0);
        let best_id = population.first().map(|m| m.id.clone()).unwrap_or_default();

        history.push(EvolutionSnapshot {
            generation: g,
            member_count: population.len(),
            best_score: best,
            avg_score: avg,
            best_id,
            best_novelty,
        });
    }

    EvolutionState {
        generation: config.generations,
        parent_id: request.parent_id.clone(),
        parent_prompt: request.parent_prompt.clone(),
        population,
        history,
        config: config.clone(),
        sound_type: analysis.sound_type_hint.clone(),
        quality_score: parent_score,
    }
}

fn create_member_from_params(
    st: SoundType,
    base_params: &ResynthesisParams,
    analysis: &AudioAnalysis,
    parent_samples: &[f32],
    generation: usize,
    parent_ids: Vec<String>,
) -> EvolutionMember {
    let samples = resynthesize::resynthesize(base_params);
    let sim = compute_similarity(parent_samples, &samples, analysis);
    let quality = compute_quality_score(&samples, analysis);
    let novelty = 1.0 - sim.overall;
    EvolutionMember {
        id: uuid::Uuid::new_v4().to_string(),
        generation,
        parent_ids,
        samples,
        params: base_params.clone(),
        similarity_to_parent: sim,
        quality_score: quality,
        novelty_score: novelty,
        score: (quality + novelty) * 0.5,
        is_elite: false,
    }
}

fn evolve_direction(
    samples: &[f32],
    analysis: &AudioAnalysis,
    direction: &str,
    intensity: f32,
    generation: usize,
    parent_id: &str,
) -> EvolutionMember {
    let (evolved, sim) = mutation::evolve_sound(samples, analysis, intensity, direction);
    let quality = compute_quality_score(&evolved, analysis);
    let novelty = 1.0 - sim.overall;
    let base_params = params_from_analysis(analysis, samples);
    EvolutionMember {
        id: uuid::Uuid::new_v4().to_string(),
        generation,
        parent_ids: vec![parent_id.to_string()],
        samples: evolved,
        params: base_params,
        similarity_to_parent: sim,
        quality_score: quality,
        novelty_score: novelty,
        score: (quality + novelty) * 0.5,
        is_elite: false,
    }
}

fn mutate_params(
    params: &ResynthesisParams,
    rate: f32,
    region_lock: &RegionLock,
    _trait_preference: &TraitPreference,
) -> ResynthesisParams {
    let mut p = params.clone();
    let seed = rand_seed();

    p = p.with_seed((rate * 10000.0) as u64 + seed);

    if !region_lock.lock_transient {
        p.click_amount = (p.click_amount + (fast_rng(seed + 1) - 0.5) * rate * 0.6).clamp(0.0, 1.0);
        p.attack_ms = (p.attack_ms * (1.0 + (fast_rng(seed + 2) - 0.5) * rate * 0.5)).max(0.3);
    }

    if !region_lock.lock_body {
        p.body_gain = (p.body_gain + (fast_rng(seed + 3) - 0.5) * rate * 0.4).clamp(0.0, 1.0);
        p.decay_ms = (p.decay_ms * (1.0 + (fast_rng(seed + 4) - 0.5) * rate * 0.4)).max(5.0);
    }

    if !region_lock.lock_tail {
        p.tail_ms = (p.tail_ms * (1.0 + (fast_rng(seed + 5) - 0.5) * rate * 0.6)).max(0.0);
        p.duration_ms = (p.duration_ms * (1.0 + (fast_rng(seed + 6) - 0.5) * rate * 0.3)).max(20.0);
    }

    if !region_lock.lock_sub {
        p.sub_gain = (p.sub_gain + (fast_rng(seed + 7) - 0.5) * rate * 0.4).clamp(0.0, 1.0);
    }

    if !region_lock.lock_noise {
        p.noise_amount = (p.noise_amount + (fast_rng(seed + 8) - 0.5) * rate * 0.4).clamp(0.0, 1.0);
    }

    p.saturation_drive = (p.saturation_drive + (fast_rng(seed + 9) - 0.5) * rate * 0.6).max(1.0);
    p.brightness = (p.brightness + (fast_rng(seed + 10) - 0.5) * rate * 0.3).clamp(0.0, 1.0);

    p
}

fn crossover_params(a: &ResynthesisParams, b: &ResynthesisParams) -> ResynthesisParams {
    let seed = rand_seed();
    let mut p = a.clone();

    if fast_rng(seed + 0) > 0.5 { p.click_amount = b.click_amount; }
    if fast_rng(seed + 1) > 0.5 { p.attack_ms = b.attack_ms; }
    if fast_rng(seed + 2) > 0.5 { p.body_gain = b.body_gain; }
    if fast_rng(seed + 3) > 0.5 { p.decay_ms = b.decay_ms; }
    if fast_rng(seed + 4) > 0.5 { p.tail_ms = b.tail_ms; }
    if fast_rng(seed + 5) > 0.5 { p.sub_gain = b.sub_gain; }
    if fast_rng(seed + 6) > 0.5 { p.noise_amount = b.noise_amount; }
    if fast_rng(seed + 7) > 0.5 { p.saturation_drive = b.saturation_drive; }
    if fast_rng(seed + 8) > 0.5 { p.brightness = b.brightness; }
    if fast_rng(seed + 9) > 0.5 { p.duration_ms = b.duration_ms; }

    p
}

fn tournament_select(population: &[EvolutionMember]) -> &EvolutionMember {
    let seed = rand_seed();
    let idx_a = (fast_rng(seed) * population.len() as f32) as usize % population.len().max(1);
    let idx_b = (fast_rng(seed + 1) * population.len() as f32) as usize % population.len().max(1);
    if population[idx_a].score >= population[idx_b].score {
        &population[idx_a]
    } else {
        &population[idx_b]
    }
}

fn select_elites(population: &[EvolutionMember], count: usize) -> Vec<EvolutionMember> {
    let mut sorted = population.to_vec();
    sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    sorted.truncate(count.min(sorted.len()));
    sorted
}

fn compute_quality_score(samples: &[f32], analysis: &AudioAnalysis) -> f32 {
    let crest = if analysis.rms > 0.0 { analysis.peak / analysis.rms } else { 1.0 };
    let has_content = analysis.peak > 0.01;
    let transient_quality = if has_content && crest > 3.0 { (crest / 20.0).min(1.0) } else { 0.3 };
    let noise_quality = 1.0 - analysis.noise_estimate.min(0.8);
    let balance = 1.0 - (analysis.brightness - 0.5).abs() * 1.5;
    let duration_ok = if analysis.duration_ms > 30.0 && analysis.duration_ms < 5000.0 { 1.0 } else { 0.3 };
    let clip_ok = if !analysis.has_clipping { 1.0 } else { 0.3 };

    transient_quality * 0.3 + noise_quality * 0.15 + balance.clamp(0.0, 1.0) * 0.2
        + duration_ok * 0.15 + clip_ok * 0.2
}

fn compute_evolution_score(quality: f32, novelty: f32, config: &EvolutionConfig) -> f32 {
    quality * config.quality_bias + novelty * config.novelty_bias
}

fn rand_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

fn fast_rng(seed: u64) -> f32 {
    let x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((x >> 33) as u32) as f32 / u32::MAX as f32
}

pub fn evolve_step(
    parent_samples: &[f32],
    analysis: &AudioAnalysis,
    parent_prompt: &str,
    parent_id: &str,
    generation: usize,
    config: &EvolutionConfig,
) -> Vec<EvolutionMember> {
    let request = EvolveStepRequest {
        parent_samples: parent_samples.to_vec(),
        parent_analysis: analysis.clone(),
        parent_prompt: parent_prompt.to_string(),
        parent_id: parent_id.to_string(),
        config: config.clone(),
        target_direction: None,
        direction_intensity: 0.5,
    };
    let state = run_evolution(&request);
    let best = state.population.into_iter().take(config.population_size).collect::<Vec<_>>();
    best
}

// ─── Infinite Variant Machine ─────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ExplorationMode {
    Safe,
    Balanced,
    Wild,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VariantGenerationConfig {
    pub exploration_mode: ExplorationMode,
    pub novelty_target: f32,
    pub quality_threshold: f32,
    pub count: usize,
    pub direction: Option<String>,
    pub direction_intensity: f32,
    pub constrain_to_genre: Option<String>,
    pub use_favorites: Vec<String>,
    pub branch_from: Option<String>,
    pub seed: u64,
}

impl Default for VariantGenerationConfig {
    fn default() -> Self {
        Self {
            exploration_mode: ExplorationMode::Balanced,
            novelty_target: 0.3,
            quality_threshold: 0.5,
            count: 6,
            direction: None,
            direction_intensity: 0.5,
            constrain_to_genre: None,
            use_favorites: Vec::new(),
            branch_from: None,
            seed: 0,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VariantResult {
    pub id: String,
    pub samples: Vec<f32>,
    pub params: ResynthesisParams,
    pub similarity_to_parent: SimilarityReport,
    pub quality_score: f32,
    pub novelty_score: f32,
    pub score: f32,
    pub direction: Option<String>,
    pub parent_id: String,
    pub generation: usize,
    pub branch_path: Vec<String>,
    pub is_favorite: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VariantTree {
    pub root_id: String,
    pub root_samples: Vec<f32>,
    pub root_params: ResynthesisParams,
    pub branches: Vec<VariantBranch>,
    pub generation_count: usize,
    pub total_variants: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VariantBranch {
    pub branch_id: String,
    pub label: String,
    pub variants: Vec<VariantResult>,
    pub parent_branch_id: Option<String>,
    pub exploration_mode: ExplorationMode,
}

pub fn generate_variants(
    samples: &[f32],
    analysis: &AudioAnalysis,
    config: &VariantGenerationConfig,
) -> Vec<VariantResult> {
    let base_params = params_from_analysis(analysis, samples);
    let mut results = Vec::new();
    let parent_id = uuid::Uuid::new_v4().to_string();

    let (mutation_rate_range, novelty_weight, diversity_spread) = match config.exploration_mode {
        ExplorationMode::Safe => (0.05..0.2, 0.2, 0.3),
        ExplorationMode::Balanced => (0.15..0.4, 0.4, 0.6),
        ExplorationMode::Wild => (0.3..0.8, 0.7, 1.0),
    };

    let seed = if config.seed > 0 { config.seed } else {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    };

    for i in 0..config.count {
        let variant_seed = seed.wrapping_mul(i as u64 + 1).wrapping_add(100);
        let t = i as f32 / config.count.max(1) as f32;

        if let Some(ref direction) = config.direction {
            let intensity = config.direction_intensity * (0.5 + t * diversity_spread);
            let mut p = base_params.clone();
            match direction.as_str() {
                "safer" => {
                    let var_amt = 0.05 + t * 0.1;
                    p = p.with_seed(variant_seed).randomize(var_amt);
                }
                "wilder" => {
                    let var_amt = 0.3 + t * 0.7;
                    p = p.with_seed(variant_seed).randomize(var_amt);
                    p.saturation_drive = (p.saturation_drive + t * 1.0).min(5.0);
                    p.noise_amount = (p.noise_amount + t * 0.3).min(1.0);
                }
                "cleaner" => {
                    p.saturation_drive = 1.0;
                    p.noise_amount = (p.noise_amount * 0.3).max(0.0);
                    p.brightness = (p.brightness + t * 0.2).min(1.0);
                    p = p.with_seed(variant_seed);
                }
                "darker" => {
                    p.brightness = (p.brightness - t * 0.4).max(0.0);
                    p.sub_gain = (p.sub_gain + t * 0.2).min(1.0);
                    p = p.with_seed(variant_seed);
                }
                "more modern" => {
                    p.decay_ms *= 0.6;
                    p.click_amount = (p.click_amount + t * 0.3).min(1.0);
                    p.brightness = (p.brightness + t * 0.2).min(1.0);
                    p.saturation_drive = (p.saturation_drive + t * 0.3).min(3.5);
                    p.noise_amount = (p.noise_amount * 0.4).max(0.0);
                }
                "more experimental" => {
                    let var_amt = 0.4 + t * 0.6;
                    p = p.with_seed(variant_seed).randomize(var_amt);
                    p.saturation_drive = (p.saturation_drive + t * 1.5).min(6.0);
                    p.pitch_hz *= if t < 0.5 { 0.7 } else { 1.4 };
                    p.noise_amount = (p.noise_amount + t * 0.4).min(1.0);
                }
                _ => {
                    p = p.with_seed(variant_seed).randomize(t * 0.3);
                }
            }
            p.seed = variant_seed;

            if let Some(ref genre) = config.constrain_to_genre {
                p = crate::audio::recreate::adapt_params_for_genre(&p, genre);
            }

            let var_samples = crate::audio::resynthesize::resynthesize(&p);
            if var_samples.is_empty() { continue; }

            let sim = compute_similarity(samples, &var_samples, analysis);
            let quality = compute_quality_score(&var_samples, analysis);
            let novelty = 1.0 - sim.overall;
            let score = quality * (1.0 - novelty_weight) + novelty * novelty_weight;
            if quality < config.quality_threshold { continue; }

            results.push(VariantResult {
                id: uuid::Uuid::new_v4().to_string(),
                samples: var_samples,
                params: p,
                similarity_to_parent: sim,
                quality_score: quality,
                novelty_score: novelty,
                score,
                direction: Some(direction.to_string()),
                parent_id: parent_id.clone(),
                generation: 0,
                branch_path: vec![direction.to_string()],
                is_favorite: false,
            });
        } else {
            let var_amt = mutation_rate_range.start + t * (mutation_rate_range.end - mutation_rate_range.start);
            let mut p = base_params.clone().with_seed(variant_seed);
            p = p.randomize(var_amt);

            if let Some(ref genre) = config.constrain_to_genre {
                p = crate::audio::recreate::adapt_params_for_genre(&p, genre);
            }

            let var_samples = crate::audio::resynthesize::resynthesize(&p);
            if var_samples.is_empty() { continue; }

            let sim = compute_similarity(samples, &var_samples, analysis);
            let quality = compute_quality_score(&var_samples, analysis);
            let novelty = 1.0 - sim.overall;
            let score = quality * (1.0 - novelty_weight) + novelty * novelty_weight;
            if quality < config.quality_threshold { continue; }

            results.push(VariantResult {
                id: uuid::Uuid::new_v4().to_string(),
                samples: var_samples,
                params: p,
                similarity_to_parent: sim,
                quality_score: quality,
                novelty_score: novelty,
                score,
                direction: None,
                parent_id: parent_id.clone(),
                generation: 0,
                branch_path: vec![],
                is_favorite: false,
            });
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(config.count);
    results
}

pub fn generate_variant_branch(
    parent: &VariantResult,
    parent_samples: &[f32],
    parent_analysis: &AudioAnalysis,
    count: usize,
    exploration: ExplorationMode,
) -> Vec<VariantResult> {
    let config = VariantGenerationConfig {
        exploration_mode: exploration,
        count,
        direction: None,
        constrain_to_genre: None,
        use_favorites: Vec::new(),
        branch_from: Some(parent.id.clone()),
        ..Default::default()
    };
    let mut children = generate_variants(parent_samples, parent_analysis, &config);
    for child in children.iter_mut() {
        child.parent_id = parent.id.clone();
        child.generation = parent.generation + 1;
        let mut path = parent.branch_path.clone();
        path.push(child.id.clone());
        child.branch_path = path;
    }
    children
}

pub fn filter_variants_by_taste(
    variants: Vec<VariantResult>,
    preferred_traits: &[String],
) -> Vec<VariantResult> {
    if preferred_traits.is_empty() { return variants; }
    let mut scored = variants;
    for v in scored.iter_mut() {
        let mut taste_boost = 0.0f32;
        let snr = v.similarity_to_parent.noise_match;
        let sub = v.similarity_to_parent.sub_match;
        let env = v.similarity_to_parent.envelope_match;
        let spc = v.similarity_to_parent.spectral_match;

        for trait_str in preferred_traits {
            match trait_str.as_str() {
                "punchy" => taste_boost += snr * 0.1 + (1.0 - sub) * 0.1,
                "clean" => taste_boost += (1.0 - snr) * 0.15,
                "dark" => taste_boost += (1.0 - spc) * 0.1 + sub * 0.1,
                "bright" => taste_boost += spc * 0.15,
                "subby" | "sub" => taste_boost += sub * 0.15,
                "experimental" => taste_boost += v.novelty_score * 0.15,
                "safe" => taste_boost += (1.0 - v.novelty_score) * 0.15,
                "tight" => taste_boost += env * 0.1,
                "aggressive" => taste_boost += snr * 0.1 + v.quality_score * 0.05,
                _ => {}
            }
        }
        v.score += taste_boost;
    }
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

pub fn build_variant_tree(root: &VariantResult, branches: Vec<VariantBranch>) -> VariantTree {
    let cloned = branches.clone();
    let total_variants: usize = cloned.iter().map(|b| b.variants.len()).sum::<usize>() + 1;
    let generation_count = cloned.iter().map(|b| {
        b.variants.iter().map(|v| v.generation).max().unwrap_or(0)
    }).max().unwrap_or(0);
    VariantTree {
        root_id: root.id.clone(),
        root_samples: root.samples.clone(),
        root_params: root.params.clone(),
        branches,
        generation_count,
        total_variants,
    }
}
