#!/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node
"use strict";
const fs=require("node:fs"), path=require("node:path"), crypto=require("node:crypto");
const P=__dirname, sha=b=>crypto.createHash("sha256").update(b).digest("hex");
function regular(file,cap=8388608){const st=fs.lstatSync(file);if(!st.isFile()||st.isSymbolicLink()||(st.mode&0o077)!==0)throw Error("capture is not a private regular file: "+file);const b=fs.readFileSync(file);if(b.length>cap)throw Error("capture exceeds byte cap: "+file);return {path:file,byte_length:b.length,sha256:sha(b),truncated:false};}
function read(file){const st=fs.lstatSync(file);if(!st.isFile()||st.isSymbolicLink())throw Error("receipt is not a regular file: "+file);return JSON.parse(fs.readFileSync(file,"utf8"));}
function termination(t){if(t.kind==="exited")return {kind:"exited",exit_code:t.exit_code};if(t.kind==="signaled")return {kind:"signal",signal:t.signal};if(t.kind==="timeout")return {kind:"timeout"};throw Error("run capture has no supported termination: "+JSON.stringify(t));}
const list=read(path.join(P,"list-capture-binding.json"));
const run=read(path.join(P,"run-output","run-process.json"));
if(run.schema!=="termrock-nextest-process-capture/v1"||run.stage!=="test_run_capture"||run.semantic_parse!=="NOT_PERFORMED"||run.direct_child_reaped!==true)throw Error("raw run receipt is incomplete or unexpected");
const wrapperTerm=read(path.join(P,"wrapper-capture","termination.json"));
if(wrapperTerm.schema!=="termrock.vis01.wrapper-termination/v1"||wrapperTerm.kind!=="exited"||!Number.isInteger(wrapperTerm.exit_code))throw Error("wrapper termination is incomplete");
const selection=regular(path.join(P,"selection-v2.json"));
if(selection.sha256!=="30a9f314bafaf1f61049a4abb85650da8b344315764b757962cf779892fca924")throw Error("trusted selection changed");
const request={schema:"termrock-nextest-reader-request/v2",invocation:{nextest_version:"0.9.146",message_format:"libtest-json-plus",message_format_version:"0.1",libtest_json_enabled:true,retries:0,stress:"none",partition:"none",machine_stream:"stdout",filter_args:[],run_ignored:"default",run_id:"termrock-vis01-status-59-r1-20261010",expected_test_count:59,targets:[{package:"termrock-visibility-tests",binary_id:"termrock-visibility-tests::status",kind:"test"}]},list_capture:{stdout:regular(list.captures.stdout.path),stderr:regular(list.captures.stderr.path),termination:list.captures.termination},run_capture:{stdout:regular(run.stdout.path),stderr:regular(run.stderr.path),termination:termination(run.termination)},wrapper_capture:{stdout:regular(path.join(P,"wrapper-capture","supervisor.stdout.log")),stderr:regular(path.join(P,"wrapper-capture","supervisor.stderr.log")),termination:{kind:"exited",exit_code:wrapperTerm.exit_code}}};
const out=path.join(P,"reader-request.json");const fd=fs.openSync(out,"wx",0o600);try{fs.writeSync(fd,JSON.stringify(request,null,2)+"\n");}finally{fs.closeSync(fd);}fs.chmodSync(out,0o600);process.stdout.write(JSON.stringify({result:"READER_REQUEST_PREPARED_FROM_RAW_HASHES",path:out,sha256:sha(fs.readFileSync(out))})+"\n");
