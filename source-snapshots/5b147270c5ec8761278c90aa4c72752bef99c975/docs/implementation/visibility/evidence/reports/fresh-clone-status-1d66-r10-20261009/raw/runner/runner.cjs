'use strict';
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const cp = require('child_process');
const {spawn} = require('child_process');
const mode = process.argv[2];
if (!['preflight','execute'].includes(mode)) throw new Error('usage: node runner.cjs preflight|execute');
const base = '/private/tmp/termrock-vis01-status-fresh-clone-1d66f444-20261009';
const repo = path.join(base,'repo');
const crate = path.join(repo,'crates/termrock-visibility-tests');
const runDir = path.join(base,'run-r10');
const resultDir = path.join(runDir,'result');
const homeDir = path.join(runDir,'home');
const tmpDir = path.join(runDir,'tmp');
const targetDir = path.join(base,'target-r10');
const cargoHome = '/private/tmp/termrock-vis04-deferred-controls-b106-cache-repair-20261009-v7.Qs1c13/cargo-home';
const cacheManifestPath = '/private/tmp/termrock-vis04-deferred-controls-b106-cache-repair-20261009-v7.Qs1c13/cargo-cache-manifest.json';
const toolchain = '/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin';
const rustc = path.join(toolchain,'rustc');
const cargo = path.join(toolchain,'cargo');
const python = '/usr/bin/python3';
const nextest = '/Users/donbeave/.local/share/mise/installs/aqua-nextest-rs-nextest-cargo-nextest/0.9.146/cargo-nextest';
const nodeBin = '/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node';
const filterset = 'test(=checked_in_records_render_from_exact_copies)';
const timeoutMs = 600000;
const killGraceMs = 5000;
const killCloseDeadlineMs = 5000;
const maxStreamBytes = 8 * 1024 * 1024;
const expected = {
  commit: '1d66f444f5962414b7c33734fa37376471e5f853',
  tree: '0ae9e285e8f771b8e250c63bf80f902d112dcd95',
  lock: 'd6372475262b723ebdfe3bec9134ba6c675ff1d3becb3fa4625eb9e38c40b2c2',
  nextestSha256: '7a558b157d164ab4fb6cb1a48cbac5a57b7b8ad99f5d3492eb6eda64faf91df0',
  cargoSha256: '6e17e865f3a20dd55a1d212f849f58b77124179f0de7c52973096d84ba34118d',
  rustcSha256: '766eda9d8f53afd6fc7f27b3cd2e444dd22afacb5afa710a5625fc8e45b8c941',
  pythonSha256: '34129c71a01a74f7f3b2443521519b2e5447553fa187f5fcafaaf8c42cc192e2',
  nodeSha256: '9d050fd455b56426e25d4d603c7c501cbb2630348e836cf221dcce748e90588a',
  cacheManifestSha256: '64ca31c14e9d9cbbc8c6f6ce8ad69dc1014e6955669c3eae079a9a9a64190b71',
  testFileSha256: '0dd5dbe15068259922cf23f7865e32ba0593523fb8ad0302ed04accdf2724a72',
  statusPySha256: '540358d4afafa0fec7723a1291a01ee0ff8efec05b0dd1bd371c8fc5ac791da4',
  factsSha256: 'bc765653690ee7f3e26132ce8b404e57cf5706c45d36548b496c12e923f6e6a6',
  archiveManifestSha256: '25c3f3a8791c34ecd9f7d231832f4058dd6cd34a6cd5040caab508ad31a1243f',
  nextestConfigSha256: '688eecbcf066e5fefd6ae71cd711390cf245c04ce11a03928c187660b28b9b7e',
};
const sha = b => crypto.createHash('sha256').update(b).digest('hex');
function git(...args) {
  const p = cp.spawnSync('git',['-C',repo,...args],{encoding:'buffer',env:{...process.env,GIT_OPTIONAL_LOCKS:'0'}});
  if (p.status !== 0) throw new Error(`git ${args.join(' ')} failed: ${p.stderr.toString()}`);
  return p.stdout.toString();
}
function checkInputs() {
  const problems=[];
  const need=(ok,msg)=>{if(!ok)problems.push(msg);};
  const head=git('rev-parse','HEAD').trim();
  const tree=git('rev-parse','HEAD^{tree}').trim();
  const branch=git('branch','--show-current').trim();
  const status=git('status','--porcelain=v2','--untracked-files=all');
  need(head===expected.commit,'clone commit mismatch');
  need(tree===expected.tree,'clone tree mismatch');
  need(branch==='termrock-implementation','clone branch mismatch');
  need(status==='','clone worktree is not clean');
  need(git('rev-parse','--is-shallow-repository').trim()==='true','clone is not shallow');
  const fileHashes={
    'crates/termrock-visibility-tests/Cargo.lock':expected.lock,
    'crates/termrock-visibility-tests/tests/status.rs':expected.testFileSha256,
    'tools/visibility/status.py':expected.statusPySha256,
    'tools/visibility/source-facts.json':expected.factsSha256,
    'docs/implementation/visibility/evidence/reports/status-source-archive-20261009/MANIFEST.json':expected.archiveManifestSha256,
    '.config/nextest.toml':expected.nextestConfigSha256,
  };
  const observed={};
  for (const [rel,want] of Object.entries(fileHashes)) {
    const bytes=fs.readFileSync(path.join(repo,rel)); observed[rel]=sha(bytes); need(observed[rel]===want,`source hash mismatch: ${rel}`);
  }
  const lockSha=observed['crates/termrock-visibility-tests/Cargo.lock'];
  const test=fs.readFileSync(path.join(crate,'tests/status.rs'),'utf8');
  const exactCount=(test.match(/fn checked_in_records_render_from_exact_copies\s*\(/g)||[]).length;
  need(exactCount===1,'selected test definition count is not one');
  need(test.includes('StatusFixture::from_checked_in_records()'),'selected case does not use checked-in records fixture');
  need(test.includes('let repeated = fixture.run(&[]);')&&test.includes('assert_eq!(output.stdout, repeated.stdout);'),'selected case does not compare repeated CLI output');
  const lib=fs.readFileSync(path.join(crate,'src/lib.rs'),'utf8');
  need(lib.includes('Path::new(env!("CARGO_MANIFEST_DIR"))')&&lib.includes('.join("../..")'),'test fixture does not derive source root from crate manifest directory');
  const cacheBytes=fs.readFileSync(cacheManifestPath),cm=JSON.parse(cacheBytes);
  need(sha(cacheBytes)===expected.cacheManifestSha256,'qualified cache manifest hash changed');
  need(cm.state==='COMPLETE_VERIFIED_ARCHIVE_AND_INDEX_CLOSURE','qualified cache is not complete');
  need(cm.cargo_lock_sha256===lockSha&&lockSha===expected.lock,'qualified cache lock differs');
  need(path.resolve(cm.destination_cargo_home)===path.resolve(cargoHome),'qualified cache home differs');
  need(cm.packages.length===32&&cm.locked_registry_packages===32&&cm.copied_index_entries===32&&cm.copied_archives===32,'qualified cache is not 32+32');
  const cacheMembers=[];
  for(const p of cm.packages){
    const a=fs.readFileSync(path.join(cargoHome,p.archive_path));
    const i=fs.readFileSync(path.join(cargoHome,p.index_path));
    const archiveSha=sha(a),indexSha=sha(i),ok=archiveSha===p.archive_sha256&&indexSha===p.index_sha256;
    need(ok,`cache archive/index mismatch: ${p.name}@${p.version}`);
    cacheMembers.push({name:p.name,version:p.version,archive_sha256:archiveSha,index_sha256:indexSha,matches:ok});
  }
  const archiveMemberCount=JSON.parse(fs.readFileSync(path.join(repo,'docs/implementation/visibility/evidence/reports/status-source-archive-20261009/MANIFEST.json'))).files.length;
  need(archiveMemberCount===59,'archive manifest member count changed');
  need(sha(fs.readFileSync(nextest))===expected.nextestSha256,'pinned cargo-nextest binary hash changed');
  need(fs.existsSync(cargo)&&fs.existsSync(rustc)&&fs.existsSync(python),'pinned tool binary missing');
  need(sha(fs.readFileSync(cargo))===expected.cargoSha256,'pinned cargo binary hash changed');
  need(sha(fs.readFileSync(rustc))===expected.rustcSha256,'pinned rustc binary hash changed');
  need(sha(fs.readFileSync(python))===expected.pythonSha256,'pinned Python binary hash changed');
  need(process.execPath===nodeBin&&process.version==='v24.20.0','supervisor Node runtime identity changed');
  need(sha(fs.readFileSync(nodeBin))===expected.nodeSha256,'pinned supervisor Node binary hash changed');
  const resolvedPython=cp.spawnSync('/usr/bin/which',['python3'],{encoding:'utf8',env:childEnv});
  need(resolvedPython.status===0&&resolvedPython.stdout.trim()===python,'child PATH does not resolve python3 to pinned system binary');
  for(const p of [path.join(repo,'.cargo/config'),path.join(repo,'.cargo/config.toml'),path.join(crate,'.cargo/config'),path.join(crate,'.cargo/config.toml'),path.join(cargoHome,'config'),path.join(cargoHome,'config.toml')]) need(!fs.existsSync(p),`unexpected Cargo config: ${p}`);
  need(!fs.existsSync(path.join(crate,'.config/nextest.toml')),`unexpected crate-local nextest config`);
  need(!fs.existsSync(targetDir),'isolated target directory already exists');
  if(mode==='preflight'){
    need(!fs.existsSync(resultDir),'result directory already exists');
    need(!fs.existsSync(homeDir)&&!fs.existsSync(tmpDir),'private run HOME/TMPDIR already exists');
  }
  return {problems,head,tree,branch,clean:status==='',shallow:true,fileHashes:observed,lockSha,cacheManifestSha256:sha(cacheBytes),cacheMembers,archiveMemberCount,selectedTestCount:exactCount,nextestSha256:sha(fs.readFileSync(nextest)),toolchain:'1.98.1-aarch64-apple-darwin',pythonVersion:'Python 3.9.6',nodeVersion:process.version,nodePath:process.execPath,nodeSha256:sha(fs.readFileSync(nodeBin)),filterset,targetDir};
}
const argv=['nextest','run','--offline','--locked','--ignore-default-filter','--no-tests','fail','--retries','0','--test-threads','1','-p','termrock-visibility-tests','--test','status','--filterset',filterset];
const childEnv={
  HOME:homeDir,
  TMPDIR:tmpDir,
  PATH:`${toolchain}:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin`,
  RUSTUP_HOME:'/Users/donbeave/.rustup',
  RUSTUP_TOOLCHAIN:'1.98.1-aarch64-apple-darwin',
  CARGO_HOME:cargoHome,
  CARGO_TARGET_DIR:targetDir,
  CARGO_BUILD_JOBS:'2',
  CARGO_NET_OFFLINE:'true',
  CARGO:cargo,
  RUSTC:rustc,
  NEXTEST_TEST_THREADS:'1',
  NEXTEST_RETRIES:'0',
  CARGO_TERM_COLOR:'never',
  NO_COLOR:'1',
  TERM:'dumb',
  TZ:'UTC',
  GIT_OPTIONAL_LOCKS:'0',
};
const report=checkInputs();
if(report.problems.length){console.error(JSON.stringify({mode,result:'BLOCKED_PREFLIGHT',...report},null,2));process.exit(2);}
if(mode==='preflight'){
  console.log(JSON.stringify({mode,result:'PASS_NO_CARGO',...report,invocation:{binary:nextest,args:argv,working_directory:crate,environment:childEnv,overall_timeout_ms:timeoutMs,kill_grace_ms:killGraceMs,post_sigkill_close_deadline_ms:killCloseDeadlineMs,per_stream_limit_bytes:maxStreamBytes},cargo_or_test_executed:false},null,2));
  process.exit(0);
}
for(const p of [resultDir,homeDir,tmpDir]) if(fs.existsSync(p)) throw new Error(`one-shot output path already exists: ${p}`);
fs.mkdirSync(resultDir,{recursive:false,mode:0o700});
fs.mkdirSync(homeDir,{recursive:false,mode:0o700});
fs.mkdirSync(tmpDir,{recursive:false,mode:0o700});
const stdoutPath=path.join(resultDir,'stdout.raw'),stderrPath=path.join(resultDir,'stderr.raw');
const out=fs.createWriteStream(stdoutPath,{flags:'wx',mode:0o600}),err=fs.createWriteStream(stderrPath,{flags:'wx',mode:0o600});
const startedAt=new Date().toISOString();
const child=spawn(nextest,argv,{cwd:crate,env:childEnv,detached:true,stdio:['ignore','pipe','pipe']});
let bytesOut=0,bytesErr=0,storedOut=0,storedErr=0,timeout=false,capExceeded=false,termSent=false,spawnError=null,closed=false,finalized=false;
let hardKillTimer=null,killCloseTimer=null;
function signalGroup(sig){try{process.kill(-child.pid,sig);}catch(e){if(e.code!=='ESRCH')throw e;}}
async function finish(code,signal,closeDeadlineExceeded){
  if(finalized)return;
  finalized=true;closed=true;
  clearTimeout(timer);if(hardKillTimer)clearTimeout(hardKillTimer);if(killCloseTimer)clearTimeout(killCloseTimer);
  if(closeDeadlineExceeded){child.stdout.destroy();child.stderr.destroy();child.unref();}
  out.end();err.end();
  await Promise.all([new Promise(resolve=>out.once('finish',resolve)),new Promise(resolve=>err.once('finish',resolve))]);
  const stdout=fs.readFileSync(stdoutPath),stderr=fs.readFileSync(stderrPath);
  const result={schema:'termrock-vis01-fresh-clone-single-status-case-run/v1',started_at_utc:startedAt,finished_at_utc:new Date().toISOString(),working_directory:crate,argv:[nextest,...argv],environment:childEnv,timeout_ms:timeoutMs,kill_grace_ms:killGraceMs,post_sigkill_close_deadline_ms:killCloseDeadlineMs,close_deadline_exceeded:closeDeadlineExceeded,max_stream_bytes_each:maxStreamBytes,stdout:{path:'stdout.raw',bytes:stdout.length,observed_bytes:bytesOut,sha256:sha(stdout),truncated:bytesOut>stdout.length},stderr:{path:'stderr.raw',bytes:stderr.length,observed_bytes:bytesErr,sha256:sha(stderr),truncated:bytesErr>stderr.length},exit_code:code,signal,timeout,output_limit_exceeded:capExceeded,spawn_error:spawnError,cargo_nextest_retry_count:0,selected_test_filter:filterset,cargo_or_test_attempted:true};
  const success=code===0&&!timeout&&!capExceeded&&!spawnError&&!closeDeadlineExceeded;
  fs.writeFileSync(path.join(resultDir,'run-result.json'),JSON.stringify(result,null,2)+'\n',{flag:'wx',mode:0o600});
  fs.writeFileSync(path.join(resultDir,'supervisor.exit'),String(success?0:1)+'\n',{flag:'wx',mode:0o600});
  process.exitCode=success?0:1;
}
function terminate(reason){
  if(closed||termSent)return;
  termSent=true;
  if(reason==='timeout')timeout=true;
  if(reason==='output_limit')capExceeded=true;
  signalGroup('SIGTERM');
  hardKillTimer=setTimeout(()=>{
    if(closed)return;
    signalGroup('SIGKILL');
    killCloseTimer=setTimeout(()=>{
      if(closed)return;
      finish(null,null,true);
    },killCloseDeadlineMs);
  },killGraceMs);
}
function streamInto(src,dst,name){src.on('data',chunk=>{if(name==='stdout')bytesOut+=chunk.length;else bytesErr+=chunk.length;const stored=name==='stdout'?storedOut:storedErr;const room=Math.max(0,maxStreamBytes-stored);if(room){const part=chunk.subarray(0,room);dst.write(part);if(name==='stdout')storedOut+=part.length;else storedErr+=part.length;}if(chunk.length>room)terminate('output_limit');});}
streamInto(child.stdout,out,'stdout');streamInto(child.stderr,err,'stderr');
const timer=setTimeout(()=>terminate('timeout'),timeoutMs);
child.on('error',e=>{spawnError=String(e);});
child.on('close',(code,signal)=>{finish(code,signal,false);});
