use crate::models::{ActivityLogEntry, AppSettings, MirrorSettings, Preset};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

pub fn app_data_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("ScrDeck");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub fn presets_path() -> PathBuf {
    app_data_dir().join("presets.json")
}

pub fn logs_path() -> PathBuf {
    app_data_dir().join("logs.json")
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(parsed) = serde_json::from_str::<AppSettings>(&raw) {
            return parsed;
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn builtin_presets() -> Vec<Preset> {
    let now = Utc::now().to_rfc3339();
    vec![
        Preset {
            schema_version: 1,
            id: "builtin-balanced".to_string(),
            name: "Balanced".to_string(),
            description: "Everyday mirroring profile with moderate resolution, 60 FPS, and audio.".to_string(),
            is_builtin: true,
            created_at: now.clone(),
            updated_at: now.clone(),
            settings: MirrorSettings {
                max_size: Some(1920),
                max_fps: Some(60),
                video_codec: Some("h264".to_string()),
                audio_enabled: true,
                stay_awake: true,
                control_enabled: true,
                ..Default::default()
            },
            compatibility_notes: vec!["Audio forwarding requires Android 11 (API 30) or newer.".to_string()],
        },
        Preset {
            schema_version: 1,
            id: "builtin-low-latency".to_string(),
            name: "Low Latency".to_string(),
            description: "Responsive control with reduced resolution overhead.".to_string(),
            is_builtin: true,
            created_at: now.clone(),
            updated_at: now.clone(),
            settings: MirrorSettings {
                max_size: Some(1280),
                max_fps: Some(60),
                video_bit_rate: Some("4M".to_string()),
                video_codec: Some("h264".to_string()),
                audio_enabled: false,
                stay_awake: true,
                control_enabled: true,
                ..Default::default()
            },
            compatibility_notes: vec!["Audio disabled by default to minimize encoding overhead.".to_string()],
        },
        Preset {
            schema_version: 1,
            id: "builtin-high-quality".to_string(),
            name: "High Quality".to_string(),
            description: "Higher target resolution and bitrate for visual inspection.".to_string(),
            is_builtin: true,
            created_at: now.clone(),
            updated_at: now.clone(),
            settings: MirrorSettings {
                max_size: None,
                max_fps: Some(60),
                video_bit_rate: Some("16M".to_string()),
                video_codec: Some("h264".to_string()),
                audio_enabled: true,
                stay_awake: true,
                control_enabled: true,
                ..Default::default()
            },
            compatibility_notes: vec!["Higher bitrates depend on USB bandwidth and device encoder capability.".to_string()],
        },
        Preset {
            schema_version: 1,
            id: "builtin-presentation".to_string(),
            name: "Presentation".to_string(),
            description: "Fullscreen-ready demonstration profile with stay-awake enabled.".to_string(),
            is_builtin: true,
            created_at: now.clone(),
            updated_at: now.clone(),
            settings: MirrorSettings {
                max_size: Some(1920),
                max_fps: Some(60),
                video_codec: Some("h264".to_string()),
                audio_enabled: true,
                fullscreen: true,
                stay_awake: true,
                show_touches: true,
                control_enabled: true,
                ..Default::default()
            },
            compatibility_notes: vec!["Shows touch indicators on screen for audience clarity.".to_string()],
        },
        Preset {
            schema_version: 1,
            id: "builtin-battery-conscious".to_string(),
            name: "Battery Conscious".to_string(),
            description: "Lower capture load with device screen turned off during mirroring.".to_string(),
            is_builtin: true,
            created_at: now.clone(),
            updated_at: now.clone(),
            settings: MirrorSettings {
                max_size: Some(1024),
                max_fps: Some(30),
                video_bit_rate: Some("2M".to_string()),
                video_codec: Some("h264".to_string()),
                audio_enabled: false,
                stay_awake: true,
                turn_screen_off: true,
                control_enabled: true,
                ..Default::default()
            },
            compatibility_notes: vec!["Turns off the physical phone display while mirroring.".to_string()],
        },
    ]
}

pub fn load_presets() -> Vec<Preset> {
    let mut builtins = builtin_presets();
    let path = presets_path();
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(user_presets) = serde_json::from_str::<Vec<Preset>>(&raw) {
            for p in user_presets {
                if !p.is_builtin {
                    builtins.push(p);
                }
            }
        }
    }
    builtins
}

pub fn save_user_presets(all_presets: &[Preset]) -> Result<(), String> {
    let user_only: Vec<&Preset> = all_presets.iter().filter(|p| !p.is_builtin).collect();
    let json = serde_json::to_string_pretty(&user_only).map_err(|e| e.to_string())?;
    fs::write(presets_path(), json).map_err(|e| e.to_string())
}

pub fn load_logs() -> Vec<ActivityLogEntry> {
    let path = logs_path();
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(parsed) = serde_json::from_str::<Vec<ActivityLogEntry>>(&raw) {
            return parsed;
        }
    }
    Vec::new()
}

pub fn save_logs(logs: &[ActivityLogEntry], max_entries: usize) -> Result<(), String> {
    let bounded: Vec<&ActivityLogEntry> = logs.iter().take(max_entries).collect();
    let json = serde_json::to_string_pretty(&bounded).map_err(|e| e.to_string())?;
    fs::write(logs_path(), json).map_err(|e| e.to_string())
}
