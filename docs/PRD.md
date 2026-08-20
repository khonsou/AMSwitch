# AM Profile Switch 初期 PRD v0.4

**状态**：立项评审稿｜**平台**：Windows 10/11｜**关联产品**：AM Infinity .97 / .100（鼠标）、BE745 / BD75（键盘）及后续支持板载配置的全部 AM 设备

> v0.4：全文精简重写（347 → 258 行），只保留决策与结论；演进过程由 git 历史承担。

## 目录

- [1. 背景与问题](#1-背景与问题)
- [2. 目标与非目标](#2-目标与非目标)
  - [2.1 目标（MVP）](#21-目标mvp)
  - [2.2 非目标（MVP 不做）](#22-非目标mvp-不做)
- [3. 核心用户故事](#3-核心用户故事)
- [4. 功能需求](#4-功能需求)
  - [F1 前台应用检测](#f1-前台应用检测)
  - [F2 规则引擎（两层模型）](#f2-规则引擎两层模型)
  - [F3 设备管理（容错核心）](#f3-设备管理容错核心)
  - [F4 切换执行（协议基线已实证）](#f4-切换执行协议基线已实证)
  - [F5 遥测（MVP 内置）](#f5-遥测mvp-内置)
  - [F6 状态与反馈](#f6-状态与反馈)
- [5. 技术选型](#5-技术选型)
  - [5.1 框架对比](#51-框架对比)
  - [5.2 架构（独立 App，为并入 AM Master 预留边界）](#52-架构独立-app为并入-am-master-预留边界)
  - [5.3 反作弊与安全红线](#53-反作弊与安全红线)
  - [5.4 分发与更新](#54-分发与更新)
- [6. 数据模型（rules.json）](#6-数据模型rulesjson)
- [7. 容错设计明细](#7-容错设计明细)
- [8. 里程碑（建议）](#8-里程碑建议)
- [9. QA 记录](#9-qa-记录)
- [10. 社区生态策略](#10-社区生态策略)
- [参考来源](#参考来源)

---

## 1. 背景与问题

鼠标、键盘都支持板载配置，但切换全靠手动打开网页驱动，打断心流，多数人干脆不切。本产品：检测前台应用，自动把鼠标 + 键盘一起切到对应板载配置，退出回默认，用户零操作。社区旁证：粉丝开源的 am97-cli 逆向了 .97 的 HID 协议，动机正是让鼠标参与桌面自动化 [^11^]——需求真实，协议障碍已扫清。

## 2. 目标与非目标

### 2.1 目标（MVP）

| # | 目标 | 验收标准 |
|---|------|----------|
| G1 | 前台变化自动切板载配置 | 切换 → 生效 ≤ 500ms，无感知输入中断 |
| G2 | 鼠标 + 键盘联动 | 一条规则作用于多台在线设备，各设备配置独立（两层模型） |
| G3 | 设备缺失不报错 | 离线静默跳过，不弹窗、不中断其余设备 |
| G4 | 轻量化常驻 | 内存 ≤ 60MB、空闲 CPU ≤ 0.5%、安装包 ≤ 15MB，托盘常驻开机自启 |
| G5 | 反作弊零风险 | 不注入、不读游戏内存、不 hook 输入；纯用户态 + 仅与自家 HID 配置接口通信 |
| G6 | 失败可观测 | 失败 100% 有日志并匿名上报，按游戏 ID 可统计，先于售后发现问题 |

### 2.2 非目标（MVP 不做）

配置编辑（仍由网页驱动承担，本工具只发切换指令）、手动切换/锁定模式、向设备写参数、多 Windows 用户管理、macOS/Linux、云同步、游戏内 overlay、CLI。

## 3. 核心用户故事

1. CS2 玩家：双击 cs2.exe，鼠标切游戏配置、键盘切游戏键位；退出游戏，一切自动回默认。
2. 多设备用户：.100 + BD75 一起切；只带鼠标出门时键盘离线也不报错。
3. 新用户：预置热门游戏开箱即用，也能把最近用过的程序一键添加为规则。
4. 产品经理：用户切换失败，我在后台报表里先于售后看到。

## 4. 功能需求

### F1 前台应用检测

- `SetWinEventHook` 监听 `EVENT_SYSTEM_FOREGROUND`，事件驱动、零轮询 [^1^][^2^]；Rust demo 已完成编译验证（M0 验证件 #1）。
- 解析 exe 名 / 路径 / 窗口标题；管理员权限的游戏经 `PROCESS_QUERY_LIMITED_INFORMATION` 仍可读 exe 名，读不到降级为进程名匹配——**本工具不要求管理员运行**。
- 防抖去重：同进程连续触发不重复写设备。
- 持续维护「最近前台程序」历史（排除自身），供添加游戏时选用——点「添加」按钮时本工具必然在前台，所以取的是**历史里的上一个**，不是当前。

### F2 规则引擎（两层模型）

各设备板载槽位数不一致，规则 =「游戏 → 哪些设备 → 每台设备各切到哪个配置」两层，**只存标识引用、不复制配置内容**（配置本体以设备板载为唯一事实来源）。

- **回落语义**：退出游戏一律回落全局默认配置，规则不记录任何还原关系——消费者心智只有一句：进游戏切配置，出游戏回默认。
- **术语纪律**：界面只暴露「产品名 + 板载配置名」，「槽位 / slot」仅是协议层细节，不出现在消费者文案。
- **槽位名称风险**：设备状态接口只暴露槽位序号 [^11^]，M0 验证能否读到槽位名——读不到则降级为「槽位号 + 用户本地别名」。
- **预置游戏包**（CS2、Valorant、Apex、PUBG、永劫无间、LoL 等）可改可删；用户删过的条目记 tombstone，预置包更新**不得复活**。
- **删除规则**：行尾 ✕ 常显、两步确认不弹窗，预置与自建同权；删除生效中的规则 = 立即回落默认（与退出游戏同路径）。
- **「添加游戏」弹层只选程序、不配设备**（同一功能全 app 只有一个入口，配置只在规则行完成）。三层来源保证冷启动不落空：① 预置包开箱即用；② 最近前台历史（默认选中上一个）；③ 手动选 exe（GetOpenFileName 兜底）。
- 新规则出生时全部设备指向默认配置，在规则行逐设备改选；配置列表实时读自设备，杜绝手填；**修改生效中的规则，对应设备立即补切**（所见即所得）。
- **查重**：exe 文件名（不区分大小写）即规则唯一身份，预置包同样参与。选择时预防（标「已添加」、按钮变「查看规则」）；保存时兜底（命中则定位高亮已有规则，不新建）。已知边界：同名 exe 不区分；启动器/本体多 exe 合并留待后续版本。
- 存储：`%APPDATA%/AM/app-switch/rules.json`（§6）。（后续可选增强：正在运行的进程列表、扫描 Steam/Epic 游戏库。）

### F3 设备管理（容错核心）

- 只对受支持型号（.97 / .100 / BE745 / BD75 及后续）通信，其余不枚举、不出现。
- 枚举参数（社区实证 [^11^]）：VID `0x0E8D`；dongle PID `0x0703`（.97，其余型号 M0 确认）；鼠标注线 PID `0x0880`（兼容备选 ID 对 `0x35A1/0x0035`）；Usage Page `0xFF13`；Feature Report `0x14`。
- **拓扑**：鼠标经 dongle 连接时是 relay 目标（`device_id = 0x80`），线缆直连 `0x00`；两条路径都发现、优先 dongle [^11^]。
- 热插拔：`RegisterDeviceNotification` + `WM_DEVICECHANGE`，移除即关句柄、标记 offline [^3^][^4^]；接入时**瞬态**读取型号/固件/槽位清单后立即释放（见 F4）。
- 设备状态并入状态页，不单设页签。

### F4 切换执行（协议基线已实证）

帧格式（社区逆向 + WebHID 抓包交叉验证 [^11^][^5^]，M0 实测复核）：

```
[payload_len][device_id] [05][type][len_lo][len_hi][cmd_lo][cmd_hi][data...]
```

- `device_id`：`0x00` 本体 / `0x80` dongle relay；`type`：`0x5A` 命令 / `0x5B` 回复 / `0x5D` 写确认；payload 零填充至 61 字节，走 Feature Report `0x14`。
- 忙等待：设备忙返回零长度帧，轮询 30ms × ≤40 次；一次读取可能堆叠多帧，按 cmd id 匹配到为止 [^11^]。

指令白名单（只允许这三类）：

| 命令 | ID | 用途 | 社区验证状态 [^11^] |
|------|-----|------|---------------------|
| `getProfile` | 12489 | 读当前槽位 | ✅ 真机实测 |
| `changeProfile` | 12488 | **切槽位（唯一写操作）** | ⚠️ 转录自官方 JS 未实测——**M0 第一验证项** |
| 固件/设备信息读取 | M0 确认 | 特性门控 | —— |

破坏性命令黑名单（硬编码进发送路径兜底拒绝 [^11^]）：12494 `resetSettings`（清空全部配置）、12395 `clearBonded`（毁掉 2.4G 配对）、12394 `setPairingUniaa`（配对握手）、4353 `rebootPairingDevice`（断链重启）。

- **瞬态连接**：网页驱动打开时会独占设备句柄 [^11^]，故平时不持有任何句柄；切换瞬间才 open → changeProfile → 读回校验 → close，单次占用 < 200ms；打开失败（被占用）→ 延迟重试 ≤ 2 次 → 仍失败标「忙」记日志 + 遥测，不弹窗；连续失败温和提示「请关闭网页驱动页签」。
- 多设备并行下发，单设备失败不阻塞其他设备。

### F5 遥测（MVP 内置）

目的：**先于售后发现问题**。失败事件全量上报，成功事件聚合计数（游戏 ID × 设备）。

- 失败载荷字段：`event / game_id / device / target_profile / error_code / retries / latency_ms / fw_version / app_version / install_id`。
- 错误码：`WRITE_TIMEOUT` / `DEVICE_BUSY`（被网页驱动占用）/ `DEVICE_OFFLINE` / `PROFILE_NOT_FOUND` / `VERIFY_MISMATCH`（读回不一致）/ `PROTOCOL_ERROR`。
- 隐私红线：`install_id` 为首次启动生成的随机 UUID，不含硬件 ID / 账号；只收 exe 名，**不收**窗口标题、路径、使用时长；首次启动告知，设置中可关闭。
- 本地留最近 100 条切换日志，供社区反馈与售后排查。

### F6 状态与反馈

- 页签顺序 = 动手频率：**规则（默认首页）/ 状态 / 设置**。状态页上半（一句话状态 + 流水线 + 设备清单）面向消费者，日志区面向测试与支持。
- 一句话状态示例：「待命 · 全部设备处于默认配置」/「「XX」规则生效中 · 退出后自动回默认」。
- 托盘图标三态：运行中 / 有设备离线 / 有设备失败；托盘菜单仅「打开面板、退出」。
- rules.json 不向消费者展示；诊断价值由设置页「复制诊断信息」承担（规则 + 设备/固件版本 + 最近日志一键复制），自动发现由遥测兜底。

## 5. 技术选型

### 5.1 框架对比

| 维度 | **Tauri 2.x（推荐）** | Electron | WinUI 3 / WPF |
|------|----------------------|----------|---------------|
| 安装包 | 3–10MB [^6^] | 85–250MB [^6^] | 中（依赖 .NET 运行时） |
| 常驻内存 | 空闲约 30–50MB [^6^][^7^] | 100–300MB [^7^] | 中 |
| 复用网页驱动前端 | ✅ WebView2 直接复用 | ✅ | ❌ 重写 XAML |
| HID/系统 API | Rust：hidapi + windows-rs，免驱动 | node-hid 原生模块，分发麻烦 | C# P/Invoke |
| 自动更新 | 内置增量 [^6^] | electron-updater | 需自建 |

**结论：Tauri 2.x。** 轻量化是硬指标（游戏用户对常驻进程占用极度敏感）；网页驱动前端可直接搬进 WebView2，开发量最小；Rust hidapi 与社区 Python 验证过的底层同为 libusb hidapi，API 一一对应 [^11^]。

### 5.2 架构（独立 App，为并入 AM Master 预留边界）

```
Tauri App（WebView2 UI：规则 / 状态 / 设置，复用网页驱动前端技术栈）
  └─ am-profile-switch-core（独立 Rust crate，零 Tauri 依赖）
       ├─ ForegroundMonitor   SetWinEventHook 事件驱动【M0 验证件 #1 已编译验证】
       ├─ DeviceManager       hidapi 枚举 + WM_DEVICECHANGE 热插拔 + dongle 拓扑
       ├─ HidProtocol         帧编解码 + 忙等待轮询 + 命令白名单/黑名单
       ├─ RuleEngine          rules.json 两层规则匹配与回落
       ├─ SwitchOrchestrator  瞬态连接、并行下发、重试、读回校验
       └─ Telemetry           失败全量 / 成功聚合，匿名上报
```

两条架构纪律：① 切换链路全部在 Rust core，UI 冻结/崩溃不影响核心，UI 只是观察者；② core 独立 crate、零 Tauri 依赖，未来并入 AM Master 是「搬库」不是「重写」。

### 5.3 反作弊与安全红线

现代反作弊（Vanguard / EAC / BattlEye / Ricochet）检测的是进程注入、内存读写、可疑驱动、输入自动化 [^8^]——本工具四样都不碰：纯用户态，只通过标准 HID API 向自家设备的**配置接口**发 Feature Report；同类纯外部工具长期无冲突先例 [^9^][^10^]。

红线（写进开发规范）：
- ❌ 不 OpenProcess 游戏进程做任何读写；不装内核/interception 类驱动；不做全局低级键鼠 hook
- ❌ **禁止枚举/扫描 HID 命令 ID**：扁平 ID 空间内读、写、破坏性命令混杂且字节层面无法区分，社区开发者扫描时曾误触发 `clearBonded`，真实丢失配对与全部配置 [^11^]。发送路径只放行白名单，黑名单硬编码兜底
- ❌ 不提供 `raw` 任意命令发送能力（社区 CLI 有 `am97 raw`，我们不给用户这个枪口）
- ✅ 发布前 EV 代码签名；官网挂「反作弊兼容性说明」页

### 5.4 分发与更新

Tauri 内置产出 NSIS/MSI 安装包 + 增量自动更新 [^6^]；代码签名积累 SmartScreen 信誉；安装器检测 WebView2 缺失时引导安装（Win10 1803+ / Win11 基本预装）。

## 6. 数据模型（rules.json）

只存标识引用，不复制配置内容（兼容「槽位名」与「槽位号 + 别名」两种形态，取决于 M0 验证结果）：

```json
{
  "version": 2,
  "rules": [
    {
      "game_id": "cs2",
      "match": ["cs2.exe"],
      "apply": [
        { "device": "AM Infinity .100", "slot": 2, "alias": "游戏" },
        { "device": "BD75", "slot": 1, "alias": "游戏" }
      ]
    }
  ]
}
```

- `apply` 即两层模型的落盘形式；**无 restore 字段**——退出一律回落全局默认配置（各设备 0 号配置）。
- `alias` 为本地显示名；若 M0 验证设备可读槽位名，则自动同步为设备名。
- 某设备不在线 → 静默跳过该条，其余设备照常执行。

## 7. 容错设计明细

| 场景 | 行为 |
|------|------|
| 部分设备离线 | 只切在线设备，离线者台账标「离线」，无弹窗 |
| 全部设备离线 | 规则照常匹配，托盘提示「等待设备接入」，接入后自动补切 |
| 切换中途设备被拔出 | 关句柄、本次写入作废、记日志，不重试风暴 |
| 规则中的槽位在设备上不存在 | 跳过该设备，遥测 `PROFILE_NOT_FOUND` |
| 写入错误/超时（含忙等待超限） | 重试 ≤ 2 次 → 标失败 → 托盘角标 + 日志 + 遥测 `WRITE_TIMEOUT` |
| 设备被网页驱动占用 [^11^] | 瞬态连接使冲突窗口 < 200ms；仍撞车则延迟重试，连续失败提示关闭网页驱动页签，遥测 `DEVICE_BUSY` |
| 读回校验与目标不一致 | 重试 1 次仍失败，遥测 `VERIFY_MISMATCH` |
| 设备不在受支持清单 / 固件过旧 | 不枚举不通信；旧固件提示升级，遥测记固件分布 |
| dongle 不在、插线直连 | 自动切换 0x80/0x00 寻址路径，用户无感知 |

## 8. 里程碑（建议）

| 阶段 | 内容 | 周期 |
|------|------|------|
| M0 技术验证 | ✅ 前台检测 demo（已编译验证）；HID demo：①真机验证 changeProfile(12488) 切槽位（第一优先）②枚举四款设备 VID/PID 与拓扑 ③槽位名可读性（定 F2 落盘形态）④dongle + 有线两路径实测 | 1–2 周 |
| M1 MVP | 两层规则引擎 + 设备管理 + 切换编排 + 托盘 + 预置游戏包 + 遥测；内部 dogfood | 3–4 周 |
| M2 公测 | 自动更新、代码签名、社区测试招募（优先邀请 am97-cli 作者，见 §10）；按遥测迭代 | 2–3 周 |
| M3 正式 | 官网发布 + 反作弊说明页 + 与网页驱动入口互链；评估并入 AM Master | 1 周 |

## 9. QA 记录

| # | 问题 | 结论 |
|---|------|------|
| 1 | 切换语义？ | 只做槽位切换（changeProfile），Host 不控制具体参数 |
| 2 | 槽位数不一致？ | 两层规则模型（游戏 → 设备 → 槽位），运行时按各设备实际清单解析（F2/§6） |
| 3 | 数据存什么？ | 只存设备名、配置标识、游戏 ID；无逐规则还原字段，退出一律回落默认 |
| 4 | 产品归属？ | 独立 App 先行；core 独立 crate，公测达标后并入 AM Master |
| 5 | 多 Win 用户？ | 不处理，每用户自行启动自己的实例 |
| 6 | 遥测？ | MVP 内置：游戏 ID 维度统计 + 失败全量日志，匿名、可关闭 |
| 7 | 槽位名可读吗？ | 社区实证状态接口只有槽位序号 [^11^]；M0 验证，读不到用「槽位号 + 本地别名」 |
| 8 | 添加弹层里要配设备吗？ | 不配。弹层只选程序，配置只在规则行——同一功能不出现在两个页面 |
| 9 | 用户添加的 app 要查重吗？ | 要。exe 名即规则身份；选择时预防 + 保存时兜底，命中直接定位已有规则不新建 |
| 10 | 规则能删吗？预置删了会被更新加回来吗？ | 能删，预置/自建同权，行尾 ✕ 两步确认不弹窗；删生效规则立即回落默认；tombstone 防复活 |

## 10. 社区生态策略

am97-cli 的价值是**协议情报**（帧格式、命令 ID、黑名单、忙等待机制已吸收进 F4），不是代码资产：不集成、不依赖，也不自己做 CLI——前台检测与协议发送天然同进程，拆出 CLI 只会多一个要分发、签名、维护的产物。保留两个低成本动作：发布物料致谢作者 TheMasterDingo、M2 公测优先邀请（现成的种子用户 + 最好的协议 review 者）；脚本化/自动化需求已由社区作品覆盖（MIT），不正面重叠，未来若大到值得官方介入，优先支持/收编而非替代。

---

### 参考来源

[^1^]: AutoF11 — 基于 SetWinEventHook 监听 EVENT_SYSTEM_FOREGROUND 的成熟开源实践：https://github.com/DanielCoffey1/AutoF11
[^2^]: FPS Booster Pro+ — 同样采用 SetWinEventHook + EVENT_SYSTEM_FOREGROUND 做即时游戏检测：https://fpsboosterpro.com/
[^3^]: Silicon Labs AN532《HID Library API Specification》— WM_DEVICECHANGE + DBT_DEVICEARRIVAL/DBT_DEVICEREMOVECOMPLETE 热插拔标准模式：https://www.silabs.com/documents/public/application-notes/AN532.pdf
[^4^]: Microsoft Learn — RegisterDeviceNotification / WM_DEVICECHANGE 设备插拔检测：https://learn.microsoft.com/en-us/windows/win32/devio/detecting-media-insertion-or-removal
[^5^]: Understanding WebHID and WebUSB（Configur.io）— 配置器「Output Report 下发 → Input Report 确认」的 REPL 式协议模型：https://blog.jonathanlau.io/posts/understanding-webhid-and-webusb-configur/
[^6^]: Tauri vs Electron 2026 基准（安装包 3.2MB vs 85MB、空闲内存 42MB vs 168MB、空闲 CPU <0.5%）：https://tech-insider.org/tauri-vs-electron-2026/
[^7^]: OpenReplay — Tauri/Electron 实际占用对比（安装包 2–10MB vs 80–150MB，空闲内存 30–50MB vs 150–300MB）：https://blog.openreplay.com/comparing-electron-tauri-desktop-applications/
[^8^]: Anti-Cheat Systems Explained — 反作弊检测范畴：注入、内存读写、可疑驱动、行为异常：https://madchad.net/anti-cheat-systems-explained/
[^9^]: Crosshair X 兼容性说明 — 纯外部 overlay 在 EAC/BattlEye/Ricochet/VAC/Vanguard 下无冲突：https://centerpointgaming.com/is-crosshair-x-safe.html
[^10^]: Vertex Zens — 反作弊扫描进程/驱动/内存，不直接针对 USB HID 输入流：https://vertexzens.com/blog/are-cronus-zen-scripts-detectable-2026
[^11^]: TheMasterDingo/am97-cli — 社区逆向的 AM Infinity .97 HID 协议与 CLI 实现（MIT）：https://github.com/TheMasterDingo/am97-cli
