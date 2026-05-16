# Prompt 24 — Creative Taste Modeling

cShot learns each user's unique sonic identity.

---

## 1. Taste Representation

### 1.1 User Taste Vector

Each user is represented by a **taste profile** — a structured embedding of their sonic preferences:

```python
class UserTasteProfile:
    """Complete model of a user's sonic taste."""
    
    def __init__(self, user_id):
        self.user_id = user_id
        
        # === Perceptual Preferences ===
        # Learned centroids: where in perceptual space does user prefer to be?
        self.perceptual_centroids = {
            'punch':     PreferenceDistribution(0.6, 0.15),  # mean, std
            'warmth':    PreferenceDistribution(0.5, 0.2),
            'brightness': PreferenceDistribution(0.4, 0.15),
            'depth':     PreferenceDistribution(0.3, 0.2),
            'width':     PreferenceDistribution(0.5, 0.15),
            'texture':   PreferenceDistribution(0.4, 0.2),
            'harshness': PreferenceDistribution(0.2, 0.1),
            'gloss':     PreferenceDistribution(0.5, 0.2),
        }
        
        # === Genre Affinities ===
        self.genre_affinities = {}  # genre -> weight (0-1)
        # Learned from: user's library, generated sounds, search patterns
        
        # === Sound Type Preferences ===
        self.type_preferences = {
            'kick': Preferences(),
            'snare': Preferences(),
            'hat': Preferences(),
            # ...
        }
        
        # === Emotional Preferences ===
        self.emotional_centroid = np.zeros(3)  # VAP space
        self.emotional_radius = 0.5  # how wide is taste in emotional space
        
        # === Production Preferences ===
        self.production_prefs = {
            'preferred_loudness': PreferenceDistribution(-14, 2),  # LUFS
            'preferred_compression': PreferenceDistribution(0.4, 0.2),
            'preferred_reverb': PreferenceDistribution(0.3, 0.2),
            'preferred_era': 2024,
            'era_tolerance': 10,  # years
        }
        
        # === Complexity Preferences ===
        self.complexity_pref = PreferenceDistribution(0.5, 0.2)
        
        # === Evolution History ===
        self.taste_evolution = []  # timestamp -> taste snapshot
        
    def to_embedding(self):
        """Flatten taste to a 128-D embedding for comparison."""
        vec = []
        for dist in self.perceptual_centroids.values():
            vec.extend([dist.mean, dist.std])
        vec.extend(list(self.genre_affinities.values()))
        vec.extend(self.emotional_centroid.tolist())
        vec.append(self.complexity_pref.mean)
        return np.array(vec)
```

### 1.2 Preference Distribution

```python
class PreferenceDistribution:
    """Gaussian model of a single preference dimension."""
    
    def __init__(self, mean=0.5, std=0.2, n_samples=0):
        self.mean = mean
        self.std = std
        self.n_samples = n_samples  # confidence
        
    def likelihood(self, value):
        """How likely is this user to like a sound with this value?"""
        return norm.pdf(value, loc=self.mean, scale=max(self.std, 0.05))
    
    def update(self, value, feedback_strength=0.1):
        """Bayesian update with new observation."""
        self.n_samples += 1
        learning_rate = min(0.1, 1.0 / self.n_samples ** 0.5)
        
        # Update mean
        delta = value - self.mean
        self.mean += learning_rate * delta * feedback_strength
        
        # Update std (shrink with confidence)
        self.std = max(0.05, self.std * (1 - learning_rate * 0.1) + 
                       learning_rate * abs(delta) * 0.1)
```

---

## 2. Learning from Behavior

### 2.1 Signal Types

```python
class TasteSignalExtractor:
    """Extract taste signals from user behavior."""
    
    @staticmethod
    def from_generation(user, sound, context):
        """User generated a sound — what does this say about their taste?"""
        signals = []
        
        # They chose to generate this type of sound
        signals.append({
            'type': 'sound_type',
            'value': classify_sound_type(sound),
            'weight': 0.3,
        })
        
        # The parameters they chose (or accepted)
        features = extract_perceptual_features(sound)
        for dim, value in features.items():
            signals.append({
                'type': f'perceptual_{dim}',
                'value': value,
                'weight': 0.1,
            })
        
        return signals
    
    @staticmethod
    def from_modification(user, original, modified):
        """User modified a sound — reveals direction of preference."""
        signals = []
        
        # What changed?
        orig_feats = extract_perceptual_features(original)
        mod_feats = extract_perceptual_features(modified)
        
        for dim in orig_feats:
            delta = mod_feats[dim] - orig_feats[dim]
            if abs(delta) > 0.05:
                signals.append({
                    'type': f'modify_{dim}',
                    'value': dim,
                    'direction': np.sign(delta),
                    'magnitude': abs(delta),
                    'weight': 0.2,
                })
        
        return signals
    
    @staticmethod
    def from_save(user, sound):
        """User saved a sound — strong positive signal."""
        features = extract_perceptual_features(sound)
        return [
            {'type': f'perceptual_{dim}', 'value': value, 'weight': 1.0}
            for dim, value in features.items()
        ]
    
    @staticmethod
    def from_search(user, query, selected_result):
        """User searched and selected — reveals semantic preferences."""
        # Text of query reveals semantic preference
        query_emb = embed_text(query)
        sound_emb = encode_dna(selected_result)
        
        return [{
            'type': 'semantic_direction',
            'embedding': query_emb,
            'weight': 0.5,
        }]
    
    @staticmethod
    def from_dwell_time(user, sound, duration_ms):
        """How long user listened before deciding."""
        if duration_ms > 5000:  # listened a lot
            weight = 0.5
        elif duration_ms < 1000:  # skipped quickly
            weight = -0.3
        else:
            weight = 0.1
        
        features = extract_perceptual_features(sound)
        return [
            {'type': f'perceptual_{dim}', 'value': value, 'weight': abs(weight),
             'sign': np.sign(weight)}
            for dim, value in features.items()
        ]
```

### 2.2 Taste Update Engine

```python
class TasteUpdater:
    """Update user taste profile from behavioral signals."""
    
    def update(self, profile, signals):
        """Process a batch of signals and update profile."""
        for signal in signals:
            signal_type = signal['type']
            weight = signal['weight']
            
            if signal_type.startswith('perceptual_'):
                dim = signal_type.replace('perceptual_', '')
                if dim in profile.perceptual_centroids:
                    dist = profile.perceptual_centroids[dim]
                    dist.update(signal['value'], feedback_strength=weight)
            
            elif signal_type == 'sound_type':
                st = signal['value']
                if st not in profile.type_preferences:
                    profile.type_preferences[st] = Preferences()
                profile.type_preferences[st].increment(weight)
            
            elif signal_type.startswith('modify_'):
                dim = signal_type.replace('modify_', '')
                if dim in profile.perceptual_centroids:
                    dist = profile.perceptual_centroids[dim]
                    # Move mean in the direction user modified toward
                    direction = signal.get('direction', 1)
                    dist.mean += direction * signal['magnitude'] * weight * 0.1
        
        # Record evolution timestamp
        profile.taste_evolution.append({
            'timestamp': time(),
            'n_updates': len(signals),
            'perceptual_snapshot': {k: (v.mean, v.std) 
                                     for k, v in profile.perceptual_centroids.items()},
        })
```

---

## 3. Adaptive Generation

### 3.1 Taste-Conditioned Generation

```python
def generate_with_taste(profile, prompt, n_candidates=5):
    """Generate sounds that match user taste."""
    
    # Build conditioning vector from taste profile
    taste_conditioning = {
        'perceptual_targets': {
            dim: dist.mean 
            for dim, dist in profile.perceptual_centroids.items()
        },
        'emotional_target': profile.emotional_centroid.tolist(),
        'genre_affinities': profile.genre_affinities,
        'complexity_target': profile.complexity_pref.mean,
    }
    
    # Generate candidates
    candidates = []
    for _ in range(n_candidates * 2):  # generate extra for selection
        candidate = generation_model.generate(
            prompt=prompt,
            conditioning=taste_conditioning,
            temperature=0.3 + random.random() * 0.4,  # some variety
        )
        
        # Score by taste fit
        taste_score = score_taste_fit(candidate, profile)
        candidates.append((candidate, taste_score))
    
    # Return top candidates
    candidates.sort(key=lambda x: x[1], reverse=True)
    return [c[0] for c in candidates[:n_candidates]]


def score_taste_fit(sound, profile):
    """How well does this sound match user taste (0-1)?"""
    features = extract_perceptual_features(sound)
    
    score = 1.0
    for dim, value in features.items():
        if dim in profile.perceptual_centroids:
            dist = profile.perceptual_centroids[dim]
            likelihood = dist.likelihood(value)
            score *= max(0.1, likelihood / norm.pdf(dist.mean, loc=dist.mean, scale=dist.std))
    
    # Penalize uncertainty
    for dim, dist in profile.perceptual_centroids.items():
        if dist.n_samples < 5:
            score *= 0.9  # reduce confidence for under-observed dimensions
    
    return np.clip(score, 0, 1)
```

### 3.2 Taste-Aware Exploration

```python
class TasteExplorer:
    """Balance exploitation (known taste) with exploration."""
    
    def __init__(self, profile):
        self.profile = profile
        self.exploration_rate = 0.2
        self.exploration_decay = 0.99
        
    def generate(self, prompt):
        if random.random() < self.exploration_rate:
            # Explore: generate away from taste centroid
            offset = self._generate_exploration_offset()
            return generate_with_offset(prompt, offset)
        else:
            # Exploit: match taste
            return generate_with_taste(self.profile, prompt)[0]
    
    def _generate_exploration_offset(self):
        """Generate an offset away from known preferences."""
        offset = {}
        for dim, dist in self.profile.perceptual_centroids.items():
            if dist.n_samples > 10:  # only explore well-known dimensions
                # Push toward the less-explored direction
                offset[dim] = random.uniform(-1, 1) * dist.std * 2
        return offset
    
    def on_exploration_feedback(self, liked):
        """Adjust exploration rate based on feedback."""
        if liked:
            self.exploration_rate *= self.exploration_decay
        else:
            self.exploration_rate = min(0.5, self.exploration_rate * 1.1)
```

---

## 4. Personalized Latent Spaces

### 4.1 User-Specific Embedding

```python
class PersonalizedEmbedding:
    """Taste-modified latent space for each user."""
    
    def __init__(self, base_encoder, profile):
        self.base_encoder = base_encoder
        self.profile = profile
        
        # Learnable user-specific transform
        self.user_transform = nn.Sequential(
            nn.Linear(128, 128),
            nn.Tanh(),
        )
        
    def encode(self, audio):
        """Encode audio into user-personalized latent space."""
        base_emb = self.base_encoder(audio)
        user_offset = self.user_transform(torch.tensor(self.profile.to_embedding()))
        return base_emb + user_offset * 0.1
    
    def distance(self, audio_a, audio_b):
        """User-weighted perceptual distance."""
        emb_a = self.encode(audio_a)
        emb_b = self.encode(audio_b)
        
        # Base cosine distance
        dist = 1 - cosine_similarity(emb_a, emb_b)
        
        # Re-weight by user preference salience
        # (dimensions user cares about matter more)
        salience = np.array([1 - d.std for d in self.profile.perceptual_centroids.values()])
        dist = dist * salience.mean()
        
        return dist
```

### 4.2 Collaborative Taste Filtering

```python
class CollaborativeTasteModel:
    """Learn from similar users to improve cold-start taste modeling."""
    
    def __init__(self):
        self.user_embeddings = {}  # user_id -> taste embedding (128-D)
        self.similarity_matrix = None
        
    def find_similar_users(self, user_id, n=10):
        """Find users with similar taste."""
        if user_id not in self.user_embeddings:
            return []
        
        emb = self.user_embeddings[user_id]
        similarities = []
        for uid, uemb in self.user_embeddings.items():
            if uid != user_id:
                sim = cosine_similarity(emb, uemb)
                similarities.append((uid, sim))
        
        similarities.sort(key=lambda x: x[1], reverse=True)
        return similarities[:n]
    
    def predict_preference(self, user_id, sound):
        """Predict preference based on similar users."""
        similar = self.find_similar_users(user_id)
        if not similar:
            return 0.5
        
        sound_emb = encode_dna(sound)
        prediction = 0.0
        total_weight = 0.0
        
        for similar_id, similarity in similar:
            similar_user = user_database[similar_id]
            pref = score_taste_fit(sound, similar_user.taste_profile)
            prediction += similarity * pref
            total_weight += similarity
        
        return prediction / total_weight if total_weight > 0 else 0.5
```

---

## 5. Predictive Sound Recommendation

### 5.1 Recommendation Engine

```python
class SoundRecommender:
    """Recommend sounds based on user taste profile."""
    
    def recommend(self, profile, context=None, n=10):
        candidates = []
        
        # 1. Cold pool: sounds user hasn't heard
        unexplored = self.get_unexplored_sounds(profile.user_id)
        
        for sound in unexplored:
            # Taste fit
            taste_score = score_taste_fit(sound, profile)
            
            # Novelty (how different from what they've saved)
            saved_embs = [encode_dna(s) for s in self.get_user_saved(profile.user_id)]
            if saved_embs:
                sound_emb = encode_dna(sound)
                novelty = 1 - max(cosine_similarity(sound_emb, se) for se in saved_embs)
            else:
                novelty = 0.5
            
            # Context fit (genre, BPM, key matching)
            if context:
                context_score = compute_context_fit(sound, context)
            else:
                context_score = 0.5
            
            # Combined score
            score = 0.5 * taste_score + 0.3 * novelty + 0.2 * context_score
            candidates.append((sound, score))
        
        candidates.sort(key=lambda x: x[1], reverse=True)
        return [c[0] for c in candidates[:n]]
```

### 5.2 Taste Drift Detection

```python
class TasteDriftDetector:
    """Detect when user taste is evolving."""
    
    def detect_drift(self, profile, window_size=50):
        """Check if taste has changed significantly."""
        if len(profile.taste_evolution) < window_size:
            return 0.0
        
        recent = profile.taste_evolution[-window_size:]
        older = profile.taste_evolution[-(window_size*2):-window_size]
        
        # Compare distributions
        drift = 0
        for dim in profile.perceptual_centroids:
            recent_mean = np.mean([r['perceptual_snapshot'][dim][0] for r in recent])
            older_mean = np.mean([r['perceptual_snapshot'][dim][0] for r in older])
            drift += abs(recent_mean - older_mean)
        
        return drift / len(profile.perceptual_centroids)
    
    def get_current_phase(self, profile):
        """Classify user's current taste phase."""
        drift = self.detect_drift(profile)
        
        if drift < 0.05:
            return 'stable'  # consistent taste
        elif drift < 0.15:
            return 'exploring'  # somewhat changing
        elif drift < 0.3:
            return 'transitioning'  # significant change
        else:
            return 'reinventing'  # major taste shift
    
    def suggest_next_genre(self, profile):
        """Suggest genre based on taste trajectory."""
        phase = self.get_current_phase(profile)
        
        if phase == 'stable':
            # Stay in comfort zone
            return max(profile.genre_affinities, key=profile.genre_affinities.get)
        elif phase == 'exploring':
            # Suggest adjacent genre
            current = max(profile.genre_affinities, key=profile.genre_affinities.get)
            return get_adjacent_genre(current)
        elif phase == 'reinventing':
            # Suggest something completely different
            return random.choice([g for g in all_genres 
                                  if g not in profile.genre_affinities])
```

---

## 6. Sonic Identity Formation

Over time, each cShot user develops a unique **sonic identity**:

```python
def analyze_sonic_identity(profile):
    """Generate a user's sonic identity profile."""
    
    # Find most distinctive preferences
    distinctiveness = {}
    global_preferences = get_global_average_preferences()
    
    for dim, dist in profile.perceptual_centroids.items():
        # How much does this user deviate from the average?
        deviation = abs(dist.mean - global_preferences[dim].mean) / global_preferences[dim].std
        distinctiveness[dim] = deviation
    
    return {
        'most_distinctive_trait': max(distinctiveness, key=distinctiveness.get),
        'signature_sweet_spot': {
            dim: dist.mean 
            for dim, dist in profile.perceptual_centroids.items()
            if dist.std < 0.15  # tight preference = signature
        },
        'exploration_tendency': 'explorer' if profile.complexity_pref.mean > 0.6 else 'perfectionist',
        'genre_versatility': len([g for g in profile.genre_affinities if g > 0.3]),
        'drift_velocity': TasteDriftDetector().detect_drift(profile),
        'taste_maturity': profile.taste_evolution[-1]['n_updates'] if profile.taste_evolution else 0,
    }
```
