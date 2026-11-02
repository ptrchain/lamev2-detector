# Detection Vectors for lamev2

This document explains how to detect the `lamev2` cheat (found in `shitcheat/`).

The cheat works externally:
1. It reads game memory using `NtOpenProcess` and `NtReadVirtualMemory`.
2. It draws an ImGui menu using a transparent DirectX 11 overlay window.
3. It moves the mouse and presses keys using `SendInput` and `NtUserInjectKeyboardInput`.
4. It uses math formulas to smooth mouse movement and schedule key presses.

Here are the practical ways to detect it.

---

## 1. Input Detections (Fastest and Most Reliable)

### 1.1 Check for `0xDEADC0DE` in Mouse Events
- **File:** `shitcheat/impl/memory/input.hxx` (lines 140, 160)
- **What happens:** Every time the cheat moves the mouse, it sets `dwExtraInfo = 0xDEADC0DE`.
- **How to detect:** In a mouse hook (`WH_MOUSE_LL`) or window message handler, inspect the `dwExtraInfo` field of incoming mouse messages. If it equals `0xDEADC0DE`, it is this cheat.

### 1.2 Check the Injected Flag
- **File:** `shitcheat/impl/input/keyboard_hook.hxx` and `shitcheat/core/relax/nt_input.hxx`
- **What happens:** When the cheat injects mouse or keyboard events through Windows APIs, Windows sets the injected bit:
  - Mouse: `MSLLHOOKSTRUCT.flags & LLMHF_INJECTED` (0x01)
  - Keyboard: `KBDLLHOOKSTRUCT.flags & LLKHF_INJECTED` (0x01)
- **How to detect:** Set a low-level hook (`WH_MOUSE_LL` or `WH_KEYBOARD_LL`). If inputs during gameplay have the injected bit set, flag them.

### 1.3 Compare Raw Input (`WM_INPUT`) with Cursor Movement
- **What happens:** Real mice and tablets send raw hardware packets through Windows Raw Input (`WM_INPUT`). Injected mouse moves (`SendInput`) do not send valid hardware packets.
- **How to detect:** Register the game window for Raw Input (`RegisterRawInputDevices`). If the cursor moves on screen but there are no matching `WM_INPUT` packets (or `hDevice` is null), the input was simulated.

---

## 2. Process and Memory Detections

### 2.1 Open Process Handles
- **File:** `shitcheat/impl/memory/process.hxx`
- **What happens:** The cheat opens a handle to `osu!.exe` with `PROCESS_VM_READ | PROCESS_QUERY_INFORMATION`.
- **How to detect:** Query system handles using `NtQuerySystemInformation(SystemHandleInformation)` and find processes that have open read handles to the osu! process.

### 2.2 RWX Memory for Syscall Stubs
- **File:** `shitcheat/impl/memory/syscall.hxx`
- **What happens:** The cheat allocates 32 bytes of `PAGE_EXECUTE_READWRITE` (RWX) memory using `VirtualAlloc` to run direct syscalls. The stub contains:
  ```
  4C 8B D1          mov r10, rcx
  B8 ?? ?? ?? ??    mov eax, <syscall_number>
  0F 05             syscall
  C3                ret
  ```
- **How to detect:** Scan external processes for memory pages marked as RWX, or search for that specific byte pattern.

### 2.3 Memory Scan Patterns
- **Files:** `shitcheat/impl/defs/offsets_stable.hxx` and `shitcheat/impl/defs/offsets_lazer.hxx`
- **What happens:** The cheat scans game memory for these fixed byte patterns:
  - Stable time: `5E 5F 5D C3 A1 ? ? ? ? 89 ? 04`
  - Stable state: `48 83 F8 04 73 1E`
  - Stable beatmap: `F8 01 74 04 83 65`
  - Lazer anchor: `00 00 80 44 00 00 40 44 00 00 00 00 ? ? ? ? 00 00 00 00`
- **How to detect:** Create decoy memory pages with these byte patterns and protect them with `PAGE_GUARD` or `PAGE_NOACCESS` to catch the cheat reading them.

---

## 3. Window Detections

### 3.1 Overlay Window Check
- **File:** `shitcheat/impl/ui/overlay.cxx`
- **What happens:** The cheat creates an overlay window with:
  - Class name: `"  "` (two spaces)
  - Window title: `"  "` (two spaces)
  - Width and height: 820 x 620
  - Window style: `WS_POPUP`, `WS_EX_TOPMOST`, `WS_EX_LAYERED`
- **How to detect:** Call `FindWindowA("  ", NULL)`. If a window exists with that class name and size, the cheat overlay is running.

---

## 4. File System Artifacts

### 4.1 Config Folder
- **File:** `shitcheat/impl/config/config_store.hxx`
- **What happens:** The cheat creates and reads config files at:
  ```
  %APPDATA%\lame\configs\*.cfg
  ```
- **How to detect:** Check if that folder exists or look for keys like `relax.ur`, `aim.strength`, or `autobot.aim_spread` in config files.

---

## 5. Replay and Score Detections (Server-Side)

### 5.1 Unnatural Movement Curves
- **File:** `shitcheat/core/autobot/motion_core.hxx`
- **What happens:** Autobot moves the cursor using 5th-degree polynomials (quintic splines). It guarantees that speed and acceleration change smoothly and stop cleanly at the target.
- **How to detect:** Human cursor movement has jitter, slight overshoots, and uneven speeds. Fitting replay cursor positions to polynomial equations reveals an exact mathematical curve with zero human noise.

### 5.2 Fake Accuracy Timing
- **File:** `shitcheat/core/relax/relax.hxx`
- **What happens:** The cheat's Target Accuracy mode tries to simulate human mistakes (like getting 98.5% accuracy). To keep combo, it only places 100s on isolated notes with gaps >= 170ms and spaced at least 3 notes apart.
- **How to detect:** Real players make timing mistakes on fast streams and dense bursts. In replays from this cheat, all 100s occur on slow, easy single notes and 0 occur on fast streams.

---

## Detection Summary Table

| Check | What it looks for | How easy to detect |
| :--- | :--- | :--- |
| **`0xDEADC0DE` check** | Mouse extra info set by cheat | Very easy (1 line of code) |
| **Injected flag check** | Windows `LLMHF_INJECTED` / `LLKHF_INJECTED` flags | Very easy (hook) |
| **Overlay window check** | Window class `"  "` with size 820x620 | Very easy (`FindWindowA`) |
| **Raw Input check** | Cursor moving without hardware `WM_INPUT` | Easy |
| **Config folder check** | `%APPDATA%\lame\configs` on disk | Very easy |
| **Replay spline check** | Cursor path fits clean 5th-degree polynomial | Medium (math analysis) |
| **Timing error check** | 100s only on slow notes, none on streams | Medium (stats analysis) |
