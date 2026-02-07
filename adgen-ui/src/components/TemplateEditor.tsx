import { useEffect, useMemo, useState } from "react";
import Editor from "@monaco-editor/react";
import yaml from "js-yaml";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";

import { type Template, TemplateSchema } from "../lib/schema";
import { getTemplate, saveTemplate } from "../lib/api";

export function TemplateEditor() {
  const [mode, setMode] = useState<"form" | "yaml">("form");
  const [raw, setRaw] = useState<string>("");
  const [newStyle, setNewStyle] = useState<string>("");
  const [saving, setSaving] = useState(false);
  const [status, setStatus] = useState<{ type: "ok" | "err"; msg: string } | null>(null);

  const form = useForm<Template>({
    resolver: zodResolver(TemplateSchema),
    defaultValues: { brand: "", product: "", styles: [] },
    mode: "onChange",
  });

  const { register, reset, watch, setValue, formState } = form;
  const values = watch();

  useEffect(() => {
    let active = true;
    getTemplate()
      .then((tpl) => {
        if (!active) return;
        reset(tpl);
        setRaw(yaml.dump(tpl));
      })
      .catch((err: any) => {
        if (!active) return;
        setStatus({
          type: "err",
          msg: err?.message ?? "Failed to load template",
        });
      });

    return () => {
      active = false;
    };
  }, [reset]);

  // Keep raw YAML in sync when user is in YAML mode (nice UX)
  useEffect(() => {
    if (mode === "yaml") {
      try {
        setRaw(yaml.dump(values));
      } catch {
        // ignore dump errors
      }
    }
  }, [mode, values]);

  const styles = values.styles ?? [];

  const canAddStyle = useMemo(() => {
    const s = newStyle.trim();
    if (!s) return false;
    return !styles.some((x) => x.toLowerCase() === s.toLowerCase());
  }, [newStyle, styles]);

  function addStyle() {
    const s = newStyle.trim();
    if (!s) return;
    if (styles.some((x) => x.toLowerCase() === s.toLowerCase())) return;
    setValue("styles", [...styles, s], { shouldValidate: true, shouldDirty: true });
    setNewStyle("");
  }

  function removeStyle(style: string) {
    setValue(
      "styles",
      styles.filter((s) => s !== style),
      { shouldValidate: true, shouldDirty: true }
    );
  }

  async function onSaveForm() {
    setSaving(true);
    setStatus(null);
    try {
      const parsed = TemplateSchema.parse(values);
      await saveTemplate(parsed);
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
      const parsed = TemplateSchema.parse(obj);
      await saveTemplate(parsed);
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
        <button onClick={() => setMode("form")} className={tab(mode === "form")}>
          Form
        </button>
        <button onClick={() => setMode("yaml")} className={tab(mode === "yaml")}>
          Raw YAML
        </button>

        <div className="ml-auto text-xs text-zinc-400">
          {formState.isValid ? "Valid ✅" : "Invalid ❌"}
          {formState.isDirty ? " • Unsaved" : ""}
        </div>
      </div>

      {mode === "form" ? (
        <div className="grid gap-4">
          <Section title="Template">
            <Field label="brand">
              <input {...register("brand")} className={input()} placeholder="e.g., Nike" />
            </Field>
            <Field label="product">
              <input {...register("product")} className={input()} placeholder="e.g., Air Max 90" />
            </Field>
          </Section>

          <Section title="Styles">
            <div className="flex gap-2">
              <input
                value={newStyle}
                onChange={(e) => setNewStyle(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    addStyle();
                  }
                }}
                className={input()}
                placeholder="Add a style (press Enter)…"
              />
              <button
                onClick={addStyle}
                disabled={!canAddStyle}
                className={secondary(!canAddStyle)}
              >
                Add
              </button>
            </div>

            {styles.length === 0 ? (
              <div className="text-sm text-zinc-400">
                No styles yet. Add at least one.
              </div>
            ) : (
              <div className="flex flex-wrap gap-2">
                {styles.map((s) => (
                  <div
                    key={s}
                    className="flex items-center gap-2 rounded-full border border-zinc-800 bg-zinc-950/40 px-3 py-1 text-sm"
                  >
                    <span className="text-zinc-200">{s}</span>
                    <button
                      onClick={() => removeStyle(s)}
                      className="text-zinc-400 hover:text-white"
                      aria-label={`Remove ${s}`}
                      title="Remove"
                    >
                      ×
                    </button>
                  </div>
                ))}
              </div>
            )}
          </Section>

          <div className="flex items-center gap-3">
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
        <div className="overflow-hidden rounded-2xl border border-zinc-800">
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
    active ? "bg-zinc-900 border-zinc-700 text-white" : "border-zinc-800 text-zinc-300 hover:bg-zinc-900/60",
  ].join(" ");
}
function input() {
  return "w-full rounded-xl border border-zinc-800 bg-zinc-950/40 px-3 py-2 text-sm outline-none focus:border-zinc-600";
}
function primary(disabled: boolean) {
  return [
    "rounded-xl px-4 py-2 text-sm font-semibold",
    disabled ? "bg-zinc-700 text-zinc-300" : "bg-white text-zinc-950 hover:bg-zinc-200",
  ].join(" ");
}
function secondary(disabled: boolean) {
  return [
    "rounded-xl px-4 py-2 text-sm font-semibold border border-zinc-800",
    disabled ? "text-zinc-500" : "text-zinc-200 hover:bg-zinc-900/50",
  ].join(" ");
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="rounded-2xl border border-zinc-800 bg-zinc-900/20 p-4">
      <div className="mb-3 text-sm font-semibold">{title}</div>
      <div className="grid gap-3">{children}</div>
    </div>
  );
}
function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="grid gap-1 text-xs text-zinc-400">
      <span>{label}</span>
      <div className="text-sm text-zinc-100">{children}</div>
    </label>
  );
}
