# cShot — Roadmap to Profit (Weeks 34–40)

## Week 34 — Beta User Workflow
- Onboarding flow: choose genre → generate → favorite → export (via `cshot make` or UI)
- Example project: `./cshot genre trap --count 50 --out Packs/trap_starter`

## Week 35 — Commercial Pack Templates
Pre-configured pack templates ready in `gen/pack.py` (`THEME_PLANS`):
- `./cshot theme "Noir Piano Kit"` — 50 files
- `./cshot theme "Trap God Kit"` — 50 files
- `./cshot theme "Cinematic Impacts"` — 50 files
- `./cshot theme "Hyperpop Synth Pack"` — 50 files
- `./cshot theme "Lo-fi Keys Pack"` — 50 files

Generate each, rate favorites, export as a pack.

## Week 36 — Licensing / Source Tracking
- All generated sounds are original synthesis (no sample-based generation)
- Metadata sidecar tracks: prompt, seed, generation timestamp
- `cshot make` pipeline auto-generates full tracking
- No reference audio is used for direct copying — only for statistical profile analysis

## Week 37 — Landing Page + Demo Assets
1. Generate 3 demo packs using `cshot theme` + `cshot make`
2. Pick top 10 sounds from each
3. Record 30-second before/after refinement demo using `cshot refine-feedback`
4. Write landing page copy explaining:
   - What it is: AI one-shot pack generator
   - How it works: type a prompt → get WAV
   - Why it matters: infinite samples, no sample clearance

## Week 38 — Paid Beta Offer
**Product options:**
1. **Single pack:** $29 — one themed pack (e.g. "Noir Piano Kit")
2. **Beta access:** $99 — all packs + CLI access + custom requests
3. **Custom pack:** $199 — you describe it, we generate it

**Checkout flow:**
- GitHub Sponsors / Buy Me a Coffee / Stripe link
- Email delivery of generated pack ZIP
- Or: self-serve via CLI (`cshot make "custom pack"`)

## Week 39 — Feedback From Real Producers
**Target:** 5–10 producers

**Ask:**
1. Which sounds are usable?
2. Which are trash?
3. Would you pay $29/$99/$199?
4. What categories matter most?
5. What's missing?

**Track:**
- `ratings.jsonl` collects all ratings + notes
- `taste_profile.json` shows aggregate preferences

## Week 40 — Profit MVP
**Ship checklist:**
- [ ] Stable CLI build (`./cshot make` works end-to-end)
- [ ] 3–5 generated packs in `Packs/`
- [ ] Ratings system active (user can rate/trash/favorite)
- [ ] Taste profile learning from ratings
- [ ] Commercial pack templates ready
- [ ] Paid beta offer published
- [ ] At least 1 person outside you has used it
- [ ] At least 1 person is willing to pay

**v0.2 roadmap:**
- Better piano, synth, guitar, bass, FX (iteration)
- Gradio UI improvements (drag-drop, waveform view)
- Reference search integration into generation
- Prompt suggestions from history
- Batch upscaling (generate → rate → regenerate weak)
