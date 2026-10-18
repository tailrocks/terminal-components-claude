use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::ui::ctx::RenderCtx;
use crate::widgets::action_row::ActionRow;
use crate::widgets::button::Button;
use crate::widgets::empty::{self, EmptyState};
use crate::widgets::panel::Panel;
use crate::widgets::props::{self, Prop};

#[derive(Debug, Clone)]
pub struct DetailCard {
    pub title: String,

    pub meta: Option<String>,

    pub props: Vec<Prop>,

    pub actions: ActionRow,

    pub max_width: Option<u16>,

    pub max_height: Option<u16>,

    pub reserve: u16,

    pub empty: Option<EmptyState>,

    pub focused: bool,

    pub fit: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DetailLayout {
    pub card: Rect,

    pub bg: Color,

    pub props_h: u16,

    pub slot: Rect,

    pub actions: Vec<Rect>,
}

impl DetailCard {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            meta: None,
            props: Vec::new(),
            actions: ActionRow::new(),
            max_width: None,
            max_height: None,
            reserve: 3,
            empty: None,
            focused: false,
            fit: false,
        }
    }

    pub fn fit(mut self, on: bool) -> Self {
        self.fit = on;
        self
    }

    pub fn empty(state: EmptyState) -> Self {
        Self {
            empty: Some(state),
            ..Self::new("")
        }
    }

    pub fn meta(mut self, meta: &str) -> Self {
        self.meta = Some(meta.to_owned());
        self
    }

    pub fn props(mut self, props: Vec<Prop>) -> Self {
        self.props = props;
        self
    }

    pub fn actions(mut self, row: ActionRow) -> Self {
        self.actions = row;
        self
    }

    pub fn max_size(mut self, width: Option<u16>, height: Option<u16>) -> Self {
        self.max_width = width;
        self.max_height = height;
        self
    }

    pub fn reserve(mut self, rows: u16) -> Self {
        self.reserve = rows;
        self
    }

    pub fn focused(mut self, on: bool) -> Self {
        self.focused = on;
        self
    }

    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        buttons: &mut [&mut Button],
    ) -> DetailLayout {
        let t = ctx.theme;
        if let Some(e) = &self.empty {
            empty::render(area, buf, t, e, bg);
            return DetailLayout {
                bg,
                ..DetailLayout::default()
            };
        }
        let mut card = Rect::new(
            area.x,
            area.y,
            self.max_width.map_or(area.width, |w| area.width.min(w)),
            self.max_height.map_or(area.height, |h| area.height.min(h)),
        );
        let mut panel = Panel::card(Some(&self.title)).focused(self.focused);
        if let Some(m) = &self.meta {
            panel = panel.meta(m);
        }
        if self.fit {
            let mut scratch = Buffer::empty(card);
            let probe = panel.render(card, &mut scratch, t);
            let chrome = card.height.saturating_sub(probe.height);
            let need = chrome + props::measure(&self.props, probe.width) + self.reserve;
            card.height = card.height.min(need);
        }
        let cbg = panel.bg(t);
        let inner = panel.render(card, buf, t);
        let props_h = props::render(
            Rect::new(
                inner.x,
                inner.y,
                inner.width,
                inner.height.saturating_sub(self.reserve),
            ),
            buf,
            t,
            &self.props,
            cbg,
        );
        let actions_y = inner.bottom().saturating_sub(1);
        let slot_y = (inner.y + props_h + 1).min(actions_y);
        let slot = Rect::new(inner.x, slot_y, inner.width, actions_y - slot_y);
        let actions = if buttons.is_empty() || inner.height == 0 {
            Vec::new()
        } else {
            self.actions.render(
                Rect::new(inner.x, actions_y, inner.width, 1),
                buf,
                ctx,
                cbg,
                buttons,
            )
        };
        DetailLayout {
            card,
            bg: cbg,
            props_h,
            slot,
            actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::id::WidgetId;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect()
    }

    #[test]
    fn facts_slot_and_actions_take_their_rows() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, 60, 12);
        let mut buf = Buffer::empty(area);
        let mut connect = Button::primary(WidgetId::of("c"), "Connect");
        let mut edit = Button::secondary(WidgetId::of("e"), "Edit…");
        let l = DetailCard::new("prod-eu")
            .meta("postgres")
            .props(vec![
                Prop::new("Engine", "PostgreSQL"),
                Prop::new("Host", "db"),
            ])
            .max_size(Some(40), None)
            .render(
                area,
                &mut buf,
                &mut ctx,
                t.canvas,
                &mut [&mut connect, &mut edit],
            );
        assert_eq!(l.card.width, 40, "capped at max_width");
        assert_eq!(l.props_h, 2);
        let text: String = (0..12).map(|y| row(&buf, y) + "\n").collect();
        assert!(
            text.contains("prod-eu") && text.contains("postgres"),
            "{text}"
        );
        assert!(
            text.contains("Engine") && text.contains("PostgreSQL"),
            "{text}"
        );
        assert_eq!(
            l.slot.y,
            l.card.y + 1 + 2 + 1 + 1,
            "under the facts and a blank row"
        );
        assert_eq!(l.actions.len(), 2);
        assert_eq!(
            l.actions[0].y,
            l.slot.bottom(),
            "actions on the card's last row"
        );
        assert!(row(&buf, l.actions[0].y).contains("Connect"));
        assert_eq!(
            hits.hit(ratatui::layout::Position::new(
                l.actions[1].x + 1,
                l.actions[1].y
            )),
            Some(edit.id)
        );
    }

    #[test]
    fn a_fitted_card_ends_under_its_facts() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, 40, 30);
        let mut buf = Buffer::empty(area);
        let mut ok = Button::primary(WidgetId::of("ok"), "Open");
        let card = DetailCard::new("orders").props(vec![Prop::new("Rows", "12")]);
        let tall = card
            .clone()
            .render(area, &mut buf, &mut ctx, t.canvas, &mut [&mut ok]);
        assert_eq!(tall.card.height, 30);
        let fitted = card
            .fit(true)
            .render(area, &mut buf, &mut ctx, t.canvas, &mut [&mut ok]);
        assert_eq!(fitted.props_h, 1);
        assert_eq!(fitted.actions[0].y, fitted.slot.bottom());
        assert!(fitted.card.height < 10, "{:?}", fitted.card);
        assert_eq!(
            fitted.card.bottom(),
            fitted.actions[0].y + 2,
            "the action row ends the card"
        );
    }

    #[test]
    fn nothing_selected_shows_the_empty_state_instead() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, 40, 8);
        let mut buf = Buffer::empty(area);
        let l = DetailCard::empty(EmptyState::new("Select a connection")).render(
            area,
            &mut buf,
            &mut ctx,
            t.canvas,
            &mut [],
        );
        assert_eq!(l.card, Rect::ZERO);
        let text: String = (0..8).map(|y| row(&buf, y)).collect();
        assert!(text.contains("Select a connection"));
    }
}
