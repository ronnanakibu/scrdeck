use crate::models::{AdbState, Device, DeviceTransport, DiagnosticExplanation, ScrcpyCapabilities};
use chrono::Utc;
use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;

pub fn parse_adb_devices(output: &str) -> Vec<Device> {
    let now = Utc::now().to_rfc3339();
    let mut devices = Vec::new();

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with("List of devices attached")
            || line.starts_with("* daemon")
            || line.starts_with("adb server")
        {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let serial = parts[0].to_string();
        let state_str = parts[1];

        let (adb_state, token_start) = if state_str == "no" && parts.get(2) == Some(&"permissions")
        {
            (AdbState::NoPermission, 3)
        } else {
            let st = match state_str {
                "device" => AdbState::Device,
                "unauthorized" | "authorizing" => AdbState::Unauthorized,
                "offline" | "connecting" => AdbState::Offline,
                "no" => AdbState::NoPermission,
                _ => AdbState::Unknown,
            };
            (st, 2)
        };

        let mut model = None;
        let mut product = None;
        let mut device_code = None;
        let mut transport_id = None;
        let mut has_usb_hint = false;

        for token in &parts[token_start..] {
            if let Some(val) = token.strip_prefix("model:") {
                model = Some(val.replace('_', " "));
            } else if let Some(val) = token.strip_prefix("product:") {
                product = Some(val.to_string());
            } else if let Some(val) = token.strip_prefix("device:") {
                device_code = Some(val.to_string());
            } else if let Some(val) = token.strip_prefix("transport_id:") {
                transport_id = Some(val.to_string());
            } else if token.starts_with("usb:") {
                has_usb_hint = true;
            }
        }

        let transport = if serial.starts_with("emulator-") {
            DeviceTransport::Emulator
        } else if serial.contains(':') || serial.contains("._adb-tls-connect._tcp") {
            DeviceTransport::TcpIp
        } else if has_usb_hint || !serial.is_empty() {
            DeviceTransport::Usb
        } else {
            DeviceTransport::Unknown
        };

        devices.push(Device {
            serial,
            model,
            product,
            device_code,
            transport_id,
            transport,
            adb_state,
            android_release: None,
            api_level: None,
            battery_percent: None,
            charging_state: None,
            last_seen_at: now.clone(),
        });
    }

    devices
}

pub fn parse_scrcpy_version(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("scrcpy ") {
            let ver = rest.split_whitespace().next().unwrap_or("").trim();
            if !ver.is_empty() {
                return Some(ver.to_string());
            }
        }
    }
    None
}

pub fn parse_adb_version(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Android Debug Bridge version ") {
            let ver = rest.split_whitespace().next().unwrap_or("").trim();
            if !ver.is_empty() {
                return Some(ver.to_string());
            }
        }
    }
    None
}

pub fn parse_scrcpy_help(output: &str, version: Option<String>) -> ScrcpyCapabilities {
    // Match option definitions like `    -m, --max-size=value` or `        --no-audio`
    let option_line_re =
        Regex::new(r"(?m)^\s*(?:-[a-zA-Z0-9],\s+)?(--[a-zA-Z0-9][a-zA-Z0-9-]*)").unwrap();
    let fallback_re = Regex::new(r"(?m)(?:^|\s)(--[a-zA-Z0-9][a-zA-Z0-9-]*)").unwrap();
    let mut flags_set = HashSet::new();

    for cap in option_line_re.captures_iter(output) {
        if let Some(m) = cap.get(1) {
            flags_set.insert(m.as_str().to_string());
        }
    }

    if flags_set.is_empty() {
        for cap in fallback_re.captures_iter(output) {
            if let Some(m) = cap.get(1) {
                flags_set.insert(m.as_str().to_string());
            }
        }
    }

    let mut supported_flags: Vec<String> = flags_set.iter().cloned().collect();
    supported_flags.sort();

    let parsed_successfully = !supported_flags.is_empty();

    ScrcpyCapabilities {
        scrcpy_version: version,
        supports_audio: flags_set.contains("--no-audio") || flags_set.contains("--audio-source"),
        supports_video_codec: flags_set.contains("--video-codec"),
        supports_max_fps: flags_set.contains("--max-fps"),
        supports_max_size: flags_set.contains("--max-size"),
        supports_record: flags_set.contains("--record"),
        supports_turn_screen_off: flags_set.contains("--turn-screen-off"),
        supports_stay_awake: flags_set.contains("--stay-awake"),
        supports_no_control: flags_set.contains("--no-control"),
        supports_always_on_top: flags_set.contains("--always-on-top"),
        supports_fullscreen: flags_set.contains("--fullscreen"),
        supports_show_touches: flags_set.contains("--show-touches"),
        supports_window_title: flags_set.contains("--window-title"),
        supports_orientation: flags_set.contains("--orientation")
            || flags_set.contains("--display-orientation"),
        supported_flags,
        parsed_successfully,
    }
}

/// Parses a Windows-style command-line string into individual argv tokens.
/// Follows Windows CommandLineToArgvW quoting and backslash rules and rejects control characters.
pub fn parse_windows_cmdline(input: &str) -> Result<Vec<String>, String> {
    if input.chars().any(|c| c == '\0' || c == '\r' || c == '\n') {
        return Err("Custom arguments cannot contain newline or null characters.".to_string());
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut has_token = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c == '\\' {
            let mut backslashes = 0;
            while i < chars.len() && chars[i] == '\\' {
                backslashes += 1;
                i += 1;
            }
            if i < chars.len() && chars[i] == '"' {
                for _ in 0..(backslashes / 2) {
                    current.push('\\');
                }
                if backslashes % 2 == 1 {
                    current.push('"');
                    has_token = true;
                    i += 1;
                } else {
                    in_quotes = !in_quotes;
                    has_token = true;
                    i += 1;
                }
            } else {
                for _ in 0..backslashes {
                    current.push('\\');
                }
                has_token = true;
            }
            continue;
        }

        if c == '"' {
            in_quotes = !in_quotes;
            has_token = true;
            i += 1;
            continue;
        }

        if c.is_whitespace() && !in_quotes {
            if has_token {
                args.push(current.clone());
                current.clear();
                has_token = false;
            }
            i += 1;
            continue;
        }

        current.push(c);
        has_token = true;
        i += 1;
    }

    if in_quotes {
        return Err("Unclosed double quote in custom arguments.".to_string());
    }

    if has_token {
        args.push(current);
    }

    Ok(args)
}

/// Quotes a single argument according to Windows CommandLineToArgvW rules for preview display.
pub fn quote_windows_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }
    let needs_quotes = arg.chars().any(|c| c == ' ' || c == '\t' || c == '"');
    if !needs_quotes {
        return arg.to_string();
    }

    let mut out = String::with_capacity(arg.len() + 4);
    out.push('"');
    let mut backslashes = 0usize;
    for c in arg.chars() {
        if c == '\\' {
            backslashes += 1;
        } else if c == '"' {
            for _ in 0..(backslashes * 2 + 1) {
                out.push('\\');
            }
            out.push('"');
            backslashes = 0;
        } else {
            for _ in 0..backslashes {
                out.push('\\');
            }
            backslashes = 0;
            out.push(c);
        }
    }
    for _ in 0..(backslashes * 2) {
        out.push('\\');
    }
    out.push('"');
    out
}

pub fn redact_sensitive_text(input: &str, serials: &[String]) -> String {
    let mut out = input.to_string();

    for serial in serials {
        let trimmed = serial.trim();
        if !trimmed.is_empty() {
            out = out.replace(trimmed, "[REDACTED_SERIAL]");
        }
    }

    // Redact IPv4 addresses (with optional port)
    let ip_re = Regex::new(r"\b\d{1,3}(?:\.\d{1,3}){3}(?::\d{1,5})?\b").unwrap();
    out = ip_re.replace_all(&out, "[REDACTED_IP]").to_string();

    // Redact bracketed IPv6 endpoints like [fe80::1]:5555
    let ipv6_bracket_re = Regex::new(r"\[[0-9a-fA-F:]+\](?::\d{1,5})?").unwrap();
    out = ipv6_bracket_re
        .replace_all(&out, "[REDACTED_IP]")
        .to_string();

    // Redact Windows user profile paths (single backslash, JSON-escaped double backslash, or forward slash)
    let user_path_escaped_re = Regex::new(r"(?i)[A-Z]:\\\\Users\\\\[^\\\s\x22]+").unwrap();
    out = user_path_escaped_re
        .replace_all(&out, r"C:\\Users\\[REDACTED_USER]")
        .to_string();

    let user_path_re = Regex::new(r"(?i)[A-Z]:\\Users\\[^\\\s\x22]+").unwrap();
    out = user_path_re
        .replace_all(&out, r"C:\Users\[REDACTED_USER]")
        .to_string();

    let user_path_fwd_re = Regex::new(r"(?i)[A-Z]:/Users/[^/\s\x22]+").unwrap();
    out = user_path_fwd_re
        .replace_all(&out, "C:/Users/[REDACTED_USER]")
        .to_string();

    // Redact 6-digit pairing codes if any pattern mentions pairing code or adb pair
    let pair_re = Regex::new(r"(?i)(pairing\s*code[:=\s]+)\d{4,12}").unwrap();
    out = pair_re.replace_all(&out, "${1}[REDACTED_CODE]").to_string();

    let adb_pair_re = Regex::new(r"(?i)(adb(?:\.exe)?\s+pair\s+\S+\s+)\d{4,12}").unwrap();
    out = adb_pair_re
        .replace_all(&out, "${1}[REDACTED_CODE]")
        .to_string();

    out
}

pub fn redact_json_value(val: &mut Value, serials: &[String]) {
    match val {
        Value::String(s) => {
            *s = redact_sensitive_text(s, serials);
        }
        Value::Array(arr) => {
            for item in arr {
                redact_json_value(item, serials);
            }
        }
        Value::Object(map) => {
            for v in map.values_mut() {
                redact_json_value(v, serials);
            }
        }
        _ => {}
    }
}

pub fn classify_error(stderr: &str, exit_code: Option<i32>) -> Option<DiagnosticExplanation> {
    let lower = stderr.to_lowercase();

    if lower.contains("unauthorized") || lower.contains("device unauthorized") {
        return Some(DiagnosticExplanation {
            code: "ADB_UNAUTHORIZED".to_string(),
            title: "Your phone needs permission".to_string(),
            cause: "Android has not yet authorized this computer for USB debugging.".to_string(),
            confidence: "high".to_string(),
            next_steps: vec![
                "Unlock your Android phone.".to_string(),
                "Look for the 'Allow USB debugging?' RSA prompt and tap Allow.".to_string(),
                "Refresh devices in ScrDeck and try again.".to_string(),
            ],
        });
    }

    if lower.contains("device offline") || lower.contains("offline") {
        return Some(DiagnosticExplanation {
            code: "ADB_OFFLINE".to_string(),
            title: "Device connection is not responding".to_string(),
            cause: "ADB sees the device entry, but the Android daemon is not responding."
                .to_string(),
            confidence: "high".to_string(),
            next_steps: vec![
                "Unplug and reconnect the USB cable, or toggle USB Debugging off and on."
                    .to_string(),
                "Unlock the phone screen.".to_string(),
                "If using Wireless Debugging, verify both devices are on the same Wi-Fi network."
                    .to_string(),
            ],
        });
    }

    if lower.contains("no devices/emulators found")
        || lower.contains("could not find any adb device")
    {
        return Some(DiagnosticExplanation {
            code: "NO_ADB_DEVICE".to_string(),
            title: "No Android device detected".to_string(),
            cause: "ADB did not find an active device matching the target.".to_string(),
            confidence: "high".to_string(),
            next_steps: vec![
                "Verify the USB cable supports data transfer (not charge-only).".to_string(),
                "Confirm USB Debugging is enabled in Developer Options.".to_string(),
                "Open the Devices page and run USB troubleshooting if needed.".to_string(),
            ],
        });
    }

    if lower.contains("more than one device") || lower.contains("multiple devices") {
        return Some(DiagnosticExplanation {
            code: "MULTIPLE_DEVICES".to_string(),
            title: "Choose a device".to_string(),
            cause:
                "Multiple Android devices are connected and no single target serial was resolved."
                    .to_string(),
            confidence: "high".to_string(),
            next_steps: vec![
                "Select a specific device on the Devices or Overview page.".to_string()
            ],
        });
    }

    if lower.contains("adb server version") && lower.contains("doesn't match") {
        return Some(DiagnosticExplanation {
            code: "ADB_VERSION_MISMATCH".to_string(),
            title: "ADB versions may conflict".to_string(),
            cause: "Another ADB server version is already running on this computer.".to_string(),
            confidence: "high".to_string(),
            next_steps: vec![
                "Check Settings -> Toolchain to see which ADB binary ScrDeck is using."
                    .to_string(),
                "Use ADB Tools -> Restart ADB Server (with confirmation) if you want to align the running server.".to_string(),
            ],
        });
    }

    if lower.contains("unknown option") || lower.contains("unrecognized option") {
        return Some(DiagnosticExplanation {
            code: "UNSUPPORTED_FLAG".to_string(),
            title: "This option is not supported by the installed version".to_string(),
            cause: "scrcpy rejected one of the command-line flags.".to_string(),
            confidence: "high".to_string(),
            next_steps: vec![
                "Inspect the Command Builder preview and remove or update the unsupported flag."
                    .to_string(),
                "Check your installed scrcpy version in Settings or About.".to_string(),
            ],
        });
    }

    if lower.contains("audio") && (lower.contains("not supported") || lower.contains("android 11"))
    {
        return Some(DiagnosticExplanation {
            code: "UNSUPPORTED_AUDIO".to_string(),
            title: "Audio forwarding may not work on this device".to_string(),
            cause: "Audio forwarding requires Android 11 (API 30) or newer and device support."
                .to_string(),
            confidence: "medium".to_string(),
            next_steps: vec![
                "Disable Audio forwarding in Mirroring Studio and start the session again."
                    .to_string(),
            ],
        });
    }

    if lower.contains("could not open output") || lower.contains("failed to open output") {
        return Some(DiagnosticExplanation {
            code: "RECORDING_PATH_ERROR".to_string(),
            title: "Recording output file could not be opened".to_string(),
            cause: "scrcpy could not create or write to the configured recording path.".to_string(),
            confidence: "high".to_string(),
            next_steps: vec![
                "Verify the destination folder exists and is writable.".to_string(),
                "Choose a valid .mp4 or .mkv filename in Mirroring Studio.".to_string(),
            ],
        });
    }

    if lower.contains("could not open video encoder") || lower.contains("mediacodec") {
        return Some(DiagnosticExplanation {
            code: "ENCODER_ERROR".to_string(),
            title: "Device video encoder rejected the configuration".to_string(),
            cause: "The selected codec, resolution, or FPS may not be supported by this device."
                .to_string(),
            confidence: "medium".to_string(),
            next_steps: vec![
                "Switch Video Codec back to H.264 or lower Max Size / Max FPS.".to_string(),
            ],
        });
    }

    if let Some(code) = exit_code {
        if code != 0 {
            return Some(DiagnosticExplanation {
                code: format!("SCRCPY_EXIT_{}", code),
                title: format!("Scrcpy exited with code {}", code),
                cause: "The scrcpy process ended with a non-zero exit status.".to_string(),
                confidence: "possible".to_string(),
                next_steps: vec![
                    "Review the raw stderr output below for the exact upstream message."
                        .to_string(),
                    "Verify the device remained connected and unlocked.".to_string(),
                ],
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_adb_devices_mixed_states_and_crlf() {
        let sample = "List of devices attached\r\n\
* daemon not running; starting now at tcp:5037\r\n\
* daemon started successfully\r\n\
R58M12345AB\tdevice usb:1-2 product:beyond1lte model:SM_G973F device:beyond1 transport_id:1\r\n\
192.168.1.42:5555\tunauthorized transport_id:2\r\n\
emulator-5554\toffline transport_id:3\r\n\
adb-R58M12345AB-xyz._adb-tls-connect._tcp\tdevice model:Pixel_8 transport_id:4\r\n\
0123456789ABCDEF\tno permissions (verify udev rules) transport_id:5\r\n";
        let devices = parse_adb_devices(sample);
        assert_eq!(devices.len(), 5);

        assert_eq!(devices[0].serial, "R58M12345AB");
        assert_eq!(devices[0].adb_state, AdbState::Device);
        assert_eq!(devices[0].transport, DeviceTransport::Usb);
        assert_eq!(devices[0].model.as_deref(), Some("SM G973F"));

        assert_eq!(devices[1].serial, "192.168.1.42:5555");
        assert_eq!(devices[1].adb_state, AdbState::Unauthorized);
        assert_eq!(devices[1].transport, DeviceTransport::TcpIp);

        assert_eq!(devices[2].serial, "emulator-5554");
        assert_eq!(devices[2].adb_state, AdbState::Offline);
        assert_eq!(devices[2].transport, DeviceTransport::Emulator);

        assert_eq!(devices[3].transport, DeviceTransport::TcpIp);
        assert_eq!(devices[3].model.as_deref(), Some("Pixel 8"));

        assert_eq!(devices[4].adb_state, AdbState::NoPermission);
    }

    #[test]
    fn test_parse_adb_devices_empty_and_malformed() {
        assert!(parse_adb_devices("List of devices attached\n\n").is_empty());
        assert!(parse_adb_devices("garbage_single_token\n").is_empty());
    }

    #[test]
    fn test_parse_scrcpy_and_adb_versions() {
        let scrcpy_out = "scrcpy 5.0.1 <https://github.com/Genymobile/scrcpy>\r\nDependencies:\r\n";
        assert_eq!(parse_scrcpy_version(scrcpy_out).as_deref(), Some("5.0.1"));
        assert_eq!(parse_scrcpy_version("not scrcpy output"), None);

        let adb_out = "Android Debug Bridge version 1.0.41\r\nVersion 35.0.2-12147458\r\n";
        assert_eq!(parse_adb_version(adb_out).as_deref(), Some("1.0.41"));
        assert_eq!(parse_adb_version("random text"), None);
    }

    #[test]
    fn test_parse_scrcpy_help_capabilities_v2_and_v5_fixtures() {
        let help_v5 = "\
    -m, --max-size=value
        --max-fps=value
        --video-codec=name
        --no-audio
    -S, --turn-screen-off
    -w, --stay-awake
    -r, --record=file
        --display-orientation=value
";
        let caps = parse_scrcpy_help(help_v5, Some("5.0.1".to_string()));
        assert!(caps.parsed_successfully);
        assert!(caps.supports_max_size);
        assert!(caps.supports_max_fps);
        assert!(caps.supports_video_codec);
        assert!(caps.supports_audio);
        assert!(caps.supports_record);
        assert!(caps.supports_orientation);
        assert!(!caps.supports_fullscreen);

        let empty_caps = parse_scrcpy_help("no flags here", None);
        assert!(!empty_caps.parsed_successfully);
    }

    #[test]
    fn test_parse_and_quote_windows_cmdline_roundtrip() {
        let cases = vec![
            vec!["--window-title", "My Phone", "--max-fps=60"],
            vec!["--record", r"C:\My Recordings\test 1\"],
            vec!["--window-title", r#"Quote "Here" & \Slash\"#],
            vec!["--window-title", "Unicode 📱 Workspace"],
            vec!["--empty", ""],
        ];

        for case in cases {
            let joined: String = case
                .iter()
                .map(|a| quote_windows_arg(a))
                .collect::<Vec<_>>()
                .join(" ");
            let parsed = parse_windows_cmdline(&joined).unwrap();
            assert_eq!(parsed, case, "Failed roundtrip for {:?}", case);
        }

        assert!(parse_windows_cmdline(r#"--window-title "Unclosed"#).is_err());
        assert!(parse_windows_cmdline("--flag\n--other").is_err());
        assert!(parse_windows_cmdline("--flag\0").is_err());
    }

    #[test]
    fn test_redact_sensitive_text_and_json() {
        let raw = r"Connected R58M12345AB at 192.168.1.55:5555 from C:\Users\Ronn\AppData\Local\ScrDeck pairing code: 123456";
        let redacted = redact_sensitive_text(raw, &["R58M12345AB".to_string()]);
        assert!(!redacted.contains("R58M12345AB"));
        assert!(!redacted.contains("192.168.1.55"));
        assert!(!redacted.contains("Ronn"));
        assert!(!redacted.contains("123456"));

        let mut json_val = serde_json::json!({
            "path": r"C:\Users\Alice\AppData\Local\ScrDeck\scrcpy.exe",
            "serial": "PIXEL9PRO123",
            "msg": "adb pair 192.168.0.10:37123 654321"
        });
        redact_json_value(&mut json_val, &["PIXEL9PRO123".to_string()]);
        let serialized = serde_json::to_string_pretty(&json_val).unwrap();
        assert!(!serialized.contains("Alice"));
        assert!(!serialized.contains("PIXEL9PRO123"));
        assert!(!serialized.contains("192.168.0.10"));
        assert!(!serialized.contains("654321"));
        assert!(serde_json::from_str::<Value>(&serialized).is_ok());
    }

    #[test]
    fn test_classify_error_mappings() {
        assert_eq!(
            classify_error("ERROR: Device is unauthorized", Some(1))
                .unwrap()
                .code,
            "ADB_UNAUTHORIZED"
        );
        assert_eq!(
            classify_error("error: device offline", Some(1))
                .unwrap()
                .code,
            "ADB_OFFLINE"
        );
        assert_eq!(
            classify_error("adb server version (41) doesn't match this client (39)", Some(1))
                .unwrap()
                .code,
            "ADB_VERSION_MISMATCH"
        );
        assert_eq!(
            classify_error("scrcpy: unrecognized option '--bad-flag'", Some(2))
                .unwrap()
                .code,
            "UNSUPPORTED_FLAG"
        );
        assert_eq!(
            classify_error("Unknown crash", Some(42)).unwrap().code,
            "SCRCPY_EXIT_42"
        );
        assert!(classify_error("", Some(0)).is_none());
    }
}
