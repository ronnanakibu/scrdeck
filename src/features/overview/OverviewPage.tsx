import React from "react";
import {
  CheckCircle2,
  AlertTriangle,
  Smartphone,
  Play,
  Usb,
  Wifi,
  SlidersHorizontal,
  ArrowRight,
} from "lucide-react";
import { useAppStore } from "../../stores/useAppStore";

export const OverviewPage: React.FC = () => {
  const {
    environment,
    devices,
    selectedDeviceSerial,
    presets,
    logs,
    sessions,
    setActivePage,
    applyPreset,
    startMirroring,
  } = useAppStore();

  const selectedDevice =
    devices.find((d) => d.serial === selectedDeviceSerial) || null;
  const activeSession = sessions.find(
    (s) => s.status === "running" || s.status === "starting"
  );

  const quickPresets = presets.slice(0, 4);
  const recentLogs = logs.slice(0, 5);

  return (
    <div className="mx-auto max-w-5xl space-y-6">
      {/* System Readiness Strip */}
      <section
        aria-label="System readiness"
        className="grid gap-3 sm:grid-cols-3"
      >
        <div className="flex items-center justify-between rounded-xl border border-border-default bg-surface p-4">
          <div>
            <div className="text-xs text-text-muted">scrcpy</div>
            <div className="mt-0.5 text-sm font-semibold text-text-primary tabular-nums">
              {environment?.toolchain.scrcpyValid
                ? `v${environment.toolchain.scrcpyVersion}`
                : "Not configured"}
            </div>
          </div>
          {environment?.toolchain.scrcpyValid ? (
            <CheckCircle2 size={18} className="text-status-success" />
          ) : (
            <AlertTriangle size={18} className="text-status-warning" />
          )}
        </div>

        <div className="flex items-center justify-between rounded-xl border border-border-default bg-surface p-4">
          <div>
            <div className="text-xs text-text-muted">ADB</div>
            <div className="mt-0.5 text-sm font-semibold text-text-primary tabular-nums">
              {environment?.toolchain.adbValid
                ? `v${environment.toolchain.adbVersion}`
                : "Not configured"}
            </div>
          </div>
          {environment?.toolchain.adbValid ? (
            <CheckCircle2 size={18} className="text-status-success" />
          ) : (
            <AlertTriangle size={18} className="text-status-warning" />
          )}
        </div>

        <div className="flex items-center justify-between rounded-xl border border-border-default bg-surface p-4">
          <div>
            <div className="text-xs text-text-muted">Discovered Devices</div>
            <div className="mt-0.5 text-sm font-semibold text-text-primary tabular-nums">
              {devices.length} {devices.length === 1 ? "device" : "devices"}
            </div>
          </div>
          <Smartphone size={18} className="text-text-secondary" />
        </div>
      </section>

      {/* Main Device Panel */}
      <section className="rounded-xl border border-border-default bg-surface p-6">
        {selectedDevice ? (
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="space-y-1">
              <div className="flex items-center gap-2.5">
                <h2 className="text-lg font-semibold text-text-primary">
                  {selectedDevice.model || selectedDevice.serial}
                </h2>
                <span
                  className={`rounded-full px-2.5 py-0.5 text-xs font-medium ${
                    selectedDevice.adbState === "device"
                      ? "bg-status-success/15 text-status-success"
                      : "bg-status-warning/15 text-status-warning"
                  }`}
                >
                  {selectedDevice.adbState === "device"
                    ? "Authorized / Ready"
                    : selectedDevice.adbState}
                </span>
              </div>
              <div className="flex flex-wrap items-center gap-3 text-xs text-text-secondary">
                <span className="font-mono">{selectedDevice.serial}</span>
                <span>Transport: {selectedDevice.transport.toUpperCase()}</span>
                {selectedDevice.androidRelease && (
                  <span>Android {selectedDevice.androidRelease}</span>
                )}
                {selectedDevice.batteryPercent !== null && (
                  <span className="tabular-nums">
                    Battery {selectedDevice.batteryPercent}%
                  </span>
                )}
              </div>
            </div>

            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => setActivePage("mirroring")}
                className="rounded-lg border border-border-default bg-elevated px-3.5 py-2 text-sm font-medium text-text-primary hover:bg-subtle"
              >
                Configure
              </button>
              <button
                type="button"
                disabled={
                  selectedDevice.adbState !== "device" ||
                  !environment?.toolchain.scrcpyValid ||
                  Boolean(activeSession)
                }
                onClick={() => void startMirroring()}
                className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-4 py-2 text-sm font-semibold text-accent-fg hover:bg-accent-lime-hover disabled:opacity-50"
              >
                <Play size={15} />
                {activeSession ? "Session Running" : "Start Mirroring"}
              </button>
            </div>
          </div>
        ) : (
          <div className="flex flex-col items-start justify-between gap-4 sm:flex-row sm:items-center">
            <div>
              <h2 className="text-base font-semibold text-text-primary">
                No Android device selected
              </h2>
              <p className="mt-1 text-sm text-text-secondary">
                Connect a phone via USB or Wireless Debugging to start a scrcpy
                session.
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => setActivePage("setup")}
                className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-4 py-2 text-sm font-semibold text-accent-fg hover:bg-accent-lime-hover"
              >
                <Usb size={15} />
                Connect via USB
              </button>
              <button
                type="button"
                onClick={() => setActivePage("devices")}
                className="inline-flex items-center gap-2 rounded-lg border border-border-default bg-elevated px-4 py-2 text-sm font-medium text-text-primary hover:bg-subtle"
              >
                <Wifi size={15} />
                Connect wirelessly
              </button>
            </div>
          </div>
        )}
      </section>

      {/* Quick Presets */}
      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-text-primary">
            Quick Presets
          </h3>
          <button
            type="button"
            onClick={() => setActivePage("presets")}
            className="inline-flex items-center gap-1 text-xs text-text-secondary hover:text-text-primary"
          >
            All presets <ArrowRight size={13} />
          </button>
        </div>
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {quickPresets.map((preset) => (
            <button
              key={preset.id}
              type="button"
              onClick={() => {
                applyPreset(preset);
                setActivePage("mirroring");
              }}
              className="flex flex-col items-start gap-1.5 rounded-xl border border-border-default bg-surface p-4 text-left hover:bg-elevated transition-colors"
            >
              <div className="flex items-center gap-2 text-sm font-medium text-text-primary">
                <SlidersHorizontal size={14} className="text-accent-lime" />
                {preset.name}
              </div>
              <p className="text-xs text-text-secondary line-clamp-2">
                {preset.description}
              </p>
            </button>
          ))}
        </div>
      </section>

      {/* Recent Activity */}
      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-text-primary">
            Recent Activity
          </h3>
          <button
            type="button"
            onClick={() => setActivePage("activity")}
            className="inline-flex items-center gap-1 text-xs text-text-secondary hover:text-text-primary"
          >
            View all logs <ArrowRight size={13} />
          </button>
        </div>
        <div className="rounded-xl border border-border-default bg-surface divide-y divide-border-default">
          {recentLogs.length === 0 ? (
            <div className="p-4 text-xs text-text-muted">
              No activity recorded yet.
            </div>
          ) : (
            recentLogs.map((entry) => (
              <div
                key={entry.id}
                className="flex items-center justify-between gap-4 px-4 py-3 text-xs"
              >
                <div className="min-w-0">
                  <span className="font-medium text-text-primary">
                    {entry.title}
                  </span>
                  <span className="ml-2 text-text-secondary truncate">
                    {entry.message}
                  </span>
                </div>
                <span className="shrink-0 font-mono text-[11px] text-text-muted tabular-nums">
                  {new Date(entry.timestamp).toLocaleTimeString()}
                </span>
              </div>
            ))
          )}
        </div>
      </section>
    </div>
  );
};
