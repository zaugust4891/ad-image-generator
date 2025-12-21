use anyhow::Result;
use clap::Parser;
use indicatif::MultiProgress;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_subscriber::EnvFilter;

mod backoff; mod config; mod dedupe; mod events; mod io; mod manifest; mod orchestrator; mod post; mod providers; mod prompts; mod rate_limit; mod rewrite; mod api;
use config::{RunCfg, TemplateYaml};

use providers::{ImageProvider, MockProvider, OpenAIProvider};
use prompts::{PromptTemplate, VariantGenerator};
use rewrite::{OpenAIRewriter};

#[derive(Parser, Debug)]
#[command(name = "adgen", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Run a single image-generation job (existing behavior)
    Run {
        #[arg(long)]
        config: PathBuf,

        #[arg(long)]
        template: PathBuf,

        #[arg(long)]
        out_dir: Option<PathBuf>,

        #[arg(long)]
        resume: bool,
    },

    /// Start the local HTTP API for the frontend
    Serve {
        #[arg(long, default_value = "127.0.0.1:8787")]
        bind: String,

        #[arg(long, default_value = "./run-config.yaml")]
        config_path: PathBuf,

        #[arg(long, default_value = "./template.yml")]
        template_path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let cli = Cli::parse();

    match cli.cmd {
        Command::Run { config, template, out_dir, resume } => {
            run_once(config, template, out_dir, resume, None, None).await
        }
        Command::Serve { bind, config_path, template_path } => {
            api::serve(bind, config_path, template_path).await
        }
    }
}

pub async fn run_once(
    config: PathBuf,
    template: PathBuf,
    out_dir: Option<PathBuf>,
    _resume: bool,
    run_id: Option<String>,
    events_tx: Option<broadcast::Sender<events::RunEvent>>,
) -> Result<()> {
    let run_id = run_id.unwrap_or_else(|| format!("run-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S")));
    let run_id_for_orch = run_id.clone();
    let events_for_orch = events_tx.clone();

    let result = async {
        let cfg: RunCfg = serde_yaml::from_str(&tokio::fs::read_to_string(&config).await?)?;
        let tpl_yaml: TemplateYaml = serde_yaml::from_str(&tokio::fs::read_to_string(&template).await?)?;
        let out_dir = out_dir.unwrap_or(cfg.clone().out_dir);
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

        let post = post::PostProcessor::new(cfg.post.thumbnail, cfg.post.thumb_max);
        let dedupe = if cfg.dedupe.enabled { Some(Arc::new(tokio::sync::Mutex::new(dedupe::PerceptualDeduper::new(cfg.dedupe.phash_bits, cfg.dedupe.phash_thresh)))) } else { None };
        let mp = MultiProgress::new();

        orchestrator::run_orchestrator(
            provider,
            generator,
            orchestrator::OrchestratorCfg{
                run_id: run_id_for_orch,
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
                events: events_for_orch,
            },
            orchestrator::OrchestratorExtras{ rewriter, post: Arc::new(post), dedupe },
        ).await?;

        println!("\n✅ Run complete.");
        Ok(())
    }.await;

    if let Err(ref e) = result {
        if let Some(tx) = &events_tx {
            let _ = tx.send(events::RunEvent::Failed { run_id: run_id.clone(), error: format!("{e:#}") });
        }
    }

    result
}
