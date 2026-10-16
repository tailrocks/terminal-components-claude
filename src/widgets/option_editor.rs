//! Constrained cell editors: table and grid columns whose value comes from
//! a fixed set of options.
//!
//! A column carries a [`ColumnEditor`]; the table opens a [`ChoiceEditor`]
//! on the cell (Enter, F2, Space, a click on the current cell or on its
//! `▾`). The editor is an overlay anchored to the cell through
//! [`place_with`] (it flips above at the bottom of the screen and follows
//! horizontal scrolling because the anchor is the cell as drawn this
//! frame), its cursor row is a fill ([`CursorStyle::Fill`]) so the table
//! keeps the only focus bar, and the value it commits goes through the
//! table's own edit pipeline (validator, sort, events). Behaviour is
//! configured once per table in [`EditorConfig`]; the defaults follow
//! macOS popup buttons and menus.
//!
//! | Editor | Opens | Commits |
//! | --- | --- | --- |
//! | [`ColumnEditor::Select`] | a list | Enter, Space, click |
//! | [`ColumnEditor::Combo`] | a search line over a filtered list | Enter, click |
//! | [`ColumnEditor::Radio`] | a list of radio rows with descriptions | Enter, Space, click |
//! | [`ColumnEditor::Checkbox`] | nothing: two values flip in place | at once |
//! | [`ColumnEditor::Segmented`] | a one-row strip of segments | Enter, Space, click |
//! | [`ColumnEditor::Tags`] | a checklist (several values) | Space toggles, Enter commits |

use std::collections::BTreeSet;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use crate::core::event::{Key, Outcome};
use crate::core::focus::BlurAction;
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::theme::{CursorStyle, Tone};
use crate::ui::ctx::{RenderCtx, fill};
use crate::ui::popup::{Dismiss, Place, Placement, backdrop_id, place_with, surface_for};
use crate::ui::text::{truncate, width};

/// One allowed value of a constrained column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionItem {
    /// What the cell stores (and what a validator sees).
    pub value: String,
    /// What the cell and the popup show. Defaults to the value.
    pub label: String,
    /// Colour of the value in the cell (a status pill per value).
    pub tone: Option<Tone>,
    /// A muted line beside the label in the popup.
    pub description: Option<String>,
    /// `Some(reason)`: shown but never chosen; the reason replaces the
    /// description.
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

/// The allowed values of a constrained column, in display order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options(pub Vec<OptionItem>);

impl Options {
    pub fn new(items: impl IntoIterator<Item = OptionItem>) -> Self {
        Self(items.into_iter().collect())
    }
    /// Plain values that are their own labels.
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
    /// Index of the option storing `value`.
    pub fn find(&self, value: &str) -> Option<usize> {
        self.0.iter().position(|o| o.value == value)
    }
}

/// How a column's cells are edited.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ColumnEditor {
    /// Free text in place (the long-standing editor).
    #[default]
    Text,
    /// Pick one from a list anchored to the cell (macOS popup button).
    Select(Options),
    /// Pick one from a list filtered by typing (a searchable combo box).
    /// Only listed values are accepted.
    Combo(Options),
    /// Pick one from radio rows that can carry descriptions.
    Radio(Options),
    /// Two values (`[off, on]`, a boolean) shown as a checkbox that flips
    /// in place: a checkbox, not a switch (switches apply settings at once;
    /// a cell is data).
    Checkbox(Options),
    /// Pick one of a few short values on a one-row segmented strip.
    Segmented(Options),
    /// Pick any number of values; the cell stores them joined by
    /// [`EditorConfig::tag_separator`].
    Tags(Options),
}

impl ColumnEditor {
    /// The options of a constrained editor; `None` for [`Self::Text`].
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

    /// The editor takes values from a fixed set.
    pub fn is_constrained(&self) -> bool {
        self.options().is_some()
    }

    /// The editor opens a popup (every constrained editor but Checkbox).
    pub fn has_popup(&self) -> bool {
        self.is_constrained() && !matches!(self, Self::Checkbox(_))
    }

    /// The editor the option-count ladder picks: 2 values a checkbox, 3–5
    /// a segmented strip, 6–15 a dropdown, more a searchable combo box.
    /// Radio (values with descriptions) and Tags (several values) are
    /// chosen explicitly.
    pub fn for_options(options: Options) -> Self {
        match options.len() {
            0..=2 => Self::Checkbox(options),
            3..=5 => Self::Segmented(options),
            6..=15 => Self::Select(options),
            _ => Self::Combo(options),
        }
    }

    /// A Checkbox cell's state: `Some(true)` for its second (on) value,
    /// `Some(false)` for the first, `None` for anything else.
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

    /// What a cell storing `raw` shows: option labels (joined for Tags);
    /// an unknown value as stored.
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

    /// `raw` holds a value outside the options (imported data). An empty
    /// cell is never out of set.
    pub fn out_of_set(&self, raw: &str, cfg: &EditorConfig) -> bool {
        let Some(opts) = self.options() else {
            return false;
        };
        if raw.trim().is_empty() {
            return false;
        }
        self.parts(raw, cfg).iter().any(|p| opts.find(p).is_none())
    }

    /// The tone of the option `raw` stores (single-value editors).
    pub fn tone(&self, raw: &str) -> Option<Tone> {
        let opts = self.options()?;
        if matches!(self, Self::Tags(_)) {
            return None;
        }
        opts.find(raw).and_then(|i| opts.0[i].tone)
    }

    /// The value a Checkbox flips `raw` to: the other of its two values (an
    /// unknown value becomes the first enabled one).
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

/// Behaviour shared by every constrained editor of a table or grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorConfig {
    /// How an open popup closes without a choice. Default
    /// [`Dismiss::LIGHT`]: Esc or a click outside cancels.
    pub dismiss: Dismiss,
    /// What focus leaving the table does to an open popup. Default
    /// [`BlurAction::Cancel`]: a macOS popup closes without a choice.
    /// `Commit` applies the highlighted option (Tags: the checked set).
    pub blur: BlurAction,
    /// Most option rows a popup shows before it scrolls. Default 8.
    pub max_rows: u16,
    /// Width range of a popup, frame included. Default (14, 48).
    pub popup_width: (u16, u16),
    /// How the popup marks its cursor row. Default [`CursorStyle::Fill`].
    pub cursor_style: CursorStyle,
    /// Typing jumps to the option whose label starts with the typed
    /// prefix (Select, Radio, Tags, Segmented). Default on.
    pub type_ahead: bool,
    /// Where editable popup cells show their trailing `▾`. Default
    /// [`Affordance::Focused`].
    pub affordance: Affordance,
    /// A value outside the options is drawn in the error tone with `!`.
    /// Default on.
    pub flag_out_of_set: bool,
    /// Between Tags values in the stored text and the cell. Default `", "`.
    pub tag_separator: &'static str,
    /// Where the popup goes relative to the cell. Default
    /// [`Placement::Below`] (flips above when there is no room).
    pub placement: Placement,
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
            affordance: Affordance::Focused,
            flag_out_of_set: true,
            tag_separator: ", ",
            placement: Placement::Below,
        }
    }
}

/// Which editable option cells show the `▾` that opens their editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Affordance {
    /// The focused cursor cell and the hovered cell only: the table stays
    /// quiet until a cell is the one being worked on.
    #[default]
    Focused,
    /// Every editable option cell.
    Always,
    /// None (the keyboard and a second click still open the editor).
    Never,
}

/// What a key or a click did to an open [`ChoiceEditor`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceResult {
    /// Store this value.
    Commit(String),
    /// Close without a change.
    Cancel,
    /// Store this value and move to the next (or previous) editable cell.
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

/// An open constrained editor: the popup over one cell.
#[derive(Debug, Clone)]
pub struct ChoiceEditor {
    /// The popup's id: options are its children, a click outside is
    /// [`backdrop_id`] of it.
    pub id: WidgetId,
    kind: Kind,
    pub options: Options,
    /// Option index under the cursor.
    pub cursor: usize,
    /// The option the cell held when the editor opened (`None`: empty or
    /// out of set).
    pub current: Option<usize>,
    /// Tags: the checked options.
    pub checked: BTreeSet<usize>,
    /// Combo: the search text.
    pub query: String,
    /// A validator rejected the last commit; the popup stays open.
    pub error: Option<String>,
    scroll: usize,
    visible_rows: usize,
    typeahead: crate::core::typeahead::TypeAhead,
}

impl ChoiceEditor {
    /// Open `editor` on a cell holding `raw`; `None` for editors without a
    /// popup (Text, Checkbox).
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
            editor
                .parts(raw, cfg)
                .iter()
                .filter_map(|p| options.find(p))
                .collect()
        } else {
            BTreeSet::new()
        };
        let first_enabled = options.0.iter().position(|o| !o.is_disabled()).unwrap_or(0);
        let cursor = current
            .filter(|&i| !options.0[i].is_disabled())
            .or_else(|| checked.iter().next().copied())
            .unwrap_or(first_enabled);
        Some(Self {
            id,
            kind,
            options,
            cursor,
            current,
            checked,
            query: String::new(),
            error: None,
            scroll: 0,
            visible_rows: cfg.max_rows as usize,
            typeahead: Default::default(),
        })
    }

    pub fn option_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::option_part(i))
    }

    fn option_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    /// The option index a widget id names.
    pub fn locate(&self, id: WidgetId) -> Option<usize> {
        self.id.item_of(id, Part::ROW, 0..self.options.len())
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == backdrop_id(self.id) || self.locate(id).is_some()
    }

    /// The options the popup lists, in order (Combo: those matching the
    /// query).
    pub fn shown(&self) -> Vec<usize> {
        let q = self.query.trim().to_lowercase();
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

    /// Step the cursor `delta` enabled options through the shown ones.
    fn step(&mut self, delta: isize) {
        let shown: Vec<usize> = self
            .shown()
            .into_iter()
            .filter(|&i| self.enabled(i))
            .collect();
        if shown.is_empty() {
            return;
        }
        let at = shown.iter().position(|&i| i == self.cursor);
        let next = match at {
            None => 0,
            Some(k) => (k as isize + delta).clamp(0, shown.len() as isize - 1) as usize,
        };
        self.cursor = shown[next];
    }

    fn edge(&mut self, last: bool) {
        let shown: Vec<usize> = self
            .shown()
            .into_iter()
            .filter(|&i| self.enabled(i))
            .collect();
        if let Some(&i) = if last { shown.last() } else { shown.first() } {
            self.cursor = i;
        }
    }

    /// The value Enter commits now: the cursor option (Tags: the checked
    /// set in option order). `None` when nothing enabled is under the
    /// cursor.
    fn value(&self, cfg: &EditorConfig) -> Option<String> {
        if self.kind == Kind::Tags {
            return Some(
                self.checked
                    .iter()
                    .map(|&i| self.options.0[i].value.as_str())
                    .collect::<Vec<_>>()
                    .join(cfg.tag_separator),
            );
        }
        if !self.enabled(self.cursor) || !self.shown().contains(&self.cursor) {
            return None;
        }
        Some(self.options.0[self.cursor].value.clone())
    }

    /// The commit a blur makes under [`EditorConfig::blur`].
    pub fn blur_value(&self, cfg: &EditorConfig) -> Option<String> {
        self.value(cfg)
    }

    fn toggle_checked(&mut self, i: usize) {
        if !self.enabled(i) {
            return;
        }
        if !self.checked.remove(&i) {
            self.checked.insert(i);
        }
    }

    pub fn on_key(&mut self, key: &Key, cfg: &EditorConfig) -> (Outcome, Option<ChoiceResult>) {
        if matches!(key.code, KeyCode::Char(_)) && key.chord() {
            return (Outcome::Consumed, None);
        }
        let horizontal = self.kind == Kind::Segmented;
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
            KeyCode::Up | KeyCode::Left if horizontal => self.step(-1),
            KeyCode::Down | KeyCode::Right if horizontal => self.step(1),
            KeyCode::Up => self.step(-1),
            KeyCode::Down => self.step(1),
            KeyCode::PageUp => self.step(-(self.visible_rows.max(1) as isize)),
            KeyCode::PageDown => self.step(self.visible_rows.max(1) as isize),
            KeyCode::Home => self.edge(false),
            KeyCode::End => self.edge(true),
            KeyCode::Backspace if self.kind == Kind::Combo => {
                self.query.pop();
                self.after_query();
            }
            KeyCode::Char(c) if self.kind == Kind::Combo => {
                self.query.push(c);
                self.after_query();
            }
            KeyCode::Char(c) if cfg.type_ahead && !c.is_whitespace() => {
                let prefix = self.typeahead.feed(c).to_owned();
                let found = crate::core::typeahead::TypeAhead::find(
                    &prefix,
                    self.options.0.iter().map(|o| o.label.as_str()),
                    self.cursor,
                    |i| self.enabled(i),
                );
                match found {
                    Some(i) => self.cursor = i,
                    None => return (Outcome::Consumed, None),
                }
            }
            _ => return (Outcome::Consumed, None),
        }
        (Outcome::Changed, None)
    }

    /// The query changed: the cursor goes to the first enabled match.
    fn after_query(&mut self) {
        self.error = None;
        if let Some(&i) = self.shown().iter().find(|&&i| self.enabled(i)) {
            self.cursor = i;
        }
        self.scroll = 0;
    }

    /// Text pasted while open: the Combo search takes it.
    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if self.kind != Kind::Combo {
            return Outcome::Consumed;
        }
        self.query.push_str(text.trim());
        self.after_query();
        Outcome::Changed
    }

    /// A click on the popup, an option, or outside it (the backdrop).
    pub fn on_click(
        &mut self,
        id: WidgetId,
        cfg: &EditorConfig,
    ) -> (Outcome, Option<ChoiceResult>) {
        if id == backdrop_id(self.id) {
            return if cfg.dismiss.outside_click {
                (Outcome::Changed, Some(ChoiceResult::Cancel))
            } else {
                (Outcome::Consumed, None)
            };
        }
        let Some(i) = self.locate(id) else {
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

    /// The wheel over the open popup moves its cursor.
    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        let before = self.cursor;
        self.step(delta as isize);
        if before == self.cursor {
            Outcome::Consumed
        } else {
            Outcome::Changed
        }
    }

    /// Queue the popup as an overlay anchored to `cell` (the cell as drawn
    /// this frame).
    pub fn render(&mut self, cell: Rect, buf: &Buffer, ctx: &mut RenderCtx, cfg: &EditorConfig) {
        if self.kind == Kind::Segmented {
            self.render_strip(cell, buf, ctx, cfg);
        } else {
            self.render_list(cell, buf, ctx, cfg);
        }
    }

    /// Prefix cells before a label: the current mark, a radio or a box.
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
            // frame, gutter and a free cell on each side of a note
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
        self.visible_rows = visible;
        let at = shown.iter().position(|&i| i == self.cursor).unwrap_or(0);
        if at < self.scroll {
            self.scroll = at;
        } else if at >= self.scroll + visible {
            self.scroll = at + 1 - visible;
        }
        self.scroll = self.scroll.min(shown.len().saturating_sub(visible));
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
            let shown_q = truncate(&self.query, q.width.saturating_sub(4) as usize);
            buf.set_string(q.x + 3, q.y, &shown_q, fs);
            if self.query.is_empty() {
                buf.set_string(
                    q.x + 3,
                    q.y,
                    "Search",
                    fs.patch(t.placeholder(Default::default())),
                );
            }
            ctx.set_cursor(Position::new(q.x + 3 + width(&shown_q) as u16, q.y));
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
            .skip(self.scroll)
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
                    let g = if self.checked.contains(&i) {
                        t.glyphs.check_on
                    } else {
                        t.glyphs.check_off
                    };
                    buf.set_string(x, y, g, st.fg(ink));
                    x += width(g) as u16 + 1;
                }
                _ => {
                    if self.current == Some(i) {
                        // the value the cell holds is a fact: neutral
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
        let more_above = self.scroll > 0;
        let more_below = self.scroll + (list.height as usize) < shown.len();
        let edge = Style::new().fg(t.text_secondary).bg(bg);
        let x = pa.right().saturating_sub(3);
        if more_above {
            buf.set_string(x, pa.y, t.glyphs.more_above, edge);
        }
        if more_below {
            buf.set_string(x, pa.bottom().saturating_sub(1), t.glyphs.more_below, edge);
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
        assert_eq!(c.checked, BTreeSet::from([3]));
        c.on_key(&key(KeyCode::Home), &cfg);
        c.on_key(&key(KeyCode::Char(' ')), &cfg);
        c.on_click(c.option_id(2), &cfg);
        assert!(!c.checked.contains(&2), "a disabled tag never checks");
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
        assert_eq!(c.on_click(c.option_id(3), &cfg), (Outcome::Consumed, None));
        assert_eq!(
            c.on_click(c.option_id(1), &cfg).1,
            Some(ChoiceResult::Commit("stage".into()))
        );
        assert_eq!(
            c.on_click(backdrop_id(c.id), &cfg).1,
            Some(ChoiceResult::Cancel)
        );
        cfg.dismiss = Dismiss::ESCAPE;
        assert_eq!(
            c.on_click(backdrop_id(c.id), &cfg),
            (Outcome::Consumed, None)
        );
        assert_eq!(
            c.on_key(&key(KeyCode::Esc), &cfg).1,
            Some(ChoiceResult::Cancel)
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
