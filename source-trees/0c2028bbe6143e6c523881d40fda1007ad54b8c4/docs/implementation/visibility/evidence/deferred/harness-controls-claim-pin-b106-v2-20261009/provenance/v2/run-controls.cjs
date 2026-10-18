"use strict";
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const {spawn, execFileSync} = require("node:child_process");
const BASE = __dirname;
const MAX_LOG_BYTES = 16 * 1024 * 1024;
const TIMEOUT_MS = 600000;
const TERM_GRACE_MS = 5000;
const POST_KILL_GRACE_MS = 1000;
const RUSTUP = "/Users/donbeave/.cargo/bin/rustup";
const RUSTUP_HOME = "/Users/donbeave/.rustup";
const CARGO = "/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/cargo";
const NEXTTEST = "/Users/donbeave/.local/share/mise/installs/aqua-nextest-rs-nextest-cargo-nextest/0.9.146/cargo-nextest";
const CARGO_HOME = "/private/tmp/termrock-vis04-deferred-controls-b106-cache-repair-20261009-v7.Qs1c13/cargo-home";
const FIXED_PATH = [path.dirname(NEXTTEST), "/Users/donbeave/.cargo/bin", "/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin", "/usr/local/bin", "/opt/homebrew/bin", "/usr/bin", "/bin"].join(":");
const sha = bytes => crypto.createHash("sha256").update(bytes).digest("hex");
const readJson = file => JSON.parse(fs.readFileSync(file, "utf8"));
function verifyCargoCache(cache, plan) {
  if (cache.schema !== "termrock-vis04-locked-cargo-cache-repair/v1" || cache.state !== "COMPLETE_VERIFIED_ARCHIVE_AND_INDEX_CLOSURE" || cache.destination_cargo_home !== CARGO_HOME || cache.locked_registry_packages !== 32 || cache.packages.length !== 32) throw new Error("locked Cargo cache manifest drift");
  if (sha(fs.readFileSync(path.join(plan.external_source_root,"crates/termrock-visibility-tests/Cargo.lock"))) !== cache.cargo_lock_sha256) throw new Error("Cargo cache lockfile binding mismatch");
  const verified=[];
  for (const pkg of cache.packages) { if(pkg.archive_sha256!==pkg.checksum) throw new Error("archive checksum manifest mismatch: "+pkg.name); for (const [kind,rel,expected] of [["archive",pkg.archive_path,pkg.archive_sha256],["index",pkg.index_path,pkg.index_sha256]]) { const full=path.resolve(CARGO_HOME,rel); if(!full.startsWith(CARGO_HOME+path.sep)) throw new Error("cache path escapes CARGO_HOME: "+rel); const st=fs.lstatSync(full),bytes=fs.readFileSync(full); if(!st.isFile()||st.isSymbolicLink()||sha(bytes)!==expected) throw new Error("Cargo cache "+kind+" digest mismatch: "+rel); verified.push({kind,path:rel,sha256:expected}); }}
  if(verified.length!==64) throw new Error("expected 64 archive/index cache files, observed "+verified.length);
  const c=cache.index_config,cp=path.resolve(CARGO_HOME,c.path); if(!cp.startsWith(CARGO_HOME+path.sep)) throw new Error("index config escapes CARGO_HOME"); const st=fs.lstatSync(cp),b=fs.readFileSync(cp); if(!st.isFile()||st.isSymbolicLink()||sha(b)!==c.sha256) throw new Error("sparse index config digest mismatch");
  verified.sort((a,b)=>a.path.localeCompare(b.path)); return {locked_packages:32,archive_files:32,sparse_index_files:32,config_sha256:c.sha256,closure_sha256:sha(Buffer.from(JSON.stringify(verified)))};
}

const iso = date => date.toISOString();
function writeAll(fd, bytes) {
  let offset = 0;
  while (offset < bytes.length) {
    const count = fs.writeSync(fd, bytes, offset, bytes.length - offset);
    if (count <= 0) throw new Error("zero-byte log write");
    offset += count;
  }
}
function walkFiles(root, rel = "") {
  const output = [];
  for (const name of fs.readdirSync(root).sort()) {
    const full = path.join(root, name);
    const next = path.posix.join(rel, name);
    const st = fs.lstatSync(full);
    if (st.isDirectory()) output.push(...walkFiles(full, next));
    else if (st.isFile() && !st.isSymbolicLink()) output.push(next);
    else throw new Error("unexpected source entry: " + next);
  }
  return output;
}
function verifySource(manifest) {
  const actual = walkFiles(manifest.external_source_root);
  const expected = manifest.files.map(file => file.path).sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error("external source inventory drift");
  for (const file of manifest.files) {
    const full = path.join(manifest.external_source_root, file.path);
    const st = fs.lstatSync(full);
    const bytes = fs.readFileSync(full);
    if (!st.isFile() || st.isSymbolicLink() || bytes.length !== file.bytes || sha(bytes) !== file.sha256) {
      throw new Error("external source digest drift: " + file.path);
    }
  }
}
function parseTestNames(source) {
  const lines = source.split(/\r?\n/);
  const names = [];
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].trim() !== "#[test]") continue;
    let j = i + 1;
    while (j < lines.length && (lines[j].trim() === "" || lines[j].trim().startsWith("#["))) j++;
    const match = (lines[j] || "").trim().match(/^fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/);
    if (!match) throw new Error("test attribute has no function at line " + (i + 1));
    names.push(match[1]);
  }
  return names;
}
function makeArgs(plan, names) {
  const filterset = names.map(name => "test(=" + name + ")").join(" | ");
  if (names.length !== 29 || plan.selected_count !== 29 || filterset !== plan.filterset) throw new Error("exact 29-test selection drift");
  return [
    "run", "1.98.1", "cargo", "nextest", "run",
    "--manifest-path", path.join(plan.external_source_root, "crates/termrock-visibility-tests/Cargo.toml"),
    "--locked", "--offline", "--package", "termrock-visibility-tests", "--test", "deferred",
    "--build-jobs", "2", "--test-threads", "1", "--run-ignored", "all",
    "--user-config-file", "none", "--ignore-default-filter", "--no-tests", "fail",
    "--no-fail-fast", "--retries", "0", "--flaky-result", "fail",
    "--failure-output", "immediate-final", "--status-level", "all", "--final-status-level", "all",
    "--no-output-indent", "--show-progress", "none", "--color", "never",
    "--filterset", filterset, "--message-format", "libtest-json-plus", "--message-format-version", "0.1"
  ];
}
function groupSignal(pid, signal, errors) {
  if (!pid) return;
  try { process.kill(-pid, signal); }
  catch (error) {
    if (error.code !== "ESRCH") {
      errors.push({signal, message: String(error)});
      try { process.kill(pid, signal); }
      catch (fallback) { if (fallback.code !== "ESRCH") errors.push({signal, message: String(fallback)}); }
    }
  }
}
function normalizeName(raw, names) {
  if (names.includes(raw)) return raw;
  const matches = names.filter(name => raw.endsWith("$" + name) || raw.endsWith("::" + name));
  if (matches.length === 1) return matches[0];
  throw new Error("unexpected Nextest test event name: " + raw);
}
function parseRunEvents(bytes, names) {
  const statuses = new Map();
  const started = new Set();
  const errors = [];
  const suiteEvents = {started: 0, succeeded: 0, success_after_tests: false};
  let suitePhase = "before";
  const decoded = bytes.toString("utf8");
  if (!Buffer.from(decoded, "utf8").equals(bytes)) errors.push("stdout contains invalid UTF-8");
  for (const [index, line] of decoded.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    let event;
    try { event = JSON.parse(line); }
    catch (_) { errors.push("invalid JSON event line " + (index + 1)); continue; }
    if (!event || typeof event !== "object" || Array.isArray(event)) { errors.push("non-object JSON event at line " + (index + 1)); continue; }
    if (event.type === "suite") {
      const metadata = event.nextest;
      if (!metadata || metadata.crate !== "termrock-visibility-tests" || metadata.test_binary !== "deferred" || metadata.kind !== "test") errors.push("suite metadata does not match the selected deferred target at line " + (index + 1));
      if (event.event === "started") {
        if (suitePhase !== "before") errors.push("duplicate or out-of-order suite start at line " + (index + 1));
        suiteEvents.started++;
        suitePhase = "running";
      } else if (event.event === "ok") {
        if (suitePhase !== "running") errors.push("suite success without one open suite at line " + (index + 1));
        suiteEvents.succeeded++;
        suiteEvents.success_after_tests = statuses.size === names.length && [...statuses.values()].every(result => result === "passed");
        if (!suiteEvents.success_after_tests) errors.push("suite success preceded all selected passing test outcomes at line " + (index + 1));
        suitePhase = "ended";
      } else if (["failed", "ignored", "timeout", "cancelled", "exec-failed"].includes(event.event)) {
        if (suitePhase !== "running") errors.push("suite terminal event without one open suite at line " + (index + 1));
        errors.push("non-success suite event at line " + (index + 1) + ": " + event.event);
        suitePhase = "ended";
      }
      else errors.push("unknown suite event at line " + (index + 1) + ": " + String(event.event));
      continue;
    }
    if (event.type !== "test") { errors.push("unknown Nextest event type at line " + (index + 1)); continue; }
    if (suitePhase !== "running") errors.push("test event outside the single active suite at line " + (index + 1));
    if (typeof event.name !== "string") { errors.push("test event missing name at line " + (index + 1)); continue; }
    let name;
    try { name = normalizeName(event.name, names); }
    catch (error) { errors.push(String(error)); continue; }
    if (event.event === "started") {
      if (started.has(name)) errors.push("duplicate started event for " + name);
      if (statuses.has(name)) errors.push("started event followed terminal event for " + name);
      started.add(name);
      continue;
    }
    const result = {ok: "passed", failed: "failed", ignored: "skipped", timeout: "incomplete", cancelled: "incomplete", "exec-failed": "incomplete"}[event.event];
    if (!result) { errors.push("unknown test event kind at line " + (index + 1) + ": " + String(event.event)); continue; }
    if (!started.has(name)) errors.push("terminal event without started event for " + name);
    if (statuses.has(name)) errors.push("duplicate terminal event for " + name);
    else statuses.set(name, result);
  }
  for (const name of names) {
    if (!started.has(name)) errors.push("missing started event for " + name);
    if (!statuses.has(name)) errors.push("missing terminal outcome for " + name);
  }
  if (statuses.size !== 29) errors.push("expected 29 terminal outcomes, observed " + statuses.size);
  if (suiteEvents.started !== 1) errors.push("expected exactly one suite start, observed " + suiteEvents.started);
  if (suiteEvents.succeeded !== 1) errors.push("expected exactly one successful suite terminal event, observed " + suiteEvents.succeeded);
  if (suitePhase !== "ended" || !suiteEvents.success_after_tests) errors.push("suite did not close successfully after all selected tests passed");
  return {statuses, errors, suiteEvents};
}
function fixedEnv(base) {
  const scratch = path.join(base, "scratch");
  fs.mkdirSync(path.join(scratch, "home"), {recursive: true});
  return {
    PATH: FIXED_PATH, LANG: "C", LC_ALL: "C", TERM: "dumb", HOME: path.join(scratch, "home"),
    CARGO, CARGO_HOME: CARGO_HOME, CARGO_TARGET_DIR: path.join(base, "target"), CARGO_BUILD_JOBS: "2",
    CARGO_TERM_COLOR: "never", CARGO_NET_OFFLINE: "true", TMPDIR: scratch, RUSTUP_HOME,
    RUSTUP_TOOLCHAIN: "1.98.1", NEXTEST_EXPERIMENTAL_LIBTEST_JSON: "1"
  };
}
async function main() {
  const preflightOnly = process.argv.length === 3 && process.argv[2] === "--preflight-only";
  if (process.argv.length !== 2 && !preflightOnly) throw new Error("usage: run-controls.cjs [--preflight-only]");
  const started = new Date();
  const runId = started.toISOString().replace(/[:.]/g, "-") + "-" + crypto.randomUUID().slice(0, 8);
  const runDir = path.join(BASE, "runs", runId);
  fs.mkdirSync(runDir, {recursive: true});
  const stdoutPath = path.join(runDir, "nextest.stdout.jsonl");
  const stderrPath = path.join(runDir, "nextest.stderr.log");
  const receiptPath = path.join(runDir, "receipt.json");
  let stdoutFd = fs.openSync(stdoutPath, "wx"), stderrFd = fs.openSync(stderrPath, "wx");
  const streams = {stdout: {received: 0, stored: 0, truncated: false, write_failed: false}, stderr: {received: 0, stored: 0, truncated: false, write_failed: false}};
  const cleanupErrors = [], logErrors = [];
  let totalStored = 0;
  const receipt = {
    schema: "termrock-vis04-deferred-harness-controls-run-receipt/v1", run_id: runId,
    started_at: iso(started), finished_at: null, source: {},
    selection: {target: "termrock-visibility-tests::deferred", count: 29, names: []},
    toolchain: {rust: "1.98.1", cargo_nextest: "0.9.146", build_jobs: 2, test_threads: 1, offline: true},
    command: null, environment: {inherits_parent: false, cargo_net_offline: true},
    bounds: {timeout_ms: TIMEOUT_MS, term_grace_ms: TERM_GRACE_MS, post_kill_grace_ms: POST_KILL_GRACE_MS, combined_log_cap_bytes: MAX_LOG_BYTES},
    execution: {result: "BLOCKED_PREFLIGHT", elapsed_ms: null, timed_out: false, interrupted_by: null, exit_code: null, signal: null, spawn_error: null, cargo_spawned: false, nextest_run_id: null, test_results: [], parse_errors: [], forced_close: false},
    artifacts: null, cleanup_errors: cleanupErrors, log_errors: logErrors
  };
  function capture(name, chunk) {
    const stream = streams[name];
    stream.received += chunk.length;
    if (stream.write_failed) return;
    const allowed = Math.max(0, MAX_LOG_BYTES - totalStored);
    const kept = chunk.subarray(0, Math.min(chunk.length, allowed));
    if (kept.length) {
      try { writeAll(name === "stdout" ? stdoutFd : stderrFd, kept); stream.stored += kept.length; totalStored += kept.length; }
      catch (error) { stream.write_failed = true; logErrors.push(name + ": " + String(error)); }
    }
    if (kept.length < chunk.length) stream.truncated = true;
  }
  function finalize(exitStatus) {
    const ended = new Date(); receipt.finished_at = iso(ended); receipt.execution.elapsed_ms = ended.getTime() - started.getTime();
    for (const [fd, label] of [[stdoutFd, "stdout"], [stderrFd, "stderr"]]) {
      if (fd !== null) { try { fs.closeSync(fd); } catch (error) { cleanupErrors.push(label + " close: " + String(error)); } }
    }
    stdoutFd = null; stderrFd = null;
    const artifact = (file, stream) => ({path: path.relative(BASE, file), bytes_received: streams[stream].received, bytes_stored: streams[stream].stored, sha256: sha(fs.readFileSync(file)), truncated: streams[stream].truncated, write_failed: streams[stream].write_failed});
    receipt.artifacts = {stdout: artifact(stdoutPath, "stdout"), stderr: artifact(stderrPath, "stderr"), total_bytes_stored: totalStored, total_cap_bytes: MAX_LOG_BYTES};
    const tempPath = receiptPath + ".tmp";
    fs.writeFileSync(tempPath, JSON.stringify(receipt, null, 2) + "\n", {flag: "wx"});
    fs.renameSync(tempPath, receiptPath);
    process.stdout.write(JSON.stringify({receipt: receiptPath, result: receipt.execution.result, exit_code: receipt.execution.exit_code, cargo_spawned: receipt.execution.cargo_spawned}, null, 2) + "\n");
    process.exitCode = exitStatus;
  }
  try {
    const manifestBytes = fs.readFileSync(path.join(BASE, "source-manifest.json"));
    const planBytes = fs.readFileSync(path.join(BASE, "runplan.json"));
    const preflightBytes = fs.readFileSync(path.join(BASE, "preflight.json"));
    const cacheBytes = fs.readFileSync(path.join(BASE, "cargo-cache-manifest.json"));
    const namesBytes = fs.readFileSync(path.join(BASE, "test-names.json"));
    const runnerBytes = fs.readFileSync(__filename);
    const launchBytes = fs.readFileSync(path.join(BASE, "launch.sh"));
    const manifest = JSON.parse(manifestBytes.toString("utf8"));
    const plan = JSON.parse(planBytes.toString("utf8"));
    const preflight = JSON.parse(preflightBytes.toString("utf8"));
    const namesDoc = JSON.parse(namesBytes.toString("utf8"));
    const names = namesDoc.names;
    const cacheManifest = JSON.parse(cacheBytes.toString("utf8"));
    const runnerHash = sha(runnerBytes), launchHash = sha(launchBytes), manifestHash = sha(manifestBytes), planHash = sha(planBytes);
    if (preflight.result !== "PASS_STATIC_PREFLIGHT_NO_CARGO") throw new Error("static preflight has not passed");
    if (preflight.source_manifest_sha256 !== manifestHash || preflight.runplan_sha256 !== planHash || preflight.runner_sha256 !== runnerHash || preflight.launch_script_sha256 !== launchHash) throw new Error("preflight digest bindings mismatch");
    if (plan.source_manifest_sha256 !== manifestHash || plan.runner_sha256 !== runnerHash || plan.launch_script_sha256 !== launchHash || plan.test_names_sha256 !== sha(namesBytes) || plan.cargo_cache_manifest_sha256 !== sha(cacheBytes)) throw new Error("runplan digest bindings mismatch");
    if (namesDoc.schema !== "termrock-vis04-deferred-controls-test-names/v1" || namesDoc.target !== "termrock-visibility-tests::deferred" || namesDoc.count !== 29 || names.length !== 29) throw new Error("test-name document schema mismatch");
    if (manifest.candidate_product_commit !== "1d797d41c8141fcbdc3f69d7f11eb8875ab54712" || manifest.fixture_base_commit !== "b106b2426cfce016381c58b3d380e762a454c7a6") throw new Error("source revision pin mismatch");
    if (manifest.files.length !== 14 || manifest.harness_controls.count !== 29 || names.length !== 29 || JSON.stringify(names) !== JSON.stringify(manifest.harness_controls.names)) throw new Error("29-test selection metadata drift");
    if (JSON.stringify(parseTestNames(fs.readFileSync(path.join(manifest.external_source_root, "crates/termrock-visibility-tests/tests/deferred.rs"), "utf8"))) !== JSON.stringify(names)) throw new Error("deferred.rs source order differs from frozen tests");
    verifySource(manifest);
    const cacheBefore = verifyCargoCache(cacheManifest, plan);
    receipt.cargo_cache = {manifest_sha256: sha(cacheBytes), lock_sha256: cacheManifest.cargo_lock_sha256, locked_packages: cacheBefore.locked_packages, archive_files: cacheBefore.archive_files, sparse_index_files: cacheBefore.sparse_index_files, config_sha256: cacheBefore.config_sha256, before_closure_sha256: cacheBefore.closure_sha256, after_closure_sha256: null, unchanged: null};
    if (plan.schema !== "termrock-vis04-deferred-harness-controls-runplan/v4" || plan.actual_receipt_schema !== "termrock-vis04-deferred-harness-controls-run-receipt/v1" || plan.mode !== "prepared_not_launched" || plan.selected_count !== 29 || plan.build_jobs !== 2 || plan.test_threads !== 1 || plan.offline !== true || plan.timeout_ms !== TIMEOUT_MS || plan.term_grace_ms !== TERM_GRACE_MS || plan.post_kill_grace_ms !== POST_KILL_GRACE_MS || plan.combined_log_cap_bytes !== MAX_LOG_BYTES || plan.environment_inherits_parent !== false) throw new Error("runplan safety bounds mismatch");
    const args = makeArgs(plan, names);
    if (JSON.stringify(plan.argv) !== JSON.stringify(args)) throw new Error("argv differs from frozen runplan");
    const nextestVersion = execFileSync(NEXTTEST, ["--version"], {encoding: "utf8", timeout: 5000, maxBuffer: 4096, env: {PATH: FIXED_PATH, LANG: "C", LC_ALL: "C", TERM: "dumb", CARGO_HOME: CARGO_HOME, RUSTUP_HOME}}).trim().split("\n", 1)[0];
    if (nextestVersion !== plan.nextest_version) throw new Error("pinned Nextest version drift");
    const sourceRoot = manifest.external_source_root;
    const env = fixedEnv(BASE);
    receipt.source = {candidate_product_commit: manifest.candidate_product_commit, fixture_base_commit: manifest.fixture_base_commit, source_manifest_sha256: manifestHash, runplan_sha256: planHash, runner_sha256: runnerHash, launch_script_sha256: launchHash, preflight_sha256: sha(preflightBytes), test_names_sha256: sha(namesBytes)};
    receipt.selection.names = names;
    receipt.command = {program: RUSTUP, argv: args, cwd: sourceRoot};
    receipt.environment = {inherits_parent: false, CARGO, CARGO_HOME, CARGO_TARGET_DIR: env.CARGO_TARGET_DIR, CARGO_BUILD_JOBS: "2", CARGO_TERM_COLOR: "never", CARGO_NET_OFFLINE: "true", TMPDIR: env.TMPDIR, RUSTUP_HOME, RUSTUP_TOOLCHAIN: "1.98.1", PATH: FIXED_PATH, HOME: env.HOME, NEXTEST_EXPERIMENTAL_LIBTEST_JSON: "1"};
    if (preflightOnly) {
      const cacheAfter = verifyCargoCache(cacheManifest, plan);
      receipt.cargo_cache.after_closure_sha256 = cacheAfter.closure_sha256;
      receipt.cargo_cache.unchanged = cacheAfter.closure_sha256 === cacheBefore.closure_sha256;
      if (!receipt.cargo_cache.unchanged) throw new Error("Cargo cache closure changed during runtime preflight");
      receipt.execution.result = "PASS_RUNTIME_PREFLIGHT_NO_CARGO";
      receipt.execution.test_result_interpretation = "Runner validated source, exact 29-name selection, command, tools, environment, and the 64-file offline Cargo cache closure and queried the pinned Nextest version. Static preflight separately checked Nextest help. Only external packet artifacts were written; no Cargo command, build, or Nextest test ran, and no source or repository file changed.";
      finalize(0); return;
    }
    const child = spawn(RUSTUP, args, {cwd: sourceRoot, env, detached: true, stdio: ["ignore", "pipe", "pipe"]});
    receipt.execution.cargo_spawned = true;
    let timedOut = false, interruptedBy = null, spawnError = null, settled = false, terminationStarted = false;
    let resolveStatus, resolveTermination;
    const statusPromise = new Promise(resolve => { resolveStatus = status => { if (!settled) { settled = true; resolve(status); } }; });
    const terminationPromise = new Promise(resolve => { resolveTermination = resolve; });
    let timeoutTimer, termTimer, postKillTimer;
    child.stdout.on("data", chunk => capture("stdout", chunk));
    child.stderr.on("data", chunk => capture("stderr", chunk));
    child.on("error", error => { spawnError = String(error); });
    child.on("close", (code, signal) => resolveStatus({code, signal, forcedClose: false}));
    const terminateGroup = () => {
      if (terminationStarted) return;
      terminationStarted = true;
      groupSignal(child.pid, "SIGTERM", cleanupErrors);
      termTimer = setTimeout(() => {
        groupSignal(child.pid, "SIGKILL", cleanupErrors);
        postKillTimer = setTimeout(() => {
          if (!settled) {
            try { child.stdout.destroy(); } catch (error) { cleanupErrors.push("stdout destroy: " + String(error)); }
            try { child.stderr.destroy(); } catch (error) { cleanupErrors.push("stderr destroy: " + String(error)); }
            try { child.unref(); } catch (error) { cleanupErrors.push("child unref: " + String(error)); }
            resolveStatus({code: null, signal: "SIGKILL", forcedClose: true});
          }
          resolveTermination();
        }, POST_KILL_GRACE_MS);
      }, TERM_GRACE_MS);
    };
    const signalHandler = signal => { if (interruptedBy) return; interruptedBy = signal; terminateGroup(); };
    process.on("SIGINT", () => signalHandler("SIGINT"));
    process.on("SIGTERM", () => signalHandler("SIGTERM"));
    timeoutTimer = setTimeout(() => { timedOut = true; terminateGroup(); }, TIMEOUT_MS);
    const status = await statusPromise;
    clearTimeout(timeoutTimer);
    if (timedOut || interruptedBy) { terminateGroup(); await terminationPromise; }
    else { if (termTimer) clearTimeout(termTimer); if (postKillTimer) clearTimeout(postKillTimer); }
    receipt.execution.timed_out = timedOut;
    receipt.execution.interrupted_by = interruptedBy;
    receipt.execution.exit_code = status.code;
    receipt.execution.signal = status.signal;
    receipt.execution.spawn_error = spawnError;
    receipt.execution.forced_close = status.forcedClose;
    verifySource(manifest);
    try { const cacheAfter = verifyCargoCache(cacheManifest, plan); receipt.cargo_cache.after_closure_sha256 = cacheAfter.closure_sha256; receipt.cargo_cache.unchanged = cacheAfter.closure_sha256 === cacheBefore.closure_sha256; if(!receipt.cargo_cache.unchanged) receipt.execution.parse_errors.push("64-file Cargo archive/index closure changed during execution"); } catch(error) { receipt.cargo_cache.error=String(error); receipt.execution.parse_errors.push("post-run Cargo cache closure verification failed: "+String(error)); }
    const runBytes = fs.readFileSync(stdoutPath);
    const stderrText = fs.readFileSync(stderrPath, "utf8");
    const runIds = [...stderrText.matchAll(/Nextest run ID ([0-9a-f-]{36}) with nextest profile/g)].map(match => match[1]);
    if (runIds.length === 1) receipt.execution.nextest_run_id = runIds[0];
    else receipt.execution.parse_errors.push("expected exactly one Nextest run ID, found " + runIds.length);
    const parsed = parseRunEvents(runBytes, names);
    receipt.execution.parse_errors.push(...parsed.errors);
    receipt.execution.test_results = names.map(name => ({name, result: parsed.statuses.get(name) || "not_run"}));
    const results = [...parsed.statuses.values()];
    const hasFailure = results.includes("failed");
    const completePass = results.length === 29 && results.every(result => result === "passed") && parsed.errors.length === 0 && parsed.suiteEvents.started === 1 && parsed.suiteEvents.succeeded === 1 && parsed.suiteEvents.success_after_tests && runIds.length === 1 && status.code === 0 && receipt.cargo_cache.unchanged === true;
    if (timedOut) receipt.execution.result = "BLOCKED_TIMEOUT";
    else if (interruptedBy || status.signal) receipt.execution.result = "BLOCKED_SIGNAL";
    else if (spawnError) receipt.execution.result = "BLOCKED_SPAWN";
    else if (status.forcedClose) receipt.execution.result = "BLOCKED_FORCED_CLOSE";
    else if (streams.stdout.truncated || streams.stderr.truncated || logErrors.length) receipt.execution.result = "BLOCKED_CAPTURE_LIMIT_OR_IO";
    else if (hasFailure) receipt.execution.result = "FAILED";
    else if (receipt.execution.parse_errors.length || !completePass || runIds.length !== 1) receipt.execution.result = "BLOCKED_INCOMPLETE_OR_PARSE";
    else if (status.code === 0) receipt.execution.result = "PASSED_29";
    else receipt.execution.result = "BLOCKED_NONZERO_WITHOUT_TEST_FAILURE";
    if (timedOut) finalize(124);
    else if (interruptedBy === "SIGINT") finalize(130);
    else if (interruptedBy === "SIGTERM") finalize(143);
    else if (receipt.execution.result === "PASSED_29") finalize(0);
    else if (receipt.execution.result === "FAILED") finalize(1);
    else finalize(2);
  } catch (error) {
    receipt.execution.result = "BLOCKED_PREFLIGHT_OR_LAUNCH";
    receipt.execution.spawn_error = String(error);
    finalize(2);
  }
}
main().catch(error => { process.stderr.write(String(error) + "\n"); process.exitCode = 2; });
