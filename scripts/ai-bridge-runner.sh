#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BRIDGE_DIR="$ROOT_DIR/.ai-bridge"
STATE_FILE="$BRIDGE_DIR/task-state.json"
PLAN_FILE="$BRIDGE_DIR/current-plan.md"
REPORT_FILE="$BRIDGE_DIR/codex-report.md"
LOG_FILE="$BRIDGE_DIR/execution-log.jsonl"
LOCK_DIR="$BRIDGE_DIR/.runner.lock"
RUNNER_LAST_MESSAGE_FILE="$BRIDGE_DIR/.runner-last-message.txt"

DEFAULT_STATE_JSON='{
  "taskId": "manual-current-task",
  "status": "idle",
  "planPath": ".ai-bridge/current-plan.md",
  "reportPath": ".ai-bridge/codex-report.md",
  "reviewPath": ".ai-bridge/review-notes.md",
  "agent": "codex",
  "requiresReview": true,
  "autoCommit": false,
  "lastUpdatedAt": "",
  "lastRunAt": "",
  "lastError": "",
  "lastReportFingerprint": null
}'

timestamp() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

ensure_bridge_dir() {
  mkdir -p "$BRIDGE_DIR"
}

ensure_state_file() {
  ensure_bridge_dir
  if [[ ! -f "$STATE_FILE" ]]; then
    printf '%s\n' "$DEFAULT_STATE_JSON" > "$STATE_FILE"
  fi
}

json_get() {
  local key="$1"
  node - "$STATE_FILE" "$key" <<'NODE'
const fs = require("fs");
const file = process.argv[2];
const key = process.argv[3];
const data = JSON.parse(fs.readFileSync(file, "utf8"));
const value = data[key];
if (value === undefined || value === null) {
  process.stdout.write("");
} else if (typeof value === "object") {
  process.stdout.write(JSON.stringify(value));
} else {
  process.stdout.write(String(value));
}
NODE
}

json_patch() {
  local patch_json="$1"
  PATCH_JSON="$patch_json" node - "$STATE_FILE" <<'NODE'
const fs = require("fs");
const file = process.argv[2];
const patch = JSON.parse(process.env.PATCH_JSON || "{}");
const data = JSON.parse(fs.readFileSync(file, "utf8"));
const next = { ...data, ...patch, lastUpdatedAt: new Date().toISOString() };
fs.writeFileSync(file, JSON.stringify(next, null, 2) + "\n");
NODE
}

log_event() {
  local event="$1"
  local status="$2"
  local message="$3"
  ensure_bridge_dir
  EVENT_NAME="$event" EVENT_STATUS="$status" EVENT_MESSAGE="$message" \
  node - "$LOG_FILE" <<'NODE'
const fs = require("fs");
const file = process.argv[2];
const entry = {
  ts: new Date().toISOString(),
  event: process.env.EVENT_NAME,
  status: process.env.EVENT_STATUS,
  message: process.env.EVENT_MESSAGE,
  source: "ai-bridge-runner",
};
fs.appendFileSync(file, JSON.stringify(entry) + "\n");
NODE
}

report_fingerprint() {
  node - "$REPORT_FILE" <<'NODE'
const crypto = require("crypto");
const fs = require("fs");
const file = process.argv[2];

if (!fs.existsSync(file)) {
  process.stdout.write(JSON.stringify({ exists: false, size: 0, sha256: "" }));
  process.exit(0);
}

const stat = fs.statSync(file);
const content = fs.readFileSync(file);
const sha256 = crypto.createHash("sha256").update(content).digest("hex");
process.stdout.write(JSON.stringify({ exists: stat.size > 0, size: stat.size, sha256 }));
NODE
}

report_fingerprint_exists_and_nonempty() {
  local fingerprint_json="$1"
  FINGERPRINT_JSON="$fingerprint_json" node <<'NODE'
const fingerprint = JSON.parse(process.env.FINGERPRINT_JSON || "{}");
process.exit(fingerprint.exists && Number(fingerprint.size || 0) > 0 ? 0 : 1);
NODE
}

resolve_codex_cmd() {
  if [[ -n "${AI_BRIDGE_CODEX_CMD:-}" ]]; then
    printf '%s\n' "$AI_BRIDGE_CODEX_CMD"
    return 0
  fi

  if [[ "${AI_BRIDGE_ENABLE_DEFAULT_CODEX:-0}" == "1" ]] && command -v codex >/dev/null 2>&1; then
    printf '%s\n' "codex exec -C \"$ROOT_DIR\" -a never -s workspace-write --output-last-message \"$RUNNER_LAST_MESSAGE_FILE\" -"
    return 0
  fi

  return 1
}

print_manual_hint() {
  cat <<EOF
[ai-bridge-runner] No executable Codex command is configured.
Either set AI_BRIDGE_CODEX_CMD, or explicitly allow the default codex command:

  AI_BRIDGE_ENABLE_DEFAULT_CODEX=1 bash scripts/ai-bridge-runner.sh

Recommended explicit command:

  AI_BRIDGE_CODEX_CMD='codex exec -C "$ROOT_DIR" -a never -s workspace-write -' bash scripts/ai-bridge-runner.sh

Current plan: $PLAN_FILE
EOF
}

build_prompt() {
  cat <<EOF
请在工作区 $ROOT_DIR 中执行 $PLAN_FILE 的当前任务。

要求：
- 先阅读 .ai-bridge/current-plan.md 以及相关 .ai-bridge 状态文件。
- 按计划完成实现、验证和报告更新。
- 必须更新 .ai-bridge/codex-report.md 和 .ai-bridge/agent-status.md。
- 需要时更新 .ai-bridge/execution-log.jsonl。
- 不自动 commit。
- 不自动 push。
EOF
}

main() {
  ensure_state_file

  local status
  status="$(json_get status)"
  if [[ "$status" != "ready_for_codex" ]]; then
    echo "[ai-bridge-runner] Skip: task status is '$status', not 'ready_for_codex'."
    log_event "runner_skip" "$status" "Task not ready for Codex."
    exit 0
  fi

  if [[ -d "$LOCK_DIR" ]]; then
    echo "[ai-bridge-runner] Skip: runner lock exists at $LOCK_DIR."
    log_event "runner_skip" "$status" "Runner lock already exists."
    exit 0
  fi

  mkdir "$LOCK_DIR"
  trap 'rmdir "$LOCK_DIR" >/dev/null 2>&1 || true' EXIT

  local codex_cmd
  if ! codex_cmd="$(resolve_codex_cmd)"; then
    print_manual_hint
    json_patch "{\"lastError\":\"No Codex command configured. Set AI_BRIDGE_CODEX_CMD or enable AI_BRIDGE_ENABLE_DEFAULT_CODEX=1.\",\"status\":\"failed\"}"
    log_event "runner_failed" "failed" "No Codex command configured."
    exit 1
  fi

  if [[ "${AI_BRIDGE_DRY_RUN:-0}" == "1" ]]; then
    echo "[ai-bridge-runner] Dry run enabled. Would execute:"
    printf '  %s\n' "$codex_cmd"
    log_event "runner_dry_run" "ready_for_codex" "Dry run; Codex command not executed."
    exit 0
  fi

  local run_started_at
  run_started_at="$(timestamp)"
  local report_fingerprint_before
  report_fingerprint_before="$(report_fingerprint)"

  json_patch "{\"status\":\"running\",\"lastRunAt\":\"$run_started_at\",\"lastError\":\"\",\"lastReportFingerprint\":$(printf '%s' "$report_fingerprint_before" | node -p 'JSON.stringify(JSON.parse(require("fs").readFileSync(0,"utf8")))' )}"
  log_event "runner_started" "running" "Starting Codex runner."

  local exit_code=0
  set +e
  build_prompt | /bin/bash -lc "$codex_cmd"
  exit_code=$?
  set -e

  if [[ $exit_code -ne 0 ]]; then
    json_patch "{\"status\":\"failed\",\"lastError\":\"Codex command exited with code $exit_code.\"}"
    log_event "runner_failed" "failed" "Codex command exited with code $exit_code."
    echo "[ai-bridge-runner] Failed: Codex command exited with code $exit_code."
    exit $exit_code
  fi

  local report_fingerprint_after
  report_fingerprint_after="$(report_fingerprint)"

  if ! report_fingerprint_exists_and_nonempty "$report_fingerprint_after"; then
    json_patch "{\"status\":\"failed\",\"lastError\":\"Codex finished but report file is missing or empty: $REPORT_FILE\",\"lastReportFingerprint\":$(printf '%s' "$report_fingerprint_after" | node -p 'JSON.stringify(JSON.parse(require("fs").readFileSync(0,"utf8")))' )}"
    log_event "runner_failed" "failed" "Report file missing or empty after Codex run."
    echo "[ai-bridge-runner] Failed: report file is missing or empty."
    exit 1
  fi

  if [[ "$report_fingerprint_before" == "$report_fingerprint_after" ]]; then
    json_patch "{\"status\":\"failed\",\"lastError\":\"Codex finished but codex-report.md was not updated for this run.\",\"lastReportFingerprint\":$(printf '%s' "$report_fingerprint_after" | node -p 'JSON.stringify(JSON.parse(require("fs").readFileSync(0,"utf8")))' )}"
    log_event "runner_failed" "failed" "Report fingerprint did not change after Codex run."
    echo "[ai-bridge-runner] Failed: report fingerprint did not change."
    exit 1
  fi

  json_patch "{\"status\":\"codex_done\",\"lastError\":\"\",\"lastReportFingerprint\":$(printf '%s' "$report_fingerprint_after" | node -p 'JSON.stringify(JSON.parse(require("fs").readFileSync(0,"utf8")))' )}"
  log_event "runner_codex_done" "codex_done" "Codex finished and report file is present."

  local requires_review
  requires_review="$(json_get requiresReview)"
  if [[ "$requires_review" == "true" ]]; then
    json_patch "{\"status\":\"waiting_for_chatgpt_review\"}"
    log_event "runner_waiting_for_review" "waiting_for_chatgpt_review" "Waiting for ChatGPT review."
  else
    json_patch "{\"status\":\"review_done\"}"
    log_event "runner_review_done" "review_done" "Review not required; task marked review_done."
  fi

  echo "[ai-bridge-runner] Completed successfully."
}

main "$@"
