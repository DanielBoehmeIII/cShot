/// cShot CLI — Headless sound synthesis and transformation.
/// Can be used standalone without the Tauri UI for testing, batch generation,
/// and eventual VST/plugin integration.
///
/// Usage:
///   cargo run --bin cshot-cli -- generate "punchy kick 140bpm" --output kick.wav
///   cargo run --bin cshot-cli -- analyze input.wav
///   cargo run --bin cshot-cli -- transform input.wav "darker shorter" --output transformed.wav
///   cargo run --bin cshot-cli -- benchmark

use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cshot-cli <command> [args...]");
        eprintln!("Commands:");
        eprintln!("  generate <prompt> [--output <path>]    Generate a one-shot");
        eprintln!("  analyze <file>                         Analyze a sound file");
        eprintln!("  transform <file> <prompt> [--output]    Transform a sound");
        eprintln!("  recreate <file> [--output] [--count N]  Recreate a sound");
        eprintln!("  render <prompt> [--count N] [--dir D]   Batch render variations");
        eprintln!("  list-recipes                            List built-in recipes");
        eprintln!("  benchmark                              Run synthesis benchmarks");
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
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

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
    if args.len() < 2 {
        eprintln!("Usage: cshot-cli render <prompt> [--count <N>] [--dir <directory>]");
        std::process::exit(1);
    }
    let prompt = &args[1];
    let count: usize = args.windows(2)
        .find(|w| w[0] == "--count")
        .and_then(|w| w.get(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    let dir = args.windows(2)
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
    println!("cShot Built-in Sound Types:");
    println!();
    let recipes = [
        ("kick", "Punchy kick drum with sub", 60.0),
        ("snare", "Snare with noise body and crack", 200.0),
        ("closed_hat", "Tight closed hi-hat", 400.0),
        ("open_hat", "Washy open hi-hat", 300.0),
        ("clap", "Layered clap with body", 180.0),
        ("tom", "Pitched tom drum", 120.0),
        ("perc", "FM percussion", 300.0),
        ("bass", "Sub bass hit", 55.0),
        ("fx", "Cinematic FX", 100.0),
    ];
    for (name, desc, pitch) in &recipes {
        println!("  {:15} {:40}  {} Hz", name, desc, pitch);
    }
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
