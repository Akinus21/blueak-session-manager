use niri_ipc::{Request, Response};
use niri_ipc::socket::Socket;
use std::env;

fn send_request(request: Request) -> anyhow::Result<Response> {
    let socket_path = env::var(niri_ipc::socket::SOCKET_PATH_ENV)
        .context("NIRI_SOCKET not set — is niri running?")?;
    let mut socket = Socket::connect()?;
    let reply = socket.send(request)?;
    reply.map_err(|e| anyhow::anyhow!("niri IPC error: {e}"))
}

pub fn get_windows() -> anyhow::Result<Vec<niri_ipc::Window>> {
    let response = send_request(Request::Windows)?;
    match response {
        Response::Windows(windows) => Ok(windows),
        _ => Err(anyhow::anyhow!("unexpected response type")),
    }
}

pub fn get_workspaces() -> anyhow::Result<Vec<niri_ipc::Workspace>> {
    let response = send_request(Request::Workspaces)?;
    match response {
        Response::Workspaces(workspaces) => Ok(workspaces),
        _ => Err(anyhow::anyhow!("unexpected response type")),
    }
}

pub fn focus_workspace(idx: u8) -> anyhow::Result<()> {
    let response = send_request(Request::Action(niri_ipc::Action::FocusWorkspace {
        reference: niri_ipc::WorkspaceReferenceArg::Index(idx),
    }))?;
    match response {
        Response::Ok => Ok(()),
        Response::Error(e) => Err(anyhow::anyhow!("IPC error: {}", e)),
        _ => Err(anyhow::anyhow!("unexpected response type")),
    }
}

pub fn spawn(command: Vec<String>) -> anyhow::Result<()> {
    let response = send_request(Request::Action(niri_ipc::Action::Spawn { command }))?;
    match response {
        Response::Ok => Ok(()),
        Response::Error(e) => Err(anyhow::anyhow!("IPC error: {}", e)),
        _ => Err(anyhow::anyhow!("unexpected response type")),
    }
}

pub fn move_window_to_workspace(window_id: u64, workspace_idx: u8) -> anyhow::Result<()> {
    let response = send_request(Request::Action(niri_ipc::Action::MoveWindowToWorkspace {
        window_id: Some(window_id),
        reference: niri_ipc::WorkspaceReferenceArg::Index(workspace_idx),
        focus: true,
    }))?;
    match response {
        Response::Ok => Ok(()),
        Response::Error(e) => Err(anyhow::anyhow!("IPC error: {}", e)),
        _ => Err(anyhow::anyhow!("unexpected response type")),
    }
}

pub fn move_floating_window(window_id: u64, x: i32, y: i32) -> anyhow::Result<()> {
    let response = send_request(Request::Action(niri_ipc::Action::MoveFloatingWindow {
        id: Some(window_id),
        x: niri_ipc::PositionChange::Absolute(x),
        y: niri_ipc::PositionChange::Absolute(y),
    }))?;
    match response {
        Response::Ok => Ok(()),
        Response::Error(e) => Err(anyhow::anyhow!("IPC error: {}", e)),
        _ => Err(anyhow::anyhow!("unexpected response type")),
    }
}

pub fn toggle_floating(window_id: u64) -> anyhow::Result<()> {
    let response = send_request(Request::Action(niri_ipc::Action::ToggleWindowFloating {
        id: Some(window_id),
    }))?;
    match response {
        Response::Ok => Ok(()),
        Response::Error(e) => Err(anyhow::anyhow!("IPC error: {}", e)),
        _ => Err(anyhow::anyhow!("unexpected response type")),
    }
}

pub fn focus_window(window_id: u64) -> anyhow::Result<()> {
    let response = send_request(Request::Action(niri_ipc::Action::FocusWindow {
        id: window_id,
    }))?;
    match response {
        Response::Ok => Ok(()),
        Response::Error(e) => Err(anyhow::anyhow!("IPC error: {}", e)),
        _ => Err(anyhow::anyhow!("unexpected response type")),
    }
}