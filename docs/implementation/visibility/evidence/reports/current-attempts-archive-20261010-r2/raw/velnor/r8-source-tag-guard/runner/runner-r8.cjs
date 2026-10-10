'use strict';

const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn } = require('node:child_process');

const TASK_ROOT = '/private/tmp/termrock-vis11-tool-fixture-gate-r8-source-tag-guard-luna-20261010';
const EVIDENCE = path.join(TASK_ROOT, 'evidence');
const RUN_ROOT = path.join(TASK_ROOT, 'run-root');
const PACKET_PATH = path.join(EVIDENCE, 'gate-packet-r8.json');
const PACKET_SHA = 'e94e4a9125bb7581be92078184391592f2827a84683afc65a6fe8e1b8e25ce86';
const SOURCE = '/private/tmp/termrock-vis11-tool-fixture-admission-r5-source-tag-guard-luna-20261010/source';
const CARGO_HOME = '/private/tmp/termrock-vis11-metadata-cache-delta-r6-20261009/cargo-home';
const RUSTUP_HOME = '/Users/donbeave/.rustup';
const RUST_BIN = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin';
const RUST_TOOLCHAIN = '1.98.1-aarch64-apple-darwin';
const RUSTUP_TOOLCHAIN = RUST_TOOLCHAIN;
const CARGO = path.join(RUST_BIN, 'cargo');
const RUSTC = path.join(RUST_BIN, 'rustc');
const RUSTDOC = path.join(RUST_BIN, 'rustdoc');
const NEXTEST = '/Users/donbeave/.local/share/mise/installs/aqua-nextest-rs-nextest-cargo-nextest/0.9.146/cargo-nextest';
const NODE = '/opt/homebrew/Cellar/node/26.11.0_1/bin/node';
const MISE = '/Users/donbeave/Projects/tailrocks/.termrock-visibility-runtime-20261009/mise-2026.9.18-macos-arm64/mise-v2026.9.18-macos-arm64';
const OUTPUT = path.join(RUN_ROOT, 'execution');
const TARGET = path.join(RUN_ROOT, 'target');
const LIMITS = { deadline_ms: 1200 * 1000, term_grace_ms: 15000, close_grace_ms: 10000, output_bytes: 4 * 1024 * 1024 };
const SELECTED_IDS = [
  'impl_schema2_routing::schema2_workflows_match_expected_bytes'
];

function shaBytes(bytes) { return crypto.createHash('sha256').update(bytes).digest('hex'); }
function sha(file) { return shaBytes(fs.readFileSync(file)); }
function readJson(file) { return JSON.parse(fs.readFileSync(file, 'utf8')); }
function canonical(value) {
  if (Array.isArray(value)) return '[' + value.map(canonical).join(',') + ']';
  if (value && typeof value === 'object') return '{' + Object.keys(value).sort().map(k => JSON.stringify(k) + ':' + canonical(value[k])).join(',') + '}';
  return JSON.stringify(value);
}
function exists(p) { try { fs.lstatSync(p); return true; } catch (e) { if (e && e.code === 'ENOENT') return false; throw e; } }
function pathInside(child, parent) {
  const rel = path.relative(parent, child);
  return rel === '' || (rel !== '..' && !rel.startsWith('..' + path.sep) && !path.isAbsolute(rel));
}
function noSymlinkComponents(p) {
  const absolute = path.resolve(p);
  const root = path.parse(absolute).root;
  let current = root;
  for (const part of absolute.slice(root.length).split(path.sep).filter(Boolean)) {
    current = path.join(current, part);
    const st = fs.lstatSync(current);
    if (st.isSymbolicLink()) throw new Error('symlink path component: ' + current);
  }
}
function physicalDir(p, mode) {
  noSymlinkComponents(p);
  const st = fs.lstatSync(p);
  if (!st.isDirectory() || st.isSymbolicLink()) throw new Error('not a physical directory: ' + p);
  const real = fs.realpathSync(p);
  if (real !== path.resolve(p)) throw new Error('directory resolves through another path: ' + p);
  if (mode !== undefined && (st.mode & 0o777) !== mode) throw new Error('directory mode mismatch: ' + p);
  return real;
}
function regular(file, expectedSha, expectedBytes, expectedMode, executable = false) {
  noSymlinkComponents(path.dirname(file));
  const st = fs.lstatSync(file);
  if (!st.isFile() || st.isSymbolicLink()) throw new Error('expected regular non-symlink file: ' + file);
  const digest = sha(file);
  if (expectedSha && digest !== expectedSha) throw new Error('SHA-256 mismatch: ' + file);
  if (expectedBytes !== undefined && st.size !== expectedBytes) throw new Error('byte count mismatch: ' + file);
  if (expectedMode && (st.mode & 0o777).toString(8).padStart(3, '0') !== expectedMode) throw new Error('mode mismatch: ' + file);
  if (executable && (st.mode & 0o111) === 0) throw new Error('expected executable: ' + file);
  return { path: file, bytes: st.size, sha256: digest, mode: (st.mode & 0o777).toString(8).padStart(3, '0') };
}
function safeRel(value) {
  return typeof value === 'string' && value.length > 0 && !value.startsWith('/') && !value.includes('\\') && !/[\0\t\r\n]/.test(value) && value.split('/').every(part => part && part !== '.' && part !== '..');
}
function walkTree(root, rel = '') {
  const out = [];
  for (const name of fs.readdirSync(path.join(root, rel)).sort()) {
    const abs = path.join(root, rel, name), itemPath = rel ? `${rel}/${name}` : name;
    const st = fs.lstatSync(abs), mode = (st.mode & 0o777).toString(8).padStart(3, '0');
    if (st.isSymbolicLink()) out.push({ path: itemPath, type: 'symlink', mode, target: fs.readlinkSync(abs) });
    else if (st.isDirectory()) {
      out.push({ path: itemPath, type: 'directory', mode });
      out.push(...walkTree(root, itemPath));
    } else if (st.isFile()) {
      const bytes = fs.readFileSync(abs);
      out.push({ path: itemPath, type: 'file', mode, bytes: bytes.length, sha256: shaBytes(bytes) });
    } else throw new Error('special source/cache path: ' + itemPath);
  }
  return out;
}
function verifySource(packet) {
  const sourceReal = physicalDir(SOURCE, parseInt(packet.source.root_mode, 8));
  const pin = packet.source.full_tree;
  regular(pin.path, pin.sha256, pin.bytes, pin.mode);
  const tree = readJson(pin.path);
  if (tree.source_root !== SOURCE || tree.entry_count !== packet.source.entry_count || tree.entries.length !== packet.source.entry_count || canonical(tree.counts) !== canonical(packet.source.counts)) throw new Error('source tree inventory identity/count mismatch');
  const expected = new Map();
  for (const row of tree.entries) {
    if (!safeRel(row.path) || expected.has(row.path) || !['file', 'directory', 'symlink'].includes(row.type) || !/^[0-7]{3}$/.test(row.mode)) throw new Error('bad or duplicate inventory row: ' + row.path);
    if (row.type === 'file' && (!/^[0-9a-f]{64}$/.test(row.sha256 || '') || !Number.isInteger(row.bytes) || row.bytes < 0)) throw new Error('bad file inventory row: ' + row.path);
    if (row.type === 'symlink' && (typeof row.target !== 'string' || row.target.startsWith('/') || row.target.includes('\\') || /[\0\t\r\n]/.test(row.target))) throw new Error('unsafe symlink row: ' + row.path);
    expected.set(row.path, row);
  }
  const actual = walkTree(SOURCE);
  if (canonical(actual) !== canonical(tree.entries)) throw new Error('source path/mode/content closure mismatch');
  for (const row of tree.entries.filter(item => item.type === 'symlink')) {
    const link = path.join(SOURCE, row.path), targetPath = path.resolve(path.dirname(link), row.target), targetReal = fs.realpathSync(targetPath);
    const targetRow = expected.get(path.relative(SOURCE, targetPath).split(path.sep).join('/'));
    if (!pathInside(targetReal, sourceReal) || !targetRow || targetRow.type !== 'file' || sha(targetPath) !== targetRow.sha256) throw new Error('source symlink target mismatch: ' + row.path);
  }
  if (exists(path.join(SOURCE, '.git')) || exists(path.join(SOURCE, 'target'))) throw new Error('source includes Git metadata or a build target');
  if (sha(path.join(SOURCE, 'Cargo.lock')) !== packet.source.lock_sha256) throw new Error('Cargo.lock source pin mismatch');
  const mapPin = packet.source.test_source_map;
  regular(mapPin.path, mapPin.sha256, mapPin.bytes, mapPin.mode);
  const testMap = readJson(mapPin.path);
  if (testMap.source_tree_sha256 !== pin.sha256 || testMap.selected_count !== 1 || testMap.tests.length !== 1) throw new Error('test source map identity/count mismatch');
  const selected = packet.source.selected_tests;
  if (canonical(selected.map(t => t.id).sort()) !== canonical(SELECTED_IDS) || canonical(testMap.tests) !== canonical([...selected].sort((a,b) => a.id.localeCompare(b.id)))) throw new Error('selected test IDs differ from source map');
  for (const test of selected) {
    const row = expected.get(test.source_path);
    if (!row || row.type !== 'file' || row.sha256 !== test.source_sha256) throw new Error('test source hash differs from tree: ' + test.id);
    const text = fs.readFileSync(path.join(SOURCE, test.source_path), 'utf8');
    const matches = [...text.matchAll(new RegExp('\\bfn\\s+' + test.function + '\\b', 'g'))];
    if (matches.length !== 1 || text.slice(0, matches[0].index).split('\n').length !== test.line) throw new Error('test function line/source mismatch: ' + test.id);
  }
  return { root: SOURCE, full_tree_sha256: pin.sha256, entries: tree.entry_count, files: tree.counts.file, directories: tree.counts.directory, symlinks: tree.counts.symlink, lock_sha256: packet.source.lock_sha256, selected_test_functions: selected.length, git_identity: 'NOT_ASSERTED_GITLESS_EXTERNAL_SNAPSHOT' };
}
function inventoryCachePath(cargoHome, rel, output) {
  const full = path.join(cargoHome, rel), st = fs.lstatSync(full);
  if (st.isSymbolicLink()) throw new Error('symlink in immutable Cargo cache: ' + rel);
  if (st.isDirectory()) {
    output.push({ path: rel.split(path.sep).join('/'), type: 'directory' });
    for (const name of fs.readdirSync(full).sort()) inventoryCachePath(cargoHome, path.join(rel, name), output);
  } else if (st.isFile()) output.push({ path: rel.split(path.sep).join('/'), type: 'file', size: st.size, sha256: sha(full) });
  else throw new Error('special immutable cache entry: ' + rel);
}
function verifyCache(packet) {
  const runtime = packet.runtime;
  physicalDir(CARGO_HOME);
  if (exists(path.join(CARGO_HOME, 'config')) || exists(path.join(CARGO_HOME, 'config.toml'))) throw new Error('Cargo home contains unpinned config');
  regular(runtime.archive_manifest.path, runtime.archive_manifest.sha256, runtime.archive_manifest.bytes, runtime.archive_manifest.mode);
  regular(runtime.source_index_closure.path, runtime.source_index_closure.sha256, runtime.source_index_closure.bytes, runtime.source_index_closure.mode);
  regular(runtime.source_index_summary_erratum.path, runtime.source_index_summary_erratum.sha256, runtime.source_index_summary_erratum.bytes, runtime.source_index_summary_erratum.mode);
  const archives = readJson(runtime.archive_manifest.path), closure = readJson(runtime.source_index_closure.path), erratum = readJson(runtime.source_index_summary_erratum.path);
  if (archives.archive_count !== 406 || archives.archive_bytes !== 59163074 || archives.archives.length !== 406 || archives.unpacked_source_package_directory_count !== 345 || archives.cargo_home !== CARGO_HOME) throw new Error('archive manifest count/identity mismatch');
  const expectedArchives = new Map(archives.archives.map(row => [row.name, row]));
  if (expectedArchives.size !== 406) throw new Error('duplicate archive names');
  const files = [];
  function walkArchives(dir) {
    for (const name of fs.readdirSync(dir).sort()) {
      const full = path.join(dir, name), st = fs.lstatSync(full);
      if (st.isSymbolicLink()) throw new Error('symlink in registry archive cache');
      if (st.isDirectory()) walkArchives(full);
      else if (st.isFile() && name.endsWith('.crate')) files.push(full);
      else throw new Error('unlisted/special archive cache entry: ' + full);
    }
  }
  walkArchives(path.join(CARGO_HOME, 'registry/cache'));
  let archiveBytes = 0;
  if (files.length !== 406) throw new Error('archive cache entry count mismatch');
  for (const file of files) {
    const name = path.basename(file), pin = expectedArchives.get(name), st = fs.lstatSync(file);
    if (!pin || st.size !== pin.bytes || sha(file) !== pin.sha256) throw new Error('archive differs from pinned manifest: ' + name);
    expectedArchives.delete(name); archiveBytes += st.size;
  }
  if (expectedArchives.size || archiveBytes !== 59163074) throw new Error('archive cache totals mismatch');
  if (closure.record_type !== 'velnor_cargo_registry_consumed_source_index_closure_v1' || closure.cargo_home !== CARGO_HOME || canonical(closure.roots) !== canonical(['registry/src', 'registry/index']) || closure.file_count !== 20133 || closure.directory_count !== 3666 || closure.file_bytes !== 200791635 || closure.entries.length !== 23799 || closure.symlink_count !== 0) throw new Error('source/index closure identity/count mismatch');
  const rawFiles = closure.entries.filter(row => row.type === 'file'), rawDirs = closure.entries.filter(row => row.type === 'directory');
  const derivedBytes = rawFiles.reduce((sum, row) => sum + row.size, 0);
  if (derivedBytes !== 441286079 || erratum.qualification !== 'NOT_QUALIFIED' || erratum.raw_closure.sha256 !== runtime.source_index_closure.sha256 || erratum.derived_from_raw_entries.sum_file_entry_bytes !== derivedBytes) throw new Error('source/index raw versus derived accounting mismatch');
  const expected = new Map(closure.entries.map(row => [row.path, row]));
  if (expected.size !== 23799) throw new Error('duplicate source/index paths');
  const actual = [];
  inventoryCachePath(CARGO_HOME, 'registry/src', actual);
  inventoryCachePath(CARGO_HOME, 'registry/index', actual);
  if (actual.length !== closure.entries.length) throw new Error('source/index entry count mismatch');
  let sourceFiles = 0, sourceDirs = 0, sourceBytes = 0;
  for (const row of actual) {
    const pin = expected.get(row.path);
    if (!pin || canonical(pin) !== canonical(row)) throw new Error('source/index input changed: ' + row.path);
    expected.delete(row.path);
    if (row.type === 'file') { sourceFiles++; sourceBytes += row.size; } else sourceDirs++;
  }
  if (expected.size || sourceFiles !== 20133 || sourceDirs !== 3666 || sourceBytes !== derivedBytes) throw new Error('source/index actual totals mismatch');
  const srcRoot = path.join(CARGO_HOME, 'registry/src'), registries = fs.readdirSync(srcRoot);
  if (registries.length !== 1) throw new Error('unexpected registry source roots');
  const packages = fs.readdirSync(path.join(srcRoot, registries[0]));
  if (packages.length !== 406) throw new Error('extracted source package count mismatch');
  for (const name of packages) {
    const pkg = path.join(srcRoot, registries[0], name), st = fs.lstatSync(pkg);
    if (!st.isDirectory() || st.isSymbolicLink()) throw new Error('invalid source package directory');
    regular(path.join(pkg, '.cargo-ok'), 'afbf9d0f3560b0fd7795e81c42a0a79ee6b6fc67e064f77826aee642cad28d91', 7);
  }
  return { archive_manifest_sha256: runtime.archive_manifest.sha256, archives: files.length, archive_bytes: archiveBytes, source_index_closure_sha256: runtime.source_index_closure.sha256, source_index_files: sourceFiles, source_index_directories: sourceDirs, source_index_actual_file_bytes: sourceBytes, source_index_declared_file_bytes: closure.file_bytes, extracted_package_directories: packages.length, cargo_ok_markers: packages.length, qualification: 'NOT_QUALIFIED' };
}
function snapshotMutableGlobalCache() {
  const file = path.join(CARGO_HOME, '.global-cache');
  try {
    const st = fs.lstatSync(file);
    if (st.isSymbolicLink()) return { path: file, type: 'symlink', target: fs.readlinkSync(file), status: 'observed_separately_mutable_not_offline_input' };
    if (st.isFile()) return { path: file, type: 'file', bytes: st.size, sha256: sha(file), mode: st.mode & 0o777, status: 'observed_separately_mutable_not_offline_input' };
    if (st.isDirectory()) return { path: file, type: 'directory', entries: walkTree(CARGO_HOME, '.global-cache'), status: 'observed_separately_mutable_not_offline_input' };
    return { path: file, type: 'special', status: 'observed_separately_mutable_not_offline_input' };
  } catch (e) { if (e && e.code === 'ENOENT') return { path: file, type: 'absent', status: 'observed_separately_mutable_not_offline_input' }; throw e; }
}
function verifyTools(packet) {
  const files = [];
  for (const key of ['cargo', 'rustc', 'rustdoc', 'nextest', 'node', 'mise']) files.push(regular(packet.tools[key].path, packet.tools[key].sha256, undefined, undefined, true));
  for (const [file, digest] of packet.tools.system_helpers) files.push(regular(file, digest, undefined, undefined, true));
  if (process.execPath !== NODE || process.version !== packet.tools.node.version || sha(process.execPath) !== packet.tools.node.sha256) throw new Error('pinned Node runtime mismatch');
  if (CARGO !== packet.tools.cargo.path || RUSTC !== packet.tools.rustc.path || RUSTDOC !== packet.tools.rustdoc.path || NEXTEST !== packet.tools.nextest.path || MISE !== packet.tools.mise.path) throw new Error('pinned Rust/Mise executable path mismatch');
  return files;
}
function verifyRuntimeSeeds(packet) {
  const rows = packet.runtime.runtime_seed_files;
  if (rows.length !== 13 || new Set(rows.map(row => row.relative_path)).size !== 13) throw new Error('runtime seed file list mismatch');
  return rows.map(row => {
    if (!safeRel(row.relative_path)) throw new Error('unsafe runtime seed path');
    return regular(row.path, row.sha256, row.bytes, row.mode);
  });
}
function makeEnv() {
  const runtime = path.join(OUTPUT, 'runtime');
  const pathEntries = [path.join(runtime, 'tool-bin'), path.join(runtime, 'bin'), path.dirname(NEXTEST), RUST_BIN, '/usr/bin', '/bin', '/usr/sbin', '/sbin'];
  return {
    CARGO_HOME, CARGO_TARGET_DIR: TARGET, CARGO_BUILD_JOBS: '2', CARGO_NET_OFFLINE: 'true', CARGO_TERM_COLOR: 'never',
    CARGO: CARGO, RUSTC, RUSTDOC, RUSTUP_HOME, RUSTUP_TOOLCHAIN,
    HOME: path.join(runtime, 'home'), TMPDIR: path.join(runtime, 'tmp'), TMP: path.join(runtime, 'tmp'), TEMP: path.join(runtime, 'tmp'),
    PATH: pathEntries.join(path.delimiter), NO_COLOR: '1', TERM: 'dumb', LANG: 'C.UTF-8', LC_ALL: 'C.UTF-8', TZ: 'UTC',
    NEXTEST_EXPERIMENTAL_LIBTEST_JSON: '1', GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null', GIT_NO_LAZY_FETCH: '1',
    GIT_NO_REPLACE_OBJECTS: '1', GIT_OPTIONAL_LOCKS: '0', GIT_TERMINAL_PROMPT: '0',
    MISE_DATA_DIR: path.join(runtime, 'mise-data'), MISE_CACHE_DIR: path.join(runtime, 'mise-cache'), MISE_STATE_DIR: path.join(runtime, 'mise-state'),
    MISE_GLOBAL_CONFIG_FILE: path.join(runtime, 'config/global.toml'), MISE_NO_CONFIG: '1', MISE_NO_ENV: '1', MISE_NO_HOOKS: '1',
    MISE_LOCKFILE: '0', MISE_AUTO_INSTALL: 'false', MISE_EXEC_AUTO_INSTALL: 'false', MISE_CARGO_HOME: CARGO_HOME, MISE_RUSTUP_HOME: RUSTUP_HOME
  };
}
function buildCommand(packet) {
  const ids = packet.test_selection.tests;
  if (packet.test_selection.count !== 1 || canonical(ids) !== canonical(SELECTED_IDS)) throw new Error('focused selector is not exact one-test set');
  const selectors = ids.map(id => `test(=${id})`).join(' + ');
  const options = packet.test_selection.nextest_options;
  const argv = ['nextest', 'run', ...options.slice(0, 2), '-p', packet.test_selection.package, ...options.slice(2), '-E', selectors];
  if (canonical(argv.slice(2)) !== canonical([...options.slice(0, 2), '-p', packet.test_selection.package, ...options.slice(2), '-E', selectors])) throw new Error('Nextest argv differs from frozen package selector/options');
  return { index: 1, package: packet.test_selection.package, binary: packet.test_selection.binary, expected_count: 1, test_ids: [...ids], argv: [CARGO, ...argv], cwd: SOURCE };
}
function verifyStatic(mode) {
  if (process.argv.length !== 3 || !['--preflight', '--execute'].includes(process.argv[2])) throw new Error('use exactly --preflight or --execute');
  if (process.execPath !== NODE || process.version !== 'v26.11.0') throw new Error('unpinned Node runtime');
  for (const p of [TASK_ROOT, EVIDENCE, RUN_ROOT, SOURCE, CARGO_HOME]) noSymlinkComponents(p);
  physicalDir(TASK_ROOT, 0o700); physicalDir(EVIDENCE, 0o700); physicalDir(RUN_ROOT, 0o700); physicalDir(SOURCE);
  const packet = readJson(PACKET_PATH);
  regular(PACKET_PATH, PACKET_SHA, undefined, '600');
  if (packet.source.root !== SOURCE) throw new Error('packet source root does not match pinned source constant');
  if (packet.schema !== 'termrock.vis11.fixture-focused-nextest-gate/v1' || packet.status !== 'FROZEN_PREPARED_NOT_RUN' || packet.qualification !== 'NOT_QUALIFIED' || packet.execution !== 'NOT_RUN') throw new Error('gate packet status/identity mismatch');
  if (packet.outputs.execution_directory !== OUTPUT || packet.outputs.target_directory !== TARGET || packet.runtime.target_dir !== TARGET || packet.outputs.execution_must_be_absent !== true || packet.outputs.target_must_be_absent_and_fresh !== true) throw new Error('execution/target path binding mismatch');
  if (exists(OUTPUT) || exists(TARGET)) throw new Error('execution output and fresh target must be absent');
  if (sha(path.join(SOURCE, 'Cargo.lock')) !== packet.source.lock_sha256) throw new Error('lock pin mismatch');
  for (const p of [OUTPUT, TARGET]) for (const root of [SOURCE, CARGO_HOME]) if (pathInside(p, root) || pathInside(root, p)) throw new Error('output overlaps source or Cargo home');
  if (pathInside(OUTPUT, TARGET) || pathInside(TARGET, OUTPUT)) throw new Error('execution and target paths overlap');
  const tools = verifyTools(packet);
  const seeds = verifyRuntimeSeeds(packet);
  const source = verifySource(packet);
  const cache = verifyCache(packet);
  const command = buildCommand(packet);
  const globalCache = snapshotMutableGlobalCache();
  const env = makeEnv();
  if (env.CARGO_TARGET_DIR !== TARGET || env.CARGO_HOME !== CARGO_HOME || env.CARGO_NET_OFFLINE !== 'true' || env.CARGO_BUILD_JOBS !== '2' || env.RUSTUP_TOOLCHAIN !== RUST_TOOLCHAIN) throw new Error('sanitized environment mismatch');
  return { packet, tools, seeds, source, cache, command, globalCache, environment: { inherited: false, keys: Object.keys(env).sort(), cargo_home: CARGO_HOME, cargo_target_dir: TARGET, offline: true, jobs: 2, test_threads: 1, mise_dirs_private_under_execution: true } };
}
function atomicJson(file, value) {
  const tmp = file + '.tmp', bytes = Buffer.from(JSON.stringify(value, null, 2) + '\n');
  const fd = fs.openSync(tmp, 'wx', 0o600);
  try { fs.writeFileSync(fd, bytes); fs.fsyncSync(fd); } finally { fs.closeSync(fd); }
  fs.renameSync(tmp, file); fs.chmodSync(file, 0o600);
  return { path: file, bytes: bytes.length, sha256: shaBytes(bytes) };
}
function prepareOutput(packet) {
  fs.mkdirSync(OUTPUT, { mode: 0o700 }); fs.chmodSync(OUTPUT, 0o700);
  const runtime = path.join(OUTPUT, 'runtime'); fs.mkdirSync(runtime, { mode: 0o700 });
  for (const rel of ['home', 'tmp', 'mise-data', 'mise-cache', 'mise-state', 'config', 'bin', 'tool-bin']) fs.mkdirSync(path.join(runtime, rel), { mode: 0o700 });
  const copied = [];
  for (const row of packet.runtime.runtime_seed_files) {
    const dest = path.join(runtime, row.relative_path);
    fs.mkdirSync(path.dirname(dest), { recursive: true, mode: 0o700 });
    const bytes = fs.readFileSync(row.path);
    if (bytes.length !== row.bytes || shaBytes(bytes) !== row.sha256) throw new Error('runtime seed changed: ' + row.relative_path);
    const fd = fs.openSync(dest, 'wx', parseInt(row.mode, 8));
    try { fs.writeFileSync(fd, bytes); fs.fsyncSync(fd); } finally { fs.closeSync(fd); }
    fs.chmodSync(dest, parseInt(row.mode, 8)); copied.push({ path: row.relative_path, bytes: row.bytes, sha256: row.sha256, mode: row.mode });
  }
  const alias = path.join(runtime, 'bin/mise'); fs.symlinkSync(MISE, alias);
  if (fs.readlinkSync(alias) !== MISE) throw new Error('private Mise alias mismatch');
  for (const rel of ['home', 'tmp', 'mise-data', 'mise-cache', 'mise-state', 'config', 'bin', 'tool-bin']) physicalDir(path.join(runtime, rel), 0o700);
  return { execution: OUTPUT, runtime: runtime, copied_seed_files: copied, mise_alias: { path: alias, target: MISE, sha256: sha(MISE) } };
}
function parseSummary(text) {
  const lines = text.split(/\r?\n/).map(line => line.trim()).filter(line => line.startsWith('Summary'));
  const m = lines.length === 1 ? lines[0].match(/^Summary\s+\[[^\]\r\n]+\]\s+(\d+) tests? run:\s+(\d+) passed(?:,\s+(\d+) failed)?(?:,\s+\d+ skipped)?$/) : null;
  return { lines, count: lines.length, valid: Boolean(m), executed: m ? Number(m[1]) : null, passed: m ? Number(m[2]) : null, failed: m ? Number(m[3] || 0) : null };
}
const NEXTTEST_STATUSES = new Set(['PASS', 'FAIL', 'SKIP', 'IGNORED', 'TIMEOUT', 'LEAKED', 'TERMINATED', 'CANCELLED', 'CANCELED', 'ABORTED']);
function parseOutcomes(text, ids) {
  const rows = [], unmatched = [], lines = text.split(/\r?\n/);
  let summaryAt = -1;
  for (let i = 0; i < lines.length; i++) if (lines[i].trim().startsWith('Summary')) summaryAt = summaryAt < 0 ? i : Math.min(summaryAt, i);
  const statusLine = /^\s*([A-Z][A-Z_-]*)\s+\[[^\]]+\]\s+\(\s*\d+\/\d+\)\s+.*$/;
  for (const line of lines.slice(0, summaryAt < 0 ? lines.length : summaryAt)) {
    const match = line.trimEnd().match(statusLine); if (!match) continue;
    const candidates = ids.filter(id => line.trimEnd().endsWith(' ' + id));
    if (candidates.length !== 1) { unmatched.push({ status: match[1], raw_line: line }); continue; }
    rows.push({ id: candidates[0], status: match[1], raw_line: line.trimEnd() });
  }
  const outcomes = ids.map(id => {
    const found = rows.filter(row => row.id === id), unique = [...new Map(found.map(row => [row.raw_line, row])).values()];
    const statuses = [...new Set(unique.map(row => row.status))];
    const status = unique.length === 0 ? 'NOT_OBSERVED' : statuses.length > 1 ? 'CONFLICTING_STATUS_ROWS' : unique.length > 1 ? 'DUPLICATE_STATUS_ROWS' : statuses[0];
    return { id, status, rows: found.map(row => row.raw_line), unique_status_rows: unique.length };
  });
  const passed = outcomes.filter(row => row.status === 'PASS').map(row => row.id).sort();
  const failed = outcomes.filter(row => row.status === 'FAIL' || row.status === 'TIMEOUT' || row.status === 'LEAKED' || row.status === 'TERMINATED' || row.status === 'CANCELLED' || row.status === 'CANCELED' || row.status === 'ABORTED').map(row => row.id).sort();
  const classified = outcomes.every(row => NEXTTEST_STATUSES.has(row.status));
  return { rows, outcomes, passed, failed, unmatched, exact_one_row_per_selected_test: rows.length === ids.length && unmatched.length === 0 && outcomes.every(row => row.unique_status_rows === 1 && NEXTTEST_STATUSES.has(row.status)) && new Set(rows.map(row => row.id)).size === ids.length };
}
function writeAll(fd, bytes) { let off = 0; while (off < bytes.length) { const n = fs.writeSync(fd, bytes, off, bytes.length - off); if (n <= 0) throw new Error('short output log write'); off += n; } }
function signalGroup(pid, signal) { try { process.kill(-pid, signal); return { sent: true, error: null }; } catch (e) { return { sent: false, error: String(e && e.message || e) }; } }
function runCommand(command, env, started, deadline) {
  return new Promise(resolve => {
    const logPath = path.join(OUTPUT, 'command-1.log'), terminalPath = path.join(OUTPUT, 'command-1-terminal.json');
    const fd = fs.openSync(logPath, 'wx', 0o600);
    let child, exitCode = null, signal = null, closeSeen = false, timedOut = false, overflow = false, done = false, stopRequested = false, stored = 0, observed = 0, launchError = null, timer = null, killTimer = null, closeTimer = null;
    const signals = [];
    function finish(closeExpired = false) {
      if (done) return; done = true; activeStop = null; if (timer) clearTimeout(timer); if (killTimer) clearTimeout(killTimer); if (closeTimer) clearTimeout(closeTimer);
      if (child && child.pid) signals.push({ signal: 'SIGKILL', ...signalGroup(child.pid, 'SIGKILL'), reason: 'final_process_group_cleanup' });
      try { fs.fsyncSync(fd); } catch {} try { fs.closeSync(fd); } catch {}
      const logStat = fs.statSync(logPath), complete = closeSeen && !overflow;
      const logSha = sha(logPath), text = complete ? fs.readFileSync(logPath, 'utf8') : '';
      const summary = parseSummary(text), parsed = parseOutcomes(text, SELECTED_IDS);
      const within = Date.now() < deadline;
      const exact = complete && exitCode === 0 && !timedOut && !overflow && !launchError && within && summary.valid && summary.count === 1 && summary.executed === 1 && summary.passed === 1 && summary.failed === 0 && parsed.exact_one_row_per_selected_test && parsed.passed.length === 1 && parsed.failed.length === 0 && canonical(parsed.passed) === canonical(SELECTED_IDS);
      const terminal = { schema: 'termrock.vis11.fixture-focused-nextest-terminal/v1', index: 1, exit_code: exitCode, signal, close_seen: closeSeen, close_grace_expired: closeExpired, timed_out: timedOut, overflow, launch_error: launchError, output: { path: logPath, bytes: logStat.size, sha256: logSha, complete }, summary, outcomes: parsed, process_group_signals: signals, passed: exact };
      const terminalPin = atomicJson(terminalPath, terminal);
      resolve({ index: 1, package: command.package, binary: command.binary, expected_test_ids: SELECTED_IDS, argv: command.argv, cwd: SOURCE, exit_code: exitCode, signal, timed_out: timedOut, overflow, launch_error: launchError, close_seen: closeSeen, elapsed_ms: Date.now() - started, output: terminal.output, summary, outcomes: parsed, terminal_receipt: terminalPin, command_pass: exact, status: exact ? 'passed' : 'failed_or_incomplete' });
    }
    try {
      child = spawn(CARGO, command.argv.slice(1), { cwd: SOURCE, env, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
    } catch (e) { launchError = String(e && e.stack || e); return finish(); }
    const stop = reason => {
      if (done || stopRequested) return;
      stopRequested = true;
      if (reason === 'deadline') timedOut = true;
      signals.push({ signal: 'SIGTERM', ...signalGroup(child.pid, 'SIGTERM'), reason });
      killTimer = setTimeout(() => { signals.push({ signal: 'SIGKILL', ...signalGroup(child.pid, 'SIGKILL'), reason: 'term_grace_expired' }); closeTimer = setTimeout(() => finish(true), LIMITS.close_grace_ms); }, LIMITS.term_grace_ms);
    };
    activeStop = stop;
    function capture(chunk) {
      observed += chunk.length;
      const remaining = Math.max(0, LIMITS.output_bytes - stored), keep = chunk.subarray(0, Math.min(chunk.length, remaining));
      if (keep.length) { try { writeAll(fd, keep); stored += keep.length; } catch (e) { launchError = 'output log write failed: ' + String(e && e.message || e); stop('output_log_write_failed'); return; } }
      if (keep.length !== chunk.length && !overflow) { overflow = true; stop('combined_output_cap'); }
    }
    child.stdout.on('data', capture); child.stderr.on('data', capture);
    child.once('error', e => { launchError = String(e && e.stack || e); });
    child.once('exit', (code, sig) => { exitCode = code; signal = sig; });
    child.once('close', (code, sig) => { closeSeen = true; if (exitCode === null) exitCode = code; if (signal === null) signal = sig; finish(); });
    timer = setTimeout(() => stop('deadline'), Math.max(0, deadline - Date.now()));
  });
}
let parentSignal = null, activeStop = null;
for (const sig of ['SIGTERM', 'SIGINT']) process.on(sig, () => { parentSignal = sig; if (activeStop) activeStop('supervisor_' + sig.toLowerCase()); });
async function execute(prepared) {
  const start = Date.now(), deadline = start + LIMITS.deadline_ms;
  const sourceBefore = verifySource(prepared.packet), cacheBefore = verifyCache(prepared.packet), mutableBefore = snapshotMutableGlobalCache();
  const output = prepareOutput(prepared.packet), env = makeEnv();
  atomicJson(path.join(OUTPUT, 'ready.json'), { schema: 'termrock.vis11.fixture-focused-ready/v1', status: 'STARTED_AFTER_ROOT_GRANT', qualification: 'NOT_QUALIFIED', started_utc: new Date(start).toISOString(), deadline_utc: new Date(deadline).toISOString(), packet_sha256: PACKET_SHA, runner_sha256: sha(__filename), source: sourceBefore, cache: cacheBefore, output, target: { path: TARGET, absent_before_child: true, fresh: true }, command: prepared.command, sanitized_environment: prepared.environment, limits: LIMITS });
  const result = await runCommand(prepared.command, env, start, deadline);
  const sourceAfter = verifySource(prepared.packet), cacheAfter = verifyCache(prepared.packet), mutableAfter = snapshotMutableGlobalCache();
  const resultOk = result.command_pass && sourceAfter.full_tree_sha256 === sourceBefore.full_tree_sha256 && canonical(cacheAfter) === canonical(cacheBefore) && !parentSignal;
  const commandReceipt = atomicJson(path.join(OUTPUT, 'command-1-receipt.json'), { schema: 'termrock.vis11.fixture-focused-command-receipt/v1', qualification: 'NOT_QUALIFIED', source_before: sourceBefore, source_after: sourceAfter, immutable_cache_before: cacheBefore, immutable_cache_after: cacheAfter, mutable_global_cache_before: mutableBefore, mutable_global_cache_after: mutableAfter, command: result, source_unchanged: sourceAfter.full_tree_sha256 === sourceBefore.full_tree_sha256, immutable_cache_unchanged: canonical(cacheAfter) === canonical(cacheBefore) });
  const receipt = { schema: 'termrock.vis11.fixture-focused-nextest-receipt/v1', status: resultOk ? 'focused_tests_passed' : 'focused_tests_failed_or_incomplete_preserved', qualification: 'NOT_QUALIFIED', packet_sha256: PACKET_SHA, runner_sha256: sha(__filename), source_root: SOURCE, source_tree_sha256: sourceBefore.full_tree_sha256, selected: 1, executed: result.outcomes.rows.length, passed_ids: result.outcomes.passed, failed_ids: result.outcomes.failed, outcomes: result.outcomes.outcomes, command_receipt: commandReceipt, source_after: sourceAfter, cache_after: cacheAfter, mutable_global_cache_before: mutableBefore, mutable_global_cache_after: mutableAfter, elapsed_ms: Date.now() - start, deadline_ms: LIMITS.deadline_ms, previous_48_case_result: prepared.packet.prior_result.prior_baseline, nonclaims: prepared.packet.no_claims };
  const final = atomicJson(path.join(OUTPUT, 'receipt.json'), receipt);
  process.stdout.write(JSON.stringify({ type: 'focused_gate_finished', status: receipt.status, selected: 1, executed: receipt.executed, passed: receipt.passed_ids.length, failed: receipt.failed_ids.length, receipt_sha256: final.sha256 }) + '\n');
  return resultOk ? 0 : 1;
}
(async () => {
  try {
    const prepared = verifyStatic(process.argv[2]);
    const report = { schema: 'termrock.vis11.fixture-focused-nextest-preflight/v1', status: 'READY_FOR_INDEPENDENT_REVIEW_AND_ROOT_EXECUTION_DECISION_NOT_RUN', qualification: 'NOT_QUALIFIED', packet_path: PACKET_PATH, packet_sha256: PACKET_SHA, runner_path: __filename, runner_sha256: sha(__filename), source: prepared.source, cache: prepared.cache, tools: prepared.tools, runtime_seeds: prepared.seeds.length, mutable_global_cache: prepared.globalCache, command: prepared.command, environment: prepared.environment, outputs: { execution: OUTPUT, absent: true, target: TARGET, absent_and_fresh: true }, limits: LIMITS, preflight_action: 'No Cargo, Nextest, build, Rust test, or CLI was spawned.' };
    if (process.argv[2] === '--preflight') { process.stdout.write(JSON.stringify(report, null, 2) + '\n'); return; }
    const code = await execute(prepared); process.exitCode = code;
  } catch (e) {
    const report = { schema: 'termrock.vis11.fixture-focused-nextest-runner-error/v1', status: 'preflight_or_execution_error', qualification: 'NOT_QUALIFIED', error: String(e && e.stack || e), execution_directory_exists: exists(OUTPUT) };
    process.stderr.write(JSON.stringify(report, null, 2) + '\n'); process.exitCode = 2;
  }
})();
