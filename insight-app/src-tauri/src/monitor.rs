use chrono::Utc;
use log::{error, info};
use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::ProcessStatus::GetProcessImageFileNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowTextW, GetWindowThreadProcessId, EVENT_SYSTEM_FOREGROUND,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

const IDLE_THRESHOLD_SECS: i64 = 300; // 5 分钟
const IDLE_POLL_INTERVAL_MS: u64 = 5000; // 5 秒

/// 今日统计缓存 — 使用 AtomicI64 避免读取时需要获取 MonitorContext 锁
struct DailyCache {
    focus_secs: AtomicI64,
    category_secs: Mutex<HashMap<String, i64>>,
    app_secs: Mutex<HashMap<String, i64>>,
}

static DAILY_CACHE: std::sync::OnceLock<DailyCache> = std::sync::OnceLock::new();

fn get_cache() -> Option<&'static DailyCache> {
    DAILY_CACHE.get()
}

/// 从 DB 加载今日统计数据到缓存（启动时调用一次）
fn init_daily_cache(db_conn: &Arc<Mutex<rusqlite::Connection>>) {
    let db = match db_conn.lock() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to lock DB for cache init: {e}");
            return;
        }
    };

    let focus = crate::db::query_today_focus_secs(&db).unwrap_or(0);
    let cat_stats = crate::db::query_today_category_stats(&db).unwrap_or_default();

    let mut cat_map = HashMap::new();
    for stat in &cat_stats {
        cat_map.insert(stat.category.clone(), stat.total_secs);
    }

    // 从 activities 表加载各 app 的今日累计时长
    let mut app_map = HashMap::new();
    let activities = crate::db::query_today_activities(&db).unwrap_or_default();
    for a in &activities {
        *app_map.entry(a.app_name.clone()).or_insert(0) += a.end_time.unwrap_or(0) - a.start_time;
    }

    drop(db);

    let cache = DailyCache {
        focus_secs: AtomicI64::new(focus),
        category_secs: Mutex::new(cat_map),
        app_secs: Mutex::new(app_map),
    };
    DAILY_CACHE.set(cache).ok();
    info!("Daily cache initialized: focus={focus}s");
}

/// 更新缓存：插入新活动后累加计数器
fn update_cache_after_insert(category: &str, app_name: &str, duration_secs: i64) {
    if let Some(cache) = get_cache() {
        let is_focus = category.parse::<Category>().unwrap_or(Category::Uncategorized).is_focus();
        if is_focus {
            cache.focus_secs.fetch_add(duration_secs, Ordering::Relaxed);
        }
        if let Ok(mut map) = cache.category_secs.lock() {
            *map.entry(category.to_string()).or_insert(0) += duration_secs;
        }
        if let Ok(mut map) = cache.app_secs.lock() {
            *map.entry(app_name.to_string()).or_insert(0) += duration_secs;
        }
    }
}

/// 从缓存读取 focus_secs
fn cached_focus_secs() -> i64 {
    get_cache()
        .map(|c| c.focus_secs.load(Ordering::Relaxed))
        .unwrap_or(0)
}

/// 从缓存读取某分类的累计时长
fn cached_category_secs(category: &str) -> i64 {
    get_cache()
        .and_then(|c| c.category_secs.lock().ok())
        .and_then(|m| m.get(category).copied())
        .unwrap_or(0)
}

/// 从缓存读取某应用的累计时长
fn cached_app_secs(app_name: &str) -> i64 {
    get_cache()
        .and_then(|c| c.app_secs.lock().ok())
        .and_then(|m| m.get(app_name).copied())
        .unwrap_or(0)
}

use crate::classifier::ClassifierConfig;
use crate::db::{insert_ongoing_activity, update_activity_end};
use crate::models::{Activity, Category, CurrentSession, OverlayData};

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
        let is_distraction = category.parse::<Category>().unwrap_or(Category::Uncategorized).is_distraction();

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
static HOOK_HANDLE: Mutex<Option<isize>> = Mutex::new(None);

pub fn get_distraction_state() -> Option<(i64, bool)> {
    let ctx = MONITOR_CTX.get()?.lock().ok()?;
    Some((ctx.distraction.streak_secs, ctx.distraction.is_distracted()))
}

/// 获取最后一次输入事件的系统 tick count（毫秒）
fn get_last_input_tick() -> u32 {
    unsafe {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut info).as_bool() {
            info.dwTime
        } else {
            0
        }
    }
}

/// 空闲检测线程：轮询系统输入状态，在活跃/空闲之间切换
fn start_idle_watcher() {
    std::thread::spawn(move || {
        let mut was_idle = false;
        let mut last_input_tick = get_last_input_tick();
        let mut idle_accumulated_secs: i64 = 0;

        loop {
            std::thread::sleep(Duration::from_millis(IDLE_POLL_INTERVAL_MS));

            let current_tick = get_last_input_tick();
            if current_tick != last_input_tick {
                // 用户有新输入，重置空闲计时
                idle_accumulated_secs = 0;
                last_input_tick = current_tick;
            } else {
                // 无新输入，累加空闲时间
                idle_accumulated_secs += (IDLE_POLL_INTERVAL_MS / 1000) as i64;
            }

            let is_idle = idle_accumulated_secs >= IDLE_THRESHOLD_SECS;

            let ctx_arc = match MONITOR_CTX.get() {
                Some(c) => c.clone(),
                None => continue,
            };
            let mut ctx = match ctx_arc.lock() {
                Ok(c) => c,
                Err(_) => continue,
            };

            if is_idle && !was_idle {
                // ── 活跃 → 空闲 ──
                let now = Utc::now().timestamp();
                info!("User went AFK after {idle_accumulated_secs}s of inactivity");

                // 结束当前活动记录
                if let Some(ref prev) = ctx.current.clone() {
                    if prev.db_id > 0 && now > prev.start_time {
                        let dur = now - prev.start_time;
                        let db = ctx.db_conn.lock().unwrap();
                        let _ = update_activity_end(&db, prev.db_id, now);
                        drop(db);
                        update_cache_after_insert(&prev.category, &prev.app_name, dur);
                    }
                }

                // 插入 afk 记录
                let afk_activity = Activity {
                    id: None,
                    app_name: Category::Afk.as_str().to_string(),
                    window_title: "AFK".to_string(),
                    category: Category::Afk.as_str().to_string(),
                    start_time: now,
                    end_time: None,
                };
                let db = ctx.db_conn.lock().unwrap();
                let afk_id = insert_ongoing_activity(&db, &afk_activity).unwrap_or_else(|e| {
                    error!("Failed to insert AFK record: {e}");
                    -1
                });
                drop(db);

                ctx.current = Some(CurrentSession {
                    db_id: afk_id,
                    app_name: Category::Afk.as_str().to_string(),
                    category: Category::Afk.as_str().to_string(),
                    start_time: now,
                });

                was_idle = true;

            } else if !is_idle && was_idle {
                // ── 空闲 → 活跃 ──
                let now = Utc::now().timestamp();
                info!("User returned from AFK");

                // 结束 afk 记录
                if let Some(ref prev) = ctx.current.clone() {
                    if prev.app_name == Category::Afk.as_str() && prev.db_id > 0 {
                        let db = ctx.db_conn.lock().unwrap();
                        let _ = update_activity_end(&db, prev.db_id, now);
                    }
                }

                ctx.current = None; // 下次 win_event_callback 会创建新记录
                was_idle = false;

                let (focus_secs, goal_pct) = {
                    let f = cached_focus_secs();
                    let goal = ctx.daily_goal_secs;
                    let pct = ((f as f32 / goal as f32) * 100.0).min(100.0) as u32;
                    (f, pct)
                };

                let _ = ctx.app_handle.emit("activity-changed", OverlayData {
                    current_app: "—".to_string(),
                    category: Category::Other.as_str().to_string(),
                    session_secs: 0,
                    focus_secs,
                    goal_pct,
                    category_secs: 0,
                    ai_hint: "back".to_string(),
                });
            }
        }
    });
}

pub fn start_monitoring(
    db_conn: Arc<Mutex<rusqlite::Connection>>,
    classifier: Arc<Mutex<ClassifierConfig>>,
    app_handle: AppHandle,
    daily_goal_secs: i64,
) {
    // 初始化今日统计缓存（从 DB 加载）
    init_daily_cache(&db_conn);

    let ctx = MonitorContext {
        db_conn,
        classifier,
        app_handle,
        current: None,
        daily_goal_secs,
        distraction: DistractionState::new(15), // 15 mins default
    };
    MONITOR_CTX.set(Arc::new(Mutex::new(ctx))).ok();

    // 启动空闲检测线程
    start_idle_watcher();

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
            if let Ok(mut h) = HOOK_HANDLE.lock() {
                *h = Some(hook.0 as isize);
            }
            info!("WinEventHook registered successfully.");
        }
    }
}

/// 注销 WinEventHook，在应用退出时调用
pub fn cleanup_monitoring() {
    if let Ok(mut h) = HOOK_HANDLE.lock() {
        if let Some(ptr) = h.take() {
            unsafe {
                let _ = UnhookWinEvent(HWINEVENTHOOK(ptr as *mut _));
            }
            info!("WinEventHook unregistered.");
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

    {
        let classifier = match ctx.classifier.lock() {
            Ok(c) => c,
            Err(_) => return,
        };
        if classifier.is_ignored_app(&app_name) {
            info!("Ignoring app: {}", app_name);
            return;
        }
    }

    if let Some(ref prev) = ctx.current.clone() {
        // 用户处于空闲状态时，忽略窗口切换事件（由 idle_watcher 管理）
        if prev.app_name == Category::Afk.as_str() {
            return;
        }
        if prev.app_name == app_name {
            // 同一应用的重复事件：从缓存读取，不访问 DB
            let category = prev.category.clone();
            let current_dur = now - prev.start_time;
            let is_focus = category.parse::<Category>().unwrap_or(Category::Uncategorized).is_focus();

            let mut focus_secs = cached_focus_secs();
            if is_focus {
                focus_secs += current_dur;
            }
            let cat_secs = cached_category_secs(&category) + current_dur;
            let past_app_secs = cached_app_secs(&app_name);

            let goal_secs = ctx.daily_goal_secs;
            let goal_pct = ((focus_secs as f32 / goal_secs as f32) * 100.0).min(100.0) as u32;
            let overlay_data = OverlayData {
                current_app: app_name,
                category,
                session_secs: past_app_secs + current_dur,
                focus_secs,
                goal_pct,
                category_secs: cat_secs,
                ai_hint: "focusing".to_string(),
            };
            let _ = ctx.app_handle.emit("activity-changed", overlay_data);
            return;
        }
        // 不同应用：结束上一个活动
        if prev.db_id > 0 && now > prev.start_time {
            let db = ctx.db_conn.lock().unwrap();
            let _ = update_activity_end(&db, prev.db_id, now);
        }
    }

    let (safe_app, safe_title, category) = {
        let classifier = match ctx.classifier.lock() {
            Ok(c) => c,
            Err(_) => return,
        };
        let is_sensitive = classifier.is_sensitive_app(&app_name);
        let category = classifier.classify_app(&app_name);
        let safe_app = if is_sensitive { "[protected]".to_string() } else { app_name.clone() };
        let safe_title = if is_sensitive { "[protected]".to_string() } else { window_title };
        (safe_app, safe_title, category)
    };

    ctx.distraction.update(&category, now);

    let new_activity = Activity {
        id: None,
        app_name: safe_app.clone(),
        window_title: safe_title,
        category: category.clone(),
        start_time: now,
        end_time: None,
    };

    // 更新缓存：上一个活动的时长（用真实 app_name 保证缓存一致性）
    if let Some(ref prev) = ctx.current {
        let prev_dur = now - prev.start_time;
        if prev_dur > 0 {
            update_cache_after_insert(&prev.category, &prev.app_name, prev_dur);
        }
    }

    // 只做写操作，读取全部走缓存
    let new_id = {
        let db = ctx.db_conn.lock().unwrap();
        insert_ongoing_activity(&db, &new_activity).unwrap_or(0)
    };

    ctx.current = Some(CurrentSession {
        db_id: new_id,
        app_name: safe_app.clone(),
        category: category.clone(),
        start_time: now,
    });

    let focus_secs = cached_focus_secs();
    let category_secs = cached_category_secs(&category);
    let app_secs = cached_app_secs(&app_name);
    let goal_secs = ctx.daily_goal_secs;
    let goal_pct = ((focus_secs as f32 / goal_secs as f32) * 100.0).min(100.0) as u32;

    let overlay_data = OverlayData {
        current_app: safe_app,
        category,
        session_secs: app_secs,
        focus_secs,
        goal_pct,
        category_secs,
        ai_hint: "focusing".to_string(),
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
