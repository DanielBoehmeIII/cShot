# Prompt 32 — Make cShot Feel Faster Than Searching Splice

A producer should get a usable, unique sound faster than they can search Splice.

---

## 1. The Goal

### The Benchmark

```
Splice search flow:
  1. Open browser (2-5 seconds)
  2. Type query: "dark trap kick" (3 seconds)
  3. Browse results — scan tags, pack names (5-10 seconds)
  4. Preview sounds — click each one, listen (10-30 seconds per sound)
  5. Find 3-5 candidates (30-60 seconds total)
  6. Download (2-5 seconds)
  7. Drag into DAW (2 seconds)
  Total: ~45-90 seconds for a good result, often 3-5 minutes

cShot target:
  1. Think of sound (0 seconds — you're already in the flow)
  2. Type/press one key (1 second)
  3. Sound appears, already unique (2-5 seconds generation)
  4. Drag into DAW (2 seconds)
  Total: ~5-10 seconds for a unique, mix-ready sound
```

### Why It Matters

```
Splice friction kills momentum:
  - Every second searching is a second not creating
  - Decision fatigue from too many options
  - Preview fatigue: your ears desensitize after 10-15 sounds
  - Settling: you pick "good enough" because you're tired of searching
  - The best session is when you never leave the DAW

cShot eliminates the search entirely:
  - You don't browse sounds — you create them
  - Every sound you make is unique (no "this sounds like every other track")
  - The sound fits your context, not some generic pack
  - Generation is faster than previewing 5+ samples
```

---

## 2. Current Sample-Search Friction Analysis

### Friction Points

| Friction | Pain Level | Description | cShot Solution |
|----------|-----------|-------------|----------------|
| **Search latency** | High | 2-5 seconds page load per search | Zero-latency local generation |
| **Scroll fatigue** | High | Scanning 50+ results for 1 good sound | No results to scan — one sound, good |
| **Preview fatigue** | Critical | Ears fatigue after 10-15 previews | Heuristics pick the best; user hears winner first |
| **Bad metadata** | Medium | "Dark kick" that isn't dark, "Punchy" that isn't | Semantic understanding: prompt matches output |
| **Duplicate packs** | Medium | Same 808 in 15 different packs | Every sound is unique — no duplicates |
| **Genre mismatch** | High | Pack labeled "trap" but sounds like house | Genre-aware generation from prompt + context |
| **Mix-readiness** | Critical | Splice samples almost always need EQ/fixing | Auto-mix-ready processing (Prompt 35) |
| **Decision paralysis** | Critical | 10,000 kick choices → freeze | AI proposes, user decides (fewer, better options) |
| **Context mismatch** | High | Sample at 100BPM in a 140BPM track | Auto-stretch, auto-key, auto-EQ |
| **No memory** | Medium | "Where was that kick I liked last week?" | Favorite memory + taste profile |

### The Decision Fatigue Curve

```
Quality of Decision
    ↑
 10 │                                    ┌──────────── Splice (10,000 options)
    │                                 ───│
  8 │                           ───     │
    │                      ───          │
  6 │                 ───               │
    │            ───                    │
  4 │       ───                         │
    │  ───                              │ cShot (6 options, AI-curated)
  2 │                                   
    └──────────────────────────────────────→ Number of Options
       1   5   10   50   100  500  1000

  cShot: 6 curated options → peak decision quality
  Splice: 50-1000 options → decision quality collapses
```

---

## 3. Instant Auditioning

### The Principle

```
Zero-wait audition: Every sound is immediately playable.
No "loading," no "buffering," no "generating spinner."

The moment a sound appears in the grid, it's ready to play.
```

### Pre-generation Strategies

```python
class InstantAuditionEngine:
    """
    Multiple strategies to make generation feel instant.
    """
    
    @staticmethod
    def cache_warmer():
        """Pre-generate common sounds during idle time."""
        # CPU idle → generate in background
        # Stored in LRU cache (1GB budget)
        # Common prompts: "punchy kick", "trap snare", "open hat"
        pass
    
    @staticmethod
    def progressive_refinement():
        """Show a low-quality preview while refining."""
        # t=0ms: Return cached similar sound (from embedding search)
        # t=500ms: Replace with streaming HiFi output
        # t=2000ms: Full quality mix-ready sound
        pass
    
    @staticmethod
    def latent_interpolation():
        """Start from nearest neighbor, morph to target."""
        # Find closest cached sound in latent space
        # Interpolate from cached → generated
        # User hears something immediately, it "evolves" into final
        pass
    
    @staticmethod
    def template_matching():
        """Use DSP templates for instant generation, refine with AI."""
        # Have 1000+ DSP templates (parametric kicks, snares, etc.)
        # Match prompt → closest template → instant sound
        # AI refines parameters in background
        pass
```

### Audition UX

```
Three modes, user-configurable:

1. Auto-Preview (default)
   - Click sound → instant playback
   - No loading state ever
   - Cross-fade between previews (10ms)
   - Playback position shown in waveform

2. MIDI Trigger
   - Sound mapped to MIDI note
   - Play from keyboard/controller
   - Velocity-sensitive preview
   - Can sequence cShot sounds from DAW

3. Beat-Sync Preview
   - Sound loops at project BPM
   - Auto-detects one-shot length vs loop
   - Quantized to grid
   - Hear it in context immediately
```

---

## 4. Generate-Near-This Workflows

### The Concept

```
"I like this sound, but I need it slightly different."
Not "generate from scratch" — "mutate what I have."
```

### Workflows

#### 4.1 "More Like This"

```
User hears Sound A in the grid.
User clicks "↻ More Like This"
cShot generates 6 variants of Sound A:
  ✓ Sound A₁: Slightly brighter
  ✓ Sound A₂: More sub, less click  
  ✓ Sound A₃: Longer decay
  ✓ Sound A₄: More punch
  ✓ Sound A₅: Different harmonic content
  ✓ Sound A₆: Random mutation
Generation time: ~1-2 seconds (pre-computed latent neighborhood)
```

#### 4.2 "Between These Two"

```
User selects Sound A and Sound B.
User clicks "↻ ⟷ " (Interpolate)
cShot generates 6 sounds interpolating between A and B:
  0% A, 100% B  → Sound B
  20% A, 80% B
  40% A, 60% B
  60% A, 40% B  ← Maybe the sweet spot
  80% A, 20% B
  100% A, 0% B  → Sound A
User can slide the interpolation ratio continuously.
```

#### 4.3 "Fit This Context"

```
User drags Sound A to a track.
cShot detects: 140 BPM, C minor, track is "trap brass"
User clicks "↻ Fit Context"
cShot transforms Sound A:
  - Time-stretch to 140 BPM
  - Pitch-shift to C minor
  - EQ to avoid masking existing mix elements
  - Adjust decay length for genre norms
  - Apply gentle saturation for mix glue
```

#### 4.4 "Make It Sound Like This Reference"

```
User imports or records a reference sound.
User clicks "↻ Match Reference"
cShot analyzes target acoustic features:
  - Spectral centroid
  - Transient shape
  - Harmonic profile
  - Noise floor
  - Envelope shape
Applies Sound DNA matching to transform current sound toward reference.
```

---

## 5. Semantic Search (Fast Finding, Not Browsing)

### Query Types

```
cShot understands sound descriptions at multiple levels of abstraction:

Concrete:    "808 kick, 100BPM, short decay, tuned to C"
Descriptive: "punchy dark kick with sub presence"
Emotional:   "aggressive but controlled, like anger with discipline"
Metaphor:    "a kick that sounds like black velvet"
Reference:   "like the kick in Travis Scott's 'SICKO MODE'"
Material:    "kick made of concrete hitting wet wood"
Scene:       "kick that would play in a cyberspace nightclub"
Abstract:    "a kick that feels like the number 7"
```

### Search Architecture

```
┌──────────────┐    ┌──────────────────┐    ┌────────────┐
│  User Query  │───→│  Query Encoder   │───→│  Embedding │
│  (text)      │    │  (CLAP-style)    │    │  (768-d)   │
└──────────────┘    └──────────────────┘    └────────────┘
                                                    │
┌──────────────┐    ┌──────────────────┐            │
│  Sound Grid  │←───│  Similarity Sort │←───────────┘
│  (results)   │    │  + Rerank        │    ┌────────────┐
└──────────────┘    └──────────────────┘    │  Sound DB  │
                                            │  (embeds)  │
                                            └────────────┘
```

### Search vs Generate Toggle

```
User can switch between two modes seamlessly:

🔍 Search mode:
  - Query → find in library (locally stored + cached)
  - Results in <100ms
  - Best for known sound types
  
⚡ Generate mode:
  - Query → generate new sound
  - Results in 2-5 seconds
  - Best for "I need something I've never heard"

Or hybrid: search first, if no great match → auto-generate
```

---

## 6. One-Click Variation

### Variation Dimensions

Each dimension is independently controllable. One click = mutate one axis.

```
┌────────────────────────────────────────────────────┐
│  One-Click Variation Controls                      │
├────────────────────────────────────────────────────┤
│  [↻ Pitch] [↻ Decay] [↻ Punch] [↻ Tone]           │
│  [↻ Space] [↻ Texture] [↻ Body] [↻ Attack]        │
│  [↻ ^ Random]  (lucky dip — mutate all)            │
└────────────────────────────────────────────────────┘
```

### What Each Button Does

```
[↻ Pitch]:
  - Transpose +0 to +12 semitones (random within range)
  - Or: tune to nearest harmonic of project key
  - "Pitch, but keep character"

[↻ Decay]:
  - Scale envelope release 0.5x-2x
  - Respects BPM context (won't exceed 1 bar in most cases)
  - Auto-adjusts sustain for smooth tail

[↻ Punch]:
  - Adjust attack transient: faster attack + more click
  - Or: layer a transient from another sound
  - Spectral shaping to emphasize 2-5kHz transient region

[↻ Tone]:
  - Shift spectral centroid up/down
  - Apply EQ shelf (boost/cut at 200Hz, 2kHz, 8kHz)
  - "Dark" vs "Bright" slider
  - Changes color without changing pitch/decay

[↻ Space]:
  - Add/reduce reverb (convolution or algorithmic)
  - "Dry" → "Room" → "Hall" → "Cathedral"
  - Pre-delay adjusts to BPM
  - Stereo width control

[↻ Texture]:
  - Add saturation (tape, tube, digital)
  - Bit crush, sample rate reduction
  - Noise layer (vinyl, tape hiss, digital)
  - Granular texture (scatter, stretch grains)

[↻ Body]:
  - Adjust fundamental frequency region (60-120Hz for kicks)
  - Sub vs mid balance
  - Body material simulation (paper, wood, metal, plastic)
  - Shell resonance tuning

[↻ Attack]:
  - Attack time (0.1ms → 50ms)
  - Initial transient shape (sharp → soft)
  - Pre-click vs initial transient balance
```

### Variation Speed

```
Generation time per variation:
  - DSP-only mutations: <10ms (instant)
  - AI-assisted mutations: 200-500ms
  - Full regeneration: 2-5 seconds

UX: DSP mutations happen in real-time as you click.
     AI mutations show a brief progress indicator.
     Full regeneration updates the slot progressively.
```

---

## 7. Smart Favorite Memory

### Implicit Favorites

```
cShot remembers what you liked without you having to click ★.

    "You've previewed this sound 8 times without exporting it.
     You keep coming back to it. That means something.
     I've added it to your implicit favorites."

Signals:
  - Preview count (per session and cross-session)
  - Preview duration (listening to the full sound vs skipping)
  - Export count (exported sound reused across projects)
  - Re-generation (generated similar sound twice)
  - Export with no modifications (it was good as-is)
  - Return rate (came back to this sound after generating more)
```

### Explicit Favorites

```
★ = Favorites are:
  - Cross-device synced (if cloud enabled)
  - Tagged with context at time of favoriting
    ("Favorite: trap kick, 140BPM, C#m, project 'nightcity'")
  - Searchable: "show me my favorite kicks from last month"
  - Groupable: "these 5 kicks are my 'heavy hitter' set"
  - Rateable: 1-5 stars after favoriting (or thumbs up/down)
```

### Contextual Recall

```
"I notice you're working on a house track at 126 BPM.
 The last time you made house at 126 BPM, you favorited 
 these 3 kicks, 2 snares, and a clap set.
 They're at the top of your recommendations."
```

### Taste Profile Evolution

```
Your taste profile learns over time:

Initial (cold start):
  - Genre preferences (from self-selection)
  - Prompt history patterns
  - Rejection patterns (what you generate but never use)

After 10 sessions:
  - What keywords correlate with favorites
  - What parameter ranges you prefer
  - What BPM/keys you work in most

After 100 sessions:
  - Your "signature" sound profile (see Prompt 24)
  - Genre-specific preferences
  - Time-based patterns (morning/evening sound preferences)
  - Project-specific taste (what you like for "dark" vs "uplifting" tracks)
```

---

## 8. Automatic Tagging

### Why Auto-Tagging Matters

```
Without tags: you have a folder of "sound_1245.wav"
With tags: you have a searchable, organizable library.

Manual tagging is too slow. Auto-tagging must happen:
  - At generation time (zero extra effort)
  - On import (for any sound dragged into cShot)
  - On export (embedded in WAV metadata)
```

### Tag Categories

```
Acoustic Tags (always generated):
  - Type: kick, snare, hat, clap, perc, fx, bass, synth, atmos, vocal
  - Pitch: estimated fundamental frequency
  - Duration: short (<200ms), medium, long (>2s), sustained
  - Dynamics: punchy, soft, compressed, dynamic
  - Spectral: dark, warm, bright, airy, honky, nasal
  - Texture: clean, gritty, distorted, noisy, metallic

Perceptual Tags (from Sound DNA):
  - Emotional: aggressive, calm, joyful, sad, tense, playful
  - Material: wood, metal, plastic, paper, glass, organic
  - Space: dry, room, hall, cathedral, plate, spring
  - Movement: static, swelling, pulsing, decaying, evolving

Production Tags:
  - Genre fit: trap, house, techno, pop, ambient, cinematic
  - BPM range: 60-80, 80-100, 100-120, 120-140, 140-160, 160-200
  - Mix role: sub, body, attack, texture, FX
  - Processing: processed, raw, saturated, compressed, reverbed
```

### Tagging Pipeline

```
┌─────────────┐
│  Sound      │ (raw audio, 44.1kHz, mono, variable length)
└──────┬──────┘
       ↓
┌─────────────┐
│  Feature    │ Compute: MFCCs, spectral centroid, zero-crossing rate,
│  Extraction │          onset strength, pitch salience, chroma, etc.
└──────┬──────┘
       ↓
┌─────────────────┐
│  Classifier     │ Multi-label model (trained on 100K+ labeled sounds)
│  Ensemble       │ Type classifier, material classifier, emotion classifier
└──────┬──────────┘
       ↓
┌─────────────────┐
│  Tag Generator  │ Combine predictions, confidence scores, human-readable tags
└──────┬──────────┘
       ↓
┌─────────────────┐
│  Tag Storage    │ SQLite + embedded in WAV metadata (iXML chunk)
└─────────────────┘
```

### Tag Confidence

```
Tags shown with confidence indicators:

  kick          (98% confidence — model is certain)
  dark          (72% confidence — moderately certain)
  ominous       (45% confidence — unsure, shown with ?)
  
  User can:
  - Confirm: ✓ → fixes tag, removes uncertainty
  - Reject: ✗ → removes tag, adds negative training signal
  - Correct: edits tag → adds training example
  - Add: free-text input for custom tags
```

---

## 9. Context-Aware Recommendations

### What the System Knows

```
At any moment, cShot knows:
  ✓ Project tempo and key (from DAW)
  ✓ What you're working on (track name, genre if set)
  ✓ What you've made recently (last 50 generations)
  ✓ What you've used (favorites, exports)
  ✓ What you've rejected (generated but not used)
  ✓ Your taste profile (from 100+ sessions)
  ✓ Current DAW context (arrangement position, track type)
  
With this context, recommendations become powerful.
```

### Recommendation Types

```
1. "What You Need Next" (proactive)
   - "You just generated a kick — want a matching snare?"
   - "I notice you don't have a clap yet for this track"

2. "Complete the Set"  
   - Generate a full kit from one seed sound
   - "This kick you made — here's a snare, hat, and perc that match"

3. "What's Hot" (trend-aware)
   - "This year in trap: shorter kicks, more metallic claps"
   - "cShot users making techno right now favor minimal kicks"

4. "Unexplored Territory"
   - "You've made 50 kicks but never tried this material"
   - "What if you made a kick out of glass?"

5. "Vibe Matcher"
   - "I know you're making an ambient track — here are kicks 
      designed to not punch through but sit back in the mix"
```

### Recommendation Algorithm

```
Score(sound, context) = w₁ · relevance_score + 
                        w₂ · novelty_score + 
                        w₃ · taste_score + 
                        w₄ · completion_score

relevance_score = embedding_cosine(query, sound)
novelty_score   = 1 - max_similarity(sound, user_history[last_50])
taste_score     = taste_profile.predict_preference(sound)
completion_score = completes_set(context.current_set, sound)

Return top 6 sounds sorted by Score.
```

---

## 10. Speed Benchmarks

### Target Latencies

| Operation | Current Target | Stretch Goal |
|-----------|---------------|--------------|
| Prompt entry → first preview | 2 seconds | 500ms |
| "More like this" variants | 1 second | 200ms |
| One-click mutation | 10ms | 1ms |
| Favorite memory recall | 50ms | 10ms |
| Semantic search (10K library) | 100ms | 20ms |
| Semantic search (100K library) | 500ms | 100ms |
| Auto-tag after generation | 200ms | 50ms |
| Recommendation refresh | 100ms | 20ms |
| Drag-to-DAW export | 100ms | 20ms |
| Full cold start (first open) | 2 seconds | 500ms |

### Speed vs Quality Tradeoffs

```
Fast Lane (always < 1 second):
  - DSP-only operations (mutations, parameter changes)
  - Template-based generation (from 1000 pre-made templates)
  - Library search (pre-computed embeddings)
  - Stretching/pitching existing sounds

Quality Lane (2-5 seconds):
  - Full AI generation from scratch
  - Sound DNA matching
  - Semantic interpolation
  - Context-aware adaptation

Background (async, no wait):
  - Auto-tagging enhancement
  - Embedding computation
  - Library indexing
  - Taste profile updates
  - Cache warming

The key insight: user never waits for the Quality Lane.
  - Show instant results from Fast Lane first
  - Refine to Quality Lane while user previews
  - User perceives: "it's always instant"
```

---

## Summary

| Principle | Why It's Faster Than Splice |
|-----------|---------------------------|
| Zero search | Generate, don't browse |
| Instant audition | No loading state ever |
| One-click variation | 10ms mutation vs 30s of searching |
| Smart memory | System remembers, you don't |
| Auto-tagging | No manual organization |
| Context awareness | Recommendations fit your session |
| Progressive refinement | Instant then perfect, not wait then good |
| Generate-near-this | 1s for what you need vs minutes finding it |
| Semantic search | Describe, don't filter |
| Mix-ready output | No post-processing needed |

The Splice workflow is "browse → audition → decide → download → drag → process."
The cShot workflow is "describe → drag."
