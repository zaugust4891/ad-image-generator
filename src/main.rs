use std::path::PathBuf;
use anyhow::Result;
use chrono::Utc;
use std::sync::{Arc, Mutex};

mod providers;
mod io;
mod orchestrator;
mod prompts;
mod rate_limit;
mod manifest;
mod post;
mod dedupe;

use providers::{ImageProvider, MockProvider, OpenAIProvider};
use orchestrator::{run_orchestrator_with_variants, OrchestratorParams, OrchestratorExtras};
use prompts::{PromptTemplate, VariantGenerator};
use post::{PostProcessor, PostOptions, ResizeCfg, OutFmt, WatermarkCfg};

fn pick_out_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ADGEN_OUT_DIR") { return PathBuf::from(dir); }
    let base = PathBuf::from("out");
    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    base.join(format!("run-{}", ts))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Core config
    let target_images: u64 = std::env::var("ADGEN_TARGET").ok().and_then(|v| v.parse().ok()).unwrap_or(24);
    let concurrency: usize = std::env::var("ADGEN_CONCURRENCY").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    let queue_cap: usize = std::env::var("ADGEN_QUEUE_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(64);
    let rate_per_min: u32 = std::env::var("ADGEN_RATE_PER_MIN").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let provider_name: String = std::env::var("ADGEN_PROVIDER").unwrap_or_else(|_| "mock".into());
    let mode: String = std::env::var("ADGEN_MODE").unwrap_or_else(|_| "cartesian".into());
    let seed: u64 = std::env::var("ADGEN_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(42);
    let resume: bool = std::env::var("ADGEN_RESUME").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);

    // Post-processing config via env (optional)
    let resize_w: Option<u32> = std::env::var("ADGEN_RESIZE_W").ok().and_then(|v| v.parse().ok());
    let resize_h: Option<u32> = std::env::var("ADGEN_RESIZE_H").ok().and_then(|v| v.parse().ok());
    let out_fmt: String = std::env::var("ADGEN_FMT").unwrap_or_else(|_| "png".into()); // png|jpg|webp
    let jpeg_q: u8 = std::env::var("ADGEN_JPEG_Q").ok().and_then(|v| v.parse().ok()).unwrap_or(90);
    let wm_text: Option<String> = std::env::var("ADGEN_WATERMARK_TEXT").ok();
    let wm_font: Option<String> = std::env::var("ADGEN_WATERMARK_FONT").ok();
    let wm_px: f32 = std::env::var("ADGEN_WATERMARK_PX").ok().and_then(|v| v.parse().ok()).unwrap_or(28.0);
    let wm_margin: u32 = std::env::var("ADGEN_WATERMARK_MARGIN").ok().and_then(|v| v.parse().ok()).unwrap_or(24);

    // Dedupe config
    let phash_bits: u32 = std::env::var("ADGEN_PHASH_BITS").ok().and_then(|v| v.parse().ok()).unwrap_or(64); // 8x8
    let phash_thresh: u32 = std::env::var("ADGEN_PHASH_THRESH").ok().and_then(|v| v.parse().ok()).unwrap_or(6);
    let enable_dedupe: bool = std::env::var("ADGEN_DEDUPE").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(true);

    let out_dir = pick_out_dir();
    tokio::fs::create_dir_all(&out_dir).await.ok();
    println!("Output directory: {}", out_dir.display());

    // Template (same demo)
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

    // Post-processing options
    let fmt = match out_fmt.as_str() {
        "jpg" | "jpeg" => OutFmt::Jpeg(jpeg_q),
        "webp" => OutFmt::Webp,
        _ => OutFmt::Png,
    };
    let watermark = match (wm_text, wm_font) {
        (Some(text), Some(font_path)) => Some(WatermarkCfg { text, font_path: font_path.into(), px: wm_px, margin: wm_margin }),
        _ => None,
    };
    let post = PostProcessor::new(PostOptions {
        resize: ResizeCfg { width: resize_w, height: resize_h },
        watermark,
        fmt,
    });

    // Dedupe
    let dedupe = if enable_dedupe {
        Some(Arc::new(dedupe::PerceptualDeduper::new(phash_bits, phash_thresh)))
    } else { None };

    let run_id = format!("{}-{}", provider.name(), Utc::now().timestamp());

    run_orchestrator_with_variants(
        provider,
        run_id,
        out_dir,
        Arc::new(Mutex::new(gen)),
        orchestrator::OrchestratorParams { target_images, concurrency, queue_cap, rate_per_min },
        OrchestratorExtras { post: Arc::new(post), dedupe },
        resume,
    ).await
}
