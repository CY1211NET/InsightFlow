use crate::models::{
    Activity, AppUsage, Category, CategoryAppBreakdown, CategoryStat, DailyFocus, HourlyStat,
    ModuleGoals, ModuleProgress, NoteItem, RecurringTodo, TodoItem, WebVisit,
};
use chrono::Datelike;
use log::{error, info, warn};
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;",
    )?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS activities (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            app_name     TEXT    NOT NULL,
            window_title TEXT    NOT NULL DEFAULT '',
            category     TEXT    NOT NULL DEFAULT 'unknown',
            start_time   INTEGER NOT NULL,
            end_time     INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_activities_start ON activities(start_time);
        CREATE INDEX IF NOT EXISTS idx_activities_cat ON activities(start_time, category);
        CREATE INDEX IF NOT EXISTS idx_activities_app ON activities(start_time, app_name);

        CREATE TABLE IF NOT EXISTS web_history (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            domain       TEXT    NOT NULL,
            url          TEXT    NOT NULL,
            page_title   TEXT    NOT NULL DEFAULT '',
            visit_count  INTEGER NOT NULL DEFAULT 1,
            last_visit   INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_web_history_domain ON web_history(domain);

        CREATE TABLE IF NOT EXISTS todos (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            text        TEXT    NOT NULL,
            done        INTEGER NOT NULL DEFAULT 0,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL,
            source      TEXT    NOT NULL DEFAULT 'manual',
            group_id    TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_todos_done_sort ON todos(done, sort_order);

        CREATE TABLE IF NOT EXISTS notes (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            title       TEXT    NOT NULL DEFAULT '',
            content     TEXT    NOT NULL DEFAULT '',
            color       TEXT    NOT NULL DEFAULT '#8a8278',
            pinned      INTEGER NOT NULL DEFAULT 0,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_notes_pinned_updated ON notes(pinned, updated_at);
    ",
    )?;

    // 迁移：为 web_history 增加 total_duration 列（已有则忽略）
    let _ = conn.execute_batch(
        "ALTER TABLE web_history ADD COLUMN total_duration INTEGER NOT NULL DEFAULT 0;",
    );

    // 迁移：为 todos 增加 due_date 列（已有则忽略）
    let _ = conn.execute_batch(
        "ALTER TABLE todos ADD COLUMN due_date INTEGER;",
    );

    // 迁移：为 todos 增加 target_date 和 done_date 列
    let _ = conn.execute_batch(
        "ALTER TABLE todos ADD COLUMN target_date INTEGER;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE todos ADD COLUMN done_date INTEGER;",
    );

    // 迁移：为 notes 增加便签扩展列（已有则忽略）
    let _ = conn.execute_batch(
        "ALTER TABLE notes ADD COLUMN note_type TEXT NOT NULL DEFAULT 'markdown';",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE notes ADD COLUMN checklist_items TEXT NOT NULL DEFAULT '[]';",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE notes ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE notes ADD COLUMN x INTEGER;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE notes ADD COLUMN y INTEGER;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE notes ADD COLUMN width INTEGER;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE notes ADD COLUMN height INTEGER;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE notes ADD COLUMN trashed INTEGER NOT NULL DEFAULT 0;",
    );
    let _ = conn.execute_batch(
        "ALTER TABLE notes ADD COLUMN trashed_at INTEGER;",
    );
    let _ = conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_notes_trashed ON notes(trashed);",
    );

    // 便签标签关联表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS note_tags (
            note_id INTEGER NOT NULL,
            tag     TEXT    NOT NULL,
            UNIQUE(note_id, tag)
        );
        CREATE INDEX IF NOT EXISTS idx_note_tags_note ON note_tags(note_id);
        CREATE INDEX IF NOT EXISTS idx_note_tags_tag ON note_tags(tag);",
    )?;

    // 重复待做规则表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS recurring_todos (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            text         TEXT    NOT NULL,
            repeat_type  TEXT    NOT NULL,
            weekdays     TEXT,
            start_date   INTEGER,
            end_date     INTEGER,
            custom_dates TEXT,
            created_at   INTEGER NOT NULL,
            active       INTEGER NOT NULL DEFAULT 1
        );",
    )?;

    repair_orphan_records(&conn)?;
    migrate_end_time_null(&conn)?;
    info!("Database initialized at: {}", db_path.display());
    Ok(conn)
}

fn repair_orphan_records(conn: &Connection) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    // 1. 修复 end_time < start_time 的异常记录（直接用 now 关闭）
    let repaired_invalid = conn.execute(
        "UPDATE activities SET end_time = ?1 WHERE end_time IS NOT NULL AND end_time < start_time",
        params![now],
    )?;
    // 2. 修复 end_time IS NULL 的孤立记录（程序崩溃/强制退出遗留）
    // 将时长限制在 7200 秒（2小时）以内，避免虚高数据
    let repaired_null = conn.execute(
        "UPDATE activities SET end_time = MIN(start_time + 7200, ?1) WHERE end_time IS NULL",
        params![now],
    )?;
    if repaired_invalid > 0 {
        warn!("Repaired {repaired_invalid} invalid activity records (end < start)");
    }
    if repaired_null > 0 {
        warn!("Repaired {repaired_null} dangling NULL end_time records (capped at 2h)");
    }
    Ok(())
}

/// 迁移：end_time=0 → end_time=NULL（进行中活动），同时移除 NOT NULL 约束
fn migrate_end_time_null(conn: &Connection) -> Result<()> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version >= 1 {
        return Ok(());
    }

    // 将 end_time=0 改为 NULL
    let converted = conn.execute(
        "UPDATE activities SET end_time = NULL WHERE end_time = 0",
        [],
    )?;

    // 重建表以移除 end_time 的 NOT NULL 约束
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _activities_new (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            app_name     TEXT    NOT NULL,
            window_title TEXT    NOT NULL DEFAULT '',
            category     TEXT    NOT NULL DEFAULT 'unknown',
            start_time   INTEGER NOT NULL,
            end_time     INTEGER
        );
        INSERT INTO _activities_new SELECT * FROM activities;
        DROP TABLE activities;
        ALTER TABLE _activities_new RENAME TO activities;
        CREATE INDEX IF NOT EXISTS idx_activities_start ON activities(start_time);
        CREATE INDEX IF NOT EXISTS idx_activities_cat ON activities(start_time, category);
        CREATE INDEX IF NOT EXISTS idx_activities_app ON activities(start_time, app_name);
        PRAGMA user_version = 1;",
    )?;

    if converted > 0 {
        info!("Migrated {converted} end_time=0 records to NULL (v0→v1)");
    }
    Ok(())
}

pub fn insert_activity(conn: &Connection, activity: &Activity) -> Result<i64> {
    conn.execute(
        "INSERT INTO activities (app_name, window_title, category, start_time, end_time) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![activity.app_name, activity.window_title, activity.category, activity.start_time, activity.end_time],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 插入进行中的活动（end_time = NULL）
pub fn insert_ongoing_activity(conn: &Connection, activity: &Activity) -> Result<i64> {
    conn.execute(
        "INSERT INTO activities (app_name, window_title, category, start_time, end_time) VALUES (?1, ?2, ?3, ?4, NULL)",
        params![activity.app_name, activity.window_title, activity.category, activity.start_time],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_activity_end(conn: &Connection, id: i64, end_time: i64) -> Result<()> {
    conn.execute(
        "UPDATE activities SET end_time = ?1 WHERE id = ?2",
        params![end_time, id],
    )?;
    Ok(())
}

pub fn query_today_focus_secs(conn: &Connection) -> Result<i64> {
    use chrono::Local;
    let today_start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();

    // 只统计生产力、开发相关的时间作为"专注"时长
    let focus_categories = Category::focus_strs();

    let mut total_secs = 0;
    for cat in focus_categories {
        let secs: i64 = conn.query_row(
            "SELECT COALESCE(SUM(end_time - start_time), 0) FROM activities WHERE start_time >= ?1 AND category = ?2 AND end_time IS NOT NULL AND end_time > start_time",
            params![today_start, cat],
            |row| row.get(0),
        ).unwrap_or(0);
        total_secs += secs;
    }
    Ok(total_secs)
}

/// 查询今日所有活动（按时长降序）
pub fn query_today_activities(conn: &Connection) -> Result<Vec<Activity>> {
    use chrono::Local;
    let today_start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();

    let mut stmt = conn.prepare(
        "SELECT id, app_name, window_title, category, start_time, end_time
         FROM activities
         WHERE start_time >= ?1 AND end_time IS NOT NULL AND end_time > start_time
         ORDER BY (end_time - start_time) DESC",
    )?;

    let rows = stmt.query_map(params![today_start], |row| {
        Ok(Activity {
            id: Some(row.get(0)?),
            app_name: row.get(1)?,
            window_title: row.get(2)?,
            category: row.get(3)?,
            start_time: row.get(4)?,
            end_time: row.get(5)?,
        })
    })?;

    let mut activities = Vec::new();
    for row in rows {
        match row {
            Ok(a) => activities.push(a),
            Err(e) => error!("Failed to read activity row: {e}"),
        }
    }
    Ok(activities)
}

/// 查询今日各分类累计时长（秒）
pub fn query_today_category_stats(conn: &Connection) -> Result<Vec<CategoryStat>> {
    use chrono::Local;
    let today_start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();

    let mut stmt = conn.prepare(
        "SELECT category, SUM(end_time - start_time) as total_secs
         FROM activities
         WHERE start_time >= ?1 AND end_time IS NOT NULL AND end_time > start_time
         GROUP BY category
         ORDER BY total_secs DESC",
    )?;

    let rows = stmt.query_map(params![today_start], |row| {
        Ok(CategoryStat {
            category: row.get(0)?,
            total_secs: row.get(1)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// 插入或更新浏览器访问记录（按 domain+url 去重，累加停留时长）
pub fn upsert_web_visit(
    conn: &Connection,
    domain: &str,
    url: &str,
    page_title: &str,
    timestamp: i64,
    duration_secs: i64,
) -> Result<i64> {
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM web_history WHERE domain = ?1 AND url = ?2",
            params![domain, url],
            |row| row.get(0),
        )
        .ok();

    match existing {
        Some(id) => {
            conn.execute(
                "UPDATE web_history SET visit_count = visit_count + 1, last_visit = ?1, page_title = ?2, total_duration = total_duration + ?3 WHERE id = ?4",
                params![timestamp, page_title, duration_secs, id],
            )?;
            Ok(id)
        }
        None => {
            conn.execute(
                "INSERT INTO web_history (domain, url, page_title, visit_count, last_visit, total_duration) VALUES (?1, ?2, ?3, 1, ?4, ?5)",
                params![domain, url, page_title, timestamp, duration_secs],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }
}

/// 查询指定时间范围内的活动记录
pub fn query_activities_range(conn: &Connection, start: i64, end: i64) -> Result<Vec<Activity>> {
    let mut stmt = conn.prepare(
        "SELECT id, app_name, window_title, category, start_time, end_time
         FROM activities
         WHERE start_time >= ?1 AND start_time < ?2 AND end_time IS NOT NULL AND end_time > start_time
         ORDER BY (end_time - start_time) DESC",
    )?;
    let rows = stmt.query_map(params![start, end], |row| {
        Ok(Activity {
            id: Some(row.get(0)?),
            app_name: row.get(1)?,
            window_title: row.get(2)?,
            category: row.get(3)?,
            start_time: row.get(4)?,
            end_time: row.get(5)?,
        })
    })?;
    let mut activities = Vec::new();
    for row in rows {
        match row {
            Ok(a) => activities.push(a),
            Err(e) => error!("Failed to read activity row: {e}"),
        }
    }
    Ok(activities)
}

/// 查询指定时间范围内的分类统计
pub fn query_category_stats_range(
    conn: &Connection,
    start: i64,
    end: i64,
) -> Result<Vec<CategoryStat>> {
    let mut stmt = conn.prepare(
        "SELECT category, SUM(end_time - start_time) as total_secs
         FROM activities
         WHERE start_time >= ?1 AND start_time < ?2 AND end_time IS NOT NULL AND end_time > start_time
         GROUP BY category
         ORDER BY total_secs DESC",
    )?;
    let rows = stmt.query_map(params![start, end], |row| {
        Ok(CategoryStat {
            category: row.get(0)?,
            total_secs: row.get(1)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// 查询最近 N 天的每日专注时长（填充无数据的天为 0）
pub fn query_daily_focus_series(conn: &Connection, days: u32) -> Result<Vec<DailyFocus>> {
    use chrono::{Duration, Local};
    use std::collections::HashMap;

    let today = Local::now().date_naive();
    let start_ts = (today - Duration::days(days as i64 - 1))
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| dt.and_local_timezone(Local).earliest())
        .map(|dt| dt.timestamp())
        .unwrap_or_else(|| chrono::Utc::now().timestamp() - days as i64 * 86400);

    let focus_cats = Category::focus_strs();
    let mut stmt = conn.prepare(
        "SELECT date(start_time, 'unixepoch', 'localtime') as d,
                SUM(end_time - start_time) as total_secs
         FROM activities
         WHERE start_time >= ?1
           AND category IN (?2, ?3)
           AND end_time IS NOT NULL AND end_time > start_time
         GROUP BY d
         ORDER BY d ASC",
    )?;

    let raw: HashMap<String, i64> = stmt
        .query_map(params![start_ts, focus_cats[0], focus_cats[1]], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // 填充全部 N 天（无数据的天为 0）
    let series = (0..days)
        .map(|i| {
            let date = today - Duration::days((days - 1 - i) as i64);
            let date_str = date.format("%Y-%m-%d").to_string();
            let focus_secs = raw.get(&date_str).copied().unwrap_or(0);
            DailyFocus {
                date: date_str,
                focus_secs,
            }
        })
        .collect();

    Ok(series)
}

/// 查询今日某分类的累计时长（秒）
#[allow(dead_code)]
pub fn query_category_secs_today(conn: &Connection, category: &str) -> i64 {
    use chrono::Local;
    let today_start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| dt.and_local_timezone(Local).earliest())
        .map(|dt| dt.timestamp())
        .unwrap_or(0);
    conn.query_row(
        "SELECT COALESCE(SUM(end_time - start_time), 0)
         FROM activities
         WHERE start_time >= ?1 AND category = ?2 AND end_time IS NOT NULL AND end_time > start_time",
        rusqlite::params![today_start, category],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
}

/// 查询今日某应用的累计时长（秒）
#[allow(dead_code)]
pub fn query_app_secs_today(conn: &Connection, app_name: &str) -> i64 {
    use chrono::Local;
    let today_start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| dt.and_local_timezone(Local).earliest())
        .map(|dt| dt.timestamp())
        .unwrap_or(0);
    conn.query_row(
        "SELECT COALESCE(SUM(end_time - start_time), 0)
         FROM activities
         WHERE start_time >= ?1 AND app_name = ?2 AND end_time IS NOT NULL AND end_time > start_time",
        rusqlite::params![today_start, app_name],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
}

/// 查询所有分类的今日进度（含 top-3 应用）
pub fn query_module_progress(
    conn: &Connection,
    module_ids: &[String],
    goals: &ModuleGoals,
) -> Result<Vec<ModuleProgress>> {
    use chrono::Local;
    let today_start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| dt.and_local_timezone(Local).earliest())
        .map(|dt| dt.timestamp())
        .unwrap_or(0);

    // 1) 所有分类的总时长
    let mut stmt = conn.prepare(
        "SELECT category, SUM(end_time - start_time) as total
         FROM activities
         WHERE start_time >= ?1 AND end_time IS NOT NULL AND end_time > start_time
         GROUP BY category
         ORDER BY total DESC",
    )?;
    let cat_rows: Vec<(String, i64)> = stmt
        .query_map(rusqlite::params![today_start], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // 确保所有模块都出现（即便今日无记录）
    let mut ordered: Vec<String> = if module_ids.is_empty() {
        cat_rows.iter().map(|(c, _)| c.clone()).collect()
    } else {
        module_ids.to_vec()
    };
    ordered.retain(|c| !c.is_empty());
    ordered.dedup();

    let mut result = Vec::new();

    for cat in ordered {
        let actual_secs = cat_rows
            .iter()
            .find(|(c, _)| c == &cat)
            .map(|(_, s)| *s)
            .unwrap_or(0);
        let goal_secs = goals.get(&cat);
        let goal_pct = if goal_secs > 0 {
            ((actual_secs as f64 / goal_secs as f64) * 100.0).min(100.0) as u32
        } else {
            0
        };

        // 2) top-3 应用
        let mut app_stmt = conn.prepare(
            "SELECT app_name, SUM(end_time - start_time) as dur
             FROM activities
             WHERE start_time >= ?1 AND category = ?2 AND end_time IS NOT NULL AND end_time > start_time
             GROUP BY app_name
             ORDER BY dur DESC
             LIMIT 3",
        )?;
        let top_apps: Vec<AppUsage> = app_stmt
            .query_map(rusqlite::params![today_start, &cat], |row| {
                Ok(AppUsage {
                    app_name: row.get(0)?,
                    duration_secs: row.get(1)?,
                    first_start: 0,
                    last_end: 0,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        result.push(ModuleProgress {
            category: cat.to_string(),
            actual_secs,
            goal_secs,
            goal_pct,
            top_apps,
        });
    }

    Ok(result)
}

/// 查询指定范围内每个分类下各应用的累计使用时长
pub fn query_category_app_breakdown(
    conn: &Connection,
    start: i64,
    end: i64,
) -> Result<Vec<CategoryAppBreakdown>> {
    // 一次查询拿到所有 (category, app_name, duration)，已按 duration DESC 排序
    let mut stmt = conn.prepare(
        "SELECT category, app_name,
                SUM(end_time - start_time) AS dur
         FROM activities
         WHERE start_time >= ?1 AND start_time < ?2 AND end_time IS NOT NULL AND end_time > start_time
         GROUP BY category, app_name
         ORDER BY category, dur DESC",
    )?;

    // 收集为 (category, AppUsage)
    let rows: Vec<(String, AppUsage)> = stmt
        .query_map(rusqlite::params![start, end], |row| {
            Ok((
                row.get::<_, String>(0)?,
                AppUsage {
                    app_name: row.get(1)?,
                    duration_secs: row.get(2)?,
                    first_start: 0,
                    last_end: 0,
                },
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // 按 category 分组
    let mut map: std::collections::HashMap<String, Vec<AppUsage>> =
        std::collections::HashMap::new();
    for (cat, app) in rows {
        map.entry(cat).or_default().push(app);
    }

    // 构建结果，按分类总时长降序
    let mut result: Vec<CategoryAppBreakdown> = map
        .into_iter()
        .map(|(category, apps)| {
            let total_secs = apps.iter().map(|a| a.duration_secs).sum();
            CategoryAppBreakdown {
                category,
                total_secs,
                apps,
            }
        })
        .collect();
    result.sort_by(|a, b| b.total_secs.cmp(&a.total_secs));

    Ok(result)
}

/// 清空所有活动记录
pub fn clear_activities(conn: &Connection) -> Result<usize> {
    let count = conn.execute("DELETE FROM activities", [])?;
    info!("Cleared {count} activity records");
    Ok(count)
}

/// 清空所有网页访问记录
pub fn clear_web_history(conn: &Connection) -> Result<usize> {
    let count = conn.execute("DELETE FROM web_history", [])?;
    info!("Cleared {count} web history records");
    Ok(count)
}

/// 查询指定时间范围内的小时分布（按 0-23 小时分组）
pub fn query_hourly_distribution(
    conn: &Connection,
    start: i64,
    end: i64,
) -> Result<Vec<HourlyStat>> {
    let mut stmt = conn.prepare(
        "SELECT CAST(strftime('%H', start_time, 'unixepoch', 'localtime') AS INTEGER) as hour,
                SUM(end_time - start_time) as total_secs
         FROM activities
         WHERE start_time >= ?1 AND start_time < ?2 AND end_time IS NOT NULL AND end_time > start_time
         GROUP BY hour
         ORDER BY hour ASC",
    )?;

    let rows = stmt.query_map(params![start, end], |row| {
        Ok(HourlyStat {
            hour: row.get(0)?,
            total_secs: row.get(1)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// 查询今日浏览器访问记录（按访问次数降序）
pub fn query_today_web_history(conn: &Connection) -> Result<Vec<WebVisit>> {
    use chrono::Local;
    let today_start = Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();

    let mut stmt = conn.prepare(
        "SELECT id, domain, url, page_title, visit_count, last_visit, total_duration
         FROM web_history
         WHERE last_visit >= ?1
         ORDER BY total_duration DESC",
    )?;

    let rows = stmt.query_map(params![today_start], |row| {
        Ok(WebVisit {
            id: Some(row.get(0)?),
            domain: row.get(1)?,
            url: row.get(2)?,
            page_title: row.get(3)?,
            visit_count: row.get(4)?,
            last_visit: row.get(5)?,
            total_duration: row.get(6)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ──────────────────────────────────────────────
// Todos
// ──────────────────────────────────────────────

fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

/// 返回今天 00:00:00 本地时间的 Unix 时间戳
fn today_start_ts() -> i64 {
    let now = chrono::Local::now();
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(chrono::Local)
        .unwrap()
        .timestamp()
}

pub fn list_todos(conn: &Connection) -> Result<Vec<TodoItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, text, done, sort_order, created_at, updated_at, source, group_id, due_date, target_date, done_date
         FROM todos
         ORDER BY done ASC, sort_order ASC, created_at ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        let done_i: i64 = row.get(2)?;
        Ok(TodoItem {
            id: Some(row.get(0)?),
            text: row.get(1)?,
            done: done_i != 0,
            sort_order: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            source: row.get(6)?,
            group_id: row.get(7)?,
            due_date: row.get(8)?,
            target_date: row.get(9)?,
            done_date: row.get(10)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn create_todo(
    conn: &Connection,
    text: &str,
    done: bool,
    source: &str,
    group_id: Option<&str>,
    due_date: Option<i64>,
    target_date: Option<i64>,
) -> Result<TodoItem> {
    let ts = now_ts();
    let td = target_date.unwrap_or_else(today_start_ts);
    let next_sort: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM todos",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
        + 1;

    conn.execute(
        "INSERT INTO todos (text, done, sort_order, created_at, updated_at, source, group_id, due_date, target_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            text,
            if done { 1 } else { 0 },
            next_sort,
            ts,
            ts,
            source,
            group_id,
            due_date,
            td
        ],
    )?;

    Ok(TodoItem {
        id: Some(conn.last_insert_rowid()),
        text: text.to_string(),
        done,
        sort_order: next_sort,
        created_at: ts,
        updated_at: ts,
        source: source.to_string(),
        group_id: group_id.map(|s| s.to_string()),
        due_date,
        target_date: Some(td),
        done_date: None,
    })
}

pub fn set_todo_done(conn: &Connection, id: i64, done: bool) -> Result<()> {
    let ts = now_ts();
    let done_date: Option<i64> = if done { Some(today_start_ts()) } else { None };
    conn.execute(
        "UPDATE todos SET done = ?1, done_date = ?2, updated_at = ?3 WHERE id = ?4",
        params![if done { 1 } else { 0 }, done_date, ts, id],
    )?;
    Ok(())
}

pub fn update_todo_text(conn: &Connection, id: i64, text: &str) -> Result<()> {
    let ts = now_ts();
    conn.execute(
        "UPDATE todos SET text = ?1, updated_at = ?2 WHERE id = ?3",
        params![text, ts, id],
    )?;
    Ok(())
}

pub fn update_todo_due_date(conn: &Connection, id: i64, due_date: Option<i64>) -> Result<()> {
    let ts = now_ts();
    conn.execute(
        "UPDATE todos SET due_date = ?1, updated_at = ?2 WHERE id = ?3",
        params![due_date, ts, id],
    )?;
    Ok(())
}

pub fn update_todo_target_date(conn: &Connection, id: i64, target_date: Option<i64>) -> Result<()> {
    let ts = now_ts();
    conn.execute(
        "UPDATE todos SET target_date = ?1, updated_at = ?2 WHERE id = ?3",
        params![target_date, ts, id],
    )?;
    Ok(())
}

/// 未完成任务滚动到今天：把 target_date < 今天且未完成的任务 target_date 更新为今天
pub fn rollover_todos(conn: &Connection) -> Result<usize> {
    let today = today_start_ts();
    let count = conn.execute(
        "UPDATE todos SET target_date = ?1, updated_at = ?1 WHERE done = 0 AND target_date < ?1",
        params![today],
    )?;
    if count > 0 {
        info!("Rolled over {count} incomplete todos to today");
    }
    Ok(count)
}

pub fn delete_todo(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM todos WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn reorder_todos(conn: &Connection, ids_in_order: &[i64]) -> Result<()> {
    for (i, id) in ids_in_order.iter().enumerate() {
        conn.execute(
            "UPDATE todos SET sort_order = ?1 WHERE id = ?2",
            params![i as i64 + 1, id],
        )?;
    }
    Ok(())
}

// ──────────────────────────────────────────────
// Recurring Todos
// ──────────────────────────────────────────────

pub fn list_recurring_todos(conn: &Connection) -> Result<Vec<RecurringTodo>> {
    let mut stmt = conn.prepare(
        "SELECT id, text, repeat_type, weekdays, start_date, end_date, custom_dates, created_at, active
         FROM recurring_todos ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let active_i: i64 = row.get(8)?;
        Ok(RecurringTodo {
            id: Some(row.get(0)?),
            text: row.get(1)?,
            repeat_type: row.get(2)?,
            weekdays: row.get(3)?,
            start_date: row.get(4)?,
            end_date: row.get(5)?,
            custom_dates: row.get(6)?,
            created_at: row.get(7)?,
            active: active_i != 0,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn create_recurring_todo(
    conn: &Connection,
    text: &str,
    repeat_type: &str,
    weekdays: Option<&str>,
    start_date: Option<i64>,
    end_date: Option<i64>,
    custom_dates: Option<&str>,
) -> Result<RecurringTodo> {
    let ts = now_ts();
    conn.execute(
        "INSERT INTO recurring_todos (text, repeat_type, weekdays, start_date, end_date, custom_dates, created_at, active)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
        params![text, repeat_type, weekdays, start_date, end_date, custom_dates, ts],
    )?;
    Ok(RecurringTodo {
        id: Some(conn.last_insert_rowid()),
        text: text.to_string(),
        repeat_type: repeat_type.to_string(),
        weekdays: weekdays.map(|s| s.to_string()),
        start_date,
        end_date,
        custom_dates: custom_dates.map(|s| s.to_string()),
        created_at: ts,
        active: true,
    })
}

pub fn delete_recurring_todo(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM recurring_todos WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn toggle_recurring_todo(conn: &Connection, id: i64, active: bool) -> Result<()> {
    conn.execute(
        "UPDATE recurring_todos SET active = ?1 WHERE id = ?2",
        params![if active { 1 } else { 0 }, id],
    )?;
    Ok(())
}

/// 检查重复规则，为今天命中规则的 recurring todo 自动生成普通 todo 实例
pub fn generate_recurring_todos(conn: &Connection) -> Result<usize> {
    let today = today_start_ts();
    // 获取今天是周几（1=周一 ... 7=周日）
    let weekday_today = chrono::Local::now().date_naive().weekday().number_from_monday();
    let today_str = today.to_string();

    let recurring = list_recurring_todos(conn)?;
    let mut generated = 0usize;

    for rt in recurring.iter().filter(|r| r.active) {
        let matches = match rt.repeat_type.as_str() {
            "weekday" => {
                rt.weekdays
                    .as_ref()
                    .map(|w| w.split(',').any(|d| d.trim() == weekday_today.to_string()))
                    .unwrap_or(false)
            }
            "range" => {
                let start = rt.start_date.unwrap_or(0);
                let end = rt.end_date.unwrap_or(i64::MAX);
                today >= start && today <= end
            }
            "custom" => {
                rt.custom_dates
                    .as_ref()
                    .map(|d| d.split(',').any(|ts| ts.trim() == today_str))
                    .unwrap_or(false)
            }
            _ => false,
        };

        if !matches {
            continue;
        }

        // 检查今天是否已生成过
        let group_id = format!("recurring_{}", rt.id.unwrap_or(0));
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM todos WHERE source = 'recurring' AND group_id = ?1 AND target_date = ?2",
                params![group_id, today],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if !exists {
            let _ = create_todo(conn, &rt.text, false, "recurring", Some(&group_id), None, Some(today));
            generated += 1;
        }
    }

    if generated > 0 {
        info!("Generated {generated} recurring todos for today");
    }
    Ok(generated)
}

// ──────────────────────────────────────────────
// Notes
// ──────────────────────────────────────────────

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<NoteItem> {
    let pinned_i: i64 = row.get(4)?;
    let trashed_i: i64 = row.get(12)?;
    Ok(NoteItem {
        id: Some(row.get(0)?),
        title: row.get(1)?,
        content: row.get(2)?,
        color: row.get(3)?,
        pinned: pinned_i != 0,
        note_type: row.get(5)?,
        checklist_items: row.get(6)?,
        sort_order: row.get(7)?,
        x: row.get(8)?,
        y: row.get(9)?,
        width: row.get(10)?,
        height: row.get(11)?,
        trashed: trashed_i != 0,
        trashed_at: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

const NOTE_SELECT_COLS: &str =
    "id, title, content, color, pinned, note_type, checklist_items, sort_order,
     x, y, width, height, trashed, trashed_at, created_at, updated_at";

pub fn get_note(conn: &Connection, id: i64) -> Result<NoteItem> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {NOTE_SELECT_COLS} FROM notes WHERE id = ?1",
    ))?;
    stmt.query_row(params![id], row_to_note)
}

pub fn list_notes(conn: &Connection) -> Result<Vec<NoteItem>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {NOTE_SELECT_COLS} FROM notes WHERE trashed = 0
         ORDER BY pinned DESC, sort_order ASC, updated_at DESC",
    ))?;

    let rows = stmt.query_map([], row_to_note)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_trashed_notes(conn: &Connection) -> Result<Vec<NoteItem>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {NOTE_SELECT_COLS} FROM notes WHERE trashed = 1
         ORDER BY trashed_at DESC",
    ))?;

    let rows = stmt.query_map([], row_to_note)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn create_note(
    conn: &Connection,
    title: &str,
    content: &str,
    color: &str,
    note_type: &str,
) -> Result<NoteItem> {
    let ts = now_ts();
    // sort_order = max + 1
    let max_order: i64 =
        conn.query_row("SELECT COALESCE(MAX(sort_order), 0) FROM notes WHERE trashed = 0", [], |r| r.get(0))
            .unwrap_or(0);

    conn.execute(
        "INSERT INTO notes (title, content, color, pinned, note_type, checklist_items, sort_order, trashed, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, ?4, '[]', ?5, 0, ?6, ?7)",
        params![title, content, color, note_type, max_order + 1, ts, ts],
    )?;

    Ok(NoteItem {
        id: Some(conn.last_insert_rowid()),
        title: title.to_string(),
        content: content.to_string(),
        color: color.to_string(),
        pinned: false,
        note_type: note_type.to_string(),
        checklist_items: "[]".to_string(),
        sort_order: max_order + 1,
        x: None,
        y: None,
        width: None,
        height: None,
        trashed: false,
        trashed_at: None,
        created_at: ts,
        updated_at: ts,
    })
}

pub fn update_note(
    conn: &Connection,
    id: i64,
    title: &str,
    content: &str,
    color: &str,
    note_type: &str,
    checklist_items: &str,
) -> Result<()> {
    let ts = now_ts();
    conn.execute(
        "UPDATE notes SET title = ?1, content = ?2, color = ?3, note_type = ?4, checklist_items = ?5, updated_at = ?6 WHERE id = ?7",
        params![title, content, color, note_type, checklist_items, ts, id],
    )?;
    Ok(())
}

pub fn set_note_pinned(conn: &Connection, id: i64, pinned: bool) -> Result<()> {
    let ts = now_ts();
    conn.execute(
        "UPDATE notes SET pinned = ?1, updated_at = ?2 WHERE id = ?3",
        params![if pinned { 1 } else { 0 }, ts, id],
    )?;
    Ok(())
}

pub fn delete_note(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn trash_note(conn: &Connection, id: i64) -> Result<()> {
    let ts = now_ts();
    conn.execute(
        "UPDATE notes SET trashed = 1, trashed_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![ts, id],
    )?;
    Ok(())
}

pub fn restore_note(conn: &Connection, id: i64) -> Result<()> {
    let ts = now_ts();
    conn.execute(
        "UPDATE notes SET trashed = 0, trashed_at = NULL, updated_at = ?1 WHERE id = ?2",
        params![ts, id],
    )?;
    Ok(())
}

pub fn purge_note(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn empty_trash(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM notes WHERE trashed = 1", [])?;
    Ok(())
}

pub fn reorder_notes(conn: &Connection, ids_in_order: &[i64]) -> Result<()> {
    for (i, id) in ids_in_order.iter().enumerate() {
        conn.execute(
            "UPDATE notes SET sort_order = ?1 WHERE id = ?2",
            params![i as i64, id],
        )?;
    }
    Ok(())
}

pub fn update_note_geometry(
    conn: &Connection,
    id: i64,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<()> {
    let ts = now_ts();
    conn.execute(
        "UPDATE notes SET x = ?1, y = ?2, width = ?3, height = ?4, updated_at = ?5 WHERE id = ?6",
        params![x, y, width, height, ts, id],
    )?;
    Ok(())
}

pub fn list_tags_for_note(conn: &Connection, note_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT tag FROM note_tags WHERE note_id = ?1 ORDER BY tag")?;
    let rows = stmt.query_map(params![note_id], |row| row.get(0))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_all_tags(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT tag FROM note_tags ORDER BY tag")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn add_note_tag(conn: &Connection, note_id: i64, tag: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO note_tags (note_id, tag) VALUES (?1, ?2)",
        params![note_id, tag],
    )?;
    Ok(())
}

pub fn remove_note_tag(conn: &Connection, note_id: i64, tag: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM note_tags WHERE note_id = ?1 AND tag = ?2",
        params![note_id, tag],
    )?;
    Ok(())
}

pub fn list_notes_by_tag(conn: &Connection, tag: &str) -> Result<Vec<NoteItem>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT n.{cols} FROM notes n
         JOIN note_tags t ON n.id = t.note_id
         WHERE t.tag = ?1 AND n.trashed = 0
         ORDER BY n.pinned DESC, n.sort_order ASC, n.updated_at DESC",
        cols = "id, title, content, color, pinned, note_type, checklist_items, sort_order, x, y, width, height, trashed, trashed_at, created_at, updated_at"
    ))?;
    let rows = stmt.query_map(params![tag], row_to_note)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS activities (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                app_name     TEXT    NOT NULL,
                window_title TEXT    NOT NULL DEFAULT '',
                category     TEXT    NOT NULL DEFAULT 'unknown',
                start_time   INTEGER NOT NULL,
                end_time     INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_activities_start ON activities(start_time);
            CREATE INDEX IF NOT EXISTS idx_activities_cat ON activities(start_time, category);
            CREATE INDEX IF NOT EXISTS idx_activities_app ON activities(start_time, app_name);

            CREATE TABLE IF NOT EXISTS web_history (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                domain       TEXT    NOT NULL,
                url          TEXT    NOT NULL,
                page_title   TEXT    NOT NULL DEFAULT '',
                visit_count  INTEGER NOT NULL DEFAULT 1,
                last_visit   INTEGER NOT NULL,
                total_duration INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_web_history_domain ON web_history(domain);
        ",
        )
        .unwrap();
        conn
    }

    fn today_start() -> i64 {
        Local::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp()
    }

    // ── insert_activity ──

    #[test]
    fn insert_and_query_activity() {
        let conn = test_db();
        let now = today_start() + 3600;

        let activity = Activity {
            id: None,
            app_name: "Code.exe".to_string(),
            window_title: "main.rs".to_string(),
            category: "dev".to_string(),
            start_time: now,
            end_time: Some(now + 1800),
        };

        let id = insert_activity(&conn, &activity).unwrap();
        assert!(id > 0);

        let activities = query_today_activities(&conn).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].app_name, "Code.exe");
        assert_eq!(activities[0].category, "dev");
    }

    // ── update_activity_end ──

    #[test]
    fn update_end_time() {
        let conn = test_db();
        let now = today_start() + 1000;

        let activity = Activity {
            id: None,
            app_name: "test.exe".to_string(),
            window_title: "".to_string(),
            category: "other".to_string(),
            start_time: now,
            end_time: None,
        };

        let id = insert_activity(&conn, &activity).unwrap();
        update_activity_end(&conn, id, now + 500).unwrap();

        let activities = query_today_activities(&conn).unwrap();
        assert_eq!(activities[0].end_time, Some(now + 500));
    }

    // ── query_today_focus_secs ──

    #[test]
    fn focus_secs_sums_dev_and_productivity() {
        let conn = test_db();
        let base = today_start() + 100;

        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "code".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base,
                end_time: Some(base + 1000),
            },
        )
        .unwrap();

        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "word".into(),
                window_title: "".into(),
                category: "productivity".into(),
                start_time: base,
                end_time: Some(base + 500),
            },
        )
        .unwrap();

        // entertainment: 不计入专注
        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "bilibili".into(),
                window_title: "".into(),
                category: "entertainment".into(),
                start_time: base,
                end_time: Some(base + 2000),
            },
        )
        .unwrap();

        let focus = query_today_focus_secs(&conn).unwrap();
        assert_eq!(focus, 1500); // 1000 + 500
    }

    // ── query_today_category_stats ──

    #[test]
    fn category_stats_grouped() {
        let conn = test_db();
        let base = today_start() + 100;

        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "a".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base,
                end_time: Some(base + 100),
            },
        )
        .unwrap();
        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "b".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base,
                end_time: Some(base + 200),
            },
        )
        .unwrap();
        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "c".into(),
                window_title: "".into(),
                category: "social".into(),
                start_time: base,
                end_time: Some(base + 50),
            },
        )
        .unwrap();

        let stats = query_today_category_stats(&conn).unwrap();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].category, "dev");
        assert_eq!(stats[0].total_secs, 300);
        assert_eq!(stats[1].category, "social");
        assert_eq!(stats[1].total_secs, 50);
    }

    // ── query_category_secs_today ──

    #[test]
    fn category_secs_today_specific() {
        let conn = test_db();
        let base = today_start() + 100;

        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "a".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base,
                end_time: Some(base + 300),
            },
        )
        .unwrap();

        assert_eq!(query_category_secs_today(&conn, "dev"), 300);
        assert_eq!(query_category_secs_today(&conn, "social"), 0);
    }

    // ── query_app_secs_today ──

    #[test]
    fn app_secs_today_specific() {
        let conn = test_db();
        let base = today_start() + 100;

        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "Code.exe".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base,
                end_time: Some(base + 600),
            },
        )
        .unwrap();
        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "Code.exe".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base + 700,
                end_time: Some(base + 900),
            },
        )
        .unwrap();

        assert_eq!(query_app_secs_today(&conn, "Code.exe"), 800);
        assert_eq!(query_app_secs_today(&conn, "other.exe"), 0);
    }

    // ── upsert_web_visit ──

    #[test]
    fn upsert_inserts_new_visit() {
        let conn = test_db();

        let id = upsert_web_visit(
            &conn,
            "github.com",
            "https://github.com/rust",
            "Rust",
            1000,
            60,
        )
        .unwrap();
        assert!(id > 0);

        let visits = query_today_web_history(&conn).unwrap();
        assert_eq!(visits.len(), 1);
        assert_eq!(visits[0].domain, "github.com");
        assert_eq!(visits[0].visit_count, 1);
        assert_eq!(visits[0].total_duration, 60);
    }

    #[test]
    fn upsert_increments_existing_visit() {
        let conn = test_db();

        upsert_web_visit(
            &conn,
            "github.com",
            "https://github.com/rust",
            "Rust",
            1000,
            60,
        )
        .unwrap();
        upsert_web_visit(
            &conn,
            "github.com",
            "https://github.com/rust",
            "Rust - updated",
            2000,
            30,
        )
        .unwrap();

        let visits = query_today_web_history(&conn).unwrap();
        assert_eq!(visits.len(), 1);
        assert_eq!(visits[0].visit_count, 2);
        assert_eq!(visits[0].total_duration, 90);
        assert_eq!(visits[0].page_title, "Rust - updated");
    }

    // ── query_activities_range ──

    #[test]
    fn activities_range_filters_correctly() {
        let conn = test_db();
        let base = today_start();

        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "in".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base + 1000,
                end_time: Some(base + 2000),
            },
        )
        .unwrap();
        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "out".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base + 5000,
                end_time: Some(base + 6000),
            },
        )
        .unwrap();

        let activities = query_activities_range(&conn, base, base + 3000).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].app_name, "in");
    }

    // ── clear ──

    #[test]
    fn clear_activities_removes_all() {
        let conn = test_db();
        let base = today_start() + 100;

        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "a".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base,
                end_time: Some(base + 100),
            },
        )
        .unwrap();

        let count = clear_activities(&conn).unwrap();
        assert_eq!(count, 1);
        assert_eq!(query_today_activities(&conn).unwrap().len(), 0);
    }

    #[test]
    fn clear_web_history_removes_all() {
        let conn = test_db();
        upsert_web_visit(&conn, "example.com", "https://example.com", "Ex", 1000, 10).unwrap();

        let count = clear_web_history(&conn).unwrap();
        assert_eq!(count, 1);
        assert_eq!(query_today_web_history(&conn).unwrap().len(), 0);
    }

    // ── query_hourly_distribution ──

    #[test]
    fn hourly_distribution_groups_by_hour() {
        let conn = test_db();
        let base = today_start();

        // 2 activities in hour 10
        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "a".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base + 10 * 3600,
                end_time: Some(base + 10 * 3600 + 600),
            },
        )
        .unwrap();
        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "b".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base + 10 * 3600 + 700,
                end_time: Some(base + 10 * 3600 + 1200),
            },
        )
        .unwrap();

        // 1 activity in hour 14
        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "c".into(),
                window_title: "".into(),
                category: "social".into(),
                start_time: base + 14 * 3600,
                end_time: Some(base + 14 * 3600 + 300),
            },
        )
        .unwrap();

        let hourly = query_hourly_distribution(&conn, base, base + 86400).unwrap();
        assert!(hourly.len() >= 2);

        let h10 = hourly.iter().find(|h| h.hour == 10).unwrap();
        assert_eq!(h10.total_secs, 1100); // 600 + 500

        let h14 = hourly.iter().find(|h| h.hour == 14).unwrap();
        assert_eq!(h14.total_secs, 300);
    }

    // ── query_module_progress ──

    #[test]
    fn module_progress_with_goals() {
        use crate::models::ModuleGoals;

        let conn = test_db();
        let base = today_start() + 100;

        insert_activity(
            &conn,
            &Activity {
                id: None,
                app_name: "code".into(),
                window_title: "".into(),
                category: "dev".into(),
                start_time: base,
                end_time: Some(base + 3600),
            },
        )
        .unwrap();

        let mut goals = ModuleGoals::default();
        goals.set("dev", 7200); // 2 hour goal

        let progress = query_module_progress(&conn, &["dev".to_string()], &goals).unwrap();
        assert_eq!(progress.len(), 1);
        assert_eq!(progress[0].category, "dev");
        assert_eq!(progress[0].actual_secs, 3600);
        assert_eq!(progress[0].goal_secs, 7200);
        assert_eq!(progress[0].goal_pct, 50);
    }
}
