# Migration from the old experimental API

This maps the pre-refactor root package (`src/core/`, `src/ui/`,
`src/widgets/`) onto the final library package at `crates/tui`
(`junie-tui`, Rust path `junie_tui`).

Two ground rules for reading it:

- **"Before" blocks are legacy illustrations.** They are the old API and they
  do not compile against the new crate. They are shown to make the mapping
  concrete, nothing more.
- **"After" blocks compile against the current tree.** Where a replacement
  does not exist yet, the row says so explicitly rather than describing
  something that is not there.

See the note at the top of [`quickstart.md`](quickstart.md) for the final crate
name and package layout.

---

## Part 1 — the concepts

### `Outcome` → `Response<A>`

The old API returned an event-handling verdict in nine different shapes:
`Outcome`, `(Outcome, Option<Event>)`, `(Outcome, bool)`, `bool`,
`Option<Event>`, a polled `result` field, and `()` with the owner expected to
infer what happened. `Outcome::Ignored | Consumed | Changed` carried no
action, so every widget invented its own second channel.

There is now exactly one:

```rust
pub struct Response<A = ()> { /* flow, invalidation, state, action, id */ }
```

**Before**

```rust
// legacy — does not compile against the new crate
let (outcome, activated) = button.on_key(&key);
if activated { self.saves += 1; }
if outcome == Outcome::Changed { self.dirty = true; }
```

**After**

```rust
Button::new(SAVE, "Save")
    .update(cx)
    .on_activated(|| self.saves = self.saves.saturating_add(1))
```

`Response` is `#[must_use]` — dropping it silently loses the consumed /
repaint answer the runtime needs, which was a real class of bug. Fold with
`|=`, read with `action_ref()` / `into_action()` / `take_action()`, or handle
and erase with `on_action(…)` / `on_activated(…)`:

```rust
pub fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
    let mut r = Response::ignored();
    r |= TextInput::new(NAME).update(cx, &mut self.st, &mut self.value).erase();
    r |= Response::changed();
    r
}
```

The `Flow` and `Invalidate` axes are now separate, which fixes the
boundary-wheel inconsistency: a wheel at the end of a scroll region is
`consumed` **without** `Repaint`. Ten widgets used to return `Changed` there
and two returned `Consumed`; there is one rule now, implemented once in
`ScrollState::wheel`.

`Outcome` itself is **deleted**.

### `owns` / `locate` / `row_id` / `close_id` → the runtime resolves it

Fourteen widgets had an `owns(WidgetId) -> bool` and a `locate(WidgetId)`
with four different return types, plus per-widget `row_id`, `cell_id`,
`header_id`, `close_id`, `tab_id`, `new_id`, `left_id`, `right_id`,
`toggle_id`, `option_id`, `chip_id`, `add_id`, `lead_id`, `brand_id`,
`rownum_id`, `more_id`, `bar_ids` and `scrollbar::id_for`. Three widgets drew
an interactive scrollbar with no handler at all.

All of it is **deleted**. Input is resolved *before* your code runs:
`draw` registers regions with `ui.register_part(owner, PartRef, rect)`, and
`update` receives `Intent::Pointer { part: PartRef { part, item }, .. }` with
the key already attached.

**Before**

```rust
// legacy
if list.owns(clicked_id) {
    if let Some(row) = list.locate(clicked_id) {
        self.chosen = Some(self.items[row].id);   // index, not identity
    }
}
```

**After**

```rust
orders_list()
    .update(cx, &mut self.list, &self.orders)
    .on_action(|a| {
        if let ListAction::Chose(k) = a {
            self.chosen = self.orders.iter().find(|o| ItemKey::num(o.id) == k).map(|o| o.id);
        }
    })
```

`ListAction::Chose` carries an `ItemKey`, never a display index.

### `WidgetId::child(usize)` → `ItemKey`

`WidgetId` is **deleted**. `Id` replaces it, and `WidgetId::child(index)` — the
old default child mechanism — is deleted as a *collection identity*
mechanism. Positional identity was the cause of "focus jumped to the wrong
row after a delete", "the pending close targeted the tab that took its
place", and "the edit was applied to a different record".

| Old | New | Meaning |
|---|---|---|
| `WidgetId::of("a.b")` | `Id::root("a.b")`, or `id!("b")` | a root id; `id!` prefixes `module_path!()` |
| `WidgetId::sub("x")` | `Id::sub("x")` | a named child |
| `WidgetId::child(3)` | `Id::index(3)` | **positional, UNSTABLE** under insert/remove/reorder; debug-asserted, kept only for genuinely positional cases |
| *(nothing)* | `Id::part(Part)` | a child **component** id inside a container (a `Button` inside a `Dialog`) |
| *(nothing)* | `Id::item(ItemKey)` | a keyed child, stable under reorder |
| *(nothing)* | `PartRef::of(p)` / `PartRef::item(p, k)` | a **sub-region** of one component — not an id at all |

`ItemKey` is `Index(usize)` (documented as unstable), `Num(u64)` for a domain
key, or `Text(u64)` for a hashed textual or composite key.

```rust
const DIALOG: Id = id!("confirm");
const DIALOG_FIELD: Id = DIALOG.part(Part::FIELD);   // a child COMPONENT

List::new(ROWS).key(|o: &Order| ItemKey::num(o.id))  // a keyed collection
```

`Id` also gained a separator and a kind byte, so `root("a").sub("b")` and
`root("ab")` are provably distinct — the old FNV concatenation collided.
`Debug` prints a readable path in debug builds instead of a hex hash.

### Raw `bg: Color` parameters → a contextual `Surface`

Twenty-four render methods took `bg: Color`, and the ones that did not
hard-coded their own plane instead (`t.surface_elevated`, `t.popover`).
`Panel::bg(&Theme) -> Color` existed purely so callers could extract a value
and hand it back down to every child — a manual surface-inheritance protocol.

All of it is **deleted**. The background plane is contextual:

**Before**

```rust
// legacy
let bg = panel.bg(theme);
input.render(area, buf, ctx, bg);
list.render(rows[1], buf, ctx, bg);
```

**After**

```rust
ui.with_surface(Surface::Elevated, |ui| {
    TextInput::new(NAME).draw(ui, area, &st);
});
```

The ladder is `Canvas → Surface → Elevated → Overlay → Popover`, plus the
non-ladder `Field` / `FieldHover` planes. A layer sets its own surface
(`Modal → Overlay`, `Popover`/`Tooltip → Popover`). Inside a component you
read `ui.surface()`, `ui.bg()` and `ui.surface_style()`; you never take a
colour as a parameter. `Theme::raise` is ladder-index arithmetic rather than
the old colour-equality dispatch, so a theme with two identical plane colours
still raises correctly.

The one remaining raw-colour path is `Role::Custom(Color)` — documented,
still capability-downgraded, and the escape hatch rather than the road.

### Component-local forced state → `Ui::reference`

Reference galleries and conformance fixtures no longer mutate a component builder. They make the
whole draw subtree inert at the `Ui` boundary and, when testing runtime-owned paint, name exactly
one target:

```rust
use junie_tui::{ReferenceState, ReferenceTarget, Ui};

let target = ReferenceTarget::new(SAVE, ReferenceState::FOCUSED)
    .part(junie_tui::PartRef::of(junie_tui::Part::CONTAINER));
ui.reference(Some(target), |ui| {
    Button::new(SAVE, "Save").draw(ui, area);
});
```

Use `ui.reference(None, |ui| …)` for a default or semantic-only reference. Either form suppresses
hits, focus stops, bindings, cursor, layout requests and layers throughout the subtree, including
controls drawn by a callback or slot. Only `FOCUSED`, `FOCUS_VISIBLE`, `HOVERED` and `PRESSED` can
be injected. Selection, editing, disabled/read-only and readiness still come from the same props or
state as a live component. A composite reference therefore targets one owned child/item/part; it
does not broadcast a root state to every descendant.

The old component `state_override` builders and crate-local `inherit_forced` propagation are
deleted. Migration is atomic: wrap the outer reference draw, map runtime flags to one
`ReferenceTarget`, preserve semantic fixture state, then remove the old builder call.

### `DialogBody` → a body closure

`DialogBody { Text, Input, Facts { facts, code, ack } }` was a closed enum
with a SQL-preview variant and a TablePlus typed-acknowledgement variant baked
in. It is **deleted**.

**Before**

```rust
// legacy
let dlg = Dialog::confirm(id, "Delete table", "This cannot be undone.", "Delete");
dlg.body = DialogBody::Facts { facts, code: vec![sql], ack: Some(ack_input) };
// … later, poll the widget:
if let Some(DialogResult::Action(0)) = dlg.result { … }
```

**After** — the body is an arbitrary closure that borrows application data
(`crates/tui/examples/09_composed_dialog.rs`):

```rust
ui.layer(CONFIRM, |ui, area| {
    confirm_dialog().draw(ui, area, &self.dlg, |ui, body| {
        let fields = [("Table", self.target.as_str()), ("Rows", "12,481")];
        let props = Props::new(&fields);
        let natural = [props.measure(ui, Constraints::loose(body.width, body.height)).preferred.1];
        let rows = layout::rows_measured(
            body,
            &[Track::Auto, Track::Fixed(1), Track::Flex(1)],
            &natural,
        );
        props.draw(ui, rows[0]);
        ui.rule(rows[1]);
        TextInput::new(TOKEN)
            .value(&self.token)
            .placeholder("Type the table name to confirm")
            .draw(ui, rows[2], &self.token_st);
    });
});
```

The polled `result` field is gone too: `Dialog::update` returns
`Response<DialogAction>` with `Action(ActionKey)` and
`Dismissed(DismissReason)`. Actions are named by `ActionKey`, not by an index
into a `Vec<Button>`, so inserting a button cannot silently rebind a handler.

Arming a destructive action is an `update` predicate, not a render-time
mutation:

```rust
let armed = self.token.trim() == self.target;
let actions = [
    Action::new(K_CANCEL, "Cancel"),
    Action::danger(K_DELETE, "Delete").enabled(armed),
];
```

Focus trapping, the backdrop, Esc, click-outside, focus restoration and the
hint row come from the **layer**, not from the dialog. The old
`dialog.rs`'s trap-less modal and its hand-rolled backdrop loop are deleted.

### fn-pointer extension points → traits and `&dyn Fn`

Six extension points were bare `fn` pointers and therefore could not capture
application state — a connection, a schema, a locale, a config:

`grid::Validator`, `table::validator`, `input::validator`,
`code::Highlighter`, `code::Segmenter`, `panel`'s `style_line`, and
`field_common::EditAction::Apply(fn(&mut TextBuffer))`.

**Before**

```rust
// legacy
pub validator: Option<fn(&str) -> Option<String>>,
```

**After** — a trait with a blanket impl for closures, taken as `&dyn Validate`:

```rust
pub trait Validate {
    fn check(&self, s: &str) -> Result<(), FieldError>;
}
impl<F: Fn(&str) -> Result<(), FieldError>> Validate for F { … }
```

```rust
impl Form {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        // The rule captures application state — impossible with a `fn` pointer.
        let max = self.max_len;
        let rule = move |s: &str| {
            if s.chars().count() <= max {
                Ok(())
            } else {
                Err(FieldError::coded("Too long", "len"))
            }
        };
        TextInput::new(NAME)
            .validate(&rule)
            .update(cx, &mut self.st, &mut self.value)
            .erase()
    }
}
```

A plain `fn` item still works through the same blanket impl
(`crates/tui/examples/06_validated_field.rs` uses `.validate(&valid_email)`).
`FieldError` carries a `message: Cow<'static, str>` and an optional
machine-readable `code`, replacing the bare `Option<String>`.

`EditAction::Apply(fn(&mut TextBuffer))` is **deleted**; the shared edit
keymap is a `const [Binding<EditAction>]` table.

### Public `area: Rect` fields → `FrameRead::area`

Twenty-one widgets had `pub area: Rect`, plus three `pub areas: Vec<Rect>`,
`pub brand_area` and two `pub anchor`s. They were written during render and
read during event routing, so any frame in which a widget returned early on a
tiny or empty rect left **stale geometry** that the next click used.

All **deleted**. Geometry is read from the runtime's registry, which knows
which frame it came from:

**Before**

```rust
// legacy
if button.area.contains(mouse_pos) { … }
```

**After**

```rust
let anchor = cx.area(OWNER_BTN).unwrap_or_default();
```

`FrameRead` is implemented by both `Cx` and `Ui`, and gives you `state(id)`,
`theme()`, `design()`, `area(id)` and `layout(id)`. `area` and `layout` return
**last** frame's facts and `None` on frame 1 — the honest answer, and the one
that makes anchoring a popover to a button correct rather than accidentally
correct. `Registry::area_of_part(id, PartRef)` gives a sub-region's rect.

A component that could not draw registers **nothing**, so a stale rect cannot
be read at all.

### Render-time commits → a compile error

The old `Application::render(&mut self)` sanctioned mutation during
rendering, and eight sites took it up: committing a text edit
(`input.rs:282`), a textarea commit (`textarea.rs:202`), a code-editor commit
(`code.rs:611`), arming a dialog acknowledgement (`dialog.rs:465`), closing a
select popup (`select.rs:161-167`), moving a menu cursor on hover
(`menu.rs:243`), and a diff-view `set_follow`/`scroll_to` inside the layout
cache.

`App::draw(&self, ui: &mut Ui<'_>)` takes `&self`, and every component's
`draw` takes `&self` and `&XState`. Each of those eight is now a borrow-check
error, not a review finding. Move the transition into `update`:

**Before**

```rust
// legacy — inside render()
if self.blur_pending { self.commit(); }
```

**After**

```rust
fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
    let r = email_input().update(cx, &mut self.email_st, &mut self.email);
    if let Some(TextAction::Committed) = r.action_ref() {
        self.server_error = check_uniqueness(&self.email);   // application effect
    }
    r.erase()
}
```

The two draw-phase writes that *are* legitimate — `Ui::declare_state` and
`Ui::report_layout` — go through `&mut Ui`, are consumed by the runtime, and
are read back on the **next** frame.

### Hard-coded key handling → `Binding` tables

All eighteen interactive modules hard-coded their chords inline, and
`MenuItem::shortcut: &'static str` was a *rendered label only* with no binding
behind it, so display and behaviour could silently diverge.

A component now publishes a `&'static [Binding<Cmd>]` table with a label, a
hint priority and a visibility flag, and `impl Bindings` selects a table per
`BindingState`. The same table is what the shared `HintBar` renders and what
`update` matches, so the two cannot disagree — the conformance case
`bindings_match_handled_keys` asserts it in both directions. The ~700 lines of
hand-written hint tables in the three applications are deleted.

An application adds, removes or remaps chords through `KeyMap` with a
`KeyPhase` (`Capture` before dispatch, `Bubble` after), returning an
`ActionKey`. `App::keymap()` supplies it.

### Owned data → borrowed data with a row renderer

Every old collection required owned `String`s rebuilt every frame, and none
accepted a custom row or cell renderer:

**Before**

```rust
// legacy
list.items = orders.iter()
    .map(|o| ListItem::new(o.customer.clone(), format!("{}", o.total_cents)))
    .collect();                      // N allocations per frame
```

**After** (`crates/tui/examples/07_borrowed_rows.rs`)

```rust
fn order_row(o: &Order, r: &mut RowUi<'_>) {
    if o.flagged {
        r.marker(GlyphRole::WarningMark);
    }
    r.label(&o.customer);            // borrowed &str, one grapheme walk, 0 allocs
    r.part(Part::META, 12)           // 12 columns reserved from the right
        .money(o.total_cents)
        .align(Align::Right)         // formats into the cell, no String
        .tone(if o.total_cents < 0 { Role::Danger } else { Role::Fg(FgStep::Muted) });
}
```

Only visible rows invoke the renderer. Selection lives in the caller-owned
`ListState` as a `KeySet` of `ItemKey`s, and `SelectMode` gained `Range` and
`None` beside `Single` and `Multi`.

### Other deletions worth knowing

| Deleted | Why |
|---|---|
| `Focus`'s single `barrier: Option<usize>` | replaced by focus **scopes** with `ScopeMode::Trap` and a restore map; a single barrier cannot express nested overlays |
| public `&mut Focus` / `&mut FocusRing` parameters | 186+ direct manipulation sites across the apps; focus is runtime-owned |
| `HitRegistry` barriers | replaced by `layer: LayerId` per region |
| `ScrollState`'s public fields | eight bypass sites in `grid`/`table` mutated the offset behind the widget's back |
| `Theme`'s flat 30-field `Copy` struct, `lift`, `backdrop`, the `for_level` macro, `Theme::change_glyph` | replaced by tokens + recipes, ladder arithmetic and `map_colors` |
| `RenderCtx`, `Interaction`, `begin_modal`, `focus_hidden`, public `hits` / `ring` | replaced by `Ui` / `Cx` |
| both `Placement` enums and both placement algorithms; `Rect::centered` in `dialog.rs`; the shared `WidgetId::of("popup.surface")` | one layer resolver: flip, clamp, clip, min-size |
| `ui::text::fit` / `fit_right` on render paths | replaced by `measure` + degenerate-rect rules |
| `DataTable` | absorbed by `Grid` with `NavUnit::{Row, Cell}` — its `Column`, `Cell`, third `EditState`, string sort, `validator: fn`, `locate`/`locate_header`, double cell registration and four ragged-row panics all go |
| `ScrollPanel` | callers move to `TextViewport` with tone-carrying spans |
| `Panel::bg`, `pub bg_override` | contextual `Surface` |
| SQL/database vocabulary inside `grid` (`CellValue`, `PendingChanges`, `UndoAction`, `default_validator`, `"Preview SQL"`, `primary`/`nullable`/`references`/`enum_values`) | moved to the application; the generic grid never learns about SQL |
| `MeterTone::{Warning, Exhausted, Stale, Refreshing}` | Jackin quota lifecycle, moved to the application |

---

## Part 2 — foundation modules

| Old | New | Notes |
|---|---|---|
| `src/core/event.rs` | `event.rs` (`Input`, `Key`, `Chord`, `Mouse`, `MouseKind`, `Axis`) + `response.rs` (`Response`, `Flow`, `Invalidate`, `StateFlags`) + `intent.rs` (`Intent`, `Phase`, `FocusVia`, `IntentIter`) | `Outcome` deleted. `Mouse` gains `mods`; `Secondary`/`SecondaryUp`/`Wheel(Axis, i16)` added — right and middle drag were silently dropped before |
| `src/core/focus.rs` | `focus.rs` (`FocusRing`, `FocusEntry`, `FocusState`, `ScopeId`, `ScopeMode`, `Focusability`, `FocusVis`) | the single barrier becomes scopes + traps; adds a restore map and disabled-but-registered entries |
| `src/core/hit.rs` | `hit.rs` (`Registry`, `Region`, `RegionKind`, `Hit`, `Headroom`, `Axes`) + `capture.rs` (`Capture`) | regions carry a `PartRef`, so twelve `locate` helpers die; `Capture` deletes cached-rect reconstruction in four widgets |
| `src/core/id.rs` | `id.rs` (`Id`, `ItemKey`, `Part`, `PartRef`, `id!`) | see above |
| `src/core/scroll.rs` | `scroll.rs` (`ScrollState`) | public fields removed; adds `ensure_visible_on_next_layout` |
| `src/core/text.rs` | `text/{buffer,editor}.rs` (`TextEditorCore`, `EditAction`, `EditOutcome`, `CursorPos`) | `TextBuffer`'s derived `Debug`/`Clone` over raw bytes replaced by a redacting `Debug`; zeroize added |
| `src/ui/text.rs` | `text/{measure,fuzzy,span}.rs` (`width`, `wrap`, `wrapped_rows`, `truncate`, `truncate_middle`, `fuzzy`, `Span`) | `width` is now the one width function in the workspace; `fuzzy` returns grapheme indices into the *original* label, fixing a latent mis-highlight |
| `src/runtime.rs` | `runtime.rs` (`Runtime<A>`, `App`) + `runtime/session.rs` (`TerminalSession`, `DefaultTerminal`, `run`) + `diagnostics.rs` (`Diagnostic`) | `Application::render(&mut self)` deleted; adds the two-phase frame, the layer compositor, `request_repaint_after` and `diagnostics()` |
| `src/theme.rs` | `theme/{mod,tokens,role,glyph,recipe,patch,resolve,downgrade,builder,border}.rs` + `theme/builtin/{junie,paper}.rs` | see [`theming.md`](theming.md); Junie token values preserved verbatim |
| `src/ui/ctx.rs` | `ui/{mod,cx,paint,derived,layer_buf}.rs` (`Ui`, `Cx`, `LayoutFacts`) | adds a clip rect, a surface stack, a style stack, a written-cell bitset and the `raw()` escape hatch |
| `src/ui/layout.rs` | `layout.rs` + `measure.rs` | `Split`'s vertical/horizontal minima asymmetry fixed; absorbs `button::row_layout*` and the showcase's private `rows`/`columns`/`caption` |
| `src/ui/popup.rs` | `layer.rs` (`LayerSpec`, `LayerId`, `LayerKind`, `LayerSize`, `Anchor`, `Side`, `CrossAlign`, `Dismiss`, `Backdrop`, `LayerEvent`, `DismissReason`) | one resolver replaces two `Placement` enums and two placement algorithms |

---

## Part 3 — the widget modules

Status is against the current **Slice 4** library tree. "not yet" means the
type does not exist; it is scheduled, not silently dropped. A ✅ library API
does not mean the staged application moves have happened: the apps land in
`apps/showcase`, `apps/tablepro`, and `apps/jackin-preview` across Slices 5–7,
while `junie-tui` remains the library package until the post-Slice-7 rename.

| Old module | New | Status | Notes |
|---|---|---|---|
| `brand` | `Brand` | ✅ | "the only control that fills with the accent" moves from a doc rule in code to a `Theme::junie()` recipe default; `pub area` removed |
| `button` | `Button` | ✅ | `(Outcome, bool)`/`bool` → `Response<Activated>`; `bg: Color` → `Surface`; `pub area` → `cx.area(id)`; `row_layout*` → `layout::action_row`. The reference implementation of the public-API conventions |
| `chips` | `ChipBar`, `LabelChips` | ✅ | raw `ctx.ring.register` deleted; the drop-out-of-ring-on-overflow bug fixed; TablePro's `"+ Add filter"` / `"match all ▾"` defaults removed; keys via `ItemKey` |
| `choice` | `Checkbox`, `Toggle`, `RadioGroup`, `LabelRadio` | ✅ | three `pub area`/`areas` removed; `RadioGroup::height()` deleted (`Field` measures); **cursor and value separated** — arrows no longer commit a value |
| `dialog` | `Dialog`, `DialogState`, `Action`, `ActionKey`, `DialogAction` | ✅ | see `DialogBody` above |
| `empty` | `EmptyState` (in `collection/`), `Empty` | ✅ | the free `render(…, bg)` deleted; empty / loading / partial / error become one vocabulary, absorbing `PickerStatus` |
| `field_common` | dissolved into `text/editor.rs` + `keymap.rs` | ✅ | `EditAction::Apply(fn)` deleted; the shared keymap is a `const` `Binding` table |
| `hintbar` | `HintBar`, `HintLayer`, `Hint` | ✅ | layers are now *derived*: top layer ▸ mode ▸ the focused component's visible bindings ▸ screen extras ▸ global |
| `input` | `TextInput`, `TextInputState`, `BlurPolicy`, `Secret`, `SecretPolicy` | ✅ | render-time commit+validate impossible; `validator: Option<fn>` → `&dyn Validate`; `plain_label` deleted (`Field` owns chrome); `HEIGHT` deleted (`measure`); five tiny-rect underflows fixed |
| `keyhint` | `KeyHint` | ✅ | rendered by `HintBar`; the one-off entry point stays |
| `list` | `List`, `ListState`, `SelectMode`, `KeySet` | ✅ | owned `ListItem` deleted; `row_id`/`locate`/`owns` deleted; the boundary-wheel violation fixed; `SelectMode` gains `Range`/`None` |
| `panel` | `Panel` | ⛔ not yet | `ScrollPanel` is **removed**, not ported — its callers move to `TextViewport`. `bg: Color`, `Panel::bg(t)` and `pub bg_override` are deleted whatever happens |
| `progress` | `Spinner`, `ProgressBar` | ✅ | five `bg: Color` parameters deleted; `SPINNER` frames move to `DesignTokens::motion` |
| `progress` (meters) | `Meter`, `MeterTone`, `MeterVisual` | ✅ | `METER_LOW_MAX` / `MEDIUM_MAX` move to `DesignTokens::meter`; the jackin-specific tones move to the application |
| `props` | `Props` | ✅ | the **two independent render paths** (a free fn and `PropsList::render`) collapse to one. `PropsList` — the two-column `List` variant — is ⛔ not yet |
| `scrollbar` | `Part::TRACK` / `Part::THUMB` of `ScrollRegion` | ✅ | `scrollbar::id_for` deleted (48+ call sites); one `on_scrollbar` implementation replaces seven copies; thumb drag uses pointer capture |
| `segments` | absorbed by `StatusBar` | ✅ | two priority-drop loops become one `Left`/`Center`/`Right` item strip |
| `select` | `Select`, `SelectState`, `SelectAction`, `LabelSelect` | ✅ | render-time overlay close impossible; the popup is a `Popover` layer, so the old focus-barrier bug disappears; the 10-row clip becomes a real scroll region |
| `statusbar` | `StatusBar`, `StatusItem`, `Emphasis`, `Group` | ✅ | absorbs `segments`; `STATUS_METER_TRACK` moves to `design.size.meter_track` |
| `tabs` | `Tabs`, `TabsState`, `TabsAction` | ✅ | positional `tab_id(i)`/`close_id(i)` deleted → `ItemKey`; per-frame `areas`/`widths` `Vec`s deleted; the "rebuild the widget and rescue `first`/`active`" idiom in both applications deleted |
| `textarea` | `TextArea`, `TextAreaState` | ✅ | render-time commit impossible; shares `TextEditorCore` with `input`; the missing `owns`/`on_scrollbar` supplied by `ScrollRegion`; a 1-cell-width underflow fixed |
| `code` | `CodeEditor`, `CodeEditorState`, `Highlighter`, `Segmenter`, `CodeDiagnostic` | ⛔ not yet | planned: `fn`-pointer `Highlighter`/`Segmenter` → `&dyn Fn`; per-frame `hash_text` → an edit counter; the vim key table → a default `KeyMap` |
| `completion` | `Completion`, `CompletionState`, `CompletionController` | ✅ | `CompletionController::new(editor_id, popup_id)` opens the popover without moving focus; call `Completion::update_for(editor_id, …)` before the editor update so editor-addressed completion bindings win while text remains editor-owned |
| `diff` | `DiffView`, `DiffViewState`, `DiffSource`, `DiffMode` | ⛔ not yet | planned: the data model moves behind `DiffSource`; `review_lines(f, width)` becomes `measure` |
| `grid` | `Grid`, `GridState`, `GridModel`, `GridEditor`, `GridAction`, `GridCmd`, `ColumnKey`, `Column`, `CellRef`, `CellAction`, `NavUnit`, `SortDir` | ✅ | absorbs `table`. `GridModel` has exactly three required methods (`row_count`, `row_key`, `cell`), no associated `Row` and no `col_count`; `Grid::new(id, columns)` is the sole schema authority. `cell` returns `Option<CellRef<'_>>` so `None` is a structural ragged-row hole. `CellRef::new` inherits `Column::align`; `.align(…)` is an explicit override. Everything database-shaped stays in the application adapter |
| `menu` | `MenuBar`, `ContextMenu`, `MenuItem`, `MenuState` | ⛔ not yet | planned: render-time cursor move on hover → an explicit `Intent::Pointer`; `shortcut: &'static str` → a real `Chord` that both renders and binds; label-string dispatch → `ActionKey` |
| `picker` | `FilterList` (headless), `Picker` (overlay), `CommandPalette`, `PickerState`, `ScopeKey` | ⛔ not yet | `crates/tui/examples/10_nested_overlay.rs` substitutes a `List` in a popover, and says so |
| `splitter` + `ui::layout::Split` | `SplitPane`, `SplitPaneState` | ⛔ not yet | planned: one component owning its container rect, drag through capture, four app-side copies deleted |
| `steps` | `Steps`, `StepsState`, `StepsAction`, `StepsCmd`, `StepState` | ✅ | stays a display or navigable lifecycle rail with a runtime-cached derived frontier; call `StepsState::invalidate()` after lifecycle regression or same-length interior reorder. The future step *flow* remains a separate `Wizard`. Lifecycle `Part::META` is component-owned; caller `RowUi` metadata remains row-owned. Component patches reach `CONTAINER`/`LABEL`, while arbitrary row `META`/`CELL`/custom parts remain isolated; patch, part-patch and slot overrides propagate to the embedded scrollbar |
| `table` | — | ⛔ deleted, not replaced | absorbed by `Grid` |
| `tree` | `Tree`, `TreeState`, `TreeNode`, `TreeAction` | ⛔ not yet | planned: `path: Vec<usize>` identity → `ItemKey`; `flatten()` becomes incremental and borrow-based |
| `viewport` | `TextViewport`, `ViewportState`, `ViewportAction` | ⛔ not yet | `Span<'a>` has **already** moved to `text/span.rs` and is re-exported at the root; the viewport itself is not ported |
| *(showcase sidebar)* | `NavList`, `NavListState`, `NavListAction`, `NavListCmd`, `NavMode` | ✅ | `Enter`/`Space`/click emit `Chose(ItemKey)`; Right/plain `l` emit `EnterContent(ItemKey)`. The shell, not `NavList`, hands focus to content |
| *(application minimum-size notice)* | `TooSmall` | ✅ | uses the dedicated named `Family::TOO_SMALL`, isolated from `Family::PANEL`; exact four-line copy and geometry remain unchanged |

New components that had no old module: `Field` (✅, the label / control / help
/ error chrome that `input`, `choice` and `select` each half-implemented),
`ScrollRegion` (✅), `NavList` (✅) and `TooSmall` (✅).

Also planned and ⛔ not yet: `Form`, `Wizard`, `HelpOverlay`, `PickerChain`.

---

## Part 4 — application-level machinery that simply disappears

None of this is a widget; all of it was hand-written in each of the three
applications and is now library behaviour:

- three `Focus` / `FocusRing` field sets and 186+ direct manipulation sites;
- three `HitRegistry` fields and six manual re-registration blocks;
- three press / hover / flash / double-click state machines;
- three focus-reconciliation implementations and three `saved_focus` fields;
- three `animating()` / `tick_interval()` heuristics;
- jackin's nine-arm outside-click, nine-arm click-dispatch and nine-arm
  wheel-routing matches;
- four copies of `modal_frame`;
- three form engines (jackin's `FormDialog`, tablepro's `connections.rs` and
  its `FilterEditor`), which collapse into one `Form`;
- four master-detail draggable-seam implementations and their
  `seam_container: Rect` fields;
- three "terminal too small" screens;
- the generic half of the three `PageCtx` / `Cx` request buses — focus,
  layers, capture, repaint, intents and area become `Cx`, while the product
  half (navigation commands, status messages) correctly stays per-application.

---

## A porting order that works

1. **Ids first.** Replace `WidgetId` with `Id`, and every `child(index)` on a
   collection with an `ItemKey` and a `.key(…)` closure. Do this before
   anything else; it is what makes the rest mechanical.
2. **Split `render` into `update` and `draw`.** Move every mutation into
   `update`. The compiler will find them for you — that is the point of
   `draw(&self)`.
3. **Delete `owns` / `locate` / `*_id` helpers and the `pub area` fields.**
   Register parts in `draw`; read `Intent::Pointer { part, .. }` in `update`.
4. **Replace `Outcome` returns with `Response<XAction>`,** and fold with `|=`.
5. **Drop `bg: Color` parameters.** Wrap in `ui.with_surface(…)` where the
   plane genuinely changes.
6. **Move colours into roles.** A literal colour in a component is a missing
   `Role`.
7. **Turn inline key handling into a `Binding` table** and `impl Bindings`.
   The hint bar comes free.
8. **Register a `Conformance` case.** See
   [`authoring.md`](authoring.md#registering-a-conformance-case).
