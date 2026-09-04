# Adjudication P — prototype decisions returned by the Slice 2/3 builder

**Status:** proposed. Read-only review at `HEAD 8ec40c1`. Source files cited by `file:line`.
**Authority:** `REFACTORING_GOAL.md` › `DESIGN.md` › existing rendered output/tests › current source (`COMPONENT_ARCHITECTURE.md:5`).
Facts marked **[F]** were read from the tree in this pass. Everything else is inference or decision. Two of the six premises handed to me are **wrong as stated** (P3, and half of P6); both corrections are load-bearing, so they are recorded before the decisions.

---

## Collected facts the six decisions rest on

**F1 — the mono rule table.** `mono_rules()` returns 13 rules and `mono_rules_extra()` 3, appended to every family *and* to every variant map that already declares the part (`crates/tui/src/theme/downgrade.rs:257-381`, `MONO_RULES_PER_FAMILY = 16` at `:354`). The parts reached for `DISABLED` are exactly `GUTTER` (`:288-295`), `MARKER` (`:296-303`) and `LABEL` (`:304-310`). **No rule reaches `FIELD`, `TEXT` or `PLACEHOLDER` for any state except `(FIELD, ERROR)` (`:316-320`) and `(TEXT, EDITING)` (`:326-330`). No rule mentions `BUSY` or `LOADING` at all.**

**F2 — the parts a text control paints.** `TextInput::PARTS = [FIELD, TEXT, PLACEHOLDER, MARKER, GUTTER]` (`crates/tui/src/components/input.rs:473-479`); it never paints `LABEL`. `Field` paints `LABEL` with the same live flags (`crates/tui/src/components/field.rs:211-212`), which is the only reason `FieldCase` survives case 9's `disabled` row.

**F3 — style inheritance through the fill.** An unset (`Slot::Inherit`) slot binds to `None` (`crates/tui/src/theme/resolve.rs:232-263`), and `Buffer::set_stringn` patches only the `Some` fields, so a `TEXT` style that says nothing inherits the `FIELD` fill's colour *and modifiers* per cell. A single rule on `Part::FIELD` therefore reaches the value text. `StylePatch::add`/`remove` are symmetric (`crates/tui/src/theme/patch.rs:116-130`), so `.remove(Modifier::all()).add(Modifier::DIM)` yields `add = DIM`, `sub = ALL − DIM` — the `LABEL` idiom is safe to copy.

**F4 — mono collapses the disabled foreground onto the background.** `mono()` maps `Y < 0.35 → Black` (`downgrade.rs:210-220`). Junie's `disabled_fg = #4d4d4d` (`theme/builtin/junie.rs`, pinned at `builtin/mod.rs:632`) has `Y ≈ 0.07 → Black`; `Fg(Faint) = #262626` likewise; `surfaces[0] = #000000 → Black`. **Under `ColorLevel::Mono` a disabled control is currently black-on-black** — not merely indistinguishable but unreadable. `Theme::paper()`'s disabled step lands in the `Reset` band and escapes this by luck.

**F5 — `BUSY`/`LOADING` in the recipes and in the components.** `row_like` gives `CONTAINER + BUSY` a colour only (`builtin/mod.rs:35`); `button_variant` gives `CONTAINER + BUSY` `remove(BOLD)` (`:176`), which is a no-op for every variant whose base is not bold. `LOADING` appears in no recipe. **`Button` paints a real spinner glyph** from `design.motion.spinner_frames` when `self.busy()` (`components/button.rs:373-378`) — a *symbol*, so it is mono-distinguishable without any theme rule. `TextInput` accepts `.status(Status)` (`input.rs:769`) and paints nothing for `BUSY`/`LOADING`. `List` propagates the flags into row flags (`components/list.rs:720-724`) and paints nothing for them either.

**F6 — the mono case drives state through `state_override` only.** `Fixture` carries `state_override`, `disabled`, `read_only`, `patch`, `secret` — **no `status`** (`crates/tui-testing/src/conformance/mod.rs:62-104`); case 9 forces the state and compares `(symbol, modifier)` multisets (`driver.rs:404-458`). Consequence: forcing `StateFlags::BUSY` does **not** make `Button::busy()` true, so the spinner at `button.rs:373` is never painted under the forced state. A state whose affordance comes from *props* is unreachable by case 9 as written.

**F7 — `mono_states_required_by` returns one state, not a union.** `mod.rs:127-148` is an `if / else if` chain: a case declaring `EDITS | DISABLEABLE` is only ever required to keep `EDITING`. That is why `TextInputCase` (`crates/tui/tests/conformance.rs:332-340`) can drop `DISABLED` while declaring `Caps::DISABLEABLE` and MA-8's guard stays green.

**F8 — Tabs express `ACTIVE` through forced `SELECTED`.** `tabs.rs:664-671`: the first windowed tab becomes `ACTIVE` only when the forced state contains `SELECTED`. Forcing `ACTIVE` directly paints nothing.

**F9 — `Dialog` registers only `Decorative` regions for its own id** (`components/dialog.rs:587-588`, `:615`, `:670`, `:681`); its action buttons register under `action_id(i)`, a different `Id` (`:295-297`).

**F10 — the undelivered-intent guard.** `run_update` diagnoses an undrained bucket **only if** `self.last.registry.delivers_to(owner)` (`crates/tui/src/runtime.rs:632-639`), and `delivers_to` requires a `Control` or `Part` region (`crates/tui/src/hit.rs:313-317`). `dismiss_top` addresses `Intent::Cancel` to `top.spec.owner` (`runtime.rs:555-570`) and `pump_layer_events` addresses `Intent::Layer` to the owner and to the layer id (`:600-609`). `deliverable()` already excludes `Decorative` from pointer delivery (`runtime.rs:331-333`).

**F11 — the boundary check.** `state_override_is_used_only_in_apps_and_fixtures` walks `crates/**` and `apps/**`, skipping `apps/`, `crates/tui/tests/`, `crates/tui-testing/` and a component's own builder forwarding (`xtask/src/main.rs:905-947`). `crates/tui/examples/**` is scanned, so an example may not call `.state_override(`.

---

## P1 — `.state_override` in `crates/tui/examples/**`

**Decision: keep the split until Slice 5. Do not widen the exemption — not to `examples/**`, and not to a `showcase_*` path prefix.**

**Exact change (documentation only; no code, no gate change).**
1. `COMPONENT_ARCHITECTURE.md` §18.3 row 4, append: *“Until `apps/showcase` exists (Slice 5) the migrated page lives in two halves: the runnable page in `crates/tui/examples/showcase_buttons.rs`, and the §18.3-#4 reference **state matrix** — which needs `.state_override` (A11) — in `crates/tui/tests/showcase_buttons.rs`, where `architecture::state_override_is_used_only_in_apps_and_fixtures` admits it. This is a recorded, expiring deviation: at Slice 5 both halves move into `apps/showcase/src/pages/buttons.rs` and `apps/showcase/tests/`, and the split disappears. The check's allow-list is not widened in the meantime.”*
2. `crates/tui/examples/showcase_buttons.rs:137-206`: drop the `Track::Fixed(11)` row that reserves eleven dead rows for a matrix that is not drawn (`:144-146`, `:206`), so the runnable page is honest about what it shows; keep the explanatory comment at `:197-205` verbatim.

**Rationale.** The check protects two things, and only one of them is about production code. (a) *No component forces its own visual state where behaviour depends on it* — an example cannot violate this, so the coordinator's framing is right on that half. (b) *The thirteen §17 examples are the mechanical proof that the public API suffices for an external consumer* (`architecture::examples_are_external_consumers`, §16.5). Widening the exemption to `examples/**` weakens (b) for all thirteen; widening it to a `showcase_*` prefix is precedented (§25 D-10: “a narrowed regex hides the exception; a named path shows it”) but buys a demonstration at the price of the matrix's **assertions**: an example is a binary that no test may import (`#[path]`/`include!` are forbidden by the same check, `xtask/src/main.rs:951-965`), so moving the matrix into the example would delete `tests/showcase_buttons.rs:609-611` — the only assertion in the tree that a forced rendering registers no ids — and nothing would replace it until Slice 5. One slice of an incomplete demo is a smaller loss than one slice of an unasserted reference rendering plus a permanently looser boundary rule. At Slice 5 `apps/showcase` has a `[lib]` target (§21 item 23) and `apps/showcase/tests/` can assert the page it renders; the problem is *temporary by construction*, and the correct handling of a temporary problem is a recorded deviation, not a permanent hole in a gate.

**Rejected alternatives.**
- *Widen to `crates/tui/examples/**`.* Applies the exemption to twelve files that have no claim to it and makes “A11 is showcase/fixture-only” unenforceable for the rest of the refactor.
- *Widen to `crates/tui/examples/showcase_*.rs`.* Better shaped, but see above: it costs the matrix's assertions for exactly the same one slice, and it creates an allow-list entry that a later reader must remember to delete.
- *Move the page to `crates/tui/tests/fixtures/`.* A fixture is not runnable; goal §29's “the showcase demonstrates every public component” is not satisfied by a file nobody can start.

**Test.** `cargo run -p xtask -- boundary --check state_override_is_used_only_in_apps_and_fixtures` continues to exit 0 with no allow-list; `cargo test -p tui-next --test showcase_buttons` continues to assert `b.area_of(MATRIX.index(0).index(0)).is_none()` (`tests/showcase_buttons.rs:609-611`). Slice-5 obligation, to be added to `REFACTORING_STATE.md`: *the two halves are one file under `apps/showcase` and §18.3 #4's deviation paragraph is struck.*

---

## P2 — `render::components::*` file placement

**Decision: confirm the split. `crates/tui/tests/render_components.rs` stays a separate target through Slice 4; §16.3 is amended to name the *test path* as the contract and to record the file mapping.**

**Exact change.** §16.3, replace “**The matrix** — `crates/tui/tests/render.rs`:” with:

> **The matrix.** The binding contract is the **test path**, not the file: `render::components::<component>::<state>`, `render::overrides::<case>`, `render::overlay::<case>`. During Slices 3–4 these live in two targets, because two work packages own them: `crates/tui/tests/render.rs` (foundations — `render::overrides::*`, `render::overlay::*`, painted straight onto `Ui`) and `crates/tui/tests/render_components.rs` (components — `render::components::*`, one function per matrix cell over `{junie, paper} × {truecolor, mono} × {120×40, 40×10}`). Both are ordinary `cargo test` targets whose module paths are identical to the single-file form, so every name §16.3 and the slice acceptance conditions quote resolves unchanged. **Every gate command that runs the render tests must name both targets** (`--test render --test render_components`). Merge into one target at Slice 5, when one owner holds both.

Also amend §16's “Where tests live” table (`COMPONENT_ARCHITECTURE.md:1647`) runner cell to `cargo test --workspace --test render --test render_components --test visual`, and the Appendix A Slice 3/4 gate lines that spell `--test render`.

**Rationale.** The paths §16.3 and the slice-2 acceptance conditions name are produced by the nested `mod render { mod components { … } }` in `render_components.rs:286-295`, so they are byte-identical either way **[F]**; nothing downstream reads the file name. `every_named_test_exists` scans §16.1, §16.2's suite-level bullets, §16.4 and §16.6 only (`xtask/src/main.rs:778-798`) and enumerates bare `#[test] fn` names from source (`:715-761`), so §16.3 is not machine-checked in either layout — folding the file would not add a single assertion. The split buys a real thing during Slices 3–4: two work packages edit two files instead of contending on one, and a component-matrix re-bless does not touch the foundations target's baseline lines. The one genuine cost is the `--test render` gate command silently running half the matrix; that is a real trap and the amendment closes it explicitly.

**Rejected alternative.** *Require the fold now.* It buys nothing testable, creates a cross-work-package file, and would have to be undone if the two owners diverge again in Slice 4.

**Test.** `cargo test -p tui-next --test render --test render_components` runs 384 + the foundations digests; `cargo test -p tui-next --test render_components render::components::` lists the matrix under the documented path. Add to the Slice 3 gate: `cargo test --workspace -- --list | rg -c '^render::components::' ` is non-zero.

---

## P3 — §17 example 11 leaks intents

**Correction first (this changes the decision).** The premise as handed to me is **not reproducible**: for a `Dialog`-owned modal the runtime records **no** `Diagnostic::UndeliveredIntent`. The dialog registers only `Decorative` regions under `CONFIRM` **[F9]**, and the diagnostic is gated on `Registry::delivers_to`, which requires `Control` or `Part` **[F10]**. The builder's `FINDING` comment (`crates/tui/tests/showcase_buttons.rs:476-482`) asserts the diagnostic; the test never exercises the gated shape (it uses the unconditional one at `:483`), so the claim was never observed. What actually happens with the gated shape is **worse**: `Intent::Cancel` and `Intent::Layer(Dismissed)` are addressed to `CONFIRM`, nobody drains them, `finish()` clears the queue (`runtime.rs:793`), and the dismissal is lost **silently** — `DialogAction::Dismissed` is never emitted and `*::no_diagnostics_are_emitted_during_the_journey` cannot see it. (Inference from the cited code; not executed.)

**Decision: both, and a third.**
1. **§17 example 11 takes the unconditional shape**, matching example 9 (`examples/09_composed_dialog.rs:45-73`), `tests/overlay.rs:95-99` and the passing journey fixture (`tests/showcase_buttons.rs:483-492`).
2. **The runtime keeps diagnosing a bucket whose layer closed during the same `handle`, and starts diagnosing it for a decorative owner too.** The `delivers_to` guard is widened, not narrowed.
3. §13 gains the rule the two shapes differ by.

**Exact change.**
- `crates/tui/src/runtime.rs:632-639` and the corresponding §3.3 step 9 wording:
  ```rust
  // undelivered intents are diagnosed per pass, before buckets are cleared.
  // A bucket the RUNTIME addressed (Layer, Cancel, FocusIn/FocusOut) is
  // diagnosed whatever the owner registered: pointer intents already cannot
  // reach a Decorative owner (`deliverable`), so §21 item 13's escape for
  // container regions does not apply to them, and a layer owner that
  // registers only decor would otherwise lose its own dismissal in silence.
  for owner in self.intents.undrained() {
      if self.last.registry.delivers_to(owner) || self.intents.has_runtime_addressed(owner) {
          self.services.diagnostics.push(Diagnostic::UndeliveredIntent { owner });
      }
  }
  ```
  with `IntentQueue::has_runtime_addressed(&self, owner: Id) -> bool` in `crates/tui/src/intent.rs` beside `undrained()` (`:324-330`), true when the bucket holds any `Stored::{Layer, Cancel, FocusIn, FocusOut}`.
- §17 example 11: delete the `if cx.is_open(CONFIRM) { … }` guard; hoist `actions`; keep the `cx.close_layer(CONFIRM, None)` inside `on_action` guarded by `cx.is_open(CONFIRM)` exactly as the fixture writes it (`tests/showcase_buttons.rs:483-492`). Mirror in `crates/tui/examples/11_small_app.rs:76-90`.
- §17 example 11's opener: `cx.open_layer(CONFIRM, remove_dialog().layer(cx))` instead of the bare `LayerSpec::modal(CONFIRM)` (`examples/11_small_app.rs:72`) — §26 N1 requires the component to size its own layer, and the bare spec defaults to `LayerSize::Fill` (`crates/tui/src/layer.rs:192`); `Dialog::update`'s invariant D1 corrects it a frame later, which is a self-inflicted flash the example should not teach.
- §13 gains: *“A component that owns a layer runs its `update` **unconditionally**, every frame, whether or not the layer is open. `cx.is_open(id)` guards the work the **caller** does besides the component (opening, sizing from live data), never the component's own `update`: dismissal is delivered as intents addressed to the layer's owner in the pass **after** the layer closed, and a gated call drains nothing.”*

**The invariant, restated (this is what `*::no_diagnostics_are_emitted_during_the_journey` now asserts).**

> **Every intent the runtime addresses to an owner is drained by that owner within the same `handle`, or it is reported as `Diagnostic::UndeliveredIntent { owner }`.** Runtime-addressed intents are `Layer`, `Cancel`, `FocusIn` and `FocusOut`; they are addressed to a known owner by the runtime itself, so a lost one is always a defect. Pointer intents keep §21 item 13's exemption — a `Decorative` region never receives one (`runtime.rs:331-333`), so a container that registers only decor still contributes zero diagnostics. A journey with zero diagnostics therefore proves that **every layer lifecycle event and every `Cancel` reached its owner**, which is strictly more than it proved before.

**Rationale.** The gated shape is not a style preference: the dismissal path (`handle` → `dismiss_top` → `run_update`) evaluates `cx.is_open` *after* the layer closed **[F10]**, so the guard is guaranteed to skip exactly the pass that carries the event the guard's author wanted to react to. Making the runtime silent about that (option “stop diagnosing”) would enshrine a silent-loss class the whole diagnostic exists to catch, and would do so precisely for `Dialog`, where it is already silent today. Widening the guard costs one bucket scan per undrained owner per pass, on a path that is already O(undrained).

**Rejected alternatives.**
- *Stop diagnosing a bucket whose layer closed during the same `handle`.* It hides the only mechanical signal that an app dropped a dismissal; and since the diagnostic does not currently fire for a decorative owner, adopting it would make the invariant vacuous for every `Dialog`.
- *Make `Dialog` register a `Part` region for its container so the existing guard fires.* Turns the dialog surface into a pointer target and re-opens the “a click inside the dialog chrome is an outside click / is delivered to the dialog” question §21 item 13 settled. The fix belongs in the diagnostic, not in the registry.
- *Deliver layer events to the last frame's drawer instead of the spec owner.* Requires the runtime to keep a per-layer “who drew this” map across frames and gets the answer wrong for a layer that was never drawn.

**Tests** (add to §16.1 `runtime.rs` / `layer.rs` so `every_named_test_exists` covers them):
- `runtime::a_layer_owners_dismissal_is_diagnosed_when_the_owner_does_not_drain_it` — the *gated* Roster shape, Esc on the open modal, exactly one `UndeliveredIntent { owner: CONFIRM }`.
- `runtime::a_decorative_owner_is_not_diagnosed_for_a_pointer_intent` — the §21 item 13 exemption still holds (a click on a decorative container produces no bucket and no diagnostic).
- `dialog::an_unconditional_update_receives_the_dismissal` — the unconditional shape yields `DialogAction::Dismissed(DismissReason::Esc)` and zero diagnostics.

---

## P4 — `Dialog::measured_height` and the prompt/acknowledgement control

**Decision: confirm `input_rows`.** The §26 formula was written before `prompt`/`acknowledge` existed and is incomplete, not wrong; the builder's addition is the minimal correct completion, and it keeps the “pure function of props and design tokens” property that `dialog::layer_size_is_a_pure_function_of_props_and_design_tokens` asserts.

**Exact change — §26.1's `Dialog` block, amended formula.**

> ```text
> measured_width(d)  = self.width.unwrap_or(d.size.dialog_width)
> inner_width(d)     = measured_width(d) − 2 (border) − 2 · d.space.dialog_inset
> measured_height(d) = 3                                   // border(2) + title(1)
>                    + wrapped_rows(description, inner_width(d))
>                    + input_rows(d)                        // NEW
>                    + body_block(d)
>                    + actions_block
>
> input_rows(d)   = 0                              when there is no prompt and no acknowledgement
>                 = d.size.field_height            with `.prompt(label)`
>                 = d.size.field_height + 1        with `.acknowledge(token)`   // the token echo row
> body_block(d)   = 0                              when body_rows == 0
>                 = body_rows + 1                  otherwise (the blank separator row)
>                   where body_rows defaults to d.size.code_preview_lines for `Dialog::new`
>                   and to 0 for confirm / destructive / prompt / acknowledge
> actions_block   = 0 when `actions` is empty, else 2 (the blank row + the action row)
> ```
> The `3` is charged unconditionally: a dialog with no title still reserves its row, so `measured_height` stays a function of the props' *shape* and not of their content. `draw` lays out against exactly these terms (`components/dialog.rs:605-702`), and both halves wrap the description through the same `text::wrapped_rows`, which is why the two cannot drift.

**Rationale.** Implementation matches the amended formula term for term: `input_rows` at `dialog.rs:484-492`, `body_block` at `:495-498`, `measured_height` at `:512-520`, and the draw-side consumption at `:639-654` (`y += field_h`, `+1` for `ack`). Without `input_rows` a `prompt` dialog asks the resolver for a layer three rows shorter than its own content and `draw` clamps the field to `field_h.min(actions_y - y)` (`:645`) — i.e. the prompt is silently squeezed or lost, which is the class of defect §26 N1 exists to remove. The addition is a pure function of `(props, DesignTokens)`, so invariant D1 (`dialog.rs:398-402`) keeps re-asserting a size the dialog can actually lay out.

**Rejected alternative.** *Fold the prompt into `body_rows` and let the caller state it.* The convenience constructors set `body_rows = Some(0)` (`dialog.rs:269`, `:280`) precisely because the caller does not supply a body for a prompt; requiring the *caller* to know that a prompt costs `field_height` rows re-exports a design token into every call site and breaks the moment a theme changes `field_height` — the same defect §15.1 F4 removed from `Form`.

**Tests.** `dialog::layer_size_is_a_pure_function_of_props_and_design_tokens` extended with a `prompt` and an `acknowledge` case across both themes; new `dialog::a_prompt_dialog_sizes_its_own_field_row` (the drawn `Field` rect's height equals `d.size.field_height`, and `measured_height` − the other terms equals `input_rows`). Add both names to §16.1's `dialog` list.

---

## P5 — `Dialog::state_override` and `Overrides::inherit_forced`

**Decision: confirm both, with one amendment that makes the property general instead of dialog-specific, and one that keeps the boundary check honest.**

**Confirmed as built.** `Overrides::inherit_forced` (`components/mod.rs:108-113`) is `pub(crate)`, sets the forced state only when the container has one, and is deliberately not spelled `.state_override(` so `xtask`'s regex (`xtask/src/main.rs:910`) still sees every *caller* use. `Button::inherit_forced` (`components/button.rs:242-245`) is `pub(crate)`. `Dialog::draw` passes `ov.forced_state()` into each action button (`components/dialog.rs:699`) and suppresses every `register_decor` under `forced` (`:586-589`, `:614-616`, `:669-671`, `:680-682`), so a forced dialog leaves no live, clickable controls. `Dialog::state_override` (`:355-362`) is the public A11 surface and is matched by the check's own-builder exemption (`pub const fn state_override`, `xtask:914`).

**Exact change 1 — generalise the composition half.** `Field` has the same problem and does not solve it: `Field::state_override` forces the chrome (`components/field.rs:168-175`, `:189-194`) but the control it draws at `:263` keeps its own state and **registers a live control**. The tests hide this by forcing both halves separately (`tests/render_components.rs:153-155`, `tests/conformance.rs:368-381`), which is a convention, not an invariant. Add the hook to the trait:

```rust
// crates/tui/src/field_control.rs
pub trait FieldControl {
    // …
    /// Adopt an owning container's forced state (A11 composition). The default
    /// is the identity, so a control with no overrides is unaffected.
    #[must_use]
    fn inherit_forced(self, s: Option<StateFlags>) -> Self where Self: Sized { let _ = s; self }
}
```
implemented on `TextInput` by forwarding to `self.ov.inherit_forced(s)`, and called from `Field::draw` before `self.control.draw(...)`. §12.1 gains one sentence: *“A container that can be forced into a reference state forces every component it owns, through the crate-internal `inherit_forced`; a reference rendering registers nothing, at any depth.”*

**Exact change 2 — keep the escape hatch closed.** Add `xtask` boundary rule: `inherit_forced` may appear only under `crates/tui/src/components/**` and `crates/tui/src/field_control.rs`, and never in a `pub fn` signature outside a trait default. Without it, a later slice can make the crate-internal path public and the A11 boundary check becomes decorative.

**Rationale.** A11's contract is *“a forced rendering is a picture, not a control.”* Half a picture with a live text input in it is a `DuplicateId` and a focus stop waiting to happen the first time a showcase page renders the same control twice — exactly the defect the matrix assertion at `tests/showcase_buttons.rs:609-611` was written to catch for buttons. The mechanism is already right; only its reach is short.

**Rejected alternatives.**
- *Make `state_override` public on the container and let it propagate through the public builder.* Every propagation site would then match `\.state_override(` and the boundary check would have to allow-list library source — destroying the check.
- *A `Ui`-level “forced” scope pushed around the reference rendering.* It would silently disable registration for anything drawn inside, including a control the page genuinely wants live; scope-shaped state that changes registration semantics is the `begin_modal` mistake (§1.2(5), §18.1) in a new place.

**Tests.** `dialog::a_forced_dialog_registers_no_control` (`area_of(action_id(0))` and `area_of(id)` are both `None`; the ring is empty); `field::a_forced_field_registers_no_control` (the same for `Field` over a `TextInput`, which fails today). Both added to §16.1.

---

## P6 — a disabled text control is not distinguishable under `ColorLevel::Mono` — **priority**

**Correction to the premise.** Two of the three narrowings named in the task are not what the tree does **[F]**: `ListCase` keeps `DISABLED` (`tests/conformance.rs:492-503`) and narrows `EDITING`/`BUSY`/`ACTIVE`; only `TextInputCase` narrows `DISABLED` (`:332-340`). And `ListCase` passes for `DISABLED` for the same reason `FieldCase` does — it paints `Part::LABEL` through `RowUi` (`components/list.rs:796-797`), which the `(LABEL, DISABLED)` mono rule reaches. **The rule is exactly: a component survives case 9's `disabled` row iff it paints `LABEL`, `GUTTER` or `MARKER`. A text control paints none of them for its content.**

**And a second, worse defect found in the same place [F4]:** under `ColorLevel::Mono` the `DISABLED` colour rules resolve to `Black` on a `Black` canvas for `Theme::junie()` — `disabled_fg #4d4d4d` and `Fg(Faint) #262626` both have `Y < 0.35` (`theme/downgrade.rs:210-220`). A disabled control at mono is not merely indistinguishable from an enabled one; **it is invisible.** §11.4's `DISABLED` row prescribes `fg = Role::Fg(Faint)`, which is the instruction that produces it. Goal §29 asks for *readable*, not only *distinguishable*, so both halves are in scope.

**Decision.**

**(a) Add three mono `DISABLED` rules and stop tinting at mono.**

```rust
// crates/tui/src/theme/downgrade.rs — mono_rules(), returning [(Part, StateFlags, StylePatch); 15]
// inserted with the other DISABLED rules, BEFORE the ERROR rules: state rules of
// equal specificity apply in declaration order, so ERROR's UNDERLINED must land
// after DISABLED's `remove(Modifier::all())` or it is erased.
(
    Part::FIELD,
    StateFlags::DISABLED,
    p().set_fg(Role::Fg(FgStep::Primary))       // NOT Faint: at Mono, Faint IS the background
        .remove(Modifier::all())
        .add(Modifier::DIM),
),
(
    Part::TEXT,
    StateFlags::DISABLED,
    p().set_fg(Role::Fg(FgStep::Primary))
        .remove(Modifier::all())
        .add(Modifier::DIM),
),
```
and amend the existing `(Part::LABEL, StateFlags::DISABLED)` rule (`downgrade.rs:304-310`) and `(Part::MARKER, StateFlags::DISABLED)` (`:296-303`) to `set_fg(Role::Fg(FgStep::Primary))` for the same reason. `MONO_RULES_PER_FAMILY` becomes **18** (`:354`).

`Part::PLACEHOLDER` needs **no** rule: it is painted over the `FIELD` fill and inherits its modifiers per cell **[F3]**. `Part::CONTAINER` needs no rule: a text control fills `FIELD`, not `CONTAINER`, so a `CONTAINER` rule would not reach the defect and would move far more baselines.

§11.4's table rows become:

> | `DISABLED` | no gutter glyph, no marker, **`DIM` added and every other modifier removed, on `LABEL`, `MARKER`, `FIELD` and `TEXT`**. At `Mono` the foreground is set to `Fg(Primary)`, **not** `Fg(Faint)`: `mono()` maps every step below `Y = 0.35` to `Black`, so a faint disabled foreground on a dark canvas is black-on-black — invisible rather than merely colourless (§29). Colour is excluded from case 9's comparison, so `DIM` plus the absent glyphs is the whole signal, and it is enough. <!-- amended by §P --> |
> | `BUSY` / `LOADING` | **a component obligation, not a `StateRule`**: a component that can enter `BUSY`/`LOADING` paints `Part::ICON` with `design.motion.spinner_frames`. The spinner is a *symbol*, so it satisfies case 9 without any theme rule, and a `StateRule` could not express it (a rule binds one `GlyphRole`; the spinner is a frame sequence). A component with no icon slot must not accept `.status(…)`. <!-- amended by §P --> |

**(b) `BUSY`/`LOADING` are legitimately glyph-plus-colour — but the obligation and the fixture are both missing.** `Button` already discharges it (`button.rs:373-378`) **[F5]**; `TextInput` accepts `.status()` and paints nothing **[F5]**; and case 9 could not see the spinner even where it exists, because it forces theme flags and never the props that drive painting **[F6]**. Three changes:
1. `Fixture` gains `pub status: Status`, defaulting to `Status::Ready`, and the driver derives it from the forced state (`BUSY → Busy`, `LOADING → Loading`, `ERROR → Error`) before building the case (`crates/tui-testing/src/conformance/mod.rs:62-104`, `driver.rs:436-445`). Each `Case` wires `f.status` into its props — one line each in `conformance.rs`.
2. `TextInput::draw` paints `design.motion.spinner_frames[0]` into its trailing marker cell when `live` contains `BUSY | LOADING` (the cell already exists for `ERROR`, `input.rs:876-886`), and declares `Part::ICON` in `PARTS`.
3. §16.2 case 9's description gains: *“the driver makes the forced state real — a state whose affordance comes from props (`Status`, `checked`, `disabled`) is set on the props too, or the case proves nothing about it.”*

**(c) Fix the MA-8 guard, which is what let the narrowing through [F7].** `mono_states_required_by` (`crates/tui-testing/src/conformance/mod.rs:127-148`) becomes a union rather than an `if/else if` chain:

```rust
pub fn mono_states_required_by(caps: Caps) -> Vec<StateFlags> {
    let mut out = vec![StateFlags::empty()];
    if caps.contains(Caps::FOCUSABLE)   { out.push(StateFlags::FOCUSED); }
    if caps.contains(Caps::ACTIVATES)   { out.push(StateFlags::PRESSED); }
    if caps.contains(Caps::DISABLEABLE) { out.push(StateFlags::DISABLED); }
    if caps.contains(Caps::EDITS)       { out.push(StateFlags::EDITING); }
    if caps.contains(Caps::COLLECTION)  { out.push(StateFlags::SELECTED); }
    out
}
```
A component declaring `EDITS | DISABLEABLE` is then required to keep both, which is the assertion MA-8 was written to make.

**(d) Narrowings to revert, once (a)–(c) land.**

| Case | Today | After | Why |
|---|---|---|---|
| `TextInputCase` (`conformance.rs:332-340`) | `{default, FOCUSED, EDITING, ERROR}` | **add `DISABLED`**, and `BUSY` once (b2) lands | `Caps::DISABLEABLE` is declared; (a) makes it pass; (b2) makes `BUSY` real |
| `ButtonCase` (`:236-244`) | `{default, FOCUSED, PRESSED, DISABLED}` | **add `BUSY`** | `Button` already paints the spinner **[F5]**; only the fixture was missing it |
| `FieldCase` (`:388-397`) | `{default, FOCUSED, EDITING, DISABLED, ERROR}` | unchanged; **drop the forced-`LABEL` dependence from its rationale** | it passes on its own `LABEL` today; after (a) it passes on `FIELD`/`TEXT` too, so case 9 proves what §29 requires rather than what the chrome happens to paint |
| `ListCase` (`:492-503`) | narrows `EDITING`, `BUSY`, `LOADING`, `ACTIVE` | keep narrowed, **with the comment corrected**: `BUSY` stays narrowed only until `List` paints a readiness affordance (4E/4F), and that is a named obligation, not a permanent exemption | a list is neither an editor nor the `ACTIVE` element of a strip; `BUSY` is a missing affordance, not a missing rule |
| `TabsCase` (`:590-598`) | narrows `ACTIVE` | keep narrowed, **and document why**: a tab strip's `ACTIVE` is reached through forced `SELECTED` (`tabs.rs:664-671` **[F8]**); forcing `ACTIVE` directly would paint nothing, and making it paint would make `SELECTED` and `ACTIVE` produce identical output and fail case 9's pairwise distinctness | an undocumented narrowing is indistinguishable from an oversight |
| `DialogCase` (`:684-691`), `ScrollRegionCase` (`:747-750`), `PropsCase` (`:787-790`) | heavy narrowing | unchanged | no `DISABLEABLE`/`EDITS`/`COLLECTION` caps; the union guard in (c) confirms mechanically |

**Rationale.** §29 is a goal-level requirement (“component state remains readable without relying only on colour”) and MA-8 exists to stop a component narrowing its way out of it — yet the mono table has no rule that reaches the parts a *text control* actually paints **[F1][F2]**, so `TextInputCase` had no honest choice but to narrow. Fixing the case without fixing the table would be a lie; fixing the table without fixing the `mono_states_required_by` chain leaves the same escape open for the next component. And `Fg(Faint)` at mono is a rule that actively produces an unreadable frame **[F4]** — the one place where the spec, not the implementation, is wrong.

**Rejected alternatives.**
- *`(Part::CONTAINER, DISABLED)` instead of `FIELD`/`TEXT`.* Does not reach the defect: a text control fills `FIELD`, not `CONTAINER` (`input.rs:788-789`). It would move every component's mono `disabled` baseline while leaving `TextInput` exactly as broken.
- *Give `DISABLED` a strike-through or a bracket glyph, like mono `PRESSED`.* `STRIKETHROUGH` is not universally rendered and reads as “deleted”, not “disabled”; a bracket glyph steals two columns from a field's content and would change layout, not only style — a mono fallback must never change geometry.
- *Add mono `StateRule`s for `BUSY`/`LOADING` binding a static “busy” `GlyphRole`.* Requires a new `GlyphRole` (there is none; `GlyphRole::ALL` has 39 entries, `theme/glyph.rs:98-138`), duplicates `design.motion.spinner_frames` as a second source of truth for the same affordance, and still paints nothing in a component that lays out no icon cell — the rule would be inert exactly where the gap is.
- *Leave `Fg(Faint)` and accept black-on-black at mono.* It satisfies case 9 (which excludes colour) while failing the sentence case 9 exists to enforce. Passing the test by making the frame unreadable is the definition of a test proving the wrong thing.

**Tests.**
- `theme::mono_disabled_is_dim_and_readable` *(new, `§16.1 theme/`)* — under `ColorLevel::Mono`, for `Family::{INPUT, FIELD, LIST, BUTTON}` × `Part::{FIELD, TEXT, LABEL}`: the resolved style contains `Modifier::DIM`, and `fg` is not equal to the resolved `bg` of `Surface::Canvas` (the black-on-black assertion, and the one that would have caught **[F4]**).
- `theme::mono_appends_one_state_rule_per_family` — `MONO_RULES_PER_FAMILY == 18`.
- `conformance::text_input::mono_states_are_distinguishable`, `conformance::button::mono_states_are_distinguishable`, `conformance::field::mono_states_are_distinguishable` — with the widened `mono_states()`.
- `conformance::mono_states_required_by_is_a_union` *(new, suite-level)* — a synthetic `Caps::EDITS | Caps::DISABLEABLE` requires both `EDITING` and `DISABLED`.
- `render::components::{text_input,field,list,button}::disabled` — mono baseline lines move; re-bless with a `docs/visual-changes.md` entry classified under §20.10 (see Risks).

---

## Risks

1. **Baseline movement (P6).** The mono `disabled` lines of `crates/tui/tests/baselines/components.txt` change for `text_input`, `field`, `list`, `button` and `tabs` (and the `mono` half only — the truecolor lines are untouched, because mono rules are appended only at `ColorLevel::Mono`, `downgrade.rs:245-253`). §16.3's order is binding: **change → capture → classify → bless**, with a `docs/visual-changes.md` entry before the bless or `xtask bless-guard` fails. Classify as a new §20.10 item: *“mono `DISABLED` gains `DIM` on `FIELD`/`TEXT` and stops tinting the foreground into the background.”*
2. **`inherit_forced` on `FieldControl` (P5)** is a public-trait change with a defaulted method; it is additive for the trait but `TextInput`'s impl must forward it or the fix is inert. `architecture::every_component_doc_has_the_standard_sections` will want the method documented.
3. **Widening the diagnostic (P3)** may surface `UndeliveredIntent` in code that is “working” today by silently dropping a `FocusOut`. That is the point, but it can turn `*::no_diagnostics_are_emitted_during_the_journey` red in Slice 4 for reasons unrelated to the slice; run it before the slice opens.
4. **P1's split expires by convention, not by a gate.** If the Slice-5 obligation is not written into `REFACTORING_STATE.md`, the two halves stay split forever. That ledger entry is the whole mitigation.
5. **P2's two-target split fails open** if a gate command says `--test render` alone; the §16 runner-table amendment is the mitigation and must land in the same commit.
6. **P3's premise correction is inferred, not executed.** I could not run the gated shape (read-only, no shell). The reasoning is `dialog.rs:587` (Decorative) → `hit.rs:313-317` (`delivers_to` requires Control/Part) → `runtime.rs:634` (guard). Verify with the first acceptance command below **before** editing the comment at `tests/showcase_buttons.rs:476-482`.

---

## Executable acceptance conditions

```bash
# ── P3, first: confirm the premise correction before changing anything ──
#  add a temporary gated-shape variant of the Roster fixture; today it must
#  drop the dismissal WITHOUT a diagnostic (the bug), and after the fix it
#  must emit exactly one UndeliveredIntent { owner: CONFIRM }.
cargo test -p tui-next --test showcase_buttons no_diagnostics_are_emitted_during_the_journey
cargo test -p tui-next --lib runtime::tests::a_layer_owners_dismissal_is_diagnosed_when_the_owner_does_not_drain_it
cargo test -p tui-next --lib runtime::tests::a_decorative_owner_is_not_diagnosed_for_a_pointer_intent
cargo test -p tui-next --test overlay                      # the unconditional shape stays green

# ── P6 (priority) ──
cargo test -p tui-next --lib theme::downgrade::tests::mono_appends_one_state_rule_per_family
cargo test -p tui-next --lib theme::downgrade::tests::mono_disabled_is_dim_and_readable
cargo test -p tui-next --test conformance conformance::text_input::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::button::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::field::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::list::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::tabs::mono_states_are_distinguishable
cargo test -p tui-next --test conformance conformance::mono_states_required_by_is_a_union
# the narrowing is now visible in one place, and every entry carries a reason
! rg -n 'fn mono_states' crates/tui/tests/conformance.rs -A2 | rg -v '///|//|const|&STATES|STATES:|\]|\}'

# ── P6 baselines: change → capture → classify → bless, in that order ──
cargo test -p tui-next --test render_components render::components::  # RED on the mono disabled cells
BLESS=1 cargo test -p tui-next --test render --test render_components
git diff --stat crates/tui/tests/baselines/components.txt              # mono lines only
rg -n 'mono DISABLED' docs/visual-changes.md                           # the classification exists

# ── P4 ──
cargo test -p tui-next --lib components::dialog::tests::layer_size_is_a_pure_function_of_props_and_design_tokens
cargo test -p tui-next --lib components::dialog::tests::a_prompt_dialog_sizes_its_own_field_row
! rg -n 'centered|resolve_anchor' crates/tui/src/components/          # §26.5 still holds

# ── P5 ──
cargo test -p tui-next --lib components::dialog::tests::a_forced_dialog_registers_no_control
cargo test -p tui-next --lib components::field::tests::a_forced_field_registers_no_control
cargo run -p xtask -- boundary --check state_override_is_used_only_in_apps_and_fixtures
! rg -n 'pub fn inherit_forced' crates/tui/src --glob '!field_control.rs'

# ── P1 ──
cargo run -p xtask -- boundary --check state_override_is_used_only_in_apps_and_fixtures   # unchanged allow-list
cargo test -p tui-next --test showcase_buttons                        # the matrix stays asserted
cargo build -p tui-next --example showcase_buttons                    # the page still runs
rg -n 'apps/showcase' REFACTORING_STATE.md                            # the Slice-5 obligation is recorded

# ── P2 ──
cargo test -p tui-next --test render --test render_components
cargo test --workspace -- --list | rg -c '^render::components::'      # non-zero
rg -n -- '--test render\b' COMPONENT_ARCHITECTURE.md | rg -v 'render_components' && exit 1   # no gate names one target

# ── the whole gate ──
cargo run -p xtask -- boundary
cargo run -p xtask -- doc-check
cargo test --workspace --test architecture every_named_test_exists
```

**Gate pass condition.** Every command exits 0; `crates/tui/tests/allow/legacy_api.txt` and `allow/domain.txt` stay empty; `docs/visual-changes.md` carries the mono-`DISABLED` entry before any baseline is blessed; §11.4, §12.1, §13, §16.2 case 9, §16.3, §17 example 11, §18.3 #4 and §26.1 carry the amendments above, each mirrored in `REFACTORING_STATE.md` per the change-control rule at `COMPONENT_ARCHITECTURE.md:3`.

**Relevant paths:** `/Users/donbeave/Projects/terminal-components-claude/crates/tui/src/theme/downgrade.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui/src/theme/builtin/mod.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui/src/components/{dialog,button,field,input,list,tabs,mod}.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui/src/{runtime,intent,hit,field_control}.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui-testing/src/conformance/{mod,driver}.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui/tests/{conformance,render_components,overlay,showcase_buttons}.rs`, `/Users/donbeave/Projects/terminal-components-claude/crates/tui/examples/{09_composed_dialog,11_small_app,showcase_buttons}.rs`, `/Users/donbeave/Projects/terminal-components-claude/xtask/src/main.rs`.
