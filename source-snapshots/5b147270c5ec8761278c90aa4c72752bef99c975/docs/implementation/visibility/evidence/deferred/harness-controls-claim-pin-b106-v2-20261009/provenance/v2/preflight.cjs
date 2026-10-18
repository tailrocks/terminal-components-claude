"use strict";
const fs=require("node:fs");
const path=require("node:path");
const crypto=require("node:crypto");
const {execFileSync}=require("node:child_process");
const BASE=__dirname;
const sha=b=>crypto.createHash("sha256").update(b).digest("hex");
const manifestBytes=fs.readFileSync(path.join(BASE,"source-manifest.json"));
const planBytes=fs.readFileSync(path.join(BASE,"runplan.json"));
const namesBytes=fs.readFileSync(path.join(BASE,"test-names.json"));
const runnerBytes=fs.readFileSync(path.join(BASE,"run-controls.cjs"));
const launchBytes=fs.readFileSync(path.join(BASE,"launch.sh"));
const manifest=JSON.parse(manifestBytes),plan=JSON.parse(planBytes),namesDoc=JSON.parse(namesBytes),names=namesDoc.names;
const cacheBytes=fs.readFileSync(path.join(BASE,"cargo-cache-manifest.json"));
const cache=JSON.parse(cacheBytes);
function walk(root,rel=""){
  const rows=[];
  for(const name of fs.readdirSync(root).sort()){
    const full=path.join(root,name),next=path.posix.join(rel,name),st=fs.lstatSync(full);
    if(st.isDirectory())rows.push(...walk(full,next));
    else if(st.isFile()&&!st.isSymbolicLink())rows.push(next);
    else throw new Error("unexpected source entry "+next);
  }
  return rows;
}
const actual=walk(manifest.external_source_root),expected=manifest.files.map(f=>f.path).sort();
if(JSON.stringify(actual)!==JSON.stringify(expected))throw new Error("source file inventory mismatch");
for(const file of manifest.files){
  const full=path.join(manifest.external_source_root,file.path),st=fs.lstatSync(full),bytes=fs.readFileSync(full);
  if(!st.isFile()||st.isSymbolicLink()||bytes.length!==file.bytes||sha(bytes)!==file.sha256)throw new Error("source digest mismatch "+file.path);
}
if(manifest.files.length!==14||manifest.harness_controls.count!==29||manifest.candidate_product_commit!=="1d797d41c8141fcbdc3f69d7f11eb8875ab54712"||manifest.fixture_base_commit!=="b106b2426cfce016381c58b3d380e762a454c7a6")throw new Error("source manifest pins mismatch");
const fixture=manifest.files.find(f=>f.path==="crates/termrock-visibility-tests/src/bin/fixture.rs");
if(fixture.sha256!=="a7ee4a89646439a67d2d4e016b759a2d68eecec7339310778e81acf60235adc4")throw new Error("fixture is not committed b106 baseline");
if(namesDoc.schema!=="termrock-vis04-deferred-controls-test-names/v1"||namesDoc.target!=="termrock-visibility-tests::deferred"||namesDoc.count!==29||names.length!==29||names.some((n,i)=>typeof n!=="string"||!/^[a-z0-9_]+$/.test(n)||names.indexOf(n)!==i)||JSON.stringify(names)!==JSON.stringify(manifest.harness_controls.names))throw new Error("test-name selection invalid");
const testPath=path.join(manifest.external_source_root,"crates/termrock-visibility-tests/tests/deferred.rs");
const testSource=fs.readFileSync(testPath,"utf8");
const lines=testSource.split(/\r?\n/),testNames=[];
for(let i=0;i<lines.length;i++)if(lines[i].trim()==="#[test]"){
  let j=i+1;while(j<lines.length&&(lines[j].trim()===""||lines[j].trim().startsWith("#[")))j++;
  const m=(lines[j]||"").trim().match(/^fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/);if(!m)throw new Error("malformed test attribute at line "+(i+1));testNames.push(m[1]);
}
if(JSON.stringify(testNames)!==JSON.stringify(names))throw new Error("source test functions do not match exact 29 selection");
const registry=JSON.parse(fs.readFileSync(path.join(manifest.external_source_root,"crates/termrock-e2e/cases/deferred-obligations.json"),"utf8"));
if(registry.row_count!==23||registry.case_reference_count!==25||registry.missing_historical_id!=="BD-19"||registry.rows.some(row=>row.status!=="NOT_RUN"))throw new Error("candidate registry denominator or status mismatch");
const controlSource=fs.readFileSync(path.join(manifest.external_source_root,"crates/termrock-conformance/tests/control_states.rs"),"utf8");
const productNames=new Map();
for(let i=0;i<lines.length;i++){} // Keep harness parse independent from product source.
const productLines=controlSource.split(/\r?\n/);
for(let i=0;i<productLines.length;i++)if(productLines[i].trim()==="#[test]"){
  let j=i+1,ignored=false;while(j<productLines.length&&(productLines[j].trim()===""||productLines[j].trim().startsWith("#["))){if(productLines[j].trim().startsWith("#[ignore"))ignored=true;j++;}
  const m=(productLines[j]||"").trim().match(/^fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/);if(m)productNames.set(m[1],ignored);
}
const ignoredIds=registry.rows.filter(row=>!productNames.has(row.legacy_test)||productNames.get(row.legacy_test)).map(row=>row.id);
const normalCount=registry.rows.filter(row=>productNames.has(row.legacy_test)&&!productNames.get(row.legacy_test)).length;
if(normalCount!==22||JSON.stringify(ignoredIds)!==JSON.stringify(["BD-21"]))throw new Error("product source metadata is not 22 normal / BD-21 ignored");
if(plan.schema!=="termrock-vis04-deferred-harness-controls-runplan/v4"||plan.actual_receipt_schema!=="termrock-vis04-deferred-harness-controls-run-receipt/v1")throw new Error("runplan or actual receipt schema mismatch");
if(sha(manifestBytes)!==plan.source_manifest_sha256||sha(namesBytes)!==plan.test_names_sha256||sha(runnerBytes)!==plan.runner_sha256||sha(launchBytes)!==plan.launch_script_sha256||sha(cacheBytes)!==plan.cargo_cache_manifest_sha256)throw new Error("runplan source binding mismatch");

if(cache.schema!=="termrock-vis04-locked-cargo-cache-repair/v1"||cache.state!=="COMPLETE_VERIFIED_ARCHIVE_AND_INDEX_CLOSURE"||cache.destination_cargo_home!==plan.cargo_home||cache.locked_registry_packages!==32||cache.packages.length!==32)throw new Error("locked Cargo cache manifest drift");
if(sha(fs.readFileSync(path.join(manifest.external_source_root,"crates/termrock-visibility-tests/Cargo.lock")))!==cache.cargo_lock_sha256)throw new Error("Cargo cache lockfile binding mismatch");
const cacheFiles=[];for(const pkg of cache.packages){if(pkg.archive_sha256!==pkg.checksum)throw new Error("archive checksum manifest mismatch "+pkg.name);for(const [kind,rel,expected] of [["archive",pkg.archive_path,pkg.archive_sha256],["index",pkg.index_path,pkg.index_sha256]]){const full=path.resolve(cache.destination_cargo_home,rel);if(!full.startsWith(cache.destination_cargo_home+path.sep))throw new Error("cache path escapes CARGO_HOME "+rel);const st=fs.lstatSync(full),bytes=fs.readFileSync(full);if(!st.isFile()||st.isSymbolicLink()||sha(bytes)!==expected)throw new Error("Cargo cache "+kind+" digest mismatch "+rel);cacheFiles.push({kind,path:rel,sha256:expected});}}
if(cacheFiles.length!==64)throw new Error("expected 64 archive/index cache files, observed "+cacheFiles.length);
const cacheConfig=cache.index_config,cacheConfigPath=path.resolve(cache.destination_cargo_home,cacheConfig.path),cacheConfigStat=fs.lstatSync(cacheConfigPath),cacheConfigBytes=fs.readFileSync(cacheConfigPath);if(!cacheConfigPath.startsWith(cache.destination_cargo_home+path.sep)||!cacheConfigStat.isFile()||cacheConfigStat.isSymbolicLink()||sha(cacheConfigBytes)!==cacheConfig.sha256)throw new Error("sparse index config digest mismatch");cacheFiles.sort((a,b)=>a.path.localeCompare(b.path));const cacheClosureSha256=sha(Buffer.from(JSON.stringify(cacheFiles)));

const nextest=plan.nextest_bin;
const env={PATH:plan.fixed_path,LANG:"C",LC_ALL:"C",TERM:"dumb",CARGO_HOME:plan.cargo_home,RUSTUP_HOME:plan.rustup_home};
const version=execFileSync(nextest,["--version"],{encoding:"utf8",timeout:5000,maxBuffer:4096,env}).trim().split("\n",1)[0];
if(version!==plan.nextest_version)throw new Error("pinned Nextest version mismatch");
const runHelp=execFileSync(nextest,["nextest","run","--help"],{encoding:"utf8",timeout:10000,maxBuffer:2*1024*1024,env});
for(const option of ["--manifest-path","--locked","--offline","--package","--test","--build-jobs","--test-threads","--run-ignored","--filterset","--ignore-default-filter","--user-config-file","--no-tests","--no-fail-fast","--retries","--flaky-result","--message-format","--message-format-version"])if(!runHelp.includes(option))throw new Error("pinned Nextest help omits "+option);
const rustc="/Users/donbeave/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/rustc";
const rustVersion=execFileSync(rustc,["--version"],{encoding:"utf8",timeout:5000,maxBuffer:4096,env:{PATH:plan.fixed_path,LANG:"C",LC_ALL:"C",TERM:"dumb"}}).trim();
if(!rustVersion.startsWith("rustc 1.98.1 "))throw new Error("pinned rustc version mismatch");
if(!fs.statSync(plan.cargo_home).isDirectory()||!fs.statSync(plan.rustup_home).isDirectory()||!fs.statSync(plan.cargo_bin).isFile())throw new Error("pinned offline cache/toolchain missing");
if(fs.existsSync(path.join(BASE,"runs"))||fs.existsSync(plan.target_dir)||fs.existsSync(plan.scratch_dir))throw new Error("runner appears to have been launched already");
const result={
 schema:"termrock-vis04-deferred-harness-controls-static-preflight/v1",result:"PASS_STATIC_PREFLIGHT_NO_CARGO",
 checked_at_utc:new Date().toISOString(),source_manifest_sha256:sha(manifestBytes),runplan_sha256:sha(planBytes),runner_sha256:sha(runnerBytes),launch_script_sha256:sha(launchBytes),test_names_sha256:sha(namesBytes),
 copied_source_count:manifest.files.length,fixture_source_sha256:fixture.sha256,harness_controls_selected:names.length,harness_controls_ignored:0,actual_run_receipt_schema:plan.actual_receipt_schema,cargo_cache:{manifest_sha256:sha(cacheBytes),lock_sha256:cache.cargo_lock_sha256,locked_packages:32,archive_files_verified:32,sparse_index_files_verified:32,config_sha256:cacheConfig.sha256,closure_sha256:cacheClosureSha256},
 candidate_product_commit:manifest.candidate_product_commit,fixture_base_commit:manifest.fixture_base_commit,candidate_product_metadata:{rows:23,case_references:25,normal:normalCount,ignored_ids:ignoredIds,all_rows_status:"NOT_RUN"},
 rust:rustVersion,cargo_nextest:version,nextest_run_help_options_verified:true,
 launch:{offline:true,inherits_parent_environment:false,timeout_ms:plan.timeout_ms,term_grace_ms:plan.term_grace_ms,post_kill_grace_ms:plan.post_kill_grace_ms,combined_log_cap_bytes:plan.combined_log_cap_bytes,process_group:true},
 cargo_or_test_execution:"NOT_RUN",repository_mutations:false,source_mutations:false
};
fs.writeFileSync(path.join(BASE,"preflight.json"),JSON.stringify(result,null,2)+"\n",{flag:"wx"});
process.stdout.write(JSON.stringify(result,null,2)+"\n");
