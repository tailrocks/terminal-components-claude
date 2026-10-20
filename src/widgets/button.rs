use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::theme::{ButtonKind, Kind};
use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;
use ratatui::crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub struct Button {
    pub id: WidgetId,
    pub label: String,
    pub kind: ButtonKind,
    pub disabled: bool,

    pub on: Option<bool>,
    pub busy: bool,
    pub area: Rect,

    pub focus_visible: crate::ui::ctx::FocusVisible,

    pub mnemonic: Option<usize>,

    pub tip: Option<crate::ui::tooltip::Tip>,

    pub focusable: bool,
}

impl Button {
    pub fn new(id: WidgetId, label: &str, kind: ButtonKind) -> Self {
        Self {
            id,
            label: label.to_owned(),
            kind,
            disabled: false,
            on: None,
            busy: false,
            area: Rect::ZERO,
            focus_visible: crate::ui::ctx::FocusVisible::Always,
            mnemonic: None,
            tip: None,
            focusable: true,
        }
    }

    pub fn focusable(mut self, on: bool) -> Self {
        self.focusable = on;
        self
    }

    pub fn tooltip(mut self, text: &str) -> Self {
        let key = self.tip.take().and_then(|t| t.shortcut);
        self.tip = Some(crate::ui::tooltip::Tip {
            text: text.to_owned(),
            shortcut: key,
        });
        self
    }

    pub fn shortcut(mut self, key: &str) -> Self {
        let mut tip = self.tip.take().unwrap_or_default();
        tip.shortcut = Some(key.to_owned());
        self.tip = Some(tip);
        self
    }

    pub fn focus_visible(mut self, mode: crate::ui::ctx::FocusVisible) -> Self {
        self.focus_visible = mode;
        self
    }

    pub fn primary(id: WidgetId, label: &str) -> Self {
        Self::new(id, label, ButtonKind::Primary)
    }
    pub fn secondary(id: WidgetId, label: &str) -> Self {
        Self::new(id, label, ButtonKind::Secondary)
    }
    pub fn subtle(id: WidgetId, label: &str) -> Self {
        Self::new(id, label, ButtonKind::Subtle)
    }
    pub fn danger(id: WidgetId, label: &str) -> Self {
        Self::new(id, label, ButtonKind::Danger)
    }
    pub fn toggle(id: WidgetId, label: &str, on: bool) -> Self {
        let mut b = Self::new(id, label, ButtonKind::Toggle);
        b.on = Some(on);
        b
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn width(&self) -> u16 {
        let marker = if self.on.is_some() || self.busy { 2 } else { 0 };
        (width(&self.label) + 2 + marker) as u16
    }

    pub fn hint(&self) -> crate::widgets::keyhint::Hint {
        if self.busy {
            return crate::widgets::keyhint::Hint::new("", "Working…")
                .priority(crate::widgets::keyhint::HintPriority::Normal);
        }
        crate::widgets::keyhint::Hint::new("Enter", self.label.clone())
    }

    pub fn can_activate(&self) -> bool {
        !self.disabled && !self.busy
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, bool) {
        if key.is(KeyCode::Enter) || key.is_char(' ') {
            if self.can_activate() {
                self.toggle_if_needed();
                (Outcome::Changed, true)
            } else {
                (Outcome::Consumed, false)
            }
        } else {
            (Outcome::Ignored, false)
        }
    }

    pub fn on_click(&mut self) -> bool {
        if self.can_activate() {
            self.toggle_if_needed();
            true
        } else {
            false
        }
    }

    fn toggle_if_needed(&mut self) {
        if let Some(on) = self.on.as_mut() {
            *on = !*on;
        }
    }

    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: ratatui::style::Color,
    ) {
        let area = area.intersection(*buf.area());
        let w = self.width().min(area.width);
        let area = Rect::new(area.x, area.y, w, 1.min(area.height));
        self.area = area;
        if area.is_empty() {
            return;
        }
        let mut s = ctx.state(self.id);
        s.focused = ctx.interaction.focus_visible(self.id, self.focus_visible);
        s.disabled = self.disabled;
        s.busy = self.busy;
        s.selected = self.on.unwrap_or(false);
        if self.disabled {
            s.hovered = false;
            s.pressed = false;
        }
        if self.busy {
            s.pressed = false;
        }
        let t = ctx.theme;

        let it = t.interactive(Kind::Button(self.kind), s, t.plane_of(bg));
        let style = it.style;
        let filled = it.on_fill;
        let busy_ink = style.fg.unwrap_or(t.text_secondary);
        let gutter = it.gutter_style;
        let mut text = String::new();
        if self.busy {
            text.push_str(t.glyphs.spinner_frame(ctx.interaction.tick));
            text.push(' ');
        } else if let Some(on) = self.on {
            text.push_str(if on {
                t.glyphs.dot_on
            } else {
                t.glyphs.dot_off
            });
            text.push(' ');
        }
        text.push_str(&self.label);
        let text = crate::ui::text::fit(&text, (w as usize).saturating_sub(2));
        buf.set_string(area.x, area.y, it.gutter, gutter);
        buf.set_string(area.x + 1, area.y, &text, style);

        if let Some(on) = self.on
            && !self.disabled
        {
            let ms = style.fg(if on { t.accent } else { t.text_muted });
            let ms = if s.pressed { style } else { ms };
            buf.set_string(
                area.x + 1,
                area.y,
                if on {
                    t.glyphs.dot_on
                } else {
                    t.glyphs.dot_off
                },
                ms,
            );
        }
        if self.busy {
            buf.set_string(
                area.x + 1,
                area.y,
                t.glyphs.spinner_frame(ctx.interaction.tick),
                style.fg(if filled { busy_ink } else { t.accent }),
            );
        }

        if let Some(mi) = self.mnemonic
            && !self.busy
            && self.on.is_none()
            && text == self.label
            && let Some(c) = self.label.chars().nth(mi)
        {
            let x = area.x + 1 + width(&self.label.chars().take(mi).collect::<String>()) as u16;
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_style(style.add_modifier(ratatui::style::Modifier::UNDERLINED));
                debug_assert_eq!(cell.symbol(), c.to_string());
            }
        }
        buf.set_string(area.x + w - 1, area.y, " ", style);
        if self.focusable {
            ctx.control(self.id, area, self.disabled);
        } else {
            ctx.clickable(self.id, area);
        }
        if let Some(tip) = &self.tip {
            crate::ui::tooltip::show(ctx, self.id, area, tip, None);
        }
    }
}

fn wrap_lines(width: u16, widths: &[u16], gap: u16) -> Vec<Vec<usize>> {
    let mut lines: Vec<Vec<usize>> = vec![vec![]];
    let mut used = 0u16;
    for (i, &w) in widths.iter().enumerate() {
        let line = lines.last_mut().expect("one line");
        let need = if line.is_empty() {
            w
        } else {
            used.saturating_add(gap).saturating_add(w)
        };
        if line.is_empty() || need <= width {
            line.push(i);
            used = need;
        } else {
            lines.push(vec![i]);
            used = w;
        }
    }
    lines
}

pub fn row_lines(width: u16, widths: &[u16], gap: u16) -> u16 {
    if widths.is_empty() {
        return 1;
    }
    wrap_lines(width, widths, gap).len() as u16
}

pub fn row_layout(area: Rect, widths: &[u16], gap: u16) -> Vec<Rect> {
    place_lines(area, widths, gap, false)
}

pub fn row_layout_right(area: Rect, widths: &[u16], gap: u16) -> Vec<Rect> {
    place_lines(area, widths, gap, true)
}

fn place_lines(area: Rect, widths: &[u16], gap: u16, right: bool) -> Vec<Rect> {
    let mut out = vec![Rect::ZERO; widths.len()];
    for (k, line) in wrap_lines(area.width, widths, gap).iter().enumerate() {
        let y = area.y.saturating_add(k as u16);
        let total = line.iter().map(|&i| widths[i]).sum::<u16>()
            + gap * line.len().saturating_sub(1) as u16;
        let mut x = if right {
            area.right().saturating_sub(total).max(area.x)
        } else {
            area.x
        };
        for &i in line {
            let w = widths[i].min(area.right().saturating_sub(x));
            out[i] = Rect::new(x, y, w, area.height.min(1));
            x = x.saturating_add(w).saturating_add(gap);
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowAlign {
    #[default]
    Start,

    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowOverflow {
    #[default]
    More,

    Wrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ButtonRow {
    pub align: RowAlign,

    pub gap: Option<u16>,

    pub overflow: RowOverflow,

    pub wrap: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RowLayout {
    pub rects: Vec<Option<Rect>>,

    pub more: Option<Rect>,

    pub hidden: Vec<usize>,
}

pub const MORE_LABEL: &str = "…";

impl ButtonRow {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn align(mut self, align: RowAlign) -> Self {
        self.align = align;
        self
    }
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = Some(gap);
        self
    }
    pub fn overflow(mut self, overflow: RowOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }

    pub fn step<'a>(
        &self,
        buttons: impl IntoIterator<Item = &'a Button>,
        cur: Option<WidgetId>,
        forward: bool,
    ) -> Option<WidgetId> {
        use crate::core::list::{Cursor, Moved, RowFn};
        let row: Vec<&Button> = buttons.into_iter().collect();
        let i = row.iter().position(|b| Some(b.id) == cur)?;
        let enabled = |j: usize| !row[j].disabled;
        let label = |j: usize| row[j].label.as_str();
        let src = RowFn {
            len: row.len(),
            enabled: &enabled,
            label: &label,
        };
        let mut c = Cursor::new(i)
            .axis(crate::core::hit::Axis::Horizontal)
            .wrap(self.wrap);
        match c.step(if forward { 1 } else { -1 }, &src) {
            Moved::To(j) => Some(row[j].id),
            _ => None,
        }
    }

    pub fn more_id(owner: WidgetId) -> WidgetId {
        owner.sub("more")
    }

    fn gap_for(&self, theme: &crate::theme::Theme) -> u16 {
        self.gap.unwrap_or(theme.spacing.button_gap)
    }

    fn primary(buttons: &[Button]) -> Option<usize> {
        buttons
            .iter()
            .position(|b| matches!(b.kind, ButtonKind::Primary | ButtonKind::Danger))
            .or_else(|| buttons.len().checked_sub(1))
    }

    fn lines(&self, width: u16, widths: &[u16], gap: u16) -> Vec<Vec<usize>> {
        wrap_lines(width, widths, gap)
    }

    pub fn width(&self, buttons: &[Button], theme: &crate::theme::Theme) -> u16 {
        let n = buttons.len() as u16;
        buttons.iter().map(Button::width).sum::<u16>() + self.gap_for(theme) * n.saturating_sub(1)
    }

    pub fn rows(&self, width: u16, buttons: &[Button], theme: &crate::theme::Theme) -> u16 {
        match self.overflow {
            RowOverflow::More => 1,
            RowOverflow::Wrap => {
                let widths: Vec<u16> = buttons.iter().map(Button::width).collect();
                self.lines(width, &widths, self.gap_for(theme)).len() as u16
            }
        }
    }

    pub fn rows_at_gap(&self, width: u16, buttons: &[Button], gap: u16) -> u16 {
        match self.overflow {
            RowOverflow::More => 1,
            RowOverflow::Wrap => {
                let widths: Vec<u16> = buttons.iter().map(Button::width).collect();
                self.lines(width, &widths, self.gap.unwrap_or(gap)).len() as u16
            }
        }
    }

    pub fn layout(&self, area: Rect, buttons: &[Button], theme: &crate::theme::Theme) -> RowLayout {
        let gap = self.gap_for(theme);
        let widths: Vec<u16> = buttons.iter().map(Button::width).collect();
        let mut out = RowLayout {
            rects: vec![None; buttons.len()],
            ..Default::default()
        };
        if area.is_empty() || buttons.is_empty() {
            return out;
        }
        let place = |line: &[(usize, u16)], y: u16, out: &mut RowLayout| -> Vec<Rect> {
            let total = line.iter().map(|(_, w)| *w).sum::<u16>()
                + gap * line.len().saturating_sub(1) as u16;
            let x = match self.align {
                RowAlign::Start => area.x,
                RowAlign::End => area.right().saturating_sub(total).max(area.x),
            };

            let mut cx = x;
            let rects: Vec<Rect> = line
                .iter()
                .map(|(_, w)| {
                    let w = (*w).min(area.right().saturating_sub(cx));
                    let r = Rect::new(cx, y, w, 1);
                    cx = cx.saturating_add(w).saturating_add(gap);
                    r
                })
                .collect();
            for ((i, _), r) in line.iter().zip(&rects) {
                if *i < out.rects.len() {
                    out.rects[*i] = Some(*r);
                }
            }
            rects
        };
        match self.overflow {
            RowOverflow::Wrap => {
                for (k, line) in self.lines(area.width, &widths, gap).iter().enumerate() {
                    let y = area.y + k as u16;
                    if y >= area.bottom() {
                        break;
                    }
                    let line: Vec<(usize, u16)> = line.iter().map(|&i| (i, widths[i])).collect();
                    place(&line, y, &mut out);
                }
            }
            RowOverflow::More => {
                let total = widths.iter().sum::<u16>() + gap * (widths.len() as u16 - 1);
                if total <= area.width {
                    let line: Vec<(usize, u16)> = widths.iter().copied().enumerate().collect();
                    place(&line, area.y, &mut out);
                    return out;
                }

                let primary = Self::primary(buttons).unwrap_or(0);
                let more_w = (width(theme.glyphs.ellipsis) + 2) as u16;
                let mut budget = area.width.saturating_sub(widths[primary] + gap + more_w);
                let mut keep = vec![false; buttons.len()];
                keep[primary] = true;
                for (i, &w) in widths.iter().enumerate() {
                    if i == primary {
                        continue;
                    }
                    if w + gap <= budget {
                        budget -= w + gap;
                        keep[i] = true;
                    } else {
                        break;
                    }
                }
                out.hidden = (0..buttons.len()).filter(|i| !keep[*i]).collect();

                let mut line: Vec<(usize, u16)> = Vec::new();
                for (i, &w) in widths.iter().enumerate() {
                    if keep[i] && i != primary {
                        line.push((i, w));
                    }
                }
                const MORE: usize = usize::MAX;
                if primary + 1 == buttons.len() {
                    line.push((MORE, more_w));
                    line.push((primary, widths[primary]));
                } else {
                    line.insert(
                        line.iter()
                            .position(|(i, _)| *i > primary)
                            .unwrap_or(line.len()),
                        (primary, widths[primary]),
                    );
                    line.push((MORE, more_w));
                }
                let rects = place(&line, area.y, &mut out);
                out.more = line
                    .iter()
                    .zip(rects)
                    .find(|((i, _), _)| *i == MORE)
                    .map(|(_, r)| r)
                    .filter(|r| !r.is_empty());
            }
        }
        out
    }

    pub fn render(
        &self,
        owner: WidgetId,
        area: Rect,
        buttons: &mut [Button],
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: ratatui::style::Color,
    ) -> RowLayout {
        let layout = self.layout(area, buttons, ctx.theme);
        for (b, r) in buttons.iter_mut().zip(&layout.rects) {
            match r {
                Some(r) => b.render(*r, buf, ctx, bg),
                None => b.area = Rect::ZERO,
            }
        }
        if let Some(r) = layout.more {
            let mut more = Button::subtle(Self::more_id(owner), ctx.theme.glyphs.ellipsis);
            more.render(r, buf, ctx, bg);
        }
        layout
    }
}

impl crate::core::pointer::MouseTarget for Button {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<()> {
        use crate::core::response::Response;
        if !ev.is(crate::core::intent::Phase::Click) {
            return Response::ignored();
        }
        if self.on_click() {
            Response::emit(())
        } else {
            Response::consumed()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn drawer_actions() -> Vec<Button> {
        let id = WidgetId::of("row");
        vec![
            Button::secondary(id.sub("a"), "Refresh"),
            Button::secondary(id.sub("b"), "Validate"),
            Button::secondary(id.sub("c"), "Edit…"),
            Button::danger(id.sub("d"), "Remove…"),
        ]
    }

    #[test]
    fn a_narrow_row_folds_buttons_and_keeps_the_primary_whole() {
        let t = Theme::junie();
        let buttons = drawer_actions();
        let area = Rect::new(0, 0, 30, 1);
        let l = ButtonRow::new().layout(area, &buttons, &t);
        let primary = l.rects[3].expect("the primary stays");
        assert_eq!(primary.width, buttons[3].width(), "never cut");
        assert!(primary.right() <= area.right());
        assert_eq!(l.hidden, vec![1, 2]);
        let more = l.more.expect("a … button");
        assert!(more.right() <= primary.x, "… sits before the primary");

        let mut drawn: Vec<Rect> = l.rects.iter().flatten().copied().collect();
        drawn.push(more);
        drawn.sort_by_key(|r| r.x);
        for pair in drawn.windows(2) {
            assert!(pair[0].right() + 2 <= pair[1].x, "{drawn:?}");
        }
    }

    #[test]
    fn row_layout_wraps_the_buttons_that_do_not_fit_instead_of_dropping_them() {
        let buttons = drawer_actions();
        let widths: Vec<u16> = buttons.iter().map(Button::width).collect();
        let area = Rect::new(4, 3, 30, 1);
        assert_eq!(row_lines(area.width, &widths, 2), 2);
        for rects in [
            row_layout(area, &widths, 2),
            row_layout_right(area, &widths, 2),
        ] {
            for (r, w) in rects.iter().zip(&widths) {
                assert_eq!(r.width, *w, "never cut: {rects:?}");
                assert!(r.x >= area.x && r.right() <= area.right(), "{rects:?}");
            }
            assert_eq!(rects[0].y, 3);
            assert_eq!(rects[3].y, 4, "the last button wraps: {rects:?}");
        }

        let wide = Rect::new(0, 0, 80, 1);
        assert_eq!(row_lines(wide.width, &widths, 2), 1);
        assert!(row_layout(wide, &widths, 2).iter().all(|r| r.y == 0));
    }

    #[test]
    fn a_wrapping_row_continues_on_the_next_line() {
        let t = Theme::junie();
        let buttons = drawer_actions();
        let row = ButtonRow::new().overflow(RowOverflow::Wrap);
        assert_eq!(row.rows(30, &buttons, &t), 2);
        let l = row.layout(Rect::new(0, 0, 30, 2), &buttons, &t);
        assert!(l.hidden.is_empty() && l.more.is_none());
        let ys: Vec<u16> = l.rects.iter().map(|r| r.unwrap().y).collect();
        assert_eq!(ys, vec![0, 0, 0, 1]);
        for (b, r) in buttons.iter().zip(&l.rects) {
            assert_eq!(r.unwrap().width, b.width());
        }
    }

    #[test]
    fn width_is_the_row_that_folds_nothing() {
        let t = crate::theme::Theme::junie();
        let row = ButtonRow::new();
        let buttons = drawer_actions();
        let w = row.width(&buttons, &t);
        let at = |w: u16| row.layout(Rect::new(0, 0, w, 1), &buttons, &t).hidden.len();
        assert_eq!(at(w), 0);
        assert!(at(w - 1) > 0);
    }

    #[test]
    fn a_row_that_fits_uses_the_theme_gap_and_alignment() {
        let mut t = Theme::junie();
        let buttons = drawer_actions();
        let area = Rect::new(0, 0, 80, 1);
        let l = ButtonRow::new()
            .align(RowAlign::End)
            .layout(area, &buttons, &t);
        assert_eq!(l.rects[3].unwrap().right(), 80);
        assert_eq!(l.rects[1].unwrap().x, l.rects[0].unwrap().right() + 2);
        t.spacing.button_gap = 3;
        let l = ButtonRow::new().layout(area, &buttons, &t);
        assert_eq!(l.rects[1].unwrap().x, l.rects[0].unwrap().right() + 3);
    }

    #[test]
    fn folded_buttons_are_neither_drawn_nor_clickable() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buttons = drawer_actions();
        let area = Rect::new(0, 0, 30, 1);
        let mut buf = Buffer::empty(area);
        let owner = WidgetId::of("row");
        let l = ButtonRow::new().render(owner, area, &mut buttons, &mut buf, &mut ctx, t.canvas);
        assert_eq!(buttons[1].area, Rect::ZERO);
        let text: String = (0..30).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(
            text.contains("Remove…") && !text.contains("Validate"),
            "{text}"
        );
        let more = l.more.unwrap();
        assert_eq!(
            hits.hit(ratatui::layout::Position::new(more.x + 1, 0)),
            Some(ButtonRow::more_id(owner))
        );
    }

    #[test]
    fn a_busy_primary_button_keeps_its_label_and_spinner_visible() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
        let mut b = Button::primary(WidgetId::of("b"), "Connect");
        b.busy = true;
        let w = b.width();
        b.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, t.canvas);
        for x in 1..w - 1 {
            let c = &buf[(x, 0)];
            if c.symbol() != " " {
                assert_ne!(c.fg, c.bg, "cell {x} {:?} is invisible", c.symbol());
            }
        }
    }

    #[test]
    fn the_focus_bar_reads_on_a_focused_button_at_every_level() {
        use crate::theme::{ColorLevel, contrast_ratio};
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            for kind in ["secondary", "subtle", "danger", "primary"] {
                let id = WidgetId::of("b");
                let mut b = match kind {
                    "secondary" => Button::secondary(id, "Save"),
                    "subtle" => Button::subtle(id, "Save"),
                    "danger" => Button::danger(id, "Save"),
                    _ => Button::primary(id, "Save"),
                };
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let i = Interaction {
                    focus: Some(id),
                    ..Default::default()
                };
                let mut ctx = RenderCtx::new(&t, i, &mut hits, &mut ring);
                let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
                let w = b.width();
                b.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, t.canvas);
                let bar = (0..w)
                    .map(|x| &buf[(x, 0)])
                    .find(|c| c.symbol() == t.glyphs.focus_bar)
                    .unwrap_or_else(|| panic!("{level:?} {kind}: no focus bar"));
                let r = contrast_ratio(bar.fg, bar.bg).unwrap();
                assert!(r >= 3.0, "{level:?} {kind}: focus bar {r:.2}");
            }
        }
    }

    #[test]
    fn keyboard_only_focus_hides_the_ring_after_a_click() {
        use crate::ui::ctx::FocusVisible;
        let t = Theme::junie();
        let id = WidgetId::of("b");
        let draw = |mode: FocusVisible, pointer_focus: bool| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let i = Interaction {
                focus: Some(id),
                pointer_focus,
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&t, i, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
            let mut b = Button::secondary(id, "Save").focus_visible(mode);
            let w = b.width();
            b.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, t.canvas);
            buf
        };
        let rest = {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 20, 1));
            let mut b = Button::secondary(id, "Save");
            let w = b.width();
            b.render(Rect::new(0, 0, w, 1), &mut buf, &mut ctx, t.canvas);
            buf
        };
        let keyboard = draw(FocusVisible::KeyboardOnly, false);
        assert_ne!(keyboard, rest, "keyboard focus shows");
        assert_eq!(
            draw(FocusVisible::KeyboardOnly, true),
            rest,
            "a click shows no ring"
        );
        assert_ne!(draw(FocusVisible::Always, true), rest, "Always keeps it");
    }

    #[test]
    fn the_focused_hint_names_the_button() {
        let b = Button::subtle(WidgetId::of("b"), "Cancel");
        let h = b.hint();
        assert_eq!((h.key, h.action.as_ref()), ("Enter", "Cancel"));
    }
}
