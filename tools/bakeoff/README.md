# cShot Model Bakeoff

Compare providers head-to-head on 20 test prompts (5 per category) with 5 seeds each.

## Usage

```bash
# Show available providers and prompts (dry run, no API calls)
cargo run --bin bakeoff -- --dry-run

# Run bakeoff with cShot engine (free, always works)
cargo run --bin bakeoff -- --provider cshot-engine

# Run with specific provider (requires API key in .env)
cargo run --bin bakeoff -- --provider elevenlabs

# Evaluate existing results without generating
cargo run --bin bakeoff -- --eval-only --dir ./bakeoff_results
```

## Output Structure

```
bakeoff_results/
├── bakeoff_metadata.json     # Run config, timestamps, provider info
├── {provider}/
│   ├── kick_01_seed42.wav
│   ├── kick_01_seed123.wav
│   └── ...
├── results.json              # Per-sound metrics
└── summary.json              # Aggregated provider scores
```

## Adding a Provider

1. Implement `AudioProvider` trait in `src-tauri/src/generation/`
2. Register in `build_default_registry()` in `mod.rs`
3. Provider is available for bakeoff automatically
