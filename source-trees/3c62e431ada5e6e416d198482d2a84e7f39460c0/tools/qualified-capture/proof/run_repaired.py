"""Rebuild the pinned repair and rerun the original fixture oracle, separately."""
import hashlib,json,os,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent;E=P.parent;R=E/'tool-sources/tui-snap-repaired';Q=P/'qualification'
env=os.environ.copy();env.pop('NO_COLOR',None);env['CARGO_TARGET_DIR']=str(P/'target')
records=[]
def run(argv,expected=0):
    r=subprocess.run(['rtk','proxy',*argv],cwd=R,env=env,capture_output=True,text=True,timeout=900)
    path=P/f'exact-{len(records)}.log';path.write_text(r.stdout+r.stderr)
    records.append(dict(argv=argv,exit=r.returncode,expected=expected,log=path.name));(P/'exact-commands.json').write_text(json.dumps(records,indent=2))
    assert r.returncode==expected,r.stdout+r.stderr
    return r.stdout
commit=run(['git','rev-parse','HEAD']).strip()
assert commit=='e45d3fae2ecc628e294c0b5796a8775ad2d7f0e2',commit
# Same fixture bytes, independently authored before the repair.
assert (Q/'fixture.py').read_bytes()==(E/'tool-qualification/fixture.py').read_bytes()
run(['cargo','+stable','build','--locked','--bin','tuisnap'])
run(['cargo','+stable','run','--locked','--manifest-path',str(Q/'snap-harness/Cargo.toml'),'--bin','tool-qualification','--',str(Q)])
run([str(P/'target/debug/tuisnap'),'render','--input',str(Q/'snap.frame.json'),'--format','png','--format','svg','--out',str(Q/'snap-repaired')])
run(['python3',str(Q/'qualify.py'),'--strict-snap'])
run(['cargo','+stable','test','--locked','--all-targets','--all-features'])
run(['cargo','+stable','test','--locked','--no-default-features'])
run(['cargo','+stable','test','--locked','--doc','--all-features'])
run(['cargo','+stable','fmt','--all','--','--check'])
run(['cargo','+stable','clippy','--locked','--all-targets','--all-features','--','-D','warnings'])
metadata=dict(commit=commit,source_status=run(['git','status','--porcelain']),commands=records,artifacts={})
for f in list(Q.glob('*'))+[P/'target/debug/tuisnap',P/'target/debug/tool-qualification']:
    if f.is_file():metadata['artifacts'][str(f.relative_to(P))]=hashlib.sha256(f.read_bytes()).hexdigest()
(P/'exact-provenance.json').write_text(json.dumps(metadata,indent=2))
print('Exact repair source, binary, and sentinels verified; reviewed visual approvals included.')
