# TASK-051 trusted obligations

EARLY-054 Primary J6 owner: compose path field, keyed list, mode checkbox and confirm through shared Form/List/Dialog; FsEntry and simulated filesystem remain in Jackin. Complete JA-015 retains navigation, invalid/edit/reset/paste path, wheel and cancel observations. No host filesystem access or second focus/painter engine. Bind R-001/AC-001/CHK-004 and R-002/AC-002/CHK-006; Holla Files evidence is not the Jackin J6 witness.

## Re-audit shared API consumer contract

ADJ-10: Prelude ChoiceDialog at oracle screens/prelude.rs:164 uses the shared typed navigation or configured Form bridge, preserving admitted boundary navigation commits and controlled props in both phases. No application-local cursor engine or private Form state access. This binds R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005; full source trajectories and exact key/value/cursor/callback observations remain required.

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/jackin_preview/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-004/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore manager topology and route-specific shell, ritual entry/outro phases and pending-workspace prelude. Match tree/detail selection and independent scrolling, context-sensitive row actions, launch picker cancellation, browser validation/rewind, host menus/help and last-row hints.

Use public Tree, split/detail, Brand, Buttons, hints, menus and runtime layers with one production interaction/paint owner. Keep Jackin rain as legitimate application art and keep discovery, instance identity, entry arbitration and pending-workspace policy in Jackin.

- Remove size/world/paused-frame manager compatibility rendering as a production route, not just its name; source evidence is main app.rs:4977,5189,6397 and oracle screens/manager.rs:1778.
- Keep pending prelude changes unpersisted until the later Editor save flow; source screens/prelude.rs and app_tests.rs:416,463 governs rewind and validation.
- Preserve the exact rain seed, phase constants and elapsed-caption policy. JA-062/070 use modeled/controlled-clock observations where applicable; do not call those exact PTY phases.

## Exact membership and staging

Primary complete scenario IDs: JA-002, JA-003, JA-004, JA-005, JA-006, JA-010, JA-014, JA-015, JA-070.

Legacy preservation edges protect previously closed work only. New-behavior acceptance additionally requires every exact flow contribution; no empty preservation intersection is sufficient.

The baseline expansion algorithms and finite axes in jackin.md and jackin-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Resolve all eight worlds, Full/Reduced/Paused, fixed seed 0x4A41434B494E5E5E and epoch 1788401640; the ordinary CLI cannot be credited with modeled-only state observations. JA-065 uses last reachable tick before/at expiry plus separate precise interaction-clock proof. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires its complete local primary scenarios; no additional earlier-owner flow identities are assigned. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

### JA-002

- world: first-use
- sizes: 80x24,100x30,120x40,160x50
- actions: variant frame=0,45,P1_LEN-1,P1_LEN,P1_LEN+P2_LEN,KNOCK_START,KNOCK_START+1,WARP_START,WARP_START+1,INTRO_END-1; fresh(first-use,Paused,frame)
- checkpoints: Typed text, glitch glyphs, caption, warp and final phase exact; fixed seed
- components: brand,scheduler,clipping
- source: rain.rs:475-643;app_tests.rs:139

### JA-003

- world: first-use
- sizes: 80x24,120x40
- actions: fresh(first-use,Full,0); K(Enter); T(45); K(Enter,Enter); reset; replay(first_use_plays_intro_then_manager_and_no_replay_when_returning)
- checkpoints: Guard prevents stale skip; phase skip then manager; returning never replays ritual
- components: runtime,input,scheduler
- source: app.rs:505;app_tests.rs:115

### JA-004

- world: first-use
- sizes: 80x24,120x40
- actions: fresh(first-use,Reduced,0); T(3); K(Enter); reset; fresh(first-use,Full,0); K(Ctrl-C)
- checkpoints: Reduced static boundary and manager; Ctrl-C releases entry claim and quits
- components: runtime,brand
- source: app_tests.rs:139;app.rs:505

### JA-005

- world: returning
- sizes: 80x24,100x30,120x40,160x50
- actions: K(Home,Right,Down,Tab,Esc); K(*,-,End,Home); variant K(j,k,l,h,g,G,PageDown,PageUp)
- checkpoints: Workspace/instance/session tree selection, expand/collapse, detail focus, top/middle/bottom and scroll limits
- components: tree,list-detail,focus,scrollbar
- source: screens/manager.rs:1889;app_tests.rs:158

### JA-006

- world: returning
- sizes: 80x24,100x30,120x40,160x50
- actions: click(workspace payments-platform); double-click(workspace payments-platform); click(instance 7f3a); wheel(tree,3); wheel(detail,3); drag(manager seam,4 cells right); drag(manager seam,4 cells left)
- checkpoints: Pointer hitboxes, action activation, scroll routing, split proportions and constraints
- components: tree,splitter,hit registry
- source: screens/manager.rs:on_click,on_double_click,on_drag,on_wheel

### JA-010

- world: hard-cases
- sizes: 80x24,100x30,120x40,160x50
- actions: fresh(hard-cases,Reduced,0); K(F5); T(60); K(End,PageUp,Home); wheel(tree,100); wheel(tree,-100)
- checkpoints: Discovery failure keeps last-good rows; long names, many rows, missing daemon, scroll edge clipping
- components: tree,status,scrollbar
- source: screens/manager.rs:1770;domain/fixtures.rs

### JA-014

- world: returning
- sizes: 80x24,120x40
- actions: replay(prelude_refuses_a_duplicate_name_and_cancels_cleanly); variant empty name,duplicate name; Esc until Manager
- checkpoints: Validation text and disabled action; no abandoned workspace or focus leak
- components: prompt,dialog,focus
- source: app_tests.rs:463;screens/prelude.rs

### JA-015

- world: returning
- sizes: 80x24,100x30,120x40,160x50
- actions: K(n); browser K(Down,Right,Left,Backspace,Home,End); edit path; type(/missing); commit; reset path; paste(/work/payments-platform); wheel(browser,100); Esc
- checkpoints: Folder open/parent/select vs navigate, invalid path, filtered/empty list, cursor/paste, no host filesystem read
- components: browser,text input,list,scrollbar
- source: screens/modals.rs:280-390

### JA-070

- world: arbiter fixtures
- sizes: 80x24,120x40
- actions: variant foreign entry claim,unknown discovery,missing entry time; construct oracle state then request entry/exit; compare production rendered route/status/caption
- checkpoints: No duplicate intro/outro, discovery fails closed, missing time omits elapsed; unchanged simulated running-instance count
- components: arbiter,status,brand
- source: arbiter.rs:149-204;app.rs:201

## Regression and trust closure

Run every required source-qualified Jackin and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
