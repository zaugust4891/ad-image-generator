use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::{sync::{mpsc, Semaphore}, task::JoinSet};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;
use crate::events::RunEvent;
use crate::{providers::ImageProvider, prompts::VariantGenerator, io::save_image_with_sidecar, manifest::{Manifest, ManifestRecord}, rate_limit::SimpleRateLimiter};
use crate::backoff::backoff_ms;

pub struct OrchestratorCfg{
    pub run_id: String,
    pub out_dir: std::path::PathBuf,
    pub target_images: u64,
    pub concurrency: usize,
    pub queue_cap: usize,
    pub rate_per_min: u32,
    pub price_usd_per_image: f64,
    pub backoff_base_ms: u64,
    pub backoff_factor: f64,
    pub backoff_jitter_ms: u64,
    pub progress: Option<MultiProgress>,
    pub events: Option<broadcast::Sender<RunEvent>>,
}

pub struct OrchestratorExtras{
    pub rewriter: Option<Arc<dyn crate::rewrite::PromptRewriter>>,
    pub rewriter_model: Option<String>,
    pub rewriter_system: Option<String>,
    pub rewrite_cache: Option<Arc<crate::rewrite::RewriteCache>>,
    pub post: Arc<crate::post::PostProcessor>,
    pub dedupe: Option<Arc<tokio::sync::Mutex<crate::dedupe::PerceptualDeduper>>>,
}

pub async fn run_orchestrator(
    provider: Arc<dyn ImageProvider>,
    mut generator: VariantGenerator,
    cfg: OrchestratorCfg,
    extras: OrchestratorExtras,
) -> Result<()> {
    let done = Arc::new(AtomicU64::new(0));
    let sem = Arc::new(Semaphore::new(cfg.concurrency));
    let (tx, mut rx) = mpsc::channel::<(u64, String)>(cfg.queue_cap);
    let limiter = Arc::new(SimpleRateLimiter::per_minute(cfg.rate_per_min));
    let manifest = Arc::new(Manifest::new(&cfg.out_dir));
    let pb = cfg.progress.as_ref().map(|mp|{
        let pb = mp.add(ProgressBar::new(cfg.target_images));
        pb.set_style(ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} {msg}").unwrap());
        pb
    });
    emit(&cfg.events, RunEvent::Started {
        run_id: cfg.run_id.clone(),
        total: cfg.target_images,
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
        let events = cfg.events.clone();
        let total = cfg.target_images;
        let done = done.clone();
        let extras = OrchestratorExtras{
            rewriter: extras.rewriter.clone(),
            rewriter_model: extras.rewriter_model.clone(),
            rewriter_system: extras.rewriter_system.clone(),
            rewrite_cache: extras.rewrite_cache.clone(),
            post: extras.post.clone(),
            dedupe: extras.dedupe.clone(),
        };
        let price = cfg.price_usd_per_image;
        let backoff_base_ms = cfg.backoff_base_ms;
        let backoff_factor = cfg.backoff_factor;
        let backoff_jitter_ms = cfg.backoff_jitter_ms;
        set.spawn(async move {
            emit(&events, RunEvent::Log { run_id: run_id.clone(), msg: format!("#{id} generated prompt") });

            let _permit = sem.acquire().await.unwrap();
            limiter.wait().await;
            let mut prompt_used = original.clone();
            let mut rewritten: Option<String> = None;
            if let Some(rw) = &extras.rewriter {
                // Generate cache key
                let cache_key = crate::rewrite::cache_key(
                    &original,
                    rw.name(),
                    extras.rewriter_model.as_deref().unwrap_or(""),
                    extras.rewriter_system.as_deref().unwrap_or(""),
                );

                // Check cache first
                let cached = if let Some(cache) = &extras.rewrite_cache {
                    cache.get(&cache_key).await
                } else {
                    None
                };

                let maybe = if let Some(cached_val) = cached {
                    emit(&events, RunEvent::Log { run_id: run_id.clone(), msg: format!("#{id} rewrite: cache hit") });
                    cached_val
                } else {
                    emit(&events, RunEvent::Log { run_id: run_id.clone(), msg: format!("#{id} rewrite: calling API") });
                    let result = rw.rewrite(&original).await.unwrap_or(original.clone());
                    // Store in cache
                    if let Some(cache) = &extras.rewrite_cache {
                        if let Err(e) = cache.put(&cache_key, &result).await {
                            emit(&events, RunEvent::Log {
                                run_id: run_id.clone(),
                                msg: format!("#{id} rewrite: cache write error: {e:#}")
                            });
                        }
                    }
                    result
                };

                if maybe != original {
                    rewritten = Some(maybe.clone());
                    prompt_used = maybe;
                    emit(&events, RunEvent::Log { run_id: run_id.clone(), msg: format!("#{id} rewrite: changed") });
                }
            }

            emit(&events, RunEvent::Log { run_id: run_id.clone(), msg: format!("#{id} provider: call") });
            // call provider with retry + backoff
            const MAX_RETRIES: u32 = 3;
            let mut last_error = None;
            let mut attempt = 1;
            let res = loop {
                match provider.generate(&prompt_used).await {
                    Ok(r) => break Some(r),
                    Err(e) => {
                        last_error = Some(e);
                        if attempt >= MAX_RETRIES {
                            break None;
                        }
                        let delay_ms = backoff_ms(attempt, backoff_base_ms, backoff_factor, backoff_jitter_ms);
                        emit(&events, RunEvent::Log {
                            run_id: run_id.clone(),
                            msg: format!("#{id} provider error (attempt {}/{}), retrying in {}ms", attempt, MAX_RETRIES, delay_ms)
                        });
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                        attempt += 1;
                    }
                }
            };
            let res = match res {
                Some(r) => r,
                None => {
                    emit(&events, RunEvent::Log {
                        run_id: run_id.clone(),
                        msg: format!("#{id} provider failed after {} attempts: {:#}", MAX_RETRIES, last_error.unwrap())
                    });
                    return;
                }
            };
            // dedupe
            if let Some(d) = &extras.dedupe {
                let dup = d.lock().await.is_duplicate(&res.bytes).unwrap_or(false);
                if dup {
                    emit(&events, RunEvent::Log { run_id: run_id.clone(), msg: format!("#{id} dedupe: dropped") });
                    return;
                }
            }

            // generate thumbnail if enabled
            let thumbnail = match extras.post.maybe_thumbnail(&res.bytes) {
                Ok(thumb) => thumb,
                Err(e) => {
                    emit(&events, RunEvent::Log {
                        run_id: run_id.clone(),
                        msg: format!("#{id} thumbnail error: {e:#}")
                    });
                    None
                }
            };

            // save
            if let Err(e) = save_image_with_sidecar(&out_dir, &run_id, id, provider.name(), &res, &original, rewritten.as_deref(), price, thumbnail.as_deref()).await {
                emit(&events, RunEvent::Log {
                    run_id: run_id.clone(),
                    msg: format!("#{id} save error: {e:#}")
                });
                return;
            }
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            emit(&events, RunEvent::Progress {
                run_id: run_id.clone(),
                done: n,
                total,
                cost_so_far: n as f64 * price,
            });
            emit(&events, RunEvent::Log { run_id: run_id.clone(), msg: format!("#{id} saved (done {n}/{total})") });

            if let Err(e) = manifest.append(ManifestRecord{
                id, created_at: chrono::Utc::now().to_rfc3339(), provider: provider.name(),
                model: provider.model(), prompt: &prompt_used, path_png: format!("{:08}-{}-{}.png", id, provider.name(), provider.model()),
            }).await {
                emit(&events, RunEvent::Log {
                    run_id: run_id.clone(),
                    msg: format!("#{id} manifest append error: {e:#}")
                });
            }
            if let Some(pb) = &pb { pb.inc(1); }
        });
    }
    producer.await.ok();
    while let Some(_r) = set.join_next().await {}
    if let Some(pb) = pb { pb.finish_with_message("done"); }
    emit(&cfg.events, RunEvent::Finished { run_id: cfg.run_id.clone() });
    Ok(())
}

fn emit(events: &Option<broadcast::Sender<RunEvent>>, evt: RunEvent) {
    if let Some(tx) = events {
        let _ = tx.send(evt); // ignore if no listeners
    }
}
