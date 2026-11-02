# lamev2 Detector & Analysis

Analysis, detection vectors, and an in-game detector for the `lamev2` osu! cheat.

---

## Files in This Repo

- [**case_study.md**](case_study.md): Explains how the cheat works and why its design makes it easy to detect.
- [**detection.md**](detection.md): Technical breakdown of every way to detect the cheat.
- [**detector/**](detector/): A 48-line Rust DLL that can be injected into osu! Stable or Lazer to catch the cheat while it runs.

---

## Quick Detection Summary

| Check | What It Looks For | Code |
| :--- | :--- | :--- |
| **`0xDEADC0DE` Check** | Magic number the cheat sets on mouse moves | Mouse hook checking `ms.dwExtraInfo == 0xDEADC0DE` |
| **Injected Input Check** | Simulated mouse and keyboard inputs | Hook checking `(ms.flags & 1) != 0` and `(kb.flags & 1) != 0` |
| **Overlay Window Check** | Cheat's ImGui overlay window | `FindWindowA(b"  \0", null)` |
| **Config Check** | Cheat settings folder | Checking if `%APPDATA%\lame\configs` exists |

---

## Building the Detector DLL

Requirements: Rust with MSVC.

```bash
cd detector

# For osu! Lazer (64-bit)
cargo build --release

# For osu! Stable (32-bit)
cargo build --release --target i686-pc-windows-msvc
```
