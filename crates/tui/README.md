# tui-next

Foundations of the Junie-inspired terminal component library described in
`COMPONENT_ARCHITECTURE.md`: identity, events and intents, the one reply type,
focus ring and scopes, the hit registry, pointer capture, the layer stack, the
theme model (tokens, recipes, role-level patches, six-level precedence), layout
primitives, the text editing core, the collection vocabulary and the runtime
that drives the two-phase frame.

The package is named `tui-next` during Slices 3–4 and becomes `junie-tui` at the
start of Slice 5.

## Quick start

An application implements `App`: `update` receives pre-resolved intents through
`Cx` and is the only place semantics change; `draw` paints through `Ui` and
registers regions. The runtime owns focus, hover, press, capture, layers and the
cursor.

```rust
use tui_next::{id, App, Cx, FrameRead, Focusability, Id, Intent, KeyCode, Response, Runtime, Theme, Ui};

const COUNTER: Id = id!("counter");

#[derive(Default)]
struct Demo { clicks: u32 }

impl App for Demo {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();
        for it in cx.intents(COUNTER) {
            if let Intent::Key(k) = it && k.is(KeyCode::Enter) {
                self.clicks += 1;
                r = Response::changed();
            }
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        ui.register_control(COUNTER, area, Focusability::Focusable);
        let style = ui.style(tui_next::Family::BUTTON, tui_next::Variant::DEFAULT,
                             tui_next::Part::LABEL, ui.state(COUNTER)).style;
        ui.paint_str(area, "press Enter", style);
    }
}

let mut rt = Runtime::new(Demo::default(), Theme::junie());
let area = tui_next::Rect::new(0, 0, 20, 3);
let mut buf = tui_next::Buffer::empty(area);
# #[cfg(feature = "testing")]
rt.draw_buffer(area, &mut buf);
```

With the default `crossterm` feature, `tui_next::run(app, Theme::junie())`
owns the terminal session (raw mode, alternate screen, mouse capture,
bracketed paste, a chained panic hook) and drives the loop.
