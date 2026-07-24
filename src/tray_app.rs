use crate::help_text;
use crate::hook;
use crate::settings::Settings;
use crate::startup::{ensure_registered, set_enabled as set_startup_enabled};
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MessageBoxW, TranslateMessage, MB_ICONINFORMATION, MB_OK, MSG,
    WM_QUIT,
};

const ID_ENABLED: &str = "enabled";
const ID_STARTUP: &str = "startup";
const ID_HELP: &str = "help";
const ID_EXIT: &str = "exit";

pub fn run_tray(enabled_on: bool, startup_on: bool) -> Result<(), Box<dyn std::error::Error>> {
    let icon = load_icon()?;

    let enabled_item = CheckMenuItem::with_id(ID_ENABLED, "Enabled", true, enabled_on, None);
    let startup_item =
        CheckMenuItem::with_id(ID_STARTUP, "Launch on startup", true, startup_on, None);
    let help_item = MenuItem::with_id(ID_HELP, "Help", true, None);
    let exit_item = MenuItem::with_id(ID_EXIT, "Exit", true, None);

    let menu = Menu::new();
    menu.append(&enabled_item)?;
    menu.append(&startup_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&help_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&exit_item)?;

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("CapsLang: CapsLock switches input language")
        .with_icon(icon)
        .build()?;

    let running = AtomicBool::new(true);
    let menu_channel = MenuEvent::receiver();

    unsafe {
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            while let Ok(event) = menu_channel.try_recv() {
                match event.id.as_ref() {
                    ID_ENABLED => {
                        let checked = enabled_item.is_checked();
                        hook::set_enabled(checked);
                        let mut settings = Settings::load();
                        settings.enabled = checked;
                        let _ = settings.save();
                    }
                    ID_STARTUP => {
                        let checked = startup_item.is_checked();
                        if checked {
                            let _ = ensure_registered(true);
                        } else {
                            let _ = set_startup_enabled(false);
                        }
                    }
                    ID_HELP => show_help(),
                    ID_EXIT => {
                        running.store(false, Ordering::Relaxed);
                        windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                    }
                    _ => {}
                }
            }

            if message.message == WM_QUIT {
                break;
            }

            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);

            if !running.load(Ordering::Relaxed) {
                break;
            }
        }
    }

    Ok(())
}

fn load_icon() -> Result<Icon, Box<dyn std::error::Error>> {
    let bytes = include_bytes!("../assets/tray-icon.png");
    let image = image::load_from_memory(bytes)?.into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).map_err(|e| e.into())
}

fn show_help() {
    let title = wide("CapsLang Help");
    let body = wide(help_text());
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
