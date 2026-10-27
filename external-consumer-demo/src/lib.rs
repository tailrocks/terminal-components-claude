#![deny(unsafe_code)]

pub mod app;
pub mod recipes;
pub mod reference;

pub use app::{
    Action, COOK_BUTTON, MENU_ABOUT, MENU_BAR, MENU_SCALE, NOTE_ITEM, Note, RECIPE_FACTS,
    RECIPE_LIST, RecipeBox, SCALE_BUTTON, STATUS_BAR, WORKSPACE,
};
pub use recipes::{Fact, RECIPES, Recipe};
pub use reference::ReferenceGallery;
