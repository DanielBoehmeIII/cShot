# Prompt 27 — Invent a Search Engine for Sound

cShot makes sound search meaning-based, not filename-based.

---

## 1. Search Modalities

| Modality | Query Type | Example |
|----------|-----------|---------|
| **Text** | Natural language | "dark punchy kick with warm body" |
| **Audio** | Example sound | [drag audio file] "find similar" |
| **Perceptual** | Sliders/axes | punch=0.8, warmth=0.3, brightness=0.6 |
| **Emotional** | Mood coordinates | V=[0.3], A=[0.7], P=[0.5] |
| **Genre** | Genre name | "techno kick, 2010s Berlin style" |
| **Production** | Processing chain | "compressed, saturated, wide reverb" |
| **Physical** | Material properties | "wooden resonance, metal attack" |
| **Reference** | Song/artist | "sounds like the kick in Daft Punk's One More Time" |
| **Rhythm** | Pattern | "four-on-the-floor, 128bpm" |
| **Compositional** | Combination | "timbre like sample A, transient like sample B" |
| **Descriptive** | Free text | "what you'd hear if a building fell into the ocean" |
| **Negative** | Exclusion | "NOT clicky, NOT too bright" |

---

## 2. Embedding Architecture

### 2.1 Multi-Modal Embedding Space

```python
class MultiModalAudioEncoder(nn.Module):
    """Encode audio, text, and perceptual data into shared space."""
    
    def __init__(self, dim=768):
        super().__init__()
        self.dim = dim
        
        # Audio encoder (from Prompt 13)
        self.audio_encoder = OneShotEncoder()  # 768-D output
        
        # Text encoder (CLAP-style)
        self.text_encoder = nn.TransformerEncoder(
            nn.TransformerEncoderLayer(d_model=512, nhead=8),
            num_layers=6
        )
        self.text_proj = nn.Linear(512, dim)
        
        # Perceptual encoder (from Prompt 11)
        self.perceptual_encoder = nn.Sequential(
            nn.Linear(12, 128),  # 12 perceptual axes
            nn.GELU(),
            nn.Linear(128, dim),
        )
        
        # Emotional encoder (from Prompt 12)
        self.emotion_encoder = nn.Sequential(
            nn.Linear(3, 64),  # VAP coordinates
            nn.GELU(),
            nn.Linear(64, dim),
        )
        
        # Production encoder
        self.production_encoder = nn.Sequential(
            nn.Linear(32, 128),  # production parameters
            nn.GELU(),
            nn.Linear(128, dim),
        )
        
        # Final projection to shared space
        self.shared_projection = nn.Linear(dim * 5, dim)
        
    def encode_audio(self, audio):
        return self.shared_projection(self.audio_encoder(audio))
    
    def encode_text(self, text):
        tokens = self.tokenize(text)
        features = self.text_encoder(tokens)
        return self.text_proj(features.mean(dim=1))
    
    def encode_perceptual(self, perceptual_vector):
        return self.perceptual_encoder(perceptual_vector)
    
    def encode_hybrid(self, audio=None, text=None, perceptual=None, 
                       emotion=None, production=None):
        """Encode any combination of modalities."""
        embeddings = []
        
        if audio is not None:
            embeddings.append(self.encode_audio(audio))
        if text is not None:
            embeddings.append(self.encode_text(text))
        if perceptual is not None:
            embeddings.append(self.encode_perceptual(perceptual))
        if emotion is not None:
            embeddings.append(self.emotion_encoder(emotion))
        if production is not None:
            embeddings.append(self.production_encoder(production))
        
        if not embeddings:
            return None
        
        # Average available embeddings
        combined = torch.stack(embeddings).mean(dim=0)
        return F.normalize(combined, dim=-1)
```

### 2.2 Training (Contrastive)

```python
def train_multimodal_encoder():
    model = MultiModalAudioEncoder()
    optimizer = AdamW(model.parameters(), lr=1e-4)
    
    # Data: (audio, text, perceptual, emotion) quadruples
    # Positive pairs: same sound's audio + text description
    # Negative pairs: different sounds
    
    for batch in dataloader:
        audio_emb = model.encode_audio(batch.audio)
        text_emb = model.encode_text(batch.text)
        
        # Contrastive loss (InfoNCE)
        # audio-text pairs should be close
        logits = audio_emb @ text_emb.T * temperature
        labels = torch.arange(len(batch))  # diagonal = positive pairs
        loss = F.cross_entropy(logits, labels)
        
        loss.backward()
        optimizer.step()
```

---

## 3. Indexing System

### 3.1 Multi-Index Architecture

```python
class SoundIndex:
    """Multi-layered index for semantic sound search."""
    
    def __init__(self):
        # Primary: HNSW for embedding similarity
        self.embedding_index = HNSWIndex(
            dim=768,
            metric='cosine',
            ef_construction=200,
            M=32
        )
        
        # Secondary: Inverted index for metadata filtering
        self.metadata_index = InvertedIndex()
        
        # Tertiary: Text index for full-text search
        self.text_index = TextIndex()
        
        # Store full metadata
        self.metadata_store = {}  # id -> metadata dict
    
    def add_sound(self, sound_id, audio, metadata):
        """Index a sound across all indices."""
        # Compute embedding
        emb = encoder.encode_audio(audio)
        
        # Add to embedding index
        self.embedding_index.add(sound_id, emb)
        
        # Add to metadata index
        for key, value in metadata.items():
            self.metadata_index.add(key, value, sound_id)
        
        # Add text fields to text index
        for text_field in ['description', 'tags', 'genre']:
            if text_field in metadata:
                self.text_index.add(sound_id, metadata[text_field])
        
        # Store full metadata
        self.metadata_store[sound_id] = metadata
    
    def search(self, query, k=10, filters=None):
        """Multi-stage search."""
        # Stage 1: Embedding search
        query_emb = query.get_embedding()
        candidates = self.embedding_index.search(query_emb, k=k*10)
        
        # Stage 2: Filter
        if filters:
            candidates = [c for c in candidates 
                          if self._matches_filters(c, filters)]
        
        # Stage 3: Re-rank
        if len(candidates) > k:
            candidates = self._rerank(candidates, query)
        
        return candidates[:k]
```

### 3.2 Query Builder

```python
class SoundQuery:
    """Fluent builder for sound search queries."""
    
    def __init__(self):
        self.text = None
        self.audio_example = None
        self.perceptual_targets = {}
        self.emotional_target = None
        self.genre = None
        self.filters = {}
        self.exclusions = []
        self.weights = {}
        
    def with_text(self, text):
        self.text = text
        return self
    
    def with_audio_example(self, audio):
        self.audio_example = audio
        return self
    
    def with_perceptual(self, **kwargs):
        self.perceptual_targets.update(kwargs)
        return self
    
    def with_emotion(self, v=0, a=0, p=0):
        self.emotional_target = np.array([v, a, p])
        return self
    
    def with_genre(self, genre):
        self.genre = genre
        return self
    
    def with_filter(self, key, value):
        self.filters[key] = value
        return self
    
    def without(self, feature):
        self.exclusions.append(feature)
        return self
    
    def get_embedding(self):
        """Compute unified query embedding."""
        return encoder.encode_hybrid(
            audio=self.audio_example,
            text=self.text,
            perceptual=np.array(list(self.perceptual_targets.values())) if self.perceptual_targets else None,
            emotion=self.emotional_target,
        )
    
    def __repr__(self):
        parts = []
        if self.text: parts.append(f'"{self.text}"')
        if self.genre: parts.append(f'genre={self.genre}')
        if self.perceptual_targets: parts.append(f'perceptual={self.perceptual_targets}')
        return f'SoundQuery({", ".join(parts)})'
```

---

## 4. Retrieval Pipelines

### 4.1 Cascaded Retrieval

```python
class RetrievalPipeline:
    """Efficient cascaded retrieval."""
    
    def retrieve(self, query, k=10):
        # Stage 1: Coarse (billions → thousands) — 5ms
        coarse_results = self.coarse_index.search(
            query.get_embedding(), k=1000
        )
        
        # Stage 2: Filter (thousands → hundreds) — 1ms
        filtered = self.metadata_filter(coarse_results, query.filters)
        
        # Stage 3: Re-rank (hundreds → tens) — 10ms
        reranked = self.rerank(filtered, query)
        
        # Stage 4: Diversify — 2ms
        diversified = self.diversify(reranked, k)
        
        return diversified
    
    def rerank(self, candidates, query):
        """Fine-grained re-ranking."""
        query_emb = query.get_embedding()
        scored = []
        
        for c in candidates:
            # Cosine similarity
            base_score = cosine_similarity(query_emb, c.embedding)
            
            # Metadata match bonus
            meta_score = self.compute_metadata_match(c, query.filters)
            
            # Perceptual match
            if query.perceptual_targets:
                perceptual_distance = self.compute_perceptual_distance(
                    c.perceptual, query.perceptual_targets
                )
            else:
                perceptual_distance = 0
            
            # Final score
            score = 0.6 * base_score + 0.2 * meta_score - 0.2 * perceptual_distance
            scored.append((c, score))
        
        scored.sort(key=lambda x: x[1], reverse=True)
        return [c for c, _ in scored]
```

### 4.2 Autocomplete & Suggestions

```python
class SearchSuggestions:
    """Smart search suggestions as user types."""
    
    def suggest(self, partial_query):
        suggestions = []
        
        # 1. Text completion
        text_suggestions = self.text_completer.suggest(partial_query)
        suggestions.extend([('text', s) for s in text_suggestions[:3]])
        
        # 2. Perceptual axes
        if any(axis.startswith(partial_query.lower()) for axis in PERCEPTUAL_AXES):
            matches = [a for a in PERCEPTUAL_AXES if a.startswith(partial_query.lower())]
            suggestions.extend([('perceptual', m) for m in matches])
        
        # 3. Genre completion
        genre_matches = fuzzy_match_genre(partial_query)
        suggestions.extend([('genre', g) for g in genre_matches[:3]])
        
        # 4. Emotional
        if any(emotion.startswith(partial_query.lower()) for emotion in EMOTIONS):
            matches = [e for e in EMOTIONS if e.startswith(partial_query.lower())]
            suggestions.extend([('emotion', m) for m in matches])
        
        return suggestions
```

---

## 5. Semantic Clustering

### 5.1 Sound Clusters

```python
class SoundClustering:
    """Organize sounds into semantic clusters."""
    
    def __init__(self):
        self.clusters = {}  # cluster_id -> [sound_ids]
        self.cluster_centroids = {}  # cluster_id -> embedding
        self.cluster_labels = {}  # cluster_id -> description
        
    def cluster(self, sound_ids, embeddings, n_clusters=50):
        """Cluster sounds in embedding space."""
        from sklearn.cluster import HDBSCAN
        
        # HDBSCAN: density-based, handles noise
        clusterer = HDBSCAN(min_cluster_size=10, metric='cosine')
        labels = clusterer.fit_predict(embeddings)
        
        # Organize
        for sound_id, label in zip(sound_ids, labels):
            if label == -1:
                continue  # noise
            if label not in self.clusters:
                self.clusters[label] = []
            self.clusters[label].append(sound_id)
        
        # Compute centroids and labels
        for label, members in self.clusters.items():
            member_embs = [embeddings[sound_ids.index(m)] for m in members]
            self.cluster_centroids[label] = np.mean(member_embs, axis=0)
            self.cluster_labels[label] = self._describe_cluster(members)
    
    def _describe_cluster(self, member_ids):
        """Generate human-readable description of a cluster."""
        # Aggregate metadata
        genres = Counter()
        types = Counter()
        perceptual_means = defaultdict(list)
        
        for sid in member_ids:
            meta = metadata_store[sid]
            genres.update(meta.get('genres', []))
            types.update([meta.get('sound_type', 'unknown')])
            for k, v in meta.get('perceptual', {}).items():
                perceptual_means[k].append(v)
        
        # Dominant characteristics
        top_genre = genres.most_common(1)[0][0] if genres else 'unknown'
        top_type = types.most_common(1)[0][0] if types else 'unknown'
        avg_perceptual = {k: np.mean(v) for k, v in perceptual_means.items()}
        
        return {
            'name': f'{top_genre} {top_type}s',
            'dominant_genre': top_genre,
            'dominant_type': top_type,
            'size': len(member_ids),
            'avg_perceptual': avg_perceptual,
        }
```

---

## 6. Recommendation Systems

### 6.1 Collaborative Sound Discovery

```python
class SoundRecommender:
    """Recommend sounds based on user behavior patterns."""
    
    def recommend_for_user(self, user_id, n=10):
        """Personalized sound recommendations."""
        taste = taste_model.get_taste(user_id)
        
        # 1. From taste profile
        taste_recs = self._from_taste(taste, n=n//2)
        
        # 2. From collaborative filtering
        collab_recs = self._from_collaborative(user_id, n=n//4)
        
        # 3. Novelty / serendipity
        new_recs = self._from_novelty(taste, n=n//4)
        
        # Merge and rank
        all_recs = taste_recs + collab_recs + new_recs
        all_recs.sort(key=lambda x: x.score, reverse=True)
        
        return [r.sound for r in all_recs[:n]]
    
    def _from_collaborative(self, user_id, n=5):
        """Users similar to you also liked..."""
        similar_users = collaborative_model.get_similar_users(user_id)
        liked_by_similar = Counter()
        
        for uid, similarity in similar_users:
            for sound_id, rating in user_ratings[uid].items():
                if rating > 0.7:  # liked
                    liked_by_similar[sound_id] += similarity
        
        # Exclude sounds user already has
        user_sounds = set(user_libraries[user_id])
        candidates = [s for s in liked_by_similar if s not in user_sounds]
        
        return candidates[:n]
```

### 6.2 Search Result Diversification

```python
def diversify(results, query_embedding, k=10, lambda_mmr=0.5):
    """Maximal Marginal Relevance: balance relevance and diversity."""
    selected = []
    candidates = list(results)
    
    for _ in range(k):
        best = None
        best_score = -float('inf')
        
        for c in candidates:
            # Relevance to query
            relevance = cosine_similarity(query_embedding, c.embedding)
            
            # Diversity (max dissimilarity to already selected)
            if selected:
                max_sim_to_selected = max(
                    cosine_similarity(c.embedding, s.embedding) for s in selected
                )
            else:
                max_sim_to_selected = 0
            
            # MMR score
            score = lambda_mmr * relevance - (1 - lambda_mmr) * max_sim_to_selected
            
            if score > best_score:
                best_score = score
                best = c
        
        if best:
            selected.append(best)
            candidates.remove(best)
    
    return selected
```
