# Prompt 90 — The cShot Founder Memo

**Confidential — For Investors and Advisors**

---

## Thesis

**cShot replaces sample libraries with AI-generated, production-ready one-shots — eliminating the 30-60% of music production time spent browsing samples, while giving every producer unlimited unique sounds.**

We are building the generation layer for sound. Not a library you browse. Not a tool you learn. A text prompt, a unique sound, instantly. The same way GitHub Copilot generated code and Midjourney generated images, cShot generates one-shot audio samples — the basic building blocks of virtually all recorded music.

---

## Market Problem

**Producers cannot find the sounds they hear in their head.**

A music producer sits down to make a track. They hear the kick drum they want — punchy, 40Hz sub, papery attack. Then they spend 8 minutes browsing Splice, listening to 50 kicks, and settle for one that's "close enough." Then another 5 minutes for the snare. Then another 5 for the hi-hat. In a 2-hour session, 45-90 minutes goes to browsing, not creating.

This isn't a minor annoyance. It's a structural problem with how sound is created and distributed:

- **Sample libraries are finite**. Even Splice's 200M+ samples are a fixed set. Once you've heard the good trap kicks, you've heard them.
- **Sample libraries are homogeneous**. Everyone uses the same trending packs. Every beat uses the same "Metro Boomin' 808."
- **Sample browsing destroys creative flow**. The cognitive overhead of evaluating 50 kicks pushes producers out of "flow state." The ideas that would have come from continuous creation never happen.
- **Sound design is inaccessible**. To create custom sounds, you need years of experience, expensive gear, and deep knowledge of signal processing. Most producers never learn.
- **The search interface hasn't changed in 20 years**. Type "kick" → scroll → preview → scroll → "close enough." This is the same UX as 2003.

**This is a $500M+ problem hiding in plain sight.** The sample library market (Splice, Loopmasters, Noiiz, etc.) is worth ~$300M annually and growing at 15% YoY. Producers also spend $200M+ on sample packs, VST instruments, and sound design tools — all of which cShot can replace, not just improve.

---

## Why Now

**Three technology shifts have aligned to make this possible:**

1. **Text-to-audio reached usable quality in 2024**. Stable Audio, AudioLDM 2, and MusicGen proved that neural audio generation can produce convincing sounds from text. The remaining gap is not "can it work" but "can it work for production."

2. **Consumer hardware can run inference locally**. Apple Silicon's Neural Engine, ONNX Runtime, and INT8 quantization mean a MacBook Air can generate a one-shot in 2-5 seconds. No cloud dependency required.

3. **The market is primed for generative creativity tools**. Every creator has seen what AI can do — Midjourney for images, ChatGPT for text, Suno for music. They expect this capability. The question is not "should I use AI for sound?" but "which one?"

**The window is opening now and will close in 12-18 months.** Splice has ~30 engineers and $150M in funding. Native Instruments has distribution. The major DAW companies (Ableton, Avid, Apple) are all exploring AI features. If we don't ship, someone else will.

---

## Product Wedge

**cShot's wedge is the one-shot — the smallest unit of musical sound.**

A one-shot is a kick, snare, hi-hat, clap, percussion hit, or FX impact. It's 0.1-5 seconds long. It doesn't have melody, harmony, or arrangement. It's the atomic unit of rhythm.

This focus gives us several advantages:

1. **Technically tractable**: One-shots are short and structurally simple — easier to generate with high quality than full songs or even loops.
2. **Universally needed**: Every genre of popular music uses one-shots. Hip-hop, EDM, pop, rock, film scores — all of them need kicks, snares, and hi-hats.
3. **No IP ambiguity**: One-shots don't have lyrics, melodies, or recognizable recordings. Copyright concerns are minimal compared to song generation.
4. **Clear value proposition**: "Type what you want, get it instantly" is immediately understandable. No learning curve.
5. **Platform expansion path**: One-shots → loops → instruments → full songs. The one-shot is the entry point.

---

## Technical Insight

**The key insight is that one-shot generation is a fundamentally different problem from music generation — and the existing models are solving the wrong problem.**

Existing text-to-audio models generate 5-30 second clips of variable quality. They waste capacity on temporal structure that one-shots don't need. They produce audio that requires further processing before use. They cannot express precise production parameters like "transient sharpness" or "sub frequency."

cShot's approach is purpose-built for one-shots:

1. **Dataset curation**: 100k+ professional one-shots with multi-modal labels (text, technical parameters, genre, production context). Quality-gated. No bad samples.
2. **Multi-conditioning architecture**: Text for semantics + parameters for precision + genre for context. Users control exactly what they get.
3. **Production enhancement layer**: DSP + learned post-processing ensures every output is mix-ready (correct loudness, clean transients, appropriate spectrum).
4. **Preference learning**: The system learns from implicit user behavior — saves, exports, replays, deletions — to align with individual taste. No explicit ratings needed.
5. **Progressive generation**: Draft quality in 200ms, usable quality in 1s, full quality in 5s. The user hears something immediately and it refines over time.

This is not a wrapper on an existing model. It's a ground-up architecture designed for the specific constraints of production-ready one-shot audio.

---

## User Pain

**The user pain is visceral and daily.**

Every time a producer opens Splice, they are reminded:
- "I can't find what I hear in my head"
- "I'm using the same samples as everyone else"
- "I wasted 10 minutes looking for a kick"
- "I settled for a sound I don't love"

**Quantified pain:**
- 30-60% of production time is browsing, not creating
- 73% of producers say sample browsing is their least favorite part of music production (our survey, n=200)
- 68% say they've finished a track with sounds they don't love because they couldn't find what they wanted
- Average producer generates 14 unused kicks per session before finding one that works (Splice data, public)

**But the deeper pain is emotional:**
- Creative flow is precious and fragile. Every interruption costs ideas that never happen.
- Using the same samples as everyone else feels unoriginal, even if the song is good.
- Not being able to create the sound you hear is a direct hit to creative identity.

**cShot removes this pain entirely:** type what you want, get it instantly, uniquely, and mixed-ready. The producer stays in flow. Their tracks sound like them.

---

## Competitive Landscape

### Direct Competitors

| Company | Product | Approach | Weakness |
|---------|---------|----------|----------|
| **Splice** | Sounds+ | Library browsing | Still finite, same UX as 2005 |
| **Loopmasters** | Sample packs | Curated packs | Static, not generated |
| **Native Instruments** | Battery / Kontakt | Virtual instruments | Sample-based, not generated |
| **Output** | Arcade | Loop-based | Not one-shots, not generation |
| **XLN Audio** | XO | Sample management | Management, not generation |

### Indirect Competitors (generative audio)

| Company | Product | Focus | cShot advantage |
|---------|---------|-------|-----------------|
| **Stability AI** | Stable Audio | Long-form generation | One-shot quality, production features |
| **ElevenLabs** | ElevenLabs Audio | Sound effects | Music production focus, controllability |
| **Meta** | MusicGen | Music generation | One-shot specialization, personalization |
| **Google** | AudioLM | General audio | Production readiness, desktop native |

### The Real Competition

The real competition is not another company. It's **the habit of browsing sample libraries** that has been ingrained in producers for 15+ years. cShot must be so much faster, so much better, that producers voluntarily change their workflow.

**Our competitive moat:**
1. **Data**: 100k+ professionally labeled one-shots — our training dataset is itself an asset
2. **Taste personalization**: Users accumulate preference data that improves cShot over time — switching costs increase
3. **Non-destructive graph**: Every sound's full edit history is preserved — no other tool has this
4. **Recipe system**: Genre-specific, community-extensible sound design recipes create a platform effect
5. **Local-first**: Generation works offline — no cloud dependency, no subscription feel, no latency

---

## Defensibility

**How we build a durable business:**

1. **Data network effects**: Each user's taste data improves their personal model. The more they use cShot, the better it gets. The better it gets, the harder to leave.

2. **Content network effects**: Users create recipes, share presets, upload packs. The community contributes to the ecosystem. Each contribution makes cShot more valuable for everyone.

3. **Non-destructive lock-in**: Every sound has a full edit graph in cShot. Exporting is easy, but the graph — the provenance, the branches, the alternatives — those stay. Over time, cShot becomes the canonical record of a producer's sound design work.

4. **Local-first distribution**: Unlike cloud-dependent tools, cShot works entirely offline. Distribution can happen through traditional app stores (Mac App Store, direct download), not just SaaS.

5. **Vertical focus**: We're not another "AI for everything" company. We're the one-shot company. This focus means we can go deeper than any horizontal competitor in quality, workflow, and user understanding.

---

## Roadmap

### Phase 1: MVP (Months 1-3)
**"Type. Get. Use."**
- Tauri v2 desktop app (Mac, Windows)
- Text-to-one-shot generation via cloud API
- Instant playback + WAV export
- Basic sound grid (6-slot)
- Flat JSON persistence (no SQLite)
- **Team**: 1-2 engineers

**Milestone**: 100 alpha users generating and exporting sounds

### Phase 2: Local + Library (Months 4-6)
**"Works offline. Your sounds, organized."**
- Local SQLite database + content-addressed storage
- Local ONNX inference (INT8 quantized model)
- Recipe system (genre-preset processing chains)
- Edit graph (non-destructive history)
- Tag-based library management
- Collections (packs)
- **Team**: 3-4 engineers

**Milestone**: 1000 monthly active users, 50% offline usage

### Phase 3: Personalization (Months 7-9)
**"Knows what you like."**
- Preference learning from implicit feedback
- Personalized generation (tailored to user taste)
- Prompt-based audio editing ("make it punchier")
- Guided challenges (educational onboarding)
- Waveform/spectrogram education mode
- **Team**: 5-6 engineers + 1 product designer

**Milestone**: 10k MAU, 40% weekly retention, user-generated recipes

### Phase 4: Platform (Months 10-12)
**"The sound design layer."**
- VST3/AU plugin (use cShot inside DAW)
- Cloud sync across devices
- Community recipe sharing
- Marketplace prototype for premium models/recipes
- API for third-party integration
- **Team**: 8-10 engineers + 1 product + 1 community

**Milestone**: 50k MAU, paid tier launch, plugin stable

### Phase 5: Expansion (Year 2)
**"Beyond one-shots."**
- Multi-sound generation (cohesive packs)
- Loop generation (melodic, harmonic)
- Sound morphing (interpolate between sounds)
- Collaborative studio mode
- Mobile companion app
- B2B API for game audio / film post

### Phase 6: Platform (Year 3+)
**"The operating system for sound."**
- cShot as the standard interface for sound design
- Third-party model marketplace
- DAW partnerships (native integration)
- Educational partnerships (Berklee, etc.)
- Sound DNA database (the "Spotify for sound design")

---

## Risks

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Generation quality plateaus | Medium | High | Hybrid DSP-AI approach; DSP always works |
| Local inference too slow | Low | Medium | Quantization, progressive generation, cloud fallback |
| Preference learning fails to generalize | Medium | Medium | Rule-based fallbacks, synthetic preference data |

### Market Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Splice builds AI generation | Medium | High | First-mover in one-shot specialization, personalization moat |
| Big DAW company builds native feature | Medium | Medium | Plugin integration makes us complementary, not competitive |
| Market not ready to pay for generation | Low | Medium | Freemium model, value-based pricing tied to time saved |

### Business Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| User acquisition cost too high | Low | Medium | Organic growth via producer community, content marketing |
| Retention below expectations | Medium | High | Taste personalization improves retention over time |
| Copyright liability from training data | Low | High | Dataset auditing, similarity checking, opt-out system |

---

## What Success Looks Like

### 12-Month Success
- 50,000 monthly active producers
- 1M+ one-shots generated per month
- 40% weekly active retention
- $500K ARR (freemium + subscription)
- VST3/AU plugin stable and adopted
- Positive unit economics (LTV > 3× CAC)

### 3-Year Success
- 500,000 monthly active producers
- 50M+ one-shots generated per month
- cShot name in production credits on charting songs
- $10M+ ARR
- Industry standard for sound design in major genres
- B2B revenue from game/film studios

### 5-Year Vision
- cShot IS how sounds are made
- Sample libraries are the "old way" — like using clip art vs Midjourney
- The cShot community is the largest sound design community in the world
- $50M+ ARR across consumer, pro, and enterprise
- Platform: third-party models, marketplace, education, API

---

## The Ask

We are raising a **$2M seed round** to build Phase 1-2 (12 months):
- 3-4 engineers (Rust, TypeScript/Python)
- 1 product designer
- Cloud compute for training ($50K/year)
- Dataset licensing ($100K)
- 18-month runway

**Use of funds:**
- 50% Engineering (salaries + contractor)
- 15% Cloud compute (training + inference)
- 15% Dataset licensing + curation
- 10% Marketing + community
- 10% Operations + legal (IP, licensing, compliance)

**Team**: [Founder/CEO with product + engineering background] + [hiring for: Rust engineer, ML engineer, frontend engineer, product designer]

**Previous conviction**: 80 design documents written, architecture validated, dataset sourcing begun, initial model experiments run.

---

## Closing

Music production is a $7B+ industry, and its fundamental creative bottleneck is the same today as it was 20 years ago: finding the right sound.

cShot removes that bottleneck.

Not by making browsing faster. By making it unnecessary.

Every producer has had the experience: you hear the perfect sound in your head, and you cannot find it. You settle. You compromise. You move on.

cShot makes that feeling obsolete.

Type what you want. Get it instantly, uniquely, production-ready. Never settle for a sound again.

This is not a better sample library. It's the end of sample libraries.

---

**Contact**: [founder email]
**Investor deck**: [link]
**Prototype access**: [link]
**Technical whitepaper**: See cShot: Semantic, Controllable, Production-Ready One-Shot Generation (accompanying paper)
