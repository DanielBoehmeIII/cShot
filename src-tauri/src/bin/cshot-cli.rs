#![allow(
    clippy::collapsible_if,
    clippy::get_first,
    clippy::too_many_arguments,
    clippy::blocks_in_conditions,
    clippy::manual_clamp,
    clippy::wildcard_in_or_patterns,
    clippy::redundant_closure,
    clippy::doc_markdown,
)]

// cShot CLI — Headless one-shot synthesis.
//
// Usage:
//   cargo run ... --bin cshot-cli -- oneshot kick
//   cargo run ... --bin cshot-cli -- variate kick --count 8
//   cargo run ... --bin cshot-cli -- presets
//   cargo run ... --bin cshot-cli -- qa
//   cargo run ... --bin cshot-cli -- generate "punchy kick 140bpm"

use std::path::{Path, PathBuf};
use std::time::Instant;

use cshot_lib::audio::one_shot_controls::OneShotControls;
use cshot_lib::audio::spec::{OneShotSpec, SoundClass};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "generate" => cmd_generate(&args[1..]),
        "analyze" => cmd_analyze(&args[1..]),
        "transform" => cmd_transform(&args[1..]),
        "recreate" => cmd_recreate(&args[1..]),
        "render" => cmd_render(&args[1..]),
        "list-recipes" => cmd_list_recipes(),
        "benchmark" => cmd_benchmark(),
        "oneshot" => cmd_oneshot(&args[1..]),
        "variate" => cmd_variate(&args[1..]),
        "presets" => cmd_presets(&args[1..]),
        "qa" => cmd_qa(&args[1..]),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("cShot CLI — One-shot synthesis engine");
    eprintln!();
    eprintln!("Usage: cshot-cli <command> [args...]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  generate <prompt> [--output <path>]        Generate a one-shot from prompt text");
    eprintln!("  oneshot [class] [options]                   Generate from SoundClass with controls");
    eprintln!("  variate <class> [--count N] [options]       Generate N variations of a class");
    eprintln!("  presets [list|default]                      List presets or show 808 default");
    eprintln!("  qa [--output-dir <path>]                    QA audit: all classes x 10 examples");
    eprintln!("  analyze <file>                              Analyze a sound file");
    eprintln!("  transform <file> <prompt> [--output]        Transform a sound");
    eprintln!("  recreate <file> [--output] [--count N]      Recreate a sound");
    eprintln!("  render <prompt> [--count N] [--dir D]       Batch render variations");
    eprintln!("  list-recipes                                List built-in sound types");
    eprintln!("  benchmark                                   Run synthesis benchmarks");
    eprintln!();
    eprintln!("oneshot options:");
    eprintln!("  --class <name>      Sound class (808, kick, snare, clap, closed_hat, open_hat,");
    eprintln!("                        bass_stab, impact_fx, synth_stab). Default: 808");
    eprintln!("  --duration <ms>     Duration in milliseconds (default: class-specific)");
    eprintln!("  --pitch <hz>        Base pitch in Hz (default: class-specific)");
    eprintln!("  --gain <0-1>        Output gain (default: 1.0)");
    eprintln!("  --brightness <0-1>  0=dark, 0.5=preserve, 1.0=bright");
    eprintln!("  --punch <0-1>       0=soft, 0.5=preserve, 1.0=punchy");
    eprintln!("  --decay <0-1>       0=short, 0.5=preserve, 1.0=long");
    eprintln!("  --distortion <0-1>  0=clean, 0.5=preserve, 1.0=distorted");
    eprintln!("  --transient <0-1>   0=no transient, 0.5=preserve, 1.0=max");
    eprintln!("  --noise <0-1>       0=no noise, 0.5=preserve, 1.0=max");
    eprintln!("  --body <0-1>        0=no body, 0.5=preserve, 1.0=max");
    eprintln!("  --stereo-width <0-1> 0=mono, 0.5=preserve, 1.0=wide");
    eprintln!("  --pitch-drop <0-1>  0=no drop, 0.5=preserve, 1.0=max");
    eprintln!("  --filter-sweep <0-1> 0=no sweep, 0.5=preserve, 1.0=max");
    eprintln!("  --randomize [amt]   Randomize controls by ±amount (default 0.3)");
    eprintln!("  --preview           Render and show stats without writing file");
    eprintln!("  --output <path>     Output WAV file path");
    eprintln!();
    eprintln!("variate options:");
    eprintln!("  --count <N>         Number of variations (default: 5)");
    eprintln!("  --output-dir <path> Output directory (default: ./<class>_variations)");
    eprintln!("  --random-amount <0-1>  Randomization intensity (default: 0.3)");
    eprintln!("  (also accepts all oneshot control flags as base)");
}

// ─── Argument Parsing Helpers ─────────────────────────────────

fn get_flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|w| w[0] == name)
        .and_then(|w| w.get(1))
        .map(|s| s.as_str())
}

fn get_flag_parse<F: std::str::FromStr>(args: &[String], name: &str) -> Option<F> {
    get_flag(args, name).and_then(|s| s.parse().ok())
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn has_randomize(args: &[String]) -> Option<f32> {
    let pos = args.iter().position(|a| a == "--randomize")?;
    if let Some(next) = args.get(pos + 1) {
        if let Ok(amt) = next.parse::<f32>() {
            if (0.0..=1.0).contains(&amt) {
                return Some(amt);
            }
        }
    }
    Some(0.3)
}

fn get_output_path(args: &[String], default_name: &str) -> PathBuf {
    get_flag(args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default_name))
}

fn spec_for_class(class: SoundClass) -> OneShotSpec {
    match class {
        SoundClass::Sub808 => OneShotSpec::preset_808(),
        SoundClass::Kick => OneShotSpec::preset_kick(),
        SoundClass::Snare => OneShotSpec::preset_snare(),
        SoundClass::Clap => OneShotSpec::preset_clap(),
        SoundClass::ClosedHat => OneShotSpec::preset_closed_hat(),
        SoundClass::OpenHat => OneShotSpec::preset_open_hat(),
        SoundClass::BassStab => OneShotSpec::preset_bass_stab(),
        SoundClass::ImpactFx => OneShotSpec::preset_impact_fx(),
        SoundClass::SynthStab => OneShotSpec::preset_synth_stab(),
    }
}

fn parse_controls(args: &[String]) -> Option<OneShotControls> {
    let brightness = get_flag_parse::<f32>(args, "--brightness");
    let punch = get_flag_parse::<f32>(args, "--punch");
    let decay = get_flag_parse::<f32>(args, "--decay");
    let distortion = get_flag_parse::<f32>(args, "--distortion");
    let transient = get_flag_parse::<f32>(args, "--transient");
    let noise = get_flag_parse::<f32>(args, "--noise");
    let body = get_flag_parse::<f32>(args, "--body");
    let sw = get_flag_parse::<f32>(args, "--stereo-width");
    let pd = get_flag_parse::<f32>(args, "--pitch-drop");
    let fs = get_flag_parse::<f32>(args, "--filter-sweep");

    let any = brightness.or(punch).or(decay).or(distortion)
        .or(transient).or(noise).or(body).or(sw).or(pd).or(fs);

    if any.is_some() {
        Some(OneShotControls::from_preset(
            brightness, punch, decay, distortion,
            transient, noise, body, sw, pd, fs,
        ))
    } else {
        None
    }
}

fn control_label(ctrl: &OneShotControls) -> String {
    format!(
        "b{:.2}_p{:.2}_d{:.2}_dt{:.2}",
        ctrl.brightness, ctrl.punch, ctrl.decay, ctrl.distortion
    )
}

fn print_stats(label: &str, samples: &[f32]) {
    let peak = cshot_lib::audio::compute_peak(samples);
    let rms = cshot_lib::audio::compute_rms(samples);
    let duration_ms = samples.len() as f32 / 44100.0 * 1000.0;
    println!("  {}: {:.1}ms  peak={:.3}  RMS={:.5}  samples={}", label, duration_ms, peak, rms, samples.len());
}

// ─── oneshot: Generate from SoundClass with full control ────────

fn cmd_oneshot(args: &[String]) {
    let args = &args[1..]; // skip "oneshot"

    // Parse class: either first positional arg or --class flag
    let class_str = if !args.is_empty() && !args[0].starts_with("--") {
        let candidate = args[0].to_lowercase();
        if SoundClass::is_valid(&candidate) {
            candidate
        } else {
            eprintln!("Invalid sound class: {}. Valid: {:?}", candidate, SoundClass::all_values());
            std::process::exit(1);
        }
    } else {
        get_flag(args, "--class").unwrap_or("808").to_lowercase()
    };
    let class = SoundClass::from_str(&class_str);

    // Get default spec for this class, then apply overrides
    let default_spec = spec_for_class(class);
    let duration = get_flag_parse::<f32>(args, "--duration").unwrap_or(default_spec.duration_ms);
    let pitch = get_flag_parse::<f32>(args, "--pitch").unwrap_or(default_spec.pitch_hz);
    let gain = get_flag_parse::<f32>(args, "--gain").unwrap_or(default_spec.gain);

    // Parse controls
    let has_control_flags = parse_controls(args);
    let randomize_amt = has_randomize(args);
    let preview = has_flag(args, "--preview");

    let controls = if has_control_flags.is_some() || randomize_amt.is_some() {
        let mut ctrl = has_control_flags.unwrap_or_default();
        if let Some(amt) = randomize_amt {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            ctrl = ctrl.randomize(amt, seed);
        }
        Some(ctrl)
    } else {
        None // pure preset mode — preserves exact old behavior
    };

    let spec = OneShotSpec {
        sound_class: class,
        duration_ms: duration,
        pitch_hz: pitch,
        gain,
        controls: controls.clone(),
    };

    let preview_start = if preview {
        eprintln!("Rendering preview...");
        Some(Instant::now())
    } else {
        None
    };
    let samples = spec.render();
    if let Some(s) = preview_start {
        eprintln!("Rendered in {:.1}ms", s.elapsed().as_secs_f64() * 1000.0);
    }

    if samples.is_empty() {
        eprintln!("Error: generated empty audio");
        std::process::exit(1);
    }
    if samples.iter().any(|s| s.is_nan() || s.is_infinite()) {
        eprintln!("Error: generated audio contains NaN/Inf");
        std::process::exit(1);
    }

    let peak = cshot_lib::audio::compute_peak(&samples);
    let rms = cshot_lib::audio::compute_rms(&samples);
    let actual_duration_ms = samples.len() as f32 / 44100.0 * 1000.0;

    if preview {
        println!("[preview] class={} dur={:.0}ms pitch={:.0}Hz gain={:.2}",
            class_str, duration, pitch, gain);
        if let Some(ref ctrl) = controls {
            println!("  controls: brightness={:.2} punch={:.2} decay={:.2} distortion={:.2}",
                ctrl.brightness, ctrl.punch, ctrl.decay, ctrl.distortion);
            println!("           transient={:.2} noise={:.2} body={:.2} stereo_width={:.2}",
                ctrl.transient_amount, ctrl.noise_amount, ctrl.body_amount, ctrl.stereo_width);
            println!("           pitch_drop={:.2} filter_sweep={:.2}",
                ctrl.pitch_drop, ctrl.filter_sweep);
        } else {
            println!("  controls: (default preset)");
        }
        println!("  output: {:.1}ms  peak={:.3}  RMS={:.5}  samples={}",
            actual_duration_ms, peak, rms, samples.len());
        return;
    }

    // Build output filename including class + key controls
    let output = if let Some(ref ctrl) = controls {
        let label = control_label(ctrl);
        get_output_path(args, &format!("{}_{}.wav", class_str, label))
    } else {
        get_output_path(args, &format!("{}.wav", class_str))
    };

    cshot_lib::audio::write_wav(&output, &samples, 44100)
        .expect("Failed to write WAV");

    println!("Generated: {} -> {}", class_str, output.display());
    println!("  Duration: {:.1}ms  Peak: {:.3}  RMS: {:.5}  Samples: {}",
        actual_duration_ms, peak, rms, samples.len());
    if randomize_amt.is_some() {
        println!("  Randomized: ±{:.2}", randomize_amt.unwrap_or(0.3));
    }
}

// ─── variate: Generate N variations of a class ────────────────

fn cmd_variate(args: &[String]) {
    let args = &args[1..]; // skip "variate"

    if args.is_empty() || args[0].starts_with("--") {
        eprintln!("Usage: cshot-cli variate <class> [--count N] [--output-dir <path>] [--random-amount <0-1>]");
        eprintln!("       (also accepts oneshot control flags to set base values)");
        std::process::exit(1);
    }

    let class_str = args[0].to_lowercase();
    if !SoundClass::is_valid(&class_str) {
        eprintln!("Invalid sound class: {}. Valid: {:?}", class_str, SoundClass::all_values());
        std::process::exit(1);
    }
    let class = SoundClass::from_str(&class_str);

    let count: usize = get_flag_parse::<f32>(args, "--count")
        .map(|v| v as usize)
        .unwrap_or(5)
        .max(1)
        .min(100);

    let random_amount = get_flag_parse::<f32>(args, "--random-amount").unwrap_or(0.3);
    let output_dir = get_flag(args, "--output-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("{}_variations", class_str)));

    let base_controls = parse_controls(args).unwrap_or_default();

    std::fs::create_dir_all(&output_dir).ok();
    println!("Generating {} variations of {} in {}:", count, class_str, output_dir.display());

    let base_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    for i in 0..count {
        let seed = base_seed.wrapping_add(i as u64).wrapping_mul(314159265);
        let ctrl = base_controls.randomize(random_amount, seed);

        let spec = OneShotSpec {
            sound_class: class,
            duration_ms: get_flag_parse::<f32>(args, "--duration").unwrap_or(match class {
                SoundClass::Sub808 => 800.0,
                SoundClass::Kick => 280.0,
                SoundClass::Snare => 320.0,
                SoundClass::Clap => 380.0,
                SoundClass::ClosedHat => 150.0,
                SoundClass::OpenHat => 650.0,
                SoundClass::BassStab => 350.0,
                SoundClass::ImpactFx => 1500.0,
                SoundClass::SynthStab => 600.0,
            }),
            pitch_hz: get_flag_parse::<f32>(args, "--pitch").unwrap_or(match class {
                SoundClass::Sub808 => 55.0,
                SoundClass::Kick => 100.0,
                SoundClass::Snare => 220.0,
                SoundClass::Clap => 180.0,
                SoundClass::ClosedHat => 500.0,
                SoundClass::OpenHat => 350.0,
                SoundClass::BassStab => 80.0,
                SoundClass::ImpactFx => 70.0,
                SoundClass::SynthStab => 220.0,
            }),
            gain: get_flag_parse::<f32>(args, "--gain").unwrap_or(1.0),
            controls: Some(ctrl.clone()),
        };

        let samples = spec.render();
        if samples.is_empty() || samples.iter().any(|s| s.is_nan() || s.is_infinite()) {
            eprintln!("  [{}/{}] SKIPPED (silent/NaN)", i + 1, count);
            continue;
        }

        let label = control_label(&ctrl);
        let out_path = output_dir.join(format!("{}_v{:02}_{}.wav", class_str, i + 1, label));

        if let Err(e) = cshot_lib::audio::write_wav(&out_path, &samples, 44100) {
            eprintln!("  [{}/{}] FAILED: {}", i + 1, count, e);
            continue;
        }
        print_stats(&format!("[{}/{}]", i + 1, count), &samples);
        println!("         -> {}", out_path.file_name().unwrap().to_string_lossy());
    }
    println!("Done. Output in: {}", output_dir.display());
}

// ─── presets: List/show sound class presets ──────────────────

fn cmd_presets(args: &[String]) {
    let show = args.get(1).map(|s| s.as_str()).unwrap_or("list");
    match show {
        "default" | "808" => {
            let spec = OneShotSpec::preset_808();
            println!("Default Preset: 808 (Sub808)");
            println!("  Duration: {}ms  Pitch: {}Hz  Gain: {}", spec.duration_ms, spec.pitch_hz, spec.gain);
            println!("  Controls: none (pure preset mode)");
            println!();
            println!("  This is the original cShot sound — deep sub-bass with slow pitch drop.");
            println!("  Use `cshot-cli oneshot` to generate with custom controls.");
        }
        "list" | _ => {
            println!("cShot Sound Class Presets");
            println!("========================\n");

            let classes = [
                ("808",      "Deep sub-bass 808, slow pitch drop",         800,   55.0),
                ("kick",     "Punchy kick with click attack",              280,  100.0),
                ("snare",    "Snare with noise burst + mid tone",          320,  220.0),
                ("clap",     "Clap with staggered noise bursts",           380,  180.0),
                ("closed_hat", "Tight closed hi-hat, short metallic tick", 150,  500.0),
                ("open_hat",  "Washy open hi-hat, longer sustain",         650,  350.0),
                ("bass_stab", "Sub-heavy bass stab with filter sweep",     350,   80.0),
                ("impact_fx", "Cinematic impact with long tail",          1500,   70.0),
                ("synth_stab", "Synth stab, detuned oscillator chord",     600,  220.0),
            ];

            println!("  {:<16} {:<48} {:>5}  Pitch", "Class", "Description", "Dur");
            println!("  {:-<16} {:-<48} {:-<5}  {:-<6}", "", "", "", "");
            for (name, desc, dur, pitch) in &classes {
                println!("  {:<16} {:<48} {:>4}ms  {:<.0}Hz", name, desc, dur, pitch);
            }
            println!();
            println!("  Default: 808 (preserved for backwards compatibility)");
            println!("  Use `cshot-cli oneshot <class>` to generate");
            println!("  Use `cshot-cli oneshot <class> --preview` to render without writing");
        }
    }
}

// ─── qa: QA audit — generate all classes x 10, validate, manifest ──

fn cmd_qa(args: &[String]) {
    let args = &args[1..]; // skip "qa"
    let output_dir = get_flag(args, "--output-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("test_outputs"));

    let samples_per_class: usize = get_flag_parse::<f32>(args, "--samples")
        .map(|v| v as usize)
        .unwrap_or(10)
        .max(1)
        .min(1000);

    std::fs::create_dir_all(&output_dir).ok();
    println!("cShot QA Suite");
    println!("==============");
    println!("Output: {}", output_dir.display());
    println!("Samples per class: {}\n", samples_per_class);

    let classes = [
        SoundClass::Sub808,
        SoundClass::Kick,
        SoundClass::Snare,
        SoundClass::Clap,
        SoundClass::ClosedHat,
        SoundClass::OpenHat,
        SoundClass::BassStab,
        SoundClass::ImpactFx,
        SoundClass::SynthStab,
    ];

    let base_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let mut manifest_entries: Vec<serde_json::Value> = Vec::new();
    let mut total_generated = 0usize;
    let mut total_passed = 0usize;
    let mut total_failed = 0usize;
    let mut silent_count = 0usize;
    let mut clipped_count = 0usize;
    let mut nan_count = 0usize;

    for (cix, &class) in classes.iter().enumerate() {
        let class_name = class.as_str();
        let class_dir = output_dir.join(class_name);
        std::fs::create_dir_all(&class_dir).ok();

        let default_spec = spec_for_class(class);
        let mut class_generated = 0usize;
        let mut class_passed = 0usize;
        let mut _class_failed = 0usize;

        for i in 0..samples_per_class {
            let seed = base_seed
                .wrapping_add(cix as u64)
                .wrapping_mul(314159265)
                .wrapping_add(i as u64);

            // Start from default controls and randomize for variety
            let ctrl = OneShotControls::default().randomize(0.4, seed);
            let ctrl_for_manifest = ctrl.clone();

            let spec = OneShotSpec {
                sound_class: class,
                duration_ms: default_spec.duration_ms,
                pitch_hz: default_spec.pitch_hz,
                gain: default_spec.gain,
                controls: Some(ctrl),
            };

            let samples = spec.render();
            total_generated += 1;
            class_generated += 1;

            let out_path = class_dir.join(format!("{}_{:03}.wav", class_name, i + 1));
            let mut status = "pass";
            let mut issues: Vec<String> = Vec::new();

            if samples.is_empty() {
                issues.push("silent".to_string());
                silent_count += 1;
                status = "fail";
            }
            if samples.iter().any(|s| s.is_nan() || s.is_infinite()) {
                issues.push("nan".to_string());
                nan_count += 1;
                status = "fail";
            }
            let peak = if samples.is_empty() { 0.0 } else { cshot_lib::audio::compute_peak(&samples) };
            let rms = if samples.is_empty() { 0.0 } else { cshot_lib::audio::compute_rms(&samples) };
            if peak > 0.999 {
                issues.push("clipped".to_string());
                clipped_count += 1;
                if status == "pass" { status = "warn"; }
            }
            if peak < 0.0001 && !samples.is_empty() {
                issues.push("silent".to_string());
                silent_count += 1;
                status = "fail";
            }

            if status == "pass" || status == "warn" {
                if let Err(e) = cshot_lib::audio::write_wav(&out_path, &samples, 44100) {
                    issues.push(format!("write_error:{}", e));
                    status = "fail";
                }
            }

            if status == "fail" {
                total_failed += 1;
                _class_failed += 1;
            } else {
                total_passed += 1;
                class_passed += 1;
            }

            let duration_ms = if samples.is_empty() { 0.0 } else { samples.len() as f32 / 44100.0 * 1000.0 };

            manifest_entries.push(serde_json::json!({
                "class": class_name,
                "index": i + 1,
                "seed": seed,
                "status": status,
                "issues": issues,
                "path": out_path.to_string_lossy().to_string(),
                "relative_path": format!("{}/{}_{:03}.wav", class_name, class_name, i + 1),
                "peak": peak,
                "rms": rms,
                "duration_ms": duration_ms,
                "pitch_hz": spec.pitch_hz,
                "gain": spec.gain,
                "controls": {
                    "brightness": ctrl_for_manifest.brightness,
                    "punch": ctrl_for_manifest.punch,
                    "decay": ctrl_for_manifest.decay,
                    "distortion": ctrl_for_manifest.distortion,
                    "transient_amount": ctrl_for_manifest.transient_amount,
                    "noise_amount": ctrl_for_manifest.noise_amount,
                    "body_amount": ctrl_for_manifest.body_amount,
                    "stereo_width": ctrl_for_manifest.stereo_width,
                    "pitch_drop": ctrl_for_manifest.pitch_drop,
                    "filter_sweep": ctrl_for_manifest.filter_sweep,
                },
            }));
        }

        let pass_pct = if class_generated > 0 {
            (class_passed as f32 / class_generated as f32) * 100.0
        } else {
            0.0
        };
        println!("  {:<12} {:>3}/{:3} passed ({:>5.1}%)",
            class_name, class_passed, class_generated, pass_pct);
    }

    // Build manifest
    let manifest = serde_json::json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "cshot_version": env!("CARGO_PKG_VERSION"),
        "engine": "cshot-layer-resynthesis",
        "total_sounds": total_generated,
        "samples_per_class": samples_per_class,
        "num_classes": classes.len(),
        "summary": {
            "total": total_generated,
            "passed": total_passed,
            "failed": total_failed,
            "silent": silent_count,
            "clipped": clipped_count,
            "nan": nan_count,
        },
        "entries": manifest_entries,
    });

    let manifest_path = output_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&manifest_path, &manifest_json)
        .unwrap_or_else(|e| eprintln!("Warning: could not write manifest: {}", e));

    println!();
    println!("QA Summary");
    println!("----------");
    println!("  Total sounds: {}", total_generated);
    println!("  Passed:       {}", total_passed);
    println!("  Failed:       {}", total_failed);
    println!("  Silent:       {}", silent_count);
    println!("  Clipped:      {}", clipped_count);
    println!("  NaN:          {}", nan_count);
    println!();
    let pass_pct = if total_generated > 0 {
        (total_passed as f32 / total_generated as f32) * 100.0
    } else {
        0.0
    };
    println!("  Pass rate:    {:.1}%", pass_pct);
    println!();
    println!("Manifest: {}", manifest_path.display());
    println!("Status: {}", if total_failed == 0 && total_generated > 0 { "ALL PASSED" } else { "ISSUES FOUND" });
}

// ─── Existing Commands (preserved) ────────────────────────────

fn cmd_recreate(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: cshot-cli recreate <file> [--output <path>] [--count <N>]");
        std::process::exit(1);
    }
    let path = PathBuf::from(&args[1]);
    let output = args.windows(2)
        .find(|w| w[0] == "--output")
        .and_then(|w| w.get(1))
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("recreated.wav"));
    let count: usize = args.windows(2)
        .find(|w| w[0] == "--count")
        .and_then(|w| w.get(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);

    let samples = cshot_lib::audio::read_wav(&path).expect("Failed to read WAV");
    let analysis = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    let results = cshot_lib::audio::recreate::generate_approximations(
        &samples, &analysis, count, 0.5, true, true, true,
    );

    println!("Recreated: {} ({} approximations)", path.display(), results.len());
    for (i, r) in results.iter().enumerate() {
        let out_path = if i == 0 {
            output.clone()
        } else {
            let stem = output.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
            let ext = output.extension().map(|s| s.to_string_lossy()).unwrap_or_default();
            let parent = output.parent().unwrap_or(Path::new("."));
            parent.join(format!("{}_{}.{}", stem, i, ext))
        };
        cshot_lib::audio::write_wav(&out_path, &r.samples, 44100)
            .expect("Failed to write WAV");
        println!("  [{}/{}] Similarity: {:.1}% -> {}", i + 1, results.len(),
            r.similarity.overall * 100.0, out_path.display());
    }
}

fn cmd_render(args: &[String]) {
    let sub_args = &args[1..];
    if sub_args.is_empty() {
        eprintln!("Usage: cshot-cli render <prompt> [--count <N>] [--dir <directory>]");
        std::process::exit(1);
    }
    let prompt = &sub_args[0];
    let count: usize = sub_args.windows(2)
        .find(|w| w[0] == "--count")
        .and_then(|w| w.get(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    let dir = sub_args.windows(2)
        .find(|w| w[0] == "--dir")
        .and_then(|w| w.get(1))
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("."));

    std::fs::create_dir_all(&dir).ok();
    println!("Rendering {} variants of: {}", count, prompt);

    for i in 0..count {
        let ctrl = cshot_lib::prompt_dsp::parse_prompt_rich(prompt);
        let st = cshot_lib::audio::SoundType::from_str(&ctrl.sound_type);
        let pitch = ctrl.pitch_hz.unwrap_or(200.0);
        let dur = ctrl.duration_ms.unwrap_or(300.0);
        let base = cshot_lib::audio::resynthesize::params_for_sound_type(st, pitch, dur);
        let params = ctrl.to_resynthesis_params(&base);
        let seed = (i as u64).wrapping_mul(314159265).wrapping_add(42);
        let params = params.with_seed(seed).randomize(0.3);
        let samples = cshot_lib::audio::resynthesize::resynthesize(&params);

        let out_path = dir.join(format!("{}_{:03}.wav", prompt.replace(' ', "_"), i + 1));
        cshot_lib::audio::write_wav(&out_path, &samples, 44100)
            .expect("Failed to write WAV");
        println!("  [{}/{}] {} ({:.0}ms, {} samples)",
            i + 1, count, out_path.display(),
            samples.len() as f32 / 44100.0 * 1000.0, samples.len());
    }
}

fn cmd_list_recipes() {
    println!("cShot Built-in Sound Types (SoundClass):");
    println!();
    let recipes = [
        ("808",        "Deep sub-bass 808 with slow pitch drop",         55.0,   800),
        ("kick",       "Punchy kick drum with click attack",            100.0,   280),
        ("snare",      "Snare with noise burst and mid body tone",      220.0,   320),
        ("clap",       "Layered clap with staggered noise bursts",      180.0,   380),
        ("closed_hat",  "Tight closed hi-hat with metallic tick",       500.0,   150),
        ("open_hat",    "Washy open hi-hat with longer sustain",        350.0,   650),
        ("bass_stab",  "Sub-heavy bass stab with filter movement",       80.0,   350),
        ("impact_fx",  "Cinematic impact with long evolving tail",       70.0,  1500),
        ("synth_stab", "Synth stab with detuned oscillator chord",      220.0,   600),
    ];
    println!("  {:<14} {:<48} {:>7}  {:>5}", "Class", "Description", "Pitch", "Dur");
    println!("  {:-<14} {:-<48} {:-<7}  {:-<5}", "", "", "", "");
    for (name, desc, pitch, dur) in &recipes {
        println!("  {:<14} {:<48} {:>5.0}Hz  {:>4}ms", name, desc, pitch, dur);
    }
    println!();
    println!("  Default: 808 (backwards-compatible)");
    println!("  Use `cshot-cli oneshot <class>` to generate with full control.");
    println!("  Use `cshot-cli variate <class> --count 10` for batch variations.");
}

fn cmd_generate(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: cshot-cli generate <prompt> [--output <path>]");
        std::process::exit(1);
    }
    let prompt = &args[1];
    let output = args.windows(2)
        .find(|w| w[0] == "--output")
        .and_then(|w| w.get(1))
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("output.wav"));

    let ctrl = cshot_lib::prompt_dsp::parse_prompt_rich(prompt);
    let st = cshot_lib::audio::SoundType::from_str(&ctrl.sound_type);
    let pitch = ctrl.pitch_hz.unwrap_or(200.0);
    let dur = ctrl.duration_ms.unwrap_or(300.0);
    let base = cshot_lib::audio::resynthesize::params_for_sound_type(st, pitch, dur);
    let params = ctrl.to_resynthesis_params(&base);
    let samples = cshot_lib::audio::resynthesize::resynthesize(&params);

    if samples.is_empty() {
        eprintln!("Error: generation produced empty audio");
        std::process::exit(1);
    }

    cshot_lib::audio::write_wav(&output, &samples, 44100)
        .expect("Failed to write WAV file");
    println!("Generated: {} -> {}", prompt, output.display());
    println!("  Duration: {:.1}ms", samples.len() as f32 / 44100.0 * 1000.0);
    println!("  Samples: {}", samples.len());
}

fn cmd_analyze(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: cshot-cli analyze <file>");
        std::process::exit(1);
    }
    let path = PathBuf::from(&args[1]);
    let samples = cshot_lib::audio::read_wav(&path)
        .expect("Failed to read WAV file");
    let analysis = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);

    println!("Analysis of: {}", path.display());
    println!("  Duration: {:.1}ms", analysis.duration_ms);
    println!("  Type hint: {}", analysis.sound_type_hint);
    println!("  Peak: {:.3} dB", 20.0 * analysis.peak.max(1e-10).log10());
    println!("  RMS: {:.3} dB", 20.0 * analysis.rms.max(1e-10).log10());
    println!("  Loudness: {:.1} LUFS", analysis.loudness_lufs);
    println!("  Attack: {:.1}ms", analysis.attack_ms);
    println!("  Decay: {:.1}ms", analysis.decay_ms);
    println!("  Tail: {:.1}ms", analysis.tail_ms);
    println!("  Centroid: {:.0} Hz", analysis.spectral_centroid);
    println!("  Rolloff: {:.0} Hz", analysis.spectral_rolloff);
    println!("  Brightness: {:.3}", analysis.brightness);
    println!("  ZCR: {:.4}", analysis.zero_crossing_rate);
    println!("  Pitch: {:?}", analysis.pitch_estimate.map(|p| format!("{:.0} Hz", p)).unwrap_or_else(|| "none".to_string()));
    println!("  Transients: {}", analysis.transient_count);
    println!("  Clipping: {}", analysis.has_clipping);
}

fn cmd_transform(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: cshot-cli transform <file> <prompt> [--output <path>]");
        std::process::exit(1);
    }
    let path = PathBuf::from(&args[1]);
    let prompt = &args[2];
    let output = args.windows(2)
        .find(|w| w[0] == "--output")
        .and_then(|w| w.get(1))
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("transformed.wav"));

    let samples = cshot_lib::audio::read_wav(&path)
        .expect("Failed to read source WAV");
    let ctrl = cshot_lib::prompt_dsp::parse_prompt_rich(prompt);
    let analysis = cshot_lib::audio::analyze::analyze_audio(&samples, 44100, 1);
    let st = cshot_lib::audio::SoundType::from_str(&analysis.sound_type_hint);
    let pitch = analysis.pitch_estimate.unwrap_or(200.0);
    let base = cshot_lib::audio::resynthesize::params_for_sound_type(st, pitch, analysis.duration_ms);
    let params = ctrl.to_resynthesis_params(&base);
    let transformed = cshot_lib::audio::transform::transform_with_params(&samples, &params);

    cshot_lib::audio::write_wav(&output, &transformed, 44100)
        .expect("Failed to write WAV file");
    println!("Transformed: {} -> {}", path.display(), output.display());
}

fn cmd_benchmark() {
    println!("cShot Engine Benchmark");
    println!("=====================\n");

    use cshot_lib::audio::resynthesize;
    use cshot_lib::audio::SoundType;

    let types = [
        (SoundType::Kick, 60.0, 300.0),
        (SoundType::Snare, 200.0, 300.0),
        (SoundType::ClosedHat, 400.0, 150.0),
        (SoundType::OpenHat, 400.0, 500.0),
        (SoundType::Clap, 180.0, 300.0),
        (SoundType::Bass, 55.0, 600.0),
        (SoundType::Perc, 300.0, 200.0),
        (SoundType::Fx, 100.0, 1000.0),
    ];

    let mut total_samples = 0usize;
    let mut total_time = std::time::Duration::from_secs(0);

    for (i, (st, pitch, dur)) in types.iter().enumerate() {
        let params = resynthesize::params_for_sound_type(*st, *pitch, *dur);
        let start = Instant::now();
        let result = resynthesize::resynthesize(&params);
        let elapsed = start.elapsed();

        total_samples += result.len();
        total_time += elapsed;

        println!("  {:12} {:4.0}ms {:6} samples {:7.3}ms",
            format!("{:?}:", st),
            dur,
            result.len(),
            elapsed.as_secs_f64() * 1000.0,
        );

        if i == 3 {
            println!();
        }
    }

    println!();
    println!("  Total: {} samples in {:.3}ms",
        total_samples,
        total_time.as_secs_f64() * 1000.0,
    );
    println!("  Rate: {:.0} samples/sec",
        total_samples as f64 / total_time.as_secs_f64(),
    );
}
