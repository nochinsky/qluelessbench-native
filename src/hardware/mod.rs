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
    // This is a simplified implementation
    // A full implementation would use platform-specific APIs

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

#[cfg(target_os = "windows")]
fn get_windows_gpu() -> Option<String> {
    // Try wmic command (available on all Windows versions)
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

#[cfg(target_os = "linux")]
fn get_linux_gpu() -> Option<String> {
    // Try to get GPU from lspci
    std::process::Command::new("lspci")
        .arg("-v")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| {
            output
                .lines()
                .find(|line| {
                    line.to_lowercase().contains("vga") || line.to_lowercase().contains("3d")
                })
                .map(|line| {
                    // Extract GPU name after the colon
                    line.split(':')
                        .nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default()
                })
        })
}

#[cfg(target_os = "macos")]
fn get_macos_gpu() -> Option<String> {
    // Use system_profiler
    std::process::Command::new("system_profiler")
        .arg("SPDisplaysDataType")
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
}
