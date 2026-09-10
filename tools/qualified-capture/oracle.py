"""Independent fixture oracle; no candidate or captured output seeds expectations."""
import ast, copy, json, re, sys, termios
from pathlib import Path
P=Path(sys.argv[1]).resolve()
def require(condition, detail='qualification requirement failed'):
    if not condition: raise RuntimeError(detail)
MODS=['bold','dim','reverse','underline','italic','strikethrough','hidden','blink']
def blank(x,y):
    return dict(x=x,y=y,symbol=' ',width=1,continuation=False,fg='default',bg='default',**{m:False for m in MODS})
expected=[blank(x,y) for y in range(12) for x in range(32)]
def setcell(x,y,symbol,**kw): expected[y*32+x].update(symbol=symbol,**kw)
for x,s in enumerate('READY'):setcell(x,0,s)
setcell(0,1,'B',bold=True);setcell(2,1,'D',dim=True);setcell(4,1,'R',reverse=True)
setcell(6,1,'X',bold=True,dim=True,reverse=True)
setcell(8,1,'Q',fg='#0a141e',bg='#28323c')
setcell(0,2,'P',fg=196,bg=22);setcell(0,3,'A',fg=9,bg=4)
setcell(0,4,'M',bold=True,underline=True)
setcell(0,5,'界',width=2,fg='#010203',bg='#040506')
setcell(1,5,'',width=0,continuation=True,fg='#010203',bg='#040506')
setcell(2,5,'e\u0301',fg='#010203',bg='#040506')
setcell(31,6,'C');setcell(0,7,'H',hidden=True);setcell(2,7,'K',blink=True);setcell(4,7,'S',strikethrough=True)
setcell(0,8,'F',dim=True,fg='#ffffff',bg='#000000')
# Encode serialization differences only, never remove semantic differences.
def color(c):
    if c=='Default':return 'default'
    if isinstance(c,dict):
        if 'Indexed' in c:return c['Indexed']
        if 'Rgb' in c:return '#%02x%02x%02x'%tuple(c['Rgb'][k] for k in ['r','g','b'])
    return c
def snap_cells(data):
    return [dict(x=c['x'],y=c['y'],symbol=c['symbol'],width=c['width'],continuation=c['continuation'],fg=color(c['fg']),bg=color(c['bg']),**{m:c['mods'][m] for m in MODS}) for c in data['cells']]
def test_cells(data):
    return [dict(x=c['x'],y=c['y'],symbol=c['char'],fg=c['fg'],bg=c['bg'],bold=c['bold'],dim=c['dim'],reverse=c['inverse'],underline=c['underline'],italic=c['italic'],strikethrough=c['strike'],hidden=c['invisible'],blink=c['blink']) for c in data['data']['cells']]
def compare(actual,oracle):
    diffs=[]
    if len(actual)!=len(oracle):diffs.append(dict(kind='cell-count',expected=len(oracle),actual=len(actual)))
    for a,e in zip(actual,oracle):
        for key in e:
            if key in a and a[key]!=e[key]:diffs.append(dict(x=e['x'],y=e['y'],field=key,expected=e[key],actual=a[key]))
    return diffs
snap=json.loads((P/'snap.frame.json').read_text());test=json.loads((P/'tuitest-cells.json').read_text())
actual={'tuisnap':snap_cells(snap),'tui-test':test_cells(test)}
report={}
for name,cells in actual.items():
    unsupported=sorted(set(expected[0])-set(cells[0]))
    diffs=compare(cells,expected)
    mutation_results=[]
    for field,value in [('symbol','!'),('bold',False),('dim',False),('reverse',False),('x',1)]:
        index={'bold':32,'dim':34,'reverse':36}.get(field,0)
        mutated=copy.deepcopy(cells);mutated[index][field]=value
        detected=any(d not in diffs for d in compare(mutated,expected))
        require(detected,(name,field))
        mutation_results.append(dict(field=field,detected=True))
    events=json.loads((P/('snap-events.json' if name=='tuisnap' else 'tuitest-events.json')).read_text())
    inputs=b''.join(bytes.fromhex(e['hex']) for e in events if e['kind']=='input')
    m=re.search(b'\x1b\\[200~(.*?)\x1b\\[201~',inputs,re.S)
    before=ast.literal_eval(events[-1]['before']);after=ast.literal_eval(events[-1]['after'])
    report[name]=dict(cell_differences=diffs,unsupported_fields=unsupported,mutations=mutation_results,
      mouse_zero_based= b'\x1b[<0;5;4M\x1b[<0;5;4m' in inputs,
      drag_endpoints= b'\x1b[<0;3;3M' in inputs and b'\x1b[<0;6;5m' in inputs,
      paste_actual_hex=m[1].hex() if m else None,paste_requested_hex='PASTE\n二'.encode().hex(),
      resized=any(e['kind']=='resize' and (e['cols'],e['rows'])==(36,14) for e in events),
      terminal_echo_canonical_restored=before[3]&(termios.ECHO|termios.ICANON)==after[3]&(termios.ECHO|termios.ICANON),
      terminal_lflag_delta=before[3]^after[3],terminal_lflag_delta_name='PENDIN' if before[3]^after[3]==termios.PENDIN else 'other',
      environment=next(e for e in events if e['kind']=='env'))
report['tuisnap']['cursor']=dict(expected=dict(x=4,y=9,visible=True,style='Bar',blinking=False),actual=snap['cursor'])
report['tui-test']['cursor']=dict(expected=dict(x=4,y=9),actual=json.loads((P/'tuitest-cursor.json').read_text())['data']['value'],unsupported=['visible','style','blinking'])
report['tuisnap']['dim_rgb_resolution']=dict(expected=[153,153,153],actual=list(map(int,re.search(r'actual=Rgb \{ r: (\d+), g: (\d+), b: (\d+) \}',(P/'snap-dim-resolution.txt').read_text()).groups())),evidence='snap-dim-resolution.txt')
for name,file,convert in [('tuisnap','snap-settled.frame.json',snap_cells),('tui-test','tuitest-settled-cells.json',test_cells)]:
    cells=convert(json.loads((P/file).read_text()));c=next(c for c in cells if (c['x'],c['y'])==(0,9))
    report[name]['settled_style']=dict(expected=dict(symbol='T',fg=4),actual={k:c[k] for k in ['symbol','fg']})
require(expected == json.loads((Path(__file__).resolve().parent/'expected-cells.json').read_text()))
(P/'qualification-results.json').write_text(json.dumps(report,indent=2,ensure_ascii=False))
for n,r in report.items():print(n,'cell diffs:',r['cell_differences'],'unsupported:',r['unsupported_fields'],'mutation tests:',len(r['mutations']))
print('Raw findings retained; explicit limits checked below.')

# A qualification report is not a pass: strict consumers reject every known loss.
if "--strict" in sys.argv:
    if any(r["cell_differences"] or r["unsupported_fields"] for r in report.values()):
        raise SystemExit(1)

if "--strict-snap" in sys.argv:
    r=report['tuisnap']
    require(not r['cell_differences'] and not r['unsupported_fields'])
    require(r['cursor']['expected']==r['cursor']['actual'])
    require(r['dim_rgb_resolution']['expected']==r['dim_rgb_resolution']['actual'])
    require(r['paste_actual_hex']==r['paste_requested_hex'])
    require(r['mouse_zero_based'] and r['drag_endpoints'] and r['resized'] and r['terminal_echo_canonical_restored'])
    require(r['settled_style']['expected']==r['settled_style']['actual'])
    print('Repaired tui-snap strict sentinels PASS; tui-test limitations remain independent.')

# The complementary engine's known losses are exact, never generalized away.
r=report['tui-test']
require(r['unsupported_fields'] == ['continuation', 'width'])
require(r['cell_differences'] == [dict(x=2,y=7,field='blink',expected=True,actual=False)])
require(r['cursor']['expected'] == r['cursor']['actual'])
require(r['paste_actual_hex'] == r['paste_requested_hex'])
require(r['mouse_zero_based'] and r['drag_endpoints'] and r['resized'])
require(r['terminal_echo_canonical_restored'])
require(r['settled_style']['expected'] == r['settled_style']['actual'])
print('Independent sentinels PASS within explicitly enumerated tui-test limits.')
