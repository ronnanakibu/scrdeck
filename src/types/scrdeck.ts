export type NavPage =
  | "overview"
  | "devices"
  | "mirroring"
  | "presets"
  | "adb-tools"
  | "activity"
  | "settings"
  | "about"
  | "setup";

export type ToolchainSourceType =
  | "managedBundle"
  | "userSelected"
  | "detectedSdk"
  | "missing";

export type IntegrityStatus =
  | "verifiedChecksum"
  | "officialSourceNoChecksum"
  | "userProvided"
  | "unverified";

export interface ToolchainInfo {
  scrcpyPath: string | null;
  scrcpyVersion: string | null;
  adbPath: string | null;
  adbVersion: string | null;
  sourceType: ToolchainSourceType;
  architecture: string;
  lastValidatedAt: string | null;
  integrityStatus: IntegrityStatus;
  scrcpyValid: boolean;
  adbValid: boolean;
  notes: string[];
}

export type DeviceTransport = "usb" | "tcpIp" | "emulator" | "unknown";
export type AdbState =
  | "device"
  | "unauthorized"
  | "offline"
  | "noPermission"
  | "unknown";

export interface Device {
  serial: string;
  model: string | null;
  product: string | null;
  deviceCode: string | null;
  transportId: string | null;
  transport: DeviceTransport;
  adbState: AdbState;
  androidRelease: string | null;
  apiLevel: number | null;
  batteryPercent: number | null;
  chargingState: string | null;
  lastSeenAt: string;
}

export interface MirrorSettings {
  maxSize: number | null;
  maxFps: number | null;
  videoBitRate: string | null;
  videoCodec: string | null;
  audioEnabled: boolean;
  audioSource: string | null;
  audioCodec: string | null;
  fullscreen: boolean;
  alwaysOnTop: boolean;
  stayAwake: boolean;
  turnScreenOff: boolean;
  controlEnabled: boolean;
  showTouches: boolean;
  windowTitle: string | null;
  rotation: number | null;
  recordingEnabled: boolean;
  recordingPath: string | null;
  recordingFormat: string | null;
  customArgs: string[];
}

export interface Preset {
  schemaVersion: number;
  id: string;
  name: string;
  description: string;
  isBuiltin: boolean;
  createdAt: string;
  updatedAt: string;
  settings: MirrorSettings;
  compatibilityNotes: string[];
}

export interface ScrcpyCapabilities {
  scrcpyVersion: string | null;
  supportedFlags: string[];
  supportsAudio: boolean;
  supportsVideoCodec: boolean;
  supportsMaxFps: boolean;
  supportsMaxSize: boolean;
  supportsRecord: boolean;
  supportsTurnScreenOff: boolean;
  supportsStayAwake: boolean;
  supportsNoControl: boolean;
  supportsAlwaysOnTop: boolean;
  supportsFullscreen: boolean;
  supportsShowTouches: boolean;
  supportsWindowTitle: boolean;
  supportsOrientation: boolean;
  parsedSuccessfully: boolean;
}

export interface CommandBuildRequest {
  deviceSerial: string | null;
  settings: MirrorSettings;
  allowUnknownFlags: boolean;
}

export interface CommandConflict {
  flag: string;
  reason: string;
  severity: "error" | "warning";
}

export interface CommandBuildResult {
  executable: string;
  argv: string[];
  preview: string;
  conflicts: CommandConflict[];
  unknownFlags: string[];
  isValid: boolean;
  requiresOverride: boolean;
}

export type SessionStatus =
  | "idle"
  | "validating"
  | "blocked"
  | "starting"
  | "running"
  | "stopping"
  | "exited"
  | "failed";

export interface SessionInfo {
  sessionId: string;
  deviceSerial: string;
  presetId: string | null;
  startedAt: string;
  endedAt: string | null;
  processId: number | null;
  executablePath: string;
  argv: string[];
  exitCode: number | null;
  status: SessionStatus;
  errorSummary: string | null;
}

export type LogSeverity = "info" | "success" | "warning" | "error" | "debug";

export interface DiagnosticExplanation {
  code: string;
  title: string;
  cause: string;
  confidence: string;
  nextSteps: string[];
}

export interface ActivityLogEntry {
  id: string;
  timestamp: string;
  source: string;
  deviceSerial: string | null;
  sessionId: string | null;
  severity: LogSeverity;
  title: string;
  message: string;
  rawOutput: string | null;
  explanation: DiagnosticExplanation | null;
}

export type ThemeMode = "system" | "dark" | "light";
export type AppClosePolicy = "stopScrDeckSessions" | "keepSessionsRunning";

export interface AppSettings {
  schemaVersion: number;
  theme: ThemeMode;
  setupCompleted: boolean;
  lastActivePage: string;
  lastSelectedDevice: string | null;
  customScrcpyPath: string | null;
  customAdbPath: string | null;
  appClosePolicy: AppClosePolicy;
  maxLogEntries: number;
  redactDiagnosticsByDefault: boolean;
  perDevicePresets: Record<string, string>;
}

export interface EnvironmentStatus {
  osName: string;
  osArch: string;
  isWindows10OrOlderCaveat: boolean;
  toolchain: ToolchainInfo;
  capabilities: ScrcpyCapabilities;
  adbServerRunning: boolean;
}

export interface OemDriverLink {
  vendor: string;
  label: string;
  url: string;
  notes: string;
}

export interface DriverDiagnosis {
  status: string;
  title: string;
  confidence: string;
  summary: string;
  checklist: string[];
  oemLinks: OemDriverLink[];
}
