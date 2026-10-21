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
    ) -> ShellSlots<'a> {
        ShellSlots {
            chrome: self.chrome,
            header,
            body,
            footer,
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

/// The shell's named leaves, measured and drawn against one resolved geometry.
pub struct ShellSlots<'a> {
    chrome: Chrome,
    header: &'a dyn CompositionLeaf,
    body: &'a dyn CompositionLeaf,
    footer: &'a dyn CompositionLeaf,
}

impl ShellSlots<'_> {
    fn composition(&self) -> CompositionSlots<'_> {
        CompositionSlots::column([ContentSlot::new(self, ChildPolicy::default().flex(1))])
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
        let min_height = header
            .min
            .1
            .saturating_add(body.min.1)
            .saturating_add(footer.min.1)
            .min(constraints.max.1);
        let body_min_width = body
            .min
            .0
            .saturating_add(self.chrome.inset.saturating_mul(2));
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
        static EMPTY: EmptyLeaf = EmptyLeaf;
        let slots = self.shell.slots(
            self.header.unwrap_or(&EMPTY),
            self.body.unwrap_or(&EMPTY),
            self.footer.unwrap_or(&EMPTY),
        );
        slots.draw(ui)
    }

    /// Obtain the shell's shared resolved-layout leaf.
    pub fn slots(&self) -> ShellSlots<'a> {
        static EMPTY: EmptyLeaf = EmptyLeaf;
        self.shell.slots(
            self.header.unwrap_or(&EMPTY),
            self.body.unwrap_or(&EMPTY),
            self.footer.unwrap_or(&EMPTY),
        )
    }
}
