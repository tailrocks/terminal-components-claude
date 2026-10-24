//! The app shell block: one header, one body, one footer, laid out through
//! the shared composition machinery or chrome regions.
//!
//! Header and footer strips keep their chrome rows, the body flexes, gap
//! and inset travel through the composition spec. No second layout system;
//! the runtime keeps focus, pointer and modals.

use ratatui_core::layout::Rect;
use termrock_layout::{ChildPolicy, Constraints, Size};
use termrock_runtime::Ui;
use termrock_runtime::author::{CompositionLeaf, Measure};
use termrock_runtime::composition::{CompositionError, CompositionSlots, ContentSlot};

use crate::chrome::{Chrome, Regions};

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

    /// Bind header, body, and footer leaves to the shared chrome resolver.
    pub fn slots<'a>(
        &self,
        header: &'a dyn CompositionLeaf,
        body: &'a dyn CompositionLeaf,
        footer: &'a dyn CompositionLeaf,
    ) -> CompositionSlots<'a> {
        CompositionSlots::column([ContentSlot::owned(
            self.resolved_slots(header, body, footer),
            ChildPolicy::default().flex(1),
        )])
    }

    /// Build the canonical leaf that resolves the named shell regions.
    /// Use [`Self::slots`] for the established `CompositionSlots` consumer API.
    pub fn resolved_slots<'a>(
        &self,
        header: &'a dyn CompositionLeaf,
        body: &'a dyn CompositionLeaf,
        footer: &'a dyn CompositionLeaf,
    ) -> ShellSlots<'a> {
        ShellSlots {
            chrome: self.chrome,
            header,
            body,
            footer,
            spec: termrock_layout::Composition::new(),
        }
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
        self.resolved_slots(header, body, footer).draw(ui)
    }

    /// Begin configuring slot leaves for this shell.
    #[must_use]
    pub const fn with_slots<'a>(self) -> AppShellSlots<'a> {
        static EMPTY: EmptyLeaf = EmptyLeaf;
        AppShellSlots {
            slots: ShellSlots {
                chrome: self.chrome,
                header: &EMPTY,
                body: &EMPTY,
                footer: &EMPTY,
                spec: termrock_layout::Composition::new(),
            },
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

/// The shell's named leaves, measured and drawn against one resolved geometry.
#[derive(Clone, Copy)]
pub struct ShellSlots<'a> {
    chrome: Chrome,
    header: &'a dyn CompositionLeaf,
    body: &'a dyn CompositionLeaf,
    footer: &'a dyn CompositionLeaf,
    spec: termrock_layout::Composition,
}

impl ShellSlots<'_> {
    /// Configure the outer composition adapter while keeping shell geometry
    /// inside this block's single chrome resolver.
    /// The spec configures the outer one-child composition, not the spacing
    /// between header, body and footer. Configure internal spacing with
    /// [`Chrome::gap`] before creating the slots.
    #[must_use]
    pub const fn spec(mut self, spec: termrock_layout::Composition) -> Self {
        self.spec = spec;
        self
    }

    /// Adapt this shell leaf to the existing composition API for nesting.
    /// Keep the `ShellSlots` value alive while the returned group is used.
    pub fn composition(&self) -> CompositionSlots<'_> {
        CompositionSlots::column([ContentSlot::new(self, ChildPolicy::default().flex(1))])
            .spec(self.spec)
    }

    /// Resolve and draw the shell through the runtime's composition contract.
    pub fn draw(&self, ui: &mut Ui<'_>) -> Result<(), CompositionError> {
        self.composition().draw(ui)
    }

    /// Measure the shell through the runtime's composition contract.
    pub fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Result<Size, CompositionError> {
        self.composition().measure(ui, constraints)
    }
}

impl Measure for ShellSlots<'_> {
    fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        let area = Rect::new(0, 0, constraints.max.0, constraints.max.1);
        let regions = self.chrome.regions(area);
        let header = self.header.measure(
            ui,
            Constraints::loose(regions.header.width, regions.header.height),
        );
        let body = self.body.measure(
            ui,
            Constraints::loose(regions.body.width, regions.body.height),
        );
        let footer = self.footer.measure(
            ui,
            Constraints::loose(regions.footer.width, regions.footer.height),
        );
        // Chrome's strips are reserved even when their leaves report a
        // smaller intrinsic minimum. Count only gaps that the same resolver
        // will retain beside a nonempty strip, then clamp for narrow roots.
        let header_rows = self.chrome.header.min(constraints.max.1);
        let footer_rows = self
            .chrome
            .footer
            .min(constraints.max.1.saturating_sub(header_rows));
        let header_min = if header_rows > 0 {
            header.min.1.max(header_rows)
        } else {
            0
        };
        let footer_min = if footer_rows > 0 {
            footer.min.1.max(footer_rows)
        } else {
            0
        };
        let strip_and_gap_min = header_min
            .saturating_add(footer_min)
            .saturating_add(if header_rows > 0 { self.chrome.gap } else { 0 })
            .saturating_add(if footer_rows > 0 { self.chrome.gap } else { 0 });
        let min_height = strip_and_gap_min
            .saturating_add(body.min.1)
            .min(constraints.max.1);
        let inset = constraints.max.0.saturating_sub(regions.body.width);
        let body_min_width = body.min.0.saturating_add(inset);
        Size {
            min: (
                header
                    .min
                    .0
                    .max(body_min_width)
                    .max(footer.min.0)
                    .min(constraints.max.0),
                min_height,
            ),
            preferred: constraints.max,
        }
    }
}

impl CompositionLeaf for ShellSlots<'_> {
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect) {
        let regions = self.chrome.regions(logical);
        for (leaf, area) in [
            (self.header, regions.header),
            (self.body, regions.body),
            (self.footer, regions.footer),
        ] {
            if area.is_empty() {
                continue;
            }
            ui.with_area(area, |ui| leaf.draw(ui, area));
        }
    }
}

/// An app shell builder holding slot leaves.
pub struct AppShellSlots<'a> {
    slots: ShellSlots<'a>,
}

impl<'a> AppShellSlots<'a> {
    /// Attach the header leaf.
    #[must_use]
    pub const fn header(mut self, leaf: &'a dyn CompositionLeaf) -> Self {
        self.slots.header = leaf;
        self
    }

    /// Attach the body leaf.
    #[must_use]
    pub const fn body(mut self, leaf: &'a dyn CompositionLeaf) -> Self {
        self.slots.body = leaf;
        self
    }

    /// Attach the footer leaf.
    #[must_use]
    pub const fn footer(mut self, leaf: &'a dyn CompositionLeaf) -> Self {
        self.slots.footer = leaf;
        self
    }

    /// Resolve the slots and draw the frame.
    pub fn show(self, ui: &mut Ui<'_>) -> Result<(), CompositionError> {
        self.slots.draw(ui)
    }

    /// Obtain the shell's shared resolved-layout leaf.
    pub fn slots(&self) -> CompositionSlots<'a> {
        CompositionSlots::column([ContentSlot::owned(
            self.slots,
            ChildPolicy::default().flex(1),
        )])
    }
}
