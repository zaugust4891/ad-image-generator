use chrono::Utc;
use serde::Serialize;
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt};
use crate::providers::ImageResult;

#[derive(Serialize)]
struct Sidecar<'a> {
    id: u64,
    run_id: &'a str,
    prompt: &'a str,
    model: &'a str,
    width: u32,
    height: u32,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_usd: Option<f32>,
}

pub async fn save_output(out_dir: &Path, id: u64, run_id: &str, res: &ImageResult, ext: &str, cost_usd: Option<f32>) -> anyhow::Result<()> {
    fs::create_dir_all(out_dir).await?;

    let img_tmp = out_dir.join(format!("{:06}.{}.tmp", id, ext));
    let img = out_dir.join(format!("{:06}.{}", id, ext));
    let json_tmp = out_dir.join(format!("{:06}.json.tmp", id));
    let json = out_dir.join(format!("{:06}.json", id));

    // Write PNG atomically
    {
        let mut f = fs::File::create(&img_tmp).await?;
        f.write_all(&res.bytes).await?;
        let _ = f.sync_all().await; // best-effort
    }
    fs::rename(&img_tmp, &img).await?;

    
    let sidecar = Sidecar {
        id,
        run_id,
        prompt: &res.prompt_used,
        model: &res.model,
        width: res.width,
        height: res.height,
        created_at: Utc::now().to_rfc3339(),
        cost_usd,
    };
    let sidecar_bytes = serde_json::to_vec_pretty(&sidecar)?;
    {
        let mut f = fs::File::create(&json_tmp).await?;
        f.write_all(&sidecar_bytes).await?;
        let _ = f.sync_all().await;
    }
    fs::rename(&json_tmp, &json).await?;
    Ok(())
}