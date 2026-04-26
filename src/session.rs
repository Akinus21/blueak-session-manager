use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub version: u32,
    pub saved_at: String,
    pub workspaces: Vec<SavedWorkspace>,
    pub windows: Vec<SavedWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedWorkspace {
    pub idx: u8,
    pub name: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedWindow {
    pub app_id: String,
    pub title: String,
    pub workspace_idx: u8,
    pub is_floating: bool,
    pub is_focused: bool,
    pub float_x: Option<i32>,
    pub float_y: Option<i32>,
    pub float_width: Option<u32>,
    pub float_height: Option<u32>,
}

impl Session {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read session file: {}", path.display()))?;
        let session: Session = serde_json::from_str(&content)
            .with_context(|| "failed to parse session file")?;
        Ok(session)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp_path = path.with_file_name("session.json.tmp");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&tmp_path, content)?;
        fs::rename(&tmp_path, path)?;
        Ok(())
    }
}