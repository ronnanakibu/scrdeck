use crate::models::{ActivityLogEntry, LogSeverity, SessionInfo, SessionStatus};
use crate::parsers::classify_error;
use chrono::Utc;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use uuid::Uuid;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct ManagedSession {
    pub info: SessionInfo,
    pub child: Option<Child>,
}

pub struct ProcessManager {
    pub sessions: Arc<Mutex<HashMap<String, ManagedSession>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let guard = self.sessions.lock().await;
        let mut list: Vec<SessionInfo> = guard.values().map(|s| s.info.clone()).collect();
        list.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        list
    }

    pub async fn start_session(
        &self,
        app: AppHandle,
        executable: String,
        argv: Vec<String>,
        device_serial: String,
        preset_id: Option<String>,
        log_sink: Arc<Mutex<Vec<ActivityLogEntry>>>,
    ) -> Result<SessionInfo, String> {
        let session_id = Uuid::new_v4().to_string();
        let started_at = Utc::now().to_rfc3339();

        let mut info = SessionInfo {
            session_id: session_id.clone(),
            device_serial: device_serial.clone(),
            preset_id,
            started_at,
            ended_at: None,
            process_id: None,
            executable_path: executable.clone(),
            argv: argv.clone(),
            exit_code: None,
            status: SessionStatus::Starting,
            error_summary: None,
        };

        let mut cmd = Command::new(&executable);
        cmd.args(&argv)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                info.status = SessionStatus::Failed;
                info.ended_at = Some(Utc::now().to_rfc3339());
                info.error_summary = Some(format!("Failed to start scrcpy: {}", e));
                let _ = app.emit("session-updated", &info);
                return Err(format!("Failed to start scrcpy: {}", e));
            }
        };

        info.process_id = child.id();
        info.status = SessionStatus::Running;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        {
            let mut guard = self.sessions.lock().await;
            guard.insert(
                session_id.clone(),
                ManagedSession {
                    info: info.clone(),
                    child: Some(child),
                },
            );
        }

        let _ = app.emit("session-updated", &info);

        let start_log = ActivityLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            source: "scrcpy".to_string(),
            device_serial: Some(device_serial.clone()),
            session_id: Some(session_id.clone()),
            severity: LogSeverity::Info,
            title: "Mirroring session started".to_string(),
            message: format!("Launched scrcpy (PID {:?}) for {}", info.process_id, device_serial),
            raw_output: Some(format!("{} {}", executable, argv.join(" "))),
            explanation: None,
        };
        {
            let mut logs = log_sink.lock().await;
            logs.insert(0, start_log.clone());
        }
        let _ = app.emit("log-entry", &start_log);

        // Spawn stdout reader
        if let Some(out) = stdout {
            let app_c = app.clone();
            let sid = session_id.clone();
            let dev = device_serial.clone();
            let sink = log_sink.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let entry = ActivityLogEntry {
                        id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        source: "scrcpy".to_string(),
                        device_serial: Some(dev.clone()),
                        session_id: Some(sid.clone()),
                        severity: LogSeverity::Debug,
                        title: "scrcpy output".to_string(),
                        message: line.clone(),
                        raw_output: Some(line),
                        explanation: None,
                    };
                    {
                        let mut logs = sink.lock().await;
                        logs.insert(0, entry.clone());
                    }
                    let _ = app_c.emit("log-entry", &entry);
                }
            });
        }

        // Spawn stderr collector + process exit watcher
        let sessions_arc = self.sessions.clone();
        let app_w = app.clone();
        let sid_w = session_id.clone();
        let dev_w = device_serial.clone();
        let sink_w = log_sink.clone();

        tokio::spawn(async move {
            let mut stderr_buf = String::new();
            if let Some(err) = stderr {
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    stderr_buf.push_str(&line);
                    stderr_buf.push('\n');
                    let entry = ActivityLogEntry {
                        id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        source: "scrcpy".to_string(),
                        device_serial: Some(dev_w.clone()),
                        session_id: Some(sid_w.clone()),
                        severity: LogSeverity::Warning,
                        title: "scrcpy stderr".to_string(),
                        message: line.clone(),
                        raw_output: Some(line),
                        explanation: None,
                    };
                    {
                        let mut logs = sink_w.lock().await;
                        logs.insert(0, entry.clone());
                    }
                    let _ = app_w.emit("log-entry", &entry);
                }
            }

            // Wait for child exit
            let exit_status = {
                let mut child_opt = {
                    let mut guard = sessions_arc.lock().await;
                    guard.get_mut(&sid_w).and_then(|m| m.child.take())
                };
                if let Some(ref mut c) = child_opt {
                    c.wait().await.ok()
                } else {
                    None
                }
            };

            let code = exit_status.and_then(|s| s.code());
            let updated_info = {
                let mut guard = sessions_arc.lock().await;
                if let Some(m) = guard.get_mut(&sid_w) {
                    let was_stopping = m.info.status == SessionStatus::Stopping;
                    m.info.ended_at = Some(Utc::now().to_rfc3339());
                    m.info.exit_code = code;
                    if was_stopping || code == Some(0) {
                        m.info.status = SessionStatus::Exited;
                    } else {
                        m.info.status = SessionStatus::Failed;
                        m.info.error_summary = Some(if stderr_buf.trim().is_empty() {
                            format!("scrcpy exited with code {:?}", code)
                        } else {
                            stderr_buf.trim().to_string()
                        });
                    }
                    Some(m.info.clone())
                } else {
                    None
                }
            };

            if let Some(u) = updated_info {
                let _ = app_w.emit("session-updated", &u);
                let explanation = if u.status == SessionStatus::Failed {
                    classify_error(&stderr_buf, code)
                } else {
                    None
                };
                let exit_log = ActivityLogEntry {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                    source: "scrcpy".to_string(),
                    device_serial: Some(dev_w),
                    session_id: Some(sid_w),
                    severity: if u.status == SessionStatus::Failed {
                        LogSeverity::Error
                    } else {
                        LogSeverity::Info
                    },
                    title: if u.status == SessionStatus::Failed {
                        "Mirroring session failed".to_string()
                    } else {
                        "Mirroring session ended".to_string()
                    },
                    message: format!("Session exited with code {:?}", code),
                    raw_output: if stderr_buf.trim().is_empty() {
                        None
                    } else {
                        Some(stderr_buf)
                    },
                    explanation,
                };
                {
                    let mut logs = sink_w.lock().await;
                    logs.insert(0, exit_log.clone());
                }
                let _ = app_w.emit("log-entry", &exit_log);
            }
        });

        Ok(info)
    }

    pub async fn stop_session(&self, app: &AppHandle, session_id: &str) -> Result<SessionInfo, String> {
        let mut guard = self.sessions.lock().await;
        let managed = guard
            .get_mut(session_id)
            .ok_or_else(|| format!("Session {} not found", session_id))?;

        managed.info.status = SessionStatus::Stopping;
        if let Some(ref mut child) = managed.child {
            let _ = child.kill().await;
        }
        let info = managed.info.clone();
        let _ = app.emit("session-updated", &info);
        Ok(info)
    }

    pub async fn stop_all_sessions(&self) {
        let mut guard = self.sessions.lock().await;
        for managed in guard.values_mut() {
            if managed.info.status == SessionStatus::Running
                || managed.info.status == SessionStatus::Starting
            {
                managed.info.status = SessionStatus::Stopping;
                if let Some(ref mut child) = managed.child {
                    let _ = child.start_kill();
                }
            }
        }
    }
}
