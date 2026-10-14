"use strict";
const fs=require("node:fs");
const path=require("node:path");
const crypto=require("node:crypto");
const BASE=__dirname;
const sha=b=>crypto.createHash("sha256").update(b).digest("hex");
if(process.argv.length!==3)throw new Error("usage: record-ledger.cjs /absolute/path/to/receipt.json");
const receiptPath=path.resolve(process.argv[2]);
const receipt=JSON.parse(fs.readFileSync(receiptPath,"utf8"));
if(receipt.schema!=="termrock-vis04-product23-run-receipt/v1")throw new Error("unexpected run receipt schema");
if(receipt.execution.result==="PASS_RUNTIME_PREFLIGHT_NO_CARGO"||receipt.listing.result!=="LIST_SELECTION_MATCHED")throw new Error("there is no completed product run to record");
if(!Array.isArray(receipt.execution.case_results)||receipt.execution.case_results.length!==23)throw new Error("receipt does not contain all 23 stable case rows");
const ids=Array.from({length:18},(_,i)=>"BD-"+String(i+1).padStart(2,"0")).concat(["BD-20","BD-21","BD-22","BD-23","BD-24"]);
if(JSON.stringify(receipt.execution.case_results.map(r=>r.id))!==JSON.stringify(ids))throw new Error("stable ID order drift");
const plan=JSON.parse(fs.readFileSync(path.join(BASE,"runplan.json"),"utf8"));
if(receipt.source.source_manifest_sha256!==plan.source_manifest_sha256||receipt.source.runplan_sha256!==require("node:crypto").createHash("sha256").update(fs.readFileSync(path.join(BASE,"runplan.json"))).digest("hex"))throw new Error("receipt is not bound to this plan");
for(let i=0;i<23;i++){const expected=plan.stable_case_map[i],actual=receipt.execution.case_results[i];if(expected.id!==actual.id||expected.test_name!==actual.test_name||JSON.stringify(expected.cases)!==JSON.stringify(actual.cases))throw new Error("stable mapping drift for "+expected.id);}
const runRoot=path.resolve(BASE,"runs");const resolvedDir=path.dirname(receiptPath);if(!resolvedDir.startsWith(runRoot+path.sep)||path.basename(receiptPath)!=="receipt.json")throw new Error("receipt is outside this packet run directory");
if(receipt.execution.run_started!==true)throw new Error("no product test run was started");
if(receipt.execution.expected_failure_treatment!=="none")throw new Error("expected-failure inversion is forbidden");
const rows=receipt.execution.case_results.map(row=>({requirement_id:row.id,case_ids:row.cases,test_name:row.test_name,component_id:row.component,component_owner:row.owner,registered_ignored:row.registered_ignored,ignore_reason:row.ignore_reason,status:row.status}));
const counts={expected:23,passed:0,failed:0,skipped:0,not_run:0,incomplete:0};
for(const row of rows){if(!(row.status in counts))throw new Error("unknown outcome status for "+row.requirement_id+": "+row.status);counts[row.status]++;}
const receiptBytes=fs.readFileSync(receiptPath);
const output={schema:"termrock-vis04-product23-ledger-record/v1",recorded_at_utc:new Date().toISOString(),record_status:"external_record_prepared",repository_write_performed:false,source:{candidate_product_commit:receipt.source.candidate_product_commit,source_manifest_sha256:receipt.source.source_manifest_sha256,runplan_sha256:receipt.source.runplan_sha256},run:{run_id:receipt.run_id,nextest_run_id:receipt.execution.nextest_run_id,result:receipt.execution.result,receipt_path:receiptPath,receipt_sha256:sha(receiptBytes)},selection:{package:receipt.selection.package,binary:receipt.selection.binary,count:23,run_ignored:"all",expected_failure_treatment:"none",registered_normal_count:22,registered_ignored_ids:["BD-21"]},counts,rows,scope:{evidence_class:"candidate-direct-assertions",qualifies_paired_pty_evidence:false,product_registry_source_status_remains:"NOT_RUN",no_product_repair_performed:true}};
const runDir=path.dirname(receiptPath);
const outPath=path.join(runDir,"ledger-record.json");
fs.writeFileSync(outPath,JSON.stringify(output,null,2)+"\n",{flag:"wx"});
console.log(JSON.stringify({ledger_record:outPath,ledger_record_sha256:sha(fs.readFileSync(outPath)),receipt_sha256:output.run.receipt_sha256,counts},null,2));
