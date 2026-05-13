/// 应用活动记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Activity {
    pub id: Option<i64>,
    pub app_name: String,
    pub window_title: String,
    pub category: String,
    pub start_time: i64,  // Unix timestamp (UTC)
    pub end_time: i64,    // Unix timestamp (UTC)
}

impl Activity {
    pub fn duration_secs(&self) -> i64 {
        self.end_time - self.start_time
    }
}

/// 浏览器访问记录
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebVisit {
    pub id: Option<i64>,
    pub domain: String,
    pub url: String,
    pub page_title: String,
    pub visit_count: i32,
    pub last_visit: i64,
}

/// 每日目标
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DailyGoal {
    pub id: Option<i64>,
    pub goal_date: String,
    pub goal_text: String,
    pub status: GoalStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GoalStatus {
    Pending,
    Achieved,
    Failed,
}

impl std::fmt::Display for GoalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoalStatus::Pending => write!(f, "pending"),
            GoalStatus::Achieved => write!(f, "achieved"),
            GoalStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for GoalStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending"  => Ok(GoalStatus::Pending),
            "achieved" => Ok(GoalStatus::Achieved),
            "failed"   => Ok(GoalStatus::Failed),
            _          => Err(format!("unknown status: {s}")),
        }
    }
}
