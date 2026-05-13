use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use log::warn;

/// 应用分类规则配置（从 config.toml 加载）
#[derive(Debug, Deserialize, Serialize)]
pub struct ClassifierConfig {
    /// 关键词 -> 分类 映射（进程名或窗口标题包含关键词则归类）
    pub rules: HashMap<String, Vec<String>>,
    /// 敏感应用列表（不记录窗口标题）
    pub sensitive_apps: Vec<String>,
    /// 敏感 URL 关键词（不记录完整 URL）
    pub sensitive_url_keywords: Vec<String>,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();

        rules.insert("dev".to_string(), vec![
            "code".to_string(), "cargo".to_string(), "idea".to_string(),
            "rider".to_string(), "clion".to_string(), "vim".to_string(),
            "neovim".to_string(), "terminal".to_string(), "wt.exe".to_string(),
            "powershell".to_string(), "cmd".to_string(), "github".to_string(),
            "gitlab".to_string(), "stackoverflow".to_string(), "rust".to_string(),
        ]);

        rules.insert("entertainment".to_string(), vec![
            "bilibili".to_string(), "youtube".to_string(), "netflix".to_string(),
            "spotify".to_string(), "steam".to_string(), "epicgames".to_string(),
            "vlc".to_string(), "potplayer".to_string(), "leagueoflegends".to_string(),
        ]);

        rules.insert("social".to_string(), vec![
            "wechat".to_string(), "weixin".to_string(), "dingtalk".to_string(),
            "teams".to_string(), "slack".to_string(), "discord".to_string(),
            "telegram".to_string(), "qq".to_string(), "lark".to_string(),
            "feishu".to_string(), "twitter".to_string(), "weibo".to_string(),
        ]);

        rules.insert("productivity".to_string(), vec![
            "word".to_string(), "excel".to_string(), "powerpoint".to_string(),
            "notion".to_string(), "obsidian".to_string(), "typora".to_string(),
            "onenote".to_string(), "todoist".to_string(), "trello".to_string(),
        ]);

        rules.insert("browser".to_string(), vec![
            "chrome".to_string(), "firefox".to_string(), "msedge".to_string(),
            "brave".to_string(), "opera".to_string(),
        ]);

        ClassifierConfig {
            rules,
            sensitive_apps: vec![
                "1password".to_string(), "keepass".to_string(),
                "bitwarden".to_string(), "lastpass".to_string(),
            ],
            sensitive_url_keywords: vec![
                "password".to_string(), "token".to_string(),
                "bank".to_string(), "pay".to_string(), "secret".to_string(),
                "auth".to_string(), "login".to_string(),
            ],
        }
    }
}

impl ClassifierConfig {
    /// 从 TOML 文件加载配置，失败则使用默认配置
    pub fn load(config_path: &Path) -> Self {
        if config_path.exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(cfg) => return cfg,
                    Err(e) => warn!("Failed to parse config: {e}, using defaults"),
                },
                Err(e) => warn!("Failed to read config: {e}, using defaults"),
            }
        } else {
            // 第一次运行：写出默认配置文件供用户自定义
            let default_cfg = ClassifierConfig::default();
            if let Ok(content) = toml::to_string_pretty(&default_cfg) {
                let _ = fs::write(config_path, content);
            }
        }
        ClassifierConfig::default()
    }

    /// 根据进程名和窗口标题分类
    /// 优先级：dev > entertainment > social > productivity > browser > other
    pub fn classify(&self, app_name: &str, window_title: &str) -> String {
        let app_lower = app_name.to_lowercase();
        let title_lower = window_title.to_lowercase();

        // 按固定优先级顺序匹配，确保语义更具体的类别优先
        let priority_order = ["dev", "entertainment", "social", "productivity", "browser"];
        for category in &priority_order {
            if let Some(keywords) = self.rules.get(*category) {
                for kw in keywords {
                    if app_lower.contains(kw.as_str()) || title_lower.contains(kw.as_str()) {
                        return category.to_string();
                    }
                }
            }
        }
        "other".to_string()
    }

    /// 判断应用是否敏感（不记录窗口标题）
    pub fn is_sensitive_app(&self, app_name: &str) -> bool {
        let lower = app_name.to_lowercase();
        self.sensitive_apps.iter().any(|s| lower.contains(s.as_str()))
    }

    /// 判断 URL 是否敏感（不记录完整 URL）
    pub fn is_sensitive_url(&self, url: &str) -> bool {
        let lower = url.to_lowercase();
        self.sensitive_url_keywords.iter().any(|kw| lower.contains(kw.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_dev() {
        let cfg = ClassifierConfig::default();
        assert_eq!(cfg.classify("Code.exe", "main.rs - Visual Studio Code"), "dev");
    }

    #[test]
    fn test_classify_entertainment() {
        let cfg = ClassifierConfig::default();
        assert_eq!(cfg.classify("chrome.exe", "哔哩哔哩 bilibili"), "entertainment");
    }

    #[test]
    fn test_classify_social() {
        let cfg = ClassifierConfig::default();
        assert_eq!(cfg.classify("WeChat.exe", "微信"), "social");
    }

    #[test]
    fn test_classify_unknown() {
        let cfg = ClassifierConfig::default();
        assert_eq!(cfg.classify("unknownapp.exe", "Unknown Window"), "other");
    }

    #[test]
    fn test_sensitive_url() {
        let cfg = ClassifierConfig::default();
        assert!(cfg.is_sensitive_url("https://bank.example.com/login?token=abc123"));
        assert!(!cfg.is_sensitive_url("https://github.com/rust-lang/rust"));
    }
}
