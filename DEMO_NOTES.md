# Demo Notes — Golden One-Shot Set

Generated from `mvp-audit` with 170 candidates. Top 50 selected via balanced ranking (2 per category, then fill with best remaining).

## Best Prompts by Family

### Piano
| Prompt | Profile | Character |
|--------|---------|-----------|
| "dark soft piano stab" | acoustic | Mellow, soft attack, warm lows |
| "bright hard piano stab" | acoustic | Aggressive, bright harmonics, fast attack |
| "piano" + bell | bell | Clean bell-tone, moderate decay |
| "piano" + rhodes | rhodes | Warm electric piano, soft body |

### Synth
| Prompt | Profile | Character |
|--------|---------|-----------|
| "bright pluck" | pluck | High centroid (~10k Hz), short decay |
| "dark pluck" | pluck | Low centroid (~1.7k Hz), warm body |
| "clean pluck" | pluck | Moderate brightness, balanced envelope |
| "distorted pluck" | pluck | Saturated, higher RMS |
| "narrow pluck" | pluck | Mono-like, tight stereo |
| "wide pluck" | pluck | Stereo spread, spatial |
| "lo-fi pluck" | pluck | Lo-fi warmth, reduced bandwidth |
| "soft pluck" | pluck | Slow attack (~27ms) |
| "punchy pluck" | pluck | Fast attack (~2.5ms), high RMS |
| "synth pad" | pad | Slow attack (~100ms), long release |
| "synth lead" | lead | Bright tele-like, moderate body |

### Bass
| Prompt | Profile | Character |
|--------|---------|-----------|
| "808" | 808 | Deep sub, fast attack, long tail |
| "reese" | reese | Wide detuned bass, moderate attack |
| "distorted bass" | distorted | Saturated low end, growl |
| "fm bass" | fm | Metallic FM character |

### Guitar
| Prompt | Profile | Character |
|--------|---------|-----------|
| "nylon guitar" | nylon | Warm nylon-string pluck |
| "bright guitar" | bright | Bright acoustic pluck |

### Drums
| Prompt | Profile | Character |
|--------|---------|-----------|
| "kick" | kick | Sub-heavy, fast transient |
| "snare" | snare | Snappy, mid-focused |
| "clap" | clap | Layered noise burst |
| "hat" | closed_hat | Short, high-frequency tick |

### FX
| Prompt | Profile | Character |
|--------|---------|-----------|
| "impact" | impact | Big transient, long tail (~1.5s) |
| "riser" | riser | Slow build (~1.4s attack), spectral sweep |
| "glitch" | glitch | Rhythmic noise, glitchy texture |

## Category Distribution
25 categories, 2 files each = 50 total.

## Feature Range
- RMS: 0.05–0.40
- Peak: 0.95 (normalized)
- Spectral centroid: 1.2k–10k Hz
- Attack: 0.5ms–1.4s
- Duration: 0.1s–2.0s

## Usage
```bash
# Listen to top picks
./cshot listen outputs/golden_demo_candidates

# Regenerate specific categories
./cshot prompt "punchy kick" --count 10 --out outputs/kicks_v2/
```

## Manual Curation Notes
(L1 is placeholders — replace with real listening notes after listening.)
