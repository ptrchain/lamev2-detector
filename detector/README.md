# lamev2 Detector DLL

A small Rust DLL you can inject into osu! (Stable or Lazer) to detect the `lamev2` cheat while it runs.

---

## What It Checks

1. **`0xDEADC0DE` Mouse Check:**
   - The cheat sets `in.mi.dwExtraInfo = 0xDEADC0DE` on every mouse move it makes.
   - The DLL catches this using a mouse hook.

2. **Injected Input Check:**
   - Windows flags simulated mouse moves and key presses with `LLMHF_INJECTED` and `LLKHF_INJECTED`.
   - The DLL logs an alert if any gameplay input has these flags set.

3. **Overlay Window Check:**
   - The cheat's overlay window has a class name of `"  "` (two spaces).
   - The DLL checks for this window using `FindWindowA`.

---

## How to Build

### For osu! Stable (32-bit):
```bash
cargo build --release --target i686-pc-windows-msvc
```
DLL output: `target/i686-pc-windows-msvc/release/detector.dll`

### For osu! Lazer (64-bit):
```bash
cargo build --release
```
DLL output: `target/release/detector.dll`

---

## How to Use

1. Start `osu!.exe`.
2. Inject `detector.dll` into the osu! process.
3. A console window will pop up and print messages if it detects the cheat.
