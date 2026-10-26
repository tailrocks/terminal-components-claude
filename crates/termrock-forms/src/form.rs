use core::fmt;
use std::borrow::Cow;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle};
use crate::action::{Action, ActionKey};
use crate::event::{Chord, KeyCode};
use crate::form_layout::{FormLayout, FormLayoutError, FormRow};
use crate::id::{Id, Part, PartRef, fnv1a};
use crate::intent::{Intent, Phase};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::scroll::ScrollState;
use crate::secret::Secret;
use crate::text::width;
use crate::theme::{DesignTokens, Family, Role, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};
use crate::validate::FieldError;
use termrock_collections::collection::key::KeySet;
use termrock_controls::button::Button;
use termrock_controls::choice::{Checkbox, LabelRadio, RadioGroupAction, RadioGroupState, Toggle};
use termrock_feedback::keyhint::ChordText;
use termrock_fields::input::{ErrorState, TextAction, TextInput, TextInputState, discard_error};
use termrock_fields::textarea::{TextArea, TextAreaState};
use termrock_navigation::chip::{ChipBarState, LabelChips};
use termrock_overlays::select::{LabelSelect, SelectAction, SelectState};
use termrock_runtime::author::VerticalWindow;
use termrock_runtime::diagnostics::Diagnostic;
use termrock_runtime::scroll_region::ScrollRegion;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FieldSpan {
    Full,
    Half,
}

pub use termrock_controls::choice::InheritedFormState;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GroupKey(u16);

impl GroupKey {
    pub const ALL: GroupKey = GroupKey(0);

    pub const fn custom(name: &'static str) -> GroupKey {
        GroupKey(1 | ((fnv1a(0xcbf2_9ce4_8422_2325, name.as_bytes()) as u16) & 0x7fff))
    }
}

pub enum FieldKind<'a> {
    Text(TextInput<'a>),
    Area(TextArea<'a>),
    Select(LabelSelect<'a>),
    Radio(LabelRadio<'a>),
    Chips(LabelChips<'a>),
    Check(Checkbox<'a>),
    Toggle(Toggle<'a>),
    Chooser(Button<'a>),
    Note,
}

impl fmt::Debug for FieldKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FieldKind::Text(_) => "Text",
            FieldKind::Area(_) => "Area",
            FieldKind::Select(_) => "Select",
            FieldKind::Radio(_) => "Radio",
            FieldKind::Chips(_) => "Chips",
            FieldKind::Check(_) => "Check",
            FieldKind::Toggle(_) => "Toggle",
            FieldKind::Chooser(_) => "Chooser",
            FieldKind::Note => "Note",
        })
    }
}

pub struct FieldSpec<'a> {
    pub id: Id,
    pub label: &'a str,
    pub kind: FieldKind<'a>,
    pub required: bool,
    pub help: Option<&'a str>,
    pub span: FieldSpan,
    pub group: GroupKey,
    pub plain: bool,
}

impl fmt::Debug for FieldSpec<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldSpec")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("kind", &self.kind)
            .field("required", &self.required)
            .field("help", &self.help)
            .field("span", &self.span)
            .field("group", &self.group)
            .field("plain", &self.plain)
            .finish()
    }
}

impl<'a> FieldSpec<'a> {
    pub const fn new(id: Id, label: &'a str, kind: FieldKind<'a>) -> Self {
        FieldSpec {
            id,
            label,
            kind,
            required: false,
            help: None,
            span: FieldSpan::Full,
            group: GroupKey::ALL,
            plain: false,
        }
    }

    #[must_use]
    pub const fn required(mut self, yes: bool) -> Self {
        self.required = yes;
        self
    }

    #[must_use]
    pub const fn help(mut self, text: &'a str) -> Self {
        self.help = Some(text);
        self
    }

    #[must_use]
    pub const fn span(mut self, span: FieldSpan) -> Self {
        self.span = span;
        self
    }

    #[must_use]
    pub const fn group(mut self, group: GroupKey) -> Self {
        self.group = group;
        self
    }

    #[must_use]
    pub const fn plain(mut self, yes: bool) -> Self {
        self.plain = yes;
        self
    }
}

pub enum FieldMut<'d> {
    Text(&'d mut String),
    Secret(&'d mut Secret),
    Choice(&'d mut usize),
    Flag(&'d mut bool),
    Chips(&'d mut KeySet),
    ReadOnly,
}

impl fmt::Debug for FieldMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FieldMut::Text(_) => "Text([redacted])",
            FieldMut::Secret(_) => "Secret([redacted])",
            FieldMut::Choice(_) => "Choice(..)",
            FieldMut::Flag(_) => "Flag(..)",
            FieldMut::Chips(_) => "Chips(..)",
            FieldMut::ReadOnly => "ReadOnly",
        })
    }
}

pub enum FieldRef<'d> {
    Text(&'d str),
    Secret(&'d Secret),
    Choice(usize),
    Flag(bool),
    Chips(&'d KeySet),
    Display {
        value: &'d str,
        detail: Option<&'d str>,
    },
    Note(&'d [(&'d str, Role)]),
}

impl fmt::Debug for FieldRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldRef::Text(_) => f.write_str("Text([redacted])"),
            FieldRef::Secret(_) => f.write_str("Secret([redacted])"),
            FieldRef::Choice(value) => f.debug_tuple("Choice").field(value).finish(),
            FieldRef::Flag(value) => f.debug_tuple("Flag").field(value).finish(),
            FieldRef::Chips(_) => f.write_str("Chips(..)"),
            FieldRef::Display { .. } => f.write_str("Display([redacted])"),
            FieldRef::Note(rows) => f.debug_tuple("Note").field(&rows.len()).finish(),
        }
    }
}

pub trait FormData {
    fn value(&self, id: Id) -> FieldRef<'_>;
    fn value_mut(&mut self, id: Id) -> FieldMut<'_>;

    fn options(&self, _id: Id) -> &[&str] {
        &[]
    }

    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        (self.value_mut(id), &[])
    }

    fn visible(&self, _id: Id) -> bool {
        true
    }

    fn disabled(&self, _id: Id) -> bool {
        false
    }

    fn error(&self, _id: Id) -> Option<&str> {
        None
    }

    fn validate(&self, _id: Id, _value: FieldRef<'_>) -> Result<(), FieldError> {
        Ok(())
    }

    fn validate_all(&self) -> Result<(), (Id, FieldError)> {
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnterPolicy {
    SubmitsWhenIdle,
    Never,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FieldSizingPolicy {
    #[default]
    Fixed,
    Intrinsic,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormAction {
    Changed(Id),
    Committed(Id),
    Chose(Id),
    Action(ActionKey),
    Invalid(Id),
}

#[derive(Clone, PartialEq, Eq)]
enum SlotValue {
    None,
    Text,
    Choice(usize),
    Flag(bool),
    Chips(KeySet),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FieldShape {
    Text,
    Area,
    Other,
}

fn field_shape(kind: &FieldKind<'_>) -> FieldShape {
    match kind {
        FieldKind::Text(_) => FieldShape::Text,
        FieldKind::Area(_) => FieldShape::Area,
        FieldKind::Select(_)
        | FieldKind::Radio(_)
        | FieldKind::Chips(_)
        | FieldKind::Check(_)
        | FieldKind::Toggle(_)
        | FieldKind::Chooser(_)
        | FieldKind::Note => FieldShape::Other,
    }
}

fn field_is_secret(field: &FieldSpec<'_>) -> bool {
    match &field.kind {
        FieldKind::Text(control) => control.is_secret(),
        FieldKind::Area(control) => control.is_secret(),
        _ => false,
    }
}

static FORM_INVALID_VALUE: FieldError = FieldError {
    message: Cow::Borrowed("Invalid value"),
    code: None,
};

impl fmt::Debug for SlotValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlotValue::None => f.write_str("None"),
            SlotValue::Text => f.write_str("Text([redacted])"),
            SlotValue::Choice(value) => f.debug_tuple("Choice").field(value).finish(),
            SlotValue::Flag(value) => f.debug_tuple("Flag").field(value).finish(),
            SlotValue::Chips(keys) => f.debug_tuple("Chips").field(keys).finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct FieldSlot {
    id: Id,
    shape: FieldShape,
    sensitive: bool,
    value: SlotValue,
    input: TextInputState,
    area: TextAreaState,
    select: SelectState,
    radio: RadioGroupState,
    chips: ChipBarState,
}

impl FieldSlot {
    fn new(id: Id) -> Self {
        FieldSlot {
            id,
            shape: FieldShape::Other,
            sensitive: false,
            value: SlotValue::None,
            input: TextInputState::default(),
            area: TextAreaState::default(),
            select: SelectState::default(),
            radio: RadioGroupState::default(),
            chips: ChipBarState::default(),
        }
    }

    fn zeroize(&mut self) {
        self.input.zeroize();
        self.area.zeroize();
        if let SlotValue::Chips(keys) = &mut self.value {
            keys.none();
        }
        self.value = SlotValue::None;
    }

    fn set_shape(&mut self, shape: FieldShape) {
        if self.shape == shape {
            return;
        }
        self.input.zeroize();
        self.area.zeroize();
        self.input.set_sensitive(false);
        self.area.set_sensitive(false);
        self.sensitive = false;
        self.shape = shape;
    }

    fn set_sensitive(&mut self, sensitive: bool) -> bool {
        let was_sensitive = self.sensitive;
        self.sensitive = sensitive;
        match self.shape {
            FieldShape::Text => {
                self.input.set_sensitive(sensitive);
                self.area.set_sensitive(false);
            }
            FieldShape::Area => {
                self.input.set_sensitive(false);
                self.area.set_sensitive(sensitive);
            }
            FieldShape::Other => {
                self.input.set_sensitive(false);
                self.area.set_sensitive(false);
            }
        }
        was_sensitive != sensitive
    }

    fn is_sensitive(&self) -> bool {
        self.sensitive
    }
}

impl fmt::Debug for FieldSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldSlot")
            .field("id", &self.id)
            .field("shape", &self.shape)
            .field("sensitive", &self.sensitive)
            .field("value", &self.value)
            .field("input", &self.input)
            .field("area", &self.area)
            .field("select", &self.select)
            .field("radio", &self.radio)
            .field("chips", &self.chips)
            .finish()
    }
}

#[derive(Default)]
pub struct FormState {
    slots: Vec<FieldSlot>,
    scroll: ScrollState,
    errors: Vec<(Id, ErrorState)>,
    dirty: bool,
    reveal: Option<Id>,
    lane_scrolls: Vec<(Id, ScrollState)>,
    pointer_hold: Option<Id>,
}

impl Clone for FormState {
    fn clone(&self) -> Self {
        FormState {
            slots: self.slots.clone(),
            scroll: self.scroll,
            errors: self
                .errors
                .iter()
                .map(|(id, error)| {
                    if Self::public_errors_are_redacted() {
                        (*id, ErrorState::sensitive())
                    } else {
                        (*id, error.clone())
                    }
                })
                .collect(),
            dirty: self.dirty,
            reveal: self.reveal,
            lane_scrolls: self.lane_scrolls.clone(),
            pointer_hold: self.pointer_hold,
        }
    }
}

impl PartialEq for FormState {
    fn eq(&self, other: &Self) -> bool {
        self.slots == other.slots
            && self.scroll == other.scroll
            && self.errors.len() == other.errors.len()
            && self.errors.iter().zip(&other.errors).all(
                |((id, error), (other_id, other_error))| {
                    id == other_id
                        && (Self::public_errors_are_redacted() || error.same(other_error))
                },
            )
            && self.dirty == other.dirty
            && self.reveal == other.reveal
            && self.lane_scrolls == other.lane_scrolls
            && self.pointer_hold == other.pointer_hold
    }
}

impl Eq for FormState {}

impl fmt::Debug for FormState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FormState")
            .field("slots", &self.slots)
            .field("scroll", &self.scroll)
            .field("errors", &(!self.errors.is_empty()).then_some("[redacted]"))
            .field("dirty", &self.dirty)
            .field("reveal", &self.reveal)
            .field("pointer_hold", &self.pointer_hold)
            .finish()
    }
}

impl FormState {
    fn lane_scroll(&self, lane: Id) -> ScrollState {
        self.lane_scrolls
            .iter()
            .find(|(key, _)| *key == lane)
            .map_or(ScrollState::default(), |(_, state)| *state)
    }

    fn lane_scroll_mut(&mut self, lane: Id) -> &mut ScrollState {
        let existing = self.lane_scrolls.iter().position(|(key, _)| *key == lane);
        if let Some(index) = existing {
            if let Some((_, state)) = self.lane_scrolls.get_mut(index) {
                return state;
            }
            return &mut self.scroll;
        }
        self.lane_scrolls.push((lane, ScrollState::default()));
        match self.lane_scrolls.last_mut() {
            Some((_, state)) => state,
            None => &mut self.scroll,
        }
    }

    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn editing_text(&self, id: Id) -> Option<&str> {
        self.slots
            .iter()
            .find(|slot| slot.id == id)?
            .input
            .draft_text()
    }

    pub fn select_editing_text(&mut self, id: Id) {
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.id == id) {
            slot.input.select_all();
        }
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn error(&self, id: Id) -> Option<&FieldError> {
        self.errors
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, error)| {
                if Self::public_errors_are_redacted() {
                    &FORM_INVALID_VALUE
                } else {
                    error.as_ref()
                }
            })
    }

    pub fn set_error(&mut self, id: Id, error: Option<FieldError>) {
        let sensitivity = self.slot_sensitivity(id).filter(|sensitive| *sensitive);
        self.set_error_with_sensitivity(id, error, sensitivity);
    }

    fn set_error_with_sensitivity(
        &mut self,
        id: Id,
        error: Option<FieldError>,
        sensitivity: Option<bool>,
    ) {
        let error = error.map(|error| match sensitivity {
            Some(true) => {
                discard_error(error);
                ErrorState::sensitive()
            }
            Some(false) => ErrorState::Plain(error),
            None => ErrorState::Pending(error),
        });
        self.replace_error(id, error);
    }

    fn replace_error(&mut self, id: Id, error: Option<ErrorState>) {
        if let Some(index) = self.errors.iter().position(|(key, _)| *key == id) {
            if let Some(error) = error {
                if let Some((_, current)) = self.errors.get_mut(index) {
                    let old = core::mem::replace(current, error);
                    old.discard();
                }
            } else {
                let (_, old) = self.errors.remove(index);
                old.discard();
            }
        } else if let Some(error) = error {
            self.errors.push((id, error));
        }
    }

    fn slot_sensitivity(&self, id: Id) -> Option<bool> {
        self.slots
            .iter()
            .find(|slot| slot.id == id)
            .map(FieldSlot::is_sensitive)
    }

    const fn public_errors_are_redacted() -> bool {
        true
    }

    fn error_message(&self, id: Id, sensitive: bool) -> Option<&str> {
        self.errors
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, error)| {
                if sensitive {
                    "Invalid value"
                } else {
                    error.as_ref().message.as_ref()
                }
            })
    }

    fn redact_error(&mut self, id: Id) {
        if let Some((_, current)) = self.errors.iter_mut().find(|(key, _)| *key == id) {
            let old = core::mem::replace(current, ErrorState::sensitive());
            *current = old.redact();
        }
    }

    fn reconcile_error(&mut self, id: Id, sensitive: bool) {
        let Some(index) = self.errors.iter().position(|(key, _)| *key == id) else {
            return;
        };
        let (id, old) = self.errors.remove(index);
        if !sensitive && old.is_sensitive() {
            return;
        }
        let error = if sensitive {
            old.redact()
        } else {
            old.resolve_plain()
        };
        self.errors.insert(index, (id, error));
    }

    fn clear_error(&mut self, id: Id) {
        if let Some(index) = self.errors.iter().position(|(key, _)| *key == id) {
            let (_, old) = self.errors.remove(index);
            old.discard();
        }
    }

    pub fn clear_errors(&mut self) {
        for (_, error) in self.errors.drain(..) {
            error.discard();
        }
    }

    pub fn reveal(&mut self, id: Id) {
        self.reveal = Some(id);
    }

    pub fn zeroize(&mut self) {
        for slot in &mut self.slots {
            slot.zeroize();
        }
        self.clear_errors();
        self.reveal = None;
    }

    fn reconcile_fields_with_data<D: FormData + ?Sized>(
        &mut self,
        fields: &[FieldSpec<'_>],
        data: &D,
    ) {
        self.reconcile_fields_with_sensitivity(fields, |field| {
            field_is_secret(field) || matches!(data.value(field.id), FieldRef::Secret(_))
        });
    }

    fn reconcile_fields_with_sensitivity(
        &mut self,
        fields: &[FieldSpec<'_>],
        is_sensitive: impl Fn(&FieldSpec<'_>) -> bool,
    ) {
        if self.slots.len() == fields.len()
            && self
                .slots
                .iter()
                .zip(fields)
                .all(|(slot, field)| slot.id == field.id && slot.shape == field_shape(&field.kind))
        {
            for (slot, field) in self.slots.iter_mut().zip(fields) {
                let sensitive = is_sensitive(field);
                slot.set_sensitive(sensitive);
            }
            for field in fields {
                self.reconcile_error(field.id, is_sensitive(field));
            }
            return;
        }
        let old_errors = core::mem::take(&mut self.errors);
        for (id, error) in old_errors {
            let Some(field) = fields.iter().find(|field| field.id == id) else {
                error.discard();
                continue;
            };
            if !matches!(error, ErrorState::Pending(_)) {
                error.discard();
                continue;
            }
            let error = if is_sensitive(field) {
                error.redact()
            } else {
                error.resolve_plain()
            };
            self.errors.push((id, error));
        }
        let mut old = core::mem::take(&mut self.slots);
        self.slots.reserve(fields.len());
        for field in fields {
            let shape = field_shape(&field.kind);
            let declared_sensitive = is_sensitive(field);
            if let Some(index) = old.iter().position(|slot| slot.id == field.id) {
                let mut slot = old.remove(index);
                slot.set_shape(shape);
                slot.set_sensitive(declared_sensitive);
                self.slots.push(slot);
            } else {
                let mut slot = FieldSlot::new(field.id);
                slot.set_shape(shape);
                slot.set_sensitive(declared_sensitive);
                self.slots.push(slot);
            }
        }
        for slot in &mut old {
            slot.zeroize();
        }
    }
}

pub struct Form<'a> {
    id: Id,
    fields: &'a [FieldSpec<'a>],
    actions: &'a [Action<'a>],
    submit: ActionKey,
    enter: EnterPolicy,
    sizing: FieldSizingPolicy,
    columns: u8,
    group: GroupKey,
    ov: PartStyle<'a>,
    parts: &'a [(Part, StylePatch)],
    layout: Option<&'a FormLayout<'a>>,
}

impl fmt::Debug for Form<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Form")
            .field("id", &self.id)
            .field("fields", &self.fields.len())
            .field("actions", &self.actions.len())
            .field("submit", &self.submit)
            .field("enter", &self.enter)
            .field("sizing", &self.sizing)
            .field("columns", &self.columns)
            .field("group", &self.group)
            .field("layout", &self.layout.is_some())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy)]
struct Placement {
    index: usize,
    top: usize,
    left: u16,
    width: u16,
    height: u16,
}

#[derive(Clone, Copy)]
struct FieldWindow {
    area: Rect,
    source_first_row: usize,
    logical_height: u16,
}

fn resolve_field_window(content: Rect, placement: Placement, offset: usize) -> Option<FieldWindow> {
    let visible_start = placement.top.max(offset);
    let visible_end = placement
        .top
        .saturating_add(usize::from(placement.height))
        .min(offset.saturating_add(usize::from(content.height)));
    if visible_start >= visible_end {
        return None;
    }
    let row_offset = visible_start.saturating_sub(offset);
    Some(FieldWindow {
        area: Rect {
            x: content.x.saturating_add(placement.left),
            y: content
                .y
                .saturating_add(row_offset.min(usize::from(u16::MAX)) as u16),
            width: placement
                .width
                .min(content.width.saturating_sub(placement.left)),
            height: visible_end
                .saturating_sub(visible_start)
                .min(usize::from(u16::MAX)) as u16,
        },
        source_first_row: visible_start.saturating_sub(placement.top),
        logical_height: placement.height,
    })
}

fn native_body_visible(ui: &Ui<'_>, body: Rect) -> bool {
    !body.is_empty() && !body.intersection(ui.full()).is_empty()
}

fn native_first_row_visible(ui: &Ui<'_>, body: Rect) -> bool {
    native_body_visible(
        ui,
        Rect {
            height: body.height.min(1),
            ..body
        },
    )
}

fn native_radio_area(body: Rect, first_option: usize, option_count: usize) -> Rect {
    let remaining = option_count.saturating_sub(first_option);
    Rect {
        height: body
            .height
            .min(u16::try_from(remaining).unwrap_or(u16::MAX)),
        ..body
    }
}

fn native_button_area(ui: &Ui<'_>, button: &Button<'_>, body: Rect) -> Rect {
    let width = button
        .measure(ui, Constraints::loose(body.width, 1))
        .preferred
        .0
        .min(body.width);
    Rect {
        width,
        height: body.height.min(1),
        ..body
    }
}

fn textarea_window(
    body: Rect,
    first_source_row: usize,
    logical_field_height: u16,
    requested_rows: u16,
) -> Option<VerticalWindow> {
    let logical_rows = usize::from(requested_rows.min(logical_field_height.saturating_sub(2)));
    let visible_start = first_source_row.min(logical_rows);
    let visible_end = first_source_row
        .saturating_add(usize::from(body.height))
        .min(logical_rows);
    if body.width == 0 || visible_start >= visible_end {
        return None;
    }
    let row_offset = visible_start.saturating_sub(first_source_row);
    let visible_body = Rect {
        y: body
            .y
            .saturating_add(row_offset.min(usize::from(u16::MAX)) as u16),
        height: visible_end
            .saturating_sub(visible_start)
            .min(usize::from(u16::MAX)) as u16,
        ..body
    };
    VerticalWindow::new(visible_body, logical_rows, visible_start).ok()
}

struct LanePlan {
    id: Id,
    area: Rect,
    content_rows: usize,
    placements: Vec<Placement>,
}

fn lane_scroll_id(form: Id, lane: Id) -> Id {
    form.part(Part::BODY)
        .item(crate::id::ItemKey::Num(lane.hash()))
}

struct Placements<'fields, 'a> {
    fields: &'fields [FieldSpec<'a>],
    group: GroupKey,
    columns: u8,
    width: u16,
    gap: u16,
    field_height: u16,
    sizing: FieldSizingPolicy,
    index: usize,
    top: usize,
    half_height: Option<u16>,
}

impl Placements<'_, '_> {
    fn next<D: FormData + ?Sized>(&mut self, data: &D) -> Option<Placement> {
        while let Some((index, field)) =
            self.fields.get(self.index).map(|field| (self.index, field))
        {
            self.index = self.index.saturating_add(1);
            if !((field.group == GroupKey::ALL || field.group == self.group)
                && data.visible(field.id))
            {
                continue;
            }
            let height = placement_height(field, self.field_height, self.sizing, data);
            let is_half = self.columns == 2 && field.span == FieldSpan::Half;
            let half_width = self.width.saturating_sub(self.gap) / 2;
            if is_half && let Some(left_height) = self.half_height.take() {
                let placement = Placement {
                    index,
                    top: self.top,
                    left: half_width.saturating_add(self.gap),
                    width: self
                        .width
                        .saturating_sub(half_width)
                        .saturating_sub(self.gap),
                    height,
                };
                self.top = self
                    .top
                    .saturating_add(usize::from(height.max(left_height)));
                return Some(placement);
            }
            if let Some(left_height) = self.half_height.take() {
                self.top = self.top.saturating_add(usize::from(left_height));
            }
            let placement = Placement {
                index,
                top: self.top,
                left: 0,
                width: if is_half { half_width } else { self.width },
                height,
            };
            if is_half {
                self.half_height = Some(height);
            } else {
                self.top = self.top.saturating_add(usize::from(height));
            }
            return Some(placement);
        }
        None
    }
}

fn placement_height<D: FormData + ?Sized>(
    field: &FieldSpec<'_>,
    base: u16,
    sizing: FieldSizingPolicy,
    data: &D,
) -> u16 {
    match &field.kind {
        FieldKind::Area(area) => base.max(area.rows_in_form().saturating_add(2)),
        FieldKind::Radio(_) => {
            let rows = u16::try_from(data.options(field.id).len()).unwrap_or(u16::MAX);
            base.max(rows.saturating_add(2))
        }
        FieldKind::Note if sizing == FieldSizingPolicy::Intrinsic => {
            let rows = match data.value(field.id) {
                FieldRef::Note(rows) => rows.len(),
                _ => 0,
            };
            u16::try_from(rows).unwrap_or(u16::MAX)
        }
        FieldKind::Chooser(_) if sizing == FieldSizingPolicy::Intrinsic => {
            let detail = matches!(
                data.value(field.id),
                FieldRef::Display {
                    detail: Some(_),
                    ..
                }
            );
            2 + u16::from(detail)
        }
        _ => base,
    }
}

impl<'a> Form<'a> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BODY,
        Part::ACTIONS,
        Part::HELP,
        Part::MARKER,
        Part::TRACK,
        Part::THUMB,
    ];

    pub const fn new(id: Id, fields: &'a [FieldSpec<'a>]) -> Self {
        Form {
            id,
            fields,
            actions: &[],
            submit: ActionKey::SAVE,
            enter: EnterPolicy::SubmitsWhenIdle,
            sizing: FieldSizingPolicy::Fixed,
            columns: 1,
            group: GroupKey::ALL,
            ov: PartStyle::new(),
            parts: &[],
            layout: None,
        }
    }

    #[must_use]
    pub const fn actions(mut self, actions: &'a [Action<'a>]) -> Self {
        self.actions = actions;
        self
    }

    #[must_use]
    pub const fn submit(mut self, key: ActionKey) -> Self {
        self.submit = key;
        self
    }

    #[must_use]
    pub const fn enter(mut self, policy: EnterPolicy) -> Self {
        self.enter = policy;
        self
    }

    #[must_use]
    pub const fn field_sizing(mut self, policy: FieldSizingPolicy) -> Self {
        self.sizing = policy;
        self
    }

    #[must_use]
    pub const fn columns(mut self, columns: u8) -> Self {
        self.columns = if columns > 1 { 2 } else { 1 };
        self
    }

    #[must_use]
    pub const fn group(mut self, group: GroupKey) -> Self {
        self.group = group;
        self
    }

    pub fn try_layout(mut self, layout: &'a FormLayout<'a>) -> Result<Self, FormLayoutError> {
        layout.validate(self.fields.iter().map(|field| field.id))?;
        self.layout = Some(layout);
        Ok(self)
    }

    pub fn validate_layout<D: FormData + ?Sized>(&self, data: &D) -> Result<(), FormLayoutError> {
        let Some(layout) = self.layout else {
            return Ok(());
        };
        layout.validate(self.fields.iter().map(|field| field.id))?;
        for field in self.fields {
            if self.shown(data, field)
                && !layout
                    .lanes()
                    .iter()
                    .flat_map(|lane| lane.rows())
                    .any(|row| match row {
                        FormRow::Field(id) | FormRow::Fill(id) => *id == field.id,
                        FormRow::Pair { first, second, .. } => {
                            *first == field.id || *second == field.id
                        }
                        FormRow::Spacer(_) => false,
                    })
            {
                return Err(FormLayoutError::MissingVisibleField(field.id));
            }
        }
        Ok(())
    }

    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(patches);
        self.parts = patches;
        self
    }

    fn shown<D: FormData + ?Sized>(&self, data: &D, field: &FieldSpec<'_>) -> bool {
        (field.group == GroupKey::ALL || field.group == self.group) && data.visible(field.id)
    }

    fn native_source_row<D: FormData + ?Sized>(
        &self,
        field: &FieldSpec<'_>,
        data: &D,
        logical_height: u16,
    ) -> Option<usize> {
        if !self.shown(data, field) {
            return None;
        }
        match (&field.kind, data.value(field.id)) {
            (FieldKind::Text(_), FieldRef::Text(_) | FieldRef::Secret(_))
                if logical_height >= 3 =>
            {
                Some(1)
            }
            (FieldKind::Area(control), FieldRef::Text(_) | FieldRef::Secret(_))
                if control.rows_in_form().min(logical_height.saturating_sub(2)) > 0 =>
            {
                Some(1)
            }
            (FieldKind::Select(_), FieldRef::Choice(_)) if logical_height >= 3 => Some(1),
            (FieldKind::Radio(_), FieldRef::Choice(_)) if !data.options(field.id).is_empty() => {
                Some(1)
            }
            (FieldKind::Chips(_), FieldRef::Chips(_)) if logical_height >= 3 => Some(1),
            (FieldKind::Check(_), FieldRef::Flag(_)) if logical_height >= 3 => Some(1),
            (FieldKind::Toggle(_), FieldRef::Flag(_)) if logical_height >= 3 => Some(1),
            (FieldKind::Chooser(_), FieldRef::Display { .. })
                if (self.sizing == FieldSizingPolicy::Intrinsic && logical_height > 1)
                    || (self.sizing != FieldSizingPolicy::Intrinsic && logical_height >= 3) =>
            {
                Some(1)
            }
            _ => None,
        }
    }

    fn reveal_index<D: FormData + ?Sized>(
        &self,
        placement: Placement,
        viewport: usize,
        offset: usize,
        data: &D,
    ) -> usize {
        if viewport == 0 {
            return placement.top;
        }
        let height = usize::from(placement.height);
        let native_row = self
            .fields
            .get(placement.index)
            .and_then(|field| self.native_source_row(field, data, placement.height))
            .map_or(0, |row| row.min(height.saturating_sub(1)));
        if placement.top < offset {
            return if viewport == 1 {
                placement.top.saturating_add(native_row)
            } else {
                placement.top
            };
        }
        let bottom = placement.top.saturating_add(height.saturating_sub(1));
        if height <= viewport {
            bottom
        } else if native_row > 0 {
            placement.top.saturating_add(native_row)
        } else {
            placement.top
        }
    }

    fn register_clipped_field<D: FormData + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        field: &FieldSpec<'_>,
        data: &D,
        logical_height: u16,
    ) {
        if ui.is_inert()
            || self
                .native_source_row(field, data, logical_height)
                .is_none()
        {
            return;
        }
        let inherited_disabled = data.disabled(field.id);
        match (&field.kind, data.value(field.id)) {
            (FieldKind::Text(control), FieldRef::Text(_) | FieldRef::Secret(_)) => {
                control.register_focus_only_in_form(ui, inherited_disabled);
            }
            (FieldKind::Area(control), FieldRef::Text(_) | FieldRef::Secret(_)) => {
                control.register_focus_only_in_form(ui, inherited_disabled);
            }
            (FieldKind::Select(control), FieldRef::Choice(_)) => {
                control.register_focus_only_in_form(ui, inherited_disabled);
            }
            (FieldKind::Radio(control), FieldRef::Choice(_)) => {
                control.register_focus_only_in_form(ui, inherited_disabled);
            }
            (FieldKind::Chips(control), FieldRef::Chips(_)) => {
                control.register_focus_only_in_form(ui, inherited_disabled);
            }
            (FieldKind::Check(control), FieldRef::Flag(_)) => {
                control.register_focus_only_in_form(ui, inherited_disabled);
            }
            (FieldKind::Toggle(control), FieldRef::Flag(_)) => {
                control.register_focus_only_in_form(ui, inherited_disabled);
            }
            (FieldKind::Chooser(control), FieldRef::Display { .. }) => {
                control.register_focus_only_in_form(ui, inherited_disabled);
            }
            _ => {}
        }
    }

    fn sync_pointer_hold<D: FormData + ?Sized>(
        &self,
        cx: &Cx<'_>,
        state: &mut FormState,
        data: &D,
    ) -> bool {
        let mut started = false;
        for field in self.fields {
            for intent in cx.intents(field.id) {
                match intent {
                    Intent::Pointer {
                        phase: Phase::Press,
                        ..
                    } => {
                        state.pointer_hold = Some(field.id);
                        started = true;
                    }
                    Intent::Pointer {
                        phase: Phase::Release | Phase::DragEnd,
                        ..
                    }
                    | Intent::Cancel => state.pointer_hold = None,
                    _ => {}
                }
            }
        }

        let Some(held) = state.pointer_hold else {
            return false;
        };
        let live = self
            .fields
            .iter()
            .find(|field| field.id == held)
            .is_some_and(|field| self.shown(data, field) && !data.disabled(held));
        if !live || (!started && !cx.state(held).contains(StateFlags::PRESSED)) {
            state.pointer_hold = None;
        }
        state.pointer_hold.is_some()
    }

    fn field_height(field: &FieldSpec<'_>, design: &DesignTokens) -> u16 {
        match &field.kind {
            FieldKind::Area(area) => design
                .size
                .field_height
                .max(area.rows_in_form().saturating_add(2)),
            _ => design.size.field_height,
        }
    }

    fn placements(&self, design: &DesignTokens, width: u16) -> Placements<'_, 'a> {
        Placements {
            fields: self.fields,
            group: self.group,
            columns: self.columns,
            width,
            gap: design.space.column_gap.min(width),
            field_height: design.size.field_height,
            sizing: self.sizing,
            index: 0,
            top: 0,
            half_height: None,
        }
    }

    fn content_rows<D: FormData + ?Sized>(mut placements: Placements<'_, '_>, data: &D) -> usize {
        let mut rows = 0;
        while let Some(placement) = placements.next(data) {
            rows = rows.max(placement.top.saturating_add(usize::from(placement.height)));
        }
        rows
    }

    fn lane_plans<D: FormData + ?Sized>(
        &self,
        design: &DesignTokens,
        area: Rect,
        data: &D,
    ) -> Vec<LanePlan> {
        let Some(layout) = self.layout else {
            return Vec::new();
        };
        let lane_areas: Vec<Rect> = match (layout.lanes().len(), layout.split_policy()) {
            (1, _) => vec![area],
            (2, Some((split, gap))) => {
                let (first, second) = split.layout(area, gap);
                vec![first, second]
            }
            _ => Vec::new(),
        };
        layout
            .lanes()
            .iter()
            .zip(lane_areas)
            .map(|(lane, lane_area)| {
                let mut y = 0_usize;
                let mut placements = Vec::new();
                for (row_index, row) in lane.rows().iter().enumerate() {
                    match *row {
                        FormRow::Spacer(rows) => y = y.saturating_add(usize::from(rows)),
                        FormRow::Field(id) | FormRow::Fill(id) => {
                            let Some((index, field)) = self
                                .fields
                                .iter()
                                .enumerate()
                                .find(|(_, field)| field.id == id && self.shown(data, field))
                            else {
                                continue;
                            };
                            let min_height = placement_height(
                                field,
                                design.size.field_height,
                                self.sizing,
                                data,
                            );
                            let height = if matches!(row, FormRow::Fill(_)) {
                                let trailing_spacers = lane
                                    .rows()
                                    .iter()
                                    .skip(row_index.saturating_add(1))
                                    .map(|row| match row {
                                        FormRow::Spacer(rows) => usize::from(*rows),
                                        _ => 0,
                                    })
                                    .fold(0_usize, usize::saturating_add);
                                min_height.max(
                                    usize::from(lane_area.height)
                                        .saturating_sub(y)
                                        .saturating_sub(trailing_spacers)
                                        .min(usize::from(u16::MAX))
                                        as u16,
                                )
                            } else {
                                min_height
                            };
                            placements.push(Placement {
                                index,
                                top: y,
                                left: 0,
                                width: lane_area.width,
                                height,
                            });
                            y = y.saturating_add(usize::from(height));
                        }
                        FormRow::Pair {
                            first,
                            second,
                            split,
                            gap,
                        } => {
                            let first_field =
                                self.fields.iter().enumerate().find(|(_, field)| {
                                    field.id == first && self.shown(data, field)
                                });
                            let second_field =
                                self.fields.iter().enumerate().find(|(_, field)| {
                                    field.id == second && self.shown(data, field)
                                });
                            match (first_field, second_field) {
                                (
                                    Some((first_index, first_field)),
                                    Some((second_index, second_field)),
                                ) => {
                                    let (first_area, second_area) =
                                        split.layout(Rect::new(0, 0, lane_area.width, 1), gap);
                                    let first_height = placement_height(
                                        first_field,
                                        design.size.field_height,
                                        self.sizing,
                                        data,
                                    );
                                    let second_height = placement_height(
                                        second_field,
                                        design.size.field_height,
                                        self.sizing,
                                        data,
                                    );
                                    let height = first_height.max(second_height);
                                    placements.push(Placement {
                                        index: first_index,
                                        top: y,
                                        left: first_area.x,
                                        width: first_area.width,
                                        height: first_height,
                                    });
                                    placements.push(Placement {
                                        index: second_index,
                                        top: y,
                                        left: second_area.x,
                                        width: second_area.width,
                                        height: second_height,
                                    });
                                    y = y.saturating_add(usize::from(height));
                                }
                                (Some((index, field)), None) | (None, Some((index, field))) => {
                                    let height = placement_height(
                                        field,
                                        design.size.field_height,
                                        self.sizing,
                                        data,
                                    );
                                    placements.push(Placement {
                                        index,
                                        top: y,
                                        left: 0,
                                        width: lane_area.width,
                                        height,
                                    });
                                    y = y.saturating_add(usize::from(height));
                                }
                                (None, None) => {}
                            }
                        }
                    }
                }
                LanePlan {
                    id: lane.id,
                    area: lane_area,
                    content_rows: y,
                    placements,
                }
            })
            .collect()
    }

    fn set_slot_value(slot: &mut FieldSlot, value: &FieldMut<'_>) {
        slot.value = match value {
            FieldMut::Text(_) | FieldMut::Secret(_) => SlotValue::Text,
            FieldMut::Choice(value) => SlotValue::Choice(**value),
            FieldMut::Flag(value) => SlotValue::Flag(**value),
            FieldMut::Chips(keys) => SlotValue::Chips((**keys).clone()),
            FieldMut::ReadOnly => SlotValue::None,
        };
    }

    fn prepare_slot<'s>(
        st: &'s mut FormState,
        index: usize,
        field: &FieldSpec<'_>,
        value: &FieldMut<'_>,
    ) -> Option<&'s mut FieldSlot> {
        let secret = field_is_secret(field) || matches!(value, FieldMut::Secret(_));
        let id = field.id;
        let changed;
        {
            let slot = st.slots.get_mut(index)?;
            changed = slot.set_sensitive(secret);
            Self::set_slot_value(slot, value);
        }
        if changed && !secret {
            st.clear_error(id);
        } else if secret {
            st.redact_error(id);
        }
        st.slots.get_mut(index)
    }

    fn remember_action(first: &mut Option<FormAction>, action: FormAction) {
        if first.is_none() {
            *first = Some(action);
        }
    }

    fn update_field<'field>(
        field: &'field FieldSpec<'field>,
        cx: &mut Cx<'_>,
        slot: &mut FieldSlot,
        value: FieldMut<'_>,
        options: &[&'field str],
        inherited_disabled: bool,
    ) -> Response<FormAction> {
        match (&field.kind, value) {
            (FieldKind::Text(control), FieldMut::Text(value)) => control
                .update_in_form(cx, &mut slot.input, value, inherited_disabled)
                .map_action(|action| match action {
                    TextAction::Changed => FormAction::Changed(field.id),
                    TextAction::Committed => FormAction::Committed(field.id),
                    TextAction::Cancelled | TextAction::MoveNext | TextAction::MovePrev => {
                        FormAction::Changed(field.id)
                    }
                }),
            (FieldKind::Text(control), FieldMut::Secret(value)) => control
                .update_in_form(cx, &mut slot.input, value, inherited_disabled)
                .map_action(|action| match action {
                    TextAction::Changed => FormAction::Changed(field.id),
                    TextAction::Committed => FormAction::Committed(field.id),
                    TextAction::Cancelled | TextAction::MoveNext | TextAction::MovePrev => {
                        FormAction::Changed(field.id)
                    }
                }),
            (FieldKind::Area(control), FieldMut::Text(value)) => control
                .update_in_form(cx, &mut slot.area, value, inherited_disabled)
                .map_action(|action| match action {
                    TextAction::Changed => FormAction::Changed(field.id),
                    TextAction::Committed => FormAction::Committed(field.id),
                    TextAction::Cancelled | TextAction::MoveNext | TextAction::MovePrev => {
                        FormAction::Changed(field.id)
                    }
                }),
            (FieldKind::Area(control), FieldMut::Secret(value)) => control
                .update_in_form(cx, &mut slot.area, value, inherited_disabled)
                .map_action(|action| match action {
                    TextAction::Changed => FormAction::Changed(field.id),
                    TextAction::Committed => FormAction::Committed(field.id),
                    TextAction::Cancelled | TextAction::MoveNext | TextAction::MovePrev => {
                        FormAction::Changed(field.id)
                    }
                }),
            (FieldKind::Select(control), FieldMut::Choice(value)) => control
                .update_in_form(cx, &mut slot.select, value, options, inherited_disabled)
                .map_action(|action| match action {
                    SelectAction::Chose(_) => FormAction::Committed(field.id),
                    SelectAction::Opened | SelectAction::Closed => FormAction::Changed(field.id),
                }),
            (FieldKind::Radio(control), FieldMut::Choice(value)) => control
                .update_in_form(cx, &mut slot.radio, value, options, inherited_disabled)
                .map_action(|RadioGroupAction::Chose(_)| FormAction::Committed(field.id)),
            (FieldKind::Chips(control), FieldMut::Chips(value)) => control
                .update_in_form(cx, &mut slot.chips, value, options, inherited_disabled)
                .map_action(|_| FormAction::Committed(field.id)),
            (FieldKind::Check(control), FieldMut::Flag(value)) => control
                .update_in_form(cx, value, inherited_disabled)
                .map_action(|_| FormAction::Committed(field.id)),
            (FieldKind::Toggle(control), FieldMut::Flag(value)) => control
                .update_in_form(cx, value, inherited_disabled)
                .map_action(|_| FormAction::Committed(field.id)),
            (FieldKind::Chooser(control), FieldMut::ReadOnly) => control
                .update_in_form(cx, inherited_disabled)
                .map_action(|_| FormAction::Chose(field.id)),
            _ => Response::ignored(),
        }
    }

    fn commit_focused<D: FormData + ?Sized>(&self, cx: &Cx<'_>, st: &mut FormState, data: &mut D) {
        for (index, field) in self.fields.iter().enumerate() {
            if !self.shown(data, field) || !cx.state(field.id).contains(StateFlags::FOCUSED) {
                continue;
            }
            let disabled = data.disabled(field.id);
            let (value, _) = data.value_and_options(field.id);
            let Some(slot) = st.slots.get_mut(index) else {
                return;
            };
            let committed = match (&field.kind, value) {
                (FieldKind::Text(control), FieldMut::Text(value)) => {
                    control.commit_in_form(&mut slot.input, value)
                }
                (FieldKind::Text(control), FieldMut::Secret(value)) => {
                    control.commit_in_form(&mut slot.input, value)
                }
                (FieldKind::Area(control), FieldMut::Text(value)) => {
                    control.commit_in_form(&mut slot.area, value)
                }
                (FieldKind::Area(control), FieldMut::Secret(value)) => {
                    control.commit_in_form(&mut slot.area, value)
                }
                _ => false,
            };
            if committed && !disabled {
                st.dirty = true;
            }
            return;
        }
    }

    fn submit_form<D: FormData + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut FormState,
        data: &mut D,
    ) -> FormAction {
        self.commit_focused(cx, st, data);
        st.clear_errors();
        for field in self.fields {
            if !self.shown(data, field) {
                continue;
            }
            if let Err(error) = data.validate(field.id, data.value(field.id)) {
                let sensitive =
                    field_is_secret(field) || matches!(data.value(field.id), FieldRef::Secret(_));
                st.set_error_with_sensitivity(
                    field.id,
                    Some(Self::safe_error(
                        data,
                        field_is_secret(field),
                        field.id,
                        error,
                    )),
                    Some(sensitive),
                );
                st.reveal(field.id);
                cx.focus(field.id);
                return FormAction::Invalid(field.id);
            }
        }
        if let Err((id, error)) = data.validate_all() {
            let declared_secret = self
                .fields
                .iter()
                .find(|field| field.id == id)
                .is_some_and(field_is_secret);
            let sensitive = declared_secret || matches!(data.value(id), FieldRef::Secret(_));
            st.set_error_with_sensitivity(
                id,
                Some(Self::safe_error(data, declared_secret, id, error)),
                Some(sensitive),
            );
            st.reveal(id);
            cx.focus(id);
            return FormAction::Invalid(id);
        }
        FormAction::Action(self.submit)
    }

    pub(crate) fn submit_current<D: FormData + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut FormState,
        data: &mut D,
    ) -> Response<FormAction> {
        if let Err(error) = self.validate_layout(data) {
            cx.diagnostic(Diagnostic::InvalidComponentLayout {
                owner: self.id,
                field: error.field_id(),
            });
            return Response::ignored().for_id(self.id);
        }
        st.reconcile_fields_with_data(self.fields, data);
        Response::action(self.submit_form(cx, st, data)).for_id(self.id)
    }

    pub fn update<D: FormData + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut FormState,
        data: &mut D,
    ) -> Response<FormAction> {
        if let Err(error) = self.validate_layout(data) {
            cx.diagnostic(Diagnostic::InvalidComponentLayout {
                owner: self.id,
                field: error.field_id(),
            });
            return Response::ignored().for_id(self.id);
        }
        if self.layout.is_some() {
            return self.update_lanes(cx, st, data);
        }
        st.reconcile_fields_with_data(self.fields, data);
        let pointer_held = self.sync_pointer_hold(cx, st, data);
        let width = cx.area(self.id).map_or(0, |area| area.width);
        let content_rows = Self::content_rows(self.placements(cx.design(), width), data);
        let scroll = ScrollRegion::new(self.id).patch_part(self.parts);
        let scroll_response = scroll.update(cx, &mut st.scroll, content_rows);
        if !pointer_held
            && let Some(id) = st.reveal.take()
            && let Some(placement) = {
                let mut placements = self.placements(cx.design(), width);
                let mut found = None;
                while let Some(placement) = placements.next(data) {
                    if self
                        .fields
                        .get(placement.index)
                        .is_some_and(|field| field.id == id)
                    {
                        found = Some(placement);
                        break;
                    }
                }
                found
            }
        {
            let viewport = st.scroll.viewport_len();
            let offset = st.scroll.offset();
            st.scroll.ensure_visible_on_next_layout(
                self.reveal_index(placement, viewport, offset, data),
            );
        }

        let enter_requested = self.enter == EnterPolicy::SubmitsWhenIdle
            && self.fields.iter().any(|field| {
                self.shown(data, field)
                    && !matches!(field.kind, FieldKind::Chooser(_))
                    && cx.state(field.id).contains(StateFlags::FOCUSED)
                    && !cx.state(field.id).contains(StateFlags::EDITING)
                    && !cx.swallows_typing(field.id)
                    && cx
                        .claim_binding_chord(field.id, Chord::key(KeyCode::Enter))
                        .is_some()
            });

        let mut acc = Acc::<FormAction>::new();
        acc.fold(&scroll_response);
        let mut first = None;
        let mut placements = self.placements(cx.design(), width);
        while let Some(placement) = placements.next(data) {
            let Some(field) = self.fields.get(placement.index) else {
                continue;
            };
            if !pointer_held && cx.state(field.id).contains(StateFlags::FOCUSED) {
                let viewport = st.scroll.viewport_len();
                let offset = st.scroll.offset();
                st.scroll.ensure_visible_on_next_layout(
                    self.reveal_index(placement, viewport, offset, data),
                );
            }
            let disabled = data.disabled(field.id);
            let (value, options) = data.value_and_options(field.id);
            let Some(slot) = Self::prepare_slot(st, placement.index, field, &value) else {
                continue;
            };
            let response = Self::update_field(field, cx, slot, value, options, disabled);
            if !pointer_held
                && cx.state(field.id).contains(StateFlags::FOCUSED)
                && matches!(field.kind, FieldKind::Radio(_))
            {
                let cursor_row = placement
                    .top
                    .saturating_add(1)
                    .saturating_add(slot.radio.cursor_index());
                st.scroll.ensure_visible_on_next_layout(cursor_row);
            }
            if let Some(action) = response.action_ref().copied() {
                if matches!(action, FormAction::Committed(_)) {
                    st.dirty = true;
                }
                Self::remember_action(&mut first, action);
            }
            acc.fold(&response.erase());
        }

        let mut requested = enter_requested.then_some(self.submit);
        for (index, action) in self.actions.iter().enumerate() {
            let action_id = self.id.part(Part::ACTIONS).index(index);
            let response = Button::new(action_id, action.label())
                .variant(action.variant())
                .disabled(!action.is_enabled())
                .update(cx);
            let domain_action = action.is_enabled()
                && cx
                    .intents(action_id)
                    .any(|intent| matches!(intent, Intent::Binding(key) if key == action.key()));
            if (response.activated() || domain_action) && requested.is_none() {
                requested = Some(action.key());
            }
            acc.fold(&response.erase());
        }
        if let Some(key) = requested {
            let action = if key == self.submit {
                self.submit_form(cx, st, data)
            } else {
                FormAction::Action(key)
            };
            if key == self.submit {
                first = Some(action);
            } else {
                Self::remember_action(&mut first, action);
            }
        }

        match first {
            Some(action) => Response::action(action).for_id(self.id),
            None => acc.finish(self.id),
        }
    }

    fn update_lanes<D: FormData + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut FormState,
        data: &mut D,
    ) -> Response<FormAction> {
        st.reconcile_fields_with_data(self.fields, data);
        let pointer_held = self.sync_pointer_hold(cx, st, data);

        let area = cx
            .layout(self.id)
            .and_then(|facts| facts.logical_area)
            .or_else(|| cx.area(self.id))
            .unwrap_or_default();
        let body = Rect {
            height: area
                .height
                .saturating_sub(u16::from(!self.actions.is_empty())),
            ..area
        };
        let plans = self.lane_plans(cx.design(), body, data);
        let mut acc = Acc::<FormAction>::new();
        let mut first = None;
        for plan in &plans {
            let response = ScrollRegion::new(lane_scroll_id(self.id, plan.id))
                .patch_part(self.parts)
                .update(cx, st.lane_scroll_mut(plan.id), plan.content_rows);
            acc.fold(&response.erase());
        }
        if !pointer_held
            && let Some(id) = st.reveal.take()
            && let Some((plan, placement)) = plans.iter().find_map(|plan| {
                plan.placements
                    .iter()
                    .find(|placement| self.fields.get(placement.index).is_some_and(|f| f.id == id))
                    .map(|placement| (plan, placement))
            })
        {
            let viewport = st.lane_scroll(plan.id).viewport_len();
            let offset = st.lane_scroll(plan.id).offset();
            st.lane_scroll_mut(plan.id).ensure_visible_on_next_layout(
                self.reveal_index(*placement, viewport, offset, data),
            );
        }

        let placed: Vec<usize> = plans
            .iter()
            .flat_map(|plan| plan.placements.iter().map(|placement| placement.index))
            .collect();
        let enter_requested = self.enter == EnterPolicy::SubmitsWhenIdle
            && placed.iter().any(|index| {
                self.fields.get(*index).is_some_and(|field| {
                    !matches!(field.kind, FieldKind::Chooser(_))
                        && cx.state(field.id).contains(StateFlags::FOCUSED)
                        && !cx.state(field.id).contains(StateFlags::EDITING)
                        && !cx.swallows_typing(field.id)
                        && cx
                            .claim_binding_chord(field.id, Chord::key(KeyCode::Enter))
                            .is_some()
                })
            });

        for plan in &plans {
            for placement in &plan.placements {
                let Some(field) = self.fields.get(placement.index) else {
                    continue;
                };
                if !pointer_held && cx.state(field.id).contains(StateFlags::FOCUSED) {
                    let viewport = st.lane_scroll(plan.id).viewport_len();
                    let offset = st.lane_scroll(plan.id).offset();
                    st.lane_scroll_mut(plan.id).ensure_visible_on_next_layout(
                        self.reveal_index(*placement, viewport, offset, data),
                    );
                }
                let disabled = data.disabled(field.id);
                let (value, options) = data.value_and_options(field.id);
                let Some(slot) = Self::prepare_slot(st, placement.index, field, &value) else {
                    continue;
                };
                let response = Self::update_field(field, cx, slot, value, options, disabled);
                if !pointer_held
                    && cx.state(field.id).contains(StateFlags::FOCUSED)
                    && matches!(field.kind, FieldKind::Radio(_))
                {
                    let cursor_row = placement
                        .top
                        .saturating_add(1)
                        .saturating_add(slot.radio.cursor_index());
                    st.lane_scroll_mut(plan.id)
                        .ensure_visible_on_next_layout(cursor_row);
                }
                if let Some(action) = response.action_ref().copied() {
                    if matches!(action, FormAction::Committed(_)) {
                        st.dirty = true;
                    }
                    Self::remember_action(&mut first, action);
                }
                acc.fold(&response.erase());
            }
        }

        let mut requested = enter_requested.then_some(self.submit);
        for (index, action) in self.actions.iter().enumerate() {
            let action_id = self.id.part(Part::ACTIONS).index(index);
            let response = Button::new(action_id, action.label())
                .variant(action.variant())
                .disabled(!action.is_enabled())
                .update(cx);
            let domain_action = action.is_enabled()
                && cx
                    .intents(action_id)
                    .any(|intent| matches!(intent, Intent::Binding(key) if key == action.key()));
            if (response.activated() || domain_action) && requested.is_none() {
                requested = Some(action.key());
            }
            acc.fold(&response.erase());
        }
        if let Some(key) = requested {
            let action = if key == self.submit {
                self.submit_form(cx, st, data)
            } else {
                FormAction::Action(key)
            };
            if key == self.submit {
                first = Some(action);
            } else {
                Self::remember_action(&mut first, action);
            }
        }
        match first {
            Some(action) => Response::action(action).for_id(self.id),
            None => acc.finish(self.id),
        }
    }

    fn field_error<'d, D: FormData + ?Sized>(
        st: &'d FormState,
        data: &'d D,
        field: &FieldSpec<'_>,
    ) -> Option<&'d str> {
        let id = field.id;
        let sensitive = field_is_secret(field) || matches!(data.value(id), FieldRef::Secret(_));
        if sensitive {
            return (st.error_message(id, true).is_some() || data.error(id).is_some())
                .then_some("Invalid value");
        }
        st.error_message(id, false).or_else(|| data.error(id))
    }

    fn safe_error<D: FormData + ?Sized>(
        data: &D,
        declared_secret: bool,
        id: Id,
        error: FieldError,
    ) -> FieldError {
        if declared_secret || matches!(data.value(id), FieldRef::Secret(_)) {
            discard_error(error);
            FieldError::new("Invalid value")
        } else {
            error
        }
    }

    pub fn draw<D: FormData + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &FormState,
        data: &D,
    ) -> Rect {
        if let Err(error) = self.validate_layout(data) {
            ui.diagnostic(Diagnostic::InvalidComponentLayout {
                owner: self.id,
                field: error.field_id(),
            });
            return area;
        }
        if area.is_empty() {
            return area;
        }
        if self.layout.is_some() {
            return self.draw_lanes(ui, area, st, data);
        }
        let content_rows = Self::content_rows(self.placements(ui.design(), area.width), data);
        let actions_height = u16::from(!self.actions.is_empty());
        let body = Rect {
            height: area.height.saturating_sub(actions_height),
            ..area
        };
        let scroll = ScrollRegion::new(self.id).patch_part(self.parts);
        let content = scroll.draw(ui, body, &st.scroll, content_rows);
        let view = ScrollRegion::view(&st.scroll, content, content_rows);
        let base = view.offset();
        let form_style = self.ov.style(
            ui,
            self.id,
            Family::FORM,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        ui.fill(content, form_style.style);
        ui.register_decor(self.id, PartRef::of(Part::BODY), content);
        let mut placements = self.placements(ui.design(), area.width);
        while let Some(placement) = placements.next(data) {
            let Some(field) = self.fields.get(placement.index) else {
                continue;
            };
            let fallback;
            let slot = if let Some(slot) = st.slots.get(placement.index) {
                slot
            } else {
                fallback = FieldSlot::new(field.id);
                &fallback
            };
            let Some(window) = resolve_field_window(content, placement, base) else {
                self.register_clipped_field(ui, field, data, placement.height);
                continue;
            };
            self.draw_control(
                ui,
                window.area,
                window.source_first_row,
                window.logical_height,
                field,
                slot,
                st,
                data,
            );
        }
        self.draw_actions(ui, area);
        ui.report_layout(
            self.id,
            LayoutFacts::new(
                usize::from(body.height),
                content_rows,
                area.height,
                area.width,
            ),
        );
        area
    }

    fn draw_lanes<D: FormData + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &FormState,
        data: &D,
    ) -> Rect {
        let body = Rect {
            height: area
                .height
                .saturating_sub(u16::from(!self.actions.is_empty())),
            ..area
        };
        let plans = self.lane_plans(ui.design(), body, data);
        let form_style = self.ov.style(
            ui,
            self.id,
            Family::FORM,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        let mut max_rows = 0;
        for plan in &plans {
            max_rows = max_rows.max(plan.content_rows);
            let scroll_id = lane_scroll_id(self.id, plan.id);
            let scroll_state = st.lane_scroll(plan.id);
            let scroll = ScrollRegion::new(scroll_id).patch_part(self.parts);
            let content = scroll.draw(ui, plan.area, &scroll_state, plan.content_rows);
            ui.fill(content, form_style.style);
            ui.register_decor(scroll_id, PartRef::of(Part::BODY), content);
            let view = ScrollRegion::view(&scroll_state, plan.area, plan.content_rows);
            let base = view.offset();
            for placement in &plan.placements {
                let Some(field) = self.fields.get(placement.index) else {
                    continue;
                };
                let Some(window) = resolve_field_window(content, *placement, base) else {
                    self.register_clipped_field(ui, field, data, placement.height);
                    continue;
                };
                let fallback;
                let slot = if let Some(slot) = st.slots.get(placement.index) {
                    slot
                } else {
                    fallback = FieldSlot::new(field.id);
                    &fallback
                };
                self.draw_control(
                    ui,
                    window.area,
                    window.source_first_row,
                    window.logical_height,
                    field,
                    slot,
                    st,
                    data,
                );
            }
        }
        self.draw_actions(ui, area);

        ui.report_layout(
            self.id,
            LayoutFacts::new(usize::from(body.height), max_rows, area.height, area.width)
                .with_logical_area(area),
        );
        area
    }

    fn draw_actions(&self, ui: &mut Ui<'_>, area: Rect) {
        if self.actions.is_empty() {
            return;
        }
        let row = Rect {
            y: area.bottom().saturating_sub(1),
            height: 1,
            ..area
        };
        ui.register_decor(self.id, PartRef::of(Part::ACTIONS), row);
        let mut x = row.right();
        for (index, action) in self.actions.iter().enumerate().rev() {
            let action_id = self.id.part(Part::ACTIONS).index(index);
            let button = Button::new(action_id, action.label())
                .variant(action.variant())
                .disabled(!action.is_enabled());
            let chord_width = ui
                .effective_chord(action_id, action.key(), action.chord_ref())
                .map_or(0, |chord| {
                    width(ChordText::of(chord).as_str()).saturating_add(1)
                });
            let button_width = button
                .measure(ui, Constraints::loose(row.width, 1))
                .preferred
                .0
                .saturating_add(chord_width)
                .min(x.saturating_sub(row.x));
            x = x.saturating_sub(button_width);
            let button_area = Rect {
                x,
                width: button_width,
                ..row
            };
            button.draw(ui, button_area);
            if action.is_enabled() {
                ui.publish_dynamic_bindings(
                    action_id,
                    ui.state(action_id),
                    core::iter::once((action.key(), action.chord_ref())),
                );
            }
            if let Some(chord) = ui.effective_chord(action_id, action.key(), action.chord_ref()) {
                let text = ChordText::of(chord);
                let key_width = width(text.as_str()).min(button_area.width);
                let key = Rect {
                    x: button_area.right().saturating_sub(key_width),
                    width: key_width,
                    ..button_area
                };
                let style = ui.resolve(
                    Family::BUTTON,
                    action.variant(),
                    Part::LABEL,
                    ui.state(action_id),
                );
                ui.paint_str(key, text.as_str(), style.style);
            }
            x = x.saturating_sub(1);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_control<D: FormData + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        source_first_row: usize,
        logical_height: u16,
        field: &FieldSpec<'a>,
        slot: &FieldSlot,
        st: &FormState,
        data: &D,
    ) {
        let disabled = data.disabled(field.id);
        let error = Self::field_error(st, data, field);
        let options = data.options(field.id);
        if self.sizing == FieldSizingPolicy::Intrinsic
            && let (FieldKind::Note, FieldRef::Note(rows)) = (&field.kind, data.value(field.id))
        {
            for (row, (text, role)) in area.rows().zip(rows.iter().skip(source_first_row)) {
                ui.paint_str(row, text, ui.paint_patch(&StylePatch::new().set_fg(*role)));
                ui.register_decor(field.id, PartRef::of(Part::HELP), row);
            }
            return;
        }
        if self.sizing == FieldSizingPolicy::Intrinsic
            && let (FieldKind::Chooser(control), FieldRef::Display { value, detail }) =
                (&field.kind, data.value(field.id))
        {
            let native_button_visible = self.draw_intrinsic_chooser(
                ui,
                area,
                source_first_row,
                logical_height,
                field,
                control,
                value,
                detail,
                disabled,
            );
            if logical_height > 1 && !native_button_visible {
                control.register_focus_only_in_form(ui, disabled);
            }
            return;
        }
        if let (FieldKind::Radio(control), FieldRef::Choice(value)) =
            (&field.kind, data.value(field.id))
        {
            let (body, first_body_row) =
                self.draw_chrome_window(ui, area, field, error, source_first_row, logical_height);
            if !options.is_empty() {
                if native_body_visible(ui, native_radio_area(body, first_body_row, options.len())) {
                    control.draw_in_form_window(
                        ui,
                        body,
                        &slot.radio,
                        value,
                        options,
                        InheritedFormState { disabled },
                        first_body_row,
                    );
                } else {
                    control.register_focus_only_in_form(ui, disabled);
                }
            }
            return;
        }
        let (control_area, body_first_row) =
            self.draw_chrome_window(ui, area, field, error, source_first_row, logical_height);
        match (&field.kind, data.value(field.id)) {
            (FieldKind::Text(control), FieldRef::Text(value)) if logical_height >= 3 => {
                if native_first_row_visible(ui, control_area) {
                    control.draw_in_form(ui, control_area, &slot.input, value, disabled);
                } else {
                    control.register_focus_only_in_form(ui, disabled);
                }
            }
            (FieldKind::Text(control), FieldRef::Secret(value)) if logical_height >= 3 => {
                if native_first_row_visible(ui, control_area) {
                    control.draw_secret_in_form(ui, control_area, &slot.input, value, disabled);
                } else {
                    control.register_focus_only_in_form(ui, disabled);
                }
            }
            (FieldKind::Area(control), FieldRef::Text(value)) => {
                let logical_rows = control.rows_in_form().min(logical_height.saturating_sub(2));
                let window = textarea_window(
                    control_area,
                    body_first_row,
                    logical_height,
                    control.rows_in_form(),
                );
                if let Some(window) =
                    window.filter(|window| native_body_visible(ui, window.visible_body()))
                {
                    control.draw_in_form_window(ui, window, &slot.area, value, disabled);
                } else if logical_rows > 0 {
                    control.register_focus_only_in_form(ui, disabled);
                }
            }
            (FieldKind::Area(control), FieldRef::Secret(value)) => {
                let logical_rows = control.rows_in_form().min(logical_height.saturating_sub(2));
                let window = textarea_window(
                    control_area,
                    body_first_row,
                    logical_height,
                    control.rows_in_form(),
                );
                if let Some(window) =
                    window.filter(|window| native_body_visible(ui, window.visible_body()))
                {
                    control.draw_secret_in_form_window(ui, window, &slot.area, value, disabled);
                } else if logical_rows > 0 {
                    control.register_focus_only_in_form(ui, disabled);
                }
            }
            (FieldKind::Select(control), FieldRef::Choice(value)) if logical_height >= 3 => {
                if native_first_row_visible(ui, control_area) {
                    control.draw_in_form(
                        ui,
                        control_area,
                        &slot.select,
                        value,
                        options,
                        InheritedFormState { disabled },
                    );
                } else {
                    control.register_focus_only_in_form(ui, disabled);
                }
            }
            (FieldKind::Radio(_), FieldRef::Choice(_)) => unreachable!("radio handled above"),
            (FieldKind::Chips(control), FieldRef::Chips(_)) if logical_height >= 3 => {
                if native_first_row_visible(ui, control_area) {
                    control.draw_in_form(ui, control_area, &slot.chips, options, disabled);
                } else {
                    control.register_focus_only_in_form(ui, disabled);
                }
            }
            (FieldKind::Check(control), FieldRef::Flag(value)) if logical_height >= 3 => {
                if native_first_row_visible(ui, control_area) {
                    control.draw_in_form(ui, control_area, value, disabled);
                } else {
                    control.register_focus_only_in_form(ui, disabled);
                }
            }
            (FieldKind::Toggle(control), FieldRef::Flag(value)) if logical_height >= 3 => {
                if native_first_row_visible(ui, control_area) {
                    control.draw_in_form(ui, control_area, value, disabled);
                } else {
                    control.register_focus_only_in_form(ui, disabled);
                }
            }
            (FieldKind::Chooser(control), FieldRef::Display { value, detail })
                if logical_height >= 3 =>
            {
                let painted =
                    if native_body_visible(ui, native_button_area(ui, control, control_area)) {
                        control.draw_in_form(ui, control_area, disabled)
                    } else {
                        control.register_focus_only_in_form(ui, disabled);
                        Rect {
                            width: 0,
                            height: 0,
                            ..control_area
                        }
                    };
                let value_area = Rect {
                    x: painted.right().saturating_add(1),
                    width: control_area
                        .right()
                        .saturating_sub(painted.right())
                        .saturating_sub(1),
                    ..control_area
                };
                ui.paint_str(value_area, value, ui.surface_style());
                if let Some(detail) = detail {
                    let detail_area = Rect {
                        y: control_area.y.saturating_add(1),
                        height: 1,
                        ..control_area
                    };
                    ui.paint_str(detail_area, detail, ui.surface_style());
                }
            }
            (FieldKind::Note, FieldRef::Note(rows)) => {
                let style = self.ov.style(
                    ui,
                    self.id,
                    Family::FORM,
                    Variant::DEFAULT,
                    Part::HELP,
                    StateFlags::empty(),
                );
                for (row, (text, _role)) in area.rows().zip(rows.iter().skip(source_first_row)) {
                    ui.paint_str(row, text, style.style);
                    ui.register_decor(field.id, PartRef::of(Part::HELP), row);
                }
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_intrinsic_chooser(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        source_first_row: usize,
        logical_height: u16,
        field: &FieldSpec<'_>,
        control: &Button<'_>,
        value: &str,
        detail: Option<&str>,
        disabled: bool,
    ) -> bool {
        let mut native_button_visible = false;
        let source_end = source_first_row.saturating_add(usize::from(area.height));
        for source_row in source_first_row..source_end.min(usize::from(logical_height)) {
            let row = Rect {
                y: area.y.saturating_add(
                    source_row
                        .saturating_sub(source_first_row)
                        .min(usize::from(u16::MAX)) as u16,
                ),
                height: 1,
                ..area
            };
            match source_row {
                0 => {
                    let label_style = self.ov.style(
                        ui,
                        self.id,
                        Family::FIELD,
                        Variant::DEFAULT,
                        Part::LABEL,
                        StateFlags::empty(),
                    );
                    let label = Rect {
                        x: row.x.saturating_add(2),
                        width: row.width.saturating_sub(2),
                        ..row
                    };
                    let used = ui.paint_str(label, field.label, label_style.style);
                    if field.required && !field.label.is_empty() {
                        ui.paint_str(
                            Rect {
                                x: label.x.saturating_add(used).saturating_add(1),
                                width: 1,
                                ..label
                            },
                            "*",
                            label_style.style,
                        );
                    }
                    ui.register_decor(field.id, PartRef::of(Part::LABEL), row);
                }
                1 => {
                    let value_x = row.x.saturating_add(2);
                    let right = row.right().saturating_sub(1);
                    let button_width = control
                        .measure(ui, Constraints::loose(u16::MAX, 1))
                        .preferred
                        .0
                        .min(right.saturating_sub(value_x));
                    let button_area = Rect {
                        x: right.saturating_sub(button_width),
                        width: button_width,
                        ..row
                    };
                    let value_area = Rect {
                        x: value_x,
                        width: button_area.x.saturating_sub(value_x).saturating_sub(1),
                        ..row
                    };
                    ui.paint_str(value_area, value, ui.surface_style());
                    let native_button = control.draw_in_form(ui, button_area, disabled);
                    native_button_visible |= native_body_visible(ui, native_button);
                }
                2 => {
                    if let Some(detail) = detail {
                        ui.paint_str(
                            Rect {
                                x: row.x.saturating_add(2),
                                width: row.width.saturating_sub(3),
                                ..row
                            },
                            detail,
                            ui.surface_style(),
                        );
                    }
                }
                _ => {}
            }
        }
        native_button_visible
    }

    fn draw_chrome_window(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        field: &FieldSpec<'_>,
        error: Option<&str>,
        source_first_row: usize,
        logical_height: u16,
    ) -> (Rect, usize) {
        if area.is_empty() {
            return (area, 0);
        }
        let source_end = source_first_row.saturating_add(usize::from(area.height));
        if source_first_row == 0 {
            let label = Rect { height: 1, ..area };
            let style = self.ov.style(
                ui,
                self.id,
                Family::FIELD,
                Variant::DEFAULT,
                Part::LABEL,
                StateFlags::empty(),
            );
            let text = Rect {
                x: label.x.saturating_add(2),
                width: label.width.saturating_sub(2),
                ..label
            };
            let used = ui.paint_str(text, field.label, style.style);
            if field.required && !field.label.is_empty() {
                ui.paint_str(
                    Rect {
                        x: text.x.saturating_add(used).saturating_add(1),
                        width: 1,
                        ..text
                    },
                    "*",
                    style.style,
                );
            }
            ui.register_decor(field.id, PartRef::of(Part::LABEL), label);
        }

        let has_help = logical_height >= 3;
        let help_source_row = usize::from(logical_height.saturating_sub(1));
        if has_help
            && source_first_row <= help_source_row
            && help_source_row < source_end
            && let Some(message) = error.or(field.help)
        {
            let row = Rect {
                x: area.x,
                y: area
                    .y
                    .saturating_add((help_source_row - source_first_row) as u16),
                width: area.width,
                height: 1,
            };
            let help = Rect {
                x: row.x.saturating_add(2),
                width: row.width.saturating_sub(2),
                ..row
            };
            let flags = if error.is_some() {
                StateFlags::ERROR
            } else {
                StateFlags::empty()
            };
            let style = self.ov.style(
                ui,
                self.id,
                Family::FIELD,
                Variant::DEFAULT,
                Part::HELP,
                flags,
            );
            ui.paint_str(help, message, style.style);
            ui.register_decor(field.id, PartRef::of(Part::HELP), row);
        }

        let body_start = 1usize.max(source_first_row);
        let source_body_end = if has_help {
            usize::from(logical_height.saturating_sub(1))
        } else {
            1
        };
        let body_end = source_body_end.min(source_end);
        if body_start >= body_end {
            return (
                Rect {
                    y: area.y.saturating_add(
                        body_start
                            .saturating_sub(source_first_row)
                            .min(usize::from(u16::MAX)) as u16,
                    ),
                    height: 0,
                    ..area
                },
                body_start.saturating_sub(1),
            );
        }
        let body = Rect {
            y: area
                .y
                .saturating_add((body_start - source_first_row).min(usize::from(u16::MAX)) as u16),
            height: (body_end - body_start).min(usize::from(u16::MAX)) as u16,
            ..area
        };
        (body, body_start.saturating_sub(1))
    }

    pub fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        let rows = self
            .fields
            .iter()
            .filter(|field| field.group == GroupKey::ALL || field.group == self.group)
            .fold(0u16, |sum, field| {
                sum.saturating_add(Self::field_height(field, ui.design()))
            })
            .saturating_add(u16::from(!self.actions.is_empty()));
        Size {
            min: (8, rows.min(1)),
            preferred: (constraints.max.0, rows),
        }
        .fit(constraints)
    }

    pub fn measure_with_data<D: FormData + ?Sized>(
        &self,
        ui: &Ui<'_>,
        constraints: Constraints,
        data: &D,
    ) -> Size {
        self.measure_with_data_design(ui.design(), constraints, data)
    }

    pub(crate) fn measure_with_data_design<D: FormData + ?Sized>(
        &self,
        design: &DesignTokens,
        constraints: Constraints,
        data: &D,
    ) -> Size {
        if self.layout.is_some() {
            if self.validate_layout(data).is_err() {
                return Size::default().fit(constraints);
            }
            let body_height = constraints
                .max
                .1
                .saturating_sub(u16::from(!self.actions.is_empty()));
            let plans = self.lane_plans(
                design,
                Rect::new(0, 0, constraints.max.0, body_height),
                data,
            );
            let rows = plans
                .iter()
                .map(|plan| plan.content_rows)
                .max()
                .unwrap_or(0)
                .saturating_add(usize::from(!self.actions.is_empty()))
                .try_into()
                .unwrap_or(u16::MAX);
            return Size {
                min: (8, rows.min(1)),
                preferred: (constraints.max.0, rows),
            }
            .fit(constraints);
        }
        let rows = Self::content_rows(self.placements(design, constraints.max.0), data)
            .saturating_add(usize::from(!self.actions.is_empty()))
            .try_into()
            .unwrap_or(u16::MAX);
        Size {
            min: (8, rows.min(1)),
            preferred: (constraints.max.0, rows),
        }
        .fit(constraints)
    }
}
