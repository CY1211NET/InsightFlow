mod models;
mod db;
mod classifier;
mod monitor;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::info;
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG};

fn main() {
    // 初始化日志（开发模式输出 INFO，生产模式只输出 ERROR）
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    info!("=== InsightFlow Daemon v0.1.0 starting ===");

    // 确定数据目录（%APPDATA%\InsightFlow\）
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

    let db_path = data_dir.join("data.db");
    let config_path = data_dir.join("config.toml");

    info!("Data directory: {}", data_dir.display());

    // 初始化 SQLite
    let conn = db::init_db(&db_path).expect("Failed to initialize database");
    let db_conn = Arc::new(Mutex::new(conn));

    // 加载分类配置
    let classifier = Arc::new(classifier::ClassifierConfig::load(&config_path));
    info!("Classifier config loaded with {} modules", classifier.rules.len());

    // 专注提醒阈值（分钟），可从配置文件读取
    let distraction_threshold_mins: i64 = 15;

    // 启动 Win32 事件监听（含分心检测）
    monitor::start_monitoring(db_conn.clone(), classifier, distraction_threshold_mins);

    // 启动心跳线程（每 60 秒写入一次心跳）
    let hb_conn = db_conn.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(60));
            let db = hb_conn.lock().unwrap();
            if let Err(e) = db::insert_heartbeat(&db) {
                log::error!("Heartbeat failed: {e}");
            } else {
                info!("Heartbeat written");
            }
        }
    });

    info!("Daemon running. Press Ctrl+C to stop.");

    // Win32 消息循环（必须在主线程运行，否则事件钩子无法触发）
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            // 消息泵：保持事件钩子活跃
        }
    }

    info!("Daemon shutting down.");
}

/// 获取数据目录路径
fn get_data_dir() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(appdata).join("InsightFlow")
}

