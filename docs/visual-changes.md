# Visual changes ledger

**What this is.** The ledger `COMPONENT_ARCHITECTURE.md` §20.10 requires and `xtask bless-guard` (§16.3) reads. No baseline file (`crates/tui/tests/baselines/components.txt`, `apps/*/tests/baselines/*.txt`, `perf_baseline.txt` hit counts) may be regenerated without an entry here that names a numbered §20.10 item, **accounts for every baseline key the diff moved or added**, and names the reviewable artefact — a capture path under `shots/` for a baseline produced by a running application, **or** the frame-text dump for a baseline produced headlessly by a `Scene`, named explicitly (§16.3 as amended by §36). Every entry classifies a difference as *intended* (matches the §20.10 item), *fix* (a demonstrated defect in the old output) or *regression* (must be fixed, never blessed).

**Order, fixed (review A14, §21 item 30): change → capture → classify → bless.**

1. **Change** — land the code change on the working tree.
2. **Capture** — for an application baseline, `tools/capture.sh` / `xtask capture-matrix` writes the before/after captures into `shots/`. For a headless `Scene` matrix there is no capture and there can be none (`tools/capture.sh` drives a terminal session and cannot address a `Scene`, §36): the artefact is the frame text the failing run prints. Digest tests go red either way.
3. **Classify** — add or extend the entry under the matching §20.10 item below: the reviewable artefact, the affected tests/baseline lines, the moved and added keys, and the classification with its reason.
4. **Bless** — `BLESS=1 cargo test --workspace --test render --test render_components --test visual` (or `PERF_BLESS=1` for hit counts). `xtask bless-guard` is specified in §16.3 and **is implemented and binding** (`xtask/src/main.rs`, the `bless-guard` subcommand over the `baseline_moves_are_classified` check). It fails closed: with no base revision — neither `BLESS_GUARD_BASE` nor `GITHUB_BASE_REF` — it refuses rather than comparing against `HEAD`, because comparing against `HEAD` passes vacuously (commit `f28a81e`). This ledger is therefore machine-enforced, not convention enforced by review. <!-- corrected 2026-09-05: the previous text claimed the guard was unimplemented. It has been implemented since §47; that sentence was stale, and it is the only claim replaced here. -->

A capture cannot exist before the change, so `bless-guard` never runs locally against an unchanged tree. No baseline is regenerated because a test failed; the classification comes first.

**Entry format** (one per affected surface). Every field of an entry lives inside **one** fenced block, so a reader that strips fences loses neither the classification, nor the citation, nor the evidence:

```

- surface:   <app>/<page or component>/<state> @ <w>x<h> / <theme> / <color level>
- captures:  shots/<before>/png  →  shots/<after>/png
             — for a headless `Scene` matrix instead: `none under shots/`, the reason,
               and the named frame-text dump that replaces it
- tests:     <baseline file>:<line or name>, <test names>
- moved:     <key> <old hash> → <new hash>, one line per baseline key whose hash
             changed; `none` when nothing moved
- added:     <key>, one line per baseline key recorded for the first time; a key
             *pattern* with its exact count is admissible when one matrix generates
             the whole set; `none` when nothing was added
- class:     intended | fix | regression
- reason:    <one sentence tying it to the numbered §20.10 item>
```

**`- moved:` is read, never predicted.** The moved set cannot be known before it is generated, so this field is produced by `git diff <baseline file>` **after a scratch bless that is then discarded** (§36.3): bless into the working tree, read the diff, `git checkout --` the baseline to throw the scratch bless away, write this field from what the diff said, then bless again and commit. The discarded bless is an instrument for reading the diff; the committed bless follows the classification, so the fixed order is not violated.

**`- moved:` and `- added:` partition the diff.** A key that has a previous hash is *moved*; a key recorded for the first time is *added*. Every key the diff touches appears in exactly one of the two fields — that equality, not co-presence of an entry, is what §36.5 makes the checkable property.

**`- captures:` admits two artefacts, and only two.** A capture path under `shots/` for a baseline produced by a running application; or, for a baseline produced headlessly by a `Scene`, the statement that no capture exists or can exist plus the **name** of the frame-text dump that stands in for it — the `Mismatch` / `Missing` branch output of `Scene::assert_against`, which prints the frame text in both cases. §36 established that `tools/capture.sh` drives a terminal session and cannot address a `Scene` at all, so requiring a `shots/` path universally would make the component matrix unblessable by its own rule.

---

## Item 1 — Mono legibility fallbacks (§11.4, §21 item 25)

### 1a — `Tabs` paints §11.4's mono `PRESSED` bracket

**§20.10 classification line:** at `ColorLevel::Mono` every state gains a symbol or
modifier — for `PRESSED`, explicit reverse + `BOLD` + `PressLeft`/`PressRight`
brackets (§21 item 25).

**What changed.** §11.4's `PRESSED` row already mandated the `[label]` bracket and
never said **who paints it**. `Button::draw` consults the `LABEL` glyph slot and
painted it; the `Tabs` row fn paints its label through `RowUi`, which cannot consult
that slot, so a tab never got the bracket. `Tabs` now paints it into the pad cells
the tab already reserves — geometry is identical. Without it a pressed tab and a
focused tab are the same picture under `Mono`: the mono `PRESSED` and `FOCUSED`
styles differ only in colour, and at `Mono` there is no colour.

```
- surface:   junie-tui/tabs/pressed @ {120x40, 40x10} / {junie, paper} / mono
             (truecolor cells are untouched: the bracket branch is gated on
             `Slot::Set(GlyphRole::PressLeft)`, and that slot is set only by a rule
             appended at `ColorLevel::Mono`)
- captures:  none under `shots/` — this is a headless digest matrix, not a running
             app: `tools/capture.sh` drives a terminal session and cannot address a
             `Scene`. The reviewable artefact is the frame text the failing
             `Scene::assert_against` printed for each moved cell, alongside the
             digest diff of `crates/tui/tests/baselines/components.txt` in the same
             commit; the painted **text** gains the two bracket glyphs in cells the
             tab already reserved as padding, and nothing else moves.
- tests:     crates/tui/tests/baselines/components.txt (mono lines only),
             render::components::tabs::pressed,
             conformance::tabs::mono_states_are_distinguishable
- moved:     2 keys (rewritten to the merged diff):
  render::components::tabs::pressed 120 40 junie mono 8531aef99ed82a7c -> 9752ca681323d49c
  render::components::tabs::pressed 40 10 junie mono a1ca30a076849608 -> 324c23adb0bea02a
- added:     none
- class:     fix
- reason:    §20.10 item 1 (mono legibility fallbacks). §11.4 already prescribed the
             `PRESSED` bracket; `Tabs` did not paint it, so under `Mono` a pressed
             tab was byte-identical to a focused one — a declared rule that produced
             no output, which is a demonstrated defect in the old picture rather than
             a downgrade of it.
```

### 1b — `field::disabled` clears the required marker in mono (§29 `Slot<GlyphRole>` migration)

**What changed.** The `Slot<GlyphRole>` migration makes `Slot::Clear` distinct from
`Slot::Inherit`. The mono `DISABLED` rules for `GUTTER` and `MARKER` use
`Slot::Clear`, so the required field's reserved marker cell is now filled rather
than inheriting the `*`. The no-BLESS frame text showed `Name` with no required
marker, which is the declared mono fallback executing rather than an accidental
restyle. This is a second movement of the same keys recorded in item 18a, so it is
classified independently under item 1.

```
- surface:   junie-tui/field/disabled @ {120x40, 40x10} / {junie, paper} / mono
- captures:  none under `shots/` — this is a headless `Scene` matrix. The reviewable
             artefact is the frame-text dump printed by the no-BLESS
             `render::components::field::disabled` run; its first moved frame shows
             the required marker cell cleared. `tools/capture.sh` cannot address a
             `Scene`, and no separate capture file is in this task's ownership.
- tests:     crates/tui/tests/baselines/components.txt (mono lines only),
             render::components::field::disabled
- historical-moved: 4 keys, every one `mono` (superseded by stabilized reconciliation below):
  render::components::field::disabled 120 40 junie mono 399b0a5bc31c9d66 → a17a3ce53b0c07c0
  render::components::field::disabled 120 40 paper mono e0d02bbbddbfe054 → 3ca520240375131a
  render::components::field::disabled 40 10 junie mono d8fb0563075c66a6 → 4a6989f667440f40
  render::components::field::disabled 40 10 paper mono 55505cb874284414 → 4356d66d70ef949a
- added:     none
- class:     fix
- reason:    §20.10 item 1 (mono legibility fallbacks). The `Slot::Clear` rule now
             executes for the required marker's reserved cell, removing an inert
             fallback and making the disabled field's mono output match its declared
             glyph semantics.
```

### 1c — Select retains the mono pressed field bracket

```

### 1d — Loading has its own mono ICON modifier

```
- surface:   every `Caps::REPORTS_STATUS` component at `ColorLevel::Mono` under `LOADING`
- captures:  none; no retained eight-state digest key isolates `LOADING`
- tests:     theme::mono_parts_exactly_cover_every_reserved_rule_part,
             conformance::<reports-status-component>::mono_states_are_distinguishable
- moved:     none measured or authorized
- added:     none
- class:     fix
- reason:    §20.10 item 1. `BUSY` and `LOADING` paint the same animated ICON sequence. The
             20th generic mono fallback adds `UNDERLINED` to `Part::ICON + LOADING`, giving the
             data-loading state a capability-local signal without borrowing an unrelated fixture
             or runtime bit and without changing geometry. Any future digest movement requires
             fresh scratch capture and exact classification before bless.
```
- surface:   junie-tui/select/pressed @ {120x40, 40x10} / {junie, paper} / mono
- captures:  none under `shots/`; clean Junie 120×40 before/after frame text is
             `/tmp/fable-slice4-final-BPvDuo/repo/artifacts/frames/moved-before/mono/select--pressed.txt`
             and `/tmp/fable-slice4-final-BPvDuo/repo/artifacts/frames/moved-after/mono/select--pressed.txt`.
- tests:     crates/tui/tests/baselines/components.txt, render::components::select::pressed
- historical-moved: 4 keys (superseded by stabilized reconciliation below):
  render::components::select::pressed 120 40 junie mono 17bf131df914c266 → c4e91c58984eb68c
  render::components::select::pressed 120 40 paper mono 33ca9d784756b7a6 → 20cc3c074be6adaa
  render::components::select::pressed 40 10 junie mono eafa6492283387c6 → 75aee1f6e6798d6c
  render::components::select::pressed 40 10 paper mono cec1880d683c3a06 → 5dadb5bc71aedd0a
- added:     none
- class:     fix
- reason:    §20.10 item 1. The corrected closed Select keeps the prescribed mono
             FIELD inverse+BOLD and GUTTER `[` / MARKER `]` bracket anatomy; the
             dedicated disclosure role no longer competes for the pressed marker cell.
```

## Item 2 — Layer compositing order (§5 R7, §3.3 step 12)

captures / classification: `(pending — filled when the change lands)`

## Item 3 — `RadioGroup` separates cursor from value

captures / classification: `(pending — filled when the change lands)`

## Item 4 — `Picker` secondary action gains a mouse equivalent

captures / classification: `(pending — filled when the change lands)`

## Item 5 — `Dialog`'s `y`/`n` quick answers become an opt-in binding set

captures / classification: `(pending — filled when the change lands)`

## Item 6 — F10 / menu-bar drift fixes

captures / classification: `(pending — filled when the change lands)`

## Item 7 — Container / geometry defect fixes (a)–(j)

Items (b), (e) and (f) change pixels in the current baseline and are called out individually.

captures / classification: `(pending — filled when the change lands)`

### 7a — ScrollRegion track height and Grid's transitive movement (2026-09-06)

The current uncommitted baseline diff is classified here in full. No baseline was blessed or
edited by this ledger entry.

```
- surface:   junie-tui/ScrollRegion and junie-tui/Grid @ {120x40, 40x10} /
             {junie, paper} / {truecolor, mono}
- captures:  none under shots/ — these are headless Scene matrix cells; tools/capture.sh
             cannot address a Scene. Named evidence: the no-BLESS Scene::assert_against
             frame-text dump from
             rtk cargo test -p junie-tui --test render_components -- --nocapture.
             Durable evidence paths: docs/visual-changes.md:180 (this entry and its
             embedded frame text), crates/tui/tests/render_components.rs:1459 (the
             headless Scene matrix runner), and
             crates/tui/tests/baselines/components.txt (the exact baseline diff).
             The seven reported ScrollRegion mismatches stop at 40x10 / junie / truecolor:
             default, disabled, editing, focused, hovered, and selected report baseline
             9a8c97d4cf642078 → got f56b60ecfbd2c323; pressed reports baseline
             7eeb9df24da45ab4 → got 2bd782ba4ada20cf. Each dump prints this exact
             headless frame text:
             row 0                                  ┃
             row 1                                  │
             row 2                                  │
             row 3                                  │
             row 4                                  │
             row 5                                  │
             row 6                                  │
             row 7                                  │
             row 8                                  │
             row 9                                  │
             The complete run reported 311 passed and 21 failed:
             panel::{default,disabled,editing,focused,hovered,pressed,selected},
             scroll_region::{default,disabled,editing,focused,hovered,pressed,selected},
             and tree::{default,disabled,editing,focused,hovered,pressed,selected}.
             The complete durable pre-BLESS output, including the Panel and Tree frame-text
             dumps, is parity/component-render-current-full.txt; the clean post-BLESS run is
             parity/component-render-post-bless.txt (332 passed).
             Only the seven ScrollRegion failures overlap this baseline diff; all six Grid
             state tests completed without a reported mismatch. Each failing state stops
             at its first matrix cell, so this evidence does not silently claim later
             cells were rechecked.
- tests:     crates/tui/tests/baselines/components.txt (104 moved lines);
             render::components::scroll_region::{default,disabled,editing,focused,hovered,pressed,selected};
             render::components::grid::{default,disabled,editing,focused,hovered,selected}
- moved:     26 keys (rewritten to the merged diff):
  render::components::grid::default 120 40 junie mono cc1660d468c321f6 -> d0f6ba3fbb7ab79a
  render::components::grid::default 40 10 junie mono 868216137c400e56 -> bb17af21a1c1388a
  render::components::grid::disabled 120 40 junie mono cc1660d468c321f6 -> d0f6ba3fbb7ab79a
  render::components::grid::disabled 40 10 junie mono 868216137c400e56 -> bb17af21a1c1388a
  render::components::grid::editing 120 40 junie mono cc1660d468c321f6 -> d0f6ba3fbb7ab79a
  render::components::grid::editing 40 10 junie mono 868216137c400e56 -> bb17af21a1c1388a
  render::components::grid::focused 120 40 junie mono fe58af36f3b95730 -> dac7679930934574
  render::components::grid::focused 40 10 junie mono 420a6954a9fdbe10 -> f124fb945e76be84
  render::components::grid::hovered 120 40 junie mono cc1660d468c321f6 -> d0f6ba3fbb7ab79a
  render::components::grid::hovered 40 10 junie mono 868216137c400e56 -> bb17af21a1c1388a
  render::components::grid::selected 120 40 junie mono 47905521c7e525fb -> 222796a2adc2aaa7
  render::components::grid::selected 40 10 junie mono 95f1e6957b28b4db -> 07dbc80fd67c7857
  render::components::scroll_region::default 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::default 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::disabled 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::disabled 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::editing 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::editing 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::focused 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::focused 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::hovered 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::hovered 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::pressed 120 40 junie mono 55fca6c68455d891 -> fcd176f0c7336e03
  render::components::scroll_region::pressed 40 10 junie mono 869ee257115dc6f2 -> dce5bb00e7e45cae
  render::components::scroll_region::selected 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::selected 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
- added:     none
- class:     fix
- reason:    §20.10 item 7 (container/geometry defect fixes). ScrollRegion's shared
             scrollbar now paints the complete bounded track; Grid inherits that correction
             transitively, so its 48 movements are not a separate Grid visual change.
             The seven reported no-BLESS mismatches keep this entry as audit evidence only;
             it does not authorize a baseline bless.
```

### 7b — Tree leaf disclosure placeholder inherits the row style (2026-09-08)

A leaf row has no disclosure glyph, but the reserved fold cell was painted with the ICON
recipe instead of the row recipe, fabricating a colour affordance on blank cells. The
placeholder now inherits the row style; a real disclosure glyph and an explicit ICON patch
keep the ICON recipe. The 44 disclosure-driven baseline movements are classified here in
full; the 6 `tree::empty` cells the same BLESS run captured are not caused by this fix and
are accounted for by the control note at 7c.

```
- surface:   junie-tui/Tree @ {120x40, 40x10} / {junie, paper} / {truecolor, mono}
- captures:  none under shots/ — these are headless Scene matrix cells; tools/capture.sh
             cannot address a Scene. Named evidence: the no-BLESS Scene::assert_against
             failure output from
             cargo test -p junie-tui --test render_components -- tree --test-threads=1,
             plus the unit test
             components::tree::tests::leaf_disclosure_placeholder_inherits_row_style_but_branch_keeps_icon_style
             and the retained tree_presentation test
             default_prefix_and_opt_in_gap_move_paint_and_hits_together asserting the branch
             disclosure glyph stays and the leaf fold cell keeps the row foreground. The
             exact regenerated baseline diff is crates/tui/tests/baselines/components.txt.
- moved:     44 keys (rewritten to the merged diff):
  render::components::tree::default 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::default 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 94e7ce70bb65b804
  render::components::tree::default 120 40 paper truecolor 81f56ee97b029198 -> b1092f71d75fb128
  render::components::tree::default 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::default 40 10 junie truecolor 99095ab47ad9a104 -> c5df854de7f82804
  render::components::tree::default 40 10 paper truecolor 04a805d95fdc0948 -> 2c1861b28dfe1218
  render::components::tree::disabled 120 40 junie mono 9ad9f5ae65342def -> b47c8c607dc66da5
  render::components::tree::disabled 120 40 junie truecolor 6c1e1c6a55a57308 -> 66f0f5e22836fe6c
  render::components::tree::disabled 120 40 paper truecolor ad8787fe91ced26d -> bd7bac2b04bf1601
  render::components::tree::disabled 40 10 junie mono 322991e541e6c6ef -> 24215f18b6f71925
  render::components::tree::disabled 40 10 junie truecolor b05e8a7a5e83cf48 -> c001164f315332ac
  render::components::tree::disabled 40 10 paper truecolor dba0ce46f92191ad -> 28bbd39eca633101
  render::components::tree::editing 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::editing 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 94e7ce70bb65b804
  render::components::tree::editing 120 40 paper truecolor 81f56ee97b029198 -> b1092f71d75fb128
  render::components::tree::editing 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::editing 40 10 junie truecolor 99095ab47ad9a104 -> c5df854de7f82804
  render::components::tree::editing 40 10 paper truecolor 04a805d95fdc0948 -> 2c1861b28dfe1218
  render::components::tree::focused 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::focused 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 94e7ce70bb65b804
  render::components::tree::focused 120 40 paper truecolor 81f56ee97b029198 -> b1092f71d75fb128
  render::components::tree::focused 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::focused 40 10 junie truecolor 99095ab47ad9a104 -> c5df854de7f82804
  render::components::tree::focused 40 10 paper truecolor 04a805d95fdc0948 -> 2c1861b28dfe1218
  render::components::tree::hovered 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::hovered 120 40 junie truecolor 1a15529c5d5bc544 -> ae587d7429f2d2d4
  render::components::tree::hovered 120 40 paper truecolor c5eb5fd78905ad58 -> 6366223c93c0ddac
  render::components::tree::hovered 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::hovered 40 10 junie truecolor 7a7a93564af16404 -> a381318908fe5c54
  render::components::tree::hovered 40 10 paper truecolor c733b9963459a748 -> 376e0191ded54e9c
  render::components::tree::pressed 120 40 junie mono d450dbacbe5781fd -> 2a4e808ba871c1b9
  render::components::tree::pressed 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 069f49f3f3febc88
  render::components::tree::pressed 120 40 paper mono e7688a03d244eb33 -> ebc06c233268e8e1
  render::components::tree::pressed 120 40 paper truecolor 81f56ee97b029198 -> c94f18e732664b43
  render::components::tree::pressed 40 10 junie mono 77c59187cd64f51d -> 48f68635c4f8d9d9
  render::components::tree::pressed 40 10 junie truecolor 99095ab47ad9a104 -> 3bb041ba8aaadd08
  render::components::tree::pressed 40 10 paper mono 8cef2684bcf13753 -> 35000ca780235701
  render::components::tree::pressed 40 10 paper truecolor 04a805d95fdc0948 -> 62fb872559518823
  render::components::tree::selected 120 40 junie mono 96f1f5dbc64034bd -> 9e1c40991a3ede69
  render::components::tree::selected 120 40 junie truecolor 25d188f3267654af -> 1e04d0ca77ba5c6f
  render::components::tree::selected 120 40 paper truecolor d46a76d2cb4df19a -> fe2391d8faa6c80a
  render::components::tree::selected 40 10 junie mono fa7ef4626541ec5d -> 05ac5faae8a77a49
  render::components::tree::selected 40 10 junie truecolor 9ea34149e59474af -> cb796be352b2fbaf
  render::components::tree::selected 40 10 paper truecolor 825d1d60faeb52ca -> 5e325a89c362e5fa
- added:     none
- class:     fix
- reason:    §20.10 item 7 (container/geometry defect fixes). Tree leaves inherit the row
             style without a fabricated disclosure marker; a real disclosure glyph or an
             explicit ICON patch keeps the ICON recipe. An empty tree paints no rows, so no
             `tree::empty` cell can move from this fix; the 6 such cells the same BLESS run
             captured were already stale at the base revision and are recorded at 7c.
```

### 7c — control note: the 6 `tree::empty` cells the same BLESS run captured were already stale at the base revision (2026-09-08)

The 7b fix cannot move an empty-state cell: an empty tree paints no rows, so no leaf fold
cell exists to restyle. The `BLESS=1` run that regenerated 7b's 44 cells also captured the
6 `tree::empty` cells below, and §36.5 makes the moved/added partition checkable, so those
6 are classified here rather than folded into 7b's list. They are the same pre-existing
staleness class as this branch's other render baselines: their digests were already stale
at the base revision, and this BLESS run synced them. The control that proves the movement
pre-dates the 7b fix: with the worktree's tui sources reverted to the base revision,
`cargo test -p junie-tui --test render_components -- tree --test-threads=1` still fails
`render::components::tree::empty` against the NEW baseline values — the base sources
already disagreed with the base baselines, so neither the 7b change nor its revert
reproduces the old digests.

```
- surface:   junie-tui/Tree empty state @ {120x40, 40x10} / {junie, paper} / {truecolor,
             mono} — control note for 7b, not a separate visual change
- captures:  none under shots/ — headless Scene matrix cells; the named evidence is the
             HEAD-source control above: at the base revision's sources the
             render::components::tree::empty test already failed against the base
             baselines, so the staleness is a pre-existing defect in the recorded
             baseline, not in this tree's output
- moved:     6 keys (rewritten to the merged diff):
  render::components::tree::empty 120 40 junie mono d43c8daeaf982a7a -> 449dd86eee94479e
  render::components::tree::empty 120 40 junie truecolor 121b6cb8fded49a2 -> 031f18534b558a72
  render::components::tree::empty 120 40 paper truecolor ddbd052dff13465a -> 6aa320d4b4a79de2
  render::components::tree::empty 40 10 junie mono aeb7ad6bd94d0a1a -> de62a211f1cd715e
  render::components::tree::empty 40 10 junie truecolor ffa7ea144c4cbd62 -> 4bc1ebbfb6005ef2
  render::components::tree::empty 40 10 paper truecolor 065d1b5055061c8a -> be7ba0c4623a2cca
- added:     none
- class:     fix
- reason:    §20.10 item 7 (container/geometry defect fixes). No output defect is claimed
             for these 6 cells and the 7b placeholder restyle does not touch them: the
             HEAD-source control shows the digests were already stale at the base
             revision, and the BLESS run synced the recorded baseline to the real output
             (the same pre-existing staleness class as this branch's other render
             baselines).
```

### 7d — Panel metadata padding and Tree leaf disclosure style (2026-09-06, main line)

The same current component source correction changes fourteen retained Junie truecolor cells.
They are classified separately because they are not transitive ScrollRegion/Grid movement.


Absorbed by the merge of the main line that produced it; retained so the
transition it classifies stays readable in merged history. Post-merge movement
of these key names, if any, is classified by the integration-era entries.
```
- surface:   junie-tui/{Panel,Tree} @ 120x40 / junie / truecolor
- captures:  parity/component-render-current-full.txt (pre-BLESS frame-text output) and
             parity/component-render-post-bless.txt (clean 332-test run)
- tests:     render::components::{panel,tree}::{default,disabled,editing,focused,hovered,pressed,selected}
- moved:     70 keys (rewritten to the merged diff):
  render::components::panel::default 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::default 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::disabled 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::disabled 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::editing 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::editing 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::focused 120 40 junie mono f3cb0357a45a63b8 -> b2f09676a791a4dc
  render::components::panel::focused 40 10 junie mono 99fe87c2dd6f4010 -> f03ce99ba1f7a1fc
  render::components::panel::hovered 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::hovered 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::pressed 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::pressed 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::selected 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::selected 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::tree::default 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::default 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 94e7ce70bb65b804
  render::components::tree::default 120 40 paper mono e7688a03d244eb33 -> b309aa954616d2e7
  render::components::tree::default 120 40 paper truecolor 81f56ee97b029198 -> b1092f71d75fb128
  render::components::tree::default 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::default 40 10 junie truecolor 99095ab47ad9a104 -> c5df854de7f82804
  render::components::tree::default 40 10 paper mono 8cef2684bcf13753 -> 10dbf2bf094ddac7
  render::components::tree::default 40 10 paper truecolor 04a805d95fdc0948 -> 2c1861b28dfe1218
  render::components::tree::disabled 120 40 junie mono 9ad9f5ae65342def -> b47c8c607dc66da5
  render::components::tree::disabled 120 40 junie truecolor 6c1e1c6a55a57308 -> 66f0f5e22836fe6c
  render::components::tree::disabled 120 40 paper mono 3078e8871da44d93 -> c42b1d1f816bccbf
  render::components::tree::disabled 120 40 paper truecolor ad8787fe91ced26d -> bd7bac2b04bf1601
  render::components::tree::disabled 40 10 junie mono 322991e541e6c6ef -> 24215f18b6f71925
  render::components::tree::disabled 40 10 junie truecolor b05e8a7a5e83cf48 -> c001164f315332ac
  render::components::tree::disabled 40 10 paper mono c793162e3ac72233 -> 937742fdc48c899f
  render::components::tree::disabled 40 10 paper truecolor dba0ce46f92191ad -> 28bbd39eca633101
  render::components::tree::editing 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::editing 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 94e7ce70bb65b804
  render::components::tree::editing 120 40 paper mono e7688a03d244eb33 -> b309aa954616d2e7
  render::components::tree::editing 120 40 paper truecolor 81f56ee97b029198 -> b1092f71d75fb128
  render::components::tree::editing 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::editing 40 10 junie truecolor 99095ab47ad9a104 -> c5df854de7f82804
  render::components::tree::editing 40 10 paper mono 8cef2684bcf13753 -> 10dbf2bf094ddac7
  render::components::tree::editing 40 10 paper truecolor 04a805d95fdc0948 -> 2c1861b28dfe1218
  render::components::tree::focused 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::focused 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 94e7ce70bb65b804
  render::components::tree::focused 120 40 paper mono e7688a03d244eb33 -> b309aa954616d2e7
  render::components::tree::focused 120 40 paper truecolor 81f56ee97b029198 -> b1092f71d75fb128
  render::components::tree::focused 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::focused 40 10 junie truecolor 99095ab47ad9a104 -> c5df854de7f82804
  render::components::tree::focused 40 10 paper mono 8cef2684bcf13753 -> 10dbf2bf094ddac7
  render::components::tree::focused 40 10 paper truecolor 04a805d95fdc0948 -> 2c1861b28dfe1218
  render::components::tree::hovered 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::hovered 120 40 junie truecolor 1a15529c5d5bc544 -> ae587d7429f2d2d4
  render::components::tree::hovered 120 40 paper mono e7688a03d244eb33 -> b309aa954616d2e7
  render::components::tree::hovered 120 40 paper truecolor c5eb5fd78905ad58 -> 6366223c93c0ddac
  render::components::tree::hovered 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::hovered 40 10 junie truecolor 7a7a93564af16404 -> a381318908fe5c54
  render::components::tree::hovered 40 10 paper mono 8cef2684bcf13753 -> 10dbf2bf094ddac7
  render::components::tree::hovered 40 10 paper truecolor c733b9963459a748 -> 376e0191ded54e9c
  render::components::tree::pressed 120 40 junie mono d450dbacbe5781fd -> 2a4e808ba871c1b9
  render::components::tree::pressed 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 069f49f3f3febc88
  render::components::tree::pressed 120 40 paper mono e7688a03d244eb33 -> ebc06c233268e8e1
  render::components::tree::pressed 120 40 paper truecolor 81f56ee97b029198 -> c94f18e732664b43
  render::components::tree::pressed 40 10 junie mono 77c59187cd64f51d -> 48f68635c4f8d9d9
  render::components::tree::pressed 40 10 junie truecolor 99095ab47ad9a104 -> 3bb041ba8aaadd08
  render::components::tree::pressed 40 10 paper mono 8cef2684bcf13753 -> 35000ca780235701
  render::components::tree::pressed 40 10 paper truecolor 04a805d95fdc0948 -> 62fb872559518823
  render::components::tree::selected 120 40 junie mono 96f1f5dbc64034bd -> 9e1c40991a3ede69
  render::components::tree::selected 120 40 junie truecolor 25d188f3267654af -> 1e04d0ca77ba5c6f
  render::components::tree::selected 120 40 paper mono f23368fb682439b3 -> 733c344364f50fc7
  render::components::tree::selected 120 40 paper truecolor d46a76d2cb4df19a -> fe2391d8faa6c80a
  render::components::tree::selected 40 10 junie mono fa7ef4626541ec5d -> 05ac5faae8a77a49
  render::components::tree::selected 40 10 junie truecolor 9ea34149e59474af -> cb796be352b2fbaf
  render::components::tree::selected 40 10 paper mono c8f712571ba5be53 -> 6c0add5e74824e27
  render::components::tree::selected 40 10 paper truecolor 825d1d60faeb52ca -> 5e325a89c362e5fa
- added:     none
- class:     fix
- reason:    §20.10 item 7 (container/geometry defect fixes). Panel restores the historical
             metadata padding contract; Tree leaves inherit the row style without a fabricated
             disclosure marker. These exact retained moves are independently reviewed in the
             durable no-BLESS output before baseline blessing.
```


## Item 8 — The backdrop excludes the footer row uniformly

captures / classification: `(pending — filled when the change lands)`

## Item 9 — `StatusBar` and `segments` merge

captures / classification: `(pending — filled when the change lands)`

## Item 10 — Hints are derived from component bindings

The diff of the old hand-written hint tables against the derived output is attached here, each drifted entry classified as fix or regression.

captures / classification: `(pending — filled when the change lands)`

## Item 11 — Surface inheritance replaces colour-equality `lift`

Any cell that changes under `junie` is enumerated with the token pair that collided.

captures / classification: `(pending — filled when the change lands)`

## Item 12 — The showcase visual baseline covers the sidebar and gains three axes

captures / classification: `(pending — filled when the change lands)`

## Item 13 — `Tabs`' strip window follows the logical first tab

captures / classification: `(pending — filled when the change lands)`

## Item 14 — New cell-exact baselines for TablePro and jackin

The first generation is produced on the pre-refactor tree (Appendix A, WP‑0 — still owed, not in `07cb2c9`), then regenerated once at the end of Slice 8.

captures / classification: `(pending — filled when the change lands)`

## Item 15 — Focus-ring composition changes in migrated screens (§21 item 33)

Per affected test: old reachable count, new reachable count, the `Harness::ring().reachable()` listing, and the reason — written **before** the expected value in the test is edited. Also the home for `frame_showcase_lists_120x40`'s classified hit-count growth (§16.6, P8).

captures / classification: `(pending — filled when the change lands)`

## Item 16 — Display width follows `CellWidth::cell_width`, not raw `unicode-width` (§22)

Any string containing U+FF9E/U+FF9F measures one column wider per mark, so every cell whose line moves is enumerated with the string that re-measured.

captures / classification: `(pending — filled when the change lands)`

## Item 17 — `Anchor::Point` flips instead of covering the pointer (§26, Adjudication N1)

**Already recorded as owed.** §26 risk 6 states that the flip is a visual change for
any existing tooltip or context menu near a screen edge, and that an entry under this
item is required **before any baseline that moves is blessed**. No tooltip or
context-menu baseline may be blessed until this section carries a real entry.

captures / classification: `(pending — filled when the change lands)`

## Item 18 — Mono `DISABLED` gains `DIM` on `FIELD`/`TEXT` and stops tinting the foreground into the background (§28, Adjudication P6)

### 18a — the mono `DISABLED` rules

**§20.10 classification line:** mono `DISABLED` gains `DIM` on `FIELD`/`TEXT` and
stops tinting the foreground into the background.

**What changed.** §11.4's `DISABLED` row prescribed `fg = Role::Fg(Faint)`. At
`ColorLevel::Mono` that is a defect, not a downgrade: `mono()` maps every step
below `Y = 0.35` to `Black`, and `junie`'s `disabled_fg` (`#4d4d4d`), `Fg(Faint)`
(`#262626`) and `surfaces[0]` (`#000000`) are all below it — a disabled control
was painted **black on black**, unreadable rather than merely colourless
(goal §29 asks for readable). The mono table also reached no part a *text*
control paints for its own content, so a disabled `TextInput` was
indistinguishable from an enabled one under `Mono` at all.

Three rule changes in `crates/tui/src/theme/downgrade.rs::mono_rules()`
(`MONO_RULES_PER_FAMILY` 16 → 18):

- new `(Part::FIELD, DISABLED)` and `(Part::TEXT, DISABLED)`:
  `set_fg(Fg(Primary)).remove(Modifier::all()).add(Modifier::DIM)`;
- amended `(Part::LABEL, DISABLED)` and `(Part::MARKER, DISABLED)`:
  `Fg(Faint)` → `Fg(Primary)`, same reason.

`Part::PLACEHOLDER` needs no rule (it inherits the `FIELD` fill's modifiers per cell)
and `Part::CONTAINER` needs none (a text control fills `FIELD`). The new rules are
declared **before** the `ERROR` rules, so `ERROR`'s `UNDERLINED` is not erased
by `remove(Modifier::all())`.

The four historical `field::disabled` keys are not repeated in this entry's
machine-checked movement field: item 1b owns their current second movement, and
the previous transition remains historical git evidence.

```
- surface:   junie-tui/{text_input,field,list,button,tabs,dialog}/disabled
             @ {120x40, 40x10} / {junie, paper} / mono   (truecolor cells are untouched:
             mono rules are appended only at `ColorLevel::Mono`)
- captures:  none under `shots/` — this matrix is a headless digest matrix, not a
             running app: `tools/capture.sh` drives a terminal session and cannot
             address a `Scene`. The reviewable artefact is the frame text
             `Scene::assert_against` printed for each moved cell, alongside the digest
             diff of `crates/tui/tests/baselines/components.txt` in the same commit;
             the painted **text** is byte-identical in every moved cell (the panic
             output shows it), so the whole difference is style.
- tests:     crates/tui/tests/baselines/components.txt (mono lines only),
             render::components::{text_input,field,list,button,tabs,dialog}::disabled
- historical-moved: 12 lines, every one `mono` (superseded by stabilized reconciliation below):
  render::components::button::disabled 120 40 junie mono 023bd60f5b1ae845 → d20bb906fcfe3dd1
  render::components::button::disabled 40 10 junie mono 15af984dfc54c7c5 → bfe7ea91b76bd751
  render::components::dialog::disabled 120 40 junie mono 3162b7d5bbf2a5f5 → 2d6a3cd4c020d7e5
  render::components::dialog::disabled 40 10 junie mono 03fb01cee70da7f5 → 3ef081624f0f1fa5
  render::components::list::disabled 120 40 junie mono 8ef3444eee52116d → 5c5f27303fa8adf5
  render::components::list::disabled 40 10 junie mono 6dc1b708da3a1a6d → 86dd18d3924968f5
  render::components::tabs::disabled 120 40 junie mono 35a3a27d0daf3a0c → 383875a51445a582
  render::components::tabs::disabled 40 10 junie mono f3715f8ca6758086 → d48c88f61b9cf638
  render::components::text_input::disabled 120 40 junie mono f32a4730f22cd73a → 1db1055714d0b91e
  render::components::text_input::disabled 120 40 paper mono 44f4c6f88a4aec46 → d36c9e02c6aca8be
  render::components::text_input::disabled 40 10 junie mono 8725f1e9f6355d3a → 1e91d7e4d1cdcfde
  render::components::text_input::disabled 40 10 paper mono d63984c2d12b1c26 → 5257f3bad4cc42be
- added:     none
- class:     fix
- reason:    §20.10 item 18 (mono `DISABLED` gains `DIM` and stops tinting the
             foreground into the background). The old output was unreadable at
             `Mono` (black on black) and gave a text control's disabled state no
             signal at all; the new output is `DIM` over the primary foreground,
             which §16.2 case 9 can see and a reader can read.
             `conformance::text_input::mono_states_are_distinguishable` now keeps
             `DISABLED` instead of narrowing it away (MA-8).
```

### 18b — `Button`'s mono `PRESSED` bracket moves out of the text run (§29 Q1), and `Dialog`'s action row moves with it

**What changes.** Under §29 Q1 `Button` stops painting the `PressLeft`/`PressRight`
bracket **inside** its text run and paints it into the cells the button reserves
around the label. The old in-run bracket consumed two columns of the label's own
run, so a label that exactly filled the button was **truncated** by the very glyphs
that were supposed to make it legible under `Mono` — a demonstrated defect in the
old output, not a restyle, which is why the class below is `fix` and not `intended`.

`Dialog` moves in the same pass and for the same reason: it does not draw its own
action buttons, it draws `Button` through `inherit_forced`, so every `Dialog` state
that paints an action row inherits the new bracket placement. §20.10 item 18's tail
clause, as extended by §36, names exactly this pair — `render::components::button::pressed`'s
mono line **together with the same state of every component that draws a `Button`
through `inherit_forced`, `Dialog`'s action row in the current matrix**.

```
- surface:   junie-tui/{button,dialog}/pressed @ {120x40, 40x10} / {junie, paper} / mono
- captures:  none under `shots/` — headless `Scene` matrix (see 18a). The reviewable
             artefact is the frame-text dump printed by the no-BLESS
             `render::components::{button,dialog}::pressed` runs; it shows the
             bracket in the reserved padding rather than inside the label run.
- tests:     crates/tui/tests/baselines/components.txt (mono lines only),
             render::components::button::pressed, render::components::dialog::pressed,
             conformance::button::mono_states_are_distinguishable
- historical-moved: 8 keys, every one `mono` (superseded by stabilized reconciliation below):
  render::components::button::pressed 120 40 junie mono 3b2454b37bd7d149 → 71350c86866c7f71
  render::components::button::pressed 120 40 paper mono b37e9d409e89f5f5 → 87762b1d30c7aea5
  render::components::button::pressed 40 10 junie mono c29079987470c6c9 → 302ae7a815fa48f1
  render::components::button::pressed 40 10 paper mono b08b15823138ad75 → 5c08bee9295f9e25
  render::components::dialog::pressed 120 40 junie mono c846229d88d9cd3e → 5ccef1c43fe1ab59
  render::components::dialog::pressed 120 40 paper mono 61ac4de605eded4c → 1e15fad3fcd96a23
  render::components::dialog::pressed 40 10 junie mono 2cdd3894646d19da → 96c7c7c04a22573d
  render::components::dialog::pressed 40 10 paper mono 6c4029eb5bcbc25c → d48511478f6f91eb
- added:     none
- class:     fix
- reason:    §20.10 item 18, tail clause as extended by §36. The old in-run bracket
             truncated a label that filled the button, so the old output is
             demonstrably wrong rather than merely different; `Dialog` moves because
             it draws its buttons through `inherit_forced` and has no bracket of its
             own.
```

## Item 19 — First-generation `render::components::*` digests for the Slice-4 component matrix (§36)

**First generation only.** Nothing moves: the component did not exist in the reviewed
tree, so no cell has a before-image and no cell is a difference from anything. The
second time one of these keys changes it classifies under items 1–18 or it is a
regression, and **this item may not be cited again for the same key**.

### 19a — the fourteen Slice-4 components record their first digest lines

**What is recorded.** Fourteen components — `text_area`, `select`, `radio_group`,
`checkbox`, `toggle`, `chip_bar`, `status_bar`, `hint_bar`, `key_hint`,
`progress_bar`, `spinner`, `meter`, `empty`, `brand` — each generate
8 states (`default`, `focused`, `hovered`, `pressed`, `disabled`, `selected`,
`editing`, `empty`) × {junie, paper} × {truecolor, mono} × {120×40, 40×10} = **64**
lines, **896** in total. One matrix (`crates/tui/tests/render_components.rs::run`)
generates the whole set from one loop nest, which is why the `- added:` field below
is a key pattern with an exact count rather than 896 transcribed keys.

**What this entry is not.** It is not an approval of how these components look. A
first-generation digest cannot be reviewed as a digest: the hash is not inspectable
and there is no before-image to diff. What is reviewed is the **frame**, and only its
glyph half; the style half — `fg`, `bg`, `modifier` — is reviewed by nobody here and
is asserted instead by the 20-case conformance matrix and the `theme::*` contrast and
mono-legibility tests. A first-generation line is a **pin against future drift, not
an approval of present appearance** (§20.10 item 19, §36.4); the first review of
these components *as pictures* is the Slice-5 capture matrix.

```
- surface:   junie-tui/{text_area,select,radio_group,checkbox,toggle,chip_bar,
             status_bar,hint_bar,key_hint,progress_bar,spinner,meter,empty,brand}
             / {default,focused,hovered,pressed,disabled,selected,editing,empty}
             @ {120x40, 40x10} / {junie, paper} / {truecolor, mono}
- captures:  none under `shots/`, and none can exist — the matrix is headless:
             `Scene` draws into a `TestBackend` buffer and `tools/capture.sh` drives a
             terminal session, so it cannot address a `Scene` at all (§36.4).
             The artefact that replaces it is the **frame-text dump** printed by the
             `Missing` branch of `Scene::assert_against` during the no-BLESS
             `render_components` run. It is reviewable test output; no separate
             frame file is added because this task owns the ledger and baseline,
             not `docs/frames/`. §20.10 item 19 still treats these lines as pins
             against future drift, not approval of present appearance.
- tests:     crates/tui/tests/baselines/components.txt,
             render::components::{text_area,select,radio_group,checkbox,toggle,
             chip_bar,status_bar,hint_bar,key_hint,progress_bar,spinner,meter,empty,
             brand}::{default,focused,hovered,pressed,disabled,selected,editing,empty}
- moved:     none. No key in this entry has a previous hash; the components did not
             exist in the reviewed tree, so no cell has a before-image.
- added:     896 keys, the complete cross product of the machine-expandable pattern
             `render::components::{text_area,select,radio_group,checkbox,toggle,chip_bar,status_bar,hint_bar,key_hint,progress_bar,spinner,meter,empty,brand}::{default,focused,hovered,pressed,disabled,selected,editing,empty} {120 40,40 10} {junie,paper} {truecolor,mono}`.
             It expands to fourteen components × eight states × two sizes × two
             themes × two colour levels = 896 keys, 64 per component.
- class:     intended
- reason:    §20.10 item 19 (first-generation `render::components::*` digests for the
             Slice-4 component matrix). §16.3 requires one digest line per component
             × state × theme × colour × size and these fourteen had none; the lines
             are a pin against future drift, **not** an approval of present
             appearance, and the item may never be cited again for the same key.
```

### 19b — Existing component and overlay keys entered the archived baseline

```
- surface:   the six retained component matrices and the two overlay probes in `crates/tui/tests/baselines/components.txt`
- captures:  none under `shots/` — these are headless Scene digests; the frame text is emitted by `render_components` on a no-BLESS comparison
- tests:     crates/tui/tests/baselines/components.txt, render::components::* and overlay matrix tests
- moved:     none
- added:     386 keys: `overlay::modal_over_page 40 12 junie truecolor`, `overlay::nested_picker_over_dialog 40 12 junie truecolor`, and `render::components::{button,dialog,field,list,tabs,text_input}::{default,focused,hovered,pressed,disabled,selected,editing,empty} {120 40,40 10} {junie,paper} {truecolor,mono}`
- class:     intended
- reason:    §20.10 item 19 (first-generation digest recording). The pre-refactor revision used for the bless guard predates the component baseline file, so these retained component and overlay keys are first recordings in that file, not movements of frozen evidence.
```

## Item 20 — Forced state preserves the props-derived readiness state (§49)

### 20a — disabled readiness reporters paint their error affordance

**§20.10 classification line:** forcing `DISABLED` substitutes for the runtime state only;
the props-derived `ERROR` state remains present, so readiness-reporting components paint their
declared error affordance in truecolor and mono.

**What changed.** `render::components::{hint_bar,meter,progress_bar}::disabled` supplies
`Status::Error` while forcing `DISABLED`. Before §39's two-half state operator, the forced state
erased the props-derived `ERROR` bit, leaving each component's declared error recipe inert. The
corrected operator combines forced runtime state with props-derived state. A discarded scratch
bless measured exactly 22 moved keys: 8 HintBar, 6 Meter and 8 ProgressBar; 12 truecolor and 10
mono. No key was added.

**Independent visual review: PASS.** A fresh read-only visual analyst who did not generate the
lines reviewed the six corrected `junie` 120×40 frames (three components × truecolor and mono).
Each frame contains the declared `GlyphRole::Error` (`!`); the supplied labels, tracks and `65%`
remain intact; no error affordance appears in a ready-state cell; movement is confined to the
disabled baseline keys. Review criteria were §20.10 item 20's five rejection conditions: reject
if the error affordance is absent, appears for `Status::Ready`, changes a label, changes track
arithmetic or the percentage column beyond the affordance and its reserved columns, or occupies
no `GlyphSet` slot. **Review limit:** the evidence has no textual before-frame dumps, so exclusion
of unrelated changes relies on the exact 22-key digest scope plus the six reviewed after-frames;
the style half remains machine-asserted rather than visually recoverable from frame text.

```
- surface:   junie-tui/{hint_bar,meter,progress_bar}/disabled
             @ {120x40, 40x10} / {junie, paper} / {truecolor, mono}
- captures:  none under `shots/` — this is a headless `Scene` matrix and
             `tools/capture.sh` cannot address it. Reviewable after-frame artefacts:
             `/tmp/fable49-evidence-lStqI4/artifacts/hint-bar-disabled-junie-truecolor-frame.log`,
             `/tmp/fable49-evidence-lStqI4/artifacts/hint-bar-disabled-junie-mono-frame.log`,
             `/tmp/fable49-evidence-lStqI4/artifacts/meter-disabled-junie-truecolor-frame.log`,
             `/tmp/fable49-evidence-lStqI4/artifacts/meter-disabled-junie-mono-frame.log`,
             `/tmp/fable49-evidence-lStqI4/artifacts/progress-bar-disabled-junie-truecolor-frame.log`,
             and `/tmp/fable49-evidence-lStqI4/artifacts/progress-bar-disabled-junie-mono-frame.log`.
             Exact scope evidence is
             `/tmp/fable49-evidence-lStqI4/artifacts/components-scratch-bless.diff` and
             `/tmp/fable49-evidence-lStqI4/artifacts/moved-keys-exact.txt`.
- tests:     crates/tui/tests/baselines/components.txt (disabled lines only),
             render::components::{hint_bar,meter,progress_bar}::disabled,
             components::a_forced_component_resolves_its_props_derived_state,
             theme::readiness_states_are_digest_distinct
- historical-moved: 22 keys (superseded by stabilized reconciliation below):
  render::components::hint_bar::disabled 120 40 junie mono f19ec0db80c3b5ce → f5fc6531be4cfe81
  render::components::hint_bar::disabled 120 40 junie truecolor 8486cfc84d1044b4 → 9e0c5d44f19bd3fc
  render::components::hint_bar::disabled 120 40 paper mono 270944ce827da10c → b8930e040b525b61
  render::components::hint_bar::disabled 120 40 paper truecolor 36fd1a4de917c1ea → d28c51d6fda7fba2
  render::components::hint_bar::disabled 40 10 junie mono 5e3cb57795df81ce → ab06451beff9e981
  render::components::hint_bar::disabled 40 10 junie truecolor bb413216f6341e24 → 5a556c0a72d55bac
  render::components::hint_bar::disabled 40 10 paper mono 96e5d9b1527eaa2c → 733043d7bddd5701
  render::components::hint_bar::disabled 40 10 paper truecolor 5652e7475dd3bfea → e21dd9a6697609a2
  render::components::meter::disabled 120 40 junie mono 42061673f3aa7732 → c3944936b57ee94a
  render::components::meter::disabled 120 40 junie truecolor 100a588eaff7313c → 1b281fdd0019d6fd
  render::components::meter::disabled 120 40 paper truecolor ae585925d5bcadcd → da64fde7430c6d53
  render::components::meter::disabled 40 10 junie mono 393701c0748961da → 9ef934503dd41e92
  render::components::meter::disabled 40 10 junie truecolor b0cce946b4a54db4 → 811f5d08644e57d5
  render::components::meter::disabled 40 10 paper truecolor 7d92440289fcdb09 → 9913b064c201a977
  render::components::progress_bar::disabled 120 40 junie mono 498e91dddd35d5c0 → ef46093a72cdf519
  render::components::progress_bar::disabled 120 40 junie truecolor fd048ee1e85160fe → 7233e64a8496e588
  render::components::progress_bar::disabled 120 40 paper mono 0e05654797fcb6f2 → c13aab919eb20ccf
  render::components::progress_bar::disabled 120 40 paper truecolor 876e5d3a314c2c66 → 15a0109f80ae29c6
  render::components::progress_bar::disabled 40 10 junie mono 16ade91bcaec3208 → 50c4f3da93780841
  render::components::progress_bar::disabled 40 10 junie truecolor 716d8ca6f6ff5f0e → 5001dc42873b02d8
  render::components::progress_bar::disabled 40 10 paper mono c0fdab768ae9f842 → 6fea492cd57e287f
  render::components::progress_bar::disabled 40 10 paper truecolor 481ea61b5734b122 → f6decdbd451ab17a
- added:     none
- class:     fix
- reason:    §20.10 item 20 (forced state preserves the props-derived readiness state).
             The old output erased `ERROR`, making HintBar and ProgressBar error fixtures
             byte-identical to their ready defaults at truecolor and leaving Meter's declared
             error rule unreachable; the corrected output paints the specified error affordance.
```

## Item 10 — Derived-hint performance proof is recorded

```
- surface:   crates/tui performance inventory; no rendered baseline
- captures:  none — this is a numeric allocation proof, not a frame
- tests:     crates/tui/tests/perf_baseline.txt, frame_hintbar_derived
- moved:     none
- added:     1 key: `frame_hintbar_derived`
- class:     intended
- reason:    §20.10 item 10 (hints derived from component bindings). This is the
             first checked-in allocation record for the unchanged-focus cache that
             makes the derived footer practical; it changes no rendered cells.
```

## Item 22 — First-generation Panel, SplitPane, TextViewport and Tree digests

```
- surface:   junie-tui/{panel,split_pane,text_viewport,tree}
             / {default,focused,hovered,pressed,disabled,selected,editing,empty}
             @ {120x40, 40x10} / {junie, paper} / {truecolor, mono}
- captures:  none under `shots/`; the matrix is headless. Clean Junie 120×40
             truecolor/mono frame text for all 32 states is under
             `/tmp/fable-slice4-final-BPvDuo/repo/artifacts/frames/new-components-clean/{truecolor,mono}/{panel,split_pane,text_viewport,tree}--*.txt`;
             the complete inventory is `artifacts/new-components-320.manifest`.
- tests:     crates/tui/tests/baselines/components.txt,
             render::components::{panel,split_pane,text_viewport,tree}::*
- moved:     none
- added:     256 keys: `render::components::{panel,split_pane,text_viewport,tree}::{default,focused,hovered,pressed,disabled,selected,editing,empty} {120 40,40 10} {junie,paper} {truecolor,mono}`
- class:     intended
- reason:    §20.10 item 22 (first-generation component digests). These keys have
             no before-image and pin future drift; they are not visual approval.
```

### 22b — TextViewport performance proofs are recorded

```
- surface:   crates/tui performance inventory; no rendered baseline
- captures:  none — these are numeric allocation/index proofs, not frames
- tests:     crates/tui/tests/perf_baseline.txt, viewport_100k_lines_push,
             viewport_100k_lines_render, viewport_layout_10k_grapheme_line
- moved:     none
- added:     3 keys: `viewport_100k_lines_push`, `viewport_100k_lines_render`, `viewport_layout_10k_grapheme_line`
- class:     intended
- reason:    §20.10 item 22 (first-generation TextViewport component evidence),
             paired with §20.9 item 7's binding performance obligations. These
             first numeric records change no rendered cells.
```

## Item 23 — ChipBar semantic identity and owned-patch correction

```
- surface:   junie-tui/chip_bar/{default,focused,hovered,pressed,disabled,selected,editing}
             @ {120x40, 40x10} / {junie, paper} / {truecolor, mono}
- captures:  none under `shots/`; the matrix is headless. Exact scratch diff/key
             evidence is `/tmp/fable-slice4-final-BPvDuo/repo/artifacts/{components-scratch.diff,moved-keys-exact.txt}`.
             Clean Junie 120×40 truecolor/mono before and after frames are listed by
             `artifacts/moved-before-22.manifest` and `artifacts/moved-after-22.manifest`.
- tests:     crates/tui/tests/baselines/components.txt, render::components::chip_bar::*
- historical-moved: 39 keys (superseded by stabilized reconciliation below):
  render::components::chip_bar::default 120 40 junie mono 20dc057a9ec11402 → d410e30a8fa1287b
  render::components::chip_bar::default 120 40 junie truecolor 13a30c0fc4475b5e → ec65ee8e34084547
  render::components::chip_bar::default 120 40 paper mono 57b2cbc4e84613fa → af6ee1d9868d153b
  render::components::chip_bar::default 120 40 paper truecolor 0e3f6a0b142b8888 → 5262957dbd38ff11
  render::components::chip_bar::disabled 120 40 junie mono df02c34f10930326 → fe16ace9bdc28c17
  render::components::chip_bar::disabled 120 40 junie truecolor 456cd66502145504 → 10df00da77faf209
  render::components::chip_bar::disabled 120 40 paper mono 7cd4de1e637d0e82 → 37a6afcff9f15bb3
  render::components::chip_bar::disabled 120 40 paper truecolor 6ea3f973aa306304 → f9be19ec6dc64333
  render::components::chip_bar::disabled 40 10 junie mono b5359bbe0124832e → 83ba4197f209a1b8
  render::components::chip_bar::disabled 40 10 junie truecolor 334569a9d20c1b4e → f678035343e2efd2
  render::components::chip_bar::disabled 40 10 paper truecolor 49fb1321e27eccf2 → 2ed132f9870e09a7
  render::components::chip_bar::editing 120 40 junie mono 20dc057a9ec11402 → d410e30a8fa1287b
  render::components::chip_bar::editing 120 40 junie truecolor 13a30c0fc4475b5e → ec65ee8e34084547
  render::components::chip_bar::editing 120 40 paper mono 57b2cbc4e84613fa → af6ee1d9868d153b
  render::components::chip_bar::editing 120 40 paper truecolor 0e3f6a0b142b8888 → 5262957dbd38ff11
  render::components::chip_bar::focused 120 40 junie mono 6ad5af32d1c4ed26 → 88693cac752be367
  render::components::chip_bar::focused 120 40 junie truecolor 3d29208b34fc2d1e → 782d064c5cace447
  render::components::chip_bar::focused 120 40 paper mono 870de4e852c919c6 → 1a0425581edf8767
  render::components::chip_bar::focused 120 40 paper truecolor 489998de99db8348 → 90b548aae4ab93e1
  render::components::chip_bar::hovered 120 40 junie mono 857bb40ac027e8bc → 9aac1adf3d5f55a5
  render::components::chip_bar::hovered 120 40 junie truecolor 57ce6520fe16bda8 → 8992c7433e911eaf
  render::components::chip_bar::hovered 120 40 paper mono 57b2cbc4e84613fa → af6ee1d9868d153b
  render::components::chip_bar::hovered 120 40 paper truecolor 7a7d32240fba017d → 1270ae2154444e2a
  render::components::chip_bar::hovered 40 10 junie truecolor 83292ce01ea8e206 → f539bd9db384fbf0
  render::components::chip_bar::hovered 40 10 paper truecolor 28cd277ff3d2aef5 → 50f1629e47930e17
  render::components::chip_bar::pressed 120 40 junie mono eda5fcfe6a098dbe → 3ec2f185b7fc82ab
  render::components::chip_bar::pressed 120 40 junie truecolor bca0f9fd51095e7e → 0511d60b0fa8b033
  render::components::chip_bar::pressed 120 40 paper mono 020604870a05c0ba → 1f7343eff38605af
  render::components::chip_bar::pressed 120 40 paper truecolor 86e93e82da6defaf → b9785905ac758222
  render::components::chip_bar::pressed 40 10 junie mono 38e8004d2b8528a6 → 465c9be2eaf8153a
  render::components::chip_bar::pressed 40 10 junie truecolor ab3e79bbeb642964 → ac302d959cd1b398
  render::components::chip_bar::pressed 40 10 paper mono e3dcedc6ecd81666 → b0631a37d189adf2
  render::components::chip_bar::pressed 40 10 paper truecolor 99ae477a548be61b → 41c95aa0cccd1a6f
  render::components::chip_bar::selected 120 40 junie mono 42faaf09081f03e7 → d03067a49b2395be
  render::components::chip_bar::selected 120 40 junie truecolor 59b3e16324d9bdee → a5ab322ccf96fbe9
  render::components::chip_bar::selected 120 40 paper mono d27abdc40d74076f → 3d7d833bed6dbc3e
  render::components::chip_bar::selected 120 40 paper truecolor 0ea85aa20aae8da5 → 13ea3f709a84aaea
  render::components::chip_bar::selected 40 10 junie truecolor 130f7690a0735bc8 → b7249dc8743bd97e
  render::components::chip_bar::selected 40 10 paper truecolor af5c9c27e0e71e5d → 8cff4fad8972e657
- added:     none
- class:     fix
- reason:    §20.10 item 23 (ChipBar semantic identity and owned-patch contract).
             Checked markers, caller META and automatic part patches now follow
             the accepted contract; label/pad/close/overflow geometry is retained.
```

## Item 24 — First-generation TooSmall digests

```
- surface:   junie-tui/too_small/{default,focused,hovered,pressed,disabled,selected,editing,empty}
             @ {120x40, 40x10} / {junie, paper} / {truecolor, mono}
- captures:  none under `shots/`; the matrix is headless. Clean Junie 120×40
             truecolor/mono frame text is under
             `/tmp/fable-slice4-final-BPvDuo/repo/artifacts/frames/new-components-clean/{truecolor,mono}/too_small--*.txt`;
             exact scope is `artifacts/added-keys-exact.txt`.
- tests:     crates/tui/tests/baselines/components.txt, render::components::too_small::*
- moved:     none
- added:     64 keys: `render::components::too_small::{default,focused,hovered,pressed,disabled,selected,editing,empty} {120 40,40 10} {junie,paper} {truecolor,mono}`
- class:     intended
- reason:    §20.10 item 24 (first-generation TooSmall digests). No key has a
             before-image; these lines pin future drift and are not visual approval.
```

## Item 25 — First-generation ScrollRegion, NavList, Steps and Grid digests

```
- surface:   junie-tui/{scroll_region,nav_list,steps,grid}
             / {default,focused,hovered,pressed,disabled,selected,editing,empty}
             @ {120x40, 40x10} / {junie, paper} / {truecolor, mono}
- captures:  none under `shots/`; the matrix is headless. Clean Junie 120×40
             truecolor/mono frame text is under
             `/tmp/fable-slice4-final-BPvDuo/repo/artifacts/frames/new-components-clean/{truecolor,mono}/{scroll_region,nav_list,steps,grid}--*.txt`;
             exact scope is `artifacts/added-keys-exact.txt`.
- tests:     crates/tui/tests/baselines/components.txt,
             render::components::{scroll_region,nav_list,steps,grid}::*
- moved:     none
- added:     256 keys: `render::components::{scroll_region,nav_list,steps,grid}::{default,focused,hovered,pressed,disabled,selected,editing,empty} {120 40,40 10} {junie,paper} {truecolor,mono}`
- class:     intended
- reason:    §20.10 item 25 (first-generation component digests). No key has a
             before-image; item 21 separately governs pressed-mono thumb appearance.
```

## Item 26 — First-generation DiffView and CodeEditor digests

```
- surface:   junie-tui/{diff_view,code_editor}
             / {default,focused,hovered,pressed,disabled,selected,editing,empty}
             @ {120x40, 40x10} / {junie, paper} / {truecolor, mono}
- captures:  none under `shots/`; the matrix is headless. Clean Junie 120×40
             truecolor/mono frame text is under
             `/tmp/fable-slice4-final-BPvDuo/repo/artifacts/frames/new-components-clean/{truecolor,mono}/{diff_view,code_editor}--*.txt`;
             exact scope is `artifacts/added-keys-exact.txt`.
- tests:     crates/tui/tests/baselines/components.txt,
             render::components::{diff_view,code_editor}::*
- moved:     none
- added:     128 keys: `render::components::{diff_view,code_editor}::{default,focused,hovered,pressed,disabled,selected,editing,empty} {120 40,40 10} {junie,paper} {truecolor,mono}`
- class:     intended
- reason:    §20.10 item 26 (first-generation DiffView and CodeEditor digests).
             The lines have no before-image and pin future drift; fresh independent
             frame review and an authorized serial bless remain pending.
```

## Item 27 — First-generation Slice-4F component digests

```
- surface:   junie-tui/{filter_list,picker,completion,form,context_menu,help_overlay,
             menu_bar,picker_chain,wizard}
             / {default,focused,hovered,pressed,disabled,selected,editing,empty}
             @ {120x40, 40x10} / {junie, paper} / {truecolor, mono}
- captures:  none under `shots/`; the matrix is headless. The earlier scratch capture at
             `/tmp/fable-slice4-final-BPvDuo/repo/artifacts/frames/new-components-clean/`
             is retained as historical evidence only: item 31 changed the reference mechanism,
             so fresh scratch frame text and inventory are required before review or bless.
- tests:     crates/tui/tests/baselines/components.txt,
             render::components::{filter_list,picker,completion,form,context_menu,
             help_overlay,menu_bar,picker_chain,wizard}::*
- moved:     none
- added:     576 keys: `render::components::{filter_list,picker,completion,form,context_menu,help_overlay,menu_bar,picker_chain,wizard}::{default,focused,hovered,pressed,disabled,selected,editing,empty} {120 40,40 10} {junie,paper} {truecolor,mono}`
- class:     intended
- reason:    §20.10 item 27 (first-generation Slice-4F component digests). Dialog
             is excluded because retained keys already exist. These lines have no
             before-image; fresh independent frame review and an authorized serial
             bless remain pending. Item 27 owns these first-generation keys; item 31
             may account only for measured corrections to their generated frames.
```

### 27b — Picker borrowed-domain performance proof is recorded

```

- surface:   crates/tui performance inventory; no rendered baseline
- captures:  none — this is a numeric allocation proof, not a frame
- tests:     crates/tui/tests/perf_baseline.txt, picker_100k_borrowed_domain_render
- moved:     none
- added:     1 key: `picker_100k_borrowed_domain_render`
- class:     intended
- reason:    §20.10 item 27 (Slice-4F first-generation component evidence), paired
             with the accepted picker borrowed-domain allocation obligation. This
             first numeric record changes no rendered cells.
```

### 27c — Form steady-frame allocation proof is recorded

```
- surface:   crates/tui performance inventory; no rendered baseline
- captures:  none — this is a numeric allocation proof, not a frame
- tests:     crates/tui/tests/perf_baseline.txt, frame_form_update_draw
- moved:     none
- added:     1 key: `frame_form_update_draw`
- class:     intended
- reason:    §20.10 item 27 (Slice-4F first-generation component evidence), paired
             with the accepted allocation-free borrowing placement obligation.
             This first numeric record changes no rendered cells.
```

## Item 28 — Semantic selection is state-owned

```
- surface:   junie-tui/{list,tabs,radio_group,select} semantic-selection states
             @ {120x40, 40x10} / {junie, paper} / the exact colours below
- captures:  none under `shots/`; the matrix is headless. Exact scratch evidence is
             `/tmp/fable-slice4-final-VPCSC9/artifacts/components-scratch.diff` and
             `/tmp/fable-slice4-final-VPCSC9/artifacts/moved-keys-exact.txt`.
- tests:     crates/tui/tests/baselines/components.txt,
             render::components::{list,tabs,radio_group,select}::*
- historical-moved: 40 keys (rejected pre-reconciliation scratch record; superseded below):
  render::components::list::selected 120 40 junie mono 948062195792e82f → 164c36b18586726f
  render::components::list::selected 120 40 junie truecolor a09c6fe210fe945d → ce3be201fd5894f6
  render::components::list::selected 120 40 paper mono 69642bbd1fad5265 → 73efdde5cdb3a6c5
  render::components::list::selected 120 40 paper truecolor 494b4c33bccf8984 → 82e97d15fc758a0e
  render::components::list::selected 40 10 junie mono 6c049b1cb101664f → deac25911bc9718f
  render::components::list::selected 40 10 junie truecolor 2d8c90c882efad9d → 4e5dee5a5d3d3a76
  render::components::list::selected 40 10 paper mono b44e759285f43a85 → 447bcd08741e2165
  render::components::list::selected 40 10 paper truecolor 950b530b8705f714 → 85d6dbd488cef51e
  render::components::radio_group::pressed 120 40 junie mono 7c9348eec50e02e3 → 4d18b4872f35ce4b
  render::components::radio_group::pressed 120 40 paper mono 5e4e76a54792f289 → c7333b9ec237078d
  render::components::radio_group::pressed 40 10 junie mono 6f0aebd819bba103 → ec6c5261030351cb
  render::components::radio_group::pressed 40 10 paper mono 1d9365ff80a86ca9 → d41387891cdebb0d
  render::components::select::default 120 40 junie mono d56280a37a6d91b4 → 7e46b8873476eb92
  render::components::select::default 120 40 paper mono 439b7dee547eff04 → 8a6bf73ebb667692
  render::components::select::default 40 10 junie mono 762f74be76665454 → d64d86b5cd399d32
  render::components::select::default 40 10 paper mono 33254b34e59736a4 → 63fe454843c795b2
  render::components::select::editing 120 40 junie mono d56280a37a6d91b4 → 7e46b8873476eb92
  render::components::select::editing 120 40 paper mono 439b7dee547eff04 → 8a6bf73ebb667692
  render::components::select::editing 40 10 junie mono 762f74be76665454 → d64d86b5cd399d32
  render::components::select::editing 40 10 paper mono 33254b34e59736a4 → 63fe454843c795b2
  render::components::select::focused 120 40 junie mono 88a794db4ad7438a → 1b56e3dfe0baa374
  render::components::select::focused 120 40 paper mono 0214fd848121d88a → 90c04a6bc8dadf84
  render::components::select::focused 40 10 junie mono cfdca6c48e84fdaa → ba863a12a0524d94
  render::components::select::focused 40 10 paper mono cd220e2c1e56d62a → 45aec8182a762d24
  render::components::select::hovered 120 40 junie mono d56280a37a6d91b4 → 7e46b8873476eb92
  render::components::select::hovered 120 40 paper mono 439b7dee547eff04 → 8a6bf73ebb667692
  render::components::select::hovered 40 10 junie mono 762f74be76665454 → d64d86b5cd399d32
  render::components::select::hovered 40 10 paper mono 33254b34e59736a4 → 63fe454843c795b2
  render::components::select::pressed 120 40 junie mono 17bf131df914c266 → c4e91c58984eb68c
  render::components::select::pressed 120 40 paper mono 33ca9d784756b7a6 → 20cc3c074be6adaa
  render::components::select::pressed 40 10 junie mono eafa6492283387c6 → 75aee1f6e6798d6c
  render::components::select::pressed 40 10 paper mono cec1880d683c3a06 → 5dadb5bc71aedd0a
  render::components::tabs::selected 120 40 junie mono 643602b7923c4efc → 03263f48effb1240
  render::components::tabs::selected 120 40 junie truecolor 491cb79a666a433f → 345a10235d65f11f
  render::components::tabs::selected 120 40 paper mono 5faba1dd1e4da8d8 → ad59d0d63e9883e4
  render::components::tabs::selected 120 40 paper truecolor 292f7884f72e573c → 9aa17c11b0135f9e
  render::components::tabs::selected 40 10 junie mono 204d5a3787674b8a → 8575598b028539ba
  render::components::tabs::selected 40 10 junie truecolor 5ca402c6a6d5efea → 870f0d29c4b57860
  render::components::tabs::selected 40 10 paper mono 62b115316abc8a4a → a26add1c3797f552
  render::components::tabs::selected 40 10 paper truecolor a53ecdcfcd7ff4e9 → 2eb100378a3b3c43
- added:     none
- class:     fix
- reason:    §20.10 item 28 (state-owned semantic selection). A forced state is
             only a visual probe; List/Tabs state, RadioGroup controlled value and
             Select value exclusively own semantic selection. Review checks one
             marker/active tab, Radio one chosen plus exact press with no second,
             Select Ada/one chosen/brackets only pressed/no geometry or content
             drift, and containment. Machine proof is the forced-selection unit
             coverage plus complete conformance.
```

The Item-28 block above is retained as the historical, rejected pre-reconciliation scratch record;
it is not current blessing evidence. §20.10 now limits item 28 to its accepted 16-key authority,
and item 31 requires a fresh scratch measurement before any of those proposed movements is
classified or blessed.

## Item 29 — Select owns vertical disclosure glyphs

```
- surface:   junie-tui Select closed/open field disclosure, all themes and color levels
- captures:  none under `shots/`; exact headless unit evidence is
             `select::select_disclosure_is_exact_for_both_themes_and_color_levels`
- tests:     no retained baseline line changes; the eight-state component matrix draws Select
             closed. Focused Select and builtin-glyph unit tests cover the live open state.
- moved:     none
- added:     none
- class:     fix
- reason:    §20.10 item 29. The old open Select reused the generic collapsed-tree glyph `▸`,
             pointing sideways instead of communicating an open dropdown. Dedicated
             `SelectClosed`/`SelectOpen` roles render `▾`/`▴`. The change affects all color levels,
             preserves the right-minus-two marker cell and popup geometry, and requires no
             retained eight-state baseline movement because those fixtures are closed.
```

## Item 30 — Dialog targets one owned control

```
- surface:   junie-tui/dialog reference states, all retained theme/colour/size cells
- captures:  pending fresh discarded-scratch before/after frame text; no current capture is
             accepted for blessing
- tests:     dialog::a_reference_dialog_registers_no_control,
             dialog::reference_dialog_targets_one_owned_control_without_broadcasting,
             render::components::dialog::*
- moved:     see the stabilized item-30 entry below: exactly 28 keys
- added:     none
- class:     fix
- reason:    §20.10 item 30. A no-prompt Dialog targets only its first enabled action for
             runtime reference states; prompt focus/editing targets only the input. Root chrome
             and siblings never inherit the target state. Exact movement must be captured and
             classified before any retained baseline is changed.
```

Item 30 owns the Dialog correction, not Dialog's already-retained first-generation baseline.
No item-30 bless has run or is authorized by this entry.

## Item 31 — Central exact-target reference scope

```
- surface:   every component reference fixture migrated from component-local forced propagation
             to outer Ui::reference exact-target or inert-None scope
- captures:  pending fresh discarded-scratch baseline diff and clean frame-text dumps after the
             source tree stabilizes; historical scratch bundles are not blessing evidence
- tests:     architecture::legacy_forced_state_apis_are_absent,
             architecture::reference_rendering_is_ui_scoped, complete conformance suite,
             render::components::*
- moved:     see the stabilized item-31 entry below: exactly 177 keys
- added:     item 27 retains sole ownership of its 576 first-generation Slice-4F keys; item 31
             authorizes no unmeasured addition
- class:     fix
- reason:    §20.10 item 31 removes the broadcast/live-leak defect class. The scope makes the
             entire subtree inert while only one declared component/item/part receives runtime
             reference bits. Semantic state stays caller-owned. Accept only exact-target or
             suppression corrections; reject live/default/semantic, geometry/content, multiple-
             target, registration and unaccounted changes.
```

No retained component or performance baseline has been regenerated or blessed for items 27, 30
or 31. The required order remains change → fresh scratch capture → exact classification →
independent review → separately authorized serial bless.
 
## Stabilized scratch reconciliation — 280 moved / 1,280 added
 
The authoritative comparison is `components.before.txt` SHA-256
`1ab8e9205a19069ff5f9d97d675df77e6051c6195ad7a882766163cc2e744c9e` to the
scratch-generated `components.txt` SHA-256
`4c4dd527261acc03431858db024f884385463a40131b16ae340564da9ca42299` under
`/private/tmp/terminal-components-baseline-review.qbw8Cu/`. The six entries below exhaust the
280 moved keys. Existing items 22, 24, 25, 26 and 27 exhaust the 1,280 first-generation added
keys (20 components × 64) and remain their sole owners. No live baseline or bless is involved.
 
## Item 1 — final mono fallback movements

```
- surface:   crates/tui/tests/baselines/components.txt stabilized scratch comparison
- captures:  `/private/tmp/terminal-components-baseline-review.qbw8Cu/artifacts/components.before.txt`
             and `/private/tmp/terminal-components-baseline-review.qbw8Cu/repo/crates/tui/tests/baselines/components.txt`
- tests:     render::components::* plus the item-specific unit/conformance proofs above
- moved:     4 keys (rewritten to the merged diff):
  render::components::field::disabled 120 40 junie mono 6ea61957d4835dac -> 7c03b1f135f5eb54
  render::components::field::disabled 40 10 junie mono 8a8e49368586582c -> c7a41948ef3b26d4
  render::components::select::pressed 120 40 junie mono c4e91c58984eb68c -> 23e63edf534b4b50
  render::components::select::pressed 40 10 junie mono 75aee1f6e6798d6c -> 699174892d610db0
- added:     none
- class:     fix
- reason:    §20.10 item 1; the final field disabled and Select pressed mono fallbacks.
```

## Item 20 — final props-readiness movements

```
- surface:   crates/tui/tests/baselines/components.txt stabilized scratch comparison
- captures:  `/private/tmp/terminal-components-baseline-review.qbw8Cu/artifacts/components.before.txt`
             and `/private/tmp/terminal-components-baseline-review.qbw8Cu/repo/crates/tui/tests/baselines/components.txt`
- tests:     render::components::* plus the item-specific unit/conformance proofs above
- moved:     6 keys (rewritten to the merged diff):
  render::components::hint_bar::disabled 120 40 junie mono 592985dbcc22164b -> dea41b81a52f0e47
  render::components::hint_bar::disabled 40 10 junie mono d2a777a0b59c624b -> a82e153d933d7267
  render::components::meter::disabled 120 40 junie mono 72b307545bf95a32 -> 278af2a688e8665c
  render::components::meter::disabled 40 10 junie mono b66ee080f8eb64fa -> 9da8e06150a319a0
  render::components::progress_bar::disabled 120 40 junie mono ff0f692938ab8559 -> 11fb9771b26ce04b
  render::components::progress_bar::disabled 40 10 junie mono 31e29eaf4ff25581 -> a9003cbfb520c6c3
- added:     none
- class:     fix
- reason:    §20.10 item 20; disabled readiness reporters retain their props-owned error affordance.
```

## Item 23 — final ChipBar semantic movements

```
- surface:   crates/tui/tests/baselines/components.txt stabilized scratch comparison
- captures:  `/private/tmp/terminal-components-baseline-review.qbw8Cu/artifacts/components.before.txt`
             and `/private/tmp/terminal-components-baseline-review.qbw8Cu/repo/crates/tui/tests/baselines/components.txt`
- tests:     render::components::* plus the item-specific unit/conformance proofs above
- moved:     9 keys (rewritten to the merged diff):
  render::components::chip_bar::default 120 40 junie mono d410e30a8fa1287b -> e649e48c2d94a7d7
  render::components::chip_bar::disabled 120 40 junie mono fe16ace9bdc28c17 -> 1a9038d7d9da10dd
  render::components::chip_bar::disabled 40 10 junie mono 83ba4197f209a1b8 -> f294ad05a2d2de2e
  render::components::chip_bar::editing 120 40 junie mono d410e30a8fa1287b -> e649e48c2d94a7d7
  render::components::chip_bar::focused 120 40 junie mono d410e30a8fa1287b -> e649e48c2d94a7d7
  render::components::chip_bar::hovered 120 40 junie mono d410e30a8fa1287b -> e649e48c2d94a7d7
  render::components::chip_bar::pressed 120 40 junie mono 3ec2f185b7fc82ab -> b2ebcdfee908d6af
  render::components::chip_bar::pressed 40 10 junie mono 465c9be2eaf8153a -> f7276ed25e2ffb8c
  render::components::chip_bar::selected 120 40 junie mono d03067a49b2395be -> cb6db8076587b494
- added:     none
- class:     fix
- reason:    §20.10 item 23; ChipBar semantic identity, metadata, marker and owned-patch correction.
```

## Item 28 — final semantic-selection movements

```
- surface:   crates/tui/tests/baselines/components.txt stabilized scratch comparison
- captures:  `/private/tmp/terminal-components-baseline-review.qbw8Cu/artifacts/components.before.txt`
             and `/private/tmp/terminal-components-baseline-review.qbw8Cu/repo/crates/tui/tests/baselines/components.txt`
- tests:     render::components::* plus the item-specific unit/conformance proofs above
- moved:     4 keys (rewritten to the merged diff):
  render::components::button::selected 120 40 junie mono 13d0bae25b64bacc -> 96b4d865e2982360
  render::components::button::selected 40 10 junie mono 0f80b7f1cc3b9e4c -> 0cfe2af0722385e0
  render::components::radio_group::pressed 120 40 junie mono c1fcc82633a44665 -> 2ff02eaec05151a3
  render::components::radio_group::pressed 40 10 junie mono 96df91bc5acc71e5 -> 5ecef9c227f483a3
- added:     none
- class:     fix
- reason:    §20.10 item 28; selection comes from controlled semantic state, never reference flags.
```

## Item 30 — final Dialog exact-target movements

```
- surface:   crates/tui/tests/baselines/components.txt stabilized scratch comparison
- captures:  `/private/tmp/terminal-components-baseline-review.qbw8Cu/artifacts/components.before.txt`
             and `/private/tmp/terminal-components-baseline-review.qbw8Cu/repo/crates/tui/tests/baselines/components.txt`
- tests:     render::components::* plus the item-specific unit/conformance proofs above
- moved:     6 keys (rewritten to the merged diff):
  render::components::dialog::disabled 120 40 junie mono b737d61b383ec6f5 -> 576714b0c37c1eff
  render::components::dialog::disabled 40 10 junie mono 081728ded55d1575 -> 5b8d9fce2ae21747
  render::components::dialog::focused 120 40 junie mono d057ba49733734b3 -> a12e35d7f3be32c3
  render::components::dialog::focused 40 10 junie mono 4dfe262b3ac49703 -> 7a1b32a1cfb75e5b
  render::components::dialog::pressed 120 40 junie mono 3f5dbd922edd1637 -> 8ef17228c7bff7eb
  render::components::dialog::pressed 40 10 junie mono 9c5b0be834361ad7 -> bda5a78ad988c6b3
- added:     none
- class:     fix
- reason:    §20.10 item 30; Dialog reference state targets one owned prompt/action without broadcast.
```

## Item 31 — final central-reference movements

```
- surface:   crates/tui/tests/baselines/components.txt stabilized scratch comparison
- captures:  `/private/tmp/terminal-components-baseline-review.qbw8Cu/artifacts/components.before.txt`
             and `/private/tmp/terminal-components-baseline-review.qbw8Cu/repo/crates/tui/tests/baselines/components.txt`
- tests:     render::components::* plus the item-specific unit/conformance proofs above
- moved:     64 keys (rewritten to the merged diff):
  render::components::brand::disabled 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::disabled 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::brand::hovered 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::hovered 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::brand::pressed 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::pressed 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::chip_bar::focused 40 10 junie mono fff5116446505e3c -> 3f889ea34564264c
  render::components::chip_bar::hovered 40 10 junie mono fff5116446505e3c -> 3f889ea34564264c
  render::components::empty::editing 120 40 junie mono 796461203dd06a7e -> 534ba50f6badf4ec
  render::components::empty::editing 40 10 junie mono b0a749381c6792fe -> 740e0de8851c8b6c
  render::components::field::selected 120 40 junie mono df6c39650b0ade4c -> bbfada53d625d404
  render::components::field::selected 40 10 junie mono 623ecdc1e6dcb26c -> c379e0d03ce5b5a4
  render::components::hint_bar::editing 120 40 junie mono 1a74082eb268df4b -> be8f30b338afce4f
  render::components::hint_bar::editing 40 10 junie mono 3192a28a54d74b4b -> adafaccc1271d82f
  render::components::hint_bar::focused 120 40 junie mono aaf7cf28e13e826a -> f4af0f5e20d66b32
  render::components::hint_bar::focused 40 10 junie mono fdd3f500925e3a6a -> cc6d9bb7f8457852
  render::components::hint_bar::pressed 120 40 junie mono 197dc165a722ba63 -> a6b103102a062367
  render::components::hint_bar::pressed 40 10 junie mono 8f236bbf0fb9ea63 -> 12cc86ff03ed6247
  render::components::list::disabled 120 40 junie mono 164c36b18586726f -> fc98c428f6c305b1
  render::components::list::disabled 40 10 junie mono deac25911bc9718f -> 07a3a2e2097a7dd1
  render::components::list::focused 120 40 junie mono 164c36b18586726f -> fc98c428f6c305b1
  render::components::list::focused 40 10 junie mono deac25911bc9718f -> 07a3a2e2097a7dd1
  render::components::list::hovered 120 40 junie truecolor 96086a8fb7489f5a -> 413f0d1203429a0a
  render::components::list::hovered 40 10 junie truecolor 95c30a93fad87dda -> f488a977e2ce582a
  render::components::list::pressed 120 40 junie mono cb5e7c9046398305 -> f7555667a42767a7
  render::components::list::pressed 40 10 junie mono 7f93e89c2dddc765 -> 9bff234723dacf07
  render::components::meter::editing 120 40 junie mono 664372a5c0eb31d2 -> 5f0e50eb5d01e034
  render::components::meter::editing 40 10 junie mono d78165edbfbe84da -> f8cae75e7ed9c298
  render::components::meter::focused 120 40 junie mono fab1931d8ce0ae06 -> e71bd00db82600aa
  render::components::meter::focused 40 10 junie mono 5c7403a5b598074e -> 97718db10ff7b28e
  render::components::meter::pressed 120 40 junie mono 505b40d8b1e4656a -> 47cbac4c63a0df9c
  render::components::meter::pressed 40 10 junie mono cadbc9890f6f4f42 -> c2014767122ef000
  render::components::progress_bar::editing 120 40 junie mono 6a4f7a72886db1b1 -> a5c260c9d685e4fb
  render::components::progress_bar::editing 40 10 junie mono bd9ca2fdffa8f379 -> 567b51a35bef7473
  render::components::progress_bar::focused 120 40 junie mono 1da207d050ccbe00 -> 385f9dd546b0d75c
  render::components::progress_bar::focused 40 10 junie mono 640c9e6d0236a448 -> b161d6cb40a37114
  render::components::progress_bar::pressed 120 40 junie mono 2db6b19ba7a37a19 -> 67261ac4746eb963
  render::components::progress_bar::pressed 40 10 junie mono 6fb9de54d8806411 -> ec24ad5de83c540b
  render::components::radio_group::default 120 40 junie mono 23253cfc778734b7 -> a61fe8ade17a3465
  render::components::radio_group::default 40 10 junie mono 783665ea3658b177 -> 2bad9bce00760c25
  render::components::radio_group::disabled 120 40 junie mono 0b3191ebe74270d5 -> 41feea0417c6f64d
  render::components::radio_group::disabled 40 10 junie mono ac10dc1dca9bd455 -> 7e9b20271996868d
  render::components::radio_group::editing 120 40 junie mono 23253cfc778734b7 -> a61fe8ade17a3465
  render::components::radio_group::editing 40 10 junie mono 783665ea3658b177 -> 2bad9bce00760c25
  render::components::radio_group::focused 120 40 junie mono 23253cfc778734b7 -> a61fe8ade17a3465
  render::components::radio_group::focused 40 10 junie mono 783665ea3658b177 -> 2bad9bce00760c25
  render::components::radio_group::hovered 120 40 junie mono 23253cfc778734b7 -> a61fe8ade17a3465
  render::components::radio_group::hovered 40 10 junie mono 783665ea3658b177 -> 2bad9bce00760c25
  render::components::spinner::disabled 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::disabled 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::spinner::focused 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::focused 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::spinner::pressed 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::pressed 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::status_bar::disabled 120 40 junie mono dba9eaec3520c297 -> 431c9e4d28001609
  render::components::status_bar::disabled 40 10 junie mono c06e4f5f9ad85657 -> 2b9805cdaadbed79
  render::components::status_bar::editing 120 40 junie mono bd19c6ccb9b63237 -> 6672f81d23ca0a81
  render::components::status_bar::editing 40 10 junie mono 81e080b38aa0ee77 -> 78078a1ef4270b31
  render::components::tabs::disabled 120 40 junie mono 03263f48effb1240 -> 269cea1731877156
  render::components::tabs::disabled 40 10 junie mono 8575598b028539ba -> 231249d22e78dc9e
  render::components::tabs::focused 120 40 junie mono 03263f48effb1240 -> 269cea1731877156
  render::components::tabs::focused 40 10 junie mono 8575598b028539ba -> 231249d22e78dc9e
  render::components::tabs::hovered 120 40 junie mono 03263f48effb1240 -> b10abf07af527610
  render::components::tabs::hovered 40 10 junie mono 8575598b028539ba -> d6a2a1e72a8a6d44
- added:     none
- class:     fix
- reason:    §20.10 item 31; only exact-target/suppression corrections caused by central Ui::reference.
```


## Item 32 — First-generation per-app visual baselines (§74.3)

The app baselines are new review surfaces created by the workspace split. Their first-generation
keys are not movements of the frozen root evidence. The complete capture matrix and digest runs
are now recorded below; frame review remains separate from hash approval.

```
- surface:   showcase: 22 pages × {80x24,100x30,120x40,160x50} × {junie,paper} × {truecolor,256,16,mono}; tablepro: 21 screens × {80x24,120x40} × {junie,paper} × {truecolor,mono}; jackin-preview: 8 routes × {100x30,120x40} × {junie,paper} × {truecolor,mono}
- captures:  `shots/capture-matrix.tsv`, `shots/capture-provenance.json`, and corresponding `shots/<app>_*` ANSI/text/PNG artifacts; the reproducible entry point is `xtask capture-matrix`
- tests:     apps/showcase/tests/visual.rs::showcase_visual_baseline, apps/tablepro/tests/visual.rs::tablepro_visual_baseline, apps/jackin-preview/tests/preview.rs::jackin_visual_baseline
- moved:     none
- added:     352 keys: holla 352 (11×4×2×4)
             `{activities-multi,disk-cleanup,docker-cleanup,first-use,hard-cases,launch-failure,monorepo-child,monorepo-root,remote-host,rust-dirty,upgrade-plan} {80 24,100 30,120 40,160 50} {junie,paper} {truecolor,256,16,mono}`
- class:     intended
- reason:    §20.10 item 32; each migrated app now has cell-exact, theme-aware evidence owned beside its tests, reviewed against running captures and frozen before evidence where available.
```

## Item 33 — First-generation per-app performance baselines (§74.3)

The three app rows are produced by the shared allocation/timing harness. The root performance file
remains frozen; later allocation or byte growth is a regression.

```
- surface:   showcase, tablepro and jackin-preview frame benchmarks / 120x40 / junie / release
- captures:  none under shots/ — performance evidence is the `PERF` line emitted by each package-qualified target and the uploaded CI log
- tests:     apps/showcase/tests/perf_baseline.txt, apps/tablepro/tests/perf_baseline.txt, apps/jackin-preview/tests/perf_baseline.txt; perf_showcase_baseline, perf_tablepro_baseline, perf_jackin_baseline
- moved:     none
- added:     4 rows: `frame_showcase_lists_120x40`, `frame_tablepro_grid_500x12_120x40`, `frame_jackin_capsule_4panes_120x40`, `frame_jackin_manager_100rows_120x40`
- class:     intended
- reason:    §20.10 item 33; each migrated app now records its own measured frame row without modifying the frozen pre-refactor baseline.
```

### Item 33a — Jackin interaction performance row

```
- surface:   jackin-preview/manager key movement @ 120x40 / junie / release
- captures:  none under `shots/` — performance evidence is the `PERF` line emitted by
             `apps/jackin-preview/tests/perf.rs`
- tests:     apps/jackin-preview/tests/perf_baseline.txt:4, `key_jackin_manager_move`
- moved:     none
- added:     `key_jackin_manager_move`
- class:     intended
- reason:    §20.10 item 33; the runtime key path is measured separately and now proves
             zero allocations after constructor-time manager row projection.
```

### Item 33c — TablePro benchmark identity correction

Commit `708a0722653b2185108a43e746f73f3887017f63` corrected the benchmark's
name to its actual 500-row, 14-column query-result fixture. It retained every
numeric field (`705234 7 990`); no measurement was blessed or threshold raised.
The test now checks the active production grid, registered `id` header and first
rendered result cell. The former name incorrectly claimed twelve columns.

```
- surface:   tablepro/query results frame benchmark / 120x40 / junie / release
- captures:  none under shots/ — numeric performance contract, not a visual baseline;
             reproducible with cargo +1.88.0 test --locked --release -p tablepro --test perf
- tests:     apps/tablepro/tests/perf_baseline.txt; frame_tablepro_grid_500x14_120x40,
             perf_tablepro_baseline
- moved:     none
- added:     `frame_tablepro_grid_500x14_120x40`
- class:     fix
- reason:    §20.10 item 33; replaces the inaccurate frame_tablepro_grid_500x12_120x40
             name while retaining its 7-allocation/990-byte limit and advisory timing.
             Frozen tests/perf_baseline.txt remains unchanged (1030 allocations,
             46119 bytes for its historical 500x12 subject); distinct fixture identities
             do not establish a timing comparison or visual parity.
```

### Item 33b — Root performance inventory entered the migrated baseline

```
- surface:   `crates/tui/tests/perf_baseline.txt` root performance inventory
- captures:  none under `shots/` — these are benchmark rows, not terminal frames
- tests:     `crates/tui/tests/perf_baseline.txt` and the package-qualified performance tests
- moved:     none
- added:     31 rows: `event_dispatch_is_not_o_n`, `fit_10k_grapheme_line_to_80`, `fit_10k_grapheme_line_to_80_wide`, `focus_tab_traversal_ring_200`, `frame_form_update_draw`, `frame_hintbar_derived`, `frame_showcase_buttons_120x40`, `frame_testbackend_empty_120x40`, `fuzzy_10k_grapheme_label`, `hit_registry_size_is_bounded`, `intents_drain_is_o_1_when_the_queue_is_empty`, `list_100k_rows_render`, `list_100k_select_all`, `list_1k_rows_render`, `measure_is_allocation_free`, `mouse_move_over_1000_regions`, `paint_spans_500_rows_is_allocation_free`, `picker_100k_borrowed_domain_render`, `style_backdrop_full_screen_120x40`, `style_downgrade_theme_all_levels`, `style_resolve_10k_parts`, `style_resolve_10k_parts_with_two_overlays`, `textbuffer_offset_at_10k_line`, `textbuffer_pos_of_10k_line`, `truncate_10k_grapheme_line_to_80`, `truncate_middle_10k_to_40`, `viewport_100k_lines_push`, `viewport_100k_lines_render`, `viewport_layout_10k_grapheme_line`, `width_10k_grapheme_line`, `wrap_10k_graphemes_to_80`
- class:     intended
- reason:    §20.10 item 33; the migrated root harness preserves the frozen benchmark inventory while app-specific rows remain owned by each app.
```


---

### Item 33d — Holla's first performance row

Holla joins the migrated apps with its own measured frame row; the frozen root
performance file remains the before-image.

```
- surface:   holla frame benchmark / 120x40 / junie / release
- captures:  none under shots/ — performance evidence is the `PERF` line emitted by the package-qualified target and the uploaded CI log
- tests:     apps/holla/tests/perf_baseline.txt; perf_holla_baseline
- moved:     none
- added:     1 row: `frame_holla_first_use_120x40`
- class:     intended
- reason:    §20.10 item 33; the migrated Holla app records its own measured frame row without modifying the frozen pre-refactor baseline.
```

## Review status — Slice 4 component matrix, independent visual review (2026-09-05)

**Result: FAIL. No bless was performed, and none is authorized.**

- **Scope reviewed.** The **640** review frames for the Slice-4 component matrix, at HEAD
  `26913cc`, by a fresh read-only `read-only analyst` reviewer who did not generate the baselines.
- **Result.** **FAIL.** This supersedes the earlier **PASS** recorded against HEAD `a1759b2` in
  `REFACTORING_STATE.md`. That PASS line is retained there unedited as historical evidence; where
  the two conflict, this record governs.
- **Baseline effect: none.** `crates/tui/tests/baselines/components.txt` is **unchanged** by this
  review. No `BLESS=1` or `PERF_BLESS=1` run was made, no key moved, and no key was added. The
  entries above — items 1, 20, 23, 27, 28, 30, 31 and the stabilized scratch reconciliation — are
  unaffected and are **not** re-classified by this record.
- **Consequence.** Items 27, 30, 31 and every other Slice-4 first-generation item remain
  **unblessed**. §72's closing sentence stands verbatim: independent frame review and separate
  bless authorization remain required, and no retained baseline change is authorized.
- **Attached findings.** The independent app-frame review also found two fix-class defects, both
  mapped before any bless run:
  1. **TablePro 80×24 header collision** — the right overflow count overwrote the final visible
     header title (`shots/tablepro_junie_truecolor_80x24.txt:5`); the geometry fix belongs to
     §20.10 item 7.
  2. **TablePro mono active-cell collapse** — `GRID/CELL + ACTIVE` changed only a color, so the
     mono frame had no visible state distinction (`shots/tablepro_junie_mono_120x40.txt:6`);
     the non-color affordance belongs to §20.10 item 1.
  Both are **class: fix**, and neither authorizes a bless until corrected captures pass a fresh
  independent review.

## Review status — fresh app visual audit (2026-09-05)

**Result: FAIL. No bless is authorized.**

- **TablePro evidence remains stale.** `shots/tablepro_junie_truecolor_80x24.txt:5` still shows
  the pre-fix header collision, and `shots/tablepro_junie_mono_120x40.html:11` still shows the
  pre-fix active-cell collapse. Their provenance records point to `a358272`, before the source
  fix in `e45fde1`; the source tests are current, but the captures do not prove clearance.
- **Showcase Paper/ANSI16 contrast fails.** Metadata labels and subtitles render as bright
  `#e5e5e5` on `#ffffff` in `shots/showcase_paper_16_100x30.html:14-19` and
  `shots/showcase_paper_16_120x40.html:14-19`.
- **Showcase layout and overlays pass** in the inspected 100×30 and 120×40 frames except for
  that contrast defect. **Jackin layout, spacing, contrast and overlays pass** in all inspected
  representative frames.
- **Matrix integrity passes:** 96/96 records are `ok`, all 480 artifacts hash-match, stderr is
  empty for all records, and coverage is 3 apps × 4 color levels × 2 themes × 4 sizes.

This is an attached current FAIL, not a reclassification of the historical Slice-4 review above.
The contrast fix is classified under §20.10 item 32; fresh captures and a new independent review
are required before blessing.
### Item 34 — Migrated app visual repair baselines

The current digest movement is classified before the refreshed capture matrix and independent review. The exact old→new pairs are retained so a later bless cannot hide an unreviewed movement.

```
- surface:   showcase digest repair @ 22 pages / all four sizes / paper + shared Grid/Table cells / all relevant colour levels
- captures:  `shots/capture-matrix.tsv` and corresponding app artifacts; current matrix refresh and independent review are required before approval
- tests:     apps/showcase/tests/baselines/showcase.txt, `showcase_visual_baseline`
- moved:     144 keys:
  Buttons 100 30 paper 16 558271144e30b31c → Buttons 100 30 paper 16 3f31309902fbc2cc
  Buttons 120 40 paper 16 dccc8772b87f1a48 → Buttons 120 40 paper 16 1ed30c75fec13f90
  Buttons 160 50 paper 16 47ad839b761e5410 → Buttons 160 50 paper 16 139920a7b61b8f78
  Buttons 80 24 paper 16 1beae9a30e6e8259 → Buttons 80 24 paper 16 f16961098f9ca069
  Chips & selects 100 30 paper 16 e9235e5d3f165898 → Chips & selects 100 30 paper 16 2d5b5de5ceb4b7c4
  Chips & selects 120 40 paper 16 19b1aa15b2cf0c37 → Chips & selects 120 40 paper 16 60de18f009c46a4b
  Chips & selects 160 50 paper 16 0bf97215d5b26e9f → Chips & selects 160 50 paper 16 75a9d36d5ca4f913
  Chips & selects 80 24 paper 16 1e5e5c5424b37043 → Chips & selects 80 24 paper 16 58e73fc0c981688b
  Chrome 100 30 paper 16 d650923b4f37cd40 → Chrome 100 30 paper 16 30a4effa148b24d0
  Chrome 120 40 paper 16 d41f054d4c1b966e → Chrome 120 40 paper 16 4a55eaaab393617e
  Chrome 160 50 paper 16 db5205ad78ac03fe → Chrome 160 50 paper 16 d05870fe5101df8e
  Chrome 80 24 paper 16 d337323e6754cccc → Chrome 80 24 paper 16 79ea53f3c60abf6c
  Data grid 100 30 junie 16 44c31f31011020ac → Data grid 100 30 junie 16 d09e81e22aaddfd5
  Data grid 100 30 junie 256 4e5326bce6a037f3 → Data grid 100 30 junie 256 40bf28e3a6147096
  Data grid 100 30 junie mono 5044d0254653b4fa → Data grid 100 30 junie mono 9a450827ade2e249
  Data grid 100 30 junie truecolor 27d79eab0db00a5d → Data grid 100 30 junie truecolor 62f0178a5371cb62
  Data grid 100 30 paper 16 1dc77a0e6d81bce0 → Data grid 100 30 paper 16 28796d034234542f
  Data grid 100 30 paper 256 df434a9363579ae3 → Data grid 100 30 paper 256 3c2185e9aedaea7a
  Data grid 100 30 paper mono 00c47eb286838116 → Data grid 100 30 paper mono eee115b41b760955
  Data grid 100 30 paper truecolor 21d9faae4242509d → Data grid 100 30 paper truecolor a5e47ca5e066cb98
  Data grid 120 40 junie 16 115389ca7a77e057 → Data grid 120 40 junie 16 13e26f3af2ee1268
  Data grid 120 40 junie 256 0f00eada3f782d85 → Data grid 120 40 junie 256 ae996e9a7f270d40
  Data grid 120 40 junie mono 68f57e777c8f6573 → Data grid 120 40 junie mono 8f0f39215ca7b908
  Data grid 120 40 junie truecolor b6df914b59e711e5 → Data grid 120 40 junie truecolor 95e4d2bddf60df36
  Data grid 120 40 paper 16 b5f8fe1d8db21c53 → Data grid 120 40 paper 16 7ed0138d4e1fe4a6
  Data grid 120 40 paper 256 16e7c675af1e1e56 → Data grid 120 40 paper 256 d6cf6485e6624d0f
  Data grid 120 40 paper mono 09bc1c05cd28e187 → Data grid 120 40 paper mono 7a2e61e7f6dd9364
  Data grid 120 40 paper truecolor f2cfc94a2375ff99 → Data grid 120 40 paper truecolor e0cceb143a7020dc
  Data grid 160 50 junie 16 bf9dc961b8941e3f → Data grid 160 50 junie 16 4fef292f60d67d78
  Data grid 160 50 junie 256 1ca7b7e186865b65 → Data grid 160 50 junie 256 9985f8f164e4cc30
  Data grid 160 50 junie mono 63dbf452609390db → Data grid 160 50 junie mono 59ff3cd8db2c4e00
  Data grid 160 50 junie truecolor 7905116f77960a91 → Data grid 160 50 junie truecolor 71760b1cab5d4cc2
  Data grid 160 50 paper 16 8682785125adc363 → Data grid 160 50 paper 16 0723611b493d9616
  Data grid 160 50 paper 256 b17cfcc5f528d1be → Data grid 160 50 paper 256 dcda04a28cc5d467
  Data grid 160 50 paper mono 9ddde3595e42ca67 → Data grid 160 50 paper mono 6b09d44d3b5e0ea4
  Data grid 160 50 paper truecolor 0109e59e243084bd → Data grid 160 50 paper truecolor 5cf8d8d416ec2200
  Data grid 80 24 junie 16 5bbafcce6df367cd → Data grid 80 24 junie 16 672d85934ec1bf20
  Data grid 80 24 junie 256 2cf4106a7a122c70 → Data grid 80 24 junie 256 c854ab232e0800b9
  Data grid 80 24 junie mono 2104f1bee57c0a53 → Data grid 80 24 junie mono 2eb45a78a230d024
  Data grid 80 24 junie truecolor 925c3b9732c11c6d → Data grid 80 24 junie truecolor 3cca73dc989b1b2e
  Data grid 80 24 paper 16 f6c7898b9c2fdd19 → Data grid 80 24 paper 16 8abc95eb7e26d4a6
  Data grid 80 24 paper 256 32f0120dd1967618 → Data grid 80 24 paper 256 a9b8877cacf4feb5
  Data grid 80 24 paper mono 34dd9f59bf53ad7d → Data grid 80 24 paper mono 45a50905805f6fb6
  Data grid 80 24 paper truecolor e546efc9235e4d38 → Data grid 80 24 paper truecolor c161ac56c4546925
  Dialogs 100 30 paper 16 e89751811a23d73f → Dialogs 100 30 paper 16 128403b53fc64a2f
  Dialogs 120 40 paper 16 f23e7d9c13c4c0da → Dialogs 120 40 paper 16 5b156793c36370aa
  Dialogs 160 50 paper 16 9cffaeccb4bad8a2 → Dialogs 160 50 paper 16 83bc30ade4e2f672
  Dialogs 80 24 paper 16 18a0c064898558fc → Dialogs 80 24 paper 16 fabbcc82a14d342c
  Editable tables 100 30 paper 16 11d4331f22b4dd35 → Editable tables 100 30 paper 16 83ccd9efa11010f9
  Editable tables 120 40 paper 16 e2f59f75614792a7 → Editable tables 120 40 paper 16 a565f6f2468966d3
  Editable tables 160 50 paper 16 567f3dbfce920c5e → Editable tables 160 50 paper 16 9240b9c63217159a
  Editable tables 80 24 paper 16 5c993d9d02a1f7d7 → Editable tables 80 24 paper 16 a1d6196b14827017
  Editor 100 30 paper 16 47510e498b919135 → Editor 100 30 paper 16 b4b6d77a739c95b5
  Editor 120 40 paper 16 2093ad11ae705a1a → Editor 120 40 paper 16 0ac5fe44b20b6992
  Editor 160 50 paper 16 0df51f7ef6cd05e2 → Editor 160 50 paper 16 34d1889625615c7a
  Editor 80 24 paper 16 0be8238a36bbb9d9 → Editor 80 24 paper 16 7af410a3832f9eed
  Forms 100 30 paper 16 85480ab95e2cc5c6 → Forms 100 30 paper 16 23b79924e3c87282
  Forms 120 40 paper 16 8fc0fe65a04d0575 → Forms 120 40 paper 16 4598392a22f1900d
  Forms 160 50 paper 16 d2f72dc024e8994b → Forms 160 50 paper 16 92047362ec8a49cf
  Forms 80 24 paper 16 dd62419daa857585 → Forms 80 24 paper 16 bbc8af7aad4ae931
  Inputs 100 30 paper 16 38e9f496420f44a6 → Inputs 100 30 paper 16 932a354c9ec61bf6
  Inputs 120 40 paper 16 cc36ec6140116d07 → Inputs 120 40 paper 16 543c48b3b277a1f7
  Inputs 160 50 paper 16 db27ce5eaba0db57 → Inputs 160 50 paper 16 4debb3cfee5f4147
  Inputs 80 24 paper 16 8007b71c7e161854 → Inputs 80 24 paper 16 2b84370996d88fc4
  Lists 100 30 paper 16 3531f6c6b52615a0 → Lists 100 30 paper 16 24216c59acac2590
  Lists 120 40 paper 16 e8d3761dbe48471e → Lists 120 40 paper 16 f150abcd819df40e
  Lists 160 50 paper 16 b7128328d0af1296 → Lists 160 50 paper 16 263ba361ff0c9646
  Lists 80 24 paper 16 ee385ce725a3935d → Lists 80 24 paper 16 d283cdbdd6840319
  Overview 100 30 paper 16 a978ad29d923fee7 → Overview 100 30 paper 16 778034a14d0376e7
  Overview 120 40 paper 16 2417bdb9924cba28 → Overview 120 40 paper 16 1026eba325858ffc
  Overview 160 50 paper 16 9971bb8d0bac611c → Overview 160 50 paper 16 c145cc173575d73c
  Overview 80 24 paper 16 3abb74290d9b4fdb → Overview 80 24 paper 16 b53cd06e40bb3ee3
  Panels 100 30 paper 16 cc6bd43739327287 → Panels 100 30 paper 16 a5bb826cffba04d7
  Panels 120 40 paper 16 87188d44fc85b652 → Panels 120 40 paper 16 b98658ad7d4d8782
  Panels 160 50 paper 16 b44f1720123000f8 → Panels 160 50 paper 16 f23ce255f1671428
  Panels 80 24 paper 16 fbc641b8ae46ea7a → Panels 80 24 paper 16 49079bc8a9e6f18a
  Pickers 100 30 paper 16 168754257d557b71 → Pickers 100 30 paper 16 dcddf7ece4e3c6e5
  Pickers 120 40 paper 16 4c7e6581d406bd6e → Pickers 120 40 paper 16 6f582461c4898442
  Pickers 160 50 paper 16 fb0a886c48e1d2cd → Pickers 160 50 paper 16 c7a240a931e14f05
  Pickers 80 24 paper 16 8d6dd5e48e445bce → Pickers 80 24 paper 16 645bed658ef8b4ba
  Progress 100 30 paper 16 93de8b4e18bae9a0 → Progress 100 30 paper 16 e14866d3cfe61ff0
  Progress 120 40 paper 16 6ad45ae774945cb8 → Progress 120 40 paper 16 87fc8e8585b12bf8
  Progress 160 50 paper 16 388d40e1e107ae5c → Progress 160 50 paper 16 4288e343372058bc
  Progress 80 24 paper 16 35470b6298cfa87d → Progress 80 24 paper 16 b75eab2e7aa32739
  Scrolling 100 30 paper 16 a1cd37a2668f5742 → Scrolling 100 30 paper 16 772406b3cf8ba652
  Scrolling 120 40 paper 16 b4f6be9cde8c27fc → Scrolling 120 40 paper 16 803759e23bcd20b4
  Scrolling 160 50 paper 16 94b35a043aaf363d → Scrolling 160 50 paper 16 5069f5b16b54794d
  Scrolling 80 24 paper 16 446838cff6525808 → Scrolling 80 24 paper 16 6b3a9d64cd3f6ebc
  Settings 100 30 paper 16 4bb2221fd4521547 → Settings 100 30 paper 16 1929d7c4db2655c7
  Settings 120 40 paper 16 e92e15d5c83d3ffc → Settings 120 40 paper 16 8d1668810e8d321c
  Settings 160 50 paper 16 d584feea44b2cba4 → Settings 160 50 paper 16 bf360a9410212844
  Settings 80 24 paper 16 333afaab7d70eb68 → Settings 80 24 paper 16 232485ed5e28dfa4
  Sidebars 100 30 paper 16 77318c5412d99330 → Sidebars 100 30 paper 16 e91d08fd58611120
  Sidebars 120 40 paper 16 22873fd53c970f30 → Sidebars 120 40 paper 16 190d35dd9900a5c0
  Sidebars 160 50 paper 16 e9fa817b546f7e0b → Sidebars 160 50 paper 16 dc863ea86a8da69b
  Sidebars 80 24 paper 16 2fd6e96cf453e3b5 → Sidebars 80 24 paper 16 179360e0e2808085
  Tables 100 30 junie 16 e965123995c05745 → Tables 100 30 junie 16 33902d75922de664
  Tables 100 30 junie 256 e6d6826bf03a9454 → Tables 100 30 junie 256 3e9af10c2a5e5e0b
  Tables 100 30 junie mono 3b0a3c094c884934 → Tables 100 30 junie mono 26a19bf34bec53a7
  Tables 100 30 junie truecolor 6bc75591a99f8299 → Tables 100 30 junie truecolor d02470a233b6684a
  Tables 100 30 paper 16 576de12d927c81bb → Tables 100 30 paper 16 55468d7cee898120
  Tables 100 30 paper 256 23fe9a6c057f2203 → Tables 100 30 paper 256 9bfb9897144c024a
  Tables 100 30 paper mono 25d0eb8b47e3014e → Tables 100 30 paper mono 27c73976616c2ca9
  Tables 100 30 paper truecolor 89d13a52813ae4ec → Tables 100 30 paper truecolor 64522c741e260e1f
  Tables 120 40 junie 16 f69625d6eda10b69 → Tables 120 40 junie 16 998267e3cadd15f8
  Tables 120 40 junie 256 6355036929f6a9da → Tables 120 40 junie 256 a1597be77dc34f85
  Tables 120 40 junie mono 06a237372ad099cf → Tables 120 40 junie mono 7e6c63ea6874e29c
  Tables 120 40 junie truecolor 506e870e013bac68 → Tables 120 40 junie truecolor 787ba406b60d7ed1
  Tables 120 40 paper 16 88081a0c66b9fec5 → Tables 120 40 paper 16 6c779f5143b99d74
  Tables 120 40 paper 256 1bd765d18abba950 → Tables 120 40 paper 256 db218eac7c2196ad
  Tables 120 40 paper mono 57ad91272844df65 → Tables 120 40 paper mono 4b54535694b24562
  Tables 120 40 paper truecolor f758ecdabfd99388 → Tables 120 40 paper truecolor 0f3680938fe3cee9
  Tables 160 50 junie 16 5e8a3df8d0925550 → Tables 160 50 junie 16 79eac3e1bb808cb9
  Tables 160 50 junie 256 ac63846e2546c297 → Tables 160 50 junie 256 7d4c08a9f045ccd8
  Tables 160 50 junie mono e451345cfe00e87c → Tables 160 50 junie mono 5219b568ef9972f7
  Tables 160 50 junie truecolor 69c0287a01c857c3 → Tables 160 50 junie truecolor b6a107ce2e3f4502
  Tables 160 50 paper 16 1aab54cb3ca99e1c → Tables 160 50 paper 16 ed61b2e9e08ef219
  Tables 160 50 paper 256 b29926c1951a92f3 → Tables 160 50 paper 256 542b2c462ea100e6
  Tables 160 50 paper mono db42cdf699d08ce2 → Tables 160 50 paper mono 1e4241f13f0acc1d
  Tables 160 50 paper truecolor 3b092e224a697c2d → Tables 160 50 paper truecolor 36797e5cd89530c8
  Tables 80 24 junie 16 a9d1eccad95120fd → Tables 80 24 junie 16 a31d5256122a5e94
  Tables 80 24 junie 256 2901c8b5cdc895db → Tables 80 24 junie 256 1575db1ef0431e38
  Tables 80 24 junie mono a88a358a1ff92324 → Tables 80 24 junie mono 8f5b063a1c5da6a3
  Tables 80 24 junie truecolor b06721de063eda80 → Tables 80 24 junie truecolor 5231b5d33ae9856f
  Tables 80 24 paper 16 a07ffcecfacaa031 → Tables 80 24 paper 16 83bc3073229fc2f6
  Tables 80 24 paper 256 714649b9c70fe153 → Tables 80 24 paper 256 23f6d8c17a4884f2
  Tables 80 24 paper mono d87f9f51e703c318 → Tables 80 24 paper mono ef0d1fa7db2f08bf
  Tables 80 24 paper truecolor d7e085b5ed31447f → Tables 80 24 paper truecolor 718b302f31ff7518
  Task runner 100 30 paper 16 7146985c63bd828d → Task runner 100 30 paper 16 e93219e16f8791ed
  Task runner 120 40 paper 16 1e9511ee1ebfd683 → Task runner 120 40 paper 16 0841bf75618ae303
  Task runner 160 50 paper 16 3619ac69a64ebddb → Task runner 160 50 paper 16 2a1875dfb6abf55b
  Task runner 80 24 paper 16 89d45c30580e70d5 → Task runner 80 24 paper 16 60f4c6c229a2a42d
  Terminal 100 30 paper 16 e42cafd0c0b06a7c → Terminal 100 30 paper 16 d4f7ba97711b99b0
  Terminal 120 40 paper 16 3edf58443f103232 → Terminal 120 40 paper 16 7951fccdb947dc06
  Terminal 160 50 paper 16 41b72f7208f2e3ca → Terminal 160 50 paper 16 655db358adebb1fe
  Terminal 80 24 paper 16 ffdfa274c744e237 → Terminal 80 24 paper 16 b40ac4371613b6cf
  Text areas 100 30 paper 16 70c1332c2313c3da → Text areas 100 30 paper 16 473a15728b296f6e
  Text areas 120 40 paper 16 50a2a3cfde198232 → Text areas 120 40 paper 16 dad5ae35430aa5fe
  Text areas 160 50 paper 16 0fc760ba987d74aa → Text areas 160 50 paper 16 6ca46a05385e2976
  Text areas 80 24 paper 16 5ba16bbee0111897 → Text areas 80 24 paper 16 bf0e5286ae1ae863
  Trees 100 30 paper 16 3f8150c8cd97f60d → Trees 100 30 paper 16 e0bd6c6e1ce24e89
  Trees 120 40 paper 16 fec9af252fdb0ef4 → Trees 120 40 paper 16 afc9ec8f0f02f1f8
  Trees 160 50 paper 16 bc787b4071a8ef4c → Trees 160 50 paper 16 9b5a29124c278550
  Trees 80 24 paper 16 3f798680854e9339 → Trees 80 24 paper 16 a2ab802e3e241399
- added:     none
- class:     fix
- reason:    §20.10 item 34; the Paper/ANSI16 contrast repair and shared Grid/Table geometry correction changed only the exact cells listed above.
```

## Review status — current Grid and TablePro digest movements (2026-09-05)

**Classification only. No baseline was modified, regenerated, copied, or blessed.** The
read-only current-main render audit records these exact old→new pairs so a later review can
distinguish the source correction from approval of a retained digest.

```
- surface:   junie-tui Grid / 120x40 / Junie / truecolor
- captures:  current no-BLESS frame-text output; retained `crates/tui/tests/baselines/components.txt`
             remains unchanged
- tests:     `render::components::grid::{default,disabled,editing,focused,hovered,selected}`
- moved:     6 keys:
  render::components::grid::default 120 40 junie truecolor 9817dac1ed7c9346 → da9a5d1263990b5e
  render::components::grid::disabled 120 40 junie truecolor 9817dac1ed7c9346 → da9a5d1263990b5e
  render::components::grid::editing 120 40 junie truecolor 9817dac1ed7c9346 → da9a5d1263990b5e
  render::components::grid::focused 120 40 junie truecolor 58ca02d397496888 → a1a2abce7d5923e0
  render::components::grid::hovered 120 40 junie truecolor 1cccb015edb67dd0 → 9c96eae598803c28
  render::components::grid::selected 120 40 junie truecolor 2f975888c9b83ac7 → d08b7201048fb0cf
- added:     none
- class:     fix
- reason:    §20.10 item 7. The current Grid geometry/runtime correction changes the rendered
             overflow/state cells; these hashes are recorded as an unblessed source-fix movement,
             not as permission to update `components.txt`.
```

```
- surface:   TablePro `connections` / 120x40 / Junie / truecolor
- captures:  current app-owned digest evidence; the frozen pre-refactor root digest remains
             untouched at `tests/baselines/tablepro.txt`, and the app digest remains untouched at
             `apps/tablepro/tests/baselines/tablepro.txt`
- tests:     pre-refactor `src/bin/tablepro/visual_tests.rs::tablepro_visual_baseline`;
             `apps/tablepro/tests/visual.rs::tablepro_visual_baseline`
- moved:     1 key (cross-owner migration mapping):
  `tests/baselines/tablepro.txt` `120x40 connections` 9c7fb68135d17b20
    → `apps/tablepro/tests/baselines/tablepro.txt` `connections 120 40 junie truecolor` c1fdb4fc02dacd7f
- added:     none in this audit entry
- class:     intended
- reason:    §20.10 items 14 and 32. The application package owns the first-generation TablePro
             matrix after the package split; this cross-owner pair is recorded for auditability,
             not asserted as byte-equivalent retained evidence and not approved for blessing.
```

```
- surface:   current TablePro `connections` / 120x40 / Junie / truecolor
- captures:  current no-BLESS app audit; the application-owned baseline remains unchanged and no
             capture is accepted as approval
- tests:     `apps/tablepro/tests/visual.rs::tablepro_visual_baseline`
- moved:     1 key:
  `apps/tablepro/tests/baselines/tablepro.txt` `connections 120 40 junie truecolor`
    c1fdb4fc02dacd7f → c61db28c2277776b
- added:     none
- class:     regression
- reason:    §20.10 items 14 and 32. The current source does not render the `connections` screen
             through the audited route, so this movement is a source regression, not an intended
             visual migration. Repair routing, rerun the no-BLESS audit, and obtain fresh review;
             this entry never authorizes a baseline update or blessing.
```

The eight pairs above are unblessed audit evidence. The TablePro regression must be fixed before
any retained baseline change; a future independent frame review and separate serial authorization
are still required before blessing.

 ## Review status — Slice 4 component matrix, independent visual review (2026-09-05)

### Fresh independent audit — candidate `fb017a7` (2026-09-05)

**Result: FAIL. Candidate rejected as a bundle. No baseline was edited, regenerated, copied, or
blessed.** The findings below are review evidence only; they do not authorize any retained digest
movement.

#### Exact regressions

| Surface | Finding | Classification / contract |
|---|---|---|
| Steps | The candidate adds a `Bullet` glyph to `Skipped`. | **Regression** — §20.10 item 25; `Skipped` lifecycle/glyph semantics changed. |
| TooSmall | The `empty` fixture renders non-empty content. | **Regression** — §20.10 item 24; the first-generation empty fixture contract is violated. |
| PickerChain | The selected fixture drops `Vault` from the rendered chain. | **Regression** — §20.10 item 27; stage ordering/content is not preserved. |
| Wizard | The fixture fabricates a `Details` state/step. | **Regression** — §20.10 item 27; the fixture is not source-backed. |

The candidate was reviewed as one bundle (`fb017a7`, including its staged visual-contract changes),
so no individual hunk is accepted by this audit. Other plausible disabled-contrast, semantic-
selection, glyph, and state-affordance changes are **deferred** pending isolated source changes,
fresh frame evidence, and a new independent review.

#### Current capture and blessing status

- The latest **TablePro and Jackin captures are stale** against the current source lineage and
  must be rerun through the capture matrix before they support review or classification.
- **Showcase ANSI16 contrast is suspect**, not cleared; fresh Showcase captures and independent
  review are required.
- **No baseline was modified and no bless was run.** Visual blessing remains unauthorized until
  the four regressions are resolved, deferred changes are adjudicated, stale app captures are
  refreshed, and a fresh independent review grants separate authorization.

### Latest independent app audit — source `06bf0e6`/`4534a11` (2026-09-05)

**Mixed result against `origin/main` `1852338`. No baseline was edited, regenerated, copied, or blessed.** This entry supersedes
the prior audit's current-result claims while retaining that entry as historical evidence.

| Surface | Result | Exact evidence |
|---|---|---|
| Showcase | **PASS** | Inspected Showcase frames, including the available ANSI16 evidence. |
| TablePro `connections` | **FAIL** | Digest `c1fdb4fc → fb80a107`; regression under §20.10 items 14 and 32. |
| Jackin visual fixture | **FAIL** | `accounts-1password-step-1` fails its `chainargos` expectation. |
| Jackin rain | **3 PASS** | Intro timeline, outro caption/skip, and deterministic starfield checks. |

Capture provenance is stale: `shots/capture-provenance.json` records revision
`a358272665d49d74e4fc17a262d6061621e861d2`, not the audited `06bf0e6`/`4534a11` lineage. The
TablePro and Jackin evidence therefore requires a fresh provenance-backed capture run. ANSI16
evidence remains provisional until that run is independently reconciled.

**Visual blessing remains unauthorized.** The TablePro movement and Jackin fixture failure must be
resolved, captures rerun with current provenance, and the resulting frames independently reviewed
before any retained baseline change or bless.
## Item 35 — Empty regions inherit the owning style; hover no longer lifts foreign rows

Two corrections to shared collection/interaction painting, integrated from the
reviewed branch line and re-based over the merged main line. (a) The empty
branches of `List`, `Grid`, `FilterList` (and `Picker`, which delegates to it)
route through `EmptyState::draw_inherited`, so the owner's EMPTY-part style
blanket-fills the empty rect instead of only styling the title run — a blank
cell inside an empty component no longer ignores the component's authored
style. (b) NavList adds `StateFlags::HOVERED` to the difference masks of its
container and header fills, so the hover tint lifts only the hovered row
(never the section gaps, the header row or the rows below), and Junie's LIST
row-hover plane moves from `RaisedSurface` to `HoverSurface`, one rung up the
elevation ladder. Frame text is byte-identical before and after for every
moved key (no-BLESS dumps compared); the movement is confined to cell styles,
and only truecolor cells moved for these key names that the merged ledger did
not already account.

```
- surface:   junie-tui collection components @ 120x40 and 40x10 / junie + paper / truecolor
- captures:  no-BLESS frame-text dumps for every moved key (text-identical old vs new);
             refreshed capture matrix and independent review follow the integration freeze
- tests:     render::components::{filter_list,grid,list,picker}::empty,
             render::components::nav_list::hovered,
             apps/showcase sidebar contract `row_hover_does_not_lift_headers_gaps_or_other_rows`
- moved:     20 keys (rewritten to the merged diff):
  render::components::filter_list::empty 120 40 junie truecolor 1d170068e8dc22c9 -> 964725de563d6f01
  render::components::filter_list::empty 120 40 paper truecolor 67f6d78d0e4d6297 -> 5657ae8b892e03c1
  render::components::filter_list::empty 40 10 junie truecolor 1efd980f03fcdfc9 -> 81fe767b5e763581
  render::components::filter_list::empty 40 10 paper truecolor 299686fa5483e7e7 -> 9fe579eb452ad7c1
  render::components::grid::empty 120 40 junie truecolor 312b25df0c0619c1 -> ef17e66997c72ad1
  render::components::grid::empty 120 40 paper truecolor 2147e9f589409929 -> f7028c8dabc46061
  render::components::grid::empty 40 10 junie truecolor c3633f90fa6f6f01 -> 676a00ea0ce59071
  render::components::grid::empty 40 10 paper truecolor 03cc64396325c9f9 -> 8012463ca99de109
  render::components::list::empty 120 40 junie truecolor 121b6cb8fded49a2 -> 031f18534b558a72
  render::components::list::empty 120 40 paper truecolor ddbd052dff13465a -> 6aa320d4b4a79de2
  render::components::list::empty 40 10 junie truecolor ffa7ea144c4cbd62 -> 4bc1ebbfb6005ef2
  render::components::list::empty 40 10 paper truecolor 065d1b5055061c8a -> be7ba0c4623a2cca
  render::components::picker::empty 120 40 junie truecolor 24a1c023e47afb91 -> edd1c31de913f869
  render::components::picker::empty 120 40 paper truecolor 13a1c24275705d9b -> 4ff1f03b4b7d8b83
  render::components::picker::empty 40 10 junie truecolor e8d69c83a18c8671 -> 714bb240ee33a55d
  render::components::picker::empty 40 10 paper truecolor b9694c43428d2f4b -> b8b9a70e4a14e6bb
  render::components::nav_list::hovered 120 40 junie truecolor 8d002c6b06d998a1 -> 651b9384d988b491
  render::components::nav_list::hovered 120 40 paper truecolor 2d3671c68d9a4fd7 -> 4b970bd6535516e7
  render::components::nav_list::hovered 40 10 junie truecolor 9be3a9429545f621 -> 9e351f7ef2d50ca1
  render::components::nav_list::hovered 40 10 paper truecolor 98693e337bb6cf67 -> d1c48195b012bcf7
- added:     none
- class:     fix
- reason:    §20.10 item 35. An empty component painting unstyled blank cells and a hover
             tint that lifted rows nobody hovered were demonstrated defects of the old
             shared path; the corrections remove the enabling condition (style and flag
             propagation now flow through the owner's own part style / difference mask).
```

## Item 36 — Showcase pages: coverage expansion, absorbed historical flows, Holla shell parity

The merged Showcase is the union of two reviewed lines: the branch's
single-constructor coverage expansion (every public component reachable, §13
one-constructor discipline, Wizard/Form flow, Panel wrappers, Meter/Spinner/
List/Split additions) and the main line's historical page flows (the Chrome
status strip, the richer Pickers fixture, the Dialogs gutter), re-expressed
through the merged semantic-paint API. Two Holla production-parity
corrections landed after the merge made the divergence measurable against the
reference fixture: the Editor page heading is always its full title
(`Code editor` — the narrow-layout shortening contradicted the Holla
reference at every size) and the shell header paints the capability cluster
only when two cells clear it from the breadcrumb (Holla omits it at 89 and 90
columns rather than crowd the route title). 208 page cells move for that union: the 168 truecolor cells and the 40
mono/ANSI16/ANSI256 Buttons and Chips & selects cells, whose absorbed
historical content changed every colour level of those pages.

```
- surface:   showcase digest @ 22 pages / all four sizes / junie + paper / truecolor
- captures:  apps/showcase/tests/fixtures/holla-page-headings.tsv (352 rows) and
             holla-shell-headers.tsv (355 rows) from the Holla 794b095 production binary,
             both green; refreshed capture matrix and independent review follow the freeze
- tests:     apps/showcase/tests/visual.rs::showcase_visual_baseline,
             apps/showcase/tests/page_headings.rs (both reference tests),
             apps/showcase/tests/app_tests.rs (34 passed)
- moved:     672 keys (rewritten to the merged diff):
  Buttons 100 30 junie 16 8744db9d7f731eb5 -> e8ca399647b35c42
  Buttons 100 30 junie 256 911355f405d13d06 -> 9c13c1769086100c
  Buttons 100 30 junie mono c056b37d3654a35a -> 006b19eaa35c98c5
  Buttons 100 30 junie truecolor db8842891214291b -> fe0e93bf5afe1041
  Buttons 100 30 paper 16 3f31309902fbc2cc -> 93dc402e53f19cee
  Buttons 100 30 paper 256 7df3cd7aea4e099a -> 2793a4c1b72d53cf
  Buttons 100 30 paper mono b6515fe380afd6ec -> 9e75bb8156d48bc5
  Buttons 100 30 paper truecolor e7f6594d1b73662e -> 72771236c223b09e
  Buttons 120 40 junie 16 1b5525dd17519efd -> 2c5c0915fc97c841
  Buttons 120 40 junie 256 1035025825f50485 -> 48d7abdecf2e51b0
  Buttons 120 40 junie mono 31d681d41c4758c4 -> cac2fba0743e801e
  Buttons 120 40 junie truecolor 7c8e856b02877428 -> 0e0230295d61ad10
  Buttons 120 40 paper 16 1ed30c75fec13f90 -> 05e39c1d6aef13a6
  Buttons 120 40 paper 256 ba88705efc7c3320 -> 50fc35c84b471915
  Buttons 120 40 paper mono 74fc2cb6da7a2754 -> db46c90f7dca133e
  Buttons 120 40 paper truecolor cf97fef12a7ae2e6 -> a8b787b1ed05e5de
  Buttons 160 50 junie 16 aa3de33c642db44d -> 2bd3da79d3e27574
  Buttons 160 50 junie 256 6f269773a40c9215 -> 3e0b9f193ee2e441
  Buttons 160 50 junie mono 4aca5c803da2013c -> 9efe6182e35e3f39
  Buttons 160 50 junie truecolor 33640adfb2384e7c -> acbe4ce03a8e4cfd
  Buttons 160 50 paper 16 139920a7b61b8f78 -> 88097567ec151215
  Buttons 160 50 paper 256 767f89b3d8424998 -> 42e57fbfca821ab2
  Buttons 160 50 paper mono 1c5f6068f7beaae4 -> 7001a2e9d5fc7225
  Buttons 160 50 paper truecolor 51fffa465307d192 -> ac1f9849aaf6b281
  Buttons 80 24 junie 16 25e43db7757c9cf0 -> 1602713e637b6852
  Buttons 80 24 junie 256 1c2034c590e88ff3 -> 71c94e4fe19529ee
  Buttons 80 24 junie mono 2edd6e1f364eee7f -> b0c072abb34b52c8
  Buttons 80 24 junie truecolor bc9668017776d9bb -> 63944c74deb24c62
  Buttons 80 24 paper 16 f16961098f9ca069 -> 30b23a8b11963946
  Buttons 80 24 paper 256 8d658efffb8da885 -> 1c33c96d08aa8b79
  Buttons 80 24 paper mono 05224573e7a84c35 -> c25b7b58734a0e2a
  Buttons 80 24 paper truecolor 17c5541afb122d8d -> 5bc7a70b3410bfd2
  Chips & selects 100 30 junie 16 f155f233dd29fc80 -> 1cc427547f2ea457
  Chips & selects 100 30 junie 256 e85ed45c32000c3f -> 3fa60db7f71af8c1
  Chips & selects 100 30 junie mono 603f573c73555218 -> f6f26d7f6499eaa6
  Chips & selects 100 30 junie truecolor 90cea677dd372564 -> 39bae62fae9b3d48
  Chips & selects 100 30 paper 16 2d5b5de5ceb4b7c4 -> 54dd0fddb0d4c68f
  Chips & selects 100 30 paper 256 281fd3032b70772d -> d210e4f4f1956582
  Chips & selects 100 30 paper mono af5e1c168215198e -> 6a00ac3fdfac5fec
  Chips & selects 100 30 paper truecolor 8720c887325c51be -> 9ac6ba2ad200e886
  Chips & selects 120 40 junie 16 2913c2531b7e430f -> 03e7233934ff7fcd
  Chips & selects 120 40 junie 256 9871994f32c5d377 -> 7f5766099c299dac
  Chips & selects 120 40 junie mono d5e96b32ee80c069 -> c7b368745a1d58fc
  Chips & selects 120 40 junie truecolor d9906e251df7bbec -> 78bf7d4f007195d3
  Chips & selects 120 40 paper 16 60de18f009c46a4b -> e1eafcbd219fce97
  Chips & selects 120 40 paper 256 bbd2971a61552c78 -> a1c92ef8e16a651e
  Chips & selects 120 40 paper mono bd8e6d19dbd9c687 -> d966cdac1140a13e
  Chips & selects 120 40 paper truecolor bfa1f4010cfde7fc -> a970a0f58c5fa88b
  Chips & selects 160 50 junie 16 e286c65adec06f2f -> 7d3be23db66ef172
  Chips & selects 160 50 junie 256 fdaf98e43e4bed0f -> a34639bdd5df40f7
  Chips & selects 160 50 junie mono 3a25036555816c41 -> acd9bb55a100e03d
  Chips & selects 160 50 junie truecolor 406115e625f6fd40 -> a29ddaf8f118e1da
  Chips & selects 160 50 paper 16 75a9d36d5ca4f913 -> 7a916cd4365c36b0
  Chips & selects 160 50 paper 256 716bc96bbce995e0 -> adf6cf82965ff69d
  Chips & selects 160 50 paper mono ebdec188774395c7 -> f2c21b4865c44f57
  Chips & selects 160 50 paper truecolor c1a64f12bc95c340 -> d08fae39488ad1a0
  Chips & selects 80 24 junie 16 234244a5a8e77816 -> b49f61f813118dad
  Chips & selects 80 24 junie 256 b8483e95e8f2d40d -> 4c44895412aff2ec
  Chips & selects 80 24 junie mono de75faafe9597d34 -> f12edc34c54d82df
  Chips & selects 80 24 junie truecolor 6b77fd69490eb878 -> 89d1ba62f79d9e80
  Chips & selects 80 24 paper 16 58e73fc0c981688b -> 855f17074aa8d489
  Chips & selects 80 24 paper 256 ba7a6db260a4bd28 -> be1e74d56351ada7
  Chips & selects 80 24 paper mono 5c5f999f76394b6c -> d051ecb474251045
  Chips & selects 80 24 paper truecolor 25818be0f9f69332 -> a9610e66eb23a19c
  Chrome 100 30 junie 16 037d44997822075d -> 65046f5af326600f
  Chrome 100 30 junie 256 ea6d243cbacca9a7 -> 9d151cca4fdc6fbe
  Chrome 100 30 junie mono 5e90338b38c73387 -> f8f5a397db1b92c6
  Chrome 100 30 junie truecolor 3d11006843e1a430 -> 565ae37bbefe6bb2
  Chrome 100 30 paper 16 30a4effa148b24d0 -> a7a8ebbc48ec4f0a
  Chrome 100 30 paper 256 25929ebddbc53258 -> 9c9efa1a4a6072c1
  Chrome 100 30 paper mono 34087c0e36179867 -> dda65a456b8db200
  Chrome 100 30 paper truecolor ef2e2da72d1823b1 -> 05d9acdb8b172786
  Chrome 120 40 junie 16 0cc66952fbc32cb3 -> 4abc09dfe6259629
  Chrome 120 40 junie 256 64c00c943d0f0774 -> 8acf8962bb7ad1ac
  Chrome 120 40 junie mono 7c8371fd1bdba26f -> 7d52b7e5644516de
  Chrome 120 40 junie truecolor 90b60563553ffcef -> 6d612f4203525b71
  Chrome 120 40 paper 16 4a55eaaab393617e -> 2574c8e90858a726
  Chrome 120 40 paper 256 05f90818073531e8 -> b816be99390e9f6f
  Chrome 120 40 paper mono b861c193cb3faed7 -> 705bd8f0d1542adc
  Chrome 120 40 paper truecolor 1f18368d877e1092 -> 175c4054765de10a
  Chrome 160 50 junie 16 5838f4ea4811738b -> 635233b221bc92dc
  Chrome 160 50 junie 256 4fc24f2a18be565c -> 5d9c6f756c26869f
  Chrome 160 50 junie mono c75c610fb6524367 -> 53d581ecdabe298d
  Chrome 160 50 junie truecolor 9bdc15fef162647b -> 077103fedb58eb66
  Chrome 160 50 paper 16 d05870fe5101df8e -> c88806cebbb96ee1
  Chrome 160 50 paper 256 2c27f5d71cea9210 -> 330c18c0f4447f82
  Chrome 160 50 paper mono b592da4190032157 -> f026e1aac5666b6b
  Chrome 160 50 paper truecolor dc5247f8aeedb29e -> d192c9a85642ab17
  Chrome 80 24 junie 16 e76a176498412f34 -> fe667144d6e1bd36
  Chrome 80 24 junie 256 371e9d271aa4cfc8 -> aa36f36847638ce7
  Chrome 80 24 junie mono 9a052920208a1a8c -> 1d55783410ec76a6
  Chrome 80 24 junie truecolor b44f90fc2c49c1fb -> 01f8fde914c9aa1d
  Chrome 80 24 paper 16 79ea53f3c60abf6c -> 06c2a8083c90aea7
  Chrome 80 24 paper 256 86025840ac855e48 -> 6beae275d49cb302
  Chrome 80 24 paper mono ce43b418f4c56ab2 -> 1651e4acb291f274
  Chrome 80 24 paper truecolor 7c22142446ff5076 -> 2e534d53a8831a62
  Data grid 100 30 junie 16 d09e81e22aaddfd5 -> 1e43c2a98a8faeae
  Data grid 100 30 junie 256 40bf28e3a6147096 -> c1356a31bff3bd65
  Data grid 100 30 junie mono 9a450827ade2e249 -> 00102f8daf72b931
  Data grid 100 30 junie truecolor 62f0178a5371cb62 -> 66c162ec9cd60c46
  Data grid 100 30 paper 16 28796d034234542f -> 3148dbe1c7ca08a3
  Data grid 100 30 paper 256 3c2185e9aedaea7a -> 4cba8af356cc37ce
  Data grid 100 30 paper mono eee115b41b760955 -> a1815830daeea84b
  Data grid 100 30 paper truecolor a5e47ca5e066cb98 -> 0b3da0a627d382af
  Data grid 120 40 junie 16 13e26f3af2ee1268 -> 3e5fe92de29b4fc5
  Data grid 120 40 junie 256 ae996e9a7f270d40 -> 0af7ddf1c35afcc9
  Data grid 120 40 junie mono 8f0f39215ca7b908 -> d30ee896d1184954
  Data grid 120 40 junie truecolor 95e4d2bddf60df36 -> e987fb604e53e2a0
  Data grid 120 40 paper 16 7ed0138d4e1fe4a6 -> 963561b54091581c
  Data grid 120 40 paper 256 d6cf6485e6624d0f -> 4b42b6875f6a21d7
  Data grid 120 40 paper mono 7a2e61e7f6dd9364 -> 5ba2b694c1b95f54
  Data grid 120 40 paper truecolor e0cceb143a7020dc -> 448ff101b977b001
  Data grid 160 50 junie 16 4fef292f60d67d78 -> 6f77d2914ba532fe
  Data grid 160 50 junie 256 9985f8f164e4cc30 -> a58a947d596f5b6a
  Data grid 160 50 junie mono 59ff3cd8db2c4e00 -> 82eb2ab2382c03af
  Data grid 160 50 junie truecolor 71760b1cab5d4cc2 -> 7da2e88fd4c885f9
  Data grid 160 50 paper 16 0723611b493d9616 -> e3869060301ad60b
  Data grid 160 50 paper 256 dcda04a28cc5d467 -> f13b5896eeddbba0
  Data grid 160 50 paper mono 6b09d44d3b5e0ea4 -> 815f5b112c06109f
  Data grid 160 50 paper truecolor 5cf8d8d416ec2200 -> cf2459a2bab86988
  Data grid 80 24 junie 16 672d85934ec1bf20 -> 38b108745be1906f
  Data grid 80 24 junie 256 c854ab232e0800b9 -> d44f9f9d4c0ffcf9
  Data grid 80 24 junie mono 2eb45a78a230d024 -> 95569b7e2545f42d
  Data grid 80 24 junie truecolor 3cca73dc989b1b2e -> 6b902583853e063e
  Data grid 80 24 paper 16 8abc95eb7e26d4a6 -> aee1def9846a84fe
  Data grid 80 24 paper 256 a9b8877cacf4feb5 -> ea417c9eb4eff695
  Data grid 80 24 paper mono 45a50905805f6fb6 -> b03a5d3cd2102c1d
  Data grid 80 24 paper truecolor c161ac56c4546925 -> 5cb900d7f35441b9
  Dialogs 100 30 junie 16 e494d2d80942a581 -> 72c9fe4a82959497
  Dialogs 100 30 junie 256 b8e03b9694e7b54e -> 1512aac2c7ee1e81
  Dialogs 100 30 junie mono 0bf6ea6b89660479 -> 86e016a505f839ce
  Dialogs 100 30 junie truecolor fd773272908f5b18 -> bc9b983a9922ff15
  Dialogs 100 30 paper 16 128403b53fc64a2f -> 98ac5efb031f1aaa
  Dialogs 100 30 paper 256 42239c42499a155a -> ea5e22be3007a6c2
  Dialogs 100 30 paper mono 7c6137f90b2d8d8d -> 36195730ea39b816
  Dialogs 100 30 paper truecolor d309b20f2b36403a -> 8813522e136e1c38
  Dialogs 120 40 junie 16 61b9feb1c4766240 -> c79043ef927423ac
  Dialogs 120 40 junie 256 fcaef6667a4a2cb2 -> 3b4e65d186dc22c3
  Dialogs 120 40 junie mono 8c98d711d0f90b68 -> 3a3c0f9faf637e73
  Dialogs 120 40 junie truecolor 1f90abfc5a2ee5dc -> 6d0c54ef112abcc5
  Dialogs 120 40 paper 16 5b156793c36370aa -> f34ccde8aee3ec11
  Dialogs 120 40 paper 256 de4f89f66a14cbbf -> e782030c44ad2eb5
  Dialogs 120 40 paper mono cafae4c489fd78b4 -> f4efb3cc05e206d3
  Dialogs 120 40 paper truecolor c2f8ebd1ced71ae0 -> 2097c094530fb154
  Dialogs 160 50 junie 16 5e541e21b5c95918 -> 2911684b38e46255
  Dialogs 160 50 junie 256 6fb913043c356442 -> d243f3dfbb423510
  Dialogs 160 50 junie mono 2cf4d7a3ea6a9510 -> 7ef1cb7bc01208c8
  Dialogs 160 50 junie truecolor d2029ed61d060b28 -> 11d77c3032b422a6
  Dialogs 160 50 paper 16 83bc30ade4e2f672 -> c32fd7a632e03ffc
  Dialogs 160 50 paper 256 eace90425eaf83d7 -> 92dc79ccd7db058a
  Dialogs 160 50 paper mono a90387af9289a374 -> fb3661e8c2585ebc
  Dialogs 160 50 paper truecolor b7fdd333cf615984 -> a6837a578a4592af
  Dialogs 80 24 junie 16 f7fce06f0fbaa088 -> 14134c1142f94553
  Dialogs 80 24 junie 256 77c61e9eb07a4184 -> 7198eec9dcdd7ca7
  Dialogs 80 24 junie mono fb165a68ce40264c -> a0b959eb3345ac93
  Dialogs 80 24 junie truecolor 5f0e4082dc9825e6 -> fdb68a108d53e19b
  Dialogs 80 24 paper 16 fabbcc82a14d342c -> 11720a927cf4cf8c
  Dialogs 80 24 paper 256 3819ced393dec44b -> 04326d28b546e61c
  Dialogs 80 24 paper mono 144d29939af2277e -> 8869e99940d4f1f1
  Dialogs 80 24 paper truecolor 0802eac740248c06 -> adddf3af6f255a8d
  Editable tables 100 30 junie 16 6076d093b657a547 -> 3419f900fcdc0f65
  Editable tables 100 30 junie 256 da6025a06d41c3ed -> c715af554c49a696
  Editable tables 100 30 junie mono 01db42c0cc5ac271 -> 557dc355d8e39950
  Editable tables 100 30 junie truecolor 04e71f8e6f360a0f -> 54eaf353a44174d0
  Editable tables 100 30 paper 16 83ccd9efa11010f9 -> 2d0adcf5d58b3108
  Editable tables 100 30 paper 256 1f6b412058e09aaa -> 600bbe2ecfc3312a
  Editable tables 100 30 paper mono 9a4b8fe6f49da67f -> 2811bea689b7999c
  Editable tables 100 30 paper truecolor a9fb0ef9d445517e -> fcb7beb9eb4cfcf4
  Editable tables 120 40 junie 16 d52d693ce521df4d -> 92d83b8f234fe94e
  Editable tables 120 40 junie 256 af0ce399352adcf6 -> 970a1a3edc9bccb3
  Editable tables 120 40 junie mono ccbf26147816acb1 -> 3b6b127627ec9ee9
  Editable tables 120 40 junie truecolor f4e379b137edee30 -> b06fa4f8ff204938
  Editable tables 120 40 paper 16 a565f6f2468966d3 -> f0402f91fad708fd
  Editable tables 120 40 paper 256 0114e5df29f96b78 -> 9fc2b4a1d2e08ae8
  Editable tables 120 40 paper mono 02625a2a554cd407 -> 44c0332b70b47c5f
  Editable tables 120 40 paper truecolor 9c53feb99374cfd1 -> 90b9d86f3482c662
  Editable tables 160 50 junie 16 f33a1c00806e4828 -> e350d68368ebfb23
  Editable tables 160 50 junie 256 1fbe0ec336c5a32f -> 4a4c810f180e02fc
  Editable tables 160 50 junie mono fe90187e6aacf206 -> a17857406e69ab3c
  Editable tables 160 50 junie truecolor 416ad29f9063cd15 -> ffafca58fa4d490b
  Editable tables 160 50 paper 16 9240b9c63217159a -> 1f9da956fb20fcd6
  Editable tables 160 50 paper 256 d18ecdd8760f6585 -> 460e539286d5a3e3
  Editable tables 160 50 paper mono df4ad767e820624c -> 2870ad80adc1f948
  Editable tables 160 50 paper truecolor ab2c3f97229024f4 -> 334eae0a31b54aef
  Editable tables 80 24 junie 16 3da8f9212f7f85dd -> dbc9251121697e30
  Editable tables 80 24 junie 256 a8c0a3982f2caff8 -> 44efeea3697357e8
  Editable tables 80 24 junie mono 61684adfedaaaaef -> ec8abf5476372b36
  Editable tables 80 24 junie truecolor 4bd580393729e7c5 -> 5da09ec0ccfd2b9b
  Editable tables 80 24 paper 16 a1d6196b14827017 -> 6feadfef0c60f8d3
  Editable tables 80 24 paper 256 9f20b2af1c39f43b -> 24f47a507961b057
  Editable tables 80 24 paper mono df366a5503c68475 -> 989152ee80831d2c
  Editable tables 80 24 paper truecolor a6f56cabaf502502 -> cd59df04ae3e1a5c
  Forms 100 30 junie 16 2fa78f33d3bc1ee4 -> 103f129d8d4f9dd0
  Forms 100 30 junie 256 1675b1ea31539f54 -> 2e7f50e792994585
  Forms 100 30 junie mono 56ce1a593db20a4b -> 3cd655b15dbdbec8
  Forms 100 30 junie truecolor 4197f191a34a213e -> 51feb484f3a91163
  Forms 100 30 paper 16 23b79924e3c87282 -> 98b75b80f2bdb10e
  Forms 100 30 paper 256 3a961c2bb24ab163 -> 5d9c9d9a34beab4c
  Forms 100 30 paper mono 33daeac92202aab9 -> cd20f3461144d130
  Forms 100 30 paper truecolor 52a0b73fe6925d9d -> 3cc3b2819be59599
  Forms 120 40 junie 16 81eedde7f4640c6b -> c9dbc17cab0155ff
  Forms 120 40 junie 256 e9c2235e3fdb8d3e -> 34b403881593b056
  Forms 120 40 junie mono 956e018677b71a78 -> f2b4144ec48ada11
  Forms 120 40 junie truecolor 010656e79c7e13a0 -> a7587c45f6542553
  Forms 120 40 paper 16 4598392a22f1900d -> 8344461fb92737d9
  Forms 120 40 paper 256 8bf3f96180410548 -> 0607443554c3e948
  Forms 120 40 paper mono ca2a731abb01be62 -> cf41c412f2f7fc4d
  Forms 120 40 paper truecolor 8b23b96e8f3f0c23 -> ec0bf5a4dca4b32d
  Forms 160 50 junie 16 512abd0495bb76b9 -> d8a82c9d046bb5f7
  Forms 160 50 junie 256 bb2863f898865716 -> 7a086815d3a61d9a
  Forms 160 50 junie mono af09b4aa48cb76ec -> f52a9a3f5bdb7f55
  Forms 160 50 junie truecolor 5ca04cf23c96042c -> 700478572b21b73f
  Forms 160 50 paper 16 92047362ec8a49cf -> 12a8775ecd1a85e8
  Forms 160 50 paper 256 afff4b49a65916cc -> eae28710fa1786a3
  Forms 160 50 paper mono 647080bd1193d3ce -> 264662e48c466815
  Forms 160 50 paper truecolor 3ddf8732366ac063 -> 2a932de169d8bb58
  Forms 80 24 junie 16 306f24045269e4c1 -> 083236ee9ac9fdbc
  Forms 80 24 junie 256 7e4154dfb96a65ef -> bf3b7462a52fe8e9
  Forms 80 24 junie mono 284f80fa5fea772c -> ef277be59bf764fd
  Forms 80 24 junie truecolor f16d3661424d94a7 -> 860ef6ea4f7e8b36
  Forms 80 24 paper 16 bbc8af7aad4ae931 -> 3db76fd96b7b7396
  Forms 80 24 paper 256 2501f23e07253376 -> 07bb72ae69afdbda
  Forms 80 24 paper mono 2710553ae3109560 -> ef8295a89c47aaaf
  Forms 80 24 paper truecolor 7f29602e4bd121b8 -> f6bee9df64fc4c75
  Inputs 100 30 junie 16 891a1ece20247604 -> 5ad18eab84a3d462
  Inputs 100 30 junie 256 9475002be4969648 -> 511d3a569ffd43b9
  Inputs 100 30 junie mono b16048c258ca9f2c -> 85ed68467ba196df
  Inputs 100 30 junie truecolor f67099d38aab0eec -> c1764ca6490d081c
  Inputs 100 30 paper 16 932a354c9ec61bf6 -> 19b1bec546252c81
  Inputs 100 30 paper 256 3bada1564678c500 -> 16f64d54dc10e068
  Inputs 100 30 paper mono bd5a1c177208d9ba -> 99a9539dde79e633
  Inputs 100 30 paper truecolor e80b0be25aa921cc -> ad5d0bcdc7d700b8
  Inputs 120 40 junie 16 e1746f8894ced3e9 -> 9a41b5cbed51415d
  Inputs 120 40 junie 256 15cae86994bbfd82 -> a803f2d2835deea9
  Inputs 120 40 junie mono 773933339d678199 -> 0ca91063afde0b1e
  Inputs 120 40 junie truecolor 01296ee331fd50d0 -> aed19d86f82f092e
  Inputs 120 40 paper 16 543c48b3b277a1f7 -> 20cc55e87d9e495b
  Inputs 120 40 paper 256 7df764774401a9d1 -> 68cc6541c358b080
  Inputs 120 40 paper mono 25f0185da08d92af -> c7171b3cbb8bf748
  Inputs 120 40 paper truecolor a00b22594dff0c3e -> 94a76c51065a4746
  Inputs 160 50 junie 16 e97e3b35b6f3e611 -> d0d2a3c43baa7f87
  Inputs 160 50 junie 256 2b2ecca280780c5e -> 5e3fb70f1874a55b
  Inputs 160 50 junie mono c4a359cacb34e881 -> 1d5a26add46471c6
  Inputs 160 50 junie truecolor 100ef7e096412b14 -> d4530c3ac206a8b2
  Inputs 160 50 paper 16 4debb3cfee5f4147 -> f97c0a0537894517
  Inputs 160 50 paper 256 622e783d9a542055 -> b66a3dfe906428a6
  Inputs 160 50 paper mono df0744783c6f31f7 -> 84191b84917d6cc2
  Inputs 160 50 paper truecolor cadb4cbdfa7772ea -> c69c9e271fd06ae6
  Inputs 80 24 junie 16 d2ee1eabc27462b0 -> ea7e47e16730af0d
  Inputs 80 24 junie 256 5f64c87aec99f9db -> 95d2af52d668af90
  Inputs 80 24 junie mono b9c77bad385cc330 -> d74c7be2b60221ad
  Inputs 80 24 junie truecolor ad1edc8de483eb45 -> 9d7542ca33ab2423
  Inputs 80 24 paper 16 2b84370996d88fc4 -> 6d8edf38961f0462
  Inputs 80 24 paper 256 1652d8fc5f753fbc -> 9c22f0867f4229f5
  Inputs 80 24 paper mono e3e1019071dd9972 -> 12b53c35087dd11b
  Inputs 80 24 paper truecolor d846c8b452ac9d3f -> 425a11194a3e0857
  Lists 100 30 junie 16 04b9f9e79a0d3144 -> eedc0c38852cfeb8
  Lists 100 30 junie 256 17f030c01d69f718 -> 4e26457c803e6590
  Lists 100 30 junie mono 3c9d4f584302a2ce -> 108db6aa9f892d29
  Lists 100 30 junie truecolor b3db4b5baf620cbe -> ef6b4e73eb870847
  Lists 100 30 paper 16 24216c59acac2590 -> 96ac5ae937993138
  Lists 100 30 paper 256 871c539ea6965346 -> 6169289828371aaa
  Lists 100 30 paper mono 6610e8a8e36fdf8a -> fde5421d012d8689
  Lists 100 30 paper truecolor 9949161ffb0b67a2 -> 5b484284c96e8a6b
  Lists 120 40 junie 16 d13064bb1985ac34 -> 70f01793ee7b9a7c
  Lists 120 40 junie 256 626e785cf7c26585 -> 81ab80bf66d3743e
  Lists 120 40 junie mono 6e00e74f87dbe662 -> 8af4e114bbee96d3
  Lists 120 40 junie truecolor e170a5f0fbf6eb15 -> 56887458435a4e8a
  Lists 120 40 paper 16 f150abcd819df40e -> 44d139c15f52cbdc
  Lists 120 40 paper 256 bac23a93da93a886 -> 457b6fc0869df12b
  Lists 120 40 paper mono 0d365f95a2bf6d1e -> cc65519091d2d579
  Lists 120 40 paper truecolor 033f5faf0e03c4af -> 8b9facdfb4fd1ee9
  Lists 160 50 junie 16 7803f35c60deda94 -> 1d06c2749bdd5b2b
  Lists 160 50 junie 256 ac5ea94e8ef7000d -> b46774f54900fa01
  Lists 160 50 junie mono 51eb1bfa1df1ae5a -> 251f99329dad923c
  Lists 160 50 junie truecolor 1edf6535d2704429 -> e699a991d8ab1069
  Lists 160 50 paper 16 263ba361ff0c9646 -> 49d322d658fd43b7
  Lists 160 50 paper 256 d41dd06df39f546e -> a4f8b3d9d6c252b0
  Lists 160 50 paper mono a7249c92e106924e -> a562be7457edc07c
  Lists 160 50 paper truecolor 2648fcefa0d0b3ab -> 7c197d3a029caa80
  Lists 80 24 junie 16 94bb4f50bdb28aaf -> dcabe406d5578927
  Lists 80 24 junie 256 a278961b50c0327a -> d7688b804890b332
  Lists 80 24 junie mono 2e18ec0fa2b18f6b -> bc87332f237d6da7
  Lists 80 24 junie truecolor d3ace5fa23766e3c -> 5210ef1b12f19fec
  Lists 80 24 paper 16 d283cdbdd6840319 -> 791fbd4b35405cc3
  Lists 80 24 paper 256 18878b251cc940d8 -> aec4d66bf0c365c8
  Lists 80 24 paper mono 921ae0aa7a01397b -> 6a19b7ff77bccfaf
  Lists 80 24 paper truecolor 21c8b9cb7fbb13a1 -> 3bd599b96370471c
  Overview 100 30 junie 16 c84eadf26d332567 -> 1f654c7cb5f8da57
  Overview 100 30 junie 256 cfa2df8299ac0206 -> 4c401ca04c10ec86
  Overview 100 30 junie mono 6a8381659bc5323f -> 9677a786d54df29a
  Overview 100 30 junie truecolor 84692e720b0087f0 -> 030b3e88d0fa099a
  Overview 100 30 paper 16 778034a14d0376e7 -> dc46e74de06d3dde
  Overview 100 30 paper 256 1820a7ea7c7e28b6 -> 0651a526828b26ac
  Overview 100 30 paper mono fda9ec3b6d4f8bc5 -> 63671ee2ac3cd9ec
  Overview 100 30 paper truecolor 2e950e642d961041 -> 4704d28506ebc844
  Overview 120 40 junie 16 14c87375bff25aee -> 0820072bdb86d37c
  Overview 120 40 junie 256 de3cbdd7c30c66d4 -> 8d0047ab5308b099
  Overview 120 40 junie mono 0449d1a419d152d2 -> 2fe0245c1536a66d
  Overview 120 40 junie truecolor 5a6e40a2b9f15534 -> f42c0cfa5a5350a5
  Overview 120 40 paper 16 1026eba325858ffc -> a97aedf4be90e575
  Overview 120 40 paper 256 8eeb7f06dc743267 -> 580c8a310718abc3
  Overview 120 40 paper mono be6e2a0fabffe8d4 -> 831f08ca1bd5f597
  Overview 120 40 paper truecolor 86391cedd8770781 -> d10cb321f9cc20f1
  Overview 160 50 junie 16 62e2cdff049ca85a -> 808b4e1a8e8e28cc
  Overview 160 50 junie 256 2a9d198136483dc0 -> ce80b738cbbfaa49
  Overview 160 50 junie mono bbaa93b4b3dc3606 -> fc1ef38332be41f9
  Overview 160 50 junie truecolor dfcf9a49c5c880f8 -> 447a17c4d5e95eaa
  Overview 160 50 paper 16 c145cc173575d73c -> 398e00246f2fa03b
  Overview 160 50 paper 256 e0531723a10bfa2b -> ce47f7437d69c148
  Overview 160 50 paper mono c35522348f6d861c -> 1f29bc1cc389772b
  Overview 160 50 paper truecolor cbd64c02e8459883 -> 57d5e91721f8a349
  Overview 80 24 junie 16 6d3cc65333e86265 -> cb22e10c546e1b2a
  Overview 80 24 junie 256 5ad9c8c4b6ec5e78 -> 114ab1c8b7dc4222
  Overview 80 24 junie mono 8c975a670b2b89ed -> bac36a26c0424902
  Overview 80 24 junie truecolor d90b4f828db1e180 -> 0a84372b36d2e4d3
  Overview 80 24 paper 16 b53cd06e40bb3ee3 -> 59ca5160d2da2300
  Overview 80 24 paper 256 9489a712b15dba27 -> e5eb6dd544dcaa45
  Overview 80 24 paper mono 6b36fb9320b75495 -> 759f7ab123c00bec
  Overview 80 24 paper truecolor 138c2bdf6853116e -> 1cf9cefbac1a27de
  Panels 100 30 junie 16 75cd1df93e65f33c -> a644842e1bc267c3
  Panels 100 30 junie 256 09263fb38d09d24d -> 74b6b2f398b11ee3
  Panels 100 30 junie mono e91fdf8ed2397436 -> 395b72d85bc93751
  Panels 100 30 junie truecolor a1e7b1ecc089df1c -> 49c81385a5548cb0
  Panels 100 30 paper 16 a5bb826cffba04d7 -> 27f975ec0617705a
  Panels 100 30 paper 256 1db25bf3bb8cb969 -> e28e975fec0f5482
  Panels 100 30 paper mono 8ee0ba08d85c0aba -> 316a7623e5882080
  Panels 100 30 paper truecolor 766bf28e720536db -> 3e4051f099724a60
  Panels 120 40 junie 16 018c43ebfe5687b3 -> 2373d5293ea9d8aa
  Panels 120 40 junie 256 4731b8cfe7582d73 -> 20bda44ea5cd9bbd
  Panels 120 40 junie mono d7b6fbfed4afa6cf -> 1008a32372c07e85
  Panels 120 40 junie truecolor 9c383e26d65122e6 -> 5bc89a5f1643f910
  Panels 120 40 paper 16 b98658ad7d4d8782 -> 9b9071b494e99384
  Panels 120 40 paper 256 22eb0fc8cac6bfda -> 01961fd2f9e909b2
  Panels 120 40 paper mono f471ef68c3b5a7eb -> 48723486a044edd0
  Panels 120 40 paper truecolor 1f0451e63d21b62f -> 303159214c89fda0
  Panels 160 50 junie 16 37729753f64bea41 -> b6cf7acabb4f8f42
  Panels 160 50 junie 256 607a8008dabcdf8f -> ab5946721cdc3d32
  Panels 160 50 junie mono 80e56576e9c19f1f -> 42d797249d73d815
  Panels 160 50 junie truecolor dc9ee20d1c8e1b92 -> c14f6a5f38c51d8a
  Panels 160 50 paper 16 f23ce255f1671428 -> f6b3975df53191ee
  Panels 160 50 paper 256 f7170f1aa69a5cac -> 525354d82e4a81e3
  Panels 160 50 paper mono 76c714791601b23b -> bac4bd93c69d6e34
  Panels 160 50 paper truecolor 7210b2b38efd9065 -> e5d5eb13caa726a4
  Panels 80 24 junie 16 3644fdc438411ca5 -> 3f77d34870c6e837
  Panels 80 24 junie 256 9680b3c2b61938e2 -> ecf48ab6bcea3ddc
  Panels 80 24 junie mono c3d7e295d1894533 -> 14074dc402879b06
  Panels 80 24 junie truecolor 75a2d6a2d30315be -> 56774b7ac13adeb2
  Panels 80 24 paper 16 49079bc8a9e6f18a -> 386409c29062481e
  Panels 80 24 paper 256 1ddcf1d54cf1bb0c -> 5c70bf18e4ce4fa3
  Panels 80 24 paper mono 5a04e2bb68014829 -> beb321d57b76ae53
  Panels 80 24 paper truecolor d967fd6f844e7c94 -> 2113b9190833e390
  Pickers 100 30 junie 16 db846f269966fa40 -> f3fb259f2bf7bfe7
  Pickers 100 30 junie 256 a1416915e04c0ea6 -> 9c7f040ff62b2c10
  Pickers 100 30 junie mono 29b3d18247946c2c -> 42a1e41211ea75cc
  Pickers 100 30 junie truecolor e0b9b73551b4bfbc -> 47f61dbb7f9e17b2
  Pickers 100 30 paper 16 dcddf7ece4e3c6e5 -> b0fa72aeaa652f0b
  Pickers 100 30 paper 256 8402074d949d2f69 -> a12d64b2172865a4
  Pickers 100 30 paper mono 74660610fc0ac01a -> dcb1360207db5cfe
  Pickers 100 30 paper truecolor d81bb96d4e983071 -> 77a3752785ed964a
  Pickers 120 40 junie 16 281329f85c66eb19 -> d79882a986c0f5e1
  Pickers 120 40 junie 256 6d7fac195b17d6d6 -> 8e0b63ad61b3b1cb
  Pickers 120 40 junie mono bb91160f61f6048f -> f55a29bb359b2c8a
  Pickers 120 40 junie truecolor 48a2d91ac4ffe26c -> 62fca270ca0ae0a4
  Pickers 120 40 paper 16 6f582461c4898442 -> 70a7c50331e34b57
  Pickers 120 40 paper 256 1d9207655641ccce -> ebcadf611eae5bd1
  Pickers 120 40 paper mono 213fd8dabf3e4c9b -> 08317b5b81d17268
  Pickers 120 40 paper truecolor ff84ea4e82393235 -> 668159e9d644e414
  Pickers 160 50 junie 16 e0ac5d9d5ebfbc8c -> a77599b52dad3500
  Pickers 160 50 junie 256 a043b8469116a241 -> 41f929e54cf2e452
  Pickers 160 50 junie mono 87af1d37a30ae1f2 -> c02f6a3dc54c3a0b
  Pickers 160 50 junie truecolor e43a5609c603f2af -> a7d8d516a587c225
  Pickers 160 50 paper 16 c7a240a931e14f05 -> b5e7238c7eb3893c
  Pickers 160 50 paper 256 93d12563a106aae7 -> 5cc36dbcbc60346c
  Pickers 160 50 paper mono a58a9ff63d0d283e -> 53c69bb0ebd7c43f
  Pickers 160 50 paper truecolor cab2a1ef563da322 -> 8d604ecf5817dc59
  Pickers 80 24 junie 16 f7efd81c6d2adbf4 -> ad21dd1b946bc89b
  Pickers 80 24 junie 256 c6488eb5c7b1c82c -> e0e49966088ef65d
  Pickers 80 24 junie mono 0b86ad1a30240c14 -> 8fc293a9d65efe67
  Pickers 80 24 junie truecolor 2c017924fb26a86b -> 7576363b2a6d48aa
  Pickers 80 24 paper 16 645bed658ef8b4ba -> 08a364caa67e53ef
  Pickers 80 24 paper 256 b4528d893418c1e3 -> 37aae298df2d3011
  Pickers 80 24 paper mono 73dc6a6420e41460 -> b011a7b75a609f6b
  Pickers 80 24 paper truecolor 98c09d0f86f8da88 -> da79e2e95ac12284
  Progress 100 30 junie 16 695ff0b7a9e358c9 -> fc5582ef049dabbf
  Progress 100 30 junie 256 6103ddd326dfaeb1 -> aefd92476cabae89
  Progress 100 30 junie mono 57d5d76fbb2edc37 -> 04be809a01748a6e
  Progress 100 30 junie truecolor 331a051e80d375a4 -> 32d227d77e5475e8
  Progress 100 30 paper 16 e14866d3cfe61ff0 -> 8b185d9399c7259b
  Progress 100 30 paper 256 9bae6c4a78eaaa36 -> bb9798676dec9802
  Progress 100 30 paper mono 342958bb7bfa2613 -> b9a354919fc7d46c
  Progress 100 30 paper truecolor d6264f0111e33e95 -> 20abee474a67b6d8
  Progress 120 40 junie 16 5272d7de8b3facf7 -> 1aed0e7c2c18945a
  Progress 120 40 junie 256 a0372ba431c72b56 -> 9f6fbf24f47569d4
  Progress 120 40 junie mono 8b18ff5ba4bdc67f -> 549f09927a321876
  Progress 120 40 junie truecolor 7baf59f8d8d5b5e9 -> bd2dba5d18ea4eea
  Progress 120 40 paper 16 87fc8e8585b12bf8 -> 7b5805eab33b6527
  Progress 120 40 paper 256 18bc0487fdc961ea -> 831d6eaa05ec0e62
  Progress 120 40 paper mono 848098844c719ab3 -> c122ff7ed1214cc4
  Progress 120 40 paper truecolor ccaef0fb2d609646 -> cbbb60f5c8ed3070
  Progress 160 50 junie 16 4204592fa21c7888 -> 926fa0b441887786
  Progress 160 50 junie 256 e42190a1d31a9725 -> a9ec5fa7c209c314
  Progress 160 50 junie mono 3611255fca929077 -> 9d20cd6c7d30f7d5
  Progress 160 50 junie truecolor 2280c7736bc3ccf6 -> 6579623c825d117b
  Progress 160 50 paper 16 4288e343372058bc -> 9dad85c0ff9d9c24
  Progress 160 50 paper 256 423c1dbddb3a9fe1 -> 6122d0e2fc675665
  Progress 160 50 paper mono 15d7bf6493dd714b -> 98f94aeed5688609
  Progress 160 50 paper truecolor 9262ae2c70674700 -> d06649e3d77409de
  Progress 80 24 junie 16 5d27f3cfc7d8bcc1 -> 21b395830503f81f
  Progress 80 24 junie 256 d8c1dfded7c6cf2f -> 6629a63ac81475e2
  Progress 80 24 junie mono 5cb6a4ae62765f68 -> c1572494f7d1960b
  Progress 80 24 junie truecolor 6777e2423e22d137 -> e82373aefe85f94c
  Progress 80 24 paper 16 b75eab2e7aa32739 -> 2af3e960c56f4750
  Progress 80 24 paper 256 d8e34ee1dcba7782 -> ccc581a11589b00e
  Progress 80 24 paper mono b3181a3d9f272156 -> 70ad2ed9944c2e4b
  Progress 80 24 paper truecolor b44a5347eac1c580 -> a7c15131b93b2b86
  Scrolling 100 30 junie 16 cc3b79fa814df996 -> 6f264f1f50e9da85
  Scrolling 100 30 junie 256 58355b7c4abcbb82 -> fd67c80f856836a3
  Scrolling 100 30 junie mono 85609fe8da312ff0 -> fbfc92ea721450c4
  Scrolling 100 30 junie truecolor 04650ab5350712d2 -> b29f09c2fca6daac
  Scrolling 100 30 paper 16 772406b3cf8ba652 -> 870d1d4163f9464c
  Scrolling 100 30 paper 256 65adb940463e8b18 -> 3fb517fa571c73c7
  Scrolling 100 30 paper mono 48feb76820439922 -> b41ecace5edc266e
  Scrolling 100 30 paper truecolor 8335bff36d8bfa64 -> 73a4c22e444635fa
  Scrolling 120 40 junie 16 591493663d8695fa -> cc8305fa7a835c22
  Scrolling 120 40 junie 256 ab75554d4eef44d1 -> 7dc9b2b0e5accd46
  Scrolling 120 40 junie mono 6a2161e65c44211c -> 290b10eac8693159
  Scrolling 120 40 junie truecolor 6df9bf911c28b495 -> 9d5821a79cdfd361
  Scrolling 120 40 paper 16 803759e23bcd20b4 -> e7d847664ccc5ad5
  Scrolling 120 40 paper 256 cc335bce68b6822e -> 419ab7f1a73e5cc9
  Scrolling 120 40 paper mono d0d0932e6b58109a -> f2179015d85451bd
  Scrolling 120 40 paper truecolor d2d983846353e3a7 -> 8c345f1d04fb3dad
  Scrolling 160 50 junie 16 adaff424a09fbbdb -> 2bbdd62f3c4942c5
  Scrolling 160 50 junie 256 db2ee33c9329717e -> 8e5c4daedddc6d7b
  Scrolling 160 50 junie mono ce1c20c42d4cd6b5 -> 73ee680f20893528
  Scrolling 160 50 junie truecolor edae3b93a80e5e49 -> c461ead04aafdc8e
  Scrolling 160 50 paper 16 5069f5b16b54794d -> ed73051fe66b3af8
  Scrolling 160 50 paper 256 a757c7f64290f017 -> 2b28b46971de043a
  Scrolling 160 50 paper mono b3161ac1d825b3f1 -> a0d8d606d74a324e
  Scrolling 160 50 paper truecolor 4ed67ddde9873420 -> 000be41d23513e84
  Scrolling 80 24 junie 16 0460b5f54c90e368 -> 5f3af0af1e5dd29b
  Scrolling 80 24 junie 256 e63082cfde15d8d1 -> 7d6f772cb780ec56
  Scrolling 80 24 junie mono 6ac8c2cdc8d7c26a -> bf797129982a64d9
  Scrolling 80 24 junie truecolor 9f8e4438bc18683a -> 5f2c4ef84b62b468
  Scrolling 80 24 paper 16 6b3a9d64cd3f6ebc -> fd140462ae385a50
  Scrolling 80 24 paper 256 c38955049baae564 -> e17a99a22661475a
  Scrolling 80 24 paper mono 45a079abd91a1146 -> 149fcef40882ae9d
  Scrolling 80 24 paper truecolor 09275e4321fc7df1 -> 578b31aaeac9f91a
  Settings 100 30 junie 16 6c905aabbf32d66a -> c970377cf789bfc3
  Settings 100 30 junie 256 2640e3848afff69b -> 8dc641ea67ff816e
  Settings 100 30 junie mono 86547b5f6f021942 -> 051d567c52f5a768
  Settings 100 30 junie truecolor 723b483d15fe0726 -> 882a4442829c57bc
  Settings 100 30 paper 16 1929d7c4db2655c7 -> eeb1348dc39a09c4
  Settings 100 30 paper 256 08f3f77857a15458 -> e047e857902b58a2
  Settings 100 30 paper mono bc67608decf5fba6 -> 7b32e41016d9eb1c
  Settings 100 30 paper truecolor f9e132fb7db7218a -> 9d5e48b5b1a84395
  Settings 120 40 junie 16 2cd103c827e6c673 -> f481015ba957ca1a
  Settings 120 40 junie 256 df5c7acdc48666af -> 356d4115df9aa28e
  Settings 120 40 junie mono 0418a57de6d1a197 -> 5cabbbf71ea928cf
  Settings 120 40 junie truecolor cbab485837d81d9a -> a70afbaad85e965c
  Settings 120 40 paper 16 8d1668810e8d321c -> 597f884ae6470451
  Settings 120 40 paper 256 f3a92958af1d8691 -> 1a1cda673ee6cddd
  Settings 120 40 paper mono 927bc5169a1e705f -> 135518824e5565a9
  Settings 120 40 paper truecolor 1f7c97b9f14921bc -> e66788ef33e92505
  Settings 160 50 junie 16 051dceb6832330eb -> aae1dd49c5901cab
  Settings 160 50 junie 256 9273a879d47110bf -> 675abb24af5f0d79
  Settings 160 50 junie mono 21f774b4d2655b6f -> 204e1602ecf898fc
  Settings 160 50 junie truecolor 4dc99bafe25fbcde -> 0cd3f9f361679969
  Settings 160 50 paper 16 bf360a9410212844 -> d1d567831fec8dfe
  Settings 160 50 paper 256 f6cb705c6902db89 -> 16cd241a4794743c
  Settings 160 50 paper mono 6645a03eb7769e7f -> 8f98336526013398
  Settings 160 50 paper truecolor 603ed507f4431ee0 -> 1388cd944fefa6fa
  Settings 80 24 junie 16 2c0b5f445a8afa68 -> 3c49eeac17f5c4a7
  Settings 80 24 junie 256 1761fb45e9d0bcb0 -> 5769f1b8f8bd1b23
  Settings 80 24 junie mono a68e214bfec6a4e0 -> b45b43f3de4d8d8b
  Settings 80 24 junie truecolor 1a1cf89239a4319c -> 5448faee9ea0c06c
  Settings 80 24 paper 16 232485ed5e28dfa4 -> 5b0f5b3084210884
  Settings 80 24 paper 256 32ea2a5d878cece7 -> 9979bd2c31c6f6a3
  Settings 80 24 paper mono b44ab94a569d75f2 -> 120daf25f38caed5
  Settings 80 24 paper truecolor 698d4ebb52facead -> 3ff520540fe79c70
  Sidebars 100 30 junie 16 a7c37eb4273c18a8 -> 62bc4a02e438cc7a
  Sidebars 100 30 junie 256 6ca7294384ad29d4 -> 06b6ae357e16c17d
  Sidebars 100 30 junie mono ddc61c05a79444cc -> e85dcc897334d25b
  Sidebars 100 30 junie truecolor a11d4b94c5b32dee -> 499ff63f225b0d0e
  Sidebars 100 30 paper 16 e91d08fd58611120 -> 563f392825c4937d
  Sidebars 100 30 paper 256 d8cbdd4856b71410 -> 8acab28047796584
  Sidebars 100 30 paper mono f48f1b0f963c488a -> 5060e8bb29201485
  Sidebars 100 30 paper truecolor b01e0a67c4a84a70 -> 4be17d8cff952e36
  Sidebars 120 40 junie 16 d06d2a6953ea9c3a -> 8d9bef135198caca
  Sidebars 120 40 junie 256 292c20cb063d06f2 -> 6cf13dc5ac25e654
  Sidebars 120 40 junie mono c10f862bdaf25826 -> 9d3974500dccba9b
  Sidebars 120 40 junie truecolor ae45f894183dad78 -> fca4ec6a3c5a40b2
  Sidebars 120 40 paper 16 190d35dd9900a5c0 -> ad2fcbb246180975
  Sidebars 120 40 paper 256 61b21773f986f6f4 -> a57eb0256513f885
  Sidebars 120 40 paper mono 46a0b63e08b8cb44 -> 37f51057c2b379db
  Sidebars 120 40 paper truecolor dfac967ad5797f96 -> 924484145f33fd44
  Sidebars 160 50 junie 16 5b506ec0292ff1b7 -> b3b3cac9722bc9b3
  Sidebars 160 50 junie 256 39e6faf396190088 -> a393a9a9f9d910d3
  Sidebars 160 50 junie mono faca4be1505de2e5 -> 24d30ce442e4eeaa
  Sidebars 160 50 junie truecolor 1e49a16c29efad92 -> 1228610e05aba909
  Sidebars 160 50 paper 16 dc863ea86a8da69b -> d23c10a12e48a610
  Sidebars 160 50 paper 256 ce832b5dabc41535 -> 4ae6751af82ea83a
  Sidebars 160 50 paper mono 5e6311b02d85b73b -> bb461e03f554e978
  Sidebars 160 50 paper truecolor b8a4b306de28d5c2 -> fb20760b2523d0d9
  Sidebars 80 24 junie 16 0ab64bdc3f05ae37 -> fddb88f2ff04aa8e
  Sidebars 80 24 junie 256 9b412685f29bb263 -> 4b6659cc8332c0d4
  Sidebars 80 24 junie mono 721255f17d861a43 -> 1d7206308e2a78a4
  Sidebars 80 24 junie truecolor f3af464d7fa340d1 -> efbbdec9cfcb62b4
  Sidebars 80 24 paper 16 179360e0e2808085 -> e1d0bcca6f8cfc35
  Sidebars 80 24 paper 256 3ba2f5d684155ded -> a28fca0ea7bb02a9
  Sidebars 80 24 paper mono 565e870cfbdc8b73 -> 058e3f3edfaba29e
  Sidebars 80 24 paper truecolor 36e438fac2c2fa2d -> c5348cef51a81892
  Tables 100 30 junie 16 33902d75922de664 -> 579cfb8332ca3a9e
  Tables 100 30 junie 256 3e9af10c2a5e5e0b -> 70e744f65fc02bb2
  Tables 100 30 junie mono 26a19bf34bec53a7 -> 7325fe8e747a4e48
  Tables 100 30 junie truecolor d02470a233b6684a -> e24b3049a1a17513
  Tables 100 30 paper 16 55468d7cee898120 -> 9bae1948ae9a22bb
  Tables 100 30 paper 256 9bfb9897144c024a -> d4559a63ec42710f
  Tables 100 30 paper mono 27c73976616c2ca9 -> 4af21392152ded5e
  Tables 100 30 paper truecolor 64522c741e260e1f -> 62562c17a3429313
  Tables 120 40 junie 16 998267e3cadd15f8 -> e7bf8f5a614ad248
  Tables 120 40 junie 256 a1597be77dc34f85 -> 1027ad52921e9770
  Tables 120 40 junie mono 7e6c63ea6874e29c -> 1e2b7479b3050d44
  Tables 120 40 junie truecolor 787ba406b60d7ed1 -> 8e13ed023b8ce5f9
  Tables 120 40 paper 16 6c779f5143b99d74 -> 42fdc0f05bbdc893
  Tables 120 40 paper 256 db218eac7c2196ad -> a888a7456dc1eaea
  Tables 120 40 paper mono 4b54535694b24562 -> ed4dd53ad33f7c5e
  Tables 120 40 paper truecolor 0f3680938fe3cee9 -> 8dfbc96139a3e672
  Tables 160 50 junie 16 79eac3e1bb808cb9 -> fc4f620e766e10dd
  Tables 160 50 junie 256 7d4c08a9f045ccd8 -> 84b677ab0f6ab3b5
  Tables 160 50 junie mono 5219b568ef9972f7 -> cbdab901281907e7
  Tables 160 50 junie truecolor b6a107ce2e3f4502 -> 3bac44f5bfec3c25
  Tables 160 50 paper 16 ed61b2e9e08ef219 -> e37fd2934cb07e62
  Tables 160 50 paper 256 542b2c462ea100e6 -> 7311920a1b45b8d1
  Tables 160 50 paper mono 1e4241f13f0acc1d -> 505727bb8fb29679
  Tables 160 50 paper truecolor 36797e5cd89530c8 -> 538b96d5e62918f7
  Tables 80 24 junie 16 a31d5256122a5e94 -> ae94dbafdc2ae99e
  Tables 80 24 junie 256 1575db1ef0431e38 -> 55430c5e9f2acf6f
  Tables 80 24 junie mono 8f5b063a1c5da6a3 -> e2e38add79ed6c77
  Tables 80 24 junie truecolor 5231b5d33ae9856f -> 7b54844ece174f42
  Tables 80 24 paper 16 83bc3073229fc2f6 -> 486e4d393476f8ed
  Tables 80 24 paper 256 23f6d8c17a4884f2 -> 014626f7b5f6dadf
  Tables 80 24 paper mono ef0d1fa7db2f08bf -> 4c0792356014923f
  Tables 80 24 paper truecolor 718b302f31ff7518 -> 268df38b118ce7b6
  Task runner 100 30 junie 16 af6d249bb441de38 -> 5aabf7d8f89d71be
  Task runner 100 30 junie 256 ea91d5ef71b82137 -> 2d893b2480cc19bd
  Task runner 100 30 junie mono f50ab5e1998b46b5 -> 10c535564a5cc4cb
  Task runner 100 30 junie truecolor 06195ede11c2b975 -> 9f393e4d5ce9030b
  Task runner 100 30 paper 16 e93219e16f8791ed -> 8cfcd745866c5b7f
  Task runner 100 30 paper 256 7e64ce0faf095187 -> 19076548d356e9eb
  Task runner 100 30 paper mono 6b2f4de099e9733d -> a5f94b81b42796ad
  Task runner 100 30 paper truecolor 578bedb543e38d13 -> d77e83fe65f09325
  Task runner 120 40 junie 16 ff2e3fa5b817a597 -> 558df6eca395be50
  Task runner 120 40 junie 256 13cd0699cb2fb683 -> dbb85c4f35657a00
  Task runner 120 40 junie mono 4a09f8058175c938 -> 1621fee37e2fdf35
  Task runner 120 40 junie truecolor 007c7615d10525a2 -> 64dd5fc84105ac64
  Task runner 120 40 paper 16 0841bf75618ae303 -> 1e1d8a509ef113ef
  Task runner 120 40 paper 256 e0388991979f497f -> de0460a86161c2ed
  Task runner 120 40 paper mono fee12e9730599dc8 -> f2ca85dd47db9571
  Task runner 120 40 paper truecolor aa1d72d13e38cfd3 -> 8f6b4854a1534082
  Task runner 160 50 junie 16 e5f76021b0328417 -> 6705d4449b9a6d51
  Task runner 160 50 junie 256 e349f66c7c9490d3 -> a7ed21f19e4902cf
  Task runner 160 50 junie mono ddeeabb80fc891d0 -> 568310c8988a109c
  Task runner 160 50 junie truecolor 13bab8f30d0c6076 -> 999cf69c4ba9a8bb
  Task runner 160 50 paper 16 2a1875dfb6abf55b -> 7f1a4070b94ddb8a
  Task runner 160 50 paper 256 a4c380c5d102a867 -> 4b8bdeaebaeca392
  Task runner 160 50 paper mono 12fddb8995330608 -> 58efd1fef33aa6ba
  Task runner 160 50 paper truecolor fdc38dbb3b5c677f -> 8671ffbcdee4c9fb
  Task runner 80 24 junie 16 a29167a60866e3c0 -> 68e4c58a5da54d8d
  Task runner 80 24 junie 256 867c39c732b875b5 -> 24151e50d57cf1af
  Task runner 80 24 junie mono 00c3b3dbc086cb07 -> 01a3e25cee975331
  Task runner 80 24 junie truecolor 719af30785c32b3e -> b210f09ee4bcc2af
  Task runner 80 24 paper 16 60f4c6c229a2a42d -> 44d86759277ffd0a
  Task runner 80 24 paper 256 783a9e0b9061201f -> eb7132dbfc0eaf62
  Task runner 80 24 paper mono 0649700489f47b41 -> 638edf81e3e82595
  Task runner 80 24 paper truecolor e8174d4d7a1ce997 -> d0df8a5d02af7886
  Terminal 100 30 junie 16 24c03297aa731088 -> a93d5bd7873eaa3d
  Terminal 100 30 junie 256 1ef2769c14c87d5b -> 279b4f37502bcaee
  Terminal 100 30 junie mono 89a034135c9f9360 -> 950f404e57dee5b6
  Terminal 100 30 junie truecolor 69b1af9ad50f184c -> 489f7b3da54b6f64
  Terminal 100 30 paper 16 d4f7ba97711b99b0 -> bc94d7f93c6eb4eb
  Terminal 100 30 paper 256 1d8cae1191cd0455 -> 45db26b9ce87d281
  Terminal 100 30 paper mono 719262c4a58441a6 -> c69e7ceb785e0481
  Terminal 100 30 paper truecolor d2cb6ec566c44ea9 -> 62a1a08409389ad9
  Terminal 120 40 junie 16 a85064ca4212d566 -> 6fe2112b229dd4d7
  Terminal 120 40 junie 256 8a95b49afd6c81dc -> 9bf3c4a2ffa3a282
  Terminal 120 40 junie mono 4e885c1560a99110 -> 403a9da450c6254e
  Terminal 120 40 junie truecolor c31beb317c8d9543 -> 51822b6e3071199e
  Terminal 120 40 paper 16 7951fccdb947dc06 -> 01bd633a8c0e6feb
  Terminal 120 40 paper 256 64dfe3fd4c907f2b -> b0218746cfa0f9a3
  Terminal 120 40 paper mono 092b59b50009b58e -> 9ca237cd44d7defd
  Terminal 120 40 paper truecolor d6fd0dc2e53fcf4a -> 0aeae7aed0e97a46
  Terminal 160 50 junie 16 8ddca7891c65abe6 -> 4cac4a0ff50dd8a8
  Terminal 160 50 junie 256 eb83f6e1210319c8 -> a4e56121ca5888b1
  Terminal 160 50 junie mono 6a2e135d86bb5ab8 -> 566f689ee3c6ee51
  Terminal 160 50 junie truecolor a5802c385a318dff -> 0a6db3d7268677ea
  Terminal 160 50 paper 16 655db358adebb1fe -> 67cf50754ca061da
  Terminal 160 50 paper 256 814ca24465fd6b03 -> ac8df2581c2fbe5b
  Terminal 160 50 paper mono c98f8057e56be2be -> eb95c8c193d74bfa
  Terminal 160 50 paper truecolor 5e44e4d19fae4be6 -> 90c0320b5b3ea96e
  Terminal 80 24 junie 16 4cd59c73cd0a7071 -> 66aea04906f851cf
  Terminal 80 24 junie 256 269bd5fa04e54e5d -> e8a93e85b5ce6258
  Terminal 80 24 junie mono 82029e3b113e2793 -> d551eb366e8e616f
  Terminal 80 24 junie truecolor e1afe330c3056262 -> 7ab0ce9bb0299e14
  Terminal 80 24 paper 16 b40ac4371613b6cf -> 2db9dad386200d76
  Terminal 80 24 paper 256 33204b922eaa6556 -> 177720208a6842bc
  Terminal 80 24 paper mono 602c7ec89275f193 -> 193cd29d868fd525
  Terminal 80 24 paper truecolor 425bf39619b58c1d -> 324ac9de98bd78e0
  Text areas 100 30 junie 16 a47ebee7e8bec816 -> da63daae694e732d
  Text areas 100 30 junie 256 3405a5aeb73a1c88 -> 1071ac94f47e30d8
  Text areas 100 30 junie mono a3e94514dcf7001e -> 9faa4f32f2d3d9b2
  Text areas 100 30 junie truecolor 55e8235c610d06ff -> 6a20d86772cf001d
  Text areas 100 30 paper 16 473a15728b296f6e -> 35357091d19d5d94
  Text areas 100 30 paper 256 f40c5e12301b72bf -> 192912bb51b51299
  Text areas 100 30 paper mono 9b3bc64a138c7862 -> 6b2ac9eb0b0a5422
  Text areas 100 30 paper truecolor 2838ccf6326cd991 -> 6b07b2492963a705
  Text areas 120 40 junie 16 5a33f477e8a0eba0 -> 3f5e6f486ac9da6f
  Text areas 120 40 junie 256 af74ca978db0ccab -> ec236f52cac294f7
  Text areas 120 40 junie mono 9dae94656c731e26 -> b07c19bb26b29110
  Text areas 120 40 junie truecolor eea6373c74e3d195 -> b49cc09b31d8976b
  Text areas 120 40 paper 16 dad5ae35430aa5fe -> 371c5c22208e8b38
  Text areas 120 40 paper 256 ca85c4ec84f4bd7f -> 1661e68783d93c6c
  Text areas 120 40 paper mono 39500e5ad7acdfb0 -> 0d19322f564841b2
  Text areas 120 40 paper truecolor 02564b8a84f0510a -> 0915a0b0dca84bab
  Text areas 160 50 junie 16 5cd2bce685c6c3dc -> 5651dedee62809b2
  Text areas 160 50 junie 256 a15302655166428b -> 7c9c703fe9722e38
  Text areas 160 50 junie mono ac730c0cea1f25f2 -> 5028cc3dead59a2d
  Text areas 160 50 junie truecolor c3cac9542a64d729 -> 4589b0553290419c
  Text areas 160 50 paper 16 6ca46a05385e2976 -> 844d95b8e463fbdb
  Text areas 160 50 paper 256 2acd0de6fcf220af -> 164086a04ac3331d
  Text areas 160 50 paper mono bf077aa016c61754 -> e5602ab8c75d4173
  Text areas 160 50 paper truecolor 7516f0e5255550a2 -> a4710ce812560892
  Text areas 80 24 junie 16 a19792b1cee0d811 -> f37fb225d4da22ef
  Text areas 80 24 junie 256 702d939a5cea8352 -> a625b49ffca6af0e
  Text areas 80 24 junie mono 4687c0109d44143f -> e91219cfdebfdf1b
  Text areas 80 24 junie truecolor fe35252d61e637d9 -> ca306b95c0c5d80f
  Text areas 80 24 paper 16 bf0e5286ae1ae863 -> 7238bda56ea7538e
  Text areas 80 24 paper 256 4eb976524c7ce4f8 -> 3f2adee4e2e681ef
  Text areas 80 24 paper mono df16aa49d48c9041 -> 4d7fa8daa0266561
  Text areas 80 24 paper truecolor 905581249740106f -> fa5bc191647fdcd5
  Trees 100 30 junie 16 8ae53adc3a788b5b -> c54f4e62482f99e2
  Trees 100 30 junie 256 332e4dbe657a5fd4 -> a04f2fd9fb15d09f
  Trees 100 30 junie mono b5c35331451601ad -> 4ae935af93ea1ef1
  Trees 100 30 junie truecolor 6b0e6e31fad1f162 -> 74bfd7486630230a
  Trees 100 30 paper 16 e0bd6c6e1ce24e89 -> 7bfb4b14a40a1925
  Trees 100 30 paper 256 70c182e30e2a2d03 -> 5758365972d954fb
  Trees 100 30 paper mono 084ff9b273bd264d -> b22d1eedee935adf
  Trees 100 30 paper truecolor ea5587ca0c44545e -> d6173830a3e0edf7
  Trees 120 40 junie 16 0a85bb7a355e46b6 -> c689d6b742d31c73
  Trees 120 40 junie 256 de5f61f9ed04bc80 -> 3dc193354d956805
  Trees 120 40 junie mono 1ad8115bea15fee8 -> ff653e36c157a50c
  Trees 120 40 junie truecolor 07df4021fd2db07a -> 9d79e0bdab180110
  Trees 120 40 paper 16 afc9ec8f0f02f1f8 -> de088ebfa8eb2fe2
  Trees 120 40 paper 256 f5d7662eb2e786ba -> f5043171cc94d99e
  Trees 120 40 paper mono e43b2d3219b89cd8 -> ab8b6fd685bd1c6e
  Trees 120 40 paper truecolor b878a24e2d7636f0 -> a274995d500e5bd9
  Trees 160 50 junie 16 a8ba5b64202700fe -> 0b03cd125d31bf98
  Trees 160 50 junie 256 03476280d535fcb0 -> a66ef91803bc0570
  Trees 160 50 junie mono 51d2b32728b68240 -> 2383ee5a7807c9bd
  Trees 160 50 junie truecolor 139465568f5131b6 -> fdf15678a067f857
  Trees 160 50 paper 16 9b5a29124c278550 -> 2131fc106d257a93
  Trees 160 50 paper 256 c8cf018be7a31c92 -> 867c1c41f0ba55df
  Trees 160 50 paper mono 8b68a83885c58828 -> e449b20051400947
  Trees 160 50 paper truecolor cac6d55ec439dda4 -> 147d3f6d899fead2
  Trees 80 24 junie 16 332ba41080842901 -> 2dfd108ce024ebed
  Trees 80 24 junie 256 383ae8bdec52a041 -> dd023321bb6ea6a3
  Trees 80 24 junie mono 1834fae7b875ab81 -> 38bdd98b8d747355
  Trees 80 24 junie truecolor 87e6c125e0b15873 -> 4a61cb98f84481d1
  Trees 80 24 paper 16 a2ab802e3e241399 -> 99cb5d35b1342530
  Trees 80 24 paper 256 0a31055d3a6e6a85 -> 7609d0054c479f1d
  Trees 80 24 paper mono 9bc265923bbad909 -> 2515dcbd28c87397
  Trees 80 24 paper truecolor 6881da448052bed5 -> 45fe30eeb952c420
- added:     32 keys: the Editor page, recorded for the first time under its corrected name
             `{Code editor} {80 24,100 30,120 40,160 50} {junie,paper} {truecolor,256,16,mono}`

- class:     intended

- reason:    §20.10 item 36. The movement is the deliberate union of reviewed coverage
             expansion and absorbed historical flows, pinned by Holla production reference
             fixtures; no unreviewed page drift remains after the reference tests.

```

## Item 37 — TablePro surfaces re-blessed to the merged shell

The merged TablePro shell is the historical restoration already recorded on
the main line plus the branch's role-owned corrections: legacy tree gutters
resolve `Role::OnSurfaceInverse` through `StylePatch` instead of raw black,
the embedded Grid carries the detailed-gutter and full-height scrollbar-track
corrections, and empty/hover painting follows item 35. All 21 surfaces were
re-blessed against that merged rendering; the recorded truecolor cells moved,
their mono siblings are accounted by the merged entries.

```
- surface:   tablepro digest @ 21 surfaces / 120x40 + 80x24 / junie + paper / truecolor
- captures:  apps/tablepro/tests/visual.rs::tablepro_visual_baseline green; refreshed
             capture matrix and independent review follow the integration freeze
- tests:     apps/tablepro/tests/visual.rs, the tablepro unit and interaction suites
             (164 passed), undo-fixture neighbor-tab test
- moved:     168 keys (rewritten to the merged diff):
  completion-popup 120 40 junie mono f134dace0e8c2dad -> 20f074f4a04d87b1
  completion-popup 120 40 junie truecolor 0d893043379e3b1d -> 0226bb9b3235e758
  completion-popup 120 40 paper mono 765979f9eb8a3489 -> aa84d94de6237c07
  completion-popup 120 40 paper truecolor 0ba20e97193e73d7 -> e457aab981ceda16
  completion-popup 80 24 junie mono ad086db520747ec3 -> 4a841db94d0ed96b
  completion-popup 80 24 junie truecolor 1717a94ce9f6f515 -> 7bf9874bad715331
  completion-popup 80 24 paper mono 74b5475e97cbf4df -> e6efd92f9b97dba1
  completion-popup 80 24 paper truecolor e04827894f8dbc4e -> 41715bbec37573bc
  connections 120 40 junie mono 0c2ccf3fc2ce97e9 -> 285133213b807edd
  connections 120 40 junie truecolor c1fdb4fc02dacd7f -> 7cbeb4e578f58140
  connections 120 40 paper mono 7f3b6302cbe85759 -> 4582c48515c2d96b
  connections 120 40 paper truecolor 4738175f731c3f9a -> fe45e0ab34788579
  connections 80 24 junie mono c1d956c283dc6577 -> b173423d1aa11875
  connections 80 24 junie truecolor a0a87160573711e3 -> e5785859acdacb32
  connections 80 24 paper mono 1b06fd69c3c6118f -> 28dd678bde97e01b
  connections 80 24 paper truecolor ffe9f9616005be8f -> 419266cfc92cbf13
  connections-failed 120 40 junie mono 6f915cbb8b64dd01 -> 7282af42b6118d4b
  connections-failed 120 40 junie truecolor 5caee999f1f8c455 -> 85751834590c565f
  connections-failed 120 40 paper mono 85c4713f0c6a9085 -> fb16c8f370c0ed33
  connections-failed 120 40 paper truecolor 0b5ee042f5beca3d -> 19c65378e9fea62e
  connections-failed 80 24 junie mono 3df0aa54ae5de7ff -> 817efd149bf23887
  connections-failed 80 24 junie truecolor 089838277b5b9ecd -> 582d36ab66312a0e
  connections-failed 80 24 paper mono 41fa0ddd9da7ac53 -> b79296eb801dfc73
  connections-failed 80 24 paper truecolor 27ea7dcf6513bbe4 -> 54c900eab2dc4097
  error-result 120 40 junie mono 24e9e8da2210fbe4 -> 3d212aedb87d7397
  error-result 120 40 junie truecolor 9bb8a8d61e765e3e -> 62a9c466ce8ab494
  error-result 120 40 paper mono dbd0485d6e64ab14 -> fb9ec2f9d07b127d
  error-result 120 40 paper truecolor 0523f37543238c5e -> d9f891a3a73b90c2
  error-result 80 24 junie mono cedfca53372948de -> 0fd30ddc57adfa6d
  error-result 80 24 junie truecolor 59bdbaf21a7cfbd0 -> f51ed147a9cbda89
  error-result 80 24 paper mono 10215f0153663ece -> 85c55e8fa7bd2aef
  error-result 80 24 paper truecolor bbe0888507ffa6cb -> d81aa1ae979064b0
  explain-plan 120 40 junie mono 0b6354fccdd53f63 -> 02f4b03d86804397
  explain-plan 120 40 junie truecolor 26e3af8dfeed400b -> 32dbc4b59a0099c4
  explain-plan 120 40 paper mono fb3fb3d8b3f9910f -> 47a4967c83892b3f
  explain-plan 120 40 paper truecolor 34b43520633931af -> a85ad303fc0fd314
  explain-plan 80 24 junie mono 75761b3c6e3b4781 -> 87ccd8d13f5b57d7
  explain-plan 80 24 junie truecolor e8b95bd2d750ad3f -> b5642e0c5d45b159
  explain-plan 80 24 paper mono 7064f670562e7f05 -> 8bf3dcdcdaf406cd
  explain-plan 80 24 paper truecolor f3bba3a470d5d0d6 -> 2b8a7b9510285314
  explorer-focused 120 40 junie mono 7f656cf3fe2953f9 -> f8142a433861fed6
  explorer-focused 120 40 junie truecolor 68598d0eb5203a31 -> b02e5c6a233666d3
  explorer-focused 120 40 paper mono 17819e4df8dffb65 -> 232e36a0ca9e924e
  explorer-focused 120 40 paper truecolor 329ea03fa79df4dd -> ad1a962f72a919d7
  explorer-focused 80 24 junie mono 8fdcb2b5647e0467 -> 62ece815ae8422f1
  explorer-focused 80 24 junie truecolor 11ad01e0d9e19869 -> 00c8014e2ecf7ca7
  explorer-focused 80 24 paper mono 4a8bbd546f5b0733 -> 5fca4d19c49da309
  explorer-focused 80 24 paper truecolor eb2c3da51dd19a84 -> 06fa2b86fc333844
  filter-editor 120 40 junie mono 27cc45d78aa3a506 -> 02f76da22c28dec3
  filter-editor 120 40 junie truecolor 12851a1aaa78b9d2 -> 1999866dd78d581b
  filter-editor 120 40 paper mono 47cf7f9e7ff2af32 -> 199f2ae2197a9c0b
  filter-editor 120 40 paper truecolor 1aa092b43d859cd5 -> e97c0db81aadcb19
  filter-editor 80 24 junie mono 2ecbfbd40803bc00 -> cd991e8e60459b42
  filter-editor 80 24 junie truecolor a53ad97e61d4b9c4 -> 6fd27280f2e1cc2f
  filter-editor 80 24 paper mono 3466ac455d3f547c -> 4018c1fa7e05bcd2
  filter-editor 80 24 paper truecolor 74c571314cddc60c -> 70df5ca40efff65a
  grid-cell-editing 120 40 junie mono 1f33aa000f68f36c -> b47b03146355f46d
  grid-cell-editing 120 40 junie truecolor 9251cadadd462d8c -> b5031cc098fe0717
  grid-cell-editing 120 40 paper mono 46a581506e138c70 -> 199f2ae2197a9c0b
  grid-cell-editing 120 40 paper truecolor 3590a14ce0d1cbc7 -> 1f46c5e955708c77
  grid-cell-editing 80 24 junie mono 83eb22119e071d76 -> 6bcae9079dfd0f32
  grid-cell-editing 80 24 junie truecolor 09b3a2abbc053a2a -> 36a704942904794f
  grid-cell-editing 80 24 paper mono be56a95247d74d72 -> 4018c1fa7e05bcd2
  grid-cell-editing 80 24 paper truecolor ce49b8146211205e -> 04eafedcd6292eac
  help-dialog 120 40 junie mono 8e8057dbcb5ed746 -> 3a817591caeee4f6
  help-dialog 120 40 junie truecolor bd74778f16b4a18e -> a2a9ab28690997f7
  help-dialog 120 40 paper mono 1b92ad776b10c512 -> 3e714099386c0e5a
  help-dialog 120 40 paper truecolor 2a8f89b12264f8d1 -> e41be54cb130e167
  help-dialog 80 24 junie mono 765af3ddb68f4a40 -> 472e00b8e1954bcd
  help-dialog 80 24 junie truecolor 0981e73a21e0f300 -> 8475c62dfe31a31b
  help-dialog 80 24 paper mono 54b242de475b2a5c -> 5fa1620ef01f9e2d
  help-dialog 80 24 paper truecolor 7c6a35d64935f8c0 -> 5d120f13305884a0
  history-tab 120 40 junie mono cae0ab669f7bdd1a -> af7882c38db68349
  history-tab 120 40 junie truecolor 8af99e0a7e7d4232 -> b5e3bbc79d613d52
  history-tab 120 40 paper mono 9139b74572dd517e -> f8a3dae1c921048b
  history-tab 120 40 paper truecolor dc0cbfbfae6cf701 -> e28d3f48d5ffd99e
  history-tab 80 24 junie mono cf265cf750410bec -> 4a9b89e6b2f349e5
  history-tab 80 24 junie truecolor 3a3dced6d5583ba4 -> c83eb6f5b2b5266f
  history-tab 80 24 paper mono 18816e127a5eba40 -> 4b85b62898343115
  history-tab 80 24 paper truecolor 3a4a0f1e95194990 -> 240a4ee7c858d3dc
  maximised-tab 120 40 junie mono 0f38d927cf5d4331 -> 3a9dfc2ae3bade2d
  maximised-tab 120 40 junie truecolor c6aeea3b116e993b -> e4ed181b31ef0bdf
  maximised-tab 120 40 paper mono 1d7a5c48729648f1 -> 11ffcb29e0276ecd
  maximised-tab 120 40 paper truecolor 25f57c8619281482 -> 14437d9f9266cc96
  maximised-tab 80 24 junie mono c45a85012264ff0f -> 7361de83abbe9db5
  maximised-tab 80 24 junie truecolor 99600272ed17bf8f -> a741cb596a696819
  maximised-tab 80 24 paper mono 87d5d662553d5d77 -> 4d534130c71ec879
  maximised-tab 80 24 paper truecolor 7ddfd721260f6d27 -> 63896c97531bd407
  pending-change-bar 120 40 junie mono 33728302cd4710ac -> b47b03146355f46d
  pending-change-bar 120 40 junie truecolor e631f4fcecbc3482 -> b5031cc098fe0717
  pending-change-bar 120 40 paper mono f42140e257c2180c -> 199f2ae2197a9c0b
  pending-change-bar 120 40 paper truecolor 76e89bc1af1a7f9e -> 1f46c5e955708c77
  pending-change-bar 80 24 junie mono 95262e26aa3188b6 -> 6bcae9079dfd0f32
  pending-change-bar 80 24 junie truecolor b3e5f0bba602cdd4 -> 36a704942904794f
  pending-change-bar 80 24 paper mono 96d3aa9a522fb416 -> 4018c1fa7e05bcd2
  pending-change-bar 80 24 paper truecolor 8721494d43cfe70b -> 04eafedcd6292eac
  query-editing 120 40 junie mono 520cccb881ee39ff -> 0e182eb2c9cdf0d1
  query-editing 120 40 junie truecolor ed2d5bddc30c518d -> 26ae40f40ffcb06c
  query-editing 120 40 paper mono 117e6b53275b60c7 -> 8c123136b9baab67
  query-editing 120 40 paper truecolor 03d2f5dbdcf03ba8 -> 4cdbbecf4399aaaa
  query-editing 80 24 junie mono f1687dad8c1c39d5 -> 3fa506527dadafdb
  query-editing 80 24 junie truecolor 5ebfffce412dc3e5 -> 763a678455e83e55
  query-editing 80 24 paper mono 6ed042f21948526d -> f598d9e8d32d4a81
  query-editing 80 24 paper truecolor 78fd4ffdf74c64fd -> 8da490e0cfdd3be4
  quick-switcher 120 40 junie mono 2e1e1c782fad9753 -> 2331637495ef7512
  quick-switcher 120 40 junie truecolor 930a8aaf48e36d1f -> a908891a0cdf47ca
  quick-switcher 120 40 paper mono fbc0c8d918c4328f -> fa48bda736d5a401
  quick-switcher 120 40 paper truecolor 8aad9531b6d1e78d -> 3b9daf8e3b206655
  quick-switcher 80 24 junie mono d36eca6741ca9a51 -> 43ca600a571c24e9
  quick-switcher 80 24 junie truecolor 203abc2ae0189783 -> 9df461fe7b23a995
  quick-switcher 80 24 paper mono e929da4ced731285 -> f2a395e248d208f7
  quick-switcher 80 24 paper truecolor 738324e236dfd614 -> f400e1bee21514cd
  results-grid 120 40 junie mono baf2209b245b8f35 -> 13be5e88047fff1e
  results-grid 120 40 junie truecolor e0bce08e8e374d6d -> f948ae3eafb4dfb4
  results-grid 120 40 paper mono 41aa19495f561989 -> 10e5c0ec05b86e78
  results-grid 120 40 paper truecolor dc2569482483a81f -> 72efe86e072cdd5a
  results-grid 80 24 junie mono 68a5a24b16c82abb -> ab10a72fa6631d28
  results-grid 80 24 junie truecolor deee9021a7ef5fc5 -> e5c6b089992fed1a
  results-grid 80 24 paper mono df9d33608b3309df -> c2c27002d4b72cfa
  results-grid 80 24 paper truecolor aed3a34518b4a426 -> c5d85d23733d552c
  safe-mode-picker 120 40 junie mono 8d0f0c6fdea90314 -> 2f42775134b39ade
  safe-mode-picker 120 40 junie truecolor 648308fb64003c7e -> 22c85057bb9098df
  safe-mode-picker 120 40 paper mono c1ddede4880dc39c -> 5f341251e6c8fb0e
  safe-mode-picker 120 40 paper truecolor e3efe3fb43224b22 -> da99889654375359
  safe-mode-picker 80 24 junie mono 708e98e63b0f8dee -> 434e6360711d3062
  safe-mode-picker 80 24 junie truecolor 69bb02b2aa8b1790 -> 0261508477206928
  safe-mode-picker 80 24 paper mono 24c3f32cca555f06 -> c7745cb61a98d0b2
  safe-mode-picker 80 24 paper truecolor a319e4984311f847 -> 18f3de2ae1b4234d
  safety-dialog-typed-ack 120 40 junie mono 5220bccaec87f062 -> b01fba8abc3abf96
  safety-dialog-typed-ack 120 40 junie truecolor 850c12405d5fceea -> 7f78c8806be79519
  safety-dialog-typed-ack 120 40 paper mono 21f2911e3d1c6476 -> 0668188383300b1c
  safety-dialog-typed-ack 120 40 paper truecolor b518ac98a20a3dd3 -> d2a52a70fe5c7f87
  safety-dialog-typed-ack 80 24 junie mono 344df72d9f626a24 -> cad6c5e3fe9a4450
  safety-dialog-typed-ack 80 24 junie truecolor 56cdc51014f403dc -> 9dcd27db40af236e
  safety-dialog-typed-ack 80 24 paper mono 4ea02a2a06896748 -> a8e454bb9b81019a
  safety-dialog-typed-ack 80 24 paper truecolor f8706ba09a7beb82 -> f77f0d4779d082db
  structure-view 120 40 junie mono 64b968a7efc4134f -> 598b8c530d5e1fe3
  structure-view 120 40 junie truecolor e0c3358c94523dfb -> 87e3f89bb155cccc
  structure-view 120 40 paper mono 0aa87366e545b20b -> 831fc500c1764aef
  structure-view 120 40 paper truecolor 40f8be6a6ee68bb1 -> 5b045802e8ecb576
  structure-view 80 24 junie mono 37617140675a7e45 -> 1fc55956f293f667
  structure-view 80 24 junie truecolor 5222c1f4be457ecf -> ac9c285e4e6f5bf4
  structure-view 80 24 paper mono 8353c8f2c650f479 -> eb689470569daacb
  structure-view 80 24 paper truecolor 67e27daab116e620 -> a24d479365b4bf2f
  tab-list-picker 120 40 junie mono 0d5aa84b6066a5ff -> f979d0692d40604c
  tab-list-picker 120 40 junie truecolor e3edb355b6e57c01 -> 0cf8339853597c9c
  tab-list-picker 120 40 paper mono dd99d08198af9647 -> 02e20a98d50eef82
  tab-list-picker 120 40 paper truecolor 6a9644f95349a23c -> f23264a67dd53d3c
  tab-list-picker 80 24 junie mono ee17605d1da7e5d5 -> 7b0107d5135412f2
  tab-list-picker 80 24 junie truecolor 3988da764df1c599 -> 8df3e0b341cc4fcd
  tab-list-picker 80 24 paper mono 80bb61dfd812bbed -> 3558828f55441770
  tab-list-picker 80 24 paper truecolor fdff544292391b79 -> c19dff7150078fe3
  table-grid 120 40 junie mono 28848cd29e81a145 -> 02f76da22c28dec3
  table-grid 120 40 junie truecolor df9b192dedfb5449 -> 1999866dd78d581b
  table-grid 120 40 paper mono 43434154c9b79771 -> 199f2ae2197a9c0b
  table-grid 120 40 paper truecolor 07f406c097a249ab -> e97c0db81aadcb19
  table-grid 80 24 junie mono 4dddcd80f594cb6b -> cd991e8e60459b42
  table-grid 80 24 junie truecolor e1c80431ed646941 -> 6fd27280f2e1cc2f
  table-grid 80 24 paper mono 69f9856a1b59cdf7 -> 4018c1fa7e05bcd2
  table-grid 80 24 paper truecolor ac6c26802bcdc60a -> 70df5ca40efff65a
  workbench-default 120 40 junie mono f72f183fe4908e71 -> 3a817591caeee4f6
  workbench-default 120 40 junie truecolor e7d679efa83fe16b -> a2a9ab28690997f7
  workbench-default 120 40 paper mono c53e82aaef027839 -> 3e714099386c0e5a
  workbench-default 120 40 paper truecolor 42cb8adca93471dc -> e41be54cb130e167
  workbench-default 80 24 junie mono 537e3ed64aa4034f -> 472e00b8e1954bcd
  workbench-default 80 24 junie truecolor d31a7970b9c59f1f -> 8475c62dfe31a31b
  workbench-default 80 24 paper mono 90997a64bb9fbfef -> 5fa1620ef01f9e2d
  workbench-default 80 24 paper truecolor cb3448c40a05f099 -> 5d120f13305884a0
- added:     none
- class:     intended
- reason:    §20.10 item 37. The movement records the merged shell's reviewed rendering
             after two divergent lines of deliberate corrections were unified; no
             surface-specific drift remains outside the union.
```

## Item 38 — Jackin scenario matrix re-blessed over the recipe corrections

The eight-scenario Jackin preview app is unchanged through the merge
(`Scenario::ALL`, paused fixtures); the recorded matrix predated the
crates/tui recipe corrections the merge carries (container-part badge recipe,
authored capability palettes, item 35's empty-part inheritance). All 64
scenario cells were re-blessed against the current app and all 64 moved:
the truecolor and mono cells alike carry the recipe corrections (the authored
mono palette is one of them), so both halves are claimed here.

```
- surface:   jackin digest @ 8 scenarios / 100x30 + 120x40 / junie + paper / truecolor
- captures:  apps/jackin-preview/tests/visual.rs::jackin_visual_baseline green; refreshed
             capture matrix and independent review follow the integration freeze
- tests:     apps/jackin-preview/tests/visual.rs, the jackin unit suites (192 passed),
             editor-save workspace-payload binding, chrome navigation test
- moved:     64 keys (rewritten to the merged diff):
  accounts-mixed 100 30 junie truecolor 41159786d21c091a -> 180c748201bc6b11
  accounts-mixed 100 30 paper truecolor cea4b08306445dd5 -> 5b1ecca142dfbe07
  accounts-mixed 120 40 junie truecolor ef399d1251dc0dd8 -> 0c4d5c018c06efc3
  accounts-mixed 120 40 paper truecolor d50e6eca5192b4be -> 7182a4744a345e0f
  capsule-multi 100 30 junie truecolor 67648cad2d2fcccf -> bb8d41b2ebf4061a
  capsule-multi 100 30 paper truecolor ac5b4f1cabb10969 -> cafe562af8058787
  capsule-multi 120 40 junie truecolor 6be7027a0c42675b -> 0429b5c7a0396fd0
  capsule-multi 120 40 paper truecolor af1ebfe02fb05ad5 -> 557b649604c72579
  first-use 100 30 junie truecolor e3eb9740cbbf8393 -> 7b6d614c5626c72d
  first-use 100 30 paper truecolor 1ead52401712fd79 -> cd2d6be4174d46ef
  first-use 120 40 junie truecolor e80ce1a927ba5a83 -> 77caa7098884a649
  first-use 120 40 paper truecolor e2f1394ba9602db1 -> afef3f689442021b
  hard-cases 100 30 junie truecolor dcd9a1c0dacf656e -> 26b91547137cff79
  hard-cases 100 30 paper truecolor ab560e8791da02ba -> 2754442543a2629c
  hard-cases 120 40 junie truecolor 09bd7830798104b6 -> aeaf5dc3e365eb3b
  hard-cases 120 40 paper truecolor fa7f2c00b027fb82 -> f4c12681e8a961c1
  launch-failure 100 30 junie truecolor 9923629b677277bc -> 476dfcf432b70fd1
  launch-failure 100 30 paper truecolor ea547ceb73f6a19c -> 577b4d0c573d03d5
  launch-failure 120 40 junie truecolor 333b4aaf9b7384cc -> d8265fa7796854bd
  launch-failure 120 40 paper truecolor 57ef93c258526aac -> b11a3beb1cab1fe1
  launch-running 100 30 junie truecolor 1d5ac647e3178a15 -> 82ca41a068374bd4
  launch-running 100 30 paper truecolor f26a587efc7c9597 -> 642a7ca2a8d08f36
  launch-running 120 40 junie truecolor 2f973120ddd2c425 -> 7be756152db292e8
  launch-running 120 40 paper truecolor e538a12468b2a4cf -> e988b1dddcd79c4a
  outro-last 100 30 junie truecolor 8bc54654cbde1d1b -> e08f270ea0550d73
  outro-last 100 30 paper truecolor 9ec82e294983c007 -> 131e91991fc6872d
  outro-last 120 40 junie truecolor 80500c2316d44537 -> 4d63825cfdd97fbd
  outro-last 120 40 paper truecolor c4b38af6a33c868b -> 8d1d3d1b901b862e
  returning 100 30 junie truecolor 3259ff84d494d2c9 -> db2a797c45aa4acf
  returning 100 30 paper truecolor 2f1302505e9a403b -> 72b7727c3e7d446b
  returning 120 40 junie truecolor 60821d9a0b7821b9 -> b3215ee90849c90b
  returning 120 40 paper truecolor 1a9122ef93ff6dc3 -> d0d60d05ee0bfd73
  accounts-mixed 100 30 junie mono 55d2f0ce29ffa295 -> 4006540866e69c3d
  accounts-mixed 100 30 paper mono d5d77ca6d9ec3ed3 -> 1749ab020292e5e7
  accounts-mixed 120 40 junie mono 2f00ee6433758b81 -> 52026180b8fda6ef
  accounts-mixed 120 40 paper mono cd407f23672b2de3 -> fb18df620bd16b77
  capsule-multi 100 30 junie mono 55f22589906bc6bc -> 4bfb2b876f426435
  capsule-multi 100 30 paper mono fc56004f1b2ca694 -> 18e5f27890d81613
  capsule-multi 120 40 junie mono f10617f2f930d294 -> 0867a599615c686e
  capsule-multi 120 40 paper mono 92ec27530b587e6c -> aa20c2aa7970668e
  first-use 100 30 junie mono 172a7c79991d68ea -> 1ce61ee685505f05
  first-use 100 30 paper mono 4d07026fb091dc50 -> 0cba6ca32a79f7a3
  first-use 120 40 junie mono df2f67f9b55fc0aa -> 5b62264dd69d73c5
  first-use 120 40 paper mono 2969d6cfcb8e0010 -> d3bc10909d9674f3
  hard-cases 100 30 junie mono d25a8ad366603ee5 -> 976e0ac2ff1064c7
  hard-cases 100 30 paper mono fc4a1e8993409141 -> e346de1bfe923201
  hard-cases 120 40 junie mono 1f5132855e8675e5 -> a0183190d8abe137
  hard-cases 120 40 paper mono fd555dbd0e54ab21 -> 1cce38b247c13be9
  launch-failure 100 30 junie mono 67d14e4d81525443 -> a9d496f31fe32a7c
  launch-failure 100 30 paper mono e73c9d3b4059cfb7 -> 0a9cec1d4e1bb6fe
  launch-failure 120 40 junie mono 29ff79bf7686a7b3 -> ad6bcc6f22c76d74
  launch-failure 120 40 paper mono 0e248b82d15a2147 -> b99bcb7c9205b25e
  launch-running 100 30 junie mono e97ae00e5cdc34fa -> 887780d4b354419b
  launch-running 100 30 paper mono 7bcf2d5b715b7dce -> 639611829bf92c93
  launch-running 120 40 junie mono d4573861aa433e5a -> 177ffee83b95067b
  launch-running 120 40 paper mono bac68c89bd1f00ae -> ac80a82230b2f203
  outro-last 100 30 junie mono afa77b3b5574e591 -> e1afb4d167a530f5
  outro-last 100 30 paper mono 78c093d75b09aae1 -> 444a96716922ce7d
  outro-last 120 40 junie mono 478f9b69559ec209 -> ef363f4fda2bc0ea
  outro-last 120 40 paper mono b4ea440adc5601a9 -> 55856c59724258ea
  returning 100 30 junie mono 12003531bbbb9495 -> 6db6cdf7f3353878
  returning 100 30 paper mono b978438008d7b751 -> ba3d0bca168e59d2
  returning 120 40 junie mono cf87d8181c54f435 -> 78a2b61980bc2587
  returning 120 40 paper mono 83c937554910f031 -> 88d630b9c82893bd
- added:     none
- class:     intended
- reason:    §20.10 item 38. The scenario contracts, rain timing constants and duration
             wording are untouched (they remain regressions under the closing clause);
             only the styling corrections flowed into the recorded cells.
```


## Item 39 — crates/tui digests re-blessed to the merged semantic-paint pipeline

The merged component rendering is the union of two reviewed lines: the branch's
semantic-paint corrections (items 22–31 and 35: semantic selection, owned glyphs
and dialog targets, forced-state readiness, ChipBar identity, ScrollRegion/NavList/
Steps/Grid first generations, DiffView/CodeEditor digests, empty-region inheritance
and hover isolation) and the main line's historical component restoration (§7d).
The recorded crates/tui digests predate one side of that union, so they are
re-blessed once over it. Exactly 594 retained component keys move (54 truecolor
and 540 mono/ANSI16/ANSI256); the per-family attributions and their
frame-text guarantees remain in the items cited above, and this entry pins the
complete merged-diff pair set so the bless cannot hide an unreviewed movement.

```
- surface:   crates/tui render_components digest @ full component/state matrix / all sizes / junie + paper / truecolor + 256 + 16 + mono
- captures:  none under shots/ — component digests are pinned by `render_components`; the capture-matrix refresh and independent review cover the app surfaces built on them
- tests:     crates/tui/tests/render_components.rs
- moved:     594 keys (rewritten to the merged diff):
  render::components::brand::default 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::default 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::brand::disabled 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::disabled 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::brand::editing 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::editing 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::brand::empty 120 40 junie mono 82c24983883c241d -> 6180f2b0899d86cf
  render::components::brand::empty 40 10 junie mono 8e64b3bf489db79d -> 8a2989a1dc02eacf
  render::components::brand::focused 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::focused 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::brand::hovered 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::hovered 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::brand::pressed 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::pressed 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::brand::selected 120 40 junie mono 7ebcdaa2bd323a7c -> ef1cd1e975f4611e
  render::components::brand::selected 40 10 junie mono 6b84e3d4400315fc -> 736e38877d74651e
  render::components::button::default 120 40 junie mono 1aafcb632b64bb09 -> ae1695116d2a7d79
  render::components::button::default 40 10 junie mono 8eb20d2cb7769089 -> f4bcd09e0eaf3af9
  render::components::button::disabled 120 40 junie mono d20bb906fcfe3dd1 -> ee7652c0ccb0707d
  render::components::button::disabled 40 10 junie mono bfe7ea91b76bd751 -> 81111d3ce74b53fd
  render::components::button::editing 120 40 junie mono 1aafcb632b64bb09 -> ae1695116d2a7d79
  render::components::button::editing 40 10 junie mono 8eb20d2cb7769089 -> f4bcd09e0eaf3af9
  render::components::button::empty 120 40 junie mono 82c24983883c241d -> 6180f2b0899d86cf
  render::components::button::empty 40 10 junie mono 8e64b3bf489db79d -> 8a2989a1dc02eacf
  render::components::button::focused 120 40 junie mono c371306941cd65c9 -> 9e7102d1b39bd339
  render::components::button::focused 40 10 junie mono 7fddb1a683d31b49 -> cbfa477a6b9270b9
  render::components::button::hovered 120 40 junie mono f9e4755e0f6594d1 -> 82271690d4c182e1
  render::components::button::hovered 40 10 junie mono 66ae716a48b9ae51 -> 77c74c5c41cfe461
  render::components::button::selected 120 40 junie mono 13d0bae25b64bacc -> 96b4d865e2982360
  render::components::button::selected 40 10 junie mono 0f80b7f1cc3b9e4c -> 0cfe2af0722385e0
  render::components::checkbox::disabled 120 40 junie mono 37d191376a12b670 -> 7d452f7dbb668d28
  render::components::checkbox::disabled 40 10 junie mono fd1f3dba93662770 -> 12adba20cd1e4228
  render::components::checkbox::focused 120 40 junie mono 11cfb5b4cfea4f34 -> 3a4fae1bda479930
  render::components::checkbox::focused 40 10 junie mono 0cbb124b11f14f54 -> c44869a5cc459850
  render::components::checkbox::pressed 120 40 junie mono 88fe1cc884671642 -> ad0ecd886b56bcb4
  render::components::checkbox::pressed 40 10 junie mono 876ca96abc19e3e2 -> d5590944b23f0ad4
  render::components::checkbox::selected 120 40 junie mono e9095edf7388b823 -> 2d890e8a19124333
  render::components::checkbox::selected 40 10 junie mono 546279dc45e69d83 -> ae1992f4f5c5f513
  render::components::chip_bar::default 120 40 junie mono d410e30a8fa1287b -> e649e48c2d94a7d7
  render::components::chip_bar::default 40 10 junie mono fff5116446505e3c -> 3f889ea34564264c
  render::components::chip_bar::disabled 120 40 junie mono fe16ace9bdc28c17 -> 1a9038d7d9da10dd
  render::components::chip_bar::disabled 40 10 junie mono 83ba4197f209a1b8 -> f294ad05a2d2de2e
  render::components::chip_bar::editing 120 40 junie mono d410e30a8fa1287b -> e649e48c2d94a7d7
  render::components::chip_bar::editing 40 10 junie mono fff5116446505e3c -> 3f889ea34564264c
  render::components::chip_bar::focused 120 40 junie mono d410e30a8fa1287b -> e649e48c2d94a7d7
  render::components::chip_bar::focused 40 10 junie mono fff5116446505e3c -> 3f889ea34564264c
  render::components::chip_bar::hovered 120 40 junie mono d410e30a8fa1287b -> e649e48c2d94a7d7
  render::components::chip_bar::hovered 40 10 junie mono fff5116446505e3c -> 3f889ea34564264c
  render::components::chip_bar::pressed 120 40 junie mono 3ec2f185b7fc82ab -> b2ebcdfee908d6af
  render::components::chip_bar::pressed 40 10 junie mono 465c9be2eaf8153a -> f7276ed25e2ffb8c
  render::components::chip_bar::selected 120 40 junie mono d03067a49b2395be -> cb6db8076587b494
  render::components::chip_bar::selected 40 10 junie mono 6926f807c9ab6fed -> 483bc1aec7c86357
  render::components::code_editor::empty 120 40 junie mono 115797b05201b814 -> 79fb383578b7a530
  render::components::code_editor::empty 40 10 junie mono f87d3014da9de654 -> 230313dec8e74d70
  render::components::code_editor::focused 120 40 junie mono 760af746a9592068 -> b50a48e58dc9ccfe
  render::components::code_editor::focused 40 10 junie mono acd4bcbc22de1f68 -> 6f6b6a57b2282f3e
  render::components::code_editor::pressed 120 40 junie mono d9c2954165ae59b8 -> 802d6aaa5d87e21e
  render::components::code_editor::pressed 40 10 junie mono 28b0a2f529f31198 -> 0212054a0abdfc7e
  render::components::completion::default 120 40 junie mono 2c52b2cb646f5dd5 -> 882afd2760c7edb1
  render::components::completion::default 40 10 junie mono 41fdaff714ab0375 -> a601e598d07dbcc1
  render::components::completion::disabled 120 40 junie mono a8d9f96587b2426b -> 67e8a43e66d8ecf1
  render::components::completion::disabled 40 10 junie mono 382ab6944939137b -> 2798668fdb65ef61
  render::components::completion::editing 120 40 junie mono 2c52b2cb646f5dd5 -> 882afd2760c7edb1
  render::components::completion::editing 40 10 junie mono 41fdaff714ab0375 -> a601e598d07dbcc1
  render::components::completion::empty 120 40 junie mono 748ad16e5a9201ab -> 820072608f6c435d
  render::components::completion::empty 40 10 junie mono 786258fd876255db -> deb6005648d9212d
  render::components::completion::focused 120 40 junie mono 2c52b2cb646f5dd5 -> 882afd2760c7edb1
  render::components::completion::focused 40 10 junie mono 41fdaff714ab0375 -> a601e598d07dbcc1
  render::components::completion::hovered 120 40 junie mono 2c52b2cb646f5dd5 -> 882afd2760c7edb1
  render::components::completion::hovered 40 10 junie mono 41fdaff714ab0375 -> a601e598d07dbcc1
  render::components::completion::pressed 120 40 junie mono ebe498e8ebdbda13 -> f7d8e7f5aae6bb73
  render::components::completion::pressed 40 10 junie mono 46c0a4bc5841cec3 -> ce2f2109c502ffa3
  render::components::completion::selected 120 40 junie mono 2c52b2cb646f5dd5 -> 882afd2760c7edb1
  render::components::completion::selected 40 10 junie mono 41fdaff714ab0375 -> a601e598d07dbcc1
  render::components::context_menu::default 120 40 junie mono e6c6987845be0a57 -> 998d1db79f0769bb
  render::components::context_menu::default 40 10 junie mono f37c624d906d3b07 -> 3dc60ff9df0ec58b
  render::components::context_menu::disabled 120 40 junie mono 61deb5fd4c10a3f5 -> fb91d7290bf1a9a5
  render::components::context_menu::disabled 40 10 junie mono 31e14e63d8691f55 -> 1e57518db66a8495
  render::components::context_menu::editing 120 40 junie mono e6c6987845be0a57 -> 998d1db79f0769bb
  render::components::context_menu::editing 40 10 junie mono f37c624d906d3b07 -> 3dc60ff9df0ec58b
  render::components::context_menu::empty 120 40 junie mono 4e94a268eb16abc5 -> 1a886f55e2dea46f
  render::components::context_menu::empty 40 10 junie mono 87bc2ad1e2b94765 -> a8a3f5834061619f
  render::components::context_menu::focused 120 40 junie mono cdf6a75218252d3b -> f222d056d3ed9dd5
  render::components::context_menu::focused 40 10 junie mono 3b0d77754e60716b -> bbfbb4c6d279c725
  render::components::context_menu::hovered 120 40 junie mono e6c6987845be0a57 -> 998d1db79f0769bb
  render::components::context_menu::hovered 40 10 junie mono f37c624d906d3b07 -> 3dc60ff9df0ec58b
  render::components::context_menu::pressed 120 40 junie mono 4eaf78e4938a6f01 -> 35fab4a1cfca73f9
  render::components::context_menu::pressed 40 10 junie mono 7538bb142cdd8b61 -> 58f38a7093f5fdd9
  render::components::context_menu::selected 120 40 junie mono e6c6987845be0a57 -> 998d1db79f0769bb
  render::components::context_menu::selected 40 10 junie mono f37c624d906d3b07 -> 3dc60ff9df0ec58b
  render::components::dialog::default 120 40 junie mono b737d61b383ec6f5 -> 576714b0c37c1eff
  render::components::dialog::default 40 10 junie mono 081728ded55d1575 -> 5b8d9fce2ae21747
  render::components::dialog::disabled 120 40 junie mono b737d61b383ec6f5 -> 576714b0c37c1eff
  render::components::dialog::disabled 40 10 junie mono 081728ded55d1575 -> 5b8d9fce2ae21747
  render::components::dialog::editing 120 40 junie mono b737d61b383ec6f5 -> 576714b0c37c1eff
  render::components::dialog::editing 40 10 junie mono 081728ded55d1575 -> 5b8d9fce2ae21747
  render::components::dialog::focused 120 40 junie mono d057ba49733734b3 -> a12e35d7f3be32c3
  render::components::dialog::focused 40 10 junie mono 4dfe262b3ac49703 -> 7a1b32a1cfb75e5b
  render::components::dialog::hovered 120 40 junie mono b737d61b383ec6f5 -> 24a1491e71bd03a3
  render::components::dialog::hovered 40 10 junie mono 081728ded55d1575 -> de2539c7def07b4b
  render::components::dialog::pressed 120 40 junie mono 3f5dbd922edd1637 -> 8ef17228c7bff7eb
  render::components::dialog::pressed 40 10 junie mono 9c5b0be834361ad7 -> bda5a78ad988c6b3
  render::components::dialog::selected 120 40 junie mono b737d61b383ec6f5 -> 576714b0c37c1eff
  render::components::dialog::selected 40 10 junie mono 081728ded55d1575 -> 5b8d9fce2ae21747
  render::components::diff_view::default 120 40 junie mono 399e8e1ab5f8f8ef -> 48b4d0fc32d2706b
  render::components::diff_view::default 40 10 junie mono 5db767855e9b1063 -> fbedddb070982393
  render::components::diff_view::disabled 120 40 junie mono 399e8e1ab5f8f8ef -> 48b4d0fc32d2706b
  render::components::diff_view::disabled 40 10 junie mono 5db767855e9b1063 -> fbedddb070982393
  render::components::diff_view::editing 120 40 junie mono 399e8e1ab5f8f8ef -> 48b4d0fc32d2706b
  render::components::diff_view::editing 40 10 junie mono 5db767855e9b1063 -> fbedddb070982393
  render::components::diff_view::empty 120 40 junie mono 0420ba2db61ff883 -> cff1daf31c6442a9
  render::components::diff_view::empty 40 10 junie mono fae3b2396e621d07 -> 4e9eb870f60f0881
  render::components::diff_view::focused 120 40 junie mono b5489bd20a306647 -> 5896e6210741e0cf
  render::components::diff_view::focused 40 10 junie mono 8e55901518cf1bc7 -> c5ccfb8c6378950f
  render::components::diff_view::hovered 120 40 junie mono 399e8e1ab5f8f8ef -> 48b4d0fc32d2706b
  render::components::diff_view::hovered 40 10 junie mono 5db767855e9b1063 -> fbedddb070982393
  render::components::diff_view::pressed 120 40 junie mono fbc31eba29382a14 -> b01b383c64e53a9e
  render::components::diff_view::pressed 40 10 junie mono c0d7c73381ea2af4 -> 785d29437cf99156
  render::components::diff_view::selected 120 40 junie mono 399e8e1ab5f8f8ef -> 48b4d0fc32d2706b
  render::components::diff_view::selected 40 10 junie mono 5db767855e9b1063 -> fbedddb070982393
  render::components::empty::default 120 40 junie mono cddba26acfed672d -> 43032bdcc787a177
  render::components::empty::default 40 10 junie mono d6241ec8730e39ad -> b047517df5851b77
  render::components::empty::disabled 120 40 junie mono 527c9121b0b6bf8b -> b18b8dcc860c7021
  render::components::empty::disabled 40 10 junie mono 7a64644b19b8438b -> c627efa1b3f323a1
  render::components::empty::editing 120 40 junie mono 796461203dd06a7e -> 534ba50f6badf4ec
  render::components::empty::editing 40 10 junie mono b0a749381c6792fe -> 740e0de8851c8b6c
  render::components::empty::focused 120 40 junie mono cddba26acfed672d -> 43032bdcc787a177
  render::components::empty::focused 40 10 junie mono d6241ec8730e39ad -> b047517df5851b77
  render::components::empty::hovered 120 40 junie mono cddba26acfed672d -> 43032bdcc787a177
  render::components::empty::hovered 40 10 junie mono d6241ec8730e39ad -> b047517df5851b77
  render::components::empty::pressed 120 40 junie mono cddba26acfed672d -> 43032bdcc787a177
  render::components::empty::pressed 40 10 junie mono d6241ec8730e39ad -> b047517df5851b77
  render::components::empty::selected 120 40 junie mono cddba26acfed672d -> 43032bdcc787a177
  render::components::empty::selected 40 10 junie mono d6241ec8730e39ad -> b047517df5851b77
  render::components::field::default 120 40 junie mono df6c39650b0ade4c -> bbfada53d625d404
  render::components::field::default 40 10 junie mono 623ecdc1e6dcb26c -> c379e0d03ce5b5a4
  render::components::field::disabled 120 40 junie mono 6ea61957d4835dac -> 7c03b1f135f5eb54
  render::components::field::disabled 40 10 junie mono 8a8e49368586582c -> c7a41948ef3b26d4
  render::components::field::editing 120 40 junie mono 0ceaa4a8ed16e37c -> 0b67d0871ed59b84
  render::components::field::editing 40 10 junie mono 4ec41de042d6631c -> bd0b87a38bee3324
  render::components::field::empty 120 40 junie mono 30fa24a4a4ecdd59 -> d5a35484d2f42fef
  render::components::field::empty 40 10 junie mono ad91e7beafeedab9 -> fecf0453167b120f
  render::components::field::focused 120 40 junie mono e0effcaadcaf991a -> e307212b01aa1ffc
  render::components::field::focused 40 10 junie mono 50f588885beb01fa -> e4cd4a264b0e451c
  render::components::field::hovered 120 40 junie mono df6c39650b0ade4c -> bbfada53d625d404
  render::components::field::hovered 40 10 junie mono 623ecdc1e6dcb26c -> c379e0d03ce5b5a4
  render::components::field::pressed 120 40 junie mono 00f596f6dc359242 -> a464aa67b753d71e
  render::components::field::pressed 40 10 junie mono 433cf6a02ad228a2 -> 67ae3008bc25787e
  render::components::field::selected 120 40 junie mono df6c39650b0ade4c -> bbfada53d625d404
  render::components::field::selected 40 10 junie mono 623ecdc1e6dcb26c -> c379e0d03ce5b5a4
  render::components::filter_list::default 120 40 junie mono 5173a3946a6f6d5d -> e2b89dabf9a1378b
  render::components::filter_list::default 40 10 junie mono d1164217669aa67d -> 0102af140fa1032b
  render::components::filter_list::disabled 120 40 junie mono e48d23717d79dbe7 -> f2b83f0f43dbcf5f
  render::components::filter_list::disabled 40 10 junie mono da80e7b7ffbee9e7 -> e34fcaffa270237f
  render::components::filter_list::editing 120 40 junie mono 5173a3946a6f6d5d -> e2b89dabf9a1378b
  render::components::filter_list::editing 40 10 junie mono d1164217669aa67d -> 0102af140fa1032b
  render::components::filter_list::empty 120 40 junie mono 3a116dbe6b6cd15d -> 4dec323f8655796f
  render::components::filter_list::empty 120 40 junie truecolor 1d170068e8dc22c9 -> 964725de563d6f01
  render::components::filter_list::empty 120 40 paper truecolor 67f6d78d0e4d6297 -> 5657ae8b892e03c1
  render::components::filter_list::empty 40 10 junie mono e7c1508916e21ebd -> 893be9a0169c508f
  render::components::filter_list::empty 40 10 junie truecolor 1efd980f03fcdfc9 -> 81fe767b5e763581
  render::components::filter_list::empty 40 10 paper truecolor 299686fa5483e7e7 -> 9fe579eb452ad7c1
  render::components::filter_list::focused 120 40 junie mono 6449c6c04aeb76e9 -> 8b31ff36022ecaef
  render::components::filter_list::focused 40 10 junie mono d09db330fe7eb549 -> d50b4ecfa1de4bcf
  render::components::filter_list::hovered 120 40 junie mono 5173a3946a6f6d5d -> e2b89dabf9a1378b
  render::components::filter_list::hovered 40 10 junie mono d1164217669aa67d -> 0102af140fa1032b
  render::components::filter_list::pressed 120 40 junie mono 6449c6c04aeb76e9 -> 8b31ff36022ecaef
  render::components::filter_list::pressed 40 10 junie mono d09db330fe7eb549 -> d50b4ecfa1de4bcf
  render::components::filter_list::selected 120 40 junie mono 5173a3946a6f6d5d -> e2b89dabf9a1378b
  render::components::filter_list::selected 40 10 junie mono d1164217669aa67d -> 0102af140fa1032b
  render::components::form::default 120 40 junie mono b067bba77d5c5070 -> d1d99dc466566fd8
  render::components::form::default 40 10 junie mono 9cb233805c888130 -> 0781085339805b98
  render::components::form::disabled 120 40 junie mono 41ff736d5f719956 -> 592f9e22dba5d2d6
  render::components::form::disabled 40 10 junie mono 71b0cb5d8075c456 -> 2c7daf33d112a5d6
  render::components::form::editing 120 40 junie mono b067bba77d5c5070 -> d1d99dc466566fd8
  render::components::form::editing 40 10 junie mono 9cb233805c888130 -> 0781085339805b98
  render::components::form::focused 120 40 junie mono 053f948c947c7fce -> fc3913bdb465d7d8
  render::components::form::focused 40 10 junie mono 1f6db2e690979f0e -> 3b29bac231063518
  render::components::form::hovered 120 40 junie mono b067bba77d5c5070 -> d1d99dc466566fd8
  render::components::form::hovered 40 10 junie mono 9cb233805c888130 -> 0781085339805b98
  render::components::form::pressed 120 40 junie mono b917a0c4865988e4 -> e7e36f9bccbef2fa
  render::components::form::pressed 40 10 junie mono 1d078bc7da91e364 -> 01d9d3b400034ffa
  render::components::form::selected 120 40 junie mono 3f5dc245e1ba0945 -> 8b26230b52546d35
  render::components::form::selected 40 10 junie mono 1296c0b401709c85 -> 0406c95169d02975
  render::components::grid::default 120 40 junie mono cc1660d468c321f6 -> d0f6ba3fbb7ab79a
  render::components::grid::default 40 10 junie mono 868216137c400e56 -> bb17af21a1c1388a
  render::components::grid::disabled 120 40 junie mono cc1660d468c321f6 -> d0f6ba3fbb7ab79a
  render::components::grid::disabled 40 10 junie mono 868216137c400e56 -> bb17af21a1c1388a
  render::components::grid::editing 120 40 junie mono cc1660d468c321f6 -> d0f6ba3fbb7ab79a
  render::components::grid::editing 40 10 junie mono 868216137c400e56 -> bb17af21a1c1388a
  render::components::grid::empty 120 40 junie mono 70bb10c291b1c5b9 -> e1c80530090ac30d
  render::components::grid::empty 120 40 junie truecolor 312b25df0c0619c1 -> ef17e66997c72ad1
  render::components::grid::empty 120 40 paper truecolor 2147e9f589409929 -> f7028c8dabc46061
  render::components::grid::empty 40 10 junie mono 69ef7395cb4f3699 -> ac505014d6e73a6d
  render::components::grid::empty 40 10 junie truecolor c3633f90fa6f6f01 -> 676a00ea0ce59071
  render::components::grid::empty 40 10 paper truecolor 03cc64396325c9f9 -> 8012463ca99de109
  render::components::grid::focused 120 40 junie mono fe58af36f3b95730 -> dac7679930934574
  render::components::grid::focused 40 10 junie mono 420a6954a9fdbe10 -> f124fb945e76be84
  render::components::grid::hovered 120 40 junie mono cc1660d468c321f6 -> d0f6ba3fbb7ab79a
  render::components::grid::hovered 40 10 junie mono 868216137c400e56 -> bb17af21a1c1388a
  render::components::grid::pressed 120 40 junie mono e8c3754c3598953b -> c25e1f486f7b550f
  render::components::grid::pressed 40 10 junie mono 8bfbc9f03978819b -> be4cbad819fdf29f
  render::components::grid::selected 120 40 junie mono 47905521c7e525fb -> 222796a2adc2aaa7
  render::components::grid::selected 40 10 junie mono 95f1e6957b28b4db -> 07dbc80fd67c7857
  render::components::help_overlay::default 120 40 junie mono b77395f4124e7d72 -> 52bf229cac07cf50
  render::components::help_overlay::default 40 10 junie mono 72cc93eae9d73a0a -> 1e0f90fc56a681e8
  render::components::help_overlay::disabled 120 40 junie mono b77395f4124e7d72 -> 52bf229cac07cf50
  render::components::help_overlay::disabled 40 10 junie mono 72cc93eae9d73a0a -> 1e0f90fc56a681e8
  render::components::help_overlay::editing 120 40 junie mono b77395f4124e7d72 -> 52bf229cac07cf50
  render::components::help_overlay::editing 40 10 junie mono 72cc93eae9d73a0a -> 1e0f90fc56a681e8
  render::components::help_overlay::empty 120 40 junie mono 7010adc7f972c229 -> fb4dcd2a9ec55b83
  render::components::help_overlay::empty 40 10 junie mono 26031220ef3a4451 -> 69a7aa21deb2bf6b
  render::components::help_overlay::focused 120 40 junie mono 07708abd2a0caa1e -> 826ffa81caa999b6
  render::components::help_overlay::focused 40 10 junie mono 1803f9f0fb0983c6 -> 560ef73f9c4ce922
  render::components::help_overlay::hovered 120 40 junie mono b77395f4124e7d72 -> 52bf229cac07cf50
  render::components::help_overlay::hovered 40 10 junie mono 72cc93eae9d73a0a -> 1e0f90fc56a681e8
  render::components::help_overlay::pressed 120 40 junie mono 49582265af1f9b0a -> 057b6d8b93097bc6
  render::components::help_overlay::pressed 40 10 junie mono bea563c9b6edfeba -> 06cb89e6edb35e32
  render::components::help_overlay::selected 120 40 junie mono b77395f4124e7d72 -> 52bf229cac07cf50
  render::components::help_overlay::selected 40 10 junie mono 72cc93eae9d73a0a -> 1e0f90fc56a681e8
  render::components::hint_bar::default 120 40 junie mono aaf7cf28e13e826a -> f4af0f5e20d66b32
  render::components::hint_bar::default 40 10 junie mono fdd3f500925e3a6a -> cc6d9bb7f8457852
  render::components::hint_bar::disabled 120 40 junie mono 592985dbcc22164b -> dea41b81a52f0e47
  render::components::hint_bar::disabled 40 10 junie mono d2a777a0b59c624b -> a82e153d933d7267
  render::components::hint_bar::editing 120 40 junie mono 1a74082eb268df4b -> be8f30b338afce4f
  render::components::hint_bar::editing 40 10 junie mono 3192a28a54d74b4b -> adafaccc1271d82f
  render::components::hint_bar::empty 120 40 junie mono 596490634b754be5 -> 202dfbd8ddab1e15
  render::components::hint_bar::empty 40 10 junie mono 02fc4ced2ee354e5 -> d93dd709be5dbd75
  render::components::hint_bar::focused 120 40 junie mono aaf7cf28e13e826a -> f4af0f5e20d66b32
  render::components::hint_bar::focused 40 10 junie mono fdd3f500925e3a6a -> cc6d9bb7f8457852
  render::components::hint_bar::hovered 120 40 junie mono aaf7cf28e13e826a -> f4af0f5e20d66b32
  render::components::hint_bar::hovered 40 10 junie mono fdd3f500925e3a6a -> cc6d9bb7f8457852
  render::components::hint_bar::pressed 120 40 junie mono 197dc165a722ba63 -> a6b103102a062367
  render::components::hint_bar::pressed 40 10 junie mono 8f236bbf0fb9ea63 -> 12cc86ff03ed6247
  render::components::hint_bar::selected 120 40 junie mono aaf7cf28e13e826a -> f4af0f5e20d66b32
  render::components::hint_bar::selected 40 10 junie mono fdd3f500925e3a6a -> cc6d9bb7f8457852
  render::components::key_hint::default 120 40 junie mono 0a9205149528b380 -> 8bc93f70a7b2fbf0
  render::components::key_hint::default 40 10 junie mono 7549c8a14fbc6600 -> c65a1859cea1a670
  render::components::key_hint::disabled 120 40 junie mono 0a9205149528b380 -> 8bc93f70a7b2fbf0
  render::components::key_hint::disabled 40 10 junie mono 7549c8a14fbc6600 -> c65a1859cea1a670
  render::components::key_hint::editing 120 40 junie mono 0a9205149528b380 -> 8bc93f70a7b2fbf0
  render::components::key_hint::editing 40 10 junie mono 7549c8a14fbc6600 -> c65a1859cea1a670
  render::components::key_hint::empty 120 40 junie mono 0a9205149528b380 -> 8bc93f70a7b2fbf0
  render::components::key_hint::empty 40 10 junie mono 7549c8a14fbc6600 -> c65a1859cea1a670
  render::components::key_hint::focused 120 40 junie mono 0a9205149528b380 -> 8bc93f70a7b2fbf0
  render::components::key_hint::focused 40 10 junie mono 7549c8a14fbc6600 -> c65a1859cea1a670
  render::components::key_hint::hovered 120 40 junie mono 0a9205149528b380 -> 8bc93f70a7b2fbf0
  render::components::key_hint::hovered 40 10 junie mono 7549c8a14fbc6600 -> c65a1859cea1a670
  render::components::key_hint::pressed 120 40 junie mono 0a9205149528b380 -> 8bc93f70a7b2fbf0
  render::components::key_hint::pressed 40 10 junie mono 7549c8a14fbc6600 -> c65a1859cea1a670
  render::components::key_hint::selected 120 40 junie mono 0a9205149528b380 -> 8bc93f70a7b2fbf0
  render::components::key_hint::selected 40 10 junie mono 7549c8a14fbc6600 -> c65a1859cea1a670
  render::components::list::default 120 40 junie mono 164c36b18586726f -> fc98c428f6c305b1
  render::components::list::default 40 10 junie mono deac25911bc9718f -> 07a3a2e2097a7dd1
  render::components::list::disabled 120 40 junie mono 164c36b18586726f -> fc98c428f6c305b1
  render::components::list::disabled 40 10 junie mono deac25911bc9718f -> 07a3a2e2097a7dd1
  render::components::list::editing 120 40 junie mono 164c36b18586726f -> fc98c428f6c305b1
  render::components::list::editing 40 10 junie mono deac25911bc9718f -> 07a3a2e2097a7dd1
  render::components::list::empty 120 40 junie mono d43c8daeaf982a7a -> 449dd86eee94479e
  render::components::list::empty 120 40 junie truecolor 121b6cb8fded49a2 -> 031f18534b558a72
  render::components::list::empty 120 40 paper truecolor ddbd052dff13465a -> 6aa320d4b4a79de2
  render::components::list::empty 40 10 junie mono aeb7ad6bd94d0a1a -> de62a211f1cd715e
  render::components::list::empty 40 10 junie truecolor ffa7ea144c4cbd62 -> 4bc1ebbfb6005ef2
  render::components::list::empty 40 10 paper truecolor 065d1b5055061c8a -> be7ba0c4623a2cca
  render::components::list::focused 120 40 junie mono 164c36b18586726f -> fc98c428f6c305b1
  render::components::list::focused 40 10 junie mono deac25911bc9718f -> 07a3a2e2097a7dd1
  render::components::list::hovered 120 40 junie mono 164c36b18586726f -> fc98c428f6c305b1
  render::components::list::hovered 120 40 junie truecolor 96086a8fb7489f5a -> 413f0d1203429a0a
  render::components::list::hovered 40 10 junie mono deac25911bc9718f -> 07a3a2e2097a7dd1
  render::components::list::hovered 40 10 junie truecolor 95c30a93fad87dda -> f488a977e2ce582a
  render::components::list::pressed 120 40 junie mono cb5e7c9046398305 -> f7555667a42767a7
  render::components::list::pressed 40 10 junie mono 7f93e89c2dddc765 -> 9bff234723dacf07
  render::components::list::selected 120 40 junie mono 948062195792e82f -> 1b58cb5a2edf9b8d
  render::components::list::selected 40 10 junie mono 6c049b1cb101664f -> 61198a0fd38320ad
  render::components::meter::default 120 40 junie mono fab1931d8ce0ae06 -> e71bd00db82600aa
  render::components::meter::default 40 10 junie mono 5c7403a5b598074e -> 97718db10ff7b28e
  render::components::meter::disabled 120 40 junie mono 72b307545bf95a32 -> 278af2a688e8665c
  render::components::meter::disabled 40 10 junie mono b66ee080f8eb64fa -> 9da8e06150a319a0
  render::components::meter::editing 120 40 junie mono 664372a5c0eb31d2 -> 5f0e50eb5d01e034
  render::components::meter::editing 40 10 junie mono d78165edbfbe84da -> f8cae75e7ed9c298
  render::components::meter::empty 120 40 junie mono 1730e1dbc8ccc972 -> 98ca14abe94ebb64
  render::components::meter::empty 40 10 junie mono d59a9c98c54689d2 -> 9eb11e614c9b2ec4
  render::components::meter::focused 120 40 junie mono fab1931d8ce0ae06 -> e71bd00db82600aa
  render::components::meter::focused 40 10 junie mono 5c7403a5b598074e -> 97718db10ff7b28e
  render::components::meter::hovered 120 40 junie mono fab1931d8ce0ae06 -> e71bd00db82600aa
  render::components::meter::hovered 40 10 junie mono 5c7403a5b598074e -> 97718db10ff7b28e
  render::components::meter::pressed 120 40 junie mono 505b40d8b1e4656a -> 47cbac4c63a0df9c
  render::components::meter::pressed 40 10 junie mono cadbc9890f6f4f42 -> c2014767122ef000
  render::components::meter::selected 120 40 junie mono fab1931d8ce0ae06 -> e71bd00db82600aa
  render::components::meter::selected 40 10 junie mono 5c7403a5b598074e -> 97718db10ff7b28e
  render::components::nav_list::default 120 40 junie mono 3edf5c964f3d7e70 -> 82c72da4a1b64050
  render::components::nav_list::default 40 10 junie mono 613c729105191690 -> 312fe4a1f1d2b390
  render::components::nav_list::disabled 120 40 junie mono 7546020650b6bc2a -> e685cec788a2d5b6
  render::components::nav_list::disabled 40 10 junie mono 4dd81c7a68c66e2a -> 1854c5bcbdf831b6
  render::components::nav_list::editing 120 40 junie mono 3edf5c964f3d7e70 -> 82c72da4a1b64050
  render::components::nav_list::editing 40 10 junie mono 613c729105191690 -> 312fe4a1f1d2b390
  render::components::nav_list::focused 120 40 junie mono 3edf5c964f3d7e70 -> 82c72da4a1b64050
  render::components::nav_list::focused 40 10 junie mono 613c729105191690 -> 312fe4a1f1d2b390
  render::components::nav_list::hovered 120 40 junie mono 3edf5c964f3d7e70 -> 82c72da4a1b64050
  render::components::nav_list::hovered 120 40 junie truecolor 8d002c6b06d998a1 -> 651b9384d988b491
  render::components::nav_list::hovered 120 40 paper truecolor 2d3671c68d9a4fd7 -> 4b970bd6535516e7
  render::components::nav_list::hovered 40 10 junie mono 613c729105191690 -> 312fe4a1f1d2b390
  render::components::nav_list::hovered 40 10 junie truecolor 9be3a9429545f621 -> 9e351f7ef2d50ca1
  render::components::nav_list::hovered 40 10 paper truecolor 98693e337bb6cf67 -> d1c48195b012bcf7
  render::components::nav_list::pressed 120 40 junie mono d3726f43d9cf7856 -> 0c95b43132026d74
  render::components::nav_list::pressed 40 10 junie mono aabd8a0f4ff229b6 -> 1a681d52256366b4
  render::components::nav_list::selected 120 40 junie mono 028d2f1bf8f11b18 -> 33c5df0ab947c45e
  render::components::nav_list::selected 40 10 junie mono c6d6052500f71e78 -> b408c8dc503a085e
  render::components::panel::default 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::default 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::disabled 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::disabled 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::editing 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::editing 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::focused 120 40 junie mono f3cb0357a45a63b8 -> b2f09676a791a4dc
  render::components::panel::focused 40 10 junie mono 99fe87c2dd6f4010 -> f03ce99ba1f7a1fc
  render::components::panel::hovered 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::hovered 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::pressed 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::pressed 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::panel::selected 120 40 junie mono 525b1bb2ee6b02b6 -> 46447779f431f8d8
  render::components::panel::selected 40 10 junie mono e7834a28f412d7ae -> eed9be0d1fab3db0
  render::components::picker::default 120 40 junie mono b3ae92858be43071 -> f47d8d99e29b5ecd
  render::components::picker::default 40 10 junie mono f74d37210e531a91 -> 2a28542c1ee01bad
  render::components::picker::disabled 120 40 junie mono 277c1b15f0da725f -> de2f111f23c6ebbd
  render::components::picker::disabled 40 10 junie mono 40a62da040d2132f -> ec17447085f3e97d
  render::components::picker::editing 120 40 junie mono b3ae92858be43071 -> f47d8d99e29b5ecd
  render::components::picker::editing 40 10 junie mono f74d37210e531a91 -> 2a28542c1ee01bad
  render::components::picker::empty 120 40 junie mono ff1c8f41860df5d1 -> 7eaac187f342bd87
  render::components::picker::empty 120 40 junie truecolor 24a1c023e47afb91 -> edd1c31de913f869
  render::components::picker::empty 120 40 paper truecolor 13a1c24275705d9b -> 4ff1f03b4b7d8b83
  render::components::picker::empty 40 10 junie mono 39381cf0b9a27af1 -> db2c6d3cf703cbbf
  render::components::picker::empty 40 10 junie truecolor e8d69c83a18c8671 -> 714bb240ee33a55d
  render::components::picker::empty 40 10 paper truecolor b9694c43428d2f4b -> b8b9a70e4a14e6bb
  render::components::picker::focused 120 40 junie mono 3b3e3c128e5a6a0d -> dd3c397759092899
  render::components::picker::focused 40 10 junie mono e71f4fbb98c0629d -> ee03cdef75885c49
  render::components::picker::hovered 120 40 junie mono b3ae92858be43071 -> c90ab2c81e3a114d
  render::components::picker::hovered 40 10 junie mono f74d37210e531a91 -> 3415ce6da23f8705
  render::components::picker::pressed 120 40 junie mono 3b3e3c128e5a6a0d -> dd3c397759092899
  render::components::picker::pressed 40 10 junie mono e71f4fbb98c0629d -> ee03cdef75885c49
  render::components::picker::selected 120 40 junie mono b3ae92858be43071 -> f47d8d99e29b5ecd
  render::components::picker::selected 40 10 junie mono f74d37210e531a91 -> 2a28542c1ee01bad
  render::components::picker_chain::default 120 40 junie mono 79a4feb79184f0d2 -> e760424179d13e68
  render::components::picker_chain::default 40 10 junie mono 3b157164d06fad32 -> 188b603038cf6c78
  render::components::picker_chain::disabled 120 40 junie mono 30fbbf2af069aeed -> d74d223f7a7a8a6d
  render::components::picker_chain::disabled 40 10 junie mono ad3b7fbd5f52b86d -> 235939787ed0a84d
  render::components::picker_chain::editing 120 40 junie mono 5b807aec32dff981 -> 596bfc45512f82af
  render::components::picker_chain::editing 40 10 junie mono 4f44b797af665fe1 -> 29ee929b75e3e13f
  render::components::picker_chain::empty 120 40 junie mono cb5e907ef014b325 -> 24f728ea4e9af26d
  render::components::picker_chain::empty 40 10 junie mono 13af94df52443205 -> 0c59b15414297cfd
  render::components::picker_chain::focused 120 40 junie mono 08c34c641eaea87a -> da0588a65e635080
  render::components::picker_chain::focused 40 10 junie mono 840adf5dcabf44da -> 1648a42278ce9990
  render::components::picker_chain::hovered 120 40 junie mono 79a4feb79184f0d2 -> e760424179d13e68
  render::components::picker_chain::hovered 40 10 junie mono 3b157164d06fad32 -> 188b603038cf6c78
  render::components::picker_chain::pressed 120 40 junie mono f6a1b4a7b88f37d3 -> fce4678c7e7e085f
  render::components::picker_chain::pressed 40 10 junie mono ff28d5b515b64a13 -> 7b7dfbb2db0debbf
  render::components::picker_chain::selected 120 40 junie mono 79a4feb79184f0d2 -> e760424179d13e68
  render::components::picker_chain::selected 40 10 junie mono 3b157164d06fad32 -> 188b603038cf6c78
  render::components::progress_bar::default 120 40 junie mono 1da207d050ccbe00 -> 385f9dd546b0d75c
  render::components::progress_bar::default 40 10 junie mono 640c9e6d0236a448 -> b161d6cb40a37114
  render::components::progress_bar::disabled 120 40 junie mono ff0f692938ab8559 -> 11fb9771b26ce04b
  render::components::progress_bar::disabled 40 10 junie mono 31e29eaf4ff25581 -> a9003cbfb520c6c3
  render::components::progress_bar::editing 120 40 junie mono 6a4f7a72886db1b1 -> a5c260c9d685e4fb
  render::components::progress_bar::editing 40 10 junie mono bd9ca2fdffa8f379 -> 567b51a35bef7473
  render::components::progress_bar::empty 120 40 junie mono 2d302110085c5136 -> 2ce6acd25da0570c
  render::components::progress_bar::empty 40 10 junie mono 752b3f97f25b9796 -> 3e1437939ab8926c
  render::components::progress_bar::focused 120 40 junie mono 1da207d050ccbe00 -> 385f9dd546b0d75c
  render::components::progress_bar::focused 40 10 junie mono 640c9e6d0236a448 -> b161d6cb40a37114
  render::components::progress_bar::hovered 120 40 junie mono 1da207d050ccbe00 -> 385f9dd546b0d75c
  render::components::progress_bar::hovered 40 10 junie mono 640c9e6d0236a448 -> b161d6cb40a37114
  render::components::progress_bar::pressed 120 40 junie mono 2db6b19ba7a37a19 -> 67261ac4746eb963
  render::components::progress_bar::pressed 40 10 junie mono 6fb9de54d8806411 -> ec24ad5de83c540b
  render::components::progress_bar::selected 120 40 junie mono 1da207d050ccbe00 -> 385f9dd546b0d75c
  render::components::progress_bar::selected 40 10 junie mono 640c9e6d0236a448 -> b161d6cb40a37114
  render::components::radio_group::default 120 40 junie mono 23253cfc778734b7 -> a61fe8ade17a3465
  render::components::radio_group::default 40 10 junie mono 783665ea3658b177 -> 2bad9bce00760c25
  render::components::radio_group::disabled 120 40 junie mono 0b3191ebe74270d5 -> 41feea0417c6f64d
  render::components::radio_group::disabled 40 10 junie mono ac10dc1dca9bd455 -> 7e9b20271996868d
  render::components::radio_group::editing 120 40 junie mono 23253cfc778734b7 -> a61fe8ade17a3465
  render::components::radio_group::editing 40 10 junie mono 783665ea3658b177 -> 2bad9bce00760c25
  render::components::radio_group::focused 120 40 junie mono 23253cfc778734b7 -> a61fe8ade17a3465
  render::components::radio_group::focused 40 10 junie mono 783665ea3658b177 -> 2bad9bce00760c25
  render::components::radio_group::hovered 120 40 junie mono 23253cfc778734b7 -> a61fe8ade17a3465
  render::components::radio_group::hovered 40 10 junie mono 783665ea3658b177 -> 2bad9bce00760c25
  render::components::radio_group::pressed 120 40 junie mono c1fcc82633a44665 -> 2ff02eaec05151a3
  render::components::radio_group::pressed 40 10 junie mono 96df91bc5acc71e5 -> 5ecef9c227f483a3
  render::components::radio_group::selected 120 40 junie mono 8d0ca4e2991bd3b3 -> 0d03f9f9b98936eb
  render::components::radio_group::selected 40 10 junie mono 68c8a89b7d2e9d73 -> 8ed3d898b1f6aeab
  render::components::scroll_region::default 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::default 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::disabled 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::disabled 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::editing 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::editing 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::focused 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::focused 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::hovered 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::hovered 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::scroll_region::pressed 120 40 junie mono 55fca6c68455d891 -> fcd176f0c7336e03
  render::components::scroll_region::pressed 40 10 junie mono 869ee257115dc6f2 -> dce5bb00e7e45cae
  render::components::scroll_region::selected 120 40 junie mono cf39e5566ea2431d -> 6a3eeaca06cb68b7
  render::components::scroll_region::selected 40 10 junie mono ac69feb9ccd5119d -> 741ca9c56703f6e9
  render::components::select::empty 120 40 junie mono 62b183a706e10f34 -> 39f0597a6d42c8c4
  render::components::select::empty 40 10 junie mono 6ab520d8e0c9ded4 -> 3a4caeb29a3567e4
  render::components::select::focused 120 40 junie mono 88a794db4ad7438a -> bf6222dc85c86ed6
  render::components::select::focused 40 10 junie mono cfdca6c48e84fdaa -> 02b43f436621dc76
  render::components::select::pressed 120 40 junie mono c4e91c58984eb68c -> 23e63edf534b4b50
  render::components::select::pressed 40 10 junie mono 75aee1f6e6798d6c -> 699174892d610db0
  render::components::spinner::default 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::default 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::spinner::disabled 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::disabled 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::spinner::editing 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::editing 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::spinner::empty 120 40 junie mono 3bd27cf92356d246 -> b566abc8184ef10a
  render::components::spinner::empty 40 10 junie mono 627669c231671246 -> 24b40e417bb6db0a
  render::components::spinner::focused 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::focused 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::spinner::hovered 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::hovered 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::spinner::pressed 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::pressed 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::spinner::selected 120 40 junie mono a6f2e5979c8e4d8f -> 3541335c9f45619d
  render::components::spinner::selected 40 10 junie mono 3c96bff36c20218f -> 4f49a212b6ad351d
  render::components::split_pane::hovered 120 40 junie mono 0aa202328d87e41f -> 7771bf330413d437
  render::components::split_pane::hovered 40 10 junie mono 5763a80a2217826f -> b2415724177906bf
  render::components::split_pane::pressed 120 40 junie mono 8422e4d642355417 -> efad2dfbd336473f
  render::components::split_pane::pressed 40 10 junie mono f749391b03aad49f -> 0065509ef9b5786d
  render::components::status_bar::default 120 40 junie mono d1559da39b79e586 -> 5a49407b637398c0
  render::components::status_bar::default 40 10 junie mono 94a382c6c4d907c6 -> a7df662353b052f0
  render::components::status_bar::disabled 120 40 junie mono dba9eaec3520c297 -> 431c9e4d28001609
  render::components::status_bar::disabled 40 10 junie mono c06e4f5f9ad85657 -> 2b9805cdaadbed79
  render::components::status_bar::editing 120 40 junie mono bd19c6ccb9b63237 -> 6672f81d23ca0a81
  render::components::status_bar::editing 40 10 junie mono 81e080b38aa0ee77 -> 78078a1ef4270b31
  render::components::status_bar::empty 120 40 junie mono 596490634b754be5 -> 202dfbd8ddab1e15
  render::components::status_bar::empty 40 10 junie mono 02fc4ced2ee354e5 -> d93dd709be5dbd75
  render::components::status_bar::focused 120 40 junie mono 9c52f5abe06ff816 -> 7461347ab83dd148
  render::components::status_bar::focused 40 10 junie mono 0abcb26f36a88396 -> 45724b18d5e31448
  render::components::status_bar::hovered 120 40 junie mono d1559da39b79e586 -> 5a49407b637398c0
  render::components::status_bar::hovered 40 10 junie mono 94a382c6c4d907c6 -> a7df662353b052f0
  render::components::status_bar::selected 120 40 junie mono d1559da39b79e586 -> 5a49407b637398c0
  render::components::status_bar::selected 40 10 junie mono 94a382c6c4d907c6 -> a7df662353b052f0
  render::components::steps::default 120 40 junie mono 16c60c0e71d05628 -> c61045ed41083f42
  render::components::steps::default 40 10 junie mono fbf6cac31bf12a28 -> f8bd22069139e5c2
  render::components::steps::disabled 120 40 junie mono b054f35ce7b8a314 -> 965cfe7a4805312e
  render::components::steps::disabled 40 10 junie mono 1c36ff0e2567fb14 -> 953e689fd31da82e
  render::components::steps::editing 120 40 junie mono 16c60c0e71d05628 -> c61045ed41083f42
  render::components::steps::editing 40 10 junie mono fbf6cac31bf12a28 -> f8bd22069139e5c2
  render::components::steps::focused 120 40 junie mono 16c60c0e71d05628 -> c61045ed41083f42
  render::components::steps::focused 40 10 junie mono fbf6cac31bf12a28 -> f8bd22069139e5c2
  render::components::steps::hovered 120 40 junie mono 16c60c0e71d05628 -> c61045ed41083f42
  render::components::steps::hovered 40 10 junie mono fbf6cac31bf12a28 -> f8bd22069139e5c2
  render::components::steps::pressed 120 40 junie mono 831f261cb10a53aa -> 603e38e53db54d8a
  render::components::steps::pressed 40 10 junie mono c86e449c901601ea -> f017cdb63e293aca
  render::components::steps::selected 120 40 junie mono 16c60c0e71d05628 -> c61045ed41083f42
  render::components::steps::selected 40 10 junie mono fbf6cac31bf12a28 -> f8bd22069139e5c2
  render::components::tabs::default 120 40 junie mono 03263f48effb1240 -> 269cea1731877156
  render::components::tabs::default 40 10 junie mono 8575598b028539ba -> 231249d22e78dc9e
  render::components::tabs::disabled 120 40 junie mono 03263f48effb1240 -> 269cea1731877156
  render::components::tabs::disabled 40 10 junie mono 8575598b028539ba -> 231249d22e78dc9e
  render::components::tabs::editing 120 40 junie mono 03263f48effb1240 -> 269cea1731877156
  render::components::tabs::editing 40 10 junie mono 8575598b028539ba -> 231249d22e78dc9e
  render::components::tabs::empty 120 40 junie mono cbb71915528ecabc -> d2c1739d63347980
  render::components::tabs::empty 40 10 junie mono fcafc82c5683275c -> f34f0ec350635aa0
  render::components::tabs::focused 120 40 junie mono 03263f48effb1240 -> 269cea1731877156
  render::components::tabs::focused 40 10 junie mono 8575598b028539ba -> 231249d22e78dc9e
  render::components::tabs::hovered 120 40 junie mono 03263f48effb1240 -> b10abf07af527610
  render::components::tabs::hovered 40 10 junie mono 8575598b028539ba -> d6a2a1e72a8a6d44
  render::components::tabs::pressed 120 40 junie mono 8531aef99ed82a7c -> 9752ca681323d49c
  render::components::tabs::pressed 40 10 junie mono a1ca30a076849608 -> 324c23adb0bea02a
  render::components::tabs::selected 120 40 junie mono 643602b7923c4efc -> 00e641f631c45e0a
  render::components::tabs::selected 40 10 junie mono 204d5a3787674b8a -> 1af5ca9eeb7b5548
  render::components::text_area::empty 120 40 junie mono a0bf87cfd6be303a -> 590ebeb81fcea254
  render::components::text_area::empty 40 10 junie mono def451c30c7f2cba -> 53e826817477f054
  render::components::text_area::focused 120 40 junie mono ce1406872b5ebb16 -> 9841d588f637d738
  render::components::text_area::focused 40 10 junie mono 2b87babeffae7496 -> 9e8a69cb24d16738
  render::components::text_area::pressed 120 40 junie mono ce1406872b5ebb16 -> 9841d588f637d738
  render::components::text_area::pressed 40 10 junie mono 2b87babeffae7496 -> 9e8a69cb24d16738
  render::components::text_input::empty 120 40 junie mono 7164d74b6f28ba99 -> 5b40c90c65ff372f
  render::components::text_input::empty 40 10 junie mono 0d3375cb2a3779f9 -> c5c26e53a68b500f
  render::components::text_input::focused 120 40 junie mono 35d6d4eea55110dc -> 0de614caba77d820
  render::components::text_input::focused 40 10 junie mono 2be5e4565d5323bc -> 9b054e2b7ad89f00
  render::components::text_input::pressed 120 40 junie mono 35d6d4eea55110dc -> 0de614caba77d820
  render::components::text_input::pressed 40 10 junie mono 2be5e4565d5323bc -> 9b054e2b7ad89f00
  render::components::text_viewport::default 120 40 junie mono 8b7199dc3bcf1d7f -> 76fa3fa3436b9bd7
  render::components::text_viewport::default 40 10 junie mono 6ae51735da984033 -> d52d4b972298118f
  render::components::text_viewport::disabled 120 40 junie mono 8b7199dc3bcf1d7f -> 76fa3fa3436b9bd7
  render::components::text_viewport::disabled 40 10 junie mono 6ae51735da984033 -> d52d4b972298118f
  render::components::text_viewport::editing 120 40 junie mono 8b7199dc3bcf1d7f -> 76fa3fa3436b9bd7
  render::components::text_viewport::editing 40 10 junie mono 6ae51735da984033 -> d52d4b972298118f
  render::components::text_viewport::empty 120 40 junie mono 7ee5caf9d25801d5 -> 4b429a14599c3985
  render::components::text_viewport::empty 40 10 junie mono 4e8e3ce69c0ecf99 -> c676d9b85998b6cd
  render::components::text_viewport::focused 120 40 junie mono ce0ec50093d5a64f -> 7b299f2e29f4e7e3
  render::components::text_viewport::focused 40 10 junie mono 9c828ab20e07c8cf -> 11fcfe72ddda58f3
  render::components::text_viewport::hovered 120 40 junie mono 8b7199dc3bcf1d7f -> 76fa3fa3436b9bd7
  render::components::text_viewport::hovered 40 10 junie mono 6ae51735da984033 -> d52d4b972298118f
  render::components::text_viewport::pressed 120 40 junie mono 52e340ac108c7c37 -> 046d271be822fd2b
  render::components::text_viewport::pressed 40 10 junie mono 6d4c2c7a1d9d83af -> 11a8ce05d597c0f3
  render::components::text_viewport::selected 120 40 junie mono f27c536c35f8ce2b -> b43a94003f66e203
  render::components::text_viewport::selected 40 10 junie mono 99583de8de019def -> 77d35a41bcf075eb
  render::components::toggle::default 120 40 junie mono 2d45f23596f14bc6 -> 34bf50f9749f06da
  render::components::toggle::default 40 10 junie mono 1c021a55f0b329a6 -> d6ef46ccdf65733a
  render::components::toggle::disabled 120 40 junie mono 1d407520526ee606 -> bba5a346ced7aa80
  render::components::toggle::disabled 40 10 junie mono beb844286afb9406 -> fa1a0c264f23a380
  render::components::toggle::editing 120 40 junie mono 2d45f23596f14bc6 -> 34bf50f9749f06da
  render::components::toggle::editing 40 10 junie mono 1c021a55f0b329a6 -> d6ef46ccdf65733a
  render::components::toggle::empty 120 40 junie mono 2d45f23596f14bc6 -> 34bf50f9749f06da
  render::components::toggle::empty 40 10 junie mono 1c021a55f0b329a6 -> d6ef46ccdf65733a
  render::components::toggle::focused 120 40 junie mono 0fcd0fd183469e38 -> cd3185384258edcc
  render::components::toggle::focused 40 10 junie mono fa7ca654caa4f0d8 -> f68aa0525ed1316c
  render::components::toggle::hovered 120 40 junie mono 2d45f23596f14bc6 -> 34bf50f9749f06da
  render::components::toggle::hovered 40 10 junie mono 1c021a55f0b329a6 -> d6ef46ccdf65733a
  render::components::toggle::pressed 120 40 junie mono 2c2b34a4e0bb4ed0 -> 39daf5678d923272
  render::components::toggle::pressed 40 10 junie mono 7641d693e530a6f0 -> a255fbb133860812
  render::components::toggle::selected 120 40 junie mono 73c985673a3204d6 -> dc3fdc77c98118da
  render::components::toggle::selected 40 10 junie mono 9dd7678ca006a8b6 -> f9c3dbdc8c9a853a
  render::components::too_small::default 120 40 junie mono e37f4e0c8c41d85c -> 6a7db0ae55b8cc2e
  render::components::too_small::default 40 10 junie mono 9dc2a8f43da5bbae -> cb89223cac386d26
  render::components::too_small::disabled 120 40 junie mono e37f4e0c8c41d85c -> 6a7db0ae55b8cc2e
  render::components::too_small::disabled 40 10 junie mono 9dc2a8f43da5bbae -> cb89223cac386d26
  render::components::too_small::editing 120 40 junie mono e37f4e0c8c41d85c -> 6a7db0ae55b8cc2e
  render::components::too_small::editing 40 10 junie mono 9dc2a8f43da5bbae -> cb89223cac386d26
  render::components::too_small::empty 120 40 junie mono d177c60d725d64aa -> 1965ede10e3f96dc
  render::components::too_small::empty 40 10 junie mono 256562eec5d491f8 -> b3051e2fbacc1c70
  render::components::too_small::focused 120 40 junie mono e37f4e0c8c41d85c -> 6a7db0ae55b8cc2e
  render::components::too_small::focused 40 10 junie mono 9dc2a8f43da5bbae -> cb89223cac386d26
  render::components::too_small::hovered 120 40 junie mono e37f4e0c8c41d85c -> 6a7db0ae55b8cc2e
  render::components::too_small::hovered 40 10 junie mono 9dc2a8f43da5bbae -> cb89223cac386d26
  render::components::too_small::pressed 120 40 junie mono e37f4e0c8c41d85c -> 6a7db0ae55b8cc2e
  render::components::too_small::pressed 40 10 junie mono 9dc2a8f43da5bbae -> cb89223cac386d26
  render::components::too_small::selected 120 40 junie mono e37f4e0c8c41d85c -> 6a7db0ae55b8cc2e
  render::components::too_small::selected 40 10 junie mono 9dc2a8f43da5bbae -> cb89223cac386d26
  render::components::tree::default 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::default 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 94e7ce70bb65b804
  render::components::tree::default 120 40 paper mono e7688a03d244eb33 -> b309aa954616d2e7
  render::components::tree::default 120 40 paper truecolor 81f56ee97b029198 -> b1092f71d75fb128
  render::components::tree::default 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::default 40 10 junie truecolor 99095ab47ad9a104 -> c5df854de7f82804
  render::components::tree::default 40 10 paper mono 8cef2684bcf13753 -> 10dbf2bf094ddac7
  render::components::tree::default 40 10 paper truecolor 04a805d95fdc0948 -> 2c1861b28dfe1218
  render::components::tree::disabled 120 40 junie mono 9ad9f5ae65342def -> b47c8c607dc66da5
  render::components::tree::disabled 120 40 junie truecolor 6c1e1c6a55a57308 -> 66f0f5e22836fe6c
  render::components::tree::disabled 120 40 paper mono 3078e8871da44d93 -> c42b1d1f816bccbf
  render::components::tree::disabled 120 40 paper truecolor ad8787fe91ced26d -> bd7bac2b04bf1601
  render::components::tree::disabled 40 10 junie mono 322991e541e6c6ef -> 24215f18b6f71925
  render::components::tree::disabled 40 10 junie truecolor b05e8a7a5e83cf48 -> c001164f315332ac
  render::components::tree::disabled 40 10 paper mono c793162e3ac72233 -> 937742fdc48c899f
  render::components::tree::disabled 40 10 paper truecolor dba0ce46f92191ad -> 28bbd39eca633101
  render::components::tree::editing 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::editing 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 94e7ce70bb65b804
  render::components::tree::editing 120 40 paper mono e7688a03d244eb33 -> b309aa954616d2e7
  render::components::tree::editing 120 40 paper truecolor 81f56ee97b029198 -> b1092f71d75fb128
  render::components::tree::editing 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::editing 40 10 junie truecolor 99095ab47ad9a104 -> c5df854de7f82804
  render::components::tree::editing 40 10 paper mono 8cef2684bcf13753 -> 10dbf2bf094ddac7
  render::components::tree::editing 40 10 paper truecolor 04a805d95fdc0948 -> 2c1861b28dfe1218
  render::components::tree::empty 120 40 junie mono d43c8daeaf982a7a -> 449dd86eee94479e
  render::components::tree::empty 120 40 junie truecolor 121b6cb8fded49a2 -> 031f18534b558a72
  render::components::tree::empty 120 40 paper truecolor ddbd052dff13465a -> 6aa320d4b4a79de2
  render::components::tree::empty 40 10 junie mono aeb7ad6bd94d0a1a -> de62a211f1cd715e
  render::components::tree::empty 40 10 junie truecolor ffa7ea144c4cbd62 -> 4bc1ebbfb6005ef2
  render::components::tree::empty 40 10 paper truecolor 065d1b5055061c8a -> be7ba0c4623a2cca
  render::components::tree::focused 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::focused 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 94e7ce70bb65b804
  render::components::tree::focused 120 40 paper mono e7688a03d244eb33 -> b309aa954616d2e7
  render::components::tree::focused 120 40 paper truecolor 81f56ee97b029198 -> b1092f71d75fb128
  render::components::tree::focused 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::focused 40 10 junie truecolor 99095ab47ad9a104 -> c5df854de7f82804
  render::components::tree::focused 40 10 paper mono 8cef2684bcf13753 -> 10dbf2bf094ddac7
  render::components::tree::focused 40 10 paper truecolor 04a805d95fdc0948 -> 2c1861b28dfe1218
  render::components::tree::hovered 120 40 junie mono d450dbacbe5781fd -> e4958a185b19b77d
  render::components::tree::hovered 120 40 junie truecolor 1a15529c5d5bc544 -> ae587d7429f2d2d4
  render::components::tree::hovered 120 40 paper mono e7688a03d244eb33 -> b309aa954616d2e7
  render::components::tree::hovered 120 40 paper truecolor c5eb5fd78905ad58 -> 6366223c93c0ddac
  render::components::tree::hovered 40 10 junie mono 77c59187cd64f51d -> d9e4cafcc2a6f5dd
  render::components::tree::hovered 40 10 junie truecolor 7a7a93564af16404 -> a381318908fe5c54
  render::components::tree::hovered 40 10 paper mono 8cef2684bcf13753 -> 10dbf2bf094ddac7
  render::components::tree::hovered 40 10 paper truecolor c733b9963459a748 -> 376e0191ded54e9c
  render::components::tree::pressed 120 40 junie mono d450dbacbe5781fd -> 2a4e808ba871c1b9
  render::components::tree::pressed 120 40 junie truecolor bd2e1b5ed9ef38c4 -> 069f49f3f3febc88
  render::components::tree::pressed 120 40 paper mono e7688a03d244eb33 -> ebc06c233268e8e1
  render::components::tree::pressed 120 40 paper truecolor 81f56ee97b029198 -> c94f18e732664b43
  render::components::tree::pressed 40 10 junie mono 77c59187cd64f51d -> 48f68635c4f8d9d9
  render::components::tree::pressed 40 10 junie truecolor 99095ab47ad9a104 -> 3bb041ba8aaadd08
  render::components::tree::pressed 40 10 paper mono 8cef2684bcf13753 -> 35000ca780235701
  render::components::tree::pressed 40 10 paper truecolor 04a805d95fdc0948 -> 62fb872559518823
  render::components::tree::selected 120 40 junie mono 96f1f5dbc64034bd -> 9e1c40991a3ede69
  render::components::tree::selected 120 40 junie truecolor 25d188f3267654af -> 1e04d0ca77ba5c6f
  render::components::tree::selected 120 40 paper mono f23368fb682439b3 -> 733c344364f50fc7
  render::components::tree::selected 120 40 paper truecolor d46a76d2cb4df19a -> fe2391d8faa6c80a
  render::components::tree::selected 40 10 junie mono fa7ef4626541ec5d -> 05ac5faae8a77a49
  render::components::tree::selected 40 10 junie truecolor 9ea34149e59474af -> cb796be352b2fbaf
  render::components::tree::selected 40 10 paper mono c8f712571ba5be53 -> 6c0add5e74824e27
  render::components::tree::selected 40 10 paper truecolor 825d1d60faeb52ca -> 5e325a89c362e5fa
- added:     none
- class:     intended
- reason:    §20.10 item 39. The union pipeline is the deliberate rendering of the merged
             workspace; both source lines were reviewed separately and the reference
             fixtures pin the app-level result. Reject any key outside the merged diff.
```