'use strict';

const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn } = require('node:child_process');

const TASK_ROOT = '/private/tmp/termrock-vis11-prerelease-v2-48case-gate-luna-r5-r8-20261010';
const EVIDENCE = '/private/tmp/termrock-vis11-prerelease-v2-48case-gate-luna-r5-r8-20261010/evidence';
const RUN_ROOT = path.join(TASK_ROOT, 'run-root');
const PACKET_PATH = path.join(EVIDENCE, 'gate-packet.json');
const PACKET_SHA = '5ab40a5ced48ea768f56c974ab177a64bca8639367a9a67444b1ad045625ccc1';
const SOURCE = '/private/tmp/termrock-vis11-tool-fixture-admission-r8-source-tag-snapshot-luna-20261010/source';
const CARGO_HOME = '/private/tmp/termrock-vis11-metadata-cache-delta-r6-20261009/cargo-home';
const RUSTUP_HOME = '/Users/donbeave/.rustup';
const RUST_BIN = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin';
const RUST_TOOLCHAIN = '1.98.1-aarch64-apple-darwin';
const CARGO = path.join(RUST_BIN, 'cargo');
const RUSTC = path.join(RUST_BIN, 'rustc');
const RUSTDOC = path.join(RUST_BIN, 'rustdoc');
const NEXTEST = '/Users/donbeave/.local/share/mise/installs/aqua-nextest-rs-nextest-cargo-nextest/0.9.146/cargo-nextest';
const NODE = '/opt/homebrew/Cellar/node/26.11.0_1/bin/node';
const MISE = '/Users/donbeave/Projects/tailrocks/.termrock-visibility-runtime-20261009/mise-2026.9.18-macos-arm64/mise-v2026.9.18-macos-arm64';
const OUTPUT = path.join(RUN_ROOT, 'execution');
const TARGET = '/private/tmp/termrock-vis11-tool-fixture-gate-r9-source-tag-fixture-luna-20261010/run-root/target';
const PLAN_PATH = path.join(EVIDENCE, 'impact-plan.json');
const LIMITS = { deadline_ms: 1200 * 1000, term_grace_ms: 15000, close_grace_ms: 10000, output_bytes: 4 * 1024 * 1024 };
const EXPECTED_COUNTS = [2, 2, 23, 11, 6, 4];

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
function safeRel(value) {
  return typeof value === 'string' && value.length > 0 && !value.startsWith('/') && !value.includes('\\') && !/[\0\t\r\n]/.test(value) && value.split('/').every(part => part && part !== '.' && part !== '..');
}
function regular(file, expectedSha, expectedBytes, expectedMode, executable = false) {
  noSymlinkComponents(path.dirname(file));
  const st = fs.lstatSync(file);
  if (!st.isFile() || st.isSymbolicLink()) throw new Error('expected regular non-symlink file: ' + file);
  const digest = sha(file);
  if (expectedSha && digest !== expectedSha) throw new Error('SHA-256 mismatch: ' + file + ' got ' + digest);
  if (expectedBytes !== undefined && st.size !== expectedBytes) throw new Error('byte count mismatch: ' + file);
  if (expectedMode && (st.mode & 0o777).toString(8).padStart(3, '0') !== expectedMode) throw new Error('mode mismatch: ' + file);
  if (executable && (st.mode & 0o111) === 0) throw new Error('expected executable: ' + file);
  return { path: file, bytes: st.size, sha256: digest, mode: (st.mode & 0o777).toString(8).padStart(3, '0') };
}
function collectPinnedFiles(value, out = new Map()) {
  if (Array.isArray(value)) for (const item of value) collectPinnedFiles(item, out);
  else if (value && typeof value === 'object') {
    if (typeof value.path === 'string' && value.path.startsWith('/') && /^[0-9a-f]{64}$/.test(value.sha256 || '')) {
      const prior = out.get(value.path);
      if (prior && prior.sha256 !== value.sha256) throw new Error('conflicting pins for ' + value.path);
      out.set(value.path, { path: value.path, sha256: value.sha256, bytes: value.bytes, mode: value.mode });
    }
    for (const item of Object.values(value)) collectPinnedFiles(item, out);
  }
  return [...out.values()];
}
function walkTree(root, rel = '', result = []) {
  for (const name of fs.readdirSync(path.join(root, rel)).sort()) {
    const child = rel ? rel + '/' + name : name;
    if (!safeRel(child)) throw new Error('unsafe source path: ' + child);
    const abs = path.join(root, child);
    const st = fs.lstatSync(abs);
    const mode = (st.mode & 0o777).toString(8).padStart(3, '0');
    if (st.isSymbolicLink()) result.push({ path: child, type: 'symlink', mode, target: fs.readlinkSync(abs) });
    else if (st.isDirectory()) { result.push({ path: child, type: 'directory', mode }); walkTree(root, child, result); }
    else if (st.isFile()) result.push({ path: child, type: 'file', mode, bytes: st.size, sha256: sha(abs) });
    else throw new Error('special source entry: ' + child);
  }
  return result;
}
function verifySource(packet) {
  const sourceReal = physicalDir(SOURCE, parseInt(packet.source.root_mode, 8));
  const treePin = packet.source.full_tree;
  regular(treePin.path, treePin.sha256, treePin.bytes, treePin.mode);
  const tree = readJson(treePin.path);
  if (tree.source_root !== SOURCE || canonical(tree.counts) !== canonical({ file: 2608, directory: 842, symlink: 7 }) || tree.entry_count !== 3457 || tree.entries.length !== 3457) throw new Error('full source tree pin/count mismatch');
  const expected = new Map();
  for (const item of tree.entries) {
    if (!safeRel(item.path) || expected.has(item.path) || !['file','directory','symlink'].includes(item.type) || !/^[0-7]{3}$/.test(item.mode)) throw new Error('malformed or duplicate full-tree row: ' + item.path);
    if (item.type === 'file' && (!/^[0-9a-f]{64}$/.test(item.sha256 || '') || !Number.isInteger(item.bytes) || item.bytes < 0)) throw new Error('malformed file row: ' + item.path);
    if (item.type === 'symlink' && (typeof item.target !== 'string' || !item.target || item.target.startsWith('/') || item.target.includes('\\') || /[\0\t\r\n]/.test(item.target))) throw new Error('unsafe symlink row: ' + item.path);
    expected.set(item.path, item);
  }
  const actual = walkTree(SOURCE);
  if (canonical(actual) !== canonical(tree.entries)) throw new Error('source differs from full path/mode/content tree inventory');
  for (const item of tree.entries) if (item.type === 'symlink') {
    const link = path.join(SOURCE, item.path);
    const lexicalTarget = path.resolve(path.dirname(link), item.target);
    const realTarget = fs.realpathSync(lexicalTarget);
    if (!pathInside(realTarget, sourceReal)) throw new Error('symlink escapes source root: ' + item.path);
    const targetRel = path.relative(SOURCE, lexicalTarget).split(path.sep).join('/');
    const target = expected.get(targetRel);
    if (!target || target.type !== 'file' || sha(lexicalTarget) !== target.sha256) throw new Error('symlink target is not a pinned regular file: ' + item.path);
  }
  const inventoryPin = packet.source.post_metadata_inventory;
  regular(inventoryPin.path, inventoryPin.sha256, inventoryPin.bytes, inventoryPin.mode);
  const inventory = readJson(inventoryPin.path);
  const nonDirs = tree.entries.filter(row => row.type !== 'directory');
  if (inventory.entry_count !== 2615 || inventory.entries.length !== 2615 || packet.source.entry_count !== 3457 || nonDirs.length !== 2615) throw new Error('post-metadata inventory count mismatch');
  const byPath = (a,b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0;
  const normalized = nonDirs.map(row => ({ path: row.path, type: row.type, mode: row.mode, ...(row.type === 'file' ? { bytes: row.bytes, sha256: row.sha256 } : { target: row.target, sha256: shaBytes(Buffer.from(row.target,'utf8')) }) })).sort(byPath);
  const inventoryRows = [...inventory.entries].sort(byPath);
  if (canonical(normalized) !== canonical(inventoryRows)) throw new Error('post-metadata file/symlink inventory disagrees with full tree');
  if (sha(path.join(SOURCE, 'Cargo.lock')) !== packet.source.lock_sha256 || fs.statSync(path.join(SOURCE, 'Cargo.lock')).size !== packet.source.lock_bytes) throw new Error('post-metadata lock identity mismatch');
  for (const row of packet.source.relevant_files) {
    const item = expected.get(row.path);
    if (!item || item.type !== 'file' || item.sha256 !== row.sha256 || item.bytes !== row.bytes || item.mode !== row.mode || sha(path.join(SOURCE,row.path)) !== row.sha256) throw new Error('relevant source input mismatch: ' + row.path);
  }
  for (const rel of ['Cargo.toml','Cargo.lock','.config/nextest.toml']) if (!packet.source.relevant_files.some(row => row.path === rel)) throw new Error('missing explicit build/Nextest input pin: ' + rel);
  if (exists(path.join(SOURCE, '.git')) || exists(path.join(SOURCE, 'target'))) throw new Error('source has Git metadata or build output');
  const mapPin = packet.source.test_source_map;
  regular(mapPin.path, mapPin.sha256, mapPin.bytes, mapPin.mode);
  const map = readJson(mapPin.path);
  if (map.selected_count !== 48 || map.tests.length !== 48 || map.source_tree_sha256 !== treePin.sha256) throw new Error('test source-map identity mismatch');
  const selected = packet.test_selection.tests;
  if (selected.length !== 48 || new Set(selected.map(test => test.id)).size !== 48) throw new Error('selected test IDs are not 48 unique cases');
  if (canonical(selected.map(test => ({ id:test.id, package:test.package, group_index:test.group_index, source_path:test.source_path, function:test.function, line:test.line, source_sha256:test.source_sha256 })).sort((a,b)=>a.id.localeCompare(b.id)) ) !== canonical(map.tests.map(test => ({id:test.id,package:test.package,group_index:test.group_index,source_path:test.source_path,function:test.function,line:test.line,source_sha256:test.source_sha256})).sort((a,b)=>a.id.localeCompare(b.id)))) throw new Error('selected test map differs from pinned source map');
  for (const test of selected) {
    const item = expected.get(test.source_path);
    if (!item || item.type !== 'file' || item.sha256 !== test.source_sha256) throw new Error('selected test source not pinned: ' + test.id);
    const text = fs.readFileSync(path.join(SOURCE,test.source_path),'utf8');
    const re = new RegExp('\\bfn\\s+' + test.function + '\\b','g');
    const matches = [...text.matchAll(re)];
    if (matches.length !== 1 || text.slice(0,matches[0].index).split('\n').length !== test.line) throw new Error('test source line/function mismatch: ' + test.id);
  }
  return { root: SOURCE, full_tree_sha256: treePin.sha256, full_tree_entries: tree.entry_count, regular_files: tree.counts.file, directories: tree.counts.directory, symlinks: tree.counts.symlink, post_metadata_inventory_sha256: inventoryPin.sha256, lock_sha256: packet.source.lock_sha256, selected_source_functions: selected.length, git_identity: 'NOT_ASSERTED_GITLESS_EXTERNAL_SNAPSHOT' };
}
function inventoryCachePath(cargoHome, rel, output) {
  const full = path.join(cargoHome, rel);
  const st = fs.lstatSync(full);
  if (st.isSymbolicLink()) throw new Error('symlink in immutable Cargo registry closure: ' + rel);
  if (st.isDirectory()) {
    output.push({ path: rel.split(path.sep).join('/'), type: 'directory' });
    for (const name of fs.readdirSync(full).sort()) inventoryCachePath(cargoHome,path.join(rel,name),output);
  } else if (st.isFile()) output.push({ path: rel.split(path.sep).join('/'), type: 'file', size: st.size, sha256: sha(full) });
  else throw new Error('special cache entry: ' + rel);
}
function verifyImmutableCache(packet) {
  const archivePin = packet.runtime.archive_manifest;
  const closurePin = packet.runtime.source_index_closure;
  regular(archivePin.path,archivePin.sha256,archivePin.bytes,archivePin.mode);
  regular(closurePin.path,closurePin.sha256,closurePin.bytes,closurePin.mode);
  const archives = readJson(archivePin.path);
  if (archives.record_type !== 'velnor_private_cargo_registry_archive_manifest_v1' || archives.cargo_home !== CARGO_HOME || archives.archive_count !== 406 || archives.archive_bytes !== 59163074 || archives.archives.length !== 406 || archives.unpacked_source_package_directory_count !== 345) throw new Error('archive manifest identity/count mismatch');
  const expectedArchives = new Map(archives.archives.map(row => [row.name,row]));
  if (expectedArchives.size !== 406) throw new Error('duplicate archive names in manifest');
  const archiveRoot = path.join(CARGO_HOME,'registry/cache');
  const crateFiles=[];
  function walkArchives(dir){for(const name of fs.readdirSync(dir).sort()){const full=path.join(dir,name),st=fs.lstatSync(full);if(st.isSymbolicLink())throw new Error('symlink in archive cache');if(st.isDirectory())walkArchives(full);else if(st.isFile()&&name.endsWith('.crate'))crateFiles.push(full);else throw new Error('unlisted/special archive entry: '+full);}}
  walkArchives(archiveRoot);
  let archiveBytes=0;
  if(crateFiles.length!==406)throw new Error('archive count mismatch');
  for(const file of crateFiles){const name=path.basename(file),row=expectedArchives.get(name),st=fs.lstatSync(file);if(!row||row.bytes!==st.size||row.sha256!==sha(file))throw new Error('archive differs from pinned manifest: '+name);expectedArchives.delete(name);archiveBytes+=st.size;}
  if(expectedArchives.size||archiveBytes!==59163074)throw new Error('archive closure totals mismatch');
  const closure=readJson(closurePin.path);
  if(closure.record_type!=='velnor_cargo_registry_consumed_source_index_closure_v1'||closure.cargo_home!==CARGO_HOME||canonical(closure.roots)!==canonical(['registry/src','registry/index'])||closure.file_count!==20133||closure.directory_count!==3666||closure.file_bytes!==200791635||closure.symlink_count!==0||closure.entries.length!==23799)throw new Error('source/index closure identity mismatch');
  const errPin=packet.runtime.source_index_summary_erratum;
  regular(errPin.path,errPin.sha256,errPin.bytes,errPin.mode);
  const erratum=readJson(errPin.path);
  const rawFiles=closure.entries.filter(e=>e.type==='file');const rawDirs=closure.entries.filter(e=>e.type==='directory');
  const derivedBytes=rawFiles.reduce((sum,e)=>sum+e.size,0);
  const packageDirs=new Set(rawDirs.map(e=>e.path.split('/')).filter(parts=>parts.length===4&&parts[0]==='registry'&&parts[1]==='src').map(parts=>parts[3]));
  const markers=rawFiles.filter(e=>e.path.endsWith('/.cargo-ok'));
  if(erratum.schema!=='termrock.vis11.cache-source-index-summary-erratum/v1'||erratum.qualification!=='NOT_QUALIFIED'||erratum.raw_closure.sha256!==closurePin.sha256||erratum.raw_closure.declared_file_bytes!==closure.file_bytes||erratum.derived_from_raw_entries.entry_count!==closure.entries.length||erratum.derived_from_raw_entries.file_count!==rawFiles.length||erratum.derived_from_raw_entries.directory_count!==rawDirs.length||erratum.derived_from_raw_entries.sum_file_entry_bytes!==derivedBytes||erratum.derived_from_raw_entries.immediate_source_package_directory_count!==packageDirs.size||erratum.derived_from_raw_entries.cargo_ok_marker_count!==markers.length||derivedBytes!==441286079||packageDirs.size!==406||markers.length!==406)throw new Error('derived source/index accounting erratum mismatch');
  if(packet.runtime.cache_summary.source_index_bytes!==derivedBytes||packet.runtime.cache_summary.source_index_bytes_declared_in_raw_closure!==closure.file_bytes||packet.runtime.cache_summary.source_package_directories!==packageDirs.size)throw new Error('cache summary does not reflect raw/derived values');
  const expected=new Map(closure.entries.map(e=>[e.path,e]));if(expected.size!==23799)throw new Error('duplicate source/index paths');
  const actual=[];inventoryCachePath(CARGO_HOME,'registry/src',actual);inventoryCachePath(CARGO_HOME,'registry/index',actual);
  if(actual.length!==closure.entries.length)throw new Error('source/index entry count mismatch');
  let files=0,dirs=0,bytes=0;
  for(const row of actual){const pinned=expected.get(row.path);if(!pinned||canonical(pinned)!==canonical(row))throw new Error('source/index entry changed: '+row.path);expected.delete(row.path);if(row.type==='file'){files++;bytes+=row.size;}else dirs++;}
  if(expected.size||files!==closure.file_count||dirs!==closure.directory_count||bytes!==derivedBytes)throw new Error('source/index entry totals mismatch');
  const srcRoot=path.join(CARGO_HOME,'registry/src');const registries=fs.readdirSync(srcRoot);if(registries.length!==1)throw new Error('unexpected registry source roots');
  const packageRoot=path.join(srcRoot,registries[0]);const packages=fs.readdirSync(packageRoot);if(packages.length!==406)throw new Error('extracted package count mismatch');
  for(const name of packages){const pkg=path.join(packageRoot,name),st=fs.lstatSync(pkg);if(!st.isDirectory()||st.isSymbolicLink())throw new Error('invalid extracted package directory');regular(path.join(pkg,'.cargo-ok'),'afbf9d0f3560b0fd7795e81c42a0a79ee6b6fc67e064f77826aee642cad28d91',7);}
  return {archive_manifest_sha256:archivePin.sha256,archive_count:crateFiles.length,archive_bytes:archiveBytes,source_index_closure_sha256:closurePin.sha256,source_index_files:files,source_index_directories:dirs,source_index_entry_bytes:bytes,raw_closure_declared_file_bytes:closure.file_bytes,derived_source_package_directories:packages.length,archive_manifest_declared_unpacked_source_package_directories:archives.unpacked_source_package_directory_count,exact_cargo_ok_markers:406,qualification:'NOT_QUALIFIED'};
}
function snapshotMutableGlobalCache() {
  const file=path.join(CARGO_HOME,'.global-cache');
  try{const st=fs.lstatSync(file);if(st.isSymbolicLink())return{path:file,kind:'symlink',target:fs.readlinkSync(file),status:'recorded_mutable_not_input'};if(st.isFile())return{path:file,kind:'regular_file',size_bytes:st.size,sha256:sha(file),mode:(st.mode&0o777).toString(8).padStart(3,'0'),status:'recorded_mutable_not_input'};if(st.isDirectory())return{path:file,kind:'directory',status:'recorded_mutable_not_input',inventory:walkTree(CARGO_HOME,'.global-cache')};return{path:file,kind:'special',status:'recorded_mutable_not_input'};}catch(e){if(e&&e.code==='ENOENT')return{path:file,kind:'absent',status:'recorded_mutable_not_input'};throw e;}
}
function verifyRuntimeSeeds(packet) {
  const rows=packet.runtime.runtime_seed_files;if(rows.length!==13||new Set(rows.map(r=>r.relative_path)).size!==13)throw new Error('runtime seed list mismatch');
  return rows.map(row=>{if(!safeRel(row.relative_path))throw new Error('unsafe runtime seed path');return regular(row.path,row.sha256,row.bytes,row.mode);});
}
function verifyTools(packet) {
  const files=[];for(const key of ['cargo','rustc','rustdoc','nextest','node','mise']){const t=packet.tools[key];files.push(regular(t.path,t.sha256,undefined,undefined,true));}
  for(const [file,digest] of packet.tools.system_helpers)files.push(regular(file,digest,undefined,undefined,true));
  if(process.execPath!==NODE||process.version!==packet.tools.node.version||sha(process.execPath)!==packet.tools.node.sha256)throw new Error('runner Node path/version/hash mismatch');
  if(CARGO!==packet.tools.cargo.path||RUSTC!==packet.tools.rustc.path||RUSTDOC!==packet.tools.rustdoc.path||NEXTEST!==packet.tools.nextest.path||MISE!==packet.tools.mise.path)throw new Error('runtime tool path mismatch');
  const cargoReal=physicalDir(CARGO_HOME);
  if(exists(path.join(CARGO_HOME,'config'))||exists(path.join(CARGO_HOME,'config.toml')))throw new Error('Cargo home has unpinned config');
  return {tools:files,cargo_home:{path:CARGO_HOME,realpath:cargoReal,config_absent:true}};
}
function buildCommands(packet) {
  const groups=packet.test_selection.groups;
  if(groups.length!==6||groups.map(g=>g.expected_count).join(',')!==EXPECTED_COUNTS.join(','))throw new Error('six exact group sizes mismatch');
  const tests=packet.test_selection.tests;
  const all=groups.flatMap(g=>g.tests);
  if(all.length!==48||new Set(all.map(t=>t.id)).size!==48||packet.test_selection.selected_unique_tests!==48)throw new Error('48-case unique denominator mismatch');
  const plan=readJson(PLAN_PATH);
  if(sha(PLAN_PATH)!==packet.impact_plan.sha256||plan.commands.length!==6)throw new Error('impact plan pin mismatch');
  const commands=[];
  for(let i=0;i<groups.length;i++){
    const group=groups[i],sourcePlan=plan.commands[i];
    if(group.index!==i+1||group.package!==sourcePlan.package||group.label!==sourcePlan.label||group.expected_count!==sourcePlan.expected_count||canonical(group.test_ids)!==canonical(sourcePlan.selected_tests))throw new Error('packet group differs from impact plan at '+(i+1));
    if(group.tests.length!==group.expected_count||canonical(group.tests.map(t=>t.id))!==canonical(group.test_ids))throw new Error('test source map does not match group IDs');
    const argv=[CARGO,'nextest','run','--locked','--offline','-p',group.package,'--build-jobs','2','--test-threads','1','--no-tests','fail','--user-config-file','none','--ignore-default-filter','--retries','0','--no-fail-fast','--color','never','-E',group.test_ids.map(id=>'test(='+id+')').join(' | ')];
    commands.push({index:group.index,package:group.package,label:group.label,expected_count:group.expected_count,tests:group.tests,argv});
  }
  return commands;
}
function verifyPinnedInputs(packet) {
  const packetFile=regular(PACKET_PATH,PACKET_SHA);
  const pinned=collectPinnedFiles(packet);const checked=pinned.map(item=>regular(item.path,item.sha256,item.bytes,item.mode));
  const seeds=verifyRuntimeSeeds(packet);const toolRecords=verifyTools(packet);
  const metaReview=readJson(packet.metadata_actual_review.path);const metaResult=readJson(packet.metadata_actual_result.path);
  if(metaReview.verdict!=='VERIFIED_METADATA_ONLY_SUCCESS_NOT_QUALIFIED'||metaReview.result.path!==packet.metadata_actual_result.path||metaReview.result.sha256!==packet.metadata_actual_result.sha256||metaReview.result.status!=='PASS_METADATA_ONLY'||metaReview.result.child_exit!==0||metaReview.result.child_closed!==true||metaReview.result.qualification!=='NOT_QUALIFIED'||metaResult.status!=='PASS_METADATA_ONLY'||metaResult.qualification!=='NOT_QUALIFIED')throw new Error('metadata result evidence is not the reviewed metadata-only result');
  if(metaResult.tests_run!==false||metaResult.build_run!==false||metaResult.cli_run!==false||metaResult.post_integrity.source.onlyCargoLockMayChange!==true||metaResult.post_integrity.source.unexpected.length!==0||metaResult.post_integrity.cargo_lock_after_sha256!==packet.source.lock_sha256)throw new Error('metadata result permits unexpected source mutation or includes product execution');
  const metaTerminal=readJson(metaReview.terminal.path);if(sha(metaReview.terminal.path)!==metaReview.terminal.sha256||metaTerminal.result_sha256!==packet.metadata_actual_result.sha256||metaReview.terminal.result_sha256_matches!==true)throw new Error('metadata terminal binding mismatch');
  for(const raw of [metaReview.raw.stdout_path&&{path:metaReview.raw.stdout_path,sha256:metaReview.raw.stdout_sha256,bytes:metaReview.raw.stdout_bytes},metaReview.raw.stderr_path&&{path:metaReview.raw.stderr_path,sha256:metaReview.raw.stderr_sha256,bytes:metaReview.raw.stderr_bytes},metaReview.raw.command_path&&{path:metaReview.raw.command_path,sha256:metaReview.raw.command_sha256}].filter(Boolean))regular(raw.path,raw.sha256,raw.bytes);
  if(metaReview.result.metadata.packages!==484||metaReview.result.metadata.workspace_members!==78||metaReview.result.metadata.resolve_nodes!==484||metaReview.result.metadata.targets!==2259)throw new Error('reviewed metadata graph counts changed');
  const seedReview=readJson(packet.runtime.runtime_seed_review.path);const actualSeedReview=readJson(packet.runtime.runtime_seed_review.actual_result_review.path);
  const miseWrapper=seedReview.runtime&&seedReview.runtime.mise_wrapper;
  if(!miseWrapper||miseWrapper.mise_binary_sha256!==packet.tools.mise.sha256||miseWrapper.mise_release!=='2026.9.18 macos-arm64'||actualSeedReview.verdict!=='CLI_EXIT_ZERO_EXPECTED_SNAPSHOT_MISMATCH')throw new Error('private Mise seed provenance mismatch');
  const cacheReview=readJson(packet.runtime.cache_independent_review.path);if(cacheReview.verdict!=='READY')throw new Error('cache output review is not READY');
  const summaryReview=readJson(packet.runtime.cache_summary_erratum_review.path);if(summaryReview.verdict!=='READY_STATIC_PREDICATE_REVIEW_ONLY')throw new Error('cache summary erratum review mismatch');
  return {packet:packetFile,pinned_external_files:checked.length,runtime_seed_files:seeds.length,tools:toolRecords,metadata_review_sha256:packet.metadata_actual_review.sha256,metadata_result_sha256:packet.metadata_actual_result.sha256};
}
function verifyStatic(mode) {
  if(process.argv.length!==3||!['--preflight','--execute'].includes(process.argv[2]))throw new Error('use exactly --preflight or --execute');
  if(process.execPath!==NODE||process.version!=='v26.11.0')throw new Error('unpinned Node runtime');
  for(const p of [TASK_ROOT,EVIDENCE,RUN_ROOT,SOURCE,CARGO_HOME,TARGET])noSymlinkComponents(p);
  physicalDir(TASK_ROOT,0o700);physicalDir(EVIDENCE,0o700);physicalDir(RUN_ROOT,0o700);physicalDir(TARGET,0o755);
  const packet=readJson(PACKET_PATH);
  if(sha(PACKET_PATH)!==PACKET_SHA||packet.schema!=='termrock.vis11.prerelease-v2-rust-nextest-affected-gate/v1'||packet.status!=='FROZEN_PREPARED_NOT_RUN'||packet.qualification!=='NOT_QUALIFIED')throw new Error('frozen packet identity/status mismatch');
  const inputs=verifyPinnedInputs(packet);const source=verifySource(packet);const cache=verifyImmutableCache(packet);const commands=buildCommands(packet);const globalCache=snapshotMutableGlobalCache();
  if(exists(OUTPUT))throw new Error('execution output directory already exists');
  for(const p of [OUTPUT,TARGET])for(const root of [SOURCE,CARGO_HOME])if(pathInside(p,root)||pathInside(root,p))throw new Error('output/target overlaps source or cache');
  if(pathInside(OUTPUT,TARGET)||pathInside(TARGET,OUTPUT))throw new Error('execution output overlaps target');
  if(path.dirname(OUTPUT)!==RUN_ROOT)throw new Error('execution output must be direct run-root child');
  const envSummary={inherited_environment:[],cargo_home:CARGO_HOME,cargo_target_dir:TARGET,cargo_net_offline:true,cargo_build_jobs:2,rust_toolchain:RUST_TOOLCHAIN,mise_data_isolated:true,mise_auto_install_disabled:true,path_order:['private tool-bin','private Mise alias','Nextest 0.9.146','Rust 1.98.1','/usr/bin','/bin','/usr/sbin','/sbin']};
  return {packet,commands,source,cache,globalCache,inputs,mode,environment:envSummary};
}
function parseSummary(text) {
  const lines=text.split(/\r?\n/).map(line=>line.trim()).filter(line=>line.startsWith('Summary '));
  const m=(lines.at(-1)||'').match(/Summary\s+\[[^\]]+\]\s+(\d+) tests? run:\s+(\d+) passed(?:,\s+(\d+) failed)?/);
  return {lines,executed:m?Number(m[1]):null,passed:m?Number(m[2]):null,failed:m?Number(m[3]||0):null};
}
function passedIds(text,tests) {
  const found=[];
  for(const line of text.split(/\r?\n/)){if(!/^\s*PASS\s+\[[^\]]+\]\s+\(\s*\d+\/\d+\)\s+/.test(line))continue;for(const test of tests)if(line.trimEnd().endsWith(' '+test.id))found.push(test.id);}
  return found.sort();
}
function atomicJson(file,value) {
  const tmp=file+'.tmp';const bytes=Buffer.from(JSON.stringify(value,null,2)+'\n');const fd=fs.openSync(tmp,'wx',0o600);try{fs.writeFileSync(fd,bytes);fs.fsyncSync(fd);}finally{fs.closeSync(fd);}fs.renameSync(tmp,file);fs.chmodSync(file,0o600);return{path:file,bytes:bytes.length,sha256:shaBytes(bytes)};
}
function createOutputRoots(packet) {
  fs.mkdirSync(OUTPUT,{mode:0o700});const runtime=path.join(OUTPUT,'runtime');fs.mkdirSync(runtime,{mode:0o700});
  for(const rel of ['home','tmp','mise-data','mise-cache','mise-state','config','bin','tool-bin'])fs.mkdirSync(path.join(runtime,rel),{mode:0o700});
  const copied=[];
  for(const seed of packet.runtime.runtime_seed_files){const dest=path.join(runtime,seed.relative_path);fs.mkdirSync(path.dirname(dest),{recursive:true,mode:0o700});const bytes=fs.readFileSync(seed.path);if(bytes.length!==seed.bytes||shaBytes(bytes)!==seed.sha256)throw new Error('runtime seed changed during copy: '+seed.relative_path);const fd=fs.openSync(dest,'wx',parseInt(seed.mode,8));try{fs.writeFileSync(fd,bytes);fs.fsyncSync(fd);}finally{fs.closeSync(fd);}fs.chmodSync(dest,parseInt(seed.mode,8));copied.push({relative_path:seed.relative_path,bytes:seed.bytes,sha256:seed.sha256,mode:seed.mode});}
  const alias=path.join(runtime,'bin/mise');fs.symlinkSync(MISE,alias);if(fs.readlinkSync(alias)!==MISE)throw new Error('Mise alias target mismatch');
  for(const rel of ['home','tmp','mise-data','mise-cache','mise-state','config','bin','tool-bin'])physicalDir(path.join(runtime,rel),0o700);
  return{execution:OUTPUT,runtime_root:runtime,created_seed_files:copied,mise_alias:{path:alias,target:MISE,sha256:sha(MISE)},existing_warm_target:TARGET};
}
function makeEnv() {
  const runtime=path.join(OUTPUT,'runtime');const pathEntries=[path.join(runtime,'tool-bin'),path.join(runtime,'bin'),path.dirname(NEXTEST),RUST_BIN,'/usr/bin','/bin','/usr/sbin','/sbin'];
  return {CARGO_HOME,CARGO_TARGET_DIR:TARGET,CARGO_BUILD_JOBS:'2',CARGO_NET_OFFLINE:'true',CARGO_TERM_COLOR:'never',CARGO:RUST_BIN+'/cargo',RUSTC,RUSTDOC,RUSTUP_HOME,RUSTUP_TOOLCHAIN:RUST_TOOLCHAIN,HOME:path.join(runtime,'home'),TMPDIR:path.join(runtime,'tmp'),TMP:path.join(runtime,'tmp'),TEMP:path.join(runtime,'tmp'),PATH:pathEntries.join(path.delimiter),NO_COLOR:'1',TERM:'dumb',LANG:'C.UTF-8',LC_ALL:'C.UTF-8',TZ:'UTC',NEXTEST_EXPERIMENTAL_LIBTEST_JSON:'1',GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_NO_LAZY_FETCH:'1',GIT_NO_REPLACE_OBJECTS:'1',GIT_OPTIONAL_LOCKS:'0',GIT_TERMINAL_PROMPT:'0',MISE_DATA_DIR:path.join(runtime,'mise-data'),MISE_CACHE_DIR:path.join(runtime,'mise-cache'),MISE_STATE_DIR:path.join(runtime,'mise-state'),MISE_GLOBAL_CONFIG_FILE:path.join(runtime,'config/global.toml'),MISE_NO_CONFIG:'1',MISE_NO_ENV:'1',MISE_NO_HOOKS:'1',MISE_LOCKFILE:'0',MISE_AUTO_INSTALL:'false',MISE_EXEC_AUTO_INSTALL:'false',MISE_CARGO_HOME:CARGO_HOME,MISE_RUSTUP_HOME:RUSTUP_HOME};
}
function writeAll(fd,bytes){let off=0;while(off<bytes.length){const n=fs.writeSync(fd,bytes,off,bytes.length-off);if(n<=0)throw new Error('short log write');off+=n;}}
function logSnapshot(file,complete){try{const st=fs.lstatSync(file);if(!st.isFile()||st.isSymbolicLink())throw new Error('not regular');return{path:file,bytes:st.size,sha256:sha(file),complete};}catch(e){return{path:file,bytes:null,sha256:null,complete:false,error:String(e&&e.stack||e)};}}
function signalGroup(pid,signal){try{process.kill(-pid,signal);return{sent:true,error:null};}catch(e){return{sent:false,error:String(e&&e.message||e)};}}
let activeStop=null,parentSignal=null;for(const sig of ['SIGTERM','SIGINT'])process.on(sig,()=>{parentSignal=sig;if(activeStop)activeStop('supervisor_'+sig.toLowerCase(),false);});
function runOne(command,deadline,env,budget){return new Promise(resolve=>{
  const logPath=path.join(OUTPUT,'command-'+command.index+'.log'),terminalPath=path.join(OUTPUT,'command-'+command.index+'-terminal.json');
  const started=new Date().toISOString();let fd=null,child=null,launchError=null,exitCode=null,signal=null,exitEvent=false,closeEvent=false,timedOut=false,interrupted=false,reason=null,finished=false,termTimer=null,closeTimer=null,deadlineTimer=null,overflow=false,writeError=null,commandStored=0,commandObserved=0;const signals=[];
  const base=()=>({index:command.index,label:command.label,package:command.package,expected_tests:command.expected_count,test_ids:command.tests.map(t=>t.id),executable:CARGO,argv:command.argv,cwd:SOURCE,started_utc:started,pid:child?child.pid:null,exit_code:exitCode,signal,timed_out:timedOut,interrupted,terminal_reason:reason,launch_error:launchError,exit_event_received:exitEvent,close_event_received:closeEvent,kill_signals:[...signals],output_capture:{gate_cap_bytes:LIMITS.output_bytes,stored_gate_bytes:budget.stored,observed_gate_bytes:budget.observed,stored_command_bytes:commandStored,observed_command_bytes:commandObserved,limit_exceeded:overflow||budget.exceeded,write_error:writeError}});
  const terminal=value=>{try{return atomicJson(terminalPath,value);}catch(e){return{path:terminalPath,error:String(e&&e.stack||e)};}};
  try{fd=fs.openSync(logPath,'wx',0o600);}catch(e){const result={...base(),status:'log_open_failed',summary_lines:[],measured_counts:{executed:null,passed:null,failed:null},passed_test_ids:[],counts_exact:false,output:logSnapshot(logPath,false)};result.terminal_receipt=terminal(result);return resolve(result);}
  try{child=spawn(CARGO,command.argv.slice(1),{cwd:SOURCE,env,detached:true,stdio:['ignore','pipe','pipe']});}catch(e){launchError=String(e&&e.stack||e);}
  const closeFd=()=>{if(fd!==null){try{fs.fsyncSync(fd);}catch{}try{fs.closeSync(fd);}catch{}fd=null;}};
  const clearTimers=()=>{if(deadlineTimer)clearTimeout(deadlineTimer);if(termTimer)clearTimeout(termTimer);if(closeTimer)clearTimeout(closeTimer);};
  function finalize(closeExpired){if(finished)return;finished=true;closeEvent=!closeExpired&&closeEvent;clearTimers();if(child&&child.pid)signals.push({signal:'SIGKILL',...signalGroup(child.pid,'SIGKILL'),reason:'final_group_cleanup'});closeFd();const out=logSnapshot(logPath,closeEvent&&!overflow&&!budget.exceeded&&writeError===null);const text=out.complete?fs.readFileSync(logPath,'utf8'):'';const summary=out.complete?parseSummary(text):{lines:[],executed:null,passed:null,failed:null};const pass=out.complete?passedIds(text,command.tests):[];const within=Date.now()<deadline;const exact=out.complete&&closeEvent&&exitCode===0&&!timedOut&&!interrupted&&!launchError&&!overflow&&!budget.exceeded&&writeError===null&&summary.executed===command.expected_count&&summary.passed===command.expected_count&&summary.failed===0&&canonical(pass)===canonical(command.tests.map(t=>t.id).sort())&&within;const result={...base(),finished_utc:new Date().toISOString(),close_event_received:closeEvent,post_kill_close_grace_expired:closeExpired,kill_signals:[...signals],summary_lines:summary.lines,measured_counts:{executed:summary.executed,passed:summary.passed,failed:summary.failed},passed_test_ids:pass,counts_exact:exact,output:out,status:closeExpired?'post_kill_close_grace_expired':overflow||budget.exceeded?'output_cap_exceeded':writeError?'log_write_failed':timedOut?'timed_out':interrupted?'interrupted':launchError?'spawn_failed':'completed'};result.terminal_receipt=terminal(result);process.stdout.write(JSON.stringify({type:'child_finished',index:command.index,package:command.package,exit_code:exitCode,counts_exact:exact,terminal_receipt_error:result.terminal_receipt.error||null})+'\n');resolve(result);}
  if(!child){closeFd();const result={...base(),finished_utc:new Date().toISOString(),status:'spawn_failed',summary_lines:[],measured_counts:{executed:null,passed:null,failed:null},passed_test_ids:[],counts_exact:false,output:logSnapshot(logPath,true)};result.terminal_receipt=terminal(result);return resolve(result);}
  process.stdout.write(JSON.stringify({type:'child_started',index:command.index,package:command.package,pid:child.pid,started_utc:started})+'\n');
  function stop(why,isTimeout){if(finished||reason)return;reason=why;timedOut=Boolean(isTimeout);interrupted=!timedOut;signals.push({signal:'SIGTERM',...signalGroup(child.pid,'SIGTERM'),reason:why});termTimer=setTimeout(()=>{signals.push({signal:'SIGKILL',...signalGroup(child.pid,'SIGKILL'),reason:'term_grace_expired'});closeTimer=setTimeout(()=>finalize(true),LIMITS.close_grace_ms);},LIMITS.term_grace_ms);}
  activeStop=stop;
  function capture(chunk){commandObserved+=chunk.length;budget.observed+=chunk.length;const allowed=Math.max(0,LIMITS.output_bytes-budget.stored);const selected=chunk.subarray(0,Math.min(chunk.length,allowed));if(selected.length){try{writeAll(fd,selected);commandStored+=selected.length;budget.stored+=selected.length;}catch(e){writeError=String(e&&e.stack||e);stop('output_log_write_failed',false);}}if(selected.length!==chunk.length&&!overflow){overflow=true;budget.exceeded=true;budget.limit_trigger_command=command.index;stop('combined_gate_output_cap_exceeded',false);}}
  child.stdout.on('data',capture);child.stderr.on('data',capture);child.once('error',e=>{launchError=String(e&&e.stack||e);});child.once('exit',(code,sig)=>{exitEvent=true;exitCode=code;signal=sig;});child.once('close',(code,sig)=>{if(finished)return;closeEvent=true;if(!exitEvent){exitCode=code;signal=sig;}clearTimers();finalize(false);});deadlineTimer=setTimeout(()=>stop('whole_gate_deadline',true),Math.max(0,deadline-Date.now()));if(parentSignal)stop('supervisor_'+parentSignal.toLowerCase(),false);
});}
function inventoryRuntime(root,rel='',result={}){for(const name of fs.readdirSync(path.join(root,rel)).sort()){const child=rel?rel+'/'+name:name,abs=path.join(root,child),st=fs.lstatSync(abs);if(st.isSymbolicLink())result[child]={type:'symlink',target:fs.readlinkSync(abs),mode:st.mode&0o777};else if(st.isDirectory()){result[child]={type:'directory',mode:st.mode&0o777};inventoryRuntime(root,child,result);}else if(st.isFile())result[child]={type:'file',size:st.size,sha256:sha(abs),mode:st.mode&0o777};else result[child]={type:'special',mode:st.mode&0o777};}return result;}
async function execute(packet,commands,preflight){const start=Date.now(),deadline=start+LIMITS.deadline_ms;const sourceBefore=verifySource(packet);const cacheBefore=verifyImmutableCache(packet);const mutableBefore=snapshotMutableGlobalCache();const roots=createOutputRoots(packet);const env=makeEnv();const ready={schema:'termrock.vis11.prerelease-v2-nextest-ready/v1',status:'STARTED_AFTER_ROOT_GRANT',qualification:'NOT_QUALIFIED',started_utc:new Date(start).toISOString(),deadline_utc:new Date(deadline).toISOString(),packet_sha256:PACKET_SHA,runner_sha256:sha(__filename),source:sourceBefore,immutable_cache_before:cacheBefore,mutable_global_cache_before:mutableBefore,output_roots:roots,commands:commands.map(c=>({index:c.index,package:c.package,label:c.label,expected_count:c.expected_count,argv:c.argv,cwd:SOURCE})),child_environment:env,limits:LIMITS};const readyFile=atomicJson(path.join(OUTPUT,'ready.json'),ready);const budget={stored:0,observed:0,exceeded:false,limit_trigger_command:null};const results=[];
  for(const command of commands){if(Date.now()>=deadline||parentSignal||budget.exceeded){results.push({index:command.index,label:command.label,package:command.package,status:'not_started_after_gate_stop',expected_tests:command.expected_count,test_ids:command.tests.map(t=>t.id),argv:command.argv,cwd:SOURCE,counts_exact:false});continue;}let sourceImmediatelyBefore;try{sourceImmediatelyBefore=verifySource(packet);}catch(e){parentSignal=parentSignal||'SOURCE_PRECHILD_VERIFICATION_FAILURE';results.push({index:command.index,label:command.label,package:command.package,status:'not_started_source_guard_failed',expected_tests:command.expected_count,test_ids:command.tests.map(t=>t.id),argv:command.argv,cwd:SOURCE,source_before:{ok:false,error:String(e&&e.stack||e)},counts_exact:false});continue;}const result=await runOne(command,deadline,env,budget);result.source_before={ok:true,closure:sourceImmediatelyBefore};let post;try{post={ok:true,closure:verifySource(packet)};}catch(e){post={ok:false,error:String(e&&e.stack||e)};}result.source_after=post.ok?{ok:true,full_tree_sha256:packet.source.full_tree.sha256,entries:post.closure.full_tree_entries}:post;result.source_changed_or_unverifiable=!post.ok;result.command_pass=result.source_before&&result.source_before.ok===true&&result.counts_exact===true&&result.terminal_receipt&&!result.terminal_receipt.error&&post.ok&&Date.now()<deadline;const final=atomicJson(path.join(OUTPUT,'command-'+command.index+'-receipt.json'),result);result.final_receipt=final;results.push(result);if(!post.ok)parentSignal=parentSignal||'SOURCE_POSTFLIGHT_FAILURE';}
  let sourceAfterAll,cacheAfter,seedsAfter;try{sourceAfterAll={ok:true,closure:verifySource(packet)};}catch(e){sourceAfterAll={ok:false,error:String(e&&e.stack||e)};}try{cacheAfter={ok:true,closure:verifyImmutableCache(packet)};}catch(e){cacheAfter={ok:false,error:String(e&&e.stack||e)};}try{seedsAfter={ok:true,closure:verifyRuntimeSeeds(packet)};}catch(e){seedsAfter={ok:false,error:String(e&&e.stack||e)};}const mutableAfter=snapshotMutableGlobalCache();const runtimeTree=inventoryRuntime(path.join(OUTPUT,'runtime'));const passed=results.filter(r=>r.command_pass===true).reduce((n,r)=>n+r.expected_tests,0);const executed=results.reduce((n,r)=>n+(r.measured_counts&&Number.isInteger(r.measured_counts.executed)?r.measured_counts.executed:0),0);const allPass=results.length===6&&results.every(r=>r.command_pass===true)&&passed===48&&executed===48&&sourceAfterAll.ok&&cacheAfter.ok&&seedsAfter.ok&&Date.now()<deadline&&!parentSignal&&!budget.exceeded&&canonical(cacheBefore)===canonical(cacheAfter.closure);
  const receipt={schema:'termrock.vis11.prerelease-v2-rust-nextest-gate-receipt/v1',status:allPass?'all_selected_cases_passed':'failed_or_incomplete_preserved',qualification:'NOT_QUALIFIED',scope:'48 selected Rust Nextest cases only; no full-workspace or product/release qualification.',packet_sha256:PACKET_SHA,runner_sha256:sha(__filename),source_root:SOURCE,source_before:sourceBefore,source_after:sourceAfterAll.ok?{ok:true,full_tree_sha256:packet.source.full_tree.sha256,entries:sourceAfterAll.closure.full_tree_entries}:sourceAfterAll,immutable_cache_before:cacheBefore,immutable_cache_after:cacheAfter.ok?cacheAfter.closure:cacheAfter,mutable_global_cache_before:mutableBefore,mutable_global_cache_after:mutableAfter,runtime_seed_after:seedsAfter,runtime_output_tree:runtimeTree,selected_tests_expected:48,selected_tests_executed:executed,selected_tests_passed:passed,selected_tests_failed_or_unverified:48-passed,command_count_expected:6,commands:results,raw_gate_output:{combined_cap_bytes:LIMITS.output_bytes,stored_bytes:budget.stored,observed_bytes:budget.observed,limit_exceeded:budget.exceeded,limit_trigger_command:budget.limit_trigger_command},deadline:{started_utc:new Date(start).toISOString(),deadline_utc:new Date(deadline).toISOString(),finished_utc:new Date().toISOString(),elapsed_ms:Date.now()-start,limit_ms:LIMITS.deadline_ms,within_deadline:Date.now()<deadline},ready_receipt:readyFile,metadata_only_result_not_counted:true,nonclaims:['Prior 35-case results remain preserved and are not counted here.','No result counts without exact raw PASS rows, summary counts, exit 0, and closed child process.','No full-workspace green, CLI/generator qualification, release qualification, or Git identity claim.']};const final=atomicJson(path.join(OUTPUT,'receipt.json'),receipt);process.stdout.write(JSON.stringify({type:'gate_finished',status:receipt.status,selected:48,executed,passed,elapsed_ms:receipt.deadline.elapsed_ms,receipt_sha256:final.sha256})+'\n');return allPass?0:1;}
(async()=>{try{const prepared=verifyStatic(process.argv[2]);const packet=prepared.packet,commands=prepared.commands;const preflight={schema:'termrock.vis11.prerelease-v2-nextest-preflight/v1',status:'READY_FOR_INDEPENDENT_REVIEW_AND_ROOT_EXECUTION_DECISION_NOT_RUN',qualification:'NOT_QUALIFIED',packet_path:PACKET_PATH,packet_sha256:PACKET_SHA,runner_path:__filename,runner_sha256:sha(__filename),node:{path:NODE,version:process.version,sha256:sha(NODE)},source:prepared.source,immutable_cache:prepared.cache,mutable_global_cache:prepared.globalCache,checked_external_inputs:prepared.inputs,commands:commands.map(c=>({index:c.index,package:c.package,label:c.label,expected_count:c.expected_count,test_ids:c.tests.map(t=>t.id),argv:c.argv,cwd:SOURCE})),output_roots:{execution:OUTPUT,execution_absent:true,target:TARGET,target_preexisting:true},child_environment:prepared.environment,limits:LIMITS,action:'No Cargo, Nextest, build, test, CLI, or workflow generation ran during preflight.'};if(process.argv[2]==='--preflight'){process.stdout.write(JSON.stringify(preflight,null,2)+'\n');return;}const code=await execute(packet,commands,preflight);process.exitCode=code;}catch(error){const report={schema:'termrock.vis11.prerelease-v2-nextest-runner-error/v1',status:'preflight_or_runner_error',qualification:'NOT_QUALIFIED',error:String(error&&error.stack||error),execution_started:exists(OUTPUT)};process.stderr.write(JSON.stringify(report,null,2)+'\n');process.exitCode=2;}})();
