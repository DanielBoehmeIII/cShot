use super::{SoundType, SAMPLE_RATE};
use super::analyze::{analyze_audio, AudioAnalysis};
use super::resynthesize::{self, ResynthesisParams};
use super::params::ExposedParams;

// ─── Pack Sound Role ────────────────────────────────────

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Hash, Eq)]
pub enum SoundRole {
    Kick,
    Snare,
    Clap,
    ClosedHat,
    OpenHat,
    Tom,
    Perc,
    Bass,
    Fx,
    Other,
}

impl SoundRole {
    pub fn from_sound_type(st: SoundType) -> Self {
        match st {
            SoundType::Kick => SoundRole::Kick,
            SoundType::Snare => SoundRole::Snare,
            SoundType::ClosedHat => SoundRole::ClosedHat,
            SoundType::OpenHat => SoundRole::OpenHat,
            SoundType::Clap => SoundRole::Clap,
            SoundType::Tom => SoundRole::Tom,
            SoundType::Perc => SoundRole::Perc,
            SoundType::Bass => SoundRole::Bass,
            SoundType::Fx => SoundRole::Fx,
            SoundType::Other => SoundRole::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SoundRole::Kick => "kick",
            SoundRole::Snare => "snare",
            SoundRole::Clap => "clap",
            SoundRole::ClosedHat => "closed_hat",
            SoundRole::OpenHat => "open_hat",
            SoundRole::Tom => "tom",
            SoundRole::Perc => "perc",
            SoundRole::Bass => "bass",
            SoundRole::Fx => "fx",
            SoundRole::Other => "other",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            SoundRole::Kick | SoundRole::Snare | SoundRole::Clap | SoundRole::Tom => "drum",
            SoundRole::ClosedHat | SoundRole::OpenHat => "hat",
            SoundRole::Perc => "percussion",
            SoundRole::Bass => "bass",
            SoundRole::Fx => "fx",
            SoundRole::Other => "other",
        }
    }

    pub fn energy_range(&self) -> (f32, f32) {
        match self {
            SoundRole::Kick => (0.7, 1.0),
            SoundRole::Snare => (0.6, 0.95),
            SoundRole::Clap => (0.5, 0.85),
            SoundRole::ClosedHat => (0.3, 0.6),
            SoundRole::OpenHat => (0.2, 0.5),
            SoundRole::Tom => (0.4, 0.7),
            SoundRole::Perc => (0.3, 0.6),
            SoundRole::Bass => (0.5, 0.8),
            SoundRole::Fx => (0.3, 0.7),
            SoundRole::Other => (0.2, 0.6),
        }
    }

    pub fn total_count() -> usize { 10 }
}

// ─── Pack Profile ────────────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PackProfile {
    pub genre: String,
    pub energy: f32,           // 0.0-1.0
    pub brightness: f32,       // 0.0-1.0
    pub complexity: f32,       // 0.0-1.0
    pub pitch_focus: f32,      // 0.0=low, 0.5=mid, 1.0=high
    pub variation: f32,        // 0.0-1.0
    pub kit_completeness: f32, // 0.0-1.0 how many roles are filled
}

impl Default for PackProfile {
    fn default() -> Self {
        Self {
            genre: "generic".to_string(),
            energy: 0.6,
            brightness: 0.5,
            complexity: 0.4,
            pitch_focus: 0.3,
            variation: 0.4,
            kit_completeness: 0.8,
        }
    }
}

impl PackProfile {
    pub fn for_genre(genre: &str) -> Self {
        match genre {
            "trap" => Self {
                genre: "trap".to_string(),
                energy: 0.8, brightness: 0.4, complexity: 0.5,
                pitch_focus: 0.1, variation: 0.5, kit_completeness: 0.9,
            },
            "techno" => Self {
                genre: "techno".to_string(),
                energy: 0.85, brightness: 0.6, complexity: 0.4,
                pitch_focus: 0.3, variation: 0.3, kit_completeness: 0.85,
            },
            "lo-fi" | "lofi" => Self {
                genre: "lo-fi".to_string(),
                energy: 0.3, brightness: 0.3, complexity: 0.3,
                pitch_focus: 0.5, variation: 0.4, kit_completeness: 0.7,
            },
            "cinematic" => Self {
                genre: "cinematic".to_string(),
                energy: 0.7, brightness: 0.6, complexity: 0.6,
                pitch_focus: 0.5, variation: 0.6, kit_completeness: 0.6,
            },
            "house" => Self {
                genre: "house".to_string(),
                energy: 0.75, brightness: 0.7, complexity: 0.3,
                pitch_focus: 0.4, variation: 0.3, kit_completeness: 0.8,
            },
            "drill" => Self {
                genre: "drill".to_string(),
                energy: 0.85, brightness: 0.3, complexity: 0.5,
                pitch_focus: 0.1, variation: 0.4, kit_completeness: 0.85,
            },
            "dubstep" => Self {
                genre: "dubstep".to_string(),
                energy: 0.9, brightness: 0.7, complexity: 0.7,
                pitch_focus: 0.3, variation: 0.6, kit_completeness: 0.7,
            },
            _ => Self {
                genre: genre.to_string(),
                ..Default::default()
            },
        }
    }

    pub fn roles_for_profile(&self) -> Vec<SoundRole> {
        let mut roles = Vec::new();
        roles.push(SoundRole::Kick);
        roles.push(SoundRole::Snare);
        roles.push(SoundRole::ClosedHat);
        roles.push(SoundRole::OpenHat);
        if self.kit_completeness > 0.6 {
            roles.push(SoundRole::Clap);
            roles.push(SoundRole::Bass);
        }
        if self.kit_completeness > 0.75 {
            roles.push(SoundRole::Tom);
            roles.push(SoundRole::Perc);
        }
        if self.kit_completeness > 0.85 {
            roles.push(SoundRole::Fx);
        }
        roles
    }
}

// ─── Cohesion Metrics ────────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PackCohesion {
    pub overall: f32,
    pub role_balance: f32,
    pub energy_consistency: f32,
    pub brightness_consistency: f32,
    pub duration_consistency: f32,
    pub duplicate_score: f32,
    pub diversity_score: f32,
}

pub fn analyze_pack_cohesion(analyses: &[AudioAnalysis], roles: &[SoundRole]) -> PackCohesion {
    if analyses.is_empty() {
        return PackCohesion {
            overall: 0.0, role_balance: 0.0, energy_consistency: 0.0,
            brightness_consistency: 0.0, duration_consistency: 0.0,
            duplicate_score: 1.0, diversity_score: 0.0,
        };
    }

    // Role balance: how evenly distributed
    let role_count = roles.len();
    let unique_roles: std::collections::HashSet<SoundRole> = roles.iter().copied().collect();
    let role_balance = unique_roles.len() as f32 / role_count.max(1) as f32;

    // Energy consistency
    let energies: Vec<f32> = analyses.iter().map(|a| a.rms).collect();
    let mean_energy: f32 = energies.iter().sum::<f32>() / energies.len() as f32;
    let energy_var: f32 = energies.iter().map(|e| (e - mean_energy).powi(2)).sum::<f32>() / energies.len() as f32;
    let energy_consistency = (1.0 - (energy_var * 10.0).min(1.0)).max(0.0);

    // Brightness consistency
    let brightnesses: Vec<f32> = analyses.iter().map(|a| a.brightness).collect();
    let mean_b: f32 = brightnesses.iter().sum::<f32>() / brightnesses.len() as f32;
    let b_var: f32 = brightnesses.iter().map(|b| (b - mean_b).powi(2)).sum::<f32>() / brightnesses.len() as f32;
    let brightness_consistency = (1.0 - (b_var * 5.0).min(1.0)).max(0.0);

    // Duration consistency
    let durations: Vec<f32> = analyses.iter().map(|a| a.duration_ms).collect();
    let mean_d: f32 = durations.iter().sum::<f32>() / durations.len() as f32;
    let d_var: f32 = durations.iter().map(|d| ((d - mean_d) / mean_d.max(1.0)).powi(2)).sum::<f32>() / durations.len() as f32;
    let duration_consistency = (1.0 - (d_var * 3.0).min(1.0)).max(0.0);

    // Duplicate detection: compare spectral profiles
    let mut dup_score = 1.0f32;
    for i in 0..analyses.len() {
        for j in (i + 1)..analyses.len() {
            if i < analyses[i].spectral_profile.len() && j < analyses[j].spectral_profile.len() {
                let profile_i = &analyses[i].spectral_profile;
                let profile_j = &analyses[j].spectral_profile;
                let len = profile_i.len().min(profile_j.len());
                let mut corr = 0.0f32;
                let mut e_i = 0.0f32;
                let mut e_j = 0.0f32;
                for k in 0..len {
                    corr += profile_i[k] * profile_j[k];
                    e_i += profile_i[k] * profile_i[k];
                    e_j += profile_j[k] * profile_j[k];
                }
                let denom = (e_i * e_j).sqrt().max(1e-10);
                let similarity = (corr / denom).clamp(0.0, 1.0);
                if similarity > 0.85 {
                    dup_score *= 0.5;
                }
            }
        }
    }

    // Diversity: count unique sound types
    let types: std::collections::HashSet<&str> = analyses.iter().map(|a| a.sound_type_hint.as_str()).collect();
    let diversity_score = (types.len() as f32 / 10.0).min(1.0);

    let overall = (role_balance * 0.25 + energy_consistency * 0.2
        + brightness_consistency * 0.15 + duration_consistency * 0.1
        + dup_score * 0.15 + diversity_score * 0.15)
        .clamp(0.0, 1.0);

    PackCohesion {
        overall,
        role_balance,
        energy_consistency,
        brightness_consistency,
        duration_consistency,
        duplicate_score: dup_score,
        diversity_score,
    }
}

// ─── Pack Generation ─────────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GeneratedPack {
    pub name: String,
    pub genre: String,
    pub description: String,
    pub sounds: Vec<GeneratedPackSound>,
    pub cohesion: PackCohesion,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GeneratedPackSound {
    pub role: SoundRole,
    pub name: String,
    pub samples: Vec<f32>,
    pub energy: f32,
    pub params: String,
}

pub fn generate_intelligent_pack(profile: &PackProfile, sound_count: usize) -> GeneratedPack {
    let roles = profile.roles_for_profile();
    let mut sounds: Vec<GeneratedPackSound> = Vec::new();
    let mut analyses: Vec<AudioAnalysis> = Vec::new();
    let mut used_seeds: std::collections::HashSet<u64> = std::collections::HashSet::new();

    // Generate sounds filling roles
    let count = sound_count.min(roles.len() * 2).max(roles.len());
    for i in 0..count {
        let role = if i < roles.len() {
            roles[i]
        } else {
            // Extra sounds: pick random role with bias toward percussion
            let idx = (i * 7 + 3) % roles.len();
            roles[idx]
        };

        let st = match role {
            SoundRole::Kick => SoundType::Kick,
            SoundRole::Snare => SoundType::Snare,
            SoundRole::ClosedHat => SoundType::ClosedHat,
            SoundRole::OpenHat => SoundType::OpenHat,
            SoundRole::Clap => SoundType::Clap,
            SoundRole::Tom => SoundType::Tom,
            SoundRole::Perc => SoundType::Perc,
            SoundRole::Bass => SoundType::Bass,
            SoundRole::Fx => SoundType::Fx,
            SoundRole::Other => SoundType::Other,
        };

        // Base parameters
        let base_pitch = match st {
            SoundType::Kick | SoundType::Bass => 40.0 + (i as f32 * 7.0) % 40.0,
            SoundType::Snare => 180.0 + (i as f32 * 11.0) % 60.0,
            SoundType::ClosedHat => 300.0 + (i as f32 * 13.0) % 200.0,
            SoundType::OpenHat => 250.0 + (i as f32 * 17.0) % 200.0,
            SoundType::Clap => 150.0 + (i as f32 * 19.0) % 80.0,
            SoundType::Tom => 80.0 + (i as f32 * 23.0) % 100.0,
            SoundType::Perc => 200.0 + (i as f32 * 29.0) % 400.0,
            SoundType::Fx => 50.0 + (i as f32 * 31.0) % 300.0,
            _ => 200.0,
        };

        let dur = match st {
            SoundType::Kick => 300.0, SoundType::Snare => 350.0,
            SoundType::ClosedHat => 150.0, SoundType::OpenHat => 500.0,
            SoundType::Clap => 300.0, SoundType::Bass => 600.0,
            SoundType::Perc => 200.0, SoundType::Fx => 800.0,
            SoundType::Tom => 400.0, SoundType::Other => 300.0,
        };

        // Apply genre adaptation via the recreate module
        let mut base = resynthesize::params_for_sound_type(st, base_pitch, dur);
        base = super::recreate::adapt_params_for_genre(&base, &profile.genre);

        // Apply energy and variation
        let (min_energy, max_energy) = role.energy_range();
        let energy_range = max_energy - min_energy;
        let energy_target = min_energy + energy_range * profile.energy;

        // Introduce controlled variation
        let variation = profile.variation * 0.3;
        let seed = (i as u64).wrapping_mul(2654435761).wrapping_add(profile.genre.len() as u64);
        let mut p = base.clone().with_seed(seed);
        p = p.randomize(variation);

        // Energy shaping
        p.body_gain = (p.body_gain * energy_target).clamp(0.1, 1.0);
        p.sub_gain = (p.sub_gain * energy_target).clamp(0.0, 1.0);

        // Avoid duplicate params
        let params_key = format!("{:.1}{:.1}{:.1}", p.pitch_hz, p.decay_ms, p.click_amount);
        let hash = params_key.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        if used_seeds.contains(&hash) {
            p = p.randomize(0.2);
        }
        used_seeds.insert(hash);

        let samples = resynthesize::resynthesize(&p);
        if samples.is_empty() { continue; }

        let analysis = analyze_audio(&samples, SAMPLE_RATE, 1);

        // Create a name
        let name = format!("{} {} {:02}", profile.genre, role.as_str(), i + 1);

        sounds.push(GeneratedPackSound {
            role,
            name,
            samples,
            energy: analysis.rms,
            params: format!("{:?}", p),
        });
        analyses.push(analysis);
    }

    // Avoid duplicates by checking spectral similarity
    sounds = deduplicate_pack_sounds(sounds, 0.85);

    // Compute cohesion
    let roles_final: Vec<SoundRole> = sounds.iter().map(|s| s.role).collect();
    let cohesion = analyze_pack_cohesion(&analyses, &roles_final);

    let desc = format!("{} sound pack — {} sounds, {} roles, cohesion {:.0}%",
        profile.genre, sounds.len(), roles_final.len(), cohesion.overall * 100.0);

    GeneratedPack {
        name: format!("{} Pack", profile.genre),
        genre: profile.genre.clone(),
        description: desc,
        sounds,
        cohesion,
    }
}

// ─── Smart Kit Intelligence ──────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct KitAdvice {
    pub missing_roles: Vec<String>,
    pub duplicate_roles: Vec<(String, usize)>,
    pub tonal_issues: Vec<String>,
    pub transient_issues: Vec<String>,
    pub balance_rating: f32,
    pub recommendations: Vec<String>,
}

pub fn analyze_kit_composition(sounds: &[GeneratedPackSound]) -> KitAdvice {
    let mut role_counts: std::collections::HashMap<SoundRole, usize> = std::collections::HashMap::new();
    for s in sounds {
        *role_counts.entry(s.role).or_insert(0) += 1;
    }

    let essential = [SoundRole::Kick, SoundRole::Snare, SoundRole::ClosedHat, SoundRole::OpenHat];
    let mut missing_roles = Vec::new();
    for role in &essential {
        if !role_counts.contains_key(role) {
            missing_roles.push(role.as_str().to_string());
        }
    }

    let role_str = |r: &SoundRole| r.as_str().to_string();
    let mut duplicate_roles: Vec<(String, usize)> = role_counts.iter()
        .filter(|&(_, &c)| c > 1)
        .map(|(r, &c)| (role_str(r), c))
        .collect();
    duplicate_roles.sort_by(|a, b| b.1.cmp(&a.1));

    let mut tonal_issues = Vec::new();
    if role_counts.contains_key(&SoundRole::Kick) && role_counts.contains_key(&SoundRole::Bass) {
        if let (Some(kick), Some(bass)) = (
            sounds.iter().find(|s| s.role == SoundRole::Kick),
            sounds.iter().find(|s| s.role == SoundRole::Bass),
        ) {
            let kick_analysis = super::analyze::analyze_audio(&kick.samples, SAMPLE_RATE, 1);
            let bass_analysis = super::analyze::analyze_audio(&bass.samples, SAMPLE_RATE, 1);
            if let (Some(kp), Some(bp)) = (kick_analysis.pitch_estimate, bass_analysis.pitch_estimate) {
                let ratio = if kp > bp { kp / bp } else { bp / kp };
                if ratio < 1.5 || (ratio > 2.5 && ratio < 3.5) {
                    // Good: octave or fifth apart
                } else if ratio < 2.0 {
                    tonal_issues.push(format!("Kick ({:.0}Hz) and Bass ({:.0}Hz) may clash: ratio {:.2}", kp, bp, ratio));
                }
            }
        }
    }
    if role_counts.contains_key(&SoundRole::Snare) && role_counts.contains_key(&SoundRole::Clap) {
        tonal_issues.push("Snare and Clap both present — ensure they occupy different frequency ranges.".to_string());
    }

    let mut transient_issues = Vec::new();
    let mut all_transient_strengths: Vec<(String, f32)> = Vec::new();
    for s in sounds {
        let analysis = super::analyze::analyze_audio(&s.samples, SAMPLE_RATE, 1);
        all_transient_strengths.push((s.role.as_str().to_string(), analysis.transient_strength));
    }
    for i in 0..all_transient_strengths.len() {
        for j in (i + 1)..all_transient_strengths.len() {
            let diff = (all_transient_strengths[i].1 - all_transient_strengths[j].1).abs();
            if diff > 5.0 {
                transient_issues.push(format!(
                    "Transient mismatch: {} ({:.1}) vs {} ({:.1}) — consider matching character",
                    all_transient_strengths[i].0, all_transient_strengths[i].1,
                    all_transient_strengths[j].0, all_transient_strengths[j].1
                ));
            }
        }
    }
    transient_issues.truncate(5);

    let mut recommendations = Vec::new();
    if missing_roles.contains(&"kick".to_string()) {
        recommendations.push("Add a kick drum to anchor the low end.".to_string());
    }
    if missing_roles.contains(&"snare".to_string()) {
        recommendations.push("Add a snare or clap for backbeat.".to_string());
    }
    if missing_roles.contains(&"closed_hat".to_string()) {
        recommendations.push("Add closed hi-hats for rhythm.".to_string());
    }
    if missing_roles.contains(&"open_hat".to_string()) {
        recommendations.push("Add open hi-hats for variation.".to_string());
    }
    if role_counts.get(&SoundRole::Kick).copied().unwrap_or(0) > 2 {
        recommendations.push("Multiple kicks detected — consider selecting your primary kick.".to_string());
    }
    if role_counts.get(&SoundRole::Snare).copied().unwrap_or(0) > 2 {
        recommendations.push("Multiple snares — layer them or choose one primary.".to_string());
    }
    let total = sounds.len();
    if total < 4 {
        recommendations.push(format!("Only {} sounds in kit — add more for a complete drum kit.", total));
    } else if total > 16 {
        recommendations.push(format!("{} sounds is a large kit — consider reducing for focus.", total));
    }

    let role_coverage = missing_roles.len() as f32;
    let dup_penalty = duplicate_roles.len() as f32 * 0.1;
    let tonal_penalty = tonal_issues.len() as f32 * 0.05;
    let transient_penalty = transient_issues.len() as f32 * 0.03;
    let size_bonus = (total as f32 / 8.0).min(1.0) * 0.2;
    let balance_rating = (1.0 - role_coverage * 0.15 - dup_penalty - tonal_penalty - transient_penalty + size_bonus).clamp(0.0, 1.0);

    KitAdvice {
        missing_roles,
        duplicate_roles,
        tonal_issues,
        transient_issues,
        balance_rating,
        recommendations,
    }
}

pub fn generate_balanced_kit(profile: &PackProfile) -> GeneratedPack {
    let mut pack = generate_intelligent_pack(profile, 8);
    let advice = analyze_kit_composition(&pack.sounds);

    // If missing essential roles, add them
    if !advice.missing_roles.is_empty() {
        for missing in &advice.missing_roles {
            let role = match missing.as_str() {
                "kick" => SoundRole::Kick,
                "snare" => SoundRole::Snare,
                "closed_hat" => SoundRole::ClosedHat,
                "open_hat" => SoundRole::OpenHat,
                _ => continue,
            };
            let st = match role {
                SoundRole::Kick => SoundType::Kick,
                SoundRole::Snare => SoundType::Snare,
                SoundRole::ClosedHat => SoundType::ClosedHat,
                SoundRole::OpenHat => SoundType::OpenHat,
                _ => continue,
            };
            let pitch = match st {
                SoundType::Kick => 50.0 + profile.pitch_focus * 50.0,
                SoundType::Snare => 200.0,
                SoundType::ClosedHat => 400.0,
                SoundType::OpenHat => 300.0,
                _ => 200.0,
            };
            let dur = duration_for_type(st);
            let mut base = resynthesize::params_for_sound_type(st, pitch, dur);
            base = super::recreate::adapt_params_for_genre(&base, &profile.genre);
            base.body_gain = (base.body_gain * (0.5 + profile.energy * 0.5)).clamp(0.1, 1.0);
            let seed = pack.sounds.len() as u64;
            base = base.with_seed(seed).randomize(0.15);
            let samples = resynthesize::resynthesize(&base);
            if !samples.is_empty() {
                pack.sounds.push(GeneratedPackSound {
                    role,
                    name: format!("{} {}", profile.genre, role.as_str()),
                    samples,
                    energy: 0.5,
                    params: format!("{:?}", base),
                });
            }
        }
    }

    pack.description = format!(
        "{} kit — {} sounds, {} roles, balance {:.0}%, cohesion {:.0}%",
        profile.genre, pack.sounds.len(),
        pack.sounds.iter().map(|s| s.role).collect::<std::collections::HashSet<_>>().len(),
        analyze_kit_composition(&pack.sounds).balance_rating * 100.0,
        pack.cohesion.overall * 100.0,
    );

    pack
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

fn deduplicate_pack_sounds(sounds: Vec<GeneratedPackSound>, threshold: f32) -> Vec<GeneratedPackSound> {
    let mut deduped: Vec<GeneratedPackSound> = Vec::new();
    for sound in sounds {
        let samples = &sound.samples;
        let is_dup = deduped.iter().any(|existing| {
            let min_len = samples.len().min(existing.samples.len());
            if min_len < 256 { return false; }
            let mut corr = 0.0f32;
            let mut e1 = 0.0f32;
            let mut e2 = 0.0f32;
            let step = (min_len / 256).max(1);
            let mut count = 0;
            for i in (0..min_len).step_by(step) {
                corr += samples[i] * existing.samples[i];
                e1 += samples[i] * samples[i];
                e2 += existing.samples[i] * existing.samples[i];
                count += 1;
            }
            if count == 0 { return false; }
            let denom = (e1 * e2).sqrt().max(1e-10);
            let sim = (corr / denom).clamp(0.0, 1.0);
            sim > threshold
        });
        if !is_dup {
            deduped.push(sound);
        }
    }
    deduped
}
