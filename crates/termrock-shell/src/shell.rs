use ratatui_core::layout::Rect;
use termrock_layout::{ChildPolicy, Constraints, Size};
use termrock_runtime::Ui;
use termrock_runtime::author::{CompositionLeaf, Measure};
use termrock_runtime::composition::{CompositionError, CompositionSlots, ContentSlot};

use crate::chrome::{Chrome, Regions};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppShell {
    pub chrome: Chrome,
}

impl Default for AppShell {
    fn default() -> Self {
        Self::new()
    }
}

impl AppShell {
    pub const fn new() -> Self {
        Self {
            chrome: Chrome::new(),
        }
    }

    #[must_use]
    pub const fn chrome(mut self, chrome: Chrome) -> Self {
        self.chrome = chrome;
        self
    }

    #[must_use]
    pub const fn header(mut self, rows: u16) -> Self {
        self.chrome = self.chrome.header(rows);
        self
    }

    #[must_use]
    pub const fn gap(mut self, rows: u16) -> Self {
        self.chrome = self.chrome.gap(rows);
        self
    }

    #[must_use]
    pub const fn footer(mut self, rows: u16) -> Self {
        self.chrome = self.chrome.footer(rows);
        self
    }

    #[must_use]
    pub const fn inset(mut self, cols: u16) -> Self {
        self.chrome = self.chrome.inset(cols);
        self
    }

    pub fn regions(&self, area: Rect) -> Regions {
        self.chrome.regions(area)
    }

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

    pub fn show(
        &self,
        ui: &mut Ui<'_>,
        header: &dyn CompositionLeaf,
        body: &dyn CompositionLeaf,
        footer: &dyn CompositionLeaf,
    ) -> Result<(), CompositionError> {
        self.resolved_slots(header, body, footer).draw(ui)
    }

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

    #[must_use]
    pub const fn header_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        self.with_slots().header(leaf)
    }

    #[must_use]
    pub const fn body_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        self.with_slots().body(leaf)
    }

    #[must_use]
    pub const fn footer_slot<'a>(self, leaf: &'a dyn CompositionLeaf) -> AppShellSlots<'a> {
        self.with_slots().footer(leaf)
    }
}

#[derive(Clone, Copy)]
pub struct ShellSlots<'a> {
    chrome: Chrome,
    header: &'a dyn CompositionLeaf,
    body: &'a dyn CompositionLeaf,
    footer: &'a dyn CompositionLeaf,
    spec: termrock_layout::Composition,
}

impl ShellSlots<'_> {
    #[must_use]
    pub const fn spec(mut self, spec: termrock_layout::Composition) -> Self {
        self.spec = spec;
        self
    }

    pub fn composition(&self) -> CompositionSlots<'_> {
        CompositionSlots::column([ContentSlot::new(self, ChildPolicy::default().flex(1))])
            .spec(self.spec)
    }

    pub fn draw(&self, ui: &mut Ui<'_>) -> Result<(), CompositionError> {
        self.composition().draw(ui)
    }

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

pub struct AppShellSlots<'a> {
    slots: ShellSlots<'a>,
}

impl<'a> AppShellSlots<'a> {
    #[must_use]
    pub const fn header(mut self, leaf: &'a dyn CompositionLeaf) -> Self {
        self.slots.header = leaf;
        self
    }

    #[must_use]
    pub const fn body(mut self, leaf: &'a dyn CompositionLeaf) -> Self {
        self.slots.body = leaf;
        self
    }

    #[must_use]
    pub const fn footer(mut self, leaf: &'a dyn CompositionLeaf) -> Self {
        self.slots.footer = leaf;
        self
    }

    pub fn show(self, ui: &mut Ui<'_>) -> Result<(), CompositionError> {
        self.slots.draw(ui)
    }

    pub fn slots(&self) -> CompositionSlots<'a> {
        CompositionSlots::column([ContentSlot::owned(
            self.slots,
            ChildPolicy::default().flex(1),
        )])
    }
}
