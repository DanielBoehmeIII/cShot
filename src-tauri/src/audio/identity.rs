use super::SAMPLE_RATE;
use super::analyze::AudioAnalysis;

// ─── Identity & Messaging ──────────────────────────────

pub const APP_NAME: &str = "cShot";
pub const APP_TAGLINE: &str = "AI Promptable Sound Design";
pub const APP_VERSION: &str = "0.1.0-beta";
pub const APP_DESCRIPTION: &str = "cShot is a new kind of sound design instrument — promptable, recreatable, local-first.";

pub fn identity_statement() -> String {
    format!(
        r#"{name} {version} — {tagline}

{description}

What cShot IS:
  • A promptable sound design engine
  • A local-first audio instrument
  • A recreation & mutation tool
  • A creative collaborator for sound designers
  • An intelligent pack generator

What cShot is NOT:
  • Not a DAW
  • Not a spectral editor DAW replacement
  • Not a cloud-dependent service
  • Not a traditional sample library
  • Not a replacement for synthesis expertise — 
    it augments it

Why local-first matters:
  • Your sounds stay on your machine
  • No API keys needed for core features
  • Works offline
  • No tracking, no telemetry
  • Your taste model learns locally

Why promptable sound design matters:
  • Describe what you hear, not how to build it
  • Iterate faster than patch editing
  • Generate variations in seconds
  • Explore sonic territory you wouldn't find browsing menus

Why recreation & mutation matters:
  • Learn from reference sounds
  • Adapt existing material to new contexts
  • Genre-shift with one click
  • Hybridize two sounds into something new"#,
        name = APP_NAME,
        version = APP_VERSION,
        tagline = APP_TAGLINE,
        description = APP_DESCRIPTION
    )
}

// ─── Default Presets ──────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DefaultPreset {
    pub name: String,
    pub sound_type: String,
    pub description: String,
    pub tags: Vec<String>,
    pub params_hint: String,
}

pub fn default_presets() -> Vec<DefaultPreset> {
    vec![
        // Kicks
        DefaultPreset { name: "Deep Sub".into(), sound_type: "kick".into(), description: "Sub-heavy kick with soft click, long tail".into(), tags: vec!["sub".into(), "deep".into(), "warm".into()], params_hint: "sub_gain:0.8, click:0.3, decay:250ms".into() },
        DefaultPreset { name: "808 Classic".into(), sound_type: "kick".into(), description: "808-style boomy kick with pitch drop".into(), tags: vec!["808".into(), "boomy".into(), "trap".into()], params_hint: "pitch_drop:0.8, sub_gain:0.9, body_gain:0.6".into() },
        DefaultPreset { name: "Tight Punch".into(), sound_type: "kick".into(), description: "Short punchy kick with strong click".into(), tags: vec!["punchy".into(), "tight".into(), "electronic".into()], params_hint: "click:0.8, decay:120ms, attack:0.5ms".into() },
        DefaultPreset { name: "Techno Thump".into(), sound_type: "kick".into(), description: "Hard techno kick with distortion".into(), tags: vec!["techno".into(), "distorted".into(), "hard".into()], params_hint: "saturation:2.5, decay:180ms, body_gain:0.9".into() },
        DefaultPreset { name: "Soft Acoustic".into(), sound_type: "kick".into(), description: "Gentle acoustic-style kick".into(), tags: vec!["soft".into(), "acoustic".into(), "warm".into()], params_hint: "click:0.2, decay:300ms, brightness:0.3".into() },
        DefaultPreset { name: "Cinematic Boom".into(), sound_type: "kick".into(), description: "Booming cinematic kick with long tail".into(), tags: vec!["cinematic".into(), "big".into(), "epic".into()], params_hint: "tail:400ms, sub_gain:0.9, duration:600ms".into() },
        // Snares
        DefaultPreset { name: "Crack Snare".into(), sound_type: "snare".into(), description: "Bright snare with sharp crack".into(), tags: vec!["crack".into(), "bright".into(), "sharp".into()], params_hint: "noise:0.6, brightness:0.8, click:0.6".into() },
        DefaultPreset { name: "Trap Snare".into(), sound_type: "snare".into(), description: "Layered trap snare with body".into(), tags: vec!["trap".into(), "layered".into(), "fat".into()], params_hint: "noise:0.7, body_gain:0.6, decay:180ms".into() },
        DefaultPreset { name: "Rimshot".into(), sound_type: "snare".into(), description: "Tight rimshot-style snare".into(), tags: vec!["rim".into(), "tight".into(), "acoustic".into()], params_hint: "click:0.7, body_gain:0.3, brightness:0.7".into() },
        // Hi-hats
        DefaultPreset { name: "Tight Hat".into(), sound_type: "closed_hat".into(), description: "Short tight closed hi-hat".into(), tags: vec!["tight".into(), "short".into(), "bright".into()], params_hint: "noise:1.0, decay:60ms, brightness:0.9".into() },
        DefaultPreset { name: "Washy Open".into(), sound_type: "open_hat".into(), description: "Washy open hi-hat with long decay".into(), tags: vec!["open".into(), "washy".into(), "long".into()], params_hint: "noise:1.0, decay:300ms, brightness:0.8".into() },
        DefaultPreset { name: "Sizzle Hat".into(), sound_type: "closed_hat".into(), description: "Sizzling hi-hat with metallic character".into(), tags: vec!["sizzle".into(), "metallic".into(), "bright".into()], params_hint: "noise:1.0, brightness:0.95, saturation:1.3".into() },
        // Claps
        DefaultPreset { name: "Room Clap".into(), sound_type: "clap".into(), description: "Warm room clap with body".into(), tags: vec!["clap".into(), "warm".into(), "room".into()], params_hint: "body_gain:0.4, noise:0.8, decay:200ms".into() },
        DefaultPreset { name: "Layered Clap".into(), sound_type: "clap".into(), description: "Multi-hit layered clap".into(), tags: vec!["layered".into(), "big".into(), "electronic".into()], params_hint: "noise:0.9, body_gain:0.3, saturation:1.5".into() },
        // Bass
        DefaultPreset { name: "Deep 808".into(), sound_type: "bass".into(), description: "Deep 808 sub bass".into(), tags: vec!["808".into(), "sub".into(), "deep".into()], params_hint: "sub_gain:0.9, body_gain:0.8, pitch_drop:0.3".into() },
        DefaultPreset { name: "Distorted Bass".into(), sound_type: "bass".into(), description: "Aggressive distorted bass".into(), tags: vec!["distorted".into(), "aggressive".into(), "gritty".into()], params_hint: "saturation:3.0, body_gain:0.9, sub_gain:0.5".into() },
        // Percussion
        DefaultPreset { name: "Metallic Perc".into(), sound_type: "perc".into(), description: "Metallic FM percussion".into(), tags: vec!["metallic".into(), "perc".into(), "bright".into()], params_hint: "brightness:0.8, pitch:400Hz, noise:0.4".into() },
        DefaultPreset { name: "Wooden Perc".into(), sound_type: "perc".into(), description: "Wooden percussion hit".into(), tags: vec!["wooden".into(), "organic".into(), "warm".into()], params_hint: "brightness:0.3, noise:0.3, body_gain:0.6".into() },
        // FX
        DefaultPreset { name: "Cinematic Impact".into(), sound_type: "fx".into(), description: "Big cinematic impact".into(), tags: vec!["impact".into(), "cinematic".into(), "epic".into()], params_hint: "sub_gain:0.7, duration:1500ms, saturation:2.0".into() },
        DefaultPreset { name: "Riser Sweep".into(), sound_type: "fx".into(), description: "Rising sweep for builds".into(), tags: vec!["riser".into(), "sweep".into(), "build".into()], params_hint: "pitch_drop:0.0, noise:0.8, duration:2000ms".into() },
    ]
}

// ─── Quick Start Workflows ─────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WorkflowStep {
    pub action: String,
    pub description: String,
    pub example: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
}

pub fn quick_start_workflows() -> Vec<Workflow> {
    vec![
        Workflow {
            name: "Generate a Kick".to_string(),
            description: "Create your first kick drum sound".to_string(),
            steps: vec![
                WorkflowStep { action: "open".to_string(), description: "Open cShot Generator view".to_string(), example: "Launch the app".to_string() },
                WorkflowStep { action: "prompt".to_string(), description: "Type a prompt".to_string(), example: "\"punchy kick with sub 140bpm\"".to_string() },
                WorkflowStep { action: "generate".to_string(), description: "Click Generate".to_string(), example: "Wait ~100ms for synthesis".to_string() },
                WorkflowStep { action: "play".to_string(), description: "Preview the sound".to_string(), example: "Click the play button".to_string() },
                WorkflowStep { action: "variant".to_string(), description: "Try variants".to_string(), example: "Click \"brighter\" or \"punchier\"".to_string() },
            ],
        },
        Workflow {
            name: "Recreate a Sound".to_string(),
            description: "Upload a reference sound and recreate it".to_string(),
            steps: vec![
                WorkflowStep { action: "upload".to_string(), description: "Upload a WAV reference".to_string(), example: "Drag & drop or click to browse".to_string() },
                WorkflowStep { action: "analyze".to_string(), description: "Let cShot analyze it".to_string(), example: "Automatic analysis".to_string() },
                WorkflowStep { action: "recreate".to_string(), description: "Click Recreate".to_string(), example: "Generates multiple approximations".to_string() },
                WorkflowStep { action: "choose".to_string(), description: "Pick the best recreation".to_string(), example: "Sorted by similarity score".to_string() },
                WorkflowStep { action: "mutate".to_string(), description: "Try mutations".to_string(), example: "\"Make this cleaner\" or \"Shift to techno\"".to_string() },
            ],
        },
        Workflow {
            name: "Generate a Pack".to_string(),
            description: "Create a genre-consistent sound pack".to_string(),
            steps: vec![
                WorkflowStep { action: "choose-genre".to_string(), description: "Pick a genre".to_string(), example: "\"trap\", \"techno\", \"lo-fi\"".to_string() },
                WorkflowStep { action: "set-count".to_string(), description: "Set number of sounds".to_string(), example: "8-16 sounds".to_string() },
                WorkflowStep { action: "generate".to_string(), description: "Generate the pack".to_string(), example: "Balanced roles, cohesive energy".to_string() },
                WorkflowStep { action: "review".to_string(), description: "Review cohesion metrics".to_string(), example: "Role balance, duplicate detection".to_string() },
                WorkflowStep { action: "export".to_string(), description: "Export the pack".to_string(), example: "As ZIP or individual WAVs".to_string() },
            ],
        },
        Workflow {
            name: "Morph Between Sounds".to_string(),
            description: "Blend two presets into something new".to_string(),
            steps: vec![
                WorkflowStep { action: "pick-a".to_string(), description: "Select first sound".to_string(), example: "A kick".to_string() },
                WorkflowStep { action: "pick-b".to_string(), description: "Select second sound".to_string(), example: "A snare".to_string() },
                WorkflowStep { action: "morph".to_string(), description: "Adjust morph amount".to_string(), example: "50% blend".to_string() },
                WorkflowStep { action: "generate".to_string(), description: "Generate morphed sound".to_string(), example: "Hybrid parameters".to_string() },
            ],
        },
    ]
}

// ─── Package-Level Information ─────────────────────────

pub fn capability_summary() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Sound Types", "10 types: kick, snare, closed_hat, open_hat, clap, tom, perc, bass, fx, other"),
        ("Generation", "Prompt-driven synthesis via text description"),
        ("Recreation", "Analyze references -> generate approximations"),
        ("Mutation", "26+ mutation presets: genre-shift, evolve, exaggerate, clean, modernize"),
        ("Hybridization", "Blend two sounds into a morphed result"),
        ("Resynthesis", "5-layer engine: transient, body, noise, sub, tail"),
        ("Spectral Editing", "Isolate regions, remove mud, soften harshness, tilt spectrum"),
        ("Pack Generation", "Genre-consistent packs with cohesion metrics"),
        ("MIDI Support", "GM drum mapping, velocity sensitivity, pitch mapping"),
        ("Parameter Controls", "3 modes: Simple (5), Advanced (13), Sound Designer (21)"),
        ("Taste Learning", "Local-only personalization from usage patterns"),
        ("DSP Quality", "Analog saturation, lookahead limiter, oversampling-ready"),
        ("DAW Integration", "CLI plugin, ready for nih-plug VST3/CLAP wrapping"),
        ("Local-First", "All processing local. No cloud. No tracking."),
    ]
}

// ─── Sound Identity Intelligence ──────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SoundIdentity {
    pub transient_identity: TransientIdentity,
    pub tonal_identity: TonalIdentity,
    pub texture_identity: TextureIdentity,
    pub tail_identity: TailIdentity,
    pub aggressiveness: f32,
    pub density: f32,
    pub genre_role: String,
    pub mix_role: String,
    pub overall_character: Vec<String>,
    pub embedding: Vec<f32>,
    pub identity_fingerprint: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TransientIdentity {
    pub attack_sharpness: f32,
    pub click_character: String,
    pub transient_density: f32,
    pub transient_spectral_centroid: f32,
    pub attack_time_ms: f32,
    pub has_pre_attack: bool,
    pub transient_energy_ratio: f32,
    pub classification: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TonalIdentity {
    pub has_dominant_pitch: bool,
    pub pitch_hz: Option<f32>,
    pub tonal_noise_ratio: f32,
    pub harmonic_richness: f32,
    pub spectral_centroid: f32,
    pub brightness: f32,
    pub sub_energy: f32,
    pub low_mid_balance: f32,
    pub classification: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TextureIdentity {
    pub noise_character: String,
    pub texture_density: f32,
    pub noise_movement: f32,
    pub noise_floor_db: f32,
    pub spectral_flatness: f32,
    pub zero_crossing_rate: f32,
    pub classification: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TailIdentity {
    pub tail_length_ms: f32,
    pub decay_rate: f32,
    pub tail_character: String,
    pub has_resonance: bool,
    pub tail_texture_density: f32,
    pub tail_tonal_content: f32,
    pub tail_noise_content: f32,
    pub classification: String,
}

pub fn compute_sound_identity(analysis: &AudioAnalysis) -> SoundIdentity {
    let transient_identity = compute_transient_identity(analysis);
    let tonal_identity = compute_tonal_identity(analysis);
    let texture_identity = compute_texture_identity(analysis);
    let tail_identity = compute_tail_identity(analysis);
    let aggressiveness = compute_aggressiveness(analysis);
    let density = compute_density(analysis);
    let genre_role = genre_role_from_analysis(analysis);
    let mix_role = mix_role_from_analysis(analysis);
    let overall_character = compute_overall_character(&transient_identity, &tonal_identity, &texture_identity, &tail_identity, aggressiveness, density);
    let embedding = compute_identity_embedding(analysis, &transient_identity, &tonal_identity, &texture_identity, &tail_identity, aggressiveness, density);
    let identity_fingerprint = compute_identity_fingerprint(&embedding);

    SoundIdentity {
        transient_identity,
        tonal_identity,
        texture_identity,
        tail_identity,
        aggressiveness,
        density,
        genre_role,
        mix_role,
        overall_character,
        embedding,
        identity_fingerprint,
    }
}

fn compute_transient_identity(analysis: &AudioAnalysis) -> TransientIdentity {
    let sharpness = analysis.transient_sharpness;
    let click_char = analysis.click_character_hint.clone();
    let density = analysis.transient_density;
    let sc = analysis.transient_spectral_centroid;
    let attack = analysis.attack_ms;
    let has_pre = analysis.leading_silence_ms > 0.5;

    let transient_ratio = if analysis.peak > 0.001 {
        let onset_samples = (analysis.attack_ms / 1000.0 * SAMPLE_RATE as f32 * 2.0) as usize;
        let onset_samples = onset_samples.min(analysis.envelope.len() / 2).max(2);
        if analysis.envelope.len() > onset_samples * 2 {
            let onset_energy: f32 = analysis.envelope[..onset_samples].iter().sum();
            let total_energy: f32 = analysis.envelope.iter().sum();
            if total_energy > 0.0 { (onset_energy / total_energy).min(1.0) } else { 0.3 }
        } else { 0.3 }
    } else { 0.0 };

    let classification = if sharpness > 0.7 && attack < 2.0 {
        "ultra_fast".to_string()
    } else if sharpness > 0.5 && attack < 5.0 {
        "fast".to_string()
    } else if sharpness > 0.3 && attack < 15.0 {
        "moderate".to_string()
    } else {
        "slow".to_string()
    };

    TransientIdentity {
        attack_sharpness: sharpness,
        click_character: click_char,
        transient_density: density,
        transient_spectral_centroid: sc,
        attack_time_ms: attack,
        has_pre_attack: has_pre,
        transient_energy_ratio: transient_ratio,
        classification,
    }
}

fn compute_tonal_identity(analysis: &AudioAnalysis) -> TonalIdentity {
    let has_pitch = analysis.has_pitch;
    let pitch = analysis.pitch_estimate;
    let tonal_noise = 1.0 - analysis.noise_estimate;
    let sc = analysis.spectral_centroid;
    let brightness = analysis.brightness;
    let sub = analysis.sub_energy_ratio;

    let harmonic_richness = if has_pitch && sc > 200.0 {
        ((sc / pitch.unwrap_or(200.0)) / 10.0).min(1.0)
    } else {
        0.0
    };

    let low_mid = if sc > 100.0 {
        let low_end = sub;
        let mid = 1.0 - sub - brightness;
        if mid > 0.001 { (low_end / mid).min(1.0) } else { 0.5 }
    } else { 0.5 };

    let classification = if has_pitch && tonal_noise > 0.6 {
        "tonal_dominant".to_string()
    } else if has_pitch && tonal_noise > 0.3 {
        "mixed_tonal".to_string()
    } else if sc > 4000.0 && brightness > 0.6 {
        "bright_noise".to_string()
    } else if sub > 0.4 {
        "sub_heavy".to_string()
    } else {
        "noise_dominant".to_string()
    };

    TonalIdentity {
        has_dominant_pitch: has_pitch,
        pitch_hz: pitch,
        tonal_noise_ratio: tonal_noise,
        harmonic_richness,
        spectral_centroid: sc,
        brightness,
        sub_energy: sub,
        low_mid_balance: low_mid,
        classification,
    }
}

fn compute_texture_identity(analysis: &AudioAnalysis) -> TextureIdentity {
    let zcr = analysis.zero_crossing_rate;
    let noise_floor = analysis.noise_floor_db;
    let brightness = analysis.brightness;

    let classification = if zcr > 0.25 && brightness > 0.6 {
        "bright_noise".to_string()
    } else if zcr > 0.15 {
        "moderate_noise".to_string()
    } else if zcr > 0.08 {
        "low_noise".to_string()
    } else {
        "clean".to_string()
    };

    let movement = if analysis.transient_count > 2 {
        0.3 + (analysis.transient_count as f32 * 0.05).min(0.5)
    } else {
        analysis.noise_estimate * 0.3
    };

    let flatness = super::analyze::compute_spectral_flatness(&[analysis.peak, analysis.rms, brightness, zcr]);

    TextureIdentity {
        noise_character: classification.clone(),
        texture_density: analysis.transient_density * 0.5 + analysis.noise_estimate * 0.5,
        noise_movement: movement,
        noise_floor_db: noise_floor,
        spectral_flatness: flatness,
        zero_crossing_rate: zcr,
        classification,
    }
}

fn compute_tail_identity(analysis: &AudioAnalysis) -> TailIdentity {
    let tail = analysis.tail_ms;
    let decay = analysis.decay_ms;
    let noise = analysis.noise_estimate;

    let classification = if tail > 500.0 {
        "long".to_string()
    } else if tail > 150.0 {
        "medium".to_string()
    } else if tail > 30.0 {
        "short".to_string()
    } else {
        "none".to_string()
    };

    let has_res = decay > tail * 0.5 && tail > 50.0;
    let tail_tex = if tail > 100.0 { noise * 0.4 } else { noise * 0.1 };
    let tail_tonal = if tail > 50.0 && analysis.has_pitch { 0.3 } else { 0.1 };

    TailIdentity {
        tail_length_ms: tail,
        decay_rate: if decay > 0.0 { tail / decay.max(1.0) } else { 1.0 },
        tail_character: classification.clone(),
        has_resonance: has_res,
        tail_texture_density: tail_tex,
        tail_tonal_content: tail_tonal,
        tail_noise_content: noise,
        classification,
    }
}

fn compute_aggressiveness(analysis: &AudioAnalysis) -> f32 {
    let crest_norm = (analysis.crest_factor / 20.0).min(1.0);
    let transient_norm = (analysis.transient_strength / 10.0).min(1.0);
    let attack_norm = if analysis.attack_ms > 0.0 {
        1.0 - (analysis.attack_ms / 30.0).min(1.0)
    } else { 0.0 };
    let loudness_norm = (analysis.loudness_lufs + 60.0) / 60.0;
    crest_norm * 0.3 + transient_norm * 0.25 + attack_norm * 0.25 + loudness_norm.clamp(0.0, 1.0) * 0.2
}

fn compute_density(analysis: &AudioAnalysis) -> f32 {
    let transient_density = analysis.transient_density;
    let rms_density = (analysis.rms * 5.0).min(1.0);
    let noise_density = analysis.noise_estimate;
    let spectral_fullness = if analysis.spectral_centroid > 200.0 && analysis.spectral_centroid < 8000.0 {
        1.0 - (analysis.spectral_centroid - 2000.0).abs() / 8000.0
    } else { 0.5 };
    transient_density * 0.3 + rms_density * 0.25 + noise_density * 0.2 + spectral_fullness * 0.25
}

fn genre_role_from_analysis(analysis: &AudioAnalysis) -> String {
    match analysis.sound_type_hint.as_str() {
        "kick" => {
            if analysis.sub_energy_ratio > 0.4 { "808/trap_kick".to_string() }
            else if analysis.attack_ms < 3.0 && analysis.crest_factor > 10.0 { "punchy_kick".to_string() }
            else if analysis.duration_ms > 500.0 { "cinematic_kick".to_string() }
            else { "generic_kick".to_string() }
        }
        "snare" => {
            if analysis.noise_estimate > 0.6 && analysis.transient_count > 2 { "layered_snare".to_string() }
            else if analysis.transient_strength > 3.0 { "crack_snare".to_string() }
            else { "generic_snare".to_string() }
        }
        "closed_hat" => "tight_hat".to_string(),
        "open_hat" => "washy_hat".to_string(),
        "clap" => {
            if analysis.transient_count > 3 { "layered_clap".to_string() }
            else { "generic_clap".to_string() }
        }
        "bass" => {
            if analysis.sub_energy_ratio > 0.5 { "sub_bass".to_string() }
            else { "mid_bass".to_string() }
        }
        "fx" => {
            if analysis.duration_ms > 1500.0 { "cinematic_fx".to_string() }
            else { "short_fx".to_string() }
        }
        _ => format!("{}_general", analysis.sound_type_hint),
    }
}

fn mix_role_from_analysis(analysis: &AudioAnalysis) -> String {
    if analysis.crest_factor > 12.0 && analysis.transient_strength > 3.0 {
        "backbone".to_string()
    } else if analysis.crest_factor > 8.0 && analysis.rms > 0.15 {
        "groove_element".to_string()
    } else if analysis.noise_estimate > 0.6 && analysis.spectral_centroid > 4000.0 {
        "texture".to_string()
    } else if analysis.sub_energy_ratio > 0.4 {
        "low_foundation".to_string()
    } else if analysis.duration_ms > 1000.0 {
        "atmospheric".to_string()
    } else if analysis.transient_count > 2 {
        "rhythmic_element".to_string()
    } else {
        "accent".to_string()
    }
}

fn compute_overall_character(
    ti: &TransientIdentity,
    to: &TonalIdentity,
    tx: &TextureIdentity,
    ta: &TailIdentity,
    aggressiveness: f32,
    density: f32,
) -> Vec<String> {
    let mut chars = Vec::new();
    if aggressiveness > 0.7 { chars.push("aggressive".to_string()); }
    else if aggressiveness < 0.3 { chars.push("gentle".to_string()); }
    if density > 0.6 { chars.push("dense".to_string()); }
    else if density < 0.3 { chars.push("sparse".to_string()); }
    chars.push(ti.classification.clone());
    chars.push(to.classification.clone());
    chars.push(tx.classification.clone());
    chars.push(ta.classification.clone());
    if to.brightness > 0.6 { chars.push("bright".to_string()); }
    else if to.brightness < 0.3 { chars.push("dark".to_string()); }
    if to.sub_energy > 0.4 { chars.push("subby".to_string()); }
    if ta.tail_length_ms > 300.0 { chars.push("ambient_tail".to_string()); }
    if ta.tail_length_ms < 30.0 { chars.push("tight".to_string()); }
    if ti.click_character == "metallic" { chars.push("metallic".to_string()); }
    chars
}

fn compute_identity_embedding(
    _analysis: &AudioAnalysis,
    ti: &TransientIdentity,
    to: &TonalIdentity,
    tx: &TextureIdentity,
    ta: &TailIdentity,
    aggressiveness: f32,
    density: f32,
) -> Vec<f32> {
    vec![
        ti.attack_sharpness,
        ti.transient_density,
        (ti.transient_spectral_centroid / 10000.0).min(1.0),
        (ti.attack_time_ms / 50.0).min(1.0),
        ti.transient_energy_ratio,
        to.tonal_noise_ratio,
        to.harmonic_richness,
        (to.spectral_centroid / 10000.0).min(1.0),
        to.brightness,
        to.sub_energy,
        to.low_mid_balance,
        tx.texture_density,
        tx.noise_movement,
        (tx.noise_floor_db / -90.0).clamp(0.0, 1.0),
        tx.spectral_flatness,
        (ta.tail_length_ms / 2000.0).min(1.0),
        ta.decay_rate,
        ta.tail_texture_density,
        ta.tail_tonal_content,
        aggressiveness,
        density,
    ]
}

fn compute_identity_fingerprint(embedding: &[f32]) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for &v in embedding.iter() {
        let hash_val: u64 = (v * 1000.0) as u64;
        hash_val.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}

pub fn sound_identity_distance(a: &SoundIdentity, b: &SoundIdentity) -> f32 {
    if a.embedding.len() != b.embedding.len() { return 1.0; }
    let mut dist = 0.0f32;
    for i in 0..a.embedding.len() {
        let d = a.embedding[i] - b.embedding[i];
        dist += d * d;
    }
    (dist / a.embedding.len() as f32).sqrt()
}

pub fn find_similar_sounds(target: &SoundIdentity, candidates: &[SoundIdentity], top_k: usize) -> Vec<(usize, f32)> {
    let mut scored: Vec<(usize, f32)> = candidates.iter().enumerate()
        .map(|(i, c)| (i, sound_identity_distance(target, c)))
        .collect();
    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);
    scored
}
