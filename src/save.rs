use crate::ipc;
use crate::session::{SavedWindow, SavedWorkspace, Session};
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;

pub fn save(path: &Path, verbose: bool) -> anyhow::Result<()> {
    let workspaces = ipc::get_workspaces()?;
    let windows = ipc::get_windows()?;

    let mut workspace_map: HashMap<u64, SavedWorkspace> = HashMap::new();
    let saved_workspaces: Vec<SavedWorkspace> = workspaces.iter().filter_map(|w| {
        let sw = SavedWorkspace {
            idx: w.idx,
            name: w.name.clone(),
            output: w.output.clone(),
        };
        workspace_map.insert(w.id, sw);
        Some(SavedWorkspace {
            idx: w.idx,
            name: w.name.clone(),
            output: w.output.clone(),
        })
    }).collect();

    let mut saved_windows: Vec<SavedWindow> = Vec::new();

    for window in &windows {
        let app_id = window.app_id.clone().unwrap_or_default();
        if app_id.is_empty() {
            continue;
        }

        let workspace_idx = if let Some(ws_id) = window.workspace_id {
            workspace_map.get(&ws_id).map(|w| w.idx)
        } else {
            None
        };

        let workspace_idx = match workspace_idx {
            Some(idx) => idx,
            None => continue,
        };

        let saved_window = SavedWindow {
            app_id,
            title: window.title.clone().unwrap_or_default(),
            workspace_idx,
            is_floating: false,
            is_focused: window.is_focused,
            float_x: None,
            float_y: None,
            float_width: None,
            float_height: None,
        };

        saved_windows.push(saved_window);
    }

    saved_windows.sort_by(|a, b| {
        match a.workspace_idx.cmp(&b.workspace_idx) {
            std::cmp::Ordering::Equal => {
                if a.is_focused == b.is_focused {
                    std::cmp::Ordering::Less
                } else if a.is_focused {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            }
            ord => ord,
        }
    });

    let session = Session {
        version: 1,
        saved_at: Utc::now().to_rfc3339(),
        workspaces: saved_workspaces,
        windows: saved_windows,
    };

    if verbose {
        eprintln!("Saved {} windows across {} workspaces",
            session.windows.len(), session.workspaces.len());
        for w in &session.windows {
            eprintln!("  [{}/{}] {} ({}) {}",
                w.workspace_idx, if w.is_focused { "focused" } else { "unfocused" },
                w.app_id, if w.is_floating { "floating" } else { "tiled" }, w.title);
        }
    }

    session.save(path)?;

    log::info!("Saved session to {}", path.display());

    Ok(())
}