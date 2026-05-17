# Beta Ship Checklist

## Week 132 — Final Verification

### 1. Installation & Run

- [ ] `npm install` completes without errors
- [ ] `npm run tauri dev` starts the application
- [ ] App cold starts in <2s
- [ ] No unhandled panics on startup
- [ ] No JavaScript console errors on load

### 2. Upload / Reference

- [ ] Reference upload dialog opens
- [ ] WAV files are analyzed (duration, sample rate, waveform)
- [ ] MP3 upload validation fails gracefully
- [ ] Reference waveform renders
- [ ] Reference playback works
- [ ] Clear reference works

### 3. Generation

- [ ] Prompt "punchy kick 140" generates a sound
- [ ] Prompt with reference generates a conditioned sound
- [ ] Empty prompt shows validation error
- [ ] Generation spinner appears during generation
- [ ] Sound card renders with waveform, tags, score
- [ ] Variants auto-generate (5 variants)
- [ ] "Generate more variants" button works
- [ ] Error messages are user-friendly
- [ ] Silent output handling works

### 4. Local Engine

- [ ] cShot Engine generates without any API key or configuration
- [ ] Generation works immediately after fresh install
- [ ] Cloud provider errors never block generation
- [ ] Provider selector shows cShot Engine as default, always available

### 5. Preview / Playback

- [ ] Click sound card to play
- [ ] Click again to stop
- [ ] Playback indicator shows correctly
- [ ] Multiple sounds can be played sequentially
- [ ] Waveform renders for all sounds

### 6. Library

- [ ] Library view loads sounds
- [ ] Pagination works (Load More)
- [ ] Sound type filter works
- [ ] Favorites filter works
- [ ] Search by prompt works
- [ ] Search by tag works
- [ ] Search by type works
- [ ] Search by source works
- [ ] Search by model works
- [ ] Sound card shows prompt, type, tags, duration, score

### 7. Import

- [ ] Single WAV import works
- [ ] Single MP3 import works
- [ ] Imported file is analyzed (duration, RMS, peak)
- [ ] Imported file gets auto-tags
- [ ] Folder scan shows file preview
- [ ] Folder import respects file count limit (200)
- [ ] Folder import filters by supported formats (WAV, MP3)
- [ ] Folder import skips oversized files (>50MB)
- [ ] Imported sounds show in library

### 8. Search

- [ ] Search by prompt text
- [ ] Search by sound type
- [ ] Search by tag name
- [ ] Search by source (generated, variant, imported)
- [ ] Search by model name
- [ ] Search by variant name
- [ ] Search by notes

### 9. Favorites

- [ ] Toggle favorite on sound card
- [ ] Toggle favorite on variant card
- [ ] Favorites filter shows favorited sounds
- [ ] Favorite count updates
- [ ] Export all favorites (zip) works
- [ ] Favorites persist across app restarts

### 10. Packs

- [ ] Create pack works
- [ ] Add sound to pack works
- [ ] Remove sound from pack works
- [ ] Pack cohesion displays (tag overlap, duration consistency, etc.)
- [ ] Pack export (zip) works
- [ ] Pack detail view works
- [ ] Update pack notes works

### 11. Export

- [ ] Single sound export to Desktop
- [ ] Semantic filename generated
- [ ] Duplicate filename handled (incrementing suffix)
- [ ] Export notification shows filename and size
- [ ] Export all favorites (zip) works
- [ ] Exported WAV plays correctly

### 12. Cleanup / Reset

- [ ] Clear failed sounds works
- [ ] Clear orphaned files works
- [ ] Clear all generated sounds (preserve favorites) works
- [ ] Reset database (backup + recreate) works
- [ ] Rebuild metadata works
- [ ] Integrity scan detects missing/orphan files
- [ ] Clear recent prompts works
- [ ] Clear session memory works
- [ ] Storage location display works

### 13. Build Verification

- [ ] `cargo check` passes in src-tauri/
- [ ] `npx tsc --noEmit` passes
- [ ] `npm run build` completes
- [ ] `npm run tauri build` produces working binary
- [ ] No compiler warnings for Rust
- [ ] No TypeScript errors
