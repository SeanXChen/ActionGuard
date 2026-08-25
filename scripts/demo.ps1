<#
.ActionGuard - real demo script (for recording the 10-second GIF / screenshots)
===============================================================================
Story: AI tries a destructive action -> ActionGuard evaluates -> refusal ->
       command not executed -> ledger evidence.

Everything runs on the real engine - nothing is simulated. Two ways to record:

  Option A (pure CLI, fastest)
    Run this script and record the terminal (ScreenToGif / OBS / Win+G).
    Scenes: doctor --test (real verification), policy-check --explain,
    run <destructive command> (real refusal, exit 126, nothing executed),
    git log proves the workspace survived.

  Option B (full story, recommended for the 10-second GIF)
    Finish Scenes 1-3, then follow Scene 4: start a GUI protected session,
    type the destructive command in the interactive PowerShell, watch the
    hook refuse it, then inspect the ledger.

Usage:
  powershell -ExecutionPolicy Bypass -File scripts\demo.ps1
#>
[CmdletBinding()]
param(
  # Path to the actionguard CLI. Empty = auto-detect (PATH -> debug build).
  [string]$ActionGuard = ""
)

$ErrorActionPreference = "Stop"

# locate the CLI (PATH -> debug build)
if (-not $ActionGuard) {
  $candidate = Join-Path $PSScriptRoot "..\src-tauri\target\debug\actionguard.exe"
  if (Test-Path $candidate) {
    $ActionGuard = $candidate
  } else {
    $cmd = Get-Command actionguard -ErrorAction SilentlyContinue
    if ($cmd) { $ActionGuard = $cmd.Source }
  }
}
if (-not $ActionGuard -or -not (Test-Path $ActionGuard)) {
  Write-Host "CLI not found. Build it first: cd src-tauri; cargo build --bin actionguard" -ForegroundColor Yellow
  exit 2
}
Write-Host "==> Using CLI: $ActionGuard" -ForegroundColor Cyan

function Invoke-Ag($cli, $argsList) {
  $out = Join-Path $env:TEMP ("ag-demo-out-" + [guid]::NewGuid().ToString("N") + ".txt")
  $err = Join-Path $env:TEMP ("ag-demo-err-" + [guid]::NewGuid().ToString("N") + ".txt")
  $oldEap = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try {
    & $cli @argsList 1> $out 2> $err
    $code = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $oldEap
  }
  $stdout = if (Test-Path $out) { Get-Content $out -Raw } else { "" }
  $stderr = if (Test-Path $err) { Get-Content $err -Raw } else { "" }
  Remove-Item $out, $err -Force -ErrorAction SilentlyContinue
  [pscustomobject]@{ ExitCode = $code; Stdout = $stdout; Stderr = $stderr }
}

function Step-Banner($title) {
  Write-Host ""
  Write-Host ("=" * 72) -ForegroundColor DarkCyan
  Write-Host ("  " + $title) -ForegroundColor Cyan
  Write-Host ("=" * 72) -ForegroundColor DarkCyan
}

function Pause-Step($hint) {
  Write-Host ""
  Write-Host $hint -ForegroundColor DarkGray
  Read-Host "Press Enter to continue..." | Out-Null
}

# 1. prepare a throwaway git repo (real context; later we prove it survives)
$demoRoot = Join-Path $env:TEMP "actionguard-demo"
if (Test-Path $demoRoot) { Remove-Item $demoRoot -Recurse -Force }
New-Item -ItemType Directory -Path $demoRoot -Force | Out-Null
Set-Location $demoRoot
git init -q .
Set-Content -Path "notes.md" -Value "draft" -Encoding UTF8
git add notes.md
git commit -q -m "wip: draft"
Set-Content -Path "notes.md" -Value "rewritten" -Encoding UTF8
git add notes.md
git commit -q -m "wip: rewrite"
Write-Host "==> Demo repo ready: $demoRoot (2 commits - we will prove they survive)" -ForegroundColor Cyan

# Scene 1 - doctor --test (real machine verification)
Step-Banner "Scene 1 - real machine verification (actionguard doctor --test)"
Pause-Step "Recording now: doctor --test on this machine - measured, not marketing."
$r1 = Invoke-Ag $ActionGuard @("doctor", "--test")
Write-Host $r1.Stdout
if ($r1.Stderr) { Write-Host $r1.Stderr -ForegroundColor Yellow }
Pause-Step "doctor --test done. Each line is a boundary's measured status."

# Scene 2 - policy-check (decision explanation, DRY RUN)
Step-Banner "Scene 2 - decision explanation (actionguard policy-check --explain)"
Pause-Step "Now the engine explains its decision for a destructive command - dry run, nothing executes."
$r2 = Invoke-Ag $ActionGuard @("policy-check", "git reset --hard HEAD~1", "--explain")
Write-Host $r2.Stdout
if ($r2.Stderr) { Write-Host $r2.Stderr -ForegroundColor Yellow }
Pause-Step "policy-check is pure decision. Next: the real thing."

# Scene 3 - run (real refusal, exit 126, command NOT executed)
Step-Banner 'Scene 3 - real refusal (actionguard run "git reset --hard HEAD~1")'
Pause-Step "The engine evaluates this command for real: decision is a refusal, the command does NOT run."
$r3 = Invoke-Ag $ActionGuard @("run", "git reset --hard HEAD~1")
Write-Host $r3.Stdout
if ($r3.Stderr) { Write-Host $r3.Stderr -ForegroundColor Yellow }
Write-Host ""
Write-Host ("Exit code: " + $r3.ExitCode + "  (126 = refused by the safety layer, command not executed)") -ForegroundColor Cyan
$count = (git rev-list --count HEAD)
Write-Host ("Commits still intact: " + $count + " (workspace survived - the destructive command never ran)") -ForegroundColor Green
Pause-Step "Key evidence: the command was refused, the repo is unharmed."

# Scene 4 - GUI protected session + interactive PowerShell (full story for the GIF)
Step-Banner "Scene 4 - full story (GUI protected session + interactive PowerShell)"
Write-Host @"

This records the real path from the README:
Agent -> ActionGuard -> refusal -> ledger.

  1. In a NEW PowerShell window run:
        cd "$demoRoot"
        "$ActionGuard" protect .

     ActionGuard GUI will open.

  2. In the GUI click Start Protected Session (Protect this computer) and keep the session active.

  3. Back in THIS interactive PowerShell, type and press Enter:
        git reset --hard HEAD~1

     The hook intercepts before execution: approval required / denied, the command never runs.

  4. Inspect the ledger:
        "$ActionGuard" session list
        "$ActionGuard" actions show <session-id from the previous step>
"@ -ForegroundColor DarkGray

$goGui = Read-Host "Start the GUI and do the manual steps above, then come back (y/N)"
if ($goGui -match "^[yY]") {
  $goStats = Read-Host "Show aggregate stats (actionguard stats)? (y/N)"
  if ($goStats -match "^[yY]") {
    $st = Invoke-Ag $ActionGuard @("stats")
    Write-Host $st.Stdout
    if ($st.Stderr) { Write-Host $st.Stderr -ForegroundColor Yellow }
  }
} else {
  Write-Host "Skipped Scene 4. The pure CLI material is enough: verify -> decide -> refuse -> evidence." -ForegroundColor DarkGray
}

# cleanup
Set-Location $env:TEMP
Remove-Item $demoRoot -Recurse -Force -ErrorAction SilentlyContinue
Write-Host ""
Write-Host "==> Demo finished, throwaway repo cleaned up." -ForegroundColor DarkCyan
Write-Host "Recording tips: ScreenToGif or OBS for the terminal; Win+Shift+S for GUI screenshots." -ForegroundColor DarkGray
