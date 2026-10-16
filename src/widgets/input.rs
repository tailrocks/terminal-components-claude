use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Modifier;

use crate::core::edit::{EditEntry, EditKind, FieldKeys, FocusSelect, UndoStack};
use crate::core::event::{Key, Outcome};
use crate::core::focus::{BlurAction, request_focus_next};
use crate::core::id::WidgetId;
use crate::core::text::TextBuffer;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::width;
use crate::widgets::field_common::{
    Applied, EditAction, PointerField, PointerSelect, PressState, apply_edit, click_select,
    drag_select, edit_key_with, press_select,
};

/// A text validator: `Some(message)` rejects the value.
#[derive(Clone)]
pub struct TextValidator(pub std::rc::Rc<ValidateFn>);

/// The function a [`TextValidator`] wraps.
pub type ValidateFn = dyn Fn(&str) -> Option<String>;

impl std::fmt::Debug for TextValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TextValidator")
    }
}

/// Single-line text input.
///
/// How focus becomes an edit is the field's [`EditEntry`] (default: the
/// theme's, [`EditEntry::Immediate`]): focus starts the edit with the
/// caret shown at once ([`Self::focus_select`] selects the value, so
/// typing replaces it; a click puts the caret at the pointer), typing
/// edits, Enter applies, Tab applies and moves focus on through the ring
/// ([`request_focus_next`]), Esc reverts while the value is dirty and
/// otherwise passes on. Under [`EditEntry::Explicit`] the field has a
/// navigation mode (the gutter bar shows focus, Enter or F2 starts
/// editing) and an editing mode (caret shown, Enter commits, Esc reverts).
#[derive(Debug, Clone)]
pub struct TextInput {
    pub id: WidgetId,
    pub label: String,
    pub placeholder: String,
    pub buffer: TextBuffer,
    pub disabled: bool,
    pub required: bool,
    /// The error a required field left empty shows. `None` (default)
    /// follows the theme's [`crate::theme::Strings::required`].
    pub required_message: Option<String>,
    /// The theme's `required` text as of the last render.
    required_text: &'static str,
    pub help: String,
    pub error: Option<String>,
    pub editing: bool,
    /// Value before editing began, for Esc.
    snapshot: String,
    /// Horizontal scroll (display columns).
    scroll: usize,
    pub area: Rect,
    /// Area of the text run (inside the field), for click-to-cursor.
    text_area: Rect,
    /// Validation hook; a closure may capture state (an allowed list, a
    /// schema).
    pub validator: Option<TextValidator>,
    /// Hide the "optional" suffix on non-required fields.
    pub plain_label: bool,
    /// Draw every grapheme as `•`. `text()` still returns the raw value,
    /// which is transient edit state only: never log or render it.
    pub masked: bool,
    /// While not editing, show the last N graphemes in clear after the
    /// mask (`••••••••k7Qz`). Editing always masks everything.
    pub reveal_tail: u8,
    /// Wrap a label longer than the field onto several rows instead of
    /// cutting it with `…`. Off by default (form labels are short); a
    /// typed-acknowledgement prompt turns it on, because its label is the
    /// instruction and must be read whole. Measure with [`Self::height_for`].
    pub wrap_label: bool,
    /// What an edit in progress does when the field loses focus
    /// ([`Self::on_blur`]). Default [`BlurAction::Commit`].
    pub blur_action: BlurAction,
    /// How focus starts an edit. `None` (default) follows the theme's
    /// [`crate::theme::Theme::edit_entry`] ([`EditEntry::Immediate`]).
    pub edit_entry: Option<EditEntry>,
    /// The entry policy resolved at the last render (events use it).
    entry: EditEntry,
    /// Where the caret lands when the keyboard starts an immediate edit.
    /// Default [`FocusSelect::SelectAll`].
    pub focus_select: FocusSelect,
    /// The control-key preset. `None` (default) follows the theme's
    /// [`crate::theme::Theme::field_keys`] ([`FieldKeys::Emacs`]).
    pub field_keys: Option<FieldKeys>,
    /// The key preset resolved at the last render.
    keys: FieldKeys,
    /// Undo history of the value (Ctrl+Z, Ctrl+Shift+Z).
    undo: UndoStack,
    /// The field held focus at its last render, so a newly gained focus
    /// starts an immediate edit exactly once.
    focus_seen: bool,
    /// The edit in progress was started by focus (not by the pointer or
    /// Enter), so it may end quietly with the focus.
    focus_edit: bool,
    /// What a drag, double click and triple click select.
    /// Default [`PointerSelect::DEFAULT`].
    pub pointer_select: PointerSelect,
    press: PressState,
    /// A click on the label focuses the field and starts its edit (the
    /// HTML `<label>` convention, macOS form labels). Default `true`.
    pub label_hit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    Committed,
    Cancelled,
    /// Committed via Tab: caller should also move focus forward/backward.
    CommittedTab {
        backward: bool,
    },
    Changed,
}

impl TextInput {
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_owned(),
            placeholder: String::new(),
            buffer: TextBuffer::single(""),
            disabled: false,
            required: false,
            required_message: None,
            required_text: crate::theme::Strings::DEFAULT.required,
            help: String::new(),
            error: None,
            editing: false,
            snapshot: String::new(),
            scroll: 0,
            area: Rect::ZERO,
            text_area: Rect::ZERO,
            validator: None,
            plain_label: false,
            masked: false,
            reveal_tail: 0,
            wrap_label: false,
            blur_action: BlurAction::Commit,
            edit_entry: None,
            entry: crate::theme::Theme::DEFAULT_EDIT_ENTRY,
            focus_select: FocusSelect::default(),
            field_keys: None,
            keys: FieldKeys::default(),
            undo: UndoStack::new(),
            focus_seen: false,
            focus_edit: false,
            pointer_select: PointerSelect::DEFAULT,
            press: PressState::default(),
            label_hit: true,
        }
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.to_owned();
        self
    }

    /// See [`Self::required_message`].
    pub fn required_message(mut self, message: &str) -> Self {
        self.required_message = Some(message.to_owned());
        self
    }
    pub fn value(mut self, v: &str) -> Self {
        self.buffer.set_text(v);
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
        self.validator = Some(TextValidator(std::rc::Rc::new(v)));
        self
    }
    pub fn masked(mut self) -> Self {
        self.masked = true;
        self
    }
    pub fn reveal_tail(mut self, n: u8) -> Self {
        self.reveal_tail = n;
        self
    }
    /// See [`Self::wrap_label`].
    pub fn wrap_label(mut self, on: bool) -> Self {
        self.wrap_label = on;
        self
    }
    /// See [`Self::blur_action`].
    pub fn blur_action(mut self, action: BlurAction) -> Self {
        self.blur_action = action;
        self
    }
    /// See [`Self::edit_entry`].
    pub fn edit_entry(mut self, entry: EditEntry) -> Self {
        self.set_edit_entry(entry);
        self
    }
    /// [`Self::edit_entry`] on a field already built.
    pub fn set_edit_entry(&mut self, entry: EditEntry) {
        self.edit_entry = Some(entry);
        self.entry = entry;
    }
    /// The former type-to-edit switch: `true` is
    /// [`EditEntry::TypeToEdit`], `false` [`EditEntry::Explicit`].
    #[deprecated(note = "use `edit_entry(EditEntry::TypeToEdit)` or `EditEntry::Explicit`")]
    pub fn type_to_edit(self, on: bool) -> Self {
        self.edit_entry(if on {
            EditEntry::TypeToEdit
        } else {
            EditEntry::Explicit
        })
    }
    /// See [`Self::focus_select`].
    pub fn focus_select(mut self, select: FocusSelect) -> Self {
        self.focus_select = select;
        self
    }
    /// See [`Self::field_keys`].
    pub fn field_keys(mut self, keys: FieldKeys) -> Self {
        self.field_keys = Some(keys);
        self.keys = keys;
        self
    }
    /// The entry policy in effect (the field's own, or the theme's as of
    /// the last render).
    pub fn entry(&self) -> EditEntry {
        self.edit_entry.unwrap_or(self.entry)
    }
    /// The focused field takes printable keys before page and app letter
    /// shortcuts: it is editing, or focus alone makes it edit (or typing
    /// starts the edit). Applications ask the focused control this before
    /// running a plain-letter shortcut.
    pub fn wants_text_input(&self) -> bool {
        !self.disabled && (self.editing || self.entry() != EditEntry::Explicit)
    }
    /// The value differs from the one the edit began with.
    pub fn is_dirty(&self) -> bool {
        self.editing && self.buffer.text() != self.snapshot
    }
    /// Something to undo in this field.
    pub fn can_undo(&self) -> bool {
        self.undo.can_undo()
    }
    /// The footer hints for this field while it holds focus: the edit keys
    /// ([`crate::widgets::keyhint::EDIT_FIELD`]) once focus edits, `Enter
    /// Edit` while an [`EditEntry::Explicit`] field waits.
    pub fn hints(&self) -> Vec<crate::widgets::keyhint::Hint> {
        use crate::widgets::keyhint::{EDIT_FIELD, hint};
        if self.disabled {
            Vec::new()
        } else if self.wants_text_input() {
            EDIT_FIELD.to_vec()
        } else {
            vec![hint("Enter", "Edit")]
        }
    }
    /// See [`Self::pointer_select`].
    pub fn pointer_select(mut self, sel: PointerSelect) -> Self {
        self.pointer_select = sel;
        self
    }

    /// Focus left the field (see [`crate::core::focus::Focus::take_changes`]).
    /// An edit in progress ends per [`Self::blur_action`] and reports the
    /// same event Enter or Esc would, so the owner's validation and save
    /// hooks run; a field that is not editing returns `None`.
    pub fn on_blur(&mut self) -> Option<InputEvent> {
        if !self.editing {
            return None;
        }
        // an edit focus started and nothing changed ends quietly: tabbing
        // through a form is not a round of commits
        if self.entry() == EditEntry::Immediate && !self.is_dirty() {
            self.editing = false;
            self.buffer.clear_selection();
            return None;
        }
        match self.blur_action {
            BlurAction::Commit => {
                self.commit();
                Some(InputEvent::Committed)
            }
            BlurAction::Cancel => {
                self.cancel();
                Some(InputEvent::Cancelled)
            }
            BlurAction::Keep => None,
        }
    }

    /// Overwrite and drop the value (owners clear secrets this way).
    pub fn clear(&mut self) {
        self.buffer.set_text("");
        self.snapshot.clear();
        self.error = None;
        self.undo.clear();
    }

    /// Graphemes as they will be drawn: raw, or `mask` (one cell, `•` by
    /// default) per grapheme when masked (the tail stays in clear while not
    /// editing).
    fn display_graphemes(&self, editing: bool, mask: &str) -> Vec<(usize, String, usize)> {
        let text = self.buffer.text();
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
        self.buffer.text()
    }

    pub fn begin_edit(&mut self) {
        if self.disabled || self.editing {
            return;
        }
        self.editing = true;
        self.focus_edit = false;
        self.snapshot = self.buffer.text().to_owned();
        self.buffer.clear_selection();
    }

    pub fn commit(&mut self) {
        self.editing = false;
        self.buffer.clear_selection();
        self.validate();
    }

    pub fn cancel(&mut self) {
        self.editing = false;
        if self.buffer.text() != self.snapshot {
            // the revert is one more step back: Ctrl+Z brings the text back
            self.undo.record(&self.buffer, EditKind::Other);
        }
        let snap = self.snapshot.clone();
        self.buffer.set_text(snap);
        self.validate();
    }

    /// Start the edit focus gained ([`EditEntry::Immediate`]): the value
    /// is selected or the caret placed per [`Self::focus_select`].
    fn begin_focus_edit(&mut self) {
        if self.disabled || self.editing {
            return;
        }
        self.begin_edit();
        self.focus_select.apply(&mut self.buffer);
        self.focus_edit = true;
    }

    /// After Enter applied (or Esc reverted) an immediate edit, the field
    /// keeps the caret: a new edit starts from the applied value. Owners
    /// that end the edit themselves (a submit the validation blocked) call
    /// it to hand the caret back.
    pub fn resume(&mut self) {
        if self.entry() == EditEntry::Immediate {
            self.begin_edit();
        }
    }

    /// Recompute validation from the current value. A custom validator owns
    /// validation when supplied; otherwise `required` rejects an empty value.
    pub fn validate(&mut self) -> bool {
        self.error = if let Some(v) = &self.validator {
            (v.0)(self.buffer.text())
        } else if self.required && self.buffer.is_empty() {
            Some(
                self.required_message
                    .clone()
                    .unwrap_or_else(|| self.required_text.to_owned()),
            )
        } else {
            None
        };
        self.error.is_none()
    }

    /// Height of the tallest single-label field: label + field +
    /// (help|error) line. Layouts that reserve a fixed slot use it; a
    /// layout that measures uses [`Self::height`] or [`Self::height_for`].
    pub const HEIGHT: u16 = 3;

    /// Columns the field's focus gutter takes left of the label and the
    /// text. Render the field at `x - GUTTER` to line its label up with
    /// text that starts at `x` (a dialog title, a paragraph).
    pub const GUTTER: u16 = 2;

    /// Rows this field draws: the label, the field, and the help or error
    /// row only when there is something to show in it.
    pub fn height(&self) -> u16 {
        2 + self.message_rows()
    }

    /// [`Self::height`] for a field `width` columns wide, counting every
    /// row of a wrapped label ([`Self::wrap_label`]).
    pub fn height_for(&self, cols: u16) -> u16 {
        self.label_lines(cols).len().max(1) as u16 + 1 + self.message_rows()
    }

    fn message_rows(&self) -> u16 {
        u16::from(self.error.is_some() || !self.help.is_empty())
    }

    /// The label as drawn in a field `width` wide: one line, unless the
    /// label wraps and is longer than the room beside the gutter.
    fn label_lines(&self, cols: u16) -> Vec<String> {
        let room = cols.saturating_sub(Self::GUTTER) as usize;
        if self.wrap_label && room > 0 && width(&self.label) > room {
            crate::ui::text::wrap(&self.label, room)
        } else {
            vec![self.label.clone()]
        }
    }

    /// Handle a key while focused. The caller decides what to do with the
    /// returned event (e.g. move focus on `CommittedTab`).
    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<InputEvent>) {
        if self.disabled {
            return (Outcome::Ignored, None);
        }
        let entry = self.entry();
        if !self.editing {
            match entry {
                // a key before the first render after the focus move starts
                // the focus edit here; after an edit ended in place (a click
                // on dead space) typing resumes at the caret
                EditEntry::Immediate if self.focus_seen => self.begin_edit(),
                EditEntry::Immediate => self.begin_focus_edit(),
                EditEntry::TypeToEdit | EditEntry::Explicit => {
                    if key.is(KeyCode::Enter) || key.is(KeyCode::F(2)) {
                        self.begin_edit();
                        return (Outcome::Changed, None);
                    }
                    if entry == EditEntry::TypeToEdit
                        && let KeyCode::Char(c) = key.code
                        && !key.ctrl()
                        && !key.alt()
                    {
                        self.begin_edit();
                        self.buffer.move_doc_end(false);
                        self.undo.record(&self.buffer, EditKind::Typing);
                        self.buffer.insert_char(c);
                        self.live_validate();
                        return (Outcome::Changed, Some(InputEvent::Changed));
                    }
                    return (Outcome::Ignored, None);
                }
            }
        }
        let action = edit_key_with(key, false, self.keys);
        match action {
            EditAction::Commit => {
                self.commit();
                self.resume();
                (Outcome::Changed, Some(InputEvent::Committed))
            }
            EditAction::Cancel => {
                if entry == EditEntry::Immediate && !self.is_dirty() {
                    // nothing to revert: Esc belongs to the page (close,
                    // back, clear)
                    return (Outcome::Ignored, None);
                }
                self.cancel();
                self.resume();
                (Outcome::Changed, Some(InputEvent::Cancelled))
            }
            EditAction::Tab { backward } => {
                self.commit();
                // the ring moves on whether or not the owner does
                request_focus_next(backward);
                (
                    Outcome::Changed,
                    Some(InputEvent::CommittedTab { backward }),
                )
            }
            // a key the field has no use for (↑↓, PgUp, an unbound chord)
            // passes on from an immediate edit, so a list under a search
            // field and the page's shortcuts still answer it
            EditAction::None if entry == EditEntry::Immediate => (Outcome::Ignored, None),
            EditAction::None => (Outcome::Consumed, None),
            action => match apply_edit(&mut self.buffer, &mut self.undo, action, key.code, false) {
                Some(Applied::Edited) => {
                    self.live_validate();
                    (Outcome::Changed, Some(InputEvent::Changed))
                }
                _ => (Outcome::Changed, None),
            },
        }
    }

    /// Insert pasted text (only while editing).
    /// A paste is typing: a focused field that is not editing yet starts
    /// editing, exactly as the first typed character would.
    /// A multi-line paste flattens: its line breaks become spaces (a
    /// trailing break is dropped), so words never run together.
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
        let flat = text
            .trim_end_matches(['\r', '\n'])
            .replace("\r\n", " ")
            .replace(['\r', '\n'], " ");
        self.undo.record(&self.buffer, EditKind::Other);
        self.buffer.insert_str(&flat);
        self.live_validate();
        Outcome::Changed
    }

    fn live_validate(&mut self) {
        if self.error.is_some() {
            self.validate();
        }
    }

    /// Byte offset of the grapheme under `pos`. Mouse positions use the
    /// same display geometry as rendering: a masked CJK/emoji grapheme
    /// occupies one cell, regardless of raw width.
    fn offset_at(&self, pos: Position) -> usize {
        // a pointer left of the text names the column before the first
        // visible one, so a drag past the edge scrolls the field
        let col =
            (self.scroll as isize + (pos.x as isize - self.text_area.x as isize)).max(0) as usize;
        let mut width = 0;
        for (bi, _, gw) in self.display_graphemes(true, crate::theme::Glyphs::UNICODE.mask) {
            if width + gw > col {
                return bi;
            }
            width += gw;
        }
        self.buffer.text().len()
    }

    /// See [`Self::label_hit`].
    pub fn label_hit(mut self, on: bool) -> Self {
        self.label_hit = on;
        self
    }

    /// `pos` is on the label above the field row ([`Self::label_hit`]).
    fn on_label(&self, pos: Position) -> bool {
        self.label_hit && pos.y < self.area.y
    }

    /// Mouse click on the field. Focus is handled by the app; the click
    /// enters editing (one click, never two) and places the cursor at the
    /// pointer. Same as [`Self::on_click_count`] with a count of 1.
    pub fn on_click(&mut self, pos: Position) -> Outcome {
        self.on_click_count(pos, 1)
    }

    /// A completed click with its multi-click `count`
    /// ([`crate::core::pointer::PointerState::count_click`]): 1 places the
    /// caret, 2 selects the word (the whole value when masked), 3 the
    /// line, per [`Self::pointer_select`]. The release that ends a drag
    /// keeps the dragged selection.
    pub fn on_click_count(&mut self, pos: Position, count: u8) -> Outcome {
        if self.disabled {
            return Outcome::Consumed;
        }
        if self.on_label(pos) {
            // a label click edits as focus would, without placing a caret
            if !self.editing {
                self.begin_focus_edit();
            }
            return Outcome::Changed;
        }
        self.begin_edit();
        let offset = self.offset_at(pos);
        click_select(
            &mut self.buffer,
            &mut self.press,
            self.pointer_select,
            offset,
            count,
            self.masked,
        );
        Outcome::Changed
    }

    /// Button down on the field: remembers where, so a drag selects from
    /// there. The press alone changes nothing; the completed click (or the
    /// drag) starts editing.
    pub fn on_press(&mut self, pos: Position) -> Outcome {
        self.on_press_count(pos, 1)
    }

    /// A press with the click `count` it starts
    /// ([`crate::core::pointer::PointerState::press_count`]): the second
    /// press of a double click selects the word, and a drag from it
    /// extends by words ([`PointerSelect::drag_by_word`]).
    pub fn on_press_count(&mut self, pos: Position, count: u8) -> Outcome {
        if self.disabled || self.on_label(pos) {
            return Outcome::Consumed;
        }
        // the pointer owns this focus: the click places the caret, the
        // keyboard's focus selection does not apply
        self.focus_seen = true;
        self.focus_edit = false;
        let offset = self.offset_at(pos);
        if press_select(
            &mut self.buffer,
            &mut self.press,
            self.pointer_select,
            offset,
            count,
            self.masked,
        ) {
            self.begin_edit();
            return Outcome::Changed;
        }
        Outcome::Consumed
    }

    /// Drag with the button held since a press on the field: starts
    /// editing and selects from the press to the pointer
    /// ([`PointerSelect::drag`]). Past the field's left or right edge the
    /// selection runs on and the field scrolls to follow it.
    pub fn on_drag(&mut self, pos: Position) -> Outcome {
        if self.disabled || !self.pointer_select.drag {
            return Outcome::Ignored;
        }
        let offset = self.offset_at(pos);
        self.begin_edit();
        drag_select(&mut self.buffer, &mut self.press, offset);
        Outcome::Changed
    }

    pub fn render(
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
                && let Some(e) = self.error.as_mut()
            {
                crate::theme::Strings::follow(e, old, required);
            }
        }
        self.entry = self.edit_entry.unwrap_or(ctx.theme.edit_entry);
        self.keys = self.field_keys.unwrap_or(ctx.theme.field_keys);
        // focus newly gained starts an immediate edit, so the caret shows
        // the moment Tab lands here
        let has_focus = ctx.interaction.has_focus(self.id) && !ctx.inert;
        if has_focus && !self.focus_seen && self.entry == EditEntry::Immediate {
            self.begin_focus_edit();
        } else if !ctx.interaction.has_focus(self.id)
            && self.entry == EditEntry::Immediate
            && self.editing
            && self.focus_edit
            && !self.is_dirty()
        {
            // a clean edit focus started ends with the focus even when the
            // owner never forwarded the blur (nothing to commit or report)
            self.editing = false;
            self.buffer.clear_selection();
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
        s.disabled = self.disabled;
        s.editing = self.editing && s.focused;
        s.error = self.error.is_some();
        if self.disabled {
            s.hovered = false;
        }
        // An edit that lost focus is ended by `on_blur` on the event path,
        // never here (`s.editing` needs focus); render only starts the
        // edit a newly gained focus asks for (above).

        // a wrapped label takes as many rows as it needs, every one whole
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
        // label row
        let mut label = self.label.clone();
        let name_w = width(&self.label);
        // the "optional" suffix only appears when the field is wide enough to
        // hold it whole; a clipped suffix reads worse than none
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
            // the wrapped label carries no suffix marks
        } else if self.required && !self.disabled && name_w + 4 <= area.width as usize {
            buf.set_string(
                area.x + 2 + name_w as u16 + 1,
                area.y,
                "*",
                t.accent_fg().bg(bg),
            );
        } else if show_optional {
            buf.set_string(
                area.x + 2 + name_w as u16 + 2,
                area.y,
                t.strings.optional,
                t.faint().bg(bg),
            );
        }

        // field row
        if area.height < label_rows + 1 {
            return;
        }
        let field = Rect::new(area.x, area.y + label_rows, area.width, 1);
        self.area = field;
        if self.label_hit && !self.label.is_empty() {
            // the label is the field's own target: clicking it focuses and
            // edits the field
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
        let inner = Rect::new(
            field.x + 2.min(field.width),
            field.y,
            field.width.saturating_sub(3 + trailing),
            1,
        );
        self.text_area = inner;
        if !self.disabled && self.pointer_select.drag {
            ctx.autoscroll(inner);
        }
        let text = self.buffer.text();
        if text.is_empty() {
            // the placeholder stays until the first character, with the
            // caret on it while editing (macOS)
            let p = crate::ui::text::truncate(&self.placeholder, inner.width as usize);
            buf.set_string(inner.x, inner.y, &p, t.placeholder(s));
            if s.editing && inner.width > 0 {
                ctx.set_cursor(Position::new(inner.x, inner.y));
            }
        } else {
            // horizontal scroll so the cursor stays visible
            let glyphs = self.display_graphemes(s.editing, t.glyphs.mask);
            let cursor_off = self.buffer.cursor_offset();
            let cursor_col: usize = glyphs
                .iter()
                .filter(|(bi, _, _)| *bi < cursor_off)
                .map(|(_, _, w)| *w)
                .sum();
            let total: usize = glyphs.iter().map(|(_, _, w)| *w).sum();
            let w = inner.width as usize;
            if w > 0 {
                if cursor_col < self.scroll {
                    self.scroll = cursor_col;
                } else if cursor_col >= self.scroll + w {
                    self.scroll = cursor_col + 1 - w;
                }
                if !s.editing {
                    self.scroll = 0;
                }
                // The insertion cursor needs a cell after the final glyph.
                self.scroll = self.scroll.min(total.saturating_add(1).saturating_sub(w));
                let mut boundary = 0;
                for (_, _, gw) in &glyphs {
                    if boundary >= self.scroll {
                        break;
                    }
                    boundary += gw;
                }
                self.scroll = boundary.min(cursor_col);
            }
            let sel = self.buffer.selection();
            let mut col = 0usize;
            let clipped_right = total > self.scroll.saturating_add(w);
            let visible_width = w.saturating_sub(usize::from(clipped_right));
            for (bi, g, gw) in &glyphs {
                let gw = *gw;
                if col + gw <= self.scroll {
                    col += gw;
                    continue;
                }
                let displayed_col = col.saturating_sub(self.scroll);
                if displayed_col + gw > visible_width {
                    break;
                }
                let x = inner.x + displayed_col as u16;
                let mut st = fs;
                if let Some(r) = &sel
                    && r.contains(bi)
                {
                    st = t.selection();
                }
                if s.editing {
                    st = st
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(t.accent);
                }
                if displayed_col == 0 && self.scroll > 0 && gw > 0 {
                    buf.set_string(x, inner.y, t.glyphs.ellipsis, fs.fg(t.text_muted));
                    col += gw;
                    continue;
                }
                buf.set_string(x, inner.y, g, st);
                col += gw;
            }
            if clipped_right && inner.width > 0 {
                buf.set_string(
                    inner.right() - 1,
                    inner.y,
                    t.glyphs.ellipsis,
                    fs.fg(t.text_muted),
                );
            }
            if s.editing && inner.width > 0 {
                let cx = inner.x + (cursor_col - self.scroll) as u16;
                ctx.set_cursor(Position::new(cx.min(inner.right() - 1), inner.y));
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
        if !self.disabled {
            ctx.pointer_shape(self.id, crate::core::shape::PointerShape::Text);
        }

        // help / error row
        if area.height >= label_rows + 2 {
            let msg_y = field.y + 1;
            if let Some(e) = &self.error {
                buf.set_string(
                    label_x,
                    msg_y,
                    crate::ui::text::truncate(e, area.width.saturating_sub(2) as usize),
                    t.error_fg().bg(bg),
                );
            } else if !self.help.is_empty() {
                buf.set_string(
                    label_x,
                    msg_y,
                    crate::ui::text::truncate(&self.help, area.width.saturating_sub(2) as usize),
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

impl PointerField for TextInput {
    fn pointer_id(&self) -> WidgetId {
        self.id
    }
    fn on_press_count(&mut self, pos: Position, count: u8) -> Outcome {
        TextInput::on_press_count(self, pos, count)
    }
    fn on_drag(&mut self, pos: Position) -> Outcome {
        TextInput::on_drag(self, pos)
    }
    fn on_click_count(&mut self, pos: Position, count: u8) -> Outcome {
        TextInput::on_click_count(self, pos, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            i.buffer.select_all();
            i.buffer.backspace();
            i
        };
        let mut commit = edited(BlurAction::default());
        assert_eq!(commit.on_blur(), Some(InputEvent::Committed));
        assert!(!commit.editing);
        assert_eq!(commit.text(), "");
        assert!(commit.error.is_some(), "the commit validates");
        let mut cancel = edited(BlurAction::Cancel);
        assert_eq!(cancel.on_blur(), Some(InputEvent::Cancelled));
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

    /// Render with focus somewhere else, so the next render with focus is
    /// a focus gain.
    fn render_unfocused(input: &mut TextInput, area: Rect, buf: &mut Buffer) {
        let theme = Theme::for_level(ColorLevel::TrueColor);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        input.render(area, buf, &mut ctx, theme.surface);
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
        assert_eq!(i.buffer.selected_text(), Some("ab"), "Tab selects all");
        // typing replaces the selection, Enter applies and keeps the caret
        i.on_key(&plain('x'));
        assert_eq!(i.text(), "x");
        assert_eq!(
            i.on_key(&bare(KeyCode::Enter)),
            (Outcome::Changed, Some(InputEvent::Committed))
        );
        assert!(i.editing, "Enter applies; the field keeps the caret");
        // Esc reverts while dirty, then passes on
        i.on_key(&plain('y'));
        assert_eq!(i.on_key(&bare(KeyCode::Esc)).1, Some(InputEvent::Cancelled));
        assert_eq!(i.text(), "x", "reverted to the applied value");
        assert_eq!(
            i.on_key(&bare(KeyCode::Esc)),
            (Outcome::Ignored, None),
            "a clean Esc belongs to the page"
        );
        // a clean blur is quiet
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
        assert_eq!(end.buffer.selection(), None);
        assert_eq!(end.buffer.cursor_offset(), 3);
        let mut start = TextInput::new(WidgetId::of("s"), "Name")
            .value("abc")
            .edit_entry(EditEntry::Immediate)
            .focus_select(FocusSelect::CaretStart);
        render(&mut start, area, &mut buf);
        assert_eq!(start.buffer.cursor_offset(), 0);
        // pointer focus: the click lands first, the render keeps its caret
        let mut click = TextInput::new(WidgetId::of("c"), "Name")
            .value("abc")
            .edit_entry(EditEntry::Immediate);
        render_unfocused(&mut click, area, &mut buf);
        let ta = click.text_area;
        click.on_click(Position::new(ta.x + 1, ta.y));
        render(&mut click, area, &mut buf);
        assert_eq!(click.buffer.selection(), None, "a click never selects all");
        assert_eq!(click.buffer.cursor_offset(), 1, "caret at the click point");
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
        assert_eq!(ev, Some(InputEvent::CommittedTab { backward: false }));
        assert!(!i.editing);
        // the owner did nothing with the event: the ring still moves
        f.settle(&r);
        assert!(f.is(WidgetId::of("next")), "Tab lands on the next stop");
    }

    #[test]
    fn each_entry_mode_starts_the_edit_its_own_way() {
        // Explicit: today's navigation mode
        let mut off = TextInput::new(WidgetId::of("off"), "Name")
            .value("ab")
            .edit_entry(EditEntry::Explicit);
        assert!(!off.wants_text_input());
        assert_eq!(off.on_key(&plain('x')), (Outcome::Ignored, None));
        assert!(!off.editing, "letters stay with the page");
        assert_eq!(off.on_key(&bare(KeyCode::Enter)).0, Outcome::Changed);
        assert!(off.editing, "Enter starts the edit");
        // TypeToEdit: the first character starts it, at the end
        let mut on = TextInput::new(WidgetId::of("on"), "Name")
            .value("ab")
            .edit_entry(EditEntry::TypeToEdit);
        assert!(on.wants_text_input());
        assert_eq!(
            on.on_key(&plain('x')),
            (Outcome::Changed, Some(InputEvent::Changed))
        );
        assert!(on.editing);
        assert_eq!(on.text(), "abx", "typing lands at the end");
        assert_eq!(
            on.on_key(&bare(KeyCode::Esc)).1,
            Some(InputEvent::Cancelled)
        );
        assert_eq!(on.text(), "ab", "Esc reverts what typing started");
        let ctrl = key(
            KeyCode::Char('s'),
            ratatui::crossterm::event::KeyModifiers::CONTROL,
        );
        assert_eq!(on.on_key(&ctrl).0, Outcome::Ignored, "shortcuts pass");
        // the deprecated switch maps onto the policy
        #[allow(deprecated)]
        let legacy = TextInput::new(WidgetId::of("l"), "Name").type_to_edit(true);
        assert_eq!(legacy.entry(), EditEntry::TypeToEdit);
        // Immediate: any key reaching an unrendered focused field edits
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
    fn undo_redo_and_the_field_clipboard() {
        use ratatui::crossterm::event::KeyModifiers as M;
        let mut i = TextInput::new(WidgetId::of("u"), "Name")
            .edit_entry(EditEntry::Immediate)
            .focus_select(FocusSelect::CaretEnd);
        for c in "ab cd".chars() {
            i.on_key(&plain(c));
        }
        assert_eq!(i.text(), "ab cd");
        i.on_key(&key(KeyCode::Char('z'), M::CONTROL));
        assert_eq!(i.text(), "ab ", "one undo takes back the last word");
        i.on_key(&key(KeyCode::Char('z'), M::CONTROL | M::SHIFT));
        assert_eq!(i.text(), "ab cd", "Ctrl+Shift+Z redoes it");
        // Emacs (default): Ctrl+A is line start, Ctrl+L selects all,
        // Alt+W copies, Ctrl+W cuts, Ctrl+Y yanks
        i.on_key(&key(KeyCode::Char('a'), M::CONTROL));
        assert_eq!(i.buffer.cursor_offset(), 0);
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
        // Desktop: Ctrl+A selects all, Ctrl+X cuts, Ctrl+V pastes
        let mut d = TextInput::new(WidgetId::of("d"), "Name")
            .value("xyz")
            .field_keys(FieldKeys::Desktop);
        d.begin_edit();
        d.on_key(&key(KeyCode::Char('a'), M::CONTROL));
        assert_eq!(d.buffer.selected_text(), Some("xyz"));
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
        // pointer mechanics from a field that is not editing yet
        let mut i = TextInput::new(WidgetId::of("s"), "Name")
            .value("hello big world")
            .edit_entry(EditEntry::Explicit);
        let area = Rect::new(0, 0, 40, 3);
        let mut buf = Buffer::empty(area);
        render(&mut i, area, &mut buf);
        let ta = i.text_area;
        let at = |col: u16| Position::new(ta.x + col, ta.y);
        // press at "e", drag to after "ll": the release keeps the selection
        let (p, d) = (at(1), at(4));
        i.on_press(p);
        assert!(!i.editing, "the press alone changes nothing");
        assert_eq!(i.on_drag(d), Outcome::Changed);
        assert!(i.editing, "the drag edits");
        assert_eq!(i.buffer.selected_text(), Some("ell"));
        i.on_click_count(d, 1);
        assert_eq!(i.buffer.selected_text(), Some("ell"), "release keeps it");
        // a plain click afterwards places the caret
        i.on_click_count(at(2), 1);
        assert_eq!(i.buffer.selected_text(), None);
        // double click: the word; triple: the whole line
        i.on_click_count(at(7), 2);
        assert_eq!(i.buffer.selected_text(), Some("big"));
        i.on_click_count(at(7), 3);
        assert_eq!(i.buffer.selected_text(), Some("hello big world"));
        // a masked field selects the whole secret on a double click
        let mut m = TextInput::new(WidgetId::of("m"), "Key")
            .value("ab cd")
            .masked();
        render(&mut m, area, &mut buf);
        let p = Position::new(m.text_area.x, m.text_area.y);
        m.on_click_count(p, 2);
        assert_eq!(m.buffer.selected_text(), Some("ab cd"));
        // configurable off: clicks only place the caret, drags select nothing
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
        assert_eq!(off.buffer.selected_text(), None);
        assert_eq!(off.buffer.cursor(), 7);
    }

    #[test]
    fn a_drag_after_a_double_click_extends_by_words() {
        let mut i = TextInput::new(WidgetId::of("w"), "Name").value("alpha beta gamma delta");
        let area = Rect::new(0, 0, 40, 3);
        let mut buf = Buffer::empty(area);
        render(&mut i, area, &mut buf);
        let ta = i.text_area;
        let at = |col: u16| Position::new(ta.x + col, ta.y);
        // first click, then the second press of the double click on "beta"
        i.on_click_count(at(7), 1);
        assert_eq!(i.on_press_count(at(7), 2), Outcome::Changed);
        assert_eq!(i.buffer.selected_text(), Some("beta"), "the press selects");
        // dragging into "gamma" takes the whole word, not the characters
        i.on_drag(at(12));
        assert_eq!(i.buffer.selected_text(), Some("beta gamma"));
        // backwards into "alpha": from the start of alpha to the end of beta
        i.on_drag(at(1));
        assert_eq!(i.buffer.selected_text(), Some("alpha beta"));
        i.on_click_count(at(1), 2);
        assert_eq!(
            i.buffer.selected_text(),
            Some("alpha beta"),
            "release keeps"
        );
        // configurable: by characters from the press
        let mut c = TextInput::new(WidgetId::of("c"), "Name")
            .value("alpha beta gamma delta")
            .pointer_select(PointerSelect::DEFAULT.drag_by_word(false));
        render(&mut c, area, &mut buf);
        c.on_click_count(at(7), 1);
        assert_eq!(c.on_press_count(at(7), 2), Outcome::Consumed);
        c.on_drag(at(12));
        assert_eq!(c.buffer.selected_text(), Some("eta g"));
    }

    #[test]
    fn dragging_past_the_left_edge_scrolls_the_field() {
        let long = "0123456789abcdefghijklmnopqrstuvwxyz";
        let mut i = TextInput::new(WidgetId::of("e"), "Name").value(long);
        let area = Rect::new(0, 0, 16, 3);
        let mut buf = Buffer::empty(area);
        render(&mut i, area, &mut buf);
        // show the end: the caret at the end scrolls the start out
        i.begin_edit();
        i.buffer.select_range(long.len(), long.len());
        i.buffer.clear_selection();
        render(&mut i, area, &mut buf);
        let first = i.scroll;
        assert!(first > 0, "the start is scrolled out");
        let ta = i.text_area;
        i.on_press(Position::new(ta.x + 3, ta.y));
        // past the left edge: the selection runs on and the field follows
        i.on_drag(Position::new(ta.x.saturating_sub(1), ta.y));
        render(&mut i, area, &mut buf);
        assert!(i.scroll < first, "scrolled back: {} → {}", first, i.scroll);
        let sel = i.buffer.selection().expect("a selection");
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
        i.buffer.insert_str("w");
        let area = Rect::new(0, 0, 30, 3);
        let mut buf = Buffer::empty(area);
        i.render(area, &mut buf, &mut ctx, theme.canvas);
        assert!(i.editing, "render is not a blur: on_blur ends the edit");
        assert_eq!(ctx.cursor, None, "an unfocused edit shows no caret");
        assert_eq!(i.on_blur(), Some(InputEvent::Committed));
    }

    #[test]
    fn forward_blur_reaches_only_the_field_that_lost_focus() {
        use crate::core::focus::{FocusChange, forward_blur};
        let mut a = TextInput::new(WidgetId::of("a"), "A").value("a");
        let mut b = TextInput::new(WidgetId::of("b"), "B").value("b");
        a.begin_edit();
        // a dirty edit: a clean immediate one would end quietly
        a.buffer.insert_str("x");
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
        assert_eq!(i.error.as_deref(), Some("guest is not a role"));
        i = i.value("admin");
        assert!(i.validate());
    }

    #[test]
    fn required_error_clears_after_keyboard_and_paste_corrections() {
        let mut input = TextInput::new(WidgetId::of("required"), "Name").required(true);
        assert!(!input.validate());
        input.begin_edit();
        input.on_paste("valid");
        assert!(input.error.is_none());
        input.buffer.select_all();
        input.buffer.backspace();
        assert!(!input.validate());
        input.on_key(&Key {
            code: KeyCode::Char('x'),
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        });
        assert!(input.error.is_none());
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
            input.buffer.cursor(),
            2,
            "the caret lands under the pointer"
        );
        // a second click only moves the caret
        input.on_click(Position::new(input.text_area.x + 4, 1));
        assert!(input.editing);
        assert_eq!(input.buffer.cursor(), 4);
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
            assert_eq!(input.buffer.cursor_offset(), offset);
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
        assert_eq!(input.buffer.cursor_offset(), 20);
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
                        input.error = Some("Invalid".into());
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
        assert_eq!(input.buffer.cursor_offset(), input.text().len());
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
