# Adjudication Q — three residuals from the Adjudication P code pass

**Status:** accepted and applied for Q1–Q3 in the current tree. The source review was read-only at `HEAD 0f66160`; implementation follow-up evidence is recorded below. Facts marked **[F]** describe that review snapshot; later execution and source corrections are recorded as applied evidence. One premise handed to the review (Q1's byte-identity observation) could not be reproduced statically — R1 is now closed by the three-phase execution record. This status does not claim that the concurrent Slice 4 wave or the full workspace gate is green.

---

## Collected facts

**[F1] `RowUi` honours the resolved glyph slot in exactly one of five places.** `RowUi::style_of` returns `r.style` and discards `Resolved.glyph`/`size`/`align` (`crates/tui/src/collection/rowui.rs:112-118`). `label`, `label_patched`, `label_spans`, `label_fmt` all route through it (`:156-159`, `:162-168`, `:191-196`, `:199-211`). `marker(g)` resolves `Part::MARKER`, then paints the **caller's** `g` and ignores `r.glyph` (`:121-126`). `part(p, w)` hands `CellUi` only `r.style` (`:256-271`). Only `gutter()` matches on `r.glyph` (`:130-146`).

**[F2] The two components that pass mono `PRESSED` today do it themselves, not through `RowUi`.** `Button::draw` matches `ls.glyph == Some(GlyphRole::PressLeft)` and paints `PressLeft` + label + `PressRight` **inside the text run**, shifting the label right by one (`crates/tui/src/components/button.rs:402-416`). `Tabs::draw` paints the same pair into the tab's two **pad** cells — `tab.x` and `tab.x + 1 + label_w`, where `tab_w = 1 + label_w + 1 + close_w` (`crates/tui/src/components/tabs.rs:691-702`, `:713-728`) — so the tab's geometry is unchanged.

**[F3] `List` never delegates a glyph-carrying part to `RowUi`.** It fills `Part::CONTAINER` with the row's flags (`crates/tui/src/components/list.rs:755-763`), paints `GUTTER` and `MARKER` itself with `match g.glyph { Some(..) => ui.glyph, None => ui.fill }` (`:764-789`), and gives `RowUi` only the label/meta remainder at `row.x + 3` (`:790-798`). `ListCase` keeps `PRESSED` and is green (`crates/tui/tests/conformance.rs:519-530`), so for a list-shaped row `CONTAINER`'s reverse + `BOLD` alone satisfies case 9.

**[F4] `LABEL` carries exactly one glyph binding in the mono table.** Of the 18 rules, the glyph-binding ones are `GUTTER/FOCUSED`, `MARKER/{SELECTED,CHECKED,DISABLED,ERROR,WARNING,DIRTY}`, `GUTTER/DISABLED`, `RULE/ACTIVE` — and `LABEL/PRESSED → PressLeft` (`crates/tui/src/theme/downgrade.rs:257-383`; `MONO_RULES_PER_FAMILY = 18` at `:386`).

**[F5] §11.4 already mandates the bracket.** The `PRESSED` row reads `CONTAINER` reverse + `BOLD` "**and** `Part::LABEL` bracketed with `GlyphRole::PressLeft` / `GlyphRole::PressRight`" (`COMPONENT_ARCHITECTURE.md:1041`). §11.4 never says *who* paints it.

**[F6] The Slice-4 shapes split two ways.** `ChipBar::PARTS = [CONTAINER, LABEL, CLOSE, OVERFLOW]` (`COMPONENT_ARCHITECTURE.md:2621`) — tab-shaped, pads and a close cell, no `MARKER`/`GUTTER`. `RadioGroup::PARTS = [CONTAINER, GUTTER, MARKER, LABEL]` (`:2607`) — list-shaped. Menu items, steps and picker rows are list-shaped.

**[F7] `Fixture::state_override` is `pub` (`crates/tui-testing/src/conformance/mod.rs:76`), `status` is `pub` (`:83`), and `force` is the only writer that pairs them (`:110-129`).** Case 9 is the only caller (`driver.rs:439`). Every case reads the field (`crates/tui/tests/conformance.rs:387`, `:396`, `:450`, `:552`). §16.2's declared `Fixture` (`COMPONENT_ARCHITECTURE.md:1782-1786`) lists **none** of `state_override`, `patch`, `secret`, `status` — the doc's type is three amendments stale.

**[F8] The named grep can never exit 0, and its intent is not satisfied today either.** `rg -n 'fn mono_states' … -A2` yields, per case, the signature line (filtered: contains `]`), the `const STATES: …` line (filtered: contains `const`), and `StateFlags::empty(),` — which matches none of `///|//|const|&STATES|STATES:|\]|\}` and always survives, so `rg -v` exits 0 and `!` fails (`COMPONENT_ARCHITECTURE.md:6173`, `:6202`). Separately, counting drops against the ten-state default (`conformance/mod.rs:138-149`): `TabsCase` drops **ERROR, WARNING, EDITING, BUSY, ACTIVE** but documents only `ACTIVE` (`conformance.rs:618-634`); `FieldCase` drops **SELECTED, PRESSED, WARNING, BUSY, ACTIVE** and documents none (`:406-419`); `TextInputCase` drops **SELECTED, PRESSED, WARNING, ACTIVE** and documents none (`:344-358`); only `ListCase` names all three of its drops (`:513-530`). §28.6's claim that the intent "is currently satisfied by doc comments on all five cases" is **false for four of the five**.

---

## Q1 — `Tabs` mono `PRESSED`

**Decision: (a), confirmed — with the idiom named, and with the `RowUi` question answered by splitting it.**

**(a) is not a new affordance; it is compliance with §11.4 as already written [F5].** The `PRESSED` row requires the bracket *in addition to* the `CONTAINER` rule. `Button` discharges it; `Tabs` now discharges it. What was missing from the spec is the sentence saying who paints it, and Adjudication P's table assumed every component paints its own label the way `Button` does.

**R1 is proven in three phases.** With the Tabs bracket block enabled, `conformance::tabs::mono_states_are_distinguishable` exited `0`. With only `tabs.rs:719–728` disabled, the same test exited nonzero because `TabsCase` reported mono `PRESSED`/`FOCUSED` equality. Restoring that block returned the test to exit `0`. The `CONTAINER` rule's `BOLD` alone therefore did not distinguish this pair; the reserved-cell bracket is independently required.

**Why the bracket cannot move into `RowUi` (rejects (c) as stated).** The bracket needs two columns that are not the label's. `Button` has a gutter and a trailing pad; a tab has two pad cells [F2]; a `List`/`Tree`/`Props`/`Grid` row has none — `RowUi::label` fills the whole remainder (`rowui.rs:170-188`). Teaching `label` to bracket would take two content columns from every pressed row and pull the ellipsis in by two — a mono fallback changing geometry, which §28.6 already rejected in terms ("a bracket glyph steals two columns … a mono fallback must never change geometry", `COMPONENT_ARCHITECTURE.md:6171`). `RowUi` has no information with which to reserve those columns; only the component that laid out the row does. This is structurally the same conclusion §28.6(b) reached for the spinner: **the theme rule states the affordance; the component that owns the cells paints it.**

**Why (b) is rejected.** A `(Part::TAB, PRESSED)` modifier rule is per-family-part, so it does not scale — chips would need `(CHIP, PRESSED)`, steps `(STEP, PRESSED)`, and each new part needs a modifier not already spoken for by `FOCUSED` (BOLD), `DISABLED` (DIM) or `ERROR`/`EDITING` (UNDERLINED). Worse, it makes `PRESSED` mean something different for a tab than for a button, which destroys the single mono vocabulary case 9's pairwise comparison rests on. It also hides that `CONTAINER`'s rule already fires.

**Is `RowUi` ignoring the glyph slot itself a defect? Yes for `marker` and `part`; no for `label`.** A part that owns a **cell** must paint `Resolved.glyph` when `Some` — that is what `gutter()` does [F1], what `List` does for its own gutter/marker [F3], and what `Tabs` does for `CLOSE` (`tabs.rs:732`). `RowUi::marker(g)` violates it outright: it resolves `Part::MARKER` and then throws the answer away, so every mono `MARKER` rule (`SELECTED→Chosen`, `CHECKED→Checked`, `ERROR→Error`, `WARNING/DIRTY→Dirty`, `DISABLED→` clear) is inert for any component that uses it [F1][F4]. `RowUi::part` drops `glyph`, `size` and `align` for the same reason. `Part::LABEL` is different in kind: it is a **text run with no reserved glyph cell**, so "paint the resolved glyph" has no defined placement there, and §12.2 currently says nothing either way (`COMPONENT_ARCHITECTURE.md:1088-1122`).

**The `marker`/`part` correction is applied.** `Resolved.glyph` and `PartMetrics.glyph` are now `Slot<GlyphRole>`, so `Slot::Clear` remains distinct from `Slot::Inherit`; cell-owning `RowUi` methods honor `Inherit`, `Set` and `Clear` without changing reserved geometry. The existing callers in `crates/tui/examples/07_borrowed_rows.rs` and `crates/tui/examples/08_dynamic_tabs.rs` are live regression coverage. Label methods remain text-run methods with no automatic glyph placement.

**Scaling to Slice 4** (this is the answer the question asks for): chips are tab-shaped [F6] and use the same reserved-pad bracket; menu items, steps, picker rows and radio rows are list-shaped and need **nothing** — `CONTAINER`'s reverse + `BOLD` already carries `PRESSED`, as `ListCase` proves [F3]. So the rule is one sentence, not five implementations.

**Applied geometry rule.** The shared bracket helper takes two already-reserved cells and does not resize, shift, measure or truncate the label. `Button` passes its gutter and trailing pad; `Tabs` (and ChipBar) pass their existing pads. The label rectangle, total width and close-cell geometry remain unchanged.

---

## Q2 — `Fixture::state_override`

**Decision: make the field private, with a read accessor. `force` becomes the only way to set it.**

Keeping it public with a documented invariant plus a check is a symptom patch: the invariant is enforceable only by a reviewer noticing an assignment, and the enabling condition — a settable field that leaves `status` stale — survives. Making it private removes the enabling condition, which is the whole of P6(iii): a forced state that props do not implement is a case proving nothing [F7].

```rust
// crates/tui-testing/src/conformance/mod.rs
pub struct Fixture {
    pub disabled: bool, pub read_only: bool, pub theme: Theme,
    pub color: ColorLevel, pub area: Rect, pub rows: Vec<FixtureRow>,
    pub patch: Option<(Part, StylePatch)>, pub secret: Option<&'static str>,
    /// Forced state (A11) — private, because setting it without the props
    /// the state implies re-opens the P6(iii) gap. Write it with
    /// [`Fixture::force`]; read it with [`Fixture::forced`].
    state_override: StateFlags,
    /// Readiness, derived from `force`. Private for the same reason.
    status: Status,
}
impl Fixture {
    #[must_use] pub const fn forced(&self) -> StateFlags { self.state_override }
    #[must_use] pub const fn status(&self) -> Status { self.status }
    #[must_use] pub fn force(mut self, s: StateFlags) -> Self { /* unchanged body */ }
}
```

`status` goes private with it — a case that sets `status` alone would reproduce the mirror-image gap (props busy, theme flags not). All existing uses are reads [F7] and become `f.forced()` / `f.status()`; `Default` is in the same module and is unaffected. `Fixture` keeps `Clone + Debug`; nothing constructs it by struct literal outside the crate.

**Rejected.** *Public + documented invariant + a `debug_assert` in the driver* — fires only on the path that happens to run, says nothing at review time, and leaves the field settable. *A `Forced(StateFlags, Status)` newtype field, still public* — same hole one indirection deeper. *An `xtask` boundary rule forbidding `state_override =`* — a text check standing in for a language feature that is already available and free here.

---

## Q3 — the acceptance grep

**Decision: withdraw the grep. Replace it with a declared reason that case 9 checks.** A grep cannot read prose, and this one is doubly wrong: it can never pass [F8], and the property it was standing in for is not true today either — four of five narrowing cases document fewer states than they drop [F8].

```rust
// crates/tui-testing/src/conformance/mod.rs — on `trait Conformance`
/// Why `mono_states()` narrows [`DEFAULT_MONO_STATES`], naming **every**
/// state it drops. Empty exactly when nothing is narrowed.
fn mono_narrowing_reason() -> &'static str { "" }
```

```rust
// driver.rs, inside `mono_states_are_distinguishable`, after the two
// existing narrowing assertions (:417-434):
let dropped: Vec<StateFlags> = super::DEFAULT_MONO_STATES
    .iter().copied().filter(|s| !states.contains(s)).collect();
let why = C::mono_narrowing_reason();
assert_eq!(dropped.is_empty(), why.is_empty(),
    "{}: mono_narrowing_reason() must be non-empty exactly when mono_states() narrows", C::NAME);
for s in &dropped {
    for (name, _) in s.iter_names() {
        assert!(why.contains(name),
            "{}: mono_states() drops {name}, and mono_narrowing_reason() does not say why", C::NAME);
    }
}
```

It lives in case 9 rather than as a 21st case, so the "20-case matrix" language, the §16.2 table and `every_named_test_exists` are all untouched, and the failure lands in `conformance::<component>::mono_states_are_distinguishable` — beside the narrowing it is about. `iter_names()` is available (`bitflags` is already the dependency, `conformance/mod.rs:10`), and `StateFlags::empty()` yields no names and is in every list, so it never trips.

Unlike the grep, this survives a file move, cannot be satisfied by a comment about a different state, and catches the recurrence the grep was aimed at: a reason written once and a second state dropped later.

**Expected initial failures** (this is the check working, not a regression): `tabs`, `field`, `text_input` and every heavily-narrowed case (`dialog`, `scroll_region`, `props`) must have their reasons written; `list`'s existing prose already names all three drops and needs only moving into the method [F8].

---

## Fresh analyst adjudications carried forward

These Slice 4 findings were supplied after the Q pass. The decisions are recorded here for
status; the open items remain open. They add no new Q architecture.

**`OVERLAY` versus `TRAPS_FOCUS` — decision.** `Caps::OVERLAY` means that a component opens a
layer. `Caps::TRAPS_FOCUS` is separate, implies `OVERLAY`, and is reserved for a real focus
scope; modal cases declare both. `Select` opens a pointer-barrier `Popover`, so it declares
`OVERLAY` only and remains non-trapping with focus on its field. Focus-out dismissal is the
separate popover property; it does not turn a `Popover` into a focus trap. The capability split
and its self-policing case-14 rules are recorded in architecture §29.6/§29.8.

**`FieldControl` item channel — decision with an open follow-up.** The scalar
`FieldControl::draw` shape cannot carry per-phase items. Item-bearing choice controls therefore
stay on their direct per-phase paths; `Form` drives the three choice controls directly. Whether
to add an item-aware composition path or widen the scalar trait remains open. Q does not widen
the trait or move items into props.

**`RadioGroup::value(ItemKey)` — open contract question.** The draw-time controlled prop and
cursor/value separation exist in code, but §17.0 A7 does not document them. The public contract
wording and controlled-state adjudication remain open; Q records no replacement API.

**`ChipBar` `Activated(add_key)` — open action-naming question.** The add affordance uses the
caller-stated `add_key` and currently emits the existing `Activated(ItemKey)` action. There is
no `Added`/`AddRequested` variant. Whether that action should be renamed or split remains open;
Q does not change the current behavior.

**`StatusBar` `hovered_part` — open integration question.** The analyst found that a stateless
status bar could not paint per-item hover from the old frame snapshot. A `FrameRead::hovered_part`
primitive is now present in the concurrent tree, but `StatusBar` still does not consume it.
Whether per-item hover becomes a StatusBar contract and test remains open; Q does not claim the
visual gap closed.

## Risks

1. **R1 is closed.** The enabled → bracket-disabled → restored sequence above proves that this Tabs pair needs the reserved-cell bracket in addition to `CONTAINER`'s `BOLD`.
2. **Q1 is a per-component obligation, so it can be forgotten.** Mitigated by the shared helper plus the boundary condition A3: no component may open-code `GlyphRole::PressLeft`.
3. **The Button mono baseline is intentionally eligible to move.** The bracket now uses reserved cells, so its mono line requires the §20.10 item 18 capture/classify/bless sequence before blessing.
4. **Q3's reason check is now a maintained invariant.** Any future narrowing without a reason, or any omitted state name, fails case 9.
5. **Q2 is a `tui-next-testing` public-API break** (two fields). Dev-only crate, `publish = false` (`COMPONENT_ARCHITECTURE.md:1665`); no consumer outside the workspace.
6. **The `RowUi::marker`/`part` defect is fixed in the current source.** The `Slot<GlyphRole>` migration and the two existing example callers are covered by the applied A4 contract.

---

## Exact document amendments

1. **§11.4, `PRESSED` row (`COMPONENT_ARCHITECTURE.md:1041`)** — append: *"The `CONTAINER` half is a `StateRule`. The bracket is a **component obligation**, like the `BUSY` spinner two rows down and for the same reason: a rule binds a glyph but cannot reserve the two columns it needs. A component that reserves pad cells around its label (`Button`'s gutter and trailing pad, a tab's or a chip's pads) paints `PressLeft`/`PressRight` **into those cells**, never into the label's own run, so a mono fallback never changes geometry. A component whose row has no spare pad — every `RowUi`-labelled collection row — expresses `PRESSED` through the `CONTAINER` rule alone, which is already distinguishable (`conformance::list::mono_states_are_distinguishable`). `RowUi` does not paint the bracket. <!-- amended by §29 -->"*
2. **§12.2, after the `RowUi` block (`:1122`)** — add: *"**The glyph slot (binding).** `Resolved.glyph` and `PartMetrics.glyph` are `Slot<GlyphRole>`, preserving `Inherit`, `Set` and `Clear`. Cell-owning `RowUi` methods honor the resolved slot and reserved geometry; `label`, `label_patched`, `label_spans` and `label_fmt` remain text-run methods without automatic glyph placement. The existing example callers are regression coverage. <!-- amended by §29 -->"*
3. **§16.2's `Fixture` declaration (`:1782-1786`)** — replace with the real eight-field shape of Q2 above, `state_override` and `status` private, and add: *"`force(StateFlags)` is the only way to set a forced state; it sets `status` alongside the flags, so P6(iii)'s props-driven-affordance gap cannot be re-opened by a later case. `forced()` and `status()` are the reads."*
4. **§16.2 case 9 (`:1828`)** — append: *"Every narrowing carries a machine-checked reason: `Conformance::mono_narrowing_reason()` is non-empty exactly when `mono_states()` narrows, and names every state dropped (checked by `iter_names()` containment inside this case). The grep §28.6 named in its place could never exit 0 and did not express the property. <!-- amended by §29 -->"*
5. **§28.6, Tests paragraph (`:6173`)** — strike the `! rg -n 'fn mono_states' …` sentence; replace with: *"~~The narrowing must stay visible in one place…~~ **Struck (§29 Q3):** the grep always matched the `StateFlags::empty(),` line every case has and could never exit 0, and four of the five narrowings did not in fact name the states they dropped. Replaced by `Conformance::mono_narrowing_reason()`, asserted inside case 9."*
6. **§28.8 (`:6202`)** — delete the grep line; replace with the A5/A6 commands below.
7. **§20.10** — extend item 18's acceptance column with *"and, under §29 Q1, `render::components::button::pressed`'s mono line if `Button`'s bracket moves out of the text run"*; no new item.
8. **§16.1, `components/*.rs` list** — add `button::mono_pressed_does_not_truncate_the_label`, `tabs::mono_pressed_brackets_the_reserved_pad_cells`.
9. **New §29 "Adjudication Q — Slice 3 residuals"** carrying Q1/Q2/Q3 verbatim, with `<!-- amended by §29 -->` markers on §11.4, §12.2, §16.1, §16.2 (two places), §28.6, §28.8, and mirrored in `REFACTORING_STATE.md` per the change-control rule at `COMPONENT_ARCHITECTURE.md:3`.

---

## Executable acceptance conditions

```bash
# ── A1 (R1): settle what actually distinguishes mono PRESSED, before any prose lands
cargo test -p tui-next --test conformance conformance::tabs::mono_states_are_distinguishable
cargo test -p tui-next --lib components::tabs::tests::mono_pressed_brackets_the_reserved_pad_cells
#   new: at ColorLevel::Mono with forced PRESSED, cells tab.x and tab.x+1+label_w hold
#   PressLeft/PressRight; the label run is byte-identical to the FOCUSED rendering.
#   Then, with the bracket block at tabs.rs:719-728 temporarily disabled, case 9 must
#   still be RED — if it is GREEN, CONTAINER's BOLD already distinguished the two and
#   §29 Q1's narrative is corrected to say so.

# ── A2: Q1 — the idiom is shared, and geometry never moves
cargo test -p tui-next --lib components::button::tests::mono_pressed_does_not_truncate_the_label
cargo test -p tui-next --test conformance conformance::button::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::list::mono_states_are_distinguishable

# ── A3: exactly one implementation of the bracket
test "$(rg -c -- 'GlyphRole::PressLeft' crates/tui/src/components/ | rg -v 'mod\.rs' | wc -l)" -eq 0

# ── A4: live RowUi callers exercise the Slot contract
rg -n '\.marker\(' crates/tui/examples/07_borrowed_rows.rs crates/tui/examples/08_dynamic_tabs.rs
rtk cargo test -p tui-next --lib collection::rowui

# ── A5: Q2 — the forced state is unsettable except through `force`
! rg -n 'pub (state_override|status)\s*:' crates/tui-testing/src/conformance/mod.rs
rg -n 'state_override\s*=|status\s*=' crates/tui-testing/src/conformance/mod.rs
cargo test -p tui-next --test conformance          # every case compiles against forced()/status()
rg -n 'state_override: StateFlags,' crates/tui-testing/src/conformance/mod.rs | rg -v 'pub '

# ── A6: Q3 — the reason is declared and checked, not grepped
cargo test -p tui-next --test conformance conformance:: -- --include-ignored
!  rg -n "rg -v '///" COMPONENT_ARCHITECTURE.md      # the broken grep is gone from the doc
rg -n 'mono_narrowing_reason' crates/tui-testing/src/conformance/{mod,driver}.rs \
      crates/tui/tests/conformance.rs COMPONENT_ARCHITECTURE.md

# ── the whole gate
cargo test --workspace --test render --test render_components
cargo run -p xtask -- boundary
cargo run -p xtask -- doc-check
cargo test --workspace --test architecture every_named_test_exists
rg -n 'Adjudication Q' COMPONENT_ARCHITECTURE.md REFACTORING_STATE.md
```

**Gate pass condition.** Every command exits 0; the 797-test suite is green with the reasons written; `docs/visual-changes.md` carries a §20.10 item 18 line **before** any mono baseline is blessed, and only if A2 moves `Button`'s bracket; `crates/tui/tests/allow/*.txt` stay empty; the nine amendments above are applied and mirrored in `REFACTORING_STATE.md`.

**Relevant paths:** `/Users/donbeave/Projects/terminal-components-claude/crates/tui/src/collection/rowui.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui/src/components/{tabs,button,list}.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui/src/theme/downgrade.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui/src/ui/paint.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui-testing/src/conformance/{mod,driver}.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui/tests/conformance.rs`, `/Users/donbeave/Projects/terminal-components-claude/COMPONENT_ARCHITECTURE.md`, `/Users/donbeave/Projects/terminal-components-claude/docs/reviews/adjudication-p-prototype-decisions.md`.
