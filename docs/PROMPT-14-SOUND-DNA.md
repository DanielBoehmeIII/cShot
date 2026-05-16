# Prompt 14 — Sound DNA

cShot evolves sounds like biological organisms.

---

## 1. The Sound DNA Representation

Sound DNA is a **structured genetic encoding** of a one-shot. It decomposes sound into discrete, inheritable, and mutable genetic units.

### 1.1 DNA Structure

```
Sound DNA = {
    # Structural Genes (encode physical sound properties)
    timbre_gene:          Spectral fingerprint (256-dim vector)
    transient_gene:       Transient shape code (128-dim)
    harmonic_gene:        Harmonic profile (64-dim)
    noise_gene:           Noise distribution parameters (64-dim)
    modulation_gene:      Amplitude/frequency modulation behavior (32-dim)
    spectral_evolution:   How spectrum changes over time (128-dim trajectory)
    stereo_gene:          Stereo field parameters (32-dim)
    
    # Expressive Genes (encode higher-level properties)
    emotional_gene:       VAP coordinates + semantic embedding (64-dim)
    genre_gene:           Genre probability distribution (32-dim)
    production_gene:      Mix engineering parameters (32-dim)
    
    # Regulatory Genes (control expression of other genes)
    expression_gene:      Gain/attenuation per gene (16-dim)
    interaction_gene:     How genes interact (16 x 16 interaction matrix)
    
    # Metadata
    lineage:              Ancestry tree (list of parent IDs)
    mutation_history:     List of (mutation_type, location) pairs
    fitness_scores:       History of fitness evaluations
}
```

Total: ~768-dim floating point vector per sound.

### 1.2 Analogy to Biological DNA

| Biological Concept | Sound DNA Equivalent |
|-------------------|---------------------|
| Gene | A specific sound attribute (timbre, transient, etc.) |
| Allele | Variant of a sound attribute |
| Chromosome | One-shot = complete set of genes |
| Genotype | Full DNA encoding |
| Phenotype | The rendered audio |
| Gene Expression | How latent DNA maps to generated audio |
| Regulatory Gene | Controls influence of other genes |
| Mutation | Random perturbation to a gene |
| Crossover | Mixing genes from two or more parents |
| Natural Selection | User preference / fitness function |
| Speciation | Latent clustering of similar sounds |
| Evolution | Population-level change over generations |
| Lineage | Parent-child ancestry tree |

---

## 2. Encoding Pipeline

```
Input Audio
    ↓
Encoder Model (from Prompt 13)
    ↓
Latent Trajectory + Global Code  (128-D + 128-D)
    ↓
DNA Decomposition Module
    ├── Timbre Decoder     →  timbre_gene (256-D)
    ├── Transient Analyzer →  transient_gene (128-D)
    ├── Harmonic Analyzer  →  harmonic_gene (64-D)
    ├── Noise Analyzer     →  noise_gene (64-D)
    ├── Mod Analyzer       →  modulation_gene (32-D)
    ├── Spectral Evolver   →  spectral_evolution (128-D)
    ├── Stereo Analyzer    →  stereo_gene (32-D)
    ├── Emotion Predictor  →  emotional_gene (64-D)
    ├── Genre Classifier   →  genre_gene (32-D)
    └── Production Analyzer→  production_gene (32-D)
    ↓
Normalize → Concatenate → Sound DNA (768-D)
```

### 2.1 Decomposition Detail

```python
class DNAEncoder(nn.Module):
    def __init__(self):
        self.timbre_encoder = nn.Sequential(
            SpectralEncoder(),     # mel + spectral features
            nn.Linear(512, 256)
        )
        self.transient_encoder = nn.Sequential(
            TransientFeatureExtractor(),  # envelope, onsets, etc.
            nn.Linear(256, 128)
        )
        self.harmonic_encoder = HarmonicProfileEncoder(64)
        self.noise_encoder = NoiseProfileEncoder(64)
        self.mod_encoder = ModulationEncoder(32)
        self.spectral_evolver = SpectralEvolutionEncoder(128)
        self.stereo_encoder = StereoFieldEncoder(32)
        self.emotion_encoder = EmotionEncoder(64)
        self.genre_encoder = GenreEncoder(32)
        self.production_encoder = ProductionEncoder(32)
        
    def forward(self, audio):
        return {
            'timbre': self.timbre_encoder(audio),
            'transient': self.transient_encoder(audio),
            'harmonic': self.harmonic_encoder(audio),
            'noise': self.noise_encoder(audio),
            'modulation': self.mod_encoder(audio),
            'spectral_evolution': self.spectral_evolver(audio),
            'stereo': self.stereo_encoder(audio),
            'emotional': self.emotion_encoder(audio),
            'genre': self.genre_encoder(audio),
            'production': self.production_encoder(audio),
        }
```

---

## 3. Decoding / Expression Pipeline

```
Sound DNA (768-D)
    ↓
Regulatory Gene Expression:
    - Apply gene_Gating:     suppress weak genes
    - Apply interaction:     cross-gene coupling
    ↓
Synthesis Parameter Prediction:
    ├── DSP Parameter Generator  →  synth params (osc, filter, envelope, FX)
    └── Neural Residual Generator →  residual audio (fine detail, texture)
    ↓
Hybrid Rendering:
    - DSP engine renders base sound from parameters
    - Neural generator adds residual detail
    ↓
Rendered One-Shot
```

---

## 4. Genetic Operations

### 4.1 Mutation

```python
def mutate_dna(dna, mutation_rate=0.1, mutation_scale=0.2):
    """Apply random mutations to sound DNA."""
    mutated = dna.copy()
    
    for gene_name in dna.genes:
        gene = dna[gene_name]
        # Per-gene mutation rate
        rate = mutation_rate * gene_mutation_sensitivity[gene_name]
        
        # Gaussian mutation
        mutation_mask = torch.rand_like(gene) < rate
        noise = torch.randn_like(gene) * mutation_scale
        mutated[gene_name] = gene + mutation_mask * noise
        
        # Optional: constrained mutation (keep in valid range)
        if has_bounds(gene_name):
            mutated[gene_name] = clamp(mutated[gene_name], gene_bounds[gene_name])
    
    return mutated

# Specialized mutation types:
def transient_mutate(dna, amount):
    """Deform transient gene (attack shape)."""
    pass

def harmonic_mutate(dna, amount):
    """Shift harmonic spectrum (formant shift)."""
    pass

def genre_mutate(dna, target_genre):
    """Shift genre gene toward target."""
    pass
```

### 4.2 Crossover (Recombination)

```python
def crossover(dna_a, dna_b, method='uniform'):
    """Create child DNA from two parents."""
    child = {}
    
    if method == 'uniform':
        # Each gene randomly from either parent
        for gene_name in dna_a.genes:
            child[gene_name] = random_choice([dna_a[gene_name], dna_b[gene_name]])
    
    elif method == 'single_point':
        # Split at random gene boundary
        split = random.randint(0, len(dna_a.genes))
        for i, gene_name in enumerate(dna_a.genes):
            child[gene_name] = dna_a[gene_name] if i < split else dna_b[gene_name]
    
    elif method == 'blend':
        # Interpolate all genes
        alpha = random.uniform(0.3, 0.7)
        for gene_name in dna_a.genes:
            child[gene_name] = lerp(dna_a[gene_name], dna_b[gene_name], alpha)
    
    elif method == 'semantic':
        # Crossover preserving semantic meaning
        # e.g., timbre from A, transient from B
        child['timbre'] = dna_a['timbre']   # keep A's timbre
        child['transient'] = dna_b['transient']  # get B's transient
        child['harmonic'] = crossover_blend(dna_a['harmonic'], dna_b['harmonic'])
        # ... controlled semantic mixing
    
    return child
```

### 4.3 Multi-Parent Crossover

```python
def multi_parent_crossover(parents, weights=None):
    """Combine genes from 3+ parents."""
    if weights is None:
        weights = [1/len(parents)] * len(parents)
    
    child = {}
    for gene_name in parents[0].genes:
        # Weighted average across all parents
        weighted_sum = sum(w * p[gene_name] for w, p in zip(weights, parents))
        child[gene_name] = weighted_sum
    
    return child
```

---

## 5. Similarity Search

### 5.1 DNA Similarity Metrics

```python
def dna_similarity(dna_a, dna_b):
    """Compare two sounds via their DNA."""
    scores = {}
    
    # Gene-level cosine similarities
    for gene_name in dna_a.genes:
        scores[gene_name] = cosine_similarity(
            dna_a[gene_name], dna_b[gene_name]
        )
    
    # Overall weighted similarity
    overall = sum(w * scores[g] for g, w in GENE_WEIGHTS.items())
    
    # Perceptual similarity (from Prompt 11)
    perceptual_sim = perceptual_similarity(dna_a, dna_b)
    
    return {
        'overall': overall,
        'perceptual': perceptual_sim,
        'per_gene': scores
    }
```

### 5.2 Search Modes

| Mode | Query | Returns |
|------|-------|---------|
| Global similarity | Sound DNA | Top-K nearest by overall cosine |
| Gene-specific | "Find similar timbre" | Top-K by timbre gene only |
| Compositional | "Timbre like A, transient like B" | Sounds near (A.timbre, B.transient) |
| Lineage search | "Siblings of this sound" | All sounds with same parent |
| Semantic search | "Punchy dark kick" | Text-encoded query → DNA space |
| Evolution search | "Most evolved" | Sounds with most mutations from ancestor |

---

## 6. Evolution Systems

### 6.1 Evolutionary Algorithm

```python
def evolve_sounds(
    initial_population, 
    generations=100,
    population_size=50,
    mutation_rate=0.2,
    crossover_rate=0.7,
    fitness_fn=None
):
    """Evolve a population of sounds toward higher fitness."""
    
    population = [encode(audio) for audio in initial_population]
    
    for gen in range(generations):
        # Evaluate fitness
        fitnesses = [fitness_fn(decode(dna)) for dna in population]
        
        # Selection (tournament)
        new_population = []
        while len(new_population) < population_size:
            if random() < crossover_rate:
                # Select two parents
                parent_a = tournament_select(population, fitnesses)
                parent_b = tournament_select(population, fitnesses)
                child = crossover(parent_a, parent_b)
            else:
                # Copy parent
                parent = tournament_select(population, fitnesses)
                child = copy(parent)
            
            # Mutation
            if random() < mutation_rate:
                child = mutate(child)
            
            new_population.append(child)
        
        population = new_population
    
    return [decode(dna) for dna in population]
```

### 6.2 Interactive Evolution (User-in-the-Loop)

```python
# User act as fitness function:
#   - Pick sounds they like → become parents
#   - Discard sounds they don't → removed from pool
#   - Rate sounds → fitness score

def interactive_session(initial_sounds, n_rounds=10):
    population = [encode(s) for s in initial_sounds]
    
    for round in range(n_rounds):
        # Render all sounds
        audio_samples = [decode(dna) for dna in population]
        
        # User selects favorites
        selected_indices = user_pick_favorites(audio_samples, n=5)
        selected = [population[i] for i in selected_indices]
        
        # Generate next generation from selection
        new_population = []
        for _ in range(len(population)):
            a, b = random_choice(selected, 2, replace=False)
            child = crossover(a, b)
            child = mutate(child)
            new_population.append(child)
        
        population = new_population
    
    return population
```

### 6.3 Speciation & Niche Preservation

```python
def evolve_with_speciation(population, n_species=5):
    """Maintain multiple species (sound types) during evolution."""
    while True:
        # Cluster population into species
        species = cluster_sounds(population, n_species)
        
        # Evolve within each species
        new_population = []
        for species_group in species:
            best = select_fittest(species_group, n=2)
            offspring = reproduce_within_species(best, n=len(species_group))
            new_population.extend(offspring)
        
        # Allow occasional cross-species breeding
        crossbreed = cross_species_breed(population, rate=0.1)
        new_population.extend(crossbreed)
        
        population = new_population
```

---

## 7. Style Inheritance

### 7.1 Lineage Tracking

```
Sound A (ancestor, 808 kick)
    ↓ mutation
Sound B (child, deeper 808)
    ↓ crossover with Sound C (acoustic kick)
Sound D (grandchild, hybrid 808/acoustic)
    ↓ mutation × 2
Sound E (great-grandchild, processed 808)
    ↓ ...
```

Each sound carries its full ancestry in `dna.lineage`.

### 7.2 Style Inheritance Strength

```python
def inheritance_analysis(child, parents):
    """Analyze which parent contributed which traits."""
    contribution = {}
    for gene_name in child.genes:
        for parent_idx, parent in enumerate(parents):
            sim = cosine_similarity(child[gene_name], parent[gene_name])
            contribution[f"{gene_name}_parent{parent_idx}"] = sim
    return contribution

# Example output:
# {
#   'timbre_parent0': 0.92,  # timbre inherited from parent A
#   'timbre_parent1': 0.45,  
#   'transient_parent0': 0.33,
#   'transient_parent1': 0.89,  # transient inherited from parent B
#   ...
# }
```

### 7.3 Lineage Visualization

```
        ┌── Sound B (timbre variant)
Sound A ─┤
         └── Sound C (noise variant)
                │
                ├── Sound D (crossover B × C)
                │
                └── Sound E (mutant)
```

---

## 8. Audio Evolution System Architecture

### 8.1 Population Management

```python
class SoundEvolutionSystem:
    def __init__(self):
        self.population = []        # List[SoundDNA]
        self.dna_library = {}       # id -> SoundDNA
        self.lineage_tree = {}      # id -> [parent_ids]
        self.generation = 0
        
    def add_sound(self, audio, metadata=None):
        dna = encode(audio)
        dna.id = generate_id()
        dna.metadata = metadata
        dna.generation = self.generation
        dna.parent_ids = []
        self.population.append(dna)
        self.dna_library[dna.id] = dna
        return dna.id
    
    def generate_offspring(self, parent_ids, n=5, mutation_rate=0.1):
        parents = [self.dna_library[id] for id in parent_ids]
        children = []
        for _ in range(n):
            if len(parents) >= 2:
                child_dna = crossover(random.choice(parents), random.choice(parents))
            else:
                child_dna = copy(parents[0])
            child_dna = mutate(child_dna, rate=mutation_rate)
            child_dna.id = generate_id()
            child_dna.parent_ids = parent_ids
            child_dna.generation = self.generation + 1
            children.append(child_dna)
        self.population.extend(children)
        for c in children:
            self.dna_library[c.id] = c
        return children
```

### 8.2 Fitness Functions

```python
# Acoustic fitness (objective):
fitness_balanced_spectrum = -variance(spectral_centroid_per_band)
fitness_punchy = transient_peak_ratio * attack_sharpness
fitness_clean = -noise_floor - spectral_roughness
fitness_wide = stereo_width
fitness_warm = warmth_index

# Perceptual fitness (from Prompt 11):
fitness_perceptual_quality = predict_human_preference(dna)

# User preference fitness (learned):
fitness_user_taste = user_taste_model.predict(dna)

# Task fitness (context-aware):
fitness_mix_fit = predict_mix_compatibility(dna, target_song_bpm, target_key)
```

### 8.3 Mutation Operators

| Operator | Effect | Gene Target |
|----------|--------|-------------|
| Gaussian | Add small random noise | Any |
| Swap | Swap two gene segments | Transient, spectral |
| Scaling | Multiply gene by scalar | Any |
| Interpolation | Move toward another sound | Any |
| Inversion | Reverse gene order | Spectral evolution |
| Permutation | Shuffle subcomponents | Noise, modulation |
| Expression | Toggle regulatory gene | Expression |
| Pruning | Zero out weak dimensions | Any |
| Mix | Blend with random sound | Any |
| Extreme | Push to boundary values | Any |

---

## 9. Applications

| Application | How It Works |
|-------------|--------------|
| **Sound breeding** | User selects favorites → system evolves next generation |
| **Library growth** | Existing sounds mutated for instant variation |
| **Genre hybridization** | Crossover between genre clusters |
| **Production chains** | Lineage tracking from raw to processed |
| **Procedural sound design** | Evolve toward target perceptual parameters |
| **Style preservation** | Gene-specific inheritance maintains identity |
| **Novelty search** | Fitness = distance from all existing sounds |
| **Sound fossils** | Archive evolutionary dead ends for inspiration |
