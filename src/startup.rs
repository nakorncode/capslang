use std::os::windows::process::CommandExt;
use std::process::Command;
use std::{fs, path::PathBuf};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const TASK_NAME: &str = r"NakornCode\CapsLang";

pub fn remove_legacy_startup_shortcut() {
    if let Some(startup) = startup_folder() {
        let shortcut = startup.join("CapsLang.lnk");
        let _ = fs::remove_file(shortcut);
    }
}

pub fn is_startup_enabled() -> bool {
    match query_task_xml() {
        Some(xml) => xml.contains("<Enabled>true</Enabled>") || xml.contains("<Enabled>1</Enabled>"),
        None => false,
    }
}

pub fn ensure_registered(enable_at_logon: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let tr = format!("\"{}\"", exe.display());

    let status = Command::new("schtasks")
        .args([
            "/Create",
            "/F",
            "/TN",
            TASK_NAME,
            "/SC",
            "ONLOGON",
            "/RL",
            "HIGHEST",
            "/IT",
            "/TR",
            &tr,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err(format!("schtasks /Create failed with {status}"));
    }

    set_enabled(enable_at_logon)
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    if !task_exists() {
        return if enabled {
            ensure_registered(true)
        } else {
            Ok(())
        };
    }

    let flag = if enabled { "/ENABLE" } else { "/DISABLE" };
    let status = Command::new("schtasks")
        .args(["/Change", "/TN", TASK_NAME, flag])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("schtasks /Change failed with {status}"))
    }
}

pub fn run_task() -> Result<(), String> {
    let status = Command::new("schtasks")
        .args(["/Run", "/TN", TASK_NAME])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("schtasks /Run failed with {status}"))
    }
}

pub fn task_exists() -> bool {
    query_task_xml().is_some()
}

fn query_task_xml() -> Option<String> {
    let output = Command::new("schtasks")
        .args(["/Query", "/TN", TASK_NAME, "/XML"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout).ok()
}

fn startup_folder() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA").map(PathBuf::from)?;
    Some(
        appdata
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup"),
    )
}
