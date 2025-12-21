import { z } from "zod";

export const RunConfigSchema = z.object({
  provider: z.object({
    kind: z.enum(["mock", "openai"]),
    model: z.string().optional(),
    api_key_env: z.string().optional(),
    width: z.number().int().positive().optional(),
    height: z.number().int().positive().optional(),
    price_usd_per_image: z.number().nonnegative().optional(),
  }),
  orchestrator: z.object({
    target_images: z.number().int().nonnegative(),
    concurrency: z.number().int().positive(),
    queue_cap: z.number().int().positive(),
    rate_per_min: z.number().int().positive(),
    backoff_base_ms: z.number().int().nonnegative(),
    backoff_factor: z.number().positive(),
    backoff_jitter_ms: z.number().int().nonnegative(),
  }),
  dedupe: z.object({
    enabled: z.boolean(),
    phash_bits: z.number().int().positive(),
    phash_thresh: z.number().int().nonnegative(),
  }),
  post: z.object({
    thumbnail: z.boolean(),
    thumb_max: z.number().int().positive(),
  }),
  rewrite: z.object({
    enabled: z.boolean(),
    model: z.string().optional(),
    system: z.string().optional(),
    max_tokens: z.number().int().positive().optional(),
    cache_file: z.string().optional(),
  }),
  out_dir: z.string(),
  seed: z.number().int().nonnegative(),
});

export type RunConfig = z.infer<typeof RunConfigSchema>;

export const TemplateSchema = z.object({
  brand: z.string().min(1),
  product: z.string().min(1),
  styles: z.array(z.string().min(1)).min(1),
});
export type Template = z.infer<typeof TemplateSchema>;
