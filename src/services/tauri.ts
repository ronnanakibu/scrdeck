import { invoke } from "@tauri-apps/api/core";
import type {
  ActivityLogEntry,
  AppSettings,
  CommandBuildRequest,
  CommandBuildResult,
  Device,
  DriverDiagnosis,
  EnvironmentStatus,
  Preset,
  SessionInfo,
} from "../types/scrdeck";

export const tauriApi = {
  getEnvironmentStatus: () =>
    invoke<EnvironmentStatus>("get_environment_status"),
  detectToolchain: () => invoke<EnvironmentStatus>("detect_toolchain"),
  installManagedTools: () => invoke<EnvironmentStatus>("install_managed_tools"),
  getDevices: () => invoke<Device[]>("get_devices"),
  startAdbServer: () => invoke<string>("start_adb_server"),
  restartAdbServerConfirmed: () =>
    invoke<string>("restart_adb_server_confirmed"),
  pairWirelessDevice: (address: string, pairingCode: string) =>
    invoke<string>("pair_wireless_device", { address, pairingCode }),
  connectWirelessDevice: (address: string) =>
    invoke<string>("connect_wireless_device", { address }),
  disconnectWirelessDevice: (serial: string) =>
    invoke<string>("disconnect_wireless_device", { serial }),
  diagnoseUsb: () => invoke<DriverDiagnosis>("diagnose_usb"),
  buildScrcpyCommandPreview: (request: CommandBuildRequest) =>
    invoke<CommandBuildResult>("build_scrcpy_command_preview", { request }),
  parseCustomArgsString: (input: string) =>
    invoke<string[]>("parse_custom_args_string", { input }),
  startMirroring: (request: CommandBuildRequest, presetId: string | null) =>
    invoke<SessionInfo>("start_mirroring", { request, presetId }),
  stopMirroring: (sessionId: string) =>
    invoke<SessionInfo>("stop_mirroring", { sessionId }),
  getActiveSessions: () => invoke<SessionInfo[]>("get_active_sessions"),
  getPresets: () => invoke<Preset[]>("get_presets"),
  savePreset: (preset: Preset) => invoke<Preset[]>("save_preset", { preset }),
  deletePreset: (presetId: string) =>
    invoke<Preset[]>("delete_preset", { presetId }),
  importPresetJson: (jsonContent: string) =>
    invoke<Preset>("import_preset_json", { jsonContent }),
  exportPresetJson: (presetId: string) =>
    invoke<string>("export_preset_json", { presetId }),
  getSettings: () => invoke<AppSettings>("get_settings"),
  updateSettings: (settings: AppSettings) =>
    invoke<AppSettings>("update_settings", { settings }),
  getActivityLogs: () => invoke<ActivityLogEntry[]>("get_activity_logs"),
  clearActivityLogs: () => invoke<void>("clear_activity_logs"),
  exportDiagnosticsBundle: (redactIdentifiers: boolean) =>
    invoke<string>("export_diagnostics_bundle", {
      options: { redactIdentifiers },
    }),
};
