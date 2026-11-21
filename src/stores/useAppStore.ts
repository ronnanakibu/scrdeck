import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { tauriApi } from "../services/tauri";
import type {
  ActivityLogEntry,
  AppSettings,
  CommandBuildResult,
  Device,
  EnvironmentStatus,
  MirrorSettings,
  NavPage,
  Preset,
  SessionInfo,
  ThemeMode,
} from "../types/scrdeck";

export const DEFAULT_MIRROR_SETTINGS: MirrorSettings = {
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
};

export function applyThemeToDocument(theme: ThemeMode) {
  const root = document.documentElement;
  if (theme === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", theme);
  }
}

interface AppStore {
  initialized: boolean;
  activePage: NavPage;
  environment: EnvironmentStatus | null;
  settings: AppSettings | null;
  devices: Device[];
  devicesLoading: boolean;
  devicesError: string | null;
  selectedDeviceSerial: string | null;
  presets: Preset[];
  activePresetId: string | null;
  mirrorSettings: MirrorSettings;
  allowUnknownFlags: boolean;
  commandPreview: CommandBuildResult | null;
  sessions: SessionInfo[];
  logs: ActivityLogEntry[];
  installingTools: boolean;
  installError: string | null;

  initializeApp: () => Promise<void>;
  setActivePage: (page: NavPage) => void;
  refreshToolchain: () => Promise<void>;
  installManagedTools: () => Promise<void>;
  refreshDevices: () => Promise<void>;
  selectDevice: (serial: string | null) => void;
  applyPreset: (preset: Preset) => void;
  updateMirrorSettings: (patch: Partial<MirrorSettings>) => void;
  setAllowUnknownFlags: (allow: boolean) => void;
  refreshCommandPreview: () => Promise<void>;
  startMirroring: () => Promise<SessionInfo>;
  stopMirroring: (sessionId: string) => Promise<void>;
  savePreset: (preset: Preset) => Promise<void>;
  deletePreset: (presetId: string) => Promise<void>;
  updateAppSettings: (patch: Partial<AppSettings>) => Promise<void>;
  refreshLogs: () => Promise<void>;
  clearLogs: () => Promise<void>;
}

export const useAppStore = create<AppStore>((set, get) => ({
  initialized: false,
  activePage: "overview",
  environment: null,
  settings: null,
  devices: [],
  devicesLoading: false,
  devicesError: null,
  selectedDeviceSerial: null,
  presets: [],
  activePresetId: "builtin-balanced",
  mirrorSettings: { ...DEFAULT_MIRROR_SETTINGS },
  allowUnknownFlags: false,
  commandPreview: null,
  sessions: [],
  logs: [],
  installingTools: false,
  installError: null,

  initializeApp: async () => {
    try {
      const [settings, env, presets, logs, sessions] = await Promise.all([
        tauriApi.getSettings(),
        tauriApi.detectToolchain(),
        tauriApi.getPresets(),
        tauriApi.getActivityLogs(),
        tauriApi.getActiveSessions(),
      ]);

      applyThemeToDocument(settings.theme);

      const needsSetup =
        !settings.setupCompleted ||
        !env.toolchain.scrcpyValid ||
        !env.toolchain.adbValid;

      const initialPage: NavPage = needsSetup
        ? "setup"
        : ((settings.lastActivePage as NavPage) || "overview");

      const balanced = presets.find((p) => p.id === "builtin-balanced");

      set({
        initialized: true,
        settings,
        environment: env,
        presets,
        logs,
        sessions,
        activePage: initialPage,
        selectedDeviceSerial: settings.lastSelectedDevice,
        mirrorSettings: balanced
          ? { ...balanced.settings }
          : { ...DEFAULT_MIRROR_SETTINGS },
      });

      if (env.toolchain.adbValid) {
        await get().refreshDevices();
      }
      await get().refreshCommandPreview();

      // Subscribe to backend events
      await listen<SessionInfo>("session-updated", (event) => {
        const updated = event.payload;
        set((state) => {
          const exists = state.sessions.some(
            (s) => s.sessionId === updated.sessionId
          );
          const nextSessions = exists
            ? state.sessions.map((s) =>
                s.sessionId === updated.sessionId ? updated : s
              )
            : [updated, ...state.sessions];
          return { sessions: nextSessions };
        });
      });

      await listen<ActivityLogEntry>("log-entry", (event) => {
        const entry = event.payload;
        set((state) => ({
          logs: [entry, ...state.logs].slice(
            0,
            state.settings?.maxLogEntries ?? 500
          ),
        }));
      });
    } catch (err) {
      console.error("Failed to initialize ScrDeck:", err);
      set({ initialized: true });
    }
  },

  setActivePage: (page) => {
    set({ activePage: page });
    const current = get().settings;
    if (current && page !== "setup") {
      const next = { ...current, lastActivePage: page };
      set({ settings: next });
      void tauriApi.updateSettings(next);
    }
  },

  refreshToolchain: async () => {
    const env = await tauriApi.detectToolchain();
    set({ environment: env });
    await get().refreshCommandPreview();
  },

  installManagedTools: async () => {
    set({ installingTools: true, installError: null });
    try {
      const env = await tauriApi.installManagedTools();
      set({ environment: env, installingTools: false });
      await get().refreshCommandPreview();
    } catch (err) {
      set({
        installingTools: false,
        installError: err instanceof Error ? err.message : String(err),
      });
    }
  },

  refreshDevices: async () => {
    set({ devicesLoading: true, devicesError: null });
    try {
      const devices = await tauriApi.getDevices();
      const currentSelected = get().selectedDeviceSerial;
      const stillPresent = devices.some((d) => d.serial === currentSelected);
      const nextSelected = stillPresent
        ? currentSelected
        : devices.length === 1
          ? devices[0].serial
          : null;

      set({
        devices,
        devicesLoading: false,
        selectedDeviceSerial: nextSelected,
      });
      await get().refreshCommandPreview();
    } catch (err) {
      set({
        devicesLoading: false,
        devicesError: err instanceof Error ? err.message : String(err),
      });
    }
  },

  selectDevice: (serial) => {
    set({ selectedDeviceSerial: serial });
    const current = get().settings;
    if (current) {
      const next = { ...current, lastSelectedDevice: serial };
      set({ settings: next });
      void tauriApi.updateSettings(next);
    }
    void get().refreshCommandPreview();
  },

  applyPreset: (preset) => {
    set({
      activePresetId: preset.id,
      mirrorSettings: { ...preset.settings },
    });
    void get().refreshCommandPreview();
  },

  updateMirrorSettings: (patch) => {
    set((state) => ({
      mirrorSettings: { ...state.mirrorSettings, ...patch },
    }));
    void get().refreshCommandPreview();
  },

  setAllowUnknownFlags: (allow) => {
    set({ allowUnknownFlags: allow });
    void get().refreshCommandPreview();
  },

  refreshCommandPreview: async () => {
    try {
      const { selectedDeviceSerial, mirrorSettings, allowUnknownFlags } = get();
      const preview = await tauriApi.buildScrcpyCommandPreview({
        deviceSerial: selectedDeviceSerial,
        settings: mirrorSettings,
        allowUnknownFlags,
      });
      set({ commandPreview: preview });
    } catch {
      // Ignore preview errors when backend is unavailable in unit tests
    }
  },

  startMirroring: async () => {
    const {
      selectedDeviceSerial,
      mirrorSettings,
      allowUnknownFlags,
      activePresetId,
    } = get();
    const session = await tauriApi.startMirroring(
      {
        deviceSerial: selectedDeviceSerial,
        settings: mirrorSettings,
        allowUnknownFlags,
      },
      activePresetId
    );
    set((state) => ({
      sessions: [session, ...state.sessions],
    }));
    return session;
  },

  stopMirroring: async (sessionId) => {
    const updated = await tauriApi.stopMirroring(sessionId);
    set((state) => ({
      sessions: state.sessions.map((s) =>
        s.sessionId === updated.sessionId ? updated : s
      ),
    }));
  },

  savePreset: async (preset) => {
    const presets = await tauriApi.savePreset(preset);
    set({ presets });
  },

  deletePreset: async (presetId) => {
    const presets = await tauriApi.deletePreset(presetId);
    set({ presets });
  },

  updateAppSettings: async (patch) => {
    const current = get().settings;
    if (!current) return;
    const next = { ...current, ...patch };
    applyThemeToDocument(next.theme);
    const saved = await tauriApi.updateSettings(next);
    set({ settings: saved });
  },

  refreshLogs: async () => {
    const logs = await tauriApi.getActivityLogs();
    set({ logs });
  },

  clearLogs: async () => {
    await tauriApi.clearActivityLogs();
    set({ logs: [] });
  },
}));
