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
- moved:     4 lines, every one `mono` (`git diff crates/tui/tests/baselines/components.txt`):
  render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747 → 8531aef99ed82a7c
  render::components::tabs::pressed 120 40 paper mono ca497a2a34358f51 → 57bc3c9afc387ab6
  render::components::tabs::pressed 40 10 junie mono 8a4bc1549eca3857 → a1ca30a076849608
  render::components::tabs::pressed 40 10 paper mono dfaa198c140af319 → 35b0ee5d62f85452
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
- moved:     8 keys:
  render::components::field::disabled 120 40 junie mono a17a3ce53b0c07c0 → 6ea61957d4835dac
  render::components::field::disabled 120 40 paper mono 3ca520240375131a → 229e3e1e96741fcc
  render::components::field::disabled 40 10 junie mono 4a6989f667440f40 → 8a8e49368586582c
  render::components::field::disabled 40 10 paper mono 4356d66d70ef949a → d1a459721634068c
  render::components::select::pressed 120 40 junie mono 17bf131df914c266 → c4e91c58984eb68c
  render::components::select::pressed 120 40 paper mono 33ca9d784756b7a6 → 20cc3c074be6adaa
  render::components::select::pressed 40 10 junie mono eafa6492283387c6 → 75aee1f6e6798d6c
  render::components::select::pressed 40 10 paper mono cec1880d683c3a06 → 5dadb5bc71aedd0a
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
- moved:     12 keys:
  render::components::hint_bar::disabled 120 40 junie mono f5fc6531be4cfe81 → 592985dbcc22164b
  render::components::hint_bar::disabled 120 40 paper mono b8930e040b525b61 → fdc4972eb9e1d5b3
  render::components::hint_bar::disabled 40 10 junie mono ab06451beff9e981 → d2a777a0b59c624b
  render::components::hint_bar::disabled 40 10 paper mono 733043d7bddd5701 → cf244c80445dccd3
  render::components::meter::disabled 120 40 junie mono c3944936b57ee94a → 72b307545bf95a32
  render::components::meter::disabled 120 40 paper mono 5b6d92f61b8c8784 → dcb0e5704e94b512
  render::components::meter::disabled 40 10 junie mono 9ef934503dd41e92 → b66ee080f8eb64fa
  render::components::meter::disabled 40 10 paper mono 4b44e2bf17c01014 → b1b6ec010ae0db62
  render::components::progress_bar::disabled 120 40 junie mono ef46093a72cdf519 → ff0f692938ab8559
  render::components::progress_bar::disabled 120 40 paper mono c13aab919eb20ccf → 89f01f2d9442bbed
  render::components::progress_bar::disabled 40 10 junie mono 50c4f3da93780841 → 31e29eaf4ff25581
  render::components::progress_bar::disabled 40 10 paper mono 6fea492cd57e287f → b405be2d9cf4cf2d
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
- moved:     39 keys:
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
  render::components::chip_bar::focused 120 40 junie mono 6ad5af32d1c4ed26 → d410e30a8fa1287b
  render::components::chip_bar::focused 120 40 junie truecolor 3d29208b34fc2d1e → ec65ee8e34084547
  render::components::chip_bar::focused 120 40 paper mono 870de4e852c919c6 → af6ee1d9868d153b
  render::components::chip_bar::focused 120 40 paper truecolor 489998de99db8348 → 5262957dbd38ff11
  render::components::chip_bar::hovered 120 40 junie mono 857bb40ac027e8bc → d410e30a8fa1287b
  render::components::chip_bar::hovered 120 40 junie truecolor 57ce6520fe16bda8 → b5abf93467d48d7b
  render::components::chip_bar::hovered 120 40 paper mono 57b2cbc4e84613fa → af6ee1d9868d153b
  render::components::chip_bar::hovered 120 40 paper truecolor 7a7d32240fba017d → 7c0e750395d0ba6f
  render::components::chip_bar::hovered 40 10 junie truecolor 83292ce01ea8e206 → f9ffd9c455ac3bf4
  render::components::chip_bar::hovered 40 10 paper truecolor 28cd277ff3d2aef5 → 7bc243a33857e4c2
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
- reason:    §20.10 item 23; ChipBar semantic identity, metadata, marker and owned-patch correction.
```

## Item 28 — final semantic-selection movements

```
- surface:   crates/tui/tests/baselines/components.txt stabilized scratch comparison
- captures:  `/private/tmp/terminal-components-baseline-review.qbw8Cu/artifacts/components.before.txt`
             and `/private/tmp/terminal-components-baseline-review.qbw8Cu/repo/crates/tui/tests/baselines/components.txt`
- tests:     render::components::* plus the item-specific unit/conformance proofs above
- moved:     16 keys:
  render::components::button::selected 120 40 junie mono 1aafcb632b64bb09 → 13d0bae25b64bacc
  render::components::button::selected 120 40 junie truecolor 7cad0003dd9cd1fb → c3a5557f65a5ff3f
  render::components::button::selected 120 40 paper mono c72f9f98688eba59 → 4330743d0d1af0f4
  render::components::button::selected 120 40 paper truecolor 12c06980e5dd7a45 → 93ffce78c25faa79
  render::components::button::selected 40 10 junie mono 8eb20d2cb7769089 → 0f80b7f1cc3b9e4c
  render::components::button::selected 40 10 junie truecolor 6d11dbe79a66f3fb → 2b38a4b81a3e733f
  render::components::button::selected 40 10 paper mono 3851ec67ff1947d9 → 000a22404ee1f874
  render::components::button::selected 40 10 paper truecolor 5085b4b28e2a59c5 → 6b3f15ee40b3e7f9
  render::components::radio_group::pressed 120 40 junie mono 7c9348eec50e02e3 → c1fcc82633a44665
  render::components::radio_group::pressed 120 40 paper mono 5e4e76a54792f289 → af7f1cd31203adff
  render::components::radio_group::pressed 40 10 junie mono 6f0aebd819bba103 → 96df91bc5acc71e5
  render::components::radio_group::pressed 40 10 paper mono 1d9365ff80a86ca9 → f604b3a6b831c57f
  render::components::select::selected 120 40 junie mono 7e46b8873476eb92 → d56280a37a6d91b4
  render::components::select::selected 120 40 paper mono 8a6bf73ebb667692 → 439b7dee547eff04
  render::components::select::selected 40 10 junie mono d64d86b5cd399d32 → 762f74be76665454
  render::components::select::selected 40 10 paper mono 63fe454843c795b2 → 33254b34e59736a4
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
- moved:     28 keys:
  render::components::dialog::disabled 120 40 junie mono 2d6a3cd4c020d7e5 → b737d61b383ec6f5
  render::components::dialog::disabled 120 40 junie truecolor 0b134dfe49fc91dc → 3b3a35187a544472
  render::components::dialog::disabled 120 40 paper mono 6e84f6e549134175 → 98782672ba8211f1
  render::components::dialog::disabled 120 40 paper truecolor 9f89355d75801b00 → c32427125f196b4a
  render::components::dialog::disabled 40 10 junie mono 3ef081624f0f1fa5 → 081728ded55d1575
  render::components::dialog::disabled 40 10 junie truecolor 36cf11641eca3792 → bca990b4d6367868
  render::components::dialog::disabled 40 10 paper mono 9033e3ce2e9ff745 → f9fb14d2886dbe31
  render::components::dialog::disabled 40 10 paper truecolor 5d6911160dfe8ed6 → 37273c6e6232fa10
  render::components::dialog::focused 120 40 junie mono e7bc12c620075c81 → d057ba49733734b3
  render::components::dialog::focused 120 40 junie truecolor b82da0aa6fffe40c → e065084756b33d8b
  render::components::dialog::focused 120 40 paper mono 7146cb2e85b75125 → b98d7b7273a0e757
  render::components::dialog::focused 120 40 paper truecolor 1e3d201ae9d99644 → 9d9571be04a4c0be
  render::components::dialog::focused 40 10 junie mono 1d278875d97aa571 → 4dfe262b3ac49703
  render::components::dialog::focused 40 10 junie truecolor 26b98acf725e69b2 → 2b18e06440ec4e99
  render::components::dialog::focused 40 10 paper mono fa545df20bff6775 → 9c16f8153d3a62b7
  render::components::dialog::focused 40 10 paper truecolor 46350bbf66a5df82 → 74d7167d1083f680
  render::components::dialog::hovered 120 40 junie truecolor 5eeecbeef8c91ebe → f97b987bdc0dda4e
  render::components::dialog::hovered 120 40 paper truecolor 1444e3f0be1ea366 → 10b4f4c8055da652
  render::components::dialog::hovered 40 10 junie truecolor e31b313a51836a04 → 0d4777d50be8bda4
  render::components::dialog::hovered 40 10 paper truecolor 28ec0bb3e9085d2c → b9b70b6482d88f30
  render::components::dialog::pressed 120 40 junie mono 5ccef1c43fe1ab59 → 3f5dbd922edd1637
  render::components::dialog::pressed 120 40 junie truecolor 6126c36902705892 → 26ff02710a0cb783
  render::components::dialog::pressed 120 40 paper mono 1e15fad3fcd96a23 → fc38ebbf4b4a77a5
  render::components::dialog::pressed 120 40 paper truecolor ee25f336393bf144 → f0f9b921aa926aa3
  render::components::dialog::pressed 40 10 junie mono 96c7c7c04a22573d → 9c5b0be834361ad7
  render::components::dialog::pressed 40 10 junie truecolor 9ae04969711d60c8 → 77e6216bd4c20935
  render::components::dialog::pressed 40 10 paper mono d48511478f6f91eb → 047f00cacd93a4f5
  render::components::dialog::pressed 40 10 paper truecolor b4b8b36be4cc3e92 → 9e66540776422725
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
- moved:     177 keys:
  render::components::brand::disabled 120 40 junie mono 542069e6442a2293 → 7ebcdaa2bd323a7c
  render::components::brand::disabled 120 40 paper mono 4c59d9e6a3a4ce31 → 9e50413b3fc34bda
  render::components::brand::disabled 40 10 junie mono 2ddca9d38dbc3293 → 6b84e3d4400315fc
  render::components::brand::disabled 40 10 paper mono dee2fa93ee0777b1 → e213e779778043da
  render::components::brand::hovered 120 40 junie mono 7e7fbf3b99a99294 → 7ebcdaa2bd323a7c
  render::components::brand::hovered 120 40 junie truecolor 76939f92a53f033b → 9d2d64860f2f8023
  render::components::brand::hovered 120 40 paper truecolor bd5c69403efb3f97 → 0d8b5dc35b5705d7
  render::components::brand::hovered 40 10 junie mono fd1438c6acd80a14 → 6b84e3d4400315fc
  render::components::brand::hovered 40 10 junie truecolor 9af865d2b71bc53b → c8164be27cac5623
  render::components::brand::hovered 40 10 paper truecolor 3886143394806d97 → 6908170d98df53d7
  render::components::brand::pressed 120 40 junie mono 1fc69404d9ec1cb2 → 7ebcdaa2bd323a7c
  render::components::brand::pressed 120 40 paper mono 9e3f676394dd316c → 9e50413b3fc34bda
  render::components::brand::pressed 40 10 junie mono 024b6c44deaa0ab2 → 6b84e3d4400315fc
  render::components::brand::pressed 40 10 paper mono 08b13ce8991b04ec → e213e779778043da
  render::components::chip_bar::focused 40 10 junie mono 19b2b01e2d5e3aee → fff5116446505e3c
  render::components::chip_bar::focused 40 10 junie truecolor 2e7b0fb3b8bf2744 → 668e2c4ce5d503d8
  render::components::chip_bar::focused 40 10 paper mono c5f1dd6c13ac527a → 1ccb93fcbda7a6d8
  render::components::chip_bar::focused 40 10 paper truecolor 74437fb990dd86d0 → 0775d556d329cac0
  render::components::chip_bar::hovered 40 10 junie mono c16a4ad0871bee92 → fff5116446505e3c
  render::components::empty::editing 120 40 junie mono a5954830ab10a836 → 796461203dd06a7e
  render::components::empty::editing 120 40 paper mono 159c44e709f138aa → 506b0270c49589c2
  render::components::empty::editing 40 10 junie mono 73a78a9b97c4a0b6 → b0a749381c6792fe
  render::components::empty::editing 40 10 paper mono 1e84ace01f67d92a → b33395c3e2426242
  render::components::field::disabled 120 40 junie truecolor 85e79cecae9d1646 → e7a56e7918cb534a
  render::components::field::disabled 120 40 paper truecolor 14fe4d9f81a492f0 → 3dd81fd41bf3f994
  render::components::field::disabled 40 10 junie truecolor 782c78e216fd00c6 → 9a1518cb8d1b1e0a
  render::components::field::disabled 40 10 paper truecolor a967976c184a9050 → b20ea46edc236874
  render::components::field::selected 120 40 junie mono 1901d574eff61cda → df6c39650b0ade4c
  render::components::field::selected 120 40 paper mono 94f9b888f9e45fbe → 8cb42b3d2c97499c
  render::components::field::selected 40 10 junie mono 782f9a9cc9ee6dfa → 623ecdc1e6dcb26c
  render::components::field::selected 40 10 paper mono 85518d64f4e6935e → f5cc9fc0aaf4d2bc
  render::components::hint_bar::editing 120 40 junie mono 197dc165a722ba63 → 1a74082eb268df4b
  render::components::hint_bar::editing 120 40 paper mono 61ed307c72eee3c3 → 2398bb2e5e8f23eb
  render::components::hint_bar::editing 40 10 junie mono 8f236bbf0fb9ea63 → 3192a28a54d74b4b
  render::components::hint_bar::editing 40 10 paper mono a678a41a168a23a3 → f8cccce4b61521cb
  render::components::hint_bar::focused 120 40 junie mono 613495a851f8f589 → aaf7cf28e13e826a
  render::components::hint_bar::focused 120 40 paper mono dba3123d0856ca39 → b6ad74c672463592
  render::components::hint_bar::focused 40 10 junie mono e1888a0b600f4889 → fdd3f500925e3a6a
  render::components::hint_bar::focused 40 10 paper mono d49f56ab16488bd9 → d384912eb722bab2
  render::components::hint_bar::pressed 120 40 junie mono 71f3e50a1cd28877 → 197dc165a722ba63
  render::components::hint_bar::pressed 120 40 paper mono 5474ae4cb628d235 → 61ed307c72eee3c3
  render::components::hint_bar::pressed 40 10 junie mono 04ada5648bb8be97 → 8f236bbf0fb9ea63
  render::components::hint_bar::pressed 40 10 paper mono 5ee82ffc6f78e555 → a678a41a168a23a3
  render::components::list::disabled 120 40 junie mono 5c5f27303fa8adf5 → 164c36b18586726f
  render::components::list::disabled 120 40 junie truecolor 7702b43f4eda930a → ce3be201fd5894f6
  render::components::list::disabled 120 40 paper mono 83c3412c6d3274e1 → 73efdde5cdb3a6c5
  render::components::list::disabled 120 40 paper truecolor bf8cc2562eba8739 → 82e97d15fc758a0e
  render::components::list::disabled 40 10 junie mono 86dd18d3924968f5 → deac25911bc9718f
  render::components::list::disabled 40 10 junie truecolor c5d8f6f984a4af4a → 4e5dee5a5d3d3a76
  render::components::list::disabled 40 10 paper mono 12d2ab9c62b89201 → 447bcd08741e2165
  render::components::list::disabled 40 10 paper truecolor 682fd642796805b9 → 85d6dbd488cef51e
  render::components::list::focused 120 40 junie mono c82eb2ea27b199e1 → 164c36b18586726f
  render::components::list::focused 120 40 junie truecolor 82a91ae72dbb6f5f → ce3be201fd5894f6
  render::components::list::focused 120 40 paper mono cf4253e49708d4c7 → 73efdde5cdb3a6c5
  render::components::list::focused 120 40 paper truecolor 2b20bcba9fcb1768 → 82e97d15fc758a0e
  render::components::list::focused 40 10 junie mono 63512ef3e9965241 → deac25911bc9718f
  render::components::list::focused 40 10 junie truecolor 0194da55648030ff → 4e5dee5a5d3d3a76
  render::components::list::focused 40 10 paper mono af2da36dce26e4a7 → 447bcd08741e2165
  render::components::list::focused 40 10 paper truecolor 9747894746c94f28 → 85d6dbd488cef51e
  render::components::list::hovered 120 40 junie truecolor a1a0471bd768c936 → 96086a8fb7489f5a
  render::components::list::hovered 120 40 paper truecolor 9a7984a04053a64e → bfd8354e2d316762
  render::components::list::hovered 40 10 junie truecolor fafce092e21183f6 → 95c30a93fad87dda
  render::components::list::hovered 40 10 paper truecolor 6ccee43f083e391e → d2a22c49a21f3a72
  render::components::list::pressed 120 40 junie mono 58d1cbccb78dd4f7 → cb5e7c9046398305
  render::components::list::pressed 120 40 junie truecolor 4f8509df4dd3d6a3 → d0e89104e36aa2ea
  render::components::list::pressed 120 40 paper mono 4a88db0dfb7b711b → f2ce0cf198981d97
  render::components::list::pressed 120 40 paper truecolor e608c80c5d2853e4 → 3e524d83e9dfe491
  render::components::list::pressed 40 10 junie mono fb08873b8e58e857 → 7f93e89c2dddc765
  render::components::list::pressed 40 10 junie truecolor 0cfde54fb5f36ba3 → 7a673016b0ed65aa
  render::components::list::pressed 40 10 paper mono 2eccd01dd15f4bfb → d3717338bef62977
  render::components::list::pressed 40 10 paper truecolor 174d354087cf3c44 → d5f14e0f54f98a11
  render::components::meter::editing 120 40 junie mono 505b40d8b1e4656a → 664372a5c0eb31d2
  render::components::meter::editing 120 40 paper mono d9ca112b604ad0a2 → d675ab88d26b023a
  render::components::meter::editing 40 10 junie mono cadbc9890f6f4f42 → d78165edbfbe84da
  render::components::meter::editing 40 10 paper mono 8293158f29f6fe32 → 9a34a8b1841a59ca
  render::components::meter::focused 120 40 junie mono 70423261a7bd78f5 → fab1931d8ce0ae06
  render::components::meter::focused 120 40 paper mono b5e594f26fe1bd45 → e4f6dea05001cc56
  render::components::meter::focused 40 10 junie mono 0fe3c151b92cb09d → 5c7403a5b598074e
  render::components::meter::focused 40 10 paper mono 79a468f3745ebc65 → c662525e6a93f976
  render::components::meter::pressed 120 40 junie mono d64ec4302df10c1d → 505b40d8b1e4656a
  render::components::meter::pressed 120 40 paper mono 690dc1b4fdd43815 → d9ca112b604ad0a2
  render::components::meter::pressed 40 10 junie mono 1412560a60380fb5 → cadbc9890f6f4f42
  render::components::meter::pressed 40 10 paper mono fcdd70aa2e175ec5 → 8293158f29f6fe32
  render::components::progress_bar::editing 120 40 junie mono 2db6b19ba7a37a19 → 6a4f7a72886db1b1
  render::components::progress_bar::editing 120 40 paper mono 5783d256a3809c35 → 873dab61602ad3cd
  render::components::progress_bar::editing 40 10 junie mono 6fb9de54d8806411 → bd9ca2fdffa8f379
  render::components::progress_bar::editing 40 10 paper mono 8cf8dab6400b1af5 → 685c8af2d9395a8d
  render::components::progress_bar::focused 120 40 junie mono 4b4dacffff053913 → 1da207d050ccbe00
  render::components::progress_bar::focused 120 40 paper mono 6ad98f1f563de507 → 7eba31cf17200e64
  render::components::progress_bar::focused 40 10 junie mono 24e86ec5a1af2c1b → 640c9e6d0236a448
  render::components::progress_bar::focused 40 10 paper mono d91e30c3c835c8f7 → 0e0ca48ae46cc5a4
  render::components::progress_bar::pressed 120 40 junie mono 870c75bbeae28382 → 2db6b19ba7a37a19
  render::components::progress_bar::pressed 120 40 paper mono 6935627f9b728f3e → 5783d256a3809c35
  render::components::progress_bar::pressed 40 10 junie mono 4add7e04e4bb701a → 6fb9de54d8806411
  render::components::progress_bar::pressed 40 10 paper mono 0f469f6f190038ce → 8cf8dab6400b1af5
  render::components::radio_group::default 120 40 junie mono 8d0ca4e2991bd3b3 → 23253cfc778734b7
  render::components::radio_group::default 120 40 junie truecolor 4cfffc6ffd36528d → 57c4e8ba8931346e
  render::components::radio_group::default 120 40 paper mono cddac78e37c431e1 → ec7745f70f85f805
  render::components::radio_group::default 120 40 paper truecolor cab52f27bce68f0c → d080a80f269c3da2
  render::components::radio_group::default 40 10 junie mono 68c8a89b7d2e9d73 → 783665ea3658b177
  render::components::radio_group::default 40 10 junie truecolor a6505c92890a94cd → 3c36518edf6682ae
  render::components::radio_group::default 40 10 paper mono 2a5ce6267b6fe421 → dc874cf17df524c5
  render::components::radio_group::default 40 10 paper truecolor 787b9fdc0632f1cc → 1ad944db4e336262
  render::components::radio_group::disabled 120 40 junie mono 9ca1d569771f97f9 → 0b3191ebe74270d5
  render::components::radio_group::disabled 120 40 junie truecolor 1251e94d115ace12 → 1fae8745ccbb85d6
  render::components::radio_group::disabled 120 40 paper mono 749bc8e6f76a4bd5 → fb9c0ebf7b8b46e9
  render::components::radio_group::disabled 120 40 paper truecolor 19c5f3c3e2c3e84d → 85f5f2ff0f013059
  render::components::radio_group::disabled 40 10 junie mono b4b65f522beec779 → ac10dc1dca9bd455
  render::components::radio_group::disabled 40 10 junie truecolor 7977c854937a4192 → 7ed4912d9a7c0756
  render::components::radio_group::disabled 40 10 paper mono 24f2cd193c475a15 → 37e747dbe508c429
  render::components::radio_group::disabled 40 10 paper truecolor f8859e7bb6df418d → 411c0befb9a22e99
  render::components::radio_group::editing 120 40 junie mono 8d0ca4e2991bd3b3 → 23253cfc778734b7
  render::components::radio_group::editing 120 40 junie truecolor 4cfffc6ffd36528d → 57c4e8ba8931346e
  render::components::radio_group::editing 120 40 paper mono cddac78e37c431e1 → ec7745f70f85f805
  render::components::radio_group::editing 120 40 paper truecolor cab52f27bce68f0c → d080a80f269c3da2
  render::components::radio_group::editing 40 10 junie mono 68c8a89b7d2e9d73 → 783665ea3658b177
  render::components::radio_group::editing 40 10 junie truecolor a6505c92890a94cd → 3c36518edf6682ae
  render::components::radio_group::editing 40 10 paper mono 2a5ce6267b6fe421 → dc874cf17df524c5
  render::components::radio_group::editing 40 10 paper truecolor 787b9fdc0632f1cc → 1ad944db4e336262
  render::components::radio_group::focused 120 40 junie mono ee13d94382f12131 → 23253cfc778734b7
  render::components::radio_group::focused 120 40 junie truecolor 7000b9777c32900a → 57c4e8ba8931346e
  render::components::radio_group::focused 120 40 paper mono cc0a53514e52334b → ec7745f70f85f805
  render::components::radio_group::focused 120 40 paper truecolor d342f6697e0ed372 → d080a80f269c3da2
  render::components::radio_group::focused 40 10 junie mono f46e5a6a2edb4231 → 783665ea3658b177
  render::components::radio_group::focused 40 10 junie truecolor abda211333e8c53a → 3c36518edf6682ae
  render::components::radio_group::focused 40 10 paper mono b186aeef57c12beb → dc874cf17df524c5
  render::components::radio_group::focused 40 10 paper truecolor f83b950099c85582 → 1ad944db4e336262
  render::components::radio_group::hovered 120 40 junie mono 8d0ca4e2991bd3b3 → 23253cfc778734b7
  render::components::radio_group::hovered 120 40 junie truecolor af89cdd0ddff5393 → fec6ff03181c2f40
  render::components::radio_group::hovered 120 40 paper mono cddac78e37c431e1 → ec7745f70f85f805
  render::components::radio_group::hovered 120 40 paper truecolor 11f985a2e69f5be0 → e90e83af3dd1bbc2
  render::components::radio_group::hovered 40 10 junie mono 68c8a89b7d2e9d73 → 783665ea3658b177
  render::components::radio_group::hovered 40 10 junie truecolor 7f905945e0ed5e13 → aa8c6792dc7ee680
  render::components::radio_group::hovered 40 10 paper mono 2a5ce6267b6fe421 → dc874cf17df524c5
  render::components::radio_group::hovered 40 10 paper truecolor 1942c9ab971a1ce0 → fc7ed7d8f4f02a82
  render::components::radio_group::pressed 120 40 junie truecolor 3ec363db305aba2a → 7c45f61130c600e6
  render::components::radio_group::pressed 120 40 paper truecolor b74398b48260115b → b74072a1943741bb
  render::components::radio_group::pressed 40 10 junie truecolor 00d9ed0a89bbf0da → d9de17746e8da6a6
  render::components::radio_group::pressed 40 10 paper truecolor c445079c40047ecb → 9d9612086a084deb
  render::components::spinner::disabled 120 40 junie mono 47e80e5becda6b47 → a6f2e5979c8e4d8f
  render::components::spinner::disabled 120 40 paper mono 3712d6c2925afc31 → cde3224c2f4427f7
  render::components::spinner::disabled 40 10 junie mono 6ec8511427004947 → 3c96bff36c20218f
  render::components::spinner::disabled 40 10 paper mono 59a46a67b7caa5b1 → 3a628723108a61f7
  render::components::spinner::focused 120 40 junie mono 11cf5f159990aea8 → a6f2e5979c8e4d8f
  render::components::spinner::focused 120 40 paper mono eed43ed2309f6630 → cde3224c2f4427f7
  render::components::spinner::focused 40 10 junie mono 313abf3278399528 → 3c96bff36c20218f
  render::components::spinner::focused 40 10 paper mono 3e237ad9339230b0 → 3a628723108a61f7
  render::components::spinner::pressed 120 40 junie mono 11cf5f159990aea8 → a6f2e5979c8e4d8f
  render::components::spinner::pressed 120 40 paper mono eed43ed2309f6630 → cde3224c2f4427f7
  render::components::spinner::pressed 40 10 junie mono 313abf3278399528 → 3c96bff36c20218f
  render::components::spinner::pressed 40 10 paper mono 3e237ad9339230b0 → 3a628723108a61f7
  render::components::status_bar::disabled 120 40 junie mono 6fde7ed8cd7b6dbd → dba9eaec3520c297
  render::components::status_bar::disabled 120 40 paper mono 62d3682b5f66e245 → 41896645db38db73
  render::components::status_bar::disabled 40 10 junie mono 970f612c8771fdfd → c06e4f5f9ad85657
  render::components::status_bar::disabled 40 10 paper mono e3e7e5d5a20cc485 → 6800bbd9e5610313
  render::components::status_bar::editing 120 40 junie mono 52be1d0861e6320f → bd19c6ccb9b63237
  render::components::status_bar::editing 120 40 paper mono b27ab00101de2f83 → 7920bb18ed9d09ab
  render::components::status_bar::editing 40 10 junie mono 74cd4ae2843dda4f → 81e080b38aa0ee77
  render::components::status_bar::editing 40 10 paper mono a0e61d38bf975c63 → ced36ea09e518bcb
  render::components::tabs::disabled 120 40 junie mono 383875a51445a582 → 03263f48effb1240
  render::components::tabs::disabled 120 40 paper mono 4034eb30c2d2df46 → ad59d0d63e9883e4
  render::components::tabs::disabled 40 10 junie mono d48c88f61b9cf638 → 8575598b028539ba
  render::components::tabs::disabled 40 10 paper mono ebdcc0ecece3cf70 → a26add1c3797f552
  render::components::tabs::focused 120 40 junie mono dbf77ccc2cf211a1 → 03263f48effb1240
  render::components::tabs::focused 120 40 junie truecolor 9ecda219f913871c → 345a10235d65f11f
  render::components::tabs::focused 120 40 paper mono 7abfc94e7df50945 → ad59d0d63e9883e4
  render::components::tabs::focused 120 40 paper truecolor c8e58521b7b83935 → 9aa17c11b0135f9e
  render::components::tabs::focused 40 10 junie mono 2d01bc2342f6ec25 → 8575598b028539ba
  render::components::tabs::focused 40 10 junie truecolor ad29ee72c476b517 → 870f0d29c4b57860
  render::components::tabs::focused 40 10 paper mono bb77aa02abb1525d → a26add1c3797f552
  render::components::tabs::focused 40 10 paper truecolor 86a9012d0bb33bd6 → 2eb100378a3b3c43
  render::components::tabs::hovered 120 40 junie mono f6696925141d5026 → 03263f48effb1240
  render::components::tabs::hovered 120 40 junie truecolor 8ab1d880f7c1a627 → c06630a519f23619
  render::components::tabs::hovered 120 40 paper truecolor 6cd1f2cd84322fe9 → e4e9b986e64b867c
  render::components::tabs::hovered 40 10 junie mono b36480355e85f24c → 8575598b028539ba
  render::components::tabs::hovered 40 10 junie truecolor 97f8e11cb3038468 → 057df2cc503ba26e
  render::components::tabs::hovered 40 10 paper truecolor db730884c57255d0 → 016928830f8890ed
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
- added:     936 keys: showcase 704 (22×4×2×4), tablepro 168 (21×2×2×2), jackin-preview 64 (8×2×2×2)
             `{Buttons,Chips & selects,Chrome,Data grid,Dialogs,Editable tables,Editor,Forms,Inputs,Lists,Overview,Panels,Pickers,Progress,Scrolling,Settings,Sidebars,Tables,Task runner,Terminal,Text areas,Trees} {80 24,100 30,120 40,160 50} {junie,paper} {truecolor,256,16,mono}`
             `{completion-popup,connections,connections-failed,error-result,explain-plan,explorer-focused,filter-editor,grid-cell-editing,help-dialog,history-tab,maximised-tab,pending-change-bar,query-editing,quick-switcher,results-grid,safe-mode-picker,safety-dialog-typed-ack,structure-view,tab-list-picker,table-grid,workbench-default} {80 24,120 40} {junie,paper} {truecolor,mono}`
             `{accounts-mixed,capsule-multi,first-use,hard-cases,launch-failure,launch-running,outro-last,returning} {100 30,120 40} {junie,paper} {truecolor,mono}`
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

---

## Review status — Slice 4 component matrix, independent visual review (2026-09-05)

**Result: FAIL. No bless was performed, and none is authorized.**

- **Scope reviewed.** The **640** review frames for the Slice-4 component matrix, at HEAD
  `26913cc`, by a fresh read-only `opus-analyst` reviewer who did not generate the baselines.
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
