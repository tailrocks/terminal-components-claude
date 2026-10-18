'use strict';

// Read-only, later-point-in-time reconstruction of the preserved R4 pair-build
// attempt. This file does not launch Cargo/Nextest or any product executable.
const fs = require('node:fs/promises');
const path = require('node:path');
const crypto = require('node:crypto');

const EXEC = '/private/tmp/termrock-vis06-pair-build-r2-20261009/execution';
const ATTEMPT = path.join(EXEC, 'pair-178a-20261009-1052Z-r3-attempt-04');
const RUNNER = path.join(EXEC, 'pair-build-runner-r4.cjs');
const RUN_ROOT = '/private/tmp/termrock-vis06-real-holla-pair-178a-20261009-1052Z-r3';
const RUN_ID = 'pair-178a-20261009-1052Z-r3';
const EVIDENCE = path.join(RUN_ROOT, 'evidence', RUN_ID);
const CARGO_HOME = '/private/tmp/termrock-vis06-cargo-home-pair-178a-20261009-1052Z-r3';
const TRACE_ROOT = '/private/tmp/termrock-vis06-pair-build-r2-20261009/tool-traces/pair-178a-20261009-1052Z-r3';
const CLOSURE = '/private/tmp/termrock-vis06-pair-build-r2-20261009/preparation/cache-closure-r3.json';
const CACHE_REVIEW = '/private/tmp/termrock-vis06-pair-build-r2-20261009/preparation/cache-closure-independent-review-luna.json';
const SUITE_IDENTITY = '/private/tmp/termrock-vis06-pair-build-r2-20261009/preparation/suite-identity-r3.json';
const ACTUAL_REVIEW = path.join(ATTEMPT, 'actual-result-review-luna.json');
const STATIC_REVIEW = path.join(ATTEMPT, 'review-luna.json');
const RESULT = path.join(ATTEMPT, 'runner-result.json');
const OUTPUT = path.join(EXEC, 'pair-build-postflight-reconstruction-r4.json');

const PINS = [
  {label: 'runner', path: RUNNER, sha256: '4e75182162124f6b8398c305feba33021cdfc11463eca83ee6d1c4f23b1c322c'},
  {label: 'preflight', path: path.join(ATTEMPT, 'preflight.json'), sha256: 'c0e88ff9594195c1e509940f0a761dab689c2786b002183bb69f9a94af349212'},
  {label: 'static_review', path: STATIC_REVIEW, sha256: 'afc2d08038e9d9ab102991881bdbd5029ae791e7706f2e473d50e611a1da68e4'},
  {label: 'actual_result_review', path: ACTUAL_REVIEW, sha256: '08bb7571a165413316307f33c657965e7bffc2604a934b6a7166b779519a3727'},
  {label: 'cache_closure', path: CLOSURE, sha256: '437eeadc8cf26858e372536688fc6e14d10683557c66bca56ee8ffb272e9d158'},
  {label: 'cache_review', path: CACHE_REVIEW, sha256: '823af41a3adb98d81f2694e03b9392fc13198241501fdeee4a6d52a16e3ee511'},
  {label: 'suite_identity', path: SUITE_IDENTITY, sha256: 'a9b028f77d9590b9215d79f10658ed7498706b8c10c81858f532100a6b81e45f'},
  {label: 'runner_result', path: RESULT, sha256: '8f6204894095d1ed64cad234a5a47ecbeee6e5894ef3d1c41812a878ac70fe57'}
];

function sha256(bytes) {
  return crypto.createHash('sha256').update(bytes).digest('hex');
}

function gitBlobOid(bytes) {
  const header = Buffer.from('blob ' + bytes.length + '\0', 'utf8');
  return crypto.createHash('sha1').update(header).update(bytes).digest('hex');
}

function byteSort(left, right) {
  return Buffer.compare(Buffer.from(left, 'utf8'), Buffer.from(right, 'utf8'));
}

function statSignature(stat) {
  return [String(stat.dev), String(stat.ino), String(stat.size), String(stat.mtimeMs), String(stat.ctimeMs)].join(':');
}

async function checkedFile(filePath) {
  const before = await fs.lstat(filePath);
  if (before.isSymbolicLink() || !before.isFile()) throw new Error('expected regular non-symlink file: ' + filePath);
  const bytes = await fs.readFile(filePath);
  const after = await fs.lstat(filePath);
  if (statSignature(before) !== statSignature(after) || bytes.length !== after.size) {
    throw new Error('file changed while being read: ' + filePath);
  }
  return {
    path: filePath,
    bytes: bytes.length,
    sha256: sha256(bytes),
    mode: after.mode & 0o777
  };
}

async function checkedJson(filePath, expectedSha256) {
  const record = await checkedFile(filePath);
  if (expectedSha256 && record.sha256 !== expectedSha256) {
    throw new Error('pinned JSON changed: ' + filePath);
  }
  const bytes = await fs.readFile(filePath);
  if (sha256(bytes) !== record.sha256) throw new Error('JSON changed after hash read: ' + filePath);
  return {record, value: JSON.parse(bytes.toString('utf8'))};
}

async function pathExistsNoFollow(filePath) {
  try {
    const stat = await fs.lstat(filePath);
    return {exists: true, kind: stat.isSymbolicLink() ? 'symlink' : stat.isDirectory() ? 'directory' : stat.isFile() ? 'file' : 'other'};
  } catch (error) {
    if (error && error.code === 'ENOENT') return {exists: false, kind: 'missing'};
    throw error;
  }
}

async function walkInventory(root) {
  const entries = [];
  async function visit(directory, relative) {
    const dirents = await fs.readdir(directory, {withFileTypes: true});
    dirents.sort((a, b) => byteSort(a.name, b.name));
    for (const dirent of dirents) {
      const absolute = path.join(directory, dirent.name);
      const childRelative = relative ? relative + '/' + dirent.name : dirent.name;
      const stat = await fs.lstat(absolute);
      if (stat.isSymbolicLink()) {
        entries.push({path: childRelative, kind: 'symlink', target: await fs.readlink(absolute)});
      } else if (stat.isDirectory()) {
        entries.push({path: childRelative, kind: 'directory'});
        await visit(absolute, childRelative);
      } else if (stat.isFile()) {
        const record = await checkedFile(absolute);
        entries.push({path: childRelative, kind: 'file', bytes: record.bytes, sha256: record.sha256});
      } else {
        throw new Error('unexpected non-file in inventory: ' + absolute);
      }
    }
  }
  const rootStat = await fs.lstat(root);
  if (rootStat.isSymbolicLink() || !rootStat.isDirectory()) throw new Error('inventory root is not a real directory: ' + root);
  await visit(root, '');
  return entries;
}

async function sourceBlobMap(sourceRoot) {
  const entries = [];
  async function visit(directory, relative) {
    const dirents = await fs.readdir(directory, {withFileTypes: true});
    dirents.sort((a, b) => byteSort(a.name, b.name));
    for (const dirent of dirents) {
      const absolute = path.join(directory, dirent.name);
      const childRelative = relative ? relative + '/' + dirent.name : dirent.name;
      const stat = await fs.lstat(absolute);
      let bytes;
      if (stat.isSymbolicLink()) {
        bytes = Buffer.from(await fs.readlink(absolute), 'utf8');
      } else if (stat.isDirectory()) {
        await visit(absolute, childRelative);
        continue;
      } else if (stat.isFile()) {
        bytes = await fs.readFile(absolute);
      } else {
        throw new Error('unexpected source snapshot entry: ' + absolute);
      }
      entries.push({path: childRelative, blob_oid: gitBlobOid(bytes), bytes: bytes.length});
    }
  }
  const rootStat = await fs.lstat(sourceRoot);
  if (rootStat.isSymbolicLink() || !rootStat.isDirectory()) throw new Error('source snapshot root is not a real directory: ' + sourceRoot);
  await visit(sourceRoot, '');
  entries.sort((a, b) => byteSort(a.path, b.path));
  const hash = crypto.createHash('sha256');
  for (const entry of entries) {
    hash.update(Buffer.from(entry.path, 'utf8'));
    hash.update(Buffer.from([0]));
    hash.update(Buffer.from(entry.blob_oid, 'ascii'));
    hash.update(Buffer.from('\n'));
  }
  return {
    file_count: entries.length,
    bytes: entries.reduce((sum, entry) => sum + entry.bytes, 0),
    included_path_blob_map_sha256: hash.digest('hex')
  };
}

function exactInventoryHash(entries) {
  return sha256(Buffer.from(JSON.stringify(entries), 'utf8'));
}

async function compareCapturedTree(snapshot) {
  if (!snapshot || !snapshot.path || !Array.isArray(snapshot.entries)) {
    return {status: 'not_recorded', expected_path: snapshot && snapshot.path || null};
  }
  const actual = await walkInventory(snapshot.path);
  const expectedDigest = exactInventoryHash(snapshot.entries);
  const actualDigest = exactInventoryHash(actual);
  const recordedDigestMatchesEntries = !snapshot.inventory_sha256 || expectedDigest === snapshot.inventory_sha256;
  return {
    path: snapshot.path,
    expected_entries: snapshot.entry_count,
    actual_entries: actual.length,
    expected_inventory_sha256: snapshot.inventory_sha256 || expectedDigest,
    actual_inventory_sha256: actualDigest,
    recorded_inventory_hash_matches_entries: recordedDigestMatchesEntries,
    exact_match: recordedDigestMatchesEntries && JSON.stringify(actual) === JSON.stringify(snapshot.entries)
  };
}

function collectAbsoluteFileRecords(value, found, seen) {
  if (!value || typeof value !== 'object') return;
  if (!Array.isArray(value) && typeof value.path === 'string' && value.path.startsWith('/') &&
      /^[0-9a-f]{64}$/.test(value.sha256 || '') && Number.isFinite(value.bytes)) {
    const key = value.path + '\0' + value.sha256;
    if (!seen.has(key)) {
      seen.add(key);
      found.push(value);
    }
    return;
  }
  if (Array.isArray(value)) {
    for (const member of value) collectAbsoluteFileRecords(member, found, seen);
  } else {
    for (const member of Object.values(value)) collectAbsoluteFileRecords(member, found, seen);
  }
}

async function compareFileRecords(records) {
  const results = [];
  for (const record of records) {
    try {
      const actual = await checkedFile(record.path);
      results.push({
        path: record.path,
        expected_sha256: record.sha256,
        actual_sha256: actual.sha256,
        expected_bytes: record.bytes,
        actual_bytes: actual.bytes,
        exact_match: actual.sha256 === record.sha256 && actual.bytes === record.bytes
      });
    } catch (error) {
      results.push({path: record.path, exact_match: false, error: String(error.message || error)});
    }
  }
  return results;
}

async function compareRelativeFiles(root, records, keyName) {
  const results = [];
  for (const record of records || []) {
    const relative = record[keyName];
    if (typeof relative !== 'string' || path.isAbsolute(relative) || relative.split(/[\\/]/).includes('..')) {
      results.push({path: relative || null, exact_match: false, error: 'invalid relative input path'});
      continue;
    }
    const absolute = path.resolve(root, relative);
    if (!absolute.startsWith(path.resolve(root) + path.sep)) {
      results.push({path: relative, exact_match: false, error: 'input path escaped root'});
      continue;
    }
    try {
      const actual = await checkedFile(absolute);
      results.push({
        path: relative,
        expected_sha256: record.sha256,
        actual_sha256: actual.sha256,
        expected_bytes: record.bytes,
        actual_bytes: actual.bytes,
        exact_match: actual.sha256 === record.sha256 && actual.bytes === record.bytes
      });
    } catch (error) {
      results.push({path: relative, exact_match: false, error: String(error.message || error)});
    }
  }
  return results;
}

async function reconstruct() {
  const generatedAt = new Date().toISOString();
  const outputState = await pathExistsNoFollow(OUTPUT);
  if (outputState.exists) throw new Error('reconstruction output already exists; refusing overwrite');
  const collectorRecord = await checkedFile(__filename);

  const pinnedRecords = [];
  const pinFailures = [];
  const pinData = new Map();
  for (const pin of PINS) {
    try {
      const file = await checkedFile(pin.path);
      const valid = file.sha256 === pin.sha256;
      pinnedRecords.push({...pin, actual_sha256: file.sha256, bytes: file.bytes, exact_match: valid});
      if (!valid) pinFailures.push(pin.label);
      if (valid && ['runner_result', 'actual_result_review'].includes(pin.label)) {
        pinData.set(pin.label, JSON.parse(await fs.readFile(pin.path, 'utf8')));
      }
    } catch (error) {
      pinFailures.push(pin.label);
      pinnedRecords.push({...pin, exact_match: false, error: String(error.message || error)});
    }
  }
  if (pinFailures.length) throw new Error('fixed evidence pins did not verify: ' + pinFailures.join(', '));

  const result = pinData.get('runner_result');
  const actualReview = pinData.get('actual_result_review');
  const preflightPath = result.preflight_path;
  const invocationPath = result.invocation_path;
  const grantPath = result.grant_path;
  const [preflightRead, invocationRead, grantRead, closureRead, cacheReviewRead, suiteIdentityRead] = await Promise.all([
    checkedJson(preflightPath, result.preflight_sha256),
    checkedJson(invocationPath, result.invocation_sha256),
    checkedJson(grantPath, result.grant_sha256),
    checkedJson(CLOSURE, PINS.find(pin => pin.label === 'cache_closure').sha256),
    checkedJson(CACHE_REVIEW, PINS.find(pin => pin.label === 'cache_review').sha256),
    checkedJson(SUITE_IDENTITY, PINS.find(pin => pin.label === 'suite_identity').sha256)
  ]);
  const preflight = preflightRead.value;
  const invocation = invocationRead.value;
  const closure = closureRead.value;
  const cacheReview = cacheReviewRead.value;
  const suiteIdentity = suiteIdentityRead.value;
  const runJsonPath = path.join(EVIDENCE, 'run.json');
  const manifestPath = path.join(EVIDENCE, 'subject-manifest.json');
  const sourceInputsPath = path.join(EVIDENCE, 'source-inputs.json');
  const buildEnvironmentPath = path.join(EVIDENCE, 'build-environment.json');
  const supervisorResultPath = path.join(RUN_ROOT, 'supervisor', 'result.json');
  const artifactPaths = [runJsonPath, manifestPath, sourceInputsPath, buildEnvironmentPath, supervisorResultPath];
  const artifactRecords = [];
  for (const filePath of artifactPaths) {
    const record = await checkedFile(filePath);
    artifactRecords.push(record);
  }
  const [runRead, manifestRead, sourceInputsRead, buildEnvironmentRead, supervisorRead] = await Promise.all([
    checkedJson(runJsonPath),
    checkedJson(manifestPath),
    checkedJson(sourceInputsPath),
    checkedJson(buildEnvironmentPath),
    checkedJson(supervisorResultPath)
  ]);
  const run = runRead.value;
  const manifest = manifestRead.value;
  const sourceInputs = sourceInputsRead.value;
  const buildEnvironment = buildEnvironmentRead.value;
  const supervisor = supervisorRead.value;

  const runHash = runRead.record.sha256;
  const manifestHash = manifestRead.record.sha256;
  const sourceInputsHash = sourceInputsRead.record.sha256;
  const buildEnvironmentHash = buildEnvironmentRead.record.sha256;
  const supervisorHash = supervisorRead.record.sha256;
  const suiteBinding = {
    run_id_matches: run.run_id === RUN_ID && manifest.run_id === RUN_ID && sourceInputs.run_id === RUN_ID,
    run_state: run.state,
    suite_revision: manifest.suite_revision,
    suite_sha256: manifest.suite_sha256,
    suite_identity_matches: manifest.suite_revision === suiteIdentity.suite_revision &&
      manifest.suite_sha256 === suiteIdentity.suite_sha256,
    expected_generation: manifest.expected_generation,
    qualification_status: run.qualification && run.qualification.status,
    source_qualification_status: sourceInputs.qualification_status,
    subject_manifest_hash_matches_run: run.subject_manifest_sha256 === manifestHash,
    source_inputs_hash_matches_run: run.source_input_evidence && run.source_input_evidence.sha256 === sourceInputsHash &&
      manifest.build_evidence && manifest.build_evidence.source_inputs.sha256 === sourceInputsHash,
    build_environment_hash_matches_run: run.build_environment_evidence && run.build_environment_evidence.sha256 === buildEnvironmentHash &&
      manifest.build_evidence && manifest.build_evidence.build_environment.sha256 === buildEnvironmentHash,
    supervisor_hash: supervisorHash
  };
  const reviewedArtifacts = actualReview.pair_artifact_review || {};
  const reviewedSources = reviewedArtifacts.source_commits_and_trees || {};
  const sourceRoleBindingsMatch = ['reference', 'candidate'].every(role => {
    const reviewed = reviewedSources[role] || {};
    const recorded = sourceInputs.sources && sourceInputs.sources[role] || {};
    const runCommit = role === 'reference' ? run.reference_commit : run.candidate_commit;
    return reviewed.commit === recorded.source_commit && reviewed.commit === runCommit &&
      reviewed.tree === recorded.tree_oid;
  });
  const independentReviewBindings = {
    run_json_hash_matches: reviewedArtifacts.run_json_sha256 === runHash,
    manifest_hash_matches: reviewedArtifacts.manifest_sha256 === manifestHash,
    source_inputs_hash_matches: reviewedArtifacts.source_inputs_sha256 === sourceInputsHash,
    build_environment_hash_matches: reviewedArtifacts.build_environment_sha256 === buildEnvironmentHash,
    supervisor_hash_matches: reviewedArtifacts.supervisor_result_sha256 === supervisorHash,
    suite_revision_matches: !!reviewedArtifacts.suite && reviewedArtifacts.suite.revision === manifest.suite_revision,
    suite_sha256_matches: !!reviewedArtifacts.suite && reviewedArtifacts.suite.sha256 === manifest.suite_sha256,
    source_commits_and_trees_match: sourceRoleBindingsMatch,
    expected_generation_is_null: reviewedArtifacts.expected_generation === null && manifest.expected_generation === null,
    qualification_is_blocked: reviewedArtifacts.qualification_status === 'blocked' &&
      run.qualification && run.qualification.status === 'blocked'
  };

  const expectedCommits = {
    reference: 'b274dd57f4dd078ade6e424d546d83efbd2e8526',
    candidate: '1d797d41c8141fcbdc3f69d7f11eb8875ab54712'
  };
  const artifactSubjects = [];
  for (const subject of manifest.subjects || []) {
    const role = subject.role;
    const expectedCommit = expectedCommits[role] || null;
    const executable = subject.executable || {};
    const builder = subject.builder_receipt || {};
    let executableRecord = null;
    try {
      executableRecord = await checkedFile(executable.path);
    } catch (error) {
      executableRecord = {error: String(error.message || error)};
    }
    const buildDir = path.join(EVIDENCE, 'build', role, 'holla');
    const buildFiles = [];
    let buildExitText = null;
    for (const name of ['build.argv.json', 'build.exit', 'build.stdout', 'build.stderr']) {
      try {
        const record = await checkedFile(path.join(buildDir, name));
        buildFiles.push(record);
        if (name === 'build.exit') buildExitText = (await fs.readFile(record.path, 'utf8')).trim();
      } catch (error) {
        buildFiles.push({path: path.join(buildDir, name), error: String(error.message || error)});
      }
    }
    const actualOutputRoot = subject.actual_output_root;
    artifactSubjects.push({
      role,
      source_commit: subject.source_commit,
      expected_source_commit: expectedCommit,
      source_commit_matches: subject.source_commit === expectedCommit,
      executable_path: executable.path,
      executable_sha256_expected: executable.sha256,
      executable_sha256_current: executableRecord.sha256 || null,
      executable_bytes_current: executableRecord.bytes || null,
      executable_matches_manifest: executableRecord.sha256 === executable.sha256,
      executable_under_expected_target_root: typeof executable.path === 'string' &&
        executable.path.startsWith(path.join(EVIDENCE, 'targets', role) + path.sep),
      builder_receipt_sha256: subject.builder_receipt_sha256,
      builder_receipt_inner_sha256_matches: builder.sha256 === subject.builder_receipt_sha256,
      builder_receipt_executable_path_matches: builder.executable_path === executable.path,
      builder_receipt_executable_sha256_matches: builder.executable_sha256 === executable.sha256,
      builder_receipt_source_commit_matches: builder.source_commit === subject.source_commit,
      build_exit_text: buildExitText,
      build_exit_success: buildExitText === '0',
      build_record_files: buildFiles,
      actual_output_root: actualOutputRoot,
      actual_output_root_state: await pathExistsNoFollow(actualOutputRoot)
    });
  }

  const directRecords = [];
  const directSeen = new Set();
  collectAbsoluteFileRecords(result.inputs_before, directRecords, directSeen);
  const currentDirectRecords = await compareFileRecords(directRecords);
  const projectRoot = path.dirname(path.dirname(result.command.cwd));
  const suiteRoot = path.resolve(projectRoot, suiteIdentity.package_path);
  const suiteFiles = await compareRelativeFiles(suiteRoot, result.inputs_before.suite.files, 'path');
  const packageRoot = result.inputs_before.visibility_package.root;
  const packageFiles = await compareRelativeFiles(packageRoot, result.inputs_before.visibility_package.files, 'relative_path');

  const sourceSnapshotChecks = {};
  for (const role of ['reference', 'candidate']) {
    const sourceRecord = sourceInputs.sources && sourceInputs.sources[role];
    if (!sourceRecord || typeof sourceRecord.path !== 'string') {
      sourceSnapshotChecks[role] = {exact_match: false, error: 'source input record missing'};
      continue;
    }
    const current = await sourceBlobMap(sourceRecord.path);
    sourceSnapshotChecks[role] = {
      path: sourceRecord.path,
      recorded_commit: sourceRecord.source_commit,
      recorded_tree_oid: sourceRecord.tree_oid,
      recorded_included_file_count: sourceRecord.included_file_count,
      current_included_file_count: current.file_count,
      recorded_included_path_blob_map_sha256: sourceRecord.included_path_blob_map_sha256,
      current_included_path_blob_map_sha256: current.included_path_blob_map_sha256,
      included_source_snapshot_matches: current.file_count === sourceRecord.included_file_count &&
        current.included_path_blob_map_sha256 === sourceRecord.included_path_blob_map_sha256
    };
  }

  const rawStdoutPath = result.stdout.path;
  const rawStderrPath = result.stderr.path;
  const stdoutRecord = await checkedFile(rawStdoutPath);
  const stderrRecord = await checkedFile(rawStderrPath);
  const stdoutBytes = await fs.readFile(rawStdoutPath);
  const stderrBytes = await fs.readFile(rawStderrPath);
  const stdoutReadSha256 = sha256(stdoutBytes);
  const stderrReadSha256 = sha256(stderrBytes);
  const rawLogBindings = {
    stdout_path_is_exact_attempt_log: rawStdoutPath === path.join(ATTEMPT, 'logs', 'nextest.stdout.raw'),
    stdout_current_hash_matches_runner_result: stdoutRecord.sha256 === result.stdout.sha256,
    stdout_current_bytes_match_runner_result: stdoutRecord.bytes === result.stdout.bytes_saved &&
      result.stdout.bytes_saved === result.stdout.bytes_seen,
    stdout_parsed_buffer_hash_matches_checked_read: stdoutReadSha256 === stdoutRecord.sha256,
    stdout_parsed_buffer_length_matches_checked_read: stdoutBytes.length === stdoutRecord.bytes,
    review_has_no_stdout_binding: (
      !Object.prototype.hasOwnProperty.call(actualReview, 'stdout_path') &&
      !Object.prototype.hasOwnProperty.call(actualReview, 'stdout_sha256')
    ) || (actualReview.stdout_path === null && actualReview.stdout_sha256 === null),
    stderr_path_is_exact_attempt_log: rawStderrPath === path.join(ATTEMPT, 'logs', 'nextest.stderr.raw'),
    stderr_current_hash_matches_runner_result: stderrRecord.sha256 === result.stderr.sha256,
    stderr_current_bytes_match_runner_result: stderrRecord.bytes === result.stderr.bytes_saved &&
      result.stderr.bytes_saved === result.stderr.bytes_seen,
    stderr_parsed_buffer_hash_matches_checked_read: stderrReadSha256 === stderrRecord.sha256,
    stderr_parsed_buffer_length_matches_checked_read: stderrBytes.length === stderrRecord.bytes,
    stderr_path_matches_actual_review: rawStderrPath === actualReview.stderr_path,
    stderr_hash_matches_actual_review: result.stderr.sha256 === actualReview.stderr_sha256 &&
      stderrRecord.sha256 === actualReview.stderr_sha256,
    actual_review_binds_runner_result_path: actualReview.runner_result_path === RESULT,
    actual_review_binds_runner_result_hash: actualReview.runner_result_sha256 === PINS.find(pin => pin.label === 'runner_result').sha256
  };
  const rawLogBindingFailures = Object.entries(rawLogBindings)
    .filter(([, value]) => value !== true)
    .map(([name]) => name);
  if (rawLogBindingFailures.length) {
    throw new Error('raw log path/byte/hash bindings did not verify (' + rawLogBindingFailures.join(', ') + '); refusing to parse unbound logs');
  }
  const parsedBufferRecords = {
    stdout: {bytes: stdoutBytes.length, sha256: stdoutReadSha256},
    stderr: {bytes: stderrBytes.length, sha256: stderrReadSha256}
  };
  const stdout = stdoutBytes.toString('utf8');
  const stderr = stderrBytes.toString('utf8');
  const rawLines = (stdout + '\n' + stderr).split(/\r?\n/);
  const terminalRowRegex = /^\s*PASS\s+\[\s*[0-9.]+s\]\s+\(1\/1\)\s+termrock-visibility-tests::subjects\s+supervises_real_holla_pair_build\s*$/;
  const summaryRegex = /^\s*Summary\s+\[[^\]]+\]\s+1 test run:\s+1 passed \(1 slow\), 16 skipped\s*$/;
  const runIdRegex = /Nextest run ID ([0-9a-f-]+) with nextest profile: default/g;
  const terminalRows = rawLines.filter(line => terminalRowRegex.test(line));
  const summaries = rawLines.filter(line => summaryRegex.test(line));
  const runIds = Array.from((stdout + '\n' + stderr).matchAll(runIdRegex), match => match[1]);
  const rawChildExitIsZero = result.cargo_exit_code === 0 && result.cargo_signal === null &&
    result.timed_out === false && result.output_cap_exceeded === false && result.spawn_error === null;

  const currentCacheEntries = await walkInventory(CARGO_HOME);
  const currentCacheFiles = new Map(currentCacheEntries.filter(entry => entry.kind === 'file').map(entry => [entry.path, entry]));
  const expectedCacheFiles = new Map(closure.inventory.map(entry => [entry.path, entry]));
  const cacheMismatches = [];
  const cacheRows = [];
  let nonGlobalMatched = 0;
  for (const entry of closure.inventory) {
    const current = currentCacheFiles.get(entry.path);
    const matches = !!current && current.kind === 'file' && current.bytes === entry.size && current.sha256 === entry.sha256;
    if (!matches) cacheMismatches.push({
      path: entry.path,
      expected_size: entry.size,
      expected_sha256: entry.sha256,
      current_size: current ? current.bytes : null,
      current_sha256: current ? current.sha256 : null
    });
    if (entry.path !== '.global-cache' && matches) nonGlobalMatched += 1;
    cacheRows.push({
      path: entry.path,
      size: current ? current.bytes : null,
      sha256: current ? current.sha256 : null
    });
  }
  const extraCacheEntries = currentCacheEntries.filter(entry =>
    (entry.kind === 'file' && !expectedCacheFiles.has(entry.path)) || entry.kind === 'symlink');
  const nonGlobalCacheExtras = extraCacheEntries.filter(entry => entry.path !== '.global-cache');
  const expectedCacheDigest = sha256(Buffer.from(JSON.stringify(closure.inventory.map(entry => ({
    path: entry.path,
    size: entry.size,
    sha256: entry.sha256
  }))), 'utf8'));
  const currentCacheDigest = sha256(Buffer.from(JSON.stringify(cacheRows), 'utf8'));
  const globalExpected = closure.inventory.find(entry => entry.path === '.global-cache') || null;
  const globalCurrent = currentCacheFiles.get('.global-cache') || null;

  const traceSnapshot = result.compiler_trace_tree;
  const compilerTraceCheck = await compareCapturedTree(traceSnapshot);
  const resolvedSourceChecks = [];
  for (const snapshot of result.resolved_cargo_source_trees_after || []) {
    resolvedSourceChecks.push(await compareCapturedTree(snapshot));
  }

  const outputRoots = [];
  for (const subject of manifest.subjects || []) {
    outputRoots.push({
      role: subject.role,
      path: subject.actual_output_root,
      state: await pathExistsNoFollow(subject.actual_output_root)
    });
  }

  const pinFilesAtEnd = [];
  for (const pin of PINS) {
    try {
      const record = await checkedFile(pin.path);
      pinFilesAtEnd.push({label: pin.label, path: pin.path, sha256: record.sha256, unchanged_from_pin: record.sha256 === pin.sha256});
    } catch (error) {
      pinFilesAtEnd.push({label: pin.label, path: pin.path, unchanged_from_pin: false, error: String(error.message || error)});
    }
  }
  const directMismatches = currentDirectRecords.filter(row => !row.exact_match);
  const suiteMismatches = suiteFiles.filter(row => !row.exact_match);
  const packageMismatches = packageFiles.filter(row => !row.exact_match);
  const sourceSnapshotMismatches = Object.entries(sourceSnapshotChecks).filter(([, row]) => !row.included_source_snapshot_matches);
  const cacheImmutableMismatches = cacheMismatches.filter(row => row.path !== '.global-cache');
  const outputIntegrity = {
    all_fixed_pins_match: pinnedRecords.every(row => row.exact_match),
    direct_input_records_currently_match: directMismatches.length === 0,
    suite_package_currently_matches: suiteMismatches.length === 0,
    visibility_test_package_currently_matches: packageMismatches.length === 0,
    included_source_snapshots_currently_match: sourceSnapshotMismatches.length === 0,
    compiler_trace_currently_matches: compilerTraceCheck.exact_match === true,
    resolved_cargo_source_trees_currently_match: resolvedSourceChecks.every(row => row.exact_match === true),
    non_global_seed_cache_currently_matches: cacheImmutableMismatches.length === 0 && nonGlobalCacheExtras.length === 0,
    pin_files_stable_during_collection: pinFilesAtEnd.every(row => row.unchanged_from_pin)
  };

  const report = {
    schema: 'termrock-vis06-pair-build-postflight-reconstruction/v1',
    generated_at_utc: generatedAt,
    purpose: 'Read-only, later-point-in-time reconstruction of an existing pair-build attempt. Does not replace or upgrade its original runner result.',
    collector: {
      path: __filename,
      sha256: collectorRecord.sha256,
      node_version: process.versions.node,
      child_processes_spawned: false
    },
    original_attempt: {
      runner_result_path: RESULT,
      runner_result_sha256: PINS.find(pin => pin.label === 'runner_result').sha256,
      runner_path: RUNNER,
      runner_sha256: PINS.find(pin => pin.label === 'runner').sha256,
      attempt_root: ATTEMPT,
      run_root: RUN_ROOT,
      run_id: RUN_ID,
      original_runner_conclusion: result.conclusion,
      original_cargo_exit_code: result.cargo_exit_code,
      wrapper_exit_observation: {
        code: 125,
        observer: 'Root coordinator report',
        stored_in_runner_result: false
      },
      original_runner_counts: result.counts,
      original_postflight_errors: result.postflight_errors,
      recorded_inputs_after: result.inputs_after,
      recorded_product_build_output_tree: result.product_build_output_tree
    },
    preserved_independent_review: {
      path: ACTUAL_REVIEW,
      sha256: PINS.find(pin => pin.label === 'actual_result_review').sha256,
      verdict: actualReview.verdict,
      actual_execution: actualReview.actual_execution,
      wrapper_incomplete_causes: actualReview.wrapper_incomplete_causes,
      pair_artifact_review: actualReview.pair_artifact_review,
      cache_integrity_separation: actualReview.cache_integrity_separation
    },
    independent_review_bindings: independentReviewBindings,
    fixed_evidence_pins: pinnedRecords,
    raw_nextest_reconstruction: {
      stdout: stdoutRecord,
      stderr: stderrRecord,
      run_ids: runIds,
      selected_terminal_rows: terminalRows,
      selected_terminal_row_count: terminalRows.length,
      selected_summaries: summaries,
      summary_count: summaries.length,
      raw_log_bindings: rawLogBindings,
      parsed_buffers: parsedBufferRecords,
      cargo_exit_code: result.cargo_exit_code,
      no_timeout_or_output_cap: rawChildExitIsZero,
      cleanup: result.cleanup,
      raw_case_result: terminalRows.length === 1 && summaries.length === 1 && runIds.length === 1 && rawChildExitIsZero ?
        'ONE_SELECTED_CASE_PASS' : 'RAW_LOG_RECONSTRUCTION_INCOMPLETE',
      wrapper_parser_recorded: result.counts,
      interpretation: 'The raw case PASS is independent evidence. The original wrapper remains PAIR_BUILD_GATE_INCOMPLETE; Root separately reported wrapper exit 125, which is not stored in runner-result.json. No gate-green conclusion is assigned here.'
    },
    control_inputs: {
      preflight: {path: preflightPath, sha256: preflightRead.record.sha256},
      invocation: {path: invocationPath, sha256: invocationRead.record.sha256},
      grant: {path: grantPath, sha256: grantRead.record.sha256},
      cache_closure_review: {path: CACHE_REVIEW, sha256: cacheReviewRead.record.sha256, verdict: cacheReview.verdict},
      suite_identity: {
        path: SUITE_IDENTITY,
        sha256: suiteIdentityRead.record.sha256,
        suite_revision: suiteIdentity.suite_revision,
        suite_sha256: suiteIdentity.suite_sha256,
        package_path: suiteIdentity.package_path
      }
    },
    pair_artifact_chain: {
      files: artifactRecords,
      run_identity: {
        run_id: run.run_id,
        state: run.state,
        qualification: run.qualification,
        subject_manifest_sha256_recorded: run.subject_manifest_sha256,
        source_input_evidence: run.source_input_evidence,
        build_environment_evidence: run.build_environment_evidence
      },
      suite_binding: suiteBinding,
      subjects: artifactSubjects,
      capture_output_roots: outputRoots,
      output_root_interpretation: 'The manifest names expected app output roots; this collector records their current presence only. The actual-result review scopes this attempt to build supervision, not product launch, capture, admission, or qualification.'
    },
    source_snapshot_reconstruction: sourceSnapshotChecks,
    current_input_reconstruction: {
      absolute_file_records_checked: currentDirectRecords.length,
      absolute_file_mismatch_count: directMismatches.length,
      absolute_file_mismatches: directMismatches,
      suite_files_checked: suiteFiles.length,
      suite_mismatch_count: suiteMismatches.length,
      suite_mismatches: suiteMismatches,
      visibility_package_files_checked: packageFiles.length,
      visibility_package_mismatch_count: packageMismatches.length,
      visibility_package_mismatches: packageMismatches
    },
    cache_reconstruction: {
      closure_path: CLOSURE,
      closure_sha256: PINS.find(pin => pin.label === 'cache_closure').sha256,
      cache_review_path: CACHE_REVIEW,
      cache_review_sha256: PINS.find(pin => pin.label === 'cache_review').sha256,
      cache_review_binding_matches: cacheReview.verdict === 'PASS_CACHE_PREPARATION_ONLY' &&
        cacheReview.bindings && cacheReview.bindings.closure &&
        cacheReview.bindings.closure.sha256 === PINS.find(pin => pin.label === 'cache_closure').sha256,
      cargo_home: CARGO_HOME,
      closure_inventory_count: closure.inventory.length,
      closure_inventory_bytes: closure.inventory_bytes,
      original_inputs_before_inventory_sha256: result.inputs_before.cache.inventory_sha256,
      original_seed_inventory_sha256_recomputed: expectedCacheDigest,
      seed_inventory_matches_original_input_digest: expectedCacheDigest === result.inputs_before.cache.inventory_sha256,
      current_inventory_sha256: currentCacheDigest,
      current_matches_seed_inventory: cacheMismatches.length === 0 && extraCacheEntries.length === 0,
      current_non_global_seed_files_matching: nonGlobalMatched,
      current_non_global_mismatch_count: cacheImmutableMismatches.length,
      current_non_global_mismatches: cacheImmutableMismatches,
      current_extra_entries: extraCacheEntries.map(entry => ({path: entry.path, kind: entry.kind, bytes: entry.bytes || null})),
      current_non_global_extra_entries: nonGlobalCacheExtras.map(entry => ({path: entry.path, kind: entry.kind, bytes: entry.bytes || null})),
      global_cache_seed: globalExpected,
      global_cache_current: globalCurrent || null,
      historical_review_observation: actualReview.cache_integrity_separation
    },
    cargo_source_trees_after_reconstruction: resolvedSourceChecks,
    compiler_trace_reconstruction: compilerTraceCheck,
    qualification_and_scope: {
      qualification_status: run.qualification && run.qualification.status,
      expected_generation: manifest.expected_generation,
      actual_review_scope_limit: actualReview.pair_artifact_review && actualReview.pair_artifact_review.scope_limit,
      separate_linker_execution_proven: false,
      note: 'Compiler -### traces are retained; this reconstruction does not claim a separate ld execution record or product runtime parity.'
    },
    current_integrity_summary: outputIntegrity,
    time_basis: {
      original_execution_started_at_utc: result.started_at_utc,
      original_execution_finished_at_utc: result.finished_at_utc,
      reconstruction_generated_at_utc: generatedAt,
      interpretation: 'Current filesystem observations are later than the original execution. Historical cache observations are attributed only to the pinned independent actual-result review.'
    }
  };

  await fs.writeFile(OUTPUT, JSON.stringify(report, null, 2) + '\n', {flag: 'wx', mode: 0o600});
  const outputRecord = await checkedFile(OUTPUT);
  process.stdout.write(JSON.stringify({
    output_path: OUTPUT,
    output_sha256: outputRecord.sha256,
    original_wrapper_conclusion: result.conclusion,
    raw_nextest_case_result: report.raw_nextest_reconstruction.raw_case_result,
    suite_current_mismatches: suiteMismatches.length,
    package_current_mismatches: packageMismatches.length,
    source_snapshot_mismatches: sourceSnapshotMismatches.length,
    current_non_global_cache_mismatches: cacheImmutableMismatches.length,
    global_cache_current_bytes: globalCurrent ? globalCurrent.bytes : null,
    qualification_status: report.qualification_and_scope.qualification_status
  }) + '\n');
}

reconstruct().catch(error => {
  process.stderr.write('RECONSTRUCTION_FAILED: ' + String(error && error.stack || error) + '\n');
  process.exitCode = 1;
});
