use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 应用/活动分类枚举 — 消灭魔法字符串
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Dev,
    Productivity,
    Entertainment,
    Social,
    Browser,
    Afk,
    Other,
    Uncategorized,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Productivity => "productivity",
            Self::Entertainment => "entertainment",
            Self::Social => "social",
            Self::Browser => "browser",
            Self::Afk => "afk",
            Self::Other => "other",
            Self::Uncategorized => "uncategorized",
        }
    }

    pub fn is_focus(&self) -> bool {
        matches!(self, Self::Dev | Self::Productivity)
    }

    pub fn is_distraction(&self) -> bool {
        matches!(self, Self::Entertainment | Self::Social)
    }

    pub fn is_afk(&self) -> bool {
        matches!(self, Self::Afk)
    }

    /// 所有专注分类的字符串切片，供 SQL IN 查询使用
    pub fn focus_strs() -> [&'static str; 2] {
        [Self::Dev.as_str(), Self::Productivity.as_str()]
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Category {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dev" => Ok(Self::Dev),
            "productivity" => Ok(Self::Productivity),
            "entertainment" => Ok(Self::Entertainment),
            "social" => Ok(Self::Social),
            "browser" => Ok(Self::Browser),
            "afk" => Ok(Self::Afk),
            "other" => Ok(Self::Other),
            "uncategorized" => Ok(Self::Uncategorized),
            _ => Ok(Self::Uncategorized), // 未知分类归为 uncategorized
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: Option<i64>,
    pub app_name: String,
    pub window_title: String,
    pub category: String,
    pub start_time: i64,
    /// None = 活动进行中（SQL NULL），Some = 已结束
    pub end_time: Option<i64>,
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
            category: Category::Other.as_str().into(),
            session_secs: 0,
            focus_secs: 0,
            goal_pct: 0,
            category_secs: 0,
            ai_hint: "initializing".into(),
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
    pub hour: u32, // 0-23
    pub total_secs: i64,
}

/// 待做事项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub id: Option<i64>,
    pub text: String,
    pub done: bool,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub source: String, // manual | markdown
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub due_date: Option<i64>,
    #[serde(default)]
    pub target_date: Option<i64>,
    #[serde(default)]
    pub done_date: Option<i64>,
}

/// Markdown 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoImportResult {
    pub imported: usize,
    pub ignored: usize,
}

/// 重复待做规则
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringTodo {
    pub id: Option<i64>,
    pub text: String,
    pub repeat_type: String, // weekday | range | custom
    #[serde(default)]
    pub weekdays: Option<String>, // "1,3,5"
    #[serde(default)]
    pub start_date: Option<i64>,
    #[serde(default)]
    pub end_date: Option<i64>,
    #[serde(default)]
    pub custom_dates: Option<String>, // "ts1,ts2,ts3"
    pub created_at: i64,
    pub active: bool,
}

/// 便签
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteItem {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub color: String,
    pub pinned: bool,
    pub note_type: String,
    pub checklist_items: String,
    pub sort_order: i64,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub trashed: bool,
    pub trashed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}
