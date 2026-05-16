# Prompt 23 — Model the Workflow of Elite Sound Designers

cShot thinks like an experienced sound designer.

---

## 1. Cognitive Model of Sound Design

### 1.1 The Sound Designer's Mind

Elite sound designers operate with a layered cognitive model:

```
Layer 1: INTENTION (What do I want? What emotion? What role?)
    ↓
Layer 2: MATERIAL THINKING (What physical object makes this sound?)
    ↓
Layer 3: SIGNAL IMAGINATION (What processing chain creates this?)
    ↓
Layer 4: TEXTURAL LAYERING (What combinations of sounds work?)
    ↓
Layer 5: MIX AWARENESS (How does this fit with other elements?)
    ↓
Layer 6: ITERATION (Does this work? What should change?)
    ↓
Back to Layer 1 (refined intention)
```

### 1.2 Designer Personas

| Persona | Primary Thinking Mode | Tool Preference | Iteration Style |
|---------|----------------------|-----------------|-----------------|
| **Tactile designer** | Physical/material | Hardware synths, mics, objects | Hands-on, tweak until it feels right |
| **Modular thinker** | Signal flow | Modular synths, complex chains | Patch → listen → repatch |
| **Sample curator** | Library search | Sample libraries, recorders | Collect → categorize → audition |
| **Synthesis expert** | Math/parameters | Soft synths, FM, wavetable | Define → compute → refine |
| **Foley artist** | Real-world objects | Microphones, props, recording | Capture → process → enhance |
| **Hybrid producer** | All of the above | DAW + everything | Layers all approaches |

### 1.3 Workflow Patterns

```
Pattern 1: "Start from Reference"
  1. Find reference sound in library / real world
  2. Analyze: what makes this sound work?
  3. Deconstruct: layer separation, processing chain
  4. Rebuild: recreate with own twist
  5. Evaluate: compare to reference, refine

Pattern 2: "Start from Emotion"
  1. Define emotional target (dark, euphoric, tense)
  2. Map emotion to acoustic parameters
  3. Generate raw material
  4. Process toward emotional target
  5. Validate: does it feel right?

Pattern 3: "Start from Experiment"
  1. Random patch / unknown processing chain
  2. Discover interesting sound
  3. Analyze what makes it interesting
  4. Refine toward usable sound
  5. Catalog for future use

Pattern 4: "Start from Constraint"
  1. Define constraints (genre, BPM, key, track already mixed)
  2. Design within constraints
  3. Push boundaries where possible
  4. Test in context
  5. Adjust for fit
```

---

## 2. Design Phase Analysis

### 2.1 Phase Decomposition

Research shows elite designers go through these phases per sound:

| Phase | Time % | Activities |
|-------|--------|------------|
| 1. Exploration | 30% | Search, audition, random patches, serendipity |
| 2. Selection | 10% | Choose direction, commit to approach |
| 3. Development | 40% | Main design work, processing, layering |
| 4. Refinement | 15% | EQ, compression, fit to mix, polish |
| 5. Validation | 5% | A/B test, context check, get feedback |

### 2.2 Pattern Libraries (Expert Knowledge)

```python
class SoundDesignKnowledge:
    """Expert-level sound design patterns."""
    
    kick_design_patterns = {
        'layered_kick': [
            {'layer': 'sub', 'source': 'sine_80hz', 'process': 'compress, saturate'},
            {'layer': 'body', 'source': '808_sample', 'process': 'eq_boost_100hz, short_reverb'},
            {'layer': 'attack', 'source': 'noise_click', 'process': 'transient_shaper, eq_boost_4khz'},
            {'mix': 'sub -6dB, body 0dB, attack -3dB'},
        ],
        'fm_kick': [
            {'algorithm': 'fm_synth', 'carrier': '80hz, sine',
             'modulator': '160hz, sine, depth=50%', 'pitch_envelope': '-50% over 100ms'},
            {'process': 'compression_4:1, eq_boost_60hz, saturation'},
        ],
        'acoustic_kick_process': [
            {'record': 'kick_drum, mic_position=inside_off_center'},
            {'blend': 'sub_sine_50hz', 'process': 'eq_cut_boxy_400hz, boost_thump_80hz'},
            {'room': 'convolution_reverb, drum_room_ir'},
        ],
    }
    
    layering_strategies = {
        'complementary': ['Layer lows from sound A', 'Layer mids from sound B', 
                          'Layer highs from sound C', 'EQ each to occupy own band'],
        'parallel': ['Duplicate sound', 'Process each copy differently', 
                     'Blend to taste', 'Creates richness without muddiness'],
        'spectral_layering': ['Analyze spectrum of each layer', 
                              'Ensure each occupies unique frequency zone',
                              'Check phase coherence between layers'],
        'temporal_layering': ['Attack from sound A', 'Sustain from sound B',
                               'Tail from sound C', 'Crossfade between layers'],
    }
```

---

## 3. AI-Assisted Workflow Systems

### 3.1 Workflow Engine

```python
class WorkflowEngine:
    """Models and assists the sound design workflow."""
    
    def __init__(self):
        self.current_phase = 'exploration'
        self.design_state = {
            'intention': None,
            'reference': None,
            'generated_sounds': [],
            'accepted_sounds': [],
            'processing_chain': [],
            'layers': [],
            'mix_context': None,
        }
        self.history = []
        
    def detect_phase(self, user_action):
        """Detect which design phase user is in based on actions."""
        if isinstance(user_action, SearchQuery):
            return 'exploration'
        elif isinstance(user_action, SelectSound):
            return 'selection'
        elif isinstance(user_action, ModifySound):
            # How many modifications? Early = development, late = refinement
            n_mods = len([a for a in self.history if isinstance(a, ModifySound)])
            if n_mods < 5:
                return 'development'
            return 'refinement'
        elif isinstance(user_action, TestInMix):
            return 'validation'
        return self.current_phase
    
    def suggest_next_action(self):
        """Suggest what to do next based on current phase."""
        suggestions = {
            'exploration': [
                'Try generating in a different genre',
                'Search for reference sounds with similar emotional profile',
                'Try extreme parameter values',
                'Layer two contrasting sounds',
            ],
            'selection': [
                'A/B test your top 3 candidates',
                'Listen in context of your track',
                'Consider how this fits the arrangement',
            ],
            'development': [
                'Try adding a complementary layer',
                'Process the transient separately',
                'Experiment with parallel processing',
                'Add movement with modulation',
            ],
            'refinement': [
                'Check frequency masking with other elements',
                'Fine-tune the decay to fit the BPM',
                'Adjust the stereo placement',
                'A/B against your reference',
            ],
            'validation': [
                'Solo with the kick/bass — check for phase issues',
                'Listen at low volume — does it still cut through?',
                'Check on multiple playback systems',
                'Get a second opinion',
            ],
        }
        return suggestions.get(self.current_phase, [])
```

### 3.2 Intelligent Assistance Levels

| Level | What AI Does | User Role |
|-------|-------------|-----------|
| 0 — Tool | Executes explicit commands | Full control, all decisions |
| 1 — Assistant | Suggests options, user chooses | Decision maker |
| 2 — Collaborator | Proposes directions, user refines | Co-creator |
| 3 — Apprentice | Learns from user, proposes improvements | Mentor |
| 4 — Partner | Discusses goals, co-creates | Creative partnership |

---

## 4. Iteration Intelligence

### 4.1 Iteration Pattern Analysis

```python
class IterationAnalyzer:
    """Analyze user iteration patterns and optimize suggestions."""
    
    def __init__(self):
        self.history = []
        
    def record_action(self, action, sound_before, sound_after):
        self.history.append({
            'action': action,
            'sound_before': encode_dna(sound_before),
            'sound_after': encode_dna(sound_after),
            'timestamp': time(),
        })
    
    def get_effective_operations(self):
        """Discover which operations the user finds most effective."""
        embeddings = [encode_dna(e['sound_before']) for e in self.history]
        deltas = []
        
        for i in range(1, len(self.history)):
            # How much did the sound change?
            change = cosine_distance(self.history[i]['sound_after'], 
                                     self.history[i]['sound_before'])
            
            # How much did the user like the result?
            # (inferred from: did they keep it? Did they build on it?)
            kept = self.history[i]['sound_after'] in self.history[i+1:].get('sound_before', [])
            
            deltas.append({
                'action': self.history[i]['action'],
                'change_magnitude': change,
                'kept': kept,
            })
        
        # Find patterns: which actions lead to kept sounds?
        effective = [d for d in deltas if d['kept']]
        return Counter([d['action'] for d in effective])
    
    def suggest_iteration(self, current_sound):
        """Suggest next iteration based on history."""
        embedding = encode_dna(current_sound)
        
        # Find most similar past state that led to a breakthrough
        for i, entry in enumerate(self.history):
            if cosine_similarity(embedding, entry['sound_before']) > 0.9:
                # This user was at a similar state before — what did they do?
                if i + 1 < len(self.history):
                    return self.history[i + 1]['action']
        
        return None
```

### 4.2 Breakthrough Detection

```python
def detect_breakthrough(history):
    """Detect when a user made a significant creative leap."""
    embeddings = [h['sound_after'] for h in history]
    
    breakthroughs = []
    for i in range(1, len(history)):
        # Sudden large change in latent space
        change = cosine_distance(embeddings[i], embeddings[i-1])
        
        # Was it followed by sustained exploration?
        subsequent_stability = np.mean([
            cosine_distance(embeddings[j], embeddings[i])
            for j in range(i+1, min(i+5, len(embeddings)))
        ])
        
        if change > 0.5 and subsequent_stability < 0.2:
            breakthroughs.append({
                'index': i,
                'action': history[i]['action'],
                'magnitude': change,
                'timestamp': history[i]['timestamp'],
            })
    
    return breakthroughs
```

---

## 5. Suggestion Systems

### 5.1 Contextual Suggestions

```python
class SuggestionEngine:
    """Generate contextually relevant suggestions."""
    
    def suggest_for_current_state(self, state):
        suggestions = []
        
        # Based on current sound characteristics
        if state.current_sound:
            if state.current_sound.punch < 0.3:
                suggestions.append({
                    'type': 'improvement',
                    'target': 'punch',
                    'text': 'This sounds weak — try a faster attack and 4kHz boost',
                    'actions': [
                        {'transient_shaper': {'attack': '+3dB', 'sustain': '-2dB'}},
                        {'eq': {'freq': 4000, 'gain': 3, 'q': 2}},
                    ]
                })
            
            if state.current_sound.noise_floor < -60:
                suggestions.append({
                    'type': 'texture',
                    'target': 'character',
                    'text': 'This is very clean — some analog saturation could add character',
                    'actions': [
                        {'saturation': {'type': 'tape', 'amount': 0.3}},
                    ]
                })
        
        # Based on missing elements
        if state.n_layers == 1:
            suggestions.append({
                'type': 'structure',
                'target': 'layering',
                'text': 'Try layering — a sub layer would add weight',
                'actions': ['add_sub_layer'],
            })
        
        # Based on genre conventions
        if state.genre == 'techno' and state.bpm > 130:
            suggestions.append({
                'type': 'genre',
                'target': 'authenticity',
                'text': 'For techno, try a shorter decay and more mid-range punch',
                'actions': [
                    {'decay': {'target': 0.3}},
                    {'eq': {'freq': 2000, 'gain': 2, 'q': 1.5}},
                ]
            })
        
        return suggestions
```

### 5.2 Counter-Suggestions

```python
def suggest_counterpoint(sound):
    """Suggest the opposite of what the user has (for variety)."""
    profile = analyze_perceptual(sound)
    
    opposites = {
        'dark': 'bright',
        'warm': 'cold',
        'soft': 'hard',
        'narrow': 'wide',
        'dry': 'wet',
        'clean': 'dirty',
        'simple': 'complex',
        'stable': 'evolving',
        'thin': 'fat',
        'tight': 'loose',
        'acoustic': 'synthetic',
        'natural': 'processed',
    }
    
    dominant = max(profile, key=profile.get)
    opposite = opposites.get(dominant, 'different')
    
    return {
        'text': f"You've been making {dominant} sounds. Try something {opposite}.",
        'suggested_parameters': invert_perceptual_params(profile),
    }
```

---

## 6. Creative Co-Pilot Architecture

```python
class CreativeCoPilot:
    """Full creative partnership system."""
    
    def __init__(self):
        self.workflow = WorkflowEngine()
        self.iteration = IterationAnalyzer()
        self.suggestion = SuggestionEngine()
        self.knowledge = SoundDesignKnowledge()
        self.taste = TasteProfile()
        
    def listen(self, user_action, context):
        """Observe user action and update internal state."""
        self.workflow.history.append(user_action)
        phase = self.workflow.detect_phase(user_action)
        self.iteration.record_action(user_action, context.before, context.after)
        
    def speak(self, context):
        """Generate response to user."""
        output = []
        
        # 1. Phase awareness
        phase = self.workflow.current_phase
        output.append(f"Phase: {phase}")
        
        # 2. Suggestions
        suggestions = self.suggestion.suggest_for_current_state(context)
        if suggestions:
            output.append("Suggestions:")
            for s in suggestions[:3]:
                output.append(f"  - {s['text']}")
        
        # 3. Counterpoint (every 3rd interaction)
        if len(self.workflow.history) % 3 == 0:
            counter = suggest_counterpoint(context.current_sound)
            output.append(f"Counterpoint: {counter['text']}")
        
        # 4. Iteration insight
        iter_suggestion = self.iteration.suggest_iteration(context.current_sound)
        if iter_suggestion:
            output.append(f"History suggests: {iter_suggestion}")
        
        # 5. Knowledge
        if 'kick' in str(user_action).lower():
            patterns = self.knowledge.kick_design_patterns
            key = random.choice(list(patterns.keys()))
            output.append(f"Design pattern: '{key}' — {patterns[key][0]}")
        
        return '\n'.join(output)
    
    def collaborate(self, user_intent, context):
        """Full co-creation session."""
        # 1. Understand intent
        intent_embedding = self.understand_intent(user_intent)
        
        # 2. Generate candidates with taste adaptation
        candidates = self.generate_candidates(intent_embedding, self.taste)
        
        # 3. Present to user
        selected = self.user.select(candidates)
        
        # 4. Iterate with user feedback
        for round in range(5):
            refinements = self.suggest_refinements(selected)
            selected = self.user.apply_refinements(selected, refinements)
            
            # Adapt taste model
            self.taste.update(selected, feedback=self.user.get_feedback(selected))
        
        return selected
```
