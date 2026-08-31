pub mod boundary;
pub mod correlation;
pub mod models;
pub use models::BoundaryKind;
pub use boundary::BoundaryStatus; // re-export for CLI consumers
mod approval;
mod bridge;
mod commands;
pub mod doctor;
pub mod platform;
pub mod policy;
pub mod risk;
pub mod setup;
pub mod shell_hooks;
mod snapshot;
pub mod storage;
mod terminal;
mod watcher;

fn handle_request(mut stream: std::net::TcpStream, dist_path: &std::path::Path) {
    use std::io::{Read, Write};
    let mut buffer = [0u8; 8192];
    if let Ok(n) = stream.read(&mut buffer) {
        let request = String::from_utf8_lossy(&buffer[..n]);
        let first_line = request.lines().next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return;
        }
        let path = parts[1].trim_start_matches('/');
        let file_path = dist_path.join(path);

        let (status, mime, body) = if file_path.is_file() {
            let mime = if path.ends_with(".css") {
                "text/css"
            } else if path.ends_with(".js") {
                "application/javascript"
            } else if path.ends_with(".html") {
                "text/html"
            } else {
                "application/octet-stream"
            };
            if let Ok(mut f) = std::fs::File::open(&file_path) {
                let mut data = Vec::new();
                let _ = f.read_to_end(&mut data);
                (200, mime, data)
            } else {
                (404, "text/plain", b"File not found".to_vec())
            }
        } else {
            let index_path = dist_path.join("index.html");
            if let Ok(mut f) = std::fs::File::open(&index_path) {
                let mut data = Vec::new();
                let _ = f.read_to_end(&mut data);
                (200, "text/html", data)
            } else {
                (500, "text/plain", b"index.html not found".to_vec())
            }
        };

        let response = format!(
            "HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            status, mime, body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(&body);
        let _ = stream.flush();
    }
}

// No dead-code warning needed here: `run_http_server` is called from `main.rs`
// when invoked with `--serve <dist_path> <port>`.

/// Handles the `--serve <dist_path> <port>` command-line mode for the HTTP server.
pub fn run_http_server(dist_path: &str, port_str: &str) {
    let port: u16 = port_str.parse().unwrap_or(47832);
    let dist_path = std::path::PathBuf::from(dist_path);
    let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    listener.set_nonblocking(true).unwrap();
    loop {
        if let Ok((stream, _)) = listener.accept() {
            let dist = dist_path.clone();
            std::thread::spawn(move || {
                handle_request(stream, &dist);
            });
        } else {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

use std::sync::{Arc, RwLock};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(commands::AppState {
        session: std::sync::Mutex::new(None),
        config: std::sync::Mutex::new(storage::load_config()),
        policy: Arc::new(RwLock::new(policy::load_policy_set())),
        approvals: Arc::new(approval::ApprovalStore::new()),
        last_policy_mtime: std::sync::Mutex::new(0),
    });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_startup_args,
            commands::choose_workspace,
            commands::start_session,
            commands::stop_session,
            commands::undo_active_session,
            commands::allow_batch,
            commands::deny_batch,
            commands::undo_session,
            commands::list_sessions,
            commands::get_session,
            commands::get_active_session,
            commands::get_pending_batch,
            commands::get_config,
            commands::update_config,
            commands::get_active_stats,
            commands::get_ledger,
            commands::list_pending_approvals,
            commands::resolve_approval,
            commands::preview_learn_rule,
            commands::get_enforcement_paths,
            commands::get_coverage,
            commands::window_minimize,
            commands::window_toggle_maximize,
            commands::window_close,
        ])
        .setup(|app| {
            let _ = storage::ensure_dirs();
            use tauri::Manager;
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--hidden"]),
            ))?;
            app.handle().plugin(tauri_plugin_opener::init())?;
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(size) = window.outer_size() {
                    if let Ok(pos) = window.outer_position() {
                        if let Ok(scale) = window.scale_factor() {
                            let log_path = std::env::current_exe()
                                .ok()
                                .and_then(|p| p.parent().map(|p| p.join("ag_geometry.log")))
                                .unwrap_or_default();
                            let _ = std::fs::write(&log_path, format!(
                                "outer_size={}x{}\nouter_pos=({}, {})\nscale_factor={}\n",
                                size.width, size.height, pos.x, pos.y, scale
                            ));
                        }
                    }
                }
            }
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ActionGuard");
}
