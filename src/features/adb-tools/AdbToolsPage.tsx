import React, { useState } from "react";
import { Server, RefreshCw, AlertTriangle, Wrench } from "lucide-react";
import { useAppStore } from "../../stores/useAppStore";
import { tauriApi } from "../../services/tauri";

export const AdbToolsPage: React.FC = () => {
  const { environment, refreshDevices, setActivePage } = useAppStore();
  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [confirmRestart, setConfirmRestart] = useState(false);
  const [busy, setBusy] = useState(false);

  const adbReady = Boolean(environment?.toolchain.adbValid);

  const handleStartServer = async () => {
    setBusy(true);
    setErrorMsg(null);
    setStatusMsg(null);
    try {
      const res = await tauriApi.startAdbServer();
      setStatusMsg(res || "ADB server is running.");
      await refreshDevices();
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const handleRestartConfirmed = async () => {
    setBusy(true);
    setErrorMsg(null);
    setStatusMsg(null);
    try {
      const res = await tauriApi.restartAdbServerConfirmed();
      setStatusMsg(res || "ADB server restarted.");
      setConfirmRestart(false);
      await refreshDevices();
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-text-primary">ADB Tools</h2>
        <p className="text-xs text-text-secondary">
          v0.1 connection utilities. ScrDeck treats the host ADB server as
          shared infrastructure and never kills it silently.
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        <div className="rounded-xl border border-border-default bg-surface p-5 space-y-3">
          <div className="flex items-center gap-2 text-sm font-semibold text-text-primary">
            <Server size={16} className="text-accent-lime" />
            ADB Server Control
          </div>
          <p className="text-xs text-text-secondary">
            Active binary:{" "}
            <span className="font-mono text-text-primary">
              {environment?.toolchain.adbPath || "Not configured"}
            </span>
          </p>
          <div className="flex flex-wrap gap-2 pt-1">
            <button
              type="button"
              disabled={!adbReady || busy}
              onClick={() => void handleStartServer()}
              className="rounded-lg bg-accent-lime px-3.5 py-2 text-xs font-semibold text-accent-fg hover:bg-accent-lime-hover disabled:opacity-50"
            >
              Start / Check ADB Server
            </button>
            <button
              type="button"
              disabled={!adbReady || busy}
              onClick={() => setConfirmRestart(true)}
              className="rounded-lg border border-border-default bg-elevated px-3.5 py-2 text-xs font-medium text-text-primary hover:bg-subtle disabled:opacity-50"
            >
              Restart ADB Server...
            </button>
          </div>
        </div>

        <div className="rounded-xl border border-border-default bg-surface p-5 space-y-3">
          <div className="flex items-center gap-2 text-sm font-semibold text-text-primary">
            <Wrench size={16} className="text-accent-lime" />
            Device &amp; Connection Actions
          </div>
          <p className="text-xs text-text-secondary">
            Refresh connected devices or open the USB &amp; Wireless
            troubleshooting tools.
          </p>
          <div className="flex flex-wrap gap-2 pt-1">
            <button
              type="button"
              disabled={!adbReady}
              onClick={() => void refreshDevices()}
              className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3.5 py-2 text-xs font-medium text-text-primary hover:bg-subtle disabled:opacity-50"
            >
              <RefreshCw size={13} />
              Refresh Devices
            </button>
            <button
              type="button"
              onClick={() => setActivePage("devices")}
              className="rounded-lg border border-border-default bg-elevated px-3.5 py-2 text-xs font-medium text-text-primary hover:bg-subtle"
            >
              Open Devices &amp; Wireless
            </button>
          </div>
        </div>
      </div>

      {confirmRestart && (
        <div className="rounded-xl border border-status-warning/40 bg-status-warning/10 p-4 space-y-3">
          <div className="flex items-start gap-2.5">
            <AlertTriangle size={18} className="shrink-0 text-status-warning" />
            <div className="text-xs space-y-1">
              <div className="font-semibold text-text-primary">
                Confirm ADB Server Restart
              </div>
              <p className="text-text-secondary">
                Restarting the shared ADB server may temporarily interrupt
                Android Studio or other development tools using ADB on port
                5037.
              </p>
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <button
              type="button"
              onClick={() => setConfirmRestart(false)}
              className="rounded-lg border border-border-default px-3 py-1.5 text-xs text-text-secondary"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={() => void handleRestartConfirmed()}
              className="rounded-lg bg-status-warning px-3.5 py-1.5 text-xs font-semibold text-black"
            >
              Confirm Restart
            </button>
          </div>
        </div>
      )}

      {statusMsg && (
        <div className="rounded-xl border border-status-success/40 bg-status-success/10 p-4 text-xs text-status-success">
          {statusMsg}
        </div>
      )}
      {errorMsg && (
        <div className="rounded-xl border border-status-error/40 bg-status-error/10 p-4 text-xs text-status-error">
          {errorMsg}
        </div>
      )}
    </div>
  );
};
