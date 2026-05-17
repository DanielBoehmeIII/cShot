use std::path::PathBuf;
use std::time::Instant;
use serde_json::Value;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let dry_run = args.contains(&"--dry-run".to_string());
    let eval_only = args.contains(&"--eval-only".to_string());
    let provider = get_arg_value(&args, "--provider").unwrap_or_else(|| "cshot-engine".to_string());
    let results_dir = get_arg_value(&args, "--dir").unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/cShot/bakeoff", home)
    });

    let prompts_path = find_prompts_json();
    let prompts_data = std::fs::read_to_string(&prompts_path)
        .expect("Could not read prompts.json");
    let prompts: Value = serde_json::from_str(&prompts_data)
        .expect("Invalid prompts.json");

    if dry_run {
        print_dry_run(&prompts, &provider, &results_dir);
        return;
    }

    if eval_only {
        println!("Eval-only mode — scanning {} for results...", results_dir);
        let metadata_path = PathBuf::from(&results_dir).join("bakeoff_metadata.json");
        if metadata_path.exists() {
            let data = std::fs::read_to_string(&metadata_path).unwrap_or_default();
            let meta: Value = serde_json::from_str(&data).unwrap_or(Value::Null);
            let total = meta.get("total_attempted").and_then(|v| v.as_i64()).unwrap_or(0);
            let succeeded = meta.get("total_succeeded").and_then(|v| v.as_i64()).unwrap_or(0);
            println!("  Previously: {}/{} succeeded", succeeded, total);
        } else {
            println!("  No results found at {}. Run with --provider first.", results_dir);
        }
        return;
    }

    println!("cShot Model Bakeoff");
    println!("====================");
    println!("Provider:   {}", provider);
    println!("Results:    {}", results_dir);
    println!("NOTE: Real generation requires the cshot app to be running.");
    println!("This scaffold creates the result directory structure and metadata.");
    println!("Connect actual generation when real providers are wired up.");
    println!();

    let categories = prompts["categories"].as_object().unwrap();
    let seeds = prompts["seeds"].as_array().unwrap();
    let seed_values: Vec<i64> = seeds.iter().filter_map(|s| s.as_i64()).collect();

    let results_dir = PathBuf::from(&results_dir);
    let provider_dir = results_dir.join(&provider);
    std::fs::create_dir_all(&provider_dir).ok();

    let mut generation_log: Vec<Value> = Vec::new();
    let mut total = 0;
    let mut succeeded = 0;

    for (cat_name, cat_data) in categories {
        let cat_dir = provider_dir.join(cat_name);
        std::fs::create_dir_all(&cat_dir).ok();

        let cat_prompts = cat_data["prompts"].as_array().unwrap();
        for prompt_entry in cat_prompts {
            let prompt_id = prompt_entry["id"].as_i64().unwrap();
            let prompt_text = prompt_entry["text"].as_str().unwrap();

            for &seed in &seed_values {
                total += 1;

                let filename = format!("{}_{:02}_seed{}.wav", cat_name, prompt_id, seed);
                let filepath = cat_dir.join(&filename);

                print!("  [{:>2}/{:>2}] {} | seed {} ...",
                    total, 100, prompt_text, seed);

                let start = Instant::now();

                std::fs::write(&filepath, &[0u8; 44]).ok();

                let elapsed = start.elapsed().as_millis();
                succeeded += 1;
                println!(" ✓ (placeholder, {}ms)", elapsed);

                generation_log.push(serde_json::json!({
                    "provider": provider,
                    "category": cat_name,
                    "prompt_id": prompt_id,
                    "prompt": prompt_text,
                    "seed": seed,
                    "file": filepath.to_string_lossy(),
                    "success": true,
                    "placeholder": true,
                    "wall_clock_ms": elapsed,
                }));
            }
        }
    }

    let metadata = serde_json::json!({
        "bakeoff_version": 1,
        "run_started_at": chrono_now_iso(),
        "provider": provider,
        "total_attempted": total,
        "total_succeeded": succeeded,
        "total_failed": total - succeeded,
        "seeds": seed_values,
        "generation_log": generation_log,
    });

    let metadata_path = results_dir.join("bakeoff_metadata.json");
    std::fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata).unwrap(),
    ).ok();

    println!();
    println!("====================");
    println!("Done: {}/{} succeeded", succeeded, total);
    println!("Metadata: {}", metadata_path.display());
    println!();
    println!("Next steps:");
    println!("  Run with --provider elevenlabs when the real provider is connected");
    println!("  Run with --eval-only --dir {} to re-evaluate", results_dir.display());
}

fn print_dry_run(prompts: &Value, provider: &str, results_dir: &str) {
    println!("cShot Model Bakeoff — Dry Run");
    println!("==============================");
    println!("Provider:   {}", provider);
    println!("Results:    {}", results_dir);
    println!();

    let categories = prompts["categories"].as_object().unwrap();
    let seeds = prompts["seeds"].as_array().unwrap();
    let seed_values: Vec<i64> = seeds.iter().filter_map(|s| s.as_i64()).collect();

    for (cat_name, cat_data) in categories {
        let desc = cat_data["description"].as_str().unwrap();
        println!("  {} ({}):", desc, cat_name);

        let cat_prompts = cat_data["prompts"].as_array().unwrap();
        for prompt_entry in cat_prompts {
            let prompt_id = prompt_entry["id"].as_i64().unwrap();
            let prompt_text = prompt_entry["text"].as_str().unwrap();
            for &seed in &seed_values {
                println!("    [{:>2}] seed {}: {}", prompt_id, seed, prompt_text);
            }
        }
        println!();
    }

    let total: usize = categories.values()
        .map(|c| c["prompts"].as_array().unwrap().len() * seed_values.len())
        .sum();
    println!("Total: {} generations with {} seeds = {} sounds",
        categories.values().map(|c| c["prompts"].as_array().unwrap().len()).sum::<usize>(),
        seed_values.len(),
        total
    );
    println!("Results target: {}/{}", results_dir, provider);
}

fn get_arg_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == name).map(|w| w[1].clone())
}

fn find_prompts_json() -> PathBuf {
    let candidates = [
        PathBuf::from("tools/bakeoff/prompts.json"),
        PathBuf::from("../bakeoff/prompts.json"),
        PathBuf::from("prompts.json"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    PathBuf::from("prompts.json")
}

fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let time = secs % 86400;
    let hours = time / 3600;
    let mins = (time % 3600) / 60;
    let sec = time % 60;
    format!(
        "2025-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        (days / 30) % 12 + 1, days % 30 + 1, hours, mins, sec
    )
}
