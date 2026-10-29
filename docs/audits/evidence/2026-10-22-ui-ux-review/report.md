# UI/UX and interaction review — 2026-10-22

Reviewer: independent reviewer (read-only on the reviewed commits; this branch
carries only this report).

Reviewed shas (exact):

- `ef35a7f6be0d6e2922c5f0796dd3d49c5e243191` — refactor(showcase): paint the chrome page from one responsive layout
- `493f9054f0b23199e677f477b89798195ade4228` — refactor(showcase): paint the task runner and terminal pages live
- `849d26dad78477a7bb0e78a68d59794af4fca81f` — fix(theme,overlays): restore frozen SQL syntax ink and menu title identity
- `150320ab3169dc81c3eb5b60c2be8a4cd991b2a3` — feat(jackin-preview): live accounts page replaces the frozen accounts frames
- `fc2c3bd4e19029828234cce6352f2074624ac7c2` — refactor(jackin-preview): take the 256 downgrade from the theme ladder
- `bdf62e3beaa3320ca200b30d82f638bdeee70cff` — docs: record the relocation and digest-blessing decisions (context only)

Note on the task brief: `fc2c3bd4e` is titled "refactor(jackin-preview): take the
256 downgrade from the theme ladder". The accounts-page commit is `150320ab3`.
The brief's label for `fc2c3bd4e` ("merge(jackin): land the live accounts page
over the frozen accounts frames") names `150320ab3` instead.

Base for attribution: `5dfff7f20` (parent of `ef35a7f6b`). Chain:
`5dfff7f20 → ef35a7f6b → 493f9054f → 849d26dad → 8800b2eaa → bdf62e3be → 150320ab3 → fc2c3bd4e`.

New files under review: `crates/showcase-demos/tests/no_false_parity.rs`,
`crates/jackin-preview-app/tests/accounts_page.rs`.

---

## Findings

```
crates/showcase-demos/src/pages/terminal.rs:237: 🔴 blocker: the stored scenario showcase::pages::terminal can no longer be captured. The launch advances one stage per update() call, so the settled frame needs ~36 updates, and the page implements no seek_paused; the harness's stable-frame wait stops the clock before stage 7. Five of five manual captures and the harness trial itself fail with "wait:✓ Ready timed out ... never appeared" after 60.6s, while the base captures the same frame in 6.0s. Drive the launch from a seekable deterministic frame counter (implement seek_paused) or settle on the first tick and paint the finished state.
crates/jackin-preview-app/src/app/accounts_page.rs:1: 🔴 blocker: the live accounts page breaks three stored, reapproved frames that the base matched exactly. jackin::accounts::detail::120x40::truecolor, jackin::accounts::drawer::120x40::truecolor and jackin::accounts::filter::120x40::truecolor all go PASS → FAIL (946, 936 and 201 differing cells). The harness run of the same seven stored scenarios reports 3 passed / 4 failed on the base and 0 passed / 7 failed on the reviewed tip. Reconcile the live page with the approved frames, or hold the change.
crates/showcase-demos/src/pages/taskrunner.rs:1: 🔴 blocker: the live task runner regresses the stored frame at every size. 120x40 truecolor goes from 23 differing cells to 1125, 160x50 from 23 to 1605, 72x20 from 18 to 333. The new page drops the panel rule and paints the "Targets" title bold white where the approved frame has it secondary. Match the approved frame before landing.
crates/showcase-demos/src/pages/chrome.rs:1: 🟠 major: the claim "Every approved frame (72x20, 80x24, 100x30, 120x40, 160x50) now renders from the live widgets" does not hold. The live page still differs from the approved frame by 18 cells (72x20, 80x24, 100x30) and 23 cells (120x40, 160x50) in truecolor, and by about 2400 cells at 256, 16 and none. The differing cells are the rule under the shared demo frame's "Components" header: 0x262626 in the candidate, 0x3a3a3a in the approved frame. The rewrite is byte-identical to the base (0 cells changed), so it neither fixes nor worsens that divergence. Fix the rule ink and re-measure all five sizes and all five colour modes before claiming a match.
crates/jackin-preview-app/src/app/accounts_page.rs:154: 🟠 major: the new hint bar advertises keys that do not perform the advertised action. "e Edit…" is bound to CMD_EDITOR_OPEN, which is handled only on the Manager route, so e does nothing here; "t Disable" is not bound on the accounts route at all; "d Remove…" is bound to CMD_CAPSULE_DETACH, while the remove action is bound to x (CMD_ACCOUNT_REMOVE); "m Usage" is bound to CMD_MANAGER, so m navigates to the Manager (Usage is u). The reference binds e, t, d and m to edit, toggle, remove and usage. Bind the advertised keys or change the hint text.
crates/termrock-theme/src/builtin/junie.rs:76: 🟠 major: the truecolor syntax tokens were changed on the stated premise that "the frozen corpus paints SQL keywords bold white". The frozen oracle does the opposite: the approved truecolor frames paint SQL and Rust keywords bold blue 0x79b8ff (74 keyword cells across the corpus, for example snapshots/showcase/pages/codeeditor/120x40/truecolor.ansi and snapshots/tablepro/query/explain/120x40/truecolor.ansi), the 256 frames paint Indexed(111) and the 16 frames LightBlue, which are exactly the values of the frozen baseline source at tag visual-baseline (src/theme.rs: syntax_keyword: BLUE_INFO, i(111), LightBlue). The change moves keyword to 0xffffff, string to 0xb3b3b3, number to 0xb3b3b3, operator and punct to 0x808080 and diagnostic_error to 0xe44545, and because the ANSI-16 palette is derived from the truecolor tokens it also moves the 16-colour keyword from LightBlue to White. Revert the truecolor syntax tokens to the baseline values. Separately, the authored ANSI-256 table (crates/termrock-theme/src/palettes.rs, keyword 231) already disagrees with the baseline (111) and needs its own review.
crates/jackin-preview-app/src/app/launch_dialog_plate.rs:131: 🟠 major: taking the shared 256 ladder changes four inks in the dialog card: 0x4d4d4d 242→238, 0x262626 238→235, 0x19191c 232→233, 0xff6b6b 203→210. The approved 256 frames carry 242 for the card border (snapshots/jackin/cockpit/cancel_confirm/*/256.ansi), so the frames are 139 cells worse at 72x20, 120x40 and 160x50, and the commit message does not call out a visual change. Keep the corpus-true index for the faint ink, or re-bless with an explicit visual-change note and independent review.
crates/jackin-preview-app/src/app/accounts_page.rs:553: 🟡 minor: the "+ Add account…" hit region is computed from the tree row index without the list's scroll offset (top + index), so once the tree scrolls the control is registered off-screen or not at all and a click on the painted row hits nothing. Derive the control rect from the painted row or let the List own the region.
crates/jackin-preview-app/src/app/accounts_page.rs:1580: 🟡 minor: the filtered pane title is truncated one character later than the reference ("Accounts · filter…" against the approved "Accounts · filt…"), because the truncation room comes from the LIST_PANE_W constant instead of the painted panel width. Truncate to the painted title width.
crates/jackin-preview-app/src/app/accounts_page.rs:226: 🟡 minor: the tree always appends the "Unsupported" row and the "+ Add account…" row. The reference omits both while a filter is active, and the approved filter frame shows only the matching rows. Skip both rows when the filter query is non-empty.
crates/jackin-preview-app/src/app/accounts_page.rs:463: 🟡 minor: with the drawer open the header crumb, status and running count are blanked ("The open drawer keeps the header bare: the panel owns the row"), but the approved drawer frame keeps "Accounts › Claude › Work  ⠋ refreshing 1  inside the Construct  ● 2 running" in row 0. The change is visible in the frame and is not called out. Keep the header, or call the visual change out and re-bless.
crates/jackin-preview-app/src/app/accounts_page.rs:470: 🟡 minor: the header drops the "●" glyph before the running count (candidate "2 running", approved "● 2 running") and the inspector panel loses the separator rule under its title. Both are part of the frame reconciliation, not separate work.
crates/jackin-preview-app/src/app.rs:4095: 🟡 minor: after a filter query is committed the list stops taking keys. update_accounts returns early while self.accounts.filtering is true (app.rs:3915), so the list never gets its update, and the new && !self.accounts.filtering guard means Enter cannot open the drawer; only Esc clears the state, and Esc also discards the committed query. Clear filtering when a non-empty query is committed.
crates/jackin-preview-app/src/app/accounts_page.rs:404: 🟡 minor: the filter is an inline field drawn on the blank row under the header, replacing the reference's "Filter accounts" dialog. The dialog had an Apply action and the standard Esc and outside-click dismissal, which the inline field has no equivalent of. Keep the dialog, or give the inline field the same dismissal contract.
crates/showcase-demos/src/pages/mod.rs:134: 🟡 minor: the new shared section_strip keeps seven unused colour constants alive with let _ = (CANVAS, WHITE, SECONDARY, FAINT, RULE, ACCENT, TAB_ACTIVE); and hard-codes the inks as RGB instead of using the theme tokens. The same strip is duplicated in chrome.rs (paint_section_strip). Use the theme tokens, drop the dead constants, and keep the strip in one place.
crates/showcase-demos/tests/no_false_parity.rs:16: 🟡 minor: the guard greps three literal strings in three page files, so a captured table written any other way passes, and the jackin pages and the other showcase pages are not covered at all. Extend the guard to every page module and to the shape of the table, not to three literals.
crates/jackin-preview-app/src/app/launch_dialog_plate.rs:9: ⚪ nit: the file states that the 16-colour and mono remap is still an app-local shim, and the file remains a recolour painter over a hard-coded 66x11 card rectangle. AGENTS.md puts the rule in the shared theme. Move the reduced ladder into termrock-theme and drop the app-local table.
crates/showcase-demos/src/pages/chrome.rs:970: ⚪ nit: the page paints its own scrollbar (thumb "┃"/"━", track "┆"/"┄") with hard-coded inks and no rest, hover or pressed states, so the page cannot demonstrate the shared scrollbar contract it is a showcase of. Use the shared scrollbar or mark the mock as an illustration.
```

---

## Per-screen fidelity

Measurements are cell-level differences against the approved frames in
`/Users/donbeave/Projects/frozen-baseline-coord-20261021/source/snapshots/`
(the harness store; `baselines/oracle/ORACLE.toml` is `pending = true`, so the
gate reads `snapshots/`). "base" is the same capture taken with the `5dfff7f20`
binaries, to attribute the difference to the reviewed commits.

| screen | size | colour mode | matches the frozen frame | size of the difference | base for comparison |
|---|---|---|---|---|---|
| showcase/pages/chrome | 72x20 | truecolor | no | 18 cells, 1 row | 18 cells, 1 row |
| showcase/pages/chrome | 80x24 | truecolor | no | 18 cells, 1 row | 18 cells, 1 row |
| showcase/pages/chrome | 100x30 | truecolor | no | 18 cells, 1 row | 18 cells, 1 row |
| showcase/pages/chrome | 120x40 | truecolor | no | 23 cells, 1 row | 23 cells, 1 row |
| showcase/pages/chrome | 160x50 | truecolor | no | 23 cells, 1 row | 23 cells, 1 row |
| showcase/pages/chrome | 120x40 | 256 | no | 2478 cells, 38 rows | 2478 cells, 38 rows |
| showcase/pages/chrome | 120x40 | 16 | no | 2421 cells, 36 rows | 2421 cells, 36 rows |
| showcase/pages/chrome | 120x40 | none | no | 2424 cells, 37 rows | 2424 cells, 37 rows |
| showcase/pages/chrome | 120x40 | nocolor | no | 6 cells, 1 row | 6 cells, 1 row |
| showcase/pages/taskrunner | 72x20 | truecolor | no | 333 cells, 12 rows | 18 cells, 1 row |
| showcase/pages/taskrunner | 120x40 | truecolor | no | 1125 cells, 32 rows | 23 cells, 1 row |
| showcase/pages/taskrunner | 160x50 | truecolor | no | 1605 cells, 42 rows | 23 cells, 1 row |
| showcase/pages/taskrunner | 120x40 | 256 | no | 1399 cells, 37 rows | 3481 cells, 38 rows |
| showcase/pages/terminal | 72x20 | truecolor | not capturable | capture times out | 18 cells, 1 row |
| showcase/pages/terminal | 120x40 | truecolor | not capturable | capture times out (harness FAIL at 60.6s) | 23 cells, 1 row |
| jackin/accounts/detail | 72x20 | truecolor | no | 492 cells, 16 rows | 1382 cells, 20 rows |
| jackin/accounts/detail | 120x40 | truecolor | no | 946 cells, 22 rows | 0 cells (harness PASS) |
| jackin/accounts/detail | 160x50 | truecolor | no | 2067 cells, 48 rows | 7703 cells, 50 rows |
| jackin/accounts/drawer | 72x20 | truecolor | not capturable | capture times out | capture times out |
| jackin/accounts/drawer | 120x40 | truecolor | no | 936 cells, 21 rows | 0 cells (harness PASS) |
| jackin/accounts/drawer | 160x50 | truecolor | no | 2072 cells, 47 rows | capture failed |
| jackin/accounts/filter | 72x20 | truecolor | not capturable | capture times out | capture times out |
| jackin/accounts/filter | 120x40 | truecolor | no | 201 cells, 6 rows | 0 cells (harness PASS) |
| jackin/accounts/filter | 160x50 | truecolor | no | 2747 cells, 48 rows | capture failed |
| jackin/cockpit/cancel_confirm | 72x20 | 256 | no | 1207 cells, 19 rows | 1068 cells, 19 rows |
| jackin/cockpit/cancel_confirm | 120x40 | 256 | no | 4508 cells, 39 rows | 4369 cells, 39 rows |
| jackin/cockpit/cancel_confirm | 160x50 | 256 | no | 7632 cells, 49 rows | 7493 cells, 49 rows |
| jackin/cockpit/cancel_confirm | 120x40 | truecolor | yes | 0 cells | 0 cells |
| showcase/flows/terminal/mid | 120x40 | truecolor | no | 2453 cells, 38 rows | 2475 cells, 38 rows |
| showcase/flows/taskrunner/running | 120x40 | truecolor | no | 2573 cells, 38 rows | 2004 cells, 38 rows |

The approved frame for `showcase/pages/chrome` and every other showcase page
also carries a left navigation pane that the candidate no longer renders, so
the showcase suite is broadly red before the reviewed commits; the numbers above
are the honest per-screen deltas, not a clean-sheet verdict.

---

## Interaction findings

| area | screen / control | finding | severity |
|---|---|---|---|
| hint bar keys | jackin accounts, ACCOUNT_HINTS | "e Edit…", "t Disable", "d Remove…", "m Usage" are advertised but do not do that: e is unhandled on this route, t is unbound, d is bound to the capsule detach and the remove action is x, m navigates to the Manager while Usage is u | major |
| keyboard reachability | jackin accounts, list after a committed filter | the list stops receiving keys because update_accounts returns early while filtering is set, and the new guard on the list Chose action blocks Enter; only Esc clears it, and Esc discards the committed query | minor |
| hit region | jackin accounts, "+ Add account…" row | the registered control rect is computed from the row index without the scroll offset, so it does not follow the painted row | minor |
| dismissal | jackin accounts, filter | the reference's "Filter accounts" dialog (Apply, Esc, outside click) is replaced by an inline field with no dismissal contract of its own | minor |
| reachability of the settled state | showcase terminal page | the stored frame needs ~36 update cycles and no seek_paused, so the approved frame is unreachable; 5 of 5 captures time out | blocker |
| focus indicator | jackin accounts tree | one "▎" only: the List is drawn first and the shared accounts painter paints over it, so the tree cursor row carries the single focus bar (approved frames show "▎     Work"); the drawer state moves the bar to the panel title as approved | none found |
| selection vs focus | jackin accounts | selection keeps its accent fill; the cursor row is marked with the bar, matching the approved frames | none found |
| responsive layout | jackin accounts | the wide/narrow switch is area-based (width 100, height 24), not an exact page size; 72x20 and 80x24 use the narrow band and 120x40 and 160x50 the wide one | ok |
| deterministic frames | showcase chrome | three consecutive captures of the same case are byte-identical (no spinner flakiness) | ok |
| key handling | showcase chrome | the brand is a live Brand control and the menu labels are drawn text; the context menu and hover cases in the harness are pending, not stored | not assessed |

---

## Commands run

```
# worktrees (read-only for the review)
git -C /Users/donbeave/Projects/terminal-components-claude worktree add /Users/donbeave/Projects/review-uiux-20261021 --detach fc2c3bd4e
git -C /Users/donbeave/Projects/terminal-components-claude worktree add /Users/donbeave/Projects/review-uiux-base-20261021 --detach 5dfff7f20

# candidate and base binaries (private target dirs)
cd /Users/donbeave/Projects/review-uiux-20261021 && CARGO_TARGET_DIR=/Users/donbeave/Projects/review-uiux-20261021-target cargo build --locked -p showcase -p tablepro -p jackin-preview -p holla --bins
cd /Users/donbeave/Projects/review-uiux-base-20261021 && CARGO_TARGET_DIR=/Users/donbeave/Projects/review-uiux-base-target cargo build --locked -p showcase -p tablepro -p jackin-preview -p holla --bins
# both: "Finished dev profile ... 121 crates compiled"

# frozen harness, the seven stored scenarios of the touched screens
cd /Users/donbeave/Projects/frozen-baseline-coord-20261021/source
VISUAL_FULL=1 VISUAL_REVIEW=0 VISUAL_BIN_DIR=<bin dir> CARGO_TARGET_DIR=/Users/donbeave/Projects/review-uiux-20261021-harness-target \
  cargo nextest run --locked --manifest-path Cargo.toml -p junie-tui --test visual_baseline --run-ignored only \
  --filter-expr 'test(showcase::pages::terminal::120x40::truecolor) or test(showcase::pages::taskrunner::120x40::truecolor) or test(showcase::pages::chrome::120x40::truecolor) or test(jackin::accounts::detail::120x40::truecolor) or test(jackin::accounts::drawer::120x40::truecolor) or test(jackin::accounts::filter::120x40::truecolor) or test(jackin::cockpit::cancel_confirm::120x40::256)' --no-fail-fast

# reviewed tip (fc2c3bd4e)
#   Summary [60.636s] 7 tests run: 0 passed, 7 failed, 28908 skipped
#   FAIL showcase::pages::taskrunner::120x40::truecolor
#   FAIL showcase::pages::chrome::120x40::truecolor
#   FAIL jackin::cockpit::cancel_confirm::120x40::256
#   FAIL jackin::accounts::filter::120x40::truecolor
#   FAIL jackin::accounts::detail::120x40::truecolor
#   FAIL jackin::accounts::drawer::120x40::truecolor
#   FAIL showcase::pages::terminal::120x40::truecolor   (60.605s, "wait:✓ Ready timed out")

# base (5dfff7f20), same filter
#   Summary [6.666s] 7 tests run: 3 passed, 4 failed, 28908 skipped
#   PASS jackin::accounts::filter::120x40::truecolor
#   PASS jackin::accounts::detail::120x40::truecolor
#   PASS jackin::accounts::drawer::120x40::truecolor
#   FAIL showcase::pages::chrome::120x40::truecolor
#   FAIL showcase::pages::terminal::120x40::truecolor
#   FAIL showcase::pages::taskrunner::120x40::truecolor
#   FAIL jackin::cockpit::cancel_confirm::120x40::256

# per-case capture and cell-level diff against the approved frames
VISUAL_TOOL=capture VISUAL_NAME=<case> VISUAL_BIN_DIR=<bin dir> \
  /Users/donbeave/Projects/review-uiux-20261021-harness-target/debug/deps/visual_baseline-fa4aa9cef9b2fe88
# 30 cases for base and reviewed tip, plus the terminal page 5x with the reviewed tip
#   (5/5 timeouts) and 3x with the base (3/3 captures, 25950 bytes)

# full showcase suite with the reviewed tip
VISUAL_FULL=1 VISUAL_REVIEW=0 VISUAL_BIN_DIR=... cargo nextest run ... --filter-expr 'test(/^showcase::/)' --no-fail-fast
#   started, still running after 1h45m on a loaded shared machine; no result is claimed for it

# determinism
#   three captures of showcase/pages/chrome/120x40/truecolor: identical md5 (b04f14e4c4fd4e437e417834852a566e)

# oracle cross-check of the theme finding
git show visual-baseline:src/theme.rs | grep -n 'syntax_keyword'
#   772: syntax_keyword: BLUE_INFO   961: self.syntax_keyword = i(111);   1031: self.syntax_keyword = LightBlue;
git show fc2c3bd4e:crates/termrock-theme/src/builtin/junie.rs | sed -n '70,100p'
#   keyword: WHITE, string: WHITE_70, number: WHITE_70, operator: WHITE_50, punct: WHITE_50
```

Measurement caveats: the harness store under
`/Users/donbeave/Projects/frozen-baseline-coord-20261021/source/snapshots/` is
the store the gate reads, and it agrees with the frozen baseline source at tag
`visual-baseline`. The repository's `baselines/tuiscotti-v1/` is a different,
older admission (admission record: 2026-10-03, `legacy_replayed_conversion`)
and disagrees with both, so it is not used as the oracle here. The harness
writes its `actual` tree into a shared cache under
`/Users/donbeave/Projects/frozen-baseline-coord-20261021/source/target/`, which
other sessions were writing to at the same time; every number in this report
comes from a per-case capture written by this review, not from that tree.

---

## Verdict

**HELD.**

Bounded scope: the four reviewed commits on
`wip/review-ui-ux-20261021` (branched from `fc2c3bd4e`). The hold covers the
showcase chrome, task runner and terminal pages, the Jackin accounts page, the
Junie syntax tokens and the Jackin launch-dialog 256 ladder.

The three blockers are independent of each other and each one alone keeps the
delivery out: the terminal page makes a stored frame unreachable, the live
accounts page turns three frames that matched exactly into mismatches, and the
live task runner regresses a stored frame at every size. The two major theme
findings also need a decision before landing, because both change pixels
against the frozen oracle and neither is called out as a visual change.

Not reviewed: the Holla, TablePro and Jackin screens outside the touched files,
the 40x12 and 60x18 pending sizes, the pending harness cases, and the
application-level parity gates.
