use crate::audio::{DspParams, resynthesize::ResynthesisParams};
use std::sync::LazyLock;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PromptDspControls {
    pub sound_type: String,
    pub sound_type_score: f32,
    pub attack_ms: Option<f32>,
    pub decay_ms: Option<f32>,
    pub tail_ms: Option<f32>,
    pub duration_ms: Option<f32>,
    pub pitch_hz: Option<f32>,
    pub pitch_drop_ratio: Option<f32>,
    pub noise_amount: Option<f32>,
    pub saturation_drive: Option<f32>,
    pub brightness: Option<f32>,
    pub sub_gain: Option<f32>,
    pub click_amount: Option<f32>,
    pub transient_boost: Option<f32>,
    pub body_gain: Option<f32>,
    // New advanced controls
    pub density: Option<f32>,           // 0.0 (thin/sparse) to 1.0 (dense/full)
    pub aggressiveness: Option<f32>,    // 0.0 (soft/gentle) to 1.0 (aggressive/harsh)
    pub warmth: Option<f32>,            // 0.0 (cold/sterile) to 1.0 (warm/rich)
    pub crunch: Option<f32>,            // 0.0 (smooth) to 1.0 (crunchy/gritty)
    pub texture: Option<f32>,           // 0.0 (smooth/clean) to 1.0 (textured/rough)
    pub stereo_width: Option<f32>,      // 0.0 (mono) to 1.0 (wide)
    pub tonal_noise_balance: Option<f32>, // 0.0 (tonal) to 1.0 (noisy)
    pub descriptors: Vec<DetectedDescriptor>,
    pub genre_hints: Vec<String>,
    pub bpm: Option<f32>,
    // Compound edit state
    pub compound_parts: Vec<CompoundEditPart>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompoundEditPart {
    pub text: String,
    pub descriptors: Vec<String>,
    pub is_exclusion: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DetectedDescriptor {
    pub word: String,
    pub category: String,
    pub confidence: f32,
    pub description: String,
}

impl PromptDspControls {
    pub fn to_dsp_params(&self) -> DspParams {
        let mut dsp = DspParams::default();
        if let Some(b) = self.brightness { if b > 0.6 { dsp.bright = true; } }
        if let Some(b) = self.brightness { if b < 0.3 { dsp.dark = true; } }
        if self.transient_boost.unwrap_or(0.0) > 0.0 || self.click_amount.unwrap_or(0.0) > 0.3 { dsp.punch = true; }
        if let Some(s) = self.saturation_drive { if s > 1.2 { dsp.gain = s; } }
        if let Some(n) = self.noise_amount { dsp.noise_amt = n; }
        if let Some(d) = self.decay_ms { dsp.decay_factor = (d / 200.0).clamp(0.3, 3.0); }
        if let Some(d) = self.duration_ms { let factor = d / 500.0; dsp.decay_factor = factor.clamp(0.3, 3.0); }
        if self.brightness.unwrap_or(0.5) > 0.7 { dsp.high_pass = true; }
        if self.brightness.unwrap_or(0.5) < 0.3 { dsp.low_pass = true; }
        dsp.bpm = self.bpm;
        dsp
    }

    pub fn to_resynthesis_params(&self, base: &ResynthesisParams) -> ResynthesisParams {
        let mut p = base.clone();
        if let Some(v) = self.attack_ms { p.attack_ms = v; }
        if let Some(v) = self.decay_ms { p.decay_ms = v; }
        if let Some(v) = self.tail_ms { p.tail_ms = v; }
        if let Some(v) = self.duration_ms { p.duration_ms = v; }
        if let Some(v) = self.pitch_hz { p.pitch_hz = v; }
        if let Some(v) = self.pitch_drop_ratio { p.pitch_drop_ratio = v; }
        if let Some(v) = self.noise_amount { p.noise_amount = v; }
        if let Some(v) = self.saturation_drive { p.saturation_drive = v; }
        if let Some(v) = self.brightness { p.brightness = v; }
        if let Some(v) = self.sub_gain { p.sub_gain = v; }
        if let Some(v) = self.click_amount { p.click_amount = v; }
        if let Some(v) = self.body_gain { p.body_gain = v; }

        // Apply new semantic controls
        if let Some(d) = self.density {
            let density_body = (d - 0.5) * 0.4;
            p.body_gain = (p.body_gain + density_body).clamp(0.1, 1.0);
            if d > 0.6 { p.saturation_drive = (p.saturation_drive + (d - 0.6) * 0.3).max(1.0); }
        }
        if let Some(a) = self.aggressiveness {
            p.click_amount = (p.click_amount + (a - 0.5) * 0.4).clamp(0.0, 1.0);
            p.saturation_drive = (p.saturation_drive + (a - 0.5) * 0.6).max(1.0);
            if a > 0.5 { p.attack_ms = (p.attack_ms * (1.0 - (a - 0.5) * 0.4)).max(0.3); }
        }
        if let Some(w) = self.warmth {
            let warmth_tilt = (w - 0.5) * 0.3;
            p.brightness = (p.brightness - warmth_tilt).clamp(0.0, 1.0);
            p.saturation_drive = (p.saturation_drive + (w - 0.5) * 0.3).max(1.0);
            if w > 0.5 { p.sub_gain = (p.sub_gain + (w - 0.5) * 0.15).clamp(0.0, 1.0); }
        }
        if let Some(c) = self.crunch {
            p.saturation_drive = (p.saturation_drive + c * 0.6).max(1.0);
            if c > 0.5 { p.noise_amount = (p.noise_amount + (c - 0.5) * 0.15).clamp(0.0, 1.0); }
        }
        if let Some(t) = self.texture {
            p.noise_amount = (p.noise_amount + (t - 0.5) * 0.2).clamp(0.0, 1.0);
            if t > 0.5 { p.saturation_drive = (p.saturation_drive + (t - 0.5) * 0.15).max(1.0); }
        }
        if let Some(_s) = self.stereo_width {
            // Stereo width is applied post-generation in dsp.rs
        }
        if let Some(t) = self.tonal_noise_balance {
            p.noise_amount = (t * 0.8).clamp(0.0, 1.0);
            p.body_gain = ((1.0 - t) * 0.8).clamp(0.1, 1.0);
        }
        p
    }
}

pub fn parse_prompt_rich(text: &str) -> PromptDspControls {
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    let (sound_type, sound_type_score) = classify_type_rich(&words);
    let compound_parts = parse_compound_edits(&lower);
    
    // Compute intensity modifiers from "very", "slightly", "extremely", etc.
    let intensity_scale = compute_intensity_modifiers(&words);
    
    let mut ctrl = PromptDspControls {
        sound_type,
        sound_type_score,
        attack_ms: None,
        decay_ms: None,
        tail_ms: None,
        duration_ms: None,
        pitch_hz: None,
        pitch_drop_ratio: None,
        noise_amount: None,
        saturation_drive: None,
        brightness: None,
        sub_gain: None,
        click_amount: None,
        transient_boost: None,
        body_gain: None,
        density: None,
        aggressiveness: None,
        warmth: None,
        crunch: None,
        texture: None,
        stereo_width: None,
        tonal_noise_balance: None,
        descriptors: Vec::new(),
        genre_hints: Vec::new(),
        bpm: None,
        compound_parts,
    };

    for word in &words {
        let bpm_str = word.trim_end_matches("bpm").trim_end_matches("BPM");
        if let Ok(bpm) = bpm_str.parse::<f32>() {
            if bpm >= 60.0 && bpm <= 200.0 { ctrl.bpm = Some(bpm); }
        }
    }

    // Track which descriptors have intensity modifiers preceding them
    let mut descriptor_intensities: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    for i in 0..words.len() {
        let word = words[i];
        if let Some(_mapped) = DESCRIPTOR_MAP.get(word) {
            if is_meaningful_descriptor(word) {
                let intensity = if i > 0 {
                    get_intensity_for_word(words[i - 1], &intensity_scale)
                } else {
                    1.0
                };
                descriptor_intensities.insert(word.to_string(), intensity);
            }
        }
    }

    for word in &words {
        if let Some(mapped) = DESCRIPTOR_MAP.get(word) {
            if !ctrl.descriptors.iter().any(|d| d.word == *word) {
                let intensity = descriptor_intensities.get(*word).copied().unwrap_or(1.0);
                let adj_confidence = (mapped.confidence * intensity).min(1.0);
                ctrl.descriptors.push(DetectedDescriptor {
                    word: word.to_string(),
                    category: mapped.category.to_string(),
                    confidence: adj_confidence,
                    description: mapped.description.to_string(),
                });
            }
            apply_descriptor_scaled(&mut ctrl, mapped, descriptor_intensities.get(*word).copied().unwrap_or(1.0));
        }
    }

    for word in &words {
        if let Some(genre) = GENRE_MAP.get(word) {
            if !ctrl.genre_hints.contains(&genre.to_string()) {
                ctrl.genre_hints.push(genre.to_string());
                apply_genre(&mut ctrl, genre);
            }
        }
    }

    // Apply compound edits (handle "but", "without", conflicting descriptors)
    apply_compound_edits(&mut ctrl);

    apply_genre_scaling(&mut ctrl);
    clamp_controls(&mut ctrl);
    ctrl
}

fn is_meaningful_descriptor(word: &str) -> bool {
    !matches!(word, "very" | "extremely" | "slightly" | "somewhat" | "barely" | "super" | "ultra" | "rather" | "quite" | "kind of" | "a bit" | "extra" | "less" | "more" | "not" | "no")
}

fn compute_intensity_modifiers(words: &[&str]) -> std::collections::HashMap<String, f32> {
    let mut map: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    for i in 0..words.len() {
        let mult = match words[i] {
            "extremely" | "insanely" | "incredibly" | "super" | "ultra" => 1.8,
            "very" | "really" | "especially" | "extra" | "highly" => 1.5,
            "quite" | "rather" | "fairly" | "noticeably" => 1.2,
            "somewhat" | "kind of" => 0.6,
            "slightly" | "barely" | "a bit" | "little" => 0.35,
            "less" => 0.5,
            "more" => 1.4,
            "not" | "no" => 0.0,
            _ => continue,
        };
        if i + 1 < words.len() {
            map.insert(words[i + 1].to_string(), mult);
        }
    }
    map
}

fn get_intensity_for_word(word: &str, intensity_map: &std::collections::HashMap<String, f32>) -> f32 {
    intensity_map.get(word).copied().unwrap_or(1.0)
}

fn apply_descriptor_scaled(ctrl: &mut PromptDspControls, mapping: &DescriptorMapping, intensity: f32) {
    if intensity <= 0.0 { return; }
    
    let original_apply = mapping.apply;
    
    // Save current values to apply with scaling
    let before_attack = ctrl.attack_ms;
    let before_decay = ctrl.decay_ms;
    let before_tail = ctrl.tail_ms;
    let before_duration = ctrl.duration_ms;
    let before_pitch = ctrl.pitch_hz;
    let before_noise = ctrl.noise_amount;
    let before_sat = ctrl.saturation_drive;
    let before_brightness = ctrl.brightness;
    let before_sub = ctrl.sub_gain;
    let before_click = ctrl.click_amount;
    let before_transient = ctrl.transient_boost;
    let before_body = ctrl.body_gain;
    
    (original_apply)(ctrl);
    
    let scale_opt = |before: Option<f32>, after: Option<f32>| -> Option<f32> {
        match (before, after) {
            (Some(b), Some(a)) => {
                let diff = a - b;
                Some(b + diff * intensity)
            }
            (None, Some(a)) => Some(a * intensity.min(1.0)),
            (Some(b), None) => Some(b),
            (None, None) => None,
        }
    };
    
    ctrl.attack_ms = scale_opt(before_attack, ctrl.attack_ms);
    ctrl.decay_ms = scale_opt(before_decay, ctrl.decay_ms);
    ctrl.tail_ms = scale_opt(before_tail, ctrl.tail_ms);
    ctrl.duration_ms = scale_opt(before_duration, ctrl.duration_ms);
    ctrl.pitch_hz = scale_opt(before_pitch, ctrl.pitch_hz);
    ctrl.noise_amount = scale_opt(before_noise, ctrl.noise_amount);
    ctrl.saturation_drive = scale_opt(before_sat, ctrl.saturation_drive);
    ctrl.brightness = scale_opt(before_brightness, ctrl.brightness);
    ctrl.sub_gain = scale_opt(before_sub, ctrl.sub_gain);
    ctrl.click_amount = scale_opt(before_click, ctrl.click_amount);
    ctrl.transient_boost = scale_opt(before_transient, ctrl.transient_boost);
    ctrl.body_gain = scale_opt(before_body, ctrl.body_gain);
}

fn classify_type_rich(words: &[&str]) -> (String, f32) {
    let mut scores: Vec<(&str, i32)> = vec![
        ("kick", 0), ("snare", 0), ("closed_hat", 0), ("open_hat", 0),
        ("clap", 0), ("tom", 0), ("perc", 0), ("bass", 0), ("fx", 0), ("other", 0),
    ];
    let keywords: Vec<(&str, Vec<(&str, i32)>)> = vec![
        ("kick", vec![("kick", 10), ("bass", 1)]),
        ("kicker", vec![("kick", 8)]),
        ("snare", vec![("snare", 10)]),
        ("clap", vec![("clap", 10)]),
        ("hat", vec![("closed_hat", 5), ("open_hat", 5)]),
        ("hi-hat", vec![("closed_hat", 5), ("open_hat", 5)]),
        ("hihat", vec![("closed_hat", 5), ("open_hat", 5)]),
        ("closed", vec![("closed_hat", 8)]),
        ("open", vec![("open_hat", 8)]),
        ("cymbal", vec![("open_hat", 4)]),
        ("ride", vec![("open_hat", 3)]),
        ("crash", vec![("fx", 5), ("open_hat", 3)]),
        ("tom", vec![("tom", 10)]),
        ("perc", vec![("perc", 8)]),
        ("bass", vec![("bass", 8), ("kick", 2)]),
        ("sub", vec![("bass", 6)]),
        ("fx", vec![("fx", 8)]),
        ("riser", vec![("fx", 8)]),
        ("impact", vec![("fx", 7), ("kick", 3)]),
        ("rim", vec![("perc", 6)]),
        ("rimshot", vec![("perc", 8)]),
        ("whoosh", vec![("fx", 9)]),
        ("sweep", vec![("fx", 7)]),
        ("boom", vec![("kick", 4), ("bass", 3)]),
        ("hit", vec![("perc", 4), ("kick", 2)]),
        ("thud", vec![("kick", 5)]),
        ("808", vec![("kick", 5), ("bass", 3)]),
        ("drill", vec![("bass", 4), ("kick", 2)]),
    ];
    for word in words {
        if let Some(mappings) = keywords.iter().find(|(k, _)| *k == *word) {
            for (st, score) in &mappings.1 {
                if let Some(entry) = scores.iter_mut().find(|(s, _)| *s == *st) {
                    entry.1 += score;
                }
            }
        }
    }
    let total: i32 = scores.iter().map(|(_, s)| s).sum();
    scores.sort_by(|a, b| b.1.cmp(&a.1));
    let best = scores[0];
    let confidence = if total > 0 { best.1 as f32 / total as f32 } else { 0.0 };
    (best.0.to_string(), confidence)
}

struct DescriptorMapping {
    category: &'static str,
    confidence: f32,
    description: &'static str,
    apply: fn(&mut PromptDspControls),
}

static DESCRIPTOR_MAP: LazyLock<HashMap<&'static str, DescriptorMapping>> = LazyLock::new(|| {
    let mut m: HashMap<&str, DescriptorMapping> = HashMap::new();

    // Transient / attack descriptors
    m.insert("punchy", DescriptorMapping { category: "transient", confidence: 0.9, description: "Sharper attack, transient boost", apply: |c| { c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.4); c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.2); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); }});
    m.insert("punch", DescriptorMapping { category: "transient", confidence: 0.9, description: "Sharper attack, more impact", apply: |c| { c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.3); c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) + 0.15); }});
    m.insert("crack", DescriptorMapping { category: "transient", confidence: 0.85, description: "Sharp crack attack", apply: |c| { c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.4); c.attack_ms = Some(c.attack_ms.unwrap_or(2.0).min(1.0)); }});
    m.insert("snap", DescriptorMapping { category: "transient", confidence: 0.8, description: "Snappy attack", apply: |c| { c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.3); c.attack_ms = Some(c.attack_ms.unwrap_or(2.0).min(1.0)); }});
    m.insert("click", DescriptorMapping { category: "transient", confidence: 0.8, description: "Click transient", apply: |c| { c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.5); c.attack_ms = Some(0.5); }});
    m.insert("soft", DescriptorMapping { category: "transient", confidence: 0.8, description: "Slower attack, softer transient", apply: |c| { c.attack_ms = Some(c.attack_ms.unwrap_or(2.0) + 5.0); c.click_amount = Some(c.click_amount.unwrap_or(0.3) * 0.3); c.transient_boost = Some(c.transient_boost.unwrap_or(0.3) * 0.3); }});
    m.insert("hard", DescriptorMapping { category: "transient", confidence: 0.85, description: "Stronger transient", apply: |c| { c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.5); c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.2); }});
    m.insert("aggressive", DescriptorMapping { category: "transient", confidence: 0.75, description: "Aggressive transient", apply: |c| { c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.6); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.3); }});

    // Spectral / timbre descriptors
    m.insert("bright", DescriptorMapping { category: "spectral", confidence: 0.9, description: "Emphasize high frequencies", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.3); }});
    m.insert("crisp", DescriptorMapping { category: "spectral", confidence: 0.85, description: "Crisp high end", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.25); }});
    m.insert("shiny", DescriptorMapping { category: "spectral", confidence: 0.75, description: "Shiny, bright", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.2); }});
    m.insert("dark", DescriptorMapping { category: "spectral", confidence: 0.9, description: "Emphasize low frequencies, roll off highs", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.3); }});
    m.insert("dull", DescriptorMapping { category: "spectral", confidence: 0.7, description: "Reduce high frequencies", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.2); }});
    m.insert("muffled", DescriptorMapping { category: "spectral", confidence: 0.7, description: "Muffled, dark", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.25); }});
    m.insert("warm", DescriptorMapping { category: "spectral", confidence: 0.8, description: "Warm, reduced highs", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.15); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); }});
    m.insert("airy", DescriptorMapping { category: "spectral", confidence: 0.7, description: "Airy high end", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.2); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.1); }});
    m.insert("metallic", DescriptorMapping { category: "spectral", confidence: 0.8, description: "Metallic resonance", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.2); c.pitch_hz = Some(c.pitch_hz.unwrap_or(200.0) * 1.5); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); }});
    m.insert("thin", DescriptorMapping { category: "spectral", confidence: 0.6, description: "Thin, lacking body", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.15); c.body_gain = Some(c.body_gain.unwrap_or(0.5) * 0.5); c.sub_gain = Some(0.0); }});

    // Distortion / saturation
    m.insert("distorted", DescriptorMapping { category: "distortion", confidence: 0.9, description: "Add distortion", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.8); }});
    m.insert("crunchy", DescriptorMapping { category: "distortion", confidence: 0.75, description: "Crunchy distortion", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.5); }});
    m.insert("gritty", DescriptorMapping { category: "distortion", confidence: 0.75, description: "Gritty texture", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.4); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.1); }});
    m.insert("clean", DescriptorMapping { category: "distortion", confidence: 0.8, description: "Clean, minimal distortion", apply: |c| { c.saturation_drive = Some(1.0); c.noise_amount = Some(0.0); }});
    m.insert("saturated", DescriptorMapping { category: "distortion", confidence: 0.85, description: "Saturated, warm drive", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.5); }});

    // Temporal / envelope
    m.insert("short", DescriptorMapping { category: "temporal", confidence: 0.9, description: "Short duration, fast decay", apply: |c| { c.duration_ms = Some(c.duration_ms.unwrap_or(300.0) * 0.5); c.decay_ms = Some(c.decay_ms.unwrap_or(200.0) * 0.4); c.tail_ms = Some(0.0); }});
    m.insert("long", DescriptorMapping { category: "temporal", confidence: 0.85, description: "Long duration, sustained tail", apply: |c| { c.duration_ms = Some(c.duration_ms.unwrap_or(300.0) * 2.0); c.tail_ms = Some(c.tail_ms.unwrap_or(50.0) + 200.0); }});
    m.insert("tight", DescriptorMapping { category: "temporal", confidence: 0.8, description: "Tight, controlled decay", apply: |c| { c.decay_ms = Some(c.decay_ms.unwrap_or(200.0) * 0.5); c.tail_ms = Some(0.0); }});
    m.insert("ring", DescriptorMapping { category: "temporal", confidence: 0.6, description: "Ringing sustain", apply: |c| { c.decay_ms = Some(c.decay_ms.unwrap_or(200.0) * 1.5); }});
    m.insert("fast", DescriptorMapping { category: "temporal", confidence: 0.7, description: "Fast decay", apply: |c| { c.decay_ms = Some(c.decay_ms.unwrap_or(200.0) * 0.3); }});

    // Low-end / sub
    m.insert("deep", DescriptorMapping { category: "sub", confidence: 0.8, description: "Deep sub frequencies", apply: |c| { c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.3); c.pitch_hz = Some(c.pitch_hz.unwrap_or(200.0) * 0.7); }});
    m.insert("sub", DescriptorMapping { category: "sub", confidence: 0.85, description: "Added sub bass", apply: |c| { c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.4); }});
    m.insert("subby", DescriptorMapping { category: "sub", confidence: 0.8, description: "Subby low end", apply: |c| { c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.3); c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.1); }});
    m.insert("boomy", DescriptorMapping { category: "sub", confidence: 0.7, description: "Boomy low end", apply: |c| { c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.2); c.decay_ms = Some(c.decay_ms.unwrap_or(200.0) * 1.3); }});
    m.insert("low", DescriptorMapping { category: "sub", confidence: 0.6, description: "Low frequencies emphasized", apply: |c| { c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.15); c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.1); }});

    // Noise/texture
    m.insert("noisy", DescriptorMapping { category: "noise", confidence: 0.8, description: "Add noise texture", apply: |c| { c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.3); }});
    m.insert("noise", DescriptorMapping { category: "noise", confidence: 0.7, description: "Noisy texture", apply: |c| { c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.2); }});

    // Cinematic / FX
    m.insert("cinematic", DescriptorMapping { category: "fx", confidence: 0.8, description: "Cinematic, long tail, low boom", apply: |c| { c.tail_ms = Some(c.tail_ms.unwrap_or(50.0) + 300.0); c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.25); c.duration_ms = Some(c.duration_ms.unwrap_or(300.0) * 2.5); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); }});
    m.insert("epic", DescriptorMapping { category: "fx", confidence: 0.7, description: "Epic, large sound", apply: |c| { c.tail_ms = Some(c.tail_ms.unwrap_or(50.0) + 400.0); c.duration_ms = Some(c.duration_ms.unwrap_or(300.0) * 3.0); c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.2); }});
    m.insert("massive", DescriptorMapping { category: "fx", confidence: 0.7, description: "Massive, large", apply: |c| { c.duration_ms = Some(c.duration_ms.unwrap_or(300.0) * 2.0); c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.3); c.tail_ms = Some(c.tail_ms.unwrap_or(50.0) + 200.0); }});

    // Body
    m.insert("fat", DescriptorMapping { category: "body", confidence: 0.75, description: "Fat, full body", apply: |c| { c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.25); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); }});
    m.insert("thick", DescriptorMapping { category: "body", confidence: 0.7, description: "Thick body", apply: |c| { c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.2); }});
    m.insert("heavy", DescriptorMapping { category: "body", confidence: 0.7, description: "Heavy, weighty", apply: |c| { c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.2); c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.15); }});

    // New expanded descriptors for semantic control
    m.insert("glossy", DescriptorMapping { category: "spectral", confidence: 0.75, description: "Glossy, polished high end", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.25); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.05); }});
    m.insert("vintage", DescriptorMapping { category: "character", confidence: 0.8, description: "Vintage character, warm saturation", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.3); c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.1); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.08); }});
    m.insert("analog", DescriptorMapping { category: "character", confidence: 0.75, description: "Analog warmth, gentle saturation", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.05); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.05); }});
    m.insert("digital", DescriptorMapping { category: "character", confidence: 0.7, description: "Clean digital, precise", apply: |c| { c.saturation_drive = Some(1.0); c.noise_amount = Some(0.0); c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.1); }});
    m.insert("tight", DescriptorMapping { category: "temporal", confidence: 0.85, description: "Tight, controlled envelope", apply: |c| { c.decay_ms = Some(c.decay_ms.unwrap_or(200.0) * 0.4); c.tail_ms = Some(0.0); c.attack_ms = Some(c.attack_ms.unwrap_or(2.0).min(1.0)); }});
    m.insert("boomy", DescriptorMapping { category: "sub", confidence: 0.8, description: "Boomy low end with resonance", apply: |c| { c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.3); c.decay_ms = Some(c.decay_ms.unwrap_or(200.0) * 1.4); c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.1); }});
    m.insert("round", DescriptorMapping { category: "spectral", confidence: 0.7, description: "Round, soft highs", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.15); c.attack_ms = Some(c.attack_ms.unwrap_or(2.0) + 2.0); }});
    m.insert("punchy", DescriptorMapping { category: "transient", confidence: 0.95, description: "Strong punch and impact", apply: |c| { c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.5); c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.25); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); }});
    m.insert("smooth", DescriptorMapping { category: "spectral", confidence: 0.75, description: "Smooth, rolled-off highs", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.15); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); }});
    m.insert("tinny", DescriptorMapping { category: "spectral", confidence: 0.65, description: "Thin, tinny sound", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.3); c.sub_gain = Some(0.0); c.body_gain = Some(c.body_gain.unwrap_or(0.5) * 0.3); }});
    m.insert("woody", DescriptorMapping { category: "spectral", confidence: 0.7, description: "Wooden, organic timbre", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.1); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.1); }});
    m.insert("hollow", DescriptorMapping { category: "spectral", confidence: 0.65, description: "Hollow, mid-scooped", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.15); c.body_gain = Some(c.body_gain.unwrap_or(0.5) * 0.5); }});
    m.insert("glass", DescriptorMapping { category: "spectral", confidence: 0.6, description: "Glassy, high resonance", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.35); c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.1); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); }});
    m.insert("sizzle", DescriptorMapping { category: "spectral", confidence: 0.65, description: "Sizzling high end", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.3); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.15); }});
    m.insert("dense", DescriptorMapping { category: "body", confidence: 0.7, description: "Dense, layered", apply: |c| { c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.2); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.15); }});

    // Advanced texture / density descriptors
    m.insert("dense", DescriptorMapping { category: "density", confidence: 0.85, description: "Dense, full, packed", apply: |c| { c.density = Some(c.density.unwrap_or(0.5) + 0.35); c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.15); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); }});
    m.insert("sparse", DescriptorMapping { category: "density", confidence: 0.75, description: "Sparse, minimal", apply: |c| { c.density = Some(c.density.unwrap_or(0.5) - 0.3); c.body_gain = Some(c.body_gain.unwrap_or(0.5) * 0.5); c.noise_amount = Some(0.0); }});
    m.insert("full", DescriptorMapping { category: "density", confidence: 0.8, description: "Full, rich", apply: |c| { c.density = Some(c.density.unwrap_or(0.5) + 0.25); c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.2); }});
    m.insert("thin", DescriptorMapping { category: "density", confidence: 0.7, description: "Thin, lacking body", apply: |c| { c.density = Some(c.density.unwrap_or(0.5) - 0.25); c.body_gain = Some(c.body_gain.unwrap_or(0.5) * 0.4); c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) * 0.3); }});

    // Aggressiveness descriptors
    m.insert("aggressive", DescriptorMapping { category: "aggressiveness", confidence: 0.9, description: "Aggressive, hard-hitting", apply: |c| { c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) + 0.4); c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.5); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.4); c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.2); }});
    m.insert("gentle", DescriptorMapping { category: "aggressiveness", confidence: 0.8, description: "Gentle, soft", apply: |c| { c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) - 0.35); c.transient_boost = Some(c.transient_boost.unwrap_or(0.3) * 0.3); c.attack_ms = Some(c.attack_ms.unwrap_or(2.0) + 5.0); c.saturation_drive = Some(1.0); }});
    m.insert("harsh", DescriptorMapping { category: "aggressiveness", confidence: 0.7, description: "Harsh, piercing", apply: |c| { c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) + 0.3); c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.25); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.3); }});
    m.insert("soft", DescriptorMapping { category: "aggressiveness", confidence: 0.8, description: "Soft, subdued", apply: |c| { c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) - 0.3); c.attack_ms = Some(c.attack_ms.unwrap_or(2.0) + 5.0); c.click_amount = Some(c.click_amount.unwrap_or(0.3) * 0.3); c.transient_boost = Some(c.transient_boost.unwrap_or(0.3) * 0.3); }});
    m.insert("intense", DescriptorMapping { category: "aggressiveness", confidence: 0.75, description: "Intense, powerful", apply: |c| { c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) + 0.35); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.35); c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.35); }});

    // Warmth descriptors (expanded)
    m.insert("warm", DescriptorMapping { category: "warmth", confidence: 0.9, description: "Warm, rich harmonics", apply: |c| { c.warmth = Some(c.warmth.unwrap_or(0.5) + 0.35); c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.15); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.15); c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.05); }});
    m.insert("cold", DescriptorMapping { category: "warmth", confidence: 0.7, description: "Cold, sterile", apply: |c| { c.warmth = Some(c.warmth.unwrap_or(0.5) - 0.3); c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.1); c.saturation_drive = Some(1.0); }});
    m.insert("rich", DescriptorMapping { category: "warmth", confidence: 0.75, description: "Rich, harmonically full", apply: |c| { c.warmth = Some(c.warmth.unwrap_or(0.5) + 0.25); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.1); }});
    m.insert("sterile", DescriptorMapping { category: "warmth", confidence: 0.65, description: "Sterile, clean, cold", apply: |c| { c.warmth = Some(c.warmth.unwrap_or(0.5) - 0.3); c.saturation_drive = Some(1.0); c.noise_amount = Some(0.0); }});

    // Crunch / grit descriptors
    m.insert("crunch", DescriptorMapping { category: "crunch", confidence: 0.8, description: "Crunchy distortion character", apply: |c| { c.crunch = Some(c.crunch.unwrap_or(0.5) + 0.4); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.5); c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) + 0.2); }});
    m.insert("crunchy", DescriptorMapping { category: "crunch", confidence: 0.85, description: "Crunchy, gritty", apply: |c| { c.crunch = Some(c.crunch.unwrap_or(0.5) + 0.35); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.4); }});
    m.insert("grit", DescriptorMapping { category: "crunch", confidence: 0.75, description: "Gritty texture", apply: |c| { c.crunch = Some(c.crunch.unwrap_or(0.5) + 0.3); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.3); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.08); }});
    m.insert("smooth", DescriptorMapping { category: "crunch", confidence: 0.8, description: "Smooth, no grit", apply: |c| { c.crunch = Some(c.crunch.unwrap_or(0.5) - 0.3); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.5).min(1.2)); c.noise_amount = Some(c.noise_amount.unwrap_or(0.3) * 0.5); }});

    // Texture descriptors
    m.insert("textured", DescriptorMapping { category: "texture", confidence: 0.8, description: "Textured, rough surface", apply: |c| { c.texture = Some(c.texture.unwrap_or(0.5) + 0.3); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.12); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); }});
    m.insert("rough", DescriptorMapping { category: "texture", confidence: 0.75, description: "Rough, unrefined", apply: |c| { c.texture = Some(c.texture.unwrap_or(0.5) + 0.35); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.15); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); }});
    m.insert("silky", DescriptorMapping { category: "texture", confidence: 0.6, description: "Silky, smooth", apply: |c| { c.texture = Some(c.texture.unwrap_or(0.5) - 0.3); c.noise_amount = Some(0.0); c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.1); }});
    m.insert("polished", DescriptorMapping { category: "texture", confidence: 0.7, description: "Polished, refined", apply: |c| { c.texture = Some(c.texture.unwrap_or(0.5) - 0.2); c.noise_amount = Some(c.noise_amount.unwrap_or(0.3) * 0.3); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.5).min(1.3)); }});

    // Stereo width descriptors
    m.insert("wide", DescriptorMapping { category: "stereo", confidence: 0.85, description: "Wide stereo image", apply: |c| { c.stereo_width = Some(c.stereo_width.unwrap_or(0.0) + 0.5); }});
    m.insert("mono", DescriptorMapping { category: "stereo", confidence: 0.8, description: "Mono, centered", apply: |c| { c.stereo_width = Some(0.0); }});
    m.insert("spacious", DescriptorMapping { category: "stereo", confidence: 0.7, description: "Spacious, wide", apply: |c| { c.stereo_width = Some(c.stereo_width.unwrap_or(0.0) + 0.35); }});
    m.insert("stereo", DescriptorMapping { category: "stereo", confidence: 0.75, description: "Stereo width", apply: |c| { c.stereo_width = Some(c.stereo_width.unwrap_or(0.0) + 0.3); }});

    // Tonal/noise balance descriptors
    m.insert("tonal", DescriptorMapping { category: "tonal_noise", confidence: 0.8, description: "Tonal, musical pitch content", apply: |c| { c.tonal_noise_balance = Some(c.tonal_noise_balance.unwrap_or(0.5) - 0.35); c.noise_amount = Some(c.noise_amount.unwrap_or(0.3) * 0.3); }});
    m.insert("noisy", DescriptorMapping { category: "tonal_noise", confidence: 0.85, description: "Noisy, less tonal", apply: |c| { c.tonal_noise_balance = Some(c.tonal_noise_balance.unwrap_or(0.5) + 0.35); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.3); }});
    m.insert("pure", DescriptorMapping { category: "tonal_noise", confidence: 0.7, description: "Pure tone, clean", apply: |c| { c.tonal_noise_balance = Some(0.0); c.noise_amount = Some(0.0); }});
    m.insert("sizzle", DescriptorMapping { category: "tonal_noise", confidence: 0.65, description: "Sizzling noise top", apply: |c| { c.tonal_noise_balance = Some(c.tonal_noise_balance.unwrap_or(0.5) + 0.2); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.15); c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.2); }});

    // Humanization / analog feel descriptors
    m.insert("analog", DescriptorMapping { category: "humanize", confidence: 0.85, description: "Analog warmth, subtle instability", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.15); }});
    m.insert("vintage", DescriptorMapping { category: "humanize", confidence: 0.8, description: "Vintage character, tape feel", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.25); c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.1); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.05); }});
    m.insert("human", DescriptorMapping { category: "humanize", confidence: 0.7, description: "Human feel, imperfect transients", apply: |c| { c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) - 0.1); c.attack_ms = Some(c.attack_ms.unwrap_or(2.0) + 1.0); }});
    m.insert("organic", DescriptorMapping { category: "humanize", confidence: 0.7, description: "Organic, natural feel", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.05); }});
    m.insert("natural", DescriptorMapping { category: "humanize", confidence: 0.65, description: "Natural, less synthetic", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.08); c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) - 0.1); }});
    m.insert("tape", DescriptorMapping { category: "humanize", confidence: 0.8, description: "Tape saturation, drift, warmth", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.35); c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.1); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.05); }});
    m.insert("breathing", DescriptorMapping { category: "humanize", confidence: 0.6, description: "Subtle amplitude modulation, living feel", apply: |c| { c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.05); }});
    m.insert("alive", DescriptorMapping { category: "humanize", confidence: 0.6, description: "Living, non-static sound", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.08); }});
    m.insert("raw", DescriptorMapping { category: "humanize", confidence: 0.65, description: "Raw, unpolished character", apply: |c| { c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.08); }});
    m.insert("loose", DescriptorMapping { category: "temporal", confidence: 0.6, description: "Loose, less rigid timing", apply: |c| { c.attack_ms = Some(c.attack_ms.unwrap_or(2.0) + 2.0); c.decay_ms = Some(c.decay_ms.unwrap_or(200.0) * 1.2); }});
    
    // ─── Week 205-208: New competitive descriptors ───
    m.insert("modern", DescriptorMapping { category: "character", confidence: 0.85, description: "Modern, clean, polished production sound", apply: |c| { c.decay_ms = Some(c.decay_ms.unwrap_or(200.0) * 0.7); c.tail_ms = Some(c.tail_ms.unwrap_or(50.0) * 0.5); c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.15); c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.1); c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.1); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.15); c.noise_amount = Some(c.noise_amount.unwrap_or(0.3) * 0.5); }});
    m.insert("huge", DescriptorMapping { category: "body", confidence: 0.8, description: "Huge, massive, larger than life", apply: |c| { c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.3); c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.25); c.tail_ms = Some(c.tail_ms.unwrap_or(50.0) + 200.0); c.duration_ms = Some(c.duration_ms.unwrap_or(300.0) * 1.8); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); }});
    m.insert("punchy", DescriptorMapping { category: "transient", confidence: 0.95, description: "Strong punch and impact", apply: |c| { c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.5); c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.25); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.2); }});
    m.insert("smooth", DescriptorMapping { category: "spectral", confidence: 0.75, description: "Smooth, rolled-off highs", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.15); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); }});
    m.insert("tinny", DescriptorMapping { category: "spectral", confidence: 0.65, description: "Thin, tinny sound", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.3); c.sub_gain = Some(0.0); c.body_gain = Some(c.body_gain.unwrap_or(0.5) * 0.3); }});
    m.insert("woody", DescriptorMapping { category: "spectral", confidence: 0.7, description: "Wooden, organic timbre", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) - 0.1); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.1); }});
    m.insert("hollow", DescriptorMapping { category: "spectral", confidence: 0.65, description: "Hollow, mid-scooped", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.15); c.body_gain = Some(c.body_gain.unwrap_or(0.5) * 0.5); }});
    m.insert("glass", DescriptorMapping { category: "spectral", confidence: 0.6, description: "Glassy, high resonance", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.35); c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.1); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.1); }});
    m.insert("sizzle", DescriptorMapping { category: "spectral", confidence: 0.65, description: "Sizzling high end", apply: |c| { c.brightness = Some(c.brightness.unwrap_or(0.5) + 0.3); c.noise_amount = Some(c.noise_amount.unwrap_or(0.0) + 0.15); }});
    m.insert("dense", DescriptorMapping { category: "body", confidence: 0.7, description: "Dense, layered", apply: |c| { c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.2); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.15); }});
    m.insert("crunch", DescriptorMapping { category: "crunch", confidence: 0.85, description: "Crunchy distortion character", apply: |c| { c.crunch = Some(c.crunch.unwrap_or(0.5) + 0.4); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.5); c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) + 0.2); }});
    m.insert("brutal", DescriptorMapping { category: "aggressiveness", confidence: 0.8, description: "Brutal, extreme aggression", apply: |c| { c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) + 0.5); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.7); c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.5); c.click_amount = Some(c.click_amount.unwrap_or(0.0) + 0.3); }});
    m.insert("delicate", DescriptorMapping { category: "aggressiveness", confidence: 0.7, description: "Delicate, gentle touch", apply: |c| { c.aggressiveness = Some(c.aggressiveness.unwrap_or(0.5) - 0.4); c.transient_boost = Some(c.transient_boost.unwrap_or(0.3) * 0.2); c.attack_ms = Some(c.attack_ms.unwrap_or(2.0) + 8.0); c.click_amount = Some(0.0); c.saturation_drive = Some(1.0); }});
    m.insert("powerful", DescriptorMapping { category: "body", confidence: 0.8, description: "Powerful, forceful presence", apply: |c| { c.body_gain = Some(c.body_gain.unwrap_or(0.5) + 0.25); c.sub_gain = Some(c.sub_gain.unwrap_or(0.0) + 0.2); c.transient_boost = Some(c.transient_boost.unwrap_or(0.0) + 0.3); c.saturation_drive = Some(c.saturation_drive.unwrap_or(1.0) + 0.15); }});

    m
});

static GENRE_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("trap", "trap");
    m.insert("house", "house");
    m.insert("techno", "techno");
    m.insert("drill", "drill");
    m.insert("dubstep", "dubstep");
    m.insert("lo-fi", "lo-fi");
    m.insert("lofi", "lo-fi");
    m.insert("ambient", "ambient");
    m.insert("cinematic", "cinematic");
    m.insert("electronic", "electronic");
    m.insert("dance", "dance");
    m.insert("hip-hop", "hip-hop");
    m.insert("rnb", "rnb");
    m.insert("jazz", "jazz");
    m.insert("rock", "rock");
    m.insert("metal", "metal");
    m.insert("orchestral", "orchestral");
    m.insert("synthwave", "synthwave");
    m.insert("vaporwave", "vaporwave");
    m.insert("phonk", "phonk");
    m.insert("footwork", "footwork");
    m.insert("garage", "garage");
    m.insert("industrial", "industrial");
    m.insert("hyperpop", "hyperpop");
    m.insert("breakbeat", "breakbeat");
    m.insert("future-bass", "future-bass");
    m
});

#[allow(dead_code)]
fn apply_descriptor(ctrl: &mut PromptDspControls, mapping: &DescriptorMapping) {
    (mapping.apply)(ctrl);
}

fn apply_genre(ctrl: &mut PromptDspControls, genre: &str) {
    match genre {
        "trap" => {
            ctrl.sub_gain = Some(ctrl.sub_gain.unwrap_or(0.0) + 0.35);
            ctrl.click_amount = Some(ctrl.click_amount.unwrap_or(0.0) + 0.2);
            ctrl.saturation_drive = Some(ctrl.saturation_drive.unwrap_or(1.0) + 0.2);
            ctrl.tail_ms = Some(ctrl.tail_ms.unwrap_or(50.0) + 50.0);
        }
        "drill" => {
            ctrl.tail_ms = Some(ctrl.tail_ms.unwrap_or(50.0) + 150.0);
            ctrl.sub_gain = Some(ctrl.sub_gain.unwrap_or(0.0) + 0.4);
            ctrl.pitch_drop_ratio = Some(ctrl.pitch_drop_ratio.unwrap_or(0.3) + 0.2);
            ctrl.saturation_drive = Some(ctrl.saturation_drive.unwrap_or(1.0) + 0.3);
        }
        "house" => {
            ctrl.click_amount = Some(ctrl.click_amount.unwrap_or(0.0) + 0.3);
            ctrl.decay_ms = Some(ctrl.decay_ms.unwrap_or(200.0) * 0.7);
            ctrl.body_gain = Some(ctrl.body_gain.unwrap_or(0.5) + 0.1);
            ctrl.brightness = Some(ctrl.brightness.unwrap_or(0.5) + 0.1);
        }
        "techno" => {
            ctrl.saturation_drive = Some(ctrl.saturation_drive.unwrap_or(1.0) + 0.3);
            ctrl.click_amount = Some(ctrl.click_amount.unwrap_or(0.0) + 0.2);
            ctrl.noise_amount = Some(ctrl.noise_amount.unwrap_or(0.0) + 0.1);
        }
        "lo-fi" => {
            ctrl.saturation_drive = Some(ctrl.saturation_drive.unwrap_or(1.0) + 0.2);
            ctrl.noise_amount = Some(ctrl.noise_amount.unwrap_or(0.0) + 0.15);
            ctrl.brightness = Some(ctrl.brightness.unwrap_or(0.5) - 0.15);
        }
        "cinematic" => {
            ctrl.tail_ms = Some(ctrl.tail_ms.unwrap_or(50.0) + 300.0);
            ctrl.sub_gain = Some(ctrl.sub_gain.unwrap_or(0.0) + 0.3);
            ctrl.duration_ms = Some(ctrl.duration_ms.unwrap_or(300.0) * 2.0);
        }
        "ambient" => {
            ctrl.tail_ms = Some(ctrl.tail_ms.unwrap_or(50.0) + 200.0);
            ctrl.noise_amount = Some(ctrl.noise_amount.unwrap_or(0.0) + 0.1);
            ctrl.saturation_drive = Some(1.0);
        }
        "dubstep" => {
            ctrl.saturation_drive = Some(ctrl.saturation_drive.unwrap_or(1.0) + 0.6);
            ctrl.sub_gain = Some(ctrl.sub_gain.unwrap_or(0.0) + 0.3);
            ctrl.transient_boost = Some(ctrl.transient_boost.unwrap_or(0.0) + 0.3);
        }
        _ => {}
    }
}

/// Get genre-specific interpretation multiplier for a descriptor.
/// Genres change how descriptors like "punchy" or "warm" are interpreted.
fn genre_multiplier(genre: &str, descriptor_category: &str) -> f32 {
    match genre {
        "trap" => match descriptor_category {
            "sub" => 1.4,
            "transient" => 1.2,
            "body" => 0.8,
            _ => 1.0,
        },
        "drill" => match descriptor_category {
            "sub" => 1.5,
            "transient" => 1.1,
            "distortion" => 1.2,
            _ => 1.0,
        },
        "house" => match descriptor_category {
            "transient" => 1.3,
            "temporal" => 0.8,
            "spectral" => 1.1,
            _ => 1.0,
        },
        "techno" => match descriptor_category {
            "distortion" => 1.3,
            "transient" => 1.1,
            "noise" => 1.2,
            _ => 1.0,
        },
        "lo-fi" => match descriptor_category {
            "distortion" => 1.2,
            "noise" => 1.3,
            "spectral" => 0.8,
            _ => 1.0,
        },
        "cinematic" => match descriptor_category {
            "sub" => 1.3,
            "temporal" => 1.4,
            "fx" => 1.3,
            _ => 1.0,
        },
        "dubstep" => match descriptor_category {
            "distortion" => 1.5,
            "sub" => 1.3,
            "transient" => 1.2,
            _ => 1.0,
        },
        "ambient" => match descriptor_category {
            "temporal" => 1.3,
            "noise" => 1.2,
            "spectral" => 0.7,
            _ => 1.0,
        },
        "dnb" | "drum and bass" => match descriptor_category {
            "transient" => 1.3,
            "temporal" => 0.8,
            "sub" => 1.2,
            _ => 1.0,
        },
        "phonk" => match descriptor_category {
            "distortion" => 1.3,
            "sub" => 1.2,
            "body" => 1.1,
            _ => 1.0,
        },
        _ => 1.0,
    }
}

/// Apply genre-specific scaling to all controls
fn apply_genre_scaling(ctrl: &mut PromptDspControls) {
    for genre in &ctrl.genre_hints {
        if let Some(v) = ctrl.sub_gain {
            ctrl.sub_gain = Some(v * genre_multiplier(genre, "sub"));
        }
        if let Some(v) = ctrl.transient_boost {
            ctrl.transient_boost = Some(v * genre_multiplier(genre, "transient"));
        }
        if let Some(v) = ctrl.saturation_drive {
            ctrl.saturation_drive = Some(1.0 + (v - 1.0) * genre_multiplier(genre, "distortion"));
        }
        if let Some(v) = ctrl.noise_amount {
            ctrl.noise_amount = Some(v * genre_multiplier(genre, "noise"));
        }
        if let Some(v) = ctrl.brightness {
            ctrl.brightness = Some((v * genre_multiplier(genre, "spectral")).min(1.0));
        }
        if let Some(v) = ctrl.body_gain {
            ctrl.body_gain = Some((v * genre_multiplier(genre, "body")).min(1.0));
        }
    }
}

/// Get semantic neighbors for a descriptor (descriptors that are related)
pub fn semantic_neighbors(word: &str) -> Vec<(&'static str, f32)> {
    match word {
        "punchy" => vec![("hard", 0.8), ("aggressive", 0.6), ("crack", 0.5), ("snap", 0.5)],
        "soft" => vec![("gentle", 0.8), ("smooth", 0.6), ("round", 0.5), ("warm", 0.4)],
        "bright" => vec![("crisp", 0.8), ("airy", 0.6), ("shiny", 0.7), ("glossy", 0.5)],
        "dark" => vec![("dull", 0.7), ("muffled", 0.6), ("warm", 0.5), ("boomy", 0.3)],
        "warm" => vec![("rich", 0.7), ("dark", 0.5), ("smooth", 0.6), ("round", 0.5), ("vintage", 0.5)],
        "aggressive" => vec![("punchy", 0.7), ("hard", 0.8), ("intense", 0.8), ("distorted", 0.5), ("gritty", 0.5)],
        "distorted" => vec![("crunchy", 0.8), ("gritty", 0.7), ("saturated", 0.6), ("aggressive", 0.5)],
        "clean" => vec![("digital", 0.6), ("pure", 0.5), ("crisp", 0.4)],
        "noisy" => vec![("gritty", 0.6), ("lo-fi", 0.5), ("textured", 0.5), ("raw", 0.4), ("sizzle", 0.4)],
        "deep" => vec![("sub", 0.8), ("boomy", 0.6), ("low", 0.7)],
        "metallic" => vec![("bright", 0.5), ("tinny", 0.4), ("glass", 0.5)],
        "tight" => vec![("short", 0.6), ("snap", 0.5), ("click", 0.4)],
        "vintage" => vec![("analog", 0.7), ("warm", 0.6), ("tape", 0.6), ("lo-fi", 0.3)],
        "analog" => vec![("vintage", 0.7), ("warm", 0.5), ("saturated", 0.4), ("tape", 0.5), ("organic", 0.4)],
        "digital" => vec![("clean", 0.6), ("crisp", 0.4), ("precise", 0.3)],
        "organic" => vec![("natural", 0.7), ("human", 0.5), ("analog", 0.4), ("warm", 0.3)],
        "natural" => vec![("organic", 0.7), ("human", 0.6), ("analog", 0.3)],
        "human" => vec![("natural", 0.6), ("organic", 0.5), ("loose", 0.4)],
        "tape" => vec![("analog", 0.6), ("vintage", 0.6), ("warm", 0.5), ("saturated", 0.4)],
        "dense" => vec![("full", 0.8), ("thick", 0.7), ("fat", 0.6), ("heavy", 0.5)],
        "sparse" => vec![("thin", 0.7), ("clean", 0.4), ("minimal", 0.6)],
        "gentle" => vec![("soft", 0.8), ("smooth", 0.7), ("round", 0.5), ("warm", 0.4)],
        "cold" => vec![("sterile", 0.8), ("clean", 0.6), ("digital", 0.5), ("bright", 0.4)],
        "crunch" => vec![("gritty", 0.8), ("distorted", 0.7), ("crunchy", 0.9), ("aggressive", 0.5)],
        "textured" => vec![("rough", 0.8), ("gritty", 0.6), ("noisy", 0.5), ("complex", 0.4)],
        "smooth" => vec![("silky", 0.7), ("polished", 0.6), ("clean", 0.5), ("soft", 0.4)],
        "wide" => vec![("spacious", 0.8), ("stereo", 0.7), ("big", 0.4)],
        "tonal" => vec![("pure", 0.7), ("clean", 0.5), ("musical", 0.6)],
        "harsh" => vec![("aggressive", 0.7), ("bright", 0.6), ("piercing", 0.8), ("metallic", 0.4)],
        "rich" => vec![("warm", 0.7), ("full", 0.8), ("dense", 0.6), ("fat", 0.5)],
        _ => vec![],
    }
}

/// Compute descriptor intensity based on how many related words are present
pub fn compute_descriptor_intensity(descriptors: &[DetectedDescriptor], target_word: &str) -> f32 {
    let mut intensity = 0.0f32;
    let mut count = 0;
    for desc in descriptors {
        if desc.word == target_word {
            intensity += desc.confidence;
            count += 1;
        }
        let neighbors = semantic_neighbors(target_word);
        for (neighbor, weight) in &neighbors {
            if desc.word == *neighbor {
                intensity += desc.confidence * weight;
                count += 1;
            }
        }
    }
    if count > 0 { (intensity / count as f32).min(1.0) } else { 0.0 }
}

fn parse_compound_edits(lower: &str) -> Vec<CompoundEditPart> {
    let mut parts = Vec::new();
    let separators = [" but ", " AND ", " without ", " however ", " except ", " minus "];
    for sep in &separators {
        if lower.contains(sep) {
            let segments: Vec<&str> = lower.split(sep).collect();
            if segments.len() >= 2 {
                for (idx, seg) in segments.iter().enumerate() {
                    let seg_lower = seg.to_lowercase();
                    let words: Vec<&str> = seg_lower.split_whitespace().collect();
                    let mut descs = Vec::new();
                    for w in &words {
                        if DESCRIPTOR_MAP.contains_key(w) || GENRE_MAP.contains_key(w) {
                            descs.push(w.to_string());
                        }
                    }
                    if !descs.is_empty() {
                        let is_exclusion = sep.trim() == "without"
                            || sep.trim() == "except"
                            || sep.trim() == "minus"
                            || seg_lower.contains("less")
                            || seg_lower.contains("without")
                            || (idx > 0 && seg_lower.contains("but not"))
                            || (idx > 0 && seg_lower.contains("no "))
                            || (idx > 0 && seg_lower.contains("not "));
                        parts.push(CompoundEditPart {
                            text: seg.trim().to_string(),
                            descriptors: descs,
                            is_exclusion,
                        });
                    }
                }
            }
        }
    }
    parts
}

fn apply_compound_edits(ctrl: &mut PromptDspControls) {
    if ctrl.compound_parts.is_empty() { return; }

    let mut has_exclusion = false;
    let mut exclusion_descriptors: Vec<String> = Vec::new();

    for part in &ctrl.compound_parts {
        if part.is_exclusion {
            has_exclusion = true;
            for d in &part.descriptors {
                exclusion_descriptors.push(d.clone());
            }
        }
    }

    if has_exclusion {
        for excl in &exclusion_descriptors {
            match excl.as_str() {
                "harsh" | "harshness" => {
                    ctrl.brightness = ctrl.brightness.map(|b| (b - 0.2).max(0.0));
                    ctrl.saturation_drive = ctrl.saturation_drive.map(|s| (s - 0.2).max(1.0));
                }
                "thin" | "thinness" => {
                    ctrl.body_gain = ctrl.body_gain.map(|b| (b + 0.15).min(1.0));
                    ctrl.sub_gain = ctrl.sub_gain.map(|s| (s + 0.1).min(1.0));
                }
                "mud" | "muddy" | "muddiness" => {
                    if let Some(d) = ctrl.density { ctrl.density = Some((d - 0.15).max(0.0)); }
                    ctrl.brightness = ctrl.brightness.map(|b| (b + 0.1).min(1.0));
                }
                "body" => {
                    let b = ctrl.body_gain.unwrap_or(0.5);
                    ctrl.body_gain = Some((b + 0.2).min(1.0));
                }
                "punch" | "punchiness" => {
                    let p = ctrl.transient_boost.unwrap_or(0.5);
                    ctrl.transient_boost = Some((p + 0.15).max(0.0));
                }
                "distortion" | "distorted" => {
                    ctrl.saturation_drive = ctrl.saturation_drive.map(|s| (s - 0.4).max(1.0));
                }
                "noise" | "noisy" => {
                    ctrl.noise_amount = ctrl.noise_amount.map(|n| (n - 0.2).max(0.0));
                }
                "brightness" | "bright" => {
                    ctrl.brightness = ctrl.brightness.map(|b| (b - 0.15).max(0.0));
                }
                _ => {
                    // Generic reversal: undo the effect of the excluded descriptor
                    if let Some(mapped) = DESCRIPTOR_MAP.get(excl.as_str()) {
                        let mut reverse_map = PromptDspControls::default();
                        (mapped.apply)(&mut reverse_map);
                        let neutralizer = |val: Option<f32>, rev: Option<f32>| -> Option<f32> {
                            match (val, rev) {
                                (Some(v), Some(r)) => Some((v - (r - 0.5) * 0.7).clamp(0.0, if r > 1.0 { 5.0 } else { 1.0 })),
                                _ => val,
                            }
                        };
                        ctrl.attack_ms = neutralizer(ctrl.attack_ms, reverse_map.attack_ms);
                        ctrl.decay_ms = neutralizer(ctrl.decay_ms, reverse_map.decay_ms);
                        ctrl.noise_amount = neutralizer(ctrl.noise_amount, reverse_map.noise_amount);
                        ctrl.saturation_drive = neutralizer(ctrl.saturation_drive, reverse_map.saturation_drive);
                        ctrl.brightness = neutralizer(ctrl.brightness, reverse_map.brightness);
                        ctrl.sub_gain = neutralizer(ctrl.sub_gain, reverse_map.sub_gain);
                        ctrl.click_amount = neutralizer(ctrl.click_amount, reverse_map.click_amount);
                        ctrl.transient_boost = neutralizer(ctrl.transient_boost, reverse_map.transient_boost);
                        ctrl.body_gain = neutralizer(ctrl.body_gain, reverse_map.body_gain);
                    }
                }
            }
        }
    }

    // ─── Vote-based conflict resolution ───
    // Instead of winner-takes-all, compute net effect per parameter
    let mut _conflict_type: Option<(&'static str, &'static str)> = None;
    let descriptor_set: std::collections::HashSet<String> = ctrl.descriptors.iter().map(|d| d.word.clone()).collect();

    let conflicting_pairs: [(&str, &str); 11] = [
        ("bright", "dark"),
        ("punchy", "soft"),
        ("aggressive", "gentle"),
        ("clean", "distorted"),
        ("tight", "boomy"),
        ("thin", "fat"),
        ("crisp", "muffled"),
        ("noisy", "clean"),
        ("harsh", "smooth"),
        ("wet", "dry"),
        ("deep", "thin"),
    ];

    for &(a, b) in conflicting_pairs.iter() {
        if descriptor_set.contains(a) && descriptor_set.contains(b) {
            _conflict_type = Some((a, b));
            let a_conf = ctrl.descriptors.iter().find(|d| d.word == a).map(|d| d.confidence).unwrap_or(0.5);
            let b_conf = ctrl.descriptors.iter().find(|d| d.word == b).map(|d| d.confidence).unwrap_or(0.5);
            let net_vote = a_conf - b_conf;
            
            // Compute blended effect based on net vote
            let mut blend = PromptDspControls::default();
            let a_mapped = DESCRIPTOR_MAP.get(a);
            let b_mapped = DESCRIPTOR_MAP.get(b);
            
            if net_vote.abs() > 0.1 {
                // Clear previous applications of these descriptors
                ctrl.descriptors.retain(|d| d.word != a && d.word != b);
                
                let winner = if net_vote > 0.0 { a } else { b };
                let winner_conf = net_vote.abs();
                if let Some(wm) = if net_vote > 0.0 { a_mapped } else { b_mapped } {
                    ctrl.descriptors.push(DetectedDescriptor {
                        word: winner.to_string(),
                        category: wm.category.to_string(),
                        confidence: winner_conf.clamp(0.0, 1.0),
                        description: wm.description.to_string(),
                    });
                    for &(excl_a, excl_b) in conflicting_pairs.iter() {
                        if (excl_a == a && excl_b == b) || (excl_a == b && excl_b == a) {
                            break;
                        }
                    }
                    let apply_scale = winner_conf.clamp(0.2, 1.0);
                    (wm.apply)(&mut blend);
                    let scale_opt = |b: Option<f32>, a: Option<f32>| -> Option<f32> {
                        match (b, a) {
                            (Some(bv), Some(av)) => Some(bv + (av - bv) * apply_scale),
                            (_, Some(av)) => Some(av * apply_scale),
                            _ => None,
                        }
                    };
                    ctrl.attack_ms = scale_opt(ctrl.attack_ms, blend.attack_ms);
                    ctrl.decay_ms = scale_opt(ctrl.decay_ms, blend.decay_ms);
                    ctrl.tail_ms = scale_opt(ctrl.tail_ms, blend.tail_ms);
                    ctrl.noise_amount = scale_opt(ctrl.noise_amount, blend.noise_amount);
                    ctrl.saturation_drive = scale_opt(ctrl.saturation_drive, blend.saturation_drive);
                    ctrl.brightness = scale_opt(ctrl.brightness, blend.brightness);
                    ctrl.sub_gain = scale_opt(ctrl.sub_gain, blend.sub_gain);
                    ctrl.click_amount = scale_opt(ctrl.click_amount, blend.click_amount);
                    ctrl.transient_boost = scale_opt(ctrl.transient_boost, blend.transient_boost);
                    ctrl.body_gain = scale_opt(ctrl.body_gain, blend.body_gain);
                }
            }
        }
    }
}

fn clamp_controls(ctrl: &mut PromptDspControls) {
    if let Some(v) = ctrl.attack_ms { ctrl.attack_ms = Some(v.clamp(0.1, 100.0)); }
    if let Some(v) = ctrl.decay_ms { ctrl.decay_ms = Some(v.clamp(5.0, 2000.0)); }
    if let Some(v) = ctrl.tail_ms { ctrl.tail_ms = Some(v.clamp(0.0, 3000.0)); }
    if let Some(v) = ctrl.duration_ms { ctrl.duration_ms = Some(v.clamp(10.0, 5000.0)); }
    if let Some(v) = ctrl.pitch_hz { ctrl.pitch_hz = Some(v.clamp(20.0, 8000.0)); }
    if let Some(v) = ctrl.pitch_drop_ratio { ctrl.pitch_drop_ratio = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.noise_amount { ctrl.noise_amount = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.saturation_drive { ctrl.saturation_drive = Some(v.clamp(1.0, 5.0)); }
    if let Some(v) = ctrl.brightness { ctrl.brightness = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.sub_gain { ctrl.sub_gain = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.click_amount { ctrl.click_amount = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.transient_boost { ctrl.transient_boost = Some(v.clamp(0.0, 2.0)); }
    if let Some(v) = ctrl.body_gain { ctrl.body_gain = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.density { ctrl.density = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.aggressiveness { ctrl.aggressiveness = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.warmth { ctrl.warmth = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.crunch { ctrl.crunch = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.texture { ctrl.texture = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.stereo_width { ctrl.stereo_width = Some(v.clamp(0.0, 1.0)); }
    if let Some(v) = ctrl.tonal_noise_balance { ctrl.tonal_noise_balance = Some(v.clamp(0.0, 1.0)); }
}

// ─── Prompt Mutation Engine ───────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PromptMutation {
    pub edit_text: String,
    pub intensity: f32,
    pub mutations: Vec<String>,
    pub preserve_identity: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MutationResult {
    pub original_params: ResynthesisParams,
    pub mutated_params: ResynthesisParams,
    pub edits_applied: Vec<String>,
    pub identity_preservation: f32,
    pub mutation_description: String,
}

static PROMPT_MUTATION_MAP: LazyLock<HashMap<&'static str, PromptMutationDef>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("harder", PromptMutationDef {
        saturation_delta: (0.4, 0.6),
        click_delta: (0.15, 0.25),
        attack_scale: (0.6, 0.8),
        body_delta: (0.05, 0.1),
        sub_delta: (0.0, 0.05),
        noise_delta: (0.0, 0.05),
        brightness_delta: (0.0, 0.1),
        decay_scale: (0.7, 0.9),
        tail_scale: (0.7, 0.9),
        preserve_core: true,
        description: "Increases aggression while preserving sound identity",
    });
    m.insert("cleaner", PromptMutationDef {
        saturation_delta: (-0.4, -0.2),
        click_delta: (0.0, 0.05),
        attack_scale: (1.0, 1.0),
        body_delta: (0.0, 0.0),
        sub_delta: (0.0, 0.0),
        noise_delta: (-0.3, -0.15),
        brightness_delta: (0.05, 0.15),
        decay_scale: (0.7, 0.9),
        tail_scale: (0.6, 0.8),
        preserve_core: true,
        description: "Reduces noise and saturation while keeping the original character",
    });
    m.insert("warmer", PromptMutationDef {
        saturation_delta: (0.15, 0.3),
        click_delta: (-0.05, 0.0),
        attack_scale: (1.0, 1.1),
        body_delta: (0.05, 0.1),
        sub_delta: (0.05, 0.15),
        noise_delta: (0.0, 0.05),
        brightness_delta: (-0.2, -0.1),
        decay_scale: (1.0, 1.1),
        tail_scale: (1.0, 1.1),
        preserve_core: true,
        description: "Adds analog warmth while preserving the original sound",
    });
    m.insert("more analog", PromptMutationDef {
        saturation_delta: (0.2, 0.4),
        click_delta: (-0.1, 0.0),
        attack_scale: (0.9, 1.1),
        body_delta: (0.05, 0.15),
        sub_delta: (0.0, 0.1),
        noise_delta: (0.05, 0.1),
        brightness_delta: (-0.1, 0.0),
        decay_scale: (1.0, 1.2),
        tail_scale: (1.0, 1.2),
        preserve_core: true,
        description: "Adds analog-style saturation, warmth, and subtle instability",
    });
    m.insert("more futuristic", PromptMutationDef {
        saturation_delta: (0.1, 0.3),
        click_delta: (0.1, 0.2),
        attack_scale: (0.6, 0.8),
        body_delta: (0.0, 0.05),
        sub_delta: (0.1, 0.2),
        noise_delta: (-0.2, 0.0),
        brightness_delta: (0.2, 0.35),
        decay_scale: (0.5, 0.7),
        tail_scale: (0.4, 0.6),
        preserve_core: true,
        description: "Bright, tight, modern digital character",
    });
    m.insert("more distorted", PromptMutationDef {
        saturation_delta: (0.8, 1.5),
        click_delta: (0.1, 0.2),
        attack_scale: (0.5, 0.7),
        body_delta: (0.05, 0.15),
        sub_delta: (0.0, 0.1),
        noise_delta: (0.05, 0.1),
        brightness_delta: (0.05, 0.1),
        decay_scale: (0.8, 0.9),
        tail_scale: (0.7, 0.8),
        preserve_core: true,
        description: "Heavy distortion while maintaining transient identity",
    });
    m.insert("less harsh", PromptMutationDef {
        saturation_delta: (-0.3, -0.1),
        click_delta: (-0.1, 0.0),
        attack_scale: (1.0, 1.2),
        body_delta: (0.0, 0.05),
        sub_delta: (0.0, 0.05),
        noise_delta: (-0.1, 0.0),
        brightness_delta: (-0.2, -0.1),
        decay_scale: (1.0, 1.1),
        tail_scale: (1.0, 1.1),
        preserve_core: true,
        description: "Softens harsh frequencies while preserving punch",
    });
    m.insert("tighter transient", PromptMutationDef {
        saturation_delta: (0.0, 0.1),
        click_delta: (0.1, 0.2),
        attack_scale: (0.4, 0.6),
        body_delta: (-0.05, 0.0),
        sub_delta: (0.0, 0.0),
        noise_delta: (-0.1, 0.0),
        brightness_delta: (0.0, 0.05),
        decay_scale: (0.5, 0.6),
        tail_scale: (0.3, 0.5),
        preserve_core: true,
        description: "Sharpens attack, shortens decay for tighter transient",
    });
    m.insert("fatter low end", PromptMutationDef {
        saturation_delta: (0.05, 0.15),
        click_delta: (-0.05, 0.0),
        attack_scale: (1.0, 1.0),
        body_delta: (0.05, 0.15),
        sub_delta: (0.2, 0.35),
        noise_delta: (-0.1, 0.0),
        brightness_delta: (-0.05, 0.0),
        decay_scale: (1.0, 1.1),
        tail_scale: (1.0, 1.15),
        preserve_core: true,
        description: "Enhances sub and body frequencies for fatter low end",
    });
    m.insert("more cinematic", PromptMutationDef {
        saturation_delta: (0.15, 0.3),
        click_delta: (0.0, 0.05),
        attack_scale: (0.9, 1.0),
        body_delta: (0.1, 0.2),
        sub_delta: (0.15, 0.3),
        noise_delta: (0.05, 0.1),
        brightness_delta: (0.05, 0.15),
        decay_scale: (1.2, 1.5),
        tail_scale: (1.5, 2.0),
        preserve_core: true,
        description: "Expands sound with longer tail, more sub, dramatic character",
    });
    m.insert("punchier", PromptMutationDef {
        saturation_delta: (0.15, 0.3),
        click_delta: (0.15, 0.3),
        attack_scale: (0.5, 0.7),
        body_delta: (0.05, 0.1),
        sub_delta: (0.0, 0.05),
        noise_delta: (-0.05, 0.0),
        brightness_delta: (0.05, 0.1),
        decay_scale: (0.7, 0.8),
        tail_scale: (0.6, 0.7),
        preserve_core: true,
        description: "Enhances transient impact and punch",
    });
    m.insert("softer", PromptMutationDef {
        saturation_delta: (-0.2, 0.0),
        click_delta: (-0.2, -0.1),
        attack_scale: (1.2, 1.5),
        body_delta: (-0.05, 0.0),
        sub_delta: (-0.05, 0.0),
        noise_delta: (0.0, 0.05),
        brightness_delta: (-0.1, 0.0),
        decay_scale: (1.1, 1.2),
        tail_scale: (1.1, 1.2),
        preserve_core: true,
        description: "Softens attack and reduces aggression",
    });
    m.insert("brighter", PromptMutationDef {
        saturation_delta: (0.05, 0.15),
        click_delta: (0.05, 0.1),
        attack_scale: (0.8, 0.9),
        body_delta: (-0.05, 0.0),
        sub_delta: (-0.05, 0.0),
        noise_delta: (0.05, 0.1),
        brightness_delta: (0.2, 0.35),
        decay_scale: (0.8, 0.9),
        tail_scale: (0.7, 0.8),
        preserve_core: true,
        description: "Increases brightness and presence",
    });
    m.insert("darker", PromptMutationDef {
        saturation_delta: (0.1, 0.2),
        click_delta: (-0.1, 0.0),
        attack_scale: (1.0, 1.1),
        body_delta: (0.05, 0.1),
        sub_delta: (0.05, 0.15),
        noise_delta: (-0.1, 0.0),
        brightness_delta: (-0.3, -0.2),
        decay_scale: (1.0, 1.1),
        tail_scale: (1.0, 1.1),
        preserve_core: true,
        description: "Darkens tone while adding body and sub",
    });
    m.insert("subbier", PromptMutationDef {
        saturation_delta: (0.0, 0.05),
        click_delta: (0.0, 0.05),
        attack_scale: (1.0, 1.0),
        body_delta: (0.0, 0.05),
        sub_delta: (0.25, 0.4),
        noise_delta: (-0.1, 0.0),
        brightness_delta: (-0.05, 0.0),
        decay_scale: (1.0, 1.1),
        tail_scale: (1.0, 1.15),
        preserve_core: true,
        description: "Enhances sub frequencies",
    });
    m.insert("thinner", PromptMutationDef {
        saturation_delta: (-0.1, 0.0),
        click_delta: (0.0, 0.05),
        attack_scale: (0.9, 1.0),
        body_delta: (-0.2, -0.1),
        sub_delta: (-0.2, -0.1),
        noise_delta: (0.0, 0.05),
        brightness_delta: (0.05, 0.1),
        decay_scale: (0.8, 0.9),
        tail_scale: (0.5, 0.7),
        preserve_core: true,
        description: "Reduces body and sub for a thinner sound",
    });
    m.insert("fattier", PromptMutationDef {
        saturation_delta: (0.1, 0.2),
        click_delta: (-0.05, 0.0),
        attack_scale: (1.0, 1.1),
        body_delta: (0.1, 0.2),
        sub_delta: (0.1, 0.2),
        noise_delta: (-0.05, 0.0),
        brightness_delta: (-0.05, 0.0),
        decay_scale: (1.0, 1.15),
        tail_scale: (1.0, 1.2),
        preserve_core: true,
        description: "Adds body thickness and fullness",
    });
    m
});

struct PromptMutationDef {
    saturation_delta: (f32, f32),
    click_delta: (f32, f32),
    attack_scale: (f32, f32),
    body_delta: (f32, f32),
    sub_delta: (f32, f32),
    noise_delta: (f32, f32),
    brightness_delta: (f32, f32),
    decay_scale: (f32, f32),
    tail_scale: (f32, f32),
    preserve_core: bool,
    description: &'static str,
}

pub fn apply_prompt_mutation(
    params: &ResynthesisParams,
    edit_text: &str,
    intensity: f32,
) -> (ResynthesisParams, Vec<String>, f32) {
    let intensity = intensity.clamp(0.0, 1.0);
    let lower = edit_text.to_lowercase();
    let mut params = params.clone();
    let mut edits = Vec::new();
    let mut identity_score = 1.0f32;

    let words: Vec<&str> = PROMPT_MUTATION_MAP.keys().copied().collect();
    for &keyword in &words {
        if lower.contains(keyword) {
            if let Some(def) = PROMPT_MUTATION_MAP.get(keyword) {
                let sat_min = def.saturation_delta.0 * intensity;
                let sat_max = def.saturation_delta.1 * intensity;
                let sat_delta = (sat_min + sat_max) * 0.5;
                params.saturation_drive = (params.saturation_drive + sat_delta).max(1.0);

                let clk_delta = (def.click_delta.0 + def.click_delta.1) * 0.5 * intensity;
                params.click_amount = (params.click_amount + clk_delta).clamp(0.0, 1.0);

                let atk_scale = 1.0 + ((def.attack_scale.0 + def.attack_scale.1) * 0.5 - 1.0) * intensity;
                params.attack_ms = (params.attack_ms * atk_scale).max(0.3);

                let body_delta = (def.body_delta.0 + def.body_delta.1) * 0.5 * intensity;
                params.body_gain = (params.body_gain + body_delta).clamp(0.1, 1.0);

                let sub_delta = (def.sub_delta.0 + def.sub_delta.1) * 0.5 * intensity;
                params.sub_gain = (params.sub_gain + sub_delta).clamp(0.0, 1.0);

                let noise_delta = (def.noise_delta.0 + def.noise_delta.1) * 0.5 * intensity;
                params.noise_amount = (params.noise_amount + noise_delta).clamp(0.0, 1.0);

                let bright_delta = (def.brightness_delta.0 + def.brightness_delta.1) * 0.5 * intensity;
                params.brightness = (params.brightness + bright_delta).clamp(0.0, 1.0);

                let decay_scale = 1.0 + ((def.decay_scale.0 + def.decay_scale.1) * 0.5 - 1.0) * intensity;
                params.decay_ms = (params.decay_ms * decay_scale).max(1.0);

                let tail_scale = 1.0 + ((def.tail_scale.0 + def.tail_scale.1) * 0.5 - 1.0) * intensity;
                params.tail_ms = (params.tail_ms * tail_scale).max(0.0);

                edits.push(def.description.to_string());

                if def.preserve_core {
                    identity_score -= intensity * 0.05;
                } else {
                    identity_score -= intensity * 0.15;
                }
            }
        }
    }

    identity_score = identity_score.clamp(0.3, 1.0);
    (params, edits, identity_score)
}

pub fn apply_prompt_mutation_to_samples(
    samples: &[f32],
    edit_text: &str,
    intensity: f32,
) -> (Vec<f32>, Vec<String>, f32) {
    let analysis = crate::audio::analyze::analyze_audio(samples, crate::audio::SAMPLE_RATE, 1);
    let base_params = crate::audio::recreate::params_from_analysis(&analysis, samples);
    let (mutated_params, edits, identity) = apply_prompt_mutation(&base_params, edit_text, intensity);
    let recreated = crate::audio::resynthesize::resynthesize(&mutated_params);
    (recreated, edits, identity)
}

pub fn compound_prompt_mutation(
    params: &ResynthesisParams,
    edits: &[&str],
    intensities: &[f32],
) -> (ResynthesisParams, Vec<String>, f32) {
    let mut current = params.clone();
    let mut all_edits = Vec::new();
    let mut total_identity = 1.0f32;

    for (i, edit) in edits.iter().enumerate() {
        let intensity = intensities.get(i).copied().unwrap_or(0.5);
        let (new_params, edit_strs, ident) = apply_prompt_mutation(&current, edit, intensity);
        current = new_params;
        all_edits.extend(edit_strs);
        total_identity = (total_identity * ident).max(0.3);
    }

    (current, all_edits, total_identity)
}
