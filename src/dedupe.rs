use anyhow::Result;

use img_hash::{HasherConfig, HashAlg, ImageHash};
use std::collections::HashSet;

pub struct PerceptualDeduper{
    hasher: HasherConfig,
    seen: HashSet<ImageHash>,
    threshold: u32,
}
impl PerceptualDeduper{
    pub fn new(bits:u32, threshold:u32)->Self{
        Self{ hasher: HasherConfig::new().hash_alg(HashAlg::DoubleGradient).hash_size(bits/8, bits/8), seen: HashSet::new(), threshold }
    }
    pub fn is_duplicate(&mut self, bytes:&[u8])->Result<bool>{
        let img = img_hash::image::load_from_memory(bytes)?;
        let hash = self.hasher.to_hasher().hash_image(&img);
        for h in &self.seen {
            if hash.dist(h) <= self.threshold { return Ok(true); }
        }
        self.seen.insert(hash);
        Ok(false)
    }
}
