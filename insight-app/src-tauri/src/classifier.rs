use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::models::ModuleConfig;

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
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        let modules = default_modules();
        ClassifierConfig {
            modules,
            rules: HashMap::new(),
            sensitive_apps: vec!["1password".to_string(), "keepass".to_string()],
            sensitive_url_keywords: vec![
                "password".to_string(),
                "token".to_string(),
                "bank".to_string(),
                "pay".to_string(),
                "secret".to_string(),
                "auth".to_string(),
                "login".to_string(),
            ],
        }
    }
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
        for module in &self.modules {
            for kw in &module.app_keywords {
                let kw = kw.to_lowercase();
                if !kw.is_empty() && app_lower.contains(kw.as_str()) {
                    return module.id.clone();
                }
            }
        }
        "uncategorized".to_string()
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
        "uncategorized".to_string()
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
