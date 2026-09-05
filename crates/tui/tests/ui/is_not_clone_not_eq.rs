//! §16.1 / §15: `Secret` is deliberately neither `Clone` nor `PartialEq` —
//! a copy is a second place the plaintext lives, and equality is a timing
//! oracle. Raw access is crate-private; the public facade exposes only
//! redacted formatting and non-content metadata.

use junie_tui::Secret;

fn main() {
    let s = Secret::new(String::from("hunter2"));
    let _copy = s.clone();
    let _same = s == Secret::new(String::from("hunter2"));
    let _raw = s.expose();
}
