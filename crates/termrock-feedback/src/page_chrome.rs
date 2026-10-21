//! The chrome a page draws around its content: the page frame, the section
//! strip and the card surface.
//!
//! These blocks own the geometry and the painting. The page owns the content
//! and the policy: the ink it paints with, the tabs it offers and the
//! breakpoint at which the section caption appears. A page never measures a
//! width to decide what the chrome looks like; it hands the threshold to the
//! block and the block decides.

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::id::Part;
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{Family, PaintStyle, Variant};
use crate::ui::Ui;

/// The colours the chrome paints with.
///
/// The page owns the palette. The block only knows which role a colour plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChromeInk {
    /// The page background behind the chrome.
    pub canvas: Color,
    /// The card surface.
    pub surface: Color,
    /// The primary ink: the page title and the active tab label.
    pub ink: Color,
    /// Secondary ink: inactive tabs and card titles.
    pub secondary: Color,
    /// Muted ink: descriptive text.
    pub muted: Color,
    /// Faint ink: metadata and captions.
    pub faint: Color,
    /// The colour of the hairline rules.
    pub rule: Color,
    /// The accent: the active tab underline.
    pub accent: Color,
    /// The background of the active tab.
    pub tab_active: Color,
}

impl ChromeInk {
    /// A palette with every role set to the terminal default.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            canvas: Color::Reset,
            surface: Color::Reset,
            ink: Color::Reset,
            secondary: Color::Reset,
            muted: Color::Reset,
            faint: Color::Reset,
            rule: Color::Reset,
            accent: Color::Reset,
            tab_active: Color::Reset,
        }
    }

    /// Packs a `0x00rrggbb` literal into a colour.
    #[must_use]
    pub const fn rgb(hex: u32) -> Color {
        Color::Rgb(
            ((hex >> 16) & 0xff) as u8,
            ((hex >> 8) & 0xff) as u8,
            (hex & 0xff) as u8,
        )
    }

    /// The pair style the chrome paints with: a foreground over a background.
    #[must_use]
    pub fn pair(&self, fg: Color, bg: Color) -> PaintStyle {
        let mut style = PaintStyle::new();
        style = style.fg(fg);
        style.bg(bg)
    }
}

impl Default for ChromeInk {
    fn default() -> Self {
        Self::new()
    }
}

/// What the page frame paints next to the title.
#[derive(Clone, Copy, Debug)]
enum Slot<'a> {
    /// Theme-driven metadata, painted with the `LIST`/`META` part style.
    Meta(&'a str),
    /// A muted description, painted with an explicit style.
    Blurb { text: &'a str, style: PaintStyle },
}

/// The header of a page: the title, the slot next to it and the body
/// rectangle that starts two rows below the header.
///
/// The frame paints the title with the `PANEL`/`TITLE` part style unless the
/// page overrides it, fills the page with the surface style unless the page
/// overrides it, and truncates the slot with an ellipsis when it does not
/// fit.
#[derive(Clone, Debug)]
pub struct PageFrame<'a> {
    title: &'a str,
    title_style: Option<PaintStyle>,
    slot: Option<Slot<'a>>,
    surface: Option<PaintStyle>,
}

impl<'a> PageFrame<'a> {
    /// A frame for `title` with no metadata.
    #[must_use]
    pub const fn new(title: &'a str) -> Self {
        Self {
            title,
            title_style: None,
            slot: None,
            surface: None,
        }
    }

    /// Overrides the title style.
    #[must_use]
    pub fn title_style(mut self, style: impl Into<PaintStyle>) -> Self {
        self.title_style = Some(style.into());
        self
    }

    /// Overrides the fill the frame paints behind the page.
    #[must_use]
    pub fn surface(mut self, style: impl Into<PaintStyle>) -> Self {
        self.surface = Some(style.into());
        self
    }

    /// Paints `text` next to the title with the `LIST`/`META` part style.
    #[must_use]
    pub const fn meta(mut self, text: &'a str) -> Self {
        self.slot = Some(Slot::Meta(text));
        self
    }

    /// Paints `text` next to the title with an explicit style. The muted
    /// description of a page, the kind a page draws for itself.
    #[must_use]
    pub fn blurb(mut self, text: &'a str, style: impl Into<PaintStyle>) -> Self {
        self.slot = Some(Slot::Blurb {
            text,
            style: style.into(),
        });
        self
    }

    /// Paints the frame and returns the body rectangle below the header.
    #[must_use]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return Rect::ZERO;
        }
        let surface = match self.surface {
            Some(style) => style,
            None => ui.surface_style(),
        };
        ui.fill(area, surface);
        let title_style = match self.title_style {
            Some(style) => style,
            None => {
                ui.style(
                    Family::PANEL,
                    Variant::DEFAULT,
                    Part::TITLE,
                    StateFlags::empty(),
                )
                .style
            }
        };
        let title_row = Rect { height: 1, ..area };
        let title_width = width(self.title).min(area.width);
        let _ = ui.paint_str(title_row, self.title, title_style);
        match self.slot {
            Some(Slot::Meta(text)) => {
                let meta_style = ui
                    .style(
                        Family::LIST,
                        Variant::DEFAULT,
                        Part::META,
                        StateFlags::empty(),
                    )
                    .style;
                self.paint_slot(ui, area, title_width, text, meta_style);
            }
            Some(Slot::Blurb { text, style }) => {
                self.paint_slot(ui, area, title_width, text, style);
            }
            None => {}
        }
        Rect {
            y: area.y.saturating_add(2),
            height: area.height.saturating_sub(2),
            ..area
        }
    }

    /// Paints the frame and hands the body rectangle to `body`.
    pub fn draw_with(&self, ui: &mut Ui<'_>, area: Rect, body: impl FnOnce(&mut Ui<'_>, Rect)) {
        if area.is_empty() {
            return;
        }
        let body_area = self.draw(ui, area);
        body(ui, body_area);
    }

    fn paint_slot(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        title_width: u16,
        text: &str,
        style: PaintStyle,
    ) {
        if text.is_empty() || area.width <= title_width.saturating_add(4) {
            return;
        }
        let slot = Rect {
            x: area.x.saturating_add(title_width).saturating_add(2),
            width: area.width.saturating_sub(title_width).saturating_sub(3),
            height: 1,
            ..area
        };
        if slot.is_empty() {
            return;
        }
        if width(text) <= slot.width {
            let _ = ui.paint_str(slot, text, style);
            return;
        }
        let budget = Rect {
            width: slot.width.saturating_sub(1),
            ..slot
        };
        let used = ui.paint_str(budget, text, style);
        let _ = ui.paint_str(
            Rect::new(slot.x.saturating_add(used), slot.y, 1, 1),
            "…",
            style,
        );
    }
}

/// The strip of section tabs a page draws under its header.
///
/// The strip owns the caption breakpoint: the caption appears when the strip
/// is at least `caption_min_width` columns wide. The page hands the threshold
/// over instead of measuring the width itself.
#[derive(Clone, Debug)]
pub struct SectionStrip<'a> {
    tabs: &'a [&'a str],
    active: usize,
    caption: Option<&'a str>,
    caption_min_width: u16,
    ink: ChromeInk,
}

impl<'a> SectionStrip<'a> {
    /// A strip over `tabs` with the first tab active.
    #[must_use]
    pub const fn new(tabs: &'a [&'a str]) -> Self {
        Self {
            tabs,
            active: 0,
            caption: None,
            caption_min_width: 0,
            ink: ChromeInk::new(),
        }
    }

    /// The tab that is active.
    #[must_use]
    pub const fn active(mut self, index: usize) -> Self {
        self.active = index;
        self
    }

    /// The caption the strip paints on the right.
    #[must_use]
    pub const fn caption(mut self, caption: &'a str) -> Self {
        self.caption = Some(caption);
        self
    }

    /// The width the strip needs before it paints the caption.
    #[must_use]
    pub const fn caption_min_width(mut self, columns: u16) -> Self {
        self.caption_min_width = columns;
        self
    }

    /// The palette the strip paints with.
    #[must_use]
    pub const fn ink(mut self, ink: ChromeInk) -> Self {
        self.ink = ink;
        self
    }

    /// Paints the strip: the tabs, the caption and the rule below them.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let mut x = area.x;
        let mut tab_end = area.x;
        for (index, label) in self.tabs.iter().enumerate() {
            let tab_width = width(label).saturating_add(3);
            if x.saturating_add(tab_width) > area.right() {
                break;
            }
            let style = if index == self.active {
                self.ink
                    .pair(self.ink.ink, self.ink.tab_active)
                    .add_modifier(Modifier::BOLD)
            } else {
                self.ink.pair(self.ink.secondary, self.ink.canvas)
            };
            ui.fill(Rect::new(x, area.y, tab_width, 1), style);
            let _ = ui.paint_str(
                Rect::new(x.saturating_add(1), area.y, tab_width.saturating_sub(1), 1),
                label,
                style,
            );
            tab_end = x.saturating_add(tab_width);
            x = tab_end.saturating_add(1);
        }
        if let Some(caption) = self
            .caption
            .filter(|_| area.width >= self.caption_min_width)
        {
            let caption_width = width(caption);
            let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
            if caption_x > tab_end.saturating_add(2) {
                let _ = ui.paint_str(
                    Rect::new(caption_x, area.y, caption_width, 1),
                    caption,
                    self.ink.pair(self.ink.faint, self.ink.canvas),
                );
            }
        }
        if area.height < 2 {
            return;
        }
        let rule_y = area.y.saturating_add(1);
        let _ = ui.paint_str(
            Rect {
                y: rule_y,
                height: 1,
                ..area
            },
            &"─".repeat(usize::from(area.width)),
            self.ink.pair(self.ink.rule, self.ink.canvas),
        );
        let active = self
            .tabs
            .get(self.active)
            .map(|label| width(label).saturating_add(3))
            .unwrap_or(0)
            .min(area.width);
        let _ = ui.paint_str(
            Rect::new(area.x, rule_y, active, 1),
            &"━".repeat(usize::from(active)),
            self.ink.pair(self.ink.accent, self.ink.canvas),
        );
    }
}

/// Where the metadata of a card sits next to its title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardMeta {
    /// The metadata is right-aligned in the card's text column.
    Right,
    /// The metadata follows the title and only falls back to the right edge
    /// when the title leaves it no room.
    Flow,
}

/// Plain lines of text painted with the surface style.
#[derive(Clone, Copy, Debug)]
pub struct LineBlock<'a> {
    lines: &'a [&'a str],
}

impl<'a> LineBlock<'a> {
    /// A block of `lines`.
    #[must_use]
    pub const fn new(lines: &'a [&'a str]) -> Self {
        Self { lines }
    }

    /// Paints the lines, one per row, clipped to `area`.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let style = ui.surface_style();
        for (offset, line) in self.lines.iter().enumerate() {
            let Ok(offset) = u16::try_from(offset) else {
                break;
            };
            if offset >= area.height {
                break;
            }
            let row = Rect {
                y: area.y.saturating_add(offset),
                height: 1,
                ..area
            };
            let _ = ui.paint_str(row, line, style);
        }
    }
}

/// The chrome of a card: the filled surface, the title, the metadata and the
/// hairline rule under the title row.
#[derive(Clone, Debug)]
pub struct CardChrome<'a> {
    title: &'a str,
    meta: &'a str,
    ink: ChromeInk,
    title_style: Option<PaintStyle>,
    surface_style: Option<PaintStyle>,
    meta_layout: CardMeta,
    left_inset: u16,
    right_inset: u16,
}

impl<'a> CardChrome<'a> {
    /// The chrome of a card with a title and no metadata.
    #[must_use]
    pub const fn new(title: &'a str) -> Self {
        Self {
            title,
            meta: "",
            ink: ChromeInk::new(),
            title_style: None,
            surface_style: None,
            meta_layout: CardMeta::Right,
            left_inset: 2,
            right_inset: 2,
        }
    }

    /// The metadata the card paints on the title row.
    #[must_use]
    pub const fn meta(mut self, meta: &'a str) -> Self {
        self.meta = meta;
        self
    }

    /// The palette the card paints with.
    #[must_use]
    pub const fn ink(mut self, ink: ChromeInk) -> Self {
        self.ink = ink;
        self
    }

    /// Overrides the title style.
    #[must_use]
    pub fn title_style(mut self, style: impl Into<PaintStyle>) -> Self {
        self.title_style = Some(style.into());
        self
    }

    /// Overrides the fill the card paints behind its chrome.
    #[must_use]
    pub fn surface_style(mut self, style: impl Into<PaintStyle>) -> Self {
        self.surface_style = Some(style.into());
        self
    }

    /// Where the metadata sits.
    #[must_use]
    pub const fn meta_layout(mut self, layout: CardMeta) -> Self {
        self.meta_layout = layout;
        self
    }

    /// The columns the card keeps free on the left of its text column.
    #[must_use]
    pub const fn left_inset(mut self, columns: u16) -> Self {
        self.left_inset = columns;
        self
    }

    /// The columns the card keeps free on the right of its text column.
    #[must_use]
    pub const fn right_inset(mut self, columns: u16) -> Self {
        self.right_inset = columns;
        self
    }

    /// Paints the card chrome and returns the text column of the card.
    pub fn draw(&self, ui: &mut Ui<'_>, card: Rect) -> Rect {
        let surface = match self.surface_style {
            Some(style) => style,
            None => self.ink.pair(self.ink.ink, self.ink.surface),
        };
        ui.fill(card, surface);
        if card.height == 0 || card.width < 4 {
            return Rect::ZERO;
        }
        let text_x = card.x.saturating_add(self.left_inset);
        let text_w = card
            .width
            .saturating_sub(self.left_inset)
            .saturating_sub(self.right_inset);
        let title_style = match self.title_style {
            Some(style) => style,
            None => self.ink.pair(self.ink.secondary, self.ink.surface),
        };
        let _ = ui.paint_str(
            Rect::new(text_x, card.y, text_w, 1),
            self.title,
            title_style,
        );
        if !self.meta.is_empty() {
            let meta_style = self.ink.pair(self.ink.faint, self.ink.surface);
            match self.meta_layout {
                CardMeta::Right => {
                    let meta_w = width(self.meta);
                    if meta_w < text_w {
                        let _ = ui.paint_str(
                            Rect::new(
                                text_x.saturating_add(text_w.saturating_sub(meta_w)),
                                card.y,
                                meta_w,
                                1,
                            ),
                            self.meta,
                            meta_style,
                        );
                    }
                }
                CardMeta::Flow => {
                    if text_w > 0 {
                        let meta_w = width(self.meta);
                        let title_w = width(self.title);
                        let min_x = text_x.saturating_add(title_w.saturating_mul(2));
                        let fitted = text_w >= meta_w;
                        let right_x = text_x.saturating_add(text_w.saturating_sub(meta_w));
                        if fitted && right_x >= min_x {
                            let _ = ui.paint_str(
                                Rect::new(right_x, card.y, meta_w, 1),
                                self.meta,
                                meta_style,
                            );
                        } else if min_x < text_x.saturating_add(text_w) {
                            let room = text_x.saturating_add(text_w).saturating_sub(min_x);
                            let _ = ui.paint_str(
                                Rect::new(min_x, card.y, room, 1),
                                self.meta,
                                meta_style,
                            );
                        }
                    }
                }
            }
        }
        if card.height > 1 && text_w > 0 {
            let _ = ui.paint_str(
                Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
                &"─".repeat(usize::from(text_w)),
                self.ink.pair(self.ink.rule, self.ink.surface),
            );
        }
        Rect {
            x: text_x,
            y: card.y,
            width: text_w,
            height: card.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::Color;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::{SCREEN, Stub};
    use crate::theme::Theme;

    const CANVAS: u32 = 0x000000;
    const SURFACE: u32 = 0x111111;
    const WHITE: u32 = 0xffffff;
    const SECONDARY: u32 = 0xb3b3b3;
    const MUTED: u32 = 0x8a8a8a;
    const FAINT: u32 = 0x737373;
    const RULE: u32 = 0x3a3a3a;
    const ACCENT: u32 = 0x48e054;
    const TAB_ACTIVE: u32 = 0x27272a;

    const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
    const SECTION_CAPTION: &str = "the live widget";
    const SECTION_WIDE: u16 = 76;

    fn rgb(hex: u32) -> Color {
        Color::Rgb(
            ((hex >> 16) & 0xff) as u8,
            ((hex >> 8) & 0xff) as u8,
            (hex & 0xff) as u8,
        )
    }

    fn ink(fg: u32, bg: u32) -> PaintStyle {
        PaintStyle::new().fg(rgb(fg)).bg(rgb(bg))
    }

    fn palette() -> ChromeInk {
        ChromeInk {
            canvas: rgb(CANVAS),
            surface: rgb(SURFACE),
            ink: rgb(WHITE),
            secondary: rgb(SECONDARY),
            muted: rgb(MUTED),
            faint: rgb(FAINT),
            rule: rgb(RULE),
            accent: rgb(ACCENT),
            tab_active: rgb(TAB_ACTIVE),
        }
    }

    // ---------------------------------------------------------------------
    // The page-owned path the blocks replace. Copied verbatim from the
    // showcase pages so the comparison proves the blocks paint the same
    // cells the pages painted before.
    // ---------------------------------------------------------------------

    fn legacy_frame(
        ui: &mut Ui<'_>,
        area: Rect,
        title: &'static str,
        meta: &'static str,
        body: impl FnOnce(&mut Ui<'_>, Rect),
    ) {
        if area.is_empty() {
            return;
        }
        ui.fill(area, ui.surface_style());
        let title_style = ui
            .style(
                Family::PANEL,
                Variant::DEFAULT,
                Part::TITLE,
                StateFlags::empty(),
            )
            .style;
        let meta_style = ui
            .style(
                Family::LIST,
                Variant::DEFAULT,
                Part::META,
                StateFlags::empty(),
            )
            .style;
        let title_area = Rect { height: 1, ..area };
        let title_width = width(title).min(area.width);
        ui.paint_str(title_area, title, title_style);
        if !meta.is_empty() && area.width > title_width.saturating_add(4) {
            let meta_area = Rect {
                x: area.x.saturating_add(title_width).saturating_add(2),
                width: area.width.saturating_sub(title_width).saturating_sub(3),
                height: 1,
                ..area
            };
            legacy_paint_clipped_meta(ui, meta_area, meta, meta_style);
        }
        let body_area = Rect {
            y: area.y.saturating_add(2),
            height: area.height.saturating_sub(2),
            ..area
        };
        body(ui, body_area);
    }

    fn legacy_paint_clipped_meta(
        ui: &mut Ui<'_>,
        meta_area: Rect,
        meta: &str,
        meta_style: PaintStyle,
    ) {
        if meta_area.is_empty() {
            return;
        }
        if width(meta) <= meta_area.width {
            ui.paint_str(meta_area, meta, meta_style);
        } else {
            let budget = Rect {
                width: meta_area.width.saturating_sub(1),
                ..meta_area
            };
            let used = ui.paint_str(budget, meta, meta_style);
            ui.paint_str(
                Rect::new(meta_area.x.saturating_add(used), meta_area.y, 1, 1),
                "…",
                meta_style,
            );
        }
    }

    fn legacy_paint_blurb(ui: &mut Ui<'_>, body: Rect, title: &str, blurb: &str) {
        let title_w = width(title);
        if body.width <= title_w.saturating_add(4) {
            return;
        }
        let meta = Rect::new(
            body.x.saturating_add(title_w).saturating_add(2),
            body.y.saturating_sub(2),
            body.width.saturating_sub(title_w).saturating_sub(3),
            1,
        );
        let style = ink(MUTED, CANVAS);
        if width(blurb) <= meta.width {
            ui.paint_str(meta, blurb, style);
            return;
        }
        if meta.width == 0 {
            return;
        }
        let budget = Rect {
            width: meta.width.saturating_sub(1),
            ..meta
        };
        let used = ui.paint_str(budget, blurb, style);
        ui.paint_str(
            Rect::new(meta.x.saturating_add(used), meta.y, 1, 1),
            "…",
            style,
        );
    }

    fn legacy_tab_width(label: &str) -> u16 {
        width(label).saturating_add(3)
    }

    fn legacy_paint_section_strip(ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let mut x = area.x;
        let mut tab_end = area.x;
        for (index, label) in SECTIONS.iter().enumerate() {
            let w = legacy_tab_width(label);
            if x.saturating_add(w) > area.right() {
                break;
            }
            let style = if index == 0 {
                ink(WHITE, TAB_ACTIVE).add_modifier(Modifier::BOLD)
            } else {
                ink(SECONDARY, CANVAS)
            };
            ui.fill(Rect::new(x, area.y, w, 1), style);
            ui.paint_str(
                Rect::new(x.saturating_add(1), area.y, w.saturating_sub(1), 1),
                label,
                style,
            );
            tab_end = x.saturating_add(w);
            x = tab_end.saturating_add(1);
        }
        if area.width >= SECTION_WIDE {
            let caption_width = width(SECTION_CAPTION);
            let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
            if caption_x > tab_end.saturating_add(2) {
                ui.paint_str(
                    Rect::new(caption_x, area.y, caption_width, 1),
                    SECTION_CAPTION,
                    ink(FAINT, CANVAS),
                );
            }
        }
        if area.height < 2 {
            return;
        }
        let rule_y = area.y.saturating_add(1);
        ui.paint_str(
            Rect {
                y: rule_y,
                height: 1,
                ..area
            },
            &"─".repeat(usize::from(area.width)),
            ink(RULE, CANVAS),
        );
        let active = legacy_tab_width(SECTIONS[0]).min(area.width);
        ui.paint_str(
            Rect::new(area.x, rule_y, active, 1),
            &"━".repeat(usize::from(active)),
            ink(ACCENT, CANVAS),
        );
    }

    fn legacy_paint_card_chrome(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
        ui.fill(card, ink(WHITE, SURFACE));
        if card.height == 0 || card.width < 4 {
            return;
        }
        let text_x = card.x.saturating_add(2);
        let text_w = card.width.saturating_sub(4);
        ui.paint_str(
            Rect::new(text_x, card.y, text_w, 1),
            title,
            ink(SECONDARY, SURFACE),
        );
        if !meta.is_empty() {
            let meta_w = width(meta);
            if meta_w < text_w {
                ui.paint_str(
                    Rect::new(
                        text_x.saturating_add(text_w.saturating_sub(meta_w)),
                        card.y,
                        meta_w,
                        1,
                    ),
                    meta,
                    ink(FAINT, SURFACE),
                );
            }
        }
        if card.height > 1 && text_w > 0 {
            ui.paint_str(
                Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
                &"─".repeat(usize::from(text_w)),
                ink(RULE, SURFACE),
            );
        }
    }

    fn legacy_paint_card_chrome_flow(
        ui: &mut Ui<'_>,
        card: Rect,
        title: &str,
        meta: &str,
        right_inset: u16,
    ) {
        ui.fill(card, ink(WHITE, SURFACE));
        if card.height == 0 || card.width < 4 {
            return;
        }
        let text_x = card.x.saturating_add(2);
        let text_w = card.width.saturating_sub(2).saturating_sub(right_inset);
        ui.paint_str(
            Rect::new(text_x, card.y, text_w, 1),
            title,
            ink(SECONDARY, SURFACE),
        );
        if !meta.is_empty() && text_w > 0 {
            let meta_w = width(meta);
            let title_w = width(title);
            let min_x = text_x.saturating_add(title_w.saturating_mul(2));
            let fitted = text_w >= meta_w;
            let right_x = text_x.saturating_add(text_w.saturating_sub(meta_w));
            if fitted && right_x >= min_x {
                ui.paint_str(
                    Rect::new(right_x, card.y, meta_w, 1),
                    meta,
                    ink(FAINT, SURFACE),
                );
            } else if min_x < text_x.saturating_add(text_w) {
                let room = text_x.saturating_add(text_w).saturating_sub(min_x);
                ui.paint_str(Rect::new(min_x, card.y, room, 1), meta, ink(FAINT, SURFACE));
            }
        }
        if card.height > 1 && text_w > 0 {
            ui.paint_str(
                Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
                &"─".repeat(usize::from(text_w)),
                ink(RULE, SURFACE),
            );
        }
    }

    fn legacy_paint_card_chrome_clearing(ui: &mut Ui<'_>, card: Rect, title: &str, meta: &str) {
        ui.fill(card, ink(WHITE, SURFACE).remove_modifier(Modifier::all()));
        if card.height == 0 || card.width < 4 {
            return;
        }
        let text_x = card.x.saturating_add(2);
        let text_w = card.width.saturating_sub(4);
        ui.paint_str(
            Rect::new(text_x, card.y, text_w, 1),
            title,
            ink(SECONDARY, SURFACE),
        );
        if !meta.is_empty() {
            let meta_w = width(meta);
            if meta_w < text_w {
                ui.paint_str(
                    Rect::new(
                        text_x.saturating_add(text_w.saturating_sub(meta_w)),
                        card.y,
                        meta_w,
                        1,
                    ),
                    meta,
                    ink(FAINT, SURFACE),
                );
            }
        }
        if card.height > 1 && text_w > 0 {
            ui.paint_str(
                Rect::new(text_x, card.y.saturating_add(1), text_w, 1),
                &"─".repeat(usize::from(text_w)),
                ink(RULE, SURFACE),
            );
        }
    }

    // ---------------------------------------------------------------------

    fn render(area: Rect, draw: impl FnOnce(&mut Ui<'_>, Rect)) -> Buffer {
        let mut rt = Runtime::new(Stub::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_scene(area, &mut buf, |ui, scene| draw(ui, scene))
            .commit_presented();
        buf
    }

    fn assert_same(legacy: &Buffer, block: &Buffer, what: &str) {
        for y in 0..SCREEN.height {
            for x in 0..SCREEN.width {
                let pos = ratatui_core::layout::Position::new(x, y);
                let a = legacy.cell(pos).expect("legacy cell");
                let b = block.cell(pos).expect("block cell");
                assert_eq!(a.symbol(), b.symbol(), "{what}: symbol at ({x},{y})");
                assert_eq!(a.fg, b.fg, "{what}: fg at ({x},{y})");
                assert_eq!(a.bg, b.bg, "{what}: bg at ({x},{y})");
                assert_eq!(a.modifier, b.modifier, "{what}: modifier at ({x},{y})");
            }
        }
    }

    #[test]
    fn page_frame_matches_the_page_owned_frame() {
        for (columns, rows) in [(120u16, 40u16), (94, 36), (51, 6), (20, 4)] {
            let area = Rect::new(0, 0, columns, rows);
            let legacy = render(area, |ui, scene| {
                legacy_frame(ui, scene, "Lists", "Single selection", |ui, body| {
                    let style = ui.surface_style();
                    ui.fill(body, style);
                });
            });
            let block = render(area, |ui, scene| {
                PageFrame::new("Lists").meta("Single selection").draw_with(
                    ui,
                    scene,
                    |ui, body| {
                        ui.fill(body, ui.surface_style());
                    },
                );
            });
            assert_same(&legacy, &block, "page frame");
        }
    }

    #[test]
    fn page_frame_blurb_matches_the_page_owned_blurb() {
        let blurb = "Single and multiple selection, disabled items, scrolling, empty state";
        for (columns, rows) in [(120u16, 40u16), (94, 36), (51, 6), (20, 4)] {
            let area = Rect::new(0, 0, columns, rows);
            let legacy = render(area, |ui, scene| {
                legacy_frame(ui, scene, "Lists", "", |ui, body| {
                    legacy_paint_blurb(ui, body, "Lists", blurb);
                });
            });
            let block = render(area, |ui, scene| {
                PageFrame::new("Lists")
                    .blurb(blurb, ink(MUTED, CANVAS))
                    .draw_with(ui, scene, |_ui, _body| {});
            });
            assert_same(&legacy, &block, "page frame blurb");
        }
    }

    #[test]
    fn section_strip_matches_the_page_owned_strip() {
        for (columns, rows) in [(120u16, 40u16), (94, 36), (76, 6), (40, 6), (10, 6)] {
            let area = Rect::new(0, 0, columns, rows);
            let legacy = render(area, |ui, scene| {
                legacy_paint_section_strip(
                    ui,
                    Rect {
                        height: 2.min(scene.height),
                        ..scene
                    },
                );
            });
            let block = render(area, |ui, scene| {
                SectionStrip::new(&SECTIONS)
                    .caption(SECTION_CAPTION)
                    .caption_min_width(SECTION_WIDE)
                    .ink(palette())
                    .draw(
                        ui,
                        Rect {
                            height: 2.min(scene.height),
                            ..scene
                        },
                    );
            });
            assert_same(&legacy, &block, "section strip");
        }
    }

    #[test]
    fn card_chrome_matches_the_page_owned_card_chrome() {
        for (columns, rows) in [(120u16, 40u16), (40, 8), (4, 3), (3, 3)] {
            let area = Rect::new(0, 0, columns, rows);
            let legacy = render(area, |ui, scene| {
                legacy_paint_card_chrome(ui, scene, "Playground", "the live widget");
            });
            let block = render(area, |ui, scene| {
                let _ = CardChrome::new("Playground")
                    .meta("the live widget")
                    .ink(palette())
                    .draw(ui, scene);
            });
            assert_same(&legacy, &block, "card chrome");
        }
    }
    #[test]
    fn card_chrome_flow_matches_the_page_owned_grid_card() {
        for (columns, rows, inset) in [(120u16, 40u16, 0u16), (120, 40, 2), (40, 8, 2)] {
            let area = Rect::new(0, 0, columns, rows);
            let legacy = render(area, |ui, scene| {
                legacy_paint_card_chrome_flow(ui, scene, "customers", "12 rows", inset);
            });
            let block = render(area, |ui, scene| {
                let _ = CardChrome::new("customers")
                    .meta("12 rows")
                    .ink(palette())
                    .right_inset(inset)
                    .meta_layout(CardMeta::Flow)
                    .draw(ui, scene);
            });
            assert_same(&legacy, &block, "card chrome flow");
        }
    }

    #[test]
    fn card_chrome_clearing_surface_matches_the_page_owned_picker_card() {
        for (columns, rows) in [(120u16, 40u16), (40, 8)] {
            let area = Rect::new(0, 0, columns, rows);
            let legacy = render(area, |ui, scene| {
                legacy_paint_card_chrome_clearing(ui, scene, "Open a picker", "Enter · click");
            });
            let block = render(area, |ui, scene| {
                let _ = CardChrome::new("Open a picker")
                    .meta("Enter · click")
                    .ink(palette())
                    .surface_style(
                        palette()
                            .pair(palette().ink, palette().surface)
                            .remove_modifier(Modifier::all()),
                    )
                    .draw(ui, scene);
            });
            assert_same(&legacy, &block, "card chrome clearing surface");
        }
    }
}
