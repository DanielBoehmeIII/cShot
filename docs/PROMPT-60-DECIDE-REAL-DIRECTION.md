# Prompt 60 — Decide the Real Direction of cShot

Based on everything learned across 60 prompts of design, research, and planning, decide what cShot should become.

---

## 1. Possible Directions

### Direction A: AI One-Shot Generator

**What it is:** The current vision. Type a prompt, get a unique one-shot. Generate, preview, export. Single-sound focus.

**Upside:**
- Narrow, focused, shippable
- Core premise validated in alpha (kicks are magical)
- Minimal UX — one screen, one flow
- Easy to explain: "Type what you want, get the sound"

**Risks:**
- Quality variance undermines trust
- Model dependency (ElevenLabs API controls quality ceiling)
- Hard to differentiate long-term — many competitors emerging
- Low switching cost for users (they can try any generator)

**Technical Difficulty:** Medium (DSP + API integration)

**Market Potential:**
- TAM: ~$200M (sample market) but as a tool, not a marketplace
- Revenue: $10-20/user/month subscription
- Defensibility: Low (model access is the moat, and it's rented)

**Creative Importance:** Medium. Useful tool. Replaces sample browsing, not sound design.

**Defensibility:** Low. Model access is rented. Competitors can match quality. No network effects. No data moat at alpha scale.

---

### Direction B: AI Sample-Pack Builder

**What it is:** Generate cohesive sample packs from a single prompt. 10-20 sounds, organized by type, normalized, named, and export-ready.

**Upside:**
- Higher value per generation (one prompt = 10+ sounds)
- Pack cohesion is a technical moat (hard to do well)
- Higher willingness to pay (packs are $10-50 vs single sounds)
- Alignment with alpha insight: users want generators, not libraries
- B2B potential: sell packs to Splice, Loopmasters, producers

**Risks:**
- More complex UX
- Quality variance amplified across 10+ sounds
- Pack market is crowded — needs to be clearly better
- Batch generation cost (API) is 10x higher

**Technical Difficulty:** High (clustering, cohesion metrics, batch processing, quality filtering)

**Market Potential:**
- TAM: $500M+ (sample pack market)
- Revenue: $20-50/pack or subscription for unlimited packs
- Defensibility: Medium-High (cohesion algorithms, quality filter, generation pipeline)

**Creative Importance:** High. Actually changes how sample packs are made.

**Defensibility:** Medium. The embedding clustering + quality pipeline is hard to replicate, but not impossible.

---

### Direction C: Semantic Sound Browser

**What it is:** Index and search the user's existing sample library using natural language. "Find the kick that sounds punchy and dark" instead of scrolling through 5000 samples.

**Upside:**
- Solves a real pain point (sample library management)
- Works with user's existing content — no generation needed initially
- Model requirements are lighter (CLAP embeddings, not generation)
- Defensible: embedding index grows with user's library (data moat)
- Can add generation as a second feature

**Risks:**
- Not a new product category — Splice already does this
- Requires importing user's sample library (trust barrier)
- Technical complexity of library scanning + indexing
- Users may not want another library manager (they have Finder/Explorer)

**Technical Difficulty:** Medium (CLAP embeddings, file scanning, SQLite FTS, UI)

**Market Potential:**
- TAM: Same as sample market (~$200M)
- Revenue: Hard to monetize alone — feature, not product
- Defensibility: Medium (user's indexed library is a switching cost)

**Creative Importance:** Medium. Makes existing workflows faster but doesn't enable new creativity.

**Defensibility:** Medium. User's indexed library is a mild switching cost. Embedding models are commoditizing.

---

### Direction D: DAW-Native Sound Designer

**What it is:** A VST3/AU plugin that lives inside the user's DAW. Generate, preview, and drag onto tracks without leaving the DAW.

**Upside:**
- Zero workflow friction (no export/import)
- Higher willingness to pay (producers pay for plugins)
- Plugin distribution is well-understood
- Can integrate with DAW session context (BPM, key, arrangement)

**Risks:**
- Vastly more complex technically (Tauri → VST bridge)
- DAW compatibility matrix (Live, FL Studio, Logic, Pro Tools, etc.)
- Plugin development is a different skill set
- Distribution requires signing, installer, sometimes approval

**Technical Difficulty:** Very High (plugin framework, DAW APIs, real-time audio, UI framework in plugin context)

**Market Potential:**
- TAM: ~$5B (plugin market)
- Revenue: $50-200/plugin or subscription
- Defensibility: Medium (plugin distribution has some moat)

**Creative Importance:** High. DAW-native generation would be genuinely new.

**Defensibility:** Medium. Plugin distribution moat is real but competitors would build plugins too.

---

### Direction E: Personal Sonic Identity Engine

**What it is:** Learn each user's sonic preferences and generate sounds that match their personal style. The model fine-tunes on what each user keeps and exports.

**Upside:**
- Deeply personal, high switching cost
- Gets better the more you use it
- Strongest possible data moat
- High emotional connection: "cShot knows my sound"

**Risks:**
- Requires user accounts and centralized learning
- Privacy concerns with audio preferences
- Slow to demonstrate value (needs many sessions to learn)
- Cold start problem — bad until enough data

**Technical Difficulty:** Very High (fine-tuning per user, preference learning, privacy architecture)

**Market Potential:**
- TAM: Depends on delivery mechanism (app, plugin, or service)
- Revenue: Premium subscription for personal model
- Defensibility: Very High (personalized model is a true moat)

**Creative Importance:** Very High. Personal sonic identity is a new concept.

**Defensibility:** Very High. A model that knows your taste better than you do is hard to leave. This is the defensibility play.

---

### Direction F: Generative Sound Laboratory

**What it is:** An experimental playground for sound design. Morph, crossbreed, randomize, and explore. Not for quick results but for discovery and inspiration.

**Upside:**
- High creative value
- Morphing + high-level controls (Prompt 57, 58) are natural here
- Attracts power users and sound designers
- Can be a lead-in to a more focused product

**Risks:**
- Hard to monetize (experimental tools don't sell well)
- High complexity — morphing, controls, real-time processing
- Users don't know what they want when they open it
- Competition from existing tools (Reaktor, Max/MSP, VCV Rack)

**Technical Difficulty:** Very High (morphing, real-time DSP, model integration, latency management)

**Market Potential:**
- TAM: Niche (experimental sound design)
- Revenue: Hard — one-time license at best
- Defensibility: Medium (unique combination, but each feature exists elsewhere)

**Creative Importance:** Very High. This is the most creatively valuable direction.

**Defensibility:** Low-Medium. Each feature exists in isolation elsewhere. The combination is unique but not a moat.

---

### Direction G: Marketplace for AI-Generated Sounds

**What it is:** A platform where users generate and sell sounds. cShot provides the generation tools, marketplace infrastructure, curation, and licensing.

**Upside:**
- Network effects: more creators → more buyers → more value
- Highest revenue potential (marketplaces take 30-50%)
- cShot is both the tool and the distribution channel
- First-mover advantage in AI sample marketplaces

**Risks:**
- Requires critical mass on both sides (chicken-and-egg problem)
- Legal uncertainty around AI-generated audio copyright
- Content moderation at scale
- Competitors: Splice, Loopmasters already have distribution
- Most complex product to build
- Quality control across thousands of creators

**Technical Difficulty:** Extremely High (marketplace infra + generation + content moderation + licensing + payments)

**Market Potential:**
- TAM: $1B+ (sample marketplace)
- Revenue: 30-50% commission on sales
- Defensibility: High (network effects + curation + creator retention)

**Creative Importance:** Medium. Enables commerce more than creativity.

**Defensibility:** High. Marketplace network effects are the strongest moat — if you have them.

---

### Direction H: Local-First Sound Model Platform

**What it is:** All generation runs on the user's machine. No cloud dependency, no API costs, no latency variance. Privacy-first, offline-capable.

**Upside:**
- Zero ongoing API costs
- No latency variance (predictable local inference)
- Privacy sells (no audio leaves the machine)
- Works offline
- Technical moat: optimizing models for local inference is hard

**Risks:**
- Requires building/distilling own model (very hard)
- Local model quality may lag behind cloud models
- Large model downloads (1-5GB) for first use
- Requires GPU or Apple Silicon for acceptable speed
- ONNX/ML inference on desktop is immature

**Technical Difficulty:** Extremely High (model distillation, ONNX runtime, optimization, cross-platform ML)

**Market Potential:**
- TAM: Same as generator ($200M) but as premium product
- Revenue: Premium pricing ($20-40/month) — no API costs means better margins
- Defensibility: High (local model optimization is hard)

**Creative Importance:** Same as generation (Direction A)

**Defensibility:** High. Local model optimization + privacy are genuine moats. But only if the model quality is competitive.

---

## 2. Comparison Matrix

| Direction | Upside | Risk | Tech Diff | Market | Creative | Defensibility | Our Fit |
|-----------|--------|------|-----------|--------|----------|---------------|---------|
| A: One-shot gen | Medium | High | Medium | $200M | Medium | Low | We're here now |
| B: Pack builder | High | Medium | High | $500M | High | Medium | Natural evolution |
| C: Sound browser | Medium | Medium | Medium | $200M | Medium | Medium | Adjacent |
| D: DAW plugin | High | High | Very High | $5B | High | Medium | Future play |
| E: Personal engine | Very High | Very High | Very High | ? | Very High | Very High | Long-term play |
| F: Sound lab | Medium | High | Very High | Niche | Very High | Low | Side project risk |
| G: Marketplace | Very High | Very High | Extreme | $1B+ | Medium | High | Too early |
| H: Local-first | High | High | Extreme | $200M | Medium | High | Technical bet |

---

## 3. Recommended Direction: B + A → D → E (Phased)

cShot should evolve through four phases, each building on the last:

```
Phase 1 (Now — 3 months): Premium One-Shot Generator (A)
  - Focus: Become the best kick and bass generator
  - Ship: Repair chain, SoundScore, high-level controls
  - Target: Individual producers who need instant sounds
  - Revenue: $15/month subscription

Phase 2 (3-6 months): Pack Builder (B)
  - Focus: From single sounds to cohesive packs
  - Ship: Pack generator, cohesion metrics, batch export
  - Target: Producers who want complete kits
  - Revenue: $25/month (includes Phase 1) or $10/pack

Phase 3 (6-12 months): DAW Plugin (D)
  - Focus: Generate inside the DAW
  - Ship: VST3/AU plugin, DAW context awareness
  - Target: Power users who want zero friction
  - Revenue: $50-100 one-time plugin + subscription

Phase 4 (12-18 months): Personal Sonic Identity (E)
  - Focus: Learn each user's taste
  - Ship: Preference learning, personalized fine-tuning
  - Target: Loyal users who want cShot to know them
  - Revenue: $30-50/month for personalized model
```

### What We Build Now

```typescript
const NOW = {
  product: 'cShot — instant one-shot generator',
  focus: 'kicks and bass first',
  users: 'individual producers, beatmakers',
  price: '$15/month or $150/year',
  platform: 'Tauri + React + Rust (desktop app)',
  model: 'ElevenLabs SFX API + DSP repair chain',
  key_features: [
    'Single-screen generation UI',
    'Prompt suggestion chips',
    'High-level sound controls (punch, body, weight, snap, air)',
    'Auto-repair chain (trim, normalize, EQ)',
    'SoundScore (quality ranking)',
    'Reference upload (drag-drop your track)',
    'One-click WAV export',
    'Usage tracking + improvement loop',
  ],
  not_building: [
    'Library/browser',     // users don't want it
    'Marketplace',         // too early, too complex
    'Social features',     // not needed for alpha
    'Collaboration',       // not needed yet
    'Mobile',              // desktop only
    'Plugin',              // Phase 3
    'Personalization',     // Phase 4
    'Teacher/forking',     // nice-to-have, defer
  ],
};
```

### What We Cut (For Now or Forever)

| Feature | Cut Reason | Cut Level |
|---------|-----------|-----------|
| Library browser | Users treat cShot as faucet, not collection | Forever |
| Social sharing | No demand in alpha | Forever |
| Mobile app | Desktop is the right platform for audio production | Forever |
| Text-to-music (full songs) | Out of scope — one-shots only | Forever |
| Stem separation | Different product | Forever |
| Real-time generation | Latency too high for real-time | Forever |
| VST3 plugin | Phase 3 — too early now | Phase 3 |
| Personal fine-tuning | Phase 4 — data requirements too high now | Phase 4 |
| Marketplace | Phase 5+ if at all | Phase 5+ |
| Multi-language prompts | English first, prove product before localizing | Phase 5+ |
| Web version | Desktop audio is the right experience | Phase 5+ |

---

## 4. Why This Direction Wins

### It matches what alpha testers actually loved

Testers didn't ask for a library. They didn't ask for social features. They didn't ask for a plugin (most of them). They asked for:
- "Make the kicks even better" ← Direction A strength
- "Make packs from my favorites" ← Direction B
- "I wish this was in my DAW" ← Future Direction D
- "How does it know what I like?" ← Future Direction E

The phased approach lets us deliver on each request at the right time.

### It builds a defensibility progression

```
Phase 1: Product quality moat (best kick generator)
    ↓
Phase 2: Pipeline moat (cohesion algorithms, batch generation)
    ↓
Phase 3: Distribution moat (DAW plugin install base)
    ↓
Phase 4: Data moat (personalized model, switching cost)
```

Each phase creates a new defensibility layer. After Phase 4, leaving cShot means losing a model that knows your taste — a huge switching cost.

### It generates revenue early

Phase 1 subscription covers the API costs and funds development. Phase 2 increases ARPU (packs are higher value). Phase 3 plugin is a new revenue stream. Phase 4 premium tier is the highest margin.

### It's buildable by a small team

| Phase | Team Size | Time | Key Hires |
|-------|-----------|------|-----------|
| 1 | 1-2 devs | 3 months | Rust + React dev |
| 2 | 2-3 devs | 3 months | Audio DSP engineer |
| 3 | 3-4 devs | 6 months | Plugin dev + QA |
| 4 | 4-5 devs | 6 months | ML engineer + backend |

---

## 5. What cShot Should NOT Become

### Do NOT become a Splice clone

The sample library market is owned by Splice. cShot should not try to be "Splice but AI." It should be a generation tool that complements Splice.

### Do NOT become a general AI audio tool

No stem separation, no text-to-music, no mastering. Focus on one-shots. The narrower the focus, the better the product. "Everything for everyone" is the death of small products.

### Do NOT become a platform too early

No marketplace, no third-party models, no plugin SDK. These add complexity without proving the core value. Prove the generator first, then expand.

### Do NOT chase enterprise

Enterprise sales cycles kill small products. Sell to individual producers who make decisions and pay with their own credit cards. Enterprise can come later.

---

## 6. The One-Sentence Product Vision

> **cShot is the fastest way to get a usable drum sound from an idea.**

Not "AI-powered generative audio platform." Not "the future of sound design." Just: you have an idea, you type it, you get the sound, you use it. Everything else is implementation.

---

## 7. What to Ship Tomorrow

```
Open cShot → Type "punchy kick 140bpm" → Press Enter
→ 3 seconds later, hear a kick that makes you smile
→ Click export → Drag WAV into Ableton
→ It fits. No EQ needed. No trim needed.
→ You make music.

That's the product. That's always been the product.
Everything in these 60 prompts serves that 10-second loop.
```

---

## 8. Final Summary

cShot should be the best kick and bass generator, then the best pack builder, then a DAW plugin, then a personal sonic identity engine. Ship Phase 1 in 3 months. Cut everything that doesn't serve the 10-second generation loop. The alpha proved the core premise works. Now execute.

These 60 prompts have designed every layer of the product:
- **Prompts 11-14:** Psychoacoustics and perception
- **Prompts 15-22:** AI architecture and datasets
- **Prompts 23-30:** Taste modeling and vision
- **Prompts 31-38:** DAW integration, UI, mix readiness
- **Prompts 39-42:** MVP spec, build plan, stack, scope
- **Prompts 43-50:** Audio tools, preview, library, tagging, alpha flow, checklist
- **Prompts 51-54:** Feedback systems, failure taxonomy, repair chain, SoundScore
- **Prompts 55-56:** Pack builder, cohesion metrics
- **Prompts 57-58:** High-level controls, sound morphing
- **Prompts 59-60:** Alpha postmortem, strategic direction

Everything is designed. It's time to build.
