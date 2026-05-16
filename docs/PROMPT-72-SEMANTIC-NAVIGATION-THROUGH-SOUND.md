# Prompt 72 — Semantic Navigation Through Sound

Design a system where users navigate sound semantically instead of browsing folders. "More metallic." "Less muddy." "Harder transient." "Warmer but cleaner." "Like this but cinematic." "More expensive sounding." "Closer to old Lex Luger drums."

---

## 1. The Navigation Problem

### Why Folders Are Broken

```
Current navigation:
  Folder A: "Kicks"
    └── Subfolder: "Trap Kicks"
         └── Subfolder: "Punchy Trap Kicks"
              └── 200 WAV files with names like "Kick_01_trap_punchy.wav"

  Problem: The folder tree is a TAXONOMY, not a NAVIGATION.
  
  A taxonomy is static, rigid, pre-computed.
  Navigation is dynamic, personal, iterative.
  
  "I need a kick that's punchy but not too bright, with a short decay..."
  
  With folders: this is 30 minutes of browsing.
  With semantic nav: this is 3 seconds of adjusting sliders.

The fundamental insight:
  Producers don't think in folders. They think in DIMENSIONS.
  "Punchy." "Bright." "Deep." "Warm." "Aggressive."
  These are not tags — they are axes in a perceptual space.
  Navigation should operate along these axes directly.
```

### The Semantic Gap

```
What user says:                   What folder structure assumes:
──────────────────────────────────────────────────────────────────
"punchy kick"                     Type: kick, Style: punchy
"more metallic"                   ✗ Not expressible in folders
"less muddy"                      ✗ Not expressible in folders
"like this but harder"            ✗ Not expressible (needs reference)
"cinematic version of this kick"  ✗ Not expressible (needs reference)
"closer to Lex Luger drums"       ✗ Not expressible (needs cultural ref)
"warmer but still punchy"         ✗ Compound constraint
"make it sound expensive"         ✗ Abstract concept

Current tools solve this by:
  - More tags (but tag space is finite)
  - Better search (but search is one-shot, not iterative)
  - Curated packs (but someone else's curation)
  
None of these solve the fundamental problem:
  Sound is multidimensional. Navigation should be too.
```

---

## 2. Semantic Control Architecture

### The Core Loop

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  User Intent                                                        │
│  "I want this but punchier and darker"                              │
│         │                                                           │
│         ▼                                                           │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Semantic Parser                                              │  │
│  │  • Parse natural language into axis modifications             │  │
│  │  • "punchier" → transient_axis: +0.3                         │  │
│  │  • "darker" → spectral_axis: -0.2                            │  │
│  │  • "but" → preserve original character on unaffected axes    │  │
│  └──────────────────────┬───────────────────────────────────────┘  │
│                         │                                          │
│                         ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Embedding Navigator                                         │  │
│  │  • Current sound: embedding_ref                              │  │
│  │  • Apply semantic deltas: embedding_target =                  │  │
│  │    embedding_ref + Σ(axis_direction * axis_magnitude)        │  │
│  │  • Find nearest neighbors: FAISS search                      │  │
│  │  • Or generate: decode embedding_target → new sound          │  │
│  └──────────────────────┬───────────────────────────────────────┘  │
│                         │                                          │
│                         ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Result Presentation                                         │  │
│  │  • Show top 5 results (search) or generated sound            │  │
│  │  • Show navigation sliders for each modified axis            │  │
│  │  • "How much punchier?" — slider adjusts magnitude           │  │
│  │  • User can iterate: "more" or "less" on any axis            │  │
│  └──────────────────────┬───────────────────────────────────────┘  │
│                         │                                          │
│                         ▼                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Feedback Loop                                               │  │
│  │  User selects result → implicit label: "this direction was   │  │
│  │  correct" → axis directions refined per user                 │  │
│  │  Over time: personal axis calibration ("aggressive" means    │  │
│  │  different things to different users)                        │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Architecture Layers

```
┌─────────────────────────────────────────────────────────────────────┐
│  LAYER 1: Natural Language Interface                               │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Input: "like this but punchier and darker"                   │  │
│  │                                                               │  │
│  │  Parser Pipeline:                                             │  │
│  │    1. Intent classification: MODIFY                           │  │
│  │    2. Reference extraction: "this" → current sound           │  │
│  │    3. Axis extraction:                                        │  │
│  │       - "punchier" → transient_axis: +0.3                    │  │
│  │       - "darker" → spectral_axis: -0.2                      │  │
│  │    4. Constraint parsing: "but" → preserve unaffected axes  │  │
│  │    5. Magnitude calibration:                                  │  │
│  │       - Default: 0.3 step per "er/more" modifier            │  │
│  │       - User-historical: scale by personal calibration      │  │
│  │                                                               │  │
│  │  Supported patterns:                                          │  │
│  │    "X but Y"                   → modify X, add Y constraint  │  │
│  │    "X without Y"               → preserve X, remove Y         │  │
│  │    "like X but Yer"           → reference X, add Y           │  │
│  │    "more X less Y"            → increase X, decrease Y       │  │
│  │    "X that sounds like Y"     → find X with Y's character    │  │
│  │    "make it sound more X"     → increase X                    │  │
│  │    "less X but keep the Y"    → decrease X, preserve Y        │  │
│  └───────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 2: Axis Manipulation Engine                                 │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Each semantic modifier maps to an axis direction vector:     │  │
│  │                                                               │  │
│  │  ┌─────────────────────┬──────────────────┬─────────────────┐ │  │
│  │  │ Natural Language    │ Embedding Axis   │ Direction       │ │  │
│  │  ├─────────────────────┼──────────────────┼─────────────────┤ │  │
│  │  │ more punchy         │ transient         │ +transient_vec  │ │  │
│  │  │ less punchy         │ transient         │ -transient_vec  │ │  │
│  │  │ harder              │ transient.attack  │ +attack_vec     │ │  │
│  │  │ brighter            │ spectral.balance  │ +bright_vec     │ │  │
│  │  │ darker              │ spectral.balance  │ -bright_vec     │ │  │
│  │  │ warmer              │ spectral.warmth   │ +warmth_vec     │ │  │
│  │  │ metallic            │ timbre.material   │ +metallic_vec   │ │  │
│  │  │ wooden              │ timbre.material   │ +wooden_vec     │ │  │
│  │  │ muddy               │ spectral.clarity  │ -clarity_vec    │ │  │
│  │  │ clean               │ spectral.clarity  │ +clarity_vec    │ │  │
│  │  │ expensive           │ production.cost   │ +expensive_vec  │ │  │
│  │  │ cheap               │ production.cost   │ -expensive_vec  │ │  │
│  │  │ aggressive          │ energy            │ +energy_vec     │ │  │
│  │  │ gentle              │ energy            │ -energy_vec     │ │  │
│  │  │ cinematic           │ genre.cinematic   │ +cinematic_vec  │ │  │
│  │  │ vintage             │ period.vintage    │ +vintage_vec    │ │  │
│  │  │ modern              │ period.modern     │ +modern_vec     │ │  │
│  │  │ textured            │ texture.grit      │ +grit_vec       │ │  │
│  │  │ smooth              │ texture.grit      │ -grit_vec       │ │  │
│  │  │ roomy               │ mix.space         │ +room_vec       │ │  │
│  │  │ dry                 │ mix.space         │ -room_vec       │ │  │
│  │  │ compressed          │ production.dynamic│ +compressed_vec │ │  │
│  │  │ dynamic             │ production.dynamic│ -compressed_vec │ │  │
│  │  └─────────────────────┴──────────────────┴─────────────────┘ │  │
│  │                                                               │  │
│  │  Compound operations:                                          │  │
│  │    "warmer but cleaner" ->                                      │  │
│  │      +0.3 * warmth_vec + 0.2 * clarity_vec                     │  │
│  │    → (These axes might interact — MoE handles this)            │  │
│  └───────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 3: Embedding Operations                                     │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Core operation:                                               │  │
│  │                                                               │  │
│  │    embed_target = embed_ref + Σ(α_i × d_i)                    │  │
│  │                                                               │  │
│  │    where:                                                      │  │
│  │      embed_ref = UShOt embedding of current sound             │  │
│  │      α_i = magnitude of modifier i (0.0 to 1.0)              │  │
│  │      d_i = unit direction vector for modifier i               │  │
│  │                                                               │  │
│  │  Clone mode (preserve original character):                    │  │
│  │    Axis-specific modifier: only modify target axis            │  │
│  │    Leave all other axes unchanged                              │  │
│  │    → Achieved by zeroing non-target dimensions               │  │
│  │                                                               │  │
│  │  Interpolation mode (morph between two sounds):               │  │
│  │    embed_morph = (1-t) × embed_a + t × embed_b               │  │
│  │    then apply semantic deltas on top                          │  │
│  │                                                               │  │
│  │  Reference mode (match reference character):                  │  │
│  │    embed_target = embed_ref + (embed_reference - embed_other) │  │
│  │    "make this sound like that kick"                          │  │
│  │    → vector from other to reference applied to source         │  │
│  └───────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 4: Navigation UI                                            │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  (See Section 4 for full UI design)                           │  │
│  └───────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Latent-Axis Discovery

### Discovering Axes from Data

```
Some semantic axes are obvious (punchy, bright).
Others are latent — discoverable only through data.

Latent axis discovery pipeline:

  1. PCA on embedding space:
     Top principal components ≈ dominant perceptual axes.
     PC1 often = "timbre brightness" (explains 22% variance)
     PC2 often = "transient sharpness" (explains 15% variance)
     PC3 often = "energy/density" (explains 11% variance)

  2. Interpret PCs through correlation with labels:
     PC1 correlates with spectral centroid (r=0.89)
     PC2 correlates with attack time (r=0.81)
     PC3 correlates with RMS energy (r=0.78)

  3. Discover CULTURAL axes (not derivable from acoustics):
     "expensive" — learned from producer preference data
     "vintage" — learned from decade-labeled sounds
     "Lex Luger" — learned from similarity judgments
     "trap" — learned from genre-labeled data

  4. Discover PERSONAL axes (per user):
     "my style" — learned from user's export history
     "Rachel's snares" — learned from user's favorites
     "dark vibe" — learned from user's session context

  Latent axis = meaningful direction in embedding space that:
    a) Explains significant variance in user behavior
    b) Can be labeled with a semantic descriptor
    c) Enables useful navigation operations
```

### Axis Discovery Methods

```
Method 1 — Supervised Axis Learning:
  Given: labeled pairs (sound_a, sound_b, axis_label)
  Task: find direction d such that:
    (embed_a - embed_b) · d ≈ 1 if axis differs,
    0 if axis is same
  Result: interpretable axis direction

Method 2 — Unsupervised Axis Discovery:
  Given: large collection of one-shots
  Method: 
    a) Apply β-VAE with 32 latent dimensions
    b) Identify which dimensions are most disentangled
    c) Label through probe: "this dimension controls brightness"
    d) Combine correlated disentangled dims into axes
  Result: latent axes with partial disentanglement

Method 3 — Cultural Axis Mining:
  Given: user interaction data (favorites, exports, prompts)
  Method:
    a) Find sounds frequently co-exported in sessions
    b) Compute vector between embedding clusters
    c) Label through prompt analysis: "users prompt 'grimey' for these"
    d) Validate: "do users who like A also like B?"
  Result: culturally grounded axes

Method 4 — Interactive Axis Discovery:
  Given: real-time user feedback
  Method:
    a) User adjusts sliders, marks results as "better"/"worse"
    b) Learn direction that maximizes "better" predictions
    c) Ask user to name the axis: "What would you call this?"
    d) Add to personal axis library
  Result: personalized experiential axes
  
Axis quality metrics:
  - Interpretability: can humans describe the axis? (label agreement > 0.7)
  - Consistency: does moving along d produce expected changes? (80%+)
  - Independence: is d orthogonal to existing axes? (cosine < 0.3)
  - Utility: do users navigate with this axis? (usage frequency)
```

### Axis Library

```
Built-in axes (shipped with cShot):
  ┌─────────────────────┬──────────┬─────────────────────────────┐
  │ Axis                │ Type     │ Discovery Method            │
  ├─────────────────────┼──────────┼─────────────────────────────┤
  │ Brightness          │ Acoustic │ Signal processing (centroid) │
  │ Punch               │ Acoustic │ Transient analysis          │
  │ Warmth              │ Acoustic │ Spectral modeling           │
  │ Weight              │ Acoustic │ Low-frequency energy        │
  │ Air                 │ Acoustic │ High-frequency content      │
  │ Grit                │ Acoustic │ Noise/harmonic ratio        │
  │ Room/space          │ Acoustic │ Decay/reverb analysis       │
  │ Realism             │ Acoustic │ Naturalness metrics          │
  │ Energy              │ Acoustic │ RMS + transient density     │
  │ Density             │ Acoustic │ Spectral flux               │
  ├─────────────────────┼──────────┼─────────────────────────────┤
  │ Genre affinity      │ Cultural │ Labeled data                │
  │ Era (vintage/modern)│ Cultural │ Decade-labeled data         │
  │ Production cost     │ Cultural │ Expert ratings              │
  │ Mix readiness       │ Cultural │ Producer feedback           │
  │ Emotional valence   │ Cultural │ Listener ratings            │
  └─────────────────────┴──────────┴─────────────────────────────┘

Discovered axes (emerge from data):
  ┌─────────────────────┬──────────┬─────────────────────────────┐
  │ Axis                │ Source   │ Discovery Trigger           │
  ├─────────────────────┼──────────┼─────────────────────────────┤
  │ "That Metro sound"  │ Users    │ 500+ users ref "Metro"     │
  │ "Old Kanye kicks"   │ Users    │ 300+ users ref "old Kanye" │
  │ "Pharrell snare"    │ Users    │ 200+ users ref "Pharrell"  │
  │ "Bedroom pop"       │ Users    │ 150+ users in genre tags   │
  │ "Soundcloud rap"    │ Users    │ 200+ users in prompt data  │
  │ "TikTok drum"       │ Users    │ 100+ users in prompt data  │
  │ "Aggressive but clean"│ Users  │ Shared prompt pattern      │
  │ "Lofi warmth"       │ Users    │ Co-occurrence cluster      │
  └─────────────────────┴──────────┴─────────────────────────────┘

Personal axes (emerge per user):
  ┌─────────────────────┬──────────┬─────────────────────────────┐
  │ Axis                │ Source   │ Personalization             │
  ├─────────────────────┼──────────┼─────────────────────────────┤
  │ "Your signature"    │ User     │ Most exported sound vector  │
  │ "Your style"        │ User     │ Cluster of user favorites   │
  │ "Current mood"      │ User     │ Recent session patterns     │
  │ "Project sound"     │ User     │ Current track reference     │
  └─────────────────────┴──────────┴─────────────────────────────┘
```

---

## 4. UI Navigation System

### The Sound Compass

```
The primary interface for semantic navigation.

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Sound Compass                                       [≡]    │  │
│  │                                                              │  │
│  │  Reference: [punchy_trap_kick_03.wav ▾]   [↺ Replace]      │  │
│  │                                                              │  │
│  │  ┌──────────────────────────────────────────────────────┐   │  │
│  │  │                                                     │   │  │
│  │  │                 Punchier ←──●──→ Softer              │   │  │
│  │  │                                                     │   │  │
│  │  │                 Brighter ←──●──→ Darker              │   │  │
│  │  │                                                     │   │  │
│  │  │                 Warmer  ←──●──→ Colder              │   │  │
│  │  │                                                     │   │  │
│  │  │                 Aggressive ←─●──→ Gentle            │   │  │
│  │  │                                                     │   │  │
│  │  │                 More body ←──●──→ Less body          │   │  │
│  │  │                                                     │   │  │
│  │  │                 More air ←───●──→ Drier             │   │  │
│  │  │                                                     │   │  │
│  │  │                 More metal ←──●──→ More wood         │   │  │
│  │  │                                                     │   │  │
│  │  │                 More space ←──●──→ Tighter           │   │  │
│  │  │                                                     │   │  │
│  │  │                 More grit ←───●──→ Cleaner          │   │  │
│  │  │                                                     │   │  │
│  │  │                 More real ←───●──→ More synth       │   │  │
│  │  │                                                     │   │  │
│  │  └──────────────────────────────────────────────────────┘   │  │
│  │                                                              │  │
│  │  ┌──────────────────────────────────────────────────────┐   │  │
│  │  │  [Reset]  [Randomize]  [Copy current]  [Save as...] │   │  │
│  │  └──────────────────────────────────────────────────────┘   │  │
│  │                                                              │  │
│  │  ┌──────────────────────────────────────────────────────┐   │  │
│  │  │  Results [5]                                         │   │  │
│  │  │                                                      │   │  │
│  │  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐       │   │  │
│  │  │  │ ▶ 87% │ │ ▶ 82% │ │ ▶ 79% │ │ ▶ 74% │       │   │  │
│  │  │  │        │ │        │ │        │ │        │       │   │  │
│  │  │  │ 0.72s  │ │ 0.68s  │ │ 0.71s  │ │ 0.65s  │       │   │  │
│  │  │  │ ★★★★★ │ │ ★★★★  │ │ ★★★★  │ │ ★★★    │       │   │  │
│  │  │  └────────┘ └────────┘ └────────┘ └────────┘       │   │  │
│  │  │                                                      │   │  │
│  │  │  [Generate 5 more]  [Batch export]  [Add to pack]   │   │  │
│  │  └──────────────────────────────────────────────────────┘   │  │
│  │                                                              │  │
│  │  ┌──────────────────────────────────────────────────────┐   │  │
│  │  │  Natural Language: [like this but punchier and...▸]  │   │  │
│  │  └──────────────────────────────────────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

Key design decisions:
  1. Reference sound always visible — context for navigation
  2. Sliders show CURRENT position, not absolute — relative navigation
  3. Results update in real-time as sliders move (debounced 200ms)
  4. Each result shows: similarity score, duration, quality rating
  5. Natural language input at bottom — translates to slider positions
  6. "Clone" checkbox: preserve character, only modify selected axis
```

### The Sound Map (2D Explorer)

```
Alternative navigation mode: 2D latent space exploration.

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Sound Map                                           [List ▤]      │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                                                              │  │
│  │        dark                                                  │  │
│  │         │                                                    │  │
│  │         │    ┌──┐                                            │  │
│  │         │   │kick│                                           │  │
│  │         │    └──┘              ┌──┐                         │  │
│  │         │                     │hat│                          │  │
│  │         │                      └──┘                          │  │
│  │         │                                                     │  │
│  │   soft──┼───────────────────────────────●─── hard            │  │
│  │         │                              YOU                    │  │
│  │         │                                                     │  │
│  │         │        ┌──┐                                         │  │
│  │         │       │snr│                                         │  │
│  │         │        └──┘       ┌──┐                              │  │
│  │         │                  │clap│                             │  │
│  │         │                   └──┘                              │  │
│  │         │                                                     │  │
│  │        bright                                                 │  │
│  │                                                               │  │
│  │  Axis 1: Dark ↔ Bright  (PCA component 1, 22% variance)      │  │
│  │  Axis 2: Soft ↔ Hard    (PCA component 2, 15% variance)      │  │
│  │                                                               │  │
│  │  [Zoom] [Pan] [Axis: Brightness ▾] [Axis: Punch ▾]           │  │
│  │                                                               │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  At position ●:  "punchy kick, medium-bright, medium-hard"        │
│  Current: kick_03   |   Selected: snare_12 (similar character)     │
│                                                                     │
│  [▶ Preview] [♥ Favorite] [⤓ Export] [Generate at position]       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

The sound map is:
  - 2D projection of the 1024d embedding space (UMAP/t-SNE)
  - User sees all sounds as points in perceptual space
  - Near = perceptually similar. Far = different.
  - User can navigate by clicking + dragging through space
  - "Generate at position" → create new sound at clicked location
  - Axis selector: choose which dimensions map to X and Y axes
  - Type coloring: kicks in red, snares in blue, etc.
```

### Semantic Search Bar

```
The command line for sound.

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  [🔍 Search or describe...                                    ▸]  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Suggestions ▼                                               │  │
│  │                                                              │  │
│  │  ⌘R  "punchy trap kick 140bpm"                              │  │
│  │  ⌘S  "deep 808 with long tail"                              │  │
│  │  ⌘N  "like [sound] but brighter"                            │  │
│  │  ⌘M  "more metallic"                                        │  │
│  │  ⌘L  "less muddy"                                           │  │
│  │  ⌘A  "closer to old Lex Luger drums"                        │  │
│  │                                                              │  │
│  │  History ▼                                                   │  │
│  │  "punchy trap kick"                                         │  │
│  │  "like kick_03 but more aggressive"                          │  │
│  │  "cinematic impact dark"                                    │  │
│  │  "warmer but cleaner"                                       │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

Semantic search bar features:
  - Combines text search + semantic modifiers in one input
  - "punchy trap kick darker harder" → search + modify
  - "like [sound_name] but X" → reference-based
  - "closer to [producer/style]" → cultural reference
  - History shows previous successful navigations
  - Suggested completions based on current context
  - Keyboard shortcuts for power users
```

### Navigation Modes Comparison

```
┌──────────────────┬────────────────┬──────────────┬──────────────────┐
│ Mode             │ Best For       │ Friction     │ Power            │
├──────────────────┼────────────────┼──────────────┼──────────────────┤
│ Sound Compass    │ Fine-tuning    │ Low          │ High             │
│ (sliders)        │ Incremental    │              │                  │
│                  │ Adjustments    │              │                  │
├──────────────────┼────────────────┼──────────────┼──────────────────┤
│ Sound Map        │ Exploration    │ Medium       │ Very High        │
│ (2D space)       │ Discovery      │              │                  │
│                  │ Serendipity    │              │                  │
├──────────────────┼────────────────┼──────────────┼──────────────────┤
│ Semantic Search  │ Quick actions  │ Very low     │ Medium           │
│ (text bar)       │ Known intent   │              │                  │
│                  │ Power users    │              │                  │
├──────────────────┼────────────────┼──────────────┼──────────────────┤
│ Reference Morph  │ Specific       │ Low          │ High             │
│ (drag reference) │ Matching       │              │                  │
│                  │ Style transfer │              │                  │
├──────────────────┼────────────────┼──────────────┼──────────────────┤
│ Voice Control    │ Hands-free     │ Very low     │ Medium           │
│ (speech input)   │ Workflow flow  │              │                  │
│                  │ Accessibility  │              │                  │
└──────────────────┴────────────────┴──────────────┴──────────────────┘
```

---

## 5. Semantic Safety Constraints

### Why Safety Matters

```
Semantic navigation is powerful — and dangerous.

Without constraints:
  "more metallic" → dialed to 1.0 → sounds like screeching metal
  "closer to [copyrighted sound]" → exact copy of protected work
  "less muddy" → entire low end removed → thin, useless sound
  "more aggressive" → clipping, distortion, ear fatigue

Semantic navigation needs GUARDRAILS — not to limit creativity,
but to keep results musically useful.

The paradox of infinite control:
  Maximum degrees of freedom = maximum chance of bad results.
  Good constraints enable good creativity.
```

### Guardrail Types

```
Type 1 — Perceptual Bounds:
  Each axis has a MUSICAL range (not a theoretical range).
  
  ┌──────────────────────┬──────────┬──────────┬──────────────┐
  │ Axis                 │ Min      │ Max      │ Rationale    │
  ├──────────────────────┼──────────┼──────────┼──────────────┤
  │ Brightness           │ -0.8     │ 0.8      │ Past max =    │
  │                      │          │          │ harsh/piercing│
  │ Punch                │ -0.7     │ 0.9      │ Past min =    │
  │                      │          │          │ no transient  │
  │ Warmth               │ -0.8     │ 0.7      │ Past max =    │
  │                      │          │          │ muddy         │
  │ Energy               │ -0.9     │ 0.8      │ Past max =    │
  │                      │          │          │ distortion    │
  │ Grit                 │ -0.6     │ 0.7      │ Past max =    │
  │                      │          │          │ noise only    │
  │ Realism              │ -0.5     │ 0.5      │ Binary is not │
  │                      │          │          │ useful        │
  └──────────────────────┴──────────┴──────────┴──────────────┘

  These bounds are:
    - Learned from data: what range contains 95% of "good" sounds
    - Genre-aware: "punchy" has wider range in trap than in lo-fi
    - User-adaptable: expert users can expand bounds
    - Soft limits: user can push past with warning

Type 2 — Coherence Guards:
  Ensure the modified sound still makes sense as a sound.
  
  ┌────────────────────────────────────────────────────────────┐
  │ Coherence Checks:                                          │
  │                                                            │
  │ After applying semantic deltas:                            │
  │   a) Is the sound still recognizable as its type?          │
  │      "This still sounds like a kick"                       │
  │      Threshold: type classifier confidence > 0.5           │
  │                                                            │
  │   b) Does the sound have a usable transient?               │
  │      "The attack hasn't been destroyed"                    │
  │      Threshold: transient energy > -30dB                   │
  │                                                            │
  │   c) Is the frequency balance reasonable?                  │
  │      "No extreme spectral holes"                           │
  │      Threshold: no 10dB+ dip in any critical band          │
  │                                                            │
  │   d) Is the duration appropriate?                          │
  │      "A kick is still 200-800ms, not 3 seconds"           │
  │      Threshold: duration within type-specific bounds       │
  │                                                            │
  │   e) Is the sound mix-ready?                               │
  │      "Peak level isn't clipping"                           │
  │      Threshold: peak < -0.5dB, RMS within range            │
  └────────────────────────────────────────────────────────────┘

Type 3 — Copyright Guard:
  Prevent semantic navigation from reproducing copyrighted sounds.
  
  ┌────────────────────────────────────────────────────────────┐
  │ Copyright Protection:                                      │
  │                                                            │
  │   a) Reference lock: when user says "like this [ref]",     │
  │      the output must be embed-distance > threshold from    │
  │      the reference. Prevents exact copying.                │
  │                                                            │
  │   b) Producer lock: "make it sound like [producer]"        │
  │      moves toward that producer's cluster centroid,        │
  │      NEVER toward a specific copyrighted sound.            │
  │      Threshold: > 0.3 cosine distance from any commercial  │
  │      sample                                                │
  │                                                            │
  │   c) Nearest-neighbor check: before returning any result,  │
  │      check if it's within 0.05 cosine of a copyrighted     │
  │      sound in the database. If so, reject and notify.     │
  │                                                            │
  │   d) Prompt watermark: "Lex Luger drums" → sound is        │
  │      inspired by the style, not a specific sample.         │
  │      Always generate, never retrieve.                      │
  └────────────────────────────────────────────────────────────┘

Type 4 — Quality Guardrail:
  Ensure navigation doesn't produce bad-sounding results.
  
  ┌────────────────────────────────────────────────────────────┐
  │ Quality Checks:                                            │
  │                                                            │
  │   After navigation, run SoundScore on result:              │
  │     - Score < 60: reject, suggest alternative direction    │
  │     - Score 60-70: warn user, show score                   │
  │     - Score > 70: proceed normally                         │
  │                                                            │
  │   Specific quality checks:                                 │
  │     - No phase cancellation artifacts                      │
  │     - No DC offset                                         │
  │     - No ultrasonic artifacts (>22kHz)                     │
  │     - Transient-to-noise ratio > threshold                 │
  │     - Spectral flatness within expected range              │
  └────────────────────────────────────────────────────────────┘

Type 5 — User-Specific Guardrails:
  Learn from user behavior what "too far" means.
  
  ┌────────────────────────────────────────────────────────────┐
  │ Personal Calibration:                                      │
  │                                                            │
  │   For each user, learn:                                    │
  │     - Their preferred range for each axis                  │
  │     - Their acceptable quality threshold                   │
  │     - Their genre-specific preferences                     │
  │     - Their "too much" boundary per modifier               │
  │                                                            │
  │   Personal guardrails tighten navigation bounds:           │
  │     - User A: "punchy" max = 0.6 (likes softer kicks)     │
  │     - User B: "punchy" max = 0.9 (likes aggressive)       │
  │                                                            │
  │   Calibration updates every 50 interactions:               │
  │     - If user backs off "punchy" from 0.7 to 0.4:         │
  │       new soft limit = 0.7 (they don't like extreme)      │
  └────────────────────────────────────────────────────────────┘
```

### Guardrail Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  User Intent: "more punchy, magnitude 0.5"                         │
│         │                                                           │
│         ▼                                                           │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Step 1: Axis Bound Check                                     │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │  Requested: punchy +0.5 from base (base=0.3) → 0.8    │  │  │
│  │  │  Check: 0.8 <= user_punchy_max(0.85) ✓                 │  │  │
│  │  │  Check: 0.8 <= global_punchy_max(0.9) ✓                │  │  │
│  │  │  Result: PASS                                           │  │  │
│  │  └────────────────────────────────────────────────────────┘  │  │
│  │                                                               │  │
│  │  Step 2: Embedding Operation                                  │  │
│  │  embed_target = embed_ref + 0.5 * punchy_direction            │  │
│  │                                                               │  │
│  │  Step 3: Coherence Check                                      │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │  Type classifier: kick (confidence 0.92) > 0.5 ✓      │  │  │
│  │  │  Transient energy: -22dB > -30dB ✓                     │  │  │
│  │  │  Spectral balance: no holes ✓                          │  │  │
│  │  │  Duration: 412ms (within bounds) ✓                     │  │  │
│  │  │  Result: PASS                                           │  │  │
│  │  └────────────────────────────────────────────────────────┘  │  │
│  │                                                               │  │
│  │  Step 4: Copyright Check                                      │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │  Nearest copyright sound distance: 0.41 > 0.05 ✓      │  │  │
│  │  │  Result: PASS                                           │  │  │
│  │  └────────────────────────────────────────────────────────┘  │  │
│  │                                                               │  │
│  │  Step 5: Quality Check                                        │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │  SoundScore: 78 > 70 ✓                                 │  │  │
│  │  │  Phase: clean ✓, DC: none ✓, Ultrasonic: none ✓       │  │  │
│  │  │  Result: PASS                                           │  │  │
│  │  └────────────────────────────────────────────────────────┘  │  │
│  │                                                               │  │
│  │  ALL CHECKS PASSED → Return result                            │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  If ANY check fails:                                                │
│    ┌────────────────────────────────────────────────────────────┐  │
│    │  "This modification would make the sound less usable.      │  │
│    │   Try a smaller adjustment, or unlock advanced mode."      │  │
│    │  [Reduce to 0.3]  [Advanced Mode]  [Cancel]               │  │
│    └────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 6. Explainability Methods

### Why Explainability Matters

```
Semantic navigation is opaque to users:
  "I moved the 'punchy' slider and something changed. What exactly?"

Users need to understand:
  - What does this axis MEAN?
  - What changed in the sound?
  - How much did it change?
  - Why did this result appear?

Without explainability, navigation feels like magic.
With explainability, it feels like control.
```

### Explanation Types

```
Type 1 — Spectral Visualization:
  Show the frequency response change.
  
  ┌────────────────────────────────────────────────────────────┐
  │  Before: ▁▂▃▅▇███▇▅▃▂▁  (dark kick, low-mid focused)    │
  │  After:  ▁▂▃▅▇██████▇▅▃  (brighter, more high-end)      │
  │  Delta:  +2.3dB at 3kHz, +1.1dB at 5kHz                  │
  │  Explanation: "More metallic' boosted the 3-5kHz range   │
  │  where metallic resonance lives."                        │
  └────────────────────────────────────────────────────────────┘

Type 2 — Waveform Overlay:
  Show the transient shape change.
  
  ┌────────────────────────────────────────────────────────────┐
  │  Before:  ▁▂▃▄▅▆▇████▇▆▅▄▃▂▁_ _ _ _ _                   │
  │  After:   ▁▂▃▄▅▆██████████▇▆▅▄▃▂▁_ _ _                  │
  │  Delta:   Attack shortened by 8ms, peak +2.1dB            │
  │  Explanation: "'Punchier' sharpened the attack transient  │
  │  and increased peak level."                              │
  └────────────────────────────────────────────────────────────┘

Type 3 — Axis Contribution Chart:
  Show which axes were modified and by how much.
  
  ┌────────────────────────────────────────────────────────────┐
  │  Axis Changes Applied:                                     │
  │                                                            │
  │  Punchy:    ████████░░░░░░░░  +0.5  (significant)        │
  │  Bright:    ██░░░░░░░░░░░░░░  +0.1  (minor)              │
  │  Dark:      ░░░░░░░░░░░░░░░░   0.0  (unchanged)          │
  │  Warm:      ░░░░░░░░░░░░░░░░   0.0  (unchanged)          │
  │  Metallic:  ████████░░░░░░░░  +0.5  (significant)        │
  │                                                            │
  │  Unintended changes:                                       │
  │  Energy:    ██░░░░░░░░░░░░░░  +0.15 (minor side effect)  │
  └────────────────────────────────────────────────────────────┘

Type 4 — Embedding Direction Visualization:
  Show the direction in embedding space relative to the overall distribution.

  ┌────────────────────────────────────────────────────────────┐
  │  Direction "punchier" moves sound:                         │
  │    Toward: 80% of trap kicks, 20% of lo-fi kicks          │
  │    Away from: 90% of ambient textures                     │
  │    Nearest labeled landmark: "Metro Boomin kick"           │
  │    Style cluster: Modern trap production                   │
  └────────────────────────────────────────────────────────────┘

Type 5 — Natural Language Explanation:
  Generate human-readable description of what changed.
  
  ┌────────────────────────────────────────────────────────────┐
  │  "I made your kick 30% punchier by sharpening the attack  │
  │   transient. It now has a harder initial hit that cuts    │
  │   through the mix better. I also made it slightly (10%)   │
  │   brighter as a side effect, which is typical for punchy  │
  │   kicks. You can dial it back if you want."               │
  └────────────────────────────────────────────────────────────┘
```

---

## 7. Implementation Roadmap

```
Phase 1 — Core Engine (2 months):
  ✓ Build axis direction library (30+ semantic directions)
  ✓ Build embedding navigation core (vector arithmetic)
  ✓ Build guardrail system (perceptual bounds + coherence)
  ✓ Build nearest-neighbor search path
  ✓ Unit tests for each axis direction

Phase 2 — UI (1 month):
  ✓ Sound Compass (10-axis slider interface)
  ✓ Semantic search bar with suggestions
  ✓ Real-time result updates (debounced 200ms)
  ✓ Reference sound selector
  ✓ Natural language parser (basic)

Phase 3 — Advanced (2 months):
  ✓ Sound Map (2D UMAP projection)
  ✓ Reference morph drag-and-drop
  ✓ Cultural axis discovery pipeline
  ✓ Copyright guardrail
  ✓ Quality guardrail + SoundScore integration

Phase 4 — Personalization (1 month):
  ✓ Personal axis calibration per user
  ✓ User-specific guardrail bounds
  ✓ Personal axis discovery
  ✓ Explainability UI (all 5 types)
  ✓ Voice control (speech-to-axis)
  ✓ A/B test: semantic nav vs folder browsing (target: 3x faster)

Total timeline: ~6 months to full semantic navigation experience
```

---

## 8. Summary

```
Semantic Navigation Through Sound

  Core insight:
    Producers think in perceptual dimensions, not folder trees.
    Navigation should operate along those dimensions directly.

  Architecture:
    Four layers: NL parser → axis engine → embedding ops → navigation UI
    Guardrails at every step: perceptual, coherence, copyright, quality, personal

  Key capabilities:
    "punchier" → +0.3 on transient axis → real-time result
    "like this but darker" → reference + spectral delta
    "closer to Lex Luger" → cultural axis vector
    "warmer but cleaner" → multi-axis compound
    "more metallic, less muddy" → delta composition

  UI modes:
    Sound Compass (sliders — fine control)
    Sound Map (2D — exploration)
    Semantic Search (text — quick)
    Reference Morph (drag-drop — matching)

  This changes the fundamental interaction model of sound design:
    From: "Browse folders until you find something close enough."
    To:   "Describe the change you want. The system takes you there."
```

