//! Hardware detection module.
//!
//! Provides system information including CPU, GPU, and memory.

use crate::results::SystemInfo;
use sysinfo::System;

/// Get comprehensive system information.
pub fn get_system_info() -> SystemInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Get CPU info
    let cpu = sys.cpus().first();
    let cpu_brand = cpu
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_count_logical = sys.cpus().len();
    let cpu_count_physical = sys.physical_core_count();

    // Get memory info
    let memory_total = sys.total_memory();
    let memory_total_gb = memory_total as f64 / (1024.0 * 1024.0 * 1024.0);

    // Get platform info
    let platform = std::env::consts::OS.to_string();
    let platform_release = get_platform_release();

    SystemInfo {
        platform,
        platform_release,
        cpu: Some(cpu_brand),
        cpu_count_logical,
        cpu_count_physical,
        memory_total_gb,
        gpu: get_gpu_info(),
        storage: None,
    }
}

/// Get the platform release/version.
fn get_platform_release() -> String {
    System::long_os_version().unwrap_or_else(|| "Unknown".to_string())
}

/// Get GPU information.
fn get_gpu_info() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        return get_windows_gpu();
    }

    #[cfg(target_os = "linux")]
    {
        return get_linux_gpu();
    }

    #[cfg(target_os = "macos")]
    {
        return get_macos_gpu();
    }

    #[allow(unreachable_code)]
    None
}

// ---------------------------------------------------------------------------
// Windows GPU detection
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn get_windows_gpu() -> Option<String> {
    // Strategy 1: PowerShell — available on Windows 10+ by default
    // Try Get-CimInstance first (wmic is deprecated since Windows 10 21H1)
    if let Some(gpu) = get_windows_gpu_powershell() {
        return Some(gpu);
    }

    // Strategy 2: wmic fallback — still present on most installs,
    // may be absent on fresh Windows 11 installs
    if let Some(gpu) = get_windows_gpu_wmic() {
        return Some(gpu);
    }

    None
}

#[cfg(target_os = "windows")]
fn get_windows_gpu_powershell() -> Option<String> {
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name | Select-Object -First 1",
        ])
        .output()
        .ok()
        .and_then(|output| {
            if !output.status.success() {
                return None;
            }
            let stdout = String::from_utf8(output.stdout).ok()?;
            let name = stdout.trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
}

#[cfg(target_os = "windows")]
fn get_windows_gpu_wmic() -> Option<String> {
    std::process::Command::new("wmic")
        .args(["path", "win32_VideoController", "get", "name"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| {
            output
                .lines()
                .map(|l| l.trim())
                .find(|line| !line.is_empty() && !line.starts_with("Name"))
                .map(|s| s.to_string())
        })
}

// ---------------------------------------------------------------------------
// Linux GPU detection
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn get_linux_gpu() -> Option<String> {
    // Strategy 1: lspci -nn (machine-parseable, available on most distros)
    if let Some(gpu) = get_linux_gpu_lspci() {
        return Some(gpu);
    }

    // Strategy 2: /sys/class/drm fallback
    if let Some(gpu) = get_linux_gpu_sysfs() {
        return Some(gpu);
    }

    None
}

#[cfg(target_os = "linux")]
fn get_linux_gpu_lspci() -> Option<String> {
    // lspci -nn gives concise output like:
    // 01:00.0 VGA compatible controller [0300]: NVIDIA Corporation ... [10de:xxxx] (rev a1)
    std::process::Command::new("lspci")
        .arg("-nn")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| {
            output
                .lines()
                .find(|line| {
                    let lower = line.to_lowercase();
                    lower.contains("vga compatible controller")
                        || lower.contains("3d controller")
                        || lower.contains("display controller")
                })
                .map(|line| {
                    // Format: "01:00.0 VGA compatible controller [0300]: NVIDIA Corporation ... [10de:xxxx]"
                    // Extract everything between the first colon and the PCI ID brackets
                    let after_first_colon = line.split(':').skip(1).collect::<Vec<_>>().join(":");
                    let after_first_colon = after_first_colon.trim();
                    // Strip trailing PCI ID like "[10de:2684]" and revision "(rev a1)"
                    let cleaned = after_first_colon
                        .split('[')
                        .next()
                        .unwrap_or(after_first_colon)
                        .trim();
                    // Remove trailing "(rev ...)" if present
                    let cleaned = if let Some(pos) = cleaned.rfind("(rev") {
                        cleaned[..pos].trim()
                    } else {
                        cleaned
                    };
                    cleaned.to_string()
                })
        })
}

#[cfg(target_os = "linux")]
fn get_linux_gpu_sysfs() -> Option<String> {
    // Read GPU name from /sys/class/drm/card0/device/gpu_name or similar
    std::fs::read_to_string("/sys/class/drm/card0/device/gpu_name")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// macOS GPU detection
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn get_macos_gpu() -> Option<String> {
    // Strategy 1: system_profiler (comprehensive but slow ~200ms)
    if let Some(gpu) = get_macos_gpu_system_profiler() {
        return Some(gpu);
    }

    // Strategy 2: ioreg (faster ~50ms)
    if let Some(gpu) = get_macos_gpu_ioreg() {
        return Some(gpu);
    }

    None
}

#[cfg(target_os = "macos")]
fn get_macos_gpu_system_profiler() -> Option<String> {
    std::process::Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-detailLevel", "mini"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| {
            output
                .lines()
                .find(|line| line.contains("Chipset Model:"))
                .map(|line| {
                    line.split(':')
                        .nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default()
                })
        })
}

#[cfg(target_os = "macos")]
fn get_macos_gpu_ioreg() -> Option<String> {
    std::process::Command::new("ioreg")
        .args(["-l", "-n", "AGXAccelerator", "-r"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| {
            output
                .lines()
                .find(|line| line.contains("\"model\""))
                .and_then(|line| {
                    line.split('=')
                        .nth(1)
                        .map(|s| s.trim().trim_matches('"').to_string())
                })
                .filter(|s| !s.is_empty())
        })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_info() {
        let info = get_system_info();
        assert!(!info.platform.is_empty());
        assert!(info.cpu_count_logical > 0);
        assert!(info.memory_total_gb > 0.0);
    }

    #[test]
    fn test_system_info_cpu() {
        let info = get_system_info();
        assert!(info.cpu.is_some());
        assert!(!info.cpu.as_ref().unwrap().is_empty());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_gpu_detection() {
        // At least one of the two strategies should work on a Windows machine with a GPU
        let gpu = get_windows_gpu();
        // We can't assert it's Some in CI (no GPU), but we can verify no panic
        let _ = gpu;
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_powershell_command_runs() {
        // Verify PowerShell command doesn't crash even if no GPU found
        let result = get_windows_gpu_powershell();
        let _ = result;
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_gpu_detection() {
        let gpu = get_linux_gpu();
        // On CI (no GPU) this will be None, on dev machines with GPU it should be Some
        let _ = gpu;
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_gpu_detection() {
        let gpu = get_macos_gpu();
        let _ = gpu;
    }
}
