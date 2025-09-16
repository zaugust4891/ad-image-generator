use std::{path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicU64, Ordering}, Mutex}};
use anyhow::Result;
use tokio::{sync::{mpsc, Semaphore}, task::JoinSet, time::{sleep, Duration}};

use crate::{io::save_output, providers::{ImageProvider, ProviderError}, prompts::VariantGenerator};

#[derive(Debug, Clone)]
pub struct ImageJob {
	pub id: u64,
	pub prompt: String,
}

pub async fn run_orchestrator(
	provider: Arc<dyn ImageProvider>,
	run_id: String,
	out_dir: PathBuf,
	generator: Arc<Mutex<VariantGenerator>>, // thread-safe prompt source
	target_images: u64,
	concurrency: usize,
	queue_cap: usize,
) -> Result<()> {
	let (tx, mut rx) = mpsc::channel::<ImageJob>(queue_cap);
	let tx_arc = Arc::new(tx); // keep shareable Sender
	let sem = Arc::new(Semaphore::new(concurrency));
	let completed = Arc::new(AtomicU64::new(0));
	let next_id = Arc::new(AtomicU64::new(1));

	// Seed initial batch: up to min(target, 2×concurrency, queue_cap)
	let initial = std::cmp::min(
		target_images,
		std::cmp::min((concurrency as u64) * 2, queue_cap as u64),
	);
	for _ in 0..initial {
		if let Some(p) = { generator.lock().unwrap().next() } {
			let id = next_id.fetch_add(1, Ordering::Relaxed);
			tx.send(ImageJob { id, prompt: p }).await?;
		}
	}

	// Drop our local strong reference; tasks keep tx_arc clones
	let tx_task = tx_arc.clone();
	drop(tx_arc); 

	let mut joinSet = JoinSet::new();

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
		let target_images_local = target_images;

		joinset.spawn(async move { 
		// Ensure permit is held for the duration of the task.
			let _permit = permit;

			// Retry policy (simple): up to 3 attempts with backoff
			let mut last_err: Option<anyhow::Error> = None;
			for attempt in 1..=3u32 {
				match provider.generate(&job.prompt).await {
					Ok(res) => {
						if let Err(e) = save_output(&out_dir, job.id, &run_id, &res).await {
							last_err = Some(e);
						} else {
							let done = completed.fetch_add(1, Ordering::Relaxed) + 1;

							// If we still need more, enqueue the next variant
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

	println!("all jobs drained. Completed: {}", completed.load(Ordering::Relaxed));
	Ok(())
}




