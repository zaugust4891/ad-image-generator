use anyhow::Result;
use image::{imageops::FilterType, ImageFormat};
use std::io::Cursor;

#[allow(dead_code)]

pub struct PostProcessor{ pub make_thumb: bool, pub thumb_max: u32 }
impl PostProcessor{
    pub fn new(make_thumb: bool, thumb_max: u32) -> Self { Self{make_thumb, thumb_max} }
    #[allow(dead_code)]
    pub fn maybe_thumbnail(&self, bytes:&[u8]) -> Result<Option<Vec<u8>>> {
        if !self.make_thumb { return Ok(None); }
        let img = image::load_from_memory(bytes)?;
        let thumb = img.resize(self.thumb_max, self.thumb_max, FilterType::Lanczos3);
        let mut buf = Vec::new();
        thumb.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
        Ok(Some(buf))
    }
}
