//! Stage rail: an ordered list of steps with a lifecycle state each. The
//! rail knows nothing about what the steps mean; it draws the frontier,
//! the completed count and one row per step, optionally as a focus stop
//! with a cursor for detail selection.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::NavKeys;
use crate::core::list::{Cursor, Navigable, RowSource};
use crate::core::scroll::{ScrollPane, Scrollable};
use crate::ui::ctx::{RenderCtx, VisualState};
use crate::ui::list_core::ListCore;
use crate::ui::row::{RowLook, RowUi};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum StepState {
    #[default]
    Queued,
    Running,
    Done,
    Skipped,
    Failed,
    Blocked,
}

impl StepState {
    pub fn label(self) -> &'static str {
        match self {
            StepState::Queued => "queued",
            StepState::Running => "running",
            StepState::Done => "done",
            StepState::Skipped => "skipped",
            StepState::Failed => "failed",
            StepState::Blocked => "blocked",
        }
    }

    pub fn terminal(self) -> bool {
        matches!(
            self,
            StepState::Done | StepState::Skipped | StepState::Failed
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub label: String,
    pub state: StepState,
    /// Right-aligned meta: activity text or duration.
    pub meta: Option<String>,
}

impl Step {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            state: StepState::Queued,
            meta: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StepRail {
    pub id: WidgetId,
    pub steps: Vec<Step>,
    pub selectable: bool,
    /// The shared row model ([`ListCore`]): `cursor`, `nav`, `scroll`,
    /// `content` and `empty`. The rail dereferences to it.
    pub core: ListCore,
    pub area: Rect,
    /// Show the ordinal (`03`) before the label.
    pub numbered: bool,
}

impl std::ops::Deref for StepRail {
    type Target = ListCore;
    fn deref(&self) -> &ListCore {
        &self.core
    }
}

impl std::ops::DerefMut for StepRail {
    fn deref_mut(&mut self) -> &mut ListCore {
        &mut self.core
    }
}

impl StepRail {
    /// See [`Self::nav`].
    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.core.nav = nav;
        self
    }

    pub fn new(id: WidgetId, steps: Vec<Step>) -> Self {
        let n = steps.len();
        Self {
            id,
            steps,
            selectable: false,
            core: ListCore::new(n),
            area: Rect::ZERO,
            numbered: true,
        }
    }

    /// See [`ListCore::content`]: loading or a load error replaces the
    /// steps.
    pub fn content(mut self, content: crate::ui::content::ContentState) -> Self {
        self.core.content = content;
        self
    }

    /// Override the app-wide vertical track-click policy
    /// (`TrackClick::Page` pages toward the pointer).
    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.core.scroll.track_click = Some(click);
        self
    }

    pub fn selectable(mut self, on: bool) -> Self {
        self.selectable = on;
        self
    }

    pub fn set_state(&mut self, i: usize, state: StepState) {
        if let Some(s) = self.steps.get_mut(i) {
            s.state = state;
        }
    }

    pub fn set_meta(&mut self, i: usize, meta: Option<String>) {
        if let Some(s) = self.steps.get_mut(i) {
            s.meta = meta;
        }
    }

    /// First step that is not finished: the frontier.
    pub fn frontier(&self) -> Option<usize> {
        self.steps.iter().position(|s| !s.state.terminal())
    }

    /// (done, skipped, failed)
    pub fn counts(&self) -> (usize, usize, usize) {
        let c = |st| self.steps.iter().filter(|s| s.state == st).count();
        (
            c(StepState::Done),
            c(StepState::Skipped),
            c(StepState::Failed),
        )
    }

    pub fn failed(&self) -> Option<usize> {
        self.steps.iter().position(|s| s.state == StepState::Failed)
    }

    /// The right-aligned text of a step's row: its meta, or the word of a
    /// state that has none (`queued`, `skipped`, `blocked`).
    fn meta_of(step: &Step) -> Option<String> {
        match (&step.meta, step.state) {
            (Some(m), _) => Some(m.clone()),
            (None, StepState::Queued) => Some("queued".into()),
            (None, StepState::Skipped) => Some("skipped".into()),
            (None, StepState::Blocked) => Some("blocked".into()),
            _ => None,
        }
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::row_part(i))
    }

    fn row_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    fn set_cursor(&mut self, i: usize) {
        self.core.cursor = i.min(self.steps.len().saturating_sub(1));
        self.core.scroll.ensure_visible(self.core.cursor);
    }

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if !self.selectable || self.steps.is_empty() {
            return Outcome::Ignored;
        }
        if !key.plain() {
            return Outcome::Ignored;
        }
        let Some(nav) = self.core.nav.action(key) else {
            return Outcome::Ignored;
        };
        match self
            .cursor()
            .apply(nav, self, self.core.scroll.viewport_len)
            .index()
        {
            Some(i) => {
                self.set_cursor(i);
                Outcome::Changed
            }
            None => Outcome::Ignored,
        }
    }

    pub fn on_click(&mut self, row: usize) -> Outcome {
        if !self.selectable || row >= self.steps.len() {
            return Outcome::Consumed;
        }
        self.core.scroll.pointer_move();
        self.set_cursor(row);
        Outcome::Changed
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.core.scroll.begin(ctx.track_click);
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = self.selectable && ctx.interaction.focused(self.id);
        self.core.scroll.set_content(self.steps.len());
        self.core.scroll.set_viewport(area.height as usize);
        let n = self.steps.len();
        // only a selectable rail with steps to move through is a focus
        // stop (K37)
        if self.selectable && self.core.is_focus_stop(n, focused) {
            ctx.control(self.id, area, false);
        }
        ctx.scrollable(self.id, area);
        if (!self.core.content.is_ready() || self.core.empty.is_some())
            && self.core.draw_content(n, area, buf, ctx, bg, None)
        {
            return;
        }
        let has_sb = self.core.scroll.overflows();
        let row_w = area.width.saturating_sub(u16::from(has_sb));
        let frontier = self.frontier();
        for (k, i) in self.core.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            let step = &self.steps[i];
            let rid = self.row_id(i);
            // a read-only step list is inert: no hover, press or focus
            let mut s = if self.selectable {
                ctx.child_state(self.id, rid, i == self.core.cursor)
            } else {
                VisualState::default()
            };
            s.focused = focused && i == self.core.cursor;
            let row = Rect::new(area.x, y, row_w, 1);
            let st = RowUi::paint(ctx, buf, row, s, RowLook::Bar, bg).style();
            // the state glyph is a graphic on the row's plane (hover and
            // press lift it): at 3:1 there at every colour level
            let plane = st.bg.unwrap_or(bg);
            let mark = |ink| st.fg(t.legible_mark(ink, plane));
            let (glyph, gstyle) = match step.state {
                StepState::Queued | StepState::Skipped | StepState::Blocked => (" ", st),
                StepState::Running => {
                    (t.glyphs.spinner_frame(ctx.interaction.tick), mark(t.accent))
                }
                StepState::Done => (t.glyphs.done, mark(t.success)),
                StepState::Failed => ("!", mark(t.error).add_modifier(Modifier::BOLD)),
            };
            buf.set_string(row.x + 1, y, glyph, gstyle);
            let mut x = row.x + 3;
            if self.numbered {
                let num = format!("{:02}", i + 1);
                buf.set_string(
                    x,
                    y,
                    &num,
                    st.fg(if step.state == StepState::Running {
                        t.text_secondary
                    } else {
                        t.text_faint
                    })
                    .remove_modifier(Modifier::BOLD),
                );
                x += 3;
            }
            let label_style = match step.state {
                StepState::Queued => st.fg(t.text_muted),
                StepState::Running => st.fg(t.text_primary).add_modifier(Modifier::BOLD),
                StepState::Done => st.fg(t.text_secondary),
                StepState::Skipped => st.fg(t.text_faint),
                StepState::Failed => st.fg(t.error).add_modifier(Modifier::BOLD),
                StepState::Blocked => st.fg(t.text_secondary),
            };
            let label_style = if s.focused && !s.pressed {
                label_style.add_modifier(Modifier::BOLD)
            } else {
                label_style
            };
            let meta = Self::meta_of(step);
            let meta_style = match step.state {
                StepState::Running => st.fg(t.text_secondary),
                StepState::Failed => st.fg(t.error),
                StepState::Blocked => st.fg(t.warning),
                _ => st.fg(t.text_faint),
            }
            .remove_modifier(Modifier::BOLD);
            let avail = row.right().saturating_sub(x + 1) as usize;
            let mw = meta
                .as_ref()
                .map(|m| crate::ui::text::width(m))
                .unwrap_or(0);
            let show_meta = mw > 0 && avail >= mw + 12;
            let lw = if show_meta { avail - mw - 2 } else { avail };
            // a cut label reveals its full text on hover (and on the peek
            // line for the focused step)
            let cell = Rect::new(x, y, lw as u16, 1);
            let label = crate::ui::reveal::fit(ctx, cell, &step.label, s.focused);
            buf.set_string(x, y, label, label_style);
            if show_meta && let Some(m) = &meta {
                buf.set_string(row.right().saturating_sub(mw as u16 + 1), y, m, meta_style);
            }
            if self.selectable {
                ctx.register_part(self.id, Self::row_part(i), row);
            }
            let _ = frontier;
        }
        if has_sb {
            crate::ui::fade::scroll_edges_except(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    area.y,
                    (area.right() - 1).saturating_sub(area.x),
                    area.height,
                ),
                &self.core.scroll,
                &self
                    .scroll
                    .visible_range()
                    .position(|i| i == self.core.cursor)
                    .map(|k| area.y + k as u16)
                    .into_iter()
                    .collect::<Vec<_>>(),
            );
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            self.core.scroll.draw_v(sb, buf, ctx, self.id, focused);
        }
        let _ = Style::new();
    }
}

/// The narrowest the rail shows every step's label and meta whole, one
/// row per step (plan §1.4 `Measure`): the state glyph and the ordinal,
/// the label, and the meta after its gap (a meta shows only while the
/// label keeps 10 cells beside it). An empty rail measures its empty
/// state.
impl crate::ui::measure::Measure for StepRail {
    fn measure(&self, avail: ratatui::layout::Size, ctx: &RenderCtx) -> ratatui::layout::Size {
        if self.steps.is_empty() {
            return match &self.core.empty {
                Some(e) => e.measure(avail, ctx),
                None => ratatui::layout::Size::new(0, 0),
            };
        }
        let lead = 3 + if self.numbered { 3 } else { 0 };
        let width = self
            .steps
            .iter()
            .map(|step| {
                let label = crate::ui::text::width(&step.label) as u16;
                let text = match Self::meta_of(step) {
                    // `render`: the meta shows while `avail >= mw + 12`,
                    // the label gets `avail - mw - 2`
                    Some(m) => {
                        let mw = crate::ui::text::width(&m) as u16;
                        (label + 2 + mw).max(mw + 12)
                    }
                    None => label,
                };
                lead + text + 1
            })
            .max()
            .unwrap_or(0);
        ratatui::layout::Size::new(width, self.steps.len() as u16)
    }

    /// The narrowest the rail keeps every step's meta: a label then cuts
    /// with `…` and reveals its full text on hover (and on the peek line
    /// for the focused step), but no state word or duration is dropped.
    fn min_width(&self, avail: ratatui::layout::Size, ctx: &RenderCtx) -> u16 {
        if self.steps.is_empty() {
            return self.measure(avail, ctx).width;
        }
        let lead = 3 + if self.numbered { 3 } else { 0 };
        let text = self
            .steps
            .iter()
            .map(|step| match Self::meta_of(step) {
                Some(m) => crate::ui::text::width(&m) as u16 + 12,
                None => (crate::ui::text::width(&step.label) as u16).min(12),
            })
            .max()
            .unwrap_or(0);
        lead + text + 1
    }
}

impl Scrollable for StepRail {
    fn pane(&mut self) -> &mut ScrollPane {
        &mut self.core.scroll
    }
}

impl RowSource for StepRail {
    fn len(&self) -> usize {
        self.steps.len()
    }
    fn label(&self, i: usize) -> &str {
        &self.steps[i].label
    }
}

impl Navigable for StepRail {
    fn cursor(&self) -> Cursor {
        self.core.cursor_model()
    }
}

/// The rail's pointer gestures: a wheel or a bar press scrolls it; a
/// completed click on a step is [`StepRail::on_click`].
impl crate::core::pointer::MouseTarget for StepRail {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<()> {
        use crate::core::intent::Phase;
        if let Some(r) = crate::core::pointer::scroll_input(self, ev) {
            return r;
        }
        match ev.index_of(Part::ROW) {
            Some(row) if ev.is(Phase::Click) => self.on_click(row).into(),
            _ => crate::core::response::Response::ignored(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontier_and_counts() {
        let mut r = StepRail::new(
            WidgetId::of("r"),
            ["a", "b", "c", "d"].iter().map(|s| Step::new(s)).collect(),
        );
        r.set_state(0, StepState::Done);
        r.set_state(1, StepState::Skipped);
        r.set_state(2, StepState::Running);
        assert_eq!(r.frontier(), Some(2));
        assert_eq!(r.counts(), (1, 1, 0));
        r.set_state(2, StepState::Failed);
        assert_eq!(r.failed(), Some(2));
        assert_eq!(r.frontier(), Some(3));
    }

    #[test]
    fn at_its_measured_size_every_label_and_meta_shows_whole() {
        use crate::core::{focus::FocusRing, hit::HitRegistry};
        use crate::theme::Theme;
        use crate::ui::ctx::Interaction;
        use crate::ui::measure::Measure;
        let mut r = StepRail::new(
            WidgetId::of("r"),
            [
                "Resolve workspace",
                "Pull base image",
                "Resolve credentials",
            ]
            .iter()
            .map(|s| Step::new(s))
            .collect(),
        );
        r.set_state(0, StepState::Done);
        r.set_meta(0, Some("1.2s".into()));
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let size = r.measure(ratatui::layout::Size::new(0, 0), &ctx);
        drop(ctx);
        assert_eq!(size.height, 3);
        let text = |r: &mut StepRail, w: u16| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, w, 3));
            r.render(Rect::new(0, 0, w, 3), &mut buf, &mut ctx, theme.canvas);
            (0..3)
                .map(|y| (0..w).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let whole = text(&mut r, size.width);
        for word in [
            "Resolve workspace",
            "1.2s",
            "Pull base image",
            "Resolve credentials",
            "queued",
        ] {
            assert!(
                whole.contains(word),
                "{word} whole at {}: {whole}",
                size.width
            );
        }
        // one cell narrower, a label or a meta gives way
        let less = text(&mut r, size.width - 1);
        assert!(
            !less.contains("Resolve credentials") || less.matches("queued").count() < 2,
            "{less}"
        );
        // at its min width every meta stays; labels cut with `…`
        let ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let min = r.min_width(ratatui::layout::Size::new(0, 0), &ctx);
        drop(ctx);
        assert!(min < size.width);
        let narrow = text(&mut r, min);
        assert_eq!(narrow.matches("queued").count(), 2, "{narrow}");
        assert!(narrow.contains("1.2s"), "{narrow}");
        assert!(narrow.contains('…'), "{narrow}");
    }

    #[test]
    fn a_cut_label_reveals_its_full_text_on_hover() {
        use crate::core::{focus::FocusRing, hit::HitRegistry};
        use crate::theme::Theme;
        use crate::ui::ctx::Interaction;
        use ratatui::layout::Position;
        let mut r = StepRail::new(
            WidgetId::of("r"),
            vec![Step::new("Resolve credentials for the workspace")],
        );
        let theme = Theme::junie();
        let draw = |r: &mut StepRail, hover: Option<WidgetId>| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let interaction = Interaction {
                hover,
                ..Default::default()
            };
            let mut buf = Buffer::empty(Rect::new(0, 0, 70, 3));
            {
                let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
                r.render(Rect::new(0, 0, 30, 1), &mut buf, &mut ctx, theme.canvas);
                ctx.flush_overlays(&mut buf);
            }
            let text = (0..3)
                .map(|y| (0..70).map(|x| buf[(x, y)].symbol()).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            (text, hits)
        };
        let (text, hits) = draw(&mut r, None);
        assert!(text.contains('…'), "the label is cut: {text}");
        let id = hits
            .hover_at(Position::new(8, 0))
            .expect("a hover region on the cut label");
        let (text, _) = draw(&mut r, Some(id));
        assert!(
            text.contains("Resolve credentials for the workspace"),
            "hover shows the full label: {text}"
        );
    }

    #[test]
    fn a_read_only_step_rail_does_not_react_to_hover_or_press() {
        use crate::core::{focus::FocusRing, hit::HitRegistry};
        use crate::theme::Theme;
        use crate::ui::ctx::Interaction;
        let mut r = StepRail::new(
            WidgetId::of("r"),
            ["a", "b"].iter().map(|s| Step::new(s)).collect(),
        );
        let theme = Theme::junie();
        let rid = r.row_id(0);
        let render = |r: &mut StepRail, interaction: Interaction| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 20, 2));
            r.render(Rect::new(0, 0, 20, 2), &mut buf, &mut ctx, theme.canvas);
            buf
        };
        let rest = render(&mut r, Interaction::default());
        let pressed = render(
            &mut r,
            Interaction {
                hover: Some(rid),
                pressed: Some(rid),
                ..Default::default()
            },
        );
        assert_eq!(rest, pressed);
    }

    /// A1: a step's state glyph reads on its row (3:1) in every state at
    /// every level; at 16 colours a hovered row is xterm's DarkGray, where
    /// the LightGreen spinner was 2.9:1 and the Green check 1.9:1.
    #[test]
    fn a_steps_state_glyph_reads_on_its_row_at_every_level() {
        use crate::core::{focus::FocusRing, hit::HitRegistry};
        use crate::theme::{ColorLevel, Theme, contrast_ratio};
        use crate::ui::ctx::Interaction;
        for level in ColorLevel::ALL {
            let theme = Theme::for_level(level);
            let mut r = StepRail::new(
                WidgetId::of("r"),
                ["a", "b", "c"].iter().map(|s| Step::new(s)).collect(),
            )
            .selectable(true);
            r.set_state(0, StepState::Done);
            r.set_state(1, StepState::Failed);
            r.set_state(2, StepState::Running);
            for i in 0..3 {
                let rid = r.row_id(i);
                for pressed in [None, Some(rid)] {
                    let mut hits = HitRegistry::default();
                    let mut ring = FocusRing::default();
                    let inter = Interaction {
                        hover: Some(rid),
                        pressed,
                        ..Default::default()
                    };
                    let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
                    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
                    r.render(Rect::new(0, 0, 20, 3), &mut buf, &mut ctx, theme.canvas);
                    let c = &buf[(1, i as u16)];
                    let ratio = contrast_ratio(c.fg, c.bg).unwrap();
                    assert!(
                        ratio >= 3.0,
                        "{level:?} row {i} pressed {}: {:?} {ratio:.2}",
                        pressed.is_some(),
                        c.symbol()
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod track_click_tests {
    use super::*;
    use crate::core::scroll::TrackClick;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Position;

    fn pressed_at_the_bottom(click: Option<TrackClick>, app: TrackClick) -> usize {
        let names: Vec<String> = (0..50).map(|i| format!("step {i}")).collect();
        let mut r = StepRail::new(
            WidgetId::of("r"),
            names.iter().map(|s| Step::new(s)).collect(),
        );
        r.scroll.track_click = click;
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        ctx.track_click = app;
        r.render(area, &mut buf, &mut ctx, theme.surface);
        let bottom = Position::new(r.scroll.v_track().x, r.scroll.v_track().bottom() - 1);
        assert_eq!(
            r.bar_press(bottom).outcome().or(Outcome::Consumed),
            Outcome::Changed
        );
        r.scroll.offset
    }

    /// The vertical bar follows the track-click policy (it always jumped).
    #[test]
    fn a_step_rail_track_click_can_page() {
        let page = pressed_at_the_bottom(Some(TrackClick::Page), TrackClick::Jump);
        assert!(page > 0 && page <= 10, "one page: {page}");
        assert_eq!(
            pressed_at_the_bottom(None, TrackClick::Page),
            page,
            "app-wide"
        );
        assert!(
            pressed_at_the_bottom(None, TrackClick::Jump) > page,
            "default jumps"
        );
    }
}
