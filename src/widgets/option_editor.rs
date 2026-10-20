use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::focus::BlurAction;
use crate::core::hit::Axis;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{Nav, NavKeys, unassigned_chord};
use crate::core::list::{Cursor, KeyStep, Moved, RowSource};
use crate::core::selection::{RowSelection, SelectGesture};
use crate::theme::{CursorStyle, Tone};
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Dismiss, Place, Placement, backdrop_id, place_with, surface_for};
use crate::ui::text::{truncate, width};
use crate::widgets::edit::EditField;
use crate::widgets::option_list::{OptionList, WheelMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionItem {
    pub value: String,

    pub label: String,

    pub tone: Option<Tone>,

    pub description: Option<String>,

    pub disabled: Option<String>,
}

impl OptionItem {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_owned(),
            label: value.to_owned(),
            tone: None,
            description: None,
            disabled: None,
        }
    }
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_owned();
        self
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
    pub fn description(mut self, text: &str) -> Self {
        self.description = Some(text.to_owned());
        self
    }
    pub fn disabled(mut self, reason: &str) -> Self {
        self.disabled = Some(reason.to_owned());
        self
    }
    pub fn is_disabled(&self) -> bool {
        self.disabled.is_some()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options(pub Vec<OptionItem>);

impl Options {
    pub fn new(items: impl IntoIterator<Item = OptionItem>) -> Self {
        Self(items.into_iter().collect())
    }

    pub fn values(values: &[&str]) -> Self {
        Self(values.iter().map(|v| OptionItem::new(v)).collect())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn get(&self, i: usize) -> Option<&OptionItem> {
        self.0.get(i)
    }

    pub fn find(&self, value: &str) -> Option<usize> {
        self.0.iter().position(|o| o.value == value)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ColumnEditor {
    #[default]
    Text,

    Select(Options),

    Combo(Options),

    Radio(Options),

    Checkbox(Options),

    Segmented(Options),

    Tags(Options),
}

impl ColumnEditor {
    pub fn options(&self) -> Option<&Options> {
        match self {
            Self::Text => None,
            Self::Select(o)
            | Self::Combo(o)
            | Self::Radio(o)
            | Self::Checkbox(o)
            | Self::Segmented(o)
            | Self::Tags(o) => Some(o),
        }
    }

    pub fn is_constrained(&self) -> bool {
        self.options().is_some()
    }

    pub fn has_popup(&self) -> bool {
        self.is_constrained() && !matches!(self, Self::Checkbox(_))
    }

    pub fn for_options(options: Options) -> Self {
        match options.len() {
            0..=2 => Self::Checkbox(options),
            3..=5 => Self::Segmented(options),
            6..=15 => Self::Select(options),
            _ => Self::Combo(options),
        }
    }

    pub fn checked(&self, raw: &str) -> Option<bool> {
        let Self::Checkbox(opts) = self else {
            return None;
        };
        match opts.find(raw)? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    fn parts<'a>(&self, raw: &'a str, cfg: &EditorConfig) -> Vec<&'a str> {
        match self {
            Self::Tags(_) => raw
                .split(cfg.tag_separator.trim())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect(),
            _ => vec![raw],
        }
    }

    pub fn display(&self, raw: &str, cfg: &EditorConfig) -> String {
        let Some(opts) = self.options() else {
            return raw.to_owned();
        };
        let labels: Vec<&str> = self
            .parts(raw, cfg)
            .into_iter()
            .map(|p| opts.find(p).map_or(p, |i| opts.0[i].label.as_str()))
            .collect();
        labels.join(cfg.tag_separator)
    }

    pub fn out_of_set(&self, raw: &str, cfg: &EditorConfig) -> bool {
        let Some(opts) = self.options() else {
            return false;
        };
        if raw.trim().is_empty() {
            return false;
        }
        self.parts(raw, cfg).iter().any(|p| opts.find(p).is_none())
    }

    pub fn tone(&self, raw: &str) -> Option<Tone> {
        let opts = self.options()?;
        if matches!(self, Self::Tags(_)) {
            return None;
        }
        opts.find(raw).and_then(|i| opts.0[i].tone)
    }

    pub fn toggled(&self, raw: &str) -> Option<String> {
        let Self::Checkbox(opts) = self else {
            return None;
        };
        let enabled: Vec<&OptionItem> = opts.0.iter().filter(|o| !o.is_disabled()).collect();
        match opts.find(raw) {
            Some(i) => enabled
                .iter()
                .find(|o| o.value != opts.0[i].value)
                .map(|o| o.value.clone()),
            None => enabled.first().map(|o| o.value.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorConfig {
    pub dismiss: Dismiss,

    pub blur: BlurAction,

    pub max_rows: u16,

    pub popup_width: (u16, u16),

    pub cursor_style: CursorStyle,

    pub type_ahead: bool,

    pub type_ahead_timeout: std::time::Duration,

    pub nav: NavKeys,

    pub affordance: Affordance,

    pub flag_out_of_set: bool,

    pub tag_separator: &'static str,

    pub placement: Placement,

    pub wheel: WheelMode,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            dismiss: Dismiss::LIGHT,
            blur: BlurAction::Cancel,
            max_rows: 8,
            popup_width: (14, 48),
            cursor_style: CursorStyle::Fill,
            type_ahead: true,
            type_ahead_timeout: crate::core::typeahead::TypeAhead::DEFAULT_TIMEOUT,
            nav: NavKeys::default(),
            affordance: Affordance::Focused,
            flag_out_of_set: true,
            tag_separator: ", ",
            placement: Placement::Below,
            wheel: WheelMode::Viewport,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Affordance {
    #[default]
    Focused,

    Always,

    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceResult {
    Commit(String),

    Cancel,

    CommitAndTab { value: String, backward: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Select,
    Combo,
    Radio,
    Segmented,
    Tags,
}

#[derive(Debug, Clone)]
pub struct ChoiceEditor {
    pub id: WidgetId,
    kind: Kind,
    pub options: Options,

    pub cursor: usize,

    pub current: Option<usize>,

    pub selection: RowSelection,

    pub query: EditField,

    pub error: Option<String>,

    pub list: OptionList,
    typeahead: crate::core::typeahead::TypeAhead,
}

impl ChoiceEditor {
    pub fn open(
        id: WidgetId,
        editor: &ColumnEditor,
        raw: &str,
        cfg: &EditorConfig,
    ) -> Option<Self> {
        let kind = match editor {
            ColumnEditor::Select(_) => Kind::Select,
            ColumnEditor::Combo(_) => Kind::Combo,
            ColumnEditor::Radio(_) => Kind::Radio,
            ColumnEditor::Segmented(_) => Kind::Segmented,
            ColumnEditor::Tags(_) => Kind::Tags,
            ColumnEditor::Text | ColumnEditor::Checkbox(_) => return None,
        };
        let options = editor.options()?.clone();
        let current = options.find(raw);
        let checked = if kind == Kind::Tags {
            let mut s = RowSelection::new();
            for i in editor
                .parts(raw, cfg)
                .iter()
                .filter_map(|p| options.find(p))
            {
                s.set(i, true);
            }
            s
        } else {
            RowSelection::new()
        };
        let first_enabled = options.0.iter().position(|o| !o.is_disabled()).unwrap_or(0);
        let cursor = current
            .filter(|&i| !options.0[i].is_disabled())
            .or_else(|| checked.iter().next())
            .unwrap_or(first_enabled);
        Some(Self {
            id,
            kind,
            options,
            cursor,
            current,
            selection: checked,
            query: EditField::single(""),
            error: None,
            list: Self::list(id, cfg),
            typeahead: crate::core::typeahead::TypeAhead::with_timeout(cfg.type_ahead_timeout),
        })
    }

    fn list(id: WidgetId, cfg: &EditorConfig) -> OptionList {
        let mut list = OptionList::new(id)
            .dismiss(cfg.dismiss)
            .placement(cfg.placement)
            .max_rows(cfg.max_rows)
            .width(cfg.popup_width.0, cfg.popup_width.1)
            .look(cfg.cursor_style)
            .nav(cfg.nav)
            .type_ahead(cfg.type_ahead)
            .wheel_mode(cfg.wheel);

        list.scroll.v.context = 0;
        list.scroll.v.viewport_len = cfg.max_rows as usize;
        list
    }

    pub fn option_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::option_part(i))
    }

    fn option_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    fn option_of(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::ROW, 0..self.options.len())
    }

    pub(crate) fn contains_id(&self, id: WidgetId) -> bool {
        id == self.id || id == backdrop_id(self.id) || self.option_of(id).is_some()
    }

    pub fn shown(&self) -> Vec<usize> {
        let q = self.query.text().trim().to_lowercase();
        (0..self.options.len())
            .filter(|&i| {
                q.is_empty()
                    || self.options.0[i].label.to_lowercase().contains(&q)
                    || self.options.0[i].value.to_lowercase().contains(&q)
            })
            .collect()
    }

    fn enabled(&self, i: usize) -> bool {
        self.options.get(i).is_some_and(|o| !o.is_disabled())
    }

    fn shown_rows(&self) -> Shown<'_> {
        Shown {
            ed: self,
            rows: self.shown(),
        }
    }

    fn value(&self, cfg: &EditorConfig) -> Option<String> {
        if self.kind == Kind::Tags {
            return Some(
                self.selection
                    .iter()
                    .map(|i| self.options.0[i].value.as_str())
                    .collect::<Vec<_>>()
                    .join(cfg.tag_separator),
            );
        }
        if !self.enabled(self.cursor) || !self.shown().contains(&self.cursor) {
            return None;
        }
        Some(self.options.0[self.cursor].value.clone())
    }

    pub fn blur_value(&self, cfg: &EditorConfig) -> Option<String> {
        self.value(cfg)
    }

    fn toggle_checked(&mut self, i: usize) {
        let order: Vec<usize> = (0..self.options.0.len()).collect();
        let options = &self.options;
        self.selection.apply(SelectGesture::Toggle, i, &order, |k| {
            !options.0[k].is_disabled()
        });
    }

    pub fn on_key(&mut self, key: &Key, cfg: &EditorConfig) -> (Outcome, Option<ChoiceResult>) {
        if self.kind == Kind::Combo {
            use crate::widgets::field_common::EditAction;
            let action = self.query.action(key);
            if !matches!(
                action,
                EditAction::Commit | EditAction::Cancel | EditAction::Tab { .. } | EditAction::None
            ) {
                if self.query.apply(action, key).edited {
                    self.after_query();
                }
                return (Outcome::Changed, None);
            }
        }
        if unassigned_chord::<()>(key, &[]) {
            return (Outcome::Consumed, None);
        }
        let combo = self.kind == Kind::Combo;
        match key.code {
            KeyCode::Esc if cfg.dismiss.escape => {
                return (Outcome::Changed, Some(ChoiceResult::Cancel));
            }
            KeyCode::Esc => return (Outcome::Consumed, None),
            KeyCode::Enter => {
                return match self.value(cfg) {
                    Some(v) => (Outcome::Changed, Some(ChoiceResult::Commit(v))),
                    None => (Outcome::Consumed, None),
                };
            }
            KeyCode::Tab | KeyCode::BackTab => {
                let backward = key.code == KeyCode::BackTab;
                return match self.value(cfg) {
                    Some(value) => (
                        Outcome::Changed,
                        Some(ChoiceResult::CommitAndTab { value, backward }),
                    ),
                    None => (Outcome::Consumed, None),
                };
            }
            KeyCode::Char(' ') if self.kind == Kind::Tags => {
                self.toggle_checked(self.cursor);
                return (Outcome::Changed, None);
            }
            KeyCode::Char(' ') if self.kind != Kind::Combo => {
                return match self.value(cfg) {
                    Some(v) => (Outcome::Changed, Some(ChoiceResult::Commit(v))),
                    None => (Outcome::Consumed, None),
                };
            }
            _ => {}
        }
        let step = KeyStep::of(key, &cfg.nav, cfg.type_ahead && !combo);
        let mut ta = std::mem::take(&mut self.typeahead);
        let shown = self.shown_rows();
        let at = shown.rows.iter().position(|&i| i == self.cursor);

        let axis = if self.kind == Kind::Segmented {
            Axis::Horizontal
        } else {
            Axis::Vertical
        };
        let mut cursor = Cursor::new(at.unwrap_or(0)).axis(axis);
        let moved = match step {
            Some(KeyStep::Type(c)) => cursor.type_ahead(c, &mut ta, &shown),
            Some(KeyStep::Nav(nav)) => {
                let nav = match nav {
                    Nav::Up if axis == Axis::Horizontal => Nav::Left,
                    Nav::Down if axis == Axis::Horizontal => Nav::Right,
                    n => n,
                };
                match (at, nav) {
                    (None, Nav::Up | Nav::Down | Nav::Left | Nav::Right) => cursor.first(&shown),
                    _ => cursor.apply(nav, &shown, self.list.scroll.viewport_len),
                }
            }
            None => Moved::Ignored,
        };
        let to = moved.index().map(|k| shown.rows[k]);
        self.typeahead = ta;
        match (step, moved, to) {
            (Some(KeyStep::Type(_)), Moved::Stayed(_), _) => (Outcome::Consumed, None),
            (_, _, Some(i)) => {
                self.cursor = i;
                (Outcome::Changed, None)
            }
            _ => (Outcome::Consumed, None),
        }
    }

    fn after_query(&mut self) {
        self.error = None;
        if let Some(&i) = self.shown().iter().find(|&&i| self.enabled(i)) {
            self.cursor = i;
        }
        self.list.scroll.offset = 0;
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if self.kind != Kind::Combo {
            return Outcome::Consumed;
        }
        if self.query.paste(text.trim()) {
            self.after_query();
        }
        Outcome::Changed
    }

    pub fn on_click(&mut self, id: WidgetId) -> (Outcome, Option<ChoiceResult>) {
        if id == backdrop_id(self.id) {
            return if self.list.dismiss.outside_click {
                (Outcome::Changed, Some(ChoiceResult::Cancel))
            } else {
                (Outcome::Consumed, None)
            };
        }
        let Some(i) = self.option_of(id) else {
            return (Outcome::Consumed, None);
        };
        if !self.enabled(i) {
            return (Outcome::Consumed, None);
        }
        self.cursor = i;
        if self.kind == Kind::Tags {
            self.toggle_checked(i);
            return (Outcome::Changed, None);
        }
        (
            Outcome::Changed,
            Some(ChoiceResult::Commit(self.options.0[i].value.clone())),
        )
    }

    fn step(&mut self, delta: isize) {
        let shown = self.shown_rows();
        let at = shown.rows.iter().position(|&i| i == self.cursor);
        let mut cursor = Cursor::new(at.unwrap_or(0));
        let mut moved = match at {
            None => cursor.first(&shown),
            Some(k) => Moved::Stayed(k),
        };
        if at.is_some() {
            for _ in 0..delta.unsigned_abs() {
                moved = cursor.step(delta.signum(), &shown);
            }
        }
        if let Some(k) = moved.index() {
            self.cursor = shown.rows[k];
        }
    }

    pub fn wheel(&mut self, axis: Axis, delta: i32) -> crate::core::response::Response<()> {
        use crate::core::response::Response;
        use crate::core::scroll::Scrollable;
        if self.kind == Kind::Segmented || axis == Axis::Horizontal {
            return Response::consumed();
        }
        if self.list.wheel == WheelMode::Viewport {
            return self.list.wheel(axis, delta);
        }
        let before = self.cursor;
        self.step(delta as isize);
        if before == self.cursor {
            Response::consumed()
        } else {
            Response::changed()
        }
    }

    pub fn render(&mut self, cell: Rect, buf: &Buffer, ctx: &mut RenderCtx, cfg: &EditorConfig) {
        if self.kind == Kind::Segmented {
            self.render_strip(cell, buf, ctx, cfg);
        } else {
            self.render_list(cell, buf, ctx, cfg);
        }
    }

    fn prefix(&self) -> u16 {
        match self.kind {
            Kind::Radio | Kind::Tags => 6,
            _ => 3,
        }
    }

    fn render_list(&mut self, cell: Rect, buf: &Buffer, ctx: &mut RenderCtx, cfg: &EditorConfig) {
        let screen = *buf.area();
        let shown = self.shown();
        let search = u16::from(self.kind == Kind::Combo);
        let rows = (shown.len().max(1) as u16).min(cfg.max_rows.max(1));
        let content_w = self
            .options
            .0
            .iter()
            .map(|o| {
                let extra = o
                    .disabled
                    .as_deref()
                    .or(o.description.as_deref())
                    .map_or(0, |d| width(d) + 2);
                width(&o.label) + extra
            })
            .max()
            .unwrap_or(0) as u16
            + self.prefix()
            + 7;
        let (min_w, max_w) = cfg.popup_width;
        let w = content_w.max(cell.width + 2).clamp(min_w, max_w);
        let pa = place_with(
            screen,
            cell,
            w,
            rows + 2 + search,
            cfg.placement,
            Place::DEFAULT,
        );
        let visible = pa.height.saturating_sub(2 + search).max(1) as usize;
        let at = shown.iter().position(|&i| i == self.cursor).unwrap_or(0);

        self.list.scroll.set_content(shown.len());
        self.list.scroll.set_viewport(visible);
        self.list.scroll.follow(at);
        if search > 0 {
            let inner = pa
                .intersection(screen)
                .inner(ratatui::layout::Margin::new(1, 1));
            let line = Rect::new(inner.x + 3, inner.y, inner.width.saturating_sub(3), 1);
            let mut scratch = Buffer::empty(line);
            self.query
                .draw_line(&mut scratch, line, &Self::query_look(ctx.theme));
        }
        let me = self.clone();
        let cfg = cfg.clone();
        ctx.overlay(move |buf, ctx| me.draw_list(pa, &shown, search, &cfg, buf, ctx));
    }

    fn draw_list(
        &self,
        pa: Rect,
        shown: &[usize],
        search: u16,
        cfg: &EditorConfig,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
    ) {
        let t = ctx.theme;
        let bg = t.surface_elevated;
        let inner = surface_for(pa, buf, ctx, t, Some((self.id, cfg.dismiss)));
        ctx.scrollable(self.id, inner);
        if search > 0 && inner.height > 0 {
            let q = Rect::new(inner.x, inner.y, inner.width, 1);
            let fs = t.field_style(crate::ui::ctx::VisualState {
                editing: true,
                ..Default::default()
            });
            fill(buf, q, fs);
            buf.set_string(q.x + 1, q.y, "⌕", fs.fg(t.text_muted));

            let line = Rect::new(q.x + 3, q.y, q.width.saturating_sub(3), 1);
            let caret = if self.query.text().is_empty() {
                buf.set_string(
                    line.x,
                    line.y,
                    "Search",
                    fs.patch(t.placeholder(Default::default())),
                );
                line.as_position()
            } else {
                self.query
                    .clone()
                    .draw_line(buf, line, &Self::query_look(t))
            };
            ctx.set_cursor(caret);

            ctx.register_part(self.id, PartRef::of(Part::QUERY), line);
        }
        let list = Rect::new(
            inner.x,
            inner.y + search,
            inner.width,
            inner.height.saturating_sub(search),
        );
        if shown.is_empty() {
            buf.set_string(
                list.x + 3,
                list.y,
                "No matches",
                Style::new().fg(t.text_muted).bg(bg),
            );
            return;
        }
        for (row_i, &i) in shown
            .iter()
            .skip(self.list.scroll.offset)
            .take(list.height as usize)
            .enumerate()
        {
            let y = list.y + row_i as u16;
            let o = &self.options.0[i];
            let rid = self.option_id(i);
            let mut rs = ctx.state(rid);
            rs.focused = i == self.cursor;
            rs.disabled = o.is_disabled();
            if rs.disabled {
                rs.hovered = false;
                rs.pressed = false;
            }
            rs.selected = self.current == Some(i);
            let cr = t.cursor_row(rs, bg, cfg.cursor_style);
            let st = cr.style;
            let row = Rect::new(list.x, y, list.width, 1);
            fill(buf, row, st);
            buf.set_string(row.x, y, cr.gutter, cr.gutter_style);
            let ink = if rs.disabled {
                t.disabled
            } else {
                t.row_ink(&cr, t.text_secondary)
            };
            let mut x = row.x + 1;
            match self.kind {
                Kind::Radio => {
                    let g = if self.current == Some(i) {
                        t.glyphs.radio_on
                    } else {
                        t.glyphs.radio_off
                    };
                    buf.set_string(x, y, g, st.fg(ink));
                    x += width(g) as u16 + 1;
                }
                Kind::Tags => {
                    let g = if self.selection.contains(i) {
                        t.glyphs.check_on
                    } else {
                        t.glyphs.check_off
                    };
                    buf.set_string(x, y, g, st.fg(ink));
                    x += width(g) as u16 + 1;
                }
                _ => {
                    if self.current == Some(i) {
                        buf.set_string(x, y, t.glyphs.current, st.fg(ink));
                    }
                    x += 2;
                }
            }
            let note = o.disabled.as_deref().or(o.description.as_deref());
            let note_w = note.map_or(0, |n| width(n) as u16 + 2);
            let avail = row.right().saturating_sub(x + 1);
            let label_w = if note_w > 0 && avail > note_w + 4 {
                avail - note_w
            } else {
                avail
            };
            let mut ls = if rs.disabled { st.fg(t.disabled) } else { st };
            if let Some(tone) = o.tone
                && !rs.disabled
                && !cr.on_fill
            {
                ls = ls.fg(t.tone(tone));
            }
            buf.set_string(x, y, truncate(&o.label, label_w as usize), ls);
            if let Some(n) = note
                && label_w < avail
            {
                let ns = st.fg(if rs.disabled {
                    t.disabled
                } else {
                    t.row_ink(&cr, t.text_muted)
                });
                let nx = row.right().saturating_sub(note_w - 1);
                buf.set_string(nx, y, truncate(n, (note_w - 2) as usize), ns);
            }
            if !rs.disabled {
                ctx.register_part(self.id, Self::option_part(i), row);
            }
        }
        let more_above = self.list.scroll.offset > 0;
        let more_below = self.list.scroll.offset + (list.height as usize) < shown.len();
        let edge = Style::new().fg(t.text_secondary).bg(bg);
        let x = pa.right().saturating_sub(3);
        if more_above {
            buf.set_string(x, pa.y, t.glyphs.more_above, edge);
        }
        if more_below {
            buf.set_string(x, pa.bottom().saturating_sub(1), t.glyphs.more_below, edge);
        }
    }

    fn query_look(t: &crate::theme::Theme) -> crate::widgets::edit::LineLook {
        let fs = t.field_style(crate::ui::ctx::VisualState {
            editing: true,
            ..Default::default()
        });
        crate::widgets::edit::LineLook {
            text: fs,
            selection: t.selection(),
            ellipsis: fs.fg(t.text_muted),
            ellipsis_glyph: t.glyphs.ellipsis,
            underline: None,
            follow_caret: true,
        }
    }

    fn render_strip(&mut self, cell: Rect, buf: &Buffer, ctx: &mut RenderCtx, cfg: &EditorConfig) {
        let screen = *buf.area();
        let sep = 1u16;
        let w: u16 = self
            .options
            .0
            .iter()
            .map(|o| width(&o.label) as u16 + 2)
            .sum::<u16>()
            + sep * (self.options.len().saturating_sub(1) as u16)
            + 2;
        let pa = place_with(
            screen,
            cell,
            w.max(cell.width),
            3,
            cfg.placement,
            Place::DEFAULT,
        );
        let me = self.clone();
        let cfg = cfg.clone();
        ctx.overlay(move |buf, ctx| {
            let t = ctx.theme;
            let bg = t.surface_elevated;
            let inner = surface_for(pa, buf, ctx, t, Some((me.id, cfg.dismiss)));
            let mut x = inner.x;
            for (i, o) in me.options.0.iter().enumerate() {
                let sw = width(&o.label) as u16 + 2;
                if x + sw > inner.right() {
                    break;
                }
                let rid = me.option_id(i);
                let mut rs = ctx.state(rid);
                rs.focused = i == me.cursor;
                rs.disabled = o.is_disabled();
                if rs.disabled {
                    rs.hovered = false;
                    rs.pressed = false;
                }
                let cr = t.cursor_row(rs, bg, cfg.cursor_style);
                let seg = Rect::new(x, inner.y, sw, 1);
                fill(buf, seg, cr.style);
                let mut ls = cr.style;
                if rs.disabled {
                    ls = ls.fg(t.disabled);
                } else if let Some(tone) = o.tone
                    && !cr.on_fill
                {
                    ls = ls.fg(t.tone(tone));
                }
                if me.current == Some(i) {
                    ls = ls.add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
                }
                buf.set_string(x + 1, inner.y, &o.label, ls);
                if !rs.disabled {
                    ctx.register_part(me.id, Self::option_part(i), seg);
                }
                x += sw;
                if i + 1 < me.options.len() && x < inner.right() {
                    buf.set_string(
                        x,
                        inner.y,
                        t.glyphs.divider,
                        Style::new().fg(t.border_strong).bg(bg),
                    );
                    x += sep;
                }
            }
        });
    }
}

impl crate::core::pointer::MouseTarget for ChoiceEditor {
    type Action = ChoiceResult;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<ChoiceResult> {
        use crate::core::intent::Phase;
        use crate::core::response::Response;
        if let Some((axis, delta)) = ev.wheel() {
            if ev.part().part == Part::BACKDROP {
                return Response::ignored();
            }
            return self.wheel(axis, delta).lift();
        }
        if self.kind == Kind::Combo
            && let Some(r) = crate::widgets::option_list::query_pointer(&mut self.query, ev)
        {
            return r.lift();
        }
        match ev.phase() {
            Some(Phase::Click) => match self.on_click(ev.id) {
                (_, Some(r)) => Response::emit(r),
                (o, None) => o.into(),
            },
            Some(Phase::Press | Phase::Drag | Phase::Secondary) => Response::consumed(),
            _ => Response::ignored(),
        }
    }
}

struct Shown<'a> {
    ed: &'a ChoiceEditor,
    rows: Vec<usize>,
}

impl RowSource for Shown<'_> {
    fn len(&self) -> usize {
        self.rows.len()
    }
    fn enabled(&self, k: usize) -> bool {
        self.ed.enabled(self.rows[k])
    }
    fn label(&self, k: usize) -> &str {
        &self.ed.options.0[self.rows[k]].label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn envs() -> Options {
        Options::new([
            OptionItem::new("dev").label("Development"),
            OptionItem::new("stage").label("Staging"),
            OptionItem::new("prod")
                .label("Production")
                .tone(Tone::Warning),
            OptionItem::new("dr")
                .label("Disaster recovery")
                .disabled("Read-only region"),
        ])
    }

    #[test]
    fn display_tone_and_out_of_set() {
        let cfg = EditorConfig::default();
        let e = ColumnEditor::Select(envs());
        assert_eq!(e.display("prod", &cfg), "Production");
        assert_eq!(e.tone("prod"), Some(Tone::Warning));
        assert!(!e.out_of_set("stage", &cfg));
        assert!(
            e.out_of_set("qa", &cfg),
            "an imported value outside the set"
        );
        assert_eq!(e.display("qa", &cfg), "qa", "shown as stored");
        assert!(!e.out_of_set("", &cfg));
        let tags = ColumnEditor::Tags(Options::values(&["api", "web", "db"]));
        assert!(!tags.out_of_set("api, db", &cfg));
        assert!(tags.out_of_set("api, cache", &cfg));
        assert!(!ColumnEditor::Text.out_of_set("anything", &cfg));
        let toggle = ColumnEditor::Checkbox(Options::values(&["off", "on"]));
        assert_eq!(toggle.checked("on"), Some(true));
        assert_eq!(toggle.checked("off"), Some(false));
        assert_eq!(toggle.checked("maybe"), None);
        let ladder = |n: usize| {
            let values: Vec<String> = (0..n).map(|i| format!("v{i}")).collect();
            let refs: Vec<&str> = values.iter().map(String::as_str).collect();
            ColumnEditor::for_options(Options::values(&refs))
        };
        assert!(matches!(ladder(2), ColumnEditor::Checkbox(_)));
        assert!(matches!(ladder(4), ColumnEditor::Segmented(_)));
        assert!(matches!(ladder(9), ColumnEditor::Select(_)));
        assert!(matches!(ladder(30), ColumnEditor::Combo(_)));
        assert_eq!(toggle.toggled("off").as_deref(), Some("on"));
        assert_eq!(toggle.toggled("on").as_deref(), Some("off"));
        assert_eq!(toggle.toggled("maybe").as_deref(), Some("off"));
    }

    #[test]
    fn the_popup_moves_with_nav_keys_not_raw_key_codes() {
        let vim = EditorConfig {
            nav: crate::core::keymap::NavKeys::VIM,
            type_ahead: false,
            ..EditorConfig::default()
        };
        let open = |cfg: &EditorConfig| {
            ChoiceEditor::open(
                WidgetId::of("c"),
                &ColumnEditor::Select(envs()),
                "stage",
                cfg,
            )
            .unwrap()
        };
        let mut c = open(&vim);
        c.on_key(&key(KeyCode::Char('k')), &vim);
        assert_eq!(c.cursor, 0, "k moves up under the Vim keys");
        c.on_key(&key(KeyCode::Char('G')), &vim);
        assert_eq!(c.cursor, 2, "G goes to the last enabled option");
        let no_arrows = EditorConfig {
            nav: crate::core::keymap::NavKeys::DESKTOP.arrows(false),
            ..EditorConfig::default()
        };
        let mut c = open(&no_arrows);
        c.on_key(&key(KeyCode::Up), &no_arrows);
        assert_eq!(c.cursor, 1, "arrows switched off do not move");
    }

    #[test]
    fn select_keys_skip_disabled_type_ahead_and_commit_or_cancel() {
        let cfg = EditorConfig::default();
        let mut c = ChoiceEditor::open(
            WidgetId::of("c"),
            &ColumnEditor::Select(envs()),
            "stage",
            &cfg,
        )
        .unwrap();
        assert_eq!(c.cursor, 1, "opens on the current value");
        c.on_key(&key(KeyCode::Down), &cfg);
        c.on_key(&key(KeyCode::Down), &cfg);
        assert_eq!(c.cursor, 2, "the disabled option is skipped");
        c.on_key(&key(KeyCode::End), &cfg);
        assert_eq!(c.cursor, 2, "End lands on the last enabled option");
        c.on_key(&key(KeyCode::Char('d')), &cfg);
        assert_eq!(c.cursor, 0, "type-ahead never lands on a disabled option");
        assert_eq!(
            c.on_key(&key(KeyCode::Enter), &cfg).1,
            Some(ChoiceResult::Commit("dev".into()))
        );
        assert_eq!(
            c.on_key(&key(KeyCode::Esc), &cfg).1,
            Some(ChoiceResult::Cancel)
        );
        let tab = Key {
            code: KeyCode::BackTab,
            mods: KeyModifiers::SHIFT,
        };
        assert_eq!(
            c.on_key(&tab, &cfg).1,
            Some(ChoiceResult::CommitAndTab {
                value: "dev".into(),
                backward: true
            })
        );
    }

    #[test]
    fn an_out_of_set_value_opens_with_no_current_option() {
        let cfg = EditorConfig::default();
        let c = ChoiceEditor::open(WidgetId::of("c"), &ColumnEditor::Select(envs()), "qa", &cfg)
            .unwrap();
        assert_eq!(c.current, None);
        assert_eq!(c.cursor, 0, "first enabled option");
    }

    #[test]
    fn combo_filters_by_typing_and_only_commits_listed_values() {
        let cfg = EditorConfig::default();
        let mut c =
            ChoiceEditor::open(WidgetId::of("c"), &ColumnEditor::Combo(envs()), "dev", &cfg)
                .unwrap();
        for ch in "prod".chars() {
            c.on_key(&key(KeyCode::Char(ch)), &cfg);
        }
        assert_eq!(c.shown(), vec![2]);
        assert_eq!(
            c.on_key(&key(KeyCode::Enter), &cfg).1,
            Some(ChoiceResult::Commit("prod".into()))
        );
        c.on_key(&key(KeyCode::Char('x')), &cfg);
        assert!(c.shown().is_empty());
        assert_eq!(
            c.on_key(&key(KeyCode::Enter), &cfg),
            (Outcome::Consumed, None),
            "no free text"
        );
        c.on_key(&key(KeyCode::Backspace), &cfg);
        assert_eq!(c.shown(), vec![2]);
        assert_eq!(c.on_paste(" "), Outcome::Changed);
    }

    #[test]
    fn the_combo_search_edits_at_the_caret_selects_all_and_undoes() {
        use ratatui::crossterm::event::KeyModifiers as M;
        let cfg = EditorConfig::default();
        let mut c =
            ChoiceEditor::open(WidgetId::of("c"), &ColumnEditor::Combo(envs()), "dev", &cfg)
                .unwrap();
        for ch in "prd".chars() {
            c.on_key(&key(KeyCode::Char(ch)), &cfg);
        }
        c.on_key(&key(KeyCode::Left), &cfg);
        c.on_key(&key(KeyCode::Char('o')), &cfg);
        assert_eq!(c.query.text(), "prod");
        assert_eq!(c.shown(), vec![2]);
        let k = |ch, mods| Key {
            code: KeyCode::Char(ch),
            mods,
        };
        c.on_key(&k('a', M::SUPER), &cfg);
        assert_eq!(c.query.buffer.selected_text(), Some("prod"));
        c.on_key(&key(KeyCode::Char('d')), &cfg);
        assert_eq!(c.query.text(), "d");
        c.on_key(&k('z', M::CONTROL), &cfg);
        assert_eq!(c.query.text(), "prod", "Ctrl+Z");
        assert_eq!(c.shown(), vec![2], "the list follows the undone query");
    }

    #[test]
    fn tags_toggle_with_space_and_commit_the_checked_set() {
        let cfg = EditorConfig::default();
        let opts = Options::new([
            OptionItem::new("api"),
            OptionItem::new("web"),
            OptionItem::new("legacy").disabled("Retired"),
            OptionItem::new("db"),
        ]);
        let mut c =
            ChoiceEditor::open(WidgetId::of("c"), &ColumnEditor::Tags(opts), "db", &cfg).unwrap();
        assert_eq!(c.selection.to_vec(), vec![3]);
        c.on_key(&key(KeyCode::Home), &cfg);
        c.on_key(&key(KeyCode::Char(' ')), &cfg);
        c.on_click(c.option_id(2));
        assert!(!c.selection.contains(2), "a disabled tag never checks");
        assert_eq!(
            c.on_key(&key(KeyCode::Enter), &cfg).1,
            Some(ChoiceResult::Commit("api, db".into()))
        );
    }

    #[test]
    fn clicks_choose_enabled_options_and_the_backdrop_follows_dismiss() {
        let mut cfg = EditorConfig::default();
        let mut c =
            ChoiceEditor::open(WidgetId::of("c"), &ColumnEditor::Radio(envs()), "dev", &cfg)
                .unwrap();
        assert_eq!(c.on_click(c.option_id(3)), (Outcome::Consumed, None));
        assert_eq!(
            c.on_click(c.option_id(1)).1,
            Some(ChoiceResult::Commit("stage".into()))
        );
        assert_eq!(c.on_click(backdrop_id(c.id)).1, Some(ChoiceResult::Cancel));
        cfg.dismiss = Dismiss::ESCAPE;
        c.list.dismiss = Dismiss::ESCAPE;
        assert_eq!(c.on_click(backdrop_id(c.id)), (Outcome::Consumed, None));
        assert_eq!(
            c.on_key(&key(KeyCode::Esc), &cfg).1,
            Some(ChoiceResult::Cancel)
        );
    }

    #[test]
    fn the_wheel_over_the_popup_follows_its_wheel_mode() {
        use crate::core::pointer::{MouseTarget, PointerEvent};
        let theme = Theme::junie();
        let many = Options::values(&["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]);
        let wheel = |mode: WheelMode, delta: i32, axis: Axis| {
            let cfg = EditorConfig {
                max_rows: 4,
                wheel: mode,
                ..Default::default()
            };
            let mut c = ChoiceEditor::open(
                WidgetId::of("c"),
                &ColumnEditor::Select(many.clone()),
                "a",
                &cfg,
            )
            .unwrap();
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 20));
            c.render(Rect::new(2, 1, 10, 1), &buf, &mut ctx, &cfg);
            ctx.flush_overlays(&mut buf);
            drop(ctx);
            let row = c.option_id(1);
            let pos = hits.area_of(row).expect("a shown row").as_position();
            let ev = PointerEvent::wheel_on(&hits, row, axis, delta, pos, KeyModifiers::NONE);
            let r = c.pointer(&ev).outcome();
            (r, c.cursor, c.list.scroll.offset)
        };
        assert_eq!(
            wheel(WheelMode::Cursor, 2, Axis::Vertical),
            (Outcome::Changed, 2, 0)
        );
        assert_eq!(
            wheel(WheelMode::Viewport, 3, Axis::Vertical),
            (Outcome::Changed, 0, 3)
        );
        assert_eq!(
            wheel(WheelMode::Viewport, 3, Axis::Horizontal).0,
            Outcome::Consumed
        );
    }

    #[test]
    fn the_popup_is_an_overlay_anchored_to_the_cell_with_a_fill_cursor() {
        let theme = Theme::junie();
        let cfg = EditorConfig::default();
        let mut c = ChoiceEditor::open(
            WidgetId::of("c"),
            &ColumnEditor::Radio(envs()),
            "stage",
            &cfg,
        )
        .unwrap();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
        let cell = Rect::new(10, 17, 12, 1);
        c.render(cell, &buf, &mut ctx, &cfg);
        ctx.flush_overlays(&mut buf);
        let text: Vec<String> = (0..20)
            .map(|y| (0..60).map(|x| buf[(x, y)].symbol()).collect())
            .collect();
        let at = |needle: &str| text.iter().position(|r| r.contains(needle));
        let staging = at("Staging").expect("listed");
        assert!(
            staging < 17,
            "flips above a cell at the bottom:\n{}",
            text.join("\n")
        );
        assert!(
            text[staging].contains(theme.glyphs.radio_on),
            "the current value"
        );
        assert!(
            text[at("Disaster").unwrap()].contains("Read-only region"),
            "the reason"
        );
        assert!(
            !text.iter().any(|r| r.contains(theme.glyphs.focus_bar)),
            "no second focus bar:\n{}",
            text.join("\n")
        );
        assert!(ctx.hits.area_of(c.option_id(1)).is_some());
        assert!(
            ctx.hits.area_of(c.option_id(3)).is_none(),
            "disabled: not clickable"
        );
    }
}
