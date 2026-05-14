use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: Option<i64>,
    pub app_name: String,
    pub window_title: String,
    pub category: String,
    pub start_time: i64,
    pub end_time: i64,
}

/// 发送给前端的悬浮窗数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayData {
    pub current_app: String,
    pub category: String,
    pub session_secs: i64,
    pub focus_secs: i64, // 今日专注总时长（秒）
    pub goal_pct: u32,
    #[serde(default)]
    pub category_secs: i64, // 当前分类今日累计时长
    pub ai_hint: String,
}

impl Default for OverlayData {
    fn default() -> Self {
        Self {
            current_app: "InsightFlow 启动中...".into(),
            category: "other".into(),
            session_secs: 0,
            focus_secs: 0,
            goal_pct: 0,
            category_secs: 0,
            ai_hint: "开始记录你的专注时间 🚀".into(),
        }
    }
}

/// 窗口切换信息（全局 Monitor 使用）
#[derive(Debug, Clone)]
pub struct CurrentSession {
    pub db_id: i64,
    pub app_name: String,
    pub category: String,
    pub start_time: i64,
}

/// 分类统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryStat {
    pub category: String,
    pub total_secs: i64,
}

/// Dashboard 页面数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub activities: Vec<Activity>,
    pub category_stats: Vec<CategoryStat>,
    pub total_secs: i64,
}

/// 浏览器访问记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebVisit {
    pub id: Option<i64>,
    pub domain: String,
    pub url: String,
    pub page_title: String,
    pub visit_count: i32,
    pub last_visit: i64,
    #[serde(default)]
    pub total_duration: i64,
}

/// 浏览器插件推送的请求体
#[derive(Debug, Deserialize)]
pub struct WebVisitRequest {
    pub url: String,
    pub title: Option<String>,
    pub timestamp: Option<i64>,
    pub duration: Option<i64>,
}

/// 每日专注趋势数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyFocus {
    pub date: String, // "YYYY-MM-DD"
    pub focus_secs: i64,
}

/// 每个模块的时间目标（秒，0 表示未设置）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModuleGoals {
    #[serde(default, flatten)]
    pub goals: std::collections::HashMap<String, i64>,
}

impl ModuleGoals {
    pub fn get(&self, category: &str) -> i64 {
        *self.goals.get(category).unwrap_or(&0)
    }
    pub fn set(&mut self, category: &str, goal_secs: i64) {
        self.goals.insert(category.to_string(), goal_secs);
    }
}

/// 自定义模块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleConfig {
    pub id: String,
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub app_keywords: Vec<String>,
    #[serde(default)]
    pub site_domains: Vec<String>,
}

/// 单个应用的使用时长
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUsage {
    pub app_name: String,
    pub duration_secs: i64,
    #[serde(default)]
    pub first_start: i64,
    #[serde(default)]
    pub last_end: i64,
}

/// 某分类模块的今日进度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleProgress {
    pub category: String,
    pub actual_secs: i64,
    pub goal_secs: i64, // 0 = 未设置
    pub goal_pct: u32,  // 0 if no goal
    pub top_apps: Vec<AppUsage>,
}

/// 某分类下各应用的用时明细（用于历史面板下钻）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryAppBreakdown {
    pub category: String,
    pub total_secs: i64,
    pub apps: Vec<AppUsage>, // 按时长降序，AppUsage 已在本文件定义
}

/// 小时分布统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyStat {
    pub hour: u32,        // 0-23
    pub total_secs: i64,
}
