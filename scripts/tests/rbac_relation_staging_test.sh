#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/rbac_relation_staging.sh"

fail() {
  echo "[FAIL] $*" >&2
  exit 1
}

pass() {
  echo "[PASS] $*"
}

make_mock_cargo() {
  local dir="$1"
  cat > "$dir/mock-cargo" <<'MOCK'
#!/usr/bin/env bash
echo "mock cargo $*"
if [[ -n "${MOCK_TOUCH_ROLLBACK_FILE:-}" && "$*" == *"target=rbac-backfill"* && "$*" != *"dry_run=true"* ]]; then
  file="$(printf '%s' "$*" | sed -n 's/.*rollback_file=\([^ ]*\).*/\1/p')"
  if [[ -n "$file" ]]; then
    mkdir -p "$(dirname "$file")"
    echo "[]" > "$file"
  fi
fi
MOCK
  chmod +x "$dir/mock-cargo"
}

test_missing_rollback_source_fails() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_cargo "$tmp"

  set +e
  RUSTOK_CARGO_BIN="$tmp/mock-cargo" "$SCRIPT" --run-rollback-dry --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1
  local code=$?
  set -e

  [[ $code -eq 1 ]] || fail "expected exit 1 when rollback source is missing"
  rg -q "Rollback source file is required" "$tmp/out.log" || fail "expected missing rollback source message"
  pass "rollback dry-run without snapshot fails fast"
}

test_rollback_source_allows_dry_run() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_cargo "$tmp"
  echo '[]' > "$tmp/existing.rollback.json"

  RUSTOK_CARGO_BIN="$tmp/mock-cargo" "$SCRIPT" --run-rollback-dry --rollback-source "$tmp/existing.rollback.json" --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1

  rg -q "target=rbac-backfill-rollback source=$tmp/existing.rollback.json dry_run=true" "$tmp/out.log" || fail "expected rollback dry-run to use provided source"
  rg -q "Done. Report:" "$tmp/out.log" || fail "expected report generation"
  pass "rollback dry-run uses provided snapshot source"
}

test_apply_creates_snapshot_and_rollback_apply_uses_it() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_cargo "$tmp"

  MOCK_TOUCH_ROLLBACK_FILE=1 RUSTOK_CARGO_BIN="$tmp/mock-cargo" "$SCRIPT" --run-apply --run-rollback-apply --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1

  rg -q "target=rbac-backfill-rollback source=$tmp/artifacts/rbac_backfill_" "$tmp/out.log" || fail "expected rollback apply to use generated snapshot"
  rg -q "continue_on_error=false" "$tmp/out.log" || fail "expected rollback apply args to include continue_on_error"
  pass "apply+rollback apply path uses generated snapshot"
}

main() {
  test_missing_rollback_source_fails
  test_rollback_source_allows_dry_run
  test_apply_creates_snapshot_and_rollback_apply_uses_it
  echo "All tests passed."
}

main
