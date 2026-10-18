use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Modifier;

use crate::core::edit::{EditEntry, EditKind, FieldKeys, FocusSelect, TextCmd};
use crate::core::event::{Key, Outcome};
use crate::core::focus::{BlurAction, request_focus_next};
use crate::core::id::WidgetId;
use crate::ui::ctx::RenderCtx;
use crate::ui::edit_menu::{
    EditCaps, EditMenu, EditMenuHost, edit_menu_key, edit_menu_pointer, edit_menu_route,
    editable_target, point_editable,
};
use crate::ui::text::width;
use crate::widgets::edit::{EditField, Editable, FieldError, LineLook, ValidateWhen, Validator};
use crate::widgets::field_common::{EditAction, PointerField, PointerSelect, paste_clipboard};

impl crate::ui::measure::Measure for TextInput {
    fn measure(
        &self,
        _avail: ratatui::layout::Size,
        ctx: &crate::ui::ctx::RenderCtx,
    ) -> ratatui::layout::Size {
        let t = ctx.theme;
        let suffix = if self.required {
            2
        } else if self.plain_label || self.label.is_empty() {
            0
        } else {
            4 + width(t.strings.optional)
        };

        let label = if self.wrap_label {
            suffix
        } else {
            width(&self.label) + suffix
        };

        let value = width(self.field.buffer.text()).max(width(&self.placeholder)) + 3 + 1 + 4;
        let message = self
            .field
            .error
            .as_ref()
            .map(|e| width(&e.to_string()))
            .unwrap_or(0)
            .max(width(&self.help))
            + 2;
        let w = (label + Self::GUTTER as usize).max(value).max(message) as u16;
        ratatui::layout::Size::new(w, self.height_for(w))
    }
}

#[derive(Debug, Clone)]
pub struct TextInput {
    pub id: WidgetId,
    pub label: String,
    pub placeholder: String,

    pub field: EditField,
    pub disabled: bool,
    pub required: bool,

    pub required_message: Option<String>,

    required_text: &'static str,
    pub help: String,
    pub editing: bool,

    snapshot: String,

    snapshot_error: Option<FieldError>,
    pub area: Rect,

    text_area: Rect,

    pub plain_label: bool,

    pub masked: bool,

    pub reveal_tail: u8,

    pub wrap_label: bool,

    pub blur_action: BlurAction,

    pub edit_entry: Option<EditEntry>,

    entry: EditEntry,

    pub focus_select: FocusSelect,

    pub field_keys: Option<FieldKeys>,

    focus_seen: bool,

    focus_edit: bool,

    pub label_hit: bool,

    pub validate_on: ValidateOn,

    pub readout: String,

    pub readout_link: Option<WidgetId>,

    pub readout_area: Rect,

    pub edit_menu: EditMenu,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ValidateOn {
    #[default]
    Change,

    EveryCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Committed,
    Cancelled,

    CommittedTab { backward: bool },
    Changed,
}

impl TextInput {
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_owned(),
            placeholder: String::new(),
            field: EditField::single(""),
            disabled: false,
            required: false,
            required_message: None,
            required_text: crate::theme::Strings::DEFAULT.required,
            help: String::new(),
            editing: false,
            snapshot: String::new(),
            snapshot_error: None,
            area: Rect::ZERO,
            text_area: Rect::ZERO,
            plain_label: false,
            masked: false,
            reveal_tail: 0,
            wrap_label: false,
            blur_action: BlurAction::Commit,
            edit_entry: None,
            entry: crate::theme::Theme::DEFAULT_EDIT_ENTRY,
            focus_select: FocusSelect::default(),
            field_keys: None,
            focus_seen: false,
            focus_edit: false,
            label_hit: true,
            validate_on: ValidateOn::Change,
            readout: String::new(),
            readout_link: None,
            readout_area: Rect::ZERO,
            edit_menu: EditMenu::default(),
        }
    }

    pub fn readout(mut self, text: &str) -> Self {
        self.readout = text.to_owned();
        self
    }

    pub fn set_readout(&mut self, text: &str) {
        if self.readout != text {
            self.readout = text.to_owned();
        }
    }

    pub fn readout_link(mut self, id: WidgetId) -> Self {
        self.readout_link = Some(id);
        self
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.to_owned();
        self
    }

    pub fn required_message(mut self, message: &str) -> Self {
        self.required_message = Some(message.to_owned());
        self
    }
    pub fn value(mut self, v: &str) -> Self {
        self.field.buffer.set_text(v);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }
    pub fn help(mut self, h: &str) -> Self {
        self.help = h.to_owned();
        self
    }
    pub fn plain_label(mut self) -> Self {
        self.plain_label = true;
        self
    }

    pub fn validator(mut self, v: impl Fn(&str) -> Option<String> + 'static) -> Self {
        self.field.validator = Some(Validator::check(v));
        self
    }

    pub fn validate_when(mut self, when: ValidateWhen) -> Self {
        self.field.validate_when = when;
        self
    }

    pub fn error(&self) -> Option<&str> {
        self.field.error_message()
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.field.set_error(error.map(FieldError::from));
    }
    pub fn masked(mut self) -> Self {
        self.masked = true;
        self
    }

    pub fn context_menu(mut self, on: bool) -> Self {
        self.edit_menu.enabled = on;
        self
    }
    pub fn reveal_tail(mut self, n: u8) -> Self {
        self.reveal_tail = n;
        self
    }

    pub fn wrap_label(mut self, on: bool) -> Self {
        self.wrap_label = on;
        self
    }

    pub fn blur_action(mut self, action: BlurAction) -> Self {
        self.blur_action = action;
        self
    }

    pub fn validate_on(mut self, on: ValidateOn) -> Self {
        self.validate_on = on;
        self
    }

    pub fn edit_entry(mut self, entry: EditEntry) -> Self {
        self.set_edit_entry(entry);
        self
    }

    pub fn set_edit_entry(&mut self, entry: EditEntry) {
        self.edit_entry = Some(entry);
        self.entry = entry;
    }

    #[deprecated(note = "use `edit_entry(EditEntry::TypeToEdit)` or `EditEntry::Explicit`")]
    pub fn type_to_edit(self, on: bool) -> Self {
        self.edit_entry(if on {
            EditEntry::TypeToEdit
        } else {
            EditEntry::Explicit
        })
    }

    pub fn focus_select(mut self, select: FocusSelect) -> Self {
        self.focus_select = select;
        self
    }

    pub fn text_keymap(
        mut self,
        keymap: crate::core::keymap::KeyMap<crate::core::edit::TextCmd>,
    ) -> Self {
        self.field.text_keymap = Some(keymap);
        self
    }

    fn edit_action(&self, key: &Key) -> EditAction {
        self.field.action(key)
    }

    pub fn field_keys(mut self, keys: FieldKeys) -> Self {
        self.field_keys = Some(keys);
        self.field.keys = keys;
        self
    }

    pub fn entry(&self) -> EditEntry {
        self.edit_entry.unwrap_or(self.entry)
    }

    pub fn wants_text_input(&self) -> bool {
        !self.disabled && (self.editing || self.entry() != EditEntry::Explicit)
    }

    pub fn is_dirty(&self) -> bool {
        self.editing && self.field.buffer.text() != self.snapshot
    }

    pub fn can_undo(&self) -> bool {
        self.field.can_undo()
    }

    pub fn hints(&self) -> Vec<crate::widgets::keyhint::Hint> {
        use crate::widgets::keyhint::EDIT_FIELD;

        if let Some(m) = self.edit_menu.menu() {
            return crate::core::hints::HasHints::hints(m, &crate::core::hints::HintCx::new(None));
        }
        if self.disabled {
            Vec::new()
        } else if self.wants_text_input() {
            EDIT_FIELD.to_vec()
        } else {
            vec![crate::widgets::widget_hints::EDIT]
        }
    }

    pub fn pointer_select(mut self, sel: PointerSelect) -> Self {
        self.field.sel = sel;
        self
    }

    pub fn on_blur(&mut self) -> Option<InputAction> {
        if !self.editing {
            return None;
        }

        if self.entry() == EditEntry::Immediate && !self.is_dirty() {
            self.editing = false;
            self.field.buffer.clear_selection();
            return None;
        }
        match self.blur_action {
            BlurAction::Commit => {
                self.commit();
                Some(InputAction::Committed)
            }
            BlurAction::Cancel => {
                self.cancel();
                Some(InputAction::Cancelled)
            }
            BlurAction::Keep => None,
        }
    }

    pub fn clear(&mut self) {
        self.field.buffer.set_text("");
        self.snapshot.clear();
        self.snapshot_error = None;
        self.field.error = None;
        self.field.clear_history();
    }

    fn display_graphemes(&self, editing: bool, mask: &str) -> Vec<(usize, String, usize)> {
        let text = self.field.buffer.text();
        let gs: Vec<(usize, &str)> =
            unicode_segmentation::UnicodeSegmentation::grapheme_indices(text, true).collect();
        let n = gs.len();
        gs.into_iter()
            .enumerate()
            .map(|(i, (bi, g))| {
                let reveal = self.masked
                    && !editing
                    && self.reveal_tail > 0
                    && i + (self.reveal_tail as usize) >= n;
                if self.masked && !reveal {
                    (bi, mask.to_owned(), 1)
                } else {
                    (bi, g.to_owned(), width(g))
                }
            })
            .collect()
    }

    pub fn text(&self) -> &str {
        self.field.buffer.text()
    }

    pub fn begin_edit(&mut self) {
        if self.disabled || self.editing {
            return;
        }
        self.editing = true;
        self.focus_edit = false;
        self.snapshot = self.field.buffer.text().to_owned();
        self.snapshot_error = self.field.error.clone();
        self.field.buffer.clear_selection();
    }

    pub fn commit(&mut self) {
        self.editing = false;
        self.field.buffer.clear_selection();
        let changed = self.field.buffer.text() != self.snapshot;
        self.validate_ended_edit(changed);
    }

    pub fn cancel(&mut self) {
        self.editing = false;
        if self.field.buffer.text() != self.snapshot {
            self.field.record(EditKind::Other);
        }
        let snap = self.snapshot.clone();
        self.field.buffer.set_text(snap);

        let marked = self.snapshot_error.is_some() || self.field.error.is_some();
        self.validate_ended_edit(marked);
    }

    fn validate_ended_edit(&mut self, changed: bool) {
        if changed || self.field.error.is_some() || self.validate_on == ValidateOn::EveryCommit {
            self.validate();
        }
    }

    fn begin_focus_edit(&mut self) {
        if self.disabled || self.editing {
            return;
        }
        self.begin_edit();
        self.focus_select.apply(&mut self.field.buffer);
        self.focus_edit = true;
    }

    pub fn resume(&mut self) {
        if self.entry() == EditEntry::Immediate {
            self.begin_edit();
        }
    }

    pub fn validate(&mut self) -> bool {
        self.sync_required();
        self.field.validate()
    }

    fn sync_required(&mut self) {
        self.field.required = self.required.then(|| {
            self.required_message
                .clone()
                .unwrap_or_else(|| self.required_text.to_owned())
        });
    }

    pub const HEIGHT: u16 = 3;

    pub const GUTTER: u16 = 2;

    pub fn height(&self) -> u16 {
        2 + self.message_rows()
    }

    pub fn height_for(&self, cols: u16) -> u16 {
        self.label_lines(cols).len().max(1) as u16 + 1 + self.message_rows()
    }

    fn message_rows(&self) -> u16 {
        u16::from(self.field.error.is_some() || !self.help.is_empty())
    }

    fn label_lines(&self, cols: u16) -> Vec<String> {
        let room = cols.saturating_sub(Self::GUTTER) as usize;
        if self.wrap_label && room > 0 && width(&self.label) > room {
            crate::ui::text::wrap(&self.label, room)
        } else {
            vec![self.label.clone()]
        }
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<InputAction>) {
        if self.disabled {
            return (Outcome::Ignored, None);
        }

        if let Some(o) = edit_menu_key(self, key) {
            return (
                o,
                self.edit_menu.take_edited().then_some(InputAction::Changed),
            );
        }

        if key.ctrl_char('c') && (self.masked || self.field.buffer.selected_text().is_none()) {
            return (Outcome::Ignored, None);
        }
        let entry = self.entry();
        if !self.editing {
            match entry {
                EditEntry::Immediate if self.focus_seen => self.begin_edit(),
                EditEntry::Immediate => self.begin_focus_edit(),
                EditEntry::TypeToEdit | EditEntry::Explicit => {
                    if key.is(KeyCode::Enter) || key.is(KeyCode::F(2)) {
                        self.begin_edit();
                        return (Outcome::Changed, None);
                    }

                    if matches!(self.edit_action(key), EditAction::SelectAll) {
                        self.begin_edit();
                        self.field.buffer.select_all();
                        return (Outcome::Changed, None);
                    }

                    if matches!(self.edit_action(key), EditAction::Paste) {
                        let o = paste_clipboard(|t| self.on_paste(t));
                        return (o, (o == Outcome::Changed).then_some(InputAction::Changed));
                    }
                    if entry == EditEntry::TypeToEdit
                        && let KeyCode::Char(c) = key.code
                        && !key.chord()
                    {
                        self.begin_edit();
                        self.field.buffer.move_doc_end(false);
                        self.sync_required();
                        self.field.record(EditKind::Typing);
                        self.field.buffer.insert_char(c);
                        self.field.after_edit();
                        return (Outcome::Changed, Some(InputAction::Changed));
                    }
                    return (Outcome::Ignored, None);
                }
            }
        }
        let action = self.edit_action(key);
        match action {
            EditAction::Commit => {
                self.commit();
                self.resume();
                (Outcome::Changed, Some(InputAction::Committed))
            }
            EditAction::Cancel => {
                if entry == EditEntry::Immediate && !self.is_dirty() {
                    return (Outcome::Ignored, None);
                }
                self.cancel();
                self.resume();
                (Outcome::Changed, Some(InputAction::Cancelled))
            }
            EditAction::Tab { backward } => {
                self.commit();

                request_focus_next(backward);
                (
                    Outcome::Changed,
                    Some(InputAction::CommittedTab { backward }),
                )
            }

            EditAction::None if entry == EditEntry::Immediate => (Outcome::Ignored, None),

            EditAction::None => (crate::widgets::field_common::unbound_key(key), None),
            action => {
                self.sync_required();
                if self.field.apply(action, key).edited {
                    (Outcome::Changed, Some(InputAction::Changed))
                } else {
                    (Outcome::Changed, None)
                }
            }
        }
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        if self.disabled {
            return Outcome::Ignored;
        }
        if !self.editing {
            if self.entry() == EditEntry::Immediate {
                self.begin_focus_edit();
            } else {
                self.begin_edit();
            }
        }
        self.sync_required();
        self.field.paste(text);
        Outcome::Changed
    }

    fn offset_at(&self, pos: Position) -> usize {
        let glyphs = self.display_graphemes(true, crate::theme::Glyphs::UNICODE.mask);
        self.field.glyph_offset_at(pos, &glyphs)
    }

    pub fn label_hit(mut self, on: bool) -> Self {
        self.label_hit = on;
        self
    }

    fn on_label(&self, pos: Position) -> bool {
        self.label_hit && pos.y < self.area.y
    }

    pub fn on_click(&mut self, pos: Position) -> Outcome {
        self.on_click_count(pos, 1)
    }

    pub fn on_click_count(&mut self, pos: Position, count: u8) -> Outcome {
        PointerField::on_click_count(self, pos, count)
    }

    pub fn on_press(&mut self, pos: Position) -> Outcome {
        self.on_press_count(pos, 1)
    }

    pub fn on_press_count(&mut self, pos: Position, count: u8) -> Outcome {
        PointerField::on_press_count(self, pos, count)
    }

    pub fn on_drag(&mut self, pos: Position) -> Outcome {
        PointerField::on_drag(self, pos)
    }

    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: ratatui::style::Color,
    ) {
        self.render_field(area, buf, ctx, bg);

        self.edit_menu.render(self.area, buf, ctx);
    }

    fn render_field(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: ratatui::style::Color,
    ) {
        let required = ctx.theme.strings.required;
        if self.required_text != required {
            let old = std::mem::replace(&mut self.required_text, required);
            if self.required_message.is_none()
                && let Some(e) = self.field.error.as_mut()
            {
                crate::theme::Strings::follow(e.message_mut(), old, required);
            }
        }
        self.entry = self.edit_entry.unwrap_or(ctx.theme.edit_entry);
        self.field.keys = self.field_keys.or(ctx.theme.field_keys).unwrap_or_default();

        let has_focus = ctx.interaction.has_focus(self.id) && !ctx.inert;
        if has_focus && !self.focus_seen && self.entry == EditEntry::Immediate {
            self.begin_focus_edit();
        } else if !ctx.interaction.has_focus(self.id)
            && self.entry == EditEntry::Immediate
            && self.editing
            && self.focus_edit
            && !self.is_dirty()
        {
            self.editing = false;
            self.field.buffer.clear_selection();
        }
        self.focus_seen = has_focus;
        let area = area.intersection(*buf.area());
        self.area = Rect::ZERO;
        self.text_area = Rect::ZERO;
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let mut s = ctx.state(self.id);

        let menu_open = self.edit_menu.is_open();
        s.focused = ctx.interaction.focused_under(self.id, menu_open);
        s.disabled = self.disabled;
        s.editing = self.editing && (s.focused || (menu_open && has_focus));
        s.error = self.field.error.is_some();
        if self.disabled {
            s.hovered = false;
        }

        let wrapped = self.label_lines(area.width);
        let label_rows = wrapped.len().max(1) as u16;
        let label_x = area.x + Self::GUTTER.min(area.width);
        let label_style_for = |t: &crate::theme::Theme| {
            if self.disabled {
                t.faint().bg(bg)
            } else {
                t.label(s.focused).bg(bg)
            }
        };
        if label_rows > 1 {
            let st = label_style_for(t);
            for (i, line) in wrapped.iter().enumerate() {
                let y = area.y + i as u16;
                if y >= area.bottom() {
                    break;
                }
                buf.set_string(label_x, y, line, st);
            }
        }

        let mut label = self.label.clone();
        let name_w = width(&self.label);

        let show_optional = !self.required
            && !self.label.is_empty()
            && !self.plain_label
            && name_w + 4 + width(t.strings.optional) <= area.width as usize;
        if self.required {
            label.push_str(" *");
        } else if show_optional {
            label.push_str("  ");
            label.push_str(t.strings.optional);
        }
        let label_style = label_style_for(t);
        if label_rows == 1 {
            buf.set_string(
                label_x,
                area.y,
                crate::ui::text::fit(&label, area.width.saturating_sub(2) as usize),
                label_style,
            );
        }
        if label_rows > 1 {
        } else if self.required && !self.disabled && name_w + 4 <= area.width as usize {
            let ink = if self.field.error.is_some() {
                t.tone_on(crate::theme::Tone::Error, bg)
            } else {
                t.tone_on(crate::theme::Tone::Muted, bg)
            };
            buf.set_string(
                area.x + 2 + name_w as u16 + 1,
                area.y,
                "*",
                ratatui::style::Style::new().fg(ink).bg(bg),
            );
        } else if show_optional {
            buf.set_string(
                area.x + 2 + name_w as u16 + 2,
                area.y,
                t.strings.optional,
                t.faint().bg(bg),
            );
        }

        if area.height < label_rows + 1 {
            return;
        }
        let field = Rect::new(area.x, area.y + label_rows, area.width, 1);
        self.area = field;
        if self.label_hit && !self.label.is_empty() {
            let lw = if label_rows > 1 {
                wrapped.iter().map(|l| width(l)).max().unwrap_or(0)
            } else {
                width(&label)
            }
            .min(area.width.saturating_sub(Self::GUTTER) as usize) as u16;
            let rect = Rect::new(area.x, area.y, Self::GUTTER + lw, label_rows);
            ctx.clickable(self.id, rect.intersection(area));
        }
        let fs = t.field_style(s);
        crate::ui::ctx::fill(buf, field, fs);
        let gutter = t.gutter(s, fs.bg.unwrap_or(bg), false);
        buf.set_string(field.x, field.y, t.gutter_symbol(s), gutter);
        let trailing = if s.error { 2 } else { 0 };

        let readout_w = if self.readout.is_empty() {
            0
        } else {
            width(&self.readout) as u16 + 2
        };
        let readout_w = if field.width >= 3 + trailing + readout_w + 8 {
            readout_w
        } else {
            0
        };
        let inner = Rect::new(
            field.x + 2.min(field.width),
            field.y,
            field.width.saturating_sub(3 + trailing + readout_w),
            1,
        );
        self.readout_area = Rect::ZERO;
        self.text_area = inner;
        if !self.disabled && self.field.sel.drag {
            ctx.autoscroll(inner);
        }
        let text = self.field.buffer.text();
        if text.is_empty() {
            let p = crate::ui::reveal::fit(ctx, inner, &self.placeholder, s.focused);
            buf.set_string(inner.x, inner.y, &p, t.placeholder(s));
            if s.editing && inner.width > 0 {
                ctx.set_cursor(Position::new(inner.x, inner.y));
            }
        } else {
            let glyphs = self.display_graphemes(s.editing, t.glyphs.mask);
            let look = LineLook {
                text: fs,
                selection: t.selection(),
                ellipsis: fs.fg(t.text_muted),
                ellipsis_glyph: t.glyphs.ellipsis,
                underline: s.editing.then_some(t.accent),
                follow_caret: s.editing,
            };
            let caret = self.field.draw_glyphs(buf, inner, &look, &glyphs);
            if s.editing && inner.width > 0 {
                ctx.set_cursor(caret);
            }
        }
        if s.error && field.width >= 2 {
            buf.set_string(
                field.right() - 2,
                field.y,
                "!",
                fs.fg(t.error).add_modifier(Modifier::BOLD),
            );
        }
        ctx.control(self.id, field, self.disabled);

        if self.editing && !self.masked && self.field.buffer.has_selection() {
            ctx.selection(self.id);
        }
        if !self.disabled {
            ctx.pointer_shape(self.id, crate::core::shape::PointerShape::Text);
        }
        if readout_w > 0 {
            let rx = field.right().saturating_sub(trailing + readout_w);
            let r = Rect::new(rx, field.y, readout_w, 1);
            let fbg = fs.bg.unwrap_or(bg);

            let st = match self.readout_link {
                Some(id) if !self.disabled => ctx.title_style(id, fbg),
                _ => fs.fg(t.legible(t.text_muted, fbg)),
            };
            buf.set_string(rx, field.y, format!(" {} ", self.readout), st);
            self.readout_area = r;
            if let Some(id) = self.readout_link
                && !self.disabled
            {
                ctx.clickable(id, r);
            }
        }

        if area.height >= label_rows + 2 {
            let msg_y = field.y + 1;
            let msg = Rect::new(label_x, msg_y, area.width.saturating_sub(2), 1);
            if let Some(e) = &self.field.error {
                buf.set_string(
                    label_x,
                    msg_y,
                    crate::ui::reveal::fit(ctx, msg, e, s.focused),
                    t.error_fg().bg(bg),
                );
            } else if !self.help.is_empty() {
                buf.set_string(
                    label_x,
                    msg_y,
                    crate::ui::reveal::fit(ctx, msg, &self.help, s.focused),
                    t.muted().bg(bg),
                );
            }
        }
    }
}

impl crate::core::focus::BlurTarget for TextInput {
    fn blur_id(&self) -> WidgetId {
        self.id
    }
    fn blur(&mut self) -> Outcome {
        match self.on_blur() {
            Some(_) => Outcome::Changed,
            None => Outcome::Ignored,
        }
    }
}

impl Editable for TextInput {
    fn field(&mut self) -> &mut EditField {
        &mut self.field
    }
    fn edit_id(&self) -> WidgetId {
        self.id
    }
    fn offset_at(&self, pos: Position) -> usize {
        TextInput::offset_at(self, pos)
    }
    fn pointer_enabled(&self) -> bool {
        !self.disabled
    }
    fn whole_word(&self) -> bool {
        self.masked
    }
    fn on_label(&self, pos: Position) -> bool {
        TextInput::on_label(self, pos)
    }
    fn label_click(&mut self) {
        if !self.editing {
            self.begin_focus_edit();
        }
    }
    fn pointer_begin(&mut self) {
        self.begin_edit();
    }
    fn pointer_focus(&mut self) {
        self.focus_seen = true;
        self.focus_edit = false;
    }
}

impl crate::core::pointer::MouseTarget for TextInput {
    type Action = ();

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<()> {
        use crate::core::intent::Phase;
        let pos = ev.pos();
        if let Some(o) = edit_menu_pointer(self, ev) {
            return o.into();
        }
        match ev.phase() {
            Some(Phase::Press) => self.on_press_count(pos, ev.count().max(1)).into(),
            Some(Phase::Drag) => self.on_drag(pos).into(),
            Some(Phase::Click) => self.on_click_count(pos, ev.count().max(1)).into(),
            _ => crate::core::response::Response::ignored(),
        }
    }

    fn route(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> Option<crate::core::response::Response<()>> {
        edit_menu_route(self, ev)
    }
}

impl EditMenuHost for TextInput {
    fn edit_menu(&mut self) -> &mut EditMenu {
        &mut self.edit_menu
    }
    fn edit_owner(&self) -> WidgetId {
        self.id
    }
    fn edit_caps(&self) -> EditCaps {
        let mut caps = EditCaps::of(&self.field).secret(self.masked);
        caps.editable &= !self.disabled;
        caps
    }
    fn edit_shortcut(&self, cmd: TextCmd) -> Option<String> {
        self.field.shortcut(cmd)
    }
    fn edit_target(&self, ev: &crate::core::pointer::PointerEvent) -> bool {
        editable_target(self, ev)
    }
    fn edit_point(&mut self, pos: Position) {
        point_editable(self, pos);
    }
    fn edit_run(&mut self, cmd: TextCmd) -> bool {
        if self.masked && matches!(cmd, TextCmd::Cut | TextCmd::Copy) {
            return false;
        }
        if !self.editing {
            self.begin_edit();
        }
        self.sync_required();
        self.field.run(cmd).edited
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cut_help_line_reveals_on_hover_and_on_the_peek_line() {
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        let t = crate::theme::Theme::junie();
        let help = "Checked out after the clone finishes, before any hook runs";
        let mut f = TextInput::new(WidgetId::of("f"), "Branch").help(help);
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let area = Rect::new(0, 0, 24, 3);
        let mut buf = Buffer::empty(area);
        let mut ctx = RenderCtx::new(
            &t,
            crate::ui::ctx::Interaction {
                focus: Some(f.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        f.render(area, &mut buf, &mut ctx, t.canvas);
        assert_eq!(ctx.peek.as_deref(), Some(help));
        ctx.flush_overlays(&mut buf);
        drop(ctx);
        assert!(
            hits.hover_at(Position::new(4, 2)).is_some(),
            "the cut help hovers"
        );
    }

    #[test]
    fn measure_shows_label_value_and_help_whole() {
        use crate::core::focus::FocusRing;
        use crate::core::hit::HitRegistry;
        use crate::ui::measure::Measure;
        let t = crate::theme::Theme::junie();
        let mut f = TextInput::new(WidgetId::of("f"), "Branch")
            .value("feature/settlement-backoff-retry")
            .help("Checked out after the clone finishes");
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let size = {
            let ctx = RenderCtx::new(
                &t,
                crate::ui::ctx::Interaction::default(),
                &mut hits,
                &mut ring,
            );
            f.measure(ratatui::layout::Size::new(1, 1), &ctx)
        };
        let area = Rect::new(0, 0, size.width, size.height);
        let mut buf = Buffer::empty(area);
        let mut ctx = RenderCtx::new(
            &t,
            crate::ui::ctx::Interaction::default(),
            &mut hits,
            &mut ring,
        );
        f.render(area, &mut buf, &mut ctx, t.canvas);
        let text: String = (0..size.height)
            .map(|y| {
                (0..size.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        for part in [
            "Branch",
            "optional",
            "feature/settlement-backoff-retry",
            "Checked out after the clone finishes",
        ] {
            assert!(text.contains(part), "{part}:\n{text}");
        }
    }
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;

    fn render(input: &mut TextInput, area: Rect, buf: &mut Buffer) -> Option<Position> {
        let theme = Theme::for_level(ColorLevel::TrueColor);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(
            &theme,
            Interaction {
                focus: Some(input.id),
                ..Default::default()
            },
            &mut hits,
            &mut ring,
        );
        input.render(area, buf, &mut ctx, theme.surface);
        ctx.cursor
    }

    #[test]
    fn the_readout_sits_inside_the_field_and_narrows_the_text_run() {
        let row = |buf: &Buffer, y: u16| -> String {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_owned())
                .collect()
        };
        let area = Rect::new(0, 0, 40, 3);
        let mut plain = TextInput::new(WidgetId::of("q"), "Find").value("report");
        let mut buf = Buffer::empty(area);
        render(&mut plain, area, &mut buf);
        let full = plain.text_area.width;
        let mut input = TextInput::new(WidgetId::of("q"), "Find")
            .value("report")
            .readout("12 matches");
        let mut buf = Buffer::empty(area);
        render(&mut input, area, &mut buf);
        assert!(
            row(&buf, 1).trim_end().ends_with("12 matches"),
            "{}",
            row(&buf, 1)
        );
        assert_eq!(input.readout_area.right(), 40, "flush with the field's end");
        assert_eq!(input.text_area.width, full - 12, "the text run gives way");
        assert!(input.text_area.right() <= input.readout_area.x);

        let narrow = Rect::new(0, 0, 18, 3);
        let mut buf = Buffer::empty(narrow);
        render(&mut input, narrow, &mut buf);
        assert!(!row(&buf, 1).contains("matches"));
        assert_eq!(input.readout_area, Rect::ZERO);
    }

    #[test]
    fn a_readout_link_is_a_click_target_above_the_field() {
        let theme = Theme::for_level(ColorLevel::TrueColor);
        let link = WidgetId::of("q.scope");
        let mut input = TextInput::new(WidgetId::of("q"), "")
            .readout("scope ‹ here ›")
            .readout_link(link);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let area = Rect::new(0, 0, 40, 2);
        let mut buf = Buffer::empty(area);
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        input.render(area, &mut buf, &mut ctx, theme.surface);
        let r = input.readout_area;
        assert_eq!(hits.hit(Position::new(r.x + 2, r.y)), Some(link));
        assert_eq!(hits.hit(Position::new(3, r.y)), Some(input.id));

        let rest = buf[(r.x + 2, r.y)].fg;
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let hover = Interaction {
            hover: Some(link),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&theme, hover, &mut hits, &mut ring);
        input.render(area, &mut buf, &mut ctx, theme.surface);
        assert_ne!(buf[(r.x + 2, r.y)].fg, rest, "hover shows on the link");
    }

    #[test]
    fn blur_ends_the_edit_per_blur_action_and_reports_it() {
        let edited = |action| {
            let mut i = TextInput::new(WidgetId::of("b"), "Name")
                .value("old")
                .required(true)
                .blur_action(action);
            assert_eq!(
                i.on_blur(),
                None,
                "a field that is not editing ignores blur"
            );
            i.begin_edit();
            i.field.buffer.select_all();
            i.field.buffer.backspace();
            i
        };
        let mut commit = edited(BlurAction::default());
        assert_eq!(commit.on_blur(), Some(InputAction::Committed));
        assert!(!commit.editing);
        assert_eq!(commit.text(), "");
        assert!(commit.field.error.is_some(), "the commit validates");
        let mut cancel = edited(BlurAction::Cancel);
        assert_eq!(cancel.on_blur(), Some(InputAction::Cancelled));
        assert!(!cancel.editing);
        assert_eq!(cancel.text(), "old");
        let mut keep = edited(BlurAction::Keep);
        assert_eq!(keep.on_blur(), None);
        assert!(keep.editing, "the edit stays open for when focus returns");
        assert_eq!(keep.text(), "");
    }

    fn plain(c: char) -> Key {
        Key {
            code: KeyCode::Char(c),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        }
    }

    fn key(code: KeyCode, mods: ratatui::crossterm::event::KeyModifiers) -> Key {
        Key { code, mods }
    }

    fn bare(code: KeyCode) -> Key {
        key(code, ratatui::crossterm::event::KeyModifiers::NONE)
    }

    fn render_unfocused(input: &mut TextInput, area: Rect, buf: &mut Buffer) {
        let theme = Theme::for_level(ColorLevel::TrueColor);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        input.render(area, buf, &mut ctx, theme.surface);
    }

    #[test]
    fn ctrl_c_without_a_selection_is_left_to_the_app() {
        let ctrl_c = key(
            KeyCode::Char('c'),
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        );

        let mut input = TextInput::new(WidgetId::of("f"), "Name")
            .value("draft")
            .field_keys(FieldKeys::Desktop);
        input.begin_edit();
        input.field.buffer.clear_selection();
        assert_eq!(
            input.on_key(&ctrl_c).0,
            Outcome::Ignored,
            "reaches the quit guard"
        );
        assert!(input.editing, "the edit goes on");
        input.field.buffer.select_all();
        assert_ne!(
            input.on_key(&ctrl_c).0,
            Outcome::Ignored,
            "a selection is the field's"
        );
    }

    #[test]
    fn ctrl_c_copies_a_selection_under_every_preset_but_never_a_secret() {
        use crate::core::edit::clipboard;
        let ctrl_c = key(
            KeyCode::Char('c'),
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        );
        let marked = |input: &mut TextInput| {
            let theme = Theme::for_level(ColorLevel::TrueColor);
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 3));
            input.render(Rect::new(0, 0, 30, 3), &mut buf, &mut ctx, theme.surface);
            ring.has_selection(input.id)
        };
        let mut input = TextInput::new(WidgetId::of("f"), "Name")
            .value("draft")
            .field_keys(FieldKeys::Emacs);
        input.begin_edit();
        input.field.buffer.select_all();
        clipboard::copy("");
        assert_ne!(input.on_key(&ctrl_c).0, Outcome::Ignored);
        assert_eq!(clipboard::text().as_deref(), Some("draft"));
        assert!(marked(&mut input), "the frame reports the selection");

        let mut secret = TextInput::new(WidgetId::of("p"), "Password")
            .value("hunter2")
            .field_keys(FieldKeys::Desktop)
            .masked();
        secret.begin_edit();
        secret.field.buffer.select_all();
        clipboard::copy("");
        assert_eq!(secret.on_key(&ctrl_c).0, Outcome::Ignored);
        assert_eq!(clipboard::text().as_deref(), Some(""));
        assert!(!marked(&mut secret), "a hidden value reports no selection");
    }

    #[test]
    fn immediate_focus_shows_the_caret_and_selects_the_value() {
        let area = Rect::new(0, 0, 30, 3);
        let mut buf = Buffer::empty(area);
        let mut i = TextInput::new(WidgetId::of("i"), "Name")
            .value("ab")
            .edit_entry(EditEntry::Immediate);
        render_unfocused(&mut i, area, &mut buf);
        assert!(!i.editing, "no focus, no edit");
        let caret = render(&mut i, area, &mut buf);
        assert!(i.editing, "focus alone starts the edit");
        assert!(caret.is_some(), "the caret shows the moment focus lands");
        assert_eq!(
            i.field.buffer.selected_text(),
            Some("ab"),
            "Tab selects all"
        );

        i.on_key(&plain('x'));
        assert_eq!(i.text(), "x");
        assert_eq!(
            i.on_key(&bare(KeyCode::Enter)),
            (Outcome::Changed, Some(InputAction::Committed))
        );
        assert!(i.editing, "Enter applies; the field keeps the caret");

        i.on_key(&plain('y'));
        assert_eq!(
            i.on_key(&bare(KeyCode::Esc)).1,
            Some(InputAction::Cancelled)
        );
        assert_eq!(i.text(), "x", "reverted to the applied value");
        assert_eq!(
            i.on_key(&bare(KeyCode::Esc)),
            (Outcome::Ignored, None),
            "a clean Esc belongs to the page"
        );

        assert_eq!(i.on_blur(), None);
        assert!(!i.editing);
    }

    #[test]
    fn focus_select_places_the_caret_per_policy_and_a_click_wins() {
        let area = Rect::new(0, 0, 30, 3);
        let mut buf = Buffer::empty(area);
        let mut end = TextInput::new(WidgetId::of("e"), "Name")
            .value("abc")
            .edit_entry(EditEntry::Immediate)
            .focus_select(FocusSelect::CaretEnd);
        render(&mut end, area, &mut buf);
        assert_eq!(end.field.buffer.selection(), None);
        assert_eq!(end.field.buffer.cursor_offset(), 3);
        let mut start = TextInput::new(WidgetId::of("s"), "Name")
            .value("abc")
            .edit_entry(EditEntry::Immediate)
            .focus_select(FocusSelect::CaretStart);
        render(&mut start, area, &mut buf);
        assert_eq!(start.field.buffer.cursor_offset(), 0);

        let mut click = TextInput::new(WidgetId::of("c"), "Name")
            .value("abc")
            .edit_entry(EditEntry::Immediate);
        render_unfocused(&mut click, area, &mut buf);
        let ta = click.text_area;
        click.on_click(Position::new(ta.x + 1, ta.y));
        render(&mut click, area, &mut buf);
        assert_eq!(
            click.field.buffer.selection(),
            None,
            "a click never selects all"
        );
        assert_eq!(
            click.field.buffer.cursor_offset(),
            1,
            "caret at the click point"
        );
    }

    #[test]
    fn tab_applies_and_asks_the_ring_to_move_on() {
        let r = {
            let mut r = FocusRing::default();
            r.register(WidgetId::of("t"));
            r.register(WidgetId::of("next"));
            r
        };
        let mut f = crate::core::focus::Focus::default();
        f.focus(WidgetId::of("t"));
        f.take_changes();
        let mut i = TextInput::new(WidgetId::of("t"), "Name")
            .value("a")
            .edit_entry(EditEntry::Immediate);
        i.on_key(&plain('b'));
        let (o, ev) = i.on_key(&bare(KeyCode::Tab));
        assert_eq!(o, Outcome::Changed);
        assert_eq!(ev, Some(InputAction::CommittedTab { backward: false }));
        assert!(!i.editing);

        f.settle(&r);
        assert!(f.is(WidgetId::of("next")), "Tab lands on the next stop");
    }

    #[test]
    fn each_entry_mode_starts_the_edit_its_own_way() {
        let mut off = TextInput::new(WidgetId::of("off"), "Name")
            .value("ab")
            .edit_entry(EditEntry::Explicit);
        assert!(!off.wants_text_input());
        assert_eq!(off.on_key(&plain('x')), (Outcome::Ignored, None));
        assert!(!off.editing, "letters stay with the page");
        assert_eq!(off.on_key(&bare(KeyCode::Enter)).0, Outcome::Changed);
        assert!(off.editing, "Enter starts the edit");

        let mut on = TextInput::new(WidgetId::of("on"), "Name")
            .value("ab")
            .edit_entry(EditEntry::TypeToEdit);
        assert!(on.wants_text_input());
        assert_eq!(
            on.on_key(&plain('x')),
            (Outcome::Changed, Some(InputAction::Changed))
        );
        assert!(on.editing);
        assert_eq!(on.text(), "abx", "typing lands at the end");
        assert_eq!(
            on.on_key(&bare(KeyCode::Esc)).1,
            Some(InputAction::Cancelled)
        );
        assert_eq!(on.text(), "ab", "Esc reverts what typing started");
        let ctrl = key(
            KeyCode::Char('s'),
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        );
        assert_eq!(on.on_key(&ctrl).0, Outcome::Ignored, "shortcuts pass");

        #[allow(deprecated)]
        let legacy = TextInput::new(WidgetId::of("l"), "Name").type_to_edit(true);
        assert_eq!(legacy.entry(), EditEntry::TypeToEdit);

        let mut now = TextInput::new(WidgetId::of("n"), "Name")
            .value("ab")
            .edit_entry(EditEntry::Immediate);
        assert!(now.wants_text_input());
        now.on_key(&plain('q'));
        assert_eq!(now.text(), "q", "the selection is replaced, not a shortcut");
        let mut locked = TextInput::new(WidgetId::of("d"), "Name")
            .edit_entry(EditEntry::TypeToEdit)
            .disabled(true);
        assert!(!locked.wants_text_input());
        assert_eq!(locked.on_key(&plain('x')).0, Outcome::Ignored);
    }

    #[test]
    fn an_editing_field_passes_unbound_chords_on() {
        use ratatui::crossterm::event::KeyModifiers as M;
        for entry in [
            EditEntry::Immediate,
            EditEntry::TypeToEdit,
            EditEntry::Explicit,
        ] {
            for keys in [FieldKeys::Emacs, FieldKeys::Desktop] {
                let mut i = TextInput::new(WidgetId::of("c"), "Name")
                    .value("abc")
                    .edit_entry(entry)
                    .field_keys(keys);
                i.begin_edit();
                for (c, m) in [
                    ('q', M::CONTROL),
                    ('n', M::CONTROL),
                    ('x', M::ALT),
                    ('k', M::SUPER),
                ] {
                    assert_eq!(
                        i.on_key(&key(KeyCode::Char(c), m)),
                        (Outcome::Ignored, None),
                        "{entry:?} {keys:?}: {m:?}+{c} passes on"
                    );
                    assert!(i.editing, "the edit stays open");
                }
                assert_eq!(i.text(), "abc");

                assert_eq!(
                    i.on_key(&key(KeyCode::Char('a'), M::SUPER)).0,
                    Outcome::Changed,
                    "{entry:?} {keys:?}: Cmd+A"
                );
                let bound = match keys {
                    FieldKeys::Emacs => 'e',
                    FieldKeys::Desktop => 'a',
                };
                assert_eq!(
                    i.on_key(&key(KeyCode::Char(bound), M::CONTROL)).0,
                    Outcome::Changed,
                    "{entry:?} {keys:?}: Ctrl+{bound}"
                );

                if entry != EditEntry::Immediate {
                    assert_eq!(
                        i.on_key(&bare(KeyCode::F(5))).0,
                        Outcome::Consumed,
                        "{entry:?}: F5 stays with the edit"
                    );
                }
            }
        }
    }

    #[test]
    fn undo_redo_and_the_field_clipboard() {
        use ratatui::crossterm::event::KeyModifiers as M;
        let mut i = TextInput::new(WidgetId::of("u"), "Name")
            .edit_entry(EditEntry::Immediate)
            .focus_select(FocusSelect::CaretEnd)
            .field_keys(FieldKeys::Emacs);
        for c in "ab cd".chars() {
            i.on_key(&plain(c));
        }
        assert_eq!(i.text(), "ab cd");
        i.on_key(&key(KeyCode::Char('z'), M::CONTROL));
        assert_eq!(i.text(), "ab ", "one undo takes back the last word");
        i.on_key(&key(KeyCode::Char('z'), M::CONTROL | M::SHIFT));
        assert_eq!(i.text(), "ab cd", "Ctrl+Shift+Z redoes it");

        i.on_key(&key(KeyCode::Char('a'), M::CONTROL));
        assert_eq!(i.field.buffer.cursor_offset(), 0);
        i.on_key(&key(KeyCode::Char('l'), M::CONTROL));
        i.on_key(&key(KeyCode::Char('w'), M::ALT));
        assert_eq!(
            crate::core::edit::clipboard::text().as_deref(),
            Some("ab cd")
        );
        i.on_key(&key(KeyCode::Char('w'), M::CONTROL));
        assert_eq!(i.text(), "", "Ctrl+W cuts the selection");
        i.on_key(&key(KeyCode::Char('y'), M::CONTROL));
        assert_eq!(i.text(), "ab cd", "Ctrl+Y yanks it back");

        let mut d = TextInput::new(WidgetId::of("d"), "Name")
            .value("xyz")
            .field_keys(FieldKeys::Desktop);
        d.begin_edit();
        d.on_key(&key(KeyCode::Char('a'), M::CONTROL));
        assert_eq!(d.field.buffer.selected_text(), Some("xyz"));
        d.on_key(&key(KeyCode::Char('x'), M::CONTROL));
        assert_eq!(d.text(), "");
        d.on_key(&key(KeyCode::Char('v'), M::CONTROL));
        d.on_key(&key(KeyCode::Char('v'), M::CONTROL));
        assert_eq!(d.text(), "xyzxyz");
        d.on_key(&key(KeyCode::Char('z'), M::CONTROL));
        assert_eq!(d.text(), "xyz");
    }

    #[test]
    fn a_multi_line_paste_flattens_into_one_line() {
        let mut i = TextInput::new(WidgetId::of("p"), "Name").focus_select(FocusSelect::CaretEnd);
        i.on_paste("one\ntwo\r\nthree\n");
        assert_eq!(i.text(), "one two three");
    }

    #[test]
    fn drag_double_and_triple_click_select_text() {
        let mut i = TextInput::new(WidgetId::of("s"), "Name")
            .value("hello big world")
            .edit_entry(EditEntry::Explicit);
        let area = Rect::new(0, 0, 40, 3);
        let mut buf = Buffer::empty(area);
        render(&mut i, area, &mut buf);
        let ta = i.text_area;
        let at = |col: u16| Position::new(ta.x + col, ta.y);

        let (p, d) = (at(1), at(4));
        i.on_press(p);
        assert!(!i.editing, "the press alone changes nothing");
        assert_eq!(i.on_drag(d), Outcome::Changed);
        assert!(i.editing, "the drag edits");
        assert_eq!(i.field.buffer.selected_text(), Some("ell"));
        i.on_click_count(d, 1);
        assert_eq!(
            i.field.buffer.selected_text(),
            Some("ell"),
            "release keeps it"
        );

        i.on_click_count(at(2), 1);
        assert_eq!(i.field.buffer.selected_text(), None);

        i.on_click_count(at(7), 2);
        assert_eq!(i.field.buffer.selected_text(), Some("big"));
        i.on_click_count(at(7), 3);
        assert_eq!(i.field.buffer.selected_text(), Some("hello big world"));

        let mut m = TextInput::new(WidgetId::of("m"), "Key")
            .value("ab cd")
            .masked();
        render(&mut m, area, &mut buf);
        let p = Position::new(m.text_area.x, m.text_area.y);
        m.on_click_count(p, 2);
        assert_eq!(m.field.buffer.selected_text(), Some("ab cd"));

        let mut off = TextInput::new(WidgetId::of("o"), "Name")
            .value("hello big world")
            .pointer_select(PointerSelect::OFF);
        render(&mut off, area, &mut buf);
        let ta = off.text_area;
        let at = |col: u16| Position::new(ta.x + col, ta.y);
        let (p, d, w) = (at(1), at(4), at(7));
        off.on_press(p);
        assert_eq!(off.on_drag(d), Outcome::Ignored);
        off.on_click_count(w, 2);
        assert_eq!(off.field.buffer.selected_text(), None);
        assert_eq!(off.field.buffer.cursor(), 7);
    }

    #[test]
    fn a_drag_after_a_double_click_extends_by_words() {
        let mut i = TextInput::new(WidgetId::of("w"), "Name").value("alpha beta gamma delta");
        let area = Rect::new(0, 0, 40, 3);
        let mut buf = Buffer::empty(area);
        render(&mut i, area, &mut buf);
        let ta = i.text_area;
        let at = |col: u16| Position::new(ta.x + col, ta.y);

        i.on_click_count(at(7), 1);
        assert_eq!(i.on_press_count(at(7), 2), Outcome::Changed);
        assert_eq!(
            i.field.buffer.selected_text(),
            Some("beta"),
            "the press selects"
        );

        i.on_drag(at(12));
        assert_eq!(i.field.buffer.selected_text(), Some("beta gamma"));

        i.on_drag(at(1));
        assert_eq!(i.field.buffer.selected_text(), Some("alpha beta"));
        i.on_click_count(at(1), 2);
        assert_eq!(
            i.field.buffer.selected_text(),
            Some("alpha beta"),
            "release keeps"
        );

        let mut c = TextInput::new(WidgetId::of("c"), "Name")
            .value("alpha beta gamma delta")
            .pointer_select(PointerSelect::DEFAULT.drag_by_word(false));
        render(&mut c, area, &mut buf);
        c.on_click_count(at(7), 1);
        assert_eq!(c.on_press_count(at(7), 2), Outcome::Consumed);
        c.on_drag(at(12));
        assert_eq!(c.field.buffer.selected_text(), Some("eta g"));
    }

    #[test]
    fn dragging_past_the_left_edge_scrolls_the_field() {
        let long = "0123456789abcdefghijklmnopqrstuvwxyz";
        let mut i = TextInput::new(WidgetId::of("e"), "Name").value(long);
        let area = Rect::new(0, 0, 16, 3);
        let mut buf = Buffer::empty(area);
        render(&mut i, area, &mut buf);

        i.begin_edit();
        i.field.buffer.select_range(long.len(), long.len());
        i.field.buffer.clear_selection();
        render(&mut i, area, &mut buf);
        let first = i.field.line_scroll();
        assert!(first > 0, "the start is scrolled out");
        let ta = i.text_area;
        i.on_press(Position::new(ta.x + 3, ta.y));

        i.on_drag(Position::new(ta.x.saturating_sub(1), ta.y));
        render(&mut i, area, &mut buf);
        assert!(
            i.field.line_scroll() < first,
            "scrolled back: {} → {}",
            first,
            i.field.line_scroll()
        );
        let sel = i.field.buffer.selection().expect("a selection");
        assert!(sel.start < first, "selects text that was out of view");
    }

    #[test]
    fn rendering_without_focus_never_ends_the_edit() {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut i = TextInput::new(WidgetId::of("r"), "Name").value("v");
        i.begin_edit();
        i.field.buffer.insert_str("w");
        let area = Rect::new(0, 0, 30, 3);
        let mut buf = Buffer::empty(area);
        i.render(area, &mut buf, &mut ctx, theme.canvas);
        assert!(i.editing, "render is not a blur: on_blur ends the edit");
        assert_eq!(ctx.cursor, None, "an unfocused edit shows no caret");
        assert_eq!(i.on_blur(), Some(InputAction::Committed));
    }

    #[test]
    fn forward_blur_reaches_only_the_field_that_lost_focus() {
        use crate::core::focus::{FocusChange, forward_blur};
        let mut a = TextInput::new(WidgetId::of("a"), "A").value("a");
        let mut b = TextInput::new(WidgetId::of("b"), "B").value("b");
        a.begin_edit();

        a.field.buffer.insert_str("x");
        b.begin_edit();
        let change = FocusChange {
            from: a.id,
            to: Some(b.id),
        };
        assert_eq!(
            forward_blur(change, &mut [&mut a, &mut b]),
            Outcome::Changed
        );
        assert!(!a.editing);
        assert!(b.editing);
        assert_eq!(
            forward_blur(change, &mut [&mut a, &mut b]),
            Outcome::Ignored
        );
    }

    #[test]
    fn a_validator_closure_can_capture_state() {
        let allowed = ["admin".to_owned(), "member".to_owned()];
        let mut i = TextInput::new(WidgetId::of("role"), "Role").validator(move |v| {
            (!allowed.iter().any(|a| a == v)).then(|| format!("{v} is not a role"))
        });
        i = i.value("guest");
        assert!(!i.validate());
        assert_eq!(i.field.error.as_deref(), Some("guest is not a role"));
        i = i.value("admin");
        assert!(i.validate());
    }

    #[test]
    fn the_required_marker_is_neutral_until_the_field_is_invalid() {
        let theme = Theme::junie();
        let mut input = TextInput::new(WidgetId::of("required"), "Name").required(true);
        let area = Rect::new(0, 0, 30, 3);
        let mut buf = Buffer::empty(area);
        render(&mut input, area, &mut buf);

        let star = &buf[(2 + 4 + 1, 0)];
        assert_eq!(star.symbol(), "*");
        assert_eq!(star.fg, theme.text_muted, "the accent means focus");
        assert!(!input.validate());
        let mut buf = Buffer::empty(area);
        render(&mut input, area, &mut buf);
        assert_eq!(buf[(2 + 4 + 1, 0)].fg, theme.error);
    }

    #[test]
    fn passing_through_an_untouched_required_field_marks_no_error() {
        let mut input = TextInput::new(WidgetId::of("required"), "Name").required(true);

        input.begin_edit();
        input.commit();
        assert!(
            input.field.error.is_none(),
            "an untouched field is not invalid yet"
        );
        input.begin_edit();
        input.cancel();
        assert!(input.field.error.is_none());

        input.begin_edit();
        input.on_paste("x");
        input.commit();
        assert!(input.field.error.is_none());
        input.begin_edit();
        input.field.buffer.select_all();
        input.field.buffer.backspace();
        input.commit();
        assert!(input.field.error.is_some(), "an edit that empties it is");

        input.begin_edit();
        input.on_paste("ok");
        input.cancel();
        assert!(
            input.field.error.is_some(),
            "Esc brings the invalid value back"
        );
        input.begin_edit();
        input.on_paste("ok");
        input.commit();
        assert!(input.field.error.is_none());

        let mut input = TextInput::new(WidgetId::of("required"), "Name")
            .required(true)
            .validate_on(ValidateOn::EveryCommit);
        input.begin_edit();
        input.commit();
        assert!(input.field.error.is_some());
    }

    #[test]
    fn validate_when_decides_whether_typing_rechecks_the_value() {
        use crate::widgets::edit::ValidateWhen;
        let digits =
            |v: &str| (!v.chars().all(|c| c.is_ascii_digit())).then(|| "Digits only".to_owned());
        let ch = |c| Key {
            code: KeyCode::Char(c),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let bs = Key {
            code: KeyCode::Backspace,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let mut quiet = TextInput::new(WidgetId::of("q"), "Port").validator(digits);
        quiet.begin_edit();
        quiet.on_key(&ch('x'));
        assert_eq!(quiet.error(), None, "no error while typing");
        quiet.commit();
        assert_eq!(quiet.error(), Some("Digits only"), "the commit validates");
        quiet.begin_edit();
        quiet.on_key(&bs);
        assert_eq!(quiet.error(), None, "fixed: the error goes at once");
        let mut live = TextInput::new(WidgetId::of("l"), "Port")
            .validator(digits)
            .validate_when(ValidateWhen::Live);
        live.begin_edit();
        live.on_key(&ch('x'));
        assert_eq!(live.error(), Some("Digits only"), "Live: every keystroke");
        let mut commit = TextInput::new(WidgetId::of("c"), "Port")
            .validator(digits)
            .validate_when(ValidateWhen::Commit);
        commit.begin_edit();
        commit.on_key(&ch('x'));
        commit.commit();
        commit.begin_edit();
        commit.on_key(&bs);
        assert_eq!(commit.error(), Some("Digits only"), "Commit: kept");
    }

    #[test]
    fn required_error_clears_after_keyboard_and_paste_corrections() {
        let mut input = TextInput::new(WidgetId::of("required"), "Name").required(true);
        assert!(!input.validate());
        input.begin_edit();
        input.on_paste("valid");
        assert!(input.field.error.is_none());
        input.field.buffer.select_all();
        input.field.buffer.backspace();
        assert!(!input.validate());
        input.on_key(&Key {
            code: KeyCode::Char('x'),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        assert!(input.field.error.is_none());
    }

    #[test]
    fn one_click_enters_editing_at_the_pointer_and_a_disabled_field_ignores_it() {
        let mut input = TextInput::new(WidgetId::of("i"), "Name").value("hello");
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let area = Rect::new(0, 0, 30, 3);
        let mut buf = Buffer::empty(area);
        input.render(area, &mut buf, &mut ctx, theme.canvas);
        assert!(!input.editing);
        let o = input.on_click(Position::new(input.text_area.x + 2, 1));
        assert_eq!(o, Outcome::Changed);
        assert!(input.editing, "the first click edits");
        assert_eq!(
            input.field.buffer.cursor(),
            2,
            "the caret lands under the pointer"
        );

        input.on_click(Position::new(input.text_area.x + 4, 1));
        assert!(input.editing);
        assert_eq!(input.field.buffer.cursor(), 4);
        let mut locked = TextInput::new(WidgetId::of("d"), "Locked")
            .value("x")
            .disabled(true);
        locked.render(area, &mut buf, &mut ctx, theme.canvas);
        assert_eq!(locked.on_click(Position::new(2, 1)), Outcome::Consumed);
        assert!(!locked.editing);
    }

    #[test]
    fn masked_clicks_follow_display_graphemes() {
        let mut input = TextInput::new(WidgetId::of("masked"), "Secret")
            .value("日👩‍💻e\u{301}a")
            .masked();
        input.begin_edit();
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);
        render(&mut input, area, &mut buf);
        for (col, offset) in [(0, 0), (1, 3), (2, 14), (3, 17), (4, 18)] {
            input.on_click(Position::new(input.text_area.x + col, 1));
            assert_eq!(input.field.buffer.cursor_offset(), offset);
        }
    }

    #[test]
    fn scrolled_masked_clicks_and_cursor_share_geometry() {
        let mut input = TextInput::new(WidgetId::of("masked"), "Secret")
            .value("日本語👩‍💻e\u{301}")
            .masked();
        input.begin_edit();
        let area = Rect::new(0, 0, 6, 3);
        let mut buf = Buffer::empty(area);
        let cursor = render(&mut input, area, &mut buf).unwrap();
        assert!(input.text_area.contains(cursor));
        input.on_click(Position::new(input.text_area.x + 1, 1));
        assert_eq!(input.field.buffer.cursor_offset(), 20);
        assert_eq!(buf[(input.text_area.x + 1, 1)].symbol(), "•");
    }

    #[test]
    fn narrow_fields_stay_within_their_allocated_rectangle() {
        for width in 0..12 {
            for height in 0..4 {
                for invalid in [false, true] {
                    let mut input = TextInput::new(WidgetId::of("narrow"), "A long label")
                        .value("日本語👩‍💻abcdef")
                        .required(true)
                        .help("Some help");
                    if invalid {
                        input.field.error = Some("Invalid".into());
                    }
                    input.begin_edit();
                    let area = Rect::new(3, 2, width, height);
                    let mut buf = Buffer::empty(Rect::new(0, 0, 20, 8));
                    let cursor = render(&mut input, area, &mut buf);
                    assert!(cursor.is_none_or(|p| area.contains(p)));
                    for y in 0..8 {
                        for x in 0..20 {
                            if !area.contains(Position::new(x, y)) {
                                assert_eq!(buf[(x, y)].symbol(), " ", "{area:?}: ({x}, {y})");
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn wide_graphemes_keep_cursor_and_clicks_aligned_after_scroll() {
        let mut input = TextInput::new(WidgetId::of("wide"), "Name").value("日本語a");
        input.begin_edit();
        let area = Rect::new(0, 0, 8, 3);
        let mut buf = Buffer::empty(area);
        let cursor = render(&mut input, area, &mut buf).unwrap();
        assert!(input.text_area.contains(cursor));
        input.on_click(cursor);
        assert_eq!(input.field.buffer.cursor_offset(), input.text().len());
    }

    #[test]
    fn the_text_is_the_edge_autoscroll_body_unless_disabled() {
        let theme = Theme::junie();
        for disabled in [false, true] {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut w = TextInput::new(WidgetId::of("e"), "Name").value("hello");
            w.disabled = disabled;
            let area = Rect::new(0, 0, 20, 3);
            let mut buf = Buffer::empty(area);
            w.render(area, &mut buf, &mut ctx, theme.surface);
            let ta = w.text_area;
            let found = hits.autoscroll_at(Position::new(ta.x, ta.y));
            assert_eq!(found, (!disabled).then_some(ta), "disabled {disabled}");
        }
    }

    #[test]
    fn clicking_the_label_focuses_and_edits_the_field() {
        let theme = Theme::junie();
        let draw = |input: &mut TextInput| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 3));
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            input.render(Rect::new(0, 0, 30, 3), &mut buf, &mut ctx, theme.surface);
            drop(ctx);
            (hits, ring)
        };
        let mut input = TextInput::new(WidgetId::of("name"), "Name").value("Ada");
        let (hits, ring) = draw(&mut input);
        assert_eq!(
            hits.hit(Position::new(3, 0)),
            Some(input.id),
            "the label is a target"
        );
        assert_eq!(
            hits.hit(Position::new(28, 0)),
            None,
            "blank space after it is not"
        );
        assert!(ring.contains(input.id));
        input.on_press_count(Position::new(3, 0), 1);
        assert_eq!(input.on_click(Position::new(3, 0)), Outcome::Changed);
        assert!(input.editing, "a label click starts the edit");
        let mut off = TextInput::new(WidgetId::of("off"), "Name").label_hit(false);
        let (hits, _) = draw(&mut off);
        assert_eq!(hits.hit(Position::new(3, 0)), None);
    }
}
