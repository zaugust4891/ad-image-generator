use anyhow::Result;
use img_hash::{HasherConfig, HashAlg, ImageHash};
use std::sync::Mutex;

/// Checks near-duplicates using perceptual hash (pHash).
pub struct PerceptualDeduper {
    hasher: img_hash::Hasher,
    /// Hamming distance threshold to consider images "the same".
    /// 0 -> exact hash match, 5..10 typical for lenient dedupe.
    threshold: u32,
    seen: Mutex<Vec<ImageHash>>, // simple Vec; fine for first pass
}

impl PerceptualDeduper {
    /// `hash_bits` -> 8 (8x8) or 16 (16x16) etc. Larger = slower but more discriminative.
    pub fn new(hash_bits: u32, threshold: u32) -> Self {
        let size = (hash_bits as f32).sqrt().round() as u32;
        let hasher = HasherConfig::new()
            .hash_alg(HashAlg::Gradient) // robust default
            .hash_size(size, size)
            .to_hasher();
        Self { hasher, threshold, seen: Mutex::new(Vec::new()) }
    }

    /// Compute hash from image bytes (decodes once) and decide if duplicate.
    /// Returns (is_duplicate, hash_base64).
    pub fn check_and_insert(&self, bytes: &[u8]) -> Result<(bool, String)> {
        let img = image::load_from_memory(bytes)?;
        let hash = self.hasher.hash_image(&img);
        let hash_str = hash.to_base64();

        let mut guard = self.seen.lock().unwrap();
        for h in guard.iter() {
            let dist = hash.dist(h);
            if dist <= self.threshold {
                return Ok((true, hash_str));
            }
        }
        guard.push(hash);
        Ok((false, hash_str))
    }
}
