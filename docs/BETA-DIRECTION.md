# cShot Beta Direction

## What Alpha Proved

1. **The core loop works.** Type → generate → hear → export. The 10-second
   cycle from idea to WAV file is real and testable.
2. **Producers want speed.** The value of generating vs. browsing is
   immediately understood.
3. **Variants are valuable.** 5 variants per generation increases engagement.
4. **The library matters.** Users want to keep, organize, and find their sounds.

## What Alpha Failed to Prove

1. **Sound quality is sufficient.** Mock DSP sounds are functional but not
   "wow" — the magic moment requires better audio.
2. **Reference upload is intuitive.** Users don't naturally try uploading.
3. **The price point.** No one has paid. Willingness to pay is unknown.
4. **Retention.** Do users come back the next day? Unknown.

## Beta Thesis

cShot should evolve into a **local-first one-shot generator with quick export
to DAW** — not a sample library, not a marketplace, not a plugin. The product
is: you type, you hear, you use.

## What to Build Next (8-Week Roadmap)

### Weeks 1-2: Sound Quality
- Improve mock DSP output (better envelopes, more variety, type-specific
  synthesis parameters)
- Add saturation and filtering to the DSP pipeline

### Weeks 3-4: Library Improvements
- Better search (FTS5, tag-based)
- Bulk delete
- Export multiple at once

### Weeks 5-6: Workspace Integration
- Remember last export folder
- Better DAW-friendly filenames
- Quick export to configurable locations

### Weeks 7-8: Polish
- Keyboard shortcuts (space to play, enter to generate)
- Performance pass
- Error state coverage

## What to Cut

- Pack builder (deferred to post-beta)
- Bakeoff (developer tool, not user-facing)
- Embeddings/semantic search (nice but not essential)
- Provider abstraction beyond mock + one real provider

## Risk Register

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Sound quality not good enough | High | DSP improvement sprint |
| Users don't return | Medium | Notifications, email, DAW integration |
| Mock DSP can't scale | Low | Add real provider when needed |
| Feature creep | High | Strict scope management |
