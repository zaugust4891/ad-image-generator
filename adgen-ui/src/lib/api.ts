export type RunConfig = {
  provider: { kind: "mock" | "openai"; model: string; width: number; height: number; price_usd_per_image: number };
  orchestrator: { target_images: number; concurrency: number; queue_cap: number; rate_per_min: number; backoff_base_ms: number; backoff_factor: number; backoff_jitter_ms: number };
  dedupe: { enabled: boolean; phash_bits: number; phash_thresh: number };
  post: { thumbnail: boolean; thumb_max: number };
  rewrite: { enabled: boolean; model: string; system: string; max_tokens: number };
  out_dir: string;
  seed: number;
};

export type Template = { brand: string; product: string; styles: string[] };

import { API_BASE_URL as BASE } from "./config";

export async function getConfig(): Promise<RunConfig> {
  const r = await fetch(`${BASE}/api/config`);
  if (!r.ok) throw new Error("Failed to load config");
  return r.json();
}
export async function saveConfig(cfg: RunConfig): Promise<void> {
  const r = await fetch(`${BASE}/api/config`, { method: "PUT", headers: { "content-type": "application/json" }, body: JSON.stringify(cfg) });
  if (!r.ok) throw new Error("Failed to save config");
}

export async function getTemplate(): Promise<Template> {
  const r = await fetch(`${BASE}/api/template`);
  if (!r.ok) throw new Error("Failed to load template");
  return r.json();
}
export async function saveTemplate(t: Template): Promise<void> {
  const r = await fetch(`${BASE}/api/template`, { method: "PUT", headers: { "content-type": "application/json" }, body: JSON.stringify(t) });
  if (!r.ok) throw new Error("Failed to save template");
}

export async function startRun(): Promise<{ run_id: string }> {
  const r = await fetch(`${BASE}/api/run`, { method: "POST" });
  if (!r.ok) throw new Error("Failed to start run");
  return r.json();
}

export async function listImages(): Promise<{ name: string; url: string; created_ms: number }[]> {
  const r = await fetch(`${BASE}/api/images`);
  if (!r.ok) throw new Error("Failed to list images");
  return r.json();
}
