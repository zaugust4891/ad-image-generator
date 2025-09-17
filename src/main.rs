use std::path::{PathBuf};
use anyhow::Result;
use chrono::Utc;
use std::sync::{Arc, Mutex};

mod providers;
mod io;
mod orchestrator;
mod prompts;
mod rate_limit;
mod manifest;

use providers::{ImageProvider, MockProvider, OpenAIProvider};
use orchestrator::{run_orchestrator_with_variants, OrchestratorParams};
use prompts::{PromptTemplate, VariantGenerator};

fn pick_out_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ADGEN_OUT_DIR") {
        return PathBuf::from(dir);
    }
    let base = PathBuf::from("out");
    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    base.join(format!("run-{}", ts))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Config
    let target_images: u64 = std::env::var("ADGEN_TARGET").ok().and_then(|v| v.parse().ok()).unwrap_or(24);
    let concurrency: usize = std::env::var("ADGEN_CONCURRENCY").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let queue_cap: usize = std::env::var("ADGEN_QUEUE_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(64);
    let rate_per_min: u32 = std::env::var("ADGEN_RATE_PER_MIN").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let provider_name: String = std::env::var("ADGEN_PROVIDER").unwrap_or_else(|_| "mock".into());
    let mode: String = std::env::var("ADGEN_MODE").unwrap_or_else(|_| "cartesian".into());
    let seed: u64 = std::env::var("ADGEN_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(42);
    let resume: bool = std::env::var("ADGEN_RESUME").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);

    let out_dir = pick_out_dir();
    tokio::fs::create_dir_all(&out_dir).await.ok();
    println!("Output directory: {}", out_dir.display());

    // Template (inline demo)
    let tpl = PromptTemplate {
        brand: "Sierra Sparkling Water".into(),
        product: "12oz Lime".into(),
        audience: vec!["fitness enthusiasts".into(), "busy professionals".into(), "students".into()],
        style: vec!["studio lighting".into(), "cinematic".into(), "flat lay".into()],
        background: vec!["granite countertop".into(), "gym bench".into(), "wood table".into()],
        cta: vec!["Refresh naturally".into(), "Zero sugar. Full flavor.".into(), "Hydrate different.".into()],
    };

    let gen = match mode.as_str() {
        "random" => VariantGenerator::new_random(tpl, seed),
        _ => VariantGenerator::new_cartesian(tpl),
    };

    // Provider
    let provider: Arc<dyn ImageProvider> = match provider_name.as_str() {
        "openai" => {
            let key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
            let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-image-1".into());
            let size  = std::env::var("OPENAI_SIZE").unwrap_or_else(|_| "1024x1024".into());
            Arc::new(OpenAIProvider::new(key, model, size))
        }
        _ => Arc::new(MockProvider),
    };

    let run_id = format!("{}-{}", provider.name(), Utc::now().timestamp());

    run_orchestrator_with_variants(
        provider,
        run_id,
        out_dir,
        Arc::new(Mutex::new(gen)),
        OrchestratorParams { target_images, concurrency, queue_cap, rate_per_min },
        resume,
    ).await
}
