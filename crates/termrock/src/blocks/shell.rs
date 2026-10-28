pub use termrock_shell::{AppShell, AppShellSlots, Chrome, EmptyLeaf, Regions, ShellSlots};

use termrock_runtime::author::CompositionLeaf;

pub trait AppShellExt: Sized {
    fn with_slots<'a>(self) -> AppShellSlots<'a>;

    fn header_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a>;

    fn body_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a>;

    fn footer_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a>;
}

impl AppShellExt for AppShell {
    fn with_slots<'a>(self) -> AppShellSlots<'a> {
        AppShell::with_slots(self)
    }

    fn header_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        AppShell::header_slot(self, leaf)
    }

    fn body_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        AppShell::body_slot(self, leaf)
    }

    fn footer_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        AppShell::footer_slot(self, leaf)
    }
}
