#![allow(unsafe_op_in_unsafe_fn)]
use std::{ffi::c_void, thread, time::Duration};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Console::AllocConsole;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

unsafe extern "system" fn mouse_proc(c: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if c >= 0 {
        let m = &*(l as *const MSLLHOOKSTRUCT);
        if m.dwExtraInfo == 0xDEADC0DE {
            eprintln!("lamev2 magic ExtraInfo: 0xDEADC0DE!");
        } else if (m.flags & 1) != 0 {
            eprintln!("Injected mouse movement!");
        }
    }
    CallNextHookEx(std::ptr::null_mut(), c, w, l)
}

unsafe extern "system" fn kbd_proc(c: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if c >= 0 && (*(l as *const KBDLLHOOKSTRUCT)).flags & 1 != 0 {
        eprintln!("Injected key event (Relax/Tap Assist)!");
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
                eprintln!("Found lame overlay window (class '  ')!");
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
