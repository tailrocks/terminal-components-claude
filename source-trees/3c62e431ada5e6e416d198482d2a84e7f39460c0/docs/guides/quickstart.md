# Quick start

`junie-tui` is a component library for terminal user interfaces, built on
[`ratatui-core`](https://docs.rs/ratatui-core). It gives you buttons, text
inputs, lists, tabs, dialogs, overlays and a semantic theme system, and it
owns the parts of a TUI that are tedious and easy to get wrong: hit testing,
hover, press, focus traversal, modal focus traps, cursor placement, wheel
routing and layer z-order.

---

> ## Current package names
>
> The component library in this workspace is **`junie-tui`** (Rust path
> `junie_tui`, source `crates/tui`). The three applications are separate
> packages under `apps/` and consume its public facade. The old root package
> and legacy source tree have been removed.
>
> The snippets and examples in this guide target the current component library,
> so their imports use `junie_tui`.
>
> The in-tree examples under `crates/tui/examples/` use the same `junie_tui`
> package name.

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
| `02_custom_theme.rs` | a complete custom palette and border set |
| `03_partial_theme.rs` | a partial theme override with safe derivation |
| `04_family_recipe.rs` | a global family recipe override |
| `05_instance_patch.rs` | two buttons, one locally overridden |
| `06_validated_field.rs` | a text field with external validation and an async server error |
| `07_borrowed_rows.rs` | a list over borrowed domain objects with a custom row renderer |
| `08_dynamic_tabs.rs` | tabs whose identity survives insert, remove and reorder |
| `09_composed_dialog.rs` | a modal with an arbitrary body closure |
| `10_nested_overlay.rs` | a popover opened on top of a dialog |
| `11_small_app.rs` | a complete application on shared focus and dispatch |
| `12_author_component.rs` | a downstream component using only `junie_tui::author` |
| `13_connection_form.rs` | a fifteen-field form built from the public `Form` API |

Examples 02, 03 and 04 are runnable theme examples. The form example also
shows how the public `Form` API composes text, choice, toggle and secret
controls without application-side field plumbing.

## Current component surface

The current component library exports these component types from `junie_tui`
(see [`crates/tui/src/lib.rs`](../../crates/tui/src/lib.rs) and
[`components/mod.rs`](../../crates/tui/src/components/mod.rs)):

`Brand`, `Button`, `Checkbox`, `ChipBar`, `Dialog`, `Empty`, `Field`,
`HintBar`, `KeyHint`, `List`, `Meter`, `ProgressBar`, `Props`, `RadioGroup`,
`ScrollRegion`, `Select`, `Spinner`, `StatusBar`, `Tabs`, `TextArea`,
`TextInput`, `Toggle`, `Panel`, `PropsList`, `SplitPane`, `TextViewport`,
`Tree`, `TooSmall`, `NavList`, and `Steps`.

The library also exports `CodeEditor`, `CommandPalette`, `Completion`,
`ContextMenu`, `DiffView`, `FilterList`, `Form`, `Grid`, `HelpOverlay`,
`Menu`, `MenuBar`, `Picker`, `PickerChain`, and `Wizard`, plus the labeled
variants and each component's public state/action/command types where
applicable. `CommandPalette` is the public picker alias.

`DataTable` and `ScrollPanel` remain deleted legacy names: table behavior is
provided by `Grid`, and scrollable text by `TextViewport`. The
`10_nested_overlay.rs` example intentionally keeps its historical
List-in-a-popover fixture; it does not indicate that `Picker` is unavailable.
