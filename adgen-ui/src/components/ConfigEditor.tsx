import { useEffect, useState } from "react";
import { type RunConfig } from "../lib/schema";
import Editor from "@monaco-editor/react";
import { RunConfigSchema } from "../lib/schema";
import { getConfig, saveConfig } from "../lib/api";

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import yaml from "js-yaml";

export function ConfigEditor() {
  const [mode, setMode] = useState<"form" | "yaml">("form");
  const [raw, setRaw] = useState<string>("");
  const [saving, setSaving] = useState(false);
  const [status, setStatus] = useState<{ type: "ok" | "err"; msg: string } | null>(null);

  const form = useForm<RunConfig>({
    resolver: zodResolver(RunConfigSchema),
    defaultValues: undefined as any,
    mode: "onChange",
  });

  const { register, reset, watch, formState } = form;
  const values = watch();

  useEffect(() => {
    getConfig().then((cfg) => {
      // backend returns out_dir as PathBuf in Rust JSON (string), so OK
      reset(cfg as any);
      setRaw(yaml.dump(cfg));
    });
  }, [reset]);

  // keep YAML preview in sync (only when in YAML mode)
  useEffect(() => {
    if (mode === "yaml") {
      try { setRaw(yaml.dump(values)); } catch { /* ignore */ }
    }
  }, [mode, values]);

  async function onSaveForm() {
    setSaving(true);
    setStatus(null);
    try {
      const parsed = RunConfigSchema.parse(values);
      await saveConfig(parsed as any);
      reset(parsed);
      setStatus({ type: "ok", msg: "Saved" });
    } catch (err: any) {
      setStatus({ type: "err", msg: err?.message ?? "Save failed" });
    } finally {
      setSaving(false);
    }
  }

  async function onSaveYaml() {
    setSaving(true);
    setStatus(null);
    try {
      const obj = yaml.load(raw);
      const parsed = RunConfigSchema.parse(obj);
      await saveConfig(parsed as any);
      reset(parsed);
      setStatus({ type: "ok", msg: "Saved" });
    } catch (err: any) {
      setStatus({ type: "err", msg: err?.message ?? "Save failed" });
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="grid gap-4">
      <div className="flex items-center gap-2">
        <button onClick={() => setMode("form")} className={tab(mode === "form")}>Form</button>
        <button onClick={() => setMode("yaml")} className={tab(mode === "yaml")}>Raw YAML</button>
        <div className="ml-auto text-xs text-zinc-400">
          {formState.isValid ? "Valid ✅" : "Invalid ❌"}
        </div>
      </div>

      {mode === "form" ? (
        <div className="grid grid-cols-2 gap-4">
          <Section title="Provider">
            <Field label="kind">
              <select {...register("provider.kind")} className={input()}>
                <option value="mock">mock</option>
                <option value="openai">openai</option>
              </select>
            </Field>
            <Field label="model"><input {...register("provider.model")} className={input()} /></Field>
            <Field label="api_key_env"><input {...register("provider.api_key_env")} className={input()} /></Field>
            <Field label="width"><input type="number" {...register("provider.width", { valueAsNumber: true })} className={input()} /></Field>
            <Field label="height"><input type="number" {...register("provider.height", { valueAsNumber: true })} className={input()} /></Field>
            <Field label="price_usd_per_image"><input type="number" step="0.01" {...register("provider.price_usd_per_image", { valueAsNumber: true })} className={input()} /></Field>
          </Section>

          <Section title="Orchestrator">
            <Field label="target_images"><input type="number" {...register("orchestrator.target_images", { valueAsNumber: true })} className={input()} /></Field>
            <Field label="concurrency"><input type="number" {...register("orchestrator.concurrency", { valueAsNumber: true })} className={input()} /></Field>
            <Field label="queue_cap"><input type="number" {...register("orchestrator.queue_cap", { valueAsNumber: true })} className={input()} /></Field>
            <Field label="rate_per_min"><input type="number" {...register("orchestrator.rate_per_min", { valueAsNumber: true })} className={input()} /></Field>
            <Field label="backoff_base_ms"><input type="number" {...register("orchestrator.backoff_base_ms", { valueAsNumber: true })} className={input()} /></Field>
            <Field label="backoff_factor"><input type="number" step="0.1" {...register("orchestrator.backoff_factor", { valueAsNumber: true })} className={input()} /></Field>
            <Field label="backoff_jitter_ms"><input type="number" {...register("orchestrator.backoff_jitter_ms", { valueAsNumber: true })} className={input()} /></Field>
          </Section>

          <Section title="Dedupe">
            <Field label="enabled"><input type="checkbox" {...register("dedupe.enabled")} /></Field>
            <Field label="phash_bits"><input type="number" {...register("dedupe.phash_bits", { valueAsNumber: true })} className={input()} /></Field>
            <Field label="phash_thresh"><input type="number" {...register("dedupe.phash_thresh", { valueAsNumber: true })} className={input()} /></Field>
          </Section>

          <Section title="Post">
            <Field label="thumbnail"><input type="checkbox" {...register("post.thumbnail")} /></Field>
            <Field label="thumb_max"><input type="number" {...register("post.thumb_max", { valueAsNumber: true })} className={input()} /></Field>
          </Section>

          <Section title="Rewrite">
            <Field label="enabled"><input type="checkbox" {...register("rewrite.enabled")} /></Field>
            <Field label="model"><input {...register("rewrite.model")} className={input()} /></Field>
            <Field label="system"><input {...register("rewrite.system")} className={input()} /></Field>
            <Field label="max_tokens"><input type="number" {...register("rewrite.max_tokens", { valueAsNumber: true })} className={input()} /></Field>
          </Section>

          <Section title="Output">
            <Field label="out_dir"><input {...register("out_dir")} className={input()} /></Field>
            <Field label="seed"><input type="number" {...register("seed", { valueAsNumber: true })} className={input()} /></Field>
          </Section>

          <div className="col-span-2 flex items-center gap-3">
            <button onClick={onSaveForm} disabled={!formState.isValid || saving} className={primary(!formState.isValid || saving)}>
              {saving ? "Saving..." : "Save"}
            </button>
            {status && (
              <span className={`text-xs ${status.type === "ok" ? "text-green-400" : "text-red-400"}`}>
                {status.msg}
              </span>
            )}
          </div>
        </div>
      ) : (
        <div className="rounded-2xl border border-zinc-800 overflow-hidden">
          <Editor
            height="520px"
            defaultLanguage="yaml"
            value={raw}
            onChange={(v) => setRaw(v ?? "")}
            options={{ minimap: { enabled: false }, fontSize: 13 }}
          />
          <div className="flex items-center gap-3 border-t border-zinc-800 p-3">
            <button onClick={onSaveYaml} disabled={saving} className={primary(saving)}>
              {saving ? "Saving..." : "Validate + Save YAML"}
            </button>
            {status ? (
              <span className={`text-xs ${status.type === "ok" ? "text-green-400" : "text-red-400"}`}>
                {status.msg}
              </span>
            ) : (
              <div className="text-xs text-zinc-400">
                YAML is validated by Zod before saving.
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}


function tab(active: boolean) {
  return [
    "rounded-xl px-3 py-2 text-sm border transition",
    active ? "bg-zinc-900 border-zinc-700 text-white" : "border-zinc-800 text-zinc-300 hover:bg-zinc-900/60"
  ].join(" ");
}
function input() {
  return "w-full rounded-xl border border-zinc-800 bg-zinc-950/40 px-3 py-2 text-sm outline-none focus:border-zinc-600";
}
function primary(disabled: boolean) {
  return [
    "rounded-xl px-4 py-2 text-sm font-semibold",
    disabled ? "bg-zinc-700 text-zinc-300" : "bg-white text-zinc-950 hover:bg-zinc-200"
  ].join(" ");
}
function Section({ title, children }: any) {
  return (
    <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-4">
      <div className="mb-3 text-sm font-semibold">{title}</div>
      <div className="grid gap-3">{children}</div>
    </div>
  );
}
function Field({ label, children }: any) {
  return (
    <label className="grid gap-1 text-xs text-zinc-400">
      <span>{label}</span>
      <div className="text-sm text-zinc-100">{children}</div>
    </label>
  );
}
