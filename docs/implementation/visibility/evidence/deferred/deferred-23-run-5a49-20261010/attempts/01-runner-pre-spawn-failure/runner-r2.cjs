'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn } = require('node:child_process');
const dir = '/private/tmp/termrock-vis04-deferred-23-run-r2-luna-20261010';
const base = '/private/tmp/termrock-vis04-deferred-23-request-luna-20261010';
const planPath = `${dir}/runplan-r2.json`;
const plan = JSON.parse(fs.readFileSync(planPath, 'utf8'));
const originalPath = `${base}/runplan.json`;
const original = JSON.parse(fs.readFileSync(originalPath, 'utf8'));
const command = plan.commands.find(x => x.id === 'exact-selection-run');
if (!command) throw new Error('missing exact-selection-run command');
const shaBytes = b => crypto.createHash('sha256').update(b).digest('hex');
const shaFile = p => shaBytes(fs.readFileSync(p));
const expectedPins = {
  original_runplan: '6bdc37c785375026f69038fdbac403c39ef6cab92075003979ce190a7346619a',
  test_names: '03e1e994ebe6c817d99772a0ff6a475563b39345ab758b454850ac29542c9752',
  source_manifest: '396b2722afad217841f6540d33c304f372d42bf982958ab74457c250c761e01a',
  source_archive: '9e3146671332ffc8c7e85adfd0b4ab9ba0e569c72564fb94dedde3bb796ae70e',
  rustup: 'ec1b9233e7f72990ecd8e62063fa7f6c3dfc2bec8e97f88bff165f9100ac696a',
  cargo: '6e17e865f3a20dd55a1d212f849f58b77124179f0de7c52973096d84ba34118d',
  rustc: '766eda9d8f53afd6fc7f27b3cd2e444dd22afacb5afa710a5625fc8e45b8c941',
  nextest: '7a558b157d164ab4fb6cb1a48cbac5a57b7b8ad99f5d3492eb6eda64faf91df0'
};
if (shaFile(originalPath) !== expectedPins.original_runplan) throw new Error('original runplan pin mismatch');
if (shaFile(`${base}/test-names.json`) !== expectedPins.test_names) throw new Error('test-name pin mismatch');
if (shaFile(`${base}/source-manifest.json`) !== expectedPins.source_manifest) throw new Error('source-manifest pin mismatch');
if (shaFile(`${base}/source-archive.tar`) !== expectedPins.source_archive) throw new Error('source-archive pin mismatch');
if (shaFile(command.executable) !== expectedPins.rustup) throw new Error('rustup pin mismatch');
if (shaFile(plan.tools.cargo.path) !== expectedPins.cargo) throw new Error('cargo pin mismatch');
if (shaFile(plan.tools.rustc.path) !== expectedPins.rustc) throw new Error('rustc pin mismatch');
if (shaFile(plan.tools.nextest.path) !== expectedPins.nextest) throw new Error('nextest pin mismatch');
if (JSON.stringify(command.argv) !== JSON.stringify(original.commands.find(x => x.id === command.id).argv)) throw new Error('argv differs from reviewed original plan');
if (plan.selection.filterset !== original.selection.filterset || plan.selection.selected_count !== 23) throw new Error('selection changed');
if (plan.environment.values.CARGO_NET_OFFLINE !== 'true' || plan.environment.values.CARGO_BUILD_JOBS !== '2' || plan.environment.values.NEXTEST_EXPERIMENTAL_LIBTEST_JSON !== '1') throw new Error('runtime environment contract mismatch');
const selectionReceiptPath = `${base}/run-root/list-selection-verified.json`;
const selection = JSON.parse(fs.readFileSync(selectionReceiptPath, 'utf8'));
if (selection.result !== 'EXACT_23_SELECTED_ONE_IGNORED' || selection.filter_matches !== 23 || selection.ignored_selected !== 'w13_filter_wide_trail_cells_clear') throw new Error('verified exact selection missing');
const priorReceiptPath = `${base}/run-root/run-executor-receipt.json`;
const priorReceipt = JSON.parse(fs.readFileSync(priorReceiptPath, 'utf8'));
const priorStderrPath = `${base}/run-root/nextest-run.stderr.log`;
if (priorReceipt.exit_code !== 95 || priorReceipt.stdout.bytes !== 0 || priorReceipt.stderr.sha256 !== '93f527c9bc302895be4790c48d0a99671d03cadd567d900b48e33bcc8fa39fd3') throw new Error('prior startup failure is not the expected preserved attempt');
const outRoot = plan.run_root;
const outPath = plan.output_paths.run_stdout;
const errPath = plan.output_paths.run_stderr;
const receiptPath = plan.output_paths.receipt;
if (fs.existsSync(outRoot)) throw new Error(`successor output root already exists: ${outRoot}`);
const manifest = JSON.parse(fs.readFileSync(`${base}/source-manifest.json`, 'utf8'));
const sourceRoot = manifest.source.source_root;
function verifySourceTree() {
  let verified = 0;
  for (const e of manifest.source.tree_manifest) {
    const abs = path.resolve(sourceRoot, e.path);
    if (!abs.startsWith(`${sourceRoot}/`)) throw new Error(`source path escaped root: ${e.path}`);
    const st = fs.lstatSync(abs);
    if (!st.isFile() || st.isSymbolicLink()) throw new Error(`source entry is not a regular file: ${e.path}`);
    if (String((st.mode & 0o777).toString(8)) !== e.mode) throw new Error(`source mode mismatch: ${e.path}`);
    if (st.size !== e.bytes || shaFile(abs) !== e.sha256) throw new Error(`source bytes mismatch: ${e.path}`);
    verified++;
  }
  return verified;
}
const sourceFilesBefore = verifySourceTree();
fs.mkdirSync(outRoot, { recursive: false, mode: 0o700 });
for (const p of [plan.output_paths.home, plan.output_paths.tmp]) fs.mkdirSync(p, { recursive: true, mode: 0o700 });
if (!fs.existsSync(plan.output_paths.target_dir)) throw new Error('reviewed list target is absent; refusing target reuse claim');
const stdoutFd = fs.openSync(outPath, 'wx', 0o600);
const stderrFd = fs.openSync(errPath, 'wx', 0o600);
const MAX_BYTES = 8 * 1024 * 1024;
const DEADLINE_MS = 30 * 60 * 1000;
const env = { ...plan.environment.values };
const started = new Date().toISOString();
const t0 = process.hrtime.bigint();
let outBytes = 0, errBytes = 0, capExceeded = false, timedOut = false, spawnError = null, killTimer = null;
const child = spawn(command.executable, command.argv, { cwd: command.cwd, env, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
const pid = child.pid ?? null;
function terminate(reason) {
  if (reason === 'timeout') timedOut = true;
  if (reason === 'cap') capExceeded = true;
  if (pid === null) return;
  try { process.kill(-pid, 'SIGTERM'); } catch {}
  if (!killTimer) killTimer = setTimeout(() => { try { process.kill(-pid, 'SIGKILL'); } catch {} }, 2000);
}
function drain(stream, fd, which) {
  stream.on('data', chunk => {
    const used = which === 'stdout' ? outBytes : errBytes;
    const keep = Math.max(0, Math.min(chunk.length, MAX_BYTES - used));
    if (keep) fs.writeSync(fd, chunk, 0, keep);
    if (which === 'stdout') outBytes += keep; else errBytes += keep;
    if (keep !== chunk.length) terminate('cap');
  });
}
drain(child.stdout, stdoutFd, 'stdout');
drain(child.stderr, stderrFd, 'stderr');
const timeout = setTimeout(() => terminate('timeout'), DEADLINE_MS);
child.on('error', e => { spawnError = `${e.name}: ${e.message}`; });
child.on('close', (code, signal) => {
  clearTimeout(timeout);
  if (killTimer) clearTimeout(killTimer);
  fs.fsyncSync(stdoutFd); fs.fsyncSync(stderrFd);
  fs.closeSync(stdoutFd); fs.closeSync(stderrFd);
  const ended = new Date().toISOString();
  const durationMs = Number(process.hrtime.bigint() - t0) / 1e6;
  const sourceFilesAfter = verifySourceTree();
  const status = spawnError ? 'SPAWN_ERROR' : timedOut ? 'TIMED_OUT' : capExceeded ? 'OUTPUT_CAP_EXCEEDED' : code === 0 ? 'EXIT_ZERO' : 'EXIT_NONZERO';
  const receipt = {
    schema: 'termrock.vis04.deferred-23-run-result/v1', status,
    source: { commit: manifest.source.commit, tree: manifest.source.tree, source_manifest_sha256: expectedPins.source_manifest, source_archive_sha256: expectedPins.source_archive, source_files_verified_before: sourceFilesBefore, source_files_verified_after: sourceFilesAfter },
    selection: { selected_count: 23, case_reference_count: 25, ignored_selected: 'w13_filter_wide_trail_cells_clear', exact_selection_receipt: selectionReceiptPath, exact_selection_receipt_sha256: shaFile(selectionReceiptPath) },
    prior_attempt: { receipt_path: priorReceiptPath, receipt_sha256: shaFile(priorReceiptPath), stderr_path: priorStderrPath, stderr_sha256: shaFile(priorStderrPath), exit_code: priorReceipt.exit_code, tests_executed: 0 },
    command: { id: command.id, cwd: command.cwd, executable: command.executable, argv: command.argv, exact_original_argv_preserved: true },
    inherited_environment: false, environment: env,
    tools: { rustup_sha256: expectedPins.rustup, cargo_sha256: expectedPins.cargo, rustc_sha256: expectedPins.rustc, nextest_sha256: expectedPins.nextest },
    target: { path: plan.output_paths.target_dir, reused_from_exact_selection_list: true, target_was_not_fresh: true },
    cache: { cargo_home: plan.cargo_home_candidate.path, offline: true, mutable_candidate_not_rehashed: true, no_network_fetch_requested: true },
    limits: { timeout_ms: DEADLINE_MS, stdout_cap_bytes: MAX_BYTES, stderr_cap_bytes: MAX_BYTES, process_group_signal_policy: 'SIGTERM then SIGKILL after 2 seconds; direct child close observed' },
    process: { pid, started_at: started, ended_at: ended, duration_ms: durationMs, exit_code: code, signal, timed_out: timedOut, output_cap_exceeded: capExceeded, spawn_error: spawnError },
    stdout: { path: outPath, bytes: outBytes, sha256: shaFile(outPath) }, stderr: { path: errPath, bytes: errBytes, sha256: shaFile(errPath) },
    r2_addendum: { path: `${dir}/retry-addendum.json`, sha256: shaFile(`${dir}/retry-addendum.json`) }
  };
  fs.writeFileSync(receiptPath, JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  process.stdout.write(JSON.stringify({ status, exit_code: code, signal, duration_ms: durationMs, stdout_bytes: outBytes, stderr_bytes: errBytes, receipt: receiptPath }) + '\n');
  process.exitCode = status === 'EXIT_ZERO' ? 0 : 1;
});
