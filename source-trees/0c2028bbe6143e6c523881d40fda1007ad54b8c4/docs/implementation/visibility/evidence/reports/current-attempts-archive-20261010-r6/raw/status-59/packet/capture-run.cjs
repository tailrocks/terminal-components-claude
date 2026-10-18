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
const CACHE_PATH = path.join(PACKET, "cache-closure.json");
const REQUEST_PATH = path.join(PACKET, "review-request.json");
const SUMS_PATH = path.join(PACKET, "SHA256SUMS");
const PREFLIGHT_DIR = path.join(PACKET, "preflight");
const PREFLIGHT_PATH = path.join(PREFLIGHT_DIR, "preflight.json");
const sha = bytes => crypto.createHash("sha256").update(bytes).digest("hex");
const shaFile = file => sha(fs.readFileSync(file));
const json = file => JSON.parse(fs.readFileSync(file, "utf8"));
const stable = value => JSON.stringify(value);
let eventWriteErrors = null;
let eventLogFailed = false;
function packetInputPaths() {
  return {
    selection_v2:path.join(PACKET,"selection-v2.json"),
    target_seed:path.join(PACKET,"target-seed.json"),
    list_capture_binding:path.join(PACKET,"list-capture-binding.json"),
    list_stdout:path.join(PACKET,"list-capture","raw","list.stdout.json"),
    list_stderr:path.join(PACKET,"list-capture","raw","list.stderr.log"),
    list_process:path.join(PACKET,"list-capture","raw","list-process.json"),
    runner:__filename,
    runplan:PLAN_PATH,
    source_manifest:SOURCE_PATH,
    test_names:TESTS_PATH,
    cache_closure:CACHE_PATH,
    source_freeze:path.join(PACKET,"source-freeze.json"),
    source_delta_patch:path.join(PACKET,"source-delta.patch"),
    inventory_binding:path.join(PACKET,"inventory-binding.json"),
    review_request:REQUEST_PATH,
    packet_sums:SUMS_PATH,
    preflight_source:path.join(PREFLIGHT_DIR,"source-before.json"),
    preflight_tool:path.join(PREFLIGHT_DIR,"tool-before.json"),
    preflight_cache:path.join(PREFLIGHT_DIR,"cache-before.json"),
    preflight_target:path.join(PREFLIGHT_DIR,"target-before.json"),
    preflight_receipt:PREFLIGHT_PATH
  };
}
function packetInputSnapshot(paths=packetInputPaths()) {
  const snapshot={};
  for (const [name,file] of Object.entries(paths)) {
    try { snapshot[name]=shaFile(file); }
    catch (error) { snapshot[name]=null; }
  }
  return snapshot;
}
function staticPacketInputPaths() {
  return Object.fromEntries(Object.entries(packetInputPaths()).filter(([name])=>!name.startsWith("preflight_")));
}
function staticPacketInputSnapshot() { return packetInputSnapshot(staticPacketInputPaths()); }
function staticPinsFrom(snapshot) {
  const selected={};
  for (const name of Object.keys(staticPacketInputPaths())) selected[name]=snapshot[name]===undefined?null:snapshot[name];
  return selected;
}
function packetInputChanges(before,after) {
  return Object.keys(before).filter(name=>before[name]===null || after[name]===null || before[name]!==after[name]);
}
function fail(message) { throw new Error(message); }
function writeAll(fd, bytes) {
  let written=0;
  while (written < bytes.length) {
    const remaining=bytes.length-written;
    let count;
    try { count=fs.writeSync(fd,bytes,written,remaining); }
    catch (cause) {
      const error=new Error("file write failed: "+String(cause));
      error.bytesWritten=written;
      error.cause=cause;
      throw error;
    }
    if (!Number.isSafeInteger(count) || count <= 0 || count > remaining) {
      const error=new Error("invalid or zero-byte file write result: " + String(count));
      error.bytesWritten=written;
      throw error;
    }
    written+=count;
  }
  return written;
}
function absent(file) {
  try { fs.lstatSync(file); return false; }
  catch (error) { if (error.code === "ENOENT") return true; throw error; }
}
function regular(file, expected, label) {
  const st = fs.lstatSync(file);
  if (!st.isFile() || st.isSymbolicLink()) fail(label + " is not a regular file");
  const bytes = fs.readFileSync(file);
  const digest = sha(bytes);
  if (digest !== expected) fail(label + " SHA-256 mismatch: " + digest);
  return {path:file, bytes:bytes.length, sha256:digest, mode:st.mode & 0o7777};
}
function writeJson(file, value) {
  const data = Buffer.from(JSON.stringify(value, null, 2) + "\n");
  const fd = fs.openSync(file, "wx", 0o600);
  try { writeAll(fd, data); } finally { fs.closeSync(fd); }
  fs.chmodSync(file, 0o600);
}
function privateDir(file, mustExist) {
  if (absent(file)) {
    if (mustExist) fail("required private directory absent: " + file);
    fs.mkdirSync(file, {recursive:false, mode:0o700});
    fs.chmodSync(file, 0o700);
  }
  const st = fs.lstatSync(file);
  if (!st.isDirectory() || st.isSymbolicLink() || (st.mode & 0o077) !== 0) fail("directory is not private/real: " + file);
}
function sourceAudit(input) {
  const entries = [];
  for (const item of input.source.build_input_files) {
    const result = regular(path.join(input.plan.cwd, item.path), item.sha256, "source " + item.path);
    if (result.bytes !== item.bytes) fail("source byte count mismatch: " + item.path);
    entries.push({path:item.path, bytes:result.bytes, sha256:result.sha256, mode:result.mode});
  }
  for (const proof of input.source.audit_proofs || []) {
    const result = regular(proof.path, proof.sha256, proof.label);
    entries.push({path:proof.path, bytes:result.bytes, sha256:result.sha256, mode:result.mode});
  }
  entries.sort((a,b) => a.path.localeCompare(b.path));
  const body = {schema:"termrock.nextest.source-audit.v1", source_root:input.plan.cwd, base_commit:input.source.base_publication.commit, files:entries};
  body.digest = sha(Buffer.from(stable(body)));
  return body;
}
function toolAudit(input) {
  const selected = [
    ["cargo", input.source.toolchain.cargo],
    ["rustc", input.source.toolchain.rustc],
    ["nextest", input.source.toolchain.nextest],
    ["node", input.source.toolchain.node]
  ];
  const tools = selected.map(([name,item]) => regular(item.path, item.sha256, name));
  const body = {schema:"termrock.nextest.tool-audit.v1", tools:tools.sort((a,b)=>a.path.localeCompare(b.path))};
  body.digest = sha(Buffer.from(stable(body)));
  return body;
}
function cacheAudit(input) {
  const entries = [];
  for (const item of input.cache.archives) entries.push(regular(item.archive.path, item.checksum, "archive " + item.name + " " + item.version));
  for (const item of input.cache.indexes) entries.push(regular(item.path, item.sha256, "sparse index " + item.name));
  entries.sort((a,b)=>a.path.localeCompare(b.path));
  const body = {schema:"termrock.nextest.cache-audit.v1", cargo_home:input.cache.cargo_home, package_lock_sha256:input.cache.package_lock_sha256, archive_count:input.cache.archives.length, index_count:input.cache.indexes.length, entries};
  body.digest = sha(Buffer.from(stable(body)));
  return body;
}
function targetSnapshot(root) {
  const st = fs.lstatSync(root);
  if (!st.isDirectory() || st.isSymbolicLink() || (st.mode & 0o077)!==0) fail("target root is not a private real directory");
  const entries=[];
  let bytesTotal=0;
  function visit(abs,rel) {
    const before=fs.lstatSync(abs), mode=before.mode&0o7777;
    if (before.isSymbolicLink()) { entries.push({path:rel,kind:"symlink",mode,target:fs.readlinkSync(abs)}); return; }
    if (before.isDirectory()) {
      entries.push({path:rel,kind:"directory",mode});
      for (const name of fs.readdirSync(abs).sort()) visit(path.join(abs,name),rel?rel+"/"+name:name);
      return;
    }
    if (before.isFile()) {
      const data=fs.readFileSync(abs), after=fs.lstatSync(abs);
      if (!after.isFile() || after.isSymbolicLink() || after.dev!==before.dev || after.ino!==before.ino || after.size!==before.size || after.mtimeMs!==before.mtimeMs || data.length!==before.size) fail("target changed while snapshotting: "+rel);
      bytesTotal+=data.length;
      entries.push({path:rel,kind:"file",mode,bytes:data.length,sha256:sha(data)});
      return;
    }
    fail("unsupported target entry: "+rel);
  }
  visit(root,"");
  entries.sort((a,b)=>a.path.localeCompare(b.path));
  const body = {schema:"termrock.vis01.target-tree-snapshot/v1", target_root:root, root_mode:st.mode & 0o7777, entry_count:entries.length, file_count:entries.filter(entry=>entry.kind==="file").length, bytes_total:bytesTotal, entries_sha256:sha(Buffer.from(stable(entries))), entries};
  body.digest = sha(Buffer.from(stable(body)));
  return body;
}
function verify() {
  if (process.platform !== "darwin") fail("pinned raw test-run capture requires macOS");
  const plan = json(PLAN_PATH), source = json(SOURCE_PATH), tests = json(TESTS_PATH), cache = json(CACHE_PATH), request = json(REQUEST_PATH), inventoryBinding = json(path.join(PACKET,"inventory-binding.json")), selection = json(path.join(PACKET,"selection-v2.json")), targetSeed = json(path.join(PACKET,"target-seed.json")), listBinding = json(path.join(PACKET,"list-capture-binding.json"));
  if (plan.schema !== "termrock.vis01.product-phases-nextest-raw-runplan.v1" || plan.status !== "PREPARED_FOR_ROOT_ONLY_RUN; NOT_RUN" || plan.owner !== "Root only") fail("plan schema/status/owner mismatch");
  if (tests.schema !== "termrock.nextest-target-inventory.r3" || tests.package_name !== "termrock-visibility-tests" || tests.targets.integration.name !== "status" || tests.test_names.length !== 59 || new Set(tests.test_names).size !== 59 || tests.targets.integration.source_test_count !== 59 || tests.targets.integration.expected_test_count !== 59 || stable(tests.test_names) !== stable(tests.targets.integration.source_test_names)) fail("static 59-test selection inventory mismatch");
  if (inventoryBinding.schema !== "termrock-vis01-nextest-static-inventory-binding/v1" || inventoryBinding.status_test_source.path !== tests.targets.integration.source_path || inventoryBinding.status_test_source.sha256 !== "7efa372f27b1537cbdf5add2568e62c43d43c42d7a30f29c1fc2e1b661440e0f" || inventoryBinding.status_test_source.source_test_count !== 59 || inventoryBinding.status_test_source.ignored_tests !== 0 || inventoryBinding.selection.package !== tests.package_name || inventoryBinding.selection.target !== "--test status" || inventoryBinding.selection.filter !== null || inventoryBinding.selection.expected_selected_tests !== 59) fail("source-derived static inventory binding mismatch");
  if (source.source_root !== plan.cwd || source.base_publication.commit !== plan.base_commit || source.base_publication.tree !== plan.base_tree) fail("source base/path binding mismatch");
  if (plan.target_dir !== path.join(PACKET,"target-run") || plan.output_root !== path.join(PACKET,"run-output")) fail("target/output paths are not packet-local isolated paths");
  if (shaFile(path.join(PACKET,"selection-v2.json")) !== plan.bindings.selection_v2_sha256 || shaFile(path.join(PACKET,"target-seed.json")) !== plan.bindings.target_seed_sha256 || shaFile(path.join(PACKET,"list-capture-binding.json")) !== plan.bindings.list_capture_binding_sha256) fail("selection/target/list evidence pin mismatch");
  if (selection.schema !== "termrock-nextest-selection/v2" || selection.expected_test_count !== 59 || selection.filter_args.length !== 0 || selection.run_ignored !== "default" || selection.tests.length !== 59 || selection.targets.length !== 1 || selection.targets[0].package !== tests.package_name || selection.targets[0].binary_id !== "termrock-visibility-tests::status" || selection.targets[0].kind !== "test" || selection.tests.some((test,index)=>test.test_name!==tests.test_names[index] || test.package!==tests.package_name || test.binary_id!=="termrock-visibility-tests::status" || test.kind!=="test" || test.ignored!==false || test.filter_match_status!=="matches")) fail("Rust reader v2 selection differs from exact source inventory");
  if (targetSeed.schema !== "termrock.vis01.status-target-seed/v1" || targetSeed.clone.path !== plan.target_dir || targetSeed.clone.digest !== targetSeed.snapshot.digest || targetSeed.clone.root_mode !== targetSeed.snapshot.root_mode || targetSeed.clone.entry_count !== targetSeed.snapshot.entry_count || targetSeed.clone.file_count !== targetSeed.snapshot.file_count || targetSeed.clone.bytes_total !== targetSeed.snapshot.bytes_total || targetSeed.clone.entries_sha256 !== targetSeed.snapshot.entries_sha256 || targetSeed.snapshot.schema !== "termrock.vis01.target-tree-snapshot/v1" || targetSeed.snapshot.target_root !== plan.target_dir || targetSeed.source_binding.base_commit !== plan.base_commit || targetSeed.source_binding.base_tree !== plan.base_tree || targetSeed.source_binding.status_source_sha256 !== inventoryBinding.status_test_source.sha256 || targetSeed.source_binding.list_process_record_sha256 !== listBinding.process_record.sha256) fail("target seed/list/source binding mismatch");
  if (listBinding.status !== "INDEPENDENT_ACTUAL_LIST_REVIEW_READY; TEST_BODIES_NOT_RUN" || listBinding.expected.count !== 59 || listBinding.expected.filter_args.length !== 0 || listBinding.expected.test_bodies_run !== false || listBinding.review.sha256 !== "8b9c6ec84840da137cdbae5f1a571e39059f781b891268038148d19c1dddf1d0" || shaFile(listBinding.process_record.path) !== listBinding.process_record.sha256 || shaFile(listBinding.captures.stdout.path) !== listBinding.captures.stdout.sha256 || shaFile(listBinding.captures.stderr.path) !== listBinding.captures.stderr.sha256) fail("reviewed list capture bytes do not match binding");
  for (const [name,item] of Object.entries(targetSeed.artifacts)) {
    const seeded=regular(path.join(plan.target_dir,item.path),item.sha256,"seeded target artifact "+name);
    if (seeded.bytes!==item.bytes || seeded.mode!==item.mode) fail("seeded target artifact metadata mismatch: "+name);
  }
  if (plan.package_manifest !== path.join(plan.cwd, "crates/termrock-visibility-tests/Cargo.toml")) fail("package manifest path mismatch");
  if (plan.program !== source.toolchain.cargo.path || plan.program_sha256 !== source.toolchain.cargo.sha256 || shaFile(plan.program) !== plan.program_sha256) fail("Cargo pin mismatch");
  if (process.execPath !== source.toolchain.node.path || process.version !== source.toolchain.node.version || shaFile(process.execPath) !== source.toolchain.node.sha256) fail("Node pin mismatch");
  if (shaFile(SOURCE_PATH) !== plan.bindings.source_manifest_sha256 || shaFile(TESTS_PATH) !== plan.bindings.test_names_sha256 || shaFile(CACHE_PATH) !== plan.bindings.cache_closure_sha256) fail("manifest/inventory/cache pin mismatch");
  if (shaFile(path.join(PACKET,"source-freeze.json")) !== plan.bindings.source_freeze_sha256 || shaFile(path.join(PACKET,"source-delta.patch")) !== plan.bindings.source_delta_patch_sha256 || shaFile(path.join(PACKET,"inventory-binding.json")) !== plan.bindings.inventory_binding_sha256) fail("freeze/patch/inventory-binding mismatch");
  if (shaFile(__filename) !== plan.bindings.runner_sha256) fail("runner hash mismatch");
  if (request.schema !== "termrock.nextest.raw-run-review-request/v1" || request.status !== "REQUEST_STATIC_RUNNER_REVIEW; NO_EXECUTION") fail("request status/schema mismatch");
  const pins = request.pins;
  if (pins.runner_sha256 !== shaFile(__filename) || pins.runplan_sha256 !== shaFile(PLAN_PATH) || pins.source_manifest_sha256 !== shaFile(SOURCE_PATH) || pins.test_names_sha256 !== shaFile(TESTS_PATH) || pins.cache_closure_sha256 !== shaFile(CACHE_PATH) || pins.source_freeze_sha256 !== shaFile(path.join(PACKET,"source-freeze.json")) || pins.source_delta_patch_sha256 !== shaFile(path.join(PACKET,"source-delta.patch")) || pins.inventory_binding_sha256 !== shaFile(path.join(PACKET,"inventory-binding.json")) || pins.selection_v2_sha256 !== shaFile(path.join(PACKET,"selection-v2.json")) || pins.target_seed_sha256 !== shaFile(path.join(PACKET,"target-seed.json")) || pins.list_capture_binding_sha256 !== shaFile(path.join(PACKET,"list-capture-binding.json")) || pins.packet_sums_sha256 !== shaFile(SUMS_PATH)) fail("review request does not bind packet bytes");
  if (request.packet_root !== PACKET || request.source_root !== plan.cwd || request.target_inventory_count !== 59 || request.selection_filter !== null || request.selected_target !== "termrock-visibility-tests::status" || request.filter_args.length !== 0 || request.semantic_parse !== "DELEGATED_TO_PINNED_RUST_READER_V2" || request.nextest_spawned !== false || request.tests_run !== false) fail("request target/scope binding mismatch");
  const expectedEnv = ["CARGO_BUILD_JOBS","CARGO_HOME","CARGO_NET_OFFLINE","CARGO_TARGET_DIR","CARGO_TERM_COLOR","HOME","LC_ALL","NEXTEST_EXPERIMENTAL_LIBTEST_JSON","PATH","RUSTC","RUSTUP_HOME","RUSTUP_TOOLCHAIN","TMPDIR"].sort();
  if (stable(Object.keys(plan.environment).sort()) !== stable(expectedEnv)) fail("child env is not exact allowlist");
  const expectedPath = [path.dirname(source.toolchain.nextest.path),path.dirname(source.toolchain.rustc.path),"/usr/local/bin","/opt/homebrew/bin","/usr/bin","/bin"].join(path.delimiter);
  if (plan.environment.CARGO_BUILD_JOBS !== "2" || plan.environment.CARGO_HOME !== cache.cargo_home || plan.environment.CARGO_NET_OFFLINE !== "true" || plan.environment.CARGO_TARGET_DIR !== plan.target_dir || plan.environment.CARGO_TERM_COLOR !== "never" || plan.environment.HOME !== path.join(plan.output_root,"home") || plan.environment.TMPDIR !== path.join(plan.output_root,"scratch") || plan.environment.LC_ALL !== "C.UTF-8" || plan.environment.PATH !== expectedPath || plan.environment.RUSTC !== source.toolchain.rustc.path || plan.environment.RUSTUP_HOME !== "/Users/donbeave/.rustup" || plan.environment.RUSTUP_TOOLCHAIN !== "1.98.1-aarch64-apple-darwin" || plan.environment.NEXTEST_EXPERIMENTAL_LIBTEST_JSON !== "1") fail("environment tool/cache/target/offline/private path pins mismatch");
  if (plan.limits.parent_environment_inherited !== false || plan.limits.locked !== true || plan.limits.offline !== true || plan.limits.build_jobs !== 2 || plan.limits.test_threads !== 1 || plan.limits.timeout_ms !== 600000 || plan.limits.stream_cap_bytes !== 8388608 || plan.limits.combined_cap_bytes !== 16777216 || plan.limits.term_grace_ms !== 5000 || plan.limits.post_kill_grace_ms !== 1000) fail("test-run process bounds differ from reviewed bounds");
  const expectedArgv = ["nextest","run","--manifest-path",plan.package_manifest,"--locked","--offline","--package",tests.package_name,"--test","status","--build-jobs","2","--test-threads","1","--user-config-file","none","--ignore-default-filter","--no-tests","fail","--no-fail-fast","--retries","0","--flaky-result","fail","--failure-output","immediate-final","--status-level","all","--final-status-level","all","--no-output-indent","--show-progress","none","--color","never","--message-format","libtest-json-plus","--message-format-version","0.1"];
  if (stable(plan.argv) !== stable(expectedArgv)) fail("actual Nextest run argv differs from exact reviewed 59-test target");
  const statusSource = source.build_input_files.find(x=>x.path==="crates/termrock-visibility-tests/tests/status.rs");
  const lockSource = source.build_input_files.find(x=>x.path===cache.package_lock_path);
  const indexConfigProof = source.audit_proofs.find(x=>x.path===cache.index_config.path && x.sha256===cache.index_config.sha256);
  if (source.build_input_files.length !== 108 || !statusSource || statusSource.sha256 !== "7efa372f27b1537cbdf5add2568e62c43d43c42d7a30f29c1fc2e1b661440e0f" || inventoryBinding.status_test_source.sha256 !== statusSource.sha256 || !lockSource || lockSource.sha256 !== cache.package_lock_sha256 || cache.package_lock_path !== "crates/termrock-visibility-tests/Cargo.lock" || !indexConfigProof || source.cache.closure_sha256 !== shaFile(CACHE_PATH) || source.cache.closure_path !== "/private/tmp/termrock-vis01-product-phases-r3-nextest-r5b-20261010/packet/cache-closure.json" || shaFile(source.cache.closure_path) !== source.cache.closure_sha256 || source.cache.cargo_home !== cache.cargo_home) fail("R3 source/cache closure binding mismatch");
  if (cache.archives.length !== 40 || cache.indexes.length !== 44 || cache.package_lock_sha256 !== "a80325ffed5b5cf5b652f9d9e8a859573c91b93def1e59d42b4259b65a586c1f" || cache.publisher_registry_archive_count !== 40 || cache.publisher_sparse_index_file_count !== 44) fail("reviewed R8 cache closure mismatch");
  const sourceState = sourceAudit({plan,source});
  const tools = toolAudit({source});
  const cacheState = cacheAudit({cache});
  const target = targetSnapshot(plan.target_dir);
  if (stable(target)!==stable(targetSeed.snapshot)) fail("full seeded target tree differs from target-seed snapshot");
  if (!target.entries.some(entry=>entry.path==="debug"&&entry.kind==="directory") || !target.entries.some(entry=>entry.path===".rustc_info.json"&&entry.kind==="file")) fail("seeded run target is missing its reviewed Cargo layout");
  if (!absent(plan.output_root)) fail("run output root is not fresh");
  privateDir(PREFLIGHT_DIR, true);
  return {plan,source,tests,cache,request,inventory_binding:inventoryBinding,source_audit:sourceState,tool_audit:tools,cache_audit:cacheState,target_snapshot:target};
}
function preflight() {
  const input = verify();
  if (!absent(PREFLIGHT_PATH)) fail("preflight output already exists");
  const staticPacketPins=staticPacketInputSnapshot();
  if (Object.values(staticPacketPins).some(value=>value===null)) fail("one or more static packet inputs are unreadable at preflight");
  const receipt = {
    schema:"termrock.nextest.raw-run-preflight/v1",
    result:"PREFLIGHT_READY_NO_CARGO",
    runner_path:__filename,
    runner_sha256:shaFile(__filename),
    runplan_sha256:shaFile(PLAN_PATH),
    source_manifest_sha256:shaFile(SOURCE_PATH),
    test_names_sha256:shaFile(TESTS_PATH),
    cache_closure_sha256:shaFile(CACHE_PATH),
    source_freeze_sha256:shaFile(path.join(PACKET,"source-freeze.json")),
    source_delta_patch_sha256:shaFile(path.join(PACKET,"source-delta.patch")),
    packet_sums_sha256:shaFile(SUMS_PATH),
    inventory_binding_sha256:shaFile(path.join(PACKET,"inventory-binding.json")),
    review_request_sha256:shaFile(REQUEST_PATH),
    cargo_spawned:false,nextest_spawned:false,tests_run:false,network_used:false,
    expected_source_test_count:59,selected_filter:null,
    target:{path:input.target_snapshot.target_root,digest:input.target_snapshot.digest,entry_count:input.target_snapshot.entry_count,file_count:input.target_snapshot.file_count,bytes_total:input.target_snapshot.bytes_total,entries_sha256:input.target_snapshot.entries_sha256},
    source_digest:input.source_audit.digest,tool_digest:input.tool_audit.digest,cache_digest:input.cache_audit.digest,
    packet_input_pins:staticPacketPins,
    output_root_absent:true,created_at:new Date().toISOString()
  };
  for (const [name,value] of [["source-before.json",input.source_audit],["tool-before.json",input.tool_audit],["cache-before.json",input.cache_audit],["target-before.json",input.target_snapshot]]) writeJson(path.join(PREFLIGHT_DIR,name),value);
  writeJson(PREFLIGHT_PATH,receipt);
  process.stdout.write(JSON.stringify(receipt,null,2)+"\n");
}
function event(fd, value) {
  if (eventLogFailed) return;
  const bytes=Buffer.from(JSON.stringify({at:new Date().toISOString(),...value})+"\n");
  let beforeSize=null, writeReturned=0;
  try {
    beforeSize=fs.fstatSync(fd).size;
    writeReturned=writeAll(fd,bytes);
    const afterSize=fs.fstatSync(fd).size;
    if (afterSize-beforeSize!==bytes.length || writeReturned!==bytes.length) {
      const error=new Error("process event log byte count mismatch");
      error.bytesWritten=writeReturned;
      error.fileBytesWritten=afterSize-beforeSize;
      throw error;
    }
  } catch (error) {
    eventLogFailed=true;
    let actualBytes=Number.isSafeInteger(error.fileBytesWritten)?error.fileBytesWritten:(Number.isSafeInteger(error.bytesWritten)?error.bytesWritten:writeReturned);
    let afterSize=null;
    if (beforeSize!==null) {
      try {
        afterSize=fs.fstatSync(fd).size;
        const delta=afterSize-beforeSize;
        if (Number.isSafeInteger(delta) && delta>=0) actualBytes=delta;
      } catch (_) {}
    }
    if (eventWriteErrors) eventWriteErrors.push({error:String(error),expected_bytes:bytes.length,write_returned_bytes:writeReturned,actual_bytes:actualBytes,file_size_before:beforeSize,file_size_after:afterSize});
  }
}
function runNextest() {
  const input = verify();
  if (absent(PREFLIGHT_PATH)) fail("preflight is required before run capture");
  privateDir(PREFLIGHT_DIR,true);
  const preStat=fs.lstatSync(PREFLIGHT_PATH);
  if (!preStat.isFile() || preStat.isSymbolicLink() || (preStat.mode & 0o077)!==0) fail("preflight receipt is not a private regular file");
  const pre = json(PREFLIGHT_PATH);
  if (pre.schema !== "termrock.nextest.raw-run-preflight/v1" || pre.result !== "PREFLIGHT_READY_NO_CARGO" || pre.runner_sha256 !== shaFile(__filename) || pre.runplan_sha256 !== shaFile(PLAN_PATH) || pre.source_manifest_sha256 !== shaFile(SOURCE_PATH) || pre.test_names_sha256 !== shaFile(TESTS_PATH) || pre.cache_closure_sha256 !== shaFile(CACHE_PATH) || pre.packet_sums_sha256 !== shaFile(SUMS_PATH) || pre.source_freeze_sha256 !== shaFile(path.join(PACKET,"source-freeze.json")) || pre.source_delta_patch_sha256 !== shaFile(path.join(PACKET,"source-delta.patch")) || pre.inventory_binding_sha256 !== shaFile(path.join(PACKET,"inventory-binding.json")) || pre.review_request_sha256 !== shaFile(REQUEST_PATH) || pre.cargo_spawned || pre.nextest_spawned || pre.tests_run || pre.network_used) fail("preflight does not bind this frozen raw-list packet");
  const expectedStaticNames=Object.keys(staticPacketInputPaths()).sort();
  if (!pre.packet_input_pins || stable(Object.keys(pre.packet_input_pins).sort())!==stable(expectedStaticNames)) fail("preflight static packet pin set is missing or unexpected");
  const packetPinsAfterPreflight=staticPacketInputSnapshot();
  if (Object.values(packetPinsAfterPreflight).some(value=>value===null) || stable(packetPinsAfterPreflight)!==stable(pre.packet_input_pins)) fail("static packet inputs changed since preflight");
  if (!absent(input.plan.output_root)) fail("run output root already exists");
  const targetBefore = targetSnapshot(input.plan.target_dir);
  if (targetBefore.digest !== pre.target.digest || targetBefore.entry_count!==pre.target.entry_count || targetBefore.file_count!==pre.target.file_count || targetBefore.bytes_total!==pre.target.bytes_total || targetBefore.entries_sha256!==pre.target.entries_sha256) fail("run target changed after preflight");
  const sourceBefore = sourceAudit(input), toolsBefore = toolAudit(input), cacheBefore = cacheAudit(input);
  if (sourceBefore.digest !== pre.source_digest || toolsBefore.digest !== pre.tool_digest || cacheBefore.digest !== pre.cache_digest) fail("source/tool/cache changed after preflight");
  const packetPinsBefore=packetInputSnapshot();
  if (Object.values(packetPinsBefore).some(value=>value===null)) fail("one or more packet inputs are unreadable before spawn");
  const staticPinsBeforeSpawn=staticPacketInputSnapshot();
  if (stable(staticPinsBeforeSpawn)!==stable(pre.packet_input_pins)) fail("static packet inputs changed between preflight and spawn");
  let packetPinsAtSpawn=null;
  let targetAtSpawn=null;
  fs.mkdirSync(input.plan.output_root,{mode:0o700}); fs.chmodSync(input.plan.output_root,0o700);
  privateDir(input.plan.output_root,true);
  fs.mkdirSync(input.plan.environment.HOME,{mode:0o700}); fs.chmodSync(input.plan.environment.HOME,0o700);
  fs.mkdirSync(input.plan.environment.TMPDIR,{mode:0o700}); fs.chmodSync(input.plan.environment.TMPDIR,0o700);
  const out = input.plan.output_root;
  const stdoutPath=path.join(out,"nextest.stdout.jsonl"), stderrPath=path.join(out,"nextest.stderr.log"), eventsPath=path.join(out,"process-events.jsonl"), receiptPath=path.join(out,"run-process.json");
  const stdoutFd=fs.openSync(stdoutPath,"wx",0o600), stderrFd=fs.openSync(stderrPath,"wx",0o600), eventFd=fs.openSync(eventsPath,"wx",0o600);
  eventWriteErrors=[];
  const states={stdout:{received:0,stored:0,truncated:false},stderr:{received:0,stored:0,truncated:false}};
  let total=0, child=null, closeEvent=null, spawnError=null, timeout=false, capExceeded=false, reason=null, forcedClose=false, finalized=false;
  const outputWriteErrors=[];
  let timeoutTimer=null,termTimer=null,killTimer=null;
  const signalGroup = sig => {
    if (!child || !child.pid) return;
    try { process.kill(-child.pid,sig); }
    catch (error) { if (error.code!=="ESRCH") { try { child.kill(sig); } catch (_) {} } }
  };
  const stop = why => {
    if (reason || finalized) return;
    reason=why; event(eventFd,{event:"stop",reason:why}); signalGroup("SIGTERM");
    termTimer=setTimeout(()=>{signalGroup("SIGKILL");killTimer=setTimeout(()=>{if(!closeEvent&&!finalized){forcedClose=true;try{if(child)child.kill("SIGKILL");}catch(_){};if(child&&child.stdout)child.stdout.destroy();if(child&&child.stderr)child.stderr.destroy();finalize();}},input.plan.limits.post_kill_grace_ms);},input.plan.limits.term_grace_ms);
  };
  const capture=(which,chunk)=>{
    if(finalized)return;
    const state=states[which];state.received+=chunk.length;
    if(state.write_failed){state.truncated=true;return;}
    const streamRemaining=Math.max(0,input.plan.limits.stream_cap_bytes-state.stored);
    const totalRemaining=Math.max(0,input.plan.limits.combined_cap_bytes-total);
    const keep=Math.min(chunk.length,streamRemaining,totalRemaining);
    if(keep>0){
      const fd=which==="stdout"?stdoutFd:stderrFd;
      let sizeBefore=null, writeReturned=0;
      try{
        sizeBefore=fs.fstatSync(fd).size;
        writeReturned=writeAll(fd,chunk.subarray(0,keep));
        const sizeAfter=fs.fstatSync(fd).size;
        const actualBytes=sizeAfter-sizeBefore;
        if(actualBytes!==keep || writeReturned!==keep){
          const error=new Error("captured stream byte count mismatch");
          error.bytesWritten=writeReturned;
          error.fileBytesWritten=actualBytes;
          throw error;
        }
        state.stored=sizeAfter;total+=actualBytes;
      }catch(error){
        let actualBytes=Number.isSafeInteger(error.fileBytesWritten)?error.fileBytesWritten:(Number.isSafeInteger(error.bytesWritten)?error.bytesWritten:writeReturned);
        let sizeAfter=null;
        if(sizeBefore!==null){
          try{
            sizeAfter=fs.fstatSync(fd).size;
            const delta=sizeAfter-sizeBefore;
            if(Number.isSafeInteger(delta)&&delta>=0)actualBytes=delta;
          }catch(_){state.accounting_known=false;}
        }else{state.accounting_known=false;}
        if(Number.isSafeInteger(actualBytes)&&actualBytes>=0){
          state.stored+=actualBytes;total+=actualBytes;
        }else{state.accounting_known=false;}
        state.write_failed=true;state.truncated=true;
        outputWriteErrors.push({stream:which,error:String(error),requested_bytes:keep,write_returned_bytes:writeReturned,actual_bytes_written:Number.isSafeInteger(actualBytes)&&actualBytes>=0?actualBytes:null,file_size_before:sizeBefore,file_size_after:sizeAfter});
        stop("OUTPUT_WRITE_FAILURE");
      }
    }
    if(keep<chunk.length){state.truncated=true;capExceeded=true;stop("OUTPUT_CAP_EXCEEDED");}
  };
  let parentSignal=null;
  const onInt=()=>{parentSignal="SIGINT";stop("PARENT_SIGINT");}, onTerm=()=>{parentSignal="SIGTERM";stop("PARENT_SIGTERM");};
  process.on("SIGINT",onInt);process.on("SIGTERM",onTerm);
  const finalize=()=>{
    if(finalized)return;
    finalized=true;clearTimeout(timeoutTimer);clearTimeout(termTimer);clearTimeout(killTimer);process.removeListener("SIGINT",onInt);process.removeListener("SIGTERM",onTerm);
    try{fs.closeSync(stdoutFd);}catch(_){} try{fs.closeSync(stderrFd);}catch(_){}
    event(eventFd,{event:"finalize",direct_child_reaped:Boolean(closeEvent)});try{fs.closeSync(eventFd);}catch(_){}
    let sourceAfter,toolsAfter,cacheAfter,targetAfter;
    const postErrors=[];
    try{sourceAfter=sourceAudit(input);writeJson(path.join(out,"source-after.json"),sourceAfter);}catch(error){postErrors.push("source audit: "+String(error));}
    try{toolsAfter=toolAudit(input);writeJson(path.join(out,"tool-after.json"),toolsAfter);}catch(error){postErrors.push("tool audit: "+String(error));}
    try{cacheAfter=cacheAudit(input);writeJson(path.join(out,"cache-after.json"),cacheAfter);}catch(error){postErrors.push("cache audit: "+String(error));}
    try{targetAfter=targetSnapshot(input.plan.target_dir);writeJson(path.join(out,"target-after.json"),targetAfter);}catch(error){postErrors.push("target snapshot: "+String(error));}
    if(eventWriteErrors.length)postErrors.push("process event log write failure");
    if(outputWriteErrors.length)postErrors.push("captured output write failure");
    const packetPinsAfter=packetInputSnapshot();
    const packetChanges=[...new Set([
      ...packetInputChanges(pre.packet_input_pins,staticPinsFrom(packetPinsAtSpawn||{})),
      ...packetInputChanges(packetPinsBefore,packetPinsAtSpawn||{}),
      ...packetInputChanges(packetPinsAtSpawn||{},packetPinsAfter)
    ])];
    if(packetChanges.length)postErrors.push("packet inputs changed or became unreadable: "+packetChanges.join(", "));
    const stdout=fs.readFileSync(stdoutPath),stderr=fs.readFileSync(stderrPath);
    let eventBytes=Buffer.alloc(0);
    try{eventBytes=fs.readFileSync(eventsPath);}catch(error){postErrors.push("process event log read failure: "+String(error));}
    const streamAccountingMatches=states.stdout.accounting_known!==false&&states.stderr.accounting_known!==false&&states.stdout.stored===stdout.length&&states.stderr.stored===stderr.length&&total===stdout.length+stderr.length;
    if(!streamAccountingMatches)postErrors.push("captured stream byte accounting mismatch");
    const wrapperExitCode=timeout?124:parentSignal?130:(!closeEvent||closeEvent.code===null||reason||capExceeded||postErrors.length)?125:closeEvent.code;
    const processRecord={
      schema:"termrock-nextest-process-capture/v1",stage:"test_run_capture",semantic_parse:"NOT_PERFORMED",
      program:input.plan.program,argv:input.plan.argv,cwd:input.plan.cwd,pid:child&&child.pid||null,process_group:child&&child.pid||null,
      started_at:input.started_at,finished_at:new Date().toISOString(),elapsed_ms:Date.now()-input.started_ms,
      termination:closeEvent&&closeEvent.code!==null?{kind:"exited",exit_code:closeEvent.code}:closeEvent&&closeEvent.signal?{kind:"signaled",signal:closeEvent.signal}:timeout?{kind:"timeout"}:{kind:"incomplete"},
      timed_out:timeout,cap_exceeded:capExceeded,forced_close:forcedClose,direct_child_reaped:Boolean(closeEvent),spawn_error:spawnError,
      stop_reason:reason,parent_signal:parentSignal,wrapper_exit_code:wrapperExitCode,
      stdout:{path:stdoutPath,byte_length:stdout.length,sha256:sha(stdout),truncated:states.stdout.truncated},
      stderr:{path:stderrPath,byte_length:stderr.length,sha256:sha(stderr),truncated:states.stderr.truncated},
      stream_diagnostics:{stdout_bytes_received:states.stdout.received,stderr_bytes_received:states.stderr.received,stdout_bytes_stored:stdout.length,stderr_bytes_stored:stderr.length,combined_bytes_stored:stdout.length+stderr.length,combined_bytes_tracked:total,accounting_matches_artifacts:streamAccountingMatches,combined_cap_bytes:input.plan.limits.combined_cap_bytes,output_write_errors:outputWriteErrors.slice()},
      process_events:{path:eventsPath,byte_length:eventBytes.length,sha256:sha(eventBytes),write_errors:eventWriteErrors.slice()},
      packet_inputs:{preflight:pre.packet_input_pins,before:packetPinsBefore,static_at_spawn:staticPinsFrom(packetPinsAtSpawn||{}),at_spawn:packetPinsAtSpawn,after:packetPinsAfter,stable:packetChanges.length===0,changes:packetChanges},
      preflight_packet_input_pins:pre.packet_input_pins,
      source_stable:!!sourceAfter&&sourceBefore.digest===sourceAfter.digest,
      tools_stable:!!toolsAfter&&toolsBefore.digest===toolsAfter.digest,cache_stable:!!cacheAfter&&cacheBefore.digest===cacheAfter.digest,
      target_before_digest:targetBefore.digest,target_at_spawn_digest:targetAtSpawn&&targetAtSpawn.digest||null,target_after_digest:targetAfter&&targetAfter.digest||null,post_errors:postErrors,
      input_pins:packetPinsAtSpawn
    };
    writeJson(receiptPath,processRecord);
    process.stdout.write(JSON.stringify({result:"RAW_NEXTTEST_CAPTURE_COMPLETE",semantic_parse:"NOT_PERFORMED",record:receiptPath,child_termination:processRecord.termination,wrapper_exit_code:processRecord.wrapper_exit_code,stdout_sha256:processRecord.stdout.sha256,stderr_sha256:processRecord.stderr.sha256,process_events_sha256:processRecord.process_events.sha256},null,2)+"\n");
    process.exitCode=wrapperExitCode;
  };
  try{
    packetPinsAtSpawn=packetInputSnapshot();
    if(Object.values(packetPinsAtSpawn).some(value=>value===null))fail("one or more packet inputs are unreadable at spawn");
    const staticPinsAtSpawn=staticPinsFrom(packetPinsAtSpawn);
    if(stable(staticPinsAtSpawn)!==stable(pre.packet_input_pins))fail("static packet inputs changed immediately before spawn");
    const packetChangesBeforeSpawn=packetInputChanges(packetPinsBefore,packetPinsAtSpawn);
    if(packetChangesBeforeSpawn.length)fail("packet inputs changed between initial snapshot and spawn: "+packetChangesBeforeSpawn.join(", "));
    targetAtSpawn=targetSnapshot(input.plan.target_dir);
    if(targetAtSpawn.digest!==pre.target.digest || targetAtSpawn.entry_count!==pre.target.entry_count || targetAtSpawn.file_count!==pre.target.file_count || targetAtSpawn.bytes_total!==pre.target.bytes_total || targetAtSpawn.entries_sha256!==pre.target.entries_sha256)fail("full run target changed immediately before spawn");
    event(eventFd,{event:"spawn-request",program:input.plan.program,argv:input.plan.argv,cwd:input.plan.cwd});
    input.started_at=new Date().toISOString();input.started_ms=Date.now();
    child=spawn(input.plan.program,input.plan.argv,{cwd:input.plan.cwd,env:{...input.plan.environment},detached:true,stdio:["ignore","pipe","pipe"]});
    child.stdout.on("data",chunk=>capture("stdout",chunk));child.stderr.on("data",chunk=>capture("stderr",chunk));
    child.on("error",error=>{spawnError=String(error);event(eventFd,{event:"spawn-error",error:spawnError});});
    child.on("spawn",()=>event(eventFd,{event:"spawned",pid:child.pid,process_group:child.pid}));
    child.on("close",(code,signal)=>{if(finalized)return;closeEvent={code,signal};event(eventFd,{event:"child-close",code,signal});finalize();});
    timeoutTimer=setTimeout(()=>{timeout=true;stop("TIMEOUT");},input.plan.limits.timeout_ms);
  }catch(error){spawnError=String(error);closeEvent={code:null,signal:null};event(eventFd,{event:"spawn-exception",error:spawnError});finalize();}
}
try {
  const args=process.argv.slice(2);
  if(args.length!==1) fail("usage: capture-run.cjs --preflight-only | --capture-run");
  if(args[0]==="--preflight-only") preflight();
  else if(args[0]==="--capture-run") runNextest();
  else fail("unknown mode");
} catch(error) { process.stderr.write(String(error&&error.stack||error)+"\n"); process.exitCode=125; }
