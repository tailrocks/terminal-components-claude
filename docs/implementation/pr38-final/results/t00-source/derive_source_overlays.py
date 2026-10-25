#!/usr/bin/env python3
"""Derive exact per-file source ledgers from the preserved 3a tree ledger."""
import gzip
import hashlib
import json
import pathlib
import subprocess

REPO='/Users/donbeave/Downloads/terminal-components-claude'
OUT=pathlib.Path(REPO+'/docs/implementation/pr38-final/results/t00-source')
BASE='3a2c977c881cd1745de16425c982791ee3ef2cc3'
MID='31f856f43d20920ea99aa81b8f40adeeb5bff358'
HEAD='1d6ee92adf6ccca38eeee64b1371cba1497e8fcd'
STEPS=[
 {'from':BASE,'to':MID,'paths':['crates/termrock-xtask/exceptions/v1.json'],'label':'3a-to-31f'},
 {'from':MID,'to':HEAD,'paths':['crates/showcase-ui/tests/fixtures/holla-page-headings.tsv','crates/showcase-ui/tests/navigation_contract.rs'],'label':'31f-to-1d6'},
]

def git(*args):
 return subprocess.check_output(['git','-C',REPO,*args])

def blob_row(commit,path,old):
 oid=git('rev-parse',commit+':'+path).decode().strip()
 data=git('cat-file','blob',oid)
 row=dict(old)
 row.update({'blob_oid':oid,'sha256':hashlib.sha256(data).hexdigest(),'size_bytes':len(data)})
 return row,data

def write_ledger(path,rows):
 with open(path,'wb') as raw:
  with gzip.GzipFile(fileobj=raw,mode='wb',mtime=0,compresslevel=9) as gz:
   for row in sorted(rows,key=lambda x:x['path']):
    gz.write((json.dumps(row,sort_keys=True,separators=(',',':'),ensure_ascii=False)+'\n').encode('utf-8'))

def tree_rows(path):
 with gzip.open(path,'rt',encoding='utf-8') as source:
  return [json.loads(line) for line in source]

def metrics(rows):
 return {'paths':len(rows),'tracked_bytes':sum(x['size_bytes'] for x in rows),'unique_blob_oids':len({x['blob_oid'] for x in rows})}

base_tree=tree_rows(OUT/'tracked-tree.jsonl.gz')
base_tests=tree_rows(OUT/'test-files.jsonl.gz')
states={BASE:{'tree':base_tree,'tests':base_tests}}
changes=[]

for step in STEPS:
 old_state=states[step['from']]
 old_by={x['path']:x for x in old_state['tree']}
 test_by={x['path']:x for x in old_state['tests']}
 new_tree=dict(old_by); new_tests=dict(test_by); rows=[]
 actual=set(git('diff','--name-only',step['from'],step['to']).decode().splitlines())
 expected=set(step['paths'])
 if actual!=expected: raise RuntimeError(f"unexpected delta {step['from']}..{step['to']}: {sorted(actual)}")
 for path in step['paths']:
  old_row=old_by[path]
  new_row,data=blob_row(step['to'],path,old_row)
  new_tree[path]=new_row
  if path in test_by: new_tests[path]=dict(new_row)
  old_data=git('cat-file','blob',old_row['blob_oid'])
  prefix=OUT/'source-deltas'/step['label']
  for state,content in (('old',old_data),('new',data)):
   target=prefix/state/path
   target.parent.mkdir(parents=True,exist_ok=True)
   target.write_bytes(content)
  rows.append({'path':path,'old':{'blob_oid':old_row['blob_oid'],'sha256':old_row['sha256'],'size_bytes':old_row['size_bytes'],'evidence_path':f"source-deltas/{step['label']}/old/{path}"},'new':{'blob_oid':new_row['blob_oid'],'sha256':new_row['sha256'],'size_bytes':new_row['size_bytes'],'evidence_path':f"source-deltas/{step['label']}/new/{path}"},'test_ledger_updated':path in test_by,'change':'source bytes replaced; all path and language classifications retained'})
 step['changed_files']=rows
 step['overlay_metrics']=metrics(list(new_tree.values()))
 states[step['to']]={'tree':list(new_tree.values()),'tests':list(new_tests.values())}
 write_ledger(OUT/f"tracked-tree-{step['to'][:3]}.jsonl.gz",states[step['to']]['tree'])
 write_ledger(OUT/f"test-files-{step['to'][:3]}.jsonl.gz",states[step['to']]['tests'])
 changes.append(step)

summary={'method':'Per-file overlay over the immutable 3a whole-tree and test-file ledgers. Only changed Git blob objects were read. No unchanged path was rehashed.','base_commit':BASE,'intermediate_commit':MID,'frozen_input_commit':HEAD,'steps':changes,'states':{commit:{'tree':metrics(state['tree']),'test_files':metrics(state['tests'])} for commit,state in states.items()},'limits':['The base 3a ledgers remain immutable and are not overwritten.','The 1d6 tree is 3a plus the exact one-file 31f delta and exact two-file 1d6 delta.','The 1d6 delta changes existing test paths only; path classification and language remain unchanged.','The current source input is 1d6. The integration checkout may contain plan and evidence files outside that product source tree.']}
(OUT/'source-overlay-manifest.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2,sort_keys=True)+'\n',encoding='utf-8')
print(json.dumps(summary['states'],sort_keys=True))
