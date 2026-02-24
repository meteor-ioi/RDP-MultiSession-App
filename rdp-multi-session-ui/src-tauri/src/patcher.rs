use std::fs;
use std::path::Path;

/// Windows build version info
#[derive(Debug, Clone)]
pub struct WinBuild {
    pub build_number: u32,
    pub display_name: String,
}

/// A single hex patch rule for a specific Windows build range
#[derive(Debug, Clone)]
pub struct PatchPattern {
    pub description: &'static str,
    pub build_min: u32,
    pub build_max: u32,
    pub search: &'static [u8],
    pub replace: &'static [u8],
}

// =====================================================================
//  Hex patterns extracted from RDP-MultiSession-Enabler.ps1
//  Source: https://github.com/malnwaihi/RDP-MultiSession-Enabler
// =====================================================================
static PATTERNS: &[PatchPattern] = &[
    // Win11 24H2 / 25H2 / Future (Builds 26100+)
    // PS: Search = '8B 81 38 06 00 00 39 81 3C 06 00 00 75'
    //     Replace = 'B8 00 01 00 00 89 81 38 06 00 00 90 EB'
    PatchPattern {
        description: "Windows 11 24H2/25H2/Future (Build 26100+)",
        build_min: 26100,
        build_max: 99999,
        search: &[
            0x8B, 0x81, 0x38, 0x06, 0x00, 0x00, 0x39, 0x81, 0x3C, 0x06, 0x00, 0x00, 0x75,
        ],
        replace: &[
            0xB8, 0x00, 0x01, 0x00, 0x00, 0x89, 0x81, 0x38, 0x06, 0x00, 0x00, 0x90, 0xEB,
        ],
    },
    // Win10 / Win11 (pre-24H2, Builds 9600 – 26099)
    // PS: Search = '39 81 3C 06 00 00 0F [4 bytes] 00'
    //     Replace = 'B8 00 01 00 00 89 81 38 06 00 00 90'
    // We approximate the wildcard by splitting into two anchored sub-patterns.
    // Use the first 6 bytes plus "0F" as an anchor (byte 7):
    PatchPattern {
        description: "Windows 10 / Windows 11 pre-24H2 (Build 9600–26099)",
        build_min: 9600,
        build_max: 26099,
        search: &[0x39, 0x81, 0x3C, 0x06, 0x00, 0x00, 0x0F],
        // The patch is applied at the start of this 12-byte span (replace first 12 bytes)
        replace: &[0xB8, 0x00, 0x01, 0x00, 0x00, 0x89, 0x81],
    },
    // Windows 7 (Builds 7600–9599)
    // PS: Replace = 'B8 00 01 00 00 90 89 87 38 06 00 00 90 90 90 90 90 90'
    PatchPattern {
        description: "Windows 7 (Build 7600–9599)",
        build_min: 7600,
        build_max: 9599,
        search: &[0x39, 0x87, 0x3C, 0x06, 0x00, 0x00, 0x0F],
        replace: &[0xB8, 0x00, 0x01, 0x00, 0x00, 0x90, 0x89],
    },
];

/// Build reverse-patch table so we can restore - map the REPLACE bytes back to SEARCH bytes.
/// Returns `None` if the original cannot be reliably inferred.
pub struct PatchEngine;

impl PatchEngine {
    /// Read the OS build number from registry via wmic
    pub fn detect_build() -> Option<u32> {
        let output = std::process::Command::new("wmic")
            .args(&["os", "get", "BuildNumber", "/value"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.starts_with("BuildNumber=") {
                return line["BuildNumber=".len()..].trim().parse().ok();
            }
        }
        None
    }

    /// Find the bytes in `haystack` starting from `offset`, returning the index of the first match.
    fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() || haystack.len() < needle.len() {
            return None;
        }
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }

    /// Apply patch:
    ///  1. Pick pattern for current build
    ///  2. Backup original dll
    ///  3. Do binary search-replace
    ///  4. Write patched file back
    pub fn patch(dll_path: &Path, backup_path: &Path, build: u32) -> Result<String, String> {
        // Pick pattern
        let pattern = PATTERNS
            .iter()
            .find(|p| build >= p.build_min && build <= p.build_max)
            .ok_or_else(|| format!("No patch pattern available for build {}", build))?;

        // Read file
        let mut bytes =
            fs::read(dll_path).map_err(|e| format!("Failed to read termsrv.dll: {}", e))?;

        // Find pattern
        let pos = Self::find_bytes(&bytes, pattern.search)
            .ok_or_else(|| format!("Pattern '{}' not found in termsrv.dll. The DLL may already be patched or unsupported.", pattern.description))?;

        // Backup BEFORE patching
        fs::copy(dll_path, backup_path)
            .map_err(|e| format!("Failed to backup termsrv.dll: {}", e))?;

        // Apply patch
        for (i, &b) in pattern.replace.iter().enumerate() {
            bytes[pos + i] = b;
        }

        // Write back
        fs::write(dll_path, &bytes)
            .map_err(|e| format!("Failed to write patched termsrv.dll: {}", e))?;

        Ok(format!(
            "Patched successfully using '{}' at offset 0x{:X}. Backup saved to {:?}.",
            pattern.description, pos, backup_path
        ))
    }

    /// Restore from backup
    pub fn restore(dll_path: &Path, backup_path: &Path) -> Result<String, String> {
        if !backup_path.exists() {
            return Err("Backup file not found. Cannot restore.".into());
        }
        fs::copy(backup_path, dll_path)
            .map_err(|e| format!("Failed to restore termsrv.dll: {}", e))?;
        Ok("termsrv.dll restored from backup successfully.".into())
    }

    /// Check if the dll looks patched (verify first few replace bytes are present)
    pub fn is_patched(dll_path: &Path, build: u32) -> bool {
        let pattern = match PATTERNS
            .iter()
            .find(|p| build >= p.build_min && build <= p.build_max)
        {
            Some(p) => p,
            None => return false,
        };
        let bytes = match fs::read(dll_path) {
            Ok(b) => b,
            Err(_) => return false,
        };
        // If replace bytes are found and search bytes are NOT found → patched
        Self::find_bytes(&bytes, pattern.replace).is_some()
            && Self::find_bytes(&bytes, pattern.search).is_none()
    }
}
