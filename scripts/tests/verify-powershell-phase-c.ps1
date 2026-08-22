# ActionGuard v0.2 — PowerShell Phase C real-machine verification
#
# Reproducible evidence for SECURITY_TEST_MATRIX. Non-destructive: every
# command in this script is either (a) executed through the ActionGuard
# protected path (`run`, which blocks on deny), or (b) a Remove-Item that
# targets a throwaway temp directory.
#
# What it proves:
#   Test 1  policy-check returns DENY for a destructive command.
#   Test 2  the ActionGuard protected path BLOCKS a denied command:
#           nothing executes, the marker file survives.
#   Test 3  raw `powershell -Command` of the SAME denied command executes
#           anyway — the non-interactive path bypasses ActionGuard.
#   Test 4  the PSReadLine hook script is fail-closed: with no session and
#           no sentinel, Invoke-ActionGuardCheck blocks (returns $false);
#           with the clean-teardown sentinel it allows (returns $true).
#
# Usage:  powershell -ExecutionPolicy Bypass -File verify-powershell-phase-c.ps1

param(
    [string]$Binary = ""
)
$ErrorActionPreference = 'Stop'

if (-not $Binary) {
    $Binary = Join-Path $PSScriptRoot "..\..\src-tauri\target\debug\actionguard.exe"
}
$Binary = (Resolve-Path $Binary).Path
if (-not (Test-Path $Binary)) { throw "actionguard binary not found: $Binary" }

$script:results = @()
function Assert([string]$Name, [bool]$Cond) {
    if ($Cond) {
        Write-Host "  PASS  $Name" -ForegroundColor Green
        $script:results += "PASS  $Name"
    } else {
        Write-Host "  FAIL  $Name" -ForegroundColor Red
        $script:results += "FAIL  $Name"
    }
}

Write-Host "== ActionGuard PowerShell Phase C verification ==" -ForegroundColor Cyan
Write-Host "binary: $Binary"
Write-Host ""

# ---------------------------------------------------------------- Test 1 ---
Write-Host "Test 1 — policy decision for a destructive command" -ForegroundColor Yellow
$out1 = (& $Binary policy-check "rm -rf /" --explain 2>&1 | Out-String)
Assert "policy-check 'rm -rf /' returns DENY" ($out1 -match "DENY")
Write-Host ""

# ---------------------------------------------------------------- Test 2 ---
Write-Host "Test 2 — protected path ('run') blocks a denied command" -ForegroundColor Yellow
$tmp = Join-Path $env:TEMP ("ag-ps-test-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tmp -Force | Out-Null
$marker = Join-Path $tmp "marker.txt"
Set-Content -Encoding utf8 -Path $marker -Value "test"
$kill = "Remove-Item '$tmp' -Recurse -Force"

# Install a temporary user rule that denies this Remove-Item. The regex avoids
# embedding the temp path (path backslashes would be re-escaped by YAML and
# break the regex); matching on the destructive flag pair is sufficient here
# because this rule lives only for the duration of the test.
$userPolicy = Join-Path $env:USERPROFILE ".actionguard\policies.user.yml"
$userPolicyBackup = $null
if (Test-Path $userPolicy) { $userPolicyBackup = Get-Content $userPolicy -Raw }
$ruleFile = Join-Path $env:TEMP ("ag-rule-" + [guid]::NewGuid().ToString("N") + ".yml")
$ruleYaml = "version: 1`nscope: user`nrules:`n  - id: verify-ps-deny-remove`n    match:`n      regex: `"(?i)Remove-Item.*-Recurse.*-Force`"`n    action: deny`n    risk: high`n    reason: verification-only rule (removed after test)`n"
Set-Content -Encoding utf8 -Path $ruleFile -Value $ruleYaml
& $Binary rule install $ruleFile | Out-Null

$out2 = (& $Binary policy-check $kill --explain 2>&1 | Out-String)
Assert "policy-check '$kill' is DENY after rule install" ($out2 -match "DENY")

$prevEAP = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
& $Binary run $kill 2>$null
$code2 = $LASTEXITCODE
$ErrorActionPreference = $prevEAP
$exists2 = Test-Path $marker
Assert "protected 'run' exits non-zero (code=$code2)" ($code2 -ne 0)
Assert "marker still exists after denied run — command did NOT execute" $exists2
Write-Host ""

# ---------------------------------------------------------------- Test 3 ---
Write-Host "Test 3 — raw 'powershell -Command' bypasses ActionGuard" -ForegroundColor Yellow
$prevEAP = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
powershell.exe -NoProfile -NonInteractive -Command $kill 2>$null
$code3 = $LASTEXITCODE
$ErrorActionPreference = $prevEAP
$exists3 = Test-Path $marker
Assert "raw -Command removed the marker — executed outside the boundary" (-not $exists3)
Write-Host ""

# ---------------------------------------------------------------- Test 4 ---
Write-Host "Test 4 — PSReadLine hook script fail-closed logic (invoke-level)" -ForegroundColor Yellow
$hookDir = Join-Path $env:USERPROFILE ".actionguard\sessions"
New-Item -ItemType Directory -Path $hookDir -Force | Out-Null
$hookPath = Join-Path $hookDir "current.hook"
$closedPath = Join-Path $hookDir "current.closed"

$hookBak = $null; $closedBak = $null
if (Test-Path $hookPath)   { $hookBak = Get-Content $hookPath -Raw;   Remove-Item $hookPath -Force }
if (Test-Path $closedPath) { $closedBak = Get-Content $closedPath -Raw; Remove-Item $closedPath -Force }

$init = Join-Path $env:TEMP ("ag-init-" + [guid]::NewGuid().ToString("N") + ".ps1")
& $Binary init-powershell | Out-File -FilePath $init -Encoding utf8

$probe = @'
param([string]$Init, [string]$Closed)
$script = Get-Content -LiteralPath $Init -Raw
$cut = $script.IndexOf('# Phase C:')
if ($cut -lt 0) { $cut = $script.Length }
Invoke-Expression $script.Substring(0, $cut)
$r1 = Invoke-ActionGuardCheck -Cmd 'rm -rf /'
Write-Output ("NO-SESSION=" + $r1)
Set-Content -LiteralPath $Closed -Value 'closed'
$r2 = Invoke-ActionGuardCheck -Cmd 'rm -rf /'
Write-Output ("CLOSED-SENTINEL=" + $r2)
Remove-Item -LiteralPath $Closed -Force
'@
$probeFile = Join-Path $env:TEMP ("ag-probe-" + [guid]::NewGuid().ToString("N") + ".ps1")
Set-Content -Path $probeFile -Value $probe
$res = & powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $probeFile -Init $init -Closed $closedPath
$ns = ($res | Where-Object { $_ -match "NO-SESSION=(.*)$" } | ForEach-Object { $Matches[1] } | Select-Object -First 1)
$cs = ($res | Where-Object { $_ -match "CLOSED-SENTINEL=(.*)$" } | ForEach-Object { $Matches[1] } | Select-Object -First 1)
Assert "no session + no sentinel -> fail-closed BLOCK (got: $ns)" ($ns -eq "False")
Assert "clean-teardown sentinel -> allow (got: $cs)" ($cs -eq "True")

if ($hookBak)   { Set-Content -Path $hookPath -Value $hookBak }
if ($closedBak) { Set-Content -Path $closedPath -Value $closedBak }
Write-Host ""

# ---------------------------------------------------------------- cleanup ---
Write-Host "cleanup:" -ForegroundColor Yellow
if (Test-Path $userPolicy) {
    if ($null -ne $userPolicyBackup) { Set-Content -Path $userPolicy -Value $userPolicyBackup }
    else { Remove-Item $userPolicy -Force }
    Write-Host "  user policy restored/removed: $userPolicy"
}
Remove-Item $ruleFile -Force -ErrorAction SilentlyContinue
Remove-Item $init -Force -ErrorAction SilentlyContinue
Remove-Item $probeFile -Force -ErrorAction SilentlyContinue
Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "== summary ==" -ForegroundColor Cyan
$results | ForEach-Object { Write-Host "  $_" }
$fails = $results | Where-Object { $_ -like "FAIL*" }
if ($fails.Count -gt 0) {
    Write-Host "RESULT: FAIL ($($fails.Count) failed)" -ForegroundColor Red
    exit 1
}
Write-Host "RESULT: PASS (all checks)" -ForegroundColor Green
exit 0
