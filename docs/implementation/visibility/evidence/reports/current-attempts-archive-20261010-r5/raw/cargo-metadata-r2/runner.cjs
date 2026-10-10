'use strict';

const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn, spawnSync } = require('node:child_process');

const ROOT = '/private/tmp/termrock-vis11-diagnostic-metadata-e0bc-r2-luna-20261010';
const PLAN = path.join(ROOT, 'evidence/plan.json');
const PLAN_SHA = '1d1f04db2f0cae97713ca358bed39b028360b084903c5aac605737c3dbad7dfb';
const CANDIDATE = '/private/tmp/termrock-vis11-diagnostic-candidate-e0bc-20261010';
const LOCK = path.join(CANDIDATE, 'Cargo.lock');
const CONFIG = path.join(CANDIDATE, '.velnor/config.toml');
const SOURCE = '/private/tmp/termrock-vis11-tool-fixture-admission-r7-build-only-lock-luna-20261010/source';
const SOURCE_INVENTORY = '/private/tmp/termrock-vis11-tool-fixture-capture-r15-build-only-lock-luna-20261010/evidence/full-source-tree-r15.json';
const SOURCE_MANIFEST = '/private/tmp/termrock-vis11-tool-fixture-admission-r7-build-only-lock-luna-20261010/review/candidate-source-manifest.json';
const PRIOR_ROOT = '/private/tmp/termrock-vis11-diagnostic-generate-e0bc-r3b-luna-20261010';
const PRIOR_RESULT = path.join(PRIOR_ROOT, 'run-root/execution/result.json');
const PRIOR_STDERR = path.join(PRIOR_ROOT, 'run-root/execution/diagnostic-render.stderr.bin');
const CACHE_HOME = path.join(PRIOR_ROOT, 'run-root/private/cargo-home');
const CARGO_INDEX = path.join(CACHE_HOME, 'registry/index/index.crates.io-1949cf8c6b5b557f');
const REQWEST_INDEX = path.join(CARGO_INDEX, '.cache/re/qw/reqwest');
const GIT_SOURCE = path.join(CACHE_HOME, 'git/checkouts/tuiscotti-9317ad0850957efe/a47c9aa');
const CARGO_CONFIG_SEARCH_DIRECTORIES = [CANDIDATE, '/private/tmp', '/private', '/'];
const EXPECTED_CANDIDATE = {
  head: 'e0bcfa26ec174241f09de17d04f3f45379be3452',
  tree: '6985c14d76029b01c2d8ee55d66cf3db801a29ef',
  status: ' M .velnor/config.toml\n',
  index_path: '/private/tmp/termrock-root-integration-20261010.git/worktrees/termrock-vis11-diagnostic-candidate-e0bc-20261010/index',
  index_sha256: '422b24dbb2d41463db1eb30bb6b3e66bf0baf95fca4dda8b6f61881e25269401',
  lock_sha256: 'a43fdf2640e4934ecb60289c215363298b5932effdc9f161495269159f1f917e',
  config_sha256: '6c1de7a7e7962308021e3d47909cc0853eb2c40e523e2ab3f9cc57ba92952279',
};
const EXPECTED_CACHE = {
  tree_sha256: 'fca99e44c62a4c1bd6fd1b274d143b7dac1e54e9f9751d4ea7856a329ba888f9',
  entries: 23570, files: 19820, directories: 3748, symlinks: 2, file_bytes: 573227343, root_mode: '755',
};
const MUTABLE_CARGO_BOOKKEEPING = {
  '.global-cache': { mode: '644', bytes: 81920, sha256: '709f053140a76abdb8305c9839a4f3930a4db9392b5697a68c1548afe261b0e5' },
  '.package-cache': { mode: '644', bytes: 0, sha256: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855' },
  '.package-cache-mutate': { mode: '644', bytes: 0, sha256: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855' },
};
const MUTABLE_CARGO_BOOKKEEPING_MAX_BYTES = 1048576;
const EXPECTED_PRIOR_RESULT_SHA = 'd3c2bc1ee512d80cc6345de03b29bbd20788211f067ab2fe55d8bf0f6bd70d30';
const EXPECTED_PRIOR_STDERR_SHA = '7a0334a65e5654743ea453f8a9b77ecb0e5bbd5891471bcc0ab97d13bcb8a91b';
const EXPECTED_CACHE_COPY_RESULT = {
  copy_verification_sha256: '47d08a01b8db1d0daaacd3ed253d89d5fbc32248eadf707e6c3bcf3a021ae1ff',
  materialized_manifest_sha256: 'f200d3754cc1b2706e949f7f10bde22b356cf9fd65ce49dbfce13d134252da76',
};
const CARGO = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/cargo';
const RUSTC = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/rustc';
const RUSTDOC = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/rustdoc';
const RUST = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin';
const NODE = '/opt/homebrew/Cellar/node/26.11.0_1/bin/node';
const GIT = '/usr/bin/git';
const GIT_GUARD = path.join(ROOT, 'tools/git');
const R1_REVIEW = '/private/tmp/termrock-vis11-diagnostic-metadata-e0bc-r1-independent-review-current-branch-comparison-luna-20261010.json';
const R1_REVIEW_SHA = '6aef478cf2a2823917e903dd089ee35a23ba8466068ce38d41abf07987ca8e8c';
const RUN = path.join(ROOT, 'run-root');
const EXEC = path.join(RUN, 'execution');
const PRIVATE = path.join(RUN, 'private');
const BIN = path.join(PRIVATE, 'bin');
const HOME = path.join(PRIVATE, 'home');
const TMP = path.join(PRIVATE, 'tmp');
const XDG = path.join(PRIVATE, 'xdg');
const TARGET = path.join(PRIVATE, 'cargo-target');
const RUSTUP_HOME = path.join(PRIVATE, 'rustup-home');
const GIT_BLOCK_LOG = path.join(EXEC, 'git-network-blocked.tsv');
const TIMEOUT = { command: 600000, term: 10000, close: 10000, output: 8388608 };
let executionStarted = false;

function hashBytes(bytes) { return crypto.createHash('sha256').update(bytes).digest('hex'); }
function hashFile(file) { return hashBytes(fs.readFileSync(file)); }
function exists(file) { try { fs.lstatSync(file); return true; } catch (error) { if (error && error.code === 'ENOENT') return false; throw error; } }
function canonical(value) {
  if (Array.isArray(value)) return '[' + value.map(canonical).join(',') + ']';
  if (value && typeof value === 'object') return '{' + Object.keys(value).sort().map(key => JSON.stringify(key) + ':' + canonical(value[key])).join(',') + '}';
  return JSON.stringify(value);
}
function inside(child, parent) {
  const relative = path.relative(parent, child);
  return relative === '' || (relative !== '..' && !relative.startsWith('..' + path.sep) && !path.isAbsolute(relative));
}
function noSymlinkComponents(file) {
  const absolute = path.resolve(file);
  const root = path.parse(absolute).root;
  let current = root;
  for (const part of absolute.slice(root.length).split(path.sep).filter(Boolean)) {
    current = path.join(current, part);
    if (fs.lstatSync(current).isSymbolicLink()) throw new Error('symlink path component: ' + current);
  }
}
function safeRelative(value) {
  return typeof value === 'string' && value.length > 0 && !value.startsWith('/') && !value.includes('\\') &&
    !/[\0\t\r\n]/.test(value) && value.split('/').every(part => part && part !== '.' && part !== '..');
}
function walkTree(root, relative = '', rows = []) {
  for (const name of fs.readdirSync(path.join(root, relative)).sort()) {
    const rel = relative ? relative + '/' + name : name;
    if (!safeRelative(rel)) throw new Error('unsafe cache path: ' + rel);
    const file = path.join(root, rel);
    const stat = fs.lstatSync(file);
    const mode = (stat.mode & 0o777).toString(8).padStart(3, '0');
    if (stat.isSymbolicLink()) rows.push({ path: rel, type: 'symlink', mode, target: fs.readlinkSync(file) });
    else if (stat.isDirectory()) { rows.push({ path: rel, type: 'directory', mode }); walkTree(root, rel, rows); }
    else if (stat.isFile()) rows.push({ path: rel, type: 'file', mode, bytes: stat.size, sha256: hashFile(file) });
    else throw new Error('special cache file: ' + rel);
  }
  return rows;
}
function treeSummary(rows) {
  return {
    inventory_schema: 'runner-walkTree-canonical/v1',
    tree_sha256: hashBytes(Buffer.from(canonical(rows))),
    entries: rows.length,
    files: rows.filter(row => row.type === 'file').length,
    directories: rows.filter(row => row.type === 'directory').length,
    symlinks: rows.filter(row => row.type === 'symlink').length,
    file_bytes: rows.filter(row => row.type === 'file').reduce((sum, row) => sum + row.bytes, 0),
  };
}
function verifySymlinks(root, rows) {
  for (const row of rows.filter(item => item.type === 'symlink')) {
    const absolute = path.resolve(root, row.path);
    const target = path.resolve(path.dirname(absolute), row.target);
    if (!inside(target, root)) throw new Error('cache symlink escapes root: ' + row.path);
  }
}
function rowMap(rows) { return new Map(rows.map(row => [row.path, canonical(row)])); }
function diffRows(beforeRows, afterRows) {
  const before = rowMap(beforeRows), after = rowMap(afterRows);
  const names = [...new Set([...before.keys(), ...after.keys()])].sort();
  return names.filter(name => before.get(name) !== after.get(name)).map(name => ({
    path: name,
    change: before.has(name) ? (after.has(name) ? 'changed' : 'removed') : 'added',
    before: before.get(name) || null,
    after: after.get(name) || null,
  }));
}
function runGitAt(cwd, args) {
  const env = { PATH: '/usr/bin:/bin', GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_OPTIONAL_LOCKS: '0', GIT_NO_LAZY_FETCH: '1', GIT_NO_REPLACE_OBJECTS: '1', GIT_TERMINAL_PROMPT: '0' };
  const result = spawnSync(GIT, ['-C', cwd, ...args], { encoding: 'utf8', env, timeout: 10000 });
  if (result.error || result.status !== 0) throw new Error('read-only Git query failed: ' + args.join(' '));
  return result.stdout;
}
function candidateSnapshot() {
  const head = runGitAt(CANDIDATE, ['rev-parse', 'HEAD']).trim();
  const tree = runGitAt(CANDIDATE, ['rev-parse', 'HEAD^{tree}']).trim();
  const status = runGitAt(CANDIDATE, ['status', '--porcelain=v1', '--untracked-files=all']);
  const indexPathRaw = runGitAt(CANDIDATE, ['rev-parse', '--git-path', 'index']).trim();
  const indexPath = path.isAbsolute(indexPathRaw) ? indexPathRaw : path.resolve(CANDIDATE, indexPathRaw);
  return {
    root: CANDIDATE, head, tree, status,
    index_path: indexPath, index_sha256: hashFile(indexPath),
    cargo_lock_sha256: hashFile(LOCK), config_sha256: hashFile(CONFIG),
  };
}
function verifyPreviousR3b() {
  if (hashFile(PRIOR_RESULT) !== EXPECTED_PRIOR_RESULT_SHA || hashFile(PRIOR_STDERR) !== EXPECTED_PRIOR_STDERR_SHA) throw new Error('R3b failure evidence hash mismatch');
  const result = JSON.parse(fs.readFileSync(PRIOR_RESULT, 'utf8'));
  if (result.plan_sha256 !== 'a82e3df3afe1c27a2c484f1cb8ae8dd7a75ed3f72d72dda1046d6886adf7e646' ||
      result.runner_sha256 !== 'c087adef069314fe4682092d06d3aede81f40e36a8afa59bc7b680a81f85efa7' ||
      result.status !== 'FAILED_DIAGNOSTIC_RUN' || result.cache_seed?.copy_verified !== true ||
      result.cache_seed?.shared_inodes !== 0 || result.cache_seed?.copied_tree?.tree_sha256 !== EXPECTED_CACHE.tree_sha256 ||
      result.commands?.[1]?.exit_code !== 1 || result.commands?.[1]?.stderr?.sha256 !== EXPECTED_PRIOR_STDERR_SHA ||
      !fs.readFileSync(PRIOR_STDERR, 'utf8').includes('no matching package named `reqwest` found') || result.output?.present !== false) {
    throw new Error('R3b failure and verified private-cache evidence do not match the expected missing-index condition');
  }
  const sums = path.join(PRIOR_ROOT, 'SHA256SUMS');
  if (hashFile(sums) !== 'cccc7f436ac7c161bc28fccb83e38bc43f66ee5cce5531451031e81669b2b718') throw new Error('R3b packet sums hash mismatch');
  return result;
}
function cacheRootIdentity() {
  noSymlinkComponents(CACHE_HOME);
  const rootStat = fs.lstatSync(CACHE_HOME);
  const mode = (rootStat.mode & 0o777).toString(8).padStart(3, '0');
  if (!rootStat.isDirectory() || rootStat.isSymbolicLink() || mode !== EXPECTED_CACHE.root_mode) throw new Error('mutable cache root is not the pinned physical directory or mode');
  return { type: 'directory', symlink: false, mode, dev: rootStat.dev, ino: rootStat.ino };
}
function inspectCacheRoot() {
  let componentsSafe = true, componentError = null;
  try { noSymlinkComponents(CACHE_HOME); } catch (error) { componentsSafe = false; componentError = String(error); }
  try {
    const stat = fs.lstatSync(CACHE_HOME);
    return { exists: true, type: stat.isDirectory() ? 'directory' : stat.isFile() ? 'file' : stat.isSymbolicLink() ? 'symlink' : 'other', symlink: stat.isSymbolicLink(), physical_directory: stat.isDirectory() && !stat.isSymbolicLink(), components_no_symlink: componentsSafe, component_error: componentError, mode: (stat.mode & 0o777).toString(8).padStart(3, '0'), dev: stat.dev, ino: stat.ino };
  } catch (error) {
    return { exists: false, type: 'missing-or-unreadable', symlink: false, physical_directory: false, components_no_symlink: componentsSafe, component_error: componentError, lstat_error: String(error) };
  }
}
function verifyMutableBookkeeping(rows, preflight = false) {
  const byPath = new Map(rows.map(row => [row.path, row]));
  const observed = {};
  for (const [name, expected] of Object.entries(MUTABLE_CARGO_BOOKKEEPING)) {
    const row = byPath.get(name);
    if (!row || row.type !== 'file' || row.mode !== expected.mode || row.bytes > MUTABLE_CARGO_BOOKKEEPING_MAX_BYTES) {
      throw new Error('Cargo bookkeeping path missing, non-regular, wrong mode, or over byte cap: ' + name);
    }
    if (preflight && (row.bytes !== expected.bytes || row.sha256 !== expected.sha256)) throw new Error('Cargo bookkeeping preimage mismatch: ' + name);
    observed[name] = row;
  }
  return observed;
}
function verifyCache(rows = null, preflight = false) {
  const root = cacheRootIdentity();
  const actualRows = rows || walkTree(CACHE_HOME);
  verifySymlinks(CACHE_HOME, actualRows);
  const mutableBookkeeping = verifyMutableBookkeeping(actualRows, preflight);
  const summary = treeSummary(actualRows);
  return { summary: { ...summary, root_mode: root.mode }, root, mutable_bookkeeping: mutableBookkeeping, rows: actualRows };
}
function verifyLockSourceKinds() {
  const text = fs.readFileSync(LOCK, 'utf8');
  const sources = [...text.matchAll(/^source = "([^"]+)"$/gm)].map(match => match[1]);
  const counts = new Map();
  for (const source of sources) counts.set(source, (counts.get(source) || 0) + 1);
  const registry = [...counts].filter(([source]) => source.startsWith('registry+'));
  const git = [...counts].filter(([source]) => source.startsWith('git+'));
  if (registry.length !== 1 || registry[0][0] !== 'registry+https://github.com/rust-lang/crates.io-index' || registry[0][1] !== 433 ||
      git.length !== 1 || git[0][0] !== 'git+https://github.com/tailrocks/tuiscotti?rev=a47c9aaefb34e4c00026f99d8a8dd7ee5916b274#a47c9aaefb34e4c00026f99d8a8dd7ee5916b274' || git[0][1] !== 5) {
    throw new Error('Cargo.lock source kinds/counts differ from the reviewed crates.io plus cached tuiscotti scope');
  }
  return { registry_source: registry[0][0], registry_rows: registry[0][1], git_source: git[0][0], git_rows: git[0][1] };
}
function verifyGitCheckout() {
  noSymlinkComponents(GIT_SOURCE);
  const head = runGitAt(GIT_SOURCE, ['rev-parse', 'HEAD']).trim();
  if (head !== 'a47c9aaefb34e4c00026f99d8a8dd7ee5916b274') throw new Error('cached tuiscotti checkout revision mismatch');
  return { path: GIT_SOURCE, head, git_tree: runGitAt(GIT_SOURCE, ['rev-parse', 'HEAD^{tree}']).trim() };
}
function verifyInputsAndTools() {
  if (hashFile(SOURCE_INVENTORY) !== '4b3bb9389d0277167632a33a1e24c2a9cdf6e9583012c441f6bba68e4ed6e800' || hashFile(SOURCE_MANIFEST) !== 'fa3d4662cbdae5c088c210352b680ef10f81f2f773b54f9d217291bb0160e62e') throw new Error('bound Velnor source context changed');
  if (process.execPath !== NODE || process.version !== 'v26.11.0') throw new Error('Node runtime mismatch');
  const tools = [
    [NODE, 'f75a79e4c96954709a130427e452d6720b35c90318ca8293727a4c6376198453', true],
    [CARGO, '6e17e865f3a20dd55a1d212f849f58b77124179f0de7c52973096d84ba34118d', true],
    [RUSTC, '766eda9d8f53afd6fc7f27b3cd2e444dd22afacb5afa710a5625fc8e45b8c941', true],
    [RUSTDOC, '8ce1ae78157758d4d75eee1355ed333f27e903ab6b0904e09d326009b333c2ee', true],
    [GIT, '34129c71a01a74f7f3b2443521519b2e5447553fa187f5fcafaaf8c42cc192e2', true],
    [GIT_GUARD, '638ef221a35c44979eb017a96cde44c2cb973f3fbe980a65f48cdf8ca5e9e4ab', true],
  ];
  return tools.map(([file, expected, executable]) => {
    noSymlinkComponents(file);
    const stat = fs.lstatSync(file);
    if (!stat.isFile() || stat.isSymbolicLink() || hashFile(file) !== expected || (executable && (stat.mode & 0o111) === 0)) throw new Error('tool identity mismatch: ' + file);
    return { path: file, sha256: expected, mode: (stat.mode & 0o777).toString(8).padStart(3, '0') };
  });
}
function verifyPlan() {
  if (hashFile(PLAN) !== PLAN_SHA) throw new Error('metadata plan hash mismatch');
  const plan = JSON.parse(fs.readFileSync(PLAN, 'utf8'));
  const expectedBookkeepingPreimages = Object.fromEntries(Object.entries(MUTABLE_CARGO_BOOKKEEPING).map(([name, row]) => [name, { type: 'file', ...row }]));
  if (hashFile(R1_REVIEW) !== R1_REVIEW_SHA ||
      plan.predecessor_review?.review_receipt_path !== R1_REVIEW || plan.predecessor_review?.review_receipt_sha256 !== R1_REVIEW_SHA ||
      plan.predecessor_review?.plan_sha256 !== 'ea71ec8dd3499c6e08eb3621bf0c34e6a6e9d015055a1ae16a6250227a793d1a' ||
      plan.predecessor_review?.runner_sha256 !== '31c4755f8029254df18dc04f7b94e747330bf983f362e48615863c0c342fd6a1' ||
      plan.schema !== 'termrock.vis11.diagnostic-generator-registry-index-fill-plan/v2' ||
      plan.candidate?.root !== CANDIDATE || plan.candidate?.head !== EXPECTED_CANDIDATE.head || plan.candidate?.tree !== EXPECTED_CANDIDATE.tree ||
      plan.candidate?.index_path !== EXPECTED_CANDIDATE.index_path || plan.candidate?.index_sha256 !== EXPECTED_CANDIDATE.index_sha256 ||
      plan.candidate?.cargo_lock?.sha256 !== EXPECTED_CANDIDATE.lock_sha256 || plan.candidate?.generator_config_overlay?.sha256 !== EXPECTED_CANDIDATE.config_sha256 ||
      canonical(plan.candidate?.cargo_config_search_directories) !== canonical(CARGO_CONFIG_SEARCH_DIRECTORIES) ||
      plan.working_cache?.path !== CACHE_HOME || plan.working_cache?.pre_metadata_tree_sha256 !== EXPECTED_CACHE.tree_sha256 ||
      plan.working_cache?.root_mode !== EXPECTED_CACHE.root_mode ||
      canonical(plan.working_cache?.mutable_bookkeeping_paths) !== canonical(Object.keys(MUTABLE_CARGO_BOOKKEEPING)) ||
      plan.working_cache?.mutable_bookkeeping_max_bytes_per_file !== MUTABLE_CARGO_BOOKKEEPING_MAX_BYTES ||
      canonical(plan.working_cache?.mutable_bookkeeping_preimages) !== canonical(expectedBookkeepingPreimages) ||
      plan.working_cache?.root_postcondition !== 'After Cargo closes: lstat the cache root; require a physical directory, not a symlink, mode 0755, with dev and ino equal to the pre-command lstat values.' ||
      plan.tools?.git_network_guard?.sha256 !== '638ef221a35c44979eb017a96cde44c2cb973f3fbe980a65f48cdf8ca5e9e4ab' ||
      plan.command?.executable !== CARGO || plan.command?.timeout_ms !== TIMEOUT.command || plan.command?.retry_limit !== 1 ||
      canonical(plan.command?.argv) !== canonical(['metadata', '--manifest-path', path.join(CANDIDATE, 'Cargo.toml'), '--locked', '--no-deps', '--format-version', '1']) ||
      plan.environment?.cargo_net_offline !== 'false' || !plan.environment?.git_network_guard?.includes('symlink to the pinned tools/git wrapper') ||
      plan.authorization?.operator !== 'Root only after independent review') throw new Error('metadata plan command or scope mismatch');
  return plan;
}
function verifyStatic() {
  noSymlinkComponents(ROOT); noSymlinkComponents(CANDIDATE); noSymlinkComponents(SOURCE); noSymlinkComponents(CACHE_HOME);
  const plan = verifyPlan();
  const tools = verifyInputsAndTools();
  const previous = verifyPreviousR3b();
  const candidate = candidateSnapshot();
  const expectedCandidate = { root: CANDIDATE, head: EXPECTED_CANDIDATE.head, tree: EXPECTED_CANDIDATE.tree, status: EXPECTED_CANDIDATE.status, index_path: EXPECTED_CANDIDATE.index_path, index_sha256: EXPECTED_CANDIDATE.index_sha256, cargo_lock_sha256: EXPECTED_CANDIDATE.lock_sha256, config_sha256: EXPECTED_CANDIDATE.config_sha256 };
  if (canonical(candidate) !== canonical(expectedCandidate)) throw new Error('candidate lock/source/index preimage mismatch');
  const lockSources = verifyLockSourceKinds();
  const cacheState = verifyCache(null, true);
  const cache = cacheState.summary;
  if (canonical(cache) !== canonical({ inventory_schema: 'runner-walkTree-canonical/v1', ...EXPECTED_CACHE })) throw new Error('mutable Cargo cache changed since R3b failure');
  if (exists(REQWEST_INDEX)) throw new Error('expected reqwest sparse-index cache entry is no longer absent');
  const git = verifyGitCheckout();
  const cargoConfigPaths = [...CARGO_CONFIG_SEARCH_DIRECTORIES.flatMap(directory => ['config', 'config.toml'].map(name => path.join(directory, '.cargo', name))), path.join(CACHE_HOME, 'config'), path.join(CACHE_HOME, 'config.toml')];
  if (cargoConfigPaths.some(exists)) throw new Error('unreviewed Cargo config path is present');
  if (exists(RUN)) throw new Error('metadata attempt output root must be absent');
  if (exists(path.join(PRIOR_ROOT, 'run-root/preview'))) throw new Error('prior diagnostic preview unexpectedly exists');
  if (inside(RUN, CANDIDATE) || inside(CANDIDATE, RUN) || inside(RUN, CACHE_HOME) || inside(CACHE_HOME, RUN) ||
      inside(RUN, SOURCE) || inside(SOURCE, RUN) || inside(RUN, path.dirname(CACHE_HOME)) || inside(path.dirname(CACHE_HOME), RUN)) throw new Error('metadata output root overlaps candidate, source, or mutable cache');
  return { plan, tools, previous_failure_sha256: EXPECTED_PRIOR_RESULT_SHA, candidate, cache_before: cache, cache_root_identity: cacheState.root, mutable_bookkeeping_before: cacheState.mutable_bookkeeping, cache_rows: cacheState.rows, cached_git_source: git, lock_source_kinds: lockSources, cargo_config_paths_checked: cargoConfigPaths, reqwest_index_absent: true, output_root_absent: RUN };
}
function mkdirPrivate(file) { fs.mkdirSync(file, { recursive: true, mode: 0o700 }); fs.chmodSync(file, 0o700); }
function makeOutputLayout() {
  for (const dir of [RUN, EXEC, PRIVATE, BIN, HOME, TMP, XDG, TARGET, RUSTUP_HOME]) mkdirPrivate(dir);
  const gitPath = path.join(BIN, 'git');
  if (exists(gitPath)) throw new Error('private Git guard path already exists');
  fs.symlinkSync(GIT_GUARD, gitPath, 'file');
  if (fs.realpathSync(gitPath) !== GIT_GUARD) throw new Error('private Git guard link target mismatch');
}
function childEnvironment() {
  return {
    PATH: [BIN, path.join(RUST, 'bin'), '/usr/bin', '/bin'].join(path.delimiter),
    HOME, TMPDIR: TMP, TMP, TEMP: TMP,
    XDG_CONFIG_HOME: path.join(XDG, 'config'), XDG_CACHE_HOME: path.join(XDG, 'cache'), XDG_STATE_HOME: path.join(XDG, 'state'),
    CARGO_HOME: CACHE_HOME, CARGO_TARGET_DIR: TARGET, CARGO_NET_OFFLINE: 'false',
    CARGO_NET_GIT_FETCH_WITH_CLI: 'true', CARGO_REGISTRIES_CRATES_IO_PROTOCOL: 'sparse',
    CARGO_NET_RETRY: '1', CARGO_HTTP_TIMEOUT: '30', CARGO_TERM_COLOR: 'never',
    CARGO, RUSTC, RUSTDOC, RUSTUP_HOME, RUSTUP_TOOLCHAIN: '1.98.1-aarch64-apple-darwin',
    GIT_CONFIG_GLOBAL: '/dev/null', GIT_CONFIG_NOSYSTEM: '1', GIT_OPTIONAL_LOCKS: '0',
    GIT_NO_LAZY_FETCH: '1', GIT_NO_REPLACE_OBJECTS: '1', GIT_TERMINAL_PROMPT: '0',
    GIT_NETWORK_BLOCK_LOG: GIT_BLOCK_LOG,
    LC_ALL: 'C', LANG: 'C', TZ: 'UTC', SHELL: '/bin/bash', NO_COLOR: '1', TERM: 'dumb',
  };
}
function writeLog(name, chunks) {
  const bytes = Buffer.concat(chunks);
  const file = path.join(EXEC, name);
  fs.writeFileSync(file, bytes, { flag: 'wx', mode: 0o600 });
  return { path: file, bytes: bytes.length, sha256: hashBytes(bytes) };
}
function runChild(env) {
  return new Promise(resolve => {
    const stdout = [], stderr = [];
    let total = 0, timeoutKind = null, termTimer = null, closeTimer = null, spawnError = null;
    const startedAt = new Date().toISOString();
    const start = Date.now();
    const args = ['metadata', '--manifest-path', path.join(CANDIDATE, 'Cargo.toml'), '--locked', '--no-deps', '--format-version', '1'];
    const child = spawn(CARGO, args, { cwd: CANDIDATE, env, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
    const terminate = kind => {
      if (timeoutKind) return;
      timeoutKind = kind;
      if (child.pid) {
        try { process.kill(-child.pid, 'SIGTERM'); } catch {}
        termTimer = setTimeout(() => { try { process.kill(-child.pid, 'SIGKILL'); } catch {} }, TIMEOUT.term);
      }
      closeTimer = setTimeout(() => { try { child.kill('SIGKILL'); } catch {} }, TIMEOUT.close);
    };
    const collect = (buffer, chunk) => {
      total += chunk.length;
      if (total <= TIMEOUT.output) buffer.push(chunk);
      else terminate('output_cap');
    };
    child.stdout.on('data', chunk => collect(stdout, chunk));
    child.stderr.on('data', chunk => collect(stderr, chunk));
    child.on('error', error => { spawnError = String(error); });
    const deadline = setTimeout(() => terminate('timeout'), TIMEOUT.command);
    child.on('close', (exitCode, signal) => {
      clearTimeout(deadline);
      if (termTimer) clearTimeout(termTimer);
      if (closeTimer) clearTimeout(closeTimer);
      const out = writeLog('cargo-metadata.stdout.bin', stdout);
      const err = writeLog('cargo-metadata.stderr.bin', stderr);
      resolve({ id: 'cargo-metadata', executable: CARGO, argv: args, cwd: CANDIDATE, started_at: startedAt, elapsed_ms: Date.now() - start,
        exit_code: exitCode, signal, timeout_kind: timeoutKind, spawn_error: spawnError, captured_bytes: total,
        output_cap_bytes: TIMEOUT.output, stdout: out, stderr: err, closed: true,
        success: exitCode === 0 && !signal && !timeoutKind && !spawnError && total <= TIMEOUT.output });
    });
  });
}
function atomicJson(file, value) {
  const bytes = Buffer.from(JSON.stringify(value, null, 2) + '\n');
  const temporary = file + '.tmp';
  const fd = fs.openSync(temporary, 'wx', 0o600);
  try { fs.writeFileSync(fd, bytes); fs.fsyncSync(fd); } finally { fs.closeSync(fd); }
  fs.renameSync(temporary, file);
  fs.chmodSync(file, 0o600);
  return { path: file, bytes: bytes.length, sha256: hashBytes(bytes) };
}
function metadataSummary(command) {
  if (!command.success) return { valid_json: false, package_count: null, workspace_member_count: null };
  const stdout = fs.readFileSync(command.stdout.path);
  const value = JSON.parse(stdout.toString('utf8'));
  if (!value || !Array.isArray(value.packages) || !Array.isArray(value.workspace_members) || value.packages.length === 0) throw new Error('Cargo metadata JSON lacks nonempty package/workspace arrays');
  return { valid_json: true, package_count: value.packages.length, workspace_member_count: value.workspace_members.length, resolve_present: value.resolve !== null && value.resolve !== undefined };
}
function allowedCargoCachePath(change) {
  return change.path === 'registry/index' || change.path.startsWith('registry/index/') ||
    Object.prototype.hasOwnProperty.call(MUTABLE_CARGO_BOOKKEEPING, change.path);
}
function changedOutsideAllowedCargoCache(changes) { return changes.filter(change => !allowedCargoCachePath(change)); }
async function execute(initial) {
  makeOutputLayout();
  const gitLogFd = fs.openSync(GIT_BLOCK_LOG, 'wx', 0o600);
  fs.closeSync(gitLogFd);
  const beforeCandidate = candidateSnapshot();
  const beforeRows = initial.cache_rows;
  const beforeCache = { ...treeSummary(beforeRows), root_mode: initial.cache_root_identity.mode };
  const beforeTarget = walkTree(TARGET);
  executionStarted = true;
  const command = await runChild(childEnvironment());
  const afterCandidate = candidateSnapshot();
  const afterRootObservation = inspectCacheRoot();
  let afterRows = null, afterCache = null, afterBookkeeping = null, cachePostcheckError = null;
  if (afterRootObservation.physical_directory && afterRootObservation.components_no_symlink) {
    try {
      afterRows = walkTree(CACHE_HOME);
      verifySymlinks(CACHE_HOME, afterRows);
      afterBookkeeping = verifyMutableBookkeeping(afterRows, false);
      afterCache = { ...treeSummary(afterRows), root_mode: afterRootObservation.mode };
    } catch (error) { cachePostcheckError = String(error); }
  } else cachePostcheckError = afterRootObservation.component_error || afterRootObservation.lstat_error || 'Cargo home root is not a physical directory without symlink components';
  const rootAfterIdentity = afterRootObservation.physical_directory && afterRootObservation.components_no_symlink ? { type: 'directory', symlink: false, mode: afterRootObservation.mode, dev: afterRootObservation.dev, ino: afterRootObservation.ino } : null;
  const cacheRootUnchanged = !!rootAfterIdentity && canonical(initial.cache_root_identity) === canonical(rootAfterIdentity);
  const cachePostcheckPassed = !!afterRows && !!afterCache && !!afterBookkeeping && cachePostcheckError === null;
  const targetAfter = walkTree(TARGET);
  const changes = afterRows ? diffRows(beforeRows, afterRows) : null;
  const outsideAllowedCache = changes ? changedOutsideAllowedCargoCache(changes) : null;
  const removedIndexPaths = changes ? changes.filter(change => (change.path === 'registry/index' || change.path.startsWith('registry/index/')) && change.change === 'removed') : null;
  const invalidBookkeepingChanges = changes ? changes.filter(change => Object.prototype.hasOwnProperty.call(MUTABLE_CARGO_BOOKKEEPING, change.path) && change.change !== 'changed') : null;
  const reqwestRel = 'registry/index/index.crates.io-1949cf8c6b5b557f/.cache/re/qw/reqwest';
  const reqwestRow = afterRows?.find(row => row.path === reqwestRel && row.type === 'file');
  const reqwest = reqwestRow ? { path: path.join(CACHE_HOME, reqwestRel), bytes: reqwestRow.bytes, sha256: reqwestRow.sha256 } : null;
  let metadata;
  let metadataError = null;
  try { metadata = metadataSummary(command); } catch (error) { metadata = { valid_json: false, package_count: null, workspace_member_count: null }; metadataError = String(error); }
  const gitNetworkBlockLog = fs.readFileSync(GIT_BLOCK_LOG);
  const cacheDeltaWithinAllowedScope = !!changes && outsideAllowedCache.length === 0 && removedIndexPaths.length === 0 && invalidBookkeepingChanges.length === 0;
  const candidateUnchanged = canonical(beforeCandidate) === canonical(afterCandidate) && afterCandidate.cargo_lock_sha256 === EXPECTED_CANDIDATE.lock_sha256;
  const targetUnchanged = canonical(beforeTarget) === canonical(targetAfter);
  const previewStillAbsent = !exists(path.join(PRIOR_ROOT, 'run-root/preview'));
  const passed = command.success && metadata?.valid_json === true && candidateUnchanged && cacheRootUnchanged && cachePostcheckPassed && cacheDeltaWithinAllowedScope && reqwest !== null && targetUnchanged && previewStillAbsent && gitNetworkBlockLog.length === 0;
  const result = {
    schema: 'termrock.vis11.diagnostic-generator-registry-index-fill-result/v2',
    status: passed ? 'METADATA_INDEX_FILL_SUCCEEDED_NOT_QUALIFIED' : 'METADATA_INDEX_FILL_FAILED_OR_INCOMPLETE',
    execution: 'COMPLETED', qualification: 'NOT_QUALIFIED', plan_sha256: PLAN_SHA, runner_sha256: hashFile(__filename),
    previous_r3b_failure_sha256: EXPECTED_PRIOR_RESULT_SHA,
    candidate: { before: beforeCandidate, after: afterCandidate, unchanged: candidateUnchanged },
    cache_home: { path: CACHE_HOME, before: beforeCache, after: afterCache, root_before: initial.cache_root_identity, root_after: afterRootObservation, root_identity_and_mode_unchanged: cacheRootUnchanged, postcheck_passed: cachePostcheckPassed, postcheck_error: cachePostcheckError, mutable_bookkeeping_before: initial.mutable_bookkeeping_before, mutable_bookkeeping_after: afterBookkeeping, mutable_bookkeeping_max_bytes_per_file: MUTABLE_CARGO_BOOKKEEPING_MAX_BYTES, changed_path_count: changes?.length ?? null, changed_paths: changes, changed_outside_allowed_cache_scope: outsideAllowedCache, removed_registry_index_paths: removedIndexPaths, invalid_bookkeeping_path_changes: invalidBookkeepingChanges, changes_confined_to_index_and_exact_bookkeeping_files: cacheDeltaWithinAllowedScope, reqwest_index_after: reqwest },
    cargo_target: { before_entries: beforeTarget.length, after_entries: targetAfter.length, unchanged: targetUnchanged },
    git_network_guard: { path: GIT_GUARD, sha256: hashFile(GIT_GUARD), blocked_operation_log: { path: GIT_BLOCK_LOG, bytes: gitNetworkBlockLog.length, sha256: hashBytes(gitNetworkBlockLog) }, no_git_network_operation_observed: gitNetworkBlockLog.length === 0 },
    command, metadata_summary: metadata, metadata_parse_error: metadataError,
    limits: ['Only one locked Cargo metadata --no-deps command was authorized.', 'No build, check, test, nextest, Velnor CLI, generation, or release action ran.', 'The existing isolated CARGO_HOME is mutable from this attempt onward; R3b failed-run evidence is preserved separately.', 'Success would mean only that metadata resolved and the reqwest index was populated; it would not establish complete lockfile cache closure or generator qualification.']
  };
  const receipt = atomicJson(path.join(EXEC, 'result.json'), result);
  process.stdout.write(JSON.stringify({ status: result.status, execution: result.execution, qualification: result.qualification, receipt }, null, 2) + '\n');
  if (!passed) process.exitCode = 1;
}
function preflight(inputs) {
  const { cache_rows, ...summaryInputs } = inputs;
  process.stdout.write(JSON.stringify({ status: 'PREFLIGHT_PASS_NOT_RUN', execution: 'NOT_RUN', qualification: 'NOT_QUALIFIED', plan_sha256: PLAN_SHA, runner_sha256: hashFile(__filename),
    inputs: summaryInputs, command: { executable: CARGO, argv: ['metadata', '--manifest-path', path.join(CANDIDATE, 'Cargo.toml'), '--locked', '--no-deps', '--format-version', '1'], network: 'crates.io sparse index only, with Git network subcommands blocked' },
    output_root_absent: RUN, cargo_child_spawned: false, cache_mutated: false }, null, 2) + '\n');
}
(async () => {
  try {
    const mode = process.argv[2];
    if (process.argv.length !== 3 || !['--preflight', '--execute'].includes(mode)) throw new Error('usage: runner.cjs --preflight|--execute');
    const state = verifyStatic();
    if (mode === '--preflight') preflight(state);
    else await execute(state);
  } catch (error) {
    process.stderr.write(JSON.stringify({ status: 'PREFLIGHT_OR_SETUP_FAILED', execution: executionStarted ? 'FAILED_OR_INCOMPLETE' : 'NOT_RUN', execution_started: executionStarted, qualification: 'NOT_QUALIFIED', error: String(error) }) + '\n');
    process.exitCode = 1;
  }
})();
