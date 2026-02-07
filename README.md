# adgen (ad-image-generator-m1)

Rust + React toolchain for generating batches of ad images from prompt templates, with live run monitoring over SSE.

## What It Does

- Generates images in batches using configurable providers (`mock` or `openai`)
- Builds prompts in either `AdTemplate` mode (brand/product/styles) or `GeneralPrompt` mode (single fixed prompt)
- Supports optional prompt rewriting, deduplication, thumbnails, and retry/backoff
- Saves image artifacts with sidecar metadata + a JSONL manifest
- Exposes a local API for config/template editing, run control, progress streaming, and image gallery
- Includes a React UI (`adgen-ui/`) for local operation

## Repository Layout

```text
.
├── src/                 # Rust backend (CLI + HTTP API)
├── adgen-ui/            # React/Vite frontend
├── run-config.yaml      # Runtime config (provider/orchestrator/rewrite/etc.)
├── template.yml         # Prompt template definition
├── Dockerfile
└── docker-compose.yml
```

## Prerequisites

- Rust toolchain (project currently builds with `cargo run`)
- Node.js 20+ and npm (for `adgen-ui`)
- OpenAI API key (only required when `provider.kind: openai`)

## Environment Variables

Required for OpenAI provider:

```bash
export OPENAI_API_KEY=sk-...
```

If you use `.env`, load it in your shell first (the backend does not auto-load `.env` in `main.rs`):

```bash
set -a
source .env
set +a
```

## Quick Start (Local)

1. Configure `run-config.yaml`:
   - Set `out_dir` to a writable local directory (for example `./output`)
   - Use `provider.kind: mock` for no-cost local testing
2. Configure prompts in `template.yml` (examples in the Template section below)
3. Run one-shot generation:

```bash
cargo run -- run --config ./run-config.yaml --template ./template.yml --out-dir ./output
```

4. Start backend API server (for UI usage):

```bash
cargo run -- serve --bind 127.0.0.1:8787 --config-path ./run-config.yaml --template-path ./template.yml
```

5. Start frontend (new terminal):

```bash
cd adgen-ui
npm install
VITE_API_BASE_URL=http://127.0.0.1:8787 npm run dev
```

Open the UI at the Vite URL (typically `http://127.0.0.1:5173`).

## Docker Compose Development

The compose setup runs a dev sandbox container and maps:

- `localhost:8788` -> backend API (`8787` in container)
- `localhost:5174` -> Vite dev server (`5173` in container)

Start container:

```bash
docker compose up --build -d
```

Shell into it:

```bash
docker compose exec dev-sandbox bash
```

Inside the container:

Terminal A:

```bash
cargo run -- serve --bind 0.0.0.0:8787 --config-path ./run-config.yaml --template-path ./template.yml
```

Terminal B:

```bash
cd adgen-ui
npm install
npm run dev -- --host 0.0.0.0 --port 5173
```

Then open:

- UI: `http://localhost:5174`
- API: `http://localhost:8788`

## CLI Reference

Top-level:

```bash
adgen <COMMAND>
```

Commands:

- `run`: one-shot generation
- `serve`: start HTTP API

### `run` command

```bash
adgen run --config <PATH> --template <PATH> [--out-dir <PATH>] [--resume]
```

Options:

- `--config`: path to run config YAML
- `--template`: path to template YAML
- `--out-dir`: optional override for `out_dir` from config
- `--resume`: currently parsed, but not used in orchestration logic

### `serve` command

```bash
adgen serve [--bind <ADDR>] [--config-path <PATH>] [--template-path <PATH>]
```

Defaults:

- `--bind`: `0.0.0.0:8787`
- `--config-path`: `./run-config.yaml`
- `--template-path`: `./template.yml`

## Configuration (`run-config.yaml`)

Current schema:

```yaml
provider:
  kind: openai # or mock
  model: gpt-image-1.5
  api_key_env: OPENAI_API_KEY # optional, defaults to OPENAI_API_KEY
  width: 1024
  height: 1024
  price_usd_per_image: 0.0
orchestrator:
  target_images: 25
  concurrency: 8
  queue_cap: 32
  rate_per_min: 60
  backoff_base_ms: 200
  backoff_factor: 2.0
  backoff_jitter_ms: 250
dedupe:
  enabled: false
  phash_bits: 64
  phash_thresh: 10
post:
  thumbnail: false
  thumb_max: 256
rewrite:
  enabled: false
  model: gpt-4o-mini
  system: Polish and improve the ad prompt while preserving its core intent.
  max_tokens: 64
  cache_file: ./rewrite-cache.jsonl
out_dir: ./output
seed: 42
```

Notes:

- `provider.kind: mock` generates random noise PNGs for local testing.
- `rate_per_min`, `concurrency`, and backoff settings control provider pressure.
- When `rewrite.enabled: true`, rewritten prompts can be cached if `cache_file` is set.
- `serve` validates `out_dir` at startup and fails fast if not writable.

## Template (`template.yml`)

The backend supports two modes via YAML-tagged enums.

### `AdTemplate` mode

```yaml
mode: !AdTemplate
  brand: Lumiere Botanica
  product: Midnight Recovery Serum
  styles:
    - Luxurious editorial photography with soft diffused lighting
    - Minimalist Scandinavian aesthetic with dramatic side lighting
```

Prompt generation output pattern:

```text
An advertisement image for <brand> <product> in style: <selected_style>
```

### `GeneralPrompt` mode

```yaml
mode: !GeneralPrompt
  prompt: Cinematic product shot of a glass skincare bottle on wet stone, moody lighting.
```

API clients should send/expect `TemplateYaml` in enum form. Example JSON payloads:

```json
{
  "mode": {
    "AdTemplate": {
      "brand": "Lumiere Botanica",
      "product": "Midnight Recovery Serum",
      "styles": ["Luxurious editorial photography"]
    }
  }
}
```

```json
{
  "mode": {
    "GeneralPrompt": {
      "prompt": "Cinematic product shot of a glass skincare bottle on wet stone, moody lighting."
    }
  }
}
```

## HTTP API Reference

Base URL example: `http://127.0.0.1:8787`

- `GET /api/config`: returns current run config JSON
- `PUT /api/config`: replaces config JSON
- `POST /api/config/validate`: validates config + template payload

```json
{
  "config": { "...": "RunCfg" },
  "template": { "...": "TemplateYaml" }
}
```

- Validation response shape:

```json
{
  "valid": true,
  "errors": [],
  "warnings": []
}
```

- `GET /api/template`: returns template JSON
- `PUT /api/template`: replaces template JSON
- `POST /api/run`: starts a run and returns `{ "run_id": "run-..." }` (`409` if another run is active)
- `GET /api/run/current`: returns `{ "run_id": "<id-or-null>" }`
- `GET /api/run/{id}/events`: SSE stream (`started`, `log`, `progress`, `finished`, `failed`)
- `GET /api/images`: lists generated PNGs from `out_dir`
- `GET /images/{name}`: serves a safe filename from `out_dir`

## Output Artifacts

Each accepted image writes:

- `00000001-<provider>-<model>.png`
- `00000001-<provider>-<model>.json` (sidecar metadata)
- Optional `00000001-<provider>-<model>_thumb.png` (if thumbnails enabled)

Plus append-only:

- `manifest.jsonl` (one JSON record per generated/saved image)

Sidecar includes:

- IDs/run ID/provider/model/dimensions
- Timestamp
- Original prompt and optional rewritten prompt
- Cost field (`cost_usd`)
- Optional thumbnail path

## Common Commands

Backend:

```bash
cargo check
cargo build
cargo run -- --help
cargo run -- run --help
cargo run -- serve --help
```

Frontend:

```bash
cd adgen-ui
npm install
npm run lint
npm run build
npm run dev
```

## Troubleshooting

- `Environment variable OPENAI_API_KEY not set`: export the key (or set `provider.api_key_env` to another env var name).
- `Output directory validation failed`: make `out_dir` writable and ensure it is a directory, not a file.
- `UI cannot talk to API`: verify backend bind/port and `VITE_API_BASE_URL`. For Docker compose defaults, use `http://localhost:8788`.
- `Run start fails with conflict`: only one run can be active at a time (`POST /api/run` returns 409 otherwise).
- `Provider throttling / retries`: lower `concurrency` and/or `rate_per_min`, or increase backoff values.

## Notes

- The codebase currently wires `mock` and `openai` providers in runtime selection.
- CORS is permissive in the local API server (`CorsLayer::permissive()`).
