use core::fmt::{self, Write as _};

use ratatui_core::layout::Rect;

use super::{PartStyle, SlotFn};
use crate::id::{Id, Part};
use crate::measure::{Constraints, Size};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{DesignTokens, Family, StylePatch, Variant};
use crate::ui::{FrameRead, Ui};

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
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::TITLE,
        Part::DETAIL,
        Part::HELP,
        Part::ACTIONS,
    ];

    pub const TOO_SMALL: &'static str = "Terminal too small";

    pub const QUIT: &'static str = "q Quit";

    pub const ROWS: u16 = 5;

    pub const fn new(id: Id, product: &'a str) -> Self {
        TooSmall {
            id,
            product,
            minimum: None,
            variant: Variant::DEFAULT,
            ov: PartStyle::new(),
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    #[must_use]
    pub const fn minimum(mut self, w: u16, h: u16) -> Self {
        self.minimum = Some((w, h));
        self
    }

    #[must_use]
    pub const fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }

    #[must_use]
    pub const fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub const fn slot(mut self, p: Part, f: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(p, f);
        self
    }

    pub const fn minimum_size(&self, d: &DesignTokens) -> (u16, u16) {
        match self.minimum {
            Some(m) => m,
            None => (d.size.min_width, d.size.min_height),
        }
    }

    pub const fn fits(&self, d: &DesignTokens, area: Rect) -> bool {
        let (w, h) = self.minimum_size(d);
        area.width >= w && area.height >= h
    }

    fn size_line(&self, d: &DesignTokens, area: Rect) -> LineBuf {
        let (w, h) = self.minimum_size(d);
        let mut line = LineBuf::new();

        let _ = write!(line, "Need {w}×{h}, have {}×{}", area.width, area.height);
        line
    }

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

    fn paint(w: u16, h: u16, f: impl FnOnce(&mut Ui<'_>, Rect)) -> Buffer {
        let area = Rect::new(0, 0, w, h);
        let theme = Theme::junie();
        paint_with_theme(&theme, area, f)
    }

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
