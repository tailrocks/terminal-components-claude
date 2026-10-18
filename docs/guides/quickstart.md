# Quick start

## Status

Current workspace guide for the `termrock` facade. The crate split and public APIs remain in active development.

## Authority

The package is `termrock` in `crates/termrock`; its Rust path is `termrock`. See [architecture](../architecture.md), [design reference](../design.md), and [authoring](authoring.md).

## Current evidence

The facade re-exports `App`, `Cx`, `Ui`, `Response`, `Theme`, `Button`, `Id`, `Rect`, and `run`. The API shape below follows `crates/termrock/tests/ex01_button.rs` and `crates/termrock-runtime/src/runtime.rs`.

## Remaining work

The workspace does not currently provide a published-crate installation workflow or a stable API guarantee. Confirm package and API details against the source before using this guide for an external release.

## Verification

`cargo check -p termrock --offline` checks the facade crate. The integration fixture `cargo test -p termrock --test ex01_button --offline` exercises a headless button app.

## Minimal application

Add the workspace package as a path dependency when building inside this checkout:

```toml
[dependencies]
termrock = { path = "crates/termrock" }
```

An app implements an update pass and a draw pass. Update handles interaction and returns a response; draw paints from shared app state and registers UI geometry for the next frame.

```rust
use termrock::{App, Button, Cx, Id, Rect, Response, Theme, Ui, run};

const SAVE: Id = Id::root("save");

#[derive(Default)]
struct Demo;

impl App for Demo {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(SAVE, "Save").update(cx).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(SAVE, "Save").draw(ui, Rect::new(2, 2, 12, 1));
    }
}

fn main() -> std::io::Result<()> {
    run(Demo, Theme::junie())
}
```

Use the provided `App` and `run` path for terminal applications. For deterministic tests or custom event loops, construct `Runtime` directly; the two `ex01_button` and `ex02_text_input` integration fixtures show the current headless pattern.

## Useful API areas

The facade re-exports common types from the split crates. Browse `termrock::controls` for buttons and toggles, `termrock::fields` for text inputs, `termrock::navigation` for lists and tabs, `termrock::overlays` for dialogs and menus, and `termrock::theme` for theme types. Component authors can use `termrock::author` for the lower-level registration vocabulary; its contracts are described in [authoring](authoring.md).
