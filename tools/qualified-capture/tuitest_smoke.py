import os, json, subprocess, uuid, sys
from pathlib import Path
p=Path(sys.argv[1]).resolve()
binary=Path(sys.argv[2]).resolve()
env=os.environ.copy(); env.pop('NO_COLOR',None)
name='qualification-'+uuid.uuid4().hex[:12]
records=[]
def run(args, expected=0):
    cmd=[str(binary),'--session',name,'--json',*args]
    r=subprocess.run(cmd,env=env,cwd=p,capture_output=True,text=True,timeout=20)
    records.append(dict(argv=cmd,exit=r.returncode,stdout=r.stdout,stderr=r.stderr))
    (p/'tuitest-commands.json').write_text(json.dumps(records,indent=2))
    if expected is not None and r.returncode != expected: raise RuntimeError(str(args)+r.stdout+r.stderr)
    return r.stdout
try:
    run(['run','--cols','32','--rows','12','--no-wait-ready','python3',str(p/'fixture.py'),str(p/'tuitest-events.json')])
    run(['expect','text','READY','--timeout','5000'])
    run(['wait','idle','--timeout','5000'])
    (p/'tuitest-cells.json').write_text(run(['cells','0','0','32','12']))
    (p/'tuitest-cursor.json').write_text(run(['get','cursor']))
    (p/'tuitest-state.json').write_text(run(['state']))
    run(['screenshot','-o',str(p/'tuitest.svg')])
    run(['mouse','click','4','3'])
    run(['write','\x1b[200~PASTE\n二\x1b[201~'])
    run(['mouse','drag','2','2','5','4'])
    run(['mouse','scroll','down','--amount','2'])
    run(['resize','36','14'])
    run(['type','s'])
    run(['expect','text','T','--timeout','5000'])
    run(['wait','idle','--timeout','5000'])
    (p/'tuitest-settled-cells.json').write_text(run(['cells','0','0','36','14']))
    # Negative assertion must fail, proving a missing/mutated expected glyph is detected.
    run(['expect','text','IMPOSSIBLE_MUTATION','--timeout','100'],expected=1)
    run(['type','q']);run(['wait','exit','--timeout','5000']);run(['expect','exit-code','0'],expected=1); (p/'tuitest-exit-state.json').write_text(run(['state']))
finally:
    run(['close'],expected=None)
if json.loads((p/'tuitest-exit-state.json').read_text())['data']['exited'] != 0:
    raise RuntimeError('fixture did not exit successfully')
print('completed',len(records),'commands',name)
