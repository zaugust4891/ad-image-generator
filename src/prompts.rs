use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Clone)]
pub struct PromptTemplate {
	pub brand: String,
	pub product: String,
	pub audience: Vec<String>,
	pub style: Vec<String>,
	pub background: Vec<String>,
	pub cta: Vec<String>,
}

impl PromptTemplate {
	pub fn render(
		&self,
		a_idx: usize, 
		s_idx: usize, 
		b_idx: usize, 
		c_idx: usize) -> String {
		format!(
			"Ad photo of {product} by {brand}, targeted at {aud}, {style} on a {bg} background, crisp, high detail, product centered. CTA: \"{cta}\".",
			product = self.product,
			brand = self.brand,
			aud = self.audience[a_idx],
			style = self.style[s_idx],
			bg = self.background[b_idx],
			cta = self.cta[c_idx],
		)

	}
}

/// Two ways to produce variants.
#[derive(Clone, Copy, Debug)]
pub enum VariantMode { Cartesian, Random }

/// A deterministing, thread-safe variant generator.
///Internally, we keep either cartesian indices or a seeded RNG.
pub struct VariantGenerator {
	tpl: PromptTemplate,
	mode: VariantMode,
	//Cartesian state
	a: usize, s: usize, b: usize, c: usize,
	// Random state
	rng: StdRng,
} 

impl VariantGenerator {
	pub fn new_cartesian(tpl: PromptTemplate) -> Self {
		Self { tpl, mode: VariantMode::Cartesian, a:0, s:0, b:0, c:0, rng: StdRng::seed_from_u64(0) }
	}

	pub fn new_random(tpl: PromptTemplate, seed: u64) -> Self {
		Self { tpl, mode: VariantMode::Random, a:0, s:0, b:0, c:0, rng: StdRng::seed_from_u64(seed) }
	}

	/// Returns the next prompt string, or None if the Cartesian space is exhausted.
	pub fn next(&mut self) -> Option<String> {
		let a_len = self.tpl.audience.len();
		let s_len = self.tpl.style.len();
		let b_len = self.tpl.background.len();
		let c_len = self.tpl.cta.len();
	
		match self.mode {
			VariantMode::Cartesian => {
				if self.a >= a_len { return None; }
				// Render current indices
				let prompt = self.tpl.render(self.a, self.s, self.b, self.c);
				// Advance cartesian counters (least-significant index = c)
				self.c += 1; if self.c >= c_len { self.c = 0; self.b += 1; }
				if self.b >= b_len { self.b = 0; self.s += 1; }
				if self.s >= s_len { self.s = 0; self.a += 1; }
				Some(prompt)
			}
				VariantMode::Random => {
				let a = self.rng.gen_range(0..a_len);
				let s = self.rng.gen_range(0..s_len);
				let b = self.rng.gen_range(0..b_len);
				let c = self.rng.gen_range(0..c_len);
				Some(self.tpl.render(a, s, b, c))
			}
		}
	}
}

















