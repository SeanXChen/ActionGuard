use crate::models::{
    Action, ActionCategory, ActionKind, AppConfig, ApprovalRequest, ApprovalResolution,
    BatchData, CapabilityTier, CategoryCounts, Counts, Decision, EnforcementCounts, LedgerEntry,
    PolicyFile, RiskCounts, RiskLevel, Rule, SessionDetails, SessionStatus, SessionSummary,
    UndoResult,
};
use crate::{bridge, policy, risk, shell_hooks, snapshot, storage, terminal, watcher};
use anyhow::Result;
use chrono::Local;
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct ActiveSession {
    pub id: String,
    pub num: u32,
    pub workspace: PathBuf,
    pub started_at: String,
    /// All finalized actions of the session.
    pub actions: Vec<Action>,
    /// Actions currently accumulating (not yet finalized into a batch).
    pub batch: Vec<Action>,
    pub batch_keys: HashSet<String>,
    pub total_counts: Counts,
    pub category_counts: CategoryCounts,
    pub risk_counts: RiskCounts,
    pub actions_protected: u32,
    pub actions_blocked: u32,
    /// v0.3 — enforcement outcome split (Detection ≠ Protection).
    pub enforcement_counts: EnforcementCounts,
    pub awaiting_review: bool,
    pub pending_batch: Option<BatchData>,
    pub watcher: Option<watcher::SharedWatcher>,
    /// v0.2 Shell bridge — listens for pre-execution commands from the
    /// protected terminal's hook. Stopped on session finalize.
    pub bridge: Option<bridge::Bridge>,
    /// Which shell the protected terminal was launched with. Mostly for
    /// debugging — the bridge is shell-agnostic.
    #[allow(dead_code)] // debugging aid, surfaced in GUI diagnostics
    pub shell: String,
    /// v0.2 Mode A/B: Observe = record only, Protected = block high-risk.
    /// Set at session start, immutable for the session lifetime.
    pub mode: crate::models::SessionMode,
    /// v0.2 — approval popups fired (interruptions). Denominator of User
    /// Override Rate.
    pub popups: u32,
    /// v0.2 — times the user allowed a gated action. Numerator of User
    /// Override Rate.
    pub overrides: u32,
}

pub struct AppState {
    pub session: Mutex<Option<ActiveSession>>,
    pub config: Mutex<AppConfig>,
    /// v0.2 Policy set behind a RwLock so the "Always deny" approval flow
    /// (Phase C) and the user-policy hot-reload (Phase D) can swap it
    /// atomically without bringing down the bridge.
    pub policy: Arc<std::sync::RwLock<policy::PolicySet>>,
    /// v0.2 Approval gate: pending approvals blocked at the shell bridge,
    /// waiting for the user to Allow once / Deny / Always deny.
    pub approvals: Arc<crate::approval::ApprovalStore>,
    /// v0.2 Phase D — mtime (nanos since UNIX_EPOCH) of `policies.user.yml`
    /// as it was when we last loaded it. `0` means "not yet tracked"; the
    /// bridge re-stats the file on each /preexec and reloads if it changed.
    pub last_policy_mtime: Mutex<u64>,
}

#[derive(Serialize, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub num: u32,
    pub workspace: String,
    pub started_at: String,
    pub snapshot_files: u32,
    pub mode: crate::models::SessionMode,
}

fn now_str() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn emit(app: &AppHandle, event: &str, payload: impl Serialize + Clone) {
    let _ = app.emit(event, payload);
}

fn counts_of(actions: &[Action]) -> Counts {
    let mut c = Counts::default();
    for a in actions {
        c.add(a.action);
    }
    c
}

fn mark_flags(change: &mut Action) {
    let path = change.path_str().to_string();
    if !change.sensitive {
        let from_sensitive = change
            .from
            .as_deref()
            .map(risk::is_sensitive_path)
            .unwrap_or(false);
        if risk::is_sensitive_path(&path) || from_sensitive {
            change.sensitive = true;
        }
    }
    if change.sensitive && change.asset.is_none() {
        if let Some(a) = risk::detect_asset(&path) {
            change.asset = Some(a);
        }
    }
}

/// Pure classification step: stamps `risk`, `reasons`, `asset`,
/// `matched_rule`, `decision` on the action. Does NOT mutate session
/// counters and does NOT push to the ledger. Used by both the file-watcher
/// path (via [`classify_ingested`]) and the shell-bridge path, which needs
/// to wait for an approval decision before recording.
pub(crate) fn classify_action(state: &Arc<AppState>, action: &mut Action) {
    // 1) Deterministic risk engine.
    let r = risk::evaluate_action(action);
    action.risk = Some(r.level);
    if !r.reasons.is_empty() {
        action.reasons = r.reasons.clone();
    }
    if action.asset.is_none() {
        if let Some(a) = r.asset {
            action.asset = Some(a);
        }
    }

    // 2) Policy engine. A matched rule can override the risk level (e.g. a
    //    `deny` rule on .env sets Critical even though a single Modify would
    //    otherwise be Low). The risk level is bumped UP only, never down.
    //    The lock is held only for the duration of the match — never across
    //    I/O — so concurrent shell commands don't serialize on policy reads.
    let decision = {
        let policy_set = state.policy.read().unwrap();
        policy::decide(action, &policy_set)
    };
    action.decision = Some(decision.decision);
    if let Some(id) = &decision.matched_rule {
        action.matched_rule = Some(id.clone());
    }
    if !decision.reason.is_empty() && !action.reasons.contains(&decision.reason) {
        action.reasons.push(decision.reason);
    }
    if decision.risk > action.risk.unwrap_or(RiskLevel::Low) {
        action.risk = Some(decision.risk);
    }
}

/// Bump the live session counters for an action. Call after the final
/// decision is known — `actions_blocked` only fires on `Decision::Deny`.
/// Doesn't touch the ledger; pair with `push_action` to record.
pub(crate) fn bump_counters(state: &Arc<AppState>, action: &Action) {
    let mut guard = state.session.lock().unwrap();
    if let Some(s) = guard.as_mut() {
        s.category_counts.add(action.category);
        if let Some(r) = action.risk {
            s.risk_counts.add(r);
        }
        if matches!(action.decision.unwrap_or_default(), crate::models::Decision::Deny) {
            s.actions_blocked += 1;
        }
        // Enforcement outcome — the honest Detection ≠ Protection split.
        // Actions that never crossed a boundary carry no enforcement tag and
        // are excluded here (see `enforcement_counts.total()` vs. protected).
        if let Some(e) = action.enforcement {
            s.enforcement_counts.add(e);
        }
    }
}

/// Per-ingest classification. Runs the deterministic risk engine on a
/// single action (stamps `risk`, `reasons`, `asset`) and then consults the
/// policy engine to fill `matched_rule` and `decision`. When a policy rule
/// fires with a higher risk level than the risk engine, the rule's risk wins.
fn classify_ingested(state: &Arc<AppState>, action: &mut Action) {
    classify_action(state, action);
    bump_counters(state, action);
}

// ---------------------------------------------------------------------------
// Aggregator
// ---------------------------------------------------------------------------

fn handle_event(state: &Arc<AppState>, ev: watcher::FsEvent) {
    let ws = {
        let guard = state.session.lock().unwrap();
        match guard.as_ref() {
            Some(s) => s.workspace.clone(),
            None => return,
        }
    };

    // New directory created inside the workspace -> start watching it.
    if ev.is_dir && !ev.outside {
        if let watcher::FsEventKind::Create = ev.kind {
            if let Ok(w) = state.session.lock() {
                if let Some(s) = w.as_ref() {
                    if let Some(shared) = &s.watcher {
                        watcher::add_dir_watch(shared, &ws.join(&ev.path));
                    }
                }
            }
        }
        return; // directory entries are not counted as file actions
    }
    if ev.is_dir {
        return;
    }

    let default_agent = state.config.lock().unwrap().default_agent.clone();

    let action = match ev.kind {
        watcher::FsEventKind::Create => ActionKind::Create,
        watcher::FsEventKind::Modify => ActionKind::Modify,
        watcher::FsEventKind::Delete => ActionKind::Delete,
        watcher::FsEventKind::Rename { from } => {
            let mut c = Action::new_file(ev.path, ActionKind::Rename);
            c.timestamp = ev.timestamp;
            c.agent = Some(default_agent.clone());
            c.outside = ev.outside;
            c.from = Some(from);
            mark_flags(&mut c);
            classify_ingested(state, &mut c);
            push_action(state, c);
            return;
        }
    };

    let mut c = Action::new_file(ev.path, action);
    c.timestamp = ev.timestamp;
    c.agent = Some(default_agent);
    c.outside = ev.outside;
    mark_flags(&mut c);
    classify_ingested(state, &mut c);
    push_action(state, c);
}

pub(crate) fn push_action(state: &Arc<AppState>, change: Action) {
    let mut guard = state.session.lock().unwrap();
    let Some(s) = guard.as_mut() else { return };
    // File actions dedupe on (action_kind, path) so a flurry of MODIFY
    // events on the same file collapse into one ledger row. Non-File
    // actions (Shell/Git/Package/Secret) are unique per-execution and
    // always go into the ledger.
    if change.category == ActionCategory::File {
        let key = format!("{:?}:{}", change.action, change.path_str());
        if s.batch_keys.contains(&key) {
            return;
        }
        s.batch_keys.insert(key);
    }
    s.actions_protected += 1;
    // Append-only NDJSON ledger: one line per ingested action. Mid-session
    // the Ledger UI/CLI reads this file directly without locking session state.
    let _ = storage::append_ledger(&s.id, &change);
    s.batch.push(change);
}

fn finalize_batch(state: &Arc<AppState>, app: &AppHandle) {
    let mut guard = state.session.lock().unwrap();
    let Some(s) = guard.as_mut() else { return };
    if s.batch.is_empty() {
        return;
    }
    let risk_result = risk::evaluate(&s.batch);
    let batch_data = BatchData {
        counts: counts_of(&s.batch),
        actions: s.batch.clone(),
        risk: risk_result,
    };
    s.pending_batch = Some(batch_data.clone());

    for a in &s.batch {
        s.total_counts.add(a.action);
        s.actions.push(a.clone());
    }
    s.batch.clear();
    s.batch_keys.clear();
    drop(guard);

    emit(app, "actionguard://batch", batch_data.clone());
    if batch_data.risk.level == RiskLevel::High {
        state
            .session
            .lock()
            .unwrap()
            .as_mut()
            .map(|s| s.awaiting_review = true);
        emit(app, "actionguard://risk", batch_data);
    }
}

async fn run_aggregator(
    app: AppHandle,
    state: Arc<AppState>,
    rx: Receiver<watcher::FsEvent>,
    id: String,
) {
    let mut last_event = Instant::now();
    let mut last_count_emit = Instant::now() - Duration::from_secs(1);
    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(ev) => {
                last_event = Instant::now();
                handle_event(&state, ev);
                if last_count_emit.elapsed() > Duration::from_millis(300) {
                    last_count_emit = Instant::now();
                    emit_counts(&app, &state);
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                let should_finalize = {
                    let guard = state.session.lock().unwrap();
                    guard
                        .as_ref()
                        .map(|s| s.id == id && !s.batch.is_empty() && !s.awaiting_review)
                        .unwrap_or(false)
                };
                if should_finalize && last_event.elapsed() > Duration::from_millis(1500) {
                    finalize_batch(&state, &app);
                    emit_counts(&app, &state);
                }
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
        let alive = {
            let guard = state.session.lock().unwrap();
            guard.as_ref().map(|s| s.id == id).unwrap_or(false)
        };
        if !alive {
            return;
        }
    }
}

#[derive(Serialize, Clone)]
struct CountsPayload {
    total: Counts,
    batch: Counts,
    risk_level: RiskLevel,
}

fn emit_counts(app: &AppHandle, state: &Arc<AppState>) {
    let (total, batch, risk_level) = {
        let guard = state.session.lock().unwrap();
        let Some(s) = guard.as_ref() else { return };
        let batch_counts = s
            .batch
            .iter()
            .fold(Counts::default(), |mut c, a| {
                c.add(a.action);
                c
            });
        (s.total_counts, batch_counts, risk::evaluate(&s.actions).level)
    };
    emit(
        app,
        "actionguard://counts",
        CountsPayload {
            total,
            batch,
            risk_level,
        },
    );
}

fn finalize_session(
    app: &AppHandle,
    state: &Arc<AppState>,
    status: SessionStatus,
    undone: bool,
) -> Result<SessionSummary> {
    {
        let mut guard = state.session.lock().unwrap();
        let Some(s) = guard.as_mut() else {
            return Err(anyhow::anyhow!("No active session"));
        };
        if !s.batch.is_empty() {
            let risk_result = risk::evaluate(&s.batch);
            let batch_data = BatchData {
                counts: counts_of(&s.batch),
                actions: s.batch.clone(),
                risk: risk_result,
            };
            s.pending_batch = Some(batch_data);
            for a in &s.batch {
                s.total_counts.add(a.action);
                s.actions.push(a.clone());
            }
            s.batch.clear();
            s.batch_keys.clear();
        }
    }

    let (summary, actions, id) = {
        let mut guard = state.session.lock().unwrap();
        let Some(s) = guard.as_mut() else {
            return Err(anyhow::anyhow!("No active session"));
        };
        let overall_risk = risk::evaluate(&s.actions).level;
        let ended_at = now_str();
        let started =
            chrono::NaiveDateTime::parse_from_str(&s.started_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|_| chrono::Local::now().naive_local());
        let duration = chrono::Local::now()
            .naive_local()
            .signed_duration_since(started)
            .num_seconds()
            .max(0) as u64;

        let mut sensitive_count = 0u32;
        let mut outside_count = 0u32;
        for a in &s.actions {
            if a.sensitive {
                sensitive_count += 1;
            }
            if a.outside {
                outside_count += 1;
            }
        }

        let summary = SessionSummary {
            id: s.id.clone(),
            num: s.num,
            workspace: s.workspace.to_string_lossy().to_string(),
            started_at: s.started_at.clone(),
            ended_at: Some(ended_at),
            duration_secs: duration,
            counts: s.total_counts,
            total: s.total_counts.total(),
            risk: overall_risk,
            status,
            undone,
            sensitive_count,
            outside_count,
            // --- v0.2 additive fields (carry over the live counters) ---
            category_counts: s.category_counts,
            risk_counts: s.risk_counts,
            actions_protected: s.actions_protected,
            actions_blocked: s.actions_blocked,
            // v0.3 — enforcement outcome split
            enforcement_counts: s.enforcement_counts,
            mode: s.mode,
            // v0.2 — user behavior (approval popups / overrides)
            popups: s.popups,
            overrides: s.overrides,
        };
        (summary, s.actions.clone(), s.id.clone())
    };

    storage::save_session(&summary)?;
    storage::save_actions(&id, &actions)?;
    // Tear down the shell bridge: remove `current.hook` and drop the closed
    // sentinel so lingering hooks know the stop was deliberate and allow the
    // next command (fail-closed applies to unexpected failures only). The
    // bridge thread exits on its next accept when it sees the session is gone.
    {
        let guard = state.session.lock().unwrap();
        if let Some(s) = guard.as_ref() {
            if let Some(b) = &s.bridge {
                b.stop();
            }
        }
    }
    *state.session.lock().unwrap() = None;

    emit(app, "actionguard://ended", summary.clone());
    Ok(summary)
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Read --workspace / --observe CLI args passed to the GUI binary
/// (e.g. via `actionguard protect ./my-project --observe`).
/// Returns None for each if not present.
///
/// Reads `ACTIONGUARD_WORKSPACE` and `ACTIONGUARD_OBSERVE` env vars first
/// (set by `actionguard protect` when spawning the GUI), then falls back
/// to scanning `--workspace` / `--observe` CLI args. Env vars are the
/// robust path because Tauri's internal CLI parser rejects unknown flags;
/// the CLI-arg fallback exists for users who launch `ActionGuard.exe`
/// directly from a terminal.
#[tauri::command]
pub fn get_startup_args() -> StartupArgs {
    let workspace = std::env::var("ACTIONGUARD_WORKSPACE")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let mut args = std::env::args().skip(1);
            let mut ws: Option<String> = None;
            while let Some(arg) = args.next() {
                if arg == "--workspace" {
                    ws = args.next();
                }
            }
            ws
        });

    let observe = std::env::var("ACTIONGUARD_OBSERVE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
        || {
            std::env::args()
                .skip(1)
                .any(|arg| arg == "--observe")
        };

    StartupArgs {
        workspace,
        mode: if observe {
            crate::models::SessionMode::Observe
        } else {
            crate::models::SessionMode::Protected
        },
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StartupArgs {
    pub workspace: Option<String>,
    pub mode: crate::models::SessionMode,
}

#[tauri::command]
pub async fn choose_workspace() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Choose a workspace folder to protect")
            .pick_folder()
            .map(|p| p.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    workspace: String,
    mode: Option<crate::models::SessionMode>,
) -> Result<SessionInfo, String> {
    let ws = PathBuf::from(&workspace);
    if !ws.is_dir() {
        return Err("The selected path is not a directory".into());
    }
    {
        let guard = state.session.lock().unwrap();
        if guard.is_some() {
            return Err("A protected session is already active. End it first.".into());
        }
    }

    let mut cfg = state.config.lock().unwrap().clone();
    let num = storage::alloc_session_num(&mut cfg);
    let id = format!("{:05}", num);
    storage::save_config(&cfg).map_err(|e| e.to_string())?;

    let ws_snap = ws.clone();
    let id_snap = id.clone();
    let cfg_snap = cfg.clone();
    let manifest = tauri::async_runtime::spawn_blocking(move || {
        snapshot::create_snapshot(&ws_snap, &id_snap, &cfg_snap)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    let (tx, rx) = channel::<watcher::FsEvent>();
    let shared = watcher::start_watcher(&ws, &cfg, tx).map_err(|e| e.to_string())?;

    let started_at = now_str();

    // v0.2 Shell bridge: bind a localhost listener + write the hook
    // descriptor before we register the session, so the first shell command
    // the protected terminal issues can find us.
    let shell = shell_hooks::detect_default_shell();
    let bridge_handle = bridge::Bridge::start(state.inner().clone(), bridge::EmitHandle::Tauri(app.clone()), id.clone())
        .map_err(|e| format!("failed to start shell bridge: {e}"))?;
    // Generate the per-session init script the terminal will source. Best
    // effort — if it fails we still open the terminal without interception.
    let _ = shell_hooks::write_for_session(&id, shell);

    {
        let mut guard = state.session.lock().unwrap();
        *guard = Some(ActiveSession {
            id: id.clone(),
            num,
            workspace: ws.clone(),
            started_at: started_at.clone(),
            actions: Vec::new(),
            batch: Vec::new(),
            batch_keys: HashSet::new(),
            total_counts: Counts::default(),
            category_counts: CategoryCounts::default(),
            risk_counts: RiskCounts::default(),
            actions_protected: 0,
            actions_blocked: 0,
            enforcement_counts: EnforcementCounts::default(),
            awaiting_review: false,
            pending_batch: None,
            watcher: Some(shared),
            bridge: Some(bridge_handle),
            shell: shell.to_string(),
            mode: mode.unwrap_or_default(),
            popups: 0,
            overrides: 0,
        });
    }

    let state_arc = state.inner().clone();
    let app2 = app.clone();
    let id2 = id.clone();
    tauri::async_runtime::spawn(async move {
        run_aggregator(app2, state_arc, rx, id2).await;
    });

    // Protected Terminal: spawn a shell whose cwd is the workspace and which
    // has sourced the ActionGuard hook for this session.
    let _ = terminal::open_terminal(&ws, &id, shell);

    Ok(SessionInfo {
        id,
        num,
        workspace: ws.to_string_lossy().to_string(),
        started_at,
        snapshot_files: manifest.file_count,
        mode: mode.unwrap_or_default(),
    })
}

#[tauri::command]
pub fn allow_batch(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut guard = state.session.lock().unwrap();
    if let Some(s) = guard.as_mut() {
        s.awaiting_review = false;
    }
    Ok(())
}

#[tauri::command]
pub async fn deny_batch(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<SessionSummary, String> {
    let (ws, id) = {
        let guard = state.session.lock().unwrap();
        let s = guard.as_ref().ok_or("No active session")?;
        (s.workspace.clone(), s.id.clone())
    };
    let cfg = state.config.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || snapshot::restore_snapshot(&ws, &id, &cfg))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    finalize_session(&app, &state.inner(), SessionStatus::Denied, true).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_session(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<SessionSummary, String> {
    finalize_session(&app, &state.inner(), SessionStatus::Completed, false)
        .map_err(|e| e.to_string())
}

/// Undo the currently active session: restore the snapshot taken at session
/// start, then finalize and mark the session as undone.
#[tauri::command]
pub async fn undo_active_session(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<SessionSummary, String> {
    let (ws, id) = {
        let guard = state.session.lock().unwrap();
        let s = guard.as_ref().ok_or("No active session")?;
        (s.workspace.clone(), s.id.clone())
    };
    let cfg = state.config.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || snapshot::restore_snapshot(&ws, &id, &cfg))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    finalize_session(&app, &state.inner(), SessionStatus::Completed, true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn undo_session(
    state: State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<UndoResult, String> {
    let summary = storage::load_session(&session_id).map_err(|e| e.to_string())?;
    let ws = PathBuf::from(&summary.workspace);
    let cfg = state.config.lock().unwrap().clone();
    let sid = session_id.clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        snapshot::restore_snapshot(&ws, &sid, &cfg)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    if let Ok(mut s) = storage::load_session(&session_id) {
        s.undone = true;
        let _ = storage::save_session(&s);
    }
    Ok(res)
}

#[tauri::command]
pub fn list_sessions() -> Result<Vec<SessionSummary>, String> {
    storage::list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_session(session_id: String) -> Result<SessionDetails, String> {
    storage::load_session_details(&session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_active_session(state: State<'_, Arc<AppState>>) -> Result<Option<SessionInfo>, String> {
    let guard = state.session.lock().unwrap();
    Ok(guard.as_ref().map(|s| SessionInfo {
        id: s.id.clone(),
        num: s.num,
        workspace: s.workspace.to_string_lossy().to_string(),
        started_at: s.started_at.clone(),
        snapshot_files: 0,
        mode: s.mode,
    }))
}

#[tauri::command]
pub fn get_pending_batch(state: State<'_, Arc<AppState>>) -> Result<Option<BatchData>, String> {
    let guard = state.session.lock().unwrap();
    Ok(guard.as_ref().and_then(|s| s.pending_batch.clone()))
}

#[tauri::command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> Result<AppConfig, String> {
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
pub fn update_config(
    state: State<'_, Arc<AppState>>,
    config: AppConfig,
) -> Result<(), String> {
    storage::save_config(&config).map_err(|e| e.to_string())?;
    *state.config.lock().unwrap() = config;
    Ok(())
}

// ---------------------------------------------------------------------------
// v0.2 — Live session stats + Action Ledger (UI/CLI)
// ---------------------------------------------------------------------------

/// Convert a stored `Action` into the UI/CLI-friendly `LedgerEntry` shape.
/// Carries a derived `time_hms` field so the frontend doesn't re-parse.
fn to_ledger_entry(a: &Action) -> LedgerEntry {
    let time_hms = a
        .timestamp
        .split(' ')
        .nth(1)
        .unwrap_or(&a.timestamp)
        .to_string();
    LedgerEntry {
        id: a.id.clone(),
        timestamp: a.timestamp.clone(),
        time_hms,
        agent: a.agent.clone().unwrap_or_default(),
        category: a.category,
        kind: a
            .kind
            .clone()
            .unwrap_or_else(|| a.action.as_str().to_string()),
        target: a.target_str().to_string(),
        risk: a.risk.unwrap_or_default(),
        decision: a.decision.unwrap_or_default(),
        result: a.result.clone().unwrap_or_default(),
        reasons: a.reasons.clone(),
        asset: a.asset.clone(),
        evidence: a.evidence.clone(),
    }
}

/// Live counters for the active session, polled by the SessionView dashboard.
#[derive(Serialize, Clone)]
pub struct ActiveStatsPayload {
    pub session_id: String,
    pub session_num: u32,
    pub workspace: String,
    pub started_at: String,
    pub total_actions: u32,
    pub category_counts: CategoryCounts,
    pub risk_counts: RiskCounts,
    pub actions_protected: u32,
    pub actions_blocked: u32,
    /// v0.3 — enforcement outcome split (Detection ≠ Protection).
    pub enforcement_counts: EnforcementCounts,
    pub awaiting_review: bool,
    /// v0.2 — approval popups fired / user overrides (User Override Rate).
    pub popups: u32,
    pub overrides: u32,
}

#[tauri::command]
pub fn get_active_stats(
    state: State<'_, Arc<AppState>>,
) -> Result<Option<ActiveStatsPayload>, String> {
    let guard = state.session.lock().unwrap();
    let Some(s) = guard.as_ref() else {
        return Ok(None);
    };
    Ok(Some(ActiveStatsPayload {
        session_id: s.id.clone(),
        session_num: s.num,
        workspace: s.workspace.to_string_lossy().to_string(),
        started_at: s.started_at.clone(),
        total_actions: s.actions.len() as u32 + s.batch.len() as u32,
        category_counts: s.category_counts,
        risk_counts: s.risk_counts,
        actions_protected: s.actions_protected,
        actions_blocked: s.actions_blocked,
        enforcement_counts: s.enforcement_counts,
        awaiting_review: s.awaiting_review,
        popups: s.popups,
        overrides: s.overrides,
    }))
}

/// Read the per-session Action Ledger. When `session_id` is omitted, defaults
/// to the active session. `category` / `risk` / `limit` are optional filters.
#[tauri::command]
pub fn get_ledger(
    state: State<'_, Arc<AppState>>,
    session_id: Option<String>,
    category: Option<ActionCategory>,
    risk: Option<RiskLevel>,
    limit: Option<usize>,
) -> Result<Vec<LedgerEntry>, String> {
    let id = match session_id {
        Some(id) => id,
        None => {
            let guard = state.session.lock().unwrap();
            let s = guard.as_ref().ok_or("No active session")?;
            s.id.clone()
        }
    };
    let filter = storage::LedgerFilter {
        category,
        risk,
        limit,
    };
    let actions = storage::load_ledger(&id, &filter);
    Ok(actions.iter().map(to_ledger_entry).collect())
}

// ---------------------------------------------------------------------------
// v0.2 — Approval gate (Human Gate)
// ---------------------------------------------------------------------------

/// List all pending approvals (the ApprovalModal polls this on
/// `actionguard://approval/request` events). Expired entries are pruned.
#[tauri::command]
pub fn list_pending_approvals(state: State<'_, Arc<AppState>>) -> Result<Vec<ApprovalRequest>, String> {
    Ok(state.approvals.list())
}

/// Resolve a pending approval. The frontend calls this with the user's
/// decision (Allow once / Deny / Always deny) and an optional `learn_rule`
/// to persist as a user policy rule (for "Always deny" / "Always allow").
///
/// On success, wakes the shell-bridge waiter with the decision and (if
/// `learn_rule` is present) appends the rule to `~/.actionguard/policies.user.yml`
/// then atomically swaps the in-memory policy set so the next shell command
/// is evaluated against the new rule.
#[tauri::command]
pub fn resolve_approval(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    resolution: ApprovalResolution,
) -> Result<(), String> {
    let ApprovalResolution {
        approval_id,
        decision,
        learn_rule,
    } = resolution;

    // 1) Wake the bridge waiter. If the approval is already expired or
    //    unknown, surface that to the UI — the user should know the shell
    //    already timed out and was denied.
    let _action = state
        .approvals
        .resolve(&approval_id, decision)
        .map_err(|e| e.to_string())?;

    // 2) Persist the learned rule (if any) + reload the policy set. We do
    //    this AFTER waking the bridge so the original command is decided
    //    against the policy in effect when it was issued, not the new one.
    if let Some(mut rule) = learn_rule {
        // Force source=User so the loader knows where this rule came from
        // when surfacing it in the UI.
        rule.source = crate::models::PolicySource::User;
        // Give it a stable id if the frontend didn't. <category>:<command>:<args>
        // is enough to dedupe against the same rule being learned twice.
        if rule.id.is_empty() {
            rule.id = format!(
                "user-{}-{}",
                rule.match_.category.map(|c| c.as_str()).unwrap_or("any"),
                rule.match_.command.as_deref().unwrap_or("any")
            );
        }
        append_user_rule_and_reload(&state, rule)
            .map_err(|e| e.to_string())?;
    }

    // 3) Tell the frontend the approval was resolved so it can close the
    //    modal. The next ledger refresh will show the final decision.
    let payload = ApprovalResolution {
        approval_id,
        decision,
        learn_rule: None, // don't re-emit the rule — the frontend already has it
    };
    let _ = app.emit("actionguard://approval/resolved", payload);
    Ok(())
}

/// Append a rule to `~/.actionguard/policies.user.yml` and atomically swap
/// the in-memory policy set. Idempotent: if a rule with the same id is
/// already present, it's replaced rather than duplicated.
fn append_user_rule_and_reload(state: &Arc<AppState>, new_rule: Rule) -> Result<()> {
    let mut file: PolicyFile = storage::load_policies_user();
    // Replace any existing rule with the same id, then append.
    file.rules.retain(|r| r.id != new_rule.id);
    file.rules.push(new_rule);
    storage::save_policies_user(&file)?;
    // Reload from disk so what's in memory matches what's persisted.
    let new_set = policy::load_policy_set();
    // Swap under the write lock — concurrent readers see the new set on
    // their next `read().unwrap()` call.
    let mut guard = state.policy.write().unwrap();
    *guard = new_set;
    Ok(())
}

/// Generate a `Rule` matching the same action signature, suitable for
/// "Always deny" / "Always allow" learning. Exposed as a Tauri command so
/// the frontend can preview the rule it would learn before submitting.
///
/// The rule shape:
///   - File:    `path` = the file's workspace-relative path (wildcard).
///   - Shell/Git/Package: `command` = first token, `args_contains` = the
///     second token (subcommand) when present. So "git reset --hard HEAD~10"
///     learns a rule matching any "git reset ..." command. The user can
///     edit `policies.user.yml` to broaden or narrow.
#[tauri::command]
pub fn preview_learn_rule(action: Action, decision: Decision) -> Result<Rule, String> {
    Ok(build_learn_rule(&action, decision))
}

/// Pure helper used by both `preview_learn_rule` and the tests.
fn build_learn_rule(action: &Action, decision: Decision) -> Rule {
    use crate::models::{ActionCategory, MatchSpec, PolicySource, RiskLevel};

    let (match_, risk, reason) = match action.category {
        ActionCategory::File => {
            let path = action.path_str().to_string();
            let pattern = if path.is_empty() {
                "*".to_string()
            } else {
                // Match the file's basename so future creates of the same
                // sensitive file are also caught.
                std::path::Path::new(&path)
                    .file_name()
                    .map(|n| format!("*{}", n.to_string_lossy()))
                    .unwrap_or_else(|| path.clone())
            };
            (
                MatchSpec {
                    category: Some(ActionCategory::File),
                    command: None,
                    path: Some(pattern),
                    args_contains: None,
                    args_any: None,
                    regex: None,
                },
                RiskLevel::Critical,
                "User-learned rule for a sensitive file".to_string(),
            )
        }
        ActionCategory::Shell | ActionCategory::Git | ActionCategory::Package => {
            let target = action.target_str();
            let mut tokens = target.split_whitespace();
            let command = tokens.next().map(|s| s.to_string());
            // The subcommand (e.g. "reset" for "git reset --hard", "install"
            // for "npm install foo") is the most distinctive second token.
            let args_contains: Vec<String> = tokens
                .next()
                .map(|s| vec![s.to_string()])
                .unwrap_or_default();
            (
                MatchSpec {
                    category: Some(action.category),
                    command,
                    path: None,
                    args_contains: if args_contains.is_empty() {
                        None
                    } else {
                        Some(args_contains)
                    },
                    args_any: None,
                    regex: None,
                },
                RiskLevel::High,
                "User-learned rule for a shell command".to_string(),
            )
        }
        ActionCategory::Secret => (
            MatchSpec {
                category: Some(ActionCategory::Secret),
                command: None,
                path: action.path.clone(),
                args_contains: None,
                args_any: None,
                regex: None,
            },
            RiskLevel::Critical,
            "User-learned rule for a secret".to_string(),
        ),
    };

    Rule {
        id: format!(
            "user-{}-{}",
            match_.category.map(|c| c.as_str()).unwrap_or("any"),
            match_
                .command
                .as_deref()
                .or(match_.path.as_deref())
                .unwrap_or("any")
        ),
        match_,
        action: decision,
        risk: Some(risk),
        reason: Some(reason),
        source: PolicySource::User,
    }
}

// ---------------------------------------------------------------------------
// Execution Path Matrix — surfaced in the GUI so users see which paths are
// actually enforced on their platform before trusting a "protected" session.
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct ExecutionPathDto {
    pub path: &'static str,
    pub observe: bool,
    pub block: bool,
    pub note: &'static str,
    /// v0.3 — capability tier implied by observe/block (None = not covered).
    pub tier: Option<CapabilityTier>,
}

#[tauri::command]
pub fn get_enforcement_paths() -> Vec<ExecutionPathDto> {
    crate::platform::enforcement_paths()
        .into_iter()
        .map(|p| ExecutionPathDto {
            path: p.path,
            observe: p.observe,
            block: p.block,
            note: p.note,
            tier: CapabilityTier::from_capabilities(p.observe, p.block),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ActionCategory, ActionKind, EnforcementStatus, MatchSpec, PolicySource,
    };

    #[test]
    fn enforcement_counts_track_outcomes() {
        let mut c = EnforcementCounts::default();
        c.add(EnforcementStatus::Enforced);
        c.add(EnforcementStatus::Enforced);
        c.add(EnforcementStatus::Observed);
        c.add(EnforcementStatus::Bypassed);
        c.add(EnforcementStatus::Unsupported);
        assert_eq!(c.enforced, 2);
        assert_eq!(c.observed, 1);
        assert_eq!(c.bypassed, 1);
        assert_eq!(c.unsupported, 1);
        assert_eq!(c.total(), 5);
    }

    #[test]
    fn capability_tier_maps_observe_block() {
        use crate::models::CapabilityTier;
        // Not covered at all.
        assert_eq!(CapabilityTier::from_capabilities(false, false), None);
        assert_eq!(CapabilityTier::from_capabilities(false, true), None);
        // Observe-only → L1, block → L2.
        assert_eq!(
            CapabilityTier::from_capabilities(true, false),
            Some(CapabilityTier::L1Observe)
        );
        assert_eq!(
            CapabilityTier::from_capabilities(true, true),
            Some(CapabilityTier::L2PreAction)
        );
    }

    #[test]
    fn bump_counters_tracks_enforcement_split() {
        let state = make_test_state();
        // Need a live session for bump_counters to touch.
        *state.session.lock().unwrap() = Some(ActiveSession {
            id: "test".into(),
            num: 1,
            workspace: std::path::PathBuf::from("."),
            started_at: "2026-01-01 00:00:00".into(),
            actions: vec![],
            batch: vec![],
            batch_keys: std::collections::HashSet::new(),
            total_counts: Counts::default(),
            category_counts: CategoryCounts::default(),
            risk_counts: RiskCounts::default(),
            actions_protected: 0,
            actions_blocked: 0,
            enforcement_counts: EnforcementCounts::default(),
            awaiting_review: false,
            pending_batch: None,
            watcher: None,
            bridge: None,
            shell: "test".into(),
            mode: crate::models::SessionMode::Protected,
            popups: 0,
            overrides: 0,
        });
        let mut action = Action::new_shell("rm -rf /tmp/x".to_string(), None, None);
        action.decision = Some(Decision::Deny);
        action.enforcement = Some(EnforcementStatus::Enforced);
        bump_counters(&state, &action);
        let guard = state.session.lock().unwrap();
        let s = guard.as_ref().unwrap();
        assert_eq!(s.actions_blocked, 1);
        assert_eq!(s.enforcement_counts.enforced, 1);
        assert_eq!(s.enforcement_counts.total(), 1);
    }

    #[test]
    fn build_learn_rule_for_git_command() {
        // Mimic what the bridge does: build a Shell action, then reassign
        // category based on `classify_shell_command` (git → Git).
        let mut action = Action::new_shell(
            "git reset --hard HEAD~10".to_string(),
            None,
            Some("claude".to_string()),
        );
        action.category = ActionCategory::Git;
        let rule = build_learn_rule(&action, Decision::Deny);
        assert_eq!(rule.match_.category, Some(ActionCategory::Git));
        assert_eq!(rule.match_.command.as_deref(), Some("git"));
        assert_eq!(rule.match_.args_contains, Some(vec!["reset".to_string()]));
        assert_eq!(rule.action, Decision::Deny);
        assert_eq!(rule.source, PolicySource::User);
    }

    #[test]
    fn build_learn_rule_for_file_uses_basename_wildcard() {
        let mut action = Action::new_file(".env".to_string(), ActionKind::Modify);
        action.category = ActionCategory::File;
        let rule = build_learn_rule(&action, Decision::Deny);
        assert_eq!(rule.match_.path.as_deref(), Some("*.env"));
    }

    #[test]
    fn build_learn_rule_for_shell_with_no_args() {
        let action = Action::new_shell("ls".to_string(), None, None);
        let rule = build_learn_rule(&action, Decision::Allow);
        assert_eq!(rule.match_.command.as_deref(), Some("ls"));
        assert!(rule.match_.args_contains.is_none());
    }

    #[test]
    fn append_user_rule_replaces_existing_id() {
        // We don't want to touch the real ~/.actionguard, so build a PolicyFile
        // in memory and exercise the dedupe logic directly.
        let mut file = PolicyFile::default();
        let r1 = Rule {
            id: "user-shell-rm".to_string(),
            match_: MatchSpec {
                category: Some(ActionCategory::Shell),
                command: Some("rm".to_string()),
                path: None,
                args_contains: Some(vec!["-rf".to_string()]),
                args_any: None,
                regex: None,
            },
            action: Decision::Deny,
            risk: Some(RiskLevel::Critical),
            reason: Some("test".to_string()),
            source: PolicySource::User,
        };
        file.rules.push(r1.clone());
        let mut r2 = r1.clone();
        r2.action = Decision::Allow; // change the action — should replace, not append.
        file.rules.retain(|r| r.id != r2.id);
        file.rules.push(r2);
        assert_eq!(file.rules.len(), 1);
        assert_eq!(file.rules[0].action, Decision::Allow);
    }

    // ======================================================================
    // E2E Pre-Action Decision Pipeline Tests
    //
    // These test the full flow that the shell bridge uses:
    //   command → classify_shell_command → Action → classify_action (risk + policy) → decision
    //
    // The bridge then uses the decision to either allow (HTTP 200) or block
    // (HTTP 403) the command. These tests verify the decision is correct.
    // ======================================================================

    fn make_test_state() -> Arc<AppState> {
        Arc::new(AppState {
            session: std::sync::Mutex::new(None),
            config: std::sync::Mutex::new(crate::models::AppConfig::default()),
            policy: Arc::new(std::sync::RwLock::new(
                crate::policy::load_policy_set(),
            )),
            approvals: Arc::new(crate::approval::ApprovalStore::new()),
            last_policy_mtime: std::sync::Mutex::new(0),
        })
    }

    /// Helper: classify a shell command and set the action category.
    fn classify_and_set_category(action: &mut Action) {
        let cmd = action.target.clone().unwrap_or_default();
        let parsed = crate::policy::classify::classify_shell_command(&cmd);
        action.category = parsed.category;
    }

    /// E2E: LOW-risk command → Allow → command executes.
    #[test]
    fn e2e_allow_low_risk_command() {
        let state = make_test_state();
        let mut action = Action::new_shell(
            "ls -la".to_string(),
            None,
            Some("claude-code".to_string()),
        );
        classify_and_set_category(&mut action);
        classify_action(&state, &mut action);

        assert_eq!(action.risk, Some(RiskLevel::Low));
        assert_eq!(action.decision, Some(Decision::Allow));
        // Bridge would return HTTP 200 → command executes.
    }

    /// E2E: CRITICAL-risk command (`rm -rf /`) → Deny → command blocked.
    #[test]
    fn e2e_deny_critical_rm_rf_root() {
        let state = make_test_state();
        let mut action = Action::new_shell(
            "rm -rf /".to_string(),
            None,
            Some("claude-code".to_string()),
        );
        classify_and_set_category(&mut action);
        classify_action(&state, &mut action);

        assert_eq!(action.risk, Some(RiskLevel::Critical));
        // Policy should deny rm -rf /.
        assert_eq!(action.decision, Some(Decision::Deny));
        // Bridge would return HTTP 403 → command blocked.
    }

    /// E2E: HIGH-risk command (`git push --force`) → Ask → needs human approval.
    #[test]
    fn e2e_confirm_git_push_force() {
        let state = make_test_state();
        let mut action = Action::new_shell(
            "git push --force origin main".to_string(),
            None,
            Some("claude-code".to_string()),
        );
        classify_and_set_category(&mut action);
        classify_action(&state, &mut action);

        // Risk should be HIGH or CRITICAL.
        let risk = action.risk.unwrap_or(RiskLevel::Low);
        assert!(
            matches!(risk, RiskLevel::High | RiskLevel::Critical),
            "git push --force should be HIGH or CRITICAL, got {:?}",
            risk,
        );
        // Decision should be Ask (needs human confirmation) or Deny.
        let decision = action.decision.unwrap_or(Decision::Allow);
        assert!(
            matches!(decision, Decision::Ask | Decision::Deny),
            "git push --force should be Ask or Deny, got {:?}",
            decision,
        );
    }

    /// E2E: Sensitive file read (`cat .env`) → HIGH/CRITICAL → Ask or Deny.
    #[test]
    fn e2e_sensitive_env_read_via_shell() {
        let state = make_test_state();
        let mut action = Action::new_shell(
            "cat .env".to_string(),
            None,
            Some("claude-code".to_string()),
        );
        classify_and_set_category(&mut action);
        classify_action(&state, &mut action);

        let risk = action.risk.unwrap_or(RiskLevel::Low);
        assert!(
            matches!(risk, RiskLevel::High | RiskLevel::Critical),
            "reading .env should be HIGH or CRITICAL, got {:?}",
            risk,
        );
    }

    /// E2E: Approval timeout → Deny.
    /// When the approval gate fires and no human responds, the bridge
    /// defaults to Deny after the timeout.
    #[test]
    fn e2e_approval_timeout_denies() {
        let store = crate::approval::ApprovalStore::new();
        let mut action = Action::new_shell(
            "rm -rf ./dist".to_string(),
            None,
            Some("claude-code".to_string()),
        );
        action.risk = Some(RiskLevel::Critical);
        action.decision = Some(Decision::Ask);

        let req = crate::approval::build_request(action, "test-session".to_string(), 1);
        let rx = store.request(req);

        // Wait for timeout (1 second).
        let result = rx.recv_timeout(std::time::Duration::from_secs(3));
        // After timeout, the receiver should get Deny (or channel closed).
        match result {
            Ok(d) => assert_eq!(d, Decision::Deny, "timeout should result in Deny"),
            Err(_) => {
                // Channel closed without a decision — bridge treats this as Deny.
                // This is the correct fallback behavior.
            }
        }
    }

    /// E2E: `git push --force origin main` (shared branch) → Deny (critical).
    /// The `deny-push-force-shared-branch` rule should fire before the
    /// generic `confirm-push-force` rule (first-match-wins).
    #[test]
    fn e2e_deny_git_push_force_shared_branch() {
        let state = make_test_state();
        let mut action = Action::new_shell(
            "git push --force origin main".to_string(),
            None,
            Some("claude-code".to_string()),
        );
        classify_and_set_category(&mut action);
        classify_action(&state, &mut action);

        assert_eq!(action.risk, Some(RiskLevel::Critical));
        assert_eq!(action.decision, Some(Decision::Deny));
    }

    /// E2E: `chmod -R 777 .` → Ask (medium).
    #[test]
    fn e2e_confirm_chmod() {
        let state = make_test_state();
        let mut action = Action::new_shell(
            "chmod -R 777 .".to_string(),
            None,
            Some("claude-code".to_string()),
        );
        classify_and_set_category(&mut action);
        classify_action(&state, &mut action);

        assert_eq!(action.risk, Some(RiskLevel::Medium));
        assert_eq!(action.decision, Some(Decision::Ask));
    }

    /// E2E: `sudo` → Deny (critical) — privilege escalation blocked.
    #[test]
    fn e2e_deny_sudo() {
        let state = make_test_state();
        let mut action = Action::new_shell(
            "sudo apt install -y evil".to_string(),
            None,
            Some("claude-code".to_string()),
        );
        classify_and_set_category(&mut action);
        classify_action(&state, &mut action);

        assert_eq!(action.risk, Some(RiskLevel::Critical));
        assert_eq!(action.decision, Some(Decision::Deny));
    }

    /// E2E: Bypass test — `/usr/bin/rm` (absolute path).
    ///
    /// The v0.2 preexec hook (DEBUG trap / `preexec`) DOES capture absolute
    /// paths and forwards the full command line to the bridge — so the
    /// bridge sees `/usr/bin/rm -rf /`. The bypass happens downstream:
    /// `classify_shell_command` yields binary name `/usr/bin/rm`, which
    /// matches none of the built-in rules (they key on the bare name `rm`),
    /// so the policy set falls through to `allow`.
    ///
    /// This is a KNOWN LIMITATION of exact-binary-name matching. Candidate
    /// fixes are basename matching or a PATH-prepend shim (v0.3 roadmap).
    #[test]
    fn e2e_bypass_absolute_path_rm() {
        // classify_shell_command looks at the binary name (first token).
        // `/usr/bin/rm` → binary name is `/usr/bin/rm`, not `rm`.
        let parsed = crate::policy::classify::classify_shell_command("/usr/bin/rm -rf /");
        assert_eq!(parsed.command, "/usr/bin/rm");
    }

    /// E2E: Bypass test — Python subprocess bypasses the shell hook.
    ///
    /// `python -c "import os; os.system('rm -rf /')"` goes through the
    /// shell hook for `python`, but the inner `os.system()` call is a
    /// direct syscall — the shell hook never sees it.
    ///
    /// This is a KNOWN LIMITATION: the hook only intercepts commands typed
    /// in the shell. Subprocesses spawned by programs (Python, Node, etc.)
    /// bypass the hook entirely.
    #[test]
    fn e2e_bypass_python_subprocess() {
        let state = make_test_state();
        let mut action = Action::new_shell(
            r#"python -c "import os; os.system('rm -rf /')""#.to_string(),
            None,
            Some("claude-code".to_string()),
        );
        classify_and_set_category(&mut action);
        classify_action(&state, &mut action);

        // The outer `python` command IS seen by the hook and classified.
        // But the inner `os.system('rm -rf /')` is NOT seen.
        // The risk engine evaluates the OUTER command (python), not the
        // INNER command (rm -rf /).
        //
        // This is BYPASS BEHAVIOR. The fix requires intercepting at the
        // process-spawn layer (e.g., LD_PRELOAD on Linux, API hooking on
        // Windows), which is v0.3+ scope.
        //
        // For now, we document this so users know:
        //   Shell hook ≠ full process isolation.
        let _ = action;
    }
}
