# Prompt 66 — Community Sound Graph

Design a community sound graph for cShot that connects users, sounds, prompts, genres, packs, references, remixes, branches, tags, and exports into a navigable knowledge graph.

---

## 1. What Is the Community Sound Graph?

### Definition

The community sound graph is a directed, weighted, multi-relational graph connecting every entity in the cShot ecosystem. Every generation, export, remix, comment, and pack addition adds edges to the graph.

```
Nodes (entities):
  ┌─────────────────────────────────────────────────────────────┐
  │                                                              │
  │  Users         ● ● ● ● ● ● ● ●                             │
  │  Sounds        ● ● ● ● ● ● ● ● ● ● ● ● ● ● ● ● ● ● ●    │
  │  Prompts       ● ● ● ● ● ● ● ●                             │
  │  Genres        ● ● ● ● ● ● ●                               │
  │  Packs         ● ● ● ● ●                                   │
  │  References    ● ● ● ●                                       │
  │  Tags          ● ● ● ● ● ● ● ● ● ● ● ● ●                   │
  │  Remixes       ● ● ●                                         │
  │  Exports       ● ● ● ● ● ● ● ● ● ●                           │
  │                                                              │
  └─────────────────────────────────────────────────────────────┘

Edges (relationships):
  User ──generates──► Sound
  User ──exports────► Sound
  User ──favorites──► Sound
  Sound ──uses──────► Prompt
  Sound ──has_type──► Genre
  Sound ──tagged_as─► Tag
  Sound ──remix_of──► Sound (parent)
  Pack ──contains──► Sound
  User ──created────► Pack
  Sound ──inspired──► Reference
  User ──comments──► Sound
```

---

## 2. Graph Schema

### Node Types

```rust
pub enum GraphNode {
    User {
        id: String,
        display_name: String,
        sonic_identity_hash: String,  // Hash of their identity profile
        generation_count: u64,
        join_date: DateTime<Utc>,
    },
    Sound {
        id: String,
        audio_hash: String,
        prompt_id: String,
        sound_type: SoundType,
        sound_score: f64,
        duration_ms: f64,
        spectral_centroid: f64,
        crest_factor: f64,
        embedding: Vec<f32>,          // 256-d audio embedding
        created_at: DateTime<Utc>,
    },
    Prompt {
        id: String,
        text: String,
        embedding: Vec<f32>,          // 768-d text embedding (CLAP)
        usage_count: u64,
    },
    Genre {
        id: String,
        name: String,
        parent_genre: Option<String>,
        sound_count: u64,
    },
    Pack {
        id: String,
        name: String,
        creator_id: String,
        sound_count: u64,
        description: String,
        tags: Vec<String>,
        created_at: DateTime<Utc>,
    },
    Tag {
        id: String,
        name: String,
        usage_count: u64,
    },
    Reference {
        id: String,
        original_filename: String,
        audio_hash: String,
        embedding: Vec<f32>,
        bpm: Option<u32>,
        key: Option<String>,
    },
    Export {
        id: String,
        user_id: String,
        sound_id: String,
        destination: String,          // "ableton", "desktop", "fl_studio", etc.
        exported_at: DateTime<Utc>,
    },
}
```

### Edge Types

```rust
pub enum GraphEdge {
    /// User → Sound
    Generated { user_id: String, sound_id: String, timestamp: DateTime<Utc> },
    Exported { user_id: String, sound_id: String, count: u32 },
    Favorited { user_id: String, sound_id: String },
    
    /// Sound → Sound
    RemixOf { parent_id: String, child_id: String, similarity: f64 },
    MorphedFrom { sound_a_id: String, sound_b_id: String, amount: f64 },
    
    /// Sound → Attribute
    HasPrompt { sound_id: String, prompt_id: String, alignment_score: f64 },
    HasType { sound_id: String, genre_id: String, confidence: f64 },
    TaggedAs { sound_id: String, tag_id: String, source: String },  // 'auto', 'user'
    UsedReference { sound_id: String, reference_id: String, influence: f64 },
    
    /// Sound → Pack
    PartOfPack { sound_id: String, pack_id: String, added_by: String },
    
    /// User → User
    Remixed { user_a_id: String, user_b_id: String, count: u32 },
    Collaborated { user_a_id: String, user_b_id: String, pack_id: String },
    
    /// Semantic similarity (computed, not explicit)
    SemanticallySimilar { 
        entity_a_id: String, 
        entity_b_id: String, 
        similarity: f64,
        dimension: String,  // 'audio', 'prompt', 'user_taste'
    },
}
```

---

## 3. Graph-Backed Features

### 3.1 Discovery

The graph enables discovery that flat databases cannot.

```
Without Graph:
  User searches "punchy kick" → gets all sounds tagged "punchy" and "kick"
  → Flat, keyword-based, misses relationships

With Graph:
  User searches "punchy kick" → traverses:
    1. Find prompt nodes matching "punchy kick" semantically
    2. Follow HasPrompt edges to Sound nodes
    3. For each sound, follow RemixOf edges to variations
    4. Follow User edges to see who made them
    5. Follow PartOfPack edges to find packs containing similar sounds
    6. Follow TaggedAs edges to discover related tags
    7. Follow SemanticallySimilar edges to find sonically similar sounds
       from different prompts
    
  Result: Not just matching sounds, but a neighborhood of related content
```

### Discovery Query Examples

```cypher
// Cypher-style queries for the sound graph

// 1. "Find sounds similar to one I liked"
MATCH (s:Sound {id: $sound_id})
MATCH (s)-[:SEMANTICALLY_SIMILAR]->(similar:Sound)
WHERE similar.sound_score > 60
RETURN similar
ORDER BY similar.sound_score DESC
LIMIT 20

// 2. "Who are the best kick makers?"
MATCH (u:User)-[:GENERATED]->(s:Sound {sound_type: 'kick'})
WHERE s.sound_score > 70
RETURN u.display_name, COUNT(s) as kick_count, AVG(s.sound_score) as avg_score
ORDER BY avg_score DESC
LIMIT 10

// 3. "What prompts produce the highest-scoring sounds?"
MATCH (p:Prompt)<-[:HAS_PROMPT]-(s:Sound)
WHERE s.sound_score > 80
RETURN p.text, AVG(s.sound_score) as avg_score, COUNT(s) as count
ORDER BY avg_score DESC
LIMIT 20

// 4. "Find packs in my genre wheelhouse"
MATCH (u:User {id: $user_id})
MATCH (u)-[:EXPORTED]->(exported:Sound)-[:HAS_TYPE]->(genre:Genre)
MATCH (pack:Pack)-[:CONTAINS]->(pack_sound:Sound)-[:HAS_TYPE]->(genre)
WHERE NOT (pack)-[:CONTAINS]->(:Sound {id: $excluded_sound_id})
RETURN pack, COUNT(pack_sound) as matching_sounds
ORDER BY matching_sounds DESC
LIMIT 10

// 5. "What's the remix path from this original sound?"
MATCH path = (original:Sound {id: $sound_id})-[:REMIX_OF*]->(remix:Sound)
RETURN path
ORDER BY length(path) DESC
LIMIT 50

// 6. "Which users have similar taste to me?"
MATCH (me:User {id: $user_id})
MATCH (me)-[:EXPORTED]->(my_sounds:Sound)
MATCH (other:User)-[:EXPORTED]->(their_sounds:Sound)
WHERE me <> other
WITH other, COLLECT(DISTINCT my_sounds.sound_type) as my_types,
            COLLECT(DISTINCT their_sounds.sound_type) as their_types
RETURN other.display_name, 
       size(apoc.coll.intersection(my_types, their_types)) as type_overlap
ORDER BY type_overlap DESC
LIMIT 10
```

### 3.2 Recommendations

The graph powers multi-hop recommendations that simple collaborative filtering cannot match.

```rust
pub struct GraphRecommender {
    graph: KnowledgeGraph,
}

impl GraphRecommender {
    /// "Users who exported this sound also exported..."
    pub fn sound_co_occurrence(&self, sound_id: &str, limit: u32) -> Vec<SoundRecommendation> {
        // Find users who exported this sound
        let users = self.graph.get_neighbors(sound_id, EdgeType::ExportedBy);
        
        // Find other sounds those users exported
        let mut scores: HashMap<String, f64> = HashMap::new();
        for user in users {
            let exports = self.graph.get_neighbors(user.id(), EdgeType::Exported);
            for export in exports {
                if export.id() != sound_id {
                    *scores.entry(export.id()).or_insert(0.0) += 1.0;
                }
            }
        }
        
        // Weight by sound score
        for (id, score) in scores.iter_mut() {
            if let Some(sound) = self.graph.get_node::<SoundNode>(id) {
                *score *= sound.sound_score / 100.0;
            }
        }
        
        // Sort, deduplicate, return top N
        let mut result: Vec<_> = scores.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        result.into_iter().take(limit as usize)
            .map(|(id, score)| SoundRecommendation { sound_id: id, score, reason: "Users like you also exported this sound".to_string() })
            .collect()
    }
    
    /// "Complete the pack" — given a few sounds, what's missing?
    pub fn complete_pack(&self, existing_sound_ids: &[String]) -> Vec<SoundRecommendation> {
        // Analyze the types/characters of existing sounds
        // Find gaps (e.g., has kicks but no snares)
        // Recommend sounds that fill those gaps with matching character
        
        let existing = existing_sound_ids.iter()
            .filter_map(|id| self.graph.get_node::<SoundNode>(id))
            .collect::<Vec<_>>();
        
        let types: HashSet<SoundType> = existing.iter().map(|s| s.sound_type).collect();
        let avg_centroid: f64 = existing.iter().map(|s| s.spectral_centroid).average();
        let avg_crest: f64 = existing.iter().map(|s| s.crest_factor).average();
        
        let mut recommendations = Vec::new();
        
        // Missing types
        let missing_types = vec![SoundType::Snare, SoundType::HiHat, SoundType::Clap];
        for missing in &missing_types {
            if !types.contains(missing) {
                // Find sounds of this type with similar spectral character
                let candidates = self.graph.find_sounds_by_type_and_character(
                    *missing, avg_centroid, avg_crest
                );
                if let Some(best) = candidates.first() {
                    recommendations.push(SoundRecommendation {
                        sound_id: best.id.clone(),
                        score: best.sound_score,
                        reason: format!("Completes your pack — adds a {} that matches your existing sounds", missing),
                    });
                }
            }
        }
        
        recommendations
    }
}
```

### 3.3 Attribution

The graph makes attribution automatic and verifiable.

```
Sound → RemixOf → Sound → RemixOf → Original Sound
  ↑            ↑                       ↑
User C       User B                  User A

Attribution chain:
  - Sound A: Generated by User A (original creator)
  - Sound B: Remixed from Sound A by User B
  - Sound C: Remixed from Sound B by User C
  
When Sound C is exported, the attribution metadata includes:
  - Creator: User C (final remixer)
  - Based on: Sound B by User B
  - Original: Sound A by User A
  - Contributors: [User A, User B, User C]
  - Split: 50% (A) / 30% (B) / 20% (C) — if monetized
```

### 3.4 Trend Detection

Time-series analysis on the graph reveals trends before they're obvious.

```rust
pub struct TrendDetector {
    graph: KnowledgeGraph,
}

impl TrendDetector {
    /// What genres are rising in the last 7 days?
    pub fn rising_genres(&self) -> Vec<TrendReport> {
        let now = Utc::now();
        let last_week = now - Duration::days(7);
        
        // Count new sounds per genre (traverse Generate edges)
        let mut genre_counts: HashMap<String, TrendingMetric> = HashMap::new();
        
        for edge in self.graph.edges_since(EdgeType::Generated, last_week) {
            if let Some(sound) = self.graph.get_node::<SoundNode>(&edge.target_id) {
                if let Some(genre) = self.graph.get_first_neighbor(&sound.id, EdgeType::HasType) {
                    let entry = genre_counts.entry(genre.label()).or_insert(TrendingMetric::default());
                    entry.recent_count += 1;
                    entry.avg_score += sound.sound_score;
                }
            }
        }
        
        // Compare with previous period for growth rate
        for (genre, metric) in genre_counts.iter_mut() {
            let previous_count = self.count_generations_for_genre(genre, last_week - Duration::days(7), last_week);
            metric.growth_rate = if previous_count > 0 {
                (metric.recent_count as f64 - previous_count as f64) / previous_count as f64
            } else {
                1.0 // New genre appearing
            };
            metric.avg_score /= metric.recent_count as f64;
        }
        
        let mut trends: Vec<_> = genre_counts.into_iter()
            .map(|(genre, metric)| TrendReport {
                genre,
                growth_rate: metric.growth_rate,
                recent_sounds: metric.recent_count,
                avg_sound_score: metric.avg_score,
            })
            .collect();
        
        trends.sort_by(|a, b| b.growth_rate.partial_cmp(&a.growth_rate).unwrap());
        trends.into_iter().take(10).collect()
    }
    
    /// What prompt patterns are getting the best results?
    pub fn best_prompt_patterns(&self) -> Vec<PromptPattern> {
        // Group prompts by structure: [genre] + [type] + [descriptor]
        // Score each pattern by average SoundScore of results
        
        let mut patterns: HashMap<String, Vec<f64>> = HashMap::new();
        
        for edge in self.graph.all_edges(EdgeType::HasPrompt) {
            if let Some(sound) = self.graph.get_node::<SoundNode>(&edge.source_id) {
                if let Some(prompt) = self.graph.get_node::<PromptNode>(&edge.target_id) {
                    let pattern = classify_prompt_pattern(&prompt.text);
                    patterns.entry(pattern).or_default().push(sound.sound_score);
                }
            }
        }
        
        patterns.into_iter()
            .map(|(pattern, scores)| PromptPattern {
                pattern,
                avg_score: scores.iter().sum::<f64>() / scores.len() as f64,
                count: scores.len(),
                best_prompt: self.find_best_prompt_for_pattern(&pattern),
            })
            .sorted_by(|a, b| b.avg_score.partial_cmp(&a.avg_score))
            .take(10)
            .collect()
    }
}
```

### 3.5 Collaborative Creation

The graph shows who should work together.

```rust
pub fn find_collaboration_matches(user_id: &str) -> Vec<CollaborationMatch> {
    // Analyze: user excels at kicks, needs snares
    // Find: users who excel at snares, need kicks
    // Match: "You make great kicks, they make great snares. Collaborate!"
    
    let user_strengths = analyze_user_strengths(user_id);
    let user_weaknesses = analyze_user_weaknesses(user_id);
    
    let mut matches = Vec::new();
    
    for weakness in &user_weaknesses {
        let candidates = find_users_strong_in(weakness.sound_type, 10);
        for candidate in candidates {
            let candidate_strengths = analyze_user_strengths(&candidate.user_id);
            let complementary = candidate_strengths.iter()
                .any(|s| s.sound_type == weakness.sound_type && s.score > 70)
                && candidate_weaknesses.iter()
                    .any(|w| w.sound_type == user_strengths[0].sound_type);
            
            if complementary {
                matches.push(CollaborationMatch {
                    matched_user_id: candidate.user_id,
                    their_strength: weakness.sound_type,
                    your_strength: user_strengths[0].sound_type,
                    match_score: weakness.severity * candidate.skill_level(weakness.sound_type),
                });
            }
        }
    }
    
    matches.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score));
    matches
}
```

### 3.6 Marketplace Infrastructure (Phase 5+)

```
Graph powers marketplace features:
  - Pricing: Sounds similar to highly-exported sounds → higher price
  - Curation: Top contributors identified by graph centrality
  - Licensing: Provenance chain is the license enforcement mechanism
  - Fraud detection: Unusual graph patterns (same user exporting their own sounds repeatedly)
  - Discovery: "Users who bought this pack also bought..."
  - Creator analytics: "Your sounds are trending in the trap genre"
```

---

## 4. Graph Storage & Query

### Storage Strategy

```rust
pub struct KnowledgeGraph {
    // Nodes stored as hashmap for O(1) lookup
    nodes: HashMap<String, Box<dyn GraphEntity>>,
    
    // Adjacency list for traversal
    // node_id → { edge_type → [(target_id, weight, metadata)] }
    adjacency: HashMap<String, HashMap<EdgeType, Vec<GraphEdge>>>,
    
    // Reverse index for incoming edges
    // node_id → { edge_type → [(source_id, weight, metadata)] }
    reverse_adjacency: HashMap<String, HashMap<EdgeType, Vec<GraphEdge>>>,
    
    // Embedding index for similarity search (FAISS or pgvector)
    embedding_index: EmbeddingIndex,
    
    // Metadata
    stats: GraphStats,
}
```

### Backend Options

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| **Postgres + pgvector** | Familiar, ACID, good for moderate scale (10M nodes) | Slower for multi-hop traversals | **Beta choice** |
| Neo4j | Native graph DB, fast traversals, Cypher query language | Another service to run, licensing cost | Phase 2+ |
| **Rust in-memory graph** | Blazing fast, no network latency, local-first | Memory-bound, doesn't persist automatically | **For local queries** |
| FAISS index + Postgres | Best for similarity search, scales to billions | Two systems to maintain | Phase 3+ |

**Beta choice: Local Rust in-memory graph (for fast recommendation queries) + Postgres + pgvector (for persistence and complex queries).**

### pgvector Schema

```sql
-- Enable pgvector extension
CREATE EXTENSION vector;

-- Sound embeddings for similarity search
CREATE TABLE sound_embeddings (
    sound_id UUID PRIMARY KEY REFERENCES sounds(id),
    embedding VECTOR(256),          -- Audio embedding
    prompt_embedding VECTOR(768),   -- CLAP text embedding
    sound_type TEXT,
    sound_score REAL,
    created_at TIMESTAMPTZ
);

CREATE INDEX idx_sound_embeddings ON sound_embeddings 
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);

-- Prompt embeddings for semantic search
CREATE TABLE prompt_embeddings (
    prompt_id UUID PRIMARY KEY,
    text TEXT NOT NULL,
    embedding VECTOR(768),
    usage_count INTEGER DEFAULT 1,
    first_used TIMESTAMPTZ,
    last_used TIMESTAMPTZ
);

CREATE INDEX idx_prompt_embeddings ON prompt_embeddings
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);

-- User taste embeddings
CREATE TABLE user_taste_embeddings (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    embedding VECTOR(768),
    genre_affinity VECTOR(128),
    last_updated TIMESTAMPTZ
);

-- Similarity query: "Find sounds similar to this one"
SELECT s.id, s.sound_score, 
       se.embedding <=> $target_embedding as distance
FROM sound_embeddings se
JOIN sounds s ON s.id = se.sound_id
WHERE s.sound_type = $target_type
ORDER BY distance ASC
LIMIT 20;
```

---

## 5. Graph Population

### How the Graph Grows

```
Every user action adds to the graph:

  Generation:
    Create Sound node
    Create/update Prompt node
    Add Generated edge (User → Sound)
    Add HasPrompt edge (Sound → Prompt)
    Add HasType edge (Sound → Genre)
    Compute and add SemanticallySimilar edges (if within threshold)
    
  Export:
    Add Exported edge (User → Sound)
    Increment export count
    Update user taste embeddings
    
  Favorite:
    Add Favorited edge (User → Sound)
    Update user taste embeddings
    
  Remix:
    Create Sound node (child)
    Add RemixOf edge (child → parent)
    Add Remixed edge (child user → parent user)
    Follow same generation pattern for child
    
  Pack creation:
    Create Pack node
    Add Created edge (User → Pack)
    For each sound: Add PartOfPack edge (Sound → Pack)
    
  Comment:
    Add Commented edge (User → Sound)
    
  Daily computation:
    Update SemanticallySimilar edges (recompute as new sounds arrive)
    Update trend metrics
    Prune low-value edges (similarity < 0.3)
```

### Graph Growth Estimate

```
Scale milestones:
  Beta launch (100 users):      ~10K nodes,    ~50K edges
  1K users:                     ~100K nodes,   ~500K edges
  10K users:                    ~1M nodes,     ~5M edges
  100K users:                   ~10M nodes,    ~50M edges
  1M users:                     ~100M nodes,   ~500M edges → Need Neo4j / distributed graph

Memory estimate (Rust in-memory):
  Node: ~500 bytes → 10M nodes = ~5GB (acceptable for a service)
  Edge: ~200 bytes → 50M edges = ~10GB (acceptable)
  Embedding index: 10M × 256 × 4 bytes = ~10GB (acceptable)
  
  Total for 10M nodes: ~25GB RAM — fits on a single machine for Phase 1-3.
```

---

## 6. Graph Visualization (Internal Tool)

```
Community Sound Graph Explorer (Admin Tool):

┌─────────────────────────────────────────────────────────────────┐
│  cShot Sound Graph Explorer                                     │
│                                                                  │
│  Query: [MATCH (u:User)-[:EXPORTED]->(s:Sound) WHERE ...]       │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                                                          │    │
│  │                     Graph Visualization                  │    │
│  │                                                          │    │
│  │        (User A)───exports───►(Sound 1)───has_type──►   │    │
│  │           │                    │                        │    │
│  │           │                    │ remix_of               │    │
│  │           │                    ▼                        │    │
│  │           │              (Sound 2)───has_type──►       │    │
│  │           │                    │                        │    │
│  │           │ exports            │                        │    │
│  │           ▼                    ▼                        │    │
│  │        (User B)───exports───►(Sound 3)                  │    │
│  │                                                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  Results: 347 nodes, 1,203 edges                                │
│  Query time: 42ms                                               │
│                                                                  │
│  [Export Graph] [Save as Report] [Run on Schedule]              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 7. Graph-Powered Features Roadmap

| Feature | Powers | Phase | Complexity |
|---------|--------|-------|------------|
| Sound similarity search | Discovery | Beta | Medium |
| User co-export recommendations | "Users like you also exported..." | Beta | Low |
| Prompt pattern analysis | What prompts work best | Phase 2 | Low |
| Genre trend detection | What's rising in the community | Phase 2 | Medium |
| Remix lineage tracking | Attribution, provenance | Phase 2 | Low |
| Pack completion suggestions | "Your pack needs a snare" | Phase 2 | Medium |
| Collaboration matching | "You should work with..." | Phase 3 | High |
| Creator analytics | Dashboard for top contributors | Phase 3 | Medium |
| Taste clustering | Find users with similar taste | Phase 3 | Medium |
| Marketplace pricing | Dynamic pricing from graph signals | Phase 5 | High |
| Fraud detection | Unusual graph patterns | Phase 5 | Medium |
| Viral trend detection | Exploding prompt patterns | Phase 5 | High |

---

## 8. Summary

```
Community Sound Graph — Key Design Decisions:

  1. Every interaction is an edge: Generation, export, favorite, remix,
     comment, pack addition — all add to the graph.

  2. Embeddings enable semantic similarity: Audio embeddings (256-d)
     and text embeddings (768-d) power similarity search across
     sound and prompt space.

  3. Local + Cloud hybrid: Rust in-memory graph for fast local queries,
     Postgres + pgvector for persistence and complex analytical queries.

  4. Graph powers 7+ features: Discovery, recommendations, attribution,
     trend detection, collaborative creation, marketplace, analytics.

  5. Scales with the community: In-memory graph handles 10M nodes,
     Neo4j transition planned for 100M+.

  6. Privacy-aware: User identities are hashed in public graph views.
     Raw embeddings never expose individual preference data.
```
