use crate::models::{AdbState, Device, DriverDiagnosis, OemDriverLink};
use crate::parsers::parse_adb_devices;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub async fn run_adb(adb_path: &str, args: &[&str], timeout_secs: u64) -> Result<(String, String, i32), String> {
    let exe = Path::new(adb_path);
    if !exe.is_file() {
        return Err(format!("ADB executable not found at {}", adb_path));
    }

    let mut cmd = Command::new(exe);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = timeout(Duration::from_secs(timeout_secs), cmd.output())
        .await
        .map_err(|_| format!("ADB command timed out after {}s", timeout_secs))?
        .map_err(|e| format!("Failed to run ADB: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    Ok((stdout, stderr, code))
}

pub async fn get_devices(adb_path: &str) -> Result<Vec<Device>, String> {
    let (stdout, stderr, code) = run_adb(adb_path, &["devices", "-l"], 8).await?;
    if code != 0 && stdout.trim().is_empty() {
        return Err(if stderr.trim().is_empty() {
            format!("adb devices exited with code {}", code)
        } else {
            stderr
        });
    }

    let mut devices = parse_adb_devices(&stdout);

    for dev in &mut devices {
        if dev.adb_state == AdbState::Device {
            enrich_device_metadata(adb_path, dev).await;
        }
    }

    Ok(devices)
}

pub async fn enrich_device_metadata(adb_path: &str, dev: &mut Device) {
    if let Ok((out, _, 0)) = run_adb(
        adb_path,
        &["-s", &dev.serial, "shell", "getprop", "ro.build.version.release"],
        4,
    )
    .await
    {
        let val = out.trim();
        if !val.is_empty() {
            dev.android_release = Some(val.to_string());
        }
    }

    if let Ok((out, _, 0)) = run_adb(
        adb_path,
        &["-s", &dev.serial, "shell", "getprop", "ro.build.version.sdk"],
        4,
    )
    .await
    {
        if let Ok(api) = out.trim().parse::<u32>() {
            dev.api_level = Some(api);
        }
    }

    if let Ok((out, _, 0)) = run_adb(
        adb_path,
        &["-s", &dev.serial, "shell", "dumpsys", "battery"],
        4,
    )
    .await
    {
        let mut level: Option<u8> = None;
        let mut ac = false;
        let mut usb = false;
        let mut wireless = false;
        for line in out.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("level:") {
                if let Ok(v) = rest.trim().parse::<u8>() {
                    level = Some(v);
                }
            } else if trimmed == "AC powered: true" {
                ac = true;
            } else if trimmed == "USB powered: true" {
                usb = true;
            } else if trimmed == "Wireless powered: true" {
                wireless = true;
            }
        }
        dev.battery_percent = level;
        if ac || usb || wireless {
            dev.charging_state = Some("Charging".to_string());
        } else if level.is_some() {
            dev.charging_state = Some("Discharging".to_string());
        }
    }
}

pub async fn start_server(adb_path: &str) -> Result<String, String> {
    let (stdout, stderr, code) = run_adb(adb_path, &["start-server"], 10).await?;
    if code != 0 {
        return Err(format!("adb start-server failed ({}): {}", code, stderr));
    }
    Ok(format!("{}{}", stdout, stderr).trim().to_string())
}

pub async fn restart_server_confirmed(adb_path: &str) -> Result<String, String> {
    let _ = run_adb(adb_path, &["kill-server"], 8).await;
    start_server(adb_path).await
}

pub async fn pair_wireless(
    adb_path: &str,
    address: &str,
    pairing_code: &str,
) -> Result<String, String> {
    let addr = address.trim();
    let code = pairing_code.trim();
    if addr.is_empty() || code.is_empty() {
        return Err("IP:port address and pairing code are required.".to_string());
    }

    // Never include pairing_code in returned error or log strings
    let (stdout, stderr, exit_code) = run_adb(adb_path, &["pair", addr, code], 15).await?;
    let combined = format!("{} {}", stdout, stderr);
    if exit_code != 0 || combined.to_lowercase().contains("failed") {
        return Err(format!(
            "Wireless pairing with {} did not complete. Verify the IP, pairing port, and 6-digit code.",
            addr
        ));
    }
    Ok(format!("Successfully paired with {}", addr))
}

pub async fn connect_wireless(adb_path: &str, address: &str) -> Result<String, String> {
    let addr = address.trim();
    if addr.is_empty() {
        return Err("Connection IP:port address is required.".to_string());
    }
    let (stdout, stderr, exit_code) = run_adb(adb_path, &["connect", addr], 10).await?;
    let combined = format!("{} {}", stdout, stderr).trim().to_string();
    if exit_code != 0 || combined.to_lowercase().contains("cannot connect") || combined.to_lowercase().contains("failed") {
        return Err(combined);
    }
    Ok(combined)
}

pub async fn disconnect_wireless(adb_path: &str, serial: &str) -> Result<String, String> {
    let target = serial.trim();
    if target.is_empty() {
        return Err("Target serial is required.".to_string());
    }
    let (stdout, stderr, exit_code) = run_adb(adb_path, &["disconnect", target], 8).await?;
    let combined = format!("{} {}", stdout, stderr).trim().to_string();
    if exit_code != 0 {
        return Err(combined);
    }
    Ok(combined)
}

pub fn verified_oem_driver_links() -> Vec<OemDriverLink> {
    vec![
        OemDriverLink {
            vendor: "Android Official Directory".to_string(),
            label: "Android OEM USB Drivers Directory".to_string(),
            url: "https://developer.android.com/studio/run/oem-usb".to_string(),
            notes: "Official Google directory linking to manufacturer USB driver pages.".to_string(),
        },
        OemDriverLink {
            vendor: "Google Pixel / Nexus".to_string(),
            label: "Google USB Driver".to_string(),
            url: "https://developer.android.com/studio/run/win-usb".to_string(),
            notes: "Official Google USB driver for Pixel and AOSP reference devices.".to_string(),
        },
        OemDriverLink {
            vendor: "Samsung".to_string(),
            label: "Samsung Android USB Driver".to_string(),
            url: "https://developer.samsung.com/android-usb-driver".to_string(),
            notes: "Official Samsung Developer USB driver package for Windows.".to_string(),
        },
        OemDriverLink {
            vendor: "Motorola".to_string(),
            label: "Motorola Device Manager".to_string(),
            url: "https://en-us.support.motorola.com/app/usb-drivers".to_string(),
            notes: "Official Motorola Windows USB driver support page.".to_string(),
        },
    ]
}

pub async fn diagnose_usb_connection(adb_path: Option<&str>) -> DriverDiagnosis {
    if let Some(adb) = adb_path {
        if let Ok(devices) = get_devices(adb).await {
            if devices.iter().any(|d| d.adb_state == AdbState::Device) {
                return DriverDiagnosis {
                    status: "ready".to_string(),
                    title: "ADB device is ready".to_string(),
                    confidence: "high".to_string(),
                    summary: "At least one authorized Android device is detected by ADB. No driver installation is needed.".to_string(),
                    checklist: vec![
                        "Select your device on the Devices or Overview page to start mirroring.".to_string(),
                    ],
                    oem_links: vec![],
                };
            }
            if devices.iter().any(|d| d.adb_state == AdbState::Unauthorized) {
                return DriverDiagnosis {
                    status: "unauthorized".to_string(),
                    title: "Your phone needs permission".to_string(),
                    confidence: "high".to_string(),
                    summary: "ADB sees your phone, so the USB driver is working. You only need to approve the RSA debugging prompt on the phone.".to_string(),
                    checklist: vec![
                        "Unlock your Android phone.".to_string(),
                        "Tap 'Allow' on the 'Allow USB debugging?' dialog.".to_string(),
                        "Click Refresh Devices in ScrDeck.".to_string(),
                    ],
                    oem_links: vec![],
                };
            }
            if devices.iter().any(|d| d.adb_state == AdbState::Offline) {
                return DriverDiagnosis {
                    status: "offline".to_string(),
                    title: "Device connection is not responding".to_string(),
                    confidence: "high".to_string(),
                    summary: "ADB sees the device as offline. Try reconnecting the cable or toggling USB Debugging before changing drivers.".to_string(),
                    checklist: vec![
                        "Unplug and reconnect the USB cable.".to_string(),
                        "Toggle USB Debugging off and back on in Developer Options.".to_string(),
                        "Try a different USB port directly on your PC.".to_string(),
                    ],
                    oem_links: vec![],
                };
            }
        }
    }

    DriverDiagnosis {
        status: "no_device".to_string(),
        title: "No Android device detected — possible cable, setting, or OEM driver issue".to_string(),
        confidence: "possible".to_string(),
        summary: "ADB did not list any connected device. Check your cable and phone settings first; if Windows recognizes the phone for charging/MTP but ADB still cannot see it, an OEM USB driver may be needed.".to_string(),
        checklist: vec![
            "Confirm you are using a data-capable USB cable (some cables only charge).".to_string(),
            "Unlock the phone and set USB mode to File Transfer (MTP) or PTP.".to_string(),
            "Verify Developer Options -> USB Debugging is enabled.".to_string(),
            "If Windows Device Manager shows an unknown 'ADB Interface' or yellow warning icon, install the official OEM USB driver from the links below.".to_string(),
        ],
        oem_links: verified_oem_driver_links(),
    }
}
