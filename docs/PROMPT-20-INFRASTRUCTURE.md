# Prompt 20 — Build the Infrastructure for the Future of Sound Design

cShot: the operating system for future sound design.

---

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           cShot Platform Architecture                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      User Layer                                       │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │   │
│  │  │  DAW      │ │  Web App │ │  Mobile  │ │  CLI     │ │  API     │  │   │
│  │  │  Plugin   │ │  (React) │ │  (Swift) │ │  (Python)│ │  (REST)  │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      Service Layer                                   │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │   │
│  │  │  Auth    │ │  Search  │ │  Gen.    │ │  Evolve  │ │  Collab  │  │   │
│  │  │  Service │ │  Service │ │  Service │ │  Service │ │  Service │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      Inference Layer                                 │   │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐       │   │
│  │  │  GPU Farm  │ │  Edge/     │ │  DSP       │ │  Model     │       │   │
│  │  │  (Cloud)   │ │  On-device │ │  Engine    │ │  Registry  │       │   │
│  │  └────────────┘ └────────────┘ └────────────┘ └────────────┘       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      Data Layer                                      │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │   │
│  │  │  Vector  │ │  Audio   │ │  Object  │ │  Cache   │ │  Event   │  │   │
│  │  │  DB      │ │  Store   │ │  Store   │ │  Layer   │ │  Stream  │  │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Storage Architecture

### 2.1 Vector Database (Primary Index)

**Purpose**: DNA embedding (768-D) + perceptual embedding (128-D) + text embedding (768-D)

**Technology**: Qdrant / Milvus (self-hosted) + pgvector for relational integration

**Scale**: 1B+ vectors (DNA), 100B+ with future growth

**Index strategy**:
```
- HNSW (Hierarchical Navigable Small World) for exact-ish search (recall 0.99)
- Product quantization (PQ) for memory reduction (4x compression)
- IVF-PQ for billion-scale (coarse + fine search)
```

**Sharding**: By embedding type and recency. Hot shards (recent/trending) in memory, cold on SSD.

### 2.2 Audio Store

**Technology**: S3-compatible (MinIO self-hosted, or AWS S3/Cloudflare R2)

**Organization**:
```
samples/{category}/{subcategory}/{id}.wav
  e.g., samples/kick/808_kick/a1b2c3d4.wav
```

**Formats**:
- Master: 44.1kHz/24-bit FLAC (lossless)
- Preview: 22.05kHz/16-bit OGG Vorbis (streaming)
- Feature: pre-computed features alongside audio

**Tiering**:
```
Hot tier (SSD):   10% most-accessed samples   → <1ms access
Warm tier (S3):   90% of samples              → 10-50ms access
Cold tier (Glacier): Backup/archived          → minutes
```

### 2.3 Object Store (Metadata & User Data)

**Technology**: PostgreSQL with JSONB

**Schema**:
```sql
-- Core samples table
CREATE TABLE samples (
    id UUID PRIMARY KEY,
    file_path TEXT NOT NULL,
    duration FLOAT,
    sample_rate INT,
    channels INT,
    sound_type TEXT[],
    genres JSONB,  -- {genre: confidence, ...}
    perceptual_features JSONB,
    emotional_features JSONB,
    dna_embedding_id UUID REFERENCES embeddings(id),
    license TEXT,
    source TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

-- Embeddings (large, separate table)
CREATE TABLE embeddings (
    id UUID PRIMARY KEY,
    vector FLOAT[] NOT NULL,  -- 768-dim
    model_version TEXT,
    indexed_at TIMESTAMP
);

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY,
    taste_profile JSONB,
    library_ids UUID[],
    created_at TIMESTAMP
);

-- User interactions
CREATE TABLE interactions (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    sample_id UUID REFERENCES samples(id),
    interaction_type TEXT,  -- generated, saved, played, modified, exported
    feedback FLOAT,  -- -1 to 1
    context JSONB,
    created_at TIMESTAMP
);
```

### 2.4 Cache Layer

**Technology**: Redis Cluster

**Cached items**:
- Popular search results (TTL: 60s)
- Recently generated sounds (TTL: 300s)
- User sessions (TTL: 24h)
- Frequent query embeddings (TTL: 3600s)
- Hot vector index partitions (LRU)

---

## 3. Inference Architecture

### 3.1 GPU Inference (Cloud)

```
┌──────────────────────────────────────────────┐
│              GPU Inference Cluster             │
│                                                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐      │
│  │  NVIDIA  │ │  NVIDIA  │ │  NVIDIA  │ ...  │
│  │  A100    │ │  A100    │ │  L40S    │      │
│  └──────────┘ └──────────┘ └──────────┘      │
│                                                │
│  Triton Inference Server (TensorRT)            │
│  - Perceptual embedding model (batch)          │
│  - Emotional embedding model (batch)          │
│  - Diffusion refinement (premium tier)        │
│  - Text embedding model                       │
└──────────────────────────────────────────────────┘
```

**Model serving**:
- ONNX Runtime / TensorRT for optimal inference
- Dynamic batching for embedding extraction
- Diffusion refinement: 1 request per GPU, 10 steps
- Target: 1000 embedding queries/sec, 10 premium generates/sec

### 3.2 Edge/On-Device Inference

```python
class LocalInferenceEngine:
    """Runs cShot entirely on-device."""
    
    def __init__(self):
        self.dsp_engine = DSPEngine()  # Prompt 18, ~0 params
        self.neural_frontend = onnx.load("frontend.onnx")  # 5M params
        self.dna_encoder = onnx.load("dna_encoder.onnx")  # 10M params
        
    def generate(self, prompt):
        params = self.neural_frontend(prompt)
        audio = self.dsp_engine.render(params)
        return audio
    
    def analyze(self, audio):
        return self.dna_encoder(audio)
    
    def search_local(self, query_embedding, library_embeddings):
        # Simple cosine search, no vector DB needed
        scores = cosine_similarity(query_embedding, library_embeddings)
        top_k = argsort(scores, descending=True)[:10]
        return top_k
```

**On-device capabilities**:
- Quick generation (DSP only): any device
- Standard generation (DSP + small NN): modern laptop/phone
- DNA encoding: modern laptop/phone
- Local search (up to 10K sounds): any device
- Premium generation (DSP + diffusion): GPU laptop only

### 3.3 Streaming Generation

For DAW integration, generation must be real-time:

```python
class StreamingGenerator:
    """Generate audio in streaming mode for DAW use."""
    
    def __init__(self):
        self.dsp_engine = DSPEngine()
        self.buffer_size = 256  # samples
        self.latency = self.buffer_size / 44100 * 1000  # ~5.8ms
    
    def generate_block(self, dsp_params):
        """Generate one block of audio for streaming."""
        block = self.dsp_engine.render_block(dsp_params, self.buffer_size)
        return block
    
    def set_parameter(self, param, value):
        """Change parameter on-the-fly (sample-accurate)."""
        self.dsp_engine.set_param(param, value)
```

---

## 4. Retrieval Architecture

### 4.1 Multi-Stage Retrieval

```python
class RetrievalPipeline:
    """Cascaded retrieval for speed + accuracy."""
    
    def search(self, query, n=10):
        # Stage 1: Coarse search (millions → thousands)
        candidates = self.vector_db.search(
            query.embedding, 
            k=1000, 
            index='coarse', 
            ef=128
        )
        
        # Stage 2: Metadata filter (thousands → hundreds)
        if query.filters:
            candidates = self.metadata_filter(candidates, query.filters)
        
        # Stage 3: Fine re-rank (hundreds → tens)
        if query.re_rank:
            candidates = self.re_rank(candidates, query.embedding)
        
        # Stage 4: Diversity (optional)
        if query.diverse:
            candidates = self.maximal_marginal_relevance(candidates, query.embedding, n)
        
        return candidates[:n]
```

### 4.2 Semantic Caching

```python
class SemanticCache:
    """Cache similar query results to avoid redundant generation."""
    
    def __init__(self):
        self.cache = {}  # embedding_hash -> (result, timestamp)
        self.similarity_threshold = 0.95
        
    def get(self, query_embedding):
        for cached_emb, (result, ts) in self.cache.items():
            if cosine_similarity(query_embedding, cached_emb) > self.similarity_threshold:
                return result
        return None
    
    def put(self, query_embedding, result):
        key = hash(query_embedding.tobytes())
        self.cache[key] = (result, time())
```

### 4.3 Hybrid Search

```
Text Query: "dark punchy kick"
    ↓
┌──────────────────────────────────────────────┐
│              Hybrid Search                     │
│                                                │
│  Text Encoder (CLAP / MuLan)                  │
│       → text_embedding (768-D)                 │
│                                                │
│  Vector DB Search (by embedding)              │
│       → candidates_A                          │
│                                                │
│  Metadata Filter (genre=techno, type=kick)    │
│       → candidates_B                          │
│                                                │
│  Fusion: weighted combination                 │
│       → ranked results                         │
└──────────────────────────────────────────────────┘
```

---

## 5. Deployment Architecture

### 5.1 Hybrid Local + Cloud

```
┌──────────────────────────────────────────────────────────────────┐
│                         Deployment Model                           │
│                                                                    │
│  Local (primary)                    Cloud (augmentation)           │
│  ┌───────────────────┐             ┌─────────────────────┐        │
│  │ DSP Engine        │  offline    │ GPU Inference        │        │
│  │ Small NN models   │  ────────▶  │ Premium Generation   │        │
│  │ Local vector DB   │  ◀────────  │ Large embedding DB   │        │
│  │ User library      │  sync       │ Collaborative        │        │
│  │ Quick generation  │             │ Community features   │        │
│  └───────────────────┘             └─────────────────────┘        │
│                                                                    │
│  "Local-first, cloud-augmented"                                   │
│  - 80% of operations are local (fast, private, offline)          │
│  - 20% use cloud (premium quality, community, sync)              │
│  - Full functionality without internet                           │
└──────────────────────────────────────────────────────────────────────┘
```

### 5.2 Sync Protocol

```python
class SyncManager:
    """Synchronize local and cloud state."""
    
    def sync_library(self):
        # Push: local changes to cloud
        new_samples = self.get_local_new_since(self.last_sync)
        self.cloud_api.upload_samples(new_samples)
        
        # Pull: cloud changes to local
        updated = self.cloud_api.get_updated_since(self.last_sync)
        for sample in updated:
            self.local_store.save(sample)
        
        # Pull: taste model updates
        cloud_taste = self.cloud_api.get_taste_model(self.user_id)
        self.local_taste.merge(cloud_taste)
        
        self.last_sync = now()
```

### 5.3 Infrastructure as Code

```yaml
# k8s deployment (simplified)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cshot-gpu-inference
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cshot-inference
  template:
    spec:
      containers:
      - name: triton
        image: cshot/triton-server:latest
        resources:
          limits:
            nvidia.com/gpu: 1
        env:
        - name: MODEL_PATH
          value: "s3://cshot-models/production/"
---
apiVersion: v1
kind: Service
metadata:
  name: cshot-api
spec:
  ports:
  - port: 443
    targetPort: 8080
  selector:
    app: cshot-api
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: cshot-inference-hpa
spec:
  minReplicas: 2
  maxReplicas: 20
  metrics:
  - type: Pods
    pods:
      metric:
        name: inference_queue_depth
      target:
        type: AverageValue
        averageValue: 10
```

---

## 6. Indexing Architecture

### 6.1 Index Build Pipeline

```
Raw audio → DNA encoder → 768-D embedding → PQ compression
                                                ↓
                                         Train coarse quantizer
                                                ↓
                                         Build IVF index (1M centroids)
                                                ↓
                                         Build HNSW graph per cluster
                                                ↓
                                         Write to disk (sharded by category)
                                                ↓
                                         Load into memory (hot shards first)
```

### 6.2 Index Updates (Online)

```python
class IndexUpdater:
    """Handle embedding index updates without full rebuild."""
    
    def __init__(self, vector_db):
        self.vector_db = vector_db
        self.pending = []
        self.rebuild_threshold = 10000  # rebuild after 10K new vectors
        
    def add_embedding(self, embedding, metadata):
        self.pending.append((embedding, metadata))
        
        if len(self.pending) < 100:
            # Fast path: just append to current index
            self.vector_db.insert(embedding, metadata)
        else:
            # Batch insert
            self.vector_db.bulk_insert(self.pending)
            self.pending = []
        
        if self.vector_db.size() > self.rebuild_threshold:
            self.schedule_rebuild()
```

### 6.3 Multi-Modal Index

```
Index Types:
  - DNA Index (768-D): Primary acoustic similarity
  - Text Index (768-D): Text-to-sound similarity (CLAP)
  - Perceptual Index (128-D): Perceptual feature similarity
  - Metadata Index: Exact-match filtering (type, genre, etc.)

Query types:
  - acoustic(query_audio): DNA Index + re-rank
  - text("punchy kick"): Text Index + Metadata filter
  - perceptual(punch=0.8): Perceptual Index
  - hybrid(text + audio): Fusion of DNA + Text indexes
```

---

## 7. Distributed Audio Intelligence

### 7.1 Collaborative Evolution

```
User A's taste ──┐
User B's taste ──┼── Global taste model ──→ Better generation for all
User C's taste ──┘

Privacy: taste model is a compressed vector (128-D), not raw data.
Federated: each user's taste stays on-device; only anonymous gradients shared.
```

### 7.2 Event Stream Architecture

```python
# All system events streamed through Kafka
events = {
    'sound_generated':  {'user', 'prompt', 'sound_id', 'latency', 'model'},
    'sound_saved':      {'user', 'sound_id', 'timestamp'},
    'sound_modified':   {'user', 'sound_id', 'modifications'},
    'search_conducted': {'user', 'query', 'results', 'latency'},
    'feedback_given':   {'user', 'sound_id', 'rating'},
    'taste_updated':    {'user', 'model_version'},
    'generation_failed': {'user', 'error', 'context'},
}

# Stream consumers:
# - Real-time analytics
# - Taste model training pipeline
# - Anomaly detection
# - Usage billing
# - Recommendation updates
```

---

## 8. Scaling Targets

| Metric | Launch | Year 1 | Year 2 | Year 3 |
|--------|--------|--------|--------|--------|
| Users | 1,000 | 50,000 | 500,000 | 5,000,000 |
| Sounds in library | 100K | 10M | 100M | 1B |
| Embeddings indexed | 100K | 10M | 100M | 1B |
| Generations/day | 10K | 1M | 10M | 100M |
| Searches/day | 1K | 100K | 1M | 10M |
| GPU nodes | 2 | 10 | 50 | 200 |
| Storage (audio) | 1TB | 100TB | 1PB | 10PB |
| Vector DB size | 100GB | 10GB* | 100GB* | 1TB* |

*After PQ compression. Uncompressed would be 100x larger.

---

## 9. Technology Choices

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Vector DB | Qdrant (self-hosted) | Open source, Rust, fast, disk-based overflow |
| Audio store | MinIO (S3 compatible) | Self-hosted, scalable, cheap |
| Metadata DB | PostgreSQL + pgvector | Reliable, vector capabilities, JSON support |
| Cache | Redis Cluster | Fast, proven, available everywhere |
| GPU inference | Triton + TensorRT | Production-grade, multi-model, dynamic batching |
| Orchestration | Kubernetes (k3s) | Lightweight, edge-compatible |
| Message queue | NATS (edge) / Kafka (cloud) | Fast edge messaging, durable cloud streaming |
| Model format | ONNX | Universal, cross-platform, hardware-optimized |
| Edge runtime | ONNX Runtime Mobile | Runs on iOS/Android, small footprint |
| Sync protocol | gRPC + Delta sync | Efficient bi-directional sync |
| Monitoring | Prometheus + Grafana | Open source, battle-tested |

---

## 10. The Operating System Vision

cShot is designed to be the **platform layer** that future sound design tools are built on:

```
cShot provides:
  ✓ Sound generation (DSP + AI)
  ✓ Sound understanding (perceptual, emotional, latent)
  ✓ Sound search (semantic, acoustic, hybrid)
  ✓ Sound evolution (mutation, crossover, lineage)
  ✓ Sound storage (at scale, forever)
  ✓ Sound sync (local-first, cloud-augmented)
  ✓ Sound collaboration (shared evolution, taste sharing)
  ✓ Sound API (for other tools to build on)

cShot does NOT provide:
  ✗ DAW functionality (let Ableton/FL Studio/Cubase do that)
  ✗ Audio recording/editing (specialized tools exist)
  ✗ Mixing/mastering (different domain)
  ✗ UI/visual design (beyond sound interaction)
  
Instead, cShot INTEGRATES with:
  → DAWs via plugin (VST3, AU, AAX)
  → Production tools via API
  → Sample libraries as their intelligence layer
  → Hardware via embedded runtime
```
