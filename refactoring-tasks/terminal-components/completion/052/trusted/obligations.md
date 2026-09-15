# TASK-052 trusted obligations

## Fixed ADJ-15 Select consumer

ADJ-15 is binding for Oracle src/bin/jackin_preview/screens/editor.rs178 dirty policy and screens/config.rs1700,1721,1838 schema/config/auth choices; editor1532–1536 and shared FormDialog1073–1135 preserve focused open Select precedence. Compose the shared configured Select with navigation(SelectNavigation::Commit).open_keys(SelectOpenKeys::ConsumeUnhandled) in both phases, or the same LabelSelect props through the existing Form bridge. Consume existing Chose only: changed values produce one domain callback, clamped/equal-value choice produces none while preserving Changed/closure flow. Do not manually set a second Select value or duplicate navigation. Preserve exact source open Tab/BackTab consumption, closed traversal, outer global-chord precedence, Esc, external focus-out, disabled/read-only and option rebuild behavior. Generic unrelated Select defaults remain unchanged. Bind R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005 to exact current task-owned component contributions and intact source scenarios under the existing ownership map; no future closure receipt is required. Baseline006 freezes these policy branches independently before candidate implementation.

EARLY-054 Consume J6's shared Jackin browser composition through intact JA-017 editor-workdir trajectory; preserve path choice, cancellation and editor state without a second browser implementation. Bind R-001/AC-001/CHK-004 and R-002/AC-002/CHK-006; Holla Files evidence is not the Jackin J6 witness.

## Re-audit shared API consumer contract

ADJ-10: source-qualified configuration RadioGroup/Form consumers commit admitted navigation, including clamped repeated-key requests, through the shared typed event/private Form bridge. Ordinary unrelated forms keep their documented default. This is binding under R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005. Direct and full application traces must prove the actual source composition; an API declaration or compile-only adaptation does not close parity.

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/jackin_preview/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-004/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore the five Editor tabs and five Settings tabs, General fields, mounts/environments/roles/accounts configuration, inherited and local values, dirty counts, validation, save preview, failure/retry and leave branches.

Share reusable field/form/configuration composition between Editor and Settings, using controlled values and stable row identities. Keep scope, workspace/account eligibility and save jobs in Jackin; preserve source masking and asynchronous save ownership.

- Replace simplified and historical editor alternatives with one live component-backed editor across dirty states, all tabs and all sizes; main app.rs:4706,5019,6416 is the enabling duplication.
- Preserve the oracle difference between config.rs:1308 number-key isolation selection and i guard paths; this task does not redesign isolation policy.
- JA-024 is the real hard-cases CLI world with more than 100 roles, not a substituted returning-world fixture. Plain environment values remain masked except explicit reveal; repeated edits count once.

## Exact membership and staging

Primary complete scenario IDs: JA-008, JA-012, JA-013, JA-016, JA-017, JA-018, JA-019, JA-020, JA-021, JA-023, JA-024, JA-025, JA-026, JA-027, JA-029.

All primary scenarios require every checkpoint. Dependency or scenario routing cannot be relaxed by the executor. A closed checkpoint is monotonic even while another checkpoint in its scenario remains future-owned.

The baseline expansion algorithms and finite axes in jackin.md and jackin-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Resolve all eight worlds, Full/Reduced/Paused, fixed seed 0x4A41434B494E5E5E and epoch 1788401640; the ordinary CLI cannot be credited with modeled-only state observations. JA-065 uses last reachable tick before/at expiry plus separate precise interaction-clock proof. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires JA-022, JA-028 through the exact frame and semantic flow tables. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

### JA-008

- world: returning,first-use
- sizes: 80x24,120x40
- actions: variant select(workspace); K(e),K(d,Esc),K(w),K(o); for delete-confirm variant confirm destructive choice; T(20)
- checkpoints: Edit route; delete guard/pending removal; prewarm status; GitHub selection/status; exact in-memory effect
- components: buttons,dialog,picker,status
- source: screens/manager.rs:1787-1882

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

## Regression and trust closure

Run every required source-qualified Jackin and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
