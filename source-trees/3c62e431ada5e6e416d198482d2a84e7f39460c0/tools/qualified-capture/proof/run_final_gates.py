import os,subprocess,json,time
from pathlib import Path
P=Path(__file__).resolve().parent;R=P.parent/'tool-sources/tui-snap-repaired';env=os.environ.copy();env.pop('NO_COLOR',None);env['CARGO_TARGET_DIR']=str(P/'target')
assert subprocess.check_output(['rtk','proxy','git','rev-parse','HEAD'],cwd=R,text=True).strip()=='e45d3fae2ecc628e294c0b5796a8775ad2d7f0e2'
commands=[['cargo','+stable','test','--locked','--all-targets','--all-features'],['cargo','+stable','test','--locked','--no-default-features'],['cargo','+stable','test','--locked','--doc','--all-features'],['cargo','+stable','build','--locked','--all-targets','--all-features'],['cargo','+stable','fmt','--all','--','--check'],['cargo','+stable','clippy','--locked','--all-targets','--all-features','--','-D','warnings']]
records=[]
(P/"full-gates.json").write_text("[]\n")
(P/"matrix-status.json").write_text(json.dumps(dict(commit="e45d3fae2ecc628e294c0b5796a8775ad2d7f0e2",status="running",required_gate_count=len(commands)),indent=2))
for i,c in enumerate(commands):
    path=P/f'full-{i}.log';started=time.monotonic()
    with path.open('w') as log:
        child=subprocess.Popen(['rtk','proxy',*c],cwd=R,env=env,stdout=log,stderr=subprocess.STDOUT)
        print('started',i,child.pid,flush=True)
        code=child.wait(timeout=900)
    records.append(dict(argv=c,exit=code,seconds=time.monotonic()-started,log=path.name))
    (P/'full-gates.json').write_text(json.dumps(records,indent=2));print('completed',i,code,flush=True)
    if code:raise SystemExit(code)

(P/"matrix-status.json").write_text(json.dumps(dict(commit="e45d3fae2ecc628e294c0b5796a8775ad2d7f0e2",status="complete",required_gate_count=len(commands),passed=len(records)),indent=2))
