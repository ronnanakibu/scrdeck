use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ToolchainSourceType {
    ManagedBundle,
    UserSelected,
    DetectedSdk,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IntegrityStatus {
    VerifiedChecksum,
    OfficialSourceNoChecksum,
    UserProvided,
    Unverified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainInfo {
    pub scrcpy_path: Option<String>,
    pub scrcpy_version: Option<String>,
    pub adb_path: Option<String>,
    pub adb_version: Option<String>,
    pub source_type: ToolchainSourceType,
    pub architecture: String,
    pub last_validated_at: Option<String>,
    pub integrity_status: IntegrityStatus,
    pub scrcpy_valid: bool,
    pub adb_valid: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DeviceTransport {
    Usb,
    TcpIp,
    Emulator,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AdbState {
    Device,
    Unauthorized,
    Offline,
    NoPermission,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub serial: String,
    pub model: Option<String>,
    pub product: Option<String>,
    pub device_code: Option<String>,
    pub transport_id: Option<String>,
    pub transport: DeviceTransport,
    pub adb_state: AdbState,
    pub android_release: Option<String>,
    pub api_level: Option<u32>,
    pub battery_percent: Option<u8>,
    pub charging_state: Option<String>,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorSettings {
    pub max_size: Option<u32>,
    pub max_fps: Option<u32>,
    pub video_bit_rate: Option<String>,
    pub video_codec: Option<String>,
    pub audio_enabled: bool,
    pub audio_source: Option<String>,
    pub audio_codec: Option<String>,
    pub fullscreen: bool,
    pub always_on_top: bool,
    pub stay_awake: bool,
    pub turn_screen_off: bool,
    pub control_enabled: bool,
    pub show_touches: bool,
    pub window_title: Option<String>,
    pub rotation: Option<u32>,
    #[serde(default)]
    pub crop: Option<String>,
    pub recording_enabled: bool,
    pub recording_path: Option<String>,
    pub recording_format: Option<String>,
    #[serde(default)]
    pub custom_args: Vec<String>,
}

impl Default for MirrorSettings {
    fn default() -> Self {
        Self {
            max_size: Some(1920),
            max_fps: Some(60),
            video_bit_rate: None,
            video_codec: Some("h264".to_string()),
            audio_enabled: true,
            audio_source: None,
            audio_codec: None,
            fullscreen: false,
            always_on_top: false,
            stay_awake: true,
            turn_screen_off: false,
            control_enabled: true,
            show_touches: false,
            window_title: None,
            rotation: None,
            crop: None,
            recording_enabled: false,
            recording_path: None,
            recording_format: None,
            custom_args: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
    pub settings: MirrorSettings,
    #[serde(default)]
    pub compatibility_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetImportPreview {
    pub preset: Preset,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrcpyCapabilities {
    pub scrcpy_version: Option<String>,
    pub supported_flags: Vec<String>,
    pub supports_audio: bool,
    pub supports_video_codec: bool,
    pub supports_max_fps: bool,
    pub supports_max_size: bool,
    pub supports_record: bool,
    pub supports_turn_screen_off: bool,
    pub supports_stay_awake: bool,
    pub supports_no_control: bool,
    pub supports_always_on_top: bool,
    pub supports_fullscreen: bool,
    pub supports_show_touches: bool,
    pub supports_window_title: bool,
    pub supports_orientation: bool,
    pub parsed_successfully: bool,
}

impl Default for ScrcpyCapabilities {
    fn default() -> Self {
        Self {
            scrcpy_version: None,
            supported_flags: Vec::new(),
            supports_audio: true,
            supports_video_codec: true,
            supports_max_fps: true,
            supports_max_size: true,
            supports_record: true,
            supports_turn_screen_off: true,
            supports_stay_awake: true,
            supports_no_control: true,
            supports_always_on_top: true,
            supports_fullscreen: true,
            supports_show_touches: true,
            supports_window_title: true,
            supports_orientation: true,
            parsed_successfully: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandBuildRequest {
    pub device_serial: Option<String>,
    pub settings: MirrorSettings,
    pub allow_unknown_flags: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandConflict {
    pub flag: String,
    pub reason: String,
    pub severity: String, // "error" | "warning"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandBuildResult {
    pub executable: String,
    pub argv: Vec<String>,
    pub preview: String,
    pub conflicts: Vec<CommandConflict>,
    pub unknown_flags: Vec<String>,
    pub is_valid: bool,
    pub requires_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionStatus {
    Idle,
    Validating,
    Blocked,
    Starting,
    Running,
    Stopping,
    Exited,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub session_id: String,
    pub device_serial: String,
    pub preset_id: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub process_id: Option<u32>,
    pub executable_path: String,
    pub argv: Vec<String>,
    pub exit_code: Option<i32>,
    pub status: SessionStatus,
    pub error_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LogSeverity {
    Info,
    Success,
    Warning,
    Error,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLogEntry {
    pub id: String,
    pub timestamp: String,
    pub source: String, // "app" | "adb" | "scrcpy" | "toolchain"
    pub device_serial: Option<String>,
    pub session_id: Option<String>,
    pub severity: LogSeverity,
    pub title: String,
    pub message: String,
    pub raw_output: Option<String>,
    pub explanation: Option<DiagnosticExplanation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticExplanation {
    pub code: String,
    pub title: String,
    pub cause: String,
    pub confidence: String, // "high" | "medium" | "possible"
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThemeMode {
    System,
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AppClosePolicy {
    StopScrDeckSessions,
    KeepSessionsRunning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub schema_version: u32,
    pub theme: ThemeMode,
    pub setup_completed: bool,
    pub last_active_page: String,
    pub last_selected_device: Option<String>,
    pub custom_scrcpy_path: Option<String>,
    pub custom_adb_path: Option<String>,
    pub app_close_policy: AppClosePolicy,
    pub max_log_entries: usize,
    pub redact_diagnostics_by_default: bool,
    #[serde(default)]
    pub per_device_presets: std::collections::HashMap<String, String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            theme: ThemeMode::Dark,
            setup_completed: false,
            last_active_page: "overview".to_string(),
            last_selected_device: None,
            custom_scrcpy_path: None,
            custom_adb_path: None,
            app_close_policy: AppClosePolicy::StopScrDeckSessions,
            max_log_entries: 500,
            redact_diagnostics_by_default: true,
            per_device_presets: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub os_name: String,
    pub os_arch: String,
    pub is_windows_10_or_older_caveat: bool,
    pub toolchain: ToolchainInfo,
    pub capabilities: ScrcpyCapabilities,
    pub adb_server_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OemDriverLink {
    pub vendor: String,
    pub label: String,
    pub url: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverDiagnosis {
    pub status: String,
    pub title: String,
    pub confidence: String,
    pub summary: String,
    pub checklist: Vec<String>,
    pub oem_links: Vec<OemDriverLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticExportOptions {
    pub redact_identifiers: bool,
}
