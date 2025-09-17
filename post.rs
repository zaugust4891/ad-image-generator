use anyhow::Result;
use image::{DynamicImage, ImageOutputFormat, imageops::FilterType};
use std::path::PathBuf;
use std::io::Cursor;

/// Output format configuration.
#[derive(Clone)]
pub enum OutFmt {
    Png,
    Jpeg(u8),   // quality 1..=100
    Webp,       // lossless default
}

#[derive(Clone)]
pub struct WatermarkCfg {
    pub text: String,
    pub font_path: PathBuf,
    pub px: f32,        // font size in px
    pub margin: u32,    // margin from edges
}

#[derive(Clone, Default)]
pub struct ResizeCfg {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone, Default)]
pub struct PostOptions {
    pub resize: ResizeCfg,
    pub watermark: Option<WatermarkCfg>,
    pub fmt: OutFmt,
}

pub struct PostProcessor {
    opts: PostOptions,
    // Lazy font load; None if not provided / fails to load.
    font: Option<rusttype::Font<'static>>,
}

impl PostProcessor {
    pub fn new(mut opts: PostOptions) -> Self {
        // Fallback to PNG if not set via env
        if let OutFmt::Jpeg(q) = opts.fmt {
            let clamped = q.clamp(1, 100);
            opts.fmt = OutFmt::Jpeg(clamped);
        }
        let font = match &opts.watermark {
            Some(w) => std::fs::read(&w.font_path)
                .ok()
                .and_then(|bytes| rusttype::Font::try_from_vec(bytes))
                .map(|f| unsafe { std::mem::transmute::<rusttype::Font<'_>, rusttype::Font<'static>>(f) }),
            None => None,
        };
        Self { opts, font }
    }

    /// Process bytes -> (bytes_out, new_w, new_h)
    pub fn process(&self, bytes: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
        let mut img = image::load_from_memory(bytes)?;
        // Resize
        if self.opts.resize.width.is_some() || self.opts.resize.height.is_some() {
            let (nw, nh) = target_size(img.width(), img.height(), &self.opts.resize);
            if nw > 0 && nh > 0 && (nw != img.width() || nh != img.height()) {
                img = img.resize_exact(nw, nh, FilterType::CatmullRom);
            }
        }
        // Watermark (optional)
        if let (Some(wm), Some(font)) = (&self.opts.watermark, &self.font) {
            watermark_text(&mut img, font, wm);
        }
        // Encode
        let (w,h) = (img.width(), img.height());
        let mut out = Vec::new();
        match &self.opts.fmt {
            OutFmt::Png => {
                img.write_to(&mut Cursor::new(&mut out), ImageOutputFormat::Png)?;
            }
            OutFmt::Jpeg(q) => {
                let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, *q);
                enc.encode_image(&img)?;
            }
            OutFmt::Webp => {
                let mut enc = image::codecs::webp::WebPEncoder::new(&mut out);
                enc.encode_image(&img)?;
            }
        }
        Ok((out, w, h))
    }
}

fn target_size(w: u32, h: u32, r: &ResizeCfg) -> (u32, u32) {
    match (r.width, r.height) {
        (Some(nw), Some(nh)) => (nw, nh),
        (Some(nw), None) => {
            let nh = ((h as f64) * (nw as f64 / w as f64)).round() as u32;
            (nw, nh)
        }
        (None, Some(nh)) => {
            let nw = ((w as f64) * (nh as f64 / h as f64)).round() as u32;
            (nw, nh)
        }
        _ => (w, h),
    }
}

fn watermark_text(img: &mut DynamicImage, font: &rusttype::Font, wm: &WatermarkCfg) {
    use image::{Rgba, RgbaImage};
    use imageproc::drawing::draw_text_mut;
    // Convert to RGBA8 for drawing
    let mut rgba: RgbaImage = img.to_rgba8();

    let color = Rgba([255u8, 255u8, 255u8, 200u8]); // white-ish, semi-opaque
    let scale = rusttype::Scale::uniform(wm.px);

    // Measure approximate text size by laying it at (0,0) and finding the v_metrics / glyphs widths.
    // For simplicity, we’ll just position near bottom-right with a margin.
    let x = (rgba.width()  as i32 - wm.margin as i32 - (wm.px as i32 * wm.text.len() as i32 / 2).max(50)).max(0);
    let y = (rgba.height() as i32 - wm.margin as i32 - wm.px as i32).max(0);

    draw_text_mut(&mut rgba, color, x as i32, y as i32, scale, font, &wm.text);
    *img = DynamicImage::ImageRgba8(rgba);
}
