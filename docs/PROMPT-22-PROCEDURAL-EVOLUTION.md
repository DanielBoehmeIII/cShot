# Prompt 22 — Procedural Sound Evolution

cShot evolves entire sonic ecosystems.

---

## 1. Audio Genomes

### 1.1 Genome Structure

```python
class AudioGenome:
    """Complete genetic encoding of a one-shot for evolution."""
    
    def __init__(self):
        # === Structural Genes (physical sound production) ===
        self.physical = {
            'material_density':        Gene(range=(0.1, 20.0), mutation_rate=0.1),
            'material_stiffness':      Gene(range=(0.01, 1000.0), mutation_rate=0.1),
            'internal_damping':        Gene(range=(0.001, 0.5), mutation_rate=0.15),
            'shape_factor':            Gene(range=(0.0, 1.0), mutation_rate=0.2),
            'thickness':               Gene(range=(0.001, 0.1), mutation_rate=0.15),
            'size':                    Gene(range=(0.01, 10.0), mutation_rate=0.1),
            'surface_hardness':        Gene(range=(0.0, 1.0), mutation_rate=0.2),
            'roughness':               Gene(range=(0.0, 1.0), mutation_rate=0.2),
            'excitation_velocity':     Gene(range=(0.1, 50.0), mutation_rate=0.15),
            'excitation_mass':         Gene(range=(0.001, 10.0), mutation_rate=0.15),
            'contact_hardness':        Gene(range=(0.0, 1.0), mutation_rate=0.2),
        }
        
        # === Spectral Genes (frequency shaping) ===
        self.spectral = {
            'spectral_centroid':       Gene(range=(100, 8000), mutation_rate=0.15),
            'spectral_rolloff':        Gene(range=(500, 18000), mutation_rate=0.1),
            'harmonic_balance':        Gene(range=(-1.0, 1.0), mutation_rate=0.2),
            'formant_peaks':           Gene(dim=3, range=(100, 5000), mutation_rate=0.15),
            'noise_content':           Gene(range=(0.0, 1.0), mutation_rate=0.2),
            'inharmonicity':           Gene(range=(0.0, 0.5), mutation_rate=0.2),
        }
        
        # === Temporal Genes (envelope shaping) ===
        self.temporal = {
            'attack_time':             Gene(range=(0.0001, 2.0), mutation_rate=0.15),
            'attack_curve':            Gene(range=(0.1, 5.0), mutation_rate=0.2),
            'decay_time':              Gene(range=(0.01, 5.0), mutation_rate=0.15),
            'decay_curve':             Gene(range=(0.1, 5.0), mutation_rate=0.2),
            'sustain_level':           Gene(range=(0.0, 1.0), mutation_rate=0.1),
            'release_time':            Gene(range=(0.01, 10.0), mutation_rate=0.15),
            'transient_sharpness':     Gene(range=(0.0, 1.0), mutation_rate=0.2),
        }
        
        # === Modulatory Genes (time-varying behavior) ===
        self.modulatory = {
            'pitch_envelope_depth':    Gene(range=(0, 2000), mutation_rate=0.15),
            'pitch_envelope_rate':     Gene(range=(-5.0, 5.0), mutation_rate=0.2),
            'filter_sweep':            Gene(range=(0.0, 1.0), mutation_rate=0.15),
            'amplitude_modulation':    Gene(range=(0.0, 1.0), mutation_rate=0.2),
            'frequency_modulation':    Gene(range=(0.0, 1.0), mutation_rate=0.2),
        }
        
        # === Spatial Genes ===
        self.spatial = {
            'stereo_width':            Gene(range=(0.0, 1.0), mutation_rate=0.15),
            'pan_position':            Gene(range=(-1.0, 1.0), mutation_rate=0.1),
            'reverb_amount':           Gene(range=(0.0, 1.0), mutation_rate=0.2),
            'reverb_decay':            Gene(range=(0.1, 10.0), mutation_rate=0.15),
            'early_reflections':       Gene(range=(0.0, 1.0), mutation_rate=0.2),
        }
        
        # === Expressive Genes (high-level attributes) ===
        self.expressive = {
            'emotional_valence':       Gene(range=(-1.0, 1.0), mutation_rate=0.1),
            'emotional_arousal':       Gene(range=(-1.0, 1.0), mutation_rate=0.1),
            'genre_affinity':          Gene(dim=32, range=(0.0, 1.0), mutation_rate=0.05),
            'complexity':              Gene(range=(0.0, 1.0), mutation_rate=0.1),
            'production_era':          Gene(range=(0.0, 1.0), mutation_rate=0.05),
        }
        
        # === Regulatory Genes (control expression of other genes) ===
        self.regulatory = {
            'gene_expression_weights':  Gene(dim=50, range=(0.0, 2.0), mutation_rate=0.02),
            'gene_interaction_matrix':  Gene(dim=(50, 50), range=(-1.0, 1.0), mutation_rate=0.01),
        }
        
        # === Lineage ===
        self.lineage = {
            'parent_ids': [],
            'generation': 0,
            'mutation_history': [],
            'fitness_history': [],
        }
    
    def to_vector(self):
        """Flatten genome to a single vector for crossover."""
        vec = []
        for gene_group in [self.physical, self.spectral, self.temporal,
                           self.modulatory, self.spatial, self.expressive,
                           self.regulatory]:
            for gene_name, gene in gene_group.items():
                vec.extend(gene.flatten())
        return np.array(vec)
    
    def from_vector(self, vec):
        """Reconstruct genome from flat vector."""
        # (inverse of to_vector)
        pass
```

### 1.2 Gene Class

```python
class Gene:
    """A single genetic unit with mutation behavior."""
    
    def __init__(self, range=None, dim=1, mutation_rate=0.1, value=None):
        self.range = range  # (min, max) or None
        self.dim = dim
        self.mutation_rate = mutation_rate
        self.value = value if value is not None else self.random_value()
        
    def random_value(self):
        if self.dim == 1:
            return random.uniform(*self.range) if self.range else random.random()
        else:
            return np.random.uniform(*self.range, size=self.dim) if self.range else np.random.random(self.dim)
    
    def mutate(self, rate_mult=1.0):
        """Apply mutation to gene value."""
        rate = self.mutation_rate * rate_mult
        
        # Gaussian mutation (continuous genes)
        noise = np.random.randn(*np.atleast_1d(self.value).shape) * rate
        
        # Scale noise to range
        if self.range:
            span = self.range[1] - self.range[0]
            noise = noise * span * 0.1
        
        new_value = self.value + noise
        
        # Clip to range
        if self.range:
            new_value = np.clip(new_value, self.range[0], self.range[1])
        
        return new_value
    
    def crossover(self, other, method='uniform'):
        """Recombine with another gene."""
        if method == 'uniform':
            mask = np.random.random(self.dim) < 0.5
            child_value = np.where(mask, self.value, other.value)
        elif method == 'blend':
            alpha = random.uniform(0.3, 0.7)
            child_value = alpha * self.value + (1 - alpha) * other.value
        elif method == 'simulated_binary':
            # SBX crossover (common in real-coded GAs)
            beta = random.uniform(0, 1)
            eta = 20  # distribution index
            if beta <= 0.5:
                beta_q = (2 * beta) ** (1 / (eta + 1))
            else:
                beta_q = (1 / (2 * (1 - beta))) ** (1 / (eta + 1))
            child_value = 0.5 * ((1 + beta_q) * self.value + (1 - beta_q) * other.value)
        
        child = Gene(range=self.range, dim=self.dim, 
                     mutation_rate=self.mutation_rate, value=child_value)
        return child
```

---

## 2. Mutation Operators

| Operator | Description | Effect | Target |
|----------|-------------|--------|--------|
| Point mutation | Random value change in one gene | Small variation | Any single gene |
| Gaussian drift | Add Gaussian noise to all genes | Gradual evolution | All continuous genes |
| Boundary push | Push value to edge of range | Extreme variation | Any gene |
| Swap | Swap two gene values | Regulatory reshuffle | Any two genes |
| Inversion | Reverse a gene sequence | Temporal reversal | Temporal genes |
| Duplication | Duplicate a gene | Amplification | Any gene |
| Deletion | Remove a gene | Simplification | Any gene (set to 0) |
| Creep | Small bias in one direction | Directional evolution | Any continuous gene |
| Saltation | Large random jump | Experimental variation | Any gene |
| Gene expression toggle | Invert regulatory weight | Change gene influence | Regulatory genes |
| Interaction perturbation | Modify gene interaction matrix | System-level change | Interaction matrix |

```python
def mutate(genome, operator='gaussian', strength=1.0):
    """Apply mutation operator to genome."""
    if operator == 'gaussian':
        for group in [genome.physical, genome.spectral, genome.temporal,
                      genome.modulatory, genome.spatial, genome.expressive]:
            for gene_name, gene in group.items():
                if random.random() < gene.mutation_rate * strength:
                    gene.value = gene.mutate(strength)
    
    elif operator == 'boundary_push':
        group = random.choice(list(genome.__dict__.values()))
        gene_name = random.choice(list(group.keys()))
        gene = group[gene_name]
        if gene.range:
            gene.value = random.choice([gene.range[0] * 1.1, gene.range[1] * 1.1])
            gene.value = np.clip(gene.value, gene.range[0], gene.range[1])
    
    elif operator == 'saltation':
        group = random.choice(list(genome.__dict__.values()))
        gene_name = random.choice(list(group.keys()))
        gene = group[gene_name]
        gene.value = gene.random_value()
    
    elif operator == 'gene_expression_shuffle':
        # Randomly change regulatory weights
        for gene_name in genome.regulatory:
            if random.random() < 0.3:
                genome.regulatory[gene_name].value = np.random.uniform(0, 2)
    
    genome.lineage['mutation_history'].append({
        'operator': operator,
        'strength': strength,
        'generation': genome.lineage['generation']
    })
    
    return genome
```

---

## 3. Crossover (Recombination) Systems

| Method | Description | Best For |
|--------|-------------|----------|
| Single-point | Split at random gene boundary | Rough mixing |
| Two-point | Two split points, swap middle | Moderate mixing |
| Uniform | Per-gene random selection | Fine-grained mixing |
| Blend (BLX-α) | Interpolate between values | Continuous evolution |
| Simulated binary (SBX) | Simulate binary crossover | Continuous optimization |
| Gene-group | Swap entire functional group (e.g., all temporal genes) | Functional mixing |
| Semantic | Mix by perceptual role (timbre from A, transient from B) | Controlled breeding |
| Multi-parent | Weighted average of 3+ parents | Diverse offspring |
| Speciation-aware | Crossover within same cluster | Preserve identity |
| Hybridization | Crossover between different clusters | Novel hybrids |

```python
def crossover(genome_a, genome_b, method='uniform'):
    """Create child genome from two parents."""
    child = AudioGenome()
    
    if method == 'uniform':
        for group_name in ['physical', 'spectral', 'temporal', 'modulatory', 
                           'spatial', 'expressive', 'regulatory']:
            group_a = getattr(genome_a, group_name)
            group_b = getattr(genome_b, group_name)
            group_c = getattr(child, group_name)
            
            for gene_name in group_a:
                if random.random() < 0.5:
                    group_c[gene_name].value = group_a[gene_name].value.copy()
                else:
                    group_c[gene_name].value = group_b[gene_name].value.copy()
    
    elif method == 'blend':
        alpha = random.uniform(0.3, 0.7)
        for group_name in ['physical', 'spectral', 'temporal', 'modulatory',
                           'spatial', 'expressive', 'regulatory']:
            group_a = getattr(genome_a, group_name)
            group_b = getattr(genome_b, group_name)
            group_c = getattr(child, group_name)
            
            for gene_name in group_a:
                va = group_a[gene_name].value
                vb = group_b[gene_name].value
                group_c[gene_name].value = alpha * va + (1 - alpha) * vb
    
    elif method == 'semantic':
        # Keep timbre from A, transient from B
        child.physical = deepcopy(genome_a.physical)  # physical = timbre from A
        child.temporal = deepcopy(genome_b.temporal)  # temporal = transient from B
        # Blend everything else
        for group_name in ['spectral', 'modulatory', 'spatial', 'expressive']:
            group_a = getattr(genome_a, group_name)
            group_b = getattr(genome_b, group_name)
            group_c = getattr(child, group_name)
            for gene_name in group_a:
                group_c[gene_name].value = (group_a[gene_name].value + 
                                            group_b[gene_name].value) / 2
    
    # Inherit lineage
    child.lineage['parent_ids'] = [genome_a.lineage.get('id'), genome_b.lineage.get('id')]
    child.lineage['generation'] = max(genome_a.lineage['generation'], 
                                       genome_b.lineage['generation']) + 1
    
    return child
```

---

## 4. Trait Inheritance & Expression

### 4.1 Dominance and Recessiveness

```python
class TraitExpression:
    """Determine which parent's traits are expressed."""
    
    def __init__(self):
        # Dominance matrix: which gene group dominates when inherited
        self.dominance = {
            'attack_time': 'dominant_from_louder_parent',
            'spectral_centroid': 'co-dominant',
            'noise_content': 'recessive',
            'stereo_width': 'dominant_from_wider_parent',
            'emotional_valence': 'blended',
        }
    
    def express(self, child_genome, parent_a, parent_b):
        """Determine expressed phenotype from genotype."""
        expressed = AudioGenome()
        
        for gene_group in ['physical', 'spectral', 'temporal']:
            for gene_name, gene in getattr(parent_a, gene_group).items():
                dominance_rule = self.dominance.get(gene_name, 'blended')
                
                if dominance_rule == 'blended':
                    alpha = random.uniform(0.4, 0.6)
                    expressed[gene_group][gene_name].value = (
                        alpha * getattr(parent_a, gene_group)[gene_name].value +
                        (1 - alpha) * getattr(parent_b, gene_group)[gene_name].value
                    )
                elif dominance_rule == 'dominant_from_louder_parent':
                    louder_parent = parent_a if parent_a.loudness > parent_b.loudness else parent_b
                    expressed[gene_group][gene_name].value = getattr(louder_parent, gene_group)[gene_name].value
                elif dominance_rule == 'recessive':
                    # Only expressed if both parents have it
                    expressed[gene_group][gene_name].value = child_genome[gene_group][gene_name].value
                elif dominance_rule == 'co-dominant':
                    # Average of both
                    expressed[gene_group][gene_name].value = (
                        getattr(parent_a, gene_group)[gene_name].value +
                        getattr(parent_b, gene_group)[gene_name].value
                    ) / 2
        
        return expressed
```

### 4.2 Epistasis (Gene Interaction)

```python
class EpistasisModel:
    """Model how genes interact with each other."""
    
    def __init__(self):
        # Interaction network: which genes affect expression of others
        self.interactions = {
            'size': ['spectral_centroid', 'decay_time'],  # size affects these
            'density': ['pitch', 'inharmonicity'],
            'excitation_velocity': ['attack_time', 'transient_sharpness', 'loudness'],
            'attack_time': ['transient_sharpness', 'emotional_arousal'],
            'stereo_width': ['emotional_arousal', 'reverb_amount'],
        }
    
    def apply_epistasis(self, genome):
        """Apply gene interaction effects."""
        expressed = deepcopy(genome)
        
        for source_gene, target_genes in self.interactions.items():
            # Extract source gene value
            for group in ['physical', 'spectral', 'temporal', 'spatial', 'expressive']:
                if source_gene in getattr(genome, group):
                    source_value = getattr(genome, group)[source_gene].value
                    break
            
            # Apply to target genes
            for target in target_genes:
                for group in ['physical', 'spectral', 'temporal', 'spatial', 'expressive']:
                    if target in getattr(expressed, group):
                        current = getattr(expressed, group)[target].value
                        getattr(expressed, group)[target].value = current * (0.5 + source_value * 0.5)
                        break
        
        return expressed
```

---

## 5. Evolution Engine

### 5.1 Core Loop

```python
class SoundEvolutionEngine:
    """Core evolution loop for procedural sound design."""
    
    def __init__(self, population_size=100):
        self.population = [AudioGenome() for _ in range(population_size)]
        self.generation = 0
        self.fitness_history = []
        self.species_clusters = {}
        
    def evaluate_fitness(self, genomes):
        """Evaluate fitness of each genome."""
        fitnesses = []
        for genome in genomes:
            # Render to audio
            audio = self.genome_to_audio(genome)
            
            # Multiple fitness dimensions
            f_punch = compute_punch(audio)
            f_warmth = compute_warmth(audio)
            f_quality = compute_perceptual_quality(audio)
            f_novelty = compute_novelty(genome, self.population)
            
            # User preference (if available)
            f_user = self.user_taste_model.predict(genome) if self.user_taste_model else 0.5
            
            # Composite fitness
            fitness = (
                0.3 * f_punch +
                0.2 * f_warmth +
                0.2 * f_quality +
                0.1 * f_novelty +
                0.2 * f_user
            )
            fitnesses.append(fitness)
        return fitnesses
    
    def selection(self, fitnesses, n_parents=20):
        """Select parents for next generation."""
        # Tournament selection
        selected = []
        for _ in range(n_parents):
            tournament = random.sample(list(enumerate(fitnesses)), 3)
            winner = max(tournament, key=lambda x: x[1])
            selected.append(self.population[winner[0]])
        return selected
    
    def step(self):
        """One generation of evolution."""
        # Evaluate
        fitnesses = self.evaluate_fitness(self.population)
        self.fitness_history.append(np.mean(fitnesses))
        
        # Selection
        parents = self.selection(fitnesses)
        
        # Reproduction
        next_population = []
        while len(next_population) < len(self.population):
            p1, p2 = random.sample(parents, 2)
            
            if random.random() < 0.7:  # crossover rate
                child = crossover(p1, p2, method=random.choice(['uniform', 'blend', 'semantic']))
            else:
                child = deepcopy(p1 if random.random() < 0.5 else p2)
            
            if random.random() < 0.3:  # mutation rate
                child = mutate(child, operator=random.choice(
                    ['gaussian', 'boundary_push', 'saltation', 'gene_expression_shuffle']
                ))
            
            next_population.append(child)
        
        # Elitism: keep best 2
        best_indices = np.argsort(fitnesses)[-2:]
        next_population[:2] = [self.population[i] for i in best_indices]
        
        self.population = next_population
        self.generation += 1
        
        return self.population, fitnesses
```

### 5.2 Fitness Landscapes

```python
class FitnessLandscape:
    """Define fitness functions for different evolutionary goals."""
    
    @staticmethod
    def for_punch(genome):
        audio = genome_to_audio(genome)
        return compute_punch(audio)
    
    @staticmethod
    def for_emotional_target(genome, target_vap):
        audio = genome_to_audio(genome)
        vap = predict_emotion(audio)
        return 1.0 - euclidean_distance(vap, target_vap) / 2.0
    
    @staticmethod
    def for_genre_fidelity(genome, target_genre):
        audio = genome_to_audio(genome)
        genre_probs = classify_genre(audio)
        return genre_probs[target_genre]
    
    @staticmethod
    def for_diversity(genome, population):
        """Encourage novelty."""
        average_similarity = np.mean([
            genome_similarity(genome, other) 
            for other in population 
            if other is not genome
        ])
        return 1.0 - average_similarity
    
    @staticmethod
    def for_evolutionary_goal(genome, goal_embedding):
        audio = genome_to_audio(genome)
        emb = encode_dna(audio)
        return cosine_similarity(emb, goal_embedding)
```

---

## 6. User-Guided Evolution

### 6.1 Interactive Breeding

```python
class InteractiveBreeding:
    """User acts as fitness function by selecting preferred sounds."""
    
    def present_generation(self, n_candidates=12):
        """Present a generation to user for selection."""
        candidates = random.sample(self.engine.population, n_candidates)
        
        # Render all
        audio_samples = [genome_to_audio(g) for g in candidates]
        
        # User selects favorites (via UI)
        selected_indices = self.user.select_favorites(audio_samples, n=2)
        selected = [candidates[i] for i in selected_indices]
        
        # Generate next generation from selection
        next_gen = []
        for _ in range(len(self.engine.population)):
            p1, p2 = random.sample(selected, 2)
            child = crossover(p1, p2)
            if random.random() < 0.2:
                child = mutate(child)
            next_gen.append(child)
        
        self.engine.population = next_gen
        return audio_samples  # return for playback
```

### 6.2 Evolutionary Search

```python
def evolutionary_search(target_description, n_generations=50):
    """Evolve toward a target sound description."""
    engine = SoundEvolutionEngine(population_size=200)
    target_emb = embed_text(target_description)
    
    for gen in range(n_generations):
        # Custom fitness: similarity to target
        fitnesses = []
        for genome in engine.population:
            audio = genome_to_audio(genome)
            emb = encode_dna(audio)
            fitness = cosine_similarity(emb, target_emb)
            fitnesses.append(fitness)
        
        # Early stopping
        if max(fitnesses) > 0.95:
            break
        
        # Selection and reproduction
        parents = engine.selection(np.array(fitnesses), n_parents=30)
        engine.population = engine.reproduce(parents, engine.population_size)
    
    # Return best
    best_idx = np.argmax([engine.evaluate_fitness([g])[0] for g in engine.population])
    return engine.population[best_idx]
```

### 6.3 Autonomous Sound Breeding

```python
class AutonomousBreeder:
    """Breed sounds toward an optimal goal without user intervention."""
    
    def breed_for_punch(self, n_generations=100):
        engine = SoundEvolutionEngine(population_size=100)
        
        for gen in range(n_generations):
            fitnesses = [compute_punch(genome_to_audio(g)) for g in engine.population]
            parents = engine.selection(np.array(fitnesses))
            engine.population = engine.reproduce(parents)
            
            if gen % 10 == 0:
                print(f"Gen {gen}: max punch = {max(fitnesses):.3f}")
        
        best = max(engine.population, key=lambda g: compute_punch(genome_to_audio(g)))
        return best
    
    def diversify_species(self, n_species=5):
        """Evolve multiple distinct sonic species simultaneously."""
        species = [SoundEvolutionEngine(population_size=50) for _ in range(n_species)]
        
        for gen in range(100):
            for i, sp in enumerate(species):
                fitnesses = [sp.evaluate_fitness([g])[0] for g in sp.population]
                
                # Add niche pressure: penalize similarity to other species
                for j, other_sp in enumerate(species):
                    if i != j:
                        for k, genome in enumerate(sp.population):
                            sim_to_other = max(genome_similarity(genome, og) 
                                               for og in other_sp.population)
                            fitnesses[k] -= sim_to_other * 0.3  # niche penalty
                
                parents = sp.selection(np.array(fitnesses))
                sp.population = sp.reproduce(parents)
        
        return species
```

---

## 7. Ecosystem Evolution

### 7.1 Sonic Ecosystems

An ecosystem is a population of interrelated sounds that co-evolve:

```
Kick ↔ Snare ↔ Hi-hat ↔ Clap ↔ Tom
  ↕       ↕        ↕       ↕      ↕
(share evolutionary pressure: genre, mix balance, emotional tone)
```

```python
class SonicEcosystem:
    """Co-evolve multiple sound types together."""
    
    def __init__(self, target_genre='techno'):
        self.target_genre = target_genre
        self.species = {
            'kick': SoundEvolutionEngine(population_size=50),
            'snare': SoundEvolutionEngine(population_size=50),
            'hihat': SoundEvolutionEngine(population_size=50),
            'clap': SoundEvolutionEngine(population_size=50),
            'tom': SoundEvolutionEngine(population_size=30),
        }
        
    def step(self):
        """Co-evolve all species."""
        # Evaluate each species
        all_fitnesses = {}
        for name, sp in self.species.items():
            fitnesses = []
            for genome in sp.population:
                audio = genome_to_audio(genome)
                
                # Individual fitness
                f_quality = compute_perceptual_quality(audio)
                f_genre = classify_genre(audio)[self.target_genre]
                
                # Ecosystem fitness: how well does this sound work with others?
                f_cohesion = 0.0
                for other_name, other_sp in self.species.items():
                    if other_name != name:
                        other_audio = genome_to_audio(other_sp.population[0])
                        f_cohesion += compute_mix_compatibility(audio, other_audio)
                f_cohesion /= len(self.species) - 1
                
                fitness = 0.3 * f_quality + 0.4 * f_genre + 0.3 * f_cohesion
                fitnesses.append(fitness)
            
            # Evolution step per species
            parents = sp.selection(np.array(fitnesses))
            sp.population = sp.reproduce(parents)
    
    def export_pack(self, n_per_type=5):
        """Export a cohesive sample pack."""
        pack = {}
        for name, sp in self.species.items():
            sorted_pop = sorted(sp.population, 
                               key=lambda g: sp.evaluate_fitness([g])[0], 
                               reverse=True)
            pack[name] = [(genome_to_audio(sorted_pop[i]), sorted_pop[i]) 
                         for i in range(n_per_type)]
        return pack
```

---

## 8. Lineage & Phylogenetic Trees

```python
class LineageTracker:
    """Track sound evolution lineages."""
    
    def __init__(self):
        self.sounds = {}  # id -> genome
        self.tree = {}    # id -> [parent_ids]
    
    def register_sound(self, genome):
        sound_id = uuid4()
        self.sounds[sound_id] = genome
        self.tree[sound_id] = genome.lineage['parent_ids']
        return sound_id
    
    def get_ancestors(self, sound_id, depth=10):
        """Get lineage tree up to depth generations back."""
        ancestors = []
        current_ids = [sound_id]
        
        for _ in range(depth):
            next_ids = []
            for cid in current_ids:
                parents = self.tree.get(cid, [])
                for pid in parents:
                    if pid in self.sounds:
                        ancestors.append((pid, self.sounds[pid]))
                        next_ids.append(pid)
            if not next_ids:
                break
            current_ids = next_ids
        
        return ancestors
    
    def get_mutation_rate(self, sound_id_a, sound_id_b):
        """Genetic distance between two sounds."""
        a = self.sounds[sound_id_a].to_vector()
        b = self.sounds[sound_id_b].to_vector()
        return 1.0 - cosine_similarity(a, b)
    
    def render_phylogeny(self, sound_ids):
        """Generate phylogenetic tree visualization data."""
        # UMAP on genome vectors + hierarchical clustering
        vectors = [self.sounds[sid].to_vector() for sid in sound_ids]
        tree = hierarchical_clustering(vectors)
        return tree
```
