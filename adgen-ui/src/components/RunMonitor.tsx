import { useEffect, useMemo, useRef, useState } from "react";

type RunState = "idle" | "running" | "finished" | "failed";

type RunEvent =
  | { type: "started"; run_id: string; total: number }
  | { type: "log"; run_id: string; msg: string }
  | { type: "progress"; run_id: string; done: number; total: number }
  | { type: "finished"; run_id: string }
  | { type: "failed"; run_id: string; error: string };

const BASE = "http://127.0.0.1:8787";

export function RunMonitor({
  runId,
  onStartRun,
}: {
  runId: string | null;
  onStartRun: () => Promise<void>;
}) {
  const [state, setState] = useState<RunState>("idle");
  const [done, setDone] = useState(0);
  const [total, setTotal] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const logRef = useRef<HTMLDivElement | null>(null);

  const pct = useMemo(() => {
    if (!total) return 0;
    return Math.round((done / total) * 100);
  }, [done, total]);

  // Auto-scroll logs when new lines arrive
  useEffect(() => {
    const el = logRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [logs]);

  // Connect to SSE when runId changes
  useEffect(() => {
    if (!runId) return;

    setState("running");
    setLogs((prev) => [...prev, `Connected to run ${runId}`]);
    setError(null);

    const es = new EventSource(`${BASE}/api/run/${runId}/events`);

    es.addEventListener("message", (msg) => {
      try {
        const evt = JSON.parse((msg as MessageEvent).data) as RunEvent;

        switch (evt.type) {
          case "started":
            setTotal(evt.total);
            setDone(0);
            setState("running");
            setLogs((prev) => [...prev, `Run started: total=${evt.total}`]);
            break;

          case "log":
            setLogs((prev) => [...prev, evt.msg]);
            break;

          case "progress":
            setDone(evt.done);
            setTotal(evt.total);
            break;

          case "finished":
            setState("finished");
            setLogs((prev) => [...prev, "✅ Finished"]);
            es.close();
            break;

          case "failed":
            setState("failed");
            setError(evt.error);
            setLogs((prev) => [...prev, `❌ Failed: ${evt.error}`]);
            es.close();
            break;
        }
      } catch (e) {
        setLogs((prev) => [...prev, `⚠️ Bad event payload: ${(e as Error).message}`]);
      }
    });

    es.onerror = () => {
      // This fires on disconnects; not always fatal, but we should surface it.
      setLogs((prev) => [...prev, "⚠️ SSE connection error/disconnected"]);
      // Do not immediately set failed, because EventSource sometimes reconnects.
    };

    return () => {
      es.close();
    };
  }, [runId]);

  return (
    <div className="grid gap-4">
      <div className="flex items-center gap-3">
        <button
          onClick={onStartRun}
          className="rounded-xl bg-white px-4 py-2 text-sm font-semibold text-zinc-950 hover:bg-zinc-200"
        >
          Start Run
        </button>

        <StatusPill state={state} />

        <div className="ml-auto text-xs text-zinc-400">
          {total ? `${done}/${total} (${pct}%)` : "No run yet"}
        </div>
      </div>

      <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-4">
        <div className="mb-2 text-xs text-zinc-400">Progress</div>
        <div className="h-3 w-full overflow-hidden rounded-full bg-zinc-800">
          <div className="h-full bg-zinc-200" style={{ width: `${pct}%` }} />
        </div>
      </div>

      <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-4">
        <div className="mb-2 text-xs text-zinc-400">Live Logs</div>

        <div
          ref={logRef}
          className="h-[420px] overflow-auto rounded-xl border border-zinc-800 bg-zinc-950/40 p-3 font-mono text-xs text-zinc-200"
        >
          {logs.length === 0 ? (
            <div className="text-zinc-500">No logs yet.</div>
          ) : (
            logs.map((line, idx) => (
              <div key={idx} className="whitespace-pre-wrap">
                {line}
              </div>
            ))
          )}
        </div>

        {error && (
          <div className="mt-3 rounded-xl border border-red-900/40 bg-red-950/30 p-3 text-sm text-red-200">
            {error}
          </div>
        )}
      </div>
    </div>
  );
}

function StatusPill({ state }: { state: "idle" | "running" | "finished" | "failed" }) {
  const cls =
    state === "running"
      ? "border-zinc-700 text-zinc-200"
      : state === "finished"
      ? "border-emerald-900/40 text-emerald-200"
      : state === "failed"
      ? "border-red-900/40 text-red-200"
      : "border-zinc-800 text-zinc-400";

  const label =
    state === "running" ? "Running" : state === "finished" ? "Finished" : state === "failed" ? "Failed" : "Idle";

  return <div className={`rounded-full border px-3 py-1 text-xs ${cls}`}>{label}</div>;
}
