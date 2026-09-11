# TASK-057 trusted obligations

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/jackin_preview/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-004/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Prove all 70 JA scenarios and every frozen world, motion, size, color and checkpoint expansion pass together on one integrated Jackin tree, including the connected 40-step journey, lifecycle and no-clone performance proof.

Independently reject historical-frame switches and generic widget replication while retaining legitimate rain art, stable manager/pane identities, asynchronous save ownership and the accepted public facade.

- This is validation-only. Do not repair application or library source, alter expected output, or waive failures. Return evidence to the existing responsible production task and reverify the integrated result.
- The eight CLI worlds, Full/Reduced/Paused motion, color/NO_COLOR, minimum-size recovery and controlled-clock boundary requirements are exhaustive; modeled-only arbiter/BlockedSidecar observations remain in their declared lane.
- The Jackin unresolved-failure set must be empty. All required Jackin tests and applicable release performance checks execute, including the no-clone replacement rather than the obsolete clone benchmark.

## Exact membership and staging

Primary complete scenario IDs: JA-001, JA-062, JA-063, JA-065, JA-066, JA-067, JA-068, JA-069.

Closure additionally requires every JA source scenario below, every finite variant and all checkpoints, regardless of earlier primary ownership. No Jackin unresolved failure remains. All primary scenarios require every checkpoint. Dependency or scenario routing cannot be relaxed by the executor. A closed checkpoint is monotonic even while another checkpoint in its scenario remains future-owned.

The baseline expansion algorithms and finite axes in jackin.md and jackin-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Resolve all eight worlds, Full/Reduced/Paused, fixed seed 0x4A41434B494E5E5E and epoch 1788401640; the ordinary CLI cannot be credited with modeled-only state observations. JA-065 uses last reachable tick before/at expiry plus separate precise interaction-clock proof. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires JA-022, JA-028, JA-011, JA-040, JA-043, JA-044, JA-045, JA-056, JA-058 through the exact frame and semantic flow tables. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

### JA-001

- world: first-use,returning,accounts-mixed,launch-running,launch-failure,capsule-multi,outro-last,hard-cases
- sizes: 80x24,100x30,120x40,160x50
- actions: fresh(each world,Paused,0); draw; T(5)
- checkpoints: Initial route/full cells/cursor; paused frames unchanged; all eight worlds
- components: shell,brand,collections,panes,meters
- source: scenario.rs:5;app.rs:201

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

### JA-007

- world: returning
- sizes: 80x24,120x40
- actions: variant select(Current directory),select(payments-platform),select(running instance),select(stopped instance),select(pane),select(+ New workspace); K(Enter)
- checkpoints: Each row activation reaches its oracle route/dialog or refusal; chosen domain identity preserved
- components: tree,picker,dialog
- source: screens/manager.rs:activate

### JA-008

- world: returning,first-use
- sizes: 80x24,120x40
- actions: variant select(workspace); K(e),K(d,Esc),K(w),K(o); for delete-confirm variant confirm destructive choice; T(20)
- checkpoints: Edit route; delete guard/pending removal; prewarm status; GitHub selection/status; exact in-memory effect
- components: buttons,dialog,picker,status
- source: screens/manager.rs:1787-1882

### JA-009

- world: returning
- sizes: 80x24,120x40
- actions: variant select(instance); K(n),K(a),K(x),K(i),K(t,Esc),K(p,Esc),K(r); separate fresh confirmed stop/purge variants
- checkpoints: Session picker, shell, inspect, stop/purge confirmation, reconnect/restore; status-specific enabled actions
- components: picker,dialog,inspect
- source: screens/manager.rs:1829-1882

### JA-010

- world: hard-cases
- sizes: 80x24,100x30,120x40,160x50
- actions: fresh(hard-cases,Reduced,0); K(F5); T(60); K(End,PageUp,Home); wheel(tree,100); wheel(tree,-100)
- checkpoints: Discovery failure keeps last-good rows; long names, many rows, missing daemon, scroll edge clipping
- components: tree,status,scrollbar
- source: screens/manager.rs:1770;domain/fixtures.rs

### JA-011

- world: returning
- sizes: 80x24,120x40
- actions: replay(manager_launch_picker_hides_agents_without_an_account); variant choose agent with one account,with multiple accounts,with no valid account; Escape each nested picker
- checkpoints: Unavailable agent filtering, account defaults/preferences, cancellation keeps selection and no launch effect
- components: picker,focus,overlay
- source: app_tests.rs:1110;domain/workspace.rs

### JA-012

- world: returning
- sizes: 80x24,100x30,120x40,160x50
- actions: K(n); K(Space,Enter,Enter,Enter); replay(prelude_creates_a_pending_workspace_and_opens_the_editor)
- checkpoints: All five steps, skipped Edit, pending editor, no durable workspace before save
- components: browser,choice,prompt,editor
- source: screens/prelude.rs:31;app_tests.rs:416

### JA-013

- world: returning
- sizes: 80x24,120x40
- actions: K(n); browser K(g); type(https://github.com/example/demo.git); choose source; choose custom destination; type(/work/demo); continue through Edit and Workdir; type(Demo); rewind Esc through each step
- checkpoints: Git/source/destination validation; choice persistence and stepper labels; no persistence during rewind
- components: browser,form,picker
- source: screens/prelude.rs; screens/modals.rs:266

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

### JA-016

- world: returning
- sizes: 80x24,100x30,120x40,160x50
- actions: K(Down,e); K(1,2,3,4,5,],[); variant click(each of five editor tabs); Enter then Tab/BackTab through captured ring then Esc
- checkpoints: All editor tabs and body/action focus paths; active/inactive/dirty styling and final-row hints
- components: tabs,focus,forms
- source: screens/editor.rs:48,1615

### JA-017

- world: returning
- sizes: 80x24,120x40
- actions: K(Down,e); focus(editor.name); K(Enter); type(x); K(Left,Home,End,Backspace,Delete,Esc); focus each General checkbox/select/browser and activate once
- checkpoints: Name editing/cursor/cancel, uniqueness; keep-awake/git-pull/dirty policy/workdir changes; global letters do not navigate during editing
- components: text input,checkbox,select,browser
- source: screens/editor.rs:130,1540;app.rs:579

### JA-018

- world: returning
- sizes: 80x24,120x40
- actions: replay(editor_edits_count_once_preview_then_saves_and_returns)
- checkpoints: Readonly+isolation edits count once; leave confirmation cancels; save preview/saving/completion persist exact mount and restore manager
- components: config,dialog,async ownership
- source: app_tests.rs:497

### JA-019

- world: returning
- sizes: 80x24,120x40
- actions: K(Down,e,2,Enter); variant K(a),K(e),K(r),K(i),K(1),K(2),K(3),K(o),K(d,u); fixture running_isolated=true then K(i); dismiss nested form
- checkpoints: Mount add/edit/delete/undo, all isolation states, refusal while running, GitHub status, removed row restoration
- components: config table,form,dialog,select
- source: screens/config.rs:1201-1414

### JA-020

- world: returning
- sizes: 80x24,120x40
- actions: K(Down,e,3,Enter); K(Space,Enter); K(/); type(architect); K(Enter); K(a); select(role fixture); Enter; Esc
- checkpoints: Role enabled/default, search no-match/match, load picker, account eligibility and dirty-count semantics
- components: list,picker,checkbox
- source: screens/editor.rs:1192

### JA-021

- world: returning
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(editor_env_plain_value_stays_masked_and_can_be_shown)
- checkpoints: Plain env masked initially, m reveal/remask, new key form, cursor and tail-only display, exact dirty state
- components: secret field,form,config table
- source: app_tests.rs:538

### JA-022

- world: returning
- sizes: 80x24,120x40
- actions: K(Down,e,4,Enter); variant K(e),K(p),K(s),K(d,Esc),K(d,Right,Enter,u); add invalid key and duplicate key then submit; fold role section Left/Right/Space
- checkpoints: Reference picker, scope move, delete confirmation/undo, invalid/duplicate errors, removed row and role folding
- components: config,op picker,dialog
- source: screens/config.rs:1201,1737,2159

### JA-023

- world: returning
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(editor_accounts_tab_switches_inherited_defaults_off_and_extra_accounts_on); K(/); type(no-such-account); Esc
- checkpoints: Inherited defaults, local enables/disables, preferred account, counts, no-match filter; retained identity after sorting
- components: grouped list,checkbox,filter
- source: app_tests.rs:1028;screens/editor.rs:1079

### JA-024

- world: hard-cases
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(environments_stay_readable_with_a_hundred_roles); checkpoint(Roles-end-load-row); K(Esc,4,Enter); checkpoint(Environments-reentered); K(End,PageUp,Home); wheel(config,100); wheel(config,-100)
- checkpoints: More-than-100-role summary/override picker remains readable; top/middle/bottom boundaries and fades; real CLI hard-cases world
- components: config,virtual collection,scrollbar
- source: app_tests.rs:1137

### JA-025

- world: returning
- sizes: 80x24,120x40
- actions: K(Down,e); change General field; K(Esc); variant choose Stay,Discard,Save; save fail-once fixture then retry
- checkpoints: Unsaved leave branches preserve/discard/persist pending state exactly; failed save keeps edits and prevents duplicate job
- components: dialog,form,async ownership
- source: screens/editor.rs:open_preview,start_save,finish_save,on_esc_top

### JA-026

- world: returning
- sizes: 80x24,100x30,120x40,160x50
- actions: K(s); K(1,2,3,4,5,],[); variant click(each settings tab); Enter,Tab,BackTab,Esc
- checkpoints: Five settings tabs, General coauthor/DCO toggles, focus ring and hints
- components: tabs,checkbox,focus
- source: screens/settings.rs:40,826

### JA-027

- world: returning
- sizes: 80x24,120x40
- actions: K(s,2,Enter); edit mount scope with s; add/edit/delete/undo mount; K(Esc,3,Enter); add/edit/scope/delete/undo env; Ctrl-S then cancel
- checkpoints: Global mount/env semantics and scope picker differ correctly from workspace scope; preview lists exact changes
- components: shared config,form,dialog
- source: screens/settings.rs:59;screens/config.rs:130-148,1527

### JA-028

- world: returning
- sizes: 80x24,120x40
- actions: K(s,4,Enter); K(Space,d,c); return Settings; K(5,Enter,Space,o); Ctrl-S then cancel
- checkpoints: Agent auth mode/default, Accounts handoff, trust toggle and source open, unsupported choices/labels
- components: list,select,checkbox,status
- source: screens/settings.rs:705-778

### JA-029

- world: hard-cases
- sizes: 80x24,120x40
- actions: replay(settings_trust_toggle_and_failed_save_keep_edits)
- checkpoints: First save failure keeps pending edits; second succeeds; correct global state and manager return
- components: dialog,async status
- source: app_tests.rs:581

### JA-030

- world: accounts-mixed
- sizes: 80x24,100x30,120x40,160x50
- actions: K(Home,Down,Down,Tab,Esc,*, -,End,Home); K(/); type(Work); Esc; type filter with no matches; Esc
- checkpoints: Overview/provider/account/Add rows, health, quota, freshness, default/disabled state, search and focus
- components: tree,meters,split detail,filter
- source: screens/accounts.rs:61,1946

### JA-031

- world: accounts-mixed
- sizes: 80x24,100x30,120x40,160x50
- actions: click(account Work); move(account action); down(account action); up(same); reset; down(action); up(outside); drag(accounts seam,4 cells); wheel(tree,100); wheel(inspector,100); wheel(inspector,-100)
- checkpoints: Hover/down/up/flash, no outside-release activation, splitter geometry, independent wheel boundaries and fades
- components: hit registry,splitter,scrollbar,buttons
- source: screens/accounts.rs:on_click,on_drag,on_wheel;app.rs:1535

### JA-032

- world: accounts-mixed
- sizes: 80x24,120x40
- actions: replay(accounts_register_with_a_1password_reference_and_never_render_the_secret)
- checkpoints: Four picker steps, duplicate ref rejected, Codex throttled save/refresh, secrets absent from every frame and debug metadata
- components: form,op,picker,secret,async
- source: app_tests.rs:273

### JA-033

- world: accounts-mixed
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(accounts_plain_key_is_masked_everywhere_and_remove_asks_first)
- checkpoints: Typing masks secret; approved tail only; save source fingerprint does not embed key; removal cancellation retains account
- components: secret input,form,dialog
- source: app_tests.rs:359

### JA-034

- world: first-use
- sizes: 80x24,120x40
- actions: replay(complete_jackin_flow_keyboard_first) through account registration step 10; separately capture every form source/provider change
- checkpoints: Claude Personal/Work local folders, Codex/Grok refs with endpoint, OpenCode plain key; exactly five saved accounts
- components: form,browser,select,op
- source: app_tests.rs:646-786

### JA-035

- world: accounts-mixed
- sizes: 80x24,120x40
- actions: select(account Work); variant K(e),K(d),K(Space),K(v),K(r),K(m),K(x,Esc),K(x,Right,Enter),K(F5); T(60)
- checkpoints: Edit, enable/disable, default, validation lifecycle, refresh, masked display, removal; exact registry/default/effect changes
- components: form,dialog,tree,async
- source: screens/accounts.rs:1975-2111

### JA-036

- world: hard-cases
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(hard_cases_refresh_keeps_last_good_and_help_opens_everywhere); variant select each fixture account then refresh T(60)
- checkpoints: Broker unreachable, missing/invalid source, stale/failed/unsupported/exhausted/throttled quota remain honest; no fabricated successful result
- components: meters,status,help
- source: app_tests.rs:618;domain/fixtures.rs;sim/provider.rs

### JA-037

- world: accounts-mixed with op error variants
- sizes: 80x24,120x40
- actions: open account op picker; variant Locked,AuthorizationRequired,PermissionDenied; T(4); choose retry after deterministic unlock; repeat each step Esc backward and cancel
- checkpoints: Loading/error/retry messages, account/vault/item/field focus, concealed-only fields, no secret leaks; parent form values retained
- components: async picker,overlay,focus,secret policy
- source: screens/modals.rs:1555,1663,1803;sim/onepassword.rs

### JA-038

- world: accounts-mixed
- sizes: 80x24,120x40
- actions: open op item picker; type(Anthropic); T(4); K(Down,PageDown,PageUp,Home,End); type(no-match); T(4); Esc; wheel(picker,100); resize 80x24 then 120x40
- checkpoints: Query debounce, empty results, stable selection, wheel without changed selection, loading capture and clipped long results
- components: picker,text input,scroll
- source: screens/modals.rs:1889;app.rs:1283

### JA-039

- world: returning
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(usage_overlay_is_read_only_and_hands_off_to_accounts); reset K(u,Down,Enter,PageDown,PageUp,r); T(60); click(account); wheel(list,3); wheel(detail,100)
- checkpoints: Usage read-only overview/provider/account, meters/timestamps, refresh and selected-account handoff; correct scroll target
- components: meters,list-detail,viewport
- source: app_tests.rs:400;screens/usage.rs:752

### JA-040

- world: launch-running
- sizes: 80x24,100x30,120x40,160x50
- actions: fresh(launch-running,Full,0); T(1) repeated through deterministic launch completion; checkpoint each of eleven Stage changes and cached/skipped state; T(12) handoff
- checkpoints: All stage frontier/status/activity/account lines, build progress, handoff phase and Capsule arrival
- components: steps,progress,spinner,scheduler
- source: sim/launch.rs:10,241;screens/cockpit.rs;app_tests.rs:179

### JA-041

- world: launch-running
- sizes: 80x24,120x40
- actions: fresh(launch-running,Full,0); T(40); K(b,PageUp,Home,PageDown,End,Esc); click(build log); wheel(log,100); wheel(log,-100); K(i,Esc,c,Esc)
- checkpoints: Build log overlay full scroll range, at-tail behavior, info/credential views; modal input never launches/cancels underneath
- components: log viewport,info,scrollbar,overlay
- source: screens/cockpit.rs:575;app_tests.rs:646

### JA-042

- world: launch-failure
- sizes: 80x24,100x30,120x40,160x50
- actions: replay(launch_failure_returns_to_the_construct_when_another_instance_runs); separate variant no other instance remains
- checkpoints: Network failure details and acknowledge; nonfinal manager status vs final lifecycle; no successful launch effect
- components: error dialog,steps,arbiter
- source: app_tests.rs:203;sim/launch.rs:79

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

### JA-062

- world: outro-last
- sizes: 80x24,100x30,120x40,160x50
- actions: end last instance using JA-058; Full checkpoints at outro 0,OUT_WARP-1,OUT_WARP,OUT_WARP+OUT_CAPTION-1,end-1; variant elapsed present/absent; K(Enter) at each phase on reset
- checkpoints: Warp/caption phase cells, elapsed grammar, skip behavior, final quit and terminal restoration
- components: brand,scheduler,arbiter,runtime
- source: rain.rs:643-850;arbiter.rs:119

### JA-063

- world: returning,accounts-mixed
- sizes: 80x24,100x30,120x40,160x50
- actions: for each host route Manager/Accounts/Usage/Settings/Editor/Prelude open reachable help; K(PageDown,PageUp,Down,Up,Esc); F10; switch menus; click strip links
- checkpoints: Route-specific help/commands, host menu actions, focus restoration; hints remain last row across all layers
- components: help,menubar,hintbar,overlay
- source: app.rs:699,882;app_tests_chrome.rs:110

### JA-064

- world: accounts-mixed
- sizes: 80x24,100x30,120x40,160x50
- actions: K(a); focus(op chooser); Enter; T(4); open next picker; wheel(outside modal,3); K(Tab,BackTab); click(outside); Esc; reopen; nested choice select then Esc
- checkpoints: Topmost overlay captures wheel/key/pointer; layer-specific outside cancellation; parent form values/focus preserved; no background activation
- components: overlay stack,focus,form,picker
- source: app.rs:1229,1255,1333,1756

### JA-065

- world: returning,capsule-multi,accounts-mixed
- sizes: 80x24,100x30,120x40,160x50
- actions: move(action); down(action); checkpoint; up(action); checkpoint; T(at 139ms); T(at 140ms); reset down(action),drag(outside),up(outside); type navigation key; move(action)
- checkpoints: Hover/press/flash boundaries and keyboard hover suppression; no accidental activation from outside release
- components: interaction,style state,runtime clock
- source: app.rs:340,1535

### JA-066

- world: returning,capsule-multi,accounts-mixed
- sizes: 80x24,100x30,120x40,160x50
- actions: navigate to scrolled/focused state; resize 71x19; send Down/Enter; resize 72x20; resize 80x24; resize 160x50; resize original; separate too-small q and Ctrl-C
- checkpoints: Minimum guard blocks normal actions; exact TooSmall cells; recover state/focus/scroll/layout/cursor; quit still works
- components: too small,runtime,layout
- source: app.rs:41,505,2251;app_tests.rs:264

### JA-067

- world: all eight CLI worlds
- sizes: 80x24,100x30,120x40,160x50
- actions: for each color truecolor,256,16,none and NO_COLOR without explicit color: fresh(world,Paused,0); rerun JA-016,JA-030,JA-046 representative interactions
- checkpoints: Color conversion, mono modifiers, selected/focused/disabled differentiation; exact glyphs and cursor
- components: theme,style resolver,all reused components
- source: main.rs:34;shots/audit/jackin-*

### JA-068

- world: CLI
- sizes: 80x24,120x40
- actions: spawn --help; variant invalid --color,--scenario,--motion,--frame; variant JACKIN_NO_MOTION=1 with explicit --motion full,reduced,paused; run paused --frame 45; terminate normal q/Ctrl-C paths
- checkpoints: Exact CLI help/errors/exit 2, argument precedence, frame selection, alternate screen and terminal mode restoration
- components: CLI,runtime
- source: main.rs:34-108;scenario.rs:82

### JA-069

- world: first-use
- sizes: 120x40
- actions: replay(complete_jackin_flow_keyboard_first) with every action checkpoint and frozen oracle tick counts
- checkpoints: Connected 40-step registration/workspace/configuration/launch/session/selection/detach/reconnect/multiple-instance/final-exit journey; exactly retained domain effects
- components: all Jackin component families
- source: app_tests.rs:646-1026

### JA-070

- world: arbiter fixtures
- sizes: 80x24,120x40
- actions: variant foreign entry claim,unknown discovery,missing entry time; construct oracle state then request entry/exit; compare production rendered route/status/caption
- checkpoints: No duplicate intro/outro, discovery fails closed, missing time omits elapsed; unchanged simulated running-instance count
- components: arbiter,status,brand
- source: arbiter.rs:149-204;app.rs:201

## Regression and trust closure

Run every required source-qualified Jackin and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; none may remain for this app. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
