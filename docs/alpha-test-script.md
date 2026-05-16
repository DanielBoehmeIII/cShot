# cShot Alpha Test Script

## Before Starting

- Open cShot
- No account, no setup needed
- Cursor is in the prompt input

---

## Test 1: Basic Generation

1. Type `punchy kick 140` in the prompt bar
2. Press Enter
3. Observe: spinner appears, then SoundCard loads with waveform
4. Click the waveform to hear the sound
5. Click heart to favorite
6. Click Export
7. Verify WAV file appears on Desktop

**Check:** Generation < 10s, sound plays, file exports.

---

## Test 2: Variant Generation

1. Generate a sound (Test 1)
2. Observe 5 variant cards appear below
3. Click each variant to preview
4. Favorite 2 variants
5. Export 1 variant

**Check:** Variants sound different, play correctly.

---

## Test 3: Reference Upload

1. Click WAV button next to prompt
2. Select a .wav file
3. Observe ReferenceCard with waveform and metadata
4. Type `same character, snappier attack`
5. Press Enter — new sound generates from reference
6. Preview both reference and generated sound

**Check:** Reference loads, generation relates to reference.

---

## Test 4: Library

1. Switch to Library tab
2. Observe all generated sounds listed
3. Search for `kick`
4. Filter by type (select Kick)
5. Play a sound from library
6. Delete a sound
7. Switch back to Generate tab

**Check:** Library shows sounds, search/filter works.

---

## Test 5: Edge Cases

1. Generate with empty prompt → error message shown
2. Type one word `kick` → vague prompt warning appears
3. Click Generate rapidly → only one generation runs
4. Try importing a non-WAV file → clear error
5. Close and reopen app → favorites persist

**Check:** All edge cases handled gracefully.

---

## Test 6: Cleanup Tools

1. Switch to Tools tab
2. Click "Clear Failed Sounds" → confirm
3. Click "Scan Integrity" → view report
4. Click "Clear Orphaned Files" → confirm

**Check:** Tools work without errors.
