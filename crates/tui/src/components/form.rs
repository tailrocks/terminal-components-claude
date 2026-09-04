//! Declared-field form composition (`COMPONENT_ARCHITECTURE.md` §15.1, §67).

use core::fmt;

use ratatui_core::layout::Rect;

use super::button::Button;
use super::chip::{ChipBarState, LabelChips};
use super::choice::{Checkbox, LabelRadio, RadioGroupAction, RadioGroupState, Toggle};
use super::input::{TextAction, TextInput, TextInputState};
use super::keyhint::ChordText;
use super::scroll_region::ScrollRegion;
use super::select::{LabelSelect, SelectAction, SelectState};
use super::textarea::{TextArea, TextAreaState};
use super::{Acc, Overrides};
use crate::action::{Action, ActionKey};
use crate::collection::KeySet;
use crate::event::{Chord, KeyCode};
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

/// Width participation in a two-column form.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FieldSpan {
    /// Occupy the whole row.
    Full,
    /// Share a row with the next visible half-width field when possible.
    Half,
}

#[derive(Clone, Copy)]
pub(crate) struct InheritedFormState {
    pub(crate) disabled: bool,
}

/// A section identity. [`GroupKey::ALL`] is visible in every section.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GroupKey(u16);

impl GroupKey {
    /// Fields shared by every section.
    pub const ALL: GroupKey = GroupKey(0);

    /// A stable application-defined section key.
    pub const fn custom(name: &'static str) -> GroupKey {
        GroupKey(1 | ((fnv1a(0xcbf2_9ce4_8422_2325, name.as_bytes()) as u16) & 0x7fff))
    }
}

/// A configured field control. Values and option lists remain in [`FormData`].
pub enum FieldKind<'a> {
    /// Single-line text.
    Text(TextInput<'a>),
    /// Multi-line text.
    Area(TextArea<'a>),
    /// Positional select.
    Select(LabelSelect<'a>),
    /// Positional radio group.
    Radio(LabelRadio<'a>),
    /// Positional chip set.
    Chips(LabelChips<'a>),
    /// Boolean checkbox.
    Check(Checkbox<'a>),
    /// Boolean toggle.
    Toggle(Toggle<'a>),
    /// Owner-driven rich chooser.
    Chooser(Button<'a>),
    /// Static decorative rows.
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

/// One declared field.
pub struct FieldSpec<'a> {
    /// Control identity.
    pub id: Id,
    /// Label painted by [`Field`](crate::components::Field).
    pub label: &'a str,
    /// Configured control.
    pub kind: FieldKind<'a>,
    /// Whether to paint the required marker.
    pub required: bool,
    /// Optional help row.
    pub help: Option<&'a str>,
    /// Full- or half-width layout.
    pub span: FieldSpan,
    /// Section membership.
    pub group: GroupKey,
    /// Suppress optional-label chrome.
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
    /// Declare a field.
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

    /// Set required chrome.
    #[must_use]
    pub const fn required(mut self, yes: bool) -> Self {
        self.required = yes;
        self
    }

    /// Set help text.
    #[must_use]
    pub const fn help(mut self, text: &'a str) -> Self {
        self.help = Some(text);
        self
    }

    /// Set width participation.
    #[must_use]
    pub const fn span(mut self, span: FieldSpan) -> Self {
        self.span = span;
        self
    }

    /// Set section membership.
    #[must_use]
    pub const fn group(mut self, group: GroupKey) -> Self {
        self.group = group;
        self
    }

    /// Set plain-label chrome.
    #[must_use]
    pub const fn plain(mut self, yes: bool) -> Self {
        self.plain = yes;
        self
    }
}

/// Mutable controlled value borrowed for one field update.
pub enum FieldMut<'d> {
    /// Plain text.
    Text(&'d mut String),
    /// Secret text; never cloned by the form.
    Secret(&'d mut Secret),
    /// Positional option index.
    Choice(&'d mut usize),
    /// Boolean value.
    Flag(&'d mut bool),
    /// Positional checked set.
    Chips(&'d mut KeySet),
    /// No mutable value.
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

/// Shared controlled value borrowed for one field draw or validation.
pub enum FieldRef<'d> {
    /// Plain text.
    Text(&'d str),
    /// Secret text; display and debug remain redacted.
    Secret(&'d Secret),
    /// Positional option index.
    Choice(usize),
    /// Boolean value.
    Flag(bool),
    /// Positional checked set.
    Chips(&'d KeySet),
    /// Rich chooser display.
    Display {
        /// Main value line.
        value: &'d str,
        /// Optional detail line.
        detail: Option<&'d str>,
    },
    /// Decorative note rows.
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

/// Owner-supplied values, visibility, options and validation.
pub trait FormData {
    /// Borrow one value for draw or validation.
    fn value(&self, id: Id) -> FieldRef<'_>;
    /// Borrow one value for update.
    fn value_mut(&mut self, id: Id) -> FieldMut<'_>;

    /// Choice labels, empty for non-choice fields.
    fn options(&self, _id: Id) -> &[&str] {
        &[]
    }

    /// Value and choice labels under one mutable borrow.
    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        (self.value_mut(id), &[])
    }

    /// Whether a field participates this frame.
    fn visible(&self, _id: Id) -> bool {
        true
    }

    /// Dynamic disabled state inherited by the configured control.
    fn disabled(&self, _id: Id) -> bool {
        false
    }

    /// External validation error.
    fn error(&self, _id: Id) -> Option<&str> {
        None
    }

    /// Validate one field.
    ///
    /// # Errors
    /// Returns the local validation failure for this value.
    fn validate(&self, _id: Id, _value: FieldRef<'_>) -> Result<(), FieldError> {
        Ok(())
    }

    /// Validate cross-field rules.
    ///
    /// # Errors
    /// Returns the field to focus and its cross-field validation failure.
    fn validate_all(&self) -> Result<(), (Id, FieldError)> {
        Ok(())
    }
}

/// Enter-key arbitration.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnterPolicy {
    /// Submit only when the focused control was idle.
    SubmitsWhenIdle,
    /// Never synthesize submit from Enter.
    Never,
}

/// A form action. Values never leave through this type.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormAction {
    /// An in-flight draft changed.
    Changed(Id),
    /// A value committed.
    Committed(Id),
    /// A chooser fired.
    Chose(Id),
    /// An action-row command fired.
    Action(ActionKey),
    /// Submit failed and focus was staged to this field.
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

    fn is_sensitive(&self) -> bool {
        self.input.is_sensitive() || self.area.is_sensitive()
    }
}

impl fmt::Debug for FieldSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldSlot")
            .field("id", &self.id)
            .field("value", &self.value)
            .field("input", &self.input)
            .field("area", &self.area)
            .field("select", &self.select)
            .field("radio", &self.radio)
            .field("chips", &self.chips)
            .finish()
    }
}

/// Durable form control state. It stores no field declarations or other props.
#[derive(Default)]
pub struct FormState {
    slots: Vec<FieldSlot>,
    scroll: ScrollState,
    errors: Vec<(Id, FieldError)>,
    dirty: bool,
    reveal: Option<Id>,
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
                    if self.slot_is_sensitive(*id) {
                        (*id, FieldError::new("Invalid value"))
                    } else {
                        (*id, error.clone())
                    }
                })
                .collect(),
            dirty: self.dirty,
            reveal: self.reveal,
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
                        && (self.slot_is_sensitive(*id)
                            || other.slot_is_sensitive(*other_id)
                            || error == other_error)
                },
            )
            && self.dirty == other.dirty
            && self.reveal == other.reveal
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
    /// Whether a committed value differs from the owner's clean point.
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the current owner values clean.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Local validation error for `id`.
    pub fn error(&self, id: Id) -> Option<&FieldError> {
        self.errors
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, e)| e)
    }

    /// Set or clear a local validation error.
    pub fn set_error(&mut self, id: Id, error: Option<FieldError>) {
        let error = error.map(|error| {
            if self.slot_is_sensitive(id) {
                discard_error(error);
                FieldError::new("Invalid value")
            } else {
                error
            }
        });
        if let Some(index) = self.errors.iter().position(|(key, _)| *key == id) {
            if let Some(error) = error {
                if let Some((_, current)) = self.errors.get_mut(index) {
                    let old = core::mem::replace(current, error);
                    discard_error(old);
                }
            } else {
                let (_, old) = self.errors.remove(index);
                discard_error(old);
            }
        } else if let Some(error) = error {
            self.errors.push((id, error));
        }
    }

    fn slot_is_sensitive(&self, id: Id) -> bool {
        self.slots
            .iter()
            .find(|slot| slot.id == id)
            .is_some_and(FieldSlot::is_sensitive)
    }

    fn redact_error(&mut self, id: Id) {
        if let Some((_, current)) = self.errors.iter_mut().find(|(key, _)| *key == id) {
            let old = core::mem::replace(current, FieldError::new("Invalid value"));
            discard_error(old);
        }
    }

    /// Clear every local validation error.
    pub fn clear_errors(&mut self) {
        for (_, error) in self.errors.drain(..) {
            discard_error(error);
        }
    }

    /// Request reveal after the next update-side layout.
    pub fn reveal(&mut self, id: Id) {
        self.reveal = Some(id);
    }

    /// Overwrite every in-flight text draft.
    pub fn zeroize(&mut self) {
        for slot in &mut self.slots {
            slot.zeroize();
        }
        self.clear_errors();
        self.reveal = None;
    }

    fn reconcile_fields(&mut self, fields: &[FieldSpec<'_>]) {
        if self.slots.len() == fields.len()
            && self
                .slots
                .iter()
                .zip(fields)
                .all(|(slot, field)| slot.id == field.id)
        {
            return;
        }
        self.clear_errors();
        let mut old = core::mem::take(&mut self.slots);
        self.slots.reserve(fields.len());
        for field in fields {
            if let Some(index) = old.iter().position(|slot| slot.id == field.id) {
                self.slots.push(old.remove(index));
            } else {
                self.slots.push(FieldSlot::new(field.id));
            }
        }
        for slot in &mut old {
            slot.zeroize();
        }
    }
}

fn discard_error(error: FieldError) {
    if let std::borrow::Cow::Owned(mut message) = error.message {
        zeroize_string(&mut message);
    }
}

fn zeroize_string(value: &mut String) {
    let mut bytes = core::mem::take(value).into_bytes();
    bytes.fill(0);
    core::hint::black_box(&bytes);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    bytes.clear();
}

/// Ordered, validated form composition.
///
/// ## Construction
/// `Form::new(id, fields)` receives one declaration-order field slice. Values
/// and option lists remain in the owner's [`FormData`].
///
/// ## Ownership
/// Caller owns every value and [`FormState`]. The form stores only borrowed
/// configuration; state stores per-field control state keyed by [`Id`].
///
/// ## Configuration
/// `.actions`, `.submit`, `.enter`, `.columns`, `.group`, and
/// `.patch_part` configure one instance used in both phases.
///
/// ## Variants
/// Uses `Family::FORM`'s default variant. Child controls retain their configured
/// variants.
///
/// ## States
/// Visible fields expose their child control states. Dynamic disabled state is
/// inherited and combined with each configured control's disabled state.
///
/// ## Actions
/// Reports [`FormAction`]. No field value leaves through an action.
///
/// ## Commands
/// The submit identity comes from `.submit`; action-row commands retain their
/// declared [`ActionKey`], including an ordinary cancel action.
///
/// ## Bindings
/// Child controls publish their own bindings. Action chords are declared on
/// [`Action`]. Enter submission obeys [`EnterPolicy`].
///
/// ## Update
/// Reconciles slots, updates visible controls in declaration order, tracks
/// commits, updates scrolling, and runs submit validation.
///
/// ## Render
/// Registers and paints visible fields in declaration order, omits hidden
/// fields completely, and paints the optional action row.
///
/// ## Measure
/// Field heights derive only from declarations and current design tokens.
/// Two adjacent half-width fields share one row.
///
/// ## Overrides
/// `.patch_part` applies to [`Form::PARTS`]. Child-control overrides remain on
/// each configured [`FieldKind`].
///
/// ## Identity
/// The form and every field use caller-supplied [`Id`] values. No parallel
/// field-key namespace exists.
///
/// ## Testing
/// `FormCase` supplies conformance coverage; module tests cover declaration
/// order, visibility, validation, scrolling, dirty state, and secret handling.
///
/// ## Invariants
/// Draw never mutates state or values. At most one action is returned per
/// update. Hidden fields retain drafts. Debug output redacts text drafts.
pub struct Form<'a> {
    id: Id,
    fields: &'a [FieldSpec<'a>],
    actions: &'a [Action<'a>],
    submit: ActionKey,
    enter: EnterPolicy,
    columns: u8,
    group: GroupKey,
    ov: Overrides<'a>,
    parts: &'a [(Part, StylePatch)],
}

impl fmt::Debug for Form<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Form")
            .field("id", &self.id)
            .field("fields", &self.fields.len())
            .field("actions", &self.actions.len())
            .field("submit", &self.submit)
            .field("enter", &self.enter)
            .field("columns", &self.columns)
            .field("group", &self.group)
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

struct Placements<'fields, 'a> {
    fields: &'fields [FieldSpec<'a>],
    group: GroupKey,
    columns: u8,
    width: u16,
    gap: u16,
    field_height: u16,
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
            let height = match &field.kind {
                FieldKind::Area(area) => {
                    self.field_height.max(area.rows_in_form().saturating_add(2))
                }
                _ => self.field_height,
            };
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

impl<'a> Form<'a> {
    /// The parts this composition styles.
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::BODY,
        Part::ACTIONS,
        Part::HELP,
        Part::MARKER,
        Part::TRACK,
        Part::THUMB,
    ];

    /// Declare a form over one field array.
    pub const fn new(id: Id, fields: &'a [FieldSpec<'a>]) -> Self {
        Form {
            id,
            fields,
            actions: &[],
            submit: ActionKey::SAVE,
            enter: EnterPolicy::SubmitsWhenIdle,
            columns: 1,
            group: GroupKey::ALL,
            ov: Overrides::new(),
            parts: &[],
        }
    }

    /// Set action-row declarations.
    #[must_use]
    pub const fn actions(mut self, actions: &'a [Action<'a>]) -> Self {
        self.actions = actions;
        self
    }

    /// Set the submit identity.
    #[must_use]
    pub const fn submit(mut self, key: ActionKey) -> Self {
        self.submit = key;
        self
    }

    /// Set Enter arbitration.
    #[must_use]
    pub const fn enter(mut self, policy: EnterPolicy) -> Self {
        self.enter = policy;
        self
    }

    /// Set one or two columns; other values clamp to that supported range.
    #[must_use]
    pub const fn columns(mut self, columns: u8) -> Self {
        self.columns = if columns > 1 { 2 } else { 1 };
        self
    }

    /// Select the active field section.
    #[must_use]
    pub const fn group(mut self, group: GroupKey) -> Self {
        self.group = group;
        self
    }

    /// Set per-part patches.
    #[must_use]
    pub const fn patch_part(mut self, patches: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.patch_part(patches);
        self.parts = patches;
        self
    }

    fn shown<D: FormData + ?Sized>(&self, data: &D, field: &FieldSpec<'_>) -> bool {
        (field.group == GroupKey::ALL || field.group == self.group) && data.visible(field.id)
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

    fn set_slot_value(slot: &mut FieldSlot, value: &FieldMut<'_>) {
        slot.value = match value {
            FieldMut::Text(_) => SlotValue::Text,
            FieldMut::Secret(_) => {
                slot.input.mark_sensitive();
                slot.area.mark_sensitive();
                SlotValue::Text
            }
            FieldMut::Choice(value) => SlotValue::Choice(**value),
            FieldMut::Flag(value) => SlotValue::Flag(**value),
            FieldMut::Chips(keys) => SlotValue::Chips((**keys).clone()),
            FieldMut::ReadOnly => SlotValue::None,
        };
    }

    fn prepare_slot<'s>(
        st: &'s mut FormState,
        index: usize,
        id: Id,
        value: &FieldMut<'_>,
    ) -> Option<&'s mut FieldSlot> {
        let secret = matches!(value, FieldMut::Secret(_));
        {
            let slot = st.slots.get_mut(index)?;
            Self::set_slot_value(slot, value);
        }
        if secret {
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
            (FieldKind::Text(control), FieldMut::Secret(value)) => {
                slot.input.mark_sensitive();
                control
                    .update_in_form(cx, &mut slot.input, value, inherited_disabled)
                    .map_action(|action| match action {
                        TextAction::Changed => FormAction::Changed(field.id),
                        TextAction::Committed => FormAction::Committed(field.id),
                        TextAction::Cancelled | TextAction::MoveNext | TextAction::MovePrev => {
                            FormAction::Changed(field.id)
                        }
                    })
            }
            (FieldKind::Area(control), FieldMut::Text(value)) => control
                .update_in_form(cx, &mut slot.area, value, inherited_disabled)
                .map_action(|action| match action {
                    TextAction::Changed => FormAction::Changed(field.id),
                    TextAction::Committed => FormAction::Committed(field.id),
                    TextAction::Cancelled | TextAction::MoveNext | TextAction::MovePrev => {
                        FormAction::Changed(field.id)
                    }
                }),
            (FieldKind::Area(control), FieldMut::Secret(value)) => {
                slot.area.mark_sensitive();
                control
                    .update_in_form(cx, &mut slot.area, value, inherited_disabled)
                    .map_action(|action| match action {
                        TextAction::Changed => FormAction::Changed(field.id),
                        TextAction::Committed => FormAction::Committed(field.id),
                        TextAction::Cancelled | TextAction::MoveNext | TextAction::MovePrev => {
                            FormAction::Changed(field.id)
                        }
                    })
            }
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
                st.set_error(field.id, Some(Self::safe_error(data, field.id, error)));
                st.reveal(field.id);
                cx.focus(field.id);
                return FormAction::Invalid(field.id);
            }
        }
        if let Err((id, error)) = data.validate_all() {
            st.set_error(id, Some(Self::safe_error(data, id, error)));
            st.reveal(id);
            cx.focus(id);
            return FormAction::Invalid(id);
        }
        FormAction::Action(self.submit)
    }

    /// Update controls, scrolling, validation and the action row.
    pub fn update<D: FormData + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        st: &mut FormState,
        data: &mut D,
    ) -> Response<FormAction> {
        st.reconcile_fields(self.fields);
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
            st.scroll.ensure_visible_on_next_layout(placement.top);
        }

        let enter_requested = self.enter == EnterPolicy::SubmitsWhenIdle
            && self.fields.iter().any(|field| {
                self.shown(data, field)
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
                st.scroll.ensure_visible_on_next_layout(placement.top);
            }
            let disabled = data.disabled(field.id);
            let (value, options) = data.value_and_options(field.id);
            let Some(slot) = Self::prepare_slot(st, placement.index, field.id, &value) else {
                continue;
            };
            let response = Self::update_field(field, cx, slot, value, options, disabled);
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

    fn field_error<'d, D: FormData + ?Sized>(
        st: &'d FormState,
        data: &'d D,
        id: Id,
    ) -> Option<&'d str> {
        if matches!(data.value(id), FieldRef::Secret(_)) {
            return (st.error(id).is_some() || data.error(id).is_some()).then_some("Invalid value");
        }
        st.error(id)
            .map(|error| error.message.as_ref())
            .or_else(|| data.error(id))
    }

    fn safe_error<D: FormData + ?Sized>(data: &D, id: Id, error: FieldError) -> FieldError {
        if matches!(data.value(id), FieldRef::Secret(_)) {
            discard_error(error);
            FieldError::new("Invalid value")
        } else {
            error
        }
    }

    /// Draw the visible fields and action row. This method never mutates state.
    pub fn draw<D: FormData + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        st: &FormState,
        data: &D,
    ) -> Rect {
        if area.is_empty() {
            return area;
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
            let bottom = placement.top.saturating_add(usize::from(placement.height));
            if bottom <= base || placement.top >= base.saturating_add(usize::from(content.height)) {
                continue;
            }
            let y_offset = placement.top.saturating_sub(base);
            let field_area = Rect {
                x: content.x.saturating_add(placement.left),
                y: content
                    .y
                    .saturating_add(y_offset.min(usize::from(u16::MAX)) as u16),
                width: placement
                    .width
                    .min(content.width.saturating_sub(placement.left)),
                height: placement.height.min(
                    content
                        .height
                        .saturating_sub(y_offset.min(usize::from(u16::MAX)) as u16),
                ),
            };
            self.draw_control(ui, field_area, field, slot, st, data);
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

    fn draw_control<D: FormData + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        field: &FieldSpec<'a>,
        slot: &FieldSlot,
        st: &FormState,
        data: &D,
    ) {
        let disabled = data.disabled(field.id);
        let error = Self::field_error(st, data, field.id);
        let options = data.options(field.id);
        let control_area = self.draw_chrome(ui, area, field, error);
        match (&field.kind, data.value(field.id)) {
            (FieldKind::Text(control), FieldRef::Text(value)) => {
                control.draw_in_form(ui, control_area, &slot.input, value, disabled);
            }
            (FieldKind::Text(control), FieldRef::Secret(value)) => {
                control.draw_secret_in_form(ui, control_area, &slot.input, value, disabled);
            }
            (FieldKind::Area(control), FieldRef::Text(value)) => {
                control.draw_in_form(ui, control_area, &slot.area, value, disabled);
            }
            (FieldKind::Area(control), FieldRef::Secret(value)) => {
                control.draw_secret_in_form(ui, control_area, &slot.area, value, disabled);
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
            (FieldKind::Radio(control), FieldRef::Choice(value)) => {
                control.draw_in_form(
                    ui,
                    control_area,
                    &slot.radio,
                    value,
                    options,
                    InheritedFormState { disabled },
                );
            }
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
                for (row, (text, _role)) in area.rows().zip(rows.iter()) {
                    ui.paint_str(row, text, style.style);
                    ui.register_decor(field.id, PartRef::of(Part::HELP), row);
                }
            }
            _ => {}
        }
    }

    fn draw_chrome(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        field: &FieldSpec<'_>,
        error: Option<&str>,
    ) -> Rect {
        if area.is_empty() {
            return area;
        }
        let label = Rect { height: 1, ..area };
        let label_style = self.ov.style(
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
        let used = ui.paint_str(text, field.label, label_style.style);
        if field.required && !field.label.is_empty() {
            ui.paint_str(
                Rect {
                    x: text.x.saturating_add(used).saturating_add(1),
                    width: 1,
                    ..text
                },
                "*",
                label_style.style,
            );
        }
        ui.register_decor(field.id, PartRef::of(Part::LABEL), label);
        let message = error.or(field.help);
        if let Some(message) = message
            && area.height >= 3
        {
            let help = Rect {
                x: area.x.saturating_add(2),
                y: area.bottom().saturating_sub(1),
                width: area.width.saturating_sub(2),
                height: 1,
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
            ui.register_decor(field.id, PartRef::of(Part::HELP), help);
        }
        Rect {
            y: area.y.saturating_add(1),
            height: area.height.saturating_sub(2),
            ..area
        }
    }

    /// Preferred form extent under the declared fields and design tokens.
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
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::event::{Input, Key, KeyModifiers};
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

    #[derive(Default)]
    struct Data {
        name: String,
        hidden: String,
        flag: bool,
        choice: usize,
        secret: Secret,
        chips: KeySet,
        flags: TestFlags,
    }

    impl FormData for Data {
        fn value(&self, id: Id) -> FieldRef<'_> {
            match id {
                NAME => FieldRef::Text(&self.name),
                HIDDEN => FieldRef::Text(&self.hidden),
                FLAG => FieldRef::Flag(self.flag),
                CHOICE => FieldRef::Choice(self.choice),
                SECRET => FieldRef::Secret(&self.secret),
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
                HIDDEN => FieldMut::Text(&mut self.hidden),
                FLAG => FieldMut::Flag(&mut self.flag),
                CHOICE => FieldMut::Choice(&mut self.choice),
                SECRET => FieldMut::Secret(&mut self.secret),
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
        [
            FieldSpec::new(NAME, "Name", FieldKind::Text(TextInput::new(NAME))),
            FieldSpec::new(HIDDEN, "Hidden", FieldKind::Text(TextInput::new(HIDDEN))),
            FieldSpec::new(FLAG, "", FieldKind::Check(Checkbox::new(FLAG, "Flag"))),
            FieldSpec::new(
                CHOICE,
                "Choice",
                FieldKind::Select(LabelSelect::new(CHOICE)),
            ),
            FieldSpec::new(
                SECRET,
                "Secret",
                FieldKind::Text(TextInput::new(SECRET).secret(SecretPolicy::default())),
            ),
            FieldSpec::new(NOTE, "", FieldKind::Note),
            FieldSpec::new(
                CHOOSER,
                "Chooser",
                FieldKind::Chooser(Button::new(CHOOSER, "Choose")),
            ),
        ]
    }

    fn draw(data: &Data, state: &mut FormState) -> (Runtime<Stub>, Buffer) {
        let fields = fields();
        state.reconcile_fields(&fields);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_scene(SCREEN, &mut buffer, |ui, area| {
            Form::new(FORM, &fields).draw(ui, area, state, data);
        });
        (runtime, buffer)
    }

    fn draw_secret_field(data: &Data, state: &mut FormState, kind: FieldKind<'static>) -> Buffer {
        let fields = [FieldSpec::new(SECRET, "Secret", kind)];
        state.reconcile_fields(&fields);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_scene(SCREEN, &mut buffer, |ui, area| {
            Form::new(FORM, &fields).draw(ui, area, state, data);
        });
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
                    value: "chosen",
                    detail: None,
                },
            }
        }

        fn value_mut(&mut self, _id: Id) -> FieldMut<'_> {
            match self.kind {
                MatrixKind::Text | MatrixKind::Area => FieldMut::Text(&mut self.text),
                MatrixKind::Select | MatrixKind::Radio => FieldMut::Choice(&mut self.choice),
                MatrixKind::Chips => FieldMut::Chips(&mut self.chips),
                MatrixKind::Check | MatrixKind::Toggle => FieldMut::Flag(&mut self.flag),
                MatrixKind::Chooser => FieldMut::ReadOnly,
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
            }
        }
    }

    impl App for MatrixApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let fields = [matrix_field(self.data.kind)];
            let response = Form::new(FORM, &fields)
                .actions(ACTIONS)
                .enter(self.enter)
                .update(cx, &mut self.state, &mut self.data);
            self.last = response.action_ref().copied();
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let fields = [matrix_field(self.data.kind)];
            let form = Form::new(FORM, &fields).actions(ACTIONS).enter(self.enter);
            if self.reference {
                ui.reference(self.target, |ui| {
                    form.draw(ui, SCREEN, &self.state, &self.data);
                });
            } else {
                form.draw(ui, SCREEN, &self.state, &self.data);
            }
        }

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    fn matrix_runtime(kind: MatrixKind) -> (Runtime<MatrixApp>, Buffer) {
        let mut runtime = Runtime::new(MatrixApp::new(kind), Theme::junie());
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.focus(), Some(MATRIX));
        (runtime, buffer)
    }

    struct FieldsApp {
        state: FormState,
        data: Data,
        last: Option<FormAction>,
        area: Rect,
    }

    impl Default for FieldsApp {
        fn default() -> Self {
            Self {
                state: FormState::default(),
                data: Data::default(),
                last: None,
                area: SCREEN,
            }
        }
    }

    impl App for FieldsApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let fields = fields();
            let response = Form::new(FORM, &fields).update(cx, &mut self.state, &mut self.data);
            self.last = response.action_ref().copied();
            response.erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let fields = fields();
            Form::new(FORM, &fields).draw(ui, self.area, &self.state, &self.data);
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
        runtime.draw_scene(SCREEN, &mut buffer, |ui, area| {
            Form::new(FORM, &fields).draw(ui, area, &state, &data);
        });
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

    #[test]
    fn scroll_reveals_the_focused_field_from_update_not_draw() {
        let tiny = Rect::new(0, 0, SCREEN.width, 4);
        let app = FieldsApp {
            area: tiny,
            ..FieldsApp::default()
        };
        let mut runtime = Runtime::new(app, Theme::junie());
        let mut buffer = Buffer::empty(tiny);
        runtime.draw_buffer(tiny, &mut buffer);
        runtime.set_focus(Some(CHOOSER));
        let _ = runtime.handle(Input::Tick);
        assert_eq!(runtime.app().state.scroll.offset(), 0);
        runtime.draw_buffer(tiny, &mut buffer);
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(tiny, &mut buffer);
        assert!(runtime.app().state.scroll.offset() > 0);
    }

    #[test]
    fn submit_commits_the_in_flight_edit_before_validating() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Text);
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(press(KeyCode::Char('x')));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.app().data.text, "", "keystroke remains a draft");
        let action_id = FORM.part(Part::ACTIONS).index(0);
        runtime.set_focus(Some(action_id));
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(Input::Key(Key {
            code: KeyCode::Char('s'),
            mods: KeyModifiers::CONTROL,
        }));
        runtime.draw_buffer(SCREEN, &mut buffer);
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
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer);
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
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let action_id = FORM.part(Part::ACTIONS).index(0);
        assert_eq!(runtime.focus(), Some(action_id));
        let _ = runtime.handle(Input::Key(Key {
            code: KeyCode::Char('s'),
            mods: KeyModifiers::CONTROL,
        }));
        runtime.draw_buffer(SCREEN, &mut buffer);
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
            let _ = runtime.handle(press(KeyCode::Enter));
            runtime.draw_buffer(SCREEN, &mut buffer);
            assert_eq!(
                runtime.app().last,
                Some(FormAction::Action(ActionKey::SAVE)),
                "{kind:?} did not yield Enter to Form"
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

        for kind in [MatrixKind::Text, MatrixKind::Area] {
            let (mut runtime, mut buffer) = matrix_runtime(kind);
            let _ = runtime.handle(Input::Tick);
            runtime.draw_buffer(SCREEN, &mut buffer);
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
            let _ = runtime.handle(press(KeyCode::Enter));
            runtime.draw_buffer(SCREEN, &mut buffer);
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
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(Input::Key(Key {
            code: KeyCode::Char('s'),
            mods: KeyModifiers::CONTROL,
        }));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(
            runtime.app().last,
            Some(FormAction::Action(ActionKey::SAVE))
        );
    }

    #[test]
    fn dirty_is_set_by_a_commit_not_by_a_keystroke() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Text);
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert!(!runtime.app().state.is_dirty());
        let _ = runtime.handle(press(KeyCode::Char('x')));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert!(
            !runtime.app().state.is_dirty(),
            "typing changed only the child draft"
        );
        runtime.set_focus(Some(FORM.part(Part::ACTIONS).index(0)));
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert!(
            runtime.app().state.is_dirty(),
            "focus-out commit marked the form dirty"
        );
    }

    #[test]
    fn chooser_activation_emits_chose_with_the_field_id() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Chooser);
        let _ = runtime.handle(press(KeyCode::Char(' ')));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.app().last, Some(FormAction::Chose(MATRIX)));
    }

    #[test]
    fn space_stays_child_owned_and_unrelated_intents_stay_ignored() {
        let (mut runtime, mut buffer) = matrix_runtime(MatrixKind::Check);
        let _ = runtime.handle(press(KeyCode::F(12)));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.app().last, None);
        assert!(!runtime.app().data.flag);
        let _ = runtime.handle(press(KeyCode::Char(' ')));
        runtime.draw_buffer(SCREEN, &mut buffer);
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
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(press(KeyCode::Char('x')));
        runtime.draw_buffer(SCREEN, &mut buffer);
        let action_id = FORM.part(Part::ACTIONS).index(0);
        runtime.set_focus(Some(action_id));
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(Input::Key(Key {
            code: KeyCode::Char('s'),
            mods: KeyModifiers::CONTROL,
        }));
        runtime.draw_buffer(SCREEN, &mut buffer);
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
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert!(runtime.app().state.slots[0].select.is_open());
        let action_id = FORM.part(Part::ACTIONS).index(0);
        let _ = runtime.handle(press(KeyCode::Tab));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.focus(), Some(action_id));
        assert!(!runtime.app().state.slots[0].select.is_open());

        runtime.set_focus(Some(MATRIX));
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert!(runtime.app().state.slots[0].select.is_open());
        let _ = runtime.handle(press(KeyCode::Esc));
        runtime.draw_buffer(SCREEN, &mut buffer);
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
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.app().last, None);
        assert!(!runtime.app().data.flag);

        let mut remapped = MatrixApp::new(MatrixKind::Check);
        remapped.keymap.remap_component(
            MATRIX,
            ActionKey::custom("Toggle (Enter)"),
            Chord::key(KeyCode::F(2)),
        );
        let mut runtime = Runtime::new(remapped, Theme::junie());
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(press(KeyCode::Enter));
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.app().last, None, "removed Enter did not submit");
        let _ = runtime.handle(press(KeyCode::F(2)));
        runtime.draw_buffer(SCREEN, &mut buffer);
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
        let mut state = FormState::default();
        state.reconcile_fields(&fields());
        if let Some(slot) = state.slots.iter_mut().find(|slot| slot.id == SECRET) {
            slot.input.mark_sensitive();
            slot.area.mark_sensitive();
        }
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
    fn secret_validator_error_is_generic_before_form_state_retention() {
        let mut data = Data::default();
        data.secret.set("swordfish");
        let mut state = FormState::default();
        state.reconcile_fields(&fields());
        let error = FieldError::new(format!("invalid {}", data.secret.expose()));
        let safe = Form::safe_error(&data, SECRET, error);
        state.set_error(SECRET, Some(safe));
        assert_eq!(
            state.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{state:?}").contains("swordfish"));
    }

    #[test]
    fn classifying_secret_slot_redacts_preexisting_local_error() {
        let mut data = Data::default();
        data.secret.set("swordfish");
        let fields = fields();
        let mut state = FormState::default();
        state.reconcile_fields(&fields);
        state.set_error(SECRET, Some(FieldError::new("swordfish")));
        let index = fields
            .iter()
            .position(|field| field.id == SECRET)
            .expect("secret field");
        let (value, _) = data.value_and_options(SECRET);
        let _ = Form::prepare_slot(&mut state, index, SECRET, &value);
        assert_eq!(
            state
                .error(SECRET)
                .map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
    }

    #[test]
    fn sensitive_form_state_clone_and_equality_do_not_copy_draft() {
        let mut left = FormState::default();
        left.reconcile_fields(&fields());
        let mut right = FormState::default();
        right.reconcile_fields(&fields());
        for (state, value) in [(&mut left, "swordfish"), (&mut right, "different")] {
            let slot = state
                .slots
                .iter_mut()
                .find(|slot| slot.id == SECRET)
                .expect("secret slot");
            slot.input.mark_sensitive();
            slot.input.begin(value);
        }
        left.set_error(NAME, Some(FieldError::new("name required")));
        right.set_error(NAME, Some(FieldError::new("name required")));
        left.set_error(SECRET, Some(FieldError::new("swordfish")));
        right.set_error(SECRET, Some(FieldError::new("different")));
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
            Some("name required")
        );
        assert_eq!(
            copy.error(SECRET).map(|error| error.message.as_ref()),
            Some("Invalid value")
        );
        assert!(!format!("{copy:?}").contains("swordfish"));
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
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let slot_count = runtime.app().state.slots.len();
        runtime.app_mut().data.config.options = MatrixOptions::Alternate;
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
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
            runtime.draw_buffer(SCREEN, &mut buffer);
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
            runtime.draw_buffer(SCREEN, &mut buffer);
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
        let mut buffer = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut buffer);
        runtime.set_focus(Some(SECRET));
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        let _ = runtime.handle(press(KeyCode::Char('!')));
        runtime.draw_buffer(SCREEN, &mut buffer);
        runtime.set_focus(Some(CHOOSER));
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(SCREEN, &mut buffer);
        assert_eq!(runtime.app().data.secret.expose(), "hunter2!");
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
