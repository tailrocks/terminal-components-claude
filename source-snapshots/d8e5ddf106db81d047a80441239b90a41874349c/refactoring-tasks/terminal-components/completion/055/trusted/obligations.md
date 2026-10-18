# TASK-055 trusted obligations

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/jackin_preview/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-004/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore live Capsule pane/tab topology, prefix grammar and timeout, terminal input/replies, scrollback/selection/copy, split resize/zoom, tab rename/close/context menus, palette, usage/info overlays, takeover, detach and reconnect.

Use public retained TextViewport/Code, split, Tabs, menus, picker and runtime capture with stable pane/tab identities. Keep simulated PTY transcripts/topology and World.clipboard in Jackin. Draw each reusable surface once through its live owner.

- Delete the 120x40 capsule paint-over route rather than merely making it active at more sizes; main app.rs:4037,5746,5766,6324 is the split ownership evidence.
- Preserve Ctrl-B literal-prefix and unknown-key behavior, prefix timeout, Alt-Shift resizing and modal capture exactly. Fixed oracle coordinates must address the same visible controls.
- Maintain retained viewport ownership without cloning pane transcripts; do not resurrect capsule_pane_clone_4x2000 just to satisfy an obsolete historical benchmark name.

## Exact membership and staging

Primary complete scenario IDs: JA-007, JA-040, JA-043, JA-044, JA-045, JA-046, JA-047, JA-048, JA-049, JA-050, JA-051, JA-052, JA-053, JA-054, JA-055, JA-057.

All primary scenarios require every checkpoint. Dependency or scenario routing cannot be relaxed by the executor. A closed checkpoint is monotonic even while another checkpoint in its scenario remains future-owned.

The baseline expansion algorithms and finite axes in jackin.md and jackin-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Resolve all eight worlds, Full/Reduced/Paused, fixed seed 0x4A41434B494E5E5E and epoch 1788401640; the ordinary CLI cannot be credited with modeled-only state observations. JA-065 uses last reachable tick before/at expiry plus separate precise interaction-clock proof. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires JA-056, JA-058 through the exact frame and semantic flow tables. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

### JA-007

- world: returning
- sizes: 80x24,120x40
- actions: variant select(Current directory),select(payments-platform),select(running instance),select(stopped instance),select(pane),select(+ New workspace); K(Enter)
- checkpoints: Each row activation reaches its oracle route/dialog or refusal; chosen domain identity preserved
- components: tree,picker,dialog
- source: screens/manager.rs:activate

### JA-040

- world: launch-running
- sizes: 80x24,100x30,120x40,160x50
- actions: fresh(launch-running,Full,0); T(1) repeated through deterministic launch completion; checkpoint each of eleven Stage changes and cached/skipped state; T(12) handoff
- checkpoints: All stage frontier/status/activity/account lines, build progress, handoff phase and Capsule arrival
- components: steps,progress,spinner,scheduler
- source: sim/launch.rs:10,241;screens/cockpit.rs;app_tests.rs:179

### JA-043

- world: launch-running with CredentialsLocked,BlockedSidecar
- sizes: 80x24,120x40
- actions: fixture LaunchPlan=CredentialsLocked; advance to block; resolve unlock/retry; advance completion; reset LaunchPlan=BlockedSidecar; advance block; K(Enter); reset K(Esc)
- checkpoints: Recoverable credentials/error frontier; retry clears hold; modeled-only blocked sidecar status and dismissal
- components: steps,picker,dialog
- source: sim/launch.rs:79,413;screens/cockpit.rs:641

### JA-044

- world: launch-running
- sizes: 80x24,120x40
- actions: variant K(Ctrl-C),K(Ctrl-Q,Esc),K(Ctrl-Q,Right,Enter),K(d); advance deterministic ticks
- checkpoints: Cancel/detach/quit confirmation; state/effects and entry ownership match oracle; build stops or persists exactly
- components: dialog,async ownership,arbiter
- source: screens/cockpit.rs:575

### JA-045

- world: returning
- sizes: 80x24,120x40
- actions: replay(cockpit_resolves_every_effective_account_for_the_container)
- checkpoints: All effective accounts resolved, preferred launches, runtime auth sources and per-account failures; no secret rendering
- components: credentials,steps,status
- source: app_tests.rs:1201

### JA-046

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: fresh(capsule-multi,Paused,0); K(Ctrl-B,n,Ctrl-B,p); variant Ctrl-B then digits 1,2,0,9; click(each tab)
- checkpoints: Active/inactive tabs and panes, labels/account suffix/state, out-of-range tab feedback, viewport geometry
- components: tabs,split panes,status
- source: screens/capsule.rs:1258,1433

### JA-047

- world: capsule-multi
- sizes: 80x24,120x40
- actions: K(Ctrl-B); capture prefix; T(prefix timeout ticks); K(Ctrl-B,Esc); K(Ctrl-B,Ctrl-B); K(Ctrl-B,Ctrl-L); K(Ctrl-B,unknown-char); K(Ctrl-B,r)
- checkpoints: Prefix state/timeout/cancel, literal prefix oracle behavior, clear, invalid command status and redraw flash
- components: runtime,keymap,hints,viewport
- source: screens/capsule.rs:1258;on_tick

### JA-048

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: variant K(Ctrl-B,c),K(Ctrl-B,%),K(Ctrl-B,double-quote); choose agent and account; cancel each picker on separate reset; select shell variant
- checkpoints: New tab and horizontal/vertical split; correct account restrictions; cancellation no spawn; topology retained
- components: picker,split panes,tabs
- source: screens/capsule.rs:509,567,650

### JA-049

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: K(Ctrl-B,h,Ctrl-B,j,Ctrl-B,k,Ctrl-B,l); K(Alt-Shift-Left,Alt-Shift-Right,Alt-Shift-Up,Alt-Shift-Down); K(Ctrl-B,z,Ctrl-B,z); drag(each seam,4 cells)
- checkpoints: Directional nearest focus, split resize minimums, zoom/unzoom restores proportions, exact inactive borders
- components: splitter,split panes,focus
- source: screens/capsule.rs:749,768,828,2116

### JA-050

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: T(60); type(hello); K(Backspace); paste(world); K(Enter); T(60); variant focus shell/each agent and repeat
- checkpoints: Live input, echo, cursor, deterministic replies/transcript, line wrapping and viewport follow-tail
- components: terminal viewport,text input,runtime paste
- source: screens/capsule.rs:1340,2180;sim/pty.rs:1125

### JA-051

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: T(60); K(PageUp,PageUp,Home); wheel(active pane,-100); wheel(inactive pane,3); K(PageDown,End); drag(transcript Refactor first cell,8 cells right); double-click(word Refactor); K(y,End)
- checkpoints: Scrollback boundaries, active wheel target, select/word copy exact World.clipboard, return-live and cursor mode
- components: terminal viewport,selection,clipboard
- source: app_tests.rs:646; screens/capsule.rs:2038-2180

### JA-052

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(tab_context_menu_renames_and_closes_by_mouse_and_keyboard); separate Ctrl-B comma rename with text editing; Ctrl-B m context; secondary-click tab
- checkpoints: Context menu placement, target tab identity, rename cursor, close confirmation; focus restored
- components: tabs,context menu,prompt
- source: app_tests_chrome.rs:74;screens/capsule.rs:303,326

### JA-053

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(menu_bar_opens_switches_and_runs_an_action); F10; Left/Right across all menus; Down/Up/Home/End; Esc; click each menu; hover adjacent title; click Brand/About
- checkpoints: Menu open/switch/dismiss/action/disabled states and hitboxes, terminal keys captured while menus open
- components: menubar,menu,dialog
- source: app_tests_chrome.rs:43;screens/capsule.rs:228,368

### JA-054

- world: capsule-multi
- sizes: 80x24,120x40
- actions: replay(command_palette_scrolls_with_the_wheel_and_keeps_the_selection); Ctrl-Backslash; type(no match); Backspace; type(rename); Enter; Esc; separate Ctrl-B Space and Ctrl-B colon
- checkpoints: Palette open/query/no-match/wheel/selection/action/cancel; same keyboard semantics at narrow size
- components: picker,command palette,focus
- source: app_tests_chrome.rs:161;screens/capsule.rs:843,922

### JA-055

- world: capsule-multi
- sizes: 80x24,100x30,120x40,160x50
- actions: K(Ctrl-B,u); K(Down,PageDown,PageUp); wheel(usage modal,100); Esc; click(usage chip); Esc; click(container chip); K(y); Esc; menu GitHub and About
- checkpoints: Usage read-only, all account meters, info copy, chip hitboxes, modal scroll and focus restore
- components: meters,info dialog,scrollbar,status chips
- source: screens/capsule.rs:1130,1161,1209,2555

### JA-056

- world: capsule-multi
- sizes: 80x24,120x40
- actions: variant K(Ctrl-B,x),K(Ctrl-B,&); cancel then confirm on fresh world; close last pane/tab in one-instance fixture
- checkpoints: Pane vs tab confirmation and content; topology shrinks coherently; final pane triggers correct lifecycle
- components: dialog,split panes,tabs,arbiter
- source: screens/capsule.rs:1050,1258;sim/pty.rs:1525

### JA-057

- world: capsule-multi with attached_by other client
- sizes: 80x24,120x40
- actions: fresh takeover fixture; type(ignored); K(Enter); reset; K(Esc)
- checkpoints: Takeover capture, input suppression, Enter displaces other client, Esc detaches without takeover
- components: overlay,focus,runtime
- source: screens/capsule.rs:1872

### JA-058

- world: capsule-multi
- sizes: 80x24,120x40
- actions: replay(detach_reconnect_and_final_exit_plays_one_outro); replay(still_inside_feedback_when_other_instances_remain)
- checkpoints: Detach retains daemon tabs/panes; reconnect selection; nonfinal exit stays manager; final exit gets exactly one outro
- components: arbiter,manager,panes,dialog
- source: app_tests.rs:219,251

## Regression and trust closure

Run every required source-qualified Jackin and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
