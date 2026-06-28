#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BRIDGE_DIR="$ROOT_DIR/.ai-bridge"
STATE_FILE="$BRIDGE_DIR/task-state.json"
PLAN_FILE="$BRIDGE_DIR/current-plan.md"
RUNNER_SCRIPT="$ROOT_DIR/scripts/ai-bridge-runner.sh"
LOG_FILE="$BRIDGE_DIR/execution-log.jsonl"
POLL_INTERVAL="${AI_BRIDGE_WATCH_INTERVAL_SECONDS:-5}"

log_event() {
  local event="$1"
  local status="$2"
  local message="$3"
  EVENT_NAME="$event" EVENT_STATUS="$status" EVENT_MESSAGE="$message" \
  node - "$LOG_FILE" <<'NODE'
const fs = require("fs");
const file = process.argv[2];
const entry = {
  ts: new Date().toISOString(),
  event: process.env.EVENT_NAME,
  status: process.env.EVENT_STATUS,
  message: process.env.EVENT_MESSAGE,
  source: "ai-bridge-watch",
};
fs.appendFileSync(file, JSON.stringify(entry) + "\n");
NODE
}

ensure_state_file() {
  if [[ ! -f "$STATE_FILE" ]]; then
    bash "$RUNNER_SCRIPT" >/dev/null 2>&1 || true
  fi
}

json_get_status() {
  node - "$STATE_FILE" <<'NODE'
const fs = require("fs");
const file = process.argv[2];
const data = JSON.parse(fs.readFileSync(file, "utf8"));
process.stdout.write(String(data.status || ""));
NODE
}

trigger_runner_if_ready() {
  ensure_state_file
  local status
  status="$(json_get_status)"
  if [[ "$status" == "ready_for_codex" ]]; then
    log_event "watcher_trigger" "$status" "Detected ready_for_codex; invoking runner."
    if ! bash "$RUNNER_SCRIPT"; then
      log_event "watcher_runner_failed" "failed" "Runner exited non-zero; watcher will continue waiting."
    fi
  fi
}

watch_with_fswatch() {
  echo "[ai-bridge-watch] Using fswatch."
  log_event "watcher_started" "watching" "Watcher started with fswatch."
  trigger_runner_if_ready
  fswatch -0 "$PLAN_FILE" "$STATE_FILE" | while IFS= read -r -d '' _event; do
    trigger_runner_if_ready
  done
}

watch_with_polling() {
  echo "[ai-bridge-watch] Using polling every ${POLL_INTERVAL}s."
  echo "[ai-bridge-watch] Install fswatch for event-based watching: brew install fswatch"
  log_event "watcher_started" "watching" "Watcher started with polling fallback."

  local last_plan_mtime=""
  local last_state_mtime=""

  while true; do
    local plan_mtime=""
    local state_mtime=""

    if [[ -f "$PLAN_FILE" ]]; then
      plan_mtime="$(stat -f '%m' "$PLAN_FILE" 2>/dev/null || stat -c '%Y' "$PLAN_FILE" 2>/dev/null || true)"
    fi
    if [[ -f "$STATE_FILE" ]]; then
      state_mtime="$(stat -f '%m' "$STATE_FILE" 2>/dev/null || stat -c '%Y' "$STATE_FILE" 2>/dev/null || true)"
    fi

    if [[ "$plan_mtime" != "$last_plan_mtime" || "$state_mtime" != "$last_state_mtime" ]]; then
      last_plan_mtime="$plan_mtime"
      last_state_mtime="$state_mtime"
      trigger_runner_if_ready
    fi

    sleep "$POLL_INTERVAL"
  done
}

main() {
  mkdir -p "$BRIDGE_DIR"

  if [[ ! -f "$RUNNER_SCRIPT" ]]; then
    echo "[ai-bridge-watch] Missing runner script: $RUNNER_SCRIPT" >&2
    exit 1
  fi

  if command -v fswatch >/dev/null 2>&1 && [[ "${AI_BRIDGE_FORCE_POLLING:-0}" != "1" ]]; then
    watch_with_fswatch
  else
    watch_with_polling
  fi
}

main "$@"
