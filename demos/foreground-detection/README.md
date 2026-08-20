# AM Foreground-App Detection Demo (M0 Proof #1)

**English** | [简体中文](README.zh-CN.md)

Implements **F1 foreground-app detection** from the AM Profile Switch PRD: listens for `EVENT_SYSTEM_FOREGROUND` via `SetWinEventHook` — **event-driven, zero polling** — and prints the foreground app name in real time on every window switch.

The source compiles cleanly for the Windows target (`x86_64-pc-windows-gnu`) with zero warnings.

## Setup (one-time)

Install Rust on Windows 10/11 (~2 minutes):

```powershell
winget install Rustlang.Rustup
```

Or download and run https://rustup.rs. Restart the terminal afterwards.

## Run

```powershell
cd am-foreground-demo
cargo run --release
```

The first run downloads dependencies and compiles (~1 minute). Then switch between windows and the terminal prints in real time:

```
AM 前台检测 Demo —— 切换任意窗口试试（Ctrl+C 退出）

[#  1 |    0.0s] explorer.exe             pid=8612
[#  2 |    3.2s] cs2.exe                  pid=15204  Counter-Strike 2
[#  3 |    8.7s] chrome.exe               pid=2108   AM Profile Switch PRD - Google Chrome
[#  4 |   12.1s] cs2.exe                  pid=15204  Counter-Strike 2
```

(The demo's own output is in Chinese — that first line is what the binary prints.)

## What this demo validates

| Checkpoint | How to see it |
|------------|---------------|
| Detection immediacy | Click another window — the terminal prints **instantly**, no polling delay |
| Resource usage | Leave it running: Task Manager shows CPU ≈ 0%, RAM ≈ 1–2 MB (pure native, no UI) |
| exe name capture | The third column is exactly the key the rule engine will match on |
| Games running as admin | Launch a game as administrator and switch to it: the exe name is still readable (`PROCESS_QUERY_LIMITED_INFORMATION` works across privilege levels); the rare unreadable process shows `<未知>` — the release version falls back to process-name matching |
| Event storms | Rapid Alt+Tab: one line per actual foreground change — the release version adds debouncing on top |

## Scope boundary (deliberately not in this demo)

- Rule matching and profile switching (F2/F4, M1 scope)
- Debouncing, tray residence, minimize-to-background
- These are known-solvable and not among the technical risks M0 must retire

## Files

```
am-foreground-demo/
├── Cargo.toml      # Dependency: the windows crate 0.62 (Microsoft's official Rust bindings)
└── src/main.rs     # All logic, ~100 lines, commented line by line
```

## FAQ

- **Antivirus flags it?** The source is fully transparent — have your security team review it; the release build will be EV code-signed.
- **Can I get an exe directly?** On any machine with Rust: `cargo build --release` → `target/release/am-foreground-demo.exe` — single file, portable, no install.
- **Why not hack it together in Python/PowerShell?** A polling-based demo can't validate the core architecture decision — event-driven with zero overhead — and that decision is what determines the release version's resource footprint.
