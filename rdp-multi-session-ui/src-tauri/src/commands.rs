use serde::Serialize;
#[cfg(target_os = "windows")]
use std::process::Command;

#[derive(Serialize)]
pub struct AppStatus {
    pub is_active: bool,
    pub os_build: String,
    pub persistence_enabled: bool,
    pub defender_excluded: bool,
}

// ============================================================
//  Windows-only helpers
// ============================================================
#[cfg(target_os = "windows")]
mod win {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    /// Check whether a scheduled task exists.
    pub fn has_scheduled_task(task_name: &str) -> bool {
        let output = Command::new("schtasks")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["/query", "/tn", task_name])
            .output();
        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    /// Check whether termsrv.dll is in the Defender exclusion list.
    pub fn has_defender_exclusion() -> bool {
        let script = "(Get-MpPreference).ExclusionPath -contains 'C:\\Windows\\System32\\termsrv.dll'";
        let output = Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["-NoProfile", "-NonInteractive", "-Command", script])
            .output();
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                stdout.eq_ignore_ascii_case("true")
            }
            Err(_) => false,
        }
    }

    /// Stop TermService, returning an error if it genuinely fails.
    /// "service is not started" is treated as success.
    pub fn stop_term_service() -> Result<(), String> {
        let output = Command::new("net")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["stop", "TermService", "/y"])
            .output()
            .map_err(|e| format!("Failed to run 'net stop TermService': {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("is not started") {
                return Err(format!("Failed to stop TermService: {}", stderr));
            }
        }
        Ok(())
    }

    /// Take ownership and grant Administrators full control on termsrv.dll.
    pub fn grant_dll_permissions() {
        let _ = Command::new("takeown")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["/F", r"C:\Windows\System32\termsrv.dll"])
            .output();

        let _ = Command::new("icacls")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&[r"C:\Windows\System32\termsrv.dll", "/grant", "Administrators:F", "/Q"])
            .output();
    }
}

// ============================================================
//  Tauri commands
// ============================================================

#[tauri::command]
pub fn get_system_status() -> Result<AppStatus, String> {
    #[cfg(target_os = "windows")]
    {
        use crate::patcher::PatchEngine;
        use std::path::Path;

        let build_opt = PatchEngine::detect_build();
        let os_build = match build_opt {
            Some(b) => format!("Windows (Build {})", b),
            None => "Windows (Unknown)".into(),
        };
        let is_active = match build_opt {
            Some(b) => PatchEngine::is_patched(Path::new(r"C:\Windows\System32\termsrv.dll"), b),
            None => false,
        };

        Ok(AppStatus {
            is_active,
            os_build,
            persistence_enabled: win::has_scheduled_task("RDP-MultiSession-Persistence"),
            defender_excluded: win::has_defender_exclusion(),
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(AppStatus {
            is_active: false,
            os_build: "Mock OS (Not Windows)".into(),
            persistence_enabled: false,
            defender_excluded: false,
        })
    }
}

#[tauri::command]
pub fn set_defender_exclusion(enable: bool) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let script = if enable {
            "Add-MpPreference -ExclusionPath 'C:\\Windows\\System32\\termsrv.dll'"
        } else {
            "Remove-MpPreference -ExclusionPath 'C:\\Windows\\System32\\termsrv.dll'"
        };

        let output = Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["-NoProfile", "-NonInteractive", "-Command", script])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(format!("Defender exclusion {}", if enable { "added" } else { "removed" }))
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable;
        Ok("Mock Defender exclusion (non-Windows build)".into())
    }
}

#[tauri::command]
pub fn set_persistence(enable: bool) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let task_name = "RDP-MultiSession-Persistence";
        if enable {
            let exe_path = std::env::current_exe()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let tr_arg = format!("\"{}\" --daemon", exe_path);
            let output = Command::new("schtasks")
                .creation_flags(CREATE_NO_WINDOW)
                .args(&[
                    "/create",
                    "/tn", task_name,
                    "/tr", &tr_arg,
                    "/sc", "onstart",
                    "/ru", "SYSTEM",
                    "/rl", "HIGHEST",
                    "/f"
                ])
                .output()
                .map_err(|e| e.to_string())?;

            if output.status.success() {
                Ok("Persistence scheduled task created.".into())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        } else {
            let output = Command::new("schtasks")
                .creation_flags(CREATE_NO_WINDOW)
                .args(&["/delete", "/tn", task_name, "/f"])
                .output()
                .map_err(|e| e.to_string())?;

            if output.status.success() || String::from_utf8_lossy(&output.stderr).contains("ERROR: The specified task name") {
                Ok("Persistence scheduled task removed.".into())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable;
        Ok("Mock Persistence (non-Windows build)".into())
    }
}

#[tauri::command]
pub async fn check_updates() -> Result<String, String> {
    let url_direct = "https://raw.githubusercontent.com/malnwaihi/RDP-MultiSession-Enabler/main/termsrv_offsets.json";
    let url_mirror_1 = "https://ghp.ci/https://raw.githubusercontent.com/malnwaihi/RDP-MultiSession-Enabler/main/termsrv_offsets.json";
    let url_mirror_2 = "https://mirror.ghproxy.com/https://raw.githubusercontent.com/malnwaihi/RDP-MultiSession-Enabler/main/termsrv_offsets.json";

    let urls = vec![url_direct, url_mirror_1, url_mirror_2];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    for url in urls {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.text().await {
                    Ok(text) => {
                        // Validate it's parseable JSON
                        match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(_) => {
                                return Ok(format!(
                                    "Fetched and validated from {}: {} bytes",
                                    url,
                                    text.len()
                                ));
                            }
                            Err(e) => {
                                return Err(format!(
                                    "Fetched from {} but JSON is invalid: {}",
                                    url, e
                                ));
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
            _ => continue,
        }
    }

    Err("Failed to fetch updates from all mirrors.".into())
}

#[tauri::command]
pub fn patch_rdp() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use crate::patcher::PatchEngine;
        use std::path::Path;
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let dll_path    = Path::new(r"C:\Windows\System32\termsrv.dll");
        let backup_path = Path::new(r"C:\Windows\System32\termsrv.dll.rdp_backup");

        let build = PatchEngine::detect_build()
            .ok_or("Could not detect Windows build number")?;

        // 1. Stop TermService — must succeed before we can modify the DLL
        win::stop_term_service()?;

        // 2. Take ownership and grant full control
        win::grant_dll_permissions();

        // 3. Apply hex patch
        let result = PatchEngine::patch(dll_path, backup_path, build)?;

        // 4. Restart TermService
        let _ = Command::new("net")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["start", "TermService"])
            .output();

        Ok(result)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok("Mock patch applied successfully (non-Windows build)".into())
    }
}

#[tauri::command]
pub fn restore_rdp() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use crate::patcher::PatchEngine;
        use std::path::Path;
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let dll_path    = Path::new(r"C:\Windows\System32\termsrv.dll");
        let backup_path = Path::new(r"C:\Windows\System32\termsrv.dll.rdp_backup");

        // 1. Stop service — must succeed before we can overwrite the DLL
        win::stop_term_service()?;

        // 2. Take ownership and grant full control (same as patch_rdp)
        win::grant_dll_permissions();

        // 3. Restore from backup
        let result = PatchEngine::restore(dll_path, backup_path)?;

        // 4. Restart service
        let _ = Command::new("net")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["start", "TermService"])
            .output();

        Ok(result)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok("Mock restore successful (non-Windows build)".into())
    }
}

#[tauri::command]
pub fn save_logs(log_content: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let base = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());

    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    let desktop = std::path::Path::new(&base).join("Desktop");

    // Use timestamp to avoid overwriting previous logs
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let filename = format!("RDP_Manager_Logs_{}.txt", timestamp);
    let file = desktop.join(&filename);

    std::fs::write(&file, log_content).map_err(|e| e.to_string())?;
    Ok(file.to_string_lossy().to_string())
}
