use async_trait::async_trait;
use anyhow::Result;
use image::{ImageBuffer, Rgba};
use rand::Rng;
use base64::Engine as _;


#[derive(Debug, Clone)]
pub struct ImageResult {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    #[allow(unused)]

    pub prompt_used: String,
    pub model: String,
}

#[async_trait]
pub trait ImageProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<ImageResult>;
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    #[allow(dead_code)]

    fn price_usd_per_image(&self) -> f64 { 0.0 }
}

#[derive(Clone)]
pub struct MockProvider { pub model: String, pub w: u32, pub h: u32 }
#[async_trait]
impl ImageProvider for MockProvider {
    async fn generate(&self, prompt: &str) -> Result<ImageResult> {
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
    }
    fn name(&self) -> &str { "mock" }
    fn model(&self) -> &str { &self.model }
}

#[derive(Clone)]
pub struct OpenAIProvider { pub client: reqwest::Client, pub model: String, pub api_key: String, pub w:u32, pub h:u32, pub price: f64 }
#[async_trait]
impl ImageProvider for OpenAIProvider {
    async fn generate(&self, prompt: &str) -> Result<ImageResult> {
        #[derive(serde::Serialize)] struct Req<'a>{prompt:&'a str, size:String, model:String}
        #[derive(serde::Deserialize)] struct Resp{data:Vec<Item>}
        #[derive(serde::Deserialize)] struct Item{b64_json:String}
        let req = Req{prompt, size: format!("{}x{}", self.w, self.h), model:self.model.clone()};
        let resp = self.client.post("https://api.openai.com/v1/images/generations")
            .bearer_auth(&self.api_key)
            .json(&req)
            .send().await?
            .error_for_status()?
            .json::<Resp>().await?;
        let bytes = base64::engine::general_purpose::STANDARD.decode(&resp.data[0].b64_json)?;
        Ok(ImageResult{bytes, width:self.w, height:self.h, prompt_used:prompt.to_string(), model:self.model.clone()})
    }
    fn name(&self) -> &str { "openai" }
    fn model(&self) -> &str { &self.model }
    fn price_usd_per_image(&self) -> f64 { self.price }
}
