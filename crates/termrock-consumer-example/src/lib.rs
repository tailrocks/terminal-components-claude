//! An external consumer of the public Termrock component library.
//!
//! This crate stands outside every application in the repository: it depends
//! on the public `termrock` facade only, carries its own domain data (a
//! recipe shelf) and composes a real screen — shell, layout block and
//! interactive widgets — through public APIs.
//!
//! The crate contains no private helpers borrowed from an application, no
//! hand-written geometry and no direct painting: the shell owns the layout,
//! the shared widgets own the interaction, and the consumer only maps typed
//! widget actions onto typed domain actions.

#![deny(unsafe_code)]

pub mod app;
pub mod recipes;

pub use app::{
    Action, COOK_BUTTON, MENU_ABOUT, MENU_BAR, MENU_SCALE, NOTE_ITEM, Note, RECIPE_FACTS,
    RECIPE_LIST, RecipeBox, SCALE_BUTTON, STATUS_BAR, WORKSPACE,
};
pub use recipes::{Fact, RECIPES, Recipe};
