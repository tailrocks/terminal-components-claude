# Visual changes ledger

**What this is.** The ledger `COMPONENT_ARCHITECTURE.md` §20.10 requires and `xtask bless-guard` (§16.3) reads. No baseline file (`crates/tui/tests/baselines/components.txt`, `apps/*/tests/baselines/*.txt`, `perf_baseline.txt` hit counts) may be regenerated without an entry here that names a §20.10 item **and** a capture path under `shots/`. Every entry classifies a difference as *intended* (matches the §20.10 item), *fix* (a demonstrated defect in the old output) or *regression* (must be fixed, never blessed).

**Order, fixed (review A14, §21 item 30): change → capture → classify → bless.**

1. **Change** — land the code change on the working tree.
2. **Capture** — `tools/capture.sh` / `xtask capture-matrix` writes the before/after captures into `shots/`; digest tests go red.
3. **Classify** — add or extend the entry under the matching §20.10 item below: capture paths, the affected tests/baseline lines, and the classification with its reason.
4. **Bless** — `BLESS=1 cargo test --workspace --test render --test visual` (or `PERF_BLESS=1` for hit counts). `xtask bless-guard` runs in CI on the committed tree and fails the commit if step 3 is missing.

A capture cannot exist before the change, so `bless-guard` never runs locally against an unchanged tree. No baseline is regenerated because a test failed; the classification comes first.

**Entry format** (one per affected surface):

```
- surface:   <app>/<page or component>/<state> @ <w>x<h> / <theme> / <color level>
- captures:  shots/<before>.png  →  shots/<after>.png
- tests:     <baseline file>:<line or name>, <test names>
- class:     intended | fix | regression
- reason:    <one sentence tying it to the §20.10 item>
```

---

## Item 1 — Mono legibility fallbacks (§11.4, §21 item 25)

### 1a — mono `DISABLED` gains `DIM` on `FIELD`/`TEXT` and stops tinting the foreground into the background (§28 P6)

**§20.10 classification line:** mono DISABLED gains DIM on FIELD/TEXT and stops
tinting the foreground into the background.

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

`PLACEHOLDER` needs no rule (it inherits the `FIELD` fill's modifiers per cell)
and `CONTAINER` needs none (a text control fills `FIELD`). The new rules are
declared **before** the `ERROR` rules, so `ERROR`'s `UNDERLINED` is not erased
by `remove(Modifier::all())`.

**Second change in the same table pass:** `Tabs` now paints §11.4's mono
`PRESSED` bracket (`[label]`) into the pad cells the tab already reserves —
geometry is identical — because the row fn paints the tab label through
`RowUi`, which cannot consult the `LABEL` glyph slot the way `Button::draw`
does. Without it a pressed tab and a focused tab are the same picture without
colour.

```
- surface:   tui-next/{text_input,field,list,button,tabs,dialog}/disabled and tabs/pressed
             @ {120x40, 40x10} / {junie, paper} / mono   (truecolor cells are untouched:
             mono rules are appended only at `ColorLevel::Mono`)
- captures:  none under `shots/` — this matrix is a headless digest matrix, not a
             running app: `tools/capture.sh` drives a terminal session and cannot
             address a `Scene`. The reviewable artifact is the digest diff of
             `crates/tui/tests/baselines/components.txt` in the same commit; the
             painted **text** is byte-identical in every moved cell (the panic
             output shows it), so the whole difference is style.
- tests:     crates/tui/tests/baselines/components.txt (mono lines only),
             render::components::{text_input,field,list,button,tabs,dialog}::disabled,
             render::components::tabs::pressed
- moved:     20 lines, every one `mono` (`git diff crates/tui/tests/baselines/components.txt`):
```
  render::components::button::disabled 120 40 junie mono 023bd60f5b1ae845 → d20bb906fcfe3dd1
  render::components::button::disabled 40 10 junie mono 15af984dfc54c7c5 → bfe7ea91b76bd751
  render::components::dialog::disabled 120 40 junie mono 3162b7d5bbf2a5f5 → 2d6a3cd4c020d7e5
  render::components::dialog::disabled 40 10 junie mono 03fb01cee70da7f5 → 3ef081624f0f1fa5
  render::components::field::disabled 120 40 junie mono ee232d54d927e1b4 → 399b0a5bc31c9d66
  render::components::field::disabled 120 40 paper mono 2ce10fc98c3c6524 → e0d02bbbddbfe054
  render::components::field::disabled 40 10 junie mono bb701f176484fbb4 → d8fb0563075c66a6
  render::components::field::disabled 40 10 paper mono 8850862fa2588ec4 → 55505cb874284414
  render::components::list::disabled 120 40 junie mono 8ef3444eee52116d → 5c5f27303fa8adf5
  render::components::list::disabled 40 10 junie mono 6dc1b708da3a1a6d → 86dd18d3924968f5
  render::components::tabs::disabled 120 40 junie mono 35a3a27d0daf3a0c → 383875a51445a582
  render::components::tabs::disabled 40 10 junie mono f3715f8ca6758086 → d48c88f61b9cf638
  render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747 → 8531aef99ed82a7c
  render::components::tabs::pressed 120 40 paper mono ca497a2a34358f51 → 57bc3c9afc387ab6
  render::components::tabs::pressed 40 10 junie mono 8a4bc1549eca3857 → a1ca30a076849608
  render::components::tabs::pressed 40 10 paper mono dfaa198c140af319 → 35b0ee5d62f85452
  render::components::text_input::disabled 120 40 junie mono f32a4730f22cd73a → 1db1055714d0b91e
  render::components::text_input::disabled 120 40 paper mono 44f4c6f88a4aec46 → d36c9e02c6aca8be
  render::components::text_input::disabled 40 10 junie mono 8725f1e9f6355d3a → 1e91d7e4d1cdcfde
  render::components::text_input::disabled 40 10 paper mono d63984c2d12b1c26 → 5257f3bad4cc42be
```
- class:     fix
- reason:    §20.10 item 1 (mono legibility fallbacks). The old output was
             unreadable at `Mono` (black on black) and gave a text control's
             disabled state no signal at all; the new output is `DIM` over the
             primary foreground, which §16.2 case 9 can see and a reader can
             read. `conformance::text_input::mono_states_are_distinguishable`
             now keeps `DISABLED` instead of narrowing it away (MA-8).
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
