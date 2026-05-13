use chrono::Utc;
use log::{error, info};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::ProcessStatus::GetProcessImageFileNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowTextW, GetWindowThreadProcessId, EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS,
};

use crate::classifier::ClassifierConfig;
use crate::db::{insert_activity, query_today_focus_secs, update_activity_end};
use crate::models::{Activity, CurrentSession, OverlayData};

/// 专注提醒状态
#[derive(Debug, Clone, Default)]
pub struct DistractionState {
    pub distraction_start: Option<i64>,
    pub streak_secs: i64,
    pub last_alert_at: Option<i64>,
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

    pub fn update(&mut self, category: &str, now: i64) -> bool {
        let is_distraction = category == "entertainment" || category == "social";

        if is_distraction {
            if self.distraction_start.is_none() {
                self.distraction_start = Some(now);
            }
            self.streak_secs = now - self.distraction_start.unwrap_or(now);

            if self.streak_secs >= self.threshold_secs {
                let should_alert = self.last_alert_at
                    .map(|t| now - t > 600)
                    .unwrap_or(true);
                if should_alert {
                    self.last_alert_at = Some(now);
                    return true;
                }
            }
        } else {
            self.distraction_start = None;
            self.streak_secs = 0;
        }
        false
    }

    pub fn is_distracted(&self) -> bool {
        self.streak_secs >= self.threshold_secs
    }
}

pub struct MonitorContext {
    pub db_conn: Arc<Mutex<rusqlite::Connection>>,
    pub classifier: Arc<Mutex<ClassifierConfig>>,
    pub app_handle: AppHandle,
    pub current: Option<CurrentSession>,
    pub daily_goal_secs: i64,
    pub distraction: DistractionState,
}

static MONITOR_CTX: std::sync::OnceLock<Arc<Mutex<MonitorContext>>> = std::sync::OnceLock::new();

pub fn get_distraction_state() -> Option<(i64, bool)> {
    let ctx = MONITOR_CTX.get()?.lock().ok()?;
    Some((ctx.distraction.streak_secs, ctx.distraction.is_distracted()))
}

pub fn start_monitoring(
    db_conn: Arc<Mutex<rusqlite::Connection>>,
    classifier: Arc<Mutex<ClassifierConfig>>,
    app_handle: AppHandle,
    daily_goal_secs: i64,
) {
    let ctx = MonitorContext {
        db_conn,
        classifier,
        app_handle,
        current: None,
        daily_goal_secs,
        distraction: DistractionState::new(15), // 15 mins default
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
            info!("WinEventHook registered successfully.");
        }
    }
}

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

    let mut title_buf = [0u16; 512];
    let title_len = GetWindowTextW(hwnd, &mut title_buf) as usize;
    let window_title = OsString::from_wide(&title_buf[..title_len])
        .to_string_lossy()
        .to_string();

    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    let app_name = get_process_name(pid).unwrap_or_else(|| "unknown".to_string());

    if pid == 0 || app_name == "unknown" {
        return;
    }

    let ctx_arc = match MONITOR_CTX.get() {
        Some(c) => c.clone(),
        None => return,
    };
    let mut ctx = match ctx_arc.lock() {
        Ok(c) => c,
        Err(_) => return,
    };

    if let Some(ref prev) = ctx.current.clone() {
        if now > prev.start_time {
            let db = ctx.db_conn.lock().unwrap();
            let _ = update_activity_end(&db, prev.db_id, now);
        }
    }

    let (safe_title, category) = {
        let classifier = match ctx.classifier.lock() {
            Ok(c) => c,
            Err(_) => return,
        };
        let safe_title = if classifier.is_sensitive_app(&app_name) {
            "[protected]".to_string()
        } else {
            window_title
        };
        let category = classifier.classify_app(&app_name);
        (safe_title, category)
    };

    ctx.distraction.update(&category, now);

    let new_activity = Activity {
        id: None,
        app_name: app_name.clone(),
        window_title: safe_title,
        category: category.clone(),
        start_time: now,
        end_time: 0,
    };

    // 先在独立块内完成所有 DB 操作并释放锁，避免与 ctx.current 可变借用冲突 (E0502)
    let (new_id, focus_secs, category_secs) = {
        let db = ctx.db_conn.lock().unwrap();
        let id = insert_activity(&db, &new_activity).unwrap_or(0);
        let secs = query_today_focus_secs(&db).unwrap_or(0);
        let cat_secs = crate::db::query_category_secs_today(&db, &category);
        (id, secs, cat_secs)
    }; // db (MutexGuard) 在此处 drop，锁释放
    ctx.current = Some(CurrentSession {
        db_id: new_id,
        app_name: app_name.clone(),
        category: category.clone(),
        start_time: now,
    });

    let goal_secs = ctx.daily_goal_secs;
    let goal_pct = ((focus_secs as f32 / goal_secs as f32) * 100.0).min(100.0) as u32;

    let overlay_data = OverlayData {
        current_app: app_name,
        category,
        session_secs: 0,
        focus_secs,
        goal_pct,
        category_secs,
        ai_hint: "专注中...".to_string(), // 这里可以接入真正的AI提示
    };

    // 通知前端更新
    let _ = ctx.app_handle.emit("activity-changed", overlay_data);
}

fn get_process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let len = GetProcessImageFileNameW(handle, &mut buf) as usize;
        if len == 0 {
            return None;
        }
        let full_path = OsString::from_wide(&buf[..len])
            .to_string_lossy()
            .to_string();
        full_path.split(['\\', '/']).last().map(|s| s.to_string())
    }
}
