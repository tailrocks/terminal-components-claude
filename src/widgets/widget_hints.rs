//! [`HasHints`] for the focusable widgets whose hints come from their own
//! configuration: the footer a page composes names what the focused
//! widget's keys do now, never a key it does not take (K10, CO3).
//!
//! The modals' hints live with them (`Dialog`, `Picker`, `ContextMenu`,
//! `MenuBar`, `ChoiceDialog`, `FormDialog`, `InfoDialog`, `HelpOverlay`).

use crate::core::hints::{
    Binding, Bindings, HasHints, HintCx, actions_bindings, nav_bindings, nav_hints,
};
use crate::core::keymap::hint_key;
use crate::widgets::button::Button;
use crate::widgets::code::CodeEditor;
use crate::widgets::completion::Completion;
use crate::widgets::grid::{DataGrid, GridCmd};
use crate::widgets::input::TextInput;
use crate::widgets::keyhint::{EDIT_CELL, Hint, hint};
use crate::widgets::list::{ListBox, ListCmd, SelectMode};
use crate::widgets::select::Select;
use crate::widgets::table::{DataTable, TableCmd};
use crate::widgets::tabs::Tabs;
use crate::widgets::textarea::TextArea;
use crate::widgets::tree::TreeView;

/// `Enter Edit`: the hint of a control whose Enter starts an edit. Every
/// app takes it from here (or from the widget's [`HasHints`]), never
/// spells it (K10).
pub const EDIT: Hint = hint(EDIT_PAIR.0, EDIT_PAIR.1);

/// [`EDIT`] as a `(key, action)` pair, for footers that list pairs.
pub const EDIT_PAIR: (&str, &str) = ("Enter", "Edit");

/// A list: how the cursor moves; a multi-select list's toggle and
/// select-all keys.
impl HasHints for ListBox {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        if self.items.is_empty() {
            return vec![];
        }
        let mut v = nav_hints(&self.nav, self.type_ahead, false);
        if self.mode == SelectMode::Multi {
            v.push(hint("Space", "Toggle"));
            if let Some(k) = hint_key(&self.keymap, ListCmd::SelectAll) {
                v.push(Hint::new(k, "Select all"));
            }
        }
        v
    }
}

/// A tree: how the cursor moves and how a branch folds.
impl HasHints for TreeView {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        let mut v = nav_hints(&self.nav, self.type_ahead, false);
        v.push(hint("←→", "Fold"));
        v
    }
}

/// A select: `Enter Open` while closed; open, how the list moves, Enter
/// chooses and Esc closes.
impl HasHints for Select {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        if self.disabled {
            return vec![];
        }
        if !self.open {
            return vec![hint("Enter", "Open")];
        }
        let mut v = nav_hints(&self.nav, self.type_ahead, false);
        v.extend([hint("Enter", "Choose"), hint("Esc", "Close")]);
        v
    }
}

/// An open completion list: its own keys ([`Completion::NAV`]: ↑↓ and
/// paging only, letters keep typing into the editor), Enter accepts, Esc
/// closes (CO3). Closed, none.
impl HasHints for Completion {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        if !self.is_open() {
            return vec![];
        }
        // the editor under it types the letters: never "Type to jump"
        let mut v = nav_hints(&self.nav, false, false);
        v.extend([hint("Enter", "Accept"), hint("Esc", "Close")]);
        v
    }
}

/// A tab strip with focus: ←→ switch tabs.
impl HasHints for Tabs {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        vec![hint("←→", "Switch tab")]
    }
}

/// A table: how the cursor moves, type-ahead, sort and select-all from its
/// key map.
impl HasHints for DataTable {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        if self.is_editing() {
            return crate::widgets::keyhint::EDIT_FIELD.to_vec();
        }
        let mut v = nav_hints(&self.nav, self.type_ahead, false);
        if let Some(k) = hint_key(&self.keymap, TableCmd::SortByColumn) {
            v.push(Hint::new(k, "Sort"));
        }
        v
    }
}

/// A grid: the cell cursor, `Enter Edit` only on a cell Enter edits (K25),
/// the edit keys while a cell edits.
impl HasHints for DataGrid {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        if self.is_editing() {
            return EDIT_CELL.to_vec();
        }
        let mut v = vec![hint("↑↓←→", "Cell")];
        if self.cell_editable() {
            v.push(EDIT);
        }
        if let Some(k) = hint_key(&self.keymap, GridCmd::Copy) {
            v.push(Hint::new(k, "Copy"));
        }
        v
    }
}

impl HasHints for TextInput {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        TextInput::hints(self)
    }
}

impl HasHints for TextArea {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        TextArea::hints(self)
    }
}

impl HasHints for CodeEditor {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        CodeEditor::hints(self)
    }
}

/// A focused button names what Enter does, or that it is working (K35).
impl HasHints for Button {
    fn hints(&self, _: &HintCx) -> Vec<Hint> {
        if self.disabled {
            return vec![];
        }
        vec![self.hint()]
    }
}

// ------------------------------------------------------------ Bindings

/// A list: navigation, activation, and its key map.
impl Bindings for ListBox {
    fn bindings(&self) -> Vec<Binding> {
        let mut v = nav_bindings(&self.nav, self.type_ahead);
        v.push(Binding::group("Enter", "Activate"));
        if self.mode == SelectMode::Multi {
            v.push(Binding::group("Space", "Toggle"));
            v.extend(actions_bindings(&self.keymap));
        }
        v
    }
}

/// A tree: navigation, folding, and its key map.
impl Bindings for TreeView {
    fn bindings(&self) -> Vec<Binding> {
        let mut v = nav_bindings(&self.nav, self.type_ahead);
        v.push(Binding::group("←→", "Fold / unfold"));
        v.push(Binding::group("Enter", "Activate"));
        v.extend(actions_bindings(&self.keymap));
        v
    }
}

/// A tab strip: switching, and its key map (close, new, digits).
impl Bindings for Tabs {
    fn bindings(&self) -> Vec<Binding> {
        let mut v = vec![Binding::group("←→", "Switch tab")];
        v.extend(actions_bindings(&self.keymap));
        v
    }
}

/// A table: navigation and its key map.
impl Bindings for DataTable {
    fn bindings(&self) -> Vec<Binding> {
        let mut v = nav_bindings(&self.nav, self.type_ahead);
        v.push(Binding::group("Enter", "Activate"));
        v.extend(actions_bindings(&self.keymap));
        v
    }
}

/// A grid: the cell cursor, editing, and its key map.
impl Bindings for DataGrid {
    fn bindings(&self) -> Vec<Binding> {
        let mut v = vec![Binding::group("↑↓←→", "Cell")];
        if self.editable {
            v.push(Binding::group("Enter F2", "Edit"));
        }
        v.extend(actions_bindings(&self.keymap));
        v
    }
}

/// A select: open, choose, and the open list's navigation.
impl Bindings for Select {
    fn bindings(&self) -> Vec<Binding> {
        let mut v = vec![Binding::group("Enter Space", "Open")];
        v.extend(nav_bindings(&self.nav, self.type_ahead));
        v.push(Binding::group("Esc", "Close"));
        v
    }
}

#[cfg(test)]
#[path = "widget_hints_tests.rs"]
mod tests;
