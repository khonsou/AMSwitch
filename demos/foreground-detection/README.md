# AM 前台应用检测 Demo（M0 验证件 #1）

对应《AM App Switch 初期 PRD v0.2》的 **F1 前台应用检测**：用 `SetWinEventHook` 监听 `EVENT_SYSTEM_FOREGROUND`，**事件驱动、零轮询**，前台窗口一切换就实时打印应用名。

源码已通过对 Windows 目标（`x86_64-pc-windows-gnu`）的编译验证，零警告。

## 环境准备（仅需一次）

在 Windows 10/11 上安装 Rust（约 2 分钟）：

```powershell
winget install Rustlang.Rustup
```

或打开 https://rustup.rs 下载运行。装完重开终端。

## 运行

```powershell
cd am-foreground-demo
cargo run --release
```

首次运行会下载依赖并编译（约 1 分钟）。之后随便切换窗口，终端实时输出：

```
AM 前台检测 Demo —— 切换任意窗口试试（Ctrl+C 退出）

[#  1 |    0.0s] explorer.exe             pid=8612
[#  2 |    3.2s] cs2.exe                  pid=15204  Counter-Strike 2
[#  3 |    8.7s] chrome.exe               pid=2108   AM App Switch PRD - Google Chrome
[#  4 |   12.1s] cs2.exe                  pid=15204  Counter-Strike 2
```

## 这个 demo 验证什么

| 验证点 | 怎么看 |
|--------|--------|
| 检测即时性 | 点一下别的窗口，终端**立刻**打印，无轮询延迟 |
| 资源占用 | 挂着不动，任务管理器看 CPU ≈ 0%、内存 ≈ 1–2MB（纯原生无 UI） |
| exe 名获取 | 第三列即为规则引擎要匹配的 key |
| 管理员权限游戏 | 用管理员身份跑个游戏切过去：exe 名仍能读到（`PROCESS_QUERY_LIMITED_INFORMATION` 跨权限可读）；个别读不到的进程会显示 `<未知>`，正式版降级为进程名匹配 |
| 事件风暴 | 快速 Alt+Tab 连切，每次前台变化只打一行——正式版在此基础上加防抖去重 |

## 与正式版的边界（本 demo 刻意不做）

- 规则匹配与配置切换（F2/F4，M1 范围）
- 防抖去重、托盘常驻、最小化到后台
- 这些都是确定能做的事，不属于 M0 要排的技术风险

## 文件说明

```
am-foreground-demo/
├── Cargo.toml      # 依赖：windows crate 0.62（微软官方 Rust 绑定）
└── src/main.rs     # 全部逻辑，约 100 行，含逐行注释
```

## 常见问题

- **杀毒软件报警？** 源码全透明，可让安全团队 review；正式版会做 EV 代码签名。
- **能直接出 exe 吗？** 在有 Rust 环境的机器上 `cargo build --release`，产物在 `target/release/am-foreground-demo.exe`，单文件、免安装、拷走即用。
- **为什么不用 Python/PS 快速糊一个？** 轮询式 demo 无法验证「事件驱动 + 零开销」这个核心架构决策，而这个决策决定了正式版的资源占用表现。
