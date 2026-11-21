pub mod commands;
pub mod models;
pub mod parsers;
pub mod process;
pub mod services;
pub mod storage;

use commands::AppState;
use models::{AppClosePolicy, ScrcpyCapabilities, ToolchainInfo, ToolchainSourceType, IntegrityStatus};
use process::ProcessManager;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_settings = storage::load_settings();
    let initial_presets = storage::load_presets();
    let initial_logs = storage::load_logs();

    let initial_toolchain = ToolchainInfo {
        scrcpy_path: None,
        scrcpy_version: None,
        adb_path: None,
        adb_version: None,
        source_type: ToolchainSourceType::Missing,
        architecture: std::env::consts::ARCH.to_string(),
        last_validated_at: None,
        integrity_status: IntegrityStatus::Unverified,
        scrcpy_valid: false,
        adb_valid: false,
        notes: Vec::new(),
    };

    let app_state = AppState {
        settings: Arc::new(Mutex::new(initial_settings)),
        toolchain: Arc::new(Mutex::new(initial_toolchain)),
        capabilities: Arc::new(Mutex::new(ScrcpyCapabilities::default())),
        presets: Arc::new(Mutex::new(initial_presets)),
        logs: Arc::new(Mutex::new(initial_logs)),
        process_manager: ProcessManager::new(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_environment_status,
            commands::detect_toolchain,
            commands::install_managed_tools,
            commands::get_devices,
            commands::start_adb_server,
            commands::restart_adb_server_confirmed,
            commands::pair_wireless_device,
            commands::connect_wireless_device,
            commands::disconnect_wireless_device,
            commands::diagnose_usb,
            commands::build_scrcpy_command_preview,
            commands::parse_custom_args_string,
            commands::start_mirroring,
            commands::stop_mirroring,
            commands::get_active_sessions,
            commands::get_presets,
            commands::save_preset,
            commands::delete_preset,
            commands::import_preset_json,
            commands::export_preset_json,
            commands::get_settings,
            commands::update_settings,
            commands::get_activity_logs,
            commands::clear_activity_logs,
            commands::export_diagnostics_bundle,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let state = window.state::<AppState>();
                let policy = tauri::async_runtime::block_on(async {
                    state.settings.lock().await.app_close_policy.clone()
                });
                if policy == AppClosePolicy::StopScrDeckSessions {
                    tauri::async_runtime::block_on(async {
                        state.process_manager.stop_all_sessions().await;
                    });
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running ScrDeck");
}
