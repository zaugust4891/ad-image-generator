use async_trait::async_trait;
use std:result::Result;

#[derive(Debug)]
pub enum ProviderError {
	RateLimited,
    Http(String),
    Fatal(String),
}

#[derive(Debug)]
struct ImageResult {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    prompt_used: String,
    model: String,
}

use image::{ImageBuffer, Rgba};
trait ImageProvider ImageProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<ImageResult, ProviderError>;
    fn name(&self) -> &'static str;
}

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
















