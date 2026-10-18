# Quick start

The workspace facade is the `termrock` package at `crates/termrock`; its Rust path is `termrock`. The API and package boundaries are under active development. Read [architecture](../architecture.md), [design authority](../design.md), and [authoring contract](../contracts/authoring.md) before depending on a particular API surface.

Run the facade checks from the repository root:

```sh
cargo check -p termrock --offline
cargo test -p termrock --features testing --test ex01_button --offline
```

The `testing` feature is required for the headless integration examples. To try the interactive applications, see the launch commands in [development](../development.md).

A minimal app implements an update pass and a draw pass. Update handles interaction and returns a response; draw paints from shared app state and registers UI geometry for the next frame.

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

For deterministic tests or custom event loops, construct `Runtime` directly; the `ex01_button` and `ex02_text_input` integration fixtures show the current headless pattern. Common types are re-exported by the facade; component authors can use `termrock::author` for the registration vocabulary described in the [authoring contract](../contracts/authoring.md).
