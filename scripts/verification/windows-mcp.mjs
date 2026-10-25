#!/usr/bin/env node
/**
 * Exercise real aidget MCP stdio without a model or an Agent installation.
 * Usage: node windows-mcp.mjs <absolute-aidget.exe> <new-absolute-AIDGET_HOME>
 * The data directory must not exist. This script creates it, leaves it for
 * inspection, and terminates only its own MCP child. Exit 1 means verification
 * failed; tool discovery alone does not establish that tool calls work.
 * Once bridge calls work, aidget may automatically start a detached UI. This
 * script neither starts a UI explicitly nor manages that detached process.
 */
import { spawn } from "node:child_process";
import { mkdirSync, statSync } from "node:fs";
import { basename, isAbsolute } from "node:path";
import { createInterface } from "node:readline";

const [executable, home, ...extra] = process.argv.slice(2);
if (process.platform !== "win32") throw new Error("Run this verification on Windows");
if (!executable || !home || extra.length || !isAbsolute(executable) || !isAbsolute(home)) {
  throw new Error("Usage: node windows-mcp.mjs <absolute-aidget.exe> <new-absolute-AIDGET_HOME>");
}
if (!statSync(executable).isFile()) throw new Error("The executable must be a file");
// Exclusive creation prevents accidental use of a live aidget data directory.
mkdirSync(home);

const timeoutMs = 5000;
const summary = {
  executable: basename(executable),
  args: ["mcp"],
  platform: process.platform,
  architecture: process.arch,
  nodeVersion: process.version,
  freshAidgetHome: true,
  explicitUiLaunch: false,
  mayAutoStartUi: true,
  timeoutMsPerRequest: timeoutMs,
  steps: [],
};
const child = spawn(executable, ["mcp"], {
  env: { ...process.env, AIDGET_HOME: home, RUST_BACKTRACE: "0" },
  windowsHide: true,
  shell: false,
  stdio: ["pipe", "pipe", "pipe"],
});
const closed = new Promise((resolve) => { child.once("close", resolve); });
let stderr = "";
let nextId = 0;
let pending;
let terminalError;

/** Reject the in-flight request immediately on transport or protocol failure. */
function fail(error) {
  terminalError = error;
  if (pending) {
    clearTimeout(pending.timer);
    pending.reject(error);
    pending = undefined;
  }
}

const lines = createInterface({ input: child.stdout, crlfDelay: Infinity });
lines.on("line", (line) => {
  let response;
  try {
    response = JSON.parse(line);
  } catch (error) {
    fail(new Error("MCP stdout contained invalid JSON", { cause: error }));
    return;
  }
  // Server notifications are permitted; only the matching response settles a request.
  if (!pending || response.id !== pending.id) return;
  clearTimeout(pending.timer);
  const operation = pending;
  pending = undefined;
  if (response.error) operation.reject(new Error(JSON.stringify(response.error)));
  else operation.resolve(response.result);
});
child.stderr.setEncoding("utf8");
child.stderr.on("data", (chunk) => { stderr += chunk; });
child.on("error", fail);
child.stdin.on("error", fail);
child.on("exit", (code, signal) => {
  summary.childExit = { code, signal };
  fail(new Error(`MCP child exited: code=${code}, signal=${signal}`));
});

/** Send one JSON-RPC request, enforcing a separate deadline for every operation. */
async function request(method, params) {
  if (terminalError) throw terminalError;
  const id = ++nextId;
  const step = { id, method, ...(params.name ? { tool: params.name } : {}) };
  const started = Date.now();
  summary.steps.push(step);
  try {
    const result = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        pending = undefined;
        reject(new Error(`${method} received no response within ${timeoutMs} ms`));
      }, timeoutMs);
      pending = { id, resolve, reject, timer };
      child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
    });
    if (result?.isError) throw new Error(`MCP tool returned isError: ${JSON.stringify(result.content)}`);
    step.ok = true;
    if (method === "initialize") step.protocolVersion = result.protocolVersion;
    if (method === "tools/list") step.tools = result.tools.map((tool) => tool.name);
    if (method === "tools/call") step.contentTypes = result.content.map((item) => item.type);
    return result;
  } catch (error) {
    step.ok = false;
    step.failure = error.message;
    throw error;
  } finally {
    step.elapsedMs = Date.now() - started;
  }
}

try {
  await request("initialize", {
    protocolVersion: "2025-03-26",
    capabilities: {},
    clientInfo: { name: "aidget-windows-mcp-verification", version: "1.0.0" },
  });
  child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);
  await request("tools/list", {});
  // Positive control: guide is local; list exercises the resident UI bridge.
  await request("tools/call", { name: "aidget_guide", arguments: {} });
  await request("tools/call", { name: "aidget_list", arguments: {} });
} catch (error) {
  summary.failure = error.message;
  process.exitCode = 1;
} finally {
  child.stdin.end();
  if (child.exitCode === null && child.signalCode === null) child.kill();
  let cleanupTimer;
  const childClosed = await Promise.race([
    closed.then(() => true),
    new Promise((resolve) => { cleanupTimer = setTimeout(() => resolve(false), 2000); }),
  ]);
  clearTimeout(cleanupTimer);
  summary.mcpChildClosed = childClosed;
  if (!childClosed) {
    summary.cleanupFailure = `MCP child ${child.pid} did not close within 2000 ms`;
    process.exitCode = 1;
    // Report the unclosed child explicitly; release handles so diagnosis cannot hang.
    child.unref();
    child.stdin.destroy();
    child.stdout.destroy();
    child.stderr.destroy();
  }
  lines.close();
  if (pending) clearTimeout(pending.timer);
  // Preserve the original short panic while removing explicit local path roots.
  for (const [path, label] of [[home, "<AIDGET_HOME>"], [process.env.USERPROFILE, "<USERPROFILE>"]]) {
    if (path) stderr = stderr.replaceAll(path, label);
  }
  summary.stderr = stderr.trim();
  summary.nestedRuntimePanic = stderr.includes("Cannot start a runtime from within a runtime");
  console.log(JSON.stringify(summary, null, 2));
}
