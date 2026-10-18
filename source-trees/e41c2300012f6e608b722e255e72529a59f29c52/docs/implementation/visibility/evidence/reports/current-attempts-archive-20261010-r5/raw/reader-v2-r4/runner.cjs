'use strict';
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const {spawn, spawnSync} = require('child_process');
const {performance} = require('node:perf_hooks');

// Bounded R3 executor. Process-group and close handling follows the reviewed
// VIS-18 R12 executor (executor.cjs SHA c94f2161d341b20b209f565f0e4f7fe7acefaa1657113e318aa4640cbe491280).
const NODE = '/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node';
const PACKET_PATH = '/private/tmp/termrock-vis08-nextest-reader-v2-status-r4-execution-packet-20261010/packet.json';
const PACKET_SHA256 = 'a7c175440b0b534ff3c576ec66aac97178510b7a85c24a09197c92459595531d';
const EXPECTED_PACKAGE_DIGEST = '4bb9caa4df4014301044bb1f59ff1f7ffa61236a007a73da3c97e57104b7e240';
const EXPECTED_PACKAGE_FILES = [
  ["Cargo.lock",9097,420,"a80325ffed5b5cf5b652f9d9e8a859573c91b93def1e59d42b4259b65a586c1f"],
  ["Cargo.toml",399,420,"62457d004c79177c557970d452081007affb4a72ac1374f4c4fc3f0b45739be3"],
  ["README.md",5338,420,"67762a6f6df40b6e5514677e1dfbed4e429d7e1b91a99ddb53110b211df0126a"],
  ["src/bin/fixture.rs",21577,420,"a7ee4a89646439a67d2d4e016b759a2d68eecec7339310778e81acf60235adc4"],
  ["src/bin/nextest-result-reader.rs",1528,420,"518d527af27dd880057db6c4ef029244d59fe8cdcfdf6d3822b8aad2e7b83500"],
  ["src/lib.rs",51111,420,"5ec6fe8e4a4bbb15e888c403fc91e35a6431c6541481c162d4da3aca996f2f2c"],
  ["src/nextest_result.rs",115612,420,"8f4a8d77aa1f32250bd0df79138daf0e77ee6c47afb9b71a9ebe7347a4729c8d"],
  ["tests/cache_source_audit.rs",75580,420,"9f82e8ae736575ff68bdf4adf96ef695c658ba99136f683b95f03cd147f4ed8f"],
  ["tests/deferred.rs",68958,420,"d5d336c7527dcdb56704df6be68e937bc8be10ee0545b592a79e63402561ec94"],
  ["tests/queue_v2.rs",194318,420,"07d9ccc5c96bb3cfef46218c78090acc0dd98a2af450e254e1dd23c60de1f361"],
  ["tests/status.rs",147342,420,"e09256ea95055a3b9c33ea12b05f572661bc84bf3ff41c105e016bd279932d3b"],
  ["tests/subjects.rs",207089,420,"bd676ba4c8f5bbbac996797d6d7cf0fc1534a7b6bda9bf2dbc7e9d4aa73f72e4"],
];
const EXPECTED_TESTS = [
  "validates_exact_nine_selection_and_list_identities",
  "reads_machine_records_from_the_configured_stderr_stream",
  "verifies_nine_observed_tests_across_two_binaries_and_plural_summary",
  "rejects_structured_records_split_across_stdout_and_stderr",
  "list_ignored_and_filter_status_must_match_static_selection",
  "selected_filter_and_ignored_status_must_match_run_policy",
  "rejects_missing_extra_duplicate_and_suffix_terminal_identities",
  "terminal_identity_before_start_is_invalid",
  "identical_repeated_suite_snapshots_are_observations_not_extra_tests",
  "rejects_suite_start_after_terminal_snapshot",
  "filtered_out_u64_values_are_preserved_and_inventory_anomalies_marked",
  "list_ignored_does_not_become_run_filtered_out",
  "human_nextest_contradiction_blocks_without_rewriting_machine_results",
  "child_summary_is_checked_only_when_bound_to_one_suite",
  "human_only_capture_is_unavailable_not_a_pass",
  "preserves_child_pass_and_wrapper_125_as_distinct_process_facts",
  "rejects_other_format_and_retry_stress_or_partition_invocations",
  "request_file_capture_metadata_and_trusted_selection_hash_are_checked",
  "v2_accepts_empty_filter_and_exact_59_case_status_selection",
  "v2_rejects_empty_selection",
  "v2_rejects_oversized_selection",
  "v2_accepts_142_selected_identities_across_four_targets",
  "v2_rejects_count_and_target_mismatches"
];
const EXPECTED_RUNTIME_ROOT = '/private/tmp/termrock-vis08-nextest-reader-v2-status-r4-execution-20261010';
const LIMITS = { stage_timeout_ms:900000, cli_timeout_ms:30000, term_grace_ms:5000,
  post_kill_close_timeout_ms:5000, stage_output_bytes:4194304, cli_output_bytes:1048576,
  request_bytes:1048576, raw_capture_bytes:8388608 };
let packet;
let eventFd = null;
let active = null;
let externalSignal = null;
let finished = false;

function fail(message) { throw new Error(message); }
function sha(bytes) { return crypto.createHash('sha256').update(bytes).digest('hex'); }
function hashU64(n) { const b=Buffer.alloc(8); b.writeBigUInt64BE(BigInt(n)); return b; }
function jsonHash(value) { return sha(Buffer.from(JSON.stringify(value),'utf8')); }
function writeAll(fd, bytes) {
  const b=Buffer.isBuffer(bytes)?bytes:Buffer.from(String(bytes),'utf8');
  let offset=0;
  while(offset<b.length) { const n=fs.writeSync(fd,b,offset,b.length-offset); if(n<=0) fail('write made no progress'); offset+=n; }
}
function readRegular(file, max) {
  const fd=fs.openSync(file,fs.constants.O_RDONLY|(fs.constants.O_NOFOLLOW||0));
  try {
    const s=fs.fstatSync(fd);
    if(!s.isFile()||s.size>max) fail('not a bounded regular file: '+file);
    const chunks=[]; let offset=0;
    while(offset<s.size) { const b=Buffer.alloc(Math.min(1024*1024,s.size-offset)); const n=fs.readSync(fd,b,0,b.length,offset); if(n<=0)fail('short read: '+file); chunks.push(b.subarray(0,n)); offset+=n; }
    return Buffer.concat(chunks);
  } finally { fs.closeSync(fd); }
}
function fileHash(file) { return sha(readRegular(file,256*1024*1024)); }
function readJson(file,max=16*1024*1024) { try{return JSON.parse(readRegular(file,max).toString('utf8'));}catch(e){fail('invalid JSON '+file+': '+String(e));} }
function lstatDir(file, mode) {
  const s=fs.lstatSync(file);
  if(!s.isDirectory()||s.isSymbolicLink()||fs.realpathSync(file)!==file) fail('directory is not real/canonical: '+file);
  if(mode!==undefined&&(s.mode&0o777)!==mode) fail('directory mode mismatch: '+file);
  return s;
}
function lstatFile(file) {
  const s=fs.lstatSync(file);
  if(!s.isFile()||s.isSymbolicLink()) fail('expected regular file: '+file);
  return s;
}
function exists(file) { try{fs.lstatSync(file);return true;}catch(e){if(e&&e.code==='ENOENT')return false;throw e;} }
function safeRel(rel) { return typeof rel==='string'&&rel.length>0&&!path.posix.isAbsolute(rel)&&!rel.split('/').some(x=>x==='..'||x==='.'||x===''); }
function walk(root) {
  lstatDir(root);
  const files=[],dirs=[];
  function visit(dir,rel) {
    const entries=fs.readdirSync(dir,{withFileTypes:true}).sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0);
    for(const e of entries) {
      const abs=path.join(dir,e.name),r=rel?rel+'/'+e.name:e.name,s=fs.lstatSync(abs);
      if(s.isSymbolicLink()) fail('symlink in protected tree: '+abs);
      if(s.isDirectory()) { dirs.push(r); visit(abs,r); }
      else if(s.isFile()) files.push(r);
      else fail('special filesystem entry in protected tree: '+abs);
    }
  }
  visit(root,''); files.sort(); dirs.sort(); return {files,dirs};
}
function sourcePackageSnapshot(p) {
  const root=p.candidate.package; const walked=walk(root);
  const expected=EXPECTED_PACKAGE_FILES.map(x=>x[0]).sort();
  if(JSON.stringify(walked.files)!==JSON.stringify(expected)) fail('R3 package file inventory changed');
  const h=crypto.createHash('sha256'),rows=[];
  for(const [rel,bytes,mode,digest] of EXPECTED_PACKAGE_FILES) {
    const abs=path.join(root,rel),data=readRegular(abs,1024*1024),s=fs.lstatSync(abs);
    if(data.length!==bytes||(s.mode&0o777)!==mode||sha(data)!==digest) fail('R3 source file differs: '+rel);
    const name=Buffer.from(rel,'utf8'); h.update(hashU64(name.length));h.update(name);h.update(hashU64(data.length));h.update(data);
    rows.push({path:rel,bytes,mode,sha256:digest});
  }
  const digest=h.digest('hex');
  if(digest!==EXPECTED_PACKAGE_DIGEST) fail('R3 full package digest mismatch: '+digest);
  return {root,file_count:rows.length,digest,files:rows};
}
function validateFreezeAndReview(p) {
  const parentFreezeBytes=readRegular(p.candidate.freeze.path,1024*1024);
  const parentReviewBytes=readRegular(p.candidate.review.path,1024*1024);
  const parentPatch=readRegular(p.candidate.patch.path,1024*1024);
  if(sha(parentFreezeBytes)!==p.candidate.freeze.sha256||sha(parentReviewBytes)!==p.candidate.review.sha256
    ||sha(parentPatch)!==p.candidate.patch.sha256) fail('R3 parent source review/freeze bytes changed');
  const freeze=JSON.parse(parentFreezeBytes.toString('utf8'));
  const parentReview=JSON.parse(parentReviewBytes.toString('utf8'));
  if(freeze.candidate_package!==p.candidate.parent_package||parentReview.verdict!=='APPROVE'
    ||freeze.allowed_paths.length!==3) fail('R3 parent source lineage mismatch');
  const parentRows=new Map(freeze.allowed_paths.map(x=>[x.path,x.sha256]));
  const unchanged=['crates/termrock-visibility-tests/src/lib.rs','crates/termrock-visibility-tests/src/bin/nextest-result-reader.rs'];
  for(const rel of unchanged) {
    const digest=EXPECTED_PACKAGE_FILES.find(x=>x[0]===rel.replace('crates/termrock-visibility-tests/',''))?.[3];
    if(!digest||parentRows.get(rel)!==digest) fail('R3 parent and v2 package differ outside reader module: '+rel);
  }
  const delta=p.candidate.source_delta_review;
  if(fileHash(delta.path)!==delta.sha256) fail('v2 source review bytes changed');
  const review=readJson(delta.path,1024*1024);
  const modulePath=path.join(p.candidate.package,'src/nextest_result.rs');
  const moduleHash=fileHash(modulePath);
  if(review.verdict!=='READY_STATIC_SOURCE_ONLY'||review.candidate?.path!==modulePath
    ||review.candidate?.sha256!==moduleHash||moduleHash!==delta.candidate_source_sha256
    ||review.base?.path!==path.join(p.candidate.parent_package,'src/nextest_result.rs')
    ||review.base?.sha256!==delta.base_source_sha256||review.base?.sha256!==parentRows.get('crates/termrock-visibility-tests/src/nextest_result.rs'))
    fail('v2 reader source delta review does not bind the candidate and R3 base');
  return {parent_freeze_sha256:p.candidate.freeze.sha256,parent_review_sha256:p.candidate.review.sha256,
    parent_patch_sha256:p.candidate.patch.sha256,v2_review_sha256:delta.sha256,v2_source_sha256:moduleHash,
    v2_base_source_sha256:review.base.sha256};
}
function cacheReceiptSnapshot(p) {
  const c=p.cache, receipt=readJson(c.source_receipt.path,16*1024*1024);
  if(fileHash(c.source_receipt.path)!==c.source_receipt.sha256 || receipt.state!=='PREPARED_NOT_USED'
    ||receipt.runtime_cargo_home!==c.source_cargo_home||receipt.files.length!==c.source_receipt.file_count
    ||receipt.archive_count!==c.source_receipt.registry_archives||receipt.sparse_index_record_count!==c.source_receipt.sparse_index_records
    ||receipt.config_count!==c.source_receipt.config_files||receipt.extracted_sources_copied!==0||receipt.git_sources_copied!==0
    ||receipt.source_cargo_home!=='/private/tmp/termrock-vis18-cache-source-audit-r4-20261010/cargo-home') fail('R3 cache receipt binding mismatch');
  const rootStat=lstatDir(c.source_cargo_home,0o700);
  const walked=walk(c.source_cargo_home);
  const expected=receipt.files.map(f=>f.relative).sort();
  if(JSON.stringify(walked.files)!==JSON.stringify(expected)) fail('source Cargo cache inventory mismatch');
  const rows=[];
  let archiveCount=0,indexCount=0,configCount=0;
  for(const f of receipt.files) {
    if(!safeRel(f.relative)) fail('unsafe cache path in receipt');
    const abs=path.join(c.source_cargo_home,f.relative),b=readRegular(abs,64*1024*1024),s=fs.lstatSync(abs);
    if(b.length!==f.bytes||(s.mode&0o777)!==f.mode||sha(b)!==f.sha256||s.nlink!==1
      ||String(s.dev)!==f.destination_device||String(s.ino)!==f.destination_inode) fail('source cache differs from receipt: '+f.relative);
    if(f.relative.endsWith('.crate')) archiveCount++;
    if(f.relative.includes('/.cache/')) indexCount++;
    if(f.relative.endsWith('/config.json')) configCount++;
    rows.push({relative:f.relative,bytes:f.bytes,mode:f.mode,sha256:f.sha256,dev:String(s.dev),ino:String(s.ino),nlink:s.nlink});
  }
  if(archiveCount!==c.source_receipt.registry_archives||indexCount!==c.source_receipt.sparse_index_records||configCount!==c.source_receipt.config_files) fail('cache closure counts mismatch');
  const sum=crypto.createHash('sha256');
  for(const f of rows) { const r=Buffer.from(f.relative);sum.update(hashU64(r.length));sum.update(r);sum.update(hashU64(f.bytes));sum.update(Buffer.from(f.sha256,'hex')); }
  return {root:c.source_cargo_home,root_dev:String(rootStat.dev),root_ino:String(rootStat.ino),files:rows,file_count:rows.length,
    archives:archiveCount,sparse_indexes:indexCount,configs:configCount,inventory_sha256:sum.digest('hex'),receipt_sha256:c.source_receipt.sha256};
}
function fixtureSnapshot(p) {
  const rows=[];
  for(const exercise of [p.commands.cli_exercise,p.commands.cli_exercise_v2]) {
    for(const f of exercise.fixture_files) {
      const b=readRegular(f.path,Math.max(f.bytes,1)),s=fs.lstatSync(f.path);
      if(b.length!==f.bytes||sha(b)!==f.sha256||s.nlink!==1) fail('CLI fixture input changed: '+f.path);
      rows.push({path:f.path,bytes:f.bytes,sha256:f.sha256,mode:s.mode&0o777,dev:String(s.dev),ino:String(s.ino)});
    }
  }
  return rows;
}
function toolsSnapshot(p) {
  const out={node:{path:NODE,sha256:fileHash(NODE),version:process.version}};
  for(const [name,t] of Object.entries(p.runtime.tools)) {
    const digest=fileHash(t.path); if(digest!==t.sha256) fail('pinned tool hash mismatch: '+name);
    out[name]={path:t.path,sha256:digest,version:t.version};
  }
  if(out.node.sha256!=='9d050fd455b56426e25d4d603c7c501cbb2630348e836cf221dcce748e90588a'||out.node.version!=='v24.20.0') fail('Node identity mismatch');
  return out;
}
function snapshot(p) {
  return {packet_sha256:PACKET_SHA256,package:sourcePackageSnapshot(p),freeze:validateFreezeAndReview(p),cache:cacheReceiptSnapshot(p),
    tools:toolsSnapshot(p),cli_fixtures:fixtureSnapshot(p)};
}
function preparePacket() {
  const bytes=readRegular(PACKET_PATH,2*1024*1024);
  if(sha(bytes)!==PACKET_SHA256) fail('execution packet SHA-256 mismatch');
  const p=JSON.parse(bytes.toString('utf8'));
  if(p.schema!=='termrock-vis08-nextest-reader-root-execution-packet/v2'||p.state!=='PREPARED_FOR_ROOT_LAUNCH_NOT_AUTHORIZED_BY_AUTHOR'
    ||p.runtime.attempt_root!==EXPECTED_RUNTIME_ROOT||p.selection.expected_test_count!==23
    ||JSON.stringify(p.selection.test_names)!==JSON.stringify(EXPECTED_TESTS)
    ||p.runtime.supervisor.overall_timeout_ms!==900000||p.runtime.supervisor.term_grace_ms!==5000
    ||p.runtime.supervisor.post_kill_close_timeout_ms!==5000||p.runtime.supervisor.combined_stdout_stderr_limit_bytes!==4194304
    ||p.commands.nextest_run.expected_started!==23||p.commands.nextest_run.expected_terminal_ok!==23) fail('execution packet contract mismatch');
  if(p.candidate.manifest_sha256!==EXPECTED_PACKAGE_FILES.find(x=>x[0]==='Cargo.toml')[3]
    ||p.candidate.lock_sha256!==EXPECTED_PACKAGE_FILES.find(x=>x[0]==='Cargo.lock')[3]
    ||p.candidate.package_length_prefixed_sha256!==EXPECTED_PACKAGE_DIGEST) fail('manifest/lock/package pin mismatch');
  const root=p.runtime.attempt_root;
  if(root!==EXPECTED_RUNTIME_ROOT||!path.isAbsolute(root)||path.dirname(root)!=='/private/tmp') fail('runtime root mismatch');
  if(exists(root)) fail('runtime output root already exists');
  const expectedFilter=EXPECTED_TESTS.map(name=>'test(=nextest_result::tests::'+name+')').join(' | ');
  if(!p.commands.nextest_run.argv.includes('--filterset')||p.commands.nextest_run.argv[p.commands.nextest_run.argv.indexOf('--filterset')+1]!==expectedFilter) fail('Nextest filter differs from exact 23-test set');
  if(p.commands.nextest_run.argv.includes('--test-timeout')||p.commands.nextest_run.argv.includes('--retries')&&p.commands.nextest_run.argv[p.commands.nextest_run.argv.indexOf('--retries')+1]!=='0') fail('unsupported or nonzero Nextest timeout/retry option');
  if(p.runtime.environment.CARGO_HOME!==path.join(root,'cargo-home')||p.runtime.environment.CARGO_TARGET_DIR!==path.join(root,'target')
    ||p.runtime.environment.HOME!==path.join(root,'home')||p.runtime.environment.TMPDIR!==path.join(root,'tmp')
    ||p.runtime.environment.CARGO_NET_OFFLINE!=='true'||p.runtime.environment.CARGO_BUILD_JOBS!=='2') fail('isolated offline environment mismatch');
  if(p.commands.cli_exercise.expected_report_fields.selection_count!=='9'
    ||p.commands.cli_exercise_v2.expected_report_fields.selection_count!=='59'
    ||p.commands.cli_exercise_v2.expected_report_fields.expected_test_count!==59) fail('CLI exercise counts differ from the packet contract');
  return p;
}
function makeEnv(p) { return Object.assign({},p.runtime.environment); }
function mkdirPrivate(dir) { fs.mkdirSync(dir,{mode:0o700});fs.chmodSync(dir,0o700);lstatDir(dir,0o700); }
function makeRunTree(p) {
  fs.mkdirSync(p.runtime.attempt_root,{mode:0o700});fs.chmodSync(p.runtime.attempt_root,0o700);lstatDir(p.runtime.attempt_root,0o700);
  for(const d of [p.runtime.cargo_home,p.runtime.home,p.runtime.target_dir,p.runtime.tmp_dir,p.runtime.logs_dir,p.runtime.cli_output_dir]) mkdirPrivate(d);
  for(const d of [p.runtime.home,p.runtime.target_dir,p.runtime.tmp_dir,p.runtime.logs_dir,p.runtime.cli_output_dir]) if(fs.readdirSync(d).length!==0) fail('fresh runtime directory is nonempty: '+d);
}
function copyCache(p, sourceSnapshot, deadline) {
  const checkDeadline=()=>{if(performance.now()>=deadline)fail('overall execution deadline exhausted during cache copy');};
  checkDeadline();
  const dest=p.runtime.cargo_home, source=p.cache.source_cargo_home;
  const dirs=new Set(['']);
  for(const f of sourceSnapshot.files) { let parent=path.posix.dirname(f.relative);while(parent!=='.'){dirs.add(parent);parent=path.posix.dirname(parent);} }
  for(const rel of Array.from(dirs).filter(Boolean).sort((a,b)=>a.split('/').length-b.split('/').length||a.localeCompare(b))) {
    checkDeadline();
    const src=path.join(source,rel),s=fs.lstatSync(src); if(!s.isDirectory()||s.isSymbolicLink())fail('cache directory changed during copy');
    const d=path.join(dest,rel);fs.mkdirSync(d,{mode:s.mode&0o777});fs.chmodSync(d,s.mode&0o777);
  }
  const copyRows=[];
  for(const f of sourceSnapshot.files) {
    checkDeadline();
    const src=path.join(source,f.relative),dst=path.join(dest,f.relative),b=readRegular(src,64*1024*1024);
    fs.writeFileSync(dst,b,{flag:'wx',mode:f.mode});fs.chmodSync(dst,f.mode);
    const s=fs.lstatSync(src),d=fs.lstatSync(dst);
    if(d.isSymbolicLink()||!d.isFile()||d.nlink!==1||d.size!==f.bytes||sha(readRegular(dst,64*1024*1024))!==f.sha256
      ||(d.mode&0o777)!==f.mode||(String(s.dev)===String(d.dev)&&String(s.ino)===String(d.ino))) fail('private cache byte-copy verification failed: '+f.relative);
    copyRows.push({relative:f.relative,bytes:f.bytes,mode:f.mode,sha256:f.sha256,source:{dev:String(s.dev),ino:String(s.ino),nlink:s.nlink},destination:{dev:String(d.dev),ino:String(d.ino),nlink:d.nlink}});
  }
  const actual=walk(dest);if(actual.files.length!==sourceSnapshot.files.length)fail('copied cache file count mismatch');
  const receipt={schema:'termrock-vis08-r5-runtime-cache-copy/v1',state:'COPIED_AND_VERIFIED_FOR_ONE_OFFLINE_RUN',source_receipt_sha256:sourceSnapshot.receipt_sha256,
    source_cargo_home:source,runtime_cargo_home:dest,file_count:copyRows.length,archive_count:sourceSnapshot.archives,
    sparse_index_record_count:sourceSnapshot.sparse_indexes,config_count:sourceSnapshot.configs,
    extracted_sources_copied:0,git_sources_copied:0,copy_method:'independent byte copies; no symlinks or hardlinks',files:copyRows};
  const out=path.join(p.runtime.attempt_root,'cache-copy-receipt.json');fs.writeFileSync(out,JSON.stringify(receipt,null,2)+'\n',{flag:'wx',mode:0o600});fs.chmodSync(out,0o600);
  checkDeadline();
  return {path:out,sha256:fileHash(out),file_count:copyRows.length,archive_count:sourceSnapshot.archives,sparse_index_record_count:sourceSnapshot.sparse_indexes};
}
function event(type,details={}) { if(eventFd!==null)writeAll(eventFd,JSON.stringify(Object.assign({at_utc:new Date().toISOString(),type},details))+'\n'); }
function runStage(stage,program,args,cwd,env,timeout,cap,stdoutPath,stderrPath) {
  return new Promise(resolve=>{
    const outFd=fs.openSync(stdoutPath,'wx',0o600),errFd=fs.openSync(stderrPath,'wx',0o600);
    const start=Date.now();let captured=0,reason=null,close=null,exit=null,error=null,closed=false,forced=false,done=false,termTimer=null,killTimer=null,outerTimer=null;
    const child=spawn(program,args,{cwd,env,detached:true,stdio:['ignore','pipe','pipe']});
    if(!child.pid){close={code:null,signal:null,spawn_error:'missing child pid'};error='missing child pid';finish();return;}
    const pid=child.pid; active={stage,child,pid,reason:null,signal:null,terminate:null};
    event('spawned',{stage,pid,process_group:pid,program,args,cwd,timeout_ms:timeout,output_cap_bytes:cap});
    const signalGroup=sig=>{try{process.kill(-pid,sig);event('group_signal',{stage,pid,process_group:pid,signal:sig});}catch(e){event('group_signal_error',{stage,pid,process_group:pid,signal:sig,error:String(e)});}};
    function terminate(why) {
      if(reason||done)return;reason=why;if(active)active.reason=why;
      event('termination_requested',{stage,reason:why,elapsed_ms:Date.now()-start});signalGroup('SIGTERM');
      termTimer=setTimeout(()=>{if(closed)return;event('term_grace_expired',{stage,reason:why});signalGroup('SIGKILL');
        killTimer=setTimeout(()=>{if(closed)return;forced=true;event('post_kill_close_timeout',{stage,reason:why});child.stdout.destroy();child.stderr.destroy();close={code:null,signal:null,forced_close_timeout:true};finish();},LIMITS.post_kill_close_timeout_ms);
      },LIMITS.term_grace_ms);
    }
    active.terminate=terminate;
    function capture(fd,chunk,stream) {
      if(done)return;const left=Math.max(0,cap-captured),take=Math.min(left,chunk.length);
      if(take>0){writeAll(fd,chunk.subarray(0,take));captured+=take;}
      if(take<chunk.length&&!reason){event('output_limit_exceeded',{stage,stream,captured_bytes:captured,dropped_bytes:chunk.length-take});terminate('output-cap');}
    }
    child.stdout.on('data',b=>capture(outFd,b,'stdout'));child.stderr.on('data',b=>capture(errFd,b,'stderr'));
    child.on('error',e=>{error=String(e);event('child_error',{stage,error});});
    child.on('exit',(code,signal)=>{exit={code,signal};event('child_exit',{stage,code,signal});});
    child.on('close',(code,signal)=>{closed=true;close={code,signal};event('child_close',{stage,code,signal});if(termTimer)clearTimeout(termTimer);if(killTimer)clearTimeout(killTimer);finish();});
    outerTimer=setTimeout(()=>terminate('timeout'),timeout);
    function finish() {
      if(done||!close)return;done=true;if(outerTimer)clearTimeout(outerTimer);if(termTimer)clearTimeout(termTimer);if(killTimer)clearTimeout(killTimer);
      child.stdout.removeAllListeners('data');child.stderr.removeAllListeners('data');
      try{fs.closeSync(outFd);}catch(_){}try{fs.closeSync(errFd);}catch(_){}
      const reaped=Boolean(closed&&!forced&&exit&&exit.code===close.code&&exit.signal===close.signal);
      const result={stage,program,args,cwd,pid,process_group:pid,detached:true,exit,close,error,child_reaped:reaped,forced_close_timeout:forced,
        timeout_ms:timeout,output_limit_bytes:cap,output_bytes_captured:captured,timed_out:reason==='timeout',output_limit_exceeded:reason==='output-cap',termination_reason:reason,
        elapsed_ms:Date.now()-start,stdout:{path:stdoutPath,bytes:fs.statSync(stdoutPath).size,sha256:fileHash(stdoutPath)},stderr:{path:stderrPath,bytes:fs.statSync(stderrPath).size,sha256:fileHash(stderrPath)}};
      const okay=Boolean(reaped&&close.code===0&&!close.signal&&!error&&!forced&&!reason);
      result.stage_exit_ok=okay;if(active&&active.stage===stage)active=null;resolve(result);
      if(forced){child.unref();setImmediate(()=>process.exit(125));}
    }
  });
}
function installSignals() {
  const onSignal=sig=>{
    if(externalSignal){event('repeat_external_signal',{first:externalSignal,again:sig});if(active&&active.terminate)active.terminate('external-'+sig);if(active){try{process.kill(-active.pid,'SIGKILL');}catch(_){}}return;}
    externalSignal=sig;event('external_signal',{signal:sig});if(active&&active.terminate)active.terminate('external-'+sig);
  };
  process.on('SIGTERM',()=>onSignal('SIGTERM'));process.on('SIGINT',()=>onSignal('SIGINT'));
}
function parseNextest(stdoutPath,stderrPath) {
  const max=LIMITS.stage_output_bytes, stdout=readRegular(stdoutPath,max).toString('utf8'),stderr=readRegular(stderrPath,max).toString('utf8');
  const records=[],malformed=[];
  for(const [stream,text] of [['stdout',stdout],['stderr',stderr]]) for(const [index,line] of text.split(/\r?\n/).entries()) {
    const t=line.trim();if(!t.startsWith('{'))continue;
    let v;try{v=JSON.parse(t);}catch(e){if(/"type"\s*:\s*"(?:suite|test)"/.test(t))malformed.push({stream,line:index+1});continue;}
    if(v&&['suite','test'].includes(v.type))records.push({stream,value:v});
  }
  const suiteRecords=records.filter(x=>x.value.type==='suite');
  const testRecords=records.filter(x=>x.value.type==='test');
  const unexpected=[];
  if(records.some(x=>x.stream!=='stdout')) unexpected.push({reason:'structured suite/test records split across stdout and stderr'});
  if(suiteRecords.some(x=>!x.value.nextest||typeof x.value.nextest!=='object'))
    unexpected.push({reason:'suite event missing Nextest metadata'});
  const expectedNames=EXPECTED_TESTS.map(x=>'nextest_result::tests::'+x);
  const packageName='termrock-visibility-tests',binaryName='termrock_visibility_tests';
  const started=new Map(),terminal=new Map();
  const metadata=suiteRecords.filter(r=>r.value.nextest&&typeof r.value.nextest==='object').map(r=>r.value.nextest);
  const expectedFull=new Set(expectedNames.map(n=>packageName+'::'+binaryName+'$'+n));
  for(const r of testRecords) {
    const v=r.value;
    if(r.stream!=='stdout'){unexpected.push({reason:'structured test event on stderr',name:v.name||null});continue;}
    if(typeof v.name!=='string'||!expectedFull.has(v.name)){unexpected.push({reason:'unexpected test identity',name:v.name||null});continue;}
    if(v.event==='started') {
      if(terminal.has(v.name)) unexpected.push({reason:'test started after terminal event',name:v.name});
      started.set(v.name,(started.get(v.name)||0)+1);
    } else if(v.event==='ok') {
      if(started.get(v.name)!==1) unexpected.push({reason:'terminal event before exactly one start',name:v.name});
      terminal.set(v.name,(terminal.get(v.name)||0)+1);
    } else {
      unexpected.push({reason:'unexpected test event',name:v.name,event:v.event||null});
    }
  }
  const metaOkay=suiteRecords.length>0&&metadata.length===suiteRecords.length
    &&metadata.every(m=>m.crate===packageName&&m.test_binary===binaryName&&m.kind==='lib');
  const expectedFullList=Array.from(expectedFull).sort();
  const allStarted=expectedFullList.every(n=>started.get(n)===1)&&started.size===EXPECTED_TESTS.length;
  const allOk=expectedFullList.every(n=>terminal.get(n)===1)&&terminal.size===EXPECTED_TESTS.length
    &&testRecords.filter(r=>r.value.event!=='started'&&r.value.event!=='ok').length===0;
  const result={machine_stream:'stdout',record_count:records.length,suite_records:records.filter(x=>x.value.type==='suite').length,
    test_records:testRecords.length,expected_test_count:EXPECTED_TESTS.length,expected_full_names:expectedFullList,
    started:Object.fromEntries(Array.from(started.entries()).sort()),terminal_ok:Object.fromEntries(Array.from(terminal.entries()).sort()),
    metadata,malformed_structured_lines:malformed,unexpected,metadata_valid:metaOkay,exact_started:allStarted,exact_terminal_ok:allOk};
  result.valid=records.length>0&&malformed.length===0&&unexpected.length===0&&metaOkay&&allStarted&&allOk;
  return result;
}
function checkCliReport(reportPath,expected) {
  const bytes=readRegular(reportPath,LIMITS.cli_output_bytes),report=JSON.parse(bytes.toString('utf8'));
  const selected=report.run?.selected_tests;
  if(report.schema!==expected.schema||report.acceptance!=='accepted'||report.selection_count!==expected.selection_count
    ||report.run?.machine_state!=='verified'||!Array.isArray(selected)||selected.length!==expected.selected_tests_length)
    fail('CLI smoke report mismatch: '+reportPath);
  if(expected.expected_test_count!==undefined&&report.expected_test_count!==expected.expected_test_count) fail('v2 expected count mismatch');
  if(expected.target_count!==undefined&&(!Array.isArray(report.targets)||report.targets.length!==expected.target_count)) fail('v2 target count mismatch');
  return {path:reportPath,bytes:bytes.length,sha256:sha(bytes),schema:report.schema,acceptance:report.acceptance,
    selection_count:report.selection_count,machine_state:report.run.machine_state,selected_test_count:selected.length,
    expected_test_count:report.expected_test_count??null,target_count:report.targets?.length??null};
}
function versionProbe(p, deadline) {
  const remaining=()=>Math.floor(deadline-performance.now());
  const probes=[['cargo',p.runtime.tools.cargo.path,['--version'],p.runtime.tools.cargo.version],
    ['rustc',p.runtime.tools.rustc.path,['--version'],p.runtime.tools.rustc.version],
    ['rustdoc',p.runtime.tools.rustdoc.path,['--version'],p.runtime.tools.rustdoc.version],
    ['nextest',p.runtime.tools.nextest.path,['--version'],p.runtime.tools.nextest.version]];
  const results=[];
  const env={PATH:p.runtime.environment.PATH,HOME:p.runtime.home,LANG:'C',LC_ALL:'C.UTF-8',RUSTUP_HOME:p.runtime.environment.RUSTUP_HOME};
  for(const [name,program,args,expected] of probes) {
    const budget=remaining();
    if(budget<=0) fail('overall execution deadline exhausted before version probe: '+name);
    const r=spawnSync(program,args,{cwd:p.candidate.package,env,encoding:'utf8',timeout:Math.min(10000,budget),maxBuffer:65536});
    if(performance.now()>=deadline) fail('overall execution deadline exceeded during version probe: '+name);
    const stdout=String(r.stdout||''),stderr=String(r.stderr||''),versionLine=(stdout.split(/\r?\n/,1)[0]||'').trim();
    if(r.error||r.status!==0||r.signal||versionLine!==expected||stderr.length!==0) fail('tool version probe failed for '+name+': '+String(r.error||stderr||r.status)+'; first_stdout_line='+versionLine);
    results.push({name,program,args,version:expected,version_first_line:versionLine,stdout,stderr,
      stdout_sha256:sha(Buffer.from(stdout,'utf8')),stderr_sha256:sha(Buffer.from(stderr,'utf8')),exit_code:r.status});
  }
  return results;
}
function writeNew(file,value) { fs.writeFileSync(file,JSON.stringify(value,null,2)+'\n',{flag:'wx',mode:0o600});fs.chmodSync(file,0o600); }
function validateAll(deadline=null) {
  if(process.execPath!==NODE||process.version!=='v24.20.0') fail('invoke with the pinned Node 24.20.0 executable');
  packet=preparePacket();
  if(deadline!==null&&performance.now()>=deadline) fail('overall execution deadline exhausted before input snapshot');
  const before=snapshot(packet);
  if(deadline!==null&&performance.now()>=deadline) fail('overall execution deadline exhausted during input snapshot');
  return {packet,before};
}
function preflight() {
  const {packet:p,before}=validateAll();
  process.stdout.write(JSON.stringify({state:'PREFLIGHT_PASS_NO_CARGO_NO_TEST',authorized:false,runner_path:__filename,runner_sha256:fileHash(__filename),
    packet_path:PACKET_PATH,packet_sha256:PACKET_SHA256,attempt_root:p.runtime.attempt_root,source_package_digest:before.package.digest,
    source_review_bindings:before.freeze,
    source_cache_files:before.cache.file_count,source_cache_inventory_sha256:before.cache.inventory_sha256,tools:before.tools,
    nextest_filterset:p.commands.nextest_run.argv[p.commands.nextest_run.argv.indexOf('--filterset')+1],selected_tests:EXPECTED_TESTS.length,
    exact_launch_argv:[NODE,__filename,'--run']},null,2)+'\n');
}
async function execute() {
  const started=performance.now();
  packet=preparePacket();
  const overallTimeout=packet.runtime.supervisor.overall_timeout_ms;
  const deadline=started+overallTimeout;
  const {packet:p,before}=validateAll(deadline);
  makeRunTree(p);
  if(performance.now()>=deadline) fail('overall execution deadline exhausted while creating private runtime tree');
  eventFd=fs.openSync(path.join(p.runtime.logs_dir,'process-events.jsonl'),'wx',0o600);
  installSignals();
  const cacheCopy=copyCache(p,before.cache,deadline);
  const versions=versionProbe(p,deadline);
  writeNew(path.join(p.runtime.attempt_root,'preflight.json'),{state:'PREFLIGHT_PASS_BEFORE_CHILDREN',authorized:false,packet_sha256:PACKET_SHA256,
    runner_sha256:fileHash(__filename),source_snapshot_sha256:jsonHash(before),cache_copy:cacheCopy,tool_versions:versions,
    exact_test_count:EXPECTED_TESTS.length,output_root:p.runtime.attempt_root,
    overall_deadline:{timeout_ms:overallTimeout,elapsed_ms:performance.now()-started,monotonic_clock:'performance.now'}});
  const env=makeEnv(p),stages=[];
  const remaining=(label)=>{const ms=Math.floor(deadline-performance.now());if(ms<=0)fail('overall execution deadline exhausted before '+label);return ms;};
  const cmd=p.commands.nextest_run;
  const nextestOut=path.join(p.runtime.logs_dir,'nextest.stdout.raw'),nextestErr=path.join(p.runtime.logs_dir,'nextest.stderr.raw');
  const nextest=await runStage('nextest-run',cmd.program,cmd.argv,cmd.cwd,env,Math.min(LIMITS.stage_timeout_ms,remaining('Nextest')),LIMITS.stage_output_bytes,nextestOut,nextestErr);stages.push(nextest);
  let identities=null,build=null;const reports=[];
  if(nextest.stage_exit_ok&&!externalSignal&&performance.now()<deadline) {
    identities=parseNextest(nextestOut,nextestErr);
    if(identities.valid&&!externalSignal&&performance.now()<deadline) {
      const b=p.commands.cli_build;
      build=await runStage('cli-build',b.program,b.argv,b.cwd,env,Math.min(LIMITS.stage_timeout_ms,remaining('CLI build')),LIMITS.stage_output_bytes,
        path.join(p.runtime.logs_dir,'cli-build.stdout.raw'),path.join(p.runtime.logs_dir,'cli-build.stderr.raw'));stages.push(build);
      if(build.stage_exit_ok&&!externalSignal&&performance.now()<deadline) {
        for(const [label,x] of [['v1',p.commands.cli_exercise],['v2',p.commands.cli_exercise_v2]]) {
          const budget=Math.min(x.timeout_ms,remaining('CLI '+label+' exercise'));
          const cli=await runStage('cli-exercise-'+label,x.program,x.argv,x.cwd,env,budget,x.output_cap_bytes,
            x.stdout_path,x.stderr_path);stages.push(cli);
          if(!cli.stage_exit_ok||externalSignal||performance.now()>=deadline) break;
          reports.push(checkCliReport(x.stdout_path,x.expected_report_fields));
        }
      }
    }
  }
  let after=null,afterError=null;try{after=snapshot(p);}catch(e){afterError=String(e&&e.stack||e);}
  const elapsed=performance.now()-started,withinDeadline=elapsed<=overallTimeout;
  const stable=Boolean(after&&jsonHash(before)===jsonHash(after));
  const allStages=stages.length===4&&stages.every(x=>x.stage_exit_ok);
  const pass=Boolean(allStages&&identities&&identities.valid&&build&&reports.length===2&&stable&&withinDeadline&&!externalSignal&&!afterError);
  const result={schema:'termrock-vis08-nextest-reader-execution-result/v3',state:pass?'PASS':'FAIL',runner:{path:__filename,sha256:fileHash(__filename)},
    packet:{path:PACKET_PATH,sha256:PACKET_SHA256},package:{root:p.candidate.package,digest:before.package.digest,file_count:before.package.file_count},
    preflight_path:path.join(p.runtime.attempt_root,'preflight.json'),cache_copy:cacheCopy,tool_versions:versions,stages,nextest_identities:identities,
    cli_reports:reports,source_cache_tool_fixture_inputs_unchanged:stable,after_snapshot_error:afterError,external_signal:externalSignal,
    overall_deadline:{timeout_ms:overallTimeout,elapsed_ms:elapsed,within_deadline:withinDeadline,clock:'monotonic performance.now',cleanup_grace_ms:LIMITS.term_grace_ms+LIMITS.post_kill_close_timeout_ms},
    raw_process_events:{path:path.join(p.runtime.logs_dir,'process-events.jsonl'),sha256:fileHash(path.join(p.runtime.logs_dir,'process-events.jsonl'))},
    limitations:{product_behavior:'NOT_TESTED',product_acceptance:'NOT_ESTABLISHED',reader_unit_subset:'23 exact nextest_result::tests identities',synthetic_cli_contracts:['v1 nine-case','v2 fifty-nine-case'],overall_reader_result:pass}};
  writeNew(path.join(p.runtime.attempt_root,'runner-result.json'),result);
  fs.closeSync(eventFd);eventFd=null;
  process.stdout.write((pass?'PASS':'FAIL')+' '+JSON.stringify({wrapper_exit:pass?0:125,result_path:path.join(p.runtime.attempt_root,'runner-result.json'),nextest_child_exit:nextest.close?.code,selected_started:identities?.exact_started??false,selected_terminal_ok:identities?.exact_terminal_ok??false,cli_reports:reports.length,elapsed_ms:elapsed,within_deadline:withinDeadline})+'\n');
  process.exitCode=pass?0:125;
}
const mode=process.argv[2];
try {
  if(mode==='--preflight'&&process.argv.length===3) preflight();
  else if(mode==='--run'&&process.argv.length===3) execute().catch(e=>{process.stderr.write(String(e&&e.stack||e)+'\n');process.exitCode=125;});
  else fail('usage: node runner.cjs --preflight | --run');
} catch(e) { process.stderr.write(String(e&&e.stack||e)+'\n');process.exitCode=125; }
