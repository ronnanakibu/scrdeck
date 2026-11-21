import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, act } from "@testing-library/react";
import App from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    switch (cmd) {
      case "get_settings":
        return {
          schemaVersion: 1,
          theme: "dark",
          setupCompleted: true,
          lastActivePage: "overview",
          lastSelectedDevice: null,
          customScrcpyPath: null,
          customAdbPath: null,
          appClosePolicy: "stopScrDeckSessions",
          maxLogEntries: 500,
          redactDiagnosticsByDefault: true,
          perDevicePresets: {},
        };
      case "detect_toolchain":
        return {
          osName: "windows",
          osArch: "x86_64",
          isWindows10OrOlderCaveat: false,
          toolchain: {
            scrcpyPath: "C:\\tools\\scrcpy.exe",
            scrcpyVersion: "5.0.1",
            adbPath: "C:\\tools\\adb.exe",
            adbVersion: "1.0.41",
            sourceType: "managedBundle",
            architecture: "x86_64",
            lastValidatedAt: "2026-11-18T00:00:00Z",
            integrityStatus: "officialSourceNoChecksum",
            scrcpyValid: true,
            adbValid: true,
            notes: [],
          },
          capabilities: {
            scrcpyVersion: "5.0.1",
            supportedFlags: ["--serial", "--max-size", "--max-fps"],
            supportsAudio: true,
            supportsVideoCodec: true,
            supportsMaxFps: true,
            supportsMaxSize: true,
            supportsRecord: true,
            supportsTurnScreenOff: true,
            supportsStayAwake: true,
            supportsNoControl: true,
            supportsAlwaysOnTop: true,
            supportsFullscreen: true,
            supportsShowTouches: true,
            supportsWindowTitle: true,
            supportsOrientation: true,
            parsedSuccessfully: true,
          },
          adbServerRunning: true,
        };
      case "get_presets":
        return [
          {
            schemaVersion: 1,
            id: "builtin-balanced",
            name: "Balanced",
            description: "Everyday mirroring profile.",
            isBuiltin: true,
            createdAt: "2026-11-18T00:00:00Z",
            updatedAt: "2026-11-18T00:00:00Z",
            settings: {
              maxSize: 1920,
              maxFps: 60,
              videoBitRate: null,
              videoCodec: "h264",
              audioEnabled: true,
              audioSource: null,
              audioCodec: null,
              fullscreen: false,
              alwaysOnTop: false,
              stayAwake: true,
              turnScreenOff: false,
              controlEnabled: true,
              showTouches: false,
              windowTitle: null,
              rotation: null,
              recordingEnabled: false,
              recordingPath: null,
              recordingFormat: null,
              customArgs: [],
            },
            compatibilityNotes: [],
          },
        ];
      case "get_activity_logs":
      case "get_active_sessions":
      case "get_devices":
        return [];
      case "build_scrcpy_command_preview":
        return {
          executable: "C:\\tools\\scrcpy.exe",
          argv: ["--max-size=1920"],
          preview: "C:\\tools\\scrcpy.exe --max-size=1920",
          conflicts: [],
          unknownFlags: [],
          isValid: false,
          requiresOverride: false,
        };
      default:
        return null;
    }
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));

describe("ScrDeck App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders Overview with system readiness and empty device state", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText("Tools Ready")).toBeInTheDocument();
    });
    expect(screen.getByText("v5.0.1")).toBeInTheDocument();
    expect(screen.getByText("No Android device selected")).toBeInTheDocument();
  });

  it("navigates across sidebar pages smoothly", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText("Tools Ready")).toBeInTheDocument();
    });

    // Navigate to Devices
    const devicesBtn = screen.getByRole("button", { name: "Devices" });
    await act(async () => {
      devicesBtn.click();
    });
    await waitFor(() => {
      expect(screen.getByText("Connected Android Devices")).toBeInTheDocument();
    });

    // Navigate to Presets
    const presetsBtn = screen.getByRole("button", { name: "Presets" });
    await act(async () => {
      presetsBtn.click();
    });
    await waitFor(() => {
      expect(screen.getByText("Import Preset JSON")).toBeInTheDocument();
    });

    // Navigate to About
    const aboutBtn = screen.getByRole("button", { name: "About ScrDeck" });
    await act(async () => {
      aboutBtn.click();
    });
    await waitFor(() => {
      expect(screen.getByText("Upstream Attribution & Disclaimer")).toBeInTheDocument();
    });
  });
});
