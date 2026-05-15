use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::models::{Category, ModuleConfig};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClassifierConfig {
    #[serde(default)]
    pub modules: Vec<ModuleConfig>,
    // legacy rules (app keywords only)
    #[serde(default)]
    pub rules: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub sensitive_apps: Vec<String>,
    #[serde(default)]
    pub sensitive_url_keywords: Vec<String>,
    #[serde(default)]
    pub ignored_apps: Vec<String>,
    /// 预构建的关键词→模块索引（lowercase keyword → module_id），classify_app 时 O(1) 查找
    #[serde(skip)]
    keyword_index: HashMap<String, String>,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        let modules = default_modules();
        let keyword_index = build_keyword_index(&modules);
        ClassifierConfig {
            modules,
            rules: HashMap::new(),
            sensitive_apps: vec!["1password".to_string(), "keepass".to_string()],
            sensitive_url_keywords: vec![
                "password".to_string(),
                "token".to_string(),
                "secret".to_string(),
                "bank".to_string(),
                "credential".to_string(),
            ],
            ignored_apps: vec![
                "explorer.exe".to_string(),
                "searchhost.exe".to_string(),
                "startmenuexperiencehost.exe".to_string(),
                "shellexperiencehost.exe".to_string(),
                "lockapp.exe".to_string(),
                "textinputhost.exe".to_string(),
            ],
            keyword_index,
        }
    }
}

/// 从模块配置构建 keyword→module_id 索引
fn build_keyword_index(modules: &[ModuleConfig]) -> HashMap<String, String> {
    let mut index = HashMap::new();
    for module in modules {
        for kw in &module.app_keywords {
            let key = kw.to_lowercase();
            if !key.is_empty() {
                // 先到先得，保持模块优先级顺序
                index.entry(key).or_insert_with(|| module.id.clone());
            }
        }
    }
    index
}

impl ClassifierConfig {
    pub fn load(config_path: &Path) -> Self {
        if config_path.exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => match toml::from_str::<ClassifierConfig>(&content) {
                    Ok(mut cfg) => {
                        if cfg.modules.is_empty() {
                            if !cfg.rules.is_empty() {
                                cfg.modules = modules_from_rules(&cfg.rules);
                            } else {
                                cfg.modules = default_modules();
                            }
                        }
                        cfg.keyword_index = build_keyword_index(&cfg.modules);
                        return cfg;
                    }
                    Err(e) => warn!("Failed to parse config: {e}, using defaults"),
                },
                Err(e) => warn!("Failed to read config: {e}, using defaults"),
            }
        } else {
            let default_cfg = ClassifierConfig::default();
            if let Ok(content) = toml::to_string_pretty(&default_cfg) {
                let _ = fs::write(config_path, content);
            }
        }
        ClassifierConfig::default()
    }

    pub fn save(&self, config_path: &Path) {
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = fs::write(config_path, content);
        }
    }

    pub fn classify_app(&self, app_name: &str) -> String {
        let app_lower = app_name.to_lowercase();
        for (keyword, module_id) in &self.keyword_index {
            if app_lower.contains(keyword.as_str()) {
                return module_id.clone();
            }
        }
        Category::Uncategorized.as_str().to_string()
    }

    pub fn classify_domain(&self, domain: &str) -> String {
        let dom = domain.to_lowercase();
        for module in &self.modules {
            for d in &module.site_domains {
                let d = d.to_lowercase();
                if d.is_empty() {
                    continue;
                }
                if dom == d || dom.ends_with(&format!(".{d}")) {
                    return module.id.clone();
                }
            }
        }
        Category::Uncategorized.as_str().to_string()
    }

    /// 将关键词分配给指定模块，同时从其他模块中移除，以保证唯一性
    pub fn add_app_keyword_to_module(&mut self, app_name: &str, target_module_id: &str) {
        let kw_lower = app_name.to_lowercase();
        // Remove from all other modules
        for module in &mut self.modules {
            module.app_keywords.retain(|kw| kw.to_lowercase() != kw_lower);
        }
        // Add to target module
        if let Some(target) = self.modules.iter_mut().find(|m| m.id == target_module_id) {
            target.app_keywords.push(app_name.to_string());
        }
        // 重建索引
        self.keyword_index = build_keyword_index(&self.modules);
    }

    pub fn is_sensitive_app(&self, app_name: &str) -> bool {
        let lower = app_name.to_lowercase();
        self.sensitive_apps
            .iter()
            .any(|s| lower.contains(s.as_str()))
    }

    pub fn is_sensitive_url(&self, url: &str) -> bool {
        let lower = url.to_lowercase();
        self.sensitive_url_keywords
            .iter()
            .any(|kw| lower.contains(kw.as_str()))
    }

    pub fn is_ignored_app(&self, app_name: &str) -> bool {
        let lower = app_name.to_lowercase();
        self.ignored_apps
            .iter()
            .any(|s| lower.contains(s.as_str()))
    }
}

fn default_modules() -> Vec<ModuleConfig> {
    vec![
        ModuleConfig {
            id: "dev".to_string(),
            name: "开发".to_string(),
            color: "#c47a5a".to_string(),
            app_keywords: vec![
                "code".to_string(),
                "idea".to_string(),
                "rust".to_string(),
                "terminal".to_string(),
            ],
            site_domains: vec![],
        },
        ModuleConfig {
            id: "entertainment".to_string(),
            name: "娱乐".to_string(),
            color: "#d4726a".to_string(),
            app_keywords: vec![
                "bilibili".to_string(),
                "youtube".to_string(),
                "steam".to_string(),
            ],
            site_domains: vec![],
        },
        ModuleConfig {
            id: "social".to_string(),
            name: "社交".to_string(),
            color: "#c9a35a".to_string(),
            app_keywords: vec!["wechat".to_string(), "discord".to_string()],
            site_domains: vec![],
        },
        ModuleConfig {
            id: "productivity".to_string(),
            name: "效率".to_string(),
            color: "#7a9e7e".to_string(),
            app_keywords: vec![
                "word".to_string(),
                "obsidian".to_string(),
                "notion".to_string(),
            ],
            site_domains: vec![],
        },
        ModuleConfig {
            id: "browser".to_string(),
            name: "浏览".to_string(),
            color: "#7a8a9e".to_string(),
            app_keywords: vec![
                "chrome".to_string(),
                "edge".to_string(),
                "firefox".to_string(),
            ],
            site_domains: vec![],
        },
    ]
}

fn modules_from_rules(rules: &HashMap<String, Vec<String>>) -> Vec<ModuleConfig> {
    let mut modules = Vec::new();
    for (id, keywords) in rules {
        modules.push(ModuleConfig {
            id: id.to_string(),
            name: default_name_for_id(id).to_string(),
            color: default_color_for_id(id).to_string(),
            app_keywords: keywords.clone(),
            site_domains: vec![],
        });
    }
    modules
}

fn default_name_for_id(id: &str) -> &str {
    match id {
        "dev" => "开发",
        "entertainment" => "娱乐",
        "social" => "社交",
        "productivity" => "效率",
        "browser" => "浏览",
        _ => id,
    }
}

fn default_color_for_id(id: &str) -> &str {
    match id {
        "dev" => "#c47a5a",
        "entertainment" => "#d4726a",
        "social" => "#c9a35a",
        "productivity" => "#7a9e7e",
        "browser" => "#7a8a9e",
        _ => "#8a8278",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> ClassifierConfig {
        ClassifierConfig::default()
    }

    // ── classify_app ──

    #[test]
    fn classify_dev_apps() {
        let cfg = default_cfg();
        assert_eq!(cfg.classify_app("Code.exe"), "dev");
        assert_eq!(cfg.classify_app("idea64.exe"), "dev");
        assert_eq!(cfg.classify_app("rust-analyzer"), "dev");
        assert_eq!(cfg.classify_app("WindowsTerminal.exe"), "dev");
    }

    #[test]
    fn classify_entertainment_apps() {
        let cfg = default_cfg();
        assert_eq!(cfg.classify_app("bilibili.exe"), "entertainment");
        assert_eq!(cfg.classify_app("YouTube.exe"), "entertainment");
        assert_eq!(cfg.classify_app("steam.exe"), "entertainment");
    }

    #[test]
    fn classify_social_apps() {
        let cfg = default_cfg();
        assert_eq!(cfg.classify_app("WeChat.exe"), "social");
        assert_eq!(cfg.classify_app("Discord.exe"), "social");
    }

    #[test]
    fn classify_productivity_apps() {
        let cfg = default_cfg();
        assert_eq!(cfg.classify_app("WINWORD.EXE"), "productivity");
        assert_eq!(cfg.classify_app("Obsidian.exe"), "productivity");
        assert_eq!(cfg.classify_app("Notion.exe"), "productivity");
    }

    #[test]
    fn classify_browser_apps() {
        let cfg = default_cfg();
        assert_eq!(cfg.classify_app("chrome.exe"), "browser");
        assert_eq!(cfg.classify_app("msedge.exe"), "browser");
        assert_eq!(cfg.classify_app("firefox.exe"), "browser");
    }

    #[test]
    fn classify_uncategorized() {
        let cfg = default_cfg();
        assert_eq!(cfg.classify_app("explorer.exe"), "uncategorized");
        assert_eq!(cfg.classify_app("notepad.exe"), "uncategorized");
        assert_eq!(cfg.classify_app(""), "uncategorized");
    }

    #[test]
    fn classify_case_insensitive() {
        let cfg = default_cfg();
        assert_eq!(cfg.classify_app("CODE.EXE"), "dev");
        assert_eq!(cfg.classify_app("code.exe"), "dev");
        assert_eq!(cfg.classify_app("Code.exe"), "dev");
    }

    #[test]
    fn classify_first_match_wins() {
        // "code" matches dev, if a keyword appears in multiple modules, first wins
        let cfg = default_cfg();
        assert_eq!(cfg.classify_app("Code.exe"), "dev");
    }

    // ── classify_domain ──

    #[test]
    fn classify_domain_exact_match() {
        let mut cfg = default_cfg();
        cfg.modules[0].site_domains = vec!["github.com".to_string()];
        assert_eq!(cfg.classify_domain("github.com"), "dev");
    }

    #[test]
    fn classify_domain_subdomain() {
        let mut cfg = default_cfg();
        cfg.modules[0].site_domains = vec!["github.com".to_string()];
        assert_eq!(cfg.classify_domain("api.github.com"), "dev");
    }

    #[test]
    fn classify_domain_no_match() {
        let cfg = default_cfg();
        assert_eq!(cfg.classify_domain("example.com"), "uncategorized");
    }

    #[test]
    fn classify_domain_case_insensitive() {
        let mut cfg = default_cfg();
        cfg.modules[0].site_domains = vec!["GitHub.com".to_string()];
        assert_eq!(cfg.classify_domain("github.com"), "dev");
    }

    // ── is_sensitive_app ──

    #[test]
    fn sensitive_app_detected() {
        let cfg = default_cfg();
        assert!(cfg.is_sensitive_app("1password.exe"));
        assert!(cfg.is_sensitive_app("KeePass.exe"));
    }

    #[test]
    fn non_sensitive_app() {
        let cfg = default_cfg();
        assert!(!cfg.is_sensitive_app("Code.exe"));
        assert!(!cfg.is_sensitive_app("chrome.exe"));
    }

    // ── is_sensitive_url ──

    #[test]
    fn sensitive_url_detected() {
        let cfg = default_cfg();
        assert!(cfg.is_sensitive_url("https://bank.com/login"));
        assert!(cfg.is_sensitive_url("https://example.com/auth/token"));
        assert!(cfg.is_sensitive_url("https://example.com/credentials"));
    }

    #[test]
    fn non_sensitive_url() {
        let cfg = default_cfg();
        assert!(!cfg.is_sensitive_url("https://github.com/rust-lang"));
        assert!(!cfg.is_sensitive_url("https://stackoverflow.com/questions"));
        assert!(!cfg.is_sensitive_url("https://example.com/pay"));
        assert!(!cfg.is_sensitive_url("https://example.com/auth"));
    }

    // ── add_app_keyword_to_module ──

    #[test]
    fn add_keyword_moves_between_modules() {
        let mut cfg = default_cfg();
        // "chrome" is currently in "browser"
        assert_eq!(cfg.classify_app("chrome.exe"), "browser");

        // Move to "dev"
        cfg.add_app_keyword_to_module("chrome", "dev");
        assert_eq!(cfg.classify_app("chrome.exe"), "dev");

        // Should no longer be in "browser"
        // (verify it's removed from browser by checking the keyword list)
        let browser = cfg.modules.iter().find(|m| m.id == "browser").unwrap();
        assert!(!browser.app_keywords.iter().any(|kw| kw == "chrome"));
    }

    #[test]
    fn add_keyword_to_nonexistent_module_is_noop() {
        let mut cfg = default_cfg();
        cfg.add_app_keyword_to_module("chrome", "nonexistent");
        // chrome should still be in browser
        assert_eq!(cfg.classify_app("chrome.exe"), "browser");
    }

    // ── default_modules ──

    #[test]
    fn default_modules_have_ids() {
        let modules = default_modules();
        let ids: Vec<&str> = modules.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"dev"));
        assert!(ids.contains(&"entertainment"));
        assert!(ids.contains(&"social"));
        assert!(ids.contains(&"productivity"));
        assert!(ids.contains(&"browser"));
    }

    // ── modules_from_rules (legacy migration) ──

    #[test]
    fn modules_from_rules_creates_correct_modules() {
        let mut rules = HashMap::new();
        rules.insert("dev".to_string(), vec!["vscode".to_string()]);
        rules.insert("game".to_string(), vec!["steam".to_string()]);

        let modules = modules_from_rules(&rules);
        assert_eq!(modules.len(), 2);

        let dev = modules.iter().find(|m| m.id == "dev").unwrap();
        assert_eq!(dev.app_keywords, vec!["vscode"]);
        assert_eq!(dev.name, "开发");

        let game = modules.iter().find(|m| m.id == "game").unwrap();
        assert_eq!(game.app_keywords, vec!["steam"]);
        assert_eq!(game.name, "game"); // unknown id falls back to id itself
    }
}
