#!/Users/donbeave/.local/share/mise/installs/node/24.20.0/bin/node
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const {spawn, spawnSync, execFileSync} = require('node:child_process');

const PACKET = __dirname;
const PLAN_PATH = path.join(PACKET, 'runplan.json');
const PREFLIGHT_PATH = path.join(PACKET, 'preflight.json');
const sha = b => crypto.createHash('sha256').update(b).digest('hex');
const read = p => fs.readFileSync(p);
const readJson = p => JSON.parse(read(p).toString('utf8'));
const stable = x => JSON.stringify(x);
const fail = m => { throw new Error(m); };

function regularPin(p, expectedSha, expectedBytes, label) {
  const st = fs.lstatSync(p);
  if (!st.isFile() || st.isSymbolicLink()) fail(label + ' is not a regular file: ' + p);
  const bytes = read(p);
  const digest = sha(bytes);
  if (digest !== expectedSha || (expectedBytes !== undefined && bytes.length !== expectedBytes)) fail(label + ' pin mismatch: ' + p);
  return {path:p, sha256:digest, bytes:bytes.length, mode:st.mode & 0o7777, realpath:fs.realpathSync(p)};
}
function noSymlinkPath(p) {
  const absolute = path.resolve(p);
  const parsed = path.parse(absolute);
  let current = parsed.root;
  for (const part of absolute.slice(parsed.root.length).split(path.sep).filter(Boolean)) {
    current = path.join(current, part);
    const st = fs.lstatSync(current);
    if (st.isSymbolicLink()) fail('symlink path component: ' + current);
  }
  return fs.realpathSync(absolute);
}
function absent(p) {
  try { fs.lstatSync(p); return false; }
  catch (e) { if (e && e.code === 'ENOENT') return true; throw e; }
}
function writeJson(p, value) {
  const bytes = Buffer.from(JSON.stringify(value, null, 2) + '\n');
  const fd = fs.openSync(p, 'wx', 0o600);
  try { fs.writeSync(fd, bytes); fs.fsyncSync(fd); }
  finally { fs.closeSync(fd); }
  fs.chmodSync(p, 0o600);
}
function ensurePrivateDir(p, create) {
  if (absent(p)) {
    if (!create) fail('required private directory absent: ' + p);
    fs.mkdirSync(p, {mode:0o700}); fs.chmodSync(p, 0o700);
  }
  const st = fs.lstatSync(p);
  if (!st.isDirectory() || st.isSymbolicLink() || (st.mode & 0o077) !== 0) fail('directory is not a private real directory: ' + p);
  return {path:p, realpath:fs.realpathSync(p), mode:st.mode & 0o7777, dev:String(st.dev), ino:String(st.ino)};
}
function git(args, cwd) {
  return execFileSync('/usr/bin/git', args, {cwd, env:{PATH:'/usr/bin:/bin', LANG:'C', LC_ALL:'C'}, encoding:null, timeout:30000, maxBuffer:8*1024*1024});
}
function verifySelectionMetadataCorrection(plan) {
  const s=plan.source;
  const originalPin=regularPin(s.reviewed_manifest_path,s.reviewed_manifest_sha256,undefined,'frozen original R4 source manifest');
  const correctedPin=regularPin(s.manifest_path,s.manifest_sha256,undefined,'versioned corrected R4 manifest copy');
  const correctionPin=regularPin(s.selection_correction_path,s.selection_correction_sha256,undefined,'R4 selector metadata correction addendum');
  const correctionReviewPin=regularPin(s.selection_correction_review_path,s.selection_correction_review_sha256,undefined,'independent R4 selector metadata correction review');
  const originalBytes=read(s.reviewed_manifest_path),correctedBytes=read(s.manifest_path);
  const oldName='tui::cursor_projection_tests::style_code_matrix_preserves_default_and_six_decscsr_shapes';
  const newName='tui::cursor_projection_tests::style_code_matrix_preserves_default_and_six_decscusr_shapes';
  const needle='"'+oldName+'"';
  const originalText=originalBytes.toString('utf8');
  if(originalText.split(needle).length!==2)fail('original R4 manifest does not contain exactly one pinned selector typo');
  const expected=Buffer.from(originalText.replace(needle,'"'+newName+'"'),'utf8');
  if(!expected.equals(correctedBytes))fail('corrected R4 manifest differs beyond the single pinned selector spelling');
  const original=JSON.parse(originalBytes.toString('utf8')),corrected=JSON.parse(correctedBytes.toString('utf8'));
  const selection=readJson(plan.selection.path),addendum=readJson(s.selection_correction_path);
  const correctionReview=readJson(s.selection_correction_review_path);
  const sourcePath=path.join(s.root,'crates/tuiscotti-runtime/src/tui/mod.rs');
  const sourceBytes=read(sourcePath),sourceText=sourceBytes.toString('utf8');
  const sourceFn='fn '+newName.split('::').at(-1)+'(';
  if(original.selected_tests?.unit?.[2]!==oldName||corrected.selected_tests?.unit?.[2]!==newName||selection.unit_tests?.[2]!==newName||!sourceText.includes(sourceFn))fail('selector correction does not agree with exact selection and source function');
  if(addendum.schema!=='termrock.vis19.r4-selection-metadata-correction/v1'||addendum.status!=='PREPARED_FOR_INDEPENDENT_METADATA_REVIEW_ONLY_NOT_AUTHORIZED'||addendum.original_manifest?.path!==s.reviewed_manifest_path||addendum.original_manifest?.sha256!==originalPin.sha256||addendum.corrected_manifest?.path!==s.manifest_path||addendum.corrected_manifest?.sha256!==correctedPin.sha256||addendum.exact_delta?.json_pointer!=='/selected_tests/unit/2'||addendum.exact_delta?.before!==oldName||addendum.exact_delta?.after!==newName||addendum.exact_delta?.replacement_count!==1||addendum.selection_authority?.path!==plan.selection.path||addendum.selection_authority?.sha256!==sha(read(plan.selection.path))||addendum.selection_authority?.selected_test_count!==9||addendum.selection_authority?.exact_unit_name!==newName||addendum.source_confirmation?.path!==sourcePath||addendum.source_confirmation?.sha256!==sha(sourceBytes)||addendum.source_confirmation?.function_declaration!==sourceFn||addendum.source_review?.path!==s.source_review_path||addendum.source_review?.sha256!==s.source_review_sha256||addendum.source_patch?.path!==s.patch_path||addendum.source_patch?.sha256!==s.patch_sha256||addendum.preserved_failures?.r3_result_path!==s.prior_result_path||addendum.preserved_failures?.r3_result_sha256!==s.prior_result_sha256||addendum.preserved_failures?.r3_actual_review_sha256!==s.prior_actual_review_sha256)fail('selector correction addendum does not bind the exact metadata/source inputs');
  if(correctionReview.verdict!=='READY_METADATA_CORRECTION_ONLY'||correctionReview.artifacts?.original_manifest?.path!==s.reviewed_manifest_path||correctionReview.artifacts?.original_manifest?.sha256!==originalPin.sha256||correctionReview.artifacts?.corrected_manifest?.path!==s.manifest_path||correctionReview.artifacts?.corrected_manifest?.sha256!==correctedPin.sha256||correctionReview.artifacts?.correction_addendum?.path!==s.selection_correction_path||correctionReview.artifacts?.correction_addendum?.sha256!==correctionPin.sha256||correctionReview.verified?.delta?.json_pointer!=='/selected_tests/unit/2'||correctionReview.verified?.delta?.before!==oldName||correctionReview.verified?.delta?.after!==newName||correctionReview.verified?.selection_authority?.sha256!==sha(read(plan.selection.path))||correctionReview.verified?.source_confirmation?.sha256!==sha(sourceBytes)||correctionReview.verified?.source_review?.sha256!==s.source_review_sha256||correctionReview.verified?.preserved_r3_failure?.result_sha256!==s.prior_result_sha256||correctionReview.verified?.preserved_r3_failure?.actual_review_sha256!==s.prior_actual_review_sha256)fail('independent metadata correction review does not bind the exact delta and inputs');
  if(originalPin.sha256!==s.reviewed_manifest_sha256||correctedPin.bytes!==originalPin.bytes+1)fail('selector correction original/corrected manifest byte relationship differs');
  return {original_manifest:originalPin,corrected_manifest:correctedPin,correction_addendum:correctionPin,correction_review:correctionReviewPin,exact_delta:{json_pointer:'/selected_tests/unit/2',before:oldName,after:newName},source_function:{path:sourcePath,sha256:sha(sourceBytes)}};
}
function composeR4Manifest(plan) {
  const s=plan.source;
  const selectionCorrection=verifySelectionMetadataCorrection(plan);
  const current=readJson(s.manifest_path),prior=readJson(s.prior_manifest_path);
  if(current.schema!=='termrock.vis19.cursor-r4-test-only-overlay/v1'||prior.schema!=='termrock-vis19-cursor-increment-review/v1')fail('R4/R3 source manifest schema mismatch');
  if(current.project.root!==s.root||current.project.head!==s.base_commit||current.project.tree!==s.base_tree)fail('R4 project root/base identity mismatch');
  if(current.prior.r3_source_manifest_sha256!==s.prior_manifest_sha256||current.prior.r3_source_patch_sha256!==s.prior_patch_sha256||current.prior.r3_static_source_review_sha256!==s.prior_source_review_sha256)fail('R4 prior source pins differ from R3 execution packet');
  if(current.prior.r3_actual_result_sha256!==s.prior_result_sha256)fail('R4 prior actual result pin mismatch');
  const fullPatch=current.artifacts.find(x=>x.path==='r4-full-source.patch');
  const replay=current.artifacts.find(x=>x.path==='replay.rs');
  if(!fullPatch||fullPatch.sha256!==s.patch_sha256||!Number.isInteger(fullPatch.bytes)||!replay||replay.sha256!==current.delta.proposed_file_sha256)fail('R4 source artifacts do not bind patch and replay source');
  const replayPath='crates/tuiscotti-runtime/tests/tui_shell/replay.rs';
  if(stable(current.delta.changed_paths)!==stable([replayPath])||current.delta.proposed_file_sha256!==replay.sha256)fail('R4 source delta is not the reviewed one-file replay change');
  const changedFiles=prior.changed_files.map(row=>row.path===replayPath?{...row,proposed_bytes:replay.bytes,proposed_sha256:replay.sha256}:row);
  const changedPaths=changedFiles.map(x=>x.path).sort();
  const currentPaths=[...current.scope.changed_source_files].sort();
  if(stable(currentPaths)!==stable([replayPath])||changedFiles.length!==4)fail('R4 incremental source path set or composed four-file source inventory differs');
  if(!current.selected_tests||current.selected_tests.count!==9||stable(current.selected_tests.unit)!==stable(plan.targets.find(t=>t.cargo_flag==='--lib')?.test_names)||stable(current.selected_tests.integration)!==stable(plan.targets.find(t=>t.cargo_flag==='--test')?.test_names))fail('corrected R4 manifest does not retain the exact nine selected R3 test names');
  return {...prior,r4_manifest:{path:s.manifest_path,sha256:selectionCorrection.corrected_manifest.sha256,original_path:s.reviewed_manifest_path,original_sha256:selectionCorrection.original_manifest.sha256,prior:current.prior,selection_correction:selectionCorrection.exact_delta},worktree:{...prior.worktree,path:s.root,head:current.project.head,tree:current.project.tree,tracked_changes_relative_to_base:currentPaths},patch:{path:s.patch_path,bytes:fullPatch.bytes,sha256:fullPatch.sha256},changed_files:changedFiles};
}
function verifySource(plan, manifest) {
  const src = plan.source;
  const root = src.root;
  if (manifest.worktree.path !== root || manifest.base.commit !== src.base_commit || manifest.base.tree !== src.base_tree) fail('R4 source manifest/root/base mismatch');
  if (manifest.patch.path !== src.patch_path || manifest.patch.sha256 !== src.patch_sha256) fail('R4 source patch pointer differs from manifest');
  regularPin(src.manifest_path, src.manifest_sha256, undefined, 'R4 source manifest');
  regularPin(src.patch_path, src.patch_sha256, manifest.patch.bytes, 'R4 full source patch');
  regularPin(src.identity_manifest_path, src.identity_manifest_sha256, undefined, 'base source identity manifest');
  if (path.join(root, 'Cargo.lock') !== plan.package.lock_path) fail('package lock is outside source root');
  const head = git(['rev-parse','HEAD'],root).toString('utf8').trim();
  const tree = git(['rev-parse','HEAD^{tree}'],root).toString('utf8').trim();
  if (head !== src.base_commit || tree !== src.base_tree) fail('source HEAD/tree changed');
  const status = git(['status','--porcelain=v1','--untracked-files=all'],root).toString('utf8').trimEnd();
  const rows = status ? status.split(/\r?\n/) : [];
  const paths = rows.map(line => line.slice(3)).sort();
  const wanted = manifest.changed_files.map(x => x.path).sort();
  if (stable(paths) !== stable(wanted)) fail('source worktree has unexpected changed/untracked paths');
  const diff = git(['diff','--binary','--full-index','HEAD','--'],root);
  if (sha(diff) !== src.patch_sha256 || !diff.equals(read(src.patch_path))) fail('worktree diff does not equal the pinned R4 full source patch');
  const changed = [];
  for (const row of manifest.changed_files) {
    const p = path.join(root,row.path);
    const st = fs.lstatSync(p);
    if (!st.isFile() || st.isSymbolicLink() || st.size !== row.proposed_bytes) fail('changed source type/length mismatch: '+row.path);
    const digest=sha(read(p));
    if (digest!==row.proposed_sha256) fail('changed source hash mismatch: '+row.path);
    changed.push({path:row.path,bytes:st.size,mode:st.mode&0o7777,sha256:digest});
  }
  const lock = regularPin(plan.package.lock_path, plan.package.lock_sha256, undefined, 'Cargo.lock');
  const pkg = regularPin(plan.package.manifest_path, plan.package.manifest_sha256, undefined, 'runtime Cargo.toml');
  const nx = regularPin(plan.nextest_config.path, plan.nextest_config.sha256, undefined, 'Nextest config');
  const snapshot = {root,head,tree,status,patch_sha256:sha(diff),changed_files:changed,lock,pkg,nextest_config:nx};
  snapshot.digest=sha(Buffer.from(stable(snapshot)));
  return snapshot;
}
function verifyMetadata(plan) {
  const md = plan.metadata;
  const resultPin = regularPin(md.result_path,md.result_sha256,undefined,'Root metadata result');
  const outPin = regularPin(md.stdout_path,md.stdout_sha256,undefined,'Root metadata stdout');
  const errPin = regularPin(md.stderr_path,md.stderr_sha256,undefined,'Root metadata stderr');
  const metadataManifestPin=regularPin(md.package_manifest_path,plan.package.manifest_sha256,undefined,'historical R2 package manifest used for metadata graph');
  const metadataLockPin=regularPin(md.package_lock_path,plan.package.lock_sha256,undefined,'historical R2 lock used for metadata graph');
  if(md.package_manifest_path===plan.package.manifest_path) fail('metadata graph provenance must retain its original R2 manifest path');
  const result = readJson(md.result_path);
  const raw = read(md.stdout_path);
  const metadata = JSON.parse(raw.toString('utf8'));
  if (result.status!==0 || result.packages!==146 || result.lock_before!==plan.package.lock_sha256 || result.lock_after!==plan.package.lock_sha256 || result.stdout_sha256!==md.stdout_sha256 || result.stderr_sha256!==md.stderr_sha256) fail('Root metadata result fields differ');
  if (metadata.packages.length!==146 || metadata.resolve.nodes.length!==146 || metadata.workspace_default_members.length!==1 || metadata.resolve.root!==metadata.workspace_default_members[0]) fail('metadata graph counts/root differ');
  const pkg=metadata.packages.find(x=>x.name===plan.package.name);
  if (!pkg || pkg.manifest_path!==md.package_manifest_path || !pkg.features.pty) fail('metadata runtime package/features differ');
  const targets=new Map(pkg.targets.map(t=>[t.name,t]));
  for (const target of plan.targets) {
    const found=targets.get(target.cargo_target);
    if (!found || !found.kind.includes(target.cargo_kind) || found.test!==true) fail('selected Cargo test target missing from metadata: '+target.cargo_target);
  }
  const actualActive=metadata.resolve.nodes.length;
  const receipt={result:resultPin,stdout:outPin,stderr:errPin,graph_source:{package_manifest:metadataManifestPin,lock:metadataLockPin,classification:'R2 same-content package/lock graph reused as dependency-closure evidence; actual tests compile from independently pinned R3 source worktree'},package_count:result.packages,active_node_count:actualActive,root:metadata.resolve.root,workspace_default_members:metadata.workspace_default_members,selected_targets:plan.targets.map(t=>({name:t.cargo_target,kind:t.cargo_kind}))};
  receipt.digest=sha(Buffer.from(stable(receipt)));
  return receipt;
}
function verifyCache(plan, closure, phase) {
  if (closure.schema!=='termrock.vis19.offline-cache-closure-r2/v1' || closure.cache.cargo_home!==plan.cargo_home || closure.source.lock_sha256!==plan.package.lock_sha256 || closure.cache.archive_count!==165 || closure.cache.index_file_count!==152) fail('cache closure identity/count mismatch');
  const home=ensurePrivateDir(plan.cargo_home,false);
  if (home.realpath!==plan.cargo_home || (home.mode&0o777)!==0o700) fail('CARGO_HOME path/mode mismatch');
  const rows=[];
  for (const a of closure.cache.archives) {
    const relative=path.relative(plan.cargo_home,a.path);
    if (!relative || relative.startsWith('..') || path.isAbsolute(relative)) fail('archive path escapes CARGO_HOME');
    noSymlinkPath(a.path);
    rows.push(regularPin(a.path,a.sha256,a.bytes,'crate archive '+a.name+'@'+a.version));
  }
  for (const i of closure.cache.indexes) {
    const relative=path.relative(plan.cargo_home,i.path);
    if (!relative || relative.startsWith('..') || path.isAbsolute(relative)) fail('index path escapes CARGO_HOME');
    noSymlinkPath(i.path);
    rows.push(regularPin(i.path,i.sha256,i.bytes,'sparse index '+i.name));
  }
  rows.push(regularPin(closure.cache.registry_config.path,closure.cache.registry_config.sha256,closure.cache.registry_config.bytes,'registry config'));
  rows.push(regularPin(closure.cache.cachedir_tag.path,closure.cache.cachedir_tag.sha256,closure.cache.cachedir_tag.bytes,'registry CACHEDIR.TAG'));
  const metadata=[];
  for (const m of closure.cache.mutable_cargo_metadata) {
    const st=fs.lstatSync(m.path);
    if (!st.isFile()||st.isSymbolicLink()) fail('Cargo metadata path type mismatch: '+m.path);
    const digest=sha(read(m.path));
    if (phase==='before' && (digest!==m.sha256||st.size!==m.bytes)) fail('point-in-time Cargo metadata changed before launch: '+m.path);
    metadata.push({path:m.path,bytes:st.size,sha256:digest,mode:st.mode&0o7777,dev:String(st.dev),ino:String(st.ino),nlink:st.nlink,preflight_sha256:m.sha256,classification:m.classification});
  }
  const lock=closure.cache.mutate_lock;
  if(!lock||lock.path!==path.join(plan.cargo_home,'.package-cache-mutate'))fail('Cargo mutate-lock pin missing or path mismatch');
  noSymlinkPath(lock.path);
  const lockStat=fs.lstatSync(lock.path);
  if(!lockStat.isFile()||lockStat.isSymbolicLink())fail('Cargo mutate lock is not a regular non-symlink file');
  const lockDigest=sha(read(lock.path));
  const mutateLock={path:lock.path,bytes:lockStat.size,sha256:lockDigest,mode:lockStat.mode&0o7777,dev:String(lockStat.dev),ino:String(lockStat.ino),nlink:lockStat.nlink,classification:lock.classification};
  if(mutateLock.bytes!==lock.bytes||mutateLock.sha256!==lock.sha256||mutateLock.mode!==lock.mode||mutateLock.dev!==lock.dev||mutateLock.ino!==lock.ino||mutateLock.nlink!==lock.nlink)fail('Cargo mutate lock identity/content differs from exact point-in-time pin');
  const sourceTree=registrySourceInventory(plan.cargo_home,closure);
  const homeInventory=cacheHomeInventory(plan.cargo_home,closure,phase);
  const result={cargo_home:home,seed_rows:rows.sort((a,b)=>a.path.localeCompare(b.path)),seed_count:rows.length,metadata,mutate_lock:mutateLock,registry_source_tree:sourceTree,home_inventory:homeInventory};
  result.digest=sha(Buffer.from(stable(result)));
  return result;
}
function registrySourceInventory(home,closure) {
  const registryId=closure.cache.registry_id;
  const sourceRoot=path.join(home,'registry/src',registryId);
  const locked=new Set(closure.cache.archives.map(a=>a.name+'-'+a.version));
  const packageNames=fs.readdirSync(sourceRoot).sort();
  const unexpected=packageNames.filter(name=>!locked.has(name));
  if(unexpected.length)fail('unlocked extracted Cargo source roots: '+unexpected.join(','));
  const packages=[];
  for(const name of packageNames){
    const root=path.join(sourceRoot,name),rootStat=fs.lstatSync(root);
    if(!rootStat.isDirectory()||rootStat.isSymbolicLink())fail('Cargo source root is not a real directory: '+name);
    const entries=[];
    function walk(dir,relative){
      for(const ent of fs.readdirSync(dir,{withFileTypes:true})){
        const full=path.join(dir,ent.name),rel=relative?relative+'/'+ent.name:ent.name,st=fs.lstatSync(full);
        if(st.isSymbolicLink())fail('symlink in extracted Cargo source: '+name+'/'+rel);
        if(st.isDirectory())entries.push({path:rel,kind:'dir',mode:st.mode&0o7777});
        else if(st.isFile())entries.push({path:rel,kind:'file',mode:st.mode&0o7777,bytes:st.size,sha256:sha(read(full))});
        else fail('special node in extracted Cargo source: '+name+'/'+rel);
        if(st.isDirectory())walk(full,rel);
      }
    }
    walk(root,'');
    entries.sort((a,b)=>Buffer.compare(Buffer.from(a.path),Buffer.from(b.path)));
    const files=entries.filter(x=>x.kind==='file');
    const dirs=entries.filter(x=>x.kind==='dir');
    const bytes=files.reduce((n,x)=>n+x.bytes,0);
    packages.push({name,root_mode:rootStat.mode&0o7777,files:files.length,directories:dirs.length,bytes,tree_sha256:sha(Buffer.from(stable(entries)))});
  }
  const result={path:sourceRoot,locked_archive_roots:locked.size,extracted_package_count:packages.length,file_count:packages.reduce((n,x)=>n+x.files,0),directory_count:packages.reduce((n,x)=>n+x.directories,0),bytes:packages.reduce((n,x)=>n+x.bytes,0),packages};
  result.digest=sha(Buffer.from(stable(result)));
  return result;
}
function cacheHomeInventory(home, closure, phase) {
  const expectedFiles=new Set(['.global-cache','.package-cache','registry/CACHEDIR.TAG']);
  const expectedDirs=new Set(['registry','registry/cache','registry/index','registry/src']);
  for (const a of closure.cache.archives) {
    const rel=path.relative(home,a.path); expectedFiles.add(rel);
    let parent=path.posix.dirname(rel); while(parent!=='.'){expectedDirs.add(parent);parent=path.posix.dirname(parent);}
  }
  for (const i of closure.cache.indexes) {
    const rel=path.relative(home,i.path); expectedFiles.add(rel);
    let parent=path.posix.dirname(rel); while(parent!=='.'){expectedDirs.add(parent);parent=path.posix.dirname(parent);}
  }
  for (const row of [closure.cache.registry_config]) {
    const rel=path.relative(home,row.path); expectedFiles.add(rel);
    let parent=path.posix.dirname(rel); while(parent!=='.'){expectedDirs.add(parent);parent=path.posix.dirname(parent);}
  }
  const mutateRel=path.relative(home,closure.cache.mutate_lock.path);
  if(!mutateRel||mutateRel.startsWith('..')||path.isAbsolute(mutateRel))fail('Cargo mutate-lock path escapes CARGO_HOME');
  expectedFiles.add(mutateRel);
  const id=closure.cache.registry_id;
  expectedDirs.add('registry/cache/'+id);
  expectedDirs.add('registry/index/'+id);
  expectedDirs.add('registry/index/'+id+'/.cache');
  expectedDirs.add('registry/src/'+id);
  const files=[],dirs=[],sourceDirs=[];
  function walk(dir,rel) {
    for (const ent of fs.readdirSync(dir,{withFileTypes:true})) {
      const childRel=rel?rel+'/'+ent.name:ent.name;
      const full=path.join(dir,ent.name),st=fs.lstatSync(full);
      if (st.isSymbolicLink()) fail('symlink in CARGO_HOME: '+childRel);
      if (st.isDirectory()) {
        dirs.push(childRel);
        if (childRel==='registry/src/'+id) {
          for (const sourceEnt of fs.readdirSync(full,{withFileTypes:true})) {
            const sourcePath=path.join(full,sourceEnt.name), sourceStat=fs.lstatSync(sourcePath);
            if (sourceStat.isSymbolicLink() || !sourceStat.isDirectory()) fail('unexpected non-directory Cargo source cache entry: '+sourceEnt.name);
            sourceDirs.push(sourceEnt.name);
          }
          continue;
        }
        walk(full,childRel);
      } else if (st.isFile()) files.push(childRel);
      else fail('special file in CARGO_HOME: '+childRel);
    }
  }
  walk(home,'');
  files.sort();dirs.sort();sourceDirs.sort();
  if (phase==='before') {
    if (stable(files)!==stable([...expectedFiles].sort()) || stable(dirs)!==stable([...expectedDirs].sort())) fail('CARGO_HOME has an unmanifested prelaunch path');
  } else {
    const allowedNonSourceFiles=new Set(expectedFiles);
    for (const f of files) if (!allowedNonSourceFiles.has(f) && !f.startsWith('registry/src/'+id+'/')) fail('unmanifested postlaunch CARGO_HOME file: '+f);
    for (const d of dirs) if (!expectedDirs.has(d) && !d.startsWith('registry/src/'+id+'/')) fail('unmanifested postlaunch CARGO_HOME directory: '+d);
    const allowedSourceNames=new Set(closure.cache.archives.map(a=>a.name+'-'+a.version));
    for (const d of sourceDirs) if (!allowedSourceNames.has(d)) fail('unlocked extracted source directory: '+d);
  }
  return {phase,file_count:files.length,directory_count:dirs.length,files,dirs,extracted_source_directory_count:sourceDirs.length,extracted_source_directories:sourceDirs.sort()};
}
function verifyTools(plan) {
  const entries=[];
  for (const [name,t] of Object.entries(plan.tools)) {
    if (!t || typeof t.path!=='string' || typeof t.sha256!=='string') fail('missing tool pin '+name);
    noSymlinkPath(t.path);
    entries.push({name,...regularPin(t.path,t.sha256,undefined,'tool '+name)});
  }
  if (process.execPath!==plan.tools.node.path || process.version!==plan.tools.node.version) fail('Node runtime identity differs');
  const result={tools:entries.sort((a,b)=>a.name.localeCompare(b.name))};
  result.digest=sha(Buffer.from(stable(result)));
  return result;
}
function selectionInventory(selection) {
  if (selection.schema!=='termrock-vis19-cursor-selection/v1' || selection.revision!=='R3' || selection.run_status!=='NOT_RUN' || selection.selected_test_count!==9) fail('R3 selection identity/count mismatch');
  if (!Array.isArray(selection.unit_tests)||!Array.isArray(selection.integration_tests)||selection.unit_tests.length!==4||selection.integration_tests.length!==5) fail('R3 selection target counts mismatch');
  const unit=selection.unit_tests;
  if([...unit,...selection.integration_tests].some(name=>typeof name!=='string'||!name.length)) fail('R3 selection contains a malformed test name');
  return {unit,integration:selection.integration_tests};
}
function verifySelection(plan, manifest, selection) {
  const pin=regularPin(plan.selection.path,plan.selection.sha256,undefined,'R3 selection manifest');
  if(!manifest.selection_artifact||manifest.selection_artifact.path!==plan.selection.path||manifest.selection_artifact.sha256!==plan.selection.sha256||manifest.selection_artifact.bytes!==pin.bytes||manifest.selection_artifact.selected_test_count!==9) fail('R3 source manifest does not bind current selection bytes/count');
  const expected=selectionInventory(selection);
  const lib=plan.targets.find(t=>t.cargo_flag==='--lib');
  const integration=plan.targets.find(t=>t.cargo_flag==='--test');
  if(!lib||!integration||stable(lib.test_names)!==stable(expected.unit)||stable(integration.test_names)!==stable(expected.integration)) fail('runplan selected tests differ from exact R3 selection manifest');
  return {pin,selected_test_count:selection.selected_test_count,unit_tests:expected.unit,integration_tests:expected.integration};
}
function verifyReviewBindings(plan, manifest) {
  const s=plan.source;
  const sourceReviewPin=regularPin(s.source_review_path,s.source_review_sha256,undefined,'R4 source review');
  const originalManifestPin=regularPin(s.reviewed_manifest_path,s.reviewed_manifest_sha256,undefined,'R4 manifest reviewed by source reviewer');
  const correctedManifestPin=regularPin(s.manifest_path,s.manifest_sha256,undefined,'R4 corrected manifest copy');
  const correctionAddendumPin=regularPin(s.selection_correction_path,s.selection_correction_sha256,undefined,'R4 metadata correction addendum');
  const correctionReviewPin=regularPin(s.selection_correction_review_path,s.selection_correction_review_sha256,undefined,'R4 metadata correction review');
  const priorSourceReviewPin=regularPin(s.prior_source_review_path,s.prior_source_review_sha256,undefined,'historical R3 source review');
  const bindingPin=regularPin(s.execution_binding_addendum_path,s.execution_binding_addendum_sha256,undefined,'R3 execution-binding addendum');
  const metadataReviewPin=regularPin(s.metadata_review_path,s.metadata_review_sha256,undefined,'R3 metadata review addendum');
  const sourceReview=readJson(s.source_review_path),priorSourceReview=readJson(s.prior_source_review_path),binding=readJson(s.execution_binding_addendum_path),metadataReview=readJson(s.metadata_review_path);
  const currentPacket=sourceReview.packet||{},currentIdentity=sourceReview.source_identity||{};
  if(sourceReview.verdict!=='READY_FOR_ROOT_TEST_EXECUTION_REVIEW'||sourceReview.execution_status!=='NOT_RUN'||currentPacket.path!==path.dirname(s.reviewed_manifest_path)||currentPacket.manifest_sha256!==originalManifestPin.sha256||currentPacket.full_source_patch_sha256!==s.patch_sha256||currentIdentity.root!==s.root||currentIdentity.base_commit!==s.base_commit||currentIdentity.base_tree!==s.base_tree||currentIdentity.changed_path_from_r3!=='crates/tuiscotti-runtime/tests/tui_shell/replay.rs') fail('R4 source review does not bind the current source candidate');
  if(sourceReview.preserved_prior_result?.r3_actual_result_path!==s.prior_result_path||sourceReview.preserved_prior_result?.r3_actual_result_sha256!==s.prior_result_sha256||manifest.r4_manifest?.prior?.r3_actual_result_sha256!==s.prior_result_sha256) fail('R4 source review does not bind the preserved R3 failure');
  if(priorSourceReview.verdict!=='READY_SOURCE_AND_SELECTION_REVIEW_ONLY'||priorSourceReview.qualification!=='NOT_RUN_NOT_QUALIFIED'||priorSourceReview.candidate.source_root!==s.prior_root||priorSourceReview.candidate.base_commit!==s.base_commit||priorSourceReview.candidate.base_tree!==s.base_tree||priorSourceReview.candidate.full_source_patch_sha256!==s.prior_patch_sha256) fail('historical R3 source review does not bind its source candidate');
  if(binding.schema!=='termrock-vis19-cursor-selection-binding-addendum/v1'||binding.source_packet.path!==s.prior_manifest_path||binding.source_packet.sha256!==s.prior_manifest_sha256||binding.source_packet.selection_path!==plan.selection.path||binding.source_packet.selection_sha256!==sha(read(plan.selection.path))||binding.source_packet.source_path!==s.prior_root||binding.source_packet.source_patch_sha256!==s.prior_patch_sha256||binding.current_selection.run_status!=='NOT_RUN'||binding.current_selection.selected_test_count!==9) fail('R3 execution-binding addendum mismatch');
  if(priorSourceReview.candidate.manifest_sha256!==binding.previous_reviewed_pins.manifest_sha256||priorSourceReview.candidate.selection_sha256!==binding.previous_reviewed_pins.selection_sha256||metadataReview.prior_review.prior_manifest_sha256!==priorSourceReview.candidate.manifest_sha256||metadataReview.prior_review.prior_selection_sha256!==priorSourceReview.candidate.selection_sha256) fail('R3 metadata addendum does not preserve historical review pins');
  if(metadataReview.verdict!=='READY_WITH_SELECTION_METADATA_DELTA'||metadataReview.qualification!=='NOT_RUN_NOT_QUALIFIED'||metadataReview.current_artifacts.root!==path.dirname(s.prior_manifest_path)||metadataReview.current_artifacts.manifest_sha256!==s.prior_manifest_sha256||metadataReview.current_artifacts.selection_sha256!==sha(read(plan.selection.path))||metadataReview.current_artifacts.source_patch_sha256!==s.prior_patch_sha256||metadataReview.prior_review.receipt!==s.prior_source_review_path||metadataReview.prior_review.receipt_sha256!==s.prior_source_review_sha256) fail('R3 metadata review does not bind current selection metadata');
  if(manifest.patch.sha256!==s.patch_sha256||manifest.worktree.path!==s.root) fail('R4 manifest/source review linkage mismatch');
  if(manifest.r4_manifest?.path!==s.manifest_path||manifest.r4_manifest?.sha256!==correctedManifestPin.sha256||manifest.r4_manifest?.original_path!==s.reviewed_manifest_path||manifest.r4_manifest?.original_sha256!==originalManifestPin.sha256)fail('R4 corrected manifest provenance is not present in the composed source identity');
  return {source_review:sourceReviewPin,original_manifest:originalManifestPin,corrected_manifest:correctedManifestPin,selection_correction_addendum:correctionAddendumPin,selection_correction_review:correctionReviewPin,prior_source_review:priorSourceReviewPin,execution_binding_addendum:bindingPin,metadata_review:metadataReviewPin,verdicts:[sourceReview.verdict,priorSourceReview.verdict,metadataReview.verdict]};
}
function testSourceInventory(plan, selection) {
  const checks=[];
  const all=[];
  for (const target of plan.targets) {
    const sourcePath=path.join(plan.source.root,target.source_path);
    const body=read(sourcePath).toString('utf8');
    const declared=new Set(Array.from(body.matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/g),m=>m[1]));
    for (const name of target.test_names) if (!declared.has(name.split('::').at(-1))) fail('selected test function absent from pinned source: '+name);
    all.push(...target.test_names);
    checks.push({target:target.cargo_target,source_path:target.source_path,source_sha256:sha(read(sourcePath)),selected_names:target.test_names});
  }
  if (all.length!==9 || new Set(all).size!==9) fail('test inventory must contain nine unique names');
  const expected=selectionInventory(selection);
  const lib=plan.targets.find(t=>t.cargo_flag==='--lib');
  const integration=plan.targets.find(t=>t.cargo_flag==='--test');
  if(!lib||!integration||stable(lib.test_names)!==stable(expected.unit)||stable(integration.test_names)!==stable(expected.integration)) fail('source test inventory differs from R3 selection');
  const r={tests:checks,total:all.length};r.digest=sha(Buffer.from(stable(r)));return r;
}
function loadInputs() {
  const plan=readJson(PLAN_PATH);
  if (plan.schema!=='termrock.vis19.cursor-nextest-plan/v1' || plan.state!=='PREPARED_FOR_ROOT_REVIEW_ONLY_NOT_AUTHORIZED' || plan.owner!=='Root only') fail('execution plan is not a Root-only review packet');
  if (plan.packet_root!==PACKET || plan.runner_path!==__filename) fail('plan packet/runner path mismatch');
  if (plan.targets.length!==2 || plan.targets.reduce((n,t)=>n+t.test_names.length,0)!==9) fail('plan must select exactly nine tests in two target binaries');
  if (plan.package.name!=='tuiscotti-runtime' || plan.package.features.length!==1 || plan.package.features[0]!=='pty') fail('runtime package/features mismatch');
  const manifest=composeR4Manifest(plan);
  const selection=readJson(plan.selection.path);
  const closure=readJson(plan.cache_closure_path);
  return {plan,manifest,selection,closure};
}
function buildCommand(input, runPaths) {
  const p=input.plan, args=['nextest','run','--manifest-path',p.package.manifest_path,'--package',p.package.name,'--locked','--offline','--features','pty','--profile','default','--build-jobs','2'];
  for (const t of p.targets) {
    if (t.cargo_flag==='--lib') args.push('--lib');
    else if (t.cargo_flag==='--test') args.push('--test',t.cargo_target);
    else fail('unsupported Cargo test selector '+t.cargo_flag);
  }
  args.push('--config-file',p.nextest_config.path,'--user-config-file','none','--ignore-default-filter','-E',p.filter_expression,'--no-tests','fail','--retries','0','--test-threads','1','--message-format','libtest-json-plus','--message-format-version','0.1','--no-fail-fast');
  const env={
    CARGO_HOME:p.cargo_home,
    CARGO_TARGET_DIR:runPaths.target,
    CARGO_BUILD_JOBS:'2',
    CARGO_NET_OFFLINE:'true',
    CARGO_TERM_COLOR:'never',
    HOME:runPaths.home,
    TMPDIR:runPaths.tmp,
    TMP:runPaths.tmp,
    TEMP:runPaths.tmp,
    PATH:[path.dirname(p.tools.nextest.path),path.dirname(p.tools.cargo.path),'/usr/bin','/bin','/usr/sbin','/sbin'].join(path.delimiter),
    RUSTUP_HOME:p.rustup_home,
    RUSTUP_TOOLCHAIN:p.rustup_toolchain,
    RUSTC:p.tools.rustc.path,
    RUSTDOC:p.tools.rustdoc.path,
    NEXTEST_EXPERIMENTAL_LIBTEST_JSON:'1',
    CARGO_TERM_PROGRESS_WHEN:'never',
    LANG:'C',
    LC_ALL:'C',
    TERM:'xterm-256color'
  };
  return {program:p.tools.cargo.path,argv:args,cwd:p.source.root,env};
}
function filterExpr(targets) {
  return targets.flatMap(t=>t.test_names.map(n=>'test(='+n+')')).join(' | ');
}
function snapshot(input,cachePhase='before') {
  const source=verifySource(input.plan,input.manifest);
  const selection=verifySelection(input.plan,input.manifest,input.selection);
  const reviews=verifyReviewBindings(input.plan,input.manifest);
  const metadata=verifyMetadata(input.plan);
  const tools=verifyTools(input.plan);
  const cache=verifyCache(input.plan,input.closure,cachePhase);
  const tests=testSourceInventory(input.plan,input.selection);
  const cfg=regularPin(input.plan.nextest_config.path,input.plan.nextest_config.sha256,undefined,'Nextest config');
  return {source,selection,reviews,metadata,tools,cache,tests,nextest_config:cfg};
}
function writePreflight() {
  const input=loadInputs();
  if (input.plan.filter_expression!==filterExpr(input.plan.targets)) fail('filter expression does not equal exact selected names');
  if (!absent(PREFLIGHT_PATH)) fail('preflight already exists');
  const runPaths={root:input.plan.output_root,home:path.join(input.plan.output_root,'home'),tmp:path.join(input.plan.output_root,'tmp'),target:path.join(input.plan.output_root,'target')};
  for (const p of Object.values(runPaths)) if (!absent(p)) fail('execution output path already exists: '+p);
  noSymlinkPath(input.plan.source.root);
  const before=snapshot(input,'before');
  const cmd=buildCommand(input,runPaths);
  const receipt={schema:'termrock.vis19.cursor-nextest-preflight/v1',status:'PREFLIGHT_READY_NO_CARGO_NEXTTEST',cargo_spawned:false,nextest_spawned:false,tests_run:false,network_used:false,runner:{path:__filename,sha256:sha(read(__filename))},plan:{path:PLAN_PATH,sha256:sha(read(PLAN_PATH))},source_manifest:{path:input.plan.source.manifest_path,sha256:sha(read(input.plan.source.manifest_path))},source_manifest_original:{path:input.plan.source.reviewed_manifest_path,sha256:sha(read(input.plan.source.reviewed_manifest_path))},selection_metadata_correction:{path:input.plan.source.selection_correction_path,sha256:sha(read(input.plan.source.selection_correction_path))},selection_metadata_correction_review:{path:input.plan.source.selection_correction_review_path,sha256:sha(read(input.plan.source.selection_correction_review_path))},source_patch:{path:input.plan.source.patch_path,sha256:sha(read(input.plan.source.patch_path))},source_reviews:{r4_source_review_sha256:sha(read(input.plan.source.source_review_path)),r3_source_review_sha256:sha(read(input.plan.source.prior_source_review_path)),execution_binding_addendum_sha256:sha(read(input.plan.source.execution_binding_addendum_path)),metadata_review_sha256:sha(read(input.plan.source.metadata_review_path))},selection:{path:input.plan.selection.path,sha256:sha(read(input.plan.selection.path))},cache_closure:{path:input.plan.cache_closure_path,sha256:sha(read(input.plan.cache_closure_path))},metadata_result:{path:input.plan.metadata.result_path,sha256:sha(read(input.plan.metadata.result_path)),stdout_sha256:sha(read(input.plan.metadata.stdout_path))},inputs_before:before,run_paths:runPaths,command:cmd,selected_test_count:9,inventory_policy:'Per binary: each selected test must have one started and one successful terminal; emitted nonselected test rows must have one started and one ignored terminal. Every suite start test_count equals selected IDs for that binary. Repeated suite starts/progress summaries are retained; final summary counts must match selected terminal results. suite.ignored is not equated with emitted ignored rows, and filtered_out is preserved as its raw unsigned decimal token without comparison to per-test rows.',bounds:input.plan.bounds,created_at_utc:new Date().toISOString()};
  writeJson(PREFLIGHT_PATH,receipt);
  process.stdout.write(JSON.stringify({status:receipt.status,preflight:PREFLIGHT_PATH,preflight_sha256:sha(read(PREFLIGHT_PATH)),selected_test_count:9,source_digest:before.source.digest,cache_digest:before.cache.digest},null,2)+'\n');
}
function parseEvents(buffer,input) {
  const errors=[], records=[];
  const text=buffer.toString('utf8');
  if (!Buffer.from(text,'utf8').equals(buffer)) errors.push('stdout is not valid UTF-8');
  for (const [i,line] of text.split(/\r?\n/).entries()) {
    if (!line) continue;
    try { const row=JSON.parse(line); if(!row||typeof row!=='object'||Array.isArray(row)) throw new Error('not object'); records.push({line:i+1,raw:line,row}); }
    catch(e){errors.push('invalid JSON record line '+(i+1)+': '+String(e&&e.message||e));}
  }
  const targetByBinary=new Map(input.plan.targets.map(t=>[t.event_binary,t]));
  const testStates=new Map();
  const suites=new Map();
  const prefix=input.plan.package.name+'::';
  for(const rec of records){
    const e=rec.row;
    if(e.type==='suite'){
      const meta=e.nextest||{};
      if(meta.crate!==input.plan.package.name||typeof meta.test_binary!=='string'){errors.push('suite package/binary identity invalid at line '+rec.line);continue;}
      const target=targetByBinary.get(meta.test_binary);
      if(!target||meta.kind!==target.event_kind){errors.push('suite target/kind mismatch '+String(meta.test_binary)+'/'+String(meta.kind));continue;}
      let s=suites.get(meta.test_binary);
      if(!s){s={binary:meta.test_binary,kind:meta.kind,starts:[],summaries:[]};suites.set(meta.test_binary,s);}
      if(e.event==='started'){
        if(!Number.isSafeInteger(e.test_count)||e.test_count!==target.test_names.length)errors.push('suite test_count does not equal selected IDs for '+meta.test_binary);
        s.starts.push({line:rec.line,test_count:e.test_count});
      }else if(e.event==='ok'||e.event==='failed'){
        const raw=rec.raw.match(/"filtered_out"\s*:\s*(\d+)(?=\s*[,}])/);
        const summary={line:rec.line,event:e.event,passed:e.passed,failed:e.failed,ignored:e.ignored,measured:e.measured,filtered_out_raw:raw?raw[1]:null};
        for(const k of ['passed','failed','ignored','measured']) if(!Number.isSafeInteger(e[k])||e[k]<0)errors.push('invalid suite count '+k+' for '+meta.test_binary);
        if(summary.filtered_out_raw===null)errors.push('suite filtered_out token is missing/noninteger for '+meta.test_binary);
        s.summaries.push(summary);
      }else errors.push('unexpected suite lifecycle event '+String(e.event));
      continue;
    }
    if(e.type!=='test'){errors.push('unexpected JSON event type '+String(e.type));continue;}
    if(typeof e.name!=='string'||!e.name.startsWith(prefix)){errors.push('invalid test identity at line '+rec.line);continue;}
    const target=input.plan.targets.find(t=>e.name.startsWith(prefix+t.event_binary+'$'));
    if(!target){errors.push('test event belongs to unselected binary: '+e.name);continue;}
    const name=e.name.slice((prefix+target.event_binary+'$').length);
    const key=target.event_binary+'$'+name;
    let state=testStates.get(key);
    if(!state){state={binary:target.event_binary,name,selected:target.test_names.includes(name),started:0,started_lines:[],terminals:[],terminal_lines:[]};testStates.set(key,state);}
    if(e.event==='started'){state.started++;state.started_lines.push(rec.line);}
    else if(['ok','failed','ignored','measured','allowed_fail','timeout','leaked'].includes(e.event)){state.terminals.push(e.event);state.terminal_lines.push(rec.line);}
    else errors.push('unexpected test event '+String(e.event)+' for '+key);
  }
  for(const target of input.plan.targets){
    for(const name of target.test_names){
      const key=target.event_binary+'$'+name,s=testStates.get(key);
      if(!s||s.started!==1||s.terminals.length!==1||s.terminals[0]!=='ok'||s.started_lines[0]>=s.terminal_lines[0])errors.push('selected test did not have exactly one start-before-ok pair: '+key);
    }
  }
  for(const [key,s] of testStates){
    if(s.started!==1||s.terminals.length!==1)errors.push('test inventory event lacks unique start/terminal: '+key);
    if(s.started_lines[0]>=s.terminal_lines[0])errors.push('test terminal does not follow its start: '+key);
    if(!s.selected&&s.terminals[0]!=='ignored')errors.push('nonselected inventory event is not ignored: '+key);
    if(s.selected&&s.terminals[0]!=='ok')errors.push('selected inventory terminal is not successful: '+key);
  }
  const suiteSummaryResults=[];
  for(const target of input.plan.targets){
    const seen=[...testStates.values()].filter(x=>x.binary===target.event_binary);
    const selectedOk=seen.filter(x=>x.selected&&x.terminals[0]==='ok').length;
    const selectedFailed=seen.filter(x=>x.selected&&x.terminals[0]==='failed').length;
    if(selectedOk+selectedFailed!==target.test_names.length)errors.push('selected terminal count mismatch: '+target.event_binary);
    const suite=suites.get(target.event_binary);
    if(!suite||suite.starts.length===0)errors.push('suite start event missing: '+target.event_binary);
    if(!suite||suite.summaries.length===0)errors.push('suite terminal summary missing: '+target.event_binary);
    if(suite){
      for(const start of suite.starts)if(start.test_count!==target.test_names.length)errors.push('suite start count mismatch: '+target.event_binary);
      const summaries=suite.summaries;
      for(let i=0;i<summaries.length;i++){
        const row=summaries[i],previous=summaries[i-1];
        const selectedOkObserved=seen.filter(x=>x.selected&&x.terminals[0]==='ok'&&x.terminal_lines[0]<row.line).length;
        const selectedFailedObserved=seen.filter(x=>x.selected&&x.terminals[0]==='failed'&&x.terminal_lines[0]<row.line).length;
        if(!suite.starts.some(start=>start.line<row.line))errors.push('suite summary precedes every suite start: '+target.event_binary+' line '+row.line);
        if(row.passed>selectedOkObserved||row.failed>selectedFailedObserved)errors.push('suite progress counts exceed selected outcomes observed by summary line: '+target.event_binary+' line '+row.line);
        if(row.event!=='ok'||row.failed!==0)errors.push('suite reports a failed terminal for all-pass selection: '+target.event_binary+' line '+row.line);
        if(row.measured!==0)errors.push('suite reports measured tests for non-benchmark selection: '+target.event_binary+' line '+row.line);
        if(previous&&(row.passed<previous.passed||row.failed<previous.failed))errors.push('suite progress summary counts moved backwards: '+target.event_binary);
      }
      const final=summaries.at(-1);
      const lastSelectedTerminalLine=Math.max(0,...seen.filter(x=>x.selected).flatMap(x=>x.terminal_lines));
      if(final&&final.line<lastSelectedTerminalLine)errors.push('final suite summary precedes selected terminal events: '+target.event_binary);
      if(final&&(final.passed!==selectedOk||final.failed!==selectedFailed||final.measured!==0))errors.push('final suite summary does not match selected terminal results: '+target.event_binary);
      suiteSummaryResults.push({binary:target.event_binary,kind:target.event_kind,started:suite.starts,terminal_summaries:summaries,selected_ok:selectedOk,selected_failed:selectedFailed});
    }
  }
  const selectedResults=input.plan.targets.flatMap(t=>t.test_names.map(n=>({binary:t.event_binary,name:n,event:testStates.get(t.event_binary+'$'+n)?.terminals[0]||'missing'})));
  return {valid:errors.length===0,errors,record_count:records.length,record_types:records.reduce((a,r)=>{const k=r.row.type+':'+r.row.event;a[k]=(a[k]||0)+1;return a;},{}),suite_events:suiteSummaryResults,discovered_test_inventory:[...testStates.values()].map(s=>({binary:s.binary,name:s.name,selected:s.selected,started:s.started,terminal:s.terminals[0]||null})).sort((a,b)=>a.binary.localeCompare(b.binary)||a.name.localeCompare(b.name)),selected_results:selectedResults,selected_count:selectedResults.length,selected_passed:selectedResults.filter(x=>x.event==='ok').length,selected_failed:selectedResults.filter(x=>x.event==='failed').length,selected_other:selectedResults.filter(x=>x.event!=='ok'&&x.event!=='failed').length,filtered_ignored_count:[...testStates.values()].filter(x=>!x.selected&&x.terminals[0]==='ignored').length};
}
function parseHumanSummary(buffer,input) {
  const errors=[];
  const text=buffer.toString('utf8');
  if(!Buffer.from(text,'utf8').equals(buffer))errors.push('stderr is not valid UTF-8');
  const plain=text.replace(/\x1b\[[0-?]*[ -/]*[@-~]/g,'');
  const runIds=[...plain.matchAll(/Nextest run ID\s+([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/gi)].map(m=>m[1].toLowerCase());
  if(new Set(runIds).size>1)errors.push('conflicting Nextest run IDs in stderr');
  const expected=input.plan.targets.reduce((n,t)=>n+t.test_names.length,0);
  const summaryLines=plain.split(/\r?\n/).filter(line=>/\bSummary\s*\[/.test(line));
  const summaries=[];
  const pattern=/^\s*Summary\s+\[\s*[^\]]+\]\s+(\d+)\s+tests?\s+run:\s*(\d+)\s+passed(?:,\s*(\d+)\s+failed)?(?:,\s*(\d+)\s+skipped)?\s*$/;
  for(const line of summaryLines){
    const m=line.match(pattern);
    if(!m){errors.push('unparsed Nextest human summary line');continue;}
    const row={total:Number(m[1]),passed:Number(m[2]),failed:Number(m[3]||0),skipped:Number(m[4]||0),raw:line};
    if(![row.total,row.passed,row.failed,row.skipped].every(Number.isSafeInteger))errors.push('Nextest human summary count is not a safe integer');
    if(row.total!==expected||row.passed!==expected||row.failed!==0||row.skipped!==0)errors.push('Nextest human summary contradicts the selected JSON test results');
    summaries.push(row);
  }
  return {valid:errors.length===0,errors,present:summaries.length>0,run_ids:runIds,summaries,expected_selected_count:expected,raw_stderr_sha256:sha(buffer)};
}
function processGroupSnapshot(pid) {
  if(!pid)return {checked:false,members:[],clean:true};
  const r=spawnSync('/bin/ps',['-axo','pid=,ppid=,pgid=,stat='],{encoding:'utf8',timeout:15000,maxBuffer:512*1024});
  if(r.error||r.status!==0) return {checked:false,error:String(r.error||r.stderr||r.status),members:[],clean:false};
  const members=r.stdout.split(/\r?\n/).filter(Boolean).map(line=>{const m=line.trim().match(/^(\d+)\s+(\d+)\s+(\d+)\s+(\S+)$/);return m?{pid:Number(m[1]),ppid:Number(m[2]),pgid:Number(m[3]),state:m[4]}:null;}).filter(Boolean).filter(x=>x.pgid===pid);
  return {checked:true,members,clean:members.length===0};
}
function artifactSnapshot(targetDir,input) {
  if(!fs.existsSync(targetDir))return {ok:false,errors:['target directory missing'],artifacts:[]};
  const deps=path.join(targetDir,'debug/deps');
  if(!fs.existsSync(deps))return {ok:false,errors:['target debug/deps missing'],artifacts:[]};
  const entries=fs.readdirSync(deps);
  const artifacts=[],errors=[],depInputs=[];
  for(const name of entries.filter(n=>n.endsWith('.d'))){
    const full=path.join(deps,name),st=fs.lstatSync(full);
    if(!st.isFile()||st.isSymbolicLink()){errors.push('dep-info is not a regular file: '+full);continue;}
    depInputs.push({path:full,bytes:st.size,sha256:sha(read(full)),text:fs.readFileSync(full,'utf8')});
  }
  for(const t of input.plan.targets){
    const matches=entries.filter(n=>new RegExp('^'+t.event_binary.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')+'-[0-9a-f]+$').test(n));
    if(matches.length!==1){errors.push('expected one executable for '+t.event_binary+', observed '+matches.length);continue;}
    const bin=path.join(deps,matches[0]),st=fs.lstatSync(bin);
    if(!st.isFile()||st.isSymbolicLink()||(st.mode&0o111)===0){errors.push('test binary is not a regular executable: '+bin);continue;}
    const d=bin+'.d';
    if(!fs.existsSync(d)){errors.push('test binary dep-info missing: '+d);continue;}
    const dep=fs.readFileSync(d,'utf8');
    const sourcePrefix=t.cargo_flag==='--lib'?'crates/tuiscotti-runtime/src/':'crates/tuiscotti-runtime/tests/';
    const sourceInputs=[t.source_path,...input.manifest.changed_files.filter(x=>x.path.startsWith(sourcePrefix)).map(x=>x.path)];
    const missing=sourceInputs.filter(rel=>!dep.includes(rel)&&!dep.includes(path.join(input.plan.source.root,rel)));
    if(missing.length)errors.push('dep-info omits source inputs for '+t.event_binary+': '+missing.join(','));
    artifacts.push({target:t.event_binary,path:bin,sha256:sha(read(bin)),bytes:st.size,mode:st.mode&0o7777,dep_info_path:d,dep_info_sha256:sha(read(d)),dep_info_bytes:fs.statSync(d).size,source_paths_checked:sourceInputs,source_paths_missing:missing});
  }
  const allSourceChecks=input.manifest.changed_files.map(x=>({path:x.path,present_in_some_dep_info:depInputs.some(d=>d.text.includes(x.path)||d.text.includes(path.join(input.plan.source.root,x.path)))}));
  for(const item of allSourceChecks)if(!item.present_in_some_dep_info)errors.push('changed source path absent from all fresh dep-info files: '+item.path);
  return {ok:errors.length===0,errors,artifacts,all_source_dep_info:allSourceChecks,dep_info_inventory:depInputs.map(d=>({path:d.path,bytes:d.bytes,sha256:d.sha256}))};
}
function inputSnapshot(input,cachePhase='before') {
  return snapshot(input,cachePhase);
}
function stableNonCacheInputs(inputs) {
  return stable({source:inputs.source,selection:inputs.selection,reviews:inputs.reviews,metadata:inputs.metadata,tools:inputs.tools,tests:inputs.tests,nextest_config:inputs.nextest_config});
}
function cacheInputsStable(before,after) {
  if(!before||!after)return false;
  if(before.cargo_home.path!==after.cargo_home.path||before.cargo_home.realpath!==after.cargo_home.realpath||before.cargo_home.mode!==after.cargo_home.mode||before.cargo_home.dev!==after.cargo_home.dev||before.cargo_home.ino!==after.cargo_home.ino)return false;
  if(stable(before.seed_rows)!==stable(after.seed_rows))return false;
  if(stable(before.mutate_lock)!==stable(after.mutate_lock))return false;
  if(stable(before.home_inventory.files)!==stable(after.home_inventory.files)||stable(before.home_inventory.dirs)!==stable(after.home_inventory.dirs))return false;
  const metadataIdentity=rows=>rows.map(x=>({path:x.path,mode:x.mode,dev:x.dev,ino:x.ino,nlink:x.nlink,classification:x.classification}));
  if(stable(metadataIdentity(before.metadata))!==stable(metadataIdentity(after.metadata)))return false;
  const oldPackages=new Map(before.registry_source_tree.packages.map(x=>[x.name,x]));
  const newPackages=new Map(after.registry_source_tree.packages.map(x=>[x.name,x]));
  for(const [name,oldRow] of oldPackages){
    const next=newPackages.get(name);
    if(!next||stable(oldRow)!==stable(next))return false;
  }
  const lockedArchiveRoots=before.registry_source_tree.locked_archive_roots;
  if(after.registry_source_tree.locked_archive_roots!==lockedArchiveRoots)return false;
  if(after.registry_source_tree.extracted_package_count<before.registry_source_tree.extracted_package_count)return false;
  return true;
}
async function waitGroupGone(pid, maxMs) {
  const end=Date.now()+maxMs;
  while(Date.now()<end){const s=processGroupSnapshot(pid);if(s.checked&&s.clean)return {clean:true,snapshot:s};await new Promise(r=>setTimeout(r,100));}
  return {clean:false,snapshot:processGroupSnapshot(pid)};
}
function execute(expectedPreflightSha) {
  const input=loadInputs();
  const preBytes=read(PREFLIGHT_PATH),preSha=sha(preBytes),pre=JSON.parse(preBytes.toString('utf8'));
  if(preSha!==expectedPreflightSha||pre.status!=='PREFLIGHT_READY_NO_CARGO_NEXTTEST'||pre.runner.sha256!==sha(read(__filename))||pre.plan.sha256!==sha(read(PLAN_PATH)))fail('preflight SHA/status/runner/plan binding mismatch');
  const before=inputSnapshot(input,'before');
  if(stable(before)!==stable(pre.inputs_before))fail('source/tool/cache/metadata inputs changed after reviewed preflight');
  const root=input.plan.output_root,home=path.join(root,'home'),tmp=path.join(root,'tmp'),target=path.join(root,'target');
  for(const p of [root,home,tmp,target])if(!absent(p))fail('fresh execution path appeared after preflight: '+p);
  ensurePrivateDir(root,true);ensurePrivateDir(home,true);ensurePrivateDir(tmp,true);
  const runPaths={root,home,tmp,target};
  const command=buildCommand(input,runPaths);
  const output={stdout:path.join(root,'nextest.stdout.jsonl'),stderr:path.join(root,'nextest.stderr.raw'),events:path.join(root,'process-events.jsonl'),receipt:path.join(root,'receipt.json')};
  const fds={stdout:fs.openSync(output.stdout,'wx',0o600),stderr:fs.openSync(output.stderr,'wx',0o600),events:fs.openSync(output.events,'wx',0o600)};
  const caps={per_stream:8*1024*1024,combined:16*1024*1024};
  const streams={stdout:{received:0,stored:0,truncated:false},stderr:{received:0,stored:0,truncated:false}};
  let combined=0,child=null,close=null,spawnError=null,timedOut=false,interrupted=null,capExceeded=false,forceClose=false,finalizing=false,finished=false;
  let timeoutTimer=null,termTimer=null,killTimer=null;
  const errors=[];const start=Date.now();
  function event(v){try{fs.writeSync(fds.events,JSON.stringify({at:new Date().toISOString(),...v})+'\n');}catch(e){errors.push('event log: '+String(e));}}
  function signalGroup(sig){if(!child||!child.pid)return;try{process.kill(-child.pid,sig);}catch(e){if(e.code!=='ESRCH')errors.push('signal '+sig+': '+String(e));}}
  function stop(reason){if(finalizing||finished)return;event({event:'stop',reason});signalGroup('SIGTERM');termTimer=setTimeout(()=>{signalGroup('SIGKILL');killTimer=setTimeout(()=>{if(!close){forceClose=true;try{if(child&&child.pid)child.kill('SIGKILL');}catch(e){errors.push('direct kill: '+String(e));}if(child?.stdout)child.stdout.destroy();if(child?.stderr)child.stderr.destroy();close={code:null,signal:'SIGKILL_FORCED_CLOSE'};void finalize();}},5000);},5000);}
  function capture(which,chunk){if(finalizing||finished)return;const s=streams[which];s.received+=chunk.length;const perLeft=Math.max(0,caps.per_stream-s.stored),combinedLeft=Math.max(0,caps.combined-combined),keep=Math.min(chunk.length,perLeft,combinedLeft);if(keep){try{fs.writeSync(fds[which],chunk,0,keep);s.stored+=keep;combined+=keep;}catch(e){errors.push(which+' write: '+String(e));stop('OUTPUT_WRITE_FAILURE');}}if(keep<chunk.length){s.truncated=true;capExceeded=true;stop('OUTPUT_CAP_EXCEEDED');}}
  const onInt=()=>{interrupted='SIGINT';event({event:'parent-signal',signal:interrupted});stop('INTERRUPTED');};
  const onTerm=()=>{interrupted='SIGTERM';event({event:'parent-signal',signal:interrupted});stop('INTERRUPTED');};
  process.on('SIGINT',onInt);process.on('SIGTERM',onTerm);
  async function finalize(){
    if(finalizing||finished)return;finalizing=true;clearTimeout(timeoutTimer);clearTimeout(termTimer);clearTimeout(killTimer);
    let cleanup={clean:false,snapshot:{checked:false,members:[]}};
    if(child?.pid){cleanup=await waitGroupGone(child.pid,10000);if(!cleanup.clean){signalGroup('SIGTERM');cleanup=await waitGroupGone(child.pid,5000);}if(!cleanup.clean){signalGroup('SIGKILL');cleanup=await waitGroupGone(child.pid,5000);}}
    process.removeListener('SIGINT',onInt);process.removeListener('SIGTERM',onTerm);
    for(const fd of Object.values(fds)){try{fs.fsyncSync(fd);fs.closeSync(fd);}catch(e){errors.push('close/fsync: '+String(e));}}
    const stdout=read(output.stdout),stderr=read(output.stderr);
    let afterInputs=null,eventsResult=null,artifacts=null,cacheAfter=null,humanSummary={valid:false,errors:['not parsed']};
    try{afterInputs=inputSnapshot(input,'after');}catch(e){errors.push('postflight inputs: '+String(e));}
    try{eventsResult=capExceeded?{valid:false,errors:['output cap exceeded'],record_count:0,selected_count:9,selected_passed:0,selected_failed:0,selected_other:9,filtered_ignored_count:0,discovered_test_inventory:[],suite_events:[]}:parseEvents(stdout,input);}catch(e){eventsResult={valid:false,errors:['JSON event parser: '+String(e)],record_count:0,selected_count:9,selected_passed:0,selected_failed:0,selected_other:9,filtered_ignored_count:0,discovered_test_inventory:[],suite_events:[]};}
    try{humanSummary=parseHumanSummary(stderr,input);}catch(e){humanSummary={valid:false,errors:['human summary parser: '+String(e)],present:false,run_ids:[],summaries:[],raw_stderr_sha256:sha(stderr)};}
    try{artifacts=artifactSnapshot(target,input);}catch(e){artifacts={ok:false,errors:[String(e)],artifacts:[]};}
    const inputsStable=afterInputs!==null&&stableNonCacheInputs(before)===stableNonCacheInputs(afterInputs)&&cacheInputsStable(before.cache,afterInputs.cache);
    const passed=Boolean(close&&close.code===0&&close.signal===null&&!timedOut&&!interrupted&&!capExceeded&&!spawnError&&!forceClose&&cleanup.clean&&inputsStable&&eventsResult.valid&&humanSummary.valid&&eventsResult.selected_count===9&&eventsResult.selected_passed===9&&eventsResult.selected_failed===0&&eventsResult.selected_other===0&&artifacts.ok&&!errors.length);
    const result={schema:'termrock.vis19.cursor-nextest-result/v1',status:passed?'PASS_SELECTED_NINE_TESTS':'FAIL_OR_INCOMPLETE',started_at_utc:new Date(start).toISOString(),finished_at_utc:new Date().toISOString(),elapsed_ms:Date.now()-start,preflight_path:PREFLIGHT_PATH,preflight_sha256:preSha,runner_path:__filename,runner_sha256:sha(read(__filename)),plan_path:PLAN_PATH,plan_sha256:sha(read(PLAN_PATH)),command,child:{pid:child&&child.pid||null,process_group:child&&child.pid||null,exit_code:close?close.code:null,signal:close?close.signal:null,spawn_error:spawnError,timed_out:timedOut,interrupted_by:interrupted,forced_close:forceClose,cargo_spawned:Boolean(child&&child.pid)},bounds:{timeout_ms:input.plan.bounds.timeout_ms,term_grace_ms:5000,kill_grace_ms:5000,process_group_cleanup_max_ms:20000,stdout_cap_bytes:caps.per_stream,stderr_cap_bytes:caps.per_stream,combined_cap_bytes:caps.combined,build_jobs:2,test_threads:1,retries:0},output:{stdout:{path:output.stdout,bytes:stdout.length,sha256:sha(stdout),received:streams.stdout.received,truncated:streams.stdout.truncated},stderr:{path:output.stderr,bytes:stderr.length,sha256:sha(stderr),received:streams.stderr.received,truncated:streams.stderr.truncated},events_path:output.events,combined_bytes:combined},human_nextest_summary:humanSummary,nextest:{valid:eventsResult.valid,errors:eventsResult.errors,record_count:eventsResult.record_count,suite_events:eventsResult.suite_events,selected_count:eventsResult.selected_count,selected_passed:eventsResult.selected_passed,selected_failed:eventsResult.selected_failed,selected_other:eventsResult.selected_other,filtered_ignored_count:eventsResult.filtered_ignored_count,discovered_test_inventory:eventsResult.discovered_test_inventory,selected_results:eventsResult.selected_results},process_group_cleanup:cleanup,inputs_before:before,inputs_after:afterInputs,inputs_stable:inputsStable,target_artifacts:artifacts,errors};
    writeJson(output.receipt,result);event({event:'receipt-written',path:output.receipt,status:result.status});
    finished=true;finalizing=false;process.exitCode=passed?0:(timedOut?124:(interrupted?130:(close&&Number.isInteger(close.code)&&close.code!==0?close.code:125)));
    process.stdout.write(JSON.stringify({status:result.status,receipt:output.receipt,exit_code:result.child.exit_code,selected:result.nextest.selected_count,passed:result.nextest.selected_passed,failed:result.nextest.selected_failed,filtered_ignored:result.nextest.filtered_ignored_count,inputs_stable:result.inputs_stable,cleanup_clean:cleanup.clean},null,2)+'\n');
  }
  event({event:'spawn-request',program:command.program,argv:command.argv,cwd:command.cwd});
  try{
    child=spawn(command.program,command.argv,{cwd:command.cwd,env:command.env,detached:true,stdio:['ignore','pipe','pipe']});
    child.stdout.on('data',b=>capture('stdout',b));child.stderr.on('data',b=>capture('stderr',b));
    child.on('spawn',()=>event({event:'spawned',pid:child.pid,process_group:child.pid}));
    child.on('error',e=>{spawnError=String(e);event({event:'spawn-error',error:spawnError});});
    child.on('close',(code,signal)=>{if(close||finalizing)return;close={code,signal};event({event:'child-close',code,signal});void finalize();});
    timeoutTimer=setTimeout(()=>{timedOut=true;stop('OUTER_TIMEOUT');},input.plan.bounds.timeout_ms);
  }catch(e){spawnError=String(e);close={code:null,signal:null};void finalize();}
}
function main(){
  const args=process.argv.slice(2);
  if(args.length===1&&args[0]==='--preflight-only'){writePreflight();return;}
  if(args.length===2&&args[0]==='--execute'&&/^[a-f0-9]{64}$/.test(args[1])){execute(args[1]);return;}
  fail('usage: runner.cjs --preflight-only | --execute <preflight-sha256>');
}
try{main();}catch(e){process.stderr.write(String(e&&e.stack||e)+'\n');process.exitCode=125;}
