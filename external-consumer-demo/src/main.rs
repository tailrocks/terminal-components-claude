//! The consumer's own binary: a runnable recipe shelf.
//!
//! It proves the crate is a standalone application built on the public
//! Termrock APIs alone — no application crate and no private helper is
//! reachable from here.

use external_consumer_demo::RecipeBox;
use termrock::Theme;

fn main() -> std::io::Result<()> {
    termrock::run(RecipeBox::default(), Theme::junie())
}
