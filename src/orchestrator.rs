use std::{path::PathBuf, sync::{Arc, atomic::{AtomicU64, Ordering}, Mutex}};
use anyhow::Result;
use tokio::{sync::{mpsc, Semaphore}, task::JoinSet, time::{sleep, Duration}};

use crate::{io::save_output, providers::{ImageProvider, ProviderError}};
use crate::prompts::VariantGenerator;
use crate::rate_limit::SimpleRateLimiter;
use crate::manifest::{ManifestWriter, ManifestRecord};
use crate::post::{PostProcessor};
use crate::dedupe::PerceptualDeduper;

#[derive(Debug, Clone)]
pub struct ImageJob { 
    pub id: u64,
    pub prompt: String
}

pub struct OrchestratorParams {
    pub target_images: u64,
    pub concurrency: usize,
    pub queue_cap: usize,
    pub rate_per_min: u32,
}

pub struct OrchestratorExtras {
    pub post: Arc<PostProcessor>,
    pub dedupe: Option<Arc<PerceptualDeduper>>,
}

pub async fn run_orchestrator_with_variants(
    provider: Arc<dyn ImageProvider>,
    run_id: String,
    out_dir: PathBuf,
    generator: Arc<Mutex<VariantGenerator>>,
    params: OrchestratorParams,
    extras: OrchestratorExtras,
    resume: bool,
) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<ImageJob>(params.queue_cap);
    let tx_arc = Arc::new(tx);
    let sem = Arc::new(Semaphore::new(params.concurrency));
    let limiter = Arc::new(SimpleRateLimiter::per_minute(params.rate_per_min.max(1)));

    // Manifest
    let manifest_path = out_dir.join("manifest.jsonl");
    let manifest = ManifestWriter::open(manifest_path.clone()).await?;

    // Resume
    let (already_completed, start_id) = if resume {
        crate::manifest::resume_state(&manifest_path).await?
    } else { (0, 1) };

    let completed = Arc::new(AtomicU64::new(already_completed));
    let next_id = Arc::new(AtomicU64::new(start_id));
    let skipped_dupes = Arc::new(AtomicU64::new(0));

    // Seed initial
    let remaining = params.target_images.saturating_sub(already_completed);
    let initial = std::cmp::min(remaining, std::cmp::min((params.concurrency as u64) * 2, params.queue_cap as u64));
    for _ in 0..initial {
        if let Some(p) = { generator.lock().unwrap().next() } {
            let id = next_id.fetch_add(1, Ordering::Relaxed);
            tx_arc.send(ImageJob { id, prompt: p }).await?;
        }
    }
    let tx_task = tx_arc.clone();
    drop(tx_arc);

    let mut joinset = JoinSet::new();

    while let Some(job) = rx.recv().await {
        if completed.load(Ordering::Relaxed) >= params.target_images { break; }



        let permit = sem.clone().acquire_owned().await.expect("semaphore");
        let provider = provider.clone();
        let out_dir = out_dir.clone();
        let run_id = run_id.clone();
        let completed = completed.clone();
        let generator = generator.clone();
        let next_id = next_id.clone();
        let tx_for_task = tx_task.clone();
        let limiter = limiter.clone();
        let manifest = manifest.clone();
        let post = extras.post.clone();
        let dedupe = extras.dedupe.clone();
        let skipped_dupes = skipped_dupes.clone();
        let target_images_local = params.target_images;

        joinset.spawn(async move {
            let _permit = permit;

            limiter.wait().await;

            let mut last_err: Option<anyhow::Error> = None;
            for attempt in 1..=3u32 {
                match provider.generate(&job.prompt).await {
                    Ok(res_raw) => {
                        // Post-process (resize/watermark/format)
                        let (processed_bytes, new_w, new_h) = match post.process(&res_raw.bytes) {
                            Ok(v) => v,
                            Err(e) => { last_err = Some(e); continue; }
                        };

                        // Dedupe (optional)
                        let mut phash: Option<String> = None;
                        if let Some(d) = &dedupe {
                            match d.check_and_insert(&processed_bytes) {
                                Ok((is_dup, h)) => {
                                    phash = Some(h);
                                    if is_dup {
                                        // Count and DO NOT save; enqueue next if needed
                                        let done_so_far = completed.load(Ordering::Relaxed);
                                        skipped_dupes.fetch_add(1, Ordering::Relaxed);
                                        if done_so_far < target_images_local {
                                            if let Some(next_prompt) = { generator.lock().unwrap().next() } {
                                                let new_id = next_id.fetch_add(1, Ordering::Relaxed);
                                                let _ = tx_for_task.send(ImageJob { id: new_id, prompt: next_prompt }).await;
                                            }
                                        }
                                        return; // skip saving duplicates
                                    }
                                }
                                Err(e) => { last_err = Some(e); continue; }
                            }
                        }

                        // Build a new ImageResult-like view for save_output sizing
                        let res_for_save = crate::providers::ImageResult {
                            bytes: processed_bytes.clone(),
                            width: if new_w > 0 { new_w } else { res_raw.width },
                            height: if new_h > 0 { new_h } else { res_raw.height },
                            prompt_used: res_raw.prompt_used.clone(),
                            model: res_raw.model.clone(),
                        };

                        if let Err(e) = save_output(&out_dir, job.id, &run_id, &res_for_save).await {
                            last_err = Some(e);
                        } else {
                            let done = completed.fetch_add(1, Ordering::Relaxed) + 1;

                            let rec = ManifestRecord {
                                id: job.id,
                                run_id: &run_id,
                                prompt: &res_for_save.prompt_used,
                                model: &res_for_save.model,
                                width: res_for_save.width,
                                height: res_for_save.height,
                                created_at: chrono::Utc::now().to_rfc3339(),
                                cost_usd: None,
                                phash: phash.as_deref(),
                            };
                            let _ = manifest.append(&rec).await;

                            if done < target_images_local {
                                if let Some(next_prompt) = { generator.lock().unwrap().next() } {
                                    let new_id = next_id.fetch_add(1, Ordering::Relaxed);
                                    let _ = tx_for_task.send(ImageJob { id: new_id, prompt: next_prompt }).await;
                                }
                            }
                            return; // success
                        }
                    }
                    Err(ProviderError::RateLimited) | Err(ProviderError::Http(_)) => {
                        sleep(Duration::from_millis(250 * attempt as u64)).await;
                    }
                    Err(ProviderError::Fatal(msg)) => {
                        last_err = Some(anyhow::anyhow!("fatal provider error: {msg}"));
                        break;
                    }
                }
            }
            if let Some(e) = last_err { eprintln!("job {} failed: {e}", job.id); }
        });
    }

    while let Some(_res) = joinset.join_next().await {}

    println!(
        "Completed {} images (target {}). Skipped dupes: {}",
        completed.load(Ordering::Relaxed),
        params.target_images,
        skipped_dupes.load(Ordering::Relaxed),
    );
    Ok(())
}
