<#
.ActionGuard v0.2.x — Enforcement Validation (Windows / PowerShell)
====================================================================
正向 + 负向端到端测试,产生 "enforcement evidence":

  Case     Kind        Command                            Expected
  A        Allow       echo AG_ALLOW_OK                   exit 0, stdout contains it
  B        Ask         git push --force origin feature/x  confirm -> blocked (fail-closed in headless CLI)
  C        Deny        Remove-Item -Recurse -Force <dir>  deny -> exit 126, directory STILL EXISTS
  D        Bypass      python -c "shutil.rmtree(...)"     received: NO, directory deleted (known boundary)
  E        Hook        (powershell init)                  Phase C RevertLine intercept present
  F        FailClosed  (hook, bridge down)                blocked by default; AG_ALLOW_ON_FAILURE=1 allows
  G        NoSession   (hook, no current.hook)            blocked; allowed after clean stop / opt-out
  H        PosixHook   (bash hook, if bash present)       blocked by default; allowed after clean stop / opt-out

Every case records:
  - ActionGuard received action: YES / NO
  - Policy decision:             allow / confirm / deny / (none)
  - Command actually executed:   YES / NO

Usage:
  powershell -ExecutionPolicy Bypass -File scripts\e2e-windows.ps1

Exit code: 0 = all enforced cases passed; 1 = a FAILED case; 2 = setup error.
#>
[CmdletBinding()]
param(
  # Path to the built CLI. Defaults to the standard debug build.
  [string]$ActionGuard = "",
  # Workspace where the test directory lives (defaults to a temp dir).
  [string]$TestRoot = ""
)

$ErrorActionPreference = "Stop"

# Run a native command and capture BOTH streams + exit code without letting
# PowerShell 5.1 turn stderr into a NativeCommandError.
function Invoke-Ag($cli, $argsList) {
  $out = Join-Path $env:TEMP ("ag-out-" + [guid]::NewGuid().ToString("N") + ".txt")
  $err = Join-Path $env:TEMP ("ag-err-" + [guid]::NewGuid().ToString("N") + ".txt")
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
  [pscustomobject]@{
    ExitCode = $code
    Stdout   = $stdout
    Stderr   = $stderr
  }
}

function Write-Result($case, $kind, $received, $decision, $executed, $pass, $note) {
  $icon = if ($pass) { "PASS" } else { "FAIL" }
  "{0,-8} {1,-7} received={2,-3} decision={3,-10} executed={4,-3} {5,-4} {6}" -f `
    $case, $kind, $received, $decision, $executed, $icon, $note | Write-Output
}

$allPass = $true

# ---------------------------------------------------------------------------
# 0. Locate the CLI binary (build it if missing).
# ---------------------------------------------------------------------------
if (-not $ActionGuard) {
  $candidate = Join-Path $PSScriptRoot "..\src-tauri\target\debug\actionguard.exe"
  if (-not (Test-Path $candidate)) {
    Write-Host "==> Building actionguard CLI (first run)..." -ForegroundColor Cyan
    Push-Location (Join-Path $PSScriptRoot "..\src-tauri")
    try {
      cargo build --bin actionguard
      if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
    } finally {
      Pop-Location
    }
  }
  $ActionGuard = $candidate
}
if (-not (Test-Path $ActionGuard)) {
  Write-Error "CLI not found at $ActionGuard"
  exit 2
}
Write-Host "==> Using CLI: $ActionGuard" -ForegroundColor Cyan

# ---------------------------------------------------------------------------
# 1. Sanity: policy-check (no side effects) — proves the engine is alive.
# ---------------------------------------------------------------------------
$sanity = Invoke-Ag $ActionGuard @("policy-check", "rm -rf /")
if ($sanity.ExitCode -ne 0) {
  Write-Error "policy-check failed: $($sanity.Stdout) $($sanity.Stderr)"
  exit 2
}

# ---------------------------------------------------------------------------
# 2. Prepare a test target directory.
# ---------------------------------------------------------------------------
if (-not $TestRoot) {
  $TestRoot = Join-Path $env:TEMP "actionguard-e2e"
}
$targetDir = Join-Path $TestRoot "test-target"
if (Test-Path $targetDir) { Remove-Item -Recurse -Force $targetDir }
New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
1..3 | ForEach-Object {
  New-Item -ItemType File -Path (Join-Path $targetDir "file$_") -Force | Out-Null
}
Write-Host "==> Test target: $targetDir" -ForegroundColor Cyan

# ---------------------------------------------------------------------------
# 3. Add a temporary user deny rule for `Remove-Item -Recurse -Force`.
#    Back up any existing user policy file first.
# ---------------------------------------------------------------------------
$userPolicyPath = Join-Path $env:USERPROFILE ".actionguard\policies.user.yml"
$backupPolicyPath = Join-Path $env:USERPROFILE ".actionguard\policies.user.yml.e2e-bak"
$hadPolicy = Test-Path $userPolicyPath
if ($hadPolicy) { Copy-Item $userPolicyPath $backupPolicyPath -Force }

$denyRule = @"
# Temporary rule injected by e2e-windows.ps1 — DO NOT commit.
version: 1
scope: shell
rules:
  - id: e2e-deny-remove-item
    match:
      category: shell
      command: remove-item
      args_contains:
        - -recurse
        - -force
    action: deny
    risk: critical
    reason: "E2E test: deny Remove-Item -Recurse -Force"
"@
New-Item -ItemType Directory -Path (Split-Path $userPolicyPath) -Force | Out-Null
if ($hadPolicy) {
  $existing = Get-Content $userPolicyPath -Raw
  Set-Content -Path $userPolicyPath -Value ($existing.TrimEnd() + "`n" + $denyRule) -Encoding UTF8
} else {
  Set-Content -Path $userPolicyPath -Value $denyRule -Encoding UTF8
}

# Verify the injected rule parses (lint) and is actually loaded.
$lint = Invoke-Ag $ActionGuard @("policy-lint", $userPolicyPath)
if ($lint.ExitCode -ne 0) {
  Write-Error "injected deny rule failed lint: $($lint.Stdout) $($lint.Stderr)"
  exit 2
}
$listOut = (Invoke-Ag $ActionGuard @("policy-list")).Stdout
if ($listOut -notmatch "e2e-deny-remove-item") {
  Write-Error "e2e deny rule not loaded into policy set"
  exit 2
}
Write-Host "==> e2e deny rule injected + verified" -ForegroundColor Cyan

try {
  # -------------------------------------------------------------------------
  # Case A — ALLOW
  # -------------------------------------------------------------------------
  $resA = Invoke-Ag $ActionGuard @("run", "echo AG_ALLOW_OK")
  $codeA = $resA.ExitCode
  $receivedA = "YES"
  $decisionA = "allow"
  $executedA = if (($resA.Stdout + $resA.Stderr) -match "AG_ALLOW_OK") { "YES" } else { "NO" }
  $passA = ($codeA -eq 0 -and $executedA -eq "YES")
  if (-not $passA) { $allPass = $false }
  Write-Result "A" "Allow" $receivedA $decisionA $executedA $passA "echo AG_ALLOW_OK"

  # -------------------------------------------------------------------------
  # Case B — ASK (confirm). In headless CLI mode a confirm is fail-closed to
  # blocked because there is no GUI approval gate. That is correct behaviour.
  # -------------------------------------------------------------------------
  $resB = Invoke-Ag $ActionGuard @("run", "git push --force origin feature/x")
  $codeB = $resB.ExitCode
  $receivedB = "YES"
  $decisionB = "confirm"
  # Fail-closed: confirm != allow, so the CLI must NOT execute it.
  $executedB = "NO"
  $passB = ($codeB -eq 126 -and ($resB.Stdout + $resB.Stderr) -match "blocked")
  if (-not $passB) { $allPass = $false }
  Write-Result "B" "Ask" $receivedB $decisionB $executedB $passB "fail-closed in headless CLI"

  # -------------------------------------------------------------------------
  # Case C — DENY. The directory must STILL EXIST afterwards.
  # -------------------------------------------------------------------------
  $resC = Invoke-Ag $ActionGuard @("run", "Remove-Item -Recurse -Force `"$targetDir`"")
  $codeC = $resC.ExitCode
  $receivedC = "YES"
  $decisionC = "deny"
  $dirExistsC = Test-Path $targetDir
  $executedC = if ($dirExistsC) { "NO" } else { "YES" }
  $passC = ($codeC -eq 126 -and $dirExistsC)
  if (-not $passC) { $allPass = $false }
  Write-Result "C" "Deny" $receivedC $decisionC $executedC $passC "target dir still exists: $dirExistsC"

  # -------------------------------------------------------------------------
  # Case D — BYPASS. Python subprocess removes the directory directly.
  # ActionGuard is NOT in the call path -> received: NO. This documents the
  # enforcement boundary, it is not a pass/fail of the enforced cases.
  # -------------------------------------------------------------------------
  $py = Get-Command python -ErrorAction SilentlyContinue
  if ($py) {
    $pyCode = "import shutil; shutil.rmtree(r'$targetDir')"
    $pyRes = Invoke-Ag "python" @("-c", $pyCode)
    $dirExistsD = Test-Path $targetDir
    $receivedD = "NO"
    $decisionD = "(none)"
    $executedD = if ($dirExistsD) { "NO" } else { "YES" }
    # Bypass is a KNOWN boundary — record it, do not fail the suite on it.
    Write-Result "D" "Bypass" $receivedD $decisionD $executedD $true "known boundary (subprocess bypass)"
  } else {
    Write-Result "D" "Bypass" "-" "-" "-" $true "python not found — skipped"
  }

  # -------------------------------------------------------------------------
  # Case E — PowerShell hook content: Phase C must contain the RevertLine
  # interception, otherwise the hook is still observe-only.
  # -------------------------------------------------------------------------
  $hook = (& $ActionGuard init-powershell 2>&1) | Out-String
  $hasRevert = $hook -match "RevertLine"
  $hasAccept = $hook -match "AcceptLine"
  $passE = ($hasRevert -and $hasAccept)
  if (-not $passE) { $allPass = $false }
  Write-Result "E" "Hook" "-" "-" "-" $passE "PowerShell Phase C RevertLine intercept present: $hasRevert"

  # -------------------------------------------------------------------------
  # Cases F/G/H — FAIL-CLOSED hook behaviour. The enforcement point must
  # BLOCK when it cannot reach the policy engine (default), and only allow
  # when AG_ALLOW_ON_FAILURE=1 (explicit opt-out) or after a deliberate
  # session stop (current.closed sentinel).
  # -------------------------------------------------------------------------
  $fakeSessions = Join-Path $TestRoot "fake-sessions"
  New-Item -ItemType Directory -Path $fakeSessions -Force | Out-Null
  $fakeHook = Join-Path $fakeSessions "current.hook"
  $fakeClosed = Join-Path $fakeSessions "current.closed"

  # Extract the decision functions from the generated PowerShell hook (the
  # PSReadLine wiring needs an interactive console, so we define only the
  # functions and drive them directly).
  $funcBlock = [regex]::Match($hook, '(?s)\bfunction Test-ActionGuardFailOpen\b.*?(?=\r?\n# Phase C)').Value
  if (-not $funcBlock) {
    Write-Error "could not extract hook functions for fail-closed test"
    exit 2
  }
  Invoke-Expression $funcBlock
  function Invoke-AgHookCheck([string]$cmd) {
    return [bool](Invoke-ActionGuardCheck -Cmd $cmd)
  }
  $script:ActionGuard__HookPath = $fakeHook
  $script:ActionGuard__ClosedPath = $fakeClosed

  # A port that is guaranteed to have no listener (bind then release).
  $deadTcp = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Loopback, 0)
  $deadTcp.Start()
  $deadPort = $deadTcp.LocalEndpoint.Port
  $deadTcp.Stop()

  # -------------------------------------------------------------------------
  # Case F — bridge unreachable: blocked by default, allowed with
  # AG_ALLOW_ON_FAILURE=1.
  # -------------------------------------------------------------------------
  Set-Content -Path $fakeHook -Value "$deadPort`nag-e2e-secret" -Encoding ascii
  Remove-Item $fakeClosed -Force -ErrorAction SilentlyContinue
  Remove-Item Env:\AG_ALLOW_ON_FAILURE -ErrorAction SilentlyContinue
  $blockedF = -not (Invoke-AgHookCheck "Remove-Item -Force x")
  $env:AG_ALLOW_ON_FAILURE = "1"
  $allowedF = Invoke-AgHookCheck "Remove-Item -Force x"
  Remove-Item Env:\AG_ALLOW_ON_FAILURE -ErrorAction SilentlyContinue
  $passF = ($blockedF -and $allowedF)
  if (-not $passF) { $allPass = $false }
  Write-Result "F" "FailClosed" "hook" "deny" $(if ($blockedF) { "NO" } else { "-" }) $passF "bridge down: blocked=$blockedF optOutAllowed=$allowedF"

  # -------------------------------------------------------------------------
  # Case G — no active session: blocked by default; allowed after a clean
  # stop (current.closed) or with AG_ALLOW_ON_FAILURE=1.
  # -------------------------------------------------------------------------
  Remove-Item $fakeHook -Force -ErrorAction SilentlyContinue
  Remove-Item $fakeClosed -Force -ErrorAction SilentlyContinue
  Remove-Item Env:\AG_ALLOW_ON_FAILURE -ErrorAction SilentlyContinue
  $blockedG1 = -not (Invoke-AgHookCheck "echo hi")
  Set-Content -Path $fakeClosed -Value "closed" -Encoding UTF8
  $allowedG2 = Invoke-AgHookCheck "echo hi"
  Remove-Item $fakeClosed -Force -ErrorAction SilentlyContinue
  $env:AG_ALLOW_ON_FAILURE = "1"
  $allowedG3 = Invoke-AgHookCheck "echo hi"
  Remove-Item Env:\AG_ALLOW_ON_FAILURE -ErrorAction SilentlyContinue
  $passG = ($blockedG1 -and $allowedG2 -and $allowedG3)
  if (-not $passG) { $allPass = $false }
  Write-Result "G" "NoSession" "hook" "deny" $(if ($blockedG1) { "NO" } else { "-" }) $passG "noSessionBlocked=$blockedG1 cleanStopAllowed=$allowedG2 optOutAllowed=$allowedG3"

  # -------------------------------------------------------------------------
  # Case H — POSIX hook (bash) behavioural test. Runs only when bash is on
  # PATH (e.g. Git Bash); skipped otherwise.
  # -------------------------------------------------------------------------
  $bashExe = Get-Command bash -ErrorAction SilentlyContinue
  if ($bashExe) {
    $posixHook = (& $ActionGuard init-bash 2>&1) | Out-String
    $posixProbe = Join-Path $TestRoot "posix-probe.sh"
    $posixHookFile = Join-Path $TestRoot "posix-hook.sh"
    $posixHome = Join-Path $TestRoot "posix-home"
    New-Item -ItemType Directory -Path (Join-Path $posixHome ".actionguard\sessions") -Force | Out-Null
    $pHook = Join-Path $posixHome ".actionguard\sessions\current.hook"
    $pClosed = Join-Path $posixHome ".actionguard\sessions\current.closed"
    # ascii encoding: a UTF-8 BOM would corrupt the bash shebang / `read port`.
    Set-Content -Path $posixHookFile -Value $posixHook -Encoding ascii
    Set-Content -Path $posixProbe -Value @"
#!/usr/bin/env bash
export HOME="\$1"
source "\$2"
echo MARKER_OK
"@ -Encoding ascii

    # H1: bridge down -> blocked (no MARKER_OK, non-zero exit).
    Set-Content -Path $pHook -Value "$deadPort`nag-e2e-secret" -Encoding ascii
    Remove-Item $pClosed -Force -ErrorAction SilentlyContinue
    Remove-Item Env:\AG_ALLOW_ON_FAILURE -ErrorAction SilentlyContinue
    $rH1 = Invoke-Ag $bashExe.Source @($posixProbe, $posixHome, $posixHookFile)
    $blockedH1 = (($rH1.Stdout + $rH1.Stderr) -notmatch "MARKER_OK") -and ($rH1.ExitCode -ne 0)

    # H2: clean stop sentinel -> allowed.
    Remove-Item $pHook -Force -ErrorAction SilentlyContinue
    Set-Content -Path $pClosed -Value "closed" -Encoding ascii
    $rH2 = Invoke-Ag $bashExe.Source @($posixProbe, $posixHome, $posixHookFile)
    $allowedH2 = ($rH2.Stdout + $rH2.Stderr) -match "MARKER_OK"

    # H3: AG_ALLOW_ON_FAILURE=1 -> allowed.
    Remove-Item $pClosed -Force -ErrorAction SilentlyContinue
    Set-Content -Path $pHook -Value "$deadPort`nag-e2e-secret" -Encoding ascii
    $env:AG_ALLOW_ON_FAILURE = "1"
    $rH3 = Invoke-Ag $bashExe.Source @($posixProbe, $posixHome, $posixHookFile)
    Remove-Item Env:\AG_ALLOW_ON_FAILURE -ErrorAction SilentlyContinue
    $allowedH3 = ($rH3.Stdout + $rH3.Stderr) -match "MARKER_OK"

    $passH = ($blockedH1 -and $allowedH2 -and $allowedH3)
    if (-not $passH) { $allPass = $false }
    Write-Result "H" "PosixHook" "hook" "deny" $(if ($blockedH1) { "NO" } else { "-" }) $passH "bridgeDownBlocked=$blockedH1 cleanStopAllowed=$allowedH2 optOutAllowed=$allowedH3"
  } else {
    Write-Result "H" "PosixHook" "-" "-" "-" $true "bash not found - skipped"
  }

  Write-Host ""
  if ($allPass) {
    Write-Host "RESULT: ALL ENFORCED CASES PASSED" -ForegroundColor Green
  } else {
    Write-Host "RESULT: ONE OR MORE CASES FAILED" -ForegroundColor Red
  }
  Write-Host "Note: D (bypass) documents the known enforcement boundary - it is"
  Write-Host "expected that subprocess/absolute-path invocations bypass v0.2."
}
finally {
  # Restore the user policy file.
  if ($hadPolicy) {
    Copy-Item $backupPolicyPath $userPolicyPath -Force
    Remove-Item $backupPolicyPath -Force
  } else {
    Remove-Item $userPolicyPath -Force -ErrorAction SilentlyContinue
  }
}

exit $(if ($allPass) { 0 } else { 1 })
