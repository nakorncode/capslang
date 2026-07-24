#![windows_subsystem = "windows"]

mod elevate;
mod hook;
mod settings;
mod startup;
mod tray_app;

use crate::elevate::{hand_off_to_elevated_instance, is_elevated, take_single_instance};
use crate::settings::Settings;
use crate::startup::{ensure_registered, is_startup_enabled, remove_legacy_startup_shortcut};
use crate::tray_app::run_tray;
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

fn main() {
    if !is_elevated() {
        hand_off_to_elevated_instance();
        return;
    }

    let _mutex = match take_single_instance() {
        Some(mutex) => mutex,
        None => return,
    };

    let first_run = !Settings::exists();
    let settings = Settings::load();

    remove_legacy_startup_shortcut();
    let startup_on = first_run || is_startup_enabled();
    let _ = ensure_registered(startup_on);

    if first_run {
        let _ = settings.save();
    }

    hook::install(settings.enabled);
    hook::force_caps_lock_off();

    if let Err(error) = run_tray(settings.enabled, startup_on) {
        show_message("CapsLang", &format!("Failed to start tray icon:\n{error}"));
    }
}

fn show_message(title: &str, body: &str) {
    let title = wide(title);
    let body = wide(body);
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(body.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) fn help_text() -> &'static str {
    "CapsLang is a tiny tray tool that remaps CapsLock to switch the\r\n\
     Windows input language.\r\n\r\n\
     CapsLock — next input language\r\n\
     Alt+CapsLock — real CapsLock toggle\r\n\r\n\
     CapsLang runs elevated by default so the remap also works in\r\n\
     administrator windows. The first launch asks for UAC once, then a\r\n\
     logon scheduled task starts CapsLang quietly afterward.\r\n\r\n\
     Tray menu: Enabled, Launch on startup, Help, Exit.\r\n\r\n\
     Turn off any PowerToys CapsLock remap while CapsLang is running.\r\n\r\n\
     Credit by nakorncode\r\n\
     https://github.com/nakorncode/capslang"
}
