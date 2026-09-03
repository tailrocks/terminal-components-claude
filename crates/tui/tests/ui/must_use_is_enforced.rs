//! §16.1 `response.rs`: `Response` is `#[must_use]` — dropping one silently
//! loses the consumed / repaint answer, which is the whole dispatch contract
//! (§6.1). Compile-fail with `unused_must_use` denied.
#![deny(unused_must_use)]

use tui_next::Response;

fn make() -> Response<()> {
    Response::consumed()
}

fn main() {
    make();
}
