# AM Profile Switch

**进游戏切配置，出游戏回默认。**

[English](README.md) | **简体中文**

Windows 托盘常驻轻量工具：检测前台应用变化，通过 HID 自动切换 AM 设备的板载配置——支持 AM Infinity .97 / .100（鼠标）、BE745 / BD75（键盘）及后续支持板载配置的全部 AM 设备。

- 进游戏：鼠标、键盘各切到规则指定的板载配置
- 出游戏：全部设备回落到默认配置
- 设备不在线：静默跳过，不报错

## 状态

立项阶段 ｜ PRD v0.3.8 评审中 ｜ M0 技术验证进行中

## 仓库结构

```
docs/PRD.md                     初期 PRD：技术选型、HID 协议基线、容错矩阵、里程碑
demos/foreground-detection/     M0 验证件 #1：Rust 前台检测 demo（SetWinEventHook，事件驱动零轮询）
demos/gui/index.html            GUI 交互稿：单文件 H5，浏览器直接打开，全部按钮可点，标题栏可切中英文
```

## 关键设计约束

- **只切配置，不写参数**：HID 发送路径仅 3 条白名单命令，破坏性命令硬编码拦截，禁止枚举命令 ID
- **反作弊红线**：纯用户态进程，不注入、不读游戏内存、不 hook 键鼠输入
- **轻量化**：常驻内存 ≤ 60MB，空闲 CPU ≤ 0.5%，安装包 ≤ 15MB，切换 ≤ 500ms
- 技术选型：Tauri 2.x + Rust（详见 PRD §5）

## 社区参与

欢迎海外 Community 用户参与：Issue 和 PR 用中文或英文都可以；GUI 交互稿与产品 UI 均为中英双语，海外评审可走英文全流程。

## 致谢

HID 协议基线参考社区逆向成果 [am97-cli](https://github.com/TheMasterDingo/am97-cli)（MIT），感谢作者 TheMasterDingo。
