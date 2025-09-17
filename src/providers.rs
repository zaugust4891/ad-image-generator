use async_trait::async_trait;
use std:result::Result;

#[derive(Debug)]
pub enum ProviderError {
	RateLimited,
    Http(String),
    Fatal(String),
}

#[derive(Debug, Clone)]
struct ImageResult {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    prompt_used: String,
    model: String,
}

#[async_trait]
use image::{ImageBuffer, Rgba};
pub trait ImageProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<ImageResult, ProviderError>;
    fn name(&self) -> &'static str;
}


///------Mock Provider-----///

use image::{ImageBuffer, Rgba};

pub struct MockProvider;

#[async_trait]
impl ImageProvider for MockProvider {
	fn name(&self) -> &'static str {"mock"}

	async fn generate(&self, prompt: &str) -> Result<ImageResult, ProviderError> {
		// CPU-bound image synthesis. Small and fast enough to do inline for now.
		let w = 256u32;
        let h = 256u32;
        let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(w, h);
        for (x, y, p) in img.enumerate_pixels_mut() {
	        let v = ((x ^ y) & 0xFF) as u8;
	        *p = Rgba([v, 255 - v, (prompt.len() % 255) as u8, 255]);
        }
        let mut png_bytes: Vec<u8> = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .map_err(|e| ProviderError::Fatal(format!("encode error: {e}")))?;

        Ok(ImageResult {
            bytes: png_bytes,
            width: w,
            height: h,
            prompt_used: prompt.to_string(),
            model: "mock".to_string(),
        })
    }
}



///-----OpenAI provider (images/generations)-----///

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;


pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    size: String, // e.g. 1024x1024
}


impl OpenAIProvider {
    pub fn new(api_key: String, model: String, size: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            size,
        }
    }
}

#[derive(serde::Serialize)]
struct OpenAIRequest<'a> {
    prompt: &'a str,
    n: u32,
    size: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<&'a str>,
}

#[derive(serde::Deserialize)]
struct OpenAIResponseData {
    b64_json: String,
}

#[derive(serde::Deserialize)]
struct OpenAIResponse {
    data: Vec<OpenAIResponseData>,
}


#[async_trait]

impl ImageProvider for OpenAIProvider {
    fn name(&self) -> &'static str { "openai "}

    async fn generate(&self, prompt: &str) -> Result<ImageResult, ProviderError> {
        let url = "https://api.openai.com/v1/images/generations";
        let body = OpenAIRequest {
            prompt,
            n: 1,
            size, & self.size,
            model: Some(self.model)
        };

        let resp = self.client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Http(format!("request error: {e}")))?;

        let status = resp.status();
        if status.as_u16 == 429 { return Err(ProviderError::RateLimited); }
        if status.is_server_error { return Err(ProviderError::Http(format!("status {status}"))); }
        if !status.is_success() { return Err(ProviderError::Http(format!("unexpected status {status} "))); }

        let parsed: OpenAIResponse = resp.json().await
            .map_err(|e| ProviderError::Http(format!("json decode: {e}")))?;
        let first = parsed.data.into_iter().next()
            .ok_or_else(|| ProviderError::Http("empty data".into()))?;
        let bytes = B64.decode(first.b64_json.as_bytes())
            .map_err(|e| ProviderError::Http(format!("base64: {e}")))?;

        // Try to read dimensions - if fail, keep (0,0)
        let (w, h) = image::load_from_memory(&bytes)
            .map(|img| img.dimensions())
            .unwrap_or((0,0));

        Ok(ImageResult {
            bytes,
            width: w,
            height: h,
            prompt_used: prompt.to_string(),
            model: self.model.clone(),
        })
    }
} 








