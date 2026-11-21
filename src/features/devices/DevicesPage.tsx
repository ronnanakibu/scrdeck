import React, { useState } from "react";
import {
  RefreshCw,
  Smartphone,
  Wifi,
  Usb,
  Copy,
  Check,
  AlertTriangle,
  CheckCircle2,
  Wrench,
  Unplug,
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useAppStore } from "../../stores/useAppStore";
import { tauriApi } from "../../services/tauri";
import type { AdbState, DriverDiagnosis } from "../../types/scrdeck";

function stateBadge(state: AdbState) {
  switch (state) {
    case "device":
      return {
        label: "Authorized / Ready",
        cls: "bg-status-success/15 text-status-success",
      };
    case "unauthorized":
      return {
        label: "Authorization needed",
        cls: "bg-status-warning/15 text-status-warning",
      };
    case "offline":
      return {
        label: "Offline",
        cls: "bg-status-error/15 text-status-error",
      };
    default:
      return {
        label: "Unavailable",
        cls: "bg-subtle text-text-secondary",
      };
  }
}

export const DevicesPage: React.FC = () => {
  const {
    devices,
    devicesLoading,
    devicesError,
    selectedDeviceSerial,
    selectDevice,
    refreshDevices,
    setActivePage,
  } = useAppStore();

  const [copiedSerial, setCopiedSerial] = useState<string | null>(null);
  const [showWireless, setShowWireless] = useState(false);
  const [pairAddress, setPairAddress] = useState("");
  const [pairCode, setPairCode] = useState("");
  const [connectAddress, setConnectAddress] = useState("");
  const [wirelessStatus, setWirelessStatus] = useState<string | null>(null);
  const [wirelessError, setWirelessError] = useState<string | null>(null);
  const [wirelessBusy, setWirelessBusy] = useState(false);

  const [diagnosis, setDiagnosis] = useState<DriverDiagnosis | null>(null);

  const handleCopySerial = async (serial: string) => {
    await navigator.clipboard.writeText(serial);
    setCopiedSerial(serial);
    setTimeout(() => setCopiedSerial(null), 1500);
  };

  const handlePair = async (e: React.FormEvent) => {
    e.preventDefault();
    setWirelessBusy(true);
    setWirelessError(null);
    setWirelessStatus(null);
    try {
      const msg = await tauriApi.pairWirelessDevice(pairAddress, pairCode);
      setPairCode(""); // Clear transient pairing code immediately
      setWirelessStatus(msg);
      await refreshDevices();
    } catch (err) {
      setWirelessError(err instanceof Error ? err.message : String(err));
    } finally {
      setWirelessBusy(false);
    }
  };

  const handleConnect = async (e: React.FormEvent) => {
    e.preventDefault();
    setWirelessBusy(true);
    setWirelessError(null);
    setWirelessStatus(null);
    try {
      const msg = await tauriApi.connectWirelessDevice(connectAddress);
      setWirelessStatus(msg);
      await refreshDevices();
    } catch (err) {
      setWirelessError(err instanceof Error ? err.message : String(err));
    } finally {
      setWirelessBusy(false);
    }
  };

  const handleDisconnect = async (serial: string) => {
    await tauriApi.disconnectWirelessDevice(serial);
    await refreshDevices();
  };

  const handleDiagnose = async () => {
    const res = await tauriApi.diagnoseUsb();
    setDiagnosis(res);
  };

  return (
    <div className="mx-auto max-w-5xl space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold text-text-primary">
            Connected Android Devices
          </h2>
          <p className="text-xs text-text-secondary">
            Select a target device explicitly before starting a scrcpy session.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => setShowWireless(!showWireless)}
            className="inline-flex items-center gap-2 rounded-lg border border-border-default bg-elevated px-3 py-2 text-xs font-medium text-text-primary hover:bg-subtle"
          >
            <Wifi size={14} />
            Wireless Debugging
          </button>
          <button
            type="button"
            onClick={() => void handleDiagnose()}
            className="inline-flex items-center gap-2 rounded-lg border border-border-default bg-elevated px-3 py-2 text-xs font-medium text-text-primary hover:bg-subtle"
          >
            <Wrench size={14} />
            Troubleshoot USB
          </button>
          <button
            type="button"
            onClick={() => void refreshDevices()}
            disabled={devicesLoading}
            className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-3.5 py-2 text-xs font-semibold text-accent-fg hover:bg-accent-lime-hover disabled:opacity-50"
          >
            <RefreshCw
              size={14}
              className={devicesLoading ? "animate-spin" : ""}
            />
            Refresh Devices
          </button>
        </div>
      </div>

      {devicesError && (
        <div className="rounded-xl border border-status-error/40 bg-status-error/10 p-4 text-xs text-status-error">
          {devicesError}
        </div>
      )}

      {/* Wireless Debugging Panel */}
      {showWireless && (
        <div className="rounded-xl border border-border-default bg-surface p-5 space-y-4">
          <div className="flex items-start justify-between">
            <div>
              <h3 className="text-sm font-semibold text-text-primary">
                Wireless Debugging (Android 11+)
              </h3>
              <p className="text-xs text-text-secondary mt-0.5">
                Your phone and PC must be on the same trusted local Wi-Fi
                network. Pairing codes are never saved to logs.
              </p>
            </div>
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <form onSubmit={handlePair} className="space-y-3 rounded-lg border border-border-default bg-canvas p-4">
              <div className="text-xs font-semibold text-text-primary">
                1. Pair with pairing code
              </div>
              <input
                type="text"
                placeholder="IP:PairingPort (e.g. 192.168.1.42:37123)"
                value={pairAddress}
                onChange={(e) => setPairAddress(e.target.value)}
                className="w-full rounded-lg border border-border-default bg-surface px-3 py-2 text-xs text-text-primary placeholder:text-text-muted"
              />
              <input
                type="password"
                placeholder="6-digit pairing code"
                value={pairCode}
                onChange={(e) => setPairCode(e.target.value)}
                className="w-full rounded-lg border border-border-default bg-surface px-3 py-2 text-xs text-text-primary placeholder:text-text-muted"
              />
              <button
                type="submit"
                disabled={wirelessBusy}
                className="rounded-lg bg-accent-lime px-3 py-1.5 text-xs font-semibold text-accent-fg hover:bg-accent-lime-hover disabled:opacity-50"
              >
                Pair Device
              </button>
            </form>

            <form onSubmit={handleConnect} className="space-y-3 rounded-lg border border-border-default bg-canvas p-4">
              <div className="text-xs font-semibold text-text-primary">
                2. Connect to Wireless Debugging endpoint
              </div>
              <input
                type="text"
                placeholder="IP:ConnectPort (e.g. 192.168.1.42:41555)"
                value={connectAddress}
                onChange={(e) => setConnectAddress(e.target.value)}
                className="w-full rounded-lg border border-border-default bg-surface px-3 py-2 text-xs text-text-primary placeholder:text-text-muted"
              />
              <p className="text-[11px] text-text-muted">
                Note: Android uses a different port for connecting than for
                pairing.
              </p>
              <button
                type="submit"
                disabled={wirelessBusy}
                className="rounded-lg border border-border-default bg-elevated px-3 py-1.5 text-xs font-medium text-text-primary hover:bg-subtle disabled:opacity-50"
              >
                Connect
              </button>
            </form>
          </div>

          {wirelessStatus && (
            <div className="rounded-lg border border-status-success/40 bg-status-success/10 p-3 text-xs text-status-success">
              {wirelessStatus}
            </div>
          )}
          {wirelessError && (
            <div className="rounded-lg border border-status-error/40 bg-status-error/10 p-3 text-xs text-status-error">
              {wirelessError}
            </div>
          )}
        </div>
      )}

      {/* USB / Driver Diagnosis */}
      {diagnosis && (
        <div className="rounded-xl border border-border-default bg-surface p-5 space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <AlertTriangle size={16} className="text-status-warning" />
              <h3 className="text-sm font-semibold text-text-primary">
                {diagnosis.title}
              </h3>
            </div>
            <span className="text-[11px] uppercase tracking-wider text-text-muted">
              Confidence: {diagnosis.confidence}
            </span>
          </div>
          <p className="text-xs text-text-secondary">{diagnosis.summary}</p>
          <ul className="list-disc space-y-1 pl-5 text-xs text-text-secondary">
            {diagnosis.checklist.map((item, i) => (
              <li key={i}>{item}</li>
            ))}
          </ul>
          {diagnosis.oemLinks.length > 0 && (
            <div className="flex flex-wrap gap-2 pt-2">
              {diagnosis.oemLinks.map((link) => (
                <button
                  key={link.url}
                  type="button"
                  onClick={() => void openUrl(link.url)}
                  className="rounded-lg border border-border-default bg-elevated px-3 py-1.5 text-xs font-medium text-accent-lime hover:bg-subtle"
                >
                  {link.label}
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Device List */}
      {devices.length === 0 ? (
        <div className="rounded-xl border border-border-default bg-surface p-8 text-center space-y-3">
          <Smartphone size={28} className="mx-auto text-text-muted" />
          <h3 className="text-sm font-semibold text-text-primary">
            No Android device detected
          </h3>
          <p className="mx-auto max-w-md text-xs text-text-secondary">
            Connect your phone with a data-capable USB cable, unlock it, and
            ensure USB Debugging is enabled in Developer Options.
          </p>
          <div className="flex justify-center gap-2 pt-2">
            <button
              type="button"
              onClick={() => setActivePage("setup")}
              className="rounded-lg bg-accent-lime px-4 py-2 text-xs font-semibold text-accent-fg hover:bg-accent-lime-hover"
            >
              Open Setup Assistant
            </button>
          </div>
        </div>
      ) : (
        <div className="grid gap-3">
          {devices.map((dev) => {
            const badge = stateBadge(dev.adbState);
            const isSelected = dev.serial === selectedDeviceSerial;
            return (
              <div
                key={dev.serial}
                className={`flex flex-col gap-4 rounded-xl border p-4 sm:flex-row sm:items-center sm:justify-between transition-colors ${
                  isSelected
                    ? "border-accent-lime bg-elevated"
                    : "border-border-default bg-surface"
                }`}
              >
                <div className="space-y-1.5">
                  <div className="flex flex-wrap items-center gap-2">
                    {dev.transport === "tcpIp" ? (
                      <Wifi size={16} className="text-text-secondary" />
                    ) : (
                      <Usb size={16} className="text-text-secondary" />
                    )}
                    <span className="text-sm font-semibold text-text-primary">
                      {dev.model || "Android Device"}
                    </span>
                    <span
                      className={`rounded-full px-2.5 py-0.5 text-[11px] font-medium ${badge.cls}`}
                    >
                      {badge.label}
                    </span>
                    {isSelected && (
                      <span className="inline-flex items-center gap-1 rounded-full bg-accent-lime/20 px-2 py-0.5 text-[11px] font-medium text-accent-lime">
                        <CheckCircle2 size={11} /> Selected
                      </span>
                    )}
                  </div>

                  <div className="flex flex-wrap items-center gap-3 text-xs text-text-secondary">
                    <button
                      type="button"
                      onClick={() => void handleCopySerial(dev.serial)}
                      className="inline-flex items-center gap-1 font-mono text-text-muted hover:text-text-primary"
                      title="Copy serial"
                    >
                      {dev.serial}
                      {copiedSerial === dev.serial ? (
                        <Check size={12} className="text-status-success" />
                      ) : (
                        <Copy size={12} />
                      )}
                    </button>
                    <span>
                      Android {dev.androidRelease || "Unavailable"}
                      {dev.apiLevel ? ` (API ${dev.apiLevel})` : ""}
                    </span>
                    <span>
                      Battery:{" "}
                      {dev.batteryPercent !== null
                        ? `${dev.batteryPercent}% (${dev.chargingState || ""})`
                        : "Unavailable"}
                    </span>
                  </div>

                  {dev.adbState === "unauthorized" && (
                    <p className="text-xs text-status-warning">
                      Unlock your phone and tap Allow on the &ldquo;Allow USB
                      debugging?&rdquo; RSA prompt.
                    </p>
                  )}
                </div>

                <div className="flex flex-wrap items-center gap-2">
                  {dev.transport === "tcpIp" && (
                    <button
                      type="button"
                      onClick={() => void handleDisconnect(dev.serial)}
                      className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-canvas px-3 py-1.5 text-xs text-text-secondary hover:text-text-primary"
                    >
                      <Unplug size={13} />
                      Disconnect
                    </button>
                  )}
                  <button
                    type="button"
                    onClick={() => {
                      selectDevice(dev.serial);
                      setActivePage("mirroring");
                    }}
                    className={`rounded-lg px-3.5 py-1.5 text-xs font-semibold ${
                      isSelected
                        ? "bg-accent-lime text-accent-fg"
                        : "border border-border-default bg-canvas text-text-primary hover:bg-subtle"
                    }`}
                  >
                    {isSelected ? "Open Mirroring" : "Select Device"}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
