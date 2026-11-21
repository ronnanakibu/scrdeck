import React, { useState } from "react";
import { Download, Trash2, Search, AlertCircle } from "lucide-react";
import { useAppStore } from "../../stores/useAppStore";
import { tauriApi } from "../../services/tauri";
import type { LogSeverity } from "../../types/scrdeck";

export const ActivityLogsPage: React.FC = () => {
  const { logs, settings, clearLogs } = useAppStore();
  const [severityFilter, setSeverityFilter] = useState<LogSeverity | "all">(
    "all"
  );
  const [search, setSearch] = useState("");
  const [exportedBundle, setExportedBundle] = useState<string | null>(null);

  const filtered = logs.filter((entry) => {
    if (severityFilter !== "all" && entry.severity !== severityFilter) {
      return false;
    }
    if (search.trim()) {
      const q = search.toLowerCase();
      return (
        entry.title.toLowerCase().includes(q) ||
        entry.message.toLowerCase().includes(q) ||
        (entry.rawOutput?.toLowerCase().includes(q) ?? false)
      );
    }
    return true;
  });

  const handleExportDiagnostics = async () => {
    const redact = settings?.redactDiagnosticsByDefault ?? true;
    const bundle = await tauriApi.exportDiagnosticsBundle(redact);
    setExportedBundle(bundle);
  };

  return (
    <div className="mx-auto max-w-5xl space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold text-text-primary">
            Activity &amp; Logs
          </h2>
          <p className="text-xs text-text-secondary">
            Local process output, error explanations, and sanitized diagnostic
            export.
          </p>
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => void handleExportDiagnostics()}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3 py-2 text-xs font-medium text-text-primary hover:bg-subtle"
          >
            <Download size={14} />
            Export Sanitized Diagnostics
          </button>
          <button
            type="button"
            onClick={() => void clearLogs()}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border-default bg-elevated px-3 py-2 text-xs font-medium text-text-secondary hover:text-status-error"
          >
            <Trash2 size={14} />
            Clear
          </button>
        </div>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-3">
        <div className="relative flex-1 min-w-[200px]">
          <Search
            size={14}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted"
          />
          <input
            type="text"
            placeholder="Search logs or raw output..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full rounded-lg border border-border-default bg-surface pl-9 pr-3 py-2 text-xs text-text-primary"
          />
        </div>
        <select
          aria-label="Filter by severity"
          value={severityFilter}
          onChange={(e) =>
            setSeverityFilter(e.target.value as LogSeverity | "all")
          }
          className="rounded-lg border border-border-default bg-surface px-3 py-2 text-xs text-text-primary"
        >
          <option value="all">All severities</option>
          <option value="info">Info</option>
          <option value="success">Success</option>
          <option value="warning">Warning</option>
          <option value="error">Error</option>
          <option value="debug">Debug</option>
        </select>
      </div>

      {exportedBundle && (
        <div className="rounded-xl border border-border-default bg-surface p-4 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-text-primary">
              Sanitized Diagnostic Bundle (serials, IPs, and user paths redacted)
            </span>
            <button
              type="button"
              onClick={() => setExportedBundle(null)}
              className="text-xs text-text-secondary hover:text-text-primary"
            >
              Close
            </button>
          </div>
          <pre className="max-h-56 overflow-auto rounded-lg border border-border-default bg-canvas p-3 font-mono text-xs text-text-primary">
            {exportedBundle}
          </pre>
        </div>
      )}

      <div className="space-y-2">
        {filtered.length === 0 ? (
          <div className="rounded-xl border border-border-default bg-surface p-6 text-center text-xs text-text-muted">
            No matching log entries.
          </div>
        ) : (
          filtered.map((entry) => (
            <div
              key={entry.id}
              className="rounded-xl border border-border-default bg-surface p-4 space-y-2"
            >
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <span
                    className={`rounded-full px-2 py-0.5 text-[10px] font-semibold uppercase ${
                      entry.severity === "error"
                        ? "bg-status-error/15 text-status-error"
                        : entry.severity === "warning"
                          ? "bg-status-warning/15 text-status-warning"
                          : entry.severity === "success"
                            ? "bg-status-success/15 text-status-success"
                            : "bg-subtle text-text-secondary"
                    }`}
                  >
                    {entry.severity}
                  </span>
                  <span className="text-xs font-semibold text-text-primary">
                    {entry.title}
                  </span>
                  <span className="text-[11px] text-text-muted">
                    [{entry.source}]
                  </span>
                </div>
                <span className="font-mono text-[11px] text-text-muted tabular-nums">
                  {new Date(entry.timestamp).toLocaleString()}
                </span>
              </div>

              <p className="text-xs text-text-secondary">{entry.message}</p>

              {entry.explanation && (
                <div className="rounded-lg border border-status-warning/30 bg-status-warning/10 p-3 text-xs space-y-1">
                  <div className="flex items-center gap-1.5 font-semibold text-text-primary">
                    <AlertCircle size={14} className="text-status-warning" />
                    Explain this error: {entry.explanation.title}
                  </div>
                  <p className="text-text-secondary">
                    {entry.explanation.cause}
                  </p>
                  <ul className="list-disc pl-5 text-text-secondary">
                    {entry.explanation.nextSteps.map((step, idx) => (
                      <li key={idx}>{step}</li>
                    ))}
                  </ul>
                </div>
              )}

              {entry.rawOutput && (
                <pre className="overflow-x-auto rounded-lg border border-border-default bg-canvas p-2.5 font-mono text-[11px] text-text-muted">
                  {entry.rawOutput}
                </pre>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
};
