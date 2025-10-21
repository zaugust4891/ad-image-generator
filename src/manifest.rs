use serde::Serialize;
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt};

#[derive(Serialize)]
pub struct ManifestRecord<'a>{
    pub id: u64,
    pub created_at: String,
    pub provider: &'a str,
    pub model: &'a str,
    pub prompt: &'a str,
    pub path_png: String,
}

pub struct Manifest{ path: std::path::PathBuf }
impl Manifest{
    pub fn new(out_dir:&Path)->Self{ Self{ path: out_dir.join("manifest.jsonl") } }
    pub async fn append(&self, rec: ManifestRecord<'_>) -> anyhow::Result<()> {
        let mut f = fs::OpenOptions::new().create(true).append(true).open(&self.path).await?;
        let line = serde_json::to_string(&rec)?;
        f.write_all(line.as_bytes()).await?;
        f.write_all(b"\n").await?;
        Ok(())
    }
}
