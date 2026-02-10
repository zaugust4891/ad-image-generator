export function LandingPage({ onLogin, onSignup }: { onLogin: () => void; onSignup: () => void }) {
  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      {/* Nav */}
      <nav className="mx-auto flex max-w-6xl items-center justify-between px-6 py-5">
        <div className="text-lg font-semibold tracking-tight">adgen</div>
        <div className="flex items-center gap-3">
          <button
            onClick={onLogin}
            className="rounded-lg px-4 py-2 text-sm text-zinc-300 transition hover:text-white"
          >
            Login
          </button>
          <button
            onClick={onSignup}
            className="rounded-lg bg-white px-4 py-2 text-sm font-semibold text-zinc-950 transition hover:bg-zinc-200"
          >
            Sign Up
          </button>
        </div>
      </nav>

      {/* Hero */}
      <section className="mx-auto max-w-6xl px-6 pb-24 pt-20 text-center">
        <div className="mx-auto max-w-2xl">
          <p className="mb-4 text-sm font-medium uppercase tracking-widest text-zinc-500">
            AI-powered ad generation
          </p>
          <h1 className="text-5xl font-bold leading-tight tracking-tight sm:text-6xl">
            One prompt.
            <br />
            <span className="text-zinc-400">Endless variations.</span>
          </h1>
          <p className="mx-auto mt-6 max-w-lg text-lg text-zinc-400">
            Write a single creative brief and let adgen produce dozens of
            unique image variations — different styles, compositions, and
            perspectives — all from one prompt.
          </p>
          <div className="mt-10 flex items-center justify-center gap-4">
            <button
              onClick={onSignup}
              className="inline-flex items-center gap-2 rounded-lg bg-white px-6 py-3 text-sm font-semibold text-zinc-950 transition hover:bg-zinc-200"
            >
              Get Started
              <span aria-hidden="true">&rarr;</span>
            </button>
            <button
              onClick={onLogin}
              className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 px-6 py-3 text-sm font-semibold text-zinc-100 transition hover:bg-zinc-900"
            >
              Login
            </button>
          </div>
        </div>

        {/* Dashboard preview */}
        <div className="mx-auto mt-16 max-w-4xl overflow-hidden rounded-2xl border border-zinc-800 bg-zinc-900/30 p-1 shadow-2xl shadow-black/40">
          <div className="rounded-xl bg-zinc-900/60 px-6 py-10">
            <div className="grid grid-cols-3 gap-4">
              <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
                <div className="mb-2 h-2 w-12 rounded bg-zinc-700" />
                <div className="h-24 rounded-lg bg-zinc-800/50" />
              </div>
              <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
                <div className="mb-2 h-2 w-16 rounded bg-zinc-700" />
                <div className="h-24 rounded-lg bg-zinc-800/50" />
              </div>
              <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
                <div className="mb-2 h-2 w-10 rounded bg-zinc-700" />
                <div className="h-24 rounded-lg bg-zinc-800/50" />
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Features grid */}
      <section className="mx-auto max-w-6xl px-6 py-20">
        <div className="mb-12 text-center">
          <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
            How it works
          </h2>
          <p className="mx-auto mt-3 max-w-md text-zinc-400">
            From a single prompt to a full creative suite — in one run.
          </p>
        </div>

        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          <FeatureCard
            step="01"
            title="Define your template"
            description="Set your brand, product, and style variants. adgen combines them into unique prompt permutations automatically."
          />
          <FeatureCard
            step="02"
            title="Expand with AI rewriting"
            description="Each prompt variant is optionally rewritten by an AI model to add creative nuance — turning one idea into many."
          />
          <FeatureCard
            step="03"
            title="Generate in batch"
            description="Spin up concurrent image generation across your chosen provider with built-in rate limiting and backoff."
          />
          <FeatureCard
            step="04"
            title="Deduplicate intelligently"
            description="Perceptual hashing filters out near-duplicate results so every image in your output is distinct."
          />
          <FeatureCard
            step="05"
            title="Monitor in real time"
            description="Watch progress live via SSE streaming — see logs, status, and generated images as they arrive."
          />
          <FeatureCard
            step="06"
            title="Own your output"
            description="Everything runs locally. Images, metadata sidecars, and manifests stay on your machine."
          />
        </div>
      </section>

      {/* Alternating feature sections */}
      <section className="mx-auto max-w-6xl px-6 py-20">
        <div className="grid items-center gap-12 lg:grid-cols-2">
          <div>
            <p className="mb-2 text-sm font-medium uppercase tracking-widest text-zinc-500">
              Prompt expansion
            </p>
            <h3 className="text-2xl font-bold tracking-tight sm:text-3xl">
              One brief, infinite angles
            </h3>
            <p className="mt-4 text-zinc-400">
              Write a single creative direction. adgen's template engine crosses
              your brand voice, product details, and style modifiers to produce
              every combination — then AI rewriting adds polish and variety to
              each variant.
            </p>
            <ul className="mt-6 space-y-3 text-sm text-zinc-400">
              <li className="flex items-start gap-2">
                <span className="mt-0.5 block h-1.5 w-1.5 rounded-full bg-zinc-500" />
                Template-driven prompt permutations
              </li>
              <li className="flex items-start gap-2">
                <span className="mt-0.5 block h-1.5 w-1.5 rounded-full bg-zinc-500" />
                Optional AI rewriting for creative variety
              </li>
              <li className="flex items-start gap-2">
                <span className="mt-0.5 block h-1.5 w-1.5 rounded-full bg-zinc-500" />
                Full control over style and tone
              </li>
            </ul>
          </div>
          <div className="rounded-2xl border border-zinc-800 bg-zinc-900/30 p-6">
            <div className="space-y-3">
              <div className="rounded-lg bg-zinc-800/50 px-4 py-3 text-sm text-zinc-400">
                "Minimalist product shot, soft lighting..."
              </div>
              <div className="flex items-center justify-center">
                <div className="h-6 w-px bg-zinc-700" />
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div className="rounded-lg bg-zinc-800/50 px-3 py-2 text-xs text-zinc-500">
                  Variant A — warm tones
                </div>
                <div className="rounded-lg bg-zinc-800/50 px-3 py-2 text-xs text-zinc-500">
                  Variant B — cool tones
                </div>
                <div className="rounded-lg bg-zinc-800/50 px-3 py-2 text-xs text-zinc-500">
                  Variant C — high contrast
                </div>
                <div className="rounded-lg bg-zinc-800/50 px-3 py-2 text-xs text-zinc-500">
                  Variant D — editorial
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="mt-24 grid items-center gap-12 lg:grid-cols-2">
          <div className="order-2 lg:order-1 rounded-2xl border border-zinc-800 bg-zinc-900/30 p-6">
            <div className="space-y-2">
              <div className="flex items-center gap-3">
                <div className="h-2 w-2 rounded-full bg-zinc-500" />
                <div className="h-2 flex-1 rounded bg-zinc-800" />
              </div>
              <div className="flex items-center gap-3">
                <div className="h-2 w-2 rounded-full bg-zinc-400" />
                <div className="h-2 w-3/4 rounded bg-zinc-800" />
              </div>
              <div className="flex items-center gap-3">
                <div className="h-2 w-2 rounded-full bg-zinc-400" />
                <div className="h-2 w-1/2 rounded bg-zinc-800" />
              </div>
              <div className="mt-4 h-3 w-full rounded-full bg-zinc-800">
                <div className="h-3 w-2/3 rounded-full bg-zinc-600" />
              </div>
            </div>
          </div>
          <div className="order-1 lg:order-2">
            <p className="mb-2 text-sm font-medium uppercase tracking-widest text-zinc-500">
              Real-time monitoring
            </p>
            <h3 className="text-2xl font-bold tracking-tight sm:text-3xl">
              Watch every generation live
            </h3>
            <p className="mt-4 text-zinc-400">
              SSE-powered event streaming delivers logs, progress updates, and
              results to your browser the instant they happen. No polling, no
              page refreshes.
            </p>
            <ul className="mt-6 space-y-3 text-sm text-zinc-400">
              <li className="flex items-start gap-2">
                <span className="mt-0.5 block h-1.5 w-1.5 rounded-full bg-zinc-500" />
                Live progress bar and status updates
              </li>
              <li className="flex items-start gap-2">
                <span className="mt-0.5 block h-1.5 w-1.5 rounded-full bg-zinc-500" />
                Streaming log output
              </li>
              <li className="flex items-start gap-2">
                <span className="mt-0.5 block h-1.5 w-1.5 rounded-full bg-zinc-500" />
                Instant image preview on completion
              </li>
            </ul>
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="mx-auto max-w-6xl px-6 py-20">
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 px-6 py-16 text-center">
          <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
            Ready to generate?
          </h2>
          <p className="mx-auto mt-3 max-w-md text-zinc-400">
            Set up your first template and produce ad variations in minutes.
          </p>
          <div className="mt-8 flex items-center justify-center gap-4">
            <button
              onClick={onSignup}
              className="inline-flex items-center gap-2 rounded-lg bg-white px-6 py-3 text-sm font-semibold text-zinc-950 transition hover:bg-zinc-200"
            >
              Sign Up
              <span aria-hidden="true">&rarr;</span>
            </button>
            <button
              onClick={onLogin}
              className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 px-6 py-3 text-sm font-semibold text-zinc-100 transition hover:bg-zinc-900"
            >
              Login
            </button>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="mx-auto max-w-6xl border-t border-zinc-800 px-6 py-8">
        <div className="flex items-center justify-between text-xs text-zinc-500">
          <span>adgen — local AI image generation</span>
          <span>Built for creators who own their pipeline.</span>
        </div>
      </footer>
    </div>
  );
}

function FeatureCard({
  step,
  title,
  description,
}: {
  step: string;
  title: string;
  description: string;
}) {
  return (
    <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-6 transition hover:border-zinc-700 hover:bg-zinc-900/40">
      <div className="mb-3 text-xs font-medium text-zinc-500">{step}</div>
      <div className="mb-2 text-sm font-semibold text-zinc-100">{title}</div>
      <div className="text-sm text-zinc-400">{description}</div>
    </div>
  );
}
