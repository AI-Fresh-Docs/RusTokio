#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_SCRIPT="$REPO_ROOT/scripts/verify/report-control-plane-remediation-progress.py"

FIXTURE_ROOT="$(mktemp -d)"
trap 'rm -rf "$FIXTURE_ROOT"' EXIT

PLAN_FIXTURE="$FIXTURE_ROOT/plan.md"
cat > "$PLAN_FIXTURE" <<'MD'
- [x] done one
- [~] in progress one
- [ ] pending one
- [~] in progress two
MD

OUTPUT="$(RUSTOK_REMEDIATION_PLAN_PATH="$PLAN_FIXTURE" python3 "$REPORT_SCRIPT")"

if ! grep -q "completed: 1" <<<"$OUTPUT"; then
  echo "expected completed count missing" >&2
  echo "$OUTPUT" >&2
  exit 1
fi
if ! grep -q "in_progress: 2" <<<"$OUTPUT"; then
  echo "expected in_progress count missing" >&2
  echo "$OUTPUT" >&2
  exit 1
fi
if ! grep -q "pending: 1" <<<"$OUTPUT"; then
  echo "expected pending count missing" >&2
  echo "$OUTPUT" >&2
  exit 1
fi

echo "control_plane_remediation_progress_report_test.sh: PASS"
