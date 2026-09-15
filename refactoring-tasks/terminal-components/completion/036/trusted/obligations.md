# TASK-036 trusted obligations

## Fixed ADJ-16 Steps consumer

ADJ-16 is binding for oracle src/bin/showcase/pages/terminal.rs97: use shared Steps::passive(id).presentation(StepsPresentation::Rail) with the existing stable keys, borrowed lifecycle/row/meta data and source default numbering. Preserve custom meta Some(text)/Some(empty)/None, all lifecycle/spinner frames, exact ordinal/meta geometry, scroll/fade and narrow allocation. Passive retains wheel/scrollbar but no row focus/hit/hover/action; do not substitute disabled or existing ClickOnly new. No app-owned ordinal/meta/glyph rail painter or invented activation callback. Domain stage/timer/frontier data stays in this app. Bind R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005 to exact current task-owned component contributions and intact source scenarios under the existing ownership map; no future closure receipt is required. Baseline006 freezes these policy branches independently before candidate implementation.

This file is part of the immutable task package, not candidate-writable configuration. It binds R-001/AC-001/CHK-004 and R-002/AC-002/CHK-006. Source rows are copied without changing their action or assertion content from the showcase scenario register. Every copied field below is normative except its historical capture-status description, which is not a parity verdict.

## Observable outcome

Restore Diff unified/review/empty modes and narrow review fallback, original Unicode press anchor and copy; CodeEditor completion/find/diagnostics/run; prose versus follow-tail logs; independent scroll owners/thumb offsets; terminal seven-stage success/failure rail and split; deterministic progress pause/restart; and Task runner queued/running/max-two/failure/cancel flows. Preserve exact Tick-driven domain sequences, not an elapsed-time approximation.

## Architecture and non-regression

Use shared DiffView, CodeEditor, TextViewport, ScrollRegion, Split, Steps, Progress and runtime layers. No duplicated output selection, scrollbar/fade, diagnostic painter or per-app timer mechanism. Application domain pipeline and styled output content remain application-owned.

All task-owned, prerequisite and previously closed scenarios must pass. Execute the complete required inventory without fail-fast and account for every actual result. Only exact unfinished future-owner failures in the immutable stage map may remain; missing execution, new failures, unexpected errors, changed classification or reopening a closed scenario fail. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture checks must pass. A diagnostic failure is never relabelled a parity pass.

## Trust and expansion

The oracle commit is 02f5294bfdbf38004cc49130d0aff1d01f31434c; main architecture starts at 7b27732a8c3c131760ec3438f641cb3c11343a42. Accepted prerequisite receipts, taskfmt 52d9f1eb7721f409bc47beb9fced7997b5c13ede and the independently qualified proof executable are host-owned. Materialized actions, numeric oracle coordinates, scenario/checkpoint membership, palettes, clocks, fonts, profiles, semantic mappings and exact expected cells are sealed before this task starts. Candidate code cannot read expected artifacts or write the host catalog, receipts, refs, comparator or another run. The dispatcher/bootstrap is produced by TASK-001, runners by TASK-070, accounting by TASK-071 and architecture probes by TASK-072. These commands are prerequisite deliverables, not evidence that they already execute.

The host supplies the immutable full application expansion grammar and proof contract alongside this file. Expand all recorded sizes, colors, focus stops, targets, ticks, source-qualified test bodies and checkpoint boundaries exactly as the accepted baseline did. Preserve every original assertion in ROUTE seeds. Source-only helpers and SCAN/SWEEP membership are materialized before this task; the executor never expands against its own implementation. Capture full row-major schema-3 cells, blank/wide continuation cells, modifiers, cursor and semantic transitions after every listed checkpoint. Preview/state-only or exact-clock lanes remain honest; executable reachability and terminal lifecycle require an actual process and owned PTY. Both capture operations must produce an explicit complete result, including accepted non-applicability rather than omitted output.

Cross-cutting full-app rows may be closure-owned; earlier tasks still preserve the subset already closed in the host ledger. Final closure replays the union on one exact tree and requires zero application-unresolved identities. Future-owned diagnostic failures are reported as failures, never success.

## Exact scenario contracts

### APP:SC-BASE-panels

- **page:** panels
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Titled/nested/framed/scrollable/follow panels
- **source_refs:** O:pages/panels.rs:71
- **components:** Panel,ScrollPanel,List

### APP:SC-BASE-progress

- **page:** progress
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw at tick0
- **required_observable_proof:** Live/static progress variants and labels
- **source_refs:** O:pages/progress.rs:44
- **components:** Progress,Spinner,Button,Panel

### APP:SC-BASE-scrolling

- **page:** scrolling
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw at tick0
- **required_observable_proof:** Wrapped prose/list/log position labels and fades
- **source_refs:** O:pages/scrolling.rs:54
- **components:** ScrollPanel,List,scrollbar,fade

### APP:SC-BASE-terminal

- **page:** terminal
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw at tick0
- **required_observable_proof:** Terminal/step rail split and commands
- **source_refs:** O:pages/terminal.rs:205
- **components:** TerminalViewport,StepRail,SplitPane,Button

### APP:SC-BASE-editor

- **page:** codeeditor
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Syntax/block marks/completion closed; no cursor while navigation focused
- **source_refs:** O:pages/editor.rs:319
- **components:** CodeEditor,Completion,Panel

### APP:SC-BASE-diff

- **page:** diff
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Unified hunks; Unicode lines; controls and chrome
- **source_refs:** O:pages/diff.rs:87
- **components:** DiffView,TerminalViewport,Button,Panel

### APP:SC-BASE-taskrunner

- **page:** taskrunner
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw at tick0
- **required_observable_proof:** Pipeline idle; targets/tree; log; disabled cancel
- **source_refs:** O:pages/taskrunner.rs:215
- **components:** Tree,ScrollPanel,Progress,Button,Dialog

### APP:SC-PANEL-SCROLL

- **page:** panels
- **sizes:** 80x24,100x30,120x40
- **action_checkpoints:** at80x24 assert nested list clipped with no focus/hit; at100x30/120x40 focus(nested list);Down; all sizes focus(framed prose);End;Home;PageDown;wheel(prose,+3);focus(log);End;f;Up;wheel(log,-2);drag(log scrollbar,thumb,bottom)
- **required_observable_proof:** Independent scroll owners; prose never follows; log follow; exact narrow nested-card clipping and absent target; nested focus/gutters at taller sizes
- **source_refs:** O:pages/panels.rs:158
- **components:** Panel,ScrollPanel,List,fade

### APP:SC-PROGRESS-RUN

- **page:** progress
- **sizes:** 80x24,120x40
- **action_checkpoints:** ticks1;click Pause;ticks10;click Resume;ticks82;click Restart;ticks167
- **required_observable_proof:** Live+static state appearance; pause freezes build; restart zero; final build1.0/status; spinner checkpoint phase
- **source_refs:** O:pages/progress.rs:180
- **components:** Progress,Spinner,clock

### APP:SC-SCROLL-OWNERS

- **page:** scrolling
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** focus(prose);End;wheel(prose,+1);Home;wheel(prose,-1);wheel(list,+5);hover(list row);wheel(list,+1);focus(log);End;Up;tick;End;tick
- **required_observable_proof:** Boundary wheels consumed; focus unchanged under unrelated wheel; hover follows new row; prose never follows; log follows only tail
- **source_refs:** O:app_tests.rs:814,878;O:pages/scrolling.rs:113
- **components:** ScrollPanel,List,scrollbar,fade

### APP:SC-SCROLL-THUMB

- **page:** scrolling
- **sizes:** 80x24,120x40
- **action_checkpoints:** wheel(list,+8);Down(thumb center);Drag(thumb center+3rows);Up;click(track bottom);resize100x30;Tick;resize80x24
- **required_observable_proof:** Thumb retains grab offset; track page/drag; labels match first frame; row counts and edge fades exact
- **source_refs:** O:app_tests.rs:736,777
- **components:** scrollbar,layout

### APP:SC-TERMINAL-RUN

- **page:** terminal
- **sizes:** 80x24,120x40,160x50
- **action_checkpoints:** click Run;ticks10;ticks26;ticks40;ticks18;ticks14;ticks1;click Run with a failure;ticks10;ticks13
- **required_observable_proof:** Seven-stage rail; cached/skipped stage; final prompt; failed network stage and blocked successors
- **source_refs:** O:pages/terminal.rs:34,139
- **components:** TerminalViewport,StepRail,Spinner

### APP:SC-TERMINAL-SELECT

- **page:** terminal
- **sizes:** 80x24,120x40
- **action_checkpoints:** ticks40;focus(terminal);Home;PageDown;End;Up;f;wheel(terminal,-3);Down(text cell);Drag(later line);Up;y;Esc;Shift+Right;Shift+Down;y;drag(split seam,left,right)
- **required_observable_proof:** Follow state; copy text/line count; original press anchor; cleared selection; keyboard selection; clamped split and no focus theft
- **source_refs:** O:pages/terminal.rs:271;O:src/widgets/viewport.rs:1352;O:app_tests.rs:982
- **components:** TerminalViewport,selection,SplitPane

### APP:SC-EDITOR-EDIT

- **page:** codeeditor
- **sizes:** 80x24,120x40,160x50
- **action_checkpoints:** focus(code editor);Enter;Home;paste(café 東京);Shift+Right;Backspace;Tab;BackTab;Esc;};{;/;type(fetch);Enter;n;N;Esc
- **required_observable_proof:** Unicode editing/selection and cursor; Tab behavior; block movement; find and next/previous match
- **source_refs:** O:pages/editor.rs:438;O:src/widgets/code.rs:285,406
- **components:** CodeEditor,text-edit,search

### APP:SC-EDITOR-COMPLETE

- **page:** codeeditor
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(code editor);Ctrl+Space;Down;PageDown;PageUp;Enter;Ctrl+Space;Esc;fresh Ctrl+Space;click completion row;wheel(editor,+3);wheelH(editor,+3);drag(editor scrollbar,top,bottom)
- **required_observable_proof:** Popup anchor/selection/accept replacement; dismissal; completion receives keys first; scroll owner
- **source_refs:** O:pages/editor.rs:217,254,438;O:src/widgets/completion.rs:99
- **components:** Completion,CodeEditor,overlay

### APP:SC-EDITOR-RUN

- **page:** codeeditor
- **sizes:** 120x40
- **action_checkpoints:** focus(code editor);Ctrl+R;ticks9;ticks1;move to blank between blocks;Ctrl+R;edit current block with .unwrap() and todo!;Esc;Ctrl+R;ticks10;edit one char
- **required_observable_proof:** Running highlight; first run77ms; no-block message; warning/error diagnostic spans; editing clears diagnostics
- **source_refs:** O:pages/editor.rs:269,279,438
- **components:** CodeEditor,diagnostics,clock

### APP:SC-DIFF-MODES

- **page:** diff
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** click Review;focus(diff viewport);Down;Right;click Empty;click Empty;click Review
- **required_observable_proof:** Review split vs narrow unified fallback; exact line numbers/glyphs/colors; empty preserves focus stops; mode switch clears selection/copied state
- **source_refs:** O:pages/diff.rs:62,103,210
- **components:** DiffView,TerminalViewport,Button

### APP:SC-DIFF-SELECT

- **page:** diff
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(diff viewport);End;Home;PageDown;wheel(diff,+2);Down(東京 line);Drag(café line);Up;y;Esc;click Review;repeat Unicode drag;y;drag(scrollbar,top,bottom)
- **required_observable_proof:** Copy Unicode payload and selection; release must preserve original press anchor; narrow/review coordinate mapping; fades/panning
- **source_refs:** O:pages/diff.rs:103,234,300
- **components:** DiffView,TerminalViewport,selection

### APP:SC-TASK-RUN

- **page:** taskrunner
- **sizes:** 80x24,120x40,160x50
- **action_checkpoints:** r;ticks1;ticks9;ticks1000;click Run pipeline;ticks10;click Cancel;ticks1000;assert pipeline task states unchanged while modal;Esc;click Cancel;ticks1000;y
- **required_observable_proof:** Queued/running max2; log steps; integration failure; done badge/status; rerun reset; cancel dialog freezes tick; finished results survive cancellation
- **source_refs:** O:pages/taskrunner.rs:114,137,365
- **components:** Tree,Progress,ScrollPanel,Dialog,clock

### APP:SC-TASK-SCROLL

- **page:** taskrunner
- **sizes:** 80x24,120x40
- **action_checkpoints:** focus(target tree);Left;Right;End;Home;r;focus(Run pipeline);Enter;ticks20;focus(log);Home;wheel(log,+2);f;ticks10;End;drag(log scrollbar,thumb,top);assert target-tree scrollbar absent;resize72x20;drag(tree scrollbar,top,bottom);resize original size
- **required_observable_proof:** Context keys: r ignored while tree/log focused as oracle; log follow/paused; tree fold; independent scrollbar/focus; tree fits original80x24/120x40 but overflows72x20; exact resize clamp and retained state
- **source_refs:** O:pages/taskrunner.rs:365,479
- **components:** Tree,ScrollPanel,scrollbar

## Correction boundary

If exact parity requires a shared-component fix, unsupported public extension, changed source authority or new trusted fixture, stop NEEDS_REPLAN and name the responsible owner. Do not duplicate mechanics inside this app or edit expected evidence. A completed task produces host-verified results and an accepted tree receipt, not a candidate-written approval.
