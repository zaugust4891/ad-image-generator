use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateYaml {
	pub brand: String,
	pub product: String,
	pub audience: Vec<String>,
	pub style: Vec<String>,
	pub background: Vec<String>,
	pub cta: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Mock,
    Openai,
    // can add other providers here if needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
	pub kind: ProviderKind,
	#[serde(default)]
	pub openai_model: Option<String>,
	#[serde(default)]
	pub openai_size: Option<String>,
	#[serde(default)]
	pub price_usd_per_image: Option<f32>, /// Deterministic per-image cost for budgeting (e.g., 0.04)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariantModeYaml { Cartesian, Random }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupeConfig {
    #[serde(default = "default_true")] pub enabled: bool,
    #[serde(default = "default_phash_bits")] pub phash_bits: u32,
    #[serde(default = "default_phash_thresh")] pub phash_thresh: u32,
}
fn default_true() -> bool { true }
fn default_phash_bits() -> u32 { 64 }
fn default_phash_thresh() -> u32 { 6 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostConfig {
    pub fmt: OutFmtYaml,           // png | jpeg | webp
    #[serde(default)] pub jpeg_quality: Option<u8>,
    #[serde(default)] pub width: Option<u32>,
    #[serde(default)] pub height: Option<u32>,
    #[serde(default)] pub watermark_text: Option<String>,
    #[serde(default)] pub watermark_font: Option<PathBuf>,
    #[serde(default)] pub watermark_px: Option<f32>,
    #[serde(default)] pub watermark_margin: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub target_images: u64,
    pub concurrency: usize,
    pub queue_cap: usize,
    pub rate_per_min: u32,
    /// backoff base milliseconds, e.g., 300 (we’ll apply exponential attempts + jitter)
    #[serde(default = "default_backoff_ms")] pub backoff_base_ms: u64,
    #[serde(default = "default_backoff_factor")] pub backoff_factor: f64, // e.g., 2.0
    #[serde(default = "default_backoff_jitter_ms")] pub backoff_jitter_ms: u64, // add 0..=jitter
}
fn default_backoff_ms() -> u64 { 300 }
fn default_backoff_factor() -> f64 { 2.0 }
fn default_backoff_jitter_ms() -> u64 { 250 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
	pub provider: ProviderConfig,
	pub variant_mode: VariantModeYaml, // "cartesian" or "random"
	pub seed: Option<u64>, // used when mode=random
	pub orchestrator: OrchestratorConfig,
	pub post: PostConfig,
	pub dedupe: DedupeConfig,
	// Optional fixed output dir; if omitted we create run-YYYYMMDD_HHMMSS under ./out
    #[serde(default)] pub out_dir: Option<PathBuf>,
    /// Resume from existing manifest.jsonl in out_dir if present
    #[serde(default)] pub resume: bool,


}

pub fn choose_ext(fmt: &OutFmtYaml) -> &'static str {
	match fmt {
		OutFmtYaml::Png => "png",
		OutFmtYaml::Jpeg => "jpg",
		OutFmtYaml::Webp => "webp",
	}
}