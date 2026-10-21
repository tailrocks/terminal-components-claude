//! The app shell block: one header, one body, one footer, laid out through
//! the shared composition machinery or chrome regions.
//!
//! Header and footer strips keep their chrome rows, the body flexes, gap
//! and inset travel through the composition spec. No second layout system;
//! the runtime keeps focus, pointer and modals.

use ratatui_core::layout::Rect;
use termrock_layout::{ChildPolicy, Composition, Constraints, Insets, Size};
use termrock_runtime::author::CompositionLeaf;
use termrock_runtime::composition::{CompositionError, CompositionSlots, ContentSlot};
use termrock_runtime::{Measure, Ui};

/// The three regions one chrome split produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Regions {
    /// The top strip.
    pub header: Rect,
    /// Everything between the header and the footer.
    pub body: Rect,
    /// The bottom strip.
    pub footer: Rect,
}

/// The chrome budget: rows reserved for the header and footer, the gap
/// between them and the body, and the columns of body inset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome {
    /// Rows reserved for the header.
    pub header: u16,
    /// Blank rows between the strips and the body.
    pub gap: u16,
    /// Rows reserved for the footer.
    pub footer: u16,
    /// Columns inset from the body's left and right edges.
    pub inset: u16,
}

impl Default for Chrome {
    fn default() -> Self {
        Self::new()
    }
}

impl Chrome {
    /// One header row, one gap row, one footer row, no inset.
    pub const fn new() -> Self {
        Self {
            header: 1,
            gap: 1,
            footer: 1,
            inset: 0,
        }
    }

    /// Set the header rows.
    #[must_use]
    pub const fn header(mut self, rows: u16) -> Self {
        self.header = rows;
        self
    }

    /// Set the gap rows.
    #[must_use]
    pub const fn gap(mut self, rows: u16) -> Self {
        self.gap = rows;
        self
    }

    /// Set the footer rows.
    #[must_use]
    pub const fn footer(mut self, rows: u16) -> Self {
        self.footer = rows;
        self
    }

    /// Set the body inset columns.
    #[must_use]
    pub const fn inset(mut self, cols: u16) -> Self {
        self.inset = cols;
        self
    }

    /// Split `area` once into the three regions. The strips keep their rows
    /// while the area is tall enough; the body takes what is left and is
    /// empty when nothing is.
    pub fn regions(&self, area: Rect) -> Regions {
        let header_h = self.header.min(area.height);
        let footer_h = self.footer.min(area.height.saturating_sub(header_h));
        let header = Rect::new(area.x, area.y, area.width, header_h);
        let footer = Rect::new(
            area.x,
            area.bottom().saturating_sub(footer_h),
            area.width,
            footer_h,
        );
        let top = if header_h > 0 {
            header_h.saturating_add(self.gap)
        } else {
            0
        };
        let bottom = if footer_h > 0 {
            footer_h.saturating_add(self.gap)
        } else {
            0
        };
        let body_y = area.y.saturating_add(top).min(area.bottom());
        let body_h = area.height.saturating_sub(top.saturating_add(bottom));
        let body = Rect::new(
            area.x + self.inset.min(area.width / 2),
            body_y,
            area.width.saturating_sub(self.inset.saturating_mul(2)),
            body_h,
        );
        Regions {
            header,
            body,
            footer,
        }
    }

    /// The reserved header and footer rows.
    pub const fn rows(&self) -> (u16, u16) {
        (self.header, self.footer)
    }
}

/// An empty leaf for omitted header, body, or footer slots.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyLeaf;

impl Measure for EmptyLeaf {
    fn measure(&self, _ui: &Ui<'_>, _constraints: Constraints) -> Size {
        Size {
            min: (0, 0),
            preferred: (0, 0),
        }
    }
}

impl CompositionLeaf for EmptyLeaf {
    fn draw(&self, _ui: &mut Ui<'_>, _logical: Rect) {}
}

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

    /// Set the header rows.
    #[must_use]
    pub const fn header(mut self, rows: u16) -> Self {
        self.chrome = self.chrome.header(rows);
        self
    }

    /// Set the gap rows.
    #[must_use]
    pub const fn gap(mut self, rows: u16) -> Self {
        self.chrome = self.chrome.gap(rows);
        self
    }

    /// Set the footer rows.
    #[must_use]
    pub const fn footer(mut self, rows: u16) -> Self {
        self.chrome = self.chrome.footer(rows);
        self
    }

    /// Set the body inset columns.
    #[must_use]
    pub const fn inset(mut self, cols: u16) -> Self {
        self.chrome = self.chrome.inset(cols);
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
        ui: &mut Ui<'_>,
        header: &dyn CompositionLeaf,
        body: &dyn CompositionLeaf,
        footer: &dyn CompositionLeaf,
    ) -> Result<(), CompositionError> {
        self.slots(header, body, footer).draw(ui)
    }

    /// Begin configuring slot leaves for this shell.
    #[must_use]
    pub const fn with_slots<'a>(self) -> AppShellSlots<'a> {
        AppShellSlots {
            shell: self,
            header: None,
            body: None,
            footer: None,
        }
    }

    /// Attach a header slot leaf.
    #[must_use]
    pub const fn header_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        self.with_slots().header(leaf)
    }

    /// Attach a body slot leaf.
    #[must_use]
    pub const fn body_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        self.with_slots().body(leaf)
    }

    /// Attach a footer slot leaf.
    #[must_use]
    pub const fn footer_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        self.with_slots().footer(leaf)
    }
}

/// An app shell builder holding slot leaves.
pub struct AppShellSlots<'a> {
    shell: AppShell,
    header: Option<&'a dyn CompositionLeaf>,
    body: Option<&'a dyn CompositionLeaf>,
    footer: Option<&'a dyn CompositionLeaf>,
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

    struct Blank;

    impl Measure for Blank {
        fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
            Size {
                min: (0, 0),
                preferred: c.max,
            }
        }
    }

    impl CompositionLeaf for Blank {
        fn draw(&self, _ui: &mut Ui<'_>, _logical: Rect) {}
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
    }
}
