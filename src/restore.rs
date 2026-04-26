use crate::app_launch::AppLauncher;
use crate::ipc;
use crate::session::Session;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

struct PendingWindow {
    app_id: String,
    workspace_idx: u8,
    is_floating: bool,
    float_x: Option<i32>,
    float_y: Option<i32>,
}

pub fn restore(
    session_path: &Path,
    _app_map_path: Option<&Path>,
    timeout_secs: u64,
    dry_run: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    if !session_path.exists() {
        log::info!("No session file found — nothing to restore");
        return Ok(());
    }

    let session = Session::load(session_path)?;

    if session.version != 1 {
        anyhow::bail!("unsupported session version {}", session.version);
    }

    if verbose {
        eprintln!("Restoring session from {} (saved at {})",
            session_path.display(), session.saved_at);
        eprintln!("  {} workspaces, {} windows",
            session.workspaces.len(), session.windows.len());
    }

    let launcher = AppLauncher::new()?;

    let mut workspace_windows: HashMap<u8, Vec<usize>> = HashMap::new();
    for (idx, window) in session.windows.iter().enumerate() {
        workspace_windows.entry(window.workspace_idx).or_default().push(idx);
    }

    let mut pending: Vec<(PendingWindow, usize)> = Vec::new();

    let mut workspace_indices: Vec<u8> = workspace_windows.keys().copied().collect();
    workspace_indices.sort();

    for ws_idx in workspace_indices {
        if let Err(e) = ipc::focus_workspace(ws_idx) {
            log::warn!("failed to focus workspace {}: {}", ws_idx, e);
        }
        std::thread::sleep(Duration::from_millis(100));

        for &window_idx in workspace_windows.get(&ws_idx).unwrap() {
            let window = &session.windows[window_idx];

            let existing = ipc::get_windows()?;
            let already_running = existing.iter().any(|w| {
                w.app_id.as_ref() == Some(&window.app_id)
            });

            if already_running {
                log::info!("{} already running — skipping", window.app_id);
                continue;
            }

            if let Some(cmd) = launcher.resolve(&window.app_id) {
                if cmd.is_empty() {
                    log::info!("{} marked as skip — skipping", window.app_id);
                    continue;
                }

                if verbose {
                    eprintln!("Would spawn: {} on workspace {}", cmd.join(" "), window.workspace_idx);
                }

                if !dry_run {
                    if let Err(e) = ipc::spawn(cmd.clone()) {
                        log::warn!("failed to spawn {}: {}", window.app_id, e);
                        continue;
                    }
                }

                pending.push((PendingWindow {
                    app_id: window.app_id.clone(),
                    workspace_idx: window.workspace_idx,
                    is_floating: window.is_floating,
                    float_x: window.float_x,
                    float_y: window.float_y,
                }, window_idx));
            } else {
                log::warn!("Cannot find launch command for app_id=\"{}\" title=\"{}\" — skipping",
                    window.app_id, window.title);
            }
        }
    }

    if dry_run {
        return Ok(());
    }

    let timeout = Duration::from_secs(timeout_secs);
    let poll_interval = Duration::from_millis(500);
    let start = std::time::Instant::now();

    let mut remaining: Vec<(PendingWindow, usize)> = pending;
    let mut matched_windows: HashMap<usize, u64> = HashMap::new();
    let matched_ids: &[u64] = &matched_windows.values().copied().collect::<Vec<_>>();

    while !remaining.is_empty() && start.elapsed() < timeout {
        std::thread::sleep(poll_interval);

        let windows = match ipc::get_windows() {
            Ok(w) => w,
            Err(e) => {
                log::warn!("failed to get windows: {}", e);
                continue;
            }
        };

        let current_matched: Vec<u64> = matched_windows.values().copied().collect();

        let mut still_remaining = Vec::new();

        for (pending, original_idx) in remaining {
            let matching = windows.iter().find(|w| {
                w.app_id.as_ref() == Some(&pending.app_id) &&
                !current_matched.contains(&w.id)
            });

            if let Some(found) = matching {
                matched_windows.insert(original_idx, found.id);

                if pending.is_floating {
                    if let Err(e) = ipc::toggle_floating(found.id) {
                        log::warn!("failed to toggle floating: {}", e);
                    }
                }

                if let (Some(x), Some(y)) = (pending.float_x, pending.float_y) {
                    if let Err(e) = ipc::move_floating_window(found.id, x, y) {
                        log::warn!("failed to position floating window: {}", e);
                    }
                }
            } else {
                still_remaining.push((pending, original_idx));
            }
        }

        remaining = still_remaining;
    }

    if !remaining.is_empty() {
        log::warn!("{} windows never appeared", remaining.len());
    }

    if let Some(&(ref _pending, idx)) = pending.iter().find(|(_, idx)| session.windows[*idx].is_focused) {
        if let Some(&window_id) = matched_windows.get(idx) {
            if let Err(e) = ipc::focus_window(window_id) {
                log::warn!("failed to focus window: {}", e);
            }
        }
    }

    Ok(())
}