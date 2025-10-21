use anyhow::Result;
use clap::Parser;
use indicatif::MultiProgress;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

mod backoff; mod config; mod dedupe; mod io; mod manifest; mod orchestrator; mod post; mod providers; mod prompts; mod rate_limit; mod rewrite;
use config::{RunCfg, TemplateYaml};
use orchestrator::{OrchestratorCfg, OrchestratorExtras};
use providers::{ImageProvider, MockProvider, OpenAIProvider};
use prompts::{PromptTemplate, VariantGenerator};
use rewrite::{OpenAIRewriter};

#[derive(Parser, Debug)]
#[command(name="adgen", version)]
struct Cli{
    #[arg(long)] config: PathBuf,
    #[arg(long)] template: PathBuf,
    #[arg(long)] out_dir: Option<PathBuf>,
    #[arg(long)] resume: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let cli = Cli::parse();
    let cfg: RunCfg = serde_yaml::from_str(&tokio::fs::read_to_string(&cli.config).await?)?;
    let tpl_yaml: TemplateYaml = serde_yaml::from_str(&tokio::fs::read_to_string(&cli.template).await?)?;
    let out_dir = cli.out_dir.unwrap_or(cfg.clone().out_dir);
    tokio::fs::create_dir_all(&out_dir).await?;

    // Provider
    let provider: Arc<dyn ImageProvider> = match cfg.provider.kind.as_str(){
        "mock" => Arc::new(MockProvider{ model: cfg.provider.model.clone().unwrap_or_else(||"mock-v1".into()), w: cfg.provider.width.unwrap_or(512), h: cfg.provider.height.unwrap_or(512) }),
        "openai" => {
            let key = std::env::var(cfg.provider.api_key_env.clone().unwrap_or_else(||"OPENAI_API_KEY".into()))?;
            Arc::new(OpenAIProvider{ client:reqwest::Client::new(), model: cfg.provider.model.clone().unwrap_or_else(||"gpt-image-1".into()), api_key: key, w: cfg.provider.width.unwrap_or(1024), h: cfg.provider.height.unwrap_or(1024), price: cfg.provider.price_usd_per_image.unwrap_or(0.0)})
        }
        other => anyhow::bail!("unknown provider: {other}"),
    };

    // Prompt generator
    let tpl = PromptTemplate{ brand: tpl_yaml.brand, product: tpl_yaml.product, styles: tpl_yaml.styles };
    let generator = VariantGenerator::new(tpl, cfg.seed);

    // Rewrite
    let rewriter: Option<Arc<dyn rewrite::PromptRewriter>> = if cfg.rewrite.enabled {
        let key = std::env::var(cfg.provider.api_key_env.clone().unwrap_or_else(||"OPENAI_API_KEY".into())).unwrap_or_default();
        Some(Arc::new(OpenAIRewriter::new(
            key,
            cfg.rewrite.model.clone().unwrap_or_else(||"gpt-4o-mini".into()),
            cfg.rewrite.system.clone().unwrap_or_else(||"Polish and improve the ad prompt while preserving its core intent.".into()),
            cfg.rewrite.max_tokens.unwrap_or(64),
        )))
    } else { None };

    // Post
    let post = post::PostProcessor::new(cfg.post.thumbnail, cfg.post.thumb_max);

    // Dedupe
    let dedupe = if cfg.dedupe.enabled { Some(Arc::new(tokio::sync::Mutex::new(dedupe::PerceptualDeduper::new(cfg.dedupe.phash_bits, cfg.dedupe.phash_thresh)))) } else { None };

    // Progress
    let mp = MultiProgress::new();

    // Run
    orchestrator::run_orchestrator(
        provider,
        generator,
        OrchestratorCfg{
            run_id: format!("run-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S")),
            out_dir,
            target_images: cfg.orchestrator.target_images,
            concurrency: cfg.orchestrator.concurrency,
            queue_cap: cfg.orchestrator.queue_cap,
            rate_per_min: cfg.orchestrator.rate_per_min,
            price_usd_per_image: cfg.provider.price_usd_per_image.unwrap_or(0.0),
            backoff_base_ms: cfg.orchestrator.backoff_base_ms,
            backoff_factor: cfg.orchestrator.backoff_factor,
            backoff_jitter_ms: cfg.orchestrator.backoff_jitter_ms,
            progress: Some(mp.clone()),
        },
        OrchestratorExtras{ rewriter, post: Arc::new(post), dedupe },
    ).await?;

    println!("\n✅ Run complete.");
    Ok(())
}
