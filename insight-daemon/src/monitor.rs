use std::sync::{Arc, Mutex};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{HWND, HANDLE};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowTextW, GetWindowThreadProcessId,
    EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::System::ProcessStatus::GetProcessImageFileNameW;
use log::{info, error, warn};
use chrono::Utc;

use crate::models::Activity;
use crate::db::{insert_activity, update_activity_end};
use crate::classifier::ClassifierConfig;

/// 当前正在进行的活动（全局共享状态）
#[derive(Debug, Clone)]
pub struct CurrentActivity {
    pub activity_id: i64,   // 已写入 DB 的记录 ID
    pub app_name: String,
    pub category: String,
    pub start_time: i64,
}

/// 专注提醒状态
#[derive(Debug, Clone, Default)]
pub struct DistractionState {
    /// 连续娱乐/社交类活动的起始时间（None = 当前不在分心状态）
    pub distraction_start: Option<i64>,
    /// 连续分心时长（秒），超过阈值时触发提醒
    pub streak_secs: i64,
    /// 上次发送提醒的时间（避免频繁提醒）
    pub last_alert_at: Option<i64>,
    /// 提醒阈值（秒，默认 15 分钟）
    pub threshold_secs: i64,
}

impl DistractionState {
    pub fn new(threshold_mins: i64) -> Self {
        DistractionState {
            distraction_start: None,
            streak_secs: 0,
            last_alert_at: None,
            threshold_secs: threshold_mins * 60,
        }
    }

    /// 更新分心状态，返回是否需要触发提醒
    pub fn update(&mut self, category: &str, now: i64) -> bool {
        let is_distraction = matches!(category, "entertainment" | "social");

        if is_distraction {
            if self.distraction_start.is_none() {
                self.distraction_start = Some(now);
            }
            self.streak_secs = now - self.distraction_start.unwrap_or(now);

            // 超过阈值且距上次提醒超过 10 分钟
            if self.streak_secs >= self.threshold_secs {
                let should_alert = self.last_alert_at
                    .map(|t| now - t > 600) // 至少间隔 10 分钟
                    .unwrap_or(true);
                if should_alert {
                    self.last_alert_at = Some(now);
                    return true;
                }
            }
        } else {
            // 重置分心计数
            self.distraction_start = None;
            self.streak_secs = 0;
        }
        false
    }

    pub fn is_distracted(&self) -> bool {
        self.streak_secs >= self.threshold_secs
    }
}

/// 监听器上下文（通过线程局部变量传递给回调）
pub struct MonitorContext {
    pub db_conn: Arc<Mutex<rusqlite::Connection>>,
    pub classifier: Arc<ClassifierConfig>,
    pub current: Option<CurrentActivity>,
    pub distraction: DistractionState,
}

// 全局静态上下文（Win32 回调无法携带参数，使用全局状态）
static MONITOR_CTX: std::sync::OnceLock<Arc<Mutex<MonitorContext>>> = std::sync::OnceLock::new();

/// 获取当前分心状态（供 HTTP server 查询）
pub fn get_distraction_state() -> Option<(i64, bool)> {
    let ctx = MONITOR_CTX.get()?.lock().ok()?;
    Some((ctx.distraction.streak_secs, ctx.distraction.is_distracted()))
}

/// 注册 Win32 事件钩子，开始监听前台窗口切换
pub fn start_monitoring(
    db_conn: Arc<Mutex<rusqlite::Connection>>,
    classifier: Arc<ClassifierConfig>,
    distraction_threshold_mins: i64,
) {
    let ctx = MonitorContext {
        db_conn,
        classifier,
        current: None,
        distraction: DistractionState::new(distraction_threshold_mins),
    };
    MONITOR_CTX.set(Arc::new(Mutex::new(ctx))).ok();

    unsafe {
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        if hook.0.is_null() {
            error!("Failed to register WinEventHook!");
        } else {
            info!("WinEventHook registered successfully. Monitoring started.");
        }
    }
}

/// Win32 事件回调 —— 每次前台窗口切换时触发
unsafe extern "system" fn win_event_callback(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _event_time: u32,
) {
    let now = Utc::now().timestamp();

    // 1. 获取窗口标题
    let mut title_buf = [0u16; 512];
    let title_len = GetWindowTextW(hwnd, &mut title_buf) as usize;
    let window_title = OsString::from_wide(&title_buf[..title_len])
        .to_string_lossy()
        .to_string();

    // 2. 获取进程名
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    let app_name = get_process_name(pid).unwrap_or_else(|| "unknown".to_string());

    // 3. 忽略无意义的切换（pid=0 或空标题）
    if pid == 0 || app_name == "unknown" {
        return;
    }

    info!("[{}] Foreground: {} | {}", now, app_name, window_title);

    // 4. 更新全局状态
    let ctx_arc = match MONITOR_CTX.get() {
        Some(c) => c.clone(),
        None => return,
    };
    let mut ctx = match ctx_arc.lock() {
        Ok(c) => c,
        Err(_) => return,
    };

    // 5. 结束上一个活动
    if let Some(ref prev) = ctx.current.clone() {
        if now > prev.start_time {
            let db = ctx.db_conn.lock().unwrap();
            if let Err(e) = update_activity_end(&db, prev.activity_id, now) {
                error!("Failed to update activity end: {e}");
            }
        }
    }

    // 6. 对敏感应用进行标题脱敏
    let safe_title = if ctx.classifier.is_sensitive_app(&app_name) {
        "[protected]".to_string()
    } else {
        window_title
    };

    // 7. 分类并写入新活动
    let category = ctx.classifier.classify(&app_name, &safe_title);

    // 8. 更新分心状态
    let _needs_alert = ctx.distraction.update(&category, now);
    if _needs_alert {
        warn!(
            "Distraction alert: {} minutes of {} category detected!",
            ctx.distraction.streak_secs / 60,
            category
        );
    }

    let new_activity = Activity {
        id: None,
        app_name: app_name.clone(),
        window_title: safe_title,
        category: category.clone(),
        start_time: now,
        end_time: 0, // 未结束，end_time 暂为 0
    };

    let insert_result = {
        let db = ctx.db_conn.lock().unwrap();
        insert_activity(&db, &new_activity)
    };
    match insert_result {
        Ok(id) => {
            ctx.current = Some(CurrentActivity {
                activity_id: id,
                app_name,
                category,
                start_time: now,
            });
        }
        Err(e) => error!("Failed to insert activity: {e}"),
    }
}

/// 从 PID 获取进程名（不含路径）
fn get_process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle: HANDLE = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid,
        ).ok()?;

        let mut buf = [0u16; 1024];
        let len = GetProcessImageFileNameW(handle, &mut buf) as usize;
        if len == 0 {
            return None;
        }
        let full_path = OsString::from_wide(&buf[..len]).to_string_lossy().to_string();
        // 只取文件名部分
        full_path.split(['\\', '/']).last().map(|s| s.to_string())
    }
}
