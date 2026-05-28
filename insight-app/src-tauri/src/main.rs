#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod classifier;
mod db;
mod http_server;
mod models;
mod monitor;

use log::info;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};
use windows::Win32::Foundation::{COLORREF, HWND};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, HOT_KEY_MODIFIERS, MOD_CONTROL, MOD_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetLayeredWindowAttributes, SetWindowLongW, GWL_EXSTYLE, LWA_ALPHA, WM_HOTKEY,
    WS_EX_LAYERED,
};

use models::{
    Category, CategoryAppBreakdown, DailyFocus, DashboardData, HourlyStat, ModuleConfig,
    ModuleGoals, ModuleProgress, NoteItem, OverlayData, RecurringTodo, TodoImportResult,
    TodoItem, WebVisit,
};

#[derive(Debug, Serialize)]
struct DistractionStateRes {
    #[serde(rename = "streakSecs")]
    streak_secs: i64,
    #[serde(rename = "isDistracted")]
    is_distracted: bool,
}

#[tauri::command]
fn get_distraction_state() -> Result<DistractionStateRes, String> {
    let state = monitor::get_distraction_state().unwrap_or((0, false));
    Ok(DistractionStateRes {
        streak_secs: state.0,
        is_distracted: state.1,
    })
}

#[tauri::command]
fn correct_activity_category(
    app_name: String,
    new_category: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 1. Update classifier config
    {
        let mut classifier = state.classifier.lock().unwrap();
        classifier.add_app_keyword_to_module(&app_name, &new_category);
        classifier.save(&state.config_path);
    }

    // 2. Update DB
    {
        let db = state.db_conn.lock().unwrap();
        db.execute(
            "UPDATE activities SET category = ?1 WHERE app_name = ?2",
            rusqlite::params![new_category, app_name],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ClearDataOptions {
    activities: bool,
    #[serde(rename = "webHistory")]
    web_history: bool,
    #[serde(rename = "moduleConfig")]
    module_config: bool,
    #[serde(rename = "windowSettings")]
    window_settings: bool,
}

#[tauri::command]
fn clear_data(state: State<'_, AppState>, options: ClearDataOptions) -> Result<(), String> {
    if options.activities {
        let db = state.db_conn.lock().unwrap();
        db::clear_activities(&db).map_err(|e| e.to_string())?;
    }
    if options.web_history {
        let db = state.db_conn.lock().unwrap();
        db::clear_web_history(&db).map_err(|e| e.to_string())?;
    }
    if options.module_config {
        let _ = std::fs::remove_file(&state.config_path);
        let mut classifier = state.classifier.lock().unwrap();
        *classifier = classifier::ClassifierConfig::load(&state.config_path);
        info!("Module config reset to defaults");
    }
    if options.window_settings {
        let window_json = state.data_dir.join("window.json");
        let _ = std::fs::remove_file(window_json);
        let mut ws = state.window_state.lock().unwrap();
        *ws = WindowState::default();
        info!("Window settings reset to defaults");
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WindowState {
    x: i32,
    y: i32,
    opacity: f64,
    daily_goal_secs: i64,
    #[serde(default)]
    module_goals: ModuleGoals,
    #[serde(default)]
    pinned: bool,
    #[serde(default = "default_focus_mins")]
    pomodoro_focus_mins: i32,
    #[serde(default = "default_break_mins")]
    pomodoro_break_mins: i32,
}

fn default_focus_mins() -> i32 {
    25
}
fn default_break_mins() -> i32 {
    5
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 40,
            y: 80,
            opacity: 1.0,
            daily_goal_secs: 14400,
            module_goals: ModuleGoals::default(),
            pinned: false,
            pomodoro_focus_mins: 25,
            pomodoro_break_mins: 5,
        }
    }
}

struct AppState {
    db_conn: Arc<Mutex<rusqlite::Connection>>,
    data_dir: PathBuf,
    config_path: PathBuf,
    window_state: Arc<Mutex<WindowState>>,
    classifier: Arc<Mutex<classifier::ClassifierConfig>>,
}

#[tauri::command]
fn get_overlay_data(state: State<'_, AppState>) -> Result<OverlayData, String> {
    let db = state.db_conn.lock().unwrap();
    let focus_secs = db::query_today_focus_secs(&db).unwrap_or(0);
    let ws = state.window_state.lock().unwrap();
    let goal_pct = ((focus_secs as f32 / ws.daily_goal_secs as f32) * 100.0).min(100.0) as u32;

    Ok(OverlayData {
        current_app: "InsightFlow".to_string(),
        category: Category::Dev.as_str().to_string(),
        session_secs: 0,
        focus_secs,
        goal_pct,
        category_secs: 0,
        ai_hint: String::new(),
    })
}

#[tauri::command]
fn get_dashboard_data(state: State<'_, AppState>) -> Result<DashboardData, String> {
    let db = state.db_conn.lock().unwrap();
    let activities = db::query_today_activities(&db).unwrap_or_default();
    let category_stats = db::query_today_category_stats(&db).unwrap_or_default();
    let total_secs: i64 = category_stats.iter().map(|s| s.total_secs).sum();

    Ok(DashboardData {
        activities,
        category_stats,
        total_secs,
    })
}

#[tauri::command]
fn get_web_history(state: State<'_, AppState>) -> Result<Vec<WebVisit>, String> {
    let db = state.db_conn.lock().unwrap();
    db::query_today_web_history(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_dashboard_data_range(
    state: State<'_, AppState>,
    start_ts: i64,
    end_ts: i64,
) -> Result<DashboardData, String> {
    let db = state.db_conn.lock().unwrap();
    let activities = db::query_activities_range(&db, start_ts, end_ts).unwrap_or_default();
    let category_stats = db::query_category_stats_range(&db, start_ts, end_ts).unwrap_or_default();
    let total_secs: i64 = category_stats.iter().map(|s| s.total_secs).sum();
    Ok(DashboardData {
        activities,
        category_stats,
        total_secs,
    })
}

#[tauri::command]
fn get_category_app_breakdown(
    state: State<'_, AppState>,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<CategoryAppBreakdown>, String> {
    let db = state.db_conn.lock().unwrap();
    db::query_category_app_breakdown(&db, start_ts, end_ts).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_hourly_distribution(
    state: State<'_, AppState>,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<HourlyStat>, String> {
    let db = state.db_conn.lock().unwrap();
    db::query_hourly_distribution(&db, start_ts, end_ts).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_weekly_focus_series(state: State<'_, AppState>) -> Result<Vec<DailyFocus>, String> {
    let db = state.db_conn.lock().unwrap();
    db::query_daily_focus_series(&db, 7).map_err(|e| e.to_string())
}

#[tauri::command]
fn show_dashboard(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("dashboard") {
        let is_visible = win.is_visible().unwrap_or(false);
        let is_minimized = win.is_minimized().unwrap_or(false);

        if is_visible && !is_minimized {
            win.hide().map_err(|e| e.to_string())?;
        } else {
            if is_minimized {
                win.unminimize().unwrap_or(());
            }
            win.show().map_err(|e| e.to_string())?;
            win.set_focus().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn load_window_state(data_dir: &PathBuf) -> WindowState {
    let path = data_dir.join("window.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<WindowState>(&content) {
                return state;
            }
        }
    }
    WindowState::default()
}

fn save_window_state(data_dir: &PathBuf, state: &WindowState) {
    let path = data_dir.join("window.json");
    if let Ok(content) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, content);
    }
}

#[tauri::command]
fn get_pomodoro_settings(state: State<'_, AppState>) -> Result<(i32, i32), String> {
    let ws = state.window_state.lock().unwrap();
    Ok((ws.pomodoro_focus_mins, ws.pomodoro_break_mins))
}

#[tauri::command]
fn set_pomodoro_settings(
    state: State<'_, AppState>,
    focus_mins: i32,
    break_mins: i32,
) -> Result<(), String> {
    let mut ws = state.window_state.lock().unwrap();
    ws.pomodoro_focus_mins = focus_mins;
    ws.pomodoro_break_mins = break_mins;
    save_window_state(&state.data_dir, &ws);
    Ok(())
}

// ──────────────────────────────────────────────
// Todos
// ──────────────────────────────────────────────

#[tauri::command]
fn list_todos(state: State<'_, AppState>) -> Result<Vec<TodoItem>, String> {
    let db = state.db_conn.lock().unwrap();
    db::list_todos(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_todo(app: tauri::AppHandle, state: State<'_, AppState>, text: String, due_date: Option<i64>, target_date: Option<i64>) -> Result<TodoItem, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Empty todo".to_string());
    }
    let db = state.db_conn.lock().unwrap();
    let item = db::create_todo(&db, &text, false, "manual", None, due_date, target_date).map_err(|e| e.to_string())?;
    let _ = app.emit("todos-changed", ());
    Ok(item)
}

#[tauri::command]
fn toggle_todo(app: tauri::AppHandle, state: State<'_, AppState>, id: i64, done: bool) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::set_todo_done(&db, id, done).map_err(|e| e.to_string())?;
    let _ = app.emit("todos-changed", ());
    Ok(())
}

#[tauri::command]
fn update_todo(state: State<'_, AppState>, id: i64, text: String) -> Result<(), String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Empty todo".to_string());
    }
    let db = state.db_conn.lock().unwrap();
    db::update_todo_text(&db, id, &text).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_todo_due_date(state: State<'_, AppState>, id: i64, due_date: Option<i64>) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::update_todo_due_date(&db, id, due_date).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_todo_target_date(state: State<'_, AppState>, id: i64, target_date: Option<i64>) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::update_todo_target_date(&db, id, target_date).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_todo(app: tauri::AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::delete_todo(&db, id).map_err(|e| e.to_string())?;
    let _ = app.emit("todos-changed", ());
    Ok(())
}

#[tauri::command]
fn reorder_todos(state: State<'_, AppState>, ids_in_order: Vec<i64>) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::reorder_todos(&db, &ids_in_order).map_err(|e| e.to_string())
}

#[tauri::command]
fn rollover_todos(state: State<'_, AppState>) -> Result<usize, String> {
    let db = state.db_conn.lock().unwrap();
    db::rollover_todos(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn generate_recurring(state: State<'_, AppState>) -> Result<usize, String> {
    let db = state.db_conn.lock().unwrap();
    db::generate_recurring_todos(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_recurring_todos(state: State<'_, AppState>) -> Result<Vec<RecurringTodo>, String> {
    let db = state.db_conn.lock().unwrap();
    db::list_recurring_todos(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_recurring_todo(
    state: State<'_, AppState>,
    text: String,
    repeat_type: String,
    weekdays: Option<String>,
    start_date: Option<i64>,
    end_date: Option<i64>,
    custom_dates: Option<String>,
) -> Result<RecurringTodo, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Empty recurring todo".to_string());
    }
    let db = state.db_conn.lock().unwrap();
    db::create_recurring_todo(
        &db,
        &text,
        &repeat_type,
        weekdays.as_deref(),
        start_date,
        end_date,
        custom_dates.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_recurring_todo(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::delete_recurring_todo(&db, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_recurring_todo(state: State<'_, AppState>, id: i64, active: bool) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::toggle_recurring_todo(&db, id, active).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_todos_markdown(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    markdown: String,
) -> Result<TodoImportResult, String> {
    let mut imported = 0usize;
    let mut ignored = 0usize;

    let group_id = chrono::Utc::now().timestamp_millis().to_string();
    let re = regex::Regex::new(r"^\s*[-*+]\s+\[( |x|X)\]\s+(.*)$").map_err(|e| e.to_string())?;

    let db = state.db_conn.lock().unwrap();
    for line in markdown.lines() {
        let Some(cap) = re.captures(line) else {
            ignored += 1;
            continue;
        };
        let done_mark = cap.get(1).map(|m| m.as_str()).unwrap_or(" ");
        let text = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        if text.is_empty() {
            ignored += 1;
            continue;
        }
        let done = done_mark.eq_ignore_ascii_case("x");
        let _ = db::create_todo(&db, text, done, "markdown", Some(&group_id), None, None);
        imported += 1;
    }

    let _ = app.emit("todos-changed", ());
    Ok(TodoImportResult { imported, ignored })
}

// ──────────────────────────────────────────────
// Notes
// ──────────────────────────────────────────────

#[tauri::command]
fn list_notes(state: State<'_, AppState>) -> Result<Vec<NoteItem>, String> {
    let db = state.db_conn.lock().unwrap();
    db::list_notes(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_note(state: State<'_, AppState>, id: i64) -> Result<NoteItem, String> {
    let db = state.db_conn.lock().unwrap();
    db::get_note(&db, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_note(
    state: State<'_, AppState>,
    title: String,
    content: String,
    color: String,
    note_type: Option<String>,
) -> Result<NoteItem, String> {
    let db = state.db_conn.lock().unwrap();
    let color = if color.trim().is_empty() {
        "#8a8278".to_string()
    } else {
        color
    };
    let nt = note_type.unwrap_or_else(|| "markdown".to_string());
    db::create_note(&db, title.trim(), content.as_str(), color.as_str(), &nt)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_note(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: i64,
    title: String,
    content: String,
    color: String,
    note_type: Option<String>,
    checklist_items: Option<String>,
) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    let color = if color.trim().is_empty() {
        "#8a8278".to_string()
    } else {
        color
    };
    let nt = note_type.unwrap_or_else(|| "markdown".to_string());
    let ci = checklist_items.unwrap_or_else(|| "[]".to_string());
    db::update_note(&db, id, title.trim(), content.as_str(), color.as_str(), &nt, &ci)
        .map_err(|e| e.to_string())?;
        
    let _ = app.emit("note-updated", id);
    Ok(())
}

#[tauri::command]
fn pin_note(app: tauri::AppHandle, state: State<'_, AppState>, id: i64, pinned: bool) -> Result<(), String> {
    {
        let db = state.db_conn.lock().unwrap();
        db::set_note_pinned(&db, id, pinned).map_err(|e| e.to_string())?;
    }
    
    let _ = app.emit("note-updated", id);
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn delete_note(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::delete_note(&db, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_trashed_notes(state: State<'_, AppState>) -> Result<Vec<NoteItem>, String> {
    let db = state.db_conn.lock().unwrap();
    db::list_trashed_notes(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn trash_note(app: tauri::AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    {
        let db = state.db_conn.lock().unwrap();
        db::trash_note(&db, id).map_err(|e| e.to_string())?;
    }
    
    // 关闭对应的独立悬浮窗（如果打开着）
    use tauri::Manager;
    let label = format!("note_{}", id);
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.close();
    }
    
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn restore_note(app: tauri::AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    {
        let db = state.db_conn.lock().unwrap();
        db::restore_note(&db, id).map_err(|e| e.to_string())?;
    }
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn purge_note(app: tauri::AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    {
        let db = state.db_conn.lock().unwrap();
        db::purge_note(&db, id).map_err(|e| e.to_string())?;
    }
    
    use tauri::Manager;
    let label = format!("note_{}", id);
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.close();
    }
    
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn empty_trash(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    {
        let db = state.db_conn.lock().unwrap();
        db::empty_trash(&db).map_err(|e| e.to_string())?;
    }
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn reorder_notes(state: State<'_, AppState>, ids_in_order: Vec<i64>) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::reorder_notes(&db, &ids_in_order).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_note_geometry(
    state: State<'_, AppState>,
    id: i64,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    let db = state.db_conn.lock().unwrap();
    db::update_note_geometry(&db, id, x, y, width, height).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_note_window(app: tauri::AppHandle, note_id: i64) -> Result<(), String> {
    use tauri::{WebviewUrl, Emitter};
    let label = format!("note_{}", note_id);

    // If window already exists, just show and focus it
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }

    let url = WebviewUrl::App(format!("note.html?id={}", note_id).parse().unwrap());
    let win = tauri::WebviewWindowBuilder::new(&app, &label, url)
        .title("InsightFlow Note")
        .inner_size(300.0, 320.0)
        .min_inner_size(240.0, 200.0)
        .decorations(false) // 恢复无边框
        .transparent(true)  // 恢复透明
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .shadow(false) // 关键修复：关闭阴影避免 Win11 渲染卡死和偏移
        .build()
        .map_err(|e| e.to_string())?;

    // Emit note ID to the new window after it has time to load and register listeners
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(600));
        let _ = win.emit("load-note-id", note_id);
    });

    Ok(())
}

// ──────────────────────────────────────────────
// Note Tags
// ──────────────────────────────────────────────

#[tauri::command]
fn list_tags_for_note(state: State<'_, AppState>, note_id: i64) -> Result<Vec<String>, String> {
    let db = state.db_conn.lock().unwrap();
    db::list_tags_for_note(&db, note_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_all_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = state.db_conn.lock().unwrap();
    db::list_all_tags(&db).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_note_tag(app: tauri::AppHandle, state: State<'_, AppState>, note_id: i64, tag: String) -> Result<(), String> {
    {
        let db = state.db_conn.lock().unwrap();
        db::add_note_tag(&db, note_id, &tag).map_err(|e| e.to_string())?;
    }
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn remove_note_tag(app: tauri::AppHandle, state: State<'_, AppState>, note_id: i64, tag: String) -> Result<(), String> {
    {
        let db = state.db_conn.lock().unwrap();
        db::remove_note_tag(&db, note_id, &tag).map_err(|e| e.to_string())?;
    }
    let _ = app.emit("notes-changed", ());
    Ok(())
}

#[tauri::command]
fn list_notes_by_tag(state: State<'_, AppState>, tag: String) -> Result<Vec<NoteItem>, String> {
    let db = state.db_conn.lock().unwrap();
    db::list_notes_by_tag(&db, &tag).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_window_position(state: State<'_, AppState>, x: i32, y: i32) -> Result<(), String> {
    let mut ws = state.window_state.lock().unwrap();
    ws.x = x;
    ws.y = y;
    save_window_state(&state.data_dir, &ws);
    Ok(())
}

#[tauri::command]
fn get_opacity(state: State<'_, AppState>) -> Result<f64, String> {
    let ws = state.window_state.lock().unwrap();
    Ok(ws.opacity)
}

#[tauri::command]
fn set_opacity(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    opacity: f64,
) -> Result<(), String> {
    let opacity = opacity.clamp(0.1, 1.0);
    {
        let mut ws = state.window_state.lock().unwrap();
        ws.opacity = opacity;
        save_window_state(&state.data_dir, &ws);
    }
    if let Some(win) = app.get_webview_window("overlay") {
        if let Ok(hwnd) = win.hwnd() {
            set_window_opacity(HWND(hwnd.0 as _), opacity);
        }
    }
    Ok(())
}

#[tauri::command]
fn get_pinned(state: State<'_, AppState>) -> Result<bool, String> {
    let ws = state.window_state.lock().unwrap();
    Ok(ws.pinned)
}

#[tauri::command]
fn set_pinned(state: State<'_, AppState>, pinned: bool) -> Result<(), String> {
    let mut ws = state.window_state.lock().unwrap();
    ws.pinned = pinned;
    save_window_state(&state.data_dir, &ws);
    Ok(())
}

#[tauri::command]
fn get_daily_goal(state: State<'_, AppState>) -> Result<i64, String> {
    let ws = state.window_state.lock().unwrap();
    Ok(ws.daily_goal_secs)
}

#[tauri::command]
fn set_daily_goal(state: State<'_, AppState>, goal_secs: i64) -> Result<(), String> {
    let mut ws = state.window_state.lock().unwrap();
    ws.daily_goal_secs = goal_secs.max(600);
    save_window_state(&state.data_dir, &ws);
    Ok(())
}

#[tauri::command]
fn get_module_goals(state: State<'_, AppState>) -> Result<ModuleGoals, String> {
    let ws = state.window_state.lock().unwrap();
    Ok(ws.module_goals.clone())
}

#[tauri::command]
fn set_module_goal(
    state: State<'_, AppState>,
    category: String,
    goal_secs: i64,
) -> Result<(), String> {
    let mut ws = state.window_state.lock().unwrap();
    ws.module_goals.set(&category, goal_secs.max(0));
    save_window_state(&state.data_dir, &ws);
    Ok(())
}

#[tauri::command]
fn get_module_progress(state: State<'_, AppState>) -> Result<Vec<ModuleProgress>, String> {
    let db = state.db_conn.lock().unwrap();
    let ws = state.window_state.lock().unwrap();
    let modules = state.classifier.lock().unwrap().modules.clone();
    let module_ids: Vec<String> = modules.into_iter().map(|m| m.id).collect();
    db::query_module_progress(&db, &module_ids, &ws.module_goals).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_modules(state: State<'_, AppState>) -> Result<Vec<ModuleConfig>, String> {
    let classifier = state.classifier.lock().unwrap();
    Ok(classifier.modules.clone())
}

#[tauri::command]
fn save_modules(state: State<'_, AppState>, modules: Vec<ModuleConfig>) -> Result<(), String> {
    let normalized = normalize_modules(modules);
    let mut classifier = state.classifier.lock().unwrap();
    classifier.modules = normalized;
    classifier.save(&state.config_path);
    Ok(())
}

#[tauri::command]
fn resize_overlay(app: tauri::AppHandle, height: u32) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("overlay") {
        win.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 320.0,
            height: height as f64,
        }))
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_autostart() -> Result<bool, String> {
    Ok(autostart::is_autostart_enabled())
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    if enabled {
        autostart::enable_autostart()
    } else {
        autostart::disable_autostart()
    }
}

#[tauri::command]
fn get_locale(state: State<'_, AppState>) -> Result<String, String> {
    let locale_path = state.data_dir.join("locale.txt");
    if locale_path.exists() {
        std::fs::read_to_string(&locale_path)
            .map(|s| s.trim().to_string())
            .map_err(|e| e.to_string())
    } else {
        Ok("zh-CN".to_string())
    }
}

#[tauri::command]
fn set_locale(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    locale: String,
) -> Result<(), String> {
    if locale != "zh-CN" && locale != "en" {
        return Err(format!("Unsupported locale: {locale}"));
    }
    let locale_path = state.data_dir.join("locale.txt");
    std::fs::write(&locale_path, &locale).map_err(|e| e.to_string())?;
    info!("Locale set to: {locale}");
    let _ = app.emit("locale-changed", &locale);
    Ok(())
}

#[tauri::command]
fn get_theme(state: State<'_, AppState>) -> Result<String, String> {
    let theme_path = state.data_dir.join("theme.txt");
    if theme_path.exists() {
        std::fs::read_to_string(&theme_path)
            .map(|s| s.trim().to_string())
            .map_err(|e| e.to_string())
    } else {
        Ok("day".to_string())
    }
}

#[tauri::command]
fn set_theme(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    theme: String,
) -> Result<(), String> {
    if theme != "day" && theme != "night" {
        return Err(format!("Unsupported theme: {theme}"));
    }
    let theme_path = state.data_dir.join("theme.txt");
    std::fs::write(&theme_path, &theme).map_err(|e| e.to_string())?;
    info!("Theme set to: {theme}");
    let _ = app.emit("theme-changed", &theme);
    Ok(())
}

#[allow(dead_code)]
struct TrayState(tauri::tray::TrayIcon<tauri::Wry>);

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let icon = app.default_window_icon().expect("No window icon").clone();
    let show = MenuItem::with_id(app, "tray_show", "显示悬浮窗", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "tray_hide", "隐藏悬浮窗", true, None::<&str>)?;
    let open_dash = MenuItem::with_id(app, "tray_dashboard", "历史面板", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "tray_quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &open_dash, &quit_item])?;

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("InsightFlow")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "tray_show" => {
                if let Some(win) = app.get_webview_window("overlay") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "tray_hide" => {
                if let Some(win) = app.get_webview_window("overlay") {
                    let _ = win.hide();
                }
            }
            "tray_dashboard" => {
                if let Some(win) = app.get_webview_window("dashboard") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "tray_quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    app.manage(TrayState(tray));
    Ok(())
}

fn get_data_dir() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(appdata).join("InsightFlow")
}

fn normalize_modules(mut modules: Vec<ModuleConfig>) -> Vec<ModuleConfig> {
    use std::collections::HashSet;

    let mut used: HashSet<String> = HashSet::new();
    for module in &mut modules {
        let raw_id = module.id.trim().to_string();
        let base = if raw_id.is_empty() {
            module.name.trim().to_lowercase().replace(' ', "_")
        } else {
            raw_id.to_lowercase().replace(' ', "_")
        };
        let mut id = if base.is_empty() {
            "module".to_string()
        } else {
            base
        };
        if used.contains(&id) {
            let mut i = 2;
            let base_id = id.clone();
            while used.contains(&format!("{base_id}_{i}")) {
                i += 1;
            }
            id = format!("{base_id}_{i}");
        }
        used.insert(id.clone());
        module.id = id;
        module.name = module.name.trim().to_string();
        module.color = module.color.trim().to_string();
        module.app_keywords = module
            .app_keywords
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        module.site_domains = module
            .site_domains
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    modules
}

/// Set window opacity via Win32 layered window API (0.0 = transparent, 1.0 = opaque)
fn set_window_opacity(hwnd: HWND, opacity: f64) {
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
    unsafe {
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as i32);
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
    }
}

fn main() {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    #[cfg(not(debug_assertions))]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    let db_path = data_dir.join("data.db");
    let config_path = data_dir.join("config.toml");

    let conn = db::init_db(&db_path).expect("Failed to init database");
    let db_conn = Arc::new(Mutex::new(conn));
    let classifier = Arc::new(Mutex::new(classifier::ClassifierConfig::load(&config_path)));
    let window_state = Arc::new(Mutex::new(load_window_state(&data_dir)));

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .manage(AppState {
            db_conn: db_conn.clone(),
            data_dir: data_dir.clone(),
            config_path: config_path.clone(),
            window_state: window_state.clone(),
            classifier: classifier.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            get_overlay_data,
            get_dashboard_data,
            get_dashboard_data_range,
            get_category_app_breakdown,
            get_hourly_distribution,
            get_weekly_focus_series,
            get_web_history,
            show_dashboard,
            get_locale,
            set_locale,
            get_theme,
            set_theme,
            save_window_position,
            get_opacity,
            set_opacity,
            get_pinned,
            set_pinned,
            get_daily_goal,
            set_daily_goal,
            get_module_goals,
            set_module_goal,
            get_module_progress,
            get_modules,
            save_modules,
            get_autostart,
            set_autostart,
            resize_overlay,
            get_distraction_state,
            correct_activity_category,
            clear_data,
            get_pomodoro_settings,
            set_pomodoro_settings,
            list_todos,
            add_todo,
            toggle_todo,
            update_todo,
            delete_todo,
            reorder_todos,
            import_todos_markdown,
            set_todo_due_date,
            set_todo_target_date,
            rollover_todos,
            generate_recurring,
            list_recurring_todos,
            add_recurring_todo,
            delete_recurring_todo,
            toggle_recurring_todo,
            list_notes,
            get_note,
            create_note,
            update_note,
            pin_note,
            delete_note,
            list_trashed_notes,
            trash_note,
            restore_note,
            purge_note,
            empty_trash,
            reorder_notes,
            update_note_geometry,
            open_note_window,
            list_tags_for_note,
            list_all_tags,
            add_note_tag,
            remove_note_tag,
            list_notes_by_tag
        ])
        .setup(move |app| {
            setup_tray(app)?;

            // ── Dashboard：关闭时隐藏而非销毁，保证 show_dashboard 始终可用 ──
            if let Some(dash) = app.get_webview_window("dashboard") {
                let dash_hide = dash.clone();
                dash.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = dash_hide.hide();
                    }
                });
            }

            let overlay_window = app.get_webview_window("overlay").unwrap();

            // 恢复窗口位置和透明度
            let ws = window_state.lock().unwrap();
            let _ =
                overlay_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: ws.x,
                    y: ws.y,
                }));
            if let Ok(hwnd) = overlay_window.hwnd() {
                set_window_opacity(HWND(hwnd.0 as _), ws.opacity);
            }
            drop(ws);

            // 监听窗口移动事件，保存位置
            let data_dir_clone = data_dir.clone();
            let ws_clone = window_state.clone();
            overlay_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Moved(pos) = event {
                    let mut ws = ws_clone.lock().unwrap();
                    ws.x = pos.x;
                    ws.y = pos.y;
                    save_window_state(&data_dir_clone, &ws);
                }
            });

            // 启动后台监控
            let app_handle = app.handle().clone();
            let db_conn_clone = db_conn.clone();
            let classifier_clone = classifier.clone();
            let goal_secs = window_state.lock().unwrap().daily_goal_secs;

            // 启动 HTTP 服务器（浏览器插件桥接）
            let db_conn_http = db_conn.clone();
            let classifier_http = classifier.clone();
            std::thread::spawn(move || {
                http_server::start_http_server(db_conn_http, classifier_http, 5678);
            });

            let app_handle_clone = app.handle().clone();
            std::thread::spawn(move || {
                monitor::start_monitoring(
                    db_conn_clone,
                    classifier_clone,
                    app_handle_clone.clone(),
                    goal_secs,
                );

                // 注册全局快捷键 Ctrl+Shift+I 切换点击穿透
                const HOTKEY_TOGGLE_THROUGH: i32 = 1;
                unsafe {
                    let _ =
                        RegisterHotKey(None, HOTKEY_TOGGLE_THROUGH, MOD_CONTROL | MOD_SHIFT, 0x49);
                    // 0x49 = 'I'
                }
                info!("Hotkey Ctrl+Shift+I registered for click-through toggle");

                // 注册全局快捷键 Ctrl+Shift+N 切换便签面板
                const HOTKEY_TOGGLE_NOTES: i32 = 2;
                unsafe {
                    let _ =
                        RegisterHotKey(None, HOTKEY_TOGGLE_NOTES, MOD_CONTROL | MOD_SHIFT, 0x4E);
                    // 0x4E = 'N'
                }
                info!("Hotkey Ctrl+Shift+N registered for notes panel toggle");

                // Windows 消息循环
                unsafe {
                    let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
                    while windows::Win32::UI::WindowsAndMessaging::GetMessageW(&mut msg, None, 0, 0)
                        .as_bool()
                    {
                        if msg.message == WM_HOTKEY
                            && msg.wParam.0 == HOTKEY_TOGGLE_THROUGH as usize
                        {
                            let _ = app_handle_clone.emit("toggle-click-through", ());
                        }
                        if msg.message == WM_HOTKEY
                            && msg.wParam.0 == HOTKEY_TOGGLE_NOTES as usize
                        {
                            let _ = app_handle_clone.emit("toggle-notes-panel", ());
                        }
                        let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                        windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    monitor::cleanup_monitoring();
}
