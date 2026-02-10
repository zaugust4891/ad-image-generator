use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCfg{
    pub kind: String, // "mock" | "openai"
    pub model: Option<String>,
    pub api_key_env: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub price_usd_per_image: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorCfg{
    pub target_images: u64,
    pub concurrency: usize,
    pub queue_cap: usize,
    pub rate_per_min: u32,
    pub backoff_base_ms: u64,
    pub backoff_factor: f64,
    pub backoff_jitter_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupeCfg{ pub enabled: bool, pub phash_bits: u32, pub phash_thresh: u32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCfg{ pub thumbnail: bool, pub thumb_max: u32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteCfg{ pub enabled: bool, pub model: Option<String>, pub system: Option<String>, pub max_tokens: Option<u32>, pub cache_file: Option<PathBuf> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCfg{
    pub provider: ProviderCfg,
    pub orchestrator: OrchestratorCfg,
    pub dedupe: DedupeCfg,
    pub post: PostCfg,
    pub rewrite: RewriteCfg,
    pub out_dir: PathBuf,
    pub seed: u64,
    #[serde(default)]
    pub budget_limit_usd: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mode {
    AdTemplate(AdTemplate),
    GeneralPrompt(GeneralPrompt),
} 

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateYaml {
    pub mode: Mode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdTemplate{ 
    pub brand:String,
    pub product:String, 
    pub styles:Vec<String> 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralPrompt{ 
    pub prompt:String 
}
