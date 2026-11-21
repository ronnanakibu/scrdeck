import React from "react";
import { ExternalLink } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { BrandMark } from "../../components/BrandMark";
import { useAppStore } from "../../stores/useAppStore";

export const AboutPage: React.FC = () => {
  const { environment } = useAppStore();

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      <section className="rounded-xl border border-border-default bg-surface p-6 space-y-4">
        <div className="flex items-center gap-3">
          <BrandMark size={32} className="text-text-primary" />
          <div>
            <h2 className="text-xl font-semibold text-text-primary">ScrDeck</h2>
            <p className="text-xs text-text-secondary">
              Your Android. Your Workspace. · Version 0.1.0
            </p>
          </div>
        </div>

        <div className="grid gap-3 sm:grid-cols-3 rounded-lg border border-border-default bg-canvas p-4 text-xs">
          <div>
            <div className="text-text-muted">ScrDeck Version</div>
            <div className="font-mono font-medium text-text-primary mt-0.5">
              0.1.0
            </div>
          </div>
          <div>
            <div className="text-text-muted">scrcpy Version</div>
            <div className="font-mono font-medium text-text-primary mt-0.5">
              {environment?.toolchain.scrcpyVersion || "Not detected"}
            </div>
          </div>
          <div>
            <div className="text-text-muted">ADB Version</div>
            <div className="font-mono font-medium text-text-primary mt-0.5">
              {environment?.toolchain.adbVersion || "Not detected"}
            </div>
          </div>
        </div>
      </section>

      {/* Canonical Required Product Disclaimer */}
      <section className="rounded-xl border border-border-default bg-surface p-6 space-y-3">
        <h3 className="text-sm font-semibold text-text-primary">
          Upstream Attribution &amp; Disclaimer
        </h3>
        <p className="text-xs leading-relaxed text-text-secondary">
          <strong className="text-text-primary">
            ScrDeck is an independent, open-source GUI companion for scrcpy and
            ADB.
          </strong>{" "}
          It provides a graphical interface for configuring, launching, and
          managing operations performed by these third-party tools. Core screen
          mirroring, media transport, and device-control functionality are
          provided by the original scrcpy project developed by Genymobile. ADB
          is provided by Android SDK Platform-Tools. ScrDeck does not own,
          replace, reimplement, or claim authorship of these upstream tools or
          their core functionality. ScrDeck is not affiliated with, endorsed by,
          or officially supported by Genymobile or Google. scrcpy, Android SDK
          Platform-Tools, and bundled dependencies remain subject to their
          respective licenses and notices.
        </p>
      </section>

      {/* Official Upstream Links */}
      <section className="rounded-xl border border-border-default bg-surface p-6 space-y-3">
        <h3 className="text-sm font-semibold text-text-primary">
          Official Upstream Projects
        </h3>
        <div className="flex flex-wrap gap-3">
          <button
            type="button"
            onClick={() => void openUrl("https://github.com/Genymobile/scrcpy")}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3.5 py-2 text-xs font-medium text-accent-lime hover:bg-subtle"
          >
            Genymobile / scrcpy (Apache-2.0)
            <ExternalLink size={13} />
          </button>
          <button
            type="button"
            onClick={() =>
              void openUrl("https://developer.android.com/tools/adb")
            }
            className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3.5 py-2 text-xs font-medium text-accent-lime hover:bg-subtle"
          >
            Android Debug Bridge (ADB)
            <ExternalLink size={13} />
          </button>
        </div>
      </section>
    </div>
  );
};
