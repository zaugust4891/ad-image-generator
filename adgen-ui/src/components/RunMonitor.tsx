import { useEffect, useMemo, useRef, useState } from "react";
import { API_BASE_URL as BASE } from "../lib/config";

type RunState = "idle" | "running" | "finished" | "failed";
type ConnectionState = "connecting" | "connected" | "disconnected" | "reconnecting";

type RunEvent =
  | { type: "started"; run_id: string; total: number }
  | { type: "log"; run_id: string; msg: string }
  | { type: "progress"; run_id: string; done: number; total: number; cost_so_far: number }
  | { type: "finished"; run_id: string }
  | { type: "failed"; run_id: string; error: string };

const MAX_RECONNECT_ATTEMPTS = 5;
const BASE_DELAY_MS = 1000;
const MAX_DELAY_MS = 30000;

function getBackoffDelay(attempt: number): number {
  return Math.min(BASE_DELAY_MS * Math.pow(2, attempt), MAX_DELAY_MS);
}

export function RunMonitor({
  runId,
  onStartRun,
  onImageAdded,
}: {
  runId: string | null;
  onStartRun: () => Promise<void>;
  onImageAdded?: () => void;
}) {
  const [state, setState] = useState<RunState>("idle");
  const [connectionState, setConnectionState] = useState<ConnectionState>("disconnected");
  const [done, setDone] = useState(0);
  const [total, setTotal] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const [costSoFar, setCostSoFar] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const logRef = useRef<HTMLDivElement | null>(null);
  const reconnectAttemptRef = useRef(0);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const esRef = useRef<EventSource | null>(null);

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

    let isMounted = true;

    function connect() {
      if (!isMounted || !runId) return;

      setConnectionState(reconnectAttemptRef.current > 0 ? "reconnecting" : "connecting");

      const es = new EventSource(`${BASE}/api/run/${runId}/events`);
      esRef.current = es;

      es.onopen = () => {
        if (!isMounted) return;
        setConnectionState("connected");
        reconnectAttemptRef.current = 0;
        setLogs((prev) => [...prev, `Connected to run ${runId}`]);
      };

      es.addEventListener("message", (msg) => {
        if (!isMounted) return;
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
              setCostSoFar(evt.cost_so_far);
              onImageAdded?.();
              break;

            case "finished":
              setState("finished");
              setConnectionState("disconnected");
              setLogs((prev) => [...prev, "✅ Finished"]);
              es.close();
              break;

            case "failed":
              setState("failed");
              setConnectionState("disconnected");
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
        if (!isMounted) return;
        es.close();
        esRef.current = null;

        // Only attempt reconnection if the run is still in progress
        if (state !== "finished" && state !== "failed") {
          setConnectionState("disconnected");

          if (reconnectAttemptRef.current < MAX_RECONNECT_ATTEMPTS) {
            const delay = getBackoffDelay(reconnectAttemptRef.current);
            setLogs((prev) => [
              ...prev,
              `⚠️ Connection lost. Reconnecting in ${(delay / 1000).toFixed(1)}s (attempt ${reconnectAttemptRef.current + 1}/${MAX_RECONNECT_ATTEMPTS})`,
            ]);

            reconnectTimeoutRef.current = setTimeout(() => {
              reconnectAttemptRef.current += 1;
              connect();
            }, delay);
          } else {
            setLogs((prev) => [
              ...prev,
              "❌ Max reconnection attempts reached. Please refresh the page or start a new run.",
            ]);
          }
        }
      };
    }

    setState("running");
    setError(null);
    reconnectAttemptRef.current = 0;
    connect();

    return () => {
      isMounted = false;
      esRef.current?.close();
      esRef.current = null;
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
        reconnectTimeoutRef.current = null;
      }
    };
  }, [runId, onImageAdded]);

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
        <ConnectionIndicator state={connectionState} />

        <div className="ml-auto text-xs text-zinc-400">
          {total ? `${done}/${total} (${pct}%)` : "No run yet"}
        </div>
      </div>

      <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-4">
        <div className="mb-2 flex items-center justify-between">
          <div className="text-xs text-zinc-400">Progress</div>
          {costSoFar > 0 && (
            <div className="text-xs text-zinc-400">
              Cost: <span className="text-zinc-200">${costSoFar.toFixed(4)}</span>
            </div>
          )}
        </div>
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

function StatusPill({ state }: { state: RunState }) {
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

function ConnectionIndicator({ state }: { state: ConnectionState }) {
  const dotColor =
    state === "connected"
      ? "bg-emerald-500"
      : state === "connecting" || state === "reconnecting"
      ? "bg-yellow-500"
      : "bg-zinc-500";

  const animate = state === "connecting" || state === "reconnecting" ? "animate-pulse" : "";

  const label =
    state === "connected"
      ? "Connected"
      : state === "connecting"
      ? "Connecting..."
      : state === "reconnecting"
      ? "Reconnecting..."
      : "Disconnected";

  return (
    <div className="flex items-center gap-1.5 text-xs text-zinc-400">
      <div className={`h-2 w-2 rounded-full ${dotColor} ${animate}`} />
      {label}
    </div>
  );
}
