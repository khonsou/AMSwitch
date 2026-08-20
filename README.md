# AM Profile Switch

**Game in, profile on. Game out, back to default.**

**English** | [简体中文](README.zh-CN.md)

A lightweight Windows tray app: it watches the foreground app and automatically switches the onboard profiles of your AM devices over HID — supporting AM Infinity .97 / .100 (mice), BE745 / BD75 (keyboards), and all future AM devices with onboard profiles.

- Game launches: mouse and keyboard each switch to the onboard profile set by the matching rule
- Game exits: every device falls back to the default profile
- Device offline: silently skipped — no errors, no interruptions

## Status

Early stage ｜ PRD v0.3.8 under review (Chinese) ｜ M0 technical validation in progress

## Repository layout

```
docs/PRD.md                     Product requirements doc (Chinese): tech selection, HID protocol baseline, fault matrix, milestones
demos/foreground-detection/     M0 proof #1: Rust foreground-detection demo (SetWinEventHook, event-driven, zero polling)
demos/gui/index.html            GUI interactive mock: single-file H5 — open in any browser, every button works, 中/EN toggle in the title bar
```

## Key design constraints

- **Switch only, never write**: the HID send path is limited to a 3-command whitelist; destructive commands are hard-blocked; command-ID enumeration is forbidden
- **Anti-cheat red lines**: pure user-mode process — no injection, no game memory reads, no keyboard/mouse hooks
- **Lightweight**: ≤ 60 MB resident RAM, ≤ 0.5% idle CPU, ≤ 15 MB installer, ≤ 500 ms switch latency
- Stack: Tauri 2.x + Rust (see PRD §5)

## Community

AngryMiao community members are welcome to join: issues and PRs in **English or Chinese** are both fine. The GUI mock and the product UI are bilingual (中/EN).

## Acknowledgements

The HID protocol baseline builds on the community reverse-engineering project [am97-cli](https://github.com/TheMasterDingo/am97-cli) (MIT) — many thanks to TheMasterDingo.
