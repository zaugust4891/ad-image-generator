import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listImages, startRun, getCurrentRun, getConfig, getTemplate, validateConfig, getCostEstimate, getCostSummary } from "./lib/api";
import type { UserResponse, ValidationResult } from "./lib/api";
import { TemplateEditor } from "./components/TemplateEditor";
import { ConfigEditor } from "./components/ConfigEditor";
import { RunMonitor } from "./components/RunMonitor";
import { LandingPage } from "./components/LandingPage";
import { AuthPage } from "./components/AuthPage";
import { CostDashboard } from "./components/CostDashboard";


type Nav = "dashboard" | "config" | "template" | "run" | "gallery" | "costs";

export default function App() {
  const [user, setUser] = useState<UserResponse | null>(null);
  const [authMode, setAuthMode] = useState<"login" | "signup" | null>(null);
  const [nav, setNav] = useState<Nav>("dashboard");
  const [runId, setRunId] = useState<string | null>(null);

  // Gallery images state - lifted to App for real-time refresh
  const [images, setImages] = useState<{ name: string; url: string; created_ms: number }[]>([]);

  // Debounced gallery refresh
  const refreshTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const refreshGallery = useCallback(() => {
    // Clear any pending refresh
    if (refreshTimeoutRef.current) {
      clearTimeout(refreshTimeoutRef.current);
    }
    // Debounce by 500ms to avoid too many API calls
    refreshTimeoutRef.current = setTimeout(() => {
      listImages().then(setImages).catch(() => setImages([]));
    }, 500);
  }, []);

  // Initial gallery load
  useEffect(() => {
    listImages().then(setImages).catch(() => setImages([]));
  }, []);

  const title = useMemo(() => {
    if (nav === "dashboard") return "Dashboard";
    if (nav === "config") return "Run Config";
    if (nav === "template") return "Template";
    if (nav === "run") return "Run Monitor";
    if (nav === "costs") return "Cost Tracking";
    return "Gallery";
  }, [nav]);

  // Check for active run on mount
  useEffect(() => {
    getCurrentRun().then(({ run_id }) => {
      if (run_id) {
        setRunId(run_id);
        setNav("run");
      }
    }).catch(() => {
      // Ignore errors - backend might not be running
    });
  }, []);

  const [runLoading, setRunLoading] = useState(false);
  const [runError, setRunError] = useState<string | null>(null);

  // Validation state
  const [validating, setValidating] = useState(false);
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);

  async function handleValidate() {
    setValidating(true);
    setValidationResult(null);
    try {
      const config = await getConfig();
      const template = await getTemplate();
      const result = await validateConfig(config, template);
      setValidationResult(result);
    } catch (err: any) {
      setValidationResult({
        valid: false,
        errors: [{ field: "request", message: err?.message ?? "Validation failed" }],
        warnings: [],
      });
    } finally {
      setValidating(false);
    }
  }

  async function handleStartRun() {
    setRunLoading(true);
    setRunError(null);
    try {
      const { run_id } = await startRun();
      setRunId(run_id);
      setNav("run");
    } catch (err: any) {
      setRunError(err?.message ?? "Failed to start run");
    } finally {
      setRunLoading(false);
    }
  }

  if (!user && authMode) {
    return (
      <AuthPage
        initialMode={authMode}
        onAuth={(u) => { setUser(u); setAuthMode(null); }}
        onBack={() => setAuthMode(null)}
      />
    );
  }

  if (!user) {
    return (
      <LandingPage
        onLogin={() => setAuthMode("login")}
        onSignup={() => setAuthMode("signup")}
      />
    );
  }

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      <div className="mx-auto grid max-w-7xl grid-cols-[260px_1fr] gap-6 p-6">
        <Sidebar nav={nav} setNav={setNav} user={user} onLogout={() => setUser(null)} />

        <main className="rounded-2xl border border-zinc-800 bg-zinc-950/40 shadow-[0_0_0_1px_rgba(255,255,255,0.04)] backdrop-blur">
          <Topbar title={title} onRun={handleStartRun} runLoading={runLoading} runError={runError} />

          <div className="p-6">
            {nav === "dashboard" && (
              <Dashboard
                onOpenGallery={() => setNav("gallery")}
                onValidate={handleValidate}
                validating={validating}
                validationResult={validationResult}
                onClearValidation={() => setValidationResult(null)}
              />
            )}
            {nav === "config" && <ConfigEditor />}
            {nav === "template" && <TemplateEditor />}
            {nav === "run" && <RunMonitor runId={runId} onStartRun={handleStartRun} onImageAdded={refreshGallery} />}
            {nav === "costs" && <CostDashboard />}
            {nav === "gallery" && <Gallery images={images} />}
          </div>
        </main>
      </div>
    </div>
  );
}

function Sidebar({ nav, setNav, user, onLogout }: { nav: Nav; setNav: (n: Nav) => void; user: UserResponse; onLogout: () => void }) {
  const item = (id: Nav, label: string) => (
    <button
      onClick={() => setNav(id)}
      className={[
        "w-full rounded-xl px-3 py-2 text-left text-sm transition",
        nav === id ? "bg-zinc-900 text-white" : "text-zinc-300 hover:bg-zinc-900/60 hover:text-white",
      ].join(" ")}
    >
      {label}
    </button>
  );

  return (
    <aside className="flex flex-col rounded-2xl border border-zinc-800 bg-zinc-950/40 p-4 backdrop-blur">
      <div className="mb-4">
        <div className="text-lg font-semibold tracking-tight">adgen</div>
        <div className="text-xs text-zinc-400">Local image generation studio</div>
      </div>
      <div className="space-y-2">
        {item("dashboard", "Dashboard")}
        {item("config", "Run Config")}
        {item("template", "Template")}
        {item("run", "Run Monitor")}
        {item("gallery", "Gallery")}
        {item("costs", "Cost Tracking")}
      </div>
      <div className="mt-6 rounded-xl border border-zinc-800 bg-zinc-900/30 p-3 text-xs text-zinc-400">
        Tip: keep concurrency + rate balanced to avoid provider throttling.
      </div>
      <div className="mt-auto pt-4 border-t border-zinc-800">
        <div className="flex items-center justify-between">
          <div className="min-w-0">
            <div className="truncate text-sm font-medium text-zinc-200">
              {user.name || user.email}
            </div>
            {user.name && (
              <div className="truncate text-xs text-zinc-500">{user.email}</div>
            )}
          </div>
          <button
            onClick={onLogout}
            className="ml-2 shrink-0 rounded-lg px-2 py-1 text-xs text-zinc-500 transition hover:bg-zinc-800 hover:text-zinc-300"
          >
            Log out
          </button>
        </div>
      </div>
    </aside>
  );
}

function Topbar({ title, onRun, runLoading, runError }: { title: string; onRun: () => void; runLoading: boolean; runError: string | null }) {
  return (
    <div className="flex items-center justify-between border-b border-zinc-800 px-6 py-4">
      <div className="flex items-center gap-3">
        <div className="text-base font-semibold tracking-tight">{title}</div>
        {runError && <div className="text-xs text-red-400">{runError}</div>}
      </div>
      <button
        onClick={onRun}
        disabled={runLoading}
        className={[
          "rounded-xl px-4 py-2 text-sm font-semibold",
          runLoading ? "bg-zinc-700 text-zinc-300" : "bg-white text-zinc-950 hover:bg-zinc-200",
        ].join(" ")}
      >
        {runLoading ? "Starting..." : "Run generation"}
      </button>
    </div>
  );
}

function Dashboard({
  onOpenGallery,
  onValidate,
  validating,
  validationResult,
  onClearValidation,
}: {
  onOpenGallery: () => void;
  onValidate: () => void;
  validating: boolean;
  validationResult: ValidationResult | null;
  onClearValidation: () => void;
}) {
  const [estimate, setEstimate] = useState<number | null>(null);
  const [totalSpend, setTotalSpend] = useState<number | null>(null);
  const [budget, setBudget] = useState<number | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const [cfg, summary] = await Promise.all([getConfig(), getCostSummary()]);
        setTotalSpend(summary.total_cost);
        setBudget(cfg.budget_limit_usd ?? null);
        const price = cfg.provider.price_usd_per_image ?? 0;
        if (price > 0) {
          const est = await getCostEstimate(cfg.orchestrator.target_images, price);
          setEstimate(est.estimated_cost);
        }
      } catch { /* ignore on dashboard */ }
    }
    load();
  }, []);

  const overBudget = budget && budget > 0 && estimate !== null && totalSpend !== null
    && (totalSpend + estimate) > budget;

  return (
    <div className="grid gap-4">
      <div className="grid grid-cols-4 gap-4">
        <Card title="Runs" value="Ready" sub="Start a run to see live progress." />
        <Card title="Output" value="out/" sub="Gallery updates as images land." />
        <Card title="Health" value="Local API" sub="localhost:8787 expected." />
        <Card
          title="Next Run Est."
          value={estimate !== null ? `$${estimate.toFixed(4)}` : "—"}
          sub={overBudget ? "⚠ Exceeds budget" : "Based on config"}
        />
      </div>

      <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-5">
        <div className="mb-2 text-sm font-semibold">Quick actions</div>
        <div className="flex gap-3">
          <button onClick={onOpenGallery} className="rounded-xl border border-zinc-700 px-4 py-2 text-sm hover:bg-zinc-900">
            Open gallery
          </button>
          <button
            onClick={onValidate}
            disabled={validating}
            className={[
              "rounded-xl border border-zinc-700 px-4 py-2 text-sm",
              validating ? "opacity-50 cursor-not-allowed" : "hover:bg-zinc-900"
            ].join(" ")}
          >
            {validating ? "Validating..." : "Validate config"}
          </button>
        </div>
      </div>

      {validationResult && (
        <div className={[
          "rounded-2xl border p-5",
          validationResult.valid
            ? "border-emerald-900/40 bg-emerald-950/20"
            : "border-red-900/40 bg-red-950/20"
        ].join(" ")}>
          <div className="flex items-center justify-between mb-3">
            <div className={[
              "text-sm font-semibold",
              validationResult.valid ? "text-emerald-200" : "text-red-200"
            ].join(" ")}>
              {validationResult.valid ? "✓ Configuration Valid" : "✗ Configuration Invalid"}
            </div>
            <button
              onClick={onClearValidation}
              className="text-xs text-zinc-400 hover:text-zinc-200"
            >
              Dismiss
            </button>
          </div>

          {validationResult.errors.length > 0 && (
            <div className="space-y-2 mb-3">
              {validationResult.errors.map((err, idx) => (
                <div key={idx} className="rounded-xl border border-red-900/30 bg-red-950/30 p-3">
                  <div className="text-sm text-red-200">
                    <span className="font-mono text-xs text-red-300">{err.field}</span>: {err.message}
                  </div>
                  {err.suggestion && (
                    <div className="mt-1 text-xs text-red-300/70">
                      Suggestion: {err.suggestion}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {validationResult.warnings.length > 0 && (
            <div className="space-y-2">
              {validationResult.warnings.map((warn, idx) => (
                <div key={idx} className="rounded-xl border border-yellow-900/30 bg-yellow-950/20 p-3 text-sm text-yellow-200">
                  ⚠ {warn}
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function Card({ title, value, sub }: { title: string; value: string; sub: string }) {
  return (
    <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-5">
      <div className="text-xs text-zinc-400">{title}</div>
      <div className="mt-2 text-2xl font-semibold tracking-tight">{value}</div>
      <div className="mt-2 text-xs text-zinc-400">{sub}</div>
    </div>
  );
}

function Gallery({ images }: { images: { name: string; url: string; created_ms: number }[] }) {
  return (
    <div className="grid gap-4">
      {images.length === 0 ? (
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-5 text-sm text-zinc-400">
          No images yet. Run a generation to populate the gallery.
        </div>
      ) : (
        <div className="grid grid-cols-3 gap-4">
          {images.map((img) => (
            <div key={img.name} className="overflow-hidden rounded-2xl border border-zinc-800 bg-zinc-900/20">
              <img src={img.url} alt={img.name} className="aspect-square w-full object-cover" />
              <div className="p-2 text-xs text-zinc-400 truncate">{img.name}</div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
