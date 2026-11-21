use crate::models::{
    ActivityLogEntry, AppSettings, CommandBuildRequest, CommandBuildResult, Device,
    DiagnosticExportOptions, DriverDiagnosis, EnvironmentStatus, LogSeverity, Preset,
    ScrcpyCapabilities, SessionInfo, ToolchainInfo,
};
use crate::parsers::{parse_windows_cmdline, redact_sensitive_text};
use crate::process::ProcessManager;
use crate::services::{adb, command_builder, toolchain};
use crate::storage;
use chrono::Utc;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub toolchain: Arc<Mutex<ToolchainInfo>>,
    pub capabilities: Arc<Mutex<ScrcpyCapabilities>>,
    pub presets: Arc<Mutex<Vec<Preset>>>,
    pub logs: Arc<Mutex<Vec<ActivityLogEntry>>>,
    pub process_manager: ProcessManager,
}

async fn append_log(
    app: &AppHandle,
    state: &AppState,
    source: &str,
    severity: LogSeverity,
    title: &str,
    message: &str,
    device_serial: Option<String>,
    raw_output: Option<String>,
) {
    let entry = ActivityLogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        source: source.to_string(),
        device_serial,
        session_id: None,
        severity,
        title: title.to_string(),
        message: message.to_string(),
        raw_output,
        explanation: None,
    };
    let max_entries = {
        let s = state.settings.lock().await;
        s.max_log_entries
    };
    {
        let mut logs = state.logs.lock().await;
        logs.insert(0, entry.clone());
        if logs.len() > max_entries {
            logs.truncate(max_entries);
        }
        let _ = storage::save_logs(&logs, max_entries);
    }
    let _ = app.emit("log-entry", &entry);
}

#[tauri::command]
pub async fn get_environment_status(state: State<'_, AppState>) -> Result<EnvironmentStatus, String> {
    let tc = state.toolchain.lock().await.clone();
    let caps = state.capabilities.lock().await.clone();
    Ok(EnvironmentStatus {
        os_name: std::env::consts::OS.to_string(),
        os_arch: std::env::consts::ARCH.to_string(),
        is_windows_10_or_older_caveat: false,
        toolchain: tc,
        capabilities: caps,
        adb_server_running: false,
    })
}

#[tauri::command]
pub async fn detect_toolchain(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<EnvironmentStatus, String> {
    let (custom_scrcpy, custom_adb) = {
        let s = state.settings.lock().await;
        (s.custom_scrcpy_path.clone(), s.custom_adb_path.clone())
    };

    let (tc, caps) =
        toolchain::detect_toolchain(custom_scrcpy.as_deref(), custom_adb.as_deref()).await;

    {
        let mut tc_guard = state.toolchain.lock().await;
        *tc_guard = tc.clone();
    }
    {
        let mut caps_guard = state.capabilities.lock().await;
        *caps_guard = caps.clone();
    }

    append_log(
        &app,
        &state,
        "toolchain",
        if tc.scrcpy_valid && tc.adb_valid {
            LogSeverity::Success
        } else {
            LogSeverity::Warning
        },
        "Toolchain checked",
        &format!(
            "scrcpy: {} | adb: {}",
            tc.scrcpy_version.as_deref().unwrap_or("missing"),
            tc.adb_version.as_deref().unwrap_or("missing")
        ),
        None,
        None,
    )
    .await;

    Ok(EnvironmentStatus {
        os_name: std::env::consts::OS.to_string(),
        os_arch: std::env::consts::ARCH.to_string(),
        is_windows_10_or_older_caveat: false,
        toolchain: tc,
        capabilities: caps,
        adb_server_running: false,
    })
}

#[tauri::command]
pub async fn install_managed_tools(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<EnvironmentStatus, String> {
    append_log(
        &app,
        &state,
        "toolchain",
        LogSeverity::Info,
        "Downloading official scrcpy bundle",
        "Fetching latest Windows x64 release from github.com/Genymobile/scrcpy...",
        None,
        None,
    )
    .await;

    let (tag, sha256, _integrity) = toolchain::install_official_scrcpy_bundle().await?;

    append_log(
        &app,
        &state,
        "toolchain",
        LogSeverity::Success,
        "Official scrcpy bundle installed",
        &format!(
            "Installed {} (SHA-256: {}). Checksum not provided by upstream; authenticity relies on the official HTTPS source.",
            tag, sha256
        ),
        None,
        None,
    )
    .await;

    detect_toolchain(app, state).await
}

#[tauri::command]
pub async fn get_devices(state: State<'_, AppState>) -> Result<Vec<Device>, String> {
    let adb_path = {
        let tc = state.toolchain.lock().await;
        tc.adb_path.clone()
    };
    let adb_exe = adb_path.ok_or_else(|| "ADB executable is not configured or missing.".to_string())?;
    adb::get_devices(&adb_exe).await
}

#[tauri::command]
pub async fn start_adb_server(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let adb_exe = {
        let tc = state.toolchain.lock().await;
        tc.adb_path.clone()
    }
    .ok_or_else(|| "ADB executable is not available.".to_string())?;

    let res = adb::start_server(&adb_exe).await?;
    append_log(
        &app,
        &state,
        "adb",
        LogSeverity::Info,
        "ADB server checked/started",
        if res.is_empty() { "ADB server is running." } else { &res },
        None,
        None,
    )
    .await;
    Ok(res)
}

#[tauri::command]
pub async fn restart_adb_server_confirmed(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let adb_exe = {
        let tc = state.toolchain.lock().await;
        tc.adb_path.clone()
    }
    .ok_or_else(|| "ADB executable is not available.".to_string())?;

    let res = adb::restart_server_confirmed(&adb_exe).await?;
    append_log(
        &app,
        &state,
        "adb",
        LogSeverity::Warning,
        "ADB server restarted by user",
        "Shared ADB server was restarted after user confirmation.",
        None,
        Some(res.clone()),
    )
    .await;
    Ok(res)
}

#[tauri::command]
pub async fn pair_wireless_device(
    app: AppHandle,
    state: State<'_, AppState>,
    address: String,
    pairing_code: String,
) -> Result<String, String> {
    let adb_exe = {
        let tc = state.toolchain.lock().await;
        tc.adb_path.clone()
    }
    .ok_or_else(|| "ADB executable is not available.".to_string())?;

    let res = adb::pair_wireless(&adb_exe, &address, &pairing_code).await;
    match &res {
        Ok(msg) => {
            append_log(
                &app,
                &state,
                "adb",
                LogSeverity::Success,
                "Wireless pairing succeeded",
                msg,
                Some(address),
                None,
            )
            .await;
        }
        Err(err) => {
            append_log(
                &app,
                &state,
                "adb",
                LogSeverity::Error,
                "Wireless pairing failed",
                err,
                Some(address),
                None,
            )
            .await;
        }
    }
    res
}

#[tauri::command]
pub async fn connect_wireless_device(
    app: AppHandle,
    state: State<'_, AppState>,
    address: String,
) -> Result<String, String> {
    let adb_exe = {
        let tc = state.toolchain.lock().await;
        tc.adb_path.clone()
    }
    .ok_or_else(|| "ADB executable is not available.".to_string())?;

    let res = adb::connect_wireless(&adb_exe, &address).await;
    match &res {
        Ok(msg) => {
            append_log(
                &app,
                &state,
                "adb",
                LogSeverity::Success,
                "Wireless device connected",
                msg,
                Some(address),
                None,
            )
            .await;
        }
        Err(err) => {
            append_log(
                &app,
                &state,
                "adb",
                LogSeverity::Error,
                "Wireless connection failed",
                err,
                Some(address),
                None,
            )
            .await;
        }
    }
    res
}

#[tauri::command]
pub async fn disconnect_wireless_device(
    app: AppHandle,
    state: State<'_, AppState>,
    serial: String,
) -> Result<String, String> {
    let adb_exe = {
        let tc = state.toolchain.lock().await;
        tc.adb_path.clone()
    }
    .ok_or_else(|| "ADB executable is not available.".to_string())?;

    let res = adb::disconnect_wireless(&adb_exe, &serial).await?;
    append_log(
        &app,
        &state,
        "adb",
        LogSeverity::Info,
        "Wireless device disconnected",
        &res,
        Some(serial),
        None,
    )
    .await;
    Ok(res)
}

#[tauri::command]
pub async fn diagnose_usb(state: State<'_, AppState>) -> Result<DriverDiagnosis, String> {
    let adb_exe = {
        let tc = state.toolchain.lock().await;
        tc.adb_path.clone()
    };
    Ok(adb::diagnose_usb_connection(adb_exe.as_deref()).await)
}

#[tauri::command]
pub async fn build_scrcpy_command_preview(
    state: State<'_, AppState>,
    request: CommandBuildRequest,
) -> Result<CommandBuildResult, String> {
    let exe = {
        let tc = state.toolchain.lock().await;
        tc.scrcpy_path
            .clone()
            .unwrap_or_else(|| "scrcpy.exe".to_string())
    };
    let caps = state.capabilities.lock().await.clone();
    Ok(command_builder::build_scrcpy_command(&exe, &request, &caps))
}

#[tauri::command]
pub async fn parse_custom_args_string(input: String) -> Result<Vec<String>, String> {
    parse_windows_cmdline(&input)
}

#[tauri::command]
pub async fn start_mirroring(
    app: AppHandle,
    state: State<'_, AppState>,
    request: CommandBuildRequest,
    preset_id: Option<String>,
) -> Result<SessionInfo, String> {
    let tc = state.toolchain.lock().await.clone();
    let exe = tc
        .scrcpy_path
        .ok_or_else(|| "scrcpy.exe is not configured or missing.".to_string())?;
    if !tc.scrcpy_valid {
        return Err("scrcpy.exe could not be validated. Check Settings -> Toolchain.".to_string());
    }

    let caps = state.capabilities.lock().await.clone();
    let built = command_builder::build_scrcpy_command(&exe, &request, &caps);
    if !built.is_valid {
        let msg = built
            .conflicts
            .first()
            .map(|c| c.reason.clone())
            .unwrap_or_else(|| "Command validation failed or requires unknown-flag override.".to_string());
        return Err(msg);
    }

    let serial = request
        .device_serial
        .ok_or_else(|| "No target device selected.".to_string())?;

    state
        .process_manager
        .start_session(
            app,
            built.executable,
            built.argv,
            serial,
            preset_id,
            state.logs.clone(),
        )
        .await
}

#[tauri::command]
pub async fn stop_mirroring(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<SessionInfo, String> {
    state.process_manager.stop_session(&app, &session_id).await
}

#[tauri::command]
pub async fn get_active_sessions(state: State<'_, AppState>) -> Result<Vec<SessionInfo>, String> {
    Ok(state.process_manager.list_sessions().await)
}

#[tauri::command]
pub async fn get_presets(state: State<'_, AppState>) -> Result<Vec<Preset>, String> {
    Ok(state.presets.lock().await.clone())
}

#[tauri::command]
pub async fn save_preset(
    state: State<'_, AppState>,
    mut preset: Preset,
) -> Result<Vec<Preset>, String> {
    if preset.is_builtin {
        return Err("Built-in presets cannot be overwritten directly. Use 'Save as' instead.".to_string());
    }
    let now = Utc::now().to_rfc3339();
    if preset.id.trim().is_empty() {
        preset.id = format!("user-{}", Uuid::new_v4());
        preset.created_at = now.clone();
    }
    preset.updated_at = now;
    preset.schema_version = 1;

    let mut guard = state.presets.lock().await;
    if let Some(existing) = guard.iter_mut().find(|p| p.id == preset.id) {
        *existing = preset;
    } else {
        guard.push(preset);
    }
    storage::save_user_presets(&guard)?;
    Ok(guard.clone())
}

#[tauri::command]
pub async fn delete_preset(
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<Vec<Preset>, String> {
    let mut guard = state.presets.lock().await;
    if let Some(target) = guard.iter().find(|p| p.id == preset_id) {
        if target.is_builtin {
            return Err("Built-in presets cannot be deleted.".to_string());
        }
    }
    guard.retain(|p| p.id != preset_id);
    storage::save_user_presets(&guard)?;
    Ok(guard.clone())
}

#[tauri::command]
pub async fn import_preset_json(
    state: State<'_, AppState>,
    json_content: String,
) -> Result<Preset, String> {
    let mut parsed: Preset = serde_json::from_str(&json_content)
        .map_err(|e| format!("Invalid preset JSON format: {}", e))?;
    if parsed.schema_version == 0 {
        return Err("Preset JSON is missing a valid schemaVersion.".to_string());
    }
    parsed.is_builtin = false;
    parsed.id = format!("user-{}", Uuid::new_v4());
    let now = Utc::now().to_rfc3339();
    parsed.created_at = now.clone();
    parsed.updated_at = now;

    let mut guard = state.presets.lock().await;
    guard.push(parsed.clone());
    storage::save_user_presets(&guard)?;
    Ok(parsed)
}

#[tauri::command]
pub async fn export_preset_json(
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<String, String> {
    let guard = state.presets.lock().await;
    let preset = guard
        .iter()
        .find(|p| p.id == preset_id)
        .ok_or_else(|| "Preset not found.".to_string())?;
    serde_json::to_string_pretty(preset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    Ok(state.settings.lock().await.clone())
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    storage::save_settings(&settings)?;
    let mut guard = state.settings.lock().await;
    *guard = settings.clone();
    Ok(settings)
}

#[tauri::command]
pub async fn get_activity_logs(state: State<'_, AppState>) -> Result<Vec<ActivityLogEntry>, String> {
    Ok(state.logs.lock().await.clone())
}

#[tauri::command]
pub async fn clear_activity_logs(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.logs.lock().await;
    guard.clear();
    storage::save_logs(&guard, 0)?;
    Ok(())
}

#[tauri::command]
pub async fn export_diagnostics_bundle(
    state: State<'_, AppState>,
    options: DiagnosticExportOptions,
) -> Result<String, String> {
    let tc = state.toolchain.lock().await.clone();
    let logs = state.logs.lock().await.clone();
    let serials: Vec<String> = logs
        .iter()
        .filter_map(|l| l.device_serial.clone())
        .collect();

    let bundle = serde_json::json!({
        "scrdeckVersion": "0.1.0",
        "exportedAt": Utc::now().to_rfc3339(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "toolchain": tc,
        "logs": logs,
    });

    let raw = serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())?;
    if options.redact_identifiers {
        Ok(redact_sensitive_text(&raw, &serials))
    } else {
        Ok(raw)
    }
}
