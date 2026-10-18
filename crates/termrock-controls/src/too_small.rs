#![doc = r" `TooSmall` — the below-minimum-size notice (`COMPONENT_ARCHITECTURE.md`"]
#![doc = r#" §16.4 item 7, §18.3 row 21; `DESIGN.md` "Responsive rules")."#]
#![doc = r""]
#![doc = r" `DESIGN.md` outranks the architecture document and fixes both the minimum"]
#![doc = r#" size and the notice verbatim: *"Minimum size is `72×20`. Below it both apps"#]
#![doc = r" show a centred four-line notice (product name, `Terminal too small`,"]
#![doc = r#" `Need 72×20, have W×H`, `q Quit`) and nothing else."* §16.4 item 7 repeats"#]
#![doc = r" the two library-owned strings as a *contract*, because three application"]
#![doc = r" tests assert them unchanged. They are `const`s here so no caller can retype"]
#![doc = r" one differently."]

use core::fmt::{self, Write as _};

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn};
use crate::id::{Id, Part};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{DesignTokens, Family, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

#[doc = r" A stack buffer wide enough for the size line at every `u16` pair."]
#[doc = r""]
#[doc = r" `Need 65535×65535, have 65535×65535` is 36 bytes; the buffer is 64, so the"]
#[doc = r" line is formatted **in place** and the notice allocates nothing. A"]
#[doc = r" `format!` here would put an allocation on the paint path of the one screen"]
#[doc = r" that renders when the terminal is already in a degraded state."]
#[derive(Clone, Copy, Debug)]
struct LineBuf {
    buf: [u8; 64],
    len: usize,
}

impl LineBuf {
    const fn new() -> Self {
        LineBuf {
            buf: [0; 64],
            len: 0,
        }
    }

    #[doc = r" What has been written so far. Every write is a whole `&str`, so the"]
    #[doc = r" prefix is always valid UTF-8 even if a write was refused."]
    fn as_str(&self) -> &str {
        self.buf
            .get(..self.len)
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or("")
    }
}

impl fmt::Write for LineBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len.saturating_add(s.len());
        let Some(dst) = self.buf.get_mut(self.len..end) else {
            return Err(fmt::Error);
        };
        dst.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[doc = r" The centred notice shown when the terminal is below the minimum size."]
#[doc = r""]
#[doc = r" ## Construction"]
#[doc = r" `TooSmall::new(id, product)`. `product` is the application's own name — the"]
#[doc = r" library bakes in no wordmark, exactly as [`Brand`](super::Brand) does not."]
#[doc = r""]
#[doc = r" ## Ownership"]
#[doc = r" Stateless. The caller owns the product string and decides *whether* to draw"]
#[doc = r" the notice; [`TooSmall::fits`] answers that from the same design tokens"]
#[doc = r" both phases already hold, so `update` and `draw` cannot disagree."]
#[doc = r""]
#[doc = r" ## Configuration"]
#[doc = r" `.minimum(w, h)` (default: `design.size.min_width × design.size.min_height`,"]
#[doc = r" which the built-in themes set to `72 × 20`), `.variant(Variant)`,"]
#[doc = r" `.patch`, `.patch_part`, `.slot`."]
#[doc = r""]
#[doc = r" ## Variants"]
#[doc = r" `Family::TOO_SMALL`; `DEFAULT` only."]
#[doc = r""]
#[doc = r" ## States"]
#[doc = r" None. The notice is never focused, hovered, pressed or disabled, and"]
#[doc = r" derives nothing from its props, so it resolves under `StateFlags::empty()`."]
#[doc = r""]
#[doc = r" ## Actions"]
#[doc = r" None; `TooSmall` has no `update` phase. The `q Quit` line is a *hint*: the"]
#[doc = r" key belongs to the application's own `KeyMap`, because a component that"]
#[doc = r" quit the process on a keystroke would be unusable in a showcase page."]
#[doc = r""]
#[doc = r" ## Focus"]
#[doc = r" Never a focus stop; registers no ring entry and no hit region. Nothing on"]
#[doc = r" this screen is clickable, so nothing is registered (R5)."]
#[doc = r""]
#[doc = r" ## Keyboard"]
#[doc = r" None."]
#[doc = r""]
#[doc = r" ## Mouse"]
#[doc = r" None; no `PartRef` is registered."]
#[doc = r""]
#[doc = r" ## Layout"]
#[doc = r" Five rows — the four lines of `DESIGN.md` with a blank row before the quit"]
#[doc = r" hint — centred vertically in `area` and each line centred horizontally."]
#[doc = r" `measure` returns `(the widest line, 5)`. `draw` returns the rect the block"]
#[doc = r" occupies, and paints nothing at all for a degenerate rect (R5). A line that"]
#[doc = r" falls below `area.bottom()` is dropped rather than clipped, so the notice"]
#[doc = r" never writes outside its own area even at `1 × 1`."]
#[doc = r""]
#[doc = r" ## Parts"]
#[doc = r" `CONTAINER` (the whole surface, filled on **every** non-degenerate frame,"]
#[doc = r" which is why it is `PARTS[0]`), `TITLE` (the product name), `DETAIL`"]
#[doc = r" ([`TooSmall::TOO_SMALL`]), `HELP` (the `Need …, have …` line) and `ACTIONS`"]
#[doc = r" ([`TooSmall::QUIT`])."]
#[doc = r""]
#[doc = r" Under the dedicated built-in recipe, `CONTAINER` and `TITLE` use the primary"]
#[doc = r" tone (`TITLE` is bold), followed by secondary `DETAIL`, muted `HELP`, and"]
#[doc = r" faint `ACTIONS`."]
#[doc = r""]
#[doc = r" ## Overrides"]
#[doc = r" `.patch` and `.patch_part` reach `Part::CONTAINER`, `Part::TITLE`,"]
#[doc = r" `Part::DETAIL`, `Part::HELP` and `Part::ACTIONS`. `.slot(Part::CONTAINER,"]
#[doc = r" …)` replaces the whole surface and suppresses the four lines;"]
#[doc = r" `.slot(Part::TITLE, …)`, `.slot(Part::DETAIL, …)`, `.slot(Part::HELP, …)`"]
#[doc = r" and `.slot(Part::ACTIONS, …)` replace one line each, keeping its rect."]
#[doc = r""]
#[doc = r" ## Identity"]
#[doc = r" One `Id` per instance, used to attribute style resolution and overrides;"]
#[doc = r" no items."]
#[doc = r""]
#[doc = r" ## Testing"]
#[doc = r" `TooSmallCase` with no capabilities. The pinned copy is asserted by"]
#[doc = r" `too_small::the_notice_is_the_four_pinned_lines_with_both_sizes`; the"]
#[doc = r" slot contract is asserted in both directions by"]
#[doc = r" `too_small::a_slot_changes_painted_cells_for_exactly_container_title_detail_help_and_actions`;"]
#[doc = r" `too_small::survives_tiny_rects_0x0_to_3x3` covers the dedicated notice's"]
#[doc = r" smallest real inputs; the"]
#[doc = r" application tests `showcase::below_minimum_size_shows_reduced_state` and"]
#[doc = r" its two siblings continue to assert the same strings unchanged (§16.4"]
#[doc = r" item 7)."]
#[doc = r""]
#[doc = r" ## Invariants"]
#[doc = r" [`TooSmall::TOO_SMALL`] and [`TooSmall::QUIT`], and the shape of the size"]
#[doc = r" line, are a **contract**: three application tests match on them, so they"]
#[doc = r" are `const` and are never composed from a theme, a glyph or a token. The"]
#[doc = r" notice never allocates and never writes outside `area`."]
pub struct TooSmall<'a> {
    id: Id,
    product: &'a str,
    minimum: Option<(u16, u16)>,
    variant: Variant,
    ov: PartStyle<'a>,
}

impl fmt::Debug for TooSmall<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TooSmall")
            .field("id", &self.id)
            .field("product", &self.product)
            .field("minimum", &self.minimum)
            .field("variant", &self.variant)
            .field("overrides", &self.ov)
            .finish_non_exhaustive()
    }
}

impl<'a> TooSmall<'a> {
    #[doc = r" The parts this component styles."]
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::TITLE,
        Part::DETAIL,
        Part::HELP,
        Part::ACTIONS,
    ];

    #[doc = r" The second line, verbatim (`DESIGN.md`; §16.4 item 7)."]
    pub const TOO_SMALL: &'static str = "Terminal too small";

    #[doc = r" The fourth line, verbatim (`DESIGN.md`)."]
    pub const QUIT: &'static str = "q Quit";

    #[doc = r" Rows the notice occupies: four lines plus the blank row that separates"]
    #[doc = r" the quit hint from the size line."]
    pub const ROWS: u16 = 5;

    #[doc = r" The notice for `product`."]
    pub const fn new(id: Id, product: &'a str) -> Self {
        TooSmall {
            id,
            product,
            minimum: None,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    #[doc = r" The id."]
    pub const fn id(&self) -> Id {
        self.id
    }

    #[doc = r" Override the minimum size the notice reports."]
    #[doc = r""]
    #[doc = r" Left unset, the notice reads `design.size.min_width` and"]
    #[doc = r" `design.size.min_height`, so the number in the copy and the number the"]
    #[doc = r" application compares against are the same one."]
    #[must_use]
    pub const fn minimum(mut self, w: u16, h: u16) -> Self {
        self.minimum = Some((w, h));
        self
    }

    #[doc = r" Set the variant."]
    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[doc = r" An instance patch over every part (precedence 6)."]
    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[doc = r" Per-part patches."]
    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[doc = r" Replace one part's painting; the layout is unchanged."]
    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    #[doc = r" The minimum size this notice reports, from `.minimum` or the tokens."]
    pub const fn minimum_size(&self, d: &DesignTokens) -> (u16, u16) {
        match self.minimum {
            Some(m) => m,
            None => (d.size.min_width, d.size.min_height),
        }
    }

    #[doc = r" Whether `area` is large enough, so the caller draws its real screen."]
    #[doc = r""]
    #[doc = r" Both phases have a `&DesignTokens` (`Cx::design`, `Ui::design`), so the"]
    #[doc = r" update phase and the draw phase answer this from the same numbers the"]
    #[doc = r" copy is written from — the three applications each derived the test"]
    #[doc = r" twice from two constants instead."]
    pub const fn fits(&self, d: &DesignTokens, area: Rect) -> bool {
        let (w, h) = self.minimum_size(d);
        area.width >= w && area.height >= h
    }

    #[doc = r" The size line, formatted in place."]
    fn size_line(&self, d: &DesignTokens, area: Rect) -> LineBuf {
        let (w, h) = self.minimum_size(d);
        let mut line = LineBuf::new();

        let _ = write!(line, "Need {w}×{h}, have {}×{}", area.width, area.height);
        line
    }

    #[doc = r" Paint one centred line of the notice."]
    fn line(&self, ui: &mut Ui<'_>, area: Rect, y: u16, part: Part, text: &str) {
        if y >= area.bottom() {
            return;
        }
        let x = area
            .x
            .saturating_add(area.width.saturating_sub(width(text)) / 2);
        let row = Rect {
            x,
            y,
            width: width(text).min(area.width),
            height: 1,
        };
        if let Some(f) = self.ov.slot_for(part) {
            f(ui, row);
            return;
        }

        let live = PartStyle::flags(StateFlags::empty(), StateFlags::empty());
        let s = self
            .ov
            .style(ui, self.id, Family::TOO_SMALL, self.variant, part, live);
        ui.paint_str(row, text, s.style);
    }

    #[doc = r" The draw phase; returns the rect the block occupies."]
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect {
        if area.is_empty() {
            return area;
        }
        let top = area
            .y
            .saturating_add(area.height.saturating_sub(Self::ROWS) / 2);
        let used = Rect {
            x: area.x,
            y: top,
            width: area.width,
            height: Self::ROWS.min(area.bottom().saturating_sub(top)),
        };
        let live = PartStyle::flags(StateFlags::empty(), StateFlags::empty());
        if let Some(f) = self.ov.slot_for(Part::CONTAINER) {
            f(ui, area);
            return used;
        }
        let c = self.ov.style(
            ui,
            self.id,
            Family::TOO_SMALL,
            self.variant,
            Part::CONTAINER,
            live,
        );
        ui.fill(area, c.style);
        let size_text = self.size_line(ui.design(), area);
        self.line(ui, area, top, Part::TITLE, self.product);
        self.line(
            ui,
            area,
            top.saturating_add(1),
            Part::DETAIL,
            Self::TOO_SMALL,
        );
        self.line(
            ui,
            area,
            top.saturating_add(2),
            Part::HELP,
            size_text.as_str(),
        );

        self.line(ui, area, top.saturating_add(4), Part::ACTIONS, Self::QUIT);
        used
    }

    #[doc = r" The natural size: the widest line, and five rows."]
    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let d = ui.design();

        let size_line = self.size_line(
            d,
            Rect {
                x: 0,
                y: 0,
                width: c.max.0,
                height: c.max.1,
            },
        );
        let widest = width(self.product)
            .max(width(Self::TOO_SMALL))
            .max(width(size_line.as_str()))
            .max(width(Self::QUIT));
        Size::exact(widest, Self::ROWS).fit(c)
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;
    use ratatui_core::style::{Color, Style};

    use super::*;
    use crate::theme::{Role, Surface, Theme, resolve::bind_role};
    use crate::ui::cx::LastFrame;
    use crate::ui::{FrameState, UiCore};

    const ID: Id = Id::root("too_small.tests");
    const PRODUCT: &str = "Junie Design system";

    #[doc = r" Draws `f` into a `w × h` buffer and returns it."]
    fn paint(w: u16, h: u16, f: impl FnOnce(&mut Ui<'_>, Rect)) -> Buffer {
        let area = Rect::new(0, 0, w, h);
        let theme = Theme::junie();
        paint_with_theme(&theme, area, f)
    }

    #[doc = r" Draws `f` into `area` with the supplied theme."]
    fn paint_with_theme(theme: &Theme, area: Rect, f: impl FnOnce(&mut Ui<'_>, Rect)) -> Buffer {
        let mut fs = FrameState::default();
        fs.reset(1, area);
        let mut page = Buffer::empty(area);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut fs, &mut page, &mut core, theme, &last);
            f(&mut ui, area);
        }
        page
    }

    #[doc = r" One row of a buffer as text, trailing blanks trimmed."]
    fn row(buf: &Buffer, y: u16) -> String {
        let w = buf.area.width;
        let mut s = String::new();
        for x in 0..w {
            if let Some(c) = buf.cell((x, y)) {
                s.push_str(c.symbol());
            }
        }
        s.trim_end().to_owned()
    }

    #[doc = r" §16.4 item 7 and `DESIGN.md` together: the four lines, verbatim, in"]
    #[doc = r" order, with the blank row before the quit hint, each centred, and the"]
    #[doc = r" size line reporting the **minimum** and the **actual** size in that"]
    #[doc = r" order. Three application tests match on these strings."]
    #[test]
    fn the_notice_is_the_four_pinned_lines_with_both_sizes() {
        let buf = paint(40, 15, |ui, area| {
            TooSmall::new(ID, PRODUCT).draw(ui, area);
        });

        assert_eq!(row(&buf, 5).trim(), PRODUCT);
        assert_eq!(row(&buf, 6).trim(), "Terminal too small");
        assert_eq!(row(&buf, 7).trim(), "Need 72×20, have 40×15");
        assert_eq!(row(&buf, 8), "", "row 3 of the block is blank");
        assert_eq!(row(&buf, 9).trim(), "q Quit");
        assert_eq!(row(&buf, 4), "", "nothing above the block");
        assert_eq!(row(&buf, 10), "", "nothing below it");

        for (y, text) in [
            (5u16, PRODUCT),
            (6, "Terminal too small"),
            (7, "Need 72×20, have 40×15"),
            (9, "q Quit"),
        ] {
            let painted = row(&buf, y);
            let lead = painted.len().saturating_sub(painted.trim_start().len());
            assert_eq!(
                lead as u16,
                (40 - width(text)) / 2,
                "line {y} is not centred: {painted:?}"
            );
        }

        assert_eq!(TooSmall::TOO_SMALL, "Terminal too small");
        assert_eq!(TooSmall::QUIT, "q Quit");

        let buf = paint(40, 15, |ui, area| {
            TooSmall::new(ID, PRODUCT).minimum(100, 30).draw(ui, area);
        });
        assert_eq!(row(&buf, 7).trim(), "Need 100×30, have 40×15");
    }

    #[test]
    fn quit_hint_paints_with_the_faint_tone_in_both_builtin_themes() {
        let area = Rect::new(0, 0, 40, 15);
        let top = (area.height - TooSmall::ROWS) / 2;
        let x = (area.width - width(TooSmall::QUIT)) / 2;
        for theme in [Theme::junie(), Theme::paper()] {
            let expected = bind_role(
                &theme,
                Role::Fg(crate::theme::FgStep::Faint),
                Surface::Canvas,
            );
            let buffer = paint_with_theme(&theme, area, |ui, area| {
                TooSmall::new(ID, PRODUCT).draw(ui, area);
            });
            assert_eq!(buffer.cell((x, top + 4)).map(|cell| cell.fg), expected);
        }
    }

    #[doc = r" R5 and the degenerate-rect rule. The notice is the one screen that is"]
    #[doc = r" drawn *because* the area is too small, so every rect below the block's"]
    #[doc = r" own five rows is a real case, not a theoretical one: `draw` must drop"]
    #[doc = r" the lines that do not fit rather than clip them into another row, and"]
    #[doc = r" must write nothing at all into a zero rect."]
    #[test]
    fn draw_stays_inside_its_area() {
        let full = Rect::new(0, 0, 40, 15);
        for h in 0..=6u16 {
            let area = Rect::new(2, 3, 24, h);
            let theme = Theme::junie();
            let mut fs = FrameState::default();
            fs.reset(1, full);
            let mut page = Buffer::empty(full);
            let mut core = UiCore::default();
            let last = LastFrame::default();
            let used = {
                let mut ui = Ui::new(&mut fs, &mut page, &mut core, &theme, &last);
                TooSmall::new(ID, PRODUCT).draw(&mut ui, area)
            };
            assert!(
                used.height <= area.height && used.y >= area.y,
                "h={h}: returned {used:?} outside {area:?}"
            );
            for y in 0..full.height {
                for x in 0..full.width {
                    let inside =
                        x >= area.x && x < area.right() && y >= area.y && y < area.bottom();
                    if inside {
                        continue;
                    }
                    let sym = page.cell((x, y)).map(|c| c.symbol().to_owned());
                    assert_eq!(
                        sym.as_deref(),
                        Some(" "),
                        "h={h}: wrote at ({x},{y}), outside {area:?}"
                    );
                }
            }
        }

        let buf = paint(40, 15, |ui, _| {
            let used = TooSmall::new(ID, PRODUCT).draw(ui, Rect::new(4, 4, 0, 0));
            assert_eq!(used, Rect::new(4, 4, 0, 0));
        });
        for y in 0..15 {
            assert_eq!(row(&buf, y), "", "a zero rect painted row {y}");
        }
    }

    #[doc = r" §16.2 case 19's exact tiny-area matrix. This is separate from the"]
    #[doc = r" inset containment test because all sixteen sizes are real inputs for"]
    #[doc = r" the below-minimum screen, including one-axis-empty rectangles."]
    #[test]
    fn survives_tiny_rects_0x0_to_3x3() {
        for width in 0..=3 {
            for height in 0..=3 {
                let area = Rect::new(0, 0, width, height);
                let buffer = paint(width, height, |ui, area| {
                    let used = TooSmall::new(ID, PRODUCT).draw(ui, area);
                    assert_eq!(used.x, area.x);
                    assert!(used.y >= area.y);
                    assert!(used.right() <= area.right());
                    assert!(used.bottom() <= area.bottom());
                });
                assert_eq!(buffer.area, area);
            }
        }
    }

    #[doc = r" §16.2 case 5's stable-render contract at this component's boundary."]
    #[test]
    fn draw_twice_is_byte_identical() {
        let first = paint(40, 15, |ui, area| {
            TooSmall::new(ID, PRODUCT).draw(ui, area);
        });
        let second = paint(40, 15, |ui, area| {
            TooSmall::new(ID, PRODUCT).draw(ui, area);
        });
        assert_eq!(first, second);
    }

    #[doc = r" Invariant R (§45.3): exactly the five parts named in `## Overrides`"]
    #[doc = r" honour `.slot`. The full built-in `Part` set makes the inverse half"]
    #[doc = r" real: accidentally consulting a slot for an unnamed part fails too."]
    #[test]
    fn a_slot_changes_painted_cells_for_exactly_container_title_detail_help_and_actions() {
        fn painted(slot: Option<Part>) -> Buffer {
            paint(40, 15, |ui, area| {
                let replacement = |ui: &mut Ui<'_>, area: Rect| {
                    ui.paint_str(
                        area,
                        "########################################",
                        Style::new(),
                    );
                };
                let mut notice = TooSmall::new(ID, PRODUCT);
                if let Some(part) = slot {
                    notice = notice.slot(part, &replacement);
                }
                notice.draw(ui, area);
            })
        }

        let plain = painted(None);
        for part in Part::ALL {
            let changed = painted(Some(*part)) != plain;
            assert_eq!(
                changed,
                TooSmall::PARTS.contains(part),
                "slot handling and the documented part set disagree for {part:?}"
            );
        }
        assert_eq!(
            TooSmall::PARTS,
            &[
                Part::CONTAINER,
                Part::TITLE,
                Part::DETAIL,
                Part::HELP,
                Part::ACTIONS,
            ]
        );
    }

    #[doc = r" A slot receives the rect its default painter owns: the surface gets"]
    #[doc = r" the whole area and each line gets exactly its display-width cells at"]
    #[doc = r" the centred position. Substitution must not gain trailing columns."]
    #[test]
    fn slots_keep_the_default_painters_geometry() {
        let area = Rect::new(3, 2, 40, 15);
        let top = area.y + (area.height - TooSmall::ROWS) / 2;
        let minimum = Theme::junie();
        let size = TooSmall::new(ID, PRODUCT).size_line(&minimum.design, area);
        for (part, y, text) in [
            (Part::TITLE, top, PRODUCT),
            (Part::DETAIL, top + 1, TooSmall::TOO_SMALL),
            (Part::HELP, top + 2, size.as_str()),
            (Part::ACTIONS, top + 4, TooSmall::QUIT),
        ] {
            let seen = Cell::new(Rect::ZERO);
            let replacement = |_ui: &mut Ui<'_>, rect: Rect| seen.set(rect);
            paint_with_theme(&minimum, area, |ui, area| {
                TooSmall::new(ID, PRODUCT)
                    .slot(part, &replacement)
                    .draw(ui, area);
            });
            assert_eq!(
                seen.get(),
                Rect::new(area.x + (area.width - width(text)) / 2, y, width(text), 1,),
                "wrong slot geometry for {part:?}"
            );
        }

        let normal = Cell::new(Rect::ZERO);
        paint_with_theme(&minimum, area, |ui, area| {
            normal.set(TooSmall::new(ID, PRODUCT).draw(ui, area));
        });
        assert_eq!(
            normal.get(),
            Rect::new(area.x, top, area.width, TooSmall::ROWS)
        );

        let seen = Cell::new(Rect::ZERO);
        let returned = Cell::new(Rect::ZERO);
        let replacement = |_ui: &mut Ui<'_>, rect: Rect| seen.set(rect);
        paint_with_theme(&minimum, area, |ui, area| {
            returned.set(
                TooSmall::new(ID, PRODUCT)
                    .slot(Part::CONTAINER, &replacement)
                    .draw(ui, area),
            );
        });
        assert_eq!(seen.get(), area);
        assert_eq!(
            returned.get(),
            normal.get(),
            "a CONTAINER slot replaces painting, not returned layout geometry"
        );
    }

    #[doc = r" Measurement describes the exact four-line block for the actual"]
    #[doc = r" offered size, while `fits` uses the same default or overridden minimum"]
    #[doc = r" that the third line reports."]
    #[test]
    fn fits_measure_and_copy_share_the_same_sizes() {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, 65_535, 65_535);
        let mut fs = FrameState::default();
        fs.reset(1, Rect::new(0, 0, 1, 1));
        let mut page = Buffer::empty(Rect::new(0, 0, 1, 1));
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let ui = Ui::new(&mut fs, &mut page, &mut core, &theme, &last);
        let notice = TooSmall::new(ID, PRODUCT).minimum(1, 1);
        let size_line = notice.size_line(&theme.design, area);
        let measured = notice.measure(&ui, Constraints::loose(area.width, area.height));
        let expected_width = width(PRODUCT)
            .max(width(TooSmall::TOO_SMALL))
            .max(width(size_line.as_str()))
            .max(width(TooSmall::QUIT));

        assert_eq!(measured, Size::exact(expected_width, TooSmall::ROWS));
        assert!(notice.fits(&theme.design, Rect::new(0, 0, 1, 1)));
        assert!(!notice.fits(&theme.design, Rect::new(0, 0, 0, 1)));
        assert_eq!(size_line.as_str(), "Need 1×1, have 65535×65535");
    }

    #[doc = r" §16.2 case 10's local-override contract. Both override layers change"]
    #[doc = r" this instance's cells while the borrowed theme remains byte-identical."]
    #[test]
    fn local_override_does_not_mutate_the_theme() {
        let theme = Theme::junie();
        let before = theme.clone();
        let area = Rect::new(0, 0, 40, 15);
        let plain = paint_with_theme(&theme, area, |ui, area| {
            TooSmall::new(ID, PRODUCT).draw(ui, area);
        });
        let global = StylePatch::new().set_fg(Role::Warning);
        let parts = [(Part::HELP, StylePatch::new().set_fg(Role::Danger))];
        let patched = paint_with_theme(&theme, area, |ui, area| {
            TooSmall::new(ID, PRODUCT)
                .patch(&global)
                .patch_part(&parts)
                .draw(ui, area);
        });

        assert_ne!(fgs(&patched), fgs(&plain));
        let top = (area.height - TooSmall::ROWS) / 2;
        let title_x = (area.width - width(PRODUCT)) / 2;
        let size_text = TooSmall::new(ID, PRODUCT).size_line(&theme.design, area);
        let help_x = (area.width - width(size_text.as_str())) / 2;
        assert_eq!(
            patched.cell((title_x, top)).map(|cell| cell.fg),
            bind_role(&theme, Role::Warning, Surface::Canvas),
            "the instance patch must reach an unqualified part"
        );
        assert_eq!(
            patched.cell((help_x, top + 2)).map(|cell| cell.fg),
            bind_role(&theme, Role::Danger, Surface::Canvas),
            "the per-part patch must merge after the instance patch"
        );
        assert_eq!(theme, before);
    }

    #[doc = r" §33: `PARTS` is a styling contract — exactly what `draw` resolves. The"]
    #[doc = r" property is asserted before the list: an instance patch aimed at each"]
    #[doc = r" declared part must change the painted buffer, so a part that is"]
    #[doc = r" declared but never resolved fails here rather than passing on the"]
    #[doc = r" strength of the `const` alone. `Part::CONTAINER` is `PARTS[0]` because"]
    #[doc = r" it is the only one painted on every non-degenerate frame."]
    #[test]
    fn every_declared_part_is_resolved_and_the_container_is_painted_every_frame() {
        let theme = Theme::junie();
        let marker = bind_role(&theme, Role::Danger, Surface::Canvas)
            .expect("the junie theme binds Role::Danger");
        let base = paint(40, 15, |ui, area| {
            TooSmall::new(ID, PRODUCT).draw(ui, area);
        });
        for part in TooSmall::PARTS {
            let patch = [(*part, StylePatch::new().set_fg(Role::Danger))];
            let patched = paint(40, 15, |ui, area| {
                TooSmall::new(ID, PRODUCT).patch_part(&patch).draw(ui, area);
            });
            assert_ne!(
                fgs(&patched),
                fgs(&base),
                "patching {part:?} changed nothing: it is declared but never resolved"
            );
            assert!(
                fgs(&patched).contains(&Some(marker)),
                "patching {part:?} did not reach the buffer"
            );
        }

        let patch = [(Part::CONTAINER, StylePatch::new().set_bg(Role::Danger))];
        let tiny = paint(6, 1, |ui, area| {
            TooSmall::new(ID, PRODUCT).patch_part(&patch).draw(ui, area);
        });
        assert_eq!(
            tiny.cell((0, 0)).map(|c| c.bg),
            Some(marker),
            "CONTAINER was not filled on a one-row frame"
        );
    }

    #[doc = r" Every painted foreground, in buffer order."]
    fn fgs(buf: &Buffer) -> Vec<Option<Color>> {
        let mut out = Vec::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push(buf.cell((x, y)).map(|c| c.fg));
            }
        }
        out
    }
}
