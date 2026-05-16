# Prompt 65 — Collaborative Sample Creation

Design collaboration features for cShot so users can share, remix, branch, and build sounds together.

---

## 1. Collaboration Model

### Philosophy

```
cShot collaboration is built around three principles:

  1. Sounds are starting points, not endpoints.
     Every generated sound can be remixed, branched, and evolved.
     There's no "final version" — only the current best iteration.

  2. Provenance is sacred.
     Every sound carries its full lineage: who generated it, what prompt,
     what remixes it inspired, what it was derived from. Attribution is
     automatic and immutable.

  3. Collaboration is async-first.
     Producers work alone most of the time. Collaboration means leaving
     your sound somewhere, someone picking it up and evolving it, and
     you seeing what they did. Real-time co-generation is a future
     feature, not the foundation.
```

### User Roles

| Role | Permissions | Example |
|------|------------|---------|
| Creator | Full control over own sounds | Originated a kick sound |
| Remixer | Can branch and modify shared sounds | Took someone else's kick, added punch |
| Pack Curator | Can organize sounds into shared packs | Created "Drum Essentials Vol. 1" pack |
| Viewer | Can see, preview, but not modify | Exploring a collaborator's sounds |
| Admin (pack) | Can manage pack membership, permissions | Owns a shared pack |

---

## 2. Sharing Model

### Share Levels

```
Private (default)
  ┌──────────────────────────────────────────────┐
  │ Sound exists only in your local library.     │
  │ Nobody else can see or access it.            │
  │ No cloud storage, no sync.                   │
  └──────────────────────────────────────────────┘

Shared with Link
  ┌──────────────────────────────────────────────┐
  │ Sound is uploaded to cShot cloud.            │
  │ Anyone with the link can preview and download.│
  │ No cShot account required to preview.        │
  │ Link can be:                                 │
  │   • Public (anyone with link)                │
  │   • Unlisted (only people with exact URL)    │
  │   • Password-protected                       │
  └──────────────────────────────────────────────┘

Shared with Collaborators
  ┌──────────────────────────────────────────────┐
  │ Sound is part of a shared pack or project.   │
  │ All collaborators can:                       │
  │   • Preview the sound                        │
  │   • Remix it (creates a branch)             │
  │   • Add it to their own library             │
  │   • Comment on it                           │
  │                                             │
  │ They can NOT:                                │
  │   • Delete the original                      │
  │   • Modify the original sound file           │
  │   • Change the provenance chain              │
  └──────────────────────────────────────────────┘

Public (Marketplace — Phase 5+)
  ┌──────────────────────────────────────────────┐
  │ Sound is published to the cShot marketplace. │
  │ Anyone can browse, preview, and license.     │
  │ Creator sets price and license terms.        │
  └──────────────────────────────────────────────┘
```

### Share Configuration

```rust
pub struct ShareConfig {
    pub visibility: Visibility,
    pub allow_remixing: bool,
    pub allow_export: bool,
    pub attribution_required: bool,
    pub license: LicenseType,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<u32>,
}

pub enum Visibility {
    Private,
    LinkOnly { password: Option<String> },
    Collaborators { pack_id: String },
    Public,
}

pub enum LicenseType {
    AllRightsReserved,
    CreativeCommonsZero,      // CC0 — no restrictions
    CreativeCommonsBy,        // CC-BY — attribution required
    CreativeCommonsBySa,      // CC-BY-SA — share-alike
    CreativeCommonsByNc,      // CC-BY-NC — non-commercial
    Custom(String),           // Custom license text
}
```

---

## 3. Remix System

### Branch Architecture

```
Original Sound (User A)
    │
    ├── Remix 1 (User B) — "Made it punchier"
    │       │
    │       ├── Remix 1a (User C) — "Added reverb tail"
    │       │
    │       └── Remix 1b (User B) — "Made two versions"
    │
    ├── Remix 2 (User A) — "Explored a different direction"
    │       │
    │       └── Remix 2a (User D) — "Turned it into a snare"
    │
    └── Remix 3 (User C) — "Layered with 808"
    
Each remix:
  - Links to parent sound (provenance)
  - Records the remix prompt: "Add punch and shorten tail"
  - Records the post-processing changes: transient +3dB, tail trim
  - Has its own unique ID and SoundScore
  - Can itself be remixed (any depth)
```

### Remix Operations

```rust
pub enum RemixOperation {
    /// Regenerate with modified prompt
    NewPrompt(String),
    
    /// Apply DSP transformations
    PostProcess(PostProcessParams),
    
    /// Sound morphing between two sounds
    Morph {
        sound_a_id: String,
        sound_b_id: String,
        morph_amount: f64,     // 0.0 = sound A, 1.0 = sound B
    },
    
    /// Layer multiple sounds
    Layer {
        sound_ids: Vec<String>,
        mix_levels: Vec<f64>,  // Gain per layer
    },
    
    /// Apply high-level control changes
    Controls(HighLevelControls),
}

pub struct RemixRecord {
    pub id: String,
    pub parent_id: String,
    pub creator_id: String,
    pub operation: RemixOperation,
    pub prompt: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sound_score: f64,
    pub audio_hash: String,
    pub lineage_path: Vec<String>,  // All ancestor IDs
}
```

---

## 4. Variation Trees

### Tree Structure

```rust
pub struct SoundTree {
    pub root_id: String,
    pub nodes: Vec<TreeNode>,
    pub edges: Vec<TreeEdge>,
    pub stats: TreeStats,
}

pub struct TreeNode {
    pub sound_id: String,
    pub creator_id: String,
    pub prompt: String,
    pub sound_score: f64,
    pub export_count: u32,
    pub remix_count: u32,
    pub comment_count: u32,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

pub struct TreeEdge {
    pub parent_id: String,
    pub child_id: String,
    pub operation: String,    // "remix", "morph", "layer", "control_adjust"
    pub similarity: f64,      // 0-1, how similar parent and child sound
}

pub struct TreeStats {
    pub depth: u32,
    pub total_variations: u32,
    pub total_contributors: u32,
    pub most_remixed_sound: String,
    pub average_score: f64,
}
```

### Tree Visualization (UX)

```
Sound Tree — "Punchy Kick Original" by @producer_a

                    ┌─────────────────────────────┐
                    │  Original Kick              │
                    │  Score: 72 · 4 exports      │
                    │  @producer_a · 2 weeks ago  │
                    └──────────┬──────────────────┘
                               │
            ┌──────────────────┼──────────────────┐
            │                  │                  │
            ▼                  ▼                  ▼
  ┌──────────────────┐  ┌──────────────┐  ┌──────────────┐
  │ "More Punch"     │  │ "808 Sub"   │  │ "Brighter"  │
  │ Score: 78        │  │ Score: 85   │  │ Score: 65   │
  │ @producer_b      │  │ @producer_a │  │ @producer_c │
  │ 5 exports        │  │ 12 exports  │  │ 1 export    │
  └────────┬─────────┘  └──────┬───────┘  └─────────────┘
           │                   │
    ┌──────┴──────┐           │
    ▼             ▼           ▼
  ┌─────────┐  ┌─────────┐  ┌──────────────────┐
  │ "Trap   │  │ "Lo-fi  │  │ "Sub + Layer"    │
  │ Version"│  │ Version"│  │ Score: 88        │
  │ 82      │  │ 71      │  │ @producer_d      │
  │ @pro_c  │  │ @pro_d  │  │ 8 exports        │
  └─────────┘  └─────────┘  └──────────────────┘

  [Collapse] [Show All] [Export Tree as Bundle] [View Full Screen]
```

### Remix UX Flow

```
1. User opens a sound in the detail panel
2. Sees "Remix" button (with count: "12 remixes")
3. Clicks "Remix"
4. Options:
   ┌─────────────────────────────────────────────┐
   │   How do you want to remix?                  │
   │                                              │
   │   [ Change the prompt                        │
   │     "punchy trap kick with more sub" ]       │
   │                                              │
   │   [ Adjust controls                          │
   │     Punch: ◄───●────────►                    │
   │     Body:  ◄───────●────►                    │
   │     Sub:   ◄─────────●──► ]                  │
   │                                              │
   │   [ Add post-processing                      │
   │     Reverb: ◄────●───────►                   │
   │     Saturation: ◄──●───────► ]               │
   │                                              │
   │   [ Combine with another sound               │
   │     "Drag a sound here to layer" ]           │
   │                                              │
   │   [ Generate new direction                   │
   │     "Keep the vibe, make it completely       │
   │      different: dark ambient version" ]     │
   │                                              │
   │   Notes (optional): "Made this for the       │
   │   bridge section of my track"               │
   │                                              │
   │   [Create Remix] [Cancel]                    │
   └─────────────────────────────────────────────┘
5. Remix is generated, linked to parent
6. User can publish to pack, share link, or keep private
```

---

## 5. Shared Packs

### Pack Structure

```rust
pub struct SharedPack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cover_art_url: Option<String>,
    pub creator_id: String,
    pub collaborators: Vec<PackMember>,
    pub sounds: Vec<PackSound>,
    pub tags: Vec<String>,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub visibility: Visibility,
}

pub struct PackMember {
    pub user_id: String,
    pub role: PackRole,
    pub joined_at: DateTime<Utc>,
    pub contribution_count: u32,
}

pub enum PackRole {
    Owner,       // Full control: delete, manage members, change settings
    Editor,      // Add/remove sounds, edit pack metadata
    Contributor, // Add own sounds, can't remove others'
    Viewer,      // Read-only access
}

pub struct PackSound {
    pub sound_id: String,
    pub added_by: String,
    pub category: Option<String>,  // "kicks", "snares", "fx"
    pub order: u32,
    pub notes: Option<String>,
    pub added_at: DateTime<Utc>,
}
```

### Pack Creation Flow

```
1. User clicks "Create Pack"
2. Fill in:
   ┌─────────────────────────────────────────────┐
   │   New Pack                                    │
   │                                               │
   │   Name: [Trap Essentials Vol. 1          ]   │
   │                                               │
   │   Description: [A collection of my best      │
   │   trap kicks, snares, and 808s. Free to      │
   │   remix and use in your tracks.         ]   │
   │                                               │
   │   Visibility: [◎ Shared with collaborators   │
   │                 ○ Anyone with link           │
   │                 ○ Public                     │
   │                                               │
   │   Collaboration: [✓ Allow remixing           │
   │                   ✓ Require attribution      │
   │                   □ Require approval to join] │
   │                                               │
   │   Tags: [trap] [kicks] [essentials] [add]   │
   │                                               │
   │   [Create Pack] [Cancel]                      │
   └─────────────────────────────────────────────┘
3. Drag sounds from library into pack
4. Invite collaborators via email or cShot username
5. Publish
```

---

## 6. Comment System

### Sound Comments

```rust
pub struct SoundComment {
    pub id: String,
    pub sound_id: String,
    pub user_id: String,
    pub text: String,
    pub timestamp_ms: Option<u32>,  // Position in the audio
    pub reply_to: Option<String>,
    pub reactions: Vec<Reaction>,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

pub struct Reaction {
    pub emoji: String,
    pub user_ids: Vec<String>,
}
```

### Timestamped Comments

```
Waveform with Comments:

  ┌─────────────────────────────────────────────────┐
  │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━     │
  │        ● ●  ●                                    │
  │      ●     ●    ●                                │
  │    ●              ●    ●                         │
  │  ●                  ●●     ●●                   │
  │                                                   │
  │  0ms          200ms        400ms        600ms    │
  │                                                   │
  │  💬 @producer_b at 120ms:                        │
  │  "The attack is perfect here, but the tail       │
  │   could use more body"                           │
  │                                                   │
  │  💬 @producer_c at 350ms:                        │
  │  "I layered this with an 808 and it sits         │
  │   perfectly in the mix"                         │
  │                                                   │
  │  💬 @producer_a (reply):                         │
  │  "Can you share that 808?"                      │
  │                                                   │
  │  [Add comment at cursor position...] [Send]     │
  └─────────────────────────────────────────────────┘
```

---

## 7. Version Comparison

### A/B Comparison Tool

```rust
pub struct SoundComparison {
    pub sound_a: SoundVersion,
    pub sound_b: SoundVersion,
    pub metrics: ComparisonMetrics,
    pub waveform_diff: Vec<f64>,  // Point-by-point difference
}

pub struct SoundVersion {
    pub id: String,
    pub creator: String,
    pub prompt: String,
    pub sound_score: f64,
    pub duration_ms: f64,
    pub spectral_centroid: f64,
    pub crest_factor: f64,
    pub rms: f64,
}

pub struct ComparisonMetrics {
    pub spectral_difference: f64,
    pub dynamic_difference: f64,
    pub duration_difference_ms: f64,
    pub loudness_difference_db: f64,
    pub similarity_score: f64,        // 0 (completely different) to 1 (same)
    pub preferred: Option<String>,    // User's pick
}
```

### Comparison UX

```
┌────────────────────────────────────────────────────────────┐
│  Sound Comparison                                          │
│                                                            │
│  Version A                    Version B                    │
│  @producer_a · Remix 0       @producer_b · Remix 1        │
│  ┌────────────────────┐     ┌────────────────────┐        │
│  │  Waveform A        │     │  Waveform B        │        │
│  │  ━━━━━━━━━━        │     │  ━━━━━━━━━━━━━     │        │
│  └────────────────────┘     └────────────────────┘        │
│                                                            │
│  Score: 72                    Score: 85                    │
│  Duration: 412ms              Duration: 380ms              │
│  Crest: 9.2                   Crest: 11.4                  │
│  Centroid: 2.1kHz             Centroid: 2.8kHz             │
│  Loudness: -11.2dB            Loudness: -9.8dB             │
│                                                            │
│  What changed:                                             │
│  ┌────────────────────────────────────────────────────┐   │
│  │ ✓ Punch increased (+18%)                          │   │
│  │ ✓ Brightness increased (+33%)                     │   │
│  │ ✓ Tail shortened (-32ms)                          │   │
│  │ ✓ SoundScore improved (+13 points)                │   │
│  └────────────────────────────────────────────────────┘   │
│                                                            │
│  [Play A] [Play B] [A/B Toggle] [Play Both]              │
│  [Use Version B] [Revert to A] [Save Both to Pack]       │
└────────────────────────────────────────────────────────────┘
```

---

## 8. Provenance Tracking

### Immutable Chain

Every sound carries its complete provenance chain. This is the foundation of trust in the collaboration system.

```rust
pub struct ProvenanceChain {
    pub sound_id: String,
    pub entries: Vec<ProvenanceEntry>,
    pub total_contributors: u32,
}

pub struct ProvenanceEntry {
    pub entry_type: ProvenanceEntryType,
    pub user_id: String,
    pub user_display_name: String,
    pub timestamp: DateTime<Utc>,
    pub details: String,
    pub signature: Option<String>,     // Future: cryptographic signature
}

pub enum ProvenanceEntryType {
    Generated {
        prompt: String,
        seed: u32,
        model_version: String,
    },
    Remixed {
        operation: String,
        parent_sound_id: String,
    },
    Exported {
        export_count: u32,
    },
    AddedToPack {
        pack_id: String,
        pack_name: String,
    },
    Licensed {
        license_type: String,
        licensee: String,
    },
}
```

### Provenance Display

```
Provenance — "Punchy Kick V3"

  Original generation
  ┌─────────────────────────────────────────────────────┐
  │ @producer_a · Feb 12, 2025                          │
  │ Prompt: "punchy trap kick 140bpm"                   │
  │ Seed: 847291                                        │
  │ Model: ElevenLabs SFX v2.1                          │
  └─────────────────────────────────────────────────────┘
       │
       ▼
  Remix 1 — "Added more sub"
  ┌─────────────────────────────────────────────────────┐
  │ @producer_b · Feb 14, 2025                          │
  │ Operation: Post-process (sub_boost: +3dB)          │
  │ SoundScore improved: 72 → 78                       │
  │ Note: "This works better in my track"              │
  └─────────────────────────────────────────────────────┘
       │
       ▼
  Remix 2 — "Trap version with punch"
  ┌─────────────────────────────────────────────────────┐
  │ @producer_c · Feb 16, 2025                          │
  │ Operation: New prompt + controls                   │
  │ Prompt: "same but punchier and brighter"            │
  │ Controls: punch +20%, body -10%, air +15%           │
  │ SoundScore: 85                                      │
  │ Note: "Perfect for the drop in my track"            │
  └─────────────────────────────────────────────────────┘
       │
       ▼
  Added to Pack — "Trap Essentials Vol. 1"
  ┌─────────────────────────────────────────────────────┐
  │ @producer_a · Feb 18, 2025                          │
  │ Pack has 24 sounds · 8 contributors                │
  └─────────────────────────────────────────────────────┘

  [Verify Chain] [Export as JSON] [Embed in WAV metadata]
```

---

## 9. Collaboration Bundle Export

### Bundle Format

A collaboration bundle packages a sound tree + all metadata for sharing outside cShot.

```rust
pub struct CollaborationBundle {
    pub manifest: BundleManifest,
    pub sounds: Vec<BundleSound>,
    pub provenance: ProvenanceChain,
    pub version_comparisons: Vec<SoundComparison>,
    pub comments: Vec<SoundComment>,
    pub pack_metadata: Option<PackMetadata>,
}

pub struct BundleManifest {
    pub bundle_id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub creator: String,
    pub contributors: Vec<String>,
    pub sound_count: u32,
    pub format_version: String,    // "cshot-bundle-v1"
    pub file_size_bytes: u64,
}

pub struct BundleSound {
    pub id: String,
    pub creator: String,
    pub prompt: String,
    pub audio_data_base64: String,  // WAV encoded as base64
    pub duration_ms: f64,
    pub sound_score: f64,
    pub parent_id: Option<String>,
    pub operation: Option<String>,
}

pub struct PackMetadata {
    pub pack_name: String,
    pub pack_description: String,
    pub license: String,
    pub recommended_genres: Vec<String>,
    pub bpm_range: (u32, u32),
}
```

### Export Flow

```
1. User selects sounds in tree view
2. Clicks "Export Bundle"
3. Configure:
   ┌─────────────────────────────────────────────┐
   │   Export Collaboration Bundle                │
   │                                              │
   │   Sounds: 8 selected (from 12 in tree)      │
   │   Contributors: 3 (producer_a, b, c)        │
   │                                              │
   │   Bundle Name: [Trap Kick Collab]           │
   │                                              │
   │   Include:                                   │
   │   [✓] Audio files (WAV 44.1kHz/24-bit)     │
   │   [✓] Provenance chain                      │
   │   [✓] Comments                              │
   │   [✓] Version comparisons                   │
   │   [ ] License file                          │
   │                                              │
   │   Estimated size: 4.2 MB                    │
   │                                              │
   │   [Export] [Cancel]                          │
   └─────────────────────────────────────────────┘
4. Downloads as .cshotbundle file (ZIP with manifest.json + WAVs)
5. Recipient can import into cShot or extract as individual WAVs
```

---

## 10. Conflict Handling

### Conflict Scenarios

| Scenario | Detection | Resolution |
|----------|-----------|------------|
| Two users remix the same sound simultaneously | Both create branches from same parent | Both are valid branches. No conflict. Tree shows both. |
| User deletes a sound that has remixes | Orphaned children | Warn: "This sound has 3 remixes. Delete anyway?" If confirmed, children link to grandparent. |
| Pack member removed, their sounds remain | Sound orphaned from contributor | Sounds remain in pack, but show "unclaimed" attribution. New owner can claim. |
| Two packs with the same name | Name collision on publish | Append owner name: "Trap Essentials by @user_a" vs "@user_b" |
| Imported bundle has same sound IDs as local | ID collision | Prefix with import timestamp: "sound_id_imported_20250218" |
| User exports & re-imports their own sound | Duplicate detection | Check SHA-256 hash, link to existing record, don't duplicate |

### Merge Strategy

```rust
pub enum MergeStrategy {
    /// Keep both versions as separate branches (default)
    KeepBoth,
    
    /// Replace local version with imported version
    Replace,
    
    /// Keep local version, discard imported
    KeepLocal,
    
    /// Create a new merged version based on both
    /// (difficult — requires human decision)
    CreateMerge {
        source_ids: Vec<String>,
        merge_prompt: String,
    },
}
```

---

## 11. Collaboration UX Flow Summary

```
                        ┌─────────────────┐
                        │ Generate Sound   │
                        └────────┬────────┘
                                 │
                    ┌────────────┴────────────┐
                    │                         │
                    ▼                         ▼
           ┌──────────────┐          ┌──────────────┐
           │ Keep Private │          │ Share         │
           └──────────────┘          └──────┬───────┘
                                            │
                                  ┌─────────┴─────────┐
                                  │                   │
                                  ▼                   ▼
                         ┌──────────────┐   ┌──────────────────┐
                         │ Share Link   │   │ Add to Pack      │
                         └──────┬───────┘   └────────┬─────────┘
                                │                    │
                                ▼                    ▼
                         ┌──────────────┐   ┌──────────────────┐
                         │ Anyone can   │   │ Collaborators    │
                         │ preview      │   │ can remix        │
                         └──────────────┘   └────────┬─────────┘
                                                     │
                                                     ▼
                                            ┌──────────────────┐
                                            │ Remix & Branch   │
                                            │ Tree grows       │
                                            └────────┬─────────┘
                                                     │
                                                     ▼
                                            ┌──────────────────┐
                                            │ Comment &        │
                                            │ Compare versions │
                                            └────────┬─────────┘
                                                     │
                                                     ▼
                                            ┌──────────────────┐
                                            │ Export Bundle     │
                                            │ Share with world  │
                                            └──────────────────┘
```

---

## 12. Permission Model Summary

| Action | Own Sound | Shared Pack | Public |
|--------|-----------|-------------|--------|
| Preview | ✓ | ✓ (if member) | ✓ |
| Export | ✓ | ✓ | ✓ |
| Edit tags | ✓ | Editor+ | ✗ |
| Delete | ✓ | Owner only | ✗ |
| Remix | ✓ | ✓ | ✓ |
| Add to pack | ✓ | Contributor+ | ✗ |
| Remove from pack | ✓ | Editor+ | ✗ |
| Comment | ✓ | ✓ | ✓ |
| View provenance | ✓ | ✓ | ✓ |
| Export bundle | ✓ | Editor+ | ✗ |
| Change license | ✓ | Owner only | ✗ |
| Delete pack | ✗ | Owner only | ✗ |

---

## 13. Collaboration Database Schema

```sql
-- Core collaboration tables

CREATE TABLE packs (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id),
    visibility TEXT NOT NULL DEFAULT 'private',
    license TEXT NOT NULL DEFAULT 'cc-by',
    tags TEXT[],                           -- JSON array
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE pack_members (
    pack_id UUID NOT NULL REFERENCES packs(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    role TEXT NOT NULL DEFAULT 'contributor', -- 'owner', 'editor', 'contributor', 'viewer'
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (pack_id, user_id)
);

CREATE TABLE pack_sounds (
    id UUID PRIMARY KEY,
    pack_id UUID NOT NULL REFERENCES packs(id) ON DELETE CASCADE,
    sound_id UUID NOT NULL REFERENCES sounds(id),
    added_by UUID NOT NULL REFERENCES users(id),
    category TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE remixes (
    id UUID PRIMARY KEY,
    parent_sound_id UUID NOT NULL REFERENCES sounds(id),
    child_sound_id UUID NOT NULL REFERENCES sounds(id),
    creator_id UUID NOT NULL REFERENCES users(id),
    operation TEXT NOT NULL,     -- 'new_prompt', 'post_process', 'morph', 'layer', 'controls'
    operation_params TEXT,       -- JSON
    similarity_score REAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(parent_sound_id, child_sound_id)
);

CREATE INDEX idx_remixes_parent ON remixes(parent_sound_id);
CREATE INDEX idx_remixes_creator ON remixes(creator_id);

CREATE TABLE sound_comments (
    id UUID PRIMARY KEY,
    sound_id UUID NOT NULL REFERENCES sounds(id),
    user_id UUID NOT NULL REFERENCES users(id),
    text TEXT NOT NULL,
    timestamp_ms INTEGER,           -- Position in audio
    reply_to UUID REFERENCES sound_comments(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMPTZ
);

CREATE TABLE sound_reactions (
    comment_id UUID NOT NULL REFERENCES sound_comments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    emoji TEXT NOT NULL,
    PRIMARY KEY (comment_id, user_id)
);
```

---

## 14. Summary

```
cShot Collaboration — Key Design Decisions:

  1. Async-first: Collaboration is about leaving sounds for others
     to evolve, not real-time co-generation.

  2. Trees, not lists: Every remix creates a branch. The sound tree
     is the primary organizational metaphor.

  3. Automatic attribution: Provenance is immutable and automatic.
     No one can claim credit for someone else's work.

  4. Permission model is role-based: Owner > Editor > Contributor > Viewer.
     Simple, clear, fits the typical producer workflow.

  5. Comments are timestamped: Position comments on the waveform
     so feedback is precise and contextual.

  6. Version comparison built-in: A/B any two sounds in a tree.
     Metrics show exactly what changed between versions.

  7. Bundle export: Pack a sound tree + metadata into a portable file.
     Works without cShot (extractable as WAVs).

  8. Phase timeline: Sharing links (Phase 1), Packs (Phase 2),
     Marketplace (Phase 5+). Start simple, grow with the community.
```
