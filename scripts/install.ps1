# ============================================================================
# ActionGuard - clean-machine install script (Windows PowerShell)
#
# Checks for the toolchain ActionGuard needs, builds the CLI from source, and
# runs `actionguard setup --yes` to install the PowerShell hook + built-in
# rules. Non-interactive by default (setup is --yes) so it can run on a fresh
# box / CI.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\install.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\install.ps1 -SkipBuild
#
# Exit codes: 0 = installed, 1 = missing dependency, 2 = build failed,
#             3 = setup failed, 4 = usage error.
# ============================================================================
[CmdletBinding()]
param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$WarningPreference = "Continue"

function Write-Step([string]$Msg) { Write-Host "[actionguard] $Msg" -ForegroundColor Green }
function Write-WarnLine([string]$Msg) { Write-Host "[actionguard] $Msg" -ForegroundColor Yellow }

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root

# ---- dependency check -----------------------------------------------------
$missing = @()
foreach ($tool in @("node", "npm", "cargo")) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        $missing += $tool
    }
}

if ($missing.Count -gt 0) {
    Write-WarnLine "missing required tools: $($missing -join ', ')"
    @"
Install them first, then re-run this script:

  Node.js 18+   https://nodejs.org/      (npm ships with it)
  Rust stable   https://rustup.rs/       (cargo ships with it)
  Python 3      https://www.python.org/  (optional; hooks work without it)

POSIX shell? Use scripts/install.sh instead.
"@ | Write-Host -ForegroundColor Yellow
    exit 1
}

# Version gates -------------------------------------------------------------
$nodeMajor = node -p "process.versions.node.split('.')[0]"
if ([int]$nodeMajor -lt 18) {
    Write-Host "[actionguard] Node.js >= 18 required (found $nodeMajor)." -ForegroundColor Red
    exit 1
}

$cargoVersion = (cargo --version)
Write-Step "toolchain OK: node $(node --version) / $cargoVersion"

# ---- npm install ----------------------------------------------------------
Write-Step "installing frontend dependencies (npm install)..."
npm install --no-audit --no-fund
if ($LASTEXITCODE -ne 0) { exit 2 }

# ---- build CLI ------------------------------------------------------------
$ActionGuard = ""
if ($SkipBuild) {
    $ActionGuard = (Get-Command actionguard -ErrorAction SilentlyContinue)
    if (-not $ActionGuard) {
        Write-Host "[actionguard] -SkipBuild was given but no 'actionguard' on PATH." -ForegroundColor Red
        exit 4
    }
    Write-Step "using existing binary: $($ActionGuard.Source)"
    $ActionGuard = $ActionGuard.Source
}
else {
    Write-Step "building CLI (cargo build --bin actionguard)..."
    Push-Location "$Root\src-tauri"
    try {
        cargo build --bin actionguard
        if ($LASTEXITCODE -ne 0) { exit 2 }
    }
    finally {
        Pop-Location
    }
    $ActionGuard = Join-Path $Root "src-tauri\target\debug\actionguard.exe"
}

# ---- setup ----------------------------------------------------------------
Write-Step "running actionguard setup --yes..."
& $ActionGuard setup --yes
if ($LASTEXITCODE -ne 0) { exit 3 }

# ---- report ---------------------------------------------------------------
Write-Step "installed. Verify with:"
""
"  actionguard doctor          # policy / hook / bridge / boundary status"
"  actionguard capabilities    # what ActionGuard can actually do here"
"  actionguard boundary test   # non-destructive boundary verification"
""
Write-Step "done."
