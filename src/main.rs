use std::path::PathBuf;
mod providers;
mod io;
mod orchestrator;
mod prompts;


use providers::{ImageProvider, MockProvider};
use orchestrator::run_orchestrator_with_variants;
use prompts::{PromptTemplate, VariantGenerator, VariantMode};


fn new_run_dir(base: impl Into<PathBuf>) -> PathBuf {
    let base: PathBuf = base.into();
    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    base.join(format!("run-{}", ts))
}


#[tokio::main]
async fn main() -> Result<()> {
    let target_images: u64 = std::env::var("ADGEN_TARGET").ok().and_then(|v| v.parse().ok()).unwrap_or(48);
    let concurrency: usize = std::env::var("ADGEN_CONCURRENCY").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let queue_cap: usize = std::env::var("ADGEN_QUEUE_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(64);
    let mode: String = std::env::var("ADGEN_MODE").unwrap_or_else(|_| "cartesian".into());
    let seed: u64 = std::env::var("ADGEN_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(42);


    let out_base = PathBuf::from("out");
    let run_dir = new_run_dir(&out_base);
    println!("Output directory: {}", run_dir.display());


    // Example template (inline for now)
    let tpl = PromptTemplate {
        brand: "Sierra Sparkling Water".to_string(),
        product: "12oz Lime".to_string(),
        audience: vec![
        "fitness enthusiasts".into(),
        "busy professionals".into(),
        "students".into(),
        ],
        style: vec!["studio lighting".into(), "cinematic".into(), "flat lay".into()],
        background: vec!["granite countertop".into(), "gym bench".into(), "wood table".into()],
        cta: vec!["Refresh naturally".into(), "Zero sugar. Full flavor.".into(), "Hydrate different.".into()],
    };

    let gen = match mode.as_str() {
    "random" => VariantGenerator::new_random(tpl, seed),
    _ => VariantGenerator::new_cartesian(tpl),
    };


    let provider = Arc::new(MockProvider) as Arc<dyn providers::ImageProvider>;
    let run_id = format!("{}-{}", provider.name(), Utc::now().timestamp());


    run_orchestrator_with_variants(
        provider,
        run_id,
        run_dir,
        Arc::new(Mutex::new(gen)),
        target_images,
        concurrency,
        queue_cap,
    ).await
}