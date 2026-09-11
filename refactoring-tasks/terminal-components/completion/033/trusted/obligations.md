# TASK-033 trusted obligations

## Deferred fixture changes are not oracle parity

ADJ-09 and immutable F16/F18 require exact source fixed/clipped Inputs, Buttons and TextAreas composition, including controls with no usable geometry at narrow sizes. Do not add compact section switching, reveal scrolling or hidden focus stops to fulfill the historical deferred proposal. The TextAreas fixture retains its original "Read-only transcript" label and disabled, dim, inert, unfocusable configuration. Generic shared read-only navigation/copy capability remains separately tested; it is not permission to substitute a different app fixture. Preserve real source drafts, resizing, focus and full frames at each required size; final full-app closure belongs TASK-039.

## Re-audit shared API consumer contract

ADJ-10: source-qualified Forms radio navigation commits through RadioGroupAction::Navigated or Form::radio_navigation_commits(true), not direct access to private state or a second navigation engine. This is binding under R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005. Direct and full application traces must prove the actual source composition; an API declaration or compile-only adaptation does not close parity.

This file is part of the immutable task package, not candidate-writable configuration. It binds R-001/AC-001/CHK-004 and R-002/AC-002/CHK-006. Source rows are copied without changing their action or assertion content from the showcase scenario register. Every copied field below is normative except its historical capture-status description, which is not a parity verdict.

## Observable outcome

Restore all five enabled Inputs fields and the disabled token, required/invalid email, search and masked-key editing; all TextArea selection, newline, scroll and disabled/error states; full Form reviewer, reset, validation and committed submit lifecycle; and the complete Buttons action/state matrix. Busy button completes at ≥2200ms only on eligible page tick; Form stays busy at exactly1800ms and completes strictly after it. Keep disabled/inert matrix samples noninteractive.

## Architecture and non-regression

Controlled Field/TextInput/TextArea/Form/Button state must paint and handle the same visible controls. Remove legacy_field_gutter/help and inert enabled reference fields. Place the complete Buttons matrix and its production-app assertions under Showcase; retain a separate external public-consumer example without treating it as this matrix. Library render-target consolidation remains TASK-066, not permission to alter its tests here.

All task-owned, prerequisite and previously closed scenarios must pass. Execute the complete required inventory without fail-fast and account for every actual result. Only exact unfinished future-owner failures in the immutable stage map may remain; missing execution, new failures, unexpected errors, changed classification or reopening a closed scenario fail. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture checks must pass. A diagnostic failure is never relabelled a parity pass.

## Trust and expansion

The oracle commit is 02f5294bfdbf38004cc49130d0aff1d01f31434c; main architecture starts at 7b27732a8c3c131760ec3438f641cb3c11343a42. Accepted prerequisite receipts, taskfmt 52d9f1eb7721f409bc47beb9fced7997b5c13ede and the independently qualified proof executable are host-owned. Materialized actions, numeric oracle coordinates, scenario/checkpoint membership, palettes, clocks, fonts, profiles, semantic mappings and exact expected cells are sealed before this task starts. Candidate code cannot read expected artifacts or write the host catalog, receipts, refs, comparator or another run. The dispatcher/bootstrap is produced by TASK-001, runners by TASK-070, accounting by TASK-071 and architecture probes by TASK-072. These commands are prerequisite deliverables, not evidence that they already execute.

The host supplies the immutable full application expansion grammar and proof contract alongside this file. Expand all recorded sizes, colors, focus stops, targets, ticks, source-qualified test bodies and checkpoint boundaries exactly as the accepted baseline did. Preserve every original assertion in ROUTE seeds. Source-only helpers and SCAN/SWEEP membership are materialized before this task; the executor never expands against its own implementation. Capture full row-major schema-3 cells, blank/wide continuation cells, modifiers, cursor and semantic transitions after every listed checkpoint. Preview/state-only or exact-clock lanes remain honest; executable reachability and terminal lifecycle require an actual process and owned PTY. Both capture operations must produce an explicit complete result, including accepted non-applicability rather than omitted output.

Cross-cutting full-app rows may be closure-owned; earlier tasks still preserve the subset already closed in the host ledger. Final closure replays the union on one exact tree and requires zero application-unresolved identities. Future-owned diagnostic failures are reported as failures, never success.

## Exact scenario contracts

### APP:SC-BASE-buttons

- **page:** buttons
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Four button kinds; toggle on/off; disabled; inert state matrix
- **source_refs:** O:pages/buttons.rs:70
- **components:** Button,Panel

### APP:SC-BASE-inputs

- **page:** inputs
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Six fields; error/placeholder/disabled/masked values; inert states
- **source_refs:** O:pages/inputs.rs:121
- **components:** TextInput,Field,Panel

### APP:SC-BASE-textareas

- **page:** textareas
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Long/empty/disabled/error text areas; edge fades
- **source_refs:** O:pages/textareas.rs:62
- **components:** TextArea,Panel,scrollbar

### APP:SC-BASE-forms

- **page:** forms
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** fresh draw
- **required_observable_proof:** Full form and accessible controls at each viewport
- **source_refs:** O:pages/forms.rs:129
- **components:** Field,TextInput,TextArea,Checkbox,RadioGroup,Toggle,Button

### APP:SC-BUTTON-ACTIONS

- **page:** buttons
- **sizes:** 80x24,120x40
- **action_checkpoints:** Fresh each Run task/Preview/Cancel/Delete branch: focus;Enter; fresh Space; fresh click; Auto-approve click twice;Verbose Enter twice
- **required_observable_proof:** Clicks count/status/local last; kind and toggle states identical; each activation exactly once
- **source_refs:** O:pages/buttons.rs:42,186
- **components:** Button,Status

### APP:SC-BUTTON-DISABLED

- **page:** buttons
- **sizes:** 120x40
- **action_checkpoints:** hover/down/up Disabled primary;hover/down/up Disabled;activate Start long job;attempt click again while busy
- **required_observable_proof:** No disabled activation/focus; busy nonactivatable; inert matrix never creates stops
- **source_refs:** O:pages/buttons.rs:22,186;O:app_tests.rs:221
- **components:** Button,focus

### APP:SC-BUTTON-TIME

- **page:** buttons
- **sizes:** 120x40
- **action_checkpoints:** click(Start long job) t0;1000 ordinary events at t0;1000 draws at t0;time2199ms;time2200ms+Tick;Tick;fresh start then ?;time2300ms;Tick;assert job still busy while modal;Esc;Tick;fresh start then];time2300ms;Tick;[;assert job still busy before eligible Tick;Tick
- **required_observable_proof:** Working and completion statuses; local last preserved; completion exactly once only eligible page tick; hidden/modal ownership
- **source_refs:** O:pages/buttons.rs:42,186;M:apps/showcase/tests/elapsed_contract.rs:45
- **components:** clock,Button,runtime

### APP:SC-INPUT-EDIT

- **page:** inputs
- **sizes:** 80x24,120x40,160x50
- **action_checkpoints:** Fresh each Project name/Branch/Search files: focus;Enter;Ctrl+U;type(café 東京);Home;Shift+Right;Ctrl+Right;Backspace;paste(x\ny);Tab;BackTab;Enter;type(z);Esc
- **required_observable_proof:** Actual drafts/commit/revert; grapheme cursor; selection; paste sanitation; Tab traversal; saved/Reverted footer
- **source_refs:** O:pages/inputs.rs:210;O:src/widgets/input.rs:190
- **components:** TextInput,text-edit,Field

### APP:SC-INPUT-EMAIL-MASK

- **page:** inputs
- **sizes:** 120x40
- **action_checkpoints:** focus(Owner email);Enter;Ctrl+U;Enter;assert Required and editing false;Enter;type(bad);Enter;assert invalid email and editing false;Enter;Ctrl+U;type(mira@example.test);Enter;assert valid committed email;focus(API key);Enter;End;type(秘密);Enter;fresh click Search files
- **required_observable_proof:** Required vs invalid vs valid with explicit edit re-entry; five enabled fields reachable; masked edit vs last-four reveal; one click edits
- **source_refs:** O:pages/inputs.rs:13,28,210
- **components:** TextInput,validation,masking

### APP:SC-TEXTAREA-EDIT

- **page:** textareas
- **sizes:** 80x24,100x30,120x40,160x50
- **action_checkpoints:** focus(Notes);Enter;type(café 東京);Enter;paste(second\nthird);Home;Shift+End;Backspace;Esc; at80x24 assert Commit message and Read-only transcript content controls clipped with no focus/hits; at100x30/120x40/160x50 focus(Commit message);Enter;type(!);Tab;hover/down/up(Read-only transcript)
- **required_observable_proof:** Enter inserts newline; Esc commits; multiline selection/cursor; exact narrow clipped second-card frame and absent controls; source-visible error/help and disabled sample inert at taller sizes
- **source_refs:** O:pages/textareas.rs:20,80;O:src/widgets/textarea.rs:100
- **components:** TextArea,text-edit

### APP:SC-TEXTAREA-SCROLL

- **page:** textareas
- **sizes:** 80x24,100x30,120x40
- **action_checkpoints:** focus(Task description);End;Home;PageDown;wheel(Task description,+3);Enter;End;Home;Down*40;drag(scrollbar,thumb,bottom)
- **required_observable_proof:** First/last/wrapped lines; vertical/horizontal cursor reveal; edge fade and scroll clamp
- **source_refs:** O:app_tests.rs:388;O:pages/textareas.rs:80
- **components:** TextArea,scrollbar,fade

### APP:SC-FORM-VALIDATE

- **page:** forms
- **sizes:** 80x24,120x40
- **action_checkpoints:** Ctrl+S;focus(Task name);Enter;type(abc);Enter;Ctrl+S;Enter;Ctrl+U;type(Build project);Enter;focus(Reviewer);Enter;type(bad);Enter;Ctrl+S;Enter;Ctrl+U;type(mira@example.test);Enter
- **required_observable_proof:** First-error focus; Required/min4/email validation; typed valid values remain live
- **source_refs:** O:pages/forms.rs:16,25,84,92
- **components:** Form,Field,validation

### APP:SC-FORM-SUBMIT

- **page:** forms
- **sizes:** 80x24,120x40
- **action_checkpoints:** Set Task name=Build project;Description=Line1 newline Line2;Mode Down+Space;toggle Run tests/Open PR/Auto-approve;click disabled Notify;Ctrl+S t0;time1800ms+Tick;time1800ms+1ns+Tick;click Reset
- **required_observable_proof:** Exact selected values; busy remains at1800; done after1800; reset restores full defaults; disabled skip
- **source_refs:** O:pages/forms.rs:57,92,229
- **components:** Form,TextArea,RadioGroup,Checkbox,Toggle,clock

## Correction boundary

If exact parity requires a shared-component fix, unsupported public extension, changed source authority or new trusted fixture, stop NEEDS_REPLAN and name the responsible owner. Do not duplicate mechanics inside this app or edit expected evidence. A completed task produces host-verified results and an accepted tree receipt, not a candidate-written approval.
