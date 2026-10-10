'use strict';
const fs=require('node:fs');
const path=require('node:path');
const crypto=require('node:crypto');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const read=p=>fs.readFileSync(p);
const json=p=>JSON.parse(read(p).toString('utf8'));
const base='/private/tmp/termrock-rust-tool-test-execution/vis19-cursor-r4-nine-tests-r5-20261010';
const receiptPath=base+'/attempt-01/receipt.json';
const runnerPath=base+'/runner.cjs';
const planPath=base+'/runplan.json';
const preflightPath=base+'/preflight.json';
const receipt=json(receiptPath),plan=json(planPath),preflight=json(preflightPath);
const receiptBytes=read(receiptPath),stdout=read(receipt.output.stdout.path),stderr=read(receipt.output.stderr.path),processEvents=read(receipt.output.events_path);
const errors=[];
function pin(pathname,expected,label){const bytes=read(pathname),actual=sha(bytes);if(expected&&actual!==expected)errors.push(label+' sha256 mismatch');return {path:pathname,bytes:bytes.length,sha256:actual};}
const filePins={receipt:pin(receiptPath,null,'receipt'),stdout:pin(receipt.output.stdout.path,receipt.output.stdout.sha256,'stdout'),stderr:pin(receipt.output.stderr.path,receipt.output.stderr.sha256,'stderr'),process_events:pin(receipt.output.events_path,null,'process events'),runner:pin(runnerPath,receipt.runner_sha256,'runner'),runplan:pin(planPath,receipt.plan_sha256,'runplan'),preflight:pin(preflightPath,receipt.preflight_sha256,'preflight')};
if(sha(receiptBytes)!==filePins.receipt.sha256)errors.push('receipt read inconsistency');
if(receipt.status!=='FAIL_OR_INCOMPLETE'||receipt.child?.exit_code!==0||receipt.child?.signal!==null)errors.push('receipt outcome fields differ from observed case');
if(receipt.nextest?.valid!==true||receipt.nextest?.selected_count!==9||receipt.nextest?.selected_passed!==9||receipt.nextest?.selected_failed!==0||receipt.nextest?.selected_other!==0)errors.push('receipt JSON event summary does not report nine selected passes');
if(receipt.inputs_stable!==true||receipt.process_group_cleanup?.clean!==true||receipt.target_artifacts?.ok!==true)errors.push('receipt input/cleanup/artifact checks are not all true');
const events=[];
for(const [index,line] of stdout.toString('utf8').split(/\r?\n/).entries()){if(!line)continue;try{events.push({line:index+1,raw:line,row:JSON.parse(line)});}catch(e){errors.push('invalid JSONL at line '+(index+1));}}
if(events.length!==receipt.nextest.record_count)errors.push('raw event count differs from runner receipt');
const targets=new Map(plan.targets.map(t=>[t.event_binary,t]));
const tests=new Map();
const suites=new Map();
const prefix=plan.package.name+'::';
for(const rec of events){const e=rec.row;if(e.type==='suite'){const m=e.nextest||{};const target=targets.get(m.test_binary);if(!target||m.crate!==plan.package.name||m.kind!==target.event_kind){errors.push('raw suite identity mismatch at line '+rec.line);continue;}let q=suites.get(m.test_binary);if(!q){q={binary:m.test_binary,kind:m.kind,starts:[],summaries:[]};suites.set(m.test_binary,q);}if(e.event==='started')q.starts.push({line:rec.line,test_count:e.test_count});else if(e.event==='ok'||e.event==='failed'){const f=rec.raw.match(/"filtered_out"\s*:\s*(\d+)(?=\s*[,}])/);q.summaries.push({line:rec.line,event:e.event,passed:e.passed,failed:e.failed,ignored:e.ignored,measured:e.measured,filtered_out_raw:f?f[1]:null});}else errors.push('unexpected raw suite event '+e.event);continue;}
  if(e.type!=='test'||typeof e.name!=='string'||!e.name.startsWith(prefix)){errors.push('unexpected/non-test event at line '+rec.line);continue;}
  const target=plan.targets.find(t=>e.name.startsWith(prefix+t.event_binary+'$'));if(!target){errors.push('raw test event lacks selected binary binding');continue;}const name=e.name.slice((prefix+target.event_binary+'$').length),key=target.event_binary+'$'+name;let q=tests.get(key);if(!q){q={binary:target.event_binary,name,selected:target.test_names.includes(name),starts:[],terminals:[]};tests.set(key,q);}if(e.event==='started')q.starts.push(rec.line);else q.terminals.push({event:e.event,line:rec.line});
}
const selected=[];
const perBinary=[];
for(const t of plan.targets){let started=0,passed=0,failed=0;for(const name of t.test_names){const q=tests.get(t.event_binary+'$'+name);if(!q||q.starts.length!==1||q.terminals.length!==1||q.terminals[0].event!=='ok'||q.starts[0]>=q.terminals[0].line)errors.push('selected raw test is not one start-before-ok: '+t.event_binary+'$'+name);else{started++;passed++;}selected.push({binary:t.event_binary,name,event:q?.terminals[0]?.event||'missing'});}
  const suite=suites.get(t.event_binary),latest=suite?.summaries.at(-1);if(!suite||suite.starts.length<1||suite.starts.some(x=>x.test_count!==t.test_names.length)||!latest||latest.event!=='ok'||latest.passed!==t.test_names.length||latest.failed!==0||latest.measured!==0)errors.push('raw suite summary does not match target '+t.event_binary);
  perBinary.push({binary:t.event_binary,kind:t.event_kind,selected_count:t.test_names.length,selected_started:started,selected_ok:passed,selected_failed:failed,suite_started:suite?.starts||[],suite_terminal_summaries:suite?.summaries||[],suite_ignored:latest?.ignored??null,filtered_out_raw:latest?.filtered_out_raw??null});
}
for(const [key,q] of tests){if(!q.selected&&(q.starts.length!==1||q.terminals.length!==1||q.terminals[0].event!=='ignored'||q.starts[0]>=q.terminals[0].line))errors.push('unselected raw test row is not unique start-before-ignored: '+key);}
const rawHuman=stderr.toString('utf8').replace(/\x1b\[[0-?]*[ -/]*[@-~]/g,'');
const summaryLines=rawHuman.split(/\r?\n/).filter(x=>/\bSummary\s*\[/.test(x));
const hm=/^\s*Summary\s+\[\s*[^\]]+\]\s+(\d+)\s+tests?\s+run:\s*(\d+)\s+passed(?:,\s*(\d+)\s+failed)?(?:,\s*(\d+)\s+skipped)?\s*$/;
const humanSummaries=summaryLines.map(line=>{const m=line.match(hm);if(!m){errors.push('human summary line did not parse');return {raw:line,parsed:false};}return {raw:line,parsed:true,total:Number(m[1]),passed:Number(m[2]),failed:Number(m[3]||0),skipped:Number(m[4]||0)};});
const runIds=[...rawHuman.matchAll(/Nextest run ID\s+([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/gi)].map(m=>m[1].toLowerCase());
if(runIds.length!==1)errors.push('expected one observed Nextest run ID in raw stderr');
const filteredOutSum=perBinary.reduce((n,row)=>n+BigInt(row.filtered_out_raw||'0'),0n).toString();
const humanSkipped=humanSummaries.length===1&&humanSummaries[0].parsed?humanSummaries[0].skipped:null;
if(humanSummaries.length!==1||!humanSummaries[0].parsed||humanSummaries[0].total!==9||humanSummaries[0].passed!==9||humanSummaries[0].failed!==0)errors.push('human summary count fields do not show nine passes');
const runnerSource=read(runnerPath).toString('utf8');
const wrapperRequiresZeroSkipped=runnerSource.includes('row.skipped!==0');
if(!wrapperRequiresZeroSkipped)errors.push('pinned wrapper does not contain the observed zero-skipped rule');
const report={schema:'termrock.vis19.cursor-r4-r5-actual-result-reconstruction/v1',status:errors.length?'RECONSTRUCTION_INPUT_OR_BINDING_ERROR':'RECONSTRUCTED_SELECTED_PASS_WRAPPER_INCOMPLETE',generated_at_utc:new Date().toISOString(),read_only:true,cargo_spawned_by_reconstruction:false,nextest_spawned_by_reconstruction:false,source_run:{receipt:{path:receiptPath,sha256:filePins.receipt.sha256,status:receipt.status},runner:filePins.runner,runplan:filePins.runplan,preflight:filePins.preflight,raw_stdout:filePins.stdout,raw_stderr:filePins.stderr,process_events:filePins.process_events,run_id:runIds[0]||null,root_reported_runner_exit_code:125,root_reported_runner_exit_code_basis:'Root actual-launch observation; attempt receipt itself records wrapper status and child Cargo exit separately',cargo_exit_code:receipt.child.exit_code,cargo_signal:receipt.child.signal,selected:{count:receipt.nextest.selected_count,passed:receipt.nextest.selected_passed,failed:receipt.nextest.selected_failed,other:receipt.nextest.selected_other,raw_event_record_count:events.length,per_binary:perBinary,names:selected},human_summary:{lines:humanSummaries,filtered_out_raw_sum:filteredOutSum,skipped_matches_filtered_out_sum:humanSkipped!==null&&String(humanSkipped)===filteredOutSum,emitted_unselected_ignored_test_rows:[...tests.values()].filter(x=>!x.selected).length,wrapper_zero_skipped_rule_present:wrapperRequiresZeroSkipped,wrapper_rejection:receipt.human_nextest_summary.errors},integrity:{input_snapshots_stable:receipt.inputs_stable,process_group_cleanup_clean:receipt.process_group_cleanup?.clean,target_artifacts_valid:receipt.target_artifacts?.ok,all_expected_file_pins_match:errors.length===0,errors}},limits:['The original runner remains FAIL_OR_INCOMPLETE; this reconstruction does not rewrite its status or exit.','The 9 selected Rust tests passed in this invocation according to raw libtest-json-plus events, suite summaries, and human summary.','The human summary reports 88 skipped while JSON reports filtered_out 68+20 and suite ignored 0+0; the numeric totals coincide, but ignored test rows were not emitted.','This result is only evidence for the exact pinned VIS-19 cursor test packet, not broader product or corpus qualification.']};
const out=path.join(__dirname,'report.json');
const bytes=Buffer.from(JSON.stringify(report,null,2)+'\n');const fd=fs.openSync(out,'wx',0o600);try{fs.writeSync(fd,bytes);fs.fsyncSync(fd);}finally{fs.closeSync(fd);}fs.chmodSync(out,0o444);
console.log(JSON.stringify({status:report.status,path:out,bytes:bytes.length,sha256:sha(bytes),errors:errors.slice(0,20)},null,2));
