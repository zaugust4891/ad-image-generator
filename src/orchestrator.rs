use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::{sync::{mpsc, Semaphore}, task::JoinSet};

use crate::{providers::ImageProvider, prompts::VariantGenerator, io::save_image_with_sidecar, manifest::{Manifest, ManifestRecord}, rate_limit::SimpleRateLimiter};

pub struct OrchestratorCfg{
    pub run_id: String,
    pub out_dir: std::path::PathBuf,
    pub target_images: u64,
    pub concurrency: usize,
    pub queue_cap: usize,
    pub rate_per_min: u32,
    pub price_usd_per_image: f64,
    #[allow(unused)]
    pub backoff_base_ms: u64,
    #[allow(unused)]
    pub backoff_factor: f64,
    #[allow(unused)]
    pub backoff_jitter_ms: u64,
    pub progress: Option<MultiProgress>,
}

pub struct OrchestratorExtras{
    pub rewriter: Option<Arc<dyn crate::rewrite::PromptRewriter>>,
    pub post: Arc<crate::post::PostProcessor>,
    pub dedupe: Option<Arc<tokio::sync::Mutex<crate::dedupe::PerceptualDeduper>>>,
}

pub async fn run_orchestrator(
    provider: Arc<dyn ImageProvider>,
    mut generator: VariantGenerator,
    cfg: OrchestratorCfg,
    extras: OrchestratorExtras,
) -> Result<()> {
    let sem = Arc::new(Semaphore::new(cfg.concurrency));
    let (tx, mut rx) = mpsc::channel::<(u64, String)>(cfg.queue_cap);
    let limiter = Arc::new(SimpleRateLimiter::per_minute(cfg.rate_per_min));
    let manifest = Arc::new(Manifest::new(&cfg.out_dir));
    let pb = cfg.progress.as_ref().map(|mp|{
        let pb = mp.add(ProgressBar::new(cfg.target_images));
        pb.set_style(ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} {msg}").unwrap());
        pb
    });

    // Producer
    let producer = {
        let tx = tx.clone();
        tokio::spawn(async move {
            for id in 1..=cfg.target_images {
                let prompt = generator.next();
                if tx.send((id, prompt)).await.is_err() { break; }
            }
        })
    };

    // Dispatcher: receive jobs and spawn per-item tasks
    let mut set = JoinSet::new();
    drop(tx);
    while let Some((id, original)) = rx.recv().await {
        let provider = provider.clone();
        let sem = sem.clone();
        let out_dir = cfg.out_dir.clone();
        let run_id = cfg.run_id.clone();
        let manifest = manifest.clone();
        let limiter = limiter.clone();
        let pb = pb.clone();
        let extras = OrchestratorExtras{
            rewriter: extras.rewriter.clone(),
            post: extras.post.clone(),
            dedupe: extras.dedupe.clone(),
        };
        let price = cfg.price_usd_per_image;
        set.spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            limiter.wait().await;
            let mut prompt_used = original.clone();
            let mut rewritten: Option<String> = None;
            if let Some(rw) = &extras.rewriter {
                let maybe = rw.rewrite(&original).await.unwrap_or(original.clone());
                if maybe != original { rewritten = Some(maybe.clone()); prompt_used = maybe; }
            }
            // call provider
            let res = provider.generate(&prompt_used).await;
            let res = match res { Ok(r)=>r, Err(e)=>{ eprintln!("provider error: {e:?}"); return; } };
            // dedupe
            if let Some(d) = &extras.dedupe {
                let dup = d.lock().await.is_duplicate(&res.bytes).unwrap_or(false);
                if dup { return; }
            }
            // save
            if let Err(e) = save_image_with_sidecar(&out_dir, &run_id, id, provider.name(), &res, &original, rewritten.as_deref(), price).await {
                eprintln!("save error: {e:#}"); return;
            }
            let _ = manifest.append(ManifestRecord{
                id, created_at: chrono::Utc::now().to_rfc3339(), provider: provider.name(),
                model: provider.model(), prompt: &prompt_used, path_png: format!("{:08}-{}-{}.png", id, provider.name(), provider.model()),
            }).await;
            if let Some(pb) = &pb { pb.inc(1); }
        });
    }
    producer.await.ok();
    while let Some(_r) = set.join_next().await {}
    if let Some(pb) = pb { pb.finish_with_message("done"); }
    Ok(())
}
