#!/usr/bin/env bash
# ============================================================================
# ActionGuard — clean-machine install script (POSIX shell: bash/zsh/sh)
#
# Checks for the toolchain ActionGuard needs, builds the CLI from source, and
# runs `actionguard setup --yes` to install the shell hook + built-in rules.
#
# Usage:
#   ./scripts/install.sh              # interactive (prompts before setup)
#   ./scripts/install.sh --yes        # fully non-interactive (CI / fresh box)
#   ./scripts/install.sh --skip-build # use an existing actionguard on PATH
#
# Exit codes: 0 = installed, 1 = missing dependency, 2 = build failed,
#             3 = setup failed, 4 = usage error.
# ============================================================================
set -euo pipefail

# ---- config ---------------------------------------------------------------
ASSUME_YES=0
SKIP_BUILD=0
for arg in "$@"; do
  case "$arg" in
    --yes) ASSUME_YES=1 ;;
    --skip-build) SKIP_BUILD=1 ;;
    -h|--help)
      grep '^#' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown argument: $arg (see --help)" >&2
      exit 4
      ;;
  esac
done

say()  { printf '\033[1;32m[actionguard]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[actionguard]\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31m[actionguard]\033[0m %s\n' "$*" >&2; exit "${2:-1}"; }

# ---- path setup -----------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

# ---- dependency check -----------------------------------------------------
missing=()
for tool in node npm cargo python3; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    missing+=("$tool")
  fi
done

if ((${#missing[@]} > 0)); then
  warn "missing required tools: ${missing[*]}"
  cat <<'EOF'
Install them first, then re-run this script:

  Node.js 18+      https://nodejs.org/          (npm ships with it)
  Rust stable      https://rustup.rs/            (cargo ships with it)
  Python 3         https://www.python.org/       (optional; hooks work without it)

Windows? Use scripts\install.ps1 instead.
EOF
  exit 1
fi

# Version gates -------------------------------------------------------------
node_major="$(node -p 'process.versions.node.split(".")[0]')"
if ((node_major < 18)); then
  die "Node.js >= 18 required (found $node_major). Upgrade and re-run." 1
fi

say "toolchain OK: node $(node -v) · $(cargo --version)"

# ---- npm install ----------------------------------------------------------
say "installing frontend dependencies (npm install)…"
npm install --no-audit --no-fund

# ---- build CLI ------------------------------------------------------------
ACTIONGUARD=""
if ((SKIP_BUILD)); then
  ACTIONGUARD="$(command -v actionguard || true)"
  if [[ -z "$ACTIONGUARD" ]]; then
    die "--skip-build was given but no 'actionguard' binary is on PATH." 4
  fi
  say "using existing binary: $ACTIONGUARD"
else
  say "building CLI (cargo build --bin actionguard)…"
  (cd src-tauri && cargo build --bin actionguard) \
    || die "cargo build failed — see output above." 2
  ACTIONGUARD="$ROOT_DIR/src-tauri/target/debug/actionguard"
fi

# ---- setup ----------------------------------------------------------------
if ((ASSUME_YES)); then
  "$ACTIONGUARD" setup --yes || die "actionguard setup failed." 3
else
  "$ACTIONGUARD" setup || die "actionguard setup failed." 3
fi

# ---- report ---------------------------------------------------------------
say "installed. Verify with:"
echo
echo "  actionguard doctor          # policy / hook / bridge / boundary status"
echo "  actionguard capabilities    # what ActionGuard can actually do here"
echo "  actionguard boundary test   # non-destructive boundary verification"
echo
say "done."
