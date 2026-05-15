use crate::models::{
    Activity, AppUsage, Category, CategoryAppBreakdown, CategoryStat, DailyFocus, HourlyStat,
    ModuleGoals, ModuleProgress, WebVisit,
};
use log::{error, info, warn};
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;")?;

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
    ",
    )?;

    // 迁移：为 web_history 增加 total_duration 列（已有则忽略）
    let _ = conn.execute_batch(
        "ALTER TABLE web_history ADD COLUMN total_duration INTEGER NOT NULL DEFAULT 0;",
    );

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
    let now = chrono::Utc::now().timestamp();

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
