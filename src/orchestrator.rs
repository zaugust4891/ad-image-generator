use std::{path::PathBuf, sync::{Arc, atomic::{AtomicU64, Ordering}, Mutex}};
use anyhow::Result;
use tokio::{sync::{mpsc, Semaphore}, task::JoinSet, time::{sleep, Duration}};

use crate::{io::save_output, providers::{ImageProvider, ProviderError}};
use crate::prompts::VariantGenerator;
use crate::rate_limit::SimpleRateLimiter;
use crate::manifest::{ManifestWriter, ManifestRecord};

#[derive(Debug, Clone)]
pub struct ImageJob {
	pub id: u64,
	pub prompt: String,
}

pub struct OrchestratorParams {
	pub target_images: u64,
	pub concurrency: usize,
	pub queue_cap: usize,
	pub rate_per_min: u32,

}

pub async fn run_orchestrator_with_variants(
	provider: Arc<dyn ImageProvider>,
	run_id: String,
	out_dir: PathBuf,
	generator: Arc<Mutex<VariantGenerator>>, // thread-safe prompt source
	target_images: u64,
	params: OrchestratorParams,
	resume: bool,
) -> Result<()> {
	let (tx, mut rx) = mpsc::channel::<ImageJob>(params.queue_cap);
	let tx_arc = Arc::new(tx); // keep shareable Sender
	let sem = Arc::new(Semaphore::new(params.concurrency));
	let limiter = Arc::new(SimpleRateLimiter::per_minute(params.rate_per_min.max(1)));

	// Manifest: always under out_dir/manifest.jsonl
    let manifest_path = out_dir.join("manifest.jsonl");
    let manifest = ManifestWriter::open(manifest_path.clone()).await?;

    // Resume state
    let (already_completed, start_id) = if resume {
        crate::manifest::resume_state(&manifest_path).await?
    } else { (0, 1) };

    let completed = Arc::new(AtomicU64::new(already_completed));
    let next_id = Arc::new(AtomicU64::new(start_id));

    // Seed initial batch based on remaining
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


	// Receiver
	while let Some(job) = rx.recv().await {
		// Stop early if somehow we overfilled (defensive)
		if completed.load(Ordering::Relaxed) >= target_images { break; }

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
		let target_images_local = params.target_images;

		joinset.spawn(async move { 
		// Ensure permit is held for the duration of the task.
			let _permit = permit;

			// Global rate limit (coarse)
			let limiter.wait().await;

			let mut last_err: Option<anyhow::Error> = None;
			for attempt in 1..=3u32 {
				match provider.generate(&job.prompt).await {
					Ok(res) => {
						if let Err(e) = save_output(&out_dir, job.id, &run_id, &res).await {
							last_err = Some(e);
						} else {
							let done = completed.fetch_add(1, Ordering::Relaxed) + 1;

							// Append manifest (best-effort)
                            let rec = ManifestRecord {
                                id: job.id,
                                run_id: &run_id,
                                prompt: &res.prompt_used,
                                model: &res.model,
                                width: res.width,
                                height: res.height,
                                created_at: chrono::Utc::now().to_rfc3339(),
                                cost_usd: None,
                            };

                            let _ = manifest.append(&rec).await;

							// Enqueue nex if needed
							if dont < target_images_local {
								if let Some(next_prompt) = { generator.lock().unwrap().next() } {
									let new_id = next_id.fetch_add(1, Ordering::Relaxed);
									// If receiver has closed, ignore send error
									let _ = tx_for_task.send(ImageJob { id: new_id, prompt: next_prompt }).await;
								}
							}
							return; // success
						}
					}
					Err(ProviderError::RateLimited) | Err(ProviderError::Http(_)) => {
						// tansient -> backoff
						sleep(Duration::from_millis(250 * attempt as u64)).await;
					}
					Err(ProviderError::Fatal(msg)) => {
						last_err = Some(anyhow::anyhow!("fatal provider error {msg}"));
						break;
					}
				}
			}
			if let Some(e) = last_err { eprintln!("job {} failed: {e}", job.id); }
		});
	}

	// Wait for all in-flight tasks to finish
	while let Some(_res) = joinset.join_next().await {}

	println!(
        "Completed {} images (target {}). Manifest: {}/manifest.jsonl",
        completed.load(Ordering::Relaxed),
        params.target_images,
        out_dir.display()
    );
	Ok(())
}




