#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERIFY_ALL="$REPO_ROOT/scripts/verify/verify-all.sh"

if [[ ! -f "$VERIFY_ALL" ]]; then
  echo "verify-all script not found: $VERIFY_ALL" >&2
  exit 1
fi

OUTPUT_FILE="$(mktemp)"
if (cd "$REPO_ROOT" && env RUSTOK_VERIFY_SKIP_FMT=1 timeout 8s "$VERIFY_ALL" -v control-plane-remediation-minimal >"$OUTPUT_FILE" 2>&1); then
  :
fi

if ! rg -q "Control Plane Remediation Minimal" "$OUTPUT_FILE"; then
  echo "verify-all did not pick control-plane-remediation-minimal alias" >&2
  cat "$OUTPUT_FILE" >&2
  exit 1
fi

if ! rg -q "==> migration tests" "$OUTPUT_FILE"; then
  echo "runner did not start through verify-all alias" >&2
  cat "$OUTPUT_FILE" >&2
  exit 1
fi

echo "control_plane_remediation_verify_all_alias_test.sh: PASS"
