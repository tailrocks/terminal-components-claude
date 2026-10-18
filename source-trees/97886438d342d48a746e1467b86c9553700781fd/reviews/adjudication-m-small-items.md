# Adjudication M — three small surface items

**Status:** proposed. Decides the two items left open at `COMPONENT_ARCHITECTURE.md:4447-4450` (§22.10) and the one at `:4468` (§23.1 open item 1). Nothing here reopens Adjudications A–L.

**Convention:** **[F]** = collected fact with a citation. Everything else is decision or inference.

**Amends:** §10 (nothing — `Size` is confirmed unchanged), §11.2, §11.7, §12.2, §15.1, §17.0 A2/A5/A7/A10, §17 examples 2 and 13, §18.2, §21 item 1 (clarified, not changed), §22.10, §23.1, Appendix B.4.

---

## 0. Facts these decisions rest on

**[F] `ratatui_core::layout::Size` is `{ width: u16, height: u16 }`** (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-core-0.1.2/src/layout/size.rs:1-40`). Ours is `Size { min: (u16,u16), preferred: (u16,u16) }` (`COMPONENT_ARCHITECTURE.md:690`). Different shape, same name.

**[F] `ratatui_core::text` exports `Line`, `Span`, `Text`, `Masked`, `StyledGrapheme`, `ToLine`, `ToSpan`, `ToText`** (`ratatui-core-0.1.2/src/text.rs:51-64`). Ours is a role-carrying `Span<'a>` (§12.2 `RowUi::label_spans`, `:1022`; §18.2 viewport row, `:3342`), which §22.2 item 16 explicitly keeps.

**[F] `Frame::area() -> Rect`; `Frame` has no `Size` anywhere** (`ratatui-core-0.1.2/src/terminal/frame.rs:60-70`). `Frame`'s only appearance in our surface is `Runtime::draw(&mut self, frame: &mut Frame<'_>)` (§17.0 A1, `:1995`).

**[F] `symbols::border::Set<'a>` is a plain struct with eight `pub` `&'a str` fields and derives `Debug, Clone, Copy, Eq, PartialEq, Hash`** (`ratatui-core-0.1.2/src/symbols/border.rs:3-19`). Consts shipped: `PLAIN`, `ROUNDED`, `DOUBLE`, `THICK`, six dashed sets, `QUADRANT_*`, `ONE_EIGHTH_*` (`:43-260`). **There is no `+-|` ASCII set.** `Default for Set` is `PLAIN` (`:15-19`).

**[F] `Capability` has exactly one field.** `pub struct Capability { pub color: ColorLevel }`; `UnicodeLevel` was deleted by §21 item 19 (`:2387`, `:4165`).

**[F] `Theme::paper()` already specifies `symbols::border::PLAIN`** (§11.7, `:973`); §11.2 records Junie's set as `symbols::border::ROUNDED` verbatim (`:829`).

**[F] `FieldKind<'a>` wraps `Select<'a>` / `RadioGroup<'a>` / `ChipBar<'a>` without collection generics** (§15.1 `:1351-1361`, §17.0 A10 `:2470`), and A10 gives `Select::new(id, options: &'a [&'a str])` (`:2518-2519`), while §18.2 types them `Select<'a,T,K,R>`, `RadioGroup<'a,T,K,R>`, `ChipBar<'a,T,K,R>` (`:3314`, `:3315`, `:3334`) and §21 items 1 and 5 put `RadioGroup`/`ChipBar` under the per-phase item channel and the three-impl-block builder scheme (`:3983`, `:4050`).

---

## M1 — Root and `author` re-export set

### Decision

**Neither of our types is renamed and neither colliding ratatui type is exported.** MOD §1.2's binding rule is *"`junie-tui` re-exports the ratatui types the public API mentions"*; its parenthetical list was written before §22.10 found that two of the eleven names are already ours. Applying the rule rather than the list:

* `ratatui_core::layout::Size` — **not exported anywhere.** No `pub` signature in `junie-tui` names it (`Frame::area()` returns `Rect`; resize enters as `Input::Resize{w,h}`; `Backend::size`/`WindowSize` never leave `runtime/session.rs`). Our `Size { min, preferred }` keeps the name at root and in `author`, unchanged (§10).
* `ratatui_core::text::{Line, Span, Text}` — **not exported at the root and not exported flat in `author`.** No `pub` signature names them: §22.2 item 16 makes `RowUi::label_spans` build a borrowed `Line<'_>` *inside* `crates/tui/src/ui/paint.rs`, which is an implementation detail. Our `Span<'a>` keeps the name.
* Everything the surface does name is re-exported at the layer that names it.

`Ui::raw()`/`RowUi::raw()` hand out `&mut Buffer`, whose `set_line`/`set_span` are then unreachable. Two closures for that, both cheap: one new `Ui` method in our own vocabulary, and one qualified-only escape module.

### Exact Rust

```rust
// crates/tui/src/lib.rs — the application-author facade (§ B.3 item 2: one curated line each)
pub use ratatui_core::buffer::{Buffer, Cell};
pub use ratatui_core::layout::{Position, Rect};
pub use ratatui_core::style::{Color, Modifier, Style};
pub use ratatui_core::terminal::Frame;          // named by Runtime::draw (A1) — host concern, root only
pub use crate::text::Span;                      // OURS: role-carrying, used by RowUi::label_spans
pub mod theme;                                  // theme::border (M2), ColorTokens, Theme, …

// crates/tui/src/theme/border.rs — reachable as junie_tui::theme::border and junie_tui::author::border
pub use ratatui_core::symbols::border::{Set, DOUBLE, PLAIN, ROUNDED};
pub type BorderSet = Set<'static>;              // §11.2, unchanged

// crates/tui/src/author.rs
pub mod author {
    // …everything already listed at COMPONENT_ARCHITECTURE.md:3919-3956, plus:
    pub use crate::text::Span;                  // OURS
    pub use ratatui_core::terminal::Frame;      // NOT re-exported here — see below
    /// Types needed only to drive the `Ui::raw()` / `RowUi::raw()` escape hatch.
    /// The ONLY re-export not forced by a signature. `raw::Span` is ratatui's
    /// style-carrying span and is written qualified, always: `raw::Span`.
    pub mod raw {
        pub use ratatui_core::text::{Line, Span, Text};
    }
}

// crates/tui/src/ui/paint.rs — new, so the role-carrying Span is the only span in normal use (R‑3)
impl Ui<'_> {
    /// Multi-style single-line paint. Resolves each `Span`'s `Role` against the live
    /// theme and surface, then writes one `ratatui_core::text::Line` through
    /// `Buffer::set_line`. Returns columns written.
    pub fn paint_spans(&mut self, area: Rect, spans: &[Span<'_>]) -> u16;
}
```

`Frame` is **root only** — delete the `author` line above; a component author receives `Ui`, never a `Frame`, and B.4 already excludes `Runtime`/`run` for the same reason. `Buffer`, `Cell`, `Rect`, `Position`, `Color`, `Style`, `Modifier`, `Span` (ours) and `border` appear in **both** root and `author`: `Ui` is a root type whose `raw()` names `Buffer`, so an app must be able to name it without reaching into `author`.

`Span<'a>` moves from `components/viewport.rs` (§18.2 `:3342`) to `crates/tui/src/text/span.rs`. Reason: `RowUi` lives in `collection/`, and a `collection → components` dependency for a type name is backwards; `TextViewport` and `RowUi` are both consumers.

### Mechanical proof that "apps depend on junie-tui only" stays true

The existing check (`architecture::applications_depend_only_on_the_library_facade`, `:1859`) proves the *dependency edge*. It does not prove the facade is *complete* — an app can be forced back to a `ratatui-core` line by one signature naming an unexported type. Close it:

> **`architecture::every_foreign_type_in_the_public_surface_is_re_exported`** (`xtask`, rustdoc-json). For every non-local type named in a `pub` item reachable from `junie_tui::`, a `pub use` path exists under `junie_tui::`; likewise for `junie_tui::author::`. Failure prints the type, the signature that names it, and the missing facade line.

This is what makes the decision self-maintaining: the day someone puts a `Line` in a signature, the check fails and the exporting decision is forced, rather than discovered by a downstream `ratatui-core` dependency line.

### Rejected alternatives

* **Rename ours (`Measured`/`Extent` for `Size`, `RoleSpan` for `Span`).** Renames the type used by `App::min_size`, `Measure::measure` and every component's `measure` and `label_spans` — the whole public vocabulary — to make room for two types no signature mentions. `RowUi::label_spans(&[RoleSpan])` also reads worse. Cost with no benefit.
* **Rename theirs on export (`TermSize`, `TextSpan`/`StyledSpan`).** Creates a name that exists in no upstream doc, no upstream error message and no Stack Overflow answer, for types that only ever appear behind `raw()`. It also does not reduce the count of exported names.
* **A single `junie_tui::tty` / `junie_tui::ratatui` submodule for all of them.** `tty` is a lie (these are text and geometry types, not terminal-session types), and `junie_tui::ratatui::` trips §22.7 forbidden-pattern rule 14 (`\bratatui::`) at every use site — a mechanical conflict with an accepted gate. `author::raw` says exactly what it is for and collides with nothing.

### Test names

`architecture::every_foreign_type_in_the_public_surface_is_re_exported`, `architecture::applications_depend_only_on_the_library_facade` (existing, unchanged), `ui::paint_spans_matches_row_ui_label_spans` (differential over the §16.1 text corpus).

---

## M2 — The ASCII border set

### Decision

**Keep ASCII, as a plain `const` of the foreign type, in `junie_tui::theme::border`.** A `const` of a foreign type is not an `impl` and is subject to no coherence rule; `Set`'s eight fields are all `pub` **[F]**. The type alias stays exactly as §11.2 and R‑11 wrote it.

`Theme::junie()` uses `border::ROUNDED`; `Theme::paper()` uses `border::PLAIN`. `border::ASCII` is used by neither builtin — it is opt-in through the already-specified `ThemeBuilder::borders_set` (§17.0 A5).

**`Capability` has no `unicode` field** **[F]** — §21 item 19 deleted `UnicodeLevel` — so **nothing selects ASCII automatically**, and nothing should.

### Exact Rust

```rust
// crates/tui/src/theme/border.rs
pub use ratatui_core::symbols::border::{Set, DOUBLE, PLAIN, ROUNDED};

/// Pure-ASCII border set, for terminals and fonts without box-drawing glyphs.
/// Not shipped by ratatui; declared here as a plain `const` because `BorderSet`
/// is a type alias of a foreign type and can carry no inherent items (§11.2).
/// Opt in with `Theme::junie().builder().borders_set(border::ASCII).build()`.
pub const ASCII: Set<'static> = Set {
    top_left:         "+", top_right:         "+",
    bottom_left:      "+", bottom_right:      "+",
    vertical_left:    "|", vertical_right:    "|",
    horizontal_top:   "-", horizontal_bottom: "-",
};
```

`Theme::junie()` → `design.borders = border::ROUNDED`. `Theme::paper()` → `design.borders = border::PLAIN` (§11.7, unchanged).

### Rationale

Three glyphs, eight fields, ten lines, zero new types, and `Set: PartialEq` **[F]** makes the builtins assertable. §11.2's declared triple (`rounded | square | ascii`) survives without the deleted `UnicodeLevel` returning.

Automatic selection is rejected on three grounds, not taste. (1) It needs runtime terminal capability detection, which contradicts §16.4's determinism claim for the same reason §22.2 item 6 rejected keyboard-enhancement flags. (2) Borders are 8 of ~35 glyph slots; a border-only auto-switch renders a frame that is ASCII at the edges and unicode everywhere else (`GlyphRole::{Chosen, Checked, SortAsc, FollowRef, …}`) — worse than either consistent choice. (3) It would reintroduce a `Capability` axis §21 item 19 deliberately deleted, and every §16.3 baseline would fork by terminal.

**Deferred root cause, named:** if unicode-capability fallback is ever wanted, the correct shape is a `Capability` field **plus** a full `GlyphSet` fallback table applied the way §11.4's `apply_mono_fallbacks` applies colour fallbacks, plus re-blessed baselines under §20.10. That is a fresh adjudication, not a border const. `border::ASCII` is the manual, deterministic, theme-author-visible half of it and does not prejudge it.

### Rejected alternatives

* **Drop ASCII.** Leaves no supported way to theme for a terminal without box-drawing glyphs, and the mechanism to support it (`borders_set`) already exists and is already tested — so dropping saves ten lines and removes a capability §11.2 declared.
* **`pub struct BorderSet(Set<'static>)` newtype.** Re-adds the wrapper §22.2 item 12 just deleted. It forces `.0` or a `Deref` at every read, breaks `ThemeBuilder::borders_set(border::PLAIN)` (would become `borders_set(BorderSet::from(PLAIN))`), and buys only the ability to hang inherent consts — which a module of consts already provides, with better `use` ergonomics.

### Test names

`theme::ascii_border_set_is_pure_ascii` (each of the eight fields satisfies `s.is_ascii() && s.len() == 1`, which also pins `text::width(s) == 1`), `theme::builtin_border_sets_are_ratatui_sets` (`assert_eq!(Theme::junie().design.borders, border::ROUNDED)` and `assert_eq!(Theme::paper().design.borders, border::PLAIN)`), `theme::ascii_theme_renders_without_box_drawing_glyphs` (a `Scene` digest over `Theme::junie().builder().borders_set(border::ASCII).build()` contains no `U+2500..=U+257F`), `architecture::capability_has_no_unicode_field` (rustdoc-json: `Capability`'s field set is exactly `{color}`).

---

## M3 — `FieldKind` versus the collection generics

### Decision — option (a), with the option list on the data channel, not on the props

**`Select`, `RadioGroup` and `ChipBar` keep `<'a, T, K, R>` and the per-phase item channel, exactly like every other collection (§18.2, §21 items 1 and 5, unamended). `FieldKind<'a>` stays a closed, non-generic enum holding the default instantiation `<'a, &'a str, ByIndex, DefaultRow>`. The option list a form field needs reaches the control through `FormData`, which is already the form's single data channel.**

This is the only reconciliation that changes neither §21 item 1 nor §18.2 nor `FieldKind`'s shape. It costs two defaulted `FormData` methods and three type aliases.

### Exact Rust

```rust
// ── §17.0 A7 / §18.2: the three controls are ordinary collections ────────────
impl<'a, T> Select<'a, T, ByIndex, DefaultRow> { pub fn new(id: Id) -> Self; }   // items are per phase (A3)
impl<'a, T, K, R> Select<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[Part::FIELD, Part::LABEL, Part::MARKER, Part::ROW,
                                         Part::TRACK, Part::THUMB, Part::EMPTY];
    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Select<'a, T, K2, R>;
    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Select<'a, T, K, R2>;
    pub fn placeholder(self, s: &'a str) -> Self;
    pub fn popup_rows(self, n: u16) -> Self;
    pub fn read_only(self, yes: bool) -> Self;   pub fn disabled(self, yes: bool) -> Self;
    pub fn patch_part(self, ps: &'a [(Part, StylePatch)]) -> Self;
    pub fn state_override(self, s: StateFlags) -> Self;
}
impl<'a, T, K: KeyFn<T>, R: RowFn<T>> Select<'a, T, K, R> {
    pub fn update(&self, cx: &mut Cx<'_>, st: &mut SelectState, items: &[T]) -> Response<SelectAction>;
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &SelectState, items: &[T]) -> Rect;
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size;
}
// identical three-block shape for RadioGroup<'a,T,K,R> and ChipBar<'a,T,K,R> (§21 item 5)

// ── §15.1 / §17.0 A10: FieldKind stays closed and non-generic ────────────────
pub type LabelSelect<'a> = Select    <'a, &'a str, ByIndex, DefaultRow>;
pub type LabelRadio <'a> = RadioGroup<'a, &'a str, ByIndex, DefaultRow>;
pub type LabelChips <'a> = ChipBar   <'a, &'a str, ByIndex, DefaultRow>;

pub enum FieldKind<'a> {
    Text   (TextInput<'a>),
    Area   (TextArea<'a>),
    Select (LabelSelect<'a>),
    Radio  (LabelRadio<'a>),
    Chips  (LabelChips<'a>),
    Check  (Checkbox<'a>),
    Toggle (Toggle<'a>),
    Chooser(Button<'a>),
    Note,
}

// ── §15.1: the option list is data, and travels with the value ───────────────
pub trait FormData {
    fn value    (&self,     id: Id) -> FieldRef<'_>;
    fn value_mut(&mut self, id: Id) -> FieldMut<'_>;

    /// Option labels for a `Select` / `Radio` / `Chips` field; `&[]` for every other kind.
    /// Borrowed from the caller — never `'static` (§21 item 22). Painted, never returned
    /// in a `FormAction` (F5).
    fn options(&self, _id: Id) -> &[&str] { &[] }

    /// The controlled value and the option list under ONE borrow, so `Form::update` can
    /// drive a choice control without a second `&mut` (E0502). A data type with option
    /// tables overrides it by destructuring its own disjoint fields.
    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        (self.value_mut(id), &[])
    }

    fn visible (&self, _id: Id) -> bool { true }
    fn disabled(&self, _id: Id) -> bool { false }
    fn error   (&self, _id: Id) -> Option<&str> { None }
    fn validate(&self, _id: Id, _v: FieldRef<'_>) -> Result<(), FieldError> { Ok(()) }
    fn validate_all(&self) -> Result<(), (Id, FieldError)> { Ok(()) }
}
```

`Form::draw` calls `data.value(id)` + `data.options(id)` (two shared borrows). `Form::update` calls `data.value_and_options(id)` (one mutable borrow). `FieldSpec::new` stays `const fn` — `Select::new(id)` is `const`-constructible.

**Amendments this forces:** §17.0 A10's `Select::new(id, options)` and `RadioGroup::new(id, options)` become `new(id)`; §17 example 13's `conn_fields` loses its `engines/envs/groups/modes` parameters (they move to `ConnDraft::options`) and drops `use FieldKind::*;` in favour of spelled-out `FieldKind::Select(…)` — the glob currently shadows the imported `Select` type in the value namespace and compiles only by an explicit-beats-glob accident.

### Invariants

* **M3‑1 — `FormState` stores no props.** `FieldSlot`'s value is `enum SlotValue { None, Text, Choice(usize), Flag(bool), Chips(KeySet) }`, `Clone + PartialEq + Eq`; text drafts live in the slot's `TextInputState`/`TextEditorCore` (manual redacting `Debug`, `zeroize`), never as a `Secret` field, so `FormState: Clone + PartialEq + Default` holds (S2, conformance case 6) and no `FieldKind` — which holds `&dyn Fn` slots and is neither `Clone` nor `PartialEq` — is ever reachable from state.
* **M3‑2 — Props are built once and hold no values.** `Form<'a>` holds `&'a [FieldSpec<'a>]` and scalars (F1). `FieldKind` holds control *configuration* only; the item slice never enters it, so §21 item 1's B3 borrow hazard cannot arise at a form field.
* **M3‑3 — The form's identity model is positional by construction.** `FieldMut::Choice(&mut usize)` / `FieldRef::Choice(usize)` are index-based, so a keyed `Select` inside a `FieldKind` would have no channel to report its `ItemKey`. `ByIndex` in the aliases is therefore forced by the value channel, not a default that leaked. Documented consequence: a form whose option table is reordered *at runtime* must map indices itself, or use `FieldKind::Chooser`.
* **M3‑4 — `Chooser` is the escape hatch for a richer control.** A field needing a keyed, custom-row, non-string collection is `FieldKind::Chooser(Button<'a>)`: it emits `FormAction::Chose(id)` and the owner opens its own `Picker`/`Select` layer with the full `<T,K,R>` surface. No new API, already in §15.1 and example 13's shape.

### Rejected alternatives

* **(b1) Generic `FieldKind`.** A closed enum cannot carry a *per-variant* type parameter; it would need nine (`FieldKind<'a, TS,KS,RS, TR,KR,RR, TC,KC,RC>`), and every element of a `&'a [FieldSpec<'a>]` must be the same instantiation — so a form could not hold two `Select`s over different item types. Fatal, not merely ugly. It would also put type parameters into `Form`, `FormState` and every screen's field-array signature, which §13 forbids ("no gratuitous generic parameters") and §11.1 already rejected for themes.
* **(b2) `FieldKind::Control(&'a dyn FieldControl)`.** `FieldControl` has an associated type `State` (§15, `:1309-1314`) and is not dyn-safe; erasing `State` erases the very thing `FormState.slots` keys (`TextInputState` vs `SelectState`), breaking M3‑1. Even erased, `Form` still needs a discriminant to know each field's *value shape* (`Text` vs `Choice` vs `Flag` vs `Chips`) — which is the enum, re-added beside the trait object. Collapses to (a) plus indirection and a boxed-free-but-dynamic dispatch on the form path.
* **(a′) `FieldKind` variants hold the option slice (`Select(LabelSelect<'a>, &'a [&'a str])`).** Rejected: it puts data back in props, which §21 item 1 removed for exactly this class, and it re-opens the disjointness question (the props array and `&mut D` must borrow disjoint fields of the screen — a borrow error at best, a "build the array twice" workaround at worst). `FormData` already exists and already carries per-field data keyed by `Id`; a second parallel channel is the API-inconsistency class G1 removes.
* **(a″) `Select::labels(id, &[&str])` as a separate convenience constructor.** Redundant under this decision: `Select::new(id)` is already the non-generic default instantiation, and `T = &'a str` is inferred from the `FieldKind::Select` variant. One name, not two.

### Test names

`form::select_field_options_come_from_form_data`, `form::changing_options_between_frames_does_not_rebuild_props`, `form::state_holds_no_props` (static assertion: `FormState: Clone + PartialEq + Default`; `SlotValue: Clone + PartialEq + Eq`), `form::value_and_options_is_a_single_borrow` (a doctest whose `Form::update` body would be `E0502` under two calls), `form::chooser_activation_emits_chose_with_the_field_id` (retained from §15.1, now also the escape-hatch proof), `architecture::field_kind_has_no_type_parameters` (rustdoc-json: one lifetime, zero type params), `select::standalone_select_takes_items_per_phase`, `select::escape_closes_and_restores_the_cursor` (retained), `choice::radio_group_separates_cursor_from_value` (retained).

---

## Acceptance conditions (executable)

```bash
cargo check -p junie-tui --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  --workspace --all-targets --all-features
cargo test  --workspace --doc
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo build -p junie-tui --examples                     # examples 2 and 13 compile as amended
cargo tree  -p showcase -e normal --depth 1             # => junie-tui, and nothing else

cargo test --workspace --test architecture every_foreign_type_in_the_public_surface_is_re_exported
cargo test --workspace --test architecture capability_has_no_unicode_field
cargo test --workspace --test architecture field_kind_has_no_type_parameters
cargo test -p junie-tui --lib theme::ascii_border_set_is_pure_ascii
cargo test -p junie-tui --lib theme::builtin_border_sets_are_ratatui_sets
cargo test -p junie-tui --lib ui::paint_spans_matches_row_ui_label_spans
cargo test -p junie-tui --lib form::
cargo test -p junie-tui --test conformance conformance::form::

# M1: neither colliding ratatui type is re-exported, and no umbrella path appears
! rg -n 'pub use ratatui_core::(layout::\{[^}]*\bSize\b|text::\{)' crates/tui/src/lib.rs
! rg -n 'pub use ratatui_core::text::' crates/tui/src/author.rs
  rg -n 'pub mod raw' crates/tui/src/author.rs
# M2: ASCII is a const, never an impl on the alias
! rg -n 'impl (BorderSet|border::Set)' crates/tui/src
  rg -n 'pub const ASCII: Set' crates/tui/src/theme/border.rs
# M3: no items in a form control's constructor; no glob over FieldKind
! rg -n 'fn new\(id: Id, (options|labels|items): &' crates/tui/src/components/{select,choice,chip}.rs
! rg -n 'use FieldKind::\*' crates/tui/examples
```

Pass condition: all commands succeed, the four `!`-prefixed greps return no matches, and `crates/tui/tests/allow/legacy_api.txt` stays empty.

---

## Risks

1. **`Frame` at the root puts a `ratatui-core` type in our published surface.** Already true of `Rect`/`Style`/`Color`, so it adds no new class of exposure; `cargo-semver-checks` from `v0.1.1` (§22.3) is the detector. If ratatui-core ever moves `Frame`, one facade line changes and `every_foreign_type_in_the_public_surface_is_re_exported` fails loudly rather than an app's build breaking.
2. **`author::raw` re-exports a second `Span`.** Mitigated by module qualification (`raw::Span`, never a flat `use`) and by `Ui::paint_spans`, which removes the only realistic reason to reach for it. If `raw::Span` starts appearing in components, that is a signal `paint_spans` is under-specified — not a naming problem.
3. **`border::ASCII` themes still get unicode `GlyphSet` glyphs.** Stated in the const's rustdoc and above as the deferred root cause. `theme::ascii_theme_renders_without_box_drawing_glyphs` scans only the border range, deliberately, so it does not create a false impression of full ASCII safety.
4. **`FormData` widens to seven methods (five defaulted).** Same shape §12.3 chose for `GridModel` and §23 K2 reinforced. `value_and_options`'s default is correct for every kind except the three choice kinds, so a data type that forgets to override it renders an empty option list — visible immediately in `form::select_field_options_come_from_form_data` and in the first frame, not a silent wrong answer.
5. **Example 13 and §17.0 A10 change text.** Both are gated by `xtask doc-check` (§21 item 34) and `architecture::all_examples_compile`, so the amendment is verified rather than asserted; `REFACTORING_STATE.md` must mirror the three decisions before work packages 4B/4F/4G start.
