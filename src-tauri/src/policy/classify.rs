//! Shell command classifier.
//!
//! Given a raw shell command string (e.g. `"git reset --hard HEAD~10"`),
//! split it into a structured shape:
//!   - category:  Git | Package | Shell
//!   - command:    the first token (e.g. `"git"`)
//!   - args:       the remaining tokens
//!
//! The classifier is intentionally a thin token-based layer — it does not
//! evaluate pipelines, redirections, or shell operators. Those constructs
//! are handled at the shell-hook layer (the hook sees the FIRST command
//! only; the bridge gets a clean argv).

use crate::models::ActionCategory;

/// Parsed form of a shell command. `command` is always lowercase so rule
/// authors can write `command: git` regardless of how the user typed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub category: ActionCategory,
    pub command: String,
    pub args: Vec<String>,
}

/// Parse a raw command line into a `ParsedCommand`.
///
/// Strategy:
///   1. Trim + lowercase the first token only (preserve arg case).
///   2. Look up the first token against the known prefixes:
///       - git            → Git
///       - npm/pnpm/yarn  → Package
///       - pip/pip3/poetry/uv/conda → Package
///       - brew           → Package (system-level)
///       - everything else → Shell
///   3. Build args vec (rest of tokens).
pub fn classify_shell_command(line: &str) -> ParsedCommand {
    let trimmed = line.trim();
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.is_empty() {
        return ParsedCommand {
            category: ActionCategory::Shell,
            command: String::new(),
            args: Vec::new(),
        };
    }
    let first = tokens[0].to_lowercase();
    let args: Vec<String> = tokens[1..].iter().map(|s| s.to_string()).collect();
    let category = category_for(&first);
    ParsedCommand {
        category,
        command: first,
        args,
    }
}

/// Map a binary name to its action category. Public so callers can preview
/// the category for a single token without parsing the whole line.
pub fn category_for(binary: &str) -> ActionCategory {
    match binary {
        "git" => ActionCategory::Git,
        "npm" | "pnpm" | "yarn" | "npx" | "yarnpkg" => ActionCategory::Package,
        "pip" | "pip3" | "pipx" | "poetry" | "uv" | "conda" | "mamba" | "uvx" => {
            ActionCategory::Package
        }
        "brew" | "apt" | "apt-get" | "aptitude" | "dnf" | "yum" | "pacman" | "choco"
        | "scoop" | "winget" | "cargo" | "go" | "rustup" | "gem" | "bundle" | "mix"
        | "rebar3" | "stack" | "cabal" | "opam" | "nix" | "nix-env" | "nix-collect-garbage" => {
            ActionCategory::Package
        }
        _ => ActionCategory::Shell,
    }
}

/// Convenience helper: build the `kind` verb for a parsed command. For
/// Shell/Git/Package categories we currently use the canonical verb
/// "execute" — Phase C (Shell Side Effects) will add finer verbs like
/// "install", "uninstall", "push", "reset".
pub fn kind_for(parsed: &ParsedCommand) -> &'static str {
    match parsed.category {
        ActionCategory::Git => "git",
        ActionCategory::Package => {
            if parsed.args.iter().any(|a| matches!(a.as_str(), "install" | "i" | "add" | "ci"))
            {
                "install"
            } else if parsed
                .args
                .iter()
                .any(|a| matches!(a.as_str(), "uninstall" | "remove" | "rm"))
            {
                "uninstall"
            } else if parsed.args.iter().any(|a| a == "publish") {
                "publish"
            } else {
                "execute"
            }
        }
        ActionCategory::Shell | ActionCategory::File | ActionCategory::Secret => "execute",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_git() {
        let p = classify_shell_command("git reset --hard HEAD~10");
        assert_eq!(p.category, ActionCategory::Git);
        assert_eq!(p.command, "git");
        assert_eq!(p.args, vec!["reset", "--hard", "HEAD~10"]);
    }

    #[test]
    fn classify_npm() {
        let p = classify_shell_command("npm install axios");
        assert_eq!(p.category, ActionCategory::Package);
        assert_eq!(p.command, "npm");
        assert_eq!(p.args, vec!["install", "axios"]);
    }

    #[test]
    fn classify_pip3_uppercase() {
        let p = classify_shell_command("PIP3 install requests");
        // First token should be lowercased; args preserve case.
        assert_eq!(p.command, "pip3");
        assert_eq!(p.category, ActionCategory::Package);
        assert_eq!(p.args, vec!["install", "requests"]);
    }

    #[test]
    fn classify_rm_shell() {
        let p = classify_shell_command("rm -rf dist");
        assert_eq!(p.category, ActionCategory::Shell);
        assert_eq!(p.command, "rm");
        assert_eq!(p.args, vec!["-rf", "dist"]);
    }

    #[test]
    fn classify_brew() {
        let p = classify_shell_command("brew install ripgrep");
        assert_eq!(p.category, ActionCategory::Package);
        assert_eq!(p.command, "brew");
    }

    #[test]
    fn kind_for_install_vs_uninstall() {
        let install = classify_shell_command("npm install axios");
        assert_eq!(kind_for(&install), "install");
        let uninstall = classify_shell_command("npm uninstall axios");
        assert_eq!(kind_for(&uninstall), "uninstall");
        let publish = classify_shell_command("npm publish");
        assert_eq!(kind_for(&publish), "publish");
        let git_reset = classify_shell_command("git reset --hard");
        assert_eq!(kind_for(&git_reset), "git");
    }

    #[test]
    fn empty_command() {
        let p = classify_shell_command("");
        assert_eq!(p.command, "");
        assert!(p.args.is_empty());
    }
}
