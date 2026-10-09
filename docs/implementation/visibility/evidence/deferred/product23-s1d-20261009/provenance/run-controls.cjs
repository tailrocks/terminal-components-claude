"use strict";
const fs=require("node:fs");
const path=require("node:path");
const crypto=require("node:crypto");
const {spawn,execFileSync}=require("node:child_process");
const BASE=__dirname;
const MAX_LOG_BYTES=16*1024*1024;
const TIMEOUT_MS=600000;
const TERM_GRACE_MS=5000;
const POST_KILL_GRACE_MS=1000;
const RUSTUP="/Users/donbeave/.cargo/bin/rustup";
const RUSTUP_HOME="/Users/donbeave/.rustup";
const CARGO="/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/cargo";
const NEXTTEST="/Users/donbeave/.local/share/mise/installs/aqua-nextest-rs-nextest-cargo-nextest/0.9.146/cargo-nextest";
const CARGO_HOME="/private/tmp/termrock-vis06-cargo-home-vis06-holla-20261009-44968e82-6442-42c6-a5b4-8aaac2bc64fa";
const FIXED_PATH=[path.dirname(NEXTTEST),"/Users/donbeave/.cargo/bin","/usr/local/bin","/opt/homebrew/bin","/usr/bin","/bin"].join(":");
const sha=b=>crypto.createHash("sha256").update(b).digest("hex");
const iso=d=>d.toISOString();
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const readJson=p=>JSON.parse(fs.readFileSync(p,"utf8"));
function writeAll(fd,bytes){let offset=0;while(offset<bytes.length)offset+=fs.writeSync(fd,bytes,offset,bytes.length-offset);}
function verifyFileList(root,files,label){
  const expected=new Set(files.map(f=>f.path));
  const actual=[];
  function walk(dir,rel=""){
    for(const name of fs.readdirSync(dir).sort()){
      const full=path.join(dir,name),next=path.posix.join(rel,name),st=fs.lstatSync(full);
      if(st.isDirectory())walk(full,next);else actual.push(next);
    }
  }
  walk(root);
  actual.sort();
  if(JSON.stringify(actual)!==JSON.stringify([...expected].sort()))throw new Error(label+" file inventory drift");
  for(const f of files){
    const p=path.join(root,f.path),st=fs.lstatSync(p);
    if(f.type==="file"){
      if(!st.isFile()||st.isSymbolicLink())throw new Error(label+" is not regular: "+f.path);
      const bytes=fs.readFileSync(p);
      if(bytes.length!==f.bytes||sha(bytes)!==f.sha256)throw new Error(label+" digest drift: "+f.path);
    }else if(f.type==="symlink"){
      if(!st.isSymbolicLink()||fs.readlinkSync(p)!==f.target)throw new Error(label+" symlink drift: "+f.path);
    }else throw new Error(label+" has invalid entry: "+f.path);
  }
}
function makeArgs(plan,stage){
  const base=["run","1.98.1","cargo","nextest",stage,
    "--manifest-path",path.join(plan.source_root,"Cargo.toml"),
    "--locked","--offline","--package","termrock-conformance","--test","control_states",
    "--build-jobs","2","--run-ignored","all","--user-config-file","none",
    "--ignore-default-filter","--color","never","--show-progress","none",
    "--filterset",plan.filterset,"--cargo-quiet"];
  if(stage==="list")return base.concat(["--message-format","json"]);
  if(stage!=="run")throw new Error("invalid Nextest stage: "+stage);
  return base.concat(["--no-tests","fail","--no-fail-fast","--retries","0","--test-threads","1",
    "--flaky-result","fail","--failure-output","immediate-final","--status-level","all",
    "--final-status-level","all","--no-output-indent","--message-format","libtest-json-plus",
    "--message-format-version","0.1"]);
}
function processFixedEnv(base){
  const scratch=path.join(base,"scratch");fs.mkdirSync(path.join(scratch,"home"),{recursive:true});
  return {PATH:FIXED_PATH,LANG:"C",LC_ALL:"C",TERM:"dumb",HOME:path.join(scratch,"home"),CARGO,
    CARGO_HOME,CARGO_TARGET_DIR:path.join(base,"target"),CARGO_BUILD_JOBS:"2",CARGO_TERM_COLOR:"never",
    CARGO_NET_OFFLINE:"true",TMPDIR:path.join(base,"scratch"),RUSTUP_HOME,RUSTUP_TOOLCHAIN:"1.98.1",
    NEXTEST_EXPERIMENTAL_LIBTEST_JSON:"1"};
}
function normalizeName(name,names){
  if(typeof name!=="string")throw new Error("Nextest event missing test name");
  if(names.includes(name))return name;
  const matches=names.filter(n=>name.endsWith("$"+n)||name.endsWith("::"+n));
  if(matches.length===1)return matches[0];
  throw new Error("unexpected Nextest test name: "+name);
}
function parseList(data,expectedRows){
  if(!data||typeof data!=="object"||!data["rust-suites"]||typeof data["rust-suites"]!=="object")throw new Error("Nextest list has no rust-suites object");
  const expected=new Map(expectedRows.map(r=>[r.test_name,r.registered_ignored]));
  const selected=new Map();let selectedSuites=0;let targetSuite=null;
  for(const suite of Object.values(data["rust-suites"])){
    if(!suite||typeof suite!=="object"||!suite.testcases||typeof suite.testcases!=="object")continue;
    const matching=Object.entries(suite.testcases).filter(([,m])=>m&&m["filter-match"]&&m["filter-match"].status==="matches");
    if(!matching.length)continue;
    selectedSuites++;
    if(suite["package-name"]!=="termrock-conformance"||suite["binary-name"]!=="control_states"||suite.kind!=="test")throw new Error("unexpected selected suite");
    targetSuite=suite;
    for(const [name,metadata] of matching){
      if(selected.has(name))throw new Error("duplicate selected test "+name);
      if(typeof metadata.ignored!=="boolean")throw new Error("ignored metadata missing for "+name);
      selected.set(name,metadata.ignored);
    }
  }
  if(selectedSuites!==1)throw new Error("expected exactly one selected control_states test suite");
  if(selected.size!==23||selected.size!==expected.size)throw new Error("selected count is not exactly 23");
  for(const [name,ignored] of expected){
    if(!selected.has(name))throw new Error("expected test not selected: "+name);
    if(selected.get(name)!==ignored)throw new Error("ignored metadata drift: "+name);
  }
  if(!Number.isInteger(data["test-count"])||data["test-count"]<23)throw new Error("invalid Nextest total test-count");
  const allNames=new Set(Object.keys(targetSuite.testcases));
  return {selected:Object.fromEntries(selected),allNames:[...allNames].sort(),totalCount:data["test-count"]};
}
function parseRunEvents(bytes,rows,knownNames){
  const names=rows.map(r=>r.test_name),known=new Set(knownNames),started=new Set(),terminal=new Map(),statuses=new Map(),errors=[];
  let decoded;
  try{decoded=bytes.toString("utf8");}catch(e){return {statuses,errors:["run event output decode failed: "+String(e)]};}
  for(const [index,line] of decoded.split(/\r?\n/).entries()){
    if(!line.trim())continue;
    let event;
    try{event=JSON.parse(line);}catch(e){errors.push("invalid JSON event line "+(index+1));continue;}
    if(!event||event.type!=="test")continue;
    const rawName=event.name;
    let basename;
    if(typeof rawName!=="string"){errors.push("event has no test name");continue;}
    basename=rawName.includes("$")?rawName.split("$").at(-1):rawName.split("::").at(-1);
    if(!names.includes(basename)){
      errors.push((known.has(basename)?"unselected":"unknown")+" test emitted event: "+basename+" ("+event.event+")");
      continue;
    }
    let name;
    try{name=normalizeName(rawName,names);}catch(e){errors.push(String(e));continue;}
    if(event.event==="started"){started.add(name);continue;}
    const result={ok:"passed",failed:"failed",ignored:"skipped",timeout:"incomplete",cancelled:"incomplete","exec-failed":"incomplete"}[event.event];
    if(!result)continue;
    if(terminal.has(name)){
      errors.push("duplicate terminal event for "+name+": "+terminal.get(name)+" then "+event.event);
      statuses.set(name,(statuses.get(name)==="failed"||result==="failed")?"failed":"incomplete");
    }else{terminal.set(name,event.event);statuses.set(name,result);}
  }
  for(const name of started)if(!statuses.has(name))statuses.set(name,"incomplete");
  for(const name of names)if(!statuses.has(name))errors.push("missing terminal result for "+name);
  if(statuses.size!==23)errors.push("expected 23 terminal outcomes, got "+statuses.size);
  return {statuses,errors};
}
function groupSignal(pid,signal,errors){
  if(!pid)return;
  try{process.kill(-pid,signal);}catch(e){if(e.code!=="ESRCH"){errors.push(signal+": "+String(e));try{process.kill(pid,signal);}catch(f){if(f.code!=="ESRCH")errors.push(signal+" fallback: "+String(f));}}}
}
async function main(){
  const preflightOnly=process.argv.length===3&&process.argv[2]==="--preflight-only";
  if(process.argv.length!==2&&!preflightOnly)throw new Error("usage: run-controls.cjs [--preflight-only]");
  const started=new Date(),runId=started.toISOString().replace(/[:.]/g,"-")+"-"+crypto.randomUUID().slice(0,8);
  const runDir=path.join(BASE,"runs",runId);fs.mkdirSync(runDir,{recursive:true});
  const logPaths={list_stdout:path.join(runDir,"nextest-list.stdout.json"),list_stderr:path.join(runDir,"nextest-list.stderr.log"),run_stdout:path.join(runDir,"nextest-run.stdout.jsonl"),run_stderr:path.join(runDir,"nextest-run.stderr.log")};
  const fds={};for(const [name,p]of Object.entries(logPaths))fds[name]=fs.openSync(p,"wx");
  const streams=Object.fromEntries(Object.keys(logPaths).map(k=>[k,{received:0,stored:0,truncated:false,write_failed:false}]));
  const cleanupErrors=[],logErrors=[];let totalStored=0,interruptedBy=null,activePid=null;
  const receiptPath=path.join(runDir,"receipt.json");
  const names=readJson(path.join(BASE,"test-names.json"));
  const stableRows=readJson(path.join(BASE,"stable-case-map.json"));
  const receipt={schema:"termrock-vis04-product23-run-receipt/v1",run_id:runId,started_at:iso(started),finished_at:null,
    source:{candidate_product_commit:null,source_archive_sha256:null,source_manifest_sha256:null,runplan_sha256:null,runner_sha256:null,launch_script_sha256:null,preflight_sha256:null,contract_reference_sha256:null},
    toolchain:{rust:"1.98.1",cargo_nextest:"0.9.146",build_jobs:2,test_threads:1,offline:true},
    selection:{package:"termrock-conformance",binary:"control_states",expected_count:23,selected_names:names,filterset:null,run_ignored:"all",registered_normal_count:22,registered_ignored_ids:["BD-21"],listed_total_count:null,listed_test_metadata:[]},
    command:{program:RUSTUP,list_argv:null,run_argv:null,cwd:null},
    environment:{inherits_parent:false,cargo_net_offline:true,CARGO,cargo_home:CARGO_HOME,cargo_target_dir:path.join(BASE,"target"),tmpdir:path.join(BASE,"scratch"),RUSTUP_HOME,RUSTUP_TOOLCHAIN:"1.98.1",PATH:FIXED_PATH},
    bounds:{timeout_ms:TIMEOUT_MS,term_grace_ms:TERM_GRACE_MS,combined_log_cap_bytes:MAX_LOG_BYTES},
    listing:{result:"NOT_RUN",exit_code:null,signal:null,timed_out:false,spawn_error:null,selected_names:[],all_target_names:[]},
    execution:{result:"BLOCKED_PREFLIGHT",exit_code:null,signal:null,timed_out:false,interrupted_by:null,spawn_error:null,nextest_run_id:null,elapsed_ms:null,run_started:false,expected_failure_treatment:"none",case_results:stableRows.map(r=>({...r,status:"not_run"})),parse_errors:[]},
    artifacts:null,cleanup_errors:[],log_errors:[]};
  function capture(name,chunk){
    const s=streams[name];s.received+=chunk.length;
    if(s.write_failed)return;
    const allowed=Math.max(0,MAX_LOG_BYTES-totalStored),kept=chunk.subarray(0,Math.min(chunk.length,allowed));
    if(kept.length){try{writeAll(fds[name],kept);s.stored+=kept.length;totalStored+=kept.length;}catch(e){s.write_failed=true;logErrors.push(name+": "+String(e));}}
    if(kept.length<chunk.length)s.truncated=true;
  }
  function finalize(exitStatus){
    const ended=new Date();receipt.finished_at=iso(ended);receipt.execution.elapsed_ms=ended.getTime()-started.getTime();
    receipt.cleanup_errors=cleanupErrors;receipt.log_errors=logErrors;
    for(const fd of Object.values(fds)){try{fs.closeSync(fd);}catch(e){cleanupErrors.push(String(e));}}
    const artifacts={};
    for(const [name,p]of Object.entries(logPaths)){const b=fs.readFileSync(p);artifacts[name]={path:path.relative(BASE,p),bytes:streams[name].stored,bytes_received:streams[name].received,sha256:sha(b),truncated:streams[name].truncated,write_failed:streams[name].write_failed};}
    receipt.artifacts={logs:artifacts,total_bytes_stored:totalStored,total_cap_bytes:MAX_LOG_BYTES};
    const tmp=receiptPath+".tmp";fs.writeFileSync(tmp,JSON.stringify(receipt,null,2)+"\n",{flag:"wx"});fs.renameSync(tmp,receiptPath);
    console.log(JSON.stringify({receipt:receiptPath,result:receipt.execution.result,exit_code:receipt.execution.exit_code,artifacts:receipt.artifacts},null,2));process.exitCode=exitStatus;
  }
  function verifyBindings(){
    const manifestBytes=fs.readFileSync(path.join(BASE,"source-manifest.json")),planBytes=fs.readFileSync(path.join(BASE,"runplan.json")),preflightBytes=fs.readFileSync(path.join(BASE,"preflight.json"));
    const manifest=JSON.parse(manifestBytes),plan=JSON.parse(planBytes),preflight=JSON.parse(preflightBytes);
    verifyFileList(manifest.source_root,manifest.source_files,"product source");verifyFileList(manifest.contract_reference.root,manifest.contract_reference.files,"contract source");
    const archiveBytes=fs.readFileSync(manifest.archive_path);if(sha(archiveBytes)!==manifest.archive_sha256)throw new Error("S1d archive hash drift");
    const mh=sha(manifestBytes),ph=sha(planBytes),rh=sha(fs.readFileSync(__filename)),lh=sha(fs.readFileSync(path.join(BASE,"launch.sh")));
    if(preflight.result!=="PASS_PRODUCT23_STATIC_PREFLIGHT_NO_CARGO")throw new Error("static preflight is not PASS");
    if(preflight.source_manifest_sha256!==mh||preflight.runplan_sha256!==ph||preflight.runner_sha256!==rh||preflight.launch_script_sha256!==lh)throw new Error("preflight bindings differ");
    if(plan.source_manifest_sha256!==mh||plan.runner_sha256!==rh||plan.launch_script_sha256!==lh)throw new Error("runplan bindings differ");
    if(plan.cargo_path!==CARGO||!fs.statSync(CARGO).isFile())throw new Error("pinned Cargo executable mismatch");
    if(plan.candidate_product_commit!==manifest.candidate_product_commit||plan.selected_count!==23||plan.normal_count!==22||JSON.stringify(plan.ignored_ids)!==JSON.stringify(["BD-21"])||plan.expected_failure_treatment!=="none")throw new Error("plan metadata mismatch");
    if(JSON.stringify(names)!==JSON.stringify(plan.selected_names)||JSON.stringify(stableRows)!==JSON.stringify(plan.stable_case_map))throw new Error("selected names/case map drift");
    const filter= "package(=termrock-conformance) & binary(=control_states) & ("+names.map(n=>"test(="+n+")").join(" | ")+") - test(/(?i)(admission|approve|baseline|capture|snapshot|publish|write|rebuild_review_html)/)";
    if(filter!==plan.filterset)throw new Error("exact package/filter contract mismatch");
    const listArgs=makeArgs(plan,"list"),runArgs=makeArgs(plan,"run");
    if(JSON.stringify(listArgs)!==JSON.stringify(plan.list_argv)||JSON.stringify(runArgs)!==JSON.stringify(plan.run_argv))throw new Error("frozen argv differs from supervisor argv");
    const recordBytes=fs.readFileSync(path.join(BASE,"record-ledger.cjs"));if(sha(recordBytes)!==plan.ledger_recording.record_script_sha256)throw new Error("separate ledger record script drift");
    const version=execFileSync(NEXTTEST,["--version"],{encoding:"utf8",timeout:5000,maxBuffer:4096,env:{PATH:FIXED_PATH,LANG:"C",LC_ALL:"C",TERM:"dumb",CARGO_HOME,RUSTUP_HOME}}).trim().split("\n",1)[0];
    if(version!==plan.cargo_nextest_version)throw new Error("Nextest version drift");
    receipt.source={candidate_product_commit:manifest.candidate_product_commit,source_archive_sha256:manifest.archive_sha256,source_manifest_sha256:mh,runplan_sha256:ph,runner_sha256:rh,launch_script_sha256:lh,preflight_sha256:sha(preflightBytes),contract_reference_sha256:sha(Buffer.from(JSON.stringify(manifest.contract_reference.files)))};
    receipt.selection.filterset=plan.filterset;receipt.command.cwd=manifest.source_root;receipt.command.list_argv=listArgs;receipt.command.run_argv=runArgs;
    receipt.execution.case_results=stableRows.map(r=>({...r,status:"not_run"}));
    return {manifest,plan,env:processFixedEnv(BASE)};
  }
  const deadline=Date.now()+TIMEOUT_MS;
  let activeTerminator=null;
  async function runStage(name,args,env){
    const remaining=Math.max(0,deadline-Date.now());if(!remaining)return {code:null,signal:null,timed_out:true,interrupted_by:interruptedBy,spawn_error:null,forced_close:false};
    let timedOut=false,spawnError=null,settled=false,terminationStarted=false,killTimer=null,postKillTimer=null,timer=null;
    let resolveStatus,resolveTermination;
    const statusPromise=new Promise(resolve=>{resolveStatus=status=>{if(settled)return;settled=true;if(timer)clearTimeout(timer);resolve(status);};});
    const terminationCleanup=new Promise(resolve=>resolveTermination=resolve);
    let child;
    try{child=spawn(RUSTUP,args,{cwd:receipt.command.cwd,env,detached:true,stdio:["ignore","pipe","pipe"]});activePid=child.pid;}
    catch(e){return {code:null,signal:null,timed_out:false,interrupted_by:interruptedBy,spawn_error:String(e),forced_close:false};}
    child.stdout.on("data",chunk=>capture(name+"_stdout",chunk));child.stderr.on("data",chunk=>capture(name+"_stderr",chunk));
    child.on("error",e=>{spawnError=String(e);});child.on("close",(code,signal)=>resolveStatus({code,signal,forced_close:false}));
    function terminateGroup(){
      if(terminationStarted)return;
      terminationStarted=true;groupSignal(child.pid,"SIGTERM",cleanupErrors);
      killTimer=setTimeout(()=>{
        groupSignal(child.pid,"SIGKILL",cleanupErrors);
        postKillTimer=setTimeout(()=>{
          if(!settled){
            try{child.stdout.destroy();}catch(e){cleanupErrors.push("stdout destroy: "+String(e));}
            try{child.stderr.destroy();}catch(e){cleanupErrors.push("stderr destroy: "+String(e));}
            try{child.unref();}catch(e){cleanupErrors.push("child unref: "+String(e));}
            resolveStatus({code:null,signal:"SIGKILL",forced_close:true});
          }
          resolveTermination();
        },POST_KILL_GRACE_MS);
      },TERM_GRACE_MS);
    }
    activeTerminator=terminateGroup;
    timer=setTimeout(()=>{timedOut=true;terminateGroup();},remaining);
    const status=await statusPromise;
    if(timedOut||interruptedBy){terminateGroup();await terminationCleanup;}
    else{if(killTimer)clearTimeout(killTimer);if(postKillTimer)clearTimeout(postKillTimer);}
    if(activeTerminator===terminateGroup)activeTerminator=null;
    activePid=null;
    return {code:status.code,signal:status.signal,timed_out:timedOut,interrupted_by:interruptedBy,spawn_error:spawnError,forced_close:status.forced_close===true};
  }
  const onSignal=sig=>{if(interruptedBy)return;interruptedBy=sig;if(activeTerminator)activeTerminator();};
  process.on("SIGINT",()=>onSignal("SIGINT"));process.on("SIGTERM",()=>onSignal("SIGTERM"));
  try{
    const {manifest,plan,env}=verifyBindings();
    if(preflightOnly){receipt.execution.result="PASS_RUNTIME_PREFLIGHT_NO_CARGO";receipt.execution.expected_failure_treatment="none";finalize(0);return;}
    const listArgs=makeArgs(plan,"list");
    const listStatus=await runStage("list",listArgs,env);receipt.listing={result:listStatus.code===0?"LIST_EXIT_ZERO":"LIST_NONZERO",...listStatus};
    if(listStatus.timed_out||listStatus.interrupted_by||listStatus.spawn_error||listStatus.code!==0){receipt.execution.result=listStatus.timed_out?"BLOCKED_TIMEOUT":listStatus.interrupted_by?"BLOCKED_SIGNAL":"BLOCKED_LIST";receipt.execution.interrupted_by=listStatus.interrupted_by;finalize(listStatus.timed_out?124:listStatus.interrupted_by==="SIGINT"?130:2);return;}
    if(streams.list_stdout.truncated||streams.list_stderr.truncated||logErrors.length){receipt.execution.result="BLOCKED_CAPTURE_LIMIT_OR_IO";finalize(2);return;}
    const list=JSON.parse(fs.readFileSync(logPaths.list_stdout,"utf8"));
    const parsedList=parseList(list,stableRows);receipt.listing.result="LIST_SELECTION_MATCHED";receipt.listing.selected_names=Object.keys(parsedList.selected).sort();receipt.listing.all_target_names=parsedList.allNames;receipt.listing.total_count=parsedList.totalCount;receipt.listing.selected_metadata=parsedList.selected;
    receipt.selection.listed_total_count=parsedList.totalCount;receipt.selection.listed_test_metadata=stableRows.map(r=>({id:r.id,test_name:r.test_name,ignored:parsedList.selected[r.test_name]}));
    verifyFileList(manifest.source_root,manifest.source_files,"product source after list");verifyFileList(manifest.contract_reference.root,manifest.contract_reference.files,"contract source after list");
    const runArgs=makeArgs(plan,"run");receipt.execution.run_started=true;const runStatus=await runStage("run",runArgs,env);
    receipt.execution.exit_code=runStatus.code;receipt.execution.signal=runStatus.signal;receipt.execution.timed_out=runStatus.timed_out;receipt.execution.interrupted_by=runStatus.interrupted_by;receipt.execution.spawn_error=runStatus.spawn_error;receipt.execution.forced_close=runStatus.forced_close===true;
    verifyFileList(manifest.source_root,manifest.source_files,"product source after run");verifyFileList(manifest.contract_reference.root,manifest.contract_reference.files,"contract source after run");
    const runBytes=fs.readFileSync(logPaths.run_stdout);const runStderr=fs.readFileSync(logPaths.run_stderr,"utf8");const runIds=[...runStderr.matchAll(/Nextest run ID ([0-9a-f-]{36}) with nextest profile/g)].map(m=>m[1]);if(runIds.length===1)receipt.execution.nextest_run_id=runIds[0];else receipt.execution.parse_errors.push("expected exactly one Nextest run ID, got "+runIds.length);const parsed=parseRunEvents(runBytes,stableRows,parsedList.allNames);receipt.execution.parse_errors.push(...parsed.errors);
    receipt.execution.case_results=stableRows.map(row=>({...row,status:parsed.statuses.get(row.test_name)||"not_run"}));
    const values=[...parsed.statuses.values()],failed=values.includes("failed"),incomplete=values.some(v=>v!=="passed")||values.length!==23;
    if(runStatus.timed_out)receipt.execution.result="BLOCKED_TIMEOUT";
    else if(runStatus.interrupted_by||runStatus.signal)receipt.execution.result="BLOCKED_SIGNAL";
    else if(runStatus.spawn_error)receipt.execution.result="BLOCKED_SPAWN";
    else if(streams.run_stdout.truncated||streams.run_stderr.truncated||logErrors.length)receipt.execution.result="BLOCKED_CAPTURE_LIMIT_OR_IO";
    else if(failed)receipt.execution.result="FAILED";
    else if(receipt.execution.parse_errors.length||parsed.errors.length||incomplete)receipt.execution.result="BLOCKED_INCOMPLETE_OR_PARSE";
    else if(runStatus.code===0&&values.length===23&&values.every(v=>v==="passed"))receipt.execution.result="PASSED_23";
    else receipt.execution.result="BLOCKED_NONZERO_WITHOUT_CASE_FAILURE";
    if(runStatus.timed_out)finalize(124);else if(runStatus.interrupted_by==="SIGINT")finalize(130);else if(runStatus.interrupted_by==="SIGTERM")finalize(143);else if(receipt.execution.result==="PASSED_23")finalize(0);else if(receipt.execution.result==="FAILED")finalize(1);else finalize(2);
  }catch(error){receipt.execution.result="BLOCKED_PREFLIGHT_OR_SELECTION";receipt.execution.spawn_error=String(error);finalize(2);}
}
main().catch(error=>{console.error(String(error));process.exitCode=2;});
