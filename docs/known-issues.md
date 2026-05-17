# Known Limitations — cShot Beta

## Sound Quality

- cShot Engine (local synthesis) produces functional one-shots but quality
  varies by category. Kicks and bass hits are the strongest; FX and
  "other" sounds are the weakest.
- Spectral centroid estimation is approximate — uses FFT approximation.
- Pitch detection works best for monophonic sounds with clear fundamental.
- Some prompts may produce unexpected sound types if keywords are ambiguous.

## Audio

- Mono only — stereo support is not yet implemented.
- WAV export at 44.1kHz 16-bit only.
- No real-time audio output (must export to hear in DAW).
- MP3 import works but with reduced quality.

## Generation

- Sound type classification from prompts is keyword-based and may not
  capture complex intent.
- Genre presets are generic templates — they don't adapt to the specific
  prompt context.
- Reference-conditioned generation modifies audio via DSP processing on
  the reference rather than true style transfer.

## CLI & Plugin

- CLI tool (`cshot-cli`) requires Tauri build environment.
- Plugin binary (`cshot-plugin`) is a standalone test tool, not a real
  VST3/CLAP plugin. Wrapping with nih-plug is documented but not built.
- No MIDI input support yet.

## Platform

- Tested primarily on Linux and macOS.
- Windows build requires MSVC build tools.
- Apple Silicon (M1/M2) works, no specific ANE acceleration.
- No auto-update mechanism.

## Library

- Library search is basic (LIKE-based, not full-text search).
- No batch operations beyond export-all-favorites.
- No undo for delete operations.
- No cloud sync.

## Performance

- Generating sounds with very long tails (>2s) increases CPU time.
- Analysis of long audio files (>30s) may take several seconds.
- Library may slow with 500+ sounds.
- Currently all generation is single-threaded.
