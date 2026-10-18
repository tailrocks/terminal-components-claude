# Showcase contract audit

Pinned reference: `794b095c196562d38f1b6f7ce379c128af2a023d`. Implementation base: `c12cad8728755cd2d03eefdd8e02891143fca86d`. Read-only source investigation. No binary execution, screenshot inspection, behavioral pass, independent review, or visual parity claimed. Initial main compile failure is owned by integrator. All findings below are source-proven unless explicitly marked proposed.

## Scope inventory

All 22 production pages have matching migration destinations `src/bin/showcase/pages/{name}.rs` → `apps/showcase/src/pages/{name}.rs`. `pages.json` records source hashes and handler/hint/editing/animation locations; `reference-handlers.md` preserves complete extracted handler bodies for review. This is exact page inventory and source interaction inventory; it does not prove current implementation parity or test reachability.

| Page | Required interaction families |
|---|---|
| Overview | Static introduction/component content; shell [ ] pages and inspector |
| Buttons | Four action variants; two toggles; two disabled controls; busy long job; inert state matrix |
| Inputs | Enter edit, commit/cancel, selection, clear, validation and cursor |
| Text areas | Multiline editing, newline, selection, scroll, Tab next, Esc done |
| Forms | Fields, radio choice, checkboxes, submit Ctrl+S, reset, validation/focus errors |
| Lists | Single choice, multi choice, range selection, all/none, ends |
| Trees | Fold/unfold, navigation, open, expand all |
| Tables | Row/column navigation, three-state sorting, header click, select |
| Editable tables | Cell movement, edit, commit/cancel, validation, Tab next cell, double click, sort |
| Panels | Nested list; scroll viewport; follow-tail log; wheel, page, ends |
| Sidebars | Expanded/collapsed navigation, disabled rows, cursor/current split, open |
| Dialogs | All supplied button-launched dialog variants, prompt validation, results, modal barrier |
| Progress | Supplied progress surfaces and controls; tick animation, activate |
| Scrolling | Viewport, log follow, list navigation, scrollbar click/drag and wheel ownership |
| Terminal | Scroll/follow, oldest/live, drag selection, copy, clear, rail navigation, resize split |
| Code editor | Edit, complete Ctrl+Space, run block Ctrl+R, block jump, find, completion accept/close |
| Data grid | Cell edit, validation, sorting, row select, insert/delete, undo, toolbar actions |
| Chips & selects | Move/toggle/edit/add/remove/clear filters; sort/page-size/engine choices |
| Pickers | Quick/tabs/level picker variants; open/close, search/scope/hierarchy/action workflows |
| Chrome | Focus/layer-derived hints, menu activation incl. F10, contextual chrome actions |
| Settings | Tabs/jump; name/description edit; members editable table; environment multi-select; save/remove workflows |
| Task runner | Pipeline run/cancel, tree folding, log follow/scroll, tick progression |

For each row: required proof is actual production-view cells + real runtime event path + representative terminal-process path; handler source alone is insufficient. Expand nested widget action tables during each family's shared migration (page hints alone are not exhaustive widget semantics).

## Shell contract

Reference `app.rs:445`: focused navigation owns Up/Down/j/k/Home/End/g/G. Cursor movement does not navigate. Enter/Space chooses and stays navigation-focused; Right/l chooses and enters page. Page receives editing input before global shortcuts. Tab/BackTab cycle; q quit; ? help; i inspector; [ ] cyclic pages; 0 navigation; Esc returns focus to navigation, never exits or changes page. Main handles Chose and EnterContent identically (`app.rs:1199`): required focus transfer absent from that branch.

Reference `app.rs:576`: mouse down sets press and relevant focus; matching release activates; release elsewhere cancels; move clears keyboard-hover suppression; drag dispatches capture; secondary dispatches page context; wheel +/-3 follows scroll owner, sidebar wheel consumed; modal blocks background dispatch. Header help/inspector clickable. Preserve modal focus restoration and hover suppression in shared runtime rather than copied application interaction engine.

Reference `app.rs:738`: minimum 72x20; header y0/footer last row; body y2 height h-4; sidebar width19 below110,24 at110+; inspector width30 only enabled and width>=100, with two-cell gaps. Test width71/72,99/100,109/110; height19/20,30/31, plus 80x24,120x40,160x50.

Reference `app.rs:1018`: footer uses dialog hints first, then navigation hints, then focused-page hints; EDIT badge when page editing and no dialog. Tab Next appended only when not editing. Right-side transient status reserves width+3, otherwise reserves14. Hints clip as whole entries. Status expires when elapsed >4s (`app.rs:386`); flash expires >=140ms. Reference animation tick interval80ms versus idle400ms (`app.rs:308`). Product durations must use monotonic deadlines; draw/event count cannot advance them.

## Compact geometry root cause and required correction

Main draws public `NavList` to register sectioned geometry then `paint_sidebar` overpaints compact geometry (`app.rs:1227-1238`). `NavList::draw` always adds section separator/header rows in Full mode (`crates/tui/src/components/nav_list.rs:753`). Root enabling condition: two independent layout algorithms describe one visible interactive control. Source predicts at 80x24 sidebar y2: reference Buttons y3, main visible Buttons y3, main registered Overview y3. This is NOT yet binary-confirmed.

Reference compact threshold: sidebar height <22+3*2-1 =27; terminal height30 compact,31 expanded. Reference compact source comment mentions a gap but code does not advance y: compact rows contiguous. Do not implement the misleading comment.

Reference-based expected y coordinates (x=4 inside rows): compact y=2+index, clipped at exclusive h-2. At h24 visible indices0..19, Buttons y3. At h30 all22 rows visible, with unused rows24..27 non-activating. Expanded h31: Foundations heading y2, Overview y3, separator y4, Components heading y5, indices1..19 at y6..24, separator y25, Screens heading y26, Settings y27, Task runner y28. Headings/gaps must not activate. At all sizes compare hit results to this independently derived oracle, never candidate registry.

Proposed shared API: orthogonal section policy (always sections / compact / auto-fit), distinct from icon-only NavMode; single keyed row-layout traversal feeds paint, regions, clipping, and focus. Real row renderer receives flags and available rect; permit reference gutter/marker/label anatomy through public parts/renderer instead of second overpaint. Stable PageId-derived ItemKey. Runtime must invalidate presented geometry on resize/routes before incompatible queued input. Test hover/focus/press every visible row, nonactivating gaps, clipped rows, resize both directions at30/31, first input, route change, and deliberate one-cell/compact-policy mutations.

## Busy timing and status

Reference `pages/buttons.rs:43-57`: activation increments clicks, preserves local label-derived last message, sets busy_until=now+2200ms, emits global `Working…`. `:186-202` tick reaches deadline once, clears busy, emits global `Long job finished ✓`, returns changed. Busy control not activatable. Main `:184,202` sets28 frames and decrements every update, replaces local last instead of emitting global status. Same defect remains in original dirty work.

Required API: explicit monotonic now + earliest wakeup request in update; page typed status intent consumed by app-owned transient status/deadline; draw reads clock/state only; active/inactive page ownership specified without introducing lost completions. Tests start t0; busy at2199; complete once at2200 or later;1000 unrelated events at same time cannot finish; idle deadline finishes;1000 draws do not mutate clicks/busy/status/effects; repeated ticks after completion produce no second completion; Working and completed footer visible; subsequent4s expiry semantics preserved; spinner repaint scheduled. Verify local last remains reference label message, not substituted footer text.

## Untrusted original dirty work salvage

Captured exact dirty Showcase diff in `original-untrusted-showcase.patch`; SHA in provenance.json. No source edited. Worth selectively salvaging after shared APIs settle: corrected top-level Esc behavior; focus-dependent page hints/editing trait; restored page content/fixtures in Pickers (three picker kinds) and rich pages; added panel/reference rendering. Must review each patch against pinned reference and public API.

Do not blanket import: busy_frames defect remains; compact sidebar overpaint remains; many legacy_* local paint routines recreate theme/state rendering; hints-only footer still lacks complete status/dialog/EDIT semantics; blanket crate-wide clippy expectations hide lint debt. Existing candidate-derived baselines and source-scanner calls cannot validate parity. Existing tests are substantial but expected hit coordinates often derive from candidate facts and do not substitute independent reference oracles.

## Next acceptance slice

1. Integrator restores compiler; capture real reference and main click(4,3) and Buttons timing before expectations change.
2. Shared NavList section/layout/row-style API and runtime deadline/status support settle with component owner.
3. Migrate shell and Buttons, remove overpaint; run reference-derived coordinate and deadline tests.
4. Migrate remaining pages against source inventory, saving pure-view, runtime-event, terminal artifacts per page/state.
5. Independent hash-bound visual/behavior review. This audit supplies no approval or parity claim.
