import { z } from "zod";

export const RunConfigSchema = z.object({
  provider: z.object({
    kind: z.enum(["mock", "openai"]),
    model: z.string().optional(),
    api_key_env: z.string().optional(),
    width: z.number().int().min(64, "Width must be at least 64").max(4096, "Width must be at most 4096").optional(),
    height: z.number().int().min(64, "Height must be at least 64").max(4096, "Height must be at most 4096").optional(),
    price_usd_per_image: z.number().nonnegative().max(100, "Price seems too high").optional(),
  }),
  orchestrator: z.object({
    target_images: z.number().int().min(1, "Must generate at least 1 image").max(10000, "Maximum 10000 images per run"),
    concurrency: z.number().int().min(1, "Concurrency must be at least 1").max(100, "Concurrency must be at most 100"),
    queue_cap: z.number().int().min(1, "Queue capacity must be at least 1").max(1000, "Queue capacity must be at most 1000"),
    rate_per_min: z.number().int().min(1, "Rate must be at least 1/min").max(600, "Rate must be at most 600/min"),
    backoff_base_ms: z.number().int().min(100, "Backoff base must be at least 100ms").max(60000, "Backoff base must be at most 60000ms"),
    backoff_factor: z.number().min(1.1, "Backoff factor must be at least 1.1").max(5.0, "Backoff factor must be at most 5.0"),
    backoff_jitter_ms: z.number().int().nonnegative().max(10000, "Jitter must be at most 10000ms"),
  }),
  dedupe: z.object({
    enabled: z.boolean(),
    phash_bits: z.number().int().min(4, "pHash bits must be at least 4").max(64, "pHash bits must be at most 64"),
    phash_thresh: z.number().int().nonnegative().max(32, "pHash threshold must be at most 32"),
  }),
  post: z.object({
    thumbnail: z.boolean(),
    thumb_max: z.number().int().min(16, "Thumbnail size must be at least 16").max(1024, "Thumbnail size must be at most 1024"),
  }),
  rewrite: z.object({
    enabled: z.boolean(),
    model: z.string().optional(),
    system: z.string().optional(),
    max_tokens: z.number().int().min(1, "Max tokens must be at least 1").max(4096, "Max tokens must be at most 4096").optional(),
    cache_file: z.string().optional(),
  }),
  out_dir: z.string().min(1, "Output directory is required"),
  seed: z.number().int().nonnegative(),
  budget_limit_usd: z.number().nonnegative().optional(),
}).refine(
  (data) => data.provider.kind !== "openai" || (data.provider.api_key_env && data.provider.api_key_env.length > 0),
  {
    message: "api_key_env is required when using OpenAI provider",
    path: ["provider", "api_key_env"],
  }
);

export type RunConfig = z.infer<typeof RunConfigSchema>;

export const TemplateSchema = z.object({
  brand: z.string().min(1),
  product: z.string().min(1),
  styles: z.array(z.string().min(1)).min(1),
});
export type Template = z.infer<typeof TemplateSchema>;
