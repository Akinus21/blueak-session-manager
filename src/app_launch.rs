use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use xdg::BaseDirectories;

pub struct AppLauncher {
    overrides: HashMap<String, Vec<String>>,
}

impl AppLauncher {
    pub fn new() -> anyhow::Result<Self> {
        let mut overrides = HashMap::new();

        if let Ok(xdg) = BaseDirectories::with_prefix("niri-session") {
            let map_path = xdg.get_config_file("app-map.toml");
            if map_path.exists() {
                let content = fs::read_to_string(&map_path)?;
                if let Ok(toml::Value::Table(table)) = toml::from_str(&content) {
                    if let Some(apps) = table.get("apps").and_then(|v| v.as_table()) {
                        for (app_id, cmd) in apps {
                            if let Some(cmd_str) = cmd.as_str() {
                                let parts = cmd_str
                                    .split_whitespace()
                                    .map(String::from)
                                    .collect::<Vec<_>>();
                                if !parts.is_empty() {
                                    overrides.insert(app_id.clone(), parts);
                                }
                            }
                        }
                    }
                    if let Some(skip) = table.get("skip").and_then(|v| v.as_table()) {
                        if let Some(apps) = skip.get("apps").and_then(|v| v.as_array()) {
                            for app in apps {
                                if let Some(name) = app.as_str() {
                                    overrides.insert(name.to_string(), vec![]);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self { overrides })
    }

    pub fn resolve(&self, app_id: &str) -> Option<Vec<String>> {
        if let Some(cmd) = self.overrides.get(app_id) {
            if cmd.is_empty() {
                return None;
            }
            return Some(cmd.clone());
        }

        if let Some(cmd) = self.try_desktop_lookup(app_id) {
            return Some(cmd);
        }

        if app_id.contains('.') {
            if let Some(cmd) = self.try_flatpak(app_id) {
                return Some(cmd);
            }
        }

        self.try_direct_binary(app_id)
    }

    fn try_desktop_lookup(&self, app_id: &str) -> Option<Vec<String>> {
        let data_home = BaseDirectories::new().ok()?.get_data_home();
        let data_dirs = vec![
            data_home,
            PathBuf::from("/usr/local/share"),
            PathBuf::from("/usr/share"),
        ];

        for data_dir in data_dirs {
            let apps_dir = data_dir.join("applications");
            if let Ok(entries) = fs::read_dir(apps_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Some(cmd) = self.parse_desktop_file(app_id, &content) {
                                return Some(cmd);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn parse_desktop_file(&self, app_id: &str, content: &str) -> Option<Vec<String>> {
        let mut exec_line = None;
        let mut startup_wm_class = None;

        for line in content.lines() {
            if line.starts_with("Exec=") {
                exec_line = Some(line.trim_start_matches("Exec=").to_string());
            }
            if line.starts_with("StartupWMClass=") {
                startup_wm_class = Some(line.trim_start_matches("StartupWMClass=").to_string());
            }
        }

        if startup_wm_class.as_deref() == Some(app_id) {
            return exec_line.map(|e| self.clean_exec_line(e));
        }

        let first_line = content.lines().next()?;
        let filename = first_line.trim_start_matches('[');
        if filename == app_id {
            return exec_line.map(|e| self.clean_exec_line(e));
        }

        None
    }

    fn clean_exec_line(&self, exec: String) -> Vec<String> {
        exec.split_whitespace()
            .filter(|&part| !part.starts_with('%'))
            .map(String::from)
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn try_flatpak(&self, app_id: &str) -> Option<Vec<String>> {
        let output = Command::new("flatpak")
            .args(["list", "--app", "--columns=application"])
            .output()
            .ok()?;

        if output.status.success() {
            let apps = String::from_utf8_lossy(&output.stdout);
            for line in apps.lines() {
                if line.trim() == app_id {
                    return Some(vec!["flatpak".to_string(), "run".to_string(), app_id.to_string()]);
                }
            }
        }

        None
    }

    fn try_direct_binary(&self, app_id: &str) -> Option<Vec<String>> {
        let candidates = vec![
            app_id.to_string(),
            app_id.rsplit_once('.')
                .map(|(_, rest)| rest.to_string())
                .unwrap_or_default(),
        ];

        for candidate in candidates {
            if candidate.is_empty() {
                continue;
            }
            if Command::new("which").arg(&candidate).output().map(|o| o.status.success()).unwrap_or(false) {
                return Some(vec![candidate]);
            }
        }

        None
    }
}