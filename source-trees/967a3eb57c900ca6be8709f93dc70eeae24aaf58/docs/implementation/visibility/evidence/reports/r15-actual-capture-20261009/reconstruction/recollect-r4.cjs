'use strict';
const fs = require('fs');
const path = require('path');
const vm = require('vm');
const crypto = require('crypto');
const assert = require('assert');

const ROOT = '/private/tmp/termrock-rust-tool-test-execution/tag-capture-r15-r2-d990-20261009-01';
const ATTEMPT = path.join(ROOT, 'attempt-02');
const PACKAGE = '/private/tmp/termrock-rust-tool-test-execution/tag-capture-r14-r2-d990-20261009-01/package';
const OUTPUT_ROOT = '/private/tmp/termrock-tag-capture-r15-r4-output-20261009';
const OUTPUT = path.join(OUTPUT_ROOT, 'oracle-actual');
const CAPTURE_RECEIPT = path.join(OUTPUT_ROOT, 'oracle-capture-receipt.json');
const COLLECTOR_ROOT = '/private/tmp/termrock-tag-capture-r15-r4-recollection-20261009';
const REPORT_PATH = path.join(COLLECTOR_ROOT, 'reconstructed-postflight.json');
const NODE = '/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node';
const PIN = {
  runner_sha256: '0d1d07a82a1a89b684924e736c3bc743e306b4bd197e6bdb480891745ad8a425',
  manifest_sha256: 'ded529a2c0e0d9080029ee05fc843fec15d9cba8905749b026d160a6ce0042c3',
  template_sha256: '83159f9a2f30fc9d97f5b6c2beb82cb45f6a50985de7ac7f49849194d2570955',
  preflight_sha256: '4edd546cc689b1b1db6b7c198906d638bb1e102b971c86e0fb4ea3ec32bc05a3',
  original_wrapper_sha256: 'bbbaa88c6d00da04f28b3d9237397c42f0bd66b251842921c29dca89e7a09dd3',
  stdout_sha256: '61cae509dcfa8d7185726fe3e256e9c1c7e479235e1b70aaf03d105470e2ec1c',
  stderr_sha256: 'c8729887173fb6daefab7388eb0e473b9b562261ed627878f94a7e07469b2e5d',
  events_sha256: '4fa05c3e8de92bf1375c46bfe62b892b7f7cc650e613711713211a24a5279426',
  capture_receipt_sha256: 'dba8be2ed8c80e6c56418ba9e78b476b65fe027c964b757903daa5c97fd8eea2',
  package_freeze_sha256: '09429c5647d554bf53dfe66e814cea5d7ea8d47e7143d12dbbf3e689cdd13f52',
  suite_digest: '037460649d3663f27d4c3af77c36d3519f3262fae82b99535396aa0eb65f0590',
  case_set_digest: '06d263dba644256f58781d2950b3bf162a692dddb2bdaa423e755dd9672998bd',
  profile_digest: 'fc36250c315565fe5ef1b7e099b5a868398e76aaa6fd209fcf87c22824289af6',
  lock_sha256: '6c24ba612e204973dbf9414af15c8d5ee33f2d2ae6bc11bbcaff97a9c9f1e73e',
  renderer_hash: 'e25cbd7fc819a7c73d69493755154f03bf8bc1afea3f87c9a426557e637c4db4',
  test_binary_sha256: '7014b46f1facde2ac83894702619dec1565453a5163a87eb5289aa56ce3c4060',
  tag_object: '1ee5ebdcb91fd87adb9a5b28e43d4c7f421706c5',
  tag_commit: '4a79c0a2d40fca46fc406b77157ce3b3f12ec16b',
  tag_tree: '0b1f13431fdfd6060cf9f45a114afa5a99cc6c26',
  tag_executable_sha256: 'a3b93ef2c1b018cc2b337a7ed2657450669a9c9497aa35733bce2d09fe2e9bd5',
  helper_path: '/private/tmp/termrock-rust-tool-test-execution/tag-capture-r14-r2-d990-20261009-01/inputs/validator/subjects.py',
  helper_sha256: 'd990b77fbec0d01f242dbd191698939cbf5790d8dcd203337166c26688d85853',
  helper_toolchain_path: '/private/tmp/termrock-rust-tool-test-execution/tag-capture-r14-r2-d990-20261009-01/inputs/validator/toolchain.json',
  helper_toolchain_sha256: 'd3bce7d5775bf755e3d0a6fb675a4eea3e62a2ffaf775a4874d91f7a43bff072',
  expected_run_id: '00c75b56-9a85-494b-9e17-6061d3a9c9df',
};

function bytesHash(bytes) { return crypto.createHash('sha256').update(bytes).digest('hex'); }
function fileHash(file) { return bytesHash(fs.readFileSync(file)); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, 'utf8')); }
function u64(number) { const b = Buffer.alloc(8); b.writeBigUInt64BE(BigInt(number)); return b; }
function filesBelow(root) {
  const rows = [];
  function walk(directory, prefix) {
    for (const entry of fs.readdirSync(directory, {withFileTypes:true})) {
      const relative = prefix ? prefix + '/' + entry.name : entry.name;
      const full = path.join(directory, entry.name);
      const stat = fs.lstatSync(full);
      assert(!stat.isSymbolicLink(), 'symlink found in pinned file tree: ' + relative);
      if (stat.isDirectory()) {
        if (!relative.split('/').some(part => part === 'target' || part === '.git')) walk(full, relative);
      } else {
        assert(stat.isFile(), 'special path found in pinned file tree: ' + relative);
        rows.push({relative, full});
      }
    }
  }
  walk(root, '');
  rows.sort((a, b) => Buffer.compare(Buffer.from(a.relative), Buffer.from(b.relative)));
  return rows;
}
function digestTree(root) {
  const digest = crypto.createHash('sha256');
  for (const row of filesBelow(root)) {
    const relative = Buffer.from(row.relative, 'utf8');
    const contents = fs.readFileSync(row.full);
    digest.update(u64(relative.length));
    digest.update(relative);
    digest.update(u64(contents.length));
    digest.update(contents);
  }
  return digest.digest('hex');
}
function replaceExactly(source, before, after, label) {
  const count = source.split(before).length - 1;
  assert.strictEqual(count, 1, label + ' replacement source occurrence count');
  return source.replace(before, after);
}
function deepEqualActual(actual, expected, label) {
  assert.deepStrictEqual(actual, expected, label);
}
function exactFrequency(actual, expected, label) {
  const frequencies = values => {
    const map = new Map();
    for (const value of values) map.set(value, (map.get(value) || 0) + 1);
    return [...map.entries()].sort((a,b)=>a[0].localeCompare(b[0]));
  };
  deepEqualActual(frequencies(actual), frequencies(expected), label);
}

assert.strictEqual(process.execPath, NODE, 'recollector must use pinned Node executable');
assert.strictEqual(process.version, 'v24.20.0', 'recollector must use Node 24.20.0');
assert.strictEqual(fs.realpathSync(COLLECTOR_ROOT), COLLECTOR_ROOT, 'collector root must be canonical');
assert(!fs.existsSync(REPORT_PATH), 'reconstructed report must be new');
for (const [name, file, expected] of [
  ['runner', path.join(ROOT, 'runner-r4.cjs'), PIN.runner_sha256],
  ['manifest', path.join(ROOT, 'invocation-manifest-r4.json'), PIN.manifest_sha256],
  ['grant template', path.join(ROOT, 'launch-grant-r4.template.json'), PIN.template_sha256],
  ['preflight', path.join(ATTEMPT, 'preflight.json'), PIN.preflight_sha256],
  ['original wrapper receipt', path.join(ATTEMPT, 'final-receipt.json'), PIN.original_wrapper_sha256],
  ['stdout', path.join(ATTEMPT, 'stdout.log'), PIN.stdout_sha256],
  ['stderr', path.join(ATTEMPT, 'stderr.log'), PIN.stderr_sha256],
  ['process events', path.join(ATTEMPT, 'process-events.jsonl'), PIN.events_sha256],
  ['capture receipt', CAPTURE_RECEIPT, PIN.capture_receipt_sha256],
]) assert.strictEqual(fileHash(file), expected, name + ' immutable input SHA-256');

const runnerPath = path.join(ROOT, 'runner-r4.cjs');
const originalRunner = fs.readFileSync(runnerPath, 'utf8');
const packageFreeze = readJson(path.join(ROOT, 'candidate-freeze.json'));
assert.strictEqual(fileHash(path.join(ROOT, 'candidate-freeze.json')), PIN.package_freeze_sha256, 'candidate freeze digest');
const registryPath = path.join(PACKAGE, 'cases/registry.json');
const registry = readJson(registryPath);
const caseRoot = path.join(PACKAGE, 'cases');
const caseDigest = digestTree(caseRoot);
const profilePath = path.join(PACKAGE, 'profile.json');
const profile = readJson(profilePath);
const profileDigest = fileHash(profilePath);
const lockDigest = fileHash(path.join(PACKAGE, 'Cargo.lock'));
assert.strictEqual(registry.suite_revision, 'termrock-e2e-2026-10-09.2', 'frozen registry suite revision');
assert.strictEqual(caseDigest, PIN.case_set_digest, 'case-tree digest using the pinned Rust digest_tree algorithm');
assert.strictEqual(profileDigest, PIN.profile_digest, 'profile digest');
assert.strictEqual(lockDigest, PIN.lock_sha256, 'Cargo.lock digest');
assert.strictEqual(profile.renderer.expected_hash, PIN.renderer_hash, 'renderer hash in frozen profile');

const oldExpected = "const expectedSuite = {revision:'termrock-e2e-2026-10-09.1', caseSetDigest:'bd786b32e08bb1c88254caaa2e1203077363683ca3dcac0838b07dc7250cc8b5', profileDigest:'fc36250c315565fe5ef1b7e099b5a868398e76aaa6fd209fcf87c22824289af6', rendererHash:'e25cbd7fc819a7c73d69493755154f03bf8bc1afea3f87c9a426557e637c4db4'};";
const newExpected = 'const expectedSuite = ' + JSON.stringify({
  revision: registry.suite_revision,
  caseSetDigest: caseDigest,
  profileDigest,
  rendererHash: profile.renderer.expected_hash,
}) + ';';
let collectorSource = replaceExactly(originalRunner, oldExpected, newExpected, 'frozen suite metadata');
const oldInteractionCollector = "const interactions = receipt.checks.filter(check => check.dimension === 'interaction' && check.subject_role === 'oracle' && check.status === 'PASS');";
const newInteractionCollector = "const interactions = receipt.checks.filter(check => check.dimension === 'interaction' && check.subject_role === 'oracle' && check.status === 'PASS' && check.id.split(':').length === 3);";
collectorSource = replaceExactly(collectorSource, oldInteractionCollector, newInteractionCollector, 'interaction assertion selector');
const launchSentinel = 'const launchGrant = readLaunchGrant();';
const launchEnd = collectorSource.indexOf(launchSentinel);
assert(launchEnd >= 0, 'runner launch sentinel');
const exportsSource = `\nmodule.exports = { inventory, toolContext, sdkContext, rendererContext, reviewContext, gitContext, cacheContext, tagContext, validatorContext, captureContext };\n`;
const moduleObject = {exports:{}};
const sandboxProcess = Object.create(process);
sandboxProcess.argv = [process.execPath, runnerPath];
const sandbox = {require, process:sandboxProcess, __filename:runnerPath, module:moduleObject, Buffer, console, setTimeout, clearTimeout, setInterval, clearInterval};
vm.createContext(sandbox);
vm.runInContext(collectorSource.slice(0, launchEnd) + exportsSource, sandbox, {filename:runnerPath + '#reconstructed-postflight', timeout:30000});
const api = moduleObject.exports;

const preflightPath = path.join(ATTEMPT, 'preflight.json');
const preflight = readJson(preflightPath);
const originalWrapper = readJson(path.join(ATTEMPT, 'final-receipt.json'));
const captureReceipt = readJson(CAPTURE_RECEIPT);
const currentFreeze = readJson(path.join(ROOT, 'candidate-freeze.json'));
assert.strictEqual(originalWrapper.runner.sha256, PIN.runner_sha256, 'actual execution runner binding');
assert.strictEqual(originalWrapper.process.exit_code, 0, 'actual Nextest child exit');
assert.strictEqual(originalWrapper.process.wrapper_exit_code, 125, 'preserve original wrapper failure');
assert.strictEqual(originalWrapper.process.timed_out, false, 'no timeout');
assert.strictEqual(originalWrapper.process.raw_output_limit_exceeded, false, 'no output cap');
assert.strictEqual(originalWrapper.process.signal, null, 'no signal');
assert.strictEqual(originalWrapper.process.termination_reason, null, 'no forced termination');
assert.strictEqual(originalWrapper.nextest_run_id, PIN.expected_run_id, 'Nextest run ID');
assert.strictEqual(originalWrapper.result_counts.expected_tests, 1, 'selected test count');
assert.strictEqual(originalWrapper.result_counts.observed_tests, 1, 'observed test count');
assert.strictEqual(originalWrapper.result_counts.selection_count_ok, true, 'selected count matched');
assert.strictEqual(originalWrapper.input_stable, false, 'original wrapper input_stable value preserved');
assert.strictEqual(originalWrapper.after_snapshot, null, 'original postflight was absent');
assert(originalWrapper.postflight_error.includes('capture receipt does not satisfy the immutable-tag actual-only contract'), 'preserve original postflight diagnostic');
assert.strictEqual(originalWrapper.capture, null, 'original wrapper did not serialize a capture summary');

const stdoutPath = path.join(ATTEMPT, 'stdout.log');
const stderrPath = path.join(ATTEMPT, 'stderr.log');
const eventsPath = path.join(ATTEMPT, 'process-events.jsonl');
const invocationManifestPath = path.join(ROOT, 'invocation-manifest-r4.json');
const launchGrantPath = path.join(ROOT, 'launch-grant-r4.json');
const r4Manifest = readJson(invocationManifestPath);
const grant = readJson(launchGrantPath);
assert.strictEqual(fileHash(stdoutPath), originalWrapper.raw_outputs.stdout.sha256, 'stdout log matches original receipt');
assert.strictEqual(fileHash(stderrPath), originalWrapper.raw_outputs.stderr.sha256, 'stderr log matches original receipt');
assert.strictEqual(fileHash(eventsPath), PIN.events_sha256, 'process events remain original');
const rawOutput = fs.readFileSync(stdoutPath, 'utf8') + '\n' + fs.readFileSync(stderrPath, 'utf8');
assert(rawOutput.includes(PIN.expected_run_id), 'raw logs contain Nextest run ID');
assert(rawOutput.includes('1 test run: 1 passed, 0 skipped'), 'raw logs contain one passing selected test');
assert(/PASS \[\s*[0-9.]+s\]\s+termrock-e2e::holla_tag_capture captures_holla_from_pinned_immutable_tag/.test(rawOutput), 'raw logs name the exact passing test');
const jsonEvents = fs.readFileSync(stdoutPath, 'utf8').trim().split(/\r?\n/).map(JSON.parse);
const testName = 'termrock-e2e::holla_tag_capture$captures_holla_from_pinned_immutable_tag';
assert(jsonEvents.some(event=>event.type==='test'&&event.event==='started'&&event.name===testName), 'Nextest raw JSON records the exact selected test start');
assert(jsonEvents.some(event=>event.type==='test'&&event.event==='ok'&&event.name===testName), 'Nextest raw JSON records the exact selected test pass');
assert(jsonEvents.some(event=>event.type==='suite'&&event.event==='started'&&event.test_count===1), 'Nextest raw JSON records one selected test');
assert(jsonEvents.some(event=>event.type==='suite'&&event.event==='ok'&&event.passed===1&&event.failed===0), 'Nextest raw JSON records one pass and zero failures');
const processEvents = fs.readFileSync(eventsPath, 'utf8').trim().split(/\r?\n/).map(JSON.parse);
const spawnRequest = processEvents.find(event => event.type === 'spawn-request');
const childClose = processEvents.find(event => event.type === 'child-close');
assert(spawnRequest && spawnRequest.executable===r4Manifest.command.executable && spawnRequest.cwd===r4Manifest.command.cwd, 'raw spawn event matches frozen executable and cwd');
assert.deepStrictEqual(spawnRequest.argv, r4Manifest.command.argv, 'raw spawn argv matches frozen invocation manifest');
assert.deepStrictEqual(originalWrapper.command.argv, r4Manifest.command.argv, 'wrapper receipt argv matches frozen invocation manifest');
assert.deepStrictEqual(originalWrapper.command.environment, r4Manifest.command.environment, 'wrapper receipt environment matches frozen invocation manifest');
assert.strictEqual(originalWrapper.command.executable, r4Manifest.command.executable, 'wrapper executable matches frozen invocation manifest');
assert.strictEqual(originalWrapper.command.cwd, r4Manifest.command.cwd, 'wrapper cwd matches frozen invocation manifest');
assert(childClose && childClose.exit_code === 0 && childClose.signal === null && childClose.timed_out === false, 'raw process event confirms normal successful child close');
assert.strictEqual(originalWrapper.command.argv[originalWrapper.command.argv.indexOf('--manifest-path') + 1], path.join(PACKAGE, 'Cargo.toml'), 'executed package path');
assert.strictEqual(originalWrapper.command.environment.CARGO_NET_OFFLINE, 'true', 'offline capture command');
assert.strictEqual(originalWrapper.command.environment.CARGO_BUILD_JOBS, '2', 'build job bound');
assert.strictEqual(originalWrapper.command.environment.TERMROCK_VIS06_SUBJECTS_PY, PIN.helper_path, 'executed helper path');
assert.strictEqual(originalWrapper.command.environment.TERMROCK_VIS06_VALIDATOR_TOOLCHAIN, PIN.helper_toolchain_path, 'executed validator toolchain path');

const revalidationStartedAt = new Date().toISOString();
const current = {
  inventory: api.inventory(currentFreeze),
  tools: api.toolContext(),
  sdk: api.sdkContext(),
  renderer: api.rendererContext(),
  reviews: api.reviewContext(),
  git: api.gitContext(),
  cache: api.cacheContext(),
  tag: api.tagContext(),
  validator: api.validatorContext(),
};
const preSnapshot = preflight.source_snapshot;
const beforeAndAfter = {
  package_files: [preSnapshot.package_files, current.inventory.files],
  tools: [preSnapshot.tools, current.tools],
  sdk: [preSnapshot.sdk, current.sdk],
  renderer: [preSnapshot.renderer, current.renderer],
  reviews: [preSnapshot.reviews, current.reviews],
  git: [preSnapshot.git, current.git],
  cache: [preSnapshot.cache, current.cache],
  tag: [preSnapshot.tag, current.tag],
  validator: [preSnapshot.validator, current.validator],
};
const contextStable = {};
for (const [name, [before, after]] of Object.entries(beforeAndAfter)) contextStable[name] = JSON.stringify(before) === JSON.stringify(after);
assert(Object.values(contextStable).every(Boolean), 'all nine read-only input context groups must match preflight exactly');
const revalidationCompletedAt = new Date().toISOString();
assert.strictEqual(preSnapshot.suite_digest, current.inventory.suite_digest, 'frozen package tree digest stable');
assert.strictEqual(current.inventory.suite_digest, PIN.suite_digest, 'frozen package tree digest matches reviewed suite');
assert.strictEqual(current.cache.holla_tag_capture_binary.sha256, PIN.test_binary_sha256, 'reused compiled test binary digest');
assert.strictEqual(current.tag.tag_object, PIN.tag_object, 'immutable annotated tag object');
assert.strictEqual(current.tag.tag_commit, PIN.tag_commit, 'immutable peeled tag commit');
assert.strictEqual(current.tag.tag_tree, PIN.tag_tree, 'immutable tag tree');
assert.strictEqual(current.validator.helper_path, PIN.helper_path, 'approved helper source path');
assert.strictEqual(current.validator.helper_sha256, PIN.helper_sha256, 'approved helper content');
assert.strictEqual(current.validator.toolchain_path, PIN.helper_toolchain_path, 'approved toolchain source path');
assert.strictEqual(current.validator.toolchain_sha256, PIN.helper_toolchain_sha256, 'approved toolchain content');

const caseRecord = registry.cases.find(row => row.id === 'HELP-HOLLA-004');
assert(caseRecord, 'pinned registry includes the captured Holla case');
const normalizedSteps = caseRecord.steps.map(step => {
  if (step.op === 'press') return {op:'press', key:step.key};
  if (step.op === 'checkpoint') return {
    op:'checkpoint', id:step.id,
    wait:step.wait.map(wait => ({kind:wait.kind, needle:wait.needle})),
    assertions:step.assertions.map(assertion => ({
      id:assertion.id, kind:assertion.kind, needle:assertion.needle ?? null,
      left:assertion.left ?? null, right:assertion.right ?? null, requires:assertion.requires ?? null,
    })),
    legacy_snapshot_path:step.legacy_snapshot_path ?? null,
  };
  throw new Error('unsupported source case step: ' + step.op);
});
const inputProgramSha = bytesHash(Buffer.from(JSON.stringify(normalizedSteps), 'utf8'));
const expectedCase = {
  id:caseRecord.id, app:caseRecord.app, binary:caseRecord.binary, args:caseRecord.args,
  geometry:caseRecord.geometry, color_path:caseRecord.color_path,
  legacy_snapshot_root:caseRecord.legacy_snapshot_root, screen:caseRecord.screen,
  substep:caseRecord.substep, case_set_sha256:caseDigest,
  input_program_sha256:inputProgramSha,
  checkpoints:caseRecord.steps.filter(step=>step.op==='checkpoint').map(step=>({
    id:step.id,
    assertions:step.assertions.map(assertion=>({
      id:assertion.id, kind:assertion.kind, needle:assertion.needle ?? null,
      left:assertion.left ?? null, right:assertion.right ?? null, requires:assertion.requires ?? null,
    })),
  })),
};
assert.strictEqual(inputProgramSha, '40d5785862c34179116e2fc34beff682f296ee5fa176e4e39007b636f46d04fd', 'source case program hash');
assert.deepStrictEqual(captureReceipt.case, expectedCase, 'captured case identity exactly matches frozen registry source');

const checkpointIds = expectedCase.checkpoints.map(checkpoint=>checkpoint.id);
const expectedAssertionIds = expectedCase.checkpoints.flatMap(checkpoint=>checkpoint.assertions.map(assertion=>expectedCase.id + ':' + checkpoint.id + ':' + assertion.id));
const expectedAggregateIds = checkpointIds.map(id=>'oracle:' + expectedCase.id + ':' + id + ':interaction');
const interactionRows = captureReceipt.checks.filter(check=>check.dimension==='interaction' && check.subject_role==='oracle' && check.case_id===expectedCase.id);
const allInteractionRows = captureReceipt.checks.filter(check=>check.dimension==='interaction');
assert.strictEqual(interactionRows.length, 16, 'all 12 assertion and 4 aggregate interaction rows remain present');
assert.strictEqual(allInteractionRows.length, 16, 'no extra interaction rows are omitted from collection');
const assertionRows = interactionRows.filter(check=>!check.id.endsWith(':interaction'));
const aggregateRows = interactionRows.filter(check=>check.id.endsWith(':interaction'));
exactFrequency(assertionRows.map(check=>check.id), expectedAssertionIds, 'exact 12 assertion IDs with no omissions, duplicates, or extras');
exactFrequency(aggregateRows.map(check=>check.id), expectedAggregateIds, 'exact four checkpoint aggregate IDs with no omissions, duplicates, or extras');
exactFrequency(allInteractionRows.map(check=>check.id), expectedAssertionIds.concat(expectedAggregateIds), 'all interaction rows are exactly the 12 named assertions and four aggregates');
assert(interactionRows.every(check=>check.status==='PASS'), 'all 12 named assertions and all four aggregate rows are PASS');
assert.strictEqual(captureReceipt.checks.some(check=>check.dimension==='visual' && check.status==='PASS'), false, 'no visual result was accepted');
assert(captureReceipt.checks.every(check=>check.dimension==='visual' || check.status==='PASS' || check.status==='NOT_APPLICABLE'), 'all nonvisual check rows have allowed status');

const patchedExpected = api.captureContext();
assert.strictEqual(patchedExpected.receipt_exists, true, 'existing capture receipt read');
assert.strictEqual(patchedExpected.artifact_count, 40, '40 artifacts validated from the existing output tree');
assert.strictEqual(patchedExpected.interaction_assertions, 12, '12 named assertion rows validated using the Rust test selector');
assert.deepStrictEqual(Array.from(patchedExpected.checkpoint_ids), checkpointIds, 'four checkpoints match frozen case source');
assert.strictEqual(captureReceipt.schema, 'termrock-spec/parity-oracle-capture-receipt-v1', 'receipt schema');
assert.strictEqual(captureReceipt.run_id, 'tag-20261008T212348Z-25ce63f5a3b7', 'tag run ID');
assert.strictEqual(captureReceipt.capture_status, 'COMPLETE', 'capture completion');
assert.strictEqual(captureReceipt.state, 'capture_recorded', 'capture state');
assert.strictEqual(captureReceipt.execution_anchor_status, 'unverified', 'execution anchor remains unverified');
assert.strictEqual(captureReceipt.qualification.status, 'blocked', 'qualification remains blocked');
assert.strictEqual(captureReceipt.admission_status, 'NOT_RUN', 'admission remains not run');
assert.strictEqual(captureReceipt.suite.digest, PIN.suite_digest, 'receipt suite digest binds reviewed package');
assert.strictEqual(captureReceipt.suite.compiled_digest, PIN.suite_digest, 'compiled digest binds reviewed package');
assert.strictEqual(captureReceipt.suite.revision, registry.suite_revision, 'receipt suite revision binds frozen registry');
assert.strictEqual(captureReceipt.suite.case_set_digest, caseDigest, 'receipt case-set digest binds frozen cases tree');
assert.strictEqual(captureReceipt.suite.profile_digest, profileDigest, 'receipt profile digest');
assert.strictEqual(captureReceipt.suite.dependency_lock_sha256, lockDigest, 'receipt lock digest');
assert.strictEqual(captureReceipt.suite.test_binary_digest, PIN.test_binary_sha256, 'receipt compiled test binary digest');
assert.strictEqual(captureReceipt.renderer.hash, profile.renderer.expected_hash, 'receipt renderer');
assert.strictEqual(captureReceipt.oracle_identity.tag_object, PIN.tag_object, 'receipt annotated tag object');
assert.strictEqual(captureReceipt.oracle_identity.tag_commit, PIN.tag_commit, 'receipt peeled commit');
assert.strictEqual(captureReceipt.oracle_identity.source_snapshot_tree, PIN.tag_tree, 'receipt source tree');
assert.strictEqual(captureReceipt.oracle_identity.executable.actual_sha256, PIN.tag_executable_sha256, 'receipt executable digest');
assert.strictEqual(captureReceipt.validator_helper.path, PIN.helper_path, 'receipt helper path');
assert.strictEqual(captureReceipt.validator_helper.sha256, PIN.helper_sha256, 'receipt helper digest');
assert.strictEqual(captureReceipt.write_policy.write_root, OUTPUT_ROOT, 'receipt write root');
assert.strictEqual(captureReceipt.write_policy.receipt_path, CAPTURE_RECEIPT, 'receipt output path');

const artifactFiles = patchedExpected.output_files;
assert.strictEqual(artifactFiles.length, 40, 'exact output tree file count');
assert(artifactFiles.every(file=>file.links===1), 'captured files have unique link counts');
const expectedFormats = ['frame_json','ansi','html','png','ascii','txt','ascii_loss_json','png_fidelity_json','observations_json','manifest_json'];
const expectedArtifactKeys = checkpointIds.flatMap(id=>expectedFormats.map(format=>id+':'+format));
exactFrequency(captureReceipt.artifacts.map(artifact=>artifact.checkpoint_id+':'+artifact.format), expectedArtifactKeys, 'exact 40 artifact checkpoint/format bindings');

assert.strictEqual(originalWrapper.launch_grant.path, launchGrantPath, 'actual Root grant path in original receipt');
assert.strictEqual(originalWrapper.launch_grant.sha256, fileHash(launchGrantPath), 'actual Root grant digest in original receipt');
assert.strictEqual(originalWrapper.invocation_manifest.path, invocationManifestPath, 'actual manifest path in original receipt');
assert.strictEqual(originalWrapper.invocation_manifest.sha256, fileHash(invocationManifestPath), 'actual manifest digest in original receipt');
assert.strictEqual(fileHash(invocationManifestPath), PIN.manifest_sha256, 'executed manifest remains frozen');
assert.strictEqual(r4Manifest.inputs.candidate_suite_digest, PIN.suite_digest, 'manifest candidate suite binding');
assert.strictEqual(r4Manifest.inputs.candidate_freeze_sha256, PIN.package_freeze_sha256, 'manifest package freeze binding');
assert.strictEqual(r4Manifest.inputs.cargo_lock_sha256, PIN.lock_sha256, 'manifest lock binding');
assert.strictEqual(r4Manifest.inputs.suite_revision, 'termrock-e2e-2026-10-09.1', 'original manifest suite-revision claim preserved');
assert.strictEqual(r4Manifest.inputs.case_set_digest, 'bd786b32e08bb1c88254caaa2e1203077363683ca3dcac0838b07dc7250cc8b5', 'original manifest case-set claim preserved');
const captureTestSourcePath = path.join(PACKAGE, 'tests/holla_tag_capture.rs');
const captureTestSourceSha256 = fileHash(captureTestSourcePath);
const captureTestSource = fs.readFileSync(captureTestSourcePath, 'utf8');
assert(captureTestSource.includes("check.id.split(':').count() == 3"), 'frozen Rust test uses the 12 named assertion selector');
assert(packageFreeze.files.some(file=>file.path==='tests/holla_tag_capture.rs' && file.sha256===captureTestSourceSha256), 'capture test source is in the pinned package freeze');

const actualMetadata = {
  suite_revision: registry.suite_revision,
  case_set_digest: caseDigest,
  case_id: caseRecord.id,
  checkpoint_ids: checkpointIds,
  assertion_ids: expectedAssertionIds,
  aggregate_ids: expectedAggregateIds,
};
const originalCollectorExpectations = {
  suite_revision: 'termrock-e2e-2026-10-09.1',
  case_set_digest: 'bd786b32e08bb1c88254caaa2e1203077363683ca3dcac0838b07dc7250cc8b5',
  interaction_filter: 'PASS interaction rows only; includes checkpoint aggregates',
};
const reconstruction = {
  schema: 'termrock-rust-tool-test-execution/tag-holla-capture-reconstructed-postflight-v1',
  reconstructed_at_utc: new Date().toISOString(),
  interpretation: 'This is a new read-only reconstruction from the completed Nextest pass and existing capture bytes. It does not overwrite or revise the original wrapper receipt, which remains exit 125 with input_stable=false and no postflight snapshot.',
  execution: {
    runner_path: runnerPath,
    runner_sha256: fileHash(runnerPath),
    manifest_path: invocationManifestPath,
    manifest_sha256: fileHash(invocationManifestPath),
    launch_grant_path: launchGrantPath,
    launch_grant_sha256: fileHash(launchGrantPath),
    nextest_run_id: originalWrapper.nextest_run_id,
    nextest_child_exit_code: originalWrapper.process.exit_code,
    wrapper_exit_code: originalWrapper.process.wrapper_exit_code,
    selected_tests_expected: originalWrapper.result_counts.expected_tests,
    selected_tests_observed: originalWrapper.result_counts.observed_tests,
    selected_test_passed: true,
    timed_out: originalWrapper.process.timed_out,
    raw_output_limit_exceeded: originalWrapper.process.raw_output_limit_exceeded,
    process_signal: originalWrapper.process.signal,
    raw_logs: {
      stdout:{path:stdoutPath,sha256:fileHash(stdoutPath)},
      stderr:{path:stderrPath,sha256:fileHash(stderrPath)},
      events:{path:eventsPath,sha256:fileHash(eventsPath)},
    },
    original_wrapper_receipt:{path:path.join(ATTEMPT,'final-receipt.json'),sha256:fileHash(path.join(ATTEMPT,'final-receipt.json'))},
  },
  original_postflight_failure: {
    preserved: true,
    wrapper_input_stable: originalWrapper.input_stable,
    after_snapshot: originalWrapper.after_snapshot,
    diagnostic: originalWrapper.postflight_error,
    exact_reconstructed_cause: 'Original postflight aborted at three stale collector predicates: suite revision .1 vs frozen .2, case-set digest bd786… vs frozen 06d263…, and 16 interaction PASS rows counted as assertions instead of validating 12 named assertion IDs plus 4 checkpoint aggregate IDs. During this reconstruction, each of the nine current input context groups matched its preflight value; this point-in-time comparison does not establish continuous stability during the original child run.',
  },
  metadata_correction: {
    original_runner_and_manifest_unchanged: true,
    original_expected_metadata: originalCollectorExpectations,
    original_manifest_metadata: {suite_revision:r4Manifest.inputs.suite_revision,case_set_digest:r4Manifest.inputs.case_set_digest},
    observed_metadata_from_exact_frozen_package: actualMetadata,
    suite_digest: current.inventory.suite_digest,
    profile_digest: profileDigest,
    dependency_lock_sha256: lockDigest,
    test_binary_sha256: current.cache.holla_tag_capture_binary.sha256,
    capture_test_source_path: captureTestSourcePath,
    capture_test_source_sha256: captureTestSourceSha256,
    case_input_program_sha256: inputProgramSha,
    capture_receipt_path: CAPTURE_RECEIPT,
    capture_receipt_sha256: fileHash(CAPTURE_RECEIPT),
  },
  reconstructed_postflight: {
    type: 'reconstructed_after_completion_not_original_postflight',
    input_context_comparison_started_at_utc: revalidationStartedAt,
    input_context_comparison_completed_at_utc: revalidationCompletedAt,
    input_contexts_match_preflight: contextStable,
    input_contexts_match_preflight_at_reconstruction: Object.values(contextStable).every(Boolean),
    current_input_contexts: current,
    capture: patchedExpected,
    capture_receipt_fields: {
      capture_status:captureReceipt.capture_status,
      state:captureReceipt.state,
      execution_anchor_status:captureReceipt.execution_anchor_status,
      qualification_status:captureReceipt.qualification.status,
      admission_status:captureReceipt.admission_status,
      visual_pass_count:captureReceipt.checks.filter(check=>check.dimension==='visual'&&check.status==='PASS').length,
      interaction_pass_rows:interactionRows.length,
      named_assertion_rows:assertionRows.length,
      checkpoint_aggregate_rows:aggregateRows.length,
      artifacts:captureReceipt.artifacts.map(artifact=>({checkpoint_id:artifact.checkpoint_id,format:artifact.format,path:artifact.path,bytes:artifact.bytes,sha256:artifact.sha256})),
    },
  },
  qualification_limit: 'Actual output capture from an already built immutable-tag executable only. The source/build execution anchor remains unverified, qualification remains blocked, and admission remains NOT_RUN.',
  no_reexecution: true,
  no_product_source_or_suite_edits: true,
};
const reportBytes = Buffer.from(JSON.stringify(reconstruction, null, 2) + '\n');
fs.writeFileSync(REPORT_PATH, reportBytes, {mode:0o600, flag:'wx'});
console.log(JSON.stringify({status:'RECONSTRUCTED_POSTFLIGHT_PASS', report_path:REPORT_PATH, report_sha256:bytesHash(reportBytes), collector_path:__filename, collector_sha256:fileHash(__filename), nextest_run_id:PIN.expected_run_id, original_wrapper_exit_code:originalWrapper.process.wrapper_exit_code, original_input_stable:originalWrapper.input_stable, input_contexts_match_preflight_at_reconstruction:contextStable, suite_revision:registry.suite_revision, case_set_digest:caseDigest, assertion_rows:assertionRows.length, aggregate_rows:aggregateRows.length, artifact_count:artifactFiles.length, cargo_build_or_nextest_reexecuted:false}, null, 2));
