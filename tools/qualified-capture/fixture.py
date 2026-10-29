import os, sys, tty, termios, signal, select, time, json
from pathlib import Path
log = Path(sys.argv[1])
events = []
def record(kind, **data):
    events.append(dict(kind=kind, **data)); log.write_text(json.dumps(events, indent=2))
def emit(s): os.write(1, s.encode())
def size(*args):
    v = os.get_terminal_size(0); record('resize', cols=v.columns, rows=v.lines)
signal.signal(signal.SIGWINCH, size)
old = termios.tcgetattr(0); tty.setraw(0)
try:
    size(); record('env', NO_COLOR=os.environ.get('NO_COLOR'), TERM=os.environ.get('TERM'), COLORTERM=os.environ.get('COLORTERM'))
    emit('\x1b[2J\x1b[H\x1b[?7l\x1b[?1003h\x1b[?1006h\x1b[?2004hREADY')
    emit('\x1b[2;1H\x1b[1mB\x1b[0m \x1b[2mD\x1b[0m \x1b[7mR\x1b[0m \x1b[1;2;7mX\x1b[0m \x1b[38;2;10;20;30;48;2;40;50;60mQ\x1b[0m')
    emit('\x1b[3;1H\x1b[38;5;196;48;5;22mP\x1b[0m')
    emit('\x1b[4;1H\x1b[91;44mA\x1b[0m')
    emit('\x1b[5;1H\x1b[1;4mM\x1b[0m')
    emit('\x1b[6;1H\x1b[38;2;1;2;3;48;2;4;5;6m界e\u0301\x1b[0m')
    emit('\x1b[7;32HABC')
    emit('\x1b[8;1H\x1b[8mH\x1b[0m \x1b[5mK\x1b[0m \x1b[9mS\x1b[0m')
    emit('\x1b[9;1H\x1b[38;2;255;255;255;48;2;0;0;0;2mF\x1b[0m')
    emit('\x1b[?25h\x1b[6 q\x1b[10;5H')
    while True:
        ready, _, _ = select.select([0], [], [], 5)
        if not ready: continue
        data = os.read(0, 4096)
        if not data: break
        record('input', hex=data.hex())
        if data.endswith(b'q'): break
        if data.endswith(b's'):
            emit('\x1b[10;1H\x1b[31mT'); time.sleep(.1)
            emit('\x1b[10;1H\x1b[32mT'); time.sleep(.1)
            emit('\x1b[10;1H\x1b[34mT\x1b[0m')
            continue
        emit('\x1b[12;1HACK'+str(len(events))+'\x1b[10;5H')
finally:
    emit('\x1b[?1003l\x1b[?1006l\x1b[?2004l\x1b[0m')
    termios.tcsetattr(0, termios.TCSANOW, old)
    record('restored', value=termios.tcgetattr(0)==old, before=repr(old), after=repr(termios.tcgetattr(0)))
