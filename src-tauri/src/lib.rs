pub mod boundary;
mod approval;
mod bridge;
mod commands;
pub mod doctor;
pub mod models;
pub mod platform;
pub mod policy;
pub mod risk;
pub mod setup;
pub mod shell_hooks;
mod snapshot;
pub mod storage;
mod terminal;
mod watcher;

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
            // --- v0.2 additions ---
            commands::get_active_stats,
            commands::get_ledger,
            commands::list_pending_approvals,
            commands::resolve_approval,
            commands::preview_learn_rule,
            commands::get_enforcement_paths,
        ])
        .setup(|_app| {
            let _ = storage::ensure_dirs();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ActionGuard");
}
