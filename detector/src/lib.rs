#![allow(unsafe_op_in_unsafe_fn)]
use std::{ffi::c_void, thread, time::Duration};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Console::AllocConsole;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const LLKHF_INJECTED: u32 = 0x00000010;

unsafe extern "system" fn mouse_proc(c: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if c >= 0 {
        let m = &*(l as *const MSLLHOOKSTRUCT);
        // Only flag 0xDEADC0DE to avoid false-flagging OpenTabletDriver (which uses SendInput)
        if m.dwExtraInfo == 0xDEADC0DE {
            eprintln!("[DETECTION] lamev2 magic ExtraInfo: 0xDEADC0DE!");
        }
    }
    CallNextHookEx(std::ptr::null_mut(), c, w, l)
}

unsafe extern "system" fn kbd_proc(c: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if c >= 0 {
        let kb = &*(l as *const KBDLLHOOKSTRUCT);
        // Bit 4 (0x10) is LLKHF_INJECTED. Bit 0 (0x01) is LLKHF_EXTENDED (e.g. WinKey, arrows).
        if (kb.flags & LLKHF_INJECTED) != 0 {
            eprintln!("[DETECTION] Injected key event (Relax/Tap Assist): VK=0x{:02X}!", kb.vkCode);
        }
    }
    CallNextHookEx(std::ptr::null_mut(), c, w, l)
}

fn watch() {
    unsafe {
        AllocConsole();
        SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), std::ptr::null_mut(), 0);
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(kbd_proc), std::ptr::null_mut(), 0);

        thread::spawn(|| loop {
            if !FindWindowA(b"  \0".as_ptr(), std::ptr::null()).is_null() {
                eprintln!("[DETECTION] Found lame overlay window (class '  ')!");
            }
            thread::sleep(Duration::from_secs(1));
        });

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {}
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(_: *mut c_void, r: u32, _: *mut c_void) -> BOOL {
    if r == 1 { thread::spawn(watch); }
    1
}
