use crate::patcher::PatchEngine;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize)]
pub struct AppStatus {
    pub is_active: bool,
    pub os_build: String,
    pub persistence_enabled: bool,
    pub defender_excluded: bool,
}

#[tauri::command]
pub fn get_system_status() -> Result<AppStatus, String> {
    #[cfg(target_os = "windows")]
    {
        let build_opt = crate::patcher::PatchEngine::detect_build();
        let os_build = match build_opt {
            Some(b) => format!("Windows (Build {})", b),
            None => "Windows (Unknown)".into(),
        };
        let is_active = match build_opt {
            Some(b) => crate::patcher::PatchEngine::is_patched(Path::new(r"C:\Windows\System32\termsrv.dll"), b),
            None => false,
        };

        Ok(AppStatus {
            is_active,
            os_build,
            persistence_enabled: false,
            defender_excluded: false,
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
        Ok(format!("Mock Defender exclusion: {}", enable))
    }
}

#[tauri::command]
pub fn set_persistence(enable: bool) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let task_name = "RDP-MultiSession-Persistence";
        if enable {
            // Setup a scheduled task running as SYSTEM
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
            let script = format!("schtasks /delete /tn '{}' /f", task_name);
            let output = Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(&["/c", &script])
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
        Ok(format!("Mock Persistence: {}", enable))
    }
}

#[tauri::command]
pub async fn check_updates() -> Result<String, String> {
    // 1. URL List for mirror retries
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
                    Ok(text) => return Ok(format!("Fetched correctly from {}: {} bytes", url, text.len())),
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
        let dll_path    = Path::new(r"C:\Windows\System32\termsrv.dll");
        let backup_path = Path::new(r"C:\Windows\System32\termsrv.dll.rdp_backup");

        let build = PatchEngine::detect_build()
            .ok_or("Could not detect Windows build number")?;

        // 1. Stop TermService (Remote Desktop Services)
        let _ = Command::new("net")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["stop", "TermService", "/y"])
            .output();

        // 2. Take ownership and grant full control via icacls
        let _ = Command::new("takeown")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["/F", r"C:\Windows\System32\termsrv.dll"])
            .output();

        let _ = Command::new("icacls")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&[r"C:\Windows\System32\termsrv.dll", "/grant", "Administrators:F", "/Q"])
            .output();

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
        let dll_path    = Path::new(r"C:\Windows\System32\termsrv.dll");
        let backup_path = Path::new(r"C:\Windows\System32\termsrv.dll.rdp_backup");

        // 1. Stop service
        let _ = Command::new("net")
            .creation_flags(CREATE_NO_WINDOW)
            .args(&["stop", "TermService", "/y"])
            .output();

        // 2. Restore from backup
        let result = PatchEngine::restore(dll_path, backup_path)?;

        // 3. Restart service
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
    let file = desktop.join("RDP_Manager_Logs.txt");
    
    std::fs::write(&file, log_content).map_err(|e| e.to_string())?;
    Ok(file.to_string_lossy().to_string())
}
