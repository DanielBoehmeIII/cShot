# Week 40 — v3 Launch Audit

## Quality
- [ ] Generate 5 kits from different prompts, manually review all sounds
- [ ] Run `cshot gate` on each kit — expect >80% pass rate
- [ ] No clipping, no silent files, no near-duplicates in final curated pack
- [ ] All 5 demo kits from `cshot build-demo-kits` are genuinely listenable

## Speed
- [ ] `cshot demo` generates 20 sounds in <5 seconds
- [ ] `cshot make` with quality gates completes in <10 seconds for 40 sounds
- [ ] `cshot from-song` with 30s audio file completes in <15 seconds

## UX
- [ ] New user can run `cshot onboard` without reading docs
- [ ] `cshot listen` keyboard shortcuts work (f/t/g/b/space/a/B)
- [ ] `cshot --help` shows only product commands (not lab)
- [ ] `cshot lab --help` shows all advanced commands

## Product Surface
- [ ] 3 core commands work: `make`, `from-song`, `from-sample`
- [ ] `cshot make` includes quality gate, polish, rank, export
- [ ] `cshot from-song --style strict|inspired|wild` produces different results
- [ ] `cshot learn-from-folder` builds valid DNA profile
- [ ] `cshot export-cshotpack` / `import-cshotpack` round-trips cleanly

## Demand
- [ ] Landing page at docs/landing/index.html communicates value
- [ ] Video demo script ready (docs/landing/VIDEO_DEMO.md)
- [ ] Beta pricing defined (docs/landing/BETA_OFFER.md)

## Revenue (Beta)
- [ ] At least 1 pre-order or payment at $29/$99
- [ ] Or 5+ serious beta signups
- [ ] Feedback collection via `cshot feedback-pack` ready

## Retention
- [ ] Taste profile saves and reloads
- [ ] `cshot taste --rebuild` shows accurate preferences
- [ ] Rated sounds influence future generation (DNA matching)

## Launch Checklist
- [ ] README.md updated with new product commands
- [ ] Clean install path documented (`pip install -r requirements.txt`)
- [ ] Demo kits built: `cshot build-demo-kits --count 30`
- [ ] Known issues documented
- [ ] Feedback mechanism ready: `cshot feedback-pack`
- [ ] Final `cshot --help` verified
