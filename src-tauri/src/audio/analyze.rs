use super::{SoundType, SAMPLE_RATE};

pub fn compute_rms(samples: &[f32]) -> f32 {
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

pub fn compute_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max)
}

pub fn compute_crest_factor(samples: &[f32]) -> f32 {
    let peak = compute_peak(samples);
    let rms = compute_rms(samples);
    if rms > 0.0 { peak / rms } else { 1.0 }
}

pub fn compute_spectral_centroid(samples: &[f32]) -> f32 {
    let n = samples.len();
    if n < 2 {
        return 0.0;
    }
    let mut magnitude_sum = 0.0f32;
    let mut weighted_sum = 0.0f32;
    for i in 0..n.min(2048) {
        let freq = i as f32 * SAMPLE_RATE as f32 / n as f32;
        let mag = samples[i].abs();
        magnitude_sum += mag;
        weighted_sum += freq * mag;
    }
    if magnitude_sum > 0.0 {
        weighted_sum / magnitude_sum
    } else {
        0.0
    }
}

pub fn compute_energy_sub_low(samples: &[f32]) -> f32 {
    let total: f32 = samples.iter().map(|&s| s * s).sum();
    if total == 0.0 { return 0.0; }
    let mut sub = 0.0f32;
    let n = samples.len().min(2048);
    for i in 0..n {
        let freq = i as f32 * SAMPLE_RATE as f32 / n as f32;
        if freq < 100.0 {
            sub += samples[i] * samples[i];
        }
    }
    sub / total
}

pub fn compute_zero_crossing_rate(samples: &[f32]) -> f32 {
    if samples.len() < 2 { return 0.0; }
    let mut crossings = 0;
    for i in 1..samples.len() {
        if samples[i] >= 0.0 && samples[i - 1] < 0.0 {
            crossings += 1;
        } else if samples[i] < 0.0 && samples[i - 1] >= 0.0 {
            crossings += 1;
        }
    }
    crossings as f32 / samples.len() as f32
}

fn extract_prompt_keywords(prompt: &str) -> Vec<String> {
    let lower = prompt.to_lowercase();
    let mut keywords = Vec::new();
    let genre_map: Vec<&str> = vec![
        "trap", "house", "techno", "lo-fi", "lofi", "cinematic",
        "ambient", "dubstep", "dnb", "drum and bass", "rock",
        "metal", "jazz", "hip-hop", "hip hop", "rnb", "808",
        "foley", "orchestral", "synth", "electronic", "dance",
        "garage", "grime", "boom bap", "afro", "latin", "reggaeton",
        "drill", "phonk", "footwork", "jersey", "uk-garage",
        "industrial", "synthwave", "vaporwave", "breakbeat",
        "future-bass", "hyperpop", "emo", "trap-metal",
    ];
    for &genre in &genre_map {
        if lower.contains(genre) {
            let clean = genre.replace(" ", "-");
            if !keywords.contains(&clean) {
                keywords.push(clean);
            }
        }
    }
    let style_map = vec![
        "punchy", "dark", "bright", "warm", "soft", "hard",
        "aggressive", "distorted", "clean", "noisy", "deep", "sub",
        "crisp", "shiny", "gentle", "round", "tight", "fat", "dry",
        "wet", "layered", "metallic", "wooden", "organic", "digital",
        "airy", "hollow", "boomy", "click", "crack", "snap",
        "thick", "thin", "smooth", "gritty", "glitchy", "sweep",
    ];
    for &style in &style_map {
        if lower.contains(style) {
            if !keywords.contains(&style.to_string()) {
                keywords.push(style.to_string());
            }
        }
    }
    let mood_map = vec![
        "epic", "dark", "ominous", "happy", "sad", "melodic",
        "aggressive", "calm", "tense", "dreamy", "haunting",
        "uplifting", "mysterious", "heavy", "light", "powerful",
    ];
    for &mood in &mood_map {
        if lower.contains(mood) && !keywords.contains(&mood.to_string()) {
            keywords.push(mood.to_string());
        }
    }
    let production_map = vec![
        "compressed", "saturated", "limited", "reverb", "delayed",
        "phaser", "flanger", "chorus", "sidechain", "filtered",
        "lofi", "glue", "busy", "minimal", "layered", "textured",
    ];
    for &prod in &production_map {
        if lower.contains(prod) && !keywords.contains(&prod.to_string()) {
            keywords.push(prod.to_string());
        }
    }
    keywords
}

pub fn compute_attack_time(samples: &[f32]) -> f32 {
    let n = samples.len();
    if n < 100 { return 0.0; }
    let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max);
    if peak < 0.001 { return 0.0; }
    let threshold = peak * 0.5;
    for i in 0..n {
        if samples[i].abs() >= threshold {
            return i as f32 / SAMPLE_RATE as f32 * 1000.0;
        }
    }
    0.0
}

pub fn apply_autotags(samples: &[f32], sound_type: &SoundType, variant_name: Option<&str>, prompt: Option<&str>) -> Vec<String> {
    let mut tags = Vec::new();
    let rms = compute_rms(samples);
    let centroid = compute_spectral_centroid(samples);
    let crest = compute_crest_factor(samples);
    let zcr = compute_zero_crossing_rate(samples);
    let sub_energy = compute_energy_sub_low(samples);
    let duration_ms = samples.len() as f32 / SAMPLE_RATE as f32 * 1000.0;
    let attack_ms = compute_attack_time(samples);

    tags.push(sound_type.as_str().to_string());

    if centroid > 4000.0 { tags.push("bright".to_string()); }
    else if centroid > 2500.0 { tags.push("warm".to_string()); }
    if centroid < 800.0 { tags.push("dark".to_string()); }

    if centroid > 5000.0 && zcr > 0.15 { tags.push("metallic".to_string()); }
    if centroid < 500.0 && sub_energy > 0.4 { tags.push("boomy".to_string()); }

    if rms > 0.25 { tags.push("loud".to_string()); }
    if rms < 0.08 { tags.push("quiet".to_string()); }

    if duration_ms < 150.0 { tags.push("short".to_string()); }
    else if duration_ms < 500.0 { tags.push("medium".to_string()); }
    if duration_ms > 1500.0 { tags.push("long".to_string()); }

    if crest > 12.0 && rms > 0.08 { tags.push("punchy".to_string()); }
    if crest < 6.0 { tags.push("compressed".to_string()); }
    if crest >= 6.0 && crest <= 12.0 { tags.push("dynamic".to_string()); }

    if attack_ms > 0.0 && attack_ms < 3.0 { tags.push("sharp-attack".to_string()); }
    else if attack_ms >= 3.0 && attack_ms < 10.0 { tags.push("moderate-attack".to_string()); }
    else if attack_ms >= 10.0 { tags.push("soft-attack".to_string()); }

    if sub_energy > 0.3 { tags.push("sub".to_string()); }
    if sub_energy < 0.05 && centroid > 2000.0 { tags.push("thin".to_string()); }

    if zcr > 0.25 { tags.push("noisy".to_string()); }
    if zcr < 0.02 { tags.push("clean".to_string()); }

    if let Some(vname) = variant_name {
        match vname {
            "reversed" => tags.push("reversed".to_string()),
            "saturated" | "distorted" => { tags.push("distorted".to_string()); tags.push("saturated".to_string()); }
            "shortened" | "trimmed" => tags.push("short".to_string()),
            "repitched" => tags.push("repitched".to_string()),
            "shaped" => tags.push("shaped".to_string()),
            "layered" => tags.push("layered".to_string()),
            "randomized" => tags.push("randomized".to_string()),
            _ => {}
        }
    }

    if let Some(p) = prompt {
        let prompt_tags = extract_prompt_keywords(p);
        for pt in prompt_tags {
            if !tags.contains(&pt) {
                tags.push(pt);
            }
        }
        let recipe_tags = infer_recipe_tags(p, sound_type);
        for rt in recipe_tags {
            if !tags.contains(&rt) {
                tags.push(rt);
            }
        }
    }

    if centroid > 0.0 && centroid < 2000.0 { tags.push("warm-range".to_string()); }
    if centroid > 3000.0 { tags.push("bright-range".to_string()); }

    let has_clipping = samples.iter().any(|&s| s.abs() >= 1.0);
    if has_clipping { tags.push("clipped".to_string()); }

    if crest > 15.0 { tags.push("high-crest".to_string()); }

    tags
}

fn infer_recipe_tags(prompt: &str, sound_type: &SoundType) -> Vec<String> {
    let lower = prompt.to_lowercase();
    let mut tags = Vec::new();

    match sound_type {
        SoundType::Kick => {
            if lower.contains("808") || lower.contains("boomy") || lower.contains("deep") {
                tags.push("808-influence".to_string());
            }
            if lower.contains("click") || lower.contains("tight") || lower.contains("electronic") {
                tags.push("clicky-attack".to_string());
            }
            if lower.contains("distorted") || lower.contains("lo-fi") || lower.contains("gritty") {
                tags.push("processed".to_string());
            }
            if lower.contains("sub") || lower.contains("deep") || lower.contains("heavy") {
                tags.push("sub-heavy".to_string());
            }
            if lower.contains("acoustic") || lower.contains("natural") || lower.contains("rock") || lower.contains("real") {
                tags.push("acoustic-style".to_string());
            }
        }
        SoundType::Snare => {
            if lower.contains("crack") || lower.contains("bright") || lower.contains("metal") {
                tags.push("bright-crack".to_string());
            }
            if lower.contains("trap") || lower.contains("layered") || lower.contains("clap") {
                tags.push("layered-snare".to_string());
            }
            if lower.contains("rim") || lower.contains("rimshot") || lower.contains("wood") {
                tags.push("rim-style".to_string());
            }
            if lower.contains("military") || lower.contains("marching") || lower.contains("march") {
                tags.push("march-style".to_string());
            }
        }
        SoundType::ClosedHat | SoundType::OpenHat => {
            if lower.contains("tight") || lower.contains("short") || lower.contains("closed") {
                tags.push("tight-hat".to_string());
            }
            if lower.contains("wash") || lower.contains("open") || lower.contains("sizzle") {
                tags.push("washy-hat".to_string());
            }
            if lower.contains("acoustic") || lower.contains("jazz") || lower.contains("real") {
                tags.push("acoustic-hat".to_string());
            }
        }
        SoundType::Bass => {
            if lower.contains("sub") || lower.contains("808") || lower.contains("deep") {
                tags.push("sub-bass".to_string());
            }
            if lower.contains("warm") || lower.contains("round") || lower.contains("sine") {
                tags.push("warm-bass".to_string());
            }
            if lower.contains("distorted") || lower.contains("gritty") || lower.contains("saturated") {
                tags.push("distorted-bass".to_string());
            }
        }
        SoundType::Fx => {
            if lower.contains("riser") || lower.contains("build") || lower.contains("sweep") || lower.contains("rise") {
                tags.push("riser".to_string());
            }
            if lower.contains("impact") || lower.contains("hit") || lower.contains("boom") || lower.contains("cinematic") {
                tags.push("impact".to_string());
            }
            if lower.contains("reverse") || lower.contains("swell") || lower.contains("wash") {
                tags.push("swell".to_string());
            }
            if lower.contains("glitch") || lower.contains("stutter") || lower.contains("digital") {
                tags.push("glitch".to_string());
            }
            if lower.contains("orchestral") || lower.contains("cinematic") || lower.contains("dramatic") {
                tags.push("cinematic-fx".to_string());
            }
        }
        SoundType::Perc | SoundType::Tom | SoundType::Clap | SoundType::Other => {
            if lower.contains("shaker") || lower.contains("tambourine") || lower.contains("cowbell") {
                tags.push("hand-perc".to_string());
            }
            if lower.contains("electronic") || lower.contains("synth") || lower.contains("digital") {
                tags.push("electronic-perc".to_string());
            }
            if lower.contains("acoustic") || lower.contains("organic") || lower.contains("natural") {
                tags.push("acoustic-perc".to_string());
            }
        }
    }

    if lower.contains("reverb") || lower.contains("ambient") || lower.contains("space") || lower.contains("wet") {
        if !tags.contains(&"wet".to_string()) { tags.push("wet".to_string()); }
    }
    if lower.contains("dry") && !tags.contains(&"dry".to_string()) {
        tags.push("dry".to_string());
    }
    if lower.contains("layer") || lower.contains("stack") || lower.contains("mult") {
        tags.push("layered".to_string());
    }
    if lower.contains("bpm") || lower.contains("tempo") || lower.matches(|c: char| c.is_ascii_digit()).count() >= 3 {
        tags.push("tempo-specified".to_string());
    }

    tags
}
