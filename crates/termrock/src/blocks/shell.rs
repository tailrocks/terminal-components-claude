//! The app shell block: one header, one body, one footer, laid out through
//! the shared composition machinery or chrome regions.
//!
//! Header and footer strips keep their chrome rows, the body flexes, gap
//! and inset travel through the composition spec. No second layout system;
//! the runtime keeps focus, pointer and modals.
//!
//! `Chrome`, `Regions` and `AppShell` are re-exports of
//! [`termrock_shell`]: the shell crate is the single source of truth for
//! the shell's geometry and slot layout, and this block adds only the
//! facade conveniences ([`EmptyLeaf`] and [`AppShellSlots`]) on top.

use ratatui_core::layout::Rect;
use termrock_runtime::author::CompositionLeaf;
use termrock_runtime::composition::{CompositionError, CompositionSlots};
use termrock_runtime::{Measure, Ui};

pub use termrock_shell::chrome::{Chrome, Regions};
pub use termrock_shell::shell::AppShell;

/// An empty leaf for omitted header, body, or footer slots.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyLeaf;

impl Measure for EmptyLeaf {
    fn measure(
        &self,
        _ui: &Ui<'_>,
        _constraints: termrock_layout::Constraints,
    ) -> termrock_layout::Size {
        termrock_layout::Size {
            min: (0, 0),
            preferred: (0, 0),
        }
    }
}

impl CompositionLeaf for EmptyLeaf {
    fn draw(&self, _ui: &mut Ui<'_>, _logical: Rect) {}
}

/// An app shell builder holding slot leaves.
pub struct AppShellSlots<'a> {
    shell: AppShell,
    header: Option<&'a dyn CompositionLeaf>,
    body: Option<&'a dyn CompositionLeaf>,
    footer: Option<&'a dyn CompositionLeaf>,
}

/// The facade's slot-builder conveniences on [`AppShell`], so apps can
/// attach header/body/footer leaves inline. The shell crate's own
/// `AppShell` API stays the source of truth for geometry.
pub trait AppShellExt: Sized {
    /// Begin configuring slot leaves for this shell.
    fn with_slots<'a>(self) -> AppShellSlots<'a>;

    /// Attach a header slot leaf.
    fn header_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a>;

    /// Attach a body slot leaf.
    fn body_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a>;

    /// Attach a footer slot leaf.
    fn footer_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a>;
}

impl AppShellExt for AppShell {
    fn with_slots<'a>(self) -> AppShellSlots<'a> {
        AppShellSlots {
            shell: self,
            header: None,
            body: None,
            footer: None,
        }
    }

    fn header_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        self.with_slots().header(leaf)
    }

    fn body_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        self.with_slots().body(leaf)
    }

    fn footer_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        self.with_slots().footer(leaf)
    }
}

impl<'a> AppShellSlots<'a> {
    /// Attach the header leaf.
    #[must_use]
    pub const fn header(mut self, leaf: &'a dyn CompositionLeaf) -> Self {
        self.header = Some(leaf);
        self
    }

    /// Attach the body leaf.
    #[must_use]
    pub const fn body(mut self, leaf: &'a dyn CompositionLeaf) -> Self {
        self.body = Some(leaf);
        self
    }

    /// Attach the footer leaf.
    #[must_use]
    pub const fn footer(mut self, leaf: &'a dyn CompositionLeaf) -> Self {
        self.footer = Some(leaf);
        self
    }

    /// Resolve the slots and draw the frame.
    pub fn show(self, ui: &mut Ui<'_>) -> Result<(), CompositionError> {
        let empty = EmptyLeaf;
        let header = self.header.unwrap_or(&empty);
        let body = self.body.unwrap_or(&empty);
        let footer = self.footer.unwrap_or(&empty);
        self.shell.show(ui, header, body, footer)
    }

    /// Obtain the underlying composition slots.
    pub fn slots(&self) -> CompositionSlots<'a> {
        static EMPTY: EmptyLeaf = EmptyLeaf;
        let header = self.header.unwrap_or(&EMPTY);
        let body = self.body.unwrap_or(&EMPTY);
        let footer = self.footer.unwrap_or(&EMPTY);
        self.shell.slots(header, body, footer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termrock_layout::{ChildPolicy, Composition, Insets};
    use termrock_runtime::composition::ContentSlot;

    struct Blank;

    impl Measure for Blank {
        fn measure(&self, _ui: &Ui<'_>, c: termrock_layout::Constraints) -> termrock_layout::Size {
            termrock_layout::Size {
                min: (0, 0),
                preferred: c.max,
            }
        }
    }

    impl CompositionLeaf for Blank {
        fn draw(&self, _ui: &mut Ui<'_>, _logical: Rect) {}
    }

    #[test]
    fn the_facade_shell_is_the_shell_crate_type() {
        // Single source of truth: the block re-exports termrock_shell's
        // AppShell instead of a second, same-named copy.
        fn accepts(_: &termrock_shell::AppShell) {}
        accepts(&AppShell::new());
        assert_eq!(
            AppShell::new(),
            termrock_shell::shell::AppShell::new(),
            "same type, same value semantics"
        );
    }

    #[test]
    fn regions_split_the_frame_once() {
        let area = Rect::new(0, 0, 80, 24);
        let r = Chrome::new().regions(area);
        assert_eq!(r.header, Rect::new(0, 0, 80, 1));
        assert_eq!(r.footer, Rect::new(0, 23, 80, 1));
        assert_eq!(r.body, Rect::new(0, 2, 80, 20));
        let r = Chrome::new().inset(1).regions(area);
        assert_eq!(r.body, Rect::new(1, 2, 78, 20));
        let r = Chrome::new().header(0).gap(0).regions(area);
        assert_eq!(r.body, Rect::new(0, 0, 80, 23));
        assert!(r.header.is_empty());

        let r = Chrome::new().regions(Rect::new(0, 0, 10, 2));
        assert_eq!((r.header.height, r.footer.height, r.body.height), (1, 1, 0));
        let r = Chrome::new().regions(Rect::new(0, 0, 10, 0));
        assert!(r.body.is_empty() && r.footer.is_empty());
    }

    #[test]
    fn rows_reports_the_reserved_strips() {
        assert_eq!(Chrome::new().rows(), (1, 1));
        assert_eq!(Chrome::new().header(3).footer(2).rows(), (3, 2));
    }

    #[test]
    fn the_shell_lays_three_named_slots_out_as_one_column() {
        let shell = AppShell::new().chrome(Chrome::new().header(2).footer(3).gap(1).inset(2));
        let slots = shell.slots(&Blank, &Blank, &Blank);
        let area = Rect::new(0, 0, 40, 24);
        let r = shell.regions(area);
        assert_eq!(r.header, Rect::new(0, 0, 40, 2));
        assert_eq!(r.footer, Rect::new(0, 21, 40, 3));
        assert_eq!(r.body, Rect::new(2, 3, 36, 17));
        let _ = slots;
    }

    #[test]
    fn the_shell_builder_names_its_strips() {
        let shell = AppShell::new().header(2).gap(0).footer(3).inset(1);
        assert_eq!(shell.chrome.header, 2);
        assert_eq!(shell.chrome.gap, 0);
        assert_eq!(shell.chrome.footer, 3);
        assert_eq!(shell.chrome.inset, 1);
    }

    #[test]
    fn a_strip_slot_keeps_its_rows_and_the_body_flexes() {
        let strip = ChildPolicy::new(termrock_layout::Size {
            min: (0, 2),
            preferred: (u16::MAX, 2),
        })
        .min(0, 2);
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
            .padding(Insets::symmetric(chrome.inset, 0));
        assert_eq!(spec.gap, 2);
        assert_eq!(spec.padding, Insets::symmetric(3, 0));
    }

    #[test]
    fn app_shell_slots_builder_shows_with_empty_defaults() {
        let shell = AppShell::new();
        let builder = shell.with_slots().body(&Blank);
        let slots = builder.slots();
        let _ = slots;
        let _ = ContentSlot::new(&Blank, ChildPolicy::default());
    }
}
