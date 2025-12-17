import { useEffect, useMemo, useState } from "react";
import { listImages, startRun } from "./lib/api";

type Nav = "dashboard" | "config" | "template" | "run" | "gallery";

export default function App() {
  const [nav, setNav] = useState<Nav>("dashboard");
  const [runId, setRunId] = useState<string | null>(null);

  const title = useMemo(() => {
    if (nav === "dashboard") return "Dashboard";
    if (nav === "config") return "Run Config";
    if (nav === "template") return "Template";
    if (nav === "run") return "Run Monitor";
    return "Gallery";
  }, [nav]);

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100">
      <div className="mx-auto grid max-w-7xl grid-cols-[260px_1fr] gap-6 p-6">
        <Sidebar nav={nav} setNav={setNav} />

        <main className="rounded-2xl border border-zinc-800 bg-zinc-950/40 shadow-[0_0_0_1px_rgba(255,255,255,0.04)] backdrop-blur">
          <Topbar title={title} onRun={async () => {
            const { run_id } = await startRun();
            setRunId(run_id);
            setNav("run");
          }} />

          <div className="p-6">
            {nav === "dashboard" && <Dashboard onOpenGallery={() => setNav("gallery")} />}
            {nav === "config" && <ConfigEditor />}
            {nav === "template" && <TemplateEditor />}
            {nav === "run" && <RunMonitor runId={runId} />}
            {nav === "gallery" && <Gallery />}
          </div>
        </main>
      </div>
    </div>
  );
}

function Sidebar({ nav, setNav }: { nav: Nav; setNav: (n: Nav) => void }) {
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
    <aside className="rounded-2xl border border-zinc-800 bg-zinc-950/40 p-4 backdrop-blur">
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
      </div>
      <div className="mt-6 rounded-xl border border-zinc-800 bg-zinc-900/30 p-3 text-xs text-zinc-400">
        Tip: keep concurrency + rate balanced to avoid provider throttling.
      </div>
    </aside>
  );
}

function Topbar({ title, onRun }: { title: string; onRun: () => void }) {
  return (
    <div className="flex items-center justify-between border-b border-zinc-800 px-6 py-4">
      <div className="text-base font-semibold tracking-tight">{title}</div>
      <button
        onClick={onRun}
        className="rounded-xl bg-white px-4 py-2 text-sm font-semibold text-zinc-950 hover:bg-zinc-200"
      >
        Run generation
      </button>
    </div>
  );
}

function Dashboard({ onOpenGallery }: { onOpenGallery: () => void }) {
  return (
    <div className="grid gap-4">
      <div className="grid grid-cols-3 gap-4">
        <Card title="Runs" value="Ready" sub="Start a run to see live progress." />
        <Card title="Output" value="out/" sub="Gallery updates as images land." />
        <Card title="Health" value="Local API" sub="localhost:8787 expected." />
      </div>

      <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-5">
        <div className="mb-2 text-sm font-semibold">Quick actions</div>
        <div className="flex gap-3">
          <button onClick={onOpenGallery} className="rounded-xl border border-zinc-700 px-4 py-2 text-sm hover:bg-zinc-900">
            Open gallery
          </button>
          <button className="rounded-xl border border-zinc-700 px-4 py-2 text-sm hover:bg-zinc-900">
            Validate config
          </button>
        </div>
      </div>
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

// Stubs (I’ll fill these with the exact config/template fields + validation next)
function ConfigEditor() {
  return <div className="text-sm text-zinc-300">Config editor UI goes here (form + raw YAML tabs).</div>;
}
function TemplateEditor() {
  return <div className="text-sm text-zinc-300">Template editor UI goes here (brand/product/styles chips).</div>;
}
function RunMonitor({ runId }: { runId: string | null }) {
  return <div className="text-sm text-zinc-300">Run monitor for {runId ?? "(none yet)"} (SSE logs + progress + thumbnails).</div>;
}
function Gallery() {
  const [imgs, setImgs] = useState<{ name: string; url: string }[]>([]);
  useEffect(() => {
    listImages().then(setImgs).catch(() => setImgs([]));
  }, []);
  return (
    <div>
      <div className="mb-4 text-sm text-zinc-400">Latest outputs</div>
      <div className="grid grid-cols-4 gap-3">
        {imgs.map((img) => (
          <a key={img.name} href={img.url} target="_blank" rel="noreferrer" className="group overflow-hidden rounded-xl border border-zinc-800">
            <img src={img.url} className="aspect-square w-full object-cover transition group-hover:scale-[1.02]" />
          </a>
        ))}
      </div>
    </div>
  );
}
