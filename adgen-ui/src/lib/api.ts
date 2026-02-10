import { API_BASE_URL as BASE } from "./config";

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

type AdTemplateYaml = { brand: string; product: string; styles: string[] };
type GeneralPromptYaml = { prompt: string };
type TemplateYaml = { mode: { AdTemplate: AdTemplateYaml } | { GeneralPrompt: GeneralPromptYaml } };

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null;
}

function toTemplateForm(v: unknown): Template {
  if (!isRecord(v)) {
    throw new Error("Template payload is not an object");
  }

  // Backward-compatible support: plain object template shape.
  if (
    typeof v.brand === "string" &&
    typeof v.product === "string" &&
    Array.isArray(v.styles)
  ) {
    return {
      brand: v.brand,
      product: v.product,
      styles: v.styles.map(String),
    };
  }

  // Current backend shape: { mode: { AdTemplate: { ... } } }
  if (isRecord(v.mode) && isRecord(v.mode.AdTemplate)) {
    const ad = v.mode.AdTemplate;
    if (
      typeof ad.brand === "string" &&
      typeof ad.product === "string" &&
      Array.isArray(ad.styles)
    ) {
      return {
        brand: ad.brand,
        product: ad.product,
        styles: ad.styles.map(String),
      };
    }
    throw new Error("Invalid AdTemplate shape from backend");
  }

  if (isRecord(v.mode) && isRecord(v.mode.GeneralPrompt)) {
    throw new Error(
      "Template is in GeneralPrompt mode. Current Template editor supports AdTemplate fields only."
    );
  }

  throw new Error("Unrecognized template format from backend");
}

function toTemplateYaml(template: Template): TemplateYaml {
  return {
    mode: {
      AdTemplate: {
        brand: template.brand,
        product: template.product,
        styles: template.styles,
      },
    },
  };
}

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
  return toTemplateForm(await r.json());
}
export async function saveTemplate(t: Template): Promise<void> {
  const r = await fetch(`${BASE}/api/template`, {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(toTemplateYaml(t)),
  });
  if (!r.ok) throw new Error("Failed to save template");
}

export type ApiError = {
  error: string;
  code?: string;
  suggestion?: string;
};

export async function startRun(): Promise<{ run_id: string }> {
  const r = await fetch(`${BASE}/api/run`, { method: "POST" });
  if (!r.ok) {
    const err: ApiError = await r.json().catch(() => ({ error: "Failed to start run" }));
    const message = err.suggestion
      ? `${err.error}\n\nSuggestion: ${err.suggestion}`
      : err.error || "Failed to start run";
    throw new Error(message);
  }
  return r.json();
}

export async function getCurrentRun(): Promise<{ run_id: string | null }> {
  const r = await fetch(`${BASE}/api/run/current`);
  if (!r.ok) throw new Error("Failed to get current run");
  return r.json();
}

export async function listImages(): Promise<{ name: string; url: string; created_ms: number }[]> {
  const r = await fetch(`${BASE}/api/images`);
  if (!r.ok) throw new Error("Failed to list images");
  return r.json();
}

export type ValidationError = {
  field: string;
  message: string;
  suggestion?: string;
};

export type ValidationResult = {
  valid: boolean;
  errors: ValidationError[];
  warnings: string[];
};

// --- Auth ---

export type UserResponse = {
  id: number;
  email: string;
  name?: string;
  created_at: string;
  updated_at: string;
};

export async function register(
  email: string,
  password: string,
  name?: string,
): Promise<UserResponse> {
  const r = await fetch(`${BASE}/api/register`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ email, password, name: name || undefined }),
  });
  if (!r.ok) {
    const err: ApiError = await r.json().catch(() => ({ error: "Registration failed" }));
    throw new Error(err.error || "Registration failed");
  }
  return r.json();
}

export async function login(
  email: string,
  password: string,
): Promise<UserResponse> {
  const r = await fetch(`${BASE}/api/login`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ email, password }),
  });
  if (!r.ok) {
    const err: ApiError = await r.json().catch(() => ({ error: "Login failed" }));
    throw new Error(err.error || "Login failed");
  }
  return r.json();
}

export async function validateConfig(
  config: RunConfig,
  template: Template
): Promise<ValidationResult> {
  const r = await fetch(`${BASE}/api/config/validate`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ config, template: toTemplateYaml(template) }),
  });
  if (!r.ok) throw new Error("Validation request failed");
  return r.json();
}
