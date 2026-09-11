# Showcase complete-branch audit — 68/68 source reads complete

Source pins: main `7b27732a8c3c131760ec3438f641cb3c11343a42`;
moving-Holla snapshot `2e2401393c47360741ebd321679de08982dca50a`;
immutable UI oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`.
`git diff H O -- src/bin/showcase` is empty. Holla supplies identical Showcase
source here, but does not replace the immutable acceptance authority.

Partition: all68 text paths in `branch-diff-inventory.tsv` whose owner is
`showcase`,16693 added/10273 removed lines under no-rename detection. These
primarily describe a semantic move from `src/bin/showcase` into an independently
compiled `apps/showcase` crate; additions/deletions are not instructions to
restore old module architecture. No production edits or captures authorized.

## Findings

### B-SC01 — Author demonstration needs explicit non-product disposition

Main `apps/showcase/src/pages/author.rs:18–82` defines an actual public-author
facade consumer, not an old-widget bypass. Preserve that architectural capability.
However `overview.rs:165–175,197,437–460` retains its mutable state, routes its
clicks, shortens the state-language card and paints a focusable badge. H/O
`pages/overview.rs:11,174–188` deliberately has no interactive control.
TASK037's exact BASE-overview and all-page focus proof can detect the regression;
it must restore the original geometry, cell attributes and absence of page
focus/hits, not hide the badge in one size or leave a zero-area pseudo-consumer.

The remaining planning question is explicit source disposition: TASK037 owns
Overview but not `pages/author.rs` or the `pages/mod.rs` declaration. TASK068
already owns real external public-author example/conformance. Give the
unneeded product-demo source/declaration a narrow named owner, while retaining
the reusable public-author proof outside the oracle product surface. Root was
notified; no task edit or finding closure is claimed here.

### B-SC02 — CLI grammar exceeds the enumerated long-option transcript

H/O `main.rs:17–61` accepts `--color`/`-c`, `--page`/`-p`, `--help`/`-h`;
unknown arguments are ignored, but missing/unknown color or page values print
the exact diagnostic and exit2. Color spellings are case-sensitive
`truecolor|24bit|256|16|none|mono`; page names normalize the navigation labels.
Main `app.rs:1304–1353` accepts only long options, silently ignores invalid or
missing values, has no help branch, accepts extra ANSI/case-insensitive color
spellings, and gives `--theme paper` visible semantics where H/O ignores it.
Main also accepts extra PageId slugs through `from_name`.

`showcase-scenarios.tsv:75` SC-SHELL-ALL-PAGES names long page launches, help,
invalid page/color, but not short aliases, missing values, spelling/case
rejection, unknown arguments or option order/consumption. A candidate can pass
that finite transcript while still breaking these source-defined CLI paths.
TASK032 owns the production `app.rs` correction, but its staged shell
contributions do not explicitly bind this CLI grammar; TASK039 is validation
only. Add a source-derived032 CLI stdout/stderr/exit/launch contribution and
full039 scenario expansion. Preserve main's public application library and
thin binary; no old runtime transplant. Runtime color tracing found B-SC03.

### B-SC03 — Existing color adjudication contradicts explicit oracle CLI override

O `main.rs:18–30,64–72` detects a default, then an explicit color flag replaces
that level before constructing the theme. O `theme.rs:30–45` treats nonempty
NO_COLOR as Mono, then recognizes COLORTERM, then256/ghostty/kitty TERM, otherwise
ANSI16. O runtime has no second capability detection. Main
`runtime/session.rs:214` instead calls `theme.for_terminal()`, whose
`theme/downgrade.rs:377–385` intersects the supplied theme with detected level.
Main `theme/tokens.rs:546–594` also adds TERM=dumb, CLICOLOR_FORCE and non-TTY
precedence absent from O.

Concrete contradiction: under `NO_COLOR=1`, `COLORTERM=truecolor`,
`TERM=xterm-256color`, explicit `--color truecolor` produces TrueColor in O;
current main and ADJ03/009 require narrowing to Mono. Merely freezing a
truecolor-friendly environment hides this branch. `architecture-adjudication.md:33–35`
and009 R003/obligations149,278,290–304 currently require the conflicting rule,
despite HM10's unresolved phrase “reconcile flags with oracle explicitly.”

Preserve the pure public4×4 `ColorLevel::narrow_to` algebra and generic runtime
default policy. Root must explicitly adjudicate the oracle application-launch
override/default policy, assign its reusable terminal-boundary support to009
before032, and freeze executable env×argument witnesses. Do not mutate process
environment inside the app, duplicate a backend, widen an already-downgraded
authored theme, or silently declare terminal capability outside UI parity.

## Exact proposed task/scenario adjustments — not yet applied

1. **037 / B-SC01:** add narrowly scoped `apps/showcase/src/pages/author.rs` and
   the author module declaration in `apps/showcase/src/pages/mod.rs` to the
   source disposition owner.037 follows032 already. Require Overview's oracle
   absence of author badge/state/hit/focus at every declared size, not one
   hidden-size trick. Keep public-author live runtime proof in068's existing
   external example/conformance; deleting accepted API proof is forbidden.
2. **032 / B-SC02:** add source row `SC-SHELL-CLI-GRAMMAR`, a matching exact
   shell contribution and PTY stdout/stderr/exit/terminal-restoration gate.
   Enumerate each accepted color value under both flags, every normalized23
   navigation label under both page flags, both help flags, unknown argument
   ignored, `--theme paper` ignored, missing values, empty/unknown values,
   rejected `TRUECOLOR`/`ansi256`/`ansi16`, and extra-only page slugs such as
   `editor` versus oracle `codeeditor`. Include ordered pairs: repeated valid
   colors/pages last-wins; invalid before help exits2; help before invalid
   exits0; `--page --help` treats help as the invalid value. Compare exact
   oracle text/exit, not merely exit class. Successful launch captures full
   frame/header/selected page then exits through real q. Retain the original
   SC-SHELL-ALL-PAGES route/cycle cases; no case deletion.
3. **009+032 / B-SC03:** add `SC-SHELL-CLI-COLOR-ENV` with explicit host env
   fixtures, separate unset versus empty variables, and flags absent/present.
   Minimum intersections: nonempty/empty NO_COLOR; COLORTERM truecolor/24bit/
   unrecognized; TERM xterm-256color/ghostty/kitty/dumb/unset; CLICOLOR_FORCE
   absent/0/nonzero; TTY versus redirected startup. Include the concrete
   NO_COLOR+explicit-truecolor contradiction above and ANSI16 environment with
   explicit truecolor. Match actual source behavior, including terminal setup
   failure when a non-TTY launch cannot run, rather than inventing a frame.
   The generic pure capability table remains a separate009 architecture gate.
4. **039+002/006:** freeze the new rows, exact expansion membership and all
   stdout/stderr/exit/frame lanes; include them in same-tree whole-Showcase
   closure. Sync row counts, task source rows, scenario IDs, stage contribution
   bindings and protected copies only through root.070's actual-binary/action
   binding must cover argv/env identity as well as input events. This author
   changes no shared scenario, task source, or acceptance asset here.

### B-SC04 — Baseline record count disguises an unreachable page cohort

Main `tests/baselines/showcase.txt:1–737` has736 records,23 names×32,
but `PageId::ALL` has22 pages. Both `Codeeditor` and obsolete `Editor` have32
records; `Editor` occupies baseline lines226–257. `tests/visual.rs:21–34`
visits only current PageIds and asks each snapshot to match its named record.
It does not reject extra catalog members. Thus704 current snapshots can pass
with32 unreachable records:736 records are not evidence of23-page coverage,
and do not supply the missing Diff page.

Structural repair:008 assigns an explicit per-record orphan disposition,
preserving historical bytes rather than silently deleting/blessing them;071
requires exact expected/reached/extra/missing baseline membership;039 requires
the23 actual oracle pages on the same candidate tree. Historical source tests
are not independent oracle authority. This is source-proven gate incompleteness,
not a claim that a fresh product test campaign passed.

B-SC01 also needs008 assertion-level disposition: main `app_tests.rs` explicitly
requires the Author demo in Overview and complete-navigation exercises, while
`sidebar_contract.rs` asserts static Overview at a size hiding that extra demo.
Removing the badge must preserve the live public-author test through068 and
replace incompatible product assertions through protected008 adjudication;
neither wholesale test deletion nor a narrow-size concealment qualifies.

Historical heading fixtures likewise bind794b095, not immutable O. For example
the160-column Scrolling blurb stops at “where you are”, while H/O scrolling
source includes the subsequent hidden-edge fade description. Preserve their
provenance and route incompatible assertions through008;002/006/039 bind the
actual complete oracle instead of treating text-only historical rows as proof.

## Read ledger — complete source coverage, not implementation acceptance

### ADJ17 source-compatible color seam assessment — proposal only

Root's proposed `TerminalColorPolicy::{Detect, Requested(ColorLevel)}` and
additive `run_with_feedback_clock_and_color(app, theme, clock, policy)` reuse
the existing single terminal driver at main `runtime/session.rs:194–214`.
Keep existing `run`/`run_with_feedback_clock` signatures and Detect behavior.
Detect calls existing `Theme::for_terminal`; Requested calls existing
`Theme::for_level` (`theme/downgrade.rs:377–385`), preserving the pure4×4 meet
and custom/palette/mono semantics. Do not call `downgrade(requested)` directly,
which can falsely raise an already-poorer theme's declared level. Explicitly
document that Requested bypasses ambient detection, not authored-token limits.

Do not infer Requested from a supplied theme's capability: existing app
library helpers such as Jackin `run_scenario_with_theme` and Tablepro `run_with`
have no explicit override parameter and must retain default Detect semantics.
Add a narrow explicit CLI launch path, with the selected FeedbackClock retained,
into the same driver. Reexport the additive API under the existing crossterm
gate at `crates/tui/src/lib.rs:54`; update the protected API inventory and
external facade consumer. No second backend/session/event loop, mutable global,
process-environment rewrite, or environment read in Runtime/Harness is needed.

Applicability is all four actual oracle binaries, not Showcase alone:

| Oracle main | Source default/override | Theme construction |
| --- | --- | --- |
| `src/bin/showcase/main.rs` |18–30|64–72|
| `src/bin/holla/main.rs` |38–55|123–125|
| `src/bin/jackin_preview/main.rs` |33–50|114–116|
| `src/bin/tablepro/main.rs` |25–40|57–59|

Each starts with old detection and replaces it for an explicit color flag;
none performs a second detection in the old runtime. All share O
`src/theme.rs:30–45`: nonempty raw `var_os(NO_COLOR)` wins by default;
Unicode COLORTERM exactly truecolor/24bit follows; Unicode TERM containing
256color/ghostty/kitty follows; otherwise ANSI16. CLICOLOR_FORCE and TTY status
are not inputs. Therefore reuse a pure source-compatible environment resolver
over an immutable host snapshot, distinct from generic Detect, for the four
CLI launch adapters. Preserve raw non-Unicode NO_COLOR as nonempty; invalid
Unicode COLORTERM/TERM behaves like absent. Those distinctions join B-SC03's
frozen env×argv witnesses. First source-compatible default/explicit resolution,
then fresh rich Junie theme and Requested; never widen already-narrowed tokens.

009 owns the reusable seam, pure tests, no-ambient-read Requested proof and
terminal restoration;032 and the Holla/Jackin/TablePro launch owners must bind their actual
application-specific default/CLI behavior and existing feedback clock. The
precise non-Showcase task scopes remain root-owned; no shared ADJ17/009 edit is
made here. This appendix is a bounded source assessment, not an implemented API.

Every row below was read in full with `git show` at the stated branch pin.
H/O identity is separately checked; earlier oracle review was not used as a
substitute for these fresh complete-branch reads. No production test or new
capture is claimed by this source-only branch audit.

Fresh branch coverage68/68 paths. Every full line is included;
large-file reads use overlapping ranges to close truncation, not search snippets.

Mechanical cross-check against the exact `partition == showcase` inventory:
68 unique paths, zero missing/extra paths, every Git blob and full line count
equal to pinned source. `branch-diff-showcase-ledger.tsv` adds each full-file
SHA-256; this is a machine provenance ledger, not a spreadsheet deliverable.
All source differences above have staged application/component owners. B-SC01
through B-SC04 remain proposals for root synchronization and independent review;
this author's completed read census does not self-close those findings.

| Path | Side | Blob | Full lines | Semantic coverage / disposition |
| --- | --- | --- | ---: | --- |
| `apps/showcase/Cargo.toml` |M|`59328db7f8f6c69daa6db26d1dc5b0b011189d60`|25|Standalone package, thin binary/public library, facade-only dependency; preserve032/065.|
| `apps/showcase/src/lib.rs` |M|`50dc532c9e5114b8e3546a8677dec77c99c21269`|28|Public App/PageId/NAV entry, app runtime forwarding; preserve032/065/068.|
| `apps/showcase/src/main.rs` |M|`90bd4e5235019393fb49341ce2947f107d0a7570`|5|Thin production binary delegates same public app; preserve032/070.|
| `src/bin/showcase/main.rs` |H|`b225ead7cb501c93a4ffcff37b6c40b5e7ec7883`|81|Old Application/input/render, CLI color aliases/page/errors/help/tick; transport behavior maps032/039 without restoring old runtime.|
| `apps/showcase/src/render_number.rs` |M|`00b784432233cd672b1c460f4dd032325e8a5bec`|68|Stack decimal formatting, selected suffix, maximum/zero proof; preserve067 allocation intent.|
| `apps/showcase/src/pages/mod.rs` |M|`cc579b98c462e8468407fe9d181bfd60a5253607`|226|Page update/draw split, status ownership, header/body/clipping/Unicode tests, page registry; preserve032 contract, B-SC01 declaration disposition.|
| `src/bin/showcase/pages/mod.rs` |H|`64caa56d4673bc8a7aa35484ecef8a4f259016d2`|180|Old PageEvent click/press/secondary/drag/wheel/paste/dialog/tick routing, mutable focus requests, rows/columns helpers; behaviors migrate032–039 through reusable runtime, not event-router copy.|
| `apps/showcase/src/pages/author.rs` |M|`8626b8227d528832fc3ee5c1a80f19d4091794d5`|82|Custom Family/PartStyle/click/selection/focus proof; B-SC01 preserve capability outside visible oracle Overview.|
| `apps/showcase/src/pages/overview.rs` |M|`ed2323d5da2d34107e0c73e718d512a34bdabbf0`|498|19tokens/5principles/7states, responsive cards, style patches, roster nominal consumers, author hit/state;037 BASE fullframe and focus, B-SC01.|
| `src/bin/showcase/pages/overview.rs` |H|`67e765fe561870b2731962682e805824ec3ad486`|189|Exact token/typography/cards/column thresholds/principle wrapping/state geometry; no live controls;037 fullframe+032 shell/focus.|
| `apps/showcase/src/data.rs` |M|`74b3bd7172462089289f80fe0b147e52d96b5b1b`|405|24 task records retain1040–1063 identities and complete values;20languages;30 explicit keyed tree nodes/labels; identical logs/prose;120 static scroll rows. Preserve borrowed fixture allocation and keyed collection architecture;034–038 full-frame/data/action proofs.|
| `src/bin/showcase/data.rs` |H|`cf2f5cf6701754731540e21ed5bfe622d5bbe37c`|367|Owned task/tree/language/log constructors and prose fully compared to main static/keyed representation; keep values and hierarchy, not old owning widget data shape;034–038.|
| `apps/showcase/src/app.rs` |M|`51ce4dfda51fea8b5ba30b8d42f864d692e97694`|1453|Full PageId/Nav table, command phases, persisted pages/status, help/header/nav/footer/inspector/minimum geometry, CLI and two inline contract modules;032/039 restore23routes and oracle interaction through shared components. B-SC02 CLI expansion; compatibility paint-over removal already explicit032R002.|
| `src/bin/showcase/app.rs` |H|`c735c1bee0c77220447c8864e33c598da4215fd2`|1309|All23routes, cursor/current/reveal, mouse press/up/secondary/drag/wheels/hover refresh, editing-before-global,0/Esc/Ctrl+C,140msflash/4sstatus/tick gating, modal focus restoration, exact header/sidebar/inspector/footer/minimum and two click regressions.032 contributions+039 fullscene; preserve behavior without app-local runtime duplication.|
| `apps/showcase/src/pages/buttons.rs` |M|`e367696e569541e794d58a60f04a2ab21a22bd41`|428|Nine borrowed live Buttons/application checked/busy state,2200msdeadline,24 inert reference cells; legacy gutter/toggle-marker/pressed paint-over, narrow matrix/layout/status and missing page hint remain033/011 work. Keep shared real Button and reference scope, eliminate page chrome repair.|
| `src/bin/showcase/pages/buttons.rs` |H|`18b554f1393fb0be5d48eeccb1e3a3d0bb6c5b30`|232|Nine old controls, two toggles, two disabled,2200msbusy,activation status/count,15+1+remaining matrix layout,6×4 state variants and Activate footer.033 BASE/playground/time/matrix and011 shared appearance.|
| `apps/showcase/src/pages/inputs.rs` |M|`d53ac6eca33cbe4af2c193116f1bf3370aa48e29`|462|Only name/branch controlled; owner/search/API-key incorrectly inert reference draws; displayed bullet string substitutes mask model; custom gutter/help repair and state matrix flags incomplete.033 restores all five enabled fields through016 TextInput/Field, real Required/email/mask/commit/paste/focus.|
| `src/bin/showcase/pages/inputs.rs` |H|`92feb3a3740e4917aac640059ab8c7e4ea9ca5a0`|265|Six fields/five enabled, exact required/email validation, masked tail4,3-row field placement,8state matrix/cursor simulation, commit/cancel status/Tab navigation/paste/click/edit hints.033 source-derived fullframe/edit/email-mask/matrix and crossfocus paths.|
| `apps/showcase/src/pages/textareas.rs` |M|`22beb7b1479264184b0667ecdf4dd5353265c96a`|369|One controlled checklist and inert Notes/Commit; removed long Unicode suffix, altered copy, hardcoded1–8 metadata and page-local gutters/help/error/placeholder.033 restores three enabled TextAreas plus one disabled via016/014; preserve controlled value/state and fixed-clock architecture.|
| `src/bin/showcase/pages/textareas.rs` |H|`f1a30a5c49cb499d118d1e60056be6251fb8f139`|161|28lines with every fourth Unicode suffix,3enabled/1disabled,8/4row viewports,13+1+remaining geometry,commit error,click/paste/verticalwheel/thumbdrag/Tab/Enter/Esc semantics.033 Unicode/edit/scrollfullframes; preserve80x24 clipped Commit no-hit rather than reflow.|
| `apps/showcase/src/pages/forms.rs` |M|`b97b458b19960aa690a2ca28380857537d584a39`|858|Controlled drafts plus real new Form/Wizard consumer paths, but extra deploy/stepper, invisible confirm/priority, inert reviewer/options/reset, wrong validation and missing1800ms completion; extensive page paint-over.033 restores oracle composition through019/018 shared APIs and preserves their live proof;008 adjudicates conflicting historical tests, not silent deletion.|
| `src/bin/showcase/pages/forms.rs` |H|`712f51123bb2b6c6b355d5c39597a75292b4d7c3`|466|Name Required/min4bytes, optional reviewer email and description,9enabled controls/disablednotify, compact1rowtextarea/action reservation, draftcommit-before-submit,strict>1800msbusycompletion/reset/defaults, radio/toggle/key/paste/click/hints,2inline regressions.033 FORM-VALIDATE/SUBMIT andBASE; ADJ10 navigation-commit semantics retained.|
| `apps/showcase/src/pages/lists.rs` |M|`4e25b2a7f447d444d70275d004e5d6d47e7abd7d`|476|Borrowed stable-key List/MultiList rows and disabled/checked providers preserved; compact hardcoded truncation, shifted columns, missing chosen/meta/focused-panel semantics remain037/020/014. Actual120-width compact CellUi META bind is included in073 timing census.|
| `src/bin/showcase/pages/lists.rs` |H|`680034f156fedc267eb1021ee8a7890812d40646`|211|Exact three-column/card geometry, language position/chosen,12files/2disabled/2checked, empty hint, pointer/keys/wheel/thumb paths;037 fullframe/list trajectory and020 reusable row engine.|
| `apps/showcase/src/pages/trees.rs` |M|`33dc2457acfa1b7e75ca9e78413408607136b4a5`|329|Stable flat keyed nodes and expanded state preserved; page-local visible-row traversal/disclosure paint-over, padded metadata, changed selection/depth/columns remain037/020. Remove copied traversal, not shared Tree engine.|
| `src/bin/showcase/pages/trees.rs` |H|`9517113e88f5c569bd9dfa186d644a6f0ba4b88d`|144|Project/Selection card heights, width max30 split, selected path/depth versus nothing, cursor/visible/open labels, fold/keys/wheel/thumb routes;037 source trajectories+020 component behavior.|
| `apps/showcase/src/pages/chips.rs` |M|`b6e11f193a119b0cf8859eb1e54ec2c2a71f3a84`|444|Live generic ChipBar/Select hidden beneath historical paint; wrong six filters/language selector/nonclosable model, no domain add/remove/lead/clear, strip/properties/empty omitted. Preserve generic keyed controls;037 composes exact domain through020 new lead/clear/provider contract and018/027 readonly components, no paint-over.|
| `src/bin/showcase/pages/chips.rs` |H|`ff38c12d7809eeb3fc6c560db12b3d53f9e09525`|338|Three domain filters/twoenabled, four cyclic add candidates, lead toggle/edit/toggle/remove/clear, sort/page-size/disabled engine, popup outside dismissal, segment priorities/two widths, wrapped properties+empty;037 trajectories/BASE plus020/018/027 exact borrowed composition.|
| `apps/showcase/src/pages/sidebars.rs` |M|`01f3695b46a3a7fcbd48fe7c46a967a47d4d7f4b`|351|Public keyed NavList replaces old application-owned widget; preserve section/icon/badge/disabled providers and separate current/cursor. Static narrow/wide row paint-over and fixed text wrapping conceal live state;037/020/014 remove through shared layout, exact20row content cap.|
| `src/bin/showcase/pages/sidebars.rs` |H|`2e6d1f3b2330d191b77ace75d8610e671892d016`|495|Old reusable NavList code now belongs library, never restore page-local engine; rows/section blanks/cursor reveal/disabled skip/current/collapse/scrollbar/fade,28→10 sidebar width,20row cards and accent-first wrapped text bind037/020.|
| `apps/showcase/src/pages/dialogs.rs` |M|`94c157a3fa8b4d3cf7848190477407f85b99867b`|287|Actual public Dialog/layer lifecycle and controlled draft retained; choice/delete shortcuts skip modals, renamed task/history lost, validation only empty, body/action/error chrome differs and page paint-over remains.038/023 restore all four dialog flows via shared semantic actions.|
| `src/bin/showcase/pages/dialogs.rs` |H|`b41585bcf0cbc3b48b7baeaad5b9a0dd612092c8`|215|Four real modals, original long bodies/action labels/initial focus, required trim-empty/>40byte validator, persistent task name, newest-first12row result history/status and every cancel/action index;038 exact modal transcripts+023 overlay/focus/validation.|
| `apps/showcase/src/pages/progress.rs` |M|`657761bbb2ab24d84809d8e43009e7ca450e0a01`|327|Moment-based coalesced80ms ticks,0.006 increment/pause/restart and public indicators preserved. Only2/5staticbars,3wrongmeters/overlap/narrow paint-over, uncapped live widths remain036/029; no deadline catch-up loop or direct clock regressions.|
| `src/bin/showcase/pages/progress.rs` |H|`bd3aa36f2bd0ed1ae53097a3d6174456578c3bb5`|247|70wide live bars,5terminal states,14wide42%bar,9capacity tones×line/block with height guards, completion status,paused animation/restart semantics;036 matrix/time/base and029 readonly state paths.|
| `apps/showcase/src/pages/scrolling.rs` |M|`9afd8883fcbfea33130c37002b139e0d35782702`|422|Independent caller scroll state/public TextViewport preserved; wrong TextViewport replaces selectable List,409staticlog/noappend, extra raw-region changes height/focus, false following label and hardcoded narrow cells.036 restores120keyed list/400log/ticks via014/021/020; raw ScrollRegion public proof retained outside oracle product surface.|
| `src/bin/showcase/pages/scrolling.rs` |H|`6daf7186a5f5b8e62b77bdaa6937e7a0c8ae0ae6`|201|Three fullheight thirds, wrapped prose×3,120selectable rows/every7flagged,400log follow→2000cap, labels after layout, per-pointer wheel/press-drag-click/thumb semantics and focus keys;036 exact panes/list/follow trajectories,021/020/014.|
| `apps/showcase/src/pages/chrome.rs` |M|`8c1116c04b3336318be38d6bb4657635e32fc295`|455|Actual Brand/StatusBar exist but wrong broad hit regions hidden by static sessions/menu/status/hints; no MenuBar/List/ContextMenu/actions/zoom.038/023/028 must replace paint with borrowed live composition, preserving actual public architecture.|
| `src/bin/showcase/pages/chrome.rs` |H|`e4832c0c2bdedcdca1ec215483420a89f553e6f5`|370|File/View/Help menus and disabled/separator/shortcuts,4sessions, context disabled edge moves, m/F10/right-click anchors, zoom badge, priority status chips with PR/usage hits, menu>context>screen hints/status;038 complete source trajectories and023/028 component paths.|
| `apps/showcase/src/pages/panels.rs` |M|`d18c927d2fbc4df9c43777a468c9d2e0877bf764`|634|Shared Panel/TextViewport/keyed List/SplitPane and patches retained. Extra split product-demo, page-local scrollbar math/prose/log/header/meta repaint and missing focused/follow metadata must disappear through036/014/021 reusable semantics; no restored old widget engine.|
| `src/bin/showcase/pages/panels.rs` |H|`c3e31c3ab3eccb7bcf79b00a11aa8b6fda1da838`|221|Titled/untitled/nested card geometry,3targets/disabledCloud, framed wrapped prose/60line toned log, exact meta after layout/follow state, scroll-wheel/thumb/keys and conditional explanatory text;036 base+scroll and014/021/020 own canonical rendering.|
| `apps/showcase/src/pages/editable.rs` |M|`379119c72101a40fce647f2a8a9131d26d6d0cb5`|602|Caller GridModel/GridEditor/FieldError preserved; reordered Task/ID to fake initial cursor, sticky-column/width differences, cancellation counted as edit, lost status, legacy table/scrollbar/meta paint hide model.034/022 restore original six-column navigation, actual commit actions and same shared Grid engine.|
| `src/bin/showcase/pages/editable.rs` |H|`9ef258a56ac79b984cb0eeb66a631573eece2641`|180|14rows/initialTaskcolumn, readonlyID/Status,Task/Owner/Branch/u32validators, preset third-row branch error, real repeated-click edit/paste/Tab/BackTab/commit/cancel, edit counter/status distinction/position metadata/reversed legend;034 and022 exact source geometry/transcripts.|
| `apps/showcase/src/pages/tables.rs` |M|`ebbe7589f22586082bd3309602d53b8a250dc4d2`|619|Keyed caller-owned sorting and borrowed cells retained; only4sortable columns, duration seconds-only, constant width/header/color/scroll paint-over, missing activation status and inert Checks substituted.034/022 restore Tasks/Checks live behavior and original formatting without old owning DataTable.|
| `src/bin/showcase/pages/tables.rs` |H|`141791e369d3d245b0ca11097c918b2dc50bc6bb`|183|24tasks/seven columns, ≥60duration m:ss, toned statuses/zerochanges, all headers/numeric0/5/6 sorting, activation status/empty Checks focus, per-frame metadata and horizontal/vertical input;034 source sort/base and022 shared Table/Grid lanes.|
| `apps/showcase/src/pages/grid.rs` |M|`7e381fc679c1092617d8e05bb3a63bb111ad770c`|412|Generic public Grid/Model retained but5health metrics/four columns unrelated to painted40customer/eightcolumn product; selection exposes impostor.034 explicitly replaces domain through022/023 while removing static customers/invisiblegutter/scroll paint.|
| `src/bin/showcase/pages/grid.rs` |H|`e7cbcc312899023d64c3e6df96051ca2db57c1f3`|403|Deterministic96customer typed generation/40paging/estimated4812→exact96, SQL quoting and sorted pending cells/inserts/deletes, all toolbar/event statuses,4ticksave and>500seats error,focus/rowcopy/preview closure.034 fullsource grid trajectories and022/023 reusable editing/queue/dialog paths.|
| `apps/showcase/src/pages/editor.rs` |M|`d0bc3b22c9d805451601d4e9462e4c1dbe5aa318`|431|Caller highlighter/segmenter and CodeEditor state retained; attribute-token highlight lost, fake3completionitems/no draw or insertion, staticcursor/runs, editing mislabeledrunning,no diagnostics/run, unrelated emptyDiff update.036 restores actual application protocol through025/024 and proper panels, removes gutter/meta paint-over.|
| `src/bin/showcase/pages/editor.rs` |H|`9d192abd37e9de31e1d1b6ff9d3cce65d201d39d`|571|26line sample/attributehighlight,22completion candidates fuzzy score+label tie,colonword replacement/autoparen,manualNull/CtrlSpace andautomatic2byte trigger,10tick blockrun/diagnostics/msformula,copy/find/scroll/cursor/props/exact26/11/9card heights;036 completion/run/edit and025/024 public ownership.|
| `src/bin/showcase/pages/diff.rs` |H|`8eee874261a75f5ef3f370e8657f377b06887af4`|333|Missing main route/page must be recreated as facade composition:5hunks/Unicodecombining/CJK,review/unified/empty toggles reset selection,real press anchor/drag/release/copy/pan/thumb,narrowreviewfallback;3inline component/fullApp tests.032 route+036 application+026 DiffView, no fictitious loading/error fixtures.|
| `apps/showcase/tests/perf.rs` |M|`461f634a803d2c9deab13ea13fde6018ebde2ac8`|150|Seven smoke paths retain actual Harness, pointer/wheel and scene determinism but render_twice_allocates_the_same never measures allocations. Last test actualperf::bench→selfbaseline report is not protected acceptance;067/071/073 add independently bound counters/wholeframe timing,008 preserves or adjudicates existing tests.|
| `apps/showcase/tests/perf_baseline.txt` |M|`7e557fa112799bc9fc8e09524efdaf75bf10a605`|2|Historical Lists120x40 222361ns/1alloc/60bytes, not style numerator or product≤5certificate. Preserve record,067/071 use frozen protected comparables; no recapture to make current pass.|
| `apps/showcase/tests/elapsed_contract.rs` |M|`5c5ceefb518c0761c4a9d7e9c975441c32807c13`|202|Eight real Harness time tests:2200job,strict>4000status,replacement/hidden/modal eligibility,80mscoalescing/pause/restart/completion once/narrow. Preserve Moment/eligibletick architecture; cited794b095 historical pin is not immutableoracleauthority;008/032/033/036 reconcile exact actual source conditions.|
| `apps/showcase/tests/page_headings.rs` |M|`289c909524526a115c637ade49989fa9b2b0046f`|52|352page+355shell historical text-row only comparisons at794b095,Junie fourlevels/foursizes and from_name; neither23pagewholeframe nor interaction/style proof.008 preserves compatible rows;002/006/039/070 freeze canonical complete oracle matrix and capture binding.|
| `apps/showcase/tests/visual.rs` |M|`29d4272c3fe849af9b761e5f3a81154712a37357`|34|All current PageIds×4sizes×2themes×4levels snapshot selfbaseline; valuable deterministic guard but not independentoracle.008/039 preserve compatible baseline tests with protected adjudication;002/006 actual authority, no blessing escape.|
| `apps/showcase/src/pages/terminal.rs` |M|`e252125d8f6334916b983d921f67c59922fc762c`|496|Public TextArea/Steps controls wrongly replace styled selection/copy viewport;120staticlog/no domain tick,immediate stage0failure,missing real split,hardcoded0.7s/narrowrail/status.036 must use021/020/029 semantic components and actual7stage state; no page-local split/rail renderer.|
| `src/bin/showcase/pages/terminal.rs` |H|`36c1c41121711bcd0988ef418260df334e7d6a79`|386|7stages exact10/26/40/8/18/14/1ticks with stage3cachedskip,stage1midpointfailure and downstreamblocked,typedcolored output/follow2000cap,62%splitmin24/16/seam,originalpresscopy status/railmeta/reset;036 terminalrun/select,021/020/029.|
| `apps/showcase/src/pages/taskrunner.rs` |M|`2e3097b8d3e9e36ac48cde8e52f79dc45b3dffe1`|598|Shared keyed Steps/Dialog/command architecture retained;5wrong sequentialsteps advance on everyupdate notTick,missingTree/log/max2/speeds/integrationfailure,wrongcancelstate and staticpaint.036 actual six-task domain engine/cause gating,029/021/020/023 actualcomposition, preserve accepted ActionKey dispatch.|
| `src/bin/showcase/pages/taskrunner.rs` |H|`0dab54e19f842d920b6f48ce26ec335a598c7115`|488|TargetTree/folds,6tasks0.012/.018/.024speed,max2(onequeuedstartperTick),every9log,deterministicintegrationfailure,tickscounterpersistsrestart,destructivecancel preservesfinished,followinglog andnarrowoneactionlayout;036 fullrun/cancel/restart/scroll and029/021/020/023.|
| `apps/showcase/tests/sidebar_contract.rs` |M|`38958f7961ac9c906e7e6620a565f6d4b0c0b48c`|195|Actual Harness header/row geometry at heights24/30/31/40 and width110 boundary; disabled skip, current versus cursor, hover, press/release outside, clipped no-hit, resize and runtime focus. Preserve010/020/032 architecture; static Overview only at narrow size does not close B-SC01.|
| `apps/showcase/src/pages/settings.rs` |M|`a90d79f072f43db1ca50faf3ea87c48ed7f3eb63`|659|Public Tabs/borrowed List/Dialog state preserved; wrong Security tab, static General, missing Environment, no editable member columns/role/activity, overlapping buttons and hidden-member updates remain035. Restore through016/018/020/022/023/027, not static repaint.|
| `src/bin/showcase/pages/settings.rs` |H|`8123f7f319a13c8e5a3ecbc2d19586c61ef684d2`|592|General required name/description/visibility/toggles/dirty/save confirmation; Tabs-focus consumes Ctrl-S before dirty clear; member6rows/4columns/name-role validation/sort-source-row removal/dialog/status; Environment5masked vars/uppercase key grammar/add-empty/remove-checked/reset selection. Exact16/member+6/env+5 card heights and paste-versus-InputChanged dirty behavior map035 and shared016/018/020/022/023/027.|
| `apps/showcase/src/pages/pickers.rs` |M|`177e6e459b6622b6daeb59a29b8b4425faec95c6`|1169|Borrowed34 quick entries and actual public Picker/layer action draining preserved; missing true fuzzy filtering, scope persistence, secondary semantics, deletion/current-level tags; static popup query/rows/footer and ignored visible offset conceal controls.038 restores domain via024/023; preserve FilterList/PickerChain/Menu/Context API proof outside unrelated oracle surface, no app-local popup engine.|
| `src/bin/showcase/pages/pickers.rs` |H|`e70cdceedb892420d1cc31d74ebf04ab75647673`|446|22file+12task fuzzy label ranking/task penalty5/label tie; persistent scope; four tabs Delete uses filtered index and stops at1; six enabled levels/current tag; secondary status, empty-submit stays open, outside dismissal. Exact80/48/112 popup widths/card7/max8 rows map038/024/017.|
| `apps/showcase/tests/app_tests.rs` |M|`f5812515fb5c9484648cbaa3395bde6490eebfa6`|996|Real public Harness runtime ring/custom theme/init/action tests preserved; conflicting Author/forms-summary/settings/terminal expectations require008 assertion-level adjudication. Sort test names clear but performs only two header clicks.032–039 replace incompatible oracle assertions through protected proof;065/068 preserve API/runtime capabilities.|
| `src/bin/showcase/app_tests.rs` |H|`11a5d5de06cc222ffb0bfef364f1b35504f51e76`|999|Original App/TestBackend harness, broad size/clipping/focus loops, selection/scroll labels400→401/resize/thumb offset/hover refresh/wheel boundary/grid focus/narrow terminal; old vacuous Rust assertion and BLESS_UPDATE are historical, not acceptance. Map014/020/021/025/032–039 public Harness; no copied old runtime. Source panic findings mean broad tests are not assumed passing.|
| `apps/showcase/tests/baselines/showcase.txt` |M|`e32c771074ffd8bf631c449a513214d440491bb3`|737|736 historical digests across23 names with obsolete Editor plus Codeeditor; only22 current PageIds are reached. B-SC04 exact membership and008 per-record disposition; no implicit Diff coverage or baseline blessing.|
| `apps/showcase/tests/fixtures/holla-page-headings.tsv` |M|`937469be55bff7007cfac8c036dc0909f0ad8647`|352|22labels×4sizes×4levels text-only page title/blurb rows, identical text across palettes; older Scrolling copy and old slugs retain historical provenance, not immutable O or style proof.008/002/006/039.|
| `apps/showcase/tests/fixtures/holla-shell-headers.tsv` |M|`6e6cebb12984f023ccdeab6d80f59dbb19267a9f`|355|22labels×4sizes×4levels plus Overview widths89/90/91; group breadcrumbs, color text and right hint spacing; color indicator appears at91 not90. Historical text-only corpus omits Diff/interactions/cell attrs and environment/CLI semantics.032/039 fullframe and B-SC02/B-SC03 expand authority;008 preserves or adjudicates historical rows.|
