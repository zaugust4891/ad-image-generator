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
}

pub async fn save_output(out_dir: &Path, id: u64, run_id: &str, res: &ImageResult) -> anyhow::Result<()> {
    fs::create_dir_all(out_dir).await?;

    let png_tmp = out_dir.join(format!("{:06}.png.tmp", id));
    let png = out_dir.join(format!("{:06}.png", id));
    let json_tmp = out_dir.join(format!("{:06}.json.tmp", id));
    let json = out_dir.join(format!("{:06}.json", id));

    // Write PNG atomically
    {
        let mut f = fs::File::create(&png_tmp).await?;
        f.write_all(&res.bytes).await?;
        let _ = f.sync_all().await; // best-effort
    }
    fs::rename(&png_tmp, &png).await?;

    
    let sidecar = Sidecar {
        id,
        run_id,
        prompt: &res.prompt_used,
        model: &res.model,
        width: res.width,
        height: res.height,
        created_at: Utc::now().to_rfc3339(),
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