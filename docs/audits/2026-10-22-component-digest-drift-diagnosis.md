# Component digest drift — diagnosis and classification

Date: 2026-10-22. Branch `wip/diagnose-component-digest-drift-20261021`, based on
`origin/parity/termrock-visual-baseline` at `ae193b855`.

## The procedure this diagnosis follows

The panic text in `crates/termrock-test-support/src/digest.rs:277` names §20.10. The
numbered §20.10 table and its ledger live in the pre-split repository, at
`COMPONENT_ARCHITECTURE.md` section `### 20.10 Intentional visual changes` and
`docs/visual-changes.md` ("Visual changes ledger"). Both were removed by the
documentation rewrite (`590dda6cf`, "docs: rebuild unified project documentation
(#36)") and the split branch has no replacement ledger.

The ledger states the rule this diagnosis applies:

- **Order, fixed:** change → capture → classify → bless.
- **No baseline is regenerated because a test failed; the classification comes
  first.**
- Every baseline key the diff moved must appear in the `moved:` field of a ledger
  entry, read from a scratch bless that is then discarded, and every entry names
  a numbered §20.10 item plus the reviewable artefact.
- A difference is *intended* when it matches the §20.10 item, *fix* when it is a
  demonstrated defect in the old output, and *regression* when it must be fixed
  and never blessed.

The frozen oracle for this branch is the capture corpus under
`baselines/tuiscotti-v1/` (pre-refactor reference application) and the pinned
reference source commit `4169783ae883788186d161af09ba9c481bd84e5c` (`src/theme.rs`
and the widgets it drives). `snapshots/` holds captures of the refactored
application and is not an oracle: a capture of the candidate cannot prove the
candidate.

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
| `ae193b855` (branch tip) | `68aca379bb96d54f` (unchanged since the culprit) |

The rendered change is a colour-only change: the same frame text, different ink.
`a848219f` moved the Junie foreground ladder to the frozen values —
`fg[2]` muted `#808080` → `#8a8a8a`, `fg[3]` faint `#4d4d4d` → `#737373` — and
changed the scrollbar to the frozen track glyph `┆` with the frozen track ink
`#474747` and thumb ink `#808080`.

## Classification of the drift

**Intended.** The pinned reference source `4169783ae` defines `MUTED` as
`rgb(0x8a8a8a)` (`src/theme.rs:528`) and the faint ink as `rgb(0x737373)`
(`src/theme.rs:506`), and declares the scroll track glyph `┆`
(`src/theme.rs:2429`, per `docs/audits/2026-10-20-library-reference-token-oracles.md`).
The baseline file was last written at `032e9faa3`; every later theme change moved
the candidate toward the frozen oracle and no later commit re-blessed the
baseline.

The drift is not, however, a single intended change. The set of commits between
`032e9faa3` and the tip contains both oracle-aligning work and oracle-contradicting
work, and the baseline was never re-blessed for either. Two oracle-contradicting
changes are corrected in this branch (below). The remainder of the moved set is
still unclassified key by key, so **no baseline was re-blessed here**.

### The moved set, read from a discarded scratch bless

```text
cp crates/termrock-conformance/tests/baselines/components.txt /tmp/components.before.txt
BLESS=1 cargo nextest run -p termrock-conformance --all-targets --no-fail-fast
diff /tmp/components.before.txt crates/termrock-conformance/tests/baselines/components.txt
git checkout -- crates/termrock-conformance/tests/baselines/components.txt
```

157 keys moved (157 insertions, 157 deletions): 112 at truecolor, 45 at mono;
127 in the junie theme, 30 in paper; across 30 component families, the largest
being `context_menu` (14), `menu_bar` (13), `help_overlay` (11), `picker` (10) and
`brand` (10). With the scratch bless in place, 76 conformance tests still fail,
so the digest drift is not the whole failure set.

The scratch bless was discarded and the baseline file is byte-identical to
`032e9faa3` again. The moved set is the input a §20.10 ledger entry needs; it
still has to be classified per key against the oracle before any bless.

## Corrections made in this branch

### 1. SQL syntax ink contradicted the frozen corpus

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
failed. The truecolor tokens are restored to the corpus values in
`crates/termrock-theme/src/builtin/junie.rs`.

### 2. The menu bar title lost its public variant identity

`84d6477d7` ("fix(overlays): resolve menu bar title style through
MENU_BAR_TITLE_VARIANT") resolved the title under the private
`termrock.menu-bar-title` variant instead of the public `MENU/DEFAULT/TITLE`
identity. `docs/audits/2026-10-20-menubar-hover.md` states the contract: "Title
defaults come from a private recipe variant, then resolve under the public
`MENU/DEFAULT/TITLE` identity with `Ui::style_defaults`, preserving family
default variants, recipe overrides, overlays, and local part patches." The
private identity drops the family default variant and the overlay background, and
`menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence`
failed on the overlay background. The resolution identity is restored in
`crates/termrock-overlays/src/menu.rs`.

### 3. A retained edit session asked for a cursor it cannot have

`819ef8ef9` changed the caret rule to `st.editing || (read_only && focused)`,
dropping the focus requirement. An editor that keeps its edit session after a
blur is still `editing` but is not focused, so it requested a cursor the cursor
resolver rejects, and both editor tests failed with
`CursorRejected { owner: completion.composite.editor, .. }`. The caret is
requested only for a focused editor again, in
`crates/termrock-editors/src/code.rs`.

### 4. Two documentation examples were not registered

`docs/architecture/termrock-coordinator-plan.md` held an unlabelled shell block
and `docs/audits/2026-10-21-session-typed-input.md` held a `rust` block that
quotes a removed test. The documentation inventory treats both as unregistered
Rust examples. The shell block is labelled `sh`; the retained evidence is quoted
as `text`, because the helper it calls no longer exists and it can never be a
compiled example.

## The other named failures

| failure | root cause | verdict |
| --- | --- | --- |
| `termrock-xtask::ownership ownership_check_passes_on_workspace` | 103 ownership findings, all in `crates/holla-sim`, `crates/holla-ui` and `crates/jackin-preview-app` | not in the crates this branch owns; other agents' in-flight work |
| `termrock-xtask::doc_inventory tracked_workspace_documentation_has_no_unregistered_examples` | two unregistered Rust examples in `docs/` | fixed here |
| `termrock-overlays menu::tests::menu_bar_keeps_public_default_variant_overlay_and_local_patch_precedence` | `84d6477d7` broke the public `MENU/DEFAULT/TITLE` identity | fixed here (regression) |
| `termrock-navigation::item_row_columns constructor_defaults_match_the_pre_policy_full_cell_fixture` | `4d6ad4012` paints a space for an empty item glyph in the shared item row, so the historical full-cell fingerprint no longer matches | open: the change is deliberate for the jackin picker, the fixture is historical; the fixture needs a reviewed re-record, or the space belongs in the picker only |
| `termrock-editors::completing_code_editor focus_loss_dismisses_completion_but_preserves_configured_edit_session` | `819ef8ef9` asked for a caret from an unfocused editor that keeps its session | fixed here (regression) |
| `termrock-editors code::tests::focus_out_commits_by_default_but_can_preserve_the_edit_session` | same cause | fixed here (regression) |
| `termrock-theme::capability_palettes all_73_junie_slots_match_reference_capability_output` | `707b12c76` and `819ef8ef9` moved the syntax tokens off the corpus values while the authored ANSI-256 table kept them | fixed here (regression) |
| `visual-harness::visual_baseline determinism::sources_are_deterministic` | new `Instant::now` uses in `crates/tablepro-domain/src/tabs.rs` and `crates/termrock-core/src/typeahead.rs`, and a new `target_os` use in `crates/termrock-core/src/platform.rs`, are not in the harness allowlist | open: the allowlist update needs a review of crates this branch does not own |
| `visual-harness::visual_baseline overlay_sites` | new overlay sites in `crates/jackin-preview-app` and `crates/tablepro-ui` are not in the harness inventory | open: not in the crates this branch owns |
| `termrock-conformance scroll_region_fixture_exposes_the_complete_bar_at_both_matrix_sizes` | `a848219f` strips bold from the scrollbar track and thumb; the pressed thumb must keep the bold that carries its state in mono | open: the strip must not remove the bold the style itself sets |

## What is still open

- The component baseline is stale against the tip and was not re-blessed. The
  moved set is recorded above; classifying it key by key against the oracle, and
  then writing the §20.10 ledger entry, is the remaining work.
- The per-application baselines (`showcase`, `holla`, `jackin`, `tablepro`) drift
  with the same theme work and live in crates other agents own.
- The conformance failures that are not digest failures (`local_override_does_not_mutate_the_theme`
  in 40 components, `bindings_match_handled_keys`, `mono_states_are_distinguishable`,
  `hover_does_not_steal_focus`, `declared_parts_are_the_parts_actually_styled`)
  are a separate class and were not diagnosed here.
