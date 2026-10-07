//! Framework-level primitives that the future component library would
//! expose: ids, focus, hit-testing, text editing, event outcomes.
//! Scrolling already lives in the shared `termrock::ScrollState`.

pub mod event;
pub mod focus;
pub mod hit;
pub mod id;
pub mod text;
