//! v0.2 Approval gate — the fourth layer of the ActionGuard safety stack.
//!
//! When the policy engine returns `Decision::Ask` (a rule with `action:
//! confirm`) or a HIGH/CRITICAL shell action has no covering rule, the shell
//! bridge blocks the command and asks the user to resolve it.
//!
//! The flow is:
//!   1. Bridge thread calls [`ApprovalStore::request`] with an
//!      [`ApprovalRequest`] describing the action.
//!   2. The store stashes a `mpsc::Sender` keyed by approval id and returns
//!      a `Receiver` the bridge will await (with timeout).
//!   3. The frontend emits an `actionguard://approval/request` event and the
//!      [`ApprovalModal`](../../src/components/ApprovalModal.vue) shows up.
//!   4. User clicks *Allow once* / *Deny* / *Always deny*.
//!   5. Frontend calls the `resolve_approval` Tauri command, which calls
//!      [`ApprovalStore::resolve`] — the bridge waiter is woken with the
//!      decision and the shell either proceeds or is blocked.
//!
//! Timeouts: if no resolution arrives within `timeout_secs`, the bridge
//! thread's `recv_timeout` fires, the `Receiver` is dropped, and the store
//! entry becomes stale. [`ApprovalStore::list`] prunes such entries on
//! each call so the UI never shows ghost approvals.

use crate::models::{Action, ApprovalRequest, Decision};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// One waiter per pending approval. The `Sender` is `Some` until either the
/// user resolves it (consumed by `resolve`) or it's pruned as stale.
struct Pending {
    request: ApprovalRequest,
    created_at: Instant,
    waiter: Option<Sender<Decision>>,
}

/// Global, app-wide store of pending approvals. Lives in [`AppState`] and is
/// shared between the bridge thread (which creates waiters) and the
/// `resolve_approval` Tauri command (which fulfills them).
#[derive(Default)]
pub struct ApprovalStore {
    pending: Mutex<HashMap<String, Pending>>,
}

impl ApprovalStore {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new approval request. Returns the receiver the bridge
    /// will await. Also prunes any expired entries opportunistically.
    pub fn request(&self, mut request: ApprovalRequest) -> Receiver<Decision> {
        let (tx, rx) = channel();
        let created_at = Instant::now();
        // Backfill the due-at timestamp if the caller left it blank.
        if request.decision_due_at.is_empty() {
            let due = chrono::Local::now()
                .checked_add_signed(chrono::Duration::seconds(request.timeout_secs as i64))
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();
            request.decision_due_at = due;
        }
        let id = request.id.clone();
        let mut guard = self.pending.lock().unwrap();
        prune_expired(&mut guard);
        guard.insert(
            id,
            Pending {
                request,
                created_at,
                waiter: Some(tx),
            },
        );
        rx
    }

    /// Fulfill a pending approval. Returns:
    ///   - `Ok(Some(action))` — approval found and resolved; the caller can
    ///     apply `learn_rule` if present.
    ///   - `Ok(None)` — approval was already expired/resolved (no-op).
    ///   - `Err(...)` — approval id unknown.
    ///
    /// The caller (resolve_approval command) is responsible for persisting
    /// any `learn_rule` to the user policy file and reloading the policy
    /// set; this method only wakes the bridge waiter.
    pub fn resolve(&self, id: &str, decision: Decision) -> Result<Option<Action>, &'static str> {
        let mut guard = self.pending.lock().unwrap();
        prune_expired(&mut guard);
        let Some(p) = guard.remove(id) else {
            return Err("approval not found or expired");
        };
        // If the waiter is already gone (timed out), the shell has already
        // moved on with Deny. Surface that so the UI can warn.
        if let Some(tx) = p.waiter {
            let _ = tx.send(decision);
        }
        Ok(Some(p.request.action))
    }

    /// Snapshot of all pending approvals, pruned of expired entries. Used by
    /// the frontend to populate the approval modal list.
    pub fn list(&self) -> Vec<ApprovalRequest> {
        let mut guard = self.pending.lock().unwrap();
        prune_expired(&mut guard);
        guard.values().map(|p| p.request.clone()).collect()
    }

    /// Drop all pending approvals (called on session teardown). Any bridge
    /// waiter still blocked will see its `Receiver` return `RecvError`,
    /// which the bridge interprets as Deny.
    #[allow(dead_code)] // session-teardown path, wired in Phase C
    pub fn clear(&self) {
        let mut guard = self.pending.lock().unwrap();
        guard.clear();
    }
}

/// Remove entries whose timeout has elapsed. Called from every public method
/// so the store is self-cleaning — no separate reaper thread needed.
fn prune_expired(pending: &mut HashMap<String, Pending>) {
    let now = Instant::now();
    pending.retain(|_, p| {
        let max = Duration::from_secs(p.request.timeout_secs.max(1) as u64);
        // The waiter being `None` means someone already resolved it but the
        // entry hasn't been removed yet — drop it.
        p.waiter.is_some() && now.duration_since(p.created_at) < max
    });
}

/// Convenience: a global default store for places that don't have direct
/// access to [`AppState`] (e.g. CLI mode). The Tauri app uses its own
/// `Arc<ApprovalStore>` in `AppState` instead.
#[allow(dead_code)] // CLI-mode entry point, exercised once approval lands in CLI
pub fn global() -> &'static ApprovalStore {
    static STORE: OnceLock<ApprovalStore> = OnceLock::new();
    STORE.get_or_init(ApprovalStore::new)
}

/// Build an [`ApprovalRequest`] from an action that needs human review.
/// The id is a fresh UUID; the timeout comes from the app config.
pub fn build_request(action: Action, session_id: String, timeout_secs: u32) -> ApprovalRequest {
    ApprovalRequest {
        id: uuid::Uuid::new_v4().to_string(),
        session_id,
        action,
        decision_due_at: String::new(),
        timeout_secs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Action, ActionKind};

    fn sample_action() -> Action {
        let mut a = Action::new_shell(
            "git reset --hard HEAD~10".to_string(),
            None,
            Some("claude".to_string()),
        );
        a.risk = Some(crate::models::RiskLevel::Critical);
        a.reasons = vec!["Rewrites repository state".to_string()];
        a
    }

    #[test]
    fn resolve_wakes_the_waiter() {
        let store = ApprovalStore::new();
        let action = sample_action();
        let req = build_request(action.clone(), "001".into(), 30);
        let rx = store.request(req);
        let resolved = store.resolve("nonexistent-id", Decision::Allow);
        assert!(resolved.is_err(), "unknown id should error");
        // The action is borrowed into the request, so we can't easily
        // compare by value, but we can check the waiter is fulfilled.
        let id = {
            let guard = store.pending.lock().unwrap();
            guard.keys().next().cloned().unwrap()
        };
        let outcome = store.resolve(&id, Decision::Allow).unwrap();
        assert!(outcome.is_some(), "action should be returned");
        // The receiver should now hold the decision.
        let got = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(got, Decision::Allow);
    }

    #[test]
    fn expired_entries_are_pruned() {
        let store = ApprovalStore::new();
        let action = sample_action();
        let req = build_request(action, "001".into(), 1);
        let _rx = store.request(req);
        assert_eq!(store.list().len(), 1);
        std::thread::sleep(Duration::from_millis(1100));
        assert!(store.list().is_empty(), "expired entry should be pruned");
    }

    #[test]
    fn clear_drops_all_waiters() {
        let store = ApprovalStore::new();
        let action = sample_action();
        let rx = store.request(build_request(action, "001".into(), 30));
        store.clear();
        // Receiver should report disconnect because the sender was dropped.
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(_) => panic!("should not receive after clear"),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
            Err(_) => panic!("expected Disconnected"),
        }
        let _ = ActionKind::Modify; // silence unused import in some configs
    }
}
