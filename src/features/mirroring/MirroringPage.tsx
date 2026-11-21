import React, { useState } from "react";
import {
  Play,
  Square,
  Copy,
  Check,
  Save,
  RotateCcw,
  AlertTriangle,
  Plus,
  Trash2,
} from "lucide-react";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { useAppStore, DEFAULT_MIRROR_SETTINGS } from "../../stores/useAppStore";
import { tauriApi } from "../../services/tauri";

export const MirroringPage: React.FC = () => {
  const {
    environment,
    devices,
    selectedDeviceSerial,
    presets,
    activePresetId,
    mirrorSettings,
    allowUnknownFlags,
    commandPreview,
    sessions,
    applyPreset,
    updateMirrorSettings,
    setAllowUnknownFlags,
    startMirroring,
    stopMirroring,
    savePreset,
  } = useAppStore();

  const [mode, setMode] = useState<"basic" | "advanced">("basic");
  const [copiedCmd, setCopiedCmd] = useState(false);
  const [pasteArgsInput, setPasteArgsInput] = useState("");
  const [pasteError, setPasteError] = useState<string | null>(null);
  const [launchError, setLaunchError] = useState<string | null>(null);
  const [savePresetName, setSavePresetName] = useState("");
  const [showSaveModal, setShowSaveModal] = useState(false);

  const selectedDevice =
    devices.find((d) => d.serial === selectedDeviceSerial) || null;
  const activeSession = sessions.find(
    (s) =>
      s.deviceSerial === selectedDeviceSerial &&
      (s.status === "running" || s.status === "starting")
  );

  const caps = environment?.capabilities;

  const handleCopyCommand = async () => {
    if (!commandPreview) return;
    await navigator.clipboard.writeText(commandPreview.preview);
    setCopiedCmd(true);
    setTimeout(() => setCopiedCmd(false), 1500);
  };

  const handleStart = async () => {
    setLaunchError(null);
    try {
      await startMirroring();
    } catch (err) {
      setLaunchError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleParsePasteArgs = async () => {
    setPasteError(null);
    if (!pasteArgsInput.trim()) return;
    try {
      const parsed = await tauriApi.parseCustomArgsString(pasteArgsInput);
      updateMirrorSettings({
        customArgs: [...mirrorSettings.customArgs, ...parsed],
      });
      setPasteArgsInput("");
    } catch (err) {
      setPasteError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleBrowseRecordingPath = async () => {
    const path = await saveDialog({
      defaultPath: "scrdeck-recording.mp4",
      filters: [{ name: "Video", extensions: ["mp4", "mkv"] }],
    });
    if (typeof path === "string") {
      updateMirrorSettings({ recordingPath: path });
    }
  };

  const handleSaveAsPreset = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!savePresetName.trim()) return;
    await savePreset({
      schemaVersion: 1,
      id: "",
      name: savePresetName.trim(),
      description: "Custom user mirroring preset.",
      isBuiltin: false,
      createdAt: "",
      updatedAt: "",
      settings: { ...mirrorSettings },
      compatibilityNotes: [],
    });
    setSavePresetName("");
    setShowSaveModal(false);
  };

  return (
    <div className="mx-auto max-w-5xl space-y-6">
      {/* Top Action Bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border-default bg-surface p-4">
        <div className="flex flex-wrap items-center gap-3">
          <select
            aria-label="Active preset"
            value={activePresetId || ""}
            onChange={(e) => {
              const found = presets.find((p) => p.id === e.target.value);
              if (found) applyPreset(found);
            }}
            className="rounded-lg border border-border-default bg-canvas px-3 py-2 text-xs font-medium text-text-primary"
          >
            {presets.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name} {p.isBuiltin ? "(Built-in)" : ""}
              </option>
            ))}
          </select>

          <div className="inline-flex rounded-lg border border-border-default bg-canvas p-0.5">
            <button
              type="button"
              onClick={() => setMode("basic")}
              className={`rounded-md px-3 py-1 text-xs font-medium ${
                mode === "basic"
                  ? "bg-elevated text-text-primary"
                  : "text-text-secondary hover:text-text-primary"
              }`}
            >
              Basic Mode
            </button>
            <button
              type="button"
              onClick={() => setMode("advanced")}
              className={`rounded-md px-3 py-1 text-xs font-medium ${
                mode === "advanced"
                  ? "bg-elevated text-text-primary"
                  : "text-text-secondary hover:text-text-primary"
              }`}
            >
              Advanced Mode
            </button>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            onClick={() => updateMirrorSettings({ ...DEFAULT_MIRROR_SETTINGS })}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3 py-2 text-xs font-medium text-text-secondary hover:text-text-primary"
          >
            <RotateCcw size={14} />
            Reset
          </button>
          <button
            type="button"
            onClick={() => setShowSaveModal(true)}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3 py-2 text-xs font-medium text-text-primary hover:bg-subtle"
          >
            <Save size={14} />
            Save as Preset
          </button>
          {activeSession ? (
            <button
              type="button"
              onClick={() => void stopMirroring(activeSession.sessionId)}
              className="inline-flex items-center gap-2 rounded-lg bg-status-error px-4 py-2 text-xs font-semibold text-white hover:opacity-90"
            >
              <Square size={14} />
              Stop Session
            </button>
          ) : (
            <button
              type="button"
              disabled={
                !selectedDevice ||
                selectedDevice.adbState !== "device" ||
                !commandPreview?.isValid
              }
              onClick={() => void handleStart()}
              className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-4 py-2 text-xs font-semibold text-accent-fg hover:bg-accent-lime-hover disabled:opacity-50"
            >
              <Play size={14} />
              Start Mirroring
            </button>
          )}
        </div>
      </div>

      {launchError && (
        <div className="rounded-xl border border-status-error/40 bg-status-error/10 p-4 text-xs text-status-error">
          {launchError}
        </div>
      )}

      {/* Settings Grid */}
      <div className="grid gap-4 md:grid-cols-2">
        {/* Display */}
        <section className="rounded-xl border border-border-default bg-surface p-5 space-y-4">
          <h3 className="text-sm font-semibold text-text-primary">Display</h3>
          <div className="grid gap-3 sm:grid-cols-2">
            <label className="space-y-1 text-xs">
              <span className="text-text-secondary">Max Size (px)</span>
              <input
                type="number"
                placeholder="Native"
                value={mirrorSettings.maxSize ?? ""}
                onChange={(e) =>
                  updateMirrorSettings({
                    maxSize: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="w-full rounded-lg border border-border-default bg-canvas px-3 py-2 text-text-primary tabular-nums"
              />
            </label>
            <label className="space-y-1 text-xs">
              <span className="text-text-secondary">Max FPS</span>
              <input
                type="number"
                placeholder="Default"
                value={mirrorSettings.maxFps ?? ""}
                onChange={(e) =>
                  updateMirrorSettings({
                    maxFps: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="w-full rounded-lg border border-border-default bg-canvas px-3 py-2 text-text-primary tabular-nums"
              />
            </label>
          </div>

          <div className="flex flex-wrap gap-4 text-xs">
            <label className="inline-flex items-center gap-2 text-text-primary">
              <input
                type="checkbox"
                checked={mirrorSettings.fullscreen}
                onChange={(e) =>
                  updateMirrorSettings({ fullscreen: e.target.checked })
                }
              />
              Fullscreen
            </label>
            <label className="inline-flex items-center gap-2 text-text-primary">
              <input
                type="checkbox"
                checked={mirrorSettings.alwaysOnTop}
                onChange={(e) =>
                  updateMirrorSettings({ alwaysOnTop: e.target.checked })
                }
              />
              Always on top
            </label>
          </div>

          {mode === "advanced" && (
            <label className="block space-y-1 text-xs">
              <span className="text-text-secondary">Window Title</span>
              <input
                type="text"
                placeholder="Default device model"
                value={mirrorSettings.windowTitle ?? ""}
                onChange={(e) =>
                  updateMirrorSettings({
                    windowTitle: e.target.value || null,
                  })
                }
                className="w-full rounded-lg border border-border-default bg-canvas px-3 py-2 text-text-primary"
              />
            </label>
          )}
        </section>

        {/* Video & Audio */}
        <section className="rounded-xl border border-border-default bg-surface p-5 space-y-4">
          <h3 className="text-sm font-semibold text-text-primary">
            Video &amp; Audio
          </h3>
          <div className="grid gap-3 sm:grid-cols-2">
            <label className="space-y-1 text-xs">
              <span className="text-text-secondary">Video Codec</span>
              <select
                value={mirrorSettings.videoCodec ?? "h264"}
                onChange={(e) =>
                  updateMirrorSettings({ videoCodec: e.target.value })
                }
                className="w-full rounded-lg border border-border-default bg-canvas px-3 py-2 text-text-primary"
              >
                <option value="h264">H.264 (Default)</option>
                <option value="h265">H.265 (HEVC)</option>
                <option value="av1">AV1</option>
              </select>
            </label>
            <label className="space-y-1 text-xs">
              <span className="text-text-secondary">Video Bitrate</span>
              <input
                type="text"
                placeholder="e.g. 8M"
                value={mirrorSettings.videoBitRate ?? ""}
                onChange={(e) =>
                  updateMirrorSettings({
                    videoBitRate: e.target.value || null,
                  })
                }
                className="w-full rounded-lg border border-border-default bg-canvas px-3 py-2 text-text-primary font-mono"
              />
            </label>
          </div>

          <div className="space-y-1">
            <label className="inline-flex items-center gap-2 text-xs text-text-primary">
              <input
                type="checkbox"
                checked={mirrorSettings.audioEnabled}
                onChange={(e) =>
                  updateMirrorSettings({ audioEnabled: e.target.checked })
                }
              />
              Forward device audio
            </label>
            <p className="text-[11px] text-text-muted">
              Audio forwarding requires Android 11 (API 30) or newer.
            </p>
          </div>
        </section>

        {/* Control */}
        <section className="rounded-xl border border-border-default bg-surface p-5 space-y-3">
          <h3 className="text-sm font-semibold text-text-primary">Control</h3>
          <div className="grid gap-2.5 text-xs">
            <label className="inline-flex items-center gap-2 text-text-primary">
              <input
                type="checkbox"
                checked={mirrorSettings.controlEnabled}
                onChange={(e) =>
                  updateMirrorSettings({ controlEnabled: e.target.checked })
                }
              />
              Enable keyboard &amp; mouse control
            </label>
            <label className="inline-flex items-center gap-2 text-text-primary">
              <input
                type="checkbox"
                checked={mirrorSettings.stayAwake}
                onChange={(e) =>
                  updateMirrorSettings({ stayAwake: e.target.checked })
                }
              />
              Keep device awake while plugged in (--stay-awake)
            </label>
            <label className="inline-flex items-center gap-2 text-text-primary">
              <input
                type="checkbox"
                checked={mirrorSettings.turnScreenOff}
                onChange={(e) =>
                  updateMirrorSettings({ turnScreenOff: e.target.checked })
                }
              />
              Turn physical phone screen off (--turn-screen-off)
            </label>
            <label className="inline-flex items-center gap-2 text-text-primary">
              <input
                type="checkbox"
                checked={mirrorSettings.showTouches}
                onChange={(e) =>
                  updateMirrorSettings({ showTouches: e.target.checked })
                }
              />
              Show touches on device (--show-touches)
            </label>
          </div>
        </section>

        {/* Recording */}
        <section className="rounded-xl border border-border-default bg-surface p-5 space-y-3">
          <h3 className="text-sm font-semibold text-text-primary">Recording</h3>
          <label className="inline-flex items-center gap-2 text-xs text-text-primary">
            <input
              type="checkbox"
              checked={mirrorSettings.recordingEnabled}
              onChange={(e) =>
                updateMirrorSettings({ recordingEnabled: e.target.checked })
              }
            />
            Record session to file (--record)
          </label>
          {mirrorSettings.recordingEnabled && (
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="Select output .mp4 or .mkv path"
                value={mirrorSettings.recordingPath ?? ""}
                onChange={(e) =>
                  updateMirrorSettings({
                    recordingPath: e.target.value || null,
                  })
                }
                className="flex-1 rounded-lg border border-border-default bg-canvas px-3 py-2 text-xs font-mono text-text-primary"
              />
              <button
                type="button"
                onClick={() => void handleBrowseRecordingPath()}
                className="rounded-lg border border-border-default bg-elevated px-3 py-2 text-xs font-medium text-text-primary hover:bg-subtle"
              >
                Browse
              </button>
            </div>
          )}
        </section>
      </div>

      {/* Advanced Custom Arguments */}
      {mode === "advanced" && (
        <section className="rounded-xl border border-border-default bg-surface p-5 space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-semibold text-text-primary">
                Custom scrcpy Arguments
              </h3>
              <p className="text-xs text-text-secondary">
                Passed directly as argv entries without shell execution.
                {caps?.parsedSuccessfully
                  ? ` Validated against scrcpy ${caps.scrcpyVersion} (${caps.supportedFlags.length} flags).`
                  : ""}
              </p>
            </div>
          </div>

          <div className="flex gap-2">
            <input
              type="text"
              placeholder='Paste flags (e.g. --crop=1224:1440:0:0 --window-borderless)'
              value={pasteArgsInput}
              onChange={(e) => setPasteArgsInput(e.target.value)}
              className="flex-1 rounded-lg border border-border-default bg-canvas px-3 py-2 text-xs font-mono text-text-primary"
            />
            <button
              type="button"
              onClick={() => void handleParsePasteArgs()}
              className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3 py-2 text-xs font-medium text-text-primary hover:bg-subtle"
            >
              <Plus size={14} />
              Add Args
            </button>
          </div>
          {pasteError && (
            <p className="text-xs text-status-error">{pasteError}</p>
          )}

          {mirrorSettings.customArgs.length > 0 && (
            <div className="space-y-1.5">
              {mirrorSettings.customArgs.map((arg, idx) => (
                <div key={idx} className="flex items-center gap-2">
                  <input
                    type="text"
                    value={arg}
                    onChange={(e) => {
                      const next = [...mirrorSettings.customArgs];
                      next[idx] = e.target.value;
                      updateMirrorSettings({ customArgs: next });
                    }}
                    className="flex-1 rounded-lg border border-border-default bg-canvas px-3 py-1.5 text-xs font-mono text-text-primary"
                  />
                  <button
                    type="button"
                    aria-label="Remove argument"
                    onClick={() => {
                      const next = mirrorSettings.customArgs.filter(
                        (_, i) => i !== idx
                      );
                      updateMirrorSettings({ customArgs: next });
                    }}
                    className="rounded-lg border border-border-default p-1.5 text-text-secondary hover:text-status-error"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              ))}
            </div>
          )}
        </section>
      )}

      {/* Live Command Preview */}
      <section className="rounded-xl border border-border-default bg-surface p-5 space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-text-primary">
            Command Preview (Exact argv)
          </h3>
          <button
            type="button"
            onClick={() => void handleCopyCommand()}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-2.5 py-1 text-xs text-text-secondary hover:text-text-primary"
          >
            {copiedCmd ? (
              <Check size={13} className="text-status-success" />
            ) : (
              <Copy size={13} />
            )}
            {copiedCmd ? "Copied" : "Copy Command"}
          </button>
        </div>

        <pre className="overflow-x-auto rounded-lg border border-border-default bg-canvas p-3 font-mono text-xs text-text-primary">
          {commandPreview?.preview || "scrcpy.exe"}
        </pre>

        {commandPreview && commandPreview.conflicts.length > 0 && (
          <div className="space-y-1.5">
            {commandPreview.conflicts.map((c, i) => (
              <div
                key={i}
                className={`flex items-start gap-2 rounded-lg p-2.5 text-xs ${
                  c.severity === "error"
                    ? "bg-status-error/10 text-status-error"
                    : "bg-status-warning/10 text-status-warning"
                }`}
              >
                <AlertTriangle size={14} className="shrink-0 mt-0.5" />
                <span>{c.reason}</span>
              </div>
            ))}
          </div>
        )}

        {commandPreview && commandPreview.unknownFlags.length > 0 && (
          <div className="rounded-lg border border-status-warning/40 bg-status-warning/10 p-3 text-xs space-y-2">
            <div className="font-medium text-status-warning">
              Unknown flags detected:{" "}
              <span className="font-mono">
                {commandPreview.unknownFlags.join(", ")}
              </span>
            </div>
            <label className="inline-flex items-center gap-2 text-text-primary">
              <input
                type="checkbox"
                checked={allowUnknownFlags}
                onChange={(e) => setAllowUnknownFlags(e.target.checked)}
              />
              Allow unknown flags (advanced override)
            </label>
          </div>
        )}
      </section>

      {/* Save Preset Modal */}
      {showSaveModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
          <form
            onSubmit={handleSaveAsPreset}
            className="w-full max-w-md rounded-xl border border-border-default bg-surface p-5 space-y-4"
          >
            <h3 className="text-base font-semibold text-text-primary">
              Save as User Preset
            </h3>
            <input
              type="text"
              required
              placeholder="Preset name"
              value={savePresetName}
              onChange={(e) => setSavePresetName(e.target.value)}
              className="w-full rounded-lg border border-border-default bg-canvas px-3 py-2 text-sm text-text-primary"
            />
            <div className="flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setShowSaveModal(false)}
                className="rounded-lg border border-border-default px-3 py-1.5 text-xs text-text-secondary"
              >
                Cancel
              </button>
              <button
                type="submit"
                className="rounded-lg bg-accent-lime px-4 py-1.5 text-xs font-semibold text-accent-fg"
              >
                Save Preset
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
};
