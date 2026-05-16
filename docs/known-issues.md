# cShot Known Issues (Alpha)

## Generation
- Mock DSP sounds are basic — not production quality
- Real providers (ElevenLabs, Stable Audio) require API keys
- Generation quality is not validated to SoundScore standards

## UI
- No keyboard shortcuts
- No undo for deleted sounds
- No drag-and-drop from library to DAW

## Export
- Default export is Desktop — no folder picker
- WAV format only (no AIFF, FLAC, MP3)
- Export counter handles name collisions but may create many files

## Library
- Search is basic LIKE query — no fuzzy matching
- No bulk operations (select multiple, export all)
- No "find similar" or semantic search

## Local Data
- First run creates ~/.cshot/ directory automatically
- No auto-cleanup of old generated sounds
- Database backup on reset but no scheduled backups

## Cross-Platform
- Linux: tested on Ubuntu 22.04+
- macOS: tested on Apple Silicon
- Windows: not yet tested
