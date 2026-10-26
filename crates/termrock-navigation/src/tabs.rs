use core::fmt;
use core::marker::PhantomData;
use core::time::Duration;

use ratatui_core::buffer::CellWidth;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Modifier, Style};

use super::{Acc, PartStyle, SlotFn, cell_at, first_row, paint_pressed_bracket};
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::list::{List, ListAction, ListState};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, FgStep, GlyphRole, Role, Slot, StyleDefaults, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation,
};
use termrock_controls::Panel;
use termrock_runtime::empty::Status;
use termrock_runtime::hit::{Axes, Headroom};
use termrock_runtime::layer::{
    Anchor, CrossAlign, Dismiss, LayerEvent, LayerOpening, LayerSize, LayerSpec, Side,
};
use termrock_runtime::rowui::{RowFn, RowUi};

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabNavProfile {
    Desktop,

    Vim,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabsAction {
    Activated(ItemKey),
    Close(ItemKey),
    New,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TabCloseReveal {
    #[default]
    Always,

    Hover,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabRule {
    Accent,

    Neutral,

    Auto,

    FocusOnly,
}

impl TabRule {
    fn foreground(
        self,
        ui: &Ui<'_>,
        owner: Id,
        focus_hidden: Option<bool>,
        passive: bool,
        passive_focused: bool,
    ) -> Role {
        match self {
            Self::Accent => Role::Accent,
            Self::Neutral => Role::BorderStrong,
            Self::Auto if !focus_hidden.unwrap_or_else(|| ui.is_covered_by_native_layer()) => {
                Role::Accent
            }
            Self::Auto => Role::BorderStrong,
            Self::FocusOnly if passive && passive_focused => Role::Accent,
            Self::FocusOnly if passive => Role::BorderStrong,
            Self::FocusOnly
                if !focus_hidden.unwrap_or_else(|| ui.is_covered_by_native_layer())
                    && ui.owner_has_focus_presentation(owner) =>
            {
                Role::Accent
            }
            Self::FocusOnly => Role::BorderStrong,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TabMarks {
    dirty: bool,
    busy: bool,
    error: bool,
    done: bool,
}

impl TabMarks {
    #[must_use]
    pub const fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    #[must_use]
    pub const fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    #[must_use]
    pub const fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    #[must_use]
    pub const fn done(mut self, done: bool) -> Self {
        self.done = done;
        self
    }

    const fn visible(self) -> Option<TabItemMark> {
        if self.busy {
            Some(TabItemMark::Busy)
        } else if self.error {
            Some(TabItemMark::Error)
        } else if self.dirty {
            Some(TabItemMark::Dirty)
        } else if self.done {
            Some(TabItemMark::Done)
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct TabAffixes {
    prefix: Option<String>,
    suffix: Option<String>,
    trailing_padding: u16,
    plate_separators: bool,
}

impl fmt::Debug for TabAffixes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("TabAffixes");
        debug
            .field("prefix", &self.prefix)
            .field("suffix", &self.suffix);
        if self.trailing_padding > 0 {
            debug.field("trailing_padding", &self.trailing_padding);
        }
        if self.plate_separators {
            debug.field("plate_separators", &self.plate_separators);
        }
        debug.finish()
    }
}

impl TabAffixes {
    #[must_use]
    pub fn prefix(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.prefix = (!text.is_empty()).then_some(text);
        self
    }

    #[must_use]
    pub fn suffix(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.suffix = (!text.is_empty()).then_some(text);
        self
    }

    #[must_use]
    pub fn trailing_padding(mut self, cells: u16) -> Self {
        self.trailing_padding = cells;
        self
    }

    #[must_use]
    pub fn plate_separators(mut self, enabled: bool) -> Self {
        self.plate_separators = enabled;
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TabItemMark {
    Busy,
    Error,
    Dirty,
    Done,
}

impl TabItemMark {
    const fn flags(self) -> StateFlags {
        match self {
            Self::Busy => StateFlags::BUSY,
            Self::Error => StateFlags::ERROR,
            Self::Dirty => StateFlags::DIRTY,
            Self::Done => StateFlags::CHECKED,
        }
    }

    const fn default_role(self) -> Role {
        match self {
            Self::Busy => Role::Accent,
            Self::Error => Role::Danger,
            Self::Dirty => Role::Warning,
            Self::Done => Role::Success,
        }
    }

    const fn default_glyph(self) -> Option<GlyphRole> {
        match self {
            Self::Busy => None,
            Self::Error => Some(GlyphRole::Error),
            Self::Dirty => Some(GlyphRole::Dirty),
            Self::Done => Some(GlyphRole::ProgressDone),
        }
    }
}

#[derive(Default)]
struct ItemSpinnerPhase {
    frame: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabsCmd {
    Prev,
    Next,
    Activate,
    Nth(u8),
    Close,
    New,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TabsMenuCmd {
    ShowHidden,
}

const SHOW_HIDDEN_ACTION: crate::ActionKey = crate::ActionKey::custom("tabs.show-hidden");
const SHOW_HIDDEN_BINDINGS: &[Binding<TabsMenuCmd>] = &[Binding {
    action: SHOW_HIDDEN_ACTION,
    chord: Some(Chord::key(KeyCode::Down)),
    cmd: TabsMenuCmd::ShowHidden,
    label: "Show hidden tabs",
    priority: 50,
    visible: true,
}];

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabsKeyBindingsError {
    NewGroupCommand {
        index: usize,
        found: TabsCmd,
    },

    CloseGroupCommand {
        index: usize,
        found: TabsCmd,
    },

    DuplicateAction {
        action: crate::ActionKey,
    },

    ConflictingChord {
        first: crate::ActionKey,
        second: crate::ActionKey,
    },
}

impl fmt::Display for TabsKeyBindingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NewGroupCommand { index, found } => {
                write!(f, "New binding {index} uses non-New command {found:?}")
            }
            Self::CloseGroupCommand { index, found } => {
                write!(f, "Close binding {index} uses non-Close command {found:?}")
            }
            Self::DuplicateAction { action } => {
                write!(f, "Tabs key bindings reuse action identity {action:?}")
            }
            Self::ConflictingChord { first, second } => write!(
                f,
                "Tabs actions {first:?} and {second:?} declare conflicting chords"
            ),
        }
    }
}

impl std::error::Error for TabsKeyBindingsError {}

const fn b(
    action: &'static str,
    chord: Chord,
    cmd: TabsCmd,
    label: &'static str,
    visible: bool,
) -> Binding<TabsCmd> {
    Binding {
        action: crate::ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority: if visible { 60 } else { 10 },
        visible,
    }
}

const BASE: [Binding<TabsCmd>; 15] = [
    b(
        "tabs.previous",
        Chord::key(KeyCode::Left),
        TabsCmd::Prev,
        "Prev tab",
        true,
    ),
    b(
        "tabs.next",
        Chord::key(KeyCode::Right),
        TabsCmd::Next,
        "Next tab",
        true,
    ),
    b(
        "tabs.previous-vim",
        Chord::key(KeyCode::Char('h')),
        TabsCmd::Prev,
        "Prev tab",
        false,
    ),
    b(
        "tabs.next-vim",
        Chord::key(KeyCode::Char('l')),
        TabsCmd::Next,
        "Next tab",
        false,
    ),
    b(
        "tabs.activate",
        Chord::key(KeyCode::Enter),
        TabsCmd::Activate,
        "Activate",
        false,
    ),
    b(
        "tabs.activate-space",
        Chord::key(KeyCode::Char(' ')),
        TabsCmd::Activate,
        "Activate",
        false,
    ),
    b(
        "tabs.tab-1",
        Chord::key(KeyCode::Char('1')),
        TabsCmd::Nth(1),
        "Tab 1",
        false,
    ),
    b(
        "tabs.tab-2",
        Chord::key(KeyCode::Char('2')),
        TabsCmd::Nth(2),
        "Tab 2",
        false,
    ),
    b(
        "tabs.tab-3",
        Chord::key(KeyCode::Char('3')),
        TabsCmd::Nth(3),
        "Tab 3",
        false,
    ),
    b(
        "tabs.tab-4",
        Chord::key(KeyCode::Char('4')),
        TabsCmd::Nth(4),
        "Tab 4",
        false,
    ),
    b(
        "tabs.tab-5",
        Chord::key(KeyCode::Char('5')),
        TabsCmd::Nth(5),
        "Tab 5",
        false,
    ),
    b(
        "tabs.tab-6",
        Chord::key(KeyCode::Char('6')),
        TabsCmd::Nth(6),
        "Tab 6",
        false,
    ),
    b(
        "tabs.tab-7",
        Chord::key(KeyCode::Char('7')),
        TabsCmd::Nth(7),
        "Tab 7",
        false,
    ),
    b(
        "tabs.tab-8",
        Chord::key(KeyCode::Char('8')),
        TabsCmd::Nth(8),
        "Tab 8",
        false,
    ),
    b(
        "tabs.tab-9",
        Chord::key(KeyCode::Char('9')),
        TabsCmd::Nth(9),
        "Tab 9",
        false,
    ),
];
const PREVIOUS_SHIFT_ARROW: Binding<TabsCmd> = b(
    "tabs.previous-shift-arrow",
    Chord::with(KeyCode::Left, KeyModifiers::SHIFT),
    TabsCmd::Prev,
    "Prev tab",
    false,
);
const NEXT_SHIFT_ARROW: Binding<TabsCmd> = b(
    "tabs.next-shift-arrow",
    Chord::with(KeyCode::Right, KeyModifiers::SHIFT),
    TabsCmd::Next,
    "Next tab",
    false,
);
const PREVIOUS_VIM_UPPER: Binding<TabsCmd> = b(
    "tabs.previous-vim-upper",
    Chord::key(KeyCode::Char('H')),
    TabsCmd::Prev,
    "Prev tab",
    false,
);
const NEXT_VIM_UPPER: Binding<TabsCmd> = b(
    "tabs.next-vim-upper",
    Chord::key(KeyCode::Char('L')),
    TabsCmd::Next,
    "Next tab",
    false,
);
const CLOSE_X: Binding<TabsCmd> = b(
    "tabs.close",
    Chord::key(KeyCode::Char('x')),
    TabsCmd::Close,
    "Close",
    true,
);
const CLOSE_DEL: Binding<TabsCmd> = b(
    "tabs.close-delete",
    Chord::key(KeyCode::Delete),
    TabsCmd::Close,
    "Close",
    false,
);
const NEW_N: Binding<TabsCmd> = b(
    "tabs.new",
    Chord::key(KeyCode::Char('n')),
    TabsCmd::New,
    "New",
    true,
);
#[derive(Clone, Copy)]
struct EffectiveTabsKeyGroups {
    base: &'static [Binding<TabsCmd>],
    new: &'static [Binding<TabsCmd>],
    close: &'static [Binding<TabsCmd>],
}

impl EffectiveTabsKeyGroups {
    const EMPTY: Self = Self {
        base: &[],
        new: &[],
        close: &[],
    };

    fn command(self, action: crate::ActionKey) -> Option<TabsCmd> {
        [self.base, self.new, self.close]
            .into_iter()
            .find_map(|table| Binding::command(table, action))
    }
}

macro_rules! table {
    ($name:ident, $n:expr, [$($extra:expr),*]) => {
        const $name: [Binding<TabsCmd>; $n] = [
            BASE[0], BASE[1], BASE[2], BASE[3], BASE[4], BASE[5], BASE[6], BASE[7], BASE[8],
            BASE[9], BASE[10], BASE[11], BASE[12], BASE[13], BASE[14], $($extra),*
        ];
    };
}
table!(PLAIN, 15, []);
table!(CLOSABLE, 17, [CLOSE_X, CLOSE_DEL]);
table!(NEWABLE, 16, [NEW_N]);
table!(FULL, 18, [CLOSE_X, CLOSE_DEL, NEW_N]);

macro_rules! digit_free_table {
    ($name:ident, $n:expr, [$($extra:expr),*]) => {
        const $name: [Binding<TabsCmd>; $n] = [
            BASE[0], BASE[1], BASE[2], BASE[3], BASE[4], BASE[5], $($extra),*
        ];
    };
}
digit_free_table!(PLAIN_DIGIT_FREE, 6, []);
digit_free_table!(CLOSABLE_DIGIT_FREE, 8, [CLOSE_X, CLOSE_DEL]);
digit_free_table!(NEWABLE_DIGIT_FREE, 7, [NEW_N]);
digit_free_table!(FULL_DIGIT_FREE, 9, [CLOSE_X, CLOSE_DEL, NEW_N]);

macro_rules! profile_tables {
    (
        $plain:ident,
        $closable:ident,
        $newable:ident,
        $full:ident,
        $plain_digit_free:ident,
        $closable_digit_free:ident,
        $newable_digit_free:ident,
        $full_digit_free:ident;
        full [$($full_base:expr),* $(,)?];
        digit_free [$($free_base:expr),* $(,)?]
    ) => {
        const $plain: &[Binding<TabsCmd>] = &[$($full_base),*];
        const $closable: &[Binding<TabsCmd>] = &[$($full_base),*, CLOSE_X, CLOSE_DEL];
        const $newable: &[Binding<TabsCmd>] = &[$($full_base),*, NEW_N];
        const $full: &[Binding<TabsCmd>] = &[$($full_base),*, CLOSE_X, CLOSE_DEL, NEW_N];
        const $plain_digit_free: &[Binding<TabsCmd>] = &[$($free_base),*];
        const $closable_digit_free: &[Binding<TabsCmd>] = &[$($free_base),*, CLOSE_X, CLOSE_DEL];
        const $newable_digit_free: &[Binding<TabsCmd>] = &[$($free_base),*, NEW_N];
        const $full_digit_free: &[Binding<TabsCmd>] = &[$($free_base),*, CLOSE_X, CLOSE_DEL, NEW_N];
    };
}

profile_tables!(
    DESKTOP_PLAIN,
    DESKTOP_CLOSABLE,
    DESKTOP_NEWABLE,
    DESKTOP_FULL,
    DESKTOP_PLAIN_DIGIT_FREE,
    DESKTOP_CLOSABLE_DIGIT_FREE,
    DESKTOP_NEWABLE_DIGIT_FREE,
    DESKTOP_FULL_DIGIT_FREE;
    full [
        BASE[0], BASE[1], PREVIOUS_SHIFT_ARROW, NEXT_SHIFT_ARROW,
        BASE[4], BASE[5], BASE[6], BASE[7], BASE[8], BASE[9], BASE[10], BASE[11], BASE[12],
        BASE[13], BASE[14]
    ];
    digit_free [
        BASE[0], BASE[1], PREVIOUS_SHIFT_ARROW, NEXT_SHIFT_ARROW, BASE[4], BASE[5]
    ]
);
profile_tables!(
    VIM_PLAIN,
    VIM_CLOSABLE,
    VIM_NEWABLE,
    VIM_FULL,
    VIM_PLAIN_DIGIT_FREE,
    VIM_CLOSABLE_DIGIT_FREE,
    VIM_NEWABLE_DIGIT_FREE,
    VIM_FULL_DIGIT_FREE;
    full [
        BASE[0], BASE[1], BASE[2], BASE[3], PREVIOUS_VIM_UPPER, NEXT_VIM_UPPER,
        PREVIOUS_SHIFT_ARROW, NEXT_SHIFT_ARROW, BASE[4], BASE[5], BASE[6], BASE[7], BASE[8],
        BASE[9], BASE[10], BASE[11], BASE[12], BASE[13], BASE[14]
    ];
    digit_free [
        BASE[0], BASE[1], BASE[2], BASE[3], PREVIOUS_VIM_UPPER, NEXT_VIM_UPPER,
        PREVIOUS_SHIFT_ARROW, NEXT_SHIFT_ARROW, BASE[4], BASE[5]
    ]
);

#[derive(Clone, Copy)]
struct TabsProfileTables {
    plain: &'static [Binding<TabsCmd>],
    closable: &'static [Binding<TabsCmd>],
    newable: &'static [Binding<TabsCmd>],
    full: &'static [Binding<TabsCmd>],
    plain_digit_free: &'static [Binding<TabsCmd>],
    closable_digit_free: &'static [Binding<TabsCmd>],
    newable_digit_free: &'static [Binding<TabsCmd>],
    full_digit_free: &'static [Binding<TabsCmd>],
}

impl TabsProfileTables {
    const fn table(
        self,
        closable: bool,
        allow_new: bool,
        digit_jump: bool,
    ) -> &'static [Binding<TabsCmd>] {
        match (digit_jump, closable, allow_new) {
            (true, false, false) => self.plain,
            (true, true, false) => self.closable,
            (true, false, true) => self.newable,
            (true, true, true) => self.full,
            (false, false, false) => self.plain_digit_free,
            (false, true, false) => self.closable_digit_free,
            (false, false, true) => self.newable_digit_free,
            (false, true, true) => self.full_digit_free,
        }
    }
}

fn profile_tables(profile: TabNavProfile) -> TabsProfileTables {
    match profile {
        TabNavProfile::Desktop => TabsProfileTables {
            plain: DESKTOP_PLAIN,
            closable: DESKTOP_CLOSABLE,
            newable: DESKTOP_NEWABLE,
            full: DESKTOP_FULL,
            plain_digit_free: DESKTOP_PLAIN_DIGIT_FREE,
            closable_digit_free: DESKTOP_CLOSABLE_DIGIT_FREE,
            newable_digit_free: DESKTOP_NEWABLE_DIGIT_FREE,
            full_digit_free: DESKTOP_FULL_DIGIT_FREE,
        },
        TabNavProfile::Vim => TabsProfileTables {
            plain: VIM_PLAIN,
            closable: VIM_CLOSABLE,
            newable: VIM_NEWABLE,
            full: VIM_FULL,
            plain_digit_free: VIM_PLAIN_DIGIT_FREE,
            closable_digit_free: VIM_CLOSABLE_DIGIT_FREE,
            newable_digit_free: VIM_NEWABLE_DIGIT_FREE,
            full_digit_free: VIM_FULL_DIGIT_FREE,
        },
    }
}

#[derive(Clone, Default)]
pub struct TabsState {
    core: CollectionCore,
    active: Option<ItemKey>,
    first: Option<ItemKey>,
    first_index: usize,

    close_reveal_press: Option<ItemKey>,

    manual_window_active: Option<Option<ItemKey>>,
    menu: Option<TabsOpenMenu>,
}

impl PartialEq for TabsState {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core
            && self.active == other.active
            && self.first == other.first
            && self.first_index == other.first_index
            && self.close_reveal_press == other.close_reveal_press
            && self.manual_window_active == other.manual_window_active
            && self.menu == other.menu
    }
}

impl Eq for TabsState {}

impl fmt::Debug for TabsState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TabsState")
            .field("core", &self.core)
            .field("active", &self.active)
            .field("first", &self.first)
            .field("first_index", &self.first_index)
            .field("close_reveal_press", &self.close_reveal_press)
            .finish()
    }
}

impl TabsState {
    pub const fn active(&self) -> Option<ItemKey> {
        self.active
    }

    pub const fn cursor(&self) -> Option<ItemKey> {
        self.core.cursor()
    }

    pub const fn first(&self) -> Option<ItemKey> {
        self.first
    }

    pub fn set_active(&mut self, index: usize, key: ItemKey) {
        let next = Some(key);
        if self.active != next {
            self.manual_window_active = None;
        }
        self.active = next;
        self.core.set_cursor(index, key);
    }
}

impl Reconcile for TabsState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
        let previous_active = self.active;
        let r = self.core.reconcile(len, &key);
        if let Some(a) = self.active
            && !(0..len).any(|i| key(i) == a)
        {
            self.active = self.core.cursor();
        }
        match self.first {
            Some(f) => {
                if let Some(i) = (0..len).find(|&i| key(i) == f) {
                    self.first_index = i;
                } else {
                    let i = self.first_index.min(len.saturating_sub(1));
                    self.first_index = i;
                    self.first = (len > 0).then(|| key(i));
                }
            }
            None => self.first_index = 0,
        }
        if self.active != previous_active {
            self.manual_window_active = None;
        }
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
        self.close_reveal_press = None;
        self.manual_window_active = None;
        self.menu = None;
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct TabsMenuItem {
    key: ItemKey,
    label_segments: Vec<String>,
}

impl fmt::Display for TabsMenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for segment in &self.label_segments {
            f.write_str(segment)?;
        }
        Ok(())
    }
}

fn tabs_menu_key(item: &TabsMenuItem) -> ItemKey {
    item.key
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct TabsOpenMenu {
    opening: LayerOpening,
    items: Vec<TabsMenuItem>,
    list: ListState,
    more_area: Rect,
    content_len: usize,
    first_key: Option<ItemKey>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct TabsMenuFrame {
    eligible: bool,
    planned_whole_rows: bool,
    owner: Option<Id>,
    area: Rect,
    visible_area: Rect,
    more_area: Rect,
    visible_more_area: Rect,
    content_len: usize,
    viewport_len: usize,
    rows: u16,
    cols: u16,
    first_index: usize,
    first_key: Option<ItemKey>,
    last_key: Option<ItemKey>,
    items: Vec<TabsMenuItem>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum TabsWheelFrameKind {
    #[default]
    Unknown,
    Legacy,
    Planned,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct TabsWheelFrame {
    kind: TabsWheelFrameKind,
    eligible: bool,
    planned_whole_rows: bool,
    active: Option<ItemKey>,
    manual_window_active: Option<Option<ItemKey>>,
    content_len: usize,
    viewport_len: usize,
    rows: u16,
    cols: u16,
    first_index: usize,
    first_key: Option<ItemKey>,
    geometry_follow: Option<TabsGeometryFollow>,
    last_key: Option<ItemKey>,
    terminal_first_index: Option<usize>,
    area: Rect,
    visible_area: Rect,
    visible_row_area: Rect,
}

impl Default for TabsWheelFrame {
    fn default() -> Self {
        Self {
            kind: TabsWheelFrameKind::Unknown,
            eligible: false,
            planned_whole_rows: false,
            active: None,
            manual_window_active: None,
            content_len: 0,
            viewport_len: 0,
            rows: 0,
            cols: 0,
            first_index: 0,
            first_key: None,
            geometry_follow: None,
            last_key: None,
            terminal_first_index: None,
            area: Rect::new(0, 0, 0, 0),
            visible_area: Rect::new(0, 0, 0, 0),
            visible_row_area: Rect::new(0, 0, 0, 0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct TabsGeometryFollow {
    active_key: ItemKey,
    first_key: ItemKey,
    first_index: usize,
}

fn manual_window_owns_follow(
    frame_kind: TabsWheelFrameKind,
    manual_window_active: Option<Option<ItemKey>>,
    active: Option<ItemKey>,
) -> bool {
    frame_kind != TabsWheelFrameKind::Legacy && manual_window_active == Some(active)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TabWindowReceipt {
    pub viewport_len: usize,
    pub content_len: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct AffixSegment {
    index: usize,
    width: u16,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct OwnedTabAffixes {
    display_start: usize,
    display_end: usize,
    display_width: u16,
    prefix: Option<AffixSegment>,
    suffix: Option<AffixSegment>,
    trailing_padding: u16,
    plate_separators: bool,
}

impl OwnedTabAffixes {
    fn has_text_affixes(&self) -> bool {
        self.prefix.is_some() || self.suffix.is_some()
    }

    fn is_consistent(&self, label_segments: &[String]) -> bool {
        if self.display_start > self.display_end || self.display_end > label_segments.len() {
            return false;
        }
        let prefix_matches = match self.prefix {
            Some(prefix) => {
                prefix.index.saturating_add(2) == self.display_start
                    && label_segments.get(prefix.index).is_some()
                    && label_segments
                        .get(prefix.index.saturating_add(1))
                        .is_some_and(|gap| gap == " ")
            }
            None => self.display_start == 0,
        };
        let suffix_matches = match self.suffix {
            Some(suffix) => {
                suffix.index == self.display_end.saturating_add(1)
                    && label_segments.get(suffix.index).is_some()
                    && label_segments
                        .get(self.display_end)
                        .is_some_and(|gap| gap == " ")
            }
            None => self.display_end == label_segments.len(),
        };
        prefix_matches && suffix_matches
    }
}

#[derive(Clone)]
pub struct TabRowLayout {
    pub key: ItemKey,
    pub index: usize,
    pub logical_content: Rect,
    pub visible_content: Rect,
    pub logical_tab: Rect,
    pub visible_tab: Rect,

    pub logical_close: Option<Rect>,

    pub visible_close: Option<Rect>,
    logical_mark: Option<Rect>,
    visible_mark: Option<Rect>,
    mark: Option<TabItemMark>,
    close_enabled: bool,
    label_segments: Vec<String>,
    affixes: Option<OwnedTabAffixes>,
    pub label_extent: u16,
}

impl PartialEq for TabRowLayout {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.index == other.index
            && self.logical_content == other.logical_content
            && self.visible_content == other.visible_content
            && self.logical_tab == other.logical_tab
            && self.visible_tab == other.visible_tab
            && self.logical_close == other.logical_close
            && self.visible_close == other.visible_close
            && self.logical_mark == other.logical_mark
            && self.visible_mark == other.visible_mark
            && self.mark == other.mark
            && self.label_segments == other.label_segments
            && self.affixes == other.affixes
            && self.label_extent == other.label_extent
    }
}

impl Eq for TabRowLayout {}

impl fmt::Debug for TabRowLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TabRowLayout")
            .field("key", &self.key)
            .field("index", &self.index)
            .field("logical_content", &self.logical_content)
            .field("visible_content", &self.visible_content)
            .field("logical_tab", &self.logical_tab)
            .field("visible_tab", &self.visible_tab)
            .field("logical_close", &self.logical_close)
            .field("visible_close", &self.visible_close)
            .field("logical_mark", &self.logical_mark)
            .field("visible_mark", &self.visible_mark)
            .field("mark", &self.mark)
            .field("label_segments", &self.label_segments)
            .field("label_extent", &self.label_extent)
            .finish()
    }
}

impl TabRowLayout {
    fn affix_bounds(&self, part: Part) -> Option<(Rect, Rect)> {
        let affixes = self.affixes.as_ref()?;
        let (x, width) = match part {
            Part::PREFIX => {
                let prefix = affixes.prefix?;
                (self.logical_content.x, prefix.width.saturating_add(1))
            }
            Part::META => {
                let suffix = affixes.suffix?;
                let prefix_width = affixes
                    .prefix
                    .map_or(0, |prefix| prefix.width.saturating_add(1));
                (
                    self.logical_content
                        .x
                        .saturating_add(prefix_width)
                        .saturating_add(affixes.display_width),
                    suffix.width.saturating_add(1),
                )
            }
            _ => return None,
        };
        let logical = Rect {
            x,
            y: self.logical_content.y,
            width,
            height: 1,
        }
        .intersection(self.logical_content);
        Some((logical, logical.intersection(self.visible_content)))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct OwnedTabMeasurement {
    key: ItemKey,
    mark: Option<TabItemMark>,
    close_enabled: bool,
    label_segments: Vec<String>,
    natural_label_extent: Option<u64>,
    affixes: Option<OwnedTabAffixes>,
}

impl OwnedTabMeasurement {
    fn new_with_natural_extent(
        key: ItemKey,
        mark: Option<TabItemMark>,
        close_enabled: bool,
        label_segments: Vec<String>,
        affixes: Option<OwnedTabAffixes>,
        measure_natural_extent: bool,
    ) -> Self {
        let natural_label_extent =
            measure_natural_extent.then(|| natural_semantic_label_extent(&label_segments));
        Self {
            key,
            mark,
            close_enabled,
            label_segments,
            natural_label_extent,
            affixes,
        }
    }

    fn natural_label_extent(&self) -> u64 {
        self.natural_label_extent
            .unwrap_or_else(|| natural_semantic_label_extent(&self.label_segments))
    }
}

#[derive(Clone, Copy)]
struct OwnedTabsWalk<'a> {
    area: Rect,
    clip: Rect,
    items: &'a [OwnedTabMeasurement],
    previous: Option<TabWindowReceipt>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct DefaultRowGeometry {
    key: ItemKey,
    index: usize,
    logical_content: Rect,
    visible_content: Rect,
    logical_tab: Rect,
    visible_tab: Rect,
    logical_close: Option<Rect>,
    visible_close: Option<Rect>,
    logical_mark: Option<Rect>,
    visible_mark: Option<Rect>,
    mark: Option<TabItemMark>,
    close_enabled: bool,
    label_extent: u16,
}

impl DefaultRowGeometry {
    fn into_tab_row(
        self,
        label_segments: Vec<String>,
        affixes: Option<OwnedTabAffixes>,
    ) -> TabRowLayout {
        TabRowLayout {
            key: self.key,
            index: self.index,
            logical_content: self.logical_content,
            visible_content: self.visible_content,
            logical_tab: self.logical_tab,
            visible_tab: self.visible_tab,
            logical_close: self.logical_close,
            visible_close: self.visible_close,
            logical_mark: self.logical_mark,
            visible_mark: self.visible_mark,
            mark: self.mark,
            close_enabled: self.close_enabled,
            label_segments,
            affixes,
            label_extent: self.label_extent,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct DefaultRowHeader {
    row_area: Rect,
    used_area: Rect,
    rule_area: Option<Rect>,
    first_tab_x: u16,
    right_limit: u16,
    left_overflow: Option<Rect>,
    right_overflow_lane: Option<Rect>,
    more_area: Option<Rect>,
    status_w: u16,
    new_w: u16,
}

impl DefaultRowHeader {
    fn available_width(self) -> u16 {
        self.right_limit.saturating_sub(self.first_tab_x)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct DefaultRowWalk {
    row_area: Rect,
    used_area: Rect,
    rule_area: Option<Rect>,
    first_tab_x: u16,
    right_limit: u16,
    left_overflow: Option<Rect>,
    right_overflow: Option<Rect>,
    more_area: Option<Rect>,
    new_area: Option<Rect>,
    status_area: Option<Rect>,
    hidden_right: usize,
    viewport_len: usize,
    last_row: Option<DefaultRowGeometry>,
}

fn max_rows_for_width_budget(available: u16) -> usize {
    ((u32::from(available).saturating_add(1)) / 4) as usize
}

fn positive_terminal_candidate_range(len: usize, max_rows: usize) -> core::ops::Range<usize> {
    let first = if len == 0 {
        0
    } else {
        len.saturating_sub(max_rows).max(1)
    };
    first..len
}

#[derive(Clone)]
pub struct TabLayout {
    id: Id,
    area: Rect,
    clip: Rect,
    row_area: Rect,
    used_area: Rect,
    rule_area: Option<Rect>,
    rows: Vec<TabRowLayout>,

    owned_items: Option<Vec<OwnedTabMeasurement>>,

    terminal_first_index: Option<usize>,
    geometry_follow: Option<TabsGeometryFollow>,
    planned_whole_rows: bool,
    manual_window_active: Option<Option<ItemKey>>,
    first_index: usize,
    active: Option<ItemKey>,
    cursor: Option<ItemKey>,
    first: Option<ItemKey>,
    first_tab_x: u16,
    overflow_menu: bool,
    menu: Option<TabsOpenMenu>,
    allow_new: bool,
    new_key_bindings: Option<&'static [Binding<TabsCmd>]>,
    closable: bool,
    closable_keys: Option<Vec<ItemKey>>,
    close_key_bindings: Option<&'static [Binding<TabsCmd>]>,
    close_reveal: TabCloseReveal,
    close_target_w: Option<u16>,
    close_danger_hover: bool,
    close_reveal_press: Option<ItemKey>,
    status: Status,
    rule: Option<TabRule>,
    rule_focus_hidden: Option<bool>,
    passive: bool,
    passive_focused: bool,
    left_overflow: Option<Rect>,
    right_overflow: Option<Rect>,
    more_area: Option<Rect>,
    new_area: Option<Rect>,
    status_area: Option<Rect>,
    item_marks_enabled: bool,
    item_affixes_enabled: bool,
    right_limit: u16,
    hidden_right: usize,
    pub logical_area: Rect,
    pub visible_area: Rect,
    pub viewport_len: usize,
    pub content_len: usize,
    pub overflow_reserved: bool,
}

impl PartialEq for TabLayout {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.area == other.area
            && self.clip == other.clip
            && self.row_area == other.row_area
            && self.used_area == other.used_area
            && self.rule_area == other.rule_area
            && self.rows == other.rows
            && self.planned_whole_rows == other.planned_whole_rows
            && self.first_index == other.first_index
            && self.active == other.active
            && self.cursor == other.cursor
            && self.first == other.first
            && self.first_tab_x == other.first_tab_x
            && self.overflow_menu == other.overflow_menu
            && self.menu == other.menu
            && self.allow_new == other.allow_new
            && self.new_key_bindings == other.new_key_bindings
            && self.closable == other.closable
            && self.closable_keys == other.closable_keys
            && self.close_key_bindings == other.close_key_bindings
            && self.close_reveal == other.close_reveal
            && self.close_target_w == other.close_target_w
            && self.close_danger_hover == other.close_danger_hover
            && self.close_reveal_press == other.close_reveal_press
            && self.status == other.status
            && self.rule == other.rule
            && self.rule_focus_hidden == other.rule_focus_hidden
            && self.passive == other.passive
            && self.passive_focused == other.passive_focused
            && self.left_overflow == other.left_overflow
            && self.right_overflow == other.right_overflow
            && self.more_area == other.more_area
            && self.new_area == other.new_area
            && self.status_area == other.status_area
            && self.item_marks_enabled == other.item_marks_enabled
            && self.item_affixes_enabled == other.item_affixes_enabled
            && self.right_limit == other.right_limit
            && self.hidden_right == other.hidden_right
            && self.logical_area == other.logical_area
            && self.visible_area == other.visible_area
            && self.viewport_len == other.viewport_len
            && self.content_len == other.content_len
            && self.overflow_reserved == other.overflow_reserved
    }
}

impl Eq for TabLayout {}

impl fmt::Debug for TabLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TabLayout")
            .field("id", &self.id)
            .field("area", &self.area)
            .field("clip", &self.clip)
            .field("row_area", &self.row_area)
            .field("used_area", &self.used_area)
            .field("rule_area", &self.rule_area)
            .field("rows", &self.rows)
            .field("first_index", &self.first_index)
            .field("active", &self.active)
            .field("cursor", &self.cursor)
            .field("first", &self.first)
            .field("first_tab_x", &self.first_tab_x)
            .field("allow_new", &self.allow_new)
            .field("new_key_bindings", &self.new_key_bindings)
            .field("closable", &self.closable)
            .field("closable_keys", &self.closable_keys)
            .field("close_key_bindings", &self.close_key_bindings)
            .field("close_reveal", &self.close_reveal)
            .field("close_target_w", &self.close_target_w)
            .field("close_danger_hover", &self.close_danger_hover)
            .field("close_reveal_press", &self.close_reveal_press)
            .field("status", &self.status)
            .field("rule", &self.rule)
            .field("rule_focus_hidden", &self.rule_focus_hidden)
            .field("passive", &self.passive)
            .field("passive_focused", &self.passive_focused)
            .field("left_overflow", &self.left_overflow)
            .field("right_overflow", &self.right_overflow)
            .field("new_area", &self.new_area)
            .field("status_area", &self.status_area)
            .field("item_marks_enabled", &self.item_marks_enabled)
            .field("right_limit", &self.right_limit)
            .field("hidden_right", &self.hidden_right)
            .field("logical_area", &self.logical_area)
            .field("visible_area", &self.visible_area)
            .field("viewport_len", &self.viewport_len)
            .field("content_len", &self.content_len)
            .field("overflow_reserved", &self.overflow_reserved)
            .finish()
    }
}

impl TabLayout {
    pub const fn area(&self) -> Rect {
        self.area
    }

    pub const fn clip(&self) -> Rect {
        self.clip
    }

    pub fn rows(&self) -> &[TabRowLayout] {
        &self.rows
    }

    pub const fn active_key(&self) -> Option<ItemKey> {
        self.active
    }

    pub const fn cursor_key(&self) -> Option<ItemKey> {
        self.cursor
    }

    pub const fn first_key(&self) -> Option<ItemKey> {
        self.first
    }

    pub const fn first_index(&self) -> usize {
        self.first_index
    }

    pub const fn hidden_right(&self) -> usize {
        self.hidden_right
    }

    fn hidden_menu_items(&self) -> Vec<TabsMenuItem> {
        let Some(items) = self.owned_items.as_ref() else {
            return Vec::new();
        };
        let visible_end = self.first_index.saturating_add(self.viewport_len);
        items
            .iter()
            .enumerate()
            .filter(|(index, _)| *index < self.first_index || *index >= visible_end)
            .map(|(_, item)| TabsMenuItem {
                key: item.key,
                label_segments: item.label_segments.clone(),
            })
            .collect()
    }

    pub const fn used_area(&self) -> Rect {
        self.used_area
    }

    pub fn part_bounds(&self, part: PartRef) -> Option<(Rect, Rect)> {
        match part.part {
            Part::TAB => self
                .rows
                .iter()
                .find(|row| Some(row.key) == part.item)
                .map(|row| (row.logical_tab, row.visible_tab)),
            Part::PREFIX | Part::META => self
                .rows
                .iter()
                .find(|row| Some(row.key) == part.item)
                .and_then(|row| row.affix_bounds(part.part)),
            Part::CLOSE => self
                .rows
                .iter()
                .find(|row| Some(row.key) == part.item)
                .and_then(|row| Some((row.logical_close?, row.visible_close?))),
            Part::OVERFLOW => match part.item {
                Some(ItemKey::Index(0)) => self
                    .left_overflow
                    .map(|area| (area, area.intersection(self.clip))),
                Some(ItemKey::Index(1)) => self
                    .right_overflow
                    .map(|area| (area, area.intersection(self.clip))),
                Some(ItemKey::Index(2)) => self
                    .more_area
                    .map(|area| (area, area.intersection(self.clip))),
                _ => None,
            },
            Part::NEW if part.item.is_none() => self
                .new_area
                .map(|area| (area, area.intersection(self.clip))),
            Part::ICON if part.item.is_none() => self
                .status_area
                .map(|area| (area, area.intersection(self.clip))),
            Part::ICON => self
                .rows
                .iter()
                .find(|row| Some(row.key) == part.item)
                .and_then(|row| Some((row.logical_mark?, row.visible_mark?))),
            _ => None,
        }
    }

    pub const fn window_receipt(&self) -> Option<TabWindowReceipt> {
        if self.area.is_empty() {
            return None;
        }
        Some(TabWindowReceipt {
            viewport_len: self.viewport_len,
            content_len: self.content_len,
        })
    }

    fn owned_snapshot_is_consistent(&self) -> bool {
        let Some(owned_items) = self.owned_items.as_ref() else {
            return self.area.is_empty()
                && self.rows.is_empty()
                && self.terminal_first_index.is_none()
                && self.geometry_follow.is_none();
        };
        if self.area.is_empty()
            || owned_items.len() != self.content_len
            || self
                .terminal_first_index
                .is_some_and(|first| first >= self.content_len)
        {
            return false;
        }
        let rows_match = self.rows.iter().all(|row| {
            owned_items.get(row.index).is_some_and(|item| {
                item.key == row.key
                    && item.mark == row.mark
                    && item.close_enabled == row.close_enabled
                    && item.label_segments == row.label_segments
                    && item.affixes == row.affixes
                    && item
                        .affixes
                        .as_ref()
                        .is_none_or(|affixes| affixes.is_consistent(&item.label_segments))
            })
        });
        let geometry_follow_matches = self.geometry_follow.is_none_or(|follow| {
            self.active == Some(follow.active_key)
                && self.first_index == follow.first_index
                && owned_items
                    .get(follow.first_index)
                    .is_some_and(|item| item.key == follow.first_key)
                && self
                    .rows
                    .iter()
                    .any(|row| row.key == follow.active_key && !row.visible_tab.is_empty())
        });
        let more_matches = self.more_area.is_none_or(|more| {
            self.overflow_menu
                && self.first_index.saturating_add(self.hidden_right) > 0
                && more.width == 4
                && more.height == 1
                && more.intersection(self.row_area) == more
                && [
                    self.left_overflow,
                    self.right_overflow,
                    self.new_area,
                    self.status_area,
                ]
                .into_iter()
                .flatten()
                .all(|area| more.intersection(area).is_empty())
                && self
                    .rows
                    .iter()
                    .all(|row| more.intersection(row.logical_tab).is_empty())
        });
        rows_match && geometry_follow_matches && more_matches
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TabLayoutMismatch;

struct SegmentWriter(Vec<String>);

impl fmt::Write for SegmentWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0.push(value.to_owned());
        Ok(())
    }
}

struct DisplaySegments<'a>(&'a [String]);

impl fmt::Display for DisplaySegments<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for segment in self.0 {
            f.write_str(segment)?;
        }
        Ok(())
    }
}

fn capture_label_affixes(
    display_segments: Vec<String>,
    affixes: TabAffixes,
) -> (Vec<String>, Option<OwnedTabAffixes>) {
    let prefix = affixes.prefix.and_then(|text| {
        let width = crate::text::width(&text);
        (width > 0).then_some((text, width))
    });
    let suffix = affixes.suffix.and_then(|text| {
        let width = crate::text::width(&text);
        (width > 0).then_some((text, width))
    });
    if prefix.is_none()
        && suffix.is_none()
        && affixes.trailing_padding == 0
        && !affixes.plate_separators
    {
        return (display_segments, None);
    }
    let trailing_padding = affixes.trailing_padding;
    let plate_separators = affixes.plate_separators;
    let display_width = display_segments.iter().fold(0_u16, |width, segment| {
        width.saturating_add(crate::text::width(segment))
    });

    let extra_segments = usize::from(prefix.is_some()) * 2 + usize::from(suffix.is_some()) * 2;
    let mut label_segments =
        Vec::with_capacity(display_segments.len().saturating_add(extra_segments));
    let prefix = prefix.map(|(text, width)| {
        let segment = AffixSegment {
            index: label_segments.len(),
            width,
        };
        label_segments.push(text);
        label_segments.push(" ".to_owned());
        segment
    });
    let display_start = label_segments.len();
    label_segments.extend(display_segments);
    let display_end = label_segments.len();
    let suffix = suffix.map(|(text, width)| {
        label_segments.push(" ".to_owned());
        let segment = AffixSegment {
            index: label_segments.len(),
            width,
        };
        label_segments.push(text);
        segment
    });

    (
        label_segments,
        Some(OwnedTabAffixes {
            display_start,
            display_end,
            display_width,
            prefix,
            suffix,
            trailing_padding,
            plate_separators,
        }),
    )
}

enum TabsRenderSource<'a, T> {
    Legacy {
        state: &'a TabsState,
        items: &'a [T],
    },
    Planned(&'a TabLayout),
}

impl<T> Copy for TabsRenderSource<'_, T> {}

impl<T> Clone for TabsRenderSource<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

enum TabsRowSource<'a, T> {
    Legacy(&'a T),
    Planned(&'a TabRowLayout),
}

impl<T> Copy for TabsRowSource<'_, T> {}

impl<T> Clone for TabsRowSource<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct Tabs<'a, T, K = ByIndex, R = DefaultRow> {
    id: Id,
    key: K,
    row: R,
    nav_profile: Option<TabNavProfile>,
    allow_new: bool,
    new_key_bindings: Option<&'static [Binding<TabsCmd>]>,
    digit_jump: bool,
    wrap: bool,
    planned_whole_rows: bool,
    closable: bool,
    closable_keys: Option<&'a [ItemKey]>,
    close_key_bindings: Option<&'static [Binding<TabsCmd>]>,
    middle_click_closes: bool,
    close_reveal: TabCloseReveal,
    close_target_w: Option<u16>,
    close_danger_hover: bool,
    status: Status,
    rule: Option<TabRule>,
    rule_focus_hidden: Option<bool>,
    passive: bool,
    passive_focused: bool,
    overflow_menu: bool,
    item_marks: Option<fn(&T) -> TabMarks>,
    item_affixes: Option<fn(&T) -> TabAffixes>,
    ov: PartStyle<'a>,
    _t: PhantomData<fn(&T)>,
}

impl<T, K, R> fmt::Debug for Tabs<'_, T, K, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tabs")
            .field("id", &self.id)
            .field("nav_profile", &self.nav_profile)
            .field("allow_new", &self.allow_new)
            .field("new_key_bindings", &self.new_key_bindings)
            .field("digit_jump", &self.digit_jump)
            .field("wrap", &self.wrap)
            .field("closable", &self.closable)
            .field("closable_keys", &self.closable_keys)
            .field("close_key_bindings", &self.close_key_bindings)
            .field("middle_click_closes", &self.middle_click_closes)
            .field("close_reveal", &self.close_reveal)
            .field("close_target_w", &self.close_target_w)
            .field("close_danger_hover", &self.close_danger_hover)
            .field("status", &self.status)
            .field("rule", &self.rule)
            .field("rule_focus_hidden", &self.rule_focus_hidden)
            .field("passive", &self.passive)
            .field("passive_focused", &self.passive_focused)
            .field("item_marks", &self.item_marks.is_some())
            .finish_non_exhaustive()
    }
}

impl<T> Tabs<'_, T, ByIndex, DefaultRow> {
    pub const fn new(id: Id) -> Self {
        Tabs {
            id,
            key: ByIndex,
            row: DefaultRow,
            nav_profile: None,
            allow_new: false,
            new_key_bindings: None,
            digit_jump: true,
            wrap: false,
            planned_whole_rows: false,
            closable: false,
            closable_keys: None,
            close_key_bindings: None,
            middle_click_closes: true,
            close_reveal: TabCloseReveal::Always,
            close_target_w: None,
            close_danger_hover: false,
            status: Status::Ready,
            rule: None,
            rule_focus_hidden: None,
            passive: false,
            passive_focused: false,
            overflow_menu: true,
            item_marks: None,
            item_affixes: None,
            ov: PartStyle::new(),
            _t: PhantomData,
        }
    }
}

impl<'a, T, K, R> Tabs<'a, T, K, R> {
    pub const PARTS: &'static [Part] = &[
        Part::CONTAINER,
        Part::TAB,
        Part::LABEL,
        Part::PREFIX,
        Part::META,
        Part::CLOSE,
        Part::NEW,
        Part::RULE,
        Part::OVERFLOW,
        Part::BADGE,
        Part::ICON,
    ];

    pub const fn id(&self) -> Id {
        self.id
    }

    pub fn try_nav_profile(
        mut self,
        profile: Option<TabNavProfile>,
    ) -> Result<Self, TabsKeyBindingsError> {
        validate_tabs_key_bindings(profile, self.new_key_bindings, self.close_key_bindings)?;
        self.nav_profile = profile;
        Ok(self)
    }

    pub fn key<K2: Fn(&T) -> ItemKey>(self, k: K2) -> Tabs<'a, T, K2, R> {
        Tabs {
            id: self.id,
            key: k,
            row: self.row,
            nav_profile: self.nav_profile,
            allow_new: self.allow_new,
            new_key_bindings: self.new_key_bindings,
            digit_jump: self.digit_jump,
            wrap: self.wrap,
            planned_whole_rows: self.planned_whole_rows,
            closable: self.closable,
            closable_keys: self.closable_keys,
            close_key_bindings: self.close_key_bindings,
            middle_click_closes: self.middle_click_closes,
            close_reveal: self.close_reveal,
            close_target_w: self.close_target_w,
            close_danger_hover: self.close_danger_hover,
            status: self.status,
            rule: self.rule,
            rule_focus_hidden: self.rule_focus_hidden,
            passive: self.passive,
            passive_focused: self.passive_focused,
            overflow_menu: self.overflow_menu,
            item_marks: self.item_marks,
            item_affixes: self.item_affixes,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    pub fn row<R2: Fn(&T, &mut RowUi<'_>)>(self, r: R2) -> Tabs<'a, T, K, R2> {
        Tabs {
            id: self.id,
            key: self.key,
            row: r,
            nav_profile: self.nav_profile,
            allow_new: self.allow_new,
            new_key_bindings: self.new_key_bindings,
            digit_jump: self.digit_jump,
            wrap: self.wrap,
            planned_whole_rows: self.planned_whole_rows,
            closable: self.closable,
            closable_keys: self.closable_keys,
            close_key_bindings: self.close_key_bindings,
            middle_click_closes: self.middle_click_closes,
            close_reveal: self.close_reveal,
            close_target_w: self.close_target_w,
            close_danger_hover: self.close_danger_hover,
            status: self.status,
            rule: self.rule,
            rule_focus_hidden: self.rule_focus_hidden,
            passive: self.passive,
            passive_focused: self.passive_focused,
            overflow_menu: self.overflow_menu,
            item_marks: self.item_marks,
            item_affixes: self.item_affixes,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub fn allow_new(mut self, yes: bool) -> Self {
        self.allow_new = yes;
        self
    }

    #[must_use]
    pub fn overflow_menu(mut self, enabled: bool) -> Self {
        self.overflow_menu = enabled;
        self
    }

    #[must_use]
    pub fn digit_jump(mut self, yes: bool) -> Self {
        self.digit_jump = yes;
        self
    }

    #[must_use]
    pub fn wrap(mut self, yes: bool) -> Self {
        self.wrap = yes;
        self
    }

    pub fn try_new_key_bindings(
        mut self,
        bindings: Option<&'static [Binding<TabsCmd>]>,
    ) -> Result<Self, TabsKeyBindingsError> {
        validate_tabs_key_bindings(self.nav_profile, bindings, self.close_key_bindings)?;
        self.new_key_bindings = bindings;
        Ok(self)
    }

    #[must_use]
    pub fn closable(mut self, yes: bool) -> Self {
        self.closable = yes;
        self
    }

    pub fn try_close_key_bindings(
        mut self,
        bindings: Option<&'static [Binding<TabsCmd>]>,
    ) -> Result<Self, TabsKeyBindingsError> {
        validate_tabs_key_bindings(self.nav_profile, self.new_key_bindings, bindings)?;
        self.close_key_bindings = bindings;
        Ok(self)
    }

    #[must_use]
    pub fn middle_click_closes(mut self, yes: bool) -> Self {
        self.middle_click_closes = yes;
        self
    }

    #[must_use]
    pub fn closable_keys(mut self, keys: &'a [ItemKey]) -> Self {
        self.closable_keys = Some(keys);
        self
    }

    #[must_use]
    pub fn close_reveal(mut self, reveal: TabCloseReveal) -> Self {
        self.close_reveal = reveal;
        self
    }

    #[must_use]
    pub fn close_target_w(mut self, cells: u16) -> Self {
        self.close_target_w = Some(cells);
        self
    }

    #[must_use]
    pub fn close_danger_hover(mut self, danger: bool) -> Self {
        self.close_danger_hover = danger;
        self
    }

    #[must_use]
    pub fn status(mut self, s: Status) -> Self {
        self.status = s;
        self
    }

    #[must_use]
    pub fn rule(mut self, rule: TabRule) -> Self {
        self.rule = Some(rule);
        self
    }

    #[must_use]
    pub fn rule_focus_hidden(mut self, hidden: bool) -> Self {
        self.rule_focus_hidden = Some(hidden);
        self
    }

    #[must_use]
    pub fn passive(mut self, passive: bool) -> Self {
        self.passive = passive;
        self
    }

    #[must_use]
    pub fn passive_focused(mut self, focused: bool) -> Self {
        self.passive_focused = focused;
        self
    }

    #[must_use]
    pub fn item_marks(mut self, mapper: fn(&T) -> TabMarks) -> Self {
        self.item_marks = Some(mapper);
        self
    }

    #[must_use]
    pub fn patch(mut self, p: &'a StylePatch) -> Self {
        self.ov = self.ov.global(p);
        self
    }

    #[must_use]
    pub fn patch_part(mut self, ps: &'a [(Part, StylePatch)]) -> Self {
        self.ov = self.ov.part(ps);
        self
    }

    #[must_use]
    pub fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.ov = self.ov.slot(part, slot);
        self
    }

    pub fn new_key_table(&self) -> &'static [Binding<TabsCmd>] {
        self.effective_key_groups().new
    }

    pub fn close_key_table(&self) -> &'static [Binding<TabsCmd>] {
        self.effective_key_groups().close
    }

    fn table(&self) -> &'static [Binding<TabsCmd>] {
        self.effective_key_groups().base
    }

    fn effective_key_groups(&self) -> EffectiveTabsKeyGroups {
        if self.passive {
            return EffectiveTabsKeyGroups::EMPTY;
        }

        let close_enabled = self.close_binding_enabled();
        let new_base_enabled = self.allow_new && self.new_key_bindings.is_none();
        let close_base_enabled = close_enabled && self.close_key_bindings.is_none();
        let base = if self.new_key_bindings.is_none() && self.close_key_bindings.is_none() {
            table_for_navigation_profile(
                self.nav_profile,
                close_enabled,
                self.allow_new,
                self.digit_jump,
            )
        } else {
            table_for_navigation_profile(
                self.nav_profile,
                close_base_enabled,
                new_base_enabled,
                self.digit_jump,
            )
        };
        let new = if self.allow_new {
            self.new_key_bindings.unwrap_or(&[])
        } else {
            &[]
        };
        let close = if close_enabled {
            self.close_key_bindings.unwrap_or(&[])
        } else {
            &[]
        };
        EffectiveTabsKeyGroups { base, new, close }
    }

    fn close_binding_enabled(&self) -> bool {
        self.closable_keys
            .map_or(self.closable, |keys| !keys.is_empty())
    }

    fn close_enabled(&self, key: ItemKey) -> bool {
        self.closable_keys
            .map_or(self.closable, |keys| keys.contains(&key))
    }
}

pub(crate) const fn table_for(closable: bool, allow_new: bool) -> &'static [Binding<TabsCmd>] {
    match (closable, allow_new) {
        (false, false) => &PLAIN,
        (true, false) => &CLOSABLE,
        (false, true) => &NEWABLE,
        (true, true) => &FULL,
    }
}

const fn table_for_digit_jump(
    closable: bool,
    allow_new: bool,
    digit_jump: bool,
) -> &'static [Binding<TabsCmd>] {
    if digit_jump {
        return table_for(closable, allow_new);
    }
    match (closable, allow_new) {
        (false, false) => &PLAIN_DIGIT_FREE,
        (true, false) => &CLOSABLE_DIGIT_FREE,
        (false, true) => &NEWABLE_DIGIT_FREE,
        (true, true) => &FULL_DIGIT_FREE,
    }
}

fn table_for_navigation_profile(
    profile: Option<TabNavProfile>,
    closable: bool,
    allow_new: bool,
    digit_jump: bool,
) -> &'static [Binding<TabsCmd>] {
    match profile {
        Some(profile) => profile_tables(profile).table(closable, allow_new, digit_jump),
        None => table_for_digit_jump(closable, allow_new, digit_jump),
    }
}

fn validate_tabs_key_bindings(
    profile: Option<TabNavProfile>,
    new_bindings: Option<&'static [Binding<TabsCmd>]>,
    close_bindings: Option<&'static [Binding<TabsCmd>]>,
) -> Result<(), TabsKeyBindingsError> {
    if let Some(bindings) = new_bindings {
        for (index, binding) in bindings.iter().enumerate() {
            if binding.cmd != TabsCmd::New {
                return Err(TabsKeyBindingsError::NewGroupCommand {
                    index,
                    found: binding.cmd,
                });
            }
        }
    }
    if let Some(bindings) = close_bindings {
        for (index, binding) in bindings.iter().enumerate() {
            if binding.cmd != TabsCmd::Close {
                return Err(TabsKeyBindingsError::CloseGroupCommand {
                    index,
                    found: binding.cmd,
                });
            }
        }
    }

    let base = table_for_navigation_profile(
        profile,
        close_bindings.is_none(),
        new_bindings.is_none(),
        true,
    );
    let mut seen: Vec<(crate::ActionKey, Option<Chord>)> = Vec::new();
    for binding in base
        .iter()
        .chain(new_bindings.unwrap_or(&[]))
        .chain(close_bindings.unwrap_or(&[]))
    {
        if seen.iter().any(|(action, _)| *action == binding.action) {
            return Err(TabsKeyBindingsError::DuplicateAction {
                action: binding.action,
            });
        }
        if let Some(chord) = binding.chord
            && let Some((first, _)) = seen
                .iter()
                .find(|(_, prior)| prior.is_some_and(|prior| prior.effective_eq(&chord)))
        {
            return Err(TabsKeyBindingsError::ConflictingChord {
                first: *first,
                second: binding.action,
            });
        }
        seen.push((binding.action, binding.chord));
    }
    Ok(())
}

fn digits(n: usize, buf: &mut [u8; 2]) -> &str {
    let n = n.min(99);
    if n >= 10 {
        buf[0] = b'0'.saturating_add((n / 10) as u8);
        buf[1] = b'0'.saturating_add((n % 10) as u8);
        core::str::from_utf8(&buf[..]).unwrap_or("")
    } else {
        buf[0] = b'0'.saturating_add(n as u8);
        core::str::from_utf8(&buf[..1]).unwrap_or("")
    }
}

impl<T, K: KeyFn<T>, R: RowFn<T>> Tabs<'_, T, K, R> {
    fn key_at(&self, items: &[T], i: usize) -> ItemKey {
        items
            .get(i)
            .map_or(ItemKey::index(i), |it| self.key.key(it, i))
    }

    fn index_of(&self, items: &[T], key: ItemKey, hint: Option<usize>) -> Option<usize> {
        if let Some(h) = hint
            && h < items.len()
            && self.key_at(items, h) == key
        {
            return Some(h);
        }
        (0..items.len()).find(|&i| self.key_at(items, i) == key)
    }

    fn activate(&self, st: &mut TabsState, items: &[T], i: usize, acc: &mut Acc<TabsAction>) {
        if items.is_empty() {
            acc.consumed();
            return;
        }
        let i = i.min(items.len().saturating_sub(1));
        let key = self.key_at(items, i);
        st.set_active(i, key);
        acc.action(TabsAction::Activated(key));
    }

    fn follow(&self, st: &mut TabsState, items: &[T], fit: usize) {
        let Some(active) = st.active else { return };
        let Some(ai) = self.index_of(items, active, Some(st.core.cursor_index())) else {
            return;
        };
        if ai < st.first_index {
            st.first_index = ai;
            st.first = Some(active);
        } else if fit > 0 && ai >= st.first_index.saturating_add(fit) {
            let i = ai.saturating_add(1).saturating_sub(fit);
            st.first_index = i;
            st.first = Some(self.key_at(items, i));
        }
    }

    fn committed_wheel_bounds(
        &self,
        frame: TabsWheelFrame,
        layout: Option<LayoutFacts>,
        st: &TabsState,
        items: &[T],
    ) -> Option<usize> {
        if self.passive
            || frame.kind != TabsWheelFrameKind::Planned
            || !frame.eligible
            || frame.planned_whole_rows != self.planned_whole_rows
            || frame.content_len != items.len()
            || (frame.viewport_len == 0 && !self.planned_whole_rows)
            || frame.visible_area.is_empty()
            || frame.area.is_empty()
            || items.is_empty()
        {
            return None;
        }
        let facts = layout?;
        if facts.content_len != frame.content_len
            || facts.viewport_len != frame.viewport_len
            || facts.rows != frame.rows
            || facts.cols != frame.cols
        {
            return None;
        }
        if self.planned_whole_rows
            && (frame.active != st.active
                || frame.manual_window_active != st.manual_window_active
                || frame.visible_row_area.is_empty())
        {
            return None;
        }
        let terminal = frame.terminal_first_index?;
        if terminal == 0 || terminal >= items.len() {
            return None;
        }
        let last_index = items.len().saturating_sub(1);
        if frame.last_key != Some(self.key_at(items, last_index)) {
            return None;
        }
        let first_key = frame.first_key?;
        if st.first != Some(first_key) {
            return None;
        }
        let first_index = self.index_of(items, first_key, Some(frame.first_index))?;
        if first_index != frame.first_index || st.first_index != first_index {
            return None;
        }
        Some(terminal)
    }

    fn converge_manual_window(
        &self,
        frame: TabsWheelFrame,
        layout: Option<LayoutFacts>,
        st: &mut TabsState,
        items: &[T],
    ) -> bool {
        if st.manual_window_active != Some(st.active)
            || frame.kind != TabsWheelFrameKind::Planned
            || frame.planned_whole_rows != self.planned_whole_rows
            || frame.content_len != items.len()
            || frame.area.is_empty()
            || items.is_empty()
        {
            return false;
        }
        let Some(facts) = layout else {
            return false;
        };
        if facts.content_len != frame.content_len
            || facts.viewport_len != frame.viewport_len
            || facts.rows != frame.rows
            || facts.cols != frame.cols
        {
            return false;
        }
        let last_index = items.len().saturating_sub(1);
        if frame.last_key != Some(self.key_at(items, last_index)) {
            return false;
        }
        let Some(first_key) = frame.first_key else {
            return false;
        };
        let Some(first_index) = self.index_of(items, first_key, Some(frame.first_index)) else {
            return false;
        };
        if st.first == Some(first_key) && st.first_index == first_index {
            return false;
        }
        st.first = Some(first_key);
        st.first_index = first_index;
        true
    }

    fn adopt_geometry_follow(
        &self,
        frame: TabsWheelFrame,
        layout: Option<LayoutFacts>,
        st: &mut TabsState,
        items: &[T],
    ) -> bool {
        let Some(follow) = frame.geometry_follow else {
            return false;
        };
        if frame.kind != TabsWheelFrameKind::Planned
            || frame.planned_whole_rows != self.planned_whole_rows
            || manual_window_owns_follow(frame.kind, st.manual_window_active, st.active)
            || frame.content_len != items.len()
            || frame.area.is_empty()
            || frame.visible_area.is_empty()
            || st.active != Some(follow.active_key)
        {
            return false;
        }
        let Some(facts) = layout else {
            return false;
        };
        if facts.content_len != frame.content_len
            || facts.viewport_len != frame.viewport_len
            || facts.rows != frame.rows
            || facts.cols != frame.cols
            || frame.first_index != follow.first_index
            || frame.first_key != Some(follow.first_key)
        {
            return false;
        }
        let last_index = items.len().saturating_sub(1);
        if items.is_empty() || frame.last_key != Some(self.key_at(items, last_index)) {
            return false;
        }
        let Some(active_index) = self.index_of(items, follow.active_key, None) else {
            return false;
        };
        let Some(first_index) = self.index_of(items, follow.first_key, Some(follow.first_index))
        else {
            return false;
        };
        if first_index != follow.first_index || active_index < first_index {
            return false;
        }
        if st.first == Some(follow.first_key) && st.first_index == follow.first_index {
            return false;
        }
        st.first = Some(follow.first_key);
        st.first_index = follow.first_index;
        true
    }

    const fn menu_layer_id(&self) -> Id {
        self.id.sub("overflow-menu")
    }

    fn update_retired_menu_list(
        &self,
        cx: &mut Cx<'_>,
        state: &mut ListState,
        acc: &mut Acc<TabsAction>,
    ) {
        let list_id = self.menu_layer_id().sub("items");
        let list = List::new(list_id)
            .key(tabs_menu_key as fn(&TabsMenuItem) -> ItemKey)
            .whole_rows(true);
        let response = list.update(cx, state, &[]);
        acc.fold(&response.erase());
    }

    fn service_retired_menu_list(
        &self,
        cx: &mut Cx<'_>,
        state: &mut TabsState,
        acc: &mut Acc<TabsAction>,
    ) {
        if let Some(menu) = state.menu.as_mut() {
            self.update_retired_menu_list(cx, &mut menu.list, acc);
        } else {
            let mut retired = ListState::default();
            self.update_retired_menu_list(cx, &mut retired, acc);
        }
    }

    fn show_hidden_binding_available(&self) -> bool {
        let Some(chord) = SHOW_HIDDEN_BINDINGS[0].chord else {
            return false;
        };
        let action = SHOW_HIDDEN_BINDINGS[0].action;
        let groups = self.effective_key_groups();
        ![groups.base, groups.new, groups.close]
            .into_iter()
            .flatten()
            .any(|binding| {
                binding.action == action
                    || binding
                        .chord
                        .is_some_and(|configured| configured.effective_eq(&chord))
            })
    }

    fn menu_size(&self, cx: &Cx<'_>, items: &[TabsMenuItem]) -> LayerSize {
        let design = cx.design();
        let body_width = items
            .iter()
            .map(|item| {
                item.label_segments.iter().fold(0u16, |width, segment| {
                    width.saturating_add(crate::text::width(segment))
                })
            })
            .max()
            .unwrap_or(0);
        let title_width = crate::text::width("Hidden tabs").saturating_add(2);
        let chrome_width = design.space.card_inset.saturating_mul(2);
        let width = body_width
            .max(title_width)
            .saturating_add(chrome_width)
            .clamp(design.size.popup_min_width, design.size.popup_max_width);
        let rows = items
            .len()
            .min(usize::from(design.size.popup_max_rows))
            .max(1)
            .min(usize::from(u16::MAX)) as u16;
        LayerSize::Fixed(width, rows.saturating_add(3))
    }

    fn menu_frame_matches(&self, cx: &Cx<'_>, frame: &TabsMenuFrame, items: &[T]) -> bool {
        if !frame.eligible
            || frame.planned_whole_rows != self.planned_whole_rows
            || frame.owner != Some(self.id)
            || !self.overflow_menu
            || self.passive
            || items.is_empty()
            || frame.content_len != items.len()
            || frame.items.is_empty()
            || frame.more_area.is_empty()
            || frame.visible_more_area.is_empty()
            || frame.more_area.intersection(frame.area) != frame.more_area
            || cx.area(self.id) != Some(frame.visible_area)
            || cx.area_of_part(self.id, PartRef::item(Part::OVERFLOW, ItemKey::index(2)))
                != Some(frame.visible_more_area)
        {
            return false;
        }
        let Some(layout) = cx.layout(self.id) else {
            return false;
        };
        if layout.content_len != frame.content_len
            || layout.viewport_len != frame.viewport_len
            || layout.rows != frame.rows
            || layout.cols != frame.cols
        {
            return false;
        }
        let first = items
            .get(frame.first_index)
            .map(|_| self.key_at(items, frame.first_index));
        let last = items.last().map(|_| self.key_at(items, items.len() - 1));
        frame.first_key == first && frame.last_key == last
    }

    fn open_menu(
        &self,
        cx: &mut Cx<'_>,
        state: &mut TabsState,
        items: &[T],
        frame: &TabsMenuFrame,
    ) -> bool {
        if !self.menu_frame_matches(cx, frame, items)
            || cx.is_open(self.menu_layer_id())
            || cx.state(self.id).contains(StateFlags::DISABLED)
        {
            return false;
        }
        let Some(first) = frame.items.first() else {
            return false;
        };
        let mut list = ListState::default();
        let cursor = state
            .active
            .and_then(|active| frame.items.iter().position(|item| item.key == active))
            .unwrap_or(0);
        let cursor_key = frame.items.get(cursor).map_or(first.key, |item| item.key);
        list.set_cursor(cursor, cursor_key);
        let layer_id = self.menu_layer_id();
        let list_id = layer_id.sub("items");
        let spec = LayerSpec::popover(
            self.id,
            Anchor::Rect {
                rect: frame.more_area,
                side: Side::Below,
                align: CrossAlign::Start,
            },
        )
        .dismiss(Dismiss::ESC_AND_OUTSIDE)
        .initial_focus(list_id)
        .size(self.menu_size(cx, &frame.items));
        let Some(opening) = cx.open_layer_with_token(layer_id, spec) else {
            return false;
        };
        state.menu = Some(TabsOpenMenu {
            opening,
            items: frame.items.clone(),
            list,
            more_area: frame.more_area,
            content_len: frame.content_len,
            first_key: frame.first_key,
        });
        true
    }

    fn update_menu(
        &self,
        cx: &mut Cx<'_>,
        state: &mut TabsState,
        items: &[T],
        frame: &TabsMenuFrame,
        acc: &mut Acc<TabsAction>,
    ) -> bool {
        let layer_id = self.menu_layer_id();
        let mut dismissed_current = false;
        while let Some(receipt) = cx.take_layer_intent_receipt(layer_id) {
            if state
                .menu
                .as_ref()
                .is_some_and(|menu| menu.opening == receipt.opening())
            {
                match receipt.event() {
                    LayerEvent::Opened => {}
                    LayerEvent::Closed(_) | LayerEvent::Dismissed(_) => {
                        state.menu = None;
                        dismissed_current = true;
                        acc.changed();
                    }
                }
            }
        }

        if state.menu.is_none() {
            if let Some(opening) = cx.layer_opening(layer_id)
                && opening.owner() == self.id
            {
                cx.close_layer(layer_id, None);
                self.service_retired_menu_list(cx, state, acc);
                for _ in cx.intents(self.id) {
                    acc.consumed();
                }
                acc.changed();
                return true;
            }
            self.service_retired_menu_list(cx, state, acc);
            if dismissed_current {
                for _ in cx.intents(self.id) {
                    acc.consumed();
                }
            }
            return dismissed_current;
        }

        let opening = state.menu.as_ref().map(|menu| menu.opening);
        let current_opening = cx.layer_opening(layer_id);
        let valid = opening == current_opening
            && !cx.state(self.id).contains(StateFlags::DISABLED)
            && self.menu_frame_matches(cx, frame, items);
        if !valid {
            if opening.is_some() && opening == current_opening {
                cx.close_layer(layer_id, None);
            }
            self.service_retired_menu_list(cx, state, acc);
            state.menu = None;
            for _ in cx.intents(self.id) {
                acc.consumed();
            }
            acc.changed();
            return true;
        }

        let selected_key_was_removed = state
            .menu
            .as_ref()
            .and_then(|menu| menu.list.cursor())
            .is_some_and(|key| self.index_of(items, key, None).is_none());
        if selected_key_was_removed {
            if opening.is_some() && opening == current_opening {
                cx.close_layer(layer_id, None);
            }
            self.service_retired_menu_list(cx, state, acc);
            state.menu = None;
            for _ in cx.intents(self.id) {
                acc.consumed();
            }
            acc.changed();
            return true;
        }

        let Some(menu) = state.menu.as_mut() else {
            self.service_retired_menu_list(cx, state, acc);
            return true;
        };
        menu.items
            .retain(|item| self.index_of(items, item.key, None).is_some());
        if menu.items.is_empty() {
            cx.close_layer(layer_id, None);
            self.update_retired_menu_list(cx, &mut menu.list, acc);
            state.menu = None;
            for _ in cx.intents(self.id) {
                acc.consumed();
            }
            acc.changed();
            return true;
        }
        let anchor = Anchor::Rect {
            rect: frame.more_area,
            side: Side::Below,
            align: CrossAlign::Start,
        };
        if menu.more_area != frame.more_area {
            cx.reanchor_layer(layer_id, anchor);
            menu.more_area = frame.more_area;
            acc.changed();
        }
        if menu.content_len != frame.content_len || menu.first_key != frame.first_key {
            menu.content_len = frame.content_len;
            menu.first_key = frame.first_key;
            acc.changed();
        }
        let size = self.menu_size(cx, &menu.items);
        cx.resize_layer(layer_id, size);

        let list_id = layer_id.sub("items");
        let list = List::new(list_id)
            .key(tabs_menu_key as fn(&TabsMenuItem) -> ItemKey)
            .whole_rows(true);
        let response = list.update(cx, &mut menu.list, &menu.items);
        let selected = response.action_ref().copied();
        for _ in cx.intents(self.id) {
            acc.consumed();
        }
        match selected {
            Some(ListAction::Chose(key) | ListAction::Activated(key)) => {
                if let Some(index) = self.index_of(items, key, None) {
                    self.activate(state, items, index, acc);
                    self.follow(state, items, frame.viewport_len);
                    state.manual_window_active = None;
                } else {
                    acc.consumed();
                }
                cx.close_layer(layer_id, None);
                state.menu = None;
                true
            }
            _ => {
                acc.fold(&response.erase());
                true
            }
        }
    }

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TabsState, items: &[T]) -> Response<TabsAction> {
        let frame = *cx.cache::<TabsWheelFrame>(self.id);
        let menu_frame = cx.cache::<TabsMenuFrame>(self.id).clone();
        let layout = cx.layout(self.id);
        let len = items.len();
        let _ = st.reconcile(len, |i| self.key_at(items, i));
        if st.close_reveal_press.is_some_and(|key| {
            self.close_reveal != TabCloseReveal::Hover
                || self.index_of(items, key, None).is_none()
                || !self.close_enabled(key)
        }) {
            st.close_reveal_press = None;
        }
        if st.core.cursor().is_none() && len > 0 {
            let key = self.key_at(items, 0);
            st.core.set_cursor(0, key);
        }
        let active_before_initialization = st.active;
        if st.active.is_none() {
            st.active = st.core.cursor();
        }
        if st.active != active_before_initialization {
            st.manual_window_active = None;
        }
        if st.first.is_none() && len > 0 {
            st.first = Some(self.key_at(items, 0));
            st.first_index = 0;
        }
        if frame.kind == TabsWheelFrameKind::Legacy {
            st.manual_window_active = None;
        }
        let mut acc = Acc::<TabsAction>::new();
        if self.adopt_geometry_follow(frame, layout, st, items) {
            acc.changed();
        }
        if self.update_menu(cx, st, items, &menu_frame, &mut acc) {
            return acc.finish(self.id);
        }
        if self.converge_manual_window(frame, layout, st, items) {
            acc.changed();
        }
        let mut committed_bounds = None;
        let key_groups = self.effective_key_groups();
        for it in cx.intents(self.id) {
            if st.menu.is_some() {
                acc.consumed();
                continue;
            }
            match it {
                Intent::Cancel | Intent::FocusOut { .. } | Intent::Layer(_) => {
                    st.close_reveal_press = None;
                }
                Intent::Binding(action) => {
                    if self.passive {
                        acc.consumed();
                        continue;
                    }
                    if action == SHOW_HIDDEN_ACTION {
                        if self.show_hidden_binding_available()
                            && self.open_menu(cx, st, items, &menu_frame)
                        {
                            acc.changed();
                        } else {
                            acc.consumed();
                        }
                        continue;
                    }
                    let cur = st.core.cursor_index();
                    match key_groups.command(action) {
                        Some(TabsCmd::Prev) => {
                            let previous = if self.wrap && !items.is_empty() && cur == 0 {
                                len.saturating_sub(1)
                            } else {
                                cur.saturating_sub(1)
                            };
                            self.activate(st, items, previous, &mut acc);
                        }
                        Some(TabsCmd::Next) => {
                            let next =
                                if self.wrap && !items.is_empty() && cur >= len.saturating_sub(1) {
                                    0
                                } else {
                                    cur.saturating_add(1)
                                };
                            self.activate(st, items, next, &mut acc);
                        }
                        Some(TabsCmd::Activate) => self.activate(st, items, cur, &mut acc),
                        Some(TabsCmd::Nth(n)) => {
                            let i = usize::from(n).saturating_sub(1);
                            if i < len {
                                self.activate(st, items, i, &mut acc);
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(TabsCmd::Close) => {
                            if let Some(item) = items.get(cur) {
                                let key = self.key.key(item, cur);
                                if self.close_enabled(key) {
                                    acc.action(TabsAction::Close(key));
                                } else {
                                    acc.consumed();
                                }
                            } else {
                                acc.consumed();
                            }
                        }
                        Some(TabsCmd::New) => acc.action(TabsAction::New),
                        None => {}
                    }
                }
                Intent::Wheel { delta, .. } => {
                    acc.consumed();
                    if delta == 0 {
                        continue;
                    }
                    let bounds = *committed_bounds.get_or_insert_with(|| {
                        self.committed_wheel_bounds(frame, layout, st, items)
                    });
                    let Some(terminal) = bounds else {
                        continue;
                    };
                    let current = st.first_index.min(terminal);
                    let next = if delta > 0 {
                        current.saturating_add(1).min(terminal)
                    } else {
                        current.saturating_sub(1)
                    };
                    if next != st.first_index {
                        st.first_index = next;
                        st.first = items.get(next).map(|_| self.key_at(items, next));
                        st.manual_window_active = Some(st.active);
                        acc.changed();
                    }
                }
                Intent::Pointer { phase, part, .. } => {
                    match phase {
                        Phase::Press => {
                            st.close_reveal_press = if self.close_reveal == TabCloseReveal::Hover
                                && part.part == Part::TAB
                            {
                                part.item.filter(|key| {
                                    self.close_enabled(*key)
                                        && cx
                                            .area_of_part(self.id, PartRef::item(Part::CLOSE, *key))
                                            .is_none()
                                })
                            } else {
                                None
                            };
                        }
                        Phase::Release | Phase::DragEnd
                            if part.part == Part::TAB && st.close_reveal_press == part.item =>
                        {
                            st.close_reveal_press = None;
                        }
                        _ => {}
                    }
                    match (phase, part.part, part.item) {
                        (Phase::Press, Part::TAB, Some(k)) => {
                            if let Some(i) = self.index_of(items, k, Some(st.core.cursor_index())) {
                                st.core.set_cursor(i, k);
                            }
                            acc.changed();
                        }
                        (Phase::Click | Phase::DoubleClick, Part::TAB, Some(k)) => {
                            match self.index_of(items, k, Some(st.core.cursor_index())) {
                                Some(i) => self.activate(st, items, i, &mut acc),
                                None => acc.consumed(),
                            }
                        }
                        (Phase::Click, Part::CLOSE, Some(k)) => {
                            if self
                                .index_of(items, k, Some(st.core.cursor_index()))
                                .is_some()
                                && self.close_enabled(k)
                            {
                                acc.action(TabsAction::Close(k));
                            } else {
                                acc.consumed();
                            }
                        }
                        (Phase::Middle, Part::TAB | Part::CLOSE, Some(k)) => {
                            if self.middle_click_closes
                                && self
                                    .index_of(items, k, Some(st.core.cursor_index()))
                                    .is_some()
                                && self.close_enabled(k)
                            {
                                acc.action(TabsAction::Close(k));
                            }
                        }
                        (Phase::Middle, _, _) => {}
                        (Phase::Click, Part::NEW, _) => acc.action(TabsAction::New),
                        (
                            Phase::Click | Phase::DoubleClick,
                            Part::OVERFLOW,
                            Some(ItemKey::Index(2)),
                        ) => {
                            if !self.passive && self.open_menu(cx, st, items, &menu_frame) {
                                acc.changed();
                            } else {
                                acc.consumed();
                            }
                        }
                        (Phase::Click, Part::OVERFLOW, Some(ItemKey::Index(0))) => {
                            let bounds = *committed_bounds.get_or_insert_with(|| {
                                self.committed_wheel_bounds(frame, layout, st, items)
                            });
                            if let Some(terminal) = bounds {
                                let next = st.first_index.min(terminal).saturating_sub(1);
                                if next != st.first_index {
                                    st.first_index = next;
                                    st.first = items.get(next).map(|_| self.key_at(items, next));
                                    st.manual_window_active = Some(st.active);
                                    acc.changed();
                                } else {
                                    acc.consumed();
                                }
                            } else if (self.planned_whole_rows
                                && frame.kind != TabsWheelFrameKind::Legacy)
                                || (frame.kind == TabsWheelFrameKind::Planned
                                    && frame.planned_whole_rows)
                            {
                                acc.consumed();
                            } else {
                                let i = st.first_index.saturating_sub(1);
                                st.first_index = i;
                                st.first = (len > 0).then(|| self.key_at(items, i));
                                acc.changed();
                            }
                        }
                        (Phase::Click, Part::OVERFLOW, Some(ItemKey::Index(_))) => {
                            let bounds = *committed_bounds.get_or_insert_with(|| {
                                self.committed_wheel_bounds(frame, layout, st, items)
                            });
                            if let Some(terminal) = bounds {
                                let next = st.first_index.saturating_add(1).min(terminal);
                                if next != st.first_index {
                                    st.first_index = next;
                                    st.first = items.get(next).map(|_| self.key_at(items, next));
                                    st.manual_window_active = Some(st.active);
                                    acc.changed();
                                } else {
                                    acc.consumed();
                                }
                            } else if (self.planned_whole_rows
                                && frame.kind != TabsWheelFrameKind::Legacy)
                                || (frame.kind == TabsWheelFrameKind::Planned
                                    && frame.planned_whole_rows)
                            {
                                acc.consumed();
                            } else {
                                let i = st.first_index.saturating_add(1).min(len.saturating_sub(1));
                                st.first_index = i;
                                st.first = (len > 0).then(|| self.key_at(items, i));
                                acc.changed();
                            }
                        }
                        _ => acc.consumed(),
                    }
                }
                _ => {}
            }
        }
        if st.menu.is_some() {
            return acc.finish(self.id);
        }
        if let Some(mapper) = self.item_marks
            && items.iter().any(|item| mapper(item).busy)
        {
            let frames = cx.design().motion.spinner_frames;
            if frames.len() > 1 {
                let tick_ms = cx.design().motion.tick_ms.max(1);
                let now = cx.now();
                let elapsed_ms = now.as_duration().as_millis();
                let frame =
                    usize::try_from((elapsed_ms / u128::from(tick_ms)) % frames.len() as u128)
                        .unwrap_or(0);
                let phase = cx.cache::<ItemSpinnerPhase>(self.id);
                if phase.frame != frame {
                    phase.frame = frame;
                    acc.changed();
                }
                let elapsed_in_tick = elapsed_ms % u128::from(tick_ms);
                let delay = tick_ms.saturating_sub(u64::try_from(elapsed_in_tick).unwrap_or(0));
                cx.request_repaint_after(Duration::from_millis(delay));
            }
        }

        let manual_window_owned =
            manual_window_owns_follow(frame.kind, st.manual_window_active, st.active);
        if !manual_window_owned && let Some(l) = layout {
            self.follow(st, items, l.viewport_len);
        }
        acc.finish(self.id)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, st: &TabsState, items: &[T]) -> Rect {
        self.draw_impl(ui, area, TabsRenderSource::Legacy { state: st, items })
    }

    fn paint_item_mark(
        &self,
        ui: &mut Ui<'_>,
        mark: TabItemMark,
        area: Rect,
        tab_style: impl Into<crate::theme::PaintStyle>,
    ) {
        if area.is_empty() {
            return;
        }
        let tab_style = tab_style.into();
        ui.fill(area, tab_style);
        if let Some(slot) = self.ov.slot_for(Part::ICON) {
            slot(ui, area);
            return;
        }

        let mut defaults = StylePatch::new().set_fg(mark.default_role());
        if mark == TabItemMark::Done {
            defaults = defaults.remove(ratatui_core::style::Modifier::BOLD);
        }
        let local = self.ov.part_patch(Part::ICON);
        let resolved = ui.style_defaults_with_explicit(
            Family::TABS,
            Variant::DEFAULT,
            Part::ICON,
            mark.flags(),
            StyleDefaults::new(defaults),
            local.as_ref(),
        );
        let icon = resolved.resolved;
        let explicit = resolved.explicit;
        let mark_style = icon.over(tab_style);
        self.ov.note(
            ui,
            self.id,
            Family::TABS,
            Variant::DEFAULT,
            Part::ICON,
            icon,
        );

        match icon.glyph {
            Slot::Set(glyph) => {
                let text = ui.glyph_str(glyph);
                let icon_area = item_mark_glyph_area(area, crate::text::width(text));
                ui.glyph(icon_area, glyph, mark_style);
            }
            Slot::Clear => ui.fill(area, mark_style),
            Slot::Inherit => match mark {
                TabItemMark::Busy => {
                    let frames = ui.design().motion.spinner_frames;
                    if !frames.is_empty() {
                        let frame_index = if frames.len() > 1 {
                            ui.cache::<ItemSpinnerPhase>(self.id).frame % frames.len()
                        } else {
                            0
                        };
                        if let Some(frame) = frames.get(frame_index) {
                            let icon_area = item_mark_glyph_area(area, crate::text::width(frame));
                            ui.paint_str(icon_area, frame, mark_style);
                        }
                    }
                }
                _ => {
                    if let Some(glyph) = mark.default_glyph() {
                        let text = ui.glyph_str(glyph);
                        let icon_area = item_mark_glyph_area(area, crate::text::width(text));
                        ui.glyph(icon_area, glyph, mark_style);
                    }
                }
            },
        }
        termrock_runtime::author::patch_style(ui, area, icon.style, explicit);
    }

    fn paint_item_affix(
        &self,
        ui: &mut Ui<'_>,
        row: &TabRowLayout,
        part: Part,
        flags: StateFlags,
        segment: AffixSegment,
        tab_style: crate::theme::PaintStyle,
    ) {
        let Some((style_area, _)) = row.affix_bounds(part) else {
            return;
        };
        let Some(text) = row.label_segments.get(segment.index).map(String::as_str) else {
            return;
        };
        let text_x = match part {
            Part::PREFIX => style_area.x,
            Part::META => style_area.x.saturating_add(1),
            _ => return,
        };
        let text_area = Rect {
            x: text_x,
            width: segment.width,
            ..style_area
        }
        .intersection(style_area);
        if style_area.is_empty() || text_area.is_empty() {
            return;
        }

        let foreground = Role::Fg(if flags.contains(StateFlags::ACTIVE) {
            FgStep::Secondary
        } else {
            FgStep::Muted
        });
        let defaults = StylePatch::new().set_fg(foreground).remove(Modifier::BOLD);
        let local = self.ov.part_patch(part);
        let (mut semantic, ownership) = ui.style_defaults_with_fallback_ownership(
            Family::TABS,
            Variant::DEFAULT,
            part,
            flags,
            StyleDefaults::new(defaults),
            local.as_ref(),
        );

        if ownership.fg == Slot::Inherit {
            semantic.style = semantic
                .style
                .patch(ui.paint_patch(&StylePatch::new().set_fg(foreground)));
        }

        if !ownership.add.contains(Modifier::BOLD) {
            semantic.style = semantic
                .style
                .patch(ui.paint_patch(&StylePatch::new().remove(Modifier::BOLD)));
        }

        self.ov
            .note(ui, self.id, Family::TABS, Variant::DEFAULT, part, semantic);
        let style = semantic.over(tab_style);
        if row
            .affixes
            .as_ref()
            .is_some_and(|affixes| affixes.plate_separators)
        {
            ui.paint_style(text_area, style);
        } else {
            ui.paint_style(style_area, style);
        }
        let clip = ui.full();
        paint_logical_text(ui, text_area, clip, text, style);
    }

    fn paint_planned_affixes(
        &self,
        ui: &mut Ui<'_>,
        row: &TabRowLayout,
        flags: StateFlags,
        tab_style: crate::theme::PaintStyle,
    ) {
        let Some(affixes) = row.affixes.as_ref() else {
            return;
        };
        if let Some(prefix) = affixes.prefix {
            self.paint_item_affix(ui, row, Part::PREFIX, flags, prefix, tab_style);
        }
        if let Some(suffix) = affixes.suffix {
            self.paint_item_affix(ui, row, Part::META, flags, suffix, tab_style);
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one shared paint pass consumes either legacy rows or an owned semantic layout"
    )]
    fn draw_impl(&self, ui: &mut Ui<'_>, area: Rect, source: TabsRenderSource<'_, T>) -> Rect {
        let mut wheel_frame = TabsWheelFrame::default();
        let mut menu_frame = TabsMenuFrame::default();
        match source {
            TabsRenderSource::Legacy { items, .. } => {
                wheel_frame.kind = TabsWheelFrameKind::Legacy;
                wheel_frame.content_len = items.len();
                wheel_frame.area = area;
                wheel_frame.visible_area = area;
            }
            TabsRenderSource::Planned(layout) => {
                wheel_frame.kind = TabsWheelFrameKind::Planned;
                wheel_frame.planned_whole_rows = layout.planned_whole_rows;
                wheel_frame.active = layout.active;
                wheel_frame.manual_window_active = layout.manual_window_active;
                wheel_frame.content_len = layout.content_len;
                wheel_frame.viewport_len = layout.viewport_len;
                wheel_frame.rows = layout.used_area.height;
                wheel_frame.cols = layout.used_area.width;
                wheel_frame.first_index = layout.first_index;
                wheel_frame.first_key = layout.first_key();
                wheel_frame.geometry_follow = layout.geometry_follow;
                wheel_frame.last_key = layout
                    .owned_items
                    .as_ref()
                    .and_then(|items| items.last())
                    .map(|item| item.key);
                wheel_frame.terminal_first_index = layout.terminal_first_index;
                wheel_frame.area = layout.area;
                wheel_frame.visible_area = layout.visible_area;
                wheel_frame.visible_row_area = layout.row_area.intersection(layout.clip);
                wheel_frame.eligible = !ui.is_inert()
                    && !self.passive
                    && !layout.passive
                    && !layout.visible_area.is_empty()
                    && (!layout.planned_whole_rows || !wheel_frame.visible_row_area.is_empty())
                    && (layout.viewport_len > 0 || layout.planned_whole_rows)
                    && layout
                        .terminal_first_index
                        .is_some_and(|terminal| terminal > 0 && terminal < layout.content_len);
                let hidden = layout.hidden_menu_items();
                if let Some(more_area) = layout.more_area {
                    menu_frame.planned_whole_rows = layout.planned_whole_rows;
                    menu_frame.owner = Some(self.id);
                    menu_frame.area = layout.area;
                    menu_frame.visible_area = layout.visible_area;
                    menu_frame.more_area = more_area;
                    menu_frame.visible_more_area = more_area.intersection(layout.clip);
                    menu_frame.content_len = layout.content_len;
                    menu_frame.viewport_len = layout.viewport_len;
                    menu_frame.rows = layout.used_area.height;
                    menu_frame.cols = layout.used_area.width;
                    menu_frame.first_index = layout.first_index;
                    menu_frame.first_key = layout.first_key();
                    menu_frame.last_key = wheel_frame.last_key;
                    menu_frame.items = hidden;
                    menu_frame.eligible = layout.overflow_menu
                        && !ui.is_inert()
                        && !self.passive
                        && !layout.passive
                        && !ui.state(self.id).contains(StateFlags::DISABLED)
                        && !menu_frame.visible_area.is_empty()
                        && !menu_frame.visible_more_area.is_empty()
                        && !menu_frame.items.is_empty();
                }
            }
        }
        let menu_eligible = menu_frame.eligible;
        *ui.cache::<TabsWheelFrame>(self.id) = wheel_frame;
        *ui.cache::<TabsMenuFrame>(self.id) = menu_frame;
        if area.is_empty() {
            return area;
        }
        if wheel_frame.eligible
            && let TabsRenderSource::Planned(layout) = source
            && let Some(terminal) = wheel_frame.terminal_first_index
        {
            let backward = u16::try_from(wheel_frame.first_index).unwrap_or(u16::MAX);
            let forward =
                u16::try_from(terminal.saturating_sub(wheel_frame.first_index)).unwrap_or(u16::MAX);
            ui.register_scroll(
                self.id,
                layout.visible_area,
                Axes::Both,
                Headroom {
                    up: backward,
                    down: forward,
                    left: backward,
                    right: forward,
                },
            );
        }
        let len = match source {
            TabsRenderSource::Legacy { items, .. } => items.len(),
            TabsRenderSource::Planned(layout) => layout.content_len,
        };
        let row0 = match source {
            TabsRenderSource::Legacy { .. } => first_row(area),
            TabsRenderSource::Planned(layout) => layout.row_area,
        };
        let used = match source {
            TabsRenderSource::Legacy { .. } => Rect {
                height: area.height.min(2),
                ..area
            },
            TabsRenderSource::Planned(layout) => layout.used_area,
        };
        if !ui.is_inert() && !self.passive {
            ui.register_control(self.id, used, Focusability::Focusable);
        }
        let live = PartStyle::flags(ui.state(self.id), self.status.flags());
        let live = if self.passive {
            live.difference(StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE)
        } else {
            live
        };
        let key_groups = self.effective_key_groups();
        if !ui.is_inert() && !self.passive {
            ui.publish_bindings(self.id, live, key_groups.base);
            if !key_groups.new.is_empty() {
                ui.publish_additional_bindings(self.id, live, key_groups.new);
            }
            if !key_groups.close.is_empty() {
                ui.publish_additional_bindings(self.id, live, key_groups.close);
            }
            if menu_eligible && self.show_hidden_binding_available() {
                ui.publish_additional_bindings(self.id, live, SHOW_HIDDEN_BINDINGS);
            }
        }
        let ov = self.ov;
        let id = self.id;
        let strip = ov.style(
            ui,
            id,
            Family::TABS,
            Variant::DEFAULT,
            Part::CONTAINER,
            StateFlags::empty(),
        );
        ui.fill(used, strip.style);
        let rule_row = match source {
            TabsRenderSource::Legacy { .. } => {
                if area.height >= 2 {
                    Some(Rect {
                        y: area.y.saturating_add(1),
                        height: 1,
                        ..row0
                    })
                } else {
                    None
                }
            }
            TabsRenderSource::Planned(layout) => layout.rule_area,
        };
        if let Some(rr) = rule_row {
            let quiet = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::RULE,
                StateFlags::empty(),
            );
            for col in rr.columns() {
                match quiet.glyph {
                    Slot::Set(glyph) => {
                        ui.glyph(col, glyph, quiet.style);
                    }
                    Slot::Inherit => {
                        ui.glyph(col, GlyphRole::RuleQuiet, quiet.style);
                    }
                    Slot::Clear => {
                        ui.fill(col, quiet.style);
                    }
                }
            }
        }
        let overflow_last = match source {
            TabsRenderSource::Legacy { .. } => ui
                .layout(self.id)
                .is_some_and(|l| l.viewport_len < l.content_len),
            TabsRenderSource::Planned(layout) => layout.overflow_reserved,
        };
        let first_index = match source {
            TabsRenderSource::Legacy { state, items } => state
                .first
                .and_then(|f| self.index_of(items, f, Some(state.first_index)))
                .unwrap_or(0)
                .min(len),
            TabsRenderSource::Planned(layout) => layout.first_index,
        };
        let status_w: u16 = if matches!(self.status, Status::Ready) {
            0
        } else {
            2
        };
        let new_w: u16 = if self.allow_new { 4 } else { 0 };
        let left_w: u16 = if first_index > 0 { 4 } else { 0 };
        let right_w: u16 = if overflow_last || first_index > 0 {
            4
        } else {
            0
        };
        let right_limit = match source {
            TabsRenderSource::Legacy { .. } => row0
                .right()
                .saturating_sub(status_w)
                .saturating_sub(right_w)
                .saturating_sub(new_w),
            TabsRenderSource::Planned(layout) => layout.right_limit,
        };
        let first_tab_x = match source {
            TabsRenderSource::Legacy { .. } => row0.x.saturating_add(left_w),
            TabsRenderSource::Planned(layout) => layout.first_tab_x,
        };
        let mut x = first_tab_x;
        let mut fit = 0usize;
        let cursor = match source {
            TabsRenderSource::Legacy { state, .. } => state.core.cursor(),
            TabsRenderSource::Planned(layout) => layout.cursor,
        };
        let active = match source {
            TabsRenderSource::Legacy { state, .. } => state.active,
            TabsRenderSource::Planned(layout) => layout.active,
        };
        let hovered = ui.hovered_part(self.id);
        let pressed = ui.pressed_part(self.id);
        let close_reveal_press = match source {
            TabsRenderSource::Legacy { state, .. } => state.close_reveal_press,
            TabsRenderSource::Planned(layout) => layout.close_reveal_press,
        };
        let row_count = match source {
            TabsRenderSource::Legacy { items, .. } => items.len().saturating_sub(first_index),
            TabsRenderSource::Planned(layout) => layout.rows.len(),
        };
        for offset in 0..row_count {
            let i = first_index.saturating_add(offset);
            let row_source = match source {
                TabsRenderSource::Legacy { items, .. } => TabsRowSource::Legacy(&items[i]),
                TabsRenderSource::Planned(layout) => TabsRowSource::Planned(&layout.rows[offset]),
            };
            let key = match row_source {
                TabsRowSource::Legacy(item) => self.key.key(item, i),
                TabsRowSource::Planned(row) => row.key,
            };
            let mark = match row_source {
                TabsRowSource::Legacy(item) => {
                    self.item_marks.and_then(|mapper| mapper(item).visible())
                }
                TabsRowSource::Planned(row) => row.mark,
            };
            let close_enabled = match row_source {
                TabsRowSource::Legacy(_) => self.close_enabled(key),
                TabsRowSource::Planned(row) => row.close_enabled,
            };
            let mark_width = if mark.is_some() { 2 } else { 0 };
            let close_w: u16 = if close_enabled { 2 } else { 0 };
            let avail = right_limit.saturating_sub(x);
            if matches!(row_source, TabsRowSource::Legacy(_)) && avail < 3 {
                break;
            }
            if mark.is_some() && avail < 1 + 1 + mark_width + 1 + close_w {
                break;
            }

            let is_cursor = cursor == Some(key);
            let is_active = active == Some(key);
            let mut flags = StateFlags::empty();
            if is_active {
                flags |= StateFlags::ACTIVE;
            }
            if is_cursor {
                if self.passive {
                    if self.passive_focused {
                        flags |= StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE;
                    }
                } else {
                    flags |= live & (StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE);
                }
                if pressed.is_none() {
                    flags |= live & StateFlags::PRESSED;
                }
            }
            let tab_part = PartRef::item(Part::TAB, key);
            if hovered == Some(tab_part) {
                flags |= StateFlags::HOVERED;
            }
            if pressed == Some(tab_part) {
                flags |= StateFlags::PRESSED;
            }

            let extended_row = match row_source {
                TabsRowSource::Planned(row)
                    if row.affixes.as_ref().is_some_and(|affixes| {
                        affixes.trailing_padding > 0
                            || (affixes.plate_separators && affixes.has_text_affixes())
                    }) =>
                {
                    Some(row)
                }
                TabsRowSource::Planned(_) | TabsRowSource::Legacy(_) => None,
            };
            let initialized_owned_plate = if let Some(row) = extended_row {
                let tab_style =
                    self.ov
                        .style(ui, id, Family::TABS, Variant::DEFAULT, Part::TAB, flags);
                ui.fill(row.logical_tab, Style::reset());
                ui.fill(row.logical_tab, strip.style);
                ui.paint_style(row.logical_tab, tab_style.style);
                Some(tab_style)
            } else {
                None
            };
            let owns_extended_plate = extended_row.is_some();

            let content = match row_source {
                TabsRowSource::Legacy(_) => Rect {
                    x: x.saturating_add(1),
                    y: row0.y,
                    width: avail
                        .saturating_sub(1)
                        .saturating_sub(mark_width)
                        .saturating_sub(u16::from(mark.is_some()))
                        .saturating_sub(if mark.is_some() { close_w } else { 0 }),
                    height: 1,
                },
                TabsRowSource::Planned(row) => row.logical_content,
            };
            let has_text_affixes = match row_source {
                TabsRowSource::Planned(row) => row
                    .affixes
                    .as_ref()
                    .is_some_and(OwnedTabAffixes::has_text_affixes),
                TabsRowSource::Legacy(_) => false,
            };
            let row_ui_area = match row_source {
                TabsRowSource::Planned(_) if owns_extended_plate && has_text_affixes => None,
                TabsRowSource::Planned(row) if owns_extended_plate => Some(Rect {
                    width: content.width.min(row.label_extent),
                    ..content
                }),
                TabsRowSource::Planned(_) | TabsRowSource::Legacy(_) => Some(content),
            };
            if let Some(row_ui_area) = row_ui_area {
                let mut r = RowUi::new(
                    ui,
                    id,
                    Family::TABS,
                    Variant::DEFAULT,
                    flags,
                    key,
                    row_ui_area,
                );
                match row_source {
                    TabsRowSource::Legacy(item) => self.row.row(item, &mut r),
                    TabsRowSource::Planned(row) if !has_text_affixes => {
                        r.label_fmt(format_args!("{}", DisplaySegments(&row.label_segments)));
                    }
                    TabsRowSource::Planned(_) => {}
                }
            }
            if let TabsRowSource::Planned(row) = row_source
                && let Some(affixes) = row.affixes.as_ref()
                && affixes.has_text_affixes()
            {
                let label = ui.style(Family::TABS, Variant::DEFAULT, Part::LABEL, flags);
                if affixes.plate_separators {
                    if let (Some(display_segments), Some(prefix_segments)) = (
                        row.label_segments
                            .get(affixes.display_start..affixes.display_end),
                        row.label_segments.get(..affixes.display_start),
                    ) {
                        let display_offset =
                            prefix_segments.iter().fold(0_u16, |width, segment| {
                                width.saturating_add(crate::text::width(segment))
                            });
                        let display_area = Rect {
                            x: content.x.saturating_add(display_offset),
                            width: affixes.display_width,
                            ..content
                        };
                        paint_logical_segments(
                            ui,
                            display_area,
                            row.visible_content,
                            display_segments,
                            label.style,
                        );
                    }
                } else {
                    paint_logical_segments(
                        ui,
                        content,
                        row.visible_content,
                        &row.label_segments,
                        label.style,
                    );
                }
            }
            let label_w = match row_source {
                TabsRowSource::Legacy(_) => painted_width(ui, content).max(1),
                TabsRowSource::Planned(row) => row.label_extent,
            };
            let close_part = PartRef::item(Part::CLOSE, key);
            let close_visible = close_enabled
                && match self.close_reveal {
                    TabCloseReveal::Always => true,
                    TabCloseReveal::Hover => {
                        let cursor_focused = if self.passive {
                            self.passive_focused
                        } else {
                            live.contains(StateFlags::FOCUSED)
                        };
                        is_active
                            || (is_cursor && cursor_focused)
                            || hovered == Some(tab_part)
                            || hovered == Some(close_part)
                            || pressed == Some(close_part)
                    }
                };
            let close_hit_visible = close_visible
                && (self.close_reveal != TabCloseReveal::Hover || close_reveal_press != Some(key));
            let tab = match row_source {
                TabsRowSource::Legacy(_) => {
                    let tab_w = 1u16
                        .saturating_add(label_w)
                        .saturating_add(mark_width)
                        .saturating_add(1)
                        .saturating_add(close_w);
                    Rect {
                        x,
                        y: row0.y,
                        width: tab_w.min(avail),
                        height: 1,
                    }
                }
                TabsRowSource::Planned(row) => row.logical_tab,
            };
            let tail = Rect {
                x: tab.right(),
                y: row0.y,
                width: right_limit.saturating_sub(tab.right()),
                height: 1,
            };
            ui.fill(tail, strip.style);
            let ts = initialized_owned_plate.unwrap_or_else(|| {
                ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::TAB, flags)
            });
            ui.paint_style(tab, ts.style);
            if let TabsRowSource::Planned(row) = row_source {
                self.paint_planned_affixes(ui, row, flags, ts.style);
            }

            if flags.contains(StateFlags::PRESSED) {
                let ls = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::LABEL, flags);
                if matches!(ls.glyph, Slot::Set(GlyphRole::PressLeft)) {
                    paint_pressed_bracket(
                        ui,
                        cell_at(tab, tab.x),
                        cell_at(tab, tab.x.saturating_add(1).saturating_add(label_w)),
                        ls.style,
                    );
                }
            }
            if let Some(mark) = mark {
                let mark_area = match row_source {
                    TabsRowSource::Legacy(_) => Some(Rect {
                        x: x.saturating_add(1).saturating_add(label_w),
                        y: row0.y,
                        width: 2,
                        height: 1,
                    }),
                    TabsRowSource::Planned(row) => row.logical_mark,
                };
                if let Some(mark_area) = mark_area {
                    self.paint_item_mark(ui, mark, mark_area, ts.style);
                }
            }
            let close_target = if close_enabled {
                match row_source {
                    TabsRowSource::Legacy(_) => close_target_rect(tab, self.close_target_w),
                    TabsRowSource::Planned(row) => row.logical_close,
                }
            } else {
                None
            };
            let close_glyph = close_target.map(|_| close_glyph_cell(tab));
            if let (Some(close_target), Some(close_glyph)) = (close_target, close_glyph) {
                if close_visible {
                    let mut close_flags =
                        flags.difference(StateFlags::HOVERED | StateFlags::PRESSED);
                    if hovered == Some(close_part) {
                        close_flags |= StateFlags::HOVERED;
                    }
                    if pressed == Some(close_part) {
                        close_flags |= StateFlags::PRESSED;
                    }
                    let close_is_hovered = close_flags.contains(StateFlags::HOVERED);
                    let close_is_pressed = close_flags.contains(StateFlags::PRESSED);
                    let danger_active = self.close_danger_hover
                        && !close_flags.contains(StateFlags::DISABLED)
                        && (close_is_hovered || close_is_pressed);
                    let mode_patch = if danger_active && close_is_pressed {
                        Some(
                            StylePatch::new()
                                .set_fg(Role::OnDanger)
                                .set_bg(Role::Danger),
                        )
                    } else if danger_active && close_is_hovered {
                        Some(StylePatch::new().set_fg(Role::Danger))
                    } else {
                        None
                    };
                    let full_plate_style = close_flags
                        .intersects(StateFlags::HOVERED | StateFlags::PRESSED)
                        && (self.close_target_w.is_some() || danger_active);
                    let opted_close_style =
                        self.close_target_w.is_some() || self.close_danger_hover;
                    let (cs, explicit) = if opted_close_style {
                        let local = match (mode_patch, ov.part_patch(Part::CLOSE)) {
                            (Some(mode), Some(instance)) => Some(mode.merge(instance)),
                            (Some(mode), None) => Some(mode),
                            (None, instance) => instance,
                        };
                        let resolved = ui.style_defaults_with_explicit(
                            Family::TABS,
                            Variant::DEFAULT,
                            Part::CLOSE,
                            close_flags,
                            StyleDefaults::default(),
                            local.as_ref(),
                        );
                        ov.note(
                            ui,
                            id,
                            Family::TABS,
                            Variant::DEFAULT,
                            Part::CLOSE,
                            resolved.resolved,
                        );
                        (resolved.resolved, Some(resolved.explicit))
                    } else {
                        (
                            ov.style(
                                ui,
                                id,
                                Family::TABS,
                                Variant::DEFAULT,
                                Part::CLOSE,
                                close_flags,
                            ),
                            None,
                        )
                    };
                    if full_plate_style {
                        ui.paint_style(close_target, cs.style);
                    }
                    match cs.glyph {
                        Slot::Set(glyph) => {
                            ui.glyph(close_glyph, glyph, cs.style);
                        }
                        Slot::Inherit => {
                            ui.glyph(close_glyph, GlyphRole::Close, cs.style);
                        }
                        Slot::Clear => {
                            ui.fill(close_glyph, cs.style);
                        }
                    }
                    if let Some(explicit) = explicit {
                        let explicit_area = if full_plate_style {
                            close_target
                        } else {
                            close_glyph
                        };
                        termrock_runtime::author::patch_style(
                            ui,
                            explicit_area,
                            cs.style,
                            explicit,
                        );
                    }
                } else {
                    ui.fill(close_glyph, ts.style);
                }
            }
            if is_active && let Some(rr) = rule_row {
                let (rs, explicit) = match self.rule {
                    Some(rule) => {
                        let mode = StylePatch::new().set_fg(rule.foreground(
                            ui,
                            id,
                            self.rule_focus_hidden,
                            self.passive,
                            self.passive_focused,
                        ));
                        let patch = ov
                            .part_patch(Part::RULE)
                            .map_or(mode, |local| mode.merge(local));
                        let resolved = ui.style_defaults_with_explicit(
                            Family::TABS,
                            Variant::DEFAULT,
                            Part::RULE,
                            flags,
                            StyleDefaults::default(),
                            Some(&patch),
                        );
                        ov.note(
                            ui,
                            id,
                            Family::TABS,
                            Variant::DEFAULT,
                            Part::RULE,
                            resolved.resolved,
                        );
                        (resolved.resolved, Some(resolved.explicit))
                    }
                    None => (
                        ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::RULE, flags),
                        None,
                    ),
                };
                let span = Rect {
                    x: tab.x,
                    width: tab.width,
                    ..rr
                };
                for col in span.columns() {
                    match rs.glyph {
                        Slot::Set(glyph) => {
                            ui.glyph(col, glyph, rs.style);
                        }
                        Slot::Inherit => {
                            ui.glyph(col, GlyphRole::RuleActive, rs.style);
                        }
                        Slot::Clear => {
                            ui.fill(col, rs.style);
                        }
                    }
                }
                if let Some(explicit) = explicit {
                    termrock_runtime::author::patch_style(ui, span, rs.style, explicit);
                }
            }
            if !ui.is_inert() {
                ui.register_part(self.id, PartRef::item(Part::TAB, key), tab);
            }
            if close_hit_visible
                && !ui.is_inert()
                && let Some(close_target) = close_target
            {
                ui.register_part(self.id, PartRef::item(Part::CLOSE, key), close_target);
            }
            x = tab.right().saturating_add(1);
            fit = fit.saturating_add(1);
        }
        let mut buf = [0u8; 2];
        if first_index > 0 {
            let fallback = Rect {
                x: row0.x,
                y: row0.y,
                width: left_w.min(row0.width),
                height: 1,
            };
            let cell = match source {
                TabsRenderSource::Legacy { .. } => fallback,
                TabsRenderSource::Planned(layout) => layout.left_overflow.unwrap_or(fallback),
            };
            let os = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::OVERFLOW,
                StateFlags::empty(),
            );
            let used = ui.glyph(cell, GlyphRole::OverflowLeft, os.style);
            ui.paint_str(
                super::shift(cell, used),
                digits(first_index, &mut buf),
                os.style,
            );
            if !ui.is_inert() {
                ui.register_part(
                    self.id,
                    PartRef::item(Part::OVERFLOW, ItemKey::index(0)),
                    cell,
                );
            }
        }
        let hidden_right = match source {
            TabsRenderSource::Legacy { .. } => len.saturating_sub(first_index.saturating_add(fit)),
            TabsRenderSource::Planned(layout) => layout.hidden_right,
        };
        if hidden_right > 0 {
            let fallback = Rect {
                x: row0
                    .right()
                    .saturating_sub(status_w)
                    .saturating_sub(new_w)
                    .saturating_sub(4)
                    .max(row0.x),
                y: row0.y,
                width: 4.min(row0.width),
                height: 1,
            };
            let cell = match source {
                TabsRenderSource::Legacy { .. } => fallback,
                TabsRenderSource::Planned(layout) => layout.right_overflow.unwrap_or(fallback),
            };
            let os = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::OVERFLOW,
                StateFlags::empty(),
            );
            ui.fill(cell, strip.style);
            let d = digits(hidden_right, &mut buf);
            let dw = crate::text::width(d);
            let text = Rect {
                x: cell.right().saturating_sub(1).saturating_sub(dw),
                width: dw,
                ..cell
            }
            .intersection(cell);
            ui.paint_str(text, d, os.style);
            ui.glyph(
                cell_at(cell, cell.right().saturating_sub(1)),
                GlyphRole::OverflowRight,
                os.style,
            );
            if !ui.is_inert() {
                ui.register_part(
                    self.id,
                    PartRef::item(Part::OVERFLOW, ItemKey::index(1)),
                    cell,
                );
            }
        }
        if let TabsRenderSource::Planned(layout) = source
            && let Some(cell) = layout.more_area
        {
            let menu_count = layout.first_index.saturating_add(layout.hidden_right);
            let os = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::OVERFLOW,
                StateFlags::empty(),
            );
            ui.fill(cell, strip.style);
            match os.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(cell_at(cell, cell.x), glyph, os.style);
                }
                Slot::Inherit => {
                    ui.glyph(cell_at(cell, cell.x), GlyphRole::MoreRows, os.style);
                }
                Slot::Clear => {
                    ui.fill(cell_at(cell, cell.x), os.style);
                }
            }
            let count = digits(menu_count, &mut buf);
            let count_width = crate::text::width(count);
            let count_area = Rect {
                x: cell.right().saturating_sub(1).saturating_sub(count_width),
                width: count_width,
                ..cell
            }
            .intersection(cell);
            ui.paint_str(count_area, count, os.style);
            if !ui.is_inert() {
                ui.register_part(
                    self.id,
                    PartRef::item(Part::OVERFLOW, ItemKey::index(2)),
                    cell,
                );
            }
        }
        if self.allow_new {
            let fallback = Rect {
                x: row0
                    .right()
                    .saturating_sub(status_w)
                    .saturating_sub(new_w)
                    .max(row0.x),
                y: row0.y,
                width: new_w.min(row0.width),
                height: 1,
            };
            let cell = match source {
                TabsRenderSource::Legacy { .. } => fallback,
                TabsRenderSource::Planned(layout) => layout.new_area.unwrap_or(fallback),
            };
            let ns = ov.style(
                ui,
                id,
                Family::TABS,
                Variant::DEFAULT,
                Part::NEW,
                StateFlags::empty(),
            );
            let inner = super::shift(cell, 1);
            match ns.glyph {
                Slot::Set(glyph) => {
                    ui.glyph(inner, glyph, ns.style);
                }
                Slot::Inherit => {
                    ui.glyph(inner, GlyphRole::NewTab, ns.style);
                }
                Slot::Clear => {
                    ui.fill(inner, ns.style);
                }
            }
            if !ui.is_inert() {
                ui.register_part(self.id, PartRef::of(Part::NEW), cell);
            }
        }
        if status_w > 0 {
            let fallback = Rect {
                x: row0.right().saturating_sub(status_w).max(row0.x),
                y: row0.y,
                width: 1.min(row0.width),
                height: 1,
            };
            let icon_cell = match source {
                TabsRenderSource::Legacy { .. } => fallback,
                TabsRenderSource::Planned(layout) => layout.status_area.unwrap_or(fallback),
            };
            if let Some(slot) = ov.slot_for(Part::ICON) {
                slot(ui, icon_cell);
            } else {
                let icon = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::ICON, live);
                match self.status {
                    Status::Busy | Status::Loading => {
                        let frames = ui.design().motion.spinner_frames;
                        let frame = frames.first().copied().unwrap_or("");
                        ui.paint_str(icon_cell, frame, icon.style);
                    }
                    Status::Error => match icon.glyph {
                        Slot::Set(glyph) => {
                            ui.glyph(icon_cell, glyph, icon.style);
                        }
                        Slot::Inherit => {
                            ui.glyph(icon_cell, GlyphRole::Error, icon.style);
                        }
                        Slot::Clear => ui.fill(icon_cell, icon.style),
                    },
                    Status::Ready => {}

                    _ => {}
                }
            }
        }
        let viewport_len = match source {
            TabsRenderSource::Legacy { .. } => fit,
            TabsRenderSource::Planned(layout) => layout.viewport_len,
        };
        ui.report_layout(
            self.id,
            LayoutFacts::new(viewport_len, len, used.height, used.width),
        );
        if menu_eligible
            && let TabsRenderSource::Planned(layout) = source
            && let Some(menu) = layout.menu.as_ref()
        {
            let layer_id = self.menu_layer_id();
            let panel = Panel::new(layer_id.sub("panel")).title("Hidden tabs");
            let list = List::new(layer_id.sub("items"))
                .key(tabs_menu_key as fn(&TabsMenuItem) -> ItemKey)
                .whole_rows(true);
            let _ = ui.layer(layer_id, |layer_ui, layer_area| {
                panel.draw(layer_ui, layer_area, |body, inner| {
                    list.draw(body, inner, &menu.list, &menu.items);
                });
            });
        }
        used
    }

    pub fn measure(&self, ui: &Ui<'_>, c: Constraints) -> Size {
        let h = ui.design().size.tabs_height;
        Size {
            min: (8, h),
            preferred: (c.max.0, h),
        }
        .fit(c)
    }
}

fn item_mark_glyph_area(area: Rect, glyph_width: u16) -> Rect {
    if glyph_width <= 1 {
        Rect {
            x: area.right().saturating_sub(1),
            width: area.width.min(1),
            ..area
        }
    } else {
        area
    }
}

impl<T: fmt::Display, K: KeyFn<T>> Tabs<'_, T, K, DefaultRow> {
    #[must_use]
    pub fn planned_whole_rows(mut self, enabled: bool) -> Self {
        self.planned_whole_rows = enabled;
        self
    }

    #[must_use]
    pub fn item_affixes(mut self, mapper: fn(&T) -> TabAffixes) -> Self {
        self.item_affixes = Some(mapper);
        self
    }

    fn empty_default_layout(
        &self,
        area: Rect,
        clip: Rect,
        state: &TabsState,
        len: usize,
        first_index: usize,
        overflow_reserved: bool,
    ) -> TabLayout {
        TabLayout {
            id: self.id,
            area,
            clip,
            row_area: first_row(area),
            used_area: area,
            rule_area: None,
            rows: Vec::new(),
            owned_items: None,
            terminal_first_index: None,
            geometry_follow: None,
            planned_whole_rows: self.planned_whole_rows,
            manual_window_active: state.manual_window_active,
            first_index,
            active: state.active,
            cursor: state.core.cursor(),
            first: state.first,
            first_tab_x: area.x,
            overflow_menu: self.overflow_menu,
            menu: state.menu.clone(),
            allow_new: self.allow_new,
            new_key_bindings: self.new_key_bindings,
            closable: self.closable,
            closable_keys: self.closable_keys.map(<[ItemKey]>::to_vec),
            close_key_bindings: self.close_key_bindings,
            close_reveal: self.close_reveal,
            close_target_w: self.close_target_w,
            close_danger_hover: self.close_danger_hover,
            close_reveal_press: state.close_reveal_press,
            status: self.status,
            rule: self.rule,
            rule_focus_hidden: self.rule_focus_hidden,
            passive: self.passive,
            passive_focused: self.passive_focused,
            left_overflow: None,
            right_overflow: None,
            more_area: None,
            new_area: None,
            status_area: None,
            item_marks_enabled: self.item_marks.is_some(),
            item_affixes_enabled: self.item_affixes.is_some(),
            right_limit: area.x,
            hidden_right: len.saturating_sub(first_index),
            logical_area: area,
            visible_area: area.intersection(clip),
            viewport_len: 0,
            content_len: len,
            overflow_reserved,
        }
    }

    pub fn layout_default(
        &self,
        area: Rect,
        clip: Rect,
        state: &TabsState,
        items: &[T],
        previous: Option<TabWindowReceipt>,
    ) -> TabLayout {
        let len = items.len();
        if area.is_empty() {
            let first_index = state
                .first
                .and_then(|first| self.index_of(items, first, Some(state.first_index)))
                .unwrap_or(0)
                .min(len);
            let overflow_reserved =
                previous.is_some_and(|receipt| receipt.viewport_len < receipt.content_len);
            return self.empty_default_layout(
                area,
                clip,
                state,
                len,
                first_index,
                overflow_reserved,
            );
        }
        let owned_items: Vec<_> = items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let key = self.key.key(item, index);
                let mark = self.item_marks.and_then(|mapper| mapper(item).visible());
                let close_enabled = self.close_enabled(key);
                let mut writer = SegmentWriter(Vec::new());
                let _ = fmt::write(&mut writer, format_args!("{item}"));
                let affixes = self
                    .item_affixes
                    .map_or_else(TabAffixes::default, |mapper| mapper(item));
                let (label_segments, affixes) = capture_label_affixes(writer.0, affixes);
                OwnedTabMeasurement::new_with_natural_extent(
                    key,
                    mark,
                    close_enabled,
                    label_segments,
                    affixes,
                    self.planned_whole_rows,
                )
            })
            .collect();
        let len = owned_items.len();
        let first_index = state
            .first
            .map(|first| {
                let hint = state.first_index;
                if owned_items.get(hint).is_some_and(|item| item.key == first) {
                    hint
                } else {
                    owned_items
                        .iter()
                        .position(|item| item.key == first)
                        .unwrap_or(0)
                }
            })
            .unwrap_or(0)
            .min(len);

        let mut extent_scratch = Vec::new();
        let terminal_first_index =
            self.find_terminal_first_index(area, clip, &owned_items, previous, &mut extent_scratch);

        let requested_first_index = first_index;
        let manual_window_owned = state.manual_window_active == Some(state.active);
        let geometry_follow = if manual_window_owned {
            None
        } else {
            self.refine_active_window(
                OwnedTabsWalk {
                    area,
                    clip,
                    items: &owned_items,
                    previous,
                },
                state.active,
                first_index,
                &mut extent_scratch,
            )
        };
        let first_index = if manual_window_owned {
            terminal_first_index.map_or(first_index, |terminal| first_index.min(terminal))
        } else {
            geometry_follow.map_or(first_index, |follow| follow.first_index)
        };
        let mut adjusted_state = None;
        if first_index != requested_first_index {
            let mut planned = state.clone();
            planned.first_index = first_index;
            planned.first = owned_items.get(first_index).map(|item| item.key);
            adjusted_state = Some(planned);
        }
        let state_for_layout = adjusted_state.as_ref().map_or(state, |planned| planned);
        let mut layout = self.layout_default_from_owned(
            area,
            clip,
            state_for_layout,
            &owned_items,
            first_index,
            previous,
            &mut extent_scratch,
        );
        layout.owned_items = Some(owned_items);
        layout.terminal_first_index = terminal_first_index;
        layout.geometry_follow = geometry_follow;
        layout
    }

    fn walk_active_row(
        &self,
        walk: OwnedTabsWalk<'_>,
        first_index: usize,
        active_index: usize,
        extent_scratch: &mut Vec<bool>,
    ) -> Option<DefaultRowGeometry> {
        let mut active = None;
        let _ = self.walk_default_resolved(
            walk.area,
            walk.clip,
            walk.items,
            first_index,
            walk.previous,
            extent_scratch,
            |row| {
                if row.index == active_index {
                    active = Some(row);
                }
            },
        );
        active
    }

    fn refine_active_window(
        &self,
        walk: OwnedTabsWalk<'_>,
        active_key: Option<ItemKey>,
        requested_first: usize,
        extent_scratch: &mut Vec<bool>,
    ) -> Option<TabsGeometryFollow> {
        walk.previous?;
        let items = walk.items;
        let active_key = active_key?;
        let active_index = items.iter().position(|item| item.key == active_key)?;
        if active_index < requested_first {
            return None;
        }
        let requested_row =
            self.walk_active_row(walk, requested_first, active_index, extent_scratch);
        if requested_row.is_some_and(|row| !row.visible_tab.is_empty()) {
            return None;
        }

        let attainable = self.walk_active_row(walk, active_index, active_index, extent_scratch)?;
        if attainable.visible_tab.is_empty() {
            return None;
        }

        let first_candidate = requested_first.min(active_index);
        let first_index = (first_candidate..=active_index).find(|&candidate_first| {
            self.walk_active_row(walk, candidate_first, active_index, extent_scratch)
                .is_some_and(|row| {
                    !row.visible_tab.is_empty()
                        && row.visible_tab.width >= attainable.visible_tab.width
                        && row.label_extent >= attainable.label_extent
                })
        })?;
        if first_index == requested_first {
            return None;
        }
        Some(TabsGeometryFollow {
            active_key,
            first_key: items.get(first_index)?.key,
            first_index,
        })
    }

    fn find_terminal_first_index(
        &self,
        area: Rect,
        clip: Rect,
        items: &[OwnedTabMeasurement],
        previous: Option<TabWindowReceipt>,
        extent_scratch: &mut Vec<bool>,
    ) -> Option<usize> {
        let len = items.len();
        if len == 0 {
            return None;
        }

        let final_index = len - 1;
        let reaches_final = |walk: DefaultRowWalk| {
            walk.last_row
                .is_some_and(|row| row.index == final_index && !row.visible_tab.is_empty())
        };

        let start_zero =
            self.walk_default_resolved(area, clip, items, 0, previous, extent_scratch, |_| {});
        if reaches_final(start_zero) {
            return Some(0);
        }

        let reserve_more = self.overflow_menu && self.more_lane_fits(area, clip, 1, previous);
        let positive_header = self.default_row_header_with_more(area, 1, previous, reserve_more);
        let max_rows = max_rows_for_width_budget(positive_header.available_width());
        let terminal = positive_terminal_candidate_range(len, max_rows).find(|&candidate_first| {
            let walk = self.walk_default_resolved(
                area,
                clip,
                items,
                candidate_first,
                previous,
                extent_scratch,
                |_| {},
            );
            reaches_final(walk)
        });
        if terminal.is_some() || !self.planned_whole_rows || len <= 1 {
            return terminal;
        }

        let final_index = len - 1;
        let final_walk = self.walk_default_resolved(
            area,
            clip,
            items,
            final_index,
            previous,
            extent_scratch,
            |_| {},
        );
        let final_item = items.get(final_index)?;
        let available = final_walk
            .right_limit
            .saturating_sub(final_walk.first_tab_x);
        let minimum = whole_row_minimum_width(final_item);
        let minimum_width = u16::try_from(minimum).ok()?;
        let minimum_row = Rect {
            x: final_walk.first_tab_x,
            y: final_walk.row_area.y,
            width: minimum_width,
            height: 1,
        };
        if final_walk.row_area.intersection(clip).is_empty()
            || minimum > u64::from(available)
            || minimum_row.intersection(clip).is_empty()
            || whole_row_required_width(final_item) <= u64::from(available)
        {
            return None;
        }
        Some(final_index)
    }

    fn default_row_header(
        &self,
        area: Rect,
        first_index: usize,
        previous: Option<TabWindowReceipt>,
    ) -> DefaultRowHeader {
        self.default_row_header_with_more(area, first_index, previous, false)
    }

    fn default_row_header_with_more(
        &self,
        area: Rect,
        first_index: usize,
        previous: Option<TabWindowReceipt>,
        reserve_more: bool,
    ) -> DefaultRowHeader {
        debug_assert!(!area.is_empty(), "empty geometry has no row header");
        let overflow_reserved =
            previous.is_some_and(|receipt| receipt.viewport_len < receipt.content_len);
        let status_w: u16 = if matches!(self.status, Status::Ready) {
            0
        } else {
            2
        };
        let new_w: u16 = if self.allow_new { 4 } else { 0 };
        let left_w: u16 = if first_index > 0 { 4 } else { 0 };
        let more_w: u16 = if reserve_more && self.overflow_menu {
            4
        } else {
            0
        };
        let right_w: u16 = if overflow_reserved || first_index > 0 || more_w > 0 {
            4
        } else {
            0
        };
        let row_area = first_row(area);
        let rule_area = (area.height >= 2).then(|| Rect {
            y: area.y.saturating_add(1),
            height: 1,
            ..row_area
        });
        let first_tab_x = row_area.x.saturating_add(left_w);
        let right_limit = row_area
            .right()
            .saturating_sub(status_w)
            .saturating_sub(more_w)
            .saturating_sub(right_w)
            .saturating_sub(new_w);
        let used_area = Rect {
            height: area.height.min(2),
            ..area
        };
        let left_overflow = (first_index > 0).then(|| Rect {
            x: row_area.x,
            y: row_area.y,
            width: left_w.min(row_area.width),
            height: 1,
        });
        let more_area = (more_w > 0).then(|| Rect {
            x: row_area
                .right()
                .saturating_sub(status_w)
                .saturating_sub(new_w)
                .saturating_sub(more_w),
            y: row_area.y,
            width: more_w,
            height: 1,
        });
        let right_overflow_lane = (right_w > 0).then(|| Rect {
            x: row_area
                .right()
                .saturating_sub(status_w)
                .saturating_sub(new_w)
                .saturating_sub(more_w)
                .saturating_sub(right_w),
            y: row_area.y,
            width: right_w,
            height: 1,
        });

        DefaultRowHeader {
            row_area,
            used_area,
            rule_area,
            first_tab_x,
            right_limit,
            left_overflow,
            right_overflow_lane,
            more_area,
            status_w,
            new_w,
        }
    }

    fn more_lane_fits(
        &self,
        area: Rect,
        clip: Rect,
        first_index: usize,
        previous: Option<TabWindowReceipt>,
    ) -> bool {
        if !self.overflow_menu || area.is_empty() {
            return false;
        }
        let base = self.default_row_header(area, first_index, previous);
        let required = if first_index > 0 { 4_u32 } else { 0_u32 }
            .saturating_add(4)
            .saturating_add(4)
            .saturating_add(u32::from(base.new_w))
            .saturating_add(u32::from(base.status_w))
            .saturating_add(3);
        if u32::from(base.row_area.width) < required {
            return false;
        }
        self.default_row_header_with_more(area, first_index, previous, true)
            .more_area
            .is_some_and(|more| !more.intersection(clip).is_empty())
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one shared geometry walk feeds both the scalar probe and final row sink"
    )]
    fn walk_default_owned(
        &self,
        area: Rect,
        clip: Rect,
        items: &[OwnedTabMeasurement],
        first_index: usize,
        previous: Option<TabWindowReceipt>,
        extent_scratch: &mut Vec<bool>,
        emit: impl FnMut(DefaultRowGeometry),
    ) -> DefaultRowWalk {
        self.walk_default_owned_with_more(
            area,
            clip,
            items,
            first_index,
            previous,
            extent_scratch,
            false,
            emit,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one shared geometry walk feeds scalar probes and the final row sink"
    )]
    fn walk_default_owned_with_more(
        &self,
        area: Rect,
        clip: Rect,
        items: &[OwnedTabMeasurement],
        first_index: usize,
        previous: Option<TabWindowReceipt>,
        extent_scratch: &mut Vec<bool>,
        reserve_more: bool,
        mut emit: impl FnMut(DefaultRowGeometry),
    ) -> DefaultRowWalk {
        debug_assert!(
            !area.is_empty(),
            "empty geometry is handled before the walk"
        );
        let len = items.len();
        let first_index = first_index.min(len);
        let header = self.default_row_header_with_more(area, first_index, previous, reserve_more);
        let row_area = header.row_area;
        let first_tab_x = header.first_tab_x;
        let right_limit = header.right_limit;
        let used_area = header.used_area;
        let status_w = header.status_w;
        let new_w = header.new_w;

        let mut x = first_tab_x;
        let mut viewport_len = 0;
        let mut last_row = None;
        for (index, item) in items.iter().enumerate().skip(first_index) {
            let mark = item.mark;
            let mark_width = if mark.is_some() { 2 } else { 0 };
            let close_enabled = item.close_enabled;
            let close_w: u16 = if close_enabled { 2 } else { 0 };
            let avail = right_limit.saturating_sub(x);
            let requested_padding = item
                .affixes
                .as_ref()
                .map_or(0, |affixes| affixes.trailing_padding);
            let minimum_tab_width = 3_u16.saturating_add(mark_width).saturating_add(close_w);
            let reserved_padding = requested_padding.min(avail.saturating_sub(minimum_tab_width));
            if self.planned_whole_rows && whole_row_required_width(item) > u64::from(avail) {
                break;
            }
            if avail < 3 {
                break;
            }
            if mark.is_some() && avail < 1 + 1 + mark_width + 1 + close_w {
                break;
            }
            let content = Rect {
                x: x.saturating_add(1),
                y: row_area.y,
                width: avail
                    .saturating_sub(1)
                    .saturating_sub(mark_width)
                    .saturating_sub(u16::from(mark.is_some()))
                    .saturating_sub(if mark.is_some() || requested_padding > 0 {
                        close_w
                    } else {
                        0
                    })
                    .saturating_sub(reserved_padding),
                height: 1,
            };
            let label_extent = if self.planned_whole_rows {
                u16::try_from(item.natural_label_extent().max(1)).unwrap_or(u16::MAX)
            } else if item
                .affixes
                .as_ref()
                .is_some_and(OwnedTabAffixes::has_text_affixes)
            {
                logical_label_extent(&item.label_segments, content)
            } else {
                semantic_label_extent_with_scratch(
                    &item.label_segments,
                    content,
                    clip,
                    extent_scratch,
                )
            }
            .max(1);
            let semantic_tab_width = 1_u16
                .saturating_add(label_extent)
                .saturating_add(mark_width)
                .saturating_add(1)
                .saturating_add(close_w);
            let trailing_padding = if self.planned_whole_rows {
                requested_padding
            } else {
                reserved_padding.min(avail.saturating_sub(semantic_tab_width))
            };
            let tab_w = 1u16
                .saturating_add(label_extent)
                .saturating_add(mark_width)
                .saturating_add(1)
                .saturating_add(trailing_padding)
                .saturating_add(close_w);
            let tab = Rect {
                x,
                y: row_area.y,
                width: tab_w.min(avail),
                height: 1,
            };
            let close = close_enabled
                .then(|| close_target_rect(tab, self.close_target_w))
                .flatten();
            let logical_mark = mark.map(|_| Rect {
                x: x.saturating_add(1).saturating_add(label_extent),
                y: row_area.y,
                width: 2,
                height: 1,
            });
            let row = DefaultRowGeometry {
                key: item.key,
                index,
                logical_content: content,
                visible_content: content.intersection(clip),
                logical_tab: tab,
                visible_tab: tab.intersection(clip),
                logical_close: close,
                visible_close: close.map(|rect| rect.intersection(clip)),
                logical_mark,
                visible_mark: logical_mark.map(|rect| rect.intersection(clip)),
                mark,
                close_enabled,
                label_extent,
            };
            emit(row);
            last_row = Some(row);
            viewport_len += 1;
            x = tab.right().saturating_add(1);
        }

        let hidden_right = len.saturating_sub(first_index.saturating_add(viewport_len));
        let right_overflow = (hidden_right > 0).then(|| {
            if reserve_more {
                header.right_overflow_lane.unwrap_or_default()
            } else {
                Rect {
                    x: row_area
                        .right()
                        .saturating_sub(status_w)
                        .saturating_sub(new_w)
                        .saturating_sub(4)
                        .max(row_area.x),
                    y: row_area.y,
                    width: 4.min(row_area.width),
                    height: 1,
                }
            }
        });
        let new_area = self.allow_new.then(|| Rect {
            x: row_area
                .right()
                .saturating_sub(status_w)
                .saturating_sub(new_w)
                .max(row_area.x),
            y: row_area.y,
            width: new_w.min(row_area.width),
            height: 1,
        });
        let status_area = (status_w > 0).then(|| Rect {
            x: row_area.right().saturating_sub(status_w).max(row_area.x),
            y: row_area.y,
            width: 1.min(row_area.width),
            height: 1,
        });
        DefaultRowWalk {
            row_area,
            used_area,
            rule_area: header.rule_area,
            first_tab_x,
            right_limit,
            left_overflow: header.left_overflow,
            right_overflow,
            more_area: header.more_area,
            new_area,
            status_area,
            hidden_right,
            viewport_len,
            last_row,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one resolved walk derives More eligibility then emits from the same owned segments"
    )]
    fn walk_default_resolved(
        &self,
        area: Rect,
        clip: Rect,
        items: &[OwnedTabMeasurement],
        first_index: usize,
        previous: Option<TabWindowReceipt>,
        extent_scratch: &mut Vec<bool>,
        emit: impl FnMut(DefaultRowGeometry),
    ) -> DefaultRowWalk {
        let base = self.walk_default_owned(
            area,
            clip,
            items,
            first_index,
            previous,
            extent_scratch,
            |_| {},
        );
        let reserve_more = first_index.saturating_add(base.hidden_right) > 0
            && self.more_lane_fits(area, clip, first_index, previous);
        self.walk_default_owned_with_more(
            area,
            clip,
            items,
            first_index,
            previous,
            extent_scratch,
            reserve_more,
            emit,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one pure allocation pass consumes the complete current default Tabs configuration"
    )]
    fn layout_default_from_owned(
        &self,
        area: Rect,
        clip: Rect,
        state: &TabsState,
        items: &[OwnedTabMeasurement],
        first_index: usize,
        previous: Option<TabWindowReceipt>,
        extent_scratch: &mut Vec<bool>,
    ) -> TabLayout {
        let len = items.len();
        let first_index = first_index.min(len);
        let overflow_reserved =
            previous.is_some_and(|receipt| receipt.viewport_len < receipt.content_len);
        if area.is_empty() {
            return TabLayout {
                id: self.id,
                area,
                clip,
                row_area: first_row(area),
                used_area: area,
                rule_area: None,
                rows: Vec::new(),
                owned_items: None,
                terminal_first_index: None,
                geometry_follow: None,
                planned_whole_rows: self.planned_whole_rows,
                manual_window_active: state.manual_window_active,
                first_index,
                active: state.active,
                cursor: state.core.cursor(),
                first: state.first,
                first_tab_x: area.x,
                overflow_menu: self.overflow_menu,
                menu: state.menu.clone(),
                allow_new: self.allow_new,
                new_key_bindings: self.new_key_bindings,
                closable: self.closable,
                closable_keys: self.closable_keys.map(<[ItemKey]>::to_vec),
                close_key_bindings: self.close_key_bindings,
                close_reveal: self.close_reveal,
                close_target_w: self.close_target_w,
                close_danger_hover: self.close_danger_hover,
                close_reveal_press: state.close_reveal_press,
                status: self.status,
                rule: self.rule,
                rule_focus_hidden: self.rule_focus_hidden,
                passive: self.passive,
                passive_focused: self.passive_focused,
                left_overflow: None,
                right_overflow: None,
                more_area: None,
                new_area: None,
                status_area: None,
                item_marks_enabled: self.item_marks.is_some(),
                item_affixes_enabled: self.item_affixes.is_some(),
                right_limit: area.x,
                hidden_right: len.saturating_sub(first_index),
                logical_area: area,
                visible_area: area.intersection(clip),
                viewport_len: 0,
                content_len: len,
                overflow_reserved,
            };
        }
        let mut rows = Vec::new();
        let walk = self.walk_default_resolved(
            area,
            clip,
            items,
            first_index,
            previous,
            extent_scratch,
            |row| {
                let item = &items[row.index];
                rows.push(row.into_tab_row(item.label_segments.clone(), item.affixes.clone()));
            },
        );
        TabLayout {
            id: self.id,
            area,
            clip,
            row_area: walk.row_area,
            used_area: walk.used_area,
            rule_area: walk.rule_area,
            rows,
            owned_items: None,
            terminal_first_index: None,
            geometry_follow: None,
            planned_whole_rows: self.planned_whole_rows,
            manual_window_active: state.manual_window_active,
            first_index,
            active: state.active,
            cursor: state.core.cursor(),
            first: state.first,
            first_tab_x: walk.first_tab_x,
            overflow_menu: self.overflow_menu,
            menu: state.menu.clone(),
            allow_new: self.allow_new,
            new_key_bindings: self.new_key_bindings,
            closable: self.closable,
            closable_keys: self.closable_keys.map(<[ItemKey]>::to_vec),
            close_key_bindings: self.close_key_bindings,
            close_reveal: self.close_reveal,
            close_target_w: self.close_target_w,
            close_danger_hover: self.close_danger_hover,
            close_reveal_press: state.close_reveal_press,
            status: self.status,
            rule: self.rule,
            rule_focus_hidden: self.rule_focus_hidden,
            passive: self.passive,
            passive_focused: self.passive_focused,
            left_overflow: walk.left_overflow,
            right_overflow: walk.right_overflow,
            more_area: walk.more_area,
            new_area: walk.new_area,
            status_area: walk.status_area,
            item_marks_enabled: self.item_marks.is_some(),
            item_affixes_enabled: self.item_affixes.is_some(),
            right_limit: walk.right_limit,
            hidden_right: walk.hidden_right,
            logical_area: walk.used_area,
            visible_area: walk.used_area.intersection(clip),
            viewport_len: walk.viewport_len,
            content_len: len,
            overflow_reserved,
        }
    }

    pub fn draw_layout(
        &self,
        ui: &mut Ui<'_>,
        layout: &TabLayout,
    ) -> Result<Rect, TabLayoutMismatch> {
        if !layout.owned_snapshot_is_consistent()
            || layout.id != self.id
            || layout.planned_whole_rows != self.planned_whole_rows
            || layout.overflow_menu != self.overflow_menu
            || layout.allow_new != self.allow_new
            || layout.new_key_bindings != self.new_key_bindings
            || layout.closable != self.closable
            || layout.closable_keys.as_deref() != self.closable_keys
            || layout.close_key_bindings != self.close_key_bindings
            || layout.close_reveal != self.close_reveal
            || layout.close_target_w != self.close_target_w
            || layout.close_danger_hover != self.close_danger_hover
            || layout.status != self.status
            || layout.rule != self.rule
            || layout.rule_focus_hidden != self.rule_focus_hidden
            || layout.passive != self.passive
            || layout.passive_focused != self.passive_focused
            || layout.item_marks_enabled != self.item_marks.is_some()
            || layout.item_affixes_enabled != self.item_affixes.is_some()
        {
            *ui.cache::<TabsMenuFrame>(self.id) = TabsMenuFrame::default();
            return Err(TabLayoutMismatch);
        }
        Ok(self.draw_impl(ui, layout.area, TabsRenderSource::Planned(layout)))
    }
}

const fn normalized_close_target_width(requested: u16) -> u16 {
    let pad = requested.saturating_sub(1) / 2;
    1u16.saturating_add(pad.saturating_mul(2))
}

fn close_target_rect(tab: Rect, requested: Option<u16>) -> Option<Rect> {
    match requested {
        None => Some(cell_at(tab, tab.right().saturating_sub(2))),
        Some(_) if tab.is_empty() => None,
        Some(requested) => {
            let width = normalized_close_target_width(requested).min(tab.width);
            if width == 0 {
                return None;
            }
            let pad = normalized_close_target_width(requested).saturating_sub(1) / 2;
            let glyph_x = tab.right().saturating_sub(2);
            let max_left = tab.right().saturating_sub(width).max(tab.x);
            let left = glyph_x.saturating_sub(pad).max(tab.x).min(max_left);
            Some(Rect {
                x: left,
                y: tab.y,
                width,
                height: tab.height,
            })
        }
    }
}

fn close_glyph_cell(tab: Rect) -> Rect {
    cell_at(tab, tab.right().saturating_sub(2))
}

fn paint_logical_text(
    ui: &mut Ui<'_>,
    area: Rect,
    clip: Rect,
    text: &str,
    style: crate::theme::PaintStyle,
) {
    let visible = area.intersection(clip).intersection(ui.full());
    if visible.is_empty() || area.y < visible.y || area.y >= visible.bottom() {
        return;
    }

    let mut x = area.x;
    let area_right = area.right();
    for (_, grapheme) in crate::text::measure::graphemes(text) {
        let width = crate::text::measure::grapheme_width(grapheme);
        if width == 0 {
            continue;
        }
        let right = x.saturating_add(width);

        if x >= visible.x && right <= visible.right() && right <= area_right {
            ui.paint_str(Rect::new(x, area.y, width, 1), grapheme, style);
        }
        x = right;
    }
}

fn paint_logical_segments(
    ui: &mut Ui<'_>,
    area: Rect,
    clip: Rect,
    segments: &[String],
    style: crate::theme::PaintStyle,
) {
    let mut x = area.x;
    let area_right = area.right();
    for segment in segments {
        let width = crate::text::width(segment);
        if width > 0 && x < area_right {
            let segment_area = Rect::new(x, area.y, width.min(area_right.saturating_sub(x)), 1);
            paint_logical_text(ui, segment_area, clip, segment, style);
        }
        x = x.saturating_add(width);
    }
}

fn natural_semantic_label_extent(segments: &[String]) -> u64 {
    let mut x = 0_u64;
    let mut last_nonblank_end = 0_u64;
    for segment in segments {
        for (_, grapheme) in termrock_text::measure::graphemes(segment) {
            if grapheme.contains(char::is_control) {
                continue;
            }
            let width = u64::from(grapheme.cell_width());
            if width == 0 {
                continue;
            }
            x = x.saturating_add(width);
            if grapheme != " " {
                last_nonblank_end = x;
            }
        }
    }
    last_nonblank_end
}

fn whole_row_required_width(item: &OwnedTabMeasurement) -> u64 {
    let label = item.natural_label_extent().max(1);
    let mark = if item.mark.is_some() { 2_u64 } else { 0 };
    let close = if item.close_enabled { 2_u64 } else { 0 };
    let padding = item
        .affixes
        .as_ref()
        .map_or(0_u64, |affixes| u64::from(affixes.trailing_padding));
    2_u64
        .saturating_add(label)
        .saturating_add(mark)
        .saturating_add(close)
        .saturating_add(padding)
}

fn whole_row_minimum_width(item: &OwnedTabMeasurement) -> u64 {
    let mark = if item.mark.is_some() { 2_u64 } else { 0 };
    let close = if item.close_enabled { 2_u64 } else { 0 };
    3_u64.saturating_add(mark).saturating_add(close)
}

fn logical_label_extent(segments: &[String], content: Rect) -> u16 {
    let content_right = content.right();
    let mut x = content.x;
    let mut last_nonblank_end = content.x;

    'segments: for segment in segments {
        for (_, grapheme) in crate::text::measure::graphemes(segment) {
            let width = crate::text::measure::grapheme_width(grapheme);
            if width == 0 {
                continue;
            }
            let Some(remaining) = content_right.checked_sub(x) else {
                break 'segments;
            };
            if width > remaining {
                break 'segments;
            }
            x = x.saturating_add(width);
            if grapheme != " " {
                last_nonblank_end = x;
            }
        }
    }

    last_nonblank_end.saturating_sub(content.x)
}

fn semantic_label_extent_with_scratch(
    segments: &[String],
    content: Rect,
    clip: Rect,
    cells: &mut Vec<bool>,
) -> u16 {
    let visible = content.intersection(clip);
    if visible.is_empty() {
        return 0;
    }

    cells.clear();
    cells.resize(usize::from(visible.width), false);
    let mut writer_x = content.x;
    let content_right = content.right();

    for segment in segments {
        if writer_x >= content_right {
            break;
        }
        let write_x = writer_x.max(clip.x).max(content.x);
        let write_right = content_right.min(clip.right());
        if write_x >= write_right || content.y < clip.y || content.y >= clip.bottom() {
            continue;
        }

        let mut x = write_x;
        let mut remaining = write_right.saturating_sub(write_x);
        for (_, grapheme) in termrock_text::measure::graphemes(segment) {
            if grapheme.contains(char::is_control) {
                continue;
            }
            let width = grapheme.cell_width();
            if width == 0 {
                continue;
            }
            let Some(next_remaining) = remaining.checked_sub(width) else {
                break;
            };

            let offset = usize::from(x.saturating_sub(visible.x));
            if let Some(cell) = cells.get_mut(offset) {
                *cell = grapheme != " ";
            }
            for continuation in 1..width {
                let offset = usize::from(x.saturating_add(continuation).saturating_sub(visible.x));
                if let Some(cell) = cells.get_mut(offset) {
                    *cell = false;
                }
            }

            remaining = next_remaining;
            x = x.saturating_add(width);
            if remaining == 0 {
                break;
            }
        }
        writer_x = writer_x.saturating_add(x.saturating_sub(write_x));
    }

    cells
        .iter()
        .rposition(|nonblank| *nonblank)
        .map_or(0, |index| u16::try_from(index + 1).unwrap_or(u16::MAX))
}

fn painted_width(ui: &mut Ui<'_>, row: Rect) -> u16 {
    ui.with_area(row, |ui| {
        let (buf, clip) = ui.raw();
        let mut last = 0u16;
        for x in clip.columns().map(|c| c.x) {
            if let Some(c) = buf.cell(Position::new(x, clip.y))
                && c.symbol() != " "
            {
                last = x.saturating_sub(clip.x).saturating_add(1);
            }
        }
        last
    })
}

impl<T, K, R> Bindings for Tabs<'_, T, K, R> {
    type Cmd = TabsCmd;

    fn bindings(&self, _s: BindingState) -> &'static [Binding<TabsCmd>] {
        self.table()
    }
}
