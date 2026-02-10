use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize)]
struct SidecarData {
    run_id: String,
    provider: String,
    model: String,
    cost_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct CostSummary {
    pub total_cost: f64,
    pub image_count: u64,
    pub avg_cost_per_image: f64,
    pub runs: Vec<RunCost>,
    pub by_provider: Vec<ProviderCost>,
}

#[derive(Debug, Serialize)]
pub struct RunCost {
    pub run_id: String,
    pub cost: f64,
    pub image_count: u64,
}

#[derive(Debug, Serialize)]
pub struct ProviderCost {
    pub provider: String,
    pub model: String,
    pub cost: f64,
    pub image_count: u64,
}

pub async fn compute_cost_summary(out_dir: &Path) -> Result<CostSummary> {
    let mut total_cost = 0.0;
    let mut image_count: u64 = 0;
    let mut runs: HashMap<String, (f64, u64)> = HashMap::new();
    let mut providers: HashMap<(String, String), (f64, u64)> = HashMap::new();

    let mut rd = tokio::fs::read_dir(out_dir).await?;
    while let Some(entry) = rd.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        // Skip non-sidecar JSON (e.g. any config files that might be in out_dir)
        let bytes = match tokio::fs::read(&path).await {
            Ok(b) => b,
            Err(_) => continue,
        };
        let sidecar: SidecarData = match serde_json::from_slice(&bytes) {
            Ok(s) => s,
            Err(_) => continue, // skip files that don't match sidecar format
        };

        total_cost += sidecar.cost_usd;
        image_count += 1;

        let run_entry = runs.entry(sidecar.run_id).or_insert((0.0, 0));
        run_entry.0 += sidecar.cost_usd;
        run_entry.1 += 1;

        let prov_entry = providers
            .entry((sidecar.provider, sidecar.model))
            .or_insert((0.0, 0));
        prov_entry.0 += sidecar.cost_usd;
        prov_entry.1 += 1;
    }

    let mut runs_vec: Vec<RunCost> = runs
        .into_iter()
        .map(|(run_id, (cost, count))| RunCost {
            run_id,
            cost,
            image_count: count,
        })
        .collect();
    runs_vec.sort_by(|a, b| b.run_id.cmp(&a.run_id));

    let mut providers_vec: Vec<ProviderCost> = providers
        .into_iter()
        .map(|((provider, model), (cost, count))| ProviderCost {
            provider,
            model,
            cost,
            image_count: count,
        })
        .collect();
    providers_vec.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap_or(std::cmp::Ordering::Equal));

    let avg = if image_count > 0 {
        total_cost / image_count as f64
    } else {
        0.0
    };

    Ok(CostSummary {
        total_cost,
        image_count,
        avg_cost_per_image: avg,
        runs: runs_vec,
        by_provider: providers_vec,
    })
}

pub fn estimate_cost(target_images: u64, price_per_image: f64) -> f64 {
    target_images as f64 * price_per_image
}
