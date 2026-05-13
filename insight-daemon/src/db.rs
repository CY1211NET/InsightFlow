use rusqlite::{Connection, Result, params};
use std::path::Path;
use log::{info, error};
use crate::models::Activity;

/// 初始化 SQLite 数据库，创建所有表和索引
pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // 启用 WAL 模式，提升并发读写性能
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;")?;

    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS activities (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            app_name     TEXT    NOT NULL,
            window_title TEXT    NOT NULL DEFAULT '',
            category     TEXT    NOT NULL DEFAULT 'unknown',
            start_time   INTEGER NOT NULL,
            end_time     INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS web_history (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            domain       TEXT    NOT NULL,
            url          TEXT    NOT NULL,
            page_title   TEXT    NOT NULL DEFAULT '',
            visit_count  INTEGER NOT NULL DEFAULT 1,
            last_visit   INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_reports (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            report_date  TEXT    NOT NULL UNIQUE,
            summary      TEXT    NOT NULL,
            suggestions  TEXT    NOT NULL DEFAULT '',
            model_used   TEXT    NOT NULL,
            created_at   INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS daily_goals (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            goal_date    TEXT    NOT NULL UNIQUE,
            goal_text    TEXT    NOT NULL,
            status       TEXT    NOT NULL DEFAULT 'pending'
        );

        CREATE TABLE IF NOT EXISTS heartbeats (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_activities_start
            ON activities(start_time);
        CREATE INDEX IF NOT EXISTS idx_web_history_domain
            ON web_history(domain);
        CREATE INDEX IF NOT EXISTS idx_ai_reports_date
            ON ai_reports(report_date);
    ")?;

    // 修复孤儿记录（上次异常退出未结束的活动）
    repair_orphan_records(&conn)?;

    info!("Database initialized at: {}", db_path.display());
    Ok(conn)
}

/// 修复 end_time=0 的孤儿记录，用当前时间填充
fn repair_orphan_records(conn: &Connection) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    let repaired = conn.execute(
        "UPDATE activities SET end_time = ?1 WHERE end_time = 0 OR end_time < start_time",
        params![now],
    )?;
    if repaired > 0 {
        log::warn!("Repaired {repaired} orphan activity records from last session");
    }
    Ok(())
}

/// 写入活动记录
pub fn insert_activity(conn: &Connection, activity: &Activity) -> Result<i64> {
    conn.execute(
        "INSERT INTO activities (app_name, window_title, category, start_time, end_time)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            activity.app_name,
            activity.window_title,
            activity.category,
            activity.start_time,
            activity.end_time,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 更新活动的结束时间（窗口切换时调用）
pub fn update_activity_end(conn: &Connection, id: i64, end_time: i64) -> Result<()> {
    conn.execute(
        "UPDATE activities SET end_time = ?1 WHERE id = ?2",
        params![end_time, id],
    )?;
    Ok(())
}

/// 写入心跳记录（每 60 秒一次）
pub fn insert_heartbeat(conn: &Connection) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO heartbeats (timestamp) VALUES (?1)",
        params![now],
    )?;
    // 只保留最近 10 条心跳，防止表无限增长
    conn.execute(
        "DELETE FROM heartbeats WHERE id NOT IN (
            SELECT id FROM heartbeats ORDER BY id DESC LIMIT 10
        )",
        [],
    )?;
    Ok(())
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
         WHERE start_time >= ?1
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
pub fn query_today_category_stats(conn: &Connection) -> Result<Vec<(String, i64)>> {
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
         WHERE start_time >= ?1 AND end_time > start_time
         GROUP BY category
         ORDER BY total_secs DESC",
    )?;

    let rows = stmt.query_map(params![today_start], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}
