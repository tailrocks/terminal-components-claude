# Quick start

`junie-tui` is a component library for terminal user interfaces, built on
[`ratatui-core`](https://docs.rs/ratatui-core). It gives you buttons, text
inputs, lists, tabs, dialogs, overlays and a semantic theme system, and it
owns the parts of a TUI that are tedious and easy to get wrong: hit testing,
hover, press, focus traversal, modal focus traps, cursor placement, wheel
routing and layer z-order.

---

> ## Note on the crate name
>
> The library is **`junie-tui`** (Rust path `junie_tui`), and every example in
> these guides is written against that name.
>
> While the refactor is in progress the crate is temporarily published inside
> this workspace as **`tui-next`** (Rust path `tui_next`), because the root
> package still owns the legacy tree during the staged application moves
> (`COMPONENT_ARCHITECTURE.md` §47.1). The application packages land in
> `apps/showcase`, `apps/tablepro`, and `apps/jackin-preview` across Slices
> 5–7. The rename to `junie-tui` / `junie_tui` happens in one scripted commit
> between Slice 7 and Slice 8, after the root package has no legacy sources or
> binaries.
>
> Until then, read every `junie_tui::` in these guides as `tui_next::`, and
> every `junie_tui_testing::` as `tui_next_testing::`. Nothing else changes.
> The in-tree examples under `crates/tui/examples/` carry the temporary name
> and a header comment saying so.

---

## The two API layers

The crate has exactly two public entry points, and which one you use tells
you what job you are doing.

| You are… | You import | You get |
|---|---|---|
| writing an application | `junie_tui::*` | `App`, `run`, every component, the theme, layout, the collection vocabulary |
| writing a **component** | `junie_tui::author::*` | the same theme and layout vocabulary, plus `Ui`'s registration services: focus, hit regions, parts, capture, cursor, scroll, layers |

`junie_tui::author` deliberately does **not** re-export `Runtime`, `run`,
`TerminalSession`, `Registry`, `FocusRing`, `FocusState`, `App` or the
concrete components — a component author drives none of those. It also does
not require any private access: everything a downstream component needs is
in `author::`. See [`authoring.md`](authoring.md).

Application authors never need `author::`. Component authors normally do not
need the root facade.

---

## The frame model

A frame is two passes over your application, in this order:

1. **`update(&mut self, cx: &mut Cx<'_>) -> Response<()>`** — the only place
   semantics change. Input has already been resolved into *intents* addressed
   to component ids, so you read `cx.intents(id)` (or, more usually, let a
   component read them for you) and mutate your own state.
2. **`draw(&self, ui: &mut Ui<'_>)`** — pure paint. It measures, lays out,
   paints cells and registers hit regions, focus stops and parts for the
   *next* frame.

The important part is the signature: `update` takes `&mut self`, `draw` takes
`&self`. That is not a style convention — it is a **compile-time guarantee**.

```rust
fn draw(&self, ui: &mut Ui<'_>) {
    // `self` is shared here. There is no way to commit an edit, close a
    // dialog, move a selection or fire an action from inside `draw`,
    // because every one of those needs `&mut`. The borrow checker rejects
    // it before the program runs.
}
```

The legacy API had `Application::render(&mut self)`, and eight confirmed
sites in the old widget set performed a semantic transition during rendering
(committing a text edit, arming a dialog acknowledgement, closing a select
popup, moving a menu cursor on hover). Those bugs are not fixed by review;
they are unrepresentable.

Two consequences you will notice immediately:

- **Geometry is read from the previous frame.** `cx.area(id)` and
  `cx.layout(id)` return *last* frame's facts, because this frame has not
  drawn yet. That is the trade for the guarantee, and it is why anchoring a
  popover to a button reads `cx.area(BUTTON)`.
- **Props are built once and used by both phases.** Configuration lives in a
  small constructor function that takes no `&self`, so `update` can still
  hand `&mut self.field` to the component alongside it. Every example in the
  tree follows this shape.

---

## A minimal application

`crates/tui/examples/01_button.rs`, in full:

```rust
use junie_tui::{App, Button, Cx, Id, Insets, Response, Theme, Ui, id, layout, run};

const SAVE: Id = id!("save");

#[derive(Default)]
struct Counter {
    saves: u32,
}

impl App for Counter {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(SAVE, "Save")
            .update(cx)
            .on_activated(|| self.saves = self.saves.saturating_add(1))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = layout::inset(ui.full(), Insets { l: 2, t: 1, r: 2, b: 1 });
        Button::new(SAVE, "Save").draw(ui, area);
    }
}

fn main() -> std::io::Result<()> {
    run(Counter::default(), Theme::junie())
}
```

Nothing in that program registers a hit region, computes hover, tracks the
pressed state, derives an internal id, implements Tab traversal or asks which
widget the mouse is over. `Button::draw` calls
`ui.register_control(SAVE, area, Focusability::Focusable)` and the runtime
does the rest. Enter, Space and a mouse click all produce the same
`Activated`.

`id!("save")` expands to `Id::root(concat!(module_path!(), "::", "save"))`, so
ids are unique per module without a naming discipline.

### Adding state you own

Components hold **durable interaction state only** (a scroll offset, a
cursor, an in-flight edit). The value being edited is yours:

```rust
use junie_tui::{Cx, Id, Rect, Response, TextInput, TextInputState, Ui, id};

const NAME: Id = id!("name");

#[derive(Default)]
struct Screen {
    name: String,            // the controlled value — you own it
    name_st: TextInputState, // durable interaction state only
}

impl Screen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextInput::new(NAME)
            .update(cx, &mut self.name_st, &mut self.name)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        TextInput::new(NAME)
            .value(&self.name)
            .draw(ui, area, &self.name_st);
    }
}
```

`draw` takes `&TextInputState`, so committing or validating from the paint
pass is a compile error.

### Colour capability

`run` does **not** sniff the terminal. The theme it receives is the theme it
uses, so downgrade explicitly at startup:

```rust
use junie_tui::{App, ColorLevel, Theme, run};

fn start<A: App>(app: A) -> std::io::Result<()> {
    run(app, Theme::junie().downgrade(ColorLevel::detect()))
}
```

`ColorLevel::detect()` reads `NO_COLOR`, `COLORTERM` and `TERM`. See
[`theming.md`](theming.md) for what the downgrade does to a theme, and why
focus, selection and error states stay readable at `ColorLevel::Mono`.

---

## `Response`

Every `update` returns a `Response<A>`: whether input was consumed, whether a
repaint or a relayout is needed, and optionally one semantic action. It is
`#[must_use]` — dropping it silently loses the answer the runtime needs.

Fold several with `|=`, and erase the action type when you have handled it:

```rust
let mut r = Response::ignored();
r |= TextInput::new(NAME).update(cx, &mut self.name_st, &mut self.name).erase();
r |= Button::new(ADD, "Add").update(cx).on_activated(|| { /* … */ });
r
```

`on_action` / `on_activated` run a closure and return `Response<()>`;
`action_ref` / `into_action` give you the action to `match` on.

---

## Where to go next

| Guide | What it covers |
|---|---|
| [`theming.md`](theming.md) | the twelve customisation scenarios, the six-level precedence chain, `Slot` merge semantics, capability downgrade |
| [`overrides.md`](overrides.md) | `patch`, `patch_part`, part slots, custom variants, recipes — and when to reach for each |
| [`authoring.md`](authoring.md) | writing a component on `junie_tui::author`, and registering it with the twenty-case conformance suite |
| [`migration.md`](migration.md) | the old experimental API mapped to the new one, module by module |

Runnable examples live in `crates/tui/examples/`:

| File | Shows |
|---|---|
| `01_button.rs` | the minimal application above |
| `05_instance_patch.rs` | two buttons, one locally overridden |
| `06_validated_field.rs` | a text field with external validation and an async server error |
| `07_borrowed_rows.rs` | a list over borrowed domain objects with a custom row renderer |
| `08_dynamic_tabs.rs` | tabs whose identity survives insert, remove and reorder |
| `09_composed_dialog.rs` | a modal with an arbitrary body closure |
| `10_nested_overlay.rs` | a popover opened on top of a dialog |
| `11_small_app.rs` | a complete application on shared focus and dispatch |
| `12_author_component.rs` | a downstream component using only `junie_tui::author` |

Examples 02, 03 and 04 (a complete custom theme, a partial theme override and
a global recipe override) are not yet files in `crates/tui/examples/` as of
Slice 4; their code is reproduced and verified in
[`theming.md`](theming.md).

## What does not exist yet

These guides describe the implementation that exists. As of Slice 4 the
component set is:

`Brand`, `Button`, `Checkbox`, `ChipBar`, `Dialog`, `Empty`, `Field`,
`HintBar`, `KeyHint`, `List`, `Meter`, `ProgressBar`, `Props`, `RadioGroup`,
`ScrollRegion`, `Select`, `Spinner`, `StatusBar`, `Tabs`, `TextArea`,
`TextInput`, `Toggle`.

Not yet implemented, and therefore not documented as if they were:
`Picker`, `FilterList`, `CommandPalette`, `Menu`/`ContextMenu`, `Tree`,
`Grid`, `Form`, `Wizard`, `TextViewport`, `CodeEditor`, `Completion`,
`DiffView`, `SplitPane`, `Panel`, `NavList`, `Steps`, `HelpOverlay`,
`TooSmall`. Where a guide would naturally use one of these it says so and
substitutes what exists — for instance `crates/tui/examples/10_nested_overlay.rs`
implements architecture example 10's "nested picker" with a `List` in a
popover, because `Picker` does not exist yet.
