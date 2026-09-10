# Terminal verification plan: independent evidence

Planning only; current `holla-fable` worktree, inspected 2026-09-10. No product
code, capture baseline, dependency or terminal configuration changed. The probes
below used existing binaries/interpreters and synthetic text. Prior fixes in
[the verification report](tui-audit-verification.md) remain closed regressions.

## Findings and boundaries

| ID | Classification | Evidence | Impact |
| --- | --- | --- | --- |
| TV1 | P2 confirmed verification defect | `tools/ansi2png.py:36` assigns width per scalar; `:78` draws scalars separately. Widths differ from the application for combining accents, coffee and a ZWJ emoji. | PNG text/cursor/selection positions can disagree with correct application cells. |
| TV2 | P2 confirmed verification defect | `tools/ansi2png.py:12` selects one host-installed font; `:29` falls back only if font loading fails. On this host, 東, 京 and ☕ have exactly the missing-glyph bitmap. | A successful PNG can silently lose real characters. Matching cell width does not establish glyph fidelity. |
| TV3 | P2 confirmed verification defect | `tools/ansi2html.py:141/147` pads using scalar count; `tools/ansi2png.py:78–87` never clips at `cols`. Synthetic probes below reproduce both. | HTML has a third width policy; PNG overflow enters the decorative margin. |
| TV4 | P1 confirmed lifecycle defect for Unix external suspension | An isolated controlling PTY stopped the actual showcase with SIGTSTP and returned foreground ownership to its shell-like supervisor. ICANON and ECHO remained off. | The foreground shell inherits unusable terminal modes while the application is suspended. Resume followed by normal quit restores the original modes. |
| TV5 | P2 explicit proof gap, not a proven emulator defect | `tools/capture.sh:41/46/47` fixes TERM/color capabilities; `:78` saves tmux's interpreted screen. Current tests do not vary actual emulator implementations. | Palette matrices and tmux snapshots cannot establish a general terminal compatibility claim. |

TV1–TV3 concern the evidence renderer, not proof that the application renders
the same defects in a real terminal. TV4 is an actual process/TTY observation.
Missing kitty width, synchronized-update or clipboard integrations are optional
capability questions, not additional confirmed compatibility defects.

## Reproducible Unicode discrepancies

The application's reference is `src/ui/text.rs:20–21`,
`UnicodeWidthStr::width`, with grapheme iteration in `:34–35`. The pinned
`unicode-width` build and the Python rasterizer produced:

| Text | Application cells | PNG heuristic cells | Consequence |
| --- | ---: | ---: | --- |
| `e\u0301` (decomposed é) | 1 | 2 | Following text shifts right one cell; accent drawn independently. |
| `☕` | 2 | 1 | Following text shifts left one cell. |
| `👩‍💻` | 2 | 5 | Following text shifts right three cells; ZWJ sequence not shaped together. |
| `🇺🇸` | 2 | 2 | Width agreement only; does not prove flag shaping. |
| `東京` | 4 | 4 | Width agreement, but both glyphs are missing on the selected font. |

Executed with the existing Pillow interpreter, without installing dependencies:

```sh
rtk proxy /tmp/holla-venv/bin/python -B -c '
import sys
sys.path.insert(0, "tools")
import ansi2png as p
samples = ["e\u0301", "☕", "👩‍💻", "🇺🇸", "東京"]
print([(s, sum(p.wcwidth(c) for c in s)) for s in samples])
font = p.load("regular")
print(font.path)
missing = font.getmask("\U0010ffff")
print([(s, font.getmask(s).size,
        bytes(font.getmask(s)) == bytes(missing)) for s in ["東", "京", "☕", "A"]])
p.render("Ae\u0301Z", 8, 1, "/tmp/plan-terminal-combining.png")
p.render("ABCD", 2, 1, "/tmp/plan-terminal-overflow.png")
'
```

The selected file was
`/Users/donbeave/Library/Fonts/JetBrainsMonoNerdFontMono-Regular.ttf`.
東/京/☕ masks were each `(9, 11)` and byte-identical to U+10FFFF's missing
glyph; ASCII A differed. This identifies this exact font/host outcome, not every
JetBrains Mono version. Both resulting PNGs were inspected. The two-column
`ABCD` image visibly includes C and part of D in its right margin.

Rust comparison was a temporary executable linked against the existing
`target/debug/deps/libunicode_width-4aa9b272389b25b0.rlib`; its entire computation
was `UnicodeWidthStr::width(text)` over the same five literals. The hash is
build-specific; locate the current artifact with
`rg --files target/debug/deps -g 'libunicode_width*.rlib'` when repeating.
This comparison needs no new crate or application changes.

HTML reproduction:

```sh
rtk proxy /tmp/holla-venv/bin/python -B -c '
import sys
sys.path.insert(0, "tools")
from ansi2html import convert
print(convert("東京", 4, 1))
'
```

Output adds two padding spaces after 東京, although those characters already
occupy four application cells. Browser fallback fonts may further change visual
advance. HTML is therefore not an independent exact-cell oracle.

Additional explicit representation limits: PNG cell metrics are hardcoded to
9×20 at `tools/ansi2png.py:20`, and its cursor is an opaque white one-cell block
at `:88–92`. Capture metadata records only x/y/visibility at
`tools/capture.sh:79`. These are known representations, not faithful proof of
the emulator's chosen font metrics, cursor shape or glyph under the cursor.

## Suspend/resume probe

Source ownership: `TerminalSession::enter`, `src/runtime.rs:88`, enables raw
mode and installs normal/drop/panic restoration. `restore_terminal` at `:122`
releases paste, mouse, cursor, wrap, alternate-screen and raw modes. No
SIGTSTP/SIGCONT transition is wired into this owner. Module documentation at
`:4` says every exit path; `run` at `:131–132` enumerates normal quit, I/O error
and panic. Suspension is not an exit, and neither wording establishes signal
recovery. Existing limitations already acknowledge missing job-control tests
at `docs/tui-audit-verification.md:171–173`.

Executed diagnostic topology, entirely separate from the user's terminal:

1. Open a fresh PTY, set 80×24, retain its full original termios.
2. Fork a session-owning shell-like supervisor; attach only the new PTY with
   `setsid`/`login_tty`. Ignore SIGTTOU in the supervisor only.
3. Fork the actual `target/debug/showcase --page inputs` into its own process
   group. Give it PTY foreground ownership before exec; use normal SIGTSTP and
   SIGTTOU dispositions. Keeping its parent in the same session avoids the
   orphan-process-group exception to job-control stopping.
4. Wait for raw mode, send external SIGTSTP to that exact child, and confirm
   `waitpid(..., WUNTRACED)` reports a stopped process. Return foreground to
   the supervisor, then inspect slave termios from the outer diagnostic.
5. Return foreground to the application, send SIGCONT, then keyboard `q`.
   Drain output, reap both owned processes, compare full original termios.
   The successful run touched only those diagnostic children and the fresh PTY.

Observed result:

```json
{
  "running": {"canonical": false, "echo": false, "original": false},
  "stop_report": "stopped=True",
  "suspended": {"canonical": false, "echo": false, "original": false},
  "exit_report": "exit=0",
  "restored": {"canonical": true, "echo": true, "original": true}
}
```

This proves external SIGTSTP leaves raw modes active during suspension. It does
not establish a Ctrl+Z binding: raw mode disables the usual terminal-generated
job-control signal. Nor does it claim SIGKILL/SIGSTOP can be intercepted. The
probe measured termios, not a full emulator-mode restoration trace or redraw
quality after resume. Those remain acceptance work.

The temporary diagnostic source is `/tmp/plan-terminal-suspend.py` on the audit
host; `rtk proxy python3 -B /tmp/plan-terminal-suspend.py` repeated the result.
It is evidence for this observation, not a retained regression harness: failure
cleanup and startup-handshake handling need hardening before test-suite use.
Independent review identified startup PID parsing outside `try`, blocking
`waitpid` calls without a total deadline, missing failure-path reap, and cleanup
signals after successful reap (a theoretical PID-reuse hazard). A retained
harness must track live ownership, remove reaped PIDs from cleanup, guard startup,
and bound all paths. These limitations do not alter the observed successful
stop/resume/quit result; they prohibit calling the temporary probe robust.

## Structural direction and acceptance

### One capture cell model; explicit glyph capability

Do not extend the hand-maintained scalar range table. Establish a versioned
grapheme/cell representation for evidence, with application cell widths as the
reference and the observed terminal width policy recorded separately. Reuse
the application's text contract or export reference cells; do not introduce
another unverified language-specific width algorithm. PNG and HTML should
consume that representation, preserve style at cluster boundaries and clip to
declared cell dimensions. Font selection, coverage and shaping are a separate
boundary: report missing clusters, use an explicit tested fallback strategy,
and record exact font files/versions and rasterizer metrics. An unsupported
glyph must not silently count as reviewed visual evidence.

Acceptance:

- Cell fixtures cover decomposed accents, CJK, emoji ZWJ/skin-tone/flag sequences,
  VS15/VS16, tabs, mixed SGR, styled cluster boundaries and right-edge truncation.
  Assert downstream cell coordinates and cursor/selection spans, not text alone.
- PNG and HTML agree with reference cells, preserve printable graphemes, and
  never draw into padding. Wide-cluster clipping has a declared contract.
- Exact font coverage is checked, with a clear failure/unsupported record for
  missing glyphs. Shape whole clusters; merely installing a larger font does
  not repair scalar positioning. Preserve current ASCII/style output unless a
  reviewed correction explains the difference.
- Deterministic artifacts include application revision/dirty identity, fixture,
  Unicode/segmentation versions, dimensions, palette versus actual NO_COLOR,
  terminal/tmux versions, font identities and renderer settings. Baseline changes
  require source and rendered review; this plan approves none.

### Terminal lifecycle owns suspend/resume

Extend the existing session owner with explicit suspend/resume transitions,
not independent mode-reset snippets. Before a supported job-control stop,
restore terminal ownership; after continuation, reacquire modes and redraw
from current dimensions. Integrate signal notification with the event loop
using a signal-safe boundary; do not allocate, lock or run terminal I/O inside
an unsafe signal handler. Define repeated, interrupted and partially failed
transitions. Windows behavior and uncatchable signals are separate contracts.

Acceptance: retain the isolated PTY topology above as an automated test; assert
original termios while the shell owns foreground, expected raw state after
resume, and original state after normal/error/panic exits. Include repeated
suspension, resize while stopped, redraw and mouse/paste/wrap/alternate-screen
mode traces. Check normal input and existing startup rollback regressions.
Document precisely which catchable signals are supported; decide separately
whether an application exposes Ctrl+Z. No test should stop the user's process
group or depend on a user's terminal settings.

### Separate four proof layers

| Layer | Establishes | Does not establish |
| --- | --- | --- |
| App/TestBackend cells | Widget state, layout, symbols/modifiers, cursor/copy model | Backend command ordering or emulator font/width |
| Actual backend in fresh process | Emitted ANSI; e.g. real NO_COLOR modifier survival | Emulator interpretation, font shaping or job-control ownership |
| Isolated PTY/tmux | Real input/process behavior and tmux-interpreted screen | Every emulator's rendering, original byte-stream timing, direct-terminal font |
| Named real emulator, direct and through tmux | That recorded emulator/version/font/capability path | Unmeasured terminals or universal compatibility |

`tools/capture.sh:41/46/47` forces a truecolor-capable xterm/tmux environment;
`:78` uses `capture-pane -e -p -N`, not an emulator screenshot or raw output
recording. Current final snapshots cannot prove absence of intermediate frame
tearing. Select and name supported emulator configurations before claiming a
matrix. Exercise key modifiers, bracketed paste, mouse/drag, resize, minimum
size, Unicode edges, actual NO_COLOR and teardown in direct and tmux sessions.
Record failures independently from the synthetic rasterizer's limitations.

## Optional protocols: evidence before feature scope

- **Kitty text sizing/explicit width:** the protocol addresses disagreement by
  allowing a client to supply grapheme width. Width-only support is distinct
  from scaling; support is queried through cursor-position observations. It
  does not imply universal support or make current PNG widths correct. Test a
  named terminal discrepancy first; any later integration needs bounded
  negotiation, unsupported fallback, tmux behavior and input-response routing.
  A newer Unicode database alone cannot guarantee emulator agreement.
  [Primary specification](https://sw.kovidgoyal.net/kitty/text-sizing-protocol/#fixing-the-character-width-issue-for-the-terminal-ecosystem),
  [capability detection](https://sw.kovidgoyal.net/kitty/text-sizing-protocol/#detect-text-sizing).
- **Synchronized output:** final-frame captures do not measure partial-frame
  presentation. Inspect the pinned backend's emitted stream and measure a real
  repaint before proposing support. Absence of an application-level wrapper is
  not proof the dependency never emits it, nor proof of visible flicker.
- **Clipboard:** `ViewportEvent::Copy(String)` at
  `src/widgets/viewport.rs:83–84` deliberately hands transport to the owner.
  `src/bin/showcase/pages/diff.rs:2/120–122` explicitly retains demo copy data;
  it does not promise system clipboard access. OSC52 is therefore an optional
  owner integration, not a broken existing contract. The retrieved kitty
  clipboard document primarily specifies **OSC5522**, an extension, and must
  not be cited as proof that legacy OSC52 has identical negotiation or status
  semantics. Any integration needs explicit copy intent, transport limits,
  denied/unsupported behavior and tmux policy; clipboard reads need a separate
  permission/security design. [Primary clipboard specification](https://sw.kovidgoyal.net/kitty/clipboard/).

No optional protocol, dependency installation, golden regeneration or product
implementation is authorized by this document. The demonstrated obligations
are evidence fidelity and safe suspend lifecycle; broader compatibility needs
named configurations and measured proof, not protocol checklists.
