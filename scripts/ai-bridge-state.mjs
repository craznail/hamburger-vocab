#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");
const bridgeDir = path.join(rootDir, ".ai-bridge");
const stateFile = path.join(bridgeDir, "task-state.json");
const planFile = path.join(bridgeDir, "current-plan.md");
const reportFile = path.join(bridgeDir, "codex-report.md");
const reviewFile = path.join(bridgeDir, "review-notes.md");
const runnerFile = path.join(rootDir, "scripts", "ai-bridge-runner.sh");
const watchFile = path.join(rootDir, "scripts", "ai-bridge-watch.sh");
const logFile = path.join(bridgeDir, "execution-log.jsonl");

const allowedStatuses = new Set([
  "idle",
  "ready_for_codex",
  "running",
  "codex_done",
  "waiting_for_chatgpt_review",
  "review_done",
  "failed",
]);

const defaultState = {
  taskId: "manual-current-task",
  status: "idle",
  planPath: ".ai-bridge/current-plan.md",
  reportPath: ".ai-bridge/codex-report.md",
  reviewPath: ".ai-bridge/review-notes.md",
  agent: "codex",
  requiresReview: true,
  autoCommit: false,
  lastUpdatedAt: "",
  lastRunAt: "",
  lastError: "",
  lastReportFingerprint: null,
};

function fail(message, exitCode = 1) {
  console.error(`[ai-bridge-state] ${message}`);
  process.exit(exitCode);
}

function parseArgs(argv) {
  const [command, ...rest] = argv;
  const options = {};

  for (let i = 0; i < rest.length; i += 1) {
    const token = rest[i];
    if (!token.startsWith("--")) {
      fail(`Unexpected argument: ${token}`);
    }

    const key = token.slice(2);
    const next = rest[i + 1];
    if (!next || next.startsWith("--")) {
      options[key] = true;
      continue;
    }

    options[key] = next;
    i += 1;
  }

  return { command, options };
}

function readJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch (error) {
    fail(`Failed to read JSON from ${path.relative(rootDir, filePath)}: ${error.message}`);
  }
}

function readState() {
  if (!fs.existsSync(stateFile)) {
    fail("Missing .ai-bridge/task-state.json. Run doctor or restore the bridge files first.");
  }

  const state = readJson(stateFile);
  if (!allowedStatuses.has(state.status)) {
    fail(`Invalid task-state status: ${String(state.status)}`);
  }
  return state;
}

function writeState(nextState) {
  const merged = {
    ...defaultState,
    ...nextState,
    lastUpdatedAt: new Date().toISOString(),
  };
  fs.writeFileSync(stateFile, `${JSON.stringify(merged, null, 2)}\n`);
}

function appendLog(event, status, message, extra = {}) {
  const entry = {
    ts: new Date().toISOString(),
    event,
    status,
    message,
    source: "ai-bridge-state",
    ...extra,
  };
  fs.appendFileSync(logFile, `${JSON.stringify(entry)}\n`);
}

function appendReviewRecord({ taskId, result, summary, previousStatus, nextStatus }) {
  const lines = [
    "",
    "## Scripted Review Record",
    "",
    `- Time: ${new Date().toISOString()}`,
    `- Task ID: ${taskId}`,
    `- Result: ${result}`,
    `- Previous status: ${previousStatus}`,
    `- Next status: ${nextStatus}`,
    `- Summary: ${summary}`,
  ];
  fs.appendFileSync(reviewFile, `${lines.join("\n")}\n`);
}

function formatValue(value) {
  if (value === null) {
    return "null";
  }
  if (value === undefined || value === "") {
    return "";
  }
  if (typeof value === "object") {
    return JSON.stringify(value);
  }
  return String(value);
}

function printStatus(state) {
  const fields = [
    ["status", state.status],
    ["taskId", state.taskId],
    ["agent", state.agent],
    ["requiresReview", state.requiresReview],
    ["autoCommit", state.autoCommit],
    ["lastUpdatedAt", state.lastUpdatedAt],
    ["lastRunAt", state.lastRunAt],
    ["lastError", state.lastError],
    ["planPath", state.planPath],
    ["reportPath", state.reportPath],
    ["reviewPath", state.reviewPath],
    ["lastReportFingerprint", state.lastReportFingerprint],
  ];

  for (const [label, value] of fields) {
    console.log(`${label}: ${formatValue(value)}`);
  }
}

function requireStringOption(options, key) {
  const value = options[key];
  if (typeof value !== "string" || value.trim() === "") {
    fail(`Missing required option --${key}`);
  }
  return value.trim();
}

function commandStatus() {
  printStatus(readState());
}

function commandReady(options) {
  const state = readState();
  const taskId = requireStringOption(options, "task-id");
  const force = options.force === true;

  const canEnterReady = new Set(["idle", "review_done", "failed"]);
  const forceOnlyReady = new Set(["waiting_for_chatgpt_review"]);

  if (state.status === "running") {
    fail("Cannot move from running to ready_for_codex.");
  }

  if (forceOnlyReady.has(state.status) && !force) {
    fail("Cannot move from waiting_for_chatgpt_review to ready_for_codex without --force.");
  }

  if (!canEnterReady.has(state.status) && !(forceOnlyReady.has(state.status) && force)) {
    fail(`Cannot move from ${state.status} to ready_for_codex.`);
  }

  const nextState = {
    ...state,
    taskId,
    status: "ready_for_codex",
    requiresReview: true,
    autoCommit: false,
    lastError: "",
    lastReportFingerprint: null,
  };

  writeState(nextState);
  appendLog("state_ready", "ready_for_codex", "Marked task ready for Codex.", {
    taskId,
    force,
  });
  printStatus(readState());
}

function commandReviewDone(options) {
  const state = readState();
  const result = requireStringOption(options, "result");
  const summary = requireStringOption(options, "summary");

  if (!["accepted", "needs_changes", "rejected"].includes(result)) {
    fail("Invalid --result. Expected accepted, needs_changes, or rejected.");
  }

  if (!["waiting_for_chatgpt_review", "codex_done"].includes(state.status)) {
    fail(`Cannot mark review done from ${state.status}.`);
  }

  const nextStatus = result === "rejected" ? "failed" : "review_done";
  const nextLastError = result === "rejected" ? `Review rejected: ${summary}` : "";

  appendReviewRecord({
    taskId: state.taskId,
    result,
    summary,
    previousStatus: state.status,
    nextStatus,
  });

  writeState({
    ...state,
    status: nextStatus,
    lastError: nextLastError,
  });

  appendLog(
    result === "rejected" ? "state_review_rejected" : "state_review_done",
    nextStatus,
    `Review recorded with result=${result}.`,
    {
      taskId: state.taskId,
      result,
      summary,
    },
  );

  printStatus(readState());
}

function commandFail(options) {
  const state = readState();
  const message = requireStringOption(options, "message");

  writeState({
    ...state,
    status: "failed",
    lastError: message,
  });

  appendLog("state_failed", "failed", message, { taskId: state.taskId });
  printStatus(readState());
}

function commandReset(options) {
  const state = readState();
  const target = requireStringOption(options, "to");
  const force = options.force === true;

  if (target !== "idle") {
    fail("Only --to idle is supported.");
  }

  const allowedWithoutForce = new Set(["failed", "review_done", "idle"]);
  if (!allowedWithoutForce.has(state.status) && !force) {
    fail(`Cannot reset from ${state.status} to idle without --force.`);
  }

  writeState({
    ...state,
    status: "idle",
    lastError: "",
    lastReportFingerprint: null,
  });

  appendLog("state_reset", "idle", "State reset to idle.", {
    taskId: state.taskId,
    force,
  });
  printStatus(readState());
}

function commandDoctor() {
  const checks = [
    [".ai-bridge/", fs.existsSync(bridgeDir)],
    [".ai-bridge/task-state.json", fs.existsSync(stateFile)],
    [".ai-bridge/current-plan.md", fs.existsSync(planFile)],
    [".ai-bridge/codex-report.md", fs.existsSync(reportFile)],
    [".ai-bridge/review-notes.md", fs.existsSync(reviewFile)],
    ["scripts/ai-bridge-runner.sh", fs.existsSync(runnerFile)],
    ["scripts/ai-bridge-watch.sh", fs.existsSync(watchFile)],
  ];

  const failures = checks.filter(([, ok]) => !ok);
  if (failures.length > 0) {
    for (const [label] of failures) {
      console.error(`ERROR ${label} is missing.`);
    }
    process.exit(1);
  }

  const state = readJson(stateFile);
  const errors = [];

  if (!allowedStatuses.has(state.status)) {
    errors.push(`Invalid status: ${String(state.status)}`);
  }
  if (state.autoCommit !== false) {
    errors.push("autoCommit must be false.");
  }
  if (state.requiresReview !== true) {
    errors.push("requiresReview must be true.");
  }
  if (
    state.lastReportFingerprint !== null &&
    (typeof state.lastReportFingerprint !== "object" || Array.isArray(state.lastReportFingerprint))
  ) {
    errors.push("lastReportFingerprint must be null or an object.");
  }

  if (errors.length > 0) {
    for (const error of errors) {
      console.error(`ERROR ${error}`);
    }
    process.exit(1);
  }

  console.log("OK .ai-bridge environment is healthy.");
  console.log(`OK status=${state.status}`);
  console.log("OK autoCommit=false");
  console.log("OK requiresReview=true");
}

function main() {
  const { command, options } = parseArgs(process.argv.slice(2));

  switch (command) {
    case "status":
      commandStatus();
      break;
    case "ready":
      commandReady(options);
      break;
    case "review-done":
      commandReviewDone(options);
      break;
    case "fail":
      commandFail(options);
      break;
    case "reset":
      commandReset(options);
      break;
    case "doctor":
      commandDoctor();
      break;
    default:
      fail(
        "Usage: node scripts/ai-bridge-state.mjs <status|ready|review-done|fail|reset|doctor> [options]",
      );
  }
}

main();
