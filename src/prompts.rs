use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Clone)]
pub struct PromptTemplate{
    pub brand: String,
    pub product: String,
    pub styles: Vec<String>,
}

#[derive(Clone)]
pub struct VariantGenerator{ rng: StdRng, tpl: PromptTemplate }
impl VariantGenerator{
    pub fn new(tpl: PromptTemplate, seed: u64) -> Self { Self{ rng: StdRng::seed_from_u64(seed), tpl }}
    pub fn next(&mut self) -> String{
        let s = if self.tpl.styles.is_empty(){ "clean product photo".to_string() } else {
            self.tpl.styles[self.rng.random_range(0..self.tpl.styles.len())].clone()
        };
        format!("An advertisement image for {} {} in style: {}", self.tpl.brand, self.tpl.product, s)
    }
}
