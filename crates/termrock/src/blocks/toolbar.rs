use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Modifier;
use termrock_collections::{CollectionCore, Reconcile};
use termrock_core::action::ActionKey;
use termrock_core::event::{Axis, Chord, KeyCode};
use termrock_core::id::{Id, ItemKey, Part, PartRef};
use termrock_core::intent::{Intent, Phase};
use termrock_core::keymap::NavKeys;
use termrock_core::list::{Cursor, RowSource};
use termrock_core::response::{Response, StateFlags};
use termrock_layout::{Constraints, Size};
use termrock_overlays::{ContextMenu, MenuAction, MenuItem, MenuState};
use termrock_runtime::focus::Focusability;
use termrock_runtime::layer::{CrossAlign, LayerEvent, LayerOpening, Side};
use termrock_runtime::{
    Binding, Cx, FrameRead, LayoutFacts, Measure, ReferenceState, ReferenceTarget, Ui,
};
use termrock_text::width;
use termrock_theme::{
    Family, FgStep, GlyphRole, Resolved, Role, Slot, StyleDefaults, StylePatch, Variant,
};

mod tooltip;
use self::tooltip::{TooltipConfig, TooltipController};

const ACTIVATE_ENTER: ActionKey = ActionKey::custom("termrock.toolbar.activate.enter");
const ACTIVATE_SPACE: ActionKey = ActionKey::custom("termrock.toolbar.activate.space");
const OVERFLOW_MENU_ID: &str = "more-menu";
const DEFAULT_MORE_TIP: &str = "Show hidden";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToolbarBinding {
    Activate,
}

static BINDINGS: &[Binding<ToolbarBinding>] = &[
    Binding {
        action: ACTIVATE_ENTER,
        chord: Some(Chord::key(KeyCode::Enter)),
        cmd: ToolbarBinding::Activate,
        label: "Activate",
        priority: 80,
        visible: true,
    },
    Binding {
        action: ACTIVATE_SPACE,
        chord: Some(Chord::key(KeyCode::Char(' '))),
        cmd: ToolbarBinding::Activate,
        label: "Activate",
        priority: 80,
        visible: false,
    },
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolItem {
    pub label: String,

    pub shortcut: Option<String>,

    pub disabled: bool,

    pub on: Option<bool>,

    pub separator_after: bool,

    pub tip: Option<String>,

    pub reason: Option<String>,
}

impl ToolItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_owned());
        self
    }

    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[must_use]
    pub fn toggle(mut self, on: bool) -> Self {
        self.on = Some(on);
        self
    }

    #[must_use]
    pub fn separator(mut self) -> Self {
        self.separator_after = true;
        self
    }

    #[must_use]
    pub fn tip(mut self, tip: &str) -> Self {
        self.tip = Some(tip.to_owned());
        self
    }

    #[must_use]
    pub fn reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_owned());
        self.disabled = true;
        self
    }

    fn width(&self) -> u16 {
        let marker = if self.on.is_some() { 2 } else { 0 };
        let shortcut = self
            .shortcut
            .as_deref()
            .map_or(0, |text| width(text).saturating_add(1));
        2u16.saturating_add(marker)
            .saturating_add(width(&self.label))
            .saturating_add(shortcut)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolbarTip {
    pub text: String,

    pub shortcut: Option<String>,
}

impl ToolbarTip {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            shortcut: None,
        }
    }

    #[must_use]
    pub fn shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_owned());
        self
    }
}

impl Default for ToolbarTip {
    fn default() -> Self {
        Self::new(DEFAULT_MORE_TIP)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolbarTooltipPolicy {
    pub enabled: bool,

    pub delay_ms: u64,

    pub max_width: u16,
}

impl ToolbarTooltipPolicy {
    pub const DEFAULT: Self = Self {
        enabled: true,
        delay_ms: 600,
        max_width: 48,
    };

    pub const OFF: Self = Self {
        enabled: false,
        ..Self::DEFAULT
    };

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn delay_ms(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    #[must_use]
    pub const fn max_width(mut self, max_width: u16) -> Self {
        self.max_width = max_width;
        self
    }
}

impl Default for ToolbarTooltipPolicy {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToolbarReferenceState {
    #[default]
    Rest,

    Hover,

    Pressed,

    Focused,

    FocusedHover,

    Selected,

    Disabled,

    Overflow,

    Narrow,
}

impl ToolbarReferenceState {
    pub const ALL: [Self; 9] = [
        Self::Rest,
        Self::Hover,
        Self::Pressed,
        Self::Focused,
        Self::FocusedHover,
        Self::Selected,
        Self::Disabled,
        Self::Overflow,
        Self::Narrow,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarAction {
    Activated(usize),

    Toggled(usize, bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolbarActionEvent {
    pub key: ItemKey,

    pub action: ToolbarAction,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolbarActions {
    events: Vec<ToolbarActionEvent>,
}

impl ToolbarActions {
    pub fn as_slice(&self) -> &[ToolbarActionEvent] {
        &self.events
    }

    pub fn into_vec(self) -> Vec<ToolbarActionEvent> {
        self.events
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ToolbarActionEvent> {
        self.events.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolbarError {
    ItemKeyCount {
        expected: usize,

        actual: usize,
    },

    DuplicateItemKey {
        key: ItemKey,

        first_index: usize,

        duplicate_index: usize,
    },
}

impl fmt::Display for ToolbarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemKeyCount { expected, actual } => {
                write!(f, "expected {expected} item keys, received {actual}")
            }
            Self::DuplicateItemKey {
                key,
                first_index,
                duplicate_index,
            } => write!(
                f,
                "item key {key:?} is duplicated at indexes {first_index} and {duplicate_index}"
            ),
        }
    }
}

impl std::error::Error for ToolbarError {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OverflowSlot {
    key: ItemKey,
    presentation: ToolItem,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PublishedFrame {
    owner: Id,
    more_area: Rect,
    visible_rows: Vec<(ItemKey, Rect)>,
    hidden: Vec<OverflowSlot>,
}

#[derive(Clone, Debug)]
struct OverflowSession {
    owner: Id,
    opening: LayerOpening,
    snapshot: Vec<OverflowSlot>,
    menu_state: MenuState,
}

#[derive(Clone, Debug, Default)]
pub struct ToolbarState {
    collection: CollectionCore,
    observed_keys: Vec<ItemKey>,
    observed_enabled: Vec<bool>,
    more_focused: bool,
    published: RefCell<Option<PublishedFrame>>,
    overflow: Option<OverflowSession>,
    retired_overflow_openings: Vec<LayerOpening>,
    tooltip: TooltipController,
}

impl ToolbarState {
    pub const fn cursor_key(&self) -> Option<ItemKey> {
        self.collection.cursor()
    }

    pub fn cursor_index(&self) -> Option<usize> {
        self.collection
            .cursor()
            .map(|_| self.collection.cursor_index())
    }

    pub fn hidden_keys(&self) -> Vec<ItemKey> {
        self.published
            .borrow()
            .as_ref()
            .map(|frame| frame.hidden.iter().map(|slot| slot.key).collect())
            .unwrap_or_default()
    }

    pub const fn on_more(&self) -> bool {
        self.more_focused
    }

    pub fn menu_open(&self) -> bool {
        self.overflow.is_some()
    }

    fn drain_retired_overflow_openings(&mut self, cx: &mut Cx<'_>) {
        let openings = std::mem::take(&mut self.retired_overflow_openings);
        for opening in openings {
            if !drain_overflow_opening_receipts(cx, opening) {
                self.retired_overflow_openings.push(opening);
            }
        }
    }

    fn retire_overflow_opening(&mut self, cx: &mut Cx<'_>, opening: LayerOpening) {
        if !drain_overflow_opening_receipts(cx, opening)
            && !self.retired_overflow_openings.contains(&opening)
        {
            self.retired_overflow_openings.push(opening);
        }
    }
}

fn drain_overflow_opening_receipts(cx: &mut Cx<'_>, opening: LayerOpening) -> bool {
    let mut terminal = false;
    while let Some(receipt) = cx.take_layer_intent_receipt_for(opening) {
        if matches!(
            receipt.event(),
            LayerEvent::Closed(_) | LayerEvent::Dismissed(_)
        ) {
            terminal = true;
        }
    }
    terminal
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayoutRow {
    index: usize,
    key: ItemKey,
    area: Rect,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ToolbarLayout {
    area: Rect,
    rows: Vec<LayoutRow>,
    separators: Vec<(ItemKey, Rect)>,
    more: Option<Rect>,
    hidden_start: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarMoreLook {
    Close,

    Title,
}

const MORE_STYLE_FAMILY: Family = Family::custom("termrock.toolbar.more");
const MORE_CLOSE_VARIANT: Variant = Variant::custom("close");
const MORE_TITLE_VARIANT: Variant = Variant::custom("title");
const MORE_DANGER_VARIANT: Variant = Variant::custom("danger");

fn more_danger_patch(flags: StateFlags) -> StylePatch {
    if flags.contains(StateFlags::DISABLED) {
        return StylePatch::new();
    }
    if flags.contains(StateFlags::PRESSED) {
        StylePatch::new()
            .set_fg(Role::OnDanger)
            .set_bg(Role::Danger)
    } else if flags.contains(StateFlags::HOVERED) {
        StylePatch::new().set_fg(Role::DangerSoft)
    } else {
        StylePatch::new()
    }
}

fn more_look_patch(look: ToolbarMoreLook, flags: StateFlags, danger_hover: bool) -> StylePatch {
    let patch = match (
        look,
        flags.contains(StateFlags::PRESSED),
        flags.contains(StateFlags::HOVERED),
    ) {
        (ToolbarMoreLook::Close, true, _) => StylePatch::new()
            .set_fg(Role::CurrentSurface)
            .set_bg(Role::Fg(FgStep::Primary))
            .remove(Modifier::BOLD),
        (ToolbarMoreLook::Close, false, true) => StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::RaisedSurface)
            .remove(Modifier::BOLD),
        (ToolbarMoreLook::Close, false, false) => StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::CurrentSurface)
            .remove(Modifier::BOLD),
        (ToolbarMoreLook::Title, true, _) => StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::RaisedSurface)
            .add(Modifier::BOLD),
        (ToolbarMoreLook::Title, false, true) => StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::HoverSurface)
            .remove(Modifier::BOLD),
        (ToolbarMoreLook::Title, false, false) => StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .set_bg(Role::CurrentSurface)
            .remove(Modifier::BOLD),
    };
    if danger_hover {
        patch.merge(more_danger_patch(flags))
    } else {
        patch
    }
}

#[derive(Clone, Debug)]
pub struct Toolbar {
    id: Id,
    items: Vec<ToolItem>,
    item_keys: Vec<ItemKey>,
    nav: NavKeys,
    wrap: bool,
    skip_disabled: bool,
    gap: u16,
    overflow: bool,
    more_glyph: Option<String>,
    more_look: Option<ToolbarMoreLook>,
    more_danger_hover: bool,
    more_padding: u16,
    more_tip: ToolbarTip,
    tooltip_policy: ToolbarTooltipPolicy,
}

impl Toolbar {
    pub fn new(id: Id, items: Vec<ToolItem>) -> Self {
        let item_keys = (0..items.len()).map(ItemKey::index).collect();
        Self {
            id,
            items,
            item_keys,
            nav: NavKeys::default(),
            wrap: false,
            skip_disabled: true,
            gap: 1,
            overflow: true,
            more_glyph: None,
            more_look: None,
            more_danger_hover: false,
            more_padding: 1,
            more_tip: ToolbarTip::default(),
            tooltip_policy: ToolbarTooltipPolicy::DEFAULT,
        }
    }

    pub fn try_item_keys(mut self, keys: Vec<ItemKey>) -> Result<Self, ToolbarError> {
        if keys.len() != self.items.len() {
            return Err(ToolbarError::ItemKeyCount {
                expected: self.items.len(),
                actual: keys.len(),
            });
        }

        let mut first_seen = HashMap::with_capacity(keys.len());
        for (index, key) in keys.iter().copied().enumerate() {
            if let Some(first_index) = first_seen.get(&key).copied() {
                return Err(ToolbarError::DuplicateItemKey {
                    key,
                    first_index,
                    duplicate_index: index,
                });
            }
            first_seen.insert(key, index);
        }

        self.item_keys = keys;
        Ok(self)
    }

    #[must_use]
    pub const fn nav(mut self, nav: NavKeys) -> Self {
        self.nav = nav;
        self
    }

    #[must_use]
    pub const fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    #[must_use]
    pub const fn skip_disabled(mut self, skip_disabled: bool) -> Self {
        self.skip_disabled = skip_disabled;
        self
    }

    #[must_use]
    pub const fn gap(mut self, cells: u16) -> Self {
        self.gap = cells;
        self
    }

    #[must_use]
    pub const fn overflow(mut self, overflow: bool) -> Self {
        self.overflow = overflow;
        self
    }

    #[must_use]
    pub fn more_glyph(mut self, glyph: &str) -> Self {
        self.more_glyph = Some(glyph.to_owned());
        self
    }

    #[must_use]
    pub const fn more_look(mut self, look: ToolbarMoreLook) -> Self {
        self.more_look = Some(look);
        self
    }

    #[must_use]
    pub const fn more_danger_hover(mut self, enabled: bool) -> Self {
        self.more_danger_hover = enabled;
        self
    }

    pub const fn more_style_selector(look: Option<ToolbarMoreLook>) -> (Family, Variant) {
        match look {
            Some(ToolbarMoreLook::Close) => (MORE_STYLE_FAMILY, MORE_CLOSE_VARIANT),
            Some(ToolbarMoreLook::Title) => (MORE_STYLE_FAMILY, MORE_TITLE_VARIANT),
            None => (MORE_STYLE_FAMILY, MORE_DANGER_VARIANT),
        }
    }

    #[must_use]
    pub const fn more_padding(mut self, cells: u16) -> Self {
        self.more_padding = cells;
        self
    }

    #[must_use]
    pub fn more_tip(mut self, tip: ToolbarTip) -> Self {
        self.more_tip = tip;
        self
    }

    #[must_use]
    pub const fn tooltip_policy(mut self, policy: ToolbarTooltipPolicy) -> Self {
        self.tooltip_policy = policy;
        self
    }

    pub const fn more_tip_value(&self) -> &ToolbarTip {
        &self.more_tip
    }

    pub const fn tooltip_policy_value(&self) -> ToolbarTooltipPolicy {
        self.tooltip_policy
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    pub fn items(&self) -> &[ToolItem] {
        &self.items
    }

    pub fn item_keys(&self) -> &[ItemKey] {
        &self.item_keys
    }

    pub fn width(&self) -> u16 {
        let item_width = self
            .items
            .iter()
            .fold(0u16, |sum, item| sum.saturating_add(item.width()));
        let gap_count = self
            .items
            .iter()
            .skip(1)
            .fold(0u16, |count, _| count.saturating_add(1));
        let separator_count = self
            .items
            .iter()
            .filter(|item| item.separator_after)
            .fold(0u16, |count, _| count.saturating_add(1));
        item_width
            .saturating_add(self.gap.saturating_mul(gap_count))
            .saturating_add(separator_count.saturating_mul(1u16.saturating_add(self.gap)))
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut ToolbarState) -> Response<ToolbarActions> {
        state.drain_retired_overflow_openings(cx);

        if state
            .overflow
            .as_ref()
            .is_some_and(|session| session.owner != self.id)
            && let Some(session) = state.overflow.take()
        {
            cx.close_layer(session.owner.sub(OVERFLOW_MENU_ID), None);
            state.retire_overflow_opening(cx, session.opening);
        }
        let published_layout = self.committed_layout(cx);

        let routed_intents = cx.intents(self.id).collect::<Vec<_>>();

        self.reconcile_state(state, Some(&published_layout));

        let mut pending_toggles: Vec<Option<bool>> =
            self.items.iter().map(|item| item.on).collect();
        let mut actions = Vec::new();
        let mut consumed = false;
        let mut changed = false;
        let mut open_overflow = false;

        for intent in routed_intents.iter().copied() {
            match intent {
                Intent::Binding(action)
                    if Binding::command(BINDINGS, action) == Some(ToolbarBinding::Activate) =>
                {
                    consumed = true;
                    if state.more_focused {
                        open_overflow = true;
                    } else {
                        if let Some(event) =
                            self.activate_cursor(state, &published_layout, &mut pending_toggles)
                        {
                            actions.push(event);
                            changed = true;
                        }
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part,
                    ..
                } if part.part == Part::ROW => {
                    consumed = true;
                    let Some(key) = part.item else {
                        continue;
                    };
                    let Some(row) = published_layout.rows.iter().find(|row| row.key == key) else {
                        continue;
                    };
                    let Some(item) = self.items.get(row.index) else {
                        continue;
                    };
                    if item.disabled {
                        continue;
                    }

                    self.set_cursor(state, row.index, row.key);
                    if let Some(event) = self.activate(row.index, &mut pending_toggles) {
                        actions.push(event);
                        changed = true;
                    }
                }
                Intent::Pointer {
                    phase: Phase::Click | Phase::DoubleClick,
                    part,
                    ..
                } if part.part == Part::OVERFLOW => {
                    consumed = true;
                    if published_layout.more.is_some() {
                        state.more_focused = true;
                        open_overflow = true;
                        changed = true;
                    }
                }
                Intent::Key(key) => {
                    if state.more_focused
                        && key.code == KeyCode::Down
                        && key.mods == termrock_core::event::KeyModifiers::NONE
                    {
                        consumed = true;
                        open_overflow = true;
                        changed = true;
                        continue;
                    }
                    let Some(nav) = self.nav.action(&key) else {
                        continue;
                    };
                    let stops = VisibleStops {
                        items: &self.items,
                        rows: &published_layout.rows,
                        more: published_layout.more.is_some(),
                    };
                    if stops.is_empty() {
                        continue;
                    }
                    let from = if state.more_focused {
                        published_layout.rows.len()
                    } else if let Some(cursor_key) = state.cursor_key() {
                        published_layout
                            .rows
                            .iter()
                            .position(|row| row.key == cursor_key)
                            .unwrap_or(0)
                    } else {
                        0
                    };
                    let mut cursor = Cursor::new(from)
                        .wrap(self.wrap)
                        .axis(Axis::H)
                        .skip_disabled(self.skip_disabled);
                    let moved = cursor.apply(nav, &stops, 1);
                    let Some(position) = moved.index() else {
                        continue;
                    };

                    consumed = true;
                    if moved.moved() {
                        if position == published_layout.rows.len() {
                            state.more_focused = true;
                        } else if let Some(row) = published_layout.rows.get(position) {
                            self.set_cursor(state, row.index, row.key);
                        }
                        changed = true;
                    }
                }
                _ => {}
            }
        }

        if open_overflow {
            changed |= self.open_overflow(cx, state);
        }

        let mut menu_consumed = false;
        let mut menu_changed = false;
        let mut retire_overflow = false;
        let menu_id = self.id.sub(OVERFLOW_MENU_ID);
        if let Some(session) = state.overflow.as_mut() {
            let mut projected = session
                .snapshot
                .iter()
                .map(|slot| self.project_slot(slot))
                .collect::<Vec<_>>();
            let menu_items = make_overflow_items(
                &mut projected,
                cx.design().glyphs.get(GlyphRole::DotOn),
                cx.design().glyphs.get(GlyphRole::DotOff),
            );
            let menu = ContextMenu::at(menu_id, &menu_items, Position::new(0, 0))
                .anchor_to_part(
                    self.id,
                    PartRef::of(Part::OVERFLOW),
                    Side::Below,
                    CrossAlign::Start,
                )
                .item_bindings(false);
            let (response, selection) = menu.update_with_selection(cx, &mut session.menu_state);
            menu_consumed = response.is_consumed();
            menu_changed = response.is_changed();

            if matches!(response.action_ref(), Some(MenuAction::Chosen(_))) {
                if let Some(slot) = selection.and_then(|index| session.snapshot.get(index))
                    && let Some(index) = self.index_of_key(slot.key)
                    && let Some(item) = self.items.get(index)
                    && !item.disabled
                    && let Some(event) = self.activate(index, &mut pending_toggles)
                {
                    actions.push(event);
                }
                changed = true;
            }

            retire_overflow = !cx.is_open(menu_id);
        }
        if retire_overflow && let Some(session) = state.overflow.take() {
            state.retire_overflow_opening(cx, session.opening);
        }

        state.tooltip.update(
            cx,
            TooltipConfig {
                owner: self.id,
                items: &self.items,
                item_keys: &self.item_keys,
                policy: self.tooltip_policy,
                more_tip: &self.more_tip,
            },
            &published_layout,
            &routed_intents,
            state.overflow.is_some(),
        );

        if !actions.is_empty() {
            Response::action(ToolbarActions { events: actions }).for_id(self.id)
        } else if changed || menu_changed {
            Response::changed().for_id(self.id)
        } else if consumed || menu_consumed {
            Response::consumed().for_id(self.id)
        } else {
            Response::ignored().for_id(self.id)
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &ToolbarState) -> Rect {
        let runtime_state = ui.state(self.id);
        let hovered = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        let remove_root = StateFlags::FOCUSED
            | StateFlags::FOCUS_VISIBLE
            | StateFlags::HOVERED
            | StateFlags::PRESSED
            | StateFlags::SELECTED
            | StateFlags::CHECKED
            | StateFlags::DISABLED;
        let root_flags = runtime_state.difference(remove_root);
        let mut more_flags = root_flags;
        if state.more_focused && runtime_state.contains(StateFlags::FOCUSED) {
            more_flags |= StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE;
        }
        let more_part = PartRef::of(Part::OVERFLOW);
        if hovered == Some(more_part) {
            more_flags |= StateFlags::HOVERED;
        }
        if pressed == Some(more_part) {
            more_flags |= StateFlags::PRESSED;
        }
        let more_style = ui.style(Family::BUTTON, Variant::SUBTLE, Part::OVERFLOW, more_flags);
        let more_glyph = match self.more_glyph.as_deref() {
            Some(glyph) => glyph,
            None => match more_style.glyph {
                Slot::Set(role) => ui.glyph_str(role),
                Slot::Inherit | Slot::Clear => ui.theme_ref().design.glyphs.more_titles(),
            },
        };
        let more_width = width(more_glyph).saturating_add(self.more_padding.saturating_mul(2));
        let layout = self.layout(area, more_width);
        let visible_count = layout.rows.len();
        let logical_rows = if visible_count == 0 { 0 } else { 1 };
        ui.report_layout(
            self.id,
            LayoutFacts::new(
                visible_count,
                self.items.len(),
                logical_rows,
                layout.area.width,
            )
            .with_logical_area(layout.area),
        );

        let clip = ui.full();
        let visible_rows: Vec<_> = layout
            .rows
            .iter()
            .copied()
            .filter_map(|row| {
                let area = row.area.intersection(clip);
                (!area.is_empty()).then_some((row, area))
            })
            .collect();
        let more_area = layout
            .more
            .map(|area| area.intersection(clip))
            .filter(|area| !area.is_empty());

        if layout.rows.is_empty() && more_area.is_none() {
            if !ui.is_inert() {
                *state.published.borrow_mut() = None;
            }
            return layout.area;
        }

        let focus_layout = ToolbarLayout {
            area: layout.area.intersection(clip),
            rows: visible_rows.iter().map(|(row, _)| *row).collect(),
            separators: Vec::new(),
            more: more_area,
            hidden_start: layout.hidden_start,
        };
        let has_reachable_row = focus_layout.rows.iter().any(|row| {
            self.items
                .get(row.index)
                .is_some_and(|item| !self.skip_disabled || !item.disabled)
        }) || focus_layout.more.is_some();
        let focusability = if has_reachable_row {
            Focusability::Focusable
        } else {
            Focusability::Disabled
        };

        if !ui.is_inert() {
            ui.register_control(self.id, layout.area, focusability);
            for row in &layout.rows {
                ui.register_part(self.id, PartRef::item(Part::ROW, row.key), row.area);
                if self.tooltip_policy.enabled
                    && self
                        .items
                        .get(row.index)
                        .and_then(|item| item.tip.as_deref())
                        .is_some_and(|tip| !tip.is_empty())
                {
                    ui.register_hover_part(self.id, PartRef::item(Part::ROW, row.key), row.area);
                }
            }
            if let Some(more) = layout.more {
                ui.register_part(self.id, PartRef::of(Part::OVERFLOW), more);
                if self.tooltip_policy.enabled && !self.more_tip.text.is_empty() {
                    ui.register_hover_part(self.id, PartRef::of(Part::OVERFLOW), more);
                }
            }
            for (key, separator) in &layout.separators {
                ui.register_decor(self.id, PartRef::item(Part::RULE, *key), *separator);
            }
            if focusability == Focusability::Focusable {
                let live = ui.state(self.id);
                ui.publish_bindings(self.id, live, BINDINGS);
            }
        }

        let current = self.display_cursor(state, &focus_layout);

        for row in &layout.rows {
            let Some(item) = self.items.get(row.index) else {
                continue;
            };
            let part = PartRef::item(Part::ROW, row.key);
            let is_cursor = current == Some(row.index);
            let mut flags = root_flags;
            if item.on == Some(true) {
                flags |= StateFlags::CHECKED | StateFlags::SELECTED;
            }
            if item.disabled {
                flags |= StateFlags::DISABLED;
                flags = flags.difference(
                    StateFlags::FOCUSED
                        | StateFlags::FOCUS_VISIBLE
                        | StateFlags::HOVERED
                        | StateFlags::PRESSED,
                );
            } else {
                if is_cursor && runtime_state.contains(StateFlags::FOCUSED) {
                    flags |= StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE;
                }
                if hovered == Some(part) {
                    flags |= StateFlags::HOVERED;
                }
                if pressed == Some(part) {
                    flags |= StateFlags::PRESSED;
                }
            }

            self.draw_item(ui, row.area, item, flags);
        }

        for (_, separator) in &layout.separators {
            let style = ui
                .style(Family::BUTTON, Variant::DEFAULT, Part::RULE, root_flags)
                .style;
            ui.paint_str(*separator, "│", style);
        }

        if let Some(more) = layout.more {
            if self.more_look.is_some() || self.more_danger_hover {
                self.draw_more_with_appearance(ui, more, more_glyph, more_flags, more_style);
            } else {
                self.draw_more(ui, more, more_glyph, more_flags, more_style);
            }
        }

        if !ui.is_inert() {
            let published = more_area.and_then(|more_area| {
                let hidden = self
                    .items
                    .iter()
                    .zip(self.item_keys.iter().copied())
                    .filter(|(_, key)| !visible_rows.iter().any(|(row, _)| row.key == *key))
                    .map(|(presentation, key)| OverflowSlot {
                        key,
                        presentation: presentation.clone(),
                    })
                    .collect::<Vec<_>>();
                (!hidden.is_empty()).then(|| PublishedFrame {
                    owner: self.id,
                    more_area,
                    visible_rows: visible_rows
                        .iter()
                        .map(|(row, area)| (row.key, *area))
                        .collect(),
                    hidden,
                })
            });
            *state.published.borrow_mut() = published;
        }

        if let Some(session) = state
            .overflow
            .as_ref()
            .filter(|session| session.owner == self.id)
        {
            let mut projected = session
                .snapshot
                .iter()
                .map(|slot| self.project_slot(slot))
                .collect::<Vec<_>>();
            let menu_items = make_overflow_items(
                &mut projected,
                ui.glyph_str(GlyphRole::DotOn),
                ui.glyph_str(GlyphRole::DotOff),
            );
            let menu_id = self.id.sub(OVERFLOW_MENU_ID);
            let menu = ContextMenu::at(menu_id, &menu_items, Position::new(0, 0))
                .anchor_to_part(
                    self.id,
                    PartRef::of(Part::OVERFLOW),
                    Side::Below,
                    CrossAlign::Start,
                )
                .item_bindings(false);
            let _ = ui.layer(menu_id, |ui, area| menu.draw(ui, area, &session.menu_state));
        }

        let live_rows = visible_rows
            .iter()
            .map(|(row, clipped)| (row.key, row.index, *clipped))
            .collect::<Vec<_>>();
        state.tooltip.draw(
            ui,
            TooltipConfig {
                owner: self.id,
                items: &self.items,
                item_keys: &self.item_keys,
                policy: self.tooltip_policy,
                more_tip: &self.more_tip,
            },
            &live_rows,
            more_area,
            state.overflow.is_some(),
        );

        layout.area
    }

    pub fn draw_reference(
        &self,
        ui: &mut Ui<'_>,
        mut area: Rect,
        reference_state: ToolbarReferenceState,
    ) -> Rect {
        let mut reference = self.clone();
        for item in &mut reference.items {
            item.tip = None;
        }
        reference.more_tip = ToolbarTip::new("");
        reference.tooltip_policy = ToolbarTooltipPolicy::OFF;

        let target_index =
            (!reference.items.is_empty()).then(|| 1.min(reference.items.len().saturating_sub(1)));
        let target_key = target_index.and_then(|index| reference.item_keys.get(index).copied());
        let mut detached_state = ToolbarState::default();
        if let (Some(index), Some(key)) = (target_index, target_key) {
            reference.set_cursor(&mut detached_state, index, key);
        }

        match reference_state {
            ToolbarReferenceState::Selected => {
                if let Some(item) = reference.items.iter_mut().find(|item| item.on.is_some()) {
                    item.on = Some(true);
                }
            }
            ToolbarReferenceState::Disabled => {
                if let Some(index) = target_index
                    && let Some(item) = reference.items.get_mut(index)
                {
                    item.disabled = true;
                }
            }
            ToolbarReferenceState::Overflow => {
                reference.overflow = true;
                area.width = area.width.min(reference.width().saturating_sub(1));
            }
            ToolbarReferenceState::Narrow => area.width = area.width.min(16),
            ToolbarReferenceState::Rest
            | ToolbarReferenceState::Hover
            | ToolbarReferenceState::Pressed
            | ToolbarReferenceState::Focused
            | ToolbarReferenceState::FocusedHover => {}
        }

        let target_state = match reference_state {
            ToolbarReferenceState::Hover => Some(ReferenceState::HOVERED),
            ToolbarReferenceState::Pressed => {
                Some(ReferenceState::HOVERED | ReferenceState::PRESSED)
            }
            ToolbarReferenceState::Focused => {
                Some(ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE)
            }
            ToolbarReferenceState::FocusedHover => Some(
                ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE | ReferenceState::HOVERED,
            ),
            ToolbarReferenceState::Rest
            | ToolbarReferenceState::Selected
            | ToolbarReferenceState::Disabled
            | ToolbarReferenceState::Overflow
            | ToolbarReferenceState::Narrow => None,
        };
        let target = target_state.zip(target_key).map(|(state, key)| {
            ReferenceTarget::new(self.id, state).part(PartRef::item(Part::ROW, key))
        });

        if matches!(
            reference_state,
            ToolbarReferenceState::Focused | ToolbarReferenceState::FocusedHover
        ) {
            let focus_mark = if ui.glyph_str(GlyphRole::FocusBar).is_ascii() {
                ":"
            } else {
                "▏"
            };
            let theme = ui
                .theme_ref()
                .clone()
                .builder()
                .glyph(GlyphRole::FocusBar, focus_mark)
                .build();
            ui.with_theme(&theme, |ui| {
                ui.reference(target, |ui| reference.draw(ui, area, &detached_state))
            })
        } else {
            ui.reference(target, |ui| reference.draw(ui, area, &detached_state))
        }
    }

    fn committed_layout(&self, cx: &impl FrameRead) -> ToolbarLayout {
        let mut layout = ToolbarLayout {
            area: cx.area(self.id).unwrap_or_default(),
            more: cx.area_of_part(self.id, PartRef::of(Part::OVERFLOW)),
            ..ToolbarLayout::default()
        };

        for (index, key) in self.item_keys.iter().copied().enumerate() {
            let Some(area) = cx.area_of_part(self.id, PartRef::item(Part::ROW, key)) else {
                continue;
            };
            if area.is_empty() {
                continue;
            }
            layout.rows.push(LayoutRow { index, key, area });
        }

        layout.rows.sort_by_key(|row| (row.area.y, row.area.x));
        layout
    }

    fn activate_cursor(
        &self,
        state: &ToolbarState,
        layout: &ToolbarLayout,
        pending_toggles: &mut [Option<bool>],
    ) -> Option<ToolbarActionEvent> {
        let index = state.cursor_index()?;
        let key = state.cursor_key()?;
        layout
            .rows
            .iter()
            .any(|row| row.index == index && row.key == key)
            .then(|| self.activate(index, pending_toggles))
            .flatten()
    }

    fn published_frame_matches(&self, cx: &impl FrameRead, frame: &PublishedFrame) -> bool {
        frame.owner == self.id
            && !frame.more_area.is_empty()
            && cx.area_of_part(self.id, PartRef::of(Part::OVERFLOW)) == Some(frame.more_area)
            && frame.visible_rows.iter().all(|(key, area)| {
                cx.area_of_part(self.id, PartRef::item(Part::ROW, *key)) == Some(*area)
            })
            && frame.hidden.iter().all(|slot| {
                cx.area_of_part(self.id, PartRef::item(Part::ROW, slot.key))
                    .is_none()
            })
    }

    fn project_slot(&self, slot: &OverflowSlot) -> ToolItem {
        let Some(index) = self.index_of_key(slot.key) else {
            let mut tombstone = slot.presentation.clone();
            tombstone.disabled = true;
            return tombstone;
        };
        let Some(item) = self.items.get(index) else {
            let mut tombstone = slot.presentation.clone();
            tombstone.disabled = true;
            return tombstone;
        };
        item.clone()
    }

    fn open_overflow(&self, cx: &mut Cx<'_>, state: &mut ToolbarState) -> bool {
        if !self.overflow || state.overflow.is_some() {
            return false;
        }
        let published = state.published.borrow().clone();
        let Some(frame) = published.filter(|frame| self.published_frame_matches(cx, frame)) else {
            return false;
        };
        if frame.hidden.is_empty() {
            return false;
        }
        let snapshot = frame.hidden;
        let mut projected = snapshot
            .iter()
            .map(|slot| self.project_slot(slot))
            .collect::<Vec<_>>();
        let menu_items = make_overflow_items(
            &mut projected,
            cx.design().glyphs.get(GlyphRole::DotOn),
            cx.design().glyphs.get(GlyphRole::DotOff),
        );
        let menu_id = self.id.sub(OVERFLOW_MENU_ID);
        let menu = ContextMenu::at(menu_id, &menu_items, Position::new(0, 0))
            .anchor_to_part(
                self.id,
                PartRef::of(Part::OVERFLOW),
                Side::Below,
                CrossAlign::Start,
            )
            .item_bindings(false);
        let Some(layer) = menu.try_layer(cx) else {
            return false;
        };
        let Some(opening) = cx.open_layer_with_token(menu_id, layer) else {
            return false;
        };
        state.overflow = Some(OverflowSession {
            owner: self.id,
            opening,
            snapshot,
            menu_state: MenuState::default(),
        });
        true
    }

    fn draw_item(&self, ui: &mut Ui<'_>, area: Rect, item: &ToolItem, flags: StateFlags) {
        let variant = if item.on.is_some() {
            Variant::TOGGLE
        } else {
            Variant::SUBTLE
        };
        let container = ui
            .style(Family::BUTTON, variant, Part::CONTAINER, flags)
            .style;
        ui.fill(area, container);

        let pad = container.as_style().bg.map_or(container, |bg| {
            container.fg(bg).remove_modifier(Modifier::BOLD)
        });
        let gutter_area = Rect::new(area.x, area.y, 1, 1);
        let gutter = ui.style(Family::BUTTON, variant, Part::GUTTER, flags);
        match gutter.glyph {
            Slot::Set(glyph) => {
                ui.glyph(gutter_area, glyph, gutter.style);
            }
            Slot::Inherit | Slot::Clear => ui.fill(gutter_area, pad),
        }

        let mut x = area.x.saturating_add(1);
        if let Some(on) = item.on {
            let marker = ui.style(Family::BUTTON, variant, Part::MARKER, flags);
            let marker_glyph = match marker.glyph {
                Slot::Set(glyph) => Some(glyph),
                Slot::Inherit if on => Some(GlyphRole::DotOn),
                Slot::Inherit => Some(GlyphRole::DotOff),
                Slot::Clear => None,
            };
            let marker_area = Rect::new(x, area.y, 1, 1);
            if let Some(glyph) = marker_glyph {
                ui.glyph(marker_area, glyph, marker.over(container));
            } else {
                ui.fill(marker_area, marker.style);
            }
            x = x.saturating_add(2);
        }

        let label_width = width(&item.label);
        let label_area = Rect::new(x, area.y, label_width, 1);
        let label = ui.style(Family::BUTTON, variant, Part::LABEL, flags);
        ui.fill(label_area, label.style);
        ui.paint_str(label_area, &item.label, label.over(container));
        x = x.saturating_add(label_width);

        if let Some(shortcut) = item.shortcut.as_deref() {
            x = x.saturating_add(1);
            let meta = ui.style(Family::BUTTON, variant, Part::META, flags);
            let style = meta.over(container).remove_modifier(Modifier::BOLD);
            ui.paint_str(Rect::new(x, area.y, width(shortcut), 1), shortcut, style);
        }
    }

    fn draw_more(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        glyph: &str,
        flags: StateFlags,
        overflow: Resolved,
    ) {
        let variant = Variant::SUBTLE;
        let container = ui
            .style(Family::BUTTON, variant, Part::CONTAINER, flags)
            .style;
        ui.fill(area, container);

        let left_padding = self.more_padding.min(area.width);
        if left_padding > 0 {
            let gutter_area = Rect::new(area.x, area.y, 1, 1);
            let gutter = ui.style(Family::BUTTON, variant, Part::GUTTER, flags);
            match gutter.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(gutter_area, glyph, gutter.style);
                }
                Slot::Inherit | Slot::Clear => {
                    let pad = container.as_style().bg.map_or(container, |bg| {
                        container.fg(bg).remove_modifier(Modifier::BOLD)
                    });
                    ui.fill(gutter_area, pad);
                }
            }
        }

        let glyph_area = Rect::new(
            area.x.saturating_add(left_padding),
            area.y,
            width(glyph).min(area.width.saturating_sub(left_padding)),
            1,
        );
        ui.paint_str(glyph_area, glyph, overflow.over(container));
    }

    fn draw_more_with_appearance(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        glyph: &str,
        flags: StateFlags,
        overflow_base: Resolved,
    ) {
        let button_variant = Variant::SUBTLE;
        let container_base = if self.more_look.is_some() {
            Resolved::default()
        } else {
            ui.style(Family::BUTTON, button_variant, Part::CONTAINER, flags)
        };
        let style_flags = if self.more_look.is_some() {
            flags.difference(StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE)
        } else {
            flags
        };
        let gutter_base = ui.style(Family::BUTTON, button_variant, Part::GUTTER, style_flags);
        let container =
            self.resolve_more_appearance(ui, Part::CONTAINER, style_flags, container_base);
        let gutter = self.resolve_more_appearance(ui, Part::GUTTER, style_flags, gutter_base);
        let overflow = self.resolve_more_appearance(ui, Part::OVERFLOW, style_flags, overflow_base);
        let container_style = container.style;
        ui.fill(area, container_style);

        let left_padding = self.more_padding.min(area.width);
        if left_padding > 0 {
            let gutter_area = Rect::new(area.x, area.y, 1, 1);
            match gutter.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(gutter_area, glyph, gutter.style);
                }
                Slot::Inherit | Slot::Clear => {
                    let pad = container_style.as_style().bg.map_or(container_style, |bg| {
                        container_style.fg(bg).remove_modifier(Modifier::BOLD)
                    });
                    ui.fill(gutter_area, pad);
                }
            }
        }

        let glyph_area = Rect::new(
            area.x.saturating_add(left_padding),
            area.y,
            width(glyph).min(area.width.saturating_sub(left_padding)),
            1,
        );
        ui.paint_str(glyph_area, glyph, overflow.over(container_style));
    }

    fn resolve_more_appearance(
        &self,
        ui: &Ui<'_>,
        part: Part,
        flags: StateFlags,
        mut base: Resolved,
    ) -> Resolved {
        if let Some(look) = self.more_look {
            let (family, variant) = Self::more_style_selector(Some(look));
            let defaults = StyleDefaults::new(more_look_patch(look, flags, self.more_danger_hover));
            let appearance = ui.style_defaults(family, variant, part, flags, defaults, None);
            base.style = appearance.style;
            return base;
        }

        let danger_patch = more_danger_patch(flags);
        if !self.more_danger_hover || danger_patch.is_empty() {
            return base;
        }

        let (family, variant) = Self::more_style_selector(None);
        let danger = ui.style_defaults(
            family,
            variant,
            part,
            flags,
            StyleDefaults::new(danger_patch),
            None,
        );
        let mut style = base.style.with_fg_from(danger.style);
        if flags.contains(StateFlags::PRESSED) {
            style = style.with_bg_from(danger.style);
        }
        base.style = style;
        base
    }

    fn layout(&self, area: Rect, configured_more_width: u16) -> ToolbarLayout {
        let area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: if area.height == 0 { 0 } else { 1 },
        };
        if area.is_empty() || self.items.is_empty() {
            return ToolbarLayout {
                area,
                ..ToolbarLayout::default()
            };
        }

        let full = self.layout_items(area, area.right());
        if !self.overflow || full.hidden_start.is_none() {
            return full;
        }

        let more_width = area.width.min(configured_more_width);
        if more_width == 0 {
            return full;
        }
        let more_x = area.right().saturating_sub(more_width);
        let content_right = more_x.saturating_sub(self.gap);
        let mut overflow = self.layout_items(area, content_right);
        overflow.more = Some(Rect::new(more_x, area.y, more_width, 1));
        if overflow.hidden_start.is_none() {
            overflow.hidden_start = Some(overflow.rows.len());
        }
        overflow
    }

    fn layout_items(&self, area: Rect, right: u16) -> ToolbarLayout {
        let mut layout = ToolbarLayout {
            area,
            ..ToolbarLayout::default()
        };
        if area.is_empty() || self.items.is_empty() {
            return layout;
        }
        let mut x = area.x;
        for (index, (item, key)) in self.items.iter().zip(self.item_keys.iter()).enumerate() {
            let item_width = item.width();
            if x.saturating_add(item_width) > right {
                layout.hidden_start = Some(index);
                break;
            }
            layout.rows.push(LayoutRow {
                index,
                key: *key,
                area: Rect::new(x, area.y, item_width, 1),
            });

            x = x.saturating_add(item_width);
            let has_next = index.saturating_add(1) < self.items.len();
            if item.separator_after {
                let separator_x = x.saturating_add(self.gap);
                if has_next {
                    let next_x = separator_x.saturating_add(1).saturating_add(self.gap);
                    let next_width = self
                        .items
                        .get(index.saturating_add(1))
                        .map_or(0, ToolItem::width);
                    if separator_x >= right || next_x.saturating_add(next_width) > right {
                        layout.hidden_start = Some(index.saturating_add(1));
                        break;
                    }
                    layout
                        .separators
                        .push((*key, Rect::new(separator_x, area.y, 1, 1)));
                    x = next_x;
                } else if separator_x < right {
                    layout
                        .separators
                        .push((*key, Rect::new(separator_x, area.y, 1, 1)));
                }
            } else if has_next {
                x = x.saturating_add(self.gap);
            }
        }
        layout
    }

    fn reconcile_state(&self, state: &mut ToolbarState, layout: Option<&ToolbarLayout>) {
        let enabled: Vec<bool> = self.items.iter().map(|item| !item.disabled).collect();
        let source_changed =
            state.observed_keys != self.item_keys || state.observed_enabled != enabled;
        if source_changed {
            state.collection.invalidate();
        }
        let collection_eligible = |index: usize| {
            self.items
                .get(index)
                .is_some_and(|item| !self.skip_disabled || !item.disabled)
        };
        let _ = state.collection.reconcile_with(
            self.items.len(),
            |index| self.key_at(index),
            collection_eligible,
        );
        state.observed_keys.clone_from(&self.item_keys);
        state.observed_enabled = enabled;

        if state.collection.cursor().is_none() {
            let initial = if self.skip_disabled {
                self.items.iter().position(|item| !item.disabled)
            } else {
                self.items
                    .iter()
                    .position(|item| !item.disabled)
                    .or_else(|| (!self.items.is_empty()).then_some(0))
            };
            if let Some(index) = initial {
                self.set_cursor(state, index, self.key_at(index));
            }
        }

        if self.skip_disabled
            && let Some(key) = state.collection.cursor()
            && let Some(index) = self.index_of_key(key)
            && self.items.get(index).is_some_and(|item| item.disabled)
        {
            let replacement = self.nearest_eligible(index);
            if let Some(replacement) = replacement {
                self.set_cursor(state, replacement, self.key_at(replacement));
            } else {
                state.collection.clear_cursor();
            }
        }

        if let Some(layout) = layout
            && state.more_focused
        {
            if layout.more.is_some() {
                return;
            }
            state.more_focused = false;
        }

        if let Some(layout) = layout
            && let Some(key) = state.collection.cursor()
            && !layout.rows.iter().any(|row| row.key == key)
        {
            let from = state.collection.cursor_index();
            let replacement = self.nearest_visible_eligible(from, layout);
            if let Some(index) = replacement {
                self.set_cursor(state, index, self.key_at(index));
            } else if layout.more.is_some() {
                state.more_focused = true;
            } else {
                state.collection.clear_cursor();
            }
        } else if let Some(layout) = layout
            && state.collection.cursor().is_none()
            && !layout.rows.is_empty()
        {
            let replacement = self.nearest_visible_eligible(0, layout);
            if let Some(index) = replacement {
                self.set_cursor(state, index, self.key_at(index));
            } else if layout.more.is_some() {
                state.more_focused = true;
            }
        } else if let Some(layout) = layout
            && state.collection.cursor().is_none()
            && layout.rows.is_empty()
            && layout.more.is_some()
        {
            state.more_focused = true;
        }
    }

    fn nearest_eligible(&self, from: usize) -> Option<usize> {
        let eligible = |index: usize| {
            self.items
                .get(index)
                .is_some_and(|item| !self.skip_disabled || !item.disabled)
        };
        (from.min(self.items.len())..self.items.len())
            .find(|&index| eligible(index))
            .or_else(|| {
                (0..from.min(self.items.len()))
                    .rev()
                    .find(|&index| eligible(index))
            })
    }

    fn nearest_visible_eligible(&self, from: usize, layout: &ToolbarLayout) -> Option<usize> {
        self.nearest_visible_index(from, layout, |index| {
            self.items
                .get(index)
                .is_some_and(|item| !self.skip_disabled || !item.disabled)
        })
    }

    fn nearest_visible_index(
        &self,
        from: usize,
        layout: &ToolbarLayout,
        eligible: impl Fn(usize) -> bool,
    ) -> Option<usize> {
        layout
            .rows
            .iter()
            .find(|row| row.index >= from && eligible(row.index))
            .or_else(|| {
                layout
                    .rows
                    .iter()
                    .rev()
                    .find(|row| row.index < from && eligible(row.index))
            })
            .map(|row| row.index)
            .or_else(|| {
                (!self.skip_disabled)
                    .then(|| layout.rows.first().map(|row| row.index))
                    .flatten()
            })
    }

    fn display_cursor(&self, state: &ToolbarState, layout: &ToolbarLayout) -> Option<usize> {
        if state.more_focused {
            return None;
        }
        if let Some(key) = state.collection.cursor()
            && let Some(row) = layout.rows.iter().find(|row| row.key == key)
            && self
                .items
                .get(row.index)
                .is_some_and(|item| !self.skip_disabled || !item.disabled)
        {
            return Some(row.index);
        }
        self.nearest_visible_eligible(state.collection.cursor_index(), layout)
    }

    fn activate(
        &self,
        index: usize,
        pending_toggles: &mut [Option<bool>],
    ) -> Option<ToolbarActionEvent> {
        let item = self.items.get(index)?;
        if item.disabled {
            return None;
        }
        let key = self.item_keys.get(index).copied()?;
        let action = if item.on.is_some() {
            let value = pending_toggles.get_mut(index)?.as_mut()?;
            *value = !*value;
            ToolbarAction::Toggled(index, *value)
        } else {
            ToolbarAction::Activated(index)
        };
        Some(ToolbarActionEvent { key, action })
    }

    fn set_cursor(&self, state: &mut ToolbarState, index: usize, key: ItemKey) {
        state.more_focused = false;
        state.collection.set_cursor(index, key);
    }

    fn index_of_key(&self, key: ItemKey) -> Option<usize> {
        self.item_keys
            .iter()
            .position(|candidate| *candidate == key)
    }

    #[expect(
        clippy::expect_used,
        reason = "Toolbar constructors validate an exact key sidecar before it reaches reconciliation"
    )]
    fn key_at(&self, index: usize) -> ItemKey {
        *self
            .item_keys
            .get(index)
            .expect("validated Toolbar item-key sidecar matches items")
    }
}

impl Measure for Toolbar {
    fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        let height = if self.items.is_empty() { 0 } else { 1 };
        Size::exact(self.width(), height).fit(constraints)
    }
}

fn make_overflow_items<'a>(
    projected: &'a mut [ToolItem],
    on_marker: &'static str,
    off_marker: &'static str,
) -> Vec<MenuItem<'a>> {
    projected
        .iter_mut()
        .map(|item| {
            if let Some(on) = item.on {
                let marker = if on { on_marker } else { off_marker };
                item.label = format!("{marker} {}", item.label);
            }
            let mut row = MenuItem::new(ActionKey::CONFIRM, &item.label);
            if let Some(shortcut) = item.shortcut.as_deref() {
                row = row.hint(shortcut);
            }
            if item.separator_after {
                row = row.separator();
            }
            if item.disabled {
                row = match item.reason.as_deref() {
                    Some(reason) => row.reason(reason),
                    None => row.disabled(true),
                };
            }
            row
        })
        .collect()
}

struct VisibleStops<'a> {
    items: &'a [ToolItem],
    rows: &'a [LayoutRow],
    more: bool,
}

impl VisibleStops<'_> {
    fn is_empty(&self) -> bool {
        self.rows.is_empty() && !self.more
    }
}

impl RowSource for VisibleStops<'_> {
    fn len(&self) -> usize {
        self.rows.len().saturating_add(usize::from(self.more))
    }

    fn enabled(&self, position: usize) -> bool {
        self.rows.get(position).map_or(self.more, |row| {
            self.items.get(row.index).is_some_and(|item| !item.disabled)
        })
    }

    fn label(&self, position: usize) -> &str {
        self.rows
            .get(position)
            .and_then(|row| self.items.get(row.index))
            .map_or("More", |item| item.label.as_str())
    }
}
