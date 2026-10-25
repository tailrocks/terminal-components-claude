#!/usr/bin/env python3
"""Describe the two 31f-to-1d6 source changes without running test code."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

REPO='/Users/donbeave/Downloads/terminal-components-claude'
OUT=Path(REPO+'/docs/implementation/pr38-final/results/t00-source')
BASE='3a2c977c881cd1745de16425c982791ee3ef2cc3'
OLD='31f856f43d20920ea99aa81b8f40adeeb5bff358'
NEW='1d6ee92adf6ccca38eeee64b1371cba1497e8fcd'
FIXTURE='crates/showcase-ui/tests/fixtures/holla-page-headings.tsv'
TEST='crates/showcase-ui/tests/navigation_contract.rs'

def show(revision,path):
 return subprocess.check_output(['git','-C',REPO,'show',revision+':'+path])

def digest(data): return hashlib.sha256(data).hexdigest()
old_fixture=show(OLD,FIXTURE); new_fixture=show(NEW,FIXTURE)
old_test=show(OLD,TEST).decode('utf-8'); new_test=show(NEW,TEST).decode('utf-8')
test_re=re.compile(r'(?m)^\s*#\s*\[\s*test\s*\]\s*\n(?:\s*#\[[^\n]*\]\s*\n)*\s*fn\s+([A-Za-z_]\w*)\s*\(')
old_names=test_re.findall(old_test); new_names=test_re.findall(new_test)
def test_declarations(source):
 lines=source.splitlines(); out=[]
 for index,line in enumerate(lines):
  if not re.match(r'^\s*#\s*\[\s*test\s*\]\s*$',line): continue
  cursor=index+1
  while cursor<len(lines) and (not lines[cursor].strip() or re.match(r'^\s*#\[[^\n]*\]\s*$',lines[cursor])): cursor+=1
  match=re.match(r'^\s*(?:async\s+)?fn\s+([A-Za-z_]\w*)\s*\(',lines[cursor]) if cursor<len(lines) else None
  out.append({'attribute_line':index+1,'function':match.group(1) if match else None,'function_line':cursor+1 if match else None,'status':'explicit Rust test attribute' if match else 'attribute target unresolved'})
 return out
declarations=test_declarations(new_test)
old_rows=old_fixture.decode('utf-8').splitlines(); new_rows=new_fixture.decode('utf-8').splitlines()
def key(line): return tuple(line.split('\t')[:4])
old_by={key(x):x for x in old_rows}; new_by={key(x):x for x in new_rows}
changed=[]
for row_key in sorted(set(old_by)|set(new_by)):
 if old_by.get(row_key)!=new_by.get(row_key): changed.append({'key':list(row_key),'old':old_by.get(row_key),'new':new_by.get(row_key)})
record={
 'input_commit':NEW,'base_commit':OLD,'changed_paths':[FIXTURE,TEST],
 'changed_file_evidence':[
  {'path':FIXTURE,'old_sha256':digest(old_fixture),'new_sha256':digest(new_fixture),'old_size_bytes':len(old_fixture),'new_size_bytes':len(new_fixture),'row_count_old':len(old_rows),'row_count_new':len(new_rows),'changed_rows':changed,'classification':'test fixture with expected visual text; changed expectation is parity evidence only'},
  {'path':TEST,'old_sha256':digest(show(OLD,TEST)),'new_sha256':digest(show(NEW,TEST)),'old_explicit_test_function_count':len(old_names),'new_explicit_test_function_count':len(new_names),'added_test_functions':sorted(set(new_names)-set(old_names)),'removed_test_functions':sorted(set(old_names)-set(new_names)),'classification':'Rust integration test source; source inventory only, not executed'}
 ],
 'finding_disposition':{'F01':'The latest commit changes a fixture expectation and adds a navigation assertion. The latest 1d6 CI run is complete/cancelled with a full 100-job REST inventory; 95 jobs are cancelled and only the Required aggregate failed. The changed expected bytes are unaccepted parity evidence, not a green runtime result.','F13':'The 31f change is unrelated to coverage data. The moved coverage files remain byte-identical; six BehaviorProof rows remain historical inventory, not runtime acceptance.','F20':'The latest commit does not change current product documentation or the TablePro comparator.'},
 'execution':{'tests_run':False,'runtime_behavior':'NOT VERIFIED','reason':'T00 is read-only source capture. No test plan authorizes test execution.'},
 'preservation':'Exact old and new file bytes are stored under source-deltas/31f-to-1d6/old and source-deltas/31f-to-1d6/new.'
}
(OUT/'test-inventory-delta-1d6.json').write_text(json.dumps(record,ensure_ascii=False,indent=2,sort_keys=True)+'\n',encoding='utf-8')
overlay={
 'source_commit':NEW,'base_inventory_commit':BASE,'incremental_base_commit':OLD,
 'whole_tree_ledger':'tracked-tree-1d6.jsonl.gz','test_related_ledger':'test-files-1d6.jsonl.gz',
 'changed_declarations':[{'path':TEST,'sha256':digest(show(NEW,TEST)),'test_target':'crates/showcase-ui/tests/navigation_contract.rs','explicit_test_function_count':len(declarations),'declarations':declarations,'added_declarations':[x for x in declarations if x['function'] in set(new_names)-set(old_names)],'imports_or_module_edges_changed':False,'classification':'existing Cargo integration-test target; one source assertion added; no runtime execution'}],
 'changed_fixture':[{'path':FIXTURE,'sha256':digest(new_fixture),'line_count':len(new_rows),'changed_row':changed,'classification':'updated expected display fixture; retain as unaccepted parity evidence'}],
 'test_file_path_count':93917,'test_target_inherited_inventory':'test-inventory.json test_target_candidates plus tracked-tree-1d6 and test-files-1d6 overlays',
 'test_execution':{'approved_test_plan_id':'N/A','authorized':False,'selected_cases':0,'runtime_behavior':'NOT VERIFIED'},
 'limits':['This overlay derives only the two changed file contents from exact 1d6 Git blobs.','All other declarations inherit the complete 3a inventory. The 1d6 source overlay changes only the listed paths after the one-file 31f source delta.','The fixture edit is not accepted as a green result; the latest cancelled run is in ci-head-1d6-rest.json.']
}
(OUT/'declaration-overlay-1d6.json').write_text(json.dumps(overlay,ensure_ascii=False,indent=2,sort_keys=True)+'\n',encoding='utf-8')
print(json.dumps({'fixture_changed_rows':len(changed),'old_tests':len(old_names),'new_tests':len(new_names),'added_tests':record['changed_file_evidence'][1]['added_test_functions']},ensure_ascii=False,sort_keys=True))
