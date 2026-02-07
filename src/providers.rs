use anyhow::{Context, Result};
use base64::Engine as _;
use image::{ImageBuffer, Rgba};
use rand::Rng;
use std::{future::Future, pin::Pin};


#[derive(Debug, Clone)]
pub struct ImageResult {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    #[allow(unused)]

    pub prompt_used: String,
    pub model: String,
}

pub trait ImageProvider: Send + Sync {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ImageResult>> + Send + 'a>>;
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    #[allow(dead_code)]

    fn price_usd_per_image(&self) -> f64 { 0.0 }
}

#[derive(Clone)]
pub struct MockProvider { pub model: String, pub w: u32, pub h: u32 }
impl ImageProvider for MockProvider {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ImageResult>> + Send + 'a>> {
        Box::pin(async move {
            // Create a simple noise image
            let mut rng = rand::rng();
            let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(self.w, self.h);
            for p in img.pixels_mut() {
                *p = Rgba([rng.random::<u8>(), rng.random::<u8>(), rng.random::<u8>(), 255]);
            }
            let mut buf = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut buf);
            img.write_to(&mut cursor, image::ImageFormat::Png)?;
            Ok(ImageResult { bytes: buf, width: self.w, height: self.h, prompt_used: prompt.to_string(), model: self.model.clone() })
        })
    }
    fn name(&self) -> &str { "mock" }
    fn model(&self) -> &str { &self.model }
}

#[derive(Clone)]
pub struct OpenAIProvider { pub client: reqwest::Client, pub model: String, pub api_key: String, pub w:u32, pub h:u32, pub price: f64 }
impl ImageProvider for OpenAIProvider {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ImageResult>> + Send + 'a>> {
        Box::pin(async move {
            #[derive(serde::Serialize)] struct Req<'a>{prompt:&'a str, size:String, model:String, #[serde(skip_serializing_if="Option::is_none")] response_format:Option<&'a str>}
            #[derive(serde::Deserialize)] struct Resp{data:Vec<Item>}
            #[derive(serde::Deserialize)] struct Item{b64_json:Option<String>, url:Option<String>}
            // `response_format` is only supported for DALL-E models.
            // GPT image models always return base64 and reject this parameter.
            let response_format = if self.model.starts_with("dall-e-") {
                Some("b64_json")
            } else {
                None
            };
            let req = Req{
                prompt,
                size: format!("{}x{}", self.w, self.h),
                model:self.model.clone(),
                response_format,
            };
            let resp = self.client.post("https://api.openai.com/v1/images/generations")
                .bearer_auth(&self.api_key)
                .json(&req)
                .send().await?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("OpenAI API error {status}: {body}");
            }
            let parsed = resp.json::<Resp>().await?;
            let first = parsed.data.get(0).context("OpenAI API returned no image data")?;
            let bytes = if let Some(b64) = &first.b64_json {
                base64::engine::general_purpose::STANDARD.decode(b64)?
            } else if let Some(url) = &first.url {
                self.client
                    .get(url)
                    .send()
                    .await?
                    .error_for_status()?
                    .bytes()
                    .await?
                    .to_vec()
            } else {
                anyhow::bail!("OpenAI API returned image item without b64_json or url");
            };
            Ok(ImageResult{bytes, width:self.w, height:self.h, prompt_used:prompt.to_string(), model:self.model.clone()})
        })
    }
    fn name(&self) -> &str { "openai" }
    fn model(&self) -> &str { &self.model }
    fn price_usd_per_image(&self) -> f64 { self.price }
}
//Double check this endpoint and request format
#[derive(Clone)]
pub struct GeminiProvider { pub client: reqwest::Client, pub model: String, pub api_key: String, pub w:u32, pub h:u32, pub price: f64 }
impl ImageProvider for GeminiProvider {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ImageResult>> + Send + 'a>> {
        Box::pin(async move {
            #[derive(serde::Serialize)] struct Req<'a>{prompt:&'a str, size:String, model:String, #[serde(skip_serializing_if="Option::is_none")] response_format:Option<&'a str>}
            #[derive(serde::Deserialize)] struct Resp{data:Vec<Item>}
            #[derive(serde::Deserialize)] struct Item{b64_json:String}
            let needs_response_format = self.model.starts_with("gemini-3-pro-image-preview");
            let req = Req{prompt, size: format!("{}x{}", self.w, self.h), model:self.model.clone(), response_format: if needs_response_format { Some("b64_json") } else { None }};
            let resp = self.client.post("https://gemini.googleapis.com/v1/images/generations")
                .bearer_auth(&self.api_key)
                .json(&req)
                .send().await?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Gemini API error {status}: {body}");
            }
            let parsed = resp.json::<Resp>().await?;
            let bytes = base64::engine::general_purpose::STANDARD.decode(&parsed.data[0].b64_json)?;
            Ok(ImageResult{bytes, width:self.w, height:self.h, prompt_used:prompt.to_string(), model:self.model.clone()})
        })
    }
    fn name(&self) -> &str { "gemini" }
    fn model(&self) -> &str { &self.model }
    fn price_usd_per_image(&self) -> f64 { self.price }
}
