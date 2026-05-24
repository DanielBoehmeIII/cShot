#![allow(
    clippy::collapsible_if,
    clippy::get_first,
    clippy::too_many_arguments,
    clippy::blocks_in_conditions,
)]

use std::path::PathBuf;
use std::time::Instant;
use cshot_lib::audio::resynthesize;
use cshot_lib::audio::SoundType;
use cshot_lib::audio::midi;
use cshot_lib::audio::humanize::{self, HumanizeParams};
use cshot_lib::audio::process;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct PluginPreset {
    name: String,
    sound_type: String,
    pitch_hz: f32,
    decay_ms: f32,
    brightness: f32,
    distortion: f32,
    noise_amount: f32,
    sub_gain: f32,
    click_amount: f32,
    body_gain: f32,
    pitch_drop_ratio: f32,
    attack_ms: f32,
    tail_ms: f32,
    duration_ms: f32,
    velocity_sensitivity: f32,
    humanize_analog: f32,
    humanize_amount: f32,
    tags: Vec<String>,
    category: String,
}

impl Default for PluginPreset {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            sound_type: "kick".to_string(),
            pitch_hz: 60.0,
            decay_ms: 200.0,
            brightness: 0.5,
            distortion: 0.0,
            noise_amount: 0.0,
            sub_gain: 0.3,
            click_amount: 0.4,
            body_gain: 0.7,
            pitch_drop_ratio: 0.6,
            attack_ms: 2.0,
            tail_ms: 50.0,
            duration_ms: 300.0,
            velocity_sensitivity: 0.5,
            humanize_analog: 0.0,
            humanize_amount: 0.0,
            tags: Vec::new(),
            category: "default".to_string(),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct AutomationPoint {
    param_index: usize,
    time_ms: f32,
    value: f32,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct AutomationClip {
    points: Vec<AutomationPoint>,
    loop_ms: f32,
}

fn parse_sound_type(name: &str) -> SoundType {
    match name {
        "kick" => SoundType::Kick,
        "snare" => SoundType::Snare,
        "hat" | "closed_hat" | "closed" | "ch" => SoundType::ClosedHat,
        "open_hat" | "open" | "oh" => SoundType::OpenHat,
        "clap" => SoundType::Clap,
        "bass" => SoundType::Bass,
        "perc" => SoundType::Perc,
        "fx" | "effect" => SoundType::Fx,
        "tom" => SoundType::Tom,
        _ => SoundType::Other,
    }
}

fn duration_for_type(st: SoundType) -> f32 {
    match st {
        SoundType::Kick => 300.0, SoundType::Snare => 350.0,
        SoundType::ClosedHat => 150.0, SoundType::OpenHat => 500.0,
        SoundType::Clap => 300.0, SoundType::Bass => 600.0,
        SoundType::Perc => 200.0, SoundType::Fx => 800.0,
        SoundType::Tom => 400.0, SoundType::Other => 300.0,
    }
}

fn preset_dir() -> PathBuf {
    std::env::var("CSHOT_PRESET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".cshot").join("presets")
        })
}

fn build_params(
    st: SoundType, pitch: f32, decay: f32, brightness: f32,
    distortion: f32, noise_amt: f32, sub_amt: f32, velocity: u8,
    preset: Option<&PluginPreset>,
) -> resynthesize::ResynthesisParams {
    let dur = duration_for_type(st);
    let pitch_hz = match st {
        SoundType::Kick | SoundType::Bass => pitch.clamp(30.0, 150.0),
        SoundType::Snare => pitch.clamp(150.0, 400.0),
        _ => pitch.clamp(100.0, 2000.0),
    };

    let mut p = if let Some(pr) = preset {
        let mut bp = resynthesize::params_for_sound_type(st, pr.pitch_hz, pr.duration_ms);
        bp.decay_ms = pr.decay_ms;
        bp.brightness = pr.brightness;
        bp.saturation_drive = 1.0 + pr.distortion * 2.0;
        bp.noise_amount = pr.noise_amount;
        bp.sub_gain = pr.sub_gain;
        bp.click_amount = pr.click_amount;
        bp.body_gain = pr.body_gain;
        bp.pitch_drop_ratio = pr.pitch_drop_ratio;
        bp.attack_ms = pr.attack_ms;
        bp.tail_ms = pr.tail_ms;
        bp.seed = velocity as u64;
        bp
    } else {
        let mut bp = resynthesize::params_for_sound_type(st, pitch_hz, dur);
        bp.decay_ms *= decay.max(0.1);
        bp.brightness = brightness.clamp(0.0, 1.0);
        bp.saturation_drive = 1.0 + distortion.clamp(0.0, 1.0) * 2.0;
        bp.noise_amount = noise_amt.clamp(0.0, 1.0);
        bp.sub_gain = sub_amt.clamp(0.0, 1.0);
        bp.seed = velocity as u64;
        bp
    };

    p = midi::params_for_velocity(&p, velocity);
    p
}

fn apply_automation(
    params: &resynthesize::ResynthesisParams,
    automation: &[AutomationClip],
    offset_ms: f32,
) -> resynthesize::ResynthesisParams {
    let mut p = params.clone();
    for clip in automation {
        if clip.points.is_empty() { continue; }
        let wrapped_time = if clip.loop_ms > 0.0 {
            offset_ms % clip.loop_ms
        } else {
            offset_ms
        };
        let mut sorted = clip.points.clone();
        sorted.sort_by(|a, b| a.time_ms.partial_cmp(&b.time_ms).unwrap_or(std::cmp::Ordering::Equal));
        let mut val = sorted[0].value;
        for i in 0..sorted.len() {
            if sorted[i].time_ms <= wrapped_time {
                val = sorted[i].value;
                if i + 1 < sorted.len() && sorted[i + 1].time_ms > wrapped_time {
                    let t = (wrapped_time - sorted[i].time_ms) / (sorted[i + 1].time_ms - sorted[i].time_ms).max(0.001);
                    val = sorted[i].value + (sorted[i + 1].value - sorted[i].value) * t;
                }
            }
        }
        match sorted[0].param_index {
            0 => p.pitch_hz = 20.0 + val * 1980.0,
            1 => p.saturation_drive = 1.0 + val * 4.0,
            2 => p.brightness = val,
            3 => p.decay_ms = 5.0 + val * 1000.0,
            4 => p.noise_amount = val,
            5 => p.sub_gain = val,
            6 => p.click_amount = val,
            7 => p.body_gain = val,
            _ => {}
        }
    }
    p
}

fn render_and_save(
    params: &resynthesize::ResynthesisParams,
    output_path: &str,
    print_info: bool,
    humanize_params: Option<&HumanizeParams>,
) -> Result<(), String> {
    let start = Instant::now();
    let mut samples = resynthesize::resynthesize(params);
    let elapsed = start.elapsed();

    if let Some(hp) = humanize_params {
        let seed = params.seed.wrapping_mul(params.duration_ms as u64);
        humanize::humanize(&mut samples, hp, seed);
    }

    let dsp_params = cshot_lib::audio::DspParams::default();
    process::process_sound(&mut samples, &dsp_params, params.sound_type);

    if print_info {
        let dur_ms = params.duration_ms;
        println!("  Duration:   {:.1}ms", dur_ms);
        println!("  Samples:    {}", samples.len());
        println!("  Time:       {:.3}ms", elapsed.as_secs_f64() * 1000.0);
        println!("  Rate:       {:.1}x realtime", dur_ms / (elapsed.as_secs_f64() * 1000.0).max(0.001) as f32);
    }

    let path = PathBuf::from(output_path);
    cshot_lib::audio::write_wav(&path, &samples, 44100)?;
    Ok(())
}

fn cmd_generate(args: &[String]) {
    let output = args.get(0).cloned().unwrap_or_else(|| "cshot_plugin_output.wav".to_string());
    let type_name = args.get(1).cloned().unwrap_or_else(|| "kick".to_string());
    let pitch: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(60.0);
    let decay: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.5);
    let brightness: f32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.5);
    let distortion: f32 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let noise_amt: f32 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let sub_amt: f32 = args.get(7).and_then(|s| s.parse().ok()).unwrap_or(0.3);
    let velocity: u8 = args.get(8).and_then(|s| s.parse().ok()).unwrap_or(100);
    let preset_name: Option<String> = args.get(9).cloned();
    let analog: f32 = args.get(10).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let human: f32 = args.get(11).and_then(|s| s.parse().ok()).unwrap_or(0.0);

    let st = parse_sound_type(&type_name);
    let mut preset: Option<PluginPreset> = None;
    if let Some(ref pn) = preset_name {
        let preset_path = if pn.ends_with(".json") { PathBuf::from(pn) } else { preset_dir().join(format!("{}.json", pn)) };
        if preset_path.exists() {
            if let Ok(json) = std::fs::read_to_string(&preset_path) {
                preset = serde_json::from_str(&json).ok();
            }
        }
    }

    println!("cShot Plugin v4 — Instrument Mode");
    println!("==================================");
    println!("Output:      {}", output);
    println!("Type:        {}", type_name);
    println!("Pitch:       {} Hz", pitch);
    println!("Decay:       {}", decay);
    println!("Brightness:  {}", brightness);
    println!("Distortion:  {}", distortion);
    println!("Noise:       {}", noise_amt);
    println!("Sub:         {}", sub_amt);
    println!("Velocity:    {}", velocity);
    println!("Analog:      {:.2}", analog);
    println!("Human:       {:.2}", human);
    if preset.is_some() { println!("Preset:      loaded"); }

    let params = build_params(st, pitch, decay, brightness, distortion, noise_amt, sub_amt, velocity, preset.as_ref());
    let hp = if analog > 0.0 || human > 0.0 {
        Some(HumanizeParams { analog_drift: analog * 0.3, instability: analog * 0.1, transient_randomness: human * 0.15, envelope_variation: human * 0.1, saturation_randomness: analog * 0.1, non_static_layers: (analog + human) * 0.5 * 0.15, phase_variation: analog * 0.1, humanize_transients: human * 0.2 })
    } else { None };

    match render_and_save(&params, &output, true, hp.as_ref()) {
        Ok(()) => println!("\nExported:    {}", output),
        Err(e) => eprintln!("\nExport error: {}", e),
    }
}

fn cmd_midi_trigger(args: &[String]) {
    let note: u8 = args.get(0).and_then(|s| s.parse().ok()).unwrap_or(36);
    let velocity: u8 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100);
    let output = args.get(2).cloned().unwrap_or_else(|| format!("midi_note_{}.wav", note));

    let (st, pitch) = midi::DrumNote::from_midi_note(note)
        .unwrap_or((SoundType::Perc, note as f32 * 8.0));

    println!("cShot MIDI Trigger — Note {} Velocity {}", note, velocity);
    println!("==================================");
    println!("Output:      {}", output);
    println!("Type:        {:?}", st);
    println!("Pitch:       {:.0} Hz", pitch);

    let params = build_params(st, pitch, 0.5, 0.5, 0.0, 0.0, 0.3, velocity, None);
    match render_and_save(&params, &output, true, None) {
        Ok(()) => println!("\nExported:    {}", output),
        Err(e) => eprintln!("\nExport error: {}", e),
    }
}

fn cmd_morph(args: &[String]) {
    let preset_a_name = args.get(0).cloned().unwrap_or_else(|| "preset_a.json".to_string());
    let preset_b_name = args.get(1).cloned().unwrap_or_else(|| "preset_b.json".to_string());
    let morph: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.5);
    let output = args.get(3).cloned().unwrap_or_else(|| "morphed.wav".to_string());
    let velocity: u8 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(100);

    let load_preset = |name: &str| -> Option<PluginPreset> {
        let path = PathBuf::from(name);
        if path.exists() {
            if let Ok(json) = std::fs::read_to_string(&path) {
                return serde_json::from_str(&json).ok();
            }
        }
        None
    };

    let preset_a = load_preset(&preset_a_name).unwrap_or_default();
    let preset_b = load_preset(&preset_b_name).unwrap_or_default();

    let st_a = parse_sound_type(&preset_a.sound_type);
    let st_b = parse_sound_type(&preset_b.sound_type);

    let params_a = resynthesize::params_for_sound_type(st_a, preset_a.pitch_hz, preset_a.duration_ms);
    let params_b = resynthesize::params_for_sound_type(st_b, preset_b.pitch_hz, preset_b.duration_ms);

    let morphed = midi::morph_params(&params_a, &params_b, morph);
    let final_params = midi::params_for_velocity(&morphed, velocity);

    println!("cShot Preset Morph");
    println!("==================");
    println!("From:        {} ({})", preset_a.name, preset_a.sound_type);
    println!("To:          {} ({})", preset_b.name, preset_b.sound_type);
    println!("Morph:       {:.0}%", morph * 100.0);
    println!("Velocity:    {}", velocity);
    println!("Output:      {}", output);

    match render_and_save(&final_params, &output, true, None) {
        Ok(()) => println!("\nExported:    {}", output),
        Err(e) => eprintln!("\nExport error: {}", e),
    }
}

fn cmd_batch(args: &[String]) {
    let notes: Vec<u8> = if args.len() > 1 {
        args[1..].iter().filter_map(|s| s.parse().ok()).collect()
    } else {
        vec![36, 38, 42, 46, 39, 41, 45, 48]
    };

    println!("cShot Batch — {} notes", notes.len());
    println!("==========================");

    for (i, &note) in notes.iter().enumerate() {
        let velocity = 100;
        let (st, pitch) = midi::DrumNote::from_midi_note(note)
            .unwrap_or((SoundType::Perc, note as f32 * 8.0));
        let params = build_params(st, pitch, 0.5, 0.5, 0.0, 0.0, 0.3, velocity, None);
        let output = format!("batch_{}_{}.wav", i + 1, st.as_str());
        println!("  [{}/{}] Note {} ({:?}) -> {}", i + 1, notes.len(), note, st, output);
        if let Err(e) = render_and_save(&params, &output, false, None) {
            eprintln!("    Error: {}", e);
        }
    }
    println!("\nDone. {} files generated.", notes.len());
}

fn cmd_presets(args: &[String]) {
    let sub = args.get(0).map(|s| s.as_str()).unwrap_or("list");
    match sub {
        "list" => {
            let dir = preset_dir();
            println!("cShot Presets [{}]", dir.display());
            println!("=========================");
            if dir.exists() {
                let mut count = 0;
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "json").unwrap_or(false) {
                            if let Ok(json) = std::fs::read_to_string(&path) {
                                if let Ok(preset) = serde_json::from_str::<PluginPreset>(&json) {
                                    println!("  {}: {} ({}) [{}]", path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default(), preset.name, preset.sound_type, preset.category);
                                    count += 1;
                                }
                            }
                        }
                    }
                }
                println!("\n{} presets found", count);
            } else {
                println!("No preset directory found");
            }
        }
        "save" => {
            if args.len() < 3 {
                eprintln!("Usage: presets save <name> <type> <pitch>");
                return;
            }
            let name = args.get(1).cloned().unwrap_or_default();
            let sound_type = args.get(2).cloned().unwrap_or_else(|| "kick".to_string());
            let pitch: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(60.0);

            let preset = PluginPreset {
                name: name.clone(),
                sound_type,
                pitch_hz: pitch,
                ..Default::default()
            };

            let dir = preset_dir();
            let _ = std::fs::create_dir_all(&dir);
            let path = dir.join(format!("{}.json", name));
            if let Ok(json) = serde_json::to_string_pretty(&preset) {
                if std::fs::write(&path, &json).is_ok() {
                    println!("Saved preset: {}", path.display());
                }
            }
        }
        "delete" => {
            let name = args.get(1).cloned().unwrap_or_default();
            let path = preset_dir().join(format!("{}.json", name));
            if path.exists() {
                if std::fs::remove_file(&path).is_ok() {
                    println!("Deleted preset: {}", name);
                }
            } else {
                eprintln!("Preset not found: {}", name);
            }
        }
        _ => {
            eprintln!("Usage: presets [list|save|delete]");
        }
    }
}

fn cmd_automate(args: &[String]) {
    let _output = args.get(0).cloned().unwrap_or_else(|| "automated.wav".to_string());
    let type_name = args.get(1).cloned().unwrap_or_else(|| "kick".to_string());
    let total_duration: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(2000.0);
    let note_interval: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(250.0);

    let st = parse_sound_type(&type_name);
    let pitch = 60.0;

    let automation = vec![
        AutomationClip {
            points: vec![
                AutomationPoint { param_index: 2, time_ms: 0.0, value: 0.2 },
                AutomationPoint { param_index: 2, time_ms: total_duration / 2.0, value: 0.9 },
                AutomationPoint { param_index: 2, time_ms: total_duration, value: 0.2 },
            ],
            loop_ms: 0.0,
        },
        AutomationClip {
            points: vec![
                AutomationPoint { param_index: 1, time_ms: 0.0, value: 0.0 },
                AutomationPoint { param_index: 1, time_ms: total_duration, value: 0.8 },
            ],
            loop_ms: 0.0,
        },
    ];

    let mut offset_ms = 0.0f32;
    let mut note_idx = 0;
    while offset_ms < total_duration {
        let velocity = 100;
        let mut params = build_params(st, pitch, 0.5, 0.5, 0.0, 0.0, 0.3, velocity, None);
        params = apply_automation(&params, &automation, offset_ms);
        let note_name = format!("automated_note_{}.wav", note_idx);
        if let Err(e) = render_and_save(&params, &note_name, false, None) {
            eprintln!("  Error at {}ms: {}", offset_ms as u32, e);
        }
        offset_ms += note_interval;
        note_idx += 1;
    }
    println!("\nGenerated {} automated notes", note_idx);
}

fn cmd_render_to_sample(args: &[String]) {
    let output = args.get(0).cloned().unwrap_or_else(|| "render.wav".to_string());
    let bpm: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(140.0);
    let bars: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let type_name = args.get(3).cloned().unwrap_or_else(|| "kick".to_string());

    let st = parse_sound_type(&type_name);
    let total_dur = (60.0 / bpm) * 4.0 * bars * 1000.0;
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let mut params = resynthesize::params_for_sound_type(st, 60.0, total_dur);
    params.seed = seed;
    params.decay_ms = (total_dur * 0.3).min(800.0);
    params.tail_ms = (total_dur * 0.2).min(400.0);

    let hp = HumanizeParams {
        analog_drift: 0.1,
        envelope_variation: 0.05,
        ..Default::default()
    };

    println!("cShot Render-to-Sample");
    println!("======================");
    println!("Output:      {}", output);
    println!("BPM:         {:.0}", bpm);
    println!("Bars:        {:.0}", bars);
    println!("Duration:    {:.0}ms", total_dur);
    println!("Type:        {}", type_name);

    match render_and_save(&params, &output, true, Some(&hp)) {
        Ok(()) => println!("\nExported:    {}", output),
        Err(e) => eprintln!("\nExport error: {}", e),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("cShot Plugin v4 — Real Instrument Behavior");
        println!("===========================================");
        println!();
        println!("Usage:");
        println!("  {} generate <output> <type> <pitch> <decay> <bright> <dist> <noise> <sub> [vel] [preset] [analog] [human]", args.get(0).unwrap_or(&"cshot-plugin".to_string()));
        println!("  {} midi <note> <velocity> [output]", args.get(0).unwrap_or(&"cshot-plugin".to_string()));
        println!("  {} morph <preset_a> <preset_b> <amount> [output] [velocity]", args.get(0).unwrap_or(&"cshot-plugin".to_string()));
        println!("  {} batch [notes...]", args.get(0).unwrap_or(&"cshot-plugin".to_string()));
        println!("  {} presets [list|save|delete] [name]", args.get(0).unwrap_or(&"cshot-plugin".to_string()));
        println!("  {} automate <output> <type> <duration_ms> <interval_ms>", args.get(0).unwrap_or(&"cshot-plugin".to_string()));
        println!("  {} render <output> <bpm> <bars> <type>", args.get(0).unwrap_or(&"cshot-plugin".to_string()));
        println!();
        println!("Presets stored in: {}", preset_dir().display());
        println!("DAW Parameters: 0=Pitch 1=Drive 2=Bright 3=Decay 4=Noise 5=Sub 6=Click 7=Body");
        return;
    }

    match args[1].as_str() {
        "generate" | "gen" => cmd_generate(&args[2..]),
        "midi" | "note" => cmd_midi_trigger(&args[2..]),
        "morph" => cmd_morph(&args[2..]),
        "batch" => cmd_batch(&args[2..]),
        "presets" | "preset" => cmd_presets(&args[2..]),
        "automate" | "auto" => cmd_automate(&args[2..]),
        "render" | "rtl" => cmd_render_to_sample(&args[2..]),
        _ => cmd_generate(&args[1..]),
    }
}
