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
