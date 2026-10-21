# API boundaries and architecture direction review — 2026-10-22

Reviewer: independent reviewer (read-only). No code authored, no fixes applied.

Reviewed sha (exact): `bea5350ec99b0652335714275fd3f0d078399bca`
Reviewed parent: `a972caaf1d888db8f654f59e545aa0ca7c47e26f`
Stream base (parent of the first stream commit): `5dfff7f20f8adfeb9c0438040ee0b3db58bcc0b1`

Stream under review:

- `ef35a7f6be0d6e2922c5f0796dd3d49c5e243191` — refactor(showcase): paint the chrome page from one responsive layout
- `493f9054f0b23199e677f477b89798195ade4228` — refactor(showcase): paint the task runner and terminal pages live
- `150320ab3169dc81c3eb5b60c2be8a4cd991b2a3` — feat(jackin-preview): live accounts page replaces the frozen accounts frames
- `fc2c3bd4e19029828234cce6352f2074624ac7c2` — refactor(jackin-preview): take the 256 downgrade from the theme ladder
- `a972caaf1d888db8f654f59e545aa0ca7c47e26f` — fix(theme): restore the missing per-family mono rule readers
- `bea5350ec99b0652335714275fd3f0d078399bca` — feat(holla): width-band layout policy and a real history setting
- theme oracle commit in the same history: `ec783f6c50cc11471f66c7030318d33f4578826f` — fix(theme): restore the frozen source text and syntax tokens

---

## Findings

```
crates/holla-sim/src/catalogue.rs:894: 🔴 blocker: the architecture gate fails at the reviewed sha. `python3 tools/acceptance_architecture.py` exits 1 with `architecture_checks_passed: false` and 2 `frame-marker` findings, one of them this line ("The policy is a width band, not a captured frame size: the frozen reference"); the parent `a972caaf1d` exits 0 with 0 findings. The gate regex `captured (?:terminal )?frame` matches the phrase in the new comment. Reword both new comments so the phrase is not a captured-frame marker, or narrow the rule to code, and re-run the gate to 0 findings.
crates/holla-ui/src/app.rs:3908: 🔴 blocker: second `frame-marker` finding, same rule, same cause: the new test comment "The width band carries the failure card at every viewport, not only at the captured frame sizes." Rephrase to avoid `captured frame`.
crates/jackin-preview-app/src/app/accounts_page.rs:1438: 🟠 major: the new accounts page forks shared text measurement. `fn truncate(text: &str, width: usize)` counts `chars()` and cuts at `width-1`; `termrock::text::truncate` (`termrock::text::measure::truncate`) already does grapheme- and cell-width-correct truncation and is exported from the facade. This copy cuts wide CJK glyphs at one cell and breaks grapheme clusters. Delete the local copy and call `termrock::text::truncate`.
crates/jackin-preview-app/src/app/accounts_page.rs:1561: 🟠 major: the new accounts page re-implements scrolling. `visible_rows` (line 1561) and `list_offset` (line 1567) compute the viewport length and the first visible row from the cursor index, duplicating `termrock_collections::ScrollState` (`visible_range`, `follow`) and the `List` component's own `scroll_view`. The page then hands its own `offset`/`total` to `paint_accounts_detail`, which recomputes the thumb from them, so the scrollbar thumb and the row window can disagree with the list's real scroll state. Drive the painter from the list's resolved scroll state instead of a second, app-owned offset.
crates/jackin-preview-app/src/app/accounts_page.rs:32: 🟠 major: the new accounts page hard-codes the shared painter's private geometry. `const LIST_PANE_W: u16 = 39` duplicates `LIST_W` in `crates/termrock-feedback/src/accounts_drawer.rs:130` and `crates/termrock-feedback/src/inspector_body.rs:70`. The comment says "the shared painters own this geometry; the page needs it to cut a row name before the pane edge" — that is exactly the missing shared query. Add a public pane-width accessor on the shared painter (or a content-rect query on the account list) and consume it.
crates/jackin-preview-app/src/app/accounts_page.rs:1553: 🟠 major: the new accounts page duplicates the shared painter's row-slot geometry. `fn name_column` returns 7 or 9 from `AccountRole`, hard-coding `termrock_feedback::accounts_detail::slots` and `accounts_drawer::slots` (`gutter+2`/`gutter+4` and `gutter+4`/`gutter+6`). If the shared painter's slot offsets change, the app's truncation widths silently desynchronise. Expose the name column from the shared painter and use it.
crates/jackin-preview-app/src/app/accounts_page.rs:553: 🟡 minor: `register_add_row` derives the add-account hit region from the tree row index and a hard-coded name offset, not from the resolved layout, and it ignores the list's scroll offset, so the registered control drifts off the painted row when the list scrolls. Let the shared list own the row hit region.
crates/holla-ui/src/layout.rs:1: 🟡 minor: the new `Band` policy is an application-local layout policy. The contract puts layout tools in Termrock (`termrock-layout`, `termrock-controls::MasterDetail::resolve_responsive_layout`, `termrock-layout::responsive_columns`); Termrock already owns the "one breakpoint, one resolved layout" idea. As written, Holla gets a private band enum while `holla-sim/src/catalogue.rs` keeps 20 raw `(80..=160)`-style width predicates and `holla-ui/src/app.rs` keeps 13 more, so the band policy is split across three files in two crates and the two do not agree on the boundaries (`(100..120)` in the catalogue vs `Mid` = 100..=119). Either move the band to `termrock-layout` as a shared, overridable policy or make one owner in the app and drive the catalogue from it.
crates/holla-ui/src/app.rs:378: 🟡 minor: the width-band policy does not cover the app. `parity_history_tag_plate`, `parity_files_tag_plate`, `parity_browser_tag_plate`, `parity_git_current_tag_plate`, `parity_cargo_tag_plate` and `parity_git_batch_tag_plate` still select on `(80..160)`/`(80..120)` ranges directly, and `holla-sim/src/catalogue.rs` uses `(120..160)`/`(100..120)`/`(80..120)` in 20 places. The new `Band` covers 18 sites in `app.rs` and none of these, so the "band instead of captured frame size" rule is applied to part of Holla only. Route the remaining predicates through the same policy.
crates/holla-sim/src/catalogue.rs:850: 🟡 minor: `history_enabled` reads `HOLLA_NO_HISTORY` from the environment inside the simulation crate. Environment-driven behaviour belongs to application startup, not the simulation model, and it makes every Holla test depend on ambient process state. Read the flag in `crates/holla/src/main.rs` (where `HOLLA_NO_MOTION` is already read) and pass it into `App::for_scenario`.
crates/showcase-demos/src/pages/mod.rs:136: 🟡 minor: the new shared `section_strip` duplicates `paint_section_strip` in `crates/showcase-demos/src/pages/chrome.rs:274`, which still exists with its own copy of the same tabs, rule and caption, and 12 more per-page copies exist in `chips.rs`, `dialogs.rs`, `grid.rs`, `lists.rs`, `overview.rs`, `panels.rs`, `pickers.rs`, `progress.rs`, `scrolling.rs`, `selectable.rs`, `sidebars.rs`, `tables.rs`, `trees.rs`. Two places must not implement the same chrome separately. Delete `chrome.rs`'s copy and point every page at the one in `mod.rs`.
crates/showcase-demos/src/pages/mod.rs:139: 🟡 minor: the new `section_strip` hard-codes seven RGB literals (`0x000000`, `0xffffff`, `0xb3b3b3`, `0x737373`, `0x3a3a3a`, `0x48e054`, `0x27272a`) instead of theme tokens, and keeps them alive with `let _ = (CANVAS, WHITE, ...)`. Use `Role`/`FgStep` and the surface tokens; the dead-constant tuple then disappears.
crates/showcase-demos/tests/no_false_parity.rs:16: 🟡 minor: the new false-parity guard matches three literal source strings in three page files. A captured cell table written any other way passes, and the other 14 showcase pages and the Jackin pages are not covered at all. Extend the guard to every page module and match the shape of a table, not three literals.
crates/jackin-preview-app/src/app/accounts_page.rs:154: 🟡 minor: the new accounts hint bar advertises keys that the route does not bind. `ACCOUNT_HINTS` lists "e Edit…", "t Disable", "d Remove…", "m Usage"; the keymap binds `e` to `CMD_EDITOR_OPEN` (handled only on `Route::Manager`), `t` to nothing on this route, `d` to `CMD_CAPSULE_DETACH`, and `m` to `CMD_MANAGER` (Usage is `u`); the remove action is bound to `x`. Bind the advertised keys or drop them from the hint list.
crates/jackin-preview-app/src/app/accounts_page.rs:404: 🟡 minor: the accounts filter is now an inline field painted by the app, replacing the reference's "Filter accounts" dialog. The dialog had Apply plus the standard Esc and outside-click dismissal; the inline field has no dismissal contract of its own, and it lives in the app rather than on the shared overlay layer that owns input scope and cursor behaviour. Use the shared dialog/field overlay.
crates/jackin-preview-app/src/app/accounts_page.rs:553: 🟡 minor: `register_add_row` registers a control for the "+ Add account…" tree row, but the shared `termrock_feedback` painters own the tree rows and their geometry; the app now has a second, independent owner of that row's hit region. Let the shared painter return or register its row regions.
```

---

## Dependency edges (Cargo.toml)

Production `termrock-*` crates depend only on `termrock-*`, `ratatui`, `ratatui-core`, `ratatui-crossterm` and small utility crates. No production `termrock-*` crate depends on an application crate.

Every application-facing edge, production and dev:

| from | to | kind | note |
|---|---|---|---|
| `termrock` (facade) | all `termrock-*` sub-crates | prod | expected |
| `termrock-controls` / `-editors` / `-fields` / `-forms` / `-grid` / `-navigation` / `-overlays` / `-runtime` / `-shell` | `termrock-core`, `-layout`, `-theme`, `-text`, `-render`, `-collections`, `-runtime`, `-controls`, `-fields`, `-feedback`, `-navigation`, `-viewport`, `-overlays` | prod | library-internal, direction correct |
| `termrock-session` | `termrock-core`, `-runtime`, `-terminal`, `-theme` | prod | direction correct |
| `termrock-shell` | `termrock-core`, `-layout`, `-theme`, `-text`, `-render`, `-runtime`, `-controls` | prod | direction correct |
| `termrock-test-support` | `termrock` (feature `testing`) | prod | test-only crate, name says so |
| `termrock-consumer-example` | `termrock` | prod | facade-only, no application reachability |
| `termrock-conformance` | `termrock`, `termrock-test-support`, `showcase-ui`, `jackin-preview-app`, `holla-ui`, `holla-sim`, `tablepro-ui`, `tablepro-demo` | prod | harness crate, not a production library package; the architecture gate does not treat it as one |
| `termrock-e2e`, `termrock-xtask`, `termrock-visibility-*` | (test/tooling) | dev | tooling |

Application crates:

| from | to | note |
|---|---|---|
| `showcase` | `showcase-data`, `showcase-ui`, `termrock-session`, `clap`, `ratatui` | app → library only |
| `showcase-ui` | `showcase-data`, `showcase-demos`, `termrock`, `ratatui` | app → library only |
| `showcase-demos` | `showcase-data`, `termrock`, `ratatui` | app → library only |
| `showcase-data` | `termrock` | **DATA crate → widget/library types**: `use termrock::{ItemKey, TreeNode}` at `crates/showcase-data/src/lib.rs:1`, and `TREE: &[TreeNode]` at line 265. `TreeNode` is a `termrock-navigation` widget type. Inverted edge. |
| `tablepro` | `tablepro-ui` | app only |
| `tablepro-ui` | `tablepro-demo`, `tablepro-domain`, `tablepro-sql`, `termrock`, `ratatui` | app → library only |
| `tablepro-domain` | `tablepro-demo`, `tablepro-sql`, `termrock` | **DATA crate → widget types**: `GridState`, `GridModel`, `ItemKey`, `CompletingCodeEditorState`, `FieldError` (`crates/tablepro-domain/src/tabs.rs:12`, `src/domain.rs:6`, `src/connections.rs:6`, `src/model.rs:6`, `src/grid_model.rs:63`). Inverted edge. |
| `jackin-preview` | `jackin-preview-app`, `termrock` | app only |
| `jackin-preview-app` | `jackin-preview-domain`, `-sim`, `-presentation`, `-host-ui`, `-capsule-ui`, `termrock`, `ratatui` | app → library only |
| `jackin-preview-domain` | `termrock` | **DATA crate → library type**: `termrock::Secret` (`crates/jackin-preview-domain/src/workspace.rs:540`). Inverted edge (Secret is a text value type, not a widget, but the direction is still application → library from a domain crate). |
| `jackin-preview-sim` | `jackin-preview-domain`, `termrock` | **SIM crate → library types**: `Rect`, `Role`, `SplitModel`, `CellPos`, `FgStep`, `StepState` (`crates/jackin-preview-sim/src/pty.rs:3`, `src/changes.rs:3`, `src/launch.rs:1`). Inverted edge. |
| `holla` | `holla-ui`, `termrock` | app only |
| `holla-ui` | `holla-domain`, `-catalog`, `-plan`, `-sim`, `termrock`, `ratatui` | app → library only |
| `holla-domain` | `ratatui`, `termrock-core`, `unicode-segmentation` | **stale edges**: `crates/holla-domain/Cargo.toml:9-10` declare `ratatui` and `termrock-core`, and `crates/holla-domain/src/` uses neither (0 matches for `termrock` or `ratatui`). Drop both dependencies. |

Every inverted edge above predates this stream; the stream neither added nor removed any of them. The stream's only Cargo changes are `crates/termrock-consumer-example` (new, facade-only), `crates/termrock-shell` (+`termrock-test-support` dev-dep) and `crates/termrock` (+`public_component_contracts` test target).

---

## Measured geometry and painting counts

Method: `grep -rn --include='*.rs' -E 'Rect::new|Rect \{'` for raw geometry; `grep -rn --include='*.rs' -E 'ui\.paint_str|ui\.fill\(|buf\[|buffer\[|set_string\(|set_line\('` for direct painting; `grep -rn --include='*.rs' -E 'ui\.raw\('` for correction painters. Two columns are given: **raw** (all of `src/`, test modules included) and **production** (same counts with `#[cfg(test)] mod` blocks stripped). Per-crate deltas are parent → head; the stream-base → head column covers the whole reviewed range.

### parent `a972caaf1d` → head `bea5350ec9` (raw / production)

| crate | rect raw | rect prod | paint raw | paint prod | `ui.raw()` | Δ |
|---|---|---|---|---|---|---|
| holla-ui | 221 → 221 | 210 → 210 | 216 → 216 | 210 → 210 | 0 | none |
| showcase-ui | 39 → 39 | 33 → 33 | 32 → 32 | 32 → 32 | 0 | none |
| showcase-demos | 675 → 675 | 669 → 669 | 400 → 400 | 400 → 400 | 0 | none |
| jackin-preview-app | 190 → 190 | 190 → 190 | 96 → 96 | 96 → 96 | 6 → 6 | none |
| tablepro-ui | 296 → 296 | 295 → 295 | 252 → 252 | 251 → 251 | 0 | none |
| jackin-preview-sim | 2 → 2 | 1 → 1 | 0 → 0 | 0 → 0 | 0 | none |
| jackin-preview-presentation | 4 → 4 | 4 → 4 | 1 → 1 | 1 → 1 | 0 | none |
| all other app crates | 0 → 0 | 0 → 0 | 0 → 0 | 0 → 0 | 0 | none |

**The reviewed head commit changes no application geometry, painting or correction-painter count.** No application crate grew.

### stream base `5dfff7f20f` → head `bea5350ec9` (raw counts, whole stream)

| crate | rect | paint | `ui.raw()` | Δ |
|---|---|---|---|---|
| showcase-demos | 675 → 675 | 414 → 400 | 0 → 0 | paint −14 |
| showcase-ui | 36 → 39 | 32 → 32 | 0 → 0 | rect +3 |
| holla-ui | 214 → 221 | 216 → 216 | 0 → 0 | rect +7 |
| jackin-preview-app | 189 → 190 | 99 → 96 | 6 → 6 | rect +1, paint −3 |
| tablepro-ui | 296 → 296 | 252 → 252 | 0 → 0 | none |

Per-commit attribution (raw counts):

| commit | crate | rect | paint |
|---|---|---|---|
| `ef35a7f6b` | showcase-demos | 669 → 669 | 414 → 412 |
| `493f9054f` | showcase-demos | 669 → 669 | 412 → 400 |
| `150320ab3` | jackin-preview-app | 189 → 190 | 99 → 96 |
| `fc2c3bd4e` | — | none | none |
| `ec783f6c50` | — | none | none |
| `a972caaf1d` | — | none | none |
| `bea5350ec9` | — | none | none |

The +7 rect in holla-ui and the +3 in showcase-ui come from `ee3f63c9ea` (a consolidation commit in the same ancestry, not in the named stream): all of it is inside `#[cfg(test)]` modules (`crates/holla-ui/src/screens/mod.rs` `report_parity`, `crates/showcase-ui/src/app.rs` `shell_layout_contract` / `shell_cache_contract`). Production counts for both crates went **down** over the stream: holla-ui 214 → 210 rect and 216 → 210 paint; showcase-ui 36 → 33 rect. The only production growth in the whole stream is `jackin-preview-app` rect 189 → 190, from `crates/jackin-preview-app/src/app/accounts_page.rs` (4 `Rect::new` sites: `list_area`, the filter field rect, the add-row control rect, and the `paint_filter_field` rect).

Correction painters (`ui.raw()`): 6 in `jackin-preview-app` (`capsule_split_view.rs:606,634`, `app.rs:7155,10271,10290,10309`) at both parent and head; none anywhere else. Unchanged by the stream.

---

## Commands run and results

All run in the read-only worktree `/Users/donbeave/Projects/review-api-20261022` at `bea5350ec99b0652335714275fd3f0d078399bca` (or at the named comparison sha).

| command | result |
|---|---|
| `python3 tools/acceptance_architecture.py` @ `bea5350ec9` | **exit 1**; `architecture_checks_passed: false`; `findings: 2` (both `frame-marker`: `crates/holla-sim/src/catalogue.rs:894`, `crates/holla-ui/src/app.rs:3908`); `dependency_violations: 0`; `unaccounted_inputs: 0`; `errors: 0`; `acceptance_complete: false` |
| `python3 tools/acceptance_architecture.py` @ parent `a972caaf1d` | exit 0; `architecture_checks_passed: true`; 0 findings; 0 dependency violations; 0 unaccounted inputs |
| `python3 tools/test_acceptance_architecture.py` @ `bea5350ec9` | **OK** — 39 tests ran, 0 failed (32.7 s) |
| `git show --stat` on each of the 7 stream commits | see per-commit table above |
| `grep -rn --include='*.rs' -E 'Rect::new\|Rect \{'` per app crate at 3 shas | see counts above |
| `grep -rn --include='*.rs' -E 'ui\.paint_str\|ui\.fill\(\|buf\[\|buffer\[\|set_string\(\|set_line\('` per app crate at 3 shas | see counts above |
| `grep -rn --include='*.rs' -E 'ui\.raw\('` per app crate at 2 shas | 6 in jackin-preview-app, 0 elsewhere, unchanged |
| `for f in crates/*/Cargo.toml` dependency extraction | see dependency-edge table |
| `grep -rn 'termrock\|ratatui' crates/holla-domain/src/` | 0 matches — the declared `ratatui` and `termrock-core` deps are unused |
| `grep -rn -E '\[lints\]' crates/*/Cargo.toml` | present only in application crates (`jackin-preview-app`, `jackin-preview-capsule-ui`, `jackin-preview-domain`, `jackin-preview-host-ui`, `jackin-preview-presentation`, `jackin-preview-sim`, `jackin-preview`, `tablepro-demo`, `tablepro-domain`, `tablepro-sql`, `tablepro-ui`, `tablepro`) — **no `termrock-*` crate has a `[lints]` section** |

### Lint opt-out (question 7)

Every one of the 28 `crates/termrock-*/Cargo.toml` files lacks `[lints] workspace = true`, so the workspace's declared deny set (`clippy::panic`, `clippy::indexing_slicing`, `clippy::unwrap_used`, `clippy::expect_used`, `clippy::todo`, `clippy::unimplemented`, `clippy::dbg_macro`, `clippy::print_stdout`, `clippy::print_stderr`, `clippy::undocumented_unsafe_blocks`, `clippy::mem_forget`, `rust::unsafe_code = "deny"`) is **not bound to any library crate**. The application crates all opt in.

Crates still opting out (all of `crates/termrock-*`): `termrock`, `termrock-collections`, `termrock-conformance`, `termrock-consumer-example`, `termrock-controls`, `termrock-core`, `termrock-e2e`, `termrock-editors`, `termrock-feedback`, `termrock-fields`, `termrock-forms`, `termrock-grid`, `termrock-layout`, `termrock-navigation`, `termrock-overlays`, `termrock-render`, `termrock-runtime`, `termrock-session`, `termrock-shell`, `termrock-terminal`, `termrock-test-support`, `termrock-text`, `termrock-theme`, `termrock-viewport`, `termrock-visibility-publisher`, `termrock-visibility-tests`, `termrock-xtask`.

Panic-surface inside library `src/` at the reviewed sha: 985 `.expect(`, 217 `.unwrap()`, 66 `panic!(`, 4 `unreachable!(`. This is the pre-existing gap G10 in `docs/architecture/termrock-migration-record.md`; the stream does not change it.

---

## Question 5: where does `crates/holla-ui/src/layout.rs` belong?

**In Termrock, or as one owner inside the application — not as it is now.**

The contract says "Raw geometry stays inside components and layout tools" and "Component rendering, internal layout, editing, selection, scrolling and input mechanics live in Termrock". A width-band policy is a layout tool: it decides a screen's composition from the room it has, which is layout, not application composition.

Evidence for Termrock owning it:

- `termrock-layout::responsive_columns` and `termrock-controls::MasterDetail::resolve_responsive_layout` already own "one breakpoint, one resolved layout" for the library. A second, private breakpoint vocabulary in an application crate is the duplication the contract forbids.
- The band's thresholds (80 / 100 / 120) are Holla's own business, so the *policy* belongs in Termrock with the thresholds supplied by the caller — a `BandPolicy`/`Breakpoints` value in `termrock-layout`, not a `pub enum` in `holla-ui`.
- `Band` as written has no override: `Band::of(width)` is a `const fn` over hard-coded thresholds with no builder and no configuration. The contract requires "Every behaviour remains overridable. Nothing an app or user could reasonably want different is hardcoded, and a behaviour that is a standard still keeps an override."

Evidence against a Termrock home being required *now*: the band encodes Holla-specific breakpoints and Holla-specific screen structure, so it is not obviously wrong in the application crate. The defect is that it is *incomplete and split*: `holla-ui/src/app.rs` uses `Band` at 18 sites while `holla-sim/src/catalogue.rs` keeps 20 raw `(80..=160)`/`(100..120)`/`(120..160)` predicates and `holla-ui/src/app.rs` keeps 13 more, with boundaries that disagree (`(100..120)` excludes 120, `Band::Mid` is 100..=119 inclusive). Two implementations of one policy, in two crates, with different answers at width 120.

Resolution: either promote the band to `termrock-layout` as an overridable policy and drive `holla-sim` and the remaining `app.rs` predicates from it, or keep one owner inside Holla and delete the raw ranges. The current state — one new enum plus 48 surviving raw range predicates — satisfies neither.

---

## Question 3: does `accounts_page.rs` duplicate a shared Termrock component?

Yes, in five places, all inside the same new 1657-line file:

1. **Text truncation** — local `truncate` (line 1438) vs `termrock::text::truncate`.
2. **Scrolling** — local `visible_rows` (1561) and `list_offset` (1567) vs `termrock_collections::ScrollState` and `List`'s own `scroll_view`.
3. **Pane width** — local `LIST_PANE_W = 39` (line 32) vs the private `LIST_W` in `termrock-feedback::accounts_drawer` and `inspector_body`.
4. **Row slot geometry** — local `name_column` (1553) vs `termrock_feedback::accounts_detail::slots` / `accounts_drawer::slots`.
5. **Tree row hit regions** — local `register_add_row` (553) vs the shared painter that owns those rows.

The stream's own commit message for `150320ab3` records the deletion of four view modules and the introduction of "the list keeps the keys: the list owner is drawn so the shared painter can paint over it" (`crates/jackin-preview-app/src/app/accounts_page.rs:600`). That comment describes the correct direction — a shared painter owns the rows — but the page then re-derives the painter's private geometry, its private slot offsets, its private pane width and its own scroll window in application code. The gap is not "the accounts page exists"; it is "the accounts page re-owns five things the shared painter already owns".

The 12 shared-component needs the jackin stream listed are recorded in `docs/audits/2026-10-21-jackin-live-cockpit-component-plan.json` (8 `region_mapping` + 4 `input_mapping` entries). The accounts page answers none of them from the shared layer: it supplies view data and typed actions to `paint_accounts_drawer` / `paint_accounts_detail` / `paint_inspector_body` for the *chrome*, but supplies the *rows*, the *selection*, the *scroll window* and the *hit regions* itself.

---

## Question 4: new public APIs — Termrock freedom, documentation, overridability

**`ec783f6c50` (theme oracle commit)** — `crates/termrock-theme/src/builtin/junie.rs` and `crates/termrock-theme/src/palettes.rs`. The theme is the shared layer, so palette values belong there and the change is directionally correct: it restores the frozen source's `BLUE_INFO` keyword, `SYNTAX_STRING`, `SYNTAX_NUMBER` and `DIAGNOSTIC_ERROR` tokens instead of the app-local approximation. No application type is named, no application crate is referenced, and the change is inside `termrock-theme` only.

Two concerns, neither a boundary violation but both a documentation defect:

- `crates/termrock-theme/src/glyph.rs:333` still carries the comment "12600 occurrences of U+2502 in the approved corpus at 120x40 across every colour mode, and no U+2506 track" directly above `assert_eq!(j.design.glyphs.scrollbar().begin, "┆")` — the assertion now asserts the opposite of the comment it sits under, and the comment is the record of the measurement that justifies the value. The comment must be corrected with the value, not left contradicting it.
- `crates/termrock-theme/src/builtin/mod.rs:1203` and `crates/termrock-theme/tests/capability_palettes.rs` changed the *expected* values in the same commit that changed the *actual* values, so the tests now assert the new numbers against themselves and can no longer distinguish a correct restoration from a wrong one. The oracle comparison should be against the frozen source's values, which the commit had available.

**`a972caaf1d` (mono rule readers)** — `crates/termrock-theme/src/downgrade.rs` adds seven `#[cfg(test)]` readers (`viewport_mono_rules` … `button_mono_rules`, lines 701-725) that expose the per-family `MonoRule` tables to the coverage test. Test-only, private to the crate, no application knowledge, no new public API. Clean.

**`150320ab3` (accounts page)** — no new Termrock public API. The page consumes `termrock::feedback::{AccountsDetail, AccountsDrawer, AccountsFilter, ...}` and `termrock::List`. The data it passes is view data (strings, rows, hints); no application object is passed into a library component, and the app returns no typed actions from the shared painter — the painter paints and the app owns the routing, which is the intended split for view data.

**`fc2c3bd4e` (256 downgrade)** — `crates/jackin-preview-app/src/app/launch_dialog_plate.rs` deletes the app-local 19-entry `Color::Rgb → Color::Indexed` table and calls `termrock::theme::downgrade_color`. Correct direction: the ladder lives in the theme crate now. The file's own header documents that the 16-colour and mono remap is still an app-local shim, which is an acknowledged, recorded gap rather than a new one.

**`bea5350ec9` (holla band)** — `crates/holla-ui/src/layout.rs` is a new public module in an application crate. It does not put application knowledge into Termrock (it is not in Termrock at all), it is documented, and `Band::of` is a total `const fn`. Its thresholds are hard-coded and not overridable, and it coexists with 48 other width predicates — see question 5.

**`ef35a7f6b` / `493f9054f` (showcase pages)** — the new showcase pages paint through `termrock::author::PaintStyle` and the shared widgets, and the new `crates/showcase-demos/tests/no_false_parity.rs` adds a real guard. The `author` module is the documented component-author boundary, and using it from a demo crate that exists to demonstrate components is the intended use.

---

## Post-review upstream state (context, not part of the verdict)

The report branch was rebased onto `origin/parity/termrock-visual-baseline` at `d1e3d012612ba452ac9ab7c81fb9e84e6ec3f838` before pushing, as the push protocol requires. That tip is *ahead* of the reviewed sha and already contains `e1722293c6` ("fix(holla): remove frame-marker triggering comments in width-band …"), which rewords the two comments named in the two blocker findings — the gate passes at `d1e3d012` (exit 0). The blockers are therefore stated as measured against the reviewed sha `bea5350ec9`, which is what this review was asked to assess; they no longer reproduce at the current parity tip.

The other findings were re-checked at `d1e3d012` and still stand there: `crates/jackin-preview-app/src/app/accounts_page.rs` still carries its own `truncate` (line 1466), `visible_rows` (1589), `list_offset` (1595), `name_column` (1581) and `LIST_PANE_W = 39` (line 32). Upstream also changed `holla-sim::catalogue::history_enabled` to `is_some_and(|value| value == "0")`, which inverts the flag's sense relative to the reviewed sha; that change is outside this review's scope and is not assessed here.

---

## Verdict

**HELD**, bounded scope: the reviewed sha `bea5350ec9` fails the repository's own architecture gate (exit 1, 2 `frame-marker` findings, both introduced by the reviewed commit `bea5350ec9` itself). The parent `a972caaf1d` passes with 0 findings. That is a hard blocker for the stream as it stands.

Everything else in the reviewed stream moves in the right direction:

- Dependency direction is clean: no production `termrock-*` crate depends on an application crate, and the stream adds none. The four pre-existing inverted edges (application data/sim crates depending on widget types) and the two stale `holla-domain` edges are unchanged and already recorded.
- Application crates did not grow: at the reviewed sha every app crate's raw-geometry, direct-painting and correction-painter count is identical to its parent. Over the whole stream, production geometry and painting counts fall in every application crate except `jackin-preview-app`, where rect sites rise by exactly 1 (four `Rect::new` sites in the new accounts page) while painting sites fall by 3.
- The theme oracle commit and the mono-rule reader commit keep palette logic in `termrock-theme` and add no application knowledge.
- `fc2c3bd4e` moves the 256 downgrade into the theme ladder, deleting an app-local table.

The held items are: the gate failure (blocker), the five places where the new accounts page re-owns geometry, scrolling, truncation, pane width and hit regions that `termrock-feedback` and `termrock-navigation` already own, the split width-band policy in Holla, and the false-parity guard's coverage.

Bounded scope of this review: API boundaries, dependency direction, application thinness (geometry and painting counts), behaviour ownership between application and shared layer, the new public APIs' contract compliance, the placement of the Holla band policy, the architecture gate and the library lint binding. Visual parity, cell-level frame fidelity, theme palette correctness against the frozen oracle, and application behaviour correctness beyond ownership are out of scope and were not reviewed here.
