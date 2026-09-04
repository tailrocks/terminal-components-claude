# Visual changes ledger

**What this is.** The ledger `COMPONENT_ARCHITECTURE.md` §20.10 requires and `xtask bless-guard` (§16.3) reads. No baseline file (`crates/tui/tests/baselines/components.txt`, `apps/*/tests/baselines/*.txt`, `perf_baseline.txt` hit counts) may be regenerated without an entry here that names a numbered §20.10 item, **accounts for every baseline key the diff moved or added**, and names the reviewable artefact — a capture path under `shots/` for a baseline produced by a running application, **or** the frame-text dump for a baseline produced headlessly by a `Scene`, named explicitly (§16.3 as amended by §36). Every entry classifies a difference as *intended* (matches the §20.10 item), *fix* (a demonstrated defect in the old output) or *regression* (must be fixed, never blessed).

**Order, fixed (review A14, §21 item 30): change → capture → classify → bless.**

1. **Change** — land the code change on the working tree.
2. **Capture** — for an application baseline, `tools/capture.sh` / `xtask capture-matrix` writes the before/after captures into `shots/`. For a headless `Scene` matrix there is no capture and there can be none (`tools/capture.sh` drives a terminal session and cannot address a `Scene`, §36): the artefact is the frame text the failing run prints. Digest tests go red either way.
3. **Classify** — add or extend the entry under the matching §20.10 item below: the reviewable artefact, the affected tests/baseline lines, the moved and added keys, and the classification with its reason.
4. **Bless** — `BLESS=1 cargo test --workspace --test render --test render_components --test visual` (or `PERF_BLESS=1` for hit counts). `xtask bless-guard` is specified in §16.3 and **is not implemented yet**; until it lands this ledger is convention enforced by review.

A capture cannot exist before the change, so `bless-guard` never runs locally against an unchanged tree. No baseline is regenerated because a test failed; the classification comes first.

**Entry format** (one per affected surface). Every field of an entry lives inside **one** fenced block, so a reader that strips fences loses neither the classification, nor the citation, nor the evidence:

```
- surface:   <app>/<page or component>/<state> @ <w>x<h> / <theme> / <color level>
- captures:  shots/<before>.png  →  shots/<after>.png
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
- surface:   tui-next/tabs/pressed @ {120x40, 40x10} / {junie, paper} / mono
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
- surface:   tui-next/field/disabled @ {120x40, 40x10} / {junie, paper} / mono
- captures:  none under `shots/` — this is a headless `Scene` matrix. The reviewable
             artefact is the frame-text dump printed by the no-BLESS
             `render::components::field::disabled` run; its first moved frame shows
             the required marker cell cleared. `tools/capture.sh` cannot address a
             `Scene`, and no separate capture file is in this task's ownership.
- tests:     crates/tui/tests/baselines/components.txt (mono lines only),
             render::components::field::disabled
- moved:     4 keys, every one `mono`:
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
- surface:   tui-next/{text_input,field,list,button,tabs,dialog}/disabled
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
- moved:     12 lines, every one `mono` (`git diff crates/tui/tests/baselines/components.txt`):
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
- surface:   tui-next/{button,dialog}/pressed @ {120x40, 40x10} / {junie, paper} / mono
- captures:  none under `shots/` — headless `Scene` matrix (see 18a). The reviewable
             artefact is the frame-text dump printed by the no-BLESS
             `render::components::{button,dialog}::pressed` runs; it shows the
             bracket in the reserved padding rather than inside the label run.
- tests:     crates/tui/tests/baselines/components.txt (mono lines only),
             render::components::button::pressed, render::components::dialog::pressed,
             conformance::button::mono_states_are_distinguishable
- moved:     8 keys, every one `mono`:
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
- surface:   tui-next/{text_area,select,radio_group,checkbox,toggle,chip_bar,
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
- surface:   tui-next/{hint_bar,meter,progress_bar}/disabled
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
- moved:     22 keys (HintBar 8, Meter 6, ProgressBar 8; 12 truecolor, 10 mono):
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
