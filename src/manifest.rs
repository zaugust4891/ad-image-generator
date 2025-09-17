use chrono::Utc;
use serde::Serialize;
use std::path::PathBuf;
use tokio::{fs::{self, OpenOptions}, io::{AsyncBufReadExt, AsyncWriteExt}, sync::Mutex};
use std::sync::Arc;

#[derive(Debug, Serialize, Clone)]
pub struct ManifestRecord<'a> {
    pub id: u64,
    pub run_id: &'a str,
    pub prompt: &'a str,
    pub model: &'a str,
    pub width: u32,
    pub height: u32,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f32>,
}

pub struct ManifestWriter {
    file: Arc<Mutex<tokio::fs::File>>,
    path: PathBuf,
}


impl ManifestWriter {
    pub async fn open(path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent).await?; }
        let file = OpenOptions::new().create(true).append(true).open(&path).await?;
        Ok(Self { file: Arc::new(Mutex::new(file)), path })
    }

    pub async fn append(&self, rec: &ManifestRecord<'_>) -> anyhow::Result<()> {
        let mut f = self.file.lock().await;
        let line = serde_json::to_vec(rec)?;
        f.write_all(&line).await?;
        f.write_all(b"\n").await?;
        f.flush().await?;
        Ok(())
    }

    pub fn path(&self) -> &PathBuf { &self.path }
}

/// Count lines in JSONL manifest -> (completed, next_id).
pub async fn resume_state(path: &PathBuf) -> anyhow::Result<(u64, u64)> {
    let file = match tokio::fs::File::open(path).await {
        Ok(f) => f, Err(_) => return Ok((0, 1)),
    };
    let mut lines = tokio::io::BufReader::new(file).lines();
    let mut count: u64 = 0;
    while let Some(_line) = lines.next_line().await? {
        count += 1;
    }
    Ok((count, count + 1))
}