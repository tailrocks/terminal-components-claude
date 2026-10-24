# Component digest drift — diagnosis, classification and capture-or-revert

Date: 2026-10-22. Branch `wip/diagnose-component-digest-drift-20261021`, based on
`origin/parity/termrock-visual-baseline` at `a5bb410d4`.

## The procedure this diagnosis follows

The panic text in `crates/termrock-test-support/src/digest.rs:277` names §20.10. The
numbered §20.10 table and its ledger live in the pre-split repository, at
`COMPONENT_ARCHITECTURE.md` section `### 20.10 Intentional visual changes` and
`docs/visual-changes.md` ("Visual changes ledger"). Both were removed by the
documentation rewrite (`590dda6cf`, "docs: rebuild unified project documentation
(#36)") and the split branch has no replacement ledger. The decision that
replaces the ledger on this branch is
`docs/architecture/termrock-migration-record.md`:

- **D-008 — Expected digests are a product signal, not an output to bless.**
  `crates/termrock-conformance/tests/baselines/components.txt` is an acceptance
  signal. "The product differences remain work to fix, not output to bless", and
  no pinned frozen capture or documented independent reference decision
  authorises replacing an expected digest. The red component digests are a
  product regression to be fixed in the library. No stream may set `BLESS=1`,
  edit that file, or widen, skip or weaken an assertion.
- The workspace gate section of the same file: "A diagnosis stream owns the
  bisect, the classification and the capture-or-revert decision."

The ledger rules the removed `docs/visual-changes.md` carried still describe what
a capture-or-revert decision needs: the order is change → capture → classify →
bless; no baseline is regenerated because a test failed; every moved key must be
accounted for; a difference is *intended* when it matches the reference, *fix*
when it is a demonstrated defect in the old output, and *regression* when it
must be fixed and never blessed.

**No baseline was re-blessed here.** D-008 forbids it, and the drift contains
both oracle-aligning work and oracle-contradicting work, so a blanket bless
would certify unverified output.

The frozen oracle for this branch is the capture corpus under
`baselines/tuiscotti-v1/` (the pre-refactor reference application) and the
pinned reference source commit `4169783ae883788186d161af09ba9c481bd84e5c`
(`src/theme.rs` and the widgets it drives). `snapshots/` holds captures of the
refactored application and is not an oracle: a capture of the candidate cannot
prove the candidate.

## First commit that changed the rendered output

Bisected with the real test, not by reading:

```text
git bisect start HEAD 032e9faa3
git bisect run <script: cargo nextest run -p termrock-conformance --all-targets \
  -E 'test(/render::components::tree::default/)'>
```

Result: `a848219fd858ed6352fcb403b2cc5d5953083e6f` is the first bad commit
("fix: match tablepro quick-open and tab list to the baseline").

| commit | `render::components::tree::default 120 40 junie truecolor` |
| --- | --- |
| `032e9faa3` (baseline file written) | `94e7ce70bb65b804` (matches the baseline line) |
| `1a947a251` (parent of the culprit) | `94e7ce70bb65b804` (test passes) |
| `a848219f` (culprit) | `68aca379bb96d54f` |
| `ae193b855` (branch tip at the start) | `68aca379bb96d54f` (unchanged since the culprit) |

The rendered change is a colour-only change: the same frame text, different ink.
`a848219f` moved the Junie foreground ladder to the frozen values —
`fg[2]` muted `#808080` → `#8a8a8a`, `fg[3]` faint `#4d4d4d` → `#737373` — and
changed the scrollbar to the frozen track glyph `┆` with the frozen track ink
`#474747` and thumb ink `#808080`.

## Classification of the drift

**Intended.** The pinned reference source `4169783ae` defines `MUTED` as
`rgb(0x8a8a8a)` (`src/theme.rs:528`), the faint ink as `rgb(0x737373)`
(`src/theme.rs:506`), the scroll track ink as `WHITE_28` = `rgb(0x474747)`
(`src/theme.rs:511`, used at `src/theme.rs:750`), the scroll track glyph as `┆`
(`src/theme.rs:2429`) and the handle ladder as
`Ladder::new(WHITE_50, WHITE_70, GREEN)` (`src/theme.rs:792`). The component
baseline was last written at `032e9faa3`; every later theme change moved the
candidate toward the frozen oracle and no later commit re-blessed the baseline.

The drift is not one change. The set of commits between `032e9faa3` and the tip
contains both oracle-aligning work and oracle-contradicting work, and the
baseline was never re-blessed for either.

### The moved set, read from a discarded scratch bless

```text
cp crates/termrock-conformance/tests/baselines/components.txt /tmp/components.before.txt
BLESS=1 cargo nextest run -p termrock-conformance --all-targets --no-fail-fast
diff /tmp/components.before.txt crates/termrock-conformance/tests/baselines/components.txt
git checkout -- crates/termrock-conformance/tests/baselines/components.txt
```

At `ae193b855` (the tip when the diagnosis started) 157 keys moved: 112 at
truecolor, 45 at mono; 127 junie, 30 paper; across 30 component families.

At `a5bb410d4`, after the theme restorations described below, 121 keys still
move, concentrated in:

| keys | component | note |
| --- | --- | --- |
| 24 | `grid` | `crates/termrock-grid` |
| 21 | `code_editor` | `crates/termrock-editors` |
| 14 | `filter_list` | `crates/termrock-navigation` |
| 10 | `scroll_region` | `crates/termrock-runtime`; the frozen scrollbar ink |
| 9 | `help_overlay` | `crates/termrock-overlays` |
| 9 | `context_menu` | `crates/termrock-overlays` |
| 8 | `completion` | `crates/termrock-overlays` |
| 6 | `picker_chain` | `crates/termrock-overlays` |
| 5 | `dialog` | `crates/termrock-overlays` |
| 4 | `menu_bar` | `crates/termrock-overlays` |

Both scratch blesses were discarded; the baseline file is byte-identical to
`032e9faa3` again.

### A reference conflict the owner has to settle

Two acceptance signals disagree about the Junie muted, faint and scrollbar ink.

| signal | muted | faint | scroll track |
| --- | --- | --- | --- |
| `components.txt` (blessed at `032e9faa3` from the refactored tree) | `#808080` | `#4d4d4d` | `│` in the border-subtle ink |
| pinned reference source `4169783ae` and the frozen corpus | `#8a8a8a` | `#737373` | `┆` in `#474747` |

The oracle side is backed by the pinned source (`src/theme.rs:528`, `:506`,
`:511`, `:750`, `:2429`) and by the frozen captures; the audits
`docs/audits/2026-10-20-library-reference-token-oracles.md` and
`docs/audits/2026-10-20-note-muted-baseline-reference.md` record the same
conclusion and the corpus bytes that carry it.

`b4b1a3346` ("fix(theme): restore the frozen muted and faint inks and the
scrollbar track glyph") chose the component baseline, following D-008. That
choice trades 136 conformance digest failures for eight theme test failures:
`termrock-theme::capability_palettes all_73_junie_slots_match_reference_capability_output`,
six `termrock-theme::empty_style_inheritance` cases and
`termrock-theme::theme_holla_states nested_public_layers_dim_the_composited_origin_and_preserve_unwritten_cells`
now fail, because they encode the oracle inks (`138`, `┆`).

D-008 says that "no pinned frozen capture or documented independent reference
decision authorises replacing an expected digest". For these keys there is
both: the corpus and the two audits. The decision to re-bless only the keys
whose movement the corpus and the audits account for, and to leave every other
key to the product fix, is the owner's to make and to record. No stream may
take it unilaterally, and this stream re-blessed nothing.

## Corrections

The theme, overlay and editor corrections below were found by this stream and
landed on the parity tip while the diagnosis was running: `849d26dad` carries
the theme and overlay corrections, and the caret correction is on the tip as
well. What this branch adds is the documentation inventory correction and this
record.

### 1. SQL syntax ink contradicted the frozen corpus — corrected upstream

`707b12c76` ("fix(tablepro): match the completion menu and SQL ink") and
`819ef8ef9` ("fix(tablepro): match query error, results, and explain frames")
moved the Junie syntax tokens to colours that are not in the frozen corpus:
keyword `#79b8ff` (blue), string `#8bd5ca` (teal), number `#c6a0f6` (purple),
diagnostic error `#f05d5d`, operator and punct `#8a8a8a`.

The frozen corpus paints SQL keywords bold white, the operator glyph in
`#808080`, strings and numbers in `#b3b3b3`, and the error ink in `#e44545`:

```text
baselines/tuiscotti-v1/tablepro/query/explain/120x40/truecolor.ansi
  [1;4;38;2;255;255;255] 'SELECT'   [4;38;2;128;128;128] '*'   [1;4;38;2;255;255;255] 'FROM'
baselines/tuiscotti-v1/tablepro/query/completion/120x40/256.ansi
  [1;4;38;5;231] 'SELECT'   [4;38;5;244] '*'   [1;4;38;5;231] 'FROM'
baselines/tuiscotti-v1/showcase/editor/default/120x40/truecolor.ansi
  [1;38;2;255;255;255] 'let'   [38;2;179;179;179] '200'   [38;2;179;179;179] '"attempt {attempt} '
```

The authored ANSI-256 table was never updated with them, so the reduced tiers
disagreed with the truecolor tier and
`termrock-theme::capability_palettes all_73_junie_slots_match_reference_capability_output`
failed. Restored in `crates/termrock-theme/src/builtin/junie.rs`; the same
correction landed upstream as `849d26dad`.

### 2. The menu bar title lost its public variant identity — corrected upstream

`84d6477d7` ("fix(overlays): resolve menu bar title style through
MENU_BAR_TITLE_VARIANT") resolved the title under the private
`termrock.menu-bar-title` variant instead of the public `MENU/DEFAULT/TITLE`
identity. `docs/audits/2026-10-20-menubar-hover.md` states the contract: "Title
defaults come from a private recipe variant, then resolve under the public
`MENU/DEFAULT/TITLE` identity with `Ui::style_defaults`, preserving family
default variants, recipe overrides, overlays, and local part patches." The
private identity drops the family default variant and the overlay background, and
`menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence`
failed on the overlay background. Restored in `crates/termrock-overlays/src/menu.rs`;
the same correction landed upstream as `849d26dad` and is recorded in
`docs/audits/2026-10-22-theme-menu-recipe-decision.md` ("Follow-up correction:
`849d26dad`").

### 3. A retained edit session asked for a cursor it cannot have — corrected upstream
`819ef8ef9` changed the caret rule to `st.editing || (read_only && focused)`,
dropping the focus requirement. An editor that keeps its edit session after a
blur is still `editing` but is not focused, so it requested a cursor the cursor
resolver rejects, and both editor tests failed with
`CursorRejected { owner: completion.composite.editor, .. }`. The caret is
requested only for a focused editor again, in
`crates/termrock-editors/src/code.rs`; the correction is on the parity tip.

### 4. Two documentation examples were not registered — corrected here

`docs/architecture/termrock-coordinator-plan.md` held an unlabelled shell block
and `docs/audits/2026-10-21-session-typed-input.md` held a `rust` block that
quotes a removed test. The documentation inventory treats both as unregistered
Rust examples. The shell block is labelled `bash` on the parity tip; the
retained evidence is quoted as `text` here, because the helper it calls no
longer exists and it can never be a compiled example.

## The other named failures

| failure | root cause | verdict |
| --- | --- | --- |
| `termrock-xtask::ownership ownership_check_passes_on_workspace` | 103 ownership findings, all in `crates/holla-sim`, `crates/holla-ui` and `crates/jackin-preview-app` | not in the crates this branch owns; other agents' in-flight work |
| `termrock-xtask::doc_inventory tracked_workspace_documentation_has_no_unregistered_examples` | two unregistered Rust examples in `docs/` | fixed here |
| `termrock-overlays menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence` | `84d6477d7` broke the public `MENU/DEFAULT/TITLE` identity | fixed upstream in `849d26dad` (regression) |
| `termrock-navigation::item_row_columns constructor_defaults_match_the_pre_policy_full_cell_fixture` | `4d6ad4012` paints a space for an empty item glyph in the shared item row, so the historical full-cell fingerprint no longer matches | open: the change is deliberate for the jackin picker, the fixture is historical; the fixture needs a reviewed re-record, or the space belongs in the picker only |
| `termrock-editors::completing_code_editor focus_loss_dismisses_completion_but_preserves_configured_edit_session` | `819ef8ef9` asked for a caret from an unfocused editor that keeps its session | fixed upstream (regression) |
| `termrock-editors code::tests::focus_out_commits_by_default_but_can_preserve_the_edit_session` | same cause | fixed upstream (regression) |
| `termrock-theme::capability_palettes all_73_junie_slots_match_reference_capability_output` | `707b12c76` and `819ef8ef9` moved the syntax tokens off the corpus values while the authored ANSI-256 table kept them | fixed upstream in `849d26dad` (regression) |
| `visual-harness::visual_baseline determinism::sources_are_deterministic` | new `Instant::now` uses in `crates/tablepro-domain/src/tabs.rs` and `crates/termrock-core/src/typeahead.rs`, and a new `target_os` use in `crates/termrock-core/src/platform.rs`, are not in the harness allowlist | open: the allowlist update needs a review of crates this branch does not own |
| `visual-harness::visual_baseline overlay_sites` | new overlay sites in `crates/jackin-preview-app` and `crates/tablepro-ui` are not in the harness inventory | open: not in the crates this branch owns |
| `termrock-conformance scroll_region_fixture_exposes_the_complete_bar_at_both_matrix_sizes` | `a848219f` strips bold from the scrollbar track and thumb; the pressed thumb must keep the bold that carries its state in mono | open: the strip must not remove the bold the style itself sets |

## What is still open

- 121 component keys still differ from the acceptance baseline. Each cluster
  above needs its own capture-or-revert decision against the frozen oracle.
- The muted, faint and scrollbar conflict between the component baseline and
  the oracle is an owner decision; `b4b1a3346` currently resolves it in favour
  of the baseline and leaves eight theme tests red.
- The per-application baselines (`showcase`, `holla`, `jackin`, `tablepro`) drift
  with the same theme work and live in crates other agents own.
- The conformance failures that are not digest failures
  (`local_override_does_not_mutate_the_theme` in 40 components,
  `bindings_match_handled_keys`, `mono_states_are_distinguishable`,
  `hover_does_not_steal_focus`, `declared_parts_are_the_parts_actually_styled`)
  are a separate class and were not diagnosed here.

## Verification on this branch

```text
cargo nextest run -p termrock-conformance --all-targets --no-fail-fast
  1462 tests run: 1290 passed, 172 failed, 616 skipped
cargo nextest run -p termrock-theme -p termrock-overlays -p termrock-navigation \
  -p termrock-editors -p termrock-controls -p termrock-runtime -p termrock-xtask \
  --all-targets --no-fail-fast
  1330 tests run: 1320 passed, 10 failed, 3 skipped
cargo nextest run -p visual-harness --all-targets --no-fail-fast
  20 tests run: 18 passed, 2 failed, 28895 skipped
cargo clippy --all-targets -p termrock-theme -p termrock-overlays -p termrock-editors
  no issues
cargo fmt --check -p termrock-theme -p termrock-overlays -p termrock-editors
  clean
```

The ten failures of the second command are the eight theme cases the upstream
muted/faint restoration leaves red, the navigation fixture above, and the
ownership check over crates this branch does not own.

