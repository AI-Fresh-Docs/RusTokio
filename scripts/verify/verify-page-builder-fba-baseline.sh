#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "[verify-page-builder-fba-baseline] running contract parity check"
node "$SCRIPT_DIR/verify-page-builder-contract-parity.mjs"

echo "[verify-page-builder-fba-baseline] running fallback profiles check"
node "$SCRIPT_DIR/verify-page-builder-fallback-profiles.mjs"

echo "[verify-page-builder-fba-baseline] running toggle profiles consistency check"
node "$SCRIPT_DIR/verify-page-builder-toggle-profiles-consistency.mjs"

echo "[verify-page-builder-fba-baseline] PASS"
