use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Clone)]
pub enum PromptStyle {
    AdTemplate(PromptTemplate),
    GeneralPrompt(PromptGeneral),
}

#[derive(Clone)]
pub struct PromptGeneral { pub prompt: String }

#[derive(Clone)]
pub struct PromptTemplate {
    pub brand: String,
    pub product: String,
    pub styles: Vec<String>,
}

#[derive(Clone)]
pub struct VariantGenerator { rng: StdRng, prompt_style: PromptStyle }
impl VariantGenerator {
    pub fn new(prompt_style: PromptStyle, seed: u64) -> Self {
        Self { rng: StdRng::seed_from_u64(seed), prompt_style }
    }
    pub fn next(&mut self) -> String {
        match self.prompt_style {
            PromptStyle::AdTemplate(ref tpl) => {
                let s = if tpl.styles.is_empty() {
                    "clean product photo".to_string()
                } else {
                    tpl.styles[self.rng.random_range(0..tpl.styles.len())].clone()
                };
                format!("An advertisement image for {} {} in style: {}", tpl.brand, tpl.product, s)
            }
            PromptStyle::GeneralPrompt(ref prompt) => {
                prompt.prompt.clone()
            }
        }
    }
}
