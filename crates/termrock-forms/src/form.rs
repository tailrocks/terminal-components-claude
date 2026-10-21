use core::fmt;
use std::borrow::Cow;

use ratatui_core::layout::Rect;

use super::{Acc, PartStyle};
use crate::action::{Action, ActionKey};
use crate::event::{Chord, KeyCode};
use crate::focus::Focusability;
use crate::form_layout::{FormLayout, FormLayoutError, FormRow};
use crate::id::{Id, Part, PartRef, fnv1a};
use crate::intent::Intent;
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

/// Controls whether content-only fields size themselves from their view data.
///
/// `Fixed` preserves the original Form geometry. `Intrinsic` gives notes one
/// row per supplied line and gives choosers the rows their displayed value
/// needs. Other field kinds keep their existing sizing rules.
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

    #[cfg(test)]
    fn reconcile_fields(&mut self, fields: &[FieldSpec<'_>]) {
        self.reconcile_fields_with_sensitivity(fields, field_is_secret);
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

    /// Sizes notes and chooser summaries from their supplied view data.
    ///
    /// The default [`FieldSizingPolicy::Fixed`] preserves existing geometry.
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

    /// Apply a borrowed semantic lane layout after checking its static shape
    /// and field references. Active-group/data visibility completeness is
    /// checked by [`Self::validate_layout`] before each rendering phase.
    pub fn try_layout(mut self, layout: &'a FormLayout<'a>) -> Result<Self, FormLayoutError> {
        layout.validate(self.fields.iter().map(|field| field.id))?;
        self.layout = Some(layout);
        Ok(self)
    }

    /// Validate the final builder configuration against the current group and
    /// live visibility data without reading field values or mutating state.
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

    fn reveal_index(placement: Placement, viewport: usize) -> usize {
        let bottom = placement
            .top
            .saturating_add(usize::from(placement.height).saturating_sub(1));
        if viewport > 0 && usize::from(placement.height) <= viewport {
            bottom
        } else {
            placement.top
        }
    }

    fn register_clipped_field<D: FormData + ?Sized>(
        ui: &mut Ui<'_>,
        field: &FieldSpec<'_>,
        data: &D,
    ) {
        if ui.is_inert() {
            return;
        }
        let focus = if data.disabled(field.id) {
            Focusability::Disabled
        } else {
            Focusability::Focusable
        };
        ui.register_focus_only(field.id, focus);
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
            _ => Vec::new(), // Construction preflight makes this unreachable.
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
        let width = cx.area(self.id).map_or(0, |area| area.width);
        let content_rows = Self::content_rows(self.placements(cx.design(), width), data);
        let scroll = ScrollRegion::new(self.id).patch_part(self.parts);
        let scroll_response = scroll.update(cx, &mut st.scroll, content_rows);
        if let Some(id) = st.reveal.take()
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
            st.scroll
                .ensure_visible_on_next_layout(Self::reveal_index(placement, viewport));
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
            if cx.state(field.id).contains(StateFlags::FOCUSED) {
                let viewport = st.scroll.viewport_len();
                st.scroll
                    .ensure_visible_on_next_layout(Self::reveal_index(placement, viewport));
            }
            let disabled = data.disabled(field.id);
            let (value, options) = data.value_and_options(field.id);
            let Some(slot) = Self::prepare_slot(st, placement.index, field, &value) else {
                continue;
            };
            let response = Self::update_field(field, cx, slot, value, options, disabled);
            if cx.state(field.id).contains(StateFlags::FOCUSED)
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
        // A Form is a logical owner, not a covering hit region. Its composed
        // bounds are published with the preceding draw and remain available
        // here even when every hit-test region belongs to a child control.
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
        if let Some(id) = st.reveal.take()
            && let Some((plan, placement)) = plans.iter().find_map(|plan| {
                plan.placements
                    .iter()
                    .find(|placement| self.fields.get(placement.index).is_some_and(|f| f.id == id))
                    .map(|placement| (plan, placement))
            })
        {
            let viewport = st.lane_scroll(plan.id).viewport_len();
            st.lane_scroll_mut(plan.id)
                .ensure_visible_on_next_layout(Self::reveal_index(*placement, viewport));
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
                if cx.state(field.id).contains(StateFlags::FOCUSED) {
                    let viewport = st.lane_scroll(plan.id).viewport_len();
                    st.lane_scroll_mut(plan.id)
                        .ensure_visible_on_next_layout(Self::reveal_index(*placement, viewport));
                }
                let disabled = data.disabled(field.id);
                let (value, options) = data.value_and_options(field.id);
                let Some(slot) = Self::prepare_slot(st, placement.index, field, &value) else {
                    continue;
                };
                let response = Self::update_field(field, cx, slot, value, options, disabled);
                if cx.state(field.id).contains(StateFlags::FOCUSED)
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
                Self::register_clipped_field(ui, field, data);
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
                    Self::register_clipped_field(ui, field, data);
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
        // Publish logical bounds for the next update pass without placing a
        // parent hit region over the child fields and action buttons.
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
            self.draw_intrinsic_chooser(
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
            return;
        }
        if let (FieldKind::Radio(control), FieldRef::Choice(value)) =
            (&field.kind, data.value(field.id))
        {
            let (body, first_body_row) =
                self.draw_chrome_window(ui, area, field, error, source_first_row, logical_height);
            control.draw_in_form_window(
                ui,
                body,
                &slot.radio,
                value,
                options,
                InheritedFormState { disabled },
                first_body_row,
            );
            return;
        }
        let (control_area, body_first_row) =
            self.draw_chrome_window(ui, area, field, error, source_first_row, logical_height);
        match (&field.kind, data.value(field.id)) {
            (FieldKind::Text(control), FieldRef::Text(value)) => {
                control.draw_in_form(ui, control_area, &slot.input, value, disabled);
            }
            (FieldKind::Text(control), FieldRef::Secret(value)) => {
                control.draw_secret_in_form(ui, control_area, &slot.input, value, disabled);
            }
            (FieldKind::Area(control), FieldRef::Text(value)) => {
                if let Some(window) = textarea_window(
                    control_area,
                    body_first_row,
                    logical_height,
                    control.rows_in_form(),
                ) {
                    control.draw_in_form_window(ui, window, &slot.area, value, disabled);
                }
            }
            (FieldKind::Area(control), FieldRef::Secret(value)) => {
                if let Some(window) = textarea_window(
                    control_area,
                    body_first_row,
                    logical_height,
                    control.rows_in_form(),
                ) {
                    control.draw_secret_in_form_window(ui, window, &slot.area, value, disabled);
                }
            }
            (FieldKind::Select(control), FieldRef::Choice(value)) => {
                control.draw_in_form(
                    ui,
                    control_area,
                    &slot.select,
                    value,
                    options,
                    InheritedFormState { disabled },
                );
            }
            (FieldKind::Radio(_), FieldRef::Choice(_)) => unreachable!("radio handled above"),
            (FieldKind::Chips(control), FieldRef::Chips(_)) => {
                control.draw_in_form(ui, control_area, &slot.chips, options, disabled);
            }
            (FieldKind::Check(control), FieldRef::Flag(value)) => {
                control.draw_in_form(ui, control_area, value, disabled);
            }
            (FieldKind::Toggle(control), FieldRef::Flag(value)) => {
                control.draw_in_form(ui, control_area, value, disabled);
            }
            (FieldKind::Chooser(control), FieldRef::Display { value, detail }) => {
                let painted = control.draw_in_form(ui, control_area, disabled);
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
    ) {
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
                    control.draw_in_form(ui, button_area, disabled);
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

        // Form fields reserve source row zero for the label and the final
        // source row for help, matching draw_chrome's unscrolled contract.
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

    /// Estimates the legacy linear layout without consulting [`FormData`].
    ///
    /// This compatibility method cannot account for data-dependent visibility,
    /// radio option counts, or the row pairing used when `columns(2)` is set.
    /// This estimate also ignores an explicit lane layout. Consumers with
    /// form data or [`FieldSizingPolicy::Intrinsic`] should use
    /// [`Self::measure_with_data`].
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

    /// Measures the visible fields using the same placement plan as update and draw.
    ///
    /// Measurement reads visibility and option data only. It does not update
    /// controls, validate values, invoke domain handlers, or mutate form state.
    pub fn measure_with_data<D: FormData + ?Sized>(
        &self,
        ui: &Ui<'_>,
        constraints: Constraints,
        data: &D,
    ) -> Size {
        self.measure_with_data_design(ui.design(), constraints, data)
    }

    /// Measures with explicit design tokens so composed overlays can use the
    /// same placement policy before a [`Ui`] draw pass exists.
    ///
    /// This is crate-visible; consumers should use [`Self::measure_with_data`].
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

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::event::{Input, Key, KeyModifiers};
    use crate::form_layout::FormLane;
    use crate::id::ItemKey;
    use crate::keymap::KeyMap;
    use crate::runtime::App;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::secret::SecretPolicy;
    use crate::theme::Theme;

    const FORM: Id = Id::root("form.tests");
    const SCREEN: Rect = Rect::new(0, 0, 60, 30);
    const NAME: Id = Id::root("form.tests.name");
    const HIDDEN: Id = Id::root("form.tests.hidden");
    const FLAG: Id = Id::root("form.tests.flag");
    const CHOICE: Id = Id::root("form.tests.choice");
    const SECRET: Id = Id::root("form.tests.secret");
    const NOTE: Id = Id::root("form.tests.note");
    const CHOOSER: Id = Id::root("form.tests.chooser");
    const MATRIX: Id = Id::root("form.tests.matrix");
    const CLIP_A: Id = Id::root("form.tests.clip.a");
    const CLIP_B: Id = Id::root("form.tests.clip.b");
    const CLIP_C: Id = Id::root("form.tests.clip.c");
    const CLIP_RADIO: Id = Id::root("form.tests.clip.radio");
    const CLIP_AFTER: Id = Id::root("form.tests.clip.after");
    const CLIP_LANE: Id = Id::root("form.tests.clip.lane");
    const CLIP_OPTIONS: &[&str] = &[
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
    ];
    const SAVE_CHORD: Chord = Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL);
    const ACTIONS: &[Action<'static>] = &[Action::new(ActionKey::SAVE, "Save").chord(SAVE_CHORD)];
    const OPTIONS: &[&str] = &["one", "two", "three"];
    const OTHER_OPTIONS: &[&str] = &["alpha", "beta"];
    const NOTES: &[(&str, Role)] = &[("Read this", Role::Info)];

    #[derive(Default)]
    struct TestFlags {
        show_hidden: bool,
        disabled: bool,
        fail_name: bool,
    }

    struct Data {
        name: String,
        hidden: String,
        hidden_secret: Secret,
        hidden_secret_mode: bool,
        flag: bool,
        choice: usize,
        secret: Secret,
        secret_text: String,
        secret_mode: bool,
        chips: KeySet,
        flags: TestFlags,
    }

    impl Default for Data {
        fn default() -> Self {
            Data {
                name: String::new(),
                hidden: String::new(),
                hidden_secret: Secret::default(),
                hidden_secret_mode: false,
                flag: false,
                choice: 0,
                secret: Secret::default(),
                secret_text: String::new(),
                secret_mode: true,
                chips: KeySet::default(),
                flags: TestFlags::default(),
            }
        }
    }

    impl FormData for Data {
        fn value(&self, id: Id) -> FieldRef<'_> {
            match id {
                NAME => FieldRef::Text(&self.name),
                HIDDEN if self.hidden_secret_mode => FieldRef::Secret(&self.hidden_secret),
                HIDDEN => FieldRef::Text(&self.hidden),
                FLAG => FieldRef::Flag(self.flag),
                CHOICE => FieldRef::Choice(self.choice),
                SECRET if self.secret_mode => FieldRef::Secret(&self.secret),
                SECRET => FieldRef::Text(&self.secret_text),
                NOTE => FieldRef::Note(NOTES),
                CHOOSER => FieldRef::Display {
                    value: "chosen",
                    detail: Some("detail"),
                },
                _ => FieldRef::Chips(&self.chips),
            }
        }

        fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
            match id {
                NAME => FieldMut::Text(&mut self.name),
                HIDDEN if self.hidden_secret_mode => FieldMut::Secret(&mut self.hidden_secret),
                HIDDEN => FieldMut::Text(&mut self.hidden),
                FLAG => FieldMut::Flag(&mut self.flag),
                CHOICE => FieldMut::Choice(&mut self.choice),
                SECRET if self.secret_mode => FieldMut::Secret(&mut self.secret),
                SECRET => FieldMut::Text(&mut self.secret_text),
                NOTE | CHOOSER => FieldMut::ReadOnly,
                _ => FieldMut::Chips(&mut self.chips),
            }
        }

        fn options(&self, id: Id) -> &[&str] {
            if id == CHOICE { OPTIONS } else { &[] }
        }

        fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
            match id {
                CHOICE => (FieldMut::Choice(&mut self.choice), OPTIONS),
                _ => (self.value_mut(id), &[]),
            }
        }

        fn visible(&self, id: Id) -> bool {
            id != HIDDEN || self.flags.show_hidden
        }

        fn disabled(&self, id: Id) -> bool {
            id == FLAG && self.flags.disabled
        }

        fn validate(&self, id: Id, _value: FieldRef<'_>) -> Result<(), FieldError> {
            if id == NAME && self.flags.fail_name {
                Err(FieldError::new("name required"))
            } else {
                Ok(())
            }
        }
    }

    fn fields() -> [FieldSpec<'static>; 7] {
        fields_with_secret_policy(true)
    }

    fn fields_with_secret_policy(secret: bool) -> [FieldSpec<'static>; 7] {
        let secret_control = if secret {
            TextInput::new(SECRET).secret(SecretPolicy::default())
        } else {
            TextInput::new(SECRET)
        };
        [
            FieldSpec::new(NAME, "Name", FieldKind::Text(TextInput::new(NAME))),
            FieldSpec::new(HIDDEN, "Hidden", FieldKind::Text(TextInput::new(HIDDEN))),
            FieldSpec::new(FLAG, "", FieldKind::Check(Checkbox::new(FLAG, "Flag"))),
            FieldSpec::new(
                CHOICE,
                "Choice",
                FieldKind::Select(LabelSelect::new(CHOICE)),
            ),
            FieldSpec::new(SECRET, "Secret", FieldKind::Text(secret_control)),
            FieldSpec::new(NOTE, "", FieldKind::Note),
            FieldSpec::new(
                CHOOSER,
                "Chooser",
                FieldKind::Chooser(Button::new(CHOOSER, "Choose")),
            ),
        ]
    }

    fn fields_for_app(
        plain_secret_control: bool,
        inactive_secret: bool,
    ) -> ([FieldSpec<'static>; 7], GroupKey) {
        let mut fields = fields_with_secret_policy(!plain_secret_control);
        if inactive_secret {
            fields[1] = FieldSpec::new(HIDDEN, "Hidden", FieldKind::Text(TextInput::new(HIDDEN)))
                .group(GroupKey::custom("inactive-dynamic-secret"));
            (fields, GroupKey::custom("active-dynamic-secret"))
        } else {
            (fields, GroupKey::ALL)
        }
    }

    fn draw(data: &Data, state: &mut FormState) -> (Runtime<Stub>, Buffer) {
        let fields = fields();
        state.reconcile_fields(&fields);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                Form::new(FORM, &fields).draw(ui, area, state, data);
            })
            .commit_presented();
        (runtime, buffer)
    }

    fn initialized_secret_state() -> FormState {
        let mut app = FieldsApp::default();
        app.data.secret.set("swordfish");
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.app().state.clone()
    }

    fn draw_secret_field(data: &Data, state: &mut FormState, kind: FieldKind<'static>) -> Buffer {
        let fields = [FieldSpec::new(SECRET, "Secret", kind)];
        state.reconcile_fields(&fields);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                Form::new(FORM, &fields).draw(ui, area, state, data);
            })
            .commit_presented();
        buffer
    }

    fn press(code: KeyCode) -> Input {
        Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        })
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MatrixKind {
        Text,
        Area,
        Select,
        Radio,
        Chips,
        Check,
        Toggle,
        Chooser,
        Note,
    }

    fn matrix_field(kind: MatrixKind) -> FieldSpec<'static> {
        let field_kind = match kind {
            MatrixKind::Text => FieldKind::Text(TextInput::new(MATRIX)),
            MatrixKind::Area => FieldKind::Area(TextArea::new(MATRIX, 3)),
            MatrixKind::Select => FieldKind::Select(LabelSelect::new(MATRIX)),
            MatrixKind::Radio => FieldKind::Radio(LabelRadio::new(MATRIX)),
            MatrixKind::Chips => FieldKind::Chips(LabelChips::new(MATRIX)),
            MatrixKind::Check => FieldKind::Check(Checkbox::new(MATRIX, "Check")),
            MatrixKind::Toggle => FieldKind::Toggle(Toggle::new(MATRIX, "Toggle")),
            MatrixKind::Chooser => FieldKind::Chooser(Button::new(MATRIX, "Choose")),
            MatrixKind::Note => FieldKind::Note,
        };
        FieldSpec::new(MATRIX, "Field", field_kind)
    }

    #[derive(Default)]
    enum MatrixValidation {
        #[default]
        Accept,
        Reject,
    }

    #[derive(Default)]
    enum MatrixOptions {
        #[default]
        Primary,
        Alternate,
    }

    struct MatrixConfig {
        visible: bool,
        validation: MatrixValidation,
        options: MatrixOptions,
    }

    impl Default for MatrixConfig {
        fn default() -> Self {
            Self {
                visible: true,
                validation: MatrixValidation::Accept,
                options: MatrixOptions::Primary,
            }
        }
    }

    struct MatrixData {
        kind: MatrixKind,
        text: String,
        choice: usize,
        chips: KeySet,
        flag: bool,
        validations: Cell<usize>,
        config: MatrixConfig,
        notes: Vec<(&'static str, Role)>,
        detail: Option<&'static str>,
        display: &'static str,
    }

    impl MatrixData {
        fn new(kind: MatrixKind) -> Self {
            Self {
                kind,
                text: String::new(),
                choice: 0,
                chips: KeySet::default(),
                flag: false,
                validations: Cell::new(0),
                config: MatrixConfig::default(),
                notes: Vec::new(),
                detail: None,
                display: "chosen",
            }
        }
    }

    impl FormData for MatrixData {
        fn value(&self, _id: Id) -> FieldRef<'_> {
            match self.kind {
                MatrixKind::Text | MatrixKind::Area => FieldRef::Text(&self.text),
                MatrixKind::Select | MatrixKind::Radio => FieldRef::Choice(self.choice),
                MatrixKind::Chips => FieldRef::Chips(&self.chips),
                MatrixKind::Check | MatrixKind::Toggle => FieldRef::Flag(self.flag),
                MatrixKind::Chooser => FieldRef::Display {
                    value: self.display,
                    detail: self.detail,
                },
                MatrixKind::Note => FieldRef::Note(&self.notes),
            }
        }

        fn value_mut(&mut self, _id: Id) -> FieldMut<'_> {
            match self.kind {
                MatrixKind::Text | MatrixKind::Area => FieldMut::Text(&mut self.text),
                MatrixKind::Select | MatrixKind::Radio => FieldMut::Choice(&mut self.choice),
                MatrixKind::Chips => FieldMut::Chips(&mut self.chips),
                MatrixKind::Check | MatrixKind::Toggle => FieldMut::Flag(&mut self.flag),
                MatrixKind::Chooser | MatrixKind::Note => FieldMut::ReadOnly,
            }
        }

        fn options(&self, _id: Id) -> &[&str] {
            if matches!(self.config.options, MatrixOptions::Alternate) {
                OTHER_OPTIONS
            } else {
                OPTIONS
            }
        }

        fn value_and_options(&mut self, _id: Id) -> (FieldMut<'_>, &[&str]) {
            match self.kind {
                MatrixKind::Select | MatrixKind::Radio | MatrixKind::Chips => {
                    let options = if matches!(self.config.options, MatrixOptions::Alternate) {
                        OTHER_OPTIONS
                    } else {
                        OPTIONS
                    };
                    let kind = self.kind;
                    let value = match kind {
                        MatrixKind::Select | MatrixKind::Radio => {
                            FieldMut::Choice(&mut self.choice)
                        }
                        MatrixKind::Chips => FieldMut::Chips(&mut self.chips),
                        _ => FieldMut::ReadOnly,
                    };
                    (value, options)
                }
                _ => (self.value_mut(MATRIX), &[]),
            }
        }

        fn visible(&self, _id: Id) -> bool {
            self.config.visible
        }

        fn validate(&self, _id: Id, _value: FieldRef<'_>) -> Result<(), FieldError> {
            self.validations
                .set(self.validations.get().saturating_add(1));
            if matches!(self.config.validation, MatrixValidation::Reject) {
                Err(FieldError::new("invalid"))
            } else {
                Ok(())
            }
        }
    }

    struct MatrixApp {
        state: FormState,
        data: MatrixData,
        last: Option<FormAction>,
        keymap: KeyMap,
        enter: EnterPolicy,
        reference: bool,
        target: Option<crate::ReferenceTarget>,
        sizing: FieldSizingPolicy,
        area: Rect,
    }

    impl MatrixApp {
        fn new(kind: MatrixKind) -> Self {
            Self {
                state: FormState::default(),
                data: MatrixData::new(kind),
                last: None,
                keymap: KeyMap::new(),
                enter: EnterPolicy::SubmitsWhenIdle,
                reference: false,
                target: None,
                sizing: FieldSizingPolicy::Fixed,
                area: SCREEN,
            }
        }

        fn with_sizing(kind: MatrixKind, sizing: FieldSizingPolicy) -> Self {
            Self {
                sizing,
                ..Self::new(kind)
            }
        }
    }

    impl App for MatrixApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let fields = [matrix_field(self.data.kind)];
            let response = Form::new(FORM, &fields)
                .actions(ACTIONS)
                .enter(self.enter)
                .field_sizing(self.sizing)
                .update(cx, &mut self.state, &mut self.data);
            self.last = response.action_ref().copied();
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let fields = [matrix_field(self.data.kind)];
            let form = Form::new(FORM, &fields)
                .actions(ACTIONS)
                .enter(self.enter)
                .field_sizing(self.sizing);
            if self.reference {
                ui.reference(self.target, |ui| {
                    form.draw(ui, self.area, &self.state, &self.data);
                });
            } else {
                form.draw(ui, self.area, &self.state, &self.data);
            }
        }

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    fn matrix_runtime(kind: MatrixKind) -> (Runtime<MatrixApp>, Buffer) {
        matrix_runtime_with_sizing(kind, FieldSizingPolicy::Fixed)
    }

    fn matrix_runtime_with_sizing(
        kind: MatrixKind,
        sizing: FieldSizingPolicy,
    ) -> (Runtime<MatrixApp>, Buffer) {
        let mut runtime = Runtime::new(MatrixApp::with_sizing(kind, sizing), Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(MATRIX));
        (runtime, buffer)
    }

    struct FieldsApp {
        state: FormState,
        data: Data,
        last: Option<FormAction>,
        area: Rect,
        plain_secret_control: bool,
        inactive_secret: bool,
    }

    impl Default for FieldsApp {
        fn default() -> Self {
            Self {
                state: FormState::default(),
                data: Data::default(),
                last: None,
                area: SCREEN,
                plain_secret_control: false,
                inactive_secret: false,
            }
        }
    }

    impl App for FieldsApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let (fields, group) = fields_for_app(self.plain_secret_control, self.inactive_secret);
            let response =
                Form::new(FORM, &fields)
                    .group(group)
                    .update(cx, &mut self.state, &mut self.data);
            self.last = response.action_ref().copied();
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let (fields, group) = fields_for_app(self.plain_secret_control, self.inactive_secret);
            Form::new(FORM, &fields)
                .group(group)
                .draw(ui, self.area, &self.state, &self.data);
        }
    }

    #[test]
    fn tab_order_follows_declaration_order_skipping_hidden() {
        let mut state = FormState::default();
        let (runtime, _) = draw(&Data::default(), &mut state);
        let ids: Vec<Id> = runtime.ring().reachable().map(|entry| entry.id).collect();
        assert_eq!(ids, [NAME, FLAG, CHOICE, SECRET, CHOOSER]);
    }

    #[test]
    fn first_draw_registers_fields_before_update_reconciles_state() {
        let state = FormState::default();
        let data = Data::default();
        let fields = fields();
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime
            .draw_scene(SCREEN, &mut buffer, |ui, area| {
                Form::new(FORM, &fields).draw(ui, area, &state, &data);
            })
            .commit_presented();
        assert!(runtime.ring().is_registered(NAME));
        assert!(runtime.ring().is_registered(FLAG));
    }

    #[test]
    fn hidden_field_registers_no_ring_entry_and_keeps_its_draft() {
        let mut state = FormState::default();
        let data = Data::default();
        state.reconcile_fields(&fields());
        let hidden = state
            .slots
            .iter_mut()
            .find(|slot| slot.id == HIDDEN)
            .expect("hidden field has a stable state slot");
        hidden.input.begin("unsaved draft");
        let _ = draw(&data, &mut state);
        assert_eq!(state.slots.len(), fields().len());
        let (runtime, _) = draw(&data, &mut state);
        assert!(!runtime.ring().is_registered(HIDDEN));
        let hidden = state
            .slots
            .iter()
            .find(|slot| slot.id == HIDDEN)
            .expect("hidden field kept its stable state slot");
        assert!(
            hidden.input.is_editing(),
            "hiding did not discard its draft"
        );
    }

    #[test]
    fn declaration_reorder_preserves_field_state_by_id() {
        let original = [
            FieldSpec::new(NAME, "Name", FieldKind::Text(TextInput::new(NAME))),
            FieldSpec::new(HIDDEN, "Hidden", FieldKind::Text(TextInput::new(HIDDEN))),
        ];
        let reordered = [
            FieldSpec::new(HIDDEN, "Hidden", FieldKind::Text(TextInput::new(HIDDEN))),
            FieldSpec::new(NAME, "Name", FieldKind::Text(TextInput::new(NAME))),
        ];
        let mut state = FormState::default();
        state.reconcile_fields(&original);
        state.slots[0].input.begin("unsaved draft");

        state.reconcile_fields(&reordered);

        assert_eq!(
            state.slots.iter().map(|slot| slot.id).collect::<Vec<_>>(),
            [HIDDEN, NAME]
        );
        assert!(!state.slots[0].input.is_editing());
        assert!(state.slots[1].input.is_editing());
    }

    #[test]
    fn field_height_is_a_pure_function_of_spec_and_design_tokens() {
        let field = FieldSpec::new(NAME, "Name", FieldKind::Text(TextInput::new(NAME)));
        let design = Theme::junie().design;
        assert_eq!(
            Form::field_height(&field, &design),
            design.size.field_height
        );
    }

    fn row_text(buffer: &Buffer, y: u16) -> String {
        (0..buffer.area.width)
            .map(|x| buffer.cell((x, y)).expect("row cell").symbol())
            .collect()
    }

    #[derive(Default)]
    struct RadioClipData {
        before_a: String,
        before_b: String,
        before_c: String,
        selected: usize,
        after: String,
    }

    impl FormData for RadioClipData {
        fn value(&self, id: Id) -> FieldRef<'_> {
            match id {
                CLIP_A => FieldRef::Text(&self.before_a),
                CLIP_B => FieldRef::Text(&self.before_b),
                CLIP_C => FieldRef::Text(&self.before_c),
                CLIP_RADIO => FieldRef::Choice(self.selected),
                CLIP_AFTER => FieldRef::Text(&self.after),
                _ => FieldRef::Text(""),
            }
        }

        fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
            match id {
                CLIP_A => FieldMut::Text(&mut self.before_a),
                CLIP_B => FieldMut::Text(&mut self.before_b),
                CLIP_C => FieldMut::Text(&mut self.before_c),
                CLIP_RADIO => FieldMut::Choice(&mut self.selected),
                CLIP_AFTER => FieldMut::Text(&mut self.after),
                _ => FieldMut::ReadOnly,
            }
        }

        fn options(&self, id: Id) -> &[&str] {
            if id == CLIP_RADIO { CLIP_OPTIONS } else { &[] }
        }

        fn validate(&self, _id: Id, _value: FieldRef<'_>) -> Result<(), FieldError> {
            Ok(())
        }
    }

    struct RadioClipApp<'form, 'spec, 'state> {
        form: &'form Form<'spec>,
        state: &'state mut FormState,
        data: &'state mut RadioClipData,
    }

    impl App for RadioClipApp<'_, '_, '_> {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.form.update(cx, self.state, self.data).erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            self.form.draw(ui, ui.full(), self.state, self.data);
        }
    }

    #[test]
    fn partially_scrolled_radio_draws_source_window_without_repeating_chrome() {
        let fields = [
            FieldSpec::new(CLIP_A, "Before A", FieldKind::Text(TextInput::new(CLIP_A))),
            FieldSpec::new(CLIP_B, "Before B", FieldKind::Text(TextInput::new(CLIP_B))),
            FieldSpec::new(CLIP_C, "Before C", FieldKind::Text(TextInput::new(CLIP_C))),
            FieldSpec::new(
                CLIP_RADIO,
                "Operating mode",
                FieldKind::Radio(LabelRadio::new(CLIP_RADIO)),
            )
            .help("Pick a stable mode."),
            FieldSpec::new(
                CLIP_AFTER,
                "Following",
                FieldKind::Text(TextInput::new(CLIP_AFTER)),
            ),
        ];
        let rows = [
            FormRow::Field(CLIP_A),
            FormRow::Field(CLIP_B),
            FormRow::Field(CLIP_C),
            FormRow::Field(CLIP_RADIO),
            FormRow::Field(CLIP_AFTER),
            FormRow::spacer(30),
        ];
        let lanes = [FormLane::new(CLIP_LANE, &rows)];
        let layout = FormLayout::new(&lanes);
        let form = Form::new(FORM, &fields)
            .try_layout(&layout)
            .expect("valid radio clipping layout");
        let mut state = FormState::default();
        state.lane_scroll_mut(CLIP_LANE).set_content(54);
        state.lane_scroll_mut(CLIP_LANE).set_viewport(12);
        state.lane_scroll_mut(CLIP_LANE).scroll_to(18);
        let data = RadioClipData::default();
        let area = Rect::new(0, 0, 50, 12);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                form.draw(ui, area, &state, &data);
            })
            .commit_presented();

        assert!(!row_text(&buffer, 0).contains("Operating mode"));
        assert!(row_text(&buffer, 0).contains("iota"));
        assert!(row_text(&buffer, 1).contains("kappa"));
        assert!(row_text(&buffer, 2).contains("Pick a stable mode."));
        assert!(row_text(&buffer, 3).contains("Following"));
        assert_eq!(
            runtime
                .area_of_part(CLIP_RADIO, PartRef::item(Part::ROW, ItemKey::index(8)))
                .map(|rect| rect.y),
            Some(0)
        );
        assert_eq!(
            runtime
                .area_of_part(CLIP_RADIO, PartRef::item(Part::ROW, ItemKey::index(9)))
                .map(|rect| rect.y),
            Some(1)
        );
        assert!(
            runtime
                .area_of_part(CLIP_RADIO, PartRef::item(Part::ROW, ItemKey::index(0)))
                .is_none()
        );
    }

    #[test]
    fn focused_radio_scroll_reveals_the_live_cursor_row_from_update() {
        let fields = [
            FieldSpec::new(CLIP_A, "Before A", FieldKind::Text(TextInput::new(CLIP_A))),
            FieldSpec::new(CLIP_B, "Before B", FieldKind::Text(TextInput::new(CLIP_B))),
            FieldSpec::new(CLIP_C, "Before C", FieldKind::Text(TextInput::new(CLIP_C))),
            FieldSpec::new(
                CLIP_RADIO,
                "Operating mode",
                FieldKind::Radio(LabelRadio::new(CLIP_RADIO)),
            )
            .help("Pick a stable mode."),
            FieldSpec::new(
                CLIP_AFTER,
                "Following",
                FieldKind::Text(TextInput::new(CLIP_AFTER)),
            ),
        ];
        let rows = [
            FormRow::Field(CLIP_A),
            FormRow::Field(CLIP_B),
            FormRow::Field(CLIP_C),
            FormRow::Field(CLIP_RADIO),
            FormRow::Field(CLIP_AFTER),
            FormRow::spacer(30),
        ];
        let lanes = [FormLane::new(CLIP_LANE, &rows)];
        let layout = FormLayout::new(&lanes);
        let form = Form::new(FORM, &fields)
            .try_layout(&layout)
            .expect("valid radio clipping layout");
        let mut state = FormState::default();
        let mut data = RadioClipData::default();
        state.reconcile_fields_with_data(&fields, &data);
        state.lane_scroll_mut(CLIP_LANE).set_content(54);
        state.lane_scroll_mut(CLIP_LANE).set_viewport(12);

        let app = RadioClipApp {
            form: &form,
            state: &mut state,
            data: &mut data,
        };
        let area = Rect::new(0, 0, 50, 12);
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let Some(slot) = runtime
            .app_mut()
            .state
            .slots
            .iter_mut()
            .find(|slot| slot.id == CLIP_RADIO)
        else {
            panic!("radio field slot was reconciled");
        };
        slot.radio.set_cursor(9, ItemKey::index(9));
        runtime.set_focus(Some(CLIP_RADIO));
        let _ = runtime.settle();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        assert!(
            runtime
                .app()
                .state
                .slots
                .iter()
                .find(|slot| slot.id == CLIP_RADIO)
                .is_some_and(|slot| slot.radio.cursor_index() == 9)
        );
        assert!(
            runtime.app().state.lane_scroll(CLIP_LANE).offset() >= 8,
            "update reveals the focused original option row"
        );
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert_eq!(
            runtime
                .area_of_part(CLIP_RADIO, PartRef::item(Part::ROW, ItemKey::index(9)))
                .map(|rect| rect.y),
            Some(11)
        );
    }

    #[test]
    fn radio_field_paints_one_row_per_option() {
        let (_runtime, buffer) = matrix_runtime(MatrixKind::Radio);
        let rows: Vec<u16> = ["one", "two", "three"]
            .into_iter()
            .map(|label| {
                (0..buffer.area.height)
                    .find(|y| row_text(&buffer, *y).contains(label))
                    .expect(label)
            })
            .collect();
        assert!(rows[0] < rows[1] && rows[1] < rows[2]);
        assert!(
            row_text(&buffer, rows[0]).contains('●'),
            "the value row wears the on glyph"
        );
        assert!(row_text(&buffer, rows[1]).contains('○'));
        assert!(row_text(&buffer, rows[2]).contains('○'));
    }

    #[test]
    fn scroll_reveals_the_focused_field_from_update_not_draw() {
        let tiny = Rect::new(0, 0, SCREEN.width, 4);
        let app = FieldsApp {
            area: tiny,
            ..FieldsApp::default()
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(tiny);
        runtime.draw_buffer(tiny, &mut buffer).commit_presented();
        runtime.set_focus(Some(CHOOSER));
        assert_eq!(runtime.app().state.scroll.offset(), 0);
        drop(runtime.draw_buffer(tiny, &mut buffer));
        assert_eq!(
            runtime.app().state.scroll.offset(),
            0,
            "paint cannot reveal focus"
        );
        let _ = runtime.settle();
        let settled_offset = runtime.app().state.scroll.offset();
        runtime.draw_buffer(tiny, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.scroll.offset(), settled_offset);
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(tiny, &mut buffer).commit_presented();
        assert!(runtime.app().state.scroll.offset() > 0);
    }

    #[test]
    fn submit_commits_the_in_flight_edit_before_validating() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Text);
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Char('x')));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().data.text, "", "keystroke remains a draft");
        let action_id = FORM.part(Part::ACTIONS).index(0);
        runtime.set_focus(Some(action_id));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('s'),
                mods: KeyModifiers::CONTROL,
            }),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().data.validations.get(), 1);
        assert_eq!(
            runtime.app().last,
            Some(FormAction::Action(ActionKey::SAVE))
        );
    }

    #[test]
    fn submit_validates_every_visible_field_then_focuses_the_first_error() {
        let mut app = MatrixApp::new(MatrixKind::Check);
        app.data.config.validation = MatrixValidation::Reject;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().data.validations.get(), 1);
        assert_eq!(runtime.app().last, Some(FormAction::Invalid(MATRIX)));
        assert!(runtime.app().state.error(MATRIX).is_some());
        assert_eq!(runtime.focus(), Some(MATRIX));
    }

    #[test]
    fn submit_skips_hidden_fields_during_validation() {
        let mut app = MatrixApp::new(MatrixKind::Check);
        app.data.config.visible = false;
        app.data.config.validation = MatrixValidation::Reject;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let action_id = FORM.part(Part::ACTIONS).index(0);
        assert_eq!(runtime.focus(), Some(action_id));
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('s'),
                mods: KeyModifiers::CONTROL,
            }),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().data.validations.get(), 0);
        assert_eq!(
            runtime.app().last,
            Some(FormAction::Action(ActionKey::SAVE))
        );
    }

    #[test]
    fn enter_submits_only_when_the_focused_control_is_not_editing() {
        for kind in [
            MatrixKind::Select,
            MatrixKind::Radio,
            MatrixKind::Chips,
            MatrixKind::Check,
            MatrixKind::Toggle,
            MatrixKind::Chooser,
        ] {
            let (mut runtime, mut buffer) = matrix_runtime(kind);
            let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Enter));
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
            let expected = if kind == MatrixKind::Chooser {
                FormAction::Chose(MATRIX)
            } else {
                FormAction::Action(ActionKey::SAVE)
            };
            assert_eq!(
                runtime.app().last,
                Some(expected),
                "{kind:?} did not deliver Enter to the focused control first"
            );
            assert_eq!(runtime.app().data.choice, 0, "{kind:?} changed choice");
            assert!(!runtime.app().data.flag, "{kind:?} changed flag");
            assert!(
                runtime.app().data.chips.is_empty(),
                "{kind:?} changed chips"
            );
            assert!(
                runtime
                    .app()
                    .state
                    .slots
                    .first()
                    .is_some_and(|slot| !slot.select.is_open()),
                "{kind:?} opened child popup"
            );
        }

        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Chooser);
        runtime.set_focus(Some(MATRIX));
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Char(' ')));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(
            runtime.app().last,
            Some(FormAction::Chose(MATRIX)),
            "Space should activate a focused chooser"
        );

        for kind in [MatrixKind::Text, MatrixKind::Area] {
            let (mut runtime, mut buffer) = matrix_runtime(kind);
            let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
            let editing = runtime
                .app()
                .state
                .slots
                .first()
                .is_some_and(|slot| match kind {
                    MatrixKind::Text => slot.input.is_editing(),
                    MatrixKind::Area => slot.area.is_editing(),
                    _ => false,
                });
            assert!(editing, "{kind:?} did not enter editing after focus");
            let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Enter));
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
            let expected = match kind {
                MatrixKind::Text => FormAction::Committed(MATRIX),
                MatrixKind::Area => FormAction::Changed(MATRIX),
                _ => unreachable!(),
            };
            assert_eq!(
                runtime.app().last,
                Some(expected),
                "{kind:?} did not keep Enter in the editing child"
            );
        }
    }

    #[test]
    fn submit_chord_is_declared_on_the_action_not_baked_in() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Check);
        let action_id = FORM.part(Part::ACTIONS).index(0);
        runtime.set_focus(Some(action_id));
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('s'),
                mods: KeyModifiers::CONTROL,
            }),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(
            runtime.app().last,
            Some(FormAction::Action(ActionKey::SAVE))
        );
    }

    #[test]
    fn dirty_is_set_by_a_commit_not_by_a_keystroke() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Text);
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(!runtime.app().state.is_dirty());
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Char('x')));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(
            !runtime.app().state.is_dirty(),
            "typing changed only the child draft"
        );
        runtime.set_focus(Some(FORM.part(Part::ACTIONS).index(0)));
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(
            runtime.app().state.is_dirty(),
            "focus-out commit marked the form dirty"
        );
    }

    #[test]
    fn chooser_activation_emits_chose_with_the_field_id() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Chooser);
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Char(' ')));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().last, Some(FormAction::Chose(MATRIX)));
    }

    #[test]
    fn intrinsic_note_rows_use_live_count_and_role_tones() {
        let (mut runtime, mut buffer) =
            matrix_runtime_with_sizing(MatrixKind::Note, FieldSizingPolicy::Intrinsic);
        runtime.app_mut().data.notes = vec![
            ("Info line", Role::Info),
            ("Warning line", Role::Warning),
            ("Success line", Role::Success),
            ("Fourth line", Role::Fg(crate::theme::FgStep::Primary)),
            ("Fifth line", Role::Danger),
        ];
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(row_text(&buffer, 0).contains("Info line"));
        assert!(row_text(&buffer, 1).contains("Warning line"));
        assert!(row_text(&buffer, 4).contains("Fifth line"));
        let info = buffer.cell((2, 0)).expect("info cell").fg;
        let warning = buffer.cell((2, 1)).expect("warning cell").fg;
        assert_ne!(info, warning, "each row uses its supplied semantic role");

        let after = FieldSpec::new(
            CLIP_AFTER,
            "After",
            FieldKind::Text(TextInput::new(CLIP_AFTER)),
        );
        let fields = [matrix_field(MatrixKind::Note), after];
        let form = Form::new(FORM, &fields).field_sizing(FieldSizingPolicy::Intrinsic);
        for count in [0, 1, 5] {
            let rows = vec![("note", Role::Info); count];
            let mut data = MatrixData::new(MatrixKind::Note);
            data.notes = rows;
            let mut placements = form.placements(&Theme::junie().design, SCREEN.width);
            let note = placements.next(&data).expect("note placement");
            let following = placements.next(&data).expect("following placement");
            assert_eq!(note.height, count as u16);
            assert_eq!(following.top, count);
            assert_eq!(
                Form::content_rows(form.placements(&Theme::junie().design, 60), &data),
                count + 3
            );
            let measured = Cell::new(Size::default());
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            runtime
                .draw_scene(SCREEN, &mut buffer, |ui, _| {
                    measured.set(form.measure_with_data(
                        ui,
                        Constraints::loose(SCREEN.width, SCREEN.height),
                        &data,
                    ));
                })
                .commit_presented();
            assert_eq!(measured.get().preferred.1, count as u16 + 3);
        }
    }

    #[test]
    fn partially_scrolled_intrinsic_note_starts_at_source_row() {
        let (mut runtime, _) =
            matrix_runtime_with_sizing(MatrixKind::Note, FieldSizingPolicy::Intrinsic);
        let area = Rect::new(0, 0, SCREEN.width, 3);
        runtime.app_mut().area = area;
        runtime.app_mut().data.notes = vec![
            ("note zero", Role::Info),
            ("note one", Role::Info),
            ("note two", Role::Info),
            ("note three", Role::Info),
            ("note four", Role::Info),
        ];
        runtime.app_mut().state.scroll.set_content(5);
        runtime.app_mut().state.scroll.set_viewport(2);
        runtime.app_mut().state.scroll.scroll_to(2);
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        assert!(row_text(&buffer, 0).contains("note two"));
        assert!(row_text(&buffer, 1).contains("note three"));
        assert!(!row_text(&buffer, 0).contains("note zero"));
    }

    #[test]
    fn intrinsic_chooser_paints_value_before_trailing_button_and_hits_only_button() {
        let (mut runtime, mut buffer) =
            matrix_runtime_with_sizing(MatrixKind::Chooser, FieldSizingPolicy::Intrinsic);
        runtime.app_mut().data.detail = Some("Unicode detail 界");
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(row_text(&buffer, 1).contains("chosen"));
        assert!(row_text(&buffer, 2).contains("Unicode detail"));
        assert_eq!(runtime.app().last, None);

        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(crate::event::MouseKind::Down, 4, 1),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(crate::event::MouseKind::Up, 4, 1),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().last, None, "value has no chooser hit region");

        let button_x = row_text(&buffer, 1)
            .find("Choose")
            .expect("trailing Choose label") as u16;
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(crate::event::MouseKind::Down, button_x, 1),
        );
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            crate::runtime::stub::mouse(crate::event::MouseKind::Up, button_x, 1),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().last, Some(FormAction::Chose(MATRIX)));
    }

    #[test]
    fn intrinsic_chooser_clips_unicode_safely_in_narrow_and_wide_regions() {
        let (mut runtime, _) =
            matrix_runtime_with_sizing(MatrixKind::Chooser, FieldSizingPolicy::Intrinsic);
        runtime.app_mut().data.display = "界e\u{301} 長い値";
        runtime.app_mut().data.detail = Some("A detail line");
        for width in [0, 1, 2, 8, 16, SCREEN.width] {
            let area = Rect::new(0, 0, width, 4);
            runtime.app_mut().area = area;
            let mut buffer = Buffer::empty(area);
            runtime.draw_buffer(area, &mut buffer).commit_presented();
            if width >= 32 {
                let value_row = row_text(&buffer, 1).replace(' ', "");
                assert!(value_row.contains("界e"), "value row: {value_row:?}");
                assert!(row_text(&buffer, 2).contains("A detail"));
            }
        }
    }

    #[test]
    fn space_stays_child_owned_and_unrelated_intents_stay_ignored() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Check);
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::F(12)));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().last, None);
        assert!(!runtime.app().data.flag);
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Char(' ')));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().last, Some(FormAction::Committed(MATRIX)));
        assert!(runtime.app().data.flag);
    }

    #[test]
    fn note_rows_register_only_decorative_regions() {
        let mut state = FormState::default();
        let (runtime, _) = draw(&Data::default(), &mut state);
        assert!(!runtime.ring().is_registered(NOTE));
    }

    #[test]
    fn at_most_one_action_per_frame_in_declaration_order() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Text);
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Char('x')));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let action_id = FORM.part(Part::ACTIONS).index(0);
        runtime.set_focus(Some(action_id));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(
            &mut runtime,
            Input::Key(Key {
                code: KeyCode::Char('s'),
                mods: KeyModifiers::CONTROL,
            }),
        );
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().data.text, "x", "child focus-out committed");
        assert_eq!(
            runtime.app().data.validations.get(),
            1,
            "submit ran exactly once"
        );
        assert_eq!(
            runtime.app().last,
            Some(FormAction::Action(ActionKey::SAVE)),
            "the submit response wins the single output slot"
        );
    }

    #[test]
    fn open_select_popover_dismisses_on_focus_out_and_esc_closes_only_the_popover() {
        let mut app = MatrixApp::new(MatrixKind::Select);
        app.enter = EnterPolicy::Never;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(runtime.app().state.slots[0].select.is_open());
        let action_id = FORM.part(Part::ACTIONS).index(0);
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Tab));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.focus(), Some(action_id));
        assert!(!runtime.app().state.slots[0].select.is_open());

        runtime.set_focus(Some(MATRIX));
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(runtime.app().state.slots[0].select.is_open());
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Esc));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(!runtime.app().state.slots[0].select.is_open());
        assert_eq!(runtime.focus(), Some(MATRIX));
    }

    #[test]
    fn effective_enter_binding_can_be_removed_or_remapped() {
        let mut removed = MatrixApp::new(MatrixKind::Check);
        removed
            .keymap
            .remove_component(MATRIX, ActionKey::custom("Toggle (Enter)"));
        let mut runtime = Runtime::new(removed, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().last, None);
        assert!(!runtime.app().data.flag);

        let mut remapped = MatrixApp::new(MatrixKind::Check);
        remapped.keymap.remap_component(
            MATRIX,
            ActionKey::custom("Toggle (Enter)"),
            Chord::key(KeyCode::F(2)),
        );
        let mut runtime = Runtime::new(remapped, Theme::junie());
        let _ = runtime.initialize();
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().last, None, "removed Enter did not submit");
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::F(2)));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert!(runtime.app().data.flag, "remapped chord reached the child");
    }

    #[test]
    fn form_action_variants_carry_no_value() {
        fn identity(action: FormAction) -> Id {
            match action {
                FormAction::Changed(id)
                | FormAction::Committed(id)
                | FormAction::Chose(id)
                | FormAction::Invalid(id) => id,
                FormAction::Action(_) => FORM,
            }
        }
        assert_eq!(identity(FormAction::Action(ActionKey::SAVE)), FORM);
    }

    #[test]
    fn zeroize_overwrites_every_secret_draft() {
        let mut state = FormState::default();
        state.reconcile_fields(&fields());
        if let Some(slot) = state.slots.iter_mut().find(|slot| slot.id == SECRET) {
            slot.input.begin("swordfish");
        }
        state.zeroize();
        assert!(!format!("{state:?}").contains("swordfish"));
    }

    #[test]
    fn secret_field_frame_masks_even_without_control_policy() {
        let mut data = Data::default();
        data.secret.set("swordfish");
        let mut state = FormState::default();
        let buffer = draw_secret_field(&data, &mut state, FieldKind::Text(TextInput::new(SECRET)));
        let frame: String = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(
            !frame.contains("swordfish"),
            "secret field reached the frame"
        );
        let mask = Theme::junie()
            .design
            .glyphs
            .get(SecretPolicy::default().mask);
        assert!(
            frame.matches(mask).count()
                >= "swordfish"
                    .chars()
                    .count()
                    .saturating_sub(SecretPolicy::default().synthetic_tail),
            "secret field did not paint its mask: {frame}"
        );
    }

    #[test]
    fn secret_area_frame_is_masked_instead_of_painting_plaintext() {
        let mut data = Data::default();
        data.secret.set("swordfish");
        let mut state = FormState::default();
        let buffer =
            draw_secret_field(&data, &mut state, FieldKind::Area(TextArea::new(SECRET, 3)));
        let frame: String = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(
            !frame.contains("swordfish"),
            "secret area reached the frame"
        );
        let mask = Theme::junie()
            .design
            .glyphs
            .get(SecretPolicy::default().mask);
        assert!(
            frame.matches(mask).count() >= "swordfish".chars().count(),
            "secret area did not paint its mask: {frame}"
        );
    }

    #[test]
    fn secret_field_errors_are_generic_in_the_frame_and_cleared_on_zeroize() {
        let mut data = Data::default();
        data.secret.set("swordfish");
        let mut state = initialized_secret_state();
        state.set_error(SECRET, Some(FieldError::new("swordfish")));
        let (_, buffer) = draw(&data, &mut state);
        let frame: String = buffer
            .content()
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(
            !frame.contains("swordfish"),
            "secret error reached the frame"
        );
        assert_eq!(
            state.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        state.zeroize();
        assert!(state.error(SECRET).is_none());
    }

    #[test]
    fn unreconciled_field_errors_stay_generic_until_sensitivity_is_known() {
        const DETAIL: &str = "secret validation detail";

        let mut plain = FormState::default();
        plain.set_error(SECRET, Some(FieldError::new(DETAIL)));
        assert_eq!(
            plain.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{plain:?}").contains(DETAIL));
        let plain_copy = plain.clone();
        assert_eq!(
            plain_copy.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );

        plain.reconcile_fields(&fields_with_secret_policy(false));
        assert_eq!(
            plain.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        let plain_data = Data {
            secret_mode: false,
            ..Default::default()
        };
        let plain_fields = fields_with_secret_policy(false);
        assert_eq!(
            Form::field_error(&plain, &plain_data, &plain_fields[4]),
            Some(DETAIL)
        );

        let mut secret = FormState::default();
        secret.set_error(SECRET, Some(FieldError::new(DETAIL)));
        secret.reconcile_fields(&fields());
        assert_eq!(
            secret.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{secret:?}").contains(DETAIL));
    }

    #[test]
    fn dynamic_secret_transition_keeps_form_error_generic_until_update() {
        const DETAIL: &str = "dynamic secret validation detail";

        let mut app = FieldsApp {
            plain_secret_control: true,
            ..Default::default()
        };
        app.data.secret_mode = false;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        runtime
            .app_mut()
            .state
            .reconcile_fields(&fields_with_secret_policy(false));
        runtime
            .app_mut()
            .state
            .set_error(SECRET, Some(FieldError::new(DETAIL)));
        runtime.app_mut().data.secret_mode = true;

        assert!(matches!(
            runtime.app().data.value(SECRET),
            FieldRef::Secret(_)
        ));
        {
            let value = runtime.app_mut().data.value_mut(SECRET);
            assert!(matches!(value, FieldMut::Secret(_)));
        }

        let state = &runtime.app().state;
        assert_eq!(
            state.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{state:?}").contains(DETAIL));

        let copy = state.clone();
        assert_eq!(
            copy.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{copy:?}").contains(DETAIL));

        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        assert_eq!(
            runtime
                .app()
                .state
                .error(SECRET)
                .map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
    }

    fn assert_public_error_is_redacted(state: &FormState, id: Id, detail: &str) {
        assert_eq!(
            state.error(id).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{state:?}").contains(detail));

        let copy = state.clone();
        assert_eq!(
            copy.error(id).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{copy:?}").contains(detail));
    }

    #[test]
    fn hidden_dynamic_secret_transition_redacts_error_and_draft() {
        const DETAIL: &str = "hidden dynamic secret detail";

        let mut app = FieldsApp::default();
        app.data.flags.show_hidden = true;
        app.data.hidden_secret_mode = false;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let fields = fields_with_secret_policy(true);
        let app = runtime.app_mut();
        let data = &app.data;
        app.state.reconcile_fields_with_data(&fields, data);
        let slot = runtime
            .app_mut()
            .state
            .slots
            .iter_mut()
            .find(|slot| slot.id == HIDDEN)
            .expect("hidden slot");
        slot.input.begin("plain draft");
        runtime.app_mut().state.set_error_with_sensitivity(
            HIDDEN,
            Some(FieldError::new(DETAIL)),
            Some(false),
        );

        runtime.app_mut().data.hidden_secret_mode = true;
        runtime.app_mut().data.flags.show_hidden = false;
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);

        assert_public_error_is_redacted(&runtime.app().state, HIDDEN, DETAIL);
        let slot = runtime
            .app()
            .state
            .slots
            .iter()
            .find(|slot| slot.id == HIDDEN)
            .expect("hidden slot retained");
        assert!(slot.input.is_sensitive());
        assert!(!slot.input.is_editing());
    }

    #[test]
    fn inactive_dynamic_secret_transition_redacts_error_and_draft() {
        const DETAIL: &str = "inactive dynamic secret detail";

        let mut app = FieldsApp {
            inactive_secret: true,
            ..Default::default()
        };
        app.data.flags.show_hidden = true;
        app.data.hidden_secret_mode = false;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();

        let (fields, group) = fields_for_app(false, true);
        let app = runtime.app_mut();
        let data = &app.data;
        app.state.reconcile_fields_with_data(&fields, data);
        let slot = runtime
            .app_mut()
            .state
            .slots
            .iter_mut()
            .find(|slot| slot.id == HIDDEN)
            .expect("inactive slot");
        slot.input.begin("inactive draft");
        runtime.app_mut().state.set_error_with_sensitivity(
            HIDDEN,
            Some(FieldError::new(DETAIL)),
            Some(false),
        );

        assert!(
            !Form::new(FORM, &fields)
                .group(group)
                .shown(&runtime.app().data, &fields[1])
        );
        runtime.app_mut().data.hidden_secret_mode = true;
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);

        assert_public_error_is_redacted(&runtime.app().state, HIDDEN, DETAIL);
        let slot = runtime
            .app()
            .state
            .slots
            .iter()
            .find(|slot| slot.id == HIDDEN)
            .expect("inactive slot retained");
        assert!(slot.input.is_sensitive());
        assert!(!slot.input.is_editing());
    }

    #[test]
    fn direct_owner_flip_redacts_rendered_text_before_update() {
        const DETAIL: &str = "direct owner flip detail";

        for area in [false, true] {
            let kind = if area {
                FieldKind::Area(TextArea::new(SECRET, 3))
            } else {
                FieldKind::Text(TextInput::new(SECRET))
            };
            let fields = [FieldSpec::new(SECRET, "Secret", kind)];
            let mut data = Data {
                secret_mode: false,
                ..Default::default()
            };
            let mut state = FormState::default();
            state.reconcile_fields_with_data(&fields, &data);
            state.set_error_with_sensitivity(SECRET, Some(FieldError::new(DETAIL)), Some(false));
            assert_eq!(Form::field_error(&state, &data, &fields[0]), Some(DETAIL));

            data.secret_mode = true;
            assert_eq!(
                Form::field_error(&state, &data, &fields[0]),
                Some("Invalid value")
            );
            assert_public_error_is_redacted(&state, SECRET, DETAIL);
        }
    }

    #[test]
    fn malformed_dynamic_secret_is_redacted_for_non_text_slots() {
        const DETAIL: &str = "non-text dynamic secret detail";

        let fields = [FieldSpec::new(
            SECRET,
            "Secret",
            FieldKind::Check(Checkbox::new(SECRET, "Secret")),
        )];
        let mut data = Data {
            secret_mode: false,
            ..Default::default()
        };
        let mut state = FormState::default();
        state.reconcile_fields_with_data(&fields, &data);
        state.set_error_with_sensitivity(SECRET, Some(FieldError::new(DETAIL)), Some(false));
        assert_eq!(Form::field_error(&state, &data, &fields[0]), Some(DETAIL));

        data.secret_mode = true;
        assert_public_error_is_redacted(&state, SECRET, DETAIL);
    }

    #[test]
    fn malformed_dynamic_secret_error_does_not_reappear_after_downgrade() {
        const DETAIL: &str = "non-text downgrade detail";

        let fields = [FieldSpec::new(
            SECRET,
            "Secret",
            FieldKind::Check(Checkbox::new(SECRET, "Secret")),
        )];
        let mut data = Data {
            secret_mode: true,
            ..Default::default()
        };
        let mut state = FormState::default();
        state.reconcile_fields_with_data(&fields, &data);
        state.set_error(SECRET, Some(FieldError::new(DETAIL)));
        assert_public_error_is_redacted(&state, SECRET, DETAIL);

        data.secret_mode = false;
        state.reconcile_fields_with_data(&fields, &data);
        assert!(state.error(SECRET).is_none());
        assert!(Form::field_error(&state, &data, &fields[0]).is_none());
    }

    #[test]
    fn resolved_plain_dynamic_text_and_area_errors_stay_generic_after_secret_transition() {
        const DETAIL: &str = "resolved dynamic secret validation detail";

        for area in [false, true] {
            let kind = if area {
                FieldKind::Area(TextArea::new(SECRET, 3))
            } else {
                FieldKind::Text(TextInput::new(SECRET))
            };
            let fields = [FieldSpec::new(SECRET, "Secret", kind)];
            let mut data = Data {
                secret_mode: false,
                ..Default::default()
            };
            let mut state = FormState::default();
            state.reconcile_fields(&fields);
            state.set_error_with_sensitivity(SECRET, Some(FieldError::new(DETAIL)), Some(false));
            assert_eq!(Form::field_error(&state, &data, &fields[0]), Some(DETAIL));

            data.secret_mode = true;
            assert!(matches!(data.value(SECRET), FieldRef::Secret(_)));
            assert_eq!(
                Form::field_error(&state, &data, &fields[0]),
                Some("Invalid value")
            );

            assert_eq!(
                state.error(SECRET).map(|error| error.message.as_ref()),
                Some("Invalid value")
            );
            assert!(!format!("{state:?}").contains(DETAIL));
            let copy = state.clone();
            assert_eq!(
                copy.error(SECRET).map(|error| error.message.as_ref()),
                Some("Invalid value")
            );
            assert!(!format!("{copy:?}").contains(DETAIL));

            {
                let value = data.value_mut(SECRET);
                assert!(matches!(&value, FieldMut::Secret(_)));
                let _ = Form::prepare_slot(&mut state, 0, &fields[0], &value);
            }
            assert_eq!(
                state.error(SECRET).map(|error| error.message.as_ref()),
                Some("Invalid value")
            );
        }
    }

    #[test]
    fn secret_validator_error_is_generic_before_form_state_retention() {
        let mut data = Data::default();
        data.secret.set("swordfish");
        let mut state = FormState::default();
        state.reconcile_fields(&fields());
        let error = FieldError::new(format!("invalid {}", data.secret.expose()));
        let safe = Form::safe_error(&data, true, SECRET, error);
        state.set_error(SECRET, Some(safe));
        assert_eq!(
            state.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{state:?}").contains("swordfish"));
    }

    #[test]
    fn reconciling_same_shape_secret_fields_redacts_existing_errors() {
        let plain_fields = fields_with_secret_policy(false);
        let secret_fields = fields_with_secret_policy(true);
        let mut state = FormState::default();
        state.reconcile_fields(&plain_fields);
        state.set_error(SECRET, Some(FieldError::new("swordfish")));
        assert_eq!(
            state.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );

        state.reconcile_fields(&plain_fields);
        let data = Data {
            secret_mode: false,
            ..Default::default()
        };
        assert_eq!(
            Form::field_error(&state, &data, &plain_fields[4]),
            Some("swordfish")
        );
        state.reconcile_fields(&secret_fields);
        assert_eq!(
            state.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{state:?}").contains("swordfish"));

        state.reconcile_fields(&plain_fields);
        assert!(state.error(SECRET).is_none());
    }

    #[test]
    fn sensitive_form_state_clone_and_equality_do_not_copy_draft() {
        let mut left = initialized_secret_state();
        let mut right = initialized_secret_state();
        for (state, value) in [(&mut left, "swordfish"), (&mut right, "different")] {
            let slot = state
                .slots
                .iter_mut()
                .find(|slot| slot.id == SECRET)
                .expect("secret slot");
            slot.input.begin(value);
        }
        left.set_error(NAME, Some(FieldError::coded("name required", "required")));
        right.set_error(NAME, Some(FieldError::coded("name required", "required")));
        left.set_error(SECRET, Some(FieldError::new("swordfish")));
        right.set_error(SECRET, Some(FieldError::new("different")));
        left.reconcile_fields(&fields());
        right.reconcile_fields(&fields());
        assert_eq!(left, right);
        let copy = left.clone();
        let slot = copy
            .slots
            .iter()
            .find(|slot| slot.id == SECRET)
            .expect("cloned secret slot");
        assert!(slot.input.is_sensitive());
        assert_eq!(
            copy.error(NAME).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert_eq!(copy.error(NAME).and_then(|error| error.code), None);
        assert_eq!(
            copy.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{copy:?}").contains("swordfish"));
    }

    #[test]
    fn form_update_classifies_secret_before_retaining_local_error() {
        let mut app = FieldsApp::default();
        app.data.secret.set("swordfish");
        app.state.reconcile_fields(&fields());
        app.state.set_error(
            SECRET,
            Some(FieldError::new(format!(
                "invalid {}",
                app.data.secret.expose()
            ))),
        );
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        assert_eq!(
            runtime
                .app()
                .state
                .error(SECRET)
                .map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{:?}", runtime.app().state).contains("swordfish"));
    }

    #[test]
    fn form_reconciles_secret_and_text_values_in_both_directions() {
        let mut app = FieldsApp::default();
        app.data.secret.set("swordfish");
        app.data.secret_text = "ordinary".to_owned();
        app.plain_secret_control = true;
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        {
            let state = &mut runtime.app_mut().state;
            let slot = state
                .slots
                .iter_mut()
                .find(|slot| slot.id == SECRET)
                .expect("secret slot");
            slot.input.begin("swordfish");
            slot.input.set_error(Some(FieldError::new("secret detail")));
            assert!(slot.input.is_sensitive());
        }

        runtime.app_mut().data.secret_mode = false;
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        let slot = runtime
            .app()
            .state
            .slots
            .iter()
            .find(|slot| slot.id == SECRET)
            .expect("text slot");
        assert!(!slot.input.is_sensitive());
        assert!(!slot.input.is_editing());
        assert!(slot.input.error().is_none());

        runtime.app_mut().data.secret_mode = true;
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        let slot = runtime
            .app()
            .state
            .slots
            .iter()
            .find(|slot| slot.id == SECRET)
            .expect("secret slot restored");
        assert!(slot.input.is_sensitive());
        assert!(!slot.input.is_editing());
    }

    #[test]
    fn every_declared_field_resolves_a_value() {
        let data = Data::default();
        for field in fields() {
            let _ = data.value(field.id);
        }
    }

    #[test]
    fn select_field_options_come_from_form_data() {
        assert_eq!(Data::default().options(CHOICE), OPTIONS);
    }

    #[test]
    fn changing_options_between_frames_does_not_rebuild_props() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Select);
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let slot_count = runtime.app().state.slots.len();
        runtime.app_mut().data.config.options = MatrixOptions::Alternate;
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().state.slots.len(), slot_count);
        assert_eq!(runtime.app().data.options(MATRIX), OTHER_OPTIONS);
        assert_eq!(runtime.focus(), Some(MATRIX));
    }

    #[test]
    fn state_holds_no_props() {
        fn assert_state<T: Clone + PartialEq + Default>() {}
        fn assert_slot<T: Clone + PartialEq + Eq>() {}
        assert_state::<FormState>();
        assert_slot::<SlotValue>();
    }

    #[test]
    fn value_and_options_is_a_single_borrow() {
        let mut data = Data::default();
        let (value, options) = data.value_and_options(CHOICE);
        assert!(matches!(value, FieldMut::Choice(_)) && options == OPTIONS);
    }

    #[test]
    fn dynamic_disabled_state_is_inherited_by_configured_control() {
        let mut state = FormState::default();
        let data = Data {
            flags: TestFlags {
                disabled: true,
                ..TestFlags::default()
            },
            ..Data::default()
        };
        let (runtime, _) = draw(&data, &mut state);
        assert!(
            runtime
                .ring()
                .entry(FLAG)
                .is_some_and(|entry| entry.disabled)
        );
    }

    #[test]
    fn reference_form_registers_no_controls_or_parts_for_any_field_kind() {
        for kind in [
            MatrixKind::Text,
            MatrixKind::Area,
            MatrixKind::Select,
            MatrixKind::Radio,
            MatrixKind::Chips,
            MatrixKind::Check,
            MatrixKind::Toggle,
            MatrixKind::Chooser,
        ] {
            let mut app = MatrixApp::new(kind);
            app.reference = true;
            let mut runtime = Runtime::new(app, Theme::junie());
            let mut buffer = Buffer::empty(SCREEN);
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
            let action_id = FORM.part(Part::ACTIONS).index(0);
            assert!(!runtime.registry().has_owner(FORM), "{kind:?} form leaked");
            assert!(
                !runtime.registry().has_owner(MATRIX),
                "{kind:?} child leaked"
            );
            assert!(
                !runtime.registry().has_owner(action_id),
                "{kind:?} action leaked"
            );
            assert!(!runtime.ring().is_registered(MATRIX));
            assert!(!runtime.ring().is_registered(action_id));
        }
    }

    #[test]
    fn reference_form_targets_only_one_child() {
        let render = |target| {
            let mut app = MatrixApp::new(MatrixKind::Check);
            app.reference = true;
            app.target = target;
            let mut runtime = Runtime::new(app, Theme::junie().downgrade(crate::ColorLevel::Mono));
            let mut buffer = Buffer::empty(SCREEN);
            runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
            buffer
        };
        let suppressed = render(None);
        let pressed = render(Some(crate::ReferenceTarget::new(
            MATRIX,
            crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
        )));
        assert_ne!(pressed, suppressed);
    }

    #[test]
    fn direct_secret_editing_uses_the_sealed_text_target() {
        let mut app = FieldsApp::default();
        app.data.secret = Secret::new("hunter2".to_owned());
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.set_focus(Some(SECRET));
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        let _ = crate::runtime::stub::deliver(&mut runtime, press(KeyCode::Char('!')));
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        runtime.set_focus(Some(CHOOSER));
        let _ = crate::runtime::stub::deliver(&mut runtime, Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
        assert_eq!(runtime.app().data.secret.expose(), "hunter2!");
    }

    #[test]
    fn textarea_parent_window_intersects_the_logical_form_body() {
        let window = textarea_window(Rect::new(2, 5, 12, 4), 2, 7, 5)
            .expect("visible source rows intersect the logical editor");
        assert_eq!(window.visible_body(), Rect::new(2, 5, 12, 3));
        assert_eq!(window.logical_rows(), 5);
        assert_eq!(window.first_source_row(), 2);

        let short = textarea_window(Rect::new(2, 5, 12, 4), 1, 7, 2)
            .expect("the configured TextArea row count caps logical height");
        assert_eq!(short.visible_body(), Rect::new(2, 5, 12, 1));
        assert_eq!(short.logical_rows(), 2);
        assert_eq!(short.first_source_row(), 1);

        assert!(textarea_window(Rect::new(2, 5, 12, 0), 0, 7, 5).is_none());
        assert!(textarea_window(Rect::new(2, 5, 0, 2), 0, 7, 5).is_none());
        assert!(textarea_window(Rect::new(2, 5, 12, 2), 3, 5, 3).is_none());
    }

    #[test]
    fn nonclone_secret_path_masks_without_materialising_plaintext() {
        let mut state = FormState::default();
        let data = Data {
            secret: Secret::new("unique-secret-value".to_owned()),
            ..Data::default()
        };
        let (_, buffer) = draw(&data, &mut state);
        let painted: String = buffer
            .content
            .iter()
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(!painted.contains("unique-secret-value"));
    }
}
