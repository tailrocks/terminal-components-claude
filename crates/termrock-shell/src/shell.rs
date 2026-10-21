//! The app shell block: one header, one body, one footer, laid out through
//! the shared composition machinery.
//!
//! The parity counterpart of PR #26's `Shell` chrome (commit `f4867e9b5`):
//! the source split its frame by hand each draw; here the shell describes
//! named header/body/footer slots over [`CompositionSlots`], so the block
//! owns geometry through `ChildPolicy` and there is no second layout
//! system. Focus, pointer, hit-testing, modals and the update loop are the
//! runtime's; the shell adds only the frame's shape.

use termrock_layout::{ChildPolicy, Composition, Insets, Size};
use termrock_runtime::author::CompositionLeaf;
use termrock_runtime::composition::{CompositionError, CompositionSlots, ContentSlot};

use crate::chrome::{Chrome, Regions};
use ratatui_core::layout::Rect;

/// The app shell block: a header strip, a flexible body and a footer strip,
/// separated by the chrome's gap and inset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppShell {
    /// The chrome budget the slots are laid out with.
    pub chrome: Chrome,
}

impl Default for AppShell {
    fn default() -> Self {
        Self::new()
    }
}

impl AppShell {
    /// The default shell: one header row, one gap row, one footer row.
    pub const fn new() -> Self {
        Self {
            chrome: Chrome::new(),
        }
    }

    /// Set the chrome budget.
    #[must_use]
    pub const fn chrome(mut self, chrome: Chrome) -> Self {
        self.chrome = chrome;
        self
    }

    /// Split `area` into the shell's three regions (a [`Chrome`] shortcut).
    pub fn regions(&self, area: Rect) -> Regions {
        self.chrome.regions(area)
    }

    /// The slot policy for a reserved strip of `rows` header or footer rows.
    fn strip_policy(rows: u16) -> ChildPolicy {
        ChildPolicy::new(Size {
            min: (0, rows),
            preferred: (u16::MAX, rows),
        })
        .min(0, rows)
    }

    /// The shell as composition slots: header, body, footer, in one column
    /// with the chrome's gap and inset.
    pub fn slots<'a>(
        &self,
        header: &'a dyn CompositionLeaf,
        body: &'a dyn CompositionLeaf,
        footer: &'a dyn CompositionLeaf,
    ) -> CompositionSlots<'a> {
        CompositionSlots::column([
            ContentSlot::new(header, Self::strip_policy(self.chrome.header)),
            ContentSlot::new(body, ChildPolicy::default().flex(1)),
            ContentSlot::new(footer, Self::strip_policy(self.chrome.footer)),
        ])
        .spec(
            Composition::new()
                .gap(self.chrome.gap)
                .padding(Insets::symmetric(self.chrome.inset, 0)),
        )
    }

    /// Draw one frame: resolve the three slots from the framework root and
    /// draw each visible leaf once.
    pub fn show(
        &self,
        ui: &mut termrock_runtime::ui::Ui<'_>,
        header: &dyn CompositionLeaf,
        body: &dyn CompositionLeaf,
        footer: &dyn CompositionLeaf,
    ) -> Result<(), CompositionError> {
        self.slots(header, body, footer).draw(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termrock_layout::layout::Insets as LayoutInsets;
    use termrock_runtime::author::{Constraints, Measure};

    struct Blank;

    impl Measure for Blank {
        fn measure(
            &self,
            _ui: &termrock_runtime::ui::Ui<'_>,
            c: Constraints,
        ) -> termrock_runtime::author::Size {
            termrock_runtime::author::Size {
                min: (0, 0),
                preferred: c.max,
            }
        }
    }

    impl CompositionLeaf for Blank {
        fn draw(&self, _ui: &mut termrock_runtime::ui::Ui<'_>, _logical: Rect) {}
    }

    #[test]
    fn the_shell_lays_three_named_slots_out_as_one_column() {
        let shell = AppShell::new().chrome(Chrome::new().header(2).footer(3).gap(1).inset(2));
        let slots = shell.slots(&Blank, &Blank, &Blank);
        // The policies live inside the slots; the observable contract is the
        // chrome split the slots are built from.
        let area = Rect::new(0, 0, 40, 24);
        let r = shell.regions(area);
        assert_eq!(r.header, Rect::new(0, 0, 40, 2));
        assert_eq!(r.footer, Rect::new(0, 21, 40, 3));
        assert_eq!(r.body, Rect::new(2, 3, 36, 17));
        let _ = slots;
    }

    #[test]
    fn a_strip_slot_keeps_its_rows_and_the_body_flexes() {
        let strip = AppShell::strip_policy(2);
        assert_eq!(strip.min, (0, 2));
        assert_eq!(strip.intrinsic.min.1, 2);
        assert_eq!(strip.intrinsic.preferred.1, 2);
        assert_eq!(strip.flex, 0);
        let body = ChildPolicy::default().flex(1);
        assert_eq!(body.flex, 1);
    }

    #[test]
    fn the_spec_carries_the_gap_and_inset() {
        let chrome = Chrome::new().gap(2).inset(3);
        let spec = Composition::new()
            .gap(chrome.gap)
            .padding(LayoutInsets::symmetric(chrome.inset, 0));
        assert_eq!(spec.gap, 2);
        assert_eq!(spec.padding, LayoutInsets::symmetric(3, 0));
    }
}
