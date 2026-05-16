# Prompt 34 — Provenance and Sonic Chain of Custody

Every sound tells a story. cShot makes that story visible, verifiable, and legally useful.

---

## 1. What Provenance Tracks

### The Provenance Record

Every generated sound carries a complete audit trail of its existence:

```
Input Sources:
  - Original prompt text (normalized, with timestamp)
  - Reference audio files (hash + metadata, NOT the file itself)
  - User parameters and settings at generation time
  - Any imported samples or MIDI data

Transformations:
  - Model inference parameters (seed, temperature, CFG scale, steps)
  - DSP processing chain (EQ, compression, reverb, etc.)
  - Any user edits (trim, fade, pitch shift, time stretch)
  - Layer operations (sound A + sound B blended)
  - Mutation/variant operations (parent sound hashes)

Model Information:
  - Model version (exact: "cshot-generator-v1.2.3-20260501")
  - Architecture name
  - Training dataset version
  - Quantization level (FP32, FP16, INT8)
  - Any LoRA/adapters applied

Generation Parameters:
  - Duration, sample rate, bit depth
  - Target BPM and key (from DAW or user-set)
  - Stereo width, pan position
  - Output format settings

Similarity Scores:
  - Max similarity to training data
  - Max similarity to copyrighted works DB
  - Max similarity to user's own library
  - All-clear status

Export History:
  - Every export event (timestamp, format, destination)
  - DAW session name (if exported from plugin)
  - Project context (BPM, key, genre)
  - User who exported (if multi-user system)
  - Any post-generation modifications

Licensing State:
  - Current license tier at time of generation
  - Commercial-safe determination
  - Any restrictions flagged
  - Recommended attribution
```

---

## 2. Metadata Schema

### Core Schema (in WAV Metadata)

```json
{
  "cshot": {
    "spec_version": "1.0",
    "id": "cshot_sound_a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "created_at": "2026-05-15T14:30:00.000Z",
    
    "model": {
      "name": "cshot-generator-v1",
      "version": "1.2.3",
      "commit": "b7a8c9d0e1f2...",
      "architecture": "diffusion-transformer",
      "training_dataset": "cshot-dataset-v3",
      "training_cutoff": "2026-04-01",
      "parameters": 847_000_000
    },
    
    "generation": {
      "prompt": "punchy trap kick with short decay",
      "prompt_normalized": "punchy trap kick short decay",
      "prompt_embedding_hash": "emb_hash_...",
      "seed": 424242,
      "temperature": 0.8,
      "cfg_scale": 7.5,
      "inference_steps": 50,
      "sampler": "ddim",
      "duration_frames": 21467,
      "sample_rate": 44100,
      "channels": 1,
      "bit_depth": 32,
      "hardware": "NVIDIA RTX 4090",
      "inference_time_ms": 2340,
      "generator_version": "1.2.3"
    },
    
    "context": {
      "bpm": 140,
      "key": "C#",
      "scale": "minor",
      "time_signature": "4/4",
      "daw": "ableton_live_12",
      "project_path_hash": "proj_hash_",
      "project_name": "nightcity_track"
    },
    
    "provenance_chain": [
      {
        "type": "generation",
        "timestamp": "2026-05-15T14:30:00.000Z",
        "description": "Initial generation",
        "parent_id": null
      }
    ],
    
    "similarity": {
      "training_data": {
        "max_similarity": 0.32,
        "nearest_neighbor_id": null,
        "nearest_neighbor_similarity": 0.32
      },
      "copyrighted_works": {
        "max_similarity": 0.28,
        "nearest_match": null,
        "index_version": "cw-index-v7-20260501"
      },
      "user_library": {
        "max_similarity": 0.45,
        "nearest_match_id": "user_sound_xyz",
        "nearest_match_similarity": 0.45
      },
      "all_clear": true,
      "checked_at": "2026-05-15T14:30:03.000Z"
    },
    
    "licensing": {
      "tier": "pro",
      "commercial_safe": true,
      "confidence": 0.97,
      "recommended_attribution": null,
      "restrictions": []
    },
    
    "audio_signature": {
      "sha256": "a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890",
      "perceptual_hash": "phash_...",
      "waveform_hash": "whash_...",
      "duration_seconds": 0.487,
      "rms": 0.45,
      "peak": 0.92,
      "loudness_integrated": -12.3
    },
    
    "exports": [
      {
        "exported_at": "2026-05-15T15:00:00.000Z",
        "format": "wav",
        "sample_rate": 44100,
        "bit_depth": 24,
        "destination_hash": "dest_context_hash",
        "project": "nightcity_track",
        "user": "user_uuid"
      }
    ]
  }
}
```

### WAV Metadata Storage

```
iXML chunk (standard WAV metadata extension):

  <ixml>
    <cshot spec_version="1.0">
      <id>cshot_sound_a1b2c3d4...</id>
      <created_at>2026-05-15T14:30:00Z</created_at>
      <model>
        <name>cshot-generator-v1</name>
        <version>1.2.3</version>
      </model>
      <generation>
        <prompt>punchy trap kick...</prompt>
        <seed>424242</seed>
      </generation>
      <licensing>
        <commercial_safe>true</commercial_safe>
      </licensing>
      <signature>
        <sha256>a1b2c3d4...</sha256>
      </signature>
    </cshot>
  </ixml>

The full JSON is embedded in an iXML chunk.
For AIFF: embedded in the 'APPL' chunk.
For FLAC: embedded in Vorbis comments (cshot_* prefixed).
For MP3: embedded in ID3v2 PRIV frame.
```

### Storage Formats

| Field | Where Stored | Purpose |
|-------|-------------|---------|
| Full JSON metadata | SQLite database (library.db) | Fast querying, search |
| SHA-256 hash | Filename (content-addressed) | Integrity, dedup |
| Perceptual hash | SQLite (sounds table) | Similarity search |
| Audio data | /library/audio/{sha_prefix}/*.wav | Content storage |
| Full provenance chain | Per-sound JSON file (provenance.db) | Audit trail |
| User-facing summary | WAV iXML chunk | Portability |
| Model version pin | /library/models/versions.json | Reproducibility |
| Session context | Project .cshot file | Per-project context |

---

## 3. Cryptographic Hashes

### Hash Strategy

```
Multi-hash system for different verification needs:

SHA-256 (audio content):
  - Hash of raw Float32 interleaved audio samples
  - Used for: deduplication, integrity verification
  - Purpose: "Is this exact audio file the same as that one?"

Perceptual Hash (audio perception):
  - Hash of spectrogram features (like pHash for images)
  - Used for: similarity search, near-duplicate detection
  - Purpose: "Does this sound like that one?"

Waveform Hash (visual representation):
  - Downsampled, quantized waveform envelope
  - Used for: waveform thumbnail caching
  - Purpose: "Show me the waveform without decoding the full file"

Prompt Embedding Hash:
  - Hash of the CLAP-style text embedding
  - Used for: prompt deduplication, prompt similarity
  - Purpose: "Have I seen this prompt before?"

Model State Hash:
  - Hash of (model weights hash + seed + params)
  - Used for: reproducibility
  - Purpose: "Can I exactly recreate this sound?"
```

### Verification Flow

```python
def verify_sound_integrity(sound_path):
    """
    Verify that a sound's content matches its provenance metadata.
    Returns (verified: bool, issues: list[str]).
    """
    # Read WAV file
    audio, sr = load_wav(sound_path)
    metadata = read_ixml_chunk(sound_path)
    
    issues = []
    
    # 1. Check content hash
    actual_hash = sha256(audio.tobytes())
    if actual_hash != metadata['audio_signature']['sha256']:
        issues.append(f"SHA-256 mismatch: expected {metadata['sha256']}, got {actual_hash}")
    
    # 2. Check perceptual hash (tolerant to encoding differences)
    actual_phash = perceptual_hash(audio)
    expected_phash = metadata['audio_signature']['perceptual_hash']
    hamming_dist = hamming_distance(actual_phash, expected_phash)
    if hamming_dist > 5:
        issues.append(f"Perceptual hash mismatch: distance={hamming_dist}")
    
    # 3. Check duration
    actual_duration = len(audio) / sr
    expected_duration = metadata['audio_signature']['duration_seconds']
    if abs(actual_duration - expected_duration) > 0.01:
        issues.append(f"Duration mismatch: expected {expected_duration}s, got {actual_duration}s")
    
    # 4. Verify provenance chain integrity (cryptographic links)
    if not verify_provenance_chain(metadata):
        issues.append("Provenance chain has integrity gap")
    
    # 5. Check signature (if signed)
    if 'signature' in metadata:
        if not verify_cshot_signature(metadata):
            issues.append("cShot signature verification failed")
    
    return len(issues) == 0, issues
```

### Signing

```
cShot signs provenance metadata so users can verify:
  1. The sound was actually generated by cShot (not faked)
  2. The metadata hasn't been tampered with
  3. The generation timestamp is authentic

Signing approach:
  - Ed25519 keypair (offline master key, online signing subkey)
  - Sign: SHA-256(metadata_json + audio_sha256)
  - Verify: cShot's public key embedded in the application
  - Key rotation: versioned, with transition windows

What signing enables:
  - Legal admissibility (proven metadata origin)
  - Tamper detection (any modification invalidates signature)
  - Authenticity verification ("this was really made by cShot")
  - User trust (metadata can be independently verified)
```

---

## 4. Audit Logs

### Audit Log Structure

```
Every action that touches a sound is logged.

Audit log entry:
{
  "id": "audit_001",
  "timestamp": "2026-05-15T14:30:00.123Z",
  "action": "generate",
  "sound_id": "cshot_sound_a1b2c3d4...",
  "user_id": "user_xyz",
  "session_id": "session_abc",
  "details": {
    "prompt": "punchy trap kick",
    "seed": 424242,
    "inference_time_ms": 2340
  },
  "previous_state": null,
  "new_state": {
    "sha256": "a1b2c3d4...",
    "duration": 0.487
  },
  "signature": "ed25519_sig_..."
}
```

### Action Types

```
Action                  Description
─────────────────────────────────────────────────────
generate                Sound generated from prompt
regenerate              Same prompt, new seed
variant                 Generated variant from existing
mutate                  DSP mutation applied
edit_user               User modified sound (trim, fade, etc.)
edit_dsp                DSP processing applied
layer                   Two sounds layered
export                  Sound exported to file
export_daw              Sound exported to DAW via drag
favorite_add            Sound marked as favorite
favorite_remove         Sound removed from favorites
tag_add                 Tag added
tag_remove              Tag removed
tag_edit                Tag edited
similarity_check        Similarity check run
similarity_flag         Similarity flag triggered
license_check           License check performed
metadata_view           User viewed provenance card
provenance_export       User exported audit report
delete                  Sound deleted from library
```

### Log Storage

```
Local storage:
  - SQLite database: audit.db
  - Append-only (INSERT only, no UPDATE or DELETE)
  - Auto-vacuum disabled (historical integrity)
  - Rotated yearly (old logs archived)

Cloud storage (if enabled):
  - Append-only event stream
  - Immutable log (cryptographic chain of entries)
  - Replicated across regions
  - Queryable via API

Log retention:
  - Free tier: 90 days
  - Pro tier: 2 years
  - Enterprise: 7 years (or custom)

Log integrity:
  - Each entry is hashed with previous entry's hash (blockchain-lite)
  - Periodically signed anchor published (daily hash root)
  - Users can verify their logs haven't been altered
```

### Blockchain-Lite Chain

```python
class AuditChain:
    """Simple hash chain for audit log integrity."""
    
    def __init__(self, db):
        self.db = db
        
    def append(self, entry):
        # Get previous entry's hash
        prev = self.db.execute("SELECT hash FROM audit_log ORDER BY id DESC LIMIT 1").fetchone()
        prev_hash = prev[0] if prev else "0" * 64
        
        # Compute entry hash (includes previous)
        entry_data = json.dumps(entry, sort_keys=True)
        entry_hash = sha256(f"{prev_hash}{entry_data}".encode())
        entry['previous_hash'] = prev_hash
        entry['hash'] = entry_hash
        
        # Insert
        self.db.execute(
            "INSERT INTO audit_log (entry_json, hash, previous_hash, timestamp) VALUES (?, ?, ?, ?)",
            [json.dumps(entry), entry_hash, prev_hash, entry['timestamp']]
        )
        
    def verify_chain(self):
        """Verify the entire audit chain is intact."""
        rows = self.db.execute(
            "SELECT id, entry_json, hash, previous_hash FROM audit_log ORDER BY id"
        ).fetchall()
        
        prev_hash = "0" * 64
        for row in rows:
            entry_data = json.loads(row['entry_json'])
            expected_hash = sha256(f"{prev_hash}{json.dumps(entry_data, sort_keys=True)}".encode())
            if expected_hash != row['hash']:
                return False, f"Chain broken at entry {row['id']}"
            prev_hash = row['hash']
        
        return True, None
```

---

## 5. User-Visible Provenance Cards

### The Provenance Card

Every sound has a provenance card — a visual summary of its history.

```
┌─────────────────────────────────────────────────────┐
│  🔍 Sound Provenance                                │
│  ─────────────────────                               │
│                                                      │
│  🎯 Generation                                       │
│  ┌──────────────────────────────────────────────┐   │
│  │ Prompt: "punchy trap kick with short decay"  │   │
│  │ Seed: 424242 · Temperature: 0.8              │   │
│  │ Model: cShot-generator-v1 v1.2.3             │   │
│  │ Date: 2026-05-15 14:30 UTC                   │   │
│  │ ⏱ 2.34s inference on NVIDIA RTX 4090         │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  📋 Context                                          │
│  ┌──────────────────────────────────────────────┐   │
│  │ Project: nightcity_track                     │   │
│  │ BPM: 140 · Key: C# minor · 4/4              │   │
│  │ DAW: Ableton Live 12                        │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  🔗 Lineage                                         │
│  ┌──────────────────────────────────────────────┐   │
│  │  Root generation → No parents                │   │
│  │  └── Variant A (seed 424243) ↗               │   │
│  │  └── Variant B (seed 424244) → This sound    │   │
│  │  └── Variant C (seed 424245)                 │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ✅ Safety                                           │
│  ┌──────────────────────────────────────────────┐   │
│  │ ✅ Commercial-safe (confidence: 97%)         │   │
│  │ ✅ No similarity to copyrighted works        │   │
│  │ ⚠ Similar to your sound "old_kick.wav" (45%)│   │
│  │ Checked against 3 reference databases       │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  📦 Export History                                   │
│  ┌──────────────────────────────────────────────┐   │
│  │ 1. 2026-05-15 15:00 → nightcity_track (WAV) │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  [Verify Integrity] [Download Audit Report]          │
│  [Share Provenance] [Report Issue]                   │
└─────────────────────────────────────────────────────┘
```

### Provenance Card Access

```
Where to find provenance:
  - Sound grid: right-click → "Show Provenance"
  - Library: click info icon on any sound
  - After generation: auto-shown for 5 seconds
  - Finder (via macOS Quick Look plugin): spacebar → see provenance
  - Standalone app: sidebar panel when sound selected
  - Export dialog: summary shown before export
  - Web export: embedded in download metadata
```

---

## 6. Commercial-Use Confidence Scores

### Confidence Score Calculation

```python
def compute_commercial_confidence(sound):
    """
    Calculate a confidence score (0.0-1.0) that this sound 
    is safe for commercial use.
    """
    score = 1.0
    
    # Deductions
    if sound.similarity.training_max > 0.6:
        score -= 0.3  # Too close to training data
    
    if sound.similarity.copyrighted_max > 0.5:
        score -= 0.5  # Too close to copyrighted works
    
    if sound.similarity.copyrighted_max > 0.7:
        score -= 0.3  # Additional penalty for high similarity
    
    if not sound.licensing.commercial_allowed:
        score -= 0.5  # License restriction
    
    if sound.source == 'imported' and not sound.import_verified:
        score -= 0.4  # Unverified import
    
    if len(sound.provenance_chain) == 0:
        score -= 0.2  # No provenance (pre-cShot import)
    
    # Bonuses
    if sound.licensing.tier == 'enterprise':
        score += 0.1  # Enterprise indemnification
    
    if sound.similarity.all_clear:
        score += 0.1  # All reference checks passed
    
    if sound.audit_chain_verified:
        score += 0.05  # Audit chain intact
    
    return max(0.0, min(1.0, score))
```

### Score Display

```
Confidence shown as a badge on every sound:

  95-100%  🟢 Excellent    "Commercial-safe with high confidence"
  80-94%   🟡 Good         "Commercial-safe, review recommended"
  60-79%   🟠 Fair         "Probably safe, review strongly recommended"
  40-59%   🔴 Risky        "May have clearance issues"
  <40%     ⛔ Blocked      "Not safe for commercial use"

Clicking the badge shows the breakdown:
  "Score: 97% ⬆
   ✓ No training data similarity (94% → +25%)
   ✓ No copyrighted work match (99% → +35%)
   ⚠ User library similarity 45% (-5%)
   ✓ Pro license tier (→ +10%)
   ✓ Audit chain verified (→ +5%)
   ✓ All checks passed (→ +10%)
   Baseline: 50%"
```

### Library-Level Tools

```
Whole-library compliance scan:

  ┌────────────────────────────────────────────────────┐
  │  Library Compliance Report                         │
  ├────────────────────────────────────────────────────┤
  │  Total sounds: 2,345                               │
  │                                                     │
  │  🟢 Excellent (95-100%):  1,892 (80.7%)            │
  │  🟡 Good (80-94%):          312 (13.3%)            │
  │  🟠 Fair (60-79%):          112 (4.8%)             │
  │  🔴 Risky (40-59%):          24 (1.0%)             │
  │  ⛔ Blocked (<40%):            5 (0.2%)            │
  │                                                     │
  │  Recommended actions:                               │
  │  • Review 24 risky sounds before commercial release │
  │  • 5 blocked sounds can be regenerated             │
  │  • 112 fair sounds: bulk audit recommended         │
  │                                                     │
  │  [Export Report] [Bulk Regenerate Risky]            │
  │  [Filter by Confidence]                             │
  └────────────────────────────────────────────────────┘
```

### Batch Operations

```
Operations on multiple sounds:

  "Select all / Filter by confidence < 70%"
  
  Actions:
  - Bulk regenerate (with new seeds)
  - Bulk re-check (re-run similarity against latest indexes)
  - Bulk export with confidence report
  - Mark as "reviewed" (acknowledge risk)
  - Remove from library
  - Generate audit report for selected
  
  Audit report for legal:
  - CSV/PDF export
  - Per-sound provenance summary
  - Aggregate statistics
  - Confidence distribution
  - Any flagged sounds with details
  - Timestamped and signed
```

---

## 7. Provenance API

### Query Endpoints

```
GET /sound/{id}/provenance       → Full provenance record
GET /sound/{id}/provenance/summary → User-friendly summary
GET /sound/{id}/provenance/chain   → Lineage/provenance chain
GET /sound/{id}/provenance/verify  → Integrity verification
GET /library/compliance            → Library-level report
GET /export/{id}/audit-report      → Downloadable audit PDF
```

### Webhook Events

```
Events that trigger provenance-related webhooks:

  sound.generated     → { sound_id, prompt, seed }
  sound.exported      → { sound_id, format, destination }
  sound.similarity_flag → { sound_id, similarity, matched_work }
  sound.compliance_change → { sound_id, old_score, new_score }
  library.audit_complete → { report_id, summary_stats }
```

---

## Summary

| Component | Purpose | Implementation |
|-----------|---------|---------------|
| Metadata schema | Structured record of every aspect | JSON in WAV iXML + SQLite |
| Cryptographic hashes | Content integrity + identity | SHA-256, pHash, Ed25519 signing |
| Audit logs | Immutable action history | Append-only hash chain |
| Provenance cards | User-visible sound story | Interactive UI component |
| Confidence scores | Commercial-safety metric | Weighted algorithm per sound |
| Library tools | Bulk compliance management | Scans, filters, batch operations |
| API | Programmatic access | REST endpoints for provenance |

The chain of custody isn't just legal boilerplate — it's a product feature. Users should feel the weight of confidence when they see a 97% commercial-safe badge. That's the difference between "I think this is OK" and "I know this is OK."
