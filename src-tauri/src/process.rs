use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::System;

// 只杀 IDE 主进程，不杀 CLI 工具
const QODER_PROCESS_NAMES: &[&str] = &[
    "qoderwork cn.exe",
    "qoder cn.exe",
];

/// 杀死所有 Qoder IDE 进程（两遍法确保干净）
pub fn kill_qoder_ide() -> Result<u32, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let killed = kill_matching_processes(&sys);
    log::info!("[process] 第一轮关闭 {} 个 IDE 进程", killed);

    if killed > 0 {
        // 等待进程完全退出
        thread::sleep(Duration::from_secs(2));

        // 第二轮确保完全关闭
        sys.refresh_all();
        let killed2 = kill_matching_processes(&sys);
        if killed2 > 0 {
            log::info!("[process] 第二轮关闭 {} 个残余进程", killed2);
            thread::sleep(Duration::from_millis(500));
        }
    }

    Ok(killed as u32)
}

/// 启动 Qoder IDE
pub fn launch_qoder_ide() -> Result<(), String> {
    let exe = find_qoder_exe()?;
    Command::new(&exe)
        .spawn()
        .map_err(|e| format!("启动 IDE 失败: {}", e))?;
    log::info!("[process] 已启动 IDE: {:?}", exe);
    Ok(())
}

/// 检查 IDE 是否在运行
pub fn is_qoder_ide_running() -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();

    let current_pid = sysinfo::get_current_pid().ok();

    for (pid, process) in sys.processes() {
        if Some(*pid) == current_pid {
            continue;
        }
        let name = process.name().to_string_lossy().to_lowercase();
        if QODER_PROCESS_NAMES.iter().any(|p| name == *p) {
            return true;
        }
    }
    false
}

/// 杀死匹配的进程
fn kill_matching_processes(sys: &System) -> usize {
    let current_pid = sysinfo::get_current_pid().ok();
    let mut count = 0;

    for (pid, process) in sys.processes() {
        // 排除自身
        if Some(*pid) == current_pid {
            continue;
        }
        let name = process.name().to_string_lossy().to_lowercase();
        if QODER_PROCESS_NAMES.iter().any(|p| name == *p) {
            if process.kill() {
                count += 1;
                log::info!("[process] 已终止进程: {} (PID: {})", name, pid);
            } else {
                log::warn!("[process] 终止进程失败: {} (PID: {})", name, pid);
            }
        }
    }
    count
}

/// 发现 Qoder IDE 可执行文件路径
fn find_qoder_exe() -> Result<PathBuf, String> {
    // 1. 检查 %LOCALAPPDATA%\QoderWork CN\QoderWork CN.exe
    if let Some(local_app) = std::env::var_os("LOCALAPPDATA") {
        let path = PathBuf::from(&local_app).join("QoderWork CN").join("QoderWork CN.exe");
        if path.exists() {
            return Ok(path);
        }
        // 2. 检查 %LOCALAPPDATA%\Programs\QoderWork CN\QoderWork CN.exe
        let path = PathBuf::from(&local_app)
            .join("Programs")
            .join("QoderWork CN")
            .join("QoderWork CN.exe");
        if path.exists() {
            return Ok(path);
        }
    }

    // 3. 检查 Program Files
    if let Some(pf) = std::env::var_os("ProgramFiles") {
        let path = PathBuf::from(pf).join("QoderWork CN").join("QoderWork CN.exe");
        if path.exists() {
            return Ok(path);
        }
    }

    // 4. 检查 %APPDATA%\..\Local\QoderWork CN
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let local = PathBuf::from(appdata)
            .parent()
            .map(|p| p.join("Local").join("QoderWork CN").join("QoderWork CN.exe"));
        if let Some(path) = local {
            if path.exists() {
                return Ok(path);
            }
        }
    }

    // 5. 尝试通过注册表查找
    #[cfg(target_os = "windows")]
    {
        if let Ok(path) = find_from_registry() {
            return Ok(path);
        }
    }

    Err("无法找到 Qoder IDE 可执行文件，请确认已安装".to_string())
}

#[cfg(target_os = "windows")]
fn find_from_registry() -> Result<PathBuf, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    for root in [&hklm, &hkcu] {
        if let Ok(uninstall) =
            root.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
        {
            for key_name in uninstall.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(subkey) = uninstall.open_subkey(&key_name) {
                    let display_name: Result<String, _> = subkey.get_value("DisplayName");
                    if let Ok(name) = display_name {
                        if name.to_lowercase().contains("qoder") {
                            let install_location: Result<String, _> =
                                subkey.get_value("InstallLocation");
                            if let Ok(loc) = install_location {
                                let exe =
                                    PathBuf::from(&loc).join(format!("{}.exe", name));
                                if exe.exists() {
                                    return Ok(exe);
                                }
                                // 尝试直接在目录中找 .exe
                                if let Ok(entries) = std::fs::read_dir(&loc) {
                                    for entry in entries.flatten() {
                                        let p = entry.path();
                                        if p.extension().map_or(false, |e| e == "exe") {
                                            let fname = p
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy()
                                                .to_lowercase();
                                            if fname.contains("qoder") {
                                                return Ok(p);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err("注册表中未找到 Qoder IDE".to_string())
}
