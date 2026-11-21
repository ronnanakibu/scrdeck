use crate::models::{
    CommandBuildRequest, CommandBuildResult, CommandConflict, ScrcpyCapabilities,
};
use crate::parsers::quote_windows_arg;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Component, Path};

fn has_control_chars(s: &str) -> bool {
    s.chars().any(|c| c == '\0' || c == '\r' || c == '\n')
}

fn canonical_short_flag(short: char) -> Option<&'static str> {
    match short {
        's' => Some("--serial"),
        'd' => Some("--select-usb"),
        'e' => Some("--select-tcpip"),
        'm' => Some("--max-size"),
        'b' => Some("--video-bit-rate"),
        'f' => Some("--fullscreen"),
        'w' => Some("--stay-awake"),
        'S' => Some("--turn-screen-off"),
        'n' => Some("--no-control"),
        't' => Some("--show-touches"),
        'r' => Some("--record"),
        'K' => Some("--keyboard"),
        'M' => Some("--mouse"),
        'G' => Some("--gamepad"),
        'N' => Some("--no-playback"),
        'p' => Some("--port"),
        'V' => Some("--verbosity"),
        'h' => Some("--help"),
        'v' => Some("--version"),
        _ => None,
    }
}

#[derive(Debug)]
enum ParsedFlag {
    Long(String),
    KnownShort {
        raw: String,
        canonical: &'static str,
    },
    UnknownShort(String),
}

fn extract_flag_info(arg: &str) -> Option<ParsedFlag> {
    if arg.starts_with("--") {
        let name = arg.split('=').next().unwrap_or(arg);
        if name.len() > 2 {
            return Some(ParsedFlag::Long(name.to_string()));
        }
        return None;
    }
    if arg.starts_with('-') && arg.len() >= 2 {
        let ch = arg.chars().nth(1)?;
        if ch.is_ascii_alphanumeric() {
            let raw = format!("-{}", ch);
            if let Some(canonical) = canonical_short_flag(ch) {
                return Some(ParsedFlag::KnownShort { raw, canonical });
            }
            return Some(ParsedFlag::UnknownShort(raw));
        }
    }
    None
}

fn is_protected_target_flag(canonical: &str) -> bool {
    matches!(
        canonical,
        "--serial" | "--select-usb" | "--select-tcpip" | "--tcpip"
    )
}

fn is_valid_bitrate(val: &str) -> bool {
    let re = Regex::new(r"^\d+(?:\.\d+)?[KkMmGg]?$").unwrap();
    if !re.is_match(val) {
        return false;
    }
    let num_part = val.trim_end_matches(['K', 'k', 'M', 'm', 'G', 'g']);
    num_part.parse::<f64>().map(|n| n > 0.0).unwrap_or(false)
}

fn is_valid_crop(val: &str) -> bool {
    let parts: Vec<&str> = val.split(':').collect();
    if parts.len() != 4 {
        return false;
    }
    let nums: Result<Vec<u32>, _> = parts.iter().map(|p| p.parse::<u32>()).collect();
    match nums {
        Ok(v) => v[0] > 0 && v[1] > 0,
        Err(_) => false,
    }
}

pub fn build_scrcpy_command(
    executable: &str,
    request: &CommandBuildRequest,
    capabilities: &ScrcpyCapabilities,
) -> CommandBuildResult {
    let mut argv: Vec<String> = Vec::new();
    let mut conflicts: Vec<CommandConflict> = Vec::new();
    let mut unknown_flags: Vec<String> = Vec::new();
    let mut managed_flags: HashSet<String> = HashSet::new();

    let known_set: HashSet<&str> = capabilities
        .supported_flags
        .iter()
        .map(|f| f.as_str())
        .collect();

    // 1. Target device serial
    match &request.device_serial {
        Some(serial) => {
            let trimmed = serial.trim();
            if trimmed.is_empty() {
                conflicts.push(CommandConflict {
                    flag: "--serial".to_string(),
                    reason:
                        "No target device is selected. Select a device before starting mirroring."
                            .to_string(),
                    severity: "error".to_string(),
                });
            } else if trimmed.starts_with('-')
                || has_control_chars(trimmed)
                || trimmed.chars().any(|c| c.is_whitespace())
            {
                conflicts.push(CommandConflict {
                    flag: "--serial".to_string(),
                    reason: "Selected device serial contains invalid characters.".to_string(),
                    severity: "error".to_string(),
                });
            } else {
                argv.push("--serial".to_string());
                argv.push(trimmed.to_string());
                managed_flags.insert("--serial".to_string());
            }
        }
        None => {
            conflicts.push(CommandConflict {
                flag: "--serial".to_string(),
                reason: "No target device is selected. Select a device before starting mirroring."
                    .to_string(),
                severity: "error".to_string(),
            });
        }
    }

    let s = &request.settings;

    // 2. Display & Video settings
    if let Some(max_size) = s.max_size {
        if max_size == 0 || max_size > 16384 {
            conflicts.push(CommandConflict {
                flag: "--max-size".to_string(),
                reason: "Max Size must be between 1 and 16384 pixels (or leave blank for native)."
                    .to_string(),
                severity: "error".to_string(),
            });
        } else {
            if capabilities.parsed_successfully && !capabilities.supports_max_size {
                conflicts.push(CommandConflict {
                    flag: "--max-size".to_string(),
                    reason: "Installed scrcpy version did not report --max-size support."
                        .to_string(),
                    severity: "warning".to_string(),
                });
            }
            argv.push(format!("--max-size={}", max_size));
            managed_flags.insert("--max-size".to_string());
        }
    }

    if let Some(max_fps) = s.max_fps {
        if max_fps == 0 || max_fps > 1000 {
            conflicts.push(CommandConflict {
                flag: "--max-fps".to_string(),
                reason: "Max FPS must be between 1 and 1000 (or leave blank for default)."
                    .to_string(),
                severity: "error".to_string(),
            });
        } else {
            if capabilities.parsed_successfully && !capabilities.supports_max_fps {
                conflicts.push(CommandConflict {
                    flag: "--max-fps".to_string(),
                    reason: "Installed scrcpy version did not report --max-fps support."
                        .to_string(),
                    severity: "warning".to_string(),
                });
            }
            argv.push(format!("--max-fps={}", max_fps));
            managed_flags.insert("--max-fps".to_string());
        }
    }

    if let Some(bit_rate) = &s.video_bit_rate {
        let trimmed = bit_rate.trim();
        if !trimmed.is_empty() {
            if has_control_chars(trimmed) || !is_valid_bitrate(trimmed) {
                conflicts.push(CommandConflict {
                    flag: "--video-bit-rate".to_string(),
                    reason: "Video bitrate must be a positive number with optional K/M/G suffix (e.g. 8M, 4000K)."
                        .to_string(),
                    severity: "error".to_string(),
                });
            } else {
                if capabilities.parsed_successfully && !known_set.contains("--video-bit-rate") {
                    conflicts.push(CommandConflict {
                        flag: "--video-bit-rate".to_string(),
                        reason:
                            "Installed scrcpy version did not report --video-bit-rate support."
                                .to_string(),
                        severity: "warning".to_string(),
                    });
                }
                argv.push(format!("--video-bit-rate={}", trimmed));
                managed_flags.insert("--video-bit-rate".to_string());
            }
        }
    }

    if let Some(codec) = &s.video_codec {
        let trimmed = codec.trim();
        if !trimmed.is_empty() {
            if has_control_chars(trimmed) || trimmed.chars().any(|c| c.is_whitespace()) {
                conflicts.push(CommandConflict {
                    flag: "--video-codec".to_string(),
                    reason: "Video codec contains invalid characters.".to_string(),
                    severity: "error".to_string(),
                });
            } else {
                if !matches!(trimmed, "h264" | "h265" | "av1") {
                    conflicts.push(CommandConflict {
                        flag: "--video-codec".to_string(),
                        reason: format!(
                            "Unrecognized video codec '{}'. Expected h264, h265, or av1.",
                            trimmed
                        ),
                        severity: "warning".to_string(),
                    });
                }
                if capabilities.parsed_successfully && !capabilities.supports_video_codec {
                    conflicts.push(CommandConflict {
                        flag: "--video-codec".to_string(),
                        reason: "Installed scrcpy version did not report --video-codec support."
                            .to_string(),
                        severity: "warning".to_string(),
                    });
                }
                argv.push(format!("--video-codec={}", trimmed));
                managed_flags.insert("--video-codec".to_string());
            }
        }
    }

    if let Some(crop) = &s.crop {
        let trimmed = crop.trim();
        if !trimmed.is_empty() {
            if has_control_chars(trimmed) || !is_valid_crop(trimmed) {
                conflicts.push(CommandConflict {
                    flag: "--crop".to_string(),
                    reason: "Crop must follow width:height:x:y format with positive width and height (e.g. 1224:1440:0:0)."
                        .to_string(),
                    severity: "error".to_string(),
                });
            } else {
                if capabilities.parsed_successfully && !known_set.contains("--crop") {
                    conflicts.push(CommandConflict {
                        flag: "--crop".to_string(),
                        reason: "Installed scrcpy version did not report --crop support."
                            .to_string(),
                        severity: "warning".to_string(),
                    });
                }
                argv.push(format!("--crop={}", trimmed));
                managed_flags.insert("--crop".to_string());
            }
        }
    }

    // 3. Audio settings
    if !s.audio_enabled {
        if capabilities.parsed_successfully && !capabilities.supports_audio {
            conflicts.push(CommandConflict {
                flag: "--no-audio".to_string(),
                reason: "Installed scrcpy version did not report --no-audio support.".to_string(),
                severity: "warning".to_string(),
            });
        }
        argv.push("--no-audio".to_string());
        managed_flags.insert("--no-audio".to_string());

        if s.audio_source
            .as_deref()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
            || s.audio_codec
                .as_deref()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
        {
            conflicts.push(CommandConflict {
                flag: "--audio-source".to_string(),
                reason: "Audio source/codec options are ignored while audio forwarding is disabled."
                    .to_string(),
                severity: "warning".to_string(),
            });
        }
    } else {
        if let Some(source) = &s.audio_source {
            let trimmed = source.trim();
            if !trimmed.is_empty() {
                if has_control_chars(trimmed) || trimmed.chars().any(|c| c.is_whitespace()) {
                    conflicts.push(CommandConflict {
                        flag: "--audio-source".to_string(),
                        reason: "Audio source contains invalid characters.".to_string(),
                        severity: "error".to_string(),
                    });
                } else {
                    if capabilities.parsed_successfully && !known_set.contains("--audio-source") {
                        conflicts.push(CommandConflict {
                            flag: "--audio-source".to_string(),
                            reason:
                                "Installed scrcpy version did not report --audio-source support."
                                    .to_string(),
                            severity: "warning".to_string(),
                        });
                    }
                    argv.push(format!("--audio-source={}", trimmed));
                    managed_flags.insert("--audio-source".to_string());
                }
            }
        }
        if let Some(acodec) = &s.audio_codec {
            let trimmed = acodec.trim();
            if !trimmed.is_empty() {
                if has_control_chars(trimmed) || trimmed.chars().any(|c| c.is_whitespace()) {
                    conflicts.push(CommandConflict {
                        flag: "--audio-codec".to_string(),
                        reason: "Audio codec contains invalid characters.".to_string(),
                        severity: "error".to_string(),
                    });
                } else {
                    if !matches!(trimmed, "opus" | "aac" | "flac" | "raw") {
                        conflicts.push(CommandConflict {
                            flag: "--audio-codec".to_string(),
                            reason: format!(
                                "Unrecognized audio codec '{}'. Expected opus, aac, flac, or raw.",
                                trimmed
                            ),
                            severity: "warning".to_string(),
                        });
                    }
                    if capabilities.parsed_successfully && !known_set.contains("--audio-codec") {
                        conflicts.push(CommandConflict {
                            flag: "--audio-codec".to_string(),
                            reason:
                                "Installed scrcpy version did not report --audio-codec support."
                                    .to_string(),
                            severity: "warning".to_string(),
                        });
                    }
                    argv.push(format!("--audio-codec={}", trimmed));
                    managed_flags.insert("--audio-codec".to_string());
                }
            }
        }
    }

    // 4. Window & Control settings
    if s.fullscreen {
        if capabilities.parsed_successfully && !capabilities.supports_fullscreen {
            conflicts.push(CommandConflict {
                flag: "--fullscreen".to_string(),
                reason: "Installed scrcpy version did not report --fullscreen support.".to_string(),
                severity: "warning".to_string(),
            });
        }
        argv.push("--fullscreen".to_string());
        managed_flags.insert("--fullscreen".to_string());
    }

    if s.always_on_top {
        if capabilities.parsed_successfully && !capabilities.supports_always_on_top {
            conflicts.push(CommandConflict {
                flag: "--always-on-top".to_string(),
                reason: "Installed scrcpy version did not report --always-on-top support."
                    .to_string(),
                severity: "warning".to_string(),
            });
        }
        argv.push("--always-on-top".to_string());
        managed_flags.insert("--always-on-top".to_string());
    }

    if !s.control_enabled && s.stay_awake {
        conflicts.push(CommandConflict {
            flag: "--stay-awake".to_string(),
            reason: "Keep device awake (--stay-awake) requires device control to be enabled (incompatible with --no-control)."
                .to_string(),
            severity: "error".to_string(),
        });
    }

    if !s.control_enabled && s.show_touches {
        conflicts.push(CommandConflict {
            flag: "--show-touches".to_string(),
            reason: "Show touches (--show-touches) requires device control to be enabled (incompatible with --no-control)."
                .to_string(),
            severity: "error".to_string(),
        });
    }

    if s.stay_awake {
        if capabilities.parsed_successfully && !capabilities.supports_stay_awake {
            conflicts.push(CommandConflict {
                flag: "--stay-awake".to_string(),
                reason: "Installed scrcpy version did not report --stay-awake support.".to_string(),
                severity: "warning".to_string(),
            });
        }
        argv.push("--stay-awake".to_string());
        managed_flags.insert("--stay-awake".to_string());
    }

    if s.turn_screen_off {
        if capabilities.parsed_successfully && !capabilities.supports_turn_screen_off {
            conflicts.push(CommandConflict {
                flag: "--turn-screen-off".to_string(),
                reason: "Installed scrcpy version did not report --turn-screen-off support."
                    .to_string(),
                severity: "warning".to_string(),
            });
        }
        argv.push("--turn-screen-off".to_string());
        managed_flags.insert("--turn-screen-off".to_string());
    }

    if !s.control_enabled {
        if capabilities.parsed_successfully && !capabilities.supports_no_control {
            conflicts.push(CommandConflict {
                flag: "--no-control".to_string(),
                reason: "Installed scrcpy version did not report --no-control support.".to_string(),
                severity: "warning".to_string(),
            });
        }
        argv.push("--no-control".to_string());
        managed_flags.insert("--no-control".to_string());
    }

    if s.show_touches {
        if capabilities.parsed_successfully && !capabilities.supports_show_touches {
            conflicts.push(CommandConflict {
                flag: "--show-touches".to_string(),
                reason: "Installed scrcpy version did not report --show-touches support."
                    .to_string(),
                severity: "warning".to_string(),
            });
        }
        argv.push("--show-touches".to_string());
        managed_flags.insert("--show-touches".to_string());
    }

    if let Some(title) = &s.window_title {
        let trimmed = title.trim();
        if !trimmed.is_empty() {
            if has_control_chars(trimmed) {
                conflicts.push(CommandConflict {
                    flag: "--window-title".to_string(),
                    reason: "Window title cannot contain newline or null characters.".to_string(),
                    severity: "error".to_string(),
                });
            } else {
                if capabilities.parsed_successfully && !capabilities.supports_window_title {
                    conflicts.push(CommandConflict {
                        flag: "--window-title".to_string(),
                        reason: "Installed scrcpy version did not report --window-title support."
                            .to_string(),
                        severity: "warning".to_string(),
                    });
                }
                argv.push(format!("--window-title={}", trimmed));
                managed_flags.insert("--window-title".to_string());
            }
        }
    }

    if let Some(rot) = s.rotation {
        if rot > 3 {
            conflicts.push(CommandConflict {
                flag: "--orientation".to_string(),
                reason: "Rotation must be 0 (0°), 1 (90°), 2 (180°), or 3 (270°).".to_string(),
                severity: "error".to_string(),
            });
        } else {
            let orientation_flag = if capabilities.parsed_successfully
                && !known_set.contains("--orientation")
                && known_set.contains("--display-orientation")
            {
                "--display-orientation"
            } else {
                "--orientation"
            };

            if capabilities.parsed_successfully && !capabilities.supports_orientation {
                conflicts.push(CommandConflict {
                    flag: orientation_flag.to_string(),
                    reason: "Installed scrcpy version did not report orientation flag support."
                        .to_string(),
                    severity: "warning".to_string(),
                });
            }
            argv.push(format!("{}={}", orientation_flag, rot * 90));
            managed_flags.insert("--orientation".to_string());
            managed_flags.insert("--display-orientation".to_string());
        }
    }

    // 5. Recording settings
    if s.recording_enabled {
        match &s.recording_path {
            Some(path) if !path.trim().is_empty() => {
                let trimmed = path.trim();
                if has_control_chars(trimmed) {
                    conflicts.push(CommandConflict {
                        flag: "--record".to_string(),
                        reason: "Recording path contains invalid control characters.".to_string(),
                        severity: "error".to_string(),
                    });
                } else {
                    let p = Path::new(trimmed);
                    let has_parent_traversal =
                        p.components().any(|c| matches!(c, Component::ParentDir));
                    if has_parent_traversal {
                        conflicts.push(CommandConflict {
                            flag: "--record".to_string(),
                            reason: "Recording path cannot contain '..' traversal segments."
                                .to_string(),
                            severity: "error".to_string(),
                        });
                    } else if p.is_dir() {
                        conflicts.push(CommandConflict {
                            flag: "--record".to_string(),
                            reason: "Recording path points to a directory; specify a file path (e.g. recording.mp4)."
                                .to_string(),
                            severity: "error".to_string(),
                        });
                    } else {
                        if let Some(parent) = p.parent() {
                            if !parent.as_os_str().is_empty() && !parent.exists() {
                                conflicts.push(CommandConflict {
                                    flag: "--record".to_string(),
                                    reason: format!(
                                        "Recording destination folder does not exist: {}",
                                        parent.display()
                                    ),
                                    severity: "error".to_string(),
                                });
                            }
                        }
                        if p.exists() {
                            conflicts.push(CommandConflict {
                                flag: "--record".to_string(),
                                reason: "Recording file already exists and will be overwritten by scrcpy."
                                    .to_string(),
                                severity: "warning".to_string(),
                            });
                        }
                        if capabilities.parsed_successfully && !capabilities.supports_record {
                            conflicts.push(CommandConflict {
                                flag: "--record".to_string(),
                                reason: "Installed scrcpy version did not report --record support."
                                    .to_string(),
                                severity: "warning".to_string(),
                            });
                        }
                        argv.push(format!("--record={}", trimmed));
                        managed_flags.insert("--record".to_string());

                        if let Some(fmt) = &s.recording_format {
                            let fmt_trimmed = fmt.trim();
                            if !fmt_trimmed.is_empty() {
                                if !matches!(
                                    fmt_trimmed,
                                    "mp4" | "mkv" | "m4a" | "aac" | "opus" | "flac" | "wav"
                                ) {
                                    conflicts.push(CommandConflict {
                                        flag: "--record-format".to_string(),
                                        reason: format!(
                                            "Unsupported recording format '{}'. Expected mp4, mkv, m4a, aac, opus, flac, or wav.",
                                            fmt_trimmed
                                        ),
                                        severity: "error".to_string(),
                                    });
                                } else {
                                    if capabilities.parsed_successfully
                                        && !known_set.contains("--record-format")
                                    {
                                        conflicts.push(CommandConflict {
                                            flag: "--record-format".to_string(),
                                            reason: "Installed scrcpy version did not report --record-format support."
                                                .to_string(),
                                            severity: "warning".to_string(),
                                        });
                                    }
                                    argv.push(format!("--record-format={}", fmt_trimmed));
                                    managed_flags.insert("--record-format".to_string());
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                conflicts.push(CommandConflict {
                    flag: "--record".to_string(),
                    reason: "Recording is enabled, but no output file path is specified."
                        .to_string(),
                    severity: "error".to_string(),
                });
            }
        }
    }

    // 6. Custom arguments validation & conflict detection
    let mut seen_custom_flags: HashSet<String> = HashSet::new();
    let mut custom_has_no_audio = false;
    let mut custom_has_no_video = false;
    let mut custom_has_no_control = false;
    let mut custom_has_otg = false;

    for custom in &s.custom_args {
        let trimmed = custom.trim();
        if trimmed.is_empty() {
            continue;
        }
        if has_control_chars(trimmed) {
            conflicts.push(CommandConflict {
                flag: trimmed.to_string(),
                reason: "Custom argument contains invalid newline or null characters.".to_string(),
                severity: "error".to_string(),
            });
            continue;
        }

        if let Some(parsed) = extract_flag_info(trimmed) {
            match parsed {
                ParsedFlag::Long(flag_name) => {
                    if is_protected_target_flag(&flag_name) {
                        conflicts.push(CommandConflict {
                            flag: flag_name,
                            reason: "Custom arguments cannot override ScrDeck's selected device target (--serial / -s / -d / -e / --tcpip)."
                                .to_string(),
                            severity: "error".to_string(),
                        });
                        continue;
                    }

                    if flag_name == "--no-audio" {
                        custom_has_no_audio = true;
                    } else if flag_name == "--no-video" {
                        custom_has_no_video = true;
                    } else if flag_name == "--no-control" {
                        custom_has_no_control = true;
                    } else if flag_name == "--otg" {
                        custom_has_otg = true;
                    }

                    if managed_flags.contains(&flag_name) {
                        conflicts.push(CommandConflict {
                            flag: flag_name.clone(),
                            reason: format!(
                                "Custom argument '{}' duplicates a setting already managed by Mirroring Studio.",
                                flag_name
                            ),
                            severity: "warning".to_string(),
                        });
                    } else if !seen_custom_flags.insert(flag_name.clone()) {
                        conflicts.push(CommandConflict {
                            flag: flag_name.clone(),
                            reason: format!(
                                "Duplicate custom argument '{}' specified multiple times.",
                                flag_name
                            ),
                            severity: "warning".to_string(),
                        });
                    }

                    if capabilities.parsed_successfully && !known_set.contains(flag_name.as_str()) {
                        unknown_flags.push(flag_name);
                    }
                }
                ParsedFlag::KnownShort { raw, canonical } => {
                    if is_protected_target_flag(canonical) {
                        conflicts.push(CommandConflict {
                            flag: raw,
                            reason: "Custom arguments cannot override ScrDeck's selected device target (--serial / -s / -d / -e / --tcpip)."
                                .to_string(),
                            severity: "error".to_string(),
                        });
                        continue;
                    }

                    if canonical == "--no-control" {
                        custom_has_no_control = true;
                    }

                    if managed_flags.contains(canonical) {
                        conflicts.push(CommandConflict {
                            flag: raw.clone(),
                            reason: format!(
                                "Custom argument '{}' ({}) duplicates a setting already managed by Mirroring Studio.",
                                raw, canonical
                            ),
                            severity: "warning".to_string(),
                        });
                    } else if !seen_custom_flags.insert(canonical.to_string()) {
                        conflicts.push(CommandConflict {
                            flag: raw.clone(),
                            reason: format!(
                                "Duplicate custom argument '{}' ({}) specified multiple times.",
                                raw, canonical
                            ),
                            severity: "warning".to_string(),
                        });
                    }

                    if capabilities.parsed_successfully && !known_set.contains(canonical) {
                        unknown_flags.push(raw);
                    }
                }
                ParsedFlag::UnknownShort(raw) => {
                    if capabilities.parsed_successfully {
                        unknown_flags.push(raw);
                    }
                }
            }
        }

        argv.push(trimmed.to_string());
    }

    // Cross-flag incompatibility checks involving custom_args
    if custom_has_no_audio && s.audio_enabled {
        conflicts.push(CommandConflict {
            flag: "--no-audio".to_string(),
            reason: "Custom '--no-audio' conflicts with 'Forward device audio' enabled in Mirroring Studio. Disable audio in the UI instead."
                .to_string(),
            severity: "error".to_string(),
        });
    }
    if !s.audio_enabled
        && (seen_custom_flags.contains("--audio-source")
            || seen_custom_flags.contains("--audio-codec")
            || seen_custom_flags.contains("--audio-bit-rate")
            || seen_custom_flags.contains("--require-audio"))
    {
        conflicts.push(CommandConflict {
            flag: "--audio-source".to_string(),
            reason: "Custom audio options conflict with audio forwarding being disabled."
                .to_string(),
            severity: "error".to_string(),
        });
    }
    if custom_has_no_control && (s.stay_awake || s.show_touches) {
        conflicts.push(CommandConflict {
            flag: "--no-control".to_string(),
            reason: "Custom '--no-control' is incompatible with '--stay-awake' or '--show-touches'."
                .to_string(),
            severity: "error".to_string(),
        });
    }
    if custom_has_no_video
        && (s.max_size.is_some()
            || s.max_fps.is_some()
            || s.video_bit_rate
                .as_deref()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
            || s.crop
                .as_deref()
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false))
    {
        conflicts.push(CommandConflict {
            flag: "--no-video".to_string(),
            reason: "Custom '--no-video' disables video mirroring while video options are configured."
                .to_string(),
            severity: "warning".to_string(),
        });
    }
    if custom_has_otg
        && (s.recording_enabled
            || s.max_size.is_some()
            || s.max_fps.is_some()
            || s.video_codec.is_some())
    {
        conflicts.push(CommandConflict {
            flag: "--otg".to_string(),
            reason: "Custom '--otg' mode disables video mirroring and recording; clear video/recording options or remove '--otg'."
                .to_string(),
            severity: "error".to_string(),
        });
    }

    let has_errors = conflicts.iter().any(|c| c.severity == "error");
    let requires_override = !unknown_flags.is_empty() && !request.allow_unknown_flags;
    let is_valid = !has_errors && !requires_override;

    let preview_parts: Vec<String> = std::iter::once(quote_windows_arg(executable))
        .chain(argv.iter().map(|a| quote_windows_arg(a)))
        .collect();
    let preview = preview_parts.join(" ");

    CommandBuildResult {
        executable: executable.to_string(),
        argv,
        preview,
        conflicts,
        unknown_flags,
        is_valid,
        requires_override,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MirrorSettings;
    use crate::parsers::parse_windows_cmdline;

    fn full_caps() -> ScrcpyCapabilities {
        ScrcpyCapabilities {
            scrcpy_version: Some("5.0.1".to_string()),
            supported_flags: vec![
                "--serial",
                "--select-usb",
                "--select-tcpip",
                "--max-size",
                "--max-fps",
                "--video-bit-rate",
                "--video-codec",
                "--crop",
                "--no-audio",
                "--audio-source",
                "--audio-codec",
                "--fullscreen",
                "--always-on-top",
                "--stay-awake",
                "--turn-screen-off",
                "--no-control",
                "--show-touches",
                "--window-title",
                "--orientation",
                "--record",
                "--record-format",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            parsed_successfully: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_build_command_matches_argv_and_blocks_serial_overrides() {
        let caps = full_caps();

        for bad_args in [
            vec!["-s".to_string(), "OTHER_DEVICE".to_string()],
            vec!["-sOTHER_DEVICE".to_string()],
            vec!["--serial=OTHER_DEVICE".to_string()],
            vec!["-d".to_string()],
            vec!["--select-tcpip".to_string()],
            vec!["--tcpip=192.168.1.5:5555".to_string()],
        ] {
            let mut settings = MirrorSettings::default();
            settings.custom_args = bad_args.clone();

            let req = CommandBuildRequest {
                device_serial: Some("R58M12345AB".to_string()),
                settings,
                allow_unknown_flags: false,
            };

            let res = build_scrcpy_command("scrcpy.exe", &req, &caps);
            assert!(!res.is_valid, "Expected invalid for {:?}", bad_args);
            assert!(
                res.conflicts.iter().any(|c| c.severity == "error"),
                "Expected error conflict for {:?}",
                bad_args
            );
        }
    }

    #[test]
    fn test_preview_roundtrips_to_exact_argv() {
        let caps = full_caps();
        let mut settings = MirrorSettings::default();
        settings.window_title = Some(r#"My "Phone" \Test\"#.to_string());
        settings.video_bit_rate = Some("8M".to_string());
        settings.crop = Some("1224:1440:0:0".to_string());
        settings.rotation = Some(1);

        let req = CommandBuildRequest {
            device_serial: Some("192.168.1.42:5555".to_string()),
            settings,
            allow_unknown_flags: false,
        };

        let res = build_scrcpy_command(r"C:\Program Files\ScrDeck\scrcpy.exe", &req, &caps);
        assert!(res.is_valid);
        let parsed = parse_windows_cmdline(&res.preview).unwrap();
        assert_eq!(parsed[0], r"C:\Program Files\ScrDeck\scrcpy.exe");
        assert_eq!(&parsed[1..], res.argv.as_slice());
    }

    #[test]
    fn test_numeric_and_format_validation() {
        let caps = full_caps();
        let mut settings = MirrorSettings::default();
        settings.max_size = Some(0);
        settings.max_fps = Some(2000);
        settings.video_bit_rate = Some("not-a-bitrate".to_string());
        settings.crop = Some("0:100:0:0".to_string());
        settings.rotation = Some(5);

        let req = CommandBuildRequest {
            device_serial: Some("ABC123".to_string()),
            settings,
            allow_unknown_flags: false,
        };
        let res = build_scrcpy_command("scrcpy.exe", &req, &caps);
        assert!(!res.is_valid);
        assert!(res.conflicts.iter().filter(|c| c.severity == "error").count() >= 5);
    }

    #[test]
    fn test_incompatible_flags_detected() {
        let caps = full_caps();
        let mut settings = MirrorSettings::default();
        settings.control_enabled = false;
        settings.stay_awake = true;
        settings.show_touches = true;

        let req = CommandBuildRequest {
            device_serial: Some("ABC123".to_string()),
            settings,
            allow_unknown_flags: false,
        };
        let res = build_scrcpy_command("scrcpy.exe", &req, &caps);
        assert!(!res.is_valid);
        assert!(res.conflicts.iter().any(|c| c.flag == "--stay-awake" && c.severity == "error"));
        assert!(res.conflicts.iter().any(|c| c.flag == "--show-touches" && c.severity == "error"));
    }

    #[test]
    fn test_unknown_flag_requires_override_and_short_flag_duplicate() {
        let mut caps = ScrcpyCapabilities::default();
        caps.parsed_successfully = true;
        caps.supported_flags = vec!["--serial".to_string(), "--max-size".to_string()];

        let mut settings = MirrorSettings::default();
        settings.max_fps = None;
        settings.video_codec = None;
        settings.stay_awake = false;
        settings.custom_args = vec!["--experimental-flag".to_string(), "-m1024".to_string()];

        let req = CommandBuildRequest {
            device_serial: Some("ABC123".to_string()),
            settings: settings.clone(),
            allow_unknown_flags: false,
        };
        let res = build_scrcpy_command("scrcpy.exe", &req, &caps);
        assert!(res.requires_override);
        assert!(!res.is_valid);
        assert!(res.conflicts.iter().any(|c| c.flag == "-m" && c.severity == "warning"));

        let req_override = CommandBuildRequest {
            device_serial: Some("ABC123".to_string()),
            settings,
            allow_unknown_flags: true,
        };
        let res2 = build_scrcpy_command("scrcpy.exe", &req_override, &caps);
        assert!(!res2.requires_override);
        assert!(res2.is_valid);
        assert!(res2.preview.contains("--experimental-flag"));
    }

    #[test]
    fn test_display_orientation_fallback_for_newer_scrcpy() {
        let mut caps = full_caps();
        caps.supported_flags.retain(|f| f != "--orientation");
        caps.supported_flags.push("--display-orientation".to_string());

        let mut settings = MirrorSettings::default();
        settings.rotation = Some(2);

        let req = CommandBuildRequest {
            device_serial: Some("ABC123".to_string()),
            settings,
            allow_unknown_flags: false,
        };
        let res = build_scrcpy_command("scrcpy.exe", &req, &caps);
        assert!(res.is_valid);
        assert!(res.argv.contains(&"--display-orientation=180".to_string()));
    }
}
