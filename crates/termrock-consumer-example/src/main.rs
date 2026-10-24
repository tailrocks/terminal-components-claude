//! The consumer's own binary: a runnable recipe shelf.
//!
//! It proves the crate is a standalone application built on the public
//! Termrock APIs alone — no application crate and no private helper is
//! reachable from here.

use termrock::Theme;
use termrock_consumer_example::RecipeBox;

fn main() -> std::io::Result<()> {
    termrock::run(RecipeBox::default(), Theme::junie())
}
