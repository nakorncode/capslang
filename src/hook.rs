use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, GetAsyncKeyState, GetKeyState, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_CAPITAL,
    VK_MENU,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, PostMessageW, SetWindowsHookExW, HC_ACTION,
    KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_INPUTLANGCHANGEREQUEST, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP,
};

const HKL_NEXT: isize = 1;
const INPUTLANGCHANGE_FORWARD: usize = 2;

static ENABLED: AtomicBool = AtomicBool::new(true);

pub fn install(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
    unsafe {
        let _ = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0);
    }
}

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn force_caps_lock_off() {
    if is_caps_lock_on() {
        toggle_caps_lock();
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code != HC_ACTION as i32 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    if !ENABLED.load(Ordering::Relaxed) {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let event = *(lparam.0 as *const KBDLLHOOKSTRUCT);
    if event.vkCode != VK_CAPITAL.0 as u32 {
        return CallNextHookEx(None, code, wparam, lparam);
    }

    let message = wparam.0 as u32;
    if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
        if key_down(VK_MENU.0 as i32) {
            toggle_caps_lock();
        } else {
            force_caps_lock_off();
            switch_input_language();
        }
    }

    if matches!(
        message,
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP
    ) {
        return LRESULT(1);
    }

    CallNextHookEx(None, code, wparam, lparam)
}

fn key_down(vkey: i32) -> bool {
    unsafe { GetAsyncKeyState(vkey) < 0 }
}

fn is_caps_lock_on() -> bool {
    unsafe { GetKeyState(VK_CAPITAL.0 as i32) & 1 != 0 }
}

fn toggle_caps_lock() {
    unsafe {
        keybd_event(VK_CAPITAL.0 as u8, 0x45, KEYEVENTF_EXTENDEDKEY, 0);
        keybd_event(
            VK_CAPITAL.0 as u8,
            0x45,
            KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
            0,
        );
    }
}

fn switch_input_language() {
    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_invalid() {
        return;
    }

    let _ = unsafe {
        PostMessageW(
            Some(foreground),
            WM_INPUTLANGCHANGEREQUEST,
            WPARAM(INPUTLANGCHANGE_FORWARD),
            LPARAM(HKL_NEXT),
        )
    };
}
