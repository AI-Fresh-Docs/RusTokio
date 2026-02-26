#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/rbac_cutover_baseline.sh"

fail() {
  echo "[FAIL] $*" >&2
  exit 1
}

pass() {
  echo "[PASS] $*"
}

make_mock_curl() {
  local dir="$1"
  cat > "$dir/mock-curl" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail

state_file="${MOCK_CURL_STATE_FILE:-}"
if [[ -z "$state_file" ]]; then
  echo "MOCK_CURL_STATE_FILE is required" >&2
  exit 1
fi

count=0
if [[ -f "$state_file" ]]; then
  count="$(cat "$state_file")"
fi
count=$((count + 1))
printf '%s' "$count" > "$state_file"

profile="${MOCK_CURL_PROFILE:-steady}"
case "$profile" in
  mismatch)
    mismatch="$count"
    ;;
  steady|*)
    mismatch="0"
    ;;
esac

cat <<METRICS
rustok_rbac_decision_mismatch_total ${mismatch}
rustok_rbac_shadow_compare_failures_total 0
rustok_rbac_permission_checks_denied $((2 * count))
rustok_rbac_permission_checks_allowed $((10 * count))
METRICS
MOCK
  chmod +x "$dir/mock-curl"
}

test_baseline_passes_when_mismatch_is_stable() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_curl "$tmp"

  MOCK_CURL_STATE_FILE="$tmp/state" MOCK_CURL_PROFILE=steady RUSTOK_CURL_BIN="$tmp/mock-curl" "$SCRIPT" \
    --samples 3 --interval-sec 0 --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1

  rg -q "Done. Report:" "$tmp/out.log" || fail "expected report output"
  report="$(rg -o 'Done\. Report: .*' "$tmp/out.log" | sed 's/Done\. Report: //')"
  rg -q "status: pass" "$report" || fail "expected pass gate in report"
  pass "baseline report passes with stable mismatch"
}

test_baseline_fails_when_mismatch_changes() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_curl "$tmp"

  set +e
  MOCK_CURL_STATE_FILE="$tmp/state" MOCK_CURL_PROFILE=mismatch RUSTOK_CURL_BIN="$tmp/mock-curl" "$SCRIPT" \
    --samples 3 --interval-sec 0 --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1
  code=$?
  set -e

  [[ "$code" -eq 1 ]] || fail "expected non-zero exit when mismatch delta is non-zero"
  rg -q "Mismatch delta is" "$tmp/out.log" || fail "expected mismatch gate message"
  pass "baseline helper enforces zero mismatch gate"
}

test_allow_mismatch_disables_strict_gate() {
  local tmp
  tmp="$(mktemp -d)"
  make_mock_curl "$tmp"

  MOCK_CURL_STATE_FILE="$tmp/state" MOCK_CURL_PROFILE=mismatch RUSTOK_CURL_BIN="$tmp/mock-curl" "$SCRIPT" \
    --samples 2 --interval-sec 0 --allow-mismatch --artifacts-dir "$tmp/artifacts" >"$tmp/out.log" 2>&1

  rg -q "Done. Report:" "$tmp/out.log" || fail "expected successful output with --allow-mismatch"
  pass "allow-mismatch flag bypasses strict gate"
}

test_baseline_passes_when_mismatch_is_stable
test_baseline_fails_when_mismatch_changes
test_allow_mismatch_disables_strict_gate

echo "All rbac_cutover_baseline tests passed"
