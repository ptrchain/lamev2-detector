# Case Study: Why lamev2's Approach Is Bad

The developer of `lamev2` (in `shitcheat/`) claimed in `ARCHITECTURE.md` that their cheat is well-designed and hard to detect because:
1. It is external and only reads game memory (no memory writes, no DLL injection).
2. It uses direct NT syscalls.
3. It uses smooth math (quintic Hermite splines) for mouse movement.

In reality, the cheat makes basic mistakes that make it trivial to detect.

---

## 1. Hardcoded Magic Value (`0xDEADC0DE`)

In `impl/memory/input.hxx`, every mouse move the cheat injects sets:
```cpp
in.mi.dwExtraInfo = 0xDEADC0DE;
```
The developer did this so their own mouse hook (`mouse_hook.hxx`) could tell the difference between the player's physical mouse movements and the cheat's simulated mouse movements:
```cpp
if ( ms->dwExtraInfo == 0xDEADC0DE ) {
    return CallNextHookEx( self->m_hook, code, wparam, lparam );
}
```
Because of this, any anticheat or in-game hook just has to check `GetMessageExtraInfo()` or the hook struct. If it sees `0xDEADC0DE`, the player is cheating.

---

## 2. Obvious Overlay Window Class (`"  "`)

In `impl/ui/overlay.cxx`, the cheat sets up a DirectX 11 + Dear ImGui overlay:
```cpp
wc.lpszClassName = L"  "; // two space characters
CreateWindowExW(..., wc.lpszClassName, L"  ", WS_POPUP, 0, 0, 820, 620, ...);
```
The developer probably thought naming the window class two spaces would hide it.

Normal Windows applications do not create top-level, topmost, layered windows with a class name of two spaces and a size of 820x620. A detector can find this with a single line of code:
```c
HWND hwnd = FindWindowA("  ", NULL);
```

---

## 3. Synthetic Input and Raw Input

The cheat relies heavily on the idea that external input simulation (`SendInput`, `NtUserInjectKeyboardInput`) cannot be caught. This ignores how Windows handles input:

- **Injected flags:** When user-mode APIs inject mouse or keyboard input, the Windows kernel sets the `LLMHF_INJECTED` and `LLKHF_INJECTED` flags. A standard low-level hook can see these flags immediately. Real hardware inputs never have these flags set.
- **Raw Input (`WM_INPUT`):** osu! supports Raw Input. Physical mice and tablets send raw HID reports with a valid hardware device handle (`hDevice`). Injected inputs do not produce valid raw HID reports. Comparing cursor movement against incoming raw input packets instantly exposes that the cursor is moving without hardware input.

---

## 4. Allocating RWX Memory for Syscalls

In `impl/memory/syscall.hxx`, the cheat allocates executable memory to run direct syscalls:
```cpp
m_stub = VirtualAlloc( nullptr, 32, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE );
```
It writes a small template (`mov r10, rcx; mov eax, ...; syscall; ret`) into this buffer to call `NtReadVirtualMemory` and `NtOpenProcess`.

Memory pages marked as `PAGE_EXECUTE_READWRITE` (RWX) are one of the first things any anticheat or scanner looks for. Allocating RWX memory in an external process to bypass hooks makes the process stand out immediately.

---

## 5. Unrealistic Movement and Replay Data

The developer spent a lot of time on a pure motion engine using quintic Hermite splines. While the math looks nice on paper, it causes two big problems:

1. **No human jitter:** The cursor path follows an exact 5th-degree polynomial with zero acceleration at the end of each jump. Real players have hand tremor, overshooting, and micro-corrections. Replays with zero deviations from a polynomial curve are easy to flag with basic statistical checks.
2. **Fake-looking accuracy distribution:** The cheat has a "Target Accuracy" mode that tries to simulate human accuracy (e.g. 98.5%). To avoid breaking combos, the cheat only drops 100s on slow, isolated single circles (notes with gaps >= 170ms and spaced at least 3 objects apart). It never drops 100s on fast streams or difficult patterns. In match stats, this creates an unnatural pattern where a player has 100% accuracy on hard parts and randomly misses timing on easy, isolated notes.

---

## Summary

The developer was arrogant about their "pure architecture" and "external safety", but:
- They tagged their own mouse input with `0xDEADC0DE`.
- They gave their overlay an easily searchable window class (`"  "`).
- They used user-mode input injection that sets the OS injected flags.
- They left config files sitting in `%APPDATA%\lame\configs\`.
- A 48-line Rust DLL is enough to detect all of it.
