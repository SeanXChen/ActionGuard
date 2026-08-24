// ActionGuard v0.2 — Action Safety Layer (local engine)
//
// This binary is the developer-first CLI surface. It reads from ~/.actionguard/
// directly and does not require the Tauri GUI to be running. The full set of
// subcommands covers: setup, uninstall, doctor, status, policy check/list/lint/
// path/edit, session list/show, actions show, allow, deny, init-{bash,zsh,fish,
// powershell}, run, protect, stats, capabilities, boundary.
//
// For `allow` / `deny` / `run`, the CLI talks to the active session's bridge
// via HTTP (the same `current.hook` descriptor the shell hooks use). For
// everything else, it reads session/ledger/policy files directly off disk.

use actionguard_lib::models::{
    Action, ActionCategory, Decision, MatchSpec, PolicySource, RiskLevel, Rule,
};
use actionguard_lib::doctor;
use actionguard_lib::policy;
use actionguard_lib::risk;
use actionguard_lib::setup;
use actionguard_lib::shell_hooks;
use actionguard_lib::storage;

use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "actionguard",
    version,
    about = "ActionGuard v0.2 — Action Safety Layer (local engine)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// One-command install: detect OS/shell, preview changes, create
    /// ~/.actionguard, install built-in rules + shell hook, then self-check.
    Setup {
        /// Skip the confirmation prompt (CI / scripting).
        #[arg(long)]
        yes: bool,
    },
    /// Remove exactly what `setup` installed — marker block, hooks, and
    /// optionally the whole ~/.actionguard ledger.
    Uninstall {
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
    /// Detailed machine status: policy, shell hook, bridge, boundaries.
    Doctor {
        /// Also run the non-destructive end-to-end boundary test.
        #[arg(long)]
        test: bool,
    },
    /// Show whether a session is active and where the hook socket is.
    Status,
    /// Dry-run the risk engine + policy on a command. No execution.
    PolicyCheck {
        cmd: String,
        /// Print the full reasoning behind the decision.
        #[arg(long)]
        explain: bool,
    },
    /// List all loaded rules (builtin + user).
    PolicyList,
    /// Validate a YAML rules file.
    PolicyLint { file: PathBuf },
    /// Print the path to the user policies file.
    PolicyPath,
    /// Edit the user policies file in $EDITOR.
    PolicyEdit,
    /// Session subcommands.
    Session {
        #[command(subcommand)]
        cmd: SessionCmd,
    },
    /// Actions subcommands.
    Actions {
        #[command(subcommand)]
        cmd: ActionsCmd,
    },
    /// Allow a pending approval (interactive picker if no id).
    Allow { id: Option<String> },
    /// Deny a pending approval. `--always` learns a deny rule for the future.
    Deny {
        id: Option<String>,
        #[arg(long)]
        always: bool,
    },
    /// Print the bash hook script to stdout.
    InitBash,
    /// Print the zsh hook script to stdout.
    InitZsh,
    /// Print the fish hook script to stdout.
    InitFish,
    /// Print the PowerShell hook script to stdout.
    InitPowershell,
    /// Wrap a single command with the safety layer (no shell needed).
    Run { cmd: Vec<String> },
    /// Start a protected session for a workspace (spawns the GUI if available).
    Protect {
        /// Workspace path to protect.
        workspace: PathBuf,
        /// Start in Observe mode (record only, never block).
        #[arg(long)]
        observe: bool,
    },
    /// Aggregate metric — actions protected across all sessions.
    Stats {
        /// Write the full report as JSON to this path (local validation).
        #[arg(long)]
        export: Option<PathBuf>,
    },
    /// Capability Tier Model — what ActionGuard can actually do on each path
    /// (L1 observe … L4 system), plus the local execution-path matrix.
    Capabilities,
    /// Boundary Registry — list / test the local action boundaries.
    Boundary {
        #[command(subcommand)]
        cmd: BoundaryCmd,
    },
    /// Community rule ecosystem — search installed rules or install a rule
    /// YAML file into the user policy.
    Rule {
        #[command(subcommand)]
        cmd: RuleCmd,
    },
}

#[derive(Subcommand)]
enum SessionCmd {
    /// List all sessions.
    List,
    /// Show details of one session.
    Show { id: String },
}

#[derive(Subcommand)]
enum ActionsCmd {
    /// Show actions for a session with optional filters.
    Show {
        session: String,
        #[arg(long)]
        category: Option<String>,
        #[arg(long, value_delimiter = ',')]
        risk: Option<Vec<String>>,
        #[arg(long, default_value = "50")]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum BoundaryCmd {
    /// List detected action boundaries on this machine.
    List,
    /// Verify a boundary end-to-end (or all boundaries when no name given).
    Test {
        /// Boundary name to test (substring match).
        name: Option<String>,
    },
}

#[derive(Subcommand)]
enum RuleCmd {
    /// Search loaded rules (builtin + user) by keyword.
    Search { query: String },
    /// Install a community rule YAML file into the user policy.
    Install { file: PathBuf },
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Some(Cmd::Setup { yes }) => std::process::exit(setup::run_setup(yes)),
        Some(Cmd::Uninstall { yes }) => std::process::exit(setup::run_uninstall(yes)),
        Some(Cmd::Doctor { test }) => std::process::exit(doctor::run_doctor(test)),
        None | Some(Cmd::Status) => status(),
        Some(Cmd::PolicyCheck { cmd, explain }) => policy_check(&cmd, explain),
        Some(Cmd::PolicyList) => policy_list(),
        Some(Cmd::PolicyLint { file }) => policy_lint(&file),
        Some(Cmd::PolicyPath) => policy_path(),
        Some(Cmd::PolicyEdit) => policy_edit(),
        Some(Cmd::Session { cmd }) => match cmd {
            SessionCmd::List => session_list(),
            SessionCmd::Show { id } => session_show(&id),
        },
        Some(Cmd::Actions { cmd }) => match cmd {
            ActionsCmd::Show {
                session,
                category,
                risk,
                limit,
            } => actions_show(&session, category.as_deref(), risk.as_deref(), limit),
        },
        Some(Cmd::Allow { id }) => allow(id.as_deref()),
        Some(Cmd::Deny { id, always }) => deny(id.as_deref(), always),
        Some(Cmd::InitBash) => print_init("bash"),
        Some(Cmd::InitZsh) => print_init("zsh"),
        Some(Cmd::InitFish) => print_init("fish"),
        Some(Cmd::InitPowershell) => print_init("powershell"),
        Some(Cmd::Run { cmd }) => run(&cmd.join(" ")),
        Some(Cmd::Protect { workspace, observe }) => protect(&workspace, observe),
        Some(Cmd::Stats { export }) => stats(export.as_deref()),
        Some(Cmd::Capabilities) => capabilities(),
        Some(Cmd::Boundary { cmd }) => match cmd {
            BoundaryCmd::List => boundary_list(),
            BoundaryCmd::Test { name } => boundary_test(name.as_deref()),
        },
        Some(Cmd::Rule { cmd }) => match cmd {
            RuleCmd::Search { query } => rule_search(&query),
            RuleCmd::Install { file } => rule_install(&file),
        },
    }
}

// ---------------------------------------------------------------------------
// status — print active session + hook socket location
// ---------------------------------------------------------------------------

/// Print the Execution Path Matrix for the current platform.
fn print_enforcement_paths() {
    println!();
    println!("Execution Path Matrix (this platform):");
    println!("  {:<42} {:<8} {:<8} {}", "path", "observe", "block", "note");
    println!("  {:<42} {:<8} {:<8} {}", "────", "───────", "─────", "────");
    for p in actionguard_lib::platform::enforcement_paths() {
        println!(
            "  {:<42} {:<8} {:<8} {}",
            p.path,
            p.yes_no(p.observe),
            p.yes_no(p.block),
            p.note
        );
    }
    println!();
}

fn status() {
    let link = storage::current_hook_symlink();
    match read_hook_descriptor() {
        Ok((port, secret, raw)) => {
            println!("actionguard v0.2.0");
            println!("active session:   yes");
            println!("hook descriptor:  {}", link.display());
            println!("port:             {port}");
            println!("secret:           {secret}");
            // Try to find session id from the descriptor's sibling .json files.
            if let Some(id) = infer_session_id(&raw) {
                println!("session id:       {id}");
            }
            // Probe the bridge.
            match http_get(port, &secret, "/status") {
                Ok(body) => {
                    println!("bridge:           ok — {body}");
                }
                Err(e) => {
                    println!("bridge:           unreachable ({e})");
                }
            }
        }
        Err(e) => {
            println!("actionguard v0.2.0");
            println!("active session:   no");
            println!("reason:           {e}");
        }
    }
    print_enforcement_paths();
}

/// Read `current.hook` and return (port, secret, raw_body).
fn read_hook_descriptor() -> Result<(u16, String, String), String> {
    let link = storage::current_hook_symlink();
    let raw = std::fs::read_to_string(&link)
        .map_err(|e| format!("no active hook ({e})"))?;
    let mut lines = raw.lines();
    let port: u16 = lines
        .next()
        .ok_or("hook file empty (no port line)")?
        .trim()
        .parse()
        .map_err(|e: std::num::ParseIntError| format!("bad port: {e}"))?;
    let secret = lines
        .next()
        .ok_or("hook file missing secret line")?
        .trim()
        .to_string();
    Ok((port, secret, raw))
}

/// Best-effort: derive the session id from the symlink target path. On
/// Windows `current.hook` is a copy, so this returns None and the user
/// falls back to reading `current.hook`'s sibling `.json` files.
fn infer_session_id(_raw: &str) -> Option<String> {
    let dir = storage::sessions_dir();
    let entries = std::fs::read_dir(&dir).ok()?;
    // The most recent `<id>.hook` file is almost certainly the active one.
    let mut best: Option<(std::time::SystemTime, String)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str())?;
        if ext != "hook" {
            continue;
        }
        let stem = path.file_stem()?.to_str()?.to_string();
        let mtime = entry.metadata().ok()?.modified().ok()?;
        if best.as_ref().map(|(t, _)| mtime > *t).unwrap_or(true) {
            best = Some((mtime, stem));
        }
    }
    best.map(|(_, id)| id)
}

// ---------------------------------------------------------------------------
// policy-check — dry-run classify + decide on a command, no exec
// ---------------------------------------------------------------------------

fn policy_check(cmd: &str, explain: bool) {
    let set = policy::load_policy_set();
    let mut action = Action::new_shell(cmd.to_string(), None, Some("cli".to_string()));
    let parsed = policy::classify::classify_shell_command(cmd);
    action.category = parsed.category;
    action.kind = Some(policy::classify::kind_for(&parsed).to_string());

    // Stamp the risk engine first (so the printed risk reflects reality).
    let r = risk::evaluate_action(&action);
    action.risk = Some(r.level);
    action.reasons = r.reasons.clone();
    if action.asset.is_none() {
        if let Some(a) = r.asset {
            action.asset = Some(a);
        }
    }

    let decision = policy::decide(&action, &set);

    if explain {
        // --explain mode: developer-friendly reasoning output.
        println!("Command:       {cmd}");
        println!("Category:      {}", action.category.as_str());
        println!();
        println!("Decision:      {}", decision_str(decision.decision).to_uppercase());
        println!("Risk:          {}", decision.risk.as_str().to_uppercase());
        println!("Matched:       {}", decision.matched_rule.as_deref().unwrap_or("(no rule)"));
        println!("Reason:        {}", if decision.reason.is_empty() { "(none)" } else { &decision.reason });
        println!();

        if decision.decision == Decision::Allow {
            println!("→ ActionGuard would ALLOW this command to run.");
        } else if decision.decision == Decision::Deny {
            println!("→ ActionGuard would DENY this command.");
        } else {
            println!("→ ActionGuard would PAUSE this command for human approval.");
        }

        // Show the risk engine's independent assessment.
        if !action.reasons.is_empty() {
            println!();
            println!("Risk engine:   {}", action.reasons.join("; "));
        }

        // Show the override path.
        println!();
        let override_cmd = match decision.decision {
            Decision::Allow | Decision::Ask => "actionguard allow",
            Decision::Deny => "actionguard deny",
        };
        println!("Override:      {override_cmd}");
        println!("Learn rule:    actionguard deny --always \"{cmd}\"");
    } else {
        // Standard mode: concise output.
        println!("command:        {cmd}");
        println!("category:       {}", action.category.as_str());
        println!("risk:           {}", decision.risk.as_str());
        println!("decision:       {}", decision_str(decision.decision));
        if let Some(rule_id) = &decision.matched_rule {
            println!("matched rule:   {rule_id}");
        }
        if !decision.reason.is_empty() {
            println!("reason:         {}", decision.reason);
        }
        if !action.reasons.is_empty() && action.reasons[0] != decision.reason {
            println!("risk reasons:   {}", action.reasons.join("; "));
        }
    }
}

fn decision_str(d: Decision) -> &'static str {
    match d {
        Decision::Allow => "allow",
        Decision::Ask => "confirm",
        Decision::Deny => "deny",
    }
}

// ---------------------------------------------------------------------------
// policy-list — print all loaded rules
// ---------------------------------------------------------------------------

fn policy_list() {
    let set = policy::load_policy_set();
    if set.rules.is_empty() {
        println!("(no rules loaded)");
        return;
    }
    println!("{:<32} {:<8} {:<10} {:<40} {}", "ID", "SRC", "ACTION", "MATCH", "REASON");
    for r in &set.rules {
        let src = match r.source {
            PolicySource::Builtin => "builtin",
            PolicySource::User => "user",
            PolicySource::Project => "project",
        };
        let action = decision_str(r.action);
        let m = match_spec_summary(&r.match_);
        let reason = r.reason.clone().unwrap_or_default();
        println!("{:<32} {:<8} {:<10} {:<40} {}", r.id, src, action, m, reason);
    }
}

fn match_spec_summary(m: &MatchSpec) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(c) = m.category {
        parts.push(format!("cat={}", c.as_str()));
    }
    if let Some(cmd) = &m.command {
        parts.push(format!("cmd={cmd}"));
    }
    if let Some(p) = &m.path {
        parts.push(format!("path={p}"));
    }
    if let Some(args) = &m.args_contains {
        parts.push(format!("args=[{}]", args.join(",")));
    }
    if let Some(args) = &m.args_any {
        parts.push(format!("args-any=[{}]", args.join(",")));
    }
    if let Some(re) = &m.regex {
        parts.push(format!("regex=/{re}/"));
    }
    if parts.is_empty() {
        "(any)".to_string()
    } else {
        parts.join(" ")
    }
}

// ---------------------------------------------------------------------------
// policy-lint — validate a YAML rules file
// ---------------------------------------------------------------------------

fn policy_lint(path: &PathBuf) {
    match policy::lint_file(path) {
        Ok(parsed) => {
            println!("ok — {} rule(s) in {}", parsed.rules.len(), path.display());
            for r in &parsed.rules {
                println!("  - {}  (action: {})", r.id, decision_str(r.action));
            }
        }
        Err(e) => {
            eprintln!("lint failed: {e}");
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// policy-path — print path to policies.user.yml
// ---------------------------------------------------------------------------

fn policy_path() {
    println!("{}", storage::user_policy_path().display());
}

// ---------------------------------------------------------------------------
// policy-edit — open policies.user.yml in $EDITOR
// ---------------------------------------------------------------------------

fn policy_edit() {
    let path = storage::user_policy_path();
    if !path.exists() {
        // Seed it with an empty-but-valid template.
        let template = "version: 1\nscope: user\nrules: []\n";
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, template);
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = std::process::Command::new(&editor)
        .arg(&path)
        .status();
    match status {
        Ok(s) if s.success() => println!("ok — {path:?} saved"),
        Ok(s) => {
            eprintln!("editor exited with {s}");
            std::process::exit(s.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("could not launch $EDITOR ({editor}): {e}");
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// rule search / install — community rule ecosystem
// ---------------------------------------------------------------------------

fn rule_search(query: &str) {
    use actionguard_lib::models::PolicySource;

    let set = policy::load_policy_set();
    let q = query.to_lowercase();
    let hits: Vec<_> = set
        .rules
        .iter()
        .filter(|r| {
            r.id.to_lowercase().contains(&q)
                || r.reason
                    .as_deref()
                    .map(|s| s.to_lowercase().contains(&q))
                    .unwrap_or(false)
                || match_spec_summary(&r.match_).to_lowercase().contains(&q)
        })
        .collect();
    if hits.is_empty() {
        println!("(no rules match '{query}')");
        println!("tip: search matches rule id, reason and match spec across builtin + user rules.");
        println!("tip: community packs arrive via `actionguard rule install <file.yml>`.");
        return;
    }
    println!("{:<32} {:<8} {:<10} {:<40} {}", "ID", "SRC", "ACTION", "MATCH", "REASON");
    for r in &hits {
        let src = match r.source {
            PolicySource::Builtin => "builtin",
            PolicySource::User => "user",
            PolicySource::Project => "project",
        };
        let action = decision_str(r.action);
        let m = match_spec_summary(&r.match_);
        let reason = r.reason.clone().unwrap_or_default();
        println!("{:<32} {:<8} {:<10} {:<40} {}", r.id, src, action, m, reason);
    }
    println!();
    println!("{} rule(s) match '{query}'", hits.len());
}

fn rule_install(file: &PathBuf) {
    use actionguard_lib::models::PolicySource;

    let parsed = match policy::lint_file(file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("rule install failed — invalid rule file: {e}");
            std::process::exit(1);
        }
    };
    if parsed.rules.is_empty() {
        eprintln!("rule install: no rules found in {}", file.display());
        std::process::exit(1);
    }
    let mut user = storage::load_policies_user();
    let mut added: Vec<String> = Vec::new();
    let mut replaced: Vec<String> = Vec::new();
    for mut r in parsed.rules {
        r.source = PolicySource::User;
        if user.rules.iter().any(|x| x.id == r.id) {
            user.rules.retain(|x| x.id != r.id);
            replaced.push(r.id.clone());
        } else {
            added.push(r.id.clone());
        }
        user.rules.push(r);
    }
    if let Err(e) = storage::save_policies_user(&user) {
        eprintln!("rule install: could not write {}: {e}", storage::user_policy_path().display());
        std::process::exit(1);
    }
    println!("ok — installed to {}", storage::user_policy_path().display());
    if !added.is_empty() {
        println!("  added:    {}", added.join(", "));
    }
    if !replaced.is_empty() {
        println!("  replaced: {}", replaced.join(", "));
    }
    println!("  rules are active on the next policy load (CLI reloads every command; GUI watches the file).");
}

// ---------------------------------------------------------------------------
// session list / show
// ---------------------------------------------------------------------------

fn session_list() {
    match storage::list_sessions() {
        Ok(sessions) if sessions.is_empty() => {
            println!("(no sessions recorded yet)");
        }
        Ok(sessions) => {
            println!("{:<8} {:<24} {:<10} {:<10} {:<8} {:<8} {:<6} {}", "#", "STARTED", "STATUS", "MODE", "TOTAL", "BLOCKED", "RISK", "WORKSPACE");
            for s in sessions {
                println!("{:<8} {:<24} {:<10} {:<10} {:<8} {:<8} {:<6} {}",
                    format!("#{:05}", s.num),
                    s.started_at,
                    session_status_str(s.status),
                    s.mode.as_str(),
                    s.total,
                    s.actions_blocked,
                    s.risk.as_str(),
                    s.workspace,
                );
            }
        }
        Err(e) => {
            eprintln!("list sessions failed: {e}");
            std::process::exit(1);
        }
    }
}

fn session_status_str(s: actionguard_lib::models::SessionStatus) -> &'static str {
    use actionguard_lib::models::SessionStatus;
    match s {
        SessionStatus::Active => "active",
        SessionStatus::Completed => "completed",
        SessionStatus::Denied => "denied",
    }
}

fn session_show(id: &str) {
    match storage::load_session(id) {
        Ok(s) => {
            println!("session:       #{:05}", s.num);
            println!("id:            {id}");
            println!("workspace:     {}", s.workspace);
            println!("started:       {}", s.started_at);
            if let Some(ended) = &s.ended_at {
                println!("ended:         {ended}");
            }
            println!("duration:      {}s", s.duration_secs);
            println!("status:        {}", session_status_str(s.status));
            println!("mode:          {}", s.mode.as_str());
            println!("total actions: {}", s.total);
            println!("undone:        {}", s.undone);
            println!("detected:      {}  (recorded across all boundaries)", s.actions_protected);
            println!("blocked:       {}  (deny decisions)", s.actions_blocked);
            println!();
            println!("enforcement (Detection ≠ Protection):");
            println!("  enforced:     {}  (actually stopped before execution)", s.enforcement_counts.enforced);
            println!("  observed:     {}  (recorded, could not block)", s.enforcement_counts.observed);
            println!("  bypassed:     {}  (execution bypassed the boundary)", s.enforcement_counts.bypassed);
            println!("  unsupported:  {}  (path not covered)", s.enforcement_counts.unsupported);
            println!();
            println!("category counts:");
            println!("  file:     {}", s.category_counts.file);
            println!("  shell:    {}", s.category_counts.shell);
            println!("  git:      {}", s.category_counts.git);
            println!("  package:  {}", s.category_counts.package);
            println!("  secret:   {}", s.category_counts.secret);
            println!();
            println!("risk counts:");
            println!("  low:       {}", s.risk_counts.low);
            println!("  medium:    {}", s.risk_counts.medium);
            println!("  high:      {}", s.risk_counts.high);
            println!("  critical:  {}", s.risk_counts.critical);
        }
        Err(e) => {
            eprintln!("session {id} not found: {e}");
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// actions show — print ledger entries
// ---------------------------------------------------------------------------

fn actions_show(session: &str, category: Option<&str>, risk: Option<&[String]>, limit: usize) {
    let mut filter = storage::LedgerFilter::default();
    if let Some(c) = category {
        filter.category = parse_category(c);
    }
    if let Some(risks) = risk {
        // Use the FIRST risk level as the filter (multi-risk filter is v0.3).
        if let Some(first) = risks.first() {
            if let Some(rl) = parse_risk(first) {
                filter.risk = Some(rl);
            }
        }
    }
    filter.limit = Some(limit);

    let actions = storage::load_ledger(session, &filter);
    if actions.is_empty() {
        println!("(no actions match the filter for session {session})");
        return;
    }
    println!("{:<21} {:<14} {:<10} {:<40} {:<10} {:<10} {}",
        "TIME", "AGENT", "KIND", "TARGET", "RISK", "RESULT", "REASONS");
    for a in actions {
        let target = a.target_str();
        let target_short = if target.len() > 40 { &target[..40] } else { target };
        println!("{:<21} {:<14} {:<10} {:<40} {:<10} {:<10} {}",
            a.timestamp,
            a.agent.as_deref().unwrap_or("—"),
            a.kind.as_deref().unwrap_or(""),
            target_short,
            a.risk.map(|r| r.as_str()).unwrap_or("—"),
            a.result.as_deref().unwrap_or("—"),
            a.reasons.join("; "),
        );
    }
}

fn parse_category(s: &str) -> Option<ActionCategory> {
    match s.to_lowercase().as_str() {
        "file" => Some(ActionCategory::File),
        "shell" => Some(ActionCategory::Shell),
        "git" => Some(ActionCategory::Git),
        "package" => Some(ActionCategory::Package),
        "secret" => Some(ActionCategory::Secret),
        _ => None,
    }
}

fn parse_risk(s: &str) -> Option<RiskLevel> {
    match s.to_lowercase().as_str() {
        "low" => Some(RiskLevel::Low),
        "medium" => Some(RiskLevel::Medium),
        "high" => Some(RiskLevel::High),
        "critical" => Some(RiskLevel::Critical),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// allow / deny — headless approval resolution via POST /resolve
// ---------------------------------------------------------------------------

fn allow(id: Option<&str>) {
    resolve_approval(id, Decision::Allow, false);
}

fn deny(id: Option<&str>, always: bool) {
    resolve_approval(id, Decision::Deny, always);
}

fn resolve_approval(id: Option<&str>, decision: Decision, always: bool) {
    let (port, secret, _) = match read_hook_descriptor() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("actionguard: no active session ({e})");
            std::process::exit(2);
        }
    };

    // If no id given, list pending and pick the first (or prompt).
    let approval_id = match id {
        Some(id) => id.to_string(),
        None => {
            let body = match http_get(port, &secret, "/pending") {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("actionguard: could not fetch pending approvals ({e})");
                    std::process::exit(2);
                }
            };
            let pending: Vec<ApprovalRequestCli> = match serde_json::from_str(&body) {
                Ok(p) => p,
                Err(_) => Vec::new(),
            };
            if pending.is_empty() {
                println!("(no pending approvals)");
                return;
            }
            if pending.len() == 1 {
                pending[0].id.clone()
            } else {
                println!("pending approvals:");
                for (i, p) in pending.iter().enumerate() {
                    println!("  [{}] {} — {} ({})",
                        i,
                        p.id,
                        p.action.target.as_deref().unwrap_or(""),
                        p.action.risk.map(|r| r.as_str()).unwrap_or("—"),
                    );
                }
                print!("pick [0-{}]: ", pending.len() - 1);
                let _ = std::io::stdout().flush();
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_err() {
                    eprintln!("invalid input");
                    std::process::exit(2);
                }
                let idx: usize = input.trim().parse().unwrap_or(usize::MAX);
                if idx >= pending.len() {
                    eprintln!("out of range");
                    std::process::exit(2);
                }
                pending[idx].id.clone()
            }
        }
    };

    // Build the resolution. For `deny --always`, attach a learn_rule that
    // matches the action's first token (so future commands with the same
    // first token are denied without prompting).
    let mut learn_rule: Option<Rule> = None;
    if always {
        // Fetch the pending action to derive the rule from it.
        let body = http_get(port, &secret, "/pending").unwrap_or_default();
        let pending: Vec<ApprovalRequestCli> = serde_json::from_str(&body).unwrap_or_default();
        if let Some(p) = pending.iter().find(|p| p.id == approval_id) {
            let target = p.action.target.as_deref().unwrap_or("");
            let first_token = target.split_whitespace().next().unwrap_or("");
            if !first_token.is_empty() {
                learn_rule = Some(Rule {
                    id: String::new(),
                    match_: MatchSpec {
                        category: Some(p.action.category),
                        command: Some(first_token.to_string()),
                        path: None,
                        args_contains: None,
                        args_any: None,
                        regex: None,
                    },
                    action: Decision::Deny,
                    risk: Some(RiskLevel::High),
                    reason: Some("Always denied via actionguard".to_string()),
                    source: PolicySource::User,
                });
            }
        }
    }

    let resolution = ApprovalResolutionCli {
        approval_id: approval_id.clone(),
        decision,
        learn_rule,
    };
    let body = serde_json::to_string(&resolution).unwrap_or_default();
    match http_post(port, &secret, "/resolve", &body) {
        Ok(resp) => {
            println!("resolved approval {approval_id} → {} ({resp})", decision_str(decision));
        }
        Err(e) => {
            eprintln!("actionguard: resolve failed ({e})");
            std::process::exit(2);
        }
    }
}

// Local DTOs mirroring the Rust models. We avoid depending on the `tauri`
// re-exports by re-declaring just the fields the CLI reads.
#[derive(Debug, Deserialize)]
struct ApprovalRequestCli {
    id: String,
    action: ActionCli,
}

#[derive(Debug, Deserialize)]
struct ActionCli {
    category: ActionCategory,
    target: Option<String>,
    risk: Option<RiskLevel>,
}

#[derive(Debug, serde::Serialize)]
struct ApprovalResolutionCli {
    approval_id: String,
    decision: Decision,
    learn_rule: Option<Rule>,
}

// ---------------------------------------------------------------------------
// init-* — print shell hook script to stdout
// ---------------------------------------------------------------------------

fn print_init(shell: &str) {
    print!("{}", shell_hooks::generate(shell));
}

// ---------------------------------------------------------------------------
// run — wrap a single command with the safety layer
// ---------------------------------------------------------------------------

fn run(cmd: &str) {
    // 1) Classify in-process so we can print a preview regardless of whether
    //    a session is active.
    let set = policy::load_policy_set();
    let mut action = Action::new_shell(cmd.to_string(), None, Some("cli".to_string()));
    let parsed = policy::classify::classify_shell_command(cmd);
    action.category = parsed.category;
    action.kind = Some(policy::classify::kind_for(&parsed).to_string());
    let r = risk::evaluate_action(&action);
    action.risk = Some(r.level);
    if action.asset.is_none() {
        action.asset = r.asset;
    }
    let decision = policy::decide(&action, &set);

    eprintln!("actionguard: {cmd}");
    eprintln!("  category: {}, risk: {}, decision: {}",
        action.category.as_str(),
        decision.risk.as_str(),
        decision_str(decision.decision));
    if let Some(rule_id) = &decision.matched_rule {
        eprintln!("  matched: {rule_id}");
    }

    // 2) If there's an active session, ask the bridge (it may block for
    //    approval). Otherwise, exec if allow / confirm; refuse if deny.
    let bridge_decision = if let Ok((port, secret, _)) = read_hook_descriptor() {
        match ask_bridge(port, &secret, cmd) {
            Ok(bridge_resp) => bridge_resp,
            Err(e) => {
                eprintln!("actionguard: bridge unreachable ({e}) — falling back to in-process decision");
                decision_str(decision.decision).to_string()
            }
        }
    } else {
        decision_str(decision.decision).to_string()
    };

    // 3) Exec if allow.
    if bridge_decision == "allow" {
        // Build the command. We use the OS shell so pipelines etc. work.
        let mut child = if cfg!(target_os = "windows") {
            std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", cmd])
                .spawn()
        } else {
            std::process::Command::new("sh").args(["-c", cmd]).spawn()
        };
        match child.as_mut() {
            Ok(c) => {
                let status = c.wait();
                match status {
                    Ok(s) => std::process::exit(s.code().unwrap_or(0)),
                    Err(e) => {
                        eprintln!("actionguard: command failed ({e})");
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("actionguard: spawn failed ({e})");
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("actionguard: blocked ({bridge_decision})");
        std::process::exit(126);
    }
}

/// POST /preexec to the bridge; return the decision string ("allow"/"deny").
fn ask_bridge(port: u16, secret: &str, cmd: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "command": cmd,
        "cwd": std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default(),
        "shell": "cli",
    });
    let resp = http_post(port, secret, "/preexec", &body.to_string())?;
    let parsed: serde_json::Value = serde_json::from_str(&resp).map_err(|e| e.to_string())?;
    let decision = parsed
        .get("decision")
        .and_then(|v| v.as_str())
        .unwrap_or("deny");
    Ok(decision.to_string())
}

// ---------------------------------------------------------------------------
// protect — start a protected session for a workspace
// ---------------------------------------------------------------------------

fn protect(workspace: &PathBuf, observe: bool) {
    let ws = match canonicalize_workspace(workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("actionguard: {e}");
            std::process::exit(1);
        }
    };

    let mode_str = if observe { "observe" } else { "protected" };
    let mode_label = if observe {
        "Mode A (Observe)"
    } else {
        "Mode B (Protected Execution)"
    };

    // Load policy set to count rules for the banner.
    let rule_count = policy::load_policy_set().rules.len();

    // Print the startup banner.
    println!("ActionGuard");
    println!("───────────");
    println!("Mode:        {}", mode_label);
    println!("Workspace:   {}", ws.display());
    println!();
    println!("Policies loaded: {}", rule_count);
    println!("Pending approval: 0");
    print_enforcement_paths();
    println!("Ready.");
    println!();

    // Try to find the GUI binary next to the CLI binary.
    let gui_binary = find_gui_binary();

    match gui_binary {
        Some(exe) => {
            // Pass workspace + mode via env vars, NOT CLI args — Tauri's
            // internal arg parser rejects unknown flags like `--workspace`.
            // The GUI reads `ACTIONGUARD_WORKSPACE` / `ACTIONGUARD_OBSERVE`
            // in `get_startup_args()`.
            //
            // Redirect the child's stdio to null so the GUI doesn't hold
            // the CLI's stdout/stderr pipes open — without this, the parent
            // shell hangs waiting on the pipe even after the CLI exits.
            use std::process::Stdio;
            let mut cmd = std::process::Command::new(&exe);
            cmd.env("ACTIONGUARD_WORKSPACE", &ws);
            if observe {
                cmd.env("ACTIONGUARD_OBSERVE", "1");
            }
            cmd.stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            match cmd.spawn() {
                Ok(child) => {
                    eprintln!(
                        "GUI started (pid {}). Workspace pre-selected, mode set to '{}'.",
                        child.id(),
                        mode_str
                    );
                }
                Err(e) => {
                    eprintln!("Could not launch GUI ({e})");
                    eprintln!("Falling back to manual instructions:");
                    print_manual_instructions(&ws, mode_str);
                }
            }
        }
        None => {
            print_manual_instructions(&ws, mode_str);
        }
    }
}

fn canonicalize_workspace(p: &PathBuf) -> Result<PathBuf, String> {
    let abs = if p.is_absolute() {
        p.clone()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("cannot determine cwd: {e}"))?
            .join(p)
    };
    if !abs.is_dir() {
        return Err(format!("not a directory: {}", abs.display()));
    }
    Ok(abs)
}

fn find_gui_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    // The GUI binary is built from src/main.rs and named `actionguard-gui`
    // (see Cargo.toml). The `-gui` suffix avoids NTFS case-insensitivity
    // collisions with the CLI binary `actionguard`.
    let candidates: Vec<&str> = if cfg!(target_os = "windows") {
        vec!["actionguard-gui.exe", "ActionGuard-GUI.exe"]
    } else if cfg!(target_os = "macos") {
        // On macOS the bundled .app is the primary GUI entry; fall back to
        // the raw binary for dev mode.
        vec!["actionguard-gui", "ActionGuard-GUI"]
    } else {
        vec!["actionguard-gui", "ActionGuard-GUI"]
    };
    for name in candidates {
        let path = dir.join(name);
        // Defensive: never return the current binary (would cause infinite
        // recursion if the GUI binary is missing and only the CLI exists).
        if path.exists() && std::fs::canonicalize(&path).ok() != std::fs::canonicalize(&exe).ok() {
            return Some(path);
        }
    }
    None
}

fn print_manual_instructions(ws: &PathBuf, mode_str: &str) {
    eprintln!("actionguard: GUI binary not found next to the CLI.");
    eprintln!();
    eprintln!("To protect this workspace:");
    eprintln!("  1. Open the ActionGuard desktop app");
    eprintln!("  2. Choose this folder:  {}", ws.display());
    eprintln!("  3. Select '{}' mode", mode_str);
    eprintln!("  4. Click 'Start Protected Session'");
    eprintln!();
    eprintln!("In dev mode, launch the GUI in a separate terminal:");
    eprintln!("  cargo run --bin actionguard-gui");
    eprintln!();
    eprintln!("Or source the shell hook for an existing session:");
    eprintln!("  eval \"$(actionguard init-bash)\"");
    eprintln!();
    eprintln!("Dry-run the policy on a command first:");
    eprintln!("  actionguard policy-check \"npm install axios\"");
}

// ---------------------------------------------------------------------------
// stats — aggregate metric across all sessions
// ---------------------------------------------------------------------------

fn stats(export: Option<&std::path::Path>) {
    use actionguard_lib::models::EnforcementCounts;
    let sessions = storage::list_sessions().unwrap_or_default();
    let total_sessions = sessions.len();
    let mut total_detected = 0u32;
    let mut total_blocked = 0u32;
    let mut enforcement = EnforcementCounts::default();
    let mut risk_counts = (0u32, 0u32, 0u32, 0u32); // low, medium, high, critical
    for s in &sessions {
        total_detected += s.actions_protected;
        total_blocked += s.actions_blocked;
        enforcement.enforced += s.enforcement_counts.enforced;
        enforcement.observed += s.enforcement_counts.observed;
        enforcement.bypassed += s.enforcement_counts.bypassed;
        enforcement.unsupported += s.enforcement_counts.unsupported;
        risk_counts.0 += s.risk_counts.low;
        risk_counts.1 += s.risk_counts.medium;
        risk_counts.2 += s.risk_counts.high;
        risk_counts.3 += s.risk_counts.critical;
    }
    println!("Actions Detected:   {total_detected}  (recorded across all boundaries)");
    println!("Actions Blocked:    {total_blocked}  (deny decisions)");
    println!();
    println!("Enforcement (Detection ≠ Protection):");
    println!("  Enforced:         {}  (actually stopped before execution)", enforcement.enforced);
    println!("  Observed:         {}  (recorded, could not block)", enforcement.observed);
    println!("  Bypassed:         {}  (execution bypassed the boundary)", enforcement.bypassed);
    println!("  Unsupported:      {}  (path not covered)", enforcement.unsupported);
    println!();
    println!("Sessions:                {total_sessions}");
    println!("Risk breakdown:");
    println!("  LOW:       {}", risk_counts.0);
    println!("  MEDIUM:    {}", risk_counts.1);
    println!("  HIGH:      {}", risk_counts.2);
    println!("  CRITICAL:  {}", risk_counts.3);

    if let Some(path) = export {
        #[derive(serde::Serialize)]
        struct RiskBreakdown {
            low: u32,
            medium: u32,
            high: u32,
            critical: u32,
        }
        #[derive(serde::Serialize)]
        struct Report<'a> {
            generated_at: String,
            total_sessions: usize,
            total_detected: u32,
            total_blocked: u32,
            enforcement: &'a EnforcementCounts,
            risk: RiskBreakdown,
            sessions: &'a [actionguard_lib::models::SessionSummary],
        }
        let report = Report {
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            total_sessions,
            total_detected,
            total_blocked,
            enforcement: &enforcement,
            risk: RiskBreakdown {
                low: risk_counts.0,
                medium: risk_counts.1,
                high: risk_counts.2,
                critical: risk_counts.3,
            },
            sessions: &sessions,
        };
        let json = serde_json::to_string_pretty(&report).unwrap_or_else(|e| {
            eprintln!("error: failed to serialize report: {e}");
            String::new()
        });
        if std::fs::write(path, json).is_ok() {
            println!();
            println!("Report written to {}", path.display());
        } else {
            eprintln!("error: could not write report to {}", path.display());
        }
    }
}

// ---------------------------------------------------------------------------
// capabilities — Capability Tier Model + local execution-path matrix
// ---------------------------------------------------------------------------

fn capabilities() {
    use actionguard_lib::models::CapabilityTier;

    println!("Capability Tier Model (Detection ≠ Protection):");
    println!("──────────────────────────────────────────────────────────────");
    for tier in [
        CapabilityTier::L1Observe,
        CapabilityTier::L2PreAction,
        CapabilityTier::L3Runtime,
        CapabilityTier::L4System,
    ] {
        println!("  {:<22} {}", tier.label(), tier.description());
    }
    println!();
    let paths = actionguard_lib::platform::enforcement_paths();
    let width = paths
        .iter()
        .map(|p| p.path.chars().count())
        .max()
        .unwrap_or(0);
    println!("Execution Path Matrix (this machine):");
    println!("──────────────────────────────────────────────────────────────");
    let mut covered = 0usize;
    for p in &paths {
        let tier = CapabilityTier::from_capabilities(p.observe, p.block);
        if tier.is_some() {
            covered += 1;
        }
        println!(
            "  {:<width$}  {:<12} {}",
            p.path,
            tier.map(|t| t.label().to_string()).unwrap_or_else(|| "not covered".to_string()),
            p.note,
            width = width
        );
    }
    println!();
    println!("{covered}/{} execution paths covered by an ActionGuard boundary.", {
        paths.len()
    });
    println!();
    // Same live source as `boundary list` and `doctor` — one registry, three
    // renderers. If this line drifts from them, that is a bug.
    let boundaries = actionguard_lib::boundary::detect_boundaries();
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for d in &boundaries {
        *counts.entry(d.status.label()).or_insert(0) += 1;
    }
    let rendered = counts
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join(" · ");
    println!("Boundary registry (same live source as `boundary list` / `doctor`): {rendered}");
    println!();
    println!("Note: 'Observed' records an action; 'Enforced' stops it before");
    println!("execution. A Deny is only as strong as the path it runs on.");
}

// ---------------------------------------------------------------------------
// boundary list / test — Boundary Registry
// ---------------------------------------------------------------------------

fn boundary_list() {
    use actionguard_lib::boundary;
    println!();
    println!("Action Boundary Classes (A–F):");
    println!("  A. Tool Hook          — pre-action hook inside an automation tool (CodeBuddy PreToolUse)");
    println!("  B. Exec Approval      — automation's own execution boundary (OpenClaw, Manus Desktop)");
    println!("  C. Protected Shell    — shell hook path (bash/zsh/fish, PowerShell)");
    println!("  D. Runtime Sandbox    — runtime/process-level control (future L3)");
    println!("  E. System Enforcement — OS-level, vendor-independent (future L4)");
    println!("  F. Remote             — actions never land on this machine (Manus Cloud)");
    println!();
    println!("Boundary Registry (this machine):");
    println!("──────────────────────────────────────────────────");
    for d in boundary::detect_boundaries() {
        println!("{}", d.name);
        println!("  kind:           {}", d.kind.label());
        println!("  status:         {}", d.status.label());
        println!(
            "  enforceable:    {}",
            if d.enforceable { "yes" } else { "no (no pre-action mechanism)" }
        );
        if !d.scope.is_empty() {
            println!("  scope:          {}", d.scope);
        }
        let verified = match d.verification {
            boundary::Verification::Core => "✓ Core Verified",
            boundary::Verification::Community => "✓ Community Verified",
            boundary::Verification::None => "? Not verified",
        };
        println!("  verification:   {verified}");
        if !d.contributor.is_empty() {
            println!("  contributor:    {}", d.contributor);
        }
        println!(
            "  last verified:  {}",
            if d.last_verified.is_empty() {
                "—"
            } else {
                d.last_verified.as_str()
            }
        );
        println!("  note:           {}", d.note);
        println!();
    }
}

fn boundary_test(name: Option<&str>) {
    use actionguard_lib::boundary;
    for line in boundary::test_boundaries(name) {
        println!("{line}");
    }
}

// ---------------------------------------------------------------------------
// HTTP helpers — minimal HTTP/1.1 client for the local bridge
// ---------------------------------------------------------------------------

fn http_get(port: u16, secret: &str, path: &str) -> Result<String, String> {
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nX-ActionGuard-Secret: {secret}\r\nConnection: close\r\n\r\n"
    );
    http_roundtrip(port, req)
}

fn http_post(port: u16, secret: &str, path: &str, body: &str) -> Result<String, String> {
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nX-ActionGuard-Secret: {secret}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    http_roundtrip(port, req)
}

fn http_roundtrip(port: u16, req: String) -> Result<String, String> {
    let mut stream = TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_secs(2),
    )
    .map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(60))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    let raw = String::from_utf8_lossy(&buf).to_string();
    // Split headers / body.
    let body = raw
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or(raw);
    Ok(body)
}
