//! Per-user installer for a standalone Windows build, driven by the binary
//! itself: `RePikaChat.exe --install`.
//!
//! No admin rights, no registry writes, no bundled runtime. It copies the
//! executable into `%LOCALAPPDATA%\Programs\RePikaChat`, adds a Start Menu
//! shortcut and leaves an uninstaller beside it.

use std::path::PathBuf;
use std::process::Command;

pub fn install_dir() -> Result<PathBuf, String> {
    let local = std::env::var("LOCALAPPDATA")
        .map_err(|_| "LOCALAPPDATA is not set, so the per-user install path is unknown.".to_string())?;
    if local.trim().is_empty() {
        return Err("LOCALAPPDATA is empty.".to_string());
    }
    Ok(PathBuf::from(local).join("Programs").join("RePikaChat"))
}

fn start_menu_dir() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA")
        .map_err(|_| "APPDATA is not set, so the Start Menu path is unknown.".to_string())?;
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs"))
}

fn ps_quote(path: &str) -> String {
    path.replace('\'', "''")
}

#[cfg(windows)]
fn write_shortcut(lnk: &PathBuf, target: &PathBuf, dir: &PathBuf) -> Result<(), String> {
    let script = format!(
        "$w = New-Object -ComObject WScript.Shell; \
         $s = $w.CreateShortcut('{lnk}'); \
         $s.TargetPath = '{target}'; \
         $s.WorkingDirectory = '{dir}'; \
         $s.Description = 'RePikaChat — local AI companion'; \
         $s.IconLocation = '{target},0'; \
         $s.Save()",
        lnk = ps_quote(&lnk.display().to_string()),
        target = ps_quote(&target.display().to_string()),
        dir = ps_quote(&dir.display().to_string()),
    );
    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .status()
        .map_err(|e| format!("could not run PowerShell to create the shortcut ({e})"))?;
    if status.success() {
        Ok(())
    } else {
        Err("PowerShell could not create the Start Menu shortcut.".to_string())
    }
}

#[cfg(not(windows))]
fn write_shortcut(_lnk: &PathBuf, _target: &PathBuf, _dir: &PathBuf) -> Result<(), String> {
    Err("Shortcuts are only created on Windows. Run the binary directly on this platform.".to_string())
}

pub fn install() -> Result<Vec<String>, String> {
    if !cfg!(windows) {
        return Err(
            "The self-installer targets Windows. On other systems, copy the binary somewhere on your PATH."
                .to_string(),
        );
    }

    let exe = std::env::current_exe()
        .map_err(|e| format!("cannot locate the running executable ({e})"))?;
    let dir = install_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("cannot create {} ({e})", dir.display()))?;

    let target = dir.join("RePikaChat.exe");
    if exe != target {
        std::fs::copy(&exe, &target).map_err(|e| {
            format!(
                "cannot copy {} to {} ({e})",
                exe.display(),
                target.display()
            )
        })?;
    }

    // Small uninstaller that survives deleting the folder it lives in.
    let uninstall = dir.join("Uninstall.cmd");
    let _ = std::fs::write(
        &uninstall,
        format!(
            "@echo off\r\n\
             echo Removing RePikaChat...\r\n\
             \"%~dp0RePikaChat.exe\" --uninstall\r\n"
        ),
    );

    let mut lines = Vec::new();
    let menu = start_menu_dir()?;
    std::fs::create_dir_all(&menu).ok();
    let lnk = menu.join("RePikaChat.lnk");
    match write_shortcut(&lnk, &target, &dir) {
        Ok(()) => lines.push(format!("Start Menu shortcut: {}", lnk.display())),
        Err(e) => lines.push(format!("Start Menu shortcut skipped: {e}")),
    }

    lines.insert(0, format!("Installed to: {}", target.display()));
    lines.push(format!("Uninstall with: \"{}\" --uninstall", target.display()));
    lines.push("Your memories are kept on uninstall; add --purge to erase them.".to_string());
    Ok(lines)
}

pub fn uninstall(purge: bool) -> Result<Vec<String>, String> {
    if !cfg!(windows) {
        return Err("The self-uninstaller targets Windows.".to_string());
    }
    let dir = install_dir()?;
    let mut lines = Vec::new();

    if let Ok(menu) = start_menu_dir() {
        let lnk = menu.join("RePikaChat.lnk");
        if lnk.exists() && std::fs::remove_file(&lnk).is_ok() {
            lines.push(format!("Removed shortcut: {}", lnk.display()));
        }
    }

    if purge {
        let data = crate::store::data_dir();
        if data.exists() {
            std::fs::remove_dir_all(&data)
                .map_err(|e| format!("cannot erase {} ({e})", data.display()))?;
            lines.push(format!("Erased memories in {}", data.display()));
        }
    } else {
        lines.push(format!(
            "Kept memories in {} (run with --purge to erase).",
            crate::store::data_dir().display()
        ));
    }

    // A running executable cannot delete itself, so hand the folder to a short
    // detached script that waits for this process to exit.
    let script_path = std::env::temp_dir().join("repikachat-uninstall.cmd");
    let body = format!(
        "@echo off\r\ntimeout /t 2 /nobreak >nul\r\nrmdir /s /q \"{}\"\r\ndel /q \"%~f0\"\r\n",
        dir.display()
    );
    std::fs::write(&script_path, body)
        .map_err(|e| format!("cannot write the cleanup script ({e})"))?;
    Command::new("cmd")
        .args(["/C", "start", "/min", "cmd", "/C"])
        .arg(&script_path)
        .spawn()
        .map_err(|e| format!("cannot schedule folder cleanup ({e})"))?;

    lines.push(format!("Removing {}", dir.display()));
    Ok(lines)
}
