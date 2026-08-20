//! AM App Switch — M0 前台应用检测 Demo（Windows only）
//!
//! 机制：SetWinEventHook + EVENT_SYSTEM_FOREGROUND，事件驱动、零轮询。
//! 运行：cargo run --release，然后随便切换窗口，终端实时打印前台应用。

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowTextW, GetWindowThreadProcessId,
    TranslateMessage, EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
static SEQ: AtomicU64 = AtomicU64::new(0);

fn main() {
    let _ = START.set(Instant::now());
    println!("AM 前台检测 Demo —— 切换任意窗口试试（Ctrl+C 退出）\n");

    // 启动时先报一次当前前台应用
    unsafe { report(GetForegroundWindow()) };

    unsafe {
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,   // eventMin
            EVENT_SYSTEM_FOREGROUND,   // eventMax
            None,                      // 不注入任何 DLL（OUTOFCONTEXT 模式）
            Some(on_foreground_event), // 回调
            0,                         // 监听所有进程
            0,                         // 监听所有线程
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        if hook.0.is_null() {
            eprintln!("SetWinEventHook 失败");
            std::process::exit(1);
        }

        // WinEvent 回调跑在消息循环上，本线程必须持续泵消息
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }
        let _ = UnhookWinEvent(hook);
    }
}

unsafe extern "system" fn on_foreground_event(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_thread: u32,
    _time: u32,
) {
    if event == EVENT_SYSTEM_FOREGROUND {
        unsafe { report(hwnd) };
    }
}

/// 打印一行：序号 | 相对时间 | exe 名 | pid | 窗口标题
unsafe fn report(hwnd: HWND) {
    if hwnd == HWND::default() {
        return;
    }
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 {
        return;
    }

    // 读进程完整路径再取文件名；读不到（如管理员进程）时降级为 "<未知>"
    let exe = process_path(pid)
        .and_then(|p| {
            Path::new(&p)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "<未知>".to_string());

    let title = window_title(hwnd);
    let n = SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    let t = START.get().map(|s| s.elapsed().as_secs_f32()).unwrap_or(0.0);
    println!("[#{n:>3} | {t:7.1}s] {exe:<24} pid={pid:<7} {title}");
}

/// 读进程完整路径。PROCESS_QUERY_LIMITED_INFORMATION 足够，
/// 不需要管理员权限；跨权限失败时返回 None，调用方降级处理。
fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 260];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut size);
        let _ = CloseHandle(h);
        ok.ok()?;
        Some(String::from_utf16_lossy(&buf[..size as usize]))
    }
}

fn window_title(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = unsafe { GetWindowTextW(hwnd, &mut buf) };
    if len <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..len as usize])
}
