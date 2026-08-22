// ActionGuard v0.3 — platform capability detection
//
// The Execution Path Matrix: for each way a command can reach a shell, report
// whether ActionGuard can observe it and whether it can actually BLOCK it on
// the current platform. Every row maps to a Capability Tier (L1 observe …
// L4 system, see models::CapabilityTier) so "detected" is never confused with
// "protected". The GUI and the CLI (`actionguard capabilities`) both surface
// this so users never assume a "protected" session intercepts paths it does
// not.

/// One row of the Execution Path Matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionPath {
    pub path: &'static str,
    pub observe: bool,
    pub block: bool,
    pub note: &'static str,
}

/// Return the per-execution-path enforcement capabilities of THIS platform.
pub fn enforcement_paths() -> Vec<ExecutionPath> {
    #[cfg(target_os = "windows")]
    {
        vec![
            ExecutionPath {
                path: "PowerShell interactive (PSReadLine)",
                observe: true,
                block: true,
                note: "Phase C: deny reverts the line before execution",
            },
            ExecutionPath {
                path: "PowerShell scripts / -Command / piped stdin",
                observe: true,
                block: false,
                note: "not intercepted (bypasses PSReadLine)",
            },
            ExecutionPath {
                path: "cmd.exe interactive",
                observe: false,
                block: false,
                note: "no hook installed (Windows)",
            },
            ExecutionPath {
                path: "bash/zsh/fish (WSL/MSYS2)",
                observe: true,
                block: true,
                note: "hook works if sourced inside the POSIX shell",
            },
            ExecutionPath {
                path: "Python subprocess / os.system / subprocess.run",
                observe: false,
                block: false,
                note: "known bypass — not covered in v0.2",
            },
            ExecutionPath {
                path: "Absolute-path process invocation",
                observe: false,
                block: false,
                note: "known bypass — not covered in v0.2",
            },
        ]
    }
    #[cfg(target_os = "linux")]
    {
        vec![
            ExecutionPath {
                path: "bash interactive",
                observe: true,
                block: true,
                note: "DEBUG trap + SIGINT on deny",
            },
            ExecutionPath {
                path: "zsh interactive",
                observe: true,
                block: true,
                note: "preexec returns 1 on deny",
            },
            ExecutionPath {
                path: "fish interactive",
                observe: true,
                block: true,
                note: "commandline -f cancel on deny",
            },
            ExecutionPath {
                path: "non-interactive scripts / cron / pipelines",
                observe: true,
                block: false,
                note: "not intercepted",
            },
            ExecutionPath {
                path: "Python subprocess / os.system / subprocess.run",
                observe: false,
                block: false,
                note: "known bypass — not covered in v0.2",
            },
            ExecutionPath {
                path: "Absolute-path process invocation",
                observe: false,
                block: false,
                note: "known bypass — not covered in v0.2",
            },
        ]
    }
    #[cfg(target_os = "macos")]
    {
        vec![
            ExecutionPath {
                path: "bash interactive",
                observe: true,
                block: true,
                note: "DEBUG trap + SIGINT on deny",
            },
            ExecutionPath {
                path: "zsh interactive",
                observe: true,
                block: true,
                note: "preexec returns 1 on deny",
            },
            ExecutionPath {
                path: "fish interactive",
                observe: true,
                block: true,
                note: "commandline -f cancel on deny",
            },
            ExecutionPath {
                path: "non-interactive scripts / launchd / pipelines",
                observe: true,
                block: false,
                note: "not intercepted",
            },
            ExecutionPath {
                path: "Python subprocess / os.system / subprocess.run",
                observe: false,
                block: false,
                note: "known bypass — not covered in v0.2",
            },
            ExecutionPath {
                path: "Absolute-path process invocation",
                observe: false,
                block: false,
                note: "known bypass — not covered in v0.2",
            },
        ]
    }
}

impl ExecutionPath {
    pub fn yes_no(&self, b: bool) -> &'static str {
        if b {
            "YES"
        } else {
            "NO"
        }
    }
}
