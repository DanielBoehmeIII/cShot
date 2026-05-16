# Prompt 76 — Sonic Social Networks

Explore the idea of a social layer for sound creation. Users share generations, branch sounds, remix packs, follow taste profiles, publish sound worlds, collaborate on themes, and evolve shared sonic identities. Propose social graph architecture, feed algorithms, collaborative workflows, identity systems, and creator discovery mechanisms.

---

## 1. Why Social?

### The Isolation Problem

```
Current sound creation is fundamentally solitary.

Producer workflow today:
  1. Open DAW alone
  2. Search for sounds alone
  3. Process sounds alone
  4. Write music alone
  5. Export track alone
  6. Maybe share finished track on social media

  99% of the process is invisible to others.
  No feedback during creation. No collaboration. No community.

  Compare to:
    - Visual art: artists share WIPs, get feedback, iterate publicly
    - Code: developers PR, review, pair program, share snippets
    - Music performance: bands jam together, feed off each other
    
  Sound design has no real-time social layer.
  That's a product gap, not a user preference.

The hypothesis:
  Sound creation is social because sound itself is cultural.
  A kick drum doesn't exist in a vacuum — it exists in the context
  of tracks, genres, scenes, and communities.
  
  A social layer for sound creation would:
    - Make creators feel less isolated
    - Accelerate skill development through feedback
    - Enable new creative workflows (collaborative sound design)
    - Create community-owned sonic identities
    - Generate network effects that strengthen the platform
```

### What Sonic Social Means

```
Sonic Social ≠ Audio Social Media.

  Audio Social Media (SoundCloud, Spotify):
    - Share finished tracks
    - Listen and comment
    - Follow artists
    - Passive consumption

  Sonic Social (cShot):
    - Share individual sounds (not just finished tracks)
    - Branch and remix each other's sounds
    - Co-create in real-time
    - Follow taste profiles (not just people)
    - Publish sound worlds (not just albums)
    - Evolve shared sonic identities
    - Active creation, not passive consumption

  Key insight:
    SoundCloud is social around LISTENING.
    cShot Social is social around MAKING.
    
    The unit of social interaction is the SOUND, not the track.
    A producer sharing a kick they made is more intimate,
    more collaborative, and more useful than sharing a finished song.
```

---

## 2. Social Graph Architecture

### Graph Entities and Relationships

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Nodes:                                                             │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐  │
│  │  User      │  │  Sound     │  │  Pack      │  │  Prompt    │  │
│  │  (creator) │  │  (one-shot)│  │  (collection)│  │  (text)    │  │
│  └────────────┘  └────────────┘  └────────────┘  └────────────┘  │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐  │
│  │  Remix     │  │  Taste     │  │  World     │  │  Genre     │  │
│  │  (derived) │  │  (profile) │  │  (universe)│  │  (category)│  │
│  └────────────┘  └────────────┘  └────────────┘  └────────────┘  │
│                                                                     │
│  Edges (Relationships):                                            │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                                                              │  │
│  │  User ───FOLLOWS───▶ User          (social graph)           │  │
│  │  User ───CREATED──▶ Sound         (attribution)             │  │
│  │  User ───EXPORTED─▶ Sound         (usage signal)            │  │
│  │  User ───FAVORITED▶ Sound         (preference signal)       │  │
│  │  User ───PURCHASED▶ Pack          (marketplace signal)      │  │
│  │  User ───HAS_TASTE▶ TasteProfile  (personalization)         │  │
│  │                                                              │  │
│  │  Sound ──REMIX_OF──▶ Sound        (lineage)                 │  │
│  │  Sound ──PART_OF───▶ Pack         (membership)              │  │
│  │  Sound ──SIMILAR_TO▶ Sound        (embedding distance)      │  │
│  │  Sound ──MATCHES───▶ Prompt       (alignment)               │  │
│  │  Sound ──BELONGS_TO▶ Genre        (classification)          │  │
│  │                                                              │  │
│  │  Pack ──INCLUDES───▶ Sound        (composition)             │  │
│  │  Pack ──COLLAB_ON──▶ User+User    (co-creation)             │  │
│  │  Pack ──PUBLISHED_IN▶ World       (context)                 │  │
│  │                                                              │  │
│  │  TasteProfile ──ALIGNS_WITH▶ Pack (recommendation)          │  │
│  │  TasteProfile ──SIMILAR_TO──▶ TasteProfile (taste matching) │  │
│  │                                                              │  │
│  │  World ──CONTAINS──▶ Pack        (curated collection)        │  │
│  │  World ──THEMED_AS─▶ Theme       (conceptual anchor)         │  │
│  │                                                              │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  Graph DB: Neo4j for relationship-heavy queries                    │
│  Vector DB: pgvector for similarity queries                        │
│  Combined: Neo4j → get candidate IDs → pgvector → rank by sim    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Social Queries

```
Query 1 — Feed Generation:
  "Show me sounds from people I follow, ranked by relevance."

  MATCH (user:User {id: $user_id})-[:FOLLOWS]->(followed:User)
  MATCH (followed)-[:CREATED]->(sound:Sound)
  OPTIONAL MATCH (user)-[:FAVORITED]->(sound)
  RETURN sound, 
         followed.name AS creator,
         CASE WHEN fav IS NOT NULL THEN true ELSE false END AS favorited
  ORDER BY sound.created_at DESC
  LIMIT 50

Query 2 — Taste-Matched Discovery:
  "Show me sounds that match my taste that I haven't heard."

  taste_embed = get_user_taste_embedding($user_id)
  heard_ids = MATCH (user)-[:FAVORITED|EXPORTED]->(s:Sound) RETURN s.id
  
  SELECT s.*, cosine_distance(s.embedding, taste_embed) AS distance
  FROM sounds s
  WHERE s.id NOT IN heard_ids
    AND s.created_at > NOW() - INTERVAL '7 days'
  ORDER BY distance ASC
  LIMIT 20

Query 3 — Remix Tree:
  "Show me the full remix lineage of this sound."

  MATCH (sound:Sound {id: $sound_id})
  MATCH path = (sound)-[:REMIX_OF*0..5]->(ancestor:Sound)
  MATCH descendant_path = (descendant:Sound)-[:REMIX_OF*1..5]->(sound)
  RETURN ancestor, sound, descendant
  // Full tree: roots → current → branches

Query 4 — Collaborative Discovery:
  "Find creators whose taste overlaps with mine for potential collab."

  my_taste = get_user_taste_embedding($user_id)
  
  SELECT u.id, u.name, 
         cosine_distance(u.taste_embedding, my_taste) AS taste_distance,
         COUNT(collab.id) AS collabs_done
  FROM users u
  WHERE u.id != $user_id
    AND cosine_distance(u.taste_embedding, my_taste) < 0.3
    AND u.reputation_score > 500
  ORDER BY taste_distance ASC
  LIMIT 10
```

---

## 3. Feed Algorithms

### Feed Types

```
Feed 1 — Following Feed (Social):
  Content: Sounds/packs from followed creators
  Order: Recency (newest first)
  Purpose: Stay updated with creators you care about

Feed 2 — Discovery Feed (Taste):
  Content: Sounds/packs matching your taste embedding
  Order: Taste match score × freshness decay
  Purpose: Find new sounds you'll love
  Diversity: MMR to avoid all similar sounds

Feed 3 — Trending Feed (Popularity):
  Content: Sounds/packs with most engagement this period
  Order: Engagement velocity (newly hot rises fast)
  Purpose: What's popular in the community
  Periods: 24h, 7d, 30d, all-time

Feed 4 — Collaborative Feed (Network):
  Content: Sounds from your taste-twin creators
  Order: Taste match × recency
  Purpose: "People like you also like these creators"

Feed 5 — Remix Feed (Lineage):
  Content: New remixes of sounds you created/favorited
  Order: Recency
  Purpose: See how your sounds evolve through the community

Feed 6 — World Feed (Curation):
  Content: Sounds in a specific Sound World
  Order: Curator's choice (manual ordering)
  Purpose: Immersive exploration of a theme
```

### Feed Ranking Algorithm

```
Score = Σ(weight_i × feature_i)

Features:
  ┌─────────────────────────┬────────┬────────────────────────┐
  │ Feature                 │ Weight │ Source                  │
  ├─────────────────────────┼────────┼────────────────────────┤
  │ Creator follow weight   │ 0.20   │ Follow graph           │
  │ Taste match             │ 0.25   │ Embedding cosine       │
  │ Engagement velocity     │ 0.15   │ Likes/remixes/hr       │
  │ Freshness               │ 0.10   │ exp(-days_since/7)     │
  │ Creator reputation      │ 0.05   │ Reputation score       │
  │ Sound quality           │ 0.10   │ SoundScore             │
  │ Social proof            │ 0.10   │ Total likes + remixes  │
  │ Recency of interaction  │ 0.05   │ Last interaction with  │
  │                         │        │ this creator           │
  └─────────────────────────┴────────┴────────────────────────┘

Personalization:
  Weights are USER-ADAPTIVE.
  If user mostly engages with Discovery feed → taste_match weight increases.
  If user mostly follows specific creators → follow_weight increases.
  Weights re-calibrated weekly per user.

Diversity enforcement:
  After ranking top 200, apply MMR (Maximal Marginal Relevance):
    Selected = []
    Remaining = top 200
    While len(Selected) < 50:
      best = argmax(score_i - λ × max(sim(Selected, i)))
      Selected.append(best)
      Remaining.remove(best)
    λ = diversity factor (0.3 default, tunable)
  
  Result: 50 diverse, high-quality, personalized feed items.
```

### Interaction Signals

```
Each interaction is a signal for the feed algorithm:

  ┌────────────────────┬──────────┬──────────┬────────────────┐
  │ Interaction        │ Weight   │ Type     │ Feed Effect    │
  ├────────────────────┼──────────┼──────────┼────────────────┤
  │ Like sound         │ +1.0     │ Explicit │ More like this │
  │ Remix sound        │ +2.0     │ Explicit │ Strong signal  │
  │ Export sound       │ +1.5     │ Implicit │ Quality signal │
  │ Purchase pack      │ +1.0     │ Explicit │ Genre signal   │
  │ Share sound        │ +1.5     │ Explicit │ Social signal  │
  │ Comment on sound   │ +0.5     │ Explicit │ Engagement     │
  │ Extended preview   │ +0.3     │ Implicit │ Interest       │
  │ Quick preview skip │ -0.2     │ Implicit │ Not interested │
  │ Hide sound         │ -1.0     │ Explicit │ Strong neg     │
  │ Report sound       │ -2.0     │ Explicit │ Content issue  │
  │ Follow creator     │ +1.0     │ Explicit │ More from this │
  │ Add to world       │ +1.0     │ Explicit │ Curation value │
  └────────────────────┴──────────┴──────────┴────────────────┘
```

---

## 4. Collaborative Workflows

### Real-Time Co-Creation

```
Two or more producers designing sounds together in real-time.

┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  Session: "Dark Trap Collab"                                       │
│  Participants: @producer_a, @producer_b, @producer_c               │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Shared interface:                                          │   │
│  │                                                             │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐           │   │
│  │  │  A's sound  │  │  B's sound │  │  C's sound │           │   │
│  │  │  ▶ kick_01 │  │  ▶ snare_03│  │  ▶ hat_07  │           │   │
│  │  │  Score: 84 │  │  Score: 79 │  │  Score: 81 │           │   │
│  │  │  [▲ 12]    │  │  [▲ 8]     │  │  [▲ 15]    │           │   │
│  │  └────────────┘  └────────────┘  └────────────┘           │   │
│  │                                                             │   │
│  │  ┌─────────────────────────────────────────────────────┐   │   │
│  │  │  Chat:                                                │   │   │
│  │  │  A: "let's make this punchier"                       │   │   │
│  │  │  B: "try adding saturation"                          │   │   │
│  │  │  C: "sent a version with more body"                  │   │   │
│  │  │  A: "that's it, let's use B's transient + C's body" │   │   │
│  │  └─────────────────────────────────────────────────────┘   │   │
│  │                                                             │   │
│  │  ┌─────────────────────────────────────────────────────┐   │   │
│  │  │  Shared pack: "Dark Trap Collab" (12 sounds so far) │   │   │
│  │  │  Cohesion score: 0.89 (tight cluster)               │   │   │
│  │  │  [Export pack] [Invite more] [End session]          │   │   │
│  │  └─────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  Architecture:                                                     │
│    • WebSocket connection for real-time sync                       │
│    • Each participant generates sounds independently               │
│    • Sounds shared to session via blackboard                       │
│    • Embedding centroid computed from all session sounds           │
│    • All participants see centroid shift as sounds are added       │
│    • Vote system: "best kick" → featured in pack preview          │
│    • Session ends → pack published with all creator credits        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Async Collaboration

```
Not everything needs to be real-time.

Async collaboration workflow:
  1. Creator A publishes a "seed sound" — a starting point
  2. Creator B remixes it: adds processing, changes character
  3. Creator C remixes B's remix: new genre context
  4. All remixes tracked in provenance chain
  5. Original creator notified of each remix
  6. Remix tree grows organically over days/weeks

  "I made a kick yesterday. This morning, 5 people had remixed it
   into different styles. One turned it into a cinematic impact.
   Another made it sound like a 90s house kick. I would never have
   thought of these directions. The community evolved my sound."

  Remix notifications:
    "Your kick DTA_Kick_01 was remixed by @producer_x"
    "Your remix of @producer_y's snare was exported by 12 users"
    "Your sound is trending in the 'Dark Trap' genre this week"

  Remix tree visualization:
    @alice: Punchy_Kick_01
      ├── @bob: Punchy_Kick_01_processed
      │    └── @charlie: Punchy_Kick_01_cinematic
      │         └── @diana: Punchy_Kick_01_orchestral
      └── @eve: Punchy_Kick_01_808_version
           └── @frank: Punchy_Kick_01_808_dark
```

### Collaborative Challenges

```
Time-boxed, theme-based collaborative events.

Challenge format:
  ┌────────────────────────────────────────────────────────────────┐
  │  "Trap Kick Challenge — Week 47"                              │
  │                                                                │
  │  Theme: "Make the punchiest kick you can"                     │
  │  Duration: 48 hours                                            │
  │  Constraint: 140bpm, must use only cShot tools                 │
  │  Prize: Featured on homepage + $50 creator credit              │
  │                                                                │
  │  Participants: 243                                             │
  │  Submissions: 412 kicks (some submitted multiple)              │
  │                                                                │
  │  Winners:                                                      │
  │    🥇 @producer_a — Score: 94 — "aggressive, clean, unique"   │
  │    🥈 @producer_b — Score: 91 — "punchiest attack ever"       │
  │    🥉 @producer_c — Score: 89 — "perfect mix placement"       │
  │                                                                │
  │  All submissions compiled into "Week 47 Challenge Pack"       │
  │  Available in marketplace (proceeds to challenge fund)         │
  └────────────────────────────────────────────────────────────────┘

  Challenge types:
    - Genre-specific: "Best lo-fi snare"
    - Constraint-based: "Make a kick using only 3 parameters"  
    - Mashup: "Combine trap and orchestral drums"
    - Seasonal: "Halloween impact sounds"
    - Collaborative: "Team challenge — 5 people make a pack together"
    - Remix: "Remix this seed sound into something completely different"
```

---

## 5. Identity Systems

### Sonic Identity

```
Your sonic identity = the unique character of the sounds you make.

Components:
  ┌────────────────────────────────────────────────────────────────┐
  │  Sonic Identity Profile:                                      │
  │                                                                │
  │  Signature Embedding: centroid of ALL user's created sounds   │
  │  → "This is what @producer_a sounds like"                     │
  │                                                                │
  │  Genre Tendencies: genre classifier on user's sounds          │
  │  → "Primarily trap (65%), also drill (20%), experimental (15%)│
  │                                                                │
  │  Production Fingerprint: average production_style axis value  │
  │  → "Processed (0.78), saturated (0.65), compressed (0.82)"   │
  │                                                                │
  │  Texture Signature: average texture axis value                │
  │  → "Gritty (0.71), warm (0.68)"                              │
  │                                                                │
  │  Energy Profile: distribution of energy axis across sounds    │
  │  → "Mostly aggressive (0.8), some gentle (0.2)"              │
  │                                                                │
  │  Era Alignment: decade classifier on user's sounds            │
  │  → "Modern (2020s: 60%), vintage (2010s: 30%)"               │
  │                                                                │
  │  Taste Evolution: how identity changed over time              │
  │  → "Started with dark trap, evolved into experimental"        │
  │                                                                │
  │  Visual Identity: generated avatar/logo from sound analysis   │
  │  → Waveform art based on signature sound                      │
  └────────────────────────────────────────────────────────────────┘

Identity can be:
  - Public: "This is who I am as a sound designer"
  - Followable: "I want more sounds like @producer_a makes"
  - Comparable: "Our styles are 72% similar — we should collab"
  - Evident: "Your style has shifted toward cinematic this month"
  - Protectable: "This sound is clearly a @producer_a style sound"
```

### Taste Profiles

```
Taste profile = what you LIKE (not what you MAKE).

  ┌────────────────────────────────────────────────────────────────┐
  │  Taste Profile: @user_beatmaker                               │
  │                                                                │
  │  Taste Embedding: centroid of ALL user's exported/favorited   │
  │  sounds + purchased packs                                      │
  │  → "This is the sound @user_beatmaker prefers"                │
  │                                                                │
  │  Preferred Characters:                                         │
  │    punchy: 0.85 (aggressive kicks)                            │
  │    bright: 0.72 (clear, cutting snares)                       │
  │    short_decay: 0.80 (tight hi-hats)                          │
  │    processed: 0.65 (produced, not raw)                        │
  │                                                                │
  │  Genre Preference Distribution:                                │
  │    trap: 55% | drill: 25% | house: 10% | lo-fi: 10%           │
  │                                                                │
  │  Creator Affinities:                                           │
  │    @producer_a (82% taste match)                              │
  │    @producer_b (76% taste match)                              │
  │    @producer_c (71% taste match)                              │
  │                                                                │
  │  Taste Twins (users with 90%+ similar taste):                 │
  │    @beatmaker_99, @producer_x, @drum_lover                    │
  │    → "Follow their discoveries to find sounds you'll love"    │
  │                                                                │
  │  Taste Evolution:                                              │
  │    6 months ago: 80% trap, now: 55% trap, 25% drill           │
  │    "Your taste is diversifying toward drill"                  │
  └────────────────────────────────────────────────────────────────┘

  Taste following: Follow a taste profile, not just a user.
    "I follow @producer_a's taste because they always find great kicks"
    → You see what THEY export, favorite, and purchase
    → Even if you don't want to make their style of music
```

### Creator Discovery Mechanisms

```
How creators find each other and build audiences:

  1. TASTE MATCHING
     "Your taste is 85% similar to @producer_a"
     → Suggest follows, collaborations, and shared discovery
     → "Users with your taste also follow @producer_b"

  2. REMIX NETWORKS
     Every remix creates a connection between creators.
     "You remixed @producer_c's sound" → notification → follow
     Over time: remix graph = trust graph

  3. CHALLENGE PARTICIPATION
     "You and @producer_d both placed in the Trap Kick Challenge"
     → Shared experience → natural connection

  4. COLLABORATIVE PACKS
     "Join this collaborative pack: Dark Trap Session #4"
     → Work together → build relationship → future collabs

  5. SOUND-TO-CREATOR DISCOVERY
     "I love this kick → who made it?" → follow creator
     Sound attribution creates organic discovery.

  6. GENRE HUBS
     Pages for each genre showing top sounds, packs, and creators.
     "Explore Trap creators" → filter, sort, discover

  7. FEATURED CURATION
     Algorithm-curated "Creators to Watch" lists.
     "Rising talent in Drill" → 10 creators gaining traction

  8. VERIFIED BADGES + REPUTATION
     Trust signals help users find quality creators.
     "Top 10% creator" badge → visible in search results
```

---

## 6. Sound Worlds

### What Is a Sound World?

```
A Sound World is a CURATED COLLECTION of sounds around a theme,
created and maintained by a community.

  Not a playlist. Not a pack. A WORLD.

  World: "Neon Tokyo — Cyberpunk Sound World"
  
  Content:
    - 200+ sounds contributed by 30+ creators
    - Kicks, snares, hats, 808s, FX, textures
    - All sharing a cyberpunk/dark synth aesthetic
    - Cohesion enforced by embedding centroid ("Neon Tokyo")
  
  Structure:
    ┌─────────────────────────────────────────────────────────┐
    │  World: Neon Tokyo                                     │
    │  Theme: Cyberpunk drum sounds, dark synths, glitch FX  │
    │  Creators: 30                                           │
    │  Sounds: 200+                                           │
    │  Followers: 12,000                                     │
    │  Monthly active: 3,000 (downloading from world)        │
    │                                                          │
    │  Channels:                                               │
    │  ├── Kicks (45 sounds)                                 │
    │  ├── Snares (30 sounds)                                │
    │  ├── Hi-hats (40 sounds)                               │
    │  ├── 808s/Bass (25 sounds)                             │
    │  ├── Percussion (20 sounds)                            │
    │  ├── FX (30 sounds)                                    │
    │  └── Textures (15 sounds)                              │
    │                                                          │
    │  Featured: "Kick of the Week" — community voted        │
    │  Trending: "Glitch FX pack" — most downloaded this week│
    │  Fresh: "Ambient textures vol 2" — newly added         │
    └─────────────────────────────────────────────────────────┘
```

### Sound World Mechanics

```
Creating a World:
  1. Creator proposes theme + embedding centroid
  2. Minimum 10 seed sounds to establish character
  3. World goes public when 20+ sounds from 5+ creators
  4. World has moderators (original creator + appointed)
  5. World has guidelines (what sounds belong, quality bar)

Contributing to a World:
  1. Creator makes a sound similar to world's centroid
  2. Sound proposed to world (embedding check: must be < 0.3 from centroid)
  3. Moderator approves or rejects (or auto-approve for trusted contributors)
  4. If approved: sound added to world, creator attributed
  5. If rejected: suggestion to try different direction

World Evolution:
  - Centroid shifts slowly as new sounds are added
  - "Neon Tokyo 2023" → "Neon Tokyo 2024" (centroid evolved)
  - Worlds can fork: "Neon Tokyo" → "Neon Tokyo (Dark)"
  - Worlds can merge: "Neon Tokyo" + "Cyber Drums" → "Neon Cyber"

World Benefits:
  - Discoverability: contributors get exposure from world's audience
  - Cohesion: world curators ensure consistent quality
  - Community: shared creative identity around a theme
  - Context: sounds in a world tell a story together
  - Monetization: world can publish packs (revenue split among contributors)
```

---

## 7. Feed Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Feed Generation Pipeline                                     │  │
│  │                                                               │  │
│  │  1. Candidate Selection (pre-computed per user)               │  │
│  │     • Following: last 100 sounds from followed creators       │  │
│  │     • Taste: top 100 sounds by taste match (fresh < 7d)      │  │
│  │     • Trending: top 50 sounds by velocity (rolling window)    │  │
│  │     • Collaborative: top 50 sounds from taste twins           │  │
│  │     • Remix: last 50 remixes of user's/followed sounds        │  │
│  │     → ~350 candidates                                         │  │
│  │                                                               │  │
│  │  2. Deduplication (remove exact duplicates)                   │  │
│  │  3. Hard filtering (remove hidden/reported/low-quality)       │  │
│  │  4. Scoring (weighted features per user)                      │  │
│  │  5. Diversification (MMR with λ=0.3)                          │  │
│  │  6. Ranking → top 50 for display                              │  │
│  │  7. Caching (5-minute TTL, invalidated on new interaction)    │  │
│  │                                                               │  │
│  │  Total pipeline: ~100ms per user at 50K DAU                   │  │
│  │  Cached for 5 min → 50K feeds × 100ms = 5000s →              │  │
│  │  Need 17 concurrent workers for real-time refresh             │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Real-Time Updates                                            │  │
│  │                                                               │  │
│  │  When user interacts (like, export, follow):                 │  │
│  │    1. Update user's interaction graph (Neo4j)                 │  │
│  │    2. Update user's taste embedding (vector DB)              │  │
│  │    3. Invalidate feed cache for that user                     │  │
│  │    4. Re-compute feed (async, 500ms to regenerate)            │  │
│  │    5. Push update via WebSocket if user is active             │  │
│  │                                                               │  │
│  │  When a followed creator publishes:                           │  │
│  │    1. Insert sound into feed queue                            │  │
│  │    2. Push notification to followers: "X posted a new sound"  │  │
│  │    3. Add to followers' feed caches precomputed superset      │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 8. Implementation Roadmap

```
Phase 1 — Social Foundation (2 months):
  ✓ User profiles + follow system
  ✓ Sound attribution (creator stamped on every sound)
  ✓ Remix tracking (chain of custody)
  ✓ Basic activity feed (following only)
  ✓ Like/favorite sounds

Phase 2 — Discovery (1 month):
  ✓ Taste embedding computation per user
  ✓ Discovery feed (taste-based)
  ✓ Trending feed (engagement velocity)
  ✓ Creator discovery ("who made this?")
  ✓ SoundWorld creation + browsing

Phase 3 — Collaboration (2 months):
  ✓ Collaborative pack creation
  ✓ Real-time co-creation sessions
  ✓ Remix tree visualization
  ✓ Challenges system
  ✓ Chat and commenting on sounds

Phase 4 — Identity (1 month):
  ✓ Sonic identity profiles
  ✓ Taste profiles (public/followable)
  ✓ Taste matching ("your taste is X% similar to Y")
  ✓ Creator discovery mechanisms
  ✓ Visual identity generation

Phase 5 — Community Evolution (ongoing):
  ✓ Community guidelines + moderation
  ✓ Advanced feed personalization
  ✓ World growth mechanics
  ✓ Creator analytics (audience insights)
  ✓ Monetization (creators earn from world contributions)
  ✓ API for third-party social integrations

Total timeline: ~6 months to full sonic social network
```

---

## 9. Summary

```
Sonic Social Networks

  Core insight:
    Sound creation is culturally social but technically solitary.
    A social layer for SOUND MAKING (not just track sharing) unlocks
    entirely new creative workflows.

  Key concepts:
    - Sound attribution: every sound has a creator
    - Remix lineage: sounds evolve through the community
    - Taste profiles: follow what people like, not just who they are
    - Sound Worlds: community-curated theme collections
    - Co-creation: real-time and async collaborative sound design

  Social graph:
    Users ↔ Sounds ↔ Packs ↔ Worlds
    Edges: follow, created, remixed, favorited, exported
    Queries: feed generation, taste matching, remix trees, collab discovery

  Feed algorithms:
    6 feed types (following, discovery, trending, collaborative, remix, world)
    Score = weighted features (taste match, recency, quality, social proof)
    Diversity enforced via MMR
    User-adaptive weights re-calibrated weekly

  Identity:
    Sonic identity = what you MAKE
    Taste profile = what you LIKE
    Both are followable, comparable, and evolvable

  The sonic social network transforms cShot from a tool into a community.
  Sounds become social objects. Creation becomes collaborative.
  The platform becomes the place where sonic identity lives.
```

