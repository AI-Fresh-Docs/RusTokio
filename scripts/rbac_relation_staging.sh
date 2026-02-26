#!/usr/bin/env bash
set -euo pipefail

# Staged RBAC relation migration helper for staging environments.
# Runs cleanup task targets in a safe sequence and stores logs/artifacts.

usage() {
  cat <<'USAGE'
Usage:
  scripts/rbac_relation_staging.sh [options]

Options:
  --env <name>                Loco environment (default: staging)
  --limit <N>                 Optional candidate limit for staged backfill
  --exclude-user-ids <list>   Comma-separated UUIDs to skip
  --exclude-roles <list>      Comma-separated legacy roles to skip
  --continue-on-error         Continue backfill/rollback on per-user failures
  --run-apply                 Run non-dry-run backfill step
  --run-rollback-dry          Run rollback dry-run after backfill
  --run-rollback-apply        Run actual rollback (dangerous; explicit)
  --artifacts-dir <dir>       Output folder for logs/report (default: artifacts/rbac-staging)
  --help                      Show this message

Examples:
  scripts/rbac_relation_staging.sh --run-apply --run-rollback-dry
  scripts/rbac_relation_staging.sh --limit 100 --exclude-roles super_admin --run-apply
USAGE
}

ENV_NAME="staging"
LIMIT=""
EXCLUDE_USER_IDS=""
EXCLUDE_ROLES=""
CONTINUE_ON_ERROR="false"
RUN_APPLY="false"
RUN_ROLLBACK_DRY="false"
RUN_ROLLBACK_APPLY="false"
ARTIFACTS_DIR="artifacts/rbac-staging"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --env)
      ENV_NAME="$2"; shift 2 ;;
    --limit)
      LIMIT="$2"; shift 2 ;;
    --exclude-user-ids)
      EXCLUDE_USER_IDS="$2"; shift 2 ;;
    --exclude-roles)
      EXCLUDE_ROLES="$2"; shift 2 ;;
    --continue-on-error)
      CONTINUE_ON_ERROR="true"; shift ;;
    --run-apply)
      RUN_APPLY="true"; shift ;;
    --run-rollback-dry)
      RUN_ROLLBACK_DRY="true"; shift ;;
    --run-rollback-apply)
      RUN_ROLLBACK_APPLY="true"; shift ;;
    --artifacts-dir)
      ARTIFACTS_DIR="$2"; shift 2 ;;
    --help)
      usage; exit 0 ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1 ;;
  esac
done

mkdir -p "$ARTIFACTS_DIR"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
ROLLBACK_FILE="$ARTIFACTS_DIR/rbac_backfill_${TS}.rollback.json"
REPORT_FILE="$ARTIFACTS_DIR/rbac_relation_stage_report_${TS}.md"

build_args() {
  local target="$1"
  local args="target=${target}"

  if [[ -n "$LIMIT" ]]; then
    args+=" limit=${LIMIT}"
  fi
  if [[ -n "$EXCLUDE_USER_IDS" ]]; then
    args+=" exclude_user_ids=${EXCLUDE_USER_IDS}"
  fi
  if [[ -n "$EXCLUDE_ROLES" ]]; then
    args+=" exclude_roles=${EXCLUDE_ROLES}"
  fi
  if [[ "$CONTINUE_ON_ERROR" == "true" ]]; then
    args+=" continue_on_error=true"
  fi

  echo "$args"
}

run_step() {
  local name="$1"
  local args="$2"
  local log_file="$ARTIFACTS_DIR/${TS}_${name}.log"

  echo "==> ${name}: cargo loco task --name cleanup --env ${ENV_NAME} --args \"${args}\""
  cargo loco task --name cleanup --env "$ENV_NAME" --args "$args" | tee "$log_file"
}

# 1) Baseline
run_step "01_pre_report" "target=rbac-report"

# 2) Dry-run backfill
run_step "02_backfill_dry_run" "$(build_args rbac-backfill) dry_run=true rollback_file=${ROLLBACK_FILE}"

# 3) Apply backfill (optional)
if [[ "$RUN_APPLY" == "true" ]]; then
  run_step "03_backfill_apply" "$(build_args rbac-backfill) rollback_file=${ROLLBACK_FILE}"
  run_step "04_post_report" "target=rbac-report"
else
  echo "Skipping apply step (use --run-apply to enable)."
fi

# 4) Rollback dry-run (optional)
if [[ "$RUN_ROLLBACK_DRY" == "true" ]]; then
  run_step "05_rollback_dry_run" "target=rbac-backfill-rollback source=${ROLLBACK_FILE} dry_run=true"
fi

# 5) Rollback apply (optional, explicit)
if [[ "$RUN_ROLLBACK_APPLY" == "true" ]]; then
  run_step "06_rollback_apply" "target=rbac-backfill-rollback source=${ROLLBACK_FILE} continue_on_error=${CONTINUE_ON_ERROR}"
  run_step "07_post_rollback_report" "target=rbac-report"
fi

cat > "$REPORT_FILE" <<REPORT
# RBAC relation staged migration report

- Timestamp (UTC): ${TS}
- Environment: ${ENV_NAME}
- Artifacts directory: ${ARTIFACTS_DIR}
- Rollback snapshot: ${ROLLBACK_FILE}
- Apply step enabled: ${RUN_APPLY}
- Rollback dry-run enabled: ${RUN_ROLLBACK_DRY}
- Rollback apply enabled: ${RUN_ROLLBACK_APPLY}
- Limit: ${LIMIT:-<none>}
- Excluded user IDs: ${EXCLUDE_USER_IDS:-<none>}
- Excluded roles: ${EXCLUDE_ROLES:-<none>}
- Continue on error: ${CONTINUE_ON_ERROR}

## Generated logs

$(for f in "$ARTIFACTS_DIR"/${TS}_*.log; do
  [[ -e "$f" ]] || continue
  echo "- $(basename "$f")"
done)
REPORT

echo "Done. Report: ${REPORT_FILE}"
echo "Rollback snapshot: ${ROLLBACK_FILE}"
