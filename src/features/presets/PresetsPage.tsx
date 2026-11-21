import React, { useState } from "react";
import {
  Copy,
  Trash2,
  Upload,
  Download,
  Play,
  SlidersHorizontal,
} from "lucide-react";
import { useAppStore } from "../../stores/useAppStore";
import { tauriApi } from "../../services/tauri";
import type { Preset } from "../../types/scrdeck";

export const PresetsPage: React.FC = () => {
  const { presets, applyPreset, savePreset, deletePreset, setActivePage } =
    useAppStore();

  const [importJson, setImportJson] = useState("");
  const [showImport, setShowImport] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);
  const [exportedJson, setExportedJson] = useState<string | null>(null);

  const handleDuplicate = async (preset: Preset) => {
    await savePreset({
      ...preset,
      id: "",
      name: `${preset.name} (Copy)`,
      isBuiltin: false,
    });
  };

  const handleImport = async (e: React.FormEvent) => {
    e.preventDefault();
    setImportError(null);
    try {
      await tauriApi.importPresetJson(importJson);
      const updated = await tauriApi.getPresets();
      useAppStore.setState({ presets: updated });
      setImportJson("");
      setShowImport(false);
    } catch (err) {
      setImportError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleExport = async (presetId: string) => {
    const json = await tauriApi.exportPresetJson(presetId);
    setExportedJson(json);
  };

  return (
    <div className="mx-auto max-w-5xl space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold text-text-primary">Presets</h2>
          <p className="text-xs text-text-secondary">
            Built-in profiles are read-only templates. Duplicate or save your
            own presets as portable JSON.
          </p>
        </div>
        <button
          type="button"
          onClick={() => setShowImport(!showImport)}
          className="inline-flex items-center gap-2 rounded-lg border border-border-default bg-elevated px-3.5 py-2 text-xs font-medium text-text-primary hover:bg-subtle"
        >
          <Upload size={14} />
          Import Preset JSON
        </button>
      </div>

      {showImport && (
        <form
          onSubmit={handleImport}
          className="rounded-xl border border-border-default bg-surface p-4 space-y-3"
        >
          <div className="text-xs font-semibold text-text-primary">
            Paste Preset JSON
          </div>
          <textarea
            rows={5}
            value={importJson}
            onChange={(e) => setImportJson(e.target.value)}
            placeholder='{"schemaVersion": 1, "name": "My Preset", ...}'
            className="w-full rounded-lg border border-border-default bg-canvas p-3 font-mono text-xs text-text-primary"
          />
          {importError && (
            <p className="text-xs text-status-error">{importError}</p>
          )}
          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={() => setShowImport(false)}
              className="rounded-lg border border-border-default px-3 py-1.5 text-xs text-text-secondary"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="rounded-lg bg-accent-lime px-4 py-1.5 text-xs font-semibold text-accent-fg"
            >
              Validate &amp; Import
            </button>
          </div>
        </form>
      )}

      {exportedJson && (
        <div className="rounded-xl border border-border-default bg-surface p-4 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-text-primary">
              Exported Preset JSON
            </span>
            <button
              type="button"
              onClick={() => setExportedJson(null)}
              className="text-xs text-text-secondary hover:text-text-primary"
            >
              Close
            </button>
          </div>
          <pre className="max-h-48 overflow-auto rounded-lg border border-border-default bg-canvas p-3 font-mono text-xs text-text-primary">
            {exportedJson}
          </pre>
        </div>
      )}

      <div className="grid gap-3 md:grid-cols-2">
        {presets.map((preset) => (
          <div
            key={preset.id}
            className="flex flex-col justify-between rounded-xl border border-border-default bg-surface p-5 space-y-4"
          >
            <div className="space-y-1.5">
              <div className="flex items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <SlidersHorizontal size={15} className="text-accent-lime" />
                  <h3 className="text-sm font-semibold text-text-primary">
                    {preset.name}
                  </h3>
                </div>
                <span
                  className={`rounded-full px-2 py-0.5 text-[11px] font-medium ${
                    preset.isBuiltin
                      ? "bg-subtle text-text-secondary"
                      : "bg-accent-lime/15 text-accent-lime"
                  }`}
                >
                  {preset.isBuiltin ? "Built-in" : "User"}
                </span>
              </div>
              <p className="text-xs text-text-secondary">
                {preset.description}
              </p>
              <div className="flex flex-wrap gap-2 pt-1 font-mono text-[11px] text-text-muted">
                <span>
                  {preset.settings.maxSize
                    ? `${preset.settings.maxSize}px`
                    : "Native"}
                </span>
                <span>·</span>
                <span>{preset.settings.maxFps ?? "Default"} FPS</span>
                <span>·</span>
                <span>{preset.settings.audioEnabled ? "Audio On" : "Muted"}</span>
              </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-2 border-t border-border-default pt-3">
              <div className="flex gap-1.5">
                <button
                  type="button"
                  onClick={() => void handleDuplicate(preset)}
                  title="Duplicate preset"
                  className="inline-flex items-center gap-1 rounded-lg border border-border-default bg-canvas px-2.5 py-1.5 text-xs text-text-secondary hover:text-text-primary"
                >
                  <Copy size={13} />
                  Duplicate
                </button>
                <button
                  type="button"
                  onClick={() => void handleExport(preset.id)}
                  title="Export JSON"
                  className="inline-flex items-center gap-1 rounded-lg border border-border-default bg-canvas px-2.5 py-1.5 text-xs text-text-secondary hover:text-text-primary"
                >
                  <Download size={13} />
                  Export
                </button>
                {!preset.isBuiltin && (
                  <>
                    {confirmDeleteId === preset.id ? (
                      <button
                        type="button"
                        onClick={() => {
                          void deletePreset(preset.id);
                          setConfirmDeleteId(null);
                        }}
                        className="rounded-lg bg-status-error px-2.5 py-1.5 text-xs font-medium text-white"
                      >
                        Confirm Delete
                      </button>
                    ) : (
                      <button
                        type="button"
                        onClick={() => setConfirmDeleteId(preset.id)}
                        title="Delete preset"
                        className="rounded-lg border border-border-default bg-canvas p-1.5 text-text-secondary hover:text-status-error"
                      >
                        <Trash2 size={13} />
                      </button>
                    )}
                  </>
                )}
              </div>

              <button
                type="button"
                onClick={() => {
                  applyPreset(preset);
                  setActivePage("mirroring");
                }}
                className="inline-flex items-center gap-1.5 rounded-lg bg-accent-lime px-3 py-1.5 text-xs font-semibold text-accent-fg hover:bg-accent-lime-hover"
              >
                <Play size={13} />
                Use Preset
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
