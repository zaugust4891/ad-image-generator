export function LandingPage({ onLogin, onSignup }: { onLogin: () => void; onSignup: () => void }) {
  return (
    <div className="min-h-screen bg-black text-white selection:bg-[#4353FF]/30">
      {/* Nav */}
      <nav className="mx-auto flex max-w-7xl items-center justify-between px-8 py-6">
        <div className="text-xl font-bold tracking-tight">adgen</div>
        <div className="flex items-center gap-4">
          <button
            onClick={onLogin}
            className="px-4 py-2 text-sm text-zinc-400 transition hover:text-white"
          >
            Login
          </button>
          <button
            onClick={onSignup}
            className="rounded-full bg-[#4353FF] px-5 py-2.5 text-sm font-semibold text-white shadow-lg shadow-[#4353FF]/20 transition hover:bg-[#5563FF]"
          >
            Get Started
          </button>
        </div>
      </nav>

      {/* Hero */}
      <section className="relative mx-auto max-w-7xl px-8 pb-32 pt-28 text-center">
        {/* Background glow */}
        <div className="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
          <div className="absolute left-1/2 top-0 h-[600px] w-[900px] -translate-x-1/2 rounded-full bg-[#4353FF]/[0.07] blur-[120px]" />
        </div>

        <div className="relative mx-auto max-w-3xl">
          <div className="mb-8 inline-flex items-center gap-2 rounded-full border border-zinc-800 bg-zinc-900/60 px-4 py-1.5 text-xs text-zinc-400 backdrop-blur-sm">
            <span className="h-1.5 w-1.5 rounded-full bg-[#4353FF]" />
            AI-powered ad generation
          </div>
          <h1 className="text-6xl font-bold leading-[1.08] tracking-tight sm:text-7xl lg:text-8xl">
            One prompt.
            <br />
            <span className="bg-gradient-to-r from-zinc-500 to-zinc-700 bg-clip-text text-transparent">
              Endless variations.
            </span>
          </h1>
          <p className="mx-auto mt-8 max-w-xl text-lg leading-relaxed text-zinc-400">
            Write a single creative brief and let adgen produce dozens of
            unique image variations — different styles, compositions, and
            perspectives — all from one prompt.
          </p>
          <div className="mt-12 flex items-center justify-center gap-4">
            <button
              onClick={onSignup}
              className="inline-flex items-center gap-2 rounded-full bg-[#4353FF] px-8 py-3.5 text-sm font-semibold text-white shadow-xl shadow-[#4353FF]/25 transition hover:bg-[#5563FF]"
            >
              Start Generating
              <span aria-hidden="true">&rarr;</span>
            </button>
            <button
              onClick={onLogin}
              className="inline-flex items-center gap-2 rounded-full border border-zinc-800 px-8 py-3.5 text-sm font-semibold text-zinc-300 transition hover:border-zinc-600 hover:text-white"
            >
              Login
            </button>
          </div>
        </div>

        {/* Dashboard preview */}
        <div className="relative mx-auto mt-20 max-w-6xl overflow-hidden rounded-2xl border border-zinc-800/80 bg-zinc-950/60 p-1 shadow-2xl shadow-[#4353FF]/5 backdrop-blur-sm">
          <div className="rounded-xl bg-zinc-950/80 p-3">
            {/* Window chrome */}
            <div className="mb-3 flex items-center gap-1.5 px-2">
              <div className="h-2.5 w-2.5 rounded-full bg-zinc-800" />
              <div className="h-2.5 w-2.5 rounded-full bg-zinc-800" />
              <div className="h-2.5 w-2.5 rounded-full bg-zinc-800" />
              <div className="ml-3 h-4 flex-1 rounded bg-zinc-900 font-mono text-[9px] leading-4 text-zinc-600">
                localhost:8787
              </div>
            </div>

            <div className="grid grid-cols-[140px_1fr] gap-2">
              {/* Mini sidebar */}
              <div className="space-y-1.5 rounded-lg border border-zinc-800/60 bg-black/40 p-2.5">
                <div className="mb-2 px-1.5 text-[10px] font-semibold text-zinc-300">adgen</div>
                <div className="rounded-md border border-[#4353FF]/30 bg-[#4353FF]/10 px-1.5 py-1 text-[9px] text-[#8B96FF]">Dashboard</div>
                <div className="rounded-md px-1.5 py-1 text-[9px] text-zinc-600">Run Config</div>
                <div className="rounded-md px-1.5 py-1 text-[9px] text-zinc-600">Template</div>
                <div className="rounded-md px-1.5 py-1 text-[9px] text-zinc-600">Run Monitor</div>
                <div className="rounded-md px-1.5 py-1 text-[9px] text-zinc-600">Gallery</div>
                <div className="rounded-md px-1.5 py-1 text-[9px] text-zinc-600">Cost Tracking</div>
              </div>

              {/* Main content area */}
              <div className="space-y-2">
                {/* Stat cards row */}
                <div className="grid grid-cols-4 gap-1.5">
                  <div className="rounded-lg border border-zinc-800/60 bg-black/40 p-2">
                    <div className="text-[8px] text-zinc-500">Runs</div>
                    <div className="mt-0.5 text-sm font-semibold text-zinc-100">Ready</div>
                    <div className="mt-0.5 text-[7px] text-zinc-700">Start a run</div>
                  </div>
                  <div className="rounded-lg border border-zinc-800/60 bg-black/40 p-2">
                    <div className="text-[8px] text-zinc-500">Output</div>
                    <div className="mt-0.5 text-sm font-semibold text-zinc-100">out/</div>
                    <div className="mt-0.5 text-[7px] text-zinc-700">Gallery updates live</div>
                  </div>
                  <div className="rounded-lg border border-zinc-800/60 bg-black/40 p-2">
                    <div className="text-[8px] text-zinc-500">Health</div>
                    <div className="mt-0.5 flex items-center gap-1">
                      <div className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                      <span className="text-sm font-semibold text-zinc-100">Online</span>
                    </div>
                    <div className="mt-0.5 text-[7px] text-zinc-700">localhost:8787</div>
                  </div>
                  <div className="rounded-lg border border-zinc-800/60 bg-black/40 p-2">
                    <div className="text-[8px] text-zinc-500">Next Run Est.</div>
                    <div className="mt-0.5 text-sm font-semibold text-[#4353FF]">$0.3200</div>
                    <div className="mt-0.5 text-[7px] text-zinc-700">Based on config</div>
                  </div>
                </div>

                {/* Progress bar section */}
                <div className="rounded-lg border border-zinc-800/60 bg-black/40 p-2.5">
                  <div className="mb-1.5 flex items-center justify-between">
                    <div className="text-[9px] text-zinc-500">Progress — Run #a3f8c2</div>
                    <div className="flex items-center gap-1.5">
                      <div className="text-[8px] text-zinc-600">Cost: <span className="text-zinc-400">$0.2400</span></div>
                      <div className="rounded-full bg-[#4353FF]/15 px-1.5 py-0.5 text-[8px] font-medium text-[#8B96FF]">
                        Running
                      </div>
                    </div>
                  </div>
                  <div className="h-2 w-full overflow-hidden rounded-full bg-zinc-900">
                    <div className="h-full w-3/4 rounded-full bg-gradient-to-r from-[#4353FF] to-[#6B78FF]" />
                  </div>
                  <div className="mt-1 text-[8px] text-zinc-600">6 / 8 images generated</div>
                </div>

                {/* Mini gallery grid */}
                <div className="rounded-lg border border-zinc-800/60 bg-black/40 p-2.5">
                  <div className="mb-1.5 text-[9px] font-medium text-zinc-500">Generated Images</div>
                  <div className="grid grid-cols-5 gap-1">
                    <img src="/preview/texas_1.png" alt="Generated ad example 1" className="aspect-square rounded object-cover" />
                    <img src="/preview/texas_2.png" alt="Generated ad example 2" className="aspect-square rounded object-cover" />
                    <img src="/preview/texas_3.png" alt="Generated ad example 3" className="aspect-square rounded object-cover" />
                    <img src="/preview/texas_4.png" alt="Generated ad example 4" className="aspect-square rounded object-cover" />
                    <img src="/preview/texas_5.png" alt="Generated ad example 5" className="aspect-square rounded object-cover" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Trusted by — social proof */}
      <section className="border-y border-zinc-900 py-16">
        <div className="mx-auto max-w-7xl px-8">
          <p className="mb-10 text-center text-xs uppercase tracking-widest text-zinc-600">
            Trusted by creative teams worldwide
          </p>
          <div className="flex flex-wrap items-center justify-center gap-x-16 gap-y-6 text-lg font-semibold text-zinc-800">
            <span>Studio&thinsp;Co</span>
            <span>Aperture</span>
            <span>Brandwave</span>
            <span>PixelForge</span>
            <span>Creatix</span>
          </div>
        </div>
      </section>

      {/* Features grid */}
      <section className="relative mx-auto max-w-7xl px-8 py-32">
        <div className="mb-16 text-center">
          <p className="mb-4 text-xs uppercase tracking-widest text-[#4353FF]">How it works</p>
          <h2 className="text-4xl font-bold tracking-tight sm:text-5xl">
            From prompt to production
          </h2>
          <p className="mx-auto mt-4 max-w-lg text-zinc-400">
            Six steps from a single creative brief to a full ad suite.
          </p>
        </div>

        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
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
      <section className="mx-auto max-w-7xl px-8 py-32">
        <div className="grid items-center gap-16 lg:grid-cols-2">
          <div>
            <p className="mb-3 text-xs uppercase tracking-widest text-[#4353FF]">
              Prompt expansion
            </p>
            <h3 className="text-3xl font-bold tracking-tight sm:text-4xl">
              One brief, infinite angles
            </h3>
            <p className="mt-5 text-lg leading-relaxed text-zinc-400">
              Write a single creative direction. adgen's template engine crosses
              your brand voice, product details, and style modifiers to produce
              every combination — then AI rewriting adds polish and variety to
              each variant.
            </p>
            <ul className="mt-8 space-y-4 text-sm text-zinc-400">
              <li className="flex items-start gap-3">
                <span className="mt-1 block h-1.5 w-1.5 rounded-full bg-[#4353FF]" />
                Template-driven prompt permutations
              </li>
              <li className="flex items-start gap-3">
                <span className="mt-1 block h-1.5 w-1.5 rounded-full bg-[#4353FF]" />
                Optional AI rewriting for creative variety
              </li>
              <li className="flex items-start gap-3">
                <span className="mt-1 block h-1.5 w-1.5 rounded-full bg-[#4353FF]" />
                Full control over style and tone
              </li>
            </ul>
          </div>
          <div className="rounded-2xl border border-zinc-800/60 bg-zinc-950/40 p-8">
            <div className="space-y-4">
              <div className="rounded-lg border border-zinc-800/40 bg-black/40 px-5 py-3.5 text-sm text-zinc-400">
                "Minimalist product shot, soft lighting..."
              </div>
              <div className="flex items-center justify-center">
                <div className="h-8 w-px bg-zinc-800" />
              </div>
              <div className="grid grid-cols-2 gap-2.5">
                <div className="rounded-lg border border-zinc-800/40 bg-black/40 px-4 py-2.5 text-xs text-zinc-500">
                  Variant A — warm tones
                </div>
                <div className="rounded-lg border border-zinc-800/40 bg-black/40 px-4 py-2.5 text-xs text-zinc-500">
                  Variant B — cool tones
                </div>
                <div className="rounded-lg border border-zinc-800/40 bg-black/40 px-4 py-2.5 text-xs text-zinc-500">
                  Variant C — high contrast
                </div>
                <div className="rounded-lg border border-zinc-800/40 bg-black/40 px-4 py-2.5 text-xs text-zinc-500">
                  Variant D — editorial
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="mt-32 grid items-center gap-16 lg:grid-cols-2">
          <div className="order-2 rounded-2xl border border-zinc-800/60 bg-zinc-950/40 p-8 lg:order-1">
            <div className="space-y-3">
              <div className="flex items-center gap-3">
                <div className="h-2 w-2 rounded-full bg-[#4353FF]" />
                <div className="h-2 flex-1 rounded bg-zinc-900" />
              </div>
              <div className="flex items-center gap-3">
                <div className="h-2 w-2 rounded-full bg-[#6B78FF]" />
                <div className="h-2 w-3/4 rounded bg-zinc-900" />
              </div>
              <div className="flex items-center gap-3">
                <div className="h-2 w-2 rounded-full bg-[#6B78FF]" />
                <div className="h-2 w-1/2 rounded bg-zinc-900" />
              </div>
              <div className="mt-5 h-3 w-full overflow-hidden rounded-full bg-zinc-900">
                <div className="h-3 w-2/3 rounded-full bg-gradient-to-r from-[#4353FF] to-[#6B78FF]" />
              </div>
            </div>
          </div>
          <div className="order-1 lg:order-2">
            <p className="mb-3 text-xs uppercase tracking-widest text-[#4353FF]">
              Real-time monitoring
            </p>
            <h3 className="text-3xl font-bold tracking-tight sm:text-4xl">
              Watch every generation live
            </h3>
            <p className="mt-5 text-lg leading-relaxed text-zinc-400">
              SSE-powered event streaming delivers logs, progress updates, and
              results to your browser the instant they happen. No polling, no
              page refreshes.
            </p>
            <ul className="mt-8 space-y-4 text-sm text-zinc-400">
              <li className="flex items-start gap-3">
                <span className="mt-1 block h-1.5 w-1.5 rounded-full bg-[#4353FF]" />
                Live progress bar and status updates
              </li>
              <li className="flex items-start gap-3">
                <span className="mt-1 block h-1.5 w-1.5 rounded-full bg-[#4353FF]" />
                Streaming log output
              </li>
              <li className="flex items-start gap-3">
                <span className="mt-1 block h-1.5 w-1.5 rounded-full bg-[#4353FF]" />
                Instant image preview on completion
              </li>
            </ul>
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="relative mx-auto max-w-7xl px-8 py-32">
        <div className="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
          <div className="absolute bottom-0 left-1/2 h-[400px] w-[700px] -translate-x-1/2 rounded-full bg-[#4353FF]/[0.05] blur-[100px]" />
        </div>
        <div className="relative rounded-3xl border border-zinc-800/60 bg-zinc-950/40 px-8 py-20 text-center backdrop-blur-sm">
          <h2 className="text-4xl font-bold tracking-tight sm:text-5xl">
            Ready to generate?
          </h2>
          <p className="mx-auto mt-5 max-w-md text-lg text-zinc-400">
            Set up your first template and produce ad variations in minutes.
          </p>
          <div className="mt-10 flex items-center justify-center gap-4">
            <button
              onClick={onSignup}
              className="inline-flex items-center gap-2 rounded-full bg-[#4353FF] px-8 py-3.5 text-sm font-semibold text-white shadow-xl shadow-[#4353FF]/25 transition hover:bg-[#5563FF]"
            >
              Get Started Free
              <span aria-hidden="true">&rarr;</span>
            </button>
            <button
              onClick={onLogin}
              className="inline-flex items-center gap-2 rounded-full border border-zinc-800 px-8 py-3.5 text-sm font-semibold text-zinc-300 transition hover:border-zinc-600 hover:text-white"
            >
              Login
            </button>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="mx-auto max-w-7xl border-t border-zinc-900 px-8 py-10">
        <div className="flex items-center justify-between text-xs text-zinc-600">
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
    <div className="group rounded-2xl border border-zinc-800/60 bg-zinc-950/40 p-7 transition-all duration-300 hover:border-[#4353FF]/30 hover:bg-zinc-950/60 hover:shadow-lg hover:shadow-[#4353FF]/5">
      <div className="mb-4 text-xs font-medium text-[#4353FF]">{step}</div>
      <div className="mb-2 text-base font-semibold text-zinc-100">{title}</div>
      <div className="text-sm leading-relaxed text-zinc-500">{description}</div>
    </div>
  );
}
