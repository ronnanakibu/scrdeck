use crate::models::{
    IntegrityStatus, ScrcpyCapabilities, ToolchainInfo, ToolchainSourceType,
};
use crate::parsers::{parse_adb_version, parse_scrcpy_help, parse_scrcpy_version};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const ALLOWED_DOWNLOAD_HOSTS: &[&str] = &[
    "api.github.com",
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
    "dl.google.com",
];

pub fn is_allowed_url(url: &str) -> bool {
    if let Some(rest) = url.strip_prefix("https://") {
        let host = rest.split('/').next().unwrap_or("").to_lowercase();
        return ALLOWED_DOWNLOAD_HOSTS
            .iter()
            .any(|allowed| host == *allowed || host.ends_with(&format!(".{}", allowed)));
    }
    false
}

pub fn managed_tools_root() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("ScrDeck").join("tools")
}

pub fn find_in_path(binary_name: &str) -> Option<PathBuf> {
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(binary_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn find_common_android_sdk_adb() -> Option<PathBuf> {
    if let Some(local_app_data) = dirs::data_local_dir() {
        let sdk_adb = local_app_data
            .join("Android")
            .join("Sdk")
            .join("platform-tools")
            .join("adb.exe");
        if sdk_adb.is_file() {
            return Some(sdk_adb);
        }
    }
    if let Ok(android_home) = std::env::var("ANDROID_HOME") {
        let p = PathBuf::from(android_home)
            .join("platform-tools")
            .join("adb.exe");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

async fn run_tool_capture(exe: &Path, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new(exe);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = timeout(Duration::from_secs(8), cmd.output())
        .await
        .map_err(|_| format!("Timed out running {:?}", exe))?
        .map_err(|e| format!("Failed to execute {:?}: {}", exe, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if stdout.trim().is_empty() && !stderr.trim().is_empty() {
        Ok(stderr)
    } else {
        Ok(format!("{}\n{}", stdout, stderr))
    }
}

pub async fn detect_toolchain(
    custom_scrcpy: Option<&str>,
    custom_adb: Option<&str>,
) -> (ToolchainInfo, ScrcpyCapabilities) {
    let mut notes = Vec::new();
    let mut source_type = ToolchainSourceType::Missing;
    let mut integrity_status = IntegrityStatus::Unverified;

    // 1. Resolve scrcpy.exe
    let mut scrcpy_path: Option<PathBuf> = None;
    if let Some(custom) = custom_scrcpy {
        let p = PathBuf::from(custom);
        if p.is_file() {
            scrcpy_path = Some(p);
            source_type = ToolchainSourceType::UserSelected;
            integrity_status = IntegrityStatus::UserProvided;
        } else {
            notes.push(format!("Configured custom scrcpy path not found: {}", custom));
        }
    }

    if scrcpy_path.is_none() {
        let managed_current = managed_tools_root().join("current").join("scrcpy.exe");
        if managed_current.is_file() {
            scrcpy_path = Some(managed_current);
            source_type = ToolchainSourceType::ManagedBundle;
            integrity_status = IntegrityStatus::OfficialSourceNoChecksum;
        }
    }

    if scrcpy_path.is_none() {
        if let Some(in_path) = find_in_path("scrcpy.exe") {
            scrcpy_path = Some(in_path);
            source_type = ToolchainSourceType::DetectedSdk;
            integrity_status = IntegrityStatus::UserProvided;
        }
    }

    // 2. Resolve adb.exe (prefer custom -> sibling of scrcpy -> Android SDK -> PATH)
    let mut adb_path: Option<PathBuf> = None;
    if let Some(custom) = custom_adb {
        let p = PathBuf::from(custom);
        if p.is_file() {
            adb_path = Some(p);
        } else {
            notes.push(format!("Configured custom ADB path not found: {}", custom));
        }
    }

    if adb_path.is_none() {
        if let Some(ref sc) = scrcpy_path {
            if let Some(parent) = sc.parent() {
                let sibling_adb = parent.join("adb.exe");
                if sibling_adb.is_file() {
                    adb_path = Some(sibling_adb);
                }
            }
        }
    }

    if adb_path.is_none() {
        if let Some(sdk_adb) = find_common_android_sdk_adb() {
            adb_path = Some(sdk_adb);
        } else if let Some(path_adb) = find_in_path("adb.exe") {
            adb_path = Some(path_adb);
        }
    }

    // 3. Validate scrcpy & build capability registry
    let mut scrcpy_version = None;
    let mut scrcpy_valid = false;
    let mut capabilities = ScrcpyCapabilities::default();

    if let Some(ref sc) = scrcpy_path {
        match run_tool_capture(sc, &["--version"]).await {
            Ok(out) => {
                scrcpy_version = parse_scrcpy_version(&out);
                scrcpy_valid = scrcpy_version.is_some();
                if !scrcpy_valid {
                    notes.push("scrcpy.exe ran, but version output could not be parsed.".to_string());
                }
            }
            Err(e) => {
                notes.push(format!("Failed to validate scrcpy.exe: {}", e));
            }
        }

        if scrcpy_valid {
            match run_tool_capture(sc, &["--help"]).await {
                Ok(help_out) => {
                    capabilities = parse_scrcpy_help(&help_out, scrcpy_version.clone());
                }
                Err(e) => {
                    notes.push(format!("Could not inspect scrcpy --help: {}", e));
                }
            }
        }
    }

    // 4. Validate adb
    let mut adb_version = None;
    let mut adb_valid = false;
    if let Some(ref adb) = adb_path {
        match run_tool_capture(adb, &["version"]).await {
            Ok(out) => {
                adb_version = parse_adb_version(&out);
                adb_valid = adb_version.is_some();
                if !adb_valid {
                    notes.push("adb.exe ran, but version output could not be parsed.".to_string());
                }
            }
            Err(e) => {
                notes.push(format!("Failed to validate adb.exe: {}", e));
            }
        }
    }

    let info = ToolchainInfo {
        scrcpy_path: scrcpy_path.map(|p| p.to_string_lossy().to_string()),
        scrcpy_version,
        adb_path: adb_path.map(|p| p.to_string_lossy().to_string()),
        adb_version,
        source_type,
        architecture: std::env::consts::ARCH.to_string(),
        last_validated_at: Some(Utc::now().to_rfc3339()),
        integrity_status,
        scrcpy_valid,
        adb_valid,
        notes,
    };

    (info, capabilities)
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(serde::Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn install_official_scrcpy_bundle() -> Result<(String, String, IntegrityStatus), String> {
    let api_url = "https://api.github.com/repos/Genymobile/scrcpy/releases/latest";
    if !is_allowed_url(api_url) {
        return Err("Release API URL is not in the official allowlist.".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("ScrDeck/0.1.0")
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let release: GithubRelease = client
        .get(api_url)
        .send()
        .await
        .map_err(|e| format!("Failed to query official scrcpy release: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse release metadata: {}", e))?;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name.starts_with("scrcpy-win64-") && a.name.ends_with(".zip"))
        .ok_or_else(|| {
            format!(
                "No Windows x64 zip asset found in scrcpy release {}.",
                release.tag_name
            )
        })?;

    if !is_allowed_url(&asset.browser_download_url) {
        return Err(format!(
            "Download URL host is not allowlisted: {}",
            asset.browser_download_url
        ));
    }

    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download {}: {}", asset.name, e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read downloaded archive: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256_hex = hex::encode(hasher.finalize());

    let root = managed_tools_root();
    let downloads_dir = root.join("downloads");
    let staging_dir = root.join(format!("staging-{}", release.tag_name));
    let current_dir = root.join("current");
    let backup_dir = root.join("previous");

    fs::create_dir_all(&downloads_dir).map_err(|e| e.to_string())?;
    if staging_dir.exists() {
        let _ = fs::remove_dir_all(&staging_dir);
    }
    fs::create_dir_all(&staging_dir).map_err(|e| e.to_string())?;

    let zip_path = downloads_dir.join(&asset.name);
    {
        let mut f = fs::File::create(&zip_path).map_err(|e| e.to_string())?;
        f.write_all(&bytes).map_err(|e| e.to_string())?;
    }

    // Extract zip safely
    let file = fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let enclosed = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };

        // Strip top-level folder (e.g. scrcpy-win64-v5.0.1/...)
        let components: Vec<_> = enclosed.components().collect();
        let rel_path: PathBuf = if components.len() > 1 {
            components[1..].iter().collect()
        } else {
            continue;
        };

        let out_path = staging_dir.join(&rel_path);
        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = fs::File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    if !staging_dir.join("scrcpy.exe").is_file() {
        let _ = fs::remove_dir_all(&staging_dir);
        return Err("Extracted archive did not contain scrcpy.exe.".to_string());
    }

    // Swap with rollback support
    if backup_dir.exists() {
        let _ = fs::remove_dir_all(&backup_dir);
    }
    if current_dir.exists() {
        fs::rename(&current_dir, &backup_dir).map_err(|e| e.to_string())?;
    }
    if let Err(e) = fs::rename(&staging_dir, &current_dir) {
        if backup_dir.exists() {
            let _ = fs::rename(&backup_dir, &current_dir);
        }
        return Err(format!("Failed to activate extracted toolchain: {}", e));
    }

    Ok((
        release.tag_name,
        sha256_hex,
        IntegrityStatus::OfficialSourceNoChecksum,
    ))
}
