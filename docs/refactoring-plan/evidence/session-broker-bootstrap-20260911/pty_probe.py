"""Own every PTY/process; never manipulate the user's terminal."""
import fcntl
import json
import os
import re
import select
import signal
import struct
import subprocess
import sys
import termios
import time


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def supervisor(binary, mode):
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)
    signal.signal(signal.SIGTTOU, signal.SIG_IGN)
    signal.signal(signal.SIGTTIN, signal.SIG_IGN)
    child = subprocess.Popen([binary, mode], preexec_fn=os.setpgrp)
    os.tcsetpgrp(0, child.pid)
    print(f"\n@@PID {child.pid}", flush=True)
    while True:
        pid, status = os.waitpid(child.pid, os.WUNTRACED)
        require(pid == child.pid, "waitpid returned an unowned process")
        if os.WIFSTOPPED(status):
            os.tcsetpgrp(0, os.getpgrp())
            print(f"\n@@STOPPED {os.WSTOPSIG(status)}", flush=True)
            if sys.stdin.readline().strip() != "fg":
                return 2
            os.tcsetpgrp(0, child.pid)
            os.kill(child.pid, signal.SIGCONT)
        else:
            code = os.waitstatus_to_exitcode(status)
            print(f"\n@@CHILD_EXIT {code}", flush=True)
            return 0 if code == 0 else 3


class Probe:
    def __init__(self, binary, mode):
        self.master, slave = os.openpty()
        fcntl.ioctl(self.master, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 120, 0, 0))
        self.initial = termios.tcgetattr(self.master)
        self.output = bytearray()
        self.pid = None
        self.stops = 0
        self.latencies = []
        self.child = subprocess.Popen(
            [sys.executable, __file__, "supervise", binary, mode],
            stdin=slave, stdout=slave, stderr=slave, start_new_session=True,
            env={**os.environ, "TERM": "xterm-256color"},
        )
        os.close(slave)

    def pump(self, timeout=0.02):
        if select.select([self.master], [], [], timeout)[0]:
            try:
                chunk = os.read(self.master, 65536)
            except OSError:
                chunk = b""
            self.output.extend(chunk)

    def wait(self, predicate, label, bound=10.0):
        start = time.monotonic()
        while not predicate():
            if time.monotonic() - start > bound:
                raise AssertionError(f"timeout: {label}")
            self.pump()

    def marker(self, value, count=1, bound=10.0):
        token = ("@@" + value).encode()
        self.wait(lambda: self.output.count(token) >= count, value, bound)

    def send(self, value):
        os.write(self.master, value)

    def canonical(self):
        current = termios.tcgetattr(self.master)
        require(current == self.initial, f"terminal modes not restored: before={self.initial!r} after={current!r} output={bytes(self.output)!r}")

    def stop(self, bound=10.0):
        self.stops += 1
        before = time.monotonic()
        os.kill(self.pid, signal.SIGTSTP)
        self.marker("STOPPED", self.stops, bound)
        self.latencies.append(round((time.monotonic() - before) * 1000, 3))
        self.canonical()

    def resume(self, cols=None, rows=None):
        if cols is not None:
            fcntl.ioctl(self.master, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
        self.send(b"fg\n")

    def close(self):
        if self.pid is not None and b"@@CHILD_EXIT " not in self.output:
            try:
                os.kill(self.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        if self.child.poll() is None:
            self.child.kill()
        self.child.wait(timeout=5)
        os.close(self.master)


def run(binary, mode):
    probe = Probe(binary, mode)
    try:
        probe.marker("PID ")
        probe.pid = int(re.search(rb"@@PID (\d+)", probe.output)[1])
        probe.marker("AWAIT_START")
        probe.canonical()
        probe.send(b"continue\n")
        probe.marker("FAILED_SETUP_CLEAN")
        probe.canonical()
        # Default stop after failed setup, before the first successful session.
        probe.stop()
        probe.resume()
        probe.send(b"continue\n")
        probe.marker("READY 1")
        require(not termios.tcgetattr(probe.master)[3] & termios.ICANON, "session did not enter raw mode")
        settled = time.monotonic() + 0.35
        before = len(probe.output)
        while time.monotonic() < settled:
            probe.pump()
        require(len(probe.output) == before, "idle service emitted terminal output")
        # No real input is sent to wake the blocked crossterm read.
        probe.stop(bound=0.7 if mode == "block-read" else 10.0)
        probe.resume(100, 30)
        probe.marker("RESUMED 100 30")
        probe.stop()
        probe.resume(80, 24)
        probe.marker("RESUMED 80 24")
        probe.send(b"n")
        probe.marker("INACTIVE 1")
        probe.canonical()
        summary = re.search(rb"@@INACTIVE 1 wakes=(\d+) ticks=(\d+) settle=(\d+)", probe.output)
        require(summary is not None, "missing service summary")
        require(int(summary[1]) >= 2, "service wait not exercised")
        require(summary.groups()[1:] == (b"0", b"0"), "idle service delivered logical work")
        # No unregister: inactive default stop and next activation must both work.
        probe.stop(bound=0.7 if mode == "unregister" else 10.0)
        probe.resume()
        probe.send(b"continue\n")
        probe.marker("READY 2")
        probe.send(b"n")
        probe.marker("INACTIVE 2")
        probe.canonical()
        probe.stop()
        probe.resume()
        probe.send(b"continue\n")
        probe.marker("CHILD_EXIT 0")
        probe.canonical()
        text = bytes(probe.output)
        require(text.count(b"\x1b[?1049h") == text.count(b"\x1b[?1049l") == 5, "alternate-screen entry/exit counts differ")
        require(text.count(b"\x1b[?2004h") == text.count(b"\x1b[?2004l") == 5, "bracketed-paste entry/exit counts differ")
        return {"mode": mode or "positive", "outcome": "pass", "stop_latency_ms": probe.latencies}
    finally:
        probe.close()


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "supervise":
        sys.exit(supervisor(sys.argv[2], sys.argv[3]))
    binary = os.path.abspath(sys.argv[1])
    results = [run(binary, "")]
    for mutant, reason in [
        ("block-read", "timeout: STOPPED"),
        ("dirty-stop", "terminal modes not restored"),
        ("idle-tick", "idle service delivered logical work"),
        ("unregister", "timeout: STOPPED"),
    ]:
        try:
            run(binary, mutant)
        except AssertionError as error:
            require(reason in str(error), f"wrong rejection for {mutant}: {error}")
            results.append({"mode": mutant, "outcome": "rejected", "reason": str(error)})
        else:
            raise AssertionError(f"mutant survived: {mutant}")
    print(json.dumps({"platform": sys.platform, "results": results}, indent=2))
