# TASK-056 trusted obligations

EARLY-054 Consume J6's shared browser through intact JA-059 dirty-exit simulated export/cancel/exit trajectory. Preserve typed result/cancellation and source in-memory output only; no actual export or host filesystem access. Bind R-001/AC-001/CHK-004 and R-002/AC-002/CHK-006; Holla Files evidence is not the Jackin J6 witness.

## Re-audit shared API consumer contract

ADJ-10: dirty-exit ChoiceDialog in oracle screens/capsule.rs:1093 uses the shared typed navigation or configured Form bridge, preserving admitted boundary navigation commits and controlled props in both phases. This is the complete JA-059 dirty-exit owner, not a Capsule split control. No application-local cursor engine or private Form state access. This binds R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005; full source trajectories and exact key/value/cursor/callback observations remain required.

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/jackin_preview/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-004/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore Compact and Advanced Inspect with file identity retained across mode changes, drill-in Diff, narrow stacking and independent scroll; restore every dirty-exit inspect/export/keep/purge branch and nested cancellation/focus restoration.

Compose public Tree, DiffView, Split, browser and Dialog/runtime layers. Keep changesets, export policy, running-instance state and dirty-exit arbitration in Jackin; generic diff/input/layer behavior remains in shared components.

- Export uses the existing simulated World outcome and produces no real file export, provider or daemon effect.
- Compact list/drill-in and Advanced tree/diff are distinct action-reached layouts; F2/m/d, selected-file restoration and captured scrollbar drags must work, not be painted placeholders.
- Final exit/outro entry is checked with preceding Capsule ownership; cancellation leaves topology and pending policy unchanged according to the oracle.
- F09 remains deferred: compact-open and Advanced Diff handlers discard DiffView's copy event (oracle inspect.rs:250,343). JA-060 selects nonempty diff text and presses y in both modes; World.clipboard, clipboard_gen and the retained modal/selection must match the unchanged oracle state. Do not implement the historical proposed preview-clipboard delivery or introduce a CustomModal effect API for it. This does not suppress copy events in reusable DiffView or alter other application copy owners.

## Exact membership and staging

Primary complete scenario IDs: JA-009, JA-056, JA-058, JA-059, JA-060, JA-061.

All primary scenarios require every checkpoint. Dependency or scenario routing cannot be relaxed by the executor. A closed checkpoint is monotonic even while another checkpoint in its scenario remains future-owned.

The baseline expansion algorithms and finite axes in jackin.md and jackin-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Resolve all eight worlds, Full/Reduced/Paused, fixed seed 0x4A41434B494E5E5E and epoch 1788401640; the ordinary CLI cannot be credited with modeled-only state observations. JA-065 uses last reachable tick before/at expiry plus separate precise interaction-clock proof. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires its complete local primary scenarios; no additional earlier-owner flow identities are assigned. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

### JA-009

- world: returning
- sizes: 80x24,120x40
- actions: variant select(instance); K(n),K(a),K(x),K(i),K(t,Esc),K(p,Esc),K(r); separate fresh confirmed stop/purge variants
- checkpoints: Session picker, shell, inspect, stop/purge confirmation, reconnect/restore; status-specific enabled actions
- components: picker,dialog,inspect
- source: screens/manager.rs:1829-1882

### JA-056

- world: capsule-multi
- sizes: 80x24,120x40
- actions: variant K(Ctrl-B,x),K(Ctrl-B,&); cancel then confirm on fresh world; close last pane/tab in one-instance fixture
- checkpoints: Pane vs tab confirmation and content; topology shrinks coherently; final pane triggers correct lifecycle
- components: dialog,split panes,tabs,arbiter
- source: screens/capsule.rs:1050,1258;sim/pty.rs:1525

### JA-058

- world: capsule-multi
- sizes: 80x24,120x40
- actions: replay(detach_reconnect_and_final_exit_plays_one_outro); replay(still_inside_feedback_when_other_instances_remain)
- checkpoints: Detach retains daemon tabs/panes; reconnect selection; nonfinal exit stays manager; final exit gets exactly one outro
- components: arbiter,manager,panes,dialog
- source: app_tests.rs:219,251

### JA-059

- world: outro-last
- sizes: 80x24,100x30,120x40,160x50
- actions: K(Ctrl-Q); variant inspect changes then back,export then cancel,export simulated path then exit,keep instance,discard/purge confirmation; each fresh dirty policy fixture
- checkpoints: Dirty exit branch, export result/status, cancellation, retained/deleted in-memory state, elapsed outro; no actual export
- components: dialog,browser,inspect,arbiter
- source: screens/capsule.rs:1019,1080,2204

### JA-060

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(inspect_changes_opens_from_the_view_menu_in_both_modes); checkpoint(Capsule-after-source-replay); K(F10,Right,Right,End,Enter); checkpoint(Inspect-compact-reopened); compact K(Down,Enter,PageDown,PageUp,Esc); K(m); advanced Tab,Down,Tab; F2; d; Esc; separate fresh capsule-multi at each listed size for each compact-open and advanced-Diff variant: K(F10,Right,Right,End,Enter,Enter); advanced only K(m,Tab); K(Home,Ctrl+Shift+Home); checkpoint(selected_diff_copy); K(y); checkpoint(copy_event_discarded)
- checkpoints: Compact list/drill-in and advanced tree/diff, selected file persists, diff display toggle, narrow stacked layout; nonempty selected Diff copy event is discarded in compact-open and advanced Diff routes: World.clipboard and clipboard_gen unchanged, selected text and modal remain; no preview/system clipboard delivery
- components: tree,file diff,split,overlay
- source: app_tests_chrome.rs:133;screens/inspect.rs:625;inline tests:881,922,1001;screens/inspect.rs:240-251,301-345;src/widgets/diff.rs:286;src/widgets/viewport.rs:1316-1389

### JA-061

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: open Inspect Advanced; click(file); wheel(tree,100); wheel(diff,100); wheel(diff,-100); drag(diff scrollbar bottom,top); resize 80x24 then 160x50 then original
- checkpoints: Diff/table region scroll routing and bounds, file selection, scrollbar capture, resize preserves semantic state
- components: tree,diff viewport,scrollbar
- source: screens/inspect.rs:643,714,718

## Regression and trust closure

Run every required source-qualified Jackin and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
