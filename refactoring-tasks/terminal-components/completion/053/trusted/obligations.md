# TASK-053 trusted obligations

## Fixed ADJ-15 Select consumer

ADJ-15 is binding for Oracle src/bin/jackin_preview/screens/accounts.rs532 provider choice, including shared FormDialog open-Select precedence. Compose the shared configured Select with navigation(SelectNavigation::Commit).open_keys(SelectOpenKeys::ConsumeUnhandled) in both phases, or the same LabelSelect props through the existing Form bridge. Consume existing Chose only: changed values produce one domain callback, clamped/equal-value choice produces none while preserving Changed/closure flow. Do not manually set a second Select value or duplicate navigation. Preserve exact source open Tab/BackTab consumption, closed traversal, outer global-chord precedence, Esc, external focus-out, disabled/read-only and option rebuild behavior. Generic unrelated Select defaults remain unchanged. Bind R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005 to exact current task-owned component contributions and intact source scenarios under the existing ownership map; no future closure receipt is required. Baseline006 freezes these policy branches independently before candidate implementation.

EARLY-054 Consume J6/J8 shared browser and staged picker paths through intact JA-034 account-folder/provider trajectory; filesystem/provider data and job lifecycle remain app-owned, with shared component painting and input. Bind R-001/AC-001/CHK-004 and R-002/AC-002/CHK-006; Holla Files evidence is not the Jackin J6 witness.

## Re-audit shared API consumer contract

ADJ-10: source-qualified Accounts choice consumers use the shared typed navigation or configured Form bridge, preserving controlled values and exact callbacks without application-local key dispatch. This is binding under R-001/AC-001/CHK-004, R-002/AC-002/CHK-006 and R-003/AC-003/CHK-005. Direct and full application traces must prove the actual source composition; an API declaration or compile-only adaptation does not close parity.

These planner-owned clauses are immutable task inputs. They bind R-001/AC-001/CHK-004 to source scenarios, R-002/AC-002/CHK-006 to ownership, R-003/AC-003/CHK-005 to full accounting and R-004/AC-004/CHK-001 to trust. Source citations below resolve at oracle commit 02f5294bfdbf38004cc49130d0aff1d01f31434c under src/bin/jackin_preview/ unless explicitly rooted elsewhere. No captured golden digest is fabricated here; accepted TASK-004/006 receipts supply the future independently captured artifacts.

## Bounded outcome and source clauses

Restore Accounts tree/filter/detail/actions, local-folder/reference/plain-key forms, validation and refresh jobs, Usage meters/details/scroll and account handoff, plus the full nested 1Password account/vault/item/field picker.

Use shared Tree, split, form/secret, Meter, viewport and async picker/layer ownership. Keep account registry/defaults/provider world jobs and World.clipboard in Jackin; no real credential/provider operations.

- Render every health/staleness/error/unsupported/exhausted outcome honestly; failed refresh retains last-good state rather than fabricating success.
- Loading, debounce, Locked/AuthorizationRequired/PermissionDenied, retry/back/cancel and concealed-only field filtering all remain separate checkpoints; parent form values and focus survive.
- No synthetic secret may appear in unauthorized frames, debug metadata or source fingerprint; approved source tails follow the exact oracle policy, not a blanket text mask in the comparator.

## Exact membership and staging

Primary complete scenario IDs: JA-022, JA-028, JA-030, JA-031, JA-032, JA-033, JA-034, JA-035, JA-036, JA-037, JA-038, JA-039, JA-064.

All primary scenarios require every checkpoint. Dependency or scenario routing cannot be relaxed by the executor. A closed checkpoint is monotonic even while another checkpoint in its scenario remains future-owned.

The baseline expansion algorithms and finite axes in jackin.md and jackin-scenarios.tsv are part of the frozen catalog input, not mutable candidate policy. Resolve all eight worlds, Full/Reduced/Paused, fixed seed 0x4A41434B494E5E5E and epoch 1788401640; the ordinary CLI cannot be credited with modeled-only state observations. JA-065 uses last reachable tick before/at expiry plus separate precise interaction-clock proof. All colors truecolor/256/16/mono and NO_COLOR, required cursors, resize, pointer down/up and intermediate frames remain required. Non-oracle main themes stay compatible architecture coverage, not alternative UX expectations.

## Frozen flow contributions

This task requires JA-011 through the exact frame and semantic flow tables. Every selected checkpoint is a complete frame. Semantic assertions have separate fixed fields/types and never claim cell equality. Complete parent ownership is fixed in app-flow-stage-audit.tsv. Read app-flow-contribution-contract.md for direct seed derivation and mandatory independent equivalence qualification. App closure reruns every app contribution and every full original parent.

## Source scenario clauses

### JA-011

- world: returning
- sizes: 80x24,120x40
- actions: replay(manager_launch_picker_hides_agents_without_an_account); variant choose agent with one account,with multiple accounts,with no valid account; Escape each nested picker
- checkpoints: Unavailable agent filtering, account defaults/preferences, cancellation keeps selection and no launch effect
- components: picker,focus,overlay
- source: app_tests.rs:1110;domain/workspace.rs

### JA-022

- world: returning
- sizes: 80x24,120x40
- actions: K(Down,e,4,Enter); variant K(e),K(p),K(s),K(d,Esc),K(d,Right,Enter,u); add invalid key and duplicate key then submit; fold role section Left/Right/Space
- checkpoints: Reference picker, scope move, delete confirmation/undo, invalid/duplicate errors, removed row and role folding
- components: config,op picker,dialog
- source: screens/config.rs:1201,1737,2159

### JA-028

- world: returning
- sizes: 80x24,120x40
- actions: K(s,4,Enter); K(Space,d,c); return Settings; K(5,Enter,Space,o); Ctrl-S then cancel
- checkpoints: Agent auth mode/default, Accounts handoff, trust toggle and source open, unsupported choices/labels
- components: list,select,checkbox,status
- source: screens/settings.rs:705-778

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

### JA-064

- world: accounts-mixed
- sizes: 80x24,100x30,120x40,160x50
- actions: K(a); focus(op chooser); Enter; T(4); open next picker; wheel(outside modal,3); K(Tab,BackTab); click(outside); Esc; reopen; nested choice select then Esc
- checkpoints: Topmost overlay captures wheel/key/pointer; layer-specific outside cancellation; parent form values/focus preserved; no background activation
- components: overlay stack,focus,form,picker
- source: app.rs:1229,1255,1333,1756

## Regression and trust closure

Run every required source-qualified Jackin and workspace identity/profile, not test listings or aggregate counts. Preserve stable identities, secret ownership and accepted asynchronous cancellation/target protections. Whole-workspace build, MSRV compile, formatting, lint and compatible architecture gates remain green at each stage. Record actual future-owned diagnostic failures without calling them passes or ignoring them; task-owned and previously closed identities must all pass. The final unfiltered workspace suite and global unresolved-set closure belong to TASK-069.

The application does not own expected snapshots, normalized identities, coordinate lookup, artifact approval, test disposition or verification binaries. Read-only host inputs are additionally protected by process isolation, source/trust hashes and exact-tree receipts. No candidate output may populate the oracle namespace; no tuisnap accept or blessing environment is allowed.
