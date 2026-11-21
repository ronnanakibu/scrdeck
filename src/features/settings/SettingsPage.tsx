import React from "react";
import { FolderOpen, RefreshCw, Download } from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "../../stores/useAppStore";
import type { AppClosePolicy, ThemeMode } from "../../types/scrdeck";

export const SettingsPage: React.FC = () => {
  const {
    settings,
    environment,
    installingTools,
    installError,
    updateAppSettings,
    refreshToolchain,
    installManagedTools,
  } = useAppStore();

  if (!settings) return null;

  const handleBrowseScrcpy = async () => {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: "Executable", extensions: ["exe"] }],
    });
    if (typeof selected === "string") {
      await updateAppSettings({ customScrcpyPath: selected });
      await refreshToolchain();
    }
  };

  const handleBrowseAdb = async () => {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: "Executable", extensions: ["exe"] }],
    });
    if (typeof selected === "string") {
      await updateAppSettings({ customAdbPath: selected });
      await refreshToolchain();
    }
  };

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-text-primary">Settings</h2>
        <p className="text-xs text-text-secondary">
          Configure appearance, toolchain paths, process lifecycle, and local
          privacy defaults.
        </p>
      </div>

      {/* Appearance */}
      <section className="rounded-xl border border-border-default bg-surface p-5 space-y-3">
        <h3 className="text-sm font-semibold text-text-primary">Appearance</h3>
        <div className="flex flex-wrap gap-2">
          {(["dark", "light", "system"] as ThemeMode[]).map((mode) => (
            <button
              key={mode}
              type="button"
              onClick={() => void updateAppSettings({ theme: mode })}
              className={`rounded-lg border px-4 py-2 text-xs font-medium capitalize ${
                settings.theme === mode
                  ? "border-accent-lime bg-elevated text-text-primary"
                  : "border-border-default bg-canvas text-text-secondary hover:text-text-primary"
              }`}
            >
              {mode}
            </button>
          ))}
        </div>
      </section>

      {/* Toolchain */}
      <section className="rounded-xl border border-border-default bg-surface p-5 space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-text-primary">Toolchain</h3>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => void refreshToolchain()}
              className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3 py-1.5 text-xs text-text-primary hover:bg-subtle"
            >
              <RefreshCw size={13} />
              Re-validate
            </button>
            <button
              type="button"
              disabled={installingTools}
              onClick={() => void installManagedTools()}
              className="inline-flex items-center gap-1.5 rounded-lg bg-accent-lime px-3 py-1.5 text-xs font-semibold text-accent-fg hover:bg-accent-lime-hover disabled:opacity-50"
            >
              <Download size={13} />
              {installingTools ? "Installing..." : "Install Official Bundle"}
            </button>
          </div>
        </div>

        {installError && (
          <div className="rounded-lg border border-status-error/40 bg-status-error/10 p-3 text-xs text-status-error">
            {installError}
          </div>
        )}

        <div className="space-y-3 text-xs">
          <div className="space-y-1">
            <span className="text-text-secondary">
              Custom scrcpy.exe path (optional)
            </span>
            <div className="flex gap-2">
              <input
                type="text"
                value={settings.customScrcpyPath ?? ""}
                placeholder={
                  environment?.toolchain.scrcpyPath || "Auto-detected / Managed"
                }
                onChange={(e) =>
                  void updateAppSettings({
                    customScrcpyPath: e.target.value || null,
                  })
                }
                className="flex-1 rounded-lg border border-border-default bg-canvas px-3 py-2 font-mono text-text-primary"
              />
              <button
                type="button"
                onClick={() => void handleBrowseScrcpy()}
                className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3 py-2 text-text-primary hover:bg-subtle"
              >
                <FolderOpen size={14} />
                Browse
              </button>
            </div>
          </div>

          <div className="space-y-1">
            <span className="text-text-secondary">
              Custom adb.exe path (optional)
            </span>
            <div className="flex gap-2">
              <input
                type="text"
                value={settings.customAdbPath ?? ""}
                placeholder={
                  environment?.toolchain.adbPath || "Auto-detected / Bundled"
                }
                onChange={(e) =>
                  void updateAppSettings({
                    customAdbPath: e.target.value || null,
                  })
                }
                className="flex-1 rounded-lg border border-border-default bg-canvas px-3 py-2 font-mono text-text-primary"
              />
              <button
                type="button"
                onClick={() => void handleBrowseAdb()}
                className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3 py-2 text-text-primary hover:bg-subtle"
              >
                <FolderOpen size={14} />
                Browse
              </button>
            </div>
          </div>
        </div>
      </section>

      {/* Behavior & Privacy */}
      <section className="rounded-xl border border-border-default bg-surface p-5 space-y-4">
        <h3 className="text-sm font-semibold text-text-primary">
          Behavior &amp; Privacy
        </h3>

        <label className="block space-y-1 text-xs">
          <span className="text-text-secondary">
            When ScrDeck closes while mirroring sessions are running
          </span>
          <select
            value={settings.appClosePolicy}
            onChange={(e) =>
              void updateAppSettings({
                appClosePolicy: e.target.value as AppClosePolicy,
              })
            }
            className="w-full max-w-md rounded-lg border border-border-default bg-canvas px-3 py-2 text-text-primary"
          >
            <option value="stopScrDeckSessions">
              Stop ScrDeck-launched scrcpy sessions (Default)
            </option>
            <option value="keepSessionsRunning">
              Leave running scrcpy windows open
            </option>
          </select>
        </label>

        <label className="inline-flex items-center gap-2 text-xs text-text-primary">
          <input
            type="checkbox"
            checked={settings.redactDiagnosticsByDefault}
            onChange={(e) =>
              void updateAppSettings({
                redactDiagnosticsByDefault: e.target.checked,
              })
            }
          />
          Redact device serials, IP addresses, and local user paths in exported
          diagnostic bundles by default
        </label>
      </section>
    </div>
  );
};
