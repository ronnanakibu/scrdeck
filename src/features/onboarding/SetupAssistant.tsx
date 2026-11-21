import React, { useState } from "react";
import {
  CheckCircle2,
  Download,
  FolderOpen,
  RefreshCw,
  Usb,
  Wifi,
  AlertTriangle,
  ArrowRight,
  ChevronDown,
  ChevronUp,
} from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useAppStore } from "../../stores/useAppStore";
import { tauriApi } from "../../services/tauri";
import type { DriverDiagnosis } from "../../types/scrdeck";

export const SetupAssistant: React.FC = () => {
  const {
    environment,
    devices,
    installingTools,
    installError,
    installManagedTools,
    refreshToolchain,
    refreshDevices,
    updateAppSettings,
    setActivePage,
  } = useAppStore();

  const [step, setStep] = useState<number>(1);
  const [connectionMode, setConnectionMode] = useState<"usb" | "wireless">("usb");
  const [showTechDetails, setShowTechDetails] = useState(false);
  const [diagnosis, setDiagnosis] = useState<DriverDiagnosis | null>(null);
  const [diagnosing, setDiagnosing] = useState(false);

  const toolsReady =
    Boolean(environment?.toolchain.scrcpyValid) &&
    Boolean(environment?.toolchain.adbValid);

  const authorizedDevice = devices.find((d) => d.adbState === "device");
  const unauthorizedDevice = devices.find((d) => d.adbState === "unauthorized");

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

  const handleRunDiagnosis = async () => {
    setDiagnosing(true);
    try {
      await refreshDevices();
      const res = await tauriApi.diagnoseUsb();
      setDiagnosis(res);
    } finally {
      setDiagnosing(false);
    }
  };

  const handleFinishSetup = async () => {
    await updateAppSettings({ setupCompleted: true });
    setActivePage("overview");
  };

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div className="rounded-xl border border-border-default bg-surface p-6">
        <div className="flex items-center justify-between border-b border-border-default pb-4">
          <div>
            <p className="text-xs font-medium uppercase tracking-wider text-text-muted">
              Setup Assistant · Step {step} of 5
            </p>
            <h2 className="mt-1 text-xl font-semibold text-text-primary">
              {step === 1 && "Choose how you want to connect"}
              {step === 2 && "Check scrcpy and ADB tools"}
              {step === 3 && "Prepare your Android device"}
              {step === 4 && "Connect and authorize ADB"}
              {step === 5 && "Verify connection and open workspace"}
            </h2>
          </div>
          <button
            type="button"
            onClick={handleFinishSetup}
            className="text-xs font-medium text-text-secondary hover:text-text-primary"
          >
            Skip for now
          </button>
        </div>

        {/* Step 1: Welcome & Connection Mode */}
        {step === 1 && (
          <div className="mt-6 space-y-5">
            <p className="text-sm text-text-secondary">
              ScrDeck is an independent GUI companion for{" "}
              <strong className="text-text-primary">scrcpy</strong> and{" "}
              <strong className="text-text-primary">ADB</strong>. USB is
              recommended for your first setup because it is easier to diagnose
              and provides a stable baseline.
            </p>

            <div className="grid gap-3 sm:grid-cols-2">
              <button
                type="button"
                onClick={() => setConnectionMode("usb")}
                className={`flex flex-col items-start gap-2 rounded-xl border p-4 text-left transition-colors ${
                  connectionMode === "usb"
                    ? "border-accent-lime bg-elevated"
                    : "border-border-default bg-canvas hover:bg-subtle"
                }`}
              >
                <Usb size={20} className="text-accent-lime" />
                <div className="font-medium text-text-primary">
                  USB Cable (Recommended)
                </div>
                <p className="text-xs text-text-secondary">
                  Connect with a data-capable USB cable. Best for first-time
                  setup and lowest latency.
                </p>
              </button>

              <button
                type="button"
                onClick={() => setConnectionMode("wireless")}
                className={`flex flex-col items-start gap-2 rounded-xl border p-4 text-left transition-colors ${
                  connectionMode === "wireless"
                    ? "border-accent-lime bg-elevated"
                    : "border-border-default bg-canvas hover:bg-subtle"
                }`}
              >
                <Wifi size={20} className="text-accent-lime" />
                <div className="font-medium text-text-primary">
                  Wireless Debugging
                </div>
                <p className="text-xs text-text-secondary">
                  Pair over local Wi-Fi using Android 11+ Wireless Debugging
                  pairing code.
                </p>
              </button>
            </div>

            <div className="flex justify-end pt-2">
              <button
                type="button"
                onClick={() => setStep(2)}
                className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-4 py-2 text-sm font-semibold text-accent-fg hover:bg-accent-lime-hover"
              >
                Continue
                <ArrowRight size={16} />
              </button>
            </div>
          </div>
        )}

        {/* Step 2: Toolchain Check */}
        {step === 2 && (
          <div className="mt-6 space-y-5">
            <div className="rounded-lg border border-border-default bg-canvas p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-text-primary">
                  scrcpy executable
                </span>
                <span
                  className={`text-xs font-medium ${
                    environment?.toolchain.scrcpyValid
                      ? "text-status-success"
                      : "text-status-warning"
                  }`}
                >
                  {environment?.toolchain.scrcpyValid
                    ? `Ready (v${environment.toolchain.scrcpyVersion})`
                    : "Not found"}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-text-primary">
                  ADB executable
                </span>
                <span
                  className={`text-xs font-medium ${
                    environment?.toolchain.adbValid
                      ? "text-status-success"
                      : "text-status-warning"
                  }`}
                >
                  {environment?.toolchain.adbValid
                    ? `Ready (v${environment.toolchain.adbVersion})`
                    : "Not found"}
                </span>
              </div>
            </div>

            {!toolsReady && (
              <div className="space-y-3">
                <p className="text-xs text-text-secondary">
                  ScrDeck can download the official Windows x64 scrcpy release
                  from Genymobile&apos;s GitHub repository (which includes ADB),
                  or you can point ScrDeck to an existing scrcpy.exe on your PC.
                </p>
                <div className="flex flex-wrap gap-3">
                  <button
                    type="button"
                    onClick={installManagedTools}
                    disabled={installingTools}
                    className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-4 py-2 text-sm font-semibold text-accent-fg hover:bg-accent-lime-hover disabled:opacity-50"
                  >
                    <Download size={16} />
                    {installingTools
                      ? "Downloading official scrcpy..."
                      : "Install Official scrcpy Bundle"}
                  </button>
                  <button
                    type="button"
                    onClick={handleBrowseScrcpy}
                    className="inline-flex items-center gap-2 rounded-lg border border-border-default bg-elevated px-4 py-2 text-sm font-medium text-text-primary hover:bg-subtle"
                  >
                    <FolderOpen size={16} />
                    I already have scrcpy.exe
                  </button>
                </div>
                {installError && (
                  <div className="rounded-lg border border-status-error/40 bg-status-error/10 p-3 text-xs text-status-error">
                    {installError}
                  </div>
                )}
              </div>
            )}

            <button
              type="button"
              onClick={() => setShowTechDetails(!showTechDetails)}
              className="inline-flex items-center gap-1 text-xs text-text-secondary hover:text-text-primary"
            >
              {showTechDetails ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
              Show technical details
            </button>

            {showTechDetails && (
              <div className="rounded-lg border border-border-default bg-canvas p-3 font-mono text-xs text-text-secondary space-y-1">
                <div>OS: {environment?.osName} ({environment?.osArch})</div>
                <div>scrcpy: {environment?.toolchain.scrcpyPath || "None"}</div>
                <div>adb: {environment?.toolchain.adbPath || "None"}</div>
              </div>
            )}

            <div className="flex justify-between pt-2">
              <button
                type="button"
                onClick={() => setStep(1)}
                className="rounded-lg border border-border-default px-4 py-2 text-sm text-text-secondary hover:text-text-primary"
              >
                Back
              </button>
              <button
                type="button"
                onClick={() => setStep(3)}
                disabled={!toolsReady}
                className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-4 py-2 text-sm font-semibold text-accent-fg hover:bg-accent-lime-hover disabled:opacity-50"
              >
                Continue
                <ArrowRight size={16} />
              </button>
            </div>
          </div>
        )}

        {/* Step 3: Prepare Android */}
        {step === 3 && (
          <div className="mt-6 space-y-4">
            <p className="text-xs text-text-secondary">
              Menu names vary slightly across Samsung, Xiaomi/Redmi, Pixel,
              Motorola, and OnePlus devices.
            </p>
            <ol className="list-decimal space-y-2 pl-5 text-sm text-text-primary">
              <li>Unlock your Android phone and open <strong>Settings</strong>.</li>
              <li>Open <strong>About phone</strong> (or About device).</li>
              <li>
                Find <strong>Build number</strong> (sometimes inside Software
                information) and tap it <strong>7 times</strong> until Developer
                Options is enabled.
              </li>
              <li>
                Return to Settings, open <strong>Developer options</strong>, and
                enable{" "}
                <strong>
                  {connectionMode === "usb"
                    ? "USB debugging"
                    : "Wireless debugging"}
                </strong>
                .
              </li>
            </ol>

            <div className="flex justify-between pt-2">
              <button
                type="button"
                onClick={() => setStep(2)}
                className="rounded-lg border border-border-default px-4 py-2 text-sm text-text-secondary hover:text-text-primary"
              >
                Back
              </button>
              <button
                type="button"
                onClick={() => {
                  setStep(4);
                  void handleRunDiagnosis();
                }}
                className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-4 py-2 text-sm font-semibold text-accent-fg hover:bg-accent-lime-hover"
              >
                I&apos;ve enabled debugging
                <ArrowRight size={16} />
              </button>
            </div>
          </div>
        )}

        {/* Step 4: Connect & Authorize */}
        {step === 4 && (
          <div className="mt-6 space-y-4">
            {authorizedDevice ? (
              <div className="flex items-start gap-3 rounded-lg border border-status-success/40 bg-status-success/10 p-4">
                <CheckCircle2 size={20} className="shrink-0 text-status-success" />
                <div>
                  <div className="text-sm font-semibold text-text-primary">
                    Device authorized and ready
                  </div>
                  <p className="text-xs text-text-secondary mt-1">
                    Detected {authorizedDevice.model || authorizedDevice.serial} (
                    {authorizedDevice.serial}).
                  </p>
                </div>
              </div>
            ) : unauthorizedDevice ? (
              <div className="flex items-start gap-3 rounded-lg border border-status-warning/40 bg-status-warning/10 p-4">
                <AlertTriangle size={20} className="shrink-0 text-status-warning" />
                <div>
                  <div className="text-sm font-semibold text-text-primary">
                    Your phone needs permission
                  </div>
                  <p className="text-xs text-text-secondary mt-1">
                    Unlock your phone and tap <strong>Allow</strong> on the
                    &ldquo;Allow USB debugging?&rdquo; RSA prompt, then click
                    Re-test Connection.
                  </p>
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                <div className="rounded-lg border border-border-default bg-canvas p-4">
                  <div className="text-sm font-medium text-text-primary">
                    {diagnosis?.title || "Waiting for Android device"}
                  </div>
                  <p className="mt-1 text-xs text-text-secondary">
                    {diagnosis?.summary ||
                      "Connect your phone with a data-capable USB cable and unlock the screen."}
                  </p>
                  {diagnosis && diagnosis.checklist.length > 0 && (
                    <ul className="mt-3 list-disc space-y-1 pl-5 text-xs text-text-secondary">
                      {diagnosis.checklist.map((item, idx) => (
                        <li key={idx}>{item}</li>
                      ))}
                    </ul>
                  )}
                </div>

                {diagnosis && diagnosis.oemLinks.length > 0 && (
                  <div className="rounded-lg border border-border-default bg-canvas p-4 space-y-2">
                    <div className="text-xs font-semibold text-text-primary">
                      Official OEM USB Driver Links (only if Windows lacks an ADB interface)
                    </div>
                    <div className="grid gap-2 sm:grid-cols-2">
                      {diagnosis.oemLinks.map((link) => (
                        <button
                          key={link.url}
                          type="button"
                          onClick={() => void openUrl(link.url)}
                          className="rounded-lg border border-border-default bg-elevated p-2.5 text-left hover:bg-subtle"
                        >
                          <div className="text-xs font-medium text-accent-lime">
                            {link.label}
                          </div>
                          <div className="text-[11px] text-text-muted">
                            {link.vendor}
                          </div>
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}

            <div className="flex flex-wrap items-center justify-between gap-3 pt-2">
              <button
                type="button"
                onClick={() => setStep(3)}
                className="rounded-lg border border-border-default px-4 py-2 text-sm text-text-secondary hover:text-text-primary"
              >
                Back
              </button>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={handleRunDiagnosis}
                  disabled={diagnosing}
                  className="inline-flex items-center gap-2 rounded-lg border border-border-default bg-elevated px-4 py-2 text-sm font-medium text-text-primary hover:bg-subtle"
                >
                  <RefreshCw
                    size={15}
                    className={diagnosing ? "animate-spin" : ""}
                  />
                  Re-test Connection
                </button>
                <button
                  type="button"
                  onClick={() => setStep(5)}
                  className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-4 py-2 text-sm font-semibold text-accent-fg hover:bg-accent-lime-hover"
                >
                  Continue
                  <ArrowRight size={16} />
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Step 5: Summary */}
        {step === 5 && (
          <div className="mt-6 space-y-5">
            <div className="rounded-lg border border-border-default bg-canvas p-4 space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-text-secondary">scrcpy</span>
                <span className="font-mono text-text-primary">
                  {environment?.toolchain.scrcpyVersion || "Not configured"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-text-secondary">ADB</span>
                <span className="font-mono text-text-primary">
                  {environment?.toolchain.adbVersion || "Not configured"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-text-secondary">Connected devices</span>
                <span className="tabular-nums text-text-primary">
                  {devices.length}
                </span>
              </div>
            </div>

            <div className="flex justify-between pt-2">
              <button
                type="button"
                onClick={() => setStep(4)}
                className="rounded-lg border border-border-default px-4 py-2 text-sm text-text-secondary hover:text-text-primary"
              >
                Back
              </button>
              <button
                type="button"
                onClick={handleFinishSetup}
                className="inline-flex items-center gap-2 rounded-lg bg-accent-lime px-5 py-2 text-sm font-semibold text-accent-fg hover:bg-accent-lime-hover"
              >
                Open Workspace
                <ArrowRight size={16} />
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
