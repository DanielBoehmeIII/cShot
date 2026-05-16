# cShot Known Issues (Beta)

## Generation
- Mock DSP sounds are improved but not production-grade AI quality
- ElevenLabs provider requires API key and internet
- Stable Audio and AudioLDM providers are stubs (not implemented)
- No local model inference (all generation is cloud API or mock DSP)

## UI
- No undo for deleted sounds (deletion is permanent)
- No drag-and-drop from library to DAW (button-based export only)
- Waveform interaction is click-to-play only (no scrubbing, no zoom)
- No tagging editor in the UI (auto-tags only)

## Export
- Default export is Desktop — no folder picker UI
- WAV format only (no AIFF, FLAC, MP3 export)
- Export counter handles name collisions but may create many files
- No batch export UI beyond "Export All" variants

## Library
- Search is basic SQL LIKE query — no fuzzy matching
- No bulk operations (select multiple, export all)
- No "find similar" or semantic search
- No pagination (all sounds load at once)
- No pack management UI (CRUD exists in backend only)

## Local Data
- First run creates ~/.cshot/ directory automatically
- No auto-cleanup of old generated sounds
- Database backup on reset but no scheduled backups

## Cross-Platform
- Linux: tested on Ubuntu 22.04+
- macOS: tested on Apple Silicon
- Windows: not yet tested
