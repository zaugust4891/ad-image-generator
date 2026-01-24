# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

### Backend (Rust)
- `cargo build --release` — build the backend
- `cargo check` — type-check without building
- `cargo test` — run tests
- `cargo run -- run --config ./run-config.yaml --template ./template.yml --out_dir ./output` — one-shot image generation
- `cargo run -- serve --bind 127.0.0.1:8787 --config-path ./run-config.yaml --template-path ./template.yml` — start HTTP API server

### Frontend (React/Vite, in `adgen-ui/`)
- `npm run dev` — dev server with HMR (port 5173)
- `npm run build` — production build (TypeScript + Vite)
- `npm run lint` — ESLint
- `npm run preview` — preview production build

## Architecture

This is a monorepo with a Rust backend and React frontend for generating ad images via AI providers.

### Backend (`src/`)

Two entry modes via CLI subcommands:
- **`Run`**: One-shot batch image generation
- **`Serve`**: Axum HTTP API server (default `127.0.0.1:8787`)

**Image generation pipeline:**
```
VariantGenerator (template → prompt variants)
  → Orchestrator (concurrency/rate limiting via Semaphore)
    → Optional PromptRewriter (GPT-4o-mini polish)
    → ImageProvider (Mock or OpenAI DALL-E)
    → Optional PerceptualDeduper (pHash filtering)
    → PostProcessor (thumbnails)
    → save_image_with_sidecar (atomic write-then-rename)
    → Manifest append (JSONL log)
    → EventEmitter (SSE broadcast to frontend)
```

**Key modules:**
- `orchestrator.rs` — concurrency control, worker dispatch, event emission
- `providers.rs` — `ImageProvider` trait with `MockProvider` (random noise PNG) and `OpenAIProvider`
- `prompts.rs` — `VariantGenerator` builds prompts from template (brand + product + style)
- `rewrite.rs` — `PromptRewriter` trait, `OpenAIRewriter` with optional cache
- `dedupe.rs` — perceptual hash deduplication with configurable threshold
- `io.rs` — atomic PNG saving + JSON sidecar metadata
- `manifest.rs` — append-only JSONL manifest
- `api.rs` — Axum routes, SSE streaming, CORS, image serving with path traversal protection
- `events.rs` — `RunEvent` enum (Started, Log, Progress, Finished, Failed) over broadcast channel

### Frontend (`adgen-ui/`)

React 19 + TypeScript + Vite + Tailwind CSS (dark theme). API client points to `http://127.0.0.1:8787`.

**Key components:**
- `ConfigEditor` — form/raw-YAML dual mode with React Hook Form + Zod validation
- `TemplateEditor` — brand, product, dynamic style list editing
- `RunMonitor` — SSE-connected real-time progress (progress bar, live logs, status pill)
- Gallery — grid display of generated images from `/api/images`

**API routes served by backend:**
- `GET/PUT /api/config` — run configuration
- `GET/PUT /api/template` — prompt template
- `POST /api/run` — start generation run
- `GET /api/run/{id}/events` — SSE event stream
- `GET /api/images` — list output images
- `GET /images/{name}` — serve individual image files

## Configuration

**`run-config.yaml`** controls provider (mock/openai), orchestrator (concurrency, rate limiting, backoff), deduplication (pHash), post-processing (thumbnails), prompt rewriting, output directory, and seed.

**`template.yml`** defines brand, product, and style variants for prompt generation.

**Environment:** `OPENAI_API_KEY` must be set for the OpenAI provider (referenced via `config.provider.api_key_env`).

## Output Artifacts

Generated files in `out_dir`:
- `{id:08}-{provider}-{model}.png` — generated image
- `{id:08}-{provider}-{model}.json` — sidecar metadata (prompts, cost, dimensions, timestamps)
- `manifest.jsonl` — append-only log of all generated images
