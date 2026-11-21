import React from "react";
import { RefreshCw, Smartphone, CheckCircle2, AlertTriangle } from "lucide-react";
import type { Device, EnvironmentStatus, NavPage } from "../types/scrdeck";

interface TopBarProps {
  activePage: NavPage;
  environment: EnvironmentStatus | null;
  selectedDevice: Device | null;
  devicesLoading: boolean;
  onRefreshDevices: () => void;
  onGoToDevices: () => void;
}

const PAGE_TITLES: Record<NavPage, string> = {
  overview: "Overview",
  devices: "Devices",
  mirroring: "Mirroring Studio",
  presets: "Presets",
  "adb-tools": "ADB Tools",
  activity: "Activity & Logs",
  settings: "Settings",
  about: "About ScrDeck",
  setup: "Setup Assistant",
};

export const TopBar: React.FC<TopBarProps> = ({
  activePage,
  environment,
  selectedDevice,
  devicesLoading,
  onRefreshDevices,
  onGoToDevices,
}) => {
  const toolsReady =
    Boolean(environment?.toolchain.scrcpyValid) &&
    Boolean(environment?.toolchain.adbValid);

  return (
    <header className="flex h-14 shrink-0 items-center justify-between border-b border-border-default bg-surface px-6">
      <div className="flex items-center gap-3">
        <h1 className="text-base font-semibold text-text-primary">
          {PAGE_TITLES[activePage]}
        </h1>
      </div>

      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={onGoToDevices}
          className="flex items-center gap-2 rounded-lg border border-border-default bg-elevated px-3 py-1.5 text-xs text-text-secondary hover:text-text-primary transition-colors"
          title="Selected target device"
        >
          <Smartphone size={14} className="text-text-muted" />
          {selectedDevice ? (
            <span className="flex items-center gap-2">
              <span className="font-medium text-text-primary">
                {selectedDevice.model || selectedDevice.serial}
              </span>
              <span className="font-mono text-[11px] text-text-muted">
                {selectedDevice.serial}
              </span>
            </span>
          ) : (
            <span>No device selected</span>
          )}
        </button>

        <button
          type="button"
          onClick={onRefreshDevices}
          disabled={devicesLoading || !environment?.toolchain.adbValid}
          aria-label="Refresh devices"
          className="flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-2.5 py-1.5 text-xs font-medium text-text-secondary hover:text-text-primary disabled:opacity-50 transition-colors"
        >
          <RefreshCw
            size={14}
            className={devicesLoading ? "animate-spin" : ""}
          />
          <span className="hidden sm:inline">Refresh</span>
        </button>

        <div
          className={`flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium ${
            toolsReady
              ? "bg-status-success/15 text-status-success"
              : "bg-status-warning/15 text-status-warning"
          }`}
        >
          {toolsReady ? (
            <CheckCircle2 size={13} />
          ) : (
            <AlertTriangle size={13} />
          )}
          <span>{toolsReady ? "Tools Ready" : "Setup Needed"}</span>
        </div>
      </div>
    </header>
  );
};
