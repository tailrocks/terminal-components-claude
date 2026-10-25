#!/usr/bin/env python3
"""Preserve source bytes for current F01 and F13 rechecks from the historical review target."""
import difflib
import hashlib
import json
import pathlib
import subprocess
from collections import Counter

REPO='/Users/donbeave/Downloads/terminal-components-claude'
OUT=pathlib.Path(REPO+'/docs/implementation/pr38-final/results/t00-source')
OLD='e08ac57d10ae54c87337eae0be08b1f5e8f4bda4'
NEW='1d6ee92adf6ccca38eeee64b1371cba1497e8fcd'
BASELINE='crates/holla-ui/tests/baselines/app.txt'
RENAMES=[
 ('crates/showcase-ui/src/coverage/allowlist.txt','crates/showcase-ui/tests/coverage/allowlist.txt'),
 ('crates/showcase-ui/src/coverage/catalog.rs','crates/showcase-ui/tests/coverage/catalog.rs'),
 ('crates/showcase-ui/src/coverage/catalog_data.rs','crates/showcase-ui/tests/coverage/catalog_data.rs'),
 ('crates/showcase-ui/src/coverage/declarations.rs','crates/showcase-ui/tests/coverage/declarations.rs'),
 ('crates/showcase-ui/src/coverage/declarations_data.rs','crates/showcase-ui/tests/coverage/declarations_data.rs'),
 ('crates/showcase-ui/src/coverage/mod.rs','crates/showcase-ui/tests/coverage/mod.rs'),
 ('crates/showcase-ui/src/coverage/scan.rs','crates/showcase-ui/tests/coverage/scan.rs'),
]

def blob(rev,path):
 oid=subprocess.check_output(['git','-C',REPO,'rev-parse',rev+':'+path],text=True).strip()
 data=subprocess.check_output(['git','-C',REPO,'cat-file','blob',oid])
 return oid,data

def record_bytes(folder,relative,data):
 p=OUT/'source-deltas'/'e08-to-1d6'/folder/relative
 p.parent.mkdir(parents=True,exist_ok=True); p.write_bytes(data)
 return {'blob_oid':None,'sha256':hashlib.sha256(data).hexdigest(),'size_bytes':len(data),'evidence_path':str(p.relative_to(OUT))}

old_oid,old_baseline=blob(OLD,BASELINE); new_oid,new_baseline=blob(NEW,BASELINE)
old_path=record_bytes('old',BASELINE,old_baseline); old_path['blob_oid']=old_oid
new_path=record_bytes('new',BASELINE,new_baseline); new_path['blob_oid']=new_oid
old_lines=old_baseline.decode('utf-8').splitlines(); new_lines=new_baseline.decode('utf-8').splitlines()
baseline_diff=list(difflib.unified_diff(old_lines,new_lines,fromfile=OLD+':'+BASELINE,tofile=NEW+':'+BASELINE,lineterm=''))

f13=[]; moved_contents={}
for old,new in RENAMES:
 old_oid,old_data=blob(OLD,old); new_oid,new_data=blob(NEW,new)
 old_meta=record_bytes('old',old,old_data); old_meta['blob_oid']=old_oid
 new_meta=record_bytes('new',new,new_data); new_meta['blob_oid']=new_oid
 if old_data!=new_data: raise RuntimeError(f'F13 rename bytes differ: {old} -> {new}')
 f13.append({'old_path':old,'new_path':new,'old':old_meta,'new':new_meta,'status':'byte-identical rename'})
 moved_contents[new]=new_data

allow=[line for line in moved_contents['crates/showcase-ui/tests/coverage/allowlist.txt'].decode('utf-8').splitlines() if line.strip() and not line.lstrip().startswith('#')]
categories=Counter(line.rsplit(' | ',1)[-1].strip() for line in allow)
proof_data=moved_contents['crates/showcase-ui/tests/coverage/declarations_data.rs'].decode('utf-8')
proof_count=proof_data.count('BehaviorProof {')

record={
 'historical_review_target':OLD,'frozen_source_input':NEW,
 'f01':{'area':'Holla baseline and remote CI','baseline_path':BASELINE,'old':old_path,'new':new_path,'baseline_line_diff':baseline_diff,'assessment':'The expected historical baseline changed in the target branch. Current run evidence is in ci-head-1d6-rest.json. The changed expected bytes and new test are not a green runtime proof.'},
 'f13':{'moved_files':f13,'allowlist_path':'crates/showcase-ui/tests/coverage/allowlist.txt','allowlist_rows':len(allow),'disposition_categories':dict(categories),'behavior_proof_rows':proof_count,'proof_kinds':'All six behavior proofs concern Diff, as retained from historical F13 review.','assessment':'All seven R100 files moved from src/coverage to tests/coverage with identical bytes. Coverage data is registry evidence, not a complete behavior demonstration.'},
 'f20':{'assessment':'Current product documentation paths are unchanged from the historical review input. The TablePro comparator source remains cmp_values_nulls_last; no test result is claimed.'},
 'runtime_evidence':{'tests_run_by_T00':False,'approved_test_plan_id':'N/A','zero_case_behavior':'NOT VERIFIED'}
}
(OUT/'finding-deltas-e08-to-1d6.json').write_text(json.dumps(record,ensure_ascii=False,indent=2,sort_keys=True)+'\n',encoding='utf-8')
print(json.dumps({'baseline_diff_lines':len(baseline_diff),'f13_moved_files':len(f13),'f13_files_identical':True,'allowlist_rows':len(allow),'categories':dict(categories),'behavior_proofs':proof_count},sort_keys=True))
