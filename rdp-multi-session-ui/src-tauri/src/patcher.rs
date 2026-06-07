use std::fs;
use std::path::Path;

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
    // PS: Search = '39 81 3C 06 00 00 0F [4 bytes] 00'  (14 bytes total)
    //     Replace = 'B8 00 01 00 00 89 81 38 06 00 00 90' (12 bytes)
    //
    // The PS1 uses regex wildcards; here we match the 8-byte anchor:
    //   cmp [rcx+638h], eax (39 81 3C 06 00 00) + jnz near (0F 85)
    // Replace with 8 bytes:
    //   mov eax, 100h (B8 00 01 00 00) + mov [rcx+638h], eax prefix (89 81 90)
    PatchPattern {
        description: "Windows 10 / Windows 11 pre-24H2 (Build 9600–26099)",
        build_min: 9600,
        build_max: 26099,
        search: &[0x39, 0x81, 0x3C, 0x06, 0x00, 0x00, 0x0F, 0x85],
        replace: &[0xB8, 0x00, 0x01, 0x00, 0x00, 0x89, 0x81, 0x90],
    },
    // Windows 7 (Builds 7600–9599)
    // PS: search uses 0x87 register encoding, replace has NOPs
    PatchPattern {
        description: "Windows 7 (Build 7600–9599)",
        build_min: 7600,
        build_max: 9599,
        search: &[0x39, 0x87, 0x3C, 0x06, 0x00, 0x00, 0x0F, 0x85],
        replace: &[0xB8, 0x00, 0x01, 0x00, 0x00, 0x90, 0x89, 0x90],
    },
];

pub struct PatchEngine;

impl PatchEngine {
    /// Read the OS build number from cmd ver command
    pub fn detect_build() -> Option<u32> {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let output = std::process::Command::new("cmd")
                .args(&["/c", "ver"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .ok()?;
            let text = String::from_utf8_lossy(&output.stdout);

            // Expected format: Microsoft Windows [Version 10.0.19045.3803]
            // or Windows 7: Microsoft Windows [Version 6.1.7601]
            if let Some(start) = text.find("[Version ") {
                let ver_str = &text[start + 9..];
                if let Some(end) = ver_str.find(']') {
                    let current_ver = &ver_str[..end];
                    let parts: Vec<&str> = current_ver.split('.').collect();
                    if parts.len() >= 3 {
                        return parts[2].trim().parse().ok();
                    }
                }
            }
            None
        }
        #[cfg(not(target_os = "windows"))]
        {
            Some(26100) // Dummy for testing
        }
    }

    /// Find the bytes in `haystack`, returning the index of the first match.
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
    ///  3. Do binary search-replace (search.len must equal replace.len)
    ///  4. Write patched file back
    pub fn patch(dll_path: &Path, backup_path: &Path, build: u32) -> Result<String, String> {
        let pattern = PATTERNS
            .iter()
            .find(|p| build >= p.build_min && build <= p.build_max)
            .ok_or_else(|| format!("No patch pattern available for build {}", build))?;

        if pattern.search.len() != pattern.replace.len() {
            return Err(format!(
                "Internal error: search/replace length mismatch for '{}'",
                pattern.description
            ));
        }

        let mut bytes =
            fs::read(dll_path).map_err(|e| format!("Failed to read termsrv.dll: {}", e))?;

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

    /// Check if the dll looks patched.
    /// Verifies that the replace bytes are present AND the search bytes are absent,
    /// which avoids false positives from coincidental byte matches.
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
        Self::find_bytes(&bytes, pattern.replace).is_some()
            && Self::find_bytes(&bytes, pattern.search).is_none()
    }
}
