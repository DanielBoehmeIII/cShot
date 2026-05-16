# Prompt 15 — AI Sound Designer Agent

cShot is not a tool — it is an agent that understands sound design.

---

## 1. Agent Architecture

### 1.1 System Overview

```
┌─────────────────────────────────────────────────────┐
│                   cShot Agent                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │  Memory   │  │  Taste   │  │  Sound Intuition  │  │
│  │  System   │  │  Model   │  │  (Generation Core) │  │
│  └──────────┘  └──────────┘  └──────────────────┘  │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Workflow  │  │ Retrieval│  │   Knowledge       │  │
│  │  Engine   │  │  System  │  │   Base            │  │
│  └──────────┘  └──────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 1.2 Core Agent Capabilities

| Capability | Input | Output |
|-----------|-------|--------|
| Generate | Prompt, params, reference | New one-shot audio |
| Modify | Audio + instruction | Modified audio |
| Recommend | Context, taste profile | List of suggested sounds |
| Organize | Library of sounds | Tagged/clustered library |
| Pack | Genre/mood/theme | Cohesive sample pack |
| Learn | User interactions | Updated taste model |
| Explain | Sound | Text description of design decisions |

---

## 2. Memory Systems

### 2.1 Episodic Memory (Session Context)

```python
class EpisodicMemory:
    """Remembers what happened in current and recent sessions."""
    
    def __init__(self):
        self.current_session = []
        self.recent_sessions = deque(maxlen=10)
        self.working_context = {}
        
    def remember_interaction(self, user_input, sound_output, feedback):
        episode = {
            'timestamp': now(),
            'user_input': user_input,
            'sound_id': sound_output.id,
            'sound_dna': sound_output.dna,
            'feedback': feedback,
            'context': self.working_context.copy()
        }
        self.current_session.append(episode)
        
    def get_session_context(self):
        """Summarize current session for continuity."""
        if not self.current_session:
            return None
        # Extract patterns: what sounds did they like? what direction?
        liked = [e for e in self.current_session if e.feedback > 0]
        return {
            'n_generated': len(self.current_session),
            'liked_sounds': [e.sound_dna for e in liked],
            'current_direction': self.infer_direction(liked),
            'recent_action': self.current_session[-1].user_input
        }
```

### 2.2 Semantic Memory (Knowledge of Sound Design)

```python
class SemanticMemory:
    """Encyclopedic knowledge about sound design."""
    
    def __init__(self):
        self.genre_knowledge = {
            'house': {
                'kick': 'punchy, 125-130BPM, 4-on-the-floor',
                'clap': 'layered, reverb tail, on 2 and 4',
                'hihat': 'closed on 8ths, open on offbeats',
                'typical_key': 'minor, often F or G',
                'energy_curve': 'build every 8-16 bars'
            },
            'trap': {
                'kick': 'heavy 808 sub, distorted, long decay',
                'snare': 'huge, layered, on 3',
                'hihat': 'rapid rolls, triplets',
                'typical_key': 'minor, dark',
                'energy_curve': 'drop-focused, minimal buildup'
            },
            # ... 50+ genres
        }
        
        self.production_knowledge = {
            'mixing': {
                'kick_punch': 'sidechain compression, transient shaping, 60Hz boost',
                'snare_crack': '2-4kHz presence boost, short reverb',
                'warmth': '200-500Hz saturation, tube emulation',
                'width': 'mid-side processing, stereo delay, haas effect'
            },
            'sound_design_techniques': {
                'layering': 'combine 2-3 sounds for complexity',
                'fm_synthesis': 'carrier/modulator ratios for metallic tones',
                'granular': 'grain size, density, scatter for texture',
                'reverb_design': 'convolution, algorithmic, spring, plate'
            },
            # ...
        }
        
        self.instrument_knowledge = {
            'kick_drum': {
                'types': ['808', 'acoustic', 'electronic', 'layered', 'processed'],
                'freq_range': '20-150Hz (fundamental), 2-5kHz (click)',
                'typical_adsr': 'fast attack (0-5ms), medium decay (100-500ms)',
                'genre_associations': {...}
            },
            # ...
        }
```

### 2.3 Procedural Memory (Workflows)

```python
class ProceduralMemory:
    """Remembered workflows for sound design tasks."""
    
    def __init__(self):
        self.workflows = {
            'make_kick_punchier': [
                {'step': 'analyze_transient', 'params': {'window': '5ms'}},
                {'step': 'apply_transient_shaper', 'params': {'attack_boost': 3, 'sustain_cut': -2}},
                {'step': 'eq_boost', 'params': {'freq': 60, 'q': 1.5, 'gain': 2}},
                {'step': 'saturate', 'params': {'amount': 0.3, 'type': 'tube'}},
                {'step': 'compress', 'params': {'ratio': 4, 'attack': 1, 'release': 50}},
            ],
            'add_warmth': [
                {'step': 'cut_below', 'params': {'freq': 30}},
                {'step': 'eq_boost', 'params': {'freq': 200, 'q': 0.7, 'gain': 2}},
                {'step': 'saturate', 'params': {'amount': 0.2, 'type': 'tape'}},
                {'step': 'subtle_compression', 'params': {'ratio': 2, 'threshold': -20}},
            ],
            'make_wider': [
                {'step': 'mid_side_encode'},
                {'step': 'stereo_delay', 'params': {'time': '15ms', 'feedback': 0.2}},
                {'step': 'haas_effect', 'params': {'delay_ms': 12}},
                {'step': 'reverb', 'params': {'size': 0.6, 'decay': 1.5, 'width': 1.0}},
            ],
            # ...
        }
```

### 2.4 Working Memory (Current Task)

```python
class WorkingMemory:
    """Current task state and attention focus."""
    
    def __init__(self):
        self.active_task = None
        self.attention_focus = []  # what agent is currently focused on
        self.subgoals = []
        self.progress = {}
        
    def set_task(self, task):
        self.active_task = task
        self.subgoals = decompose_task(task)
        self.progress = {sg: 0.0 for sg in self.subgoals}
        self.attention_focus = [self.subgoals[0]]
        
    def update_progress(self, subgoal, delta):
        self.progress[subgoal] = min(1.0, self.progress[subgoal] + delta)
```

---

## 3. Taste Modeling

### 3.1 User Taste Profile

```python
class TasteProfile:
    """Learned model of a user's preferences."""
    
    def __init__(self, user_id):
        self.user_id = user_id
        self.genre_affinities = {}         # genre -> score (0-1)
        self.sound_type_affinities = {}    # kick/snare/hat -> score
        self.perceptual_preferences = {}   # perceptual axis -> target value
        self.dna_archetypes = []           # liked sound DNA vectors
        self.disliked_patterns = []        # disliked DNA patterns
        
    def update(self, sound, feedback):
        """Update taste model from user feedback."""
        # Update genre affinity
        for genre, prob in sound.genre_gene.items():
            g = self.genre_affinities.get(genre, 0.5)
            self.genre_affinities[genre] = g + 0.1 * (feedback - g)
        
        # Update perceptual preferences
        for axis, value in sound.perceptual_features.items():
            p = self.perceptual_preferences.get(axis, {'mean': 0.5, 'std': 0.3})
            # Bayesian update
            n = p.get('n', 0) + 1
            new_mean = (p['mean'] * (n-1) + value * feedback) / n
            self.perceptual_preferences[axis] = {
                'mean': new_mean,
                'std': sqrt(p['std']**2 + (value - new_mean)**2 / n),
                'n': n
            }
        
        # Store liked DNA
        if feedback > 0.5:
            self.dna_archetypes.append(sound.dna)
            if len(self.dna_archetypes) > 100:
                self.dna_archetypes.pop(0)
```

### 3.2 Preference Prediction

```python
def predict_preference(self, sound):
    """Predict how much user will like a sound (0-1)."""
    score = 0.0
    weights = []
    
    # Genre match
    for genre, prob in sound.genre_gene.items():
        affinity = self.genre_affinities.get(genre, 0.5)
        score += affinity * prob
        weights.append(prob)
    
    # Perceptual distance to preferred zone
    for axis, pref in self.perceptual_preferences.items():
        value = sound.perceptual_features.get(axis, 0.5)
        distance = abs(value - pref['mean']) / (pref['std'] + 0.01)
        score *= exp(-distance)  # Gaussian preference
    
    # DNA similarity to liked archetypes
    if self.dna_archetypes:
        dna_sims = [cosine_sim(sound.dna, a) for a in self.dna_archetypes]
        score += 0.3 * max(dna_sims)
    
    return sigmoid(score)
```

### 3.3 Taste Evolution Tracking

The agent tracks how user taste evolves over time, and can detect:
- **Shifts**: Gradual change in preferred genre or sound type
- **Exploration**: Periods of high-variance feedback (user trying new things)
- **Plateaus**: Repeated generation of similar sounds
- **Fatigue**: Decreasing engagement → recommend variety

---

## 4. Workflow Systems

### 4.1 Sound Design Workflow Engine

```python
class WorkflowEngine:
    """Orchestrates multi-step sound design tasks."""
    
    def plan_workflow(self, user_request):
        """Decompose a user request into a workflow plan."""
        plan = []
        
        if 'punchier' in user_request.lower():
            plan.extend(['analyze', 'transient_shape', 'eq', 'saturate', 'evaluate'])
        elif 'warmer' in user_request.lower():
            plan.extend(['analyze', 'eq', 'saturate', 'compress', 'evaluate'])
        elif 'create pack' in user_request.lower():
            plan.extend(['analyze_genre', 'plan_pack', 'generate_core', 
                        'generate_variations', 'mix_balance', 'export_pack'])
        elif 'design' in user_request.lower() and 'kick' in user_request.lower():
            plan.extend(['select_style', 'design_body', 'design_attack', 
                        'design_tail', 'layer', 'process', 'evaluate'])
        
        return plan
    
    def execute_step(self, step, context):
        if step == 'analyze':
            return self.analyze_sound(context.sound)
        elif step == 'transient_shape':
            return self.apply_transient_shaper(context.sound, context.params)
        elif step == 'eq':
            return self.apply_eq(context.sound, context.params)
        ...
```

### 4.2 Common Workflows

| Workflow | Steps | Output |
|----------|-------|--------|
| Sound Design from Scratch | style_select → design → layer → process → export | New sound |
| Sound Modification | analyze → target_parameter → modify → evaluate | Modified sound |
| Sample Pack Creation | genre_analysis → kit_plan → generate_core → generate_variations → export_bank | Pack of sounds |
| Library Organization | scan → analyze → cluster → tag → catalog | Organized library |
| Sound Matching | analyze_reference → extract_target_params → synthesize → compare | Matched sound |
| Trend Analysis | scan_production_trends → identify_patterns → apply_to_library | Trend-informed suggestions |

### 4.3 Agentic Decision-Making

```python
def decide_action(self, state):
    """Choose next action based on current state and goals."""
    if self.working_memory.active_task:
        return self.execute_next_workflow_step()
    
    if state.user_tired or state.session_long:
        return {'action': 'recommend_break', 'message': 'Try something different?'}
    
    if state.user_repeated_action_3x:
        return {'action': 'suggest_variation', 'strategy': 'opposite_direction'}
    
    if state.new_sound_in_library and not state.user_has_seen:
        return {'action': 'suggest_exploration', 'sound': state.new_sound}
    
    return {'action': 'await_input'}
```

---

## 5. Retrieval Systems

### 5.1 Multi-Modal Retrieval

```python
class SoundRetrievalSystem:
    """Retrieve sounds by multiple modalities."""
    
    def __init__(self):
        self.vector_db = VectorDatabase(dimension=768, metric='cosine')  # DNA vectors
        self.text_index = TextIndex()  # text metadata
        self.perceptual_index = PerceptualIndex()  # perceptual features
        
    def search(self, query):
        # Multi-modal query
        results = []
        
        # Text search
        if isinstance(query, str):
            text_results = self.text_index.search(query)
            results.extend(text_results)
        
        # Acoustic similarity
        if hasattr(query, 'audio'):
            dna = encode(query.audio)
            similar = self.vector_db.search(dna, k=20)
            results.extend(similar)
        
        # Perceptual search
        if isinstance(query, dict) and 'perceptual' in query:
            perceptual_results = self.perceptual_index.search(query['perceptual'])
            results.extend(perceptual_results)
        
        # Fusion
        return self.fuse_results(results)
    
    def fuse_results(self, results):
        """Combine multi-modal results with learned weights."""
        # Weighted combination of scores from different modalities
        fused = {}
        for r in results:
            key = r.id
            if key not in fused:
                fused[key] = {'sound': r.sound, 'score': 0.0}
            fused[key]['score'] += r.modality_weight * r.score
        return sorted(fused.values(), key=lambda x: x['score'], reverse=True)
```

### 5.2 Search Modalities

| Query Type | Example | Retrieval Method |
|-----------|---------|-----------------|
| Text | "dark punchy kick" | Text embedding → similarity search |
| Acoustic | (audio file) | DNA encode → vector search |
| Perceptual | punch=0.8, warm=0.3 | Perceptual feature search |
| Genre | "techno kick" | Genre gene filter |
| Emotion | "euphoric" | Emotional embedding search |
| Reference | "like this but brighter" | DNA offset + perceptual delta |
| Production | "sidechained, compressed" | Production gene search |
| Hybrid | "warm kick like track X" | Multi-modal fusion |

---

## 6. Adaptive Generation

### 6.1 Generation with Taste Conditioning

```python
def generate_with_taste(self, prompt, user):
    """Generate sound adapted to user's taste."""
    taste = self.taste_model.get_profile(user.id)
    
    # Condition generation on taste
    generated = self.generation_model.generate(
        prompt=prompt,
        conditioning={
            'genre_affinities': taste.genre_affinities,
            'perceptual_target': taste.perceptual_preferences,
            'style_archetypes': taste.dna_archetypes[:3],
        }
    )
    
    # Predict user preference
    predicted_like = self.taste_model.predict_preference(generated)
    
    # If low predicted preference, try adjusting
    if predicted_like < 0.5:
        adjusted = self.refine_for_taste(generated, taste)
        return adjusted, predicted_like
    
    return generated, predicted_like
```

### 6.2 Exploration vs Exploitation

The agent balances between:
- **Exploitation**: Generate sounds similar to what user has liked before
- **Exploration**: Suggest sounds outside user's typical preference zone

```python
exploration_prob = 0.2  # base exploration
if session_length > 20:
    exploration_prob += 0.1  # more exploration in long sessions
if user_just_rejected_3:
    exploration_prob += 0.2  # try something different

if random() < exploration_prob:
    # Explore: generate away from taste centroid
    taste_offset = random_direction() * exploration_radius
    generation_params = taste_conditioning + taste_offset
else:
    # Exploit: generate toward taste
    generation_params = taste_conditioning
```

### 6.3 Learning from Implicit Feedback

```python
def learn_from_behavior(self, user, interaction):
    """Learn from implicit signals (regeneration, modification, dwell time)."""
    feedback = 0.0
    
    if interaction.type == 'regenerated':
        feedback = -0.2  # didn't like it
    elif interaction.type == 'modified':
        feedback = 0.3   # liked but wanted changes
    elif interaction.type == 'saved':
        feedback = 0.8   # definitely liked
    elif interaction.type == 'exported':
        feedback = 1.0   # strong approval
    elif interaction.type == 'dwell':
        feedback = sigmoid((interaction.dwell_time - 3.0) / 1.0)  # 3+ seconds = engaged
    
    self.taste_model.update(interaction.sound, feedback)
```

---

## 7. Knowledge Base Integration

cShot maintains a knowledge base of sound design that the agent references:

| Domain | Knowledge |
|--------|-----------|
| Genre conventions | BPM ranges, typical sounds, arrangement patterns |
| Production techniques | EQ, compression, reverb, saturation best practices |
| Synthesis | FM, subtractive, wavetable, granular, physical modeling |
| Acoustics | Room modes, harmonic series, psychoacoustics |
| Industry trends | Current popular sounds, genre evolution |
| User community | Shared taste profiles, collaborative filtering |
| Sound history | Classic sounds, iconic productions, sample culture |

---

## 8. Agent Persona

cShot speaks to the user as a **collaborative sound design partner**:

| Situation | Agent Response Style |
|-----------|---------------------|
| User asks for a kick | "I'll design a kick. What genre? What energy level?" |
| User rejects a sound | "Got it — less aggressive? Let me adjust the attack." |
| User says "surprise me" | "Let's try something in the direction of [current trend]. Here's a sound." |
| User is looping same genre | "You've been in techno for a while. Want to try something in DnB?" |
| User saves many sounds | "Noting your preference for warm, punchy kicks with short decay." |
