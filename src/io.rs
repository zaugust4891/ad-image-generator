use chrono::Utc;
use serde::Serialize;
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt};

use crate::providers::ImageResult;

#[derive(Serialize)]
struct Sidecar<'a> {
    id: u64,
    run_id: &'a str,
    provider: &'a str,
    model: &'a str,
    width: u32,
    height: u32,
    created_at: String,
    original_prompt: &'a str,
    rewritten_prompt: Option<&'a str>,
    cost_usd: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail_path: Option<String>,
}

pub async fn save_image_with_sidecar(
    out_dir: &Path,
    run_id: &str,
    id: u64,
    provider: &str,
    res: &ImageResult,
    original_prompt: &str,
    rewritten_prompt: Option<&str>,
    cost_usd: f64,
    thumbnail: Option<&[u8]>,
) -> anyhow::Result<()> {
    fs::create_dir_all(out_dir).await?;
    let stem = format!("{:08}-{}-{}", id, provider, res.model);
    let png = out_dir.join(format!("{}.png", stem));
    let json = out_dir.join(format!("{}.json", stem));
    let png_tmp = out_dir.join(format!("{}.png.tmp", stem));
    let json_tmp = out_dir.join(format!("{}.json.tmp", stem));

    {
        let mut f = fs::File::create(&png_tmp).await?;
        f.write_all(&res.bytes).await?;
        let _ = f.sync_all().await;
    }
    fs::rename(&png_tmp, &png).await?;

    // Save thumbnail if provided
    let thumbnail_path = if let Some(thumb_bytes) = thumbnail {
        let thumb_name = format!("{}_thumb.png", stem);
        let thumb_path = out_dir.join(&thumb_name);
        let thumb_tmp = out_dir.join(format!("{}_thumb.png.tmp", stem));
        {
            let mut f = fs::File::create(&thumb_tmp).await?;
            f.write_all(thumb_bytes).await?;
            let _ = f.sync_all().await;
        }
        fs::rename(&thumb_tmp, &thumb_path).await?;
        Some(thumb_name)
    } else {
        None
    };

    let sidecar = Sidecar {
        id, run_id, provider, model: &res.model, width: res.width, height: res.height,
        created_at: Utc::now().to_rfc3339(),
        original_prompt,
        rewritten_prompt,
        cost_usd,
        thumbnail_path,
    };
    let bytes = serde_json::to_vec_pretty(&sidecar)?;
    {
        let mut f = fs::File::create(&json_tmp).await?;
        f.write_all(&bytes).await?;
        let _ = f.sync_all().await;
    }
    fs::rename(&json_tmp, &json).await?;
    Ok(())
}
