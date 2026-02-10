import { useEffect, useState } from "react";
import { getCostSummary, getCostEstimate, getConfig } from "../lib/api";
import type { CostSummary } from "../lib/api";

function usd(n: number): string {
  return `$${n.toFixed(4)}`;
}

export function CostDashboard() {
  const [summary, setSummary] = useState<CostSummary | null>(null);
  const [estimate, setEstimate] = useState<number | null>(null);
  const [budget, setBudget] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function load() {
      setLoading(true);
      setError(null);
      try {
        const [s, cfg] = await Promise.all([getCostSummary(), getConfig()]);
        setSummary(s);
        setBudget(cfg.budget_limit_usd ?? null);

        const price = cfg.provider.price_usd_per_image ?? 0;
        if (price > 0) {
          const est = await getCostEstimate(cfg.orchestrator.target_images, price);
          setEstimate(est.estimated_cost);
        } else {
          setEstimate(null);
        }
      } catch (err: any) {
        setError(err?.message ?? "Failed to load cost data");
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  if (loading) {
    return (
      <div className="text-sm text-zinc-400">Loading cost data...</div>
    );
  }

  if (error) {
    return (
      <div className="rounded-2xl border border-red-900/40 bg-red-950/30 p-5 text-sm text-red-300">
        {error}
      </div>
    );
  }

  if (!summary) return null;

  const lastRun = summary.runs.length > 0 ? summary.runs[0] : null;
  const budgetPct = budget && budget > 0 ? (summary.total_cost / budget) * 100 : null;

  return (
    <div className="grid gap-4">
      {/* Summary cards */}
      <div className="grid grid-cols-4 gap-4">
        <Card title="Total Spend" value={usd(summary.total_cost)} sub={`${summary.image_count} images generated`} />
        <Card title="Avg / Image" value={summary.image_count > 0 ? usd(summary.avg_cost_per_image) : "—"} sub="Average cost per image" />
        <Card
          title="Next Run Est."
          value={estimate !== null ? usd(estimate) : "—"}
          sub={estimate !== null ? "Based on current config" : "Set price_usd_per_image"}
        />
        <Card
          title="Last Run"
          value={lastRun ? usd(lastRun.cost) : "—"}
          sub={lastRun ? `${lastRun.image_count} images` : "No runs yet"}
        />
      </div>

      {/* Budget bar */}
      {budget !== null && budget > 0 && (
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-5">
          <div className="mb-3 flex items-center justify-between">
            <div className="text-sm font-semibold text-zinc-200">Budget</div>
            <div className="text-xs text-zinc-400">
              {usd(summary.total_cost)} / {usd(budget)}
            </div>
          </div>
          <div className="h-3 w-full overflow-hidden rounded-full bg-zinc-800">
            <div
              className={[
                "h-full rounded-full transition-all",
                budgetPct! > 100
                  ? "bg-red-500"
                  : budgetPct! > 80
                    ? "bg-yellow-500"
                    : "bg-emerald-500",
              ].join(" ")}
              style={{ width: `${Math.min(budgetPct!, 100)}%` }}
            />
          </div>
          {budgetPct! > 100 && (
            <div className="mt-2 text-xs text-red-400">
              Budget exceeded by {usd(summary.total_cost - budget)}
            </div>
          )}
          {budgetPct! > 80 && budgetPct! <= 100 && (
            <div className="mt-2 text-xs text-yellow-400">
              Approaching budget limit ({budgetPct!.toFixed(0)}% used)
            </div>
          )}
        </div>
      )}

      {/* Estimate + budget warning for next run */}
      {estimate !== null && budget !== null && budget > 0 && (
        <EstimateWarning estimate={estimate} totalSoFar={summary.total_cost} budget={budget} />
      )}

      {/* Per-run breakdown */}
      {summary.runs.length > 0 && (
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-5">
          <div className="mb-3 text-sm font-semibold text-zinc-200">Cost by Run</div>
          <div className="space-y-1">
            <div className="grid grid-cols-[1fr_80px_80px] gap-2 text-xs text-zinc-500 border-b border-zinc-800 pb-2">
              <div>Run ID</div>
              <div className="text-right">Images</div>
              <div className="text-right">Cost</div>
            </div>
            {summary.runs.map((run) => (
              <div key={run.run_id} className="grid grid-cols-[1fr_80px_80px] gap-2 text-sm py-1">
                <div className="truncate text-zinc-300 font-mono text-xs">{run.run_id}</div>
                <div className="text-right text-zinc-400">{run.image_count}</div>
                <div className="text-right text-zinc-200">{usd(run.cost)}</div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Per-provider breakdown */}
      {summary.by_provider.length > 0 && (
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-5">
          <div className="mb-3 text-sm font-semibold text-zinc-200">Cost by Provider</div>
          <div className="space-y-1">
            <div className="grid grid-cols-[1fr_1fr_80px_80px] gap-2 text-xs text-zinc-500 border-b border-zinc-800 pb-2">
              <div>Provider</div>
              <div>Model</div>
              <div className="text-right">Images</div>
              <div className="text-right">Cost</div>
            </div>
            {summary.by_provider.map((p) => (
              <div key={`${p.provider}-${p.model}`} className="grid grid-cols-[1fr_1fr_80px_80px] gap-2 text-sm py-1">
                <div className="text-zinc-300">{p.provider}</div>
                <div className="text-zinc-400 font-mono text-xs">{p.model}</div>
                <div className="text-right text-zinc-400">{p.image_count}</div>
                <div className="text-right text-zinc-200">{usd(p.cost)}</div>
              </div>
            ))}
          </div>
        </div>
      )}

      {summary.runs.length === 0 && (
        <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-5 text-sm text-zinc-400">
          No cost data yet. Run a generation to start tracking costs.
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

function EstimateWarning({
  estimate,
  totalSoFar,
  budget,
}: {
  estimate: number;
  totalSoFar: number;
  budget: number;
}) {
  const afterRun = totalSoFar + estimate;
  const pct = (afterRun / budget) * 100;

  if (pct <= 80) return null;

  const exceeds = afterRun > budget;

  return (
    <div
      className={[
        "rounded-2xl border p-5",
        exceeds
          ? "border-red-900/40 bg-red-950/20"
          : "border-yellow-900/40 bg-yellow-950/20",
      ].join(" ")}
    >
      <div className={`text-sm font-semibold ${exceeds ? "text-red-200" : "text-yellow-200"}`}>
        {exceeds
          ? "Next run would exceed budget"
          : "Next run will approach budget limit"}
      </div>
      <div className={`mt-1 text-xs ${exceeds ? "text-red-300/70" : "text-yellow-300/70"}`}>
        Current: {usd(totalSoFar)} + Estimated: {usd(estimate)} = {usd(afterRun)} / {usd(budget)} ({pct.toFixed(0)}%)
      </div>
    </div>
  );
}
