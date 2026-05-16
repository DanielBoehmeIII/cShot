# Prompt 33 — Copyright-Safe Sound Generation

cShot must be legally defensible, commercially safe, and user-trustworthy from day one.

---

## 1. Risk Analysis

### Risk Categories

| Risk | Severity | Likelihood | Description |
|------|----------|------------|-------------|
| Sample copyright | Critical | High | Model reproduces training samples verbatim |
| Interpolation risk | High | Medium | Model generates sounds that are "too close" to copyrighted works |
| Training data risk | Critical | Medium | Training on copyrighted material without license |
| Style imitation | Medium | High | User prompts to imitate a specific artist's sound |
| Reconstruction risk | High | Medium | Model can be prompted to reconstruct known copyrighted sounds |
| Derivative work claims | Medium | Low | Generated sound similar enough to trigger derivative work claim |
| Trademark risk | Low | Medium | Generated sound includes trademarked sonic elements |
| Moral rights | Low | Low | Claim of unauthorized imitation despite no copyright violation |
| User data leakage | Medium | Low | User's reference audio leaks through model or is stored improperly |
| Platform liability | High | Medium | Hosting/model-provider liability for copyright infringement |

### Legal Landscape

```
Current state (2025-2026):
  - US Copyright Office: AI-generated works can be copyrighted if 
    human authorship is "sufficiently creative"
  - EU AI Act: Requires transparency for training data, opt-out for 
    copyrighted works
  - UK: Proposed exception for AI training on copyrighted works for 
    research, unclear for commercial
  - Japan: More permissive, allows training on copyrighted works
  - China: Requires licensing for training data, government oversight
  
Key precedents:
  - Getty Images v. Stability AI (ongoing): Training on copyrighted 
    images without license
  - Authors Guild v. OpenAI (ongoing): Text generation that 
    reproduces copyrighted works
  - Anderson v. Stability AI (ongoing): Style imitation claims
  
For audio specifically:
  - Sound recordings have separate copyright from compositions
  - Sampling without clearance is infringement (Bridgeport Music, 2004)
  - No clear precedent yet for AI-generated audio imitation
  - "Sound-alike" recordings are generally legal, but imitation of 
    the specific recording is not
```

### The cShot Position

```
cShot's legal strategy:
  1. Train only on licensed, public domain, or synthetic data
  2. Prevent output-side reproduction and similarity
  3. Full provenance tracking for every generated sound
  4. Transparent to users about what's safe for commercial use
  5. API-based similarity checking against known copyrighted works
  6. User assumes responsibility for prompt choice (style prompting)
  
cShot is NOT:
  - A tool to replicate existing copyrighted recordings
  - A "clone" of any artist's sample library
  - Designed to evade copyright protections
  
cShot IS:
  - A tool to create original sounds
  - A creativity amplification system
  - Built from the ground up with legal safety
```

---

## 2. Dataset Rules

### Dataset Sourcing Policy

```
Tier 1 — Safe (always allowed):
  - Public domain audio (pre-1928 recordings, government works)
  - Creative Commons Zero (CC0) licensed samples
  - Original recordings produced by cShot team
  - Synthetic audio generated algorithmically (no IP claims)
  - Licensed sample packs with explicit AI-training permission

Tier 2 — Conditional (case-by-case review):
  - Creative Commons BY (attribution required, track in metadata)
  - License-encumbered with specific AI-training allowance
  - User-uploaded content (with explicit license grant to cShot)
  - API-accessible datasets with permissive terms

Tier 3 — Prohibited (never allowed):
  - Commercial music recordings (major label catalog)
  - Sample packs without AI-training permission
  - Copyrighted sound effects libraries
  - User-uploaded content without rights verification
  - YouTube/streaming rips
  - Peer-to-peer shared sample packs
```

### Data Provenance Requirements

```
Every training sample must have:
  ✓ Source URL or origin document
  ✓ License type and version
  ✓ Rights holder (if known)
  ✓ Date of acquisition
  ✓ Terms of use for ML training (explicit)
  ✓ Any attribution requirements
  ✓ Any usage restrictions (non-commercial only, etc.)
  ✓ Contact information for rights inquiries

This is stored in a provenance database, not just a README.
```

### Training Data Filters

```python
class TrainingDataFilter:
    """Multi-stage filter for training data safety."""
    
    @staticmethod
    def deduplicate(samples):
        """Remove near-identical samples to prevent overfitting."""
        # Perceptual hash comparison
        # Embedding similarity > 0.95 → flag as duplicate
        # Remove all but one from each cluster
        pass
    
    @staticmethod
    def screen_copyrighted(samples):
        """Screen against known copyrighted works database."""
        # Perceptual matching against reference library
        # If similarity > 0.8 with any known copyrighted work → reject
        # If similarity > 0.6 → flag for human review
        pass
    
    @staticmethod
    def filter_low_quality(samples):
        """Remove samples that don't meet quality bar."""
        # SNR < 20dB → reject
        # Clipping detected → reject
        # Duration < 50ms → reject (likely truncation artifact)
        # Obvious noise/static → reject
        pass
    
    @staticmethod
    def verify_license(samples):
        """Verify license metadata is present and valid."""
        # Must have license field
        # Must have source field
        # If license requires attribution, store attribution text
        # If license explicitly prohibits ML training → reject
        pass
```

---

## 3. Generation Safety Checks

### Output-Side Safety

Generation safety is not just about training data. It's about what the model produces.

```
┌─────────────┐    ┌────────────────┐    ┌──────────────┐
│  Model       │───→│  Safety Filter │───→│  Output to   │
│  Generates   │    │  Pipeline      │    │  User        │
└─────────────┘    └────────────────┘    └──────────────┘
                          │
                          ↓
                    ┌──────────────┐
                    │  Blocked?    │──→ Log, notify user, offer alternative
                    └──────────────┘
```

### Safety Filter Pipeline

```python
def safety_filter_pipeline(generated_audio, prompt, params):
    """
    Run generated audio through multiple safety checks.
    Returns (safe, warnings, metadata).
    """
    checks = [
        verbatim_match_check(generated_audio),      # Exact reproduction?
        near_duplicate_check(generated_audio),       # Too similar to training?
        copyrighted_similarity(generated_audio),     # Matches copyrighted work?
        prompt_safety_check(prompt),                 # Prompt asking for infringement?
        reconstruction_attack_check(prompt, audio),  # Prompt trying to extract training data?
    ]
    
    failed = [c for c in checks if not c.passed]
    
    if any(c.severity == 'critical' for c in failed):
        return BLOCKED, failed
    elif any(c.severity == 'warning' for c in failed):
        return WARNING, failed
    else:
        return SAFE, []
```

### Specific Checks

#### 3.1 Verbatim Match Detection

```
Goal: Detect if model reproduces a training sample exactly.

Method:
  1. Compute spectrogram of generated audio
  2. Perceptual hash (pHash) of spectrogram
  3. Compare against index of all training sample hashes
  4. If exact match → BLOCKED
  
Threshold:
  - pHash Hamming distance < 2 → exact match
  - Also check raw sample correlation > 0.99
  
Action:
  - BLOCK output
  - Log incident (model version, prompt, params, matched training sample ID)
  - Notify user: "This sound closely matches existing copyrighted material."
  - Generate alternative
```

#### 3.2 Near-Duplicate Detection

```
Goal: Detect if generated sound is "too close" to any training sample.

Method:
  1. Compute Sound DNA embedding of generated audio
  2. Find nearest neighbor in training set
  3. If cosine similarity > threshold → flag
  
Thresholds:
  - Similarity > 0.95: BLOCKED (effectively identical)
  - Similarity > 0.85: WARNING (suspiciously similar)
  - Similarity > 0.75: INFO (log only)
  
Action (for WARNING level):
  - Show user: "This sound is similar to [source]"
  - Offer to regenerate with more variation
  - Log for review
```

#### 3.3 Copyrighted Work Similarity

```
Goal: Match against known copyrighted works (not just training set).

Method:
  1. Query external similarity service (or local database)
  2. Compare against catalog of commercial sound recordings
  3. Check against registered sample packs
  4. Check against opt-out registry
  
Sources for reference database:
  - Public catalog of registered works
  - Publisher-provided reference library
  - User opt-out submissions
  - Licensed third-party similarity API (e.g., Audible Magic, Gracenote)
  - Crawled public registries
  
Action:
  - Similarity > 0.80 → BLOCKED + notify user
  - Similarity > 0.65 → WARNING + suggest different direction
  - Below threshold → PASS
```

#### 3.4 Prompt Safety Check

```
Goal: Detect if user is trying to infringe copyright.

Blocked prompts:
  - "Make the kick from [song name] by [artist]"
  - "Recreate the [album] snare sound"
  - "Exactly like the sample in [popular track]"
  - "Clone the [specific sample pack] kick"
  - "[artist] style but note-for-note their [song] kick"
  
Allowed prompts (style reference is OK):
  - "Make a dark trap kick like [artist] would use"
  - "Aggressive techno kick in the style of [genre]"
  - "Give it a [producer]-inspired distortion sound"
  
Distinction: 
  - Requesting a SPECIFIC copyrighted sound → blocked
  - Requesting a STYLE or GENRE → allowed
  - "Like [artist]" = allowed (style reference)
  - "The kick from [artist's song]" = blocked (specific work)
```

#### 3.5 Reconstruction Attack Detection

```
Goal: Prevent users from extracting training data via prompts.

Known attack patterns:
  - "Generate the first 3 seconds of [copyrighted song]"
  - Prompt engineering to extract specific training samples
  - Iterative refinement toward a target sound
  
Detection:
  - Monitor generation similarity over time
  - If user is iteratively converging toward a known work → flag
  - Rate-limit re-generation of "more like this" on suspicious trajectories
  
Defense:
  - Apply differential privacy noise to latent space queries
  - Limit latent space navigation toward known embedding regions
  - Randomize seed on repeated "more like this" requests
```

---

## 4. Similarity Detection System

### Architecture

```
┌──────────────┐
│  Generated   │
│  Audio       │
└──────┬───────┘
       ↓
┌──────────────┐
│  Embedding   │ ← Sound DNA model (768-dim)
│  Extraction  │
└──────┬───────┘
       ↓
┌─────────────────────────────────────┐
│         Similarity Index            │
│  ┌─────────────┐ ┌───────────────┐ │
│  │ Training    │ │ Copyrighted   │ │
│  │ Data Index  │ │ Works Index   │ │
│  │ (local)     │ │ (local + API) │ │
│  └─────────────┘ └───────────────┘ │
│  ┌─────────────┐ ┌───────────────┐ │
│  │ User        │ │ Public Domain │ │
│  │ Library     │ │ Reference     │ │
│  └─────────────┘ └───────────────┘ │
└─────────────────────────────────────┘
       ↓
┌──────────────┐
│  Risk Score  │
│  + Action    │
└──────────────┘
```

### Indexing Strategy

```
All indexes use approximate nearest neighbor (ANN) search.

Index types:
  - HNSW (Hierarchical Navigable Small World): for fast, accurate search
  - Product Quantization: for memory-efficient storage at scale

Scale targets:
  - Training data index: 1M+ embeddings → ~2GB RAM (PQ-compressed)
  - Copyrighted works index: 10M+ → ~20GB RAM
  - User library index: per-user, up to 100K → ~200MB
  - Public domain reference: 500K → ~1GB

Search time: <10ms per index, <50ms for full pipeline
```

### Similarity Score

```python
def compute_similarity_score(generated_embedding, reference_embedding):
    """
    Compute a human-interpretable similarity score.
    Returns 0.0 (completely different) to 1.0 (identical).
    """
    cosine_sim = cosine_similarity(generated_embedding, reference_embedding)
    
    # Scale: map from [0.5, 1.0] to [0.0, 1.0]
    # (below 0.5 cosine similarity is essentially unrelated)
    if cosine_sim < 0.5:
        return 0.0
    
    scaled = (cosine_sim - 0.5) / 0.5
    return min(1.0, max(0.0, scaled))
```

---

## 5. Provenance Metadata

### Metadata Schema

Every generated sound carries this metadata (in WAV file, database, and API response):

```json
{
  "cshot_provenance": {
    "version": "1.0",
    "generated_at": "2026-05-15T14:30:00Z",
    "model": {
      "name": "cshot-generator-v1",
      "version": "1.2.3",
      "architecture": "diffusion-transformer",
      "training_dataset": "cshot-dataset-v3",
      "training_cutoff": "2026-04-01"
    },
    "generation": {
      "prompt": "punchy trap kick with short decay",
      "seed": 424242,
      "params": {
        "bpm": 140,
        "key": "C#m",
        "temperature": 0.8,
        "cfg_scale": 7.5,
        "duration_seconds": 0.5,
        "stereo_width": 0.0
      },
      "inference_time_ms": 2340,
      "hardware": "NVIDIA RTX 4090"
    },
    "similarity_checks": {
      "training_data_max_similarity": 0.32,
      "copyrighted_works_max_similarity": 0.28,
      "user_library_max_similarity": 0.45,
      "all_clear": true
    },
    "provenance_chain": [
      {
        "type": "generation",
        "timestamp": "2026-05-15T14:30:00Z",
        "description": "Initial generation from prompt"
      }
    ],
    "licensing": {
      "status": "commercial_safe",
      "confidence": 0.97,
      "explanation": "No significant similarity to any known copyrighted work. Generated from licensed training data.",
      "recommended_attribution": "Generated with cShot",
      "restrictions": []
    },
    "audio_signature": {
      "sha256": "a1b2c3d4e5f6...",
      "duration_seconds": 0.487,
      "sample_rate": 44100,
      "channels": 1,
      "perceptual_hash": "pHash_abc123..."
    }
  }
}
```

---

## 6. Licensing Model

### User-Facing License Tiers

```
┌──────────────────────────────────────────────────────────────┐
│                   cShot Licensing Model                       │
├─────────────┬──────────────┬─────────────┬───────────────────┤
│             │ Free Tier    │ Pro Tier    │ Enterprise Tier   │
├─────────────┼──────────────┼─────────────┼───────────────────┤
│ Monthly gen │ 100 sounds   │ 1000 sounds │ Unlimited         │
│ Commercial  │ Yes (with    │ Yes (full)  │ Yes (full)        │
│ use         │ attribution) │             │                   │
│ Output      │ CC BY-NC     │ cShot Pro   │ Custom license    │
│ license     │              │ License     │                   │
│ Similarity  │ Basic check  │ Full check  │ Full + enterprise │
│ checking    │              │             │ reference DB      │
│ Provenance  │ Basic        │ Full        │ Full + audit trail│
│ metadata    │              │             │                   │
│ Indemnifica-│ No           │ Limited     │ Full legal        │
│ tion        │              │ ($10K cap)  │ indemnification   │
│ Private     │ No           │ No          │ Yes (air-gapped)  │
│ deployment  │              │             │                   │
│ API access  │ No           │ Yes         │ Yes + SLA         │
└─────────────┴──────────────┴─────────────┴───────────────────┘
```

### License Texts

#### Free Tier (CC BY-NC 4.0 + cShot addendum)

```
You may:
  - Use generated sounds in non-commercial projects
  - Share with attribution "Generated with cShot"
  - Modify and adapt

You may NOT:
  - Use in commercial releases without upgrading
  - Claim the sound as entirely your own creation
  - Use to create competing AI training data

Commercial use requires attribution: "Sounds generated with cShot"
```

#### Pro Tier (cShot Pro License)

```
You may:
  - Use in commercial releases (tracks, films, games, etc.)
  - Modify, adapt, and sublicense
  - Distribute as part of larger works
  - Register copyright on your final work (including the sound)

You may NOT:
  - Redistribute generated sounds as standalone sample packs
  - Use generated sounds to train competing AI models
  - Claim trademark on generated sounds

No attribution required for commercial use.
Limited indemnification: cShot will defend against third-party 
copyright claims if similarity checks passed at generation time.
```

#### Enterprise Tier

```
Custom terms including:
  - Full legal indemnification
  - Private deployment (on-premises, air-gapped)
  - Custom training data (optional: include your own sounds)
  - Custom similarity reference database (your IP only)
  - Audit-ready provenance for legal/compliance
  - Volume pricing
  - SLA guarantees
```

---

## 7. User-Facing Trust System

### Trust Indicators

Every sound shows a trust badge:

```
┌───────────────────────┐
│  ✅ Commercial-Safe   │
│  Generated: 2 min ago │
│  Model: cShot v1.2.3  │
│  Checked: 3 databases │
└───────────────────────┘

┌───────────────────────┐
│  ⚠️ Similar to        │
│  "example_pack.wav"   │
│  (similarity: 0.72)   │
│  Review recommended   │
└───────────────────────┘

┌───────────────────────┐
│  🚫 BLOCKED           │
│  Too similar to       │
│  copyrighted work     │
│  Similarity: 0.88     │
│  Regenerating...      │
└───────────────────────┘
```

### Trust Dashboard

```
Settings → Trust & Safety:

┌────────────────────────────────────────────────────┐
│  cShot Trust Dashboard                             │
├────────────────────────────────────────────────────┤
│  Trust Overview:                                   │
│  ✓ 98.2% of your sounds are commercial-safe        │
│  ✓ 1.6% have warnings (reviewed, no issues)        │
│  ⚠ 0.2% were blocked (auto-regenerated)            │
│                                                    │
│  Similarity Checks Active:                         │
│  ✓ Training data index (1,234,567 sounds)          │
│  ✓ Copyrighted works index (8,765,432 sounds)      │
│  ✓ Your library index (2,345 sounds)               │
│  ─ Opt-out registry (enabled)                      │
│                                                    │
│  Your Exports This Month:                          │
│  47 sounds exported, 100% commercial-safe          │
│  0 flags, 0 disputes, 0 takedowns                 │
│                                                    │
│  [Download Full Audit Report]  [View Recent Flags] │
└────────────────────────────────────────────────────┘
```

### Opt-Out Registry

```
Rights holders can submit sounds to cShot's opt-out registry:

  1. Submit: audio file + proof of rights
  2. Verification: automated + human review
  3. If accepted: hash added to opt-out index
  4. Effect: cShot will never generate sounds similar to 
     this, and will block if attempted

This is NOT:
  - A backdoor to check if cShot can reproduce your work
  - A way to see what's in the training data
  - A copyright registration system

This IS:
  - A good-faith system for rights protection
  - Automated and scalable
  - Free for rights holders
```

### Incident Response

```
If a user receives a copyright claim:

  1. User submits claim via Trust Dashboard
  2. cShot investigates within 24 hours:
     - Check generation provenance
     - Re-run similarity checks at time of claim
     - Compare against claimed work
  3. Outcomes:
     - False claim → Provide audit report to user + claimant
     - Legitimate issue → Identify scope, notify affected users
     - If our fault → Regenerate, update safety system, compensate affected users
  4. Audit report is legally admissible:
     - Timestamped, signed logs
     - Model version pinning
     - Complete generation parameter chain
     - Similarity check results at generation time
```

### Transparency Reports

Published quarterly:

```
cShot Transparency Report Q2 2026:

  Total sounds generated:     4,237,891
  Sounds blocked:             8,432 (0.20%)
  Warnings shown:            62,194 (1.47%)
  Commercial-safe rate:       98.33%
  
  Copyright claims received:  3
  Claims upheld:              0
  Claims dismissed:           3
  
  Opt-out submissions:        1,234
  Opt-out accepted:           892
  Opt-out rejected:           342 (insufficient rights proof)
  
  Training data updates:      2 new datasets, 0 removals
  Model versions:             1 major, 3 minor updates
  
  Incident timeline:
    - No safety incidents this quarter
    - One false positive flag fixed (update v1.2.1)
```

---

## Summary

| Component | What It Protects | How It Works |
|-----------|-----------------|--------------|
| Dataset rules | Training integrity | Tiered sourcing, provenance tracking, dedup |
| Generation safety | Output integrity | 5-stage filter pipeline |
| Similarity detection | Copyright owners | ANN search across multiple indexes |
| Provenance metadata | User liability | Complete generation audit trail |
| Licensing model | All parties | Tiered, clear, enforceable |
| Trust system | User confidence | Badges, dashboard, opt-out, incident response |

The goal is not just legal compliance — it's user trust. A producer should feel confident that every cShot sound is safe to use in a commercial release, and that if any issue arises, there's a clear, auditable chain of custody.
