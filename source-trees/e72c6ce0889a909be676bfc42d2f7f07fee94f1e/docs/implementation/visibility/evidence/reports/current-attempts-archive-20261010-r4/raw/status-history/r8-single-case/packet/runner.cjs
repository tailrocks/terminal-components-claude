#!/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node
"use strict";
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const {spawn} = require("node:child_process");

const PACKET = __dirname;
const PLAN_PATH = path.join(PACKET, "runplan.json");
const SOURCE_PATH = path.join(PACKET, "source-manifest.json");
const TESTS_PATH = path.join(PACKET, "test-names.json");
const REQUEST_PATH = path.join(PACKET, "review-request.json");
const PACKET_CHECKSUMS_PATH = path.join(PACKET, "SHA256SUMS");
const PREFLIGHT_DIR = path.join(PACKET, "preflight");
const PREFLIGHT_PATH = path.join(PREFLIGHT_DIR, "runner-preflight.json");
const sha = value => crypto.createHash("sha256").update(value).digest("hex");
const shaFile = file => sha(fs.readFileSync(file));
const readJson = file => JSON.parse(fs.readFileSync(file, "utf8"));
const stable = value => JSON.stringify(value);
function fail(message) { throw new Error(message); }
function isAbsent(file) {
  try { fs.lstatSync(file); return false; }
  catch (error) { if (error.code === "ENOENT") return true; throw error; }
}
function assertRegular(file, expected, label) {
  const stat = fs.lstatSync(file);
  if (!stat.isFile() || stat.isSymbolicLink()) fail(label + " is not a regular file");
  const actual = shaFile(file);
  if (actual !== expected) fail(label + " hash mismatch: " + actual);
  return {path: file, sha256: actual, bytes: stat.size, mode: stat.mode & 0o7777};
}
function writeJson(file, value) {
  const temp = file + ".tmp";
  fs.writeFileSync(temp, JSON.stringify(value, null, 2) + "\n", {flag: "wx", mode: 0o600});
  fs.renameSync(temp, file);
  fs.chmodSync(file, 0o600);
}
function ensurePrivateDirectory(directory, create) {
  if (isAbsent(directory)) {
    if (!create) fail("required private directory is absent: " + directory);
    fs.mkdirSync(directory, {mode: 0o700});
    fs.chmodSync(directory, 0o700);
  }
  const stat = fs.lstatSync(directory);
  if (!stat.isDirectory() || stat.isSymbolicLink() || (stat.mode & 0o077) !== 0) fail("directory is not a private real directory: " + directory);
}
function parseTestNames(source) {
  const lines = source.split(/\r?\n/);
  const names = [];
  for (let index = 0; index < lines.length; index++) {
    if (!/^#\[(?:tokio::)?test(?:\([^]]*\))?\]$/.test(lines[index].trim())) continue;
    let next = index + 1;
    while (next < lines.length && (lines[next].trim() === "" || lines[next].trim().startsWith("#["))) next++;
    const match = (lines[next] || "").trim().match(/^(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/);
    if (!match) fail("cannot map test attribute at source line " + (index + 1));
    names.push(match[1]);
  }
  return names;
}
function verifyInputs() {
  if (process.platform !== "darwin") fail("pinned supervisor requires macOS");
  const plan = readJson(PLAN_PATH);
  const source = readJson(SOURCE_PATH);
  const tests = readJson(TESTS_PATH);
  const request = readJson(REQUEST_PATH);
  const cachePath = plan.bindings.cache_closure_path;
  const cache = readJson(cachePath);
  if (plan.schema !== "termrock.vis01.status-attempt-history-nextest-runplan.r2" || tests.schema !== "termrock.nextest-target-inventory.r3" || request.schema !== "termrock.nextest.runner-review-request.r2") fail("packet schema version mismatch");
  if (plan.status !== "PREPARED_FOR_ROOT_ONLY_EXECUTION; NOT_RUN" || plan.owner !== "Root only") fail("runplan is not the prepared Root-owned input");
  if (plan.bindings.generic_runner_sha256 !== shaFile(__filename)) fail("runplan does not pin this adapted runner");
  if (plan.runner_path !== __filename || request.packet !== PACKET || request.source_root !== plan.cwd || request.pins.runner_path !== __filename || request.pins.runplan_path !== PLAN_PATH || request.pins.source_manifest_path !== SOURCE_PATH || request.pins.test_names_path !== TESTS_PATH || request.pins.cache_closure_path !== cachePath || request.pins.packet_checksums_path !== PACKET_CHECKSUMS_PATH) fail("review request/runplan paths do not bind this packet");
  const packageRelative = path.relative(plan.cwd, plan.package_manifest);
  if (plan.cwd !== source.source_root || packageRelative.startsWith("..") || path.isAbsolute(packageRelative) || !source.build_input_files.some(file => file.path === packageRelative)) fail("workspace/package path binding mismatch");
  if (plan.program !== source.toolchain.cargo.path || shaFile(plan.program) !== source.toolchain.cargo.sha256 || plan.program_sha256 !== source.toolchain.cargo.sha256) fail("Cargo tool pin mismatch");
  if (shaFile(source.toolchain.rustc.path) !== source.toolchain.rustc.sha256 || shaFile(source.toolchain.nextest.path) !== source.toolchain.nextest.sha256) fail("Rust/Nextest tool pin mismatch");
  if (process.execPath !== source.toolchain.node.path || process.version !== source.toolchain.node.version || shaFile(process.execPath) !== source.toolchain.node.sha256) fail("Node runtime pin mismatch");
  if (shaFile(SOURCE_PATH) !== plan.bindings.source_manifest_sha256) fail("source manifest hash differs from runplan");
  if (shaFile(TESTS_PATH) !== plan.bindings.test_names_sha256) fail("test inventory hash differs from runplan");
  if (shaFile(cachePath) !== plan.bindings.cache_closure_sha256 || cachePath !== source.cache.closure_path) fail("cache closure hash/path mismatch");
  if (request.status !== "REQUEST_STATIC_RUNNER_REVIEW; NO_EXECUTION") fail("review request status is not frozen for static review");
  if (request.pins.runplan_sha256 !== shaFile(PLAN_PATH) || request.pins.source_manifest_sha256 !== shaFile(SOURCE_PATH) || request.pins.test_names_sha256 !== shaFile(TESTS_PATH) || request.pins.cache_closure_sha256 !== shaFile(cachePath) || request.pins.runner_sha256 !== shaFile(__filename) || request.pins.packet_checksums_sha256 !== shaFile(PACKET_CHECKSUMS_PATH)) fail("review request does not bind these runner inputs");
  if (plan.bindings.base_publication_commit && source.base_publication && plan.bindings.base_publication_commit !== source.base_publication.commit) fail("published source base binding mismatch");
  const lockEntry = source.build_input_files.find(file => file.path === "crates/termrock-visibility-tests/Cargo.lock");
  if (!lockEntry || cache.package_lock_path !== lockEntry.path || cache.cargo_home !== plan.environment.CARGO_HOME || cache.full_workspace_lock_sha256 !== lockEntry.sha256 || cache.package_lock_sha256 !== lockEntry.sha256) fail("verified offline cache closure does not match the pinned standalone package lock");
  if (cache.publisher_registry_archive_count !== undefined && cache.publisher_registry_archive_count !== cache.archives.length) fail("cache archive count metadata mismatch");
  if (cache.publisher_sparse_index_file_count !== undefined && cache.publisher_sparse_index_file_count !== cache.indexes.length) fail("cache sparse-index count metadata mismatch");

  const expectedEnv = ["CARGO_BUILD_JOBS", "CARGO_HOME", "CARGO_NET_OFFLINE", "CARGO_TARGET_DIR", "CARGO_TERM_COLOR", "HOME", "LC_ALL", "NEXTEST_EXPERIMENTAL_LIBTEST_JSON", "PATH", "RUSTC", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "TMPDIR"].sort();
  if (stable(Object.keys(plan.environment).sort()) !== stable(expectedEnv)) fail("child environment is not the exact reviewed allowlist");
  if (plan.environment.CARGO_NET_OFFLINE !== "true" || plan.environment.CARGO_BUILD_JOBS !== "2" || plan.environment.CARGO_TARGET_DIR !== source.warm_target.target_dir || plan.environment.CARGO_TERM_COLOR !== "never") fail("offline/jobs/target controls differ");
  if (plan.environment.PATH.split(path.delimiter)[0] !== path.dirname(source.toolchain.nextest.path)) fail("Nextest PATH pin mismatch");
  if (plan.environment.HOME !== path.join(plan.limits.fresh_output_root, "home") || plan.environment.TMPDIR !== path.join(plan.limits.fresh_output_root, "scratch")) fail("HOME/TMPDIR must be private run-local paths");
  if (plan.limits.parent_environment_inherited !== false || plan.limits.locked !== true || plan.limits.offline !== true || plan.limits.build_jobs !== 2 || plan.limits.test_threads !== 1 || plan.limits.retries !== 0 || plan.limits.timeout_ms !== 600000 || plan.limits.term_grace_ms !== 5000 || plan.limits.post_kill_grace_ms !== 1000 || plan.limits.combined_stdout_stderr_cap_bytes !== 16777216) fail("execution limits differ from the reviewed bounds");

  const packageName = tests.package_name;
  const binaryTarget = tests.targets.binary;
  const integrationTarget = tests.targets.integration;
  const argv = plan.argv;
  const filterSet = tests.test_names.map(name => "test(=" + name + ")").join(" | ");
  if (!filterSet || tests.test_names.length !== 1) fail("focused retry must select exactly one inventoried source test");
  const expectedArgv = [
    "nextest", "run", "--manifest-path", plan.package_manifest,
    "--locked", "--offline", "--package", packageName,
    "--test", integrationTarget.name,
    "--build-jobs", String(plan.limits.build_jobs),
    "--test-threads", String(plan.limits.test_threads),
    "--user-config-file", "none", "--ignore-default-filter",
    "--no-tests", "fail", "--no-fail-fast", "--retries", String(plan.limits.retries),
    "--flaky-result", "fail", "--failure-output", "immediate-final",
    "--status-level", "all", "--final-status-level", "all",
    "--no-output-indent", "--show-progress", "none", "--color", "never",
    "--filterset", filterSet,
    "--message-format", "libtest-json-plus", "--message-format-version", "0.1"
  ];
  if (stable(argv) !== stable(expectedArgv)) fail("Nextest argv differs from the exact reviewed target and execution contract");

  if (!Array.isArray(source.build_input_files) || !source.build_input_files.some(file => file.path === binaryTarget.source_path) || !source.build_input_files.some(file => file.path === integrationTarget.source_path)) fail("build source paths are not pinned by the source manifest");
  if (binaryTarget.selection_role !== "build_dependency_only" || binaryTarget.selected_unit_test_count !== 0 || binaryTarget.suite_expected !== false || !Array.isArray(binaryTarget.source_unit_test_names) || new Set(binaryTarget.source_unit_test_names).size !== binaryTarget.source_unit_test_names.length) fail("fixture binary must be recorded as an unselected build dependency with its source inventory");
  const expectedBinaryDepInfoPaths = [
    "debug/" + binaryTarget.name + ".d",
    "debug/deps/" + binaryTarget.name + "-<16-lowerhex>.d"
  ];
  if (binaryTarget.artifact_required !== true || stable(binaryTarget.dep_info_paths) !== stable(expectedBinaryDepInfoPaths) || binaryTarget.dep_info_path !== undefined) fail("fixture build-dependency artifact contract is incomplete");
  if (integrationTarget.artifact_path_pattern !== "debug/deps/" + integrationTarget.name + "-<16-lowerhex>" || integrationTarget.dep_info_path_pattern !== "debug/deps/" + integrationTarget.name + "-<16-lowerhex>.d") fail("integration target artifact patterns are not exact Cargo hash paths");
  const binaryTests = parseTestNames(fs.readFileSync(path.join(plan.cwd, binaryTarget.source_path), "utf8"));
  const integrationTests = parseTestNames(fs.readFileSync(path.join(plan.cwd, integrationTarget.source_path), "utf8"));
  if (!Array.isArray(integrationTarget.source_test_names) || integrationTarget.source_test_count !== integrationTarget.source_test_names.length || new Set(integrationTarget.source_test_names).size !== integrationTarget.source_test_names.length) fail("full source test inventory is missing or inconsistent");
  if (binaryTests.length !== binaryTarget.expected_unit_test_count || stable(binaryTests.slice().sort()) !== stable(binaryTarget.source_unit_test_names.slice().sort()) || integrationTests.length !== integrationTarget.source_test_count || stable(integrationTests.slice().sort()) !== stable(integrationTarget.source_test_names.slice().sort())) fail("source #[test] declarations differ from the complete source inventory");
  if (!Array.isArray(tests.test_names) || tests.test_names.some(name => !integrationTarget.source_test_names.includes(name))) fail("selected test is not declared in the pinned source");
  if (integrationTarget.expected_test_count !== tests.test_names.length || new Set(tests.test_names).size !== tests.test_names.length || plan.expected.selected_test_count !== tests.test_names.length || plan.expected.selected_target !== integrationTarget.name) fail("selected test names/count are inconsistent");

  const output = plan.limits.fresh_output_root;
  if (!isAbsent(output)) fail("fresh output root already exists");
  ensurePrivateDirectory(PREFLIGHT_DIR, false);
  const packetPins = {runplan_sha256: shaFile(PLAN_PATH), source_manifest_sha256: shaFile(SOURCE_PATH), test_names_sha256: shaFile(TESTS_PATH), cache_closure_sha256: shaFile(cachePath), runner_sha256: shaFile(__filename), packet_checksums_sha256: shaFile(PACKET_CHECKSUMS_PATH), review_request_sha256: shaFile(REQUEST_PATH)};
  return {plan, source, tests, cache, output, packetPins};
}
function sourceAudit(input) {
  const entries = [];
  for (const file of input.source.build_input_files) {
    const record = assertRegular(path.join(input.plan.cwd, file.path), file.sha256, "source " + file.path);
    entries.push({path: file.path, sha256: record.sha256, bytes: record.bytes, mode: record.mode});
  }
  for (const proof of input.source.audit_proofs || []) {
    const record = assertRegular(proof.path, proof.sha256, proof.label || "pinned audit proof");
    entries.push({path: proof.path, sha256: record.sha256, bytes: record.bytes, mode: record.mode});
  }
  entries.sort((a, b) => a.path.localeCompare(b.path));
  const result = {schema: "termrock.nextest.source-audit.v1", source_root: input.plan.cwd, base_commit: input.source.base_publication && input.source.base_publication.commit || null, files: entries};
  result.digest = sha(Buffer.from(stable(result)));
  return result;
}
function toolAudit(input) {
  const tools = [
    ["cargo", input.source.toolchain.cargo],
    ["rustc", input.source.toolchain.rustc],
    ["nextest", input.source.toolchain.nextest],
    ["node", input.source.toolchain.node]
  ].map(([name, tool]) => ({name, ...assertRegular(tool.path, tool.sha256, name)})).sort((a, b) => a.name.localeCompare(b.name));
  const result = {schema: "termrock.nextest.tool-audit.v1", tools};
  result.digest = sha(Buffer.from(stable(result)));
  return result;
}
function cacheAudit(input) {
  const entries = [];
  for (const archive of input.cache.archives) entries.push(assertRegular(archive.archive.path, archive.checksum, "registry archive " + archive.name + " " + archive.version));
  for (const index of input.cache.indexes) entries.push(assertRegular(index.path, index.sha256, "sparse index " + index.name));
  entries.sort((a, b) => a.path.localeCompare(b.path));
  const result = {
    schema: "termrock.nextest.cache-audit.v1",
    cargo_home: input.cache.cargo_home,
    lock_sha256: input.cache.full_workspace_lock_sha256,
    registry_archives: input.cache.publisher_registry_archive_count,
    sparse_indexes: input.cache.publisher_sparse_index_file_count,
    resolved_packages: input.cache.publisher_resolved_package_count === undefined ? null : input.cache.publisher_resolved_package_count,
    entries
  };
  result.digest = sha(Buffer.from(stable(result)));
  return result;
}
function snapshotTree(root) {
  const rootStat = fs.lstatSync(root);
  if (!rootStat.isDirectory() || rootStat.isSymbolicLink()) fail("target root is not a real directory");
  const entries = [];
  let bytesTotal = 0;
  function visit(abs, rel) {
    const before = fs.lstatSync(abs), mode = before.mode & 0o7777;
    if (before.isSymbolicLink()) { entries.push({path: rel, kind: "symlink", mode, target: fs.readlinkSync(abs)}); return; }
    if (before.isDirectory()) {
      entries.push({path: rel, kind: "directory", mode});
      for (const name of fs.readdirSync(abs).sort()) visit(path.join(abs, name), rel ? rel + "/" + name : name);
      return;
    }
    if (before.isFile()) {
      const data = fs.readFileSync(abs), after = fs.lstatSync(abs);
      if (!after.isFile() || after.isSymbolicLink() || after.dev !== before.dev || after.ino !== before.ino || after.size !== before.size || after.mtimeMs !== before.mtimeMs || data.length !== before.size) fail("target changed while snapshotting: " + rel);
      bytesTotal += data.length;
      entries.push({path: rel, kind: "file", mode, bytes: data.length, sha256: sha(data)});
      return;
    }
    fail("unsupported target entry: " + rel);
  }
  visit(root, "");
  entries.sort((a, b) => a.path.localeCompare(b.path));
  const summary = {
    schema: "termrock.nextest.target-snapshot.v1",
    target_root: root,
    root_mode: rootStat.mode & 0o7777,
    entry_count: entries.length,
    regular_file_count: entries.filter(entry => entry.kind === "file").length,
    directory_count: entries.filter(entry => entry.kind === "directory").length,
    symlink_count: entries.filter(entry => entry.kind === "symlink").length,
    regular_file_bytes: bytesTotal,
    entries_sha256: sha(Buffer.from(stable(entries))),
    entries
  };
  summary.digest = sha(Buffer.from(stable(summary)));
  return summary;
}
function targetArtifactCandidates(snapshot, tests) {
  const bin = tests.targets.binary.name;
  const binStem = bin.replace(/-/g, "_");
  const integration = tests.targets.integration.name;
  return snapshot.entries.filter(entry => {
    const base = path.posix.basename(entry.path);
    return entry.path === "debug/" + bin || entry.path === "debug/" + bin + ".d" ||
      (entry.path.startsWith("debug/deps/") && (base.startsWith(binStem + "-") || base.startsWith(integration + "-"))) ||
      (entry.path.startsWith("debug/.fingerprint/") && (base === "bin-" + bin || base.startsWith("test-integration-test-" + integration)));
  });
}
function cargoHashedArtifactPath(name, entry, extension) {
  const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp("^debug/deps/" + escapedName + "-[0-9a-f]{16}" + extension + "$").test(entry.path);
}
function verifyTargetArtifacts(before, after, input) {
  const tests = input.tests, target = input.plan.environment.CARGO_TARGET_DIR;
  const beforePaths = new Set(before.entries.filter(entry => entry.kind === "file").map(entry => entry.path));
  const changed = after.entries.filter(entry => entry.kind === "file" && !beforePaths.has(entry.path));
  const integrationExes = after.entries.filter(entry => entry.kind === "file" && cargoHashedArtifactPath(tests.targets.integration.name, entry, ""));
  const integrationExe = integrationExes.length === 1 ? integrationExes[0] : null;
  const binaryExes = after.entries.filter(entry => entry.kind === "file" && entry.path === tests.targets.binary.artifact_path);
  const binaryExe = binaryExes.length === 1 ? binaryExes[0] : null;
  const depFiles = after.entries.filter(entry => entry.kind === "file" && entry.path.endsWith(".d"));
  const binaryDepCandidates = depFiles.filter(entry => tests.targets.binary.dep_info_paths.includes(entry.path) || cargoHashedArtifactPath(tests.targets.binary.name, entry, "\\.d"));
  const binaryDep = binaryDepCandidates.length === 1 ? binaryDepCandidates[0] : null;
  const integrationDepCandidates = depFiles.filter(entry => cargoHashedArtifactPath(tests.targets.integration.name, entry, "\\.d"));
  const integrationDep = integrationDepCandidates.length === 1 ? integrationDepCandidates[0] : null;
  const missing = [];
  if (integrationExes.length !== 1) missing.push("expected exactly one hash-pinned integration executable for " + tests.targets.integration.name);
  if (binaryExes.length !== 1) missing.push("expected exactly one fixture build-dependency executable at " + tests.targets.binary.artifact_path);
  if (binaryDepCandidates.length !== 1) missing.push("expected exactly one fixture .d dep-info artifact from the exact allowed path forms");
  if (integrationDepCandidates.length !== 1) missing.push("expected exactly one hash-pinned integration .d dep-info artifact");
  if (!binaryExe || !beforePaths.has(binaryExe.path) && !changed.some(entry => entry.path === binaryExe.path)) missing.push("binary executable not produced: " + tests.targets.binary.artifact_path);
  if (!integrationExe) missing.push("integration test executable not found for " + tests.targets.integration.name);
  if (binaryExe && (binaryExe.mode & 0o7777) !== 0o755) missing.push("build-dependency helper executable mode is not 0755: " + binaryExe.path);
  if (integrationExe && (integrationExe.mode & 0o111) === 0) missing.push("integration test artifact is not executable: " + integrationExe.path);
  if (!binaryDep) missing.push("binary .d dep-info not found");
  if (!integrationDep) missing.push("integration .d dep-info not found");
  const provenance = [];
  for (const [role, record, sourcePath] of [["binary", binaryDep, tests.targets.binary.source_path], ["integration", integrationDep, tests.targets.integration.source_path]]) {
    if (!record) continue;
    const abs = path.join(target, record.path);
    const body = fs.readFileSync(abs, "utf8");
    const expectedSource = path.join(input.plan.cwd, sourcePath);
    const packageRelativeSource = path.relative(path.dirname(input.plan.package_manifest), expectedSource);
    const sourceEntry = input.source.build_input_files.find(file => file.path === sourcePath);
    const depTokens = body.replace(/\\\r?\n/g, " ").split(/\s+/);
    const present = depTokens.includes(packageRelativeSource) || depTokens.includes(expectedSource);
    provenance.push({role, path: record.path, sha256: record.sha256, bytes: record.bytes, source_path: sourcePath, package_relative_source_path: packageRelativeSource, source_sha256: sourceEntry ? sourceEntry.sha256 : null, source_present_in_dep_info: present});
    if (!present) missing.push(role + " .d file does not name the pinned source path");
  }
  const recorded = [binaryExe, integrationExe, binaryDep, integrationDep].filter(Boolean).map(record => ({path: record.path, sha256: record.sha256, bytes: record.bytes, mode: record.mode}));
  for (const record of recorded) if (!beforePaths.has(record.path) && !changed.some(entry => entry.path === record.path)) missing.push("artifact was not created during this run: " + record.path);
  return {ok: missing.length === 0, missing, paths: recorded, source_dep_info: provenance, new_target_files: changed.map(entry => ({path: entry.path, sha256: entry.sha256, bytes: entry.bytes})), target_artifact_candidates: targetArtifactCandidates(after, tests).map(entry => ({path: entry.path, sha256: entry.sha256, bytes: entry.bytes}))};
}
function writeEvent(fd, value) {
  fs.writeSync(fd, JSON.stringify({at: new Date().toISOString(), ...value}) + "\n");
}
function parseEvents(buffer, input) {
  const expected = new Set(input.tests.test_names);
  const started = new Set(), outcomes = new Map(), suites = new Map(), errors = [];
  const text = buffer.toString("utf8");
  if (!Buffer.from(text, "utf8").equals(buffer)) errors.push("Nextest stdout is not valid UTF-8");
  const expectedEventPrefix = input.tests.package_name + "::" + input.tests.targets.integration.event_test_binary + "$";
  for (const [index, line] of text.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    let event;
    try { event = JSON.parse(line); } catch (_) { errors.push("invalid JSON event line " + (index + 1)); continue; }
    if (!event || typeof event !== "object" || Array.isArray(event)) { errors.push("non-object event line " + (index + 1)); continue; }
    if (event.type === "suite") {
      const meta = event.nextest || {};
      if (meta.crate !== input.tests.package_name) { errors.push("unexpected suite package at line " + (index + 1)); continue; }
      const binary = meta.test_binary;
      if (typeof binary !== "string") { errors.push("suite lacks test-binary identity at line " + (index + 1)); continue; }
      const key = binary;
      const current = suites.get(key) || {test_binary: binary, kind: meta.kind, test_count: event.test_count, started: 0, ok: 0, failed: 0, phase: "before"};
      if (event.event === "started") {
        if (current.phase !== "before" || !Number.isInteger(event.test_count)) errors.push("duplicate/invalid suite start " + key);
        current.test_count = event.test_count; current.started++; current.phase = "running";
      } else if (event.event === "ok") {
        if (current.phase !== "running") errors.push("suite success without start " + key);
        current.ok++; current.phase = "ended";
      } else {
        current.failed++; current.phase = "ended"; errors.push("suite did not finish successfully: " + key + "/" + event.event);
      }
      suites.set(key, current);
      continue;
    }
    if (event.type !== "test") { errors.push("unexpected event type at line " + (index + 1)); continue; }
    if (typeof event.name !== "string") { errors.push("test event has no name at line " + (index + 1)); continue; }
    if (!event.name.startsWith(expectedEventPrefix)) { errors.push("test event has unexpected package/binary identity: " + event.name); continue; }
    const name = event.name.slice(expectedEventPrefix.length);
    if (!expected.has(name)) { errors.push("unexpected test name: " + name); continue; }
    if (event.event === "started") {
      if (started.has(name) || outcomes.has(name)) errors.push("duplicate/out-of-order test start: " + name);
      started.add(name);
    } else {
      if (!started.has(name) || outcomes.has(name)) errors.push("terminal event without exactly one start: " + name);
      const outcome = event.event === "ok" ? "passed" : event.event === "failed" ? "failed" : "incomplete";
      outcomes.set(name, outcome);
    }
  }
  for (const name of expected) {
    if (!started.has(name)) errors.push("missing start: " + name);
    if (!outcomes.has(name)) errors.push("missing terminal result: " + name);
    else if (outcomes.get(name) !== "passed") errors.push("not passed: " + name + " (" + outcomes.get(name) + ")");
  }
  const integrationSuite = suites.get(input.tests.targets.integration.event_test_binary);
  if (!integrationSuite || integrationSuite.started !== 1 || integrationSuite.ok !== 1 || integrationSuite.test_count !== input.tests.test_names.length || integrationSuite.phase !== "ended") errors.push("exact integration test suite did not complete with the expected count");
  for (const suite of suites.values()) {
    const isIntegration = suite.test_binary === input.tests.targets.integration.event_test_binary;
    const isBuildDependency = suite.test_binary === input.tests.targets.binary.name.replace(/-/g, "_");
    if (isBuildDependency) errors.push("unselected build-dependency binary appeared as a test suite: " + suite.test_binary);
    else if (!isIntegration) errors.push("unselected test binary appeared: " + suite.test_binary);
    if (!isIntegration) continue;
    if (suite.started !== 1 || suite.ok !== 1 || suite.phase !== "ended") errors.push("suite did not have one ordered successful lifecycle: " + suite.test_binary);
  }
  const results = [...expected].sort().map(name => ({name, result: outcomes.get(name) || "missing"}));
  const passed = results.filter(result => result.result === "passed").length;
  const failed = results.filter(result => result.result === "failed").length;
  const incomplete = results.filter(result => result.result === "incomplete" || result.result === "missing").length;
  return {errors, started: started.size, passed, failed, incomplete, suites: [...suites.values()], results};
}
function preflight() {
  const input = verifyInputs();
  if (!isAbsent(PREFLIGHT_PATH)) fail("preflight receipt already exists");
  const source = sourceAudit(input), tools = toolAudit(input), cache = cacheAudit(input);
  const target = snapshotTree(input.plan.environment.CARGO_TARGET_DIR);
  const stale = targetArtifactCandidates(target, input.tests);
  if (stale.length) fail("new binary/integration artifacts already exist before execution: " + stale.map(entry => entry.path).join(", "));
  const outputAbsent = isAbsent(input.output);
  if (!outputAbsent) fail("fresh output root appeared during preflight");
  writeJson(path.join(PREFLIGHT_DIR, "source-before.json"), source);
  writeJson(path.join(PREFLIGHT_DIR, "tool-before.json"), tools);
  writeJson(path.join(PREFLIGHT_DIR, "cache-before.json"), cache);
  writeJson(path.join(PREFLIGHT_DIR, "target-before.json"), target);
  const receipt = {
    schema: "termrock.nextest-runner-preflight.v1",
    result: "PREFLIGHT_READY_NO_CARGO",
    runner_path: __filename,
    runner_sha256: shaFile(__filename),
    runplan_sha256: shaFile(PLAN_PATH),
    source_manifest_sha256: shaFile(SOURCE_PATH),
    test_names_sha256: shaFile(TESTS_PATH),
    cache_closure_sha256: shaFile(input.plan.bindings.cache_closure_path),
    packet_checksums_sha256: shaFile(PACKET_CHECKSUMS_PATH),
    review_request_sha256: shaFile(REQUEST_PATH),
    cargo_spawned: false, nextest_spawned: false, tests_run: false, network_used: false,
    selected_cases_expected_from_source: input.tests.test_names.length,
    target_selectors: {package: input.tests.package_name, test: input.tests.targets.integration.name},
    build_dependencies: [{name: input.tests.targets.binary.name, selection_role: input.tests.targets.binary.selection_role, source_unit_test_count: input.tests.targets.binary.expected_unit_test_count, source_unit_test_names: input.tests.targets.binary.source_unit_test_names, selected_unit_test_count: input.tests.targets.binary.selected_unit_test_count, artifact_path: input.tests.targets.binary.artifact_path, artifact_mode: "0755", dep_info_path: input.tests.targets.binary.dep_info_path}],
    source_digest: source.digest, tool_digest: tools.digest, cache_digest: cache.digest,
    target_root: target.target_root, target_digest: target.digest, target_entries: target.entry_count, target_file_bytes: target.regular_file_bytes,
    new_target_artifacts_absent: true, output_root_absent: outputAbsent,
    created_at: new Date().toISOString()
  };
  writeJson(PREFLIGHT_PATH, receipt);
  process.stdout.write(JSON.stringify(receipt, null, 2) + "\n");
  return receipt;
}
function sameDigest(before, after, label) {
  if (before.digest !== after.digest) fail(label + " changed between preflight and spawn");
}
function run() {
  const input = verifyInputs();
  const preStat = fs.lstatSync(PREFLIGHT_PATH);
  if (!preStat.isFile() || preStat.isSymbolicLink() || (preStat.mode & 0o077) !== 0) fail("preflight receipt is not a private regular file");
  const pre = readJson(PREFLIGHT_PATH);
  if (pre.schema !== "termrock.nextest-runner-preflight.v1" || pre.result !== "PREFLIGHT_READY_NO_CARGO" || pre.runner_sha256 !== shaFile(__filename) || pre.runplan_sha256 !== shaFile(PLAN_PATH) || pre.source_manifest_sha256 !== shaFile(SOURCE_PATH) || pre.test_names_sha256 !== shaFile(TESTS_PATH) || pre.packet_checksums_sha256 !== shaFile(PACKET_CHECKSUMS_PATH) || pre.review_request_sha256 !== shaFile(REQUEST_PATH) || pre.cargo_spawned !== false || pre.nextest_spawned !== false || pre.tests_run !== false || pre.network_used !== false) fail("preflight receipt does not bind this exact no-Cargo input");
  const sourceBefore = sourceAudit(input), toolsBefore = toolAudit(input), cacheBefore = cacheAudit(input);
  sameDigest({digest: pre.source_digest}, sourceBefore, "source");
  sameDigest({digest: pre.tool_digest}, toolsBefore, "toolchain");
  sameDigest({digest: pre.cache_digest}, cacheBefore, "cache closure");
  const targetBefore = snapshotTree(input.plan.environment.CARGO_TARGET_DIR);
  if (targetBefore.digest !== pre.target_digest) fail("warm target changed after preflight; rerun in a new reviewed packet");
  if (targetArtifactCandidates(targetBefore, input.tests).length) fail("new CLI target artifacts are present before execution");
  if (!isAbsent(input.output)) fail("fresh output root already exists");

  const output = input.output;
  fs.mkdirSync(output, {mode: 0o700}); fs.chmodSync(output, 0o700);
  ensurePrivateDirectory(output, false);
  fs.mkdirSync(input.plan.environment.HOME, {mode: 0o700}); fs.chmodSync(input.plan.environment.HOME, 0o700);
  fs.mkdirSync(input.plan.environment.TMPDIR, {mode: 0o700}); fs.chmodSync(input.plan.environment.TMPDIR, 0o700);
  ensurePrivateDirectory(input.plan.environment.HOME, false);
  ensurePrivateDirectory(input.plan.environment.TMPDIR, false);
  writeJson(path.join(output, "source-before.json"), sourceBefore);
  writeJson(path.join(output, "tool-before.json"), toolsBefore);
  writeJson(path.join(output, "cache-before.json"), cacheBefore);
  writeJson(path.join(output, "target-before.json"), targetBefore);

  const stdoutPath = path.join(output, "nextest.stdout.jsonl");
  const stderrPath = path.join(output, "nextest.stderr.log");
  const eventsPath = path.join(output, "process-events.jsonl");
  const receiptPath = path.join(output, "final-receipt.json");
  const stdoutFd = fs.openSync(stdoutPath, "wx", 0o600), stderrFd = fs.openSync(stderrPath, "wx", 0o600), eventsFd = fs.openSync(eventsPath, "wx", 0o600);
  const streams = {stdout: {received: 0, stored: 0, truncated: false}, stderr: {received: 0, stored: 0, truncated: false}};
  const errors = {io: [], cleanup: [], signal: []};
  const start = Date.now(), startedAt = new Date().toISOString();
  let total = 0, child = null, closeEvent = null, spawnError = null, timedOut = false, interruptedBy = null, capExceeded = false, forcedClose = false, reason = null, finalized = false;
  let timeoutTimer = null, termTimer = null, killTimer = null;
  const signalGroup = signal => {
    if (!child || !child.pid) return;
    try { process.kill(-child.pid, signal); }
    catch (error) {
      if (error.code !== "ESRCH") {
        errors.signal.push(signal + ": " + String(error));
        try { child.kill(signal); } catch (direct) { if (direct.code !== "ESRCH") errors.signal.push("direct " + signal + ": " + String(direct)); }
      }
    }
  };
  const stop = why => {
    if (reason || finalized) return;
    reason = why; writeEvent(eventsFd, {event: "stop", reason: why}); signalGroup("SIGTERM");
    termTimer = setTimeout(() => {
      signalGroup("SIGKILL");
      killTimer = setTimeout(() => {
        if (!closeEvent && !finalized) {
          forcedClose = true;
          try { if (child && child.pid) child.kill("SIGKILL"); } catch (error) { errors.signal.push("direct SIGKILL: " + String(error)); }
          if (child && child.stdout) child.stdout.destroy();
          if (child && child.stderr) child.stderr.destroy();
          finalize(); process.exitCode = 125;
        }
      }, input.plan.limits.post_kill_grace_ms);
    }, input.plan.limits.term_grace_ms);
  };
  const capture = (which, chunk) => {
    if (finalized) return;
    const state = streams[which]; state.received += chunk.length;
    const remaining = Math.max(0, input.plan.limits.combined_stdout_stderr_cap_bytes - total);
    const keep = Math.min(remaining, chunk.length);
    if (keep > 0) {
      try {
        const fd = which === "stdout" ? stdoutFd : stderrFd;
        fs.writeSync(fd, chunk, 0, keep); state.stored += keep; total += keep;
      } catch (error) { errors.io.push(which + " write failed: " + String(error)); stop("OUTPUT_WRITE_FAILURE"); }
    }
    if (keep < chunk.length) { state.truncated = true; capExceeded = true; stop("OUTPUT_CAP_EXCEEDED"); }
  };
  const onInt = () => { interruptedBy = "SIGINT"; writeEvent(eventsFd, {event: "parent-signal", signal: "SIGINT"}); stop("INTERRUPTED"); };
  const onTerm = () => { interruptedBy = "SIGTERM"; writeEvent(eventsFd, {event: "parent-signal", signal: "SIGTERM"}); stop("INTERRUPTED"); };
  process.on("SIGINT", onInt); process.on("SIGTERM", onTerm);

  const finalize = () => {
    if (finalized) return;
    finalized = true; clearTimeout(timeoutTimer); clearTimeout(termTimer); clearTimeout(killTimer);
    process.removeListener("SIGINT", onInt); process.removeListener("SIGTERM", onTerm);
    try { fs.closeSync(stdoutFd); } catch (error) { errors.cleanup.push("stdout close: " + String(error)); }
    try { fs.closeSync(stderrFd); } catch (error) { errors.cleanup.push("stderr close: " + String(error)); }
    writeEvent(eventsFd, {event: "finalize", direct_child_reaped: Boolean(closeEvent)});
    try { fs.closeSync(eventsFd); } catch (error) { errors.cleanup.push("event log close: " + String(error)); }
    const stdout = fs.readFileSync(stdoutPath), stderr = fs.readFileSync(stderrPath);
    let sourceAfter, toolsAfter, cacheAfter, targetAfter, artifactReport;
    try { sourceAfter = sourceAudit(input); writeJson(path.join(output, "source-after.json"), sourceAfter); }
    catch (error) { errors.io.push("source post-audit: " + String(error)); sourceAfter = {error: String(error)}; }
    try { toolsAfter = toolAudit(input); writeJson(path.join(output, "tool-after.json"), toolsAfter); }
    catch (error) { errors.io.push("tool post-audit: " + String(error)); toolsAfter = {error: String(error)}; }
    try { cacheAfter = cacheAudit(input); writeJson(path.join(output, "cache-after.json"), cacheAfter); }
    catch (error) { errors.io.push("cache post-audit: " + String(error)); cacheAfter = {error: String(error)}; }
    try {
      targetAfter = snapshotTree(input.plan.environment.CARGO_TARGET_DIR);
      writeJson(path.join(output, "target-after.json"), targetAfter);
      artifactReport = verifyTargetArtifacts(targetBefore, targetAfter, input);
    } catch (error) { errors.io.push("target/artifact post-audit: " + String(error)); targetAfter = {error: String(error)}; artifactReport = {ok: false, missing: [String(error)]}; }
    let events;
    try { events = capExceeded ? {errors: ["combined stdout/stderr cap exceeded"], started: 0, passed: 0, failed: 0, incomplete: 0, suites: [], results: []} : parseEvents(stdout, input); }
    catch (error) { events = {errors: ["event parser: " + String(error)], started: 0, passed: 0, failed: 0, incomplete: input.tests.test_names.length, suites: [], results: []}; }
    const packetInputChanges = [];
    for (const [name, file, expected] of [["runplan", PLAN_PATH, input.packetPins.runplan_sha256], ["source manifest", SOURCE_PATH, input.packetPins.source_manifest_sha256], ["test names", TESTS_PATH, input.packetPins.test_names_sha256], ["cache closure", input.plan.bindings.cache_closure_path, input.packetPins.cache_closure_sha256], ["runner", __filename, input.packetPins.runner_sha256], ["packet checksums", PACKET_CHECKSUMS_PATH, input.packetPins.packet_checksums_sha256], ["review request", REQUEST_PATH, input.packetPins.review_request_sha256]]) {
      try { if (shaFile(file) !== expected) packetInputChanges.push(name); }
      catch (_) { packetInputChanges.push(name + " unreadable"); }
    }
    const stableInputs = !sourceAfter.error && !toolsAfter.error && !cacheAfter.error && sourceBefore.digest === sourceAfter.digest && toolsBefore.digest === toolsAfter.digest && cacheBefore.digest === cacheAfter.digest && packetInputChanges.length === 0;
    const allErrors = [...events.errors, ...artifactReport.missing, ...errors.io, ...errors.cleanup, ...errors.signal];
    if (!stableInputs) allErrors.push("source, toolchain, or cache manifest changed during run");
    if (packetInputChanges.length) allErrors.push("packet inputs changed during run: " + packetInputChanges.join(", "));
    let result = "FAILED_EVENT_OR_ARTIFACT_CONTRACT";
    if (timedOut) result = "TIMED_OUT";
    else if (interruptedBy) result = "INTERRUPTED";
    else if (capExceeded) result = "OUTPUT_CAP_EXCEEDED";
    else if (spawnError) result = "SPAWN_FAILED";
    else if (closeEvent && closeEvent.code !== 0) result = "FAILED_EXIT";
    else if (forcedClose || !closeEvent) result = "BLOCKED_CHILD_NOT_REAPED";
    else if (!allErrors.length && stableInputs && artifactReport.ok && events.started === input.tests.test_names.length && events.passed === input.tests.test_names.length && events.failed === 0 && events.incomplete === 0) result = "PASSED_SELECTED_INTEGRATION_TARGET";
    const receipt = {
      schema: "termrock.nextest-execution-receipt.v1",
      result, run_id: path.basename(output), started_at: startedAt, finished_at: new Date().toISOString(), elapsed_ms: Date.now() - start,
      runplan_path: PLAN_PATH, runplan_sha256: shaFile(PLAN_PATH), source_manifest_path: SOURCE_PATH, source_manifest_sha256: shaFile(SOURCE_PATH),
      test_names_sha256: shaFile(TESTS_PATH), runner_path: __filename, runner_sha256: shaFile(__filename),
      preflight_receipt_path: PREFLIGHT_PATH, preflight_receipt_sha256: shaFile(PREFLIGHT_PATH),
      child: {program: input.plan.program, argv: input.plan.argv, cwd: input.plan.cwd, environment: input.plan.environment, pid: child && child.pid || null, process_group: child && child.pid || null, exit_code: closeEvent ? closeEvent.code : null, signal: closeEvent ? closeEvent.signal : null, spawn_error: spawnError, timed_out: timedOut, interrupted_by: interruptedBy, cap_exceeded: capExceeded, forced_close: forcedClose, direct_child_reaped: Boolean(closeEvent), cargo_spawned: Boolean(child && child.pid)},
      nextest_events: {expected_count: input.tests.test_names.length, started: events.started, passed: events.passed, failed: events.failed, incomplete: events.incomplete, suites: events.suites, results: events.results, errors: events.errors},
      source_stable_before_after: stableInputs,
      packet_input_changes: packetInputChanges,
      manifests: {source_before: sourceBefore, source_after: sourceAfter, tools_before: toolsBefore, tools_after: toolsAfter, cache_before: cacheBefore, cache_after: cacheAfter},
      target: {before_digest: targetBefore.digest, after_digest: targetAfter.digest || null, before_entries: targetBefore.entry_count, after_entries: targetAfter.entry_count || null, artifacts: artifactReport},
      output: {combined_cap_bytes: input.plan.limits.combined_stdout_stderr_cap_bytes, bytes_stored: total, stdout: {path: stdoutPath, received: streams.stdout.received, stored: streams.stdout.stored, truncated: streams.stdout.truncated, sha256: sha(stdout)}, stderr: {path: stderrPath, received: streams.stderr.received, stored: streams.stderr.stored, truncated: streams.stderr.truncated, sha256: sha(stderr)}, process_events_path: eventsPath},
      limits: {locked: true, offline: true, parent_environment_inherited: false, build_jobs: 2, test_threads: 1, retries: 0, timeout_ms: input.plan.limits.timeout_ms, term_grace_ms: input.plan.limits.term_grace_ms, post_kill_grace_ms: input.plan.limits.post_kill_grace_ms, output_cap_bytes: input.plan.limits.combined_stdout_stderr_cap_bytes},
      scope_limits: input.plan.scope_limits || [],
      errors: allErrors
    };
    writeJson(receiptPath, receipt);
    process.stdout.write(JSON.stringify({result, receipt: receiptPath, exit_code: receipt.child.exit_code, started: events.started, passed: events.passed, failed: events.failed, incomplete: events.incomplete, source_stable: stableInputs, artifacts_verified: artifactReport.ok}, null, 2) + "\n");
    if (result === "PASSED_SELECTED_INTEGRATION_TARGET") process.exitCode = 0;
    else if (timedOut) process.exitCode = 124;
    else if (interruptedBy) process.exitCode = 130;
    else if (closeEvent && closeEvent.code !== null) process.exitCode = closeEvent.code === 0 ? 1 : closeEvent.code;
    else process.exitCode = 125;
  };
  try {
    writeEvent(eventsFd, {event: "spawn-request", program: input.plan.program, argv: input.plan.argv, cwd: input.plan.cwd});
    child = spawn(input.plan.program, input.plan.argv, {cwd: input.plan.cwd, env: {...input.plan.environment}, detached: true, stdio: ["ignore", "pipe", "pipe"]});
    child.stdout.on("data", chunk => capture("stdout", chunk));
    child.stderr.on("data", chunk => capture("stderr", chunk));
    child.on("error", error => { spawnError = String(error); writeEvent(eventsFd, {event: "spawn-error", error: spawnError}); });
    child.on("spawn", () => writeEvent(eventsFd, {event: "spawned", pid: child.pid, process_group: child.pid}));
    child.on("close", (code, signal) => { if (finalized) return; closeEvent = {code, signal}; writeEvent(eventsFd, {event: "child-close", code, signal}); finalize(); });
    timeoutTimer = setTimeout(() => { timedOut = true; stop("TIMEOUT"); }, input.plan.limits.timeout_ms);
  } catch (error) {
    spawnError = String(error); closeEvent = {code: null, signal: null};
    writeEvent(eventsFd, {event: "spawn-exception", error: spawnError}); finalize();
  }
}
function main() {
  const args = process.argv.slice(2);
  if (args.length === 1 && args[0] === "--preflight-only") { preflight(); return; }
  if (args.length !== 0) fail("usage: runner.cjs [--preflight-only]");
  run();
}
try { main(); }
catch (error) { process.stderr.write(String(error && error.stack || error) + "\n"); process.exitCode = 125; }
