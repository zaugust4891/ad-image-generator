use anyhow::Result;
use clap::Parser;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

mod providers;
mod orchestrator;
mod prompts;
mod rate_limit;
mod manifest;
mod post;
mod dedupe;
mod config;
mod backoff;
mod io; 

use providers::{ImageProvider, MockProvider, OpenAIProvider};
use orchestrator::{run_orchestrator_with_variants, OrchestratorParams, OrchestratorExtras};
use prompts::{PromptTemplate, VariantGenerator};
use post::{PostProcessor, PostOptions, ResizeCfg, OutFmt, WatermarkCfg};
use config::{RunConfig, TemplateYaml, VariantModeYaml, OutFmtYaml, choose_ext};

#[derive(Parser, Debug)]
#[command(name="adgen", version)]
struct Cli {
    /// Run configuration YAML
    #[arg(long)]
    config: PathBuf,
    /// Prompt template YAML
    #[arg(long)]
    template: PathBuf,
    /// Optional explicit output directory (overrides config.out_dir)
    #[arg(long)]
    out_dir: Option<PathBuf>,
}

fn pick_out_dir(cli_out: &Option<PathBuf>, cfg_out: &Option<PathBuf>) -> PathBuf {
    if let Some(x) = cli_out { return x.clone(); }
    if let Some(x) = cfg_out { return x.clone(); }
    let base = PathBuf::from("out");
    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    base.join(format!("run-{}", ts))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Logs
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    let args = Cli::parse();

    // Load YAMLs
    let cfg_text = tokio::fs::read_to_string(&args.config).await?;
    let mut cfg: RunConfig = serde_yaml::from_str(&cfg_text)?;

    let tpl_text = tokio::fs::read_to_string(&args.template).await?;
    let tpl_yml: TemplateYaml = serde_yaml::from_str(&tpl_text)?;

    // Merge env overrides (examples; add more if you like)
    if let Ok(v) = std::env::var("ADGEN_TARGET") {
        if let Ok(n) = v.parse() { cfg.orchestrator.target_images = n; }
    }
    if let Ok(v) = std::env::var("ADGEN_CONCURRENCY") {
        if let Ok(n) = v.parse() { cfg.orchestrator.concurrency = n; }
    }
    if let Ok(v) = std::env::var("ADGEN_RATE_PER_MIN") {
        if let Ok(n) = v.parse() { cfg.orchestrator.rate_per_min = n; }
    }

    let out_dir = pick_out_dir(&args.out_dir, &cfg.out_dir);
    tokio::fs::create_dir_all(&out_dir).await.ok();
    info!(out = %out_dir.display(), "Output directory");

    // Build PromptTemplate & VariantGenerator
    let tpl = PromptTemplate {
        brand: tpl_yml.brand,
        product: tpl_yml.product,
        audience: tpl_yml.audience,
        style: tpl_yml.style,
        background: tpl_yml.background,
        cta: tpl_yml.cta,
    };

    let mode = match cfg.variant_mode {
        VariantModeYaml::Random => "random",
        VariantModeYaml::Cartesian => "cartesian",
    };
    let gen = match mode {
        "random" => VariantGenerator::new_random(tpl, cfg.seed.unwrap_or(42)),
        _ => VariantGenerator::new_cartesian(tpl),
    };

    // Provider selection + price
    let price_usd = cfg.provider.price_usd_per_image.unwrap_or(0.0);
    let provider: Arc<dyn ImageProvider> = match cfg.provider.kind {
        config::ProviderKind::Mock => Arc::new(MockProvider),
        config::ProviderKind::Openai => {
            let key = std::env::var("OPENAI_API_KEY")
                .expect("OPENAI_API_KEY not set for openai provider");
            let model = cfg.provider.openai_model.clone().unwrap_or_else(|| "gpt-image-1".into());
            let size  = cfg.provider.openai_size.clone().unwrap_or_else(|| "1024x1024".into());
            Arc::new(OpenAIProvider::new(key, model, size))
        }
    };

    // Post-processing
    let fmt = match cfg.post.fmt {
        OutFmtYaml::Png => OutFmt::Png,
        OutFmtYaml::Jpeg => OutFmt::Jpeg(cfg.post.jpeg_quality.unwrap_or(90).clamp(1,100)),
        OutFmtYaml::Webp => OutFmt::Webp,
    };
    let ext = choose_ext(&cfg.post.fmt);
    let watermark = match (&cfg.post.watermark_text, &cfg.post.watermark_font) {
        (Some(text), Some(font_path)) => Some(WatermarkCfg {
            text: text.clone(),
            font_path: font_path.clone(),
            px: cfg.post.watermark_px.unwrap_or(28.0),
            margin: cfg.post.watermark_margin.unwrap_or(24),
        }),
        _ => None,
    };
    let post = PostProcessor::new(PostOptions {
        resize: ResizeCfg { width: cfg.post.width, height: cfg.post.height },
        watermark,
        fmt,
    });

    let run_id = format!("{}-{}", provider.name(), Utc::now().timestamp());

    run_orchestrator_with_variants(
        provider,
        run_id,
        out_dir,
        Arc::new(Mutex::new(gen)),
        OrchestratorParams {
            target_images: cfg.orchestrator.target_images,
            concurrency: cfg.orchestrator.concurrency,
            queue_cap: cfg.orchestrator.queue_cap,
            rate_per_min: cfg.orchestrator.rate_per_min,
            // new backoff fields will be read inside orchestrator via params pass-through or captured cfg
        },
        OrchestratorExtras {
            post: Arc::new(post),
            dedupe: if cfg.dedupe.enabled {
                Some(Arc::new(dedupe::PerceptualDeduper::new(
                    cfg.dedupe.phash_bits,
                    cfg.dedupe.phash_thresh,
                )))
            } else { None }
        },
        cfg.resume,
        // pass-thru extras we need at worker time:
        Some(ext.to_string()),
        price_usd,
        cfg.orchestrator.backoff_base_ms,
        cfg.orchestrator.backoff_factor,
        cfg.orchestrator.backoff_jitter_ms,
    ).await
}
