# TASK-038 trusted obligations

This file is part of the immutable task package, not candidate-writable configuration. It binds R-001/AC-001/CHK-004 and R-002/AC-002/CHK-006. Source rows are copied without changing their action or assertion content from the showcase scenario register. Every copied field below is normative except its historical capture-status description, which is not a parity verdict.

## Observable outcome

Restore confirm/rename/three-choice/destructive dialogs, validation, initial focus, background capture and result history; quick grouped Files/Tasks picker, scopes/alternate choose/no-match; tab picker close-until-last and all six enabled levels; Chrome menu/context menu, disabled/gap rules, status chips and brand actions.

## Architecture and non-regression

Use shared Dialog, Picker, Menu/MenuBar, StatusBar and KeyHint with runtime-owned layer placement/capture/focus restoration. Preserve distinct actual outside-click rules; do not force every popup into one invented modal policy. No fake disabled picker rows.

All task-owned, prerequisite and previously closed scenarios must pass. Execute the complete required inventory without fail-fast and account for every actual result. Only exact unfinished future-owner failures in the immutable stage map may remain; missing execution, new failures, unexpected errors, changed classification or reopening a closed scenario fail. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture checks must pass. A diagnostic failure is never relabelled a parity pass.

## Trust and expansion

The oracle commit is 02f5294bfdbf38004cc49130d0aff1d01f31434c; main architecture starts at 7b27732a8c3c131760ec3438f641cb3c11343a42. Accepted prerequisite receipts, taskfmt 52d9f1eb7721f409bc47beb9fced7997b5c13ede and the independently qualified proof executable are host-owned. Materialized actions, numeric oracle coordinates, scenario/checkpoint membership, palettes, clocks, fonts, profiles, semantic mappings and exact expected cells are sealed before this task starts. Candidate code cannot read expected artifacts or write the host catalog, receipts, refs, comparator or another run. The dispatcher/bootstrap is produced by TASK-001, runners by TASK-070, accounting by TASK-071 and architecture probes by TASK-072. These commands are prerequisite deliverables, not evidence that they already execute.

The host supplies the immutable full application expansion grammar and proof contract alongside this file. Expand all recorded sizes, colors, focus stops, targets, ticks, source-qualified test bodies and checkpoint boundaries exactly as the accepted baseline did. Preserve every original assertion in ROUTE seeds. Source-only helpers and SCAN/SWEEP membership are materialized before this task; the executor never expands against its own implementation. Capture full row-major schema-3 cells, blank/wide continuation cells, modifiers, cursor and semantic transitions after every listed checkpoint. Preview/state-only or exact-clock lanes remain honest; executable reachability and terminal lifecycle require an actual process and owned PTY. Both capture operations must produce an explicit complete result, including accepted non-applicability rather than omitted output.

Cross-cutting full-app rows may be closure-owned; earlier tasks still preserve the subset already closed in the host ledger. Final closure replays the union on one exact tree and requires zero application-unresolved identities. Future-owned diagnostic failures are reported as failures, never success.

## Exact scenario contracts

### APP:SC-BASE-dialogs

- **page:** dialogs
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Four launcher kinds; Nothing yet results
- **source_refs:** O:pages/dialogs.rs:102
- **components:** Dialog,Button,Panel

### APP:SC-BASE-pickers

- **page:** pickers
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Three launcher variants and result placeholder
- **source_refs:** O:pages/pickers.rs:314
- **components:** Picker,Button,Props,Panel

### APP:SC-BASE-chrome

- **page:** chrome
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Brand/menu/session/status composition and hint priority
- **source_refs:** O:pages/chrome.rs:191
- **components:** Brand,MenuBar,Menu,List,StatusBar,KeyHints

### APP:SC-DIALOG-CONFIRM

- **page:** dialogs
- **sizes:** 80x24,120x40,160x50
- **action_checkpoints:** fresh mouse click Confirm run from navigation;Tab;BackTab;Esc;assert navigation restored;focus(Confirm run);Enter;Esc;assert trigger restored;reopen;y;reopen;n;click Delete branch;Enter;reopen;Right;Enter
- **required_observable_proof:** Confirm primary focus; destructive cancel focus; quick answers; result history; close restores saved prior focus: navigation after fresh mouse launch, trigger after focused keyboard launch
- **source_refs:** O:pages/dialogs.rs:49,155;O:app_tests.rs:454
- **components:** Dialog,Button,focus

### APP:SC-DIALOG-PROMPT

- **page:** dialogs
- **sizes:** 80x24,120x40
- **action_checkpoints:** click Rename task;Enter;Ctrl+U;Enter;assert empty rejection and editing false;Enter;type(x repeated41);Enter;assert max40 rejection and editing false;Enter;Ctrl+U;type(Review schema);Enter;reopen;Enter;paste(café 東京);Esc;assert input reverted and modal still open;Esc;assert dialog canceled and saved name unchanged
- **required_observable_proof:** Empty and41-byte ASCII names blocked with exact errors; explicit re-entry reaches each state; rename result returned; cursor/paste while editing; first Esc reverts edit, second Esc cancels dialog without changing saved name
- **source_refs:** O:pages/dialogs.rs:25,49;O:app_tests.rs:480
- **components:** Dialog,TextInput,validation

### APP:SC-DIALOG-CHOICES

- **page:** dialogs
- **sizes:** 120x40
- **action_checkpoints:** Fresh each action: click Three choices; choose Cancel/Discard/Save by arrow then Enter; fresh click each action; attempt click outside;wheel background
- **required_observable_proof:** Save initial focus; all three result variants; background barrier and history order
- **source_refs:** O:pages/dialogs.rs:66,155
- **components:** Dialog,overlay

### APP:SC-PICKER-QUICK

- **page:** pickers
- **sizes:** 80x24,120x40,160x50
- **action_checkpoints:** click Open quickly;type(api);Down;Alt+Enter;reopen;Tab;Tab;Tab;type(zzzz-no-result);Enter;Ctrl+U;PageDown;wheel(picker,+3);Esc
- **required_observable_proof:** Grouped fuzzy results/matched spans; alt new-tab status; All/Files/Tasks scope cycle; empty Enter keeps open; query clear
- **source_refs:** O:pages/pickers.rs:132,241,381;O:src/widgets/picker.rs:272
- **components:** Picker,text-edit,overlay

### APP:SC-PICKER-TABS-LEVEL

- **page:** pickers
- **sizes:** 80x24,120x40,160x50
- **action_checkpoints:** click Switch tab;Delete;Delete;Delete;Delete;Enter;click Choose a level;Up;Down;Enter;reopen;hover/down/up each of six levels in fresh runs;Esc
- **required_observable_proof:** Close tab secondary action until last cannot close; selected result; nonsearchable level initial3; all six options enabled; width112 clamp
- **source_refs:** O:pages/pickers.rs:105,132,241
- **components:** Picker,selection

### APP:SC-CHROME-MENUS

- **page:** chrome
- **sizes:** 80x24,120x40,160x50
- **action_checkpoints:** focus(sessions);F10;Right;Down;Enter;focus(sessions);m;Down;Enter;right-click session row;hover disabled menu item;click outside;click brand;click usage;click PR
- **required_observable_proof:** MenuBar vs context ownership; focused session; disabled/gaps inert; menu hint layer; status chips/brand messages
- **source_refs:** O:pages/chrome.rs:115,135,250
- **components:** MenuBar,Menu,StatusBar,KeyHints,List

## Correction boundary

If exact parity requires a shared-component fix, unsupported public extension, changed source authority or new trusted fixture, stop NEEDS_REPLAN and name the responsible owner. Do not duplicate mechanics inside this app or edit expected evidence. A completed task produces host-verified results and an accepted tree receipt, not a candidate-written approval.
